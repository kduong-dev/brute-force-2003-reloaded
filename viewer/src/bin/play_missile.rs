//! Mesh projectiles (bullet-type 2, BTYPE_MESH_PROJECTILE; issue #73, the Therm Sweeper): a
//! missile that flies, steers, splits in two (MIRV) and explodes, from default.xbe's projectile
//! class (0x226000-0x229000) and its weapon type's damage handler (0x22d0b0), checked against
//! the xemu takes in todo/73-therm-sweeper/ (the tester's spec there has the measurements).
//!
//!  - **Launch** (FUN_0022f1d0 case 2 -> FUN_0022fb50): one projectile object, the bullet's
//!    mesh-name (h_e9327576: model h_07d84a52, attached effect h_194dc78e with its sound
//!    0021996c), at the muzzle, along the shot (after the accuracy turn), at the bullet's speed
//!    (FUN_00227ce0), given the weapon's target and aim point (FUN_00226360). Its life is range
//!    / speed (FUN_002324c0 through the holder's skill hook, which leaves the range alone for
//!    the squad: 20 / 30 = 0.667 s), its straight phase h_ea5327e6 of that (0.1 s), then its
//!    second timer h_f0a56297 (5 s) runs (FUN_00227dd0).
//!  - **Each game frame** (FUN_00226cc0, here in 30 Hz ticks after the physics move): steered
//!    (FUN_002266d0) along its own heading in the straight phase, then toward its target's
//!    origin (the feet) or its aim point, its node turned along the velocity it had before (so
//!    the 60-degree test lags a step); the proximity fuse; life, then the second timer, run down.
//!  - **The split** (WM_MIRV_ON_EXPLODE, FUN_00227f80), when its life is out: the parent goes
//!    without a blast; `<mirv>` h_fe04300d children (2) leave beside it, each turned spread-max x
//!    90 degrees (9) off its heading at a random roll, after the hostiles the split finds within
//!    the range and 30 degrees, else 200 m along the parent's heading. A child is NEUTRAL (team
//!    2): its own searches find nobody, and without the split's target it has no fuse.
//!  - **Hitting something** (the collision callback FUN_002278e0 -> 0x22d0b0): the world, a
//!    body (standing, or lying knocked down), a breakable object or a Sentry. The body or
//!    object struck takes a roll of Damage min..max (FUN_0022c4d0; a character's x its type's
//!    damage-multiplier x the struck bone's Damage, FUN_00125940), a blast object goes off at
//!    the missile's centre (FUN_0022bfc0) and the explosion effect type plays (FUN_0022cfc0). A
//!    child whose second timer runs out, or whose fuse fires, goes off the same way where it is,
//!    with no direct hit (FUN_00227760).
//!  - **The blast** (FUN_0022bfc0, dealt once by FUN_00224770 -> FUN_0022c4d0): its own roll,
//!    after 0.05-0.25 s, to every body it reaches with nothing of the world in between (a body
//!    behind cover isn't touched at all, FUN_00224a90): half of it anywhere inside (Damage
//!    h_011cb154 false), x 0.7 for the shooter's own side (FUN_0004b9c0), x the body's
//!    damage-type factor; the scenery's share is the same, flat. Its pass lays the scorch
//!    (bullet h_06a27365) under the blast point, and every blast shakes the player's view
//!    (FUN_00228c70, play_shake.rs). No hit or blast knocks its victim down (`hurt_still`).
//!
//! Only mesh projectiles steered at a steady speed that split (h_0b03ece5 and a `<mirv>`: the
//! Therm Sweeper) fly here. The Sweeper V's (steady, no MIRV) keeps play.rs's plain `Shot` until
//! #72 checks it against its own takes; the others (the iKhan-GPL's lobbed cluster, the
//! creatures' weapons) fly under their physics body's gravity, which isn't traced.
//!
//! `BF_MISSILE_LOG=1` prints each missile's launch, seeks, retargets, split (with its children
//! and their targets), fuse and end (where, how far from its launch, how long after, what it
//! struck), and each body its blast reaches with the damage; `BF_MISSILE_DEBUG=1` also prints
//! every body each target search looks at (distance, angle, line of sight).
//!
//! Not here: the difficulty step on every hit on a character (x1.4 on enemies, x0.6 on the
//! squad on STANDARD, 0x125600: #127, todo/damage-factor/answer.md); the 75 m noise for the AI's
//! hearing (FUN_0004d490); the damage event (FUN_001fe3d0).

use super::*;
use super::grenade::{ScreenTint, BLAST_HEARD, BLAST_QUIET, DECAL_DELAY, GRENADE_DECAL_SCALE};
use super::shake::CameraShake;
use bf_viewer::bf::character::EffectType;
use bf_viewer::bf::weapon::MeshBullet;

