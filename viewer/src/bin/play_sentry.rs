//! The Sentry (#80), the squad's proximity mine: "Explosive trap that detonates when an enemy
//! approaches."
//!
//! It's set down and goes off as play_grenade.rs's other types do (place_hi; its explosion
//! h_01f142eb: Damage 62.5-88.5, type 10, radius 4; exp-mine + light_explosion and 145f09e5).
//! Its `timer` is 9999 s: what sets it off is the item set IFSET_PROXIMITY_EXPLOSIVE's
//! (function-type 8) handlers in default.xbe, vtable 0x39bc50:
//!  - on-placed 0x147570 marks it placed (+0x1bc bit 2) once it's down; nothing else arms it
//!    (no arming timer in the code);
//!  - per frame 0x147620, once placed: a check every 0.05-0.15 s (`CHECK_EVERY`), and if the
//!    check says so it goes off at once (vfunc +0x15c);
//!  - the check 0x146e00 goes through every living character: a friend - on the mine's team
//!    (unless the team table at 0x3ffd80 makes that team hostile to itself: team 7) or the
//!    thrower - within the radius
//!    stops it there (no blast this check); anyone else within it is a target. It goes off when
//!    there's a target and no friend within the radius. The radius is the item's h_0a811e94
//!    (3 m, every Sentry), the test 0x146cb0 a 3D sphere round the mine's node.
//!  - characters whose type handles mines and that know of this one only count standing on it
//!    (filter 0x147430, `WISE_REACH`);
//!  - it has 1 hitpoint: a shot that strikes it sets it off.
//!
//! The demo has no enemies: the squad is one team, so its mines only go off for a test hook's
//! hostile (BF_TEST_HOSTILE) or a shot. Measured (todo/80-sentry-mine, the tester's spec): never
//! set off by Tex (the thrower) walking or running over it or standing beside it, nor by Flint
//! stepping onto it (0.16 m); an enemy brought up a slope set it off at 2.51 m across / 0.63 m
//! above the mine's feet, not at 2.61 / 0.67; gone <= 0.1 s after; a shot sets it off.
//! Not done: disarming an enemy's mine (the tutorial's "Hold [X] to disarm": 0x14a410 state 6,
//! FUN_00121860), the AI's avoidance of it (hazard kind 3 at +0x110, 2 m then 5 m at +0x114),
//! one blast setting off another (needs blasts to hurt objects).

use super::*;
use super::grenade::{Grenade, GrenadeKits};

/// function-type IFSET_PROXIMITY_EXPLOSIVE (the item set registered at 0x1469f9 with the
/// vtable 0x39bc50): the Sentry's definitions.
const PROXIMITY_EXPLOSIVE: i64 = 8;
/// The check comes when the mine's timer (+0x1d0) passes CHECK_EVERY s (0x147620: += dt, then
/// compared with 0.15 at 0x3a51e4); then the timer restarts at a random 0..CHECK_JITTER s (the
/// game's rand in [1, 2) - 1, times 0.1 at 0x3a4ff0): a check every 0.05-0.15 s, the first
/// 0.15 s after it's down.
const CHECK_EVERY: f32 = 0.15;
const CHECK_JITTER: f32 = 0.1;
/// The trigger radius if a definition hasn't h_0a811e94 (every Sentry has 3).
const DEFAULT_RADIUS: f32 = 3.0;
/// Characters whose type handles mines (character type byte +0x297, hazard kind 3) and whose
/// brain knows of this mine (brain vfunc +0x94(3, mine)) count only standing on it: within
/// WISE_REACH m across (0.09 = 0.3^2 at 0x3a52f0) and WISE_DY m up or down (+-1.5 at 0x3a4fb4,
/// 0x3a52f4) - filter 0x147430. Which characters have the flag isn't read from the data yet:
/// only BF_TEST_HOSTILE's `wise` makes one so.
const WISE_REACH: f32 = 0.3;
const WISE_DY: f32 = 1.5;
/// BF_TEST_HOSTILE's defaults: from 5 m out, step in to the mine at 0.25 m/s, after waiting
/// RIG_WAIT s where it's put. (The test's choice: slow enough that a check every <= 0.15 s
/// catches it within 0.04 m.)
const RIG_FROM: f32 = 5.0;
const RIG_TO: f32 = 0.0;
const RIG_SPEED: f32 = 0.25;
const RIG_WAIT: f32 = 1.0;

