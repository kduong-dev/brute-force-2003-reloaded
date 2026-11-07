//! The squad's routes (#335, the second slice of #242; viewer/docs/nav/squad-on-routes.md "The
//! squad on routes"): the AI squad's follow and its Move To walk routes over the level's world
//! points (bf/nav.rs: `Graph::plan`, FUN_000416a0; `Walker`, FUN_00025730 / FUN_00026d80), and the
//! AI update's three steps on the brain's route that come before its goal (FUN_0001afe0,
//! 0x1b17f-0x1b18b): the route round a friend in the way (FUN_00041870), the catch-up to the own
//! point (FUN_00041ff0) and the 2 s stuck handling (FUN_000426b0). Every function cited is in
//! viewer/docs/code-map/ai.md ("AI"); the reading is todo/242-ai-routes/leads.md §3-§4 with the
//! corrections nav/corrections-to-todo-242.md lists.
//!
//!  - **One route record.** A brain walks one route, brain+0x930: the goal holder's +0x8f0 (the
//!    holder is brain+0x40) that MOVE_OUT's update FUN_00045fa0, PATROL's FUN_0002b490 and
//!    RETURN_HOME's FUN_00034690 hand FUN_00026d80, and the follow's own (FUN_00022130). So the
//!    stuck handling and the blocker route, which the AI update runs on brain+0x930 whatever the
//!    goal, apply to Move To as to the follow (`Router::walker`).
//!  - **The own points** (FUN_0014f860) of the controlled character (the character update keeps
//!    its point while its brain idles under the player: a route to a character ends there) and
//!    of the AI squad and its followers, every frame (`keep_point`).
//!  - **The follow** (#331, play_follow.rs, from play.rs `squad_ai` in FOLLOW and WINGMAN): the
//!    game's follow states, whose chase walks a route to the leader's own point (state 0) and whose
//!    walk to a cover point near him (state 5) another, both through `plan` and `step` here.
//!  - **Move To** (`move_out`, from play_orders.rs): GOAL_MOVE_OUT's route to the cursor's point
//!    (0x45d00), walked at 2 m to its end (FUN_00045fa0).
//!  - **The liquid guard** (play_liquids.rs `burning_ahead`): a safety net only. The game keeps
//!    out of lava by the graph's layout (no point lies over a harmful liquid, nav/lava.md "Lava"),
//!    and the corridor rays pass through liquids; a member whose next step would still go into one
//!    holds there (`shore`).
//!  - **The pace record** (#344, `Router::cap`, `pace_ramp`): the level the move is held to,
//!    starting at a stand and stepping toward the move asked, up and down, with the drops that set
//!    it down for a while (the stuck detours, the way round a friend, the follow's hold, the hard
//!    turn, the door step, the feelers). The walk's direction is the steering target through the
//!    AI's avoidance (play_avoid.rs, the callers').
//!  - **The door step** (#346, `door_step`, FUN_00026820): at a door point whose next entry is a
//!    door point too, the corridor between them blocked, a half-second stand with the feelers off.
//!
//! Test hooks: BF_ROUTE_LOG=1 (each plan and why, each point reached, corner held, skip,
//! catch-up, stuck, detour, re-route, warp, hard turn and blocker step of the AI squad's routes;
//! =2 also their own points' changes and the pace level's steps), BF_TEST_WARP=<s>,<who>,<x>,<z>[,<heading>][;...] (the controlled
//! character, `me`, or a member put on the floor at x, z at time s, facing the heading in degrees,
//! 0 = -z: the takes' trainer `tp`), BF_TEST_PRISM=1 (every character counts as inside one of the
//! level's prisms,
//! brain+0x3c4 bit 10, which the demo doesn't read: the walk's 1 m radius, the dash a run, and
//! FUN_00041870's route round a friend, which only runs then).

use bf_viewer::bf::level::WorldPoint;
use bf_viewer::bf::nav::{self, Dest, Entry, Graph, NoRoute, OwnChange, OwnPoint, Route, Step, Walker};

use super::*;

/// The BF_TEST_WARP hook, after the frame's input and before the characters' step.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, warp_hook.after(read_input).before(update_player).run_if(in_state(AppState::Playing)));
}

/// FUN_000426b0: the time (s) without the squared distance to the current target falling
/// (route+0x1c over 2.0, 0x4267c) that counts as stuck.
const STUCK_AFTER: f32 = 2.0;
/// The stuck test's progress (FUN_000426b0, 0x42760-0x4278f): the squared distance `d2` to the
/// target under the best `best`, or equal to it, is the new best. The demo's one change: a member
/// asking to move (`pushing`) whose distance is its best's to the bit hasn't moved at all, and is
/// no closer. The game's characters aren't held to the bit: its AI has no ledge refusal (its routes
/// jump between a take-off's and a landing's points, FUN_00026520 / FUN_000267b0) and its physics
/// leaves a body pressed into a wall moving; the demo's AI ledge stop (play.rs `ledge_ahead`) puts
/// a body back exactly where it was every frame. (#344's review: the tutorial's lava yard, both
/// marines at the edge of a jump's landing, h_0656d0f4, kind 0x1b, 10 m over the route's next point
/// h_03090779, for good and never stuck.) Any movement at all is progress when it comes closer, as
/// the game's.
fn progress(d2: f32, best: f32, pushing: bool) -> bool {
    d2 < best || (d2 == best && !pushing)
}

/// FUN_000425b0: the point's position + 0.25 m (0x3e800000) up, where the third stuck in a row at
/// one own point puts the character: the route's current entry's point (EAX at 0x42839-0x42857,
/// todo/344-ai-avoidance leads §7), the one it couldn't reach.
const WARP_LIFT: f32 = 0.25;
/// FUN_00042280's grace after a detour (route+0x1c = -20.0, 0xc1a00000) and FUN_00042490's after
/// a re-route (-5.0, 0xc0a00000). As read, FUN_000426b0 sets route+0x1c to 0 on every update the
/// pace level is under 2, and both set it to 1 for a while (below), so the grace is only the
/// level's.
const DETOUR_GRACE: f32 = -20.0;
const REROUTE_GRACE: f32 = -5.0;
/// The pace level ([brain+0x1a8]+0, `Router::cap`: 1 stand, 2 walk, 3 run, 4 dash), which
/// FUN_0004d970's move is held to (`pace`), set down with its timer (+0x28, FUN_000124f0) for
/// this long (s): FUN_00042280's detour 1 for 1.0, FUN_00042490's re-route 1 for 1.5,
/// FUN_00041870's route round a friend at most 3 for 0.5 and its wait 1 for 0.5 (the follow's
/// hold, state 2, is play_follow.rs's HOLD_STAND). The ramp climbs back once the timer runs out
/// (`ai_update`).
const DETOUR_HOLD: f32 = 1.0;
const REROUTE_HOLD: f32 = 1.5;
const BLOCKER_RUN: f32 = 0.5;
/// The pace level's top, the dash (FUN_00058b10's level 4).
pub const NO_CAP: i32 = 4;
/// The pace ramp's step (s, the record's +0x20, FUN_00058670 each AI frame): 0.17 (0x3e2e147b)
/// for a squad brain (brain+0x3c4 bit 8) in FOLLOW or WINGMAN (brain+0x58 1 or 4; or with +0x770
/// set, not read), 1.0 (0x3f800000) for any other. Run out, its timer lets the level step one
/// notch toward the move asked, up (FUN_000589b0 to 2, FUN_00058a60 to 3, FUN_00058aa0 to 4) or
/// down (FUN_00058b10 to 3, FUN_00058aa0 to 2, FUN_00058a20 to 1), and starts again at the step
/// (PACE_STAND for the step down to 1): a member of the follow climbs from a stand to the dash in
/// 0.34 s, one on a Move To in 2 s.
const PACE_STEP_SQUAD: f32 = 0.17;
const PACE_STEP: f32 = 1.0;
/// The pace record's timer at the brain's init (FUN_00058530: the level 1, the timer 0.3 s,
/// 0x3e99999a, running) and after the step down from a walk to a stand (FUN_00058a20).
const PACE_STAND: f32 = 0.3;
/// The pace record's two distances (m, +0xc and +0x10), which FUN_00058670 sets each AI frame with
/// the step: a squad brain in FOLLOW or WINGMAN 2.7 (0x402ccccd) and 4.0 (0x40800000)...
const PACE_WALK_SQUAD: f32 = 2.7;
const PACE_RUN_SQUAD: f32 = 4.0;
/// ...any other 3.2 (0x404ccccd) and 6.0 (0x40c00000), a squad member on a Move To among them
/// (`pace_near`: a walk within the first of the route's end, at most a run within the second).
const PACE_WALK: f32 = 3.2;
const PACE_RUN: f32 = 6.0;
/// FUN_000587d0, the pace at a corner (`corner_pace`): the cosine between the way to the current
/// point and the leg after it under this (0x3a4ff0, about 84 degrees) is a sharp corner, a walk...
const CORNER_SHARP: f32 = 0.1;
/// ...under this (0x3a5280, about 69 degrees) a notch under the move asked...
const CORNER_BENT: f32 = 0.365;
/// ...and a gentler one a walk only while the character's move is off the way to the point (their
/// cosine under this, 0x3a5038, about 53 degrees)...
const CORNER_OFF: f32 = 0.6;
/// ...within this of it (m, 3D: 5.76, 0x3a527c, its square)...
const CORNER_NEAR: f32 = 2.4;
/// ...each with the pace record's timer at this (s, 0x3fc00000), so the ramp climbs back after it.
const CORNER_HOLD: f32 = 1.5;
/// FUN_0004c530 / FUN_0004c480: a direction across shorter than this squared (0x3a4ffc) has none
/// (the corners' pace here; play_follow.rs `turn_check`).
pub(super) const FLAT_MIN: f32 = 1.192_092_9e-7;
/// FUN_0001ad50 (the AI update at 0x1b16b, after the ramp): a route walker at a run or a dash
/// turning faster than this (rad/s, the character's +0x488)...
const HARD_TURN: f32 = 2.5;
/// ...for over this long (s, brain+0x35c counting it) drops to a walk...
const HARD_TURN_TIME: f32 = 1.0;
/// ...with the pace record's timer at this (s, 0x3f800000).
const HARD_TURN_WALK: f32 = 1.0;
/// The door step (FUN_00026820, `door_step`): with the corridor between the two door points
/// blocked the pace level 1 with its timer at this (s, 0x3f000000)...
const DOOR_STAND: f32 = 0.5;
/// ...and the feelers off for this (s, brain+0x924 = 0x3fc00000).
const DOOR_FEELERS_OFF: f32 = 1.5;
/// FUN_00041870: a friend within 8 m (3D, 64 its square), its pace level under 3, whose own point is
/// the member's current point, its own point or its skipped one, and lying the member's way (the
/// x and z of the two unit directions, to it and to the current point, dot over 0.75).
const BLOCKER_NEAR: f32 = 8.0;
const BLOCKER_DOT: f32 = 0.75;
/// FUN_00041870's flood round it: within 15 m (0x41700000 at the filter's +0x110) of the member's
/// own point, to the route's next point (the filter 0x41070 takes only that one; its inner test
/// 0x262a80 returns 1).
const BLOCKER_FLOOD: f32 = 15.0;
/// FUN_00041870: no way round and the friend a human player's (FUN_00112800): the member waits,
/// counting brain+0x370 by the frame's time; at 7 s in MOVE_OUT it's GUARD (the full change) and
/// asks 12d51a1c (0x41fa2: no block in any chatter file, silent).
const BLOCKER_WAIT: f32 = 7.0;
const LINE_BLOCKED: u32 = 0x12D5_1A1C;

