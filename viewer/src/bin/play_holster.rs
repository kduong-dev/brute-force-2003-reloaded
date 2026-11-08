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
//!  - #409 made it a stow hardpoint on `Bip01 L Thigh` by mirroring Flint's (`Holsters::new`,
//!    `MirrorArm::left_hip`: the same mirror as the left hand's mount, with the thighs' own
//!    reflection). Flint is the squad member who wears a pistol on his hip: his place 1,
//!    h_f4ccf78c on his right thigh, objects-common.xml h_160a1622, where his LZR-10 hangs barrel
//!    down. Its point is scaled by the ratio of the two thighs' lengths (Kingman's 0.511 m to
//!    Flint's 0.457) and moved back by `HIP_SHIFT` (a fit: clear of where his empty left hand rests).
//!    The pistols' HOLSTER hardpoints h_10ab40ac (the four in objects-dm.xml and objects-m03_c.xml,
//!    the Kman Auto's among them) are all turned the same way (ORIENTATION 0 0 1 0; the rifles'
//!    another), so the Kman Auto hangs from it barrel down as Flint's LZR-10 does.
//!  - **#410 moves it up to the front of his waist** and onto the bone the thighs hang from
//!    (`Bip01 Pelvis`, where a belt rides): #409's place as his idle stands, moved by `WAIST_LIFT`
//!    and turned about the gun by `WAIST_TURN` in the model's space there, then kept under the pelvis.
//!    His hands now draw and holster (play_reach.rs), and his right hand can't reach the thigh's
//!    side across his body (see `WAIST_LIFT`): the front of his waist on the left is where a cross
//!    draw's holster sits, and his left hand reaches it too.
//!  - The berserk draw takes the second gun from it into his left hand, and the stow puts it back,
//!    the left hand going to it (play_reach.rs).
//!
//! The demo gives it to the type that carries the Berserker (skill-set 3) on a model without the
//! stow hardpoints (`Holsters::new`), so Tex's own holsters (his three hardpoints, the ramps) and
//! every other model's are as they were.
//!
//! **The swap**: Kingman's motion set has no weapon switch clip (Flint's and Hawk's
//! `Sc_w1_2_w2` is the pistols' own, ms_kingman has none), so the switch had been instant, the gun
//! in hand gone and the other there. The demo gives him a switch of its own, with the squad's
//! machinery (`play.rs` `start_switch`, `Switching`) but no clip to play over his body. Since #410 his
//! right hand does it (play_reach.rs): it carries the gun in it to the holster, lets it go there at
//! `SWAP_DROP`, takes the other there at `SWAP_GRAB` and brings it up by `SWAP_TIME`, the aim and the
//! shots held back until then. (#409's guns glided between the hand and the hip on their own, on
//! opposite arcs, which went through his left thigh for about 30 ms and hung the incoming gun 0.4 m
//! behind his back; a gun now only ever moves with the hand that holds it.)

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;
use bf_viewer::bf::weapon::{self, Hardpoint};

