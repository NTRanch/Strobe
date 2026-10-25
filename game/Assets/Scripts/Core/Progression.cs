using System;

namespace Strobe.Core
{
    /// <summary>
    /// W3/W5 progression: maps a heroine's lifetime clear count to her
    /// W5 stage through the sidecar ladder inherited from Track A
    /// (0/1/3/6/10/15 clears -&gt; stages 0..5). Pure mapping; state
    /// lives in <see cref="HeroineState"/>.
    ///
    /// Invariants (carried from the Track A engine):
    ///  - Monotonic: as a count grows, the stage never decreases.
    ///  - No decay, no punishment states: nothing here ever lowers a
    ///    count or a stage.
    ///  - Counts at or below 0 map to stage 0; negative stored counts
    ///    are clamped to 0 by <see cref="HeroineState"/>.
    /// </summary>
    public static class Progression
    {
        // Ladder thresholds, ascending, one per stage 1..5.
        private static readonly int[] Thresholds = { 1, 3, 6, 10, 15 };

        /// <summary>The pure mapping: lifetime clear count -&gt; stage.</summary>
        public static W5Stage StageForCount(int clearCount)
        {
            if (clearCount <= 0)
            {
                return W5Stage.Unaware;
            }

            var stage = 0;
            for (var i = 0; i < Thresholds.Length; i++)
            {
                if (clearCount >= Thresholds[i])
                {
                    stage = i + 1;
                }
            }

            return (W5Stage)stage;
        }

        /// <summary>Clears still needed to reach the next stage; 0 at the top.</summary>
        public static int ClearsToNextStage(int clearCount)
        {
            var stage = (int)StageForCount(clearCount);
            if (stage >= Thresholds.Length)
            {
                return 0;
            }

            return Math.Max(0, Thresholds[stage] - Math.Max(0, clearCount));
        }

        /// <summary>
        /// Record one completed clear (a finished session at any tier)
        /// for a heroine. Returns her stage after the clear.
        /// </summary>
        public static W5Stage RecordClear(HeroineState state)
        {
            if (state == null)
            {
                throw new ArgumentNullException(nameof(state));
            }

            state.AddClear();
            return state.Stage;
        }
    }
}
