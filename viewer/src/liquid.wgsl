// A level's liquid surface (level_scene::LiquidMaterial, #4), as the game's two liquid material
// classes draw theirs. The textures are sampled as stored (Rgba8Unorm) and the shaders' sums and
// products are taken on those values, as the console's combiners take the texels' bytes; the
// result is clamped as the combiners clamp it, handed to bevy as linear light (`to_linear`), and
// fogged (every one of the game's liquid shaders fogs its output). Nothing doubles a colour.
//   BF_LavaMaterial (h_f124a774: lava, sludge, toxic waste, chloride; FUN_00091bc0's pixel
//   shaders, todo/146-liquids leads §6.2): t0 texture-0, t1 texture-1, each at uv x 1 / its scale
//   + the clock x its translate (vertex program 0x42), unlit:
//     0x27 t0 + t1 (ONE / ONE)          0x28 lerp(t0, t1, t1.a), alpha t1.a (ONE / INVSRCALPHA)
//     0x29 t0 x t0.a + t1, alpha t0.a   0x2a t0 + t1 (opaque)
//     0x2b lerp(t0, t1 x v0, t1.a)      0x2c one layer as it is
//   with v0 the level's key light's colour (`color0`).
//   BF_OceanMaterialBase (h_0f8904b9: water; pixel shader 0x2f, vertex program 0x45):
//     (t0 x color + t1 x color-1) / 2 x v0, alpha the ramp at the vertex's own height in its
//     mesh; v0 the vertex light, the level's ambient and its two directional lights (`water_light`).
// A layer may be an animated texture: its frames are the layers of a 2D array, played in a loop
// on the material's clock (`frame`). Its alpha carries the glare's key where the blend lets it
// (GLARE_KEY: the wrapper's stencil value, #289, src/glare.wgsl).

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    mesh_view_bindings::{lights, view},
}
#import bevy_render::maths::PI

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::main_pass_post_lighting_processing,
}
#endif

// level_scene::LiquidParams
struct Liquid {
    // the game's pixel shader (0x27 - 0x2c, 0x2f; 0 the BF_LIQUID_DEBUG white), how its output
    // is blended (BLEND_*), 0x2c's layer, and the material's clock (s)
    shader: u32,
    blend: u32,
    layer: u32,
    clock: f32,
    // per layer: 1 / its scale (u, v), its translate (u, v)
    coords0: vec4<f32>,
    coords1: vec4<f32>,
    // per layer: its frame count, its frame rate, 1 if its translate is divided by its scale
    anim0: vec4<f32>,
    anim1: vec4<f32>,
    // the water's color and color-1; BF_LavaMaterial's v0 in color0
    color0: vec4<f32>,
    color1: vec4<f32>,
    // the glare's key (level_scene::glare_key): the wrapper's stencil value / 255, 1 for none
    key: f32,
}

@group(2) @binding(100) var<uniform> liquid: Liquid;
@group(2) @binding(101) var layer0: texture_2d_array<f32>;
@group(2) @binding(102) var layer0_sampler: sampler;
@group(2) @binding(103) var layer1: texture_2d_array<f32>;
@group(2) @binding(104) var layer1_sampler: sampler;
@group(2) @binding(105) var ramp: texture_2d<f32>;
@group(2) @binding(106) var ramp_sampler: sampler;

// level_scene's BLEND_*: none (opaque), SRCALPHA / INVSRCALPHA, ONE / INVSRCALPHA, ONE / ONE
const BLEND_NONE: u32 = 0u;
const BLEND_ADD: u32 = 3u;

// A layer's texture coordinates: uv x 1 / its scale + its scroll, the clock x its translate,
// divided by its scale for the water (vertex program 0x45's frac(clock x translate / scale),
// 0x92f70's c26 / c27) and not for lava (0x42's c25 / c26: repeats per second). The fraction
// keeps the precision: the textures repeat.
fn coords(uv: vec2<f32>, c: vec4<f32>, a: vec4<f32>) -> vec2<f32> {
    var scroll = liquid.clock * c.zw;
    if a.z > 0.5 {
        scroll = scroll * c.xy;
    }
    return uv * c.xy + fract(scroll);
}

