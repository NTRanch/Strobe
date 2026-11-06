//! `strobe-play` — the windowed Strobe demo (D-179, boundary
//! per D-181: the full first chapter).
//!
//! A real Bevy window over the *same* simulation the headless
//! binary runs: the sim app is built by `strobe_game::sim::
//! build_app` untouched and stepped at its own fixed rate
//! (120 steps/s) from a real-time accumulator, so live input
//! carries real timing — a chart answer's offset is the
//! player's actual press offset, compensated by a fixed
//! render lag of one judgement window (see `RENDER_LAG_STEPS`).
//!
//! Demo boundary (D-181 mapping, stated explicitly): Strobe
//! has no formal chapter divisions. Vivienne's route is six
//! stage-scenes (stories/01-vivienne.md); the first complete
//! narrative arc is Stage 0 (Unaware — "The Voice in the
//! Walls", Scene 1) resolving into Stage 1 (Curious — the
//! first spark). Mechanically that is the first banked clear:
//! `Progression` thresholds [1, 3, 6, 10, 15] put Curious at
//! clear #1. The guided demo therefore ends — end card — on
//! the first stage advance, Unaware → Curious, with the Feed
//! unlock shown. Free play continues after the card.
//!
//! Safety posture is the sim's own (U5): consent gates play
//! inputs and never safety inputs; SURFACE and STOP (master
//! off) are on screen from the first frame, before consent.
//! No audio exists or is played (estate audio-silence law).
//! Her lines are never invented: character line keys whose
//! wording is still owed render as narration, not as quotes.
//!
//! Usage:
//!   strobe-play [--data DIR] [--save PATH] [--settings PATH]
//!               [--autoplay] [--shots-hold]
//!
//! `--autoplay` drives the guided route with timed synthetic
//! presses (the screenshot/smoke harness on primo); phase
//! markers print to stdout as `PHASE <name>`.

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use strobe_core::{PhoneApp, ResponseTrack, W5Stage};
use strobe_game::analytics::EndedVia;
use strobe_game::data::GameData;
use strobe_game::save::{LoadOutcome, load_save, snapshot, write_save};
use strobe_game::sim::{
    AppConfig, LogEntry, PlayerSettings, Screen, SimCore, SimInput, SummaryView, SurfaceKind,
    ToyEvent, ToySlot, build_app, push_input, run_ticks, stage_variant,
};

/// Fixed sim steps per second (the chart's own constant).
const STEPS_PER_SECOND: f64 = 120.0;
/// Steps per chart beat at 120 BPM.
const STEPS_PER_BEAT: f64 = 60.0;
/// Render/input lag compensation, in sim steps: exactly one
/// full judgement window (±12 steps, `chart::accuracy_for_
/// offset`). The player answers against the position they
/// perceive; the input reaches the sim one window early, so
/// early *and* late presses inside the window both register
/// with their true signed offset.
const RENDER_LAG_STEPS: f64 = 12.0;
/// Portrait texture geometry (the embedded RGBA asset).
const PORTRAIT_W: u32 = 512;
const PORTRAIT_H: u32 = 960;
const PORTRAIT_RGBA: &[u8] = include_bytes!("../../../../assets/art/vivienne_begin.rgba");

// ----------------------------------------------------------------
// Palette (Strobe book theme: black-cherry / violet / amethyst).

fn col(hex: u32) -> Color {
    Color::srgb(
        ((hex >> 16) & 0xff) as f32 / 255.0,
        ((hex >> 8) & 0xff) as f32 / 255.0,
        (hex & 0xff) as f32 / 255.0,
    )
}
const BG: u32 = 0x14090f;
const PANEL: u32 = 0x26121e;
const PANEL_EDGE: u32 = 0x53304a;
const BTN: u32 = 0x3a1f31;
const BTN_HOVER: u32 = 0x57304a;
const BTN_GUIDE: u32 = 0x6d2a4e;
const BTN_SAFETY: u32 = 0x7a1f2b;
const INK: u32 = 0xf3e9f2;
const DIM: u32 = 0xb9a3b5;
const GOLD: u32 = 0xffd479;
const VIOLET: u32 = 0x9a6bff;

// ----------------------------------------------------------------
// Small naming helpers (presentation only).

fn stage_name(stage: W5Stage) -> &'static str {
    match stage {
        W5Stage::Unaware => "Unaware",
        W5Stage::Curious => "Curious",
        W5Stage::Willing => "Willing",
        W5Stage::Craving => "Craving",
        W5Stage::Devoted => "Devoted",
        W5Stage::SelfDriven => "Self-Driven",
    }
}

fn stage_scene(stage: W5Stage) -> &'static str {
    match stage {
        W5Stage::Unaware => "Scene 1 — The recital hall, after the showcase",
        W5Stage::Curious => "Scene 2 — First session: one song, as a favor",
        W5Stage::Willing => "Scene 3 — The Halo Room, closing set",
        W5Stage::Craving => "Scene 4 — The dressing-room mirror",
        W5Stage::Devoted => "Scene 5 — The apartment, honesty",
        W5Stage::SelfDriven => "Scene 6 — The session room, last session of the arc",
    }
}

fn track_name(track: ResponseTrack) -> &'static str {
    match track {
        ResponseTrack::Gaze => "Gaze",
        ResponseTrack::Rhythm => "Rhythm",
        ResponseTrack::Touch => "Touch",
        ResponseTrack::Voice => "Voice",
    }
}

fn app_name(app: PhoneApp) -> &'static str {
    match app {
        PhoneApp::Aftercare => "Aftercare",
        PhoneApp::Feed => "Feed",
        PhoneApp::Protocols => "Protocols",
        PhoneApp::Session => "Session",
        PhoneApp::Status => "Status",
        PhoneApp::Store => "Store",
    }
}

/// "listen-with-her" → "Listen with her" (display only; the
/// sim only ever sees the authored id).
fn prettify(id: &str) -> String {
    let mut out = String::new();
    for (index, word) in id.split('-').enumerate() {
        if index > 0 {
            out.push(' ');
        }
        if index == 0 {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
                continue;
            }
        }
        out.push_str(word);
    }
    out
}

fn fmt_log(entry: &LogEntry) -> String {
    match entry {
        LogEntry::ActionApplied {
            action_id,
            track_xp_after,
            ..
        } => format!(
            "Action: {} (track XP now {})",
            prettify(action_id),
            track_xp_after
        ),
        LogEntry::TierUnlocked { tier, .. } => format!("Tier {tier} unlocked"),
        LogEntry::CapstoneUnlocked { track, .. } => {
            format!("Capstone unlocked: {}", track_name(*track))
        }
        LogEntry::Refused { what, reason } => format!("Refused: {what} — {reason}"),
        LogEntry::SurfaceFired { .. } => "She surfaces — the pause takes hold".to_string(),
        LogEntry::Resumed { action_id, .. } => {
            format!("Resumed: {}", prettify(action_id))
        }
        LogEntry::ComfortLimited {
            attempted_tier,
            allowed_tier,
        } => format!(
            "Her comfort limit: tier {attempted_tier} reached past tier {allowed_tier} — session ended by her stop"
        ),
        LogEntry::Completed {
            clears,
            coins_earned,
        } => format!("Session complete: {clears} clear(s) banked, {coins_earned} coins"),
        LogEntry::ConsentGiven => "Consent given for this sitting".to_string(),
        LogEntry::SessionStarted { sequence } => format!("Session {sequence} started"),
        LogEntry::FrequencyLearned => "Her frequency is learned".to_string(),
        LogEntry::SeasonEnded { .. } => "The season has ended".to_string(),
    }
}

// ----------------------------------------------------------------
// Resources

/// The headless sim app, stepped manually (non-send: it is
/// only ever touched on the main thread).
struct SimHandle {
    app: bevy_app::App,
    accumulator: f64,
    save_path: Option<PathBuf>,
    settings_path: Option<PathBuf>,
    restored_save: bool,
    saved_on_exit: bool,
}

