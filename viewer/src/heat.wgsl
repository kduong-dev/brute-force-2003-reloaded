// A character as a heat figure in Brutus's Vengar view (src/bin/play_vengar.rs, `HeatMaterial`):
// unlit, its colour by its normal in view space. The game draws a living character in vision mode
// 2 with the material at the vision set's +0x10 (class 0xe2c39ac2, FUN_000a8610): vertex shader
// 0x69 gives each vertex the colour (k, k, 0) and the texture coordinate (n.xy + c17.w) x c18.w +
// c28.xy from its normal in view space (c17.w 1 and c18.w 0.5, the vision lights' table at
// 0x41b3a8; c28.xy (0.01, 0.15 + a scroll that's 0 here)), and pixel shader 0x44 multiplies the
// colour by the texture there. So a normal facing the view lands 0.15 below the texture's middle,
// one facing up 0.65 from it, down 0.35, sideways 0.52. The texture it samples wasn't found: the
// colours here are a radial ramp about the middle, fitted to the takes (yellow facing the view,
// orange to red toward the rims and on what faces up: take08's shoulders).

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Heat {
    // the colour at the texture's middle (sRGB), and the alpha
    middle: vec4<f32>,
    // the colour at the ramp's far end (sRGB), and the curve's power (w)
    edge: vec4<f32>,
    // where the ramp starts and ends, as distances from the texture's middle (x, y)
    ramp: vec4<f32>,
};

@group(2) @binding(0) var<uniform> heat: Heat;

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var n = normalize(in.world_normal);
    if !is_front {
        n = -n;
    }
    let v = normalize((view.view_from_world * vec4<f32>(n, 0.0)).xyz);
    // the texture coordinate's offset from the middle (x right, y up in view space, as D3D's)
    let off = v.xy * 0.5 + vec2<f32>(0.01, 0.15);
    let k = pow(clamp((length(off) - heat.ramp.x) / max(heat.ramp.y - heat.ramp.x, 1e-3), 0.0, 1.0), heat.edge.w);
    let c = mix(heat.middle.rgb, heat.edge.rgb, k);
    return vec4<f32>(to_linear(c), heat.middle.a);
}
