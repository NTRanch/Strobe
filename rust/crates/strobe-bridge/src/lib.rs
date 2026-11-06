//! Strobe bridge: the Lovense output bridge (SPEC §4, Bevy form
//! in SPEC-BEVY §3.5).
//!
//! The bridge enforces the standing safety contract — master OFF
//! by default, ceiling clamped in code, stop synchronous, no
//! resume on reconnect, 5 s watchdog — as a **pure gate core**
//! ([`gate`]) in the flowguard shape: `decide(state, request) →
//! allow (possibly clamped) | deny (reason)`, and a deny is final:
//! no override parameter exists anywhere in this crate.
//!
//! Transports sit behind the [`transport`] seam: a mock that
//! records every outbound command for assertion, and a real HTTP
//! transport speaking the §4 wire contract (compiled in,
//! config-gated off in the slice build). The [`worker`] owns a
//! transport on a dedicated thread so the fixed-step simulation is
//! never blocked by IO.
//!
//! U5 law, restated for this crate: nothing here can gate a stop.
//! `StopAll` and `Disarm` are accepted in every state, and the
//! stop path is the highest-priority path through the worker.

#![forbid(unsafe_code)]

pub mod gate;
pub mod transport;
pub mod worker;

pub use gate::{BridgeState, Decision, DenyReason, Gate, GateConfig, Request};
pub use transport::{MockTransport, RealTransport, Transport, TransportError, WireCommand};
pub use worker::{BridgeHandle, BridgeSnapshot};
