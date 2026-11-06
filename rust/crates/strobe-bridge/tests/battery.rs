//! The bridge safety battery (SPEC §4 criteria 1–7, Bevy form in
//! SPEC-BEVY §8.1): every criterion as named tests against the
//! mock transport, split half validation / half adversarial in
//! estate idiom. The gate core takes `now_tick` from the test, so
//! the watchdog battery is deterministic — no test sleeps.
//!
//! Criterion 7 (tier clamp across a chart run) is a game-level
//! gate: it lives in strobe-game's D6 clamp test, where the chart
//! and `ToyCurve` meet. Everything else is here.

use strobe_bridge::gate::parse_pattern_rule;
use strobe_bridge::{
    BridgeState, Decision, DenyReason, Gate, GateConfig, MockTransport, Request, Transport,
    WireCommand,
};

/// One rig: a gate plus the mock it emits into, driven the way the
/// worker drives them (decide, then emit allowed commands).
struct Rig {
    gate: Gate,
    mock: MockTransport,
}

impl Rig {
    fn new() -> Rig {
        Rig::with_config(GateConfig::default())
    }

    fn with_config(config: GateConfig) -> Rig {
        Rig {
            gate: Gate::new(config),
            mock: MockTransport::new(),
        }
    }

    fn handle(&mut self, request: &Request, now_tick: u64) -> Decision {
        let decision = self.gate.decide(request, now_tick);
        if let Decision::Allow { commands } = &decision {
            for command in commands {
                self.mock
                    .send_command(command)
                    .expect("the mock accepts while connected");
            }
        }
        decision
    }

    fn tick(&mut self, now_tick: u64) {
        if let Some(command) = self.gate.tick(now_tick) {
            self.mock
                .send_command(&command)
                .expect("the mock accepts while connected");
        }
    }

    fn connect_and_arm(&mut self) {
        self.handle(&Request::Connect, 0);
        self.handle(&Request::Discover, 0);
        self.handle(&Request::Arm, 0);
    }

    fn log(&self) -> Vec<WireCommand> {
        self.mock.log()
    }
}

fn valid_pattern() -> Request {
    Request::SendPattern {
        strengths: vec![5; 10],
        step_ms: 250,
    }
}

// Validation -----------------------------------------------------

#[test]
fn armed_send_reaches_the_wire() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    let decision = rig.handle(&Request::Send { intensity: 12 }, 10);
    assert_eq!(
        Decision::Allow {
            commands: vec![WireCommand::Function { strength: 12 }]
        },
        decision
    );
    assert_eq!(vec![WireCommand::Function { strength: 12 }], rig.log());
    assert_eq!(BridgeState::Active, rig.gate.state());
}

#[test]
fn valid_pattern_reaches_the_wire() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    let decision = rig.handle(&valid_pattern(), 10);
    assert!(matches!(decision, Decision::Allow { .. }));
    assert_eq!(
        vec![WireCommand::Pattern {
            strengths: vec![5; 10],
            step_ms: 250
        }],
        rig.log()
    );
}

#[test]
fn discovery_is_unarmed_safe() {
    let mut rig = Rig::new();
    rig.handle(&Request::Connect, 0);
    rig.handle(&Request::Discover, 0);
    assert_eq!(BridgeState::Discovered, rig.gate.state());
    assert!(rig.log().is_empty(), "discovery never emits output");
}

#[test]
fn heartbeats_inside_the_window_keep_output_alive() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    rig.handle(&valid_pattern(), 10);
    for tick in (100..=500).step_by(100) {
        rig.handle(&Request::Heartbeat, tick);
        rig.tick(tick);
    }
    assert!(rig.gate.output_active());
    assert_eq!(BridgeState::Active, rig.gate.state());
    assert_eq!(1, rig.log().len(), "no zeroing command was emitted");
}

#[test]
fn stop_after_output_is_the_immediate_next_outbound() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    rig.handle(&valid_pattern(), 10);
    rig.handle(&Request::StopAll, 20);
    assert_eq!(
        vec![
            WireCommand::Pattern {
                strengths: vec![5; 10],
                step_ms: 250
            },
            WireCommand::StopAll,
        ],
        rig.log()
    );
    assert!(!rig.gate.output_active());
}

#[test]
fn pattern_rule_text_round_trips_the_wire_shape() {
    assert_eq!(Ok(250), parse_pattern_rule("V:1;F:v;S:250#"));
    let command = WireCommand::Pattern {
        strengths: vec![5; 10],
        step_ms: 250,
    };
    let json = command.to_wire_json();
    assert!(json.contains("\"rule\":\"V:1;F:v;S:250#\""), "json: {json}");
    assert!(json.contains("\"apiVer\":2"), "json: {json}");
}

