//! Shared estate analytics (SPEC-BEVY G9-B): one envelope, typed
//! payloads, no free text anywhere — events carry enums, booleans,
//! and bounded numbers only, so the no-PII / no-transcript rule is
//! a property of the type system, not of reviewer vigilance.
//!
//! The sink is an injected trait object. Live builds default to
//! the JSONL file sink; tests use the null sink and assert the
//! spine's state hash is identical with the sink off. A sink that
//! fails degrades to the null sink and never throws into gameplay.

use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use strobe_core::{PhoneApp, ResponseTrack, SessionEndReason, W5Stage};

/// The envelope schema version.
pub const ANALYTICS_SCHEMA_VERSION: u32 = 1;

/// The title id every Strobe envelope carries.
pub const TITLE_ID: &str = "strobe";
/// The lane every Strobe envelope carries.
pub const LANE: &str = "ntranch";

/// The fixed event vocabulary: the shared spine plus the
/// `strobe.`-namespaced set from SPEC §6 / G9-B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// A play session of the app started.
    SessionStart,
    /// A play session of the app ended.
    SessionEnd,
    /// A level (here: a heroine session) started.
    LevelStart,
    /// A level completed.
    LevelComplete,
    /// A feature was used (payload names it).
    FeatureUsed,
    /// A setting changed (payload names it).
    SettingsChanged,
    /// A funnel step was reached (payload names it).
    FunnelStep,
    /// strobe.tier_unlocked
    StrobeTierUnlocked,
    /// strobe.surface_beat
    StrobeSurfaceBeat,
    /// strobe.comfort_limit
    StrobeComfortLimit,
    /// strobe.stage_changed
    StrobeStageChanged,
    /// strobe.capstone_unlocked
    StrobeCapstoneUnlocked,
    /// strobe.clock_advanced
    StrobeClockAdvanced,
    /// strobe.season_ended
    StrobeSeasonEnded,
}

impl EventType {
    /// The wire name of the event type.
    #[must_use]
    pub fn wire_name(self) -> &'static str {
        match self {
            EventType::SessionStart => "session_start",
            EventType::SessionEnd => "session_end",
            EventType::LevelStart => "level_start",
            EventType::LevelComplete => "level_complete",
            EventType::FeatureUsed => "feature_used",
            EventType::SettingsChanged => "settings_changed",
            EventType::FunnelStep => "funnel_step",
            EventType::StrobeTierUnlocked => "strobe.tier_unlocked",
            EventType::StrobeSurfaceBeat => "strobe.surface_beat",
            EventType::StrobeComfortLimit => "strobe.comfort_limit",
            EventType::StrobeStageChanged => "strobe.stage_changed",
            EventType::StrobeCapstoneUnlocked => "strobe.capstone_unlocked",
            EventType::StrobeClockAdvanced => "strobe.clock_advanced",
            EventType::StrobeSeasonEnded => "strobe.season_ended",
        }
    }
}

/// How a heroine session ended, for payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndedVia {
    /// The player completed the session.
    Completed,
    /// The comfort limit ended it.
    ComfortLimit,
    /// The season ran out mid-session.
    SeasonOver,
}

impl From<SessionEndReason> for EndedVia {
    fn from(reason: SessionEndReason) -> Self {
        match reason {
            SessionEndReason::Completed => EndedVia::Completed,
            SessionEndReason::ComfortLimit => EndedVia::ComfortLimit,
            SessionEndReason::None => EndedVia::SeasonOver,
        }
    }
}

/// Named features for `feature_used` (an enum, never a string).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    /// The chart player ran.
    ChartPlayer,
    /// A save was written.
    SaveWritten,
    /// A save was restored.
    SaveRestored,
    /// The Surface Beat fired.
    SurfaceBeat,
    /// The toy bridge was armed.
    ToyBridgeArmed,
}

/// Named settings for `settings_changed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingName {
    /// Analytics enabled/disabled.
    AnalyticsEnabled,
    /// The install id was rotated.
    InstallIdRotated,
    /// The player haptic ceiling.
    PlayerCeiling,
    /// The toy master switch.
    ToyMaster,
}

