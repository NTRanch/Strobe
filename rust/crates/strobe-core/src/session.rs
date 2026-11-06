//! The A2 session model: Response × Corruption.
//!
//! - Response is her in-the-moment trance/arousal depth. It unlocks
//!   nothing by itself and resets between sessions.
//! - Corruption is her growing desire to dive deeper into HER OWN
//!   kink. It rises only through kink-aligned action, decays on
//!   off-kink action or rushing, and is the only gate on the tier
//!   ladder. It is not a resistance bar: nothing here is "broken"
//!   or "overcome".
//! - An action pitched above the tier her corruption has unlocked
//!   meets her comfort limit: the session ends gracefully, XP
//!   already earned is KEPT (zero progress loss), no clear is
//!   recorded, and an aftercare beat is required.
//! - Composure is a pacing readout (strain), never a fail bar.
//! - The Surface Beat (SCENARIO.md) is a pacing event inside the
//!   session: he brings her out of trance, her authored reaction
//!   plays, and play resumes through `resume_trance`. Surfacing is
//!   never a failure and never ends the session; the beat's only
//!   corruption source is a resume she initiates herself.
//!
//! Deterministic and allocation-light: one action in, one result
//! out, no clocks, no randomness. Frequency response is a fixed
//! gaussian around her profile's center. All arithmetic follows
//! the C# expression shapes exactly, so float assertions hold
//! bit-for-bit where the C# tests assert exact values.

use crate::error::CoreError;
use crate::heroine_profile::HeroineProfile;
use crate::heroine_state::HeroineState;
use crate::progression::Progression;
use crate::response_track::ResponseTrack;
use crate::surface_beat::{SurfaceReaction, SurfaceResult, SurfaceTrigger};
use crate::w5_stage::W5Stage;

/// Why a session ended. None while it is still running.
/// Discriminants port verbatim from the C#.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum SessionEndReason {
    /// Still running.
    None = 0,
    /// Ended by `Session::complete`.
    Completed = 1,
    /// Ended by her comfort limit.
    ComfortLimit = 2,
}

/// One player action inside a session (ADAPT-SPEC A2). An action
/// belongs to a tier (its depth in HER kink ladder), works one
/// response track, sits somewhere on the frequency dial, and is
/// either aligned with her recorded kink vector or not. Alignment
/// is authored per content beat, never guessed by this model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SessionAction {
    track: ResponseTrack,
    tier: i32,
    kink_aligned: bool,
    rushed: bool,
    initiated_by_her: bool,
    dial_position: f64,
}

impl SessionAction {
    /// Build one action.
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `tier` is outside
    /// 0..=`Session::MAX_TIER` or `dial_position` outside 0.0..=1.0.
    pub fn new(
        track: ResponseTrack,
        tier: i32,
        kink_aligned: bool,
        rushed: bool,
        initiated_by_her: bool,
        dial_position: f64,
    ) -> Result<SessionAction, CoreError> {
        if !(0..=Session::MAX_TIER).contains(&tier) {
            return Err(CoreError::OutOfRange {
                param: "tier",
                message: "Action tier is 0..Session::MAX_TIER.",
            });
        }
        if !(0.0..=1.0).contains(&dial_position) {
            return Err(CoreError::OutOfRange {
                param: "dial_position",
                message: "Dial position is 0..1.",
            });
        }
        Ok(SessionAction {
            track,
            tier,
            kink_aligned,
            rushed,
            initiated_by_her,
            dial_position,
        })
    }

    /// The response track this action works.
    #[must_use]
    pub fn track(&self) -> ResponseTrack {
        self.track
    }

    /// Depth of this action in her kink ladder, 0..=MAX_TIER.
    #[must_use]
    pub fn tier(&self) -> i32 {
        self.tier
    }

    /// True when the action matches her recorded desire/block.
    #[must_use]
    pub fn kink_aligned(&self) -> bool {
        self.kink_aligned
    }

    /// True when the action jumps pacing (pushing too fast).
    #[must_use]
    pub fn rushed(&self) -> bool {
        self.rushed
    }

    /// True when she asked for this step (Craving+ behaviour).
    #[must_use]
    pub fn initiated_by_her(&self) -> bool {
        self.initiated_by_her
    }

    /// Where on the frequency dial the action sits, 0..=1.
    #[must_use]
    pub fn dial_position(&self) -> f64 {
        self.dial_position
    }
}

