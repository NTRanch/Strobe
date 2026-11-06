//! The headless Strobe binary: boots the app, optionally
//! restores a save, plays a scripted input file (or the built-in
//! demo route — SPEC §1's first session), writes the save and
//! settings beside each other, and prints the canonical state.
//!
//! Usage:
//!   strobe [--data DIR] [--save PATH] [--settings PATH]
//!          [--script PATH] [--ticks N] [--analytics PATH]
//!          [--tutorial]
//!
//! With `--tutorial`, the guided first loop runs instead of a
//! script (D-161): consent and the stop/pause surfaces first,
//! then the pattern and the beat, then progression — taught
//! through the simulation's own inputs, with the toy slot
//! recording so the delivery is visible in the output.
//!
//! With no script, the demo route plays: view Status, request a
//! session, play the authored tier-0/1 actions, fire the Surface
//! Beat at the authored moment, resume with her count, complete,
//! and acknowledge the aftercare close. No audio is ever played
//! (estate audio-silence law): the chart is data, and this binary
//! renders nothing audible.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;

use strobe_game::analytics::{JsonlEventSink, NullEventSink};
use strobe_game::data::GameData;
use strobe_game::save::{LoadOutcome, load_save, snapshot, write_save};
use strobe_game::sim::{
    AppConfig, LogEntry, PlayerSettings, Screen, SimCore, SimInput, ToyEvent, ToySlot, build_app,
    push_input, run_ticks,
};
use strobe_game::tutorial::{Step, chapters};

#[derive(serde::Deserialize)]
struct ScriptStep {
    tick: u64,
    input: SimInput,
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn default_data_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("assets/data"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/data"),
    ];
    for candidate in candidates {
        if candidate.join("vivienne.json").exists() {
            return candidate;
        }
    }
    PathBuf::from("assets/data")
}

fn demo_script() -> Vec<ScriptStep> {
    let mut steps: Vec<ScriptStep> = Vec::new();
    let mut tick = 1_u64;
    let mut push = |steps: &mut Vec<ScriptStep>, input: SimInput| {
        steps.push(ScriptStep { tick, input });
        tick += 30;
    };
    // Consent first, always: the demo consents before it looks
    // at anything, exactly as a player does.
    push(&mut steps, SimInput::GiveConsent);
    push(
        &mut steps,
        SimInput::Navigate {
            screen: Screen::Status,
        },
    );
    push(
        &mut steps,
        SimInput::Navigate {
            screen: Screen::Hub,
        },
    );
    push(&mut steps, SimInput::RequestSession);
    for action in [
        "listen-with-her",
        "watch-her-listen",
        "listen-with-her",
        "echo-her-phrase",
        "meet-her-gaze",
        "keep-the-count",
    ] {
        push(
            &mut steps,
            SimInput::ApplyAction {
                action_id: action.to_string(),
            },
        );
    }
    push(&mut steps, SimInput::FireSurface);
    push(
        &mut steps,
        SimInput::ResumeTrance {
            action_id: "she-hums-the-count".to_string(),
        },
    );
    push(
        &mut steps,
        SimInput::ApplyAction {
            action_id: "count-her-down".to_string(),
        },
    );
    push(&mut steps, SimInput::CompleteSession);
    push(&mut steps, SimInput::AcknowledgeAftercare);
    steps
}

