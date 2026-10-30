//! The Roller (#79), the squad's rolling bomb: "Round and intelligent - the rolling bomb seeks
//! enemies when deployed." Set down and blown up as play_grenade.rs's other placed types are (its
//! explosion h_f213e5b1: Damage 117-156, type 10, radius 5); what it does between is its item set's,
//! IFSET_ROLLING_BOMB (function-type 13), whose vtable 0x39ba60 has three slots of its own
//! (todo/79-roller-grenade/leads.md §3; viewer/docs/code-map.md):
//!  - the let-go 0x148640 -> FUN_00148720 makes the **seek helper** (`Roller`): a point 50 m
//!    (h_11884f2e) along the placer's horizontal aim (0x10fbf0), a check due in 1.0 s, no target;
//!  - every frame once it's out of the hand, 0x148690 -> FUN_001481c0 / FUN_00148220 (`seek`):
//!    the timers, and every 0.5 s a check: a target search (FUN_001483e0) when it has no living
//!    target or every 10th check (h_0a811e94), then the line of sight to the target (FUN_00148100):
//!    seen, the point moves to its hips; unseen for 5 s, it's dropped. Every frame the ball is
//!    spun toward the point (FUN_002bad80, `DRIVE_PUSH`), and **it goes off** once the point is
//!    within half the explosion's radius across and between 1.5 m below and 2 m above it;
//!  - the shutdown FUN_00148660 deletes the helper (the component goes with the grenade).
//!
//! The search (FUN_001483e0) takes the nearest living character of a team the team table makes
//! hostile to the bomb's (0x3ffd80: never the squad for the squad's), never the one who set it
//! down, within 50 m (3D, hips to the ball's middle) and in sight; a target is kept until it dies,
//! stays out of sight 5 s, or another comes within half its distance. The sight line (FUN_00148100:
//! the standard line-of-sight filter, vtable 0x39cc18, skipping the ball and the target) meets
//! only the terrain and the placed objects' collision: not the level's blockers and hinters, nor
//! characters, corpses, items, grenades, effects or liquids (`clear`). The takes' misses (05, 08,
//! 11, 12) are that ray meeting e34's scenery (leads Round 2 Q1).
//!
//! How it rolls is Ipion's (a 40 kg ball spun by an angular push every frame, its friction on the
//! ground not in the game's code), so it's a fit to the takes' 20 Hz logs of the ball
//! (todo/79-roller-grenade take05-take16 `.watch.csv`): the ball **hops**, flying straight between
//! ground contacts every ~0.6 s, and only a contact changes its way along the ground, toward a
//! drive the push builds (`drive`, `hop`). That gives the takes' speed by the half second (0.5,
//! 3.1, 5.6, 7.0, 7.5, 8.4 m/s from the let-go; a ceiling of 8.5), their 0.5-0.7 s lag after a
//! target is found, the wide turns braking to a few m/s, and the 45-degree target circled (take05).
//! A wall stops it with no bounce; it stays against it, pressed toward the point.
//!
//! Test hooks: BF_ROLLER_LOG=1 prints the let-go, every check (the search's candidates, the sight
//! line and what blocks it, the point), each ground contact, and the proximity blast; =2 also the
//! ball's position every frame. BF_ROLLER_RAYS=<file> casts the sight line once the map is up for
//! each line `x1 y1 z1 x2 y2 z2 [label]` of the file and prints what it meets.
//! Not done: the AI doesn't avoid a Roller (the set has no hazard slot, leads §3.5; nor does the
//! game's); a shot or a blast taking its 1 hp; a target on a ledge or a walking one (not filmed).

use super::*;
use super::grenade::{Grenade, GrenadeKit, GrenadeKits};

