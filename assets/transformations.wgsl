// Relevant sources.
// View types:
// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/view.wgsl
// Prepared to be sent to GPU in:
// https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_render/src/view/mod.rs#L316-L333

// Same components as View, just renamed to avoid conflict while keeping nice syntax highlighting 
struct ExampleView {
    // Camera projection * inverse_view
    view_proj: mat4x4<f32>,

    // Camera view * inverse_projection
    inverse_view_proj: mat4x4<f32>,

    // Camera view
    view: mat4x4<f32>,

    // Camera inverse_view
    inverse_view: mat4x4<f32>,

    // Camera projection
    projection: mat4x4<f32>,

    // Camera inverse_projection
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

/// Clip space or NDC (normalized device coordinate)
/// https://www.w3.org/TR/webgpu/#coordinate-systems
/// (-1.0, -1.0) in NDC is located at the bottom-left corner of NDC
/// (1.0, 1.0) in NDC is located at the top-right corner of NDC
/// Z is depth where 1.0 is near clipping plane, and 0.0 is inf far away

/// UV space
/// 0.0, 0.0 is the top left
/// 1.0, 1.0 is the bottom right

/// Convert view space position to world space
fn position_view_to_world(view_pos: vec3<f32>) -> vec3<f32> {
    let world_pos = view.inverse_view * vec4(view_pos, 1.0);
    return world_pos.xyz;
}

/// Convert world space position to view space
fn position_world_to_view(world_pos: vec3<f32>) -> vec3<f32> {
    let view_pos = view.view * vec4(world_pos, 1.0);
    return view_pos.xyz;
}

/// Convert clip space position to world space
fn position_clip_to_world(clip_pos: vec3<f32>) -> vec3<f32> {
    let world_pos = view.inverse_view_proj * vec4(clip_pos, 1.0);
    return world_pos.xyz / world_pos.w;
}

/// Convert world space position to clip space
fn position_world_to_clip(world_pos: vec3<f32>) -> vec3<f32> {
    let clip_pos = view.view_proj * vec4(world_pos, 1.0);
    return clip_pos.xyz / clip_pos.w;
}

/// Retrieve the camera near clipping plane
fn camera_near() -> f32 {
    return view.projection[3][2];
}

/// Convert clip space depth to linear world space
fn depth_clip_to_linear(clip_depth: f32) -> f32 {
    return camera_near() / clip_depth;
}

/// Convert clip space xy coordinate [-1.0 .. 1.0] to uv [0.0 .. 1.0]
fn clip_to_uv(clip: vec2<f32>) -> vec2<f32> {
    return clip * vec2(0.5, -0.5) + vec2(0.5, 0.5);
}

/// Convert uv [0.0 .. 1.0] coordinate to clip space xy [-1.0 .. 1.0]
fn uv_to_clip(uv: vec2<f32>) -> vec2<f32> {
    return (uv - vec2(0.5)) * vec2(2.0, -2.0);
}