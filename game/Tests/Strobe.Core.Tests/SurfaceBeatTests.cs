using System;
using Strobe.Core;
using Xunit;

namespace Strobe.Core.Tests
{
    /// <summary>
    /// Surface Beat law (SCENARIO.md). Validation: surfacing shallows
    /// response without ending anything, the authored reaction comes
    /// back untouched, re-entry lands faster, and her own initiated
    /// resume earns the self-recognition corruption bonus. Adversarial
    /// half: surfacing must never end a session, never require
    /// aftercare, never move corruption by itself; the state machine
    /// refuses out-of-order calls; the per-session cap holds; and an
    /// overreaching resume still meets the comfort limit.
    /// </summary>
    public sealed class SurfaceBeatTests
    {
        private static HeroineProfile Profile(double center = 0.5)
        {
            return new HeroineProfile(
                "test-heroine", KinkVectorKind.HiddenDesire, center, "desire", "block");
        }

        private static SessionAction Aligned(bool byHer = false, int tier = 0)
        {
            return new SessionAction(ResponseTrack.Voice, tier, true, false, byHer, 0.5);
        }

        private static SurfaceReaction Reaction()
        {
            return new SurfaceReaction(
                ReactionGrammar.FaceCover, "line.surface", "expr.surface");
        }

        private static Session DeepSession()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            session.ApplyAction(Aligned());
            session.ApplyAction(Aligned());
            session.ApplyAction(Aligned());
            return session;
        }

        [Fact]
        public void Surface_ShallowsResponse_AndMarksSurfaced()
        {
            var session = DeepSession();
            var corruptionBefore = session.Corruption;
            var result = session.Surface(SurfaceTrigger.NameSpoken, Reaction());
            Assert.True(session.IsSurfaced);
            Assert.Equal(1, session.SurfacesThisSession);
            Assert.Equal(-19.8, result.ResponseDelta, 3);
            Assert.Equal(16.2, session.Response, 3);
            Assert.Equal(corruptionBefore, session.Corruption);
            Assert.Equal(SessionEndReason.None, session.EndReason);
        }

        [Fact]
        public void Surface_ReturnsTheAuthoredReaction()
        {
            var session = DeepSession();
            var reaction = Reaction();
            var result = session.Surface(SurfaceTrigger.Kiss, reaction);
            Assert.Same(reaction, result.Reaction);
            Assert.Equal(ReactionGrammar.FaceCover, result.Reaction.Grammar);
            Assert.Equal("line.surface", result.Reaction.VoiceLineKey);
            Assert.Equal(1, result.SurfacesThisSession);
        }

        [Fact]
        public void Resume_LandsAtReEntryFactor()
        {
            var session = DeepSession();
            session.Surface(SurfaceTrigger.CommandWord, Reaction());
            var result = session.ResumeTrance(Aligned());
            Assert.False(session.IsSurfaced);
            Assert.Equal(18.0, result.ResponseDelta, 3);
        }

        [Fact]
        public void Resume_ByHer_Aligned_EarnsSelfRecognitionBonus()
        {
            var session = DeepSession();
            session.Surface(SurfaceTrigger.EyeContact, Reaction());
            var byHer = session.ResumeTrance(Aligned(byHer: true));
            Assert.Equal(12.0, byHer.CorruptionDelta, 3);

            var other = DeepSession();
            other.Surface(SurfaceTrigger.EyeContact, Reaction());
            var byHim = other.ResumeTrance(Aligned(byHer: false));
            Assert.Equal(8.0, byHim.CorruptionDelta, 3);
        }

        [Fact]
        public void Summary_CountsSurfaces()
        {
            var session = DeepSession();
            session.Surface(SurfaceTrigger.NameSpoken, Reaction());
            session.ResumeTrance(Aligned());
            session.Surface(SurfaceTrigger.CountToZero, Reaction());
            var summary = session.Complete();
            Assert.Equal(2, summary.Surfaces);
        }

        [Fact]
        public void Surface_AtZeroResponse_StillPlaysAndNeverEnds()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            var result = session.Surface(SurfaceTrigger.ToyStop, Reaction());
            Assert.Equal(0.0, result.ResponseDelta, 3);
            Assert.True(session.IsSurfaced);
            Assert.Equal(SessionEndReason.None, session.EndReason);
            Assert.False(session.AftercareRequired);
        }

        [Fact]
        public void Surface_TwiceWithoutResume_Throws()
        {
            var session = DeepSession();
            session.Surface(SurfaceTrigger.NameSpoken, Reaction());
            Assert.Throws<InvalidOperationException>(
                () => session.Surface(SurfaceTrigger.Kiss, Reaction()));
        }

        [Fact]
        public void Resume_WithoutSurface_Throws()
        {
            var session = DeepSession();
            Assert.Throws<InvalidOperationException>(() => session.ResumeTrance(Aligned()));
        }

        [Fact]
        public void ApplyAction_WhileSurfaced_Throws()
        {
            var session = DeepSession();
            session.Surface(SurfaceTrigger.NameSpoken, Reaction());
            Assert.Throws<InvalidOperationException>(() => session.ApplyAction(Aligned()));
            Assert.True(session.IsSurfaced);
        }

        [Fact]
        public void Surface_AndResume_AfterComplete_Throw()
        {
            var session = DeepSession();
            session.Complete();
            Assert.Throws<InvalidOperationException>(
                () => session.Surface(SurfaceTrigger.Kiss, Reaction()));
            Assert.Throws<InvalidOperationException>(() => session.ResumeTrance(Aligned()));
        }

        [Fact]
        public void Surface_NullReaction_Throws()
        {
            var session = DeepSession();
            Assert.Throws<ArgumentNullException>(
                () => session.Surface(SurfaceTrigger.Kiss, null));
        }

        [Fact]
        public void Surface_Cap_IsEnforced()
        {
            var session = DeepSession();
            for (var i = 0; i < Session.MaxSurfacesPerSession; i++)
            {
                session.Surface(SurfaceTrigger.CountToZero, Reaction());
                session.ResumeTrance(Aligned());
            }

            Assert.Equal(Session.MaxSurfacesPerSession, session.SurfacesThisSession);
            Assert.Throws<InvalidOperationException>(
                () => session.Surface(SurfaceTrigger.CountToZero, Reaction()));
        }

        [Fact]
        public void Resume_Overreach_StillMeetsComfortLimit()
        {
            var session = new Session(Profile(), new HeroineState("test-heroine"));
            session.ApplyAction(Aligned());
            session.Surface(SurfaceTrigger.NameSpoken, Reaction());
            var result = session.ResumeTrance(Aligned(tier: Session.MaxTier));
            Assert.True(result.EndedByComfortLimit);
            Assert.Equal(SessionEndReason.ComfortLimit, session.EndReason);
            Assert.True(session.AftercareRequired);
            Assert.False(session.IsSurfaced);
        }

        [Fact]
        public void Reaction_NullKeys_NormalizeToEmpty()
        {
            var reaction = new SurfaceReaction(ReactionGrammar.GazeAvert, null, null);
            Assert.Equal(string.Empty, reaction.VoiceLineKey);
            Assert.Equal(string.Empty, reaction.ExpressionKey);
        }
    }
}
