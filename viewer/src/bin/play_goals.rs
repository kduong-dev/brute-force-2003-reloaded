//! The level characters' goals on routes (#336, the third slice of #242; viewer/docs/brain.md "Its
//! goals on routes", nav.md "The enemies on routes"): the brains of the characters a level's
//! spawn-triggers make walk their goals over the level's world points (bf/nav.rs, the squad's route
//! code in play_routes.rs) as default.xbe's goals do, and a test map NPC set to fight closes in on
//! its enemy as the attack goals' approach does. Every function cited is in
//! viewer/docs/code-map.md, "AI"; the reading is todo/242-ai-routes/leads.md §3-§5, with
//! todo/132-mutants/leads.md §2 (the stand-off) and todo/83-fire-breath/leads.md §2.
//!
//!  - **The route record** is the brain's one, brain+0x930 (`Router::walker`): the goal holder's
//!    +0x8f0, which RETURN_HOME, PATROL's first leg, PURSUE and the attack approach hand
//!    FUN_00026d80. So the AI update's route steps run on it whatever the goal (play_routes.rs
//!    `ai_update`: the catch-up FUN_00041ff0, the 2 s stuck handling FUN_000426b0 with, for a brain
//!    out of the squad, its GOAL_ATTACK_GUARD branch FUN_00042620, `stuck_guard`).
//!  - **GOAL_RETURN_HOME** (53, start 0x345d0, update FUN_00034690; `home`): a route to the home
//!    point, refused (brain+0x3c4 bit 11, for good) when the home point's +0x24 bit 15 differs from
//!    the own point's or there's no route; walked at 2 m at the spawn's `locomotion`; at its end the
//!    home area's leash goes on (brain+0x21c bit 1).
//!  - **GOAL_PATROL** (4, start FUN_0002af70, update FUN_0002b490; `patrol_start`, `patrol`): the
//!    chain as before (brain+0x958), and its first leg a route to the valid point FUN_0004c2e0
//!    finds visible within 20 m of the chain's current patrol point (`Graph::visible_near`); with
//!    none, or no route to it, the start refuses. The chain is walked once the leg is over, its ends
//!    by FUN_00026940 (`nav::ChainEnd`).
//!  - **The attack goals' approach** (GOAL_ATTACK 7: start FUN_0002e490, update FUN_0002e9c0;
//!    ATTACK_VARY 8: start FUN_0002c9d0, update FUN_0002d710; `fight`): approach 1, a route to the
//!    target's own point whose length is under a budget (FUN_0002e800: 2.5 x the distance under 10
//!    m, 2 under 25, 1.75 under 50, else 1.5, x1.2 over a 10 m height difference; the search held
//!    within it, DAT_004021a8), walked without firing at the dash beyond the stand-off (FUN_00023e40:
//!    min(the type's h_150b4e02, the gun's reach)) and at a run inside it, re-planned every 4 s,
//!    until within (the near bound + 3 x the stand-off) / 4 of the target (26.25 m for the Red Hand,
//!    the hounds and the mutants), where it stands and fires (approach 2) until the target is
//!    beyond the stand-off again. A route over its budget, or none, marks the target unreachable
//!    for 2 s (its contact's +0x80 bit 6, +0x84): it stands and fires meanwhile.
//!  - **GOAL_PURSUE** (10, start 0x2c630, update FUN_0002c730; `fight`): a route to the target's own
//!    point when the target is inside the home leash (FUN_00023630), dashed, planned again at its
//!    end, done as soon as a contact is seen.
//!  - **The home area** (`Leash`: the spawn-trigger's h_f63b86a2 / h_062fbaf0, at least 7 m, round
//!    the home point; on at home, or from the spawn without a home-point): outside it the sense step
//!    posts EVT_PAST_HOME_AREA (FUN_00044480), whose GOAL_PATH_TOWARD_HOME (106, 20.19, over
//!    EVT_ENEMY_SEEN's 18; `past_home`, `toward_home`) runs RETURN_HOME's route at a run, without
//!    firing, until it's more than 7 m inside (FUN_000347b0 with FUN_000235d0) or the route ends;
//!    ATTACK_VARY's approach stops within 1 m of the area's edge with its target outward
//!    (FUN_00024080, `Leash::at_edge`).
//!  - **What the demo's brain draws**: the attack goal is its personality's EVT_ENEMY_SEEN draw
//!    (FUN_00054c40, once a fight); ATTACK_COVER (19) and DEBUG1's (120) cover choice are #26's, so
//!    the demo takes their no-cover path (ATTACK_COVER refused and struck out, the draw's next goal
//!    ATTACK_VARY; DEBUG1 running ATTACK_VARY): ATTACK_VARY's approach; its state-2 moves (strafes,
//!    dodges) are #26's, it stands and fires. GOAL_PURSUE runs when the ray to the target has met
//!    something for LOST_SIGHT s (FUN_0002e330's EVT_TGT_LOST_SIGHT, whose entry's
//!    REPOSITION_LOS_TO_TGT is #26's) for a personality whose entry lists it. The demo's brain
//!    has no INVESTIGATE, GOTO, HIDE, SNIPE or
//!    SCOUT goal, and posts no event with a point (no AI command triggers), so GOAL_IDLE's start
//!    has none to route to.
//!  - **The move** (`go`): the steering target through the AI's avoidance (play_avoid.rs `steer`,
//!    FUN_000255d0), the pace near the route's points (FUN_0004d840, `routes::pace_near`), the move
//!    at the mode asked held to the pace level (FUN_0004d970) and turned at 200 degrees a second
//!    (FUN_00019af0); an enemy type moves by its type's speeds (play_enemies.rs's drive,
//!    FUN_0012c6c0: a walk, a run or the dash by the pace).
//!
//! The demo's stall guess (a walk given up after gaining under 0.1 m in 1 s) is gone: a walk that
//! can't be routed refuses its goal, as the game's starts do, and a stuck one is the stuck
//! handling's.
//!
//! Test hooks: BF_BRAIN_LOG=1 (each goal's start, refusal, end; the attack goal drawn; the approach's
//! budget and its stops; the pursuit; ATTACK_GUARD; the home area's size at the spawn, leaving it,
//! its edge); BF_ROUTE_LOG=1 (each route planned and walked,
//! play_routes.rs); BF_GOAL_WATCH=<s> (every s seconds of the game's clock, a row for each level
//! character and test map NPC set to fight: its goal, where it is, its own point, its route's entry,
//! its target and how far, the approach's state).

use bf_viewer::bf::character::Game;
use bf_viewer::bf::level::{PatrolPoint, WorldPoint};
use bf_viewer::bf::nav::{self, ChainEnd, Dest, Entry, Graph, Walker};

use super::*;

/// The watch rows (BF_GOAL_WATCH), after the characters' step.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, watch.after(update_player).run_if(in_state(AppState::Playing)));
}

/// The goal numbers the demo's brains tell apart (the GOAL_ name table 0x3bcbb4): GOAL_ATTACK,
/// GOAL_ATTACK_VARY, GOAL_PURSUE, GOAL_ATTACK_COVER, GOAL_PATH_TOWARD_HOME, GOAL_DEBUG1.
pub const GOAL_ATTACK: i64 = 7;
pub const GOAL_ATTACK_VARY: i64 = 8;
pub const GOAL_PURSUE: i64 = 10;
pub const GOAL_ATTACK_COVER: i64 = 19;
pub const GOAL_PATH_TOWARD_HOME: i64 = 106;
pub const GOAL_DEBUG1: i64 = 120;
/// EVT_ENEMY_SEEN (4), EVT_TGT_LOST_SIGHT (7) and EVT_PAST_HOME_AREA (84) (the EVT_ name table
/// 0x3bdb9c): the attack goal's entry, the one whose goals follow a target lost from sight, and the
/// one the sense step posts outside the home area (`default`'s and h_0bb586cf's: PATH_TOWARD_HOME
/// at 20.19).
const EVT_ENEMY_SEEN: i64 = 4;
const EVT_TGT_LOST_SIGHT: i64 = 7;
const EVT_PAST_HOME_AREA: i64 = 84;

