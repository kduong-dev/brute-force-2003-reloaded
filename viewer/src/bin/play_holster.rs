//! Edward Kingman's two pistols, one on each hip, and the swap between them (#409, #410,
//! **demo-own**: the product owner's design, after #407 gave the demo's Kingman class a second Kman
//! Auto).
//!
//! Kingman's compound h_e0b89937 has no stow hardpoint at all (objects-m03_c.xml ~9868: its 17
//! hardpoints are the hands' TRIGGER_HAND and SUPPORT_HAND, the head's and the bones' effect
//! points), so a gun that isn't in his hand was hidden. The demo gives the class **a holster on the
//! side of each hip**, as a real gun holster sits, and **each gun belongs to a side and its hand**
//! (the product owner's design, #410): the first slot's on his right hip and in his right hand, the
//! second slot's on his left hip and in his left hand (play.rs `HeldWeapon::left`). The game's two
//! stow places the guns' slots start on (`StowPlaces`: place 0 the first gun's, 1 the second's) are
//! the two holsters (`Holsters::mounts`):
//!
//!  - The right one is Flint's stow hardpoint, the squad member who wears a pistol on his hip: his
//!    place 1, h_f4ccf78c on his right thigh, objects-common.xml h_160a1622, where his LZR-10 hangs
//!    barrel down. The left one is its mirror (`MirrorArm::left_hip`: the same mirror as the left
//!    hand's mount, with the thighs' own reflection). Its point is scaled by the ratio of the two
//!    thighs' lengths (Kingman's 0.511 m to Flint's 0.457) and moved back by `HIP_SHIFT` (#409's fit:
//!    clear of where the hand on that side hangs).
//!  - #410 moves each up the side of his hip by `HIP_LIFT` (the right one's mirrored) and onto the
//!    bone the thighs hang from (`Bip01 Pelvis`, where a belt rides), so the hand on that side
//!    reaches it with the elbow bent (play_reach.rs).
//!  - The pistols' HOLSTER hardpoints h_10ab40ac (the four in objects-dm.xml and objects-m03_c.xml,
//!    the Kman Auto's among them) are all turned the same way (ORIENTATION 0 0 1 0; the rifles'
//!    another), so the Kman Auto hangs from it barrel down as Flint's LZR-10 does.
//!
//! (#409 hung both guns on the left hip, the one not in his right hand there; round 1 of #410 moved
//! that holster to the front of his waist for a cross draw. The product owner asked for one on each
//! side instead, each hand reaching only its own.)
//!
//! The demo gives them to the type that carries the Berserker (skill-set 3) on a model without the
//! stow hardpoints (`Holsters::new`), so Tex's own holsters (his three hardpoints, the ramps) and
//! every other model's are as they were.
//!
//! **The swap**: Kingman's motion set has no weapon switch clip (Flint's and Hawk's `Sc_w1_2_w2` is
//! the pistols' own, ms_kingman has none), so the switch had been instant, the gun in hand gone and
//! the other there. The demo gives him a switch of its own, with the squad's machinery (`play.rs`
//! `start_switch`, `Switching`) but no clip to play over his body, and his hands do it **side to
//! side** (play_reach.rs): the hand in use takes its gun to its own hip and lets it go there
//! (`SWAP_DROP` to `SWAP_GRAB`), then the other hand goes to its own hip, takes the other gun and
//! brings it up (by `SWAP_TIME`), the aim and the shots held back until then; his arms exchange their
//! poses as the hands pass (play.rs `Player::hand`), so the gun in his left hand is carried and aimed
//! as the clips carry the right one's. A gun only ever moves with the hand that holds it.

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;
use bf_viewer::bf::weapon::{self, Hardpoint};