/// A Sentry that's down (placed): its check timer (+0x1d0) and its owner's team when it was
/// set down (+0x1c0, written by 0x147570).
#[derive(Component)]
pub(super) struct Mine {
    check: f32,
    team: u8,
}

/// The Sentries that are down, for the shots (1 hitpoint): each one's entity and bounding
/// sphere (world centre, radius), as of the last `trip`; and the shots on their way to one -
/// the mine, and seconds until the shot gets there (it goes off then). play.rs's
/// `update_player` tests each shot against them as it's made, before the teammates in its way
/// (the first on the line takes it); the bodies on the ground don't stop shots (they're shoved
/// and the shot goes on), so a mine behind one is still struck.
#[derive(Resource, Default)]
pub(super) struct MineTargets {
    targets: Vec<(Entity, Vec3, f32)>,
    struck: Vec<(Entity, f32)>,
}

impl MineTargets {
    /// The nearest mine on a shot's line (from `origin` along `dir`) before `dist`, and how far
    /// along it the shot meets its sphere.
    pub(super) fn first_on(&self, origin: Vec3, dir: Vec3, dist: f32) -> Option<(Entity, f32)> {
        self.targets.iter().filter_map(|&(e, c, r)| {
            let along = (c - origin).dot(dir);
            let miss = (origin + dir * along).distance(c);
            (along > 0.0 && miss < r && along - r < dist).then(|| (e, (along - (r * r - miss * miss).sqrt()).max(0.0)))
        }).min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// A shot meets mine `e` `t` m out, flying at `speed` m/s: it goes off when the shot gets
    /// there. (`now`, the sim time, for BF_SENTRY_LOG.)
    pub(super) fn strike(&mut self, e: Entity, t: f32, speed: f32, now: f32) {
        self.struck.push((e, t / speed.max(1.0)));
        if std::env::var("BF_SENTRY_LOG").is_ok() {
            println!("t {now:.2}: a shot strikes the mine {e} {t:.1} m out");
        }
    }
}

/// The test hooks' set-up (read when a map starts) and the first mine they're staged round.
#[derive(Resource, Default)]
pub(super) struct TestRig {
    /// BF_TEST_HOSTILE: the character, from / to (m), speed (m/s), wise (see WISE_REACH)
    hostile: Option<(usize, f32, f32, f32, bool)>,
    /// BF_TEST_MINE_FRIEND: the character, how far from the mine (m), for how long after it's
    /// down (s)
    friend: Option<(usize, f32, f32)>,
    /// BF_TEST_SHOOT_MINE: seconds after the first mine is down
    shoot: Option<f32>,
    /// the first mine down: it, where it lies, the way from its thrower to it when it came down
    /// (flat, unit), when
    first: Option<(Entity, Vec3, Vec3, f32)>,
    /// the hostile's distance when the mine went (it's held there after)
    held_at: Option<f32>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<MineTargets>().init_resource::<TestRig>()
        .add_systems(OnEnter(AppState::Playing), reset)
        .add_systems(Update, test_rig.after(squad_control).before(update_player).run_if(in_state(AppState::Playing)));
    // (`trip` runs in play_grenade.rs's chain, right before the grenades' fuses)
}

/// A character by name (brutus, flint, hawk, tex) or CHARACTERS index.
fn character(s: &str) -> Option<usize> {
    let s = s.trim();
    s.parse::<usize>().ok().filter(|&i| i < CHARACTERS.len()).or_else(|| CHARACTERS.iter().position(|c| c.eq_ignore_ascii_case(s)))
}

/// A new map: no shots on their way, the test hooks read afresh.
///
/// Test hooks:
///  - BF_TEST_HOSTILE=<character>[,<from m>,<to m>,<m/s>][,wise]: that squadmate is on another
///    team (hostile to the squad's mines) and stands still, out of the squad AI. Once the first
///    Sentry is down it's put <from> m (default 5) beyond the mine, on the far side from the
///    thrower, waits RIG_WAIT s, then is stepped in toward the mine at <m/s> (0.25) until <to>
///    m (0) across; held where it was once the mine goes. `wise`: it handles mines and knows
///    this one (counts only standing on it).
///  - BF_TEST_MINE_FRIEND=<character>,<m>[,<s>]: once the first mine is down, that squadmate
///    is held <m> m beside it (square to the thrower's line) for <s> s (default: for good), then
///    let go to the squad AI; until the mine is down it follows the squad as usual.
///  - BF_TEST_SHOOT_MINE=<s>: <s> s after the first mine is down, the player (under
///    BF_TEST_GOTO's autopilot) aims at it and fires until it's gone.
fn reset(mut targets: ResMut<MineTargets>, mut rig: ResMut<TestRig>) {
    targets.targets.clear();
    targets.struck.clear();
    let list = |k: &str| std::env::var(k).ok().map(|v| v.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>());
    let num = |v: &[String], i: usize, d: f32| v.get(i).and_then(|x| x.parse::<f32>().ok()).unwrap_or(d);
    *rig = TestRig {
        hostile: list("BF_TEST_HOSTILE").and_then(|v| Some((character(v.first()?)?, num(&v, 1, RIG_FROM), num(&v, 2, RIG_TO), num(&v, 3, RIG_SPEED),
                                                           v.iter().any(|x| x.eq_ignore_ascii_case("wise"))))),
        friend: list("BF_TEST_MINE_FRIEND").and_then(|v| Some((character(v.first()?)?, num(&v, 1, 2.0), num(&v, 2, f32::MAX)))),
        shoot: std::env::var("BF_TEST_SHOOT_MINE").ok().and_then(|v| v.trim().parse::<f32>().ok()),
        first: None,
        held_at: None,
    };
}

/// How far above the feet a character is for the mine's check (m). The game tests the
/// character's node position (0x146cb0); where that is on the body isn't read. Fitted to the
/// recording's enemy brought up a slope toward the mine (take03): feet 2.51 m across and 0.63 m
/// above the mine's node set it off, 2.61 m / 0.67 m didn't (2 s), so with the 3 m sphere the
/// point is 0.81-1.01 m above the feet; the middle of that. (The mine's own point is its
/// model's middle, GRENADE_RADIUS above the ground here: the model is 0.14 m tall.) On flat
/// ground a character sets it off from 2.88 m across.
const BODY_CENTRE: f32 = 0.9;

/// Where a character is for the mine's check: BODY_CENTRE above the feet (play.rs's position is
/// the root, -GROUND above them), raised with a jump.
fn body_point(u: &Player) -> Vec3 {
    u.position + Vec3::Y * (u.height + GROUND + BODY_CENTRE)
}

/// The team the game's team table (0x3ffd80, 10 x 10 bytes) makes hostile to itself: its
/// diagonal is 0 but for team 7. (Characters on different teams are always hostile to each
/// other in the check: the table is only looked up for the same team.) The demo's teams (0 the
/// squad, 1 BF_TEST_HOSTILE's) aren't the game's numbers; none is 7.
const SELF_HOSTILE_TEAM: u8 = 7;

/// Whether `u` is a friend of a mine set down by `thrower` on team `team`: the same team (and
/// not one hostile to itself) or the thrower (0x146e00).
fn friend(u: &Player, thrower: usize, team: u8) -> bool {
    (u.team == team && team != SELF_HOSTILE_TEAM) || u.character == thrower
}

/// The game's check (0x146e00) for a mine at `at` with trigger `radius`, set down by character
/// `thrower` on team `team`: true if any character not its friend is within the radius and no
/// friend is - the first friend found within it ends the check. `wise` characters count only
/// standing on it (WISE_REACH).
fn check(at: Vec3, radius: f32, thrower: usize, team: u8, units: &[&Player], wise: impl Fn(&Player) -> bool) -> bool {
    let mut found = false;
    for u in units.iter().filter(|u| !u.dead) {
        let p = body_point(u);
        if wise(u) {
            let d = p - at;
            if d.y.abs() >= WISE_DY || Vec2::new(d.x, d.z).length() >= WISE_REACH {
                continue;
            }
        }
        let near = p.distance_squared(at) < radius * radius;
        if friend(u, thrower, team) {
            // a friend near: not this time
            if near {
                return false;
            }
        } else {
            found |= near;
        }
    }
    found
}

/// The Sentries that are down: each runs its check every 0.05-0.15 s and goes off when it says
/// so, or when a shot that struck it (play.rs's `update_player`, see `MineTargets`) gets there
/// (the fuse set to 0: play_grenade.rs's `fly_grenades` sets it off this frame). Their bounding
/// spheres are handed to the next frame's shots. BF_SENTRY_LOG=1 prints each check with a
/// target about, and what set one off.
#[allow(clippy::too_many_arguments)]
pub(super) fn trip(mut commands: Commands, time: Res<Time>, player: Res<Player>, squad: Res<Squad>,
                   kits: Option<Res<GrenadeKits>>, mut targets: ResMut<MineTargets>, rig: Res<TestRig>,
                   mut mines: Query<(Entity, &mut Grenade, &Transform, Option<&mut Mine>)>, mut rng: Local<u32>) {
    let Some(kits) = kits else { return };
    let dt = frame_dt(&time);
    let log = std::env::var("BF_SENTRY_LOG").is_ok();
    let is_mine = |g: &Grenade| g.landed && kits.0.get(g.kind).is_some_and(|k| k.def.function_type == PROXIMITY_EXPLOSIVE);
    let now = player.sim_time;
    let mut due = vec![];
    targets.struck.retain_mut(|(e, left)| {
        *left -= dt;
        if *left <= 0.0 {
            due.push(*e);
        }
        *left > 0.0
    });
    // (the demo's own random stream for the check timer's restart, not the game's)
    let mut roll = || {
        *rng = rng.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
        (*rng >> 9) as f32 / (1u32 << 23) as f32
    };
    let hostile_wise = rig.hostile.filter(|h| h.4).map(|h| h.0);
    let units: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).collect();
    let mut spheres = vec![];
    for (e, mut g, tr, mine) in &mut mines {
        if !is_mine(&g) {
            continue;
        }
        if due.contains(&e) {
            g.fuse = 0.0;
            if log {
                println!("t {now:.2}: the mine {e} is shot: it goes off");
            }
            continue;
        }
        let (centre, r) = kits.0[g.kind].bounds;
        spheres.push((e, tr.transform_point(centre), r));
        // placed: the owner's team kept with it (0x147570 writes it at +0x1c0)
        let Some(mut m) = mine else {
            let team = units.iter().find(|u| u.character == g.thrower).map_or(0, |u| u.team);
            commands.entity(e).insert(Mine { check: 0.0, team });
            continue;
        };
        m.check += dt;
        if m.check <= CHECK_EVERY {
            continue;
        }
        m.check = roll() * CHECK_JITTER;
        let def = &kits.0[g.kind].def;
        let radius = if def.proximity_radius > 0.0 { def.proximity_radius } else { DEFAULT_RADIUS };
        let at = tr.translation;
        let go = check(at, radius, g.thrower, m.team, &units, |u| hostile_wise == Some(u.character) && u.team != 0);
        if log {
            // the nearest target and friend (3D to the body point, and across)
            let nearest = |friends: bool| units.iter().filter(|u| !u.dead && friend(u, g.thrower, m.team) == friends)
                .map(|u| (body_point(u).distance(at), Vec2::new(u.position.x - at.x, u.position.z - at.z).length()))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((d, across)) = nearest(false).filter(|t| t.0 < radius + 1.5) {
                let (f, f_across) = nearest(true).unwrap_or((f32::MAX, f32::MAX));
                println!("t {now:.2}: mine {e} check: target {d:.2} m ({across:.2} m across), nearest friend {f:.2} m ({f_across:.2} m across): {}",
                         if go { "goes off" } else { "stays" });
            }
        }
        if go {
            g.fuse = 0.0;
        }
    }
    targets.targets = spheres;
}

