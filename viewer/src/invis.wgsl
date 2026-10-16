// Hawk's refraction while she's stealthed (src/bin/play_stealth.rs, `InvisMaterial`): the game's
// BF_InvisMaterial (class 0xf3bf04d1; todo/198 leads §4). Its draw (0xa1ba0) runs vertex program
// 0x3f (0x3d9678; 0x40 and 0x41 the same after skinning) and pixel shader 0x26 (0x3e0410) over
// her whole model and the guns she holds, with one texture, the frame copy (0xa1a40: the back
// buffer copied after the sorted transparent pass and the decals, FUN_000e4da0), clamped:
//  - the lookup: the vertex's place on screen shifted by c24.w x n.z x (n.x, n.y) in NDC (y up),
//    n its normal in world space (c20-c22 the node's world transform, not renormalised) and
//    c24.w the controller's offset (play_skills.rs `Cloak`), then made the copy's coordinate
//    (((ndc + shift) + 1) x 0.5, y flipped, x the copy's size in texels: c25);
//  - the colour: c26.rgb + (-0.5 n.z) x (1 - c26.rgb) = (0.9 - 0.05 n.z, 1, 1), its alpha the
//    material's (c26.w);
//  - the pixel: the copy there times the colour (r0 = t0 x v0), fogged by her own depth (the
//    final combiner's fog.a x r0 + (1 - fog.a) x fog), blended over the picture by the alpha
//    (SRCALPHA / INVSRCALPHA).
// The console multiplies the copy's stored (sRGB) values, so the colour is applied to those here.
// The game works the shift out per vertex and interpolates it; here per fragment from the
// interpolated normal (the same where the normals vary smoothly). The shift is a share of the
// game's 4:3 screen; the demo's view is narrower, so it's made the same angle here (`screen`,
// play_stealth.rs `game_screen`).

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::{view, view_transmission_texture, view_transmission_sampler},
}
#ifdef DISTANCE_FOG
#import bevy_pbr::{mesh_view_bindings::fog, fog::linear_fog}
#endif

// x the lookup's offset (c24.w), y the alpha (c26.w), z the colour's red (c26.x)
@group(2) @binding(0) var<uniform> invis: vec4<f32>;
// xy the game's screen, half-width and half-height, in the demo's view's half-heights
@group(2) @binding(1) var<uniform> screen: vec4<f32>;

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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = in.world_normal;
    let shift = invis.x * n.z * n.xy;
    // the game's screen (NDC, y up) to the copy's pixels (y down), kept inside the view (the
    // stage's clamp: the game's copy is the viewport's size)
    let size = vec2<f32>(textureDimensions(view_transmission_texture));
    let at = in.position.xy + vec2<f32>(shift.x * screen.x, -shift.y * screen.y) * 0.5 * view.viewport.w;
    let lo = view.viewport.xy + vec2<f32>(0.5);
    let hi = view.viewport.xy + view.viewport.zw - vec2<f32>(0.5);
    let copy = textureSampleLevel(view_transmission_texture, view_transmission_sampler, clamp(at, lo, hi) / size, 0.0).rgb;
    // the vertex colour (the console clamps it to 0..1)
    let red = clamp(invis.z + (-0.5 * n.z) * (1.0 - invis.z), 0.0, 1.0);
    var color = vec4<f32>(to_linear(to_gamma(copy) * vec3<f32>(red, 1.0, 1.0)), invis.y);
#ifdef DISTANCE_FOG
    // the levels' fog is linear (play.rs's DistanceFog), with no light scattering; its own
    // module rather than bevy's whole PBR library, whose compile held up the level's pipelines
    // as a map loads
    color = linear_fog(fog, color, distance(in.world_position.xyz, view.world_position.xyz), vec3<f32>(0.0));
#endif
    return color;
}
