//! Port of the C# `SurfaceBeatTests.cs` (14 cases) from
//! `Tests/Strobe.Core.Tests` in primo `~/strobe-trackb`.
//!
//! Surface Beat law (SCENARIO.md). Validation: surfacing shallows
//! response without ending anything, the authored reaction comes
//! back untouched, re-entry lands faster, and her own initiated
//! resume earns the self-recognition corruption bonus.
//! Adversarial half: surfacing must never end a session, never
//! require aftercare, never move corruption by itself; the state
//! machine refuses out-of-order calls; the per-session cap holds;
//! and an overreaching resume still meets the comfort limit.
//!
//! Mapping notes: the C# asserts reaction *identity* (`Assert.Same`);
//! the port moves the reaction value, so identity is asserted on
//! its content. The C# null-keys case maps to the port's designed
//! state: empty keys are stored verbatim and stay empty.

use strobe_core::{
    CoreError, HeroineProfile, HeroineState, KinkVectorKind, ReactionGrammar, ResponseTrack,
    Session, SessionAction, SessionEndReason, SurfaceReaction, SurfaceTrigger,
};

/// The C# tests compare these floats at 3 decimal places.
fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 0.0005,
        "expected {expected}, got {actual}"
    );
}

fn profile() -> HeroineProfile {
    HeroineProfile::new(
        "test-heroine",
        KinkVectorKind::HiddenDesire,
        0.5,
        "desire",
        "block",
        &[],
    )
    .expect("test profile is valid")
}

fn aligned(by_her: bool, tier: i32) -> SessionAction {
    SessionAction::new(ResponseTrack::Voice, tier, true, false, by_her, 0.5)
        .expect("test action is valid")
}

fn reaction() -> SurfaceReaction {
    SurfaceReaction::new(ReactionGrammar::FaceCover, "line.surface", "expr.surface")
}

fn deep_session<'a>(profile: &'a HeroineProfile, state: &mut HeroineState) -> Session<'a> {
    let mut session = Session::new(profile, state).expect("session");
    for _ in 0..3 {
        session
            .apply_action(state, aligned(false, 0))
            .expect("applies");
    }
    session
}

#[test]
fn surface_shallows_response_and_marks_surfaced() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    let corruption_before = session.corruption();
    let result = session
        .surface(SurfaceTrigger::NameSpoken, Some(reaction()))
        .expect("surfaces");
    assert!(session.is_surfaced());
    assert_eq!(1, session.surfaces_this_session());
    assert_close(result.response_delta(), -19.8);
    assert_close(session.response(), 16.2);
    assert_eq!(corruption_before, session.corruption());
    assert_eq!(SessionEndReason::None, session.end_reason());
}

#[test]
fn surface_returns_the_authored_reaction() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    let result = session
        .surface(SurfaceTrigger::Kiss, Some(reaction()))
        .expect("surfaces");
    assert_eq!(ReactionGrammar::FaceCover, result.reaction().grammar());
    assert_eq!("line.surface", result.reaction().voice_line_key());
    assert_eq!(1, result.surfaces_this_session());
}

#[test]
fn resume_lands_at_re_entry_factor() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    session
        .surface(SurfaceTrigger::CommandWord, Some(reaction()))
        .expect("surfaces");
    let result = session
        .resume_trance(&mut state, aligned(false, 0))
        .expect("resumes");
    assert!(!session.is_surfaced());
    assert_close(result.response_delta(), 18.0);
}

#[test]
fn resume_by_her_aligned_earns_self_recognition_bonus() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    session
        .surface(SurfaceTrigger::EyeContact, Some(reaction()))
        .expect("surfaces");
    let by_her = session
        .resume_trance(&mut state, aligned(true, 0))
        .expect("resumes");
    assert_close(by_her.corruption_delta(), 12.0);

    let mut other_state = HeroineState::new("test-heroine").expect("state");
    let mut other = deep_session(&profile, &mut other_state);
    other
        .surface(SurfaceTrigger::EyeContact, Some(reaction()))
        .expect("surfaces");
    let by_him = other
        .resume_trance(&mut other_state, aligned(false, 0))
        .expect("resumes");
    assert_close(by_him.corruption_delta(), 8.0);
}