/// FUN_0002e800's budget: the route's length must be under the distance to the target times this
/// (under 10 m, 10-25, 25-50, beyond) ...
const BUDGET: [(f32, f32); 3] = [(10.0, 2.5), (25.0, 2.0), (50.0, 1.75)];
const BUDGET_FAR: f32 = 1.5;
/// ... and that x1.2 more when the two differ in height by over 10 m (both 0x2e8a8-0x2e8f3).
const BUDGET_HIGH: f32 = 10.0;
const BUDGET_HIGH_FACTOR: f32 = 1.2;
/// FUN_0002e800's timer at a route taken (FUN_000124f0(4.0), 0x40800000): the approach plans again
/// when it has run out (FUN_0002e9c0 at 0x2ec10, its +0x350 bits 0 and 1).
const REPLAN: f32 = 4.0;
/// A route over the budget, or none: the target's contact entry +0x80 bit 6 with +0x84 = 2.0
/// (0x40000000, FUN_0002e490 / FUN_0002e9c0): FUN_00023ee0 refuses the approach meanwhile.
const UNREACHABLE: f32 = 2.0;
/// FUN_0002e9c0's stop short of the target: (FUN_00023da0 + 3 x FUN_00023e40) x 0.25.
const STOP_STANDOFF: f32 = 3.0;
const STOP_SHARE: f32 = 0.25;
/// The approach's modes (FUN_0004d970): the dash beyond the stand-off, a run inside it (local_e4 4
/// or 3, FUN_0002e9c0); PURSUE's dash (FUN_0002c730, mode 4).
const DASH: i32 = 4;
const RUN: i32 = 3;
/// FUN_00023e40 / FUN_00023da0: the gun's reach x0.8 / x0.4 when its Damage falloff mode (weapon
/// data +0x34) is 2, the squared falloff (the Bower 20's).
const FALLOFF_SQUARED: i64 = 2;
const STANDOFF_SQUARED: f32 = 0.8;
const NEAR_SQUARED: f32 = 0.4;
/// GOAL_PURSUE's end: a contact entry of kind 1 whose seen timers (+0x30, +0x40) are at least this
/// (0.1, 0x3a4ff0; FUN_0002c730 at 0x2c8a5): the demo's, any contact seen this long in a row.
const PURSUE_SEEN: f32 = 0.1;
/// FUN_00042620(1.0, 6.0), from the stuck handling (0x42868): the target seen within this (s) ...
const GUARD_SEEN: f32 = 1.0;
/// ... GOAL_ATTACK_GUARD pushed for this long (s).
const GUARD_TIME: f32 = 6.0;
/// The home area's leash (FUN_00018510): the spawn's h_f63b86a2 / h_062fbaf0 held at 7 at least
/// when both are given (not under 0), else the brain's constructor's 100 and 20 (0x166e7-0x166f1:
/// 0x42c80000, 0x41a00000).
const LEASH_MIN: f32 = 7.0;
const LEASH_RADIUS: f32 = 100.0;
const LEASH_HEIGHT: f32 = 20.0;
/// PATH_TOWARD_HOME's end (FUN_000347b0): done once FUN_000235d0 puts the character more than this
/// (m, 7.0) inside its home area. With e23's 7 m areas never: it walks to its route's end.
const HOME_DEEP: f32 = 7.0;
/// FUN_00024080's two numbers: the margin it refuses within (its third argument, 1.0 from
/// ATTACK_VARY's start and update) and the dot under which the target isn't homeward (0.3,
/// 0x24162).
const EDGE_MARGIN: f32 = 1.0;
const EDGE_DOT: f32 = 0.3;
/// PATH_TOWARD_HOME's mode (FUN_000347b0: FUN_0004d970 mode 3, a run, with both its last
/// arguments).
const HOME_RUN: i32 = 3;
/// How far ahead along its way (m) the demo's drive is aimed (play_enemies.rs `Drive::goal`): past
/// where the drive lets go (0.15 m + its slide, at most 1.4 m at a dash), so it never does while a
/// goal walks. The demo's.
const CARROT: f32 = 5.0;

/// A character's goals on routes (`Player::goals`): the state the brain keeps for them (see the
/// module notes).
#[derive(Clone, Debug, Default)]
pub struct Goals {
    /// GOAL_RETURN_HOME's (or PATH_TOWARD_HOME's) route is planned and walked (its start ran)
    home: bool,
    /// GOAL_PATH_TOWARD_HOME is on (`past_home`, `toward_home`)
    pub toward: bool,
    /// GOAL_PATROL's chain (the goal holder's +0x918, brain+0x958), walked once its first leg is
    /// over, and where the character was at its last step (FUN_00025870's from)
    pub chain: Option<Walker>,
    chain_last: Option<Vec3>,
    /// the attack goal drawn for this fight (EVT_ENEMY_SEEN's), None out of a fight
    pub attack: Option<i64>,
    /// the attack goal's approach (brain+0x4a8)
    approach: Approach,
    /// the approach's re-plan timer (s left, FUN_0002e800's 4 s)
    replan: f32,
    /// the target unreachable for so long (s: its contact's +0x80 bit 6 and +0x84)
    unreachable: f32,
    /// GOAL_PURSUE's state
    pursuit: Pursuit,
    /// how long a contact has been seen in a row (s: PURSUE's end)
    seen_for: f32,
    /// how long the ray to the target has met something (s: its contact entry's +0x28, which
    /// FUN_0002e330 compares with LOST_SIGHT for EVT_TGT_LOST_SIGHT: PURSUE's start)
    no_ray: f32,
    /// GOAL_ATTACK_GUARD pushed by the stuck handling (FUN_00042620): s left
    guard: f32,
    /// the target and its distance last frame (BF_GOAL_WATCH)
    foe: Option<(Who, f32)>,
}

/// The attack goal's approach (brain+0x4a8, FUN_0002e490 / FUN_0002e9c0): 0 not decided (the
/// goal's start to run), 1 the route to the target, 2 standing (the strafes and dodges of
/// ATTACK_VARY's state 2 are #26's). 3 (an event point's route, +0x88 bit 13) has no event in the
/// demo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Approach {
    #[default]
    Start,
    On,
    Standing,
}

/// GOAL_PURSUE (FUN_0002c730's brain+0x72c state 2, walking): not on, refused or failed for this
/// loss of sight (the demo tries once until the target is seen again), or walking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Pursuit {
    #[default]
    Off,
    Tried,
    On,
}

impl Goals {
    /// The goal it runs now, for the log and the watch: ATTACK_GUARD, PURSUE, the attack goal, a
    /// walk home, the patrol.
    fn now(&self) -> &'static str {
        if self.toward {
            "PATH_TOWARD_HOME"
        } else if self.guard > 0.0 {
            "ATTACK_GUARD (stuck)"
        } else if self.pursuit == Pursuit::On {
            "PURSUE"
        } else if self.attack.is_some() {
            match self.approach {
                Approach::On => "attack: approach",
                Approach::Standing => "attack: stands",
                Approach::Start => "attack",
            }
        } else if self.home {
            "RETURN_HOME"
        } else if self.chain.is_some() {
            "PATROL"
        } else {
            "-"
        }
    }

    /// The fight is over (no contact left): the attack goal's and the pursuit's state dropped (the
    /// goals' ends: FUN_0002e7b0, 0x28e70, reset the route, which the caller does).
    pub fn fight_over(&mut self) {
        self.attack = None;
        self.approach = Approach::Start;
        self.pursuit = Pursuit::Off;
        self.seen_for = 0.0;
        self.no_ray = 0.0;
        self.unreachable = 0.0;
        self.guard = 0.0;
        self.foe = None;
    }

    /// The attack goal is broken off for a while (PATH_TOWARD_HOME over it): its approach and the
    /// pursuit start again after (the goal's start runs again), the goal drawn kept.
    pub fn fight_paused(&mut self) {
        self.approach = Approach::Start;
        self.pursuit = Pursuit::Off;
    }

    /// The base goal is let go (a fight begins, a draw again, a script holds the character): the
    /// walk home's and the patrol's state dropped.
    pub fn calm_over(&mut self) {
        self.home = false;
        self.chain = None;
        self.chain_last = None;
    }
}

