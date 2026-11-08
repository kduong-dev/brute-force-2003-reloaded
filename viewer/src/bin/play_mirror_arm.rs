//! The mirrored left arm of a character that holds a second gun without a berserk set of its
//! own (#407, **demo-own**: the product owner's request for Edward Kingman's Berserker), and the
//! mirrored left hip the second gun is holstered on (#409, play_holster.rs).
//!
//! Only Tex has the animation set for two guns (his berserk motions `Sc_w1_bz_*`, and the
//! hardpoint SECOND_HAND h_15ab6d29 on his left hand, weapon.rs). Edward Kingman's ms_kingman has
//! neither, so while the Berserker holds a second gun the demo poses his **left arm as the mirror
//! of his right**: each frame, after the aim has turned the right arm to the crosshair
//! (play.rs `step_player`), the left clavicle, upper arm, forearm and hand take the right ones'
//! orientations reflected across a vertical-ish plane, chosen by what the right arm is doing
//! (`plane_normal`):
//!
//!  - **Not aiming** (standing, walking, running, the gun hanging at his side: the aim's weight is
//!    0, play.rs `Player::aim_weight`): **the torso's own plane**, the one between the shoulders (its
//!    normal the line from the left upper arm to the right in the bone both clavicles hang from, the
//!    neck, so it turns with the shoulders). The arms are then plain mirrors of each other: both
//!    hang, both swing, both guns held alike. (#409: until then the plane followed the right
//!    gun's barrel whenever it was more than 12 degrees off the vertical (less than 78 up or
//!    down), and a hanging gun's barrel points down and a little back or in, 73 degrees below the
//!    horizontal in Kingman's carry idle,
//!    so its heading, a few centimetres of horizontal run, was anywhere: 60 degrees off the
//!    torso's plane, which put the left hand in front of his crotch with the gun on its side (its
//!    muzzle 0.08 m right of his centre line where a mirror has it 0.28 m left of it) while the
//!    right hung at his side, and wobbled the left arm in a walk as the heading wandered.)
//!  - **Aiming or firing** (the aim's weight 1): **the aim's vertical plane**, the plane through
//!    the point between the shoulders that holds the right gun's barrel direction and the up axis.
//!    (The torso is not square to the aim: standing, it stays about 16 degrees off it, and a mirror
//!    across the torso's own plane doubled that: the left barrel splayed 25 - 30 degrees outward and
//!    the hand sat farther from the body, round 3 of #407.) With the barrel in the plane its
//!    reflection is itself, so both barrels are parallel to the aim, and the two hands are level
//!    and the same distance from the aim line; only a shift along the aim of the shoulders' offset
//!    from the plane remains (about 0.15 m at 16 degrees).
//!  - **Between**: the two normals blended by the aim's weight, and the aim's share faded out as the
//!    barrel nears the vertical (`MIN_HEADING`, `FULL_HEADING`), so a gun going up to the aim, or
//!    coming down from it, swings the left arm through the planes between and the plane never
//!    jumps.
//!
//! The bones are the Biped's (`Bip01 R|L Clavicle`, `UpperArm`, `Forearm`, `Hand`, the names the
//! game's hit parts use, play_damage.rs `HIT_BONE_NAMES`). A bone's frame, reflected across a plane
//! with the matrix `M`, is `M W K`, where `K` is a constant reflection in the bone's own axes that
//! turns the right bone's frame into the left's at rest (`K = Rr0^T M0 Rl0`, with `M0` the rest
//! pose's own symmetry plane, between the shoulders in the bind pose, `Character::bind_local`): so
//! the left bone is `Wl = M Wr K`, and at rest `M = M0` gives the bind pose back. A gun held by the
//! right hand at `(r, t)` is held by the left at `(K^T r S, K^T t)`, with `S` the reflection of the
//! gun's own lateral axis (`lateral`), which keeps its barrel and its grip where they were: the
//! left gun has the right one's orientation and the mirror of its place, right way up. The left
//! hip's holster is the right one's mirrored the same way across the pelvis's own middle plane
//! (`mirror_on_bone`, #410: play_holster.rs). A model missing any of the eight arm bones, or whose
//! clavicles hang from different bones, has no `MirrorArm`.
//!
//! **Kingman's left-side gun (#410)**: while it is in hand his arms are exchanged (`swap`: each the
//! mirror of the other across the torso's plane, so the left carries and aims as the clips have the
//! right do it), and in berserk the right arm is the mirror of the left (`apply` from the left,
//! `M Wl K^T`).

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;