/// The test hooks (see `reset`): the hostile and the friend held where they're put round the
/// first mine, out of the squad AI.
fn test_rig(mut player: ResMut<Player>, mut squad: ResMut<Squad>, kits: Option<Res<GrenadeKits>>, mut rig: ResMut<TestRig>,
            mines: Query<(Entity, &Grenade, &Transform)>) {
    if rig.hostile.is_none() && rig.friend.is_none() && rig.shoot.is_none() {
        return;
    }
    let Some(kits) = kits else { return };
    let now = player.sim_time;
    if rig.first.is_none() {
        let first = mines.iter().find(|(_, g, _)| g.landed && kits.0.get(g.kind).is_some_and(|k| k.def.function_type == PROXIMITY_EXPLOSIVE));
        if let Some((e, g, tr)) = first {
            let from = std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.character == g.thrower).map_or(Vec3::ZERO, |u| u.position);
            let dir = Vec3::new(tr.translation.x - from.x, 0.0, tr.translation.z - from.z).normalize_or(Vec3::X);
            rig.first = Some((e, tr.translation, dir, now));
            if std::env::var("BF_SENTRY_LOG").is_ok() {
                println!("t {now:.2}: first mine {e} down at {:.2}", tr.translation);
            }
        }
    }
    let gone = rig.first.is_some_and(|(e, ..)| !mines.contains(e));
    // (where it lies now: one set down on the run slides a little after it's down)
    if let Some((e, at, ..)) = rig.first.as_mut() {
        if let Ok((_, _, tr)) = mines.get(*e) {
            *at = tr.translation;
        }
    }
    let hostile = rig.hostile;
    let friend = rig.friend;
    let first = rig.first;
    let mut held_at = rig.held_at;
    for m in squad.0.iter_mut() {
        let goal = if let Some((c, from, to, speed, _)) = hostile.filter(|h| h.0 == m.character) {
            m.team = 1;
            m.test_hold = true;
            first.and_then(|(_, at, dir, down)| {
                let d = match held_at {
                    Some(d) => d,
                    None => (from - speed * (now - down - RIG_WAIT).max(0.0)).max(to),
                };
                if gone && held_at.is_none() {
                    held_at = Some(d);
                    println!("{} held at {d:.2} m from where the mine was", CHARACTERS[c]);
                }
                Some((at + dir * d, -dir))
            })
        } else if let Some((_, dist, until)) = friend.filter(|f| f.0 == m.character) {
            m.test_hold = first.is_some_and(|(_, _, _, down)| now - down < until);
            first.filter(|_| m.test_hold).map(|(_, at, dir, _)| {
                let side = Vec3::new(-dir.z, 0.0, dir.x);
                (at + side * dist, -side)
            })
        } else {
            None
        };
        if m.test_hold {
            m.move_input = Vec2::ZERO;
            m.sprint = false;
            m.aim = false;
        }
        // stood there (after a blast's knock-down it's left alone)
        if let Some((p, face)) = goal.filter(|_| m.knock.is_none() && !m.dead) {
            m.position.x = p.x;
            m.position.z = p.z;
            m.position.y = floor_y(p.x, p.z, p.y + 2.0) - GROUND;
            m.prev_xz = Vec2::new(p.x, p.z);
            m.yaw = (-face.x).atan2(-face.z);
        }
    }
    rig.held_at = held_at;
    // the player aims at the mine and fires: the crosshair ray turned onto its middle (a few
    // steps, as the camera's place depends on where it looks)
    if let (Some(after), Some((e, at, _, down))) = (rig.shoot, first) {
        if now - down >= after && !gone {
            let target = mines.get(e).map_or(at, |(_, g, tr)| tr.transform_point(kits.0[g.kind].bounds.0));
            for _ in 0..4 {
                let (cam, ray) = aim_ray(&player);
                let want = (target - cam).normalize_or(ray);
                let yaw = |v: Vec3| (-v.x).atan2(-v.z);
                player.cam_yaw += wrap_angle(yaw(want) - yaw(ray));
                player.cam_pitch = (player.cam_pitch + want.y.clamp(-1.0, 1.0).asin() - ray.y.clamp(-1.0, 1.0).asin()).clamp(-1.2, 0.5);
            }
            // (from the hip, as take10's burst: the camera stays behind)
            player.aim = false;
            player.fire = true;
        }
    }
}
