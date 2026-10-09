//! The Energy grenade's bolts (#74): "Releases a maelstrom of charged electrical bolts upon
//! detonation."
//!
//! Its blast is play_grenade.rs's like any other (fuse 1.75 s from landing; the explosion
//! h_f7d6b42e: Damage 39-97.5, damage-type 6 = DTYPE_PARTICLE in the XBE's table at 0x3bd9d0,
//! radius 9; the effect type h_e1f6c1e6: stun_grenade_master + stun_hit_s, sound f875b6c6, no
//! light; decal h_f4d65518), but its damage isn't dealt at once: bolts carry it.
//!
//! What the data and the recording (todo/Energy Grenade.mp4; the reference agent's frames)
//! give:
//!  - stun_grenade_master's emitter throws ~6-7 particles in its first 0.33 s (24/s to 0.22 s,
//!    then down to 0 at 0.33 s), at 3.84 m/s, each carrying the effect "stun_grenade" (its
//!    effect appearance, ale_fx.rs CLASS_SPAWNER): a beam ribbon (ARCb, blue-white, 0.59 m wide)
//!    through ~26 particles laid over 0.3 s by an emitter whose offset sweeps 5.8 m out along
//!    its z in 0.2 s while jittering sideways and climbing, each drifting 1.6 m/s for 0.49 s.
//!    That's a bolt: a jagged strand reaching out ~6 m from a head crawling out at 3.84 m/s,
//!    gone 0.8 s after it starts. The recording: 3-8 strands, crawling out from +0.1-0.18 s to
//!    +0.77-0.83 s; 3.84 m/s for 0.8 s plus the 5.8 m reach is ~9 m, the explosion's radius.
//!  - The bolts strike bodies in reach, living and dead: strands end on a standing squadmate
//!    and on corpses; a corpse is thrown into the air; Hawk at the grenade died at +0.13 s;
//!    Brutus (the thrower, at 0 m) lost 31.5% of his bar in one step at +0.2 s; a squadmate
//!    further off was thrown up and back at +0.6 s, down ~1 s, then up again, not killed.
//!
//! Whether the game's bolts seek bodies, or are random bolts drawn to bodies near them, the
//! recording doesn't settle. Here (an inference): the master effect's own random bolts run as
//! the data has them, and every body within the radius draws a bolt of its own (`Bolt`) from
//! the middle toward it, which strikes it when its reach gets there; the damage, the knock-down
//! and the shove come with the strike.

use super::*;
use bf_viewer::bf::hash::h;

/// The ALE effect whose bolts the blast releases (in the explosion's effect type's list): a blast
/// with it deals its damage through bolts (`releases_bolts`).
const BOLT_MASTER: &str = "stun_grenade_master";
/// The bolts' first strike, after the blast (s): Hawk, at the grenade, died at +0.13 s in the
/// recording; Brutus at 0 m lost his 31.5% at +0.2 s (the health bar may lag a frame or two).
/// Measured.
const BOLT_DELAY: f32 = 0.13;
/// How fast a bolt's head crawls out (m/s): stun_grenade_master.emt's particle SPEED, 3.84.
const BOLT_SPEED: f32 = 3.84;
/// How far a bolt reaches out from its head (m) and in how long (s): stun_grenade.emt's offset
/// along z, 5.80 m at 0.198 s.
const BOLT_REACH: f32 = 5.8;
const BOLT_REACH_TIME: f32 = 0.198;
/// The least a bolt's reach is scaled to, for a body at the grenade (the thrower standing on it):
/// a short arc on them, not a dot. The demo's choice.
const BOLT_MIN_SCALE: f32 = 0.15;
/// How long a bolt's entity stays (s): its emitter's 0.32 s plus its particles' 0.49 s
/// (stun_grenade.emt LIFESPAN and LIFE), and some.
const BOLT_LIFE: f32 = 0.9;
/// The thrower's damage from a bolt, as a share of the explosion's Damage max: Brutus at 0 m lost
/// 31.5% of his 105 HP (33) in one step, 0.34 of 97.5. Fitted to that one measurement; the game's
/// rule isn't known (the Frag's thrower took 0.2 x the roll, play_grenade.rs SELF_DAMAGE: 7.8-19.5
/// here, which the recording's 33 doesn't fit).
const SELF_BOLT: f32 = 0.34;
/// A struck body (living or dead) is thrown back and up (m/s, world): the recording's squadmate
/// went up and back and was down ~1 s; a corpse was thrown into the air. The amounts are the
/// demo's (the data has none).
const THROW_BACK: f32 = 3.0;
const THROW_UP: f32 = 4.0;
/// A corpse struck: shoved along a line from below it, this far through (m) (the ragdoll's
/// `shove`, which brings the bones it passes up to SHOT_SHOVE m/s along the line).
const CORPSE_SHOVE_FROM: f32 = 1.0;