/// FUN_002266d0: each frame the part of the velocity across the way it's steered is cut by this
/// share (DAT_00406f94 = 0.5)...
const STEER_ACROSS: f32 = 0.5;
/// ... and its speed along that way pulled this share of the way to the bullet's speed, unless
/// it's within STEER_SLACK m/s of it already.
const STEER_SPEED: f32 = 0.15;
/// FUN_002266d0: a speed along the steering way within this (m/s) of the bullet's is left alone.
const STEER_SLACK: f32 = 0.01;
/// FUN_00226cc0: an aim point within 1 m (squared, 1), or more than 60 degrees off the heading
/// (a dot product under 0.5, [0x3a4f74]), is given up for a point RETARGET_AHEAD s of its
/// velocity ahead (and the target with it); a missile that seeks looks again RESEEK s later
/// (+0x1d0).
const RETARGET_NEAR_SQ: f32 = 1.0;
/// FUN_00226cc0: the cosine under which an aim point is too far off the heading (60 degrees,
/// [0x3a4f74] = 0.5).
const RETARGET_COS: f32 = 0.5;
/// FUN_00226cc0 (0x227038): the aim point given up for is put its velocity (vtable +0x54) x
/// this far ahead: 6 km at 30 m/s.
const RETARGET_AHEAD: f32 = 200.0;
/// FUN_00227f80: a child the split has no target for aims this far (m) along the parent's
/// heading (its frame's -z x 200).
const SPLIT_AHEAD: f32 = 200.0;
/// FUN_00226cc0: when (s) a missile that seeks looks again after giving up its aim point (+0x1d0).
const RESEEK: f32 = 0.5;
/// FUN_00226b80: a missile that seeks and has no target looks every SEEK_EVERY s for the most
/// central hostile within its range and SEEK_CONE degrees of its heading (FUN_002237c0 with
/// 60.0, one target).
const SEEK_EVERY: f32 = 0.25;
/// FUN_00226b80: the cone (degrees off its heading) a child without a target looks in.
const SEEK_CONE: f32 = 60.0;
/// FUN_00227f80: the split looks for up to its child count of hostiles within the range and
/// SPLIT_CONE degrees of the parent's heading (FUN_002237c0 with 30.0).
const SPLIT_CONE: f32 = 30.0;
/// FUN_00227f80: at most this many children (a count over 5 is 6).
const MIRV_MAX: i64 = 6;
/// FUN_00227f80: a child's life is the bullet's range / speed x (1 - CHILD_LIFE + 2 x CHILD_LIFE
/// x U): 0.85-1.15 of it.
const CHILD_LIFE: f32 = 0.15;
/// FUN_00227f80: a child is turned off the parent's heading by spread-max x SPREAD_TURN degrees.
const SPREAD_TURN: f32 = 90.0;
/// FUN_00226cc0: a WM_MIRV_ON_APOGEE parent splits once its upward speed is under this (m/s),
/// past its straight phase. (No mesh projectile that flies here has that mode: the iKhan-GPL's
/// does, and it's lobbed. Kept as the code has it.)
const APOGEE_VY: f32 = 0.5;
/// FUN_00226cc0: the proximity fuse looks every FUSE_EVERY s (+0x218), within the Damage radius
/// x FUSE_REACH (FUN_00227dd0 -> +0x214: 1.69 m for the Therm Sweeper's 4.5).
const FUSE_EVERY: f32 = 0.25;
/// FUN_00227dd0: the fuse's reach as a share of the Damage radius (+0x214).
const FUSE_REACH: f32 = 0.375;
/// FUN_0022bfc0: the blast object deals its damage BLAST_DELAY + BLAST_DELAY_MORE x U s after it
/// goes off (+0x2c): 0.05-0.25 s.
const BLAST_DELAY: f32 = 0.05;
/// FUN_0022bfc0: the random part of the blast object's delay (s, x U).
const BLAST_DELAY_MORE: f32 = 0.2;
/// FUN_0022c4d0, a blast object's damage with Damage h_011cb154 false: this share of its roll,
/// at any distance inside the radius.
const BLAST_SHARE: f32 = 0.5;
/// FUN_0022c4d0: a hit on the shooter's own side (FUN_0004b9c0: both characters, one in the
/// other's squad; here the same team, the shooter too) deals this share.
const FRIENDLY: f32 = 0.7;
/// ALIGN_NEUTRAL (the ALIGN_ table at 0x3bc94c): the side the projectile constructor gives
/// (+0x21c = 2 at 0x2265ec), which a MIRV child keeps (the split calls FUN_00227dd0, which
/// copies the owner's team, before it copies the owner: 0x2282bf, 0x22834a). Row 2 of 0x3ffd80
/// is hostile only to PC_NO_TEAM (7), a side no character of the demo is on: a child's own
/// searches find nobody, and its fuse needs the split's target (0x22731c-0x22732d).
const NEUTRAL: u8 = 2;
/// A body's reach for the blast: the blast's physics query (FUN_00224a90) gathers bodies whose
/// collision is inside its sphere, a character's being its parts (the motion set's
/// `<collision>` bones, each with its own radius). The demo's body is a standing cylinder this
/// round (m) and BODY_HEIGHT tall from the feet, and a blast reaches it when the cylinder's
/// nearest point is within the radius. Fitted to the takes, not traced: Tex was hurt by his own
/// blast on the wall 5 m from his feet and not at 6 m (takes 12, 13, 15; the arms holding the
/// launcher reach out toward it), so at least 0.44 m; play.rs's BODY_RADIUS (0.4) missed at 5 m.
const BLAST_BODY_REACH: f32 = 0.6;
/// Its line of sight is to the body's middle, this high (m): the demo's.
const BODY_MIDDLE: f32 = 1.0;
/// The blast's line of sight leaves from this far (m) off the surface the missile struck (along
/// its normal; up for a body or the air): the game's blast sits where the missile's centre was
/// when it touched, and a ray from the surface itself meets that surface at once. The demo's.
const COVER_LIFT: f32 = 0.25;
/// The search's line of sight (FUN_002237c0) is a ray to the character's origin, the feet, that
/// meets only the terrain (`Arena::ray_terrain`: the query's flags 0xfffffffa leave out the
/// level's placed geometry, rocks and walls, FUN_000df0a0); it stops this short of the feet (m),
/// so the ground they stand on doesn't count (the AI's sight test leaves 0.3 too,
/// `hostile_in_sight`). The demo's.
const SIGHT_SLACK: f32 = 0.3;
/// A missile's sound and its explosion's are heard at this volume from the player (play.rs's
/// grenade trail and play_sounds use 0.8); farther ones fall off as play_sounds does a
/// squadmate's (1 - d / SOUND_REACH, at least SOUND_FLOOR).
const SOUND_VOLUME: f32 = 0.8;
/// The distance (m) over which a missile's sounds fall off (play_sounds' squadmate rule).
const SOUND_REACH: f32 = 35.0;
/// The least share of SOUND_VOLUME a missile's sounds keep however far (play_sounds' rule).
const SOUND_FLOOR: f32 = 0.15;
/// The push handed to `hurt_still` for a missile's direct hit (m/s): along the missile's way, and
/// up. With no knockdown from a missile it only sets which way the blood sprays and, for a kill,
/// the death push that throws the body (as play.rs's shots push). The demo's numbers: the
/// game's push (h_e39b994b, 2, copied to the damage record +0x54) isn't in m/s.
const HIT_PUSH: (f32, f32) = (4.0, 1.5);
/// The same for the blast (m/s): out from it, and up (the grenades' push at their middle,
/// `hurt_by_blast`). The demo's numbers.
const BLAST_PUSH: (f32, f32) = (6.0, 4.0);
/// FUN_00228c70: a blast shakes the view within this many times the Damage radius (weapon type
/// +0x1fc x 7.0)...
const SHAKE_REACH: f32 = 7.0;
/// ... by the weapon's Damage (weapon type +0x1f0, the Damage record's amount: its max) x this
/// (0.00769), at most 1...
const SHAKE_PER_DAMAGE: f32 = 0.007_69;
/// ... held this long (s, FUN_0006bdd0's second argument 0.5)...
const SHAKE_HOLD: f32 = 0.5;
/// ... then falling to nothing over this x the falloff (s)...
const SHAKE_FALL: f32 = 0.75;
/// ... and past half the reach, not at all if the falloff is this or less.
const SHAKE_LEAST: f32 = 0.1;
/// A body lying knocked down is struck where a missile passes within this (m) of one of its
/// ragdoll's bones (`ray_lying`). The demo's: about a limb's or the torso's half thickness.
const LYING_RADIUS: f32 = 0.2;
/// The effects' user parameter (`AleEffect::sp`, the game's +0xbc that FUN_0021c380 sets): a hit's
/// explosion is started at 1 (FUN_0022cbe0's factor 1.0); a timeout's isn't set (FUN_0022cea0),
/// taken as 1 too (a guess). Every curve of exp-rocket-* has one variant, so it changes nothing
/// there.
const EXPLOSION_SP: f32 = 1.0;

/// A mesh projectile's shot this frame: play.rs's shot builder hands these over instead of a
/// `Shot`. The weapon (its `Game::weapons` key), where it leaves from and which way (the muzzle,
/// the shot's direction after the accuracy turn), the aim point (weapon +0x1ec: where the
/// crosshair's ray meets the world, as the shots' `target`) and the line the shooter aims
/// along (the crosshair's ray, or the AI's line to its target), on which the target is found.
#[derive(Clone, Copy, Debug)]
pub struct Launch {
    pub weapon: u32,
    pub origin: Vec3,
    pub dir: Vec3,
    pub aim: Vec3,
    pub look: (Vec3, Vec3),
}

/// A projectile object's model (its parts, each placed in the object's frame), its attached
/// effect type (h_18b6ab72's first slot: the trail's ALE effects and sounds) and its bounding
/// radius (the distance from its origin to the far side of its model's box: what FUN_00227f80
/// spaces the children by, its vtable +0xac read as the bounding sphere).
struct MissileKit {
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform)>,
    trail: EffectType,
    radius: f32,
}

/// The projectile objects' kits, by object type, loaded when a map starts (None: no model).
#[derive(Resource, Default)]
struct MissileKits(HashMap<u32, Option<MissileKit>>);

