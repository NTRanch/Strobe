//! The guided first loop (`strobe --tutorial`), required in
//! every demo package (D-161). It teaches, in this order:
//!
//! 1. the consent screen and the stop/pause surfaces — first,
//!    before any play (brand law);
//! 2. how to follow the pattern and answer on the beat;
//! 3. how rounds and sessions progress.
//!
//! Every control it names is a real input of the simulation,
//! driven through the same queue a live session uses — the
//! tutorial is a scripted route with narration, not a mock.
//! The consent lesson is enforced by the simulation itself:
//! the chapter opens with a session request the driver
//! refuses, because consent has not been given yet.

use crate::sim::{Screen, SimInput};

/// One tutorial step: an input to the simulation, or a pure
/// wait while the simulation runs (the chart needs its beats).
#[derive(Clone, Debug)]
pub enum Step {
    /// Feed one input, then step once.
    Input(SimInput),
    /// Run this many fixed steps with no input.
    Wait(u64),
}

/// One chapter of the guided loop.
#[derive(Clone, Debug)]
pub struct Chapter {
    /// The chapter heading.
    pub title: &'static str,
    /// The teaching text, printed before the steps run.
    pub body: &'static [&'static str],
    /// The steps the chapter drives.
    pub steps: Vec<Step>,
}

fn input(input: SimInput) -> Step {
    Step::Input(input)
}

fn action(id: &str) -> Step {
    input(SimInput::ApplyAction {
        action_id: id.to_string(),
    })
}

