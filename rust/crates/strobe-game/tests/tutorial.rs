//! The D-161 tutorial battery: the guided first loop teaches
//! consent and the stop/pause surfaces FIRST, then the beat,
//! then progression — and every claim it makes is checked
//! against the simulation it drives. Apps here are built raw
//! (no harness consent): the consent gate is the subject.

mod common;

use std::sync::Arc;

use bevy_app::App;
use strobe_game::analytics::NullEventSink;
use strobe_game::sim::{
    AppConfig, LogEntry, PlayerSettings, SimInput, ToyEvent, ToySlot, build_app, push_input,
    run_ticks,
};
use strobe_game::tutorial::{Step, chapters};

use common::{core, load_data};

/// The settings the shipped binary boots with (defaults —
/// master off, ceiling 20), plus readout recording so the
/// delivery the tutorial teaches from is observable.
fn binary_settings() -> PlayerSettings {
    PlayerSettings {
        record_readout_history: true,
        ..PlayerSettings::default()
    }
}

fn raw_app() -> App {
    let settings = binary_settings();
    build_app(AppConfig {
        data: load_data(),
        settings,
        sink: Arc::new(NullEventSink),
        toy: ToySlot::Null(strobe_core::NullToyOutput),
        restored: None,
        restored_save: false,
        session_id: "tutorial-test".to_string(),
        save_path: None,
    })
}

fn run_chapters(app: &mut App, upto: usize) {
    for chapter in chapters().iter().take(upto) {
        for step in &chapter.steps {
            match step {
                Step::Input(input) => {
                    push_input(app, input.clone());
                    run_ticks(app, 1);
                }
                Step::Wait(ticks) => run_ticks(app, *ticks),
            }
        }
    }
}

fn log(app: &App) -> Vec<LogEntry> {
    core(app).log.clone()
}

fn position(entries: &[LogEntry], needle: &LogEntry) -> Option<usize> {
    entries.iter().position(|entry| entry == needle)
}

fn refusal(entries: &[LogEntry], reason_part: &str) -> Option<usize> {
    entries.iter().position(|entry| match entry {
        LogEntry::Refused { reason, .. } => reason.contains(reason_part),
        _ => false,
    })
}

#[test]
fn chapters_teach_in_the_required_order() {
    let chapters = chapters();
    assert_eq!(5, chapters.len());
    assert!(
        chapters[1].title.contains("consent"),
        "{}",
        chapters[1].title
    );
    assert!(
        chapters[2].title.contains("stop") && chapters[2].title.contains("pause"),
        "{}",
        chapters[2].title
    );
    assert!(chapters[3].title.contains("beat"), "{}", chapters[3].title);
    assert!(
        chapters[4].title.contains("progress"),
        "{}",
        chapters[4].title
    );
}

#[test]
fn play_is_refused_until_consent_and_consent_is_not_saved() {
    let mut app = raw_app();
    assert!(!core(&app).consent_given());
    push_input(&mut app, SimInput::RequestSession);
    run_ticks(&mut app, 1);
    let entries = log(&app);
    assert!(
        refusal(&entries, "consent required").is_some(),
        "log: {entries:?}"
    );
    assert_eq!(0, core(&app).sessions_started);
    // Safety and settings never wait for consent (U5): none of
    // them may be refused FOR CONSENT (the surface probe is
    // refused only because no session is active — a state
    // refusal, not a consent refusal).
    push_input(&mut app, SimInput::SetPlayerCeiling { ceiling: 7 });
    push_input(&mut app, SimInput::FireSurface);
    push_input(&mut app, SimInput::DisarmToy);
    push_input(&mut app, SimInput::SetToyMaster { on: false });
    run_ticks(&mut app, 1);
    let entries = log(&app);
    assert_eq!(
        1,
        entries
            .iter()
            .filter(|e| match e {
                LogEntry::Refused { reason, .. } => reason.contains("consent required"),
                _ => false,
            })
            .count(),
        "only the session request was consent-refused: {entries:?}"
    );
    assert_eq!(7, core(&app).settings.player_ceiling);
    // Consent lands, and play opens.
    push_input(&mut app, SimInput::GiveConsent);
    run_ticks(&mut app, 1);
    assert!(core(&app).consent_given());
    assert!(log(&app).contains(&LogEntry::ConsentGiven));
    push_input(&mut app, SimInput::RequestSession);
    run_ticks(&mut app, 1);
    assert_eq!(1, core(&app).sessions_started);
}

