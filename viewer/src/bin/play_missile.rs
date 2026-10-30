//! Mesh projectiles (bullet-type 2, BTYPE_MESH_PROJECTILE; issue #73, the Therm Sweeper, #72,
//! the Sweeper V, and #239, the lobbed ones): a missile that flies, steers or falls, splits
//! (MIRV) and explodes, from default.xbe's projectile class (0x226000-0x229000) and its weapon
//! type's damage handler (0x22d0b0), checked against the xemu takes in todo/73-therm-sweeper/,
//! todo/72-sweeper-v/ (the tester's specs there have the measurements) and todo/55-ikhan-gpl/.
//!
//!  - **Launch** (FUN_0022f1d0 case 2 -> FUN_0022fb50): one projectile object, the bullet's
//!    mesh-name (h_e9327576: model h_07d84a52, attached effect h_194dc78e with its sound
//!    0021996c), at the muzzle, along the shot (after the accuracy turn), at the bullet's speed
//!    (FUN_00227ce0), given the weapon's target and aim point (FUN_00226360). Its life is range
//!    / speed (FUN_002324c0 through the holder's skill hook, which leaves the range alone for
//!    the squad: 20 / 30 = 0.667 s), its straight phase h_ea5327e6 of that (0.1 s), then its
//!    second timer h_f0a56297 (5 s) runs (FUN_00227dd0).
//!  - **Each game frame** (FUN_00226cc0, here in 30 Hz ticks after the physics move): steered
//!    (FUN_002266d0) along its velocity in the straight phase, then toward its target's root
//!    (the hips, `root_point`) or its aim point. It gives its aim up within 1 m of it or over 60
//!    degrees off its heading, which stays the launch's (a child's, the split's) direction all
//!    its flight; then life and the second timer run down, and the proximity fuse looks.
//!  - **The split** (WM_MIRV_ON_EXPLODE, FUN_00227f80), when its life is out: the parent goes
//!    without a blast; `<mirv>` h_fe04300d children (2) leave beside it, each turned spread-max x
//!    90 degrees (9) off its velocity at a random roll, after the hostiles the split finds within
//!    the range and 30 degrees, else 200 m along the parent's velocity. A child is NEUTRAL (team
//!    2): its own searches find nobody, and without the split's target it has no fuse.
//!  - **Hitting something** (the collision callback FUN_002278e0 -> 0x22d0b0): the world, a
//!    body (standing, or lying knocked down), a breakable object or a Sentry. The body or
//!    object struck takes a roll of Damage min..max (FUN_0022c4d0; a character's x its type's
//!    damage-multiplier x the struck hit part's bone's Damage, FUN_00125940, then the victim's
//!    difficulty step: play_damage.rs), a blast object goes off at the missile's centre
//!    (FUN_0022bfc0) and the explosion effect type plays (FUN_0022cfc0). A child whose second
//!    timer runs out, or whose fuse fires, goes off the same way where it is, with no direct hit
//!    (FUN_00227760).
//!  - **The blast** (FUN_0022bfc0, dealt once by FUN_00224770 -> FUN_0022c4d0): its own roll,
//!    after 0.05-0.25 s, to every body its physics ball meets (a hit part within its radius and
//!    `BLAST_PART_REACH` more) with nothing of the world in between (a body behind cover isn't
//!    touched at all, FUN_00224a90): half of it anywhere inside (Damage
//!    h_011cb154 false), x 0.7 for the shooter's own side (FUN_0004b9c0), x the body's
//!    difficulty step and damage-type factor (play_damage.rs); the scenery's share is the same,
//!    flat, with no step. Its pass lays the scorch (bullet h_06a27365) under the blast point,
//!    and every blast shakes the player's view (FUN_00228c70, play_shake.rs). No hit or blast
//!    knocks its victim down (`hurt_still`).
//!
//! Every mesh projectile flies here (`flies`). Those steered at a steady speed (h_0b03ece5) are
//! the Therm Sweeper and the Sweeper V, the only two bullet-type 2 weapons with it in every
//! level's data (#72 spec §2). The Sweeper V's has no `<mirv>` (count and mode 0), no homing or
//! seeking and no second timer, so the same tick makes it the first missile that doesn't split:
//! it flies straight along its launch at 60 m/s (FUN_002266d0 steering it along its own
//! velocity, flag 4 alone) and goes off where it strikes, at its fuse, or where it is when its
//! life (range / speed: 200 / 60 = 3.33 s) runs out (h_0976a9dc, FUN_00227760).
//!
//!  - **Lobbed** (#239): the others, the iKhan-GPL's round (and the hounds' breath, which the
//!    demo keeps on the plain shot: `flies`), aren't steered at all: without flag 4, and without
//!    homing or seeking, FUN_00226cc0 skips FUN_002266d0 (0x226dd9-0x226dfc), so the body keeps
//!    the speed FUN_00227ce0 gave it at the launch and falls under the Ipion world's gravity
//!    (FUN_0016a250) with its template's damping (FUN_00158cd0): `falls` (by the object type's
//!    h_11e7d864, which takes the steady missiles' bodies off the gravity), GRAVITY,
//!    SPEED_DAMPING. A collision sets it off at once, as any missile; WM_MIRV_ON_APOGEE (the
//!    iKhan-GPL's) splits it once its upward speed is under APOGEE_VY past its straight phase,
//!    its life or its fuse out, and the children leave at the parent's speed then (FUN_00227f80
//!    at 0x228748-0x228794) and fall too. An AI pitches its lobbed rounds up to come down on its
//!    target and holds fire with the target within its own blast's reach (`lob`, `lob_holds`:
//!    the demo's, untraced).
//!
//! `BF_MISSILE_LOG=1` prints each missile's launch (a lobbed one's line ends "lobbed (gravity
//! 9.81, damping 0.1 a second)"), seeks, retargets, apogee, split (with its children and their
//! targets), fuse and end (where, how far from its launch, how long after, what it struck), and
//! each body its blast reaches with the damage; `BF_MISSILE_DEBUG=1` also prints every body each
//! target search looks at (distance, its root, angle, line of sight).
//!
//! Not here: the 75 m noise for the AI's hearing (FUN_0004d490); the damage event
//! (FUN_001fe3d0).

