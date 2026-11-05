//! The AI's steering through its avoidance (#344; viewer/docs/nav.md "The steering"). FUN_000255d0
//! makes the direction of every walking goal from its steering target in four stages and takes the
//! first that steers: the direction to the target only when nothing is in the way, and a stage
//! that steers replaces it (no blend). The reading is todo/344-ai-avoidance/leads.md §1-§5, checked
//! against the asm; every function is in viewer/docs/code-map.md ("AI").
//!
//!  1. **The feelers** (FUN_00018c40, `feel`): three rays along the character's heading from its
//!     hips, a centre one and two at 25 degrees either side, as long as its pace allows. A side ray
//!     that meets something turns it square to its heading, away from the hit; a wall ahead sends
//!     it along the wall to the roomier side, held to the side its target lies on. A hit at a dash
//!     drops the pace to a run instead, or first takes the follow's speed boost off. They meet the
//!     terrain where it's too steep to walk, the placed objects, the blockers and the hinters,
//!     never a character: what keeps the game's members a metre off e13's hinter bands.
//!  2. **The push** (the sensor's +0x3c, 0x521c0, `push`): away from every character within 1.5 m
//!     ahead of it (or 0.5 m anywhere), by 1 over the distance.
//!  3. **The side-step** (the sensor's +0x4c, 0x52750, `side_step`): a step across, away from the
//!     nearest obstacle disc in its path. Which objects the game's two lists hold isn't traced, so
//!     the demo has none to give it (`obstacles`).
//!  4. **The plain direction** (the sensor's +0x24, 0x519f0, `plain`): to the target.
//!
//! The direction goes to the move (FUN_0004d970, play_routes.rs `pace`, which the feelers' drops
//! hold) and the AI update's steering FUN_00019ed0, which turns the character's view toward it at
//! 200 degrees a second (`view_turn`) and whose FUN_00019af0 turns the move toward it at most 200
//! degrees a second (`move_turn`). The move is the character's stick (char+0x5e8, the commanded
//! move, never the speed it makes), and its heading (iface +0x34: the feelers', the push's and the
//! next turn's) is that command (`heading`, #363). The character update then picks the motion from
//! the stick in its own view frame (FUN_001195c0, `pick`): it runs facing its move, and steps
//! sideways or backs off only while its view lags its move (play.rs `step_player`). The callers are
//! the demo's route walkers, the follow (play.rs `squad_ai`: the squad's, the tutorial marines',
//! and the test map's straight walk) and Move To (play_orders.rs `move_out`); the enemies' goals
//! walk no routes yet (#336) and aren't steered.
//!
//! Test hooks: BF_AVOID_LOG=1 prints each frame a stage before the plain direction steers (which
//! feelers met what and how far, the response, the pace drops, the push and from whom);
//! BF_AVOID_OFF=1 (a diagnostic, not the game) steers by the plain direction alone, as before #344;
//! BF_AVOID_FREE_TURN=1 (a diagnostic, not the game) leaves the move's turn to the facing's 9 rad/s,
//! as before #344's fix round. BF_GAIT_LOG=1 (play.rs `gait_log`) prints each frame every route
//! walker's motion: its gait, its view, body and move, the pick's case, its step and its clip
//! layers.

use super::*;

/// FUN_000255d0 runs only for a target off the world's origin: a squared length under this
/// (0x3a4fe4) is none, and the direction is left as it was (zero).
const NO_TARGET: f32 = 0.001;

/// The brain's feelers (brain+0x354: 0x14-byte entries {length, angle, axis, enabled, 1.0} the
/// brain's constructor fills at 0x16ee0-0x16f85, and nothing changes): (length m, yaw degrees about
/// the heading frame's up). The centre 3.15 m (0x40499999), the left and right 2.0475 m
/// (0x40030a3d) at +25 and -25 degrees (FUN_000255d0's 25.0; a positive yaw turns the heading toward
/// its left).
const FEELERS: [(f32, f32); 3] = [(3.15, 0.0), (2.0475, 25.0), (2.0475, -25.0)];
/// The feelers' origin: the character's position (iface +0xc -> +0x24.., the hips) this far lower
/// (0x3a5028)...
const FEELER_DROP: f32 = 0.25;
/// ...and moved this far back along the heading (0x18e66-0x18ee5: half the frame's third column,
/// which FUN_0001d0c0 makes the heading's reverse).
const FEELER_BACK: f32 = 0.5;
/// The lengths at a walk (the pace level 2, 0x3a5038) and at a dash (4, 0x3a5088); a run's are
/// the list's.
const FEELER_WALK: f32 = 0.6;
const FEELER_DASH: f32 = 1.6;
/// With the speed factor char+0x720 over 1 the lengths are also x this (0x3a50b8) x the factor:
/// x1.125 for the follow's boost (1.25); at 1 or under, x1.
const FEELER_FACTOR: f32 = 0.9;
/// A ray this long or shorter (0x3a4f98) isn't cast.
const FEELER_MIN: f32 = 0.01;
/// A terrain hit whose normal's y is over this (0x3a5154; 0x19226-0x19244) is walkable ground
/// under the tilted ray, not a block. An object's or a blocker's always is.
const WALKABLE: f32 = 0.7;
/// A wall ahead with both sides free: two more rays along the side rays, this many times their
/// length (0x3a4fd0).
const LONG: f32 = 4.0;
/// A feeler hit at a dash (or at a run inside a prism) sets the pace level down a notch with its
/// timer at this (s, 0x3fc00000), or with the speed factor over 1 the speed record's timer
/// (brain+0x8a4) instead, which keeps its factor off char+0x720 that long (FUN_00048270).
pub const DASH_DROP: f32 = 1.5;
/// The hysteresis at a wall ahead: the side picked away from the target's at a run or a dash, the
/// pace level 2 with its timer at this (s, 0x40000000) and the target's side taken.
const TURN_WALK: f32 = 2.0;
/// FUN_00018b50: no side for a heading whose square is this or under (1.19e-7)...
const SIDE_HEADING: f32 = 1.192_092_9e-7;
/// ...or a target whose squared distance across is under this (2e-5, 4.5 mm).
const SIDE_NEAR: f32 = 2e-5;
/// A steer is normalised when its square is over this (0x3a50cc).
const STEER_MIN: f32 = 0.0001;
/// FUN_00115b00's ground probes: this far (m, 0x3ecccccd) ahead along the move and twice and three
/// times that, each the first ground within GROUND_BELOW (0x40a00000) under the hips; the third only
/// when the second's height change is over GROUND_JUMP (2.75). The feelers drop a change over
/// GROUND_GARBAGE (100, 0x18f1a).
const GROUND_PROBE: f32 = 0.4;
const GROUND_BELOW: f32 = 5.0;
const GROUND_JUMP: f32 = 2.75;
const GROUND_GARBAGE: f32 = 100.0;
/// The character's heading (iface +0x34, 0x11e860) is its commanded move (char+0x5e8 put back in
/// the world, FUN_0011e680) when its flat square is at least this (0x3a519c), else the body's
/// forward (`heading`). FUN_0011e680's own zero test (1.42e-14) is smaller still: the least move the
/// AI commands is a stand's (MOVE_STAND, 1e-4 squared), which passes both.
const MOVE_NONE: f32 = 1e-10;
/// FUN_00115b00's ground probes run along the velocity (char+0x470, normalised) unless its square
/// is under this (0.0001, its row: the out left unwritten, so no tilt).
const PROBE_STILL: f32 = 0.0001;
/// Stage 2: the characters within this (m, across: brain+0x3bc, set 1.5 by the brain's
/// constructor), within PUSH_ANGLE of the heading (brain+0x3c0, 90 degrees: the half in front)...
const PUSH_RADIUS: f32 = 1.5;
/// ...or within this anywhere (0.25, 0x3a5028, its square).
const PUSH_NEAR: f32 = 0.5;
/// ...and overlapping in height (FUN_00052120 with a 1.0 m margin, the two characters' extents by
/// iface +0x20): the demo takes the two bodies' spans (BODY_HEIGHT each, from the feet) grown by
/// the margin, a guess on the extents (code map: medium).
const PUSH_MARGIN: f32 = 1.0;
/// The push's distances under this (squared, 0x3a519c) are left out.
const PUSH_MIN: f32 = 1e-10;
/// Stage 3 (0x52750): an obstacle within its radius + this (m, FUN_000255d0's third argument 2.0)
/// of the character (3D)...
const STEP_REACH: f32 = 2.0;
/// ...at most this big (m, 0x3a500c; the first list's)...
const STEP_MAX_RADIUS: f32 = 15.0;
/// ...overlapping in height (FUN_00052120 with 0.7, 0x3f333333: as the push's, the first list's)...
const STEP_MARGIN: f32 = 0.7;
/// ...and in front: the unit offset along the heading over this (0x3a4ff0) for the first list's,
/// over 0 for the second's.
const STEP_AHEAD: f32 = 0.1;
/// The character's own disc for the side-step (the sensor's +0x20, 0x52c90, on its own body: type
/// kind 3, a character, 0.5 m at its node, 0x52cea).
const SELF_RADIUS: f32 = 0.5;
/// FUN_00019af0, the move's turn (`move_turn`): at most this fast (rad/s, 0x405f66f3: 200 degrees
/// a second) the move turns toward the steering direction...
const MOVE_TURN_MAX: f32 = 3.490_658_5;
/// ...its rate eased toward the one wanted, (k x the last + the wanted) / (k + 1) a game frame, k
/// this at a run or a dash (0x3a5008)...
const MOVE_TURN_EASE_RUN: f32 = 2.0;
/// ...and this at a walk, the direction's size 0.51 or under (0x3a5084; 0x3a4fd0)...
const MOVE_TURN_EASE_WALK: f32 = 4.0;
const MOVE_TURN_SLOW: f32 = 0.51;
/// ...no turn within a degree (57.2958 x the angle against 1.0, 0x19c5e-0x19c79)...
const MOVE_TURN_LEAST: f32 = 1.0;
/// ...and the steering direction taken as it is when its size is under 0.1 (0x3a4ff0, a stand's
/// 0.01) or when the last frame's was, its square under 0.1 (FUN_000162c0 on brain+0x1cc: the AI
/// update keeps each frame's after the steering, 0x1b3fb).
const MOVE_TURN_STILL: f32 = 0.1;
/// FUN_0004d970's size of the direction by the level the AI moves at: a stand's x0.01, a walk's
/// x0.5, a run's or a dash's x1 (FUN_00019af0 reads it as the stick's tilt).
const MOVE_STAND: f32 = 0.01;
const MOVE_WALK: f32 = 0.5;
/// FUN_00019ed0, the view's turn (`view_turn`): at most this fast (rad/s), the type's turn rate
/// (type+0x2d0) x char+0x7ec, which the brain's set-up makes 200 / (the rate x 57.2958) for a type
/// with the squad bit (0x17699-0x176ab, 0x3a52dc = 200): 200 degrees a second whatever the type.
/// (A type without the bit gets 1.8 x its rate, 0x17737: since #364 the test map's NPCs set to
/// follow can be of such a type, a Shrike soldier, and the demo gives them these 200 degrees too:
/// not modelled, `CharacterType::turn_rate` isn't passed to `view_turn`.)
const VIEW_TURN_MAX: f32 = 3.490_658_5;
/// ...no turn within a degree (0x3a4fe0 = 0.0174533 rad against the angle, 0x1a009), nor for a
/// direction whose square is under 1e-16 (0x3a52b0, 0x19f7e)...
const VIEW_TURN_LEAST: f32 = 0.017_453_3;
/// ...and a turn the frame can finish taken at the fraction it needs, the kept one first set to
/// this share of it (x 0x3a4f74 = 0.5 at 0x1a086), so a turn's last degrees ease out.
const VIEW_TURN_CLOSE: f32 = 0.5;
/// FUN_001195c0's forward test (`pick`): the move's cosine against the view's forward at least this
/// (0x3ecccccd) takes the forward or the backward case by its sign...
const PICK_STRAIGHT: f32 = 0.4;
/// ...or this with the gun up (char+0x7dc over 0, FUN_000d4440)...
const PICK_STRAIGHT_RAISED: f32 = 0.25;
/// ...and the body this close to its target yaw (char+0x4a0) is on its move: under it a body more
/// than 45 degrees off the view steps sideways (0x46d314 = 0.5236, 30 degrees, a static init at
/// 0x30bdd0).
const PICK_ON_MOVE: f32 = 0.523_6;

