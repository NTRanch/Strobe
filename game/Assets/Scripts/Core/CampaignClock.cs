using System;

namespace Strobe.Core
{
    /// <summary>How a heroine's season ends when its clock runs out (A4).</summary>
    public enum SeasonEnding
    {
        /// <summary>She reached Self-Driven: the full arc, in her own time.</summary>
        SelfDriven = 0,

        /// <summary>Craving or Devoted: a real arc, unfinished by days not by failure.</summary>
        Deepened = 1,

        /// <summary>Curious or Willing: a beginning. The season can be replayed.</summary>
        Begun = 2,

        /// <summary>Still Unaware: the softest ending. Not a game-over, a not-yet.</summary>
        NotYet = 3,
    }

    /// <summary>
    /// ADAPT-SPEC A4: one heroine's season is a bounded clock of days
    /// with a fixed number of action slots per day. Sessions, hub
    /// beats, and feed beats each cost a slot. When the clock runs
    /// out the season is evaluated against her stage — failure is an
    /// ending, never a wall, and the player's pacing resource
    /// (Focus) is a budgeting constraint, not a survival stat.
    /// </summary>
    public sealed class CampaignClock
    {
        public CampaignClock(int totalDays, int slotsPerDay)
        {
            if (totalDays <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(totalDays), totalDays, "A season needs at least one day.");
            }

            if (slotsPerDay <= 0 || slotsPerDay > 8)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(slotsPerDay), slotsPerDay, "Slots per day are bounded 1..8.");
            }

            TotalDays = totalDays;
            SlotsPerDay = slotsPerDay;
            Day = 1;
            Slot = 0;
        }

        public int TotalDays { get; }

        public int SlotsPerDay { get; }

        /// <summary>Current day, 1-based.</summary>
        public int Day { get; private set; }

        /// <summary>Slots already spent today, 0..SlotsPerDay.</summary>
        public int Slot { get; private set; }

        /// <summary>True when every slot of every day is spent.</summary>
        public bool SeasonOver => Day > TotalDays;

        /// <summary>Slots left today.</summary>
        public int SlotsLeftToday => SeasonOver ? 0 : SlotsPerDay - Slot;

        /// <summary>Spend one slot. Returns false when the season is over.</summary>
        public bool SpendSlot()
        {
            if (SeasonOver)
            {
                return false;
            }

            Slot++;
            if (Slot >= SlotsPerDay)
            {
                Slot = 0;
                Day++;
            }

            return true;
        }

        /// <summary>Evaluate the season's ending from her stage.</summary>
        public static SeasonEnding EvaluateEnding(W5Stage stage)
        {
            switch (stage)
            {
                case W5Stage.SelfDriven:
                    return SeasonEnding.SelfDriven;
                case W5Stage.Devoted:
                case W5Stage.Craving:
                    return SeasonEnding.Deepened;
                case W5Stage.Willing:
                case W5Stage.Curious:
                    return SeasonEnding.Begun;
                default:
                    return SeasonEnding.NotYet;
            }
        }
    }
}
