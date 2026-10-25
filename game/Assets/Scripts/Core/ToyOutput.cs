using System;

namespace Strobe.Core
{
    /// <summary>
    /// The toy-output seam (W4/D6 + ADAPT-SPEC A2). The transport
    /// (Lovense bridge) sits behind this interface; Core owns only
    /// the intensity law:
    ///
    ///   delivered = curve x tierCeiling x playerCeiling,
    ///   always 0 while the master switch is off.
    ///
    /// The tier ceiling is A2's consent rule in arithmetic: hardware
    /// intensity can never exceed the deepest tier her corruption
    /// has unlocked this session. The player ceiling and the master
    /// switch are the standing bridge safety contract (master OFF by
    /// default, ceiling clamped in code, stop synchronous, no resume
    /// on reconnect) and are enforced by the bridge, not here — here
    /// they arrive as already-clamped inputs.
    /// </summary>
    public interface IToyOutput
    {
        /// <summary>Deliver one intensity sample, 0..MaxIntensity.</summary>
        void Send(int intensity);

        /// <summary>Stop all output now. Synchronous by contract.</summary>
        void StopAll();
    }

    /// <summary>Pure intensity computation for the D6/A2 curve.</summary>
    public static class ToyCurve
    {
        /// <summary>Bridge intensity scale (Lovense 0..20).</summary>
        public const int MaxIntensity = 20;

        /// <summary>
        /// Compute the intensity to deliver.
        /// </summary>
        /// <param name="masterOn">The bridge master switch.</param>
        /// <param name="curveValue">D6 curve output before ceilings, 0..1
        /// (music layer x judgement layer, computed by the caller).</param>
        /// <param name="unlockedTier">Deepest tier her corruption has
        /// unlocked this session, 0..Session.MaxTier.</param>
        /// <param name="playerCeiling">Player-set ceiling, 0..MaxIntensity.</param>
        public static int ComputeIntensity(
            bool masterOn, double curveValue, int unlockedTier, int playerCeiling)
        {
            if (!masterOn)
            {
                return 0;
            }

            if (curveValue < 0.0)
            {
                curveValue = 0.0;
            }
            else if (curveValue > 1.0)
            {
                curveValue = 1.0;
            }

            if (unlockedTier < 0)
            {
                unlockedTier = 0;
            }
            else if (unlockedTier > Session.MaxTier)
            {
                unlockedTier = Session.MaxTier;
            }

            if (playerCeiling < 0)
            {
                playerCeiling = 0;
            }
            else if (playerCeiling > MaxIntensity)
            {
                playerCeiling = MaxIntensity;
            }

            // Her tier ceiling: tier t of MaxTier permits t/MaxTier of
            // the scale. Tier 0 is touch-free: rhythm presence only
            // comes with the first willing tier.
            var tierCeiling = MaxIntensity * unlockedTier / Session.MaxTier;
            var raw = (int)Math.Round(curveValue * tierCeiling);
            return Math.Min(raw, playerCeiling);
        }
    }

    /// <summary>
    /// A toy output that does nothing. The default everywhere: a
    /// build without a connected bridge is silent by construction.
    /// </summary>
    public sealed class NullToyOutput : IToyOutput
    {
        public void Send(int intensity)
        {
        }

        public void StopAll()
        {
        }
    }
}
