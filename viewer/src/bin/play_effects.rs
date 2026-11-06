//! Effect types spawned in the world (issue #373): the effect object's own work beyond its
//! particles, from default.xbe's effect object and its effect type's records, checked against
//! the barrels' and the rack's takes (todo/85-interactive-scenery, #366's takes).
//!
//! An effect type (objecttypes `<effect>`, `bf/character.rs` `EffectType`) lists what an object of
//! it does where it's started: its ALE effects and light effect, its sounds, and two lists the
//! effect object's tick (0x21b7a0) works through each game frame it runs, after its particles:
//!  - **damage areas** (`<h_fb0a5f1d><Damage ..>`, parser FUN_00191170, the object's own damage
//!    code FUN_0021ade0): `AreaDamage`, below;
//!  - **shakes** (`<shake-effects><shake ..>`, parser FUN_00191470, FUN_0021b220 and FUN_0021b620):
//!    `EffectShake`, below. Its `<rumble-effects>` (FUN_001912f0, FUN_0021b410: the pad's motors)
//!    aren't played: the demo has no pad vibration.
//!
//! Anything that starts an effect type asks for it with an `EffectSpawn` pushed on
//! `EffectSpawns`: the breaking scenery for a debris list's effect (the barrels', the rack's:
//! play_scenery.rs, which owned the damage areas until #373 generalised them), a grenade's blast
//! for the damage of its explosion's effect type (a Molotov's cloud, #78: `fly_grenades`), the
//! test hook BF_TEST_EFFECT, and the tickets to come (the Supermutant's pound, #212, the Molotov
//! Mutant's burst, #211). `start_effects` makes the look and sounds (unless the caller did) and
//! sets the areas and shakes going; `area_damage` and `shake_view` run them.
//!
//! **Damage areas** (FUN_0021ade0, read in full). Per frame, for each record while h_f724cb8c <
//! the effect's age <= h_f724cb8c + `duration`: every target within `range` of the effect's place
//! (a grid query, then 3D, to the target's place: no ray, so no line of sight and no cover) whose
//! factor for the record's Type is above 0 and whose hit points are above 0 takes amount x dt,
//! x (1 - d / range) for falloff 1 (DFALL_LINEAR), (1 - d / range)^2 x (amount x dt)^2 for 2
//! (DFALL_EXPONENTIAL: squared value and all), in full for 0 (NONE) and 3 (HALF_LIFE). It goes
//! to the target's damage handler (vtable slot 0, 0x125600) as a record with no attacker and no
//! body, so the target's difficulty step, skill and modifiers and its factor for the damage type
//! all apply (play_damage.rs `Hit`), and the objects (breakables) in range take it as the same
//! (`Scenery::area_at_work`: the rack's explosion sets off the barrels around it). The record
//! carries h_ed582b3c, the area's push, scaled by ((1 - w) x 0.4 + 0.6) (w the world's load,
//! FUN_000d9090), as the factor of a kill's shove (#366: `area_damage`'s `death_shoves`: a
//! body-less request thrown from the effect's place; a Type-3 kill is vaporised instead). So
//! the Molotov's cloud (h_1a256542: 78 FLAME a second for 1 s within 7 m, falloff 1, push 1)
//! and the pound (h_ece33110: 50 POWERBLADE a second for 1 s within 7.5 m, falloff 0, push 2.5)
//! need nothing of their own here. The player hurt by one has the red screen tint of a blast.
//!
//! **Shakes** (FUN_0021b220 then FUN_0021b620, from the same tick): for each record while its
//! start < the effect's age <= start + `duration`, every character within `range` of the
//! effect's place (the same point as the damage's) that has a camera following it gets
//! FUN_0006bdd0 with the amplitude lerp(h_f944a10f, h_f6d585b1, (age - start) / duration), x 1 - d /
//! range for falloff 1 and (1 - d / range)^2 x amplitude^2 for 2, no hold and no fall: the
//! camera's one-frame amplitude, set again each frame (`CameraShake::hold_each_frame`,
//! play_shake.rs). Only the controlled character's camera exists here. The pound's shake: 0.7
//! falling to 0 over 2 s, from 1 s on, within 10 m, falloff 2 (so 0.49 at the pound at 1 s).
//!
//! The effect object runs from its start until its last ALE effect has finished (FUN_0021a850
//! empties a slot when its ALE instance answers true to its vtable +0x2c with 4, read as
//! "finished", and the tick ends the object when all eight slots are empty, 0x21bc34), so a
//! damage area or a shake outlasting its effects' particles is cut short in the game; the demo
//! runs each record its whole length, as the barrels' and the rack's always did. The line
//! `BF_DAMAGE_LOG` prints as an effect starts says when a record is longer than its ALE effects
//! (the cloud's run 5.77 s and the pound's 10.93 s, `Compiled::duration`, an estimate: neither).
//!
//! **When**: the takes' first HP loss from a barrel or a rack comes about 0.1 s after the effect is
//! spawned (AREA_DELAY: takes 01-04 and 07, a measurement of the break chain, the same 0.1 s as a
//! grenade's blast to its damage), and the demo gives every effect spawn the same wait before the
//! effect's clock starts, its damage and shake windows counted from then. It's unmeasured for a
//! spawn that isn't a barrel's or a rack's (no take shows a Molotov's cloud or a pound).
//!
//! Test hook: `BF_TEST_EFFECT=<s>,<effect type>,<x>,<z>[,<y>][;...]` starts the effect type (an
//! `h_xxxxxxxx` hash, or a name) at sim time `s` at (x, z), on the floor there or at world height
//! `y`, with everything a spawned effect has (look, sounds, damage areas, shakes). Logs:
//! `BF_DAMAGE_LOG=1` prints each effect started and each damage tick (the time, the effect, the
//! record's age, the distance and falloff share, the victim, what it takes after every factor,
//! its hit points left) and each area's end with its victims' totals; `BF_COMBAT_LOG=1` the ticks
//! alone (its older form), `BF_SCENERY_LOG=1` the area's end, and `BF_SHAKE_LOG=1` the shake the
//! effects hold on the camera, four times a second.

