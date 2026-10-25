//! The console's glare (#289): the bloom the game lays over the picture after the world is
//! drawn, keyed per surface by the stencil value its material writes. Shared by the demo and the
//! level viewer: every camera with the console's look (`level_scene::console_look`) runs it.
//! The code read (viewer/docs/code-map.md, "Effects / ALE"; the page is viewer/docs/levels.md,
//! "The glare"):
//!
//!  - **The keys.** Each frame the world pass opens a stencil state that every pixel drawn
//!    replaces with its reference (FUN_000d4180: STENCILENABLE 1, STENCILPASS REPLACE, STENCILREF
//!    255), over a stencil cleared to 255 (0xd9ea4: D3DDevice_Clear with 0xff), and closes it
//!    after the sorted list (0xe8717). A wrapper material (h_eb58ab52) sets the reference to its
//!    h_e59d69a0 for what it wraps on every draw (0x9ea90 -> FUN_0009e960), and flags the frame
//!    for the glare when it isn't 255: lava 60 (sdm_e13) or 95 (the tutorial, e40), sludge 200,
//!    toxic waste 1, water 255. (Decals do the same with their type's byte +0x22, FUN_00213fc0,
//!    and so does the Vengar view's heat figure material, 0xa8c80.)
//!  - **The bright pass** (FUN_000e5f60's first draw): the back buffer (as an A8R8G8B8 texture,
//!    t0) and the depth-stencil buffer read as an R8G8B8A8 texture (format byte 0x41, t1: its
//!    alpha the stencil byte of D24S8), into the 320 x 240 target h_0796c3b0 (FUN_000e49d0).
//!    Pixel shader 0x1f keeps the picture where its luminance (c0 = 0.299, 0.587, 0.114) is above
//!    t1.a (the mux on t1.a + 0.5 - lum), x t0.a: so the stencil value is a luminance threshold,
//!    255 never passing.
//!  - **The blur** (its second draw, pixel shader 0x20: t0 x v0): 24 copies of that target added
//!    ONE / ONE into the 160 x 120 target h_0575b9b9, each shifted by k x 0.006 of the screen
//!    (k = 1, 2, 3; 0x40cfc8) in eight directions 45 degrees apart, weighted by its vertex colour,
//!    trunc(0.1 x 255) = 25 (FUN_000e47b0: 0x40cfc4 x 0x3a50e8 through FUN_002ebf10, ftol). In
//!    all 2.35 times the kept picture, saturating as the 8-bit target does.
//!  - **The composite** (FUN_000e5d40, pixel shader 0x3a: t0 as it is): that target stretched
//!    over the picture, added ONE / ONE (the byte 0x3c06e5 is 1 and nothing writes it), z off.
//!  - It runs after the decals, before the refraction's copy (FUN_000e81e0 0xe8755), in vision
//!    mode 0 and in the Vengar view (mode 2, after its world pass and the heat figures; 0xe8917),
//!    on every level, whenever a keyed surface was drawn (+0xb44).
//!
//! Here the key travels in the picture's alpha: a keyed material writes its stencil value / 255
//! there (`level_scene::LiquidMaterial`, src/liquid.wgsl `GLARE_KEY`), everything else 1 (bevy's
//! opaque materials write 1, its blends keep 1 over them; a blend that covers the view must keep
//! the alpha as it is: play_grenade.rs's tint quad, `KeepAlpha`). The bright pass reads the
//! threshold from it; the game's other factor, the frame buffer's alpha t0.a, is taken as 1 (the
//! lava's and the sludge's pixel shaders 0x2b and 0x2a write alpha 1). The three passes work on
//! the picture's sRGB values, as the console's combiners and blending take the frame buffer's
//! bytes (src/glare.wgsl), and the composite writes the picture's alpha back to 1.
//!
//! Only the liquids write keys so far. The game's other keyed surfaces (the wrappers' values on
//! glow shells, lit glow maps, the glow shader's lamps, the memory chip; decal types; the heat
//! figures) multiply by their own frame buffer alpha, which isn't worked out for them yet.
//!
//! Test hooks: `BF_GLARE=0` adds nothing (the passes run with no weight), to compare;
//! `BF_GLARE_DEBUG=1` shows the keys in place of the picture (grey: a key; white: none), `=2`
//! the blur alone.

