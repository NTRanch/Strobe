//! The bridge worker (SPEC-BEVY §3.5): a dedicated thread owning
//! the gate and the transport, fed by a command channel, so the
//! fixed-step simulation is never blocked by IO and never blocks
//! it. State snapshots are published through a shared cell for
//! the game's readout (B6).
//!
//! Stop priority: each time the worker wakes, it drains every
//! immediately-queued message and processes `StopAll` / `Disarm`
//! ahead of queued output, so a stop is never stuck behind the
//! output it exists to cancel. Combined with the adapter's local
//! mirror (below), this is how Core's synchronous `stop_all()`
//! contract is discharged across an async seam (SPEC-BEVY §3.5):
//! (1) the adapter zeroes its local gate mirror in the calling
//! step — no further send passes from that instant; (2) the
//! zeroing command is the immediate next outbound on the
//! transport.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use strobe_core::ToyOutput;

use crate::gate::{Gate, GateConfig, Request};
use crate::transport::Transport;

/// A point-in-time view of the bridge for the game (B6).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BridgeSnapshot {
    /// The published bridge state.
    pub state: crate::gate::BridgeState,
    /// True while output is flowing.
    pub output_active: bool,
    /// True while the master switch is on.
    pub armed: bool,
    /// Total wire commands emitted since boot.
    pub commands_sent: u64,
    /// The most recent transport failure, if any.
    pub last_error: Option<String>,
}

enum WorkerMessage {
    Handle { request: Request, now_tick: u64 },
    Tick { now_tick: u64 },
    Shutdown,
}

fn is_stop_priority(message: &WorkerMessage) -> bool {
    matches!(
        message,
        WorkerMessage::Handle {
            request: Request::StopAll | Request::Disarm,
            ..
        }
    )
}

/// The game-side handle to the bridge worker.
pub struct BridgeHandle {
    sender: Sender<WorkerMessage>,
    snapshot: Arc<Mutex<BridgeSnapshot>>,
    join: Option<JoinHandle<()>>,
}

impl BridgeHandle {
    /// Spawn the worker with the given gate configuration and
    /// transport. The bridge starts disconnected and disarmed;
    /// the game drives `Connect` / `Arm` explicitly.
    pub fn spawn<T: Transport + 'static>(config: GateConfig, transport: T) -> BridgeHandle {
        let (sender, receiver) = channel::<WorkerMessage>();
        let snapshot = Arc::new(Mutex::new(BridgeSnapshot::default()));
        let worker_snapshot = Arc::clone(&snapshot);
        let join = std::thread::spawn(move || {
            run_worker(receiver, Gate::new(config), transport, worker_snapshot);
        });
        BridgeHandle {
            sender,
            snapshot,
            join: Some(join),
        }
    }

    /// Queue one request at the caller's current tick.
    /// Fire-and-forget: denials and state land in the snapshot.
    pub fn request(&self, request: Request, now_tick: u64) {
        let _ = self
            .sender
            .send(WorkerMessage::Handle { request, now_tick });
    }

    /// Advance the worker's watchdog to the caller's current tick.
    pub fn tick(&self, now_tick: u64) {
        let _ = self.sender.send(WorkerMessage::Tick { now_tick });
    }

    /// The latest published snapshot.
    #[must_use]
    pub fn snapshot(&self) -> BridgeSnapshot {
        self.snapshot
            .lock()
            .map(|snapshot| snapshot.clone())
            .unwrap_or_default()
    }

    /// Shut the worker down and join it.
    pub fn shutdown(mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn run_worker<T: Transport>(
    receiver: Receiver<WorkerMessage>,
    mut gate: Gate,
    mut transport: T,
    snapshot: Arc<Mutex<BridgeSnapshot>>,
) {
    let mut commands_sent: u64 = 0;
    let mut last_error: Option<String> = None;
    'outer: while let Ok(first) = receiver.recv() {
        // Drain everything already queued and let stops jump the
        // queue (stable: relative order otherwise preserved).
        let mut batch = vec![first];
        while let Ok(extra) = receiver.try_recv() {
            batch.push(extra);
        }
        batch.sort_by_key(|message| !is_stop_priority(message));
        for message in batch {
            match message {
                WorkerMessage::Shutdown => break 'outer,
                WorkerMessage::Tick { now_tick } => {
                    if let Some(command) = gate.tick(now_tick) {
                        match transport.send_command(&command) {
                            Ok(()) => commands_sent += 1,
                            Err(error) => last_error = Some(error.to_string()),
                        }
                    }
                }
                WorkerMessage::Handle { request, now_tick } => {
                    match gate.decide(&request, now_tick) {
                        crate::gate::Decision::Allow { commands } => {
                            for command in &commands {
                                match transport.send_command(command) {
                                    Ok(()) => commands_sent += 1,
                                    Err(error) => {
                                        last_error = Some(error.to_string());
                                        // Any transport failure is a
                                        // drop: zero and disconnect.
                                        let _ = gate.decide(&Request::Disconnect, now_tick);
                                    }
                                }
                            }
                        }
                        crate::gate::Decision::Deny { .. } => {}
                    }
                }
            }
        }
        if let Ok(mut cell) = snapshot.lock() {
            *cell = BridgeSnapshot {
                state: gate.state(),
                output_active: gate.output_active(),
                armed: gate.armed(),
                commands_sent,
                last_error: last_error.clone(),
            };
        }
    }
}