use super::*;
use super::grenade::{ScreenTint, BLAST_HEARD, BLAST_QUIET};
use bf_viewer::bf::character::{AreaDamage, EffectShake};
use bf_viewer::bf::hash::h;

/// An effect's clock starts this long after it is spawned: the takes' first HP loss is ~0.1 s
/// after the object's hitpoints reach 0, for the barrel (takes 01-04: 2.04 -> 2.13-2.14 s) and
/// the rack alike (take07). Measured on those two; the same as a grenade's DAMAGE_DELAY, and
/// assumed for every other spawn (see the notes: unmeasured).
const AREA_DELAY: f32 = 0.1;
/// A character's point that a damage area (and a shake) measures to (m above the feet): the
/// barrel's range of 3 m from its centre (0.435 m up) reached Tex 2.7 m away along the ground and
/// not 2.9 m (takes 04 / 05), so a point 1.2-1.74 m up. The middle of that: a fit. The game
/// measures to the character's position record (vtable +0x40: the hips, #366).
const CENTRE_UP: f32 = 1.45;
/// How often BF_SHAKE_LOG reports the shake held on the camera (s).
const SHAKE_LOG_EVERY: f32 = 0.25;

/// A request to start an effect type where something went off. `start_effects` takes them each
/// frame.
#[derive(Clone, Debug)]
pub struct EffectSpawn {
    /// the effect type (an objecttypes `<effect>`'s base name)
    pub kind: u32,
    /// the effect's place: its ALE effects play here and its damage areas and shakes measure
    /// from it (a barrel's: its archetype's centre, `Breakable::centre`)
    pub at: Vec3,
    /// where a kill's shove goes from: the effect object's own place in the game (#366: the
    /// barrel's origin, not its centre); `at` for a spawn that has no other
    pub origin: Vec3,
    /// the effect's turn (a barrel's: its placement's)
    pub rotation: Quat,
    /// the breakable object (the scenery's list index) it came from, which its own areas leave
    /// alone: the barrel that broke
    pub source: Option<usize>,
    /// whether to play the ALE effects, the light effect and the sounds, and to throw the loose
    /// bodies about (pickups, debris: `pickups::Blasts`); false for a caller that does those
    /// itself (a grenade's blast, whose volume, ground and push it has worked out)
    pub look: bool,
}

impl EffectSpawn {
    /// Effect type `kind` started at `at` with all it has: its look, its sounds, its damage areas
    /// and its shakes, unturned, from no breakable object.
    pub fn new(kind: u32, at: Vec3) -> Self {
        EffectSpawn { kind, at, origin: at, rotation: Quat::IDENTITY, source: None, look: true }
    }

