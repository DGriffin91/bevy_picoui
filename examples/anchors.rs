use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*, sprite::Anchor};

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

fn update(mut pico: ResMut<Pico>) {
    let main_box = pico.add(PicoItem {
        depth: Some(0.01),
        x: Val::Px(0.0),
        y: Val::Px(0.0),
        width: Val::VMin(50.0),
        height: Val::VMin(50.0),
        corner_radius: Val::Percent(4.0),
        border_width: Val::Px(1.0),
        border_color: Color::WHITE,
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        background: Color::WHITE * 0.1,
        ..default()
    });

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
        // 0.0 for center anchors, multiplied by x,y so it is not offset for center axis
        let center_anchor = (parent_anchor.as_vec() * 2.0).abs();
        pico.add(PicoItem {
            depth: Some(0.5),
            x: Val::Px(8.0 * center_anchor.x),
            y: Val::Px(8.0 * center_anchor.y),
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            corner_radius: Val::Px(4.0),
            border_width: Val::Px(1.0),
            border_color: Color::WHITE,
            anchor: parent_anchor.clone(),
            anchor_parent: parent_anchor.clone(),
            background: RGB_PALETTE[0][0],
            parent: Some(main_box),
            ..default()
        });
        pico.add(PicoItem {
            depth: Some(0.9),
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            corner_radius: Val::Px(4.0),
            border_width: Val::Px(1.0),
            border_color: Color::WHITE,
            anchor_parent: parent_anchor.clone(),
            background: RGB_PALETTE[0][2],
            parent: Some(main_box),
            ..default()
        });
    }
}
