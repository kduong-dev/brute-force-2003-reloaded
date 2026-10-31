//! The squad's routes (#335, the second slice of #242; viewer/docs/nav.md "The squad on
//! routes"): the AI squad's follow and its Move To walk routes over the level's world points
//! (bf/nav.rs: `Graph::plan`, FUN_000416a0; `Walker`, FUN_00025730 / FUN_00026d80), and the AI
//! update's three steps on the brain's route that come before its goal (FUN_0001afe0,
//! 0x1b17f-0x1b18b): the route round a friend in the way (FUN_00041870), the catch-up to the own
//! point (FUN_00041ff0) and the 2 s stuck handling (FUN_000426b0). Every function cited is in
//! viewer/docs/code-map.md ("AI"); the reading is todo/242-ai-routes/leads.md §3-§4 with the
//! corrections nav.md lists.
//!
//!  - **One route record.** A brain walks one route, brain+0x930: the goal holder's +0x8f0 (the
//!    holder is brain+0x40) that MOVE_OUT's update FUN_00045fa0, PATROL's FUN_0002b490 and
//!    RETURN_HOME's FUN_00034690 hand FUN_00026d80, and the follow's own (FUN_00022130). So the
//!    stuck handling and the blocker route, which the AI update runs on brain+0x930 whatever the
//!    goal, apply to Move To as to the follow (`Router::walker`).
//!  - **The own points** (FUN_0014f860) of the controlled character (the character update keeps
//!    its point while its brain idles under the player: a route to a character ends there) and
//!    of the AI squad and its followers, every frame (`keep_point`).
//!  - **The follow** (`follow`, from play.rs `squad_ai` in FOLLOW and WINGMAN): the demo's
//!    formation place, its distances kept until #331 brings the game's follow states, reached by
//!    a route and never a straight chase, planned again as the follow's state 0 plans (see
//!    `follow`). Without a level (the test map's floor has no world points) it stays the straight
//!    walk it was.
//!  - **Move To** (`move_out`, from play_orders.rs): GOAL_MOVE_OUT's route to the cursor's point
//!    (0x45d00), walked at 2 m to its end (FUN_00045fa0).
//!  - **The liquid guard** (play_liquids.rs `burning_ahead`): a safety net only. The game keeps
//!    out of lava by the graph's layout (no point lies over a harmful liquid, nav.md "Lava"), and
//!    the corridor rays pass through liquids; a member whose next step would still go into one
//!    holds there (`shore`).
//!
//! Test hooks: BF_ROUTE_LOG=1 (each plan and why, each point reached, corner held, skip,
//! catch-up, stuck, detour, re-route, warp and blocker step of the AI squad's routes; =2 also
//! their own points' changes), BF_TEST_WARP=<s>,<who>,<x>,<z>[,<heading>][;...] (the controlled
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

