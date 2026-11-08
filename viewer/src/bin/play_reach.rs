//! Edward Kingman's hands reaching to his hip holster as he draws and holsters (#410, **demo-own**:
//! the product owner's request after #409). His motion set ms_kingman has no holster, draw or switch
//! clip (its only weapon overlay is `Sc_w1_drop_pickup_w1`, a ground pickup), so the demo poses the
//! arm itself: a two-bone IK on `Bip01 R|L UpperArm`, `Forearm` and `Hand`, laid over whatever the
//! clips, the aim and the mirrored left arm (play_mirror_arm.rs) have made of the pose, so the legs
//! keep walking or running under it.
//!
//!  - **The swap** (play.rs `switch_clips_of`, Kingman's timed switch): the right hand carries the
//!    gun in it across his front to the holster at the front of his left hip (play_holster.rs, which
//!    #410 moves up there from his thigh, out of this hand's reach) over `LEG_TIME`, lets it go there
//!    (the switch's drop: the gun is the holster's from that frame), stays `DWELL` while it closes on
//!    the other, takes it (the grab) and brings it up to the carry pose over `LEG_TIME`; the aim comes
//!    after, as after any switch. One hand does both legs, the holster first and the draw after it,
//!    so the two guns never travel at once (the two Kman Autos are one model: at the hip, where the
//!    hand lets one go and takes the other, they lie in one place).
//!  - **The berserk draw and stow** (`Reaches::left_gun`): the left hand reaches forward to the same
//!    holster, closes on the second gun there and brings it up into the mirrored pose; the stow takes
//!    it back down, lets go and the arm returns to its clip. The second gun is drawn the moment the
//!    skill says so (play_skills.rs: the Berserker on a type of its own has no toggle clip), but it
//!    stays on the hip until the hand gets there, and fires only once the hand has it up, `2 LEG_TIME`
//!    on; the mirrored arm (play_mirror_arm.rs) eases in as the hand comes up with it and out as it
//!    comes back empty.
//!  - **The arm** (`pose_arm`): the gun's grip point follows an eased path from where the hand holds
//!    it in the pose to where it is at the holster, bowed out in front of the body as far as the path
//!    crosses it (`BOW`); the elbow turns from where the pose bends it toward its pole (`POLE_RIGHT`,
//!    `POLE_LEFT`: out and forward across the body, out and back on its own side); the hand turns from
//!    its own turn under the forearm half way to the turn that holds the holstered gun in its mount
//!    (`WRIST_SHARE`), and the gun turns in the hand about its grip for the rest, as far along as the
//!    leg is (`Reach::hold`), so at the hip the gun in the hand is exactly where the holster has it
//!    and the hand-over pops nothing; the clavicle and, for the reach across the body, the spine turn
//!    toward the hip (`CLAVICLE_REACH`, `TWIST_ACROSS`) where the arm alone falls short. At the start of
//!    a leg the IK gives the pose back exactly (the target is where the hand is, the pole the elbow's
//!    own bend, the hand's turn its own), so nothing pops in or out.
//!
//! Only the type that has `Holsters` (play_holster.rs: the Berserker on a model with no stow
//! hardpoints, the demo's Kingman class) reaches; every other character's switch and berserk is as
//! it was.

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;

use super::mirror_arm::MirrorArm;

