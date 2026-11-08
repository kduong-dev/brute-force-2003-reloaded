//! The mirrored left arm of a character that holds a second gun without a berserk set of its
//! own (#407, **demo-own**: the product owner's request for Edward Kingman's Berserker).
//!
//! Only Tex has the animation set for two guns (his berserk motions `Sc_w1_bz_*`, and the
//! hardpoint SECOND_HAND h_15ab6d29 on his left hand, weapon.rs). Edward Kingman's ms_kingman has
//! neither, so while the Berserker holds a second gun the demo poses his **left arm as the mirror
//! of his right**: each frame, after the aim has turned the right arm to the crosshair
//! (play.rs `step_player`), the left clavicle, upper arm, forearm and hand take the right ones'
//! orientations reflected across **the aim's vertical plane**, the plane through the point between
//! the shoulders that holds the right gun's barrel direction and the up axis. (The torso is not
//! square to the aim: standing, it stays about 16 degrees off it, and a mirror across the torso's
//! own plane doubled that: the left barrel splayed 25 - 30 degrees outward and the hand sat farther
//! from the body, round 3 of #407.) With the barrel in the plane its reflection is itself, so both
//! barrels are parallel to the aim, and the two hands are level and the same distance from the aim
//! line; only a shift along the aim of the shoulders' offset from the plane remains (about 0.15 m
//! at 16 degrees).
//!
//! The bones are the Biped's (`Bip01 R|L Clavicle`, `UpperArm`, `Forearm`, `Hand`, the names the
//! game's hit parts use, play_damage.rs `HIT_BONE_NAMES`). A bone's frame, reflected across a plane
//! with the matrix `M`, is `M W K`, where `K` is a constant reflection in the bone's own axes that
//! turns the right bone's frame into the left's at rest (`K = Rr0^T M0 Rl0`, with `M0` the rest
//! pose's own symmetry plane, between the shoulders in the bind pose, `Character::bind_local`): so
//! the left bone is `Wl = M Wr K`, and at rest `M = M0` gives the bind pose back. A gun held by the
//! right hand at `(r, t)` is held by the left at `(K^T r S, K^T t)`, with `S` the reflection of the
//! gun's own lateral axis (`lateral`), which keeps its barrel and its grip where they were: the
//! left gun has the right one's orientation and the mirror of its place, right way up. A model
//! missing any of the eight bones, or whose clavicles hang from different bones, has no `MirrorArm`.

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;

/// The bone names of an arm, top down, with the side letter to put in: `Bip01 {side} Clavicle` ...
const ARM: [&str; 4] = ["Clavicle", "UpperArm", "Forearm", "Hand"];
/// The least horizontal length of the barrel's direction (unit barrel) for it to give the aim's
/// plane; a barrel more vertical than that (about 78 degrees up or down) uses the torso's plane.
const MIN_HEADING: f32 = 0.2;

/// A model's two arms, to pose the left as the mirror of the right (see the module notes).
#[derive(Clone, Debug)]
pub struct MirrorArm {
    /// (right, left) bone indices, clavicle first, then the upper arm, forearm and hand
    pairs: [(usize, usize); 4],
    /// the bone both clavicles hang from
    parent: usize,
    /// the torso's mirror plane's unit normal in the parent's frame (left to right shoulder in the
    /// bind pose), the plane used when the aim is near the vertical
    normal: Vec3,
    /// for each pair, the constant reflection `K` in the bone's own axes (see the module notes)
    flip: [Mat3; 4],
}

/// The rotation of a transform.
fn rot(m: &Mat4) -> Quat {
    m.to_scale_rotation_translation().1
}

/// The matrix of the reflection across the plane through the origin with unit normal `n`.
fn reflection(n: Vec3) -> Mat3 {
    let f = |v: Vec3| v - 2.0 * n * n.dot(v);
    Mat3::from_cols(f(Vec3::X), f(Vec3::Y), f(Vec3::Z))
}