/// function-type IFSET_ROLLING_BOMB (the item set registered with the vtable 0x39ba60): the
/// Roller's definitions (h_fba7c475, m09_b's h_15f9c51a).
pub(super) const ROLLING_BOMB: i64 = 13;
/// The first check comes this long after the let-go (FUN_00148720: helper+0x24 = 1.0; the takes:
/// 0.93-1.02 s after the helper is first read, 14 takes).
const FIRST_CHECK: f32 = 1.0;
/// Then a check every CHECK_EVERY s (FUN_00148220 resets +0x24 to 0x3f000000, no carry; the
/// takes: every 0.507 s of the app clock, 201 intervals).
const CHECK_EVERY: f32 = 0.5;
/// A target may stay out of sight this long before it's dropped (+0x28 = 5.0, 0x40a00000, reset
/// at each check that sees it; take12: `t28` fell 1:1 while unseen and went back to 5.00).
const GRACE: f32 = 5.0;
/// The checks left before the first full search (+0x34 = 5 at the let-go: moot, as a check with no
/// target searches anyway; the takes: 5 -> 10 at the first check).
const FIRST_RESEARCH: i32 = 5;
/// The search radius and the first point's distance if a definition hasn't h_11884f2e (both
/// Rollers have 50; FUN_00148720 copies type+0x17c to +0x2c).
const DEFAULT_RADIUS: f32 = 50.0;
/// Checks between full searches if a definition hasn't h_0a811e94 (both Rollers have 10: type+0x178
/// -> +0x30).
const DEFAULT_RESEARCHES: i32 = 10;
/// The proximity rule (FUN_00148220): the point between DY_BELOW and DY_ABOVE m of the ball's
/// height (0x3a52f4 -1.5, 0x3a5008 2.0, exclusive) and within PROXIMITY_SHARE (0x3a4f74 0.5) of
/// the explosion's Damage radius across (item+0x1d8 -> +0x1fc: the Roller's 5 -> 2.5 m). The takes:
/// the last row before each of the nine blasts 2.46-2.70 m across from the target; take18's enemy
/// unseen behind a panel at 1.7-2.1 m didn't set it off (the rule is on the point, not on anyone).
const DY_BELOW: f32 = -1.5;
const DY_ABOVE: f32 = 2.0;
const PROXIMITY_SHARE: f32 = 0.5;
/// The drive's growth toward the point (m/s^2), its hold along the way to it (/s) and its decay
/// across it (/s): **a fit**. The game spins the ball toward the point every frame with the same
/// angular push (FUN_002bad80 with 4.0, 0x40c8f8, about the level axis square to the way to the
/// point, scaled by the horizontal part of the unit vector to it, not by the frame's time), and the
/// ground's friction inside Ipion turns the spin into the roll: none of that is in the game's code.
/// The demo keeps a drive velocity standing for the spin, `drive`; each ground contact takes the
/// ball's velocity CONTACT_SHARE of the way to it (`hop`). Fitted to takes 05-16's logs (contact
/// times and the speed of each hop between, 51 flights; then the whole paths, in a simulation fed
/// each take's logged points): straight, the hops leave at 3.1, 5.6, 7.2 and 8.5 m/s (the logs:
/// 2.5-3.8, 5.0-6.1, 6.5-7.7, 8.1-9.0) and the 0.5 s windows from the let-go come out 0.4, 3.2,
/// 5.4, 7.0, 7.7, 8.6 m/s (the spec: 0.5, 3.1, 5.6, 7.0, 7.5, 8.4), the ceiling DRIVE_PUSH /
/// DRIVE_HOLD = 8.5 m/s (the spec: 8-9); a target found, the way changes 0.6-0.7 s later (the
/// takes: 0.5-0.8 s), a 110-130 degree turn is done 1.3 s after (1.0-1.2 s) slowing to 2.4-3.4 m/s
/// (0.9-2.5), the blasts of takes 06, 07, 14, 15, 16 come 3.8, 4.2, 2.2, 2.8, 2.9 s after the
/// let-go (4.07, 4.48, 2.38, 3.16, 3.78), and take05's target 42 degrees off is circled for 2.5 s
/// before the blast at 5.3 s (5.9 s, circled 4 s). The paths keep within 0.6-2.0 m rms of the logs
/// (take05's circle goes round the other side: 4.4 m). A lower DRIVE_ACROSS (1.5) circles take05
/// too but turns the wide ones slower; a higher one (1.65) no longer circles it. The demo's own
/// runs of the takes' stagings on e34 are in viewer/docs/grenades.md, **Roller**.
const DRIVE_PUSH: f32 = 11.5;
const DRIVE_HOLD: f32 = 1.35;
const DRIVE_ACROSS: f32 = 1.6;
/// A ground contact takes the ball's velocity along the ground this share of the way to the drive
/// (`hop`): a fit with the drive's (see DRIVE_PUSH).
const CONTACT_SHARE: f32 = 0.8;
/// The ball leaves each ground contact at this speed along the ground's normal (m/s): **a fit**,
/// the takes' hops (51 flights between contacts: 0.4-0.85 s, 0.25-0.8 m high, leaving at 1.9-4.2
/// m/s; their mean flight 0.60 s). In flight its speed along the ground holds (the logs: straight
/// within 0.005-0.09 m rms) and it falls at the world's 9.81 (the logs' parabolas: 9.2-10.3).
const HOP_SPEED: f32 = 2.94;

