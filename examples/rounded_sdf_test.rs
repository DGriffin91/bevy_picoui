use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    math::{vec3, vec4},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, BlendState, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
    sprite::{Anchor, Material2d, Material2dKey, Material2dPlugin},
};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem, PicoMaterials},
    PicoMaterialPlugin, PicoPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Make it so the shader can be hot reloaded.
            watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
            ..default()
        }))
        .add_plugins((
            PicoPlugin::default(),
            Material2dPlugin::<CustomMaterial>::default(),
            PicoMaterialPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

fn update(
    mut commands: Commands,
    mut pico: ResMut<Pico>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    pico_materials: PicoMaterials<CustomMaterial>,
    mut custom_material: Local<Option<Handle<CustomMaterial>>>,
    asset_server: ResMut<AssetServer>,
) {
    if custom_material.is_none() {
        *custom_material = Some(
            materials.add(CustomMaterial {
                material_settings: CustomMaterialUniform {
                    corner_radius: 100.0,
                    edge_softness: 1.0,
                    border_thickness: 1.0,
                    border_softness: 1.0,
                    border_color: vec4(1.0, 0.0, 0.0, 1.0),
                    background_color1: vec4(1.0, 0.0, 0.0, 1.0),
                    background_color2: vec4(0.0, 0.0, 1.0, 1.0),
                    background_mat: Transform::from_translation(vec3(0.0, 0.0, 0.0))
                        .with_rotation(Quat::from_rotation_z(45.0f32.to_radians()))
                        .with_scale(vec3(0.7, 0.7, 0.0))
                        .compute_matrix(),
                    flags: 1,
                },
                texture: Some(
                    asset_server
                        .load("images/generic-rpg-ui-inventario.png")
                        .into(),
                ),
                blend_state: None,
            }),
        );
    }
    let custom_material = custom_material.as_mut().unwrap();

    let bg = pico.add(PicoItem {
        x: Val::Px(0.0),
        y: Val::Px(0.0),
        style: ItemStyle {
            corner_radius: Val::Percent(10.0),
            background_color: RGB_PALETTE[0][0] * 0.2,
            border_width: Val::Px(1.0),
            border_color: RGB_PALETTE[0][3],
            ..default()
        },
        //width: Val::VMin(70.0),
        //height: Val::VMin(70.0),
        width: Val::Percent(50.0),
        height: Val::Percent(30.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        ..default()
    });

    pico.add(PicoItem {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        style: ItemStyle::default().set_custom_material(
            &mut commands,
            custom_material,
            pico_materials,
        ),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        parent: Some(bg),
        ..default()
    });
}

#[derive(ShaderType, Debug, Clone)]
pub struct CustomMaterialUniform {
    pub corner_radius: f32,
    pub edge_softness: f32,
    pub border_thickness: f32,
    pub border_softness: f32,
    pub border_color: Vec4,
    pub background_color1: Vec4,
    pub background_color2: Vec4,
    pub background_mat: Mat4,
    pub flags: u32,
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "bcadcaba-77eb-48ad-84a0-6716d83bb6a1"]
#[bind_group_data(CustomMaterialKey)]
pub struct CustomMaterial {
    #[uniform(0)]
    material_settings: CustomMaterialUniform,
    #[texture(1)]
    #[sampler(2)]
    texture: Option<Handle<Image>>,
    blend_state: Option<BlendState>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CustomMaterialKey {
    blend_state: Option<BlendState>,
}

impl From<&CustomMaterial> for CustomMaterialKey {
    fn from(material: &CustomMaterial) -> Self {
        CustomMaterialKey {
            blend_state: material.blend_state,
        }
    }
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/rounded_sdf.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target) = fragment.targets[0].as_mut() {
                target.blend = key.bind_group_data.blend_state;
            }
        }
        Ok(())
    }
}
