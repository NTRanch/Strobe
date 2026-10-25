using System;

namespace Strobe.Core
{
    /// <summary>
    /// How he brings her out of trance for one Surface Beat
    /// (SCENARIO.md, the Surface Beat spec). The trigger is his pacing
    /// tool: chosen by the player, fired deliberately. It is never the
    /// comfort limit and never a failure state.
    /// </summary>
    public enum SurfaceTrigger
    {
        NameSpoken = 0,
        CountToZero = 1,
        ToyStop = 2,
        Kiss = 3,
        CommandWord = 4,
        Orgasm = 5,
        EyeContact = 6,
    }

    /// <summary>
    /// The shape of her fluster when she surfaces: how her reaction
    /// shows the want was hers. One grammar per heroine per beat,
    /// authored in her scene spec. These are shapes, not content: the
    /// words and faces live in the text/art layers, keyed by
    /// <see cref="SurfaceReaction"/>.
    /// </summary>
    public enum ReactionGrammar
    {
        /// <summary>Standard reaction until her own is authored.</summary>
        GazeAvert = 0,

        /// <summary>Posture holds while the face fails (Yuna; her variant is pending cast-book authoring).</summary>
        ComposureCrack = 1,

        /// <summary>Slow blink, reaches for him (Cordelia).</summary>
        SlowBlinkReach = 2,

        /// <summary>Covers her face, laughing (Vivienne).</summary>
        LaughingCover = 3,

        /// <summary>Her body answers before her face does (Valentina).</summary>
        BodyAnswersFirst = 4,

        /// <summary>Her nod at the position switch is the beat (Maren).</summary>
        SwitchConsent = 5,

        /// <summary>Command register cracks into plain wanting (Manon).</summary>
        FramingBreak = 6,

        /// <summary>Both hands over her face, grinning through her fingers (Sadie; the 2342056 shape).</summary>
        FaceCover = 7,
    }

    /// <summary>
    /// Her authored reaction to one surface: a grammar plus the text
    /// and art keys the presentation layer resolves. Authored per
    /// heroine in her scene spec and supplied by the caller, never
    /// guessed by this model — the same discipline as
    /// <see cref="SessionAction.KinkAligned"/>. Keys may be empty
    /// (no line / no dedicated expression); empty is a designed state,
    /// null is not accepted as distinct from it.
    /// </summary>
    public sealed class SurfaceReaction
    {
        public SurfaceReaction(
            ReactionGrammar grammar, string voiceLineKey, string expressionKey)
        {
            Grammar = grammar;
            VoiceLineKey = voiceLineKey ?? string.Empty;
            ExpressionKey = expressionKey ?? string.Empty;
        }

        public ReactionGrammar Grammar { get; }

        /// <summary>Text-layer key for her surface line (may be empty).</summary>
        public string VoiceLineKey { get; }

        /// <summary>Art-layer key for her surface expression (may be empty).</summary>
        public string ExpressionKey { get; }
    }

    /// <summary>What one surface changed. Returned by Session.Surface.</summary>
    public sealed class SurfaceResult
    {
        internal SurfaceResult(
            double responseDelta, SurfaceReaction reaction, int surfacesThisSession)
        {
            ResponseDelta = responseDelta;
            Reaction = reaction;
            SurfacesThisSession = surfacesThisSession;
        }

        /// <summary>Always zero or negative: surfacing shallows trance.</summary>
        public double ResponseDelta { get; }

        /// <summary>The authored reaction the presentation must play.</summary>
        public SurfaceReaction Reaction { get; }

        /// <summary>Running count of surfaces in this session.</summary>
        public int SurfacesThisSession { get; }
    }
}
