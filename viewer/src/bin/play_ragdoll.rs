//! The game's ragdoll, for every character (#259): the bodies, joints, masses, damping, physics
//! step and pushes of the game's ragdoll as #280's study read them from default.xbe
//! (todo/280-ragdoll/answer.md, spec.md S1-S9; viewer/docs/code-map.md), on a small rigid-body
//! solver of the demo's own. It replaces the verlet point masses play.rs had (#94), and #129's
//! enemy-only branch of it.
//!
//!  - **Bodies** (FUN_00127290, FUN_00114010, FUN_00114d00, FUN_00114b80): one for each of the
//!    character's hit parts (its type's `<collision>` bones, `damage::hit_bones`: eleven on every
//!    biped), shaped by the part's collision surface (`Character::part_collision`) and turning
//!    about its mass centre (`Character::part_mass_centre`, where Ipion puts a body's core); a
//!    0.05 m ball for each ancestor of a hit part (pelvis, spines, neck, clavicles) and for the
//!    character's own node (the model's root, the root body char+0x500). Masses: 9 kg for the
//!    root and for a body whose bone hangs from it (Bip01), 4.5 kg for the rest, 90 kg a biped;
//!    inertia isotropic, 0.5 and 0.25 x the mass (4.5 and 1.125 kg m^2).
//!  - **Joints** (FUN_00113120): one per body with its parent's, at the child bone's origin, hard,
//!    with the game's ranges by the child bone's name (`joint_limits`: knees and elbows one-way
//!    hinges, the hips, spine, head and shoulders within three ranges), about the pose at death (an
//!    Ipion constraint's frames are made from the bodies as they are, FUN_002b9370); a bone not in
//!    its table (Bip01, the pelvis, the spine's base, the neck, the clavicles) is held rigid to its
//!    parent as it was at death, so the hips, the chest and the shoulders move as pieces. The
//!    solver takes each such rigid piece as one body (`Body`), its parts' masses summed.
//!  - **Bones without a body** (below the lowest hit part: feet, toes, hands, fingers, a Shrike's
//!    HorseLink; and every bone of the face, the hair and the props) are carried rigidly in their
//!    pose at death under their parent: the game disconnects the whole skeleton from the
//!    animation (node flag 0x20, FUN_00127160) and simulates the bodies alone. Bones keep their
//!    lengths: a bone with a body takes its body's turn and its place from its parent's frame.
//!  - **The start** (FUN_00127290): every body exactly on its bone at death (the hit parts are
//!    beamed to their bones, FUN_00169140), each given an impulse of 9 x the character's velocity
//!    at its centre (the 9 kg bodies start at that velocity, the 4.5 kg ones at twice it), which
//!    the build gives only to a character that moves. One that stood still stays as it stood for
//!    REST_WAKE s, then goes (or at once if something pushes it): Ipion makes an object out of
//!    the simulation (FUN_002b2f40: movement state 8) and only a push or a touch revives it
//!    (FUN_002b3c60, FUN_002bb1e0), and the takes' frozen Shrikes hold their pose 0.35-0.45 s,
//!    until about when their dropped gun reaches their feet (`REST_WAKE`, a fit). No other push
//!    on a death: the CBT_DEAD handler's shove and up-kick (FUN_00115f90) are in the code but in
//!    no take, so they wait behind DEATH_SHOVE.
//!  - **The step** (Ipion): 66 steps a second whatever the frame rate, gravity 9.81, damping
//!    `v *= 1 - k dt` (0.95 a second linear, 0.5 angular, +0.1 once calm), contact with the level's
//!    faces (`Arena::faces_near`) with friction 4 and restitution 0.04: a body that lands stays.
//!    Bodies of one ragdoll don't meet each other, nor the living's capsules (spec S2.5). Every
//!    15-19 steps Ipion's movement check (FUN_002bb980, FUN_002be350) finds each body moving,
//!    calm or still by how far it went since its references, and freezes the ragdoll once all
//!    are still (`movement_check`, `CHECK_EVERY`).
//!  - **Pushes**: the queued shove (FUN_00127c60: `Ragdoll::shove`, the Vengar's charge,
//!    play_vengar.rs); a blast's push on a corpse's root body (FUN_00224a90: `Ragdoll::blast`);
//!    a shot into a body on the ground, the demo's placeholder (`Ragdoll::shot`, spec S6.5); the
//!    Energy grenade's bolt (`Ragdoll::fling`, play_energy.rs).
//!
//! The solver (the demo's, not the game's: Ipion solves a ragdoll's joints together as one linear
//! system each step, FUN_002db0a0 / FUN_002db5a0): extended position-based dynamics (Müller et
//! al., "Detailed Rigid Body Simulation with Extended Position Based Dynamics", 2020), SUBSTEPS
//! substeps a step, each moving the bodies, holding the joints' points and ranges and the
//! contacts by moving the bodies, then taking the velocities from the moves and applying
//! restitution and sliding friction. The game's isotropic inertia makes every angular correction
//! a plain turn (a rigid piece's is its parts' summed, `Body::inv_inertia`). A crate was weighed
//! (rapier, avian): either would need the level's triangles mirrored into its own world and kept
//! in step with the doors and breakables the arena already tracks, for a dozen bodies a
//! character.
//!
//! Test hooks: `BF_RAGDOLL_LOG=1` prints a ragdoll's bodies (their parts, mass, inertia, contact
//! points, how high the lowest point is over the floor), joints (ranges, which way a bend takes
//! the limb) and start, then a line a frame for its first 3 s and every half second after (the
//! root body's place and speed, the fastest body and spin, how high the trunk is over the floor,
//! the worst joint gap, the contacts, how many bodies are calm and still, asleep), and each shove
//! and blast; `BF_RAGDOLL_DRAW=1` draws every ragdoll's bodies (their surfaces, the balls) and
//! joints.

use super::*;
use bf_viewer::arena::Face;