/// FUN_000225d0's flood for a point near the leader: 12 m (FUN_00020170(.., 0, 12.0) puts 12 at
/// the filter's +0x110, the radius FUN_000594a0 floods within). The demo's follow takes its
/// place's point among the points the leader's own point floods to within it (`follow`).
const NEAR_LEADER: f32 = 12.0;
/// FUN_00020b10's d3 (its +0x14): 7 m in WINGMAN, 9.8 m (1.4 x 7) in FOLLOW. The follow's state 0
/// plans again while the leader is farther than it from the route's end (FUN_00022130, 0x22251);
/// play.rs `squad_ai` scales it by `orders::spacing` (8 / 11.2 in WINGMAN: 7.0).
pub const REPLAN: f32 = 9.8;
/// FUN_000426b0: the time (s) without the squared distance to the current target falling
/// (route+0x1c over 2.0, 0x4267c) that counts as stuck.
const STUCK_AFTER: f32 = 2.0;
/// How much closer (m) to its target a character asking to move must come for the stuck time to
/// start again: a guess, and a stand-in until #344. The game's test has no margin (0x42760-
/// 0x4278f: a squared distance over the best adds the time, one under or equal is the new best
/// and the time 0), so its jitter would start the time again too, but its members don't get
/// pinned: the feelers (FUN_000255d0 / FUN_00018c40, #344) keep them about a metre off a hinter
/// band. The demo's, without them, press against it, and its collision jitters one held there by
/// about a centimetre, a new best every few frames (#335's review: Hawk held 3.69 m from her
/// target, the time started again after 0.07-0.47 s, so the 2 s never came); 5 cm is five times
/// that, and a walk at 2 m/s gains it in a frame at 15 fps. To go with #344's feelers.
const STUCK_PROGRESS: f32 = 0.05;
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
/// FUN_00041870's route round a friend at most 3 for 0.5 and its wait 1 for 0.5, the follow's
/// hold (state 2, FUN_000223c0: 0x3e99999a) 1 for 0.3. The ramp climbs back once the timer runs
/// out (`ai_update`).
const DETOUR_HOLD: f32 = 1.0;
const REROUTE_HOLD: f32 = 1.5;
const BLOCKER_RUN: f32 = 0.5;
const FOLLOW_HOLD: f32 = 0.3;
/// The pace level's top, the dash (FUN_00058b10's level 4). The demo's level starts there and
/// leaves it only by the drops above, then climbs back (`ai_update`); the game's starts at 1 (the
/// brain's init, 0x1706f) and follows the move asked down as well as up (FUN_00058a20: 2 to 1
/// with 0.3 s; FUN_00058aa0: 3 to 2; FUN_00058b10: 4 to 3), so its every start from a stand climbs
/// too: that start-up ramp is #344's (todo/344-ai-avoidance leads §6).
pub const NO_CAP: i32 = 4;
/// The pace ramp's step (s, the record's +0x20, FUN_00058670 each AI frame): 0.17 (0x3e2e147b)
/// for a squad brain (brain+0x3c4 bit 8) in FOLLOW or WINGMAN (brain+0x58 1 or 4; or with +0x770
/// set, not read), 1.0 (0x3f800000) for any other. Run out, its timer lets the level climb one
/// notch toward the move asked (FUN_000589b0 to 2, FUN_00058a60 to 3, FUN_00058aa0 to 4) and
/// starts again at the step: a member of the follow climbs from 1 to the dash in 0.51 s, one on
/// a Move To in 3 s.
const PACE_STEP_SQUAD: f32 = 0.17;
const PACE_STEP: f32 = 1.0;
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
/// The demo's follow ends its route at the place itself (`Dest::Beside`) only when the corridor
/// from the place's point to it is clear (FUN_000410a0's 0.8 m, as the walk's skips test) and the
/// place is on a floor no more than this under or over the leader's feet: the demo's, so a place
/// on a ledge below, in a pit or up a cliff isn't walked to (one up a slope is: the floor is
/// looked for from this over his feet).
const PLACE_DROP: f32 = 2.0;
/// The demo's follow looks for a clear corridor to its place from this many of the points
/// nearest it (`follow_dest`), so a place that an object hides from its nearest point is still
/// walked to from one beside it: a guess (e34, #335's fix round: Object 464 stood between
/// h_f6632343 and the places beyond it, and the members held at the point).
const PLACE_TRIES: usize = 4;

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
    /// where the follow's leader stood when its route was planned
    planned: Option<Vec3>,
    /// where the follow's leader stood when its hold began (`follow`: a route's end short of its
    /// place, no route, or its place's point the one it stands at); None out of a hold
    hold: Option<Vec3>,
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
}

impl Default for Router {
    /// A spawned character's: its own point wanted, no route, its pace level at the top.
    fn default() -> Self {
        Router { own: OwnPoint::spawned(), own_at: None, walker: None, dest: None, planned: None, hold: None, ordered: None, last: None, stuck_at: None, stuck_count: 0,
                 cap: NO_CAP, cap_left: 0.0, asked: 1, wait: 0.0, stale: false }
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
        self.planned = None;
        self.ordered = None;
    }

