// A level material (bevy's standard one) lit also by the level's point lights the way the game
// lights its scenery: each lamp's colour x (1 - distance / range) x N.L, nothing past its range,
// no shadows (see level_scene::LevelMaterial). It takes the lamps of its light pair only, and an
// indoor material (INDOOR_PAIR) the level's indoor pair in place of bevy's directional lights, which
// are the outdoor pair (level_scene::spawn_lighting, `indoor_lit`). A terrain layer (TERRAIN_DETAIL)
// samples its colour and detail textures here, as the game's terrain pass does
// (level_scene::terrain_detail).

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, calculate_diffuse_color, calculate_F0},
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_UNLIT_BIT},
    lighting,
    shadows,
    mesh_view_bindings as view_bindings,
    mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
}
#import bevy_render::maths::PI
#endif

struct Lamp {
    // world position, and the distance the light reaches
    at: vec3<f32>,
    range: f32,
    // linear colour (its strength), and the falloff kind (unused: every kind is linear here)
    color: vec3<f32>,
    falloff: f32,
    // its pairs, as the game's light keeps them (its +0x94): 2 the indoor pair's, 4 the outdoor's
    pairs: u32,
}

// The level's pairs (level_scene::light_buffer): the way toward each of the indoor pair's two
// lights (w 1 when the slot holds one) and its linear colour, and the way toward each light of the
// outdoor pair, which are bevy's directional lights (level_scene::spawn_lighting)
struct Pairs {
    indoor_dir: array<vec4<f32>, 2>,
    indoor_color: array<vec4<f32>, 2>,
    outdoor_dir: array<vec4<f32>, 2>,
}

struct LevelLights {
    pairs: Pairs,
    lamps: array<Lamp>,
}

@group(2) @binding(100) var<storage, read> level_lights: LevelLights;
// a terrain layer's detail: the game's c20 = (1 / s^2, s^2, the detail's scale, its share at the
// eye), s the distance it fades out at (FUN_00090f50); zero for the other materials
@group(2) @binding(101) var<uniform> detail: vec4<f32>;
// a terrain layer's colour texture and its detail texture (the colour's where it names none)
@group(2) @binding(102) var colour_texture: texture_2d<f32>;
@group(2) @binding(103) var colour_sampler: sampler;
@group(2) @binding(104) var detail_texture: texture_2d<f32>;
@group(2) @binding(105) var detail_sampler: sampler;

#ifndef PREPASS_PIPELINE
#ifdef INDOOR_PAIR
// Whether one of bevy's directional lights, by the way toward it, is a light of the level's outdoor
// pair (and not, say, the Vengar view's vision pair, play_vengar.rs `vision_lights`).
fn outdoor_light(to: vec3<f32>) -> bool {
    for (var k = 0u; k < 2u; k++) {
        let o = level_lights.pairs.outdoor_dir[k];
        if o.w != 0.0 && dot(to, normalize(o.xyz)) > 0.9999 {
            return true;
        }
    }
    return false;
}

