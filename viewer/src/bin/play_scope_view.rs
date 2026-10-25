//! The scope's views over the picture (#283): the scope frame round everyone's but Flint's, and
//! Flint's own overlay. Both are HUD widgets the crosshair widget's update makes the first time it
//! runs (0xc6030): the frame (`h_1018edf6`, FUN_000b4010) always, Flint's overlay (`h_fc21e242`,
//! FUN_000b3520) when the controlled character's skill's +0xa8 has bit 0 (hers, 0x2d5e40;
//! `skills::scope_frame`). Neither is in the HUD's layout data. Sources: the leads
//! (todo/283-scope-hud/leads.md §2-3), the takes in todo/60-kman-auto, 63-mk-aslt, 126-follow-camera
//! and 62-l-shot-75, and the functions cited (viewer/docs/code-map.md, "HUD"); the page is
//! viewer/docs/squad.md ("Two kinds of scope view").
//!
//!  - **The frame** (its update FUN_000b4070, its draw 0xb4240): scoped (char+0x7a8, the scope's
//!    level) with the skill's bit 0 clear, its scale is 1 at once; otherwise it eases to 3 at 6 a
//!    second, its alpha 1 - clamp(scale - 2, 0, 1): leaving the scope it grows x3 about the
//!    picture's middle in 0.33 s, fading over the last 0.17 s. Drawn as four quads from the
//!    middle, the quarter h_1807ec04 1:1 (256 x 128 units) in each, mirrored, its edge texels
//!    carried on to the screen's edges (CLAMP addressing, FUN_0006a610), blended ZERO /
//!    ONE_MINUS_SRC_COLOR (the sprite draw 0xba950's mode 5): the picture times 1 - the texture,
//!    40% outside the rim, 20% on its line, untouched in the opening (x 164..476, y 127..353 of
//!    640 x 480; #60 take07, #63 take01, #54 take01).
//!  - **Flint's overlay** (its update 0xb3700, its draw FUN_000b3970): scoped, its scale eases to
//!    2 (OVERLAY_IN), otherwise to 3, at the frame's 6 a second and with its fade: it comes in over
//!    0.17 s shrinking from 3x to 2x, and goes the same way. Drawn as two textures in four
//!    mirrored quads each, added (mode 2, ONE / ONE), from the middle out: the dotted ring
//!    h_f0d8a0a7, 160 x 120 units x the ring scale, then the corner smear h_f5b6a9f8, 160 x 120 x
//!    the smear scale, both x the widget's scale, their texels times grey 30 and 50: the smear's
//!    streaks round the picture's corners, the ring an ellipse of teal and yellow-green dots
//!    near the edges (#126 take16). The ring's scale runs 1.5 to 1.2 and the smear's 1.1 to 1.0
//!    with the scope's focus (FUN_000702a0's cam+0x1f8: the centre ray's depth) as a share of
//!    the follow camera's clip range. Not drawn in a vision mode (the camera's +0xd8: Brutus's
//!    Vengar view).
//!
//! The console blends in the frame buffer's values (sRGB), the UI here in linear light, where an
//! added colour over a lit scene comes out much fainter and the same alpha darkens less; so both
//! are drawn as a pass over the main camera's picture (src/scope_view.wgsl) in those values,
//! after the Vengar view's pass and before the HUD, which the game draws over them (the takes'
//! radar and bars are as bright scoped as not). The pass goes with the HUD: not under BF_NO_HUD,
//! nor while the HUD is hidden (a cutscene's letterbox, the map editor).
//!
//! In a window wider than the game's 4:3 picture both carry on into the sides as their CLAMP
//! addressing would: the frame's grey to the window's edges, the overlay's outer texels past its
//! quads (the demo's choice: the game's screen ends at the 4:3 edge).
//!
//! Test hooks: the scope's own (play.md, "The scope": `BF_TEST_SCOPE`, `BF_TEST_SCOPE_PRESS`,
//! `BF_SCOPE_LOG`); `BF_SCOPE_VIEW_LOG=1` prints the two widgets' scale and fade, the focus and
//! the ring and smear scales on each frame they change.