use super::*;
use super::damage::{self, Difficulty};
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
/// velocity (the frame FUN_0001d0c0 builds from it, its -z x 200).
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
/// SPLIT_CONE degrees of the parent's velocity (FUN_002237c0 with 30.0).
const SPLIT_CONE: f32 = 30.0;
/// FUN_00227f80: at most this many children (a count over 5 is 6).
const MIRV_MAX: i64 = 6;
/// FUN_00227f80: a child's life is the bullet's range / speed x (1 - CHILD_LIFE + 2 x CHILD_LIFE
/// x U): 0.85-1.15 of it.
const CHILD_LIFE: f32 = 0.15;
/// FUN_00227f80: a child is turned off the parent's velocity by spread-max x SPREAD_TURN degrees.
const SPREAD_TURN: f32 = 90.0;
/// FUN_00226cc0: a WM_MIRV_ON_APOGEE parent splits once its upward speed is under this (m/s),
/// past its straight phase (0x2272b4-0x2272ea: the body's velocity, vtable +0x54, its y, under
/// [0x3a4f74]; the straight phase +0x230 at 0). The iKhan-GPL's round is the one mesh
/// projectile with that mode: one fired level is already falling, so it splits the tick its
/// straight phase (1.04 s) ends, if it hasn't struck anything by then.
const APOGEE_VY: f32 = 0.5;
/// FUN_0016a250: the Ipion world's gravity, (0, -9.81, 0) (0x16a4a7, 0x16a4c7; #161's reading).
/// Every body under the world's gravity controller falls at it (m/s^2): here the projectile
/// objects without h_11e7d864 (`falls`).
const GRAVITY: f32 = 9.81;
/// FUN_00158cd0 / FUN_00158f80, the body template a game object's bodies are made from (vtable
/// +0x158 for the root, +0x154 for the others; the projectile's vtable 0x3946c8 has both): its
/// 0.1 where Ipion's own template constructor (0x2b4710) puts the speed damping's 0.01 (+0x2c,
/// after the rotation-inertia minimum 0.03 at +0x28; the rotation damping's 0.01 x 3 after it),
/// so read as the share of its speed a body loses a second (#161's inference, now by the
/// constructor's layout; not followed into Ipion's step). The demo takes it as v x (1 - 0.1 dt) a
/// tick: 5% of the iKhan-GPL's speed over its half second to the ground 12 m out. Both iKhan takes
/// would fit a stronger damping, ~0.7 a second (the flight to take01's cliff 0.47-0.50 s, the
/// demo's 0.40): but the steady missiles' bodies come from the same template (0x3dcccccd at
/// FUN_00158cd0's +0x2c), and at 0.7 the Sweeper V's steering would hold ~52 m/s where #72's
/// take05 timed 59.5. Open (play.md § Missiles).
const SPEED_DAMPING: f32 = 0.1;
/// FUN_00226cc0: the proximity fuse's clock (+0x218) gains each tick's time while it's under
/// FUSE_EVERY s; on the tick it's at least that, it's cleared and the fuse looks, within the
/// Damage radius x FUSE_REACH (FUN_00227dd0 -> +0x214: 1.69 m for the Therm Sweeper's 4.5, 3.375
/// m for the Sweeper V's 9). At 30 Hz that's every 9th tick, 0.3 s (18 m of a Sweeper V's flight).
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
/// A direct hit's hit part is looked for along the missile's way into the body from this far (m)
/// before the contact (`damage::struck_bone`): the game's missile meets the part's own body
/// (FUN_002278e0), where the demo's met the cylinder round it. The demo's.
const STRUCK_BACK: f32 = 0.5;
/// ALIGN_NEUTRAL (the ALIGN_ table at 0x3bc94c): the side the projectile constructor gives
/// (+0x21c = 2 at 0x2265ec), which a MIRV child keeps (the split calls FUN_00227dd0, which
/// copies the owner's team, before it copies the owner: 0x2282bf, 0x22834a). Row 2 of 0x3ffd80
/// is hostile only to PC_NO_TEAM (7), a side no character of the demo is on: a child's own
/// searches find nobody, and its fuse needs the split's target (0x22731c-0x22732d).
const NEUTRAL: u8 = 2;
/// How far past the Damage radius a body's hit part is still met by a blast (m). The game tests
/// no distance: the blast is a physics ball of the Damage radius (FUN_0022c2e0, made the tick
/// its delay runs out by FUN_00225c40), and FUN_00224a90 walks the bodies the physics engine
/// lists as overlapping it; a character is gathered when one of its collision-bone bodies is
/// among them (todo/weapons-plan answers-batch1 §9). The demo meets the ball with the hit parts'
/// collision surfaces as drawn (`damage::part_distances`); the engine's own overlap rule isn't in
/// the game's code, so this margin is a fit to three self-damage takes: the Sweeper V's blast hurt
/// Tex six times in six at the gate 10.2 m away for a 9 m radius (todo/72 take01), the Therm
/// Sweeper's didn't at 6 m for 4.5 m (todo/73 take13) and did at 5 m (take15). In the demo's
/// geometry for those spots, Tex aiming at the blast with his arms out, his nearest part is 9.50-
/// 9.57, 5.18-5.20 and ~4.2 m from it: a margin in [0.57, 0.68), of which this is the middle.
const BLAST_PART_REACH: f32 = 0.62;
/// A body without hit parts to meet (no model or hit-part list) is met as a standing cylinder this
/// round (m) and BODY_HEIGHT tall from the feet, when its nearest point is within the radius: the
/// same three takes measured from the feet axis (10.30, 5.94 and 4.94 m in the demo's geometry,
/// the takes' 10.17, 5.9 and 4.9) want [1.30, 1.44); this is the middle. A fit, as
/// BLAST_PART_REACH.
const BLAST_BODY_REACH: f32 = 1.35;
/// Its line of sight is to the body's middle, this high (m): the demo's.
const BODY_MIDDLE: f32 = 1.0;
/// The blast's line of sight leaves from this far (m) off the surface the missile struck (along
/// its normal; up for a body or the air): the game's blast sits where the missile's centre was
/// when it touched, and a ray from the surface itself meets that surface at once. The demo's.
const COVER_LIFT: f32 = 0.25;
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
    /// the radius its splash goes by (#5, play_liquids.rs `object_enters`): its mesh archetype's
    /// h_1ab09530 (`Game::archetype_sphere`; h_13a8dfb8 0.046 m, h_07d84a52 0.17, h_e9e61b85
    /// 0.43), else `radius` (a compound's isn't in the data: the demo's)
    splash_radius: f32,
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
    /// its heading, the node's -z as its tick reads it (the 60-degree test, the seek's cone):
    /// the launch direction (the aim after the accuracy turn) for a parent, the split's for a
    /// child, all its flight. FUN_002266d0 turns only the node, and each physics step sets the
    /// node back from the body (0x159eb0), whose turn is set only at the launch and the split
    /// (FUN_00227ce0's vtable +0x50)
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
    /// whether its body is under the world's gravity (`falls`, by its object type)
    falls: bool,
    /// game time not yet run in ticks (s)
    clock: f32,
    /// where it was launched and how long ago (BF_MISSILE_LOG)
    from: Vec3,
    age: f32,
    /// its trail effects (their user parameter follows its life)
    trails: Vec<Entity>,
    /// in a liquid (its obj+0x104 above 0, play_liquids.rs `object_enters`): it splashes going in
    /// only from out of one
    wet: bool,
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

/// WTYPE_NATURAL, a weapon-type (the WTYPE_ table at 0x3bf714: 0 PISTOL, 1 RIFLE, 2 CANNON ... 6
/// DATA, 7 NATURAL): the creatures' own weapons, the hounds' Breath Attacks and the Seers'
/// Psionic Blasts (44 definitions across the levels' data, 16 distinct names).
const WTYPE_NATURAL: i64 = 7;

