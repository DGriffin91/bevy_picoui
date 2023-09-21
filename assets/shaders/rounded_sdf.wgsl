
#import bevy_sprite::mesh2d_vertex_output  MeshVertexOutput

#import bevy_sprite::mesh2d_view_bindings as view_bindings
#import bevy_sprite::mesh2d_bindings mesh

#import bevy_core_pipeline::tonemapping somewhat_boring_display_transform


fn rounded_box_sdf(center: vec2<f32>, size: vec2<f32>, radius: vec4<f32>) -> f32 {
    var r = radius;
    r = vec4(select(r.zw, r.xy, center.x > 0.0), r.w, r.z);
    r.x  = select(r.y, r.x, center.y > 0.0);
    let q = abs(center) - size + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r.x;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let corner_radius = 100.0;
    let edge_softness = 1.0;
    var border_thickness = 0.0;
    let border_softness = 0.0;
    let border_color = vec4(1.0, 0.0, 0.0, 1.0);
    let background_color = vec4(0.2, 0.2, 0.6, 1.0);
    // TODO add gradient, texture

    // Softening the border makes it larger, compensate for that
    border_thickness = max(border_thickness - border_softness, 0.0);
    let main_softness_offset = (max(border_softness, edge_softness));

    // When the border or rect is softened we need to make the whole thing smaller to fit in the rect
    let softness_offset = max(border_softness, edge_softness);

    // mesh is 1x1 so the x and y scale is the full size of the rect
    let size = vec2(mesh.model[0][0], mesh.model[1][1]); 

    let max_radius = min(size.x, size.y)* 0.5;
    let r = min(corner_radius, max_radius);

    let pos = in.uv.xy * size;

    var distance = rounded_box_sdf(pos - (size / 2.0), size / 2.0, vec4(r));

    let main_alpha = 1.0 - smoothstep(0.0, edge_softness, distance + main_softness_offset); 
    let a = 1.0 - smoothstep(0.0, border_softness, -distance - border_thickness - softness_offset);
    let b = 1.0 - smoothstep(0.0, border_softness, distance + softness_offset);
    let border_alpha  = a * b;

    var color = background_color;
    color *= main_alpha;
    color = mix(color, border_color, border_alpha);

    return color;
}
