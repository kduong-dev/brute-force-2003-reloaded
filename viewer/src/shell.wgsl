// A power-up's shell (src/bin/play_powerups.rs, `ShellMaterial`; #15): the material class
// h_0112da6f (instance vtable 0x3a0900) that BRUTE and FORCE POWER's material controllers draw
// their character with. Its begin (0xa37e0) turns blending on, ONE / ONE (render states +0x154,
// +0xc8, +0xdc = 1); its draw (0xa3860) runs vertex program 0x52 (0x53 for the skinned format
// 0xe7fd942a) with pixel shader 0x44 over the character's mesh, with c27 = the material's
// `color` (and the draw's alpha) and c28 = (translate-u x clock, translate-v x clock, 0, offset):
//  - the vertex: its skinned position pushed out along its skinned normal by c28.w (the
//    material's `offset`, 0.03 m), then to the screen;
//  - the texture coordinate: its normal in view space, (n.xy + c17.w) x c18.w + c28.xy, c17.w 1
//    and c18.w 0.5 (the light pair's, FUN_000dcc10): a sphere map of the view's normal, scrolled
//    by the material's clock;
//  - the pixel: the colour x the texture (r0 = v0 x t0), added to the picture.
// The wrapper material over it (h_eb58ab52) sets the stencil to its h_e59d69a0 for every pixel
// drawn (0x9ea90): the glare's key (src/glare.rs), here the picture's alpha, replaced.
// The texture is bound as linear amounts (Rgba8Unorm): the console adds its bytes to the
// picture's, as the ALE's added particles do here (src/ale_fx.rs `texture`).

#import bevy_pbr::{
    mesh_functions,
    skinning,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
}

// rgb the material's `color` (c27.rgb), a the glare key (the wrapper's h_e59d69a0 / 255)
@group(2) @binding(0) var<uniform> color: vec4<f32>;
// x, y the scroll a second (translate-u, translate-v), z the material's clock (s), w the offset (m)
@group(2) @binding(1) var<uniform> params: vec4<f32>;
@group(2) @binding(2) var shell_texture: texture_2d<f32>;
@group(2) @binding(3) var shell_sampler: sampler;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef SKINNED
    let world_from_local = skinning::skin_model(vertex.joint_indices, vertex.joint_weights, vertex.instance_index);
    let n = skinning::skin_normals(world_from_local, vertex.normal);
#else
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let n = normalize(mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index));
#endif
    let p = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    // (program 0x52: R11.xyz += R10 x c28.w, the normal as skinned)
    out.world_position = vec4<f32>(p.xyz + n * params.w, 1.0);
    out.world_normal = n;
    out.position = position_world_to_clip(out.world_position.xyz);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // the normal in view space (x right, y up, as the console's c2 / c3 rows give them)
    let v = normalize((view.view_from_world * vec4<f32>(normalize(in.world_normal), 0.0)).xyz);
    let uv = (v.xy + vec2<f32>(1.0)) * 0.5 + params.xy * params.z;
    let t = textureSample(shell_texture, shell_sampler, uv).rgb;
    return vec4<f32>(color.rgb * t, color.a);
}
