using System;
using Strobe.Core;
using Xunit;

namespace Strobe.Core.Tests
{
    /// <summary>
    /// A2 session law (validation): corruption gates tiers, aligned
    /// play raises corruption, frequency is learnable, completion
    /// records the clear. Adversarial half: response alone must never
    /// unlock a tier, off-kink/rushed play must never raise
    /// corruption, pushing past her corruption must end the session
    /// gracefully with progress kept, and a dead session refuses work.
    /// </summary>
    public sealed class SessionTests
    {
        private static HeroineProfile Profile(double center = 0.5)
        {
            return new HeroineProfile(
                "test-heroine", KinkVectorKind.HiddenDesire, center, "desire", "block");
        }

        private static SessionAction Aligned(
            int tier = 0, double dial = 0.5, bool rushed = false, bool byHer = false)
        {
            return new SessionAction(ResponseTrack.Voice, tier, true, rushed, byHer, dial);
        }

        [Fact]
        public void AlignedPlay_RaisesResponseAndCorruption()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            var result = session.ApplyAction(Aligned());
            Assert.True(result.ResponseDelta > 0);
            Assert.True(result.CorruptionDelta > 0);
            Assert.Equal(SessionEndReason.None, session.EndReason);
        }

        [Fact]
        public void Corruption_CrossingThreshold_UnlocksTier()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            var unlocked = -1;
            for (var i = 0; i < 4 && unlocked < 0; i++)
            {
                unlocked = session.ApplyAction(Aligned()).TierUnlocked;
            }

