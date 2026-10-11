//! The damage a character takes (#127): default.xbe's path from a hit to the hit points, read by
//! the analyst in todo/damage-factor/answer.md (every function cited is in docs/code-map.md),
//! checked against the takes' HP logs of #35, #73 and #115.
//!
//!  1. **The roll** (FUN_0022c940): Damage min + U x (max - min). The shooter's own factors (a
//!     power-up's char+0x754, Flint's double and scoped hits through its skill's +0xec, #124)
//!     aren't in the demo.
//!  2. **The hit record** (FUN_0022c4d0): a direct hit takes the roll, a blast's share its part
//!     of the blast's roll (play_missile.rs), then x the hit factor:
//!     - an instant ray (FUN_00230040): its range falloff (play_shots.rs) x FUN_00125940;
//!     - a projectile (FUN_002278e0): the missile's +0x1cc (always 1.0) x FUN_00125940;
//!     - a blast's share: 1 (FUN_00224a90 passes 0 to FUN_00224320).
//!
//!     FUN_00125940 is the victim type's `damage-multiplier` (h_e7e9fcd3, type+0x220: 0.62 for
//!     the campaign squad, 1 for the Shrikes and the deathmatch squad) x the `Damage` of the bone
//!     whose hit part was struck (`struck_bone`: the head x5, or x3 for Brutus and the hounds;
//!     upper arms x0.8; forearms and calves x0.5; the rest x1). Then x0.7 when the shooter and
//!     the victim are both the squad's (FUN_0004b9c0), the shooter's own blast included.
//!  3. **The victim** (0x125600, a character's combat-target vtable slot 0, 0x39ddcc). Every hit
//!     on a character goes through it: a weapon's hit or blast (MSG_DAMAGE from the weapon's
//!     handler 0x22d0b0), a damage area (FUN_0021ade0), a fall (FUN_0012b740), FUN_00223160 (a
//!     charge). It takes x (1 - 0.1n) for a victim on an enemy team and x (1 + 0.1n) for the
//!     squad's, n the difficulty (`Difficulty`; STANDARD -4: enemies x1.4, the squad x0.6); then
//!     its skill's factor (vtable +0xd8 at 0x125747: Vengar's 0.85, #122, not here); then
//!     FUN_002232d0's factor for the damage type (h_142be76f, `Game::damage_factor`). Only the
//!     trainer's `!hit` calls FUN_002232d0 directly (the one direct call is 0x1257c0, in
//!     0x125600 itself), so older xemu logs of it carry no step.
//!
//! What uses it: play.rs's shots (the player's and the AI's) and falls, play_missile.rs's direct
//! hits and blasts, the grenades (play_grenade.rs, play_energy.rs, play_sonic.rs, play_gas.rs)
//! and the scenery's damage areas (play_scenery.rs). The grenades keep their own rules (their
//! shares aren't the game's yet: grenades.md); their amounts fitted to the takes on a squad
//! victim are divided by the takes' step (`footage_squad_step`), so on STANDARD they come out
//! as measured. Not stepped: the test hooks' kills (BF_TEST_DIE, the test map's suicide and
//! instant kill) and a harmful liquid's (play.rs: the game's liquid damage isn't traced).
//!
//! `BF_DIFFICULTY=<standard|hard|brutal|n>` sets the difficulty (`Difficulty::for_map`).
//! `BF_COMBAT_LOG=1` prints each hit with its factors (`Hit::report`), before play.rs's `hurt`
//! line with the hit points left. `BF_HIT_PARTS=1` draws every standing character's hit parts
//! (`draw_hit_parts`).

use super::*;
use bf_viewer::bf::character::{LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};
use bf_viewer::bf::hash::h;

