//! A runtime for the game's ALE particle effects (`Game::effects`, common/effects-common.ale),
//! driven by the nodes' own parameters: the power-ups' spinning icons, the slide puff.
//!
//! Each effect pairs emitters with appearances (see `bf::ale`):
//!  emitter     lifespan (s; huge = forever), transform (offset and turn of its frame), initial
//!              particles, rate (/s), particle life (s), speed (m/s); its shape by class:
//!              cube (width / height / depth, spread about +y), cone (radius and spread about
//!              +y), sphere (radius, outward)
//!  appearance  colour, alpha, size, width factor, aspect and rotation over a particle's life;
//!              a texture, its blend (5,2 added to the picture, 5,6 alpha), and "perp": the
//!              quad lies flat in the emitter's frame (normal +y), turned by the appearance's
//!              transform at the effect's time it was born (the icons' particles fan out round
//!              the turn); otherwise it faces the camera.
//! Curves with keys run on the emitter's time in seconds (flagged ones repeat); per-particle
//! floats run over the particle's life (0-1). An emitter flagged "attached" (h_e2999ffd: the
//! projectiles' bolts) keeps its particles in its own frame, so they travel with it; one with no
//! rate and no initial count fires a single particle when it starts (the guns' tracer).
//! Motion-blurred appearances (h_fe7c2071) and stretched ones turn along their particle's
//! motion. Two appearances are drawn: the basic one (class h_111035aa, a quad per particle)
//! and the beam (class h_1f55f13e: a ribbon through the emitter's live particles in the order
//! they were born, facing the camera; its own colour h_048767e8, alpha h_04fc9016 and width
//! h_fba203b8 over a particle's life, texture h_1ba23359, blend h_1506eb6c): the cutter's
//! trail, the sniper's beam.
//!
//! A "light_" effect (an effect type's light-effect: light_explosion, light_phosphor ...) lights
//! the scene instead of drawing: each of its particles is a point light whose reach is the
//! appearance's size (m) and whose strength is its colour x alpha x size, as the hand-made DNA
//! light in play_fx.rs reads light_powerup_pill.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    math::Affine2,
    pbr::NotShadowCaster,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::bf::ale::Node;
use crate::bf::character::Game;
use crate::bf::hash::h;

// emitter parameters
const LIFESPAN: u32 = 0xF27F_DE7D;
const TRANSFORM: u32 = 0xE13A_59A1;
const INITIAL: u32 = 0x0F9A_9D52;
/// For an emitter with no rate and no initial count, read at its start as a burst:
/// exp-lrg-flash's 4.1 (the recording's first flash frame is a wide haze, not one flare). Only
/// for the grenades' effects (`Compiled::recorded`): elsewhere it's a curve over the emitter's
/// time on ~40 emitters (laser hit rings, tracers ...), more likely an emit count; those keep
/// one particle. (It is Freelancer's Emitter_EmitCount: its hash is the game's hash of that
/// name. Keyed over the emitter's time on smoke-grenade-flsh - 9 keys of 10.6-13.7 over
/// 0.09-0.98 s - where the Gas recording's flash flickers for ~1 s; the demo's is the one
/// burst. How the keys emit isn't settled: exp-lrg-flash and sonic_grenade are keyed too.)
const BURST: u32 = 0xE722_1F95;
const RATE: u32 = 0x023C_350C;
const LIFE: u32 = 0x0A63_5880;
const SPEED: u32 = 0x0AB1_80C5;
const CUBE: [u32; 3] = [0xFD22_A11C, 0xEFB0_F8CF, 0x14B7_4EB2];
const CUBE_SPREAD: [u32; 2] = [0xE282_006B, 0x08C7_217F];
const CONE_RADIUS: [u32; 2] = [0x1C73_D16B, 0xF636_F07F];
const CONE_SPREAD: [u32; 2] = [0x100E_7F55, 0xFA4B_5E41];
const SPHERE_RADIUS: [u32; 2] = [0xF231_FF60, 0x1874_DE74];
const CLASS_CONE: u32 = 0xE1F4_0411;
const CLASS_SPHERE: u32 = 0xF940_F7F3;
const ATTACHED: u32 = 0xE299_9FFD;
const MOTION_BLUR: u32 = 0xFE7C_2071;
/// The basic particle appearance (billboard / perp quad).
const CLASS_APPEARANCE: u32 = 0x1110_35AA;
/// The beam appearance (a ribbon through the particles) and its parameters.
const CLASS_BEAM: u32 = 0x1F55_F13E;
const BEAM_COLOR: u32 = 0x0487_67E8;
const BEAM_ALPHA: u32 = 0x04FC_9016;
const BEAM_WIDTH: u32 = 0xFBA2_03B8;
const BEAM_TEXTURE: u32 = 0x1BA2_3359;
const BEAM_BLEND: u32 = 0x1506_EB6C;
// appearance parameters
const COLOR: u32 = 0xF89C_93A4;
const ALPHA: u32 = 0xF8E7_645A;
const SIZE: u32 = 0x00FA_C601;
const WIDTH: u32 = 0x04B5_FC1B;
const ASPECT: u32 = 0x0DB2_CC8D;
const ROTATE: u32 = 0xF24D_7541;
const PERP: u32 = 0x0D64_5074;
const TEXTURE: u32 = 0xF736_F94E;
const BLEND: u32 = 0x1DAE_A8C0;
/// An animated texture's frame over a particle's life (0-1 of its frames). Read from the data's
/// values: a ramp (Exp5 on exp-lrg-add 0.61 -> 1.0, the fire puffs 0.21 -> 0.98, campfires 0 ->
/// 1) runs the flipbook over the particle's life; a constant above 0 (the muzzle flashes'
/// 0.60, exp-lrg-fire's 0.70) holds that one frame; a constant 0 (the shield icon's arcb)
/// leaves the texture playing at its own rate. An inference (Freelancer's TexFrame reads the
/// same way), applied to the grenades' effects only (`Compiled::recorded`): played at 30 fps
/// and wrapped, exp-lrg-add went back to its bright first frames half a second into the Frag's
/// blast, which the recording doesn't show. Other effects keep the 30 fps loop.
const TEX_FRAME: u32 = 0x1865_7E4A;
const BLEND_ADD: (u32, u32) = (5, 2);
/// The effect appearance (class h_0ec77ea0): each particle of its emitter carries an effect of
/// its own, named by h_0ec7a290 (stun_grenade_master's node "stun_grenade_master" -> the
/// effect "stun_grenade": the Energy grenade's bolts). Only for the grenades' effects
/// (`Compiled::recorded`); the library has other effect appearances, unchecked.
const CLASS_SPAWNER: u32 = 0x0EC7_7EA0;
const SPAWN_EFFECT: u32 = 0x0EC7_A290;
/// The missing emitters given a stand-in (`missing_emitter`): stun_grenade_master's
/// h_ed10c55f (stun_grenade_init.app) and h_f48dc74d (stun_grenade_init#1.app).
const STAND_IN_EMITTERS: [u32; 2] = [0xED10_C55F, 0xF48D_C74D];

