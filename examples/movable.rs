use std::time::Duration;

use bevy::{asset::ChangeWatcher, math::*, prelude::*, sprite::Anchor};

use bevy_picoui::{
    palette::RGB_PALETTE,
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
        .add_plugins(PicoPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
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
        background: Color::WHITE * 0.2,
        ..default()
    });

    // Radius of the circle in Vh
    let radius = Val::Vh(3.0);

    // Need to use a consistent id, usually the id is generated from spatial components of the item
    let id = 098743542350897;
    if let Some(state) = pico.state.get(&id) {
        if let Some(drag) = state.drag {
            let delta = drag.delta();
            *position += delta;
        };
    }

    // Clamp position to parent box
    let bbox = pico.bbox(main_box);
    let mut size = (bbox.zw() - bbox.xy()) / 2.0;
    size -= vec2(pico.val_x(radius), pico.val_y(radius)); // include circle radius
    *position = position.clamp(-size, size);

    pico.add(PicoItem {
        depth: Some(0.9),
        x: Val::Vw(position.x * 100.0),
        y: Val::Vh(position.y * 100.0),
        width: radius * 2.0,
        height: radius * 2.0,
        corner_radius: Val::Vh(3.0),
        border_width: Val::Vh(0.1),
        border_color: Color::WHITE,
        background: RGB_PALETTE[2][3],
        parent: Some(main_box),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        spatial_id: Some(id), // Manually set id
        ..default()
    });
}