/// Whether a weapon's shots fly here: a mesh projectile, a bullet-type 2 with a projectile object
/// (FUN_0022f1d0 case 2 -> FUN_0022fb50, `MeshBullet`), steered at a steady speed (h_0b03ece5,
/// flag 4), with a `<mirv>` (the Therm Sweeper) or without (the Sweeper V, #72), or lobbed (#239,
/// `falls`: the iKhan-GPL). The game flies every one so; the demo leaves out the creatures'
/// (WTYPE_NATURAL): the hounds' four Breath Attacks are the other lobbed ones, and e34's Shrike
/// Hounds fire theirs (h_1492c676: 24 m/s, an 8 m blast of damage-type 4) at the squad, so
/// they keep play.rs's plain shot until their breath is checked against footage of its own
/// (#83, #130). The seven mesh projectiles of every level's data are listed in play.md §
/// Missiles.
pub fn flies(def: &WeaponDef) -> bool {
    def.mesh_bullet.is_some() && def.weapon_type != WTYPE_NATURAL
}

/// Whether a projectile of the object type `object` falls under GRAVITY with SPEED_DAMPING
/// (#239): unless its `<base>` has h_11e7d864 (`WeaponDef::no_gravity`), with which its root
/// body's maker FUN_00158cd0 takes the body off the world's gravity controller (0x158ec0 ->
/// FUN_0016a760). In every level's data that's set on the two steady missiles' objects
/// (h_e9327576, h_fa69b507) and on no other mesh projectile's: so the iKhan-GPL's round (and the
/// hounds' breath) falls, and the Therm Sweeper and the Sweeper V fly as before, which their takes
/// show with no sag (#73's take20: children climbing straight at 8.5 degrees for 168.6 m). The
/// steering doesn't stand in for it: a Sweeper V, steered along its own velocity (flag 4 alone,
/// FUN_002266d0), would bend under gravity as freely as an unsteered round. An object type the
/// map doesn't define (a gun brought in with another level's character type,
/// `Game::load_type_assets`) goes by its bullet instead: it falls unless steered at a steady
/// speed (`steady`), the same answer on every mesh projectile in the data.
fn falls(object: Option<&WeaponDef>, steady: bool) -> bool {
    object.map_or(!steady, |o| !o.no_gravity)
}

/// One physics step of a projectile that falls (`falls`), `dt` long: GRAVITY on its velocity,
/// then SPEED_DAMPING.
fn fall(vel: Vec3, dt: f32) -> Vec3 {
    (vel - Vec3::Y * GRAVITY * dt) * (1.0 - SPEED_DAMPING * dt)
}

/// Whether a weapon fires lobbed rounds: a mesh projectile that flies here (`flies`) whose
/// object falls (`falls`): the iKhan-GPL. For the AI's aim (`lob`, `lob_holds`, play.rs).
pub fn lobs(def: &WeaponDef, game: &Game) -> bool {
    flies(def) && def.mesh_bullet.as_ref().is_some_and(|b| falls(game.weapons.get(&b.object), b.steady))
}

/// Whether an AI character at `at` (its feet) holds fire with `def` at `target`: a lobbed
/// weapon (`lobs`) whose round, coming down on the target, would go off within its own blast's
/// reach of it (the Damage radius and BLAST_PART_REACH more, across). The demo's, as `lob` is:
/// with the 8 m blast round it, an iKhan-GPL's holder at the target's feet would take every share
/// its target does.
pub fn lob_holds(def: &WeaponDef, game: &Game, at: Vec3, target: Vec3) -> bool {
    def.blast_radius > 0.0 && lobs(def, game) && at.xz().distance(target.xz()) <= def.blast_radius + BLAST_PART_REACH
}

/// The steepest pitch (degrees, up or down) `lob` tries: the demo's.
const LOB_STEEPEST: f32 = 80.0;
/// `lob`'s search: the pitches it scans (LOB_STEEPEST down to up, 1 degree apart), then the
/// halvings of the bracket it found. The demo's.
const LOB_STEPS: usize = 161;
const LOB_HALVINGS: usize = 24;
/// How long (s) `lob`'s flight runs before it's given up as never getting there: well past any
/// lobbed round's life (the iKhan-GPL's 1.74 s, its bomblets' 2 s at most). The demo's.
const LOB_LONGEST: f32 = 6.0;

/// A round launched at `speed` m/s and `pitch` (radians above level) from height 0, flying as
/// `tick` flies a falling one (`fall` each game frame, then the move): its height when it has gone
/// `across` m across, or None if it doesn't get that far within LOB_LONGEST s. Its split, life
/// and the world aren't in it. For `lob`.
fn lob_height(speed: f32, pitch: f32, across: f32) -> Option<f32> {
    let mut v = Vec3::new(pitch.cos(), pitch.sin(), 0.0) * speed;
    let mut p = Vec3::ZERO;
    let mut t = 0.0;
    while t < LOB_LONGEST {
        v = fall(v, shots::TICK);
        let next = p + v * shots::TICK;
        if next.x >= across {
            let f = if next.x > p.x { (across - p.x) / (next.x - p.x) } else { 0.0 };
            return Some(p.y + (next.y - p.y) * f);
        }
        p = next;
        t += shots::TICK;
    }
    None
}

/// How far across a round launched at `speed` m/s and `pitch` comes back down to `height` m
/// (flying as `lob_height` does): the last point it's at or above it. For `lob`'s fallback.
fn lob_reach(speed: f32, pitch: f32, height: f32) -> f32 {
    let mut v = Vec3::new(pitch.cos(), pitch.sin(), 0.0) * speed;
    let mut p = Vec3::ZERO;
    let mut t = 0.0;
    while t < LOB_LONGEST {
        v = fall(v, shots::TICK);
        let next = p + v * shots::TICK;
        if next.y < height && v.y < 0.0 {
            let f = if p.y > next.y { ((p.y - height) / (p.y - next.y)).clamp(0.0, 1.0) } else { 0.0 };
            return p.x + (next.x - p.x) * f;
        }
        p = next;
        t += shots::TICK;
    }
    p.x
}

