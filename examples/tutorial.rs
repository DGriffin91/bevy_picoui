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
};

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_coordinate_systems::{
    im3dtext::{Im3dText, Im3dTextCamera, ImTextPlugin},
    CoordinateTransformationsPlugin, View,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
            ..default()
        }))
        // RenderLayers are not supported for UI yet
        //.insert_resource(ImTextRenderLayers(RenderLayers::layer(1)))
        .add_plugins((
            CameraControllerPlugin,
            CoordinateTransformationsPlugin,
            ImTextPlugin,
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
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 5.5, 10.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        },
        CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        },
        Im3dTextCamera,
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

    // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
    commands.spawn((
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
                base_color_texture: Some(image_h.clone()),
                unlit: true,
                ..default()
            }),
            transform: cam_trans
                .clone()
                .with_translation(cam_trans.translation + cam_trans.forward()),
            ..default()
        },
        RenderLayers::layer(1),
    ));

    // Example Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(image_h),
                ..default()
            },
            transform: cam_trans,
            ..default()
        },
        ExampleCamera,
    ));

    // example instructions
    commands.spawn(
        TextBundle::from_section(
            "Click and drag to orbit camera\nDolly with scroll wheel\nMove with WASD",
            TextStyle {
                font_size: 12.,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

#[derive(Component)]
struct ExampleCamera;

fn update(mut commands: Commands, mut gizmos: Gizmos, camera: Query<&View, With<ExampleCamera>>) {
    let Ok(view) = camera.get_single() else {
        return;
    };

    // Draw axes
    gizmos.ray(Vec3::ZERO, Vec3::X * 1000.0, Color::RED);
    gizmos.ray(Vec3::ZERO, Vec3::Y * 1000.0, Color::GREEN);
    gizmos.ray(Vec3::ZERO, Vec3::Z * 1000.0, Color::BLUE);
    commands.spawn(Im3dText::new(Vec3::X, "+X"));
    commands.spawn(Im3dText::new(Vec3::Y, "+Y"));
    commands.spawn(Im3dText::new(Vec3::Z, "+Z"));

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
        commands.spawn(Im3dText::new(view_zero + view_x_dir, "+X"));
        commands.spawn(Im3dText::new(view_zero + view_y_dir, "+Y"));
        commands.spawn(Im3dText::new(view_zero + view_z_dir, "+Z"));
    }
}
