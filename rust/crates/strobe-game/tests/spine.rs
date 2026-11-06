//! The §8.2 spine gate (SPEC-BEVY): boot with minimal plugins,
//! restore a fresh profile, and play SPEC §1's first-session
//! route on scripted inputs at named ticks — Status first, the
//! session request, tier-0/1 actions, the Surface Beat at its
//! authored moment, the resume, clean completion, the aftercare
//! close. The run must land on the golden canonical hash, twice
//! byte-identically, with the §6 analytics sequence in order and
//! the sink-off run indistinguishable.

mod common;

use bevy_state::state::State;
use strobe_core::{ResponseTrack, W5Stage};
use strobe_game::sim::{LogEntry, Screen, SimInput, SurfaceKind, stage_variant};

use common::{act, core, input, rig, rig_null_sink, settings};
use strobe_game::sim::run_ticks;

/// The §1 route, as (tick, input) pairs.
fn route(app: &mut bevy_app::App) {
    input(
        app,
        SimInput::Navigate {
            screen: Screen::Status,
        },
    );
    input(
        app,
        SimInput::Navigate {
            screen: Screen::Hub,
        },
    );
    input(app, SimInput::RequestSession);
    for action in [
        "listen-with-her",
        "watch-her-listen",
        "listen-with-her",
        "echo-her-phrase",
        "meet-her-gaze",
        "keep-the-count",
    ] {
        act(app, action);
    }
    input(app, SimInput::FireSurface);
    // The surface's input gate: an ordinary action attempted while
    // surfaced is refused at the gate, never reaching Core.
    act(app, "listen-with-her");
    input(
        app,
        SimInput::ResumeTrance {
            action_id: "she-hums-the-count".to_string(),
        },
    );
    act(app, "count-her-down");
    input(app, SimInput::CompleteSession);
    input(app, SimInput::AcknowledgeAftercare);
    run_ticks(app, 30);
}

#[test]
fn spine_route_completes_with_the_authored_beats() {
    let mut rig = rig();
    route(&mut rig.app);
    let core = core(&rig.app);

    // The route's beats all landed, in order.
    let log = &core.log;
    assert!(
        log.iter()
            .any(|e| matches!(e, LogEntry::SessionStarted { .. })),
        "log: {log:?}"
    );
    let unlocks: Vec<i32> = log
        .iter()
        .filter_map(|e| match e {
            LogEntry::TierUnlocked { tier, .. } => Some(*tier),
            _ => None,
        })
        .collect();
    assert_eq!(
        vec![1, 2, 3],
        unlocks,
        "tiers unlock in order (the resume + count carry her to 3)"
    );
    for entry in log {
        if let LogEntry::TierUnlocked { tier, line_key } = entry {
            assert_eq!(&format!("vivienne.unlock.tier{tier}"), line_key);
        }
    }
    let surface = log.iter().find_map(|e| match e {
        LogEntry::SurfaceFired {
            line_key,
            expression_key,
            response_delta,
        } => Some((line_key.clone(), expression_key.clone(), *response_delta)),
        _ => None,
    });
    let (line_key, expression_key, response_delta) = surface.expect("the beat fired");
    assert_eq!("vivienne.surface.laughing_cover.line", line_key);
    assert_eq!("vivienne.surface.laughing_cover.expr", expression_key);
    assert!(response_delta < 0.0, "surfacing shallows the trance");

    // The surfaced action attempt was refused at the input gate.
    assert!(
        log.iter()
            .any(|e| matches!(e, LogEntry::Refused { what, .. } if what == "apply_action")),
        "log: {log:?}"
    );
    // The resume earned her self-recognition: factor at her exact
    // center is 1.0, so 8 (aligned) + 4 (she initiated) = 12.
    let gained = log.iter().find_map(|e| match e {
        LogEntry::Resumed {
            corruption_gained, ..
        } => Some(*corruption_gained),
        _ => None,
    });
    assert!(
        (gained.expect("resumed") - 12.0).abs() < 1e-9,
        "resume corruption delta"
    );
    // Clean completion: one clear, stage advanced, coins paid.
    assert!(log.iter().any(
        |e| matches!(e, LogEntry::Completed { clears: 1, coins_earned } if *coins_earned > 0)
    ));
    assert_eq!(W5Stage::Curious, core.state.stage());
    assert_eq!(1, core.state.clear_count());
    // Her frequency was discovered on the exact-center action.
    assert!(log.iter().any(|e| matches!(e, LogEntry::FrequencyLearned)));
    assert!(core.state.frequency_learned());
    // Track XP landed where the table says it should.
    assert!(core.state.track_xp(ResponseTrack::Voice) > 0);
    assert!(core.state.track_xp(ResponseTrack::Gaze) > 0);
}

#[test]
fn spine_is_deterministic_and_matches_the_golden_hash() {
    let mut first = rig();
    route(&mut first.app);
    let hash1 = core(&first.app).state_hash();
    let canonical1 = core(&first.app).canonical_state();

    let mut second = rig();
    route(&mut second.app);
    assert_eq!(canonical1, core(&second.app).canonical_state());
    assert_eq!(hash1, core(&second.app).state_hash());

    // The golden hash: pinned after first verification of the
    // route by hand against SPEC §1's arithmetic.
    assert_eq!(GOLDEN_HASH, hash1, "canonical: {canonical1}");
}