/// A character's route state: its own world point and its brain's route (the fields of the brain
/// the route code reads), kept with the character through the hand-over (`Player::router`).
#[derive(Clone, Debug)]
pub struct Router {
    /// char+0x24c, +0x250 and its timers (FUN_0014f860), wanted from the spawn with its walk on
    /// (FUN_00017b60: +0x250 |= 1 | 4)
    pub own: OwnPoint,
    /// the own point's position, for the hand-over's Move To to it (FUN_0005cc20 sends the
    /// members to the one left's world point)
    pub own_at: Option<Vec3>,
    /// brain+0x930 with brain+0x478 (its target) and +0x9dc (its corner point); None while empty
    pub walker: Option<Walker>,
    /// what the route goes to, as planned (a detour or a re-route plans to it again)
    pub dest: Option<Dest>,
    /// MOVE_OUT's route was planned for this order (the point as ordered)
    ordered: Option<Vec3>,
    /// where the character was at its last walk step (FUN_00025870's from)
    last: Option<Vec3>,
    /// brain+0x9c0 / +0x9c4: the own point at the last stuck, and how many times in a row since
    stuck_at: Option<u32>,
    stuck_count: u32,
    /// [brain+0x1a8]+0, the pace level its move is held to, and its timer's time left (s,
    /// +0x2c; 0 run out)
    pub cap: i32,
    cap_left: f32,
    /// the move it asked for ([brain+0x1a8]+4: 1 stand, 2 walk, 3 run, 4 dash; `pace` writes it,
    /// the ramp climbs toward it)
    pub asked: i32,
    /// brain+0x370: seconds waiting behind a friend with no way round
    wait: f32,
    /// a mode change dropped its goal (FUN_00018040's goal reset): the route goes at its next
    /// AI update, where the graph is at hand
    pub stale: bool,
    /// the brain's avoidance fields (play_avoid.rs, #344): the feelers-off timer, the speed
    /// record's timer, char+0x720 as it stands, the characters near it
    pub avoid: avoid::Avoid,
    /// brain+0x1b4: a position of the brain's own the pace goes by instead of the route's end
    /// (FUN_0004d840's first branch, `pace_near`): GOAL_ATTACK's and ATTACK_VARY's approach set it
    /// to where they stop short of the target (FUN_0002e9c0 at holder+0x174, FUN_0002d710 at
    /// holder[0x5d]: #336, play_goals.rs) and clear it when the approach ends; None (zero) else
    pub stop: Option<Vec3>,
    /// The pace record's +0x24 (-1, off, from FUN_00058530): when over 0 it is the length of every
    /// notch's timer in the ramp (FUN_000589e0, FUN_00058a60, FUN_00058ad0 take it over the +0x20
    /// step, `pace_ramp`). ATTACK_FALLBACK's update sets it to 0.1 s each frame (0x31899,
    /// `fallback::STEP_TIME`), so a character falling back climbs to its run in tenths of a second;
    /// the pace near a route's end (FUN_00058590) sets it back to -1 when it forces a walk
    /// (0x585bb, 0x585cb: `end_pace`). Nothing else writes it, so it stays after the goal.
    pub step_time: f32,
}

impl Default for Router {
    /// A spawned character's: its own point wanted, no route, its pace record as the brain's init
    /// makes it (FUN_00058530: the level 1, a stand, its timer at PACE_STAND, +0x24 at -1).
    fn default() -> Self {
        Router { own: OwnPoint::spawned(), own_at: None, walker: None, dest: None, ordered: None, last: None, stuck_at: None, stuck_count: 0,
                 cap: 1, cap_left: PACE_STAND, asked: 1, wait: 0.0, stale: false, avoid: avoid::Avoid::default(), stop: None, step_time: -1.0 }
    }
}

impl Router {
    /// Whether a route is under way (brain+0x934 set and entries in it).
    pub fn walking(&self) -> bool {
        self.walker.as_ref().is_some_and(|w| w.walking())
    }

    /// FUN_0004e8f0 on its route: the destination and avoid bits it set cleared, the entries gone.
    pub fn drop_route(&mut self, g: &mut Graph) {
        if let Some(mut w) = self.walker.take() {
            w.reset(g);
        }
        self.dest = None;
        self.ordered = None;
    }

    /// The pace level set down to `cap` with its timer at `time` s ([brain+0x1a8]+0 and
    /// FUN_000124f0 on its +0x28): the drops (DETOUR_HOLD and the others, the hard turn's, the
    /// door step's, the feelers').
    pub fn set_pace(&mut self, cap: i32, time: f32) {
        self.cap = cap;
        self.cap_left = time;
    }
}

/// FUN_0004d970's move for a route walker that asks for `asked` (2 walk, 3 run, 4 dash): the mode
/// asked goes to the pace record's +4, which the level climbs toward (`ai_update`), and it moves
/// at the lower of the two (the fourth argument set, as every route walker's is): `m.walk` at 2,
/// `m.sprint` at 4 (a run in a prism, BF_TEST_PRISM), and at level 1 the stick stopped (the game
/// scales it to 0.01: a stand). True when it moves.
pub fn pace(m: &mut Player, asked: i32) -> bool {
    m.router.asked = asked;
    let at = asked.min(m.router.cap);
    m.walk = at == 2;
    m.sprint = at >= 4 && !in_prism();
    if at <= 1 {
        m.move_input = Vec2::ZERO;
    }
    at > 1
}

/// Whether `m` has the squad's pace record settings (FUN_00058670: a squad brain, bit 8, in
/// FOLLOW or WINGMAN, brain+0x58 1 or 4; char+0x770 not read): the ramp's PACE_STEP_SQUAD and the
/// distances PACE_WALK_SQUAD and PACE_RUN_SQUAD. A test map NPC set to follow has them as a
/// squad member's (#364: it runs the squad's follow in FOLLOW, so it should climb to the dash
/// as the squad does).
fn squad_pace(m: &Player) -> bool {
    (m.in_squad() || m.follow_set()) && matches!(m.command.mode, brain::Mode::Follow | brain::Mode::Wingman)
}

/// FUN_0004d840, the pace near the route's points, which FUN_0004d970 runs after it writes the
/// move asked (`asked`, here `Router::asked`) when its fifth argument is set: MOVE_OUT's update
/// (FUN_00045fa0) sets it, and so do the enemies' route goals (RETURN_HOME, PATROL, PURSUE and
/// GOAL_ATTACK's approach, #336: play_goals.rs); the follow's walk (state 0, FUN_00022130) doesn't
/// (0x22377 pushes 0), nor do its states 5, 6 and 10, so the follow isn't slowed by it. With a
/// position of the brain's own at brain+0x1b4 (`Router::stop`: GOAL_ATTACK's approach sets it to
/// where it stops short of its target, #336; the brain's init, FUN_00016ff0, and the follow's state
/// 10 zero it) the pace goes by the distance across to that, FUN_00058610 as at a route's end, and
/// no corner is looked at. Else, with a route:
///  - FUN_00058610 / FUN_00058590: the distance across from the character to the route's last
///    entry (FUN_00040fc0: its point's position, else its own) within the record's walk distance
///    (+0xc: PACE_WALK, PACE_WALK_SQUAD) with the move asked 2 or more: the move asked 2 and the
///    level at most 2 (+0x24 set to -1 too: not read); else within the run distance (+0x10) with
///    the move asked 3 or more: the move asked 3 and the level at most 3. Take08's Hawk on her
///    Move To went from the dash to a run 5.6 m out and to a walk 2.8 m out (#318; at 10 Hz);
///  - with neither, and an entry after the current one (FUN_000233a0 on both), the corner's
///    (`corner_pace`, FUN_000587d0), whose answer FUN_0004d840 drops.
///
/// FUN_0004d970 then moves at the lower of the move asked and the level (`pace`), so the caller
/// reads both after this. BF_ROUTE_LOG=2 says when the end or a corner slows it.
pub fn pace_near(m: &mut Player, g: &Graph) {
    let (walk, run) = pace_distances(m);
    let me = m.position;
    let moving = avoid::moving(m);
    // (brain+0x1b4 set: by the distance to it alone, 0x4d84c-0x4d86f)
    if let Some(stop) = m.router.stop {
        let off = me.xz().distance(stop.xz());
        let was = (m.router.asked, m.router.cap);
        if let Some(why) = end_pace(&mut m.router, off, walk, run) {
            log_pace(m, why, was, off);
        }
        return;
    }
    let Some(w) = m.router.walker.as_ref().filter(|w| w.walking()) else { return };
    let last = w.entry_at(g, w.entries.len() as i32 - 1);
    let corner = w.entry_at(g, w.index).zip(w.entry_at(g, w.index + 1));
    let r = &mut m.router;
    let was = (r.asked, r.cap);
    let off = last.map_or(f32::MAX, |p| me.xz().distance(p.xz()));
    let why = match end_pace(r, off, walk, run) {
        Some(why) => why,
        None => match corner {
            Some((cur, next)) if corner_pace(r, me, moving, cur, next, walk) => "slows at a corner",
            _ => return,
        },
    };
    log_pace(m, why, was, off);
}

/// The pace near the end of a walk without a route, toward `end` (MOVE_OUT's straight walk on a
/// map without points: the demo's, whose game counterpart, FUN_00045fa0 with an empty route,
/// stands): FUN_00058590 by the distance across to it, as FUN_0004d840 takes it to a route's end
/// (`pace_near`).
pub fn pace_to(m: &mut Player, end: Vec3) {
    let (walk, run) = pace_distances(m);
    let off = m.position.xz().distance(end.xz());
    let was = (m.router.asked, m.router.cap);
    if let Some(why) = end_pace(&mut m.router, off, walk, run) {
        log_pace(m, why, was, off);
    }
}

