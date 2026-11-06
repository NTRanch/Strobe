//! Persisted progress (SPEC-BEVY G7-B): a serde DTO in the game
//! crate — Core carries no persistence code. The snapshot stores
//! only reconstructible facts; live objects are rebuilt by
//! construction and the rest is replay (hub installs and versions
//! through `install_app` / `upgrade_app`, the clock through slot
//! spends), so a restored game has passed through the same code
//! paths as a played one.
//!
//! A save that fails to parse or validate is never overwritten or
//! deleted: the game starts fresh and the bad file is preserved
//! beside the save slot as `<name>.corrupt`, surfaced in the hub.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strobe_core::{CampaignClock, CoreError, HeroineState, PhoneApp, PhoneHub, ResponseTrack};

use crate::data::GameData;

/// The current snapshot schema version.
pub const SAVE_SCHEMA_VERSION: u32 = 1;

/// The persisted progress DTO.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SaveSnapshot {
    /// Snapshot schema version.
    pub schema_version: u32,
    /// The heroine this progress belongs to.
    pub heroine_id: String,
    /// Completed sessions (drives the stage).
    pub clear_count: i32,
    /// Per-track XP, indexed by `ResponseTrack` discriminant.
    pub track_xp: [i32; 4],
    /// Outfit coins banked.
    pub outfit_coins: i32,
    /// Whether her frequency has been discovered.
    pub frequency_learned: bool,
    /// Hub installs, indexed by `PhoneApp` discriminant.
    pub hub_installs: [bool; 6],
    /// Hub content versions, indexed by `PhoneApp` discriminant.
    pub hub_versions: [i32; 6],
    /// The clock's day when saved.
    pub clock_day: i32,
    /// The clock's slot within the day when saved.
    pub clock_slot: i32,
    /// The season this progress belongs to.
    pub season_id: String,
}

/// The live objects a snapshot restores into.
pub struct RestoredGame {
    /// Her progression state.
    pub state: HeroineState,
    /// The hub, rebuilt by replay.
    pub hub: PhoneHub,
    /// The clock, rebuilt by replay.
    pub clock: CampaignClock,
}

/// How a save load ended.
pub enum LoadOutcome {
    /// No save file exists: a fresh game.
    Fresh,
    /// A save was restored.
    Restored(Box<RestoredGame>),
    /// The save was corrupt: fresh game, file preserved as
    /// `.corrupt`, and the flag the hub surfaces.
    CorruptPreserved {
        /// The path the corrupt bytes were preserved at.
        preserved_path: PathBuf,
    },
}

/// Capture the live objects into a snapshot.
#[must_use]
pub fn snapshot(
    data: &GameData,
    state: &HeroineState,
    hub: &PhoneHub,
    clock: &CampaignClock,
) -> SaveSnapshot {
    let mut track_xp = [0_i32; 4];
    for track in ResponseTrack::ALL {
        track_xp[track.discriminant() as usize] = state.track_xp(track);
    }
    let mut hub_installs = [false; 6];
    let mut hub_versions = [0_i32; 6];
    for app in PhoneApp::ALL {
        hub_installs[app.discriminant() as usize] = hub.is_installed(app);
        hub_versions[app.discriminant() as usize] = hub.version_of(app);
    }
    SaveSnapshot {
        schema_version: SAVE_SCHEMA_VERSION,
        heroine_id: data.profile.id().to_string(),
        clear_count: state.clear_count(),
        track_xp,
        outfit_coins: state.outfit_coins(),
        frequency_learned: state.frequency_learned(),
        hub_installs,
        hub_versions,
        clock_day: clock.day(),
        clock_slot: clock.slot(),
        season_id: data.season_id.clone(),
    }
}

/// Write a snapshot to the save path (pretty JSON, stable key
/// order via serde struct order).
pub fn write_save(path: &Path, snapshot: &SaveSnapshot) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    std::fs::write(path, json)
}

