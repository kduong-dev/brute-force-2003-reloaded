//! Brutus's Spirit of Vengar beyond his own character (#122): the charge into enemies and the
//! Vengar view. His skill's own part (the toggle, regeneration, the drain and the damage he
//! takes) is play_skills.rs's `Vengar`. Sources: the spec and takes in `todo/35-special-skills/`
//! (spec.md Part 3: takes 06-09, 22-24, 27, 30) and the functions cited (viewer/docs/code-map.md,
//! "Skills and stamina" and, for the view's materials and shaders, "Effects / ALE"); the page is
//! viewer/docs/skills.md.
//!
//!  - **The charge** (FUN_0013c1c0, which his update 0x13c0b0 runs every frame while he dashes,
//!    motion state 8, with the skill on; the demo's dash is the sprint): if a ray from his body
//!    along his facing, or 0.2 rad either side, meets a hostile character within 2 m
//!    (FUN_00123080), every character not on his side within 1.5 m (h_e26c9e84) and 30 degrees
//!    (h_f3d82857) of his facing is hit: h_14935eb6 (200) DTYPE_EXPLOSION through FUN_00223160,
//!    with no hit part (play_damage.rs: on STANDARD an enemy takes 280, take08), the effect type
//!    h_0dcd4fc7 on it (its sound; its ALE `vengar_hit` never starts), his type's h_1629a90a sound
//!    (h_e4c94dfe), his charge line h_13601f7b through the chatter gate, and a kill throws the body
//!    (FUN_000e00b0). Each hit sets the skill's pending cost to h_0461a627 (100), which his next
//!    frame's drain takes: one charge per activation (takes 08, 22: stamina 0 and the skill off
//!    the tick after the hit). Walking (take23) or with the skill off (take24): nothing.
//!  - **The Vengar view**, while the controlled character's Vengar is on (his on sets the vision
//!    mode 2, 0x13c020; the game's renderer FUN_000e81e0 takes it from the player's view): the
//!    world, terrain and objects, is lit by two white lights straight up and down in place of the
//!    level's (FUN_000dcc10: `vision_lights`, `vision_terrain`; their strength a fit,
//!    VISION_LIGHT); after it's drawn, the world pass
//!    (FUN_000e66d0: a zoom blur toward the screen's middle, made grey with blue darks, laid over
//!    at 70%: src/vengar.wgsl); then the living characters are drawn over it as heat figures
//!    (the vision set's material, class 0xe2c39ac2: `HeatMaterial`, src/heat.wgsl; bright for him
//!    and his hostiles, dim for his friends) with a halo; the HUD on top. Each switch, on and off,
//!    starts the view's colour fade: the picture times red, back to white in 0.6 s (FUN_000c6980).
//!
//! Test hooks: `BF_TEST_SKILL`, `BF_TEST_GOTO` with `BF_TEST_SPRINT` (the dash) and
//! `BF_TEST_GO_AT=<s>` (when the goto sets off: take08's dash at 4 s), and the test map's
//! `BF_TEST_NPC` / `BF_TEST_NPC_HP` (a frozen enemy to charge, as takes 08 and 22);
//! `BF_SKILL_LOG=1` prints each charge hit, each switch of the view, and the hit points of
//! every character with Vengar on each half second.

use super::*;
use bevy::{
    asset::weak_handle,
    core_pipeline::{
        core_3d::{graph::{Core3d, Node3d}, Camera3dDepthLoadOp},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
        tonemapping::Tonemapping,
    },
    ecs::query::QueryItem,
    image::BevyDefault,
    pbr::MaterialPlugin,
    render::{
        camera::{CameraMainTextureUsages, CameraOutputMode, CameraUpdateSystem},
        extract_component::{ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin},
        mesh::VertexAttributeValues,
        render_graph::{NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer}, AsBindGroup, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BlendState, CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            Sampler, SamplerBindingType, SamplerDescriptor, ShaderRef, ShaderStages, TextureFormat, TextureSampleType, TextureUsages,
        },
        renderer::{RenderContext, RenderDevice},
        view::{RenderLayers, ViewTarget},
        RenderApp,
    },
    window::PrimaryWindow,
};
pub use pass::{HeatParams, VengarPass};