    /// The same, but only its damage areas and shakes: the caller plays the look and sounds and
    /// throws the loose bodies (`look`).
    pub fn without_look(mut self) -> Self {
        self.look = false;
        self
    }
}

/// The effects to start this frame (`EffectSpawn`): systems that spawn effect types push here
/// before `start_effects` runs (`break_scenery` does, in the same frame).
#[derive(Resource, Default)]
pub struct EffectSpawns(pub Vec<EffectSpawn>);

/// An effect record's clock: it waits `wait` s after the effect starts, then counts its age.
#[derive(Clone, Copy, Debug)]
struct Clock {
    wait: f32,
    age: f32,
}

impl Clock {
    /// Runs the clock `dt` s. The stretch of its age this frame covers, `(from, to)`, or None
    /// while it still waits; the frame it stops waiting counts only the part after the wait.
    fn step(&mut self, dt: f32) -> Option<(f32, f32)> {
        let before = self.wait;
        self.wait -= dt;
        if self.wait > 0.0 {
            return None;
        }
        let from = self.age;
        self.age += if before > 0.0 { -self.wait } else { dt };
        Some((from, self.age))
    }
}

/// A damage area at work (see the module notes): one `<Damage>` record of an effect started.
struct Area {
    /// the effect type, for the log
    effect: u32,
    /// the breakable it came from (not hurt by it)
    source: Option<usize>,
    /// where it is: what the range is measured from
    at: Vec3,
    /// where a kill's shove goes from (`EffectSpawn::origin`; #366's take01 and take02: the
    /// records' directions are the victim's root body's place less this, to three digits)
    origin: Vec3,
    /// the record (`EffectType::damage`)
    damage: AreaDamage,
    /// seconds until it starts (AREA_DELAY and the record's h_f724cb8c), then its age since it
    /// started
    clock: Clock,
    /// per character it reached: (who: a squad member, or a test map NPC by its number, #128;
    /// its age when first and last hurting them, HP)
    victims: Vec<(Who, f32, f32, f32)>,
}

/// A shake at work: one `<shake>` record of an effect started.
struct ShakeRun {
    /// the effect type, for the log
    effect: u32,
    /// where the effect is: what the range is measured from
    at: Vec3,
    /// the record (`EffectType::shakes`)
    shake: EffectShake,
    /// the effect's own clock, from AREA_DELAY: the record's start and duration are on its age
    clock: Clock,
}

/// The effects at work: their damage areas, their shakes and their sounds waiting for their delay.
#[derive(Resource, Default)]
struct EffectsAtWork {
    /// the damage areas running, one per `<Damage>` record of each effect started
    areas: Vec<Area>,
    /// the shakes running, one per `<shake>` record
    shakes: Vec<ShakeRun>,
    /// effect sounds waiting for their delay: (s left, id, where)
    sounds: Vec<(f32, u32, Vec3)>,
    /// BF_TEST_EFFECT's entries started so far
    test_fired: usize,
    /// the sim time the shake last reported to BF_SHAKE_LOG (negative: not yet)
    shake_logged: f32,
}

/// The effects systems on a frame: they run among the scenery's (play_scenery.rs `SceneryStep`),
/// between `break_scenery`, which asks for the effects a breaking object spawns, and
/// `set_off_mines`, which reads the reach of the areas at work, so the barrels' and the rack's
/// areas start on the frame they always did.
pub fn plugin(app: &mut App) {
    app.init_resource::<EffectSpawns>()
        .init_resource::<EffectsAtWork>()
        .add_systems(OnEnter(AppState::Playing), |mut spawns: ResMut<EffectSpawns>, mut work: ResMut<EffectsAtWork>| {
            spawns.0.clear();
            *work = EffectsAtWork { shake_logged: -1.0, ..default() };
        })
        .add_systems(Update, (test_effect, start_effects, area_damage, shake_view).chain()
            .in_set(super::scenery::SceneryStep).after(super::scenery::break_scenery).before(super::scenery::set_off_mines)
            .run_if(in_state(AppState::Playing)));
}

/// A blast sound's volume heard at `listener`: full at the blast, BLAST_QUIET from BLAST_HEARD m
/// (the grenades' choice).
fn blast_volume(at: Vec3, listener: Vec3) -> f32 {
    (1.0 - at.distance(listener) / BLAST_HEARD).clamp(BLAST_QUIET, 1.0)
}