/// The physics step (s): Ipion's delta_PSI_time, 1/66 s (0x3c783e10 at env+0xb4, set by its
/// environment constructor FUN_002b4780; no game code writes it). The world steps Ipion to the
/// game's time each update (FUN_00169e20 -> FUN_002b5440), so a body falls at 66 steps a second
/// whatever the frame rate.
const STEP: f32 = 1.0 / 66.0;
/// The most steps one frame runs (a frame after a load or a stall doesn't run seconds of
/// physics at once): the demo's.
const MAX_STEPS: u32 = 8;
/// Substeps of the solver per step: the demo's (Ipion's direct solve has none). 16 hold a 90 kg
/// body's joints to under a millimetre (BF_RAGDOLL_LOG's joint gap).
const SUBSTEPS: u32 = 16;
/// Gravity (m/s^2, down): the Ipion world's (0, -9.81, 0) (FUN_0016a250 at 0x16a4a7, over Ipion's
/// default), not the characters' own 18 (their capsule mover's, FUN_0012cd10, left at death).
const GRAVITY: f32 = 9.81;
/// The root's mass (kg), and every other body's (FUN_00127290: 0x40cbe8, 9.0; 4.5 for the rest).
/// The root is the character node's body (char+0x500) and any body whose bone hangs from that
/// node (Bip01): about 90 kg a biped.
const ROOT_MASS: f32 = 9.0;
const LIMB_MASS: f32 = 4.5;
/// Inertia per kg (m^2), isotropic: the root's 0.5 (0x40cbe4) and the others' 0.25
/// (FUN_00127290 sets the core's three inertia components to these, then scales them by the
/// mass it gives it: 4.5 and 1.125 kg m^2). So no body spins easily: a push at a point mostly
/// moves it.
const ROOT_INERTIA: f32 = 0.5;
const LIMB_INERTIA: f32 = 0.25;
/// Damping a second (FUN_00127290 into the cores: linear core+0x44 = 0.95, 0x40cbe0; angular
/// core+0x24..+0x2c = 0.5, 0x40cbdc), taken each step as `v *= 1 - k dt` (Ipion's FUN_002baa70;
/// `exp(-k dt)` from k dt 0.25 on, which a step never reaches): e^-0.95t and e^-0.5t.
const LINEAR_DAMPING: f32 = 0.95;
const ANGULAR_DAMPING: f32 = 0.5;
/// Once a body is calm (its core's movement state, core+0x64, above 1: `Ragdoll::movement_check`)
/// both dampings take this much more (FUN_002ba9b0: 1.05 and 0.6).
const CALM_DAMPING: f32 = 0.1;
/// Ipion's movement check: every this many steps a ragdoll's bodies are looked at (a simulation
/// unit's step, FUN_002be4c0, counts env+0x128 down and, at 0, resets it to 15 + 5 x a random
/// 0..1 and runs FUN_002be350: every 15-19 steps; the demo takes 17, a check every ~0.26 s) ...
const CHECK_EVERY: u32 = 17;
/// ... and each body is given a movement state (FUN_002bb980, a core's): calm when, since its
/// short reference, it has gone at most CALM_MOVE m (0.0001 m^2) and its far end at most ~7 mm
/// ((1 - (q . q0)^2) x 2 x r^2 at most CALM_ARC, r its radius, core+4), and still once calm for
/// more than CALM_TIME s (env+0x90, 0.3, FUN_002b4780's default); otherwise moving, its short
/// reference moved to where it is now ...
const CALM_MOVE: f32 = 0.01;
const CALM_ARC: f32 = 2.5e-5;
const CALM_TIME: f32 = 0.3;
/// ... and a moving body that hasn't gone more than LONG_MOVE m (0.01 m^2) nor its far end more
/// than ~0.28 m (LONG_ARC, 0.04) from its long reference in LONG_TIME s counts as still too. When
/// every body is still at a check, the ragdoll is frozen (FUN_002be350 -> FUN_002bb0e0: its
/// cores' speeds zeroed, out of the simulation) until a push revives it.
const LONG_MOVE: f32 = 0.1;
const LONG_ARC: f32 = 0.04;
const LONG_TIME: f32 = 4.0;
/// The start: each body takes an impulse of this x the character's velocity at its centre (its
/// core's position, core+0x12c: FUN_00127290, 0x40cbe8's 9 again, through FUN_002b3bc0), so the
/// 9 kg bodies start at the velocity and the 4.5 kg ones at twice it. Only when the velocity's
/// square is above 0 (FUN_00127290's test before the push): a character that stood still gets
/// none.
const START_IMPULSE: f32 = 9.0;
/// Under this speed (m/s) a character stood still as it died (the demo's: the walk cycle's root
/// leaves a standing body a few mm/s), so its ragdoll gets no start push.
const AT_REST: f32 = 0.05;
/// A ragdoll that stood still holds its pose this long (s), unless pushed sooner: a fit. The takes'
/// frozen Shrikes (todo/280-ragdoll answer §3: takes 17-22, first motion 0.35-0.45 s after the
/// hit) hold every limb as it was until then, though the ragdoll is built in the hit's frame (the
/// gun leaves the hands at once, take22 2.60-2.63 s). The code's side: Ipion makes an object out
/// of the simulation (FUN_002b2f40 sets its movement state to 8: the balls'; that a still
/// character's hit-part bodies were frozen by the movement check too is an inference), and only
/// a push (FUN_002b3bc0 -> FUN_002b3c60, which the build gives a still character none of) or a
/// moving object's touch revives it (FUN_002bb1e0). What touches it in the takes is a guess: the
/// dropped gun, which reaches the feet as the body starts to move (take22: the ground at
/// 2.93-2.97 s, the body at 3.00). The demo's dropped guns don't meet the ragdoll, so this stands
/// in. Tex in take05 went 0.5 s after the hit.
const REST_WAKE: f32 = 0.35;
/// An ancestor's body: a ball this big (m) (FUN_00114b80 -> FUN_002cce90, Ipion's ball).
const BALL_RADIUS: f32 = 0.05;
/// Contact with the level: friction, the product of the two materials' (Ipion's manager,
/// 0x2bc0b0): a ragdoll body is of world-material 0 (friction 2: world-materials.xml
/// `<physical>` h_09b8b74d, FUN_001a3320; the body template's material, FUN_00114d00), the ground
/// 2 (3 on some: 6). The demo takes 4 everywhere: either way a body that touches stays put.
const FRICTION: f32 = 4.0;
/// ... and restitution, the product of the elasticities (0x2bc0e0): 0.2 x 0.2 (h_e74d9431 on
/// every material). Below a bounce of 2 g dt it's none (the solver's rule).
const RESTITUTION: f32 = 0.04;
/// How far round a body (m) the level's faces are gathered for its contacts each step, past
/// what it moves in the step: the demo's.
const CONTACT_MARGIN: f32 = 0.05;
/// How deep under a face (m) a point of a body may be and still be pushed out of it (deeper, it
/// went through: a thin wall's other side): the demo's.
const CONTACT_DEPTH: f32 = 0.3;
/// How far past a face's edge (m) a point over it still counts as over it (the seams between a
/// floor's triangles): the demo's.
const EDGE_SLACK: f32 = 0.02;
/// A point already under a face as a step starts is taken out of it at most this fast (m/s):
/// the demo's (Ipion recovers a penetration gradually; a Shrike's calf whose surface starts 3 cm
/// in the ground otherwise left it at several m/s, throwing the body up).
const RECOVERY_SPEED: f32 = 0.5;
/// A hit part's contact points: its surface's corners, at most this many (the ones furthest out
/// along 26 ways round it, when it has more): the demo's.
const MAX_POINTS: usize = 26;
/// FUN_00127c60's shove: every body but the one struck takes min(SHOVE_ALL x magnitude x factor,
/// SHOVE_ALL_CAP) N s along the shove (0x3c05f4, 0x3c05e8; only with a factor of 1 or more); the
/// one struck min(SHOVE_HIT x magnitude x factor, SHOVE_HIT_CAP) more at the point (0x3c05f0,
/// 0x3c05ec); none while the root goes faster than SHOVE_TOP_SPEED m/s.
const SHOVE_ALL: f32 = 0.5;
const SHOVE_ALL_CAP: f32 = 75.0;
const SHOVE_HIT: f32 = 8.0;
const SHOVE_HIT_CAP: f32 = 300.0;
const SHOVE_TOP_SPEED: f32 = 100.0;
/// ... times L = (1 - w) x 0.4 + 0.6, w the world's load (world+0xc58, FUN_000d9090's measure of
/// explosions and living objects, #151): 1 in a quiet scene, which the demo takes.
const SHOVE_LOAD: f32 = 1.0;
/// ... and a shove of damage type 10 (DTYPE_EXPLOSION) has a factor of at least 3.
const EXPLOSION_TYPE: i64 = 10;
const EXPLOSION_FACTOR: f32 = 3.0;
/// The CBT_DEAD handler's shoves (FUN_00115f90 -> FUN_000e00b0): the hit's (magnitude 2 x the
/// damage, the damage type, the weapon's push as the factor, at the point struck, on the part
/// struck: 0x11607a) and the up-kick (magnitude 1, type 0, factor UP_KICK = 36, 0x39dfa0,
/// straight up on every body: 0x116124; not in the air nor knocked down). As written, an L-Shot
/// kill would throw a body at the Vengar's ~16 m/s and every death would hop 3.8 m/s up, which
/// none of the eight filmed deaths does (todo/280-ragdoll answer §2.5, §3; spec S6.4): off until
/// a live read (the shot list's A1) settles it. True turns both on.
pub const DEATH_SHOVE: bool = false;
const UP_KICK: f32 = 36.0;
/// A blast's push on a corpse's root body (FUN_00224a90): 1.5 (0x3bc6f0) x (1.2 - d / r)^2 x the
/// blast's damage N s, away from the blast, d the body's distance from it and r its radius, when
/// the weapon's push is 1 or more.
const BLAST_PUSH: f32 = 1.5;
const BLAST_REACH: f32 = 1.2;
/// A shot into a body on the ground (spec S6.5, the demo's placeholder: the code pushes corpses
/// through no path found; #129 take14 shows a small shift as a lying Heavy is killed): the part
/// struck is brought up to this speed (m/s) along the shot where it's struck, not past it (a
/// burst jolts a body, it doesn't drive it), and a shot that passes this close past its end
/// (m) still meets it.
const SHOT_SHOVE: f32 = 3.0;
const SHOT_REACH: f32 = 0.2;
/// The knock-down's start (the squad's own: the game's knock-down is a clip, #129): the demo's
/// old rule for it, each body moving with the hit's push x KNOCK_PUSH and sagging KNOCK_SAG m/s,
/// both by how high it is over the floor (so the body topples rather than slides).
const KNOCK_PUSH: f32 = 1.6;
const KNOCK_SAG: f32 = 2.5;
/// The thud: the body's landing sound plays once, when its root comes this near (m) to the floor
/// under it (the demo's, as before).
const THUD_HEIGHT: f32 = 0.25;
/// Spine2's name hash: the chest, which with the root makes the trunk BF_RAGDOLL_LOG measures.
const SPINE2: u32 = 0x072A_54B5;

/// FUN_00113120's joint ranges (radians, about the joint's x, y and z: (least, most) each), by
/// the child bone's name hash; None for a bone not in its table, whose joint is rigid: its three
/// rotation axes get type 1 (0x1131e4), and type 1 is Ipion's fixed axis (the constraint
/// template's constructor FUN_002b8f20 sets the three translation axes, +0xd0..+0xd8, to 1 and the
/// rotation axes, +0xdc..+0xe4, to 0: a ball socket, so 1 holds, 0 is free; 2, limited, takes
/// the ranges at +0xf4 / +0x10c). #280's study read type 1 as free. The hashes are the ones in
/// its switch (0x113120-0x1137d5; the Biped names found by hashing, todo/129-shrikes answers Q6);
/// the eight unnamed cases (seven pairs, one bone) are other skeletons'. Zero is the pose at
/// death: the constraint's frames are made from the two bodies as they are (FUN_002b9370 takes
/// the point and the reference body's frame at the build), and every range holds 0, so no joint
/// starts past one. Which way an axis points isn't traced (the frame comes from the parent node): the
/// demo reads them in the child bone's own frame, x along the bone (the twist), y the main bend,
/// z the side bend (BF_RAGDOLL_LOG prints which way -y takes each limb).
fn joint_limits(bone: u32) -> Option<[(f32, f32); 3]> {
    let d = |a: f32, b: f32| (a.to_radians(), b.to_radians());
    let r = |x: (f32, f32), y: (f32, f32), z: (f32, f32)| Some([d(x.0, x.1), d(y.0, y.1), d(z.0, z.1)]);
    match bone {
        // Bip01 L / R Thigh: the hip
        0x1D1B_6AC1 | 0xF8AE_E8F8 => r((-45.0, 45.0), (-60.0, 110.0), (-20.0, 20.0)),
        // Bip01 L / R Calf: the knee, a hinge
        0x044F_3505 | 0x0E87_8ECD => r((0.0, 0.0), (-90.0, 0.0), (0.0, 0.0)),
        // Bip01 L / R HorseLink: a digitigrade leg's hock
        0xF5EC_631B | 0xF32E_5B7A => r((0.0, 0.0), (-30.0, 30.0), (-100.0, 25.0)),
        // Bip01 L / R Foot: the ankle
        0x1765_B430 | 0x1DAD_0FF8 => r((-40.0, 40.0), (-20.0, 40.0), (-10.0, 10.0)),
        // Bip01 Spine1, Spine2, Spine3
        0x1C23_050F | 0x072A_54B5 | 0x0E2D_6423 => r((-45.0, 45.0), (-45.0, 45.0), (-25.0, 25.0)),
        // Bip01 Neck1
        0x1D3F_CE6A => r((0.0, 0.0), (-15.0, 50.0), (-45.0, 45.0)),
        // Bip01 Head
        0xF6E2_E9F1 => r((-45.0, 45.0), (-45.0, 45.0), (-45.0, 45.0)),
        // Bip01 L / R UpperArm: the shoulder
        0xEAD6_D58E | 0xF1DE_18BE => r((-55.0, 55.0), (-40.0, 100.0), (-45.0, 80.0)),
        // Bip01 L / R Forearm: the elbow, a hinge
        0x1A3B_B0B0 | 0x1BA7_3AB3 => r((0.0, 0.0), (-100.0, 0.0), (0.0, 0.0)),
        // Bip01 L / R Hand: the wrist
        0x0578_41D8 | 0x0FB0_FA10 => r((-90.0, 90.0), (-70.0, 70.0), (-35.0, 35.0)),
        // the unnamed cases (other skeletons' bones)
        0x1FDA_09F8 | 0x04D2_C4C8 => r((-30.0, 30.0), (-60.0, 50.0), (-25.0, 40.0)),
        0x0240_627E | 0xE7F5_E047 => r((-30.0, 30.0), (-75.0, 65.0), (-30.0, 30.0)),
        0x108B_1FAD | 0x1117_95AE => r((0.0, 0.0), (-125.0, 0.0), (0.0, 0.0)),
        0xF06D_7F52 | 0xFAA5_C49A => r((0.0, 0.0), (-50.0, 40.0), (0.0, 0.0)),
        0xE8B8_3172 | 0xE270_8ABA => r((0.0, 0.0), (0.0, 90.0), (0.0, 0.0)),
        0xE347_FE67 | 0xE98F_45AF => r((0.0, 0.0), (-45.0, 55.0), (0.0, 0.0)),
        0xEB53_E377 => r((-90.0, 90.0), (-50.0, 15.0), (-45.0, 45.0)),
        0xFB2B_CE6D | 0xE022_9FD7 => r((-45.0, 45.0), (-65.0, 15.0), (-25.0, 25.0)),
        _ => None,
    }
}