/// The home area's leash (FUN_00023630 on the record at brain+0x1ec): the home object (the home
/// point; for a spawn without a home-point its own world point as it was made, char+0x24c), the
/// radius across (+0x28, brain+0x214) and the height span (+0x2c, brain+0x218) round it, and
/// whether it holds (+0x30 bit 1, brain+0x21c bit 1: FUN_00018510 sets it for a spawn without a
/// home-point and clears it for one with, which RETURN_HOME's update sets again at home, 0x3472a).
#[derive(Clone, Copy, Debug)]
pub struct Leash {
    pub on: bool,
    pub at: Option<Vec3>,
    pub radius: f32,
    pub height: f32,
}

impl Default for Leash {
    /// The brain's constructor's (0x166e7-0x166fb): off, 100 m across, 20 m high.
    fn default() -> Self {
        Leash { on: false, at: None, radius: LEASH_RADIUS, height: LEASH_HEIGHT }
    }
}

impl Leash {
    /// A spawn's (FUN_00018510): the trigger's h_f63b86a2 and h_062fbaf0 (`radius`, `height`), each
    /// at least LEASH_MIN when neither is under 0, else the constructor's; on for a spawn without a
    /// home-point, off for one with.
    pub fn of(radius: f32, height: f32, home_point: bool) -> Leash {
        let given = radius >= 0.0 && height >= 0.0;
        Leash {
            on: !home_point,
            at: None,
            radius: if given { radius.max(LEASH_MIN) } else { LEASH_RADIUS },
            height: if given { height.max(LEASH_MIN) } else { LEASH_HEIGHT },
        }
    }

    /// FUN_00023630: whether position `p` lies inside the leash: always with no home or the bit
    /// clear; else under the home's height + half the span and over it less half the span, and
    /// within the radius across.
    pub fn inside(&self, p: Vec3) -> bool {
        let Some(h) = self.at.filter(|_| self.on) else { return true };
        if h.y + self.height * 0.5 <= p.y {
            return false;
        }
        h.y - self.height * 0.5 < p.y && p.xz().distance_squared(h.xz()) <= self.radius * self.radius
    }

    /// FUN_000235d0: how far inside the home area position `p` is (m): the lower of the whole
    /// height span less its height over the home (the span not halved, as FUN_00023630 halves it)
    /// and the radius less its distance across (FUN_00023540); FLT_MAX with no home or the bit
    /// clear. PATH_TOWARD_HOME is done over HOME_DEEP, ATTACK_VARY refuses under EDGE_MARGIN.
    pub fn margin(&self, p: Vec3) -> f32 {
        let Some(h) = self.at.filter(|_| self.on) else { return f32::MAX };
        (self.height - (p.y - h.y)).min(self.radius - p.xz().distance(h.xz()))
    }

    /// FUN_00024080(brain, target, 1.0): whether a character at `me` stands at its home area's edge
    /// (`margin` under EDGE_MARGIN) with target `t` not homeward (the dot of the way to the home,
    /// 3D and unit, with the way to the target, across and unit, under EDGE_DOT): ATTACK_VARY's
    /// start refuses (0x2cad9) and its update fails (0x2d851) then. False with no home or the bit
    /// clear (the margin FLT_MAX).
    pub fn at_edge(&self, me: Vec3, t: Vec3) -> bool {
        if self.margin(me) >= EDGE_MARGIN {
            return false;
        }
        let Some(h) = self.at else { return false };
        let to_t = Vec2::new(t.x - me.x, t.z - me.z).normalize_or_zero();
        let to_h = (h - me).normalize_or_zero();
        to_h.x * to_t.x + to_h.z * to_t.y < EDGE_DOT
    }
}

/// How a goal asks FUN_0004d970 to move: the mode (2 a walk, 3 a run, 4 the dash), whether the
/// pace level holds it (its fourth argument, `clamp`) and whether FUN_0004d840 slows it near its
/// way's end (its fifth, `near`). RETURN_HOME, PATROL, PURSUE and GOAL_ATTACK pass both; ATTACK_VARY's
/// approach neither (FUN_0002d710 at 0x2e1b4: `FUN_0004d970(brain, dir, mode, 0, 0)`), so it moves
/// at its mode at once.
#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub mode: i32,
    pub clamp: bool,
    pub near: bool,
}

impl Move {
    /// A goal's walk at `mode`, held to the pace level and slowed near its way's end.
    pub fn held(mode: i32) -> Move {
        Move { mode, clamp: true, near: true }
    }
}

/// The world's ray as the route code asks it (play_nav.rs `clear`, FUN_0004ce30).
fn ray(a: Vec3, b: Vec3, max: f32) -> bool {
    let arena = world::arena();
    super::nav::clear(arena.as_deref(), a, b, max)
}

/// Whether BF_BRAIN_LOG is set (the brains' log, which the goals' lines go to).
fn log_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_BRAIN_LOG").is_ok())
}

/// FUN_000255d0 then FUN_0004d970 for a level character walking a goal toward `target` (the walk's
/// steering target, brain+0x478) as `mv` asks: the direction through the AI's avoidance
/// (play_avoid.rs `steer`); the mode asked into the pace record ([brain+0x1a8]+4, which its level
/// climbs toward, play_routes.rs `pace_ramp`) and, with `mv.near`, lowered near the way's end or a
/// corner (FUN_0004d840: `routes::pace_near`, by the stop point brain+0x1b4 when the approach set
/// one, `Router::stop`; without the level's points by that alone, `pace_to`); the move at the mode,
/// held to the level with `mv.clamp`, turned at most 200 degrees a second (FUN_00019af0,
/// `avoid::move_turn`); not on into a liquid that burns (`routes::shore`, the routes' safety net).
/// The demo moves an enemy type by its drive (play_enemies.rs: FUN_0012c6c0, the type's speed for the
/// motion type, the body turned at its turn-rate), toward a point CARROT m along the move: a walk at
/// level 2, a run at 3, the dash at 4 (a run inside a prism, BF_TEST_PRISM); at level 1 (a stand,
/// the stick at 0.01: the pace record's start, a drop) it stands this frame, the mode asked kept so
/// the level climbs toward it. `ready`: the gun raised (the ready-script's clips: the fight's). (A
/// squad-type character's brain has no drive: its walk moves nothing, as before #336.)
pub fn go(m: &mut Player, g: Option<&Graph>, target: Vec3, mv: Move, ready: bool, dt: f32) {
    let (dir, _) = avoid::steer(m, target);
    m.router.asked = mv.mode;
    if mv.near {
        match (g, m.router.stop) {
            (Some(g), _) => routes::pace_near(m, g),
            (None, Some(stop)) => routes::pace_to(m, stop),
            _ => {}
        }
    }
    let level = if mv.clamp { m.router.asked.min(m.router.cap) } else { m.router.asked };
    let size = avoid::move_size(level);
    let way = avoid::move_turn(m, dir, size, dt);
    let mt = match level {
        ..=1 => None,
        2 => Some(enemies::MT_WALK),
        3 => Some(enemies::MT_RUN),
        _ => Some(if routes::in_prism() { enemies::MT_RUN } else { enemies::MT_DASH }),
    };
    let held = way.length_squared() < 1e-6 || routes::shore(m, way, dt);
    match mt.filter(|_| !held) {
        Some(mt) => drive(m, mt, way, ready),
        None => {
            brain::stop_walk(m);
            m.router.avoid.stand();
        }
    }
}