/// Which of FUN_000255d0's stages made the direction (its return value 1, 2, 4 or 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Feelers,
    Push,
    SideStep,
    Plain,
}

/// The brain's avoidance fields (kept with the character's route state, `Router::avoid`).
#[derive(Clone, Debug)]
pub struct Avoid {
    /// brain+0x924: the feelers are off while it's over 0 (s); the walk's door step sets it to 1.5
    /// (FUN_00026820, play_routes.rs `door_step`, #346), the AI update counts it down
    /// (0x1b2d7-0x1b306), the brain's reset zeroes it (0x17610)
    pub feelers_off: f32,
    /// brain+0x8a4: the speed record's timer (s): while it runs FUN_00048270 hands char+0x720 1.0,
    /// not the record's factor (play_speed.rs `record_boost`); a feeler hit at a dash with the
    /// factor over 1 sets it to DASH_DROP (FUN_00018c40, 0x194cb)
    pub boost_off: f32,
    /// char+0x720 as the last AI frame left it (`Player::speed_boost`, the follow's 1.25 or 1.0):
    /// the feelers' length factor and their choice between the two drops
    pub factor: f32,
    /// the other characters near it this frame (their positions, play_routes.rs `near`): the
    /// push's (stage 2) look through the world's character lists
    pub near: Vec<Vec3>,
    /// brain+0x35c: how long it has turned hard at a run or a dash (FUN_0001ad50, play_routes.rs
    /// `hard_turn`), and its facing at the last AI frame (the turn rate's stand-in)
    pub turning: f32,
    pub yaw_was: Option<f32>,
    /// brain+0x1c4: the move's turn rate (rad/s, `move_turn`, FUN_00019af0)
    pub turn_rate: f32,
    /// brain+0x1cc..: the steering direction the last AI frame ended with, times its size (a
    /// stand's 0.01, a walk's 0.5, else 1; zero when no goal moved it), and this frame's
    pub dir_was: Vec2,
    pub dir_now: Vec2,
    /// char+0x5e8 put back in the world (FUN_0011e680): the move the last AI frame commanded
    /// (FUN_00019af0's, `move_turn`, times its size; zero when no goal moved it), which this
    /// frame's heading is (`heading`, iface +0x34), and this frame's (#363)
    pub moved_was: Vec2,
    pub moved: Vec2,
    /// its view frame (char+0x5bc) and its motion pick while a route walker steers it (`View`);
    /// None standing, where the demo's view is its facing (#363)
    pub view: Option<View>,
}

impl Default for Avoid {
    /// A new brain's: the feelers on, the speed record's timer run out, char+0x720 1.0 (the
    /// movement reset FUN_0012b560), no hard turn counted, no turn, no direction, no move, no view
    /// of its own.
    fn default() -> Self {
        Avoid { feelers_off: 0.0, boost_off: 0.0, factor: 1.0, near: vec![], turning: 0.0, yaw_was: None, turn_rate: 0.0, dir_was: Vec2::ZERO,
                dir_now: Vec2::ZERO, moved_was: Vec2::ZERO, moved: Vec2::ZERO, view: None }
    }
}

impl Avoid {
    /// The AI update's timers for a frame of `dt` s: brain+0x924 counted down (0x1b2d7-0x1b306,
    /// before the goal) and the speed record's brain+0x8a4 (FUN_00048270 at 0x1b3cd, after the goal:
    /// here a frame sooner); and the direction and the move the last frame ended with kept
    /// (brain+0x1cc, 0x1b3fb; char+0x5e8), this frame's zero until a goal moves it (the buffer
    /// zeroed before the goal machine, 0x1b338-0x1b350, and FUN_00019af0 writing the move as the
    /// direction it has: none with none).
    pub fn tick(&mut self, dt: f32) {
        self.feelers_off = (self.feelers_off - dt).max(0.0);
        self.boost_off = (self.boost_off - dt).max(0.0);
        self.dir_was = std::mem::take(&mut self.dir_now);
        self.moved_was = std::mem::take(&mut self.moved);
        // (a view its steering didn't turn last frame is gone: a walk after it starts from its
        // facing, `view_turn`; one it did waits for this frame's turn)
        self.view = self.view.filter(|v| v.turned).map(|v| View { turned: false, ..v });
    }

    /// A frame with no move of the AI's (a hold, the liquids' guard): no direction or move kept for
    /// the next frame's turn, and no view of its own (the demo's stand: its body faces its own way).
    pub fn stand(&mut self) {
        self.dir_now = Vec2::ZERO;
        self.moved = Vec2::ZERO;
        self.view = None;
    }
}

/// The view frame of a character the AI steers and its motion pick (#363): what FUN_00019ed0 turns
/// and FUN_001195c0 reads, kept with its steering (`Avoid::view`) as only the route walkers' steering
/// turns them in the demo.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct View {
    /// the view's yaw (char+0x5bc, R: its forward is the stick's -z; the demo's yaw, 0 down -z)
    pub yaw: f32,
    /// brain+0x1c8: the view turn's eased fraction of VIEW_TURN_MAX (`view_turn`)
    pub rate: f32,
    /// char+0x800 = 0: the last pick's backward case (`pick`), where FUN_0012cb40 takes another 0.8
    /// off the character's speed (play_speed.rs `targets`)
    pub backing: bool,
    /// turned this AI frame (`view_turn`): the step picks its motion against it only then
    /// (play.rs `step_player`), and the next frame's `Avoid::tick` drops a view left unturned
    pub turned: bool,
}

/// FUN_0004d970's size of the move direction at the pace level `level` the AI moves at (the lower
/// of its level and the mode asked): MOVE_STAND, MOVE_WALK or 1.
pub fn move_size(level: i32) -> f32 {
    match level {
        ..=1 => MOVE_STAND,
        2 => MOVE_WALK,
        _ => 1.0,
    }
}

