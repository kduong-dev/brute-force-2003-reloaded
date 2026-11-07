//! GOAL_ATTACK_COVER (19) and GOAL_DEBUG1's (120) cover-else-vary (#384, part A of #26;
//! viewer/docs/brain.md "Cover"): a level character's brain fights from the level's
//! `<cover-point>`s as default.xbe's does. Every function cited is in viewer/docs/code-map.md,
//! "AI"; the reading is todo/26-enemy-behaviour/leads.md §3, corrected where the code says
//! otherwise (brain.md lists the corrections).
//!
//!  - **The pick** (ATTACK_COVER's start 0x2f8e0 = FUN_0002f270(brain, 0.75, 30, 0); `pick`): a
//!    target is needed, within the held gun's reach when it's seen (FUN_00023d00). A flood from the
//!    character's own world point through the links within 30 m (FUN_000594a0, `Graph::flood`)
//!    takes the first cover point the filter 0x26240 passes (`accepts`): valid, not claimed, not
//!    the point held, 0.75-30 m from the character's node position, the hips (3D, both bounds
//!    taken: 0x5e070; nothing exempts a point it stands by), inside its home area (FUN_00023630),
//!    its set-valid on, not on the brain's 10 s blacklist (brain+0x570, FUN_00048000) and, on the
//!    first pass of a brain out of the squad, not one of its last four points (brain+0x57c); then
//!    for every enemy it knows (FUN_00026060, their hips too) 5-50 m from it, not past it (the
//!    enemy nearer than the point within ~75 degrees of its bearing), and inside the point's cone
//!    (under fov-angle / 2 off its facing). No sight test. A route to it (FUN_000416a0), the point
//!    claimed, the re-pick timer 23 s (10 s with a squad record, FUN_0002f250).
//!  - **The walk** (FUN_0002fd40's first branch, `walk_to`): the route at 1 m an entry, 0.15 m
//!    the point; a run, the dash when the point is over 5 m off, the pick was over 13 m and the
//!    character under half its hit points; the look on the target unless dashing (brain+0x488),
//!    so it backs and side-steps to cover facing the enemy (`walk`, FUN_001195c0's pick); the run
//!    lets the crouch go (FUN_0004d970). Arrived: the hold timer 1-2.5 s, state 1.
//!  - **At the point** (`update`, the state brain+0x518): flanked (FUN_0002f960, an enemy seen
//!    outside the cone) ends it; 1 settles (the point on the recent list); 2 holds: the target seen
//!    within 8 m blacklists the point and fails it, a crouch where the point says (h_18ed19af), more
//!    than 0.9 m off the point fails it, a seen enemy is faced and fired at and keeps the hold timer
//!    full; after 10 s at the point (or the hold out) a re-pick with an enemy in sight or the
//!    re-pick timer out, else the peek: the pop-up (8) at a crouch point, the open fire (10) with
//!    no side, else a side (the target's, with both: where it was last seen once seen out there)
//!    and a dodge motion (6) or a step out (4,
//!    until 2 m from the point and clear of the cover in front), 5-7 s out firing (7), and back
//!    (3, or the opposite dodge 5). Only state 7 counts the peek timer down (0x30c94): the pop-up
//!    (8) and the crouched fire at a h_f9f3561f point (9) last until the character is flanked, as
//!    read.
//!  - **The hook** (FUN_000313e0, `hit`, `shot_at`, `takes`): the third hit in state 2 blows the
//!    cover (blacklisted, done); a hit or a shot at it cuts the peek short; it swallows the events
//!    it handles (the message handler 0x3dce0 returns at once when the goal's hook says 1).
//!  - **DEBUG1** (`debug1_start`, `fight`): its start (0x36a70) tries cover only when the brain is
//!    hurt below half or outnumbered (FUN_00036970 with the threat score FUN_0004a3a0 and its sum
//!    FUN_0004a280, `outnumbered`), else ATTACK_VARY; the cover ended (done or failed), ATTACK_VARY
//!    (0x36b70), and from ATTACK_VARY, 5 s on, cover again whenever FUN_00036970 says so.
//!
//! What the demo leaves out (brain.md "Cover", Not done): the help call EVT_ALARM_CLOSE_RANGE at
//! the pick (#26 part F), the squad's broadcasts (EVT_ORDER_*: no level character has a squad
//! record), the cover grenadier's throws (h_e1575d8d, part D), the fire on the walk to cover and
//! back (FUN_00024f80 at 20 and 35 degrees: the demo's enemy drive holds fire while it moves,
//! play_enemies.rs `step`), the arrival's slow-down (FUN_0002fba0: only a walk takes it, as for the
//! follow), the facing the pick sets (char+0x1b4's +0x3c), the threat score's Molotov runner and
//! grenadier cases (no such goal or quick goal in the demo). A goal pushed on top (THROW_GRENADE,
//! #386) restarts DEBUG1's start when it's popped (FUN_000377b0 at 0x37d85): no goal is pushed in
//! the demo yet.
//!
//! Test hooks: BF_COVER_LOG=1 (each pick, the states, the peeks, the exits, DEBUG1's halves and
//! FUN_00036970's numbers; =2 also why each point the flood met was passed over, and a row every
//! half second of each cover goal: its state, timers, crouch, where it is and how far its target);
//! BF_TEST_TURN=<s>[,<r>] (at s, every level character with a brain within r m, 60 by default, of
//! the controlled character turns to face it: the takes' `!yaw` staging, todo/132 notes);
//! BF_TEST_PUT=<s>,<name>,<x>,<z>[,<heading>[,<hp>]][;...] (at s the level character whose brain is
//! named `name`, as BF_BRAIN_LOG names it, is put on the floor at x, z, given hp hit points when set:
//! the takes' `!place` and `hp`).

use std::collections::VecDeque;

use bf_viewer::bf::character::Game;
use bf_viewer::bf::level::CoverPoint;
use bf_viewer::bf::nav::{self, Dest, Graph};

use super::*;

/// The BF_TEST_TURN and BF_TEST_PUT hooks, between the frame's input and the characters' step.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, (put_hook, turn_hook).chain().after(read_input).before(update_player).run_if(in_state(AppState::Playing)));
}

/// The goals this module runs (the GOAL_ name table 0x3bcbb4): ATTACK_COVER, DEBUG1.
pub const GOAL_ATTACK_COVER: i64 = goals::GOAL_ATTACK_COVER;
pub const GOAL_DEBUG1: i64 = goals::GOAL_DEBUG1;
/// The events the hook FUN_000313e0 takes (the EVT_ name table 0x3bdb9c): DAMAGE_LIGHT 14,
/// COVER_BLOWN 16, TGT_ENTRENCHED 18, SHOT_AT 24, SHOT_BLOCKAGE 33, SHOT_BLOCKED_BY_FRIEND 34,
/// TGT_TOO_CLOSE 85 (its byte table 0x315a8 by event - 13).
pub const EVT_DAMAGE_LIGHT: i64 = 14;
const EVT_COVER_BLOWN: i64 = 16;
const EVT_TGT_ENTRENCHED: i64 = 18;
const EVT_SHOT_AT: i64 = 24;
const EVT_SHOT_BLOCKAGE: i64 = 33;
const EVT_SHOT_BLOCKED_BY_FRIEND: i64 = 34;
const EVT_TGT_TOO_CLOSE: i64 = 85;

