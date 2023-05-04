
// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_view_bindings.wgsl
//     https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_view_types.wgsl
//         https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/view.wgsl
#import bevy_pbr::mesh_view_bindings

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_bindings.wgsl
//     https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_types.wgsl
#import bevy_pbr::mesh_bindings

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/utils.wgsl
#import bevy_pbr::utils

#import "transformations.wgsl"

struct FragmentInput {
    /// Based on the winding order of the triangle (not the normals), true if we are facing the front of the triangle
    @builtin(front_facing) is_front: bool,

    /// frag_coord.xy: current fragment position located at half-pixel centers
    /// [0 .. render target viewport size] as in [(0.0, 0.0) .. (1280.0, 720.0)]
    /// frag_coord.z: current fragment depth where 1.0 is near clipping plane, and 0.0 is inf far away
    ///     (near / in.frag_coord.z is the linear world space distance from camera)
    /// frag_coord.w: current fragment depth where 1.0 / frag_coord.w is the linear world space distance from camera
    @builtin(position) frag_coord: vec4<f32>,

    // https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_vertex_output.wgsl
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var N = normalize(in.world_normal);
    var V = normalize(view.world_position.xyz - in.world_position.xyz);

    return vec4(N, 1.0);
}