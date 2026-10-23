//! Bolts (issue #136): the shots of bullet-type 0 and 3 (BTYPE_PROJECTILE, PHYSICAL_PROJECTILE)
//! fly as the game's pooled objects and find what they hit on the way, from default.xbe's blast
//! pool: FUN_0022bac0 (the launch), FUN_00225220 (the pool's tick) and FUN_00225440 (a bolt's
//! flight), and the damage handler each hit is dealt through (FUN_00224320, FUN_00224770 ->
//! 0x22d0b0). The guns: the A10 and A50 Bioreactive, the three LZRs, the two cutters, the Ion
//! Beamer, the Jax-iC / iP / iR, the MNR-7 Bio, the three Saryls, and the creatures' Psionic
//! Blast. No definition in the data uses bullet-type 3.
//!
//!  - **The launch** (FUN_0022bac0, from the fire loop's FUN_0022f1d0): at the muzzle (the
//!    shot's origin), along the shot (after the accuracy's turn), at the bullet's speed (+0x1c);
//!    its life (+0x28, and +0x30 its start) the bullet's range in the holder's hands
//!    (FUN_002324c0) over the speed; the damage roll made now (+0x20, FUN_0022c940: Damage
//!    min..max, then the shooter's skill's factor, `skills::roll_factor`), with the shooter
//!    (+0x48) and its team (+0x6c). Its last checked point (+0x70) is the muzzle and its sweep
//!    counter (byte +3) the pool's reset's 3 (FUN_00225000).
//!  - **Each game frame** (a 30 Hz tick, shots::TICK; FUN_00225220): its life runs down by the
//!    frame's time, and once it's out the bolt goes where it is; then FUN_00225440 counts the
//!    sweep counter down. Not yet 0, the bolt moves speed x dt along its way unchecked; at 0 the
//!    counter is set to 3 again and a ray (FUN_000ddd10) is swept from the last checked point
//!    over the way moved since and this frame's step. Nothing met: it moves on, and the last
//!    checked point is where it is. Met: it's set back 0.1 % of the way from the hit toward the
//!    last checked point (HIT_BACK), which is set there too, the body struck is gathered
//!    (FUN_00224320, the hit part's factor asked for) and dealt in the same frame (FUN_00224770),
//!    and it ends: the next frame takes it away (the flag 4 FUN_00225220 reads).
//!  - **What a sweep meets** (its filter, vtable 0x394848 with the mask 0xfffffff8: the bodies of
//!    PID 0, 1 and 2, `Arena::ray_shot`): the terrain and the placed objects, a breakable among
//!    them, but not the level's invisible blockers; a character's body (standing, play.rs's
//!    `ray_body`; knocked down, play_missile.rs's `ray_lying`; a corpse, its ragdoll's bones,
//!    `Ragdoll::first_bone`); a Sentry. Never its shooter: the filter holds the shooter's body
//!    (+0x48's, at its +0x6c).
//!  - **A hit** (the damage handler 0x22d0b0): a character takes the launch roll x the shooter's
//!    skill x the struck hit part's factor (FUN_00125940), x0.7 on its own side, then its own
//!    step and damage-type factor as it lands (play_damage.rs, through play.rs's
//!    `land_player_shot` for the controlled character's bolts and `land_ai_shot` for the rest);
//!    the world plays its material's hit effect, debris and sound with the launch roll as the
//!    record (play.rs `world_hit`) and takes the bullet's hole (play_fx.rs); a breakable object
//!    takes the roll (play_scenery.rs `ObjectHits`); a Sentry goes off (play_sentry.rs); a body on
//!    the ground is shoved and bleeds (`Ragdoll::shove`). A liquid's surface it crosses splashes,
//!    as an instant ray's does (play.rs `projectiles`), and it goes on to the bed.
//!
//! The timing, measured on all eight bolt guns of weapons batch 1 (todo/57-jax-ic/spec.md §4: 7 m
//! at 80 m/s 0.1 s in 16 of 16 hits, 25 m at 100 m/s 0.3 s in 12 of 12; the fire-to-hit sounds
//! 0.21 s apart at 10 m and 80 m/s, 6 frames; the Saryl-45's within 10 m 0.09-0.1 s; the burst in
//! the picture at the shot + 0.2 s at 10 m; the ball one step out a frame after the shot): the
//! sweeps land on game frames 3, 6, 9 ... counted from the frame after the shot's, 0.1 s apart.
//! The code would count the launch frame as the bolt's first tick (the world update runs the
//! objects, where the shot is made, before the pool's tick: 0xd9ae2, then 0xd9b07), a frame
//! sooner; the analyst couldn't settle it from the code and left it to the takes' frame counts
//! (todo/weapons-plan/answers-batch1.md §1), so a bolt's first tick is the frame after its shot's
//! (`launch_bolts` runs after `fly_bolts`). Between sweeps a bolt flies on unchecked, into what it
//! struck (the Jax-iR's ball gone in the wall 0.117-0.183 s after the shot, its burst at 0.2 s:
//! todo/59 take01). Its life ends its sweeps: the Jax-iC's last (tick 21 of its 22.5) reaches 56
//! m of its 60, and a target beyond is never met.
//!
//! Before #136 the demo picked a bolt's hit at its shot (the body or the world on its line then)
//! and landed it on the sweep that would reach it: a target that stepped out of the way was still
//! hit, one that stepped in wasn't.
//!
//! FUN_00225220 also sets the flight effect's user parameter to life / start each frame
//! (FUN_0021c380): no bolt's flight effect varies with it (every curve of the data's 22 has one
//! variant), so it isn't kept. The bolt's flight voice (FUN_0022bac0's h_03dc7b9a pick) isn't
//! played (answers-batch1.md §3: never heard in the takes).
//!
//! `BF_SHOT_LOG=1` prints each bolt's launch (its roll, life and range), each sweep that meets
//! something (what, how far from its last checked point, its tick and age) and its end.

