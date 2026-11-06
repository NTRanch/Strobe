//! The data layer (SPEC-BEVY G2-B): the slice's content lives in
//! `assets/data/*.json`, loaded at startup and mapped onto the
//! ported Core types. Nothing in Core knows these files exist; the
//! game constructs Core values from them. Invalid data fails the
//! boot loudly (Core's own constructor validation is the gate).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use strobe_core::{
    CampaignClock, CoreError, HeroineProfile, KinkVectorKind, ReactionGrammar, ResponseTrack,
    SurfaceReaction,
};

/// Everything that can go wrong loading the data files.
#[derive(Debug)]
pub enum DataError {
    /// A file could not be read.
    Io { path: String, message: String },
    /// A file did not parse or did not validate.
    Invalid { path: String, message: String },
    /// Core rejected constructed values.
    Core(CoreError),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Io { path, message } => write!(f, "cannot read {path}: {message}"),
            DataError::Invalid { path, message } => {
                write!(f, "invalid data in {path}: {message}")
            }
            DataError::Core(error) => write!(f, "core rejected data: {error}"),
        }
    }
}

impl std::error::Error for DataError {}

impl From<CoreError> for DataError {
    fn from(error: CoreError) -> Self {
        DataError::Core(error)
    }
}

/// One authored action row (SPEC §2.2's action/alignment table).
#[derive(Clone, Debug, PartialEq)]
pub struct ActionDef {
    /// Stable action id (also its line-table label key stem).
    pub id: String,
    /// The response track it trains.
    pub track: ResponseTrack,
    /// The tier it belongs to (and therefore requires).
    pub tier: i32,
    /// Whether it is aligned with her hidden vector.
    pub kink_aligned: bool,
    /// Whether it is rushed.
    pub rushed: bool,
    /// Whether she initiates it (the self-recognition lever).
    pub initiated_by_her: bool,
    /// The dial position it plays at, 0..=1.
    pub dial_position: f64,
}

/// Core stays serde-free (SPEC-BEVY G7-B), so the data layer maps
/// track / vector / grammar names onto Core's enums itself.
fn parse_track(name: &str, path: &str) -> Result<ResponseTrack, DataError> {
    match name {
        "gaze" => Ok(ResponseTrack::Gaze),
        "rhythm" => Ok(ResponseTrack::Rhythm),
        "touch" => Ok(ResponseTrack::Touch),
        "voice" => Ok(ResponseTrack::Voice),
        other => Err(DataError::Invalid {
            path: path.to_string(),
            message: format!("unknown response track '{other}'"),
        }),
    }
}

#[derive(Deserialize)]
struct RawAction {
    id: String,
    track: String,
    tier: i32,
    kink_aligned: bool,
    rushed: bool,
    initiated_by_her: bool,
    dial_position: f64,
}

#[derive(Deserialize)]
struct ActionsFile {
    context_id: String,
    actions: Vec<RawAction>,
}

#[derive(Deserialize)]
struct ProfileFile {
    id: String,
    vector_kind: String,
    frequency_center: f64,
    desire_key: String,
    block_key: String,
    track_weights: Vec<WeightRow>,
    season: SeasonRow,
    surface_reaction: ReactionRow,
    capstone_keys: BTreeMap<String, String>,
    context_id: String,
}

#[derive(Deserialize)]
struct WeightRow {
    track: String,
    weight: f64,
}

#[derive(Deserialize)]
struct SeasonRow {
    total_days: i32,
    slots_per_day: i32,
    season_id: String,
}

#[derive(Deserialize)]
struct ReactionRow {
    grammar: String,
    voice_line_key: String,
    expression_key: String,
}

