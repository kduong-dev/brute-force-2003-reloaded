//! The AI's navigation on the map's level (#334, the first slice of #242; viewer/docs/nav.md):
//! the world-point graph (bf/nav.rs) made as the map starts, the waypoint triggers' set-valid
//! applied to it as they act (play_triggers.rs hands their OPERATEs on, `WaypointSet`), and the
//! test hooks. Nothing walks a route yet but the BF_TEST_ROUTE walk: the squad's follow and
//! Move To (#335), the enemies' goals (#336) and the script walks (#337) are the next slices;
//! the Move To cursor's filter reads the graph's valid bit (`Nav::valid`).
//!
//! Test hooks:
//!  * BF_ROUTE_LOG=1: the graph at the map's start (its points, links, one-way links), each
//!    waypoint trigger's change to a point, and for the BF_TEST_ROUTE walk each plan (from where,
//!    the character's own point, the destination, the route's points and their names, the
//!    entry dropped, the search's cost), the own point's changes, and each step of the walk that
//!    does something (a point reached, a corner held, a point skipped, the end).
//!  * BF_TEST_ROUTE=<x>,<z>[,<s>] or <h_xxxxxxxx>[,<s>]: the controlled character walks a route
//!    (radius 2 m, as the AI's callers pass) to the valid world point nearest (x, the floor
//!    there, z), or to the point of that name, from time s (default BF_TEST_GO_AT's, 1 s); with
//!    BF_TEST_GOTO's start standing still (its goal the start: the autopilot leaves the stick to
//!    the walk). It runs at the stick's full tilt (BF_TEST_WALK, BF_TEST_SPRINT as for
//!    BF_TEST_GOTO), walking while the corner rule holds it (the hook's choice, see
//!    `test_route`), straight at each steering target (the AI's avoidance, FUN_000255d0, isn't
//!    this slice's), and stops at the route's end. The camera faces the way it goes, or keeps
//!    BF_CAMERA_YAW's heading when that's set (with BF_CAMERA_PITCH / BF_CAMERA_DISTANCE a view
//!    from above). The level's prisms aren't read (FUN_0004cb60's records), so no radius is held
//!    to 1 m indoors.

use bf_viewer::bf::nav::{self, Dest, Graph, NoRoute, OwnChange, OwnPoint, Route, Walker};

use super::*;

/// The graph's plugin: made as a map starts (after the level is placed), the waypoint triggers'
/// changes after their frame, the BF_TEST_ROUTE walk between the input and the character's step.
pub fn plugin(app: &mut App) {
    app.init_resource::<Nav>()
        .init_resource::<RouteTest>()
        .add_systems(OnEnter(AppState::Playing), start.after(setup))
        .add_systems(Update, (waypoints.after(triggers::run), test_route.after(waypoints).after(read_input).before(update_player))
            .run_if(in_state(AppState::Playing)));
}

/// The map's world-point graph (bf/nav.rs `Graph`), None off a level.
#[derive(Resource, Default)]
pub struct Nav {
    pub graph: Option<Graph>,
    /// BF_ROUTE_LOG is set
    pub log: bool,
}

impl Nav {
    /// Whether `Level::points`' point `i` has +0x28 bit 2 (`nav::VALID`): every point at the
    /// start (FUN_00059b60), then as the waypoint triggers set it. True without a graph.
    pub fn valid(&self, i: usize) -> bool {
        self.graph.as_ref().is_none_or(|g| g.valid(i as u32))
    }
}

/// A waypoint trigger's OPERATE (its handler 0x202c50 -> FUN_00202ba0), from play_triggers.rs:
/// the trigger (its name), its `world-point` (+0x44, by name), `set-valid` (+0x48) and
/// h_06d92de9 (+0x4c), -1 each for "leave it", and the time (s).
#[derive(Clone, Copy, Debug)]
pub struct WaypointSet {
    pub trigger: u32,
    pub point: u32,
    pub valid: i64,
    pub claim: i64,
    pub at: f32,
}