/// Perp quads lie flat in the emitter's frame, i.e. face along the emitter's axis - the
/// direction a cone emitter throws them (the Sonic's ring, the laser hits' rings, a bolt's
/// cross-section). A sphere emitter throws them every way: from one moving out at this speed
/// (m/s) or more, a perp quad faces out along its own direction, so the Frag's exp-fire-add
/// (0.48 m/s, radius 1 -> 3.3 m) is a round shell as in the recording (frag/a 0700-0730),
/// not a stack of flat discs seen edge-on. The power-ups' icons (sphere emitters at 0.02-0.16
/// m/s, verified flat) keep the emitter's frame. The threshold is the demo's: the data has no
/// flag for it. Only for the grenades' effects (`Compiled::recorded`).
const PERP_RADIAL_SPEED: f32 = 0.3;
/// Air fields, linked to an appearance by the effect's pair list (appearance -> field, as
/// Freelancer's ALE links them; FxAirField, AirField_Magnitude and AirField_Approach are the
/// game's hashes of Freelancer's names). One pulls its particles' velocity toward a wind of
/// Magnitude m/s along the field's +y (turned by its transform's rotation over the effect's
/// time), Approach of the way once each 1/APPROACH_FPS s.
///
/// Run only for the fields in AIR_FIELDS, the ones checked against a recording; every other
/// field (the Sonic's sonic_grenade_air.fld, the Frag's exp-lrg-air, the gravity fields
/// FxGravityField h_e644c021 of the shrapnels and exp-lrg-dirt, turbulence fields
/// FxTurbulenceField h_0b72ea10) is left off until checked against its own capture (#101).
/// Not done: a field node's own life (h_f27fde7d: gas-grenade.fld's 12 s) is ignored, the field
/// runs as long as its particles do; the wind is in the world's frame (the blasts are placed
/// upright, and a trail's spinning grenade would spin it), while an attached particle's
/// velocity is in its effect's frame - no attached particle has an allowed field yet.
const CLASS_AIR_FIELD: u32 = 0xE625_323F;
const AIR_MAGNITUDE: u32 = 0xE5E3_524C;
const AIR_APPROACH: u32 = 0x1042_3CEB;
/// The air fields that run (node names): the Gas cloud's two (gas-grenade.fld: a wind of about
/// +-1 m/s swinging round; gas-grenade2.fld: 0.26 m/s up; checked against the Gas recording)
/// and grenade_trail_rise (0.8 m/s up: the tester saw the trail rise the right way, thinner and
/// more vertical than the recordings' - left on, a partial match).
const AIR_FIELDS: [&str; 3] = ["gas-grenade.fld#1.fld", "gas-grenade2.fld#1.fld", "grenade_trail_rise"];
/// How often an air field's Approach is applied (per second): once per 30 fps game frame. Not
/// in the data: fitted to the Gas recording, whose cloud is ~6 m wide and ~4 m tall and swallows
/// Tex although gas-grenade_Cone.emt#1.emt throws its puffs out at 4-6.5 m/s for 2.3-4.4 s.
/// Captured from 12 m (scratchpad gas76/fps*): at 60 every puff stopped within ~0.3 m, a
/// 6.5 x 4 m cloud of billboards all centred in Tex, who stood out in front of it; at 30 it's
/// ~6.5 x 5 m and veils him as the recording does; at 15, ~9 x 6 m; at 8, ~13 m. An inference.
const APPROACH_FPS: f32 = 30.0;
/// An upright streak: a camera-facing appearance (not perp, not motion-blurred) whose width
/// factor (WIDTH) stays under this part of its smallest height factor (ASPECT, counted as 1.0 at
/// most) all its life, with a constant Rotate. It is drawn with no random roll - roll 0 is
/// upright on the screen, not in the world: it faces the camera, so under a steep camera pitch
/// it lies back with the view - turned only by its Rotate, and rides its appearance
/// transform's offset over its life (`Pair::rise`). The one
/// grenade appearance it picks out is phosphor_grenade_init_spike.app (width 0.05 -> 0.30 of a
/// size that peaks at 5.9 m): the Light recording's tall thin vertical beam (lg 0413-0487,
/// 1181-1289), where the random roll drew a fan of rays. The threshold is the demo's (the data
/// has no flag for it); only for the grenades' effects (`Compiled::recorded`).
const UPRIGHT_WIDTH: f32 = 0.5;
/// A spark: a perp appearance born longer than this (ASPECT at birth) on a cone emitter that
/// throws its particles out at PERP_RADIAL_SPEED or more. It is drawn as a camera-facing streak
/// along its motion (as a motion-blurred one is), not as a quad lying flat in the emitter's
/// frame. The one grenade appearance it picks out is phosphor_grenade-shrap.rect.app (spark.tga,
/// aspect 2.2 -> 0.25, ~100/s at 4-6 m/s within 49 degrees of up): the Light recording's
/// starburst of rays fanning up from the flare at ignition and the sparks round its base after
/// (lg 0397-0411, 1073-1085, 1200), which lying flat drew as short horizontal lines. The Frag's,
/// Gas's, Sonic's and Sentry's perp appearances are born square or squat (aspect 0.04-1.01) and
/// keep lying flat. The threshold is the demo's; only for the grenades' effects
/// (`Compiled::recorded`).
const SPARK_ASPECT: f32 = 1.5;
/// Materials per appearance: its colour and alpha at this many points of a particle's life.
const STEPS: usize = 12;
/// Particles alive at once, over all effects.
const MAX_PARTICLES: usize = 3000;
/// A light particle's lumens per unit of colour x alpha x size: the tuning play_fx.rs uses for
/// the DNA's light_powerup_pill (fitted to the capture's brightening round the DNA).
pub const LIGHT_LUMENS: f32 = 30_000.0;
/// Point lights' strength under the console look (see level_scene::POINT_LIGHT_SCALE).
const LIGHT_SCALE: f32 = crate::level_scene::POINT_LIGHT_SCALE;

/// One emitter and the appearance its particles take.
pub struct Pair {
    emitter: Node,
    app: Node,
    /// materials by life step, then by flipbook frame (one frame for a still texture)
    steps: Vec<Vec<Handle<StandardMaterial>>>,
    /// an animated texture's frames per second
    fps: f32,
    /// how the flipbook's frame is picked (see TEX_FRAME)
    frame: FrameMode,
    perp: bool,
    /// particles stay in the emitter's frame (move with it)
    attached: bool,
    /// turned along their motion
    streak: bool,
    /// an upright streak (see UPRIGHT_WIDTH): no random roll, and it rises by its appearance
    /// transform's offset over its age in seconds, up to this time (the offset's last key: the
    /// spike's 0 -> 2.26 m up over 0.92 s). The offset read over the particle's age rather than
    /// the effect's time is an inference: over the effect's time (repeating every 0.92 s) the
    /// whole beam would rise and drop back once every 0.92 s, while the recording's beam top
    /// flickers by ~25% about every 0.3 s with no slow cycle (the Light recording, frames
    /// 1181-1289: the second flare's beam top under a still camera)
    rise: Option<f32>,
    /// a beam appearance: the ribbon's material (vertex colours carry colour x alpha)
    beam: Option<Handle<StandardMaterial>>,
    /// a "light_" effect's pair: its particles are point lights
    light: bool,
    /// the fields its appearance is linked to (see CLASS_AIR_FIELD)
    fields: Vec<Node>,
    /// an effect appearance's pair (CLASS_SPAWNER): the effect each particle carries
    child: Option<Arc<Compiled>>,
}

