//! The campaign's state (#221, the tutorial's step 2 of #28; viewer/docs/campaign.md): the game
//! flags, the save-game squad slots, the squad member flags and the mission's cash, kept as
//! default.xbe's session and campaign mode keep them. The level's gamevar and score triggers
//! (play_triggers.rs) read and write them; the objectives menu (play_objectives.rs) shows them.
//!
//! The flags. The session keeps a flag tree per save slot (vtable +0x210, 0x78d60): a new
//! campaign's is `common/default-gameflags.xmb` (`Game::default_flags`). Here one tree lives as
//! long as bf_play does (`Campaign::flags`), from the first mission played on, level after level,
//! as a campaign carries it; saving it is #27's.
//!
//! The squad, two sets of flags:
//!  * the save-game slots `campaign/save-game/character-1..4/{archetype, h_03761692, ...}`. When a
//!    level other than the current one is asked for (FUN_0007f730), the session's +0xb4
//!    (0x841e0) clears them (FUN_00083fb0) and the save-game cash (FUN_000846b0), then fills them
//!    (FUN_00084230): first the campaign's squad members the zone has (the session's +0xec,
//!    FUN_00080f20: campaign-bf's `squad-char`s not in the zone's h_ed54d255), alive; then as
//!    many of the campaign's extra members (h_1f45a087: the three Confed marine types) as the
//!    zone lacks (+0xf0, 0x80ff0), alive only if a player controls the type. So in the tutorial
//!    (h_ed54d255 "brutus hawk flint"): Tex alive, Gonzales, Cogburn and Stout not.
//!  * the member flags `<type>/h_03761692` (default-gameflags' top-level `<tex>`, `<hawk>`,
//!    `<brutus>`, `<flint>` and the marine types; all false by default), written by the
//!    session's +0xbc (FUN_000814b0, in a mission only): to 1 by every squad-type character's
//!    set-up (FUN_0012f440: the level's placed ones as it loads, a spawn's clone), to 0 by its
//!    death and back to 1 by its revival (its +0x144, FUN_00112830), at a spawn-trigger's link
//!    (FUN_00209890, a trigger with h_0fa4f510 and a squad-type template) to whether the session
//!    counts the template in the squad (+0xb8, 0x83ee0: its save-game slot alive, or a player
//!    controlling its type, or no slot of its type at all), and to 1 by a spawn from a trigger
//!    without h_0fa4f510 (FUN_00209900). That gives the takes' flags: tex 1 and the marines 0 at
//!    the tutorial's start (take03's `1|0|0|0`: the marines' placements set theirs, then their
//!    spawn-triggers' links clear them), 0|0|0|0 while Tex is dead (take05, take08), and Cogburn
//!    and Stout 1 once the V-squad's spawns (no h_0fa4f510) have run (take11's `1|0|1|1`). The
//!    member check of a spawn (FUN_00209900, play_triggers.rs) reads the save-game slots
//!    (`Campaign::in_squad`), the level's gamevar-triggers the member flags.
//!
//! The cash: the campaign mode's squad (mode+0x1c, made with the mode at each level's start)
//! keeps it (`Cash`). A score-trigger adds its amount (FUN_0005c5e0, x 1 + 0.1 x the difficulty:
//! the tutorial's 5000 and 1000 are $3000 and $600 on STANDARD, take12's end screen); a death
//! pays or costs its type's award (FUN_00157070, `deaths`: the Red Hand +$60 each, Tex -$3000).
//!
//! Test hooks:
//!  * BF_FLAG_LOG=1: at a level's start the squad slots, the member flags and the mission's
//!    objectives' flags; then every flag write (who, the path, the value before and after) and
//!    every member check. BF_FLAG_LOG=all lists the whole tree at the start as well.
//!  * BF_FLAG_SET=<path>=<value>[;...]: sets flags at the level's start, after the squad's set-up
//!    and before the triggers' first update. The path: names or h_xxxxxxxx joined by `/`
//!    (`h_f48ed65e/h_03761692`, `missions/m00/h_eaf792f4/state`); the value a number or
//!    true / false.
//!  * BF_CASH_LOG=1: every change to the cash (who, the amount and its scaling, the total).

use std::collections::HashMap;