/// 0x125600's step per level of difficulty (0x3a4ff0 = 0.1): 1 - 0.1n or 1 + 0.1n (the 1 is
/// 0x3a4f78).
const STEP_PER_LEVEL: f32 = 0.1;
/// The team table at 0x3ffd80 (read from default.xbe): row t, column k is 1 when team t is
/// hostile to team k. Rows (the ALIGN_ names at 0x3bc94c): 0 CONFEDERATION (the squad), 1 SHRIKE
/// (the enemies), 2 NEUTRAL, 3-6 (hostile to each other: deathmatch teams, a guess), 7
/// PC_NO_TEAM (hostile to itself), 8 HOSTILE, 9.
const TEAM_TABLE: [[u8; 10]; 10] = [
    [0, 1, 0, 0, 0, 0, 0, 1, 1, 0],
    [1, 0, 0, 1, 1, 1, 1, 1, 1, 0],
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    [0, 1, 0, 0, 1, 1, 1, 1, 1, 0],
    [0, 1, 0, 1, 0, 1, 1, 1, 1, 0],
    [0, 1, 0, 1, 1, 0, 1, 1, 1, 0],
    [0, 1, 0, 1, 1, 1, 0, 1, 1, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 0, 1, 1, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
];
/// The squad's team (`Player::team`; the table's CONFEDERATION row).
const SQUAD_TEAM: u8 = 0;
/// FUN_0022c4d0: a hit by one of the squad on one of the squad (FUN_0004b9c0: both characters
/// with a squad, +0x59c, one in the other's, FUN_0005beb0; the shooter's own blast on themselves
/// too) deals this share, and posts EVT_SHOT_BLOCKED_BY_FRIEND (34).
pub const FRIENDLY: f32 = 0.7;
/// The menu's three difficulties, by their place in `Game::difficulty_settings`: the SELECT
/// DIFFICULTY menu (FUN_000f13c0) stores game-options +0x50 (`easy`) for STANDARD (button
/// h_087c40bb, "For the novice"), +0x54 (`normal`) for HARD (h_e4969005) and +0x58 (`hard`) for
/// BRUTAL (h_10839388).
const MENU: [&str; 3] = ["STANDARD", "HARD", "BRUTAL"];
/// The takes were played on STANDARD, the menu's first: the agent's profile's campaign was
/// started on it (#110), and both sides of every HP log fit its n (answer.md: enemies x1.397 to
/// x1.406, the squad x0.6).
const FOOTAGE: usize = 0;
/// How far (m) a shot is followed past where it met a standing body's cylinder in search of the
/// hit part it struck: across the cylinder (2 x BODY_RADIUS) and an arm held out beyond it (the
/// squad's forearm surfaces reach 0.9 m from the hips in the bind pose). The demo's.
const THROUGH: f32 = 1.5;
/// A shot that passes the cylinder between the hit parts: the points along its way through the
/// body (BODY_RADIUS x 2) it's measured from, to find the nearest part. The demo's.
const CHORD_STEPS: usize = 8;
/// BF_HIT_PARTS's colours (the demo's): a part by its bone's Damage, the head's x5 (or x3) red,
/// the upper arms' x0.8 yellow, the forearms' and calves' x0.5 cyan, the x1 rest white.
const PART_COLOURS: [(f32, Color); 4] = [(2.0, Color::srgb(1.0, 0.2, 0.2)), (0.95, Color::WHITE), (0.7, Color::srgb(1.0, 0.85, 0.2)),
                                         (0.0, Color::srgb(0.2, 0.9, 1.0))];
/// The bones the campaign's motion sets list in `<collision>` (names recovered by hashing,
/// answer.md), for the log.
const HIT_BONE_NAMES: [&str; 11] = ["Bip01", "Bip01 Spine2", "Bip01 Head", "Bip01 L Thigh", "Bip01 R Thigh", "Bip01 L Calf",
                                    "Bip01 R Calf", "Bip01 L UpperArm", "Bip01 R UpperArm", "Bip01 L Forearm", "Bip01 R Forearm"];

/// The difficulty of the map being played, n (0x7c720, the game session's vtable +0x234: the
/// int game flag campaign/difficulty-setting, path at 0x401f60; 0x125600 reads it at 0x12566d).
#[derive(Resource, Clone, Copy, Debug)]
pub struct Difficulty {
    pub n: i64,
    /// for the log: the menu's name, "none" on a multiplayer map, or "set" for a number
    pub name: &'static str,
}

impl Difficulty {
    /// The difficulty a map is played at:
    ///  - BF_DIFFICULTY=<standard|hard|brutal> (the menu's names, any case) or a number n, on any
    ///    map (a test hook: a multiplayer map has none in the game);
    ///  - else on a multiplayer map (the game's level list, campaign-bf.xmb Type: the deathmatch
    ///    arenas and the squad deathmatch maps) none, n = 0: the getter 0x7c720 returns 0
    ///    without the flag, and the session there has none (by fit: #85's take29 on sdm_e34 had
    ///    Tex take 39.4 of a barrel's 40 at 1 m, where the campaign's took ~24 at 1-2.7 m, takes
    ///    01-04: x1 and x0.6);
    ///  - else (the missions, the test map) STANDARD, the takes'.
    pub fn for_map(game: &Game, map: &str) -> Self {
        let values = game.difficulty_settings;
        let menu = |k: usize| values.map(|v| Difficulty { n: v[k], name: MENU[k] });
        if let Ok(want) = std::env::var("BF_DIFFICULTY") {
            let w = want.trim().to_lowercase();
            if let Some(d) = MENU.iter().position(|m| m.to_lowercase() == w).and_then(menu) {
                return d;
            }
            if let Ok(n) = w.parse::<i64>() {
                return Difficulty { n, name: "set" };
            }
            warn!("BF_DIFFICULTY: {want} isn't standard, hard, brutal or a number");
        }
        let map = map.to_lowercase();
        let multiplayer = game.campaign.iter().any(|l| (l.kind == LEVEL_DEATHMATCH || l.kind == LEVEL_SQUAD_DEATHMATCH)
                                                     && (l.file == map || l.zones.contains(&map)));
        if multiplayer {
            return Difficulty { n: 0, name: "none" };
        }
        menu(FOOTAGE).unwrap_or_else(|| {
            warn!("no <difficulty-settings> in game-options-en: no difficulty step");
            Difficulty { n: 0, name: "none" }
        })
    }

    /// 0x125600's step for a victim on `team`: x (1 - 0.1n) when its row of the team table has 1s
    /// in columns 3-6 (0x3ffd83..86: teams 1, 7 and 8, the enemies), else x (1 + 0.1n) (the
    /// squad's 0, NEUTRAL, the rest). On STANDARD (n = -4) an enemy takes x1.4 and the squad
    /// x0.6; on HARD (0) both x1; on BRUTAL (4) x0.6 and x1.4.
    pub fn step(&self, team: u8) -> f32 {
        let k = STEP_PER_LEVEL * self.n as f32;
        if enemy_side(team) { 1.0 - k } else { 1.0 + k }
    }
}

/// Whether 0x125600 takes a victim on `team` for an enemy (see `Difficulty::step`).
fn enemy_side(team: u8) -> bool {
    TEAM_TABLE.get(team as usize).is_some_and(|row| row[3..=6].iter().all(|&b| b != 0))
}

/// The step the takes' squad took (STANDARD on a squad victim: x0.6). The demo's amounts fitted
/// to the takes on the squad (the grenades' thrower shares, the Gas's rate) hold it already:
/// divided by it they come out as measured on STANDARD, now that every hit takes the step.
pub fn footage_squad_step(game: &Game) -> f32 {
    game.difficulty_settings.map_or(1.0, |v| Difficulty { n: v[FOOTAGE], name: MENU[FOOTAGE] }.step(SQUAD_TEAM))
}

/// FUN_0004b9c0's share for a hit by a shooter on `shooter_team` on a victim on `victim_team`:
/// FRIENDLY when both are the squad's. The demo's sides stand for the squads: the same team (the
/// test map's NPCs of a side are taken as one squad). #73's rule.
pub fn friendly(shooter_team: u8, victim_team: u8) -> f32 {
    if shooter_team == victim_team { FRIENDLY } else { 1.0 }
}

/// The victim's hit parts' bones and their `Damage` (its motion set's `<collision>` list,
/// `Game::character_hit_bones`, by the squad character's motion set; FUN_00114010 keeps those
/// that aren't 1 in the map FUN_00125940 reads). The one place they're looked up: a registry of
/// character types (#128) re-points it.
pub fn hit_bones<'a>(game: &'a Game, u: &Player) -> &'a [(u32, f32)] {
    game.character_hit_bones.get(CHARACTERS[u.character % CHARACTERS.len()]).map_or(&[], |v| v.as_slice())
}