/// Named funnel steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunnelStep {
    /// The hub was viewed.
    HubViewed,
    /// The status screen was viewed.
    StatusViewed,
    /// A session was requested.
    SessionRequested,
    /// A session started.
    SessionStarted,
    /// A session completed.
    SessionCompleted,
    /// The aftercare screen was viewed.
    AftercareViewed,
    /// The season ended.
    SeasonEnded,
}

/// The season's ending, for payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeasonEndingName {
    /// It has only begun.
    Begun,
    /// It deepened.
    Deepened,
    /// It has not yet ended.
    NotYet,
    /// She drives it herself now.
    SelfDriven,
}

impl From<strobe_core::SeasonEnding> for SeasonEndingName {
    fn from(ending: strobe_core::SeasonEnding) -> Self {
        match ending {
            strobe_core::SeasonEnding::Begun => SeasonEndingName::Begun,
            strobe_core::SeasonEnding::Deepened => SeasonEndingName::Deepened,
            strobe_core::SeasonEnding::NotYet => SeasonEndingName::NotYet,
            strobe_core::SeasonEnding::SelfDriven => SeasonEndingName::SelfDriven,
        }
    }
}

/// A stage as a bounded payload value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageName {
    /// Stage: Craving.
    Craving,
    /// Stage: Curious.
    Curious,
    /// Stage: Devoted.
    Devoted,
    /// Stage: SelfDriven.
    SelfDriven,
    /// Stage: Unaware.
    Unaware,
    /// Stage: Willing.
    Willing,
}

impl From<W5Stage> for StageName {
    fn from(stage: W5Stage) -> Self {
        match stage {
            W5Stage::Craving => StageName::Craving,
            W5Stage::Curious => StageName::Curious,
            W5Stage::Devoted => StageName::Devoted,
            W5Stage::SelfDriven => StageName::SelfDriven,
            W5Stage::Unaware => StageName::Unaware,
            W5Stage::Willing => StageName::Willing,
        }
    }
}

/// A track as a bounded payload value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackName {
    /// The Gaze track.
    Gaze,
    /// The Rhythm track.
    Rhythm,
    /// The Touch track.
    Touch,
    /// The Voice track.
    Voice,
}

impl From<ResponseTrack> for TrackName {
    fn from(track: ResponseTrack) -> Self {
        match track {
            ResponseTrack::Gaze => TrackName::Gaze,
            ResponseTrack::Rhythm => TrackName::Rhythm,
            ResponseTrack::Touch => TrackName::Touch,
            ResponseTrack::Voice => TrackName::Voice,
        }
    }
}

/// An app as a bounded payload value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppName {
    /// The Aftercare app.
    Aftercare,
    /// The Feed app.
    Feed,
    /// The Protocols app.
    Protocols,
    /// The Session app.
    Session,
    /// The Status app.
    Status,
    /// The Store app.
    Store,
}

impl From<PhoneApp> for AppName {
    fn from(app: PhoneApp) -> Self {
        match app {
            PhoneApp::Aftercare => AppName::Aftercare,
            PhoneApp::Feed => AppName::Feed,
            PhoneApp::Protocols => AppName::Protocols,
            PhoneApp::Session => AppName::Session,
            PhoneApp::Status => AppName::Status,
            PhoneApp::Store => AppName::Store,
        }
    }
}