/// The bone names of an arm, top down, with the side letter to put in: `Bip01 {side} Clavicle` ...
const ARM: [&str; 4] = ["Clavicle", "UpperArm", "Forearm", "Hand"];
/// The least horizontal length of the barrel's direction (unit barrel) for its heading to give any
/// of the aim's plane (`plane_normal`): 0.2 is 11.5 degrees off the vertical, so a barrel within
/// 12 degrees of the vertical (78 degrees or more up or down) has a heading that swings with the
/// smallest turn of the wrist, and the torso's plane is used.
const MIN_HEADING: f32 = 0.2;
/// From this horizontal length of the unit barrel (0.4 is 23.6 degrees off the vertical: 24 degrees
/// or more off it, 66 up or down or less, past the aim's pitch limit of 60 degrees,
/// play_camera.rs `PITCH_LIMIT`, and the crosshair's lift above it) the barrel's heading is trusted
/// in full; between `MIN_HEADING` and this its share ramps up in a line.
const FULL_HEADING: f32 = 0.4;
/// The seconds the left arm takes to be taken over by the mirror of the right as the second gun is
/// drawn (or the right by the left's, with Kingman's left-side gun in hand, #410), and to give itself
/// back as it is stowed (play.rs `Player::mirror`): the time a hand takes between its pose and the
/// hip (play_reach.rs `LEG_TIME`, #410), so as Kingman's hand brings the second gun up from its hip
/// the mirror comes in with it, and goes as the hand comes back empty.
/// **The demo's choice**: before #409 the pose snapped on and off.
pub const BLEND_TIME: f32 = super::reach::LEG_TIME;

/// A model's two arms, to pose the left as the mirror of the right (see the module notes).
#[derive(Clone, Debug)]
pub struct MirrorArm {
    /// (right, left) bone indices, clavicle first, then the upper arm, forearm and hand
    pairs: [(usize, usize); 4],
    /// the bone both clavicles hang from
    parent: usize,
    /// the torso's mirror plane's unit normal in the parent's frame (left to right shoulder in the
    /// bind pose), the plane used when the arms aren't aiming
    normal: Vec3,
    /// for each pair, the constant reflection `K` in the bone's own axes (see the module notes)
    flip: [Mat3; 4],
    /// the two thighs (right, left), whose parent the holsters ride on (play_holster.rs); None for a
    /// model without them
    legs: Option<(usize, usize)>,
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

/// The unit normal of the plane the left arm is mirrored across (see the module notes): `torso`, the
/// shoulders' normal (left to right, in the model's space), when the arms aren't aiming, the
/// normal of the vertical plane that holds `barrel`, the driving gun's direction in the model's
/// space (the right gun's, or Kingman's left-side gun's, #410), when `aim`, the aim's weight
/// (0 - 1), is 1, and the two blended between. The aim's share
/// is the weight times how far the barrel's heading can be trusted: none within 12 degrees of the
/// vertical (78 up or down: `MIN_HEADING`), all from 24 degrees off it (66 up or down:
/// `FULL_HEADING`).
fn plane_normal(torso: Vec3, barrel: Vec3, aim: f32) -> Vec3 {
    let heading = Vec3::new(barrel.x, 0.0, barrel.z);
    let length = barrel.length().max(1e-6);
    let trust = ((heading.length() / length - MIN_HEADING) / (FULL_HEADING - MIN_HEADING)).clamp(0.0, 1.0);
    let share = aim.clamp(0.0, 1.0) * trust;
    if share <= 0.0 {
        return torso;
    }
    let mut across = Vec3::new(-heading.z, 0.0, heading.x).normalize();
    // (a plane has two normals: the one on the shoulders' side, so the blend can't pass through 0)
    if across.dot(torso) < 0.0 {
        across = -across;
    }
    (torso * (1.0 - share) + across * share).try_normalize().unwrap_or(torso)
}

/// The mount a gun held at the right bone's `(r, t)` has on the left bone: `(K^T r S, K^T t)`, with
/// `flip` the pair's `K` and `S` the reflection of the gun's lateral axis (`lateral` of `barrel`, the
/// firing direction, and `offset`, the muzzle's place from the grip's, both in the gun's own axes).
fn mirrored_mount(flip: Mat3, r: Quat, t: Vec3, barrel: Vec3, offset: Vec3) -> (Quat, Vec3) {
    let k_inverse = flip.transpose();
    let rotation = Quat::from_mat3(&(k_inverse * Mat3::from_quat(r) * reflection(lateral(barrel, offset)))).normalize();
    (rotation, k_inverse * t)
}

/// A gun's mount `(r, t)` on a bone that is its own mirror, mirrored across a plane of that bone's
/// frame (its unit `normal`, through `point`): the place's mirror, and the mirrored gun's
/// orientation, right way up, as the hand's mount is mirrored (`mirrored_mount`, with `K` the plane's
/// reflection; `barrel` and `offset` as there). #410: Kingman's left holster, the right one's mirror
/// across his pelvis's middle (play_holster.rs).
pub fn mirror_on_bone(normal: Vec3, point: Vec3, (r, t): (Quat, Vec3), barrel: Vec3, offset: Vec3) -> (Quat, Vec3) {
    let (rotation, place) = mirrored_mount(reflection(normal), r, t - point, barrel, offset);
    (rotation, place + point)
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
        let flip_of = |r: usize, l: usize| {
            let (rr, rl) = (Mat3::from_quat(rot(&world[r])), Mat3::from_quat(rot(&world[l])));
            rr.transpose() * rest_plane * rl
        };
        let flip = pairs.map(|(r, l)| flip_of(r, l));
        let legs = find("R", "Thigh").zip(find("L", "Thigh"));
        Some(MirrorArm { pairs, parent, normal, flip, legs })
    }