/// The chart (SPEC §2.3 / §6-B): a music layer plus judged notes.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ChartData {
    /// Chart id.
    pub id: String,
    /// Tempo; the slice chart is authored at 120 BPM (500 ms/beat).
    pub bpm: u32,
    /// Total beats in the chart.
    pub total_beats: u32,
    /// The music layer: intensity per beat, 0..=1.
    pub music_layer_per_beat: Vec<f64>,
    /// Judged notes (beat index + lane).
    pub notes: Vec<ChartNote>,
}

/// One judged note.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ChartNote {
    /// The beat the note lands on.
    pub beat: u32,
    /// The lane (presentation channel) of the note.
    pub lane: String,
}

/// One line-table entry: authored text, or a key whose wording is
/// owed (SPEC §2.5 — refrain-bearing lines are never improvised).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LineEntry {
    /// A line with authored text.
    Text {
        /// The line's text.
        text: String,
    },
    /// A line whose wording is owed from its source document.
    Owed {
        /// Always "owed" for this variant.
        status: String,
        /// Where the wording comes from, when known.
        #[serde(default)]
        source: Option<String>,
    },
}

#[derive(Deserialize)]
struct LinesFile {
    lines: BTreeMap<String, LineEntry>,
}

/// The season's shape (days × slots), from `vivienne.json`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeasonData {
    /// Days in the season.
    pub total_days: i32,
    /// Session slots per day.
    pub slots_per_day: i32,
}

/// All slice data, loaded and validated.
#[derive(Clone, Debug)]
pub struct GameData {
    /// Her Core profile, constructed from `vivienne.json`.
    pub profile: HeroineProfile,
    /// Her authored Surface Beat reaction (SPEC §2.3).
    pub surface_reaction: SurfaceReaction,
    /// Her season's shape.
    pub season: SeasonData,
    /// The season's stable id.
    pub season_id: String,
    /// The scene context every action in the table plays in.
    pub context_id: String,
    /// The action/alignment table.
    pub actions: Vec<ActionDef>,
    /// The chart.
    pub chart: ChartData,
    /// The text layer: line key → entry.
    pub lines: BTreeMap<String, LineEntry>,
    /// Capstone content keys by track.
    pub capstone_keys: BTreeMap<ResponseTrack, String>,
}

