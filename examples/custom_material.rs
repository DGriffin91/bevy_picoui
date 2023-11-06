use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Anchor, Material2d, Material2dPlugin},
};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem, PicoMaterials},
    PicoMaterialPlugin, PicoPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins,
            PicoPlugin::default(),
            Material2dPlugin::<CustomMaterial>::default(),
            PicoMaterialPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

fn update(
    mut commands: Commands,
    mut pico: ResMut<Pico>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    pico_materials: PicoMaterials<CustomMaterial>,
    mut custom_material: Local<Option<Handle<CustomMaterial>>>,
) {
    if custom_material.is_none() {
        *custom_material = Some(materials.add(CustomMaterial {}));
    }
    let custom_material = custom_material.as_mut().unwrap();

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
            corner_radius: Val::Percent(10.0),
            ..default()
        }
        .set_custom_material(&mut commands, custom_material, pico_materials),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        parent: Some(bg),
        ..default()
    });
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}