use bevy::{
    asset::weak_handle,
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    image::BevyDefault,
    prelude::*,
    render::{
        camera::{CameraUpdateSystem, ExtractedCamera},
        extract_component::{ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer}, AddressMode, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FilterMode, FragmentState,
            MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, TextureDescriptor,
            TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
        },
        renderer::{RenderContext, RenderDevice},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
        Render, RenderApp, RenderSet,
    },
};

pub use settings::Glare;

/// The passes' shader (src/glare.wgsl).
const GLARE_SHADER: Handle<Shader> = weak_handle!("9d4e2b71-3c8a-4f6e-a5d0-6b1f8e2c7a93");

/// The bright pass's target, in rows: h_0796c3b0 is 320 x 240 (textures-common.xml, format 18
/// LIN_A8R8G8B8) for the game's 640 x 480 picture, half its rows. Its width here follows the
/// view's shape (`targets`), so a texel covers what it covers on the console.
const BRIGHT_ROWS: u32 = 240;
/// The blur's target, in rows: h_0575b9b9, 160 x 120 (the same file), a quarter of the picture's.
const GLOW_ROWS: u32 = 120;
/// The blur's step, a share of the screen (0x40cfc8, FUN_000e47b0): its copies sit 1, 2 and 3
/// steps out.
const STEP: f32 = 0.006;
/// The game's picture's width over its height: the step is a share of its 4:3 screen, so in a
/// wider view it keeps its size against the view's height (as vengar.wgsl's halo does).
const GAME_ASPECT: f32 = 4.0 / 3.0;
/// Each copy's weight: its vertex colour's byte, trunc(0.1 x 255) (0x40cfc4 x 0x3a50e8 through
/// FUN_002ebf10, MSVC's ftol, which truncates), over 255.
const TAP_WEIGHT: f32 = 25.0 / 255.0;

/// The camera's settings for the passes (src/glare.wgsl `Glare`). (In a module of its own: the
/// derive's own checks read as unused.)
mod settings {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::{extract_component::ExtractComponent, render_resource::ShaderType}};

    /// A camera that runs the glare (`plugin`), and what its passes take this frame.
    #[derive(Component, Clone, Copy, Default, Debug, PartialEq, ExtractComponent, ShaderType)]
    pub struct Glare {
        /// x, y: a copy's step in the picture's uv (STEP over the game's 4:3 width and its
        /// height); z, w: the bright pass's target in texels (`targets`)
        pub step: Vec4,
        /// x: each copy's weight (TAP_WEIGHT; 0 under BF_GLARE=0); y: BF_GLARE_DEBUG's view (0
        /// the picture, 1 the keys, 2 the blur alone)
        pub weight: Vec4,
    }
}

/// The two targets for a view of `size` pixels: the bright pass's (BRIGHT_ROWS rows) and the
/// blur's (GLOW_ROWS), each as wide as the view's shape makes it.
fn targets(size: UVec2) -> (UVec2, UVec2) {
    let aspect = size.x as f32 / size.y.max(1) as f32;
    let of = |rows: u32| UVec2::new(((rows as f32 * aspect).round() as u32).max(1), rows);
    (of(BRIGHT_ROWS), of(GLOW_ROWS))
}

/// Whether BF_GLARE=0 turned the glare off (the test hook: the passes still run, adding nothing).
fn off() -> bool {
    std::env::var("BF_GLARE").is_ok_and(|v| v.trim() == "0")
}

/// BF_GLARE_DEBUG=1 shows the keys in place of the picture (grey: the threshold a surface wrote;
/// white: none), =2 the blur alone (what the composite adds); 0 the picture.
fn debug_view() -> f32 {
    std::env::var("BF_GLARE_DEBUG").ok().and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(0.0)
}

