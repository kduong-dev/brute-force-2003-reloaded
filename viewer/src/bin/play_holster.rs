//! Edward Kingman's two pistols on his hips, and the swap between them (#409, **demo-own**: the
//! product owner's request, after #407 gave the demo's Kingman class a second Kman Auto).
//!
//! Kingman's compound h_e0b89937 has no stow hardpoint at all (objects-m03_c.xml ~9868: its 17
//! hardpoints are the hands' TRIGGER_HAND and SUPPORT_HAND, the head's and the bones' effect
//! points), so a gun that isn't in his hand was hidden. The demo gives the class two holsters:
//!
//!  - **The right hip** (`Holsters::new`): a stow hardpoint on `Bip01 R Thigh`, Flint's, the squad
//!    member who wears a pistol on his hip (his place 1, h_f4ccf78c on his right thigh,
//!    objects-common.xml h_160a1622: his LZR-10 hangs there barrel down), its point scaled by the
//!    ratio of the two thighs' lengths (Kingman's 0.511 m to Flint's 0.457) and moved back by
//!    `HIP_SHIFT` (a fit, to keep the gun in his hand and the one on the same hip apart after a
//!    swap). The pistols' HOLSTER hardpoints h_10ab40ac (the four in objects-dm.xml and
//!    objects-m03_c.xml, the Kman Auto's among them) are all turned the same way (ORIENTATION 0 0 1
//!    0; the rifles' another), so the Kman Auto hangs from it barrel down as Flint's LZR-10 does. The
//!    first gun's place is the game's place 0 (`StowPlaces`, the front's constructor's slot 1), so
//!    it hangs there while it isn't drawn.
//!  - **The left hip**: the right one mirrored across his body (`MirrorArm::left_hip`: the same
//!    mirror as the left hand's mount, with the thighs' own reflection). The second gun's place is
//!    the game's place 1, so it hangs there while it isn't drawn; drawn into his left hand in
//!    berserk it comes from there, and goes back to it at the end.
//!
//! The demo gives them to the type that carries the Berserker (skill-set 3) on a model without the
//! stow hardpoints (`Holsters::new`), so Tex's own holsters (his three hardpoints, the ramps) and
//! every other model's are as they were.
//!
//! **The swap**: Kingman's motion set has no weapon switch clip (Flint's and Hawk's
//! `Sc_w1_2_w2` is the pistols' own, ms_kingman has none), so the switch had been instant, the gun
//! in hand gone and the other there. The demo gives him a switch of its own, with the squad's
//! machinery (`play.rs` `start_switch`, `Switching`) but no clip to play over his body: the drawn gun
//! is let go at `SWAP_DROP` and goes to its hip, the other is taken at `SWAP_GRAB` and comes from
//! its hip, each over `GLIDE_TIME` (`Glides`), the aim and the shots held back until `SWAP_TIME`.
//!
//! **The glides**: whenever a gun's place changes between a hand and a hip (the swap, the Berserker's
//! draw and stow), its model moves from where it was to where it goes over `GLIDE_TIME`, in the
//! model's space, following the bones it leaves and the one it goes to, along an arc that bows out
//! in front of the body as far as it crosses it, so it neither pops nor goes through his coat. Only
//! characters with `Holsters` glide; every other swaps places at once as before.

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
/// How far (m, in the thigh's frame: y out from the body, z forward) the holster is moved from
/// Flint's scaled point: 0.10 m back, to the rear of the thigh. **A fit** to the frames of the demo
/// (#409), not the data's: after a swap both pistols are on his right side, and the gun in his hand
/// hangs at the very place Flint's holster sits (his hand rests 0.18 m out from the hip joint and
/// 0.08 m forward, the holster 0.14 m out and 0.1 m forward), so the two overlapped; moved back, they
/// stand side by side down the outside of the thigh, the holstered one behind.
const HIP_SHIFT: Vec2 = Vec2::new(0.0, -0.10);

/// The seconds a gun takes between a hand and a hip. **The demo's choice**: a pistol drawn in a
/// quarter of a second, a little under the squad's pistol switches (Flint's `Sc_w1_2_w2` takes
/// 1.1 s for the whole change).
pub const GLIDE_TIME: f32 = 0.25;
/// The swap of Kingman's two pistols (the demo's own switch, see the module notes): the time the
/// gun in hand is let go, and goes to its hip, and the time the other is taken, with its length.
/// **The demo's choice**: one glide after the other, so the two guns aren't in one hand together.
pub const SWAP_DROP: f32 = 0.0;
pub const SWAP_GRAB: f32 = GLIDE_TIME;
pub const SWAP_TIME: f32 = 2.0 * GLIDE_TIME;
/// How far a glide bows out in front of the body (m) for each metre it goes across it. **The
/// demo's choice**: 0.5, so the second pistol of a swap, going from his left hip to his right hand
/// (0.5 m across), crosses 0.25 m in front of his coat, and a glide down the same side hardly bows.
const BOW: f32 = 0.5;
/// The model's forward direction, which the bow goes along (`yaw_of` in play.rs: a heading of
/// atan2(-x, -z), so forward is -z in the model's space).
const FORWARD: Vec3 = Vec3::NEG_Z;

/// The two holsters of a model (see the module notes).
#[derive(Clone, Debug)]
pub struct Holsters {
    /// the model's arms and thighs, whose mirror makes the left hip
    arm: MirrorArm,
    /// the right hip's stow hardpoint: the right thigh's bone and the point in its frame
    right: (usize, Hardpoint),
}

impl Holsters {
    /// The holsters of `model` when its type carries the Berserker (`skill_set` 3) and the model has
    /// none of the three stow hardpoints (`weapon::STOW`) or the SECOND_HAND of Tex's: its `arm`,
    /// the mirror of its right side, supplies the left hip. None for every other model and type.
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

