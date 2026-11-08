//! The effect skills (#200): the skills whose work is an effect type started at a point, and the
//! parts of the game's code they share. Here the Orbital Beam (CSKILL_ORBITAL_BEAM 9: the
//! deathmatch Confed Marine's and the Ferguson Platoon's; the campaign's Sgt. Gonzales, Private
//! Cogburn and Corporal Stout); the Air Strike (6, #203: play_air_strike.rs), the Wrath of Garner
//! (13, #208), the bosses (17, 18, 21) and the Molotov Mutant (20) call the same spawner
//! (todo/npc-skills/answer.md §3.1; viewer/docs/code-map/skills-and-stamina.md, "Skills and
//! stamina"). Checked against the 18 takes in todo/200-effect-skills (spec.md §1, §2).
//!
//!  - **The spawner** (FUN_0013e410, which a skill's toggle, update or animation event calls): the
//!    skill's target point first (its vtable +0x134, into skill+0xc..+0x14: `Skill::point`), a
//!    turn about x by the record's `orientation-pitch` (FUN_0013e800) and about y by a random
//!    angle (FUN_0001fa20), then `count` times the effect type of the skill's `<effect-data>`
//!    (`EffectData`, FUN_0013de50's copy) at the point plus a random offset within +-the
//!    scatter on x and z (FUN_0013e740), each made by FUN_0013e600: the effect object by its type
//!    (FUN_000da800), its transform, its user parameter (FUN_0021c380: 1 - a random share x
//!    skill+0x30, which is the re-roll flag as 0 or 1) and its owner, the skill's character
//!    (+0xcc, and its player record +0xd4). A skill asks for it by setting `Skill::fire`
//!    (`fire_effects`, this module's system, carries it out the same frame); the effect is started
//!    as any effect type is (play_effects.rs `EffectSpawn::of_skill`): its look, sound and damage
//!    areas, which hurt everyone in range, the owner and his team too (FUN_0021ade0 tests no
//!    team), their hits the owner's (the damage record's +0x14 is the effect's +0xcc).
//!  - **The target point** (0x13df70, the vtable +0x134 of the Orbital Beam, the Air Strike and the
//!    Wrath): for a character a player controls (the world's +0xbec record, +0x24 bit 1), the
//!    reticle's lock's position (`[view+0xf0]` vtable +0x30, its vtable +0x40 record's +0x24: the
//!    root node, the hips: `missile::root_point`); with no lock a ray from the view's camera toward
//!    the reticle's centre-ray point (`Reticle::point`) over the record's reach (`target_point`).
//!    For anyone else it's left as it was: the AI's goal writes it (GOAL_TOGGLE_COMBAT_SKILL's
//!    start 0x36e80: the goal's target's position), which the demo's goal doesn't do yet
//!    (play_orders.rs refuses an attack skill's toggle: `SkillKind::attack_skill`). The takes:
//!    locked, the lock target's feet + 1.19 m (take01, 05, 08: Tex's hips); no lock, the floor 23.2
//!    and 28.5 m ahead (take02, take10) and the wall 4.9 m ahead (take03), the reticle's own point;
//!    a teammate under the crosshair isn't locked, so the ray meets his body (take11, take13) or,
//!    off its underside, the floor before him (take12, by the second ray).
//!  - **The Orbital Beam** (`OrbitalBeam`; constructor FUN_0013e8a0, vtable 0x39cae0, toggle
//!    FUN_0013e910): with stamina >= the type's stamina-cost the press calls the spawner, takes the
//!    cost at once (FUN_001258d0: take01 100 -> 45.07-45.16, the rest a frame's refill) and holds
//!    the HUD's skill name 0.2 s (`[view+0xfc]+0x74`: `Skill::name_hold`, play_hud.rs); under it,
//!    nothing (take04: 53.2-53.6 and 21.6 refused, no name). It never switches on: no on state, no
//!    start or end sound, no drain, no motion (the toggle returns 0; skill+0x1c stays 0 in every
//!    take), so the stamina refills at its recovery all the while (2.3 a second: take01 +2.32).
//!    Its effect h_ff8e79d6 (common's objecttypes line 565): the ALE effects `exp_ionbeam` (a
//!    `godray.tga` column 55 m up its emitter), `exp_ionbeam_hit` and `exp_ionbeam_pre` (glows and
//!    a shock), the sound h_f76d4757, and two damage areas, 300 LASER a second for 0.1 s within
//!    5 m, then 15 a second for 1 s within 15 m from 2 s, no falloff: ~30 then 15, 43.4-44.3 in
//!    the takes (spec §2.4). The areas reach anyone's root node within their range of the point
//!    (play_effects.rs `Rules::Game`), the owner too: at a wall 4.9 m ahead (take03) both reach him.
//!    The Confed Marine's LASER x0.6 is his shield's charge rate, and with shield-type 0 he has no
//!    shield: x1.0 (answers.md Q10; `CharacterType::vulnerability` reads the combat-target's own
//!    list, not the shield's).
//!
//! Open (todo/200-effect-skills/answers.md): **the 15.0 m edge**, the owner of take01 and take16
//! 14.98 m (horizontally) from the point was not hurt, which the code's test would hurt unless his
//! root stood 2 cm or more further back in his idle than his position (the demo's Marine is hurt
//! there, by millimetres); and that the first ray starts at **the camera** (`[view+0x10]`'s +0x48
//! object: read as the camera's eye, inferred, Q4).
//!
//! Test hooks: the controlled character of a type with the Orbital Beam (on the test map
//! `BF_TEST_SQUAD="3:confed marine@sdm_ferg"`, or `BF_TEST_SQUAD_SLOT="<s>,me,confed
//! marine@sdm_ferg"` on any map of a test session) fires it with `BF_TEST_SKILL=<s>` (V);
//! `BF_TEST_GIVE_SKILL=<skill-set>[,<cost>[,<recovery>]]` gives the character controlled at a map's
//! start that skill class instead of its own (9, the Orbital Beam; the cost and recovery the dm
//! Confed Marine's 55 and 2.3 unless given: `9,99,0` is the campaign allies'; 6, the Air Strike,
//! the Militia's 20 and 1), so a squad member fires it on any map; `BF_SKILL_LOG=1` prints each
//! toggle, the point and how it was found (the lock, the ray's hit, the second ray) and each effect
//! started.