/// FUN_00123080 looks ahead before the charge: a ray this long (m, the 2.0 at 0x12315d) from his
/// body along his facing (the +0x1b4 object's vtable +0x30, 0x10fbf0: flat)...
const GATE_REACH: f32 = 2.0;
/// ... and the same turned this far (radians) either way (FUN_00039380 with 0.2 at 0x1231b3, then
/// -0.4 from there at 0x12326b), each meeting only a hostile character's body (its filter 0x123000
/// takes characters, not him, hostile to his team by the team table: FUN_000a8520).
const GATE_TURN: f32 = 0.2;
/// The least squared distance (m^2) a victim must be off his body (0x3a5150), so one standing in
/// him isn't hit at an undefined angle.
const NEAR: f32 = 1.421_085_5e-14;
/// The charge's damage type, DTYPE_EXPLOSION (FUN_00223160's second argument, 10 at 0x13c45d).
const DAMAGE_TYPE: i64 = 10;
/// The chatter tag of a charge hit (FUN_00140a60 at 0x13c3fa): Brutus's block has probability 60,
/// replay-delay 15 s, priority 5 (brutus_chatter.xml), through play_skills.rs's gate.
const LINE_CHARGE: u32 = 0x1360_1F7B;
/// A charge kill's knock-back (FUN_000e00b0 at 0x13c50c): along Brutus's velocity (char+0x470,
/// x and z) and up by h_f5dae098 (2), the body's push here (play.rs `Ragdoll::new`) scaled by this.
/// A fit to take22 (the trooper thrown forward against the tower ~3-4 m away and down at its
/// foot in 0.8 s): FUN_000e00b0's two other values, h_0e406b26 2 and h_fa20e794 75, go into the
/// ragdoll's impulse list, whose reader wasn't traced.
const KNOCK_SCALE: f32 = 1.0;

/// How long the view's colour fade takes to go from its colour to none (s): the 0.6 FUN_000c6980 is
/// given at both switches (0x13c08b on, 0x126e53 off; take06: 2.20-2.80 s and 20.17-20.80 s).
const FADE_TIME: f32 = 0.6;
/// The vision pair's strength in the console's terms (the light a surface facing it takes, as a
/// share of its texture, in gamma terms): a fit. FUN_000dcc10 gives the pair white (1.0), but the
/// takes' Vengar views are lit at about a third of that. Worked back through the world pass from
/// each take's ground, with the ground's texture from the same spot's normal view, the light on it
/// is 0.67 on m03_a (take08 3.5 s; its terrain ambient 0.25), 0.33 on e34 (take06 3.5 s) and 0.35
/// on e01 (take09 8.0 s; ambients 0.05 and 0.04): the pair's share 0.42, 0.29 and 0.31. take06's
/// grass, which faces up as the ground does, comes out as grey as the ground (58). What scales the
/// shaders' light isn't traced.
const VISION_LIGHT: f32 = 0.35;
/// The render layer the heat figures are drawn on, by the heat camera alone.
const HEAT_LAYER: usize = 1;
/// The halo's weight for each of its 24 taps (vengar.wgsl): a fit to take09 (Brutus's edge at 8.0
/// s: the halo adds about (69, 47, 0) a step out and (27, 20, 0) two steps out, nothing at three).
const GLOW: f32 = 0.07;
/// A heat figure's look (heat.wgsl): the colour at the middle of its ramp and at its far end
/// (sRGB 0-1), the curve's power, and where the ramp starts and ends (distances of the texture
/// coordinate from the texture's middle: 0 facing the view and 0.5 at the rims, before the
/// scroll, `scroll`). The code's colour is (k, k, 0) x the material's texture, k 0.8 for the
/// viewer himself and his hostiles, 0.2 for his friends (0xa88b2-0xa890c); the texture wasn't
/// found, so the colours are fitted to the takes: the bright figures R 253-255 throughout and G
/// from 254 to ~51 (take09 8.0 s, take08 3.5 s, take06 5.0 s: G's 5th to 95th percentiles 55-254,
/// 67-254, 93-254); the dim ones R 84-130, G 17-74 (take09's Tex, 113 / 44 on average).
type Look = ([f32; 3], [f32; 3], f32, [f32; 2]);
const BRIGHT: Look = ([1.0, 1.0, 0.02], [0.89, 0.18, 0.0], 1.0, [0.2, 0.6]);
const DIM: Look = ([0.51, 0.29, 0.01], [0.33, 0.07, 0.0], 1.0, [0.2, 0.6]);
/// The heat figures' pulse: the material scrolls its texture down by its speed (+0x1c) times its
/// time (+0x18), which the material library's update adds each frame's time to (0x25cf30, from
/// FUN_001ea830): FUN_000a8610 puts the product in the vertex shader's texture offset (c28.y,
/// 0xa8919-0xa892b). The takes' figures change colour with a period of 1.0 s, all of them in step
/// and the world not (take09 3-10 s: the autocorrelation of Brutus's figure's mean green peaks at
/// 1.000 s, 0.82; take06 3-9 s the same, 0.81; Brutus's and the enemy's greens correlate 0.975),
/// the figure's mean green swinging 104-175 (take09) and 125-184 (take06, 5th-95th percentiles;
/// the demo's 103-164 with these values, e01 as take09). The speed's
/// writer wasn't found, nor the texture, so this is a fit: the ramp's middle moves up and down
/// the texture by a sine of this period, between 0 and 0.45 (`SCROLL_MID` +- `SCROLL_SWING`).
const PULSE_PERIOD: f32 = 1.0;
/// The middle of the pulse's offset, and its swing either way (texture units, see PULSE_PERIOD).
const SCROLL_MID: f32 = 0.225;
const SCROLL_SWING: f32 = 0.225;