use bf_viewer::bf::campaign::Objective;
use bf_viewer::bf::character::Game;
use bf_viewer::bf::gameflags::{FlagValue, GameFlags};
use bf_viewer::bf::hash::h;
use bf_viewer::bf::triggers::Class;

use super::*;

/// The campaign's state, the session's and the campaign mode's (see the module notes).
pub fn plugin(app: &mut App) {
    app.init_resource::<Campaign>()
        .add_systems(OnEnter(AppState::Playing), begin_level.after(setup))
        .add_systems(Update, deaths.after(update_player).run_if(in_state(AppState::Playing)));
}

/// The member flag's name under a type's folder (`<tex><h_03761692>`; the save-game slots' alive
/// flag has the same name): 0x401f74, the path's last part in FUN_000814b0 (0x81502) and 0x83ee0.
pub const H_MEMBER: u32 = 0x0376_1692;
/// The save-game folder's path, {campaign, save-game} (0x401f68, 0x401f6c).
const SAVE_GAME: [&str; 2] = ["campaign", "save-game"];
/// The save-game squad's slots, character-1..4 (the format "character-%d" at 0x40dbe8; FUN_00083fb0
/// and 0x83d10 count four).
const SLOTS: usize = 4;
/// What FUN_00083fb0 sets back to its default in each slot: archetype (0x401f70), h_03761692
/// (0x401f74), current-weapon (0x401f88), h_f02f13eb (0x401f98), h_10323f44 (0x401f9c).
const SLOT_RESETS: [u32; 5] = [0xE0D7_811B, H_MEMBER, 0xFE3F_3014, 0xF02F_13EB, 0x1032_3F44];
/// A slot's current-weapon (0x401f88): FUN_00084230 writes 0 to it with each slot it fills.
const CURRENT_WEAPON: u32 = 0xFE3F_3014;
/// The save-game cash folder campaign/save-game/cash's (0x402000-0x402008; h_ee573b51 hashes
/// "cash") three totals (0x40200c-0x402014: h_e4a36406, h_19fd3207, h_f2a54161): FUN_000846b0
/// writes 0 to each.
const CASH_TOTALS: [u32; 3] = [0xE4A3_6406, 0x19FD_3207, 0xF2A5_4161];
/// Its entries' folder, h_1f4d6a99 (0x402018).
const CASH_ENTRIES: u32 = 0x1F4D_6A99;
/// The two other save-game flags FUN_000846b0 writes 0 to: dna-archetype (0x402030, in
/// {campaign, save-game, ..} at 0x402028) and h_08b97eea (0x402024).
const SAVE_GAME_CLEARED: [u32; 2] = [0x1C32_7BAB, 0x08B9_7EEA];
/// Each entry's fields, {key, h_1c785ea9, amount, h_eac34649} (FUN_000846b0 writes 0 to each).
const CASH_ENTRY_FIELDS: [u32; 4] = [0xF66A_B3A9, 0x1C78_5EA9, 0xE01A_523C, 0xEAC3_4649];
/// How many entries, entry-1..12 (FUN_000846b0's "entry-%d" at 0x40dbd4; the schema's twelve).
const CASH_ENTRY_COUNT: usize = 12;
/// A cash key that FUN_0005c5e0 also puts in the record's own totals, besides the null name
/// (0x5c62f-0x5c637: h_0c07b477, name not recovered).
const CASH_KEY_TOTALS: u32 = 0x0C07_B477;
/// A death award's key for a type with no display name: h_1ac9db24, "NO NICE NAME" (FUN_00154a20
/// at 0x154a9a).
const NO_NICE_NAME: u32 = 0x1AC9_DB24;
/// The scaling of a score-trigger's amount and a death's award: 1 + 0.1 x the difficulty
/// (0x20bac7 and FUN_00157070: 0x3a4ff0 0.1, 0x3a4f78 1.0). STANDARD's -4 gives 0.6.
const CASH_STEP: f32 = 0.1;
/// The campaign squad's team (squad+0xc), the side whose deaths cost it (FUN_00157070 compares
/// the dead's team with it): the squad's, 0, CONFEDERATION.
const SQUAD_SIDE: u8 = 0;
/// The objectives menu's suffix for state -1 (FUN_000f8310: a space, 0x40cd24, then the string):
/// "(failed)", h_fdc04956.
const FAILED: u32 = 0xFDC0_4956;
/// The suffix for state 1: "(completed)", h_eb2ff8cb.
const COMPLETED: u32 = 0xEB2F_F8CB;
/// The flag names the logs know (hashed; the rest show as h_xxxxxxxx): the decoder's readable
/// ones and the missions' folder (h_0630ae70 "missions", recovered by hashing).
const FLAG_NAMES: [&str; 28] = ["campaign", "save-game", "character-1", "character-2", "character-3", "character-4", "archetype",
    "weapon-one", "weapon-two", "current-weapon", "hit-points", "squad", "inventory", "item-count", "amount", "key", "cash",
    "difficulty-setting", "tex", "brutus", "hawk", "flint", "missions", "state", "enabled", "max-score", "num-brief", "dna-archetype"];

