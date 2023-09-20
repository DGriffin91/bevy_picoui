use arc_mesh::generate_arc_mesh;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::system::Command,
    input::InputSystem,
    math::vec2,
    prelude::*,
    sprite::{Material2d, Mesh2dHandle},
};
use pico::{MaterialHandleEntity, Pico};
use renderer::render;
use std::{f32::consts::FRAC_PI_2, marker::PhantomData};

pub mod arc_mesh;
pub mod guard;
pub mod hash;
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
            .add_systems(
                PreUpdate,
                (render.after(InputSystem), apply_deferred).chain(),
            )
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

pub struct PicoMaterialPlugin<M: Material2d>(PhantomData<M>);

impl<M: Material2d> Default for PicoMaterialPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: Material2d> Plugin for PicoMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pico>()
            .add_systems(PreUpdate, insert_custom_material::<M>.after(render));
    }
}

#[derive(Component)]
pub struct SwapMaterialEntity(Entity);

#[derive(Component)]
pub struct TestComponent;

pub fn insert_custom_material<M: Material2d>(
    mut commands: Commands,
    query: Query<(Entity, &SwapMaterialEntity)>,
    material_entities: Query<&MaterialHandleEntity<M>>,
) {
    for (entity, swap) in &query {
        if let Ok(h) = material_entities.get(swap.0) {
            commands.add(TrySwapInsert {
                entity,
                bundle: h.0.clone(),
            });
        }
    }
}

/// A [`Command`] that attempts to add the components in a [`Bundle`] to an entity.
pub struct TrySwapInsert<T> {
    /// The entity to which the components will be added.
    pub entity: Entity,
    /// The [`Bundle`] containing the components that will be added to the entity.
    pub bundle: T,
}

impl<T> Command for TrySwapInsert<T>
where
    T: Bundle + 'static,
{
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            entity.insert(self.bundle).remove::<SwapMaterialEntity>();
        }
    }
}