impl GameData {
    /// Load and validate the data set from a directory holding
    /// `vivienne.json`, `actions.json`, `chart.json`, `lines.json`.
    pub fn load(dir: &Path) -> Result<GameData, DataError> {
        let profile_raw = read_file(&dir.join("vivienne.json"))?;
        let profile_file: ProfileFile = parse_file(&profile_raw, "vivienne.json")?;
        let actions_raw = read_file(&dir.join("actions.json"))?;
        let actions_file: ActionsFile = parse_file(&actions_raw, "actions.json")?;
        let chart_raw = read_file(&dir.join("chart.json"))?;
        let chart: ChartData = parse_file(&chart_raw, "chart.json")?;
        let lines_raw = read_file(&dir.join("lines.json"))?;
        let lines_file: LinesFile = parse_file(&lines_raw, "lines.json")?;

        if actions_file.context_id != profile_file.context_id {
            return Err(DataError::Invalid {
                path: "actions.json".to_string(),
                message: format!(
                    "context '{}' does not match the profile context '{}'",
                    actions_file.context_id, profile_file.context_id
                ),
            });
        }
        if chart.music_layer_per_beat.len() != chart.total_beats as usize {
            return Err(DataError::Invalid {
                path: "chart.json".to_string(),
                message: "music layer length must equal total_beats".to_string(),
            });
        }
        for note in &chart.notes {
            if note.beat >= chart.total_beats {
                return Err(DataError::Invalid {
                    path: "chart.json".to_string(),
                    message: format!("note beat {} is past the chart end", note.beat),
                });
            }
        }
        let mut actions: Vec<ActionDef> = Vec::with_capacity(actions_file.actions.len());
        for raw in &actions_file.actions {
            if !(0.0..=1.0).contains(&raw.dial_position) {
                return Err(DataError::Invalid {
                    path: "actions.json".to_string(),
                    message: format!("action '{}' dial outside 0..=1", raw.id),
                });
            }
            actions.push(ActionDef {
                id: raw.id.clone(),
                track: parse_track(&raw.track, "actions.json")?,
                tier: raw.tier,
                kink_aligned: raw.kink_aligned,
                rushed: raw.rushed,
                initiated_by_her: raw.initiated_by_her,
                dial_position: raw.dial_position,
            });
        }

        let vector_kind = match profile_file.vector_kind.as_str() {
            "dissolved_block" => KinkVectorKind::DissolvedBlock,
            "hidden_desire" => KinkVectorKind::HiddenDesire,
            other => {
                return Err(DataError::Invalid {
                    path: "vivienne.json".to_string(),
                    message: format!("unknown vector kind '{other}'"),
                });
            }
        };
        let grammar = match profile_file.surface_reaction.grammar.as_str() {
            "body_answers_first" => ReactionGrammar::BodyAnswersFirst,
            "composure_crack" => ReactionGrammar::ComposureCrack,
            "face_cover" => ReactionGrammar::FaceCover,
            "framing_break" => ReactionGrammar::FramingBreak,
            "gaze_avert" => ReactionGrammar::GazeAvert,
            "laughing_cover" => ReactionGrammar::LaughingCover,
            "slow_blink_reach" => ReactionGrammar::SlowBlinkReach,
            "switch_consent" => ReactionGrammar::SwitchConsent,
            other => {
                return Err(DataError::Invalid {
                    path: "vivienne.json".to_string(),
                    message: format!("unknown reaction grammar '{other}'"),
                });
            }
        };
        let mut weights: Vec<(ResponseTrack, f64)> = Vec::new();
        for row in &profile_file.track_weights {
            weights.push((parse_track(&row.track, "vivienne.json")?, row.weight));
        }
        let reaction = SurfaceReaction::new(
            grammar,
            &profile_file.surface_reaction.voice_line_key,
            &profile_file.surface_reaction.expression_key,
        );
        let profile = HeroineProfile::new(
            &profile_file.id,
            vector_kind,
            profile_file.frequency_center,
            &profile_file.desire_key,
            &profile_file.block_key,
            &weights,
        )?;
        let season = SeasonData {
            total_days: profile_file.season.total_days,
            slots_per_day: profile_file.season.slots_per_day,
        };
        // Constructing a clock validates the season bounds.
        let _ = CampaignClock::new(season.total_days, season.slots_per_day)?;
        let mut capstone_keys: BTreeMap<ResponseTrack, String> = BTreeMap::new();
        for (name, key) in &profile_file.capstone_keys {
            capstone_keys.insert(parse_track(name, "vivienne.json")?, key.clone());
        }
        Ok(GameData {
            profile,
            surface_reaction: reaction,
            season,
            season_id: profile_file.season.season_id.clone(),
            context_id: profile_file.context_id.clone(),
            actions,
            chart,
            lines: lines_file.lines,
            capstone_keys,
        })
    }

    /// Find an action by id.
    #[must_use]
    pub fn action(&self, id: &str) -> Option<&ActionDef> {
        self.actions.iter().find(|action| action.id == id)
    }

    /// Resolve a line key: authored text when present, otherwise
    /// the key itself (dev builds display keys for owed lines).
    #[must_use]
    pub fn line_text<'a>(&'a self, key: &'a str) -> &'a str {
        match self.lines.get(key) {
            Some(LineEntry::Text { text }) => text,
            _ => key,
        }
    }
}

fn read_file(path: &Path) -> Result<String, DataError> {
    std::fs::read_to_string(path).map_err(|error| DataError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

fn parse_file<T: for<'de> Deserialize<'de>>(raw: &str, name: &str) -> Result<T, DataError> {
    serde_json::from_str(raw).map_err(|error| DataError::Invalid {
        path: name.to_string(),
        message: error.to_string(),
    })
}
