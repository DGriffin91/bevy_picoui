
// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_view_bindings.wgsl
//     https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_view_types.wgsl
//         https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/view.wgsl
#import bevy_pbr::mesh_view_bindings

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_bindings.wgsl
//     https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_types.wgsl
#import bevy_pbr::mesh_bindings

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/utils.wgsl
#import bevy_pbr::utils

#import bevy_coordinate_systems::transformations

struct FragmentInput {
    /// Based on the winding order of the triangle (not the normals), true if we are facing the front of the triangle
    @builtin(front_facing) is_front: bool,

    /// frag_coord.xy: current fragment position located at half-pixel centers
    /// [0 .. render target viewport size] eg. [(0.0, 0.0) .. (1280.0, 720.0)]
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

    let ndc_a = frag_coord_to_ndc(in.frag_coord);
    let ndc_b = position_world_to_ndc(in.world_position.xyz);
    let ndc_c = position_view_to_ndc(position_world_to_view(in.world_position.xyz));
    let compare_ndc_1 = f32(distance(ndc_a, ndc_b) < 0.000001);
    let compare_ndc_2 = f32(distance(ndc_a, ndc_c) < 0.000001);

    let p = direction_world_to_view(direction_view_to_world(in.world_position.xyz));
    let compare_pos = f32(distance(p, in.world_position.xyz) < 0.0000001);

    let world_a = position_view_to_world(position_ndc_to_view(ndc_a));
    let world_b = position_ndc_to_world(ndc_a);
    let compare_ndc_to_world_1 = f32(distance(world_a, in.world_position.xyz) < 0.00001);
    let compare_ndc_to_world_2 = f32(distance(world_b, in.world_position.xyz) < 0.00001);

    let ndc_dir_a = direction_ndc_to_world(direction_world_to_ndc(N));

    let w1 = direction_clip_to_world(direction_world_to_clip(N));

    return vec4(vec3(w1), 1.0);
}