#[test]
fn the_tutorial_loop_teaches_each_claim_it_makes() {
    let mut app = raw_app();
    run_chapters(&mut app, 5);
    let entries = log(&app);
    // (a) The consent refusal precedes ConsentGiven.
    let refused = refusal(&entries, "consent required").expect("a consent refusal");
    let consented = position(&entries, &LogEntry::ConsentGiven).expect("consent given");
    assert!(refused < consented, "log: {entries:?}");
    // (a) The surface lesson: fired, held (action refused),
    // resumed — in that order.
    let surfaced = entries
        .iter()
        .position(|e| matches!(e, LogEntry::SurfaceFired { .. }))
        .expect("surface fired");
    let held = entries
        .iter()
        .position(|e| match e {
            LogEntry::Refused { reason, .. } => reason.contains("surface is held"),
            _ => false,
        })
        .expect("action refused while surfaced");
    let resumed = entries
        .iter()
        .position(|e| matches!(e, LogEntry::Resumed { .. }))
        .expect("resumed");
    assert!(surfaced < held && held < resumed, "log: {entries:?}");
    // (a) The comfort limit ends the practice session, and the
    // aftercare gate refuses play until it is acknowledged.
    let comfort = entries
        .iter()
        .position(|e| matches!(e, LogEntry::ComfortLimited { .. }))
        .expect("comfort limit fired");
    let gated = refusal(&entries, "mandatory aftercare").expect("aftercare gate refusal");
    assert!(comfort < gated, "log: {entries:?}");
    // (b) The chart delivered, clamped by the tier ceiling (5)
    // under the tutorial's own player ceiling (10) — and the
    // clamp actually engages (the master lesson is real: with
    // the master on, delivery is not zeros).
    let core = core(&app);
    assert!(!core.readout_history.is_empty());
    assert!(
        core.readout_history.iter().all(|(_, d)| *d <= 5),
        "peak: {:?}",
        core.readout_history.iter().map(|(_, d)| d).max()
    );
    assert!(
        core.readout_history.iter().any(|(_, d)| *d == 5),
        "the clamp engages at the envelope peak"
    );
    // (c) The beat session completed (the comfort-ended practice
    // session banks no clear); the aftercare gate is clear;
    // progression banked (1 clear, stage advanced past Unaware).
    assert_eq!(
        1,
        entries
            .iter()
            .filter(|e| matches!(e, LogEntry::Completed { .. }))
            .count()
    );
    assert!(!core.aftercare_gate);
    assert_eq!(1, core.state.clear_count());
    assert!(core.state.stage().discriminant() > 0);
    assert!(core.consent_given());
}

#[test]
fn the_tutorial_is_deterministic() {
    let mut first = raw_app();
    run_chapters(&mut first, 5);
    let mut second = raw_app();
    run_chapters(&mut second, 5);
    assert_eq!(format!("{:?}", log(&first)), format!("{:?}", log(&second)));
    assert_eq!(
        core(&first).canonical_state(),
        core(&second).canonical_state()
    );
    assert_eq!(core(&first).state_hash(), core(&second).state_hash());
}

#[test]
fn the_stop_zeroes_output_in_the_tutorial_session() {
    // Chapter 2's master-off stop, driven against a recording
    // slot: a Stop event lands during the tutorial's second
    // session chapter, before its completion.
    let toy_log = Arc::new(std::sync::Mutex::new(Vec::new()));
    let settings = binary_settings();
    let mut app = build_app(AppConfig {
        data: load_data(),
        settings,
        sink: Arc::new(NullEventSink),
        toy: ToySlot::Recorder(toy_log.clone()),
        restored: None,
        restored_save: false,
        session_id: "tutorial-stop".to_string(),
        save_path: None,
    });
    run_chapters(&mut app, 3);
    let events = toy_log.lock().expect("toy log").clone();
    assert!(
        events.iter().any(|e| matches!(e, ToyEvent::Stop)),
        "toy events: {events:?}"
    );
}
