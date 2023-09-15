use std::time::Duration;

use bevy::{asset::ChangeWatcher, math::*, prelude::*, render::view::RenderLayers, sprite::Anchor};

use bevy_picoui::{toggle_button, Pico, Pico2dCamera, PicoItem, PicoPlugin};

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
        Pico2dCamera,
        RenderLayers::all(),
    ));
}

fn update(mut pico: ResMut<Pico>, mut toggle_states: Local<[[bool; 10]; 10]>) {
    let vh = pico.vh;
    let main_box = pico
        .add(PicoItem {
            position: vec3(0.1 * vh, 0.5, 0.0),
            rect_anchor: Anchor::CenterLeft,
            rect: vec2(0.5 * vh, 0.5),
            background: SLATE,
            ..default()
        })
        .last();

    {
        let _guard = pico.vstack(0.01, 0.015);

        for row in &mut toggle_states {
            let lane = pico
                .add(PicoItem {
                    position: vec3(0.0, 0.0, 0.0),
                    rect: vec2(1.0, 0.08),
                    background: CURRENT,
                    rect_anchor: Anchor::TopLeft,
                    parent: [main_box; 4],
                    ..default()
                })
                .last();
            {
                let _guard = pico.hstack(0.0, 0.045);
                for toggle_state in row {
                    toggle_button(
                        &mut pico,
                        PicoItem {
                            position: vec3(0.0, 0.1, 0.0),
                            rect: vec2(0.05, 0.8),
                            background: OILVINE,
                            rect_anchor: Anchor::TopLeft,
                            parent: [lane; 4],
                            ..default()
                        },
                        OILVINE + Color::DARK_GRAY,
                        toggle_state,
                    );
                }
            }
        }
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
