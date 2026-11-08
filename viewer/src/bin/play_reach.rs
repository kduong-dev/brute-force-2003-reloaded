//! Edward Kingman's hands reaching to his hip holsters as he draws and holsters (#410, **demo-own**:
//! the product owner's request after #409). His motion set ms_kingman has no holster, draw or switch
//! clip (its only weapon overlay is `Sc_w1_drop_pickup_w1`, a ground pickup), so the demo poses the
//! arm itself: a two-bone IK on `Bip01 R|L UpperArm`, `Forearm` and `Hand`, laid over whatever the
//! clips, the aim and the mirrored arms (play_mirror_arm.rs) have made of the pose, so the legs keep
//! walking or running under it.
//!
//! **Each gun belongs to a side** (play_holster.rs, play.rs `HeldWeapon::left`): the first on his
//! right hip and in his right hand, the second on his left hip and in his left hand. A hand only ever
//! reaches to **its own hip**, close by at the side of his waist: no reach crosses his body.
//!
//!  - **Which gun each hand holds** (`Reaches`): a hand goes to its hip while the gun the game has
//!    it hold (the gun in hand if it is that side's and held, or the berserk's second gun if that is)
//!    isn't the one it holds, takes it or lets it go there, and comes back. So the game's own events
//!    move the guns at once, and the hands carry them after: the swap's drop sends the hand in use to
//!    its hip with its gun, the swap's grab the other hand to its hip for the other gun (play_holster.rs
//!    `SWAP_DROP`, `SWAP_GRAB`, `SWAP_TIME`: holster, then draw, 0.75 s); the berserk's second gun is
//!    fetched from its own hip by its own hand and put back there at the stow. A gun is placed and fires
//!    only as the hand holding it has it.
//!  - **The arm** (`pose_arm`): the gun's grip point follows an eased path from where the hand holds
//!    it in the pose to where it is at the holster, bowed out in front of the body by the width it
//!    crosses (`BOW`: little down its own side); the elbow turns from where the pose bends it toward
//!    its pole, out and back (`POLE_RIGHT`, `POLE_LEFT`); the hand turns from its own turn under the
//!    forearm to the turn that holds the holstered gun in its mount, as far along as the leg is, so
//!    the gun stays in its mount in the hand all the way and at the hip is exactly where the holster
//!    has it: the hand-over pops nothing. The clavicle shrugs toward the hip only where the arm alone
//!    falls short (`CLAVICLE_REACH`), which a hip on its own side hardly asks (at most 0.1 degrees in
//!    BF_REACH_LOG, standing, walking and running). At the
//!    start of a leg the IK gives the pose back exactly (the target is where the hand is, the pole the
//!    elbow's own bend, the hand's turn its own), so nothing pops in or out.
//!
//! (Round 1 of #410 had one holster at the front of his waist, which the right hand reached across
//! his body: it shrugged the clavicle, turned the spine toward the far hip and let the gun turn in the
//! palm for half the wrist's turn. Each hand at its own hip needs none of it.)
//!
//! Only the type that has `Holsters` (play_holster.rs: the Berserker on a model with no stow
//! hardpoints, the demo's Kingman class) reaches; every other character's switch and berserk is as
//! it was.

use bevy::prelude::*;
use bf_viewer::bf::character::Character;
use bf_viewer::bf::hash::h;

use super::mirror_arm::MirrorArm;