/// One entry of the cash record's map (FUN_0005c5e0 / FUN_0005c730's std::map node: key +0xc, sum
/// +0x10, count +0x14).
#[derive(Clone, Debug)]
pub struct CashEntry {
    /// a death's type's display name (type+0x120), a score-trigger's own key
    pub key: u32,
    /// the cash under it
    pub sum: f32,
    /// how many times something was added under it (the end screen's "24 x Red Hand")
    pub count: u32,
}

/// The campaign squad's cash (the mode's squad, mode+0x1c; FUN_0005c5e0, FUN_0005c730): its own
/// totals, what went in under the null name (the score-triggers', above and below 0), and its
/// entries by key (the deaths', by type). The end screen reads it (take12: Cash Awards +$5040 =
/// the objectives' 3000 + 600 and "24 x Red Hand +$1440"; Cash Penalties "1 x Tex -$3000").
#[derive(Clone, Debug, Default)]
pub struct Cash {
    /// +0x2c: added under the null name, above 0
    pub awards: f32,
    /// +0x30: added under the null name, below 0
    pub penalties: f32,
    /// +0x38: the entries, in key order (a std::map)
    pub entries: Vec<CashEntry>,
    /// +0x20: the squad's members who died on the level, the end screen's "Operatives Lost"
    /// (FUN_00115d50 at 0x115e10 counts a character's death into its squad's record, char+0x59c,
    /// unless a cutscene has the world; the session's +0x1d4 adds the save-game's count,
    /// h_08b97eea, 0 on a new level: take12's 1, Tex's death in the lava)
    pub lost: u32,
}

impl Cash {
    /// `amount` under `key` (FUN_0005c5e0, FUN_0005c730): the null name (and h_0c07b477) to the
    /// own totals by its sign, nothing for 0; any other key to its entry, one more in its count.
    pub fn add(&mut self, key: u32, amount: f32) {
        if key == 0 || key == CASH_KEY_TOTALS {
            if amount > 0.0 {
                self.awards += amount;
            } else if amount < 0.0 {
                self.penalties += amount;
            }
            return;
        }
        match self.entries.binary_search_by_key(&key, |e| e.key) {
            Ok(i) => {
                self.entries[i].sum += amount;
                self.entries[i].count += 1;
            }
            Err(i) => self.entries.insert(i, CashEntry { key, sum: amount, count: 1 }),
        }
    }

    /// The cash in all: the own totals and every entry.
    pub fn total(&self) -> f32 {
        self.awards + self.penalties + self.entries.iter().map(|e| e.sum).sum::<f32>()
    }
}

