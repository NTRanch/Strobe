//! The chart player (SPEC-BEVY G6-B): the music layer is king.
//! The player's job is to *confirm* the music, never to generate
//! it. Per fixed step the player produces the D6 curve value for
//! `ToyCurve`: the music layer runs whether or not notes are hit;
//! a judgement sharpens it by accuracy; a miss is never silence —
//! it produces one short, dull, low thud burst, because absent
//! output teaches nothing.
//!
//! Delivery discipline (§4 B3): output leaves as pattern bursts,
//! never per-hit function calls. The player samples the delivered
//! intensity every 15 steps (125 ms) and emits one pattern per two
//! beats (8 strengths × 125 ms = exactly the 1 s pattern floor).

use crate::data::{ChartData, ChartNote};

/// Fixed steps per second of simulation (the estate fixed dt).
pub const STEPS_PER_SECOND: u64 = 120;
/// Steps per beat at the chart's 120 BPM (500 ms).
pub const STEPS_PER_BEAT: u64 = 60;
/// Pattern sampling stride in steps (125 ms — above the 100 ms
/// pattern-step refusal).
pub const SAMPLE_STRIDE_STEPS: u64 = 15;
/// Strengths per emitted pattern (two beats of samples).
pub const PATTERN_STRENGTHS: usize = 8;
/// The thud's curve value: short, dull, low — and never zero.
pub const THUD_CURVE: f64 = 0.25;
/// How long a thud burst lasts, in steps (100 ms).
pub const THUD_STEPS: u64 = 12;
/// The presence factor the music layer holds between judgements:
/// the music is always there under the pattern.
pub const PRESENCE_FACTOR: f64 = 0.6;

/// A judged note's outcome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Judgement {
    /// The note was hit with this accuracy factor, 0..=1.
    Hit(f64),
    /// The note's beat passed unplayed: a thud.
    Miss,
}

/// The player's per-step output: the D6 curve value and whether a
/// judgement resolved on this step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartStep {
    /// The combined curve value for `ToyCurve`, 0..=1.
    pub curve: f64,
    /// A judgement that resolved exactly on this step, if any.
    pub judgement: Option<Judgement>,
}

/// Live chart-player state (a plain struct; the sim owns one while
/// a session is active).
pub struct ChartPlayer {
    step_in_chart: u64,
    total_steps: u64,
    inputs: Vec<(u64, i64)>,
    resolved: Vec<bool>,
    thud_until: u64,
    samples: Vec<i32>,
    pending_pattern: Option<(Vec<i32>, u32)>,
}

impl ChartPlayer {
    /// Start the chart. `total_beats` bounds the run.
    #[must_use]
    pub fn new(chart: &ChartData) -> ChartPlayer {
        ChartPlayer {
            step_in_chart: 0,
            total_steps: u64::from(chart.total_beats) * STEPS_PER_BEAT,
            inputs: Vec::new(),
            resolved: vec![false; chart.notes.len()],
            thud_until: 0,
            samples: Vec::new(),
            pending_pattern: None,
        }
    }

    /// Record a player input for a note beat, with its timing
    /// offset in steps (0 = on the beat; sign = early/late).
    pub fn record_input(&mut self, beat: u32, offset_steps: i64) {
        self.inputs.push((u64::from(beat), offset_steps));
    }

    /// Whether the chart has run to its end.
    #[must_use]
    pub fn finished(&self) -> bool {
        self.step_in_chart >= self.total_steps
    }

    /// The pattern step duration the player emits, in ms.
    #[must_use]
    pub fn pattern_step_ms() -> u32 {
        (SAMPLE_STRIDE_STEPS * 1000 / STEPS_PER_SECOND) as u32
    }

    /// Accuracy from a timing offset: inside ±6 steps (50 ms) is
    /// full; it falls linearly to 0.5 at ±12 steps; beyond is a
    /// miss (handled by the caller).
    #[must_use]
    pub fn accuracy_for_offset(offset_steps: i64) -> Option<f64> {
        let magnitude = offset_steps.unsigned_abs();
        match magnitude {
            0..=6 => Some(1.0),
            7..=12 => Some(1.0 - (f64::from(magnitude as u32) - 6.0) * (0.5 / 6.0)),
            _ => None,
        }
    }

    /// Advance one fixed step. `delivered` is the intensity the
    /// caller computed for the previous step's curve (the sampler
    /// records what actually left). Returns the step's curve and
    /// any judgement resolving now. Also stages an outbound pattern
    /// every two beats; take it with `take_pattern`.
    pub fn advance(&mut self, chart: &ChartData, delivered: i32) -> ChartStep {
        let step = self.step_in_chart;
        let beat = step / STEPS_PER_BEAT;
        let music = chart
            .music_layer_per_beat
            .get(beat as usize)
            .copied()
            .unwrap_or(0.0);
        let mut judgement: Option<Judgement> = None;

        // Resolve notes whose beat ends on this step.
        for (index, note) in chart.notes.iter().enumerate() {
            if self.resolved[index] || u64::from(note.beat) != beat {
                continue;
            }
            let beat_end = (beat + 1) * STEPS_PER_BEAT - 1;
            if step == u64::from(note.beat) * STEPS_PER_BEAT {
                // Beat opens: an input recorded for this beat with
                // an in-window offset resolves as a hit now.
                let recorded = self
                    .inputs
                    .iter()
                    .find(|(input_beat, _)| *input_beat == u64::from(note.beat))
                    .map(|(_, offset)| *offset);
                if let Some(offset) = recorded
                    && let Some(accuracy) = Self::accuracy_for_offset(offset)
                {
                    judgement = Some(Judgement::Hit(accuracy));
                    self.resolved[index] = true;
                }
            }
            if step == beat_end && !self.resolved[index] {
                judgement = Some(Judgement::Miss);
                self.resolved[index] = true;
                self.thud_until = step + THUD_STEPS;
            }
        }

        let curve = if step < self.thud_until {
            THUD_CURVE
        } else if let Some(Judgement::Hit(accuracy)) = judgement {
            music * accuracy
        } else {
            music * PRESENCE_FACTOR
        };

        // Sampling + pattern staging (B3 delivery discipline).
        if step.is_multiple_of(SAMPLE_STRIDE_STEPS) && step > 0 {
            self.samples.push(delivered);
            if self.samples.len() == PATTERN_STRENGTHS {
                self.pending_pattern =
                    Some((std::mem::take(&mut self.samples), Self::pattern_step_ms()));
            }
        }
        self.step_in_chart += 1;
        ChartStep { curve, judgement }
    }

    /// Take a staged outbound pattern, if one completed.
    pub fn take_pattern(&mut self) -> Option<(Vec<i32>, u32)> {
        self.pending_pattern.take()
    }
}

/// Which note (if any) an input at `beat` refers to — exposed for
/// the sim's input gate.
#[must_use]
pub fn note_at(chart: &ChartData, beat: u32) -> Option<&ChartNote> {
    chart.notes.iter().find(|note| note.beat == beat)
}