/// The victim type's `damage-multiplier` (h_e7e9fcd3, type+0x220; `Game::character_damage_multiplier`,
/// the sdm_bf squad's in place on a squad deathmatch map), 1 if the data has none. Re-pointed with
/// `hit_bones`.
pub fn damage_multiplier(game: &Game, u: &Player) -> f32 {
    game.character_damage_multiplier.get(CHARACTERS[u.character % CHARACTERS.len()]).copied().unwrap_or(1.0)
}

/// The victim type's factor for a damage type (FUN_002232d0: its combat target's h_142be76f list,
/// `Game::damage_factor`). Re-pointed with `hit_bones`.
pub fn vulnerability(game: &Game, u: &Player, damage_type: i64) -> f32 {
    game.damage_factor(CHARACTERS[u.character % CHARACTERS.len()], damage_type)
}

/// Which hit part a shot struck on `u` (its bone), for FUN_00125940. The shot left `from` along
/// `dir` (a unit vector) and met the demo's body `met` m along: play.rs's standing cylinder
/// (`ray_body`) or, knocked down, the ragdoll's bones (play_missile.rs's `ray_lying`).
///  - Standing: the first of its hit parts' collision surfaces (`Character::part_collision`,
///    posed as drawn) the shot meets, up to THROUGH m past the cylinder. The game's shots strike
///    those bodies themselves (FUN_00230040's ray, FUN_002278e0's contact), and FUN_00115000
///    gives the struck body's bone.
///  - A shot through the cylinder that meets none of them (between the legs, past an arm): the
///    part whose bone (from its joint to its first child's) comes nearest the shot's way through
///    the body. The demo's: the game's shot would pass; here the cylinder decides what's hit.
///  - Knocked down: the part whose bone is nearest the contact on the ragdoll (#73's rule; the
///    ragdoll keeps no turn of its bones to pose the surfaces with).
///
/// None without a model or a hit-part list: FUN_00125940 then takes the damage-multiplier alone,
/// as for a body that isn't one of the parts (FUN_00115000's -1).
pub fn struck_bone(u: &Player, game: &Game, from: Vec3, dir: Vec3, met: f32) -> Option<u32> {
    let bones = hit_bones(game, u);
    let l = u.loaded.as_ref()?;
    let contact = from + dir * met;
    if let Some(k) = &u.knock {
        let turn = Quat::from_rotation_y(k.ragdoll.yaw);
        let place = |i: usize| k.ragdoll.pos.get(i).map(|&p| k.ragdoll.origin + turn * p);
        return nearest_part(bones, &l.model, &place, &[contact]);
    }
    let first = posed_parts(u, game).into_iter().filter_map(|(bone, _, frame, surface)| {
        let back = frame.inverse();
        Some((ray_triangles(surface, back.transform_point3(from), back.transform_vector3(dir), met + THROUGH)?, bone))
    }).min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((_, bone)) = first {
        return Some(bone);
    }
    let root = model_root(u);
    let place = |i: usize| u.last_world.get(i).map(|w| root.transform_point3(w.w_axis.truncate()));
    let way: Vec<Vec3> = (0..=CHORD_STEPS).map(|k| contact + dir * (2.0 * BODY_RADIUS * k as f32 / CHORD_STEPS as f32)).collect();
    nearest_part(bones, &l.model, &place, &way)
}

