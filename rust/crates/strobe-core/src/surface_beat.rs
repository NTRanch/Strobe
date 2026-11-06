//! The Surface Beat's data types (SCENARIO.md): how he brings her
//! out of trance, the shape of her fluster when she surfaces, and
//! what one surface changed.

/// How he brings her out of trance for one Surface Beat
/// (SCENARIO.md, the Surface Beat spec). The trigger is his pacing
/// tool: chosen by the player, fired deliberately. It is never the
/// comfort limit and never a failure state. Discriminants port
/// verbatim from the C#.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum SurfaceTrigger {
    /// Her name, spoken.
    NameSpoken = 0,
    /// The count reaches zero (Vivienne's V2 variant).
    CountToZero = 1,
    /// The toy stops.
    ToyStop = 2,
    /// A kiss.
    Kiss = 3,
    /// A command word.
    CommandWord = 4,
    /// Orgasm.
    Orgasm = 5,
    /// Eye contact.
    EyeContact = 6,
}

impl SurfaceTrigger {
    /// The trigger's contract discriminant (0..=6).
    #[must_use]
    pub const fn discriminant(self) -> i32 {
        self as i32
    }
}

/// The shape of her fluster when she surfaces: how her reaction
/// shows the want was hers. One grammar per heroine per beat,
/// authored in her scene spec. These are shapes, not content: the
/// words and faces live in the text/art layers, keyed by
/// [`SurfaceReaction`]. Discriminants port verbatim from the C#.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum ReactionGrammar {
    /// Standard reaction until her own is authored.
    GazeAvert = 0,
    /// Posture holds while the face fails (Yuna; her variant is
    /// pending cast-book authoring).
    ComposureCrack = 1,
    /// Slow blink, reaches for him (Cordelia).
    SlowBlinkReach = 2,
    /// Covers her face, laughing (Vivienne).
    LaughingCover = 3,
    /// Her body answers before her face does (Valentina).
    BodyAnswersFirst = 4,
    /// Her nod at the position switch is the beat (Maren).
    SwitchConsent = 5,
    /// Command register cracks into plain wanting (Manon).
    FramingBreak = 6,
    /// Both hands over her face, grinning through her fingers
    /// (Sadie; the 2342056 shape).
    FaceCover = 7,
}

impl ReactionGrammar {
    /// The grammar's contract discriminant (0..=7).
    #[must_use]
    pub const fn discriminant(self) -> i32 {
        self as i32
    }
}

/// Her authored reaction to one surface: a grammar plus the text
/// and art keys the presentation layer resolves. Authored per
/// heroine in her scene spec and supplied by the caller, never
/// guessed by this model — the same discipline as
/// `SessionAction::kink_aligned`. Keys may be empty (no line / no
/// dedicated expression); empty is a designed state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceReaction {
    grammar: ReactionGrammar,
    voice_line_key: String,
    expression_key: String,
}

impl SurfaceReaction {
    /// Build a reaction record. Empty keys are a designed state.
    #[must_use]
    pub fn new(grammar: ReactionGrammar, voice_line_key: &str, expression_key: &str) -> Self {
        SurfaceReaction {
            grammar,
            voice_line_key: voice_line_key.to_owned(),
            expression_key: expression_key.to_owned(),
        }
    }

    /// The reaction's grammar.
    #[must_use]
    pub fn grammar(&self) -> ReactionGrammar {
        self.grammar
    }

    /// Text-layer key for her surface line (may be empty).
    #[must_use]
    pub fn voice_line_key(&self) -> &str {
        &self.voice_line_key
    }

    /// Art-layer key for her surface expression (may be empty).
    #[must_use]
    pub fn expression_key(&self) -> &str {
        &self.expression_key
    }
}

/// What one surface changed. Returned by `Session::surface`.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceResult {
    response_delta: f64,
    reaction: SurfaceReaction,
    surfaces_this_session: i32,
}

impl SurfaceResult {
    pub(crate) fn new(
        response_delta: f64,
        reaction: SurfaceReaction,
        surfaces_this_session: i32,
    ) -> Self {
        SurfaceResult {
            response_delta,
            reaction,
            surfaces_this_session,
        }
    }

    /// Always zero or negative: surfacing shallows trance.
    #[must_use]
    pub fn response_delta(&self) -> f64 {
        self.response_delta
    }

    /// The authored reaction the presentation must play.
    #[must_use]
    pub fn reaction(&self) -> &SurfaceReaction {
        &self.reaction
    }

    /// Running count of surfaces in this session.
    #[must_use]
    pub fn surfaces_this_session(&self) -> i32 {
        self.surfaces_this_session
    }
}
