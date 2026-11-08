//! The mirrored left arm of a character that holds a second gun without a berserk set of its
//! own (#407, **demo-own**: the product owner's request for Edward Kingman's Berserker).
//!
//! Only Tex has the animation set for two guns (his berserk motions `Sc_w1_bz_*`, and the
//! hardpoint SECOND_HAND h_15ab6d29 on his left hand, weapon.rs). Edward Kingman's ms_kingman has
//! neither, so while the Berserker holds a second gun the demo poses his **left arm as the mirror
//! of his right**: each frame, after the aim has turned the right arm to the crosshair
//! (play.rs `step_player`), the left clavicle, upper arm, forearm and hand take the right ones'
//! rotations reflected across the torso's midplane. The left gun, held at the mirror of the trigger
//! hand's mount (`MirrorArm::mount`), then follows the aim and the recoil as the right one does.
//!
//! The bones are the Biped's (`Bip01 R|L Clavicle`, `UpperArm`, `Forearm`, `Hand`, the names the
//! game's hit parts use, play_damage.rs `HIT_BONE_NAMES`). The reflection is done in the frame of
//! the bone both clavicles hang from (the spine bone that carries them, which the aim turns), so a
//! twist of the torso doesn't make the left arm twist the other way. Its plane's normal is the
//! direction between the two shoulders at rest (the bind pose, `Character::bind_local`), its
//! centre their midpoint. A rotation `delta` made on the right turns the left by `delta`
//! reflected: the same axis mirrored and the angle reversed (`mirrored`). A model missing any of
//! the eight bones, or whose clavicles hang from different bones, has no `MirrorArm`.

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;

/// The bone names of an arm, top down, with the side letter to put in: `Bip01 {side} Clavicle` ...
const ARM: [&str; 4] = ["Clavicle", "UpperArm", "Forearm", "Hand"];

/// A model's two arms, to pose the left as the mirror of the right (see the module notes).
#[derive(Clone, Debug)]
pub struct MirrorArm {
    /// (right, left) bone indices, clavicle first, then the upper arm, forearm and hand
    pairs: [(usize, usize); 4],
    /// the bone both clavicles hang from
    parent: usize,
    /// the mirror plane's unit normal in the parent's rest frame (left to right shoulder) and a
    /// point of it (the shoulders' midpoint)
    normal: Vec3,
    mid: Vec3,
    /// every bone's rest (bind) transform relative to the parent's rest frame, for the pairs'
    /// bones, same order as `pairs` (right, left)
    rest: [(Mat4, Mat4); 4],
}

/// The rotation of a transform.
fn rot(m: &Mat4) -> Quat {
    m.to_scale_rotation_translation().1
}

impl MirrorArm {
    /// The arms of `model`, from its bone names and bind pose; None when it lacks a bone or the
    /// clavicles hang from different ones.
    pub fn new(model: &Character) -> Option<Self> {
        let find = |side: &str, name: &str| model.bones.iter().position(|b| *b == h(&format!("Bip01 {side} {name}")));
        let mut pairs = [(0, 0); 4];
        for (i, name) in ARM.iter().enumerate() {
            pairs[i] = (find("R", name)?, find("L", name)?);
        }
        let parent = model.parent[pairs[0].0].filter(|&p| Some(p) == model.parent[pairs[0].1])?;
        let world = model.world(&model.bind_local());
        let inverse = world[parent].inverse();
        let rel = |b: usize| inverse * world[b];
        let (r, l) = (rel(pairs[1].0).w_axis.truncate(), rel(pairs[1].1).w_axis.truncate());
        let normal = (r - l).try_normalize()?;
        let rest = pairs.map(|(r, l)| (rel(r), rel(l)));
        Some(MirrorArm { pairs, parent, normal, mid: (r + l) * 0.5, rest })
    }

    /// `v` reflected across the plane's direction (a vector: no centre).
    fn reflect(&self, v: Vec3) -> Vec3 {
        v - 2.0 * self.normal * self.normal.dot(v)
    }

    /// A rotation as the mirror image of it: the axis reflected and the angle reversed.
    fn mirrored(&self, q: Quat) -> Quat {
        let v = -self.reflect(Vec3::new(q.x, q.y, q.z));
        Quat::from_xyzw(v.x, v.y, v.z, q.w)
    }