// Adversarial ----------------------------------------------------

#[test]
fn criterion_1_disarmed_bridge_emits_nothing_under_any_session_event() {
    // Master OFF by default: tier unlocks, combo peaks, and the
    // Surface Beat are session events; none may produce output.
    let mut rig = Rig::new();
    rig.handle(&Request::Connect, 0);
    rig.handle(&Request::Discover, 0);
    for request in [
        Request::Send { intensity: 20 },
        valid_pattern(),
        Request::StopAll,
    ] {
        rig.handle(&request, 10);
    }
    assert!(rig.log().is_empty());
    assert_eq!(BridgeState::Discovered, rig.gate.state());
}

#[test]
fn criterion_2_armed_request_for_25_reaches_the_wire_as_20() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    rig.handle(&Request::Send { intensity: 25 }, 10);
    assert_eq!(vec![WireCommand::Function { strength: 20 }], rig.log());
}

#[test]
fn criterion_2_player_ceiling_wins_below_the_bridge_ceiling() {
    let mut rig = Rig::with_config(GateConfig {
        player_ceiling: 7,
        ..GateConfig::default()
    });
    rig.connect_and_arm();
    rig.handle(&Request::Send { intensity: 20 }, 10);
    assert_eq!(vec![WireCommand::Function { strength: 7 }], rig.log());
}

#[test]
fn criterion_6_pattern_over_50_strengths_is_refused() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    let decision = rig.handle(
        &Request::SendPattern {
            strengths: vec![5; 51],
            step_ms: 250,
        },
        10,
    );
    assert_eq!(
        Decision::Deny {
            reason: DenyReason::PatternTooLong
        },
        decision
    );
    assert!(rig.log().is_empty());
}

#[test]
fn criterion_6_pattern_step_at_or_below_100ms_is_refused() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    for step_ms in [100, 50] {
        let decision = rig.handle(
            &Request::SendPattern {
                strengths: vec![5; 10],
                step_ms,
            },
            10,
        );
        assert_eq!(
            Decision::Deny {
                reason: DenyReason::PatternStepTooShort
            },
            decision
        );
    }
    assert!(rig.log().is_empty());
}

#[test]
fn criterion_6_pattern_under_one_second_is_refused() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    let decision = rig.handle(
        &Request::SendPattern {
            strengths: vec![5; 3],
            step_ms: 250,
        },
        10,
    );
    assert_eq!(
        Decision::Deny {
            reason: DenyReason::PatternTooShort
        },
        decision
    );
    assert!(rig.log().is_empty());
}

#[test]
fn criterion_6_malformed_pattern_rules_are_refused() {
    for rule in [
        "",
        "garbage",
        "V:2;F:v;S:250#",
        "V:1;F:;S:250#",
        "V:1;F:v#",
        "V:1;F:v;S:fast#",
    ] {
        assert_eq!(
            Err(DenyReason::PatternMalformed),
            parse_pattern_rule(rule),
            "rule: {rule}"
        );
    }
}

#[test]
fn output_with_no_target_is_denied() {
    let mut rig = Rig::new();
    let decision = rig.handle(&Request::Send { intensity: 10 }, 0);
    assert_eq!(
        Decision::Deny {
            reason: DenyReason::NotConnected
        },
        decision
    );
    assert!(rig.log().is_empty());
}

#[test]
fn criterion_3_stop_mid_pattern_zeroes_with_nothing_intervening() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    rig.handle(&valid_pattern(), 10);
    rig.handle(&Request::StopAll, 11);
    let log = rig.log();
    assert_eq!(2, log.len());
    assert_eq!(WireCommand::StopAll, log[1]);
    assert!(!rig.gate.output_active());
}

#[test]
fn criterion_5_watchdog_expiry_mid_pattern_zeroes_and_locks() {
    let mut rig = Rig::with_config(GateConfig {
        watchdog_window_ticks: 600,
        ..GateConfig::default()
    });
    rig.connect_and_arm();
    rig.handle(&Request::Heartbeat, 0);
    rig.handle(&valid_pattern(), 10);
    rig.tick(500); // inside the window: still flowing
    assert!(rig.gate.output_active());
    rig.tick(601); // silence past 5 s of fixed steps: zeroed + locked
    assert!(!rig.gate.output_active());
    assert_eq!(BridgeState::WatchdogStopped, rig.gate.state());
    assert_eq!(WireCommand::StopAll, rig.log()[1]);
    // Locked: output refuses until a new explicit arm.
    let decision = rig.handle(&valid_pattern(), 700);
    assert_eq!(
        Decision::Deny {
            reason: DenyReason::WatchdogLocked
        },
        decision
    );
    rig.handle(&Request::Arm, 701);
    let decision = rig.handle(&valid_pattern(), 702);
    assert!(matches!(decision, Decision::Allow { .. }));
}

