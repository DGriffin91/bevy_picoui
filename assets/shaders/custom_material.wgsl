
#import bevy_sprite::mesh2d_vertex_output  MeshVertexOutput

#import bevy_sprite::mesh2d_view_bindings as view_bindings

#import bevy_core_pipeline::tonemapping somewhat_boring_display_transform

fn modf(x: vec2<f32>, y: vec2<f32>) -> vec2<f32> {
    return x - y * floor(x/y);
}

// Original shader by Danilo Guanabara
// https://www.shadertoy.com/view/XsXXDn

@fragment
fn fragment(
    in: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    let fragCoord = in.position;

    var c = vec3<f32>(0.0, 0.0, 0.0);
    var l = 0.0;
    var z = view_bindings::globals.time; 
    var r = view_bindings::view.viewport.zw; 

    for (var i: u32 = 0u; i < 3u; i = i + 1u) {
        var uv = in.position.xy / r;
        var p = in.position.xy / r;

        uv = p;
        p -= vec2(0.5);
        p.x *= r.x / r.y;
        z += 0.07;
        l = length(p);
        uv = uv + p / l * (sin(z) + 1.0) * abs(sin(l * 9.0 - z - z));
        c[i] = 0.01 / length(modf(uv, vec2(1.0)) - vec2(0.5));
    }
    c/=l;
    c = somewhat_boring_display_transform(c);
    return vec4(pow(c, vec3(2.2)), 1.0);
}