/// ATTACK_COVER's start radii (0x2f8e0: FUN_0002f270(brain, 0.75, 30.0, 0); 0x3f400000,
/// 0x41f00000): the cover point within these of the character (3D, the filter 0x5e070), and the
/// flood within the farther of its own point (the filter's +0x110, FUN_000594a0).
const MIN_RADIUS: f32 = 0.75;
const MAX_RADIUS: f32 = 30.0;
/// The re-pick timer (FUN_0002f250: 23.0, 10.0 for a character with a squad record char+0x59c),
/// set by the pick (brain+0x55c) and when a re-pick finds nothing.
const REPICK: f32 = 23.0;
const REPICK_SQUAD: f32 = 10.0;
/// FUN_00026060: a point 5-50 m from each enemy (3D; 25 and 2500 squared, its +0x244 bit 0)...
const ENEMY_NEAR: f32 = 5.0;
const ENEMY_FAR: f32 = 50.0;
/// ... and not past it: rejected when the enemy is nearer than the point with the cosine of the
/// two bearings over this (0.25: within about 75.5 degrees).
const PAST_COS: f32 = 0.25;
/// ATTACK_COVER's state 1 pushes the point on the recent deque (brain+0x57c, FUN_000384a0) and
/// pops its oldest while it holds more than this (0x2ff85: `cmp [+0x54c], 4`).
const RECENT: usize = 4;
/// FUN_00048030 puts a point on the blacklist (brain+0x570) for this long (10.0, 0x41200000);
/// the AI update's FUN_00048080 counts it down by the frame's time and drops it at 0.
const BLACKLIST: f32 = 10.0;
/// The walk's radii (FUN_0002fd40 0x31133, 0x31185): 1.0 an entry, 0.15 for its last, the point.
const WALK_RADIUS: f32 = 1.0;
const LAST_RADIUS: f32 = 0.15;
/// The walk's dash (0x310d1-0x310fe): the point over this across (25 = 5 m squared) ...
const DASH_FAR: f32 = 5.0;
/// ... the pick's distance (brain+0x540) over this ([0x3a53c4] = 13.0) ...
const DASH_PICK: f32 = 13.0;
/// ... and the character's hit points under this share (FUN_000238e0 < 0x32: percent).
const DASH_HP: i32 = 50;
/// FUN_0004d970's modes: a walk, a run, the dash.
const WALK: i32 = 2;
const RUN: i32 = 3;
const DASH: i32 = 4;
/// State 2: the target seen within this (3D; [0x3a5290] = 64, its square) blacklists the point
/// and fails the goal (0x300d5).
const TOO_NEAR: f32 = 8.0;
/// State 2: more than this across off the point (0.81, [0x3a5538], its square) fails it.
const OFF_POINT: f32 = 0.9;
/// State 2: the hold ends once the time at the point reaches this ([0x3a4fdc] = 10.0).
const AT_COVER: f32 = 10.0;
/// The hook: the third hit in state 2 blows the cover (0x314f1: `cmp eax, 3`).
const HITS: u32 = 3;
/// FUN_00047b40 / FUN_00047d40: a dodge when the point allows one and a draw is over this (70%)
/// ...
const DODGE_ROLL: f32 = 0.3;
/// ... a short one when a second draw is at most this (30% of those).
const SHORT_ROLL: f32 = 0.3;
/// FUN_00047f20: a dodge out that wasn't a short one is forgotten (walked back) when a draw is
/// under this (70%).
const FORGET_DODGE: f32 = 0.7;
/// State 4, the step: on while within this of the point (4.0, its square) ...
const STEP_POINT: f32 = 2.0;
/// ... or of where the step started (2.25, [0x3a53c0])...
const STEP_START: f32 = 1.5;
/// ... or while FUN_0004cab0's ray along the point's facing (15 m, 0x41700000) meets something
/// within this (5.0, 0x40a00000): the cover itself. The ray starts at the character's node
/// position, the hips (its vtable +0x40 at 0x308b6: `Ctx::hips`).
const PROBE: f32 = 5.0;
/// State 3, the walk back: done when the move passes within this of the point (FUN_00025870 at
/// 0.2, 0x3e4ccccd) ...
const BACK_RADIUS: f32 = 0.2;
/// ... at a walk within this (d^2 < 1.0, [0x3a4f78]).
const BACK_WALK: f32 = 1.0;
/// State 5: back at the point when within this after the dodge (0.04, [0x3a50c0], its square).
const RETURN_NEAR: f32 = 0.2;
/// State 7: with no clear ray to the target a peek timer over this (3.0, [0x39a848]) is cut to
/// ONE_MORE (1.0).
const OUT_CUT: f32 = 3.0;
const ONE_MORE: f32 = 1.0;
/// FUN_0002f960's extra field of view (degrees, 0x41f00000) when state 2 asks.
const FLANK_EXTRA: f32 = 30.0;
/// DEBUG1's timer before it tries cover again (0x36adc / 0x36b1b: 5.0, 0x40a00000).
const VARY_TIME: f32 = 5.0;
/// The random spans (FUN_00023900(a, b), s), by where each is drawn: the arrival's hold
/// (0x3123b), the pop-up's peek (0x303c3), the open fire's hold (0x3042b), the step's (FUN_00047b40),
/// the time out (states 4 and 6, 0x30a33, 0x30ac7), the walk back's hold (FUN_00047f20), the hold
/// after the walk back (0x30fcf), after the return dodge (0x30b86) or its walk (0x30b5a), after the
/// pop-up (0x3087e) and the open fire (0x30e09), the hook's peek cuts (0x3153d, 0x3148e).
const SETTLE: (f32, f32) = (1.0, 2.5);
const POPUP: (f32, f32) = (5.5, 6.0);
const OPEN: (f32, f32) = (3.0, 4.0);
const STEP: (f32, f32) = (1.0, 2.0);
const OUT: (f32, f32) = (5.0, 7.0);
const BACK: (f32, f32) = (0.0, 3.0);
const BACK_HOME: (f32, f32) = (2.0, 3.0);
const RETURN_HOME: (f32, f32) = (1.75, 2.5);
const POPUP_END: (f32, f32) = (1.0, 2.5);
const OPEN_END: (f32, f32) = (0.0, 1.0);
const HIT_PEEK: (f32, f32) = (0.5, 1.0);
const SHOT_PEEK: (f32, f32) = (1.0, 2.0);
/// The hook's thresholds: a hit cuts a peek timer over 1 s, a shot at it one over 2 s; a shot at
/// it in state 2 keeps the hold timer at 1.5 s at least ([0x3a4f78], [0x3a5008], [0x3a4fb4]).
const HIT_PEEK_OVER: f32 = 1.0;
const SHOT_PEEK_OVER: f32 = 2.0;
const SHOT_HOLD: f32 = 1.5;
/// FUN_00040280 / FUN_00040510's cooldowns (s) when a dodge goes: the DODGE one (brain+0x104),
/// which FUN_00040280 waits out, and STUN 2, KNOCKDOWN 2 (play_enemies.rs's, [knock-down, stun]).
const DODGE_COOL: f32 = 3.0;
const DODGE_REACTION_COOL: [f32; 2] = [2.0, 2.0];
/// The dodge commands (the CT_ table 0x3bc9dc) and the motions the command handler 0x122040 plays
/// for them: CT_DODGE_LEFT (23) MT_DODGE_LEFT 15, _RIGHT (24) 16, CT_DODGE_SHORT_LEFT (33)
/// MT_DODGE_SHORT_LEFT 17, _RIGHT (34) 18 (its cases 0x122321-0x122357: by the names, nothing
/// mirrored; take06's Ugly's VARY dodge MT 18 moved it to its right).
pub const MT_DODGE_LEFT: i64 = 15;
pub const MT_DODGE_RIGHT: i64 = 16;
pub const MT_DODGE_SHORT_LEFT: i64 = 17;
pub const MT_DODGE_SHORT_RIGHT: i64 = 18;
/// The running side and back motions the view-facing walk plays (the MT_ table): MOVE_LEFT 26,
/// MOVE_LEFT_BACK 27, MOVE_LEFT_BACK_SLOW 28, MOVE_LEFT_SLOW 29, MOVE_RIGHT 30, MOVE_RIGHT_BACK 31,
/// MOVE_RIGHT_BACK_SLOW 32, MOVE_RIGHT_SLOW 33 (play_enemies.rs names the rest).
const MT_MOVE_LEFT: i64 = 26;
const MT_MOVE_LEFT_BACK: i64 = 27;
const MT_MOVE_RIGHT: i64 = 30;
const MT_MOVE_RIGHT_BACK: i64 = 31;
/// How far ahead along its way the walk's drive is aimed (play_goals.rs's CARROT, the demo's).
const CARROT: f32 = 5.0;
/// FUN_0004a3a0's threat bands (see `threat`): the rounds a second x damage x accuracy over 20, 10,
/// 5 (1.0, 0.5, 0.3, else 0.1); the Damage radius over 9, 6, 3, 0.01 m (x2.5, x2, x1.5, x1.25); the
/// gun's accuracy max under 80 (x2/3); bullet-type 2 (x1.25, the weapon data's +0x40); the
/// character's target the scorer (x1.5); the distance bands without a scope under 15 (x1.5, at
/// least 0.5), 15-35 (x1.1), over 50 (x0.75), with one over 60 (x1.5, at least 0.5), 15-35 (x0.75),
/// 35-60 (x1.1).
const THREAT_HITS: [(f32, f32); 3] = [(20.0, 1.0), (10.0, 0.5), (5.0, 0.3)];
const THREAT_LEAST: f32 = 0.1;
const THREAT_RADIUS: [(f32, f32); 4] = [(9.0, 2.5), (6.0, 2.0), (3.0, 1.5), (0.01, 1.25)];
const THREAT_ACCURACY: f32 = 80.0;
const THREAT_INACCURATE: f32 = 2.0 / 3.0;
const THREAT_LOBBED: i64 = 2;
const THREAT_LOB: f32 = 1.25;
const THREAT_MINE: f32 = 1.5;
const THREAT_NEAR: f32 = 15.0;
const THREAT_MID: f32 = 35.0;
const THREAT_FAR: f32 = 50.0;
const THREAT_SCOPED_FAR: f32 = 60.0;
const THREAT_CLOSE_LEAST: f32 = 0.5;
/// FUN_00036970: its own threat is scored at this distance (local_128 = 25.0), and it's
/// outnumbered when the target's is over twice it or the sum over its contacts over four times it.
const SELF_AT: f32 = 25.0;
const TARGET_OVER: f32 = 2.0;
const SUM_OVER: f32 = 4.0;
/// FUN_0004a280's filter 0x49bf0: a contact counts while seen with a clear ray or unseen under
/// 2.0 s ([0x3a5008]); its 3.5 s ([0x3a5340]) for a contact that dodges, is stunned, knocked down
/// or in cover isn't the demo's (no such goal on the player's side).
const SUM_UNSEEN: f32 = 2.0;
/// BF_TEST_TURN's reach when not given (m).
const TURN_REACH: f32 = 60.0;

/// brain+0x534's bits: 0 the cover blown (the hook's third hit, COVER_BLOWN without a new point:
/// the update's done), 1 / 2 a dodge out to the left / right, 3 / 4 a peek to the left / right
/// (FUN_00047b40 / FUN_00047d40), 5 the target seen while out (state 7), 7 the dodge out a short
/// one.
const BLOWN: u32 = 1 << 0;
const DODGED_LEFT: u32 = 1 << 1;
const DODGED_RIGHT: u32 = 1 << 2;
const PEEK_LEFT: u32 = 1 << 3;
const PEEK_RIGHT: u32 = 1 << 4;
const SEEN_OUT: u32 = 1 << 5;
const SHORT: u32 = 1 << 7;

/// A brain's cover fields (brain+0x510..+0x58c: FUN_0002f270, FUN_0002fd40, FUN_000313e0), its
/// crouch (brain+0x3cc), and GOAL_DEBUG1's (holder+0x758..+0x768). Kept with the brain
/// (play_brain.rs `Brain::cover`).
#[derive(Clone, Debug, Default)]
pub struct Cover {
    /// the goal whose start ran (ATTACK_COVER or DEBUG1): None, its start runs at the fight's next
    /// frame (a fight begun, or taken up again after the walk home)
    pub goal: Option<i64>,
    /// brain+0x510: the cover point held (claimed, its +0x28 bit 0), by its node
    pub point: Option<u32>,
    /// brain+0x514: the hold timer (s left; the update takes the frame's time off it at the point)
    hold: f32,
    /// brain+0x518: the state, 0 the walk, 1-10 at the point (see the module notes)
    state: u8,
    /// brain+0x51c: the step's direction (the point's right, or against it)
    step: Vec3,
    /// brain+0x528: where the step started (None: not yet)
    start: Option<Vec3>,
    /// brain+0x534: the flags (BLOWN, DODGED_LEFT ...)
    flags: u32,
    /// brain+0x538: the hits taken in state 2
    hits: u32,
    /// brain+0x53c: the time at the point since state 1 (s)
    at_cover: f32,
    /// brain+0x540: the pick's distance across to its point (m)
    pick_dist: f32,
    /// brain+0x544 / +0x548: the pick's radii, the re-pick's again
    radii: (f32, f32),
    /// brain+0x54c: the peek timer (s left; None stopped). Only state 7 counts it down (0x30c94):
    /// the pop-up (8) reads it and never does
    peek: Option<f32>,
    /// brain+0x55c: the re-pick timer (s left; None stopped), counted down at the point
    repick: Option<f32>,
    /// brain+0x570: the blacklist, {point, s left} (FUN_00048030, FUN_00048080)
    blacklist: Vec<(u32, f32)>,
    /// brain+0x57c: the last points held, the oldest first (FUN_000384a0, FUN_00038470)
    recent: VecDeque<u32>,
    /// brain+0x3cc: the crouch asked (FUN_0003f0e0, for a type with an MT_CROUCH motion)
    crouched: bool,
    /// the dodge motion the states 5 and 6 wait for (0x37790, the motion wait): its MT, and
    /// whether it has started
    dodge: Option<(i64, bool)>,
    /// where the character was at state 3's last frame (FUN_00025870's from: the walk back's pass
    /// test)
    last: Option<Vec3>,
    /// brain+0x104: the DODGE cooldown (s left; FUN_00040280 refuses while it runs)
    dodge_cool: f32,
    /// GOAL_DEBUG1's: ATTACK_VARY's half runs (holder+0x758 = 1), and its timer (holder+0x75c,
    /// s left; None stopped)
    vary: bool,
    vary_timer: Option<f32>,
}

impl Cover {
    /// Whether the cover goal runs now: ATTACK_COVER, or DEBUG1's cover half (the hook's test,
    /// DEBUG1's hook 0x36ca0 passing to ATTACK_COVER's while holder+0x758 is 0).
    pub fn holds(&self) -> bool {
        match self.goal {
            Some(GOAL_ATTACK_COVER) => true,
            Some(GOAL_DEBUG1) => !self.vary,
            _ => false,
        }
    }

