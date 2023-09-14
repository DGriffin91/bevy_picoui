use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    math::*,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::Anchor,
};

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_coordinate_systems::{CoordinateTransformationsPlugin, View};
use bevy_picoui::{button, drag_value, DragValue, Pico, Pico2dCamera, PicoItem, PicoPlugin};

fn get_default_cam_trans() -> Transform {
    Transform::from_xyz(3.0, 2.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y)
}

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
    mut images: ResMut<Assets<Image>>,
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

    // User Camera
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

    let size = Extent3d {
        width: 320,
        height: 180,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);
    let image_h = images.add(image);

    // Example Camera
    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: -1,
                    target: RenderTarget::Image(image_h.clone()),
                    ..default()
                },
                transform: get_default_cam_trans(),
                ..default()
            },
            ExampleCamera,
            RenderLayers::layer(0).with(1),
            Visibility::Visible,
            ComputedVisibility::default(),
        ))
        .with_children(|builder| {
            // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
            builder.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                        /*
                        Taken from below since we don't View yet.
                        dbg!(
                            line_box[0].distance(line_box[1]),
                            line_box[1].distance(line_box[2])
                        );
                        */
                        1.4727595, 0.82842755,
                    )))),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(image_h),
                        unlit: true,
                        ..default()
                    }),
                    transform: Transform::from_translation(-Vec3::Z),
                    ..default()
                },
                RenderLayers::layer(2),
                Camera2d::default(),
            ));
        });
}

#[derive(Component)]
struct ExampleCamera;