/// A face whose normal's y is under this (by size) is a wall to a Roller's move along the ground
/// (`fly_grenades` sweeps the move with a ray against it): steeper than 65 degrees, the game's
/// ground limit (cos 65, 0x30c440; arena.rs FLOOR_NORMAL); a gentler one is ground, met by its hops.
pub(super) const WALL_NORMAL: f32 = 0.422_618_26;

/// A Roller let go (the seek helper FUN_00148720 makes at item+0x1b4: 0x3c bytes, vtable 0x39be8c,
/// confirmed live in the takes) and the demo's drive.
#[derive(Component, Debug)]
pub(super) struct Roller {
    /// the target (+4 / +8, a weak pointer to a character): None for none
    target: Option<Who>,
    /// the bomb's side (+0x14): the placer's team (item+0x1c0, kept by the on-placed FUN_001486c0),
    /// or the item-alignment of a level's own
    team: u8,
    /// who set it down (item+0x1e4, which the search skips); None for a level's own
    owner: Option<Who>,
    /// the point it's spun toward (+0x18..+0x20)
    pub(super) point: Vec3,
    /// seconds to the next check (+0x24)
    check: f32,
    /// seconds left for an unseen target (+0x28)
    grace: f32,
    /// the search's reach (+0x2c, h_11884f2e)
    radius: f32,
    /// checks between full searches (+0x30, h_0a811e94)
    researches: i32,
    /// checks left before the next full search (+0x34)
    research: i32,
    /// the target was out of sight at the last check (+0x38)
    out_of_sight: bool,
    /// the drive along the ground (m/s, x and z): the demo's stand-in for the ball's spin (a fit,
    /// see DRIVE_PUSH)
    pub(super) drive: Vec2,
    /// the ball's radius (m): its archetype's sphere (h_1ab09530: 0.19), what it rests on the
    /// ground and meets walls by
    pub(super) radius_m: f32,
    /// when it was let go (sim s), for BF_ROLLER_LOG
    since: f32,
}

impl Roller {
    /// The helper FUN_00148720 makes as the Roller `kit` is let go at `at`: its point
    /// h_11884f2e m along `aim`, the placer's horizontal aim forward (0x10fbf0: the aim frame's
    /// forward flattened to the level), or with no placer back along the item's own frame's third
    /// column (`own_z`); a check due in FIRST_CHECK s; no target; the bomb on `team`, skipping
    /// `owner` in its search.
    pub(super) fn let_go(kit: &GrenadeKit, at: Vec3, aim: Option<Vec3>, own_z: Vec3, team: u8, owner: Option<Who>, now: f32) -> Roller {
        let radius = if kit.def.seek_radius > 0.0 { kit.def.seek_radius } else { DEFAULT_RADIUS };
        let researches = if kit.def.proximity_radius > 0.0 { kit.def.proximity_radius as i32 } else { DEFAULT_RESEARCHES };
        let point = match aim {
            Some(f) => at + Vec3::new(f.x, 0.0, f.z).normalize_or(Vec3::NEG_Z) * radius,
            None => at - own_z.normalize_or(Vec3::Z) * radius,
        };
        Roller { target: None, team, owner, point, check: FIRST_CHECK, grace: GRACE, radius, researches, research: FIRST_RESEARCH,
                 out_of_sight: false, drive: Vec2::ZERO, radius_m: kit.splash_radius, since: now }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, cast_rays.run_if(in_state(AppState::Playing)));
    // (`seek` runs in play_grenade.rs's chain, right before the grenades fly)
}