use super::*;
use super::effects::{EffectSpawn, EffectSpawns};
use super::skills::{Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{CombatSkills, EffectData, SkillData};

/// The Orbital Beam's skill-set (CSKILL_ORBITAL_BEAM, the name table 0x3bd814; the factory
/// FUN_0012e590 makes it with FUN_0013e8a0).
pub const CSKILL_ORBITAL_BEAM: i64 = 9;
/// Its name (its vtable +0x4c): "Orbital Beam", h_ff757f61 in common/stringtable/string-table-en.xml.
const NAME_ORBITAL_BEAM: u32 = 0xFF75_7F61;
/// How long its press holds the HUD's skill name at full (s): FUN_0013e910 writes 0x3e4ccccd, 0.2,
/// to the name widget's timer (`[view+0xfc]+0x74`, the widget at hud+0xfc, FUN_000c72b0), which
/// keeps the alpha 1 while it runs and then lets it fall 3.0 a second (0xc7300). take02: the name
/// at full 2.683-2.867 s, falling, gone by 2.983 (60 fps video): 0.30 s seen.
const NAME_HOLD: f32 = 0.2;
/// The second ray's length against the reach (0x3a5260, 1.25, at 0x13e306).
const SECOND_RAY: f32 = 1.25;

/// The Orbital Beam's cost and recovery that BF_TEST_GIVE_SKILL gives by default: the deathmatch
/// Confed Marine's `<combat-skill skill-set="9" h_105c13aa="55" h_f2a2beb9="2.3">`
/// (objecttypes-dm.xml's h_048b8580 and objecttypes-sdm_ferg.xml's h_fe5ef6c4 alike).
const MARINE_COST: f32 = 55.0;
const MARINE_RECOVERY: f32 = 2.3;

/// The skill classes a type of its own (play.rs `Player::own_type`: none of the squad's four) is
/// given, by skill-set, from its type's `<combat-skill>` (play.rs `spawn_unit`): the ones the demo
/// has outside the squad's four (whose classes stay the squad's, the squad menu's rule, #302): the
/// Orbital Beam, and the Air Strike (play_air_strike.rs: the Militia's).
pub fn own_type_skill(data: Option<&SkillData>) -> Option<&SkillData> {
    data.filter(|d| matches!(d.skill_set, CSKILL_ORBITAL_BEAM | air_strike::CSKILL_AIR_STRIKE))
}

/// The cost and recovery BF_TEST_GIVE_SKILL gives a skill class when its value names none: those of
/// the deathmatch type that has the class (the Militia's 20 and 1 for the Air Strike; the Confed
/// Marine's 55 and 2.3 for the Orbital Beam and any other set).
fn give_defaults(set: i64) -> (f32, f32) {
    match set {
        air_strike::CSKILL_AIR_STRIKE => (air_strike::MILITIA_COST, air_strike::MILITIA_RECOVERY),
        _ => (MARINE_COST, MARINE_RECOVERY),
    }
}

/// BF_TEST_GIVE_SKILL's value: `<skill-set>[,<cost>[,<recovery>]]`, the cost and recovery those of the
/// type that has the class when not given (`give_defaults`). None for a value that doesn't start with
/// a number.
fn parse_give(v: &str) -> Option<(i64, f32, f32)> {
    let mut it = v.split(',').map(str::trim);
    let set = it.next()?.parse::<i64>().ok()?;
    let (cost, recovery) = give_defaults(set);
    let cost = it.next().and_then(|x| x.parse::<f32>().ok()).unwrap_or(cost);
    let recovery = it.next().and_then(|x| x.parse::<f32>().ok()).unwrap_or(recovery);
    Some((set, cost, recovery))
}

/// Whether BF_TEST_GIVE_SKILL has been carried out on this map.
#[derive(Resource, Default)]
struct GiveSkill(bool);

/// Test hook BF_TEST_GIVE_SKILL (see the module notes): the controlled character, once loaded on
/// a map, takes that skill class with that cost and recovery (its max stamina kept, the stamina
/// update on, no start or end sound: the Confed Marine's type has none), as its type's spawn would
/// make it (`Skill::new`). A class the demo hasn't is said and left.
fn test_give_skill(mut player: ResMut<Player>, game: Res<GameData>, mut done: ResMut<GiveSkill>) {
    if done.0 || player.loaded.is_none() {
        return;
    }
    done.0 = true;
    let Some((set, cost, recovery)) = std::env::var("BF_TEST_GIVE_SKILL").ok().and_then(|v| parse_give(&v)) else { return };
    let data = SkillData { skill_set: set, max_stamina: player.max_stamina, cost, recovery, stamina_update: true, ..Default::default() };
    let skill = skills::Skill::new(Some(&data), &game.0.combat_skills);
    if skill.kind.is_none() {
        println!("BF_TEST_GIVE_SKILL: no skill class {set} in the demo: {} keeps its own", unit_name(&player));
        return;
    }
    player.skill = skill;
    println!("BF_TEST_GIVE_SKILL: {} has skill-set {set} (cost {cost}, recovery {recovery})", unit_name(&player));
}

/// The skill object of an effect skill's skill-set, with game-options' `<combat-skills>`
/// (`Skill::new`, FUN_0012e590's jump table): the Orbital Beam's and the Air Strike's
/// (play_air_strike.rs); None for any other.
pub fn kind(skill_set: i64, settings: &CombatSkills) -> Option<Box<dyn SkillKind>> {
    match skill_set {
        CSKILL_ORBITAL_BEAM => Some(Box::new(OrbitalBeam { data: settings.orbital_beam })),
        air_strike::CSKILL_AIR_STRIKE => Some(Box::new(air_strike::AirStrike::new(settings.air_strike))),
        _ => None,
    }
}

/// The Orbital Beam (CSKILL_ORBITAL_BEAM; constructor FUN_0013e8a0, vtable 0x39cae0): "the White
/// button calls a laser from the sky down on the crosshair's target". Its slots past the base's:
/// +4 the toggle FUN_0013e910, +0x4c its name, +0x10c and +0x118 true (0x213690: an attack skill,
/// fired once by the AI's goal), +0x134 0x13df70 (the target point). See the module notes.
struct OrbitalBeam {
    /// its `<effect-data>` (FUN_0013de50 copies the settings' +0x3b8 into skill+0x20..+0x38)
    data: EffectData,
}

impl SkillKind for OrbitalBeam {
    fn name(&self) -> u32 {
        NAME_ORBITAL_BEAM
    }

    /// FUN_0013e910: stamina (char+0x490) >= the cost (skill+4, the type's stamina-cost): the
    /// spawner (FUN_0013e410(0): `Skill::fire`), the cost off at once (FUN_001258d0, which clamps
    /// to 0..max) and the name's hold; return 0 either way, never activating.
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        if u.stamina < cost {
            if out.log {
                println!("t={:.2} {}: orbital beam refused: stamina {:.2} under its cost {cost}", u.sim_time, unit_name(u), u.stamina);
            }
            return Toggle::Refused;
        }
        u.skill.fire = true;
        u.stamina = (u.stamina - cost).clamp(0.0, u.max_stamina);
        u.skill.name_hold = NAME_HOLD;
        if out.log {
            println!("t={:.2} {}: orbital beam: stamina -{cost} -> {:.2}, its effect h_{:08x} asked for", u.sim_time, unit_name(u), u.stamina,
                     self.data.effect);
        }
        Toggle::Stays
    }

    fn effect_data(&self) -> Option<&EffectData> {
        Some(&self.data)
    }

    /// (+0x10c, 0x213690. Its +0x118, also 0x213690, has GOAL_TOGGLE_COMBAT_SKILL's update
    /// FUN_00036f60 send the toggle once and be done, the skill never switching: the demo's goal
    /// refuses an attack skill before that, `attack_skill`)
    fn attack_skill(&self) -> bool {
        true
    }
}