use super::*;
use bevy::{
    asset::weak_handle,
    core_pipeline::{core_3d::graph::{Core3d, Node3d}, fullscreen_vertex_shader::fullscreen_shader_vertex_state},
    ecs::query::QueryItem,
    image::BevyDefault,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer}, AddressMode, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FilterMode, FragmentState,
            MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, TextureDimension, TextureFormat,
            TextureSampleType,
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        view::ViewTarget,
        RenderApp,
    },
};
pub use pass::ScopePass;

/// The frame's quarter, h_1807ec04 (textures-common: 256 x 128, DXT3, one mip), which
/// FUN_000b4010 binds (FUN_000ba6d0): the frame's top-right quarter, the opening (black) at its
/// first column and last row, the outer grey (153 155 153) along its first row and last column,
/// the rim's light line (206) 158-163 texels out and the tick marks (about 62) inside it.
const FRAME_TEXTURE: u32 = 0x1807_EC04;
/// The frame's quadrant (HUD units): +0xe 256 and +0xc 128 (FUN_000b4010), the texture's size,
/// which the draw (0xb4240) lays 1:1 from the picture's middle. #60 take07 4.0 s: the model's rim
/// lies on the take's to the pixel, corners and all.
const FRAME_QUARTER: Vec2 = Vec2::new(256.0, 128.0);
/// Flint's overlay's corner smear, h_f5b6a9f8 (+0x6c, FUN_000b3520; 128 x 128, DXT3): black but
/// for cyan-teal streaks at its first column and last row, the brightest (0 170 198) 48-96 texels
/// along the edges from that corner.
const SMEAR_TEXTURE: u32 = 0xF5B6_A9F8;
/// Flint's overlay's dotted ring, h_f0d8a0a7 (+0x70, FUN_000b3520; 128 x 128, DXT3): a quarter
/// ring of teal (0 110 104) with yellow-green dots (71 144 88), 64-80 texels from its last column's
/// first texel, fading out by 96.
const RING_TEXTURE: u32 = 0xF0D8_A0A7;
/// The overlay's quadrant at scale 1 (HUD units): +0xe 160 and +0xc 120 (FUN_000b3520), times
/// the ring's or the smear's scale (floored: the CRT's floor 0x2ea51b, rounding down, so the
/// mirrored quads' negated sizes come out a unit larger when not whole) and the widget's.
const OVERLAY_QUARTER: Vec2 = Vec2::new(160.0, 120.0);
/// Both widgets' ease (a second): their scale moves toward its target by at most
/// (3 - 1 - 1) x 6 a second (FUN_000b4070: 0x40d6cc 3, 0x40d6c8 1, 0x3a4f78 1, 0x40d6c0 6; 0xb3700
/// the same from 0x40d70c, 0x40d708, 0x3a4f78 and 0x40d700). #60 take01 31.28-31.47 s: leaving
/// the scope, the frame's scale grows 0.2 a game frame (30 frames a second), 1.0 to 2.2.
const EASE: f32 = 6.0;
/// The scale both ease to out of the scope (0x40d6cc, 0x40d70c: 3.0), where their alpha is 0
/// and they're hidden.
const AWAY: f32 = 3.0;
/// Their alpha is 1 - clamp((scale - FADE_FROM) / (AWAY - FADE_FROM), 0, 1) (0x40d6c4, 0x40d704:
/// 2.0), and the sprite draw scales the colour bytes by it in the additive and the multiplying
/// modes (FUN_000ba4e0), where it can't go through the blend.
const FADE_FROM: f32 = 2.0;
/// The frame's scale in the scope: 1.0, target and current at once (FUN_000b4070, 0xb40d1). #60
/// take01 26.267 s: the frame is whole in the first scoped frame.
const FRAME_IN: f32 = 1.0;
/// Flint's overlay's scale in the scope: (1 - the reticle controller's +0x74) x 1 (0x40d708) + 1
/// (0x3a4f78). The controller's constructor (FUN_000c4f60, 0xc4fef) sets +0x74 to 0 and its +0x58
/// (FUN_000c52f0) clears it at the end of every update (0xc5a50 at 0xc5b7b); only its reset
/// FUN_000c5ca0 (a dead character's, a lost lock's) sets another value, -0.2, and the scope's
/// target doesn't matter then: so 2. #126 take16 (3.4 s, the overlay grown in since 2.27 s) fits
/// the model best at 2.0 of the scales 1.6-2.4 tried: at 1 the smear's streaks would ring the
/// middle, not the picture's corners.
const OVERLAY_IN: f32 = 2.0;
/// The ring's scale (+0x74) from the focus's share t of the clip range (`ScopeView::share`): 1.5
/// at t 0 (0x40d6f8), 1.2 at 1 (0x40d6fc), linear (0xb38fc-0xb391a).
const RING_SCALE: [f32; 2] = [1.5, 1.2];
/// The smear's (+0x78): 1.1 at t 0 (0x40d6f0), 1.0 at 1 (0x40d6f4), linear (0xb391d-0xb393b).
const SMEAR_SCALE: [f32; 2] = [1.1, 1.0];
/// The ring's and the smear's vertex colours: grey 30 (0x40d6ed) and 50 (0x40d6ec), alpha 255,
/// which the texture stage multiplies the texels by (as the frame's white gives its grey 153 the
/// takes' 40%). #126 take16: what the picture gained as the overlay grew in (2.267 to 3.4 s) is
/// 0.81 of the model's overlay at these colours, the camera's own small move in between making
/// up the rest of the frame-to-frame change.
const RING_COLOUR: f32 = 30.0;
const SMEAR_COLOUR: f32 = 50.0;
/// How fast the scope's focus (cam+0x1f8) follows the centre ray's depth (m a second, 0x40de20:
/// 1000); it takes it at once on the first scoped frame (cam+0x1f4 bit 0; FUN_000702a0).
const FOCUS_RATE: f32 = 1000.0;
/// The follow camera's near plane (m): clip-plane-near h_04570f05, 0.1 in every level (the
/// view's +0x24), which FUN_000702a0 clamps the focus to and 0xb38e3 measures its share from.
const NEAR: f32 = 0.1;
/// The far plane when the map names no follow camera the data has (m; the flat test floor): the
/// `follow_camera` archetype's 250, 35 levels' (`Game::follow_cameras`, `Level::follow_camera`).
const FAR: f32 = 250.0;