/// The campaign's state (see the module notes).
#[derive(Resource, Default)]
pub struct Campaign {
    /// the current slot's game flags (the session's vtable +0x210): default-gameflags at the first
    /// mission, carried on from level to level
    pub flags: GameFlags,
    /// the flags have been made (from the defaults)
    started: bool,
    /// the level whose squad slots the session last filled (FUN_0007f730 fills them when another
    /// level is asked for); "" none yet
    level: String,
    /// a mission, not a multiplayer map (the session's level kind 0x17506391, which
    /// FUN_000814b0 and the campaign mode need)
    pub in_mission: bool,
    /// the mission's objectives (campaign-bf's for the level), for the objectives menu
    pub objectives: Vec<Objective>,
    /// the campaign squad's cash on this level
    pub cash: Cash,
    /// the difficulty, n (`damage::Difficulty`): the cash's scaling
    pub difficulty: i64,
    /// the controlled character's type (the session's player slots, +0x130: a member a player
    /// controls is in the squad, 0x83ee0)
    pub player_kind: u32,
    /// BF_FLAG_LOG is set
    log: bool,
    /// BF_CASH_LOG is set
    cash_log: bool,
    /// names for the logs (FLAG_NAMES, the mission ids, the character types)
    names: HashMap<u32, String>,
    /// the characters seen this level and whether each was dead (`deaths`)
    seen: HashMap<Who, bool>,
    /// a new campaign asked for by the front end (JOIN CAMPAIGN's START, play_menu.rs), with the
    /// difficulty SELECT DIFFICULTY gave it (n: -4, 0, 4): the next level makes its flags anew
    /// from the defaults and writes `campaign/difficulty-setting` (the session's +0x230, 0x7c750)
    new_campaign: Option<i64>,
}

impl Campaign {
    /// A new campaign (the front end's JOIN CAMPAIGN, #226) at difficulty `n`: the next level
    /// starts it (`begin_level`).
    pub fn start_new(&mut self, n: i64) {
        self.new_campaign = Some(n);
    }

    /// A flag path for the logs: its parts' names joined by `/`.
    pub fn path_name(&self, path: &[u32]) -> String {
        path.iter().map(|n| self.names.get(n).cloned().unwrap_or_else(|| format!("h_{n:08x}"))).collect::<Vec<_>>().join("/")
    }

    /// A type's name for the logs: its own (tex) or its key, with its display name.
    fn type_name(&self, kind: u32) -> String {
        self.names.get(&kind).cloned().unwrap_or_else(|| format!("h_{kind:08x}"))
    }

    /// Writes `v` to the flag at `path` (FUN_00077eb0), logged with `why` at time `t`: its value
    /// before and after, None without such a flag (nothing is written then).
    pub fn write(&mut self, path: &[u32], v: f32, why: &str, t: f32) -> Option<(FlagValue, FlagValue)> {
        let r = self.flags.write(path, v);
        if self.log {
            match r {
                Some((old, new)) => println!("t {t:.2}: flag {} {old} -> {new} ({why})", self.path_name(path)),
                None => println!("t {t:.2}: flag {}: no such flag, nothing written ({why})", self.path_name(path)),
            }
        }
        r
    }

    /// Writes the whole number `v` to the flag at `path` (`GameFlags::write_int`), logged with
    /// `why`.
    fn write_int(&mut self, path: &[u32], v: i32, why: &str) {
        let r = self.flags.write_int(path, v);
        if self.log {
            match r {
                Some((old, new)) => println!("flag {} {old} -> {new} ({why})", self.path_name(path)),
                None => println!("flag {}: no such flag, nothing written ({why})", self.path_name(path)),
            }
        }
    }

    /// Sets the flag at `path` back to its default, logged with `why`.
    fn reset(&mut self, path: &[u32], why: &str) {
        if let (Some((old, new)), true) = (self.flags.reset(path), self.log) {
            if old != new {
                println!("flag {} {old} -> {new} ({why})", self.path_name(path));
            }
        }
    }

    /// The session's +0xbc (FUN_000814b0): in a mission, a character type's member flag
    /// `<type>/h_03761692` set to `v` (written as 1 or 0), logged with `why`.
    pub fn set_member(&mut self, kind: u32, v: bool, why: &str, t: f32) {
        if self.in_mission {
            self.write(&[kind, H_MEMBER], if v { 1.0 } else { 0.0 }, why, t);
        }
    }

    /// The path of save-game slot `n` (1-4) field `field`.
    fn slot(n: usize, field: u32) -> Vec<u32> {
        vec![h(SAVE_GAME[0]), h(SAVE_GAME[1]), h(&format!("character-{n}")), field]
    }

    /// The save-game slot holding type `kind`, 1-4 (0x83d10: the first whose archetype, read as an
    /// int, is the type's name).
    fn slot_of(&self, kind: u32) -> Option<usize> {
        (1..=SLOTS).find(|&n| self.flags.value(&Self::slot(n, h("archetype"))).is_some_and(|v| v.as_int() as u32 == kind))
    }

