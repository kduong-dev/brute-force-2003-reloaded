// A level material (bevy's standard one) lit also by the level's point lights the way the game
// lights its scenery: each lamp's colour x (1 - distance / range) x N.L, nothing past its range,
// no shadows (see level_scene::LevelMaterial). It takes the lamps of its light pair only, and an
// indoor material (INDOOR_PAIR) the level's indoor pair in place of bevy's directional lights, which
// are the outdoor pair (level_scene::spawn_lighting, `indoor_lit`).

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

// The level's indoor pair (level_scene::light_buffer): the way toward each of its two lights (w 1
// when the slot holds one) and its linear colour; `swap` 1 while indoor materials take it (0 in the
// Vengar view, whose vision pair lights both pairs' objects)
struct IndoorPair {
    dir: array<vec4<f32>, 2>,
    color: array<vec4<f32>, 2>,
    swap: u32,
}

struct LevelLights {
    indoor: IndoorPair,
    lamps: array<Lamp>,
}

@group(2) @binding(100) var<storage, read> level_lights: LevelLights;

#ifndef PREPASS_PIPELINE
#ifdef INDOOR_PAIR
// What an indoor surface's light differs by from what apply_pbr_lighting gave it: bevy's
// directional lights (the outdoor pair, with the shadow its brightest casts) taken out, exactly as
// apply_pbr_lighting put them in, and the indoor pair put in by the same diffuse and specular, at
// an illuminance of PI (a light of 1.0 shows the texture as it is). The game has a surface take
// one pair or the other (FUN_000dcc10 by the light-usage the wrapper material's draw 0x9f3c0
// reads), not both.
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
    for (var i = 0u; i < view_bindings::lights.n_directional_lights; i++) {
        var shadow = 1.0;
        if ((in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u
                && (view_bindings::lights.directional_lights[i].flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = shadows::fetch_directional_shadow(i, in.world_position, in.world_normal, view_z);
        }
        light -= lighting::directional_light(i, &input, true) * shadow;
    }
    for (var k = 0u; k < 2u; k++) {
        if level_lights.indoor.dir[k].w == 0.0 {
            continue;
        }
        var derived = lighting::derive_lighting_input(in.N, in.V, normalize(level_lights.indoor.dir[k].xyz));
        let diffuse = input.diffuse_color * lighting::Fd_Burley(&input, &derived);
        let specular = lighting::specular(&input, &derived, 1.0);
        light += (diffuse + specular) * derived.NdotL * level_lights.indoor.color[k].rgb * PI;
    }
    return light * view_bindings::view.exposure;
}
#endif
#endif

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
#ifdef INDOOR_PAIR
        let pair = 2u;
        if level_lights.indoor.swap != 0u {
            out.color = vec4<f32>(max(out.color.rgb + indoor_swap(pbr_input), vec3<f32>(0.0)), out.color.a);
        }
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