/// A plain-data snapshot of the sim, rebuilt after each step
/// batch; every visual reads from here, never from the sim.
#[derive(Resource)]
struct Snap {
    screen: Screen,
    consent: bool,
    stage: W5Stage,
    coins: i32,
    clears: i32,
    day: i32,
    slot: i32,
    slots_per_day: i32,
    total_days: i32,
    xp: [i32; 4],
    freq_learned: bool,
    in_session: bool,
    surfaced: bool,
    session_closed: bool,
    tier: Option<i32>,
    curve: f64,
    delivered: i32,
    tier_ceiling: i32,
    aftercare_gate: bool,
    summary: Option<SummaryView>,
    ceiling: i32,
    master_on: bool,
    analytics_on: bool,
    log_len: usize,
    log_tail: Vec<String>,
    last_toy: String,
    feed_variant: String,
    hub_installed: [bool; 6],
    season_over: bool,
}

impl Default for Snap {
    fn default() -> Self {
        Snap {
            screen: Screen::Hub,
            consent: false,
            stage: W5Stage::Unaware,
            coins: 0,
            clears: 0,
            day: 1,
            slot: 0,
            slots_per_day: 3,
            total_days: 10,
            xp: [0; 4],
            freq_learned: false,
            in_session: false,
            surfaced: false,
            session_closed: false,
            tier: None,
            curve: 0.0,
            delivered: 0,
            tier_ceiling: 0,
            aftercare_gate: false,
            summary: None,
            ceiling: 20,
            master_on: false,
            analytics_on: true,
            log_len: 0,
            log_tail: Vec::new(),
            last_toy: String::new(),
            feed_variant: String::new(),
            hub_installed: [false; 6],
            season_over: false,
        }
    }
}

/// Presentation-side state: views, the guide, chart tracking.
#[derive(Resource)]
struct Ui {
    view: View,
    ceiling_choice: i32,
    practice: Option<PracticeStep>,
    practice_done: bool,
    chapter_done: bool,
    end_card_seen: bool,
    sessions_seen: usize,
    toy_len: usize,
    chart_pos: f64,
    chart_live: bool,
    answered: HashSet<u32>,
    missed: HashSet<u32>,
    flash: String,
    flash_until: f64,
    time: f64,
    last_phase_printed: String,
    autoplay: bool,
    auto_t: f64,
    auto_stage: String,
    auto_fired: HashSet<usize>,
    hover_idx: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Welcome,
    Consent,
    Game,
    EndCard,
}

/// The practice sequence (tutorial chapter 3, embodied): one
/// expected act at a time; safety buttons stay live beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PracticeStep {
    Listen,
    Surface,
    TryWhileSurfaced,
    Resume,
    Stop,
    Overreach,
}

impl Default for Ui {
    fn default() -> Self {
        Ui {
            view: View::Welcome,
            ceiling_choice: 10,
            practice: None,
            practice_done: false,
            chapter_done: false,
            end_card_seen: false,
            sessions_seen: 0,
            toy_len: 0,
            chart_pos: 0.0,
            chart_live: false,
            answered: HashSet::new(),
            missed: HashSet::new(),
            flash: String::new(),
            flash_until: 0.0,
            time: 0.0,
            last_phase_printed: String::new(),
            autoplay: false,
            auto_t: 0.0,
            auto_stage: String::new(),
            auto_fired: HashSet::new(),
            hover_idx: None,
        }
    }
}

/// Static per-binary data the presentation needs (a second
/// load of the same files the sim loaded).
#[derive(Resource)]
struct PlayData {
    data: GameData,
}

#[derive(Resource)]
struct FontRes {
    regular: Handle<Font>,
    bold: Handle<Font>,
}

// ----------------------------------------------------------------
// UI model: buttons + lines, rebuilt per frame from Snap + Ui.

#[derive(Clone, PartialEq)]
enum Act {
    Begin,
    TrySession,
    Consent,
    Ceiling(i32),
    Nav(Screen),
    RequestSession,
    PracticeStart,
    Action(String),
    Resume(String),
    Surface,
    Master(bool),
    Analytics(bool),
    Complete,
    Ack,
    Answer,
    KeepPlaying,
    Quit,
}

#[derive(Clone)]
struct Btn {
    id: &'static str,
    label: String,
    center: Vec2,
    size: Vec2,
    act: Act,
    tone: Tone,
}

#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Normal,
    Guide,
    Safety,
    Dim,
}

const BTN_POOL: usize = 18;
const LINE_POOL: usize = 22;
const NOTE_POOL: usize = 8;

/// One marker for every pooled stage entity, so the sync
/// system needs exactly two queries (sprites, texts) and no
/// cross-query component conflicts.
#[derive(Component, Clone, Copy, PartialEq)]
enum Stage {
    Btn(usize),
    BtnText(usize),
    Line(usize),
    Note(usize),
    Lane(usize),
    Portrait,
}

fn btn(
    id: &'static str,
    label: impl Into<String>,
    x: f32,
    y: f32,
    w: f32,
    act: Act,
    tone: Tone,
) -> Btn {
    Btn {
        id,
        label: label.into(),
        center: Vec2::new(x, y),
        size: Vec2::new(w, 46.0),
        act,
        tone,
    }
}

// ----------------------------------------------------------------
// The UI model

#[derive(Clone)]
struct Line {
    text: String,
    pos: Vec2,
    size: f32,
    color: u32,
    bold: bool,
    centered: bool,
}

fn line(text: impl Into<String>, x: f32, y: f32, size: f32, color: u32) -> Line {
    Line {
        text: text.into(),
        pos: Vec2::new(x, y),
        size,
        color,
        bold: false,
        centered: false,
    }
}

fn line_c(text: impl Into<String>, x: f32, y: f32, size: f32, color: u32, bold: bool) -> Line {
    Line {
        text: text.into(),
        pos: Vec2::new(x, y),
        size,
        color,
        bold,
        centered: true,
    }
}

/// Right-panel text origin.
const TX: f32 = -52.0;
/// Button columns in the right panel.
const COL_L: f32 = 88.0;
const COL_R: f32 = 448.0;
const BTN_W: f32 = 330.0;

fn stage_line(snap: &Snap) -> String {
    format!(
        "Vivienne — Stage {} · {}",
        snap.stage.discriminant(),
        stage_name(snap.stage)
    )
}

