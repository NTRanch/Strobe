//! The transport seam (SPEC §4 B1): a mock that records every
//! outbound command for assertion, and a real HTTP transport
//! speaking the wire contract the Lovense project has proven:
//! `POST {scheme}://{host}:{port}/command`, JSON body, `X-platform`
//! header required; Function apiVer 1; Pattern apiVer 2 (rule
//! `V:1;F:<features>;S:<ms>#`); default ports 20010 (HTTP) /
//! 30010 (HTTPS) — host and port are configuration, never
//! constants treated as truth. The bridge holds no credentials:
//! the local Game Mode route needs no developer token.
//!
//! The real transport is compiled in but config-gated off in the
//! slice build (the game constructs it only when its settings name
//! a host). It speaks plain HTTP on the local route; TLS is out of
//! this crate's scope and requesting it is a construction error,
//! not a silent downgrade.

use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// One outbound command, as the gate decided it. The mock records
/// these verbatim; the real transport serializes them onto the
/// wire.
#[derive(Clone, Debug, PartialEq)]
pub enum WireCommand {
    /// Function apiVer 1: one strength sample.
    Function {
        /// Strength, already gate-clamped to 0..=20.
        strength: i32,
    },
    /// Pattern apiVer 2: a strength per step.
    Pattern {
        /// Per-step strengths, gate-clamped, at most 50.
        strengths: Vec<i32>,
        /// Step interval in milliseconds, above 100.
        step_ms: u32,
    },
    /// Zero every feature now.
    StopAll,
}

impl WireCommand {
    /// The JSON body for the wire contract.
    #[must_use]
    pub fn to_wire_json(&self) -> String {
        match self {
            WireCommand::Function { strength } => format!(
                "{{\"command\":\"Function\",\"parameter\":{{\"vibrate\":{strength},\"timeSec\":0}},\"apiVer\":1}}"
            ),
            WireCommand::Pattern { strengths, step_ms } => {
                let strength_list = strengths
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(";");
                let total_ms = strengths.len() as u64 * u64::from(*step_ms);
                let total_sec = (total_ms as f64) / 1_000.0;
                format!(
                    "{{\"command\":\"Pattern\",\"parameter\":{{\"rule\":\"V:1;F:v;S:{step_ms}#\",\"strength\":\"{strength_list}\",\"timeSec\":{total_sec}}},\"apiVer\":2}}"
                )
            }
            WireCommand::StopAll => {
                "{\"command\":\"Function\",\"parameter\":{\"vibrate\":0,\"timeSec\":0},\"apiVer\":1}"
                    .to_owned()
            }
        }
    }
}

/// A transport failure. The worker treats any failure as a drop:
/// output zeroes and reconnect restores discovery only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportError {
    /// The transport is not connected.
    NotConnected,
    /// The peer answered with a non-success HTTP status.
    HttpStatus(u16),
    /// The request could not be written or read.
    Io(String),
    /// Configuration asked for something this crate does not do
    /// (TLS); surfaced at construction, never silently downgraded.
    Unsupported(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::NotConnected => write!(formatter, "transport is not connected"),
            TransportError::HttpStatus(status) => {
                write!(formatter, "peer answered HTTP {status}")
            }
            TransportError::Io(message) => write!(formatter, "transport IO: {message}"),
            TransportError::Unsupported(message) => {
                write!(formatter, "unsupported transport option: {message}")
            }
        }
    }
}

impl std::error::Error for TransportError {}

/// A command channel to the toy hardware (or the mock).
pub trait Transport: Send {
    /// Emit one command.
    ///
    /// # Errors
    /// [`TransportError`] on any failure; callers treat every
    /// failure as a connection drop.
    fn send_command(&mut self, command: &WireCommand) -> Result<(), TransportError>;

    /// True while the transport believes it is connected.
    fn is_connected(&self) -> bool;
}

/// The mock transport: records every outbound command, in order,
/// for assertion (the MCP project's TESTING.md ladder, ported).
#[derive(Clone, Debug, Default)]
pub struct MockTransport {
    log: Arc<Mutex<Vec<WireCommand>>>,
    connected: bool,
}