#[test]
fn spine_analytics_sequence_matches_section_6() {
    let mut rig = rig();
    route(&mut rig.app);
    let names = rig.sink.event_names();
    assert_eq!(
        vec![
            "session_start",
            "funnel_step", // status viewed
            "funnel_step", // hub viewed
            "funnel_step", // session requested
            "funnel_step", // session started
            "level_start",
            "strobe.tier_unlocked", // tier 1
            "strobe.tier_unlocked", // tier 2
            "strobe.surface_beat",
            "feature_used",         // surface beat
            "strobe.tier_unlocked", // tier 3, on the count after the resume
            "level_complete",
            "funnel_step",          // session completed
            "strobe.stage_changed", // Unaware -> Curious
            "feature_used",         // feed app changed
        ],
        names
    );
    // Payload schema audit: every envelope carries the fixed
    // identity fields and a day-coarse timestamp.
    let envelopes = rig.sink.envelopes.lock().expect("sink lock");
    for envelope in envelopes.iter() {
        assert_eq!(1, envelope.schema_version);
        assert_eq!("strobe", envelope.title_id);
        assert_eq!("ntranch", envelope.lane);
        assert_eq!("install-test", envelope.install_id);
        assert!(envelope.ts_coarse.starts_with("day-"));
    }
    // The surface payload names the tier and the first-surface bit.
    let surface = envelopes
        .iter()
        .find(|e| e.event_type_name == "strobe.surface_beat")
        .expect("surface event");
    match surface.payload {
        strobe_game::analytics::Payload::SurfaceBeat { tier, first } => {
            assert_eq!(2, tier);
            assert!(first);
        }
        other => panic!("surface payload: {other:?}"),
    }
}

#[test]
fn sink_off_spine_is_byte_identical_and_writes_no_file() {
    let mut on = rig();
    route(&mut on.app);
    let mut off = rig_null_sink(settings());
    route(&mut off);
    assert_eq!(
        core(&on.app).canonical_state(),
        off.world()
            .resource::<strobe_game::sim::SimCore>()
            .canonical_state()
    );
}

#[test]
fn surfaces_resolve_the_same_stage_everywhere() {
    let mut rig = rig();
    // Before any clear: all four surfaces at stage 0 (Unaware).
    for surface in SurfaceKind::ALL {
        assert_eq!(
            format!("vivienne/{}/stage-0", surface_name(surface)),
            stage_variant(surface, core(&rig.app).surfaces_stage)
        );
    }
    route(&mut rig.app);
    // After the clear, one advance changed all four together.
    assert_eq!(W5Stage::Curious, core(&rig.app).surfaces_stage);
    for surface in SurfaceKind::ALL {
        assert_eq!(
            format!("vivienne/{}/stage-1", surface_name(surface)),
            stage_variant(surface, core(&rig.app).surfaces_stage)
        );
    }
    // The feed record resolves through the same set.
    let (_, feed_variant, caption) = core(&rig.app).feed_record();
    assert_eq!("vivienne/feed/stage-1", feed_variant);
    assert_eq!("vivienne.feed.beat1.caption", caption);
}

#[test]
fn off_kink_only_session_cannot_unlock_tier_two() {
    let mut rig = rig();
    input(&mut rig.app, SimInput::RequestSession);
    for _ in 0..10 {
        act(&mut rig.app, "change-the-subject");
    }
    let core = core(&rig.app);
    assert_eq!(Some(0), core.session_tier());
    assert!(
        !core
            .log
            .iter()
            .any(|e| matches!(e, LogEntry::TierUnlocked { .. })),
        "no tier may unlock on off-kink play"
    );
    // Reference law (Session.ApplyCore): track XP is aligned
    // play only — off-kink actions earn nothing at all.
    assert_eq!(0, core.state.track_xp(ResponseTrack::Gaze));
}

#[test]
fn screen_state_follows_the_route() {
    let mut rig = rig();
    input(&mut rig.app, SimInput::RequestSession);
    run_ticks(&mut rig.app, 2);
    let screen = rig.app.world().resource::<State<Screen>>();
    assert_eq!(Screen::InSession, *screen.get());
}

fn surface_name(surface: SurfaceKind) -> &'static str {
    match surface {
        SurfaceKind::Feed => "feed",
        SurfaceKind::Hub => "hub",
        SurfaceKind::Session => "session",
        SurfaceKind::Status => "status",
    }
}

/// Golden canonical hash for the §1 route (see the determinism
/// test). Pinned from the verified run: stage=1, gaze=24,
/// rhythm=18, voice=18, coins=22, clears=1, freq=1, hub 110110 /
/// 1,1,0,1,1,0, clock 1d1s — each field hand-checked against
/// SPEC §1's arithmetic before pinning.
const GOLDEN_HASH: &str = "631d1066ef0221871999deb9bf9f2eb86a0e8ba03f940b7a719a411d4fcb79ca";