/// Whether nothing of the world meets the sight line from `from` to `to` (FUN_00148100 ->
/// FUN_000de330 with the standard line-of-sight filter, vtable 0x39cc18: its kind mask 0x58 skips
/// the level's blockers and hinters, its class mask 0x415a1018 characters, effects, items,
/// grenades and liquids, and it skips the ball and the target): the terrain and the placed
/// objects' collision (`Arena::ray_shot`'s filter: the blockers left out, the liquids and open
/// doors too), as far as `to`; on the flat test floor its ground, pillars and pieces. Two points
/// within 1e-5 m are in sight (FUN_000dd8e0).
pub(super) fn clear(from: Vec3, to: Vec3) -> bool {
    blocker(from, to).is_none()
}

/// What meets the sight line from `from` to `to` first (see `clear`): how far along, and the hit
/// (None on the flat test floor's own ray).
fn blocker(from: Vec3, to: Vec3) -> Option<(f32, Option<world::ShotHit>)> {
    let d = to - from;
    let len = d.length();
    if len < 1e-5 {
        return None;
    }
    let dir = d / len;
    match world::arena().filter(|a| !a.is_flat()) {
        Some(a) => a.ray_shot(from, dir, len, None).filter(|h| h.t < len).map(|h| (h.t, Some(h))),
        None => ray_hit(from, dir, len).filter(|&t| t < len).map(|t| (t, None)),
    }
}

/// The characters the search looks through (the world's two lists, [world+0x68] and [world+0x74]:
/// every character, FUN_00146e00 walks the same): the controlled one and everyone else in the
/// world (the squad, the test map's NPCs, the level's spawned enemies).
fn in_world<'a>(player: &'a Player, squad: &'a Squad) -> Vec<&'a Player> {
    std::iter::once(player).chain(squad.0.iter()).filter(|u| !u.absent && !u.stand_in).collect()
}

/// The target search (FUN_001483e0) of `r` at `at`: the nearest living character hostile to the
/// bomb's team by the team table (0x3ffd80, `damage::hostile`), not its owner, whose hips
/// (`missile::root_point`: the character's node, the frame position FUN_001483e0 measures to) are
/// within the radius (3D, squared) and in sight (`clear`); with a living target, only one within
/// half its distance replaces it. No cone, no early exit: the nearest clear one wins.
fn search(r: &mut Roller, at: Vec3, units: &[&Player], log: bool, now: f32) {
    let mut best = r.radius * r.radius;
    let current = r.target.and_then(|w| units.iter().find(|u| u.who() == w)).filter(|u| !u.dead);
    if let Some(t) = current {
        best = best.min((super::missile::root_point(t).distance(at) * 0.5).powi(2));
    }
    let mut found = None;
    for u in units.iter().filter(|u| !u.dead && Some(u.who()) != r.owner && super::damage::hostile(r.team, u.team)) {
        let hips = super::missile::root_point(u);
        let d2 = hips.distance_squared(at);
        if d2 >= best {
            if log && d2 < r.radius * r.radius {
                println!("t {now:.2}:   {} at {:.1} m: not nearer than {:.1} m", unit_name(u), d2.sqrt(), best.sqrt());
            }
            continue;
        }
        let seen = blocker(at, hips);
        if log {
            match seen {
                None => println!("t {now:.2}:   {} at {:.1} m: in sight", unit_name(u), d2.sqrt()),
                Some((t, hit)) => println!("t {now:.2}:   {} at {:.1} m: the sight line meets {} {t:.1} m out", unit_name(u), d2.sqrt(), what(hit)),
            }
        }
        if seen.is_none() {
            best = d2;
            found = Some(u.who());
        }
    }
    if let Some(f) = found.filter(|f| Some(*f) != r.target) {
        r.target = Some(f);
    }
}

