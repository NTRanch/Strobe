//! Typed contract failures for the core.
//!
//! The C# reference signals contract violations with exceptions.
//! This port invents no exception mechanism: fallible constructors
//! and methods return `Result<_, CoreError>`, with one variant per
//! C# throw-site class (SPEC-BEVY §3.0). Nothing in the core panics
//! on player-reachable input.

use core::fmt;

/// One core contract failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    /// A plain argument contract violation (C# `ArgumentException`):
    /// an empty heroine id, a session built from a profile and a
    /// state that name different heroines.
    Argument {
        /// The offending parameter's name.
        param: &'static str,
        /// What the contract requires, in words.
        message: &'static str,
    },
    /// A required argument was absent (C# `ArgumentNullException`).
    /// Reachable only where the C# took a nullable reference type;
    /// the port models exactly those parameters as `Option`.
    NullArgument {
        /// The missing parameter's name.
        param: &'static str,
    },
    /// A numeric argument outside its permitted range (C#
    /// `ArgumentOutOfRangeException`): tiers, dial positions, track
    /// weights, clock shapes, XP/coin awards, raw enum discriminants.
    OutOfRange {
        /// The offending parameter's name.
        param: &'static str,
        /// The permitted range, in words.
        message: &'static str,
    },
    /// A state-machine refusal (C# `InvalidOperationException`): an
    /// action on an ended session, an action while surfaced, a
    /// surface with no resume, an upgrade before its install.
    InvalidOperation {
        /// What the state machine refused, in words.
        message: &'static str,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Argument { param, message } => {
                write!(formatter, "argument `{param}`: {message}")
            }
            CoreError::NullArgument { param } => {
                write!(formatter, "argument `{param}` must not be null")
            }
            CoreError::OutOfRange { param, message } => {
                write!(formatter, "argument `{param}` out of range: {message}")
            }
            CoreError::InvalidOperation { message } => {
                write!(formatter, "invalid operation: {message}")
            }
        }
    }
}

impl std::error::Error for CoreError {}
