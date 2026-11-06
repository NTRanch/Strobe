//! strobe-game — Strobe, the flagship: a fixed-FM-synth trance
//! surface game (SPEC / SPEC-BEVY). Bevy shell over `strobe-core`,
//! toy output through `strobe-bridge`.
//!
//! The shell is headless-first this wave: the app assembles with
//! minimal plugins, the simulation runs entirely in `FixedUpdate`
//! at dt = 1/120 s, and the scripted-input harness
//! ([`sim::run_ticks`]) drives the same systems the windowed
//! build will. The render entry point is the `strobe-smoke` bin
//! behind the `render` feature.

#![forbid(unsafe_code)]

pub mod analytics;
pub mod chart;
pub mod data;
pub mod save;
pub mod sim;
pub mod tutorial;