/// The heat texture's offset down at `clock` s (PULSE_PERIOD).
fn scroll(clock: f32) -> f32 {
    SCROLL_MID + SCROLL_SWING * (std::f32::consts::TAU * clock / PULSE_PERIOD).sin()
}

/// The pass's and the heat material's shaders (src/vengar.wgsl, src/heat.wgsl).
const PASS_SHADER: Handle<Shader> = weak_handle!("5e7f3a21-9c4d-4b8e-a6f2-0d1c3b5a7e90");
const HEAT_SHADER: Handle<Shader> = weak_handle!("8b2d6e14-3f7a-4c19-9e05-b4a8c2d1f6e3");

/// The Vengar plugin: the charge, after every character's skill step (so its pending cost is the
/// next frame's drain, as the game's update sets it after the drain) and before the sounds; the
/// view's state and the heat figures after it; the heat camera's projection the main camera's.
pub fn plugin(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    shaders.insert(&PASS_SHADER, Shader::from_wgsl(include_str!("../vengar.wgsl"), file!()));
    shaders.insert(&HEAT_SHADER, Shader::from_wgsl(include_str!("../heat.wgsl"), file!()));
    app.add_plugins((MaterialPlugin::<HeatMaterial>::default(), ExtractComponentPlugin::<VengarPass>::default(),
                     UniformComponentPlugin::<VengarPass>::default(), PassPlugin))
        .init_resource::<VengarView>()
        .init_resource::<TerrainCopies>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| {
            commands.insert_resource(VengarView::default());
            commands.insert_resource(TerrainCopies::default());
        })
        .add_systems(Update, (charge.after(skills::update_skills).before(play_sounds), log_health.after(skills::update_skills),
                              (heat_camera, view_state, heat_figures, vision_lights, vision_terrain).chain().after(charge))
            .run_if(in_state(AppState::Playing)))
        .add_systems(PostUpdate, follow_projection.before(CameraUpdateSystem));
}

/// BF_SKILL_LOG: the hit points of every character with Vengar on, each half second (its
/// regeneration, take07's +1.01/s).
fn log_health(player: Res<Player>, squad: Res<Squad>, mut last: Local<f32>) {
    if std::env::var("BF_SKILL_LOG").is_err() {
        return;
    }
    let t = player.sim_time;
    if (t * 2.0).floor() > (*last * 2.0).floor() {
        for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| skills::vengar_on(u)) {
            println!("t={t:.2} {}: Vengar on, {:.2} / {:.0} hp", unit_name(u), u.health, u.max_health);
        }
    }
    *last = t;
}

/// The character at `k`: 0 the player's, then the squad's (NPCs included).
fn unit<'a>(player: &'a mut Player, squad: &'a mut Squad, k: usize) -> &'a mut Player {
    if k == 0 { player } else { &mut squad.0[k - 1] }
}

/// Whether `u` dashes: motion state 8 (MT_DASH), the demo's sprint on the ground (as
/// play_skills.rs's refill reads it), not knocked down.
fn dashing(u: &Player) -> bool {
    !knocked_down(u) && matches!(u.action, Action::None) && matches!(u.gait, Gait::Sprint)
}

/// Where a character's body is (its physics body's place, char vtable +0x40): its position, raised
/// by a jump.
fn body(u: &Player) -> Vec3 {
    u.position + Vec3::Y * u.height
}

/// His facing on the ground (the model faces -z at yaw 0).
fn facing(u: &Player) -> Vec3 {
    Vec3::new(-u.yaw.sin(), 0.0, -u.yaw.cos())
}

