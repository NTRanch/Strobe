//! The boot smoke binary (SPEC-BEVY §8.5): a real Bevy window
//! under a virtual display — `DefaultPlugins`, a camera, the hub
//! frame's stage-variant portrait quad and title text — runs a
//! bounded number of updates and exits 0. Pixel comparison is
//! explicitly NOT a gate (renderer nondeterminism); "the hub frame
//! rendered" is proven by the scene contents existing and the app
//! reaching `AppExit` cleanly.
//!
//! Usage: strobe-smoke --smoke <ticks>

#![forbid(unsafe_code)]

use bevy::prelude::*;
use strobe_core::W5Stage;
use strobe_game::sim::{SurfaceKind, stage_variant};

#[derive(Resource)]
struct SmokeBudget {
    remaining: u32,
}

fn tick_budget(mut budget: ResMut<SmokeBudget>, mut exit: MessageWriter<AppExit>) {
    budget.remaining = budget.remaining.saturating_sub(1);
    if budget.remaining == 0 {
        exit.write(AppExit::Success);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ticks: u32 = args
        .windows(2)
        .find(|pair| pair[0] == "--smoke")
        .and_then(|pair| pair[1].parse().ok())
        .unwrap_or(120);
    let variant = stage_variant(SurfaceKind::Hub, W5Stage::Unaware);
    println!("strobe-smoke: hub variant key: {variant}");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "HYPNOSYNC — Strobe".to_string(),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.insert_resource(SmokeBudget { remaining: ticks });
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
        // The hub portrait quad (its texture resolves through the
        // stage variant set in the full build; the smoke proves
        // the frame, the camera, and the render loop).
        commands.spawn((
            Sprite::from_color(Color::srgb(0.91, 0.89, 0.93), Vec2::new(360.0, 480.0)),
            Transform::from_xyz(-220.0, 0.0, 0.0),
        ));
        commands.spawn((
            Text2d::new("HYPNOSYNC"),
            TextFont {
                font_size: FontSize::Px(42.0),
                ..default()
            },
            Transform::from_xyz(180.0, 180.0, 0.0),
        ));
    });
    app.add_systems(Update, tick_budget);
    let exit = app.run();
    println!("strobe-smoke: exit: {exit:?}");
    if !matches!(exit, AppExit::Success) {
        std::process::exit(1);
    }
}