/// The world's ray as the route code asks it (`nav::Clear`, FUN_0004ce30): the arena's ray
/// (the terrain, the placed objects' collision, the blockers and hinters; liquids, open doors
/// and characters aren't in it) from `a` to `b`, none when they're more than `max` apart.
pub fn clear(arena: Option<&world::Arena>, a: Vec3, b: Vec3, max: f32) -> bool {
    let d = b - a;
    let len = d.length();
    if len > max {
        return false;
    }
    if len < 1e-4 {
        return true;
    }
    arena.is_none_or(|w| w.ray(a, d / len, len).is_none())
}

/// A map starts: its graph from the level's points (`Level::points`), every point valid; the
/// waypoint triggers' first updates (FIRST_UPDATE, the level's start) set theirs in the first
/// frame.
fn start(mut nav: ResMut<Nav>, map: Res<MapLevel>, mut test: ResMut<RouteTest>) {
    let log = std::env::var("BF_ROUTE_LOG").is_ok();
    let graph = map.0.as_ref().map(|l| Graph::new(&l.points));
    if log {
        if let Some(g) = &graph {
            let links: usize = g.nodes.iter().map(|n| n.links.len()).sum();
            let one_way = g.nodes.iter().enumerate().map(|(i, n)| n.links.iter().filter(|&&k| !g.nodes[k as usize].links.contains(&(i as u32))).count()).sum::<usize>();
            let no_skip = g.nodes.iter().filter(|n| n.state & nav::NO_SKIP != 0).count();
            println!("route graph: {} points, {links} links ({one_way} one way), {no_skip} with h_ed6a7490 (no look-ahead skip)", g.nodes.len());
        }
    }
    *nav = Nav { graph, log };
    *test = RouteTest::default();
}

/// The waypoint triggers' OPERATEs this frame (play_triggers.rs `take_waypoints`), applied to the
/// graph (`Graph::waypoint`, FUN_00202ba0).
fn waypoints(mut nav: ResMut<Nav>, mut tr: ResMut<triggers::Triggers>) {
    let sets = tr.take_waypoints();
    let log = nav.log;
    let Some(g) = nav.graph.as_mut() else { return };
    for s in sets {
        let Some(i) = g.find(s.point) else {
            if log {
                println!("t {:.2}: waypoint trigger h_{:08x}: its world-point h_{:08x} is no point: nothing", s.at, s.trigger, s.point);
            }
            continue;
        };
        let what = g.waypoint(i, s.valid, s.claim);
        if log {
            let n = &g.nodes[i as usize];
            let did = match what {
                nav::Waypoint::None => "nothing (set-valid and h_06d92de9 -1)".to_string(),
                nav::Waypoint::Claim => format!("+0x28 bit 0 {}", s.claim != 0),
                nav::Waypoint::Valid(v) => format!("{} (+0x28 bit 2)", if v { "valid" } else { "invalid" }),
                nav::Waypoint::Cover(v) => format!("a cover point: its +0x64 bit 5 {v} (not the graph's)"),
                nav::Waypoint::Other(k) => format!("kind {k:#x}: its own flag, not kept"),
            };
            println!("t {:.2}: waypoint trigger h_{:08x} on point h_{:08x} (kind {}, at ({:.1}, {:.1}, {:.1})): {did}", s.at, s.trigger, n.name, n.kind,
                     n.at.x, n.at.y, n.at.z);
        }
    }
}

/// The BF_TEST_ROUTE walk's state: its destination (a point), the controlled character's own
/// point, the walk, where the character was at the last step.
#[derive(Resource, Default)]
pub struct RouteTest {
    /// the hook read: its destination (a point, once found) and when it sets off
    read: bool,
    goal: Option<Goal>,
    from: f32,
    own: Option<OwnPoint>,
    walker: Option<Walker>,
    last: Option<Vec3>,
    /// the corner the walk held at last step (the log tells each once)
    held: Option<u32>,
    /// the walk is over (its end, or no route)
    over: bool,
}

/// BF_TEST_ROUTE's destination: a position (the nearest valid point to it, found once the graph
/// is set) or a point by name.
#[derive(Clone, Copy, Debug)]
enum Goal {
    At(f32, f32),
    Named(u32),
}