/// The bones a character's ragdoll gives bodies to (FUN_00127290: its hit parts, char+0x4d8, from
/// its type's `<collision>` list; their ancestors get balls): `damage::hit_bones`' bones, the
/// squad's as an enemy's (#259: one rule for every character).
pub fn parts(game: &Game, u: &Player) -> Vec<u32> {
    damage::hit_bones(game, u).iter().map(|b| b.bone).collect()
}

/// A shove request (an entry of the world's ragdoll queue, FUN_000e00b0): the part struck (its
/// bone's name hash; None: none, every body alike), the point and the way, its magnitude, damage
/// type and factor. Applied once the ragdoll is built (FUN_000d9a20 takes the queue's shoves in
/// the update after the build).
#[derive(Clone, Copy, Debug)]
pub struct Shove {
    pub bone: Option<u32>,
    pub point: Vec3,
    pub dir: Vec3,
    pub magnitude: f32,
    pub kind: i64,
    pub factor: f32,
}

/// How a ragdoll starts: a death (the game's: the character's velocity, `START_IMPULSE`) or the
/// squad's knock-down (the demo's: thrown by the hit's push, KNOCK_PUSH and KNOCK_SAG).
#[derive(Clone, Copy, Debug)]
pub enum Start {
    Death { velocity: Vec3 },
    Knock { push: Vec3 },
}

/// One of the game's ragdoll bodies (FUN_00127290's: a hit part or a ball), as the solver body
/// that carries it holds it.
#[derive(Clone, Debug)]
struct Part {
    /// its bone (None: the character's node, the model's own frame, for a model whose root bone
    /// is a hit part)
    bone: Option<usize>,
    /// its bone's frame in the solver body's frame
    rot: Quat,
    pos: Vec3,
    /// its own centre (its core's place) in the solver body's frame, and its mass: the start push
    /// and the shoves go to the game's bodies one by one
    centre: Vec3,
    mass: f32,
}

/// A rigid body of the solver: one of the game's bodies, or a piece of them the game holds
/// rigidly together (FUN_00113120's rigid joints: the hips' node, Bip01, pelvis and spine's base;
/// the chest's Spine2, neck and clavicles), their masses summed about their common centre and
/// their inertias with them (each part's own, isotropic, and its offset's, isotropic in the mean:
/// 2/3 m r^2).
#[derive(Clone, Debug)]
struct Body {
    parts: Vec<Part>,
    /// world: its centre of mass, its turn (its first part's bone's), its velocity and spin
    x: Vec3,
    q: Quat,
    v: Vec3,
    w: Vec3,
    /// where it was at the substep's start
    x0: Vec3,
    q0: Quat,
    mass: f32,
    inv_mass: f32,
    inv_inertia: f32,
    /// its contact points from its centre in its frame, each with how round it is (a ball's
    /// radius; 0 for a surface's corner)
    points: Vec<(Vec3, f32)>,
    /// its hit parts' surfaces and its balls (centre, radius), from its centre in its frame:
    /// what shots meet
    tris: Vec<[Vec3; 3]>,
    balls: Vec<(Vec3, f32)>,
    /// how far its furthest point is from its centre (m): also its radius in the movement check
    /// (core+4, read as Ipion's radius of the core)
    reach: f32,
    /// the movement check's short and long references (place, turn, the step clock then) and its
    /// state: 1 moving, 2 calm, 3 still (`Ragdoll::movement_check`)
    short: (Vec3, Quat, f32),
    long: (Vec3, Quat, f32),
    state: u8,
}

impl Body {
    /// An impulse `j` (N s, world) at world point `at`.
    fn impulse(&mut self, at: Vec3, j: Vec3) {
        self.v += j * self.inv_mass;
        self.w += (at - self.x).cross(j) * self.inv_inertia;
    }

    /// Its frame (its centre and turn) in the world.
    fn frame(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.q, self.x)
    }

    /// The world place of a point given in its frame.
    fn at(&self, p: Vec3) -> Vec3 {
        self.x + self.q * p
    }
}

/// A joint between two bodies (FUN_00113120's, limited): at the child bone's origin, with the
/// child bone's ranges.
#[derive(Clone, Debug)]
struct Joint {
    parent: usize,
    child: usize,
    /// the joint's point from each body's centre, in its frame
    at_parent: Vec3,
    at_child: Vec3,
    /// the child bone's turn in the parent body's frame at death: its angles' zero
    rest: Quat,
    /// the child bone's turn in the child body's frame
    child_off: Quat,
    /// FUN_00113120's ranges about the child bone's x, y and z (`joint_limits`)
    limits: [(f32, f32); 3],
    /// the child's bone name hash (for the log)
    name: u32,
}

/// A point of a body against a face of the level for this step: the plane it's kept in front of
/// (`n . p >= d - slack`, its radius taken in), and this substep's push along it.
#[derive(Clone, Copy, Debug)]
struct Contact {
    body: usize,
    point: Vec3,
    n: Vec3,
    d: f32,
    /// how far under the plane the point may still be (m): as deep as it was as the step began,
    /// less RECOVERY_SPEED's way out each substep
    slack: f32,
    lambda: f32,
    /// the point's speed along `n` before the substep (for its restitution)
    vn0: f32,
}

/// A character's ragdoll: its bodies and joints, and how each bone of its skeleton is posed from
/// them.
pub struct Ragdoll {
    bodies: Vec<Body>,
    joints: Vec<Joint>,
    /// per bone: its body and its part in it (None: carried in its pose at death under its
    /// parent)
    body_of: Vec<Option<(usize, usize)>>,
    /// per bone: its parent (the model's), and its transform under it at death (a root bone's:
    /// in the model's frame)
    parent: Vec<Option<usize>>,
    local: Vec<Mat4>,
    /// the model's frame at death (the character's node then): the pose is drawn relative to it
    frame: Mat4,
    /// the root body's frame at death (a root bone without a body rides on the root body)
    root_frame0: Mat4,
    /// each bone's frame in the world now
    posed: Vec<Mat4>,
    /// time owed to the steps (s)
    owed: f32,
    /// the floor (world height) where the level has no collision (the test map's flat floor
    /// without scenery): where the feet were at death
    floor: f32,
    contacts: Vec<Contact>,
    /// the steps run (s), the movement check's clock, and the steps to its next check
    /// (`CHECK_EVERY`)
    clock: f32,
    countdown: u32,
    /// frozen: not simulated until something pushes it (`movement_check`, `REST_WAKE`)
    asleep: bool,
    /// asleep from the start (it stood still at death) until this age (s), unless pushed
    /// (`REST_WAKE`)
    wake_at: Option<f32>,
    /// seconds since it started
    pub age: f32,
    /// the bone names, for the log
    names: Vec<u32>,
}

/// One of the game's bodies as the build makes it, before the rigid pieces are put together
/// (`Ragdoll::new`): its bone, its frame at death (world), its centre in that frame, its mass and
/// inertia, its contact points (that frame, with their roundness), surface and balls, the body it
/// hangs from and its joint's ranges (None: rigid).
struct Proto {
    bone: Option<usize>,
    frame: Mat4,
    com: Vec3,
    mass: f32,
    inertia: f32,
    points: Vec<(Vec3, f32)>,
    tris: Vec<[Vec3; 3]>,
    balls: Vec<(Vec3, f32)>,
    parent: usize,
    limits: Option<[(f32, f32); 3]>,
}