use super::mirror_arm::{mirror_on_bone, MirrorArm};
use super::reach;

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
/// How far (m, in the right thigh's frame: y out from the body, z forward) #409's holster was moved
/// from Flint's scaled point: 0.10 m back, to the rear of the thigh. **A fit** to the frames of the
/// demo (#409), not the data's: Flint's holster sits where a hand hangs (Kingman's rests 0.18 m out
/// from the hip joint and 0.08 m forward, the scaled holster 0.14 m out and 0.1 m forward), so the
/// empty hand on that side would hang inside the gun. #410 takes the holster from this place up the
/// side of his hip (`HIP_LIFT`).
const HIP_SHIFT: Vec2 = Vec2::new(0.0, -0.10);
/// How far the right holster is moved from #409's place on the right thigh, in the model's space as
/// his idle clip's first frame has it (m: +x out to his right, +y up, +z back): up the side of his
/// hip toward his waist, so the hand on that side reaches it with the elbow bent; the left holster
/// is its mirror. **The demo's choice** (#410, the product owner's: "on the sides of his hips, like
/// real gun holsters ... fit to his waist or upper thigh so each hand reaches its own side
/// comfortably"): on the thigh the wrist had to reach 0.68 m from the shoulder, the arm near straight
/// (BF_REACH_LOG); lifted 0.12 m the grip is 0.10 m above his hip joints and the arm is 0.80 of its
/// length straight there.
const HIP_LIFT: Vec3 = Vec3::new(0.0, 0.12, 0.0);

/// The swap of Kingman's two pistols (the demo's own switch, see the module notes), done side to
/// side by his hands (#410, play_reach.rs): the gun in hand is let go at once and its hand carries it
/// to its own hip (`LEG_TIME`); the other gun is taken then, and the other hand goes to its own hip
/// for it (`LEG_TIME`), takes it and brings it up (`LEG_TIME`), the switch over at 0.75 s with the gun
/// up. **The demo's choice**: one hand holsters and then the other draws, where #409's two guns glided
/// at once (0.0, 0.1, 0.35 s).
pub const SWAP_DROP: f32 = 0.0;
pub const SWAP_GRAB: f32 = reach::LEG_TIME;
pub const SWAP_TIME: f32 = 3.0 * reach::LEG_TIME;

/// The holsters of a model (see the module notes).
#[derive(Clone, Debug)]
pub struct Holsters {
    /// the stow hardpoint as Flint's right thigh has it, in that thigh's frame
    right: Hardpoint,
    /// the bone the holsters ride on, the one the thighs hang from (`Bip01 Pelvis`), with the inverse
    /// of its matrix and the right thigh's in his idle: the place on the thigh is taken to the model's
    /// space there, moved up the hip (`HIP_LIFT`) and put under the pelvis (#410)
    waist: (usize, Mat4, Mat4),
    /// the pelvis's own middle plane in its frame, as the bind pose has it: the unit normal from the
    /// left thigh's joint to the right one's, and the point between them. The left holster is the
    /// right one's mirror across it (`mirror_on_bone`), so the two are mirror images on his body
    /// whatever his idle's legs do
    middle: (Vec3, Vec3),
}

impl Holsters {
    /// The holsters of `model` when its type carries the Berserker (`skill_set` 3) and the model has
    /// none of the three stow hardpoints (`weapon::STOW`) or the SECOND_HAND of Tex's: `arm`'s thighs
    /// carry them, and `rest`, the bones' matrices in its idle (play.rs `spawn_unit`: its idle clip's
    /// first frame, made only for such a type), takes the right one up its hip on the pelvis. None for
    /// every other model and type.
    pub fn new(model: &Character, arm: &MirrorArm, skill_set: Option<i64>, rest: impl FnOnce() -> Vec<Mat4>) -> Option<Self> {
        if skill_set != Some(BERSERKER_SET) || weapon::STOW.iter().chain([&weapon::SECOND_HAND]).any(|k| model.hardpoints.contains_key(k)) {
            return None;
        }
        let rest = rest();
        let (thigh, left) = arm.thighs()?;
        let calf = model.bones.iter().position(|b| *b == h("Bip01 R Calf")).filter(|&c| model.parent[c] == Some(thigh))?;
        // (the thigh's length: where the bind pose puts its calf, in its frame)
        let scale = model.bind_local()[calf].1.length() / FLINT_THIGH;
        let [w, x, y, z] = FLINT_HIP_ROT;
        let point = FLINT_HIP_POINT * scale + Vec3::new(0.0, HIP_SHIFT.x, HIP_SHIFT.y);
        let pelvis = model.parent[left].filter(|&p| Some(p) == model.parent[thigh] && rest.len() == model.bones.len())?;
        // (the pelvis's middle: between its thigh joints in the bind pose, in its frame)
        let bind = model.world(&model.bind_local());
        let inverse = bind[pelvis].inverse();
        let joint = |b: usize| inverse.transform_point3(bind[b].w_axis.truncate());
        let normal = (joint(thigh) - joint(left)).try_normalize()?;
        Some(Holsters { right: Hardpoint { point, rot: Quat::from_xyzw(x, y, z, w).normalize() },
                        waist: (pelvis, rest[pelvis].inverse(), rest[thigh]), middle: (normal, (joint(thigh) + joint(left)) * 0.5) })
    }

