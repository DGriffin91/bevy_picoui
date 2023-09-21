use bevy::{prelude::*, sprite::Anchor};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
    PicoPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PicoPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

fn update(
    mut pico: ResMut<Pico>,
    asset_server: ResMut<AssetServer>,
    mut image: Local<Option<Handle<Image>>>,
) {
    if image.is_none() {
        *image = Some(
            asset_server
                .load("images/generic-rpg-ui-inventario.png")
                .into(),
        );
    }
    let image = image.as_mut().unwrap();

    let bg = pico.add(PicoItem {
        x: Val::Px(0.0),
        y: Val::Px(0.0),
        style: ItemStyle {
            corner_radius: Val::Percent(10.0),
            background_color: RGB_PALETTE[0][0] * 0.2,
            border_width: Val::Px(1.0),
            border_color: RGB_PALETTE[0][3],
            ..default()
        },
        width: Val::VMin(70.0),
        height: Val::VMin(70.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        ..default()
    });

    pico.add(PicoItem {
        width: Val::Percent(80.0),
        height: Val::Percent(80.0),
        style: ItemStyle {
            /// For image to be fully opaque with the correct colors, the background needs to be white.
            background_color: Color::WHITE,
            image: Some(image.clone()),
            ..default()
        },
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        parent: Some(bg),
        ..default()
    });
}