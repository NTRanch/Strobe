//! The Bevy assembly (SPEC-BEVY §3 + G3-B/G5-B): resources, the
//! `Screen` state, and the session-driver system. All simulation
//! happens in `FixedUpdate` at the estate's pinned fixed step
//! (dt = 1/120 s); the driver is an exclusive system so the whole
//! step order — inputs, session, chart, bridge — is one
//! deterministic sequence with no hidden parallelism.
//!
//! Session-driver obligations (G5-B): it owns the single
//! `Session`, drains the action queue one entry per step in
//! arrival order, and publishes results for presentation. Core
//! remains the sole authority on session rules — the surface's
//! input gate is presentation-level; Core's surfaced backstop
//! raises if an ordinary action ever reaches it.
//!
//! Analytics emission (G9-B) happens at these call sites.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use bevy_app::{App, FixedUpdate};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use bevy_state::app::StatesPlugin;
use bevy_state::prelude::AppExtStates;
use bevy_state::state::{NextState, State, States};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use strobe_core::{
    CampaignClock, HeroineProfile, HeroineState, NullToyOutput, PhoneHub, ResponseTrack, Session,
    SessionAction, SessionEndReason, SessionSummary, SurfaceTrigger, ToyCurve, ToyOutput, W5Stage,
};

use crate::analytics::{
    AppName, EndedVia, Envelope, EventSink, EventType, Feature, FunnelStep, NullEventSink, Payload,
    SeasonEndingName, SettingName, StageName, TrackName,
};
use crate::chart::{ChartPlayer, Judgement};
use crate::data::GameData;
use crate::save::RestoredGame;

/// The app version stamped into analytics envelopes.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The screens (G3-B). `Hub` is the boot screen.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, States)]
#[serde(rename_all = "snake_case")]
pub enum Screen {
    /// The phone hub.
    #[default]
    Hub,
    /// A heroine's status sheet.
    Status,
    /// The session-request screen.
    SessionRequest,
    /// Inside a session.
    InSession,
    /// The post-session summary.
    Summary,
    /// The aftercare screen.
    Aftercare,
    /// The staged feed.
    Feed,
    /// The store.
    Store,
    /// The protocols library.
    Protocols,
    /// Settings.
    Settings,
}

/// The surfaces whose portraiture resolves through the stage
/// variant set (SPEC §3, A5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceKind {
    /// The hub portrait.
    Hub,
    /// The session-request portrait.
    Session,
    /// The status portrait.
    Status,
    /// The feed portrait.
    Feed,
}

impl SurfaceKind {
    /// All surfaces, alphabetical.
    pub const ALL: [SurfaceKind; 4] = [
        SurfaceKind::Feed,
        SurfaceKind::Hub,
        SurfaceKind::Session,
        SurfaceKind::Status,
    ];

    fn name(self) -> &'static str {
        match self {
            SurfaceKind::Feed => "feed",
            SurfaceKind::Hub => "hub",
            SurfaceKind::Session => "session",
            SurfaceKind::Status => "status",
        }
    }
}

/// The one stage-variant set: every surface resolves its portrait
/// variant through here, so a stage advance changes all four in
/// the same step and no surface can drift (SPEC §3, A5).
#[must_use]
pub fn stage_variant(surface: SurfaceKind, stage: W5Stage) -> String {
    format!("vivienne/{}/stage-{}", surface.name(), stage.discriminant())
}

/// Player settings (G10-B), persisted in their own file beside
/// the save.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PlayerSettings {
    /// The toy master switch (B5). Default OFF (§4 criterion 1).
    pub toy_master_on: bool,
    /// The player's own haptic ceiling, 0..=20.
    pub player_ceiling: i32,
    /// Whether analytics emission is enabled.
    pub analytics_enabled: bool,
    /// The per-install random id (rotatable in Settings).
    pub install_id: String,
    /// Test knob: record every step's curve/delivered pair.
    #[serde(default)]
    pub record_readout_history: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        PlayerSettings {
            toy_master_on: false,
            player_ceiling: 20,
            analytics_enabled: true,
            install_id: "install-unset".to_string(),
            record_readout_history: false,
        }
    }
}

impl PlayerSettings {
    /// Load settings from `path`, or defaults when absent/invalid.
    #[must_use]
    pub fn load_or_default(path: &std::path::Path) -> PlayerSettings {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    /// Persist settings to `path`.
    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        std::fs::write(path, json)
    }
}

/// What the toy slot does with output. The chart player is the
/// only output source, and it speaks in patterns (B3); discrete
/// sends exist for the `ToyOutput` seam Core defines.
pub enum ToySlot {
    /// No device: output is computed and read out, never sent.
    Null(NullToyOutput),
    /// A recording slot for tests and headless verification.
    Recorder(std::sync::Arc<std::sync::Mutex<Vec<ToyEvent>>>),
    /// The real bridge (worker thread + gate).
    Bridge(strobe_bridge::worker::BridgeToyOutput),
}

/// One recorded toy event.
#[derive(Clone, Debug, PartialEq)]
pub enum ToyEvent {
    /// A pattern burst: strengths + step duration in ms.
    Pattern(Vec<i32>, u32),
    /// A stop (zeroing) command.
    Stop,
}

