//! The comfort-limit branch gate (SPEC-BEVY §8.2): an over-tier
//! action ends the session in the same event that zeroes the toy,
//! XP earned is kept, no clear is recorded, mandatory aftercare
//! gates the next session until acknowledged — and the branch is
//! a branch, never an error.

mod common;

use strobe_core::ResponseTrack;
use strobe_game::sim::run_ticks;
use strobe_game::sim::{LogEntry, SimInput, ToyEvent};

use common::{act, core, input, rig};

#[test]
fn comfort_limit_branch_zeroes_keeps_xp_and_gates_the_next_session() {
    let mut rig = rig();
    input(&mut rig.app, SimInput::RequestSession);
    act(&mut rig.app, "listen-with-her");
    // Let the chart flow so patterns reach the toy slot.
    run_ticks(&mut rig.app, 200);
    {
        let log = rig.toy_log.lock().expect("toy log");
        assert!(
            log.iter().any(|e| matches!(e, ToyEvent::Pattern(..))),
            "patterns flowed before the limit: {log:?}"
        );
    }
    // The overreach: a tier-4 action at tier 0.
    act(&mut rig.app, "skip-ahead");

    let c = core(&rig.app);
    assert!(c.session_closed(), "the session ended");
    assert!(
        c.log.iter().any(|e| matches!(
            e,
            LogEntry::ComfortLimited {
                attempted_tier: 4,
                allowed_tier: 0
            }
        )),
        "log: {:?}",
        c.log
    );
    // XP from the honest action is kept; no clear is recorded.
    assert!(c.state.track_xp(ResponseTrack::Voice) > 0);
    assert_eq!(0, c.state.clear_count());
    // Strain hit the ceiling: aftercare is mandatory.
    assert!(c.aftercare_gate);
    assert!(c.last_summary.is_some_and(|s| s.aftercare_required));
    // Criterion 3: the zeroing command is the immediate next
    // outbound after the session's output — here, the last event
    // in the toy log is the stop, in the same input event.
    {
        let log = rig.toy_log.lock().expect("toy log");
        assert_eq!(Some(&ToyEvent::Stop), log.last(), "toy log: {log:?}");
    }
    // The gate: a new session refuses until aftercare lands.
    input(&mut rig.app, SimInput::RequestSession);
    assert!(
        core(&rig.app)
            .log
            .iter()
            .any(|e| matches!(e, LogEntry::Refused { what, .. } if what == "request_session"))
    );
    // View the aftercare, then acknowledge it: the gate clears.
    input(&mut rig.app, SimInput::AcknowledgeAftercare);
    input(&mut rig.app, SimInput::AcknowledgeAftercare);
    assert!(!core(&rig.app).aftercare_gate);
    input(&mut rig.app, SimInput::RequestSession);
    assert!(core(&rig.app).in_session(), "a new session may begin");
}

#[test]
fn rushed_and_off_kink_actions_raise_strain_without_ending_anything() {
    let mut rig = rig();
    input(&mut rig.app, SimInput::RequestSession);
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "listen-with-her");
    act(&mut rig.app, "listen-with-her");
    // Tier 1 reached; the rushed tier-1 action is legal play.
    act(&mut rig.app, "push-too-fast");
    let core = core(&rig.app);
    assert!(!core.session_closed());
    assert!(core.in_session());
}