fn update(
    mut gizmos: Gizmos,
    mut camera: Query<(&View, &mut Transform), With<ExampleCamera>>,
    mut pico: ResMut<Pico>,
    mut camera_controller: Query<&mut CameraController>,
    mut char_input_events: EventReader<ReceivedCharacter>,
) {
    if let Some(mut camera_controller) = camera_controller.iter_mut().next() {
        // Disable camera controller if pico is interacting
        camera_controller.enabled = !pico.interacting;
    }

    // Example instructions
    let right = pico.vh_right;
    pico.add(PicoItem {
        position: vec3(right, 1.0, 0.0),
        text: String::from(
            "Click and drag to orbit camera\nDolly with scroll wheel\nMove with WASD",
        ),
        rect_anchor: Anchor::BottomRight,
        rect: vec2(0.35, 0.1),
        alignment: TextAlignment::Right,
        background: Color::rgba(0.0, 0.0, 0.0, 0.3),
        ..default()
    });

    let Ok((view, mut trans)) = camera.get_single_mut() else {
        return;
    };
    let vh = pico.vh;

    let side_bar = pico
        .add(PicoItem {
            position: vec3(0.0, 0.0, 0.0),
            rect_anchor: Anchor::TopLeft,
            rect: vec2(0.2 * vh, 1.0),
            alignment: TextAlignment::Left,
            background: Color::rgba(0.2, 0.2, 0.2, 0.2),
            ..default()
        })
        .last();

    let mut tdrag =
        |pico: &mut Pico, bg: Color, label: &str, value: f32, relative: bool| -> DragValue {
            let scale = 0.01;
            let dv = drag_value(
                pico,
                bg,
                vec3(0.05, 0.0, 0.0),
                0.04,
                0.6,
                0.3,
                label,
                scale,
                value,
                Some(side_bar),
                Some(&mut char_input_events),
            );
            if relative {
                // Show relative value while dragging drag
                if let Some(state) = pico.get_state_mut(dv.drag_index) {
                    if let Some(drag) = state.drag {
                        pico.get_mut(dv.drag_index).text =
                            format!("{:.2}", drag.total_delta().x * scale)
                    }
                }
            }
            dv
        };

    {
        let _guard = pico.vstack(0.02, 0.01);

        pico.add(PicoItem {
            position: vec3(0.02, 0.0, 0.0),
            rect: vec2(1.0, 0.02),
            text: "- Camera -".into(),
            rect_anchor: Anchor::TopLeft,
            parent: Some(side_bar),
            ..default()
        })
        .last();

        let dv = tdrag(&mut pico, RED, "Local X", 0.0, true);
        let v = trans.right();
        trans.translation += v * dv.value;

        let dv = tdrag(&mut pico, GREEN, "Local Y", 0.0, true);
        let v = trans.forward();
        trans.translation += v * dv.value;

        let dv = tdrag(&mut pico, BLUE, "Local Z", 0.0, true);
        let v = trans.up();
        trans.translation += v * dv.value;

        let dv = tdrag(&mut pico, RED, "World X", trans.translation.x, false);
        trans.translation.x = dv.value;

        let dv = tdrag(&mut pico, GREEN, "World Y", trans.translation.y, false);
        trans.translation.y = dv.value;

        let dv = tdrag(&mut pico, BLUE, "World Z", trans.translation.z, false);
        trans.translation.z = dv.value;

        let btn = button(
            &mut pico,
            PicoItem {
                position: vec3(0.05, 0.0, 0.0),
                rect: vec2(0.9, 0.04),
                background: DARK_GRAY,
                text: "RESET CAMERA".to_string(),
                parent: Some(side_bar),
                ..default()
            },
        );

        pico.get_mut(btn).rect_anchor = Anchor::TopLeft;
        if pico.clicked(btn) {
            *trans = get_default_cam_trans();
        }
    }

    // Setup style for axis text
    let axis_text = |p: Vec3, s: &str| -> PicoItem {
        PicoItem {
            position: p,
            position_3d: true,
            rect: vec2(0.02 * vh, 0.02),
            background: Color::rgba(0.0, 0.0, 0.0, 0.3),
            text: s.to_string(),
            font_size: 0.02,
            ..default()
        }
    };

    // Draw axes
    gizmos.ray(Vec3::ZERO, Vec3::X * 1000.0, Color::RED);
    gizmos.ray(Vec3::ZERO, Vec3::Y * 1000.0, Color::GREEN);
    gizmos.ray(Vec3::ZERO, Vec3::Z * 1000.0, Color::BLUE);
    pico.add(axis_text(Vec3::X, "+X"));
    pico.add(axis_text(Vec3::Y, "+Y"));
    pico.add(axis_text(Vec3::Z, "+Z"));

    // Draw Camera
    {
        let view_zero = view.position_view_to_world(Vec3::ZERO);
        let view_x_dir = view.direction_view_to_world(Vec3::X);
        let view_y_dir = view.direction_view_to_world(Vec3::Y);
        let view_z_dir = view.direction_view_to_world(Vec3::Z);

        // get ndc coords for z of 0.5 in front of the camera
        let ndc_depth = view.view_z_to_depth_ndc(-1.0);
        // border around camera's perspective fov
        let line_box = [
            view.position_ndc_to_world(vec3(-1.0, -1.0, ndc_depth)),
            view.position_ndc_to_world(vec3(1.0, -1.0, ndc_depth)),
            view.position_ndc_to_world(vec3(1.0, 1.0, ndc_depth)),
            view.position_ndc_to_world(vec3(-1.0, 1.0, ndc_depth)),
            view.position_ndc_to_world(vec3(-1.0, -1.0, ndc_depth)),
        ];

        gizmos.linestrip(line_box.map(|p| p), Color::WHITE);

        // lines to camera origin
        line_box[..4]
            .iter()
            .for_each(|p| gizmos.line(*p, view_zero, Color::WHITE));

        // Draw camera axes
        gizmos.ray(view_zero, view_x_dir, Color::RED);
        gizmos.ray(view_zero, view_y_dir, Color::GREEN);
        gizmos.ray(view_zero, view_z_dir, Color::BLUE);
        pico.add(axis_text(view_zero + view_x_dir, "+X"));
        pico.add(axis_text(view_zero + view_y_dir, "+Y"));
        pico.add(axis_text(view_zero + view_z_dir, "+Z"));
    }
}

// ------
// Colors
// ------

pub const RED: Color = Color::Rgba {
    red: 0.3,
    green: 0.15,
    blue: 0.15,
    alpha: 1.0,
};
pub const GREEN: Color = Color::Rgba {
    red: 0.15,
    green: 0.3,
    blue: 0.15,
    alpha: 1.0,
};
pub const BLUE: Color = Color::Rgba {
    red: 0.15,
    green: 0.15,
    blue: 0.3,
    alpha: 1.0,
};
pub const DARK_GRAY: Color = Color::Rgba {
    red: 0.2,
    green: 0.2,
    blue: 0.2,
    alpha: 0.5,
};