impl Ragdoll {
    /// A ragdoll from a character's pose at death: `world`, its bones' transforms in the model's
    /// frame (play.rs `Player::last_world`), the model drawn at `frame` (the character's node:
    /// its position raised by its height and the soles' lift, turned by its yaw), with bodies for
    /// the bones `parts` names (the hit parts, `parts`) and their ancestors, started by `start`.
    /// `floor`: the ground's height where the level has none. Bones without a body ride on their
    /// parent in their pose at death.
    pub fn new(model: &Character, world: &[Mat4], frame: Mat4, parts: &[u32], start: Start, floor: f32) -> Self {
        let n = model.bones.len().min(world.len());
        let parent: Vec<Option<usize>> = (0..n).map(|i| model.parent[i].filter(|&p| p < n)).collect();
        let at: Vec<Mat4> = world[..n].iter().map(|m| frame * *m).collect();
        // the hit parts (with a surface or not) and every bone above one
        let mut part = vec![false; n];
        for i in 0..n {
            part[i] = parts.contains(&model.bones[i]);
        }
        if !part.contains(&true) {
            // a type without a `<collision>` list has no hit parts and no ragdoll in the game; the
            // demo gives the bones with a collision surface bodies (else the root alone), so the
            // body still falls
            for i in 0..n {
                part[i] = model.part_collision.get(i).is_some_and(|s| !s.is_empty());
            }
            if !part.contains(&true) {
                if let Some(r) = (0..n).find(|&i| parent[i].is_none()) {
                    part[r] = true;
                }
            }
        }
        let mut bodied = vec![false; n];
        for i in 0..n {
            if part[i] {
                let mut k = Some(i);
                while let Some(j) = k.filter(|&j| !bodied[j]) {
                    bodied[j] = true;
                    k = parent[j];
                }
            }
        }
        // the game's bodies. First the character's node: the model's root bone above the hit
        // parts (every model here hangs its skeleton from one, at the model's origin: the
        // Shrikes' h_12688f2d), its ball the root body (char+0x500); a model whose root bone is a
        // hit part itself gets a ball of its own at the model's origin
        let ball = |bone: Option<usize>, m: Mat4, mass: f32, inertia: f32, parent: usize, limits| Proto {
            bone, frame: m, com: Vec3::ZERO, mass, inertia: inertia * mass, points: vec![(Vec3::ZERO, BALL_RADIUS)], tris: vec![],
            balls: vec![(Vec3::ZERO, BALL_RADIUS)], parent, limits,
        };
        let node = (0..n).filter(|&i| parent[i].is_none() && bodied[i]).collect::<Vec<_>>();
        let node = match node[..] {
            [r] if !part[r] => Some(r),
            _ => None,
        };
        let mut protos = vec![match node {
            Some(r) => ball(Some(r), at[r], ROOT_MASS, ROOT_INERTIA, 0, None),
            None => ball(None, frame, ROOT_MASS, ROOT_INERTIA, 0, None),
        }];
        let mut proto_of = vec![None; n];
        if let Some(r) = node {
            proto_of[r] = Some(0);
        }
        for i in (0..n).filter(|&i| bodied[i] && Some(i) != node) {
            // (a body whose bone hangs from the character's node is the root's weight too)
            let on_node = match node { Some(r) => parent[i] == Some(r), None => parent[i].is_none() };
            let (mass, inertia) = if on_node { (ROOT_MASS, ROOT_INERTIA) } else { (LIMB_MASS, LIMB_INERTIA) };
            let above = parent[i].and_then(|p| proto_of[p]).unwrap_or(0);
            // FUN_00113120's ranges about the pose at death; a bone not in its table held as it was
            let limits = joint_limits(model.bones[i]);
            let proto = match model.part_collision.get(i).filter(|s| part[i] && !s.is_empty()) {
                Some(s) => {
                    let corners = corners(s);
                    let com = model.part_mass_centre.get(i).copied().flatten()
                        .unwrap_or_else(|| corners.iter().sum::<Vec3>() / corners.len() as f32);
                    Proto { bone: Some(i), frame: at[i], com, mass, inertia: inertia * mass, points: corners.into_iter().map(|c| (c, 0.0)).collect(),
                            tris: s.clone(), balls: vec![], parent: above, limits }
                }
                None => ball(Some(i), at[i], mass, inertia, above, limits),
            };
            proto_of[i] = Some(protos.len());
            protos.push(proto);
        }
        // the rigid pieces: each body joined to the one it hangs from by a rigid joint is one
        // with it (a body's parent comes before it, so its piece is known)
        let mut piece = vec![0usize; protos.len()];
        let mut pieces: Vec<Vec<usize>> = vec![vec![0]];
        for k in 1..protos.len() {
            if protos[k].limits.is_none() {
                piece[k] = piece[protos[k].parent];
                pieces[piece[k]].push(k);
            } else {
                piece[k] = pieces.len();
                pieces.push(vec![k]);
            }
        }
        let centre = |p: &Proto| p.frame.transform_point3(p.com);
        let turn = |p: &Proto| Quat::from_mat4(&p.frame).normalize();
        let mut bodies = vec![];
        let mut body_of = vec![None; n];
        for (b, members) in pieces.iter().enumerate() {
            let mass: f32 = members.iter().map(|&k| protos[k].mass).sum();
            let x = members.iter().map(|&k| centre(&protos[k]) * protos[k].mass).sum::<Vec3>() / mass;
            let q = turn(&protos[members[0]]);
            let back = q.inverse();
            let inertia: f32 = members.iter().map(|&k| protos[k].inertia + 2.0 / 3.0 * protos[k].mass * centre(&protos[k]).distance_squared(x)).sum();
            // each part's points, surface and balls into the body's frame
            let into = |k: usize, p: Vec3| back * (protos[k].frame.transform_point3(p) - x);
            let mut body = Body { parts: vec![], x, q, v: Vec3::ZERO, w: Vec3::ZERO, x0: x, q0: q, mass, inv_mass: 1.0 / mass, inv_inertia: 1.0 / inertia,
                                  points: vec![], tris: vec![], balls: vec![], reach: 0.0, short: (x, q, 0.0), long: (x, q, 0.0), state: 1 };
            for &k in members {
                let p = &protos[k];
                if let Some(i) = p.bone {
                    body_of[i] = Some((b, body.parts.len()));
                }
                body.parts.push(Part { bone: p.bone, rot: (back * turn(p)).normalize(), pos: into(k, Vec3::ZERO), centre: into(k, p.com), mass: p.mass });
                body.points.extend(p.points.iter().map(|&(c, r)| (into(k, c), r)));
                body.tris.extend(p.tris.iter().map(|t| t.map(|c| into(k, c))));
                body.balls.extend(p.balls.iter().map(|&(c, r)| (into(k, c), r)));
            }
            body.reach = body.points.iter().map(|(p, r)| p.length() + r).fold(0.0, f32::max);
            bodies.push(body);
        }
        // the joints between the pieces: each limited one, at its child bone's origin
        let mut joints = vec![];
        for (k, p) in protos.iter().enumerate() {
            let Some(limits) = p.limits.filter(|_| k > 0) else { continue };
            let (bp, bc) = (&bodies[piece[p.parent]], &bodies[piece[k]]);
            let anchor = p.frame.w_axis.truncate();
            let q = turn(p);
            joints.push(Joint { parent: piece[p.parent], child: piece[k], at_parent: bp.q.inverse() * (anchor - bp.x),
                                at_child: bc.q.inverse() * (anchor - bc.x), rest: (bp.q.inverse() * q).normalize(),
                                child_off: (bc.q.inverse() * q).normalize(), limits, name: p.bone.map_or(0, |i| model.bones[i]) });
        }
        // the start: a moving character's push (9 x its velocity at each of the game's bodies'
        // centres); one that stood still holds its pose a while (`REST_WAKE`)
        let mut wake_at = None;
        match start {
            Start::Death { velocity } if velocity.length() < AT_REST => wake_at = Some(REST_WAKE),
            Start::Death { velocity } => {
                for b in bodies.iter_mut() {
                    let centres: Vec<Vec3> = b.parts.iter().map(|p| b.at(p.centre)).collect();
                    for c in centres {
                        b.impulse(c, velocity * START_IMPULSE);
                    }
                }
            }
            Start::Knock { push } => {
                let low = bodies.iter().map(|b| b.x.y).fold(f32::MAX, f32::min).min(floor);
                let top = bodies.iter().map(|b| b.x.y).fold(low + 0.5, f32::max);
                for b in bodies.iter_mut() {
                    let k = ((b.x.y - low) / (top - low)).clamp(0.1, 1.0);
                    b.v = (push * KNOCK_PUSH + Vec3::NEG_Y * KNOCK_SAG) * k;
                }
            }
        }
        let local = (0..n).map(|i| match parent[i] {
            Some(p) => world[p].inverse() * world[i],
            None => world[i],
        }).collect();
        let root_frame0 = bodies[0].frame();
        let mut r = Ragdoll { bodies, joints, body_of, parent, local, frame, root_frame0, posed: vec![Mat4::IDENTITY; n], owed: 0.0, floor,
                              contacts: vec![], clock: 0.0, countdown: CHECK_EVERY, asleep: wake_at.is_some(), wake_at, age: 0.0,
                              names: model.bones[..n].to_vec() };
        r.pose();
        if std::env::var("BF_RAGDOLL_LOG").is_ok() {
            r.log_build(start);
        }
        r
    }

    /// Run it on by `dt` s of game time: as many 66 Hz steps as are owed, then the bones posed.
    pub fn step(&mut self, dt: f32) {
        self.age += dt.max(0.0);
        if self.wake_at.is_some_and(|t| self.age >= t) {
            self.wake();
        }
        if self.asleep || dt <= 0.0 {
            return;
        }
        self.owed += dt;
        let mut steps = 0;
        while self.owed >= STEP && steps < MAX_STEPS {
            self.owed -= STEP;
            self.psi();
            steps += 1;
            if self.asleep {
                self.owed = 0.0;
                break;
            }
        }
        self.owed = self.owed.min(STEP);
        self.pose();
    }

    /// The bones' transforms under their parents now (a root bone's in the model's frame at
    /// death), for play.rs `apply_pose`.
    pub fn local_pose(&self) -> Vec<(Quat, Vec3)> {
        let back = self.frame.inverse();
        (0..self.posed.len()).map(|i| {
            let m = match self.parent[i] {
                Some(p) => self.posed[p].inverse() * self.posed[i],
                None => back * self.posed[i],
            };
            let (_, q, t) = m.to_scale_rotation_translation();
            (q.normalize(), t)
        }).collect()
    }

    /// Where bone `i` is now (world).
    pub fn bone_world(&self, i: usize) -> Option<Vec3> {
        self.posed.get(i).map(|m| m.w_axis.truncate())
    }

    /// Where every bone is now (world), in the skeleton's order.
    pub fn bones_world(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.posed.iter().map(|m| m.w_axis.truncate())
    }

    /// The root body's place now (the character node's, char+0x500): where the corpse is
    /// (0x1271a0 writes it into the character's node each step), which the death camera, the
    /// blood pool and the memory chip go by.
    pub fn root(&self) -> Vec3 {
        let b = &self.bodies[0];
        b.at(b.parts[0].centre)
    }

    /// The model's first root bone's place now in the model's frame at death (the knock-down's
    /// get-up stands the character under it).
    pub fn root_model(&self) -> Vec3 {
        let i = self.parent.iter().position(|p| p.is_none()).unwrap_or(0);
        self.frame.inverse().transform_point3(self.posed.get(i).map_or(Vec3::ZERO, |m| m.w_axis.truncate()))
    }

    /// Whether the root has come down to the floor under it (the thud: THUD_HEIGHT).
    pub fn landed(&self) -> bool {
        let r = self.root();
        r.y <= self.floor_under(r) + THUD_HEIGHT
    }

    /// The floor under `p` (world height): the level's, else the flat fallback.
    fn floor_under(&self, p: Vec3) -> f32 {
        world::arena().and_then(|a| a.floor_below(p.x, p.z, p.y + 0.5)).map_or(self.floor, |f| f.0)
    }

