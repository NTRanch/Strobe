using System;

namespace Strobe.Core
{
    /// <summary>
    /// Which of the two PATTERNS vectors a heroine's arc runs on. The
    /// load-bearing rule: control never installs a foreign desire; it
    /// taps a desire she has hidden, or dissolves a block that keeps
    /// her from enjoying what she already wants. Recorded per heroine
    /// in her scene spec; mirrored here as data.
    /// </summary>
    public enum KinkVectorKind
    {
        HiddenDesire = 0,
        DissolvedBlock = 1,
    }

    /// <summary>
    /// Static per-heroine design data (the code form of her scene
    /// spec's mechanical facts). Content strings (her desire, her
    /// block, her capstone names) are keys into the text layer, not
    /// literals, so this assembly stays content-free.
    /// </summary>
    public sealed class HeroineProfile
    {
        public HeroineProfile(
            string id,
            KinkVectorKind vectorKind,
            double frequencyCenter,
            string desireKey,
            string blockKey)
        {
            if (string.IsNullOrEmpty(id))
            {
                throw new ArgumentException("Heroine id must be non-empty.", nameof(id));
            }

            if (frequencyCenter < 0.0 || frequencyCenter > 1.0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(frequencyCenter), frequencyCenter,
                    "Frequency center is a position on the 0..1 session dial.");
            }

            Id = id;
            VectorKind = vectorKind;
            FrequencyCenter = frequencyCenter;
            DesireKey = desireKey ?? string.Empty;
            BlockKey = blockKey ?? string.Empty;
        }

        /// <summary>Stable heroine id (matches her art/text keys).</summary>
        public string Id { get; }

        /// <summary>Her PATTERNS vector: hidden desire or dissolved block.</summary>
        public KinkVectorKind VectorKind { get; }

        /// <summary>
        /// Her "frequency": the kink-aligned sweet spot on the session
        /// dial (ADAPT-SPEC A2, find-her-frequency). Learned through
        /// play; see <see cref="HeroineState.FrequencyLearned"/>.
        /// </summary>
        public double FrequencyCenter { get; }

        /// <summary>Text-layer key naming her hidden desire.</summary>
        public string DesireKey { get; }

        /// <summary>Text-layer key naming her block (may be empty).</summary>
        public string BlockKey { get; }
    }
}