/// The systems: the spawner after the skills (`skills::update_skills`, whose toggles and updates
/// ask for it), before the effects start (play_effects.rs `start_effects`, in the scenery's step),
/// so an effect asked for at a press starts in that frame; and the test hook BF_TEST_GIVE_SKILL,
/// once a map, before the skills' frame.
pub fn plugin(app: &mut App) {
    app.init_resource::<GiveSkill>()
        .add_systems(OnEnter(AppState::Playing), |mut done: ResMut<GiveSkill>| done.0 = false)
        .add_systems(Update, (test_give_skill.before(skills::update_skills),
                              fire_effects.after(skills::update_skills).before(super::scenery::SceneryStep))
            .run_if(in_state(AppState::Playing)));
}

/// What one of 0x13df70's rays met (FUN_000ddd10's result record): how far along it, the face's
/// normal there (+0x18..), and whether that's the terrain (the record's kind 1, PID 2 TERRAIN;
/// an object's or a body's is kind 2).
#[derive(Clone, Copy, Debug, PartialEq)]
struct RayHit {
    t: f32,
    normal: Vec3,
    terrain: bool,
}

/// How 0x13df70 found its point, for the log.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Found {
    /// the reticle's lock: its position
    Lock,
    /// the aim's ray met the terrain (raised by the record's rise)
    Terrain,
    /// the aim's ray met an object or a body on a face that doesn't look down
    Face,
    /// the second ray, straight down from a face that looks down, or from the reach's end when the
    /// aim's ray met nothing (raised by the rise)
    Below,
    /// nothing below either: the reach under the second ray's start (raised by the rise)
    Nothing,
}