/// How a pair's flipbook frame is picked (see TEX_FRAME).
#[derive(Clone, Copy, PartialEq)]
enum FrameMode {
    /// at the texture's own rate, round and round
    Play,
    /// this part of the frames (0-1), all its life
    Hold(f32),
    /// along the appearance's TEX_FRAME curve over the particle's life
    Life,
}

impl FrameMode {
    fn of(app: &Node) -> Self {
        match app.params.get(&TEX_FRAME) {
            Some(crate::bf::ale::Value::Floats(items)) => match items.first() {
                Some((_, _, keys)) if keys.len() > 1 => FrameMode::Life,
                Some((_, _, keys)) => match keys.first() {
                    Some(&(_, v)) if v > 0.0 => FrameMode::Hold(v),
                    _ => FrameMode::Play,
                },
                None => FrameMode::Play,
            },
            _ => FrameMode::Play,
        }
    }
}

/// An effect ready to run.
pub struct Compiled {
    pub name: String,
    pairs: Vec<Pair>,
    /// Compiled with the rules checked against the Frag recording only (the grenades' blast and
    /// trail effects): the flipbook frame curve (TEX_FRAME), the opening burst (BURST) and
    /// outward-facing perp quads (PERP_RADIAL_SPEED). Every other effect plays as before them;
    /// they may hold for those too, once checked against their own captures.
    pub recorded: bool,
}

impl Compiled {
    /// The materials its particles draw with (the first of each pair's), for `warm_up`.
    pub fn materials(&self) -> Vec<Handle<StandardMaterial>> {
        self.pairs.iter().flat_map(|p| {
            let own = p.beam.clone().or_else(|| p.steps.first().and_then(|s| s.first()).cloned());
            own.into_iter().chain(p.child.iter().flat_map(|c| c.materials()))
        }).collect()
    }

    /// The effects its particles carry (its effect appearances' effects, CLASS_SPAWNER).
    pub fn children(&self) -> Vec<Arc<Compiled>> {
        self.pairs.iter().filter_map(|p| p.child.clone()).collect()
    }

    /// How long a one-shot run of the effect lasts: its longest emitter plus its longest-lived
    /// particles (finite emitters only; a few seconds otherwise).
    pub fn duration(&self) -> f32 {
        self.pairs.iter().map(|p| {
            let span = p.emitter.float(LIFESPAN).unwrap_or(1.0);
            let span = if span > 100.0 { 3.0 } else { span };
            span + p.emitter.curve(LIFE, 0.0, 0.0).unwrap_or(1.0).max(0.0)
        }).fold(0.0, f32::max)
    }
}

/// The quad and the effects compiled so far (by name hash).
#[derive(Resource)]
pub struct AleAssets {
    quad: Handle<Mesh>,
    /// by (name hash, `recorded`)
    effects: HashMap<(u32, bool), Option<Arc<Compiled>>>,
    textures: HashMap<(u32, bool), Option<Handle<Image>>>,
}

/// Seconds the effects advance this frame, if the app steps time itself (fixed-step captures);
/// otherwise real time.
#[derive(Resource)]
pub struct AleClock(pub f32);

/// A running effect at its entity's place. Inactive, it makes no new particles (those out
/// finish their lives).
#[derive(Component)]
pub struct AleEffect {
    pub fx: Arc<Compiled>,
    pub active: bool,
    /// the user parameter (0-1) that picks between a curve's variants
    pub sp: f32,
    /// run each emitter once (its lifespan), not over and over
    pub once: bool,
    placed: bool,
    /// where the effect was last frame (new particles spread from there)
    prev: Option<(Vec3, Quat)>,
    /// per pair: the beam ribbon drawing this effect's particles (beam pairs)
    strips: Vec<Option<Entity>>,
    t: f32,
    acc: Vec<f32>,
    started: bool,
    rng: u32,
}

#[derive(Component)]
struct AleParticle {
    fx: Arc<Compiled>,
    pair: usize,
    age: f32,
    life: f32,
    vel: Vec3,
    sp: f32,
    /// the effect's time when it was born (fields' curves run on the effect's time)
    born: f32,
    roll: f32,
    step: usize,
    /// perp quads: the plane they lie in (the effect's and the emitter's turn, then the
    /// appearance's turn at the effect's time when the particle was born: born over the
    /// appearance's spin, they fan out); for attached particles, in the effect's frame
    frame: Quat,
    /// attached particles: the effect they move with, and their place and motion in its frame
    owner: Option<Entity>,
    local: Vec3,
    /// the effect's turn last seen
    turn: Quat,
    /// the effect that made it, and the order it was born in (beam ribbons join particles so)
    source: Entity,
    seq: u64,
}

/// A beam ribbon: the live particles of one beam pair of one effect, joined.
#[derive(Component)]
struct BeamStrip {
    source: Entity,
    pair: usize,
    fx: Arc<Compiled>,
    mesh: Handle<Mesh>,
    age: f32,
}

#[derive(Resource, Default)]
struct Seq(u64);

#[derive(Resource, Default)]
struct Count(usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<Count>().init_resource::<Seq>()
        .add_systems(PostUpdate, (emit, animate, beams).chain().before(TransformSystem::TransformPropagate))
        .add_systems(Update, end_warm_up);
}

/// A quad drawn (out of sight) for its material's render pipeline to be built before an effect
/// first needs it; gone after this many frames.
#[derive(Component)]
struct WarmUp(u32);
/// Frames the warm-up quads stay (the captures' 60 warm-up frames, and some).
const WARM_UP_FRAMES: u32 = 90;

/// Draw each of an effect's materials once, far below the ground and never culled, so their
/// render pipelines are built while the map loads. Bevy builds a pipeline the first time
/// something needs it, on other threads, and draws nothing with it until it's ready: a blast's
/// first additive layers could go missing for its first half second when the machine was
/// busy (the reference agent's run beside two other instances).
pub fn warm_up(commands: &mut Commands, assets: &AleAssets, fx: &Compiled) {
    for m in fx.materials() {
        commands.spawn((Mesh3d(assets.quad.clone()), MeshMaterial3d(m), NotShadowCaster, bevy::render::view::NoFrustumCulling,
                        Transform::from_xyz(0.0, -500.0, 0.0).with_scale(Vec3::splat(0.01)), Visibility::default(), WarmUp(WARM_UP_FRAMES)));
    }
}

