// A level's liquid surface (level_scene::LiquidMaterial): the game's animated liquid shader
// (h_f124a774: lava, toxic rivers) and its water (h_0f8904b9), as bevy's standard material whose
// colour is replaced by the moving layers:
//   layers: texture-0 and texture-1, each a tile `texture-scale` of the mesh's uv across (lava
//           0.1: about 10 m) scrolling at its own speed (tiles per second), combined by
//           texture-1's blend mode (0: over by its alpha, 1: added, else: multiplied x 2)
//   water:  texture-0 tiled and scrolled, tinted by `color`; over it the reflection (the sky's
//           colour tinted by color-1: the game's is a texture rendered at run time) by the
//           fresnel ramp (a 128 x 1 alpha strip, looked up by how grazing the view is)
// A glowing liquid (lava) adds its colour as light; the opacity is the material's.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings::{globals, view},
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct Liquid {
    // tiling (u, v) and scroll (u, v per second) of texture-0, then texture-1
    scale0: vec2<f32>,
    scale1: vec2<f32>,
    speed0: vec2<f32>,
    speed1: vec2<f32>,
    // water: its tint (color) and the reflection's colour (sky x color-1)
    tint: vec4<f32>,
    reflection: vec4<f32>,
    // 0: two layers, 1: water
    kind: u32,
    // texture-1's blend mode
    mode: u32,
    // how much of its colour it gives as light (lava 1), its opacity, and its colour's scale
    // (2: the console's modulate x 2, blend-mode 2)
    glow: f32,
    opacity: f32,
    bright: f32,
}

@group(2) @binding(100) var<uniform> liquid: Liquid;
@group(2) @binding(101) var layer0: texture_2d<f32>;
@group(2) @binding(102) var layer0_sampler: sampler;
@group(2) @binding(103) var layer1: texture_2d<f32>;
@group(2) @binding(104) var layer1_sampler: sampler;
@group(2) @binding(105) var ramp: texture_2d<f32>;
@group(2) @binding(106) var ramp_sampler: sampler;

const REFLECT_MIN: f32 = 0.15;
const REFLECT_MAX: f32 = 0.5;

// the water's rippled normal (set by `surface`)
var<private> ripple_n: vec3<f32>;
// how far the ripples tilt the surface, per unit of brightness slope (over RIPPLE_STEP texels:
// the texture is soft, 0.005 a texel), at most RIPPLE_MAX; and the second layer's share
const RIPPLE: f32 = 25.0;
const RIPPLE_STEP: f32 = 3.0;
const RIPPLE_MAX: f32 = 0.6;
const RIPPLE_SECOND: f32 = 0.6;
const RIPPLE_SHADE: f32 = 1.2;

// the water texture's brightness slope at `p` (per texel, u and v)
fn height_slope(p: vec2<f32>) -> vec2<f32> {
    let e = RIPPLE_STEP / vec2<f32>(textureDimensions(layer0));
    let lum = vec3<f32>(0.3, 0.59, 0.11);
    let h = dot(textureSample(layer0, layer0_sampler, p).rgb, lum);
    let hu = dot(textureSample(layer0, layer0_sampler, p + vec2<f32>(e.x, 0.0)).rgb, lum);
    let hv = dot(textureSample(layer0, layer0_sampler, p + vec2<f32>(0.0, e.y)).rgb, lum);
    return vec2<f32>(hu - h, hv - h);
}

fn surface(in: VertexOutput) -> vec4<f32> {
    let t = globals.time;
    let uv = in.uv;
    if liquid.kind == 1u {
        // two layers of the water's texture (scale-u, scale-v and scale-u-1, scale-v-1: repeats
        // per uv), each drifting at its own speed; their slopes (as heights: brightness) tilt
        // the surface, so the light, the sun's glint and the reflection ripple
        let a = uv * liquid.scale0 + liquid.speed0 * t;
        let b = uv * liquid.scale1 + liquid.speed1 * t + vec2<f32>(0.37, 0.71);
        let ca = textureSample(layer0, layer0_sampler, a);
        let cb = textureSample(layer0, layer0_sampler, b);
        var tilt = (height_slope(a) + height_slope(b) * RIPPLE_SECOND) * RIPPLE;
        if length(tilt) > RIPPLE_MAX {
            tilt = normalize(tilt) * RIPPLE_MAX;
        }
        let n = normalize(in.world_normal);
        ripple_n = normalize(n + vec3<f32>(-tilt.x, 0.0, -tilt.y));
        let v = normalize(view.world_position.xyz - in.world_position.xyz);
        let grazing = 1.0 - clamp(abs(dot(ripple_n, v)), 0.0, 1.0);
        // (looking down there's still some reflection: without it a pool seen from above was its
        // murky colour over a bed of much the same colour - it vanished)
        let f = mix(REFLECT_MIN, REFLECT_MAX, textureSample(ramp, ramp_sampler, vec2<f32>(grazing, 0.5)).a);
        let c = mix(ca.rgb, cb.rgb, 0.4);
        // the ripples shade the water's own colour too, lit from one side (they show under a dark
        // sky and a low sun, sdm_e10's pond, where the reflection and the glint barely change)
        let shade = 1.0 + dot(tilt, vec2<f32>(0.7, 0.7)) * RIPPLE_SHADE;
        let water = c * liquid.tint.rgb * shade;
        // (the game's reflection is a picture of the scene; the sky's colour here, lit and dark
        // by the ripples)
        let ripple = 0.55 + 0.9 * dot(c, vec3<f32>(0.3, 0.59, 0.11));
        return vec4<f32>(mix(water, liquid.reflection.rgb * ripple, f), mix(liquid.opacity, 1.0, f));
    }
    let c0 = textureSample(layer0, layer0_sampler, uv / max(liquid.scale0, vec2<f32>(1e-3)) + liquid.speed0 * t);
    let c1 = textureSample(layer1, layer1_sampler, uv / max(liquid.scale1, vec2<f32>(1e-3)) + liquid.speed1 * t);
    var c: vec3<f32>;
    if liquid.mode == 0u {
        c = mix(c0.rgb, c1.rgb, c1.a);
    } else if liquid.mode == 1u {
        c = c0.rgb + c1.rgb;
    } else {
        c = c0.rgb * c1.rgb * 2.0;
    }
    return vec4<f32>(c, liquid.opacity);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let raw = surface(in);
    let s = vec4<f32>(raw.rgb * liquid.bright, raw.a);
    pbr_input.material.base_color = vec4<f32>(s.rgb, s.a);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    if liquid.kind == 1u {
        pbr_input.N = ripple_n;
    }
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = vec4<f32>(out.color.rgb + s.rgb * liquid.glow, out.color.a);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}