/// Blasts that release bolts this frame: (grenade type, where, the thrower's character), from
/// play_grenade.rs's `fly_grenades`.
#[derive(Resource, Default)]
pub struct BoltRequests(pub Vec<(usize, Vec3, usize)>);

/// A bolt drawn to a body: from `from` (the blast, on the ground) toward the body at `dist` m
/// along `dir` (flat), its head crawling out at BOLT_SPEED and stopping where its reach (scaled
/// to `scale` x BOLT_REACH) ends on the body.
pub(super) struct Bolt {
    kind: usize,
    thrower: usize,
    /// the struck body's character (CHARACTERS index)
    target: usize,
    from: Vec3,
    dir: Vec3,
    dist: f32,
    scale: f32,
    /// seconds since the blast
    age: f32,
    /// the bolt's effect (stun_grenade), once it has started
    entity: Option<Entity>,
    struck: bool,
    /// sim time of the blast (for BF_GRENADE_LOG)
    blast_time: f32,
}

/// The bolts out.
#[derive(Resource, Default)]
pub(super) struct Bolts(Vec<Bolt>);

/// The bolt requests and the bolts out, emptied when a map starts.
pub fn plugin(app: &mut App) {
    app.init_resource::<BoltRequests>().init_resource::<Bolts>()
        .add_systems(OnEnter(AppState::Playing), clear);
    // (`strike` runs in play_grenade.rs's chain, right after the blasts that request bolts)
}

/// A new map: no bolts.
fn clear(mut requests: ResMut<BoltRequests>, mut bolts: ResMut<Bolts>) {
    requests.0.clear();
    bolts.0.clear();
}

/// Whether a grenade type's blast deals its damage through bolts (its effect type has
/// BOLT_MASTER: the Energy).
pub fn releases_bolts(kit: &super::grenade::GrenadeKit) -> bool {
    kit.blast_fx.effects.contains(&h(BOLT_MASTER))
}

/// Where a character's body is (world): where the ragdoll lies, for the dead; else where they
/// stand.
fn body_place(u: &Player) -> Vec3 {
    if u.dead { u.body_at.unwrap_or(u.position) } else { u.position }
}

