//! The G7-B save gates: snapshot → restore round-trip identity,
//! and a corrupt save starting the game fresh with the bad file
//! preserved as `.corrupt` — never overwritten, never deleted.

mod common;

use std::sync::Arc;
use strobe_game::analytics::NullEventSink;
use strobe_game::save::{LoadOutcome, SaveSnapshot, load_save, snapshot, write_save};
use strobe_game::sim::{AppConfig, SimInput, ToySlot, build_app};

use common::{act, core, input, load_data, rig, settings, temp_path};

fn play_partial_route(app: &mut bevy_app::App) {
    input(app, SimInput::RequestSession);
    act(app, "listen-with-her");
    act(app, "watch-her-listen");
    // One more tier-0 listen crosses corruption 20: tier 1 opens,
    // so the tier-1 echo below is legal play, not an overreach.
    act(app, "listen-with-her");
    act(app, "echo-her-phrase");
    input(app, SimInput::CompleteSession);
    input(app, SimInput::AcknowledgeAftercare);
}

#[test]
fn save_round_trip_restores_the_identical_state() {
    let mut first = rig();
    play_partial_route(&mut first.app);
    let path = temp_path("round-trip");
    let snap = {
        let core = core(&first.app);
        snapshot(core.data(), &core.state, &core.hub, &core.clock)
    };
    write_save(&path, &snap).expect("write save");
    let canonical_before = core(&first.app).canonical_state();

    let data = load_data();
    let restored = match load_save(&path, &data) {
        LoadOutcome::Restored(restored) => restored,
        _ => panic!("a written save must restore"),
    };
    let app2 = build_app(AppConfig {
        data,
        settings: settings(),
        sink: Arc::new(NullEventSink),
        toy: ToySlot::Null(strobe_core::NullToyOutput),
        restored: Some(*restored),
        restored_save: true,
        session_id: "test-session".to_string(),
        save_path: None,
    });
    let core2 = app2.world().resource::<strobe_game::sim::SimCore>();
    assert_eq!(canonical_before, core2.canonical_state());
}

#[test]
fn corrupt_save_starts_fresh_and_is_preserved() {
    let path = temp_path("corrupt");
    std::fs::write(&path, b"{ this is not a save").expect("write garbage");
    let data = load_data();
    match load_save(&path, &data) {
        LoadOutcome::CorruptPreserved { preserved_path } => {
            assert!(!path.exists(), "the bad file moved aside");
            let preserved = std::fs::read(&preserved_path).expect("preserved bytes readable");
            assert_eq!(b"{ this is not a save".to_vec(), preserved);
        }
        _ => panic!("garbage must read as corrupt"),
    }
}

#[test]
fn foreign_snapshot_is_refused_as_corrupt() {
    let path = temp_path("foreign");
    let foreign = SaveSnapshot {
        schema_version: 1,
        heroine_id: "someone-else".to_string(),
        clear_count: 0,
        track_xp: [0; 4],
        outfit_coins: 0,
        frequency_learned: false,
        hub_installs: [true, true, false, true, false, false],
        hub_versions: [1, 1, 0, 1, 0, 0],
        clock_day: 1,
        clock_slot: 0,
        season_id: "vivienne-recital-residency".to_string(),
    };
    write_save(&path, &foreign).expect("write foreign save");
    let data = load_data();
    assert!(matches!(
        load_save(&path, &data),
        LoadOutcome::CorruptPreserved { .. }
    ));
}