/// 0x13df70's point for a character a player controls (see the module notes), `lock` the reticle's
/// lock's position, if it has one. Else the ray from the camera's eye `eye` toward the reticle's
/// centre-ray point `toward` over the record's reach (FUN_000ddd10 at 0x13e1ed, its filter
/// 0x13dec0: objects, bodies and the terrain, +0x10c 0xfffffff8, the character's own body left
/// out: `first`):
///  - the terrain (kind 1): the point, raised by the rise (0x13e3c6);
///  - an object or a body whose face there doesn't look down (the normal's y >= 0, 0x13e27f): the
///    point (0x13e3d3);
///  - one that looks down, or nothing within the reach: a second ray straight down (0x13e336, its
///    filter 0x13df40: objects and the terrain, no bodies: `down`) from 1 m out along the face's
///    normal, or from 1 m below the reach's end (the normal's slots start as (0, -1, 0), which the
///    ray leaves when it meets nothing), 1.25 x the reach long: where it lands, or the reach below
///    its start, raised by the rise (0x13e3b9).
fn target_point(lock: Option<Vec3>, eye: Vec3, toward: Vec3, data: &EffectData, first: impl Fn(Vec3, Vec3, f32) -> Option<RayHit>,
                down: impl Fn(Vec3, Vec3, f32) -> Option<RayHit>) -> (Vec3, Found) {
    if let Some(p) = lock {
        return (p, Found::Lock);
    }
    let dir = (toward - eye).normalize_or(Vec3::NEG_Z);
    let (from, normal) = match first(eye, dir, data.reach) {
        Some(hit) if hit.terrain => return (eye + dir * hit.t + Vec3::Y * data.rise, Found::Terrain),
        Some(hit) if hit.normal.y >= 0.0 => return (eye + dir * hit.t, Found::Face),
        Some(hit) => (eye + dir * hit.t, hit.normal),
        None => (eye + dir * data.reach, Vec3::NEG_Y),
    };
    let start = from + normal;
    match down(start, Vec3::NEG_Y, data.reach * SECOND_RAY) {
        Some(hit) => (start - Vec3::Y * hit.t + Vec3::Y * data.rise, Found::Below),
        None => (start - Vec3::Y * data.reach + Vec3::Y * data.rise, Found::Nothing),
    }
}

