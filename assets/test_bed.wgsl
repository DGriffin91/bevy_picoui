#define VIEW_PROJECTION_PERSPECTIVE

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_view_bindings.wgsl
//     https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_view_types.wgsl
//         https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/view.wgsl
#import bevy_pbr::mesh_view_bindings view

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_bindings.wgsl
//     https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/mesh_types.wgsl
#import bevy_pbr::mesh_bindings

// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_pbr/src/render/utils.wgsl
#import bevy_pbr::utils

#import bevy_coordinate_systems::view_transformations as vt

struct MeshVertexOutput {
    /// Based on the winding order of the triangle (not the normals), true if we are facing the front of the triangle
    @builtin(front_facing) is_front: bool,
    
    /// frag_coord.xy: current fragment position located at half-pixel centers
    /// [0 .. render target viewport size] eg. [(0.0, 0.0) .. (1280.0, 720.0)]
    /// frag_coord.z: current fragment depth where 1.0 is near clipping plane, and 0.0 is inf far away
    ///     (near / in.frag_coord.z is the linear world space distance from camera)
    /// frag_coord.w: current fragment depth where 1.0 / frag_coord.w is the linear world space distance from camera
    @builtin(position) frag_coord: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    #ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
    #endif
    #ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
    #endif
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
}



@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    var N = normalize(in.world_normal);
    var V = normalize(view.world_position.xyz - in.world_position.xyz);

    let ndc_a = vt::frag_coord_to_ndc(in.frag_coord);
    let ndc_b = vt::position_world_to_ndc(in.world_position.xyz);
    let ndc_c = vt::position_view_to_ndc(vt::position_world_to_view(in.world_position.xyz));
    let compare_ndc_1 = f32(distance(ndc_a, ndc_b) < 0.000001);
    let compare_ndc_2 = f32(distance(ndc_a, ndc_c) < 0.000001);

    let p = vt::direction_world_to_view(vt::direction_view_to_world(in.world_position.xyz));
    let compare_pos = f32(distance(p, in.world_position.xyz) < 0.0000001);

    let world_a = vt::position_view_to_world(vt::position_ndc_to_view(ndc_a));
    let world_b = vt::position_ndc_to_world(ndc_a);
    let compare_ndc_to_world_1 = f32(distance(world_a, in.world_position.xyz) < 0.00001);
    let compare_ndc_to_world_2 = f32(distance(world_b, in.world_position.xyz) < 0.00001);

    let world_to_clip_pos = vt::position_clip_to_world(vt::position_world_to_clip(in.world_position.xyz));
    let compare_clip_to_world_pos = f32(distance(world_to_clip_pos, in.world_position.xyz) < 0.00001);

    let world_to_clip_dir = vt::direction_clip_to_world(vt::direction_world_to_clip(N));
    let compare_clip_to_world_dir = f32(distance(world_to_clip_dir, N) < 0.00000001);

    let a_dir = vt::direction_view_to_world(vt::direction_clip_to_view(vt::direction_view_to_clip(vt::direction_world_to_view(N))));
    let compare_a_dir = f32(distance(a_dir, N) < 0.0000001);

    return vec4(vec3(compare_a_dir), 1.0);
}