/// The seconds a hand takes from where its pose has it to the hip, and back. **The demo's choice**
/// (#410): a quarter of a second each way, as #409's glide took; the swap's two legs and the hold
/// between come to `2 LEG_TIME + DWELL`, 0.55 s, against #409's 0.35 s and the squad's pistol
/// switches' 1.1 s (Flint's `Sc_w1_2_w2`).
pub const LEG_TIME: f32 = 0.25;
/// The seconds the right hand stays at the hip in a swap between letting the one gun go and taking
/// the other. **The demo's choice**: long enough to read as two acts (three frames at 60 fps).
pub const DWELL: f32 = 0.05;
/// How far the wrist's path bows out in front of the body (m) for each metre it goes across it, at
/// the middle of the leg. **The demo's choice** (#410), fitted to frames from above and the side: a
/// path across the front of the pelvis, from his right side to his left hip, has to clear his
/// coat; one down his own side hardly bows.
const BOW: f32 = 0.45;
/// The model's forward direction (play.rs `yaw_of`: a heading of atan2(-x, -z), so forward is -z),
/// which the bow goes along.
const FORWARD: Vec3 = Vec3::NEG_Z;
/// The direction the elbow bends toward at the hip, in the torso's frame as the bind pose has it
/// (+x his right, +y up, -z forward): the right arm reaching across the body bends its elbow out to
/// the right and forward, the left arm reaching forward on its own side out and back, its elbow
/// staying at his side. **The demo's choice** (#410, the lead's: "the elbow pole out and forward"
/// for the cross-body reach).
const POLE_RIGHT: Vec3 = Vec3::new(0.75, -0.25, -0.6);
const POLE_LEFT: Vec3 = Vec3::new(-0.75, -0.25, 0.6);
/// The most the clavicle turns (rad) to carry the shoulder toward a hip the arm can't reach
/// straight, and the share of the arm's full length the wrist may be from the shoulder before it
/// does. **The demo's choice**: 25 degrees, a shrug's.
const CLAVICLE_REACH: f32 = 0.44;
const STRAIGHT: f32 = 0.97;
/// How far (rad) the spine above the hips turns toward the far hip at the full reach across the
/// body (the right hand to the left hip), spread over the bones that carry the arm, so the shoulder
/// comes round to it. **The demo's choice**: 20 degrees, a turn into a cross draw.
const TWIST_ACROSS: f32 = 0.35;
/// How far (0 - 1) the hand turns at the hip, from its own turn under the forearm (the clip's wrist)
/// toward the turn that holds the holstered gun in its mount; the gun turns in the hand, about its
/// grip, for the rest. **The demo's choice** (#410): one holster for both hands, which come to it from
/// opposite sides, needs either a wrist bent far past the clip's or a gun turned in the hand; the
/// wrist taking it all bent it up to 120 degrees from straight, and its forearm's roll and its turn
/// flipped in a frame where the turns passed half a turn (BF_REACH_LOG: 152 degrees in a frame), so
/// each takes half (about 35 degrees each at the holster's `WAIST_TURN`, play_holster.rs).
const WRIST_SHARE: f32 = 0.5;
/// The passes that settle the hand's turn and the wrist's place together (`pose_arm`). **The demo's
/// choice**: the grip point lies a few centimetres from the wrist, so they settle in a few.
const PASSES: usize = 3;

/// The rotation of a transform.
fn rot(m: &Mat4) -> Quat {
    m.to_scale_rotation_translation().1
}

/// The place of a transform.
fn at(m: &Mat4) -> Vec3 {
    m.w_axis.truncate()
}

/// The ease of a leg: smoothstep, so the hand starts and stops gently.
pub fn ease(s: f32) -> f32 {
    let s = s.clamp(0.0, 1.0);
    s * s * (3.0 - 2.0 * s)
}

/// One arm's bones, top down.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bones {
    clavicle: usize,
    upper: usize,
    fore: usize,
    hand: usize,
}

