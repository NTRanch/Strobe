//! ADAPT-SPEC A4: one heroine's season is a bounded clock of days
//! with a fixed number of action slots per day. Sessions, hub
//! beats, and feed beats each cost a slot. When the clock runs out
//! the season is evaluated against her stage — failure is an
//! ending, never a wall, and the player's pacing resource (Focus)
//! is a budgeting constraint, not a survival stat.

use crate::error::CoreError;
use crate::w5_stage::W5Stage;

/// How a heroine's season ends when its clock runs out (A4).
/// Discriminants port verbatim from the C#.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum SeasonEnding {
    /// She reached Self-Driven: the full arc, in her own time.
    SelfDriven = 0,
    /// Craving or Devoted: a real arc, unfinished by days not by failure.
    Deepened = 1,
    /// Curious or Willing: a beginning. The season can be replayed.
    Begun = 2,
    /// Still Unaware: the softest ending. Not a game-over, a not-yet.
    NotYet = 3,
}

/// One heroine's season clock: `total_days` days, `slots_per_day`
/// action slots per day.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignClock {
    total_days: i32,
    slots_per_day: i32,
    day: i32,
    slot: i32,
}

impl CampaignClock {
    /// Most slots a day may hold (the C# constructor's bound).
    pub const SLOTS_PER_DAY_MAX: i32 = 8;

    /// Build a season clock. The current day starts at 1 (1-based),
    /// with no slots spent.
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `total_days <= 0` or
    /// `slots_per_day` is outside 1..=8.
    pub fn new(total_days: i32, slots_per_day: i32) -> Result<CampaignClock, CoreError> {
        if total_days <= 0 {
            return Err(CoreError::OutOfRange {
                param: "total_days",
                message: "A season needs at least one day.",
            });
        }
        if slots_per_day <= 0 || slots_per_day > Self::SLOTS_PER_DAY_MAX {
            return Err(CoreError::OutOfRange {
                param: "slots_per_day",
                message: "Slots per day are bounded 1..8.",
            });
        }
        Ok(CampaignClock {
            total_days,
            slots_per_day,
            day: 1,
            slot: 0,
        })
    }

    /// Total days in the season.
    #[must_use]
    pub fn total_days(&self) -> i32 {
        self.total_days
    }

    /// Action slots in each day.
    #[must_use]
    pub fn slots_per_day(&self) -> i32 {
        self.slots_per_day
    }

    /// Current day, 1-based.
    #[must_use]
    pub fn day(&self) -> i32 {
        self.day
    }

    /// Slots already spent today, 0..slots_per_day.
    #[must_use]
    pub fn slot(&self) -> i32 {
        self.slot
    }

    /// True when every slot of every day is spent.
    #[must_use]
    pub fn season_over(&self) -> bool {
        self.day > self.total_days
    }

    /// Slots left today.
    #[must_use]
    pub fn slots_left_today(&self) -> i32 {
        if self.season_over() {
            0
        } else {
            self.slots_per_day - self.slot
        }
    }

    /// Spend one slot. Returns false when the season is over.
    pub fn spend_slot(&mut self) -> bool {
        if self.season_over() {
            return false;
        }
        self.slot += 1;
        if self.slot >= self.slots_per_day {
            self.slot = 0;
            self.day += 1;
        }
        true
    }

    /// Evaluate the season's ending from her stage.
    #[must_use]
    pub fn evaluate_ending(stage: W5Stage) -> SeasonEnding {
        match stage {
            W5Stage::SelfDriven => SeasonEnding::SelfDriven,
            W5Stage::Devoted | W5Stage::Craving => SeasonEnding::Deepened,
            W5Stage::Willing | W5Stage::Curious => SeasonEnding::Begun,
            W5Stage::Unaware => SeasonEnding::NotYet,
        }
    }
}
