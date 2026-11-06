//! Strobe core: the pure game model, ported from the C# reference
//! implementation in primo `~/strobe-trackb/Assets/Scripts/Core/`.
//!
//! The C# is the executable specification: where this port and the C#
//! disagree about a number, the C# as-built wins until a ruling
//! changes it (SPEC-BEVY, header). The port is verified case-by-case
//! against the C# test suite (82 cases; 81 runtime tests here plus
//! `RecordClear_NullStateThrows`, discharged by construction —
//! SPEC-BEVY §8.1).
//!
//! Purity contract, enforced by the dependency graph: no engine, no
//! IO, no clock, no RNG. One action in, one result out; every value
//! is a deterministic function of the inputs.

#![forbid(unsafe_code)]

pub mod campaign_clock;
pub mod error;
pub mod heroine_profile;
pub mod heroine_state;
pub mod phone_hub;
pub mod progression;
pub mod response_track;
pub mod session;
pub mod surface_beat;
pub mod toy_output;
pub mod w5_stage;

pub use campaign_clock::{CampaignClock, SeasonEnding};
pub use error::CoreError;
pub use heroine_profile::{HeroineProfile, KinkVectorKind};
pub use heroine_state::HeroineState;
pub use phone_hub::{PhoneApp, PhoneHub};
pub use progression::Progression;
pub use response_track::{ResponseTrack, TrackLadder};
pub use session::{ActionResult, Session, SessionAction, SessionEndReason, SessionSummary};
pub use surface_beat::{ReactionGrammar, SurfaceReaction, SurfaceResult, SurfaceTrigger};
pub use toy_output::{NullToyOutput, ToyCurve, ToyOutput};
pub use w5_stage::W5Stage;
