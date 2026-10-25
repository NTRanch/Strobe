using System;

namespace Strobe.Core
{
    /// <summary>Why a session ended. None while it is still running.</summary>
    public enum SessionEndReason
    {
        None = 0,
        Completed = 1,
        ComfortLimit = 2,
    }

    /// <summary>
    /// One player action inside a session (ADAPT-SPEC A2). An action
    /// belongs to a tier (its depth in HER kink ladder), works one
    /// response track, sits somewhere on the frequency dial, and is
    /// either aligned with her recorded kink vector or not. Alignment
    /// is authored per content beat, never guessed by this model.
    /// </summary>
    public sealed class SessionAction
    {
        public SessionAction(
            ResponseTrack track,
            int tier,
            bool kinkAligned,
            bool rushed,
            bool initiatedByHer,
            double dialPosition)
        {
            if (tier < 0 || tier > Session.MaxTier)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(tier), tier, "Action tier is 0..Session.MaxTier.");
            }

            if (dialPosition < 0.0 || dialPosition > 1.0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(dialPosition), dialPosition,
                    "Dial position is 0..1.");
            }

            Track = track;
            Tier = tier;
            KinkAligned = kinkAligned;
            Rushed = rushed;
            InitiatedByHer = initiatedByHer;
            DialPosition = dialPosition;
        }

        public ResponseTrack Track { get; }

        /// <summary>Depth of this action in her kink ladder, 0..MaxTier.</summary>
        public int Tier { get; }

        /// <summary>True when the action matches her recorded desire/block.</summary>
        public bool KinkAligned { get; }

        /// <summary>True when the action jumps pacing (pushing too fast).</summary>
        public bool Rushed { get; }

        /// <summary>True when she asked for this step (Craving+ behaviour).</summary>
        public bool InitiatedByHer { get; }

        /// <summary>Where on the frequency dial the action sits, 0..1.</summary>
        public double DialPosition { get; }
    }

    /// <summary>What one action changed. Returned by Session.ApplyAction.</summary>
    public sealed class ActionResult
    {
        internal ActionResult(
            double responseDelta,
            double corruptionDelta,
            int tierUnlocked,
            bool unlockInitiatedByHer,
            bool frequencyDiscovered,
            bool capstoneUnlocked,
            bool endedByComfortLimit)
        {
            ResponseDelta = responseDelta;
            CorruptionDelta = corruptionDelta;
            TierUnlocked = tierUnlocked;
            UnlockInitiatedByHer = unlockInitiatedByHer;
            FrequencyDiscovered = frequencyDiscovered;
            CapstoneUnlocked = capstoneUnlocked;
            EndedByComfortLimit = endedByComfortLimit;
        }

        public double ResponseDelta { get; }

        public double CorruptionDelta { get; }

        /// <summary>Newly unlocked tier, or -1 when none.</summary>
        public int TierUnlocked { get; }

        /// <summary>
        /// For a tier unlock: true when the unlocking action was hers
        /// (she initiated). A2 law: every unlock is signaled BY HER —
        /// the presentation layer must surface a signal (voice line,
        /// expression delta, or her initiation) for any unlock; this
        /// flag tells it which kind fired.
        /// </summary>
        public bool UnlockInitiatedByHer { get; }

        public bool FrequencyDiscovered { get; }

        public bool CapstoneUnlocked { get; }

        public bool EndedByComfortLimit { get; }
    }

    /// <summary>Session outcome, produced by Session.Complete.</summary>
    public sealed class SessionSummary
    {
        internal SessionSummary(
            int tierReached,
            double responsePeak,
            int coinsAwarded,
            W5Stage stageAfter,
            int surfaces)
        {
            TierReached = tierReached;
            ResponsePeak = responsePeak;
            CoinsAwarded = coinsAwarded;
            StageAfter = stageAfter;
            Surfaces = surfaces;
        }

        public int TierReached { get; }

        public double ResponsePeak { get; }

        public int CoinsAwarded { get; }

        public W5Stage StageAfter { get; }

        /// <summary>Surface Beats played this session (see Session.Surface).</summary>
        public int Surfaces { get; }
    }

    /// <summary>
    /// The A2 session model: Response x Corruption.
    ///
    ///  - Response is her in-the-moment trance/arousal depth. It
    ///    unlocks nothing by itself and resets between sessions.
    ///  - Corruption is her growing desire to dive deeper into HER
    ///    OWN kink. It rises only through kink-aligned action, decays
    ///    on off-kink action or rushing, and is the only gate on the
    ///    tier ladder. It is not a resistance bar: nothing here is
    ///    "broken" or "overcome".
    ///  - An action pitched above the tier her corruption has
    ///    unlocked meets her comfort limit: the session ends
    ///    gracefully, XP already earned is KEPT (zero progress loss),
    ///    no clear is recorded, and an aftercare beat is required.
    ///  - Composure is a pacing readout (strain), never a fail bar.
    ///  - The Surface Beat (SCENARIO.md) is a pacing event inside the
    ///    session: he brings her out of trance, her authored reaction
    ///    plays, and play resumes through ResumeTrance. Surfacing is
    ///    never a failure and never ends the session; the beat's only
    ///    corruption source is a resume she initiates herself.
    ///
    /// Deterministic and allocation-light: one action in, one result
    /// out, no clocks, no randomness. Frequency response is a fixed
    /// gaussian around her profile's center.
    /// </summary>
    public sealed class Session
    {
        /// <summary>Deepest tier in any heroine's kink ladder.</summary>
        public const int MaxTier = 4;

        /// <summary>Most Surface Beats one session may hold.</summary>
        public const int MaxSurfacesPerSession = 12;

        // Fraction of Response kept across a surface. Surfacing
        // shallows trance; it never zeroes it — she becomes aware
        // inside an unchanged situation.
        private const double SurfaceRetention = 0.45;

        // Composure strain added by a surface: the fluster is a
        // pacing spike the player should respect, never a fail.
        private const double SurfaceStrain = 0.35;

        // Response multiplier on the action that re-enters trance:
        // the second descent is faster than the first.
        private const double ReEntryResponseFactor = 1.5;

        // Corruption bonus when the resume action is kink-aligned
        // AND initiated by her: she chose to go back under, which is
        // kink-aligned self-recognition (SCENARIO.md phase 4 to 5).
        private const double SelfRecognitionBonus = 4.0;

        // Corruption thresholds for tiers 1..4.
        private static readonly double[] TierThresholds = { 20.0, 40.0, 60.0, 80.0 };

        private readonly HeroineProfile _profile;
        private readonly HeroineState _state;
        private readonly int[] _sessionTrackXp = new int[4];

        public Session(HeroineProfile profile, HeroineState state)
        {
            _profile = profile ?? throw new ArgumentNullException(nameof(profile));
            _state = state ?? throw new ArgumentNullException(nameof(state));
            if (!string.Equals(profile.Id, state.HeroineId, StringComparison.Ordinal))
            {
                throw new ArgumentException(
                    "Profile and state are for different heroines.", nameof(state));
            }

            Corruption = state.CorruptionBaseline;
        }

        /// <summary>Her trance/arousal depth right now, 0..100.</summary>
        public double Response { get; private set; }

        /// <summary>Highest Response reached this session.</summary>
        public double ResponsePeak { get; private set; }

        /// <summary>Her corruption to go deeper, 0..100. The progression bar.</summary>
        public double Corruption { get; private set; }

        /// <summary>Pacing readout, 0..1. High strain = slow down. Never a fail.</summary>
        public double ComposureStrain { get; private set; }

        /// <summary>Deepest tier unlocked this session (sticky once unlocked).</summary>
        public int CurrentTier { get; private set; }

        public SessionEndReason EndReason { get; private set; }

        /// <summary>True after a comfort-limit ending: the aftercare beat is owed.</summary>
        public bool AftercareRequired => EndReason == SessionEndReason.ComfortLimit;

        /// <summary>
        /// True between Surface and ResumeTrance: she is out of
        /// trance and aware. Ordinary actions refuse in this state;
        /// the session itself is alive and may also be completed.
        /// </summary>
        public bool IsSurfaced { get; private set; }

        /// <summary>Surface Beats played so far this session.</summary>
        public int SurfacesThisSession { get; private set; }

        /// <summary>
        /// True when she is asking for the next tier herself: her
        /// stage is Craving+ and her corruption already covers the
        /// next threshold. The presentation layer surfaces this as
        /// her request (A2: at Craving she initiates; at Self-Driven
        /// she requests in advance).
        /// </summary>
        public bool HeroineRequestsNextTier
        {
            get
            {
                if (EndReason != SessionEndReason.None || CurrentTier >= MaxTier)
                {
                    return false;
                }

                return _state.Stage >= W5Stage.Craving
                    && Corruption >= TierThresholds[CurrentTier];
            }
        }

        /// <summary>The tier her current corruption would unlock.</summary>
        public int TierAllowedByCorruption
        {
            get
            {
                var allowed = 0;
                for (var i = 0; i < TierThresholds.Length; i++)
                {
                    if (Corruption >= TierThresholds[i])
                    {
                        allowed = i + 1;
                    }
                }

                return allowed;
            }
        }

        /// <summary>Frequency response multiplier for a dial position, 0.35..1.</summary>
        public double FrequencyFactor(double dialPosition)
        {
            var delta = dialPosition - _profile.FrequencyCenter;
            var gaussian = Math.Exp(-(delta * delta) / (2.0 * 0.18 * 0.18));
            return 0.35 + 0.65 * gaussian;
        }

        /// <summary>
        /// Bring her out of trance for one Surface Beat. Response
        /// shallows by the retention factor and composure strain
        /// spikes; corruption is untouched and the session does not
        /// end. The reaction is authored per heroine and supplied by
        /// the caller; the presentation layer must play it (A2 law:
        /// the beat is signaled BY HER — her line, her face).
        /// </summary>
        public SurfaceResult Surface(SurfaceTrigger trigger, SurfaceReaction reaction)
        {
            if (reaction == null)
            {
                throw new ArgumentNullException(nameof(reaction));
            }

            if (EndReason != SessionEndReason.None)
            {
                throw new InvalidOperationException("The session has ended.");
            }

            if (IsSurfaced)
            {
                throw new InvalidOperationException(
                    "She is already surfaced; resume her trance first.");
            }

            if (SurfacesThisSession >= MaxSurfacesPerSession)
            {
                throw new InvalidOperationException(
                    "This session's Surface Beats are exhausted.");
            }

            var before = Response;
            Response = Clamp100(Response * SurfaceRetention);
            ComposureStrain = Math.Min(1.0, ComposureStrain + SurfaceStrain);
            IsSurfaced = true;
            SurfacesThisSession++;
            return new SurfaceResult(Response - before, reaction, SurfacesThisSession);
        }

        /// <summary>
        /// Resume play from a surface: the action that takes her back
        /// under. It lands at the re-entry response factor (the second
        /// descent is faster), and a kink-aligned action she initiated
        /// earns the self-recognition corruption bonus — she chose to
        /// go back. Every ordinary rule still applies, including the
        /// comfort limit on an overreaching resume.
        /// </summary>
        public ActionResult ResumeTrance(SessionAction action)
        {
            if (action == null)
            {
                throw new ArgumentNullException(nameof(action));
            }

            if (EndReason != SessionEndReason.None)
            {
                throw new InvalidOperationException("The session has ended.");
            }

            if (!IsSurfaced)
            {
                throw new InvalidOperationException(
                    "She is not surfaced; there is no trance to resume.");
            }

            IsSurfaced = false;
            var bonus = action.KinkAligned && action.InitiatedByHer
                ? SelfRecognitionBonus
                : 0.0;
            return ApplyCore(action, ReEntryResponseFactor, bonus);
        }

        /// <summary>Apply one action. See the class remarks for the rules.</summary>
        public ActionResult ApplyAction(SessionAction action)
        {
            if (action == null)
            {
                throw new ArgumentNullException(nameof(action));
            }

            if (EndReason != SessionEndReason.None)
            {
                throw new InvalidOperationException("The session has ended.");
            }

            if (IsSurfaced)
            {
                throw new InvalidOperationException(
                    "She is surfaced; resume her trance or complete the session.");
            }

            return ApplyCore(action, 1.0, 0.0);
        }

        private ActionResult ApplyCore(
            SessionAction action, double responseMultiplier, double corruptionBonus)
        {

            // Pushing past her corruption: her comfort limit ends the
            // session gracefully. Nothing is rolled back.
            if (action.Tier > CurrentTier && action.Tier > TierAllowedByCorruption)
            {
                EndReason = SessionEndReason.ComfortLimit;
                ComposureStrain = 1.0;
                return new ActionResult(0.0, 0.0, -1, false, false, false, true);
            }

            var factor = FrequencyFactor(action.DialPosition);

            // Response: any action lands; aligned and on-frequency lands best.
            var responseDelta = 12.0 * factor
                * (action.KinkAligned ? 1.0 : 0.6)
                * responseMultiplier;
            Response = Clamp100(Response + responseDelta);
            if (Response > ResponsePeak)
            {
                ResponsePeak = Response;
            }

            // Corruption: only kink-aligned action raises it. Rushing
            // still counts when aligned (she likes it; pacing costs),
            // but buys less. Off-kink action always costs.
            double corruptionDelta;
            if (action.KinkAligned && !action.Rushed)
            {
                corruptionDelta = 8.0 * factor;
            }
            else if (action.KinkAligned)
            {
                corruptionDelta = 3.0;
            }
            else if (action.Rushed)
            {
                corruptionDelta = -10.0;
            }
            else
            {
                corruptionDelta = -6.0;
            }

            corruptionDelta += corruptionBonus;
            Corruption = Clamp100(Corruption + corruptionDelta);

            // Composure strain: a pacing cue only.
            if (action.Rushed || !action.KinkAligned)
            {
                ComposureStrain = Math.Min(1.0, ComposureStrain + 0.3);
            }
            else
            {
                ComposureStrain = Math.Max(0.0, ComposureStrain - 0.15);
            }

            // Tier unlock (sticky). Response alone never unlocks: the
            // corruption threshold is the only key.
            var tierUnlocked = -1;
            var allowed = TierAllowedByCorruption;
            if (allowed > CurrentTier)
            {
                CurrentTier = allowed;
                tierUnlocked = allowed;
            }

            // Frequency discovery: a near-center aligned action finds it.
            var discovered = false;
            if (action.KinkAligned
                && !_state.FrequencyLearned
                && Math.Abs(action.DialPosition - _profile.FrequencyCenter) <= 0.05)
            {
                _state.LearnFrequency();
                discovered = true;
            }

            // Track XP: aligned play only, diminishing within a session.
            var capstone = false;
            if (action.KinkAligned)
            {
                var slot = (int)action.Track;
                var award = Math.Max(1, 6 - _sessionTrackXp[slot] / 15);
                _sessionTrackXp[slot] += award;
                capstone = _state.AwardTrackXp(action.Track, award);
            }

            return new ActionResult(
                responseDelta,
                corruptionDelta,
                tierUnlocked,
                tierUnlocked >= 0 && action.InitiatedByHer,
                discovered,
                capstone,
                false);
        }

        /// <summary>
        /// End the session as completed: records the clear (hypnosis
        /// count +1, stage re-derived), pays outfit coins from depth
        /// and response, and returns the summary.
        /// </summary>
        public SessionSummary Complete()
        {
            if (EndReason != SessionEndReason.None)
            {
                throw new InvalidOperationException("The session has already ended.");
            }

            EndReason = SessionEndReason.Completed;
            var coins = CurrentTier * 5 + (int)(ResponsePeak / 10.0);
            _state.AddOutfitCoins(coins);
            var stage = Progression.RecordClear(_state);
            return new SessionSummary(
                CurrentTier, ResponsePeak, coins, stage, SurfacesThisSession);
        }

        private static double Clamp100(double value)
        {
            if (value < 0.0)
            {
                return 0.0;
            }

            return value > 100.0 ? 100.0 : value;
        }
    }
}
