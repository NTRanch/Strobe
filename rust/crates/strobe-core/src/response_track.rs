//! The four per-heroine response tracks (ADAPT-SPEC A3). Track XP
//! is earned only through kink-aligned play (see
//! [`crate::session::Session`]); each track's level-3 capstone is a
//! named ability that acts as a content key and raises the session
//! tier ceiling story-side.
//!
//! Discriminants port verbatim from the C# — Gaze = 0, Rhythm = 1,
//! Touch = 2, Voice = 3, *not* the prose order (SPEC-BEVY, header).

use crate::error::CoreError;

/// One response track.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum ResponseTrack {
    /// Witnessing, being seen, eye-contact and expression play.
    Gaze = 0,
    /// Chart play with her: accuracy, combo, frequency play.
    Rhythm = 1,
    /// Contact and sensation play.
    Touch = 2,
    /// Induction and verbal play; her finishing the VOICE's phrases.
    Voice = 3,
}

impl ResponseTrack {
    /// All four tracks, in discriminant order.
    pub const ALL: [ResponseTrack; 4] = [
        ResponseTrack::Gaze,
        ResponseTrack::Rhythm,
        ResponseTrack::Touch,
        ResponseTrack::Voice,
    ];

    /// The track's contract discriminant (0..=3).
    #[must_use]
    pub const fn discriminant(self) -> i32 {
        self as i32
    }

    /// The track's slot in fixed-size per-track arrays.
    #[must_use]
    pub const fn slot(self) -> usize {
        self as usize
    }

    /// Rebuild a track from a raw discriminant (save/data layer).
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `value` is not 0..=3. This is
    /// where the C#'s "undefined track key" rejection lives in the
    /// port: raw discriminants enter only through here.
    pub fn from_discriminant(value: i32) -> Result<ResponseTrack, CoreError> {
        match value {
            0 => Ok(ResponseTrack::Gaze),
            1 => Ok(ResponseTrack::Rhythm),
            2 => Ok(ResponseTrack::Touch),
            3 => Ok(ResponseTrack::Voice),
            _ => Err(CoreError::OutOfRange {
                param: "track",
                message: "Track is not a defined response track.",
            }),
        }
    }
}

/// XP ladder for one track. Pure functions, no state.
pub struct TrackLadder;

// Cumulative XP at which levels 1, 2, 3 are reached.
const LEVEL_THRESHOLDS: [i32; 3] = [30, 70, 120];

impl TrackLadder {
    /// Highest track level (the capstone level).
    pub const MAX_LEVEL: i32 = 3;

    /// Level for a cumulative XP total (0..=MAX_LEVEL).
    #[must_use]
    pub fn level_for_xp(xp: i32) -> i32 {
        if xp <= 0 {
            return 0;
        }
        let mut level = 0;
        for threshold in LEVEL_THRESHOLDS {
            if xp >= threshold {
                level += 1;
            }
        }
        level
    }

    /// True exactly when this XP total is at the capstone rung.
    #[must_use]
    pub fn is_capstone(xp: i32) -> bool {
        Self::level_for_xp(xp) >= Self::MAX_LEVEL
    }
}
