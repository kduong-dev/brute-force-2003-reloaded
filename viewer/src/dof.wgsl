// The screen's depth of field (#38): shader 0x1e of the renderer's stage 8 (FUN_000e7d40's composite
// FUN_000e5730). The game draws three copies of the picture — the frame, a half-size one and a
// quarter-size one, each a 2 x 2 box step of the one before — and one full-screen quad over the
// frame (SRC_ALPHA / INV_SRC_ALPHA) whose pixel shader picks the copy by the depth:
//
//   rgb   = 1/4 sharp + 1/2 half + 1/4 quarter
//   alpha = clamp(64 (2S - 1) |z - focus| + bias, 0, 1)
//
// with c0 = S (+0x117: 1 in the scope, 0.8 at low health), c1 = the focus (+0x118: the aim ray's
// hit distance eased, or the character's own depth) and c2 = (bias + 1) / 2 (+0x120: hp / 0.5 - 1
// at low health), the depth buffer bound as a texture (t0.a: the depth's top 8 bits).
//
// Here the two copies are made by sampling the picture where four pixels meet (a bilinear tap on
// their corner is their 2 x 2 average) and where four 2 x 2 blocks meet, four taps: the same box
// steps without separate targets. The focus is in metres and the depth comes from bevy's reverse-Z
// depth texture (perspective_infinite_reverse_rh: the stored value is near / z), so the alpha's
// |z - focus| is in metres as the game's is; the game's 8-bit depth byte quantises it more
// coarsely than this does. The console blends its frame buffer's values (sRGB), and so does this
// pass (the picture read as sRGB, the result written back), as the scope view's does.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Dof {
    // c0: S (1 in the scope, 0.8 at low health)
    s: f32,
    // c1: the focus's distance from the camera (m)
    focus: f32,
    // the bias (+0x120), as c2 1: hp / 0.5 - 1 at low health, 0 in the scope
    bias: f32,
    // the camera's near plane (m): the depth texture's value is near / z
    near: f32,
    // the picture's size in pixels (x, y), for the taps
    screen: vec4<f32>,
};

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var picture_sampler: sampler;
@group(0) @binding(2) var<uniform> dof: Dof;
#ifdef MULTISAMPLED
@group(0) @binding(3) var depth_texture: texture_depth_multisampled_2d;
#else
@group(0) @binding(3) var depth_texture: texture_depth_2d;
#endif

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

// The picture at `uv`, in the frame buffer's values.
fn at(uv: vec2<f32>) -> vec3<f32> {
    return to_gamma(textureSample(picture, picture_sampler, uv).rgb);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = vec2<f32>(1.0 / dof.screen.x, 1.0 / dof.screen.y);
    let sharp = at(in.uv);
    // the half-size copy: a tap where four pixels meet is their 2 x 2 average
    let half = at(in.uv + vec2<f32>(0.5, 0.5) * t);
    // the quarter-size copy: four such blocks round it, the 4 x 4 box
    var quarter = vec3<f32>(0.0);
    for (var i = 0; i < 4; i++) {
        let o = vec2<f32>(1.0, 1.0) * t * vec2<f32>(select(-1.0, 1.0, (i & 1) == 0), select(-1.0, 1.0, (i & 2) == 0));
        quarter += at(in.uv + o);
    }
    quarter *= 0.25;
    let mixed = 0.25 * sharp + 0.5 * half + 0.25 * quarter;
    // the depth at this pixel (multisampled: sample 0, the same pixel the main pass's quad covers)
    let pixel = vec2<i32>(in.uv * dof.screen.xy);
    let z = dof.near / max(textureLoad(depth_texture, pixel, 0), 1e-6);
    let alpha = clamp(64.0 * (2.0 * dof.s - 1.0) * abs(z - dof.focus) + dof.bias, 0.0, 1.0);
    return vec4<f32>(to_linear(mix(sharp, mixed, alpha)), 1.0);
}