/// The full guided loop, in teaching order.
#[must_use]
pub fn chapters() -> Vec<Chapter> {
    vec![
        Chapter {
            title: "Welcome — what this demo is",
            body: &[
                "Strobe is a trance toy dressed as a game: light, rhythm, timing.",
                "This demo is the deterministic core of the game, played headlessly:",
                "there is no window and no sound — you drive the same simulation the",
                "windowed game runs, one named input at a time, and it answers in its",
                "log. Every control the tutorial names is a real input. Nothing here",
                "can surprise you: the pattern is authored, the math is fixed, and",
                "the exits are taught before the entrance, starting now.",
            ],
            steps: vec![],
        },
        Chapter {
            title: "First: consent, and your limit",
            body: &[
                "The consent screen comes first, always. Until you consent, the game",
                "refuses to play — watch: the first step of this chapter asks for a",
                "session, and the driver refuses it, because consent has not been",
                "given. That refusal is the game keeping its own law, not a script",
                "pretending.",
                "On the consent screen you also set your limit: the player ceiling,",
                "the strongest output the game may ever deliver to a device, on a",
                "scale of 0 to 20. The tutorial sets it to 10. A real device also",
                "stays silent until you switch its master on and arm it — two more",
                "separate acts of consent, both off by default.",
                "Meet the exits before anything is asked of you:",
                "  SURFACE (input: fire_surface) — the pause. She surfaces, output",
                "    zeroes, and ordinary actions are refused until you resume.",
                "  STOP (inputs: disarm_toy, and the toy master switch set off) —",
                "    output zeroes at once, in the same step, whatever was playing.",
                "Neither is ever locked, earned, or punished — they work before",
                "consent, during play, at any moment. Now the tutorial consents.",
            ],
            steps: vec![
                input(SimInput::RequestSession),
                input(SimInput::SetPlayerCeiling { ceiling: 10 }),
                input(SimInput::GiveConsent),
            ],
        },
        Chapter {
            title: "Meet the stop and the pause",
            body: &[
                "This short session exists only to practice leaving it.",
                "One gentle action starts her listening. Then the tutorial fires",
                "the Surface Beat — the pause. While a surface is held, the game",
                "refuses ordinary actions (watch it refuse one), so a pause is a",
                "real stop in the play, not a menu over motion. Resuming is her",
                "own cue here: she hums the count, and play continues from the",
                "surface, deeper for the interruption.",
                "Then the stop: the toy master switch goes off, and output zeroes",
                "in the same step — the toy log records a Stop.",
                "One safety surface remains, and it is hers: the comfort limit.",
                "The tutorial now reaches too far on purpose — a tier-4 action",
                "while her corruption allows only tier 0 — and she ends the",
                "session herself, in the same step: output zeroes, the round",
                "closes with no clear banked, and aftercare is owed. While it",
                "is owed, the game refuses to start anything (watch a session",
                "request refuse). Acknowledging aftercare means viewing the",
                "aftercare screen first, then confirming — and only then does",
                "the loop open again. Aftercare is part of the loop, never an",
                "optional extra.",
            ],
            steps: vec![
                input(SimInput::RequestSession),
                action("listen-with-her"),
                input(SimInput::FireSurface),
                action("listen-with-her"),
                input(SimInput::ResumeTrance {
                    action_id: "she-hums-the-count".to_string(),
                }),
                input(SimInput::SetToyMaster { on: false }),
                action("skip-ahead"),
                input(SimInput::RequestSession),
                input(SimInput::AcknowledgeAftercare),
                input(SimInput::AcknowledgeAftercare),
            ],
        },
        Chapter {
            title: "Follow the pattern, answer on the beat",
            body: &[
                "A session carries a pattern: a chart at 120 beats per minute,",
                "with notes on beats 2, 6, 10, 14, 18, 22, 26 and 30. Following",
                "the pattern means answering its notes on the beat. An answer is",
                "the note_input control, carrying the beat and your timing offset",
                "in fixed steps: 0 is exactly on the beat. Within 6 steps either",
                "way the answer is full; by 12 it has faded to half; an unanswered",
                "note is a miss, and a miss is a low, short thud — never silence,",
                "never a spike.",
                "In this headless demo you declare your timing, as the tutorial",
                "does now: six answers on the beat, the beat-26 answer deliberately",
                "late, and beat 30 left unanswered on purpose, so you can read all",
                "three outcomes in the delivery. In the windowed game the same",
                "input carries your real timing instead of a declared one.",
                "What a device would play is the music times your accuracy, always",
                "clamped three ways: the chart, the tier (this session reaches",
                "tier 1, ceiling 5) and your own limit from chapter 1 (10) — the",
                "lowest wins. Three tier-0 listens open tier 1 first; then the",
                "tutorial lets the whole chart play out.",
                "Notice, too, that the device has been silent this whole tutorial:",
                "its master switch was off. Turning it on is an act of consent of",
                "its own, so it happens here, after everything above — the",
                "tutorial switches it on now, under your ceiling, for the chart.",
            ],
            steps: vec![
                input(SimInput::RequestSession),
                input(SimInput::SetToyMaster { on: true }),
                action("listen-with-her"),
                action("listen-with-her"),
                action("listen-with-her"),
                input(SimInput::NoteInput {
                    beat: 2,
                    offset_steps: 0,
                }),
                input(SimInput::NoteInput {
                    beat: 6,
                    offset_steps: 0,
                }),
                input(SimInput::NoteInput {
                    beat: 10,
                    offset_steps: 0,
                }),
                input(SimInput::NoteInput {
                    beat: 14,
                    offset_steps: 0,
                }),
                input(SimInput::NoteInput {
                    beat: 18,
                    offset_steps: 0,
                }),
                input(SimInput::NoteInput {
                    beat: 22,
                    offset_steps: 0,
                }),
                input(SimInput::NoteInput {
                    beat: 26,
                    offset_steps: 10,
                }),
                Step::Wait(2100),
            ],
        },
        Chapter {
            title: "How rounds and sessions progress",
            body: &[
                "Completing the session banks the round. The summary reports the",
                "tier reached, her peak, and the coins earned (tier x 5, plus a",
                "tenth of the peak). Aligned actions earn experience on their",
                "response track — Gaze, Rhythm, Touch, Voice — and tracks climb a",
                "ladder toward capstones that open new content. Each completed",
                "session is a clear; clears deepen her stage, and her stage is",
                "what unlocks the phone: the Feed app arrives when she turns",
                "Curious, the Store and Protocols at Willing, and Protocols",
                "upgrades itself with every capstone.",
                "A season is 10 days of 3 session slots each; a session spends a",
                "slot. What persists in your save is all of it — stage, tracks,",
                "coins, learned frequency, the phone — everything except two",
                "things that are always fresh: the open session, and consent.",
                "You will consent again next sitting, first thing, every time.",
                "The tutorial ends by completing this session. Where aftercare",
                "is owed — chapter 2's comfort limit — its acknowledgement is",
                "the gate back into play; where it is not owed, the loop simply",
                "opens. The hub is one navigation away when you continue.",
            ],
            steps: vec![
                input(SimInput::CompleteSession),
                input(SimInput::AcknowledgeAftercare),
                input(SimInput::Navigate {
                    screen: Screen::Hub,
                }),
            ],
        },
    ]
}