/// The pace record's walk and run distances for `m` (+0xc, +0x10: FUN_00058670's, `squad_pace`).
fn pace_distances(m: &Player) -> (f32, f32) {
    if squad_pace(m) { (PACE_WALK_SQUAD, PACE_RUN_SQUAD) } else { (PACE_WALK, PACE_RUN) }
}

/// FUN_00058590 (from FUN_00058610 with the distance across `off` to the end of the way): within
/// `walk` (or at it) with the move asked 2 or more, the move asked 2 and the level at most 2;
/// else within `run` with the move asked 3 or more, the move asked 3 and the level at most 3. What
/// it did, for the log; None when nothing (FUN_0004d840 then looks at the corner).
fn end_pace(r: &mut Router, off: f32, walk: f32, run: f32) -> Option<&'static str> {
    if off <= walk {
        if r.asked < 2 {
            return None;
        }
        (r.asked, r.cap) = (2, r.cap.min(2));
        r.step_time = -1.0;
        return Some("walks near its way's end");
    }
    if off <= run && r.asked >= 3 {
        (r.asked, r.cap) = (3, r.cap.min(3));
        return Some("runs at most near its way's end");
    }
    None
}

/// BF_ROUTE_LOG=2: the pace near the route's points set the level down (`was` the move asked and
/// the level before), `off` m from the way's end. (The goal asks again each frame, so the move
/// asked is lowered every frame near the end; a change of the level is logged once.)
fn log_pace(m: &Player, why: &str, was: (i32, i32), off: f32) {
    if log_level() >= 2 && m.router.cap != was.1 {
        println!("t {:.2}: {}: {why} (FUN_0004d840): the move asked {} -> {}, its pace level {} -> {} ({off:.1} m off its end)", m.sim_time, unit_name(m), was.0,
                 m.router.asked, was.1, m.router.cap);
    }
}

/// FUN_000587d0 (the pace record, the current entry's position `cur`, the next's `next`), for a
/// character at `me` moving `moving` (FUN_0011e680: its move across, unit; None standing): with
/// the level over 1 and `me` within the walk distance `walk` of `cur` (3D, strictly):
///  - `a` the way from `me` to `cur` and `b` the leg from `cur` to `next`, each across and unit
///    (FUN_0004c530, FUN_0004c480; nothing when either is shorter than FLAT_MIN squared);
///  - their cosine under CORNER_SHARP: the timer at CORNER_HOLD, and with the move asked over 2
///    the level 2 (a walk);
///  - under CORNER_BENT: the timer, and the level at most a notch under the move asked (with the
///    move asked 2 or less, the level 2);
///  - else, the move's cosine with `a` under CORNER_OFF and `me` within CORNER_NEAR of `cur`
///    (3D): as a sharp corner; standing, or moving its way, nothing.
///
/// The move asked is left as it is, so once the timer runs out the ramp climbs back a notch a
/// step (`pace_ramp`). True when it acted (the timer set; FUN_0004d840 drops the answer).
fn corner_pace(r: &mut Router, me: Vec3, moving: Option<Vec2>, cur: Vec3, next: Vec3, walk: f32) -> bool {
    if r.cap <= 1 || cur.distance_squared(me) >= walk * walk {
        return false;
    }
    let flat = |v: Vec3| {
        let d = Vec2::new(v.x, v.z);
        (d.length_squared() >= FLAT_MIN).then(|| d.normalize())
    };
    let (Some(a), Some(b)) = (flat(cur - me), flat(next - cur)) else { return false };
    let cos = a.dot(b);
    if (CORNER_SHARP..CORNER_BENT).contains(&cos) {
        r.cap_left = CORNER_HOLD;
        r.cap = if r.asked <= 2 { 2 } else { r.cap.min(r.asked - 1) };
        return true;
    }
    if cos >= CORNER_BENT && !(moving.is_some_and(|v| v.dot(a) < CORNER_OFF) && cur.distance_squared(me) < CORNER_NEAR * CORNER_NEAR) {
        return false;
    }
    r.cap_left = CORNER_HOLD;
    if r.asked > 2 {
        r.cap = 2;
    }
    true
}

/// BF_ROUTE_LOG's level (2: the own points' changes too), read once.
fn log_level() -> u8 {
    static LEVEL: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
    *LEVEL.get_or_init(|| std::env::var("BF_ROUTE_LOG").ok().map_or(0, |v| v.trim().parse().unwrap_or(1)))
}

/// BF_TEST_PRISM=1: every character counts as inside one of the level's prisms (brain+0x3c4 bit
/// 10, FUN_0004cb60), whose records (the level record's +0x114) the demo doesn't read. Read once.
pub fn in_prism() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_TEST_PRISM").is_ok_and(|v| v != "0"))
}

/// A point's name and place, for the log.
fn label(g: &Graph, i: Option<u32>) -> String {
    match i {
        Some(i) => {
            let n = &g.nodes[i as usize];
            format!("h_{:08x} ({:.1}, {:.1}, {:.1})", n.name, n.at.x, n.at.y, n.at.z)
        }
        None => "a position".into(),
    }
}

/// A route's entries for the log: each point's name and place, a bare position's place.
fn list_entries(g: &Graph, w: &[Entry]) -> String {
    w.iter().map(|e| match e.point {
        Some(p) => format!("h_{:08x} ({:.1}, {:.1})", g.nodes[p as usize].name, e.at.x, e.at.z),
        None => format!("the place ({:.1}, {:.1})", e.at.x, e.at.z),
    }).collect::<Vec<_>>().join(" ")
}

/// The world's ray as the route code asks it (play_nav.rs `clear`, FUN_0004ce30).
fn ray(a: Vec3, b: Vec3, max: f32) -> bool {
    let arena = world::arena();
    super::nav::clear(arena.as_deref(), a, b, max)
}

/// FUN_0014f860 for `u`, a frame of `dt` s: its own point kept from its position (its hips, the
/// game's transform position), while it's alive and in the world. BF_ROUTE_LOG=2 logs a change.
pub fn keep_point(u: &mut Player, g: &Graph, dt: f32) {
    if u.dead || u.absent {
        return;
    }
    let jumping = matches!(u.action, Action::JumpCrouch | Action::JumpLaunch | Action::JumpFall | Action::JumpLand);
    let change = u.router.own.update(g, u.position, dt, jumping, &ray);
    u.router.own_at = u.router.own.point.map(|p| g.nodes[p as usize].at);
    if let (Some(c), true) = (change, log_level() >= 2) {
        let (how, i) = match c {
            OwnChange::Near(i) => ("the near walk", i),
            OwnChange::Deep(i) => ("the 6-link search", i),
            OwnChange::Wide(i) => ("the 10-link search", i),
            OwnChange::Far(i) => ("the nearest valid point", i),
        };
        println!("t {:.2}: {}: own point {} by {how}", u.sim_time, unit_name(u), label(g, Some(i)));
    }
}

/// Whether `u` walks routes in the demo: an AI squad member (`Player::in_squad`, no brain), a
/// level character's brain (the tutorial's marines following the leader, play_brain.rs, and since
/// #336 every other one: its walks home, its patrol's first leg, its pursuit and its attack's
/// approach, play_goals.rs) or a test map NPC set to follow (`follow::npc_follows`, #364: its route
/// code runs whichever side it's on; it only has a route while it follows) or to fight (#336: its
/// attack's approach, a straight walk on the test map's floor, play_goals.rs `npc_fight`). The
/// test map's dummies don't.
pub fn walks_routes(u: &Player) -> bool {
    (u.in_squad() && u.brain.is_none()) || u.brain.is_some() || u.follow_set() || u.npc.is_some_and(|n| n.fight)
}

/// A friend as FUN_00041870 looks at it: who, its side, its position, its own point, whether its
/// pace is under 3 (`mates`), whether a human player has it (FUN_00112800) and whether it's in a
/// prism.
#[derive(Clone, Copy, Debug)]
pub struct Mate {
    who: Who,
    team: u8,
    at: Vec3,
    point: Option<u32>,
    slow: bool,
    human: bool,
    prism: bool,
}

/// The characters FUN_00041870 looks through (the world's two character lists): the controlled
/// one and the squad's, alive and in the world. An AI one is slow while its pace level is under 3
/// (a walk or a stand: the level follows the move asked both ways since #344, `pace_ramp`); the
/// controlled one while it walks or stands (its brain idles: the demo's guess).
pub fn mates(player: &Player, squad: &[Player]) -> Vec<Mate> {
    let prism = in_prism();
    std::iter::once(player).chain(squad.iter()).filter(|u| !u.dead && !u.absent).map(|u| {
        let slow = if u.controlled { u.move_input.length() < 0.1 || u.walk } else { u.router.cap < 3 };
        Mate { who: u.who(), team: u.team, at: u.position, point: u.router.own.point, slow, human: u.controlled, prism }
    }).collect()
}

/// The positions of the characters of `mates` (every living one in the world, of any side) near
/// `m`, not itself: the push's look through the world's character lists (play_avoid.rs `push`,
/// the sensor's 0x521c0, which takes them within 1.5 m across; this keeps those within 2 m).
pub fn near(m: &Player, mates: &[Mate]) -> Vec<Vec3> {
    let me = m.who();
    mates.iter().filter(|x| x.who != me && x.at.xz().distance_squared(m.position.xz()) < NEAR * NEAR).map(|x| x.at).collect()
}

/// `near`'s reach (m, across): the push's 1.5 m (brain+0x3bc) with room to spare.
const NEAR: f32 = 2.0;

/// The AI update's pace ramp for `m` (FUN_00058670 at 0x1b119, each AI frame, whether or not it
/// has a route: play.rs runs it for every AI character the routes' goals move, with a level's
/// points or not): the pace record's timer runs, and once it has run out with the level not the
/// move asked, the level steps one notch toward it, up or down (the move asked is the last frame's,
/// as the goal writes it after), and the timer starts again at the step, PACE_STEP_SQUAD or
/// PACE_STEP, or PACE_STAND for the step down to a stand. So a member that stops steps down to 1
/// and climbs again from there when it sets off: walk, run, dash (#33 take04's motion numbers go
/// 47, 34, 8: walk, run, dash). Then the hard turn's drop (`hard_turn`, FUN_0001ad50).
pub fn pace_ramp(m: &mut Player, dt: f32) {
    let step = if squad_pace(m) { PACE_STEP_SQUAD } else { PACE_STEP };
    let r = &mut m.router;
    // (the record's +0x24 over 0 is each notch's timer, in place of the step: FUN_000589e0 / FUN_00058a60)
    let step = if r.step_time > 0.0 { r.step_time } else { step };
    r.cap_left = (r.cap_left - dt).max(0.0);
    if r.cap_left <= 0.0 && r.asked != r.cap {
        if r.asked > r.cap {
            r.cap = (r.cap + 1).min(NO_CAP);
            r.cap_left = step;
        } else {
            r.cap = (r.cap - 1).max(1);
            r.cap_left = if r.cap == 1 { PACE_STAND } else { step };
        }
        if log_level() >= 2 {
            println!("t {:.2}: {}: its pace level steps to {} toward {} (the next in {} s)", m.sim_time, unit_name(m), m.router.cap, m.router.asked, m.router.cap_left);
        }
    }
    hard_turn(m, dt);
}