/// The pass's shader (src/scope_view.wgsl).
const PASS_SHADER: Handle<Shader> = weak_handle!("3c9e1f47-8a2b-4d6e-b5c0-7e1d2a9f4b38");

/// The scope views' plugin: their state after the reticle (its centre ray's depth is the focus)
/// and the scope's level (play.rs `follow_camera`); the pass in the render graph.
pub fn plugin(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    shaders.insert(&PASS_SHADER, Shader::from_wgsl(include_str!("../scope_view.wgsl"), file!()));
    app.add_plugins((ExtractComponentPlugin::<ScopePass>::default(), UniformComponentPlugin::<ScopePass>::default(),
                     ExtractResourcePlugin::<ScopeTextures>::default(), PassPlugin))
        .init_resource::<ScopeView>()
        .add_systems(OnEnter(AppState::Playing), (load_textures, |mut commands: Commands| commands.insert_resource(ScopeView::default())).after(setup))
        .add_systems(Update, update.after(reticle::update_reticle).before(update_weapons).run_if(in_state(AppState::Playing)));
}

/// The pass's settings (in a module of their own: the derives' own checks read as unused).
mod pass {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::{extract_component::ExtractComponent, render_resource::ShaderType}};

    /// What the main camera's scope pass draws this frame (scope_view.wgsl `Scope`).
    #[derive(Component, Clone, Copy, Default, Debug, ExtractComponent, ShaderType)]
    pub struct ScopePass {
        /// the frame: x its scale, y its fade (0: not drawn), z and w its quadrant (HUD units)
        pub frame: Vec4,
        /// Flint's overlay: x the ring's colour x the fade, y the smear's (0 and 0: not drawn)
        pub flint: Vec4,
        /// the ring's quads (HUD units, the widget's scale in them): x and y to the left and up
        /// (the floored sizes), z and w to the right and down (the negated ones)
        pub ring: Vec4,
        /// the smear's quads, as `ring`
        pub smear: Vec4,
    }
}

/// The three textures, as the console reads them (their bytes as they're stored, not as sRGB:
/// the pass works in the frame buffer's values), for the render world.
#[derive(Resource, Clone, ExtractResource)]
struct ScopeTextures {
    /// FRAME_TEXTURE
    frame: Handle<Image>,
    /// SMEAR_TEXTURE
    smear: Handle<Image>,
    /// RING_TEXTURE
    ring: Handle<Image>,
}