#[test]
fn summary_counts_surfaces() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    session
        .surface(SurfaceTrigger::NameSpoken, Some(reaction()))
        .expect("surfaces");
    session
        .resume_trance(&mut state, aligned(false, 0))
        .expect("resumes");
    session
        .surface(SurfaceTrigger::CountToZero, Some(reaction()))
        .expect("surfaces");
    let summary = session.complete(&mut state).expect("completes");
    assert_eq!(2, summary.surfaces());
}

#[test]
fn surface_at_zero_response_still_plays_and_never_ends() {
    let profile = profile();
    let state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    let result = session
        .surface(SurfaceTrigger::ToyStop, Some(reaction()))
        .expect("surfaces");
    assert_close(result.response_delta(), 0.0);
    assert!(session.is_surfaced());
    assert_eq!(SessionEndReason::None, session.end_reason());
    assert!(!session.aftercare_required());
}

#[test]
fn surface_twice_without_resume_throws() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    session
        .surface(SurfaceTrigger::NameSpoken, Some(reaction()))
        .expect("surfaces");
    assert!(matches!(
        session.surface(SurfaceTrigger::Kiss, Some(reaction())),
        Err(CoreError::InvalidOperation { .. })
    ));
}

#[test]
fn resume_without_surface_throws() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    assert!(matches!(
        session.resume_trance(&mut state, aligned(false, 0)),
        Err(CoreError::InvalidOperation { .. })
    ));
}

#[test]
fn apply_action_while_surfaced_throws() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    session
        .surface(SurfaceTrigger::NameSpoken, Some(reaction()))
        .expect("surfaces");
    assert!(matches!(
        session.apply_action(&mut state, aligned(false, 0)),
        Err(CoreError::InvalidOperation { .. })
    ));
    assert!(session.is_surfaced());
}

#[test]
fn surface_and_resume_after_complete_throw() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    session.complete(&mut state).expect("completes");
    assert!(matches!(
        session.surface(SurfaceTrigger::Kiss, Some(reaction())),
        Err(CoreError::InvalidOperation { .. })
    ));
    assert!(matches!(
        session.resume_trance(&mut state, aligned(false, 0)),
        Err(CoreError::InvalidOperation { .. })
    ));
}

#[test]
fn surface_null_reaction_throws() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    assert!(matches!(
        session.surface(SurfaceTrigger::Kiss, None),
        Err(CoreError::NullArgument { param: "reaction" })
    ));
}

#[test]
fn surface_cap_is_enforced() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = deep_session(&profile, &mut state);
    for _ in 0..Session::MAX_SURFACES_PER_SESSION {
        session
            .surface(SurfaceTrigger::CountToZero, Some(reaction()))
            .expect("surfaces");
        session
            .resume_trance(&mut state, aligned(false, 0))
            .expect("resumes");
    }
    assert_eq!(
        Session::MAX_SURFACES_PER_SESSION,
        session.surfaces_this_session()
    );
    assert!(matches!(
        session.surface(SurfaceTrigger::CountToZero, Some(reaction())),
        Err(CoreError::InvalidOperation { .. })
    ));
}

#[test]
fn resume_overreach_still_meets_comfort_limit() {
    let profile = profile();
    let mut state = HeroineState::new("test-heroine").expect("state");
    let mut session = Session::new(&profile, &state).expect("session");
    session
        .apply_action(&mut state, aligned(false, 0))
        .expect("applies");
    session
        .surface(SurfaceTrigger::NameSpoken, Some(reaction()))
        .expect("surfaces");
    let result = session
        .resume_trance(&mut state, aligned(false, Session::MAX_TIER))
        .expect("the overreach is answered, not an error");
    assert!(result.ended_by_comfort_limit());
    assert_eq!(SessionEndReason::ComfortLimit, session.end_reason());
    assert!(session.aftercare_required());
    assert!(!session.is_surfaced());
}

#[test]
fn reaction_null_keys_normalize_to_empty() {
    // C# maps null keys to String.Empty in the constructor; the
    // port's keys are non-nullable strings and empty is the
    // designed state, stored verbatim.
    let reaction = SurfaceReaction::new(ReactionGrammar::GazeAvert, "", "");
    assert_eq!("", reaction.voice_line_key());
    assert_eq!("", reaction.expression_key());
}