fn end_warm_up(mut commands: Commands, mut quads: Query<(Entity, &mut WarmUp)>) {
    for (e, mut w) in &mut quads {
        w.0 = w.0.saturating_sub(1);
        if w.0 == 0 {
            commands.entity(e).despawn();
        }
    }
}

impl AleAssets {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        AleAssets { quad: meshes.add(Rectangle::new(1.0, 1.0)), effects: HashMap::new(), textures: HashMap::new() }
    }

    /// An effect compiled already (`load`).
    pub fn cached(&mut self, effect: u32) -> Option<Arc<Compiled>> {
        self.effects.get(&(effect, false)).cloned().flatten()
    }

    /// An effect compiled already with the recorded rules (`load_recorded`).
    pub fn cached_recorded(&mut self, effect: u32) -> Option<Arc<Compiled>> {
        self.effects.get(&(effect, true)).cloned().flatten()
    }

    /// Compile an effect of the library by name hash (cached); None if the library lacks it.
    pub fn load(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                effect: u32) -> Option<Arc<Compiled>> {
        self.load_as(game, images, materials, effect, false)
    }

    /// `load`, with the rules checked against the Frag recording only (see `Compiled::recorded`):
    /// for the grenades' blast and trail effects.
    pub fn load_recorded(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                         effect: u32) -> Option<Arc<Compiled>> {
        self.load_as(game, images, materials, effect, true)
    }

    fn load_as(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
               effect: u32, recorded: bool) -> Option<Arc<Compiled>> {
        if let Some(c) = self.effects.get(&(effect, recorded)) {
            return c.clone();
        }
        // (an effect that carries itself, through its effect appearances, carries nothing)
        self.effects.insert((effect, recorded), None);
        let compiled = self.compile(game, images, materials, effect, recorded).map(Arc::new);
        if compiled.is_none() {
            warn!("ALE effect h_{effect:08x}: not in the library, or nothing it draws is supported");
        }
        self.effects.insert((effect, recorded), compiled.clone());
        compiled
    }

    fn compile(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
               effect: u32, recorded: bool) -> Option<Compiled> {
        let e = game.effects.effects.get(&effect)?.clone();
        let node = |game: &Game, i: u32| e.refs.iter().find(|r| r.3 == i).and_then(|r| game.effects.nodes.get(&r.1)).cloned();
        let node_name = |i: u32| e.refs.iter().find(|r| r.3 == i).map_or(0, |r| r.1);
        let mut pairs = vec![];
        for &(em, ap) in &e.pairs {
            let Some(app) = node(game, ap) else { continue };
            let emitter = match node(game, em) {
                Some(n) => n,
                None if recorded && STAND_IN_EMITTERS.contains(&node_name(em)) => missing_emitter(&app),
                None => continue,
            };
            if app.class == CLASS_SPAWNER {
                if !recorded {
                    continue;
                }
                let child = app.string(SPAWN_EFFECT).filter(|n| !n.is_empty())
                    .and_then(|n| self.load_as(game, images, materials, h(n), recorded));
                if let Some(child) = child {
                    pairs.push(Pair { perp: false, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![], fps: 0.0,
                                      frame: FrameMode::Play, beam: None, light: false, child: Some(child), fields: vec![], rise: None });
                }
                continue;
            }
            if app.class == CLASS_BEAM {
                let additive = app.pair(BEAM_BLEND).unwrap_or(BLEND_ADD) == BLEND_ADD;
                let name = app.string(BEAM_TEXTURE).map(h).unwrap_or(0);
                // an animated texture (the Energy's bolts: "ARCb", a 4 x 4 sheet of arcs running
                // down its cells at 30 fps): one material per frame, the ribbon stepping through
                // them over its life (`beams`). Grenade effects only; elsewhere a beam's
                // flipbook name finds no texture, as before
                let book = if recorded { game.flipbooks.get(&name).cloned().filter(|b| !b.frames.is_empty()) } else { None };
                let texture = self.texture(game, images, book.as_ref().map_or(name, |b| b.texture), additive);
                let frames: Vec<Affine2> = match &book {
                    Some(b) => b.frames.iter()
                        .map(|&[u0, v0, u1, v1]| Affine2::from_scale_angle_translation(Vec2::new(u1 - u0, v1 - v0), 0.0, Vec2::new(u0, v0))).collect(),
                    None => vec![Affine2::IDENTITY],
                };
                let mats: Vec<Handle<StandardMaterial>> = frames.into_iter().map(|uv| materials.add(StandardMaterial {
                    base_color: Color::WHITE, base_color_texture: texture.clone(), uv_transform: uv, unlit: true, double_sided: true, cull_mode: None,
                    fog_enabled: false, alpha_mode: if additive { AlphaMode::Add } else { AlphaMode::Blend }, ..default()
                })).collect();
                let material = mats[0].clone();
                let fps = book.map_or(0.0, |b| b.fps);
                pairs.push(Pair { perp: false, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![mats], fps,
                                  frame: FrameMode::Play, beam: Some(material), light: false, child: None, fields: vec![], rise: None });
                continue;
            }
            if app.class != CLASS_APPEARANCE {
                continue;
            }
            if e.name.to_ascii_lowercase().starts_with("light_") {
                pairs.push(Pair { perp: false, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![], fps: 0.0,
                                  frame: FrameMode::Play, beam: None, light: true, child: None, fields: vec![], rise: None });
                continue;
            }
            let blend = app.pair(BLEND).unwrap_or(BLEND_ADD);
            let additive = blend == BLEND_ADD;
            // a texture, or an animated one ("arcb": a 4 x 4 sheet at 30 fps)
            let name = app.string(TEXTURE).map(h).unwrap_or(0);
            let book = game.flipbooks.get(&name).cloned();
            let texture = self.texture(game, images, book.as_ref().map_or(name, |b| b.texture), additive);
            if texture.is_none() {
                warn!("{}: texture {:?} missing", e.name, app.string(TEXTURE));
            }
            let frames: Vec<Affine2> = match &book {
                Some(b) if !b.frames.is_empty() => b.frames.iter()
                    .map(|&[u0, v0, u1, v1]| Affine2::from_scale_angle_translation(Vec2::new(u1 - u0, v1 - v0), 0.0, Vec2::new(u0, v0))).collect(),
                _ => vec![Affine2::IDENTITY],
            };
            // The Xbox adds sprites to the picture's stored (gamma) values; taking an added
            // sprite's texels and colour as linear amounts adds about what the console did
            // (see play_fx.rs).
            let steps = (0..STEPS).map(|i| {
                let k = (i as f32 + 0.5) / STEPS as f32;
                let [r, g, b] = app.color(COLOR, 0.0, k).unwrap_or([1.0; 3]);
                let a = app.floats(ALPHA, 0.0, k).unwrap_or(1.0).clamp(0.0, 1.0);
                frames.iter().map(|&uv| materials.add(StandardMaterial {
                    base_color: if additive { Color::linear_rgba(r, g, b, a) } else { Color::srgba(r, g, b, a) },
                    base_color_texture: texture.clone(), uv_transform: uv, unlit: true, double_sided: true, cull_mode: None,
                    fog_enabled: false, alpha_mode: if additive { AlphaMode::Add } else { AlphaMode::Blend }, ..default()
                })).collect()
            }).collect();
            let streak = app.flag(MOTION_BLUR);
            let frame = if recorded { FrameMode::of(&app) } else { FrameMode::Play };
            // the air fields linked to this appearance (pairs appearance -> field) that run (see
            // AIR_FIELDS; the grenades' effects only)
            let fields = e.pairs.iter().filter(|p| recorded && p.0 == ap).filter_map(|p| node(game, p.1))
                .filter(|n| n.class == CLASS_AIR_FIELD && AIR_FIELDS.contains(&n.name.as_str())).collect();
            let rise = (recorded && !app.flag(PERP) && !streak && upright(&app)).then(|| offset_end(&app));
            // a spark (see SPARK_ASPECT): drawn as a streak along its motion, not a perp quad
            let spark = recorded && app.flag(PERP) && emitter.class == CLASS_CONE
                && emitter.curve(SPEED, 0.0, 0.0).unwrap_or(0.0) >= PERP_RADIAL_SPEED && app.floats(ASPECT, 0.0, 0.0).unwrap_or(1.0) > SPARK_ASPECT;
            let (perp, streak) = if spark { (false, true) } else { (app.flag(PERP), streak) };
            pairs.push(Pair { perp, attached: emitter.flag(ATTACHED), streak, emitter, app, steps, fps: book.map_or(0.0, |b| b.fps), frame,
                              beam: None, light: false, fields, child: None, rise });
        }
        (!pairs.is_empty()).then_some(Compiled { name: e.name, pairs, recorded })
    }

    fn texture(&mut self, game: &mut Game, images: &mut Assets<Image>, id: u32, linear: bool) -> Option<Handle<Image>> {
        self.textures.entry((id, linear)).or_insert_with(|| {
            let (w, hgt, px) = game.texture_rgba(id)?;
            let format = if linear { TextureFormat::Rgba8Unorm } else { TextureFormat::Rgba8UnormSrgb };
            Some(images.add(Image::new(Extent3d { width: w, height: hgt, depth_or_array_layers: 1 }, TextureDimension::D2, px,
                                       format, RenderAssetUsages::default())))
        }).clone()
    }
}