    /// The normal of the plane one arm is mirrored across in `pose` (the model's local transforms,
    /// the driving arm and the spine posed): `barrel` the driving gun's direction in its hand bone's
    /// own axes (the mount's rotation times the gun's firing direction), that hand the left one if
    /// `left` (Kingman's left-side gun, #410), else the right; `aim` the aim's weight
    /// (`plane_normal`).
    pub fn plane(&self, model: &Character, pose: &[(Quat, Vec3)], barrel: Vec3, aim: f32, left: bool) -> Vec3 {
        let world = model.world(pose);
        let torso = rot(&world[self.parent]) * self.normal;
        let hand = if left { self.pairs[3].1 } else { self.pairs[3].0 };
        plane_normal(torso, rot(&world[hand]) * barrel, aim)
    }

    /// The torso's own mirror plane's normal in `pose` (the line between the shoulders, in the bone
    /// both clavicles hang from): the plane the arms are exchanged across (`swap`).
    pub fn torso(&self, model: &Character, pose: &[(Quat, Vec3)]) -> Vec3 {
        rot(&model.world(pose)[self.parent]) * self.normal
    }

    /// Poses one arm as the mirror of the other in `pose` (the model's local transforms, the
    /// driving arm and the spine already posed), across `plane`'s plane: the left arm from the right,
    /// each left bone taking `M Wr K` (see the module notes), or, `from_left` (#410: Kingman's
    /// left-side gun drives), the right from the left, each right bone `M Wl K^T`; `blend` of the way
    /// from the bone as the clip has it (0: nothing is touched, 1: the mirror, for the second gun's
    /// draw and stow to ease the arm in and out).
    pub fn apply(&self, model: &Character, pose: &mut [(Quat, Vec3)], normal: Vec3, blend: f32, from_left: bool) {
        if blend <= 0.0 {
            return;
        }
        let world = model.world(pose);
        let plane = reflection(normal);
        for (i, &(r, l)) in self.pairs.iter().enumerate() {
            let (from, to, flip) = if from_left { (l, r, self.flip[i].transpose()) } else { (r, l, self.flip[i]) };
            let turned = Quat::from_mat3(&(plane * Mat3::from_quat(rot(&world[from])) * flip)).normalize();
            // (the bone's parent is the spine bone for the clavicle, else the bone above, already
            // turned: its world is read again each time)
            let above = model.world(pose)[model.parent[to].unwrap_or(self.parent)];
            let target = (rot(&above).inverse() * turned).normalize();
            pose[to].0 = if blend >= 1.0 { target } else { pose[to].0.slerp(target, blend).normalize() };
        }
    }

