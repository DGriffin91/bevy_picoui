
#import bevy_sprite::mesh2d_vertex_output  MeshVertexOutput

#import bevy_sprite::mesh2d_view_bindings as view_bindings
#import bevy_sprite::mesh2d_bindings mesh

#import bevy_core_pipeline::tonemapping somewhat_boring_display_transform

const MATERIAL_FLAGS_TEXTURE_BIT: u32 = 1u;

struct CustomMaterial {
    corner_radius: vec4<f32>,
    edge_softness: f32,
    border_thickness: f32,
    border_softness: f32,
    nine_patch: vec4<f32>,
    border_color: vec4<f32>,
    background_color1: vec4<f32>,
    background_color2: vec4<f32>,
    background_mat: mat4x4<f32>,
    flags: u32,
};

@group(1) @binding(0)
var<uniform> m: CustomMaterial;
@group(1) @binding(1)
var texture: texture_2d<f32>;
@group(1) @binding(2)
var texture_sampler: sampler;


fn rounded_box_sdf(center: vec2<f32>, size: vec2<f32>, radius: vec4<f32>) -> f32 {
    var r = radius;
    r = vec4(select(r.zw, r.xy, center.x > 0.0), r.w, r.z);
    r.x  = select(r.y, r.x, center.y > 0.0);
    let q = abs(center) - size + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r.x;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    var border_thickness = m.border_thickness;

    let bg_uv = (m.background_mat * vec4(in.uv - 0.5, 0.0, 1.0)).xy + 0.5;

    var background_color = mix(m.background_color1, m.background_color2, bg_uv.y);




    // Softening the border makes it larger, compensate for that
    border_thickness = max(border_thickness - m.border_softness, 0.0);
    let main_softness_offset = (max(m.border_softness, m.edge_softness));

    let scaleX = length(mesh.model[0].xyz);
    let scaleY = length(mesh.model[1].xyz);
    let right = length(normalize(mesh.model[0].xyz));
    let up = length(normalize(mesh.model[1].xyz));

    // mesh is 1x1 so the x and y scale is the full size of the rect
    let size = vec2(scaleX / right, scaleY / up); 

    if ((m.flags & MATERIAL_FLAGS_TEXTURE_BIT) != 0u) {
        if all(m.nine_patch == vec4(0.0)) {
            background_color = background_color * textureSample(texture, texture_sampler, bg_uv);
        } else {
            let dims = vec2<f32>(textureDimensions(texture).xy);
            var px = bg_uv * size;

            let top_btm = m.nine_patch.x + m.nine_patch.z;
            let right_left = m.nine_patch.y + m.nine_patch.w;
            let xmod = min(dims.x - top_btm, size.x - top_btm);
            let ymod = min(dims.y - right_left, size.y - right_left);

            px.x = select(px.x, px.x % xmod + m.nine_patch.x, 
                                px.x > m.nine_patch.x && px.x < size.x - m.nine_patch.z);
            px.y = select(px.y, px.y % ymod + m.nine_patch.y, 
                                px.y > m.nine_patch.y && px.y < size.y - m.nine_patch.w);

            px.x = select(px.x, px.x - size.x + dims.x, px.x >= size.x - m.nine_patch.z);
            px.y = select(px.y, px.y - size.y + dims.y, px.y >= size.y - m.nine_patch.w);

            background_color = background_color * textureSample(texture, texture_sampler, px / dims);
        }
    }

    let max_radius = min(size.x, size.y) * 0.5;
    let r = min(m.corner_radius, vec4(max_radius));

    let pos = in.uv.xy * size;

    var distance = rounded_box_sdf(pos - (size / 2.0), size / 2.0, r);

    let main_alpha = 1.0 - smoothstep(0.0, m.edge_softness, distance + main_softness_offset); 
    let a = 1.0 - smoothstep(0.0, m.border_softness, -distance - border_thickness - m.border_softness);
    let b = 1.0 - smoothstep(0.0, m.border_softness, distance + m.border_softness);
    let border_alpha = saturate(a * b * f32(m.border_thickness > 0.0));


    //color = mix(color, m.border_color, border_alpha);

    var premult_dst = background_color * main_alpha;
    var premult_src = vec4(m.border_color.rgb * border_alpha, border_alpha);

    // PREMULTIPLIED_ALPHA_BLENDING, BlendComponent::OVER
    let color = (1.0 * premult_src) + ((1.0 - premult_src.a) * premult_dst);
    return color;
}
