use bevy::{
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
use bevy_picoui::{
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
    widgets::{basic_drag_widget, button, hr},
    PicoPlugin,
};

fn get_default_cam_trans() -> Transform {
    Transform::from_xyz(3.0, 2.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y)
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
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
    mut gizmo_config: ResMut<GizmoConfigStore>,
) {
    gizmo_config
        .config_mut::<DefaultGizmoConfigGroup>()
        .0
        .render_layers = RenderLayers::layer(1);

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::default()),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0 * 1000.0,
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
            InheritedVisibility::default(),
        ))
        .with_children(|builder| {
            // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
            builder.spawn((
                PbrBundle {
                    mesh: meshes.add(Rectangle::new(
                        /*
                        Taken from below since we don't View yet.
                        dbg!(
                            line_box[0].distance(line_box[1]),
                            line_box[1].distance(line_box[2])
                        );
                        */
                        1.4727595, 0.82842755,
                    )),
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
    pico.add(PicoItem {
        text: String::from(
            "Click and drag to orbit camera\nDolly with scroll wheel\nMove with WASD",
        ),
        anchor_parent: Anchor::BottomRight,
        anchor: Anchor::BottomRight,
        style: ItemStyle {
            anchor_text: Anchor::BottomRight,
            justify: JustifyText::Right,
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.3),
            ..default()
        },
        x: Val::Vh(1.0),
        y: Val::Vh(1.0),
        width: Val::Vh(30.0),
        height: Val::Vh(10.0),
        ..default()
    });

    let Ok((view, mut trans)) = camera.get_single_mut() else {
        return;
    };

    let side_bar = pico.add(PicoItem {
        x: Val::Vh(0.0),
        y: Val::Vh(0.0),
        width: Val::Vh(20.0),
        height: Val::Vh(100.0),
        anchor: Anchor::TopLeft,
        style: ItemStyle {
            justify: JustifyText::Left,
            background_color: Color::rgba(0.2, 0.2, 0.2, 0.2),
            ..default()
        },
        ..default()
    });

    let mut tdrag = |pico: &mut Pico, bg: Color, label: &str, value: f32, relative: bool| -> f32 {
        let parent = pico.add(PicoItem {
            width: Val::Percent(100.0),
            height: Val::Vh(4.0),
            anchor: Anchor::TopLeft,
            parent: Some(side_bar),
            ..default()
        });
        basic_drag_widget(
            pico,
            parent,
            label,
            value,
            5.0,
            bg,
            &mut char_input_events,
            relative,
        )
    };

    {
        let _guard = pico.vstack(Val::Vh(1.0), Val::Vh(0.5), false, &side_bar);
        pico.add(PicoItem {
            x: Val::Percent(50.0),
            width: Val::Percent(100.0),
            height: Val::Vh(2.0),
            text: "Camera".into(),
            anchor: Anchor::TopCenter,
            parent: Some(side_bar),
            ..default()
        });

        hr(&mut pico, Val::Percent(95.0), Val::Vh(0.2), Some(side_bar));

        let value = tdrag(&mut pico, RED, "Local X", 0.0, true);
        let v = trans.right();
        trans.translation += v * value;

        let value = tdrag(&mut pico, GREEN, "Local Y", 0.0, true);
        let v = trans.forward();
        trans.translation += v * value;

        let value = tdrag(&mut pico, BLUE, "Local Z", 0.0, true);
        let v = trans.up();
        trans.translation += v * value;

        hr(&mut pico, Val::Percent(95.0), Val::Vh(0.2), Some(side_bar));

        let value = tdrag(&mut pico, RED, "World X", trans.translation.x, false);
        trans.translation.x = value;

        let value = tdrag(&mut pico, GREEN, "World Y", trans.translation.y, false);
        trans.translation.y = value;

        let value = tdrag(&mut pico, BLUE, "World Z", trans.translation.z, false);
        trans.translation.z = value;

        hr(&mut pico, Val::Percent(95.0), Val::Vh(0.2), Some(side_bar));

        let btn = button(
            &mut pico,
            PicoItem {
                x: Val::Percent(50.0),
                width: Val::Percent(90.0),
                height: Val::Vh(4.0),
                style: ItemStyle {
                    corner_radius: Val::Percent(10.0),
                    background_color: DARK_GRAY,
                    ..default()
                },
                anchor: Anchor::TopCenter,
                text: "RESET CAMERA".to_string(),
                parent: Some(side_bar),
                ..default()
            },
        );

        if pico.clicked(&btn) {
            *trans = get_default_cam_trans();
        }
    }

    // Setup style for axis text
    let axis_text = |p: Vec3, s: &str| -> PicoItem {
        PicoItem {
            position_3d: Some(p),
            width: Val::Vh(3.0),
            height: Val::Vh(2.0),
            style: ItemStyle {
                corner_radius: Val::Percent(20.0),
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.3),
                ..default()
            },
            text: s.to_string(),
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
    red: 0.25,
    green: 0.18,
    blue: 0.18,
    alpha: 1.0,
};
pub const GREEN: Color = Color::Rgba {
    red: 0.18,
    green: 0.25,
    blue: 0.18,
    alpha: 1.0,
};
pub const BLUE: Color = Color::Rgba {
    red: 0.18,
    green: 0.18,
    blue: 0.25,
    alpha: 1.0,
};
pub const DARK_GRAY: Color = Color::Rgba {
    red: 0.2,
    green: 0.2,
    blue: 0.2,
    alpha: 0.5,
};