/// The drive (play_enemies.rs) in motion type `mt` toward CARROT m along `way`: the drive it has
/// re-aimed when it's the same motion, else a new one, but not while a hit reaction is due or plays
/// (the walk waits for it).
fn drive(m: &mut Player, mt: i64, way: Vec2, ready: bool) {
    let goal = m.position.xz() + way.normalize_or_zero() * CARROT;
    if let Some(d) = m.enemy.drive.as_mut().filter(|d| d.mt == mt && d.ready == ready) {
        d.goal = Some(goal);
    } else if !enemies::reacting(m) {
        enemies::drive(m, mt, Some(goal), ready);
    }
}

/// A goal stands this frame (FUN_0004d970 asked a stand, mode 1): its walk stops and nothing of its
/// move is kept for the next frame's turn.
pub fn stand(m: &mut Player) {
    brain::stop_walk(m);
    m.router.asked = 1;
    m.router.avoid.stand();
}

/// What a goal's update returned: still on (1), over (2), or its start refused (0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Done,
    Refused,
}

/// GOAL_RETURN_HOME (`goal` its name for the log; PATH_TOWARD_HOME shares its start, `toward_home`)
/// for `m` (`label` its log name) to world point `home` (the graph's node) at
/// `mode` (FUN_00034690: 2, or the spawn's `locomotion`, the goal's +0x94, when over 2;
/// FUN_0004d970 takes +0x94 over any mode when it isn't 0, so locomotion 1 is a stand):
///  - the start (0x345d0), once: refused when `refused` (brain+0x3c4 bit 11, set by a refusal and
///    never cleared), when the own point's +0x24 bit 15 (h_1346678f) isn't the home point's, or when
///    no route is found (FUN_00041660 into brain+0x930); a refusal sets `refused`;
///  - the update (FUN_00034690): the route walked at 2 m (FUN_00026d80: the home point's own radius,
///    h_e6564e8a, when it has one), steered and moved (`go`, FUN_0004d970 with both its last
///    arguments); its end: done (the goal record's +0x88 bit 1), and the leash goes on (`Leash::on`,
///    brain+0x21c bit 1, 0x3472a); an empty route: done.
///
/// A mode of 1 doesn't move it: the route is walked only as far as it stands (within the radius of
/// its end: home), else the caller gives the walk up (the demo's, as before #336).
pub fn home(m: &mut Player, g: &mut Graph, home: u32, mode: i32, refused: &mut bool, label: &str, goal: &str, dt: f32) -> Status {
    if !m.goals.home {
        let bit15 = |p: u32| g.nodes[p as usize].flags & WorldPoint::BIT_15 != 0;
        let why = if *refused {
            Some("refused before (brain+0x3c4 bit 11)")
        } else if m.router.own.point.is_none_or(|o| bit15(o) != bit15(home)) {
            Some("its own point's h_1346678f isn't the home point's")
        } else if routes::plan(m, g, Dest::Point(home), goal).is_err() {
            Some("no route")
        } else {
            None
        };
        if let Some(why) = why {
            *refused = true;
            if log_on() {
                println!("t={:.2} brain {label}: {goal} refused: {why}; stays at ({:.1}, {:.1})", m.sim_time, m.position.x, m.position.z);
            }
            return Status::Refused;
        }
        m.goals.home = true;
        if log_on() {
            let n = &g.nodes[home as usize];
            println!("t={:.2} brain {label}: {goal}: a route of {} entries to h_{:08x} ({:.1}, {:.1}), {:.1} m along it", m.sim_time,
                     m.router.walker.as_ref().map_or(0, |w| w.entries.len()), n.name, n.at.x, n.at.z, m.router.walker.as_ref().map_or(0.0, |w| w.remaining(g, m.position)));
        }
    }
    if !m.router.walking() {
        m.goals.home = false;
        return Status::Done;
    }
    let s = routes::step(m, g, nav::ARRIVE, nav::ARRIVE);
    if !s.walking {
        m.goals.home = false;
        stand(m);
        return Status::Done;
    }
    if mode <= 1 {
        // (the stick at 0.01: it doesn't move, and the demo gives the walk up)
        m.goals.home = false;
        routes::stop(m, g);
        stand(m);
        return Status::Refused;
    }
    go(m, Some(g), s.target, Move::held(mode), false, dt);
    Status::Running
}

/// Whether `m`'s brain must take GOAL_PATH_TOWARD_HOME (106) now: EVT_PAST_HOME_AREA (84), which
/// the sense step posts each step for a brain out of the squad whose position is outside its home
/// area (FUN_00044480 at 0x444e7-0x4451f: FUN_00023630 false, brain+0x3c4 bits 8 and 11 clear;
/// `refused` is bit 11), draws it at 20.19 when its personality's entry has it (`homeward`), over
/// EVT_ENEMY_SEEN's 18 and the calm goals; once on it runs to its end (`toward_home`). Not when the
/// area's margin is over HOME_DEEP already (FUN_000235d0 takes the whole height span where
/// FUN_00023630 takes half: just over the top of a 20 m span the goal would end as it starts; the
/// demo leaves it off then).
pub fn past_home(m: &Player, leash: &Leash, refused: bool, homeward: bool) -> bool {
    m.goals.toward || (homeward && !refused && !leash.inside(m.position) && leash.margin(m.position) <= HOME_DEEP)
}

/// Whether personality `personality` answers EVT_PAST_HOME_AREA with GOAL_PATH_TOWARD_HOME: its
/// entry (`default`'s, which the Red Hand, the Shrikes and the mutants inherit, and h_0bb586cf's)
/// lists it; one with no entry lets the event go by.
pub fn homeward(game: &Game, personality: u32) -> bool {
    game.personality_entry(personality, EVT_PAST_HOME_AREA).is_some_and(|e| e.goals.iter().any(|g| g.0 == GOAL_PATH_TOWARD_HOME))
}

/// GOAL_PATH_TOWARD_HOME for `m` (`past_home` says when): RETURN_HOME's start (0x345d0: the route
/// to the home point `point`, refused by bit 11 or bit 15 or no route) and its own update
/// (FUN_000347b0): done once more than HOME_DEEP inside the home area (FUN_000235d0, `margin`), else
/// the route walked at 2 m at a run (FUN_0004d970 mode 3 with both its last arguments), done at its
/// end. It doesn't fire (it clears the view's override, brain+0x488, and calls no attack check): a
/// mutant moved or spawned far from its home walks home past the player (todo/132's F1 take, its
/// Uglys going PATH_TOWARD_HOME a few metres out; a teleported Ugly 33 m from home never fought).
pub fn toward_home(m: &mut Player, g: &mut Graph, point: u32, leash: &Leash, refused: &mut bool, label: &str, dt: f32) -> Status {
    if leash.margin(m.position) > HOME_DEEP {
        if m.goals.home {
            m.goals.home = false;
            routes::stop(m, g);
        }
        m.goals.toward = false;
        return Status::Done;
    }
    if !m.goals.toward && log_on() {
        println!("t={:.2} brain {label}: outside its home area ({:.1} m in): GOAL_PATH_TOWARD_HOME (EVT_PAST_HOME_AREA, 20.19)", m.sim_time, leash.margin(m.position));
    }
    m.goals.toward = true;
    let status = home(m, g, point, HOME_RUN, refused, label, "GOAL_PATH_TOWARD_HOME", dt);
    if status != Status::Running {
        m.goals.toward = false;
    }
    status
}