impl AleEffect {
    pub fn new(fx: Arc<Compiled>, sp: f32, seed: u32) -> Self {
        let n = fx.pairs.len();
        AleEffect { fx, active: true, sp, once: false, placed: false, prev: None, strips: vec![None; n], t: 0.0, acc: vec![0.0; n], started: false, rng: seed | 1 }
    }

    /// Run the effect once (a hit, a shot's tracer): each emitter for its lifespan.
    pub fn once(fx: Arc<Compiled>, sp: f32, seed: u32) -> Self {
        AleEffect { once: true, ..Self::new(fx, sp, seed) }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// A stand-in for an emitter the node library doesn't have (stun_grenade_master's h_ed10c55f
/// and h_f48dc74d, the emitters of stun_grenade_init.app and stun_grenade_init#1.app): one
/// particle at the effect's middle when it starts, living the appearance's own lifespan
/// (0.6 s, 0.45 s). A guess: the recording's cyan flash and ground wash, at the middle,
/// strongest at +0.17-0.28 s and gone by ~0.45 s, fit it. Only those two (STAND_IN_EMITTERS):
/// other grenade effects miss emitters too (exp-lrg-dirt's h_f6c66e90 in the Frag's and the
/// Roller's blast, exp-mine-dirt's h_ecb22a58 in the Sentry's), and drew nothing before; a
/// stand-in there would add a layer the Frag recording wasn't checked with.
fn missing_emitter(app: &Node) -> Node {
    use crate::bf::ale::{Curve, Value};
    let life = app.float(LIFESPAN).unwrap_or(1.0);
    Node { class: 0, name: format!("{} (its emitter is missing)", app.name), params: HashMap::from([
        (LIFESPAN, Value::Float(life)),
        (LIFE, Value::Curve(Curve(vec![(0.0, life, 0, vec![])]))),
    ]) }
}

/// Every key value of a float animation (all its sparam items).
fn float_keys(app: &Node, p: u32) -> Vec<f32> {
    match app.params.get(&p) {
        Some(crate::bf::ale::Value::Floats(items)) => items.iter().flat_map(|(_, _, keys)| keys.iter().map(|k| k.1)).collect(),
        _ => vec![],
    }
}

/// An upright streak (see UPRIGHT_WIDTH): its width factor under UPRIGHT_WIDTH of its smallest
/// height factor, that capped at 1.0, all its life, and a constant Rotate.
fn upright(app: &Node) -> bool {
    let (width, height, rotate) = (float_keys(app, WIDTH), float_keys(app, ASPECT), float_keys(app, ROTATE));
    let tall = height.iter().copied().fold(f32::MAX, f32::min).min(1.0);
    !width.is_empty() && width.iter().all(|&w| w < UPRIGHT_WIDTH * tall) && rotate.windows(2).all(|r| r[0] == r[1])
}

/// When an appearance transform's offset stops (the last key of its three translation curves,
/// s); read no later, so the curves' repeat flag doesn't take it back to the start.
fn offset_end(app: &Node) -> f32 {
    match app.params.get(&TRANSFORM) {
        Some(crate::bf::ale::Value::Transform(c)) => c.iter().take(3).flat_map(|c| c.0.iter().filter_map(|i| i.3.last().map(|k| k.0)))
            .fold(0.0, f32::max),
        _ => 0.0,
    }
}

/// A steady light: a "light_" effect's pair (grenade effects only, `Compiled::recorded`) whose
/// emitter has a constant rate (no keys) at which its particles overlap two deep or more: its
/// rate (/s). Each particle's light then rises over its first 1/rate s and falls over its last,
/// scaled by rate x life / (rate x life - 1), so the evenly spaced particles' sum stays constant
/// at the mean of the plain particles'. The one it picks out is light_phosphor (5.99/s, 0.514 s:
/// ~3 at once): as plain particles, the number lit stepped between 3 and 4 and the Light's ground
/// light pulsed by 3-11% about three times a second, where the recording's is steady within
/// 1/255 (the tester's measurement). The Frag's and the Sentry's light_explosion (one particle)
/// and the Sonic's light_sonic_grenade (a keyed rate) aren't steady and are lit as before. How
/// the console summed its lights isn't known: this keeps the data's mean.
fn steady_rate(fx: &Compiled, pair: &Pair) -> Option<f32> {
    if !fx.recorded || !pair.light {
        return None;
    }
    let constant = match pair.emitter.params.get(&RATE) {
        Some(crate::bf::ale::Value::Curve(c)) => c.0.len() == 1 && c.0[0].3.is_empty(),
        _ => false,
    };
    let rate = pair.emitter.curve(RATE, 0.0, 0.0).unwrap_or(0.0);
    let life = pair.emitter.curve(LIFE, 0.0, 0.0).unwrap_or(0.0);
    (constant && rate > 0.0 && rate * life >= 2.0).then_some(rate)
}

fn euler(deg: [f32; 3]) -> Quat {
    Quat::from_euler(EulerRot::XYZ, deg[0].to_radians(), deg[1].to_radians(), deg[2].to_radians())
}

/// A direction within `lo`..`hi` degrees of +y.
fn spread(fx: &mut AleEffect, lo: f32, hi: f32) -> Vec3 {
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    let theta = (lo + (hi - lo) * fx.random()).to_radians();
    let phi = fx.random() * std::f32::consts::TAU;
    Vec3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin())
}

fn clock(time: &Time, fixed: Option<Res<AleClock>>) -> f32 {
    fixed.map_or(time.delta_secs().min(0.1), |c| c.0)
}

#[allow(clippy::too_many_arguments)]
fn emit(mut commands: Commands, time: Res<Time>, fixed: Option<Res<AleClock>>, assets: Option<Res<AleAssets>>,
        mut count: ResMut<Count>, mut seq: ResMut<Seq>, mut meshes: ResMut<Assets<Mesh>>,
        mut effects: Query<(Entity, &mut AleEffect, &mut GlobalTransform, &Transform, Has<ChildOf>)>) {
    let Some(assets) = assets else { return };
    let dt = clock(&time, fixed);
    for (fx_entity, mut fx, mut at, local, parented) in &mut effects {
        let owner = fx_entity;
        // a new effect's world placement is known from its second frame (transforms propagate
        // after this runs); one without a parent is where its own transform says at once (a
        // blast starts on the frame it goes off), and its global transform is set from it now so
        // its attached particles and beam ribbons (placed from it in `animate`) aren't drawn at
        // the origin for that frame
        if !fx.placed && parented {
            fx.placed = true;
            continue;
        }
        if !fx.placed {
            *at = GlobalTransform::from(*local);
        }
        let place = at.compute_transform();
        fx.placed = true;
        if !fx.active {
            fx.started = false;
            continue;
        }
        let fresh = !fx.started;
        fx.started = true;
        let compiled = fx.fx.clone();
        for (i, pair) in compiled.pairs.iter().enumerate() {
            let em = &pair.emitter;
            let lifespan = em.float(LIFESPAN).unwrap_or(1.0).max(0.01);
            if fx.once && fx.t >= lifespan {
                continue;
            }
            // a finite emitter starts over while the effect stays on
            let t = fx.t % lifespan;
            let sp = fx.sp;
            let rate = em.curve(RATE, sp, t).unwrap_or(0.0).max(0.0);
            let initial = em.int(INITIAL).unwrap_or(0).max(0) as usize;
            // no rate and no initial count: its burst when it starts (exp-lrg-flash's 4), else
            // one particle (a gun's tracer)
            let burst = if compiled.recorded { em.curve(BURST, sp, 0.0).unwrap_or(0.0).round().max(1.0) as usize } else { 1 };
            let mut n = if fresh { if initial == 0 && rate == 0.0 { burst } else { initial } } else { 0 };
            let carried = fx.acc[i];
            fx.acc[i] += rate * dt;
            n += fx.acc[i] as usize;
            fx.acc[i] = fx.acc[i].fract();
            let (from, from_turn) = fx.prev.unwrap_or((place.translation, place.rotation));
            // a beam pair's ribbon, made with its first particles
            if let (true, Some(material), None) = (n > 0, &pair.beam, fx.strips[i]) {
                let mesh = meshes.add(Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default()));
                // (never culled: its bounds would be those of its first few points, its mesh
                // being rebuilt in the world every frame)
                fx.strips[i] = Some(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                    bevy::render::view::NoFrustumCulling, Transform::default(), Visibility::default(),
                    BeamStrip { source: owner, pair: i, fx: compiled.clone(), mesh, age: 0.0 })).id());
            }
            if n > 0 && std::env::var("BF_ALE_LOG").is_ok() {
                println!("{} pair {i}: t {:.3} dt {dt:.4} emits {n} (alive {}) at {:.2}", compiled.name, fx.t, count.0, place.translation);
            }
            for j in 0..n {
                if count.0 >= MAX_PARTICLES {
                    break;
                }
                // spread over the frame: the emitter's own animated offset and the effect's
                // movement since the last frame (a fast trail or a beam drawn down the line)
                let f = (j + 1) as f32 / n as f32;
                let tj = (t - dt * (1.0 - f)).max(0.0);
                let (at, rot) = (from.lerp(place.translation, f), from_turn.slerp(place.rotation, f));
                let [offset, turn, _] = em.transform(TRANSFORM, tj);
                let frame = rot * euler(turn);
                // (a grenade effect's offsets are scaled by its entity's scale: play_energy.rs
                // fits a bolt's reach to the body it strikes so)
                let offset = if compiled.recorded { Vec3::from(offset) * place.scale } else { Vec3::from(offset) };
                let origin = at + rot * offset;
                let speed = em.curve(SPEED, sp, t).unwrap_or(0.0);
                let c = |p: u32| em.curve(p, sp, t).unwrap_or(0.0);
                let (local, dir) = match em.class {
                    CLASS_SPHERE => {
                        let d = spread(&mut fx, 0.0, 180.0);
                        let (r0, r1) = (c(SPHERE_RADIUS[0]), c(SPHERE_RADIUS[1]));
                        (d * (r0 + (r1 - r0) * fx.random()), d)
                    }
                    CLASS_CONE => {
                        let d = spread(&mut fx, c(CONE_SPREAD[0]), c(CONE_SPREAD[1]));
                        let (r0, r1) = (c(CONE_RADIUS[0]), c(CONE_RADIUS[1]));
                        let flat = Vec3::new(d.x, 0.0, d.z).normalize_or_zero();
                        (flat * (r0 + (r1 - r0) * fx.random()), d)
                    }
                    _ => {
                        let size = Vec3::new(c(CUBE[0]), c(CUBE[1]), c(CUBE[2]));
                        let p = Vec3::new(fx.random() - 0.5, fx.random() - 0.5, fx.random() - 0.5) * size;
                        (p, spread(&mut fx, c(CUBE_SPREAD[0]), c(CUBE_SPREAD[1])))
                    }
                };
                let life = em.curve(LIFE, sp, t).unwrap_or(1.0).max(0.02);
                // a particle carrying an effect lives no longer than that effect runs once
                // (stun_grenade_master's 7.49 s particles carry 0.8 s bolts): it would carry an
                // effect with nothing left to draw
                let life = pair.child.as_ref().map_or(life, |c| life.min(c.duration()));
                let roll = if pair.perp || pair.rise.is_some() { 0.0 } else { fx.random() * std::f32::consts::TAU };
                let mut lie = frame * euler(pair.app.transform(TRANSFORM, fx.t)[1]) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                // a perp quad from a sphere emitter that throws its particles out: facing out
                // along its direction from the middle (see PERP_RADIAL_SPEED)
                if compiled.recorded && pair.perp && em.class == CLASS_SPHERE && speed >= PERP_RADIAL_SPEED {
                    lie = Quat::from_rotation_arc(Vec3::Z, (frame * dir).normalize_or(Vec3::Y));
                }
                count.0 += 1;
                // attached: kept in the effect's frame (offset and turn without the effect's own)
                let (owner, local, vel, lie) = if pair.attached {
                    let inv = place.rotation.inverse();
                    (Some(owner), inv * (origin - place.translation) + inv * frame * local, inv * frame * dir * speed, inv * lie)
                } else {
                    (None, Vec3::ZERO, frame * dir * speed, lie)
                };
                seq.0 += 1;
                // a steady light's particles start at the moment within the frame the rate says
                // they're due, so they come at even spacing (see `steady_rate`)
                let age = match steady_rate(&compiled, pair) {
                    Some(rate) => -((j + 1) as f32 - carried) / rate,
                    None => 0.0,
                };
                let particle = AleParticle { fx: compiled.clone(), pair: i, age, life, vel, sp, born: fx.t, roll, step: 0, frame: lie, owner, local,
                                             turn: place.rotation, source: owner.unwrap_or(fx_entity), seq: seq.0 };
                let at = Transform::from_translation(origin + frame * local).with_scale(Vec3::splat(0.001));
                if let Some(child) = &pair.child {
                    // an effect appearance: the particle carries its effect, turned to face
                    // (+z) the way the particle goes along the ground and kept upright. The turn
                    // is an inference: the Energy recording's bolts run out every way from the
                    // middle, and the bolt's own reach runs along its +z (stun_grenade.emt's
                    // offset sweeps 0 -> 5.8 m along z)
                    let going = particle.vel;
                    let flat = Vec3::new(going.x, 0.0, going.z).normalize_or(Vec3::Z);
                    let carrier = commands.spawn((Transform::from_translation(at.translation).with_rotation(Quat::from_rotation_arc(Vec3::Z, flat)),
                                                  Visibility::default(), particle)).id();
                    let seed = fx.rng ^ (seq.0 as u32).wrapping_mul(0x9E37_79B9);
                    commands.spawn((Transform::default(), Visibility::default(), AleEffect::once(child.clone(), sp, seed), ChildOf(carrier)));
                } else if pair.beam.is_some() {
                    // drawn by the ribbon, not as a quad
                    commands.spawn((at, particle));
                } else if pair.light {
                    commands.spawn((at.with_scale(Vec3::ONE), particle,
                                    PointLight { intensity: 0.0, range: 0.5, shadows_enabled: false, ..default() }));
                } else {
                    commands.spawn((Mesh3d(assets.quad.clone()), MeshMaterial3d(pair.steps[0][0].clone()), NotShadowCaster, at, particle));
                }
            }
        }
        fx.t += dt;
        fx.prev = Some((place.translation, place.rotation));
    }
}

