using System;
using Strobe.Core;
using Xunit;

namespace Strobe.Core.Tests
{
    /// <summary>
    /// W3/W5 ladder: boundaries at every rung (validation), plus the
    /// adversarial edges — negative and extreme counts must fail safe
    /// to the ladder, never throw, never skip a stage.
    /// </summary>
    public sealed class ProgressionTests
    {
        [Theory]
        [InlineData(0, W5Stage.Unaware)]
        [InlineData(1, W5Stage.Curious)]
        [InlineData(2, W5Stage.Curious)]
        [InlineData(3, W5Stage.Willing)]
        [InlineData(5, W5Stage.Willing)]
        [InlineData(6, W5Stage.Craving)]
        [InlineData(9, W5Stage.Craving)]
        [InlineData(10, W5Stage.Devoted)]
        [InlineData(14, W5Stage.Devoted)]
        [InlineData(15, W5Stage.SelfDriven)]
        [InlineData(100, W5Stage.SelfDriven)]
        public void StageForCount_FollowsTheLadder(int count, W5Stage expected)
        {
            Assert.Equal(expected, Progression.StageForCount(count));
        }

        [Fact]
        public void StageForCount_NegativeCountsFailSafeToUnaware()
        {
            Assert.Equal(W5Stage.Unaware, Progression.StageForCount(-1));
            Assert.Equal(W5Stage.Unaware, Progression.StageForCount(int.MinValue));
        }

        [Fact]
        public void StageForCount_MaxIntStaysSelfDriven()
        {
            Assert.Equal(W5Stage.SelfDriven, Progression.StageForCount(int.MaxValue));
        }

        [Fact]
        public void ClearsToNextStage_CountsDownAndStopsAtTop()
        {
            Assert.Equal(1, Progression.ClearsToNextStage(0));
            Assert.Equal(2, Progression.ClearsToNextStage(1));
            Assert.Equal(1, Progression.ClearsToNextStage(14));
            Assert.Equal(0, Progression.ClearsToNextStage(15));
            Assert.Equal(0, Progression.ClearsToNextStage(999));
        }

        [Fact]
        public void RecordClear_AdvancesCountAndStageMonotonically()
        {
            var state = new HeroineState("test-heroine");
            var previous = W5Stage.Unaware;
            for (var i = 0; i < 20; i++)
            {
                var stage = Progression.RecordClear(state);
                Assert.True(stage >= previous, "Stage must never decrease.");
                previous = stage;
            }

            Assert.Equal(20, state.ClearCount);
            Assert.Equal(W5Stage.SelfDriven, state.Stage);
        }

        [Fact]
        public void RecordClear_NullStateThrows()
        {
            Assert.Throws<ArgumentNullException>(() => Progression.RecordClear(null));
        }

        [Fact]
        public void HeroineState_RestoreClearCount_ClampsNegativeToZero()
        {
            var state = new HeroineState("test-heroine");
            state.RestoreClearCount(-7);
            Assert.Equal(0, state.ClearCount);
            Assert.Equal(W5Stage.Unaware, state.Stage);
        }

        [Fact]
        public void HeroineState_StageIsDerived_NeverStoredSeparately()
        {
            var state = new HeroineState("test-heroine");
            state.RestoreClearCount(6);
            Assert.Equal(W5Stage.Craving, state.Stage);
            Assert.Equal(36.0, state.CorruptionBaseline);
            Assert.Equal(3, state.StageTierCeiling);
        }
    }
}