use super::*;
use super::damage::{self, Difficulty};

/// FUN_00225440's sweep counter (byte +3, FUN_00225000's 3): a sweep every this many game frames.
const SWEEP_TICKS: u8 = 3;
/// FUN_00225440 sets a bolt that met something back from the hit point toward its last checked
/// point by this share of the way between them (x 0.001), and checks on from there.
const HIT_BACK: f32 = 0.001;
/// A bolt's life is out once what's left of it is this or less (s): FUN_00225440 ends it at a
/// life of 0 or less, and the demo's sum of frame times lands a hair either side of a life that's
/// a whole number of frames (the LZR-10's 80 / 100 = 0.8 s, 24 of them). The demo's.
const LIFE_SLACK: f32 = 1e-4;

/// A bolt in flight (the pool object FUN_0022bac0 fills; its offsets in the notes).
#[derive(Component)]
pub(super) struct Bolt {
    /// the shot it was fired as: the weapon's effects, decal, ammo-type, damage-type and range
    shot: Shot,
    /// its number (BF_SHOT_LOG)
    id: u32,
    /// who fired it (+0x48), their team (+0x6c), and whether that was the controlled character
    /// (its hits land as the player's shots do: play.rs `land_player_shot`)
    owner: Who,
    team: u8,
    human: bool,
    /// where it is (+4), its way (+0x10, a unit vector) and speed (+0x1c)
    pos: Vec3,
    dir: Vec3,
    speed: f32,
    /// its life left (+0x28, s)
    life: f32,
    /// the sweep counter (+3)
    counter: u8,
    /// the last checked point (+0x70)
    last: Vec3,
    /// the damage roll made at its launch (+0x20, FUN_0022c940) and the shooter's skill's factor
    /// on it
    roll: f32,
    skill: f32,
    /// game time not yet run in ticks (s)
    clock: f32,
    /// game frames flown since its launch (BF_SHOT_LOG)
    ticks: u32,
}

/// What a game frame of a bolt comes to (`Bolt::advance`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Advance {
    /// its life is out: it goes, unmoved
    Out,
    /// it moved on unchecked
    Moved,
    /// its sweep is due: from its last checked point, over this far, this frame's step (the move
    /// it makes if the sweep meets nothing)
    Sweep { from: Vec3, len: f32, step: f32 },
}