/// The hook's value: (the destination, the time) or None.
fn route_hook() -> Option<(Goal, Option<f32>)> {
    let v = std::env::var("BF_TEST_ROUTE").ok()?;
    let parts: Vec<&str> = v.split(',').map(|x| x.trim()).collect();
    if let Some(hex) = parts.first().and_then(|p| p.strip_prefix("h_")) {
        let name = u32::from_str_radix(hex, 16).ok()?;
        return Some((Goal::Named(name), parts.get(1).and_then(|s| s.parse().ok())));
    }
    let n: Vec<f32> = parts.iter().filter_map(|x| x.parse().ok()).collect();
    (n.len() >= 2).then(|| (Goal::At(n[0], n[1]), n.get(2).copied()))
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

/// BF_TEST_ROUTE (see the module notes): the controlled character's own point kept
/// (`OwnPoint::update`, FUN_0014f860), a route planned from it once (`Graph::plan`, FUN_000416a0)
/// and started (`Walker::start`, FUN_00025730), then walked (`Walker::step`, FUN_00026d80 at 2 m)
/// with the stick held toward each steering target.
fn test_route(mut test: ResMut<RouteTest>, mut nav: ResMut<Nav>, mut player: ResMut<Player>, time: Res<Time>) {
    let t = &mut *test;
    if !t.read {
        t.read = true;
        if let Some((goal, at)) = route_hook() {
            t.goal = Some(goal);
            t.from = at.or_else(|| std::env::var("BF_TEST_GO_AT").ok().and_then(|v| v.parse().ok())).unwrap_or(1.0);
            t.own = Some(OwnPoint::spawned());
        }
    }
    let (Some(goal), Some(own)) = (t.goal, t.own.as_mut()) else { return };
    let log = nav.log;
    let Some(g) = nav.graph.as_mut() else { return };
    let p = &mut *player;
    let dt = frame_dt(&time);
    let arena = world::arena();
    let ray = |a: Vec3, b: Vec3, max: f32| clear(arena.as_deref(), a, b, max);
    let me = p.position;
    // the own point, every frame (the game's for every living character with a brain)
    let jumping = matches!(p.action, Action::JumpCrouch | Action::JumpLaunch | Action::JumpFall | Action::JumpLand);
    if let Some(c) = own.update(g, me, dt, jumping, &ray) {
        if log {
            let (how, i) = match c {
                OwnChange::Near(i) => ("the near walk", i),
                OwnChange::Deep(i) => ("the 6-link search", i),
                OwnChange::Wide(i) => ("the 10-link search", i),
                OwnChange::Far(i) => ("the nearest valid point (FUN_000ab260)", i),
            };
            println!("t {:.2}: route test: own point {} by {how}, the character at ({:.1}, {:.1}, {:.1})", p.sim_time, label(g, Some(i)), me.x, me.y, me.z);
        }
    }
    if t.over || p.sim_time < t.from || p.dead {
        return;
    }
    // planned once, as a goal's start does
    if t.walker.is_none() {
        let goal = match goal {
            Goal::Named(n) => g.find(n),
            Goal::At(x, z) => g.nearest_valid(Vec3::new(x, floor_y(x, z, me.y + 50.0, world::Body::Character), z)),
        };
        let Some(goal) = goal else {
            t.over = true;
            if log {
                println!("t {:.2}: route test: no destination point", p.sim_time);
            }
            return;
        };
        let clock = std::time::Instant::now();
        let plan = g.plan(me, own.point, Dest::Point(goal), None, &ray);
        let us = clock.elapsed().as_secs_f64() * 1e6;
        match plan {
            Ok((route, found)) => {
                if log {
                    log_route(g, &route, me, own.point, goal, p.sim_time, found.popped, found.met, us);
                }
                t.walker = Some(Walker::start(g, &route, false, own.point));
                t.last = Some(me);
            }
            Err(why) => {
                t.over = true;
                if log {
                    let why = match why {
                        NoRoute::NoStart => "the character has no world point",
                        NoRoute::NoGoal => "the destination has no point",
                        NoRoute::Unreachable => "unreachable (the heap ran dry)",
                    };
                    println!("t {:.2}: route test: no route from {} to {}: {why} ({us:.0} us)", p.sim_time, label(g, own.point), label(g, Some(goal)));
                }
                return;
            }
        }
    }
    let w = t.walker.as_mut().expect("planned above");
    let last = t.last.replace(me).unwrap_or(me);
    let next = usize::try_from(w.index).ok().and_then(|i| w.entries.get(i + 1)).map(|e| e.at);
    let step = w.step(g, me, last, false, nav::ARRIVE, &ray);
    if log {
        let at = format!("the character at ({:.1}, {:.1})", me.x, me.z);
        if let Some(c) = step.corner {
            // (once a corner: what its two rays meet, FUN_00026a50's)
            if t.held != Some(c) {
                let what = |a: Vec3, b: Vec3| arena.as_deref().and_then(|w| w.ray_kind(a, (b - a).normalize_or_zero(), a.distance(b)))
                    .map_or("clear".to_string(), |(d, k)| format!("{k:?} at {d:.2} m"));
                let (a, b) = (Vec3::new(me.x, g.nodes[c as usize].at.y + 0.08, me.z), next.unwrap_or(me) + Vec3::Y * 0.08);
                let side = Vec2::new(b.z - a.z, a.x - b.x).normalize_or_zero() * nav::CORRIDOR;
                let off = Vec3::new(side.x, 0.0, side.y);
                println!("t {:.2}: route test: corner at {}: its next leg blocked (left {} from {:.3} to {:.3}, right {}), held to {} m till it's clear ({at})",
                         p.sim_time, label(g, Some(c)), what(a + off, b + off), a + off, b + off, what(a - off, b - off), nav::CORNER_RADIUS);
            }
        }
        t.held = step.corner;
        if let Some((pt, kind)) = step.reached {
            println!("t {:.2}: route test: reached {} (kind {kind}) within {:.1} m ({at})", p.sim_time, label(g, pt), step.radius);
        }
        if let Some(s) = step.skipped {
            println!("t {:.2}: route test: skipped {}: the corridor on is clear", p.sim_time, label(g, Some(s)));
        }
        if !step.walking {
            println!("t {:.2}: route test: the route's end ({at})", p.sim_time);
        }
    }
    if !step.walking {
        t.over = true;
        p.move_input = Vec2::ZERO;
        return;
    }
    // held at a corner it walks (the hook's choice: the controlled character's run turns in a
    // circle of some 0.7 m, 6 m/s at TURN_RATE, and would go round a point with a 1 m radius of
    // its own instead of closing on it; the AI's own pace near a point, FUN_0004d840, isn't read)
    if w.corner.is_some() {
        p.walk = true;
        p.sprint = false;
    }
    // the stick toward the target, in the camera's frame (as the squad AI's: play.rs `squad_ai`)
    let to = Vec2::new(step.target.x - me.x, step.target.z - me.z);
    if to.length() < 1e-3 {
        return;
    }
    let d = to.normalize();
    p.cam_yaw = std::env::var("BF_CAMERA_YAW").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| (-d.x).atan2(-d.y));
    let fwd = Vec2::new(-p.cam_yaw.sin(), -p.cam_yaw.cos());
    let right = Vec2::new(p.cam_yaw.cos(), -p.cam_yaw.sin());
    p.move_input = Vec2::new(d.dot(right), d.dot(fwd));
}

/// BF_ROUTE_LOG's lines for a plan: from where, the own point, the goal, each entry, the dropped
/// first entry, the cost.
#[allow(clippy::too_many_arguments)]
fn log_route(g: &Graph, route: &Route, me: Vec3, own: Option<u32>, goal: u32, now: f32, popped: usize, met: usize, us: f64) {
    println!("t {now:.2}: route test: from ({:.1}, {:.1}, {:.1}), own point {}, to {}: {} entries{}; the search popped {popped} and met {met} of {} points in {us:.0} us",
             me.x, me.y, me.z, label(g, own), label(g, Some(goal)), route.entries.len(),
             route.dropped.map_or(String::new(), |d| format!(", the own point {} dropped (the corridor to the second clear)", label(g, Some(d)))),
             g.nodes.len());
    for (k, e) in route.entries.iter().enumerate() {
        let flags = e.point.map_or(String::new(), |p| {
            let n = &g.nodes[p as usize];
            format!(" kind {}{}{}", n.kind, if n.state & nav::NO_SKIP != 0 { " no-skip" } else { "" }, if n.radius > 0.0 { format!(" r {:.1}", n.radius) } else { String::new() })
        });
        println!("  {k:2}: {}{flags}", label(g, e.point));
    }
}