/// Run the guided first loop: print each chapter's teaching
/// text, drive its steps through the real input queue, and
/// stream the simulation's answers (log entries, toy events)
/// after each chapter.
fn run_tutorial(app: &mut bevy_app::App, toy_events: &Arc<std::sync::Mutex<Vec<ToyEvent>>>) {
    let mut log_seen = 0_usize;
    let mut toy_seen = 0_usize;
    for (index, chapter) in chapters().iter().enumerate() {
        println!("\n== Chapter {}: {} ==", index + 1, chapter.title);
        for line in chapter.body {
            println!("{line}");
        }
        for step in &chapter.steps {
            match step {
                Step::Input(input) => {
                    push_input(app, input.clone());
                    run_ticks(app, 1);
                }
                Step::Wait(ticks) => run_ticks(app, *ticks),
            }
        }
        let core = app.world().resource::<SimCore>();
        let fresh = &core.log[log_seen..];
        for entry in fresh {
            println!("log: {entry:?}");
        }
        log_seen = core.log.len();
        if let Ok(events) = toy_events.lock() {
            for event in &events[toy_seen..] {
                println!("toy: {event:?}");
            }
            toy_seen = events.len();
        }
        if index == 3 && !core.readout_history.is_empty() {
            let peak = core
                .readout_history
                .iter()
                .map(|(_, delivered)| *delivered)
                .max()
                .unwrap_or(0);
            println!(
                "readout: {} delivered steps, peak strength {peak}",
                core.readout_history.len()
            );
        }
        if fresh.iter().any(|entry| {
            matches!(
                entry,
                LogEntry::Completed { .. } | LogEntry::ComfortLimited { .. }
            )
        }) && let Some(summary) = core.last_summary
        {
            println!("summary: {summary:?}");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tutorial_mode = args.iter().any(|arg| arg == "--tutorial");
    let data_dir = arg_value(&args, "--data")
        .map(PathBuf::from)
        .unwrap_or_else(default_data_dir);
    let save_path = arg_value(&args, "--save").map(PathBuf::from);
    let settings_path = arg_value(&args, "--settings").map(PathBuf::from);
    let script_path = arg_value(&args, "--script").map(PathBuf::from);
    let analytics_path = arg_value(&args, "--analytics").map(PathBuf::from);
    let ticks: u64 = arg_value(&args, "--ticks")
        .and_then(|value| value.parse().ok())
        .unwrap_or(3600);

    let data = match GameData::load(&data_dir) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("strobe: data load failed: {error}");
            std::process::exit(2);
        }
    };
    let mut settings = settings_path
        .as_ref()
        .map(|path| PlayerSettings::load_or_default(path))
        .unwrap_or_default();
    if settings.install_id == "install-unset" {
        // A random-per-install id from the system clock + pid: the
        // only nondeterminism at boot, kept out of the simulation.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        settings.install_id = format!("install-{seed:x}-{}", std::process::id());
    }
    let (restored, restored_save) = match &save_path {
        Some(path) => match load_save(path, &data) {
            LoadOutcome::Fresh => (None, false),
            LoadOutcome::Restored(restored) => (Some(*restored), true),
            LoadOutcome::CorruptPreserved { preserved_path } => {
                eprintln!(
                    "strobe: save was corrupt; preserved at {}; starting fresh",
                    preserved_path.display()
                );
                (None, false)
            }
        },
        None => (None, false),
    };
    let sink: Arc<dyn strobe_game::analytics::EventSink> = if settings.analytics_enabled {
        match analytics_path {
            Some(path) => Arc::new(JsonlEventSink::new(path)),
            None => Arc::new(NullEventSink),
        }
    } else {
        Arc::new(NullEventSink)
    };
    let toy_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut sim_settings = settings.clone();
    if tutorial_mode {
        // The tutorial teaches from the delivery readout, so the
        // history the readout is drawn from must be recorded.
        sim_settings.record_readout_history = true;
    }
    let mut app = build_app(AppConfig {
        data,
        settings: sim_settings,
        sink,
        toy: if tutorial_mode {
            ToySlot::Recorder(toy_events.clone())
        } else {
            ToySlot::Null(strobe_core::NullToyOutput)
        },
        restored,
        restored_save,
        session_id: format!("run-{}", std::process::id()),
        save_path: save_path.clone(),
    });

    if tutorial_mode {
        run_tutorial(&mut app, &toy_events);
        let core = app.world().resource::<SimCore>();
        println!("canonical: {}", core.canonical_state());
        println!("sha256: {}", core.state_hash());
        finish(&mut app, &save_path, &settings_path, restored_save);
        return;
    }

    let script: Vec<ScriptStep> = match &script_path {
        Some(path) => {
            let raw = std::fs::read_to_string(path).unwrap_or_else(|error| {
                eprintln!("strobe: cannot read script {}: {error}", path.display());
                std::process::exit(2);
            });
            serde_json::from_str(&raw).unwrap_or_else(|error| {
                eprintln!("strobe: script parse failed: {error}");
                std::process::exit(2);
            })
        }
        None => demo_script(),
    };
    let last_tick = script.iter().map(|step| step.tick).max().unwrap_or(0);
    let total_ticks = ticks.max(last_tick + 120);
    for step in &script {
        let already = app.world().resource::<SimCore>().tick;
        if step.tick > already {
            run_ticks(&mut app, step.tick - already);
        }
        push_input(&mut app, step.input.clone());
        run_ticks(&mut app, 1);
    }
    let already = app.world().resource::<SimCore>().tick;
    if total_ticks > already {
        run_ticks(&mut app, total_ticks - already);
    }

    let core = app.world().resource::<SimCore>();
    for entry in &core.log {
        println!("log: {entry:?}");
    }
    println!("canonical: {}", core.canonical_state());
    println!("sha256: {}", core.state_hash());
    finish(&mut app, &save_path, &settings_path, restored_save);
}

/// The shared run tail: write the save beside the settings,
/// emit the app-session end, and persist the settings.
fn finish(
    app: &mut bevy_app::App,
    save_path: &Option<PathBuf>,
    settings_path: &Option<PathBuf>,
    restored_save: bool,
) {
    let core = app.world().resource::<SimCore>();
    if let Some(path) = save_path {
        let snap = snapshot(core.data(), &core.state, &core.hub, &core.clock);
        match write_save(path, &snap) {
            Ok(()) => println!("save: written to {}", path.display()),
            Err(error) => eprintln!("strobe: save write failed: {error}"),
        }
    }
    core.emit_session_end(restored_save);
    if let Some(path) = settings_path
        && let Err(error) = core.settings.write(path)
    {
        eprintln!("strobe: settings write failed: {error}");
    }
}