/// Test hook BF_TEST_EFFECT: one entry, an effect type to start at a sim time and place.
#[derive(Clone, Debug, PartialEq)]
struct TestEffect {
    /// the controlled character's sim time (s) it starts at
    time: f32,
    /// the effect type
    kind: u32,
    /// where, on the ground plane
    x: f32,
    z: f32,
    /// the world height; None: the floor there
    y: Option<f32>,
}

/// BF_TEST_EFFECT's entries: `<s>,<effect type>,<x>,<z>[,<y>]` separated by `;`, the effect type an
/// `h_xxxxxxxx` hash or a name (hashed, as the other hooks' names are). An entry that isn't
/// four or five fields with numbers where they belong is skipped.
fn parse_test_effects(v: &str) -> Vec<TestEffect> {
    v.split(';').filter_map(|entry| {
        let f: Vec<&str> = entry.split(',').map(str::trim).collect();
        if !(4..=5).contains(&f.len()) {
            return None;
        }
        let kind = match f[1].strip_prefix("h_") {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None if !f[1].is_empty() => h(f[1]),
            None => return None,
        };
        Some(TestEffect { time: f[0].parse().ok()?, kind, x: f[2].parse().ok()?, z: f[3].parse().ok()?, y: f.get(4).and_then(|y| y.parse().ok()) })
    }).collect()
}

/// Test hook: BF_TEST_EFFECT starts the effect types it lists (see the module notes), at the
/// controlled character's sim time.
fn test_effect(player: Res<Player>, mut spawns: ResMut<EffectSpawns>, mut work: ResMut<EffectsAtWork>, mut entries: Local<Option<Vec<TestEffect>>>) {
    let list = entries.get_or_insert_with(|| std::env::var("BF_TEST_EFFECT").map(|v| parse_test_effects(&v)).unwrap_or_default());
    while let Some(e) = list.get(work.test_fired).filter(|e| e.time <= player.sim_time) {
        work.test_fired += 1;
        let y = e.y.unwrap_or_else(|| floor_y(e.x, e.z, player.position.y + GROUND + player.height.max(0.0) + 2.0, world::Body::Thing));
        let at = Vec3::new(e.x, y, e.z);
        if log_damage() {
            println!("t {:.2}: BF_TEST_EFFECT starts h_{:08x} at {at:.2}", player.sim_time, e.kind);
        }
        spawns.0.push(EffectSpawn::new(e.kind, at));
    }
}

/// Whether BF_DAMAGE_LOG is on: the effects' own log, each effect started and each damage tick
/// in full (BF_COMBAT_LOG, the damage's, prints the ticks too, without the detail: `area_damage`).
fn log_damage() -> bool {
    std::env::var("BF_DAMAGE_LOG").is_ok()
}