// What an indoor surface's light differs by from what apply_pbr_lighting gave it: bevy's
// directional lights that are the outdoor pair (with the shadow its brightest casts) taken out,
// exactly as apply_pbr_lighting put them in, and the indoor pair put in by the same diffuse and
// specular, at an illuminance of PI (a light of 1.0 shows the texture as it is). The game has a
// surface take one pair or the other (FUN_000dcc10 by the light-usage the wrapper material's draw
// 0x9f3c0 reads), not both. While the outdoor pair is put out (the Vengar view, whose vision pair
// takes both pairs' place: FUN_000dcc10's second argument) nothing is swapped; a level with no
// outdoor pair (mp2) takes the indoor pair always.
fn indoor_swap(in: PbrInput) -> vec3<f32> {
    // (the lighting input apply_pbr_lighting builds, for the base layer)
    let NdotV = max(dot(in.N, in.V), 0.0001);
    var input: lighting::LightingInput;
    input.layers[lighting::LAYER_BASE].NdotV = NdotV;
    input.layers[lighting::LAYER_BASE].N = in.N;
    input.layers[lighting::LAYER_BASE].R = reflect(-in.V, in.N);
    input.layers[lighting::LAYER_BASE].perceptual_roughness = in.material.perceptual_roughness;
    input.layers[lighting::LAYER_BASE].roughness = lighting::perceptualRoughnessToRoughness(in.material.perceptual_roughness);
    input.P = in.world_position.xyz;
    input.V = in.V;
    input.diffuse_color = calculate_diffuse_color(in.material.base_color.rgb, in.material.metallic,
                                                  in.material.specular_transmission, in.material.diffuse_transmission);
    input.F0_ = calculate_F0(in.material.base_color.rgb, in.material.metallic, in.material.reflectance);
    input.F_ab = lighting::F_AB(in.material.perceptual_roughness, NdotV);
    let view_z = dot(vec4<f32>(
        view_bindings::view.view_from_world[0].z,
        view_bindings::view.view_from_world[1].z,
        view_bindings::view.view_from_world[2].z,
        view_bindings::view.view_from_world[3].z
    ), in.world_position);
    var light = vec3<f32>(0.0);
    var lit = level_lights.pairs.outdoor_dir[0].w == 0.0 && level_lights.pairs.outdoor_dir[1].w == 0.0;
    for (var i = 0u; i < view_bindings::lights.n_directional_lights; i++) {
        let l = &view_bindings::lights.directional_lights[i];
        if !outdoor_light((*l).direction_to_light) {
            continue;
        }
        lit = lit || max((*l).color.r, max((*l).color.g, (*l).color.b)) > 0.0;
        var shadow = 1.0;
        if ((in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u
                && ((*l).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = shadows::fetch_directional_shadow(i, in.world_position, in.world_normal, view_z);
        }
        light -= lighting::directional_light(i, &input, true) * shadow;
    }
    for (var k = 0u; k < 2u; k++) {
        if !lit || level_lights.pairs.indoor_dir[k].w == 0.0 {
            continue;
        }
        var derived = lighting::derive_lighting_input(in.N, in.V, normalize(level_lights.pairs.indoor_dir[k].xyz));
        let diffuse = input.diffuse_color * lighting::Fd_Burley(&input, &derived);
        let specular = lighting::specular(&input, &derived, 1.0);
        light += (diffuse + specular) * derived.NdotL * level_lights.pairs.indoor_color[k].rgb * PI;
    }
    return light * view_bindings::view.exposure;
}
#endif
#endif

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
#ifdef TERRAIN_DETAIL
#ifdef VERTEX_UVS_A
#ifndef PREPASS_PIPELINE
    // the terrain's base pass (vertex program 0x38, pixel shaders 0x1a / 0x1b): the colour at the
    // vertex uv and the detail at that uv x its scale, mixed by the detail's share, which falls
    // from the eye as 1 - d^2 / s^2 and is 0 past s; the mix is lit as the colour was (the
    // material's own colour is the vertex colour, its alpha the layer's mask). The console works
    // out the share per vertex; here it's per pixel.
    let to_eye = in.world_position.xyz - view_bindings::view.world_position;
    let d2 = dot(to_eye, to_eye);
    let share = select(0.0, detail.w * (1.0 - d2 * detail.x), d2 <= detail.y);
    let near = textureSample(detail_texture, detail_sampler, in.uv * detail.z);
    let far = textureSample(colour_texture, colour_sampler, in.uv);
    // (the console's 8-bit combiner mixes the two in gamma terms, as stored; this mixes their
    // linear values: a small difference between two samples of one texture)
    let texel = mix(far.rgb, near.rgb, share);
    pbr_input.material.base_color = vec4<f32>(pbr_input.material.base_color.rgb * texel, pbr_input.material.base_color.a);
#endif
#endif
#endif
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
#ifdef INDOOR_PAIR
        let pair = 2u;
        out.color = vec4<f32>(max(out.color.rgb + indoor_swap(pbr_input), vec3<f32>(0.0)), out.color.a);
#else
        let pair = 4u;
#endif
        var light = vec3<f32>(0.0);
        let p = in.world_position.xyz;
        for (var i = 0u; i < arrayLength(&level_lights.lamps); i++) {
            let l = level_lights.lamps[i];
            let to = l.at - p;
            let d = length(to);
            if (l.pairs & pair) != 0u && d < l.range && d > 1e-4 {
                let n_dot_l = max(dot(pbr_input.N, to / d), 0.0);
                light += l.color * ((1.0 - d / l.range) * n_dot_l);
            }
        }
        out.color = vec4<f32>(out.color.rgb + pbr_input.material.base_color.rgb * light, out.color.a);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}