            Assert.Equal(1, unlocked);
            Assert.Equal(1, session.CurrentTier);
        }

        [Fact]
        public void ResponseAlone_NeverUnlocksATier()
        {
            // Adversarial: maximum response, zero corruption growth —
            // off-kink actions still raise response at 0.6x. No tier
            // may open, because corruption is the only key.
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            for (var i = 0; i < 12; i++)
            {
                var result = session.ApplyAction(new SessionAction(
                    ResponseTrack.Voice, 0, false, false, false, 0.5));
                Assert.Equal(-1, result.TierUnlocked);
            }

            Assert.True(session.Response > 50.0);
            Assert.Equal(0, session.CurrentTier);
            Assert.True(session.Corruption <= 0.0);
        }

        [Fact]
        public void OffKinkAndRushedPlay_LowersCorruption()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            session.ApplyAction(Aligned());
            var before = session.Corruption;
            var result = session.ApplyAction(new SessionAction(
                ResponseTrack.Voice, 0, false, true, false, 0.5));
            Assert.True(result.CorruptionDelta < 0);
            Assert.True(session.Corruption < before);
            Assert.True(session.ComposureStrain > 0.0);
        }

        [Fact]
        public void PushingPastCorruption_EndsGracefully_ProgressKept()
        {
            var state = new HeroineState("test-heroine");
            var session = new Session(Profile(), state);
            session.ApplyAction(Aligned()); // earn some XP first
            var xpBefore = state.TrackXp(ResponseTrack.Voice);
            Assert.True(xpBefore > 0);

            var result = session.ApplyAction(Aligned(tier: Session.MaxTier));
            Assert.True(result.EndedByComfortLimit);
            Assert.Equal(SessionEndReason.ComfortLimit, session.EndReason);
            Assert.True(session.AftercareRequired);
            Assert.Equal(xpBefore, state.TrackXp(ResponseTrack.Voice));
            Assert.Equal(0, state.ClearCount); // no clear recorded
        }

        [Fact]
        public void EndedSession_RefusesFurtherActions()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            session.ApplyAction(Aligned(tier: Session.MaxTier));
            Assert.Throws<InvalidOperationException>(() => session.ApplyAction(Aligned()));
            Assert.Throws<InvalidOperationException>(() => session.Complete());
        }

        [Fact]
        public void Frequency_NearCenterAlignedAction_DiscoversIt()
        {
            var state = new HeroineState("test-heroine");
            var session = new Session(Profile(0.42), state);
            var result = session.ApplyAction(new SessionAction(
                ResponseTrack.Gaze, 0, true, false, false, 0.44));
            Assert.True(result.FrequencyDiscovered);
            Assert.True(state.FrequencyLearned);
        }

        [Fact]
        public void Frequency_OffCenterOrOffKink_NeverDiscovers()
        {
            var state = new HeroineState("test-heroine");
            var session = new Session(Profile(0.42), state);
            var far = session.ApplyAction(new SessionAction(
                ResponseTrack.Gaze, 0, true, false, false, 0.9));
            Assert.False(far.FrequencyDiscovered);
            var offKink = session.ApplyAction(new SessionAction(
                ResponseTrack.Gaze, 0, false, false, false, 0.42));
            Assert.False(offKink.FrequencyDiscovered);
            Assert.False(state.FrequencyLearned);
        }

        [Fact]
        public void TrackXp_DiminishesWithinASession_AndCapsAtOneMinimum()
        {
            var state = new HeroineState("test-heroine");
            var session = new Session(Profile(), state);
            var awards = new int[8];
            for (var i = 0; i < awards.Length; i++)
            {
                var before = state.TrackXp(ResponseTrack.Voice);
                session.ApplyAction(Aligned());
                awards[i] = state.TrackXp(ResponseTrack.Voice) - before;
            }

            Assert.Equal(6, awards[0]);
            Assert.True(awards[7] < awards[0]);
            Assert.True(awards[7] >= 1);
        }

        [Fact]
        public void Capstone_UnlocksThroughAlignedPlayOnly()
        {
            var state = new HeroineState("test-heroine");
            var session = new Session(Profile(), state);
            var capstone = false;
            for (var i = 0; i < 90 && !capstone; i++)
            {
                capstone = session.ApplyAction(Aligned()).CapstoneUnlocked;
            }

            Assert.True(capstone);
            Assert.True(state.HasCapstone(ResponseTrack.Voice));
        }

        [Fact]
        public void Complete_RecordsClearAndPaysCoins()
        {
            var state = new HeroineState("test-heroine");
            var session = new Session(Profile(), state);
            for (var i = 0; i < 4; i++)
            {
                session.ApplyAction(Aligned());
            }

            var summary = session.Complete();
            Assert.Equal(1, state.ClearCount);
            Assert.Equal(W5Stage.Curious, summary.StageAfter);
            Assert.True(summary.CoinsAwarded > 0);
            Assert.Equal(summary.CoinsAwarded, state.OutfitCoins);
            Assert.Equal(session.CurrentTier, summary.TierReached);
        }

        [Fact]
        public void HeroineRequestsNextTier_OnlyAtCravingWithCorruption()
        {
            var state = new HeroineState("test-heroine");
            state.RestoreClearCount(6); // Craving; baseline corruption 36
            var session = new Session(Profile(), state);
            Assert.True(session.HeroineRequestsNextTier);

            var early = new Session(Profile(), new HeroineState("test-heroine"));
            Assert.False(early.HeroineRequestsNextTier);
        }

        [Fact]
        public void Session_MismatchedProfileAndState_Throws()
        {
            var profile = new HeroineProfile(
                "someone-else", KinkVectorKind.DissolvedBlock, 0.5, "d", "b");
            Assert.Throws<ArgumentException>(
                () => new Session(profile, new HeroineState("test-heroine")));
        }

        [Fact]
        public void SessionAction_OutOfRangeArguments_Throw()
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => new SessionAction(
                ResponseTrack.Voice, Session.MaxTier + 1, true, false, false, 0.5));
            Assert.Throws<ArgumentOutOfRangeException>(() => new SessionAction(
                ResponseTrack.Voice, 0, true, false, false, 1.5));
        }
    }
}
