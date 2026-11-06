//! Port of the C# `ProgressionTests.cs` (18 cases; 17 runtime
//! tests here) from `Tests/Strobe.Core.Tests` in primo
//! `~/strobe-trackb`.
//!
//! W3/W5 ladder: boundaries at every rung (validation), plus the
//! adversarial edges — negative and extreme counts must fail safe
//! to the ladder, never throw, never skip a stage.
//!
//! The C# case `RecordClear_NullStateThrows` is the one case of
//! the 82 discharged by construction (SPEC-BEVY §8.1(b)):
//! `Progression::record_clear` takes `&mut HeroineState`, so there
//! is no null to pass and a clear can never be recorded against a
//! missing state.

use strobe_core::{HeroineState, Progression, W5Stage};

#[test]
fn stage_for_count_follows_the_ladder_0() {
    assert_eq!(W5Stage::Unaware, Progression::stage_for_count(0));
}

#[test]
fn stage_for_count_follows_the_ladder_1() {
    assert_eq!(W5Stage::Curious, Progression::stage_for_count(1));
}

#[test]
fn stage_for_count_follows_the_ladder_2() {
    assert_eq!(W5Stage::Curious, Progression::stage_for_count(2));
}

#[test]
fn stage_for_count_follows_the_ladder_3() {
    assert_eq!(W5Stage::Willing, Progression::stage_for_count(3));
}

#[test]
fn stage_for_count_follows_the_ladder_5() {
    assert_eq!(W5Stage::Willing, Progression::stage_for_count(5));
}

#[test]
fn stage_for_count_follows_the_ladder_6() {
    assert_eq!(W5Stage::Craving, Progression::stage_for_count(6));
}

#[test]
fn stage_for_count_follows_the_ladder_9() {
    assert_eq!(W5Stage::Craving, Progression::stage_for_count(9));
}

#[test]
fn stage_for_count_follows_the_ladder_10() {
    assert_eq!(W5Stage::Devoted, Progression::stage_for_count(10));
}

#[test]
fn stage_for_count_follows_the_ladder_14() {
    assert_eq!(W5Stage::Devoted, Progression::stage_for_count(14));
}

#[test]
fn stage_for_count_follows_the_ladder_15() {
    assert_eq!(W5Stage::SelfDriven, Progression::stage_for_count(15));
}

#[test]
fn stage_for_count_follows_the_ladder_100() {
    assert_eq!(W5Stage::SelfDriven, Progression::stage_for_count(100));
}

#[test]
fn stage_for_count_negative_counts_fail_safe_to_unaware() {
    assert_eq!(W5Stage::Unaware, Progression::stage_for_count(-1));
    assert_eq!(W5Stage::Unaware, Progression::stage_for_count(i32::MIN));
}

#[test]
fn stage_for_count_max_int_stays_self_driven() {
    assert_eq!(W5Stage::SelfDriven, Progression::stage_for_count(i32::MAX));
}

#[test]
fn clears_to_next_stage_counts_down_and_stops_at_top() {
    assert_eq!(1, Progression::clears_to_next_stage(0));
    assert_eq!(2, Progression::clears_to_next_stage(1));
    assert_eq!(1, Progression::clears_to_next_stage(14));
    assert_eq!(0, Progression::clears_to_next_stage(15));
    assert_eq!(0, Progression::clears_to_next_stage(999));
}

#[test]
fn record_clear_advances_count_and_stage_monotonically() {
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut previous = W5Stage::Unaware;
    for _ in 0..20 {
        let stage = Progression::record_clear(&mut state);
        assert!(stage >= previous, "Stage must never decrease.");
        previous = stage;
    }
    assert_eq!(20, state.clear_count());
    assert_eq!(W5Stage::SelfDriven, state.stage());
}

#[test]
fn heroine_state_restore_clear_count_clamps_negative_to_zero() {
    let mut state = HeroineState::new("test-heroine").expect("state");
    state.restore_clear_count(-7);
    assert_eq!(0, state.clear_count());
    assert_eq!(W5Stage::Unaware, state.stage());
}

#[test]
fn heroine_state_stage_is_derived_never_stored_separately() {
    let mut state = HeroineState::new("test-heroine").expect("state");
    state.restore_clear_count(6);
    assert_eq!(W5Stage::Craving, state.stage());
    assert_eq!(36.0, state.corruption_baseline());
    assert_eq!(3, state.stage_tier_ceiling());
}