/// FUN_00019af0 (from the AI update's steering FUN_00019ed0, for a brain without FOLLOW_HOLD_BACK's
/// route at brain+0x684: the follow's and Move To's) for `m`, a frame of `dt` s: the move this
/// frame, turned from its heading (iface +0x34, `heading`) toward the steering direction `dir`
/// (unit, xz; FUN_000255d0's, `steer`), of size `size` (`move_size`):
///  - under MOVE_TURN_STILL in size, or after a frame that ended with none (FUN_000162c0), the
///    direction as it is; within MOVE_TURN_LEAST degrees of the heading, the heading (the rate 0);
///  - else the heading turned by the rate x `dt`, the rate eased toward the one that would close
///    the angle this frame, at most MOVE_TURN_MAX (FUN_00019ab0): (k x the last + it) / (k + 1)
///    (FUN_00019a70; MOVE_TURN_EASE_RUN or _WALK), the last taken as 0 when it turned the other way
///    (FUN_00019a20), and never past the wanted one; kept for the next frame (brain+0x1c4).
///
/// The game eases a game frame at a time (GAME_FRAME, 30 Hz); the demo eases by the frame's share
/// of them, so the turn doesn't hang on its frame rate. The direction (times `size`) is kept for
/// the next frame's test (`Avoid::dir_now`), and the move (times `size`: the stick, char+0x5e8) for
/// the next frame's heading (`Avoid::moved`, #363). The demo's character moves along its facing,
/// which turns toward this at 9 rad/s, faster than MOVE_TURN_MAX: so its facing follows the move.
pub fn move_turn(m: &mut Player, dir: Vec2, size: f32, dt: f32) -> Vec2 {
    let heading = heading(m);
    let r = &mut m.router.avoid;
    r.dir_now = dir * size;
    let out = turned(r, heading, dir, size, dt);
    r.moved = out * size;
    out
}

/// FUN_00019af0's turn of the move from `heading` toward `dir` (`move_turn`), its rate kept in `r`.
fn turned(r: &mut Avoid, heading: Vec2, dir: Vec2, size: f32, dt: f32) -> Vec2 {
    if size < MOVE_TURN_STILL || dt <= 0.0 || free_turn() {
        r.turn_rate = 0.0;
        return dir;
    }
    let angle = wrap_angle(dir.x.atan2(dir.y) - heading.x.atan2(heading.y));
    if angle.abs().to_degrees() <= MOVE_TURN_LEAST {
        r.turn_rate = 0.0;
        return heading;
    }
    if r.dir_was.length_squared() < MOVE_TURN_STILL {
        return dir;
    }
    let wanted = (angle / dt).clamp(-MOVE_TURN_MAX, MOVE_TURN_MAX);
    let last = if wanted * r.turn_rate < 0.0 { 0.0 } else { r.turn_rate };
    let k = if size > MOVE_TURN_SLOW { MOVE_TURN_EASE_RUN } else { MOVE_TURN_EASE_WALK };
    let keep = (k / (k + 1.0)).powf(dt / GAME_FRAME);
    let mut rate = wanted + (last - wanted) * keep;
    if rate.abs() > wanted.abs() {
        rate = wanted;
    }
    r.turn_rate = rate;
    let turned = heading.x.atan2(heading.y) + rate * dt;
    Vec2::new(turned.sin(), turned.cos())
}

/// FUN_00019ed0's turn of `m`'s view (`View::yaw`, the frame R = char+0x5bc its motion is picked in:
/// `pick`) toward the steering direction `dir` (unit, xz; FUN_000255d0's, `steer`) of size `size`
/// (`move_size`), a frame of `dt` s (#363):
///  - a walk that starts from a stand starts from the way it faces (the demo's: in the game a
///    standing character's body follows its view, FUN_00119fa0, where the demo's stands face their
///    own way, play.rs `step_player`);
///  - no turn for no direction, nor within VIEW_TURN_LEAST of it (the kept fraction 0);
///  - else the fraction of VIEW_TURN_MAX wanted: all of it (+-1), or for a turn the frame can
///    finish the share that finishes it, the kept fraction first set to VIEW_TURN_CLOSE of that;
///    the kept one taken as 0 when it turned the other way (FUN_00019a20) and eased toward the
///    wanted (FUN_00019a70, k as the move's, by the frame's share of the game's frames as
///    `move_turn`); the view turned by it (FUN_0011e5e0 -> char+0x5f8, which FUN_00119fa0 applies x
///    char+0x7ec) and the fraction kept (brain+0x1c8).
///
/// Its target is the steering direction, never a target to fire at: the one facing override
/// (brain+0x488) is the follow's look-point near a cover point (FUN_00022790, not done), and no
/// fight goal sets it. So the view turns to the move, and the gun finds its target with the upper
/// body (play.rs `step_player`'s aim).
pub fn view_turn(m: &mut Player, dir: Vec2, size: f32, dt: f32) {
    let facing = m.yaw;
    let v = m.router.avoid.view.get_or_insert(View { yaw: facing, ..Default::default() });
    v.turned = true;
    let angle = wrap_angle(yaw_of(dir) - v.yaw);
    if dir == Vec2::ZERO || dt <= 0.0 || angle.abs() <= VIEW_TURN_LEAST {
        v.rate = 0.0;
        return;
    }
    let most = VIEW_TURN_MAX * dt;
    let wanted = if angle.abs() < most {
        v.rate = angle / most * VIEW_TURN_CLOSE;
        angle / most
    } else {
        angle.signum()
    };
    let last = if wanted * v.rate < 0.0 { 0.0 } else { v.rate };
    let k = if size > MOVE_TURN_SLOW { MOVE_TURN_EASE_RUN } else { MOVE_TURN_EASE_WALK };
    let keep = (k / (k + 1.0)).powf(dt / GAME_FRAME);
    v.rate = wanted + (last - wanted) * keep;
    v.yaw = wrap_angle(v.yaw + v.rate * most);
}

/// The demo's yaw (0 down -z, positive to the left) of a direction across (x, z).
pub fn yaw_of(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

/// What FUN_001195c0 commands for a character the AI drives while its stick moves it (`pick`), the
/// CT_ commands its handler 0x122040 plays (#363).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// CT_RUN (3): forward along the move, MT_RUN, MT_WALK or MT_DASH by the pace
    Run,
    /// CT_MOVE_LEFT (6) / CT_MOVE_RIGHT (7): the body on its move, the view more than 45 degrees
    /// off it to the right (`left`: the move to the view's left) or the left; MT 26 / 30 at a run,
    /// the _SLOW 29 / 33 at a walk
    Side { left: bool },
    /// CT_MOVE_BACK (0xc): backing along the move, the body facing away from it within 45 degrees
    /// of the view; MT_BACKPEDAL (3) at a run, MT 4 at a walk
    Back,
    /// CT_MOVE_LEFT_BACK (0xd) / CT_MOVE_RIGHT_BACK (0xe): backing off, the body more than 45
    /// degrees off the view; MT 27 / 31 at a run, 28 / 32 at a walk
    SideBack { left: bool },
    /// no command: the motion it has goes on (backing off, the body still 30 degrees or more off
    /// the move's reverse while more than 45 degrees off the view)
    Keep,
}

/// FUN_001195c0's pick (`pick`): the motion, the case (char+0x800: true forward, false backward) and
/// the body's target yaw (char+0x4a0: the move's, or its reverse backward; the demo's yaw).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pick {
    pub motion: Motion,
    pub forward: bool,
    pub body: f32,
}

/// FUN_001195c0 (0x11987f-0x119eea) for a character whose stick moves it along the yaw `moving`
/// (the commanded move in the world), its view at the yaw `view` (R, `View::yaw`) and its body at
/// `body` (char+0x434), its gun up (`raised`: the ready timer char+0x7dc over 0) or not, dashing
/// (`dashing`: the motion MT_DASH) or not (#363):
///  - the case: the move's cosine against the view's forward at least PICK_STRAIGHT
///    (PICK_STRAIGHT_RAISED with the gun up), or a dash: forward when it's 0 or more (or dashing),
///    else backward; under it (the move mostly across the view) forward when the move is in the
///    body's front half, a hysteresis on the body;
///  - the body's target: the move's yaw, forward; its reverse, backward;
///  - the command, from the body against the view and the turn the body still has to make
///    (`PICK_ON_MOVE`): forward, the body within 45 degrees of the view, or still 30 or more off its
///    move, runs; else (on its move, the view more than 45 degrees off) it steps sideways, to the
///    side of the view its body is on. Backward, within 45 degrees of the view it backs off; else
///    still turning, no command; else it backs off sideways.
///
/// The backward case has exceptions the demo can't reach (0x119a9c-0x119b4e): in the movement
/// mode 4, with a jump motion, or with a dash coming next, the body faces the move itself (not its
/// reverse) while the case flag char+0x800 stays 0. A route walker is never in those (it has no
/// mode 4, jump or dash asked of it while it backs off), so `pick` always reverses the body here.
///
/// So on a straight its body, its view and its move agree and it runs facing its move; it steps
/// sideways only in the moments of a sharp turn its view lags, and backs off only on a reversal,
/// until its view has turned to within 66 degrees of its move. No other character's view or
/// camera goes into it.
pub fn pick(view: f32, body: f32, moving: f32, raised: bool, dashing: bool) -> Pick {
    let ahead = wrap_angle(moving - view).cos();
    let least = if raised { PICK_STRAIGHT_RAISED } else { PICK_STRAIGHT };
    let forward = if ahead.abs() >= least || dashing { ahead >= 0.0 || dashing } else { wrap_angle(body - moving).cos() >= 0.0 };
    let target = if forward { moving } else { wrap_angle(moving + std::f32::consts::PI) };
    // (the body against the view: positive to its left, b.x < 0)
    let off = wrap_angle(body - view);
    let near_view = off.abs() < std::f32::consts::FRAC_PI_4;
    let turning = wrap_angle(target - body).abs() >= PICK_ON_MOVE;
    let motion = match (forward, near_view, turning) {
        (true, true, _) | (true, false, true) => Motion::Run,
        (true, false, false) => Motion::Side { left: off > 0.0 },
        (false, true, _) => Motion::Back,
        (false, false, true) => Motion::Keep,
        (false, false, false) => Motion::SideBack { left: off < 0.0 },
    };
    Pick { motion, forward, body: target }
}