/// What a ray meets of the world within `max`: the map's terrain and objects without its blockers
/// (`Arena::ray_shot`, the weapons' filter, whose PID kinds 0-2 are 0x13df70's), or on the test
/// map's floor its ground (the terrain), pillars and tools' pieces (`ray_hit`; a hit above the
/// ground is taken for an object's face that doesn't look down).
fn world_ray(from: Vec3, dir: Vec3, max: f32) -> Option<RayHit> {
    if let Some(a) = world::arena().filter(|a| !a.is_flat()) {
        return a.ray_shot(from, dir, max, None).map(|s| RayHit { t: s.t, normal: s.normal, terrain: s.terrain });
    }
    ray_hit(from, dir, max).map(|t| {
        let ground = (from + dir * t).y <= GROUND + 0.01;
        RayHit { t, normal: if ground { Vec3::Y } else { Vec3::new(-dir.x, 0.0, -dir.z).normalize_or(Vec3::Y) }, terrain: ground }
    })
}

/// `world_ray`, or a living character's body if it comes first (`ray_body`: an upright cylinder,
/// so its face there is its side's, the normal level): 0x13df70's first ray. The player's own body
/// isn't among `others` (the filter's +0x118 leaves it out).
fn aim_ray(from: Vec3, dir: Vec3, max: f32, others: &[&Player]) -> Option<RayHit> {
    let world = world_ray(from, dir, max);
    let body = others.iter().filter(|u| !u.dead).filter_map(|u| ray_body(from, dir, u, max).map(|t| (t, u.position)))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match (body, world) {
        (Some((t, at)), w) if w.is_none_or(|w| t <= w.t) => {
            let p = from + dir * t;
            Some(RayHit { t, normal: Vec3::new(p.x - at.x, 0.0, p.z - at.z).normalize_or(-dir), terrain: false })
        }
        (_, w) => w,
    }
}

/// A random share in [0, 1) from `u`'s numbers (the game's spawner draws from its one LCG,
/// 0x46cfd8, as `(n >> 9 | 1.0) - 1.0`; the demo's are per character, `Player::random`).
fn share(u: &mut Player) -> f32 {
    u.random(1 << 23) as f32 / (1u32 << 23) as f32
}

/// The turn FUN_0013e410 gives an effect: M = Rx(the record's pitch) (FUN_0013e800, rows (1, 0, 0),
/// (0, cos, -sin), (0, sin, cos): glam's `from_rotation_x`), then M = Ry(a) x M for a random a in
/// [0, 2 pi) (FUN_0001fa20), read as play_shots.rs reads the pair (`directions`). The Orbital
/// Beam's pitch is 0: a turn about the vertical alone. The Air Strike's is 1 rad (#203), the effect
/// tilted 57 degrees off the vertical and then swung to a random side about it, as its takes show:
/// each shell's flare comes in from another side of the picture (take05's five, take07's: left,
/// right, centre), where a yaw about the effect's own axis first and the tilt after, a fixed side,
/// would bring every one in from the same.
pub(super) fn turn(u: &mut Player, data: &EffectData) -> Quat {
    let yaw = share(u) * std::f32::consts::TAU;
    Quat::from_rotation_y(yaw) * Quat::from_rotation_x(data.pitch)
}

