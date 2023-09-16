use std::time::Duration;

use bevy::{asset::ChangeWatcher, math::*, prelude::*, render::view::RenderLayers, sprite::Anchor};

use bevy_picoui::{Pico, Pico2dCamera, PicoItem, PicoPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
            ..default()
        }))
        .insert_resource(GizmoConfig {
            render_layers: RenderLayers::layer(1),
            ..default()
        })
        .add_plugins((PicoPlugin {
            create_default_2d_cam_with_order: Some(1),
        },))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 5.5, 10.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        },
        Pico2dCamera,
        RenderLayers::all(),
    ));
}

fn update(mut pico: ResMut<Pico>) {
    let main_box = pico
        .add(PicoItem {
            depth: Some(0.01),
            x: Val::Percent(0.0),
            y: Val::Percent(0.0),
            width: Val::VMin(50.0),
            height: Val::VMin(50.0),
            corner_radius: Val::Percent(4.0),
            anchor: Anchor::Center,
            anchor_parent: Anchor::Center,
            background: SLATE,
            ..default()
        })
        .last();

    for parent_anchor in [
        Anchor::Center,
        Anchor::BottomLeft,
        Anchor::BottomCenter,
        Anchor::BottomRight,
        Anchor::CenterLeft,
        Anchor::CenterRight,
        Anchor::TopLeft,
        Anchor::TopCenter,
        Anchor::TopRight,
    ] {
        pico.add(PicoItem {
            depth: Some(0.5),
            x: Val::Px(8.0),
            y: Val::Px(8.0),
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            corner_radius: Val::Px(4.0),
            anchor: parent_anchor.clone(),
            anchor_parent: parent_anchor.clone(),
            background: BURNT_RED,
            parent: Some(main_box),
            ..default()
        });
        pico.add(PicoItem {
            depth: Some(0.9),
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            corner_radius: Val::Px(4.0),
            anchor_parent: parent_anchor.clone(),
            background: CURRENT,
            parent: Some(main_box),
            ..default()
        });
    }
}

// ------
// Colors
// ------

pub const SLATE: Color = Color::Rgba {
    red: 0.156,
    green: 0.239,
    blue: 0.231,
    alpha: 1.0,
};

pub const CURRENT: Color = Color::Rgba {
    red: 0.098,
    green: 0.447,
    blue: 0.470,
    alpha: 1.0,
};

pub const BURNT_RED: Color = Color::Rgba {
    red: 0.466,
    green: 0.180,
    blue: 0.145,
    alpha: 1.0,
};

pub const OILVINE: Color = Color::Rgba {
    red: 0.549,
    green: 0.702,
    blue: 0.412,
    alpha: 1.0,
};