/// What a feeler met (FUN_000ddd10's hit record, its +0): the terrain (kind 1), an object (2: a
/// placed object's parts, a door's leaves, a broken object's remains) or a blocker (3: a blocker,
/// a hinter or a fly-blocker, FUN_000dd8e0's record).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Met {
    Terrain,
    Object,
    Blocker,
}

/// A feeler's hit: how far along (m), what it met, and the y of its normal (facing back along the
/// ray: the terrain's walkable test).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub at: f32,
    pub met: Met,
    pub up: f32,
}

/// What the feelers did (`feel`): the direction they steer (unit, xz) when they do; the pace level
/// set down with its timer; the speed record's timer started (`Avoid::boost_off`); for the log, each
/// ray's block (how far, what), the long rays' slots and whether the hysteresis turned it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Felt {
    pub dir: Option<Vec2>,
    pub pace: Option<(i32, f32)>,
    pub boost_off: bool,
    pub blocks: [Option<(f32, Met)>; 3],
    pub long: Option<(f32, f32)>,
    pub turned: bool,
}

/// FUN_00018c40 as FUN_000255d0 calls it (no forced length; its scale flag's call, vtable +0x54,
/// returns 0 and changes nothing), for a character at pace level `level` with the speed factor
/// `factor` (char+0x720), `prism` inside one of the level's prisms (brain+0x3c4 bit 10), its rays
/// from `origin` along `heading` (unit, xz) tilted by `rise` (`rise`: each ray's y gets it before
/// it's normalised, so the feelers follow a slope), steering for `target`; `cast` is the world's ray
/// (from, unit direction, length):
///  - the lengths: FEELERS x0.6 at a walk, x1.6 at a dash, x0.9 x the factor when that's over 1;
///  - each ray in turn (centre, left, right); a hit that isn't walkable terrain blocks. At a dash
///    (or a run inside a prism) the first block drops the pace a notch for DASH_DROP s, or with the
///    factor over 1 starts the speed record's timer, and the feelers steer nothing this frame
///    (0x19255-0x1952b: the hit dropped, the loop ended). Else a left block turns it square to its
///    heading to the right (the frame's x column, flattened), a right block to the left; the
///    centre's only marks it;
///  - then both sides blocked: the side whose block is farther (the right's at or under the left's:
///    left); one side: away from it; the centre alone (a wall ahead): two rays along the side rays,
///    LONG x their length, the farther slot's side (a tie: right). FUN_000ddd10 writes the distance
///    only on a hit, so a long ray that misses keeps the slot's last distance, the centre's for the
///    left one, the left one's for the right (0x195d2-0x19667): copied as read. (The pick's
///    branches for a slot of 0 or under, 0x196b6-0x196cb, never run: the slots hold the centre's
///    hit at least, over 0.001 m; they're left out.) Then the hysteresis
///    (FUN_00018b50): the target on the other side, at a run or a dash, the pace level 2 for
///    TURN_WALK s and the target's side.
pub fn feel(level: i32, factor: f32, prism: bool, origin: Vec3, heading: Vec2, rise: f32, target: Vec3, cast: impl Fn(Vec3, Vec3, f32) -> Option<Hit>) -> Felt {
    let mut out = Felt::default();
    let scale = match level {
        2 => FEELER_WALK,
        4 => FEELER_DASH,
        _ => 1.0,
    } * if factor <= 1.0 { 1.0 } else { FEELER_FACTOR * factor };
    let h = Vec3::new(heading.x, 0.0, heading.y);
    // (the heading frame's x column, flattened: the right; and its reverse)
    let right = Vec2::new(-heading.y, heading.x);
    let left = -right;
    let mut dirs = [Vec3::ZERO; 3];
    let mut lens = [0.0f32; 3];
    let mut hit = [false; 3];
    let mut dist = [0.0f32; 3];
    // (FUN_000ddd10's distance out: written on a hit, kept on a miss)
    let mut last = 0.0f32;
    let mut steer = false;
    let mut dir = Vec2::ZERO;
    for (i, &(length, yaw)) in FEELERS.iter().enumerate() {
        let mut d = Quat::from_rotation_y(yaw.to_radians()) * h;
        d.y += rise;
        let d = d.normalize_or_zero();
        dirs[i] = d;
        let len = length * scale;
        lens[i] = len;
        if len <= FEELER_MIN {
            continue;
        }
        let Some(x) = cast(origin, d, len) else { continue };
        last = x.at;
        if x.met == Met::Terrain && x.up > WALKABLE {
            continue;
        }
        out.blocks[i] = Some((x.at, x.met));
        if level == 4 || (prism && level == 3) {
            if factor <= 1.0 {
                out.pace = Some((level - 1, DASH_DROP));
            } else {
                out.boost_off = true;
            }
            return out;
        }
        hit[i] = true;
        dist[i] = x.at;
        steer = true;
        match i {
            1 => dir = right,
            2 => dir = left,
            _ => {}
        }
    }
    match hit {
        [_, true, true] => dir = if dist[2] <= dist[1] { left } else { right },
        [true, false, false] => {
            let mut slot = |i: usize| {
                if let Some(x) = cast(origin, dirs[i], lens[i] * LONG) {
                    last = x.at;
                }
                last
            };
            let l = slot(1);
            let r = slot(2);
            out.long = Some((l, r));
            let mut pick = if l > r { left } else { right };
            if let Some(side) = side_of(heading, origin, target, left, right) {
                if side != pick && level >= 3 {
                    out.pace = Some((2, TURN_WALK));
                    out.turned = true;
                    pick = side;
                }
            }
            dir = pick;
        }
        _ => {}
    }
    if steer {
        out.dir = Some(if dir.length_squared() > STEER_MIN { dir.normalize() } else { dir });
    }
    out
}

/// FUN_00018b50: the side of `heading` (unit, xz) `target` lies on seen from `origin`, `left` when
/// (target - origin) x heading in xz is over 0, else `right`; none with a zero heading (SIDE_HEADING)
/// or the target within 4.5 mm (SIDE_NEAR).
fn side_of(heading: Vec2, origin: Vec3, target: Vec3, left: Vec2, right: Vec2) -> Option<Vec2> {
    if heading.length_squared() <= SIDE_HEADING {
        return None;
    }
    let d = Vec2::new(target.x - origin.x, target.z - origin.z);
    if d.length_squared() < SIDE_NEAR {
        return None;
    }
    let d = d.normalize();
    Some(if d.x * heading.y - d.y * heading.x > 0.0 { left } else { right })
}

/// FUN_00115b00 `(char, 0.4, &out)` as the feelers read it (0x18f0e): the height change from the
/// feet (`feet`, char+0x438) to the ground GROUND_PROBE x 2 ahead of the hips `hips` along the move
/// `along` (char+0x470, unit xz), or x 3 ahead when that's over GROUND_JUMP and the ground there is
/// found; 0 standing, with no ground GROUND_PROBE ahead (the function returns before it writes),
/// or over GROUND_GARBAGE. The second probe finding nothing keeps the first's height (FUN_000dddd0
/// writes it only on a hit).
fn rise(a: &world::Arena, hips: Vec3, feet: f32, along: Option<Vec2>) -> f32 {
    let Some(v) = along else { return 0.0 };
    let ground = |k: f32| {
        let p = hips + Vec3::new(v.x, 0.0, v.y) * (GROUND_PROBE * k);
        a.ray(p, Vec3::NEG_Y, GROUND_BELOW).map(|t| p.y - t)
    };
    let Some(first) = ground(1.0) else { return 0.0 };
    let mut dh = ground(2.0).unwrap_or(first) - feet;
    if dh.abs() > GROUND_JUMP {
        if let Some(g) = ground(3.0) {
            dh = g - feet;
        }
    }
    if dh.abs() > GROUND_GARBAGE { 0.0 } else { dh }
}

/// The sensor's +0x3c (0x521c0) as FUN_000255d0 calls it (brain+0x3bc 1.5 m, brain+0x3c0 90
/// degrees, no filter): every other character `others` (their positions; the dead aren't in it)
/// within PUSH_RADIUS across of `me`, overlapping in height (PUSH_MARGIN) and lying in the half in
/// front of `heading` (the angle to it under 90 degrees) or within PUSH_NEAR anywhere, adds
/// -(d.x, d.z) / |d|^2 (the offset across over its square: 1 over the distance, away, 0x3a5000 = -1;
/// 0x523e4-0x52417): the sum when any (the direction's y 0), none else. The direction isn't
/// normalised here (FUN_0004d970 does it).
pub fn push(me: Vec3, heading: Vec2, others: &[Vec3]) -> Option<Vec2> {
    let mut dir = Vec2::ZERO;
    let mut any = false;
    for &o in others {
        let d = o - me;
        let d2 = d.x * d.x + d.z * d.z;
        if d2 >= PUSH_RADIUS * PUSH_RADIUS || d.y.abs() >= BODY_HEIGHT + PUSH_MARGIN || d2 <= PUSH_MIN {
            continue;
        }
        let ahead = heading.x * d.x + heading.y * d.z > 0.0;
        if !(ahead || d2 < PUSH_NEAR * PUSH_NEAR) {
            continue;
        }
        dir -= Vec2::new(d.x, d.z) / d2;
        any = true;
    }
    any.then_some(dir)
}