/// FUN_0001ad50 for `m`, a frame of `dt` s (the AI update at 0x1b16b, right after the ramp):
/// walking a route at a run or a dash (the level over 2) and turning faster than HARD_TURN (its
/// turn over the last frame, the demo's stand-in for the character's +0x488), brain+0x35c counts
/// the time, and over HARD_TURN_TIME the level drops to a walk for HARD_TURN_WALK and the count
/// starts again; anything else zeroes it. What keeps a runner from circling a point it must come
/// within 0.6 m of (a corner hold), where its turn at speed is wider.
fn hard_turn(m: &mut Player, dt: f32) {
    let turned = m.router.avoid.yaw_was.replace(m.yaw).map_or(0.0, |y| wrap_angle(m.yaw - y).abs());
    let r = &mut m.router;
    if !(r.walking() && r.cap > 2 && dt > 0.0 && turned / dt > HARD_TURN) {
        r.avoid.turning = 0.0;
        return;
    }
    r.avoid.turning += dt;
    if r.avoid.turning <= HARD_TURN_TIME {
        return;
    }
    r.avoid.turning = 0.0;
    r.set_pace(2, HARD_TURN_WALK);
    if log_level() > 0 {
        println!("t {:.2}: {}: turning over {HARD_TURN} rad/s for {HARD_TURN_TIME} s: walks for {HARD_TURN_WALK} s (FUN_0001ad50)", m.sim_time, unit_name(m));
    }
}

/// The AI update's route steps for `m` before its goal (FUN_0001afe0, 0x1b17f-0x1b18b), when the
/// character has an own point: a route its goal dropped, the route round a friend in the way
/// (`blocker`, FUN_00041870), the catch-up (FUN_00041ff0, `Walker::resync`) and the stuck
/// handling (`stuck`, FUN_000426b0). `mates`: this frame's friends (`mates`). The pace ramp before
/// them is `pace_ramp`.
pub fn ai_update(m: &mut Player, mates: &[Mate], g: &mut Graph, dt: f32) {
    let r = &mut m.router;
    if std::mem::take(&mut r.stale) {
        r.drop_route(g);
    }
    if r.own.point.is_none() {
        return;
    }
    blocker(m, mates, g, dt);
    let own = m.router.own.point;
    let who = unit_name(m);
    if let Some(w) = m.router.walker.as_mut() {
        let was = w.index;
        if w.resync(own) && log_level() > 0 {
            println!("t {:.2}: {who}: catches its route up: its own point {} is entry {} (was {was})", m.sim_time, label(g, own), w.index);
        }
    }
    stuck(m, g, dt);
}

/// A route walk's step for `m` (FUN_00026d80 at `radius`, `last_radius` for the route's last
/// entry: the follow's walk to a cover point passes 1.0 and 0.1, FUN_00022790; at most 1 m in a
/// prism), logged; its position kept for the next step's pass test.
pub(super) fn step(m: &mut Player, g: &mut Graph, radius: f32, last_radius: f32) -> Step {
    let me = m.position;
    let last = m.router.last.replace(me).unwrap_or(me);
    let who = unit_name(m);
    let now = m.sim_time;
    let Some(w) = m.router.walker.as_mut() else { return Step::default() };
    let on_last = usize::try_from(w.index).is_ok_and(|i| i + 1 == w.entries.len());
    let held = w.corner;
    let s = w.step(g, me, last, in_prism(), if on_last { last_radius } else { radius }, &ray);
    // (BF_ROUTE_LOG=2: the walk's state once a second)
    if log_level() >= 2 && now.floor() != (now - capture_step()).floor() {
        println!("t {now:.2}: {who}: walking entry {} of {} toward ({:.2}, {:.2}) within {:.2} m, {:.2} m off, corner {}, stuck {:.2} s, best {:.2} m, cap {} ({:.1} s), at ({:.2}, {:.2})",
                 w.index, w.entries.len(), s.target.x, s.target.z, s.radius, me.xz().distance(s.target.xz()), label(g, w.corner), w.stuck, w.best.sqrt(), m.router.cap,
                 m.router.cap_left, me.x, me.z);
    }
    if log_level() > 0 {
        let at = format!("at ({:.1}, {:.1})", me.x, me.z);
        if s.corner.is_some() && s.corner != held {
            println!("t {now:.2}: {who}: held at the corner {}: its next leg is blocked, closes to {} m ({at})", label(g, s.corner), nav::CORNER_RADIUS);
        }
        if let Some((p, kind)) = s.reached {
            println!("t {now:.2}: {who}: reached {} (kind {kind}) within {:.1} m ({at})", if p.is_some() { label(g, p) } else { "its place".into() }, s.radius);
        }
        if let Some(k) = s.skipped {
            println!("t {now:.2}: {who}: skipped {}: the corridor on is clear", label(g, Some(k)));
        }
        if !s.walking {
            println!("t {now:.2}: {who}: the route's end ({at})");
        }
    }
    if !s.walking {
        m.router.walker = None;
    }
    door_step(m, g, &s, &ray);
    s
}

/// The walk's door step (FUN_00026820, from FUN_00026d80 at 0x27059 on a door point reached;
/// #346): when the point reached in `s` is a door point (kind 6) and the next entry's point is one
/// too (the pair either side of a door), the 0.8 m corridor between the two points at their
/// positions (FUN_000410a0, as the route's other tests) blocked: the pace level 1 with its timer at
/// DOOR_STAND (the ramp climbs again after it) and the feelers off for DOOR_FEELERS_OFF
/// (brain+0x924), so the member stands half a second at the first point and then walks through
/// with nothing turning it off its line. The walk moves on regardless (the step's return is
/// ignored, 0x27059-0x2707f): no wait for the door, no read of its state, no new plan. A door that
/// stays shut is left to the stuck handling (`stuck`). Its first branch, a world button at the
/// point (+0x3c, an OT_WORLDBUTTON: its vtable +0x138 posts TRIG_SIG_USE_OBJECT), never runs: no
/// point in the data has an object (every door point's h_e5851820 is empty and its h_0b6b92ba
/// record attaches nothing, todo/346-route-doors leads §1). `clear` is the world's ray (`ray`);
/// BF_ROUTE_LOG says what it found.
fn door_step(m: &mut Player, g: &Graph, s: &Step, clear: nav::Clear) {
    let Some((Some(p), nav::kind::DOOR)) = s.reached else { return };
    let Some(next) = m.router.walker.as_ref().and_then(|w| w.current()).filter(|&n| g.nodes[n as usize].kind == nav::kind::DOOR) else { return };
    let (a, b) = (g.nodes[p as usize].at, g.nodes[next as usize].at);
    let blocked = nav::corridor_blocked(a, b, nav::CORRIDOR, clear);
    if log_level() > 0 {
        println!("t {:.2}: {}: the door step from {} to {}: the corridor between them {}", m.sim_time, unit_name(m), label(g, Some(p)), label(g, Some(next)),
                 if blocked { format!("blocked: stands {DOOR_STAND} s, its feelers off {DOOR_FEELERS_OFF} s") } else { "clear: on through".into() });
    }
    if blocked {
        m.router.set_pace(1, DOOR_STAND);
        m.router.avoid.feelers_off = DOOR_FEELERS_OFF;
    }
}

/// Plans `m`'s route to `to` (FUN_000416a0 from its own point) and starts it (FUN_00025730 with
/// its third argument 1, as the follow's and MOVE_OUT's starts pass it), logged with `why`; the
/// route it had reset, its corner point kept (brain+0x9dc). The route found, or why none.
pub(super) fn plan(m: &mut Player, g: &mut Graph, to: Dest, why: &str) -> Result<(), NoRoute> {
    plan_within(m, g, to, why, None).map(|_| ())
}

/// `plan` with the search held within `within` m of the start (DAT_004021a8, which GOAL_ATTACK's
/// approach sets for its one search, FUN_0002e800: `Graph::search`'s radius); the route found
/// (its length for the approach's budget, FUN_00023460), or why none.
pub(super) fn plan_within(m: &mut Player, g: &mut Graph, to: Dest, why: &str, within: Option<f32>) -> Result<Route, NoRoute> {
    let me = m.position;
    let own = m.router.own.point;
    let clock = std::time::Instant::now();
    let plan = g.plan(me, own, to, within, &ray);
    let us = clock.elapsed().as_secs_f64() * 1e6;
    let log = log_level() > 0;
    let dest = match to {
        Dest::Point(p) => label(g, Some(p)),
        Dest::Beside { at, point } => format!("its place ({:.1}, {:.1}) beside {}", at.x, at.z, label(g, Some(point))),
        Dest::Character { point, .. } => format!("a character at {}", label(g, point)),
    };
    match plan {
        Ok((route, found)) => {
            if log {
                println!("t {:.2}: {}: route ({why}) from ({:.1}, {:.1}, {:.1}), own point {}, to {dest}: {} entries{}; the search popped {} and met {} of {} points in {us:.0} us: {}",
                         m.sim_time, unit_name(m), me.x, me.y, me.z, label(g, own), route.entries.len(),
                         route.dropped.map_or(String::new(), |d| format!(", the own point {} dropped", label(g, Some(d)))), found.popped, found.met, g.nodes.len(),
                         list_entries(g, &route.entries));
            }
            let corner = m.router.walker.take().and_then(|mut w| {
                let c = w.corner;
                w.reset(g);
                c
            });
            let mut w = Walker::start(g, &route, true, own);
            // (the corner point is the brain's, brain+0x9dc: a new route keeps it)
            w.corner = corner;
            m.router.walker = Some(w);
            m.router.dest = Some(to);
            m.router.last = Some(me);
            Ok(route)
        }
        Err(e) => {
            if log {
                let why_not = match e {
                    NoRoute::NoStart => "it has no world point",
                    NoRoute::NoGoal => "the destination has no point",
                    NoRoute::Unreachable => "unreachable (the heap ran dry)",
                };
                println!("t {:.2}: {}: no route ({why}) from {} to {dest}: {why_not} ({us:.0} us)", m.sim_time, unit_name(m), label(g, own));
            }
            Err(e)
        }
    }
}

