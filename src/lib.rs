use arc_mesh::generate_arc_mesh;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, input::InputSystem, math::vec2, prelude::*,
    sprite::Mesh2dHandle,
};
use pico::Pico;
use renderer::render;
use std::f32::consts::FRAC_PI_2;

pub mod arc_mesh;
pub mod guard;
pub mod palette;
pub mod pico;
pub mod renderer;
pub mod widgets;

#[derive(Default)]
pub struct PicoPlugin {
    // Set if using in a scene with no 2d camera
    pub create_default_2d_cam_with_order: Option<isize>,
}

#[derive(Resource)]
pub struct CreateDefaultCamWithOrder(isize);

impl Plugin for PicoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pico>()
            .add_systems(PreUpdate, (render.after(InputSystem), apply_deferred))
            .add_systems(Startup, setup);
        if let Some(n) = self.create_default_2d_cam_with_order {
            app.insert_resource(CreateDefaultCamWithOrder(n))
                .add_systems(Startup, setup_2d_camera);
        }
    }
}

#[derive(Resource)]
pub struct MeshHandles {
    circle: Handle<Mesh>,
    rect: Handle<Mesh>,
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let arc_mesh = generate_arc_mesh(12, 1.0, 0.0, FRAC_PI_2);
    let circle: Mesh2dHandle = meshes.add(arc_mesh).into();
    let rect: Mesh2dHandle = meshes.add(shape::Quad::new(vec2(1.0, 1.0)).into()).into();

    commands.insert_resource(MeshHandles {
        circle: circle.0,
        rect: rect.0,
    });
}

fn setup_2d_camera(mut commands: Commands, order: Res<CreateDefaultCamWithOrder>) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: order.0,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });
}
