//! The async seam (SPEC-BEVY §3.5): the worker thread, its
//! stop-priority drain, and the `ToyOutput` adapter's two-part
//! stop discharge. These tests poll the published snapshot with a
//! bounded wait — the *gate* battery is the sleep-free one; this
//! file only proves the seam around it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use strobe_bridge::worker::BridgeToyOutput;
use strobe_bridge::{BridgeHandle, BridgeState, GateConfig, MockTransport, Request, WireCommand};
use strobe_core::ToyOutput;

fn wait_until(mut condition: impl FnMut() -> bool, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn worker_drives_gate_and_transport_and_publishes_snapshots() {
    let mock = MockTransport::new();
    let log = mock.log_handle();
    let handle = BridgeHandle::spawn(GateConfig::default(), mock);
    handle.request(Request::Connect, 0);
    handle.request(Request::Discover, 0);
    handle.request(Request::Arm, 0);
    handle.request(Request::Send { intensity: 25 }, 1);
    wait_until(
        || handle.snapshot().commands_sent >= 1,
        "the send to reach the wire",
    );
    let snapshot = handle.snapshot();
    assert_eq!(BridgeState::Active, snapshot.state);
    assert!(snapshot.armed);
    {
        let log = log.lock().expect("log lock");
        assert_eq!(vec![WireCommand::Function { strength: 20 }], *log);
    }
    handle.shutdown();
}

#[test]
fn adapter_stop_zeroes_the_mirror_and_the_wire_in_order() {
    let mock = MockTransport::new();
    let log = mock.log_handle();
    let handle = Arc::new(BridgeHandle::spawn(GateConfig::default(), mock));
    handle.request(Request::Connect, 0);
    handle.request(Request::Arm, 0);
    let tick = Arc::new(|| 7_u64);
    let mut output = BridgeToyOutput::new(Arc::clone(&handle), tick);
    output.send(12);
    wait_until(|| handle.snapshot().commands_sent >= 1, "the send");

    output.stop_all();
    // Part 1: the local mirror is zero in the calling step.
    assert!(!output.mirror_open());
    // A send attempted while the mirror is zeroed never queues.
    output.send(20);
    // Part 2: the zeroing command is the immediate next outbound.
    wait_until(
        || {
            log.lock()
                .map(|log| log.len() >= 2 && log[1] == WireCommand::StopAll)
                .unwrap_or(false)
        },
        "the zeroing command as the next outbound",
    );
    {
        let log = log.lock().expect("log lock");
        assert_eq!(2, log.len(), "the mirrored send never reached the wire");
    }
    // A new action re-opens the mirror explicitly.
    output.reopen();
    output.send(3);
    wait_until(|| handle.snapshot().commands_sent >= 3, "the resumed send");
    drop(output);
    Arc::try_unwrap(handle)
        .map_err(|_| "handle still shared")
        .expect("sole owner")
        .shutdown();
}