    /// FUN_00127c60: a queued shove (`Shove`) on the built ragdoll, on each of the game's bodies.
    pub fn shove(&mut self, s: &Shove) {
        let root = self.bodies[0].v.length();
        let dir = s.dir.normalize_or_zero();
        if s.magnitude <= 0.0 || root > SHOVE_TOP_SPEED || dir == Vec3::ZERO {
            return;
        }
        let factor = if s.kind == EXPLOSION_TYPE && s.factor < EXPLOSION_FACTOR { EXPLOSION_FACTOR } else { s.factor };
        let all = if factor >= 1.0 { (SHOVE_ALL * s.magnitude * factor).min(SHOVE_ALL_CAP) * SHOVE_LOAD } else { 0.0 };
        let struck = s.bone.and_then(|b| self.names.iter().position(|&x| x == b)).and_then(|i| self.body_of.get(i).copied().flatten());
        let hit = (SHOVE_HIT * s.magnitude * if factor > 0.0 { factor } else { 1.0 }).min(SHOVE_HIT_CAP) * SHOVE_LOAD + all;
        for (bi, b) in self.bodies.iter_mut().enumerate() {
            for pi in 0..b.parts.len() {
                if struck == Some((bi, pi)) {
                    b.impulse(s.point, dir * hit);
                } else if all > 0.0 {
                    let c = b.at(b.parts[pi].centre);
                    b.impulse(c, dir * all);
                }
            }
        }
        self.wake();
        if std::env::var("BF_RAGDOLL_LOG").is_ok() {
            println!("ragdoll shove: magnitude {:.1} type {} factor {factor:.1} along {dir:.2}: {all:.1} N s on every body{}", s.magnitude, s.kind,
                     if struck.is_some() { format!(", {hit:.1} N s on the part struck") } else { String::new() });
        }
    }

    /// FUN_00224a90's push of a blast (at `at`, radius `radius`, its damage `damage`) on a
    /// corpse: its root body, 1.5 x (1.2 - d / r)^2 x the damage N s away from the blast, d the
    /// root's distance from it (only within the radius). The caller checks the weapon's push.
    pub fn blast(&mut self, at: Vec3, radius: f32, damage: f32) {
        let root = self.root();
        let d = root.distance(at);
        if radius <= 0.0 || d >= radius || damage <= 0.0 {
            return;
        }
        let j = BLAST_PUSH * (BLAST_REACH - d / radius).powi(2) * damage;
        let away = (root - at).normalize_or(Vec3::Y);
        self.bodies[0].impulse(root, away * j);
        self.wake();
        if std::env::var("BF_RAGDOLL_LOG").is_ok() {
            println!("ragdoll blast at {d:.2} m of {radius:.1}: {j:.1} N s on the root along {away:.2}");
        }
    }

    /// A shot (world line from `origin` along unit `dir`, ending at `dist`) into the body: the
    /// first part it meets (its surface; a ball by its size) within SHOT_REACH past `dist` is
    /// brought up to SHOT_SHOVE m/s along the shot where it's struck (spec S6.5, the demo's), and
    /// the body wakes. Where it was struck (world), if it was.
    pub fn shot(&mut self, origin: Vec3, dir: Vec3, dist: f32) -> Option<Vec3> {
        let (t, i) = self.first_hit(origin, dir, dist + SHOT_REACH)?;
        let at = origin + dir * t;
        let b = &mut self.bodies[i];
        let r = at - b.x;
        let going = (b.v + b.w.cross(r)).dot(dir);
        let add = SHOT_SHOVE - going;
        if add > 0.0 {
            let k = b.inv_mass + b.inv_inertia * r.cross(dir).length_squared();
            b.impulse(at, dir * (add / k));
        }
        self.wake();
        Some(at)
    }

    /// The first part a line (from `origin` along unit `dir`, within `max`) meets: how far along,
    /// and the body (a hit part by its surface, a ball by its size).
    pub fn first_hit(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, usize)> {
        self.bodies.iter().enumerate().filter_map(|(i, b)| {
            let back = b.q.inverse();
            let (o, d) = (back * (origin - b.x), back * dir);
            let balls = b.balls.iter().filter_map(|&(c, r)| ray_ball(o - c, d, r));
            ray_tris(&b.tris, o, d, max).into_iter().chain(balls).filter(|&t| t > 0.0 && t <= max).min_by(|a, b| a.total_cmp(b)).map(|t| (t, i))
        }).min_by(|a, b| a.0.total_cmp(&b.0))
    }

    /// Throw the whole body along `dir` at `speed` m/s: each body brought up to it along that way
    /// (no faster where it goes so already). The Energy grenade's bolt on a body on the ground
    /// (play_energy.rs: the demo's).
    pub fn fling(&mut self, dir: Vec3, speed: f32) {
        let d = dir.normalize_or_zero();
        for b in self.bodies.iter_mut() {
            let add = speed - b.v.dot(d);
            if add > 0.0 {
                b.v += d * add;
            }
        }
        self.wake();
    }

    /// Simulated again (a push, or the end of the hold at the start): as Ipion revives a frozen
    /// unit (FUN_002bb1e0: every core moving) and as a push does to a core (FUN_002b3c60 ->
    /// FUN_002bb180), the movement check's references are timed from now (their places kept).
    fn wake(&mut self) {
        self.asleep = false;
        self.wake_at = None;
        let now = self.clock;
        for b in self.bodies.iter_mut() {
            b.short.2 = now;
            b.long.2 = now;
            b.state = 1;
        }
    }

    /// One 66 Hz step (an Ipion PSI): the contacts for it, the substeps, the damping and the
    /// rest test.
    fn psi(&mut self) {
        self.find_contacts();
        let h = STEP / SUBSTEPS as f32;
        for _ in 0..SUBSTEPS {
            for b in self.bodies.iter_mut() {
                b.x0 = b.x;
                b.q0 = b.q;
                // (a body of no mass is fixed: the tests' anchor)
                b.v.y -= if b.inv_mass > 0.0 { GRAVITY * h } else { 0.0 };
                b.x += b.v * h;
                b.q = (Quat::from_scaled_axis(b.w * h) * b.q).normalize();
            }
            for c in self.contacts.iter_mut() {
                let b = &self.bodies[c.body];
                let r = b.q0 * c.point;
                c.vn0 = c.n.dot(b.v + b.w.cross(r));
                c.lambda = 0.0;
                c.slack = (c.slack - RECOVERY_SPEED * h).max(0.0);
            }
            for k in 0..self.joints.len() {
                self.solve_joint(k);
            }
            for k in 0..self.contacts.len() {
                self.solve_contact(k);
            }
            for b in self.bodies.iter_mut() {
                b.v = (b.x - b.x0) / h;
                let dq = b.q * b.q0.inverse();
                let dq = if dq.w < 0.0 { -dq } else { dq };
                b.w = dq.to_scaled_axis() / h;
            }
            self.contact_velocities(h);
        }
        // Ipion's damping, once a step (FUN_002baa70), more once calm (FUN_002ba9b0)
        for b in self.bodies.iter_mut() {
            let calm = if b.state >= 2 { CALM_DAMPING } else { 0.0 };
            b.v *= 1.0 - (LINEAR_DAMPING + calm) * STEP;
            b.w *= 1.0 - (ANGULAR_DAMPING + calm) * STEP;
        }
        self.clock += STEP;
        self.countdown = self.countdown.saturating_sub(1);
        if self.countdown == 0 {
            self.countdown = CHECK_EVERY;
            self.movement_check();
        }
    }

    /// Ipion's movement check (FUN_002bb980 on each body, FUN_002be350 on the ragdoll): each
    /// body's state from how far it has gone and turned since its references (`CALM_MOVE`,
    /// `LONG_MOVE`); when every one is still, the ragdoll is frozen, its speeds zeroed.
    fn movement_check(&mut self) {
        let now = self.clock;
        // (how far a turn from `q0` took its far end, squared, as Ipion measures it)
        let arc = |q: Quat, q0: Quat, r: f32| {
            let d = q.dot(q0);
            (1.0 - d * d) * 2.0 * r * r
        };
        let mut still = true;
        for b in self.bodies.iter_mut() {
            b.state = if b.x.distance_squared(b.short.0) <= CALM_MOVE * CALM_MOVE && arc(b.q, b.short.1, b.reach) <= CALM_ARC {
                if now - b.short.2 <= CALM_TIME { 2 } else { 3 }
            } else {
                b.short = (b.x, b.q, now);
                if b.x.distance_squared(b.long.0) > LONG_MOVE * LONG_MOVE || arc(b.q, b.long.1, b.reach) > LONG_ARC {
                    b.long = (b.x, b.q, now);
                    1
                } else if now - b.long.2 > LONG_TIME {
                    3
                } else {
                    1
                }
            };
            still &= b.state == 3;
        }
        if still {
            self.asleep = true;
            for b in self.bodies.iter_mut() {
                b.v = Vec3::ZERO;
                b.w = Vec3::ZERO;
            }
        }
    }

    /// Joint `k` held: its two bodies' points brought together, then the child bone's turn
    /// within its ranges. Each correction is shared by the bodies' inverse masses (and inertias:
    /// isotropic, so a turn's share is plain).
    fn solve_joint(&mut self, k: usize) {
        let j = &self.joints[k];
        let (a, b) = pair(&mut self.bodies, j.parent, j.child);
        // the points together
        let (ra, rb) = (a.q * j.at_parent, b.q * j.at_child);
        let gap = (b.x + rb) - (a.x + ra);
        let len = gap.length();
        if len > 1e-7 {
            let n = gap / len;
            let (wa, wb) = (a.inv_mass + a.inv_inertia * ra.cross(n).length_squared(), b.inv_mass + b.inv_inertia * rb.cross(n).length_squared());
            let p = n * (len / (wa + wb));
            a.x += p * a.inv_mass;
            a.q = turned(a.q, ra.cross(p) * a.inv_inertia);
            b.x -= p * b.inv_mass;
            b.q = turned(b.q, -rb.cross(p) * b.inv_inertia);
        }
        // the child bone's turn since death, in its own frame, within the ranges
        let l = j.limits;
        let now = angles(j.rest.inverse() * (a.q.inverse() * b.q * j.child_off));
        let held = Vec3::new(now.x.clamp(l[0].0, l[0].1), now.y.clamp(l[1].0, l[1].1), now.z.clamp(l[2].0, l[2].1));
        if (held - now).length_squared() < 1e-12 {
            return;
        }
        let want = a.q * j.rest * from_angles(held) * j.child_off.inverse();
        let e = want * b.q.inverse();
        let e = if e.w < 0.0 { -e } else { e };
        let turn = e.to_scaled_axis();
        let total = a.inv_inertia + b.inv_inertia;
        b.q = turned(b.q, turn * (b.inv_inertia / total));
        a.q = turned(a.q, -turn * (a.inv_inertia / total));
    }