/// Brutus's charge (FUN_0013c1c0) for every character with Vengar on and dashing (see the module
/// notes).
#[allow(clippy::too_many_arguments)]
fn charge(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
          difficulty: Res<damage::Difficulty>, mut history: ResMut<skills::LineHistory>, mut reserve: ResMut<ammo::Reserve>,
          mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut images: ResMut<Assets<Image>>,
          mut materials: ResMut<Assets<StandardMaterial>>) {
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    let cfg = game.0.combat_skills.vengar;
    let n = 1 + squad.0.len();
    for i in 0..n {
        let (from, ahead, team, who, velocity, now) = {
            let u = unit(&mut player, &mut squad, i);
            if u.dead || u.loaded.is_none() || !skills::vengar_on(u) || !dashing(u) {
                continue;
            }
            (body(u), facing(u), u.team, u.who(), u.last_velocity, u.sim_time)
        };
        // the look ahead: a hostile body on one of the three rays
        let rays = [0.0, GATE_TURN, -GATE_TURN].map(|a| Quat::from_rotation_y(a) * ahead);
        let gate = (0..n).filter(|&j| j != i).any(|j| {
            let v = unit(&mut player, &mut squad, j);
            !v.dead && damage::hostile(v.team, team) && rays.iter().any(|&d| ray_body(from, d, v, GATE_REACH).is_some())
        });
        if !gate {
            continue;
        }
        let cos = (cfg.cone.to_radians()).cos();
        let mut hits = 0;
        for j in (0..n).filter(|&j| j != i) {
            let v = unit(&mut player, &mut squad, j);
            // characters (alive, not on the ground: a body flagged for the ragdoll, +0x7d8 & 3, is
            // left out), not in his squad (FUN_0005beb0: the demo's sides stand for the squads)
            if v.dead || knocked_down(v) || v.team == team || v.loaded.is_none() {
                continue;
            }
            let d = body(v) - from;
            let d2 = d.length_squared();
            if d2 <= NEAR || d2 >= cfg.reach * cfg.reach || d.dot(ahead) < d2.sqrt() * cos {
                continue;
            }
            hits += 1;
            // the effect type on it (FUN_000def50 at its body): its sound, and the ALE effects due at
            // once. Its one effect, vengar_hit, has a creation-time of 6.9e28 s (`EffectType::starts`):
            // it never shows, as in takes 08 and 22
            let at = Transform::from_translation(body(v)).with_rotation(Quat::from_rotation_y(v.yaw));
            if let Some(fx) = game.0.effect_type_defs.get(&cfg.hit_effect).cloned() {
                let now_due = fx.effects.iter().zip(fx.starts.iter().chain(std::iter::repeat(&0.0))).filter(|&(_, &s)| s <= 0.0).map(|(&e, _)| e);
                let due: Vec<u32> = now_due.chain([fx.light]).filter(|&e| e != 0).collect();
                if let Some(ale) = ale.as_deref_mut() {
                    for e in due {
                        spawn_once(&mut commands, &mut game.0, ale, &mut images, &mut materials, e, at);
                    }
                }
                v.sound_queue.extend(fx.sounds.iter().map(|&(id, ..)| (id, 1.0)));
            }
            // the damage (FUN_00223160: a bare record, no hit part, no shooter's factors)
            let mut hit = damage::Hit::new(cfg.damage, "the charge", DAMAGE_TYPE).source(who);
            let amount = hit.land(v, &game.0, &difficulty, now);
            let push = (Vec3::new(velocity.x, cfg.knock_up, velocity.z)) * KNOCK_SCALE;
            let before = v.health;
            hurt(v, &game.0, amount, HURT_CHATTER, push, at.translation, -1);
            if log {
                println!("t={now:.2} {}: the charge hits {} at {:.2} m, {:.0} deg off his facing: {amount:.1} ({before:.1} -> {:.1} hp){}",
                         who_name(who, &game.0), unit_name(v), d2.sqrt(), d.normalize().dot(ahead).clamp(-1.0, 1.0).acos().to_degrees(),
                         v.health, if v.dead { ", killed: thrown" } else { "" });
            }
        }
        if hits == 0 {
            continue;
        }
        // his own, a hit each: the line through the chatter gate (its replay delay keeps a second
        // one quiet) and his type's sound (FUN_0010e7a0 with 100, positional: 2D for the player's
        // character); then the pending cost (0x13c474: set, not added)
        let sight = sight_list(std::iter::once(&*player).chain(squad.0.iter()));
        let here = player.position;
        let u = unit(&mut player, &mut squad, i);
        let sound = u.skill.data.as_ref().map_or(0, |d| d.hit_sound);
        let distance = u.position.distance(here);
        for _ in 0..hits {
            let said = skills::line(u, &game.0, &mut history, &mut reserve, i, &sight, distance, LINE_CHARGE);
            if sound != 0 {
                u.sound_queue.push((sound, 1.0));
            }
            if log {
                println!("t={:.2} {}: line {LINE_CHARGE:08x}: {said}", u.sim_time, unit_name(u));
            }
        }
        u.skill.pending = cfg.charge_cost;
        if log {
            println!("t={:.2} {}: {hits} charge hit(s): {} stamina taken with the next drain", u.sim_time, unit_name(u), cfg.charge_cost);
        }
    }
}