/// The seconds a hand takes from where its pose has it to the hip, and back. **The demo's choice**
/// (#410): a quarter of a second each way, as #409's glide took; a swap's holster and draw, one after
/// the other, come to `3 LEG_TIME`, 0.75 s (play_holster.rs), against #409's 0.35 s and the squad's
/// pistol switches' 1.1 s (Flint's `Sc_w1_2_w2`).
pub const LEG_TIME: f32 = 0.25;
/// How far the grip's path bows out in front of the body (m) for each metre it goes across it, at
/// the middle of the leg. **The demo's choice** (#410): a path across the front of the body has to
/// clear his coat; one down his own side, as every reach now is, hardly bows.
const BOW: f32 = 0.45;
/// The model's forward direction (play.rs `yaw_of`: a heading of atan2(-x, -z), so forward is -z),
/// which the bow goes along.
const FORWARD: Vec3 = Vec3::NEG_Z;
/// The direction the elbow bends toward at the hip, in the torso's frame as the bind pose has it
/// (+x his right, +y up, -z forward): out and back, the hand coming down to its own hip with the
/// elbow at his side. **The demo's choice** (#410).
const POLE_RIGHT: Vec3 = Vec3::new(0.75, -0.25, 0.6);
const POLE_LEFT: Vec3 = Vec3::new(-0.75, -0.25, 0.6);
/// The most the clavicle turns (rad) to carry the shoulder toward a hip the arm can't reach
/// straight, and the share of the arm's full length the wrist may be from the shoulder before it
/// does. **The demo's choice**: 25 degrees, a shrug's.
const CLAVICLE_REACH: f32 = 0.44;
const STRAIGHT: f32 = 0.97;
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

    /// The clavicle of the right arm or, `left`, the left one: BF_REACH_LOG reads its shrug
    /// (`CLAVICLE_REACH`) from it.
    pub fn clavicle(&self, left: bool) -> usize {
        if left { self.left.clavicle } else { self.right.clavicle }
    }

    /// The elbow (the forearm's joint) of the right arm or, `left`, the left one, in the bones'
    /// matrices `world`.
    pub fn elbow(&self, world: &[Mat4], left: bool) -> Vec3 {
        at(&world[if left { self.left.fore } else { self.right.fore }])
    }

    /// How far the wrist of the right arm or, `left`, the left one is from its shoulder in the bones'
    /// matrices `world`, as a share of the arm's length (the upper arm's and the forearm's): 1 a
    /// straight arm (BF_REACH_LOG).
    pub fn stretch(&self, world: &[Mat4], left: bool) -> f32 {
        let arm = if left { self.left } else { self.right };
        let (s, e, w) = (at(&world[arm.upper]), at(&world[arm.fore]), at(&world[arm.hand]));
        (w - s).length() / ((e - s).length() + (w - e).length()).max(1e-6)
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

/// One hand's reach to its own hip (see the module notes): how far along it is, 0 where its pose has
/// it and 1 at the hip, moving `1 / LEG_TIME` a second, the gun it goes for and the gun it holds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reach {
    pub at: f32,
    pub gun: Option<usize>,
    /// the gun in this hand as the eye sees it: the one it has taken from its hip, until it puts it
    /// back (play.rs places it there, in its mount, and it fires only then)
    pub held: Option<usize>,
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

    /// The hand's step (see the module notes): it goes to its hip while the gun the game has it hold
    /// (`wanted`) isn't the one it holds, and back once it is. At the hip it takes the wanted gun, or
    /// lets go of the one it has, on the step it gets there or, the game changing its mind while it
    /// is there (a switch cut short just as the hand let go), on the next. Whether it took or let go
    /// this step (the guns are to be placed again).
    pub fn step_to(&mut self, wanted: Option<usize>, dt: f32) -> bool {
        let out = wanted != self.held;
        if out {
            self.gun = wanted.or(self.held);
        }
        self.step(out, dt);
        if out && self.at >= 1.0 {
            self.held = wanted;
            return true;
        }
        false
    }
}

/// Both hands' reaches (`Player::reach`).
#[derive(Clone, Debug, Default)]
pub struct Reaches {
    pub right: Reach,
    pub left: Reach,
}

impl Reaches {
    /// The left hand's reach, or the right one's.
    pub fn hand(&self, left: bool) -> &Reach {
        if left { &self.left } else { &self.right }
    }

    /// The gun in the left hand, or the right, as the eye sees it (`Reach::held`).
    pub fn held(&self, left: bool) -> Option<usize> {
        self.hand(left).held
    }

    /// Whether either hand is on its way to or from its hip.
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
/// translation under the hand bone of its own side: `HeldWeapon::in_hand`) and its grip point in its
/// own frame (`HeldWeapon::grip`).
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
}

