//! The pure bridge gate core (SPEC §4 B2, SPEC-BEVY §3.5).
//!
//! Pure: no IO, no threads, no clock of its own. Time arrives as
//! `now_tick`, a caller-supplied count of fixed steps (the
//! workspace fixed timestep is 1/120 s, the lumen rig `dt`), which
//! makes the watchdog deterministic and its battery sleep-free.
//!
//! The gate owns the safety state machine: connection, discovery,
//! arming, output activity, and the watchdog. Every request passes
//! through [`Gate::decide`]; allowed requests return the wire
//! commands to emit, denied requests return a typed reason and
//! emit nothing.

use strobe_core::ToyCurve;

/// Bridge configuration. Host/port for the real transport live in
/// the transport's own config; the gate owns only safety numbers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GateConfig {
    /// Player-set intensity ceiling, 0..=20. The player's safety
    /// control (Settings, G10-B); clamped into range at use.
    pub player_ceiling: i32,
    /// Watchdog window in fixed steps: output active + no
    /// heartbeat for this many steps ⇒ zeroed and locked until
    /// re-armed. Default: 5 s at the 1/120 s fixed step = 600.
    pub watchdog_window_ticks: u64,
}

impl Default for GateConfig {
    fn default() -> Self {
        GateConfig {
            player_ceiling: ToyCurve::MAX_INTENSITY,
            watchdog_window_ticks: 600,
        }
    }
}

/// The bridge's published state (SPEC §4 B6).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BridgeState {
    /// No transport connection.
    #[default]
    Disconnected,
    /// Connected and discovered; not armed. Discovery is
    /// unarmed-safe: it never emits output.
    Discovered,
    /// Master switch on; no output active right now.
    Armed,
    /// Master switch on and output is flowing.
    Active,
    /// The watchdog fired; output is zeroed and the bridge stays
    /// locked until a new explicit arm.
    WatchdogStopped,
}

/// Why a request was refused. A deny is final — there is no
/// override parameter, in this enum or anywhere else in the crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DenyReason {
    /// Output requested while the master switch is off.
    NotArmed,
    /// Output requested with no connected transport.
    NotConnected,
    /// The watchdog has stopped the bridge; re-arm explicitly.
    WatchdogLocked,
    /// Pattern carries more strengths than the wire allows (50).
    PatternTooLong,
    /// Pattern step interval is at or below the 100 ms floor.
    PatternStepTooShort,
    /// Pattern's total duration is under the 1 s design floor.
    PatternTooShort,
    /// Pattern rule text does not parse (wire format).
    PatternMalformed,
}

/// One request to the bridge.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Establish the transport connection (discovery follows).
    Connect,
    /// The transport connection dropped or was closed.
    Disconnect,
    /// Run discovery. Unarmed-safe; emits no output commands.
    Discover,
    /// Explicit player arm action (Settings). The only way output
    /// becomes possible; boot, discovery, and reconnect never arm.
    Arm,
    /// Explicit player disarm: zeroes output and switches the
    /// master off. Accepted in every state.
    Disarm,
    /// Deliver one intensity sample (a D6 curve sample, already
    /// clamped by Core's `ToyCurve`; the gate clamps again —
    /// defense in depth, the bridge never trusts its caller).
    Send {
        /// Requested intensity, 0..=20 after clamping.
        intensity: i32,
    },
    /// Deliver a pattern burst (D6 per-hit output is delivered as
    /// patterns, never per-hit function calls).
    SendPattern {
        /// Per-step strengths, at most 50.
        strengths: Vec<i32>,
        /// Step interval in milliseconds, strictly above 100.
        step_ms: u32,
    },
    /// Panic zero: zeroes every feature, callable in any state,
    /// never gated by the arm state.
    StopAll,
    /// A heartbeat stamped with the caller's current tick.
    Heartbeat,
}

