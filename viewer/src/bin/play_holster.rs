//! Edward Kingman's two pistols on his left hip, and the swap between them (#409, **demo-own**: the
//! product owner's request, after #407 gave the demo's Kingman class a second Kman Auto).
//!
//! Kingman's compound h_e0b89937 has no stow hardpoint at all (objects-m03_c.xml ~9868: its 17
//! hardpoints are the hands' TRIGGER_HAND and SUPPORT_HAND, the head's and the bones' effect
//! points), so a gun that isn't in his hand was hidden. The demo gives the class a holster, on his
//! **left hip**, which **whichever pistol isn't in his right hand** hangs from (the product owner's
//! rule: the second gun always comes from the nearer hip, the left, the one his left hand is at in
//! berserk; the first version of #409 gave each gun a hip of its own, and a gun left on the right
//! hip crossed his body to his left hand when the berserk was drawn after a swap). Both of the
//! game's stow places the two guns' slots start on (`StowPlaces`: place 0 the first gun's, 1 the
//! second's) are this one holster, so the right hip stays empty:
//!
//!  - It is a stow hardpoint on `Bip01 L Thigh` made by mirroring Flint's (`Holsters::new`,
//!    `MirrorArm::left_hip`: the same mirror as the left hand's mount, with the thighs' own
//!    reflection). Flint is the squad member who wears a pistol on his hip: his place 1,
//!    h_f4ccf78c on his right thigh, objects-common.xml h_160a1622, where his LZR-10 hangs barrel
//!    down. Its point is scaled by the ratio of the two thighs' lengths (Kingman's 0.511 m to
//!    Flint's 0.457) and moved back by `HIP_SHIFT` (a fit: clear of where his empty left hand rests).
//!    The pistols' HOLSTER hardpoints h_10ab40ac (the four in objects-dm.xml and objects-m03_c.xml,
//!    the Kman Auto's among them) are all turned the same way (ORIENTATION 0 0 1 0; the rifles'
//!    another), so the Kman Auto hangs from it barrel down as Flint's LZR-10 does.
//!  - The berserk draw takes the second gun from it into his left hand, and the stow puts it back.
//!
//! The demo gives it to the type that carries the Berserker (skill-set 3) on a model without the
//! stow hardpoints (`Holsters::new`), so Tex's own holsters (his three hardpoints, the ramps) and
//! every other model's are as they were.
//!
//! **The swap**: Kingman's motion set has no weapon switch clip (Flint's and Hawk's
//! `Sc_w1_2_w2` is the pistols' own, ms_kingman has none), so the switch had been instant, the gun
//! in hand gone and the other there. The demo gives him a switch of its own, with the squad's
//! machinery (`play.rs` `start_switch`, `Switching`) but no clip to play over his body: the drawn gun
//! is let go at `SWAP_DROP` and goes to the left hip, the other is taken at `SWAP_GRAB` and comes
//! out of it into his right hand, each over `GLIDE_TIME` (`Glides`), the aim and the shots held
//! back until `SWAP_TIME`. The two guns trade places, so they must not pass through each other:
//! the one going to the hip crosses in front of his body (`BOW_IN_FRONT`) and the one coming out of
//! it goes round his back (`BOW_BEHIND`), a whole body apart where they pass, and each is clear
//! of the hand or the hip it leaves before the other arrives.
//!
//! **The glides**: whenever a gun's place changes between a hand and a hip (the swap, the Berserker's
//! draw and stow), its model moves from where it was to where it goes over `GLIDE_TIME`, in the
//! model's space, following the bones it leaves and the one it goes to, along an arc that bows out
//! of the straight line as far as it crosses his body, so it neither pops nor goes through his
//! coat. Only characters with `Holsters` glide; every other swaps places at once as before.

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;
use bf_viewer::bf::weapon::{self, Hardpoint};

use super::mirror_arm::MirrorArm;

/// A gun's place under a joint: (bone, rotation, translation), as play.rs `HeldWeapon::in_hand`'s.
pub type Mount = (usize, Quat, Vec3);

/// The skill-set (default.xbe's CSKILL_ table 0x3bd814) of Tex's Berserker, which the demo's
/// Kingman class carries (character_class.rs `BERSERKER_DEMO`): the type that has it, on a model
/// with no stow hardpoints of its own, gets the holsters.
const BERSERKER_SET: i64 = 3;