    /// Contact `k` held: its point pushed out of the face's plane (its friction is the velocity
    /// pass's, `contact_velocities`: a sliding point's speed across the face taken away up to
    /// FRICTION x the push, as Ipion's contact solver works on speeds; a position-level grip held
    /// the flywheel-like bodies' turning back and slowed every fall).
    fn solve_contact(&mut self, k: usize) {
        let c = self.contacts[k];
        let b = &mut self.bodies[c.body];
        let r = b.q * c.point;
        let depth = c.d - c.slack - c.n.dot(b.x + r);
        if depth <= 0.0 {
            return;
        }
        let rn = r.cross(c.n);
        let push = depth / (b.inv_mass + b.inv_inertia * rn.length_squared());
        b.x += c.n * (push * b.inv_mass);
        b.q = turned(b.q, rn * (push * b.inv_inertia));
        self.contacts[k].lambda += push;
    }

    /// The contacts' velocity pass after a substep (XPBD's): a point pushed out of its face this
    /// substep leaves it at RESTITUTION x the speed it came in at (none below 2 g dt), whatever
    /// speed the push itself gave it, then friction: its speed across the face taken away, up to
    /// FRICTION x the substep's push as an impulse (so it holds where the push is enough, and
    /// slides slowing at 4 g where it isn't).
    fn contact_velocities(&mut self, h: f32) {
        for c in &self.contacts {
            if c.lambda <= 0.0 {
                continue;
            }
            let b = &mut self.bodies[c.body];
            let r = b.q * c.point;
            let at = b.x + r;
            let vn = c.n.dot(b.v + b.w.cross(r));
            let leave = if c.vn0 < -2.0 * GRAVITY * h { -RESTITUTION * c.vn0 } else { 0.0 };
            if (vn - leave).abs() > 1e-6 {
                let k = b.inv_mass + b.inv_inertia * r.cross(c.n).length_squared();
                b.impulse(at, c.n * ((leave - vn) / k));
            }
            let vp = b.v + b.w.cross(r);
            let vt = vp - c.n * c.n.dot(vp);
            let speed = vt.length();
            if speed > 1e-6 {
                let t = vt / speed;
                let k = b.inv_mass + b.inv_inertia * r.cross(t).length_squared();
                b.impulse(at, -t * (FRICTION * c.lambda / h).min(speed / k));
            }
        }
    }

    /// This step's contacts: each body's points against the level's faces round it
    /// (`Arena::faces_near`) that the body is in front of, over which the point is and near or
    /// under (up to two faces a point, the nearest); where the level has no collision, the flat
    /// fallback floor.
    fn find_contacts(&mut self) {
        self.contacts.clear();
        let arena = world::arena();
        let reach = |b: &Body| CONTACT_MARGIN + (b.v.length() + b.w.length() * b.reach) * STEP;
        // every body's box, and the faces round them all, once
        let boxes: Vec<(Vec3, Vec3)> = self.bodies.iter().map(|b| {
            let e = Vec3::splat(b.reach + reach(b));
            (b.x - e, b.x + e)
        }).collect();
        let faces: Vec<Face> = match arena.as_deref() {
            Some(a) => {
                let (lo, hi) = boxes.iter().fold((Vec3::MAX, Vec3::MIN), |(l, h), b| (l.min(b.0), h.max(b.1)));
                a.faces_near(lo, hi)
            }
            None => vec![],
        };
        for (bi, b) in self.bodies.iter().enumerate() {
            let (lo, hi) = boxes[bi];
            let near: Vec<(Face, Vec3)> = faces.iter().filter(|f| {
                let (flo, fhi) = (f.v[0].min(f.v[1]).min(f.v[2]), f.v[0].max(f.v[1]).max(f.v[2]));
                !(flo.cmpgt(hi).any() || fhi.cmplt(lo).any())
            }).filter_map(|f| {
                // (the side the body is on: a face holds what's in front of it)
                let n = if f.two_sided && (b.x - f.v[0]).dot(f.n) < 0.0 { -f.n } else { f.n };
                ((b.x - f.v[0]).dot(n) > 0.0).then_some((*f, n))
            }).collect();
            let gap = reach(b);
            for &(point, radius) in &b.points {
                let p = b.at(point);
                let mut found: Vec<(f32, Vec3, f32)> = vec![];
                if arena.is_none() {
                    let s = p.y - radius - self.floor;
                    if s < gap {
                        found.push((s, Vec3::Y, self.floor + radius));
                    }
                }
                for (f, n) in &near {
                    let s = (p - f.v[0]).dot(*n) - radius;
                    if s >= gap || s <= -CONTACT_DEPTH {
                        continue;
                    }
                    let foot = p - *n * (p - f.v[0]).dot(*n);
                    if closest_on_triangle(foot, f.v).distance(foot) > EDGE_SLACK {
                        continue;
                    }
                    found.push((s, *n, n.dot(f.v[0]) + radius));
                }
                found.sort_by(|x, y| x.0.total_cmp(&y.0));
                let mut kept: Vec<Vec3> = vec![];
                for (s, n, d) in found {
                    if kept.len() >= 2 || kept.iter().any(|k| k.dot(n) > 0.99) {
                        continue;
                    }
                    kept.push(n);
                    // (a point already under the face comes out of it gradually: `slack`)
                    self.contacts.push(Contact { body: bi, point, n, d, slack: (-s).max(0.0), lambda: 0.0, vn0: 0.0 });
                }
            }
        }
    }

    /// Every bone's frame now: one with a body takes its part's turn, at its place under its
    /// parent (bones keep their lengths); one without rides on its parent as it was at death; a
    /// root without a body rides on the root body.
    fn pose(&mut self) {
        let carry = self.bodies[0].frame() * self.root_frame0.inverse();
        for i in 0..self.posed.len() {
            let above = match self.parent[i] {
                Some(p) => self.posed[p],
                None => carry * self.frame,
            };
            self.posed[i] = match self.body_of[i] {
                Some((b, p)) => {
                    let (body, part) = (&self.bodies[b], &self.bodies[b].parts[p]);
                    let origin = match self.parent[i] {
                        Some(_) => above.transform_point3(self.local[i].w_axis.truncate()),
                        None => body.at(part.pos),
                    };
                    Mat4::from_rotation_translation(body.q * part.rot, origin)
                }
                None => above * self.local[i],
            };
        }
    }

    /// BF_RAGDOLL_LOG at the build: its bodies, joints (with which way a bend about -y takes the
    /// limb's far end, along the model's -z and up) and start.
    fn log_build(&self, start: Start) {
        let name = |b: Option<usize>| b.map_or("(the character's node)".to_string(), |i| bone_name(self.names[i]));
        let mass: f32 = self.bodies.iter().map(|b| b.mass).sum();
        let count: usize = self.bodies.iter().map(|b| b.parts.len()).sum();
        println!("ragdoll: {} bones, {count} of the game's bodies ({mass:.1} kg) in {} pieces, {} joints; {start:?}{}", self.posed.len(),
                 self.bodies.len(), self.joints.len(), if self.asleep { format!("; holding its pose {REST_WAKE} s") } else { String::new() });
        let (_, turn, _) = self.frame.to_scale_rotation_translation();
        let minus_z = turn * Vec3::NEG_Z;
        for b in &self.bodies {
            let parts: Vec<String> = b.parts.iter().map(|p| format!("{} {:.1} kg", name(p.bone), p.mass)).collect();
            let low = b.points.iter().map(|&(p, r)| b.at(p).y - r).fold(f32::MAX, f32::min) - self.floor_under(b.x);
            println!("  body [{}]: {:.1} kg, inertia {:.3}, {} points, {} triangles, {} balls, at {:.2} (lowest {low:.3} m over the floor), v {:.2}",
                     parts.join(", "), b.mass, 1.0 / b.inv_inertia, b.points.len(), b.tris.len(), b.balls.len(), b.x, b.v);
        }
        for j in &self.joints {
            let deg = |v: f32| v.to_degrees();
            let l = j.limits;
            // (a turn of -y about the child bone's own y takes its x, along the bone, toward its +z)
            let bend = self.bodies[j.child].q * j.child_off * Vec3::Z;
            println!("  joint {}: x {:.0}..{:.0} y {:.0}..{:.0} z {:.0}..{:.0}; -y takes its end {:.2} along the model's -z, {:.2} up", bone_name(j.name),
                     deg(l[0].0), deg(l[0].1), deg(l[1].0), deg(l[1].1), deg(l[2].0), deg(l[2].1), bend.dot(minus_z), bend.y);
        }
    }

    /// BF_RAGDOLL_LOG's line for this frame: the root body's place and speed, the fastest body and
    /// spin, the trunk's lowest point over the floor under it (the root's piece and the chest's),
    /// the worst joint gap, the contacts, how many bodies the last movement check found calm and
    /// still, asleep.
    pub fn log_line(&self, who: &str, since: f32) -> String {
        let root = &self.bodies[0];
        let chest = self.names.iter().position(|&b| b == SPINE2).and_then(|i| self.body_of[i]).map(|(b, _)| b);
        let trunk = [Some(0), chest].into_iter().flatten().map(|i| {
            let b = &self.bodies[i];
            b.points.iter().map(|&(p, r)| b.at(p).y - r).fold(f32::MAX, f32::min) - self.floor_under(b.x)
        }).fold(f32::MAX, f32::min);
        let gap = self.joints.iter().map(|j| {
            let (a, b) = (&self.bodies[j.parent], &self.bodies[j.child]);
            (b.at(j.at_child) - a.at(j.at_parent)).length()
        }).fold(0.0, f32::max);
        let label = |b: &Body| b.parts.first().map_or(String::new(), |p| p.bone.map_or("the node".to_string(), |i| bone_name(self.names[i])));
        let fastest = self.bodies.iter().max_by(|a, b| a.v.length().total_cmp(&b.v.length())).map_or((0.0, String::new()), |b| (b.v.length(), label(b)));
        let spin = self.bodies.iter().max_by(|a, b| a.w.length().total_cmp(&b.w.length())).map_or((0.0, String::new()), |b| (b.w.length(), label(b)));
        let calm = self.bodies.iter().filter(|b| b.state == 2).count();
        let still = self.bodies.iter().filter(|b| b.state == 3).count();
        format!("ragdoll {who} t {since:.2}: root at {:.2} going {:.2} m/s ({:.2} up), fastest body {:.2} m/s ({}), fastest spin {:.2} rad/s ({}), trunk {trunk:.3} m over the floor, worst joint gap {:.4} m, {} contacts, {calm} calm and {still} still of {}{}",
                self.root(), root.v.length(), root.v.y, fastest.0, fastest.1, spin.0, spin.1, gap, self.contacts.len(), self.bodies.len(),
                if self.asleep { ", asleep" } else { "" })
    }

