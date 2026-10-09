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
const BLEND_ADD: (u32, u32) = (5, 2);

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
    perp: bool,
    /// particles stay in the emitter's frame (move with it)
    attached: bool,
    /// turned along their motion
    streak: bool,
    /// a beam appearance: the ribbon's material (vertex colours carry colour x alpha)
    beam: Option<Handle<StandardMaterial>>,
    /// a "light_" effect's pair: its particles are point lights
    light: bool,
}

/// An effect ready to run.
pub struct Compiled {
    pub name: String,
    pairs: Vec<Pair>,
}

impl Compiled {
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
    effects: HashMap<u32, Option<Arc<Compiled>>>,
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
        .add_systems(PostUpdate, (emit, animate, beams).chain().before(TransformSystem::TransformPropagate));
}

impl AleAssets {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        AleAssets { quad: meshes.add(Rectangle::new(1.0, 1.0)), effects: HashMap::new(), textures: HashMap::new() }
    }

    /// An effect compiled already (`load`).
    pub fn cached(&mut self, effect: u32) -> Option<Arc<Compiled>> {
        self.effects.get(&effect).cloned().flatten()
    }

    /// Compile an effect of the library by name hash (cached); None if the library lacks it.
    pub fn load(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                effect: u32) -> Option<Arc<Compiled>> {
        if let Some(c) = self.effects.get(&effect) {
            return c.clone();
        }
        let compiled = self.compile(game, images, materials, effect).map(Arc::new);
        if compiled.is_none() {
            warn!("ALE effect h_{effect:08x}: not in the library, or nothing it draws is supported");
        }
        self.effects.insert(effect, compiled.clone());
        compiled
    }

    fn compile(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
               effect: u32) -> Option<Compiled> {
        let e = game.effects.effects.get(&effect)?.clone();
        let node = |game: &Game, i: u32| e.refs.iter().find(|r| r.3 == i).and_then(|r| game.effects.nodes.get(&r.1)).cloned();
        let mut pairs = vec![];
        for &(em, ap) in &e.pairs {
            let (Some(emitter), Some(app)) = (node(game, em), node(game, ap)) else { continue };
            if app.class == CLASS_BEAM {
                let additive = app.pair(BEAM_BLEND).unwrap_or(BLEND_ADD) == BLEND_ADD;
                let texture = app.string(BEAM_TEXTURE).and_then(|n| self.texture(game, images, h(n), additive));
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE, base_color_texture: texture, unlit: true, double_sided: true, cull_mode: None,
                    fog_enabled: false, alpha_mode: if additive { AlphaMode::Add } else { AlphaMode::Blend }, ..default()
                });
                pairs.push(Pair { perp: false, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![], fps: 0.0,
                                  beam: Some(material), light: false });
                continue;
            }
            if app.class != CLASS_APPEARANCE {
                continue;
            }
            if e.name.to_ascii_lowercase().starts_with("light_") {
                pairs.push(Pair { perp: false, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![], fps: 0.0,
                                  beam: None, light: true });
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
            pairs.push(Pair { perp: app.flag(PERP), attached: emitter.flag(ATTACHED), streak, emitter, app, steps, fps: book.map_or(0.0, |b| b.fps),
                              beam: None, light: false });
        }
        (!pairs.is_empty()).then_some(Compiled { name: e.name, pairs })
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
        mut effects: Query<(Entity, &mut AleEffect, &GlobalTransform)>) {
    let Some(assets) = assets else { return };
    let dt = clock(&time, fixed);
    for (fx_entity, mut fx, at) in &mut effects {
        let owner = fx_entity;
        // a new effect's world placement is known from its second frame (transforms propagate
        // after this runs)
        if !fx.placed {
            fx.placed = true;
            continue;
        }
        let place = at.compute_transform();
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
            // no rate and no initial count: one particle when it starts (a gun's tracer)
            let mut n = if fresh { if initial == 0 && rate == 0.0 { 1 } else { initial } } else { 0 };
            fx.acc[i] += rate * dt;
            n += fx.acc[i] as usize;
            fx.acc[i] = fx.acc[i].fract();
            let (from, from_turn) = fx.prev.unwrap_or((place.translation, place.rotation));
            // a beam pair's ribbon, made with its first particles
            if let (true, Some(material), None) = (n > 0, &pair.beam, fx.strips[i]) {
                let mesh = meshes.add(Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default()));
                fx.strips[i] = Some(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                    Transform::default(), Visibility::default(),
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
                let origin = at + rot * Vec3::from(offset);
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
                let roll = if pair.perp { 0.0 } else { fx.random() * std::f32::consts::TAU };
                let lie = frame * euler(pair.app.transform(TRANSFORM, fx.t)[1]) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                count.0 += 1;
                // attached: kept in the effect's frame (offset and turn without the effect's own)
                let (owner, local, vel, lie) = if pair.attached {
                    let inv = place.rotation.inverse();
                    (Some(owner), inv * (origin - place.translation) + inv * frame * local, inv * frame * dir * speed, inv * lie)
                } else {
                    (None, Vec3::ZERO, frame * dir * speed, lie)
                };
                seq.0 += 1;
                let particle = AleParticle { fx: compiled.clone(), pair: i, age: 0.0, life, vel, sp, roll, step: 0, frame: lie, owner, local,
                                             turn: place.rotation, source: owner.unwrap_or(fx_entity), seq: seq.0 };
                let at = Transform::from_translation(origin + frame * local).with_scale(Vec3::splat(0.001));
                if pair.beam.is_some() {
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
        let moving = turn * p.vel;
        // a light's: reach = size, strength = colour x alpha x size
        if let Some(mut light) = light {
            let size = app.floats(SIZE, p.sp, k).unwrap_or(1.0).max(0.0);
            let a = app.floats(ALPHA, p.sp, k).unwrap_or(1.0).clamp(0.0, 1.0);
            let c = Vec3::from(app.color(COLOR, p.sp, k).unwrap_or([1.0; 3]));
            let peak = c.max_element();
            light.intensity = LIGHT_LUMENS * a * size * peak * LIGHT_SCALE;
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
        let frame = (p.age * pair.fps) as usize % frames;
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
         effects: Query<(), With<AleEffect>>, mut strips: Query<(Entity, &mut BeamStrip)>) {
    let dt = clock(&time, fixed);
    let eye = camera.iter().next().map(|c| c.translation()).unwrap_or_default();
    let mut points: HashMap<(Entity, usize), Vec<(u64, Vec3, f32, f32)>> = HashMap::new();
    for (p, tr) in &particles {
        if p.fx.pairs[p.pair].beam.is_some() {
            points.entry((p.source, p.pair)).or_default().push((p.seq, tr.translation, p.age / p.life, p.sp));
        }
    }
    for (e, mut strip) in &mut strips {
        strip.age += dt;
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
