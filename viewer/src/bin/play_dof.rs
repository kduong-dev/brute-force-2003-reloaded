//! The screen's depth of field (#38): shader 0x1e itself, the scope's blur and the low-health blur
//! in one pass (FUN_000702a0's camera block, 0x7173c-0x7188d; the renderer's stage 8,
//! FUN_000e7d40, whose composite FUN_000e5730 draws it; the ticket's second comment has the reads).
//!
//! **The camera block** (the numbers the shader's constants carry):
//!  - **Scoped and alive** (cam+0x4c bit 3): S (+0x117) 1, mode 2, bias (+0x120) 0, and the focus
//!    (+0x118) is the aim ray's hit distance (+0x1f8, clamped to the clip planes and eased at
//!    1000 m/s) — here `reticle::Reticle::depth`, the centre ray's depth the scope view already
//!    sizes its frame by.
//!  - **Low health** (the hit points under half the max, not scoped): S 0.8, mode 2, the focus the
//!    character's own depth, and bias = hp / 0.5 - 1.
//!  - **A cutscene or script state**: mode 3, a different soft-contrast effect (shader 0x11), not
//!    this.
//!
//! **The pass** (`dof.wgsl`): the picture downsampled twice by 2 x 2 box steps — the half-size
//! copy and the quarter-size one — then one full-screen quad over the frame,
//! `1/4 sharp + 1/2 half + 1/4 quarter`, blended by
//! `alpha = clamp(64 (2S - 1) |z - focus| + bias, 0, 1)`. The depth comes from bevy's own depth
//! texture (reverse-Z: its value is near / z), so |z - focus| is metres as the game's is. The two
//! copies are sampled where four pixels (and four 2 x 2 blocks) meet, which is the box step
//! without separate targets. At S 1 the alpha reaches 1 about a centimetre past the focus, so the
//! effect reads as the focus distance sharp and everything else blurred; at low health the
//! character stays sharp and the world either side of him blurs.
//!
//! The pass runs after the tonemapping and before the Vengar view's pass, which is before the
//! scope view's and the HUD's — the game's stage 8 runs before its HUD too, so the scope's frame
//! and the bars draw over the blur. It draws nothing while S is 0 (neither scoped nor hurt), so
//! there is no component to add or remove.
//!
//! **One stand-in**: the character's own depth at low health is the campaign camera block's boom,
//! 3.2 m (the classes' `camera` blocks), where the game reads it from the view. Demo-own,
//! 2026-11-09.
//!
//! Test hooks: `BF_DOF_LOG=1` prints each change (on and off, the kind, the focus and the health);
//! `BF_DOF=<from>-<to>,...` holds it on over those seconds whatever the health or the scope (its
//! focus his own depth), so the on and off play in one run; `BF_TEST_HURT` (play_chatter.rs)
//! takes the player's hit points down for the low-health side; the scope's own hooks
//! (play/scope.md) the scope's.