/// An obstacle for the side-step (stage 3): its centre and radius (the sensor's +0x20, 0x52c90: by
/// its type kind, 2 OT_STATIC_GEOMETRY, 0x1a OT_PLATFORM and 0x1f OT_WORLDBUTTON its body's bounding
/// sphere through FUN_00236670), and whether it's from the first of the two world lists
/// ([world+0xa4]; the second, [world+0x12c], has its own radius at +0x38).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disc {
    pub at: Vec3,
    pub radius: f32,
    pub first: bool,
}

/// The side-step's obstacles: none in the demo. The sensor's +0x4c (0x52750) collects them from
/// [world+0xa4] and [world+0x12c], neither identified (todo/344 leads §4: blockers, hinters and
/// characters aren't in them; the radius method's character case, 0.5 m at the node, is for the
/// walker's own disc). A live read is the shot list's (0x529d0 / 0x527f7); until then the stage
/// has nothing to step round, as the leads expect for e13's pit run.
fn obstacles() -> Vec<Disc> {
    vec![]
}

/// The sensor's +0x4c (0x52750) as FUN_000255d0 calls it (its third argument STEP_REACH; the
/// second, 1.0, unread) for a character at `me` heading `heading` (unit, xz) with its own disc's
/// radius `mine`: each disc within its radius + STEP_REACH (3D), a first list's at most
/// STEP_MAX_RADIUS and overlapping in height (STEP_MARGIN), and in front (the unit offset along the
/// heading over STEP_AHEAD, a second list's offset over 0); its offset split along the heading and
/// across it (FUN_00052660); those whose offset across is under the two radii summed are in the
/// path, recorded {along, across} and sorted (FUN_00053a90: read as by the along, the records'
/// first field, the nearest first), and the direction is the first's offset across reversed (y 0:
/// a step away from the nearest obstacle in the way). None with nothing in the path.
pub fn side_step(me: Vec3, heading: Vec2, mine: f32, discs: &[Disc]) -> Option<Vec2> {
    let h = Vec3::new(heading.x, 0.0, heading.y);
    let mut inpath: Vec<(f32, Vec2)> = discs.iter().filter_map(|c| {
        let d = c.at - me;
        if d.length_squared() > (c.radius + STEP_REACH).powi(2) {
            return None;
        }
        if c.first && (c.radius > STEP_MAX_RADIUS || d.y.abs() >= BODY_HEIGHT + STEP_MARGIN || d.normalize_or_zero().dot(h) <= STEP_AHEAD) {
            return None;
        }
        if !c.first && d.dot(h) <= 0.0 {
            return None;
        }
        let along = d.dot(h);
        let across = d - h * along;
        (Vec2::new(across.x, across.z).length_squared() < (mine + c.radius).powi(2)).then_some((along, Vec2::new(across.x, across.z)))
    }).collect();
    inpath.sort_by(|a, b| a.0.total_cmp(&b.0));
    inpath.first().map(|&(_, across)| -across)
}

/// The sensor's +0x24 (0x519f0): the direction from `me` to `target` with its y 0, over the full
/// (3D) distance; zero within 0.01 m (its square under 0.0001: left as it was).
pub fn plain(me: Vec3, target: Vec3) -> Vec2 {
    let d = target - me;
    let l2 = d.length_squared();
    if l2 < STEER_MIN {
        return Vec2::ZERO;
    }
    Vec2::new(d.x, d.z) / l2.sqrt()
}

/// The character's heading (iface +0x34, 0x11e860): its commanded move, the stick the last AI frame
/// gave it (`moving`), else its body's forward (with char+0x800 = 0 the game takes the body's
/// backward instead, but a stick of nothing is a stand, where FUN_001195c0 sets it back to 1 the
/// same frame: so here always the forward). Unit, xz. Never the speed it makes: #363's cause, where
/// the demo's last step under a sideways gait turned the next move off its way. The follow's turn
/// check reads it too (play_follow.rs `turn_check`, FUN_00058740).
pub fn heading(m: &Player) -> Vec2 {
    moving(m).unwrap_or_else(|| Vec2::new(-m.yaw.sin(), -m.yaw.cos()))
}

/// The way `m`'s stick moves it (FUN_0011e680: char+0x5e8 put back in the world, flat and unit),
/// None for a stick of nothing (under MOVE_NONE): the last AI frame's command (`Avoid::moved_was`,
/// `move_turn`'s), not the speed it makes (#363). The feelers and the push look along it (through
/// `heading`), and the pace at a corner reads it (play_routes.rs `corner_pace`, FUN_000587d0).
pub fn moving(m: &Player) -> Option<Vec2> {
    let v = m.router.avoid.moved_was;
    (v.length_squared() >= MOVE_NONE).then(|| v.normalize())
}

/// The way `m` goes, from its velocity (char+0x470: the demo's last step), None under PROBE_STILL:
/// the direction FUN_00115b00 probes the ground along (`rise`), the one reader of the speed here.
fn going(m: &Player) -> Option<Vec2> {
    let v = Vec2::new(m.last_velocity.x, m.last_velocity.z);
    (v.length_squared() >= PROBE_STILL).then(|| v.normalize())
}

/// A body met, as the hit record's kind (`Met`).
fn met(k: world::BodyKind) -> Met {
    match k {
        world::BodyKind::Terrain => Met::Terrain,
        world::BodyKind::Blocker(_) | world::BodyKind::Hinter(_) => Met::Blocker,
        world::BodyKind::Object(_) | world::BodyKind::Other => Met::Object,
    }
}

/// BF_AVOID_LOG's level, read once.
fn log_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_AVOID_LOG").is_ok_and(|v| v != "0"))
}

/// BF_AVOID_OFF=1 (a diagnostic against the game): the plain direction alone, read once.
fn off() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_AVOID_OFF").is_ok_and(|v| v != "0"))
}

/// BF_AVOID_FREE_TURN=1 (a diagnostic against the game): the move takes the steering direction at
/// once, its turn left to the facing's 9 rad/s, as before #344's fix round (`move_turn`). Read once.
fn free_turn() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_AVOID_FREE_TURN").is_ok_and(|v| v != "0"))
}

/// FUN_000255d0 for `m` steering for `target` (the route's steering target, brain+0x478; the stage
/// mask 0, as every route walker passes it): the stages in order, the first that steers giving the
/// direction (unit, xz; zero for none, or a target at the origin), and which it was. The feelers
/// (off at the pace level 1 and while `Avoid::feelers_off` runs; the jump's brain+0x3c4 bit 12 is
/// never set, the demo takes no jumps) cast through the arena (`Arena::ray_kind_normal`) and their
/// drops go on the pace record (`Router::set_pace`) and `Avoid::boost_off`; the push looks through
/// `Avoid::near` (brain+0x3c4 bit 1, which skips it, has no writer found); the side-step has no
/// obstacles (`obstacles`).
pub fn steer(m: &mut Player, target: Vec3) -> (Vec2, Stage) {
    if target.length_squared() < NO_TARGET {
        return (Vec2::ZERO, Stage::Plain);
    }
    if off() {
        return (plain(m.position, target), Stage::Plain);
    }
    let heading = heading(m);
    if let Some(dir) = feelers(m, heading, target) {
        return (dir, Stage::Feelers);
    }
    if let Some(dir) = push(m.position, heading, &m.router.avoid.near) {
        if log_on() {
            let from: Vec<String> = m.router.avoid.near.iter().map(|o| format!("({:.1}, {:.1}) {:.2} m", o.x, o.z, o.xz().distance(m.position.xz()))).collect();
            println!("t {:.2}: {}: pushed off the characters near it ({}): steers ({:.2}, {:.2})", m.sim_time, unit_name(m), from.join(", "), dir.x, dir.y);
        }
        return (dir.normalize_or_zero(), Stage::Push);
    }
    if let Some(dir) = side_step(m.position, heading, SELF_RADIUS, &obstacles()) {
        return (dir.normalize_or_zero(), Stage::SideStep);
    }
    (plain(m.position, target), Stage::Plain)
}