/// New blasts draw a bolt to every body in their radius; bolts crawl out, and strike: the
/// damage (the thrower SELF_BOLT of Damage max, the others Damage max falling to nothing at the
/// radius, as play_grenade.rs's blasts), the red tint for the player, a knock-down for a living
/// squadmate, a shove into the air for a corpse.
#[allow(clippy::too_many_arguments)]
pub(super) fn strike(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
          kits: Option<Res<super::grenade::GrenadeKits>>, mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
          (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
          mut requests: ResMut<BoltRequests>, mut bolts: ResMut<Bolts>, mut tint: ResMut<super::grenade::ScreenTint>,
          mut places: Query<&mut Transform>, test: Option<Res<super::testmap::TestMap>>) {
    let Some(kits) = kits else { return };
    let dt = frame_dt(&time);
    let log = std::env::var("BF_GRENADE_LOG").is_ok();
    // a bolt to every body (living or dead) in reach of each new blast
    for (kind, at, thrower) in requests.0.drain(..) {
        let Some(kit) = kits.0.get(kind) else { continue };
        let radius = kit.blast.blast_radius;
        for u in std::iter::once(&*player).chain(squad.0.iter()) {
            let p = body_place(u);
            let flat = Vec3::new(p.x - at.x, 0.0, p.z - at.z);
            let dist = flat.length();
            if dist >= radius {
                continue;
            }
            let dir = flat.normalize_or(Vec3::Z);
            bolts.0.push(Bolt { kind, thrower, target: u.character, from: at, dir, dist, scale: (dist / BOLT_REACH).clamp(BOLT_MIN_SCALE, 1.0),
                                age: 0.0, entity: None, struck: false, blast_time: player.sim_time });
            if log {
                println!("t {:.2}: {} bolt to {} ({}) {dist:.1} m away", player.sim_time, kit.def.label, CHARACTERS[u.character],
                         if u.dead { "dead" } else { "alive" });
            }
        }
    }
    let bolt_fx = ale.as_deref_mut().and_then(|a| a.load_recorded(&mut game.0, &mut images, &mut materials, h(BOLT_MASTER)))
        .and_then(|m| m.children().into_iter().next());
    let instant = test.as_ref().is_some_and(|t| t.instant_kill);
    let p = &mut *player;
    for b in &mut bolts.0 {
        b.age += dt;
        let t = b.age - BOLT_DELAY;
        if t < 0.0 {
            continue;
        }
        // the head crawls out until the reach ends on the body
        let reach = BOLT_REACH * b.scale;
        let head = (BOLT_SPEED * t).min((b.dist - reach).max(0.0));
        let place = Transform::from_translation(b.from + b.dir * head)
            .with_rotation(Quat::from_rotation_arc(Vec3::Z, b.dir)).with_scale(Vec3::new(1.0, 1.0, b.scale));
        match b.entity {
            None => if let Some(fx) = &bolt_fx {
                let seed = (b.from.x * 311.0 + b.from.z * 97.0) as u32 ^ (b.target as u32).wrapping_mul(0x85EB_CA6B);
                b.entity = Some(commands.spawn((place, Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx.clone(), 0.0, seed),
                                                AleExpire(BOLT_LIFE), Name::new("energy bolt"))).id());
            },
            Some(e) => if let Ok(mut tr) = places.get_mut(e) {
                *tr = place;
            },
        }
        // it strikes when its reach gets to the body's side (BODY_RADIUS short of the middle):
        // at once for one standing on the grenade
        if b.struck || head + reach * (t / BOLT_REACH_TIME).min(1.0) < b.dist - BODY_RADIUS {
            continue;
        }
        b.struck = true;
        let Some(kit) = kits.0.get(b.kind) else { continue };
        let (radius, max) = (kit.blast.blast_radius, kit.blast.damage);
        let leader = p.character == b.target;
        let u = if leader { Some(&mut *p) } else { squad.0.iter_mut().find(|m| m.character == b.target) };
        let Some(u) = u else { continue };
        let throw = b.dir * THROW_BACK + Vec3::Y * THROW_UP;
        if log {
            println!("t {:.2}: {} bolt strikes {} {:.2} s after the blast, {:.1} m out", p_time(b), kit.def.label, CHARACTERS[u.character], b.age, b.dist);
        }
        if u.dead {
            // a corpse: shoved up and out
            let shoved = u.ragdoll.as_mut().is_some_and(|r| shove_up(r, throw));
            if log {
                println!("  {}'s body {}", CHARACTERS[u.character], if shoved { "thrown" } else { "missed" });
            }
            continue;
        }
        // one knocked down already: their body is thrown again
        if let Some(k) = u.knock.as_mut() {
            shove_up(&mut k.ragdoll, throw);
        }
        let k = 1.0 - b.dist / radius;
        let damage = if u.character == b.thrower { SELF_BOLT * max } else if instant && !leader { u.health.max(max * k) } else { max * k };
        hurt(u, &game.0, damage, HURT_CHATTER, throw, body_place(u) + Vec3::Y, -1);
        if leader {
            tint.0 = 0.0;
        }
        // a squadmate who lives is thrown down; the thrower stays up (Brutus did in the
        // recording), whatever `hurt`'s own chance of a knock-down
        let knocked = !u.dead && u.character != b.thrower && u.knock.is_none() && !u.action.airborne();
        u.knock_request = if knocked { Some(throw) } else if u.character == b.thrower { None } else { u.knock_request };
        if std::env::var("BF_COMBAT_LOG").is_ok() {
            println!("{} bolt at {:.1} m: {} takes {damage:.1} -> {:.1} / {:.0}{}", kit.def.label, b.dist, CHARACTERS[u.character], u.health, u.max_health,
                     if knocked { ", thrown down" } else if u.knock.is_some() { ", down already" } else if u.action.airborne() { ", in the air" } else { "" });
        }
    }
    bolts.0.retain(|b| b.age < BOLT_DELAY + BOLT_LIFE + b.dist / BOLT_SPEED);
}

/// Throw a limp body (a corpse, one knocked down) along `throw`: the ragdoll's `shove` along a
/// line from below its pelvis up through it. Whether it hit a bone.
fn shove_up(r: &mut Ragdoll, throw: Vec3) -> bool {
    let pelvis = r.origin + Quat::from_rotation_y(r.yaw) * r.pos[0];
    let line = throw.normalize_or(Vec3::Y);
    r.shove(pelvis - line * CORPSE_SHOVE_FROM, line, CORPSE_SHOVE_FROM * 2.0).is_some()
}

/// The sim time a bolt's strike happens at (its blast's time + its age), for the log.
fn p_time(b: &Bolt) -> f32 {
    b.blast_time + b.age
}
