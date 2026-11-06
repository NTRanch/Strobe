//! W3/W5 progression: maps a heroine's lifetime clear count to her
//! W5 stage through the sidecar ladder inherited from Track A
//! (0/1/3/6/10/15 clears → stages 0..5). Pure mapping; state lives
//! in [`crate::heroine_state::HeroineState`].
//!
//! Invariants (carried from the Track A engine):
//! - Monotonic: as a count grows, the stage never decreases.
//! - No decay, no punishment states: nothing here ever lowers a
//!   count or a stage.
//! - Counts at or below 0 map to stage 0; negative stored counts
//!   are clamped to 0 by `HeroineState`.

use crate::heroine_state::HeroineState;
use crate::w5_stage::W5Stage;

// Ladder thresholds, ascending, one per stage 1..5.
const THRESHOLDS: [i32; 5] = [1, 3, 6, 10, 15];

/// The pure count → stage mapping.
pub struct Progression;

impl Progression {
    /// The pure mapping: lifetime clear count → stage.
    #[must_use]
    pub fn stage_for_count(clear_count: i32) -> W5Stage {
        if clear_count <= 0 {
            return W5Stage::Unaware;
        }
        let mut stage = 0;
        for (index, threshold) in THRESHOLDS.iter().enumerate() {
            if clear_count >= *threshold {
                stage = index + 1;
            }
        }
        match stage {
            0 => W5Stage::Unaware,
            1 => W5Stage::Curious,
            2 => W5Stage::Willing,
            3 => W5Stage::Craving,
            4 => W5Stage::Devoted,
            _ => W5Stage::SelfDriven,
        }
    }

    /// Clears still needed to reach the next stage; 0 at the top.
    #[must_use]
    pub fn clears_to_next_stage(clear_count: i32) -> i32 {
        let stage = Self::stage_for_count(clear_count).discriminant();
        if stage as usize >= THRESHOLDS.len() {
            return 0;
        }
        (THRESHOLDS[stage as usize] - clear_count.max(0)).max(0)
    }

    /// Record one completed clear (a finished session at any tier)
    /// for a heroine. Returns her stage after the clear.
    ///
    /// The C# throws `ArgumentNullException` on a null state; the
    /// port takes `&mut HeroineState`, so the guarantee holds by
    /// construction (SPEC-BEVY §8.1(b) — the one C# case discharged
    /// by the type system rather than by a runtime test).
    pub fn record_clear(state: &mut HeroineState) -> W5Stage {
        state.add_clear();
        state.stage()
    }
}
