//! The tuning gate (SPEC-BEVY §8.2): in scripted focused play,
//! her authored frequency is discovered (and the Status fact
//! flips), and the Voice capstone is earnable inside the 10-day
//! season — with Protocols gaining its entry at the unlock.

mod common;

use strobe_core::{PhoneApp, ResponseTrack};
use strobe_game::sim::{LogEntry, SimInput};

use common::{act, core, input, rig};

fn short_session(app: &mut bevy_app::App, actions: &[&str]) {
    input(app, SimInput::RequestSession);
    for action in actions {
        act(app, action);
    }
    input(app, SimInput::CompleteSession);
    input(app, SimInput::AcknowledgeAftercare);
}

#[test]
fn frequency_discovery_and_voice_capstone_inside_the_season() {
    let mut rig = rig();

    // Discovery: her tier-0 listens sit inside the discovery
    // window and echo-her-phrase sits exactly on her center.
    input(&mut rig.app, SimInput::RequestSession);
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "echo-her-phrase");
    assert!(core(&rig.app).state.frequency_learned());
    assert!(
        core(&rig.app)
            .log
            .iter()
            .any(|e| matches!(e, LogEntry::FrequencyLearned))
    );
    input(&mut rig.app, SimInput::CompleteSession);
    input(&mut rig.app, SimInput::AcknowledgeAftercare);

    // Two more clears reach Willing, where Protocols installs.
    short_session(&mut rig.app, &["listen-with-her"]);
    short_session(&mut rig.app, &["listen-with-her"]);
    {
        let core = core(&rig.app);
        assert_eq!(strobe_core::W5Stage::Willing, core.state.stage());
        assert!(core.hub.is_installed(PhoneApp::Protocols));
        assert_eq!(1, core.hub.version_of(PhoneApp::Protocols));
    }

    // Focused Voice play earns the capstone inside the season.
    // Each session ramps tier 0 -> 1 on listens first (a cold
    // tier-1 action would trip the comfort limit, by design).
    let mut grind: Vec<&str> = vec!["listen-with-her"; 3];
    grind.extend(vec!["echo-her-phrase"; 37]);
    let mut sessions = 0;
    while !core(&rig.app).state.has_capstone(ResponseTrack::Voice) && sessions < 5 {
        short_session(&mut rig.app, &grind);
        sessions += 1;
    }
    let core = core(&rig.app);
    assert!(
        core.state.has_capstone(ResponseTrack::Voice),
        "capstone earned after {sessions} focused sessions; voice xp {}",
        core.state.track_xp(ResponseTrack::Voice)
    );
    assert!(core.clock.day() <= 10, "inside the 10-day season");
    // The capstone was logged with its authored content key, the
    // analytics event fired, and Protocols gained its entry.
    assert!(core.log.iter().any(|e| matches!(
        e,
        LogEntry::CapstoneUnlocked { track: ResponseTrack::Voice, content_key }
        if content_key == "vivienne.capstone.voice.sings_the_count"
    )));
    assert_eq!(2, core.hub.version_of(PhoneApp::Protocols));
    assert!(rig.sink.event_names().contains(&"strobe.capstone_unlocked"));
}