impl ToySlot {
    fn send_pattern(&mut self, strengths: &[i32], step_ms: u32) {
        match self {
            ToySlot::Null(_) => {}
            ToySlot::Recorder(events) => {
                if let Ok(mut events) = events.lock() {
                    events.push(ToyEvent::Pattern(strengths.to_vec(), step_ms));
                }
            }
            ToySlot::Bridge(adapter) => adapter.send_pattern(strengths, step_ms),
        }
    }

    fn stop_all(&mut self) {
        match self {
            ToySlot::Null(output) => output.stop_all(),
            ToySlot::Recorder(events) => {
                if let Ok(mut events) = events.lock() {
                    events.push(ToyEvent::Stop);
                }
            }
            ToySlot::Bridge(adapter) => adapter.stop_all(),
        }
    }
}

/// One scripted or live input to the simulation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SimInput {
    /// Navigate to a screen.
    Navigate {
        /// The destination screen.
        screen: Screen,
    },
    /// Give consent for this sitting (the consent screen's
    /// affirmation). Until it arrives, play inputs are refused;
    /// safety and settings inputs never wait for it.
    GiveConsent,
    /// Request a session (the game's only clock input).
    RequestSession,
    /// Queue one action from the authored table, by id.
    ApplyAction {
        /// The action id.
        action_id: String,
    },
    /// Resume a trance with an action from the table, by id.
    ResumeTrance {
        /// The action id.
        action_id: String,
    },
    /// Fire the Surface Beat with the heroine's authored reaction.
    FireSurface,
    /// Complete the active session.
    CompleteSession,
    /// Acknowledge the aftercare screen / gate.
    AcknowledgeAftercare,
    /// A chart note input.
    NoteInput {
        /// The note's beat.
        beat: u32,
        /// Timing offset in fixed steps (0 = on the beat).
        offset_steps: i64,
    },
    /// Connect the toy bridge (discovery only follows).
    ConnectToy,
    /// Arm the toy bridge (explicit player act).
    ArmToy,
    /// Disarm the toy bridge.
    DisarmToy,
    /// Disconnect the toy bridge.
    DisconnectToy,
    /// Set the toy master switch.
    SetToyMaster {
        /// The new value.
        on: bool,
    },
    /// Set the player ceiling.
    SetPlayerCeiling {
        /// The new ceiling, clamped 0..=20.
        ceiling: i32,
    },
    /// Set analytics enabled.
    SetAnalyticsEnabled {
        /// The new value.
        enabled: bool,
    },
    /// Rotate the install id.
    RotateInstallId,
}

/// The queued inputs resource.
#[derive(Default, Resource)]
pub struct InputQueue {
    queue: VecDeque<SimInput>,
}

impl InputQueue {
    /// Queue one input for the next fixed step.
    pub fn push(&mut self, input: SimInput) {
        self.queue.push_back(input);
    }
}

/// One entry in the session log: what the driver did, in order.
/// Presentation renders from this; tests assert against it.
#[derive(Clone, Debug, PartialEq)]
pub enum LogEntry {
    /// An action was applied.
    ActionApplied {
        /// The action id.
        action_id: String,
        /// Corruption after the action.
        corruption_after: f64,
        /// Her cumulative XP on the action's track after it.
        track_xp_after: i32,
    },
    /// A tier unlocked (carries its mandatory line key).
    TierUnlocked {
        /// The new tier.
        tier: i32,
        /// The internal-voice line key for the unlock.
        line_key: String,
    },
    /// A track capstone unlocked.
    CapstoneUnlocked {
        /// The track.
        track: ResponseTrack,
        /// The capstone content key.
        content_key: String,
    },
    /// An input or action was refused, with the reason.
    Refused {
        /// What was refused.
        what: String,
        /// Why.
        reason: String,
    },
    /// The Surface Beat fired.
    SurfaceFired {
        /// The reaction's voice line key.
        line_key: String,
        /// The reaction's expression key.
        expression_key: String,
        /// Response delta of the surfacing (zero or negative: the
        /// shallowing itself).
        response_delta: f64,
    },
    /// A trance resumed.
    Resumed {
        /// The action id.
        action_id: String,
        /// Corruption gained by the resume (incl. any bonus).
        corruption_gained: f64,
    },
    /// The comfort limit ended the session.
    ComfortLimited {
        /// The tier attempted.
        attempted_tier: i32,
        /// The tier allowed.
        allowed_tier: i32,
    },
    /// A session completed.
    Completed {
        /// Clears banked after completion.
        clears: i32,
        /// Coins earned by this completion.
        coins_earned: i32,
    },
    /// Consent was given for this sitting.
    ConsentGiven,
    /// A session started.
    SessionStarted {
        /// The session's sequence number this install.
        sequence: u64,
    },
    /// Her frequency was discovered.
    FrequencyLearned,
    /// The season ended.
    SeasonEnded {
        /// The ending name.
        ending: SeasonEndingName,
    },
}

/// The post-session summary view (presentation reads this).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SummaryView {
    /// The tier reached.
    pub tier_reached: i32,
    /// The session's response peak.
    pub response_peak: f64,
    /// Coins earned.
    pub coins_earned: i32,
    /// Surface Beats played in the session.
    pub surfaces: i32,
    /// Whether aftercare is mandatory after this session.
    pub aftercare_required: bool,
    /// How the session ended.
    pub ended_via: EndedVia,
}