/// Flint's stow hardpoint h_f4ccf78c on `Bip01 R Thigh` (his place 1, where his LZR-10 hangs;
/// objects-common.xml h_160a1622, the squad's models): its point in the thigh's frame, in metres
/// (x down the bone).
const FLINT_HIP_POINT: Vec3 = Vec3::new(-0.143_255, 0.121_38, 0.087_056);
/// ... and its ORIENTATION, w x y z as the data stores it.
const FLINT_HIP_ROT: [f32; 4] = [0.491_457, 0.464_461, 0.457_451, 0.577_481];
/// Flint's thigh's length, `Bip01 R Thigh` to `Bip01 R Calf` (objects-common.xml h_160a1622's joint,
/// parent-point -0.45675): what the hardpoint's point is scaled against.
const FLINT_THIGH: f32 = 0.456_75;
/// How far (m, in the right thigh's frame that the mirror turns into the left's: y out from the
/// body, z forward) the holster is moved from Flint's scaled point: 0.10 m back, to the rear of the
/// thigh. **A fit** to the frames of the demo (#409), not the data's. The first version chose it so
/// that the gun in his hand and the one on the same hip stood apart after a swap (no gun is on his
/// right hip now), and it is kept for what it still does: Flint's holster sits where a hand hangs
/// (Kingman's rests 0.18 m out from the hip joint and 0.08 m forward, the scaled holster 0.14 m out
/// and 0.1 m forward), so his empty left hand would hang inside the gun and the berserk draw would
/// be no move at all; moved back, the gun stands behind the hand, on the rear of the outer thigh,
/// and the draw is a reach back to it.
const HIP_SHIFT: Vec2 = Vec2::new(0.0, -0.10);

/// The seconds a gun takes between a hand and a hip. **The demo's choice**: a pistol drawn in a
/// quarter of a second, a little under the squad's pistol switches (Flint's `Sc_w1_2_w2` takes
/// 1.1 s for the whole change).
pub const GLIDE_TIME: f32 = 0.25;
/// The swap of Kingman's two pistols (the demo's own switch, see the module notes): the time the
/// gun in hand is let go, and goes to the left hip, the time the other is taken, and the swap's
/// length. **The demo's choice**: the second glide starts a tenth of a second after the first, so
/// the holstering reads before the draw (the two never meet: they take opposite arcs).
pub const SWAP_DROP: f32 = 0.0;
pub const SWAP_GRAB: f32 = 0.1;
pub const SWAP_TIME: f32 = SWAP_GRAB + GLIDE_TIME;
/// How far a glide to a hip bows out in front of the body (m) for each metre it goes across it.
/// **The demo's choice**: 0.5, so a gun going from his right hand to his left hip (0.5 m across)
/// crosses 0.25 m in front of his coat (the frames show it clear of it), and a glide down the
/// same side, a stow from the left hand, hardly bows.
const BOW_IN_FRONT: f32 = 0.5;
/// The same for a glide to a hand, which goes the other way, round his back, so that the gun
/// going to the hip and the one coming out of it never pass through each other: 0.9, more than the
/// front's, as the long coat's tails and his hips reach farther behind the line of the glide than
/// the coat's front reaches before it (**a fit** to frames from above and the side, #409).
const BOW_BEHIND: f32 = 0.9;
/// The model's forward direction, which the bow goes along (`yaw_of` in play.rs: a heading of
/// atan2(-x, -z), so forward is -z in the model's space).
const FORWARD: Vec3 = Vec3::NEG_Z;

/// The holster of a model (see the module notes).
#[derive(Clone, Debug)]
pub struct Holsters {
    /// the model's arms and thighs, whose mirror makes the left hip
    arm: MirrorArm,
    /// the stow hardpoint as Flint's right thigh has it: the right thigh's bone and the point in
    /// its frame, which `MirrorArm::left_hip` turns over to the left thigh
    right: (usize, Hardpoint),
}

