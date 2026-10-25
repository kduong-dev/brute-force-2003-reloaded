// The console's glare (src/glare.rs, #289), its three passes, each over the whole view:
//  - bright: the picture into the 240-row target (h_0796c3b0), each texel the average of the
//    picture's pixels it covers, kept where its luminance is above its key and black elsewhere
//    (FUN_000e5f60's first draw, pixel shader 0x1f; the key is the stencil value / 255, which
//    the picture's alpha carries here);
//  - blur: 24 copies of that target, 1, 2 and 3 steps out in eight directions 45 degrees apart,
//    each x 25 / 255, added and clamped as the 8-bit target saturates (the second draw: pixel
//    shader 0x20, the quads of FUN_000e47b0);
//  - composite: the picture with the blur, stretched over it, added (FUN_000e5d40: pixel shader
//    0x3a, ONE / ONE).
// All in the frame buffer's values (sRGB), as the console's combiners and blending take its
// bytes; the targets hold those values (Rgba8Unorm).

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// glare.rs `Glare`
struct Glare {
    // xy: a copy's step in the picture's uv; zw: the bright target's size in texels
    step: vec4<f32>,
    // x: each copy's weight; y: BF_GLARE_DEBUG's view (0 the picture, 1 the keys, 2 the blur)
    weight: vec4<f32>,
}

// the pass's source (the picture; the bright target for the blur) and, for the composite, the
// blur's target
@group(0) @binding(0) var first: texture_2d<f32>;
@group(0) @binding(1) var second: texture_2d<f32>;
@group(0) @binding(2) var linear_clamp: sampler;
@group(0) @binding(3) var<uniform> glare: Glare;

// The luminance weights as pixel shader 0x1f's c0 holds them: (0.299, 0.587, 0.114) packed into
// a colour's bytes by FUN_000a4d80 (x 255, rounded: 76, 150, 29).
const LUMA: vec3<f32> = vec3<f32>(76.0, 150.0, 29.0) / 255.0;

fn to_gamma(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

// The bright pass. The texel's share of the picture is averaged in the picture's values with its
// key (the console filters t0, the back buffer, and t1, the depth-stencil buffer read as a
// texture, at the target's texel); then the mux of stages 1-2: t1.a + 0.5 - lum at or above 0.5
// gives black, so the texel is kept where it's brighter than its key, and a key of 1 (255, every
// surface not keyed) keeps nothing. Stage 3's x t0.a (the frame buffer's alpha) is taken as 1.
@fragment
fn bright(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(first));
    let ratio = size / glare.step.zw;
    let lo = (in.position.xy - vec2<f32>(0.5)) * ratio;
    let hi = lo + ratio;
    let start = max(vec2<i32>(floor(lo)), vec2<i32>(0));
    let end = min(vec2<i32>(ceil(hi)), vec2<i32>(size));
    var sum = vec4<f32>(0.0);
    var n = 0.0;
    for (var y = start.y; y < end.y; y++) {
        for (var x = start.x; x < end.x; x++) {
            let centre = vec2<f32>(f32(x), f32(y)) + vec2<f32>(0.5);
            if all(centre >= lo) && all(centre < hi) {
                let t = textureLoad(first, vec2<i32>(x, y), 0);
                sum += vec4<f32>(to_gamma(t.rgb), t.a);
                n += 1.0;
            }
        }
    }
    var c = sum / max(n, 1.0);
    if n < 0.5 {
        // (a view smaller than the target: no pixel centre falls in the texel)
        let t = textureSampleLevel(first, linear_clamp, in.uv, 0.0);
        c = vec4<f32>(to_gamma(t.rgb), t.a);
    }
    let kept = select(vec3<f32>(0.0), c.rgb, dot(c.rgb, LUMA) > c.a);
    return vec4<f32>(kept, 1.0);
}

// The blur: the quads of FUN_000e47b0 sample the kept picture at uv + (sin a, cos a) x k x the
// step, a = 0, 45, ... 315 degrees, k = 1, 2, 3, each x its vertex colour, added ONE / ONE.
@fragment
fn blur(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var sum = vec3<f32>(0.0);
    for (var d = 0; d < 8; d++) {
        let a = f32(d) * 0.7853982;
        let dir = vec2<f32>(sin(a), cos(a)) * glare.step.xy;
        for (var k = 1; k <= 3; k++) {
            sum += textureSampleLevel(first, linear_clamp, in.uv + dir * f32(k), 0.0).rgb;
        }
    }
    return vec4<f32>(min(sum * glare.weight.x, vec3<f32>(1.0)), 1.0);
}

// The composite: the blur stretched over the picture (linearly filtered) and added to it in the
// frame buffer's values, clamped as the blend unit clamps. The picture's alpha, which carried the
// keys, goes back to 1.
@fragment
fn composite(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let here = textureLoad(first, vec2<i32>(in.position.xy), 0);
    let glow = textureSampleLevel(second, linear_clamp, in.uv, 0.0).rgb;
    // BF_GLARE_DEBUG: 1 the keys (grey: the threshold; white: none), 2 the blur alone
    if glare.weight.y > 1.5 {
        return vec4<f32>(to_linear(glow), 1.0);
    }
    if glare.weight.y > 0.5 {
        return vec4<f32>(to_linear(vec3<f32>(here.a)), 1.0);
    }
    return vec4<f32>(to_linear(min(to_gamma(here.rgb) + glow, vec3<f32>(1.0))), 1.0);
}
