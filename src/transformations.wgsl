#define_import_path bevy_coordinate_systems::transformations

// Relevant sources.
// View types:
// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/view.wgsl
// Prepared to be sent to GPU in:
// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/mod.rs#L316-L333

// Same components as View, just renamed to avoid conflict while keeping nice syntax highlighting 
struct ExampleView {
    // Camera projection * inverse_view
    // world to clip
    view_proj: mat4x4<f32>,

    // Camera view * inverse_projection
    // clip to world
    inverse_view_proj: mat4x4<f32>,

    // Camera view
    // view to world
    view: mat4x4<f32>,

    // Camera inverse_view
    // world to view
    inverse_view: mat4x4<f32>,

    // Camera projection
    // view to clip
    projection: mat4x4<f32>,

    // Camera inverse_projection
    // clip to view
    inverse_projection: mat4x4<f32>,

    // Camera world_position
    world_position: vec3<f32>,

    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
    color_grading: ColorGrading,
};


/// World space
/// +y is up

/// View space
/// -z is forward, +x is right, +y is up
/// (0.0, 0.0, -1.0) is linear distance of 1.0 in front of the camera's view relative to the camera's rotation
/// (0.0, 1.0, 0.0) is linear distance of 1.0 above the camera's view relative to the camera's rotation

/// NDC (normalized device coordinate)
/// https://www.w3.org/TR/webgpu/#coordinate-systems
/// (-1.0, -1.0) in NDC is located at the bottom-left corner of NDC
/// (1.0, 1.0) in NDC is located at the top-right corner of NDC
/// Z is depth where 1.0 is near clipping plane, and 0.0 is inf far away

/// UV space
/// 0.0, 0.0 is the top left
/// 1.0, 1.0 is the bottom right


// -----------------
// TO WORLD --------
// -----------------

/// Convert a view space position to world space
fn position_view_to_world(view_pos: vec3<f32>) -> vec3<f32> {
    let world_pos = view.view * vec4(view_pos, 1.0);
    return world_pos.xyz;
}

/// Convert a clip space position to world space
fn position_clip_to_world(clip_pos: vec4<f32>) -> vec3<f32> {
    let world_pos = view.inverse_view_proj * clip_pos;
    return world_pos.xyz;
}

/// Convert a ndc space position to world space
fn position_ndc_to_world(ndc_pos: vec3<f32>) -> vec3<f32> {
    let world_pos = view.inverse_view_proj * vec4(ndc_pos, 1.0);
    return world_pos.xyz / world_pos.w;
}

/// Convert a view space direction to world space
fn direction_view_to_world(view_dir: vec3<f32>) -> vec3<f32> {
    let world_dir = view.view * vec4(view_dir, 0.0);
    return world_dir.xyz;
}

/// Convert a clip space direction to world space
fn direction_clip_to_world(clip_dir: vec4<f32>) -> vec3<f32> {
    let world_dir = view.inverse_view_proj * clip_dir;
    return world_dir.xyz;
}

// -----------------
// TO VIEW ---------
// -----------------

/// Convert a world space position to view space
fn position_world_to_view(world_pos: vec3<f32>) -> vec3<f32> {
    let view_pos = view.inverse_view * vec4(world_pos, 1.0);
    return view_pos.xyz;
}

/// Convert a clip space position to view space
fn position_clip_to_view(clip_pos: vec4<f32>) -> vec3<f32> {
    let view_pos = view.inverse_projection * clip_pos;
    return view_pos.xyz / view_pos.w;
}

/// Convert a ndc space position to view space
fn position_ndc_to_view(ndc_pos: vec3<f32>) -> vec3<f32> {
    let view_pos = view.inverse_projection * vec4(ndc_pos, 1.0);
    return view_pos.xyz / view_pos.w;
}

/// Convert a world space direction to view space
fn direction_world_to_view(world_dir: vec3<f32>) -> vec3<f32> {
    let view_dir = view.inverse_view * vec4(world_dir, 0.0);
    return view_dir.xyz;
}

/// Convert a clip space direction to view space
fn direction_clip_to_view(clip_dir: vec4<f32>) -> vec3<f32> {
    let view_dir = view.inverse_projection * clip_dir;
    return view_dir.xyz;
}

// -----------------
// TO CLIP ---------
// -----------------

/// Convert a world space position to clip space
fn position_world_to_clip(world_pos: vec3<f32>) -> vec4<f32> {
    let clip_pos = view.view_proj * vec4(world_pos, 1.0);
    return clip_pos;
}

/// Convert a view space position to clip space
fn position_view_to_clip(view_pos: vec3<f32>) -> vec4<f32> {
    let clip_pos = view.projection * vec4(view_pos, 1.0);
    return clip_pos;
}

/// Convert a world space direction to clip space
fn direction_world_to_clip(world_dir: vec3<f32>) -> vec4<f32> {
    let clip_dir = view.view_proj * vec4(world_dir, 0.0);
    return clip_dir;
}

/// Convert a view space direction to clip space
fn direction_view_to_clip(view_dir: vec3<f32>) -> vec4<f32> {
    let clip_dir = view.projection * vec4(view_dir, 0.0);
    return clip_dir;
}

// -----------------
// TO NDC ----------
// -----------------

/// Convert a world space position to ndc space
fn position_world_to_ndc(world_pos: vec3<f32>) -> vec3<f32> {
    let ndc_pos = view.view_proj * vec4(world_pos, 1.0);
    return ndc_pos.xyz / ndc_pos.w;
}

/// Convert a view space position to ndc space
fn position_view_to_ndc(view_pos: vec3<f32>) -> vec3<f32> {
    let ndc_pos = view.projection * vec4(view_pos, 1.0);
    return ndc_pos.xyz / ndc_pos.w;
}


/// Retrieve the camera near clipping plane
fn camera_near() -> f32 {
    return view.projection[3][2];
}

/// Convert ndc space depth to linear world space
fn depth_ndc_to_linear(ndc_depth: f32) -> f32 {
    return camera_near() / ndc_depth;
}

/// Convert linear space depth to ndc world space
fn depth_linear_to_ndc(linear_depth: f32) -> f32 {
    return camera_near() / linear_depth;
}

/// Convert ndc space xy coordinate [-1.0 .. 1.0] to uv [0.0 .. 1.0]
fn ndc_to_uv(ndc: vec2<f32>) -> vec2<f32> {
    return ndc * vec2(0.5, -0.5) + vec2(0.5);
}

/// Convert uv [0.0 .. 1.0] coordinate to ndc space xy [-1.0 .. 1.0]
fn uv_to_ndc(uv: vec2<f32>) -> vec2<f32> {
    return (uv - vec2(0.5)) * vec2(2.0, -2.0);
}

/// returns the (0.0, 0.0) .. (1.0, 1.0) position within the viewport for the current render target
/// [0 .. render target viewport size] eg. [(0.0, 0.0) .. (1280.0, 720.0)] to [(0.0, 0.0) .. (1.0, 1.0)]
fn frag_coord_to_uv(frag_coord: vec2<f32>) -> vec2<f32> {
    return (frag_coord - view.viewport.xy) / view.viewport.zw;
}

/// Convert frag coord to ndc
fn frag_coord_to_ndc(frag_coord: vec4<f32>) -> vec3<f32> {
    return vec3(uv_to_ndc(frag_coord_to_uv(frag_coord.xy)), frag_coord.z);
}
