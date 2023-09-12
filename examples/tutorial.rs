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
    impico::{render_imtext, ImItem, ImPicoPlugin, ImTextCamera, Pico},
    CoordinateTransformationsPlugin, View,
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
        .add_plugins((
            CameraControllerPlugin,
            CoordinateTransformationsPlugin,
            ImPicoPlugin {
                create_default_cam_with_order: Some(1),
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update.before(render_imtext))
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
            mouse_key_enable_mouse: MouseButton::Right,
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

    let cam_trans = Transform::from_xyz(3.0, 2.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Example Camera
    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: -1,
                    target: RenderTarget::Image(image_h.clone()),
                    ..default()
                },
                transform: cam_trans,
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
) {
    // Example instructions
    commands.spawn(
        ImItem {
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

    // Setup style for axis text
    let axis_text = |p: Vec3, s: &str| -> ImItem {
        ImItem {
            position: p,
            position_3d: true,
            rect: vec2(0.02, 0.02),
            background: Color::rgba(0.0, 0.0, 0.0, 0.3),
            text: s.to_string(),
            font_size: 0.02,
            ..default()
        }
    };

    let mut drag_value = |p: Vec3, s: &str, v: f32, c: Color| -> f32 {
        let rect = vec2(0.09, 0.04);
        let pico = pico.add(ImItem {
            text: s.to_string(),
            position: p,
            rect,
            anchor: Anchor::CenterLeft,
            rect_anchor: Anchor::CenterLeft,
            ..default()
        });
        let mut b = ImItem::button2d(
            p + Vec3::X * rect.x,
            vec2(0.05, rect.y),
            &format!("{:.2}", v),
        );
        b.color = c;
        b.background = Color::rgba(1.0, 1.0, 1.0, 0.01);
        b.rect_anchor = Anchor::CenterLeft;
        let pico = pico.add(b);
        if let Some(dragged) = pico.dragged() {
            let scale = 0.01;
            pico.items.last_mut().unwrap().text = format!("{:.2}", dragged.total_delta().x * scale);
            return dragged.delta().x * scale + v;
        }
        v
    };

    let delta = drag_value(vec3(0.01, 0.15, 0.0), "Local Camera X", 0.0, Color::RED);
    let v = trans.right();
    trans.translation += v * delta;

    let delta = drag_value(vec3(0.01, 0.20, 0.0), "Local Camera Y", 0.0, Color::GREEN);
    let v = trans.forward();
    trans.translation += v * delta;

    let delta = drag_value(vec3(0.01, 0.25, 0.0), "Local Camera Z", 0.0, Color::BLUE);
    let v = trans.up();
    trans.translation += v * delta;

    let delta = drag_value(
        vec3(0.01, 0.30, 0.0),
        "World Camera X",
        trans.translation.x,
        Color::RED,
    );
    trans.translation.x = delta;

    let delta = drag_value(
        vec3(0.01, 0.35, 0.0),
        "World Camera Y",
        trans.translation.y,
        Color::GREEN,
    );
    trans.translation.y = delta;

    let delta = drag_value(
        vec3(0.01, 0.40, 0.0),
        "World Camera Z",
        trans.translation.z,
        Color::BLUE,
    );
    trans.translation.z = delta;

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