/// The Vengar view's state: whether the controlled character's vision mode is Vengar's (2: his
/// on sets it, char+0x644 and the controller's +0xd8, 0x13c032 / 0x13c03b), and the view's colour
/// fade (the view's +0xec object, FUN_000c6980).
#[derive(Resource, Default)]
struct VengarView {
    on: bool,
    /// the fade's level: 1 in the frame of a switch, falling by 1 / FADE_TIME a second
    fade: f32,
    /// the controlled character last frame (a hand-over isn't a switch: no fade)
    viewer: Option<Who>,
    /// the heat material's time, s: the game's runs from the start (`scroll`)
    clock: f32,
}

/// The camera that draws the heat figures over the main camera's picture (its child): the
/// game draws the living characters after the world pass in vision mode 2 (FUN_000e81e0, the
/// lists 0x41b690 / 0x41b518 that 0xe7ab2 puts them on), against the world's depth. Its own
/// picture starts clear and goes over the main camera's (premultiplied alpha), so the halo it
/// adds round them lands on the world. It draws the HUD too (`IsDefaultUiCamera`), over both.
#[derive(Component)]
struct HeatCamera;

/// A character's mesh drawn as a heat figure, and its own material meanwhile.
#[derive(Component)]
struct Heated(Handle<StandardMaterial>);

/// The heat figures' material (heat.wgsl).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct HeatMaterial {
    #[uniform(0)]
    heat: HeatParams,
}

impl HeatMaterial {
    /// One of the two looks (BRIGHT, DIM).
    fn new(look: Look) -> Self {
        let (m, e, power, [from, to]) = look;
        HeatMaterial { heat: HeatParams { middle: Vec4::new(m[0], m[1], m[2], 1.0), edge: Vec4::new(e[0], e[1], e[2], power),
                                          ramp: Vec4::new(from, to, SCROLL_MID, 0.0) } }
    }
}

impl Material for HeatMaterial {
    fn fragment_shader() -> ShaderRef {
        HEAT_SHADER.into()
    }
}

/// The uniforms (in a module of their own: the derives' own checks read as unused).
mod pass {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::{extract_component::ExtractComponent, render_resource::ShaderType}};

    /// What a camera's Vengar pass does this frame (vengar.wgsl `Pass`).
    #[derive(Component, Clone, Copy, Default, Debug, ExtractComponent, ShaderType)]
    pub struct VengarPass {
        /// 1: the world pass (the main camera while the view is Vengar's)
        pub world: f32,
        /// the colour fade's level
        pub fade: f32,
        /// the halo's weight a tap (the heat camera while the view is Vengar's)
        pub glow: f32,
        /// the view's width over its height
        pub aspect: f32,
    }

    /// A heat figure's colours (heat.wgsl `Heat`).
    #[derive(Clone, Copy, Debug, ShaderType)]
    pub struct HeatParams {
        pub middle: Vec4,
        pub edge: Vec4,
        pub ramp: Vec4,
    }
}

/// The heat camera under each main camera, and the main camera's pass settings.
fn heat_camera(mut commands: Commands, main: Query<(Entity, Option<&Children>, Has<VengarPass>), With<MainCamera>>,
               heat: Query<(), With<HeatCamera>>) {
    for (e, children, has_pass) in &main {
        if !has_pass {
            commands.entity(e).insert(VengarPass::default());
        }
        if children.is_some_and(|c| c.iter().any(|c| heat.contains(c))) {
            continue;
        }
        commands.spawn((
            HeatCamera,
            Camera3d { depth_load_op: Camera3dDepthLoadOp::Load, ..default() },
            Camera {
                order: 1,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                msaa_writeback: false,
                output_mode: CameraOutputMode::Write { blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING), clear_color: ClearColorConfig::None },
                ..default()
            },
            // (its own picture, not the main camera's: those of cameras alike are shared)
            CameraMainTextureUsages(CameraMainTextureUsages::default().0 | TextureUsages::COPY_DST),
            Tonemapping::None,
            RenderLayers::layer(HEAT_LAYER),
            IsDefaultUiCamera,
            Transform::default(),
            VengarPass::default(),
            ChildOf(e),
        ));
    }
}

