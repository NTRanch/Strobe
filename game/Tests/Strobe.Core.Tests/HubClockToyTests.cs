using System;
using Strobe.Core;
using Xunit;

namespace Strobe.Core.Tests
{
    /// <summary>
    /// A1 hub, A4 clock, and the toy intensity law. Validation: the
    /// unlock schedule, slot accounting, and the clamp arithmetic.
    /// Adversarial: no seventh app can exist, upgrades cannot precede
    /// installs, the season cannot overspend, and intensity can never
    /// exceed the tier ceiling, the player ceiling, or a switched-off
    /// master — regardless of the caller's curve value.
    /// </summary>
    public sealed class HubClockToyTests
    {
        [Fact]
        public void PhoneHub_ShipsWithExactlySessionAndStatus()
        {
            var hub = new PhoneHub();
            Assert.True(hub.IsInstalled(PhoneApp.Session));
            Assert.True(hub.IsInstalled(PhoneApp.Status));
            Assert.False(hub.IsInstalled(PhoneApp.Feed));
            Assert.False(hub.IsInstalled(PhoneApp.Store));
            Assert.False(hub.IsInstalled(PhoneApp.Protocols));
            Assert.False(hub.IsInstalled(PhoneApp.Aftercare));
            Assert.Equal(6, PhoneHub.AppCount);
            Assert.Equal(6, Enum.GetValues(typeof(PhoneApp)).Length);
        }

        [Fact]
        public void PhoneHub_UpgradeBeforeInstall_Throws()
        {
            var hub = new PhoneHub();
            Assert.Throws<InvalidOperationException>(() => hub.Upgrade(PhoneApp.Store));
        }

        [Fact]
        public void PhoneHub_SyncFromState_FollowsTheSchedule()
        {
            var hub = new PhoneHub();
            var state = new HeroineState("test-heroine");
            state.RestoreClearCount(3); // Willing
            hub.SyncFromState(state, aftercareOwed: false);
            Assert.True(hub.IsInstalled(PhoneApp.Feed));
            Assert.True(hub.IsInstalled(PhoneApp.Store));
            Assert.True(hub.IsInstalled(PhoneApp.Protocols));
            Assert.False(hub.IsInstalled(PhoneApp.Aftercare));
            Assert.Equal(1, hub.VersionOf(PhoneApp.Protocols));

            state.RestoreClearCount(6); // Craving
            hub.SyncFromState(state, aftercareOwed: false);
            Assert.True(hub.IsInstalled(PhoneApp.Aftercare));
        }

        [Fact]
        public void PhoneHub_ProtocolsVersion_TracksCapstones()
        {
            var hub = new PhoneHub();
            var state = new HeroineState("test-heroine");
            state.RestoreClearCount(3);
            state.RestoreTrackXp(ResponseTrack.Voice, 130);
            state.RestoreTrackXp(ResponseTrack.Gaze, 130);
            hub.SyncFromState(state, aftercareOwed: false);
            Assert.Equal(3, hub.VersionOf(PhoneApp.Protocols)); // 2 capstones + 1
        }

        [Fact]
        public void CampaignClock_SpendsSlotsAndRollsDays()
        {
            var clock = new CampaignClock(2, 4);
            for (var i = 0; i < 4; i++)
            {
                Assert.True(clock.SpendSlot());
            }

            Assert.Equal(2, clock.Day);
            Assert.Equal(4, clock.SlotsLeftToday);
            for (var i = 0; i < 4; i++)
            {
                Assert.True(clock.SpendSlot());
            }

            Assert.True(clock.SeasonOver);
            Assert.False(clock.SpendSlot()); // cannot overspend the season
            Assert.Equal(0, clock.SlotsLeftToday);
        }

        [Theory]
        [InlineData(W5Stage.SelfDriven, SeasonEnding.SelfDriven)]
        [InlineData(W5Stage.Devoted, SeasonEnding.Deepened)]
        [InlineData(W5Stage.Craving, SeasonEnding.Deepened)]
        [InlineData(W5Stage.Willing, SeasonEnding.Begun)]
        [InlineData(W5Stage.Curious, SeasonEnding.Begun)]
        [InlineData(W5Stage.Unaware, SeasonEnding.NotYet)]
        public void CampaignClock_EndingsAreSoftAndStageKeyed(
            W5Stage stage, SeasonEnding expected)
        {
            Assert.Equal(expected, CampaignClock.EvaluateEnding(stage));
        }

        [Fact]
        public void CampaignClock_InvalidShape_Throws()
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => new CampaignClock(0, 4));
            Assert.Throws<ArgumentOutOfRangeException>(() => new CampaignClock(10, 0));
            Assert.Throws<ArgumentOutOfRangeException>(() => new CampaignClock(10, 99));
        }

        [Fact]
        public void ToyCurve_MasterOff_AlwaysZero()
        {
            Assert.Equal(0, ToyCurve.ComputeIntensity(false, 1.0, Session.MaxTier, 20));
            Assert.Equal(0, ToyCurve.ComputeIntensity(false, 0.5, 2, 20));
        }

        [Fact]
        public void ToyCurve_TierZero_IsSilent()
        {
            // Her first willing tier is the floor for any output.
            Assert.Equal(0, ToyCurve.ComputeIntensity(true, 1.0, 0, 20));
        }

        [Fact]
        public void ToyCurve_NeverExceedsTierOrPlayerCeiling()
        {
            // Tier 2 of 4 -> ceiling 10; player ceiling 7 wins.
            Assert.Equal(7, ToyCurve.ComputeIntensity(true, 1.0, 2, 7));
            // Full curve, full tier, full ceiling -> full scale.
            Assert.Equal(20, ToyCurve.ComputeIntensity(true, 1.0, 4, 20));
            // Half curve at tier 4 -> 10.
            Assert.Equal(10, ToyCurve.ComputeIntensity(true, 0.5, 4, 20));
        }

        [Fact]
        public void ToyCurve_HostileInputs_AreClampedNotTrusted()
        {
            Assert.Equal(20, ToyCurve.ComputeIntensity(true, 999.0, 99, 999));
            Assert.Equal(0, ToyCurve.ComputeIntensity(true, -5.0, 4, 20));
            Assert.Equal(0, ToyCurve.ComputeIntensity(true, 1.0, -3, 20));
            Assert.Equal(0, ToyCurve.ComputeIntensity(true, 1.0, 4, -1));
        }
    }
}