#[test]
fn criterion_4_session_drop_mid_pattern_cleans_up_and_never_resumes() {
    let mut rig = Rig::new();
    rig.connect_and_arm();
    rig.handle(&valid_pattern(), 10);
    assert!(rig.gate.output_active());

    rig.handle(&Request::Disconnect, 20);
    assert!(!rig.gate.output_active());
    assert_eq!(BridgeState::Disconnected, rig.gate.state());

    // Reconnect restores discovery only.
    rig.handle(&Request::Connect, 30);
    rig.handle(&Request::Discover, 30);
    assert_eq!(BridgeState::Discovered, rig.gate.state());
    let decision = rig.handle(&valid_pattern(), 40);
    assert_eq!(
        Decision::Deny {
            reason: DenyReason::NotArmed
        },
        decision
    );
    assert_eq!(
        1,
        rig.log().len(),
        "only the original pattern was ever emitted"
    );

    // Output resumes only after a new explicit arm + a new action.
    rig.handle(&Request::Arm, 50);
    let decision = rig.handle(&valid_pattern(), 60);
    assert!(matches!(decision, Decision::Allow { .. }));
    assert_eq!(2, rig.log().len());
}

#[test]
fn real_transport_speaks_the_section_4_wire_contract() {
    // A one-shot fake device on loopback: capture the request and
    // answer 200. The transport must produce exactly the §4 wire
    // shape: POST /command, JSON body, X-platform header.
    use std::io::{BufRead, BufReader, Read, Write};
    use strobe_bridge::transport::{RealTransport, RealTransportConfig};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().expect("local addr").port();
    let server = std::thread::spawn(move || {
        // The transport's connect() is a bare TCP probe: accept and
        // drop it, then take the real POST connection.
        let (probe, _) = listener.accept().expect("accept probe");
        drop(probe);
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut request_line = String::new();
        reader.read_line(&mut request_line).expect("request line");
        let mut headers = String::new();
        let mut content_length = 0_usize;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).expect("header line");
            if line == "\r\n" {
                break;
            }
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                content_length = value.trim().parse().expect("content length");
            }
            headers.push_str(&line);
        }
        let mut body = vec![0_u8; content_length];
        reader.read_exact(&mut body).expect("body");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
            .expect("respond");
        (
            request_line,
            headers,
            String::from_utf8(body).expect("utf8 body"),
        )
    });

    let config = RealTransportConfig {
        host: "127.0.0.1".to_string(),
        port,
        use_tls: false,
        platform: "Strobe".to_string(),
    };
    let mut transport = RealTransport::new(config).expect("build transport");
    transport.connect().expect("connect");
    transport
        .send_command(&WireCommand::Function { strength: 7 })
        .expect("send");
    drop(transport);
    let (request_line, headers, body) = server.join().expect("server thread");
    assert!(
        request_line.starts_with("POST /command "),
        "line: {request_line}"
    );
    assert!(
        headers.to_ascii_lowercase().contains("x-platform: strobe"),
        "headers: {headers}"
    );
    assert!(body.contains("\"command\":\"Function\""), "body: {body}");
    assert!(body.contains("\"vibrate\":7"), "body: {body}");
    assert!(body.contains("\"apiVer\":1"), "body: {body}");
}

#[test]
fn arm_disarm_race_as_a_scripted_interleaving() {
    let mut rig = Rig::new();
    rig.handle(&Request::Connect, 0);
    // Arm and a send race a disarm: the interleaving is decided by
    // call order against the pure gate — deterministic by design.
    rig.handle(&Request::Arm, 1);
    let allowed = rig.handle(&Request::Send { intensity: 9 }, 2);
    assert!(matches!(allowed, Decision::Allow { .. }));
    rig.handle(&Request::Disarm, 3);
    let denied = rig.handle(&Request::Send { intensity: 9 }, 4);
    assert_eq!(
        Decision::Deny {
            reason: DenyReason::NotArmed
        },
        denied
    );
    // Disarm zeroed the active output as its immediate command.
    let log = rig.log();
    assert_eq!(WireCommand::StopAll, log[log.len() - 1]);
    // A later explicit arm re-opens the gate.
    rig.handle(&Request::Arm, 5);
    let allowed = rig.handle(&Request::Send { intensity: 9 }, 6);
    assert!(matches!(allowed, Decision::Allow { .. }));
}