/// Starts the effects asked for (`EffectSpawns`): each one's ALE effects and light effect where it
/// is, in its turn, run once (the barrels' and the grenades' way), its sounds after their delays
/// (those waiting from earlier frames play first, so a sound with no delay plays on the frame
/// after the spawn, as the barrels' always did), a push to the loose bodies about for each damage
/// area that has a push (not when the caller plays its own look), and its damage areas and
/// shakes set going on the effect's clock.
#[allow(clippy::too_many_arguments)]
fn start_effects(mut commands: Commands, time: Res<Time>, mut spawns: ResMut<EffectSpawns>, mut work: ResMut<EffectsAtWork>, mut game: ResMut<GameData>,
                 mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
                 mut player: ResMut<Player>, mut loose: ResMut<super::pickups::Blasts>) {
    let dt = frame_dt(&time);
    let p = &mut *player;
    work.sounds.retain_mut(|(left, id, at)| {
        *left -= dt;
        if *left <= 0.0 {
            p.sound_queue.push((*id, blast_volume(*at, p.position)));
        }
        *left > 0.0
    });
    let log = log_damage();
    for s in std::mem::take(&mut spawns.0) {
        let Some(fx) = game.0.effect_type_defs.get(&s.kind).cloned() else {
            println!("effect h_{:08x}: no such effect type", s.kind);
            continue;
        };
        // the longest of its ALE effects (the effect object's life, the notes), for the log
        let mut runs = 0.0f32;
        let listed = fx.effects.iter().filter(|&&e| e != 0).count();
        if s.look {
            if let Some(ale) = ale.as_deref_mut() {
                for (k, &e) in fx.effects.iter().chain([&fx.light]).filter(|&&e| e != 0).enumerate() {
                    if let Some(c) = ale.load_recorded(&mut game.0, &mut images, &mut materials, e) {
                        let life = c.duration();
                        if k < listed {
                            runs = runs.max(life);
                        }
                        let seed = (s.at.x * 977.0 + s.at.z * 131.0) as u32 ^ k as u32;
                        commands.spawn((Transform::from_translation(s.at).with_rotation(s.rotation), Visibility::default(),
                                        bf_viewer::ale_fx::AleEffect::once(c, 0.0, seed), AleExpire(life)));
                    }
                }
            }
            for &(id, delay, _, _) in &fx.sounds {
                work.sounds.push((delay, id, s.at));
            }
        }
        for dmg in &fx.damage {
            work.areas.push(Area { effect: s.kind, source: s.source, at: s.at, origin: s.origin, damage: *dmg,
                                   clock: Clock { wait: AREA_DELAY + dmg.delay, age: 0.0 }, victims: vec![] });
            // loose bodies about (pickups, earlier debris) are thrown by it: the push its damage
            // message carries (h_ed582b3c) is what throws a body, so an area with none (the acid's
            // and gas clouds', push 0) leaves them lying. The barrels' and the rack's are all 2 or
            // more. (A caller that plays its own look throws them itself)
            if s.look && dmg.push > 0.0 {
                loose.0.push((s.at, dmg.range));
            }
        }
        for sh in &fx.shakes {
            work.shakes.push(ShakeRun { effect: s.kind, at: s.at, shake: *sh, clock: Clock { wait: AREA_DELAY, age: 0.0 } });
        }
        if log {
            let look = if s.look { format!("ALE effects run {runs:.2} s, {} sounds", fx.sounds.len()) } else { "the caller plays its look".to_string() };
            println!("t {:.2}: effect h_{:08x} starts at {:.2}: {} damage areas (amount, type, s, range, falloff) {:?}, {} shakes (start, s, range, falloff, from, to) {:?}, {look}",
                     p.sim_time, s.kind, s.at, fx.damage.len(), fx.damage.iter().map(|d| (d.amount, d.damage_type, d.duration, d.range, d.falloff)).collect::<Vec<_>>(),
                     fx.shakes.len(), fx.shakes.iter().map(|k| (k.start, k.duration, k.range, k.falloff, k.from, k.to)).collect::<Vec<_>>());
            for d in fx.damage.iter().filter(|d| s.look && runs > 0.0 && d.delay + d.duration > runs) {
                println!("  (a damage area runs to {:.2} s, past its ALE effects' {runs:.2} s: the game's effect object would end first)", d.delay + d.duration);
            }
        }
    }
}

/// An area's damage this frame (`value`: amount x the frame's part of its time) at `d` m by its
/// falloff, as FUN_0021ade0 deals it: 1 (DFALL_LINEAR) x (1 - d / range); 2 (DFALL_EXPONENTIAL)
/// (1 - d / range)^2 x value^2 (squared value and all, as the code has it); the others, 0
/// (NONE) and 3 (HALF_LIFE: the rack's), in full. The takes' flat rack damage at 4 and 7 m
/// agrees.
pub(super) fn falloff(damage: &AreaDamage, value: f32, d: f32) -> f32 {
    let k = 1.0 - d / damage.range.max(1e-3);
    match damage.falloff {
        1 => value * k,
        2 => k * k * value * value,
        _ => value,
    }
}