/// Starts `m`'s walk of `entries`, a route found some other way than the search (FUN_00025730 with
/// its third argument 1, as FUN_00029a00 calls it after the link walk `Graph::away`: no first-entry
/// drop, FUN_00041320 belongs to the search's plan), logged with `why`; the route it had reset, its
/// corner point kept (brain+0x9dc). A stuck detour or re-route plans to the route's last point.
pub(super) fn start(m: &mut Player, g: &mut Graph, entries: Vec<Entry>, why: &str) {
    let (me, own) = (m.position, m.router.own.point);
    let route = Route { entries, dropped: None };
    let corner = m.router.walker.take().and_then(|mut w| {
        let c = w.corner;
        w.reset(g);
        c
    });
    let mut w = Walker::start(g, &route, true, own);
    w.corner = corner;
    if log_level() > 0 {
        println!("t {:.2}: {}: route ({why}) from ({:.1}, {:.1}, {:.1}), own point {}: {} entries, {:.1} m between its points, {:.1} m to walk from here: {}", m.sim_time, unit_name(m),
                 me.x, me.y, me.z, label(g, own), route.entries.len(), g.route_length(&route.entries), w.remaining(g, me), list_entries(g, &route.entries));
    }
    m.router.walker = Some(w);
    m.router.dest = route.entries.last().and_then(|e| e.point).map(Dest::Point);
    m.router.last = Some(me);
}

/// How the follow or Move To goes this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Way {
    /// toward `target` (the route's steering target, brain+0x478), `along` m of the walk left
    Go { target: Vec3, along: f32 },
    /// standing: the follow's holds, no route (the pace level at 1 is the caller's, `pace`)
    Stand,
    /// at the route's end, Move To's point reached
    There,
}

/// FUN_0004e8f0 on `m`'s route, when it has one: the follow's resets (state 0 within d2 of a
/// leader who isn't running and at its route's end, state 5 at its cover point; play_follow.rs).
pub fn stop(m: &mut Player, g: &mut Graph) {
    if m.router.walker.is_some() {
        m.router.drop_route(g);
    }
}

/// GOAL_MOVE_OUT on a route (its start 0x45d00, its update FUN_00045fa0): for each order (the
/// point as ordered, `point`), a route to the point (brain+0x8ac: the cursor's point, the order's
/// position matching it; for another position the valid point nearest it that the cursor would
/// take, bit 14 set and bit 15 clear), or with no point to its own point (FUN_00058b60); then
/// walked at 2 m (FUN_00026d80 at 2.0) to its end, at the pace the caller asks and the level
/// allows (`pace`). `There` at the end (the caller's GUARD and e635ecc6); `Stand` with no route (the update stands facing, its route empty: what the start
/// does when it finds none isn't read).
pub fn move_out(m: &mut Player, g: &mut Graph, point: Option<Vec3>) -> Way {
    if m.router.ordered != point || (m.router.walker.is_none() && m.router.dest.is_none()) {
        m.router.drop_route(g);
        let goal = point.and_then(|p| {
            g.nodes.iter().position(|n| n.at.distance_squared(p) < 1e-6).map(|i| i as u32).or_else(|| {
                g.nodes.iter().enumerate().filter(|(i, n)| g.valid(*i as u32) && n.flags & WorldPoint::BIT_14 != 0 && n.flags & WorldPoint::BIT_15 == 0)
                    .min_by(|a, b| a.1.at.distance_squared(p).total_cmp(&b.1.at.distance_squared(p))).map(|(i, _)| i as u32)
            })
        }).or(m.router.own.point);
        m.router.ordered = point;
        let Some(goal) = goal else { return Way::Stand };
        if plan(m, g, Dest::Point(goal), "Move To").is_err() {
            // (an empty route: it stands, MOVE_OUT kept; `dest` keeps the order planned)
            m.router.dest = Some(Dest::Point(goal));
            return Way::Stand;
        }
    }
    // (no route: the plan found none)
    if !m.router.walking() {
        return Way::Stand;
    }
    let s = step(m, g, nav::ARRIVE, nav::ARRIVE);
    if !s.walking {
        return Way::There;
    }
    let w = m.router.walker.as_ref().expect("walking");
    Way::Go { target: s.target, along: w.remaining(g, m.position) }
}

/// The liquid guard on `m`'s way `dir` (xz; the steered direction, play_avoid.rs `steer`)
/// (play_liquids.rs `burning_ahead`): the safety net behind the routes, true when its next step
/// would go into a liquid that burns. BF_LIQUID_LOG says so twice a second.
pub fn shore(m: &Player, dir: Vec2, dt: f32) -> bool {
    if dir.length() < 1e-4 {
        return false;
    }
    let held = world::arena().is_some_and(|a| liquids::burning_ahead(&a, m, dir.normalize()));
    if held && std::env::var("BF_LIQUID_LOG").is_ok() && (m.sim_time * 2.0).floor() != ((m.sim_time - dt) * 2.0).floor() {
        println!("t {:.2}: {} waits at a liquid that burns on its route (the routes' safety net), at {:.2}", m.sim_time, unit_name(m), m.position);
    }
    held
}

/// FUN_00041870 for `m` (from the AI update, 0x1b17f): with a route under way, every friend
/// (`mates`: not hostile, alive) within BLOCKER_NEAR whose pace is under 3, whose own point
/// is the member's current point, its own point or the point its look-ahead skipped, and that
/// lies its way (BLOCKER_DOT), is a blocker: its point gets +0x28 bit 3 (`AVOID`) for this step
/// (a point that had it already keeps it). With one, and the member or the first blocker inside
/// one of the level's prisms (brain+0x3c4 bit 10; or the blocker's brain+0x3c8 set, not
/// modelled):
///  - the first blocker's point the route's last: the last entry dropped (FUN_00040f70) and the
///    route done (route+0x20), so the walk ends there;
///  - else the flood from the member's own point to the route's next point round the marked ones
///    (FUN_000594a0, BLOCKER_FLOOD): found, the route becomes the flood's way and the old route's
///    entries after that point (FUN_00041810), started (FUN_00025730) with the cap at most 3 for
///    0.5 s; none and the blocker a human player's (FUN_00112800): the member waits (cap 1,
///    brain+0x370 counting), and after BLOCKER_WAIT in MOVE_OUT it's GUARD with 12d51a1c.
///
/// Outside the prisms nothing but the wait's count reset (brain+0x370 = 0). The marks are cleared
/// at the end: they only steer the flood. The demo reads no prisms, so this runs only with
/// BF_TEST_PRISM=1.
fn blocker(m: &mut Player, mates: &[Mate], g: &mut Graph, dt: f32) {
    let Some(w) = m.router.walker.as_ref().filter(|w| w.walking()) else { return };
    let me = m.position;
    let (current, own, skipped) = (w.current(), m.router.own.point, w.skipped);
    let mut marked: Vec<(u32, bool)> = vec![];
    let mut first: Option<Mate> = None;
    for mate in mates.iter().filter(|x| x.who != m.who() && x.team == m.team) {
        if (me - mate.at).length_squared() >= BLOCKER_NEAR * BLOCKER_NEAR || !mate.slow {
            continue;
        }
        let (Some(op), Some(cur)) = (mate.point, current) else { continue };
        if !(cur == op || own == Some(op) || skipped == Some(op)) {
            continue;
        }
        let to_mate = (mate.at - me).normalize_or_zero();
        let to_cur = (g.nodes[cur as usize].at - me).normalize_or_zero();
        if to_cur.x * to_mate.x + to_cur.z * to_mate.z <= BLOCKER_DOT {
            continue;
        }
        first.get_or_insert(*mate);
        let had = g.nodes[op as usize].state & nav::AVOID != 0;
        g.nodes[op as usize].state |= nav::AVOID;
        marked.push((op, had));
    }
    let Some(b) = first else { return };
    let log = log_level() > 0;
    let (who, now) = (unit_name(m), m.sim_time);
    if !(in_prism() || b.prism) {
        m.router.wait = 0.0;
    } else {
        let w = m.router.walker.as_mut().expect("walking");
        let mut wait = false;
        if Some(marked[0].0) == w.last() {
            // (FUN_00040f70: walking to the last entry it steps back one; the last dropped)
            let n = w.entries.len() as i32;
            if w.index == n - 1 {
                w.index = n - 2;
                w.best = f32::MAX;
                w.skipped = None;
            }
            w.entries.pop();
            w.done = true;
            if log {
                println!("t {now:.2}: {who}: a friend stands on its route's last point {}: the route is done", label(g, Some(marked[0].0)));
            }
        } else {
            let next = w.next();
            let path = own.zip(next).and_then(|(o, nx)| g.flood(o, BLOCKER_FLOOD, |p| p == nx));
            match path {
                Some(path) => {
                    let nx = *path.last().expect("the flood's way ends at the next point");
                    let mut list: Vec<Entry> = path.iter().map(|&p| Entry { at: g.nodes[p as usize].at, point: Some(p) }).collect();
                    if let Some(k) = w.entries.iter().position(|e| e.point == Some(nx)) {
                        list.extend_from_slice(&w.entries[k + 1..]);
                    }
                    if log {
                        println!("t {now:.2}: {who}: a friend in its way at {}: the way round it to {} ({} entries): {}", label(g, b.point), label(g, Some(nx)), list.len(),
                                 list_entries(g, &list));
                    }
                    let corner = w.corner;
                    w.reset(g);
                    let mut nw = Walker::start(g, &Route { entries: list, dropped: None }, true, own);
                    nw.corner = corner;
                    m.router.walker = Some(nw);
                    if m.router.cap >= NO_CAP {
                        m.router.set_pace(3, BLOCKER_RUN);
                    }
                }
                None => wait = b.human,
            }
        }
        if wait {
            let r = &mut m.router;
            r.wait += dt;
            r.set_pace(1, BLOCKER_RUN);
            if log {
                println!("t {now:.2}: {who}: no way round the player in its way at {}: waits ({:.1} s)", label(g, b.point), r.wait);
            }
            if r.wait >= BLOCKER_WAIT && m.command.mode == brain::Mode::MoveOut {
                m.router.drop_route(g);
                m.router.wait = 0.0;
                if log {
                    println!("t {now:.2}: {who}: waited {BLOCKER_WAIT} s in Move To: GUARD, asks 12d51a1c");
                }
                super::orders::order(m, brain::Mode::Guard, None, now);
                chatter::say(m, LINE_BLOCKED);
            }
        }
    }
    for (p, had) in marked {
        if !had {
            g.nodes[p as usize].state &= !nav::AVOID;
        }
    }
}