/// A projectile in flight (the game's projectile object, offsets as FUN_00227dd0 sets them).
#[derive(Component)]
struct Missile {
    /// its number (BF_MISSILE_LOG)
    id: u32,
    /// the weapon that fired it (`Game::weapons` key: its bullet, Damage and effects)
    weapon: u32,
    /// its object type (`MissileKits`)
    object: u32,
    /// who fired it (+0x1b4), their team (the side its damage counts as own, FUN_0004b9c0) and
    /// whether a player controls them (FUN_00112800, the fuse's condition: read as "a human
    /// player's", medium confidence)
    owner: Who,
    team: u8,
    human: bool,
    /// its own side (+0x21c), which its searches and its fuse go by: the shooter's team for a
    /// missile they launched (FUN_00227dd0 copies the owner's), NEUTRAL for a MIRV child
    side: u8,
    pos: Vec3,
    vel: Vec3,
    /// its heading, the node's -z: FUN_002266d0 turns it along the velocity it had before each
    /// steering, so it lags the motion by a step (the model, the 60-degree test, the seek's cone)
    heading: Vec3,
    /// +0x208: the speed steering holds it to (the bullet's)
    speed: f32,
    /// +0x1d8 life, +0x1dc its start (the trail's user parameter is life / start, FUN_00228c10),
    /// +0x230 the straight phase, +0x1d4 the second timer
    life: f32,
    start: f32,
    straight: f32,
    after: f32,
    /// +0x1d0 the seek timer (FUN_00226b80), +0x218 the fuse's clock
    seek_in: f32,
    fuse_clock: f32,
    /// +0x1ec its target, +0x1f0 its aim point
    target: Option<Who>,
    aim: Vec3,
    /// +0x204: its MIRV mode (0 once it's a child: children don't split)
    mode: i64,
    /// game time not yet run in ticks (s)
    clock: f32,
    /// where it was launched and how long ago (BF_MISSILE_LOG)
    from: Vec3,
    age: f32,
    /// its trail effects (their user parameter follows its life)
    trails: Vec<Entity>,
}

/// A missile going off: where, which way it flew, what it struck (a body, the world's surface
/// and its normal, a breakable object, a Sentry mine), and whether it was a collision (the explosion turned along
/// its way, FUN_0022cbe0) or its timers or fuse (upright, FUN_0022cea0).
struct Blow {
    id: u32,
    weapon: u32,
    owner: Who,
    team: u8,
    human: bool,
    at: Vec3,
    dir: Vec3,
    struck: Option<Who>,
    surface: Option<Vec3>,
    mine: Option<Entity>,
    /// the breakable object struck (`Arena::breakable`'s number), if the world it struck is one
    breakable: Option<usize>,
    collision: bool,
    cause: &'static str,
    from: Vec3,
    age: f32,
}

/// A blast object waiting to deal its damage (FUN_0022bfc0): seconds left (+0x2c), where, the
/// weapon (its Damage), its roll (+0x20), the shooter and their team, the missile's number.
#[derive(Clone, Copy)]
struct PendingBlast {
    left: f32,
    at: Vec3,
    /// where its line of sight leaves from (COVER_LIFT off the surface)
    eye: Vec3,
    weapon: u32,
    roll: f32,
    owner: Who,
    team: u8,
    id: u32,
}

/// The missiles' shared state: the next number, the random numbers, the blasts and scorches
/// still to come (the scorch: seconds left, the decal, the blast point it's laid under), and the
/// explosion sounds waiting for their delay (s, id, volume).
#[derive(Resource, Default)]
struct MissileState {
    next: u32,
    seed: u32,
    blasts: Vec<PendingBlast>,
    scorches: Vec<(f32, u32, Vec3)>,
    sounds: Vec<(f32, u32, f32)>,
}

impl MissileState {
    /// The game's random numbers (shots::random), from the state's own seed.
    fn random(&mut self) -> f32 {
        if self.seed == 0 {
            self.seed = 0x7357_0073;
        }
        super::shots::random(&mut self.seed)
    }
}

/// The systems: the blasts due, the missiles in flight, then this frame's launches (a missile
/// shows at the muzzle on the frame it's fired and moves from the next), before the grenades'
/// (so the scenery takes their blasts as it does the grenades').
pub fn plugin(app: &mut App) {
    app.init_resource::<MissileKits>().init_resource::<MissileState>()
        .add_systems(OnEnter(AppState::Playing), load_kits.after(setup))
        .add_systems(Update, (deal_blasts, fly_missiles, launch_missiles).chain()
            .after(update_player).before(super::grenade::GrenadeSystems).run_if(in_state(AppState::Playing)));
}

/// Whether a weapon's shots fly here: a mesh projectile steered at a steady speed that splits
/// (see the module notes). The Sweeper V's (steady, no MIRV) keeps play.rs's plain shot until
/// #72: its own takes (todo/72-sweeper-v) don't match this path's blast and damage yet.
pub fn flies(def: &WeaponDef) -> bool {
    def.mesh_bullet.as_ref().is_some_and(|b| b.steady && b.mirv_mode != 0 && b.mirv_count > 0)
}

/// The projectile objects of the weapons whose shots fly here (and their MIRV children's), their
/// models and trail effects made ready, and the trail's and the explosion's ALE effects
/// compiled with the grenades' recorded rules and the game's facing 3 (`load_recorded_facing`:
/// exp-rocket-* are the exp-lrg-* family's siblings; exp-rocket-add.app and the shrapnel's rect
/// quads are facing 3, see ale_fx's FACING) and their render pipelines built ahead
/// (`ale_fx::warm_up`).
#[allow(clippy::too_many_arguments)]
fn load_kits(mut commands: Commands, mut game: ResMut<GameData>, mut kits: ResMut<MissileKits>, mut state: ResMut<MissileState>,
             mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
             mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>,
             mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>) {
    *state = MissileState::default();
    kits.0.clear();
    let log = std::env::var("BF_MISSILE_LOG").is_ok();
    let guns: Vec<WeaponDef> = game.0.weapons.values().filter(|d| flies(d)).cloned().collect();
    for gun in &guns {
        let Some(b) = &gun.mesh_bullet else { continue };
        let objects = [b.object, if b.mirv_object != 0 { b.mirv_object } else { b.object }];
        for object in objects {
            if kits.0.contains_key(&object) {
                continue;
            }
            let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
            let kit = missile_kit(&mut game.0, &mut assets, object);
            if log {
                match &kit {
                    Some(k) => println!("missile object h_{object:08x} ({}): {} parts, radius {:.2} m, trail {:x?} sounds {:x?}", gun.label,
                                        k.parts.len(), k.radius, k.trail.effects, k.trail.sounds),
                    None => println!("missile object h_{object:08x} ({}): no model", gun.label),
                }
            }
            kits.0.insert(object, kit);
        }
        if let Some(ale) = ale.as_deref_mut() {
            let blast = game.0.effect_type_defs.get(&gun.blast_effect).cloned().unwrap_or_default();
            let trails: Vec<u32> = objects.iter().filter_map(|o| kits.0.get(o).and_then(|k| k.as_ref())).flat_map(|k| k.trail.effects.clone()).collect();
            for e in blast.effects.iter().chain([&blast.light]).chain(trails.iter()).copied().filter(|&e| e != 0) {
                if ale.cached_recorded_facing(e).is_none() {
                    if let Some(fx) = ale.load_recorded_facing(&mut game.0, &mut images, &mut materials, e) {
                        bf_viewer::ale_fx::warm_up(&mut commands, ale, &fx);
                    }
                }
            }
        }
    }
}

/// A projectile object's kit: its model (objecttypes mesh-name -> `WeaponModel`), its attached
/// effect type and its bounding radius. None if it has no model.
fn missile_kit(game: &mut Game, assets: &mut ModelAssets, object: u32) -> Option<MissileKit> {
    let def = game.weapons.get(&object).cloned();
    let arch = def.as_ref().map(|d| d.archetype).filter(|&a| a != 0).or_else(|| game.object_meshes.get(&object).copied())?;
    let model = WeaponModel::load(game, arch).ok()?;
    let mut parts = vec![];
    let (mut lo, mut hi) = (Vec3::MAX, Vec3::MIN);
    for p in &model.parts {
        let place = Transform::from_translation(p.offset).with_rotation(p.rotation);
        for g in &p.geosets {
            for v in &g.positions {
                let v = place.transform_point(Vec3::from(*v));
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        for (mesh, mat) in bf_viewer::scene::static_meshes(game, &p.geosets, assets, true) {
            parts.push((mesh, mat, place));
        }
    }
    if parts.is_empty() || lo.x > hi.x {
        return None;
    }
    let radius = ((lo + hi) * 0.5).length() + ((hi - lo) * 0.5).length();
    let trail = def.map(|d| d.attached_effect).and_then(|e| game.effect_type_defs.get(&e).cloned()).unwrap_or_default();
    Some(MissileKit { parts, trail, radius })
}

/// Where a character's origin is (the feet: the game's character origin, which a missile homes
/// on, the spec's reading of take06).
fn feet(u: &Player) -> Vec3 {
    u.position + Vec3::Y * (GROUND + u.height)
}

/// A character's name for the log.
fn name(u: &Player) -> String {
    format!("{}{}", CHARACTERS[u.character], u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)))
}

