// A level material (bevy's standard one) lit also by the level's point lights the way the game
// lights its scenery: each lamp's colour x (1 - distance / range) x N.L, nothing past its range,
// no shadows (see level_scene::LevelMaterial).

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