/// The two widgets' state: the frame's scale (+0x38: its constructor's 12 eases out of sight
/// before a scope can matter, so it starts at AWAY), Flint's overlay's (+0x84, from 3), the
/// scope's focus (cam+0x1f8, m) and whether this scope has taken it (cam+0x1f4 bit 0), and the
/// overlay's ring and smear scales (+0x74, +0x78: 1.0 from the constructor until the follow
/// camera's first update sizes them).
#[derive(Resource)]
struct ScopeView {
    frame: f32,
    overlay: f32,
    focus: f32,
    focused: bool,
    ring: f32,
    smear: f32,
    /// (BF_SCOPE_VIEW_LOG: the line printed last)
    logged: String,
}

impl Default for ScopeView {
    fn default() -> Self {
        ScopeView { frame: AWAY, overlay: AWAY, focus: NEAR, focused: false, ring: 1.0, smear: 1.0, logged: String::new() }
    }
}

impl ScopeView {
    /// The focus's share of the clip range, (focus - near) / (far - near) (0xb38e3-0xb38f9, with
    /// the view's +0x24 and +0x28).
    fn share(&self, far: f32) -> f32 {
        (self.focus - NEAR) / (far - NEAR).max(1e-3)
    }
}

/// A widget's alpha at a scale (FUN_000b4070 0xb4186-0xb41fd, 0xb3820-0xb389e).
fn fade(scale: f32) -> f32 {
    1.0 - ((scale - FADE_FROM) / (AWAY - FADE_FROM)).clamp(0.0, 1.0)
}

/// `from` moved toward `to` by at most `step` (the widgets' and the focus's ease).
fn toward(from: f32, to: f32, step: f32) -> f32 {
    from + (to - from).clamp(-step, step)
}

/// One texture's quads (see `ScopePass::ring`): 160 x 120 units x its own scale, floored as the
/// draw floors them (positive to the left and up, negated to the right and down), x the widget's.
fn quads(own: f32, widget: f32) -> Vec4 {
    let size = OVERLAY_QUARTER * own;
    Vec4::new(size.x.floor(), size.y.floor(), size.x.ceil(), size.y.ceil()) * widget
}

/// A game texture as the pass reads it: its bytes as stored (Rgba8Unorm), rows in the stored
/// order, so the console's texture coordinates read it as they read the original.
fn stored(game: &mut Game, images: &mut Assets<Image>, name: u32) -> Handle<Image> {
    let Some((w, h, px)) = game.texture_rgba(name) else {
        warn!("scope view: texture h_{name:08x} not found");
        return Handle::default();
    };
    images.add(Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px,
                          TextureFormat::Rgba8Unorm, RenderAssetUsages::RENDER_WORLD))
}

/// The three textures as the first map starts (common's: every level has them).
fn load_textures(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, loaded: Option<Res<ScopeTextures>>) {
    if loaded.is_some() {
        return;
    }
    let game = &mut game.0;
    commands.insert_resource(ScopeTextures {
        frame: stored(game, &mut images, FRAME_TEXTURE),
        smear: stored(game, &mut images, SMEAR_TEXTURE),
        ring: stored(game, &mut images, RING_TEXTURE),
    });
}