/// Whether a missile of side `side` takes a character of team `team` for hostile (the table at
/// 0x3ffd80): the demo's teams, 0 (the squad, CONFEDERATION) and 1 (the enemies), are hostile to
/// each other; NEUTRAL to neither.
fn hostile(side: u8, team: u8) -> bool {
    side != NEUTRAL && side != team
}

/// FUN_002237c0: up to `count` living characters hostile to `side` (see `hostile`) within
/// `range` m of `at` and `cone` degrees of `fwd`, to their origin, with nothing of the world in
/// between (the game casts a ray to it), the most central first.
fn search(units: &[&Player], at: Vec3, fwd: Vec3, range: f32, cone: f32, count: usize, side: u8) -> Vec<Who> {
    let cos = cone.to_radians().cos();
    let mut found: Vec<(f32, Who)> = units.iter().filter(|u| !u.dead && hostile(side, u.team)).filter_map(|u| {
        let to = feet(u) - at;
        let d = to.length();
        if d > range || d < 1e-3 {
            return None;
        }
        let dir = to / d;
        let dot = dir.dot(fwd);
        let reach = (d - SIGHT_SLACK).max(0.01);
        let clear = match world::arena() {
            Some(a) => a.ray_terrain(at, dir, reach).is_none(),
            None => ray_hit(at, dir, reach).is_none(),
        };
        if std::env::var("BF_MISSILE_DEBUG").is_ok() {
            println!("  search from {at:.2}: {} at {:.1} m, {:.0} degrees (cone {cone}), clear {clear}", name(u), d, dot.clamp(-1.0, 1.0).acos().to_degrees());
        }
        (dot >= cos && clear).then(|| (-dot, u.who()))
    }).collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().take(count).map(|f| f.1).collect()
}

/// The origin of the character `who` (None if they're gone).
fn origin_of(units: &[&Player], who: Who) -> Option<Vec3> {
    units.iter().find(|u| u.who() == who).map(|u| feet(u))
}

/// What a tick of a missile came to.
enum Tick {
    Flying,
    /// its life is out and it splits (the parent goes without a blast)
    Split,
    /// it goes off (`Blow`)
    Blow(Blow),
}

/// One game frame of a missile (FUN_00226cc0), `dt` long: the physics move (a collision ends it,
/// FUN_002278e0), then steering, the timers, the proximity fuse, and the split or the end.
#[allow(clippy::too_many_arguments)]
fn tick(m: &mut Missile, b: &MeshBullet, range: f32, radius: f32, units: &[&Player], mines: &super::sentry::MineTargets,
        dt: f32, now: f32, log: bool) -> Tick {
    // the move, and what it meets on the way (FUN_002278e0): the world (a breakable object's
    // collision among it), a body but its shooter's, standing or lying knocked down, a Sentry
    let step = m.vel * dt;
    let len = step.length();
    if len > 1e-6 {
        let dir = step / len;
        let piece = world::arena().and_then(|a| a.ray_breakable(m.pos, dir, len)).and_then(|(t, b)| b.map(|b| (t, b)));
        let mut first: Option<(f32, Option<Who>, Option<Vec3>, Option<Entity>, Option<usize>)> = super::shots::hit_normal(m.pos, dir, len)
            .map(|(t, n)| (t, None, Some(n), None, piece.filter(|p| (p.0 - t).abs() < 1e-3).map(|p| p.1)));
        for u in units.iter().filter(|u| !u.dead && u.who() != m.owner) {
            let t = if knocked_down(u) { ray_lying(m.pos, dir, u, len) } else { ray_body(m.pos, dir, u, len) };
            if let Some(t) = t.filter(|&t| first.is_none_or(|f| t < f.0)) {
                first = Some((t, Some(u.who()), None, None, None));
            }
        }
        if let Some((e, t)) = mines.first_on(m.pos, dir, len).filter(|&(_, t)| first.is_none_or(|f| t < f.0)) {
            first = Some((t, None, None, Some(e), None));
        }
        if let Some((t, struck, surface, mine, breakable)) = first {
            m.pos += dir * t;
            let cause = if struck.is_some() { "hits a body" } else if mine.is_some() { "hits a Sentry" }
                else if breakable.is_some() { "hits a breakable object" } else { "hits the world" };
            return Tick::Blow(Blow { breakable, ..blow(m, struck, surface, mine, true, cause) });
        }
        m.pos += step;
    }
    m.age += dt;
    // the straight phase; at its end a child looks for a target (FUN_00226b80)
    if m.straight > 0.0 {
        m.straight -= dt;
        if m.straight <= 0.0 {
            m.straight = 0.0;
            if b.seeks && m.mode == 0 {
                seek(m, range, units, dt, now, log);
            }
        }
    }
    // steering (FUN_002266d0): always for a steady one; along its own heading in the straight
    // phase, after it toward its target's origin or its aim point
    let homing = b.homes || b.seeks;
    if b.steady || (homing && m.straight <= 0.0) {
        if b.seeks && m.target.is_none() && m.mode == 0 {
            seek(m, range, units, dt, now, log);
        }
        let mut dir = m.vel.normalize_or(m.heading);
        if homing && m.straight <= 0.0 {
            let d = m.aim - m.pos;
            if d.length_squared() <= RETARGET_NEAR_SQ || d.normalize_or(m.heading).dot(m.heading) < RETARGET_COS {
                if log {
                    println!("t {now:.2}: missile {} gives up {} for a point ahead", m.id,
                             if d.length_squared() <= RETARGET_NEAR_SQ { "its aim point (within 1 m)" } else { "its aim (over 60 degrees off)" });
                }
                m.aim = m.pos + m.vel * RETARGET_AHEAD;
                m.target = None;
                if b.seeks {
                    m.seek_in = RESEEK;
                }
            }
            if let Some(at) = m.target.and_then(|t| origin_of(units, t)) {
                m.aim = at;
            }
            dir = (m.aim - m.pos).normalize_or(m.heading);
        }
        let speed = if b.steady { m.speed } else { m.vel.length() };
        // FUN_002266d0 turns the node along the velocity it had before this steering
        // (0x2268b4-0x22690d): the heading the next tick's 60-degree test and the seek's cone
        // read (FUN_0001fdd0, the node's -z) lags the motion by a step
        let before = m.vel;
        let along = m.vel.dot(dir);
        m.vel -= (m.vel - dir * along) * STEER_ACROSS;
        if (speed - along).abs() > STEER_SLACK {
            m.vel += dir * ((speed - along) * STEER_SPEED);
        }
        m.heading = before.normalize_or(m.heading);
    }
    // the timers: life, then the second timer
    if m.life > 0.0 {
        m.life -= dt;
    } else if m.after > 0.0 {
        m.after -= dt;
    }
    if m.mode == 1 && m.vel.y < APOGEE_VY && m.straight == 0.0 {
        return Tick::Split;
    }
    // the proximity fuse: every FUSE_EVERY s, a body (its target, if it has one, else any
    // hostile) within reach, to its origin, that it has passed (on it, or behind its velocity).
    // Only for a player's missile (FUN_00112800), and one with a target or a side other than
    // NEUTRAL (0x22731c-0x22732d): a child without the split's target has none
    let mut fused = None;
    if b.fuse && m.human && (m.target.is_some() || m.side != NEUTRAL) {
        if m.fuse_clock < FUSE_EVERY {
            m.fuse_clock += dt;
        } else {
            m.fuse_clock = 0.0;
            let reach = radius * FUSE_REACH;
            fused = units.iter().filter(|u| !u.dead && match m.target { Some(t) => u.who() == t, None => hostile(m.side, u.team) })
                .find(|u| {
                    let off = feet(u) - m.pos;
                    let d2 = off.length_squared();
                    d2 <= reach * reach && (d2 <= 0.0 || (m.vel.length_squared() > 1.2e-7 && off.dot(m.vel) <= 0.0))
                }).map(|u| name(u));
            if let Some(who) = &fused {
                if log {
                    println!("t {now:.2}: missile {} fuse: {who} within {reach:.2} m, passed", m.id);
                }
                m.life = 0.0;
                m.after = 0.0;
            }
        }
    }
    if m.life <= 0.0 {
        if m.mode == 1 || m.mode == 2 {
            return Tick::Split;
        }
        if m.after <= 0.0 {
            if b.explode_at_end || fused.is_some() {
                let cause = if fused.is_some() { "its fuse" } else { "its timers run out" };
                return Tick::Blow(blow(m, None, None, None, false, cause));
            }
            return Tick::Blow(Blow { cause: "ends without a blast", ..blow(m, None, None, None, false, "") });
        }
    }
    Tick::Flying
}

