use bevy::{
    asset::load_internal_asset,
    math::{uvec4, vec2, Vec4Swizzles},
    prelude::*,
    reflect::TypeUuid,
    render::camera::{CameraProjection, TemporalJitter},
    transform::systems::propagate_transforms,
};

pub mod im3dtext;

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
        // NOTE: The view will reflect the last frame unless the system you run is after prepare_view
        // TODO Maybe update this incrementally as change detection for the camera is triggered?
        //      (Would require manually computing the GlobalTransform)
        app.add_systems(PostUpdate, prepare_view.after(propagate_transforms));
    }
}

// https://github.com/bevyengine/bevy/blob/a9e50dd80178bc50b644f557511111ba2c265a56/crates/bevy_render/src/camera/camera.rs#L637
// https://github.com/bevyengine/bevy/blob/a9e50dd80178bc50b644f557511111ba2c265a56/crates/bevy_render/src/view/mod.rs#L347
pub fn prepare_view(
    mut commands: Commands,
    views: Query<(
        Entity,
        &Camera,
        &GlobalTransform,
        &Projection,
        Option<&TemporalJitter>,
    )>,
) {
    for (entity, camera, transform, projection, temporal_jitter) in &views {
        let projection_type = match projection {
            Projection::Perspective(_) => ProjectionType::Perspective,
            Projection::Orthographic(_) => ProjectionType::Orthographic,
        };
        let viewport = camera.viewport.clone().unwrap_or_default();
        let viewport = uvec4(
            viewport.physical_position.x,
            viewport.physical_position.y,
            viewport.physical_size.x,
            viewport.physical_size.y,
        )
        .as_vec4();
        let unjittered_projection = projection.get_projection_matrix();
        let mut projection = unjittered_projection;

        if let Some(temporal_jitter) = temporal_jitter {
            temporal_jitter.jitter_projection(&mut projection, viewport.zw());
        }

        let inverse_projection = projection.inverse();
        let view = transform.compute_matrix();
        let inverse_view = view.inverse();

        let view_proj = projection * inverse_view;

        commands.entity(entity).insert(View {
            view_proj,
            unjittered_view_proj: unjittered_projection * inverse_view,
            inverse_view_proj: view * inverse_projection,
            view,
            inverse_view,
            projection,
            inverse_projection,
            world_position: transform.translation(),
            viewport,
            projection_type,
        });
    }
}

#[derive(Clone)]
pub enum ProjectionType {
    Orthographic,
    Perspective,
}

#[derive(Clone, Component)]
pub struct View {
    pub view_proj: Mat4,
    pub unjittered_view_proj: Mat4,
    pub inverse_view_proj: Mat4,
    pub view: Mat4,
    pub inverse_view: Mat4,
    pub projection: Mat4,
    pub inverse_projection: Mat4,
    pub world_position: Vec3,
    // viewport(x_origin, y_origin, width, height)
    pub viewport: Vec4,
    pub projection_type: ProjectionType,
}

impl View {
    /// World space:
    /// +y is up

    /// View space:
    /// -z is forward, +x is right, +y is up
    /// Forward is from the camera position into the scene.
    /// (0.0, 0.0, -1.0) is linear distance of 1.0 in front of the camera's view relative to the camera's rotation
    /// (0.0, 1.0, 0.0) is linear distance of 1.0 above the camera's view relative to the camera's rotation

    /// NDC (normalized device coordinate):
    /// https://www.w3.org/TR/webgpu/#coordinate-systems
    /// (-1.0, -1.0) in NDC is located at the bottom-left corner of NDC
    /// (1.0, 1.0) in NDC is located at the top-right corner of NDC
    /// Z is depth where 1.0 is near clipping plane, and 0.0 is inf far away

    /// UV space:
    /// 0.0, 0.0 is the top left
    /// 1.0, 1.0 is the bottom right

    // -----------------
    // TO WORLD --------
    // -----------------

    /// Convert a view space position to world space
    pub fn position_view_to_world(&self, view_pos: Vec3) -> Vec3 {
        let world_pos = self.view * view_pos.extend(1.0);
        world_pos.xyz()
    }

    /// Convert a clip space position to world space
    pub fn position_clip_to_world(&self, clip_pos: Vec4) -> Vec3 {
        let world_pos = self.inverse_view_proj * clip_pos;
        world_pos.xyz()
    }

    /// Convert a ndc space position to world space
    pub fn position_ndc_to_world(&self, ndc_pos: Vec3) -> Vec3 {
        let world_pos = self.inverse_view_proj * ndc_pos.extend(1.0);
        world_pos.xyz() / world_pos.w
    }

    /// Convert a view space direction to world space
    pub fn direction_view_to_world(&self, view_dir: Vec3) -> Vec3 {
        let world_dir = self.view * view_dir.extend(0.0);
        world_dir.xyz()
    }

    /// Convert a clip space direction to world space
    pub fn direction_clip_to_world(&self, clip_dir: Vec4) -> Vec3 {
        let world_dir = self.inverse_view_proj * clip_dir;
        world_dir.xyz()
    }

    // -----------------
    // TO VIEW ---------
    // -----------------

    /// Convert a world space position to view space
    pub fn position_world_to_view(&self, world_pos: Vec3) -> Vec3 {
        let view_pos = self.inverse_view * world_pos.extend(1.0);
        view_pos.xyz()
    }