/// The unit axis of a gun's own frame that is its lateral one: of the two axes that aren't the
/// barrel's (`barrel`, the gun-local firing direction), the one the muzzle's offset from the grip
/// (`offset`, both gun-local hardpoints) has least of, as a pistol's barrel sits straight above its
/// grip, with nothing to its side.
fn lateral(barrel: Vec3, offset: Vec3) -> Vec3 {
    [Vec3::X, Vec3::Y, Vec3::Z].into_iter()
        .filter(|a| a.dot(barrel).abs() < 0.5)
        .min_by(|a, b| a.dot(offset).abs().total_cmp(&b.dot(offset).abs()))
        .unwrap_or(Vec3::X)
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
        let rel = |b: usize| (inverse * world[b]).w_axis.truncate();
        let normal = (rel(pairs[1].0) - rel(pairs[1].1)).try_normalize()?;
        let rest_plane = reflection(rot(&world[parent]) * normal);
        let flip = pairs.map(|(r, l)| {
            let (rr, rl) = (Mat3::from_quat(rot(&world[r])), Mat3::from_quat(rot(&world[l])));
            rr.transpose() * rest_plane * rl
        });
        Some(MirrorArm { pairs, parent, normal, flip })
    }

    /// Poses the left arm as the mirror of the right in `pose` (the model's local transforms, the
    /// right arm and the spine already posed), across the aim's vertical plane: `barrel` is the
    /// right gun's barrel direction in its hand bone's own axes (the mount's rotation times the
    /// gun's firing direction). Each left bone takes `M Wr K` (see the module notes).
    pub fn apply(&self, model: &Character, pose: &mut [(Quat, Vec3)], barrel: Vec3) {
        let world = model.world(pose);
        let aim = rot(&world[self.pairs[3].0]) * barrel;
        let heading = Vec3::new(aim.x, 0.0, aim.z);
        let normal = if heading.length() >= MIN_HEADING * aim.length() {
            Vec3::new(-heading.z, 0.0, heading.x).normalize()
        } else {
            rot(&world[self.parent]) * self.normal
        };
        let plane = reflection(normal);
        for (i, &(r, l)) in self.pairs.iter().enumerate() {
            let left = Quat::from_mat3(&(plane * Mat3::from_quat(rot(&world[r])) * self.flip[i])).normalize();
            // (the left bone's parent is the spine bone for the clavicle, else the left bone above,
            // already turned: its world is read again each time)
            let above = model.world(pose)[model.parent[l].unwrap_or(self.parent)];
            pose[l].0 = (rot(&above).inverse() * left).normalize();
        }
    }

    /// The left hand's mount for a gun held in the right hand by `mount` (the trigger hand's
    /// (bone, rotation, translation), `HeldWeapon::in_hand`), as (left hand bone, rotation,
    /// translation): the gun with the right one's orientation and the mirror of its place (see the
    /// module notes). `barrel` is the gun's firing direction and `offset` the muzzle's place from
    /// the grip's, both in the gun's own axes (`lateral`).
    pub fn mount(&self, (bone, r, t): (usize, Quat, Vec3), barrel: Vec3, offset: Vec3) -> Option<(usize, Quat, Vec3)> {
        if bone != self.pairs[3].0 {
            return None;
        }
        let side = lateral(barrel, offset);
        let flip_side = reflection(side);
        let k_inverse = self.flip[3].transpose();
        let rotation = Quat::from_mat3(&(k_inverse * Mat3::from_quat(r) * flip_side)).normalize();
        Some((self.pairs[3].1, rotation, k_inverse * t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reflection across the plane x = 0 turns the x axis over and keeps the other two; two of
    /// them are the identity.
    #[test]
    fn a_reflection_turns_the_normal_over() {
        let m = reflection(Vec3::X);
        assert!((m * Vec3::new(1.0, 2.0, 3.0) - Vec3::new(-1.0, 2.0, 3.0)).length() < 1e-6);
        assert!((m * m - Mat3::IDENTITY).to_cols_array().iter().all(|v| v.abs() < 1e-6));
        assert!((m.determinant() + 1.0).abs() < 1e-6);
    }

    /// A vector in the plane is its own reflection, so a barrel along the aim stays: the left gun
    /// points where the right one does.
    #[test]
    fn a_barrel_in_the_plane_is_its_own_reflection() {
        let aim = Vec3::new(0.3, 0.2, -1.0).normalize();
        let normal = Vec3::new(-aim.z, 0.0, aim.x).normalize();
        assert!((reflection(normal) * aim - aim).length() < 1e-6);
    }

    /// The gun's lateral axis is the one of the two off the barrel that its muzzle has no offset
    /// along: a barrel along -z with the muzzle above and ahead of the grip has x.
    #[test]
    fn the_lateral_axis_is_the_one_the_muzzle_has_no_offset_along() {
        assert_eq!(lateral(Vec3::NEG_Z, Vec3::new(0.0, 0.06, -0.2)), Vec3::X);
        assert_eq!(lateral(Vec3::NEG_Z, Vec3::new(0.03, 0.0, -0.2)), Vec3::Y);
        assert_eq!(lateral(Vec3::NEG_X, Vec3::new(-0.2, 0.0, 0.06)), Vec3::Y);
    }

    /// Reflecting the gun's lateral axis keeps its barrel and its up where they were: the mirrored
    /// gun is right way up (the first of round 3's faults).
    #[test]
    fn the_gun_keeps_its_barrel_and_its_up() {
        let s = reflection(Vec3::X);
        assert_eq!(s * Vec3::NEG_Z, Vec3::NEG_Z);
        assert_eq!(s * Vec3::Y, Vec3::Y);
    }
}