/// The heat camera sees as the main camera does (the zoom of the scope, the follow camera's
/// field of view).
fn follow_projection(main: Query<&Projection, (With<MainCamera>, Without<HeatCamera>)>, mut heat: Query<(&mut Projection, &ChildOf), With<HeatCamera>>) {
    for (mut p, parent) in &mut heat {
        if let Ok(m) = main.get(parent.parent()) {
            *p = m.clone();
        }
    }
}

/// The view's state for this frame: on while the controlled character's Vengar is (his on and
/// off, and the death handler's off, switch it); the colour fade from each switch of the same
/// character's (FUN_000c6980 from his on 0x13c090, and from FUN_00126df0 at 0x126e58 when the view
/// was Vengar's), not at a hand-over (the demo's choice: what the game's view does when control
/// passes with Vengar on isn't traced). Each camera's pass settings follow.
fn view_state(time: Res<Time>, player: Res<Player>, mut view: ResMut<VengarView>, windows: Query<&Window, With<PrimaryWindow>>,
              mut passes: Query<(&mut VengarPass, Has<HeatCamera>)>) {
    let dt = frame_dt(&time);
    let on = skills::vengar_on(&player);
    let viewer = player.loaded.is_some().then(|| player.who());
    view.clock += dt;
    view.fade = (view.fade - dt / FADE_TIME).max(0.0);
    if view.viewer.is_some() && view.viewer == viewer && on != view.on {
        view.fade = 1.0;
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: the Vengar view {}, the red fade from 1", player.sim_time, unit_name(&player), if on { "on" } else { "off" });
        }
    }
    view.on = on;
    view.viewer = viewer;
    let aspect = windows.single().map(|w| w.width() / w.height().max(1.0)).unwrap_or(4.0 / 3.0);
    for (mut p, heat) in &mut passes {
        *p = VengarPass { world: (on && !heat) as u8 as f32, fade: view.fade, glow: if on && heat { GLOW } else { 0.0 }, aspect };
    }
}

/// One of the level's directional lights while the view is Vengar's, and what it was.
#[derive(Component)]
struct LevelLight(DirectionalLight);

/// One of the Vengar view's two lights (`vision_lights`).
#[derive(Component)]
struct VisionLight;

/// The world's lights in the Vengar view. The world's shaders take their two lights from the
/// constants FUN_000dcc10 uploads (c16-c19: a direction and a colour each; the indoor pair, the
/// lights marked h_e02aba1c, or the outdoor pair, h_1f416687, by the object's light-usage, #143),
/// and in a vision mode (its second argument, the view's vision mode not 0, 0x9f443-0x9f45a) it
/// uploads a fixed pair in place of either: white (1.0 at 0x40d5b8) from straight up and straight
/// down (the table 0x41b3a8, filled at 0xdd10e-0xdd1dc), the ambient left as it is. Floors are
/// lit, walls by the ambient alone: take08's sunlit tower goes dark and its shaded sand light,
/// take06's blue-lit gate dark. Here the level's directional lights are put out and the pair
/// takes their place, at VISION_LIGHT (play.rs's lighting: PI lux is the console's 1.0, in linear
/// terms), without shadows; the level's own come back at the off. The terrain takes the same pair
/// per vertex (`vision_terrain`); the scenery lit by the level's indoor pair takes it too, as its
/// shader swaps only the level's own lights while they're lit (lamps.wgsl `indoor_swap`).
fn vision_lights(mut commands: Commands, view: Res<VengarView>,
                 mut lights: Query<(Entity, &mut DirectionalLight, Option<&LevelLight>), Without<VisionLight>>,
                 pair: Query<Entity, With<VisionLight>>) {
    for (e, mut light, own) in &mut lights {
        match (view.on, own) {
            (true, None) => {
                commands.entity(e).insert(LevelLight(light.clone()));
                light.illuminance = 0.0;
                light.shadows_enabled = false;
            }
            (false, Some(own)) => {
                *light = own.0.clone();
                commands.entity(e).remove::<LevelLight>();
            }
            _ => {}
        }
    }
    if view.on && pair.is_empty() {
        for to in [Vec3::NEG_Y, Vec3::Y] {
            commands.spawn((VisionLight, Transform::default().looking_to(to, Vec3::Z), DirectionalLight {
                illuminance: std::f32::consts::PI * VISION_LIGHT.powf(2.2), color: Color::WHITE, shadows_enabled: false, ..default()
            }));
        }
    }
    if !view.on {
        for e in &pair {
            commands.entity(e).despawn();
        }
    }
}

