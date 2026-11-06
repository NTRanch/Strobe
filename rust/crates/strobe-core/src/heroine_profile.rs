//! Static per-heroine design data (the code form of her scene
//! spec's mechanical facts). Content strings (her desire, her
//! block, her capstone names) are keys into the text layer, not
//! literals, so this crate stays content-free.

use crate::error::CoreError;
use crate::response_track::ResponseTrack;

/// Which of the two PATTERNS vectors a heroine's arc runs on. The
/// load-bearing rule: control never installs a foreign desire; it
/// taps a desire she has hidden, or dissolves a block that keeps her
/// from enjoying what she already wants. Recorded per heroine in her
/// scene spec; mirrored here as data. Discriminants port verbatim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum KinkVectorKind {
    /// Her arc taps a desire she has hidden.
    HiddenDesire = 0,
    /// Her arc dissolves a block guarding what she already wants.
    DissolvedBlock = 1,
}

impl KinkVectorKind {
    /// Rebuild a vector kind from a raw discriminant (data layer).
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `value` is not 0..=1.
    pub fn from_discriminant(value: i32) -> Result<KinkVectorKind, CoreError> {
        match value {
            0 => Ok(KinkVectorKind::HiddenDesire),
            1 => Ok(KinkVectorKind::DissolvedBlock),
            _ => Err(CoreError::OutOfRange {
                param: "vector_kind",
                message: "Vector kind is 0..=1.",
            }),
        }
    }
}

/// One heroine's static design data.
#[derive(Clone, Debug, PartialEq)]
pub struct HeroineProfile {
    id: String,
    vector_kind: KinkVectorKind,
    frequency_center: f64,
    desire_key: String,
    block_key: String,
    track_weights: [f64; 4],
}

impl HeroineProfile {
    /// Slowest permitted response-track weight.
    pub const MIN_TRACK_WEIGHT: f64 = 0.25;

    /// Fastest permitted response-track weight.
    pub const MAX_TRACK_WEIGHT: f64 = 3.0;

    /// Build a profile. `track_weights` names per-track learning
    /// weights (ADAPT-SPEC A3); every track it does not name keeps
    /// the default weight 1.0. Out-of-range authored data fails
    /// fast here rather than being silently clamped, so bad data
    /// cannot quietly change balance.
    ///
    /// # Errors
    /// - [`CoreError::Argument`] when `id` is empty.
    /// - [`CoreError::OutOfRange`] when `frequency_center` is
    ///   outside 0.0..=1.0, or a weight is non-finite or outside
    ///   0.25..=3.0.
    pub fn new(
        id: &str,
        vector_kind: KinkVectorKind,
        frequency_center: f64,
        desire_key: &str,
        block_key: &str,
        track_weights: &[(ResponseTrack, f64)],
    ) -> Result<HeroineProfile, CoreError> {
        if id.is_empty() {
            return Err(CoreError::Argument {
                param: "id",
                message: "Heroine id must be non-empty.",
            });
        }
        if frequency_center < 0.0 || frequency_center > 1.0 {
            return Err(CoreError::OutOfRange {
                param: "frequency_center",
                message: "Frequency center is a position on the 0..1 session dial.",
            });
        }
        let mut weights = [1.0_f64; 4];
        for &(track, weight) in track_weights {
            validate_track_weight(weight)?;
            weights[track.slot()] = weight;
        }
        Ok(HeroineProfile {
            id: id.to_owned(),
            vector_kind,
            frequency_center,
            desire_key: desire_key.to_owned(),
            block_key: block_key.to_owned(),
            track_weights: weights,
        })
    }

    /// Stable heroine id (matches her art/text keys).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Her PATTERNS vector: hidden desire or dissolved block.
    #[must_use]
    pub fn vector_kind(&self) -> KinkVectorKind {
        self.vector_kind
    }

    /// Her "frequency": the kink-aligned sweet spot on the session
    /// dial (ADAPT-SPEC A2, find-her-frequency). Learned through
    /// play; see `HeroineState::frequency_learned`.
    #[must_use]
    pub fn frequency_center(&self) -> f64 {
        self.frequency_center
    }

    /// Text-layer key naming her hidden desire.
    #[must_use]
    pub fn desire_key(&self) -> &str {
        &self.desire_key
    }

    /// Text-layer key naming her block (may be empty).
    #[must_use]
    pub fn block_key(&self) -> &str {
        &self.block_key
    }

    /// Her per-track learning weights (ADAPT-SPEC A3): profile data,
    /// bounded to 0.25..=3.0, defaulting to 1.0 for every track the
    /// profile does not name. `Session` consumes them only when
    /// awarding track XP; they never alter Response, Corruption,
    /// tier thresholds, or the capstone ladder.
    #[must_use]
    pub fn track_weights(&self) -> [f64; 4] {
        self.track_weights
    }

    /// Weight for one defined response track.
    #[must_use]
    pub fn track_weight(&self, track: ResponseTrack) -> f64 {
        self.track_weights[track.slot()]
    }

    /// Weight for a raw track discriminant (data/save layer).
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `discriminant` names no
    /// defined track — the port of the C#'s undefined-key lookup
    /// rejection.
    pub fn try_track_weight(&self, discriminant: i32) -> Result<f64, CoreError> {
        let track = ResponseTrack::from_discriminant(discriminant)?;
        Ok(self.track_weight(track))
    }
}

fn validate_track_weight(weight: f64) -> Result<(), CoreError> {
    if !weight.is_finite()
        || weight < HeroineProfile::MIN_TRACK_WEIGHT
        || weight > HeroineProfile::MAX_TRACK_WEIGHT
    {
        return Err(CoreError::OutOfRange {
            param: "weight",
            message: "Track weights must be finite and within 0.25..3.0.",
        });
    }
    Ok(())
}