    /// The frame's time on its own clocks: the blacklist's (FUN_00048080, from the AI update
    /// 0x1b10d, every frame whatever the goal), DEBUG1's timer (on the brain's timer list, brain+0x79c
    /// registered at 0x1746b) and the DODGE cooldown.
    pub fn tick(&mut self, dt: f32) {
        for b in self.blacklist.iter_mut() {
            b.1 -= dt;
        }
        self.blacklist.retain(|b| b.1 > 0.0);
        if let Some(t) = self.vary_timer.as_mut() {
            *t -= dt;
        }
        self.dodge_cool = (self.dodge_cool - dt).max(0.0);
    }
}

/// An enemy the brain knows (a kind-1 contact) as the cover goals see it (`Ctx`).
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    /// who
    pub who: Who,
    /// where it is now (its `Player::position`): the looks and the log
    pub at: Vec3,
    /// its node position, the hips (its vtable +0x40, 0x10fda0: the feet + the playing clip's
    /// root-height, play_missile.rs `root_point`): what the pick's enemy test (FUN_00026060), the
    /// flank test (FUN_0002f960), the 8 m rule and the threat score measure from
    pub hips: Vec3,
    /// where it was last seen (its entry's +0x3c, the sensor's): the peek's side once it was seen
    /// out of cover (FUN_000237b0)
    pub last: Vec3,
    /// seen now (its entry's +0x80 bit 5) with the ray clear (bit 3: the demo's sight has both in
    /// one)
    pub seen: bool,
}

/// What the goals see of the fight this frame (think's, play_brain.rs).
pub struct Ctx<'a> {
    /// the brain's name for the log
    pub label: &'a str,
    /// its own node position, the hips (its vtable +0x40: `Contact::hips`), which the pick
    /// (FUN_0002f270 hands it to the filter's 0x5e070), the 8 m rule and the step's probe measure
    /// from
    pub hips: Vec3,
    /// the goal's target (the brain's)
    pub target: Option<Contact>,
    /// the enemies it knows (kind-1 contacts)
    pub contacts: Vec<Contact>,
    /// its home area (FUN_00023630)
    pub leash: &'a goals::Leash,
    /// a squad type's brain (brain+0x3c4 bit 8: no recent deque) and a squad record (char+0x59c:
    /// the re-pick at 10 s)
    pub squad_type: bool,
    pub squad_record: bool,
    /// its type's motions (type+0x1a8 bits 8, 13, 10, 11 from its MOTION table, FUN_00130c50): a
    /// crouch (MT 6), a crouch walk (MT 7), the dodges, the short dodges
    pub crouches: bool,
    pub crouch_walks: bool,
    pub dodges: bool,
    pub short_dodges: bool,
    /// its hit points' share (FUN_000238e0: char+0x54 x 100 / char+0x50, truncated)
    pub hp_share: i32,
    /// the held gun's reach (FUN_00023d00's FUN_0001a760, play_reticle.rs `reach`)
    pub reach: f32,
    /// FUN_00036970's verdict on the target (`outnumbered`), worked out by the caller, and its
    /// numbers for the log (its own threat at 25 m, its target's, the sum over its contacts)
    pub outnumbered: bool,
    pub numbers: [f32; 3],
}

/// What the cover goal asks of the character this frame when it doesn't walk (think applies it):
/// the look (brain+0x488, a direction; None as it faces), fire at the target (FUN_00024f80 called),
/// crouched (brain+0x3cc), and a dodge motion to start (CT_DODGE_*: its MT).
#[derive(Clone, Copy, Debug, Default)]
pub struct Frame {
    pub look: Option<Vec3>,
    pub fire: bool,
    pub crouch: bool,
    pub dodge: Option<i64>,
}

/// One frame of the cover goals (`fight`): the character walked (no fire), stood as `Frame` says,
/// or ATTACK_VARY's turn (DEBUG1's other half: the caller runs play_goals.rs `fight`), or the goal
/// is over (ATTACK_COVER done or failed: the attack goal drawn again next frame).
pub enum Turn {
    Walked,
    Stand(Frame),
    Vary,
    Over,
}

/// Whether BF_COVER_LOG is set (1 the goal's events, 2 also the pick's candidates).
fn log_level() -> u8 {
    static LEVEL: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
    *LEVEL.get_or_init(|| std::env::var("BF_COVER_LOG").ok().map_or(0, |v| v.parse().unwrap_or(1)))
}

/// Whether goal `goal` is this module's (ATTACK_COVER, DEBUG1).
pub fn runs(goal: i64) -> bool {
    goal == GOAL_ATTACK_COVER || goal == GOAL_DEBUG1
}

/// FUN_00023900(a, b): a draw in [a, b) (the shared LCG 0x46cfd8; the demo's character draw).
fn rand(m: &mut Player, span: (f32, f32)) -> f32 {
    span.0 + m.unit() * (span.1 - span.0)
}

/// A point's name for the log.
fn label(g: &Graph, p: u32) -> String {
    let n = &g.nodes[p as usize];
    format!("h_{:08x} ({:.1}, {:.1})", n.name, n.at.x, n.at.z)
}

/// The start of goal `goal` when the attack draw takes it (FUN_00054c40's draw strikes out a goal
/// whose start refuses, FUN_00054bc0): ATTACK_COVER's (0x2f8e0, `pick`) or DEBUG1's (0x36a70,
/// `debug1_start`). True when it starts.
pub fn start(cv: &mut Cover, m: &mut Player, g: Option<&mut Graph>, goal: i64, ctx: &Ctx) -> bool {
    cv.goal = None;
    cv.vary = false;
    cv.vary_timer = None;
    let Some(g) = g else {
        // (no level points: no cover; DEBUG1 goes to ATTACK_VARY, `debug1_start`)
        if goal == GOAL_DEBUG1 && ctx.target.is_some() {
            cv.goal = Some(goal);
            to_vary(cv, m, ctx, "no world points");
            return true;
        }
        return false;
    };
    let ok = match goal {
        GOAL_ATTACK_COVER => pick(cv, m, g, ctx, MIN_RADIUS, MAX_RADIUS, Pick::Start),
        GOAL_DEBUG1 => debug1_start(cv, m, g, ctx),
        _ => false,
    };
    if ok {
        cv.goal = Some(goal);
    } else if log_level() > 0 || log_on() {
        println!("t={:.2} brain {}: {} refused (its start: {})", m.sim_time, ctx.label, name(goal),
                 if goal == GOAL_ATTACK_COVER { "no cover point to be had" } else { "no target" });
    }
    ok
}

/// The goal's name for the log.
fn name(goal: i64) -> &'static str {
    match goal {
        GOAL_ATTACK_COVER => "GOAL_ATTACK_COVER",
        GOAL_DEBUG1 => "GOAL_DEBUG1",
        _ => "a goal",
    }
}

/// Whether BF_BRAIN_LOG is set (the brains' log: the goals' starts and ends go there too).
fn log_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_BRAIN_LOG").is_ok())
}

/// GOAL_DEBUG1's start (0x36a70): no target, refused; FUN_00036970 (`Ctx::outnumbered`) says take
/// cover: ATTACK_COVER's start (`pick` at 0.75-30 m), holder+0x758 = 0; refused, or not
/// outnumbered: ATTACK_VARY's start (`to_vary`). (Corrects todo/26 leads §3.5, which read only the
/// update: the start tries cover only for a brain under half its hit points or outnumbered.)
fn debug1_start(cv: &mut Cover, m: &mut Player, g: &mut Graph, ctx: &Ctx) -> bool {
    if ctx.target.is_none() {
        return false;
    }
    if ctx.outnumbered && pick(cv, m, g, ctx, MIN_RADIUS, MAX_RADIUS, Pick::Start) {
        cv.vary = false;
        if log_level() > 0 || log_on() {
            println!("t={:.2} brain {}: GOAL_DEBUG1 starts in cover (FUN_00036970: hurt or outnumbered; {})", m.sim_time, ctx.label, threat_note(ctx));
        }
        return true;
    }
    to_vary(cv, m, ctx, if ctx.outnumbered { "no cover point to be had" } else { "not hurt or outnumbered (FUN_00036970)" });
    true
}

/// DEBUG1 turns to ATTACK_VARY (its start, the holder's +0x78: the demo's approach starts afresh,
/// play_goals.rs `fight`): holder+0x758 = 1 and its 5 s timer (FUN_000124f0(5.0) on holder+0x75c).
fn to_vary(cv: &mut Cover, m: &mut Player, ctx: &Ctx, why: &str) {
    cv.vary = true;
    cv.vary_timer = Some(VARY_TIME);
    m.goals.fight_paused();
    if log_level() > 0 || log_on() {
        println!("t={:.2} brain {}: GOAL_DEBUG1 runs ATTACK_VARY ({why}; {}); cover tried again in {VARY_TIME} s when hurt or outnumbered", m.sim_time, ctx.label,
                 threat_note(ctx));
    }
}

/// FUN_00036970's numbers for the log: the hit points' share, its own threat at 25 m, its
/// target's, the sum over its contacts.
fn threat_note(ctx: &Ctx) -> String {
    let [mine, theirs, sum] = ctx.numbers;
    format!("hp {}%, its threat at 25 m {mine:.2}, its target's {theirs:.2}, the sum {sum:.2}", ctx.hp_share)
}

/// ATTACK_COVER's end (0x2f900; DEBUG1's end 0x36b50 calls it too): the route reset
/// (FUN_0004e8f0), the point unclaimed and brain+0x510 = 0. The look and the speed record it
/// clears are the frame's in the demo. (Nothing stands it up: the demo's other goals stand by
/// themselves, play_testtools.rs `npc_ai`.)
pub fn end(cv: &mut Cover, m: &mut Player, g: Option<&mut Graph>) {
    if let Some(g) = g {
        routes::stop(m, g);
        if let Some(p) = cv.point {
            g.nodes[p as usize].state &= !nav::CLAIMED;
        }
    }
    cv.point = None;
    cv.goal = None;
    cv.vary = false;
    cv.vary_timer = None;
    cv.crouched = false;
    cv.dodge = None;
}

/// A dead character's brain stops: its goal's end lets the cover point go (`end`; the game's goal
/// machine ends the running goal as the brain is stopped, inferred: no point stays claimed by a body).
pub fn dead(m: &mut Player, g: Option<&mut Graph>) {
    let Some(mut b) = m.brain.take() else { return };
    if b.cover.point.is_some() {
        end(&mut b.cover, m, g);
    }
    m.brain = Some(b);
}

/// Who asks for a cover pick (the log's): a goal's start, the re-pick from state 2 (FUN_0002f270's
/// flag 1), DEBUG1's retry from ATTACK_VARY (each frame once its timer is out, as the game's: its
/// failures logged at BF_COVER_LOG=2 only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pick {
    Start,
    Again,
    Retry,
}

