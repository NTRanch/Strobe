//! Port of the C# `SessionTests.cs` (14 cases) from
//! `Tests/Strobe.Core.Tests` in primo `~/strobe-trackb`.
//!
//! A2 session law (validation): corruption gates tiers, aligned
//! play raises corruption, frequency is learnable, completion
//! records the clear. Adversarial half: response alone must never
//! unlock a tier, off-kink/rushed play must never raise
//! corruption, pushing past her corruption must end the session
//! gracefully with progress kept, and a dead session refuses work.
//!
//! C# `Throws` assertions map to `CoreError` variant assertions
//! (SPEC-BEVY §8.1(a)); float assertions keep the C# comparison.

use strobe_core::{
    CoreError, HeroineProfile, HeroineState, KinkVectorKind, ResponseTrack, Session, SessionAction,
    SessionEndReason, W5Stage,
};

fn profile(center: f64) -> HeroineProfile {
    HeroineProfile::new(
        "test-heroine",
        KinkVectorKind::HiddenDesire,
        center,
        "desire",
        "block",
        &[],
    )
    .expect("test profile is valid")
}

fn aligned(tier: i32, dial: f64, rushed: bool, by_her: bool) -> SessionAction {
    SessionAction::new(ResponseTrack::Voice, tier, true, rushed, by_her, dial)
        .expect("test action is valid")
}

fn off_kink(rushed: bool) -> SessionAction {
    SessionAction::new(ResponseTrack::Voice, 0, false, rushed, false, 0.5)
        .expect("test action is valid")
}

#[test]
fn aligned_play_raises_response_and_corruption() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let result = session
        .apply_action(&mut state, aligned(0, 0.5, false, false))
        .expect("applies");
    assert!(result.response_delta() > 0.0);
    assert!(result.corruption_delta() > 0.0);
    assert_eq!(SessionEndReason::None, session.end_reason());
}

#[test]
fn corruption_crossing_threshold_unlocks_tier() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let mut unlocked = -1;
    for _ in 0..4 {
        if unlocked >= 0 {
            break;
        }
        unlocked = session
            .apply_action(&mut state, aligned(0, 0.5, false, false))
            .expect("applies")
            .tier_unlocked();
    }
    assert_eq!(1, unlocked);
    assert_eq!(1, session.current_tier());
}

#[test]
fn response_alone_never_unlocks_a_tier() {
    // Adversarial: maximum response, zero corruption growth —
    // off-kink actions still raise response at 0.6x. No tier may
    // open, because corruption is the only key.
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    for _ in 0..12 {
        let result = session
            .apply_action(&mut state, off_kink(false))
            .expect("applies");
        assert_eq!(-1, result.tier_unlocked());
    }
    assert!(session.response() > 50.0);
    assert_eq!(0, session.current_tier());
    assert!(session.corruption() <= 0.0);
}

#[test]
fn off_kink_and_rushed_play_lowers_corruption() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    session
        .apply_action(&mut state, aligned(0, 0.5, false, false))
        .expect("applies");
    let before = session.corruption();
    let result = session
        .apply_action(&mut state, off_kink(true))
        .expect("applies");
    assert!(result.corruption_delta() < 0.0);
    assert!(session.corruption() < before);
    assert!(session.composure_strain() > 0.0);
}

#[test]
fn pushing_past_corruption_ends_gracefully_progress_kept() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    session
        .apply_action(&mut state, aligned(0, 0.5, false, false))
        .expect("applies");
    let xp_before = state.track_xp(ResponseTrack::Voice);
    assert!(xp_before > 0);

    let result = session
        .apply_action(&mut state, aligned(Session::MAX_TIER, 0.5, false, false))
        .expect("the overreach itself is answered, not an error");
    assert!(result.ended_by_comfort_limit());
    assert_eq!(SessionEndReason::ComfortLimit, session.end_reason());
    assert!(session.aftercare_required());
    assert_eq!(xp_before, state.track_xp(ResponseTrack::Voice));
    assert_eq!(0, state.clear_count()); // no clear recorded
}