impl Holsters {
    /// The holster of `model` when its type carries the Berserker (`skill_set` 3) and the model has
    /// none of the three stow hardpoints (`weapon::STOW`) or the SECOND_HAND of Tex's: its `arm`,
    /// the mirror of its right side, makes the left hip's. None for every other model and type.
    pub fn new(model: &Character, arm: &MirrorArm, skill_set: Option<i64>) -> Option<Self> {
        if skill_set != Some(BERSERKER_SET) || weapon::STOW.iter().chain([&weapon::SECOND_HAND]).any(|k| model.hardpoints.contains_key(k)) {
            return None;
        }
        let thigh = arm.right_thigh()?;
        let calf = model.bones.iter().position(|b| *b == h("Bip01 R Calf")).filter(|&c| model.parent[c] == Some(thigh))?;
        // (the thigh's length: where the bind pose puts its calf, in its frame)
        let scale = model.bind_local()[calf].1.length() / FLINT_THIGH;
        let [w, x, y, z] = FLINT_HIP_ROT;
        let point = FLINT_HIP_POINT * scale + Vec3::new(0.0, HIP_SHIFT.x, HIP_SHIFT.y);
        Some(Holsters { arm: arm.clone(), right: (thigh, Hardpoint { point, rot: Quat::from_xyzw(x, y, z, w).normalize() }) })
    }

    /// The places a gun hangs at, the entries of `HeldWeapon::stow`: its HOLSTER hardpoint `holster`
    /// on the left hip's hardpoint, the mirror of Flint's right one (`MirrorArm::left_hip`, with the
    /// gun's `barrel`, its firing direction, and `offset`, the muzzle's place from the grip's, both
    /// in its own axes), for **both the places the guns' slots start on**, 0 and 1, so whichever
    /// pistol isn't in his hand hangs on his left hip and the right hip is empty; none for the
    /// place 2, which Tex alone has.
    pub fn mounts(&self, holster: &Hardpoint, barrel: Vec3, offset: Vec3) -> [Option<Mount>; 3] {
        let (bone, hip) = self.right;
        let (r, t) = hip.mount(holster);
        let left = self.arm.left_hip((bone, r, t), barrel, offset);
        [left, left, None]
    }
}

/// How a gun's glide starts (see `Glide`).
#[derive(Clone, Copy, Debug)]
enum Start {
    /// from a place under a joint, which it follows as the joint moves
    Joint(Mount),
    /// from a place in the model's space, where a glide cut short by another got to
    Model(Mat4),
}

/// A gun on its way between two places.
#[derive(Clone, Copy, Debug)]
struct Glide {
    from: Start,
    /// the place it goes to, under the bone it hangs from
    to: Mount,
    /// how far it bows out of the straight line for each metre it goes across the body, along his
    /// forward: `BOW_IN_FRONT` to a hip, `-BOW_BEHIND` (round his back) to a hand
    bow: f32,
    /// started this frame by `Glides::put`, whose placement (the start's transform) the frame's
    /// deferred commands write over any step: the first `Glides::step` after it doesn't count
    fresh: bool,
    /// seconds gone
    time: f32,
}

/// One weapon's record: the gun entity it was last put at a place for, where, and its glide.
#[derive(Clone, Debug, Default)]
struct Track {
    /// the weapon's model entity the record is of (a gun replaced by a pickup is a new one)
    entity: Option<Entity>,
    /// the place it was last put at (None: hidden)
    at: Option<Mount>,
    /// the glide it is on, if any
    glide: Option<Glide>,
    /// where it was this frame in the model's space, for a glide cut short
    model: Mat4,
}

/// What a character's guns glide from and to (`Glides::put`, `Glides::step`), by weapon.
#[derive(Clone, Debug, Default)]
pub struct Glides(Vec<Track>);

/// What to do with a gun that is put at a place (`Glides::put`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Put {
    /// place it at once at the place, as a gun without glides is
    Now,
    /// it starts to glide: place it under the new bone with this transform (where it was)
    Glide(Transform),
    /// it is gliding there already: leave it
    Keep,
}

impl Glides {
    /// Weapon `i`'s gun `entity` is to hang at `to` now (None: not shown), `hip` whether that place
    /// is a holster rather than a hand, `world` the model's bones' matrices this frame (empty before
    /// the model is posed). A gun that moves from one place to another, both shown, starts to glide
    /// there, from where it was drawn if it was gliding already; one new, hidden, or put at the
    /// place it's at is placed at once. A glide to a hip bows in front of the body, one to a hand
    /// round the back (`BOW_IN_FRONT`, `BOW_BEHIND`), so two guns trading places don't meet.
    pub fn put(&mut self, i: usize, entity: Entity, to: Option<Mount>, hip: bool, world: &[Mat4]) -> Put {
        if self.0.len() <= i {
            self.0.resize(i + 1, Track::default());
        }
        let track = &mut self.0[i];
        if track.entity != Some(entity) {
            *track = Track { entity: Some(entity), at: to, ..Default::default() };
            return Put::Now;
        }
        let was = std::mem::replace(&mut track.at, to);
        let Some(to) = to else {
            track.glide = None;
            return Put::Now;
        };
        if was == Some(to) {
            return if track.glide.is_some() { Put::Keep } else { Put::Now };
        }
        let (Some(was), false) = (was, world.is_empty()) else {
            track.glide = None;
            return Put::Now;
        };
        let from = if track.glide.is_some() { Start::Model(track.model) } else { Start::Joint(was) };
        let glide = Glide { from, to, bow: if hip { BOW_IN_FRONT } else { -BOW_BEHIND }, fresh: true, time: 0.0 };
        let start = local(&glide, 0.0, world);
        track.model = world[to.0] * start.compute_matrix();
        track.glide = Some(glide);
        Put::Glide(start)
    }

