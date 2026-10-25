namespace Strobe.Core
{
    /// <summary>
    /// The four per-heroine response tracks (ADAPT-SPEC A3). Track XP
    /// is earned only through kink-aligned play (see Session); each
    /// track's level-3 capstone is a named ability that acts as a
    /// content key and raises the session tier ceiling story-side.
    /// </summary>
    public enum ResponseTrack
    {
        Gaze = 0,
        Rhythm = 1,
        Touch = 2,
        Voice = 3,
    }

    /// <summary>XP ladder for one track. Pure functions, no state.</summary>
    public static class TrackLadder
    {
        // Cumulative XP at which levels 1, 2, 3 are reached.
        private static readonly int[] LevelThresholds = { 30, 70, 120 };

        /// <summary>Highest track level (the capstone level).</summary>
        public const int MaxLevel = 3;

        /// <summary>Level for a cumulative XP total (0..MaxLevel).</summary>
        public static int LevelForXp(int xp)
        {
            if (xp <= 0)
            {
                return 0;
            }

            var level = 0;
            for (var i = 0; i < LevelThresholds.Length; i++)
            {
                if (xp >= LevelThresholds[i])
                {
                    level = i + 1;
                }
            }

            return level;
        }

        /// <summary>True exactly when this XP total is the capstone rung.</summary>
        public static bool IsCapstone(int xp)
        {
            return LevelForXp(xp) >= MaxLevel;
        }
    }
}