/// FUN_0002f270(brain, `min`, `max`, flag) for `m` (see the module notes): the target's reach
/// test, the flood, the route, the claim, the re-pick timer. `why`: who asks (the log's). True when
/// it found a point and a route to it.
fn pick(cv: &mut Cover, m: &mut Player, g: &mut Graph, ctx: &Ctx, min: f32, max: f32, why: Pick) -> bool {
    let again = why == Pick::Again;
    cv.radii = (min, max);
    let log = log_level();
    let now = m.sim_time;
    // (a target is needed, 0x2f444; seen with a clear ray, it must be within the held gun's
    // reach, FUN_00023d00, the two node positions apart, FUN_0001a850; the help call
    // EVT_ALARM_CLOSE_RANGE it posts first is #26 part F's)
    let Some(target) = ctx.target else { return false };
    let apart = target.hips.distance(ctx.hips);
    if target.seen && apart >= ctx.reach {
        if log > u8::from(why == Pick::Retry) {
            println!("t={now:.2} brain {}: no cover pick: its target {apart:.1} m off is beyond its gun's reach {:.1} m (FUN_00023d00)", ctx.label, ctx.reach);
        }
        return false;
    }
    let Some(own) = m.router.own.point else { return false };
    // (the filter measures from the character's node position, the hips, which FUN_0002f270 takes
    // from its vtable +0x40 at 0x2f2d0, and the enemies' likewise: take08's Ugly beside h_eed0869f
    // is 0.70 m from it by its feet and 0.93 m by its standing hips, so it takes the point)
    let me = ctx.hips;
    let enemies: Vec<Vec3> = ctx.contacts.iter().map(|c| c.hips).collect();
    // (the flood from the own point, FUN_000594a0; a brain out of the squad passes over its last
    // points on the first pass, +0x244 bit 2, and floods again without that, 0x2f42e)
    let first = !ctx.squad_type;
    let mut found = None;
    for recent in [first, false] {
        let gr: &Graph = g;
        found = gr.flood(own, max, |p| accepts(gr, cv, p, me, ctx.leash, &enemies, min, max, recent, log >= 2, ctx.label, now)).and_then(|path| path.last().copied());
        if found.is_some() || !recent {
            break;
        }
    }
    let Some(p) = found else {
        if log > u8::from(why == Pick::Retry) {
            println!("t={now:.2} brain {}: no cover point {} {min}-{max} m from it facing its {} enem{}", ctx.label, if again { "to move to" } else { "found" },
                     enemies.len(), if enemies.len() == 1 { "y" } else { "ies" });
        }
        return false;
    };
    if routes::plan(m, g, Dest::Point(p), "ATTACK_COVER: to its cover point").is_err() {
        if log > 0 {
            println!("t={now:.2} brain {}: no route to the cover point {}", ctx.label, label(g, p));
        }
        return false;
    }
    let at_p = g.nodes[p as usize].at;
    cv.state = 0;
    cv.pick_dist = at_p.xz().distance(me.xz());
    cv.flags &= !BLOWN;
    if let Some(old) = cv.point {
        g.nodes[old as usize].state &= !nav::CLAIMED;
    }
    cv.point = Some(p);
    g.nodes[p as usize].state |= nav::CLAIMED;
    cv.repick = Some(if ctx.squad_record { REPICK_SQUAD } else { REPICK });
    if log > 0 || log_on() {
        let cp = g.nodes[p as usize].cover_point;
        println!("t={now:.2} brain {}: {} cover point {}, {:.1} m off ({} entries), {:.1} m from its target, fov {:.0}, bits {:02x} (FUN_0002f270)", ctx.label,
                 if again { "moves to the" } else { "takes the" }, label(g, p), cv.pick_dist, m.router.walker.as_ref().map_or(0, |w| w.entries.len()),
                 at_p.distance(target.hips), cp.map_or(0.0, |c| c.fov), cp.map_or(0, |c| c.flags));
    }
    true
}

/// The cover filter 0x26240 (with its inner test 0x5e070 and FUN_00026060 for each enemy) on point
/// `p` for a character whose node position (the hips) is `me`, against enemies at `enemies` (theirs):
/// see the module notes. The distance test is 3D and takes both bounds (0x5e070: min^2 <= d^2 <=
/// max^2); nothing exempts a point the character stands by. `recent`: the first pass's +0x244 bit 2.
#[allow(clippy::too_many_arguments)]
fn accepts(g: &Graph, cv: &Cover, p: u32, me: Vec3, leash: &goals::Leash, enemies: &[Vec3], min: f32, max: f32, recent: bool, log: bool, who: &str,
         now: f32) -> bool {
    let n = &g.nodes[p as usize];
    if n.kind != nav::kind::COVER {
        return false;
    }
    let why = (|| {
        let Some(cp) = n.cover_point else { return Some("no cover record".to_string()) };
        if n.state & nav::VALID == 0 {
            return Some("not valid".into());
        }
        if Some(p) == cv.point {
            return Some("the point it holds".into());
        }
        if n.state & nav::CLAIMED != 0 {
            return Some("claimed".into());
        }
        let d2 = n.at.distance_squared(me);
        if d2 < min * min || d2 > max * max {
            return Some(format!("{:.2} m off", d2.sqrt()));
        }
        if !leash.inside(n.at) {
            return Some("outside its home area".into());
        }
        if !n.cover {
            return Some("its set-valid off".into());
        }
        if cv.blacklist.iter().any(|b| b.0 == p) {
            return Some("blacklisted".into());
        }
        if recent && cv.recent.contains(&p) {
            return Some("one of its last four".into());
        }
        for &e in enemies {
            if let Some(why) = faces(&cp, n.at, me, e) {
                return Some(why);
            }
        }
        None
    })();
    if log {
        println!("t={now:.2} brain {who}: cover point h_{:08x} ({:.1}, {:.1}): {}", n.name, n.at.x, n.at.z, why.as_deref().unwrap_or("taken"));
    }
    why.is_none()
}

/// FUN_00026060 for an enemy at `e`, the point `cp` at `at`, the character at `me` (both node
/// positions, the hips: their vtable +0x40): why the point won't do against that enemy, None when
/// it does (5-50 m from it, not past it, the enemy inside the cone, FUN_00023140: the angle between
/// the facing and the line to the enemy, 3D, under half the fov-angle).
fn faces(cp: &CoverPoint, at: Vec3, me: Vec3, e: Vec3) -> Option<String> {
    let d2 = at.distance_squared(e);
    if d2 > ENEMY_FAR * ENEMY_FAR || d2 < ENEMY_NEAR * ENEMY_NEAR {
        return Some(format!("{:.1} m from an enemy", d2.sqrt()));
    }
    let a = at - me;
    let b = e - me;
    let (aa, bb) = (a.length_squared(), b.length_squared());
    if bb < aa && bb > 0.0 && aa > 0.0 && a.dot(b) / (aa * bb).sqrt() > PAST_COS {
        return Some("past an enemy".into());
    }
    let angle = cone_angle(cp, at, e);
    if angle >= cp.fov * 0.5 {
        return Some(format!("an enemy {angle:.0} deg off its facing (cone {:.0})", cp.fov * 0.5));
    }
    None
}

/// FUN_00023140 with the arc cosine its callers take (degrees): the angle between point `cp`'s
/// facing and the line from `at` to `e` (3D; 0 for `e` on top of it).
fn cone_angle(cp: &CoverPoint, at: Vec3, e: Vec3) -> f32 {
    let to = e - at;
    if to.length_squared() <= 1.421_085_5e-14 {
        return 0.0;
    }
    cp.facing().dot(to.normalize()).clamp(-1.0, 1.0).acos().to_degrees()
}

/// FUN_0002f960(`extra`): whether an enemy the character sees (with FLANK_EXTRA more degrees of
/// field of view when `extra`: state 2's call) lies outside point `cp`'s cone (at or beyond half
/// the fov-angle off its facing; the enemy's node position, its vtable +0x40 at 0x2f9db). The
/// demo's sight test is play_brain.rs `Sensor::sees_at`.
fn flanked(m: &Player, cp: &CoverPoint, at: Vec3, ctx: &Ctx, extra: bool) -> Option<Who> {
    ctx.contacts.iter().find(|c| cone_angle(cp, at, c.hips) >= cp.fov * 0.5 && m.sensor.sees_at(m, c.at, if extra { FLANK_EXTRA } else { 0.0 })).map(|c| c.who)
}

/// The cover goal's state to `s`, logged with `why`.
fn set(cv: &mut Cover, s: u8, ctx: &Ctx, now: f32, why: &str) {
    if log_level() > 0 && cv.state != s {
        println!("t={now:.2} brain {}: cover state {} -> {s} ({why})", ctx.label, cv.state);
    }
    if s == 3 {
        cv.last = None;
    }
    cv.state = s;
}

/// FUN_0003f0e0(`on`): the crouch asked, for a type with an MT_CROUCH motion (type+0x1a9 bit 0).
fn crouch(cv: &mut Cover, ctx: &Ctx, on: bool) {
    if ctx.crouches {
        cv.crouched = on;
    }
}

/// One frame of DEBUG1 or ATTACK_COVER for `m` against `foe` (see the module notes). For DEBUG1's
/// ATTACK_VARY half the caller runs play_goals.rs `fight` (`Turn::Vary`).
pub fn fight(cv: &mut Cover, m: &mut Player, g: Option<&mut Graph>, goal: i64, ctx: &Ctx, dt: f32) -> Turn {
    let mut g = g;
    // (a goal whose start hasn't run: the fight's first frame, or the fight taken up again after
    // the walk home)
    if cv.goal != Some(goal) && !start(cv, m, g.as_deref_mut(), goal, ctx) {
        return Turn::Over;
    }
    if goal == GOAL_DEBUG1 && cv.vary {
        // (0x36c32-0x36c82: ATTACK_VARY's update, then with its timer out FUN_00036970 and cover's
        // start again; the demo tries before VARY's frame, the order a frame apart)
        let out = cv.vary_timer.is_some_and(|t| t < 0.0);
        if out && ctx.outnumbered {
            if let Some(g) = g.as_deref_mut() {
                if pick(cv, m, g, ctx, MIN_RADIUS, MAX_RADIUS, Pick::Retry) {
                    cv.vary = false;
                    if log_level() > 0 || log_on() {
                        println!("t={:.2} brain {}: GOAL_DEBUG1 takes cover again (FUN_00036970: hurt or outnumbered; {})", m.sim_time, ctx.label, threat_note(ctx));
                    }
                }
            }
        }
        if cv.vary {
            return Turn::Vary;
        }
    }
    let Some(g) = g else { return Turn::Vary };
    let (status, frame) = update(cv, m, g, ctx, dt);
    if log_level() >= 2 && (m.sim_time * 2.0).floor() != ((m.sim_time - dt) * 2.0).floor() {
        let d = ctx.target.map_or(-1.0, |t| t.at.distance(m.position));
        println!("cover t={:.2} {}: state {} point {} off {:.2} m hold {:.2} at {:.2} peek {:?} repick {:?} crouch {} action {:?} at ({:.2}, {:.2}) yaw {:.0} target {d:.1} m{}",
                 m.sim_time, ctx.label, cv.state, cv.point.map_or("-".to_string(), |p| label(g, p)),
                 cv.point.map_or(0.0, |p| g.nodes[p as usize].at.xz().distance(m.position.xz())), cv.hold, cv.at_cover, cv.peek.map(|t| (t * 100.0).round() / 100.0),
                 cv.repick.map(|t| t.round()), cv.crouched, m.action, m.position.x, m.position.z, m.yaw.to_degrees(),
                 if ctx.target.is_some_and(|t| t.seen) { " seen" } else { "" });
    }
    match status {
        goals::Status::Running => match frame {
            Some(f) => Turn::Stand(f),
            None => Turn::Walked,
        },
        _ => {
            let done = status == goals::Status::Done;
            if log_level() > 0 || log_on() {
                println!("t={:.2} brain {}: ATTACK_COVER {} at {}", m.sim_time, ctx.label, if done { "done" } else { "failed" },
                         cv.point.map_or("-".to_string(), |p| label(g, p)));
            }
            if goal == GOAL_DEBUG1 {
                // (0x36bfa: the cover over, ATTACK_VARY's start, holder+0x758 = 1, its timer 5 s;
                // the cover's end isn't called, its point stays claimed until DEBUG1's end or the
                // next pick)
                cv.crouched = false;
                to_vary(cv, m, ctx, if done { "the cover done" } else { "the cover failed" });
                Turn::Vary
            } else {
                end(cv, m, Some(g));
                Turn::Over
            }
        }
    }
}