/// What one action changed. Returned by `Session::apply_action`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionResult {
    response_delta: f64,
    corruption_delta: f64,
    tier_unlocked: i32,
    unlock_initiated_by_her: bool,
    frequency_discovered: bool,
    capstone_unlocked: bool,
    ended_by_comfort_limit: bool,
}

impl ActionResult {
    pub(crate) fn new(
        response_delta: f64,
        corruption_delta: f64,
        tier_unlocked: i32,
        unlock_initiated_by_her: bool,
        frequency_discovered: bool,
        capstone_unlocked: bool,
        ended_by_comfort_limit: bool,
    ) -> Self {
        ActionResult {
            response_delta,
            corruption_delta,
            tier_unlocked,
            unlock_initiated_by_her,
            frequency_discovered,
            capstone_unlocked,
            ended_by_comfort_limit,
        }
    }

    /// Change in Response from this action.
    #[must_use]
    pub fn response_delta(&self) -> f64 {
        self.response_delta
    }

    /// Change in Corruption from this action.
    #[must_use]
    pub fn corruption_delta(&self) -> f64 {
        self.corruption_delta
    }

    /// Newly unlocked tier, or -1 when none.
    #[must_use]
    pub fn tier_unlocked(&self) -> i32 {
        self.tier_unlocked
    }

    /// For a tier unlock: true when the unlocking action was hers
    /// (she initiated). A2 law: every unlock is signaled BY HER —
    /// the presentation layer must surface a signal (voice line,
    /// expression delta, or her initiation) for any unlock; this
    /// flag tells it which kind fired.
    #[must_use]
    pub fn unlock_initiated_by_her(&self) -> bool {
        self.unlock_initiated_by_her
    }

    /// True when this action discovered her frequency.
    #[must_use]
    pub fn frequency_discovered(&self) -> bool {
        self.frequency_discovered
    }

    /// True when this action unlocked a track capstone.
    #[must_use]
    pub fn capstone_unlocked(&self) -> bool {
        self.capstone_unlocked
    }

    /// True when this action ended the session at her comfort limit.
    #[must_use]
    pub fn ended_by_comfort_limit(&self) -> bool {
        self.ended_by_comfort_limit
    }
}

/// Session outcome, produced by `Session::complete`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SessionSummary {
    tier_reached: i32,
    response_peak: f64,
    coins_awarded: i32,
    stage_after: W5Stage,
    surfaces: i32,
}

impl SessionSummary {
    pub(crate) fn new(
        tier_reached: i32,
        response_peak: f64,
        coins_awarded: i32,
        stage_after: W5Stage,
        surfaces: i32,
    ) -> Self {
        SessionSummary {
            tier_reached,
            response_peak,
            coins_awarded,
            stage_after,
            surfaces,
        }
    }

    /// Deepest tier reached this session.
    #[must_use]
    pub fn tier_reached(&self) -> i32 {
        self.tier_reached
    }

    /// Highest Response reached this session.
    #[must_use]
    pub fn response_peak(&self) -> f64 {
        self.response_peak
    }

    /// Outfit coins paid by this session.
    #[must_use]
    pub fn coins_awarded(&self) -> i32 {
        self.coins_awarded
    }

    /// Her stage after the recorded clear.
    #[must_use]
    pub fn stage_after(&self) -> W5Stage {
        self.stage_after
    }

    /// Surface Beats played this session (see `Session::surface`).
    #[must_use]
    pub fn surfaces(&self) -> i32 {
        self.surfaces
    }
}

// Fraction of Response kept across a surface. Surfacing shallows
// trance; it never zeroes it — she becomes aware inside an
// unchanged situation.
const SURFACE_RETENTION: f64 = 0.45;

// Composure strain added by a surface: the fluster is a pacing
// spike the player should respect, never a fail.
const SURFACE_STRAIN: f64 = 0.35;

// Response multiplier on the action that re-enters trance: the
// second descent is faster than the first.
const RE_ENTRY_RESPONSE_FACTOR: f64 = 1.5;

// Corruption bonus when the resume action is kink-aligned AND
// initiated by her: she chose to go back under, which is
// kink-aligned self-recognition (SCENARIO.md phase 4 to 5).
const SELF_RECOGNITION_BONUS: f64 = 4.0;

// Corruption thresholds for tiers 1..4.
const TIER_THRESHOLDS: [f64; 4] = [20.0, 40.0, 60.0, 80.0];

