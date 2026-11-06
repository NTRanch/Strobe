//! The §9 dev fixtures (SPEC-BEVY): seeded saves, built through
//! the Restore paths like any save, that put Vivienne one step
//! from the moments a tuning pass needs to see — excluded from
//! the shipped profile list and from analytics by construction
//! (they are test-only loads of the save format).

mod common;

use std::sync::Arc;

use strobe_core::W5Stage;
use strobe_game::analytics::NullEventSink;
use strobe_game::save::{LoadOutcome, SaveSnapshot, load_save, write_save};
use strobe_game::sim::{
    AppConfig, LogEntry, SimInput, SurfaceKind, ToySlot, build_app, stage_variant,
};

use common::{core, input, load_data, settings, temp_path};

/// A Craving-stage snapshot: 6 clears, Voice one session from its
/// capstone (119 XP), frequency learned, clock mid-season.
fn craving_snapshot() -> SaveSnapshot {
    SaveSnapshot {
        schema_version: 1,
        heroine_id: "vivienne".to_string(),
        clear_count: 6,
        // Discriminant order: Gaze, Rhythm, Touch, Voice.
        track_xp: [40, 30, 20, 119],
        outfit_coins: 90,
        frequency_learned: true,
        // Discriminant order: Aftercare, Feed, Protocols, Session,
        // Status, Store — all installed by the Willing sync.
        hub_installs: [true, true, true, true, true, true],
        hub_versions: [1, 1, 1, 1, 1, 1],
        clock_day: 3,
        clock_slot: 0,
        season_id: "vivienne-recital-residency".to_string(),
    }
}

fn app_from_snapshot(snapshot: &SaveSnapshot, tag: &str) -> bevy_app::App {
    let path = temp_path(tag);
    write_save(&path, snapshot).expect("write fixture save");
    let data = load_data();
    let restored = match load_save(&path, &data) {
        LoadOutcome::Restored(restored) => restored,
        _ => panic!("fixture must restore"),
    };
    let mut app = build_app(AppConfig {
        data,
        settings: settings(),
        sink: Arc::new(NullEventSink),
        toy: ToySlot::Null(strobe_core::NullToyOutput),
        restored: Some(*restored),
        restored_save: true,
        session_id: "fixture".to_string(),
        save_path: None,
    });
    // Consent is per-sitting and never restored from a save: the
    // fixture player consents fresh, like a real one.
    input(&mut app, SimInput::GiveConsent);
    app
}

#[test]
fn craving_fixture_requests_the_advance_and_resume_earns_the_bonus() {
    let mut app = app_from_snapshot(&craving_snapshot(), "craving");
    assert_eq!(W5Stage::Craving, core(&app).state.stage());
    input(&mut app, SimInput::RequestSession);
    // At Craving her baseline corruption (3 × 12 = 36) already
    // clears the tier-1 threshold: she requests the advance.
    assert!(core(&app).heroine_requests_next_tier());
    // Surface, then resume on her initiative: at her exact center
    // the factor is 1.0, so the resume earns 8 + 4 (hers) = 12.
    input(&mut app, SimInput::FireSurface);
    input(
        &mut app,
        SimInput::ResumeTrance {
            action_id: "she-hums-the-count".to_string(),
        },
    );
    let gained = core(&app).log.iter().find_map(|e| match e {
        LogEntry::Resumed {
            corruption_gained, ..
        } => Some(*corruption_gained),
        _ => None,
    });
    assert!((gained.expect("resumed") - 12.0).abs() < 1e-9);
}

#[test]
fn self_driven_seed_shows_the_end_state_stage() {
    let mut snapshot = craving_snapshot();
    snapshot.clear_count = 15;
    snapshot.clock_day = 6;
    let mut app = app_from_snapshot(&snapshot, "self-driven");
    assert_eq!(W5Stage::SelfDriven, core(&app).state.stage());
    // All surfaces resolve stage 5 in the end state.
    for surface in SurfaceKind::ALL {
        assert!(stage_variant(surface, W5Stage::SelfDriven).ends_with("stage-5"));
    }
    input(&mut app, SimInput::RequestSession);
    assert!(core(&app).heroine_requests_next_tier());
}