/// FUN_0002fd40, ATTACK_COVER's update: the walk while its route has entries (FUN_00012620 on
/// brain+0x930), else the state machine at the point. The status (running, done 2, failed 0) and,
/// standing, what the character does (`Frame`; None when it walked).
fn update(cv: &mut Cover, m: &mut Player, g: &mut Graph, ctx: &Ctx, dt: f32) -> (goals::Status, Option<Frame>) {
    use goals::Status::{Done, Refused as Failed, Running};
    if cv.flags & BLOWN != 0 {
        return (Done, None);
    }
    let Some(p) = cv.point else { return (Failed, None) };
    if m.router.walking() {
        return walk_to(cv, m, g, p, ctx, dt);
    }
    let now = m.sim_time;
    let point = g.nodes[p as usize].at;
    let Some(cp) = g.nodes[p as usize].cover_point else { return (Failed, None) };
    // (the re-pick timer, 0x2fe51)
    if let Some(t) = cv.repick.as_mut() {
        *t -= dt;
    }
    if cv.state == 0 {
        return (Failed, None);
    }
    if let Some(who) = flanked(m, &cp, point, ctx, cv.state == 2) {
        if log_level() > 0 {
            println!("t={now:.2} brain {}: flanked at its cover point by {who:?} (FUN_0002f960): stands, done", ctx.label);
        }
        crouch(cv, ctx, false);
        return (Done, None);
    }
    // (the look on the point's facing, 0x2feb8; the hold timer down, 0x2ff02)
    let mut f = Frame { look: Some(cp.facing()), ..default() };
    cv.hold -= dt;
    let me = m.position;
    let off = me.xz().distance(point.xz());
    match cv.state {
        1 | 2 => {
            if cv.state == 1 {
                // (settle, 0x2ff2f: the counters, the point on the recent deque)
                cv.at_cover = 0.0;
                cv.hits = 0;
                cv.start = None;
                if !ctx.squad_type {
                    cv.recent.push_back(p);
                    while cv.recent.len() > RECENT {
                        cv.recent.pop_front();
                    }
                }
                set(cv, 2, ctx, now, "at its cover point");
            }
            // (the target seen within 8 m, after the first frame: the point blacklisted, failed; the
            // two node positions apart, 3D, their vtable +0x40 at 0x30085 and 0x3008e)
            if cv.at_cover > 0.0 {
                if let Some(t) = ctx.target.filter(|t| t.seen && t.hips.distance(ctx.hips) < TOO_NEAR) {
                    blacklist(cv, p);
                    if log_level() > 0 {
                        println!("t={now:.2} brain {}: its target {:?} {:.1} m off at its cover point: blacklisted, failed", ctx.label, t.who, t.hips.distance(ctx.hips));
                    }
                    return (Failed, None);
                }
            }
            cv.at_cover += dt;
            let crouch_point = cp.flags & CoverPoint::CROUCH != 0 && ctx.crouches && cp.flags & CoverPoint::DODGE == 0;
            if crouch_point {
                crouch(cv, ctx, true);
                if cp.flags & CoverPoint::CROUCH_FIRE != 0 && matches!(m.action, Action::Crouched) {
                    set(cv, 9, ctx, now, "a crouch-fire point, crouched");
                }
            } else {
                if cp.flags & CoverPoint::CROUCH != 0 {
                    crouch(cv, ctx, true);
                }
                if off > OFF_POINT {
                    if log_level() > 0 {
                        println!("t={now:.2} brain {}: {off:.2} m off its cover point: failed", ctx.label);
                    }
                    return (Failed, None);
                }
            }
            // (FUN_0004ad00: the nearest enemy seen with a clear ray; faced, fired at 35 degrees,
            // the hold timer kept)
            let seen = ctx.contacts.iter().filter(|c| c.seen).min_by(|a, b| a.at.distance(me).total_cmp(&b.at.distance(me))).copied();
            if let Some(c) = seen {
                let to = c.at - me;
                f.look = Some(Vec3::new(to.x, 0.0, to.z));
                f.fire = true;
                cv.hold += dt;
            }
            if cv.hold > 0.0 && cv.at_cover < AT_COVER {
                f.crouch = cv.crouched;
                return (Running, Some(f));
            }
            // (a re-pick with an enemy in sight, or the re-pick timer out; none: the timer again)
            if seen.is_some() || cv.repick.is_some_and(|t| t <= 0.0) {
                let (lo, hi) = cv.radii;
                if pick(cv, m, g, ctx, lo, hi, Pick::Again) {
                    f.crouch = cv.crouched;
                    return (Running, Some(f));
                }
                cv.repick = Some(if ctx.squad_record { REPICK_SQUAD } else { REPICK });
            }
            // (the peek)
            if crouch_point {
                cv.peek = Some(rand(m, POPUP));
                set(cv, 8, ctx, now, "the pop-up");
            } else {
                let (left, right) = (cp.flags & CoverPoint::LEFT != 0, cp.flags & CoverPoint::RIGHT != 0);
                if !left && !right {
                    cv.step = Vec3::ZERO;
                    cv.hold = rand(m, OPEN);
                    set(cv, 10, ctx, now, "no side to peek: fires from the point");
                } else {
                    let go_left = match (left, right) {
                        (true, false) => true,
                        (false, true) => false,
                        _ => {
                            // (the target's side, 0x30473-0x305ec: its place (its vtable +0x40), or
                            // with +0x534 bit 5, seen out there last time, where it was last seen
                            // (FUN_000237b0: the entry's +0x3c, carried on by its velocity +0x4c x
                            // its time unseen +0x10, which the demo's sensor doesn't keep); then
                            // the dot of (that - the point) with the point's right across, at or
                            // under 0 the left. No target: a coin's +-2 x the right, from which
                            // the point's place is taken away all the same (0x3057b-0x3058e), so
                            // far from the world's origin the place decides, as read)
                            let to = match ctx.target {
                                Some(t) => if cv.flags & SEEN_OUT != 0 { t.last } else { t.at },
                                None => cp.right() * if m.unit() <= 0.5 { -2.0 } else { 2.0 },
                            } - point;
                            to.x * cp.right().x + to.z * cp.right().z <= 0.0
                        }
                    };
                    peek(cv, m, g, p, &cp, ctx, go_left);
                    crouch(cv, ctx, false);
                }
            }
            cv.flags &= !SEEN_OUT;
        }
        3 => {
            // (the walk back, 0x30e31: the look as it faces, straight to the point at a run, a walk
            // within 1 m; done once it passes within 0.2 m of it)
            let last = cv.last.replace(me).unwrap_or(me);
            if nav::passed(last.xz(), me.xz(), point.xz(), BACK_RADIUS) || off <= BACK_RADIUS {
                cv.last = None;
                cv.hold = rand(m, BACK_HOME);
                set(cv, 1, ctx, now, "back at its cover point");
                f.look = None;
                f.fire = true;
                f.crouch = cv.crouched;
                return (Running, Some(f));
            }
            let mode = if off * off < BACK_WALK { WALK } else { RUN };
            let view = Vec3::new(-m.yaw.sin(), 0.0, -m.yaw.cos());
            walk(cv, ctx, m, None, point, mode, Some(view), dt);
            return (Running, None);
        }
        4 => {
            // (the step out, 0x308b2: on while the cover is in front, within 5 m along the
            // point's facing from the hips, or within 2 m of the point or 1.5 m of where it
            // started)
            let behind = ray_hit(ctx.hips, cp.facing(), PROBE).is_some();
            let start = *cv.start.get_or_insert(me);
            if behind || off < STEP_POINT || me.xz().distance(start.xz()) < STEP_START {
                walk(cv, ctx, m, None, me + cv.step * CARROT, RUN, Some(cp.facing()), dt);
                return (Running, None);
            }
            cv.hold = rand(m, OUT);
            cv.peek = Some(cv.hold);
            set(cv, 7, ctx, now, "stepped out");
        }
        5 | 6 => {
            // (the dodge motions' wait, 0x37790: on while the dodge plays)
            if dodging(cv, m) {
                return (Running, Some(f));
            }
            if cv.state == 6 {
                cv.peek = Some(rand(m, OUT));
                set(cv, 7, ctx, now, "dodged out");
            } else if off * off <= RETURN_NEAR * RETURN_NEAR {
                cv.hold = rand(m, RETURN_HOME);
                set(cv, 1, ctx, now, "dodged back to its cover point");
            } else {
                cv.hold = rand(m, BACK);
                set(cv, 3, ctx, now, "dodged back short of its cover point");
            }
        }
        7 => {
            // (out of cover, 0x30bb1)
            if let Some(t) = ctx.target {
                if t.seen {
                    cv.flags |= SEEN_OUT;
                } else if cv.flags & SEEN_OUT == 0 {
                    if cv.flags & SHORT != 0 {
                        // (a short dodge that showed it nothing: the point's bit 6, and the same
                        // side once more, 0x30bec)
                        g.nodes[p as usize].dodged = true;
                        cv.flags &= !(SEEN_OUT | SHORT);
                        if cv.flags & DODGED_RIGHT != 0 {
                            cv.flags &= !DODGED_RIGHT;
                            peek(cv, m, g, p, &cp, ctx, false);
                        } else if cv.flags & DODGED_LEFT != 0 {
                            cv.flags &= !DODGED_LEFT;
                            peek(cv, m, g, p, &cp, ctx, true);
                        }
                    } else if cv.peek.is_some_and(|t| t > OUT_CUT) {
                        // (the ray to the target cut: the demo's sight has the ray in "seen")
                        cv.peek = Some(ONE_MORE);
                    }
                }
            }
            if cv.state == 7 {
                if let Some(t) = cv.peek.as_mut() {
                    *t -= dt;
                }
                f.fire = true;
                if cv.peek.is_none_or(|t| t <= 0.0) {
                    if let Some(who) = flanked(m, &cp, point, ctx, false) {
                        if log_level() > 0 {
                            println!("t={now:.2} brain {}: flanked out of cover by {who:?} (FUN_0002f960): stands, done", ctx.label);
                        }
                        crouch(cv, ctx, false);
                        return (Done, None);
                    }
                    back_in(cv, m, ctx);
                }
            }
        }
        8 => {
            // (the pop-up, 0x306bd: stood up, standing: fire at 25 degrees, face the target; the
            // peek timer isn't counted down here)
            crouch(cv, ctx, false);
            if matches!(m.action, Action::None) {
                f.fire = true;
            }
            if let Some(t) = ctx.target {
                let to = t.at - me;
                f.look = Some(Vec3::new(to.x, 0.0, to.z));
            }
            if cv.peek.is_none_or(|t| t <= 0.0) {
                cv.hold = rand(m, POPUP_END);
                set(cv, 1, ctx, now, "the pop-up over");
            }
        }
        9 => {
            // (crouched fire, 0x30615: crouch, face the target, fire at 25 degrees; no way out
            // but the goal's exits)
            crouch(cv, ctx, true);
            if let Some(t) = ctx.target {
                let to = t.at - me;
                f.look = Some(Vec3::new(to.x, 0.0, to.z));
                f.fire = true;
            }
        }
        10 => {
            // (the open fire, 0x30ddd: fire at 25 degrees until the hold timer is out)
            f.fire = true;
            if cv.hold <= 0.0 {
                cv.hold = rand(m, OPEN_END);
                set(cv, 1, ctx, now, "the open fire over");
            }
        }
        _ => {}
    }
    // (a dodge just sent: its motion starts this frame, think applies it)
    if let Some((mt, false)) = cv.dodge {
        cv.dodge = Some((mt, true));
        f.dodge = Some(mt);
        f.look = None;
        f.fire = false;
    }
    f.crouch = cv.crouched;
    (Running, Some(f))
}