/// A model's two arms for the reach (see the module notes), built with its holsters.
#[derive(Clone, Debug)]
pub struct Arms {
    right: Bones,
    left: Bones,
    /// the bone both clavicles hang from, and its rotation in the bind pose (model space): the
    /// poles are in its frame as the bind pose has it
    torso: usize,
    torso_rest: Quat,
    /// the lines of the body a hand on its way to the hip must keep clear of, for BF_REACH_LOG
    /// (play.rs `reach_log`): each thigh from its joint to the knee, the spine from its lowest bone
    /// to the one the clavicles hang from, by bone and with their names
    body: Vec<(usize, usize, &'static str)>,
}

impl Arms {
    /// The arms of `model` (`arm`'s bones), when each forearm hangs from its upper arm and each
    /// hand from its forearm, as the Biped has them; None otherwise.
    pub fn new(model: &Character, arm: &MirrorArm) -> Option<Self> {
        let bones = |left: bool| {
            let [clavicle, upper, fore, hand] = arm.bones(left);
            (model.parent[fore] == Some(upper) && model.parent[hand] == Some(fore)).then_some(Bones { clavicle, upper, fore, hand })
        };
        let (right, left) = (bones(false)?, bones(true)?);
        let torso = model.parent[right.clavicle]?;
        let torso_rest = rot(&model.world(&model.bind_local())[torso]);
        let find = |name: &str| model.bones.iter().position(|b| *b == h(name));
        let body = [("Bip01 L Thigh", "Bip01 L Calf", "left thigh"), ("Bip01 R Thigh", "Bip01 R Calf", "right thigh")].into_iter()
            .filter_map(|(a, b, name)| Some((find(a)?, find(b)?, name)))
            .chain(find("Bip01 Spine").map(|s| (s, torso, "spine")))
            .collect();
        Some(Arms { right, left, torso, torso_rest, body })
    }

    /// The hand bone of the right arm or, `left`, the left one.
    pub fn hand(&self, left: bool) -> usize {
        if left { self.left.hand } else { self.right.hand }
    }

    /// The elbow (the forearm's joint) of the right arm or, `left`, the left one, in the bones'
    /// matrices `world`.
    pub fn elbow(&self, world: &[Mat4], left: bool) -> Vec3 {
        at(&world[if left { self.left.fore } else { self.right.fore }])
    }

    /// How far (degrees) the arm's upper arm, forearm and hand turned in the model's space from the
    /// bones' matrices `before` to `now` (two frames): BF_REACH_LOG's measure of a pop. Zeros when
    /// `before` isn't this model's.
    pub fn turned(&self, before: &[Mat4], now: &[Mat4], left: bool) -> [f32; 3] {
        let arm = if left { self.left } else { self.right };
        [arm.upper, arm.fore, arm.hand].map(|b| match (before.get(b), now.get(b)) {
            (Some(a), Some(n)) if before.len() == now.len() => rot(a).angle_between(rot(n)).to_degrees(),
            _ => 0.0,
        })
    }

    /// The body's lines (see `Arms::body`) in the bones' matrices `world`: from, to, name.
    pub fn body(&self, world: &[Mat4]) -> Vec<(Vec3, Vec3, &'static str)> {
        self.body.iter().map(|&(a, b, name)| (at(&world[a]), at(&world[b]), name)).collect()
    }
}

/// The distance from `p` to the segment from `a` to `b`.
pub fn distance_to_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let k = if ab.length_squared() > 1e-12 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(a + ab * k)
}

/// One hand's reach to the hip (see the module notes): how far along it is, 0 where its pose has it
/// and 1 at the hip, moving `1 / LEG_TIME` a second, and the gun it goes for.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reach {
    pub at: f32,
    pub gun: Option<usize>,
    /// this frame's place of a gun in this hand under the hand bone (rotation, translation:
    /// `pose_arm`'s, its mount turned about the grip toward the holster's) while the reach is under
    /// way, whether the hand holds one or not (a switch cut short gives it the gun after the pose);
    /// play.rs `weapon_fx` puts a gun in this hand there in place of its mount
    pub hold: Option<(Quat, Vec3)>,
}

impl Reach {
    /// Moves on by `dt`: toward the hip if `out`, else back. Whether it got to the hip this step.
    pub fn step(&mut self, out: bool, dt: f32) -> bool {
        let was = self.at;
        self.at = (self.at + if out { dt } else { -dt } / LEG_TIME).clamp(0.0, 1.0);
        out && was < 1.0 && self.at >= 1.0
    }