// A layer's frame: its animator loops (mode 1, 0x2623f0) over frames / rate seconds and shows
// frame floor(rate x t) (0x262720); a plain texture (one frame) or a rate of 0 shows its first.
fn frame(a: vec4<f32>) -> i32 {
    if a.x < 1.5 || a.y <= 0.0 {
        return 0;
    }
    let length = a.x / a.y;
    let t = liquid.clock - floor(liquid.clock / length) * length;
    return i32(clamp(floor(a.y * t), 0.0, a.x - 1.0));
}

// sRGB's curve: the texels' stored values to linear light, and back
fn to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}
fn to_gamma(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

// The water's vertex light, vertex program 0x45's v0: the level's object ambient (slot 14) + max(N.L,
// 0) x colour for each of its two directional lights (slots 16-19: the light-objects flagged
// h_1f416687, FUN_000dcc10; answer-spec Q7), clamped as a vertex colour is. They're bevy's
// ambient and directional lights (level_scene::spawn_lighting: the outdoor pair and the object
// ambient, given as sRGB at an illuminance of PI), taken back to the values the game hands the
// shader; per pixel here, per vertex there (a flat surface: the same).
fn water_light(n: vec3<f32>) -> vec3<f32> {
    var v = to_gamma(max(lights.ambient_color.rgb * view.exposure, vec3<f32>(0.0)));
    for (var i = 0u; i < lights.n_directional_lights; i++) {
        let l = lights.directional_lights[i];
        let colour = to_gamma(max(l.color.rgb * view.exposure / PI, vec3<f32>(0.0)));
        v += max(dot(n, l.direction_to_light), 0.0) * colour;
    }
    return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}

// The surface's colour in the console's terms (stored values) and its alpha.
fn surface(in: VertexOutput) -> vec4<f32> {
    let t0 = textureSample(layer0, layer0_sampler, coords(in.uv, liquid.coords0, liquid.anim0), frame(liquid.anim0));
    let t1 = textureSample(layer1, layer1_sampler, coords(in.uv, liquid.coords1, liquid.anim1), frame(liquid.anim1));
    // the water's ramp, at the vertex's own height in its mesh (stage 2's u; the second uv,
    // level_scene::water_mesh)
    var own = 1.0;
#ifdef VERTEX_UVS_B
    own = in.uv_b.x;
#endif
    let ramp_alpha = textureSample(ramp, ramp_sampler, vec2<f32>(own, 0.5)).a;
    switch liquid.shader {
        case 0x27u: {
            return vec4<f32>(t0.rgb + t1.rgb, 1.0);
        }
        case 0x28u: {
            return vec4<f32>(mix(t0.rgb, t1.rgb, t1.a), t1.a);
        }
        case 0x29u: {
            return vec4<f32>(t0.rgb * t0.a + t1.rgb, t0.a);
        }
        case 0x2au: {
            return vec4<f32>(t0.rgb + t1.rgb, 1.0);
        }
        case 0x2bu: {
            return vec4<f32>(mix(t0.rgb, t1.rgb * liquid.color0.rgb, t1.a), 1.0);
        }
        case 0x2cu: {
            return select(t0, t1, liquid.layer == 1u);
        }
        case 0x2fu: {
            let colour = (t0.rgb * liquid.color0.rgb + t1.rgb * liquid.color1.rgb) * 0.5;
            return vec4<f32>(colour * water_light(normalize(in.world_normal)), ramp_alpha);
        }
        default: {
            return vec4<f32>(1.0);
        }
    }
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let s = clamp(surface(in), vec4<f32>(0.0), vec4<f32>(1.0));
    // (what bevy's blend states take: opaque, alpha 1; ONE / ONE, the whole colour, which bevy's
    // Add takes as the colour x alpha)
    var alpha = s.a;
    if liquid.blend == BLEND_NONE || liquid.blend == BLEND_ADD {
        alpha = 1.0;
    }
    let colour = vec4<f32>(to_linear(s.rgb), alpha);
    pbr_input.material.base_color = colour;
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr_input, colour);
#ifdef GLARE_KEY
    // the stencil value its pixels write, which the glare reads from the picture's alpha
    // (src/glare.wgsl; opaque or added: LiquidSurface::specialize)
    out.color.a = liquid.key;
#endif
#endif
    return out;
}