/// Where `u`'s model is drawn: its root's frame (play.rs: the position, raised by the height and
/// the soles' lift, turned by the yaw).
fn model_root(u: &Player) -> Mat4 {
    Mat4::from_rotation_translation(Quat::from_rotation_y(u.yaw), u.position + Vec3::Y * (u.height + u.lift_now))
}

/// `u`'s hit parts as drawn, standing: each part's bone, its Damage, the frame that puts its
/// surface (`Character::part_collision`, in the bone's frame) in the world (the root's x the
/// bone's posed model frame, play.rs's `last_world`), and the surface.
fn posed_parts<'a>(u: &'a Player, game: &'a Game) -> Vec<(u32, f32, Mat4, &'a [[Vec3; 3]])> {
    let Some(l) = u.loaded.as_ref() else { return vec![] };
    let root = model_root(u);
    hit_bones(game, u).iter().filter_map(|&(bone, damage)| {
        let i = l.model.bones.iter().position(|&b| b == bone)?;
        let surface = l.model.part_collision.get(i).filter(|s| !s.is_empty())?;
        Some((bone, damage, root * *u.last_world.get(i)?, surface.as_slice()))
    }).collect()
}

/// The test hook's drawing (BF_HIT_PARTS), once the characters have moved.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, draw_hit_parts.after(update_player).run_if(in_state(AppState::Playing)));
}