    /// Whether the hand is anywhere but in its own pose.
    pub fn busy(&self) -> bool {
        self.at > 0.0
    }
}

/// Both hands' reaches (`Player::reach`) and what the left hand holds.
#[derive(Clone, Debug, Default)]
pub struct Reaches {
    pub right: Reach,
    pub left: Reach,
    /// the gun in the left hand (the berserk's second gun once the hand has taken it from the hip,
    /// until it puts it back): it hangs there, fires and is mirrored from the right only then
    pub left_gun: Option<usize>,
}

impl Reaches {
    /// The left hand's step (see the module notes): it goes to the hip while the gun the skill wants
    /// in it (`second`, the berserk's) isn't the one it holds, and back once it is. At the hip it
    /// takes the wanted gun, or lets go of the one it has. Whether it took or let go this step (the
    /// guns are to be placed again).
    pub fn step_left(&mut self, second: Option<usize>, dt: f32) -> bool {
        let out = second != self.left_gun;
        if out {
            self.left.gun = second.or(self.left_gun);
        }
        if self.left.step(out, dt) {
            self.left_gun = second;
            return true;
        }
        false
    }

    /// Whether either hand is on its way to or from the hip.
    pub fn busy(&self) -> bool {
        self.right.busy() || self.left.busy()
    }
}

/// The point the gun's grip is at `e` (eased, 0 - 1) of the way from `from` (where the hand holds it
/// in the pose) to `to` (at the holster): a line, bowed out in front of the body by `BOW` of the width
/// it crosses.
fn waypoint(from: Vec3, to: Vec3, e: f32) -> Vec3 {
    let across = (to - from).dot(FORWARD.cross(Vec3::Y)).abs();
    from.lerp(to, e) + FORWARD * (BOW * across * (std::f32::consts::PI * e).sin())
}

/// Sets bone `b`'s rotation so it is `turn` in the model's space, under its parent as `pose` has it.
fn set_world_rot(model: &Character, pose: &mut [(Quat, Vec3)], b: usize, turn: Quat) {
    let above = model.parent[b].map_or(Quat::IDENTITY, |p| rot(&model.world(pose)[p]));
    pose[b].0 = (above.inverse() * turn).normalize();
}

/// Turns bone `b` by `turn` in the model's space, about its own joint.
fn turn_bone(model: &Character, pose: &mut [(Quat, Vec3)], b: usize, turn: Quat) {
    let now = rot(&model.world(pose)[b]);
    set_world_rot(model, pose, b, turn * now);
}

/// Two-bone IK: turns `arm`'s upper arm and forearm so its wrist (the hand bone's joint) is at
/// `target`, the elbow bent toward `pole` (a direction in the model's space; only its part square to
/// the line from the shoulder counts). A target past the arm's reach leaves the arm straight toward
/// it. The bones are turned the least way from where they were, so their roll is the pose's, and the
/// hand keeps its turn under the forearm (`pose_arm` turns it after).
fn two_bone(model: &Character, pose: &mut [(Quat, Vec3)], arm: Bones, target: Vec3, pole: Vec3) {
    let world = model.world(pose);
    let (s, e, w) = (at(&world[arm.upper]), at(&world[arm.fore]), at(&world[arm.hand]));
    let (a, b) = ((e - s).length(), (w - e).length());
    if a < 1e-4 || b < 1e-4 {
        return;
    }
    let to = target - s;
    let d = to.length().clamp((a - b).abs() + 1e-4, a + b - 1e-4);
    let u = to.try_normalize().unwrap_or((w - s).normalize_or(Vec3::NEG_Y));
    let v = (pole - u * pole.dot(u)).try_normalize().unwrap_or_else(|| u.any_orthonormal_vector());
    let cos = ((a * a + d * d - b * b) / (2.0 * a * d)).clamp(-1.0, 1.0);
    let elbow = s + (u * cos + v * (1.0 - cos * cos).max(0.0).sqrt()) * a;
    let wrist = s + u * d;
    turn_bone(model, pose, arm.upper, Quat::from_rotation_arc((e - s).normalize(), (elbow - s).normalize()));
    let world = model.world(pose);
    let (e1, w1) = (at(&world[arm.fore]), at(&world[arm.hand]));
    turn_bone(model, pose, arm.fore, Quat::from_rotation_arc((w1 - e1).normalize(), (wrist - e1).normalize()));
}