/// GOAL_PATROL's start (FUN_0002af70) for `m` at `here` (its position) over `patrols` (the level's
/// patrol points, world+0x1d0) and the graph: the patrol point nearest it (3D, FUN_000394b0), the
/// first head (`<patrol-first-point>`) of that point's iD, and the chain from the head along the
/// forward links until there are none or they come back to the head, its index at the nearest
/// point's place (the head's when that isn't on it); under two points it refuses. Then the first
/// leg: the valid point FUN_0004c2e0 finds visible within 20 m of the chain's current point
/// (`Graph::visible_near`), and a route to it (FUN_000416a0 into brain+0x930, started with
/// FUN_00025730(route, 1, 0)); with none, or no route, the start refuses (0x2b348, 0x2b425). The
/// chain (brain+0x958) is a walk of its patrol points (`Walker::chain`: turned round at an open end,
/// a loop's last point on to its start, FUN_00026940). Whether it started.
pub fn patrol_start(m: &mut Player, g: &mut Graph, patrols: &[PatrolPoint], label: &str) -> bool {
    let here = m.position;
    let log = log_on();
    let now = m.sim_time;
    let refuse = |why: String| {
        if log {
            println!("t={now:.2} brain {label}: GOAL_PATROL refused: {why}");
        }
        false
    };
    let Some(nearest) = patrols.iter().min_by(|a, b| a.at.distance(here).total_cmp(&b.at.distance(here))) else { return refuse("no patrol points".into()) };
    let Some(head) = patrols.iter().find(|p| p.head && p.group == nearest.group) else { return refuse(format!("no head of iD {}", nearest.group)) };
    let mut chain = vec![head];
    let mut next = head.forward;
    while let Some(p) = patrols.iter().find(|p| p.name == next && next != 0 && p.name != head.name) {
        if chain.len() > patrols.len() {
            break;
        }
        chain.push(p);
        next = p.forward;
    }
    if chain.len() < 2 {
        return refuse(format!("a chain of {} point", chain.len()));
    }
    let index = chain.iter().position(|p| p.name == nearest.name).unwrap_or(0);
    let at = chain[index].at;
    // (the first leg, FUN_0004c2e0 at the chain's current point)
    let Some(leg) = g.visible_near(at, &ray) else { return refuse(format!("no valid point in sight within 20 m of its patrol point ({:.1}, {:.1})", at.x, at.z)) };
    if routes::plan(m, g, Dest::Point(leg), "GOAL_PATROL's first leg").is_err() {
        let n = &g.nodes[leg as usize];
        return refuse(format!("no route to h_{:08x} ({:.1}, {:.1}), the point by its patrol point", n.name, n.at.x, n.at.z));
    }
    let last = chain[chain.len() - 1];
    let end = if last.forward != 0 && last.backward != 0 && head.forward != 0 && head.backward != 0 { ChainEnd::Loop } else { ChainEnd::TurnRound };
    let entries: Vec<Entry> = chain.iter().map(|p| Entry { at: p.at, point: g.find(p.name) }).collect();
    m.goals.chain = Some(Walker { entries, index: index as i32, best: f32::MAX, chain: Some(end), ..Default::default() });
    m.goals.chain_last = None;
    if log {
        let n = &g.nodes[leg as usize];
        println!("t={now:.2} brain {label}: patrols iD {} ({} points, from point {index}, ({:.1}, {:.1}), nearest of {} at {:.1} m; {end:?} at its ends); its first leg a \
                  route of {} entries to h_{:08x} ({:.1}, {:.1}), {:.1} m from the patrol point", head.group, chain.len(), at.x, at.z, patrols.len(),
                 nearest.at.distance(here), m.router.walker.as_ref().map_or(0, |w| w.entries.len()), n.name, n.at.x, n.at.z, n.at.distance(at));
    }
    true
}

/// GOAL_PATROL's update (FUN_0002b490): its first leg's route (brain+0x930) walked while it has
/// entries, then the chain (brain+0x958), each at 2 m (FUN_00026d80: a patrol point's own
/// h_e6564e8a above 0 instead, the four by the tutorial's gate 1 m); steered and moved at a walk
/// (`go`: FUN_0004d970 mode 2 with both its last arguments). A point's trigger-object would be sent
/// an event (FUN_00026940): none in the data the demo plays (logged, `patrols` the level's patrol
/// points). Whether it walks (held to a stand by the pace level or not).
pub fn patrol(m: &mut Player, g: &mut Graph, patrols: &[PatrolPoint], label: &str, dt: f32) -> bool {
    let mut target = None;
    if m.router.walking() {
        let s = routes::step(m, g, nav::ARRIVE, nav::ARRIVE);
        if s.walking {
            target = Some(s.target);
        }
    }
    if target.is_none() {
        let me = m.position;
        let last = m.goals.chain_last.replace(me).unwrap_or(me);
        let Some(w) = m.goals.chain.as_mut() else { return false };
        let s = w.step(g, me, last, routes::in_prism(), nav::ARRIVE, &ray);
        if log_on() {
            if let Some((Some(p), _)) = s.reached {
                let n = &g.nodes[p as usize];
                let trigger = patrols.iter().find(|q| q.name == n.name).map_or(0, |q| q.trigger);
                println!("t={:.2} brain {label}: reaches patrol point h_{:08x} ({:.1}, {:.1}) within {:.1} m{}{}", m.sim_time, n.name, n.at.x, n.at.z, s.radius,
                         match s.turned {
                             Some(ChainEnd::TurnRound) => ": the chain's end, turned round",
                             Some(ChainEnd::Loop) => ": the loop's end, on from its start",
                             None => "",
                         }, if trigger != 0 { format!("; its trigger-object h_{trigger:08x} not sent (#227)") } else { String::new() });
            }
        }
        if !s.walking {
            m.goals.chain = None;
            return false;
        }
        target = Some(s.target);
    }
    let Some(t) = target else { return false };
    go(m, Some(g), t, Move::held(2), false, dt);
    true
}

/// FUN_00023e40, the attack goals' stand-off for `m` (EAX the brain): its type's h_150b4e02 (the
/// sensor's +0xcc), held to the gun in hand's reach (FUN_00232520, play_reticle.rs `reach`; x0.8
/// with its Damage falloff mode 2); without a gun the setting alone.
pub fn standoff(m: &Player, game: &Game) -> f32 {
    let setting = game.character_type(m.kind).map_or(0.0, |t| t.ai.standoff);
    held_reach(m, STANDOFF_SQUARED).map_or(setting, |r| setting.min(r))
}

/// FUN_00023da0, the near bound for `m`: its type's h_e244891c (the sensor's +0xd4), held to the
/// gun in hand's reach (x0.4 with its Damage falloff mode 2).
pub fn near_bound(m: &Player, game: &Game) -> f32 {
    let setting = game.character_type(m.kind).map_or(0.0, |t| t.ai.near_bound);
    held_reach(m, NEAR_SQUARED).map_or(setting, |r| setting.min(r))
}

/// The gun in `m`'s hand's reach (FUN_00232520), x `squared` when its Damage falloff is mode 2;
/// None without a gun.
fn held_reach(m: &Player, squared: f32) -> Option<f32> {
    let def = &m.loaded.as_ref()?.weapons.get(m.weapon)?.def;
    let r = reticle::reach(m, def);
    Some(if def.shots.falloff == FALLOFF_SQUARED { r * squared } else { r })
}

/// FUN_0002e9c0's stop: how far short of its target the approach stops, (`near` + 3 x
/// `standoff`) / 4.
pub fn stop_short(standoff: f32, near: f32) -> f32 {
    (near + STOP_STANDOFF * standoff) * STOP_SHARE
}

/// FUN_0002e800's budget for a target `d` m away (3D) and `dy` m above or below: the radius the
/// search is held within (DAT_004021a8) and the route's length must be under.
pub fn budget(d: f32, dy: f32) -> f32 {
    let r = if dy.abs() > BUDGET_HIGH { d * BUDGET_HIGH_FACTOR } else { d };
    r * BUDGET.iter().find(|(under, _)| d < *under).map_or(BUDGET_FAR, |b| b.1)
}

/// What an attack goal's draw comes to in the demo (EVT_ENEMY_SEEN's goal, `draw_attack`): its
/// approach with the move held to the pace level and slowed near its stop (GOAL_ATTACK), or not
/// (ATTACK_VARY, and DEBUG1, which runs ATTACK_VARY once ATTACK_COVER is done); None for a goal that
/// stands (ATTACK_GUARD, ATTACK_GUARD_CROUCH; the goals of #26 and the mutants' tickets).
pub fn approaches(goal: i64) -> Option<bool> {
    match goal {
        GOAL_ATTACK => Some(true),
        GOAL_ATTACK_VARY | GOAL_DEBUG1 => Some(false),
        _ => None,
    }
}