/// The glare: its settings each frame on the cameras that have it, and its passes in the render
/// graph, after the tone mapping (none here) and the Vengar view's pass (play_vengar.rs orders
/// itself before it), before the scope's views and the HUD.
pub fn plugin(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    shaders.insert(&GLARE_SHADER, Shader::from_wgsl(include_str!("glare.wgsl"), file!()));
    app.add_plugins((ExtractComponentPlugin::<Glare>::default(), UniformComponentPlugin::<Glare>::default(), PassPlugin))
        .add_systems(PostUpdate, follow_view.after(CameraUpdateSystem));
}

/// Each glare camera's settings from its view's size this frame.
fn follow_view(mut cameras: Query<(&Camera, &mut Glare)>) {
    let weight = if off() { 0.0 } else { TAP_WEIGHT };
    for (camera, mut glare) in &mut cameras {
        let Some(size) = camera.physical_viewport_size().filter(|s| s.x > 0 && s.y > 0) else { continue };
        let (bright, _) = targets(size);
        let aspect = size.x as f32 / size.y as f32;
        let now = Glare {
            step: Vec4::new(STEP * GAME_ASPECT / aspect, STEP, bright.x as f32, bright.y as f32),
            weight: Vec4::new(weight, debug_view(), 0.0, 0.0),
        };
        if *glare != now {
            *glare = now;
        }
    }
}

/// The passes in the render graph.
struct PassPlugin;

impl Plugin for PassPlugin {
    fn build(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.add_systems(Render, prepare_targets.in_set(RenderSet::PrepareResources))
            .add_render_graph_node::<ViewNodeRunner<GlareNode>>(Core3d, GlareLabel)
            .add_render_graph_edges(Core3d, (Node3d::Tonemapping, GlareLabel, Node3d::EndMainPassPostProcessing));
    }

    fn finish(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.init_resource::<GlarePipelines>();
    }
}

/// The glare's node label (the Vengar view's pass comes before it, the scope's views after it).
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct GlareLabel;

/// A view's two targets this frame (`targets`): the kept picture and its blur, 8 bits a channel
/// as the console's (LIN_A8R8G8B8), holding the picture's sRGB values.
#[derive(Component)]
struct GlareTargets {
    bright: CachedTexture,
    glow: CachedTexture,
}