/// What a sight line met, for the log: the terrain, a placed object (its index and type) or the
/// test floor's ground, pillars or pieces.
fn what(hit: Option<world::ShotHit>) -> String {
    let Some(h) = hit else { return "the test floor".into() };
    if h.terrain {
        return "the terrain".into();
    }
    match world::arena().map(|a| a.body_kind(h.body)) {
        Some(world::BodyKind::Object(i)) => format!("object {i}"),
        Some(k) => format!("{k:?}"),
        None => "something".into(),
    }
}

/// One frame of a Roller's helper (FUN_001481c0, then FUN_00148220's check), the ball at `at`:
/// the check timer runs down; an unseen target's grace runs down and at 0 the target is dropped;
/// at a check the full search runs when the count is out or there's no living target, then a
/// target in sight moves the point to its hips (the grace back to 5), one out of sight leaves it.
fn tick(r: &mut Roller, at: Vec3, units: &[&Player], dt: f32, now: f32, log: bool) {
    if r.check > 0.0 {
        r.check -= dt;
    }
    if r.out_of_sight {
        r.grace -= dt;
        if r.grace <= 0.0 && r.target.take().is_some() && log {
            println!("t {now:.2}: Roller: its target out of sight {GRACE} s: dropped");
        }
    }
    if r.check > 0.0 {
        return;
    }
    r.research -= 1;
    let living = r.target.and_then(|w| units.iter().find(|u| u.who() == w)).is_some_and(|u| !u.dead);
    if log {
        println!("t {now:.2}: Roller check at {at:.2} ({:.2} s after the let-go), checks left {}, target {}", now - r.since, r.research,
                 r.target.and_then(|w| units.iter().find(|u| u.who() == w)).map_or("none".into(), |u| unit_name(u)));
    }
    if r.research < 1 || !living {
        search(r, at, units, log, now);
        r.research = r.researches;
    }
    if let Some(t) = r.target.and_then(|w| units.iter().find(|u| u.who() == w)) {
        let hips = super::missile::root_point(t);
        if clear(at, hips) {
            r.out_of_sight = false;
            r.grace = GRACE;
            r.point = hips;
        } else {
            r.out_of_sight = true;
        }
        if log {
            println!("t {now:.2}:   target {} at {:.1} m, {}: point {:.2}", unit_name(t), hips.distance(at),
                     if r.out_of_sight { "out of sight" } else { "in sight" }, r.point);
        }
    }
    r.check = CHECK_EVERY;
}

/// The drive after `dt` with the point `to` away (3D): it grows toward the point at DRIVE_PUSH
/// times the horizontal part of the unit vector to it (FUN_00148220's push axis (d.z, 0, -d.x) /
/// |d|: a point far above or below pushes less) and is held back at DRIVE_HOLD along the way to the
/// point and DRIVE_ACROSS across it, exactly over the frame (the same at any frame rate). A fit:
/// see DRIVE_PUSH.
fn drive(u: Vec2, to: Vec3, dt: f32) -> Vec2 {
    let len = to.length();
    let flat = Vec2::new(to.x, to.z);
    if len < 1e-6 || flat.length() < 1e-6 {
        return u * (-DRIVE_HOLD * dt).exp();
    }
    let push = flat.length() / len;
    let t = flat.normalize();
    let along = u.dot(t);
    let across = u - t * along;
    let k = (-DRIVE_HOLD * dt).exp();
    let along = along * k + DRIVE_PUSH * push / DRIVE_HOLD * (1.0 - k);
    t * along + across * (-DRIVE_ACROSS * dt).exp()
}