/// A gun at the holster as a hand reaching for it holds it (play.rs builds it for the gun the hand
/// carries or goes for): its place there in the model's space, its mount in the hand (rotation,
/// translation under the hand bone: `HeldWeapon::in_hand`, or `second_hand` for the left) and its
/// grip point in its own frame (`HeldWeapon::grip`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Holstered {
    pub gun: Mat4,
    pub mount: (Quat, Vec3),
    pub grip: Vec3,
}

impl Holstered {
    /// The grip point in the hand bone's frame, where the mount puts it.
    fn in_hand(&self) -> Vec3 {
        self.mount.0 * self.grip + self.mount.1
    }

    /// The hand's turn in the model's space that holds the gun at the holster in its mount.
    fn exact(&self) -> Quat {
        (rot(&self.gun) * self.mount.0.inverse()).normalize()
    }

    /// `k` (0 - 1) of the way from `a` to `b`: the place, the mount and the grip blended (the right
    /// hand's hold at the hip in a swap, from the gun it lets go to the one it takes).
    pub fn blend(a: &Self, b: &Self, k: f32) -> Self {
        let (_, ra, ta) = a.gun.to_scale_rotation_translation();
        let (_, rb, tb) = b.gun.to_scale_rotation_translation();
        Holstered {
            gun: Mat4::from_rotation_translation(ra.slerp(rb, k), ta.lerp(tb, k)),
            mount: (a.mount.0.slerp(b.mount.0, k), a.mount.1.lerp(b.mount.1, k)),
            grip: a.grip.lerp(b.grip, k),
        }
    }
}

