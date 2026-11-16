//! Strobe dev/ops TUI (fleet TUI programme D-1562, pattern DESIGN §D2).
//!
//! A developer/operator console for the Strobe Rust workspace: build,
//! check, and boot-smoke the workspace crates, inspect bridge/core
//! state, and review the (empty-by-design) fleet trigger audit. This
//! is not a player UI and adds no gameplay features.
//!
//! Layout: [`facade`] is the narrow trait the UI programs against
//! (real subprocess facade + in-memory fake), [`app`] is the screen
//! state machine and event loop, [`panels`] holds pure widget
//! constructors, [`screens`] the member's screens, and `main.rs` is a
//! thin binary with a full-screen mode and a non-TTY line mode sharing
//! one execute path.

#![forbid(unsafe_code)]

pub mod app;
pub mod facade;
pub mod panels;
pub mod runner;
pub mod screens;

/// Maximum log lines retained in memory (lines).
pub const LOG_LINES_MAX: usize = 500;
/// Maximum bytes retained from one subprocess run (bytes).
pub const OUTPUT_BYTES_MAX: usize = 64 * 1024;
/// Default deadline for one subprocess run (seconds).
pub const RUN_DEADLINE_SECS: u64 = 900;
/// Maximum length of one rendered text line (characters).
pub const LINE_CHARS_MAX: usize = 240;
/// Maximum staged command arguments.
pub const ARGV_ITEMS_MAX: usize = 32;
/// Maximum fleet audit lines shown (lines).
pub const AUDIT_LINES_MAX: usize = 50;