/// The payload union. Every variant is enums, booleans, and
/// bounded numbers — there is no string payload anywhere.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    /// level_start / level_complete / session boundaries.
    Session {
        /// Tier reached in the session.
        tier: i32,
        /// How it ended (session_end / level_complete only).
        #[serde(skip_serializing_if = "Option::is_none")]
        ended_via: Option<EndedVia>,
    },
    /// strobe.tier_unlocked.
    TierUnlocked {
        /// The new tier.
        tier: i32,
    },
    /// strobe.surface_beat.
    SurfaceBeat {
        /// The tier at which it fired.
        tier: i32,
        /// Whether it was the first surface of the session.
        first: bool,
    },
    /// strobe.comfort_limit.
    ComfortLimit {
        /// The tier the player attempted.
        attempted_tier: i32,
        /// The tier allowed at the time.
        allowed_tier: i32,
    },
    /// strobe.stage_changed.
    StageChanged {
        /// The stage entered.
        stage: StageName,
    },
    /// strobe.capstone_unlocked.
    CapstoneUnlocked {
        /// The track whose capstone unlocked.
        track: TrackName,
    },
    /// strobe.clock_advanced.
    ClockAdvanced {
        /// The day advanced to (bounded by the season length).
        day: i32,
        /// Whether the season is over.
        season_over: bool,
    },
    /// strobe.season_ended.
    SeasonEnded {
        /// The ending reached.
        ending: SeasonEndingName,
        /// Clears banked in the season.
        clears: i32,
    },
    /// feature_used.
    FeatureUsed {
        /// The feature used.
        feature: Feature,
    },
    /// settings_changed.
    SettingsChanged {
        /// The setting changed.
        setting: SettingName,
        /// Its new boolean value, when boolean.
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    /// funnel_step.
    FunnelStep {
        /// The step reached.
        step: FunnelStep,
    },
    /// App install/upgrade facts ride as feature-adjacent payloads
    /// on the session boundary events; this variant carries them
    /// when a hub change is itself the event (feature_used).
    AppChanged {
        /// The app that changed.
        app: AppName,
        /// Its content version after the change.
        version: i32,
    },
    /// session_start / session_end of the app itself.
    AppSession {
        /// Whether a save was restored at boot.
        restored_save: bool,
    },
}

/// One analytics envelope (SPEC §6): fixed identity fields plus a
/// typed payload. `ts_coarse` is day-granularity by construction —
/// it is stamped from the campaign clock, never a wall clock.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Envelope {
    /// Envelope schema version.
    pub schema_version: u32,
    /// Always "strobe".
    pub title_id: &'static str,
    /// Always "ntranch".
    pub lane: &'static str,
    /// The app version.
    pub app_version: String,
    /// The per-install random id (rotatable in Settings).
    pub install_id: String,
    /// The per-app-session id.
    pub session_id: String,
    /// Coarse timestamp: the campaign day, `day-<n>`.
    pub ts_coarse: String,
    /// The event type.
    #[serde(rename = "event_type")]
    pub event_type_name: &'static str,
    /// The typed payload.
    pub payload: Payload,
}

/// An analytics sink. Implementations must not fail into gameplay:
/// the game treats every emit as best-effort.
pub trait EventSink: Send + Sync {
    /// Emit one envelope.
    fn emit(&self, envelope: &Envelope);
}

/// The null sink: events go nowhere (sink-off runs, tests).
pub struct NullEventSink;

impl EventSink for NullEventSink {
    fn emit(&self, _envelope: &Envelope) {}
}

/// The JSONL file sink: one envelope per line, appended.
pub struct JsonlEventSink {
    path: PathBuf,
    degraded: std::sync::Mutex<bool>,
}

impl JsonlEventSink {
    /// A sink appending to `path`.
    #[must_use]
    pub fn new(path: PathBuf) -> JsonlEventSink {
        JsonlEventSink {
            path,
            degraded: std::sync::Mutex::new(false),
        }
    }

    /// True once a write has failed and the sink degraded to null.
    #[must_use]
    pub fn degraded(&self) -> bool {
        self.degraded.lock().map(|flag| *flag).unwrap_or(true)
    }
}

impl JsonlEventSink {
    /// Mark the sink degraded (it behaves as the null sink from
    /// here on). A poisoned lock reads as degraded, never panics.
    fn degrade(&self) {
        if let Ok(mut flag) = self.degraded.lock() {
            *flag = true;
        }
    }
}

impl EventSink for JsonlEventSink {
    fn emit(&self, envelope: &Envelope) {
        if self.degraded() {
            return;
        }
        let line = match serde_json::to_string(envelope) {
            Ok(line) => line,
            Err(_) => {
                self.degrade();
                return;
            }
        };
        let result = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .and_then(|mut file| writeln!(file, "{line}"));
        if result.is_err() {
            self.degrade();
        }
    }
}