/// BF_HIT_PARTS=1: every standing character's hit parts as `struck_bone` sees them, their
/// surfaces' edges in the part's colour (PART_COLOURS), to check them against the drawn model.
fn draw_hit_parts(player: Res<Player>, squad: Res<Squad>, game: Res<GameData>, mut gizmos: Gizmos) {
    if std::env::var("BF_HIT_PARTS").is_err() {
        return;
    }
    for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.dead && u.knock.is_none()) {
        for (_, damage, frame, surface) in posed_parts(u, &game.0) {
            let colour = PART_COLOURS.iter().find(|c| damage >= c.0).map_or(Color::WHITE, |c| c.1);
            for t in surface {
                let [a, b, c] = t.map(|p| frame.transform_point3(p));
                gizmos.line(a, b, colour);
                gizmos.line(b, c, colour);
                gizmos.line(c, a, colour);
            }
        }
    }
}

/// Of `bones`, the one whose bone (the line from its joint to its first child's, placed by
/// `place`) comes nearest any of `points`.
fn nearest_part(bones: &[(u32, f32)], model: &Character, place: &dyn Fn(usize) -> Option<Vec3>, points: &[Vec3]) -> Option<u32> {
    bones.iter().filter_map(|&(bone, _)| {
        let i = model.bones.iter().position(|&b| b == bone)?;
        let a = place(i)?;
        let b = model.parent.iter().position(|&p| p == Some(i)).and_then(place).unwrap_or(a);
        let ab = b - a;
        let d = points.iter().map(|&p| {
            let k = if ab.length_squared() > 1e-8 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
            (a + ab * k).distance(p)
        }).fold(f32::MAX, f32::min);
        Some((d, bone))
    }).min_by(|a, b| a.0.total_cmp(&b.0)).map(|x| x.1)
}

/// Where along a ray (origin `o`, direction `d`; t in d's units) it first meets one of `tris`,
/// either side, within `max` (Moller-Trumbore).
fn ray_triangles(tris: &[[Vec3; 3]], o: Vec3, d: Vec3, max: f32) -> Option<f32> {
    tris.iter().filter_map(|&[a, b, c]| {
        let (e1, e2) = (b - a, c - a);
        let p = d.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-12 {
            return None;
        }
        let s = o - a;
        let u = s.dot(p) / det;
        let q = s.cross(e1);
        let v = d.dot(q) / det;
        let t = e2.dot(q) / det;
        ((0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && (0.0..=max).contains(&t)).then_some(t)
    }).min_by(|a, b| a.total_cmp(b))
}

/// A bone's name for the log (`HIT_BONE_NAMES`), else its hash.
fn bone_name(bone: u32) -> String {
    HIT_BONE_NAMES.iter().find(|n| h(n) == bone).map_or_else(|| format!("h_{bone:08x}"), |n| n.to_string())
}

/// A character's name for the log: the squad character, and the test map NPC's number.
fn who_name(who: Who) -> String {
    match who {
        Who::Squad(c) => CHARACTERS[c % CHARACTERS.len()].to_string(),
        Who::Npc(n) => format!("NPC {n}"),
    }
}

/// One hit on a character with every factor of the game's path (the module notes), for its
/// amount and the log. Built where the hit is made (the record, FUN_0022c4d0); the victim's own
/// factors (0x125600, FUN_002232d0) are taken when it reaches them (`Hit::on`).
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    /// what the amount starts from: a roll of the weapon's Damage min..max, a blast's share of its
    /// roll, or the demo's own amount for the grenades', the damage areas' and a fall's rules
    pub base: f32,
    /// what `base` is, for the log
    pub kind: &'static str,
    /// who dealt it (None: a fall, a damage area)
    pub by: Option<Who>,
    /// an instant ray's range falloff (FUN_00230040 -> FUN_00223780); 1 for anything else
    pub falloff: f32,
    /// a direct hit's FUN_00125940: the victim type's damage-multiplier, and the struck part's bone
    /// with its Damage (None: no part, x1); None for a hit with no hit part (a blast's share
    /// and the rest: FUN_00224a90 passes 0, FUN_00223160 has none)
    pub part: Option<(f32, Option<(u32, f32)>)>,
    /// FRIENDLY when the shooter and the victim are both the squad's (FUN_0004b9c0), else 1
    pub friendly: f32,
    /// the victim's step (0x125600) and the difficulty it's at, once it reaches them
    pub step: f32,
    pub difficulty: Option<Difficulty>,
    /// the damage type (`Damage damage-type`), and the victim's factor for it (FUN_002232d0) once
    /// it reaches them
    pub damage_type: i64,
    pub vulnerability: f32,
}