    /// The session's +0xb8 (0x83ee0): whether a character of type `kind` counts as in the squad:
    /// true when no save-game slot holds its type (the session doesn't track it); else its slot's
    /// alive flag (h_03761692), or, that false, whether a player controls its type.
    pub fn in_squad(&self, kind: u32) -> bool {
        let Some(n) = self.slot_of(kind) else { return true };
        self.flags.value(&Self::slot(n, H_MEMBER)).map_or(true, |v| v.as_bool()) || kind == self.player_kind
    }

    /// Adds `amount` to the cash under `key` (FUN_0005c5e0 / FUN_0005c730), logged with `why`.
    pub fn add_cash(&mut self, key: u32, amount: f32, why: &str, t: f32) {
        self.cash.add(key, amount);
        if self.cash_log {
            let key = if key == 0 { String::new() } else { format!(" [{}]", self.names.get(&key).cloned().unwrap_or_else(|| format!("h_{key:08x}"))) };
            println!("t {t:.2}: cash {amount:+.0}{key} ({why}); total {:.0}: {}", self.cash.total(), self.cash_text());
        }
    }

    /// The cash record for the logs: its totals and entries.
    fn cash_text(&self) -> String {
        let mut parts = vec![format!("awards {:.0}", self.cash.awards), format!("penalties {:.0}", self.cash.penalties)];
        for e in &self.cash.entries {
            parts.push(format!("{} x {} {:+.0}", e.count, self.names.get(&e.key).cloned().unwrap_or_else(|| format!("h_{:08x}", e.key)), e.sum));
        }
        parts.join(", ")
    }

    /// The scaling of a cash amount at this difficulty (score-triggers with h_11819686, the death
    /// awards): 1 + 0.1 x n.
    pub fn cash_scale(&self) -> f32 {
        1.0 + CASH_STEP * self.difficulty as f32
    }

    /// One line of the objectives menu (FUN_000f8310): none when the objective's `enabled` flag
    /// is false or missing; else its text, and " (completed)" when its `state` is 1, " (failed)"
    /// when -1.
    pub fn objective_line(&self, o: &Objective, game: &Game) -> Option<String> {
        let enabled = self.flags.value(&[o.path.as_slice(), &[h("enabled")]].concat())?;
        if !enabled.as_bool() {
            return None;
        }
        let mut line = game.strings.get(&o.text).cloned().unwrap_or_else(|| format!("h_{:08x}", o.text));
        let suffix = match self.flags.value(&[o.path.as_slice(), &[h("state")]].concat()).map(|v| v.as_int()) {
            Some(-1) => Some(FAILED),
            Some(1) => Some(COMPLETED),
            _ => None,
        };
        if let Some(s) = suffix.and_then(|s| game.strings.get(&s)) {
            line.push(' ');
            line.push_str(s);
        }
        Some(line)
    }

    /// The names the logs use (FLAG_NAMES, m00-m19, the loaded character types').
    fn make_names(&mut self, game: &Game) {
        for n in FLAG_NAMES {
            self.names.insert(h(n), n.to_string());
        }
        for k in 0..20 {
            let m = format!("m{k:02}");
            self.names.insert(h(&m), m);
        }
        for k in 1..=CASH_ENTRY_COUNT {
            let e = format!("entry-{k}");
            self.names.insert(h(&e), e);
        }
        for (&k, t) in &game.character_types {
            let label = game.strings.get(&t.label).cloned();
            let own = self.names.get(&k).cloned();
            let name = match (own, label) {
                (Some(o), _) => o,
                (None, Some(l)) => format!("h_{k:08x} ({l})"),
                (None, None) => format!("h_{k:08x}"),
            };
            self.names.insert(k, name);
            if let Some(l) = game.strings.get(&t.label) {
                self.names.entry(t.label).or_insert_with(|| l.clone());
            }
        }
    }
}