/// The walk to the cover point (FUN_0002fd40's first branch, see the module notes): arrived, the
/// hold timer and state 1 (the goal record's +0x88 bit 1); else the move, the look on the target
/// unless dashing. (A route that fails at its start returns 1 with the move zeroed: the next frame
/// is the state machine's, state 0, failed.)
fn walk_to(cv: &mut Cover, m: &mut Player, g: &mut Graph, p: u32, ctx: &Ctx, dt: f32) -> (goals::Status, Option<Frame>) {
    let point = g.nodes[p as usize].at;
    let far = point.xz().distance(m.position.xz());
    let dash = far > DASH_FAR && cv.pick_dist > DASH_PICK && ctx.hp_share < DASH_HP;
    let s = routes::step(m, g, WALK_RADIUS, LAST_RADIUS);
    if !s.walking {
        cv.hold = rand(m, SETTLE);
        set(cv, 1, ctx, m.sim_time, "arrived");
        return (goals::Status::Running, Some(Frame { crouch: cv.crouched, ..default() }));
    }
    // (the look on the target unless dashing, 0x31341-0x31386; its fire at 20 degrees is left out:
    // the demo's drive holds fire while it moves)
    let view = ctx.target.filter(|_| !dash).map(|t| t.at - m.position);
    walk(cv, ctx, m, Some(g), s.target, if dash { DASH } else { RUN }, view, dt);
    (goals::Status::Running, None)
}

/// FUN_0004d970(brain, the way to `target`, `mode`, 1, 1) with the look `view` (brain+0x488): the
/// crouch asked let go at a run or the dash, and at a walk for a type without MT_CROUCH_WALK
/// (0x4db45-0x4db66: FUN_0003f0e0(0) for a level over 2, or 2 without type+0x1a9 bit 5), so a
/// character crouched at one point doesn't crouch at the next for nothing; the steering
/// (play_avoid.rs `steer`), the pace near the route's end (`routes::pace_near`, with the level's
/// points) and its level, the turn (`avoid::move_turn`), and the drive (play_enemies.rs) in the
/// motion FUN_001195c0 picks for the move against the look (play_avoid.rs `pick`): a run (a walk,
/// the dash) along it, a side step or a back step with the body turned by the clip (MT_MOVE_LEFT /
/// _RIGHT, MT_BACKPEDAL, MT_MOVE_*_BACK; their walking _SLOW ones and MT_BACKSTEP at a walk), each
/// where the type has it, else the next one down (a running back motion to its walking one), else
/// straight on. No look: straight on, as play_goals.rs `go`. (A crouch kept at a walk is still the
/// demo's standing walk: think's walk lets the character's crouch go.)
#[allow(clippy::too_many_arguments)]
fn walk(cv: &mut Cover, ctx: &Ctx, m: &mut Player, g: Option<&Graph>, target: Vec3, mode: i32, view: Option<Vec3>, dt: f32) {
    let (dir, _) = avoid::steer(m, target);
    m.router.asked = mode;
    if let Some(g) = g {
        routes::pace_near(m, g);
    }
    let level = m.router.asked.min(m.router.cap);
    if level > WALK || (level == WALK && !ctx.crouch_walks) {
        crouch(cv, ctx, false);
    }
    let size = avoid::move_size(level);
    let way = avoid::move_turn(m, dir, size, dt);
    if level <= 1 || way.length_squared() < 1e-6 || routes::shore(m, way, dt) {
        brain::stop_walk(m);
        m.router.avoid.stand();
        return;
    }
    let dashing = level >= DASH && !routes::in_prism();
    let ahead = match level {
        WALK => enemies::MT_WALK,
        RUN => enemies::MT_RUN,
        _ if dashing => enemies::MT_DASH,
        _ => enemies::MT_RUN,
    };
    let walking = level == WALK;
    let mt = match view.filter(|v| v.xz().length_squared() > 1e-6) {
        None => ahead,
        Some(v) => match avoid::pick(avoid::yaw_of(v.xz()), m.yaw, avoid::yaw_of(way), true, dashing).motion {
            avoid::Motion::Run => ahead,
            avoid::Motion::Side { left } => match (left, walking) {
                (true, false) => MT_MOVE_LEFT,
                (false, false) => MT_MOVE_RIGHT,
                (true, true) => enemies::MT_MOVE_LEFT_SLOW,
                (false, true) => enemies::MT_MOVE_RIGHT_SLOW,
            },
            avoid::Motion::Back => if walking { enemies::MT_BACKSTEP } else { enemies::MT_BACKPEDAL },
            avoid::Motion::SideBack { left } => match (left, walking) {
                (true, false) => MT_MOVE_LEFT_BACK,
                (false, false) => MT_MOVE_RIGHT_BACK,
                (true, true) => enemies::MT_MOVE_LEFT_BACK_SLOW,
                (false, true) => enemies::MT_MOVE_RIGHT_BACK_SLOW,
            },
            avoid::Motion::Keep => m.enemy.drive.map_or(enemies::MT_BACKSTEP, |d| d.mt),
        },
    };
    let mt = fallback(m, mt, ahead);
    let goal = m.position.xz() + way.normalize_or_zero() * CARROT;
    if let Some(d) = m.enemy.drive.as_mut().filter(|d| d.mt == mt && d.ready) {
        d.goal = Some(goal);
    } else if !enemies::reacting(m) {
        enemies::drive(m, mt, Some(goal), true);
    }
}

/// The motion `mt` if `m`'s type has a clip for it, else the next one down (a running back or side
/// motion to its walking one, the back run to the back step), else `ahead`.
fn fallback(m: &Player, mt: i64, ahead: i64) -> i64 {
    let has = |mt: i64| m.loaded.as_ref().is_some_and(|l| l.motions.clip(mt, 0, true).is_some());
    let down = match mt {
        enemies::MT_BACKPEDAL => enemies::MT_BACKSTEP,
        MT_MOVE_LEFT_BACK => enemies::MT_MOVE_LEFT_BACK_SLOW,
        MT_MOVE_RIGHT_BACK => enemies::MT_MOVE_RIGHT_BACK_SLOW,
        MT_MOVE_LEFT => enemies::MT_MOVE_LEFT_SLOW,
        MT_MOVE_RIGHT => enemies::MT_MOVE_RIGHT_SLOW,
        _ => mt,
    };
    [mt, down].into_iter().find(|&t| has(t)).unwrap_or(ahead)
}

/// FUN_00047b40 (`left`) / FUN_00047d40 at point `cp` (node `p`): with the point's dodge and a draw
/// over 0.3, a short dodge on a second draw at most 0.3 (FUN_00040510: a type with the short
/// dodges, the point's bit 6 clear, not crouched) or else the dodge (FUN_00040280: a type with the
/// dodges, its cooldown out; its answer not looked at), state 6; otherwise the step out (state 4)
/// along the point's right (FUN_00047d40) or against it, the hold timer 1-2 s. The peek timer to
/// the hold timer when that's set (a dodge leaves the hold as it was), and the peek's side bit.
fn peek(cv: &mut Cover, m: &mut Player, g: &Graph, p: u32, cp: &CoverPoint, ctx: &Ctx, left: bool) {
    let now = m.sim_time;
    let side = if left { "left" } else { "right" };
    let mut dodged = false;
    if cp.flags & CoverPoint::DODGE != 0 && m.unit() > DODGE_ROLL {
        if m.unit() <= SHORT_ROLL {
            if ctx.short_dodges && !g.nodes[p as usize].dodged && !cv.crouched {
                send_dodge(cv, m, if left { MT_DODGE_SHORT_LEFT } else { MT_DODGE_SHORT_RIGHT });
                cv.flags |= SHORT | if left { DODGED_LEFT } else { DODGED_RIGHT };
                set(cv, 6, ctx, now, &format!("a short dodge {side}"));
                dodged = true;
            }
        } else if ctx.dodges {
            cv.flags = (cv.flags & !SHORT) | if left { DODGED_LEFT } else { DODGED_RIGHT };
            if cv.dodge_cool <= 0.0 {
                send_dodge(cv, m, if left { MT_DODGE_LEFT } else { MT_DODGE_RIGHT });
            }
            set(cv, 6, ctx, now, &format!("a dodge {side}"));
            dodged = true;
        }
    }
    if !dodged {
        cv.step = if left { -cp.right() } else { cp.right() };
        cv.step.y = 0.0;
        cv.step = cv.step.normalize_or_zero();
        cv.start = None;
        cv.hold = rand(m, STEP);
        set(cv, 4, ctx, now, &format!("steps out {side}"));
    }
    if cv.hold > f32::EPSILON {
        cv.peek = Some(cv.hold);
    }
    cv.flags |= if left { PEEK_LEFT } else { PEEK_RIGHT };
}

/// FUN_00047f20, back in: a dodge out that wasn't a short one is forgotten 70% of the time; one
/// left sends the opposite dodge (state 5); else the walk back (state 3), the hold timer 0-3 s.
fn back_in(cv: &mut Cover, m: &mut Player, ctx: &Ctx) {
    let now = m.sim_time;
    if cv.flags & (DODGED_LEFT | DODGED_RIGHT) != 0 && cv.flags & SHORT == 0 && m.unit() < FORGET_DODGE {
        cv.flags &= !(DODGED_LEFT | DODGED_RIGHT);
    }
    let back = if cv.flags & DODGED_LEFT != 0 {
        cv.flags &= !DODGED_LEFT;
        Some(if cv.flags & SHORT != 0 { MT_DODGE_SHORT_RIGHT } else { MT_DODGE_RIGHT })
    } else if cv.flags & DODGED_RIGHT != 0 {
        cv.flags &= !DODGED_RIGHT;
        Some(if cv.flags & SHORT != 0 { MT_DODGE_SHORT_LEFT } else { MT_DODGE_LEFT })
    } else {
        None
    };
    if let Some(mt) = back {
        let short = matches!(mt, MT_DODGE_SHORT_LEFT | MT_DODGE_SHORT_RIGHT);
        let ok = if short { ctx.short_dodges && !cv.crouched } else { ctx.dodges && cv.dodge_cool <= 0.0 };
        if ok {
            send_dodge(cv, m, mt);
            set(cv, 5, ctx, now, "dodges back");
            return;
        }
    }
    cv.hold = rand(m, BACK);
    set(cv, 3, ctx, now, "walks back");
}