/// The missile going off where it is now (see `Blow`), its way its velocity (the hit record's
/// +0x1c, FUN_002278e0).
fn blow(m: &Missile, struck: Option<Who>, surface: Option<Vec3>, mine: Option<Entity>, collision: bool, cause: &'static str) -> Blow {
    Blow { id: m.id, weapon: m.weapon, owner: m.owner, team: m.team, human: m.human, at: m.pos, dir: m.vel.normalize_or(m.heading), struck, surface, mine,
           breakable: None, collision, cause, from: m.from, age: m.age }
}

/// Where along a ray (within `max`) it meets a body lying knocked down: its ragdoll's bones,
/// each taken as a ball of LYING_RADIUS (the game's missile strikes the ragdoll's own bodies,
/// FUN_002278e0; the shape is the demo's).
fn ray_lying(origin: Vec3, dir: Vec3, u: &Player, max: f32) -> Option<f32> {
    let r = &u.knock.as_ref()?.ragdoll;
    let turn = Quat::from_rotation_y(r.yaw);
    r.pos.iter().filter_map(|&p| {
        let off = origin - (r.origin + turn * p);
        let b = off.dot(dir);
        let disc = b * b - (off.length_squared() - LYING_RADIUS * LYING_RADIUS);
        let t = -b - disc.max(0.0).sqrt();
        (disc >= 0.0 && t > 0.0 && t < max).then_some(t)
    }).min_by(|a, b| a.total_cmp(b))
}

/// FUN_00226b80: every SEEK_EVERY s, the most central hostile within the range and SEEK_CONE
/// degrees, with nothing in between, becomes its target. By its own side: a NEUTRAL child's
/// finds nobody (see NEUTRAL).
fn seek(m: &mut Missile, range: f32, units: &[&Player], dt: f32, now: f32, log: bool) {
    m.seek_in -= dt;
    if m.seek_in > 0.0 {
        return;
    }
    m.seek_in = SEEK_EVERY;
    if let Some(&who) = search(units, m.pos, m.heading, range, SEEK_CONE, 1, m.side).first() {
        if log && m.target != Some(who) {
            println!("t {now:.2}: missile {} seeks: {}", m.id, units.iter().find(|u| u.who() == who).map_or(String::new(), |u| name(u)));
        }
        m.target = Some(who);
        if let Some(at) = origin_of(units, who) {
            m.aim = at;
        }
    }
}

/// FUN_00227f80 (WM_MIRV_ON_EXPLODE / ON_APOGEE): the children of a parent whose life is out.
/// Each leaves beside it (`r_parent + r_child` m off along a side axis rolled at random, then
/// 360 / count degrees on for the next), turned spread-max x 90 degrees toward that side, at the
/// parent's speed, living range / speed x 0.85-1.15. If the parent seeks, the split's hostiles
/// (up to the count, within the range and 30 degrees) are dealt out to them in turn; without
/// any, each aims 200 m along the parent's heading.
#[allow(clippy::too_many_arguments)]
fn split(m: &Missile, b: &MeshBullet, range: f32, units: &[&Player], radii: (f32, f32), state: &mut MissileState, now: f32, log: bool) -> Vec<Missile> {
    let count = b.mirv_count.min(MIRV_MAX).max(0) as usize;
    if count == 0 {
        return vec![];
    }
    // the parent's node: its -z (FUN_00227f80 reads its frame)
    let fwd = m.heading;
    let up0 = if fwd.y.abs() > 0.999 { Vec3::X } else { Vec3::Y };
    let right = fwd.cross(up0).normalize();
    let up = right.cross(fwd).normalize();
    let mut roll = state.random() * std::f32::consts::TAU;
    let step = std::f32::consts::TAU / count as f32;
    let targets: Vec<Who> = if b.seeks {
        search(units, m.pos, fwd, range, SPLIT_CONE, count, m.side)
    } else if b.homes {
        m.target.into_iter().collect()
    } else {
        vec![]
    };
    let turn = (b.spread[1] * SPREAD_TURN).to_radians();
    let start = range / m.speed.max(1e-3);
    let mut out = vec![];
    for i in 0..count {
        let life = start * (1.0 - CHILD_LIFE + 2.0 * CHILD_LIFE * state.random());
        let side = up * roll.cos() + right * roll.sin();
        roll += step;
        let pos = m.pos + side * (radii.0 + radii.1);
        let heading = (fwd * turn.cos() + side * turn.sin()).normalize();
        let target = (!targets.is_empty()).then(|| targets[i % targets.len()]);
        let aim = target.and_then(|t| origin_of(units, t)).unwrap_or(m.pos + fwd * SPLIT_AHEAD);
        state.next += 1;
        out.push(Missile {
            id: state.next, weapon: m.weapon, object: if b.mirv_object != 0 { b.mirv_object } else { m.object }, owner: m.owner, team: m.team,
            side: NEUTRAL, human: m.human, pos, vel: heading * m.speed, heading, speed: m.speed, life, start, straight: start * b.straight, after: b.after,
            seek_in: 0.0, fuse_clock: 0.0, target, aim, mode: 0, clock: m.clock, from: m.from, age: m.age, trails: vec![],
        });
    }
    if log {
        let names: Vec<String> = targets.iter().map(|&t| units.iter().find(|u| u.who() == t).map_or(String::new(), |u| name(u))).collect();
        println!("t {now:.2}: missile {} splits {:.1} m from its launch, {:.3} s after it, into {} (offset {:.2} m, turned {:.1} degrees); targets {:?}",
                 m.id, m.pos.distance(m.from), m.age, out.iter().map(|c| format!("{} (life {:.3} s)", c.id, c.life)).collect::<Vec<_>>().join(", "),
                 radii.0 + radii.1, turn.to_degrees(), names);
    }
    out
}

/// A missile's entity: its model and its trail's effects riding on it (the trail's user
/// parameter starts at its life / start), its trail's sounds heard from where it is.
fn spawn_missile(commands: &mut Commands, kit: Option<&MissileKit>, ale: Option<&mut bf_viewer::ale_fx::AleAssets>,
                 mut m: Missile, listener: Vec3, sounds: &mut Vec<(u32, f32)>) {
    let place = Transform::from_translation(m.pos).looking_to(m.heading, Vec3::Y);
    let root = commands.spawn((place, Visibility::default(), Name::new("missile"))).id();
    if let Some(kit) = kit {
        for (mesh, mat, at) in &kit.parts {
            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), *at, ChildOf(root)));
        }
        if let Some(ale) = ale {
            for (i, &effect) in kit.trail.effects.iter().enumerate() {
                if let Some(fx) = ale.cached_recorded_facing(effect) {
                    let sp = if m.start > 0.0 { (m.life / m.start).clamp(0.0, 1.0) } else { 1.0 };
                    let run = bf_viewer::ale_fx::AleEffect::new(fx, sp, 0x5EED_0000 ^ m.id.wrapping_mul(7919) ^ i as u32).with_lod_sphere(kit.trail.bounds);
                    let e = commands.spawn((Transform::default(), Visibility::default(), run, ChildOf(root))).id();
                    m.trails.push(e);
                }
            }
        }
        let volume = SOUND_VOLUME * (1.0 - m.pos.distance(listener) / SOUND_REACH).clamp(SOUND_FLOOR, 1.0);
        for &(id, _, _, _) in &kit.trail.sounds {
            sounds.push((id, volume));
        }
    }
    commands.entity(root).insert(m);
}