/// One live session over a heroine's profile and persistent state.
///
/// Ownership shape: the C# session holds a reference to the state
/// object and mutates it in place. The port keeps that shape
/// without a second borrow regime: the session borrows the
/// *profile* (immutable design data), and every method that can
/// mutate persistent state (track XP, frequency discovery, the
/// completion clear) takes the caller's `&mut HeroineState` for
/// that call — the same places the C# mutated its state object.
/// The session stores the heroine's id at construction and
/// re-validates it per call, so a session can never be driven
/// against another heroine's state.
pub struct Session<'a> {
    profile: &'a HeroineProfile,
    heroine_id: String,
    response: f64,
    response_peak: f64,
    corruption: f64,
    composure_strain: f64,
    current_tier: i32,
    end_reason: SessionEndReason,
    is_surfaced: bool,
    surfaces_this_session: i32,
    session_track_xp: [i32; 4],
}

impl<'a> Session<'a> {
    /// Deepest tier in any heroine's kink ladder.
    pub const MAX_TIER: i32 = 4;

    /// Most Surface Beats one session may hold.
    pub const MAX_SURFACES_PER_SESSION: i32 = 12;

    /// Open a session. Corruption starts at her state's baseline.
    ///
    /// # Errors
    /// [`CoreError::Argument`] when the profile and the state name
    /// different heroines.
    pub fn new(
        profile: &'a HeroineProfile,
        state: &HeroineState,
    ) -> Result<Session<'a>, CoreError> {
        if profile.id() != state.heroine_id() {
            return Err(CoreError::Argument {
                param: "state",
                message: "Profile and state are for different heroines.",
            });
        }
        let corruption = state.corruption_baseline();
        Ok(Session {
            profile,
            heroine_id: state.heroine_id().to_owned(),
            response: 0.0,
            response_peak: 0.0,
            corruption,
            composure_strain: 0.0,
            current_tier: 0,
            end_reason: SessionEndReason::None,
            is_surfaced: false,
            surfaces_this_session: 0,
            session_track_xp: [0; 4],
        })
    }

    /// Her trance/arousal depth right now, 0..=100.
    #[must_use]
    pub fn response(&self) -> f64 {
        self.response
    }

    /// Highest Response reached this session.
    #[must_use]
    pub fn response_peak(&self) -> f64 {
        self.response_peak
    }

    /// Her corruption to go deeper, 0..=100. The progression bar.
    #[must_use]
    pub fn corruption(&self) -> f64 {
        self.corruption
    }

    /// Pacing readout, 0..=1. High strain = slow down. Never a fail.
    #[must_use]
    pub fn composure_strain(&self) -> f64 {
        self.composure_strain
    }

    /// Deepest tier unlocked this session (sticky once unlocked).
    #[must_use]
    pub fn current_tier(&self) -> i32 {
        self.current_tier
    }

    /// Why the session ended; `None` while it is still running.
    #[must_use]
    pub fn end_reason(&self) -> SessionEndReason {
        self.end_reason
    }

    /// True after a comfort-limit ending: the aftercare beat is owed.
    #[must_use]
    pub fn aftercare_required(&self) -> bool {
        self.end_reason == SessionEndReason::ComfortLimit
    }

    /// True between `surface` and `resume_trance`: she is out of
    /// trance and aware. Ordinary actions refuse in this state; the
    /// session itself is alive and may also be completed.
    #[must_use]
    pub fn is_surfaced(&self) -> bool {
        self.is_surfaced
    }

    /// Surface Beats played so far this session.
    #[must_use]
    pub fn surfaces_this_session(&self) -> i32 {
        self.surfaces_this_session
    }

    /// True when she is asking for the next tier herself: her stage
    /// is Craving+ and her corruption already covers the next
    /// threshold. The presentation layer surfaces this as her
    /// request (A2: at Craving she initiates; at Self-Driven she
    /// requests in advance).
    #[must_use]
    pub fn heroine_requests_next_tier(&self, state: &HeroineState) -> bool {
        if self.end_reason != SessionEndReason::None || self.current_tier >= Self::MAX_TIER {
            return false;
        }
        state.stage() >= W5Stage::Craving
            && self.corruption >= TIER_THRESHOLDS[self.current_tier as usize]
    }

    fn validate_state(&self, state: &HeroineState) -> Result<(), CoreError> {
        if state.heroine_id() != self.heroine_id {
            return Err(CoreError::Argument {
                param: "state",
                message: "Profile and state are for different heroines.",
            });
        }
        Ok(())
    }

    /// The tier her current corruption would unlock.
    #[must_use]
    pub fn tier_allowed_by_corruption(&self) -> i32 {
        let mut allowed = 0;
        for (index, threshold) in TIER_THRESHOLDS.iter().enumerate() {
            if self.corruption >= *threshold {
                allowed = index + 1;
            }
        }
        allowed as i32
    }

    /// Frequency response multiplier for a dial position,
    /// 0.35..=1: a fixed gaussian (σ0.18) around her profile's
    /// center, floored at 0.35.
    #[must_use]
    pub fn frequency_factor(&self, dial_position: f64) -> f64 {
        let delta = dial_position - self.profile.frequency_center();
        let gaussian = (-(delta * delta) / (2.0 * 0.18 * 0.18)).exp();
        0.35 + 0.65 * gaussian
    }

    /// Bring her out of trance for one Surface Beat. Response
    /// shallows by the retention factor and composure strain
    /// spikes; corruption is untouched and the session does not
    /// end. The reaction is authored per heroine and supplied by
    /// the caller; the presentation layer must play it (A2 law:
    /// the beat is signaled BY HER — her line, her face).
    ///
    /// `reaction` is an `Option` because the C# parameter is a
    /// nullable reference: `None` is its `ArgumentNullException`.
    ///
    /// # Errors
    /// - [`CoreError::NullArgument`] when `reaction` is `None`.
    /// - [`CoreError::InvalidOperation`] when the session has
    ///   ended, she is already surfaced, or this session's Surface
    ///   Beats are exhausted.
    pub fn surface(
        &mut self,
        _trigger: SurfaceTrigger,
        reaction: Option<SurfaceReaction>,
    ) -> Result<SurfaceResult, CoreError> {
        let reaction = reaction.ok_or(CoreError::NullArgument { param: "reaction" })?;
        if self.end_reason != SessionEndReason::None {
            return Err(CoreError::InvalidOperation {
                message: "The session has ended.",
            });
        }
        if self.is_surfaced {
            return Err(CoreError::InvalidOperation {
                message: "She is already surfaced; resume her trance first.",
            });
        }
        if self.surfaces_this_session >= Self::MAX_SURFACES_PER_SESSION {
            return Err(CoreError::InvalidOperation {
                message: "This session's Surface Beats are exhausted.",
            });
        }
        let before = self.response;
        self.response = clamp100(self.response * SURFACE_RETENTION);
        self.composure_strain = (self.composure_strain + SURFACE_STRAIN).min(1.0);
        self.is_surfaced = true;
        self.surfaces_this_session += 1;
        Ok(SurfaceResult::new(
            self.response - before,
            reaction,
            self.surfaces_this_session,
        ))
    }

    /// Resume play from a surface: the action that takes her back
    /// under. It lands at the re-entry response factor (the second
    /// descent is faster), and a kink-aligned action she initiated
    /// earns the self-recognition corruption bonus — she chose to
    /// go back. Every ordinary rule still applies, including the
    /// comfort limit on an overreaching resume.
    ///
    /// # Errors
    /// [`CoreError::InvalidOperation`] when the session has ended
    /// or she is not surfaced.
    pub fn resume_trance(
        &mut self,
        state: &mut HeroineState,
        action: SessionAction,
    ) -> Result<ActionResult, CoreError> {
        self.validate_state(state)?;
        if self.end_reason != SessionEndReason::None {
            return Err(CoreError::InvalidOperation {
                message: "The session has ended.",
            });
        }
        if !self.is_surfaced {
            return Err(CoreError::InvalidOperation {
                message: "She is not surfaced; there is no trance to resume.",
            });
        }
        self.is_surfaced = false;
        let bonus = if action.kink_aligned() && action.initiated_by_her() {
            SELF_RECOGNITION_BONUS
        } else {
            0.0
        };
        Ok(self.apply_core(state, action, RE_ENTRY_RESPONSE_FACTOR, bonus))
    }

    /// Apply one action. See the module remarks for the rules.
    ///
    /// # Errors
    /// [`CoreError::InvalidOperation`] when the session has ended
    /// or she is surfaced.
    pub fn apply_action(
        &mut self,
        state: &mut HeroineState,
        action: SessionAction,
    ) -> Result<ActionResult, CoreError> {
        self.validate_state(state)?;
        if self.end_reason != SessionEndReason::None {
            return Err(CoreError::InvalidOperation {
                message: "The session has ended.",
            });
        }
        if self.is_surfaced {
            return Err(CoreError::InvalidOperation {
                message: "She is surfaced; resume her trance or complete the session.",
            });
        }
        Ok(self.apply_core(state, action, 1.0, 0.0))
    }

    fn apply_core(
        &mut self,
        state: &mut HeroineState,
        action: SessionAction,
        response_multiplier: f64,
        corruption_bonus: f64,
    ) -> ActionResult {
        // Pushing past her corruption: her comfort limit ends the
        // session gracefully. Nothing is rolled back.
        if action.tier() > self.current_tier && action.tier() > self.tier_allowed_by_corruption() {
            self.end_reason = SessionEndReason::ComfortLimit;
            self.composure_strain = 1.0;
            return ActionResult::new(0.0, 0.0, -1, false, false, false, true);
        }

        let factor = self.frequency_factor(action.dial_position());

        // Response: any action lands; aligned and on-frequency lands best.
        let response_delta =
            12.0 * factor * if action.kink_aligned() { 1.0 } else { 0.6 } * response_multiplier;
        self.response = clamp100(self.response + response_delta);
        if self.response > self.response_peak {
            self.response_peak = self.response;
        }

        // Corruption: only kink-aligned action raises it. Rushing
        // still counts when aligned (she likes it; pacing costs),
        // but buys less. Off-kink action always costs.
        let mut corruption_delta = if action.kink_aligned() && !action.rushed() {
            8.0 * factor
        } else if action.kink_aligned() {
            3.0
        } else if action.rushed() {
            -10.0
        } else {
            -6.0
        };
        corruption_delta += corruption_bonus;
        self.corruption = clamp100(self.corruption + corruption_delta);

        // Composure strain: a pacing cue only.
        if action.rushed() || !action.kink_aligned() {
            self.composure_strain = (self.composure_strain + 0.3).min(1.0);
        } else {
            self.composure_strain = (self.composure_strain - 0.15).max(0.0);
        }

        // Tier unlock (sticky). Response alone never unlocks: the
        // corruption threshold is the only key.
        let mut tier_unlocked = -1;
        let allowed = self.tier_allowed_by_corruption();
        if allowed > self.current_tier {
            self.current_tier = allowed;
            tier_unlocked = allowed;
        }

        // Frequency discovery: a near-center aligned action finds it.
        let mut discovered = false;
        if action.kink_aligned()
            && !state.frequency_learned()
            && (action.dial_position() - self.profile.frequency_center()).abs() <= 0.05
        {
            state.learn_frequency();
            discovered = true;
        }

        // Track XP: aligned play only, diminishing within a session.
        // Her profile weight scales this award and nothing else.
        // Track XP is integral, so the scaled award is rounded to
        // the nearest integer (midpoints away from zero) and keeps
        // the existing minimum of 1 for an aligned action.
        let mut capstone = false;
        if action.kink_aligned() {
            let slot = action.track().slot();
            let base_award = (6 - self.session_track_xp[slot] / 15).max(1);
            let weight = self.profile.track_weight(action.track());
            let award = ((base_award as f64 * weight).round() as i32).max(1);
            self.session_track_xp[slot] += award;
            capstone = state.award_track_xp_session(action.track(), award);
        }

        ActionResult::new(
            response_delta,
            corruption_delta,
            tier_unlocked,
            tier_unlocked >= 0 && action.initiated_by_her(),
            discovered,
            capstone,
            false,
        )
    }

    /// End the session as completed: records the clear (hypnosis
    /// count +1, stage re-derived), pays outfit coins from depth
    /// and response, and returns the summary.
    ///
    /// # Errors
    /// [`CoreError::InvalidOperation`] when the session has already
    /// ended.
    pub fn complete(&mut self, state: &mut HeroineState) -> Result<SessionSummary, CoreError> {
        self.validate_state(state)?;
        if self.end_reason != SessionEndReason::None {
            return Err(CoreError::InvalidOperation {
                message: "The session has already ended.",
            });
        }
        self.end_reason = SessionEndReason::Completed;
        let coins = self.current_tier * 5 + (self.response_peak / 10.0) as i32;
        let coin_result = state.add_outfit_coins(coins);
        assert!(
            coin_result.is_ok(),
            "session coins are non-negative by construction"
        );
        let stage = Progression::record_clear(state);
        Ok(SessionSummary::new(
            self.current_tier,
            self.response_peak,
            coins,
            stage,
            self.surfaces_this_session,
        ))
    }
}

fn clamp100(value: f64) -> f64 {
    // Same shape as the hand-rolled chain it replaces, including
    // NaN passthrough (the bounds are never NaN).
    value.clamp(0.0, 100.0)
}