/// A terrain layer while the view is Vengar's, and its own mesh and material meanwhile.
#[derive(Component)]
struct LevelTerrain(Handle<Mesh>, Handle<bf_viewer::level_scene::LevelMaterial>);

/// The terrain layers' Vengar-view copies made on this map (mesh, material), by their own mesh.
#[derive(Resource, Default)]
struct TerrainCopies(HashMap<AssetId<Mesh>, (Handle<Mesh>, Handle<bf_viewer::level_scene::LevelMaterial>)>);

/// The terrain in the Vengar view, lit by the vision pair (`vision_lights`) and its own ambient
/// (the Terrain's `<ambient>`), as the console lights its vertices, in the colours' gamma terms.
/// The game's terrain matches its takes lit by its own light pair and ambient (take08's sand,
/// lit by the ambient and a grazing fill alone, (28, 29, 32) against the take's (31, 31, 32);
/// take09's e01, which has no terrain pair, by the objects' pair: (21, 27, 29) against (19, 22,
/// 20)), not by the baked light map the demo draws it with (level_scene.rs `spawn_level`); in a
/// vision mode that pair is the vision pair. So each layer's mesh is copied with its vertex
/// colours made that light (ambient + VISION_LIGHT x the up-facing share of its normal, and as much
/// for the down-facing, the pair's other light; its mask kept in their alpha), drawn unlit, and
/// both are put back at the off.
#[allow(clippy::type_complexity)]
fn vision_terrain(mut commands: Commands, view: Res<VengarView>, map: Res<MapLevel>, mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<bf_viewer::level_scene::LevelMaterial>>,
                  mut layers: Query<(Entity, &Name, &mut Mesh3d, &mut MeshMaterial3d<bf_viewer::level_scene::LevelMaterial>, Option<&LevelTerrain>)>,
                  mut copies: ResMut<TerrainCopies>) {
    let ambient = map.0.as_ref().map_or([0.0; 3], |l| l.terrain_ambient);
    for (e, name, mut mesh, mut material, own) in &mut layers {
        if !name.as_str().starts_with("terrain layer") {
            continue;
        }
        match (view.on, own) {
            (true, None) => {
                if !copies.0.contains_key(&mesh.0.id()) {
                    let Some(mut m) = meshes.get(&mesh.0).cloned() else { continue };
                    let normals: Vec<[f32; 3]> = match m.attribute(Mesh::ATTRIBUTE_NORMAL) {
                        Some(VertexAttributeValues::Float32x3(n)) => n.clone(),
                        _ => vec![],
                    };
                    let mask: Vec<f32> = match m.attribute(Mesh::ATTRIBUTE_COLOR) {
                        Some(VertexAttributeValues::Float32x4(c)) => c.iter().map(|c| c[3]).collect(),
                        _ => vec![1.0; normals.len()],
                    };
                    let colors: Vec<[f32; 4]> = normals.iter().zip(&mask).map(|(n, &a)| {
                        let light = VISION_LIGHT * (n[1].max(0.0) + (-n[1]).max(0.0));
                        let lin = |c: f32| Color::srgb((c + light).min(1.0), 0.0, 0.0).to_linear().red;
                        [lin(ambient[0]), lin(ambient[1]), lin(ambient[2]), a]
                    }).collect();
                    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    let mut mat = materials.get(&material.0).cloned().unwrap_or_default();
                    mat.base.unlit = true;
                    mat.base.base_color = Color::WHITE;
                    copies.0.insert(mesh.0.id(), (meshes.add(m), materials.add(mat)));
                }
                let Some(lit) = copies.0.get(&mesh.0.id()).cloned() else { continue };
                commands.entity(e).insert(LevelTerrain(mesh.0.clone(), material.0.clone()));
                mesh.0 = lit.0;
                material.0 = lit.1;
            }
            (false, Some(own)) => {
                mesh.0 = own.0.clone();
                material.0 = own.1.clone();
                commands.entity(e).remove::<LevelTerrain>();
            }
            _ => {}
        }
    }
}

