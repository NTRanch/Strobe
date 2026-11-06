//! One heroine's persistent progression state: her W5 stage
//! (derived, never stored independently), her response-track XP,
//! her capstones, her corruption baseline, her outfit currency,
//! and whether her frequency has been learned.
//!
//! Persistence is the caller's problem (the save layer serializes
//! the plain fields through its snapshot type); this type owns the
//! invariants:
//! - Clear count never decreases and never goes negative.
//! - Stage is always `Progression::stage_for_count` of the count; a
//!   stored stage is never trusted over the ladder.
//! - Track XP never decreases; capstones, once unlocked, stay.
//! - Corruption baseline is a pure function of stage (A2): it only
//!   ever grows across the campaign.

use crate::error::CoreError;
use crate::progression::Progression;
use crate::response_track::{ResponseTrack, TrackLadder};
use crate::session::Session;
use crate::w5_stage::W5Stage;

/// One heroine's persistent progression state.
#[derive(Clone, Debug, PartialEq)]
pub struct HeroineState {
    heroine_id: String,
    clear_count: i32,
    outfit_coins: i32,
    frequency_learned: bool,
    track_xp: [i32; 4],
    capstones: [bool; 4],
}

impl HeroineState {
    /// A fresh state for one heroine: zero clears, zero XP, no
    /// capstones, no coins, frequency unlearned.
    ///
    /// # Errors
    /// [`CoreError::Argument`] when `heroine_id` is empty.
    pub fn new(heroine_id: &str) -> Result<HeroineState, CoreError> {
        if heroine_id.is_empty() {
            return Err(CoreError::Argument {
                param: "heroine_id",
                message: "Heroine id must be non-empty.",
            });
        }
        Ok(HeroineState {
            heroine_id: heroine_id.to_owned(),
            clear_count: 0,
            outfit_coins: 0,
            frequency_learned: false,
            track_xp: [0; 4],
            capstones: [false; 4],
        })
    }

    /// The heroine this state belongs to.
    #[must_use]
    pub fn heroine_id(&self) -> &str {
        &self.heroine_id
    }

    /// Lifetime clears (hypnosis count). Monotonic.
    #[must_use]
    pub fn clear_count(&self) -> i32 {
        self.clear_count
    }

    /// Soft currency earned by session performance (A3/D.10).
    #[must_use]
    pub fn outfit_coins(&self) -> i32 {
        self.outfit_coins
    }

    /// True once her frequency sweet spot has been found in play.
    #[must_use]
    pub fn frequency_learned(&self) -> bool {
        self.frequency_learned
    }

    /// Her current W5 stage, derived from the clear count.
    #[must_use]
    pub fn stage(&self) -> W5Stage {
        Progression::stage_for_count(self.clear_count)
    }

    /// Session corruption starts here (ADAPT-SPEC A2): the share of
    /// the corruption bar her campaign growth pre-fills. Stage × 12.
    #[must_use]
    pub fn corruption_baseline(&self) -> f64 {
        f64::from(self.stage().discriminant()) * 12.0
    }

    /// Highest session tier her stage alone lets her ask for.
    #[must_use]
    pub fn stage_tier_ceiling(&self) -> i32 {
        Session::MAX_TIER.min(self.stage().discriminant())
    }

    pub(crate) fn add_clear(&mut self) {
        self.clear_count += 1;
    }

    /// Restore a persisted clear count (load path; clamps at 0).
    pub fn restore_clear_count(&mut self, clear_count: i32) {
        self.clear_count = clear_count.max(0);
    }

    /// Cumulative XP on one response track.
    #[must_use]
    pub fn track_xp(&self, track: ResponseTrack) -> i32 {
        self.track_xp[track.slot()]
    }

    /// Level on one response track (0..=TrackLadder::MAX_LEVEL).
    #[must_use]
    pub fn track_level(&self, track: ResponseTrack) -> i32 {
        TrackLadder::level_for_xp(self.track_xp[track.slot()])
    }

    /// True when the track's capstone ability is unlocked.
    #[must_use]
    pub fn has_capstone(&self, track: ResponseTrack) -> bool {
        self.capstones[track.slot()]
    }

    /// All unlocked capstone tracks, in enum order.
    #[must_use]
    pub fn capstones(&self) -> Vec<ResponseTrack> {
        ResponseTrack::ALL
            .into_iter()
            .filter(|track| self.capstones[track.slot()])
            .collect()
    }

    /// Award track XP (session path only). Returns true exactly
    /// when this award unlocks the track's capstone.
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `xp` is negative.
    pub fn award_track_xp(&mut self, track: ResponseTrack, xp: i32) -> Result<bool, CoreError> {
        if xp < 0 {
            return Err(CoreError::OutOfRange {
                param: "xp",
                message: "Track XP awards cannot be negative.",
            });
        }
        let slot = track.slot();
        self.track_xp[slot] += xp;
        if !self.capstones[slot] && TrackLadder::is_capstone(self.track_xp[slot]) {
            self.capstones[slot] = true;
            return Ok(true);
        }
        Ok(false)
    }

    /// Award track XP on the session path, where the caller has
    /// already established the award is non-negative (the session's
    /// award formula floors at 1). Returns true exactly when this
    /// award unlocks the track's capstone.
    pub(crate) fn award_track_xp_session(&mut self, track: ResponseTrack, xp: i32) -> bool {
        assert!(
            xp >= 0,
            "session XP awards are non-negative by construction"
        );
        let slot = track.slot();
        self.track_xp[slot] += xp;
        if !self.capstones[slot] && TrackLadder::is_capstone(self.track_xp[slot]) {
            self.capstones[slot] = true;
            return true;
        }
        false
    }

    /// Restore persisted track XP (load path; clamps at 0,
    /// re-derives the capstone).
    pub fn restore_track_xp(&mut self, track: ResponseTrack, xp: i32) {
        let slot = track.slot();
        self.track_xp[slot] = xp.max(0);
        self.capstones[slot] = TrackLadder::is_capstone(self.track_xp[slot]);
    }

    /// Credit outfit coins from session performance.
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `coins` is negative.
    pub fn add_outfit_coins(&mut self, coins: i32) -> Result<(), CoreError> {
        if coins < 0 {
            return Err(CoreError::OutOfRange {
                param: "coins",
                message: "Coin awards cannot be negative.",
            });
        }
        self.outfit_coins += coins;
        Ok(())
    }

    /// Restore persisted coins (load path; clamps at 0).
    pub fn restore_outfit_coins(&mut self, coins: i32) {
        self.outfit_coins = coins.max(0);
    }

    /// Mark her frequency as learned (session discovery).
    pub fn learn_frequency(&mut self) {
        self.frequency_learned = true;
    }

    /// Restore the learned flag (load path).
    pub fn restore_frequency_learned(&mut self, learned: bool) {
        self.frequency_learned = learned;
    }
}
