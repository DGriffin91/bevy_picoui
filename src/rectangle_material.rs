use std::hash::Hasher;

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, BlendState, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use crate::hash::hash_vec4;

pub const RECTANGLE_MATERIAL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 98327450932847);

pub struct RectangleMaterialPlugin;

impl Plugin for RectangleMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<RectangleMaterial>::default());
        load_internal_asset!(
            app,
            RECTANGLE_MATERIAL_HANDLE,
            "rectangle_material.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(ShaderType, Debug, Clone, Default)]
pub struct RectangleMaterialUniform {
    pub corner_radius: Vec4,
    pub edge_softness: f32,
    pub border_thickness: f32,
    pub border_softness: f32,
    pub nine_patch: Vec4,
    pub border_color: Vec4,
    pub background_color1: Vec4,
    pub background_color2: Vec4,
    pub background_mat: Mat4,
    pub flags: u32,
}

impl core::hash::Hash for RectangleMaterialUniform {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_vec4(&self.corner_radius, state);
        self.edge_softness.to_bits().hash(state);
        self.border_thickness.to_bits().hash(state);
        self.border_softness.to_bits().hash(state);
        hash_vec4(&self.border_color, state);
        hash_vec4(&self.background_color1, state);
        hash_vec4(&self.background_color2, state);
        hash_vec4(&self.background_mat.x_axis, state);
        hash_vec4(&self.background_mat.y_axis, state);
        hash_vec4(&self.background_mat.z_axis, state);
        hash_vec4(&self.background_mat.w_axis, state);
        self.flags.hash(state);
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone, Hash, Default)]
#[uuid = "56befa7d-4f01-4bf5-bd63-7b7b45ff5af6"]
#[bind_group_data(RectangleMaterialKey)]
pub struct RectangleMaterial {
    #[uniform(0)]
    pub material_settings: RectangleMaterialUniform,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    pub blend_state: Option<BlendState>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RectangleMaterialKey {
    blend_state: Option<BlendState>,
}

impl From<&RectangleMaterial> for RectangleMaterialKey {
    fn from(material: &RectangleMaterial) -> Self {
        RectangleMaterialKey {
            blend_state: material.blend_state,
        }
    }
}

impl Material2d for RectangleMaterial {
    fn fragment_shader() -> ShaderRef {
        RECTANGLE_MATERIAL_HANDLE.typed().into()
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