/// The damage areas: from their start for their duration, amount x dt x factor a frame to every
/// character within range with no test of the way between (x its difficulty step,
/// play_damage.rs), and to the breakable objects in range (chain reactions). See the module
/// notes.
#[allow(clippy::too_many_arguments)]
fn area_damage(time: Res<Time>, mut work: ResMut<EffectsAtWork>, mut scenery: ResMut<super::scenery::Scenery>, mut player: ResMut<Player>,
               mut squad: ResMut<Squad>, game: Res<GameData>, mut tint: ResMut<ScreenTint>, difficulty: Res<super::damage::Difficulty>) {
    let dt = frame_dt(&time);
    let now = player.sim_time;
    let combat = std::env::var("BF_COMBAT_LOG").is_ok();
    let detail = log_damage();
    let mut hits = super::scenery::AreaHits::default();
    for a in work.areas.iter_mut() {
        // the part of this frame the area is at work
        let Some((from, to)) = a.clock.step(dt) else { continue };
        let inside = (to.min(a.damage.duration) - from).max(0.0);
        if inside <= 0.0 {
            continue;
        }
        let value = a.damage.amount * inside;
        // the characters in range (FUN_0021ade0 measures each target's place to the effect's,
        // with no ray test; the weapons' blasts, FUN_00224a90, ray-test characters only, and not
        // these)
        for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
            if u.dead {
                continue;
            }
            let middle = u.position + Vec3::Y * (GROUND + u.height + CENTRE_UP);
            let d = middle.distance(a.at);
            if d >= a.damage.range {
                continue;
            }
            // (through the character's handler: its difficulty step, then its factor for the Type,
            // play_damage.rs; FUN_0021ade0 calls the target's vtable slot 0, 0x125600)
            let mut hit = super::damage::Hit::new(falloff(&a.damage, value, d), "area", a.damage.damage_type);
            let amount = hit.on(u, &game.0, &difficulty);
            if amount <= 0.0 {
                continue;
            }
            let first = !a.victims.iter().any(|v| v.0 == u.who());
            if first || amount >= u.health {
                // hurt as by a blast (blood, its hurt line); dies from it as from any hurt (the
                // area's record from FUN_0021ade0 taken as one with no attacker character: not
                // traced)
                let away = Vec3::new(u.position.x - a.at.x, 0.0, u.position.z - a.at.z).normalize_or(Vec3::X);
                hurt(u, &game.0, amount, chatter::Cause::bare(a.damage.damage_type), away * 3.0 + Vec3::Y * 2.0, middle, -1);
                // (a kill's own shoves, #366: FUN_0021ade0's record names no body (+0x20 = 0), its
                // way is from the effect's place (the object's origin) to the victim's point (the
                // root body, the hips: `missile::root_point`) and its push the area's, so the
                // CBT_DEAD handler's hit request is body-less and the up-kick merges into it: one
                // request of magnitude 2 x the share + 1 and factor push + 36, thrown at the
                // all-body cap (take01: the rack's 12.7 m/s, 20 degrees up). A type-3 kill is
                // vaporised and asks for none)
                if u.dead {
                    let root = super::missile::root_point(u);
                    ragdoll::death_shoves(u, ragdoll::Target::None, root, (root - a.origin).normalize_or_zero(), amount, a.damage.damage_type,
                                          ragdoll::area_push(a.damage.push));
                }
                if k == 0 {
                    tint.0 = 0.0;
                }
            } else {
                // (a hit through the handler all the same: its hurt line, FUN_00125ad0)
                chatter::hurt_line(u, chatter::Cause::bare(a.damage.damage_type), amount);
                u.health -= amount;
            }
            if combat || detail {
                let mut line = format!("t {now:.2}: area h_{:08x} at {d:.2} m: {} takes {amount:.2} (x{:.2} the step{}) -> {:.1} / {:.0}", a.effect, unit_name(u),
                                       hit.step, if hit.victim_power != 1.0 { format!(", x{:.2} its power-up", hit.victim_power) } else { String::new() },
                                       u.health, u.max_health);
                if detail {
                    line += &format!(" [age {:.3} (+{:.3}), falloff {} share {:.3}, type {} x{:.3}, push {}{}]", to, to - from, a.damage.falloff,
                                     falloff(&a.damage, 1.0, d), a.damage.damage_type, hit.vulnerability, a.damage.push, if u.dead { ", killed" } else { "" });
                }
                println!("{line}");
            }
            match a.victims.iter_mut().find(|v| v.0 == u.who()) {
                Some(v) => { v.2 = to; v.3 += amount; }
                None => a.victims.push((u.who(), to, to, amount)),
            }
        }
        // the breakable objects in range (to their centres)
        scenery.area_at_work(&a.damage, a.at, value, a.source, &mut hits);
    }
    scenery.deal_area_hits(&game.0, hits, now);
    // areas done
    work.areas.retain(|a| {
        let over = a.clock.wait <= 0.0 && a.clock.age >= a.damage.duration;
        if over && (combat || detail || std::env::var("BF_SCENERY_LOG").is_ok()) {
            println!("t {now:.2}: area h_{:08x} over ({} x {} s, range {}, type {}, falloff {})", a.effect, a.damage.amount, a.damage.duration,
                     a.damage.range, a.damage.damage_type, a.damage.falloff);
            for &(who, first, last, total) in &a.victims {
                println!("  {} hurt from +{first:.2} to +{last:.2} s: {total:.1} HP", who_name(who, &game.0));
            }
        }
        !over
    });
}