/// A map starts (after `setup`, before the triggers' first update): the flags made from the
/// defaults the first time (or for a new campaign the front end started, with its difficulty
/// written), then in a mission, as the game goes about a level:
///  * the difficulty the session's +0x234 reads, `campaign/difficulty-setting` (0x7c720: -4
///    STANDARD by the defaults; BF_DIFFICULTY overrides it), into `damage::Difficulty`;
///  * another level than the last: the session's +0xb4 (see the module notes);
///  * the level's squad-type characters set up (FUN_0012f440: each placed one, and the demo's
///    squad members in the world), their member flags 1;
///  * each spawn-trigger with h_0fa4f510 and a squad-type character template linked
///    (FUN_00209890): the type's member flag set to whether the session counts it in the squad;
///  * BF_FLAG_SET;
///  * the cash record new (the campaign mode is made with the level, FUN_000d6490).
#[allow(clippy::too_many_arguments)]
pub fn begin_level(mut campaign: ResMut<Campaign>, game: Res<GameData>, map: Res<MapLevel>, current: Res<CurrentMap>, player: Res<Player>,
                   squad: Res<Squad>, mut difficulty: ResMut<damage::Difficulty>) {
    let c = &mut *campaign;
    let game = &game.0;
    c.log = std::env::var("BF_FLAG_LOG").is_ok();
    c.cash_log = std::env::var("BF_CASH_LOG").is_ok();
    if c.names.is_empty() {
        c.make_names(game);
    }
    let new_campaign = c.new_campaign.take();
    if !c.started || new_campaign.is_some() {
        c.started = true;
        c.flags = game.default_flags.clone();
        // (a new campaign's first level is another level than the last: its squad slots filled)
        c.level.clear();
        if c.log {
            println!("flags: a new campaign's, from {}.xmb ({} flags)", game.missions.flags_file, c.flags.records(&[]).len());
        }
        if let Some(n) = new_campaign {
            // the session's +0x230 (0x7c750): SELECT DIFFICULTY's choice
            c.write_int(&[h("campaign"), h("difficulty-setting")], n as i32, "0x7c750: SELECT DIFFICULTY");
        }
    }
    c.in_mission = map.0.is_some() && !damage::multiplayer_map(game, &current.0) && !c.flags.is_empty();
    // the session's +0x234 (0x7c720): the difficulty is the slot's flag, in a mission
    if c.in_mission && std::env::var("BF_DIFFICULTY").is_err() {
        if let Some(n) = c.flags.value(&[h("campaign"), h("difficulty-setting")]).map(|v| v.as_int() as i64).filter(|&n| n != difficulty.n) {
            difficulty.n = n;
            difficulty.name = match n { -4 => "STANDARD", 0 => "HARD", 4 => "BRUTAL", _ => "set" };
        }
    }
    c.cash = Cash::default();
    c.seen.clear();
    c.difficulty = difficulty.n;
    c.player_kind = player.kind;
    let mission = game.missions.mission_of(&current.0);
    c.objectives = mission.map(|(m, _)| m.objectives.clone()).unwrap_or_default();
    let Some(level) = map.0.as_ref().filter(|_| c.in_mission) else { return };
    // another level: the session's +0xb4 (0x841e0)
    if c.level != current.0.to_lowercase() {
        c.level = current.0.to_lowercase();
        // FUN_00083fb0: each slot's archetype, alive, current-weapon, h_f02f13eb, h_10323f44
        for n in 1..=SLOTS {
            for f in SLOT_RESETS {
                c.reset(&Campaign::slot(n, f), "FUN_00083fb0: the save-game squad cleared");
            }
        }
        // FUN_000846b0: the save-game cash (logged where it changes something)
        let cash = [h(SAVE_GAME[0]), h(SAVE_GAME[1]), h("cash")];
        let fields: Vec<Vec<u32>> = CASH_TOTALS.iter().map(|&f| [&cash[..], &[f]].concat())
            .chain((1..=CASH_ENTRY_COUNT).flat_map(|e| CASH_ENTRY_FIELDS.map(|f| [&cash[..], &[CASH_ENTRIES, h(&format!("entry-{e}")), f]].concat())))
            .chain(SAVE_GAME_CLEARED.iter().map(|&f| vec![h(SAVE_GAME[0]), h(SAVE_GAME[1]), f]))
            .collect();
        for path in fields {
            if let (Some((old, new)), true) = (c.flags.write(&path, 0.0), c.log) {
                if old != new {
                    println!("flag {} {old} -> {new} (FUN_000846b0: the save-game cash cleared)", c.path_name(&path));
                }
            }
        }
        // FUN_00084230: the zone's squad members alive (+0xec), then the extras in the places of
        // those it lacks (+0xf0), alive if a player controls them
        let (present, missing): (Vec<u32>, usize) = match mission {
            Some((_, zone)) => (game.missions.squad.iter().copied().filter(|k| !zone.unavailable.contains(k) && game.character_type(*k).is_some()).collect(),
                                zone.unavailable.len()),
            None => (vec![], 0),
        };
        let extras: Vec<u32> = game.missions.extras.iter().copied().take(missing).collect();
        if c.log {
            let names = |l: &[u32]| l.iter().map(|&k| c.type_name(k)).collect::<Vec<_>>().join(", ");
            println!("campaign squad {}; extras {}; zone {} lacks {}", names(&game.missions.squad), names(&game.missions.extras), current.0,
                     mission.map_or(String::new(), |(_, z)| names(&z.unavailable)));
        }
        let controlled = c.player_kind;
        let fills: Vec<(u32, bool)> = present.iter().map(|&k| (k, true)).chain(extras.iter().map(|&k| (k, k == controlled))).collect();
        for (n, (kind, alive)) in (1..=SLOTS).zip(fills) {
            let why = format!("FUN_00084230: slot {n}, {}", c.type_name(kind));
            // (the type's name in an int record, as its bits: `slot_of` reads it back)
            c.write_int(&Campaign::slot(n, h("archetype")), kind as i32, &why);
            c.write(&Campaign::slot(n, H_MEMBER), if alive { 1.0 } else { 0.0 }, &why, 0.0);
            c.write(&Campaign::slot(n, CURRENT_WEAPON), 0.0, &why, 0.0);
        }
    }
    // FUN_0012f440: the level's characters set up as it loads (each placed one; and the demo's
    // squad members in the world, the session's own), a squad type's member flag 1
    let squad_type = |kind: u32| game.character_type(kind).is_some_and(|t| t.player_type);
    for o in level.objects.iter().filter(|o| o.tag == h("character-object") && squad_type(o.kind)) {
        c.set_member(o.kind, true, &format!("FUN_0012f440: placed {} set up", c.path_name(&[o.name])), 0.0);
    }
    for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.stand_in && u.in_squad() && squad_type(u.kind)) {
        c.set_member(u.kind, true, &format!("FUN_0012f440: the squad's {} set up", unit_name(u)), 0.0);
    }
    // FUN_00209890: each spawn-trigger with h_0fa4f510 whose template is a squad-type character,
    // linked in the file's order
    for t in &level.triggers {
        let Class::Spawn { template, member_check: true, .. } = t.class else { continue };
        let Some(o) = level.objects.iter().find(|o| o.name == template && o.tag == h("character-object")).filter(|o| squad_type(o.kind)) else { continue };
        let kept = c.in_squad(o.kind);
        let why = format!("FUN_00209890: spawn-trigger h_{:08x}'s template {}, {} the squad", t.name, c.path_name(&[template]),
                          if kept { "in" } else { "not in" });
        c.set_member(o.kind, kept, &why, 0.0);
    }
    // test hook: BF_FLAG_SET
    if let Ok(v) = std::env::var("BF_FLAG_SET") {
        for item in v.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let Some((path, value)) = item.rsplit_once('=') else { continue };
            let path: Vec<u32> = path.split('/').map(str::trim).filter(|s| !s.is_empty())
                .map(|p| p.strip_prefix("h_").and_then(|x| u32::from_str_radix(x, 16).ok()).unwrap_or_else(|| h(p))).collect();
            let v = match value.trim() { "true" => Some(1.0), "false" => Some(0.0), s => s.parse::<f32>().ok() };
            match v {
                Some(v) => {
                    let r = c.flags.write(&path, v);
                    println!("BF_FLAG_SET: {} {}", c.path_name(&path), r.map_or("isn't a flag: nothing written".to_string(), |(o, n)| format!("{o} -> {n}")));
                }
                None => println!("BF_FLAG_SET: {item}: the value isn't a number or true / false"),
            }
        }
    }
    if c.log {
        for n in 1..=SLOTS {
            let kind = c.flags.value(&Campaign::slot(n, h("archetype"))).map_or(0, |v| v.as_int() as u32);
            let alive = c.flags.value(&Campaign::slot(n, H_MEMBER)).is_some_and(|v| v.as_bool());
            if kind != 0 {
                println!("save-game slot {n}: {}, {}", c.type_name(kind), if alive { "alive" } else { "not alive" });
            }
        }
        let mut members: Vec<String> = c.flags.records(&[]).into_iter().filter(|(p, _, _)| p.len() == 2 && p[1] == H_MEMBER)
            .map(|(p, v, _)| format!("{} {}", c.type_name(p[0]), v.as_int())).collect();
        members.sort();
        println!("member flags: {}", members.join(", "));
        for o in &c.objectives {
            let state = c.flags.value(&[o.path.as_slice(), &[h("state")]].concat());
            let enabled = c.flags.value(&[o.path.as_slice(), &[h("enabled")]].concat());
            println!("objective {} ({}, ${}): enabled {}, state {}", c.path_name(&o.path), if o.primary { "primary" } else { "secondary" }, o.cash,
                     enabled.map_or("-".into(), |v| v.to_string()), state.map_or("-".into(), |v| v.to_string()));
        }
        if std::env::var("BF_FLAG_LOG").is_ok_and(|v| v == "all") {
            for (p, v, d) in c.flags.records(&[]) {
                println!("  {} = {v} (default {d})", c.path_name(&p));
            }
        }
    }
}

