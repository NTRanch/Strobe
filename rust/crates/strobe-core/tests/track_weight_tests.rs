//! Port of the C# `TrackWeightTests.cs` (20 cases) from
//! `Tests/Strobe.Core.Tests` in primo `~/strobe-trackb`.
//!
//! Per-heroine response-track weights (ADAPT-SPEC A3). The battery
//! is split evenly: ten validation tests prove the documented
//! scaling and default behavior, and ten adversarial tests prove
//! invalid data fails fast and weights cannot leak into another
//! track or into Corruption/tier math.
//!
//! Mapping note: the C# can construct an *undefined* enum key
//! (`(ResponseTrack)99`) and asserts the constructor rejects it
//! with `ArgumentException` and the lookup with
//! `ArgumentOutOfRangeException`. Rust enum values cannot be
//! undefined, so the port places that rejection at the raw-data
//! boundary instead: `ResponseTrack::from_discriminant` and
//! `HeroineProfile::try_track_weight`, both
//! `CoreError::OutOfRange`. The adversarial case below asserts
//! exactly those two rejections.

use strobe_core::{
    CoreError, HeroineProfile, HeroineState, KinkVectorKind, ResponseTrack, Session, SessionAction,
};

fn profile(weights: &[(ResponseTrack, f64)]) -> HeroineProfile {
    HeroineProfile::new(
        "test-heroine",
        KinkVectorKind::HiddenDesire,
        0.5,
        "desire",
        "block",
        weights,
    )
    .expect("test profile is valid")
}

fn profile_result(weights: &[(ResponseTrack, f64)]) -> Result<HeroineProfile, CoreError> {
    HeroineProfile::new(
        "test-heroine",
        KinkVectorKind::HiddenDesire,
        0.5,
        "desire",
        "block",
        weights,
    )
}

fn aligned(track: ResponseTrack, rushed: bool, initiated_by_her: bool) -> SessionAction {
    SessionAction::new(track, 0, true, rushed, initiated_by_her, 0.5).expect("test action is valid")
}

fn actions_to_capstone(profile: &HeroineProfile) -> i32 {
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(profile, &state).expect("session");
    for index in 1..=100 {
        let result = session
            .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
            .expect("applies");
        if result.capstone_unlocked() {
            return index;
        }
    }
    -1
}

// Validation -----------------------------------------------------

#[test]
fn no_weights_default_every_track_to_one() {
    let profile = profile(&[]);
    assert_eq!(4, profile.track_weights().len());
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Gaze));
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Rhythm));
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Touch));
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Voice));
}

#[test]
fn partial_weights_unspecified_tracks_default_to_one() {
    let profile = profile(&[(ResponseTrack::Voice, 2.0)]);
    assert_eq!(2.0, profile.track_weight(ResponseTrack::Voice));
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Gaze));
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Rhythm));
    assert_eq!(1.0, profile.track_weight(ResponseTrack::Touch));
}

#[test]
fn explicit_unit_weights_match_unweighted_session_exactly() {
    let plain_profile = profile(&[]);
    let mut plain_state = HeroineState::new("test-heroine").expect("state");
    let mut plain_session = Session::new(&plain_profile, &plain_state).expect("session");
    let unit_profile = profile(&[
        (ResponseTrack::Gaze, 1.0),
        (ResponseTrack::Rhythm, 1.0),
        (ResponseTrack::Touch, 1.0),
        (ResponseTrack::Voice, 1.0),
    ]);
    let mut unit_state = HeroineState::new("test-heroine").expect("state");
    let mut unit_session = Session::new(&unit_profile, &unit_state).expect("session");

    for _ in 0..15 {
        let plain = plain_session
            .apply_action(
                &mut plain_state,
                aligned(ResponseTrack::Voice, false, false),
            )
            .expect("applies");
        let unit = unit_session
            .apply_action(&mut unit_state, aligned(ResponseTrack::Voice, false, false))
            .expect("applies");
        assert_eq!(plain.response_delta(), unit.response_delta());
        assert_eq!(plain.corruption_delta(), unit.corruption_delta());
        assert_eq!(plain.tier_unlocked(), unit.tier_unlocked());
        assert_eq!(
            plain_state.track_xp(ResponseTrack::Voice),
            unit_state.track_xp(ResponseTrack::Voice)
        );
    }
}

