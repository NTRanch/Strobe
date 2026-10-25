using System;

namespace Strobe.Core
{
    /// <summary>
    /// The six HYPNOSYNC phone apps (ADAPT-SPEC A1). The cap is law:
    /// exactly six, and exceeding it needs a new owner ruling. Every
    /// Strobe system lives behind one of these surfaces; progression
    /// unlocks arrive as app installs and version upgrades.
    /// </summary>
    public enum PhoneApp
    {
        Aftercare = 0,
        Feed = 1,
        Protocols = 2,
        Session = 3,
        Status = 4,
        Store = 5,
    }

    /// <summary>
    /// Phone hub state: which apps are installed, and each app's
    /// version. Versions are the diegetic form of progression —
    /// Protocols v2 is a capstone arriving as a software update.
    /// </summary>
    public sealed class PhoneHub
    {
        /// <summary>The hard app cap (A1). Six. No seventh app.</summary>
        public const int AppCount = 6;

        private readonly bool[] _installed = new bool[AppCount];
        private readonly int[] _versions = new int[AppCount];

        public PhoneHub()
        {
            // The hub ships with the session surface and her status:
            // the two things the fiction needs from minute one.
            Install(PhoneApp.Session);
            Install(PhoneApp.Status);
        }

        /// <summary>Install an app (idempotent). New installs start at v1.</summary>
        public void Install(PhoneApp app)
        {
            var slot = SlotOf(app);
            if (!_installed[slot])
            {
                _installed[slot] = true;
                _versions[slot] = 1;
            }
        }

        /// <summary>True when the app is on the phone.</summary>
        public bool IsInstalled(PhoneApp app)
        {
            return _installed[SlotOf(app)];
        }

        /// <summary>App version; 0 when not installed.</summary>
        public int VersionOf(PhoneApp app)
        {
            return _versions[SlotOf(app)];
        }

        /// <summary>
        /// Bump an installed app's version (a progression upgrade
        /// arriving as an update). Throws when the app is absent:
        /// upgrading what was never installed is a programmer error.
        /// </summary>
        public void Upgrade(PhoneApp app)
        {
            var slot = SlotOf(app);
            if (!_installed[slot])
            {
                throw new InvalidOperationException(
                    "Cannot upgrade an app that is not installed: " + app);
            }

            _versions[slot]++;
        }

        /// <summary>
        /// The A1 unlock schedule, applied from heroine state: Feed
        /// arrives at Curious, Store and Protocols at Willing,
        /// Aftercare after her first comfort-limit or at Craving —
        /// whichever the campaign reaches first — and Protocols'
        /// version tracks her capstone count + 1.
        /// </summary>
        public void SyncFromState(HeroineState state, bool aftercareOwed)
        {
            if (state == null)
            {
                throw new ArgumentNullException(nameof(state));
            }

            if (state.Stage >= W5Stage.Curious)
            {
                Install(PhoneApp.Feed);
            }

            if (state.Stage >= W5Stage.Willing)
            {
                Install(PhoneApp.Store);
                Install(PhoneApp.Protocols);
            }

            if (aftercareOwed || state.Stage >= W5Stage.Craving)
            {
                Install(PhoneApp.Aftercare);
            }

            if (IsInstalled(PhoneApp.Protocols))
            {
                var target = state.Capstones.Count + 1;
                while (VersionOf(PhoneApp.Protocols) < target)
                {
                    Upgrade(PhoneApp.Protocols);
                }
            }
        }

        private static int SlotOf(PhoneApp app)
        {
            var slot = (int)app;
            if (slot < 0 || slot >= AppCount)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(app), app, "Unknown phone app.");
            }

            return slot;
        }
    }
}