    /// Moves weapon `i`'s glide on by `dt` seconds in the pose `world`: the transform the gun has
    /// under the bone it goes to this frame, the place itself on the frame it arrives. None when it
    /// isn't gliding.
    pub fn step(&mut self, i: usize, dt: f32, world: &[Mat4]) -> Option<Transform> {
        let track = self.0.get_mut(i)?;
        let glide = track.glide.as_mut()?;
        if world.is_empty() {
            return None;
        }
        // (the frame it started on shows the start's transform, which `put`'s deferred placement
        // writes after this step; the time counts from the next frame, so the glide takes its whole
        // time from the first frame it moves)
        if std::mem::take(&mut glide.fresh) {
            return None;
        }
        glide.time += dt;
        let s = (glide.time / GLIDE_TIME).clamp(0.0, 1.0);
        let at = *glide;
        track.model = world[at.to.0] * Mat4::from_rotation_translation(at.to.1, at.to.2);
        if s >= 1.0 {
            track.glide = None;
            return Some(Transform::from_translation(at.to.2).with_rotation(at.to.1));
        }
        let transform = local(&at, s, world);
        track.model = world[at.to.0] * transform.compute_matrix();
        Some(transform)
    }

    /// Whether weapon `i`'s gun is on its way between two places (so nothing else places it).
    pub fn gliding(&self, i: usize) -> bool {
        self.0.get(i).is_some_and(|t| t.glide.is_some())
    }

    /// A line for BF_WEAPON_LOG: where weapon `i`'s glide goes from and to, None when it isn't gliding.
    pub fn describe(&self, i: usize, bones: &[u32]) -> Option<String> {
        let glide = self.0.get(i)?.glide?;
        let from = match glide.from {
            Start::Joint((b, ..)) => format!("bone {:08x}", bones[b]),
            Start::Model(_) => "where it was".to_string(),
        };
        Some(format!("glides from {from} to bone {:08x} over {GLIDE_TIME} s, {}", bones[glide.to.0],
                     if glide.bow >= 0.0 { "across the front of him" } else { "round his back" }))
    }
}