/// FUN_00040280 / FUN_00040510's command: the dodge motion `mt` asked (brain+0x8b8; the motion
/// goal sends it: the demo's drive starts it this frame, `Frame::dodge`), and the cooldowns DODGE 3,
/// STUN 2, KNOCKDOWN 2 s (its DIVE one isn't the demo's; EVT_DEBUG2 to friends within 25 m, which
/// no personality lists, isn't posted).
fn send_dodge(cv: &mut Cover, m: &mut Player, mt: i64) {
    cv.dodge = Some((mt, false));
    cv.dodge_cool = DODGE_COOL;
    m.enemy.cooldowns = DODGE_REACTION_COOL;
}

/// A dodge the goal sent (`Frame::dodge`) that think couldn't start because a hit reaction plays
/// (play_enemies.rs `reacting`), logged at BF_COVER_LOG=1: the motion wait sees no dodge play and
/// the state goes on as if it had (the demo's: in the game a reaction is a goal of its own pushed
/// on top of the cover goal, which the demo's brain doesn't model).
pub fn dodge_dropped(m: &Player, label: &str, mt: i64) {
    if log_level() > 0 {
        println!("t={:.2} brain {label}: its dodge (MT {mt}) dropped: a hit reaction plays", m.sim_time);
    }
}

/// The motion wait (0x37790) for the dodge sent: true while it plays (asked this frame or the
/// last, or its motion on).
fn dodging(cv: &mut Cover, m: &Player) -> bool {
    match cv.dodge {
        Some((_, false)) => true,
        Some((mt, true)) => {
            let on = m.enemy.motion.is_some_and(|p| p.mt == mt) || m.enemy.drive.is_some_and(|d| d.mt == mt);
            if !on {
                cv.dodge = None;
            }
            on
        }
        None => false,
    }
}

/// FUN_00048030: point `p` on the blacklist for BLACKLIST s (its entry renewed if it's on it).
fn blacklist(cv: &mut Cover, p: u32) {
    match cv.blacklist.iter_mut().find(|b| b.0 == p) {
        Some(b) => b.1 = BLACKLIST,
        None => cv.blacklist.push((p, BLACKLIST)),
    }
}

/// The hook FUN_000313e0's EVT_DAMAGED case (a hit that reached the hit points): in state 2 the
/// hits are counted and the third blows the cover (the point blacklisted, the update done next
/// frame); in states 7-8 a peek timer over 1 s is cut to 0.5-1 s. (The handler takes the hit on
/// after: EVT_DAMAGED isn't one the hook keeps.)
pub fn hit(cv: &mut Cover, m: &mut Player, label: &str) {
    if !cv.holds() {
        return;
    }
    match cv.state {
        2 => {
            cv.hits += 1;
            if cv.hits >= HITS {
                cv.flags |= BLOWN;
                if let Some(p) = cv.point {
                    blacklist(cv, p);
                }
                if log_level() > 0 {
                    println!("t={:.2} brain {label}: hit {} times at its cover point: the cover blown, blacklisted", m.sim_time, cv.hits);
                }
            }
        }
        7 | 8 if cv.peek.is_some_and(|t| t > HIT_PEEK_OVER) => cv.peek = Some(rand(m, HIT_PEEK)),
        _ => {}
    }
}

/// The hook's EVT_SHOT_AT case (a shot passing close): in states 7-8 a peek timer over 2 s is cut
/// to 1-2 s; in state 2 the hold timer kept at 1.5 s at least. (The hook keeps the event: no dodge
/// is drawn for it while the cover goal runs.)
pub fn shot_at(cv: &mut Cover, m: &mut Player) {
    if !cv.holds() {
        return;
    }
    match cv.state {
        7 | 8 if cv.peek.is_some_and(|t| t > SHOT_PEEK_OVER) => cv.peek = Some(rand(m, SHOT_PEEK)),
        2 if cv.hold < SHOT_HOLD => cv.hold = SHOT_HOLD,
        _ => {}
    }
}

/// Whether the cover goal's hook keeps event `event` from the brain (FUN_000313e0 returns 1, and
/// the message handler 0x3dce0 returns at once, before its own cases and the personality's draw):
/// EVT_DAMAGE_LIGHT, EVT_COVER_BLOWN (a re-pick), EVT_SHOT_AT, the two shot blockages,
/// EVT_TGT_TOO_CLOSE, and EVT_TGT_ENTRENCHED but in states 3, 4, 7 and 8 (out of cover, where the
/// grenade may come). Every other event (EVT_DAMAGED among them) goes on to the handler. take07
/// bears it out: Tex put 1.8 m from the Ugly crouched at its cover point, TOO_CLOSE posted at 10.11 s
/// drew nothing, the next at 11.74 s (the cover failed at 8 m meanwhile) drew ATTACK_FALLBACK at
/// 11.86 s. For the events' posters of #26's other parts (the fallback's TOO_CLOSE, the grenades').
pub fn takes(cv: &Cover, event: i64) -> bool {
    if !cv.holds() {
        return false;
    }
    match event {
        EVT_DAMAGE_LIGHT | EVT_COVER_BLOWN | EVT_SHOT_AT | EVT_SHOT_BLOCKAGE | EVT_SHOT_BLOCKED_BY_FRIEND | EVT_TGT_TOO_CLOSE => true,
        EVT_TGT_ENTRENCHED => !matches!(cv.state, 3 | 4 | 7 | 8),
        _ => false,
    }
}

/// What FUN_0004a3a0 reads of a character, gathered for every character each frame (`profiles`):
/// who, its node position (the hips: its vtable +0x40, which the score's distance is taken between,
/// and the cover goals' `Contact::hips`), alive, its held gun's numbers (None without one), its
/// accuracy (FUN_00024ae0) and its brain's target (FUN_00049ca0).
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub who: Who,
    pub hips: Vec3,
    alive: bool,
    gun: Option<Gun>,
    accuracy: f32,
    target: Option<Who>,
}

/// A held gun's numbers for the threat score (the weapon data, the record at weapon+0x228): its
/// reach (FUN_00232520, play_reticle.rs `reach`), rounds a second (+0xcc the rate x +0xdc the
/// burst count when set), the Damage max (+0x18), radius (+0x24), its accuracy max (+8, an int,
/// percent), a zoom (+0x16c, h_efce7f77: "scoped") and its bullet-type (+0x40).
#[derive(Clone, Copy, Debug)]
struct Gun {
    reach: f32,
    rounds: f32,
    damage: f32,
    radius: f32,
    accuracy_max: f32,
    scoped: bool,
    bullet: i64,
}

/// Every character's `Profile` this frame: the controlled one first, then `squad` in order.
pub fn profiles(player: &Player, squad: &[Player], game: &Game, n: i64) -> Vec<Profile> {
    std::iter::once(player).chain(squad.iter()).map(|u| {
        let gun = u.loaded.as_ref().and_then(|l| l.weapons.get(u.weapon)).map(|w| {
            let d = &w.def;
            Gun {
                reach: reticle::reach(u, d),
                rounds: d.rate * d.shots.burst_count.max(1) as f32,
                damage: d.damage,
                radius: d.blast_radius,
                accuracy_max: d.shots.accuracy[1],
                scoped: d.zoom > 0.0,
                bullet: d.shots.bullet_type,
            }
        });
        Profile { who: u.who(), hips: missile::root_point(u), alive: !u.dead && !u.absent, gun, accuracy: brain::accuracy_of(u, game, n), target: brain::target(u) }
    }).collect()
}

/// FUN_0004a3a0(brain, `u`, `d`): `u`'s threat as the brain of `me` sees it at `d` m (see the
/// constants): by its gun within its reach, the gun's blast, its accuracy max and bullet, x1.5 when
/// its target is `me`, then by the distance. 0 for one dead.
fn threat(u: &Profile, me: Who, d: f32) -> f32 {
    if !u.alive {
        return 0.0;
    }
    let mut v = 0.0;
    let mut scoped = false;
    if let Some(g) = u.gun {
        if d < g.reach {
            let x = u.accuracy * g.rounds * g.damage;
            v = THREAT_HITS.iter().find(|b| x > b.0).map_or(THREAT_LEAST, |b| b.1);
        }
        v *= THREAT_RADIUS.iter().find(|b| g.radius > b.0).map_or(1.0, |b| b.1);
        if g.accuracy_max < THREAT_ACCURACY {
            v *= THREAT_INACCURATE;
        }
        scoped = g.scoped;
        if g.bullet == THREAT_LOBBED {
            v *= THREAT_LOB;
        }
    }
    if u.target == Some(me) {
        v *= THREAT_MINE;
    }
    if scoped {
        if d > THREAT_SCOPED_FAR {
            return (v * THREAT_MINE).max(THREAT_CLOSE_LEAST);
        }
        if d <= THREAT_MID {
            return if d > THREAT_NEAR { v * 0.75 } else { v };
        }
    } else {
        if d < THREAT_NEAR {
            return (v * THREAT_MINE).max(THREAT_CLOSE_LEAST);
        }
        if d >= THREAT_MID {
            return if d > THREAT_FAR { v * 0.75 } else { v };
        }
    }
    v * 1.1
}

/// FUN_00036970(target) for `m` (its own `Profile` in `all`): under half its hit points, or its
/// target's threat (`threat` at its distance, the two node positions apart) over twice its own at
/// 25 m, or the sum over its contacts seen in the last 2 s (FUN_0004a280) over four times its own.
/// The verdict and the numbers (own, the target's, the sum; the sum -1 with none, as FUN_0004a280
/// starts it).
pub fn outnumbered(m: &Player, target: Option<Who>, all: &[Profile], contacts: &[(Who, f32)]) -> (bool, [f32; 3]) {
    let me = m.who();
    let find = |w: Who| all.iter().find(|p| p.who == w);
    let hips = find(me).map_or(m.position, |p| p.hips);
    let mine = find(me).map_or(0.0, |p| threat(p, me, SELF_AT));
    let theirs = target.and_then(find).map_or(0.0, |p| threat(p, me, p.hips.distance(hips)));
    let mut sum = -1.0_f32;
    for &(w, _) in contacts.iter().filter(|c| c.1 < SUM_UNSEEN) {
        if let Some(p) = find(w).filter(|p| p.alive) {
            sum = sum.max(0.0) + threat(p, me, p.hips.distance(hips));
        }
    }
    let hurt = m.max_health != 0.0 && m.health < m.max_health * 0.5;
    (hurt || theirs > TARGET_OVER * mine || sum > SUM_OVER * mine, [mine, theirs, sum])
}

/// BF_TEST_TURN=<s>[,<r>]: at time s (the controlled character's clock) every level character with a
/// brain within r m (TURN_REACH by default) of the controlled character turns to face it, at once,
/// as the takes' trainer turned the mutants (`!yaw`, char+0x434: todo/132-mutants notes, "turned to
/// face Tex at 1.0 s"), so its sensor can see him (60 degrees either way for the mutants). Once.
fn turn_hook(player: Res<Player>, mut squad: ResMut<Squad>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Some(v) = std::env::var("BF_TEST_TURN").ok() else { return };
    let mut parts = v.split(',').map(|x| x.trim().parse::<f32>().ok());
    let Some(at) = parts.next().flatten() else { return };
    let reach = parts.next().flatten().unwrap_or(TURN_REACH);
    if player.sim_time < at {
        return;
    }
    *done = true;
    for u in squad.0.iter_mut().filter(|u| u.brain.is_some() && !u.dead && !u.absent) {
        let to = player.position - u.position;
        if to.xz().length() > reach || to.xz().length_squared() < 1e-6 {
            continue;
        }
        u.yaw = (-to.x).atan2(-to.z);
        u.cam_yaw = u.yaw;
        u.face_yaw = Some(u.yaw);
        if let Some(n) = u.npc.as_mut() {
            n.face = u.yaw;
        }
        if log_level() > 0 || log_on() {
            println!("t={:.2} BF_TEST_TURN: {} turned to face the controlled character {:.1} m off", player.sim_time, unit_name(u), to.length());
        }
    }
}