/// Stage 1 for `m` (`feel`) with its heading `heading`, from its hips FEELER_DROP lower and
/// FEELER_BACK back, tilted by the ground ahead (`rise`), against the arena; the pace drops and
/// the speed record's timer applied. Its steer, if any; BF_AVOID_LOG says what the rays met.
fn feelers(m: &mut Player, heading: Vec2, target: Vec3) -> Option<Vec2> {
    let level = m.router.cap;
    if level <= 1 || m.router.avoid.feelers_off > 0.0 {
        return None;
    }
    let arena = world::arena()?;
    let a = &*arena;
    let origin = m.position - Vec3::Y * FEELER_DROP - Vec3::new(heading.x, 0.0, heading.y) * FEELER_BACK;
    let up = rise(a, m.position, m.position.y + GROUND, going(m));
    let cast = |o: Vec3, d: Vec3, len: f32| a.ray_kind_normal(o, d, len).map(|(t, k, n)| Hit { at: t, met: met(k), up: n.y });
    let factor = m.router.avoid.factor;
    let felt = feel(level, factor, routes::in_prism(), origin, heading, up, target, cast);
    if let Some((to, time)) = felt.pace {
        m.router.set_pace(to, time);
    }
    if felt.boost_off {
        m.router.avoid.boost_off = DASH_DROP;
    }
    if log_on() && (felt.dir.is_some() || felt.pace.is_some() || felt.boost_off) {
        let rays = ["centre", "left", "right"];
        let met: Vec<String> = felt.blocks.iter().zip(rays).filter_map(|(b, r)| b.map(|(d, k)| format!("{r} meets {} at {d:.2} m", match k {
            Met::Terrain => "steep terrain",
            Met::Object => "an object",
            Met::Blocker => "a blocker",
        }))).collect();
        let what = if felt.boost_off {
            format!("at a dash: the speed boost off for {DASH_DROP} s")
        } else if let (Some((to, time)), false) = (felt.pace, felt.turned) {
            format!("at the pace level {level}: down to {to} for {time} s")
        } else {
            let d = felt.dir.unwrap_or_default();
            let long = felt.long.map_or(String::new(), |(l, r)| format!(" (a wall ahead: the long rays' slots left {l:.2}, right {r:.2}{})",
                                                                   if felt.turned { "; to the target's side, the pace level 2 for 2 s" } else { "" }));
            format!("steers ({:.2}, {:.2}){long}", d.x, d.y)
        };
        println!("t {:.2}: {}: feelers (level {level}, x{:.3}{}) {}: {what}", m.sim_time, unit_name(m),
                 match level { 2 => FEELER_WALK, 4 => FEELER_DASH, _ => 1.0 } * if factor <= 1.0 { 1.0 } else { FEELER_FACTOR * factor },
                 if up.abs() > 0.01 { format!(", the ground {up:+.2} m ahead") } else { String::new() }, met.join(", "));
    }
    felt.dir
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wall across x = `x` (facing -x), met by any ray going +x within its length: a blocker.
    fn wall_at(x: f32) -> impl Fn(Vec3, Vec3, f32) -> Option<Hit> {
        move |o: Vec3, d: Vec3, len: f32| {
            if d.x <= 1e-6 {
                return None;
            }
            let t = (x - o.x) / d.x;
            (t > 0.0 && t <= len).then_some(Hit { at: t, met: Met::Blocker, up: 0.0 })
        }
    }

    /// Nothing anywhere.
    fn open(_: Vec3, _: Vec3, _: f32) -> Option<Hit> {
        None
    }

    /// The lengths by the pace level and the speed factor: a run's the list's (3.15 / 2.0475), a
    /// walk's x0.6, a dash's x1.6, x0.9 x the factor over 1 (the follow's 1.25: x1.125); at the
    /// pace level 1 FUN_000255d0's caller never casts (`feelers`), and a ray beyond its length
    /// meets nothing.
    #[test]
    fn the_lengths_go_with_the_pace() {
        // heading +x along z = 0, a wall 3.0 m in front of the origin: the centre ray meets it at a
        // run (3.15) but not at a walk (1.89)
        let o = Vec3::ZERO;
        let (h, t) = (Vec2::X, Vec3::new(20.0, 0.0, 0.0));
        let run = feel(3, 1.0, false, o, h, 0.0, t, wall_at(3.0));
        assert_eq!(run.blocks[0], Some((3.0, Met::Blocker)));
        let walk = feel(2, 1.0, false, o, h, 0.0, t, wall_at(3.0));
        assert_eq!(walk, Felt::default(), "a walk's 1.89 m doesn't reach");
        // the boost's factor: 3.15 x 1.125 = 3.54 reaches 3.4 m; x1.0 doesn't
        assert!(feel(3, 1.25, false, o, h, 0.0, t, wall_at(3.4)).blocks[0].is_some());
        assert!(feel(3, 1.0, false, o, h, 0.0, t, wall_at(3.4)).blocks[0].is_none());
        // a factor just over 1 shortens them (x0.9 x 1.05 = 0.945): as read, not max(1, 0.9 f)
        assert!(feel(3, 1.05, false, o, h, 0.0, t, wall_at(3.0)).blocks[0].is_none());
    }

    /// A side ray's hit turns it square to its heading, away from the hit: a wall on its left
    /// sends it right. The side ray reaches 2.0475 x sin 25 = 0.87 m across at a run, 0.52 m at a
    /// walk.
    #[test]
    fn a_side_hit_steers_square_away() {
        // heading -z (the demo's yaw 0): its left is -x, the left ray goes toward it. A wall at
        // x = -0.8 facing +x
        let left_wall = |o: Vec3, d: Vec3, len: f32| {
            if d.x >= -1e-6 {
                return None;
            }
            let t = (-0.8 - o.x) / d.x;
            (t > 0.0 && t <= len).then_some(Hit { at: t, met: Met::Blocker, up: 0.0 })
        };
        let (h, t) = (Vec2::new(0.0, -1.0), Vec3::new(0.0, 0.0, -20.0));
        let f = feel(3, 1.0, false, Vec3::ZERO, h, 0.0, t, left_wall);
        assert!(f.blocks[1].is_some() && f.blocks[0].is_none() && f.blocks[2].is_none(), "only the left ray: {:?}", f.blocks);
        assert!((f.blocks[1].unwrap().0 - 0.8 / 25f32.to_radians().sin()).abs() < 1e-3);
        // turned square to the right (+x)
        let d = f.dir.unwrap();
        assert!((d - Vec2::X).length() < 1e-5, "{d:?}");
        // at a walk the wall 0.8 m off is out of its reach
        assert_eq!(feel(2, 1.0, false, Vec3::ZERO, h, 0.0, t, left_wall).dir, None);
    }

    /// A wall ahead with both sides free: the long rays pick the roomier side, then the hysteresis
    /// holds it to the target's side at a run (the pace level 2 for 2 s), not at a walk.
    #[test]
    fn a_wall_ahead_goes_to_the_roomier_side_held_to_the_targets() {
        // heading +x (its left is -z); a wall at x = 3 from z = -6 to z = 1: the long left ray
        // meets it at z = -1.4, the long right one runs past its end
        let wall = |o: Vec3, d: Vec3, len: f32| {
            if d.x <= 1e-6 {
                return None;
            }
            let t = (3.0 - o.x) / d.x;
            let z = o.z + d.z * t;
            (t > 0.0 && t <= len && (-6.0..=1.0).contains(&z)).then_some(Hit { at: t, met: Met::Blocker, up: 0.0 })
        };
        // the target straight ahead beyond the wall, a hair to the right
        let f = feel(3, 1.0, false, Vec3::ZERO, Vec2::X, 0.0, Vec3::new(20.0, 0.0, 0.01), wall);
        assert!(f.blocks[0].is_some() && f.blocks[1].is_none() && f.blocks[2].is_none());
        let (l, r) = f.long.unwrap();
        // the right long ray misses: its slot keeps the left's distance (the quirk as read), so the
        // left isn't farther and the right is picked
        assert_eq!(l, r);
        assert!((f.dir.unwrap() - Vec2::new(0.0, 1.0)).length() < 1e-5, "right (+z): {:?}", f.dir);
        assert!(!f.turned, "the target's side is the right too");
        // the target on the left: at a run the hysteresis takes the left with a 2 s walk
        let f = feel(3, 1.0, false, Vec3::ZERO, Vec2::X, 0.0, Vec3::new(20.0, 0.0, -0.5), wall);
        assert!(f.turned && f.pace == Some((2, TURN_WALK)));
        assert!((f.dir.unwrap() - Vec2::new(0.0, -1.0)).length() < 1e-5);
        // at a walk (1.5 m from the wall, its centre ray 1.89 m) it isn't held: the long rays'
        // pick stands, the right
        let f = feel(2, 1.0, false, Vec3::new(1.5, 0.0, 0.0), Vec2::X, 0.0, Vec3::new(20.0, 0.0, -0.5), wall);
        assert!(f.blocks[0].is_some() && !f.turned && f.pace.is_none());
        assert!((f.dir.unwrap() - Vec2::new(0.0, 1.0)).length() < 1e-5);
    }

    /// Both sides blocked: the side whose block is farther.
    #[test]
    fn both_sides_blocked_go_to_the_farther() {
        // heading +x; walls at z = -0.6 (its left) and z = +0.8 (its right): the right one is
        // farther along its ray
        let walls = |o: Vec3, d: Vec3, len: f32| {
            let t = if d.z < -1e-6 { (-0.6 - o.z) / d.z } else if d.z > 1e-6 { (0.8 - o.z) / d.z } else { return None };
            (t > 0.0 && t <= len).then_some(Hit { at: t, met: Met::Blocker, up: 0.0 })
        };
        let f = feel(4, 1.0, false, Vec3::ZERO, Vec2::X, 0.0, Vec3::new(20.0, 0.0, 0.0), walls);
        // (at a dash the first block only drops the pace: no steer)
        assert_eq!((f.dir, f.pace), (None, Some((3, DASH_DROP))));
        let f = feel(3, 1.0, false, Vec3::ZERO, Vec2::X, 0.0, Vec3::new(20.0, 0.0, 0.0), walls);
        assert!(f.blocks[1].is_some() && f.blocks[2].is_some());
        assert!((f.dir.unwrap() - Vec2::new(0.0, 1.0)).length() < 1e-5, "to the right (+z): {:?}", f.dir);
    }

    /// A dash's block drops the pace to a run for 1.5 s and steers nothing; with the follow's boost
    /// it takes the boost off instead (the level kept); inside a prism a run drops to a walk.
    #[test]
    fn a_dash_hit_drops_the_pace_or_the_boost() {
        let (h, t) = (Vec2::X, Vec3::new(20.0, 0.0, 0.0));
        let f = feel(4, 1.0, false, Vec3::ZERO, h, 0.0, t, wall_at(4.0));
        assert_eq!((f.dir, f.pace, f.boost_off), (None, Some((3, DASH_DROP)), false));
        let f = feel(4, 1.25, false, Vec3::ZERO, h, 0.0, t, wall_at(4.0));
        assert_eq!((f.dir, f.pace, f.boost_off), (None, None, true));
        let f = feel(3, 1.0, true, Vec3::ZERO, h, 0.0, t, wall_at(3.0));
        assert_eq!((f.dir, f.pace), (None, Some((2, DASH_DROP))));
        // outside one a run's block steers
        assert!(feel(3, 1.0, false, Vec3::ZERO, h, 0.0, t, wall_at(3.0)).dir.is_some());
    }

    /// Walkable terrain under the tilted rays isn't a block (its normal's y over 0.7); steep
    /// terrain is, and an object always is.
    #[test]
    fn walkable_ground_is_no_block() {
        let ground = |up: f32, met: Met| move |_: Vec3, _: Vec3, len: f32| Some(Hit { at: len * 0.5, met, up });
        let (h, t) = (Vec2::X, Vec3::new(20.0, 0.0, 0.0));
        assert_eq!(feel(3, 1.0, false, Vec3::ZERO, h, 0.0, t, ground(0.9, Met::Terrain)).dir, None);
        assert!(feel(3, 1.0, false, Vec3::ZERO, h, 0.0, t, ground(0.6, Met::Terrain)).dir.is_some());
        assert!(feel(3, 1.0, false, Vec3::ZERO, h, 0.0, t, ground(0.9, Met::Object)).dir.is_some());
        assert_eq!(feel(3, 1.0, false, Vec3::ZERO, h, 0.0, t, open), Felt::default());
    }

    /// The tilt: the ground rising ahead lifts the rays (each ray's y gets it before it's
    /// normalised), so a ray that would meet a rise's face passes over it.
    #[test]
    fn the_rays_follow_the_ground() {
        let seen = std::cell::Cell::new(0.0f32);
        let probe = |_: Vec3, d: Vec3, _: f32| {
            seen.set(d.y);
            None
        };
        feel(3, 1.0, false, Vec3::ZERO, Vec2::X, 0.5, Vec3::new(20.0, 0.0, 0.0), probe);
        assert!((seen.get() - 0.5 / 1.25f32.sqrt()).abs() < 1e-5);
    }

    /// FUN_00018b50: the target's side of the heading.
    #[test]
    fn the_targets_side() {
        let (l, r) = (Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0));
        // heading -z (yaw 0): the left is -x
        let h = Vec2::new(0.0, -1.0);
        assert_eq!(side_of(h, Vec3::ZERO, Vec3::new(-3.0, 0.0, -5.0), l, r), Some(l));
        assert_eq!(side_of(h, Vec3::ZERO, Vec3::new(3.0, 0.0, -5.0), l, r), Some(r));
        assert_eq!(side_of(h, Vec3::ZERO, Vec3::new(0.001, 5.0, 0.001), l, r), None, "within 4.5 mm across");
        assert_eq!(side_of(Vec2::ZERO, Vec3::ZERO, Vec3::new(3.0, 0.0, -5.0), l, r), None);
    }

    /// The push: a character 1 m ahead pushes it straight back by 1 (1 over the distance), one
    /// behind further than 0.5 m doesn't, one 0.4 m behind does, and one beyond 1.5 m doesn't;
    /// several add up.
    #[test]
    fn the_push_from_characters() {
        let me = Vec3::ZERO;
        let h = Vec2::X;
        let d = push(me, h, &[Vec3::new(1.0, 0.0, 0.0)]).unwrap();
        assert!((d - Vec2::new(-1.0, 0.0)).length() < 1e-5);
        let d = push(me, h, &[Vec3::new(0.0, 0.0, 1.25)]).unwrap_or_default();
        assert_eq!(d, Vec2::ZERO, "beside it (90 degrees) and over 0.5 m: none");
        assert!(push(me, h, &[Vec3::new(-1.0, 0.0, 0.0)]).is_none(), "behind");
        let d = push(me, h, &[Vec3::new(-0.4, 0.0, 0.0)]).unwrap();
        assert!((d - Vec2::new(2.5, 0.0)).length() < 1e-4, "within 0.5 m anywhere, 1 / 0.4 away");
        assert!(push(me, h, &[Vec3::new(1.6, 0.0, 0.0)]).is_none(), "beyond 1.5 m");
        assert!(push(me, h, &[Vec3::new(1.0, 5.0, 0.0)]).is_none(), "far above");
        let d = push(me, h, &[Vec3::new(1.0, 0.0, 0.5), Vec3::new(1.0, 0.0, -0.5)]).unwrap();
        assert!(d.y.abs() < 1e-5 && (d.x + 1.6).abs() < 1e-4, "two either side: straight back, 2 x 1 / 1.25: {d:?}");
    }

    /// The side-step: away from the nearest disc in the path, none for one off it or behind.
    #[test]
    fn the_side_step() {
        let me = Vec3::ZERO;
        let h = Vec2::X;
        // a 0.5 m disc 2 m ahead, 0.3 m to +z: in the path (0.3 < 0.5 + 0.5); step to -z by 0.3
        let d = side_step(me, h, SELF_RADIUS, &[Disc { at: Vec3::new(2.0, 0.0, 0.3), radius: 0.5, first: true }]).unwrap();
        assert!((d - Vec2::new(0.0, -0.3)).length() < 1e-5);
        assert!(side_step(me, h, SELF_RADIUS, &[Disc { at: Vec3::new(2.0, 0.0, 1.2), radius: 0.5, first: true }]).is_none(), "off the path");
        assert!(side_step(me, h, SELF_RADIUS, &[Disc { at: Vec3::new(-2.0, 0.0, 0.1), radius: 0.5, first: true }]).is_none(), "behind");
        assert!(side_step(me, h, SELF_RADIUS, &[Disc { at: Vec3::new(3.0, 0.0, 0.1), radius: 0.5, first: true }]).is_none(), "beyond 2 m of its rim");
        // two in the path: the nearer decides
        let d = side_step(me, h, SELF_RADIUS, &[Disc { at: Vec3::new(2.4, 0.0, -0.2), radius: 0.5, first: false },
                                                Disc { at: Vec3::new(1.5, 0.0, 0.4), radius: 0.5, first: true }]).unwrap();
        assert!((d - Vec2::new(0.0, -0.4)).length() < 1e-5);
    }

    /// FUN_00019af0: from a stand the move takes the direction at once; moving, it turns toward it
    /// at most 3.49 rad/s, the rate eased from the last (a third of the way a game frame at a run),
    /// and within a degree it keeps its heading; a stand's small direction is taken as it is. The
    /// heading it turns from is the last frame's move (#363), so the turn goes on from where the
    /// last one left it, whatever the character's step did.
    #[test]
    fn the_move_turns_at_most_200_degrees_a_second() {
        let dt = GAME_FRAME;
        let mut m = Player::new(2);
        // its last move -z, wanting +x: a quarter turn to its right; its last step went the other
        // way (an aimed side step's, say): the step isn't the heading
        m.router.avoid.moved = Vec2::new(0.0, -1.0);
        m.last_velocity = Vec3::new(3.0, 0.0, 0.0);
        m.router.avoid.tick(dt);
        let d = move_turn(&mut m, Vec2::X, 1.0, dt);
        assert!((d - Vec2::X).length() < 1e-5, "after a stand, at once: {d:?}");
        assert_eq!(m.router.avoid.moved, Vec2::X, "the move kept for the next frame's heading");
        let mut m = Player::new(2);
        m.router.avoid.moved = Vec2::new(0.0, -1.0);
        m.router.avoid.dir_now = Vec2::new(0.0, -1.0);
        m.last_velocity = Vec3::new(3.0, 0.0, 0.0);
        m.router.avoid.tick(dt);
        let d = move_turn(&mut m, Vec2::X, 1.0, dt);
        // the rate eased from 0: 3.49 / 3 rad/s for a frame of 1/30 s, from -z (not from +x)
        let turned = (MOVE_TURN_MAX / 3.0) * dt;
        assert!((d - Vec2::new(turned.sin(), -turned.cos())).length() < 1e-4, "{d:?}");
        assert!((m.router.avoid.turn_rate + MOVE_TURN_MAX / 3.0).abs() < 1e-4, "the rate kept: {}", m.router.avoid.turn_rate);
        // frame after frame the rate climbs to the cap, never past it, each turn from the last move
        // (the gap to the cap down by 2/3 a frame: 3.49 x (2/3)^11 = 0.04)
        for _ in 0..10 {
            m.router.avoid.tick(dt);
            move_turn(&mut m, Vec2::X, 1.0, dt);
        }
        assert!((m.router.avoid.turn_rate.abs() - MOVE_TURN_MAX).abs() < 0.05, "{}", m.router.avoid.turn_rate);
        // and it gets there: a quarter turn in well under a second at 200 degrees a second
        for _ in 0..12 {
            m.router.avoid.tick(dt);
            move_turn(&mut m, Vec2::X, 1.0, dt);
        }
        assert!((m.router.avoid.moved - Vec2::X).length() < 1e-4, "{:?}", m.router.avoid.moved);
        // at 15 fps the same ease over two game frames: (2/3)^2 of the way kept
        let mut m = Player::new(2);
        m.router.avoid.moved_was = Vec2::new(0.0, -1.0);
        m.router.avoid.dir_was = Vec2::new(0.0, -1.0);
        move_turn(&mut m, Vec2::X, 1.0, 2.0 * dt);
        assert!((m.router.avoid.turn_rate + MOVE_TURN_MAX * (1.0 - 4.0 / 9.0)).abs() < 1e-4, "{}", m.router.avoid.turn_rate);
        // within a degree: the heading; a stand's direction: as it is, and kept at its size
        let near = Vec2::new(0.5f32.to_radians().sin(), -0.5f32.to_radians().cos());
        assert_eq!(move_turn(&mut m, near, 1.0, dt), Vec2::new(0.0, -1.0));
        assert_eq!(move_turn(&mut m, Vec2::X, MOVE_STAND, dt), Vec2::X);
        assert_eq!(m.router.avoid.moved, Vec2::X * MOVE_STAND);
        assert_eq!((move_size(1), move_size(2), move_size(4)), (MOVE_STAND, MOVE_WALK, 1.0));
    }

    /// iface +0x34: the heading is the last frame's move, a stand's tiny one included, else the
    /// body's forward; never the speed it makes. A frame no goal moves it leaves none.
    #[test]
    fn the_heading_is_the_last_move() {
        let mut m = Player::new(2);
        m.yaw = std::f32::consts::FRAC_PI_2;
        m.last_velocity = Vec3::new(0.0, 0.0, 6.0);
        // nothing commanded: the body's forward (yaw 90 degrees: -x)
        assert!((heading(&m) - Vec2::new(-1.0, 0.0)).length() < 1e-5);
        assert_eq!(moving(&m), None);
        m.router.avoid.moved = Vec2::new(0.0, -0.01);
        m.router.avoid.tick(GAME_FRAME);
        assert!((heading(&m) - Vec2::new(0.0, -1.0)).length() < 1e-5, "a stand's move has a direction");
        assert!(moving(&m).is_some_and(|v| (v - Vec2::new(0.0, -1.0)).length() < 1e-5));
        // the next frame, with no move made: none again
        m.router.avoid.tick(GAME_FRAME);
        assert_eq!(moving(&m), None);
        // a stand (the liquids' guard, a hold) forgets the move and the view
        m.router.avoid.moved = Vec2::X;
        m.router.avoid.view = Some(View::default());
        m.router.avoid.stand();
        assert_eq!((m.router.avoid.moved, m.router.avoid.view), (Vec2::ZERO, None));
    }

    /// FUN_00019ed0: the view starts from the facing, turns toward the direction at most 200
    /// degrees a second, its fraction eased a third of the way a game frame at a run (a fifth at a
    /// walk), and closes the last of a turn at 2/3 of what's left a frame; within a degree it holds.
    #[test]
    fn the_view_turns_to_the_move_at_200_degrees_a_second() {
        let dt = GAME_FRAME;
        let mut m = Player::new(2);
        m.yaw = 0.0;
        // wanting +x from facing -z: the demo's yaw -90 degrees
        view_turn(&mut m, Vec2::X, 1.0, dt);
        let v = m.router.avoid.view.unwrap();
        assert!((v.rate + 1.0 / 3.0).abs() < 1e-5, "eased from 0 toward -1: {}", v.rate);
        assert!((v.yaw + VIEW_TURN_MAX * dt / 3.0).abs() < 1e-5, "{}", v.yaw);
        // at a walk the ease is a fifth
        let mut w = Player::new(2);
        view_turn(&mut w, Vec2::X, MOVE_WALK, dt);
        assert!((w.router.avoid.view.unwrap().rate + 0.2).abs() < 1e-5);
        // it gets there: 90 degrees in about half a second, and holds within a degree
        for _ in 0..20 {
            view_turn(&mut m, Vec2::X, 1.0, dt);
        }
        let v = m.router.avoid.view.unwrap();
        assert!((v.yaw + std::f32::consts::FRAC_PI_2).abs() < 0.02, "{}", v.yaw.to_degrees());
        // the last of a turn: a 2-degree gap the frame could close (6.7 degrees a frame) closes
        // by 2/3, the kept fraction first halved
        let mut c = Player::new(2);
        c.router.avoid.view = Some(View { yaw: 2f32.to_radians(), ..Default::default() });
        view_turn(&mut c, Vec2::new(0.0, -1.0), 1.0, dt);
        let v = c.router.avoid.view.unwrap();
        assert!((v.yaw.to_degrees() - 2.0 / 3.0).abs() < 1e-3, "{}", v.yaw.to_degrees());
        // within a degree: no turn, the fraction 0; no direction: none
        c.router.avoid.view = Some(View { yaw: 0.5f32.to_radians(), rate: 0.3, ..Default::default() });
        view_turn(&mut c, Vec2::new(0.0, -1.0), 1.0, dt);
        assert_eq!(c.router.avoid.view.unwrap(), View { yaw: 0.5f32.to_radians(), rate: 0.0, backing: false, turned: true });
        view_turn(&mut c, Vec2::ZERO, 1.0, dt);
        assert_eq!(c.router.avoid.view.unwrap().yaw, 0.5f32.to_radians());
        // a frame with no turn drops it at the next tick: the next walk starts from the facing
        c.router.avoid.tick(dt);
        assert!(c.router.avoid.view.is_some_and(|v| !v.turned), "turned last frame: kept for this one's turn");
        c.router.avoid.tick(dt);
        assert_eq!(c.router.avoid.view, None);
        c.yaw = 1.0;
        view_turn(&mut c, Vec2::new(-1.0f32.sin(), -1.0f32.cos()), 1.0, dt);
        assert!((c.router.avoid.view.unwrap().yaw - 1.0).abs() < 1e-5);
    }

    /// FUN_001195c0: running on its view and its body, it runs; a move across while the body has
    /// turned onto it and the view lags steps sideways to the side the body is on; a move behind
    /// the view backs off, the body turned away from it; the gun up takes the move as straight
    /// down to a cosine of 0.25, a dash always forward.
    #[test]
    fn the_motion_is_picked_against_its_own_view() {
        let d = |a: f32| a.to_radians();
        // all three along -z: forward, run, the body's target the move
        let p = pick(0.0, 0.0, 0.0, false, false);
        assert_eq!((p.motion, p.forward, p.body), (Motion::Run, true, 0.0));
        // a move 80 degrees to the view's left, the body still on the view: mostly across, the
        // move in the body's front half: forward, still 80 degrees to turn: run
        let p = pick(0.0, 0.0, d(80.0), false, false);
        assert_eq!((p.motion, p.forward), (Motion::Run, true));
        assert!((p.body - d(80.0)).abs() < 1e-5);
        // the body there, the view lagging 70 degrees off it: a side step to the left
        assert_eq!(pick(d(10.0), d(80.0), d(80.0), false, false).motion, Motion::Side { left: true });
        assert_eq!(pick(d(-10.0), d(-80.0), d(-80.0), false, false).motion, Motion::Side { left: false });
        // the view within 45 degrees of the body: a run again
        assert_eq!(pick(d(40.0), d(80.0), d(80.0), false, false).motion, Motion::Run);
        // a move 150 degrees off the view: backward, the body's target its reverse (30 degrees to
        // the view's right), the body on the view: backs off
        let p = pick(0.0, 0.0, d(150.0), false, false);
        assert_eq!((p.motion, p.forward), (Motion::Back, false));
        assert!((p.body - d(-30.0)).abs() < 1e-5);
        // the body more than 45 degrees off the view, on the reverse: backs off sideways, the move
        // to the view's left (the body to its right); still turning to it: no command
        assert_eq!(pick(0.0, d(-60.0), d(120.0), false, false).motion, Motion::SideBack { left: true });
        assert_eq!(pick(0.0, d(60.0), d(120.0), false, false).motion, Motion::Keep);
        // a move 70 degrees off the view is across without the gun up (cos 0.34 < 0.4): the case
        // by the body's half; with it up straight (0.34 >= 0.25): forward whatever the body
        assert!(pick(0.0, 0.0, d(70.0), false, false).forward, "across, in the body's front half");
        assert!(!pick(0.0, d(180.0), d(70.0), false, false).forward, "across, in the body's back half");
        assert!(pick(0.0, d(180.0), d(70.0), true, false).forward, "the gun up: straight ahead");
        // 110 degrees off: across, the body facing it forward; with the gun up straight back
        assert!(pick(0.0, d(110.0), d(110.0), false, false).forward);
        assert!(!pick(0.0, d(110.0), d(110.0), true, false).forward);
        // a dash is forward whatever the view
        assert!(pick(0.0, 0.0, d(170.0), false, true).forward);
    }

    /// The plain direction: flat, over the 3D distance, none within 0.01 m.
    #[test]
    fn the_plain_direction() {
        let d = plain(Vec3::ZERO, Vec3::new(3.0, 4.0, 0.0));
        assert!((d - Vec2::new(0.6, 0.0)).length() < 1e-5);
        assert_eq!(plain(Vec3::ZERO, Vec3::new(0.005, 0.0, 0.0)), Vec2::ZERO);
    }
}
