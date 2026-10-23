// The scope's views over the main camera's picture (src/bin/play_scope_view.rs), in the frame
// buffer's values (sRGB), as the console blends its HUD:
//  - the scope frame (0xb4240): the quarter h_1807ec04 1:1 from the picture's middle at the
//    widget's scale, mirrored into the four quadrants, its edge texels carried on past it (CLAMP),
//    blended ZERO / ONE_MINUS_SRC_COLOR (the sprite draw's mode 5): the picture times
//    1 - the texture x the fade;
//  - Flint's overlay (FUN_000b3970): the ring h_f0d8a0a7 and the smear h_f5b6a9f8, each in four
//    mirrored quads from the middle out, blended ONE / ONE (mode 2): the texels times their grey
//    vertex colours x the fade, added.
// The sprite draw's texture coordinates (0xba950): u from 0 at the quad's first x (its left, or
// its right when its width is negated) to its u1 at the other end; v from v1 - v_off at the quad's
// first y to -v_off at the other, the texture's rows running up the screen. So the frame's first
// column and last row sit at the middle (its quads start there) and the overlay's at the
// picture's corners (its quads end there).

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Scope {
    // the frame: x its scale, y its fade (0: not drawn), z and w its quadrant (HUD units)
    frame: vec4<f32>,
    // Flint's overlay: x the ring's colour x the fade, y the smear's (both 0: not drawn)
    flint: vec4<f32>,
    // the ring's quads (HUD units): x and y to the left and up, z and w to the right and down
    ring: vec4<f32>,
    // the smear's quads, the same
    smear: vec4<f32>,
};

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var picture_sampler: sampler;
@group(0) @binding(2) var<uniform> scope: Scope;
@group(0) @binding(3) var frame_texture: texture_2d<f32>;
@group(0) @binding(4) var smear_texture: texture_2d<f32>;
@group(0) @binding(5) var ring_texture: texture_2d<f32>;
@group(0) @binding(6) var clamped: sampler;

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

// One overlay texture at a point `d` (HUD units from the middle, y down): its quad there is
// `quads.xy` to the left and up, `quads.zw` to the right and down; u 1 at the middle to 0 at the
// quad's outer edge, v 0 at the middle to 1 there, clamped beyond.
fn overlay(t: texture_2d<f32>, d: vec2<f32>, quads: vec4<f32>) -> vec3<f32> {
    let size = vec2<f32>(select(quads.z, quads.x, d.x < 0.0), select(quads.w, quads.y, d.y < 0.0));
    let r = abs(d) / max(size, vec2<f32>(1e-3));
    return textureSample(t, clamped, vec2<f32>(1.0 - r.x, r.y)).rgb;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let lin = textureSample(picture, picture_sampler, in.uv);
    var c = to_gamma(lin.rgb);
    // HUD units from the picture's middle: the game's 640 x 480 is the picture's height
    let size = vec2<f32>(textureDimensions(picture));
    let d = (in.uv - 0.5) * size * (480.0 / size.y);
    if scope.frame.y > 0.0 {
        // u 0 at the middle to 1 a quadrant's width out, v 1 at the middle to 0 a height out
        let q = abs(d) / (scope.frame.x * scope.frame.zw);
        let t = textureSample(frame_texture, clamped, vec2<f32>(q.x, 1.0 - q.y)).rgb;
        c = c * (1.0 - t * scope.frame.y);
    }
    if scope.flint.x > 0.0 || scope.flint.y > 0.0 {
        // the ring drawn first, then the smear, both added
        c = c + overlay(ring_texture, d, scope.ring) * scope.flint.x + overlay(smear_texture, d, scope.smear) * scope.flint.y;
        c = min(c, vec3<f32>(1.0));
    }
    return vec4<f32>(to_linear(c), lin.a);
}