/// The two widgets' update each frame (FUN_000b4070, 0xb3700) and the focus they're sized by
/// (FUN_000702a0), then the main camera's pass settings.
#[allow(clippy::too_many_arguments)]
fn update(mut commands: Commands, time: Res<Time>, player: Res<Player>, reticle: Res<reticle::Reticle>, game: Res<GameData>,
          map: Res<MapLevel>, mut view: ResMut<ScopeView>, huds: Query<&InheritedVisibility, With<hud::HudRoot>>,
          mut cameras: Query<(Entity, Option<&mut ScopePass>), With<MainCamera>>) {
    let dt = frame_dt(&time);
    // char+0x7a8, and whose view it is: the frame (the skill's +0xa8 bit 0 clear) or Flint's
    // overlay, which the crosshair makes only for a skill with the bit (0xc6089)
    let scoped = player.scope_level > 0;
    let framed = skills::scope_frame(&player);
    // the frame: 1 at once in the scope, else toward 3
    view.frame = if scoped && framed { FRAME_IN } else { toward(view.frame, AWAY, EASE * dt) };
    // Flint's overlay: toward 2 in the scope, 3 out of it
    let target = if scoped && !framed { OVERLAY_IN } else { AWAY };
    view.overlay = toward(view.overlay, target, EASE * dt);
    // the focus (FUN_000702a0, scoped and alive): the centre ray's depth within the clip planes
    // (the map's follow camera's), taken at once on the scope's first frame, then followed at
    // FOCUS_RATE
    let far = map.0.as_ref().and_then(|l| game.0.follow_cameras.get(&l.follow_camera)).copied().unwrap_or(FAR).max(NEAR + 1e-3);
    if scoped && !player.dead {
        let f = reticle.depth.clamp(NEAR, far);
        view.focus = if view.focused { toward(view.focus, f, FOCUS_RATE * dt) } else { f };
        view.focused = true;
    } else {
        view.focused = false;
    }
    // the ring's and the smear's scales by the focus's share of the clip range (0xb38cd: the
    // follow camera's, the scope's camera)
    let t = view.share(far);
    view.ring = RING_SCALE[0] * (1.0 - t) + RING_SCALE[1] * t;
    view.smear = SMEAR_SCALE[0] * (1.0 - t) + SMEAR_SCALE[1] * t;
    // what the pass draws: with the HUD; the overlay for her view only and not in a vision mode
    // (FUN_000b3970: the camera's +0xd8)
    let shown = std::env::var("BF_NO_HUD").is_err() && huds.iter().all(|v| v.get());
    let frame_fade = if shown { fade(view.frame) } else { 0.0 };
    let flint_fade = if shown && !framed && !skills::vengar_on(&player) { fade(view.overlay) } else { 0.0 };
    let settings = ScopePass {
        frame: Vec4::new(view.frame, frame_fade, FRAME_QUARTER.x, FRAME_QUARTER.y),
        flint: Vec4::new(flint_fade * RING_COLOUR / 255.0, flint_fade * SMEAR_COLOUR / 255.0, 0.0, 0.0),
        ring: quads(view.ring, view.overlay),
        smear: quads(view.smear, view.overlay),
    };
    for (e, pass) in &mut cameras {
        match pass {
            Some(mut p) => *p = settings,
            None => {
                commands.entity(e).insert(settings);
            }
        }
    }
    // BF_SCOPE_VIEW_LOG=1: each frame something moves
    if std::env::var("BF_SCOPE_VIEW_LOG").is_ok() {
        let line = format!("frame {:.3} (fade {frame_fade:.3}), overlay {:.3} (fade {flint_fade:.3}), focus {:.2} m (share {t:.3} of {far} m), ring {:.3}, smear {:.3}",
                           view.frame, view.overlay, view.focus, view.ring, view.smear);
        if line != view.logged {
            println!("t={:.3} scope view: {line}", player.sim_time);
            view.logged = line;
        }
    }
}

/// The pass in the render graph: after the Vengar view's world pass and the glare (it draws over
/// the picture they leave, as the HUD's widgets do over the world's), before the HUD.
struct PassPlugin;

impl Plugin for PassPlugin {
    fn build(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.add_render_graph_node::<ViewNodeRunner<ScopeNode>>(Core3d, ScopeLabel)
            .add_render_graph_edges(Core3d, (vengar::VengarLabel, ScopeLabel, Node3d::EndMainPassPostProcessing))
            .add_render_graph_edge(Core3d, bf_viewer::glare::GlareLabel, ScopeLabel);
    }

    fn finish(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.init_resource::<PassPipeline>();
    }
}

/// The pass's node label.
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ScopeLabel;

/// The pass (scope_view.wgsl) on the main camera while either view shows.
#[derive(Default)]
struct ScopeNode;