impl Hit {
    /// A hit of `base` (`kind`, for the log) of damage type `damage_type`, with no other factor yet.
    pub fn new(base: f32, kind: &'static str, damage_type: i64) -> Self {
        Hit { base, kind, by: None, falloff: 1.0, part: None, friendly: 1.0, step: 1.0, difficulty: None, damage_type, vulnerability: 1.0 }
    }

    /// Dealt by `who`, on `shooter_team`: x FRIENDLY on a victim of the same side (`friendly`).
    pub fn by(mut self, who: Who, shooter_team: u8, victim: &Player) -> Self {
        self.by = Some(who);
        self.friendly = friendly(shooter_team, victim.team);
        self
    }

    /// Dealt by `who` (for the log), without FUN_0004b9c0's share: the grenades' own rules, whose
    /// numbers were fitted without it.
    pub fn source(mut self, who: Who) -> Self {
        self.by = Some(who);
        self
    }

    /// An instant ray's range falloff (FUN_00230040).
    pub fn falloff(mut self, falloff: f32) -> Self {
        self.falloff = falloff;
        self
    }

    /// A direct hit that struck `bone` on `victim` (`struck_bone`): FUN_00125940, the victim
    /// type's damage-multiplier x the bone's Damage (1 for a bone the list doesn't hold).
    pub fn struck(mut self, victim: &Player, game: &Game, bone: Option<u32>) -> Self {
        let damage = bone.map(|b| (b, hit_bones(game, victim).iter().find(|x| x.0 == b).map_or(1.0, |x| x.1)));
        self.part = Some((damage_multiplier(game, victim), damage));
        self
    }

    /// The hit reaching `u`: their step (0x125600, by their team) and their factor for the damage
    /// type (FUN_002232d0). The amount.
    pub fn on(&mut self, u: &Player, game: &Game, difficulty: &Difficulty) -> f32 {
        self.step = difficulty.step(u.team);
        self.difficulty = Some(*difficulty);
        self.vulnerability = vulnerability(game, u, self.damage_type);
        self.amount()
    }

    /// The amount: every factor's product.
    pub fn amount(&self) -> f32 {
        let part = self.part.map_or(1.0, |(m, b)| m * b.map_or(1.0, |b| b.1));
        self.base * self.falloff * part * self.friendly * self.step * self.vulnerability
    }

    /// `on`, then the log line (BF_COMBAT_LOG), at sim time `now`. The amount.
    pub fn land(&mut self, u: &Player, game: &Game, difficulty: &Difficulty, now: f32) -> f32 {
        let amount = self.on(u, game, difficulty);
        self.report(u, now);
        amount
    }

    /// BF_COMBAT_LOG: the hit on `u` with each factor and the amount (play.rs's `hurt` prints the
    /// hit points left after it).
    pub fn report(&self, u: &Player, now: f32) {
        if std::env::var("BF_COMBAT_LOG").is_err() {
            return;
        }
        let victim = format!("{}{}", CHARACTERS[u.character % CHARACTERS.len()], u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)));
        let mut s = format!("t {now:.2}: {} -> {victim}: {} {:.2}", self.by.map_or("-".into(), who_name), self.kind, self.base);
        if self.falloff != 1.0 {
            s += &format!(" x falloff {:.3}", self.falloff);
        }
        if let Some((multiplier, bone)) = self.part {
            s += &format!(" x damage-multiplier {multiplier:.2}");
            s += &match bone {
                Some((b, damage)) => format!(" x {} {damage}", bone_name(b)),
                None => " x no hit part 1".into(),
            };
        }
        if self.friendly != 1.0 {
            s += &format!(" x own side {}", self.friendly);
        }
        if let Some(d) = self.difficulty {
            s += &format!(" x {} (n {}) {} {:.2}", d.name, d.n, if enemy_side(u.team) { "enemy" } else { "squad" }, self.step);
        }
        s += &format!(" x type {} {:.2} = {:.2}", self.damage_type, self.vulnerability, self.amount());
        println!("{s}");
    }
}