/// FUN_000426b0 for `m` (from the AI update, 0x1b18b): with a route and the pace level at 2 or
/// more (0x426f7; under 2 the time is 0), the squared distance (3D) from its position to the
/// current target (FUN_000233d0) over the best so far (route+0x18, FLT_MAX at each new target)
/// adds the frame's time to route+0x1c, one under or equal is the best and the time 0 (0x42760-
/// 0x4278f: no margin, #344; `progress`, where a member asking to move that hasn't moved at all is
/// no closer, the demo's). Past STUCK_AFTER (time 0 again; the speed factor char+0x720 at 0.01
/// or under would be put back to 1 and nothing more: the demo's is never under 1, FUN_00048270;
/// char+0x250 bit 0, the own point wanted, set: the demo's always is):
///  - its own point the same as at the last stuck, the count (brain+0x9c4) over 1: put at the
///    route's current entry's point (FUN_000425b0 with it in EAX, 0x42839-0x42857: vtable +0x48
///    with the point's position + 0.25 m, then FUN_0004c5e0; medium: read as a set-position, here
///    its feet on the floor under that). For a bare position (the follow's one-entry route to a
///    leader whose own point is the member's, FUN_00022d70) FUN_000425b0 does nothing (it tests
///    EAX) and the game's member pushes on until its leader moves d3 off; the demo ends the route
///    there (route+0x20), so the follow's state 0 plans again at once (play_follow.rs): the demo's;
///  - else the detour (`detour`, FUN_00042280), and when its search finds nothing, for a brain out
///    of the squad (brain+0x3c4 bit 8 clear: the level's enemies, #336) whose target was seen in
///    the last second, GOAL_ATTACK_GUARD pushed for 6 s (FUN_00042620, play_goals.rs
///    `stuck_guard`: its goal done, the route with it), else the re-route (`reroute`,
///    FUN_00042490).
fn stuck(m: &mut Player, g: &mut Graph, dt: f32) {
    let me = m.position;
    let (cap, pushing) = (m.router.cap, m.router.asked >= 2);
    let Some(w) = m.router.walker.as_mut().filter(|w| w.walking()) else { return };
    if cap < 2 {
        w.stuck = 0.0;
        return;
    }
    let Some(target) = w.target_now(g) else { return };
    let d2 = target.distance_squared(me);
    if progress(d2, w.best, pushing) {
        w.best = d2;
        w.stuck = 0.0;
    } else {
        w.stuck += dt;
    }
    if w.stuck <= STUCK_AFTER {
        return;
    }
    w.stuck = 0.0;
    let short_of = w.current();
    let log = log_level() > 0;
    let who = unit_name(m);
    let r = &mut m.router;
    let own = r.own.point;
    r.stuck_count = if r.stuck_at == own { r.stuck_count + 1 } else { 0 };
    r.stuck_at = own;
    if log {
        println!("t {:.2}: {who}: stuck {STUCK_AFTER} s short of {} at ({:.1}, {:.1}), its own point {} ({} in a row there)", m.sim_time, label(g, short_of), me.x, me.z,
                 label(g, own), r.stuck_count + 1);
    }
    if r.stuck_count > 1 {
        match short_of {
            Some(p) => {
                let at = g.nodes[p as usize].at + Vec3::Y * WARP_LIFT;
                let feet = floor_y(at.x, at.z, at.y + STEP_UP, world::Mover::Ai.body());
                m.position = Vec3::new(at.x, feet - GROUND, at.z);
                m.prev_xz = at.xz();
                m.fall_from = feet;
                m.vy = 0.0;
                m.height = 0.0;
                m.router.last = Some(m.position);
                if log {
                    println!("t {:.2}: {}: put at its route's current point {} (FUN_000425b0)", m.sim_time, unit_name(m), label(g, short_of));
                }
            }
            None => {
                // (a bare position: no point to be put at; the demo ends the route, see above)
                if let Some(w) = r.walker.as_mut() {
                    w.done = true;
                }
                if log {
                    println!("t {:.2}: {who}: no point to be put at short of its route's end: the route is done (the demo's)", m.sim_time);
                }
            }
        }
        return;
    }
    if detour(m, g) {
        return;
    }
    if goals::stuck_guard(m) {
        m.router.drop_route(g);
        return;
    }
    reroute(m, g);
}

/// FUN_00042280: the detour. With the current entry and the route's last point (here its last
/// entry: the route's destination, `Router::dest`, is planned to again):
///  - the current entry the last: the route done (route+0x20);
///  - else, the current point valid: it loses +0x28 bit 2 (`VALID`) for one search, the own point
///    too while it has it; the next entry the last: done; the next point valid: it loses it too,
///    and a new route to the destination is planned (FUN_00041660); found, it's started with the
///    pace level at 1 for 1 s and route+0x1c at -20;
///  - the bits given back (FUN_00042240).
///
/// A route done here ends at the follow's next step: its state 0 holds there with the leader
/// within d4, else plans again to his point (FUN_00022130, play_follow.rs). False only when the
/// search found nothing.
fn detour(m: &mut Player, g: &mut Graph) -> bool {
    let log = log_level() > 0;
    let (who, now) = (unit_name(m), m.sim_time);
    let Some(w) = m.router.walker.as_mut() else { return true };
    let n = w.entries.len() as i32;
    let Some(cur) = usize::try_from(w.index).ok().filter(|&i| i < w.entries.len()) else { return true };
    if cur as i32 == n - 1 {
        w.done = true;
        if log {
            println!("t {now:.2}: {who}: the detour: stuck on its route's last entry: the route is done");
        }
        return true;
    }
    let Some(cp) = w.entries[cur].point.filter(|&p| g.valid(p)) else { return true };
    let mut off = vec![cp];
    g.nodes[cp as usize].state &= !nav::VALID;
    if let Some(o) = m.router.own.point.filter(|&o| g.valid(o)) {
        g.nodes[o as usize].state &= !nav::VALID;
        off.push(o);
    }
    let mut found = true;
    if cur as i32 + 1 == n - 1 {
        w.done = true;
        if log {
            println!("t {now:.2}: {who}: the detour: its next entry is the last: the route is done");
        }
    } else if let Some(nx) = w.entries[cur + 1].point.filter(|&p| g.valid(p)) {
        g.nodes[nx as usize].state &= !nav::VALID;
        off.push(nx);
        if log {
            println!("t {now:.2}: {who}: the detour: {} invalid for one search", off.iter().map(|&p| label(g, Some(p))).collect::<Vec<_>>().join(", "));
        }
        let dest = m.router.dest;
        found = dest.is_some_and(|d| plan(m, g, d, "the detour").is_ok());
        if found {
            m.router.set_pace(1, DETOUR_HOLD);
            if let Some(w) = m.router.walker.as_mut() {
                w.stuck = DETOUR_GRACE;
            }
        }
    }
    for p in off {
        g.nodes[p as usize].state |= nav::VALID;
    }
    found
}

/// FUN_00042490: the re-route, a new route to the route's destination (the last entry's point,
/// 0x42497; here `Router::dest`), started with the pace level at 1 for 1.5 s and route+0x1c at -5.
fn reroute(m: &mut Player, g: &mut Graph) -> bool {
    let Some(dest) = m.router.dest else { return false };
    if plan(m, g, dest, "the re-route").is_err() {
        return false;
    }
    m.router.set_pace(1, REROUTE_HOLD);
    if let Some(w) = m.router.walker.as_mut() {
        w.stuck = REROUTE_GRACE;
    }
    true
}

/// One BF_TEST_WARP entry: when (s), who (`testsquad::slot_named`: me, a member's name or slot),
/// where (x, z) and the heading (degrees), and whether it's been done.
#[derive(Clone, Debug)]
struct Warp {
    at: f32,
    who: String,
    x: f32,
    z: f32,
    heading: Option<f32>,
    done: bool,
}

