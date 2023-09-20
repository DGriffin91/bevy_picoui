// Derived from https://github.com/bevyengine/bevy/blob/v0.11.2/examples/stress_tests/many_buttons.rs
// This version of the test with picoui is much faster on bevy main due to batching.
// I'm currently getting 17ms with picoui on bevy main vs 20ms with the many_buttons example in the bevy repo on bevy main with image-freq set to 0.

//! This example shows what happens when there is a lot of buttons on screen.
//!
//! To start the demo without text run
//! `cargo run --example many_buttons --release no-text`
//!
//! //! To start the demo without borders run
//! `cargo run --example many_buttons --release no-borders`

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::*,
    prelude::*,
    sprite::Anchor,
    window::{PresentMode, WindowPlugin},
};

use bevy_picoui::{
    pico::{ItemIndex, ItemStyle, Pico, Pico2dCamera, PicoItem},
    PicoPlugin,
};

// For a total of 110 * 110 = 12100 buttons with text
const ROW_COLUMN_COUNT: usize = 110;
const FONT_SIZE: f32 = 7.0;

/// This example shows what happens when there is a lot of buttons on screen.
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(PicoPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

fn update(mut pico: ResMut<Pico>) {
    let count = ROW_COLUMN_COUNT;
    let count_f = count as f32;
    let as_rainbow = |i: usize| Color::hsl((i as f32 / count_f) * 360.0, 0.9, 0.8);

    let main_box = pico.add(PicoItem {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        ..default()
    });

    let spawn_text = std::env::args().all(|arg| arg != "no-text");
    let border = if std::env::args().all(|arg| arg != "no-borders") {
        Val::Px(2.0)
    } else {
        Val::DEFAULT
    };
    for i in 0..count {
        for j in 0..count {
            let color = as_rainbow(j % i.max(1)).into();
            let border_color = as_rainbow(i % j.max(1)).into();
            spawn_button(
                &mut pico,
                color,
                count_f,
                i,
                j,
                spawn_text,
                border,
                border_color,
                main_box,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_button(
    pico: &mut Pico,
    background_color: Color,
    total: f32,
    i: usize,
    j: usize,
    spawn_text: bool,
    border: Val,
    border_color: Color,
    parent: ItemIndex,
) {
    let width = 90.0 / total;
    let btn_index = pico.add(PicoItem {
        x: Val::Percent(100.0 / total * j as f32),
        y: Val::Percent(100.0 - 100.0 / total * i as f32),
        width: Val::Percent(width),
        height: Val::Percent(width),
        anchor: Anchor::Center,
        style: ItemStyle {
            background: background_color,
            border_color,
            border_width: border,
            font_size: Val::Px(FONT_SIZE),
            ..default()
        },
        parent: Some(parent.clone()),
        ..default()
    });
    let hovered = pico.hovered(&btn_index);
    let btn = pico.get_mut(&btn_index);
    if hovered {
        btn.style.background = Color::ORANGE_RED;
    }
    if spawn_text {
        btn.text = format!("{i}, {j}");
    }
}
