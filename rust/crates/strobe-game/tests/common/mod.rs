//! Shared harness for the strobe-game gate tests: loads the real
//! authored data and builds the headless app the same way the
//! binary does. Each test crate uses a subset of the harness, so
//! dead-code warnings are expected here and allowed module-wide.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy_app::App;
use strobe_game::analytics::NullEventSink;
use strobe_game::data::GameData;
use strobe_game::sim::{
    AppConfig, PlayerSettings, RecordingSink, SimInput, ToyEvent, ToySlot, build_app, push_input,
    run_ticks,
};

pub fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/data")
}

pub fn load_data() -> GameData {
    GameData::load(&data_dir()).expect("the authored data loads")
}

pub fn settings() -> PlayerSettings {
    PlayerSettings {
        toy_master_on: true,
        player_ceiling: 20,
        analytics_enabled: true,
        install_id: "install-test".to_string(),
        record_readout_history: false,
    }
}

/// An app with a recording sink and a recording toy slot.
pub struct Rig {
    pub app: App,
    pub sink: Arc<RecordingSink>,
    pub toy_log: Arc<Mutex<Vec<ToyEvent>>>,
}

pub fn rig() -> Rig {
    rig_with(settings())
}

pub fn rig_with(settings: PlayerSettings) -> Rig {
    let sink = Arc::new(RecordingSink::new());
    let toy_log = Arc::new(Mutex::new(Vec::new()));
    let mut app = build_app(AppConfig {
        data: load_data(),
        settings,
        sink: sink.clone(),
        toy: ToySlot::Recorder(toy_log.clone()),
        restored: None,
        restored_save: false,
        session_id: "test-session".to_string(),
        save_path: None,
    });
    // The harness plays as a consenting player (the consent gate
    // itself is exercised by the tutorial/consent battery, which
    // builds its apps raw).
    input(&mut app, SimInput::GiveConsent);
    Rig { app, sink, toy_log }
}

/// An app with the null sink (sink-off equivalence runs).
pub fn rig_null_sink(settings: PlayerSettings) -> App {
    let mut app = build_app(AppConfig {
        data: load_data(),
        settings,
        sink: Arc::new(NullEventSink),
        toy: ToySlot::Null(strobe_core::NullToyOutput),
        restored: None,
        restored_save: false,
        session_id: "test-session".to_string(),
        save_path: None,
    });
    input(&mut app, SimInput::GiveConsent);
    app
}

/// Feed one input and step once (the input lands on that step).
pub fn input(app: &mut App, input: SimInput) {
    push_input(app, input);
    run_ticks(app, 1);
}

/// Apply an authored action by id.
pub fn act(app: &mut App, action_id: &str) {
    input(
        app,
        SimInput::ApplyAction {
            action_id: action_id.to_string(),
        },
    );
}

/// The sim core, borrowed for assertions.
pub fn core(app: &App) -> &strobe_game::sim::SimCore {
    app.world().resource::<strobe_game::sim::SimCore>()
}

/// A unique temp file path for save/analytics tests.
pub fn temp_path(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("strobe-test-{}-{}", std::process::id(), tag));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("file.json")
}
