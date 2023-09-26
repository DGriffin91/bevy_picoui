use bevy::{math::vec3, prelude::*, sprite::Anchor};

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
        // For actual projects consider using https://github.com/NiklasEi/bevy_asset_loader or load assets in separate startup system
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

    let pic_index = pico.add(PicoItem {
        width: Val::Percent(80.0),
        height: Val::Percent(80.0),
        style: ItemStyle {
            /// For image to be fully opaque with the correct colors, the background needs to be white.
            background_color: Color::WHITE,
            image: Some(image.clone()),
            edge_softness: Val::Percent(25.0),
            border_width: Val::Px(1.0),
            border_color: RGB_PALETTE[0][3] * 2.0,
            corner_radius: Val::Percent(50.0),
            ..default()
        },
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        parent: Some(bg),
        ..default()
    });
    if pico.hovered(&pic_index) {
        let style = &mut pico.get_mut(&pic_index).style;
        style.background_uv_transform = Transform::from_scale(vec3(0.97, 0.97, 0.97));
        style.background_color = Color::rgb(1.3, 1.3, 1.3);
    }
}