/// The amplitude shake `s` holds `d` m from the effect when the effect is `age` s old, if it holds
/// one then: FUN_0021b620's, for a shake with range, within its window `start` < age <= `start` +
/// `duration` and its range. The amplitude lerps from `from` to `to` over the duration; falloff 1
/// takes it times 1 - d / range, falloff 2 takes (1 - d / range)^2 x the amplitude squared (as
/// the damage areas' falloff 2 squares its amount), the others leave it.
fn shake_amplitude(s: &EffectShake, age: f32, d: f32) -> Option<f32> {
    if !(s.range > 0.0 && s.start < age && age <= s.start + s.duration && d < s.range) {
        return None;
    }
    let amp = (s.to - s.from) * ((age - s.start) / s.duration) + s.from;
    let k = 1.0 - d / s.range;
    Some(match s.falloff {
        1 => k * amp,
        2 => k * k * amp * amp,
        _ => amp,
    })
}

/// The shakes at work: the strongest of those that reach the controlled character is held on the
/// view this frame (`CameraShake::hold_each_frame`; FUN_0021b620 gives the camera of every
/// character in range FUN_0006bdd0's one-frame amplitude, and the follow camera is the controlled
/// character's). A shake that's run its length is dropped.
fn shake_view(time: Res<Time>, mut work: ResMut<EffectsAtWork>, player: Res<Player>, mut shake: ResMut<super::shake::CameraShake>) {
    let dt = frame_dt(&time);
    let middle = player.position + Vec3::Y * (GROUND + player.height + CENTRE_UP);
    // (the strongest, and which effect and how far, for the log)
    let mut strongest: Option<(f32, u32, f32, f32)> = None;
    for run in work.shakes.iter_mut() {
        let Some((_, age)) = run.clock.step(dt) else { continue };
        let d = middle.distance(run.at);
        if let Some(amp) = shake_amplitude(&run.shake, age, d).filter(|_| !player.dead) {
            if strongest.is_none_or(|s| amp > s.0) {
                strongest = Some((amp, run.effect, d, age));
            }
        }
    }
    work.shakes.retain(|r| r.clock.wait > 0.0 || r.clock.age <= r.shake.start + r.shake.duration);
    shake.hold_each_frame(strongest.map_or(0.0, |s| s.0));
    if let Some((amp, effect, d, age)) = strongest {
        let due = work.shake_logged < 0.0 || player.sim_time - work.shake_logged >= SHAKE_LOG_EVERY;
        if due && std::env::var("BF_SHAKE_LOG").is_ok() {
            work.shake_logged = player.sim_time;
            println!("t {:.2}: shake held {amp:.3} by h_{effect:08x} (age {age:.2} s, {d:.2} m)", player.sim_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Molotov's cloud as the data has it (h_1a256542): 78 FLAME a second for 1 s within 7 m,
    /// falloff 1, push 1.
    fn cloud() -> AreaDamage {
        AreaDamage { amount: 78.0, damage_type: 5, push: 1.0, delay: 0.0, duration: 1.0, range: 7.0, falloff: 1 }
    }

    /// The pound's shake as the data has it (h_ece33110): from 1 s, 2 s long, 10 m, falloff 2,
    /// 0.7 falling to 0.
    fn pound() -> EffectShake {
        EffectShake { start: 1.0, duration: 2.0, range: 10.0, falloff: 2, from: 0.7, to: 0.0 }
    }

    /// FUN_0021ade0's falloffs: linear is x (1 - d / range), exponential squares the share and the
    /// value, none and half-life leave it, and at the range's edge a falloff gives nothing.
    #[test]
    fn the_falloffs_are_the_effect_objects() {
        let linear = cloud();
        assert!((falloff(&linear, 78.0 / 30.0, 0.0) - 2.6).abs() < 1e-5);
        assert!((falloff(&linear, 78.0 / 30.0, 3.5) - 1.3).abs() < 1e-5);
        assert!(falloff(&linear, 78.0 / 30.0, 7.0).abs() < 1e-6);
        let exponential = AreaDamage { falloff: 2, range: 10.0, ..linear };
        assert!((falloff(&exponential, 2.0, 5.0) - 1.0).abs() < 1e-6, "(1 - 0.5)^2 x 2^2");
        for kind in [0, 3] {
            assert_eq!(falloff(&AreaDamage { falloff: kind, ..linear }, 2.6, 6.9), 2.6);
        }
    }

    /// A clock waits, then counts the age: the frame it stops waiting counts the part past the
    /// wait only (the old barrel code's `inside`), and the frames after count whole.
    #[test]
    fn a_clock_counts_from_the_end_of_its_wait() {
        let mut c = Clock { wait: 0.1, age: 0.0 };
        assert_eq!(c.step(0.0667), None);
        let (from, to) = c.step(0.0667).unwrap();
        assert!((from - 0.0).abs() < 1e-6 && (to - 0.0333).abs() < 1e-3, "{from} {to}");
        let (from, to) = c.step(0.0667).unwrap();
        assert!((from - 0.0333).abs() < 1e-3 && (to - 0.1).abs() < 1e-3);
        // a wait that ends exactly on a frame's end starts the age at 0 and counts nothing yet
        let mut c = Clock { wait: 0.0667, age: 0.0 };
        assert_eq!(c.step(0.0667), Some((0.0, 0.0)));
        assert_eq!(c.step(0.0667), Some((0.0, 0.0667)));
    }

    /// The whole second of a damage area deals the amount, however the frames split it: 78 FLAME
    /// at the cloud's centre over 15 fps frames, and over 60 fps ones.
    #[test]
    fn a_second_of_damage_adds_up_to_the_amount() {
        for fps in [15.0f32, 30.0, 60.0] {
            let (dt, damage) = (1.0 / fps, cloud());
            let mut c = Clock { wait: AREA_DELAY, age: 0.0 };
            let mut dealt = 0.0;
            for _ in 0..(3.0 * fps) as usize {
                if let Some((from, to)) = c.step(dt) {
                    let inside = (to.min(damage.duration) - from).max(0.0);
                    dealt += falloff(&damage, damage.amount * inside, 0.0);
                }
            }
            assert!((dealt - 78.0).abs() < 0.01, "{fps} fps: {dealt}");
        }
    }

    /// FUN_0021b620: the pound's shake starts after its first second (nothing at 1.0 s, 0.49 at
    /// the pound just after), falls with the amplitude's square, is cut by the range, and ends
    /// after its three seconds.
    #[test]
    fn the_pounds_shake_ramps_down_over_its_window() {
        let s = pound();
        assert_eq!(shake_amplitude(&s, 1.0, 0.0), None);
        assert!((shake_amplitude(&s, 1.0 + 1e-4, 0.0).unwrap() - 0.49).abs() < 1e-3);
        // halfway through: amplitude 0.35, squared 0.1225; at 5 m a quarter of that
        assert!((shake_amplitude(&s, 2.0, 0.0).unwrap() - 0.1225).abs() < 1e-5);
        assert!((shake_amplitude(&s, 2.0, 5.0).unwrap() - 0.030625).abs() < 1e-5);
        assert_eq!(shake_amplitude(&s, 2.0, 10.0), None);
        assert!(shake_amplitude(&s, 3.0, 0.0).unwrap().abs() < 1e-6);
        assert_eq!(shake_amplitude(&s, 3.01, 0.0), None);
        // linear falloff takes the share once, none leaves the amplitude
        let linear = EffectShake { falloff: 1, ..s };
        assert!((shake_amplitude(&linear, 1.5, 5.0).unwrap() - 0.5 * 0.525).abs() < 1e-5);
        let none = EffectShake { falloff: 0, ..s };
        assert!((shake_amplitude(&none, 1.5, 5.0).unwrap() - 0.525).abs() < 1e-5);
        // a shake with no range is nothing
        assert_eq!(shake_amplitude(&EffectShake { range: 0.0, ..s }, 1.5, 0.0), None);
    }

    /// BF_TEST_EFFECT: the entries, their effect types as hashes or names, an optional height;
    /// a malformed entry is skipped.
    #[test]
    fn the_test_hook_reads_its_entries() {
        let got = parse_test_effects("2.0,h_1a256542,0,-10; 4.5, h_ece33110, 3.5, 4, 1.25 ;bad;1,,0,0;6,pound,1,2");
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], TestEffect { time: 2.0, kind: 0x1a25_6542, x: 0.0, z: -10.0, y: None });
        assert_eq!(got[1], TestEffect { time: 4.5, kind: 0xece3_3110, x: 3.5, z: 4.0, y: Some(1.25) });
        assert_eq!(got[2].kind, h("pound"));
    }
}