/// The pattern readout (B6): what the curve is doing *now*,
/// shown on the session screen whether or not a bridge exists.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PatternReadout {
    /// The current combined curve value.
    pub curve: f64,
    /// The delivered intensity for the current step.
    pub delivered: i32,
    /// The current tier ceiling.
    pub tier_ceiling: i32,
}

/// Everything needed to build the app.
pub struct AppConfig {
    /// The loaded game data.
    pub data: GameData,
    /// Player settings.
    pub settings: PlayerSettings,
    /// The analytics sink (null sink for sink-off runs).
    pub sink: Arc<dyn EventSink>,
    /// The toy slot.
    pub toy: ToySlot,
    /// A restored game, when a save was loaded.
    pub restored: Option<RestoredGame>,
    /// Whether boot restored a save (analytics payload).
    pub restored_save: bool,
    /// The analytics session id for this app run.
    pub session_id: String,
    /// Where the demo/main writes its save (unused by tests).
    pub save_path: Option<PathBuf>,
}

/// The simulation core: all mutable game state, owned by one
/// resource and advanced by the driver system only.
#[derive(Resource)]
pub struct SimCore {
    data: GameData,
    profile: &'static HeroineProfile,
    /// Her progression state.
    pub state: HeroineState,
    /// The phone hub.
    pub hub: PhoneHub,
    /// The campaign clock.
    pub clock: CampaignClock,
    session: Option<Session<'static>>,
    chart: Option<ChartPlayer>,
    /// The toy slot.
    pub toy: ToySlot,
    /// Player settings.
    pub settings: PlayerSettings,
    sink: Arc<dyn EventSink>,
    session_id: String,
    /// The current fixed-step tick.
    pub tick: u64,
    /// Sessions started this app run.
    pub sessions_started: u64,
    /// True while a mandatory aftercare is unacknowledged.
    pub aftercare_gate: bool,
    /// Whether the player has given consent this sitting. The
    /// consent screen comes first, always (brand law): play
    /// inputs are refused until this is set, while safety and
    /// settings inputs are never gated by it. Session-scoped
    /// by design — never saved, never in the canonical state.
    consent_given: bool,
    /// The last session's summary.
    pub last_summary: Option<SummaryView>,
    /// The driver log.
    pub log: Vec<LogEntry>,
    /// The pattern readout.
    pub readout: PatternReadout,
    /// Per-step (curve, delivered) history, when recording is on.
    pub readout_history: Vec<(f64, i32)>,
    /// The stage the surfaces currently resolve.
    pub surfaces_stage: W5Stage,
    season_ended_emitted: bool,
    install_rotation: u64,
}

impl SimCore {
    /// The loaded game data (the save layer snapshots from it).
    #[must_use]
    pub fn data(&self) -> &GameData {
        &self.data
    }

    /// Emit the app-session end envelope (G9-B call site: exit).
    pub fn emit_session_end(&self, restored_save: bool) {
        self.emit(EventType::SessionEnd, Payload::AppSession { restored_save });
    }

    /// Whether consent has been given for this sitting.
    #[must_use]
    pub fn consent_given(&self) -> bool {
        self.consent_given
    }