    /// Poses the left arm as the mirror of the right in `pose` (the model's local transforms, the
    /// right arm and the spine already posed): for each pair, the right bone's rotation away from
    /// its rest, taken in the parent's frame, is mirrored and put on the left bone's own rest.
    pub fn apply(&self, model: &Character, pose: &mut [(Quat, Vec3)]) {
        let world = model.world(pose);
        let parent_rot = rot(&world[self.parent]);
        for (i, &(r, l)) in self.pairs.iter().enumerate() {
            let right = parent_rot.inverse() * rot(&world[r]);
            let delta = right * rot(&self.rest[i].0).inverse();
            let left = parent_rot * (self.mirrored(delta) * rot(&self.rest[i].1));
            // (the left bone's parent is the spine bone for the clavicle, else the left bone above,
            // already turned: its world is read again each time)
            let above = model.world(pose)[model.parent[l].unwrap_or(self.parent)];
            pose[l].0 = (rot(&above).inverse() * left).normalize();
        }
    }

    /// The left hand's mount for a gun held in the right hand by `mount` (the trigger hand's
    /// (bone, rotation, translation), `HeldWeapon::in_hand`): the same gun at the mirror of the
    /// right hand's place at rest, turned as a right-handed gun is (a reflection of the grip with
    /// the pistol's own side kept), as (left hand bone, rotation, translation).
    pub fn mount(&self, (bone, r, t): (usize, Quat, Vec3)) -> Option<(usize, Quat, Vec3)> {
        if bone != self.pairs[3].0 {
            return None;
        }
        let gun = self.rest[3].0 * Mat4::from_rotation_translation(r, t);
        let (gun_rot, gun_pos) = (rot(&gun), gun.w_axis.truncate());
        let m = Mat3::from_cols(self.reflect(Vec3::X), self.reflect(Vec3::Y), self.reflect(Vec3::Z));
        let mirrored_rot = Quat::from_mat3(&(m * Mat3::from_quat(gun_rot) * m)).normalize();
        let mirrored_pos = self.mid + self.reflect(gun_pos - self.mid);
        let (_, hand_rot, hand_pos) = self.rest[3].1.to_scale_rotation_translation();
        Some((self.pairs[3].1, hand_rot.inverse() * mirrored_rot, hand_rot.inverse() * (mirrored_pos - hand_pos)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mirror whose plane is x = 0.5 (normal +x), with identity rests, for the rotation tests.
    fn plane() -> MirrorArm {
        MirrorArm { pairs: [(0, 1); 4], parent: 0, normal: Vec3::X, mid: Vec3::new(0.5, 0.0, 0.0), rest: [(Mat4::IDENTITY, Mat4::IDENTITY); 4] }
    }

    /// A turn about the vertical, mirrored, is the opposite turn; a turn about the forward axis
    /// too; a turn about the axis across the plane (a nod) is the same turn.
    #[test]
    fn a_mirrored_turn_reverses_the_angle_about_the_axes_in_the_plane() {
        let m = plane();
        let yaw = Quat::from_rotation_y(0.4);
        assert!(m.mirrored(yaw).angle_between(Quat::from_rotation_y(-0.4)) < 1e-5);
        let roll = Quat::from_rotation_z(0.3);
        assert!(m.mirrored(roll).angle_between(Quat::from_rotation_z(-0.3)) < 1e-5);
        let nod = Quat::from_rotation_x(0.5);
        assert!(m.mirrored(nod).angle_between(nod) < 1e-5);
        // (a mirrored mirror is the turn itself)
        let q = Quat::from_euler(EulerRot::XYZ, 0.3, -0.7, 0.2);
        assert!(m.mirrored(m.mirrored(q)).angle_between(q) < 1e-5);
    }

    /// A vector reflects across the plane's direction, a point across the plane itself.
    #[test]
    fn a_vector_reflects_across_the_plane() {
        let m = plane();
        assert!((m.reflect(Vec3::new(1.0, 2.0, 3.0)) - Vec3::new(-1.0, 2.0, 3.0)).length() < 1e-6);
        assert!((m.mid + m.reflect(Vec3::new(2.0, 0.0, 0.0) - m.mid) - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-6);
    }
}