    /// Convert a clip space position to view space
    pub fn position_clip_to_view(&self, clip_pos: Vec4) -> Vec3 {
        let view_pos = self.inverse_projection * clip_pos;
        view_pos.xyz() / view_pos.w
    }

    /// Convert a ndc space position to view space
    pub fn position_ndc_to_view(&self, ndc_pos: Vec3) -> Vec3 {
        let view_pos = self.inverse_projection * ndc_pos.extend(1.0);
        view_pos.xyz() / view_pos.w
    }

    /// Convert a world space direction to view space
    pub fn direction_world_to_view(&self, world_dir: Vec3) -> Vec3 {
        let view_dir = self.inverse_view * world_dir.extend(0.0);
        view_dir.xyz()
    }

    /// Convert a clip space direction to view space
    pub fn direction_clip_to_view(&self, clip_dir: Vec4) -> Vec3 {
        let view_dir = self.inverse_projection * clip_dir;
        view_dir.xyz()
    }

    // -----------------
    // TO CLIP ---------
    // -----------------

    /// Convert a world space position to clip space
    pub fn position_world_to_clip(&self, world_pos: Vec3) -> Vec4 {
        self.view_proj * world_pos.extend(1.0)
    }

    /// Convert a view space position to clip space
    pub fn position_view_to_clip(&self, view_pos: Vec3) -> Vec4 {
        self.projection * view_pos.extend(1.0)
    }

    /// Convert a world space direction to clip space
    pub fn direction_world_to_clip(&self, world_dir: Vec3) -> Vec4 {
        self.view_proj * world_dir.extend(0.0)
    }

    /// Convert a view space direction to clip space
    pub fn direction_view_to_clip(&self, view_dir: Vec3) -> Vec4 {
        self.projection * view_dir.extend(0.0)
    }

    // -----------------
    // TO NDC ----------
    // -----------------

    /// Convert a world space position to ndc space
    pub fn position_world_to_ndc(&self, world_pos: Vec3) -> Vec3 {
        let ndc_pos = self.view_proj * world_pos.extend(1.0);
        ndc_pos.xyz() / ndc_pos.w
    }

    /// Convert a view space position to ndc space
    pub fn position_view_to_ndc(&self, view_pos: Vec3) -> Vec3 {
        let ndc_pos = self.projection * view_pos.extend(1.0);
        ndc_pos.xyz() / ndc_pos.w
    }

    // -----------------
    // DEPTH -----------
    // -----------------

    /// Retrieve the perspective camera near clipping plane
    pub fn perspective_camera_near(&self) -> f32 {
        self.projection.w_axis[2]
    }

    /// Convert ndc depth to linear view z.
    /// Note: Depth values in front of the camera will be negative as -z is forward
    pub fn depth_ndc_to_view_z(&self, ndc_depth: f32) -> f32 {
        match self.projection_type {
            ProjectionType::Orthographic => {
                -(self.projection.w_axis[2] - ndc_depth) / self.projection.z_axis[2]
            }
            ProjectionType::Perspective => -self.perspective_camera_near() / ndc_depth,
        }

        //let view_pos = self.inverse_projection * vec4(0.0, 0.0, ndc_depth, 1.0);
        //return view_pos.z / view_pos.w;
    }

    /// Convert linear view z to ndc depth.
    /// Note: View z input should be negative for values in front of the camera as -z is forward
    pub fn view_z_to_depth_ndc(&self, view_z: f32) -> f32 {
        match self.projection_type {
            ProjectionType::Orthographic => {
                self.projection.w_axis[2] + view_z * self.projection.z_axis[2]
            }
            ProjectionType::Perspective => -self.perspective_camera_near() / view_z,
        }
        //let ndc_pos = self.projection * vec4(0.0, 0.0, view_z, 1.0);
        //return ndc_pos.z / ndc_pos.w;
    }

    // -----------------
    // UV --------------
    // -----------------

    /// returns the (0.0, 0.0) .. (1.0, 1.0) position within the viewport for the current render target
    /// [0 .. render target viewport size] eg. [(0.0, 0.0) .. (1280.0, 720.0)] to [(0.0, 0.0) .. (1.0, 1.0)]
    pub fn frag_coord_to_uv(&self, frag_coord: Vec2) -> Vec2 {
        (frag_coord - self.viewport.xy()) / self.viewport.zw()
    }

    /// Convert frag coord to ndc
    pub fn frag_coord_to_ndc(&self, frag_coord: Vec4) -> Vec3 {
        uv_to_ndc(self.frag_coord_to_uv(frag_coord.xy())).extend(frag_coord.z)
    }
}

/// Convert ndc space xy coordinate [-1.0 .. 1.0] to uv [0.0 .. 1.0]
pub fn ndc_to_uv(ndc: Vec2) -> Vec2 {
    ndc * vec2(0.5, -0.5) + Vec2::splat(0.5)
}

/// Convert uv [0.0 .. 1.0] coordinate to ndc space xy [-1.0 .. 1.0]
pub fn uv_to_ndc(uv: Vec2) -> Vec2 {
    (uv - Vec2::splat(0.5)) * vec2(2.0, -2.0)
}