impl ViewNode for ScopeNode {
    type ViewQuery = (&'static ViewTarget, &'static ScopePass, &'static DynamicUniformIndex<ScopePass>);

    fn run(&self, _graph: &mut RenderGraphContext, context: &mut RenderContext, (target, pass, index): QueryItem<Self::ViewQuery>,
           world: &World) -> Result<(), NodeRunError> {
        if pass.frame.y <= 0.0 && pass.flint.x <= 0.0 && pass.flint.y <= 0.0 {
            return Ok(());
        }
        let pipes = world.resource::<PassPipeline>();
        let Some(pipeline) = world.resource::<PipelineCache>().get_render_pipeline(pipes.pipeline) else { return Ok(()) };
        let Some(uniforms) = world.resource::<ComponentUniforms<ScopePass>>().uniforms().binding() else { return Ok(()) };
        let Some(textures) = world.get_resource::<ScopeTextures>() else { return Ok(()) };
        let images = world.resource::<RenderAssets<GpuImage>>();
        let (Some(frame), Some(smear), Some(ring)) = (images.get(&textures.frame), images.get(&textures.smear), images.get(&textures.ring)) else {
            return Ok(());
        };
        let post = target.post_process_write();
        let bind_group = context.render_device().create_bind_group("scope_view_pass", &pipes.layout, &BindGroupEntries::sequential((
            post.source, &pipes.picture, uniforms.clone(), &frame.texture_view, &smear.texture_view, &ring.texture_view, &pipes.clamped,
        )));
        let mut draw = context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("scope_view_pass"),
            color_attachments: &[Some(RenderPassColorAttachment { view: post.destination, resolve_target: None, ops: Operations::default() })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        draw.set_render_pipeline(pipeline);
        draw.set_bind_group(0, &bind_group, &[index.index()]);
        draw.draw(0..3, 0..1);
        Ok(())
    }
}

/// The pass's pipeline: the picture and its sampler, the settings, the three textures and their
/// sampler (linear, clamped to the edge: the widgets' CLAMP addressing, FUN_0006a610 on texture
/// stage 0's ADDRESSU / ADDRESSV, `[0x4698e0]+0x734` / `+0x74c`).
#[derive(Resource)]
struct PassPipeline {
    layout: BindGroupLayout,
    picture: Sampler,
    clamped: Sampler,
    pipeline: CachedRenderPipelineId,
}

impl FromWorld for PassPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>().clone();
        let layout = device.create_bind_group_layout("scope_view_layout", &BindGroupLayoutEntries::sequential(ShaderStages::FRAGMENT, (
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
            uniform_buffer::<ScopePass>(true),
            texture_2d(TextureSampleType::Float { filterable: true }),
            texture_2d(TextureSampleType::Float { filterable: true }),
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
        )));
        let linear = SamplerDescriptor { mag_filter: FilterMode::Linear, min_filter: FilterMode::Linear, ..default() };
        let picture = device.create_sampler(&linear);
        let clamped = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..linear
        });
        let pipeline = world.resource_mut::<PipelineCache>().queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("scope_view_pass".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: PASS_SHADER,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState { format: TextureFormat::bevy_default(), blend: None, write_mask: ColorWrites::ALL })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });
        PassPipeline { layout, picture, clamped, pipeline }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leaving the scope the frame grows from 1 to 3 in 1/3 s and fades over the last half (#60
    /// take01: 0.2 a game frame at 30 a second).
    #[test]
    fn frame_grows_out() {
        let mut s = FRAME_IN;
        let mut steps = 0;
        while s < AWAY {
            s = toward(s, AWAY, EASE / 30.0);
            steps += 1;
        }
        assert_eq!(steps, 10);
        assert!((fade(2.2) - 0.8).abs() < 1e-5);
        assert_eq!(fade(FRAME_IN), 1.0);
        assert_eq!(fade(AWAY), 0.0);
    }

    /// The overlay's quads at scale 2 with the focus at the near plane: the smear's 176 x 132 x 2
    /// (past the 4:3 picture's 320 x 240, so its streaks sit at the corners), the ring's 240 x 180 x 2.
    #[test]
    fn overlay_quads() {
        assert_eq!(quads(SMEAR_SCALE[0], OVERLAY_IN), Vec4::new(352.0, 264.0, 352.0, 264.0));
        assert_eq!(quads(RING_SCALE[0], OVERLAY_IN), Vec4::new(480.0, 360.0, 480.0, 360.0));
        // a scale that isn't whole: floored to the left and up, the negated sizes a unit larger
        assert_eq!(quads(1.488, 1.0), Vec4::new(238.0, 178.0, 239.0, 179.0));
    }
}