/// The `ToyOutput` adapter the session driver holds (SPEC-BEVY
/// §3.5): Core's synchronous seam over the async bridge.
///
/// `stop_all()` discharges in two parts: it sets the local gate
/// mirror to zero *in the calling step* (a later `send` from an
/// already-queued action cannot pass — sends short-circuit while
/// the mirror is zeroed, until the driver explicitly re-opens the
/// mirror for a new action via [`BridgeToyOutput::reopen`]), and
/// it queues a `StopAll` the worker prioritizes ahead of queued
/// output, making the zeroing command the immediate next outbound.
pub struct BridgeToyOutput {
    handle: Arc<BridgeHandle>,
    mirror_open: Arc<AtomicBool>,
    tick_source: Arc<dyn Fn() -> u64 + Send + Sync>,
}

impl BridgeToyOutput {
    /// Build an adapter over a shared bridge handle. `tick_source`
    /// reads the caller's current fixed-step tick (the game's
    /// `SimTick`); the adapter never reads a wall clock.
    pub fn new(
        handle: Arc<BridgeHandle>,
        tick_source: Arc<dyn Fn() -> u64 + Send + Sync>,
    ) -> BridgeToyOutput {
        BridgeToyOutput {
            handle,
            mirror_open: Arc::new(AtomicBool::new(true)),
            tick_source,
        }
    }

    /// Re-open the local mirror for a new action (the driver calls
    /// this when a new session action legitimately resumes output
    /// after a stop — §4 criterion 4's "new session action").
    pub fn reopen(&self) {
        self.mirror_open.store(true, Ordering::SeqCst);
    }

    /// Queue a pattern burst (the chart player's only output
    /// shape, §4 B3). Denied patterns never reach the wire — the
    /// gate refuses them before the transport, and a zeroed local
    /// mirror refuses them before the queue.
    pub fn send_pattern(&self, strengths: &[i32], step_ms: u32) {
        if !self.mirror_open.load(Ordering::SeqCst) {
            return;
        }
        let now_tick = (self.tick_source)();
        self.handle.request(
            Request::SendPattern {
                strengths: strengths.to_vec(),
                step_ms,
            },
            now_tick,
        );
    }

    /// Connect and run discovery (never arms, never outputs).
    pub fn connect_and_discover(&self, now_tick: u64) {
        self.handle.request(Request::Connect, now_tick);
        self.handle.request(Request::Discover, now_tick);
    }

    /// Arm the bridge (an explicit player act, B5). Arming is an
    /// explicit re-enable, so it also re-opens the local mirror.
    pub fn arm(&self, now_tick: u64) {
        self.mirror_open.store(true, Ordering::SeqCst);
        self.handle.request(Request::Arm, now_tick);
    }

    /// Disarm: zero any active output and close the gate.
    pub fn disarm(&self, now_tick: u64) {
        self.mirror_open.store(false, Ordering::SeqCst);
        self.handle.request(Request::Disarm, now_tick);
    }

    /// Disconnect: output zeroed, gate closed, discovery required
    /// before any later output (criterion 4).
    pub fn disconnect(&self, now_tick: u64) {
        self.mirror_open.store(false, Ordering::SeqCst);
        self.handle.request(Request::Disconnect, now_tick);
    }

    /// Pump the worker's watchdog clock to the caller's tick.
    pub fn pump_tick(&self, now_tick: u64) {
        self.handle.tick(now_tick);
    }

    /// True while the local mirror passes sends.
    #[must_use]
    pub fn mirror_open(&self) -> bool {
        self.mirror_open.load(Ordering::SeqCst)
    }
}

impl ToyOutput for BridgeToyOutput {
    fn send(&mut self, intensity: i32) {
        if !self.mirror_open.load(Ordering::SeqCst) {
            return;
        }
        let now_tick = (self.tick_source)();
        self.handle.request(Request::Send { intensity }, now_tick);
    }

    fn stop_all(&mut self) {
        // Part 1: zero the local mirror in this step.
        self.mirror_open.store(false, Ordering::SeqCst);
        // Part 2: the zeroing command queues with stop priority.
        let now_tick = (self.tick_source)();
        self.handle.request(Request::StopAll, now_tick);
    }
}