/// One BF_TEST_PUT entry: when (s), whose brain (its log name), where (x, z), the heading
/// (degrees, the camera's yaw: 0 faces -z), its hit points (now and at most), done.
struct Put {
    at: f32,
    name: String,
    x: f32,
    z: f32,
    heading: Option<f32>,
    hp: Option<f32>,
    done: bool,
}

/// BF_TEST_PUT=<s>,<name>,<x>,<z>[,<heading>[,<hp>]][;...]: at time s (the controlled character's
/// clock) the level character whose brain is named `name` (BF_BRAIN_LOG's name: `h_eef1a370` for
/// e23's Ugly 83302960) is put on the floor at (x, z) (the first under 50 m above where it stands),
/// facing the heading (degrees; 0 faces -z, 90 -x; empty: as it faced), its route dropped, and given
/// `hp` hit points now and at most when that's set: the takes' trainer `!place` and `hp`
/// (todo/132-mutants notes, take06's Ugly at (-64.1, 46.8)). Each entry once; a bad one is logged
/// and skipped.
fn put_hook(player: Res<Player>, mut squad: ResMut<Squad>, mut nav: ResMut<super::nav::Nav>, mut puts: Local<Option<Vec<Put>>>) {
    let puts = puts.get_or_insert_with(|| {
        let Ok(v) = std::env::var("BF_TEST_PUT") else { return vec![] };
        v.split(';').map(str::trim).filter(|e| !e.is_empty()).filter_map(|e| {
            let parts: Vec<&str> = e.split(',').map(str::trim).collect();
            let num = |i: usize| parts.get(i).and_then(|x| x.parse::<f32>().ok());
            match (num(0), parts.get(1), num(2), num(3)) {
                (Some(at), Some(name), Some(x), Some(z)) => Some(Put { at, name: name.to_string(), x, z, heading: num(4), hp: num(5), done: false }),
                _ => {
                    println!("BF_TEST_PUT: \"{e}\" isn't <s>,<name>,<x>,<z>[,<heading>[,<hp>]]: skipped");
                    None
                }
            }
        }).collect()
    });
    let now = player.sim_time;
    for p in puts.iter_mut().filter(|p| !p.done && now >= p.at) {
        p.done = true;
        let Some(u) = squad.0.iter_mut().find(|u| u.brain.as_ref().is_some_and(|b| b.label() == p.name) && !u.dead) else {
            println!("BF_TEST_PUT: no level character named \"{}\": skipped", p.name);
            continue;
        };
        if let Some(g) = nav.graph.as_mut() {
            routes::stop(u, g);
        }
        brain::stop_walk(u);
        let feet = floor_y(p.x, p.z, u.position.y + 50.0, world::Mover::of(false).body());
        u.position = Vec3::new(p.x, feet - GROUND, p.z);
        u.prev_xz = Vec2::new(p.x, p.z);
        u.fall_from = feet;
        u.vy = 0.0;
        u.enemy.velocity = Vec3::ZERO;
        if let Some(h) = p.heading {
            u.yaw = h.to_radians();
            u.cam_yaw = u.yaw;
            u.face_yaw = Some(u.yaw);
            if let Some(n) = u.npc.as_mut() {
                n.face = u.yaw;
            }
        }
        if let Some(hp) = p.hp.filter(|&hp| hp > 0.0) {
            u.health = hp;
            u.max_health = hp;
        }
        println!("t {now:.2}: BF_TEST_PUT: {} put at ({:.1}, {feet:.1}, {:.1}) facing {:.0} deg, {:.0} of {:.0} hp", p.name, p.x, p.z, u.yaw.to_degrees(), u.health,
                 u.max_health);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::level::WorldPoint;

    /// e23's h_157fd721 (take06's cover point): its ORIENTATION, fov 100, enable-left, h_18ed19af,
    /// h_f9f3561f and enable-dodge.
    fn take06_point() -> CoverPoint {
        CoverPoint { orientation: [-0.900319, 0.0, -0.435231, 0.0, 1.0, 0.0, 0.435231, 0.0, -0.900319], fov: 100.0, fov_distance: 300.0, flags: 0x1d }
    }

    /// The facing and the right of take06's point: toward the road Tex stood on, the right a quarter
    /// clockwise from it seen from above.
    #[test]
    fn a_cover_points_facing_and_right() {
        let cp = take06_point();
        let f = cp.facing();
        assert!((f - Vec3::new(0.435231, 0.0, 0.900319)).length() < 1e-5);
        let r = cp.right();
        assert!((r - Vec3::new(-0.900319, 0.0, 0.435231)).length() < 1e-5);
        // (F x R points down: R is F's right with y up)
        assert!(f.cross(r).y < -0.99);
    }

    /// FUN_00026060 on take06's point with Tex where he stood (both at their hips, the feet of
    /// take06's watch + about 0.93 m): 13.7 m off, 13 degrees into the 50-degree half cone, the Ugly
    /// 9.5 m from the point on the far side: taken. Tex at 4 m from the point, past it, or outside
    /// its cone: not.
    #[test]
    fn the_enemy_test() {
        let cp = take06_point();
        let at = Vec3::new(-66.62, 20.09, 37.66);
        let ugly = Vec3::new(-64.12, 20.46, 46.83);
        let tex = Vec3::new(-63.59, 20.48, 51.01);
        assert_eq!(faces(&cp, at, ugly, tex), None);
        assert!(faces(&cp, at, ugly, at + Vec3::new(0.0, 0.0, 4.0)).is_some(), "under 5 m");
        // (an enemy nearer than the point, within 75 degrees of its bearing: past it)
        let me = Vec3::new(-66.0, 19.5, 60.0);
        assert!(faces(&cp, at, me, Vec3::new(-66.0, 19.5, 52.0)).unwrap().contains("past"));
        // (60 degrees off the facing: outside the 50-degree half cone)
        let side = at + Vec3::new(10.0, 0.0, -2.0);
        assert!(faces(&cp, at, ugly, side).unwrap().contains("off its facing"));
    }

    /// FUN_0004a3a0's bands: the Ugly's gun (1.9 a second, 13, accuracy 0.3) scores 0.3, x1.1 at 25 m;
    /// a sharper gun within 15 m scores 1.5; anyone within 15 m scores 0.5 at least; a scoped gun 15-35
    /// m off x0.75.
    #[test]
    fn the_threat_score() {
        let me = Who::Npc(1, 1);
        let gun = Gun { reach: 80.0, rounds: 1.9, damage: 13.0, radius: 0.0, accuracy_max: 90.0, scoped: false, bullet: 0 };
        let ugly = Profile { who: me, hips: Vec3::ZERO, alive: true, gun: Some(gun), accuracy: 0.3, target: None };
        assert!((threat(&ugly, me, 25.0) - 0.33).abs() < 1e-5);
        let tex = Profile { who: Who::Squad(3), gun: Some(Gun { rounds: 8.0, damage: 12.0, ..gun }), accuracy: 0.6, ..ugly };
        assert!((threat(&tex, me, 4.2) - 1.5).abs() < 1e-5);
        assert!((threat(&tex, me, 18.9) - 1.1).abs() < 1e-5);
        let unarmed = Profile { gun: None, ..tex };
        assert_eq!(threat(&unarmed, me, 10.0), 0.5);
        assert_eq!(threat(&unarmed, me, 20.0), 0.0);
        let sniper = Profile { gun: Some(Gun { scoped: true, ..gun }), accuracy: 1.0, ..tex };
        assert!((threat(&sniper, me, 20.0) - 0.75).abs() < 1e-5);
        assert_eq!(threat(&Profile { alive: false, ..tex }, me, 4.0), 0.0);
    }

    /// The pick's 0.75 m is 3D from the character's hips (0x5e070 on FUN_0002f270's vtable +0x40
    /// position) and takes its bound: take08's Ugly beside h_eed0869f (its watch's feet; the point
    /// from levels-e23.xml), 0.70 m from it by the feet, 0.93 m by the standing hips (the mutants'
    /// idle root-height 0.928): refused by the one, taken by the other; at exactly 0.75, taken.
    #[test]
    fn the_pick_measures_from_the_hips() {
        let cp = CoverPoint { orientation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], fov: 100.0, fov_distance: 300.0, flags: 0x15 };
        let graph = |at: Vec3| Graph::new(&[WorldPoint { name: 0xEED0_869F, kind: 0x126F_47A3, at, flags: 0xffff_0000, radius: -1.0, links: vec![], state: WorldPoint::VALID,
                                                         cover: Some(cp) }]);
        let (cv, leash) = (Cover::default(), goals::Leash::default());
        let takes = |g: &Graph, me: Vec3| accepts(g, &cv, 0, me, &leash, &[], MIN_RADIUS, MAX_RADIUS, false, false, "test", 0.0);
        let g = graph(Vec3::new(-63.479, 19.790, 46.884));
        let feet = Vec3::new(-64.122, 19.530, 46.829);
        assert!(!takes(&g, feet), "0.70 m from the feet");
        assert!(takes(&g, feet + Vec3::Y * 0.928), "0.93 m from the hips");
        // (a place whose numbers are exact in binary, so the distance is 0.75 to the last bit)
        let at = Vec3::new(-63.5, 19.75, 46.875);
        assert!(takes(&graph(at), at + Vec3::X * MIN_RADIUS), "the bound itself");
    }

    /// The hook keeps what FUN_000313e0 returns 1 for, and only while the cover goal runs.
    #[test]
    fn what_the_hook_keeps() {
        let mut cv = Cover { goal: Some(GOAL_ATTACK_COVER), state: 2, ..default() };
        assert!(takes(&cv, EVT_TGT_TOO_CLOSE) && takes(&cv, EVT_SHOT_AT) && takes(&cv, EVT_DAMAGE_LIGHT));
        assert!(takes(&cv, EVT_TGT_ENTRENCHED), "hidden at the point");
        cv.state = 7;
        assert!(!takes(&cv, EVT_TGT_ENTRENCHED), "out of cover");
        assert!(!takes(&cv, 13), "EVT_DAMAGED goes on");
        cv.goal = Some(GOAL_DEBUG1);
        cv.vary = true;
        assert!(!takes(&cv, EVT_TGT_TOO_CLOSE), "DEBUG1's ATTACK_VARY half");
    }

    /// The blacklist's 10 s (FUN_00048030's 10.0, FUN_00048080's count down).
    #[test]
    fn the_blacklist_runs_out() {
        let mut cv = Cover::default();
        blacklist(&mut cv, 7);
        cv.tick(9.9);
        assert_eq!(cv.blacklist.len(), 1);
        cv.tick(0.2);
        assert!(cv.blacklist.is_empty());
    }
}