/// Poses one arm (`left` or the right) `at` of the way to its hip (`Reach::at`, eased here), for the
/// gun at the holster `holster` (see the module notes): the gun's grip point on its bowed path from
/// where the hand holds it in `pose` to where it is at the holster, the elbow from its own bend toward
/// its pole, the hand from its own turn under the forearm (the clip's wrist) to the turn that holds the
/// holstered gun in its mount, as far along as the leg is, the clavicle turned toward the hip if the
/// arm alone falls short. A gun in the hand stays in its mount, so at the hip it is exactly where the
/// holster has it.
pub fn pose_arm(model: &Character, pose: &mut [(Quat, Vec3)], arms: &Arms, left: bool, holster: &Holstered, at_: f32) {
    let e = ease(at_);
    if e <= 0.0 {
        return;
    }
    let arm = if left { arms.left } else { arms.right };
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
        hand = natural.slerp(holster.exact(), e).normalize();
    }
    (pose[arm.upper].0, pose[arm.fore].0) = arm_was;
    two_bone(model, pose, arm, grip_to - hand * grip, bend);
    set_world_rot(model, pose, arm.hand, hand);
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

    /// A hand goes to its hip while the game's gun for it isn't in it, takes it there and comes back
    /// with it; then, the gun no longer its, goes down again, lets go and comes back empty.
    #[test]
    fn a_hand_fetches_and_puts_back() {
        let mut r = Reaches::default();
        assert!(!r.left.step_to(Some(1), LEG_TIME * 0.5));
        assert_eq!(r.held(true), None, "on the hip until the hand gets there");
        assert!(r.left.step_to(Some(1), LEG_TIME * 0.5 + 1e-4));
        assert_eq!(r.held(true), Some(1));
        assert!(!r.left.step_to(Some(1), LEG_TIME));
        assert_eq!(r.left.at, 0.0, "back up");
        assert!(!r.left.step_to(None, LEG_TIME * 0.5));
        assert_eq!(r.held(true), Some(1), "in the hand until it gets to the hip");
        assert_eq!(r.left.gun, Some(1));
        assert!(r.left.step_to(None, LEG_TIME * 0.5 + 1e-4));
        assert_eq!(r.held(true), None);
        assert!(!r.left.step_to(None, LEG_TIME));
        assert!(!r.busy());
        // (the other hand is its own)
        assert_eq!(r.held(false), None);
        assert!(!r.right.busy());
    }

    /// The game changing its mind just as the hand gets to the hip (a switch cut short as it let go,
    /// play.rs `Player::cancel_switch`): the hand takes the gun back there on its next step instead of
    /// staying at the hip, and comes back up with it.
    #[test]
    fn a_hand_at_the_hip_takes_back_what_it_let_go() {
        let mut r = Reach { held: Some(0), ..Default::default() };
        assert!(!r.step_to(None, LEG_TIME * 0.5));
        assert!(r.step_to(None, LEG_TIME * 0.5 + 1e-4));
        assert_eq!((r.held, r.at), (None, 1.0));
        assert!(r.step_to(Some(0), 1.0 / 60.0), "takes it back at once");
        assert_eq!(r.held, Some(0));
        assert!(!r.step_to(Some(0), LEG_TIME));
        assert!(!r.busy());
    }

    /// A draw called off before the hand got to the hip: it turns back without the gun.
    #[test]
    fn a_draw_called_off_turns_back_empty() {
        let mut r = Reaches::default();
        r.right.step_to(Some(0), LEG_TIME * 0.6);
        assert!(!r.right.step_to(None, LEG_TIME * 0.6));
        assert_eq!(r.held(false), None);
        assert!(r.right.at < 0.6);
    }

    /// A swap side to side, as the switch's events drive the hands (play_holster.rs `SWAP_DROP`,
    /// `SWAP_GRAB`, `SWAP_TIME`): the right hand, its gun let go at once, holsters it at its hip by
    /// `LEG_TIME`; the left gun, taken then, is in the left hand at `2 LEG_TIME` and the hand back up at
    /// `3 LEG_TIME`, the swap's end; the two hands are never at their hips at once with a gun to hand
    /// over.
    #[test]
    fn a_swap_holsters_on_one_side_and_draws_on_the_other() {
        let mut r = Reaches { right: Reach { held: Some(0), ..Default::default() }, ..Default::default() };
        let dt = 1.0 / 60.0;
        let (mut put_away, mut taken) = (None, None);
        for k in 1..=60 {
            let t = k as f32 * dt;
            // (the game's: the gun let go at the start, the other taken at SWAP_GRAB)
            let (right, left) = if t < LEG_TIME { (None, None) } else { (None, Some(1)) };
            if r.right.step_to(right, dt) && put_away.is_none() {
                put_away = Some(t);
            }
            if r.left.step_to(left, dt) && taken.is_none() {
                taken = Some(t);
            }
        }
        let (put_away, taken) = (put_away.unwrap(), taken.unwrap());
        assert!((put_away - LEG_TIME).abs() < 2.0 * dt, "{put_away}");
        assert!((taken - 2.0 * LEG_TIME).abs() < 2.0 * dt, "{taken}");
        assert_eq!((r.held(false), r.held(true)), (None, Some(1)));
        assert!(!r.busy(), "both back by 3 LEG_TIME (1 s here)");
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

    /// A gun at a hip within the arm's reach, turned a way of its own, held in the hand by a mount of
    /// its own, its grip 5 cm up its frame.
    fn holster() -> Holstered {
        let grip = Vec3::new(0.0, 0.05, 0.0);
        let turn = Quat::from_rotation_z(0.7) * Quat::from_rotation_x(-0.4);
        let at = Vec3::new(-0.05, 1.0, -0.3) - turn * grip;
        Holstered { gun: Mat4::from_rotation_translation(turn, at), mount: (Quat::from_rotation_y(0.3), Vec3::new(0.0, -0.08, 0.02)), grip }
    }

    /// At the start of a leg the arm is the pose's to the last bone: no pop as the reach begins.
    #[test]
    fn a_reach_starts_from_the_pose() {
        let (model, arms, pose) = arm();
        let mut posed = pose.clone();
        pose_arm(&model, &mut posed, &arms, false, &holster(), 0.0);
        assert_eq!(posed, pose);
        // (and the first step off it hardly moves anything)
        let mut posed = pose.clone();
        pose_arm(&model, &mut posed, &arms, false, &holster(), 0.02);
        let (before, after) = (model.world(&pose), model.world(&posed));
        assert!(at(&before[5]).distance(at(&after[5])) < 0.005, "{:?}", at(&after[5]));
    }

    /// At the hip the gun in the hand, in its mount, is exactly where the holster has it: its grip
    /// point on the holster's and its turn the holster's, so it changes hands there without a pop.
    #[test]
    fn at_the_hip_the_gun_in_the_hand_is_the_holsters() {
        let (model, arms, mut pose) = arm();
        let h = holster();
        pose_arm(&model, &mut pose, &arms, false, &h, 1.0);
        let hand = model.world(&pose)[5];
        let held = hand * Mat4::from_rotation_translation(h.mount.0, h.mount.1);
        assert!(held.transform_point3(h.grip).distance(h.gun.transform_point3(h.grip)) < 1e-3, "{held:?}");
        assert!(rot(&held).angle_between(rot(&h.gun)) < 1e-3);
    }

    /// From the pose to the hip in hundredths of a leg, no bone of the arm and not the gun in the hand
    /// turns more than a few degrees in a step: nothing flips on the way.
    #[test]
    fn a_reach_turns_smoothly() {
        let (model, arms, pose) = arm();
        let h = holster();
        let mut last: Option<(Vec<Mat4>, Quat)> = None;
        for k in 0..=100 {
            let mut posed = pose.clone();
            pose_arm(&model, &mut posed, &arms, false, &h, k as f32 / 100.0);
            let world = model.world(&posed);
            let gun = rot(&world[5]) * h.mount.0;
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