/// Load the save at `path`, restoring by construction + replay.
/// Corrupt input is preserved, never destroyed (G7-B).
pub fn load_save(path: &Path, data: &GameData) -> LoadOutcome {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return LoadOutcome::Fresh,
        Err(_) => return preserve_corrupt(path),
    };
    let parsed: Result<SaveSnapshot, _> = serde_json::from_str(&raw);
    let snapshot = match parsed {
        Ok(snapshot) => snapshot,
        Err(_) => return preserve_corrupt(path),
    };
    match restore(&snapshot, data) {
        Ok(restored) => LoadOutcome::Restored(Box::new(restored)),
        Err(_) => preserve_corrupt(path),
    }
}

fn preserve_corrupt(path: &Path) -> LoadOutcome {
    let mut preserved = path.as_os_str().to_owned();
    preserved.push(".corrupt");
    let preserved_path = PathBuf::from(preserved);
    let _ = std::fs::rename(path, &preserved_path);
    LoadOutcome::CorruptPreserved { preserved_path }
}

/// Restore live objects from a snapshot: state by construction
/// plus Core's restore setters (which clamp and re-derive), hub
/// and clock by replay through the same calls live play uses.
pub fn restore(snapshot: &SaveSnapshot, data: &GameData) -> Result<RestoredGame, CoreError> {
    if snapshot.schema_version != SAVE_SCHEMA_VERSION
        || snapshot.heroine_id != data.profile.id()
        || snapshot.season_id != data.season_id
    {
        return Err(CoreError::Argument {
            param: "snapshot",
            message: "Snapshot does not belong to this heroine/season/schema.",
        });
    }
    // State by construction + restore setters (Core's load path:
    // values clamp at 0, capstones re-derive from XP).
    let mut state = HeroineState::new(&snapshot.heroine_id)?;
    state.restore_clear_count(snapshot.clear_count);
    for track in ResponseTrack::ALL {
        state.restore_track_xp(track, snapshot.track_xp[track.discriminant() as usize]);
    }
    state.restore_outfit_coins(snapshot.outfit_coins);
    state.restore_frequency_learned(snapshot.frequency_learned);

    // Hub by replay: recorded installs first, then the same sync
    // live play runs at every completion, then version bumps
    // through the live upgrade path. Any divergence from the
    // recorded hub fails the restore (the save reads as corrupt).
    let mut hub = PhoneHub::new();
    for app in PhoneApp::ALL {
        let index = app.discriminant() as usize;
        if snapshot.hub_installs[index] && !hub.is_installed(app) {
            hub.install(app);
        }
    }
    hub.sync_from_state(&state);
    for app in PhoneApp::ALL {
        let index = app.discriminant() as usize;
        let target = snapshot.hub_versions[index];
        let mut guard = 0;
        while hub.version_of(app) < target && guard <= 64 {
            hub.upgrade(app)?;
            guard += 1;
        }
        if hub.version_of(app) != target || hub.is_installed(app) != snapshot.hub_installs[index] {
            return Err(CoreError::Argument {
                param: "snapshot",
                message: "Hub replay diverged from the recorded hub.",
            });
        }
    }

    // Clock by replay: construct, then spend the recorded slots.
    let mut clock = CampaignClock::new(data.season.total_days, data.season.slots_per_day)?;
    let total_spends = (snapshot.clock_day - 1) * data.season.slots_per_day + snapshot.clock_slot;
    if total_spends < 0 {
        return Err(CoreError::Argument {
            param: "snapshot",
            message: "Clock position is negative.",
        });
    }
    for _ in 0..total_spends {
        clock.spend_slot();
    }
    if clock.day() != snapshot.clock_day || clock.slot() != snapshot.clock_slot {
        return Err(CoreError::Argument {
            param: "snapshot",
            message: "Clock replay diverged from the recorded clock.",
        });
    }
    Ok(RestoredGame { state, hub, clock })
}