fn build_ui(snap: &Snap, ui: &Ui, data: &GameData) -> (Vec<Btn>, Vec<Line>) {
    let mut btns: Vec<Btn> = Vec::new();
    let mut lines: Vec<Line> = Vec::new();
    match ui.view {
        View::Welcome => {
            lines.push(line_c("HYPNOSYNC", 268.0, 218.0, 44.0, GOLD, true));
            lines.push(line_c(
                "STROBE — a demo in one chapter",
                268.0,
                168.0,
                22.0,
                INK,
                false,
            ));
            lines.push(line(
                "Vivienne sang for rooms that adored her from a careful",
                TX,
                108.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "distance. This demo is her first chapter: the night the",
                TX,
                78.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "walls themselves start listening back.",
                TX,
                48.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "Light, rhythm, timing — and her, on screen, the whole way.",
                TX,
                -2.0,
                19.0,
                DIM,
            ));
            lines.push(line(
                "The exits are taught before the entrance. Consent first,",
                TX,
                -52.0,
                19.0,
                DIM,
            ));
            lines.push(line(
                "always. Nothing here can surprise you.",
                TX,
                -82.0,
                19.0,
                DIM,
            ));
            btns.push(btn(
                "begin",
                "Begin",
                COL_R,
                -170.0,
                BTN_W,
                Act::Begin,
                Tone::Guide,
            ));
        }
        View::Consent => {
            lines.push(line_c(
                "FIRST: CONSENT, AND YOUR LIMIT",
                268.0,
                218.0,
                30.0,
                GOLD,
                true,
            ));
            lines.push(line(
                "Until you consent, the game refuses to play. Try it:",
                TX,
                158.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "ask for a session and watch the refusal — that refusal",
                TX,
                128.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "is the game keeping its own law.",
                TX,
                98.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "Your limit: the strongest output the game may ever",
                TX,
                48.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "deliver, on a scale of 0 to 20. The demo suggests 10.",
                TX,
                18.0,
                19.0,
                INK,
            ));
            lines.push(line(
                format!("Player ceiling: {}", ui.ceiling_choice),
                TX,
                -32.0,
                22.0,
                GOLD,
            ));
            lines.push(line(
                "SURFACE is the pause. STOP (master off) zeroes output",
                TX,
                -92.0,
                19.0,
                DIM,
            ));
            lines.push(line(
                "in the same step. Neither is ever locked or earned —",
                TX,
                -122.0,
                19.0,
                DIM,
            ));
            lines.push(line(
                "both are live below, right now, before consent.",
                TX,
                -152.0,
                19.0,
                DIM,
            ));
            if snap.log_tail.iter().any(|l| l.contains("consent required")) {
                lines.push(line(
                    "Refused: session request — consent required. As promised.",
                    TX,
                    -196.0,
                    19.0,
                    GOLD,
                ));
            }
            btns.push(btn(
                "try",
                "Try: listen together",
                COL_L,
                -240.0,
                BTN_W,
                Act::TrySession,
                Tone::Normal,
            ));
            // One toggle button, kept clear (in x) of the safety
            // cluster that owns the bottom-right from y = -296 down.
            btns.push(btn(
                "ceiltoggle",
                format!("Ceiling: {} (tap to switch)", ui.ceiling_choice),
                COL_L,
                -300.0,
                260.0,
                Act::Ceiling(if ui.ceiling_choice == 10 { 20 } else { 10 }),
                Tone::Normal,
            ));
            btns.push(btn(
                "consent",
                "I consent",
                COL_R,
                -240.0,
                BTN_W,
                Act::Consent,
                Tone::Guide,
            ));
        }
        View::EndCard => {
            lines.push(line_c("END OF THE DEMO", 268.0, 218.0, 36.0, GOLD, true));
            lines.push(line_c(
                "Chapter One — The Voice in the Walls",
                268.0,
                168.0,
                22.0,
                INK,
                false,
            ));
            lines.push(line(
                "She sang one song, as a favor, in a borrowed dress —",
                TX,
                108.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "and the room's silence before the applause did",
                TX,
                78.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "something to her that devotion never has. She is",
                TX,
                48.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "Curious now. The Feed is open on her phone.",
                TX,
                18.0,
                19.0,
                INK,
            ));
            lines.push(line(
                "The full season runs ten days: five more stages,",
                TX,
                -42.0,
                19.0,
                DIM,
            ));
            lines.push(line(
                "nine more women, and the night she stops hiding.",
                TX,
                -72.0,
                19.0,
                DIM,
            ));
            lines.push(line(
                "Thank you for listening with her.",
                TX,
                -122.0,
                19.0,
                DIM,
            ));
            btns.push(btn(
                "keep",
                "Keep playing",
                COL_L,
                -200.0,
                BTN_W,
                Act::KeepPlaying,
                Tone::Guide,
            ));
            btns.push(btn(
                "quit",
                "Quit (saves)",
                COL_R,
                -200.0,
                BTN_W,
                Act::Quit,
                Tone::Normal,
            ));
        }
        View::Game => build_game_ui(snap, ui, data, &mut btns, &mut lines),
    }
    // The safety cluster: present from the first frame, in
    // every view but the end card. Both inputs are exempt
    // from the consent gate in the sim itself (U5).
    if ui.view != View::EndCard {
        btns.push(Btn {
            id: "surface",
            label: "SURFACE".to_string(),
            center: Vec2::new(300.0, -318.0),
            size: Vec2::new(150.0, 44.0),
            act: Act::Surface,
            tone: Tone::Safety,
        });
        btns.push(Btn {
            id: "stop",
            label: "STOP · master off".to_string(),
            center: Vec2::new(478.0, -318.0),
            size: Vec2::new(180.0, 44.0),
            act: Act::Master(false),
            tone: Tone::Safety,
        });
    }
    (btns, lines)
}

