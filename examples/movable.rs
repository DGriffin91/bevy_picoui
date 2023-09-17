use std::time::Duration;

use bevy::{asset::ChangeWatcher, math::*, prelude::*, render::view::RenderLayers, sprite::Anchor};

use bevy_picoui::{
    pico::{Pico, Pico2dCamera, PicoItem},
    PicoPlugin,
};

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

fn update(mut pico: ResMut<Pico>, mut position: Local<Option<Vec2>>) {
    if position.is_none() {
        *position = Some(vec2(0.0, 0.0));
    }
    let position = position.as_mut().unwrap();

    let main_box = pico.add(PicoItem {
        depth: Some(0.01),
        width: Val::VMin(50.0),
        height: Val::VMin(50.0),
        corner_radius: Val::Vh(3.0),
        border_width: Val::Px(1.0),
        border_color: Color::WHITE,
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        background: SLATE,
        ..default()
    });

    // Get the radius of the circle in pixels
    let radius = pico.uv_scale_to_px(vec2(pico.val_x(Val::Vh(3.0)), 0.0)).x;

    // Need to use a consistent id, usually the id is generated from spatial components of the item
    let id = 098743542350897;
    if let Some(state) = pico.state.get(&id) {
        if let Some(drag) = state.drag {
            let delta = drag.delta();
            *position += delta;
        };
    }

    let bbox = pico.bbox(main_box);
    let min = pico.uv_position_to_px(bbox.xy()) + radius;
    let max = pico.uv_position_to_px(bbox.zw()) - radius;
    *position = position.clamp(min, max);

    pico.add(PicoItem {
        depth: Some(0.9),
        x: Val::Px(position.x),
        y: Val::Px(position.y),
        width: Val::Vh(6.0),
        height: Val::Vh(6.0),
        corner_radius: Val::Vh(3.0),
        border_width: Val::Vh(0.1),
        border_color: Color::WHITE,
        background: BURNT_RED,
        parent: Some(main_box),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        spatial_id: Some(id), // Manually set id
        ..default()
    });
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