/// Poses one arm (`left` or the right) `at` of the way to the hip (`Reach::at`, eased here), for the
/// gun at the holster `holster` (see the module notes): the gun's grip point on its bowed path from
/// where the hand holds it in `pose` to where it is at the holster, the elbow from its own bend toward
/// its pole, the hand from its own turn under the forearm (the clip's wrist) `WRIST_SHARE` of the way
/// to the turn that holds the holstered gun in its mount; the clavicle, and for the right hand going
/// across the body the spine bones `chain` (play.rs `Loaded::arm_chain`), turned toward the hip as
/// the arm needs. The gun's place under the hand bone this frame: its mount turned about the grip
/// point by what the wrist leaves of the turn to the holster's, as far along as the leg is, so at the
/// hip the gun in the hand is exactly where the holster has it. The mount itself at 0.
pub fn pose_arm(model: &Character, pose: &mut [(Quat, Vec3)], arms: &Arms, left: bool, holster: &Holstered, at_: f32, chain: &[usize]) -> (Quat, Vec3) {
    let e = ease(at_);
    if e <= 0.0 {
        return holster.mount;
    }
    let arm = if left { arms.left } else { arms.right };
    // (the turn into a cross draw: the spine above the hips toward the far hip, about the up axis,
    // as far along as the leg is)
    if !left && !chain.is_empty() {
        bf_viewer::bf::locomotion::turn(model, pose, chain, Vec3::Y, TWIST_ACROSS * e);
    }
    let world = model.world(pose);
    let (shoulder, elbow, wrist) = (at(&world[arm.upper]), at(&world[arm.fore]), at(&world[arm.hand]));
    let grip = holster.in_hand();
    let held = wrist + rot(&world[arm.hand]) * grip;
    let grip_to = waypoint(held, holster.gun.transform_point3(holster.grip), e);
    // (where the wrist goes if the hand keeps its turn: for the shrug below)
    let target = grip_to - rot(&world[arm.hand]) * grip;
    // (the clavicle shrugs the shoulder toward a target the straight arm wouldn't reach: the least
    // turn about its joint that brings it within `STRAIGHT` of the arm's length, or of as far as the
    // pose has the wrist already, to the most it may; so none at the start of a leg)
    let length = (elbow - shoulder).length() + (wrist - elbow).length();
    let reach = (length * STRAIGHT).max((wrist - shoulder).length());
    let joint = at(&world[arm.clavicle]);
    if let (true, Some(axis)) = ((target - shoulder).length() > reach, (shoulder - joint).cross(target - joint).try_normalize()) {
        let most = (shoulder - joint).angle_between(target - joint).min(CLAVICLE_REACH);
        let short = |angle: f32| (target - joint - Quat::from_axis_angle(axis, angle) * (shoulder - joint)).length() > reach;
        let (mut lo, mut hi) = (0.0, most);
        for _ in 0..16 {
            let mid = 0.5 * (lo + hi);
            if short(mid) { lo = mid } else { hi = mid }
        }
        turn_bone(model, pose, arm.clavicle, Quat::from_axis_angle(axis, hi));
    }
    let world = model.world(pose);
    let (shoulder, elbow, wrist_now) = (at(&world[arm.upper]), at(&world[arm.fore]), at(&world[arm.hand]));
    // (the elbow's own bend, square to the line from the shoulder to the wrist, toward the pole)
    let line = (wrist_now - shoulder).normalize_or(Vec3::NEG_Y);
    let own = (elbow - shoulder) - line * (elbow - shoulder).dot(line);
    let pole = rot(&world[arms.torso]) * arms.torso_rest.inverse() * if left { POLE_LEFT } else { POLE_RIGHT }.normalize();
    let bend = match own.try_normalize() {
        Some(own) => Quat::IDENTITY.slerp(Quat::from_rotation_arc(own, pole), e) * own,
        None => pole,
    };
    // (the hand's turn and the wrist's place depend on each other: the wrist is where the grip point
    // goes less the turned grip, and the turn starts from the forearm the wrist's place gives; a few
    // passes, each from the arm as it was, settle them, the grip point's offset from the wrist being
    // short)
    let own_turn = pose[arm.hand].0;
    let arm_was = (pose[arm.upper].0, pose[arm.fore].0);
    let mut hand = rot(&world[arm.hand]);
    for _ in 0..PASSES {
        (pose[arm.upper].0, pose[arm.fore].0) = arm_was;
        two_bone(model, pose, arm, grip_to - hand * grip, bend);
        let natural = rot(&model.world(pose)[arm.fore]) * own_turn;
        hand = natural.slerp(holster.exact(), WRIST_SHARE * e).normalize();
    }
    (pose[arm.upper].0, pose[arm.fore].0) = arm_was;
    two_bone(model, pose, arm, grip_to - hand * grip, bend);
    set_world_rot(model, pose, arm.hand, hand);
    // (the rest of the turn to the holster's, in the hand's frame, about the grip point)
    let rest = (hand.inverse() * holster.exact()).normalize();
    let turn = Quat::IDENTITY.slerp(rest, e).normalize();
    ((turn * holster.mount.0).normalize(), turn * (holster.mount.1 - grip) + grip)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A leg takes `LEG_TIME` each way and says when it gets to the hip, once.
    #[test]
    fn a_leg_takes_its_time() {
        let mut r = Reach::default();
        assert!(!r.step(true, LEG_TIME * 0.5));
        assert!((r.at - 0.5).abs() < 1e-6);
        assert!(r.step(true, LEG_TIME * 0.5 + 1e-4), "it gets there");
        assert!(!r.step(true, 0.1), "and is there");
        assert!(r.busy());
        r.step(false, LEG_TIME);
        assert_eq!(r.at, 0.0);
        assert!(!r.busy());
    }

    /// The ease starts and stops at rest and is half way at half time.
    #[test]
    fn the_ease_starts_and_stops_gently() {
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(1.0), 1.0);
        assert!((ease(0.5) - 0.5).abs() < 1e-6);
        assert!(ease(0.05) < 0.01 && ease(0.95) > 0.99);
    }

    /// The left hand goes to the hip while the skill's gun isn't in it, takes it there and comes
    /// back with it; then, the skill off, goes down again, lets go and comes back empty.
    #[test]
    fn the_left_hand_fetches_and_puts_back() {
        let mut r = Reaches::default();
        assert!(!r.step_left(Some(1), LEG_TIME * 0.5));
        assert_eq!(r.left_gun, None, "on the hip until the hand gets there");
        assert!(r.step_left(Some(1), LEG_TIME * 0.5 + 1e-4));
        assert_eq!(r.left_gun, Some(1));
        assert!(!r.step_left(Some(1), LEG_TIME));
        assert_eq!(r.left.at, 0.0, "back up");
        assert!(!r.step_left(None, LEG_TIME * 0.5));
        assert_eq!(r.left_gun, Some(1), "in the hand until it gets to the hip");
        assert_eq!(r.left.gun, Some(1));
        assert!(r.step_left(None, LEG_TIME * 0.5 + 1e-4));
        assert_eq!(r.left_gun, None);
        assert!(!r.step_left(None, LEG_TIME));
        assert!(!r.busy());
    }

    /// The skill switched off before the hand got to the hip: it turns back without the gun.
    #[test]
    fn a_draw_called_off_turns_back_empty() {
        let mut r = Reaches::default();
        r.step_left(Some(0), LEG_TIME * 0.6);
        assert!(!r.step_left(None, LEG_TIME * 0.6));
        assert_eq!(r.left_gun, None);
        assert!(r.left.at < 0.6);
    }

    /// The path starts where the hand holds the grip and ends at the holster's; across the body it
    /// bows out in front, by `BOW` of the width at the middle; down its own side it hardly does.
    #[test]
    fn the_path_bows_in_front_across_the_body() {
        let (from, to) = (Vec3::new(0.4, 0.0, 0.0), Vec3::new(-0.2, 0.0, 0.1));
        assert_eq!(waypoint(from, to, 0.0), from);
        assert!((waypoint(from, to, 1.0) - to).length() < 1e-6);
        let mid = waypoint(from, to, 0.5);
        assert!((mid.z - (0.05 - BOW * 0.6)).abs() < 1e-5, "{mid:?}");
        let side = waypoint(Vec3::new(-0.4, 0.0, 0.0), Vec3::new(-0.42, -0.1, 0.1), 0.5);
        assert!(side.z > 0.04, "{side:?}");
    }

    /// A model of a right arm alone: the root, the torso a metre up, the clavicle 0.4 above that and
    /// 0.05 to the right, the upper arm 0.2 further right at the shoulder, the forearm and the hand
    /// each 0.3 below the bone above (a 0.6 m arm hanging straight down), and its pose.
    fn arm() -> (Character, Arms, Vec<(Quat, Vec3)>) {
        let n = 6;
        let model = Character {
            bones: (0..n as u32).collect(), parent: vec![None, Some(0), Some(1), Some(2), Some(3), Some(4)],
            bind_rows: vec![Mat3::IDENTITY; n], bind_pos: vec![Vec3::ZERO; n], parent_point: vec![Vec3::ZERO; n],
            child_point: vec![Vec3::ZERO; n], inverse_bind: vec![Mat4::IDENTITY; n], geosets: vec![], anims: vec![], face_anims: vec![],
            default_face: None, face: None, hardpoints: Default::default(), part_collision: vec![vec![]; n], part_mass_centre: vec![None; n],
            part_spheres: vec![None; n], bounds: None,
        };
        let bones = Bones { clavicle: 2, upper: 3, fore: 4, hand: 5 };
        let arms = Arms { right: bones, left: bones, torso: 1, torso_rest: Quat::IDENTITY, body: vec![] };
        let pose = [Vec3::ZERO, Vec3::Y, Vec3::new(0.05, 0.4, 0.0), Vec3::new(0.2, 0.0, 0.0), Vec3::new(0.0, -0.3, 0.0), Vec3::new(0.0, -0.3, 0.0)]
            .map(|t| (Quat::IDENTITY, t)).to_vec();
        (model, arms, pose)
    }

    /// A gun at the front of the left hip, within the arm's reach, turned a way of its own, held in
    /// the hand by a mount of its own, its grip 5 cm up its frame.
    fn holster() -> Holstered {
        let grip = Vec3::new(0.0, 0.05, 0.0);
        let turn = Quat::from_rotation_z(0.7) * Quat::from_rotation_x(-0.4);
        let at = Vec3::new(-0.05, 1.0, -0.3) - turn * grip;
        Holstered { gun: Mat4::from_rotation_translation(turn, at), mount: (Quat::from_rotation_y(0.3), Vec3::new(0.0, -0.08, 0.02)), grip }
    }

    /// At the start of a leg the arm is the pose's to the last bone and the gun sits in its mount: no
    /// pop as the reach begins.
    #[test]
    fn a_reach_starts_from_the_pose() {
        let (model, arms, pose) = arm();
        let mut posed = pose.clone();
        let hold = pose_arm(&model, &mut posed, &arms, false, &holster(), 0.0, &[]);
        assert_eq!(posed, pose);
        assert_eq!(hold, holster().mount);
        // (and the first step off it hardly moves anything)
        let mut posed = pose.clone();
        pose_arm(&model, &mut posed, &arms, false, &holster(), 0.02, &[]);
        let (before, after) = (model.world(&pose), model.world(&posed));
        assert!(at(&before[5]).distance(at(&after[5])) < 0.005, "{:?}", at(&after[5]));
    }

    /// At the hip the gun in the hand is exactly where the holster has it: its grip point on the
    /// holster's and its turn the holster's, so it changes hands there without a pop; and the wrist has
    /// turned only `WRIST_SHARE` of the way, the gun in the hand the rest.
    #[test]
    fn at_the_hip_the_gun_in_the_hand_is_the_holsters() {
        let (model, arms, mut pose) = arm();
        let h = holster();
        let (r, t) = pose_arm(&model, &mut pose, &arms, false, &h, 1.0, &[]);
        let hand = model.world(&pose)[5];
        let held = hand * Mat4::from_rotation_translation(r, t);
        assert!(held.transform_point3(h.grip).distance(h.gun.transform_point3(h.grip)) < 1e-3, "{held:?}");
        assert!(rot(&held).angle_between(rot(&h.gun)) < 1e-3);
        assert!(r.angle_between(h.mount.0) > 0.05, "the gun turned in the hand");
        assert!(rot(&hand).angle_between(h.exact()) > 0.05, "the wrist didn't take it all");
    }

    /// From the pose to the hip in hundredths of a leg, no bone of the arm and not the gun turns more
    /// than a few degrees in a step: nothing flips on the way.
    #[test]
    fn a_reach_turns_smoothly() {
        let (model, arms, pose) = arm();
        let h = holster();
        let mut last: Option<(Vec<Mat4>, Quat)> = None;
        for k in 0..=100 {
            let mut posed = pose.clone();
            let (r, _) = pose_arm(&model, &mut posed, &arms, false, &h, k as f32 / 100.0, &[]);
            let world = model.world(&posed);
            let gun = rot(&world[5]) * r;
            if let Some((before, gun_before)) = &last {
                for b in 2..6 {
                    assert!(rot(&before[b]).angle_between(rot(&world[b])) < 0.1, "bone {b} at {k}");
                }
                assert!(gun_before.angle_between(gun) < 0.1, "the gun at {k}");
            }
            last = Some((world, gun));
        }
    }
}