use super::*;
use bevy::{
    asset::weak_handle,
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    image::BevyDefault,
    render::{
        camera::CameraUpdateSystem,
        extract_component::{ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::{
            binding_types::{sampler, texture_2d, texture_depth_2d, texture_depth_2d_multisampled, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            FilterMode, FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderDefVal, ShaderStages,
            TextureFormat, TextureSampleType, TextureUsages,
        },
        renderer::{RenderContext, RenderDevice},
        view::{Msaa, ViewDepthTexture, ViewTarget},
        RenderApp,
    },
    window::PrimaryWindow,
};

/// The pass's shader (src/dof.wgsl).
const PASS_SHADER: Handle<Shader> = weak_handle!("8f3d2c47-1a6e-4b90-9c2d-5e7a1b4f6d23");

/// The share of the max hit points the low-health blur starts under (FUN_000702a0's 0.5).
const DOF_LOW_HEALTH: f32 = 0.5;
/// The character's own depth for the focus at low health (m): the campaign camera block's boom
/// (3.2 m at 1 degree behind the head). A stand-in for the view's depth of him: demo-own.
const DOF_SELF_DEPTH: f32 = 3.2;
/// The focus's ease in the scope (m a second, cam+0x118's 1000 at 0x40de20).
const DOF_FOCUS_RATE: f32 = 1000.0;
/// The follow camera's near plane (m; clip-plane-near h_04570f05, 0.1 in every level), which the
/// game clamps the focus to.
const DOF_NEAR: f32 = 0.1;
/// The far plane when the map names no follow camera (m; play_scope_view.rs's FAR).
const DOF_FAR: f32 = 250.0;

/// The pass's settings (in a module of their own: the derives' own checks read as unused, as in
/// play_vengar.rs).
mod pass {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::{extract_component::ExtractComponent, render_resource::ShaderType}};

    /// What the pass reads this frame (`dof.wgsl` `Dof`): S, the focus, the bias, the near plane
    /// and the picture's size.
    #[derive(Component, Clone, Copy, Default, Debug, ExtractComponent, ShaderType)]
    pub struct DofPass {
        /// c0: S, 0 when the pass draws nothing
        pub s: f32,
        /// c1: the focus's distance from the camera (m)
        pub focus: f32,
        /// the bias (+0x120), as c2's `(bias + 1) / 2` is taken from it in the game
        pub bias: f32,
        /// the camera's near plane (m): the depth texture's value is near / z
        pub near: f32,
        /// the picture's size in pixels
        pub screen: Vec4,
    }
}
pub use pass::DofPass;

/// The effect's focus state: the distance it's on (m) and whether it has taken one since the
/// scope's first frame (the game's cam+0x1f4 bit 0: taken at once, then followed).
#[derive(Resource, Default)]
pub struct Focus {
    depth: f32,
    taken: bool,
    /// (BF_DOF_LOG: what it printed last, so a still picture prints once)
    logged: String,
}

pub fn plugin(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    shaders.insert(&PASS_SHADER, Shader::from_wgsl(include_str!("../dof.wgsl"), file!()));
    app.add_plugins((ExtractComponentPlugin::<DofPass>::default(), UniformComponentPlugin::<DofPass>::default(), PassPlugin))
        .init_resource::<Focus>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| commands.insert_resource(Focus::default()))
        .add_systems(Update, update.after(follow_camera).after(reticle::update_reticle).before(update_weapons)
            .run_if(in_state(AppState::Playing)))
        // (the depth must be bindable as a texture: bevy creates it from the camera's usages,
        // before the render world's extraction and its view targets)
        .add_systems(PostUpdate, depth_usage.before(CameraUpdateSystem));
}

/// `from` moved toward `to` by at most `step`.
fn toward(from: f32, to: f32, step: f32) -> f32 {
    from + (to - from).clamp(-step, step)
}

/// Whether `t` is in one of `BF_DOF`'s windows (`<from>-<to>,...`, seconds).
fn forced_on(t: f32) -> bool {
    std::env::var("BF_DOF").map(|v| v.split(',').filter_map(|w| w.split_once('-'))
        .filter_map(|(a, b)| Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?)))
        .any(|(a, b)| (a..b).contains(&t))).unwrap_or(false)
}

/// The depth texture as something the pass can read (`Camera3d.depth_texture_usages` gains
/// `TEXTURE_BINDING`; bevy's own depth of field does the same).
fn depth_usage(mut cams: Query<&mut Camera3d, With<MainCamera>>) {
    for mut cam in &mut cams {
        let mut usages = TextureUsages::from(cam.depth_texture_usages);
        if !usages.contains(TextureUsages::TEXTURE_BINDING) {
            usages |= TextureUsages::TEXTURE_BINDING;
            cam.depth_texture_usages = usages.into();
        }
    }
}

