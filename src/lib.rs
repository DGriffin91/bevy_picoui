use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid};

pub const VIEW_TRANSFORMATIONS: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4396331565425081187);

pub struct CoordinateTransformationsPlugin;

impl Plugin for CoordinateTransformationsPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            VIEW_TRANSFORMATIONS,
            "view_transformations.wgsl",
            Shader::from_wgsl
        );
    }
}