/// The way an AI character launches a lobbed round from `origin` to come down on `target`: the
/// lower of the two pitches whose flight (`lob_height`, at the round's `speed` and the demo's
/// GRAVITY and SPEED_DAMPING) passes through the target, or, out of reach, the pitch that carries
/// it farthest at the target's height (`lob_reach`). **The demo's stand-in for the game's AI
/// aim**, neither traced nor filmed: FUN_0022e5b0 turns every weapon straight onto its aim point,
/// and what the game's AI does about a lob (its aim point, or an elevation of its own) isn't
/// read. Without it an AI firing level dropped its rounds ~5 m in front of itself and its own
/// blasts killed it (e40's Shrike placement h_1f979a8a).
pub fn lob(origin: Vec3, target: Vec3, speed: f32) -> Vec3 {
    let to = target - origin;
    let across = Vec2::new(to.x, to.z).length();
    if across < 1e-3 || speed <= 0.0 {
        return to.normalize_or(Vec3::NEG_Z);
    }
    let flat = Vec3::new(to.x, 0.0, to.z) / across;
    let way = |pitch: f32| (flat * pitch.cos() + Vec3::Y * pitch.sin()).normalize();
    let pitch_at = |k: usize| (-LOB_STEEPEST + 2.0 * LOB_STEEPEST * k as f32 / (LOB_STEPS - 1) as f32).to_radians();
    // the highest it gets over the target, and at what pitch: below it the height there rises
    // with the pitch, so the lower solution lies between the lowest pitch and that one
    let heights: Vec<(f32, Option<f32>)> = (0..LOB_STEPS).map(|k| (pitch_at(k), lob_height(speed, pitch_at(k), across))).collect();
    let best = heights.iter().filter_map(|&(a, y)| y.map(|y| (a, y))).max_by(|x, y| x.1.total_cmp(&y.1));
    match best {
        Some((top, y)) if y >= to.y => {
            let (mut lo, mut hi) = (-LOB_STEEPEST.to_radians(), top);
            if lob_height(speed, lo, across).is_some_and(|y| y >= to.y) {
                return way(lo);
            }
            for _ in 0..LOB_HALVINGS {
                let mid = 0.5 * (lo + hi);
                if lob_height(speed, mid, across).is_some_and(|y| y >= to.y) { hi = mid } else { lo = mid }
            }
            way(hi)
        }
        _ => {
            // out of reach: the pitch that carries it farthest at the target's height
            let far = (0..LOB_STEPS).map(pitch_at).filter(|&a| a > 0.0).max_by(|&a, &b| lob_reach(speed, a, to.y).total_cmp(&lob_reach(speed, b, to.y)));
            way(far.unwrap_or(std::f32::consts::FRAC_PI_4))
        }
    }
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
    let splash_radius = game.archetype_sphere(arch).map_or(radius, |s| s.1);
    Some(MissileKit { parts, trail, radius, splash_radius })
}

/// Where a character's feet are (the game's character position, char+0x458; the blast's body
/// cylinder stands on them).
fn feet(u: &Player) -> Vec3 {
    u.position + Vec3::Y * (GROUND + u.height)
}

/// The point a missile homes on, and its searches and its fuse measure to, on a character: its
/// scene node (the target's vtable +0x40 position, 0x10fda0: FUN_00226cc0 at 0x2270fe,
/// FUN_002237c0, the fuse), which for a character is its model's root node (0x158290 from
/// FUN_0012f440). FUN_0023d4f0 puts that at the feet's x and z, the feet's height + the
/// `root-height` of the clip last started (animations-*.xml; FUN_0023d240 sets it as a clip
/// starts): the hips, not the feet (Brutus's Sc_w1_rp_idle 0.98 m, the Shrikes' idles 1.014 and
/// 1.121; their height-offset is 0). A clip without one (-100000) takes the model's own
/// (model+0x40, not read: the demo's GROUND, 1.05). Knocked down, the root node follows the
/// ragdoll (not traced): the demo takes the ragdoll's root bone. The crosshair's aim capsule
/// hangs from it too (0x121d00, play_reticle.rs).
pub(super) fn root_point(u: &Player) -> Vec3 {
    let f = feet(u);
    let Some(l) = u.loaded.as_ref() else { return f - Vec3::Y * GROUND };
    if let Some(k) = &u.knock {
        if let Some(p) = l.model.parent.iter().position(|p| p.is_none()).and_then(|i| k.ragdoll.bone_world(i)) {
            return p;
        }
    }
    let height = u.layers.last().and_then(|x| l.model.anims.get(x.clip)).and_then(|a| a.root_height).unwrap_or(-GROUND);
    f + Vec3::Y * height
}

/// A character's name for the log.
fn name(u: &Player) -> String {
    format!("{}{}", unit_name(u), u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)))
}

/// Whether a missile of side `side` takes a character of team `team` for hostile (the table at
/// 0x3ffd80): the demo's teams, 0 (the squad, CONFEDERATION) and 1 (the enemies), are hostile to
/// each other; NEUTRAL to neither.
fn hostile(side: u8, team: u8) -> bool {
    side != NEUTRAL && side != team
}

/// FUN_002237c0: up to `count` living characters hostile to `side` (see `hostile`) within
/// `range` m of `at` and `cone` degrees of `fwd`, to their root (`root_point`), with nothing of
/// the world in between, the most central first. The line of sight is a ray to the root that
/// meets only the terrain (`Arena::ray_terrain`: the query's flags 0xfffffffa leave out the
/// level's placed geometry, rocks and walls, FUN_000df0a0); at the hips, the ground they stand
/// on isn't in the way.
fn search(units: &[&Player], at: Vec3, fwd: Vec3, range: f32, cone: f32, count: usize, side: u8) -> Vec<Who> {
    let cos = cone.to_radians().cos();
    let mut found: Vec<(f32, Who)> = units.iter().filter(|u| !u.dead && hostile(side, u.team)).filter_map(|u| {
        let to = root_point(u) - at;
        let d = to.length();
        if d > range || d < 1e-3 {
            return None;
        }
        let dir = to / d;
        let dot = dir.dot(fwd);
        // (the flat floor keeps its own sight test, its pillars and the test tools' scenery, #185)
        let clear = match world::arena().filter(|a| !a.is_flat()) {
            Some(a) => a.ray_terrain(at, dir, d).is_none(),
            None => ray_hit(at, dir, d).is_none(),
        };
        if std::env::var("BF_MISSILE_DEBUG").is_ok() {
            println!("  search from {at:.2}: {} at {:.1} m (their root {:.2}), {:.0} degrees (cone {cone}), clear {clear}", name(u), d, at + to, dot.clamp(-1.0, 1.0).acos().to_degrees());
        }
        (dot >= cos && clear).then(|| (-dot, u.who()))
    }).collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().take(count).map(|f| f.1).collect()
}

/// Where a missile aims on the character `who`: their root (`root_point`; None if they're
/// gone).
fn origin_of(units: &[&Player], who: Who) -> Option<Vec3> {
    units.iter().find(|u| u.who() == who).map(|u| root_point(u))
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
    // the physics step (FUN_00169e20, before the objects' ticks): a body nothing steers falls
    // under the world's gravity, its speed damped (`falls`), then moves
    if m.falls {
        m.vel = fall(m.vel, dt);
    }
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
    // the straight phase; at its end a missile that seeks searches once, target or not (a
    // search that finds nobody keeps the target), 0x226d8e-0x226da7 (`seek`)
    if m.straight > 0.0 {
        m.straight -= dt;
        if m.straight <= 0.0 {
            m.straight = 0.0;
            if b.seeks && m.mode == 0 {
                seek(m, range, units, dt, now, log);
            }
        }
    }
    // steering (FUN_002266d0): always for a steady one; along its velocity in the straight
    // phase, after it toward its target's root or its aim point
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
                    let off = d.normalize_or(m.heading).dot(m.heading).clamp(-1.0, 1.0).acos().to_degrees();
                    println!("t {now:.2}: missile {} gives up {} for a point ahead ({:.2} m away, {off:.0} degrees off its heading)", m.id,
                             if d.length_squared() <= RETARGET_NEAR_SQ { "its aim point (within 1 m)" } else { "its aim (over 60 degrees off)" }, d.length());
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
        let along = m.vel.dot(dir);
        m.vel -= (m.vel - dir * along) * STEER_ACROSS;
        if (speed - along).abs() > STEER_SLACK {
            m.vel += dir * ((speed - along) * STEER_SPEED);
        }
    }
    // the timers: life, then the second timer
    if m.life > 0.0 {
        m.life -= dt;
    } else if m.after > 0.0 {
        m.after -= dt;
    }
    if m.mode == 1 && m.vel.y < APOGEE_VY && m.straight == 0.0 {
        if log {
            println!("t {now:.2}: missile {} at its apogee: rising {:.2} m/s, {:.2} m above its launch", m.id, m.vel.y, m.pos.y - m.from.y);
        }
        return Tick::Split;
    }
    // the proximity fuse, after the give-up and the timers: every FUSE_EVERY s, a body (its
    // target, if it has one, else any hostile) within reach of its root (`root_point`), that it
    // has passed (on it, or behind its velocity).
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
                    let off = root_point(u) - m.pos;
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