/// The camera block's figures this frame: S, the focus, the bias and the near plane.
fn update(time: Res<Time>, player: Res<Player>, reticle: Res<reticle::Reticle>, game: Res<GameData>, map: Res<MapLevel>,
          windows: Query<&Window, With<PrimaryWindow>>, mut focus: ResMut<Focus>, mut commands: Commands,
          mut cams: Query<(Entity, Option<&mut DofPass>, &Projection), With<MainCamera>>) {
    let dt = frame_dt(&time);
    let hp = (player.health / player.max_health.max(1.0)).clamp(0.0, 1.0);
    let forced = forced_on(player.sim_time);
    let scoped = !forced && player.scope_level > 0 && !player.dead;
    let low = !scoped && !player.dead && (forced || hp < DOF_LOW_HEALTH);
    let log = std::env::var("BF_DOF_LOG").is_ok();
    let far = map.0.as_ref().and_then(|l| game.0.follow_cameras.get(&l.follow_camera)).copied().unwrap_or(DOF_FAR).max(DOF_NEAR + 1e-3);
    let (s, bias) = if scoped { (1.0, 0.0) } else if low { (0.8, hp / DOF_LOW_HEALTH - 1.0) } else { (0.0, 0.0) };
    if s <= 0.0 {
        focus.taken = false;
        if focus.logged != "off" {
            focus.logged = "off".into();
            if log {
                println!("t={:.2} depth of field off (health {:.0}%)", player.sim_time, hp * 100.0);
            }
        }
    } else {
        // the focus: the aim ray's depth in the scope, else his own depth
        let target = if scoped { reticle.depth.clamp(DOF_NEAR, far) } else { DOF_SELF_DEPTH.min(far) };
        focus.depth = if focus.taken { toward(focus.depth, target, DOF_FOCUS_RATE * dt) } else { target };
        focus.taken = true;
        if log {
            let line = format!("{} {:.2} m, S {s:.2}, bias {bias:.2}, health {:.0}%", if scoped { "scope" } else { "low health" },
                               focus.depth, hp * 100.0);
            if line != focus.logged {
                focus.logged = line.clone();
                println!("t={:.2} depth of field: {line}", player.sim_time);
            }
        }
    }
    let screen = windows.single().map(|w| Vec2::new(w.width(), w.height()) * w.scale_factor()).unwrap_or(Vec2::new(1280.0, 720.0));
    for (e, pass, projection) in &mut cams {
        let near = match projection { Projection::Perspective(p) => p.near, _ => DOF_NEAR };
        let settings = DofPass { s, focus: focus.depth, bias, near, screen: screen.extend(0.0).extend(0.0) };
        match pass {
            Some(mut p) => *p = settings,
            None => {
                commands.entity(e).insert(settings);
            }
        }
    }
}

/// The pass's plugin: the node between the tonemapping and the Vengar view's pass.
#[derive(Default)]
struct PassPlugin;

impl Plugin for PassPlugin {
    fn build(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.add_render_graph_node::<ViewNodeRunner<DofNode>>(Core3d, DofLabel)
            .add_render_graph_edges(Core3d, (Node3d::Tonemapping, DofLabel, super::vengar::VengarLabel));
    }

    fn finish(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.init_resource::<DofPipelines>();
    }
}

/// The pass's node label (before the Vengar view's and the scope view's, after the tonemapping).
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct DofLabel;

/// The pass (dof.wgsl): the picture, the depth and the settings.
#[derive(Default)]
struct DofNode;

impl ViewNode for DofNode {
    type ViewQuery = (&'static ViewTarget, &'static DofPass, &'static DynamicUniformIndex<DofPass>,
                      &'static ViewDepthTexture, Option<&'static Msaa>);

    fn run(&self, _graph: &mut RenderGraphContext, context: &mut RenderContext,
           (target, pass, index, depth, msaa): QueryItem<Self::ViewQuery>, world: &World) -> Result<(), NodeRunError> {
        if pass.s <= 0.0 {
            return Ok(());
        }
        let Some(uniforms) = world.resource::<ComponentUniforms<DofPass>>().uniforms().binding() else { return Ok(()) };
        let pipes = world.resource::<DofPipelines>();
        let variant = match msaa {
            Some(m) if *m != Msaa::Off => &pipes.multisampled,
            _ => &pipes.single,
        };
        let Some(pipeline) = world.resource::<PipelineCache>().get_render_pipeline(variant.pipeline) else { return Ok(()) };
        let post = target.post_process_write();
        let bind_group = context.render_device().create_bind_group("dof_pass", &variant.layout,
                                                                   &BindGroupEntries::sequential((post.source, &pipes.sampler, uniforms.clone(), depth.view())));
        let mut draw = context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("dof_pass"),
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

/// One depth binding's pipeline: the plain depth texture or the multisampled one.
struct Variant {
    layout: BindGroupLayout,
    pipeline: CachedRenderPipelineId,
}

/// The pass's two pipelines (the camera's MSAA picks one, as bevy's depth of field does: the depth
/// texture is multisampled while MSAA is on) and their sampler.
#[derive(Resource)]
struct DofPipelines {
    sampler: Sampler,
    single: Variant,
    multisampled: Variant,
}

impl FromWorld for DofPipelines {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>().clone();
        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });
        let single = variant(world, false);
        let multisampled = variant(world, true);
        DofPipelines { sampler, single, multisampled }
    }
}

