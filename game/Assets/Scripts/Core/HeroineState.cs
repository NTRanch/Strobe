using System;
using System.Collections.Generic;

namespace Strobe.Core
{
    /// <summary>
    /// One heroine's persistent progression state: her W5 stage
    /// (derived, never stored independently), her response-track XP,
    /// her capstones, her corruption baseline, her outfit currency,
    /// and whether her frequency has been learned.
    ///
    /// Persistence is the caller's problem (save layer serializes the
    /// plain fields through the Snapshot type); this type owns the
    /// invariants:
    ///  - Clear count never decreases and never goes negative.
    ///  - Stage is always <see cref="Progression.StageForCount"/> of
    ///    the count; a stored stage is never trusted over the ladder.
    ///  - Track XP never decreases; capstones, once unlocked, stay.
    ///  - Corruption baseline is a pure function of stage (A2): it
    ///    only ever grows across the campaign.
    /// </summary>
    public sealed class HeroineState
    {
        private readonly int[] _trackXp = new int[4];
        private readonly bool[] _capstones = new bool[4];

        public HeroineState(string heroineId)
        {
            if (string.IsNullOrEmpty(heroineId))
            {
                throw new ArgumentException(
                    "Heroine id must be non-empty.", nameof(heroineId));
            }

            HeroineId = heroineId;
        }

        public string HeroineId { get; }

        /// <summary>Lifetime clears (hypnosis count). Monotonic.</summary>
        public int ClearCount { get; private set; }

        /// <summary>Soft currency earned by session performance (A3/D.10).</summary>
        public int OutfitCoins { get; private set; }

        /// <summary>True once her frequency sweet spot has been found in play.</summary>
        public bool FrequencyLearned { get; private set; }

        /// <summary>Her current W5 stage, derived from the clear count.</summary>
        public W5Stage Stage => Progression.StageForCount(ClearCount);

        /// <summary>
        /// Session corruption starts here (ADAPT-SPEC A2): the share
        /// of the corruption bar her campaign growth pre-fills. At
        /// Craving (3) she begins sessions already wanting more; at
        /// Self-Driven (5) she arrives willing.
        /// </summary>
        public double CorruptionBaseline => (int)Stage * 12.0;

        /// <summary>Highest session tier her stage alone lets her ask for.</summary>
        public int StageTierCeiling => Math.Min(Session.MaxTier, (int)Stage);

        internal void AddClear()
        {
            ClearCount++;
        }

        /// <summary>Restore a persisted clear count (load path; clamps at 0).</summary>
        public void RestoreClearCount(int clearCount)
        {
            ClearCount = Math.Max(0, clearCount);
        }

        /// <summary>Cumulative XP on one response track.</summary>
        public int TrackXp(ResponseTrack track)
        {
            return _trackXp[(int)track];
        }

        /// <summary>Level on one response track (0..TrackLadder.MaxLevel).</summary>
        public int TrackLevel(ResponseTrack track)
        {
            return TrackLadder.LevelForXp(_trackXp[(int)track]);
        }

        /// <summary>True when the track's capstone ability is unlocked.</summary>
        public bool HasCapstone(ResponseTrack track)
        {
            return _capstones[(int)track];
        }

        /// <summary>All unlocked capstone tracks, in enum order.</summary>
        public IReadOnlyList<ResponseTrack> Capstones
        {
            get
            {
                var list = new List<ResponseTrack>();
                for (var i = 0; i < _capstones.Length; i++)
                {
                    if (_capstones[i])
                    {
                        list.Add((ResponseTrack)i);
                    }
                }

                return list;
            }
        }

        /// <summary>
        /// Award track XP (session path only). Returns true exactly
        /// when this award unlocks the track's capstone.
        /// </summary>
        public bool AwardTrackXp(ResponseTrack track, int xp)
        {
            if (xp < 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(xp), xp, "Track XP awards cannot be negative.");
            }

            var slot = (int)track;
            _trackXp[slot] += xp;
            if (!_capstones[slot] && TrackLadder.IsCapstone(_trackXp[slot]))
            {
                _capstones[slot] = true;
                return true;
            }

            return false;
        }

        /// <summary>Restore persisted track XP (load path; clamps at 0, re-derives capstone).</summary>
        public void RestoreTrackXp(ResponseTrack track, int xp)
        {
            var slot = (int)track;
            _trackXp[slot] = Math.Max(0, xp);
            _capstones[slot] = TrackLadder.IsCapstone(_trackXp[slot]);
        }

        /// <summary>Credit outfit coins from session performance.</summary>
        public void AddOutfitCoins(int coins)
        {
            if (coins < 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(coins), coins, "Coin awards cannot be negative.");
            }

            OutfitCoins += coins;
        }

        /// <summary>Restore persisted coins (load path; clamps at 0).</summary>
        public void RestoreOutfitCoins(int coins)
        {
            OutfitCoins = Math.Max(0, coins);
        }

        /// <summary>Mark her frequency as learned (session discovery).</summary>
        public void LearnFrequency()
        {
            FrequencyLearned = true;
        }

        /// <summary>Restore the learned flag (load path).</summary>
        public void RestoreFrequencyLearned(bool learned)
        {
            FrequencyLearned = learned;
        }
    }
}