/// This frame's launches (FUN_0022fb50): each shooter's mesh-projectile shots become missiles,
/// with the hostile character on their aiming line (before the world) as the target.
#[allow(clippy::too_many_arguments)]
fn launch_missiles(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
                   kits: Res<MissileKits>, mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut state: ResMut<MissileState>) {
    let log = std::env::var("BF_MISSILE_LOG").is_ok();
    let listener = player.position;
    let now = player.sim_time;
    let human = player.who();
    let mut sounds = vec![];
    let mut out = vec![];
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let (who, team) = (u.who(), u.team);
        out.extend(u.launches.drain(..).map(|l| (l, who, team)));
    }
    if out.is_empty() {
        return;
    }
    let units: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).collect();
    for (l, owner, team) in out {
        let Some(def) = game.0.weapons.get(&l.weapon) else { continue };
        let Some(b) = &def.mesh_bullet else { continue };
        // weapon +0x360: the character on the shooter's line, before the world, if it's hostile
        // (an inference: the character under the crosshair; a squadmate in front of a hostile
        // hides it, as it would the crosshair's)
        let (from, ray) = l.look;
        let wall = ray_hit(from, ray, 150.0).unwrap_or(150.0);
        let target = units.iter().filter(|u| !u.dead && u.who() != owner)
            .filter_map(|u| ray_body(from, ray, u, wall).map(|t| (t, u.who(), u.team))).min_by(|a, b| a.0.total_cmp(&b.0))
            .filter(|x| x.2 != team).map(|x| x.1);
        let speed = def.bullet_speed.max(1e-3);
        let start = def.range / speed;
        let dir = l.dir.normalize_or(Vec3::NEG_Z);
        state.next += 1;
        let m = Missile {
            id: state.next, weapon: l.weapon, object: b.object, owner, team, side: team, human: owner == human, pos: l.origin, vel: dir * speed, heading: dir,
            speed, life: start, start, straight: start * b.straight, after: b.after, seek_in: 0.0, fuse_clock: 0.0,
            target, aim: target.and_then(|t| origin_of(&units, t)).unwrap_or(l.aim), mode: b.mirv_mode, clock: 0.0, from: l.origin, age: 0.0, trails: vec![],
        };
        if log {
            println!("t {now:.2}: missile {} launched ({}) from {:.2} along {:.3} at {speed} m/s: life {start:.3} s, straight {:.3} s, then {} s; mirv {} x mode {}; target {:?}, aim {:.2}",
                     m.id, def.label, m.pos, dir, m.straight, m.after, b.mirv_count, b.mirv_mode,
                     target.and_then(|t| units.iter().find(|u| u.who() == t)).map(|u| name(u)), m.aim);
        }
        spawn_missile(&mut commands, kits.0.get(&b.object).and_then(|k| k.as_ref()), ale.as_deref_mut(), m, listener, &mut sounds);
    }
    drop(units);
    player.sound_queue.extend(sounds);
}

/// The missiles in flight, a 30 Hz game frame at a time (shots::TICK): their moves, steering,
/// timers, fuses, splits and ends; then what went off.
#[allow(clippy::too_many_arguments)]
fn fly_missiles(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
                kits: Res<MissileKits>, mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut state: ResMut<MissileState>,
                mut missiles: Query<(Entity, &mut Missile, &mut Transform)>, mut trails: Query<&mut bf_viewer::ale_fx::AleEffect>,
                (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
                (mut mines, mut loose, mut objects, mut tint, mut hits): (ResMut<super::sentry::MineTargets>, ResMut<super::pickups::Blasts>,
                                                                ResMut<super::scenery::ObjectBlasts>, ResMut<ScreenTint>,
                                                                ResMut<super::scenery::ObjectHits>),
                test: Option<Res<super::testmap::TestMap>>, mut shaker: ResMut<CameraShake>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_MISSILE_LOG").is_ok();
    let now = player.sim_time;
    let listener = player.position;
    let mut blows = vec![];
    let mut children = vec![];
    {
        let units: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).collect();
        for (e, mut m, mut tr) in &mut missiles {
            let Some(def) = game.0.weapons.get(&m.weapon) else { continue };
            let Some(b) = def.mesh_bullet.clone() else { continue };
            let (range, radius) = (def.range, def.blast_radius);
            m.clock += dt;
            let mut gone = false;
            while m.clock >= super::shots::TICK && !gone {
                m.clock -= super::shots::TICK;
                match tick(&mut m, &b, range, radius, &units, &mines, super::shots::TICK, now, log) {
                    Tick::Flying => {}
                    Tick::Split => {
                        let radii = |o: u32| kits.0.get(&o).and_then(|k| k.as_ref()).map_or(0.0, |k| k.radius);
                        let child = if b.mirv_object != 0 { b.mirv_object } else { m.object };
                        children.extend(split(&m, &b, range, &units, (radii(m.object), radii(child)), &mut state, now, log));
                        gone = true;
                    }
                    Tick::Blow(bl) => {
                        blows.push(bl);
                        gone = true;
                    }
                }
            }
            if gone {
                commands.entity(e).despawn();
                continue;
            }
            // (drawn where it is by now: the time not yet run in ticks carries it on, so it
            // doesn't stand still every other frame at 60 fps)
            tr.translation = m.pos + m.vel * m.clock;
            if m.heading.length_squared() > 0.5 {
                tr.look_to(m.heading, Vec3::Y);
            }
            // FUN_00228c10: the trail's user parameter is life / start
            let sp = if m.start > 0.0 { (m.life / m.start).clamp(0.0, 1.0) } else { 1.0 };
            for &t in &m.trails {
                if let Ok(mut fx) = trails.get_mut(t) {
                    fx.sp = sp;
                }
            }
        }
    }
    let mut sounds = vec![];
    for c in children {
        let kit = kits.0.get(&c.object).and_then(|k| k.as_ref());
        spawn_missile(&mut commands, kit, ale.as_deref_mut(), c, listener, &mut sounds);
    }
    player.sound_queue.extend(sounds);
    let instant = test.as_ref().is_some_and(|t| t.instant_kill);
    for bl in blows {
        go_off(&mut commands, &mut game.0, ale.as_deref_mut(), &mut images, &mut materials, bl, &mut player, &mut squad, &mut state,
               &mut mines, &mut loose, &mut objects, &mut tint, &mut hits, &mut shaker, instant, now, log);
    }
}