impl Bolt {
    /// One game frame of FUN_00225220 -> FUN_00225440 up to the sweep: the life runs down by the
    /// frame (out: it ends where it is), the sweep counter drops; not yet 0 the bolt moves on,
    /// at 0 it's set to SWEEP_TICKS again and the sweep is due over the way moved since the last
    /// checked point and this frame's step.
    fn advance(&mut self) -> Advance {
        self.ticks += 1;
        self.life -= shots::TICK;
        if self.life <= LIFE_SLACK {
            return Advance::Out;
        }
        self.counter -= 1;
        let step = self.speed * shots::TICK;
        if self.counter != 0 {
            self.pos += self.dir * step;
            return Advance::Moved;
        }
        self.counter = SWEEP_TICKS;
        Advance::Sweep { from: self.last, len: self.pos.distance(self.last) + step, step }
    }
}

/// The bolts' shared state: the next number.
#[derive(Resource, Default)]
struct BoltState {
    next: u32,
}

/// This frame's sweeps (from the last checked point, along the bolt's way, as far as what it
/// met): the loose pickups they pass are knocked as by a shot's line (play_pickups.rs `physics`).
#[derive(Resource, Default)]
pub(super) struct Sweeps(pub Vec<(Vec3, Vec3, f32)>);

/// The systems, after the characters have moved and fired (`update_player`): the bolts in flight,
/// then this frame's new ones (a bolt is drawn at the muzzle in its shot's frame and flies from
/// the next); before the grenades (a Sentry a bolt struck goes off this frame, play_sentry.rs's
/// `trip` among them), the scenery (play_scenery.rs, `ObjectHits`), the pickups (`Sweeps`), the
/// hits' blood (play_fx.rs) and the sounds.
pub fn plugin(app: &mut App) {
    app.init_resource::<BoltState>().init_resource::<Sweeps>()
        .add_systems(OnEnter(AppState::Playing), |mut state: ResMut<BoltState>, mut sweeps: ResMut<Sweeps>| {
            *state = BoltState::default();
            sweeps.0.clear();
        })
        .add_systems(Update, (fly_bolts, launch_bolts).chain()
            .after(update_player).before(super::grenade::GrenadeSystems).before(super::pickups::PickupSystems)
            .before(super::fx::FxSystems).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// What a sweep met first: how far along it, and what.
#[derive(Clone, Copy, Debug)]
struct Met {
    t: f32,
    what: What,
}

/// What a sweep can meet (see the module notes).
#[derive(Clone, Copy, Debug)]
enum What {
    /// the world: the struck surface's normal (facing back along the sweep), its world material,
    /// whether it's the terrain, and the breakable object it is, if one
    World { normal: Vec3, material: i64, terrain: bool, breakable: Option<usize> },
    /// a character standing, or knocked down
    Body(Who),
    /// a corpse (its ragdoll)
    Corpse(Who),
    /// a Sentry mine
    Mine(Entity),
}

/// What the world holds on a sweep's line within `len` m of `from` (`What::World`): the map's
/// collision less its blockers (`Arena::ray_shot`, with its world material, `ray_material`'s
/// front-face rule); on the flat test floor its ground, a pillar's box and a breakable the test
/// tools put down (as play.rs's `ray_hit` and play_shots.rs's `hit_normal` meet them), and the
/// pieces of scenery they put on it (the floor's arena). The test floor's pillars count as
/// objects: the demo's.
fn world_met(from: Vec3, dir: Vec3, len: f32) -> Option<Met> {
    let arena = world::arena();
    let shot = arena.as_ref().and_then(|a| a.ray_shot(from, dir, len, None))
        .map(|h| Met { t: h.t, what: What::World { normal: h.normal, material: h.material as i64, terrain: h.terrain, breakable: h.breakable } });
    if arena.as_ref().is_some_and(|a| !a.is_flat()) {
        return shot;
    }
    // the flat test floor: its ground (no arena: the plane at GROUND), its pillars, its placed
    // breakables' boxes
    let mut best = shot;
    let mut take = |m: Met| {
        if best.is_none_or(|b| m.t < b.t) {
            best = Some(m);
        }
    };
    if arena.is_none() && dir.y < -1e-4 {
        let t = (GROUND - from.y) / dir.y;
        if t > 1e-3 && t <= len {
            take(Met { t, what: What::World { normal: Vec3::Y, material: 0, terrain: true, breakable: None } });
        }
    }
    for c in pillars() {
        let (lo, hi) = (c - Vec3::new(0.3, 1.5, 0.3), c + Vec3::new(0.3, 1.5, 0.3));
        let Some(t) = ray_aabb(from, dir, lo, hi).filter(|&t| t <= len) else { continue };
        let at = from + dir * t;
        let d = (at - c) / Vec3::new(0.3, 1.5, 0.3);
        let k = if d.x.abs() >= d.y.abs().max(d.z.abs()) { 0 } else if d.y.abs() >= d.z.abs() { 1 } else { 2 };
        let mut n = Vec3::ZERO;
        n[k] = d[k].signum();
        take(Met { t, what: What::World { normal: n, material: 0, terrain: false, breakable: None } });
    }
    if let Some((t, i)) = super::scenery::ray_placed_hit(from, dir, len) {
        take(Met { t, what: What::World { normal: -dir, material: 0, terrain: false, breakable: Some(i) } });
    }
    best
}

/// Where a ray (from `origin` along unit `dir`) enters the box `lo`..`hi`, if it does ahead of
/// its start (play.rs `ray_hit`'s pillar test).
fn ray_aabb(origin: Vec3, dir: Vec3, lo: Vec3, hi: Vec3) -> Option<f32> {
    let (mut t0, mut t1) = (0.0f32, f32::MAX);
    for k in 0..3 {
        if dir[k].abs() < 1e-6 {
            if origin[k] < lo[k] || origin[k] > hi[k] {
                return None;
            }
        } else {
            let (a, b) = ((lo[k] - origin[k]) / dir[k], (hi[k] - origin[k]) / dir[k]);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
    }
    (t0 <= t1 && t0 > 0.0).then_some(t0)
}

/// What a bolt's sweep meets first (see the module notes): from `from` along `dir` within `len`
/// m, the world (`world_met`), every character but its shooter `owner` and one out of the world
/// (`Player::absent`), and the Sentries.
fn sweep(from: Vec3, dir: Vec3, len: f32, owner: Who, units: &[&Player], mines: &super::sentry::MineTargets) -> Option<Met> {
    let mut best = world_met(from, dir, len);
    let mut take = |m: Met| {
        if best.is_none_or(|b| m.t < b.t) {
            best = Some(m);
        }
    };
    for u in units.iter().filter(|u| u.who() != owner && !u.absent) {
        if u.dead {
            if let Some((t, _)) = u.ragdoll.as_ref().and_then(|r| r.first_bone(from, dir, len)).filter(|&(t, _)| t <= len) {
                take(Met { t, what: What::Corpse(u.who()) });
            }
            continue;
        }
        let t = if knocked_down(u) { super::missile::ray_lying(from, dir, u, len) } else { ray_body(from, dir, u, len) };
        if let Some(t) = t {
            take(Met { t, what: What::Body(u.who()) });
        }
    }
    if let Some((e, t)) = mines.first_on(from, dir, len) {
        take(Met { t, what: What::Mine(e) });
    }
    best
}

/// A character's name for the log (BF_SHOT_LOG).
fn name(u: &Player) -> String {
    format!("{}{}", unit_name(u), u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)))
}

/// The bolts in flight, a 30 Hz game frame at a time (shots::TICK; FUN_00225220 ->
/// FUN_00225440): their lives, moves and sweeps, and what each sweep meets dealt then (see the
/// module notes). A bolt that meets something, or whose life is out, goes this frame.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fly_bolts(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
             mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
             mut bolts: Query<(Entity, &mut Bolt, &mut Transform)>,
             (mut chance, mut holes, mut mines, mut objects, mut sweeps): (ResMut<GameRandom>, ResMut<super::fx::HoleRequests>,
                                                                           ResMut<super::sentry::MineTargets>, ResMut<super::scenery::ObjectHits>, ResMut<Sweeps>),
             difficulty: Res<Difficulty>, test: Option<Res<super::testmap::TestMap>>, camera: Query<&GlobalTransform, With<MainCamera>>) {
    sweeps.0.clear();
    let dt = frame_dt(&time);
    let log = std::env::var("BF_SHOT_LOG").is_ok();
    let listener = camera.iter().next().map(|c| c.translation());
    let instant_kill = test.as_ref().is_some_and(|t| t.instant_kill);
    for (e, mut b, mut tr) in &mut bolts {
        b.clock += dt;
        let mut gone = false;
        while b.clock >= shots::TICK && !gone {
            b.clock -= shots::TICK;
            let (from, len, step) = match b.advance() {
                Advance::Out => {
                    if log {
                        println!("t={:.2} bolt {} ({}) ends at its life's end after {} game frames, {:.1} m from its muzzle", player.sim_time, b.id,
                                 game.0.weapons.get(&b.shot.weapon).map_or("?", |d| d.label.as_str()), b.ticks, b.pos.distance(b.shot.origin));
                    }
                    gone = true;
                    break;
                }
                Advance::Moved => continue,
                Advance::Sweep { from, len, step } => (from, len, step),
            };
            let dir = b.dir;
            let met = {
                let units: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).collect();
                sweep(from, dir, len, b.owner, &units, &mines)
            };
            let reach = met.map_or(len, |m| m.t);
            sweeps.0.push((from, dir, reach));
            // a liquid's surface on the way: its splash there (a shot's rule, play.rs `projectiles`)
            if let Some(ale) = ale.as_deref_mut() {
                if let Some((surface, liquid)) = world::arena().and_then(|a| a.liquid_crossing(from, from + dir * reach)) {
                    let effect = liquid.effects[LIQUID_SHOT];
                    if effect != 0 {
                        spawn_once(&mut commands, &mut game.0, ale, &mut images, &mut materials, effect, Transform::from_translation(surface));
                    }
                }
            }
            let Some(met) = met else {
                b.pos += dir * step;
                b.last = b.pos;
                continue;
            };
            // met: set back from the hit toward the last checked point, which is set there
            let at = from + dir * met.t;
            b.pos = at - (at - from) * HIT_BACK;
            b.last = b.pos;
            let now = player.sim_time;
            let record = b.roll * b.skill;
            let what = deal(&mut commands, &mut game.0, ale.as_deref_mut(), &mut images, &mut materials, &b, met, from, dir,
                            &mut player, &mut squad, &mut chance, &mut holes, &mut mines, &mut objects, &difficulty, listener, instant_kill, record);
            if log {
                // (a level blocker its sweep passed, which a shot met before #136)
                let blocker = world::arena().and_then(|a| a.ray(from, dir, met.t)).filter(|&t| t < met.t - 1e-3)
                    .map_or(String::new(), |t| format!(" (past a level blocker {t:.2} m on)"));
                println!("t={now:.2} bolt {} sweep at game frame {} ({:.2} s): {what} {:.1} m on from its last checked point {:.1} m out{blocker}", b.id, b.ticks,
                         b.ticks as f32 * shots::TICK, met.t, at.distance(b.shot.origin));
            }
            gone = true;
        }
        if gone {
            commands.entity(e).despawn();
            continue;
        }
        // (drawn where it is by now: the time not yet run in ticks carries it on, so it doesn't
        // stand still every other frame at 60 fps)
        tr.translation = b.pos + b.dir * (b.speed * b.clock);
    }
}