    /// The pace level set down to `cap` with its timer at `time` s ([brain+0x1a8]+0 and
    /// FUN_000124f0 on its +0x28): the drops (DETOUR_HOLD and the others).
    fn set_pace(&mut self, cap: i32, time: f32) {
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

/// Whether `u` walks routes in the demo: an AI squad member (`Player::in_squad`, no brain) or a
/// level character's brain that follows the leader (the tutorial's marines, play_brain.rs). The
/// enemies' brains (#336) and the test map's NPCs don't yet.
pub fn walks_routes(u: &Player) -> bool {
    (u.in_squad() && u.brain.is_none()) || u.brain.as_ref().is_some_and(|b| b.follows())
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
/// one and the squad's, alive and in the world. An AI one is slow while the pace it moves at, the
/// lower of the move it asks and its level (`pace`), is a walk or a stand (the game tests the
/// level under 3, which follows the move asked down where the demo's doesn't: NO_CAP); the
/// controlled one while it walks or stands (its brain idles: the demo's guess).
pub fn mates(player: &Player, squad: &[Player]) -> Vec<Mate> {
    let prism = in_prism();
    std::iter::once(player).chain(squad.iter()).filter(|u| !u.dead && !u.absent).map(|u| {
        let slow = if u.controlled { u.move_input.length() < 0.1 || u.walk } else { u.router.asked.min(u.router.cap) < 3 };
        Mate { who: u.who(), team: u.team, at: u.position, point: u.router.own.point, slow, human: u.controlled, prism }
    }).collect()
}

/// The AI update's route steps for `m` before its goal (FUN_0001afe0): the pace ramp (FUN_00058670
/// at 0x1b119: the timer runs, and run out with the move asked above the level, the level climbs
/// a notch and the timer starts again at the step, PACE_STEP_SQUAD or PACE_STEP; the move asked is
/// the last frame's, as the goal writes it after); then at 0x1b17f-0x1b18b, when the character
/// has an own point, a route its goal dropped, the route round a friend in the way (`blocker`,
/// FUN_00041870), the catch-up (FUN_00041ff0, `Walker::resync`) and the stuck handling (`stuck`,
/// FUN_000426b0). `mates`: this frame's friends (`mates`).
pub fn ai_update(m: &mut Player, mates: &[Mate], g: &mut Graph, dt: f32) {
    let step = if m.in_squad() && matches!(m.command.mode, brain::Mode::Follow | brain::Mode::Wingman) { PACE_STEP_SQUAD } else { PACE_STEP };
    let r = &mut m.router;
    r.cap_left = (r.cap_left - dt).max(0.0);
    if r.cap_left <= 0.0 && r.asked > r.cap && r.cap < NO_CAP {
        r.cap += 1;
        r.cap_left = step;
        if log_level() >= 2 {
            println!("t {:.2}: {}: its pace climbs to {} (the next in {step} s)", m.sim_time, unit_name(m), m.router.cap);
        }
    }
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

/// A route walk's step for `m` (FUN_00026d80 at `radius`, at most 1 m in a prism), logged; its
/// position kept for the next step's pass test.
fn step(m: &mut Player, g: &mut Graph, radius: f32, last_radius: f32) -> Step {
    let me = m.position;
    let last = m.router.last.replace(me).unwrap_or(me);
    let who = unit_name(m);
    let now = m.sim_time;
    let Some(w) = m.router.walker.as_mut() else { return Step::default() };
    let on_last = usize::try_from(w.index).is_ok_and(|i| i + 1 == w.entries.len()) && w.entries.last().is_some_and(|e| e.point.is_none());
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
    s
}

/// Plans `m`'s route to `to` (FUN_000416a0 from its own point) and starts it (FUN_00025730 with
/// its third argument 1, as the follow's and MOVE_OUT's starts pass it), logged with `why`. The
/// route found, or why none.
fn plan(m: &mut Player, g: &mut Graph, to: Dest, why: &str) -> Result<(), NoRoute> {
    let me = m.position;
    let own = m.router.own.point;
    let clock = std::time::Instant::now();
    let plan = g.plan(me, own, to, None, &ray);
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
            Ok(())
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

/// How the follow or Move To goes this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Way {
    /// toward `target` (the route's steering target, brain+0x478), `along` m of the walk left,
    /// `corner` while the corner rule holds it
    Go { target: Vec3, along: f32, corner: bool },
    /// standing: no route to its place, or held at a route's end short of it (the pace level at 1
    /// is the caller's, `pace`)
    Stand,
    /// at the route's end, its place reached
    There,
}

/// The points the follow's route to `place` may go to, the nearest `place` (3D) first: the points
/// the leader's own point `leader` floods to within NEAR_LEADER (FUN_000594a0 from the leader's
/// point, as the follow's state 4 finds its point near the leader: FUN_000225d0), so ones on the
/// leader's side of whatever lies between, his point among them (the flood offers it first, and
/// it comes first of equals); without a leader's point the nearest valid point to `place`. The
/// demo's pick for its formation place (state 4 takes the first its filter takes; #331).
fn place_points(g: &Graph, place: Vec3, leader: Option<u32>) -> Vec<u32> {
    let Some(lp) = leader else { return g.nearest_valid(place).into_iter().collect() };
    let mut all = Vec::new();
    g.flood(lp, NEAR_LEADER, |p| {
        all.push((p, g.nodes[p as usize].at.distance_squared(place)));
        false
    });
    all.sort_by(|a, b| a.1.total_cmp(&b.1));
    all.into_iter().map(|(p, _)| p).collect()
}

/// The follow's destination for `place` (feet height `feet`) by the points near it, the nearest
/// first (`place_points`), and the leader's own point `leader`:
///  - the place itself beside the first of the PLACE_TRIES nearest from which the corridor to it
///    is clear (`Dest::Beside`, FUN_00059d40's position beside a point; FUN_000410a0's 0.8 m, from
///    the point to the place as high over its floor as the point stands over its own: the
///    demo's, as the route's corridors run between points at their heights), when it stands on a
///    floor no more than PLACE_DROP under or over `feet` and in no liquid that burns;
///  - a place on such a floor that none of them sees: the nearest point (the route ends there, as
///    near as the graph comes);
///  - a place no one could stand on (no such floor, or in a liquid that burns): the leader's own
///    point, the game's follow's destination (FUN_00022130's route to the leader), the nearest
///    without one. The demo's: the formation's place turns with the leader's heading, and for a
///    leader in the lava it swings round him over the far shores, its nearest point with it,
///    where his own point stays (#335's fix round, e13: with the place's nearest point Hawk went
///    to and fro between two points 21 m apart as his turning swung her place from shore to
///    shore).
///
/// None without a point. BF_ROUTE_LOG=2 says why a place isn't walked to.
fn follow_dest(g: &Graph, place: Vec3, feet: f32, points: &[u32], leader: Option<u32>) -> Option<Dest> {
    let nearest = *points.first()?;
    let body = world::Mover::Ai.body();
    let arena = world::arena();
    let floor = arena.as_deref().and_then(|a| a.floor_below(place.x, place.z, feet + PLACE_DROP, body)).map(|f| f.0);
    let dropped = !floor.is_some_and(|y| feet - y <= PLACE_DROP);
    let burns = arena.as_deref().zip(floor).is_some_and(|(a, y)| liquids::burns_at(a, Vec3::new(place.x, y + 0.1, place.z)));
    let clear = |point: u32, y: f32| {
        let p = g.nodes[point as usize].at;
        let lift = arena.as_deref().and_then(|a| a.floor_below(p.x, p.z, p.y + STEP_UP, body)).map_or(0.0, |f| (p.y - f.0).max(0.0));
        !nav::corridor_blocked(p, Vec3::new(place.x, y + lift, place.z), nav::CORRIDOR, &ray)
    };
    let seen = floor.filter(|_| !dropped && !burns).and_then(|y| points.iter().take(PLACE_TRIES).copied().find(|&p| clear(p, y)).map(|p| (p, y)));
    if seen.is_none() && log_level() >= 2 {
        println!("  (the place ({:.1}, {:.1}) not walked to: {}; its floor {:?}, the leader's feet {feet:.1})", place.x, place.z,
                 if dropped { "no floor or too low" } else if burns { "in a liquid that burns" } else { "the corridor from its nearest points blocked" }, floor);
    }
    Some(match seen {
        Some((point, y)) => Dest::Beside { at: Vec3::new(place.x, y - GROUND, place.z), point },
        None if dropped || burns => Dest::Point(leader.unwrap_or(nearest)),
        None => Dest::Point(nearest),
    })
}

/// The follow's distances at its command mode's spacing (play.rs `orders::spacing`, 1 in FOLLOW
/// and 8 / 11.2 in WINGMAN, as FUN_00020b10's five distances all go with the mode): `replan`, d3
/// (REPLAN), and `again`, the formation's set-off distance (play.rs SLOT_START), which a hold
/// waits for the leader himself to move (`follow`).
#[derive(Clone, Copy, Debug)]
pub struct Reach {
    pub replan: f32,
    pub again: f32,
}

impl Reach {
    /// The two at the formation's `spacing`, `start` its set-off distance unscaled.
    pub fn at(spacing: f32, start: f32) -> Reach {
        Reach { replan: REPLAN * spacing, again: start * spacing }
    }
}

/// The follow's way to its place `place` this frame (play.rs `squad_ai`, once it has set off: the
/// formation's set-off and settle rules are its), by a route over the graph. `leader_at` is the
/// leader's position: the game's follow routes to his own point and measures its re-plans and
/// holds by him, and the demo keys them on him too, not on the place, which turns with his
/// heading 7-12 m out in front (FORMATION), so a leader turning where he stands swings it round
/// him (#335's review: 11-13 plans in 2 s, the destination flipping between points, with the
/// leader running round his goal in lava on e13):
///  - planned to `follow_dest` (the place beside one of the points near it on the leader's side,
///    else the nearest of them, or for a place no one could stand on the leader's own point), from
///    the member's own point;
///  - planned again when the leader has moved more than d3 (3D) from where he stood when the
///    route was planned (FUN_00022130 plans again while the leader is farther than d3 from the
///    route's end, which is his point), and at a route's end beside the place when the place has
///    moved on (state 0 plans again with its route empty; the settle radius `settle`, the
///    formation's SLOT_STOP; the game's state 0 would hold there with the leader within d4, the
///    demo's formation keeps its place);
///  - held at a route's end at a point short of the place (the place out of reach: FUN_00022130
///    holds at a route's end with the leader within d4, else plans again to his point, which for
///    a place out of reach is that point or one beside it, so the demo holds whatever his
///    distance), with no route (FUN_000416a0 fails: state 2, the hold), or with its destination
///    the point it stands at (as near as the graph comes: no search for it); each hold until the
///    leader has moved `again` (3D, the formation's set-off distance) from where he stood when
///    it began: the demo's, a guess (the game's hold, state 3, FUN_00022420, ends when the leader
///    is beyond d0 of the member, which a member held on a shore 11-14 m from a leader in the
///    lava already is). A settled member sets off when its place is that far; a held one when
///    the leader himself has gone that far, his turning left out: the e13 lava leader's track
///    keeps within 2 m of its middle, and a leader running at 4.8 m/s goes 3 m in 0.6 s;
///  - walked at 2 m a point (FUN_00026d80 at 2.0, the follow's), the last entry, the place itself,
///    within `settle`, at the pace the caller asks and the level allows (`pace`: at level 1, after
///    a detour, a re-route or a hold, it stands).
pub fn follow(m: &mut Player, g: &mut Graph, place: Vec3, leader_at: Vec3, leader: Option<u32>, reach: Reach, settle: f32) -> Way {
    let mut why = "the follow";
    let log = log_level() > 0;
    for _ in 0..2 {
        if let Some(l0) = m.router.planned.filter(|_| m.router.walking()) {
            let moved = l0.distance(leader_at);
            if moved > reach.replan {
                if log {
                    println!("t {:.2}: {}: its leader is {moved:.1} m from where he stood when its route was planned (over {:.1}): plans again", m.sim_time,
                             unit_name(m), reach.replan);
                }
                m.router.drop_route(g);
                why = "the follow: its leader moved";
            }
        }
        if !m.router.walking() {
            if let Some(h) = m.router.hold {
                let moved = h.distance(leader_at);
                if moved <= reach.again {
                    return Way::Stand;
                }
                m.router.hold = None;
                if log {
                    println!("t {:.2}: {}: out of its hold: its leader {moved:.1} m from where he stood when it began (over {:.1})", m.sim_time, unit_name(m),
                             reach.again);
                }
                why = "the follow: out of its hold";
            }
            let Some(to) = follow_dest(g, place, place.y + GROUND, &place_points(g, place, leader), leader) else { return Way::Stand };
            if let Dest::Point(p) = to {
                if m.position.xz().distance(g.nodes[p as usize].at.xz()) < nav::ARRIVE {
                    begin_hold(m, g, leader_at, place, "its place's point is the one it stands at");
                    return Way::Stand;
                }
            }
            if plan(m, g, to, why).is_err() {
                begin_hold(m, g, leader_at, place, "no route");
                return Way::Stand;
            }
            m.router.planned = Some(leader_at);
        }
        let s = step(m, g, nav::ARRIVE, settle);
        if s.walking {
            let w = m.router.walker.as_ref().expect("walking");
            return Way::Go { target: s.target, along: w.remaining(g, m.position), corner: w.corner.is_some() };
        }
        // (the route's end: at the place, or at the point short of it)
        let at_place = matches!(m.router.dest, Some(Dest::Beside { .. }));
        m.router.drop_route(g);
        if m.position.xz().distance(place.xz()) <= settle {
            return Way::There;
        }
        if !at_place {
            begin_hold(m, g, leader_at, place, "the route's end short of its place");
            return Way::Stand;
        }
        why = "the follow: at the route's end short of its place";
    }
    Way::Stand
}

/// The follow's hold begins (`follow`), the leader at `leader_at` and its place `place` then: the
/// game's state 2 (FUN_000223c0), which sets the pace level to 1 for FOLLOW_HOLD before its hold,
/// state 3; `why` for the log.
fn begin_hold(m: &mut Player, g: &Graph, leader_at: Vec3, place: Vec3, why: &str) {
    m.router.hold = Some(leader_at);
    m.router.set_pace(1, FOLLOW_HOLD);
    if log_level() > 0 {
        println!("t {:.2}: {}: holds ({why}) at ({:.1}, {:.1}), its own point {}, its place ({:.1}, {:.1})", m.sim_time, unit_name(m), m.position.x, m.position.z,
                 label(g, m.router.own.point), place.x, place.z);
    }
}

/// The follow stops (its set-off rule says stand): the route dropped (state 0's hold within d2
/// resets it, FUN_0004e8f0) and its hold forgotten.
pub fn stop(m: &mut Player, g: &mut Graph) {
    if m.router.walker.is_some() {
        m.router.drop_route(g);
    }
    m.router.hold = None;
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
    Way::Go { target: s.target, along: w.remaining(g, m.position), corner: w.corner.is_some() }
}

/// The liquid guard on `m`'s way to `target` (play_liquids.rs `burning_ahead`): the safety net
/// behind the routes, true when its next step would go into a liquid that burns. BF_LIQUID_LOG
/// says so twice a second.
pub fn shore(m: &Player, target: Vec3, dt: f32) -> bool {
    let d = Vec2::new(target.x - m.position.x, target.z - m.position.z);
    if d.length() < 1e-4 {
        return false;
    }
    let held = world::arena().is_some_and(|a| liquids::burning_ahead(&a, m, d.normalize()));
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
/// adds the frame's time to route+0x1c, one under or equal is the best and the time 0 (asking to
/// move, the demo wants STUCK_PROGRESS closer: see it). Past STUCK_AFTER (time 0 again; the speed
/// factor char+0x720 at 0.01 or under would be put back to 1 and nothing more: the demo has no
/// speed factor, FUN_00048270; char+0x250 bit 0, the own point wanted, set: the demo's always
/// is):
///  - its own point the same as at the last stuck, the count (brain+0x9c4) over 1: put at the
///    route's current entry's point (FUN_000425b0 with it in EAX, 0x42839-0x42857: vtable +0x48
///    with the point's position + 0.25 m, then FUN_0004c5e0; medium: read as a set-position, here
///    its feet on the floor under that), nothing for a bare position (FUN_000425b0 tests EAX);
///  - else the detour (`detour`, FUN_00042280), and when its search finds nothing, the re-route
///    (`reroute`, FUN_00042490). (FUN_00042620's GOAL_ATTACK_GUARD between them is for a brain
///    out of the squad, brain+0x3c4 bit 8 clear: never the squad's.)
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
    // (the game starts the time again on any fall of the squared distance, or none; a character
    // asking to move must come STUCK_PROGRESS closer here until #344's feelers keep the squad off
    // the walls, the demo's collision jittering one held against a wall by a centimetre or so,
    // which with the game's test is never stuck. One standing by its goal's choice, a Toggle
    // Ability's hold, keeps the game's test: an equal distance starts the time again, so it isn't
    // stuck either)
    let closer = if pushing { d2.sqrt() < w.best.sqrt() - STUCK_PROGRESS } else { d2 <= w.best };
    if closer {
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
        if let Some(p) = short_of {
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
        return;
    }
    if detour(m, g) {
        return;
    }
    reroute(m, g);
}

/// FUN_00042280: the detour. With the current entry and the route's last point (here its last
/// entry, which for the demo's follow may be its place: the route's destination, `Router::dest`,
/// is planned to again):
///  - the current entry the last: the route done (route+0x20);
///  - else, the current point valid: it loses +0x28 bit 2 (`VALID`) for one search, the own point
///    too while it has it; the next entry the last: done; the next point valid: it loses it too,
///    and a new route to the destination is planned (FUN_00041660); found, it's started with the
///    pace level at 1 for 1 s and route+0x1c at -20;
///  - the bits given back (FUN_00042240).
///
/// False only when the search found nothing.
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
        WorldPoint { name, kind: 0xF95A_FB93, at: Vec3::new(x, 0.0, z), flags: 0xffff_4000, radius: -1.0, links: links.to_vec(), state: nav::VALID }
    }

    /// The names of a route's points.
    fn names(g: &Graph, w: &Walker) -> Vec<u32> {
        w.entries.iter().filter_map(|e| e.point).map(|p| g.nodes[p as usize].name).collect()
    }

    /// A line 1 - 2 - 3 - 4 - 7 and a way round 2 - 5 - 7: a member past 2 (its own point) on its
    /// way to 3 that comes no closer for 2 s gets the detour (FUN_00042280): 3, its own point 2 and
    /// the next, 4, off for one search, a new route to the destination 7 round them, the pace level
    /// at 1, the bits given back; the third stuck in a row at one own point puts it on the
    /// route's current point (FUN_000425b0).
    #[test]
    fn the_stuck_detour_and_the_warp() {
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3, 5]), point(3, 20.0, 0.0, &[2, 4]), point(4, 30.0, 0.0, &[3, 7]), point(5, 20.0, 10.0, &[2, 7]),
                   point(7, 40.0, 0.0, &[4, 5])];
        let mut g = Graph::new(&pts);
        let mut m = Player::new(2);
        m.position = Vec3::new(12.0, 0.0, 0.0);
        m.router.own.point = Some(1);
        plan(&mut m, &mut g, Dest::Point(5), "test").unwrap();
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7]);
        // standing by its goal's choice (asking for no move) it isn't stuck
        for _ in 0..32 {
            stuck(&mut m, &mut g, 1.0 / 15.0);
        }
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7]);
        // asking to run and no closer for over 2 s (held where it is): the detour
        m.router.asked = 3;
        for _ in 0..32 {
            stuck(&mut m, &mut g, 1.0 / 15.0);
        }
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [5, 7], "round 3 and 4 by 5");
        assert!((0..g.nodes.len() as u32).all(|i| g.valid(i)), "the bits given back");
        assert_eq!((m.router.cap, m.router.stuck_count), (1, 0));
        // held by the cap the time stays 0
        for _ in 0..40 {
            stuck(&mut m, &mut g, 1.0 / 15.0);
        }
        assert_eq!(m.router.stuck_count, 0);
        // two more stucks at the same own point, the level back up: the second detours again, the
        // third puts it on its route's current point
        for k in 1..=2 {
            plan(&mut m, &mut g, Dest::Point(5), "test").unwrap();
            m.router.cap = NO_CAP;
            for _ in 0..32 {
                stuck(&mut m, &mut g, 1.0 / 15.0);
            }
            assert_eq!(m.router.stuck_count, k);
        }
        assert_eq!(m.position.xz(), Vec2::new(20.0, 0.0), "put at its route's current point 3, not its own point 2");
    }

    /// The follow's points for its place (`place_points`): the points the leader's own point
    /// floods to within 12 m, the nearest the place first; a nearer point across a gap the links
    /// don't cross isn't among them.
    #[test]
    fn the_place_point_is_on_the_leaders_side() {
        // the leader's side: 1 - 2 - 3 along z = 0; across a gap (no link) 9 at (6, 6)
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 4.0, 0.0, &[1, 3]), point(3, 8.0, 0.0, &[2]), point(9, 6.0, 6.0, &[])];
        let g = Graph::new(&pts);
        let place = Vec3::new(5.0, 0.0, 5.0);
        assert_eq!(g.nearest_valid(place), Some(3), "the nearest of all is across the gap");
        assert_eq!(place_points(&g, place, Some(0)), [1, 2, 0]);
        // beyond 12 m of the leader's point it isn't flooded to: its own point the nearest left
        let far = [point(1, 0.0, 0.0, &[2]), point(2, 20.0, 0.0, &[1])];
        let g = Graph::new(&far);
        assert_eq!(place_points(&g, Vec3::new(19.0, 0.0, 0.0), Some(0)), [0]);
    }

    /// The pace ramp (FUN_00058670): set down to 1 for 1 s (a detour's drop), a member asking for
    /// the dash stands until the timer runs out, then climbs a notch every 0.17 s in FOLLOW (level
    /// 2 at once, 3 and 4 a step apart) and every 1.0 s on a Move To; it doesn't climb past what
    /// it asks. It moves at the lower of the level and what it asks.
    #[test]
    fn the_pace_ramp() {
        let mut g = Graph::new(&[point(1, 0.0, 0.0, &[])]);
        let dt = 1.0 / 60.0;
        let mut level_at = |mode: brain::Mode, asked: i32, t: f32| {
            let mut m = Player::new(2);
            m.command.mode = mode;
            m.router.asked = asked;
            m.router.set_pace(1, DETOUR_HOLD);
            for _ in 0..(t / dt).round() as i32 {
                ai_update(&mut m, &[], &mut g, dt);
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

    /// The stuck time of a member pressed against a wall: its distance to the target jittering by
    /// a centimetre (#335's review) is no progress while it asks to move (STUCK_PROGRESS, until
    /// #344), so the detour comes after 2 s; 10 cm closer every frame is, and it doesn't.
    #[test]
    fn jitter_is_no_progress() {
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3, 5]), point(3, 20.0, 0.0, &[2, 4]), point(4, 30.0, 0.0, &[3, 7]), point(5, 20.0, 10.0, &[2, 7]),
                   point(7, 40.0, 0.0, &[4, 5])];
        let mut g = Graph::new(&pts);
        let start = |g: &mut Graph| {
            let mut m = Player::new(2);
            m.router.own.point = Some(1);
            m.router.asked = 3;
            m.position = Vec3::new(12.0, 0.0, 0.0);
            plan(&mut m, g, Dest::Point(5), "test").unwrap();
            m
        };
        let mut m = start(&mut g);
        for k in 0..32 {
            m.position.x = 12.0 + if k % 2 == 0 { 0.01 } else { 0.0 } + k as f32 * 0.001;
            stuck(&mut m, &mut g, 1.0 / 15.0);
        }
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [5, 7], "jitter: the detour");
        let mut m = start(&mut g);
        for k in 0..32 {
            m.position.x = 12.0 + k as f32 * 0.1;
            stuck(&mut m, &mut g, 1.0 / 15.0);
        }
        assert_eq!(names(&g, m.router.walker.as_ref().unwrap()), [3, 4, 7], "walking on: no detour");
    }
}