#[allow(clippy::type_complexity)]
fn animate(mut commands: Commands, time: Res<Time>, fixed: Option<Res<AleClock>>, mut count: ResMut<Count>,
           camera: Query<&GlobalTransform, With<Camera3d>>, owners: Query<&GlobalTransform, (With<AleEffect>, Without<AleParticle>)>,
           mut particles: Query<(Entity, &mut AleParticle, &mut Transform, Option<&mut MeshMaterial3d<StandardMaterial>>, Option<&mut PointLight>)>) {
    let dt = clock(&time, fixed);
    let facing = camera.iter().next().map(|c| c.compute_transform().rotation).unwrap_or_default();
    for (e, mut p, mut tr, mat, light) in &mut particles {
        p.age += dt;
        if p.age >= p.life {
            commands.entity(e).despawn();
            count.0 = count.0.saturating_sub(1);
            continue;
        }
        let k = p.age / p.life;
        let fx = p.fx.clone();
        let pair = &fx.pairs[p.pair];
        let app = &pair.app;
        // its air fields: toward each one's wind (in the world's frame, see CLASS_AIR_FIELD)
        for field in &pair.fields {
            let t = p.born + p.age;
            let wind = euler(field.transform(TRANSFORM, t)[1]) * Vec3::Y * field.curve(AIR_MAGNITUDE, p.sp, t).unwrap_or(0.0);
            let approach = field.curve(AIR_APPROACH, p.sp, t).unwrap_or(0.0).clamp(0.0, 1.0);
            let k = 1.0 - (1.0 - approach).powf(dt * APPROACH_FPS);
            let v = p.vel;
            p.vel = v + (wind - v) * k;
        }
        // attached particles ride on their effect (left where they are if it's gone)
        let mut turn = Quat::IDENTITY;
        match p.owner.map(|o| owners.get(o)) {
            Some(Ok(o)) => {
                let o = o.compute_transform();
                let v = p.vel;
                p.local += v * dt;
                tr.translation = o.translation + o.rotation * p.local;
                turn = o.rotation;
                p.turn = turn;
            }
            Some(Err(_)) => {
                // the effect is gone: go on in the world from its last frame
                p.owner = None;
                p.vel = p.turn * p.vel;
                p.frame = p.turn * p.frame;
                p.turn = Quat::IDENTITY;
            }
            None => tr.translation += p.vel * dt,
        }
        // an upright streak rises by its appearance's offset over its age (see `Pair::rise`; in
        // the world's frame: the blasts are placed upright)
        if let Some(end) = pair.rise {
            let at = |t: f32| Vec3::from(app.transform(TRANSFORM, t.clamp(0.0, end * 0.9999))[0]);
            tr.translation += at(p.age) - at(p.age - dt);
        }
        let moving = turn * p.vel;
        // a light's: reach = size, strength = colour x alpha x size
        if let Some(mut light) = light {
            let size = app.floats(SIZE, p.sp, k).unwrap_or(1.0).max(0.0);
            let a = app.floats(ALPHA, p.sp, k).unwrap_or(1.0).clamp(0.0, 1.0);
            let c = Vec3::from(app.color(COLOR, p.sp, k).unwrap_or([1.0; 3]));
            let peak = c.max_element();
            // a steady light's particles fade in and out over one spacing (see `steady_rate`)
            let steady = match steady_rate(&fx, pair) {
                Some(rate) => {
                    let gap = 1.0 / rate;
                    let ramp = (p.age.max(0.0) / gap).min((p.life - p.age) / gap).clamp(0.0, 1.0);
                    ramp * rate * p.life / (rate * p.life - 1.0)
                }
                None => 1.0,
            };
            light.intensity = LIGHT_LUMENS * a * size * peak * LIGHT_SCALE * steady;
            light.range = size.max(0.1);
            if peak > 1e-4 {
                light.color = Color::srgb(c.x / peak, c.y / peak, c.z / peak);
            }
            continue;
        }
        let Some(mut mat) = mat else { continue };           // a beam's: the ribbon draws it
        let size = app.floats(SIZE, p.sp, k).unwrap_or(1.0).max(0.0);
        let width = (size * app.floats(WIDTH, p.sp, k).unwrap_or(1.0)).max(0.001);
        let height = (size * app.floats(ASPECT, p.sp, k).unwrap_or(1.0)).max(0.001);
        let spin = Quat::from_rotation_z(p.roll + app.floats(ROTATE, p.sp, k).unwrap_or(0.0));
        let aspect = app.floats(ASPECT, p.sp, k).unwrap_or(1.0);
        tr.rotation = if pair.perp {
            turn * p.frame * spin
        } else if (pair.streak || aspect > 1.5) && moving.length_squared() > 1.0 {
            // stretched along the motion as seen from the camera
            let v = facing.inverse() * moving;
            facing * Quat::from_rotation_z(v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2)
        } else {
            facing * spin
        };
        tr.scale = Vec3::new(width, height, 1.0);
        let step = ((k * STEPS as f32) as usize).min(STEPS - 1);
        let frames = pair.steps[step].len();
        let frame = match pair.frame {
            FrameMode::Play => (p.age * pair.fps) as usize % frames,
            FrameMode::Hold(v) => ((v * frames as f32) as usize).min(frames - 1),
            FrameMode::Life => ((app.floats(TEX_FRAME, p.sp, k).unwrap_or(0.0).clamp(0.0, 1.0) * frames as f32) as usize).min(frames - 1),
        };
        if step * 1000 + frame != p.step {
            p.step = step * 1000 + frame;
            mat.0 = pair.steps[step][frame].clone();
        }
    }
}