/// A glide's transform at `s` (0 - 1) under the bone it goes to: the model-space places it goes from
/// and to (`Start`, `Glide::to` on the bones in `world`), the position eased in a line and bowed
/// along his forward (in front if `Glide::bow` is positive, behind if not) by that much for each
/// metre it goes across the body, the rotation slerped, then put into the destination bone's frame.
fn local(g: &Glide, s: f32, world: &[Mat4]) -> Transform {
    let place = |(b, r, t): Mount| world[b] * Mat4::from_rotation_translation(r, t);
    let from = match g.from {
        Start::Joint(m) => place(m),
        Start::Model(m) => m,
    };
    let to = place(g.to);
    let (_, from_rot, from_at) = from.to_scale_rotation_translation();
    let (_, to_rot, to_at) = to.to_scale_rotation_translation();
    let ease = s * s * (3.0 - 2.0 * s);
    // (across the body: the model's x, which a bow in front of it is square to)
    let across = (from_at - to_at).dot(FORWARD.cross(Vec3::Y)).abs();
    let at = from_at.lerp(to_at, ease) + FORWARD * (g.bow * across * (std::f32::consts::PI * ease).sin());
    let model = Mat4::from_rotation_translation(from_rot.slerp(to_rot, ease), at);
    let (_, rotation, translation) = (world[g.to.0].inverse() * model).to_scale_rotation_translation();
    Transform::from_translation(translation).with_rotation(rotation)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model of two bones: 0 at the origin, 1 a metre along x.
    fn world() -> Vec<Mat4> {
        vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::X)]
    }

    /// The place `(bone, no turn, at)`.
    fn at(bone: usize, at: Vec3) -> Mount {
        (bone, Quat::IDENTITY, at)
    }

    /// A gun new to the record, or put at a place it's at, is placed at once; one that goes from a
    /// shown place to another glides, from where it was.
    #[test]
    fn a_gun_that_moves_glides_from_where_it_was() {
        let mut g = Glides::default();
        let e = Entity::from_raw(7);
        assert_eq!(g.put(0, e, Some(at(0, Vec3::ZERO)), true, &world()), Put::Now);
        assert_eq!(g.put(0, e, Some(at(0, Vec3::ZERO)), true, &world()), Put::Now);
        let Put::Glide(first) = g.put(0, e, Some(at(1, Vec3::ZERO)), false, &world()) else { panic!("no glide") };
        // (it starts where it was: the first bone's origin, which the second bone's frame has at -1 along x)
        assert!((first.translation - Vec3::NEG_X).length() < 1e-5, "{first:?}");
        assert!(g.gliding(0));
        // (the same place again leaves the glide alone)
        assert_eq!(g.put(0, e, Some(at(1, Vec3::ZERO)), false, &world()), Put::Keep);
    }

    /// A gun hidden, or one before the model is posed, or another gun's entity, doesn't glide.
    #[test]
    fn a_gun_hidden_or_new_is_placed_at_once() {
        let mut g = Glides::default();
        let e = Entity::from_raw(1);
        g.put(0, e, Some(at(0, Vec3::ZERO)), true, &world());
        assert_eq!(g.put(0, e, None, true, &world()), Put::Now);
        assert_eq!(g.put(0, e, Some(at(1, Vec3::ZERO)), false, &world()), Put::Now);
        assert_eq!(g.put(0, e, Some(at(0, Vec3::ZERO)), true, &[]), Put::Now);
        assert_eq!(g.put(0, Entity::from_raw(2), Some(at(1, Vec3::ZERO)), false, &world()), Put::Now);
        assert!(!g.gliding(0));
    }

    /// The glide lasts `GLIDE_TIME` from the first frame it moves: the step of the frame it is put on
    /// isn't taken (the deferred placement of that frame writes the start's transform over it), it is
    /// on the way before the end, and on the frame it arrives its transform is the place itself and
    /// it stops.
    #[test]
    fn a_glide_arrives_at_its_time() {
        let mut g = Glides::default();
        let e = Entity::from_raw(3);
        g.put(0, e, Some(at(0, Vec3::ZERO)), true, &world());
        g.put(0, e, Some(at(1, Vec3::new(0.0, 0.5, 0.0))), false, &world());
        assert!(g.step(0, GLIDE_TIME, &world()).is_none(), "the frame it was put on");
        assert!(g.gliding(0));
        let mid = g.step(0, GLIDE_TIME * 0.5, &world()).unwrap();
        assert!(mid.translation.x < 0.0 && mid.translation.x > -1.0, "{mid:?}");
        assert!(g.gliding(0));
        let end = g.step(0, GLIDE_TIME, &world()).unwrap();
        assert_eq!(end.translation, Vec3::new(0.0, 0.5, 0.0));
        assert!(!g.gliding(0));
        assert!(g.step(0, 0.1, &world()).is_none());
    }

    /// A glide cut short by another starts the new one where the first had got to, not from a
    /// place it never was at.
    #[test]
    fn a_glide_cut_short_goes_on_from_where_it_got_to() {
        let mut g = Glides::default();
        let e = Entity::from_raw(4);
        g.put(0, e, Some(at(0, Vec3::ZERO)), true, &world());
        g.put(0, e, Some(at(1, Vec3::ZERO)), false, &world());
        g.step(0, 0.0, &world());
        let on_the_way = g.step(0, GLIDE_TIME * 0.5, &world()).unwrap();
        let Put::Glide(restart) = g.put(0, e, Some(at(0, Vec3::new(0.0, 0.3, 0.0))), true, &world()) else { panic!("no glide") };
        // (under the first bone now: the model-space place it was at, the same under either)
        let was = world()[1] * on_the_way.compute_matrix();
        assert!((was.transform_point3(Vec3::ZERO) - restart.translation).length() < 1e-4, "{was:?} {restart:?}");
    }

    /// A glide across the body bows out in front of it, one down its own side hardly does.
    #[test]
    fn a_glide_across_the_body_bows_out_in_front() {
        let across = Glide { from: Start::Joint(at(0, Vec3::new(-0.5, 0.0, 0.0))), to: at(0, Vec3::new(0.5, 0.0, 0.0)), bow: BOW_IN_FRONT, fresh: false, time: 0.0 };
        let mid = local(&across, 0.5, &world());
        assert!((mid.translation - Vec3::new(0.0, 0.0, -BOW_IN_FRONT)).length() < 1e-5, "{mid:?}");
        let along = Glide { from: Start::Joint(at(0, Vec3::new(0.0, 0.0, 0.0))), to: at(0, Vec3::new(0.0, -0.4, 0.0)), bow: BOW_IN_FRONT, fresh: false, time: 0.0 };
        let mid = local(&along, 0.5, &world());
        assert!((mid.translation - Vec3::new(0.0, -0.2, 0.0)).length() < 1e-5, "{mid:?}");
    }

    /// `put` makes a glide to a hip bow in front of the body and one to a hand behind it.
    #[test]
    fn a_glide_to_a_hip_goes_in_front_and_to_a_hand_behind() {
        let mut g = Glides::default();
        let e = Entity::from_raw(5);
        g.put(0, e, Some(at(0, Vec3::new(0.3, 0.0, 0.0))), false, &world());
        g.put(0, e, Some(at(0, Vec3::new(-0.25, 0.0, 0.0))), true, &world());
        g.step(0, 0.0, &world());
        assert!(g.step(0, GLIDE_TIME * 0.5, &world()).unwrap().translation.z < -0.2, "to a hip: in front");
        g.step(0, GLIDE_TIME, &world());
        g.put(0, e, Some(at(0, Vec3::new(0.3, 0.0, 0.0))), false, &world());
        g.step(0, 0.0, &world());
        assert!(g.step(0, GLIDE_TIME * 0.5, &world()).unwrap().translation.z > 0.3, "to a hand: behind");
        assert!(g.describe(0, &[1, 2]).unwrap().contains("round his back"));
    }

    /// Two guns trading places, one to a hip and one to a hand, take opposite arcs, the one to the
    /// hip in front of the body and the one to the hand behind it, so they are never in one place:
    /// half way they are a body apart, and at no time of the swap, the second glide starting
    /// `SWAP_GRAB` after the first, are they within a hand's width of each other.
    #[test]
    fn two_guns_trading_places_never_meet() {
        let (hand, hip) = (Vec3::new(0.3, 0.0, 0.0), Vec3::new(-0.25, 0.0, 0.05));
        let holster = Glide { from: Start::Joint(at(0, hand)), to: at(0, hip), bow: BOW_IN_FRONT, fresh: false, time: 0.0 };
        let draw = Glide { from: Start::Joint(at(0, hip)), to: at(0, hand), bow: -BOW_BEHIND, fresh: false, time: 0.0 };
        let (a, b) = (local(&holster, 0.5, &world()).translation, local(&draw, 0.5, &world()).translation);
        assert!(a.z < -0.2 && b.z > 0.3, "{a:?} {b:?}");
        let mut closest = f32::MAX;
        for k in 0..=100 {
            let t = k as f32 * SWAP_TIME / 100.0;
            let place = |g: &Glide, start: f32| local(g, ((t - start) / GLIDE_TIME).clamp(0.0, 1.0), &world()).translation;
            closest = closest.min(place(&holster, SWAP_DROP).distance(place(&draw, SWAP_GRAB)));
        }
        assert!(closest > 0.1, "the guns came within {closest} m of each other");
    }

    /// The holster is the left hip for both of the places the guns' slots start on and nothing for
    /// the third: a pistol that isn't in his hand hangs on the left, whichever it is.
    #[test]
    fn both_guns_hang_on_the_left_hip() {
        let k = Mat3::from_cols(-Vec3::X, Vec3::Y, Vec3::Z);
        let hips = Holsters { arm: MirrorArm::symmetric(k), right: (9, Hardpoint { point: Vec3::new(-0.16, 0.12, -0.01), rot: Quat::from_rotation_x(0.5) }) };
        let holster = Hardpoint { point: Vec3::new(0.0, 0.1, -0.17), rot: Quat::from_rotation_y(std::f32::consts::PI) };
        let stow = hips.mounts(&holster, Vec3::Z, Vec3::new(0.0, 0.1, 0.4));
        assert_eq!(stow[0].expect("the first gun's place").0, 10, "the left thigh");
        assert_eq!(stow[1], stow[0], "the second gun's place is the same hip");
        assert!(stow[2].is_none());
    }
}
