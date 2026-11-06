//! The D6 clamp gate (SPEC-BEVY §8.2 / §4 criterion 7): across
//! the full chart at maximum combo, delivered intensity never
//! exceeds min(curve, tier ceiling, player ceiling) — and a miss
//! is a short, low thud, never absent output.

mod common;

use strobe_game::chart::THUD_CURVE;
use strobe_game::sim::run_ticks;
use strobe_game::sim::{SimInput, ToyEvent};

use common::{act, core, input, rig_with, settings};

fn chart_rig() -> common::Rig {
    let mut settings = settings();
    settings.record_readout_history = true;
    rig_with(settings)
}

/// Play a session to tier 1, then run the whole chart with the
/// given note inputs pre-registered.
fn run_chart(rig: &mut common::Rig, note_beats: &[u32]) {
    input(&mut rig.app, SimInput::RequestSession);
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "listen-with-her");
    assert_eq!(Some(1), core(&rig.app).session_tier());
    for beat in note_beats {
        input(
            &mut rig.app,
            SimInput::NoteInput {
                beat: *beat,
                offset_steps: 0,
            },
        );
    }
    // 32 beats × 60 steps, plus slack for the final pattern flush.
    run_ticks(&mut rig.app, 2100);
}

#[test]
fn full_chart_at_max_combo_never_exceeds_the_tier_ceiling() {
    let mut rig = chart_rig();
    run_chart(&mut rig, &[2, 6, 10, 14, 18, 22, 26, 30]);
    let core = core(&rig.app);
    assert!(!core.readout_history.is_empty());
    // Tier 1 ceiling: 20 × 1/4 = 5. Every delivered step respects
    // it, and the peak actually reaches it (the clamp engages —
    // the envelope peaks at 1.0 against a raw scale of 20).
    assert!(
        core.readout_history
            .iter()
            .all(|(_, delivered)| *delivered <= 5),
        "history max: {:?}",
        core.readout_history.iter().map(|(_, d)| d).max()
    );
    assert!(
        core.readout_history.iter().any(|(_, d)| *d == 5),
        "the clamp engages at the envelope peak"
    );
    // The wire agrees: no pattern strength exceeds the ceiling.
    let log = rig.toy_log.lock().expect("toy log");
    assert!(log.iter().any(|e| matches!(e, ToyEvent::Pattern(..))));
    for event in log.iter() {
        if let ToyEvent::Pattern(strengths, step_ms) = event {
            assert!(strengths.iter().all(|s| *s <= 5), "pattern: {strengths:?}");
            assert_eq!(125, *step_ms, "pattern step is the authored 125 ms");
            assert_eq!(8, strengths.len());
        }
    }
}

#[test]
fn a_miss_is_a_short_low_thud_never_silence() {
    let mut rig = chart_rig();
    // Beat 22 gets no input: it resolves as a miss.
    run_chart(&mut rig, &[2, 6, 10, 14, 18, 26, 30]);
    let core = core(&rig.app);
    let thud_steps: Vec<i32> = core
        .readout_history
        .iter()
        .filter(|(curve, _)| (*curve - THUD_CURVE).abs() < 1e-12)
        .map(|(_, delivered)| *delivered)
        .collect();
    assert!(!thud_steps.is_empty(), "the miss produced a thud burst");
    assert!(
        thud_steps.iter().all(|d| *d > 0 && *d <= 2),
        "the thud is low but present: {thud_steps:?}"
    );
    assert!(
        thud_steps.len() <= 24,
        "the thud is short (<= 2 bursts of 12 steps): {}",
        thud_steps.len()
    );
}