/// The attack goal personality `personality` draws for a fight (its EVT_ENEMY_SEEN entry,
/// FUN_00054c40 with `u` the draw): ATTACK_COVER refused and struck out (FUN_00054bc0) as with no
/// cover point to be had, the draw made again (the demo's: its cover choice, FUN_0002f270's flood
/// for a cover point, is #26's), so h_eed3a76a's ATTACK_COVER 1 : ATTACK_VARY 0 : ATTACK_GUARD_CROUCH
/// 0 comes to ATTACK_VARY; None without an entry.
pub fn draw_attack(game: &Game, personality: u32, u: f32) -> Option<i64> {
    let e = game.personality_entry(personality, EVT_ENEMY_SEEN)?;
    e.draw(u, &[GOAL_ATTACK_COVER]).or(e.draw(u, &[]))
}

/// Whether personality `personality` pursues a target it has lost from sight: its
/// EVT_TGT_LOST_SIGHT entry lists GOAL_PURSUE (`default`'s: REPOSITION_LOS_TO_TGT 1, PURSUE 0; the
/// reposition is #26's, so the demo's draw comes to PURSUE, as with the reposition refused).
pub fn pursues(game: &Game, personality: u32) -> bool {
    game.personality_entry(personality, EVT_TGT_LOST_SIGHT).is_some_and(|e| e.goals.iter().any(|g| g.0 == GOAL_PURSUE))
}

/// The target as the attack goals and PURSUE look at it: who, where it is now (its position: the
/// goals read the target object's, vtable +0x40, not where it was last seen), its own world point
/// (char+0x24c: a character's place routes there, FUN_0011dfa0), and how long it's been unseen.
#[derive(Clone, Copy, Debug)]
pub struct Foe {
    pub who: Who,
    pub at: Vec3,
    pub point: Option<u32>,
    pub unseen: f32,
}

impl Foe {
    /// `who`, `m`'s contact, from the frame's characters `everyone` (play.rs: each at the frame's
    /// start); None when it isn't one of them or isn't a contact.
    pub fn of(m: &Player, who: Who, everyone: &[follow::Lead]) -> Option<Foe> {
        let l = everyone.iter().find(|l| l.who == who)?;
        Some(Foe { who, at: l.at, point: l.point, unseen: m.sensor.unseen(who)? })
    }
}

/// One frame of `m`'s fight, the attack goal `goal` against `foe` (its target), over the level's
/// points `g` (None: the test map's floor, where the approach walks straight at the target, the
/// demo's): GOAL_PURSUE when `pursue` and the target is lost, else the attack goal's approach (see
/// the module notes). `standoff` and `near` are FUN_00023e40's and FUN_00023da0's; `leash` the home
/// area's. True while the approach or the pursuit walks (held to a stand by the pace level or not:
/// no fire, neither fires), false when it stands and fires (the caller's, as before #336).
#[allow(clippy::too_many_arguments)]
pub fn fight(m: &mut Player, g: Option<&mut Graph>, goal: i64, foe: Foe, pursue: bool, leash: &Leash, standoff: f32, near: f32, label: &str, dt: f32) -> bool {
    let mut g = g;
    // (the target's entry's +0x28: how long the ray from the eyes to its chest has met something,
    // whichever way the character faces; the demo's eye and chest, as `Sensor::scan`'s)
    let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
    let chest = foe.at + Vec3::Y * (GROUND + AI_AIM_HEIGHT);
    let to = chest - eye;
    let ray_clear = to.length() < 0.3 || ray_hit(eye, to.normalize(), to.length() - 0.3).is_none();
    let gs = &mut m.goals;
    gs.unreachable = (gs.unreachable - dt).max(0.0);
    gs.seen_for = if m.sensor.seen_now() { gs.seen_for + dt } else { 0.0 };
    gs.no_ray = if ray_clear { 0.0 } else { gs.no_ray + dt };
    let d = foe.at.distance(m.position);
    m.goals.foe = Some((foe.who, d));
    let log = log_on();
    // GOAL_PURSUE: the target lost from sight (FUN_0002e330: no clear ray for LOST_SIGHT s posts
    // EVT_TGT_LOST_SIGHT, whose draw comes to PURSUE in the demo); tried again once it's in sight
    let lost = m.goals.no_ray >= brain::LOST_SIGHT;
    if !lost && foe.unseen < brain::LOST_SIGHT && m.goals.pursuit == Pursuit::Tried {
        m.goals.pursuit = Pursuit::Off;
    }
    if pursue && lost && m.goals.pursuit == Pursuit::Off {
        m.router.stop = None;
        let started = pursue_start(m, g.as_deref_mut(), &foe, leash, label);
        m.goals.pursuit = if started { Pursuit::On } else { Pursuit::Tried };
    }
    if m.goals.pursuit == Pursuit::On {
        // (done as soon as a contact is seen, 0x2c886-0x2c90b: back to the attack goal)
        if m.goals.seen_for >= PURSUE_SEEN {
            if log {
                println!("t={:.2} brain {label}: GOAL_PURSUE done: a contact seen", m.sim_time);
            }
            m.goals.pursuit = Pursuit::Off;
            m.goals.approach = Approach::Start;
            if let Some(g) = g.as_deref_mut() {
                routes::stop(m, g);
            }
        } else {
            return pursue_walk(m, g, &foe, leash, label, dt);
        }
    }
    let Some(clamp) = approaches(goal) else {
        m.router.stop = None;
        return false;
    };
    let stop = stop_short(standoff, near);
    // (ATTACK_VARY's home-area edge, FUN_00024080: its start refuses and its update fails at the
    // edge with the target outward, so it fights from there; GOAL_ATTACK, `clamp`, has no such test)
    let edge = !clamp && leash.at_edge(m.position, foe.at);
    if m.goals.approach == Approach::Start {
        // (the goal's start: approach 1 when FUN_00023ee0 lets it and the budgeted route is found)
        m.goals.approach = if !edge && m.goals.unreachable <= 0.0 && approach_route(m, g.as_deref_mut(), &foe, label) {
            Approach::On
        } else {
            Approach::Standing
        };
    }
    if m.goals.approach == Approach::On && edge {
        if log {
            println!("t={:.2} brain {label}: at its home area's edge ({:.1} m in), its target outward: stands and fires (FUN_00024080)", m.sim_time,
                     leash.margin(m.position));
        }
        if let Some(g) = g.as_deref_mut() {
            routes::stop(m, g);
        }
        m.goals.approach = Approach::Standing;
    }
    if m.goals.approach == Approach::On {
        if d >= stop {
            let mode = if standoff < d { DASH } else { RUN };
            m.goals.replan -= dt;
            let walking = g.is_none() || m.router.walking();
            if (m.goals.replan <= 0.0 || !walking) && !approach_route(m, g.as_deref_mut(), &foe, label) {
                m.goals.unreachable = UNREACHABLE;
            }
            let walking = g.is_none() || m.router.walking();
            if walking {
                // brain+0x1b4: the stop point the pace goes by (FUN_0004d840)
                let to = foe.at - m.position;
                m.router.stop = Some(foe.at - to.normalize_or_zero() * stop);
                let target = match g.as_deref_mut() {
                    Some(g) => {
                        let s = routes::step(m, g, nav::ARRIVE, nav::ARRIVE);
                        if !s.walking {
                            // (its end: the goal done, +0x88 bit 1; its start again next frame)
                            m.goals.approach = Approach::Start;
                            m.router.stop = None;
                        }
                        s.target
                    }
                    None => foe.at,
                };
                let mv = Move { mode, clamp, near: clamp };
                go(m, g.as_deref(), target, mv, true, dt);
                return true;
            }
            // (no route: the update fails, 0x2ec35, and the goal's start again finds the target
            // unreachable: it stands)
            if log {
                println!("t={:.2} brain {label}: the approach has no route: stands, the target unreachable {UNREACHABLE} s", m.sim_time);
            }
            m.goals.approach = Approach::Standing;
        } else {
            if log {
                println!("t={:.2} brain {label}: the approach stops {d:.1} m from its target (under {stop:.2} m): stands and fires", m.sim_time);
            }
            if let Some(g) = g.as_deref_mut() {
                routes::stop(m, g);
            }
            m.goals.approach = Approach::Standing;
        }
    }
    // approach 2: on again once the target is beyond the stand-off and reachable (0x2ef3f)
    m.router.stop = None;
    if !edge && m.goals.unreachable <= 0.0 && standoff < d {
        if log {
            println!("t={:.2} brain {label}: its target {d:.1} m off, beyond its stand-off {standoff:.1} m: approaches", m.sim_time);
        }
        m.goals.approach = Approach::On;
    }
    false
}