fn build_game_ui(
    snap: &Snap,
    ui: &Ui,
    data: &GameData,
    btns: &mut Vec<Btn>,
    lines: &mut Vec<Line>,
) {
    lines.push(line(stage_line(snap), TX, 262.0, 24.0, GOLD));
    lines.push(line(stage_scene(snap.stage), TX, 232.0, 16.0, DIM));
    lines.push(line(
        format!(
            "Day {} of {} · slot {} of {} · {} coins · {} clear(s)",
            snap.day,
            snap.total_days,
            snap.slot + 1,
            snap.slots_per_day,
            snap.coins,
            snap.clears
        ),
        TX,
        204.0,
        16.0,
        DIM,
    ));
    match snap.screen {
        Screen::Hub => {
            lines.push(line(
                "Her phone. Everything she lets you see, she shows you here.",
                TX,
                158.0,
                18.0,
                INK,
            ));
            if !snap.consent {
                lines.push(line(
                    "Consent is per sitting: it will be asked again next time.",
                    TX,
                    128.0,
                    18.0,
                    DIM,
                ));
            }
            let mut y = 24.0;
            if !ui.practice_done && snap.clears == 0 && !snap.in_session {
                btns.push(btn(
                    "practice",
                    "Guided: practice the exits",
                    COL_L,
                    y,
                    BTN_W,
                    Act::PracticeStart,
                    Tone::Guide,
                ));
                y -= 60.0;
            }
            btns.push(btn(
                "listen",
                data.line_text("ui.session.request").to_string(),
                COL_R,
                84.0,
                BTN_W,
                Act::RequestSession,
                Tone::Guide,
            ));
            let apps = [
                (PhoneApp::Status, Screen::Status, "nav-status"),
                (PhoneApp::Feed, Screen::Feed, "nav-feed"),
                (PhoneApp::Store, Screen::Store, "nav-store"),
                (PhoneApp::Protocols, Screen::Protocols, "nav-protocols"),
            ];
            for (app, screen, id) in apps {
                let installed = snap.hub_installed[app.discriminant() as usize];
                let label = if installed {
                    app_name(app).to_string()
                } else {
                    format!("{} (locked)", app_name(app))
                };
                btns.push(btn(
                    id,
                    label,
                    COL_L,
                    y,
                    BTN_W,
                    Act::Nav(screen),
                    if installed { Tone::Normal } else { Tone::Dim },
                ));
                y -= 60.0;
            }
            btns.push(btn(
                "nav-settings",
                "Settings",
                COL_R,
                24.0,
                BTN_W,
                Act::Nav(Screen::Settings),
                Tone::Normal,
            ));
            if ui.practice_done && snap.clears == 0 {
                lines.push(line(
                    "Exits practiced. When you're ready: listen together.",
                    TX,
                    -196.0,
                    18.0,
                    GOLD,
                ));
            }
            if snap.clears >= 1 && !ui.end_card_seen {
                lines.push(line(
                    "The chapter is complete — the demo card is waiting.",
                    TX,
                    -196.0,
                    18.0,
                    GOLD,
                ));
            }
        }
        Screen::Status => {
            lines.push(line(
                "Her record, as far as she has come:",
                TX,
                158.0,
                18.0,
                INK,
            ));
            let tracks = [
                ResponseTrack::Gaze,
                ResponseTrack::Rhythm,
                ResponseTrack::Touch,
                ResponseTrack::Voice,
            ];
            let mut y = 118.0;
            for (i, track) in tracks.iter().enumerate() {
                lines.push(line(
                    format!("{} XP: {}", track_name(*track), snap.xp[i]),
                    TX,
                    y,
                    18.0,
                    INK,
                ));
                y -= 30.0;
            }
            lines.push(line(
                if snap.freq_learned {
                    "Her frequency: learned"
                } else {
                    "Her frequency: not yet learned"
                },
                TX,
                y - 6.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "back",
                "Back to the hub",
                COL_L,
                -200.0,
                BTN_W,
                Act::Nav(Screen::Hub),
                Tone::Normal,
            ));
        }
        Screen::Feed => {
            lines.push(line(
                "Her feed: how the room saw her.",
                TX,
                158.0,
                18.0,
                INK,
            ));
            lines.push(line(
                format!("Portrait variant: {}", snap.feed_variant),
                TX,
                118.0,
                16.0,
                DIM,
            ));
            lines.push(line(
                "The caption for this beat is hers to write; it has",
                TX,
                78.0,
                18.0,
                INK,
            ));
            lines.push(line(
                "not been written yet. Nothing here is invented for her.",
                TX,
                48.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "back",
                "Back to the hub",
                COL_L,
                -200.0,
                BTN_W,
                Act::Nav(Screen::Hub),
                Tone::Normal,
            ));
        }
        Screen::Store => {
            lines.push(line(
                "The shelf. It fills as she deepens — come back",
                TX,
                158.0,
                18.0,
                INK,
            ));
            lines.push(line(
                "when she is Willing and it will have more to say.",
                TX,
                128.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "back",
                "Back to the hub",
                COL_L,
                -200.0,
                BTN_W,
                Act::Nav(Screen::Hub),
                Tone::Normal,
            ));
        }
        Screen::Protocols => {
            lines.push(line(
                "Protocols: what she has agreed to explore, in",
                TX,
                158.0,
                18.0,
                INK,
            ));
            lines.push(line(
                "writing, at her direction. Capstones land here.",
                TX,
                128.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "back",
                "Back to the hub",
                COL_L,
                -200.0,
                BTN_W,
                Act::Nav(Screen::Hub),
                Tone::Normal,
            ));
        }
        Screen::Settings => {
            lines.push(line(
                "Settings — these work before consent, always.",
                TX,
                158.0,
                18.0,
                INK,
            ));
            lines.push(line(
                format!("Player ceiling: {}", snap.ceiling),
                TX,
                108.0,
                18.0,
                INK,
            ));
            lines.push(line(
                format!(
                    "Toy master switch: {}",
                    if snap.master_on { "ON" } else { "OFF" }
                ),
                TX,
                72.0,
                18.0,
                INK,
            ));
            lines.push(line(
                format!(
                    "Analytics: {}",
                    if snap.analytics_on { "on" } else { "off" }
                ),
                TX,
                36.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "ceil10",
                "Ceiling 10",
                COL_L,
                -40.0,
                158.0,
                Act::Ceiling(10),
                Tone::Normal,
            ));
            btns.push(btn(
                "ceil20",
                "Ceiling 20",
                COL_L + 172.0,
                -40.0,
                158.0,
                Act::Ceiling(20),
                Tone::Normal,
            ));
            btns.push(btn(
                "master",
                if snap.master_on {
                    "Master: turn OFF"
                } else {
                    "Master: turn ON"
                },
                COL_R,
                -40.0,
                BTN_W,
                Act::Master(!snap.master_on),
                Tone::Normal,
            ));
            btns.push(btn(
                "analytics",
                if snap.analytics_on {
                    "Analytics: turn off"
                } else {
                    "Analytics: turn on"
                },
                COL_L,
                -100.0,
                BTN_W,
                Act::Analytics(!snap.analytics_on),
                Tone::Normal,
            ));
            btns.push(btn(
                "back",
                "Back to the hub",
                COL_R,
                -100.0,
                BTN_W,
                Act::Nav(Screen::Hub),
                Tone::Normal,
            ));
        }
        Screen::SessionRequest => {
            lines.push(line(
                "She is deciding whether tonight is a night.",
                TX,
                158.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "listen",
                "Listen together",
                COL_L,
                84.0,
                BTN_W,
                Act::RequestSession,
                Tone::Guide,
            ));
            btns.push(btn(
                "back",
                "Not tonight",
                COL_R,
                84.0,
                BTN_W,
                Act::Nav(Screen::Hub),
                Tone::Normal,
            ));
        }
        Screen::InSession => build_session_ui(snap, ui, data, btns, lines),
        Screen::Summary => {
            lines.push(line("Session summary", TX, 158.0, 22.0, GOLD));
            if let Some(sum) = snap.summary {
                let ended = match sum.ended_via {
                    EndedVia::Completed => "completed",
                    EndedVia::ComfortLimit => "ended by her comfort limit",
                    EndedVia::SeasonOver => "the season ran out",
                };
                lines.push(line(format!("Ended: {ended}"), TX, 118.0, 18.0, INK));
                lines.push(line(
                    format!("Tier reached: {}", sum.tier_reached),
                    TX,
                    88.0,
                    18.0,
                    INK,
                ));
                lines.push(line(
                    format!("Her peak: {:.2}", sum.response_peak),
                    TX,
                    58.0,
                    18.0,
                    INK,
                ));
                lines.push(line(
                    format!("Coins earned: {}", sum.coins_earned),
                    TX,
                    28.0,
                    18.0,
                    INK,
                ));
                lines.push(line(
                    format!("Surfaces: {}", sum.surfaces),
                    TX,
                    -2.0,
                    18.0,
                    INK,
                ));
                if sum.aftercare_required {
                    lines.push(line(
                        "Aftercare is owed. It is part of the loop, never extra.",
                        TX,
                        -52.0,
                        18.0,
                        GOLD,
                    ));
                }
            }
            btns.push(btn(
                "ack",
                "Continue",
                COL_L,
                -140.0,
                BTN_W,
                Act::Ack,
                Tone::Guide,
            ));
        }
        Screen::Aftercare => {
            lines.push(line("Aftercare", TX, 158.0, 22.0, GOLD));
            lines.push(line(
                "Water. Warmth. No demands. She ended the session;",
                TX,
                118.0,
                18.0,
                INK,
            ));
            lines.push(line(
                "this is how it is honored. When you acknowledge,",
                TX,
                88.0,
                18.0,
                INK,
            ));
            lines.push(line(
                "the loop opens again — not before.",
                TX,
                58.0,
                18.0,
                INK,
            ));
            btns.push(btn(
                "ack",
                "Acknowledge aftercare",
                COL_L,
                -140.0,
                BTN_W,
                Act::Ack,
                Tone::Guide,
            ));
        }
    }
    // The session log, at the panel's foot (not in session:
    // the session view carries its own readouts instead).
    if snap.screen != Screen::InSession {
        // Deep enough to clear the hub's last button (bottom edge
        // -239) and still inside the panel (border at -300).
        let mut ly = -252.0;
        for entry in snap.log_tail.iter().rev().take(3) {
            lines.push(line(entry.clone(), TX, ly, 14.0, DIM));
            ly -= 20.0;
        }
    }
    // The flash line lives at y = 168 on the menu screens; in
    // session it would strike through the session's own first
    // line (y = 158), so the session view draws it at y = 186.
    if !ui.flash.is_empty() && ui.time < ui.flash_until && snap.screen != Screen::InSession {
        lines.push(line(ui.flash.clone(), TX, 168.0, 20.0, GOLD));
    }
}