use super::mirror_arm::MirrorArm;
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
/// How far (m, in the right thigh's frame that the mirror turns into the left's: y out from the
/// body, z forward) #409's holster was moved from Flint's scaled point: 0.10 m back, to the rear of
/// the thigh. **A fit** to the frames of the demo (#409), not the data's: Flint's holster sits where a
/// hand hangs (Kingman's rests 0.18 m out from the hip joint and 0.08 m forward, the scaled holster
/// 0.14 m out and 0.1 m forward), so his empty left hand would have hung inside the gun. #410 takes
/// the holster from this place up to his waist (`WAIST_LIFT`, `WAIST_TURN`).
const HIP_SHIFT: Vec2 = Vec2::new(0.0, -0.10);
/// How far the holster is moved from #409's place on his left thigh, in the model's space as his idle
/// clip's first frame has it (m: +x his right, +y up, -z forward), after `WAIST_TURN`: 0.05 to his
/// right, 0.22 up and 0.38 forward, to the front of his waist on the left, the gun hanging against
/// his coat. **The demo's choice** (#410): his right hand can't reach the thigh: the grip there is
/// 0.84 m from his right shoulder, his arm 0.69 m long (upper arm 0.343, forearm 0.346), and the
/// thigh's side is past his coat for a forearm coming across (BF_REACH_LOG: 5 cm short with the shrug
/// and the turn into the draw, and the path through the front of the thigh; at the side of his waist
/// the forearm went through his coat); at his waist's front, where a cross-draw holster sits, the
/// right hand reaches it with the elbow bent, round the outside of the coat, and the left hand
/// comes forward to it. (His coat's front is 0.27 - 0.29 m out from his middle at the waist: a coarse
/// outline from the skinned idle's vertices.)
const WAIST_LIFT: Vec3 = Vec3::new(0.05, 0.22, -0.38);
/// The turn of the holster about the gun, about the up axis in the same space (degrees), the barrel
/// kept down as #409 hung it. **A fit** (#410): the right hand comes to the holster across his front
/// and the left from his side, so no one turn of one gun at one place suits both wrists as his idle
/// holds them (BF_REACH_LOG: the two hands' guns, the wrists as the idle has them, 133 degrees apart);
/// each hand's wrist turns half the way from its own to the gun's grip and the gun turns in the hand
/// for the rest (play_reach.rs `WRIST_SHARE`), and this is the turn that asks the least of both: about
/// 67 degrees each in all, 34 - 36 of them the gun's turn in the hand, the wrists at most 70 degrees
/// from straight (his idle carries the gun at 47 - 49). (Held exactly in the mount with no turn in the
/// hand, every turn bent one wrist or the other 108 - 165 degrees from straight, and a turn of 200, the
/// cross draw's butt forward, flipped the hand 96 degrees in a frame once the gun turned in it.)
const WAIST_TURN: f32 = 20.0;

/// The swap of Kingman's two pistols (the demo's own switch, see the module notes), done by his
/// right hand (#410, play_reach.rs): the time the hand has carried the gun in it to the left hip and
/// lets it go (`LEG_TIME`), the time it takes the other there (`DWELL` on), and the swap's length,
/// the hand back up with it (`LEG_TIME` more): 0.25, 0.30 and 0.55 s. **The demo's choice**: one hand
/// does the holster and then the draw, where #409's two guns glided at once (0.0, 0.1, 0.35 s).
pub const SWAP_DROP: f32 = reach::LEG_TIME;
pub const SWAP_GRAB: f32 = SWAP_DROP + reach::DWELL;
pub const SWAP_TIME: f32 = SWAP_GRAB + reach::LEG_TIME;

/// The holster of a model (see the module notes).
#[derive(Clone, Debug)]
pub struct Holsters {
    /// the model's arms and thighs, whose mirror makes the left hip
    arm: MirrorArm,
    /// the stow hardpoint as Flint's right thigh has it: the right thigh's bone and the point in
    /// its frame, which `MirrorArm::left_hip` turns over to the left thigh
    right: (usize, Hardpoint),
    /// the bone the holster rides on, the one the thighs hang from (`Bip01 Pelvis`), with the inverse
    /// of its matrix and the left thigh's in his idle: a place on the left thigh is taken to the model's
    /// space there, moved to the waist (`WAIST_LIFT`, `WAIST_TURN`) and put under the pelvis (#410)
    waist: (usize, Mat4, Mat4),
    /// the move to the waist in the model's space at his idle: the lift, and the turn about the gun
    lift: Vec3,
    turn: Quat,
}