/// FUN_0002e800 for `m`'s approach to `foe`: a route to its place (its own point) found within the
/// budget's radius (`budget`, DAT_004021a8 for the one search) and shorter than it (FUN_00023460,
/// `Graph::route_length`): started, the re-plan timer at REPLAN; else none (a route found over the
/// budget dropped). On the test map's floor (no points) the demo's straight approach always has
/// its way. Whether it has its route.
fn approach_route(m: &mut Player, g: Option<&mut Graph>, foe: &Foe, label: &str) -> bool {
    m.goals.replan = REPLAN;
    let Some(g) = g else { return true };
    let d = foe.at.distance(m.position);
    let r = budget(d, m.position.y - foe.at.y);
    let log = log_on();
    match routes::plan_within(m, g, Dest::Character { at: foe.at, point: foe.point }, "GOAL_ATTACK's approach", Some(r)) {
        Ok(route) => {
            let len = g.route_length(&route.entries);
            if len < r {
                if log {
                    println!("t={:.2} brain {label}: approaches its target {d:.1} m off: a route of {} entries, {len:.1} m, under its budget {r:.1} m", m.sim_time,
                             route.entries.len());
                }
                return true;
            }
            if log {
                println!("t={:.2} brain {label}: its target {d:.1} m off: the route, {len:.1} m, is over its budget {r:.1} m: the target unreachable {UNREACHABLE} s",
                         m.sim_time);
            }
            routes::stop(m, g);
            false
        }
        Err(e) => {
            if log {
                println!("t={:.2} brain {label}: its target {d:.1} m off: no route within its budget {r:.1} m ({e:?}): the target unreachable {UNREACHABLE} s",
                         m.sim_time);
            }
            false
        }
    }
}

/// GOAL_PURSUE's start (0x2c630) for `m` after `foe`: refused when the target lies outside the
/// home leash (FUN_00023630, `Leash::inside`) or has no route to its place (FUN_0011dfa0's: its own
/// point; FUN_00041660 into brain+0x930); else started (FUN_00025730(route, 1, 1), the state 2).
/// The pursuer's line e072476b (5%) isn't said (the mutants' and the Red Hand's chatter: #36's).
/// On the test map's floor the demo's straight walk always starts.
fn pursue_start(m: &mut Player, g: Option<&mut Graph>, foe: &Foe, leash: &Leash, label: &str) -> bool {
    let log = log_on();
    if !leash.inside(foe.at) {
        if log {
            println!("t={:.2} brain {label}: GOAL_PURSUE refused: its target ({:.1}, {:.1}) is outside its home area", m.sim_time, foe.at.x, foe.at.z);
        }
        return false;
    }
    let Some(g) = g else { return true };
    let ok = routes::plan(m, g, Dest::Character { at: foe.at, point: foe.point }, "GOAL_PURSUE").is_ok();
    if log {
        println!("t={:.2} brain {label}: GOAL_PURSUE {}", m.sim_time,
                 if ok { format!("after its target, {:.1} m along the route", m.router.walker.as_ref().map_or(0.0, |w| w.remaining(g, m.position))) } else { "refused: no route".into() });
    }
    ok
}

/// GOAL_PURSUE's update (FUN_0002c730, state 2): the route walked at 2 m; at its end the start again
/// (a fresh route to where the target is now), which refused ends the goal (it fails, 0); moved at
/// the dash (`go`, FUN_0004d970 mode 4 with both its last arguments), the gun raised. True while it
/// walks (held to a stand by the pace level or not).
fn pursue_walk(m: &mut Player, g: Option<&mut Graph>, foe: &Foe, leash: &Leash, label: &str, dt: f32) -> bool {
    let mut g = g;
    let target = match g.as_deref_mut() {
        Some(gr) => {
            let mut s = routes::step(m, gr, nav::ARRIVE, nav::ARRIVE);
            if !s.walking {
                if !pursue_start(m, Some(gr), foe, leash, label) {
                    m.goals.pursuit = Pursuit::Tried;
                    stand(m);
                    return false;
                }
                s = routes::step(m, gr, nav::ARRIVE, nav::ARRIVE);
            }
            s.target
        }
        None => foe.at,
    };
    go(m, g.as_deref(), target, Move::held(DASH), true, dt);
    true
}

/// FUN_00042620(GUARD_SEEN, GUARD_TIME) from the stuck handling (play_routes.rs `stuck`, after a
/// detour that found nothing): for a brain out of the squad (brain+0x3c4 bit 8 clear: not a
/// follower) whose target's contact was seen now or within GUARD_SEEN s, GOAL_ATTACK_GUARD (14) is
/// pushed at priority 2 for GUARD_TIME s (FUN_00024540) and the goal it ran is done (+0x88 bit 1):
/// it stands and fires meanwhile (ATTACK_GUARD's update with no route, FUN_00031b40), its attack
/// goal started again after. True when pushed (the caller drops the route with the goal).
pub fn stuck_guard(m: &mut Player) -> bool {
    if !m.brain.as_ref().is_some_and(|b| !b.follows()) {
        return false;
    }
    let Some(unseen) = brain::target(m).and_then(|w| m.sensor.unseen(w)) else { return false };
    if unseen >= GUARD_SEEN {
        return false;
    }
    m.goals.guard = GUARD_TIME;
    m.goals.approach = Approach::Start;
    m.goals.pursuit = Pursuit::Off;
    m.router.stop = None;
    if log_on() {
        println!("t={:.2} {}: stuck with its target seen {unseen:.1} s ago: GOAL_ATTACK_GUARD for {GUARD_TIME} s (FUN_00042620)", m.sim_time, unit_name(m));
    }
    true
}

/// ATTACK_GUARD's time for a frame of `dt` s (`stuck_guard`): true while it stands guard.
pub fn guarding(m: &mut Player, dt: f32) -> bool {
    if m.goals.guard <= 0.0 {
        return false;
    }
    m.goals.guard = (m.goals.guard - dt).max(0.0);
    true
}

/// One frame of a test map NPC set to fight (no brain, `npc_ai` its stand and fire): its
/// personality's attack goal (`draw_attack`, drawn once its first contact is known) closing in on
/// the contact it goes after (`Sensor::nearest`, its place now from `everyone`) as the brains'
/// approach does: straight on the test map's floor (no world points: the demo's; the game's
/// characters always have their route), by routes where the map has points. No pursuit, no leash:
/// the NPC has neither a brain nor a home. True while its approach walks (it doesn't fire), false
/// when it stands and fires (`npc_ai`).
pub fn npc_fight(m: &mut Player, game: &Game, everyone: &[follow::Lead], g: Option<&mut Graph>, dt: f32) -> bool {
    let mut g = g;
    if m.npc.is_none_or(|n| !n.fight) || m.sensor.known() == 0 {
        if m.goals.attack.is_some() {
            m.goals.fight_over();
            m.router.stop = None;
            if let Some(g) = g.as_deref_mut() {
                routes::stop(m, g);
            }
            stand_npc(m);
        }
        return false;
    }
    let label = unit_name(m);
    if m.goals.attack.is_none() {
        // (its personality: the placement's, else its type's; play_enemies.rs `spawned` sets it as
        // the model loads, which may come after its first contact)
        let personality = Some(m.enemy.personality).filter(|&p| p != 0).or(game.character_type(m.kind).map(|t| t.ai.personalities[0])).unwrap_or(0);
        let u = m.random(1_000_000) as f32 / 1_000_000.0;
        let goal = draw_attack(game, personality, u).unwrap_or(0);
        if log_on() {
            println!("t={:.2} brain {label}: its attack goal {goal} (personality h_{personality:08x}'s EVT_ENEMY_SEEN, ATTACK_COVER's cover choice #26's){}", m.sim_time,
                     if approaches(goal).is_some() { ": approaches" } else { ": stands" });
        }
        m.goals.attack = Some(goal);
    }
    let goal = m.goals.attack.unwrap_or(0);
    let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
    let Some(foe) = m.sensor.nearest(eye).and_then(|w| Foe::of(m, w, everyone)) else {
        stand_npc(m);
        return false;
    };
    let (standoff, near) = (standoff(m, game), near_bound(m, game));
    let moves = fight(m, g, goal, foe, false, &Leash::default(), standoff, near, &label, dt);
    if !moves {
        stand_npc(m);
    } else if let Some(n) = m.npc.as_mut() {
        // (its aim the way it goes while it moves: `npc_ai` turns its "camera" to `Npc::face`)
        n.face = m.yaw;
    }
    moves
}