/// A missile going off (`Blow`): unless it ended without a blast, its explosion effect type
/// (the ALE effects and light at the point, turned along its way for a collision, and its
/// sounds), the bullet's impact sound, the direct hit's damage on the body it struck
/// (FUN_0022c4d0: a roll of Damage min..max, x 0.7 on its own side, x its damage-type factor),
/// a Sentry it struck set off, the blast object (its own roll, after 0.05-0.25 s), loose pickups
/// thrown and the scenery's share, and the scorch on the ground under the blast point (within
/// its radius), laid DECAL_DELAY later as the grenades' are.
#[allow(clippy::too_many_arguments)]
fn go_off(commands: &mut Commands, game: &mut Game, ale: Option<&mut bf_viewer::ale_fx::AleAssets>, images: &mut Assets<Image>,
          materials: &mut Assets<StandardMaterial>, bl: Blow, player: &mut Player, squad: &mut Squad, state: &mut MissileState,
          mines: &mut super::sentry::MineTargets, loose: &mut super::pickups::Blasts, objects: &mut super::scenery::ObjectBlasts, tint: &mut ScreenTint,
          hits: &mut super::scenery::ObjectHits, shaker: &mut CameraShake, instant: bool, now: f32, log: bool) {
    let Some(def) = game.weapons.get(&bl.weapon).cloned() else { return };
    let struck_name = bl.struck.and_then(|w| std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.who() == w)).map(|u| name(u));
    if log {
        println!("t {now:.2}: missile {} {}{} at {:.2}, {:.1} m from its launch, {:.3} s after it", bl.id, bl.cause,
                 struck_name.as_ref().map_or(String::new(), |n| format!(" ({n})")), bl.at, bl.at.distance(bl.from), bl.age);
    }
    if bl.cause == "ends without a blast" {
        return;
    }
    // where it goes off: the missile's centre when it touched, its physics radius (the bullet's
    // radius, 0.1 m) off the surface it struck along the surface's normal, else back along its way
    let off = def.mesh_bullet.as_ref().map_or(0.1, |b| b.radius);
    let at = match bl.surface {
        Some(n) => bl.at + n.normalize_or(-bl.dir) * off,
        None if bl.collision => bl.at - bl.dir * off,
        None => bl.at,
    };
    let fx = game.effect_type_defs.get(&def.blast_effect).cloned().unwrap_or_default();
    if let Some(ale) = ale {
        for &effect in fx.effects.iter().chain([&fx.light]).filter(|&&e| e != 0) {
            if let Some(c) = ale.load_recorded_facing(game, images, materials, effect) {
                let life = c.duration();
                let place = if bl.collision { Transform::from_translation(at).looking_to(bl.dir, Vec3::Y) } else { Transform::from_translation(at) };
                let seed = (at.x * 977.0 + at.z * 131.0) as u32 ^ bl.id;
                // (at its distance LOD from the camera, by the type's bounding sphere: the flash
                // goes beyond ~94 m, see ale_fx's LOD_TAN)
                let run = bf_viewer::ale_fx::AleEffect::once(c, EXPLOSION_SP, seed).with_lod_sphere(fx.bounds);
                commands.spawn((place, Visibility::default(), run, AleExpire(life)));
            }
        }
    }
    // the player's view shakes (FUN_00228c70, from the follow camera's eye)
    shake(shaker, at, camera_pose(player, 0.0).0, &def);
    let volume = (1.0 - at.distance(player.position) / BLAST_HEARD).clamp(BLAST_QUIET, 1.0);
    for &(id, delay, _, _) in &fx.sounds {
        if delay > 0.0 {
            state.sounds.push((delay, id, volume));
        } else {
            player.sound_queue.push((id, volume));
        }
    }
    if def.impact_sound != 0 {
        player.sound_queue.push((def.impact_sound, volume));
    }
    // a breakable object struck: a roll of Damage min..max (FUN_0022c4d0 on the object struck;
    // the scenery applies its damage-type factor), then the blast's share too
    if let Some(target) = bl.breakable {
        let (lo, hi) = (def.damage_min.min(def.damage), def.damage);
        let roll = lo + (hi - lo) * state.random();
        hits.0.push(super::scenery::ObjectHit { target, amount: roll, damage_type: def.damage_type, at: bl.at, dir: bl.dir });
        if log {
            println!("t {now:.2}: missile {} direct hit: breakable {target} takes a roll of {roll:.1}", bl.id);
        }
    }
    if let Some(e) = bl.mine {
        mines.strike(e, 0.0, 1.0, now);
    }
    let (lo, hi) = (def.damage_min.min(def.damage), def.damage);
    // the direct hit (FUN_0022c4d0 with no blast object): a roll, now, x the hit-location
    // factor (FUN_00125940 at 0x227a03: the victim type's damage-multiplier x the struck bone's
    // Damage; the missile's own +0x1cc is 1.0)
    if let Some((u, controlled)) = bl.struck.and_then(|w| unit_mut(player, squad, w)) {
        let roll = lo + (hi - lo) * state.random();
        let own = if u.team == bl.team { FRIENDLY } else { 1.0 };
        let (factor, bone) = hit_factor(u, game, bl.at);
        let amount = roll * factor * own * game.damage_factor(CHARACTERS[u.character], def.damage_type);
        // (the test map's instant kill: anyone but the player, from the player's missiles)
        let amount = if instant && bl.human && !controlled { u.health.max(amount) } else { amount };
        let push = Vec3::new(bl.dir.x, 0.0, bl.dir.z).normalize_or(Vec3::Z) * HIT_PUSH.0 + Vec3::Y * HIT_PUSH.1;
        let before = u.health;
        hurt_still(u, game, amount, push, bl.at);
        if controlled && u.health < before {
            tint.0 = 0.0;
        }
        if log {
            println!("t {now:.2}: missile {} direct hit: {} takes {amount:.1} (roll {roll:.1}, x{factor:.2} at bone {}{}) -> {:.1} / {:.0}", bl.id, name(u),
                     bone.map_or("-".to_string(), |b| format!("h_{b:08x}")), if own < 1.0 { ", own side x0.7" } else { "" }, u.health, u.max_health);
        }
    }
    let radius = def.blast_radius;
    if radius <= 0.0 || def.damage <= 0.0 {
        return;
    }
    // the blast object (FUN_0022bfc0): its own roll, dealt after its delay (`deal_blasts`)
    let roll = lo + (hi - lo) * state.random();
    let left = BLAST_DELAY + BLAST_DELAY_MORE * state.random();
    let eye = bl.at + bl.surface.unwrap_or(Vec3::Y) * COVER_LIFT;
    state.blasts.push(PendingBlast { left, at, eye, weapon: bl.weapon, roll, owner: bl.owner, team: bl.team, id: bl.id });
    if log {
        println!("t {now:.2}: missile {} blast: roll {roll:.1}, dealt in {left:.3} s within {radius} m", bl.id);
    }
    loose.0.push((at, radius));
    // objects other than characters: the blast object's entry for them is its own centre
    // (FUN_00224a90), so FUN_0022c4d0 sees them at distance 0: the whole roll with Damage
    // h_011cb154 true, half without, and no falloff (the grenades' scenery keeps its falloff:
    // play_grenade.rs, not checked against this path)
    let share = if def.blast_core { roll } else { roll * BLAST_SHARE };
    objects.0.push(super::scenery::ObjectBlast { left, at, radius, damage: share, damage_type: def.damage_type, label: def.label.clone(), flat: true });
    // the scorch: every blast, a timeout's and the fuse's too (FUN_00227760 makes a blast object
    // as well). A blast weapon's direct hit lays none (the handler 0x22d0b0 calls FUN_0022ca20 on
    // a hit only for a Damage radius of 0.05 m or less: bullet holes); the blast's pass over what
    // it reaches does (FUN_00224770 back into the handler), the static world's entry sitting at
    // the blast's centre with its normal straight up (FUN_00224a90), so the bullet's decal is
    // laid round the blast point and projected down: on the ground under it, also after a body
    // hit (take03: under the enemy struck, from f256) and a wall hit. The demo's flat decal goes
    // on the ground below, if that's within the radius (the projection's reach isn't traced).
    // The pass comes with the blast's damage, 0.05-0.25 s after the explosion; the demo lays it
    // DECAL_DELAY (0.7 s) after, as the grenades' scorch: laid as early as the code does, the
    // dark decal shows through the demo's paler added fireball as a hard dark disc (#104). The
    // takes show it from ~0.85-1.0 s, once their fireball clears
    if def.decal != 0 {
        let ground = floor_y(at.x, at.z, at.y + 0.5);
        if at.y - ground <= radius {
            state.scorches.push((DECAL_DELAY, def.decal, at));
        }
    }
}

/// `hurt` without the knockdown it may start (play.rs's rule for a hit of KNOCKDOWN_DAMAGE or
/// more): no missile hit or blast in the takes moved its victim (Tex hurt by his own blasts,
/// takes 10-16; frozen enemies hit directly and by blasts, takes 06, 18, 19, 23). The game's
/// knockdown rule isn't traced. A kill still throws the body (the death push).
fn hurt_still(u: &mut Player, game: &Game, amount: f32, push: Vec3, at: Vec3) {
    let asked = u.knock_request.is_some();
    hurt(u, game, amount, HURT_CHATTER, push, at, -1);
    if !asked && !u.dead && u.knock_request.take().is_some() && std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("  (not knocked down after all: a missile's hit, play_missile.rs)");
    }
}