    /// The places a gun hangs at, the entries of `HeldWeapon::stow` (#410): place 0, the first gun's,
    /// its HOLSTER hardpoint `holster` on **the right hip's** hardpoint, Flint's, moved up the hip
    /// (`HIP_LIFT`); place 1, the second gun's, on **the left hip's**, its mirror across the pelvis's
    /// middle (`mirror_on_bone`, with the gun's `barrel`, its firing direction, and `offset`, the
    /// muzzle's place from the grip's, both in its own axes); both under the pelvis; none for the place
    /// 2, which Tex alone has. Each gun belongs to the side its slot's place is on, and its hand
    /// (play.rs `HeldWeapon::left`).
    pub fn mounts(&self, holster: &Hardpoint, barrel: Vec3, offset: Vec3) -> [Option<Mount>; 3] {
        let (pelvis, pelvis_inverse, thigh) = self.waist;
        let (r, t) = self.right.mount(holster);
        let up = Mat4::from_translation(HIP_LIFT) * thigh * Mat4::from_rotation_translation(r, t);
        let (_, r, t) = (pelvis_inverse * up).to_scale_rotation_translation();
        let (normal, point) = self.middle;
        let (lr, lt) = mirror_on_bone(normal, point, (r.normalize(), t), barrel, offset);
        [Some((pelvis, r.normalize(), t)), Some((pelvis, lr, lt)), None]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first gun's place is the right hip's holster, lifted, and the second's the left's, its
    /// mirror across the pelvis's middle, both riding on the waist's bone; nothing for the third
    /// place (#410).
    #[test]
    fn each_gun_hangs_on_its_own_hip() {
        let hips = Holsters { right: Hardpoint { point: Vec3::new(-0.16, 0.12, -0.01), rot: Quat::from_rotation_x(0.5) },
                              waist: (3, Mat4::IDENTITY, Mat4::IDENTITY), middle: (Vec3::X, Vec3::ZERO) };
        let holster = Hardpoint { point: Vec3::new(0.0, 0.1, -0.17), rot: Quat::from_rotation_y(std::f32::consts::PI) };
        let (barrel, offset) = (Vec3::Z, Vec3::new(0.0, 0.1, 0.4));
        let stow = hips.mounts(&holster, barrel, offset);
        let ((rb, rr, rt), (lb, lr, lt)) = (stow[0].expect("the first gun's place"), stow[1].expect("the second gun's"));
        assert_eq!((rb, lb), (3, 3), "the waist's bone");
        assert!(stow[2].is_none());
        // (here the thigh's and the waist's matrices are the identity: the right place is the thigh's
        // own, lifted, and the left its mirror across x = 0)
        let (hr, ht) = hips.right.mount(&holster);
        assert!((rt - (ht + HIP_LIFT)).length() < 1e-5 && rr.angle_between(hr) < 1e-4, "{rt:?}");
        assert!((lt - Vec3::new(-rt.x, rt.y, rt.z)).length() < 1e-5, "{lt:?}");
        let (mr, _) = mirror_on_bone(Vec3::X, Vec3::ZERO, (rr, rt), barrel, offset);
        assert!(lr.angle_between(mr) < 1e-5);
    }
}
