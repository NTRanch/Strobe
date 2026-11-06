//! W5 Deepening stages (PATTERNS.md): how far a heroine has come in
//! surfacing her own hidden desire / dissolving her own block. The
//! stage is a state variable shared by progression, sessions, the
//! phone hub, and the expression/feed views (ADAPT-SPEC A1–A5).
//!
//! Discriminants are part of the contract and port verbatim from
//! the C# (SPEC-BEVY, header): Unaware = 0 … SelfDriven = 5.

use crate::error::CoreError;

/// One W5 Deepening stage.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum W5Stage {
    /// Stage 0: she has not yet surfaced her desire.
    Unaware = 0,
    /// Stage 1: first spark.
    Curious = 1,
    /// Stage 2: secret practice.
    Willing = 2,
    /// Stage 3: choosing.
    Craving = 3,
    /// Stage 4: no more hiding.
    Devoted = 4,
    /// Stage 5: self-driven.
    SelfDriven = 5,
}

impl W5Stage {
    /// The stage's contract discriminant (0..=5).
    #[must_use]
    pub const fn discriminant(self) -> i32 {
        self as i32
    }

    /// Rebuild a stage from a raw discriminant (save/data layer).
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `value` is not 0..=5.
    pub fn from_discriminant(value: i32) -> Result<W5Stage, CoreError> {
        match value {
            0 => Ok(W5Stage::Unaware),
            1 => Ok(W5Stage::Curious),
            2 => Ok(W5Stage::Willing),
            3 => Ok(W5Stage::Craving),
            4 => Ok(W5Stage::Devoted),
            5 => Ok(W5Stage::SelfDriven),
            _ => Err(CoreError::OutOfRange {
                param: "stage",
                message: "Stage discriminant is 0..=5.",
            }),
        }
    }
}