    /// Whether she is requesting the advance herself (stage ≥
    /// Craving and the corruption baseline clears tier 1) — the
    /// §9 fixture gate reads this at a seeded session's start.
    #[must_use]
    pub fn heroine_requests_next_tier(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.heroine_requests_next_tier(&self.state))
    }

    /// The active session's current tier, when in a session.
    #[must_use]
    pub fn session_tier(&self) -> Option<i32> {
        self.session.as_ref().map(Session::current_tier)
    }

    /// Whether a session is active (including a closed one awaiting
    /// completion handling — see `session_closed`).
    #[must_use]
    pub fn in_session(&self) -> bool {
        self.session.is_some()
    }

    /// Whether the active session has ended (comfort limit) and is
    /// awaiting the player's move to aftercare.
    #[must_use]
    pub fn session_closed(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.end_reason() != SessionEndReason::None)
    }

    /// Whether the Surface Beat is currently held.
    #[must_use]
    pub fn surfaced(&self) -> bool {
        self.session.as_ref().is_some_and(Session::is_surfaced)
    }

    /// The feed's current render record: framing (fixed), the
    /// stage-resolved portrait variant, and the caption key — the
    /// expression is the only staged variable (SPEC §5, Feed).
    #[must_use]
    pub fn feed_record(&self) -> (String, String, String) {
        (
            "vivienne.feed.framing.her-room".to_string(),
            stage_variant(SurfaceKind::Feed, self.state.stage()),
            "vivienne.feed.beat1.caption".to_string(),
        )
    }

    /// The canonical state serialization the determinism gate
    /// hashes (SPEC-BEVY §8.2): stage, per-track XP in
    /// discriminant order, coins, hub installs/versions, clock
    /// day/slot, surfaces taken, tier, response peak as bits.
    #[must_use]
    pub fn canonical_state(&self) -> String {
        let mut installs = String::new();
        let mut versions = String::new();
        for app in strobe_core::PhoneApp::ALL {
            installs.push(if self.hub.is_installed(app) { '1' } else { '0' });
            versions.push_str(&self.hub.version_of(app).to_string());
            versions.push(',');
        }
        // Tier / peak / surfaces serialize from the live session
        // while it is open; otherwise they are zero — the summary
        // view and a closed session awaiting aftercare are
        // presentation state and are never persisted, so they must
        // not feed the canonical form (a restored game reproduces
        // this serialization exactly).
        let (tier, peak_bits, surfaces) = match &self.session {
            Some(session) if session.end_reason() == SessionEndReason::None => (
                session.current_tier(),
                session.response_peak().to_bits(),
                session.surfaces_this_session(),
            ),
            _ => (0, 0.0_f64.to_bits(), 0),
        };
        format!(
            "strobe-canonical-v1|stage={}|gaze={}|rhythm={}|touch={}|voice={}|coins={}|clears={}|freq={}|hub={}/{}|clock={}d{}s|surfaces={}|tier={}|peak={:016x}",
            self.state.stage().discriminant(),
            self.state.track_xp(ResponseTrack::Gaze),
            self.state.track_xp(ResponseTrack::Rhythm),
            self.state.track_xp(ResponseTrack::Touch),
            self.state.track_xp(ResponseTrack::Voice),
            self.state.outfit_coins(),
            self.state.clear_count(),
            u8::from(self.state.frequency_learned()),
            installs,
            versions,
            self.clock.day(),
            self.clock.slot(),
            surfaces,
            tier,
            peak_bits,
        )
    }

    /// The SHA-256 of the canonical state serialization.
    #[must_use]
    pub fn state_hash(&self) -> String {
        let digest = Sha256::digest(self.canonical_state().as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn emit(&self, event_type: EventType, payload: Payload) {
        if !self.settings.analytics_enabled {
            return;
        }
        self.sink.emit(&Envelope {
            schema_version: crate::analytics::ANALYTICS_SCHEMA_VERSION,
            title_id: crate::analytics::TITLE_ID,
            lane: crate::analytics::LANE,
            app_version: APP_VERSION.to_string(),
            install_id: self.settings.install_id.clone(),
            session_id: self.session_id.clone(),
            ts_coarse: format!("day-{}", self.clock.day()),
            event_type_name: event_type.wire_name(),
            payload,
        });
    }

    fn refuse(&mut self, what: &str, reason: &str) {
        self.log.push(LogEntry::Refused {
            what: what.to_string(),
            reason: reason.to_string(),
        });
    }

    /// Advance the simulation one fixed step: drain the queued
    /// inputs in arrival order, then advance the chart.
    pub fn step(&mut self, inputs: Vec<SimInput>, world: &mut World) {
        self.tick += 1;
        for input in inputs {
            self.handle_input(input, world);
        }
        self.advance_chart();
        // Surfaces resolve the current stage, always together.
        self.surfaces_stage = self.state.stage();
    }

    fn set_screen(&mut self, world: &mut World, screen: Screen) {
        if let Some(mut next) = world.get_resource_mut::<NextState<Screen>>() {
            next.set(screen);
        }
    }

    /// Whether an input may run before consent: navigation,
    /// settings and privacy controls, and every safety surface.
    /// Stopping, surfacing, and disarming never wait for consent
    /// (U5); enabling anything always does.
    fn allowed_before_consent(input: &SimInput) -> bool {
        match input {
            SimInput::GiveConsent
            | SimInput::Navigate { .. }
            | SimInput::SetPlayerCeiling { .. }
            | SimInput::SetAnalyticsEnabled { .. }
            | SimInput::RotateInstallId
            | SimInput::FireSurface
            | SimInput::DisarmToy
            | SimInput::DisconnectToy
            | SimInput::ConnectToy => true,
            SimInput::SetToyMaster { on } => !*on,
            _ => false,
        }
    }

    /// The refusal label for a consent-gated input.
    fn consent_label(input: &SimInput) -> &'static str {
        match input {
            SimInput::RequestSession => "session request",
            SimInput::ApplyAction { .. } => "action",
            SimInput::ResumeTrance { .. } => "resume",
            SimInput::CompleteSession => "session completion",
            SimInput::AcknowledgeAftercare => "aftercare acknowledgement",
            SimInput::NoteInput { .. } => "note input",
            SimInput::ArmToy => "toy arming",
            SimInput::SetToyMaster { .. } => "toy master switch",
            _ => "input",
        }
    }

    fn handle_input(&mut self, input: SimInput, world: &mut World) {
        if !self.consent_given && !Self::allowed_before_consent(&input) {
            let what = Self::consent_label(&input);
            self.refuse(what, "consent required");
            return;
        }
        match input {
            SimInput::GiveConsent => {
                if !self.consent_given {
                    self.consent_given = true;
                    self.log.push(LogEntry::ConsentGiven);
                }
            }
            SimInput::Navigate { screen } => {
                match screen {
                    Screen::Status => {
                        self.emit(
                            EventType::FunnelStep,
                            Payload::FunnelStep {
                                step: FunnelStep::StatusViewed,
                            },
                        );
                    }
                    Screen::Hub => {
                        self.emit(
                            EventType::FunnelStep,
                            Payload::FunnelStep {
                                step: FunnelStep::HubViewed,
                            },
                        );
                    }
                    Screen::Aftercare => {
                        self.emit(
                            EventType::FunnelStep,
                            Payload::FunnelStep {
                                step: FunnelStep::AftercareViewed,
                            },
                        );
                    }
                    _ => {}
                }
                self.set_screen(world, screen);
            }
            SimInput::RequestSession => self.request_session(world),
            SimInput::ApplyAction { action_id } => {
                self.apply_action(&action_id);
                if self.session_closed() {
                    self.set_screen(world, Screen::Summary);
                }
            }
            SimInput::ResumeTrance { action_id } => self.resume_trance(&action_id),
            SimInput::FireSurface => self.fire_surface(),
            SimInput::CompleteSession => self.complete_session(world),
            SimInput::AcknowledgeAftercare => self.acknowledge_aftercare(world),
            SimInput::NoteInput { beat, offset_steps } => {
                if let Some(chart) = &mut self.chart {
                    chart.record_input(beat, offset_steps);
                }
            }
            SimInput::ConnectToy => {
                if let ToySlot::Bridge(adapter) = &mut self.toy {
                    adapter.connect_and_discover(self.tick);
                }
            }
            SimInput::ArmToy => {
                if let ToySlot::Bridge(adapter) = &mut self.toy {
                    adapter.arm(self.tick);
                    self.emit(
                        EventType::FeatureUsed,
                        Payload::FeatureUsed {
                            feature: Feature::ToyBridgeArmed,
                        },
                    );
                }
            }
            SimInput::DisarmToy => {
                if let ToySlot::Bridge(adapter) = &mut self.toy {
                    adapter.disarm(self.tick);
                }
            }
            SimInput::DisconnectToy => {
                if let ToySlot::Bridge(adapter) = &mut self.toy {
                    adapter.disconnect(self.tick);
                }
            }
            SimInput::SetToyMaster { on } => {
                self.settings.toy_master_on = on;
                self.emit(
                    EventType::SettingsChanged,
                    Payload::SettingsChanged {
                        setting: SettingName::ToyMaster,
                        enabled: Some(on),
                    },
                );
                if !on {
                    // Turning the master off zeroes output at once.
                    self.toy.stop_all();
                }
            }
            SimInput::SetPlayerCeiling { ceiling } => {
                self.settings.player_ceiling = ceiling.clamp(0, ToyCurve::MAX_INTENSITY);
                self.emit(
                    EventType::SettingsChanged,
                    Payload::SettingsChanged {
                        setting: SettingName::PlayerCeiling,
                        enabled: None,
                    },
                );
            }
            SimInput::SetAnalyticsEnabled { enabled } => {
                self.settings.analytics_enabled = enabled;
                self.emit(
                    EventType::SettingsChanged,
                    Payload::SettingsChanged {
                        setting: SettingName::AnalyticsEnabled,
                        enabled: Some(enabled),
                    },
                );
            }
            SimInput::RotateInstallId => {
                self.install_rotation += 1;
                let digest = Sha256::digest(
                    format!("{}:{}", self.settings.install_id, self.install_rotation).as_bytes(),
                );
                self.settings.install_id = format!(
                    "install-{}",
                    digest
                        .iter()
                        .take(6)
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
                self.emit(
                    EventType::SettingsChanged,
                    Payload::SettingsChanged {
                        setting: SettingName::InstallIdRotated,
                        enabled: None,
                    },
                );
            }
        }
    }

    fn request_session(&mut self, world: &mut World) {
        if self.aftercare_gate {
            self.refuse("request_session", "mandatory aftercare is unacknowledged");
            return;
        }
        if self.session.is_some() {
            self.refuse("request_session", "a session is already active");
            return;
        }
        if self.clock.season_over() {
            self.emit_season_ended();
            self.refuse("request_session", "the season is over");
            return;
        }
        let day_before = self.clock.day();
        if !self.clock.spend_slot() {
            self.refuse("request_session", "the season is over");
            return;
        }
        if self.clock.day() != day_before {
            self.emit(
                EventType::StrobeClockAdvanced,
                Payload::ClockAdvanced {
                    day: self.clock.day(),
                    season_over: self.clock.season_over(),
                },
            );
        }
        let session = match Session::new(self.profile, &self.state) {
            Ok(session) => session,
            Err(error) => {
                self.refuse("request_session", &error.to_string());
                return;
            }
        };
        self.sessions_started += 1;
        self.session = Some(session);
        self.chart = Some(ChartPlayer::new(&self.data.chart));
        self.log.push(LogEntry::SessionStarted {
            sequence: self.sessions_started,
        });
        self.emit(
            EventType::FunnelStep,
            Payload::FunnelStep {
                step: FunnelStep::SessionRequested,
            },
        );
        self.emit(
            EventType::FunnelStep,
            Payload::FunnelStep {
                step: FunnelStep::SessionStarted,
            },
        );
        self.emit(
            EventType::LevelStart,
            Payload::Session {
                tier: 0,
                ended_via: None,
            },
        );
        self.set_screen(world, Screen::InSession);
    }

    fn emit_season_ended(&mut self) {
        if self.season_ended_emitted {
            return;
        }
        self.season_ended_emitted = true;
        let ending = SeasonEndingName::from(CampaignClock::evaluate_ending(self.state.stage()));
        self.emit(
            EventType::StrobeSeasonEnded,
            Payload::SeasonEnded {
                ending,
                clears: self.state.clear_count(),
            },
        );
        self.emit(
            EventType::FunnelStep,
            Payload::FunnelStep {
                step: FunnelStep::SeasonEnded,
            },
        );
        self.log.push(LogEntry::SeasonEnded { ending });
    }

    /// Build the Core action for an authored row, by id. The id
    /// and label keys live in the game layer; Core's action is the
    /// rule-bearing tuple.
    fn action_def(&self, action_id: &str) -> Option<SessionAction> {
        let def = self.data.action(action_id)?;
        SessionAction::new(
            def.track,
            def.tier,
            def.kink_aligned,
            def.rushed,
            def.initiated_by_her,
            def.dial_position,
        )
        .ok()
    }

    fn apply_action(&mut self, action_id: &str) {
        let Some(action) = self.action_def(action_id) else {
            self.refuse("apply_action", &format!("unknown action '{action_id}'"));
            return;
        };
        let track = action.track();
        let action_tier = action.tier();
        // The session work happens in a scope so its borrow ends
        // before the logging / emission that follows it.
        struct Applied {
            allowed_before: i32,
            corruption_after: f64,
            result: strobe_core::ActionResult,
            track_xp_after: i32,
        }
        let applied: Result<Applied, String> = {
            let Some(session) = &mut self.session else {
                self.refuse("apply_action", "no active session");
                return;
            };
            if session.end_reason() != SessionEndReason::None {
                Err("the session has ended".to_string())
            } else if session.is_surfaced() {
                // The surface's input gate (SPEC §2.3): ordinary
                // queued actions are refused here; Core's backstop
                // stands behind this if the gate is ever bypassed.
                Err("a surface is held; resolve it first".to_string())
            } else {
                let allowed_before = session.tier_allowed_by_corruption();
                match session.apply_action(&mut self.state, action) {
                    Ok(result) => Ok(Applied {
                        allowed_before,
                        corruption_after: session.corruption(),
                        result,
                        track_xp_after: self.state.track_xp(track),
                    }),
                    Err(error) => Err(error.to_string()),
                }
            }
        };
        let applied = match applied {
            Ok(applied) => applied,
            Err(reason) => {
                self.refuse("apply_action", &reason);
                return;
            }
        };
        let result = applied.result;
        self.log.push(LogEntry::ActionApplied {
            action_id: action_id.to_string(),
            corruption_after: applied.corruption_after,
            track_xp_after: applied.track_xp_after,
        });
        if result.frequency_discovered() {
            self.log.push(LogEntry::FrequencyLearned);
        }
        if result.tier_unlocked() >= 0 {
            let tier = result.tier_unlocked();
            self.log.push(LogEntry::TierUnlocked {
                tier,
                line_key: format!("vivienne.unlock.tier{tier}"),
            });
            self.emit(
                EventType::StrobeTierUnlocked,
                Payload::TierUnlocked { tier },
            );
        }
        if result.capstone_unlocked() {
            let content_key = self
                .data
                .capstone_keys
                .get(&track)
                .cloned()
                .unwrap_or_default();
            self.log
                .push(LogEntry::CapstoneUnlocked { track, content_key });
            self.emit(
                EventType::StrobeCapstoneUnlocked,
                Payload::CapstoneUnlocked {
                    track: TrackName::from(track),
                },
            );
            // Protocols gains an entry per capstone (§5). The
            // upgrade is refused by the hub until Protocols is
            // installed (the Willing sync); the refusal is the
            // reference behavior, not an error here.
            let _ = self.hub.upgrade(strobe_core::PhoneApp::Protocols);
        }
        if result.ended_by_comfort_limit() {
            // Criterion 3: the zeroing happens in the same event.
            self.toy.stop_all();
            self.emit(
                EventType::StrobeComfortLimit,
                Payload::ComfortLimit {
                    attempted_tier: action_tier,
                    allowed_tier: applied.allowed_before,
                },
            );
            self.log.push(LogEntry::ComfortLimited {
                attempted_tier: action_tier,
                allowed_tier: applied.allowed_before,
            });
            self.close_session(EndedVia::ComfortLimit);
        }
    }

    /// Shared close-out for a session ended by its own rules (the
    /// comfort limit): summary view, the aftercare gate, chart off.
    /// The session object stays for the summary until the player
    /// moves on; a new request is refused while it stands.
    fn close_session(&mut self, ended_via: EndedVia) {
        if let Some(session) = &self.session {
            self.last_summary = Some(SummaryView {
                tier_reached: session.current_tier(),
                response_peak: session.response_peak(),
                coins_earned: 0,
                surfaces: session.surfaces_this_session(),
                aftercare_required: session.aftercare_required(),
                ended_via,
            });
        }
        if self
            .last_summary
            .is_some_and(|summary| summary.aftercare_required)
        {
            self.aftercare_gate = true;
        }
        self.chart = None;
    }

    fn fire_surface(&mut self) {
        let reaction = self.data.surface_reaction.clone();
        let fired: Result<(strobe_core::SurfaceResult, i32), String> = {
            let Some(session) = &mut self.session else {
                self.refuse("fire_surface", "no active session");
                return;
            };
            if session.end_reason() != SessionEndReason::None || session.is_surfaced() {
                Err("no surface can fire now".to_string())
            } else {
                match session.surface(SurfaceTrigger::CountToZero, Some(reaction)) {
                    Ok(result) => Ok((result, session.current_tier())),
                    Err(error) => Err(error.to_string()),
                }
            }
        };
        let (result, tier) = match fired {
            Ok(pair) => pair,
            Err(reason) => {
                self.refuse("fire_surface", &reason);
                return;
            }
        };
        // Simultaneity (§1 step 5): the toy stops as the beat fires.
        self.toy.stop_all();
        self.emit(
            EventType::StrobeSurfaceBeat,
            Payload::SurfaceBeat {
                tier,
                first: result.surfaces_this_session() <= 1,
            },
        );
        self.emit(
            EventType::FeatureUsed,
            Payload::FeatureUsed {
                feature: Feature::SurfaceBeat,
            },
        );
        self.log.push(LogEntry::SurfaceFired {
            line_key: result.reaction().voice_line_key().to_string(),
            expression_key: result.reaction().expression_key().to_string(),
            response_delta: result.response_delta(),
        });
    }

    fn resume_trance(&mut self, action_id: &str) {
        let Some(action) = self.action_def(action_id) else {
            self.refuse("resume_trance", &format!("unknown action '{action_id}'"));
            return;
        };
        let resumed: Result<strobe_core::ActionResult, String> = {
            let Some(session) = &mut self.session else {
                self.refuse("resume_trance", "no active session");
                return;
            };
            session
                .resume_trance(&mut self.state, action)
                .map_err(|error| error.to_string())
        };
        match resumed {
            Ok(result) => {
                self.log.push(LogEntry::Resumed {
                    action_id: action_id.to_string(),
                    corruption_gained: result.corruption_delta(),
                });
                if result.tier_unlocked() >= 0 {
                    let tier = result.tier_unlocked();
                    self.log.push(LogEntry::TierUnlocked {
                        tier,
                        line_key: format!("vivienne.unlock.tier{tier}"),
                    });
                    self.emit(
                        EventType::StrobeTierUnlocked,
                        Payload::TierUnlocked { tier },
                    );
                }
                // Output may flow again for the new action (§4
                // criterion 4: a new session action re-opens).
                if let ToySlot::Bridge(adapter) = &mut self.toy {
                    adapter.reopen();
                }
            }
            Err(reason) => self.refuse("resume_trance", &reason),
        }
    }

    fn complete_session(&mut self, world: &mut World) {
        let stage_before = self.state.stage();
        let completed: Result<(SessionSummary, bool), String> = {
            let Some(session) = &mut self.session else {
                self.refuse("complete_session", "no active session");
                return;
            };
            if session.end_reason() != SessionEndReason::None {
                Err("the session already ended".to_string())
            } else if session.is_surfaced() {
                Err("a surface is held; resolve it first".to_string())
            } else {
                session
                    .complete(&mut self.state)
                    .map(|summary| (summary, session.aftercare_required()))
                    .map_err(|error| error.to_string())
            }
        };
        let (summary, aftercare_required): (SessionSummary, bool) = match completed {
            Ok(pair) => pair,
            Err(reason) => {
                self.refuse("complete_session", &reason);
                return;
            }
        };
        self.hub.sync_from_state(&self.state);
        self.session = None;
        self.chart = None;
        self.last_summary = Some(SummaryView {
            tier_reached: summary.tier_reached(),
            response_peak: summary.response_peak(),
            coins_earned: summary.coins_awarded(),
            surfaces: summary.surfaces(),
            aftercare_required,
            ended_via: EndedVia::Completed,
        });
        if aftercare_required {
            self.aftercare_gate = true;
        }
        self.log.push(LogEntry::Completed {
            clears: self.state.clear_count(),
            coins_earned: summary.coins_awarded(),
        });
        self.emit(
            EventType::LevelComplete,
            Payload::Session {
                tier: summary.tier_reached(),
                ended_via: Some(EndedVia::Completed),
            },
        );
        self.emit(
            EventType::FunnelStep,
            Payload::FunnelStep {
                step: FunnelStep::SessionCompleted,
            },
        );
        let stage_after = self.state.stage();
        if stage_after != stage_before {
            self.emit(
                EventType::StrobeStageChanged,
                Payload::StageChanged {
                    stage: StageName::from(stage_after),
                },
            );
            self.emit(
                EventType::FeatureUsed,
                Payload::AppChanged {
                    app: AppName::Feed,
                    version: self.hub.version_of(strobe_core::PhoneApp::Feed),
                },
            );
        }
        if self.clock.season_over() {
            self.emit_season_ended();
        }
        self.set_screen(world, Screen::Summary);
    }

    fn acknowledge_aftercare(&mut self, world: &mut World) {
        let current = world
            .get_resource::<State<Screen>>()
            .map(|state| *state.get());
        if self.last_summary.is_some_and(|s| s.aftercare_required)
            && current != Some(Screen::Aftercare)
        {
            // Mandatory aftercare is viewed first (G8-B).
            self.emit(
                EventType::FunnelStep,
                Payload::FunnelStep {
                    step: FunnelStep::AftercareViewed,
                },
            );
            self.set_screen(world, Screen::Aftercare);
            return;
        }
        self.aftercare_gate = false;
        // A session closed by its own rules is now fully put away.
        if self
            .session
            .as_ref()
            .is_some_and(|session| session.end_reason() != SessionEndReason::None)
        {
            self.session = None;
        }
        self.set_screen(world, Screen::Hub);
    }

    fn advance_chart(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        if session.end_reason() != SessionEndReason::None
            || session.is_surfaced()
            || self.chart.is_none()
        {
            return;
        }
        let tier = session.current_tier();
        let previous_delivered = self.readout.delivered;
        let step = {
            let chart = self.chart.as_mut().expect("chart checked above");
            chart.advance(&self.data.chart, previous_delivered)
        };
        let delivered = ToyCurve::compute_intensity(
            self.settings.toy_master_on,
            step.curve,
            tier,
            self.settings.player_ceiling,
        );
        self.readout = PatternReadout {
            curve: step.curve,
            delivered,
            tier_ceiling: ToyCurve::MAX_INTENSITY * tier / Session::MAX_TIER,
        };
        if self.settings.record_readout_history {
            self.readout_history.push((step.curve, delivered));
        }
        if let Some(Judgement::Miss) = step.judgement {
            // The thud is already in the curve; nothing extra is
            // owed — absent output is the one forbidden shape.
        }
        if let Some(chart) = &mut self.chart {
            if let Some((strengths, step_ms)) = chart.take_pattern() {
                self.toy.send_pattern(&strengths, step_ms);
            }
            if chart.finished() {
                // The chart ends; the session continues in quiet
                // until the player completes it.
                self.readout.curve = 0.0;
                self.readout.delivered = 0;
            }
        }
    }
}

/// The fixed-step driver system: drains the input queue into the
/// core, in arrival order, once per step. The core is taken out of
/// the world for the step so input handling can also drive the
/// `Screen` state resources without borrow conflicts.
fn driver_system(world: &mut World) {
    let inputs: Vec<SimInput> = {
        let mut queue = world.resource_mut::<InputQueue>();
        queue.queue.drain(..).collect()
    };
    let mut core = world
        .remove_resource::<SimCore>()
        .expect("the app builder installs the sim core");
    core.step(inputs, world);
    // The bridge watchdog ticks with the simulation (§3.5): the
    // worker's clock is the sim tick, never a wall clock.
    if let ToySlot::Bridge(adapter) = &core.toy {
        adapter.pump_tick(core.tick);
    }
    world.insert_resource(core);
}

/// Build the headless app: states, resources, the driver in
/// `FixedUpdate`, ready to be stepped by the harness or a runner.
pub fn build_app(config: AppConfig) -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.init_state::<Screen>();
    app.init_resource::<InputQueue>();

    // The profile is immutable for the app's lifetime; the driver
    // holds it as a 'static reference so sessions can live in a
    // resource (their C# counterpart holds the same reference).
    let profile: &'static HeroineProfile = Box::leak(Box::new(config.data.profile.clone()));
    let (state, hub, clock) = match config.restored {
        Some(restored) => (restored.state, restored.hub, restored.clock),
        None => (
            HeroineState::new(profile.id()).expect("the data layer validated the profile"),
            PhoneHub::new(),
            CampaignClock::new(
                config.data.season.total_days,
                config.data.season.slots_per_day,
            )
            .expect("the data layer validated the season"),
        ),
    };
    let core = SimCore {
        data: config.data,
        profile,
        state,
        hub,
        clock,
        session: None,
        chart: None,
        toy: config.toy,
        settings: config.settings,
        sink: Arc::clone(&config.sink),
        session_id: config.session_id,
        tick: 0,
        sessions_started: 0,
        aftercare_gate: false,
        consent_given: false,
        last_summary: None,
        log: Vec::new(),
        readout: PatternReadout::default(),
        readout_history: Vec::new(),
        surfaces_stage: W5Stage::Unaware,
        season_ended_emitted: false,
        install_rotation: 0,
    };
    // App-session start is emitted at boot (G9-B call site: boot).
    core.emit(
        EventType::SessionStart,
        Payload::AppSession {
            restored_save: config.restored_save,
        },
    );
    app.insert_resource(core);
    app.add_systems(FixedUpdate, driver_system);
    app
}