    /// Exchanges the two arms by their mirrors across `normal`'s plane (the torso's, `torso`), `blend`
    /// of the way (#410: while Kingman holds a left-side gun his left arm carries and aims it as the
    /// clips have the right arm carry theirs, and the right arm hangs empty as the clips have the
    /// left): each left bone takes the mirror of the right one as `pose` had it, `M Wr K`, and each
    /// right bone the mirror of the left one, `M Wl K^T`, both read before either is turned.
    pub fn swap(&self, model: &Character, pose: &mut [(Quat, Vec3)], normal: Vec3, blend: f32) {
        if blend <= 0.0 {
            return;
        }
        let world = model.world(pose);
        let plane = reflection(normal);
        let mirror = |from: usize, flip: Mat3| Quat::from_mat3(&(plane * Mat3::from_quat(rot(&world[from])) * flip)).normalize();
        let targets: Vec<[(usize, Quat); 2]> = self.pairs.iter().enumerate()
            .map(|(i, &(r, l))| [(l, mirror(r, self.flip[i])), (r, mirror(l, self.flip[i].transpose()))]).collect();
        // (top down, both arms at each level: a bone's parent is turned before it is)
        for pair in targets {
            for (bone, turned) in pair {
                let above = model.world(pose)[model.parent[bone].unwrap_or(self.parent)];
                let target = (rot(&above).inverse() * turned).normalize();
                pose[bone].0 = if blend >= 1.0 { target } else { pose[bone].0.slerp(target, blend).normalize() };
            }
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
        let (rotation, place) = mirrored_mount(self.flip[3], r, t, barrel, offset);
        Some((self.pairs[3].1, rotation, place))
    }

    /// The two thighs' bones, right and left (`Bip01 R|L Thigh`): Flint's stow hardpoint is copied
    /// onto the right one, and the holsters ride on the bone both hang from (play_holster.rs); None
    /// for a model without them.
    pub fn thighs(&self) -> Option<(usize, usize)> {
        self.legs
    }

    /// One arm's bones, clavicle, upper arm, forearm and hand: the left one's if `left`, else the
    /// right one's (play_reach.rs's reach to the hip, #410).
    pub fn bones(&self, left: bool) -> [usize; 4] {
        self.pairs.map(|(r, l)| if left { l } else { r })
    }

    /// The point between the shoulders (the middle of the two upper arms' origins) in the bones'
    /// matrices `world`: the point the plane of the mirror passes through (BF_MIRROR_LOG's check).
    pub fn between_shoulders(&self, world: &[Mat4]) -> Vec3 {
        (world[self.pairs[1].0].w_axis.truncate() + world[self.pairs[1].1].w_axis.truncate()) * 0.5
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

    /// Not aiming, the plane is the torso's whatever the barrel does: Kingman's hanging gun, 73
    /// degrees below the horizontal and 51 degrees round (#409's screenshot), where the aim's plane
    /// would be turned 51 degrees off the body's.
    #[test]
    fn not_aiming_the_plane_is_the_torsos() {
        let torso = Vec3::new(0.2, 0.0, 0.98).normalize();
        let hanging = Vec3::new(0.78 * 0.285, -0.958, -0.63 * 0.285).normalize();
        assert_eq!(plane_normal(torso, hanging, 0.0), torso);
        // (a barrel along the shoulders' line, which the aim's plane would take for its heading)
        assert_eq!(plane_normal(torso, Vec3::X, 0.0), torso);
    }

    /// Aiming along a level barrel, the plane is the aim's: the normal is horizontal and square to
    /// the barrel, on the shoulders' side, whatever way the torso is turned.
    #[test]
    fn aiming_the_plane_is_the_aims() {
        let torso = Vec3::new(0.97, 0.05, 0.24).normalize();
        let barrel = Vec3::new(0.0, 0.1, -1.0).normalize();
        let n = plane_normal(torso, barrel, 1.0);
        assert!((n - Vec3::X).length() < 1e-6, "{n:?}");
        // (a barrel with the other heading puts the normal on the same side as the shoulders')
        let n = plane_normal(Vec3::NEG_X, barrel, 1.0);
        assert!((n - Vec3::NEG_X).length() < 1e-6, "{n:?}");
    }

    /// A barrel near the vertical gives the aim's plane no say however hard the aim is held: the
    /// heading of a gun pointing down is noise (the hanging gun again, aim 1).
    #[test]
    fn a_vertical_barrel_has_no_heading_to_trust() {
        let torso = Vec3::new(0.8, 0.0, 0.6);
        let down = Vec3::new(0.15, -0.98, -0.1).normalize();
        assert_eq!(plane_normal(torso, down, 1.0), torso);
        // (and half-way up the ramp it has half: 0.3 of the barrel's length is horizontal, along
        // -z, whose plane has the normal +x)
        let tilted = Vec3::new(0.0, -(1.0f32 - 0.09).sqrt(), -0.3);
        let n = plane_normal(torso, tilted, 1.0);
        assert!(n.x > 0.8 && n.x < 1.0 - 1e-3, "{n:?}");
    }

    /// The blend between the two planes is continuous: the aim's weight 0 to 1 moves the normal
    /// steadily from the torso's to the aim's, never past either.
    #[test]
    fn the_planes_blend_steadily() {
        let torso = Vec3::new(0.95, 0.0, 0.31).normalize();
        let barrel = Vec3::new(0.0, 0.0, -1.0);
        let mut last = plane_normal(torso, barrel, 0.0);
        assert_eq!(last, torso);
        for i in 1..=20 {
            let n = plane_normal(torso, barrel, i as f32 / 20.0);
            assert!((n - last).length() < 0.06, "{i}: {n:?} from {last:?}");
            assert!(n.dot(Vec3::X) >= last.dot(Vec3::X) - 1e-6);
            last = n;
        }
        assert!((last - Vec3::X).length() < 1e-5);
    }

    /// A skeleton of eleven bones whose `K`s are a reflection of x (a model whose left bones are the
    /// right ones' axes with x turned over: the real `K` is a reflection too, `Rr0^T M0 Rl0`): a right
    /// hand's mount goes to the left hand, nobody else's anywhere, the place mirrored by `K^T` and the
    /// rotation a proper one; and a mount on a bone that is its own mirror goes across the plane given
    /// in its frame, through the point given (#410's holsters on the pelvis).
    #[test]
    fn mounts_go_from_the_right_bones_to_the_left_ones() {
        let k = reflection(Vec3::X);
        let arm = MirrorArm { pairs: [(1, 2), (3, 4), (5, 6), (7, 8)], parent: 0, normal: Vec3::X, flip: [k; 4], legs: Some((9, 10)) };
        let (r, t) = (Quat::from_rotation_x(0.7), Vec3::new(0.1, -0.2, 0.3));
        let (barrel, offset) = (Vec3::Z, Vec3::new(0.0, 0.1, 0.4));
        let mirrored = Vec3::new(-0.1, -0.2, 0.3);
        let hand = arm.mount((7, r, t), barrel, offset).expect("the right hand's mount");
        assert_eq!(hand.0, 8);
        assert!((hand.2 - mirrored).length() < 1e-6 && hand.1.is_normalized());
        assert!(arm.mount((8, r, t), barrel, offset).is_none());
        assert_eq!(arm.thighs(), Some((9, 10)));
        let world = vec![Mat4::IDENTITY; 11];
        assert_eq!(arm.between_shoulders(&world), Vec3::ZERO);
        // (across the plane x = 0.02 of the bone's frame: the mirrored mount's, moved by twice that)
        let (hr, ht) = mirror_on_bone(Vec3::X, Vec3::new(0.02, 0.5, 0.0), (r, t), barrel, offset);
        assert!((ht - Vec3::new(-0.06, -0.2, 0.3)).length() < 1e-6, "{ht:?}");
        assert!(hr.angle_between(hand.1) < 1e-5);
    }

    /// A body for `swap` and `apply`: the spine 0, the right arm's clavicle, upper arm, forearm and
    /// hand 1, 3, 5, 7 and the left's 2, 4, 6, 8, the left the mirror of the right across x = 0 (so
    /// every `K` is the reflection of x), the right arm turned a way of its own and the left another.
    fn two_arms() -> (Character, MirrorArm, Vec<(Quat, Vec3)>) {
        let n = 9;
        let model = Character {
            bones: (0..n as u32).collect(), parent: vec![None, Some(0), Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)],
            bind_rows: vec![Mat3::IDENTITY; n], bind_pos: vec![Vec3::ZERO; n], parent_point: vec![Vec3::ZERO; n],
            child_point: vec![Vec3::ZERO; n], inverse_bind: vec![Mat4::IDENTITY; n], geosets: vec![], anims: vec![], face_anims: vec![],
            default_face: None, face: None, hardpoints: Default::default(), part_collision: vec![vec![]; n], part_mass_centre: vec![None; n],
            part_spheres: vec![None; n], bounds: None,
        };
        let k = reflection(Vec3::X);
        let arm = MirrorArm { pairs: [(1, 2), (3, 4), (5, 6), (7, 8)], parent: 0, normal: Vec3::X, flip: [k; 4], legs: None };
        let place = |b: usize| {
            let x = if b % 2 == 1 { 1.0 } else { -1.0 };
            match b {
                0 => Vec3::new(0.0, 1.4, 0.0),
                1 | 2 => Vec3::new(0.05 * x, 0.1, 0.0),
                3 | 4 => Vec3::new(0.15 * x, 0.0, 0.0),
                _ => Vec3::new(0.0, -0.3, 0.0),
            }
        };
        let turn = [Quat::IDENTITY, Quat::from_rotation_z(0.2), Quat::from_rotation_z(-0.1), Quat::from_rotation_x(-0.9) * Quat::from_rotation_z(0.3),
                    Quat::from_rotation_x(0.2), Quat::from_rotation_x(-0.8), Quat::from_rotation_y(0.3), Quat::from_rotation_y(0.5),
                    Quat::from_rotation_z(-0.2)];
        let pose = (0..n).map(|b| (turn[b], place(b))).collect();
        (model, arm, pose)
    }

    /// The exchange of the arms (#410): each hand ends where the other hand's mirror was, turned as
    /// its mirror (`M W K`), and exchanging them twice gives the pose back; a blend of 0 touches
    /// nothing.
    #[test]
    fn exchanging_the_arms_mirrors_each_into_the_other() {
        let (model, arm, pose) = two_arms();
        let before = model.world(&pose);
        let mut swapped = pose.clone();
        arm.swap(&model, &mut swapped, Vec3::X, 1.0);
        let after = model.world(&swapped);
        let m = reflection(Vec3::X);
        for (from, to) in [(7, 8), (8, 7), (5, 6), (6, 5)] {
            assert!((after[to].w_axis.truncate() - m * before[from].w_axis.truncate()).length() < 1e-5, "bone {to}'s place");
            let mirrored = Quat::from_mat3(&(m * Mat3::from_quat(rot(&before[from])) * m));
            assert!(rot(&after[to]).angle_between(mirrored) < 1e-3, "bone {to}'s turn");
        }
        arm.swap(&model, &mut swapped, Vec3::X, 1.0);
        for (b, (r, t)) in swapped.iter().enumerate() {
            assert!(r.angle_between(pose[b].0) < 1e-3 && (*t - pose[b].1).length() < 1e-6, "bone {b}");
        }
        let mut none = pose.clone();
        arm.swap(&model, &mut none, Vec3::X, 0.0);
        assert_eq!(none, pose);
    }

    /// The right arm as the mirror of the left (`apply` from the left, #410: a left-side gun in
    /// hand): the right hand where the left one's mirror is, and the left arm untouched.
    #[test]
    fn the_right_arm_can_mirror_the_left() {
        let (model, arm, pose) = two_arms();
        let before = model.world(&pose);
        let mut posed = pose.clone();
        arm.apply(&model, &mut posed, Vec3::X, 1.0, true);
        let after = model.world(&posed);
        assert!((after[7].w_axis.truncate() - reflection(Vec3::X) * before[8].w_axis.truncate()).length() < 1e-5);
        for b in [2, 4, 6, 8] {
            assert_eq!(posed[b], pose[b]);
        }
    }

    /// The mirrored mount keeps the gun the right way up: the left bone is `Wl = M Wr K` (M the
    /// mirror, Wr the right bone, K the bones' own reflection), the gun on it at the mount
    /// `(K^T r S, K^T t)`, and in the world its barrel is the reflection of the right gun's barrel and its
    /// up the reflection of the right gun's up (both guns' grips down), its place the reflection of
    /// the right gun's.
    #[test]
    fn a_mirrored_mount_keeps_the_guns_barrel_and_up() {
        let (barrel, up, offset) = (Vec3::NEG_Z, Vec3::Y, Vec3::new(0.0, 0.06, -0.2));
        let r = Quat::from_rotation_x(1.2) * Quat::from_rotation_y(0.4);
        let t = Vec3::new(0.1, 0.2, 0.3);
        let wr = Mat3::from_quat(Quat::from_rotation_z(0.5) * Quat::from_rotation_x(-0.8));
        let m = reflection(Vec3::new(0.6, 0.0, 0.8));
        let k = reflection(Vec3::new(0.0, 0.6, 0.8));
        let wl = m * wr * k;
        assert!((wl.determinant() - 1.0).abs() < 1e-5);
        let (left, place) = mirrored_mount(k, r, t, barrel, offset);
        let (right_barrel, right_up) = (wr * (r * barrel), wr * (r * up));
        assert!((wl * (left * barrel) - m * right_barrel).length() < 1e-5);
        assert!((wl * (left * up) - m * right_up).length() < 1e-5);
        assert!((wl * place - m * (wr * t)).length() < 1e-5);
    }
}