/// A sweep's hit `met` dealt (the damage handler 0x22d0b0, see the module notes); what it was,
/// for the log.
#[allow(clippy::too_many_arguments)]
fn deal(commands: &mut Commands, game: &mut Game, ale: Option<&mut bf_viewer::ale_fx::AleAssets>, images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>, b: &Bolt, met: Met, from: Vec3, dir: Vec3, player: &mut Player, squad: &mut Squad,
        chance: &mut GameRandom, holes: &mut super::fx::HoleRequests, mines: &mut super::sentry::MineTargets,
        objects: &mut super::scenery::ObjectHits, difficulty: &Difficulty, listener: Option<Vec3>, instant_kill: bool, record: f32) -> String {
    let s = &b.shot;
    let at = from + dir * met.t;
    let now = player.sim_time;
    match met.what {
        What::World { normal, material, terrain, breakable } => {
            // the bullet's hole, the material's hit effect, debris and sound (the launch roll as
            // the record), a breakable object's damage
            if s.decal != 0 {
                holes.0.push((s.decal, at, normal));
            }
            let hit = Shot { origin: from, dir, dist: met.t, hit: true, normal, ..*s };
            if let Some(ale) = ale {
                world_hit(commands, game, ale, images, materials, &hit, at, chance, listener, player, Some((record, material)));
            }
            if let Some(target) = breakable {
                objects.0.push(super::scenery::ObjectHit { target, amount: record, damage_type: s.damage_type, at, dir });
            }
            format!("hits the world ({}{}, material {material})", if terrain { "the terrain" } else { "an object" },
                    breakable.map_or(String::new(), |b| format!(", breakable {b}")))
        }
        What::Body(who) => {
            let Some((u, controlled)) = unit_mut(player, squad, who) else { return "hits a body that's gone".into() };
            // the struck part (FUN_00224320 asks FUN_00125940), x0.7 on the shooter's own side
            let bone = damage::struck_bone(u, game, from, dir, met.t);
            let hit = damage::Hit::new(b.roll, "roll", s.damage_type).skill(b.skill).by(b.owner, b.team, u).struck(u, game, bone);
            let landing = PendingHit { delay: 0.0, member: 0, hit, dir, local: at - u.position, ammo: s.ammo };
            let lying = knocked_down(u);
            // (the player's shot's rules on anyone but the controlled character: control may have
            // passed while it flew)
            if b.human && !controlled {
                land_player_shot(u, &landing, game, difficulty, now, instant_kill);
            } else {
                land_ai_shot(u, &landing, game, difficulty, now);
            }
            // (one knocked down is shoved too, as a shot's line shoves it, play.rs)
            if let Some(k) = u.knock.as_mut() {
                k.ragdoll.shove(from, dir, met.t);
            }
            format!("hits {}{}", name(u), if lying { " (knocked down)" } else { "" })
        }
        What::Corpse(who) => {
            let Some((u, _)) = unit_mut(player, squad, who) else { return "hits a body that's gone".into() };
            if let Some(hit) = u.ragdoll.as_mut().and_then(|r| r.shove(from, dir, met.t)) {
                u.blood.push(Bleed::shown(hit, dir, s.ammo));
            }
            format!("hits {}'s body", name(u))
        }
        What::Mine(e) => {
            // (at once: the distance is for its log, at no time a metre)
            mines.strike(e, met.t, f32::MAX, now);
            "hits a Sentry".into()
        }
    }
}