/// The targets of every view with the glare, from the texture cache (the same ones each frame
/// while the view keeps its size).
fn prepare_targets(mut commands: Commands, mut cache: ResMut<TextureCache>, device: Res<RenderDevice>,
                   views: Query<(Entity, &ExtractedCamera), With<Glare>>) {
    for (entity, camera) in &views {
        let Some(size) = camera.physical_viewport_size.filter(|s| s.x > 0 && s.y > 0) else { continue };
        let (bright, glow) = targets(size);
        let mut texture = |label: &'static str, s: UVec2| cache.get(&device, TextureDescriptor {
            label: Some(label),
            size: Extent3d { width: s.x, height: s.y, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let targets = GlareTargets { bright: texture("glare_bright", bright), glow: texture("glare_glow", glow) };
        commands.entity(entity).insert(targets);
    }
}

/// The three passes (src/glare.wgsl): the bright pass from the picture into the first target,
/// the blur from it into the second, and the picture with the blur added into the view's next
/// picture.
#[derive(Default)]
struct GlareNode;

impl ViewNode for GlareNode {
    type ViewQuery = (&'static ViewTarget, &'static GlareTargets, &'static Glare, &'static DynamicUniformIndex<Glare>);

    fn run(&self, _graph: &mut RenderGraphContext, context: &mut RenderContext, (target, targets, glare, index): QueryItem<Self::ViewQuery>,
           world: &World) -> Result<(), NodeRunError> {
        // (no settings yet: the view's size wasn't known when they were made)
        if glare.step.z < 1.0 || glare.step.w < 1.0 {
            return Ok(());
        }
        let pipes = world.resource::<GlarePipelines>();
        let cache = world.resource::<PipelineCache>();
        let (Some(bright), Some(blur), Some(composite)) =
            (cache.get_render_pipeline(pipes.bright), cache.get_render_pipeline(pipes.blur), cache.get_render_pipeline(pipes.composite)) else {
            return Ok(());
        };
        let Some(uniforms) = world.resource::<ComponentUniforms<Glare>>().uniforms().binding() else { return Ok(()) };
        let post = target.post_process_write();
        let device = context.render_device().clone();
        let group = |first: &bevy::render::render_resource::TextureView, second: &bevy::render::render_resource::TextureView| {
            device.create_bind_group("glare_pass", &pipes.layout, &BindGroupEntries::sequential((first, second, &pipes.sampler, uniforms.clone())))
        };
        let (kept, glow) = (&targets.bright.default_view, &targets.glow.default_view);
        let passes = [
            ("glare_bright", bright, group(post.source, post.source), kept),
            ("glare_blur", blur, group(kept, kept), glow),
            ("glare_composite", composite, group(post.source, glow), post.destination),
        ];
        for (label, pipeline, bind_group, view) in passes {
            let mut draw = context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(RenderPassColorAttachment { view, resolve_target: None, ops: Operations::default() })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw.set_render_pipeline(pipeline);
            draw.set_bind_group(0, &bind_group, &[index.index()]);
            draw.draw(0..3, 0..1);
        }
        Ok(())
    }
}

/// The passes' pipelines: one layout (two textures, a linear clamped sampler, the settings), the
/// bright pass and the blur into the 8-bit targets, the composite into the picture.
#[derive(Resource)]
struct GlarePipelines {
    layout: BindGroupLayout,
    sampler: Sampler,
    bright: CachedRenderPipelineId,
    blur: CachedRenderPipelineId,
    composite: CachedRenderPipelineId,
}

impl FromWorld for GlarePipelines {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>().clone();
        let layout = device.create_bind_group_layout("glare_layout", &BindGroupLayoutEntries::sequential(ShaderStages::FRAGMENT, (
            texture_2d(TextureSampleType::Float { filterable: true }),
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
            uniform_buffer::<Glare>(true),
        )));
        // (the passes' stages: CLAMP, FUN_000e5f60 / FUN_000e5d40 at +0x734 / +0x74c; the blur's
        // and the composite's LINEAR, +0x77c / +0x794)
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });
        let pipelines = world.resource::<PipelineCache>();
        let pipeline = |entry: &'static str, format: TextureFormat| pipelines.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(format!("glare_{entry}").into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: GLARE_SHADER,
                shader_defs: vec![],
                entry_point: entry.into(),
                targets: vec![Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });
        let bright = pipeline("bright", TextureFormat::Rgba8Unorm);
        let blur = pipeline("blur", TextureFormat::Rgba8Unorm);
        let composite = pipeline("composite", TextureFormat::bevy_default());
        GlarePipelines { layout, sampler, bright, blur, composite }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The targets keep the console's rows and follow the view's shape: its own 4:3 picture
    /// gives the game's 320 x 240 and 160 x 120.
    #[test]
    fn targets_follow_the_view() {
        assert_eq!(targets(UVec2::new(640, 480)), (UVec2::new(320, 240), UVec2::new(160, 120)));
        assert_eq!(targets(UVec2::new(1280, 720)), (UVec2::new(427, 240), UVec2::new(213, 120)));
    }

    /// Each copy's weight is the vertex colour's byte the game computes: trunc(0.1 x 255) = 25,
    /// the 24 copies 2.35 times the kept picture.
    #[test]
    fn copy_weight() {
        assert_eq!((0.1f32 * 255.0) as u8, 25);
        assert!((TAP_WEIGHT * 24.0 - 2.353).abs() < 1e-3);
    }
}