/// A Roller's ground contact (`fly_grenades`), the ground's `normal` up: coming down into the
/// ground, its velocity along the ground (in the ground's plane) goes CONTACT_SHARE of the way to
/// the drive turned into that plane (the spin's rim speed, whatever the slope), and it leaves again
/// at HOP_SPEED along the normal (a fit, see HOP_SPEED). Returns false, the velocity untouched, when
/// it isn't coming down into it (the ground rose under it: it's only set on the ground), so a slope
/// slows it as gravity does in its flights, and lifting it onto a rising floor gives it nothing.
pub(super) fn hop(velocity: &mut Vec3, drive: Vec2, normal: Vec3) -> bool {
    let vn = velocity.dot(normal);
    if vn >= 0.0 {
        return false;
    }
    let along = *velocity - normal * vn;
    let u = Vec3::new(drive.x, 0.0, drive.y);
    let u = (u - normal * u.dot(normal)).normalize_or_zero() * u.length();
    *velocity = along + (u - along) * CONTACT_SHARE + normal * HOP_SPEED;
    true
}

/// The ground's normal under (x, z), no higher than `top` (a thing's floor: `Arena::floor_below`
/// with `Body::Thing`); straight up on the flat test floor.
pub(super) fn ground_normal(x: f32, z: f32, top: f32) -> Vec3 {
    world::arena().and_then(|a| a.floor_below(x, z, top, world::Body::Thing)).map_or(Vec3::Y, |f| f.1.normalize_or(Vec3::Y))
}

/// How the ball turns as it rolls (rad/s about each axis): rolling along its drive, about the level
/// axis square to it, at the drive's speed over its radius (the game's ball spins with its push;
/// the drive stands for that spin).
pub(super) fn spin(r: &Roller) -> Vec3 {
    Vec3::new(r.drive.y, 0.0, -r.drive.x) / r.radius_m.max(0.05)
}

/// Every Roller out of the hand, each frame before the grenades fly (FUN_00148220 from the item
/// set's per-frame hook 0x148690): its helper (`tick`), its drive toward the point (`drive`), and
/// the proximity rule, which sets its fuse to 0 and arms it (+0x1d4 = 0, +0x1dc |= 2: it goes off
/// in this frame's grenade update, `fly_grenades`).
pub(super) fn seek(time: Res<Time>, player: Res<Player>, squad: Res<Squad>, kits: Option<Res<GrenadeKits>>,
                   mut rollers: Query<(Entity, &mut Grenade, &Transform, &mut Roller)>) {
    let Some(kits) = kits else { return };
    let dt = frame_dt(&time);
    if dt <= 0.0 {
        return;
    }
    let log = std::env::var("BF_ROLLER_LOG").ok();
    let now = player.sim_time;
    let units = in_world(&player, &squad);
    for (e, mut g, tr, mut r) in &mut rollers {
        let Some(kit) = kits.0.get(g.kind) else { continue };
        let at = tr.translation;
        tick(&mut r, at, &units, dt, now, log.is_some());
        let to = r.point - at;
        r.drive = drive(r.drive, to, dt);
        if log.as_deref() == Some("2") {
            println!("t {now:.3}: Roller {e} at {at:.3}, drive {:.2} m/s, point {:.2}", r.drive.length(), r.point);
        }
        let reach = kit.blast.blast_radius * PROXIMITY_SHARE;
        if g.fuse > 0.0 && to.y > DY_BELOW && to.y < DY_ABOVE && Vec2::new(to.x, to.z).length_squared() < reach * reach {
            g.fuse = 0.0;
            g.landed = true;
            if log.is_some() {
                println!("t {now:.2}: Roller {e} within {reach:.1} m of its point ({:.2} m across, {:+.2} m up): it goes off, {:.2} s after the let-go",
                         Vec2::new(to.x, to.z).length(), to.y, now - r.since);
            }
        }
    }
}