/// The gate's answer: the wire commands to emit (possibly empty),
/// or a final denial.
#[derive(Clone, Debug, PartialEq)]
pub enum Decision {
    /// Emit these commands, in order.
    Allow {
        /// Commands for the transport; empty for state-only requests.
        commands: Vec<crate::transport::WireCommand>,
    },
    /// Emit nothing.
    Deny {
        /// Why the request was refused.
        reason: DenyReason,
    },
}

/// Most strengths one pattern may carry (wire contract).
pub const PATTERN_STRENGTHS_MAX: usize = 50;

/// Pattern step intervals must be strictly above this (wire contract).
pub const PATTERN_STEP_MS_MIN: u32 = 100;

/// Shortest total pattern duration the bridge will emit (§4 B3:
/// sub-1 s output is refused by design).
pub const PATTERN_DURATION_MS_MIN: u64 = 1_000;

/// The safety state machine.
#[derive(Clone, Debug)]
pub struct Gate {
    config: GateConfig,
    connected: bool,
    armed: bool,
    output_active: bool,
    watchdog_stopped: bool,
    last_heartbeat_tick: u64,
}

impl Gate {
    /// A fresh gate: disconnected, master OFF. Every boot, fresh
    /// profile, and reconnect starts here (§4 criterion 1).
    #[must_use]
    pub fn new(config: GateConfig) -> Gate {
        Gate {
            config,
            connected: false,
            armed: false,
            output_active: false,
            watchdog_stopped: false,
            last_heartbeat_tick: 0,
        }
    }

    /// The published bridge state (B6).
    #[must_use]
    pub fn state(&self) -> BridgeState {
        if !self.connected {
            return BridgeState::Disconnected;
        }
        if self.watchdog_stopped {
            return BridgeState::WatchdogStopped;
        }
        if self.armed && self.output_active {
            return BridgeState::Active;
        }
        if self.armed {
            return BridgeState::Armed;
        }
        BridgeState::Discovered
    }

    /// True while output is flowing.
    #[must_use]
    pub fn output_active(&self) -> bool {
        self.output_active
    }