impl Holsters {
    /// The holster of `model` when its type carries the Berserker (`skill_set` 3) and the model has
    /// none of the three stow hardpoints (`weapon::STOW`) or the SECOND_HAND of Tex's: its `arm`,
    /// the mirror of its right side, makes the left hip's, and `rest`, the bones' matrices in its
    /// idle (play.rs `spawn_unit`: its idle clip's first frame, made only for such a type), takes that
    /// up to the waist on the pelvis. None for every other model and type.
    pub fn new(model: &Character, arm: &MirrorArm, skill_set: Option<i64>, rest: impl FnOnce() -> Vec<Mat4>) -> Option<Self> {
        if skill_set != Some(BERSERKER_SET) || weapon::STOW.iter().chain([&weapon::SECOND_HAND]).any(|k| model.hardpoints.contains_key(k)) {
            return None;
        }
        let rest = rest();
        let thigh = arm.right_thigh()?;
        let calf = model.bones.iter().position(|b| *b == h("Bip01 R Calf")).filter(|&c| model.parent[c] == Some(thigh))?;
        // (the thigh's length: where the bind pose puts its calf, in its frame)
        let scale = model.bind_local()[calf].1.length() / FLINT_THIGH;
        let [w, x, y, z] = FLINT_HIP_ROT;
        let point = FLINT_HIP_POINT * scale + Vec3::new(0.0, HIP_SHIFT.x, HIP_SHIFT.y);
        let left = arm.left_thigh()?;
        let pelvis = model.parent[left].filter(|&p| Some(p) == model.parent[thigh] && rest.len() == model.bones.len())?;
        Some(Holsters { arm: arm.clone(), right: (thigh, Hardpoint { point, rot: Quat::from_xyzw(x, y, z, w).normalize() }),
                        waist: (pelvis, rest[pelvis].inverse(), rest[left]), lift: WAIST_LIFT, turn: Quat::from_rotation_y(WAIST_TURN.to_radians()) })
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
        let (pelvis, pelvis_inverse, thigh) = self.waist;
        let left = self.arm.left_hip((bone, r, t), barrel, offset).map(|(_, r, t)| {
            // (the place on the thigh in the model's space at his idle, turned about the gun and lifted,
            // under the pelvis)
            let at = thigh * Mat4::from_rotation_translation(r, t);
            let gun = at.w_axis.truncate();
            let moved = Mat4::from_translation(self.lift + gun) * Mat4::from_quat(self.turn) * Mat4::from_translation(-gun) * at;
            let (_, r, t) = (pelvis_inverse * moved).to_scale_rotation_translation();
            (pelvis, r.normalize(), t)
        });
        [left, left, None]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The holster is the left hip for both of the places the guns' slots start on and nothing for
    /// the third: a pistol that isn't in his hand hangs on the left, whichever it is; it rides on the
    /// waist's bone, the left thigh's place carried up there by `Holsters::waist` (#410).
    #[test]
    fn both_guns_hang_on_the_left_hip() {
        let k = Mat3::from_cols(-Vec3::X, Vec3::Y, Vec3::Z);
        let arm = MirrorArm::symmetric(k);
        let hips = Holsters { arm: arm.clone(), right: (9, Hardpoint { point: Vec3::new(-0.16, 0.12, -0.01), rot: Quat::from_rotation_x(0.5) }),
                              waist: (0, Mat4::IDENTITY, Mat4::IDENTITY), lift: Vec3::new(0.0, 0.25, -0.1), turn: Quat::IDENTITY };
        let holster = Hardpoint { point: Vec3::new(0.0, 0.1, -0.17), rot: Quat::from_rotation_y(std::f32::consts::PI) };
        let (barrel, offset) = (Vec3::Z, Vec3::new(0.0, 0.1, 0.4));
        let stow = hips.mounts(&holster, barrel, offset);
        let (bone, r, t) = stow[0].expect("the first gun's place");
        assert_eq!(bone, 0, "the waist's bone");
        assert_eq!(stow[1], stow[0], "the second gun's place is the same hip");
        assert!(stow[2].is_none());
        // (the left thigh's place, lifted: here the waist's matrix is the lift alone)
        let (hr, ht) = hips.right.1.mount(&holster);
        let (_, lr, lt) = arm.left_hip((9, hr, ht), barrel, offset).unwrap();
        assert!((t - (lt + Vec3::new(0.0, 0.25, -0.1))).length() < 1e-5 && r.angle_between(lr) < 1e-4, "{t:?} {lt:?}");
    }
}