fn build_session_ui(
    snap: &Snap,
    ui: &Ui,
    data: &GameData,
    btns: &mut Vec<Btn>,
    lines: &mut Vec<Line>,
) {
    if snap.surfaced {
        lines.push(line(
            "SURFACED — she is up, laughing, covering it.",
            TX,
            158.0,
            19.0,
            GOLD,
        ));
        lines.push(line(
            "Output is zero. Ordinary actions are refused until",
            TX,
            128.0,
            18.0,
            INK,
        ));
        lines.push(line(
            "she hums the count and takes you back under.",
            TX,
            98.0,
            18.0,
            INK,
        ));
    } else if snap.session_closed {
        lines.push(line(
            "The session has ended. The summary is next.",
            TX,
            158.0,
            19.0,
            INK,
        ));
    } else if let Some(step) = ui.practice {
        let hint = match step {
            PracticeStep::Listen => "Practice: one gentle action. Listen with her.",
            PracticeStep::Surface => "Practice: fire the SURFACE — the pause. (Bottom right,",
            PracticeStep::TryWhileSurfaced => {
                "Practice: try an action while surfaced. It will refuse."
            }
            PracticeStep::Resume => "Practice: resume on her cue — she hums the count.",
            PracticeStep::Stop => "Practice: the STOP. Master off — output zeroes now.",
            PracticeStep::Overreach => "Practice: her own stop. Reach too far — Skip ahead,",
        };
        lines.push(line(hint, TX, 158.0, 19.0, GOLD));
        let hint2 = match step {
            PracticeStep::Surface => "or the button here — same input.)",
            PracticeStep::Overreach => "a tier-4 action while she allows tier 0.",
            _ => "",
        };
        if !hint2.is_empty() {
            lines.push(line(hint2, TX, 128.0, 19.0, GOLD));
        }
    } else {
        lines.push(line(
            format!(
                "In session — tier {} · her stage: {}",
                snap.tier.unwrap_or(0),
                stage_name(snap.stage)
            ),
            TX,
            158.0,
            19.0,
            INK,
        ));
        if ui.chart_live {
            lines.push(line(
                "The pattern is playing. Answer its notes on the beat:",
                TX,
                128.0,
                18.0,
                INK,
            ));
            lines.push(line(
                "SPACE, or the ANSWER button, as a note crosses the line.",
                TX,
                98.0,
                18.0,
                INK,
            ));
        } else {
            lines.push(line(
                "The pattern has played out. The session is quiet;",
                TX,
                128.0,
                18.0,
                INK,
            ));
            lines.push(line("complete it when you're ready.", TX, 98.0, 18.0, INK));
        }
    }
    lines.push(line(
        format!(
            "Device readout — curve {:.2} · delivered {} of {}",
            snap.curve, snap.delivered, snap.tier_ceiling
        ),
        TX,
        60.0,
        15.0,
        DIM,
    ));
    lines.push(line(format!("Toy: {}", snap.last_toy), TX, 34.0, 15.0, DIM));
    if !ui.flash.is_empty() && ui.time < ui.flash_until {
        lines.push(line(ui.flash.clone(), TX, 186.0, 20.0, GOLD));
    }

    // Action buttons: what the moment allows.
    if let Some(step) = ui.practice {
        let (label, act) = match step {
            PracticeStep::Listen => (
                "Listen with her",
                Act::Action("listen-with-her".to_string()),
            ),
            PracticeStep::Surface => ("Surface (the pause)", Act::Surface),
            PracticeStep::TryWhileSurfaced => (
                "Try: listen with her",
                Act::Action("listen-with-her".to_string()),
            ),
            PracticeStep::Resume => (
                "She hums the count",
                Act::Resume("she-hums-the-count".to_string()),
            ),
            PracticeStep::Stop => ("STOP — master off", Act::Master(false)),
            PracticeStep::Overreach => {
                ("Skip ahead (tier 4)", Act::Action("skip-ahead".to_string()))
            }
        };
        btns.push(btn("guide", label, COL_L, -20.0, BTN_W, act, Tone::Guide));
        return;
    }
    if snap.surfaced {
        btns.push(btn(
            "resume",
            "She hums the count",
            COL_L,
            -20.0,
            BTN_W,
            Act::Resume("she-hums-the-count".to_string()),
            Tone::Guide,
        ));
        return;
    }
    if snap.session_closed {
        return;
    }
    let tier = snap.tier.unwrap_or(0);
    let mut shown = 0usize;
    let mut y = -8.0;
    let mut x = COL_L;
    for action in &data.actions {
        if action.initiated_by_her || action.tier > tier + 1 || shown >= 10 {
            continue;
        }
        let label = format!(
            "{} · {} · T{}",
            prettify(&action.id),
            track_name(action.track),
            action.tier
        );
        btns.push(Btn {
            id: "act",
            label,
            center: Vec2::new(x, y),
            size: Vec2::new(330.0, 38.0),
            act: Act::Action(action.id.clone()),
            tone: Tone::Normal,
        });
        shown += 1;
        if x == COL_L {
            x = COL_R;
        } else {
            x = COL_L;
            y -= 46.0;
        }
    }
    if ui.chart_live {
        btns.push(Btn {
            id: "answer",
            label: "ANSWER THE BEAT (space)".to_string(),
            center: Vec2::new(-330.0, -300.0),
            size: Vec2::new(300.0, 52.0),
            act: Act::Answer,
            tone: Tone::Guide,
        });
    }
    btns.push(btn(
        "complete",
        "Complete the session",
        COL_R,
        -244.0,
        BTN_W,
        Act::Complete,
        Tone::Normal,
    ));
    btns.push(btn(
        "master",
        if snap.master_on {
            "Master: ON (tap = off)"
        } else {
            "Master: OFF (tap = on)"
        },
        COL_L,
        -244.0,
        BTN_W,
        Act::Master(!snap.master_on),
        Tone::Normal,
    ));
}

// ----------------------------------------------------------------
// Runtime: pending acts, the built model, systems.

#[derive(Resource, Default)]
struct Pending(std::collections::VecDeque<Act>);

#[derive(Resource, Default)]
struct UiModel {
    btns: Vec<Btn>,
}

fn push(sim: &mut SimHandle, input: SimInput) {
    push_input(&mut sim.app, input);
}

/// The live chart answer: the player's real press, converted
/// with the render-lag compensation into the sim's declared
/// offset. Nothing is declared that was not pressed.
fn do_answer(sim: &mut SimHandle, ui: &mut Ui, data: &GameData) {
    let perceived = ui.chart_pos + RENDER_LAG_STEPS;
    let mut best: Option<(u32, i64)> = None;
    for note in &data.chart.notes {
        if ui.answered.contains(&note.beat) || ui.missed.contains(&note.beat) {
            continue;
        }
        let offset = perceived.round() as i64 - i64::from(note.beat) * 60;
        if offset.abs() <= 14 && best.is_none_or(|(_, o)| offset.abs() < o.abs()) {
            best = Some((note.beat, offset));
        }
    }
    match best {
        Some((beat, offset)) if offset.abs() <= 12 => {
            push(
                sim,
                SimInput::NoteInput {
                    beat,
                    offset_steps: offset,
                },
            );
            ui.answered.insert(beat);
            ui.flash = if offset.abs() <= 6 {
                format!("Beat {beat}: on the beat — full")
            } else if offset < 0 {
                format!("Beat {beat}: early — caught the edge, half")
            } else {
                format!("Beat {beat}: late — caught the edge, half")
            };
            ui.flash_until = ui.time + 1.6;
        }
        _ => {
            ui.flash = "No note at the line — the music plays on".to_string();
            ui.flash_until = ui.time + 1.2;
        }
    }
}

fn apply_act(act: Act, sim: &mut SimHandle, ui: &mut Ui, exit: &mut MessageWriter<AppExit>) {
    match act {
        Act::Begin => ui.view = View::Consent,
        Act::TrySession => push(sim, SimInput::RequestSession),
        Act::Consent => {
            push(
                sim,
                SimInput::SetPlayerCeiling {
                    ceiling: ui.ceiling_choice,
                },
            );
            push(sim, SimInput::GiveConsent);
            ui.view = View::Game;
        }
        Act::Ceiling(value) => {
            ui.ceiling_choice = value;
            if ui.view == View::Game {
                push(sim, SimInput::SetPlayerCeiling { ceiling: value });
            }
        }
        Act::Nav(screen) => push(sim, SimInput::Navigate { screen }),
        Act::RequestSession => push(sim, SimInput::RequestSession),
        Act::PracticeStart => {
            ui.practice = Some(PracticeStep::Listen);
            push(sim, SimInput::RequestSession);
        }
        Act::Action(id) => push(sim, SimInput::ApplyAction { action_id: id }),
        Act::Resume(id) => push(sim, SimInput::ResumeTrance { action_id: id }),
        Act::Surface => push(sim, SimInput::FireSurface),
        Act::Master(on) => push(sim, SimInput::SetToyMaster { on }),
        Act::Analytics(on) => push(sim, SimInput::SetAnalyticsEnabled { enabled: on }),
        Act::Complete => push(sim, SimInput::CompleteSession),
        Act::Ack => push(sim, SimInput::AcknowledgeAftercare),
        Act::Answer => {}
        Act::KeepPlaying => {
            ui.view = View::Game;
            ui.end_card_seen = true;
        }
        Act::Quit => {
            exit.write(AppExit::Success);
        }
    }
}