/// Each frame after the characters' update: a character that has died (its +0x144(1)) or come
/// back to life (+0x144(0)): FUN_00112830 sets a squad type's member flag to whether it lives,
/// and at a death FUN_00157070 pays its type's award. Characters are told apart by `Who`; one
/// seen for the first time is only noted.
fn deaths(mut campaign: ResMut<Campaign>, game: Res<GameData>, player: Res<Player>, squad: Res<Squad>) {
    let c = &mut *campaign;
    if !c.in_mission {
        return;
    }
    for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.stand_in) {
        let was = c.seen.insert(u.who(), u.dead);
        if was.is_none_or(|d| d == u.dead) {
            continue;
        }
        let t = player.sim_time;
        // FUN_00115d50: a death of the campaign squad's (the demo's squad, not an NPC) counts in
        // its record's +0x20, "Operatives Lost", unless a cutscene has the world
        if u.dead && u.in_squad() && !script::in_cutscene() {
            c.cash.lost += 1;
            if c.cash_log {
                println!("t {t:.2}: operatives lost {} ({} died)", c.cash.lost, unit_name(u));
            }
        }
        let Some(ty) = game.0.character_type(u.kind) else { continue };
        if ty.player_type {
            let why = format!("FUN_00112830: {} {}", unit_name(u), if u.dead { "died" } else { "came back" });
            c.set_member(u.kind, !u.dead, &why, t);
        }
        if u.dead {
            award(c, &game.0, u, t);
        }
    }
}

/// FUN_00157070 at a character's death, in a campaign: its type's award (`Award`) x (its max
/// hit points with h_ffc62853) x 1 + 0.1 x the difficulty, keyed by the type's display name, to
/// the campaign squad (h_f61ecbaf: whoever killed it) and, for one on the squad's side, to its
/// own squad (h_199ff965). Both are the same record here.
fn award(c: &mut Campaign, game: &Game, u: &Player, t: f32) {
    let Some(ty) = game.character_type(u.kind) else { return };
    let a = ty.award;
    if a.amount == 0.0 || !(a.to_squad || a.to_own) {
        return;
    }
    let base = if a.per_damage { a.amount * u.max_health } else { a.amount };
    let amount = base * c.cash_scale();
    let key = if ty.label != 0 { ty.label } else { NO_NICE_NAME };
    let what = format!("{} died: {} x {:.2}", unit_name(u), a.amount, c.cash_scale());
    if a.to_squad {
        c.add_cash(key, amount, &format!("FUN_00157070: {what}, to the squad"), t);
    }
    if a.to_own && u.team == SQUAD_SIDE && !damage::hostile(u.team, u.team) {
        c.add_cash(key, amount, &format!("FUN_00157070: {what}, to its own squad"), t);
    }
}