/// A body lying knocked down as a blast's ball meets it: its ragdoll's bones as balls of
/// LYING_RADIUS (as `ray_lying` strikes them), each one's distance from `at` and its place,
/// nearest first (the game's ball meets the ragdoll's own bodies; the shape is the demo's).
fn lying_parts(u: &Player, at: Vec3) -> Vec<(f32, Vec3)> {
    let Some(k) = &u.knock else { return vec![] };
    let mut out: Vec<(f32, Vec3)> = k.ragdoll.bones_world().map(|bone| ((bone.distance(at) - LYING_RADIUS).max(0.0), bone)).collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// Where along a ray (within `max`) it meets a body lying knocked down: its ragdoll's bones,
/// each taken as a ball of LYING_RADIUS (the game's missile strikes the ragdoll's own bodies,
/// FUN_002278e0; the shape is the demo's).
pub(super) fn ray_lying(origin: Vec3, dir: Vec3, u: &Player, max: f32) -> Option<f32> {
    let r = &u.knock.as_ref()?.ragdoll;
    r.bones_world().filter_map(|p| {
        let off = origin - p;
        let b = off.dot(dir);
        let disc = b * b - (off.length_squared() - LYING_RADIUS * LYING_RADIUS);
        let t = -b - disc.max(0.0).sqrt();
        (disc >= 0.0 && t > 0.0 && t < max).then_some(t)
    }).min_by(|a, b| a.total_cmp(b))
}

/// FUN_00226b80: every SEEK_EVERY s (its timer +0x1d0 runs down on each call, 0 from the
/// constructor FUN_002264e0), the most central hostile within the range and SEEK_CONE degrees
/// of its heading, with nothing in between, becomes its target; finding nobody keeps the one it
/// has. The tick calls it at the straight phase's end, target or not (0x226d8e-0x226da7), and
/// every tick while there's no target (0x226e02). By its own side: a NEUTRAL child's finds
/// nobody (see NEUTRAL).
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

/// FUN_00227f80 (WM_MIRV_ON_EXPLODE / ON_APOGEE): the children of a parent whose life is out
/// (or, ON_APOGEE, that's past its apogee). Each leaves beside it (`r_parent + r_child` m off
/// along a side axis rolled at random, then 360 / count degrees on for the next), turned
/// spread-max x 90 degrees toward that side, at the parent's speed, living range / speed x
/// 0.85-1.15. The parent's speed is the bullet's for a steady one (+0x208, copied); a lobbed
/// one's children leave at the speed its body has now (0x228748-0x228794: without the bullet's
/// h_0b03ece5 a child's +0x208 is set to the parent's velocity's length, and its flag 4
/// cleared), and fall (`falling`: their object type's `falls`) as it did. If the parent seeks,
/// the split's hostiles (up to the count, within the range and 30 degrees) are dealt out to them
/// in turn; without any, each aims 200 m along the parent's velocity.
#[allow(clippy::too_many_arguments)]
fn split(m: &Missile, b: &MeshBullet, range: f32, units: &[&Player], radii: (f32, f32), falling: bool, state: &mut MissileState, now: f32,
         log: bool) -> Vec<Missile> {
    let count = b.mirv_count.min(MIRV_MAX).max(0) as usize;
    if count == 0 {
        return vec![];
    }
    // along the parent's velocity (its body's, vtable +0x54 = 0x159d70), the frame FUN_0001d0c0
    // builds from it: up is world y, or world z within ~10 degrees of vertical (|y| > 0.9848)
    let fwd = m.vel.normalize_or(m.heading);
    let up0 = if fwd.y.abs() > 0.9848 { Vec3::Z } else { Vec3::Y };
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
    // (the children's life from the bullet's speed, FUN_00227dd0 run on each before its speed
    // is set: range / +0x208 with +0x208 the bullet's)
    let start = range / m.speed.max(1e-3);
    let speed = if b.steady { m.speed } else { m.vel.length() };
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
            side: NEUTRAL, human: m.human, pos, vel: heading * speed, heading, speed, life, start, straight: start * b.straight, after: b.after,
            seek_in: 0.0, fuse_clock: 0.0, target, aim, mode: 0, falls: falling, clock: m.clock, from: m.from, age: m.age, trails: vec![], wet: false,
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
            target, aim: target.and_then(|t| origin_of(&units, t)).unwrap_or(l.aim), mode: b.mirv_mode, falls: falls(game.0.weapons.get(&b.object), b.steady),
            clock: 0.0, from: l.origin, age: 0.0, trails: vec![], wet: false,
        };
        if log {
            println!("t {now:.2}: missile {} launched ({}) from {:.2} along {:.3} at {speed} m/s: life {start:.3} s, straight {:.3} s, then {} s; mirv {} x mode {}; target {:?}, aim {:.2}{}",
                     m.id, def.label, m.pos, dir, m.straight, m.after, b.mirv_count, b.mirv_mode,
                     target.and_then(|t| units.iter().find(|u| u.who() == t)).map(|u| name(u)), m.aim,
                     if m.falls { format!("; lobbed (gravity {GRAVITY}, damping {SPEED_DAMPING} a second)") } else { String::new() });
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
                test: Option<Res<super::testmap::TestMap>>, mut shaker: ResMut<CameraShake>, difficulty: Res<Difficulty>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_MISSILE_LOG").is_ok();
    let now = player.sim_time;
    let listener = player.position;
    let mut blows = vec![];
    let mut children = vec![];
    let mut splashes = vec![];
    let arena = world::arena();
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
                let from = m.pos;
                match tick(&mut m, &b, range, radius, &units, &mines, super::shots::TICK, now, log) {
                    Tick::Flying => {
                        // into a liquid (#5): its splash, by its mesh's radius (the liquids aren't
                        // solid: it flies on through)
                        let r = kits.0.get(&m.object).and_then(|k| k.as_ref()).map_or(b.radius, |k| k.splash_radius);
                        let (to, speed) = (m.pos, m.vel.length());
                        if let Some(s) = arena.as_ref().and_then(|a| super::liquids::object_enters(a, &mut m.wet, from, to, b.radius, r, speed)) {
                            if log || std::env::var("BF_LIQUID_LOG").is_ok() {
                                println!("t {now:.2}: missile {} into a liquid (type {}) at {speed:.1} m/s: a {} splash at {:.2} (r {r:.3})", m.id, s.0.kind,
                                         if s.2 == bf_viewer::bf::character::LIQUID_BIG_SPLASH { "big" } else { "small" }, s.1);
                            }
                            splashes.push(s);
                        }
                    }
                    Tick::Split => {
                        let radii = |o: u32| kits.0.get(&o).and_then(|k| k.as_ref()).map_or(0.0, |k| k.radius);
                        let child = if b.mirv_object != 0 { b.mirv_object } else { m.object };
                        let falling = falls(game.0.weapons.get(&child), b.steady);
                        children.extend(split(&m, &b, range, &units, (radii(m.object), radii(child)), falling, &mut state, now, log));
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
            // (the mesh faces its velocity while FUN_002266d0 steers it: it turns the node along
            // it for the frame's picture. One it doesn't steer, a lobbed one, keeps its body's
            // turn, set only at the launch and the split: its heading)
            let steered = b.steady || ((b.homes || b.seeks) && m.straight <= 0.0);
            let face = if steered { m.vel } else { m.heading };
            if face.length_squared() > 1e-6 {
                tr.look_to(face.normalize(), Vec3::Y);
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
    for (t, at, slot) in splashes {
        super::liquids::splash(&mut player, &t, slot, at);
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
               &mut mines, &mut loose, &mut objects, &mut tint, &mut hits, &mut shaker, &difficulty, instant, now, log);
    }
}

/// A roll of a weapon's Damage for `u` (a draw in 0..1): FUN_0022c940's min (type+0x1f4) + U x
/// (max (+0x1f0) - min), the two as written. The iKhan-GPL's are written the wrong way round
/// (min 20, max 18), so it rolls 18-20, as take03's half shares show (13.1-14.0: x1.4 of 18.7-
/// 20.0) where taking the smaller as the min rolled 18 every time. The Therm Sweeper's and the
/// Sweeper V's (84-96, 27-37) roll as before.
fn damage_roll(def: &WeaponDef, u: f32) -> f32 {
    def.damage_min + (def.damage - def.damage_min) * u
}

/// The factor the shooter `who`'s skill puts on one roll of its missile's (FUN_0022c940 asks a
/// character shooter's skill, +0xec: Flint's doubled and scoped rolls, play_skills.rs
/// `roll_factor`), each roll its own; 1 once the shooter is gone.
fn shooter_skill(player: &mut Player, squad: &mut Squad, who: Who) -> f32 {
    unit_mut(player, squad, who).map_or(1.0, |(u, _)| super::skills::roll_factor(u))
}

/// The factor the shooter `who`'s power-ups put on a roll of its missile's (FUN_0022c940 after the
/// skill: each of its modifiers' +0x10, BRUTE POWER's x2, play_powerups.rs `roll_factor`); 1 once
/// the shooter is gone.
fn shooter_power(player: &mut Player, squad: &mut Squad, who: Who) -> f32 {
    unit_mut(player, squad, who).map_or(1.0, |(u, _)| super::powerups::roll_factor(u))
}

/// A missile going off (`Blow`): unless it ended without a blast, its explosion effect type
/// (the ALE effects and light at the point, turned along its way for a collision, and its
/// sounds), the bullet's impact sound, the direct hit's damage on the body it struck
/// (FUN_0022c4d0: a roll of Damage min..max x the struck hit part's factor, x 0.7 on its own
/// side, x the victim's step and damage-type factor: play_damage.rs),
/// a Sentry it struck set off, the blast object (its own roll, after 0.05-0.25 s), loose pickups
/// thrown and the scenery's share, and the scorch on the ground under the blast point (within
/// its radius), laid DECAL_DELAY later as the grenades' are.
#[allow(clippy::too_many_arguments)]
fn go_off(commands: &mut Commands, game: &mut Game, ale: Option<&mut bf_viewer::ale_fx::AleAssets>, images: &mut Assets<Image>,
          materials: &mut Assets<StandardMaterial>, bl: Blow, player: &mut Player, squad: &mut Squad, state: &mut MissileState,
          mines: &mut super::sentry::MineTargets, loose: &mut super::pickups::Blasts, objects: &mut super::scenery::ObjectBlasts, tint: &mut ScreenTint,
          hits: &mut super::scenery::ObjectHits, shaker: &mut CameraShake, difficulty: &Difficulty, instant: bool, now: f32, log: bool) {
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
    shake(shaker, at, camera::view(player).0, &def);
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
        let roll = damage_roll(&def, state.random()) * shooter_skill(player, squad, bl.owner) * shooter_power(player, squad, bl.owner);
        hits.0.push(super::scenery::ObjectHit { target, amount: roll, damage_type: def.damage_type, at: bl.at, dir: bl.dir });
        if log {
            println!("t {now:.2}: missile {} direct hit: breakable {target} takes a roll of {roll:.1}", bl.id);
        }
    }
    if let Some(e) = bl.mine {
        mines.strike(e, 0.0, 1.0, now);
    }
    // the direct hit (FUN_0022c4d0 with no blast object): a roll, now, x the shooter's skill's
    // and power-ups' factors (`shooter_skill`, `shooter_power`), x the hit-location factor
    // (FUN_00125940 at 0x227a03: the victim type's damage-multiplier x the struck part's bone's
    // Damage; the missile's own +0x1cc is 1.0), x0.7 on its own side, then the victim's step and
    // damage-type factor (play_damage.rs). The part: along the missile's way into the body from
    // STRUCK_BACK m before the contact
    let (factor, power) = if bl.struck.is_some() { (shooter_skill(player, squad, bl.owner), shooter_power(player, squad, bl.owner)) } else { (1.0, 1.0) };
    if let Some((u, controlled)) = bl.struck.and_then(|w| unit_mut(player, squad, w)) {
        let roll = damage_roll(&def, state.random());
        let bone = damage::struck_bone(u, game, bl.at - bl.dir * STRUCK_BACK, bl.dir, STRUCK_BACK);
        let mut hit = damage::Hit::new(roll, "roll", def.damage_type).skill(factor).power(power).by(bl.owner, bl.team, u).struck(u, game, bone);
        let amount = hit.land(u, game, difficulty, now);
        // (the test map's instant kill: anyone but the player, from the player's missiles)
        let amount = if instant && bl.human && !controlled { enemies::lethal(u).max(amount) } else { amount };
        let push = Vec3::new(bl.dir.x, 0.0, bl.dir.z).normalize_or(Vec3::Z) * HIT_PUSH.0 + Vec3::Y * HIT_PUSH.1;
        let before = u.health;
        hurt_still(u, game, amount, &hit, push, bl.at);
        if controlled && u.health < before {
            tint.0 = 0.0;
        }
        if log {
            let part = hit.part.map_or(1.0, |p| p.factor());
            println!("t {now:.2}: missile {} direct hit: {} takes {amount:.1} (roll {roll:.1}, x{part:.2} at bone {}{}, x{:.2} the step) -> {:.1} / {:.0}", bl.id,
                     name(u), bone.map_or("-".to_string(), |b| format!("h_{:08x}", b.bone)), if hit.friendly < 1.0 { ", own side x0.7" } else { "" }, hit.step,
                     u.health, u.max_health);
        }
    }
    let radius = def.blast_radius;
    if radius <= 0.0 || def.damage <= 0.0 {
        return;
    }
    // the blast object (FUN_0022bfc0): its own roll (x the shooter's skill's and power-ups'
    // factors, its own: FUN_0022c940 again), dealt after its delay (`deal_blasts`)
    let roll = damage_roll(&def, state.random()) * shooter_skill(player, squad, bl.owner) * shooter_power(player, squad, bl.owner);
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
        let ground = floor_y(at.x, at.z, at.y + 0.5, world::Body::Thing);
        if at.y - ground <= radius {
            state.scorches.push((DECAL_DELAY, def.decal, at));
        }
    }
}

/// `hurt` without the knockdown it may start (play.rs's rule for a hit of KNOCKDOWN_DAMAGE or
/// more): no missile hit or blast in the takes moved its victim (Tex hurt by his own blasts,
/// takes 10-16; frozen enemies hit directly and by blasts, takes 06, 18, 19, 23). The game's
/// knockdown rule isn't traced. A kill still throws the body (the death push). `hit` says whose it
/// is for the hurt line (play_chatter.rs: Tex's own blast, ff44e83f).
fn hurt_still(u: &mut Player, game: &Game, amount: f32, hit: &damage::Hit, push: Vec3, at: Vec3) {
    let asked = u.knock_request.is_some();
    hurt(u, game, amount, chatter::Cause::hit(hit), push, at, -1);
    if !asked && !u.dead && u.knock_request.take().is_some() && std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("  (not knocked down after all: a missile's hit, play_missile.rs)");
    }
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

/// The blasts whose delay is out (FUN_00224770 -> FUN_0022c4d0): every living body the blast's
/// ball meets (one of its hit parts within the Damage radius and BLAST_PART_REACH more), with
/// nothing of the world between the blast and the bone of a part it meets, takes the share (half
/// the roll, flat; Damage h_011cb154 true: the whole roll to half the radius, then (1 - 2 (d / r -
/// 0.5))^2 of it, d to its root), with no hit part (FUN_00224a90 passes 0), x 0.7 on the shooter's
/// own side (the shooter too), x the victim's step and damage-type factor (play_damage.rs); the
/// player's view goes red. No living body is moved (FUN_00224a90 pushes only loose objects and a
/// corpse's root body, `ragdoll::blast_corpse`; the push given to `hurt_still` only aims the blood
/// and throws the guns of a body it kills). Then the scorches and sounds due.
#[allow(clippy::too_many_arguments)]
fn deal_blasts(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut state: ResMut<MissileState>,
               mut tint: ResMut<ScreenTint>, mut decals: ResMut<super::fx::DecalRequests>, test: Option<Res<super::testmap::TestMap>>,
               difficulty: Res<Difficulty>) {
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
                // a corpse: the blast's push on its ragdoll's root body, by its roll, when the
                // weapon pushes and the blast sees one of its hit parts, as it must the living's
                // (FUN_00224a90, play_ragdoll.rs `blast_corpse`, #259)
                let eye = b.eye;
                ragdoll::blast_corpse(u, b.at, radius, b.roll, def.push, |p| {
                    let to = p - eye;
                    to.length() <= 0.05 || ray_hit(eye, to.normalize(), to.length()).is_none()
                });
                continue;
            }
            // what of the body the blast's ball meets (FUN_00224a90: one of its collision-bone
            // bodies overlapping the ball): its hit parts' surfaces as drawn
            // (`damage::part_distances`; lying knocked down, its ragdoll's bones, `lying_parts`),
            // each within the radius and BLAST_PART_REACH more; a body without them (no model or
            // hit-part list) by its standing cylinder, BLAST_BODY_REACH round and BODY_HEIGHT tall
            // from the feet
            let f = feet(u);
            let middle = f + Vec3::Y * BODY_MIDDLE;
            let parts = if knocked_down(u) { lying_parts(u, b.at) } else { damage::part_distances(u, &game.0, b.at) };
            let reach: Vec<(f32, Vec3)> = if parts.is_empty() {
                let across = (Vec2::new(b.at.x - f.x, b.at.z - f.z).length() - BLAST_BODY_REACH).max(0.0);
                let up = b.at.y - b.at.y.clamp(f.y, f.y + BODY_HEIGHT);
                vec![(across.hypot(up), middle)]
            } else {
                parts.iter().map(|&(d, bone)| ((d - BLAST_PART_REACH).max(0.0), bone)).collect()
            };
            let near = parts.first().map_or(reach[0].0, |p| p.0);
            let by = if parts.is_empty() { "its cylinder" } else { "its nearest part" };
            if reach[0].0 > radius {
                if log && reach[0].0 <= radius + 1.0 {
                    println!("t {now:.2}: missile {} blast: {} out of reach ({by} {near:.2} m away)", b.id, name(u));
                }
                continue;
            }
            // its line of sight: from the blast to the bone of a part in reach, the nearest first
            // (the game's ray goes from the centre to each listed bone body's position,
            // FUN_002b3970; one clear is enough, and a character behind cover isn't gathered)
            let clear = reach.iter().take_while(|r| r.0 <= radius).find(|r| {
                let to = r.1 - b.eye;
                to.length() <= 0.05 || ray_hit(b.eye, to.normalize(), to.length()).is_none()
            });
            if clear.is_none() {
                if log {
                    let to = reach[0].1 - b.eye;
                    println!("t {now:.2}: missile {} blast: {} {near:.1} m away, behind cover (from {:.2} to {:.2}: met at {:.2} m)", b.id, name(u), b.eye,
                             reach[0].1, ray_hit(b.eye, to.normalize_or(Vec3::Y), to.length()).unwrap_or(0.0));
                }
                continue;
            }
            let share = if def.blast_core {
                // (the character's size term, its skill's vtable +0xe0, is 0 for the squad; the
                // entry's point is the character's position, vtable +0x40: its root, FUN_00224a90)
                let x = root_point(u).distance(b.at) / radius;
                if x <= 0.5 { b.roll } else { (1.0 - 2.0 * (x - 0.5)).clamp(0.0, 1.0).powi(2) * b.roll }
            } else {
                b.roll * BLAST_SHARE
            };
            let kind = if def.blast_core { "the core rule's share" } else { "half the roll" };
            let mut hit = damage::Hit::new(share, kind, def.damage_type).by(b.owner, b.team, u);
            let amount = hit.land(u, &game.0, &difficulty, now);
            let amount = if instant && human && k != 0 { enemies::lethal(u).max(amount) } else { amount };
            let away = Vec3::new(f.x - b.at.x, 0.0, f.z - b.at.z).normalize_or(Vec3::X);
            let before = u.health;
            hurt_still(u, &game.0, amount, &hit, away * BLAST_PUSH.0 + Vec3::Y * BLAST_PUSH.1, middle);
            // (a kill's own shoves on the body, behind ragdoll::DEATH_SHOVE: the blast's record
            // has no part, and a push of 2 when its weapon's is 0, FUN_0022c4d0)
            if u.dead && before > 0.0 {
                ragdoll::death_shoves(u, None, middle, away, amount, def.damage_type, if def.push > 0.0 { def.push } else { 2.0 });
            }
            if k == 0 && u.health < before {
                tint.0 = 0.0;
            }
            if log {
                println!("t {now:.2}: missile {} blast: {} ({by} {near:.2} m away) takes {amount:.1} ({kind}{}, x{:.2} the step) -> {:.1} / {:.0}", b.id, name(u),
                         if hit.friendly < 1.0 { ", own side x0.7" } else { "" }, hit.step, u.health, u.max_health);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The iKhan-GPL's bullet (h_e9be70f0, every level alike): not steady, no homing or seeking,
    /// the proximity fuse, 3 children h_175a0adf on the apogee, spread-max 0.16, straight 0.6.
    fn ikhan() -> MeshBullet {
        MeshBullet { object: 0x175A_0ADF, radius: 0.1, homes: false, straight: 0.6, seeks: false, after: 0.0, explode_at_end: true, steady: false,
                     fuse: true, mirv_count: 3, mirv_mode: 1, mirv_object: 0x175A_0ADF, spread: [0.708, 0.16] }
    }

    /// A projectile as `launch_missiles` makes one of the iKhan-GPL's (23 m/s, range 40), moving
    /// at `vel`, `age` s after its launch.
    fn round(vel: Vec3, age: f32) -> Missile {
        let start = 40.0 / 23.0;
        Missile { id: 1, weapon: 0xE9BE_70F0, object: 0x175A_0ADF, owner: Who::Squad(0), team: 0, human: true, side: 0, pos: Vec3::ZERO, vel,
                  heading: vel.normalize(), speed: 23.0, life: start - age, start, straight: 0.0, after: 0.0, seek_in: 0.0, fuse_clock: 0.0,
                  target: None, aim: Vec3::ZERO, mode: 1, falls: true, clock: 0.0, from: Vec3::ZERO, age, trails: vec![], wet: false }
    }

    /// #239: the iKhan-GPL's round fired level from 1.44 m over flat ground (take08's muzzle over
    /// the sand) falls under 9.81 m/s^2 with the 0.1 a second damping (FUN_0016a250,
    /// FUN_00158f80): it comes down on the 16th game frame, 0.53 s and 11.9 m out (take08: on the
    /// sand ~0.47 s after the launch, "~8-10 m ahead"), at 22.4 m/s.
    #[test]
    fn a_level_round_falls_to_the_ground_12_m_out() {
        let (mut pos, mut vel) = (Vec3::Y * 1.44, Vec3::X * 23.0);
        let mut frames = 0;
        let down = loop {
            frames += 1;
            vel = fall(vel, shots::TICK);
            let next = pos + vel * shots::TICK;
            if next.y <= 0.0 {
                break pos.x + (next.x - pos.x) * pos.y / (pos.y - next.y);
            }
            pos = next;
        };
        assert_eq!(frames, 16);
        assert!((down - 11.87).abs() < 0.02, "down {down} m out");
        assert!((vel.length() - 22.39).abs() < 0.02, "at {} m/s", vel.length());
        // the object types' h_11e7d864: set on the two steady missiles' (the Therm Sweeper's, the
        // Sweeper V's), which keep their flight; not on the iKhan-GPL's round
        assert!(!falls(Some(&WeaponDef { no_gravity: true, ..Default::default() }), true));
        assert!(falls(Some(&WeaponDef::default()), false));
        // (an object type the map doesn't define goes by the bullet: steady or not)
        assert!(falls(None, false) && !falls(None, true));
    }

    /// #239: a lobbed parent's children leave at the speed its body has at the split (FUN_00227f80
    /// at 0x228748-0x228794), 14.4 degrees off its way (spread-max 0.16 x 90); a steady one's at
    /// the bullet's. Their life is the bullet's range / speed x 0.85-1.15 either way.
    #[test]
    fn a_lobbed_parents_children_leave_at_its_speed() {
        let vel = Vec3::new(18.0, -0.4, 0.0);
        let mut state = MissileState::default();
        let kids = split(&round(vel, 1.067), &ikhan(), 40.0, &[], (0.04, 0.04), true, &mut state, 0.0, false);
        assert_eq!(kids.len(), 3);
        for k in &kids {
            assert!((k.vel.length() - vel.length()).abs() < 1e-3, "child at {} m/s", k.vel.length());
            assert!((k.speed - vel.length()).abs() < 1e-3);
            assert!((k.vel.normalize().dot(vel.normalize()).acos().to_degrees() - 14.4).abs() < 0.01);
            assert!(k.life >= 0.85 * 40.0 / 23.0 && k.life <= 1.15 * 40.0 / 23.0, "life {}", k.life);
            assert_eq!((k.mode, k.side), (0, NEUTRAL));
        }
        let steady = MeshBullet { steady: true, ..ikhan() };
        let kids = split(&round(vel, 1.067), &steady, 40.0, &[], (0.04, 0.04), false, &mut state, 0.0, false);
        assert!(kids.iter().all(|k| (k.vel.length() - 23.0).abs() < 1e-3 && !k.falls));
    }

    /// The AI's lob (`lob`, the demo's stand-in): at the iKhan-GPL's 23 m/s, a target 20 m off at
    /// the muzzle's height is reached on the lower of the two pitches (~11 degrees, the flight
    /// passing through it within 2 cm); one below the muzzle on a lower pitch; one out of reach
    /// (90 m) takes the pitch that carries it farthest, between 30 and 45 degrees.
    #[test]
    fn an_ai_lobs_onto_its_target() {
        let from = Vec3::new(3.0, 1.4, -2.0);
        for (to, low) in [(Vec3::new(3.0, 1.4, -22.0), (8.0, 14.0)), (Vec3::new(-7.0, -0.6, 15.32), (0.0, 12.0))] {
            let way = lob(from, to, 23.0);
            let pitch = way.y.asin();
            let across = Vec2::new(to.x - from.x, to.z - from.z).length();
            let height = lob_height(23.0, pitch, across).expect("it gets there");
            assert!((height - (to.y - from.y)).abs() < 0.02, "{height} at {across} m");
            assert!(pitch.to_degrees() > low.0 && pitch.to_degrees() < low.1, "pitch {}", pitch.to_degrees());
            // (along the target's way across)
            let flat = Vec2::new(way.x, way.z).normalize();
            assert!(flat.dot(Vec2::new(to.x - from.x, to.z - from.z).normalize()) > 0.9999);
        }
        let far = lob(from, from + Vec3::new(0.0, 0.0, -90.0), 23.0).y.asin().to_degrees();
        assert!(far > 30.0 && far < 45.0, "far pitch {far}");
    }

    /// FUN_0022c940 rolls min + U x (max - min) as written: the iKhan-GPL's min 20 and max 18
    /// roll 18-20 (take03's half shares, 13.1-14.0 at x1.4); the Therm Sweeper's 84-96 as before.
    #[test]
    fn damage_rolls_between_min_and_max_as_written() {
        let ikhan = WeaponDef { damage: 18.0, damage_min: 20.0, ..Default::default() };
        assert_eq!((damage_roll(&ikhan, 0.0), damage_roll(&ikhan, 1.0)), (20.0, 18.0));
        let therm = WeaponDef { damage: 96.0, damage_min: 84.0, ..Default::default() };
        assert_eq!((damage_roll(&therm, 0.0), damage_roll(&therm, 0.5)), (84.0, 90.0));
    }
}