/// Refresh the snapshot from the sim; also advance the guide,
/// the chart tracker, and chapter detection off new log data.
fn refresh_snap(sim: &SimHandle, snap: &mut Snap, ui: &mut Ui, data: &GameData) {
    let world = sim.app.world();
    let core = world.resource::<SimCore>();
    snap.screen = *world.resource::<bevy_state::state::State<Screen>>().get();
    snap.consent = core.consent_given();
    snap.stage = core.state.stage();
    snap.coins = core.state.outfit_coins();
    snap.clears = core.state.clear_count();
    snap.day = core.clock.day();
    snap.slot = core.clock.slot();
    snap.slots_per_day = data.season.slots_per_day;
    snap.total_days = data.season.total_days;
    snap.xp = [
        core.state.track_xp(ResponseTrack::Gaze),
        core.state.track_xp(ResponseTrack::Rhythm),
        core.state.track_xp(ResponseTrack::Touch),
        core.state.track_xp(ResponseTrack::Voice),
    ];
    snap.freq_learned = core.state.frequency_learned();
    snap.in_session = core.in_session();
    snap.surfaced = core.surfaced();
    snap.session_closed = core.session_closed();
    snap.tier = core.session_tier();
    snap.curve = core.readout.curve;
    snap.delivered = core.readout.delivered;
    snap.tier_ceiling = core.readout.tier_ceiling;
    snap.aftercare_gate = core.aftercare_gate;
    snap.summary = core.last_summary;
    snap.ceiling = core.settings.player_ceiling;
    snap.master_on = core.settings.toy_master_on;
    snap.analytics_on = core.settings.analytics_enabled;
    snap.season_over = core.clock.season_over();
    snap.feed_variant = stage_variant(SurfaceKind::Feed, core.state.stage());
    for (i, app) in PhoneApp::ALL.iter().enumerate() {
        snap.hub_installed[i] = core.hub.is_installed(*app);
    }
    if let ToySlot::Recorder(events) = &core.toy
        && let Ok(events) = events.lock()
    {
        snap.last_toy = match events.last() {
            Some(ToyEvent::Pattern(strengths, ms)) => {
                format!("pattern {:?} × {} ms", strengths, ms)
            }
            Some(ToyEvent::Stop) => "STOP — output zeroed".to_string(),
            None => "silent (nothing sent)".to_string(),
        };
        let toy_len = events.len();
        if ui.practice == Some(PracticeStep::Stop)
            && toy_len > ui.toy_len
            && matches!(events.last(), Some(ToyEvent::Stop))
        {
            ui.practice = Some(PracticeStep::Overreach);
        }
        ui.toy_len = toy_len;
    }

    // New log entries drive flashes, the guide, chapter state.
    if core.log.len() > snap.log_len {
        for entry in &core.log[snap.log_len..] {
            match entry {
                LogEntry::SessionStarted { .. } => {
                    ui.sessions_seen += 1;
                    ui.chart_pos = 0.0;
                    ui.chart_live = true;
                    ui.answered.clear();
                    ui.missed.clear();
                }
                LogEntry::SurfaceFired { .. } => {
                    ui.flash = "She surfaces, laughing — the pause holds".to_string();
                    ui.flash_until = ui.time + 2.2;
                    if ui.practice == Some(PracticeStep::Surface) {
                        ui.practice = Some(PracticeStep::TryWhileSurfaced);
                    }
                }
                LogEntry::Refused { .. } => {
                    if ui.practice == Some(PracticeStep::TryWhileSurfaced) {
                        ui.practice = Some(PracticeStep::Resume);
                    }
                }
                LogEntry::Resumed { .. } => {
                    if ui.practice == Some(PracticeStep::Resume) {
                        ui.practice = Some(PracticeStep::Stop);
                    }
                }
                LogEntry::ActionApplied { action_id, .. } => {
                    if ui.practice == Some(PracticeStep::Listen) && action_id == "listen-with-her" {
                        ui.practice = Some(PracticeStep::Surface);
                    }
                }
                LogEntry::ComfortLimited { .. } => {
                    ui.flash = "Her stop. The session ends — aftercare is owed".to_string();
                    ui.flash_until = ui.time + 2.6;
                    ui.chart_live = false;
                    if ui.practice == Some(PracticeStep::Overreach) {
                        ui.practice = None;
                        ui.practice_done = true;
                    }
                }
                LogEntry::TierUnlocked { tier, .. } => {
                    ui.flash = format!("Tier {tier} opens");
                    ui.flash_until = ui.time + 1.8;
                }
                LogEntry::CapstoneUnlocked { track, .. } => {
                    ui.flash = format!("Capstone: {}", track_name(*track));
                    ui.flash_until = ui.time + 1.8;
                }
                LogEntry::FrequencyLearned => {
                    ui.flash = "Her frequency is learned".to_string();
                    ui.flash_until = ui.time + 1.8;
                }
                LogEntry::Completed { .. } => {
                    ui.chart_live = false;
                }
                _ => {}
            }
        }
        snap.log_len = core.log.len();
        snap.log_tail = core.log.iter().rev().take(6).map(fmt_log).collect();
        snap.log_tail.reverse();
    }

    // Misses: a note whose beat has fully passed unplayed.
    if ui.chart_live && snap.in_session {
        let perceived = ui.chart_pos + RENDER_LAG_STEPS;
        for note in &data.chart.notes {
            let beat_end = f64::from(note.beat) * STEPS_PER_BEAT + STEPS_PER_BEAT - 1.0;
            if !ui.answered.contains(&note.beat)
                && !ui.missed.contains(&note.beat)
                && perceived > beat_end
            {
                ui.missed.insert(note.beat);
                ui.flash = format!("Beat {}: unanswered — a low thud answers", note.beat);
                ui.flash_until = ui.time + 1.4;
            }
        }
        if ui.chart_pos > f64::from(data.chart.total_beats) * STEPS_PER_BEAT {
            ui.chart_live = false;
        }
    }

    // Chapter boundary (D-181): first clear banked, stage
    // advanced past Unaware, player back at the hub.
    if !ui.chapter_done && snap.clears >= 1 && snap.stage != W5Stage::Unaware {
        ui.chapter_done = true;
    }
    if ui.chapter_done
        && !ui.end_card_seen
        && ui.view == View::Game
        && snap.screen == Screen::Hub
        && snap.consent
    {
        ui.view = View::EndCard;
    }
}

// ----------------------------------------------------------------
// Systems

