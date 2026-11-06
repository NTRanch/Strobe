//! Port of the C# `HubClockToyTests.cs` (16 cases) from
//! `Tests/Strobe.Core.Tests` in primo `~/strobe-trackb`.
//!
//! A1 hub, A4 clock, and the toy intensity law. Validation: the
//! unlock schedule, slot accounting, and the clamp arithmetic.
//! Adversarial: no seventh app can exist, upgrades cannot precede
//! installs, the season cannot overspend, and intensity can never
//! exceed the tier ceiling, the player ceiling, or a switched-off
//! master — regardless of the caller's curve value.

use strobe_core::{
    CampaignClock, CoreError, HeroineState, PhoneApp, PhoneHub, ResponseTrack, SeasonEnding,
    Session, ToyCurve, W5Stage,
};

#[test]
fn phone_hub_ships_with_session_status_and_aftercare() {
    // Decision 5A (Owner ruling 2026-10-25): aftercare is the
    // closing surface of every session, so it ships installed at
    // v1 from minute one — it is never an unlock.
    let hub = PhoneHub::new();
    assert!(hub.is_installed(PhoneApp::Session));
    assert!(hub.is_installed(PhoneApp::Status));
    assert!(hub.is_installed(PhoneApp::Aftercare));
    assert_eq!(1, hub.version_of(PhoneApp::Aftercare));
    assert!(!hub.is_installed(PhoneApp::Feed));
    assert!(!hub.is_installed(PhoneApp::Store));
    assert!(!hub.is_installed(PhoneApp::Protocols));
    assert_eq!(6, PhoneHub::APP_COUNT);
    assert_eq!(6, PhoneApp::ALL.len());
}

#[test]
fn phone_hub_upgrade_before_install_throws() {
    let mut hub = PhoneHub::new();
    assert!(matches!(
        hub.upgrade(PhoneApp::Store),
        Err(CoreError::InvalidOperation { .. })
    ));
}

#[test]
fn phone_hub_sync_from_state_follows_the_schedule() {
    let mut hub = PhoneHub::new();
    let mut state = HeroineState::new("test-heroine").expect("state");
    hub.sync_from_state(&state); // Unaware: nothing new installs
    assert!(!hub.is_installed(PhoneApp::Feed));
    assert!(!hub.is_installed(PhoneApp::Store));
    assert!(!hub.is_installed(PhoneApp::Protocols));
    assert!(hub.is_installed(PhoneApp::Aftercare));

    state.restore_clear_count(3); // Willing
    hub.sync_from_state(&state);
    assert!(hub.is_installed(PhoneApp::Feed));
    assert!(hub.is_installed(PhoneApp::Store));
    assert!(hub.is_installed(PhoneApp::Protocols));
    assert_eq!(1, hub.version_of(PhoneApp::Protocols));
}

#[test]
fn phone_hub_protocols_version_tracks_capstones() {
    let mut hub = PhoneHub::new();
    let mut state = HeroineState::new("test-heroine").expect("state");
    state.restore_clear_count(3);
    state.restore_track_xp(ResponseTrack::Voice, 130);
    state.restore_track_xp(ResponseTrack::Gaze, 130);
    hub.sync_from_state(&state);
    assert_eq!(3, hub.version_of(PhoneApp::Protocols)); // 2 capstones + 1
}

#[test]
fn campaign_clock_spends_slots_and_rolls_days() {
    let mut clock = CampaignClock::new(2, 4).expect("clock");
    for _ in 0..4 {
        assert!(clock.spend_slot());
    }
    assert_eq!(2, clock.day());
    assert_eq!(4, clock.slots_left_today());
    for _ in 0..4 {
        assert!(clock.spend_slot());
    }
    assert!(clock.season_over());
    assert!(!clock.spend_slot()); // cannot overspend the season
    assert_eq!(0, clock.slots_left_today());
}

#[test]
fn campaign_clock_endings_self_driven() {
    assert_eq!(
        SeasonEnding::SelfDriven,
        CampaignClock::evaluate_ending(W5Stage::SelfDriven)
    );
}

#[test]
fn campaign_clock_endings_devoted() {
    assert_eq!(
        SeasonEnding::Deepened,
        CampaignClock::evaluate_ending(W5Stage::Devoted)
    );
}

#[test]
fn campaign_clock_endings_craving() {
    assert_eq!(
        SeasonEnding::Deepened,
        CampaignClock::evaluate_ending(W5Stage::Craving)
    );
}

#[test]
fn campaign_clock_endings_willing() {
    assert_eq!(
        SeasonEnding::Begun,
        CampaignClock::evaluate_ending(W5Stage::Willing)
    );
}

#[test]
fn campaign_clock_endings_curious() {
    assert_eq!(
        SeasonEnding::Begun,
        CampaignClock::evaluate_ending(W5Stage::Curious)
    );
}

#[test]
fn campaign_clock_endings_unaware() {
    assert_eq!(
        SeasonEnding::NotYet,
        CampaignClock::evaluate_ending(W5Stage::Unaware)
    );
}

#[test]
fn campaign_clock_invalid_shape_throws() {
    assert!(matches!(
        CampaignClock::new(0, 4),
        Err(CoreError::OutOfRange {
            param: "total_days",
            ..
        })
    ));
    assert!(matches!(
        CampaignClock::new(10, 0),
        Err(CoreError::OutOfRange {
            param: "slots_per_day",
            ..
        })
    ));
    assert!(matches!(
        CampaignClock::new(10, 99),
        Err(CoreError::OutOfRange {
            param: "slots_per_day",
            ..
        })
    ));
}

#[test]
fn toy_curve_master_off_always_zero() {
    assert_eq!(
        0,
        ToyCurve::compute_intensity(false, 1.0, Session::MAX_TIER, 20)
    );
    assert_eq!(0, ToyCurve::compute_intensity(false, 0.5, 2, 20));
}

#[test]
fn toy_curve_tier_zero_is_silent() {
    // Her first willing tier is the floor for any output.
    assert_eq!(0, ToyCurve::compute_intensity(true, 1.0, 0, 20));
}

#[test]
fn toy_curve_never_exceeds_tier_or_player_ceiling() {
    // Tier 2 of 4 -> ceiling 10; player ceiling 7 wins.
    assert_eq!(7, ToyCurve::compute_intensity(true, 1.0, 2, 7));
    // Full curve, full tier, full ceiling -> full scale.
    assert_eq!(20, ToyCurve::compute_intensity(true, 1.0, 4, 20));
    // Half curve at tier 4 -> 10.
    assert_eq!(10, ToyCurve::compute_intensity(true, 0.5, 4, 20));
}

#[test]
fn toy_curve_hostile_inputs_are_clamped_not_trusted() {
    assert_eq!(20, ToyCurve::compute_intensity(true, 999.0, 99, 999));
    assert_eq!(0, ToyCurve::compute_intensity(true, -5.0, 4, 20));
    assert_eq!(0, ToyCurve::compute_intensity(true, 1.0, -3, 20));
    assert_eq!(0, ToyCurve::compute_intensity(true, 1.0, 4, -1));
}