#[test]
fn ended_session_refuses_further_actions() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    session
        .apply_action(&mut state, aligned(Session::MAX_TIER, 0.5, false, false))
        .expect("ends by comfort limit");
    assert!(matches!(
        session.apply_action(&mut state, aligned(0, 0.5, false, false)),
        Err(CoreError::InvalidOperation { .. })
    ));
    assert!(matches!(
        session.complete(&mut state),
        Err(CoreError::InvalidOperation { .. })
    ));
}

#[test]
fn frequency_near_center_aligned_action_discovers_it() {
    let profile = profile(0.42);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let action =
        SessionAction::new(ResponseTrack::Gaze, 0, true, false, false, 0.44).expect("action");
    let result = session.apply_action(&mut state, action).expect("applies");
    assert!(result.frequency_discovered());
    assert!(state.frequency_learned());
}

#[test]
fn frequency_off_center_or_off_kink_never_discovers() {
    let profile = profile(0.42);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let far = SessionAction::new(ResponseTrack::Gaze, 0, true, false, false, 0.9).expect("action");
    let far_result = session.apply_action(&mut state, far).expect("applies");
    assert!(!far_result.frequency_discovered());
    let off =
        SessionAction::new(ResponseTrack::Gaze, 0, false, false, false, 0.42).expect("action");
    let off_result = session.apply_action(&mut state, off).expect("applies");
    assert!(!off_result.frequency_discovered());
    assert!(!state.frequency_learned());
}

#[test]
fn track_xp_diminishes_within_a_session_and_caps_at_one_minimum() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let mut awards = [0_i32; 8];
    for award in &mut awards {
        let before = state.track_xp(ResponseTrack::Voice);
        session
            .apply_action(&mut state, aligned(0, 0.5, false, false))
            .expect("applies");
        *award = state.track_xp(ResponseTrack::Voice) - before;
    }
    assert_eq!(6, awards[0]);
    assert!(awards[7] < awards[0]);
    assert!(awards[7] >= 1);
}

#[test]
fn capstone_unlocks_through_aligned_play_only() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let mut capstone = false;
    for _ in 0..90 {
        if capstone {
            break;
        }
        capstone = session
            .apply_action(&mut state, aligned(0, 0.5, false, false))
            .expect("applies")
            .capstone_unlocked();
    }
    assert!(capstone);
    assert!(state.has_capstone(ResponseTrack::Voice));
}

#[test]
fn complete_records_clear_and_pays_coins() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    for _ in 0..4 {
        session
            .apply_action(&mut state, aligned(0, 0.5, false, false))
            .expect("applies");
    }
    let summary = session.complete(&mut state).expect("completes");
    assert_eq!(1, state.clear_count());
    assert_eq!(W5Stage::Curious, summary.stage_after());
    assert!(summary.coins_awarded() > 0);
    assert_eq!(summary.coins_awarded(), state.outfit_coins());
    assert_eq!(session.current_tier(), summary.tier_reached());
}

#[test]
fn heroine_requests_next_tier_only_at_craving_with_corruption() {
    let profile = profile(0.5);
    let mut state = HeroineState::new("test-heroine").expect("state");
    state.restore_clear_count(6); // Craving; baseline corruption 36
    let session = Session::new(&profile, &state).expect("session");
    assert!(session.heroine_requests_next_tier(&state));

    let early_state = HeroineState::new("test-heroine").expect("state");
    let early = Session::new(&profile, &early_state).expect("session");
    assert!(!early.heroine_requests_next_tier(&early_state));
}

#[test]
fn session_mismatched_profile_and_state_throws() {
    let profile = HeroineProfile::new(
        "someone-else",
        KinkVectorKind::DissolvedBlock,
        0.5,
        "d",
        "b",
        &[],
    )
    .expect("profile");
    let state = HeroineState::new("test-heroine").expect("state");
    assert!(matches!(
        Session::new(&profile, &state),
        Err(CoreError::Argument { param: "state", .. })
    ));
}

#[test]
fn session_action_out_of_range_arguments_throw() {
    assert!(matches!(
        SessionAction::new(
            ResponseTrack::Voice,
            Session::MAX_TIER + 1,
            true,
            false,
            false,
            0.5
        ),
        Err(CoreError::OutOfRange { param: "tier", .. })
    ));
    assert!(matches!(
        SessionAction::new(ResponseTrack::Voice, 0, true, false, false, 1.5),
        Err(CoreError::OutOfRange {
            param: "dial_position",
            ..
        })
    ));
}