    /// The places a gun hangs at: its HOLSTER hardpoint `holster` on the right hip's hardpoint for
    /// the place 0, the mirror of that on the left hip for the place 1 (`MirrorArm::left_hip`, with
    /// the gun's `barrel`, its firing direction, and `offset`, the muzzle's place from the grip's,
    /// both in its own axes), none for the place 2: the entries of `HeldWeapon::stow`.
    pub fn mounts(&self, holster: &Hardpoint, barrel: Vec3, offset: Vec3) -> [Option<Mount>; 3] {
        let (bone, hip) = self.right;
        let (r, t) = hip.mount(holster);
        [Some((bone, r, t)), self.arm.left_hip((bone, r, t), barrel, offset), None]
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
    /// Weapon `i`'s gun `entity` is to hang at `to` now (None: not shown), `world` the model's bones'
    /// matrices this frame (empty before the model is posed). A gun that moves from one place to
    /// another, both shown, starts to glide there, from where it was drawn if it was gliding
    /// already; one new, hidden, or put at the place it's at is placed at once.
    pub fn put(&mut self, i: usize, entity: Entity, to: Option<Mount>, world: &[Mat4]) -> Put {
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
        let glide = Glide { from, to, time: 0.0 };
        track.glide = Some(glide);
        Put::Glide(local(&glide, 0.0, world))
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
        Some(format!("glides from {from} to bone {:08x} over {GLIDE_TIME} s", bones[glide.to.0]))
    }
}

/// A glide's transform at `s` (0 - 1) under the bone it goes to: the model-space places it goes from
/// and to (`Start`, `Glide::to` on the bones in `world`), the position eased in a line and bowed out
/// in front of the body by `BOW` for each metre it goes across it, the rotation slerped, then put
/// into the destination bone's frame.
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
    let at = from_at.lerp(to_at, ease) + FORWARD * (BOW * across * (std::f32::consts::PI * ease).sin());
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
        assert_eq!(g.put(0, e, Some(at(0, Vec3::ZERO)), &world()), Put::Now);
        assert_eq!(g.put(0, e, Some(at(0, Vec3::ZERO)), &world()), Put::Now);
        let Put::Glide(first) = g.put(0, e, Some(at(1, Vec3::ZERO)), &world()) else { panic!("no glide") };
        // (it starts where it was: the first bone's origin, which the second bone's frame has at -1 along x)
        assert!((first.translation - Vec3::NEG_X).length() < 1e-5, "{first:?}");
        assert!(g.gliding(0));
        // (the same place again leaves the glide alone)
        assert_eq!(g.put(0, e, Some(at(1, Vec3::ZERO)), &world()), Put::Keep);
    }

    /// A gun hidden, or one before the model is posed, or another gun's entity, doesn't glide.
    #[test]
    fn a_gun_hidden_or_new_is_placed_at_once() {
        let mut g = Glides::default();
        let e = Entity::from_raw(1);
        g.put(0, e, Some(at(0, Vec3::ZERO)), &world());
        assert_eq!(g.put(0, e, None, &world()), Put::Now);
        assert_eq!(g.put(0, e, Some(at(1, Vec3::ZERO)), &world()), Put::Now);
        assert_eq!(g.put(0, e, Some(at(0, Vec3::ZERO)), &[]), Put::Now);
        assert_eq!(g.put(0, Entity::from_raw(2), Some(at(1, Vec3::ZERO)), &world()), Put::Now);
        assert!(!g.gliding(0));
    }

    /// The glide lasts `GLIDE_TIME`: it is on the way before, and on the frame it arrives its
    /// transform is the place itself and it stops.
    #[test]
    fn a_glide_arrives_at_its_time() {
        let mut g = Glides::default();
        let e = Entity::from_raw(3);
        g.put(0, e, Some(at(0, Vec3::ZERO)), &world());
        g.put(0, e, Some(at(1, Vec3::new(0.0, 0.5, 0.0))), &world());
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
        g.put(0, e, Some(at(0, Vec3::ZERO)), &world());
        g.put(0, e, Some(at(1, Vec3::ZERO)), &world());
        let on_the_way = g.step(0, GLIDE_TIME * 0.5, &world()).unwrap();
        let Put::Glide(restart) = g.put(0, e, Some(at(0, Vec3::new(0.0, 0.3, 0.0))), &world()) else { panic!("no glide") };
        // (under the first bone now: the model-space place it was at, the same under either)
        let was = world()[1] * on_the_way.compute_matrix();
        assert!((was.transform_point3(Vec3::ZERO) - restart.translation).length() < 1e-4, "{was:?} {restart:?}");
    }

    /// A glide across the body bows out in front of it, one down its own side hardly does.
    #[test]
    fn a_glide_across_the_body_bows_out_in_front() {
        let across = Glide { from: Start::Joint(at(0, Vec3::new(-0.5, 0.0, 0.0))), to: at(0, Vec3::new(0.5, 0.0, 0.0)), time: 0.0 };
        let mid = local(&across, 0.5, &world());
        assert!((mid.translation - Vec3::new(0.0, 0.0, -0.5)).length() < 1e-5, "{mid:?}");
        let along = Glide { from: Start::Joint(at(0, Vec3::new(0.0, 0.0, 0.0))), to: at(0, Vec3::new(0.0, -0.4, 0.0)), time: 0.0 };
        let mid = local(&along, 0.5, &world());
        assert!((mid.translation - Vec3::new(0.0, -0.2, 0.0)).length() < 1e-5, "{mid:?}");
    }
}