/// BF_ROLLER_RAYS=<file>: once the map is up, the sight line (`clear`, FUN_00148100's) for each
/// line `x1 y1 z1 x2 y2 z2 [label]` of the file (world metres: the takes' logged ball and enemy hips
/// at each check), printed with what it meets first and how far along, and the placed object's
/// type. Lines starting with # are skipped.
fn cast_rays(map: Res<MapLevel>, player: Res<Player>, mut done: Local<bool>) {
    let Ok(path) = std::env::var("BF_ROLLER_RAYS") else { return };
    if *done || world::arena().is_none() || (player.loaded.is_none() && !player.stand_in) {
        return;
    }
    *done = true;
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            println!("BF_ROLLER_RAYS: {path}: {e}");
            return;
        }
    };
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.split_whitespace().collect();
        let n: Vec<f32> = f.iter().take(6).filter_map(|x| x.parse().ok()).collect();
        if n.len() < 6 {
            println!("BF_ROLLER_RAYS: skipped {line:?}");
            continue;
        }
        let (a, b) = (Vec3::new(n[0], n[1], n[2]), Vec3::new(n[3], n[4], n[5]));
        let label = f[6..].join(" ");
        match blocker(a, b) {
            None => println!("ray {label}: {a:.2} -> {b:.2} ({:.1} m): clear", a.distance(b)),
            Some((t, hit)) => {
                let kind = hit.and_then(|h| match world::arena().map(|ar| ar.body_kind(h.body)) {
                    Some(world::BodyKind::Object(i)) => map.0.as_ref().and_then(|l| l.objects.get(i as usize)).map(|o| format!(" (type h_{:08x} at {:.1})", o.kind, o.transform.w_axis.truncate())),
                    _ => None,
                }).unwrap_or_default();
                println!("ray {label}: {a:.2} -> {b:.2} ({:.1} m): meets {}{kind} {t:.2} m out, at {:.2}", a.distance(b), what(hit), a + (b - a).normalize() * t);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Straight at a far point, the hops leave at the takes' speeds: the drive grows toward the
    /// ceiling DRIVE_PUSH / DRIVE_HOLD (8.5 m/s) and each contact takes 0.8 of the way to it: from
    /// the let-go, a fall of 0.45 s then hops every 0.6 s give 3.1, 5.6, 7.2 and 8.0-8.5 m/s (the
    /// logs: 2.5-3.8, 5.0-6.1, 6.5-7.7, 8.1-9.0).
    #[test]
    fn straight_hops_speed_up_as_the_takes_do() {
        let point = Vec3::new(50.0, 0.0, 0.0);
        let (mut u, mut v) = (Vec2::ZERO, Vec3::ZERO);
        let (mut t, dt) = (0.0f32, 1.0 / 30.0);
        let mut contact = 0.45;
        let mut speeds = vec![];
        while speeds.len() < 4 {
            u = drive(u, point, dt);
            t += dt;
            if t >= contact {
                // (coming down: the fall from the hand, then each hop's)
                v.y = -HOP_SPEED;
                assert!(hop(&mut v, u, Vec3::Y));
                speeds.push(Vec2::new(v.x, v.z).length());
                contact += 2.0 * HOP_SPEED / 9.81;
            }
        }
        let want = [(2.5, 3.8), (5.0, 6.1), (6.5, 7.7), (7.5, 9.0)];
        for (s, (lo, hi)) in speeds.iter().zip(want) {
            assert!(*s >= lo && *s <= hi, "{speeds:?}");
        }
        // (the ceiling)
        for _ in 0..600 {
            u = drive(u, point, dt);
        }
        assert!((u.length() - DRIVE_PUSH / DRIVE_HOLD).abs() < 0.01, "{u}");
    }

    /// The drive is the same at 15, 30 or 60 frames a second (it's exact over a frame).
    #[test]
    fn the_drive_doesnt_depend_on_the_frame_rate() {
        let point = Vec3::new(10.0, 1.0, -20.0);
        let run = |dt: f32| {
            let mut u = Vec2::new(3.0, 4.0);
            for _ in 0..(2.0 / dt).round() as usize {
                u = drive(u, point, dt);
            }
            u
        };
        let (a, b, c) = (run(1.0 / 15.0), run(1.0 / 30.0), run(1.0 / 60.0));
        assert!(a.distance(b) < 1e-3 && b.distance(c) < 1e-3, "{a} {b} {c}");
    }

    /// A point straight above pushes nothing (the push axis is the horizontal part of the unit
    /// vector to it), and the drive only decays.
    #[test]
    fn a_point_overhead_pushes_nothing() {
        let u = drive(Vec2::new(2.0, 0.0), Vec3::new(0.0, 5.0, 0.0), 0.5);
        assert!(u.x < 2.0 && u.x > 0.0 && u.y.abs() < 1e-6, "{u}");
    }

    /// The search (FUN_001483e0) on the flat test floor (no arena: its ground and pillars, none in
    /// the way here): the nearest living hostile wins, never the squad, the owner, a corpse or a
    /// neutral; a held target gives way only to one within half its distance (3D, to the hips).
    #[test]
    fn the_search_takes_the_nearest_hostile_and_keeps_its_target() {
        let unit = |team: u8, x: f32, npc: u32, dead: bool| {
            let mut u = Player::new(3);
            u.team = team;
            // (the root over the flat floor's GROUND: the hips at y 0, `missile::root_point`)
            u.position = Vec3::new(x, 0.0, 4.5);
            u.dead = dead;
            if npc > 0 {
                u.npc = Some(super::super::testtools::Npc { id: npc, fight: false, face: 0.0 });
            }
            u
        };
        let tex = unit(0, 1.0, 0, false);
        let mut a = unit(1, 6.0, 1, false);
        let b = unit(1, -4.0, 2, false);
        let corpse = unit(1, 2.0, 3, true);
        let friend = unit(0, 1.5, 4, false);
        let neutral = unit(2, 3.0, 5, false);
        let ball = Vec3::new(0.0, GROUND + 0.19, 4.5);
        let mut r = Roller { target: None, team: 0, owner: Some(tex.who()), point: Vec3::ZERO, check: 0.0, grace: GRACE, radius: 50.0,
                             researches: 10, research: 1, out_of_sight: false, drive: Vec2::ZERO, radius_m: 0.19, since: 0.0 };
        search(&mut r, ball, &[&tex, &a, &b, &corpse, &friend, &neutral], false, 0.0);
        assert_eq!(r.target, Some(b.who()));
        // (A nearer than B, but not within half B's 4.1 m: B is kept; within half: A)
        a.position.x = 3.0;
        search(&mut r, ball, &[&tex, &a, &b], false, 0.0);
        assert_eq!(r.target, Some(b.who()));
        a.position.x = 1.5;
        search(&mut r, ball, &[&tex, &a, &b], false, 0.0);
        assert_eq!(r.target, Some(a.who()));
        // (nobody within the radius: no target; a squad bomb never takes the squad)
        let mut alone = Roller { target: None, radius: 2.0, ..r };
        search(&mut alone, ball, &[&tex, &b, &friend], false, 0.0);
        assert_eq!(alone.target, None);
    }

    /// A contact takes the velocity 0.8 of the way to the drive and leaves along the normal.
    #[test]
    fn a_hop_turns_toward_the_drive() {
        let mut v = Vec3::new(5.0, -2.0, 0.0);
        assert!(hop(&mut v, Vec2::new(0.0, 5.0), Vec3::Y));
        assert!((v - Vec3::new(1.0, HOP_SPEED, 4.0)).length() < 1e-5, "{v}");
    }

    /// On a slope the drive is turned into the ground's plane, its speed kept, and the hop leaves
    /// along the normal; a ball not coming down into the ground (the ground rose under it) is left
    /// as it was.
    #[test]
    fn a_hop_on_a_slope_stays_in_its_plane() {
        let n = Vec3::new(-0.5, 0.75f32.sqrt(), 0.0);
        let mut v = Vec3::new(4.0, -3.0, 0.0);
        assert!(hop(&mut v, Vec2::new(8.0, 0.0), n));
        assert!(((v - n * HOP_SPEED).dot(n)).abs() < 1e-4, "{v}");
        let mut up = Vec3::new(4.0, 3.0, 0.0);
        assert!(!hop(&mut up, Vec2::new(8.0, 0.0), n) && up == Vec3::new(4.0, 3.0, 0.0));
    }
}