/// BF_TEST_WARP=<s>,<who>,<x>,<z>[,<heading>][;...]: at time s (the controlled character's clock)
/// `who` (me, the controlled character, or a member by name or slot, as BF_TEST_ORDER names one)
/// is put on the floor at (x, z) (the first under 50 m above where it stands), facing the heading
/// (degrees; 0 faces -z, 90 -x, as the camera's yaw) or as it faced: the takes' trainer `tp`
/// (todo/242-ai-routes take01: Tex put on the north rim with the squad on the south one). Each
/// entry once; a bad one is logged and skipped.
fn warp_hook(mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut warps: Local<Option<Vec<Warp>>>) {
    let warps = warps.get_or_insert_with(|| {
        let Ok(v) = std::env::var("BF_TEST_WARP") else { return vec![] };
        v.split(';').map(str::trim).filter(|e| !e.is_empty()).filter_map(|e| {
            let parts: Vec<&str> = e.split(',').map(str::trim).collect();
            let num = |i: usize| parts.get(i).and_then(|x| x.parse::<f32>().ok());
            match (num(0), parts.get(1), num(2), num(3)) {
                (Some(at), Some(who), Some(x), Some(z)) => Some(Warp { at, who: who.to_string(), x, z, heading: num(4), done: false }),
                _ => {
                    println!("BF_TEST_WARP: \"{e}\" isn't <s>,<who>,<x>,<z>[,<heading>]: skipped");
                    None
                }
            }
        }).collect()
    });
    let now = player.sim_time;
    let me = player.character;
    for w in warps.iter_mut().filter(|w| !w.done && now >= w.at) {
        w.done = true;
        let Some(c) = testsquad::slot_named(&w.who, me) else {
            println!("BF_TEST_WARP: no squad member \"{}\": skipped", w.who);
            continue;
        };
        let player = &mut *player;
        let Some(p) = std::iter::once(player).chain(squad.0.iter_mut()).find(|u| u.in_squad() && u.character == c) else { continue };
        if p.dead {
            continue;
        }
        let feet = floor_y(w.x, w.z, p.position.y + 50.0, world::Mover::of(p.controlled).body());
        p.position = Vec3::new(w.x, feet - GROUND, w.z);
        p.prev_xz = Vec2::new(w.x, w.z);
        p.fall_from = feet;
        p.vy = 0.0;
        p.height = 0.0;
        if let Some(h) = w.heading {
            p.yaw = h.to_radians();
            p.cam_yaw = p.yaw;
        }
        if log_level() > 0 {
            println!("t {now:.2}: BF_TEST_WARP: {} put at ({:.1}, {feet:.1}, {:.1}) facing {:.0} deg", unit_name(p), w.x, w.z, p.yaw.to_degrees());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain world point named `name` at (x, 0, z) linking to `links` (by name), valid.
    fn point(name: u32, x: f32, z: f32, links: &[u32]) -> WorldPoint {
        WorldPoint { name, kind: 0xF95A_FB93, at: Vec3::new(x, 0.0, z), flags: 0xffff_4000, radius: -1.0, links: links.to_vec(), state: nav::VALID, cover: None }
    }

    /// The names of a route's points.
    fn names(g: &Graph, w: &Walker) -> Vec<u32> {
        w.entries.iter().filter_map(|e| e.point).map(|p| g.nodes[p as usize].name).collect()
    }

    /// The line 1 - 2 - 3 - 4 - 7 with a way round 2 - 5 - 7.
    fn line() -> Graph {
        Graph::new(&[point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3, 5]), point(3, 20.0, 0.0, &[2, 4]), point(4, 30.0, 0.0, &[3, 7]), point(5, 20.0, 10.0, &[2, 7]),
                     point(7, 40.0, 0.0, &[4, 5])])
    }

    /// A member past 2 (its own point) on its way to 3, at the dash, running `frames` stuck tests
    /// of 1/15 s with its x from `x(k)`.
    fn held(m: &mut Player, g: &mut Graph, frames: usize, x: impl Fn(usize) -> f32) {
        for k in 0..frames {
            m.position.x = x(k);
            stuck(m, g, 1.0 / 15.0);
        }
    }

    /// On the line, a member past 2 (its own point) on its way to 3 that comes no closer than its
    /// best for 2 s gets the detour (FUN_00042280): 3, its own point 2 and the next, 4, off for one
    /// search, a new route to the destination 7 round them, the pace level at 1, the bits given
    /// back; the third stuck in a row at one own point puts it on the route's current point
    /// (FUN_000425b0).
    #[test]
    fn the_stuck_detour_and_the_warp() {
        let mut g = line();
        let mut m = Player::new(2);
        m.position = Vec3::new(12.0, 0.0, 0.0);
        m.router.own.point = Some(1);
        m.router.cap = NO_CAP;
        plan(&mut m, &mut g, Dest::Point(5), "test").unwrap();
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7]);
        // standing where it is it isn't stuck: an equal distance is the best again (no margin)
        held(&mut m, &mut g, 40, |_| 12.0);
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7]);
        // a centimetre back and held there, over its best for over 2 s: the detour
        held(&mut m, &mut g, 32, |_| 11.99);
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [5, 7], "round 3 and 4 by 5");
        assert!((0..g.nodes.len() as u32).all(|i| g.valid(i)), "the bits given back");
        assert_eq!((m.router.cap, m.router.stuck_count), (1, 0));
        // held by the level the time stays 0
        held(&mut m, &mut g, 40, |_| 11.98);
        assert_eq!(m.router.stuck_count, 0);
        // two more stucks at the same own point, the level back up: the second detours again, the
        // third puts it on its route's current point
        for k in 1..=2 {
            plan(&mut m, &mut g, Dest::Point(5), "test").unwrap();
            m.router.cap = NO_CAP;
            m.position.x = 12.0;
            stuck(&mut m, &mut g, 1.0 / 15.0);
            for _ in 0..40 {
                m.position.x = 11.99;
                stuck(&mut m, &mut g, 1.0 / 15.0);
                if m.router.stuck_count == k {
                    break;
                }
            }
            assert_eq!(m.router.stuck_count, k);
        }
        assert_eq!(m.position.xz(), Vec2::new(20.0, 0.0), "put at its route's current point 3, not its own point 2");
    }

    /// The stuck test has no margin (FUN_000426b0, 0x42760-0x4278f): a member that jitters by a
    /// centimetre and comes back to its best now and then is never stuck; one that drifts away
    /// a millimetre a frame is, after 2 s.
    #[test]
    fn the_stuck_test_has_no_margin() {
        let start = |g: &mut Graph| {
            let mut m = Player::new(2);
            m.router.own.point = Some(1);
            m.router.cap = NO_CAP;
            m.position = Vec3::new(12.0, 0.0, 0.0);
            plan(&mut m, g, Dest::Point(5), "test").unwrap();
            m
        };
        let mut g = line();
        let mut m = start(&mut g);
        held(&mut m, &mut g, 60, |k| 12.0 - if k % 3 == 0 { 0.0 } else { 0.01 });
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7], "jitter back to its best: never stuck");
        let mut m = start(&mut g);
        held(&mut m, &mut g, 32, |k| 12.0 - k as f32 * 0.001);
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [5, 7], "drifting away: the detour");
        // asking to move and held exactly where it is (the demo's ledge stop): no closer, the
        // detour; standing by its goal's choice there, the game's test, never stuck
        let mut m = start(&mut g);
        m.router.asked = 4;
        held(&mut m, &mut g, 32, |_| 12.0);
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [5, 7], "pinned to the bit: the detour");
        let mut m = start(&mut g);
        held(&mut m, &mut g, 32, |_| 12.0);
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7], "standing by choice: never stuck");
    }

    /// A detour stuck on the route's last entry ends the route (route+0x20, FUN_00042280): the
    /// follow's state 0 then holds there with its leader within d4, or plans again (play_follow.rs).
    #[test]
    fn a_detour_on_the_last_entry_ends_the_route() {
        let mut g = line();
        let mut m = Player::new(2);
        m.router.own.point = Some(3);
        m.router.cap = NO_CAP;
        m.position = Vec3::new(31.0, 0.0, 0.0);
        // from 4's side to 7: its own point dropped, the route 7 alone
        plan(&mut m, &mut g, Dest::Point(5), "test").unwrap();
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [7]);
        held(&mut m, &mut g, 1, |_| 31.0);
        assert!(m.router.walker.as_ref().is_some_and(|w| !w.done));
        held(&mut m, &mut g, 32, |_| 30.99);
        assert!(m.router.walker.as_ref().is_some_and(|w| w.done));
    }

    /// A route to a bare position (the follow's one-entry route to a leader whose own point is the
    /// member's, FUN_00022d70) has no point to be put at on the third stuck: the demo ends the route
    /// (FUN_000425b0 does nothing there), as the detour on its last entry ends it the first two times.
    #[test]
    fn a_stuck_bare_position_ends_the_route() {
        let mut g = line();
        let mut m = Player::new(2);
        m.router.own.point = Some(3);
        m.router.cap = NO_CAP;
        m.position = Vec3::new(30.5, 0.0, 0.0);
        // to a character standing on its own point (4): the one-entry route to his position
        plan(&mut m, &mut g, Dest::Character { at: Vec3::new(31.0, 0.0, 3.0), point: Some(3) }, "test").unwrap();
        let bare = |m: &Player| m.router.walker.as_ref().is_some_and(|w| w.current().is_none());
        assert!(bare(&m), "{:?}", m.router.walker.as_ref().map(|w| &w.entries));
        for k in 0..3 {
            if let Some(w) = m.router.walker.as_mut() {
                w.done = false;
            }
            m.position.x = 30.5;
            stuck(&mut m, &mut g, 1.0 / 15.0);
            held(&mut m, &mut g, 32, |_| 30.4);
            assert_eq!(m.router.stuck_count, k);
            assert!(m.router.walker.as_ref().is_some_and(|w| w.done));
        }
    }

    /// The pace ramp (FUN_00058670): set down to 1 for 1 s (a detour's drop), a member asking for
    /// the dash stands until the timer runs out, then climbs a notch every 0.17 s in FOLLOW (level
    /// 2 at once, 3 and 4 a step apart) and every 1.0 s on a Move To; it doesn't climb past what
    /// it asks. It moves at the lower of the level and what it asks.
    #[test]
    fn the_pace_ramp() {
        let dt = 1.0 / 60.0;
        let level_at = |mode: brain::Mode, asked: i32, t: f32| {
            let mut m = Player::new(2);
            m.command.mode = mode;
            m.router.asked = asked;
            m.router.set_pace(1, DETOUR_HOLD);
            for _ in 0..(t / dt).round() as i32 {
                pace_ramp(&mut m, dt);
            }
            m.router.cap
        };
        assert_eq!(level_at(brain::Mode::Follow, 4, 0.98), 1, "the drop's timer still running");
        assert_eq!(level_at(brain::Mode::Follow, 4, 1.03), 2);
        assert_eq!(level_at(brain::Mode::Follow, 4, 1.15), 2);
        assert_eq!(level_at(brain::Mode::Follow, 4, 1.22), 3);
        assert_eq!(level_at(brain::Mode::Follow, 4, 1.45), 4);
        assert_eq!(level_at(brain::Mode::Follow, 3, 3.0), 3, "no higher than it asks");
        assert_eq!(level_at(brain::Mode::MoveOut, 4, 1.95), 2);
        assert_eq!(level_at(brain::Mode::MoveOut, 4, 2.08), 3);
        assert_eq!(level_at(brain::Mode::MoveOut, 4, 3.12), 4);
        let mut m = Player::new(2);
        m.router.set_pace(2, DETOUR_HOLD);
        assert!(pace(&mut m, 4) && m.walk && !m.sprint);
        m.router.set_pace(1, DETOUR_HOLD);
        m.move_input = Vec2::Y;
        assert!(!pace(&mut m, 4) && m.move_input == Vec2::ZERO && m.router.asked == 4);
    }

    /// The pace record starts at a stand with its timer at 0.3 s (FUN_00058530), and follows the
    /// move asked down as well as up: from the dash a stand asked steps 4 -> 3 -> 2 a step apart,
    /// then 2 -> 1 with the timer at 0.3 s (FUN_00058b10, FUN_00058aa0, FUN_00058a20); asked to
    /// dash again, it climbs from 1 once that has run out.
    #[test]
    fn the_pace_follows_the_move_asked_down() {
        let dt = 1.0 / 60.0;
        let mut m = Player::new(2);
        m.command.mode = brain::Mode::Follow;
        assert_eq!((m.router.cap, m.router.cap_left), (1, PACE_STAND), "the brain's init");
        let run = |m: &mut Player, t: f32| {
            for _ in 0..(t / dt).round() as i32 {
                pace_ramp(m, dt);
            }
            m.router.cap
        };
        m.router.asked = 4;
        assert_eq!(run(&mut m, 0.25), 1, "the init's 0.3 s first");
        assert_eq!(run(&mut m, 0.07), 2);
        assert_eq!(run(&mut m, 0.35), 4, "0.34 s from a walk to the dash");
        // (its timer runs out at the dash and stays so)
        assert_eq!(run(&mut m, 0.5), 4);
        m.router.asked = 1;
        assert_eq!(run(&mut m, 0.02), 3, "the timer had run out: a notch down at once");
        // (3 -> 2 a step later; then 2 -> 1, its timer at 0.3 s, a few frames of which are gone)
        assert_eq!(run(&mut m, 0.2), 2);
        assert_eq!(run(&mut m, 0.2), 1);
        assert!(m.router.cap_left > 0.2 && m.router.cap_left < PACE_STAND, "the step down to a stand sets 0.3 s: {}", m.router.cap_left);
        m.router.asked = 4;
        assert_eq!(run(&mut m, 0.2), 1, "its 0.3 s still running");
        assert_eq!(run(&mut m, 0.1), 2);
    }

    /// A test map NPC set to follow (#364) walks routes and has the squad's pace record as a member
    /// of FOLLOW does (0.17 s a notch, the walk at 2.7 m and the run at 4 m of a route's end); one
    /// that doesn't follow, an enemy on the map, and a member of another mode keep what they had.
    #[test]
    fn a_follower_npc_has_the_squads_pace_record() {
        let npc = |id: u32, follow: bool| {
            let mut m = Player::new(1);
            m.npc = Some(testtools::Npc { id, fight: false, face: 0.0, follow });
            m
        };
        let (a, b) = (npc(1, true), npc(2, false));
        assert!(walks_routes(&a) && squad_pace(&a) && pace_distances(&a) == (PACE_WALK_SQUAD, PACE_RUN_SQUAD));
        assert!(!walks_routes(&b) && !squad_pace(&b) && pace_distances(&b) == (PACE_WALK, PACE_RUN));
        // (an NPC set to follow in a mode the squad's pace isn't: Guard keeps the others')
        let mut guard = npc(3, true);
        guard.command.mode = brain::Mode::Guard;
        assert!(!squad_pace(&guard));
        // (a squad member's, as ever)
        assert!(walks_routes(&Player::new(1)) && squad_pace(&Player::new(1)));
        // (the climb: a notch every 0.17 s, as the ramp's test above has the squad's)
        let mut m = npc(1, true);
        m.router.asked = 4;
        m.router.cap_left = 0.0;
        pace_ramp(&mut m, 1.0 / 60.0);
        assert_eq!((m.router.cap, m.router.cap_left), (2, PACE_STEP_SQUAD));
    }

    /// The door step (FUN_00026820): reaching a door point whose next entry is a door point too,
    /// with the corridor between them blocked, the pace level 1 for 0.5 s and the feelers off for
    /// 1.5 s; clear, nothing; a door point followed by a plain one, nothing.
    #[test]
    fn the_door_step() {
        // 1 (a door point, `<door-point>` h_1ff1d816) -> 2 (one) -> 3 (plain), along x
        let door = |name: u32, x: f32, links: &[u32]| WorldPoint { kind: 0x1FF1_D816, ..point(name, x, 0.0, links) };
        let pts = [door(1, 0.0, &[2]), door(2, 6.0, &[1, 3]), point(3, 12.0, 0.0, &[2])];
        let g = Graph::new(&pts);
        assert_eq!((g.nodes[0].kind, g.nodes[2].kind), (nav::kind::DOOR, nav::kind::WORLD), "the door point's type");
        let walking = |index: i32| {
            let mut m = Player::new(2);
            m.router.cap = NO_CAP;
            let entries = (0..3).map(|i| Entry { at: g.nodes[i].at, point: Some(i as u32) }).collect();
            m.router.walker = Some(Walker { entries, index, best: f32::MAX, ..Default::default() });
            m
        };
        let reached = |p: u32| Step { walking: true, reached: Some((Some(p), g.nodes[p as usize].kind)), ..Default::default() };
        // a shut door across x = 3: the rays between the two door points meet it
        let shut = |a: Vec3, b: Vec3, _: f32| (a.x - 3.0).signum() == (b.x - 3.0).signum();
        let open = |_: Vec3, _: Vec3, _: f32| true;
        let mut m = walking(1);
        door_step(&mut m, &g, &reached(0), &shut);
        assert_eq!((m.router.cap, m.router.cap_left, m.router.avoid.feelers_off), (1, DOOR_STAND, DOOR_FEELERS_OFF), "blocked: a stand, the feelers off");
        let mut m = walking(1);
        door_step(&mut m, &g, &reached(0), &open);
        assert_eq!((m.router.cap, m.router.avoid.feelers_off), (NO_CAP, 0.0), "clear: on through");
        let mut m = walking(2);
        door_step(&mut m, &g, &reached(1), &shut);
        assert_eq!((m.router.cap, m.router.avoid.feelers_off), (NO_CAP, 0.0), "the next is a plain point");
    }

    /// FUN_00058590 by the distance across to the way's end: within the walk distance (or at it) a
    /// walk asked and the level at most 2, within the run distance a run and at most 3, farther
    /// nothing; a walk asked isn't raised to a run, nor a stand to a walk.
    #[test]
    fn the_pace_near_the_end() {
        let at = |asked: i32, cap: i32, off: f32| {
            let mut r = Router { asked, cap, ..Default::default() };
            let did = end_pace(&mut r, off, PACE_WALK, PACE_RUN).is_some();
            (did, r.asked, r.cap)
        };
        assert_eq!(at(4, 4, 7.0), (false, 4, 4), "beyond the run distance");
        assert_eq!(at(4, 4, PACE_RUN), (true, 3, 3), "at the run distance");
        assert_eq!(at(4, 2, 5.0), (true, 3, 2), "the level lower already");
        assert_eq!(at(4, 4, PACE_WALK), (true, 2, 2), "at the walk distance");
        assert_eq!(at(2, 2, 5.0), (false, 2, 2), "a walk asked isn't raised");
        assert_eq!(at(1, 1, 1.0), (false, 1, 1), "nor a stand");
    }

    /// FUN_000587d0 within the walk distance of the current point (3D): a corner sharper than the
    /// cosine 0.1 a walk, under 0.365 a notch under the move asked, a gentler one a walk only while
    /// the move is off the way to the point within 2.4 m; each with the timer at 1.5 s; nothing
    /// farther out or at a stand's level.
    #[test]
    fn the_pace_at_a_corner() {
        // the way to the point (0, 0, 0) is +z from (0, 0, -2); the next leg at `cos` to it
        let leg = |cos: f32| Vec3::new((1.0 - cos * cos).sqrt(), 0.0, cos) * 5.0;
        let at = |cap: i32, asked: i32, me: Vec3, cos: f32, moving: Option<Vec2>| {
            let mut r = Router { asked, cap, cap_left: 0.0, ..Default::default() };
            let did = corner_pace(&mut r, me, moving, Vec3::ZERO, leg(cos), PACE_WALK);
            (did, r.cap, r.cap_left)
        };
        let me = Vec3::new(0.0, 0.0, -2.0);
        let along = Some(Vec2::Y);
        assert_eq!(at(4, 4, me, 0.0, along), (true, 2, CORNER_HOLD), "a square corner: a walk");
        assert_eq!(at(4, 4, me, 0.2, along), (true, 3, CORNER_HOLD), "a bent one: a notch under the dash");
        assert_eq!(at(4, 3, me, 0.2, along), (true, 2, CORNER_HOLD), "a notch under a run");
        assert_eq!(at(3, 2, me, 0.2, along), (true, 2, CORNER_HOLD), "a walk asked: a walk");
        assert_eq!(at(3, 2, me, 0.0, along), (true, 3, CORNER_HOLD), "a square corner, a walk asked: the timer alone");
        assert_eq!(at(4, 4, me, 0.7, along), (false, 4, 0.0), "a gentle one, moving its way");
        assert_eq!(at(4, 4, me, 0.7, Some(Vec2::X)), (true, 2, CORNER_HOLD), "a gentle one, moving off its way");
        assert_eq!(at(4, 4, Vec3::new(0.0, 0.0, -2.5), 0.7, Some(Vec2::X)), (false, 4, 0.0), "off its way beyond 2.4 m");
        assert_eq!(at(4, 4, me, 0.7, None), (false, 4, 0.0), "standing");
        assert_eq!(at(4, 4, Vec3::new(0.0, 0.0, -3.3), 0.0, along), (false, 4, 0.0), "beyond the walk distance");
        assert_eq!(at(4, 4, Vec3::new(0.0, 2.5, -2.0), 0.0, along), (false, 4, 0.0), "3D: 3.2 m off with the height");
        assert_eq!(at(1, 4, me, 0.0, along), (false, 1, 0.0), "at a stand's level");
    }

    /// FUN_0004d840 on a route A (0, 0) -> B (10, 0) -> C (10, 10), a square corner at B: 2 m short
    /// of B at the dash it walks (the corner); 3.5 m short of C it runs at most (the end, before
    /// any corner); a Move To's distances (3.2 / 6.0) against the follow's (2.7 / 4.0).
    #[test]
    fn the_pace_near_a_routes_points() {
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3]), point(3, 10.0, 10.0, &[2])];
        let g = Graph::new(&pts);
        let walking = |index: i32, me: Vec3, mode: brain::Mode| {
            let mut m = Player::new(2);
            m.command.mode = mode;
            (m.router.cap, m.router.asked) = (NO_CAP, NO_CAP);
            m.position = me;
            m.last_velocity = Vec3::new(6.0, 0.0, 0.0);
            let entries = (0..3).map(|i| Entry { at: g.nodes[i].at, point: Some(i as u32) }).collect();
            m.router.walker = Some(Walker { entries, index, best: f32::MAX, ..Default::default() });
            m
        };
        let mut m = walking(1, Vec3::new(8.0, 0.0, 0.0), brain::Mode::MoveOut);
        pace_near(&mut m, &g);
        assert_eq!((m.router.asked, m.router.cap, m.router.cap_left), (4, 2, CORNER_HOLD), "the square corner at B: a walk, the dash still asked");
        let mut m = walking(2, Vec3::new(10.0, 0.0, 6.5), brain::Mode::MoveOut);
        pace_near(&mut m, &g);
        assert_eq!((m.router.asked, m.router.cap), (3, 3), "3.5 m short of C: a run");
        let mut m = walking(2, Vec3::new(10.0, 0.0, 5.5), brain::Mode::Follow);
        pace_near(&mut m, &g);
        assert_eq!((m.router.asked, m.router.cap), (NO_CAP, NO_CAP), "the follow's distances: 4.5 m is beyond its 4.0");
        let mut m = walking(2, Vec3::new(10.0, 0.0, 5.5), brain::Mode::MoveOut);
        pace_near(&mut m, &g);
        assert_eq!((m.router.asked, m.router.cap), (3, 3), "a Move To's: within its 6.0");
    }
}
