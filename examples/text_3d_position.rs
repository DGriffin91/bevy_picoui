use std::time::Duration;

use bevy::{asset::ChangeWatcher, math::*, prelude::*, render::view::RenderLayers, sprite::Anchor};

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_coordinate_systems::CoordinateTransformationsPlugin;
use bevy_picoui::{Pico, Pico2dCamera, PicoItem, PicoPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        // Put gizmos on layer 1 so they don't show up on the 2d camera
        .insert_resource(GizmoConfig {
            render_layers: RenderLayers::layer(1),
            ..default()
        })
        .add_plugins((
            CameraControllerPlugin,
            CoordinateTransformationsPlugin,
            PicoPlugin {
                create_default_2d_cam_with_order: Some(1),
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 5.5, 10.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        },
        CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        },
        Pico2dCamera,
        RenderLayers::all(),
    ));
}

#[derive(Component)]
struct ExampleCamera;

fn update(mut gizmos: Gizmos, mut pico: ResMut<Pico>) {
    pico.add(PicoItem {
        position: vec3(0.02, 0.02, 0.01),
        text: String::from("Click and drag to orbit camera\nDolly with scroll wheel\nMove with WASD\n\nHover over the Y axis text"),
        anchor: Anchor::TopLeft,
        ..default()
    });

    // Draw Y axis
    gizmos.ray(Vec3::ZERO, Vec3::Y * 1000.0, Color::GREEN);

    // Add 3d text
    pico.add(PicoItem {
        position: Vec3::Y * 1.1,
        position_3d: true,
        rect: vec2(0.02, 0.02),
        background: Color::rgba(0.1, 0.1, 0.1, 0.5),
        text: String::from("Y+"),
        rect_anchor: Anchor::TopLeft,
        ..default()
    });
    let axis_text_index = pico.last();
    if pico.hovered(axis_text_index) {
        // Make axis text more opaque
        pico.get_mut(axis_text_index).background = Color::rgba(0.1, 0.1, 0.1, 0.8);

        // Get 2d bounding box of axis text
        let state = pico.get_state(axis_text_index).unwrap();
        let position = vec3(state.bbox.x, state.bbox.w + 0.01, 0.1);

        // Add 2d text
        pico.add(PicoItem {
            position,
            rect: vec2(0.1, 0.02),
            background: Color::rgba(0.1, 0.1, 0.1, 0.8),
            text: String::from("HELLO WORLD"),
            rect_anchor: Anchor::TopLeft,
            ..default()
        });
    }
}