/// This frame's bolts launched (FUN_0022bac0): each shooter's new shots of a bolt gun become bolts
/// at the muzzle with their flight effect riding on them (play.rs `start_flight`), their damage
/// roll made now (GameRandom, FUN_0022c940's draw) with the shooter's skill's factor.
#[allow(clippy::too_many_arguments)]
fn launch_bolts(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
                mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut images: ResMut<Assets<Image>>,
                mut materials: ResMut<Assets<StandardMaterial>>, mut chance: ResMut<GameRandom>, mut state: ResMut<BoltState>) {
    let log = std::env::var("BF_SHOT_LOG").is_ok();
    let now = player.sim_time;
    for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        let new: Vec<Shot> = u.shots.iter().filter(|s| s.bolt).copied().collect();
        for s in new {
            let (lo, hi) = shot_rolls(&s);
            let roll = lo + (hi - lo) * chance.unit();
            let skill = skills::roll_factor(u);
            let speed = s.speed.max(1.0);
            let life = s.range / speed;
            state.next += 1;
            let b = Bolt { shot: s, id: state.next, owner: u.who(), team: u.team, human: k == 0, pos: s.origin, dir: s.dir, speed, life,
                           counter: SWEEP_TICKS, last: s.origin, roll, skill, clock: 0.0, ticks: 0 };
            if log {
                println!("t={now:.2} bolt {} launched by {} from {:.2} along {:.3} at {speed} m/s: roll {roll:.2} x skill {skill:.2}, life {life:.3} s (range {:.1} m)",
                         b.id, name(u), s.origin, s.dir, s.range);
            }
            let e = commands.spawn((Transform::from_translation(s.origin).looking_to(s.dir, Vec3::Y), Visibility::default(), b, Name::new("bolt"))).id();
            // (its flight effect's seed: a draw of the shots' random state)
            let seed = chance.stirred();
            if let Some(ale) = ale.as_deref_mut() {
                start_flight(&mut commands, &mut game.0, ale, &mut images, &mut materials, s.flight, e, seed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bolt as `launch_bolts` makes one: `speed` m/s along -z from the origin, its life `range`
    /// / `speed`.
    fn bolt(speed: f32, range: f32) -> Bolt {
        let shot = Shot { weapon: 0, origin: Vec3::ZERO, dir: Vec3::NEG_Z, dist: range, hit: false, speed, flight: 0, hit_fx: 0, damage: [1.0, 2.0],
                          damage_type: 0, ammo: 0, decal: 0, normal: Vec3::Z, falloff: 0, range, bolt: true, body: false };
        Bolt { shot, id: 1, owner: Who::Squad(3), team: 0, human: true, pos: Vec3::ZERO, dir: Vec3::NEG_Z, speed, life: range / speed,
               counter: SWEEP_TICKS, last: Vec3::ZERO, roll: 1.0, skill: 1.0, clock: 0.0, ticks: 0 }
    }

    /// The game frames a bolt that meets nothing sweeps on, each sweep's start (m from the
    /// muzzle) and length, and the frame its life ends on.
    fn flight(b: &mut Bolt) -> (Vec<(u32, f32, f32)>, u32) {
        let mut sweeps = vec![];
        loop {
            match b.advance() {
                Advance::Out => return (sweeps, b.ticks),
                Advance::Moved => {}
                Advance::Sweep { from, len, step } => {
                    sweeps.push((b.ticks, from.length(), len));
                    // (nothing met: on, and checked up to here)
                    b.pos += b.dir * step;
                    b.last = b.pos;
                }
            }
        }
    }

    /// #136: a bolt sweeps on game frames 3, 6, 9 ..., each sweep over the three frames' way since
    /// the last (8 m at the Jax-iC's 80 m/s), and its life (60 / 80 = 0.75 s) ends it on frame 23
    /// with no sweep past frame 21's, which reaches 56 m (FUN_00225220 / FUN_00225440).
    #[test]
    fn sweeps_every_third_frame_until_its_life_is_out() {
        let (sweeps, end) = flight(&mut bolt(80.0, 60.0));
        assert_eq!(end, 23);
        assert_eq!(sweeps.iter().map(|s| s.0).collect::<Vec<_>>(), vec![3, 6, 9, 12, 15, 18, 21]);
        for (k, &(_, from, len)) in sweeps.iter().enumerate() {
            assert!((from - 8.0 * k as f32).abs() < 1e-3, "sweep {k} from {from}");
            assert!((len - 8.0).abs() < 1e-3, "sweep {k} over {len}");
        }
    }

    /// A life that's a whole number of frames ends on its last one (LIFE_SLACK): the LZR-10's 80
    /// m at 100 m/s, 24 frames, its last sweep on frame 21.
    #[test]
    fn a_whole_frames_life_ends_on_its_last_frame() {
        let (sweeps, end) = flight(&mut bolt(100.0, 80.0));
        assert_eq!((end, sweeps.last().map(|s| s.0)), (24, Some(21)));
    }
}