impl MockTransport {
    /// A fresh, connected mock with an empty log.
    #[must_use]
    pub fn new() -> MockTransport {
        MockTransport {
            log: Arc::new(Mutex::new(Vec::new())),
            connected: true,
        }
    }

    /// A shared handle to the command log, for assertions from
    /// outside the worker thread.
    #[must_use]
    pub fn log_handle(&self) -> Arc<Mutex<Vec<WireCommand>>> {
        Arc::clone(&self.log)
    }

    /// Every command emitted so far, in order.
    #[must_use]
    pub fn log(&self) -> Vec<WireCommand> {
        self.log.lock().map(|log| log.clone()).unwrap_or_default()
    }

    /// Simulate the connection dropping.
    pub fn simulate_drop(&mut self) {
        self.connected = false;
    }
}

impl Transport for MockTransport {
    fn send_command(&mut self, command: &WireCommand) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        if let Ok(mut log) = self.log.lock() {
            log.push(command.clone());
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Configuration for the real HTTP transport. Host and port are
/// configuration, never constants treated as truth (§4).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealTransportConfig {
    /// Host of the local Lovense route.
    pub host: String,
    /// Port of the local route (default 20010 HTTP).
    pub port: u16,
    /// Value of the required `X-platform` header.
    pub platform: String,
    /// TLS is not implemented by this crate; setting this is a
    /// construction error, never a silent downgrade.
    pub use_tls: bool,
}

impl Default for RealTransportConfig {
    fn default() -> Self {
        RealTransportConfig {
            host: "127.0.0.1".to_owned(),
            port: 20_010,
            platform: "Strobe".to_owned(),
            use_tls: false,
        }
    }
}

/// The real transport: HTTP/1.1 over a TCP stream to the configured
/// local route. One request per connection (the local route is a
/// command endpoint; connection reuse buys nothing here).
pub struct RealTransport {
    config: RealTransportConfig,
    connected: bool,
}

impl RealTransport {
    /// Build a real transport. Does not connect until
    /// [`RealTransport::connect`] is called.
    ///
    /// # Errors
    /// [`TransportError::Unsupported`] when `use_tls` is set.
    pub fn new(config: RealTransportConfig) -> Result<RealTransport, TransportError> {
        if config.use_tls {
            return Err(TransportError::Unsupported(
                "TLS is out of scope for the slice transport; use the local HTTP route".to_owned(),
            ));
        }
        Ok(RealTransport {
            config,
            connected: false,
        })
    }

    /// Open the route (a zero-length probe: the first real command
    /// performs the actual POST; `connect` records intent and
    /// verifies reachability with a TCP handshake).
    ///
    /// # Errors
    /// [`TransportError::Io`] when the peer is unreachable.
    pub fn connect(&mut self) -> Result<(), TransportError> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let stream =
            TcpStream::connect(&address).map_err(|error| TransportError::Io(error.to_string()))?;
        drop(stream);
        self.connected = true;
        Ok(())
    }

    /// Close the route.
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
}

impl Transport for RealTransport {
    fn send_command(&mut self, command: &WireCommand) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        let address = format!("{}:{}", self.config.host, self.config.port);
        let mut stream =
            TcpStream::connect(&address).map_err(|error| TransportError::Io(error.to_string()))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| TransportError::Io(error.to_string()))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| TransportError::Io(error.to_string()))?;
        let body = command.to_wire_json();
        let request = format!(
            "POST /command HTTP/1.1\r\nHost: {}\r\nX-platform: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            address,
            self.config.platform,
            body.len(),
            body
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| TransportError::Io(error.to_string()))?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|error| TransportError::Io(error.to_string()))?;
        let status = response
            .split_whitespace()
            .nth(1)
            .and_then(|token| token.parse::<u16>().ok())
            .unwrap_or(0);
        if (200..300).contains(&status) {
            Ok(())
        } else {
            Err(TransportError::HttpStatus(status))
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