/// FUN_00125940, a direct hit's factor on a character: its type's damage-multiplier (the
/// campaign squad's 0.62) x the struck bone's Damage (its motion set's `<collision>` list: the
/// head x5, upper arms x0.8, forearms and calves x0.5, the hips, spine and thighs x1). The
/// struck bone: the one of that list nearest the contact on the posed skeleton (the ragdoll's
/// while knocked down), each taken as the line from its joint to its first child's (the calf
/// from the knee to the ankle), where the game reads the physics body the missile met
/// (FUN_00115000); the shape is the demo's. Blast shares take none of it (FUN_00224a90 passes
/// 0). Returns the factor and the bone.
fn hit_factor(u: &Player, game: &Game, at: Vec3) -> (f32, Option<u32>) {
    let who = CHARACTERS[u.character % CHARACTERS.len()];
    let multiplier = game.character_damage_multiplier.get(who).copied().unwrap_or(1.0);
    let (Some(bones), Some(l)) = (game.character_hit_bones.get(who), u.loaded.as_ref()) else { return (multiplier, None) };
    let place = |i: usize| -> Option<Vec3> {
        match &u.knock {
            Some(k) => k.ragdoll.pos.get(i).map(|&p| k.ragdoll.origin + Quat::from_rotation_y(k.ragdoll.yaw) * p),
            None => u.last_world.get(i).map(|m| u.position + Vec3::Y * (u.height + u.lift_now) + Quat::from_rotation_y(u.yaw) * m.w_axis.truncate()),
        }
    };
    // the distance from the contact to a bone's line (its joint to its first child's)
    let reach = |i: usize| -> Option<f32> {
        let a = place(i)?;
        let Some(b) = l.model.parent.iter().position(|&p| p == Some(i)).and_then(|j| place(j)) else { return Some(a.distance(at)) };
        let ab = b - a;
        let k = if ab.length_squared() > 1e-8 { ((at - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
        Some((a + ab * k).distance(at))
    };
    bones.iter().filter_map(|&(bone, damage)| {
        let i = l.model.bones.iter().position(|&b| b == bone)?;
        Some((reach(i)?, damage, bone))
    }).min_by(|a, b| a.0.total_cmp(&b.0)).map_or((multiplier, None), |(_, damage, bone)| (multiplier * damage, Some(bone)))
}

/// FUN_00228c70, every blast of a missile (its only callers, 0x227867 and 0x227b1d, are the
/// missile's two explosion paths: a grenade's blast doesn't shake the view through it): the player's
/// view shakes if the blast is within SHAKE_REACH x the Damage radius of the camera (31.5 m for
/// the Therm Sweeper's 4.5): fully within half that, then less to nothing at the edge (and not
/// at all under SHAKE_LEAST of it), by the weapon's Damage x SHAKE_PER_DAMAGE (at most 1: 0.74
/// for the Therm Sweeper's 96), held SHAKE_HOLD s and falling over SHAKE_FALL x the falloff
/// (play_shake.rs). The takes: take03's blast 6 m out turned the view +-5-10 of 960 units for
/// ~1 s, take04's at 25 m +-1-5 for ~0.7 s.
fn shake(shake: &mut CameraShake, at: Vec3, eye: Vec3, def: &WeaponDef) {
    let reach = def.blast_radius * SHAKE_REACH;
    if reach <= 1.0 {
        return;
    }
    let half = reach * 0.5;
    let strength = (def.damage * SHAKE_PER_DAMAGE).clamp(0.0, 1.0);
    let d = at.distance(eye);
    if d >= reach {
        return;
    }
    let falloff = if d <= half { 1.0 } else { 1.0 - (d - half) / half };
    if d > half && falloff <= SHAKE_LEAST {
        return;
    }
    shake.kick(strength * falloff, SHAKE_HOLD, falloff * SHAKE_FALL);
}

/// The blasts whose delay is out (FUN_00224770 -> FUN_0022c4d0): every living body within the
/// Damage radius of the blast with nothing of the world between it and the body's middle takes
/// the share (half the roll, flat; Damage h_011cb154 true: the whole roll to half the radius,
/// then (1 - 2 (d / r - 0.5))^2 of it), x 0.7 on the shooter's own side (the shooter too), x its
/// damage-type factor; the player's view goes red. Then the scorches and sounds due.
#[allow(clippy::too_many_arguments)]
fn deal_blasts(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut state: ResMut<MissileState>,
               mut tint: ResMut<ScreenTint>, mut decals: ResMut<super::fx::DecalRequests>, test: Option<Res<super::testmap::TestMap>>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_MISSILE_LOG").is_ok();
    let now = player.sim_time;
    let instant = test.as_ref().is_some_and(|t| t.instant_kill);
    let mut due = vec![];
    state.blasts.retain_mut(|b| {
        b.left -= dt;
        if b.left <= 0.0 {
            due.push(*b);
        }
        b.left > 0.0
    });
    for b in due {
        let Some(def) = game.0.weapons.get(&b.weapon) else { continue };
        let radius = def.blast_radius;
        let human = player.who() == b.owner;
        for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
            if u.dead {
                continue;
            }
            // the nearest point of the body's cylinder
            let f = feet(u);
            let across = (Vec2::new(b.at.x - f.x, b.at.z - f.z).length() - BLAST_BODY_REACH).max(0.0);
            let up = b.at.y - b.at.y.clamp(f.y, f.y + BODY_HEIGHT);
            let d = across.hypot(up);
            if d > radius {
                continue;
            }
            let middle = f + Vec3::Y * BODY_MIDDLE;
            let to = middle - b.eye;
            if to.length() > 0.05 && ray_hit(b.eye, to.normalize(), to.length()).is_some() {
                if log {
                    println!("t {now:.2}: missile {} blast: {} {d:.1} m away, behind cover (from {:.2} to {middle:.2}: met at {:.2} m)", b.id, name(u), b.eye,
                             ray_hit(b.eye, to.normalize(), to.length()).unwrap_or(0.0));
                }
                continue;
            }
            let share = if def.blast_core {
                // (the character's size term, its skill's vtable +0xe0, is 0 for the squad)
                let x = d / radius;
                if x <= 0.5 { b.roll } else { (1.0 - 2.0 * (x - 0.5)).clamp(0.0, 1.0).powi(2) * b.roll }
            } else {
                b.roll * BLAST_SHARE
            };
            let own = if u.team == b.team { FRIENDLY } else { 1.0 };
            let amount = share * own * game.0.damage_factor(CHARACTERS[u.character], def.damage_type);
            let amount = if instant && human && k != 0 { u.health.max(amount) } else { amount };
            let away = Vec3::new(f.x - b.at.x, 0.0, f.z - b.at.z).normalize_or(Vec3::X);
            let before = u.health;
            hurt_still(u, &game.0, amount, away * BLAST_PUSH.0 + Vec3::Y * BLAST_PUSH.1, middle);
            if k == 0 && u.health < before {
                tint.0 = 0.0;
            }
            if log {
                println!("t {now:.2}: missile {} blast: {} {d:.1} m away takes {amount:.1} ({}{}) -> {:.1} / {:.0}", b.id, name(u),
                         if def.blast_core { "core rule" } else { "half the roll" }, if own < 1.0 { ", own side x0.7" } else { "" },
                         u.health, u.max_health);
            }
        }
    }
    state.scorches.retain_mut(|(left, decal, at)| {
        *left -= dt;
        if *left <= 0.0 {
            decals.0.push((*decal, *at, GRENADE_DECAL_SCALE));
        }
        *left > 0.0
    });
    let p = &mut *player;
    state.sounds.retain_mut(|(left, id, volume)| {
        *left -= dt;
        if *left <= 0.0 {
            p.sound_queue.push((*id, *volume));
        }
        *left > 0.0
    });
}
