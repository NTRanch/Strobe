//! The phone hub: which of the six HYPNOSYNC apps are installed,
//! and each app's version. Versions are the diegetic form of
//! progression — Protocols v2 is a capstone arriving as a software
//! update.

use crate::error::CoreError;
use crate::heroine_state::HeroineState;
use crate::w5_stage::W5Stage;

/// The six HYPNOSYNC phone apps (ADAPT-SPEC A1). The cap is law:
/// exactly six, and exceeding it needs a new owner ruling. Every
/// Strobe system lives behind one of these surfaces; progression
/// unlocks arrive as app installs and version upgrades.
/// Discriminants port verbatim from the C#.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum PhoneApp {
    /// The care surface: every session's closing beat.
    Aftercare = 0,
    /// Her stage-resolved feed beats.
    Feed = 1,
    /// Capstone installs and their reaction beats.
    Protocols = 2,
    /// The session surface.
    Session = 3,
    /// Her record: stage, tracks, frequency.
    Status = 4,
    /// The shelf.
    Store = 5,
}

impl PhoneApp {
    /// All six apps, in discriminant order.
    pub const ALL: [PhoneApp; 6] = [
        PhoneApp::Aftercare,
        PhoneApp::Feed,
        PhoneApp::Protocols,
        PhoneApp::Session,
        PhoneApp::Status,
        PhoneApp::Store,
    ];

    /// The app's contract discriminant (0..=5).
    #[must_use]
    pub const fn discriminant(self) -> i32 {
        self as i32
    }

    /// The app's slot in the hub's fixed arrays.
    #[must_use]
    pub const fn slot(self) -> usize {
        self as usize
    }

    /// Rebuild an app from a raw discriminant (save/data layer).
    ///
    /// # Errors
    /// [`CoreError::OutOfRange`] when `value` is not 0..=5.
    pub fn from_discriminant(value: i32) -> Result<PhoneApp, CoreError> {
        match value {
            0 => Ok(PhoneApp::Aftercare),
            1 => Ok(PhoneApp::Feed),
            2 => Ok(PhoneApp::Protocols),
            3 => Ok(PhoneApp::Session),
            4 => Ok(PhoneApp::Status),
            5 => Ok(PhoneApp::Store),
            _ => Err(CoreError::OutOfRange {
                param: "app",
                message: "Unknown phone app.",
            }),
        }
    }
}

/// Phone hub state: installs and versions for the six apps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhoneHub {
    installed: [bool; Self::APP_COUNT],
    versions: [i32; Self::APP_COUNT],
}

impl Default for PhoneHub {
    fn default() -> Self {
        Self::new()
    }
}

impl PhoneHub {
    /// The hard app cap (A1). Six. No seventh app.
    pub const APP_COUNT: usize = 6;

    /// A fresh hub. The hub ships with the session surface, her
    /// status, and the care surface: aftercare is infrastructure,
    /// not an unlock (Owner ruling 2026-10-25, decision 5A — it is
    /// the closing surface of every session, so it exists from
    /// minute one, not after her first comfort-limit).
    #[must_use]
    pub fn new() -> PhoneHub {
        let mut hub = PhoneHub {
            installed: [false; Self::APP_COUNT],
            versions: [0; Self::APP_COUNT],
        };
        hub.install(PhoneApp::Session);
        hub.install(PhoneApp::Status);
        hub.install(PhoneApp::Aftercare);
        hub
    }

    /// Install an app (idempotent). New installs start at v1.
    pub fn install(&mut self, app: PhoneApp) {
        let slot = app.slot();
        if !self.installed[slot] {
            self.installed[slot] = true;
            self.versions[slot] = 1;
        }
    }

    /// True when the app is on the phone.
    #[must_use]
    pub fn is_installed(&self, app: PhoneApp) -> bool {
        self.installed[app.slot()]
    }

    /// App version; 0 when not installed.
    #[must_use]
    pub fn version_of(&self, app: PhoneApp) -> i32 {
        self.versions[app.slot()]
    }

    /// Bump an installed app's version (a progression upgrade
    /// arriving as an update).
    ///
    /// # Errors
    /// [`CoreError::InvalidOperation`] when the app is absent:
    /// upgrading what was never installed is a programmer error.
    pub fn upgrade(&mut self, app: PhoneApp) -> Result<(), CoreError> {
        let slot = app.slot();
        if !self.installed[slot] {
            return Err(CoreError::InvalidOperation {
                message: "Cannot upgrade an app that is not installed.",
            });
        }
        self.versions[slot] += 1;
        Ok(())
    }

    /// The A1 unlock schedule, applied from heroine state: Feed
    /// arrives at Curious, Store and Protocols at Willing, and
    /// Protocols' version tracks her capstone count + 1. Aftercare
    /// is not on this schedule: it ships installed (Owner ruling
    /// 2026-10-25, decision 5A).
    pub fn sync_from_state(&mut self, state: &HeroineState) {
        if state.stage() >= W5Stage::Curious {
            self.install(PhoneApp::Feed);
        }
        if state.stage() >= W5Stage::Willing {
            self.install(PhoneApp::Store);
            self.install(PhoneApp::Protocols);
        }
        if self.is_installed(PhoneApp::Protocols) {
            let target = state.capstones().len() as i32 + 1;
            while self.version_of(PhoneApp::Protocols) < target {
                let result = self.upgrade(PhoneApp::Protocols);
                assert!(
                    result.is_ok(),
                    "protocols is installed; upgrade cannot refuse"
                );
            }
        }
    }
}