/// FUN_0013e410 for `u`'s skill at `point`: `count` effects of the record's type, each at the point
/// plus a random offset within +-the scatter on x and z (FUN_0013e740), turned as `turn` gives (once,
/// or anew for each with the re-roll), with the user parameter 1 - a random share when the re-roll
/// is set, else 1 (FUN_0013e600), owned by `u`. Nothing for a record with no effect (+0x36 = 0).
fn spawn(u: &mut Player, data: &EffectData, point: Vec3) -> Vec<EffectSpawn> {
    if data.effect == 0 {
        return vec![];
    }
    let mut rotation = turn(u, data);
    let mut out = vec![];
    for _ in 0..data.count {
        let (dx, dz) = (share(u), share(u));
        let at = point + Vec3::new((dx + dx - 1.0) * data.scatter, 0.0, (dz + dz - 1.0) * data.scatter);
        let user = 1.0 - share(u) * if data.reroll { 1.0 } else { 0.0 };
        out.push(EffectSpawn::of_skill(data.effect, at, rotation, u.who(), user));
        if data.reroll {
            rotation = turn(u, data);
        }
    }
    out
}

/// The spawner asked for this frame (`Skill::fire`, FUN_0013e410): for the controlled character
/// its target point is found first (0x13df70: the reticle's lock, else the aim's rays,
/// `target_point`) and kept (`Skill::point`); anyone else's is the one it has (the AI's goal's,
/// which the demo doesn't write: see the module notes). Then the effects are asked of
/// play_effects.rs.
fn fire_effects(mut player: ResMut<Player>, mut squad: ResMut<Squad>, reticle: Res<reticle::Reticle>, mut spawns: ResMut<EffectSpawns>) {
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    if std::mem::take(&mut player.skill.fire) {
        if let Some(data) = player.skill.kind.as_ref().and_then(|k| k.effect_data()).copied() {
            let lock = reticle.lock.and_then(|who| squad.0.iter().find(|u| u.who() == who && !u.dead)).map(missile::root_point);
            let others: Vec<&Player> = squad.0.iter().collect();
            let (point, found) = target_point(lock, reticle.eye, reticle.point, &data, |f, d, m| aim_ray(f, d, m, &others), world_ray);
            player.skill.point = point;
            let made = spawn(&mut player, &data, point);
            if log {
                println!("t={:.2} {}: the target point {point:.2} ({found:?}; the camera at {:.2}, the reticle's point {:.2}, {:.1} m from the feet): {} effects",
                         player.sim_time, unit_name(&player), reticle.eye, reticle.point, point.distance(player.position + Vec3::Y * (GROUND + player.height)),
                         made.len());
            }
            spawns.0.extend(made);
        }
    }
    for m in squad.0.iter_mut().filter(|m| m.skill.fire) {
        m.skill.fire = false;
        let Some(data) = m.skill.kind.as_ref().and_then(|k| k.effect_data()).copied() else { continue };
        let point = m.skill.point;
        let made = spawn(m, &data, point);
        if log {
            println!("t={:.2} {}: the target point {point:.2} (as it was: not a player's character): {} effects", m.sim_time, unit_name(m), made.len());
        }
        spawns.0.extend(made);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Orbital Beam's record as the data has it.
    fn beam() -> EffectData {
        EffectData::orbital_beam()
    }

    /// A flat ground at height `y` (the terrain), met from above.
    fn ground(y: f32) -> impl Fn(Vec3, Vec3, f32) -> Option<RayHit> {
        move |from: Vec3, dir: Vec3, max: f32| {
            (dir.y < -1e-6).then(|| (y - from.y) / dir.y).filter(|&t| t > 0.0 && t <= max).map(|t| RayHit { t, normal: Vec3::Y, terrain: true })
        }
    }

    /// Locked, the point is the lock's, whatever the rays would meet (take01: the lock target's
    /// hips).
    #[test]
    fn a_lock_is_the_point() {
        let lock = Vec3::new(-14.58, 20.12, -9.03);
        let (p, found) = target_point(Some(lock), Vec3::ZERO, Vec3::NEG_Z, &beam(), ground(0.0), ground(0.0));
        assert_eq!((p, found), (lock, Found::Lock));
    }

    /// No lock: the aim's ray meets the ground ahead and the point is where (take02's floor 23.2 m
    /// ahead), raised by the rise when there's one (the Air Strike's 1 on the terrain).
    #[test]
    fn the_aim_meets_the_ground() {
        let eye = Vec3::new(0.0, 2.0, 0.0);
        let toward = Vec3::new(0.0, 0.0, -23.0);
        let (p, found) = target_point(None, eye, toward, &beam(), ground(0.0), ground(0.0));
        assert_eq!(found, Found::Terrain);
        assert!(p.distance(toward) < 1e-3, "{p}");
        let strike = EffectData { rise: 1.0, ..beam() };
        let (p, _) = target_point(None, eye, toward, &strike, ground(0.0), ground(0.0));
        assert!(p.distance(toward + Vec3::Y) < 1e-3, "{p}");
    }

    /// A wall's face (an object, its normal level): the point on it, not raised and not pushed out
    /// (take03: the wall 4.9 m ahead, the glow on the wall's plane).
    #[test]
    fn the_aim_meets_a_wall() {
        let wall = |from: Vec3, dir: Vec3, max: f32| {
            (dir.x < -1e-6).then(|| (-4.9 - from.x) / dir.x).filter(|&t| t > 0.0 && t <= max).map(|t| RayHit { t, normal: Vec3::X, terrain: false })
        };
        let eye = Vec3::new(2.0, 1.5, 0.0);
        let toward = Vec3::new(-4.9, 1.6, 0.0);
        let strike = EffectData { rise: 1.0, ..beam() };
        let (p, found) = target_point(None, eye, toward, &strike, wall, ground(0.0));
        assert_eq!(found, Found::Face);
        assert!(p.distance(toward) < 1e-3, "{p}");
    }

    /// A face that looks down (a body's underside, take12's teammate): the second ray drops from
    /// 1 m out along its normal to the ground, so the point is on the floor before it.
    #[test]
    fn a_face_looking_down_drops_to_the_floor() {
        let normal = Vec3::new(0.0, -0.5, 0.866);
        let underside = move |from: Vec3, dir: Vec3, _: f32| {
            (dir.z < 0.0).then(|| RayHit { t: (from.z + 15.0) / -dir.z, normal, terrain: false })
        };
        // (the face 1.2 m up, so 1 m out along its normal is still above the floor)
        let (eye, toward) = (Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, 1.2, -15.0));
        let (p, found) = target_point(None, eye, toward, &beam(), underside, ground(0.0));
        assert_eq!(found, Found::Below);
        let dir = (toward - eye).normalize();
        let hit = eye + dir * ((eye.z + 15.0) / -dir.z);
        let below = hit + normal;
        assert!((p - Vec3::new(below.x, 0.0, below.z)).length() < 1e-3, "{p} {below}");
        assert!(p.z > -15.0, "nearer the eye than the face: {p}");
    }

    /// Nothing within the reach (the sky): the second ray drops from 1 m below the reach's end, and
    /// with no ground under it either the point is the reach below that.
    #[test]
    fn nothing_within_the_reach() {
        let eye = Vec3::new(0.0, 2.0, 0.0);
        let up = Vec3::new(0.0, 12.0, -10.0);
        let sky = |_: Vec3, _: Vec3, _: f32| None;
        let end = eye + (up - eye).normalize() * 50.0;
        let (p, found) = target_point(None, eye, up, &beam(), sky, ground(0.0));
        assert_eq!(found, Found::Below);
        assert!((p - Vec3::new(end.x, 0.0, end.z)).length() < 1e-3, "{p}");
        // (the end's 37 m up and 1.25 x 50 reaches the ground; at 70 m up it wouldn't)
        let high = |_: Vec3, _: Vec3, _: f32| None;
        let (p, found) = target_point(None, eye + Vec3::Y * 40.0, up + Vec3::Y * 40.0, &beam(), sky, high);
        assert_eq!(found, Found::Nothing);
        assert!((p.y - (eye.y + 40.0 + (up - eye).normalize().y * 50.0 - 1.0 - 50.0)).abs() < 1e-3, "{p}");
    }

    /// FUN_0013e910: under the cost nothing at all (take04's refused presses at 53.2-53.6); at the
    /// cost or more the effect asked for, the cost off at once (take01: 100 -> 45), the name held
    /// 0.2 s, and the skill never on.
    #[test]
    fn the_toggle_pays_and_fires_without_switching() {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_ORBITAL_BEAM, max_stamina: 100.0, cost: 55.0, recovery: 2.3,
                                                        stamina_update: true, ..Default::default() }), &CombatSkills::default());
        u.max_stamina = 100.0;
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut k = u.skill.kind.take().unwrap();
        u.stamina = 54.9;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
        assert_eq!((u.stamina, u.skill.fire, u.skill.name_hold), (54.9, false, 0.0));
        u.stamina = 55.0;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Stays);
        assert_eq!((u.stamina, u.skill.fire, u.skill.name_hold, u.skill.on), (0.0, true, NAME_HOLD, false));
        assert_eq!(k.name(), 0xFF75_7F61);
        assert!(k.attack_skill() && k.effect_data().is_some_and(|d| d.effect == 0xFF8E_79D6));
    }

    /// Who gets the Orbital Beam from its type: a type of its own whose skill-set is 9 (the Confed
    /// Marine, the Confed allies); no other class, and nothing without data. BF_TEST_GIVE_SKILL's
    /// value: the class, then the cost and recovery, the Marine's when left out.
    #[test]
    fn the_marine_and_the_hook() {
        let marine = SkillData { skill_set: CSKILL_ORBITAL_BEAM, cost: 55.0, recovery: 2.3, ..Default::default() };
        let wrath = SkillData { skill_set: 13, ..Default::default() };
        assert!(own_type_skill(Some(&marine)).is_some());
        assert!(own_type_skill(Some(&wrath)).is_none() && own_type_skill(None).is_none());
        assert!(kind(CSKILL_ORBITAL_BEAM, &CombatSkills::default()).is_some() && kind(13, &CombatSkills::default()).is_none());
        assert_eq!(parse_give("9"), Some((9, 55.0, 2.3)));
        assert_eq!(parse_give(" 9, 99 ,0"), Some((9, 99.0, 0.0)));
        assert_eq!(parse_give("orbital"), None);
        // (the Air Strike's are the Militia's, and the Militia's type has the class too)
        assert_eq!(parse_give("6"), Some((6, 20.0, 1.0)));
        assert_eq!(parse_give("6,30"), Some((6, 30.0, 1.0)));
        let militia = SkillData { skill_set: 6, cost: 20.0, recovery: 1.0, ..Default::default() };
        assert!(own_type_skill(Some(&militia)).is_some());
        assert!(kind(6, &CombatSkills::default()).is_some());
    }

    /// FUN_0013e410 with the Orbital Beam's record: one effect, at the point itself (no scatter),
    /// owned by the character, its user parameter 1, turned about y alone (no pitch); a record
    /// with a scatter puts each within +-it on x and z, at the point's height.
    #[test]
    fn the_spawner_places_its_effects() {
        let mut u = Player::new(3);
        let point = Vec3::new(1.0, 2.0, 3.0);
        let made = spawn(&mut u, &beam(), point);
        assert_eq!(made.len(), 1);
        let s = &made[0];
        assert_eq!((s.kind, s.at, s.owner, s.user, s.rules), (0xFF8E_79D6, point, Some(u.who()), 1.0, super::super::effects::Rules::Game));
        let (axis, _) = s.rotation.to_axis_angle();
        assert!(axis.y.abs() > 0.999 || s.rotation.angle_between(Quat::IDENTITY) < 1e-4, "{axis}");
        let strike = EffectData { count: 5, scatter: 2.0, ..beam() };
        let made = spawn(&mut u, &strike, point);
        assert_eq!(made.len(), 5);
        assert!(made.iter().all(|s| (s.at.x - point.x).abs() <= 2.0 && (s.at.z - point.z).abs() <= 2.0 && s.at.y == point.y));
        assert!(spawn(&mut u, &EffectData { effect: 0, ..beam() }, point).is_empty());
    }
}