    /// The bodies and joints for BF_RAGDOLL_DRAW: each body's surface edges and balls (a cross
    /// their size), the root's in red, and each joint's point (a line from its parent body's
    /// centre).
    fn draw(&self, gizmos: &mut Gizmos) {
        for (i, b) in self.bodies.iter().enumerate() {
            let colour = if i == 0 { Color::srgb(1.0, 0.2, 0.2) } else { Color::srgb(0.2, 1.0, 0.4) };
            for &(c, r) in &b.balls {
                for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                    let d = b.q * axis * r.max(0.03);
                    gizmos.line(b.at(c) - d, b.at(c) + d, Color::srgb(1.0, 0.9, 0.2));
                }
            }
            for t in &b.tris {
                let [p, q, r] = t.map(|c| b.at(c));
                gizmos.line(p, q, colour);
                gizmos.line(q, r, colour);
                gizmos.line(r, p, colour);
            }
        }
        for j in &self.joints {
            let a = &self.bodies[j.parent];
            gizmos.line(a.x, a.at(j.at_parent), Color::srgb(0.3, 0.6, 1.0));
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, draw_ragdolls.after(update_player).run_if(in_state(AppState::Playing)));
}

/// BF_RAGDOLL_DRAW=1: every ragdoll's bodies (green: a hit part's surface, yellow: a ball, red:
/// the root's piece) and joints (blue: from the parent body's centre to the joint).
fn draw_ragdolls(player: Res<Player>, squad: Res<Squad>, mut gizmos: Gizmos) {
    if std::env::var("BF_RAGDOLL_DRAW").is_err() {
        return;
    }
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        if let Some(r) = u.ragdoll.as_ref().or(u.knock.as_ref().map(|k| &k.ragdoll)) {
            r.draw(&mut gizmos);
        }
    }
}

/// The CBT_DEAD handler's shove requests for a hit that killed `u` (FUN_00115f90 -> FUN_000e00b0
/// at 0x11607a and 0x116124): the hit's, magnitude 2 x the damage `damage` of type `kind` at
/// `at` along `dir` with the weapon's push `push` as the factor on the part struck `bone`, and
/// the up-kick unless `u` is in the air. Only with DEATH_SHOVE (off: no take shows them).
pub fn death_shoves(u: &mut Player, bone: Option<u32>, at: Vec3, dir: Vec3, damage: f32, kind: i64, push: f32) {
    if !DEATH_SHOVE {
        return;
    }
    u.shoves.push(Shove { bone, point: at, dir, magnitude: 2.0 * damage, kind, factor: push });
    if !u.action.airborne() {
        u.shoves.push(Shove { bone: None, point: u.position, dir: Vec3::Y, magnitude: 1.0, kind: 0, factor: UP_KICK });
    }
}

/// A blast's push on `u` if it's a corpse with a ragdoll (FUN_00224a90: `Ragdoll::blast`), when
/// the blast's weapon pushes (`push`, its bullet's h_e39b994b, 1 or more).
pub fn blast_corpse(u: &mut Player, at: Vec3, radius: f32, damage: f32, push: f32) {
    if push < 1.0 || !u.dead {
        return;
    }
    if let Some(r) = u.ragdoll.as_mut() {
        r.blast(at, radius, damage);
    }
}

/// Two bodies of a slice at once (`a` and `b` differ).
fn pair(bodies: &mut [Body], a: usize, b: usize) -> (&mut Body, &mut Body) {
    if a < b {
        let (x, y) = bodies.split_at_mut(b);
        (&mut x[a], &mut y[0])
    } else {
        let (x, y) = bodies.split_at_mut(a);
        (&mut y[0], &mut x[b])
    }
}

/// `q` turned by the rotation vector `v` (world).
fn turned(q: Quat, v: Vec3) -> Quat {
    if v.length_squared() < 1e-18 {
        return q;
    }
    (Quat::from_scaled_axis(v) * q).normalize()
}

/// A relative turn's angles for FUN_00113120's ranges: its twist about x, then its swing's (the
/// rest, about an axis across x) parts about y and z, as a turn vector's.
fn angles(r: Quat) -> Vec3 {
    let r = if r.w < 0.0 { -r } else { r };
    let t = Quat::from_xyzw(r.x, 0.0, 0.0, r.w);
    let tl = t.length();
    let (twist, t) = if tl > 1e-6 {
        let t = t / tl;
        (2.0 * t.x.atan2(t.w), t)
    } else {
        (0.0, Quat::IDENTITY)
    };
    let s = r * t.inverse();
    let v = Vec3::new(s.x, s.y, s.z);
    let l = v.length();
    let swing = if l > 1e-9 { v * (2.0 * l.atan2(s.w) / l) } else { v * 2.0 };
    Vec3::new(twist, swing.y, swing.z)
}

/// The turn whose `angles` these are.
fn from_angles(a: Vec3) -> Quat {
    Quat::from_scaled_axis(Vec3::new(0.0, a.y, a.z)) * Quat::from_rotation_x(a.x)
}

/// A hit part's contact points: its surface's corners, once each, at most MAX_POINTS (the
/// furthest out along 26 ways round it when it has more).
fn corners(tris: &[[Vec3; 3]]) -> Vec<Vec3> {
    let mut out: Vec<Vec3> = vec![];
    for &c in tris.iter().flatten() {
        if !out.iter().any(|o| o.distance_squared(c) < 1e-6) {
            out.push(c);
        }
    }
    if out.len() <= MAX_POINTS {
        return out;
    }
    let centre = out.iter().sum::<Vec3>() / out.len() as f32;
    let mut kept: Vec<Vec3> = vec![];
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let way = Vec3::new(x as f32, y as f32, z as f32);
                if way == Vec3::ZERO {
                    continue;
                }
                let best = out.iter().copied().max_by(|a, b| (*a - centre).dot(way).total_cmp(&(*b - centre).dot(way)));
                if let Some(b) = best.filter(|b| !kept.contains(b)) {
                    kept.push(b);
                }
            }
        }
    }
    kept
}

/// The point of triangle `v` nearest `p` (by its regions: corners, edges, face).
fn closest_on_triangle(p: Vec3, [a, b, c]: [Vec3; 3]) -> Vec3 {
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    a + ab * (vb * denom) + ac * (vc * denom)
}

/// Where a line (from `o`, relative to the ball's centre, along unit `d`) first meets a ball of
/// radius `r`.
fn ray_ball(o: Vec3, d: Vec3, r: f32) -> Option<f32> {
    let b = o.dot(d);
    let disc = b * b - (o.length_squared() - r * r);
    (disc >= 0.0).then(|| -b - disc.sqrt()).filter(|&t| t > 0.0)
}