/// Beam ribbons: each joins its effect's live particles of its pair, oldest to newest, as a
/// strip facing the camera, with the appearance's width and colour x alpha at each particle's
/// age. A ribbon goes when its particles have.
fn beams(mut commands: Commands, time: Res<Time>, fixed: Option<Res<AleClock>>, mut meshes: ResMut<Assets<Mesh>>,
         camera: Query<&GlobalTransform, With<Camera3d>>, particles: Query<(&AleParticle, &Transform)>,
         effects: Query<(), With<AleEffect>>, mut strips: Query<(Entity, &mut BeamStrip, &mut MeshMaterial3d<StandardMaterial>)>) {
    let dt = clock(&time, fixed);
    let eye = camera.iter().next().map(|c| c.translation()).unwrap_or_default();
    let mut points: HashMap<(Entity, usize), Vec<(u64, Vec3, f32, f32)>> = HashMap::new();
    for (p, tr) in &particles {
        if p.fx.pairs[p.pair].beam.is_some() {
            points.entry((p.source, p.pair)).or_default().push((p.seq, tr.translation, p.age / p.life, p.sp));
        }
    }
    for (e, mut strip, mut material) in &mut strips {
        strip.age += dt;
        // an animated beam texture steps through its frames at its own rate
        let pair = &strip.fx.pairs[strip.pair];
        if let Some(frames) = pair.steps.first().filter(|f| f.len() > 1) {
            let m = &frames[(strip.age * pair.fps) as usize % frames.len()];
            if material.0 != *m {
                material.0 = m.clone();
            }
        }
        let mut pts = points.remove(&(strip.source, strip.pair)).unwrap_or_default();
        // the ribbon goes when its effect has and its particles have died
        if pts.is_empty() && strip.age > 0.2 && effects.get(strip.source).is_err() {
            commands.entity(e).despawn();
            continue;
        }
        pts.sort_by_key(|p| p.0);
        // particles born together at one spot (several a frame from a still emitter) make one
        // point of the ribbon
        pts.dedup_by(|b, a| a.1.distance_squared(b.1) < 1e-4);
        let app = &strip.fx.pairs[strip.pair].app;
        let (mut pos, mut col, mut uv, mut idx) = (vec![], vec![], vec![], vec![]);
        if std::env::var("BF_ALE_LOG").is_ok() && !pts.is_empty() {
            let (f, l) = (pts[0], pts[pts.len() - 1]);
            println!("  ribbon {} pair {}: {} points {:.1} .. {:.1}, width {:.2} alpha {:.2}", strip.fx.name, strip.pair, pts.len(), f.1, l.1,
                     app.floats(BEAM_WIDTH, f.3, f.2).unwrap_or(-1.0), app.floats(BEAM_ALPHA, f.3, f.2).unwrap_or(-1.0));
        }
        let n = pts.len();
        for (i, &(_, p, k, sp)) in pts.iter().enumerate() {
            let prev = pts[i.saturating_sub(1)].1;
            let next = pts[(i + 1).min(n - 1)].1;
            let along = (next - prev).normalize_or_zero();
            let side = along.cross((eye - p).normalize_or_zero()).normalize_or_zero();
            let w = app.floats(BEAM_WIDTH, sp, k).unwrap_or(0.2).max(0.0) * 0.5;
            let [r, g, b] = app.color(BEAM_COLOR, sp, k).unwrap_or([1.0; 3]);
            let a = app.floats(BEAM_ALPHA, sp, k).unwrap_or(1.0).clamp(0.0, 1.0);
            let u = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.0 };
            // the beam textures run along v (as the `tracer` sprite, stretched in y, shows)
            for (s, across) in [(-1.0, 0.0), (1.0, 1.0)] {
                pos.push((p + side * w * s).to_array());
                col.push([r, g, b, a]);
                uv.push([across, u]);
            }
            if i + 1 < n {
                let j = (i * 2) as u32;
                idx.extend_from_slice(&[j, j + 1, j + 2, j + 1, j + 3, j + 2]);
            }
        }
        if let Some(mesh) = meshes.get_mut(&strip.mesh) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            mesh.insert_indices(bevy::render::mesh::Indices::U32(idx));
        }
    }
}

/// Start the idle effects of the level's placed objects (the power-ups' icons: object type ->
/// idle effect type -> its ALE effects). Returns how many started.
pub fn spawn_idle_effects(commands: &mut Commands, game: &mut Game, level: &crate::bf::level::Level, assets: &mut AleAssets,
                          images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> usize {
    let mut n = 0;
    for (i, o) in level.objects.iter().enumerate() {
        let Some(list) = game.idle_effects.get(&o.kind).and_then(|t| game.effect_types.get(t)).cloned() else { continue };
        for effect in list {
            if let Some(fx) = assets.load(game, images, materials, effect) {
                n += 1;
                if std::env::var("BF_LEVEL_DUMP").is_ok() {
                    println!("idle effect {} at {:.2}", fx.name, o.transform.w_axis.truncate());
                }
                commands.spawn((Transform::from_matrix(o.transform), Visibility::default(),
                                AleEffect::new(fx, 0.0, 0x9E37_79B9 ^ (i as u32).wrapping_mul(0x85EB_CA6B)),
                                Name::new(format!("idle effect h_{effect:08x}"))));
            }
        }
    }
    n
}