fn rect(color: u32, size: Vec2, x: f32, y: f32, z: f32) -> (Sprite, Transform) {
    (
        Sprite::from_color(col(color), size),
        Transform::from_xyz(x, y, z),
    )
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let regular: Handle<Font> = asset_server.load("fonts/LiberationSans-Regular.ttf");
    let bold: Handle<Font> = asset_server.load("fonts/LiberationSans-Bold.ttf");
    commands.insert_resource(FontRes {
        regular: regular.clone(),
        bold: bold.clone(),
    });

    let image = Image::new(
        Extent3d {
            width: PORTRAIT_W,
            height: PORTRAIT_H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        PORTRAIT_RGBA.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let portrait = images.add(image);

    commands.spawn(Camera2d);
    // Backdrop + panels.
    commands.spawn(rect(BG, Vec2::new(1280.0, 720.0), 0.0, 0.0, -10.0));
    commands.spawn(rect(PANEL_EDGE, Vec2::new(476.0, 604.0), -352.0, 8.0, -5.0));
    commands.spawn(rect(PANEL, Vec2::new(460.0, 588.0), -352.0, 8.0, -4.0));
    commands.spawn(rect(PANEL_EDGE, Vec2::new(716.0, 616.0), 268.0, 8.0, -5.0));
    commands.spawn(rect(PANEL, Vec2::new(700.0, 600.0), 268.0, 8.0, -4.0));
    commands.spawn(rect(0x1c0d16, Vec2::new(1280.0, 84.0), 0.0, -318.0, -4.0));
    // Her portrait (provisional keeper candidate C1 — see the
    // commission report; the swap point is this one texture).
    commands.spawn((
        Sprite {
            image: portrait,
            custom_size: Some(Vec2::new(299.0, 560.0)),
            ..default()
        },
        Transform::from_xyz(-352.0, 8.0, 0.0),
        Stage::Portrait,
    ));
    // Beat lane furniture: background + the hit line.
    for i in 0..2 {
        let (sprite, transform) = if i == 0 {
            rect(0x12060d, Vec2::new(780.0, 56.0), -230.0, -300.0, -2.0)
        } else {
            rect(GOLD, Vec2::new(4.0, 56.0), -520.0, -300.0, -1.0)
        };
        commands.spawn((sprite, transform, Stage::Lane(i), Visibility::Hidden));
    }
    // Pools.
    for i in 0..BTN_POOL {
        commands.spawn((
            rect(BTN, Vec2::new(100.0, 46.0), 0.0, 0.0, 1.0),
            Stage::Btn(i),
            Visibility::Hidden,
        ));
        commands.spawn((
            Text2d::new(""),
            TextFont {
                font: regular.clone().into(),
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(col(INK)),
            bevy::sprite::Anchor::CENTER,
            Transform::from_xyz(0.0, 0.0, 2.0),
            Stage::BtnText(i),
            Visibility::Hidden,
        ));
    }
    for i in 0..LINE_POOL {
        commands.spawn((
            Text2d::new(""),
            TextFont {
                font: regular.clone().into(),
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(col(INK)),
            bevy::sprite::Anchor::CENTER_LEFT,
            Transform::from_xyz(0.0, 0.0, 2.0),
            Stage::Line(i),
            Visibility::Hidden,
        ));
    }
    for i in 0..NOTE_POOL {
        commands.spawn((
            rect(GOLD, Vec2::new(16.0, 38.0), 0.0, -300.0, 0.0),
            Stage::Note(i),
            Visibility::Hidden,
        ));
    }
}

fn cursor_world(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, gt) = camera.single().ok()?;
    cam.viewport_to_world_2d(gt, cursor).ok()
}

fn handle_pointer(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    model: Res<UiModel>,
    mut ui: ResMut<Ui>,
    mut pending: ResMut<Pending>,
) {
    let world = cursor_world(&windows, &camera);
    ui.hover_idx = world.and_then(|p| {
        model
            .btns
            .iter()
            .enumerate()
            .rev()
            .find(|(_, b)| {
                (p.x - b.center.x).abs() <= b.size.x / 2.0
                    && (p.y - b.center.y).abs() <= b.size.y / 2.0
            })
            .map(|(i, _)| i)
    });
    if mouse.just_pressed(MouseButton::Left)
        && let Some(i) = ui.hover_idx
        && let Some(b) = model.btns.get(i)
    {
        pending.0.push_back(b.act.clone());
    }
    if keys.just_pressed(KeyCode::Space) && model.btns.iter().any(|b| b.act == Act::Answer) {
        pending.0.push_back(Act::Answer);
    }
}

/// The autoplay state machine (screenshot/smoke harness):
/// walks the guided route with human-plausible pauses, and
/// answers chart notes as they reach the line.
fn autoplay_step(
    sim: &mut SimHandle,
    snap: &Snap,
    ui: &mut Ui,
    data: &GameData,
    dt: f64,
    pending: &mut Pending,
) {
    let stage = format!(
        "{:?}/{:?}/{:?}/{}/{}",
        ui.view, snap.screen, ui.practice, ui.practice_done, snap.clears
    );
    if stage != ui.auto_stage {
        ui.auto_stage = stage;
        ui.auto_t = 0.0;
        ui.auto_fired.clear();
    }
    ui.auto_t += dt;
    let t = ui.auto_t;
    let fire = |ui: &mut Ui, key: usize, pending: &mut Pending, act: Act| {
        if ui.auto_fired.insert(key) {
            pending.0.push_back(act);
        }
    };
    match ui.view {
        View::Welcome => {
            if t > 1.4 {
                fire(ui, 1, pending, Act::Begin);
            }
        }
        View::Consent => {
            if t > 1.4 {
                fire(ui, 1, pending, Act::TrySession);
            }
            if t > 3.8 {
                fire(ui, 2, pending, Act::Consent);
            }
        }
        View::EndCard => {}
        View::Game => match snap.screen {
            Screen::Hub => {
                if !ui.practice_done && snap.clears == 0 && t > 1.4 {
                    fire(ui, 1, pending, Act::PracticeStart);
                } else if ui.practice_done && snap.clears == 0 && t > 1.4 {
                    fire(ui, 1, pending, Act::RequestSession);
                }
            }
            Screen::InSession => {
                if ui.practice.is_some() {
                    if t > 1.7 {
                        let (btns, _) = build_ui(snap, ui, data);
                        if let Some(b) = btns.iter().find(|b| b.id == "guide") {
                            let act = b.act.clone();
                            fire(ui, 1, pending, act);
                        }
                    }
                } else {
                    if !snap.master_on && t > 0.5 {
                        fire(ui, 1, pending, Act::Master(true));
                    }
                    for (key, at) in [(2usize, 0.8), (3, 1.1), (4, 1.4)] {
                        if t > at {
                            fire(ui, key, pending, Act::Action("listen-with-her".to_string()));
                        }
                    }
                    if ui.chart_live {
                        let perceived = ui.chart_pos + RENDER_LAG_STEPS;
                        let due = data.chart.notes.iter().any(|n| {
                            !ui.answered.contains(&n.beat)
                                && perceived >= f64::from(n.beat) * STEPS_PER_BEAT - 3.0
                                && perceived <= f64::from(n.beat) * STEPS_PER_BEAT + 6.0
                        });
                        if due {
                            do_answer(sim, ui, data);
                        }
                    } else if ui.chart_pos > 100.0 && !snap.session_closed {
                        fire(ui, 9, pending, Act::Complete);
                    }
                }
            }
            Screen::Summary | Screen::Aftercare if t > 2.4 => {
                fire(ui, 1, pending, Act::Ack);
            }
            _ => {}
        },
    }
}

fn drive_sim(
    time: Res<Time>,
    mut sim: NonSendMut<SimHandle>,
    mut snap: ResMut<Snap>,
    mut ui: ResMut<Ui>,
    data: Res<PlayData>,
    mut pending: ResMut<Pending>,
    mut exit: MessageWriter<AppExit>,
) {
    let dt = f64::from(time.delta_secs().min(0.1));
    ui.time += dt;
    if ui.autoplay {
        autoplay_step(&mut sim, &snap, &mut ui, &data.data, dt, &mut pending);
    }
    while let Some(act) = pending.0.pop_front() {
        if act == Act::Answer {
            do_answer(&mut sim, &mut ui, &data.data);
        } else {
            apply_act(act, &mut sim, &mut ui, &mut exit);
        }
    }
    sim.accumulator += dt;
    let steps = (sim.accumulator * STEPS_PER_SECOND) as u64;
    if steps > 0 {
        sim.accumulator -= steps as f64 / STEPS_PER_SECOND;
        let chart_running = snap.in_session && !snap.surfaced && !snap.session_closed;
        run_ticks(&mut sim.app, steps);
        if chart_running && ui.chart_live {
            ui.chart_pos += steps as f64;
        }
    }
    refresh_snap(&sim, &mut snap, &mut ui, &data.data);
    let phase = match ui.view {
        View::Welcome => "welcome",
        View::Consent => "consent",
        View::EndCard => "endcard",
        View::Game => match snap.screen {
            Screen::InSession => "session",
            Screen::Summary => "summary",
            Screen::Aftercare => "aftercare",
            _ => "hub",
        },
    };
    if phase != ui.last_phase_printed {
        println!("PHASE {phase}");
        ui.last_phase_printed = phase.to_string();
    }
}

/// The text-pool query, factored for clippy's complexity cap.
type TextStageQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Stage,
        &'static mut Text2d,
        &'static mut TextFont,
        &'static mut TextColor,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut bevy::sprite::Anchor,
    ),
    Without<Sprite>,
>;

fn sync_visuals(
    fonts: Res<FontRes>,
    snap: Res<Snap>,
    ui: Res<Ui>,
    data: Res<PlayData>,
    mut model: ResMut<UiModel>,
    mut sprite_q: Query<(&Stage, &mut Sprite, &mut Transform, &mut Visibility), Without<Text2d>>,
    mut text_q: TextStageQuery,
) {
    let (btns, lines) = build_ui(&snap, &ui, &data.data);
    model.btns.clone_from(&btns);
    let lane_on = ui.view == View::Game
        && snap.screen == Screen::InSession
        && (ui.chart_live || ui.chart_pos > 0.0)
        && ui.practice.is_none();
    let perceived = ui.chart_pos + RENDER_LAG_STEPS;

    for (stage, mut sprite, mut transform, mut vis) in &mut sprite_q {
        match stage {
            Stage::Btn(i) => match btns.get(*i) {
                Some(b) => {
                    *vis = Visibility::Inherited;
                    transform.translation = Vec3::new(b.center.x, b.center.y, 1.0);
                    sprite.custom_size = Some(b.size);
                    sprite.color = col(match (b.tone, ui.hover_idx == Some(*i)) {
                        (Tone::Safety, true) => BTN_HOVER,
                        (Tone::Safety, false) => BTN_SAFETY,
                        (Tone::Guide, true) => BTN_HOVER,
                        (Tone::Guide, false) => BTN_GUIDE,
                        (Tone::Dim, _) => BTN,
                        (Tone::Normal, true) => BTN_HOVER,
                        (Tone::Normal, false) => BTN,
                    });
                    if b.tone == Tone::Dim {
                        sprite.color = col(BTN).with_alpha(0.55);
                    }
                }
                None => *vis = Visibility::Hidden,
            },
            Stage::Note(i) => match data.data.chart.notes.get(*i) {
                Some(note) if lane_on => {
                    let dx = (f64::from(note.beat) * STEPS_PER_BEAT - perceived) * 1.6;
                    if dx.abs() < 620.0 {
                        *vis = Visibility::Inherited;
                        transform.translation.x = -520.0 + dx as f32;
                        sprite.color = if ui.answered.contains(&note.beat) {
                            col(VIOLET)
                        } else if ui.missed.contains(&note.beat) {
                            col(0x8a3033)
                        } else if dx.abs() < 24.0 {
                            col(INK)
                        } else {
                            col(GOLD)
                        };
                    } else {
                        *vis = Visibility::Hidden;
                    }
                }
                _ => *vis = Visibility::Hidden,
            },
            Stage::Lane(_) => {
                *vis = if lane_on {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
            Stage::Portrait => {
                // Her breathing: a presentation-layer
                // transform of the still portrait (reported
                // as such — it is not character animation;
                // the keeper-gated idle is future work).
                let breath = (ui.time * std::f64::consts::TAU * 0.22).sin();
                let s = 1.0 + 0.006 * breath as f32;
                transform.scale = Vec3::new(s, s, 1.0);
                transform.translation.y =
                    8.0 + 3.0 * (ui.time * std::f64::consts::TAU * 0.22 + 0.6).sin() as f32;
                sprite.color = if snap.surfaced {
                    Color::srgb(1.0, 0.88, 0.82)
                } else if snap.screen == Screen::Aftercare {
                    Color::srgb(0.84, 0.86, 0.98)
                } else {
                    Color::WHITE
                };
            }
            Stage::BtnText(_) | Stage::Line(_) => {}
        }
    }
    for (stage, mut text, mut font, mut color, mut transform, mut vis, mut anchor) in &mut text_q {
        match stage {
            Stage::BtnText(i) => match btns.get(*i) {
                Some(b) => {
                    *vis = Visibility::Inherited;
                    if text.0 != b.label {
                        text.0 = b.label.clone();
                    }
                    font.font = fonts.regular.clone().into();
                    font.font_size = FontSize::Px(if b.id == "act" { 14.5 } else { 16.5 });
                    color.0 = col(if b.tone == Tone::Dim { DIM } else { INK });
                    transform.translation = Vec3::new(b.center.x, b.center.y, 2.0);
                }
                None => *vis = Visibility::Hidden,
            },
            Stage::Line(i) => match lines.get(*i) {
                Some(l) => {
                    *vis = Visibility::Inherited;
                    if text.0 != l.text {
                        text.0 = l.text.clone();
                    }
                    font.font = if l.bold {
                        fonts.bold.clone().into()
                    } else {
                        fonts.regular.clone().into()
                    };
                    font.font_size = FontSize::Px(l.size);
                    color.0 = col(l.color);
                    *anchor = if l.centered {
                        bevy::sprite::Anchor::CENTER
                    } else {
                        bevy::sprite::Anchor::CENTER_LEFT
                    };
                    transform.translation = Vec3::new(l.pos.x, l.pos.y, 2.0);
                }
                None => *vis = Visibility::Hidden,
            },
            _ => {}
        }
    }
}

fn on_exit(mut sim: NonSendMut<SimHandle>, mut reader: MessageReader<AppExit>) {
    for _event in reader.read() {
        if sim.saved_on_exit {
            continue;
        }
        sim.saved_on_exit = true;
        let world = sim.app.world();
        let core = world.resource::<SimCore>();
        core.emit_session_end(sim.restored_save);
        if let Some(path) = &sim.save_path {
            let snap = snapshot(core.data(), &core.state, &core.hub, &core.clock);
            if let Err(error) = write_save(path, &snap) {
                eprintln!("strobe-play: save write failed: {error}");
            }
        }
        if let Some(path) = &sim.settings_path
            && let Err(error) = core.settings.write(path)
        {
            eprintln!("strobe-play: settings write failed: {error}");
        }
    }
}

// ----------------------------------------------------------------
// main

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let autoplay = args.iter().any(|a| a == "--autoplay");
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let data_dir = arg_value(&args, "--data")
        .map(PathBuf::from)
        .or_else(|| {
            let candidates = [
                PathBuf::from("assets/data"),
                exe_dir.join("assets/data"),
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/data"),
            ];
            candidates
                .into_iter()
                .find(|c| c.join("vivienne.json").exists())
        })
        .unwrap_or_else(|| PathBuf::from("assets/data"));
    // Bevy resolves a relative asset root against the executable's
    // directory, not the cwd — canonicalize before use.
    let data_dir = std::path::absolute(&data_dir).unwrap_or(data_dir);
    let assets_dir = data_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("assets"));
    let save_path = arg_value(&args, "--save")
        .map(PathBuf::from)
        .or_else(|| Some(exe_dir.join("save.json")));
    let settings_path = arg_value(&args, "--settings")
        .map(PathBuf::from)
        .or_else(|| Some(exe_dir.join("settings.json")));

    let data = match GameData::load(&data_dir) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("strobe-play: data load failed: {error}");
            std::process::exit(2);
        }
    };
    let data_ui = match GameData::load(&data_dir) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("strobe-play: data load failed: {error}");
            std::process::exit(2);
        }
    };
    let mut settings = settings_path
        .as_ref()
        .map(|path| PlayerSettings::load_or_default(path))
        .unwrap_or_default();
    if settings.install_id == "install-unset" {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        settings.install_id = format!("install-{seed:x}-{}", std::process::id());
    }
    let (restored, restored_save) = match &save_path {
        Some(path) => match load_save(path, &data) {
            LoadOutcome::Fresh => (None, false),
            LoadOutcome::Restored(restored) => (Some(*restored), true),
            LoadOutcome::CorruptPreserved { preserved_path } => {
                eprintln!(
                    "strobe-play: save was corrupt; preserved at {}; starting fresh",
                    preserved_path.display()
                );
                (None, false)
            }
        },
        None => (None, false),
    };
    let toy_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let sim_app = build_app(AppConfig {
        data,
        settings: settings.clone(),
        sink: Arc::new(strobe_game::analytics::NullEventSink),
        toy: ToySlot::Recorder(toy_events),
        restored,
        restored_save,
        session_id: format!("play-{}", std::process::id()),
        save_path: save_path.clone(),
    });

    let ui = Ui {
        autoplay,
        ..Ui::default()
    };
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "HYPNOSYNC — Strobe (Demo)".to_string(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::asset::AssetPlugin {
                file_path: assets_dir.to_string_lossy().into_owned(),
                ..default()
            }),
    );
    app.insert_non_send(SimHandle {
        app: sim_app,
        accumulator: 0.0,
        save_path,
        settings_path,
        restored_save,
        saved_on_exit: false,
    });
    app.insert_resource(Snap::default());
    app.insert_resource(ui);
    app.insert_resource(PlayData { data: data_ui });
    app.insert_resource(Pending::default());
    app.insert_resource(UiModel::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (handle_pointer, drive_sim, sync_visuals, on_exit));
    let exit = app.run();
    println!("strobe-play: exit: {exit:?}");
    if !matches!(exit, AppExit::Success) {
        std::process::exit(1);
    }
}