/// The pipeline for a depth texture of one kind.
fn variant(world: &mut World, multisampled: bool) -> Variant {
    let device = world.resource::<RenderDevice>().clone();
    let depth = if multisampled { texture_depth_2d_multisampled() } else { texture_depth_2d() };
    let layout = device.create_bind_group_layout("dof_pass_layout", &BindGroupLayoutEntries::sequential(ShaderStages::FRAGMENT, (
        texture_2d(TextureSampleType::Float { filterable: true }),
        sampler(SamplerBindingType::Filtering),
        uniform_buffer::<DofPass>(true),
        depth,
    )));
    let shader_defs = if multisampled { vec![ShaderDefVal::Bool("MULTISAMPLED".into(), true)] } else { vec![] };
    let pipeline = world.resource_mut::<PipelineCache>().queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(if multisampled { "dof_pass_msaa" } else { "dof_pass" }.into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_shader_vertex_state(),
        fragment: Some(FragmentState {
            shader: PASS_SHADER,
            shader_defs,
            entry_point: "fragment".into(),
            targets: vec![Some(ColorTargetState { format: TextureFormat::bevy_default(), blend: None, write_mask: ColorWrites::ALL })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
        zero_initialize_workgroup_memory: false,
    });
    Variant { layout, pipeline }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shader 0x1e's alpha (0x1e): clamp(64 (2S - 1) |z - focus| + bias, 0, 1). In the scope
    /// (S 1, bias 0) the focus is sharp and a metre off is already 64/255 of the blur; at low
    /// health (S 0.8) the bias pulls the near side down as the hit points fall.
    #[test]
    fn the_alpha_is_the_games_formula() {
        let alpha = |s: f32, focus: f32, bias: f32, z: f32| (64.0 * (2.0 * s - 1.0) * (z - focus).abs() + bias).clamp(0.0, 1.0);
        // the scope: focused at 25 m, the pixel at the focus sharp, 1 cm off over the top
        assert_eq!(alpha(1.0, 25.0, 0.0, 25.0), 0.0);
        assert!((alpha(1.0, 25.0, 0.0, 25.01) - 0.64).abs() < 1e-4);
        assert_eq!(alpha(1.0, 25.0, 0.0, 10.0), 1.0);
        assert_eq!(alpha(1.0, 25.0, 0.0, 26.0), 1.0);
        // low health at a quarter: S 0.8 (the slope 38.4 a metre) and the bias -0.5, which is over
        // the top a centimetre out
        assert_eq!(alpha(0.8, 3.2, -0.5, 3.2), 0.0);
        assert_eq!(alpha(0.8, 3.2, -0.5, 3.25), 1.0);
        assert_eq!(alpha(0.8, 3.2, -0.5, 10.0), 1.0);
        // just under half health (hp 0.49, the bias -0.02) a centimetre out is 0.364
        assert!((alpha(0.8, 3.2, -0.02, 3.21) - 0.364).abs() < 1e-3);
    }

    /// The focus takes its first value at once, then follows at FOCUS_RATE (cam+0x1f4 bit 0 and
    /// 0x40de20).
    #[test]
    fn the_focus_is_taken_at_once_and_then_followed() {
        let mut focus = Focus::default();
        assert!(!focus.taken);
        focus.depth = toward(3.2, 40.0, DOF_FOCUS_RATE / 30.0);
        assert!((focus.depth - (3.2 + 1000.0 / 30.0)).abs() < 1e-3);
    }
}
