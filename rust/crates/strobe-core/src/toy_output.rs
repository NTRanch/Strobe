//! The toy-output seam (W4/D6 + ADAPT-SPEC A2). The transport (the
//! Lovense bridge, `strobe-bridge`) sits behind this trait; Core
//! owns only the intensity law:
//!
//! delivered = curve × tier ceiling × player ceiling,
//! always 0 while the master switch is off.
//!
//! The tier ceiling is A2's consent rule in arithmetic: hardware
//! intensity can never exceed the deepest tier her corruption has
//! unlocked this session. The player ceiling and the master switch
//! are the standing bridge safety contract (master OFF by default,
//! ceiling clamped in code, stop synchronous, no resume on
//! reconnect) and are enforced by the bridge, not here — here they
//! arrive as already-clamped inputs.

use crate::session::Session;

/// A destination for toy intensity samples.
pub trait ToyOutput {
    /// Deliver one intensity sample, 0..=`ToyCurve::MAX_INTENSITY`.
    fn send(&mut self, intensity: i32);

    /// Stop all output now. Synchronous by contract.
    fn stop_all(&mut self);
}

/// Pure intensity computation for the D6/A2 curve.
pub struct ToyCurve;

impl ToyCurve {
    /// Bridge intensity scale (Lovense 0..20).
    pub const MAX_INTENSITY: i32 = 20;

    /// Compute the intensity to deliver.
    ///
    /// - `master_on`: the bridge master switch.
    /// - `curve_value`: D6 curve output before ceilings, 0..=1
    ///   (music layer × judgement layer, computed by the caller).
    /// - `unlocked_tier`: deepest tier her corruption has unlocked
    ///   this session, 0..=`Session::MAX_TIER`.
    /// - `player_ceiling`: player-set ceiling, 0..=MAX_INTENSITY.
    ///
    /// Hostile inputs are clamped, never trusted — the same
    /// arithmetic as the C#, including its round-half-to-even at
    /// the final scaling step.
    #[must_use]
    pub fn compute_intensity(
        master_on: bool,
        curve_value: f64,
        unlocked_tier: i32,
        player_ceiling: i32,
    ) -> i32 {
        if !master_on {
            return 0;
        }
        let curve = curve_value.clamp(0.0, 1.0);
        let tier = unlocked_tier.clamp(0, Session::MAX_TIER);
        let ceiling = player_ceiling.clamp(0, Self::MAX_INTENSITY);

        // Her tier ceiling: tier t of MAX_TIER permits t/MAX_TIER of
        // the scale. Tier 0 is touch-free: rhythm presence only
        // comes with the first willing tier.
        let tier_ceiling = Self::MAX_INTENSITY * tier / Session::MAX_TIER;
        let raw = (curve * f64::from(tier_ceiling)).round_ties_even() as i32;
        raw.min(ceiling)
    }
}

/// A toy output that does nothing. The default everywhere: a build
/// without a connected bridge is silent by construction.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullToyOutput;

impl ToyOutput for NullToyOutput {
    fn send(&mut self, _intensity: i32) {}

    fn stop_all(&mut self) {}
}
