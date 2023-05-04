use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use bevy_basic_camera::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(CameraControllerPlugin)
        .add_plugin(MaterialPlugin::<TestMaterial>::default())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut testmaterials: ResMut<Assets<TestMaterial>>,
) {
    let material: Handle<TestMaterial> = testmaterials.add(TestMaterial {});

    // plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: material.clone(),
        ..default()
    });

    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material,
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

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(8.0, 5.0, 8.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        })
        .insert(CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        });
}

impl Material for TestMaterial {
    fn fragment_shader() -> ShaderRef {
        "test_bed.wgsl".into()
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "e17f6ffe-1842-4822-8926-e0ed174294c8"]
pub struct TestMaterial {}
