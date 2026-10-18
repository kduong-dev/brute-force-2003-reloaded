// A level material (bevy's standard one) lit also by the level's point lights the way the game
// lights its scenery: each lamp's colour x (1 - distance / range) x N.L, nothing past its range,
// no shadows (see level_scene::LevelMaterial). A terrain layer (TERRAIN_DETAIL) samples its
// colour and detail textures here, as the game's terrain pass does (level_scene::terrain_detail).

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
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    mesh_view_bindings::view,
}
#endif

struct Lamp {
    // world position, and the distance the light reaches
    at: vec3<f32>,
    range: f32,
    // linear colour (its strength), and the falloff kind (unused: every kind is linear here)
    color: vec3<f32>,
    falloff: f32,
}

@group(2) @binding(100) var<storage, read> lamps: array<Lamp>;
// a terrain layer's detail: the game's c20 = (1 / s^2, s^2, the detail's scale, its share at the
// eye), s the distance it fades out at (FUN_00090f50); zero for the other materials
@group(2) @binding(101) var<uniform> detail: vec4<f32>;
// a terrain layer's colour texture and its detail texture (the colour's where it names none)
@group(2) @binding(102) var colour_texture: texture_2d<f32>;
@group(2) @binding(103) var colour_sampler: sampler;
@group(2) @binding(104) var detail_texture: texture_2d<f32>;
@group(2) @binding(105) var detail_sampler: sampler;

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
    let to_eye = in.world_position.xyz - view.world_position;
    let d2 = dot(to_eye, to_eye);
    let share = select(0.0, detail.w * (1.0 - d2 * detail.x), d2 <= detail.y);
    let near = textureSample(detail_texture, detail_sampler, in.uv * detail.z);
    let far = textureSample(colour_texture, colour_sampler, in.uv);
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
        var light = vec3<f32>(0.0);
        let p = in.world_position.xyz;
        for (var i = 0u; i < arrayLength(&lamps); i++) {
            let l = lamps[i];
            let to = l.at - p;
            let d = length(to);
            if d < l.range && d > 1e-4 {
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