/// A test map NPC's goal stands: its walk stops and it faces the way it is (`Npc::face`, which its
/// stand turns it to).
fn stand_npc(m: &mut Player) {
    if m.enemy.drive.is_some() {
        if let Some(n) = m.npc.as_mut() {
            n.face = m.yaw;
        }
    }
    stand(m);
}

/// BF_GOAL_WATCH=<s>: every s seconds of the controlled character's clock, a row for each level
/// character (with a brain, not following) and each test map NPC set to fight: its goal (`Goals`),
/// where it is, its own point, its route's entry and length, its target and how far, its pace level.
fn watch(player: Res<Player>, squad: Res<Squad>, nav: Res<super::nav::Nav>, mut next: Local<f32>) {
    let Some(every) = std::env::var("BF_GOAL_WATCH").ok().and_then(|v| v.parse::<f32>().ok()).filter(|&v| v > 0.0) else { return };
    let now = player.sim_time;
    if now < *next {
        return;
    }
    *next = (now / every).floor() * every + every;
    for m in squad.0.iter().filter(|m| !m.dead && !m.absent) {
        let brainy = m.brain.as_ref().is_some_and(|b| !b.follows());
        if !(brainy || m.npc.is_some_and(|n| n.fight && m.brain.is_none())) {
            continue;
        }
        let own = m.router.own.point.zip(nav.graph.as_ref()).map_or("-".to_string(), |(p, g)| format!("h_{:08x}", g.nodes[p as usize].name));
        let route = m.router.walker.as_ref().filter(|w| w.walking()).map_or("-".to_string(), |w| format!("{}/{}", w.index, w.entries.len()));
        let foe = m.goals.foe.map_or("-".to_string(), |(w, d)| format!("{w:?} {d:.1} m"));
        println!("watch t={now:.2} {}: {} at ({:.2}, {:.2}, {:.2}) own {own} route {route} target {foe} pace {} asked {} speed {:.2}", unit_name(m), m.goals.now(),
                 m.position.x, m.position.y, m.position.z, m.router.cap, m.router.asked, m.enemy.velocity.length());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FUN_0002e800's budget by the distance (and x1.2 over a 10 m height difference).
    #[test]
    fn the_approach_budget() {
        assert_eq!(budget(8.0, 0.0), 20.0, "x2.5 under 10 m");
        assert_eq!(budget(10.0, 0.0), 20.0, "x2 from 10 m");
        assert_eq!(budget(30.0, 0.0), 52.5, "x1.75 from 25 m");
        assert_eq!(budget(50.0, 0.0), 75.0, "x1.5 from 50 m");
        assert!((budget(30.0, -12.0) - 63.0).abs() < 1e-4, "x1.2 more 12 m below");
        assert_eq!(budget(30.0, 10.0), 52.5, "not at 10 m exactly");
    }

    /// FUN_0002e9c0's stop: (the near bound + 3 x the stand-off) / 4 (the Red Hand's and the
    /// mutants' 15 and 30: 26.25 m; the Supermutant's 6 and 12: 10.5 m).
    #[test]
    fn the_approach_stops_short() {
        assert_eq!(stop_short(30.0, 15.0), 26.25);
        assert_eq!(stop_short(12.0, 6.0), 10.5);
    }

    /// FUN_00023630: no home or the bit clear, anywhere; else inside the radius across and the
    /// height span round the home, its top excluded and its bottom too.
    #[test]
    fn the_home_leash() {
        let off = Leash { at: Some(Vec3::ZERO), ..Leash::of(10.0, 8.0, true) };
        assert!(off.inside(Vec3::new(500.0, 0.0, 0.0)), "a home-point spawn's leash is off");
        let on = Leash { at: Some(Vec3::ZERO), ..Leash::of(10.0, 8.0, false) };
        assert!(on.inside(Vec3::new(9.0, 3.9, 0.0)));
        assert!(!on.inside(Vec3::new(10.5, 0.0, 0.0)), "beyond the radius");
        assert!(!on.inside(Vec3::new(0.0, 4.0, 0.0)), "at the top of the span");
        assert!(!on.inside(Vec3::new(0.0, -4.0, 0.0)), "at its bottom");
        let small = Leash::of(3.0, 0.0, false);
        assert_eq!((small.radius, small.height), (7.0, 7.0), "held at 7 m at least");
        let none = Leash::of(-1.0, -1.0, false);
        assert_eq!((none.radius, none.height), (100.0, 20.0), "the constructor's when not given");
        assert!(Leash { at: None, ..none }.inside(Vec3::splat(1e4)), "no home: anywhere");
    }

    /// FUN_000235d0: the margin inside the home area is the lower of the whole span less the
    /// height over the home and the radius less the distance across; FLT_MAX with the leash off.
    /// FUN_00024080: at the edge (under 1 m in, or outside) with the target outward, not homeward.
    #[test]
    fn the_home_areas_margin_and_edge() {
        let area = Leash { at: Some(Vec3::ZERO), ..Leash::of(7.0, 7.0, false) };
        assert_eq!(area.margin(Vec3::new(3.0, 0.0, 0.0)), 4.0, "4 m in across");
        assert_eq!(area.margin(Vec3::new(1.0, 5.0, 0.0)), 2.0, "the height: 7 less 5");
        assert!(area.margin(Vec3::new(9.0, 0.0, 0.0)) < 0.0, "outside");
        assert_eq!(Leash { on: false, ..area }.margin(Vec3::new(50.0, 0.0, 0.0)), f32::MAX, "off");
        let me = Vec3::new(6.5, 0.0, 0.0);
        assert!(area.at_edge(me, Vec3::new(30.0, 0.0, 0.0)), "0.5 m in, the target outward");
        assert!(!area.at_edge(me, Vec3::new(-30.0, 0.0, 0.0)), "the target homeward");
        assert!(!area.at_edge(Vec3::new(3.0, 0.0, 0.0), Vec3::new(30.0, 0.0, 0.0)), "4 m in");
        assert!(!Leash { on: false, ..area }.at_edge(me, Vec3::new(30.0, 0.0, 0.0)), "the leash off");
    }

    /// Which drawn goals approach, and how their move is held: GOAL_ATTACK by the pace level and
    /// near its stop, ATTACK_VARY (and DEBUG1's) at its mode at once; the guards stand.
    #[test]
    fn which_goals_approach() {
        assert_eq!(approaches(GOAL_ATTACK), Some(true));
        assert_eq!(approaches(GOAL_ATTACK_VARY), Some(false));
        assert_eq!(approaches(GOAL_DEBUG1), Some(false));
        // (GOAL_ATTACK_GUARD 14, ATTACK_GUARD_CROUCH 43)
        assert_eq!(approaches(14), None);
        assert_eq!(approaches(43), None);
    }
}