/// Where a line (from `o` along `d`) first meets one of `tris`, either side, within `max`
/// (Moller-Trumbore).
fn ray_tris(tris: &[[Vec3; 3]], o: Vec3, d: Vec3, max: f32) -> Option<f32> {
    tris.iter().filter_map(|&[a, b, c]| {
        let (e1, e2) = (b - a, c - a);
        let p = d.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-12 {
            return None;
        }
        let s = o - a;
        let u = s.dot(p) / det;
        let q = s.cross(e1);
        let v = d.dot(q) / det;
        let t = e2.dot(q) / det;
        ((0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && (0.0..=max).contains(&t)).then_some(t)
    }).min_by(|a, b| a.total_cmp(b))
}

/// A bone's name for the log (the Biped's that FUN_00113120 and the hit parts name), else its hash.
fn bone_name(bone: u32) -> String {
    const NAMES: [&str; 31] = ["Bip01", "Bip01 Pelvis", "Bip01 Spine", "Bip01 Spine1", "Bip01 Spine2", "Bip01 Spine3", "Bip01 Neck",
        "Bip01 Neck1", "Bip01 Head", "Bip01 L Clavicle", "Bip01 R Clavicle", "Bip01 L UpperArm", "Bip01 R UpperArm", "Bip01 L Forearm",
        "Bip01 R Forearm", "Bip01 L Hand", "Bip01 R Hand", "Bip01 L Thigh", "Bip01 R Thigh", "Bip01 L Calf", "Bip01 R Calf",
        "Bip01 L Foot", "Bip01 R Foot", "Bip01 L HorseLink", "Bip01 R HorseLink", "Bip01 Tail", "Bip01 Tail1", "Bip01 Tail2",
        "Bip01 L Toe0", "Bip01 R Toe0", "Bip01 Ponytail1"];
    NAMES.iter().find(|n| bf_viewer::bf::hash::h(n) == bone).map_or_else(|| format!("h_{bone:08x}"), |n| n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::hash::h;

    /// FUN_00113120's switch hashes are the Biped names the table gives them.
    #[test]
    fn limit_names() {
        for (name, y) in [("Bip01 L Thigh", (-60.0, 110.0)), ("Bip01 R Calf", (-90.0, 0.0)), ("Bip01 L Forearm", (-100.0, 0.0)),
                          ("Bip01 Spine2", (-45.0, 45.0)), ("Bip01 Head", (-45.0, 45.0)), ("Bip01 R UpperArm", (-40.0, 100.0)),
                          ("Bip01 Neck1", (-15.0, 50.0)), ("Bip01 L HorseLink", (-30.0, 30.0))] {
            let l = joint_limits(h(name)).unwrap_or_else(|| panic!("{name}"));
            assert!((l[1].0.to_degrees() - y.0).abs() < 1e-3 && (l[1].1.to_degrees() - y.1).abs() < 1e-3, "{name}");
        }
        for rigid in ["Bip01", "Bip01 Pelvis", "Bip01 Spine", "Bip01 Neck", "Bip01 L Clavicle"] {
            assert!(joint_limits(h(rigid)).is_none(), "{rigid}");
        }
        assert_eq!(h("Bip01 Spine2"), SPINE2);
    }

    /// `angles` and `from_angles` undo each other within the ranges.
    #[test]
    fn angles_round_trip() {
        for a in [Vec3::new(0.3, -0.5, 0.2), Vec3::new(-0.7, 1.2, -0.3), Vec3::new(0.0, -1.5, 0.0), Vec3::new(0.5, 0.0, 1.0)] {
            let b = angles(from_angles(a));
            assert!((a - b).length() < 1e-4, "{a} {b}");
        }
    }

    /// A box body 0.4 x 0.1 x 0.1 m (its eight corners) at `x`, of `mass` (a limb's inertia).
    fn bar(x: Vec3, mass: f32) -> Body {
        let points = [-0.2f32, 0.2].iter().flat_map(|&a| [-0.05f32, 0.05].iter().flat_map(move |&b| [-0.05f32, 0.05].map(|c| (Vec3::new(a, b, c), 0.0))))
            .collect();
        Body { parts: vec![Part { bone: None, rot: Quat::IDENTITY, pos: Vec3::ZERO, centre: Vec3::ZERO, mass }], x, q: Quat::IDENTITY, v: Vec3::ZERO,
               w: Vec3::ZERO, x0: x, q0: Quat::IDENTITY, mass, inv_mass: 1.0 / mass, inv_inertia: 1.0 / (LIMB_INERTIA * mass), points, tris: vec![],
               balls: vec![], reach: 0.21, short: (x, Quat::IDENTITY, 0.0), long: (x, Quat::IDENTITY, 0.0), state: 1 }
    }

    /// A ragdoll of these bodies and joints on the flat fallback floor at height 0 (each body's
    /// movement references where it is set up: `fresh`).
    fn test_ragdoll(bodies: Vec<Body>, joints: Vec<Joint>) -> Ragdoll {
        let mut r = Ragdoll { bodies, joints, body_of: vec![], parent: vec![], local: vec![], frame: Mat4::IDENTITY, root_frame0: Mat4::IDENTITY,
                              posed: vec![], owed: 0.0, floor: 0.0, contacts: vec![], clock: 0.0, countdown: CHECK_EVERY, asleep: false, wake_at: None,
                              age: 0.0, names: vec![] };
        fresh(&mut r);
        r
    }

    /// The movement check's references put where each body is now.
    fn fresh(r: &mut Ragdoll) {
        for b in r.bodies.iter_mut() {
            b.short = (b.x, b.q, r.clock);
            b.long = (b.x, b.q, r.clock);
        }
    }

    /// A joint of bars `parent` and `child` lying end to end along x, with these ranges.
    fn bar_joint(parent: usize, child: usize, limits: [(f32, f32); 3]) -> Joint {
        Joint { parent, child, at_parent: Vec3::new(0.2, 0.0, 0.0), at_child: Vec3::new(-0.2, 0.0, 0.0), rest: Quat::IDENTITY, child_off: Quat::IDENTITY,
                limits, name: 0 }
    }

    /// Two joined bodies lying on a flat floor stay put, hold their joint and go to sleep.
    #[test]
    fn rests_on_the_floor() {
        let mut r = test_ragdoll(vec![bar(Vec3::new(0.0, 0.05, 0.0), LIMB_MASS), bar(Vec3::new(0.4, 0.05, 0.0), LIMB_MASS)],
                                 vec![bar_joint(0, 1, [(-1.0, 1.0); 3])]);
        for _ in 0..132 {
            r.psi();
        }
        for b in &r.bodies {
            assert!((b.x.y - 0.05).abs() < 0.005 && b.x.x.abs() < 0.45, "{:?}", b.x);
        }
        assert!(r.asleep);
    }

    /// A body sliding on the floor stops within a few centimetres (friction 4: 39 m/s^2).
    #[test]
    fn slides_to_a_stop() {
        let mut b = bar(Vec3::new(0.0, 0.05, 0.0), LIMB_MASS);
        b.v = Vec3::new(2.0, 0.0, 0.0);
        let mut r = test_ragdoll(vec![b], vec![]);
        for _ in 0..66 {
            r.psi();
        }
        let b = &r.bodies[0];
        assert!(b.x.x > 0.02 && b.x.x < 0.15 && b.v.length() < 0.01 && b.q.angle_between(Quat::IDENTITY) < 0.05, "{:?} {:?} {:?}", b.x, b.v, b.q);
    }

    /// A body lying on the floor set spinning about the vertical stops within a second: friction 4
    /// at its corners holds it.
    #[test]
    fn spin_stops() {
        let mut b = bar(Vec3::new(0.0, 0.05, 0.0), LIMB_MASS);
        b.w = Vec3::new(0.0, 3.0, 0.0);
        let mut r = test_ragdoll(vec![b], vec![]);
        for _ in 0..66 {
            r.psi();
        }
        let b = &r.bodies[0];
        assert!(b.w.length() < 0.05 && b.v.length() < 0.01, "{:?} {:?}", b.w, b.v);
    }

    /// Ipion's movement check (FUN_002bb980, FUN_002be350): a body that doesn't move is frozen at
    /// the second check (calm at the first, still once calm for more than 0.3 s); one that creeps
    /// 6 mm a check (more than 1 cm every second check, so never calm) is frozen once 4 s have
    /// passed within 10 cm; one going 2 cm a check never is.
    #[test]
    fn movement_check_rules() {
        let frozen_at = |step: f32| {
            let mut r = test_ragdoll(vec![bar(Vec3::new(0.0, 0.05, 0.0), LIMB_MASS)], vec![]);
            (1..60).find_map(|k| {
                r.clock = k as f32 * CHECK_EVERY as f32 * STEP;
                r.bodies[0].x.x += step;
                r.movement_check();
                r.asleep.then_some(r.clock)
            })
        };
        let t = frozen_at(0.0).expect("a still body is frozen");
        assert!((0.5..0.6).contains(&t), "still: frozen at {t}");
        let t = frozen_at(0.006).expect("a creeping body is frozen");
        assert!((4.0..4.6).contains(&t), "creeping: frozen at {t}");
        assert_eq!(frozen_at(0.02), None, "a moving body is frozen");
    }

    /// A bar hung from a fixed point by one end swings with the period its mass and inertia give
    /// (2 pi sqrt((I + m r^2) / (m g r)): 2.42 s for the game's limb, I 1.125, m 4.5, r 0.2), so the
    /// solver adds no stiffness or drag of its own to a joint.
    #[test]
    fn pendulum_period() {
        let mut anchor = bar(Vec3::new(0.0, 5.0, 0.0), LIMB_MASS);
        anchor.inv_mass = 0.0;
        anchor.inv_inertia = 0.0;
        anchor.points.clear();
        let mut hung = bar(Vec3::new(0.0, 5.0 - 0.2, 0.0), LIMB_MASS);
        hung.points.clear();
        // (a swing of about 33 degrees about z, the bar pointing down from the anchor)
        let joint = Joint { parent: 0, child: 1, at_parent: Vec3::ZERO, at_child: Vec3::new(0.0, 0.2, 0.0), rest: Quat::IDENTITY, child_off: Quat::IDENTITY,
                            limits: [(-3.0, 3.0); 3], name: 0 };
        let mut r = test_ragdoll(vec![anchor, hung], vec![joint]);
        r.bodies[1].w = Vec3::new(0.0, 0.0, 1.5);
        r.bodies[1].v = Vec3::new(1.5 * 0.2, 0.0, 0.0);
        // (no movement check: a swing that keeps within 0.1 m of where it was for 4 s counts as
        // still, LONG_MOVE, and this one would be frozen before its second period)
        r.countdown = u32::MAX;
        // the times the bar passes the bottom going the first way
        let mut crossings = vec![];
        let mut was = r.bodies[1].x.x;
        for k in 0..400 {
            r.psi();
            let x = r.bodies[1].x.x;
            if was < 0.0 && x >= 0.0 {
                crossings.push(k as f32 * STEP);
            }
            was = x;
        }
        assert!(crossings.len() >= 2, "{crossings:?}");
        let period = crossings[1] - crossings[0];
        assert!((period - 2.42).abs() < 0.1, "period {period}");
    }

    /// A body tipped onto an edge rocks with ever less energy (no bounce feeds it: its edges
    /// leave the floor at RESTITUTION of the speed they met it at), and is asleep within 6 s. (Its
    /// isotropic inertia, the game's 1.125 kg m^2 on a 0.1 m bar, is a flywheel's: an edge's knock
    /// takes little of its spin, so damping and the rest test end it.)
    #[test]
    fn rocking_dies_down() {
        let mut b = bar(Vec3::new(0.0, 0.1, 0.0), LIMB_MASS);
        b.q = Quat::from_rotation_x(0.6);
        b.w = Vec3::new(2.0, 0.0, 0.5);
        let mut r = test_ragdoll(vec![b], vec![]);
        let energy = |r: &Ragdoll| {
            let b = &r.bodies[0];
            0.5 * b.mass * b.v.length_squared() + 0.5 / b.inv_inertia * b.w.length_squared() + b.mass * GRAVITY * b.x.y
        };
        let mut last = energy(&r);
        for second in 0..6 {
            for _ in 0..66 {
                r.psi();
            }
            let e = energy(&r);
            assert!(r.asleep || e < last, "energy {e} after {} s, was {last}", second + 1);
            last = e;
        }
        assert!(r.asleep, "{:?} {:?}", r.bodies[0].w, r.bodies[0].v);
    }

    /// A chain of four bodies dropped onto the floor with their joints at a limit comes to rest
    /// where it lands: no creep along the floor once down, asleep within 3 s.
    #[test]
    fn chain_settles() {
        let bodies = (0..4).map(|k| bar(Vec3::new(0.4 * k as f32, 0.6, 0.0), LIMB_MASS)).collect();
        // each joint may bend 0..60 degrees about z (down) and nothing else: the chain lands bent
        let limits = [(0.0, 0.0), (0.0, 0.0), (-1.05, 0.0)];
        let mut r = test_ragdoll(bodies, (0..3).map(|k| bar_joint(k, k + 1, limits)).collect());
        for b in r.bodies.iter_mut() {
            b.w = Vec3::new(0.0, 0.0, -1.5);
        }
        for _ in 0..66 {
            r.psi();
        }
        let landed: Vec<Vec3> = r.bodies.iter().map(|b| b.x).collect();
        for _ in 0..132 {
            r.psi();
        }
        for (b, l) in r.bodies.iter().zip(&landed) {
            assert!(b.x.distance(*l) < 0.03, "crept from {l} to {}", b.x);
        }
        assert!(r.asleep, "still moving after 3 s");
    }
}