/// The characters as heat figures while the view is Vengar's: every living character's meshes
/// (not the unlit muzzle flash) go to the heat camera's layer with the bright look (the viewer
/// and his hostiles) or the dim one (his friends: the team table, as 0xa88ea-0xa890c reads it
/// through FUN_000a8520); the dead are drawn as the world is (0xe7acd, 0xe735d), and everyone
/// once the view is off.
#[allow(clippy::too_many_arguments)]
fn heat_figures(mut commands: Commands, view: Res<VengarView>, player: Res<Player>, squad: Res<Squad>, children: Query<&Children>,
                plain: Query<&MeshMaterial3d<StandardMaterial>, Without<Heated>>, heated: Query<(&Heated, &MeshMaterial3d<HeatMaterial>)>,
                standard: Res<Assets<StandardMaterial>>, mut assets: ResMut<Assets<HeatMaterial>>, mut looks: Local<Option<[Handle<HeatMaterial>; 2]>>) {
    let [bright, dim] = looks.get_or_insert_with(|| [assets.add(HeatMaterial::new(BRIGHT)), assets.add(HeatMaterial::new(DIM))]).clone();
    // the pulse: both looks' texture offset now (the one material, k aside)
    if view.on {
        for look in [&bright, &dim] {
            if let Some(m) = assets.get_mut(look) {
                m.heat.ramp.z = scroll(view.clock);
            }
        }
    }
    let viewer = player.who();
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let Some(l) = &u.loaded else { continue };
        let want = (view.on && !u.dead).then(|| if u.who() == viewer || damage::hostile(u.team, player.team) { &bright } else { &dim });
        for e in children.iter_descendants(l.root) {
            match (want, plain.get(e), heated.get(e)) {
                (Some(m), Ok(own), _) if !standard.get(&own.0).is_some_and(|s| s.unlit) => {
                    commands.entity(e).insert((Heated(own.0.clone()), MeshMaterial3d(m.clone()), RenderLayers::layer(HEAT_LAYER)))
                        .remove::<MeshMaterial3d<StandardMaterial>>();
                }
                (Some(m), _, Ok((_, now))) if now.0 != *m => {
                    commands.entity(e).insert(MeshMaterial3d(m.clone()));
                }
                (None, _, Ok((own, _))) => {
                    commands.entity(e).insert(MeshMaterial3d(own.0.clone())).remove::<(Heated, MeshMaterial3d<HeatMaterial>, RenderLayers)>();
                }
                _ => {}
            }
        }
    }
}

/// The Vengar pass in the render graph: after the tone mapping (none here), before the HUD.
struct PassPlugin;

impl Plugin for PassPlugin {
    fn build(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.add_render_graph_node::<ViewNodeRunner<VengarNode>>(Core3d, VengarLabel)
            .add_render_graph_edges(Core3d, (Node3d::Tonemapping, VengarLabel, Node3d::EndMainPassPostProcessing));
    }

    fn finish(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render.init_resource::<PassPipeline>();
    }
}

/// The pass's node label (the scope views' pass comes after it: play_scope_view.rs).
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(super) struct VengarLabel;

/// The pass (vengar.wgsl) on a camera whose settings ask for any of it.
#[derive(Default)]
struct VengarNode;

impl ViewNode for VengarNode {
    type ViewQuery = (&'static ViewTarget, &'static VengarPass, &'static DynamicUniformIndex<VengarPass>);

    fn run(&self, _graph: &mut RenderGraphContext, context: &mut RenderContext, (target, pass, index): QueryItem<Self::ViewQuery>,
           world: &World) -> Result<(), NodeRunError> {
        if pass.world <= 0.0 && pass.fade <= 0.0 && pass.glow <= 0.0 {
            return Ok(());
        }
        let pipes = world.resource::<PassPipeline>();
        let Some(pipeline) = world.resource::<PipelineCache>().get_render_pipeline(pipes.pipeline) else { return Ok(()) };
        let Some(uniforms) = world.resource::<ComponentUniforms<VengarPass>>().uniforms().binding() else { return Ok(()) };
        let post = target.post_process_write();
        let bind_group = context.render_device().create_bind_group("vengar_pass", &pipes.layout,
                                                                   &BindGroupEntries::sequential((post.source, &pipes.sampler, uniforms.clone())));
        let mut draw = context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("vengar_pass"),
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

/// The pass's pipeline: the picture, a linear clamped sampler and the settings.
#[derive(Resource)]
struct PassPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline: CachedRenderPipelineId,
}

impl FromWorld for PassPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>().clone();
        let layout = device.create_bind_group_layout("vengar_pass_layout", &BindGroupLayoutEntries::sequential(ShaderStages::FRAGMENT, (
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
            uniform_buffer::<VengarPass>(true),
        )));
        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: bevy::render::render_resource::FilterMode::Linear,
            min_filter: bevy::render::render_resource::FilterMode::Linear,
            ..default()
        });
        let pipeline = world.resource_mut::<PipelineCache>().queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("vengar_pass".into()),
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
        PassPipeline { layout, sampler, pipeline }
    }
}