/// Push one input into the app's queue.
pub fn push_input(app: &mut App, input: SimInput) {
    app.world_mut().resource_mut::<InputQueue>().push(input);
}

/// Step the app `ticks` fixed steps: `FixedUpdate` then the state
/// transition schedule, per tick — the deterministic harness the
/// gates run on.
pub fn run_ticks(app: &mut App, ticks: u64) {
    for _ in 0..ticks {
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut()
            .run_schedule(bevy_state::state::StateTransition);
    }
}

/// A sink that records envelopes for gate assertions.
pub struct RecordingSink {
    /// Every envelope emitted, in order.
    pub envelopes: std::sync::Mutex<Vec<Envelope>>,
}

impl RecordingSink {
    /// A fresh recording sink.
    #[must_use]
    pub fn new() -> RecordingSink {
        RecordingSink {
            envelopes: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// The wire names of the recorded event types, in order.
    #[must_use]
    pub fn event_names(&self) -> Vec<&'static str> {
        self.envelopes
            .lock()
            .map(|envelopes| {
                envelopes
                    .iter()
                    .map(|envelope| envelope.event_type_name)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for RecordingSink {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, envelope: &Envelope) {
        if let Ok(mut envelopes) = self.envelopes.lock() {
            envelopes.push(envelope.clone());
        }
    }
}

/// The null sink re-exported for builder convenience.
pub type NullSink = NullEventSink;
