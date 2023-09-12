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
use bevy_coordinate_systems::{
    impico::{button, drag_value, DragValue, ImTextCamera, Pico, PicoItem, PicoPlugin},
    CoordinateTransformationsPlugin, View,
};

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
                create_default_cam_with_order: Some(1),
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
        ImTextCamera,
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
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut camera: Query<(&View, &mut Transform), With<ExampleCamera>>,
    mut pico: ResMut<Pico>,
    mut camera_controller: Query<&mut CameraController>,
) {
    if let Some(mut camera_controller) = camera_controller.iter_mut().next() {
        // Disable camera controller if pico is interacting
        camera_controller.enabled = !pico.interacting;
    }

    // Example instructions
    commands.spawn(
        PicoItem {
            position: vec3(0.0, 0.0, 0.0),
            text: String::from(
                "Click and drag to orbit camera\nDolly with scroll wheel\nMove with WASD",
            ),
            rect_anchor: Anchor::TopLeft,
            rect: vec2(0.2, 0.1),
            alignment: TextAlignment::Left,
            background: Color::rgba(0.0, 0.0, 0.0, 0.3),
            ..default()
        }
        .keep(),
    );

    let Ok((view, mut trans)) = camera.get_single_mut() else {
        return;
    };

    pico.add(PicoItem {
        position: vec3(0.01, 0.15, 0.0),
        rect: vec2(0.1, 0.04),
        text: "- Camera -".into(),
        rect_anchor: Anchor::CenterLeft,
        ..default()
    })
    .last();

    let tdrag = |pico: &mut Pico, position: Vec3, label: &str, value: f32| -> DragValue {
        let dv = drag_value(pico, position, 0.04, 0.06, 0.04, label, 0.01, value);
        dv
    };

    let dv = tdrag(&mut pico, vec3(0.01, 0.2, 0.0), "Local X", 0.0);
    pico.get_mut(dv.drag_index).background += Color::rgba(0.0, -0.5, -0.5, 0.05); // Need * for color
    let v = trans.right();
    trans.translation += v * dv.value;

    let dv = tdrag(&mut pico, vec3(0.01, 0.25, 0.0), "Local Y", 0.0);
    pico.get_mut(dv.drag_index).background += Color::rgba(-0.5, 0.0, -0.5, 0.05); // Need * for color
    let v = trans.forward();
    trans.translation += v * dv.value;

    let dv = tdrag(&mut pico, vec3(0.01, 0.3, 0.0), "Local Z", 0.0);
    pico.get_mut(dv.drag_index).background += Color::rgba(-0.5, -0.5, 0.0, 0.05); // Need * for color
    let v = trans.up();
    trans.translation += v * dv.value;

    let dv = tdrag(
        &mut pico,
        vec3(0.01, 0.35, 0.0),
        "World X",
        trans.translation.x,
    );
    pico.get_mut(dv.drag_index).background += Color::rgba(0.0, -0.5, -0.5, 0.05); // Need * for color
    trans.translation.x = dv.value;

    let dv = tdrag(
        &mut pico,
        vec3(0.01, 0.4, 0.0),
        "World Y",
        trans.translation.y,
    );
    pico.get_mut(dv.drag_index).background += Color::rgba(-0.5, 0.0, -0.5, 0.05); // Need * for color
    trans.translation.y = dv.value;

    let dv = tdrag(
        &mut pico,
        vec3(0.01, 0.45, 0.0),
        "World Z",
        trans.translation.z,
    );
    pico.get_mut(dv.drag_index).background += Color::rgba(-0.5, -0.5, 0.0, 0.05); // Need * for color
    trans.translation.z = dv.value;

    let btn = button(
        &mut pico,
        vec3(0.01, 0.5, 0.0),
        vec2(0.1, 0.04),
        "RESET CAMERA",
    );
    pico.get_mut(btn).rect_anchor = Anchor::CenterLeft;
    if pico.clicked(btn) {
        *trans = get_default_cam_trans();
    }

    // Setup style for axis text
    let axis_text = |p: Vec3, s: &str| -> PicoItem {
        PicoItem {
            position: p,
            position_3d: true,
            rect: vec2(0.02, 0.02),
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
    commands.spawn(axis_text(Vec3::X, "+X"));
    commands.spawn(axis_text(Vec3::Y, "+Y"));
    commands.spawn(axis_text(Vec3::Z, "+Z"));

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
        commands.spawn(axis_text(view_zero + view_x_dir, "+X"));
        commands.spawn(axis_text(view_zero + view_y_dir, "+Y"));
        commands.spawn(axis_text(view_zero + view_z_dir, "+Z"));
    }
}