#[test]
fn heavy_track_first_award_scales_by_weight() {
    let profile = profile(&[(ResponseTrack::Voice, 2.0)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    // `award` reads state through the session's borrow; split the
    // reads around the call instead.
    let before = state.track_xp(ResponseTrack::Voice);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
        .expect("applies");
    assert_eq!(12, state.track_xp(ResponseTrack::Voice) - before);
}

#[test]
fn light_track_first_award_scales_by_weight() {
    let profile = profile(&[(ResponseTrack::Voice, 0.5)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let before = state.track_xp(ResponseTrack::Voice);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
        .expect("applies");
    assert_eq!(3, state.track_xp(ResponseTrack::Voice) - before);
}

#[test]
fn minimum_weight_first_award_uses_documented_integer_rounding() {
    let profile = profile(&[(ResponseTrack::Voice, HeroineProfile::MIN_TRACK_WEIGHT)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let before = state.track_xp(ResponseTrack::Voice);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
        .expect("applies");
    // 6 * 0.25 = 1.5, rounded away from zero because XP is integral.
    assert_eq!(2, state.track_xp(ResponseTrack::Voice) - before);
}

#[test]
fn maximum_weight_first_award_scales_by_weight() {
    let profile = profile(&[(ResponseTrack::Voice, HeroineProfile::MAX_TRACK_WEIGHT)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let before = state.track_xp(ResponseTrack::Voice);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
        .expect("applies");
    assert_eq!(18, state.track_xp(ResponseTrack::Voice) - before);
}

#[test]
fn weighted_awards_diminish_from_actual_weighted_progress() {
    let profile = profile(&[(ResponseTrack::Voice, 2.0)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let mut deltas = [0_i32; 3];
    for delta in &mut deltas {
        let before = state.track_xp(ResponseTrack::Voice);
        session
            .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
            .expect("applies");
        *delta = state.track_xp(ResponseTrack::Voice) - before;
    }
    assert_eq!([12, 12, 10], deltas);
}

#[test]
fn heavy_track_reaches_capstone_sooner_than_unweighted_track() {
    let heavy = actions_to_capstone(&profile(&[(ResponseTrack::Voice, 3.0)]));
    let plain = actions_to_capstone(&profile(&[]));
    assert_eq!(22, heavy);
    assert_eq!(66, plain);
    assert!(heavy < plain);
}

#[test]
fn surface_beat_response_factors_are_not_scaled_by_weights() {
    let profile = profile(&[(ResponseTrack::Voice, 3.0)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    for _ in 0..3 {
        session
            .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
            .expect("applies");
    }
    assert_eq!(45, state.track_xp(ResponseTrack::Voice));

    let surface = session
        .surface(
            strobe_core::SurfaceTrigger::NameSpoken,
            Some(strobe_core::SurfaceReaction::new(
                strobe_core::ReactionGrammar::FaceCover,
                "line.surface",
                "expr.surface",
            )),
        )
        .expect("surfaces");
    assert!((surface.response_delta() - (-19.8)).abs() <= 0.0005);
    assert!((session.response() - 16.2).abs() <= 0.0005);

    let before_resume = state.track_xp(ResponseTrack::Voice);
    let resume = session
        .resume_trance(&mut state, aligned(ResponseTrack::Voice, false, false))
        .expect("resumes");
    assert!((resume.response_delta() - 18.0).abs() <= 0.0005);
    assert!((resume.corruption_delta() - 8.0).abs() <= 0.0005);
    assert_eq!(9, state.track_xp(ResponseTrack::Voice) - before_resume);
}

// Adversarial ----------------------------------------------------

#[test]
fn zero_weight_is_rejected() {
    assert!(matches!(
        profile_result(&[(ResponseTrack::Voice, 0.0)]),
        Err(CoreError::OutOfRange {
            param: "weight",
            ..
        })
    ));
}

#[test]
fn negative_weight_is_rejected() {
    assert!(matches!(
        profile_result(&[(ResponseTrack::Voice, -1.0)]),
        Err(CoreError::OutOfRange {
            param: "weight",
            ..
        })
    ));
}

#[test]
fn nan_weight_is_rejected() {
    assert!(matches!(
        profile_result(&[(ResponseTrack::Voice, f64::NAN)]),
        Err(CoreError::OutOfRange {
            param: "weight",
            ..
        })
    ));
}

#[test]
fn infinite_weight_is_rejected() {
    assert!(matches!(
        profile_result(&[(ResponseTrack::Voice, f64::INFINITY)]),
        Err(CoreError::OutOfRange {
            param: "weight",
            ..
        })
    ));
}

#[test]
fn below_minimum_weight_is_rejected() {
    assert!(matches!(
        profile_result(&[(
            ResponseTrack::Voice,
            HeroineProfile::MIN_TRACK_WEIGHT - 0.01
        )]),
        Err(CoreError::OutOfRange {
            param: "weight",
            ..
        })
    ));
}

#[test]
fn above_maximum_weight_is_rejected() {
    assert!(matches!(
        profile_result(&[(
            ResponseTrack::Voice,
            HeroineProfile::MAX_TRACK_WEIGHT + 0.01
        )]),
        Err(CoreError::OutOfRange {
            param: "weight",
            ..
        })
    ));
}

#[test]
fn undefined_track_weight_key_and_lookup_are_rejected() {
    // The C# builds a profile from a map keyed by (ResponseTrack)99
    // and asserts rejection, then asserts the same of a lookup. In
    // the port, raw discriminants enter only through
    // from_discriminant / try_track_weight; both refuse 99.
    assert!(matches!(
        ResponseTrack::from_discriminant(99),
        Err(CoreError::OutOfRange { param: "track", .. })
    ));
    assert!(matches!(
        profile(&[]).try_track_weight(99),
        Err(CoreError::OutOfRange { param: "track", .. })
    ));
}

#[test]
fn constructor_copies_weights_later_input_mutation_has_no_effect() {
    // C# passes a Dictionary and mutates it after construction.
    // The port's constructor copies the pairs into its own array;
    // the same guarantee is asserted over the input slice's source.
    let mut weights = vec![(ResponseTrack::Voice, 2.0)];
    let profile = profile(&weights);
    weights[0].1 = 3.0;
    assert_eq!(2.0, profile.track_weight(ResponseTrack::Voice));
    assert_eq!(2.0, profile.track_weights()[ResponseTrack::Voice.slot()]);
}

#[test]
fn weights_do_not_leak_across_tracks() {
    let profile = profile(&[(ResponseTrack::Voice, 3.0), (ResponseTrack::Rhythm, 0.25)]);
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");

    let before = state.track_xp(ResponseTrack::Voice);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Voice, false, false))
        .expect("applies");
    assert_eq!(18, state.track_xp(ResponseTrack::Voice) - before);
    assert_eq!(0, state.track_xp(ResponseTrack::Gaze));
    assert_eq!(0, state.track_xp(ResponseTrack::Rhythm));
    assert_eq!(0, state.track_xp(ResponseTrack::Touch));

    let before = state.track_xp(ResponseTrack::Gaze);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Gaze, false, false))
        .expect("applies");
    assert_eq!(6, state.track_xp(ResponseTrack::Gaze) - before);
    assert_eq!(18, state.track_xp(ResponseTrack::Voice));
    assert_eq!(0, state.track_xp(ResponseTrack::Touch));

    let before = state.track_xp(ResponseTrack::Rhythm);
    session
        .apply_action(&mut state, aligned(ResponseTrack::Rhythm, false, false))
        .expect("applies");
    assert_eq!(2, state.track_xp(ResponseTrack::Rhythm) - before);
    assert_eq!(18, state.track_xp(ResponseTrack::Voice));
    assert_eq!(6, state.track_xp(ResponseTrack::Gaze));
    assert_eq!(0, state.track_xp(ResponseTrack::Touch));
}

#[test]
fn weights_do_not_change_corruption_or_tier_math() {
    let actions = [
        aligned(ResponseTrack::Voice, false, false),
        aligned(ResponseTrack::Gaze, true, false),
        SessionAction::new(ResponseTrack::Touch, 0, false, false, false, 0.5).expect("action"),
        aligned(ResponseTrack::Rhythm, false, false),
        SessionAction::new(ResponseTrack::Voice, 0, false, true, false, 0.5).expect("action"),
        aligned(ResponseTrack::Voice, false, true),
        aligned(ResponseTrack::Gaze, false, false),
        aligned(ResponseTrack::Rhythm, false, false),
        aligned(ResponseTrack::Voice, false, false),
        aligned(ResponseTrack::Voice, false, false),
    ];
    let plain_profile = profile(&[]);
    let mut plain_state = HeroineState::new("test-heroine").expect("state");
    let mut plain_session = Session::new(&plain_profile, &plain_state).expect("session");
    let weighted_profile = profile(&[(ResponseTrack::Voice, 3.0), (ResponseTrack::Gaze, 0.25)]);
    let mut weighted_state = HeroineState::new("test-heroine").expect("state");
    let mut weighted_session = Session::new(&weighted_profile, &weighted_state).expect("session");

    for action in actions {
        let plain = plain_session
            .apply_action(&mut plain_state, action)
            .expect("applies");
        let weighted = weighted_session
            .apply_action(&mut weighted_state, action)
            .expect("applies");
        assert_eq!(plain.response_delta(), weighted.response_delta());
        assert_eq!(plain.corruption_delta(), weighted.corruption_delta());
        assert_eq!(plain.tier_unlocked(), weighted.tier_unlocked());
        assert_eq!(plain_session.corruption(), weighted_session.corruption());
        assert_eq!(
            plain_session.current_tier(),
            weighted_session.current_tier()
        );
    }

    assert_ne!(
        plain_state.track_xp(ResponseTrack::Voice),
        weighted_state.track_xp(ResponseTrack::Voice)
    );
}