    /// True while the master switch is on.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.armed
    }

    /// Decide one request at `now_tick`. Allowed state-changing
    /// requests mutate the gate; denied requests never mutate it
    /// and never emit.
    pub fn decide(&mut self, request: &Request, now_tick: u64) -> Decision {
        match request {
            Request::Connect => {
                self.connected = true;
                // A (re)connect restores discovery only — never
                // arming, never output (§4 criterion 4).
                self.armed = false;
                self.output_active = false;
                self.watchdog_stopped = false;
                Decision::Allow {
                    commands: Vec::new(),
                }
            }
            Request::Disconnect => {
                // A dropped connection zeroes output (§4 B5).
                self.connected = false;
                self.armed = false;
                self.output_active = false;
                Decision::Allow {
                    commands: Vec::new(),
                }
            }
            Request::Discover => Decision::Allow {
                commands: Vec::new(),
            },
            Request::Arm => {
                if !self.connected {
                    return Decision::Deny {
                        reason: DenyReason::NotConnected,
                    };
                }
                self.armed = true;
                self.watchdog_stopped = false;
                self.last_heartbeat_tick = now_tick;
                Decision::Allow {
                    commands: Vec::new(),
                }
            }
            Request::Disarm => {
                self.armed = false;
                let commands = self.zero_output_commands();
                Decision::Allow { commands }
            }
            Request::StopAll => {
                // The panic zero: any state, never gated.
                let commands = self.zero_output_commands();
                Decision::Allow { commands }
            }
            Request::Heartbeat => {
                if self.connected {
                    self.last_heartbeat_tick = now_tick;
                }
                Decision::Allow {
                    commands: Vec::new(),
                }
            }
            Request::Send { intensity } => self.decide_send(*intensity),
            Request::SendPattern { strengths, step_ms } => self.decide_pattern(strengths, *step_ms),
        }
    }

    /// Advance the watchdog to `now_tick`. When output is active
    /// and heartbeats have been silent past the window, output is
    /// zeroed and the bridge locks until re-armed (§4 criterion 5).
    /// Returns the zeroing command to emit, if the watchdog fired.
    pub fn tick(&mut self, now_tick: u64) -> Option<crate::transport::WireCommand> {
        if !self.output_active || !self.connected {
            return None;
        }
        let silent_for = now_tick.saturating_sub(self.last_heartbeat_tick);
        if silent_for <= self.config.watchdog_window_ticks {
            return None;
        }
        self.watchdog_stopped = true;
        self.armed = false;
        self.output_active = false;
        Some(crate::transport::WireCommand::StopAll)
    }

    fn zero_output_commands(&mut self) -> Vec<crate::transport::WireCommand> {
        if self.output_active {
            self.output_active = false;
            vec![crate::transport::WireCommand::StopAll]
        } else {
            Vec::new()
        }
    }

    fn output_permit(&self) -> Result<(), DenyReason> {
        if !self.connected {
            return Err(DenyReason::NotConnected);
        }
        if self.watchdog_stopped {
            return Err(DenyReason::WatchdogLocked);
        }
        if !self.armed {
            return Err(DenyReason::NotArmed);
        }
        Ok(())
    }

    fn clamp_strength(&self, strength: i32) -> i32 {
        let ceiling = self.config.player_ceiling.clamp(0, ToyCurve::MAX_INTENSITY);
        strength.clamp(0, ceiling)
    }

    fn decide_send(&mut self, intensity: i32) -> Decision {
        if let Err(reason) = self.output_permit() {
            return Decision::Deny { reason };
        }
        let clamped = self.clamp_strength(intensity);
        self.output_active = true;
        Decision::Allow {
            commands: vec![crate::transport::WireCommand::Function { strength: clamped }],
        }
    }

    fn decide_pattern(&mut self, strengths: &[i32], step_ms: u32) -> Decision {
        // Pattern discipline refuses happen before the wire and
        // before the arm check's effects: a malformed request is
        // refused as malformed regardless of state (§4 criterion 6).
        if strengths.len() > PATTERN_STRENGTHS_MAX {
            return Decision::Deny {
                reason: DenyReason::PatternTooLong,
            };
        }
        if step_ms <= PATTERN_STEP_MS_MIN {
            return Decision::Deny {
                reason: DenyReason::PatternStepTooShort,
            };
        }
        let duration_ms = strengths.len() as u64 * u64::from(step_ms);
        if duration_ms < PATTERN_DURATION_MS_MIN {
            return Decision::Deny {
                reason: DenyReason::PatternTooShort,
            };
        }
        if let Err(reason) = self.output_permit() {
            return Decision::Deny { reason };
        }
        let clamped: Vec<i32> = strengths
            .iter()
            .map(|strength| self.clamp_strength(*strength))
            .collect();
        self.output_active = true;
        Decision::Allow {
            commands: vec![crate::transport::WireCommand::Pattern {
                strengths: clamped,
                step_ms,
            }],
        }
    }
}

/// Parse one wire pattern rule, `V:1;F:<features>;S:<ms>#`
/// (Pattern apiVer 2). Returns the step interval in milliseconds.
///
/// # Errors
/// Returns [`DenyReason::PatternMalformed`] for any text that does
/// not match the rule shape — malformed rules are refused before
/// the wire, never repaired.
pub fn parse_pattern_rule(rule: &str) -> Result<u32, DenyReason> {
    let body = rule.strip_suffix('#').ok_or(DenyReason::PatternMalformed)?;
    let mut version_ok = false;
    let mut step_ms: Option<u32> = None;
    for field in body.split(';') {
        if field == "V:1" {
            version_ok = true;
        } else if let Some(features) = field.strip_prefix("F:") {
            if features.is_empty() || !features.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(DenyReason::PatternMalformed);
            }
        } else if let Some(step) = field.strip_prefix("S:") {
            step_ms = Some(
                step.parse::<u32>()
                    .map_err(|_| DenyReason::PatternMalformed)?,
            );
        } else {
            return Err(DenyReason::PatternMalformed);
        }
    }
    if !version_ok {
        return Err(DenyReason::PatternMalformed);
    }
    step_ms.ok_or(DenyReason::PatternMalformed)
}
