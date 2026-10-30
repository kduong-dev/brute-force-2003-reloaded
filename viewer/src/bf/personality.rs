//! AI personalities (#129): `common/ai/personality.xmb`, what a character's brain does on each
//! event. A character's personality is its placement's `<ai personality-freewill>` (char+0x1bc,
//! FUN_00110cd0), else its type's (`TypeAi::personalities`). Each personality names a parent
//! (h_f38c9694) whose table it inherits, and its own `<table>` of event entries:
//!
//!   <personality name="default" h_f38c9694="h_ff1e6600">
//!     <table>
//!       <h_143df498 event="15" h_e146dc86="21" h_f167afaa="86.86">     EVT_DAMAGE_HEAVY, priority 21
//!         <h_04701700 goal="65" h_0054cf83="1"/>                        GOAL_KNOCKDOWN, weight 1
//!         <h_04701700 goal="63" h_0054cf83="1"/>                        GOAL_STUN, weight 1
//!         <h_04701700 goal="86" h_0054cf83="3"/>                        GOAL_NOOP, weight 3
//!
//! The entry's comment in the data says how a goal is drawn: "Randomize what happens when
//! damaged. The goal-pris are SUMmed and then a rand number is made from 0 to SUM. Goal chosen
//! base on this" (`EventEntry::draw`). The event and goal numbers are default.xbe's EVT_ and
//! GOAL_ name tables (0x3bdb9c, 0x3bcbb4: code map "AI"). The brain's goal machine that runs
//! them is FUN_000377b0.
//!
//! Names: `name`, `event` and `goal` are the attribute names; h_f38c9694 is read as the parent
//! (every personality names one that exists, or none at the root), h_e146dc86 the priority (the
//! squad's EVT_SHOT_AT 20.15, docs/squad.md), h_0054cf83 a goal's weight (the comment above);
//! h_f167afaa the priority's decay a second (exactly 86.86, and 0, don't decay: the AI step
//! FUN_0001b4b0), `<h_e5d8ff78><spawn-events event>` the events an entry posts in turn
//! (FUN_0003a200), and `<h_f48f8744><h_edf09800 quick-goal float_value goal>` its quick goals:
//! settings the entry makes on the brain as it's taken (FUN_0003a4a0, by the quick goal's hash:
//! `QuickGoal`). The brain builds its runtime entries with FUN_0004ece0 (todo/129-shrikes
//! answers.md, Q1).
//!
//! A squad member's personality follows its command mode (#318, the brain's PCMD_ mode at
//! brain+0x58: the type's `<ai>` names one per mode, `TypeAi::personalities`): the four's follow
//! h_169b2725, guard h_155e52d6, moveout h_0f69b8b2, and the wingman h_ea20a17d their empty
//! wingman ones inherit.

use std::collections::HashMap;

use super::bxml::Element;
use super::character::Game;
use super::hash::h;

/// EVT_DAMAGE_LIGHT (14) and EVT_DAMAGE_HEAVY (15): the events a damaged character's brain gets
/// (the EVT_ name table 0x3bdb9c), each personality's damage entries.
pub const EVT_DAMAGE_LIGHT: i64 = 14;
pub const EVT_DAMAGE_HEAVY: i64 = 15;
/// GOAL_STUN (63), GOAL_KNOCKDOWN (65) and GOAL_NOOP (86): the goals the damage entries draw
/// (the GOAL_ name table 0x3bcbb4).
pub const GOAL_STUN: i64 = 63;
pub const GOAL_KNOCKDOWN: i64 = 65;
pub const GOAL_NOOP: i64 = 86;

/// The parent attribute (h_f38c9694: every personality has one but the root's), an event entry
/// (h_143df498), its priority (h_e146dc86) and the priority's decay a second (h_f167afaa:
/// FUN_0001b4b0, exactly 86.86 and 0 don't decay), a goal of it (h_04701700) and the goal's
/// weight (h_0054cf83).
const H_PARENT: u32 = 0xF38C_9694;
const H_EVENT: u32 = 0x143D_F498;
const H_PRIORITY: u32 = 0xE146_DC86;
const H_F167AFAA: u32 = 0xF167_AFAA;
const H_GOAL: u32 = 0x0470_1700;
const H_WEIGHT: u32 = 0x0054_CF83;
/// An entry's list of events it posts (`<h_e5d8ff78><spawn-events event>`).
const H_SPAWNS: u32 = 0xE5D8_FF78;
/// EVT_DAMAGED_NO_ENEMIES (30): what a hit posts instead of the damage events when the brain knows
/// no enemy (answers.md Q1); `default` and h_0bb586cf have it post EVT_USER2 (70, "Involuntary
/// response": KNOCKDOWN 1 : STUN 1 at priority 100).
pub const EVT_DAMAGED_NO_ENEMIES: i64 = 30;
pub const EVT_USER2: i64 = 70;
/// The events that grant a squad member combat (the EVT_ name table 0x3bdb98): EVT_DAMAGED (13),
/// EVT_SHOT_AT (24) and EVT_SQUAD_MATE_FIRING (44, posted by FUN_00045530 to every other squad
/// member within its type's h_e3d8aeef of a squad-type character firing). The follow, guard and
/// moveout personalities map each to GOAL_NOOP (or a dodge) with the quick goal QG_ALLOW_COMBAT
/// ("allow combat - only if mate is within radius", the data's note on 44).
pub const EVT_DAMAGED: i64 = 13;
pub const EVT_SHOT_AT: i64 = 24;
pub const EVT_SQUAD_MATE_FIRING: i64 = 44;
/// EVT_SQUAD_IMA_TARGET (43): a hostile brain's EVT_ENEMY_SEEN handling (its message handler's
/// case 0x3e45a-0x3e49b) posts it to the character it has as its target when that's of a squad
/// type; the follow, guard and moveout personalities grant combat on it too.
pub const EVT_SQUAD_IMA_TARGET: i64 = 43;
/// EVT_IDLE_VARY_DATA (63): GOAL_IDLE_VARY's draw of its next pause: IDLE_VARY_DATA_PAUSE (97),
/// _PAUSE_CAUTIOUS (98), _PAUSE_CROUCH (99), each with its length in a QG_IDLE_TIME quick goal.
pub const EVT_IDLE_VARY_DATA: i64 = 63;
pub const GOAL_PAUSE: i64 = 97;
pub const GOAL_PAUSE_CAUTIOUS: i64 = 98;
pub const GOAL_PAUSE_CROUCH: i64 = 99;
/// The quick goals (FUN_0003a4a0, by the hash in `quick-goal`): h_f0233972 sets allow_combat
/// (brain+0x464 bit 0) and starts its 10 s timer at brain+0x468 (0x3a744); h_ea874132 sets
/// allow_cover (bit 1, 0x3a64e); h_ed46a645 gives an idle pause (its `goal`) its length (its
/// `float_value`, s: GOAL_IDLE_VARY's draw, not handled by FUN_0003a4a0).
pub const QG_ALLOW_COMBAT: u32 = 0xF023_3972;
pub const QG_ALLOW_COVER: u32 = 0xEA87_4132;
pub const QG_IDLE_TIME: u32 = 0xED46_A645;
/// An entry's quick goal list (h_f48f8744) and a quick goal of it (h_edf09800).
const H_QUICK_GOALS: u32 = 0xF48F_8744;
const H_QUICK_GOAL: u32 = 0xEDF0_9800;

/// A personality (see the module notes).
#[derive(Clone, Debug, Default)]
pub struct Personality {
    /// its parent personality (h_f38c9694), whose entries it inherits; 0 none
    pub parent: u32,
    /// its own event entries, in the file's order
    pub events: Vec<EventEntry>,
}

/// One event entry of a personality's table.
#[derive(Clone, Debug, Default)]
pub struct EventEntry {
    /// the EVT_ number
    pub event: i64,
    /// its priority (h_e146dc86): the damage entries 21 (heavy), and for the light one -20.2 in
    /// `default` ("TESTING"), 20.8 in h_0bb586cf
    pub priority: f32,
    /// h_f167afaa: the priority's decay a second (86.86 on the damage entries: no decay)
    pub h_f167afaa: f32,
    /// its goals: (the GOAL_ number, weight)
    pub goals: Vec<(i64, f32)>,
    /// the events it posts in turn (`spawn-events`), in the file's order
    pub spawns: Vec<i64>,
    /// its quick goals, in the file's order
    pub quick: Vec<QuickGoal>,
}

/// A quick goal of an event entry (`<h_edf09800 quick-goal float_value goal>`): a setting the
/// entry makes on the brain (FUN_0003a4a0 by `name`: QG_ALLOW_COMBAT, QG_ALLOW_COVER ...), with
/// its value and the goal it's for when it has them (QG_IDLE_TIME's pause lengths).
#[derive(Clone, Copy, Debug, Default)]
pub struct QuickGoal {
    /// the `quick-goal` hash
    pub name: u32,
    /// `float_value`
    pub value: Option<f32>,
    /// `goal`: the GOAL_ number it's for
    pub goal: Option<i64>,
}

impl EventEntry {
    /// Whether it has the quick goal `name`.
    pub fn has_quick(&self, name: u32) -> bool {
        self.quick.iter().any(|q| q.name == name)
    }

    /// The idle pauses of an EVT_IDLE_VARY_DATA entry: each goal with its weight and its length (s,
    /// its QG_IDLE_TIME quick goal's value; 0 when none gives one).
    pub fn pauses(&self) -> Vec<(i64, f32, f32)> {
        self.goals.iter().map(|&(g, w)| {
            let len = self.quick.iter().find(|q| q.name == QG_IDLE_TIME && q.goal == Some(g)).and_then(|q| q.value).unwrap_or(0.0);
            (g, w, len)
        }).collect()
    }

    /// A goal drawn as the data's comment says, and as FUN_00054c40 does: the weights summed per
    /// goal, a number `u` (uniform in 0..1) times the sum, and walking the goal ids upward the first
    /// whose running total reaches it; with every weight 0, the first goal (the data's fallbacks).
    /// The goals in `out` are struck out first (a refused start, FUN_00054bc0). None if nothing
    /// is left.
    pub fn draw(&self, u: f32, out: &[i64]) -> Option<i64> {
        let mut goals: Vec<(i64, f32)> = vec![];
        for &(g, w) in self.goals.iter().filter(|(g, _)| !out.contains(g)) {
            match goals.iter_mut().find(|x| x.0 == g) {
                Some(x) => x.1 += w.max(0.0),
                None => goals.push((g, w.max(0.0))),
            }
        }
        goals.sort_by_key(|x| x.0);
        let sum: f32 = goals.iter().map(|g| g.1).sum();
        if sum <= 0.0 {
            return goals.first().map(|g| g.0);
        }
        let x = u.clamp(3.6e-7, 0.999_999_6) * sum;
        let mut run = 0.0;
        for &(goal, w) in &goals {
            run += w;
            if x <= run {
                return Some(goal);
            }
        }
        goals.last().map(|g| g.0)
    }
}

/// The personalities of a `personality.xmb` document, by name hash ("default" is the one named
/// in the clear).
pub fn parse_personalities(root: &Element) -> HashMap<u32, Personality> {
    let num = |e: &Element, k: u32| e.attr(k).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(0.0);
    let name = |e: &Element, k: u32| e.attr(k).and_then(|v| v.as_hash()).filter(|&x| x != 0 && x != h("")).unwrap_or(0);
    root.children_named(h("personality")).map(|p| {
        let events = p.child(h("table")).map(|t| t.children_named(H_EVENT).map(|e| EventEntry {
            event: e.attr(h("event")).and_then(|v| v.as_i64()).unwrap_or(-1),
            priority: num(e, H_PRIORITY),
            h_f167afaa: num(e, H_F167AFAA),
            goals: e.children_named(H_GOAL).map(|g| (g.attr(h("goal")).and_then(|v| v.as_i64()).unwrap_or(-1), num(g, H_WEIGHT))).collect(),
            spawns: e.child(H_SPAWNS).map(|s| s.children_named(h("spawn-events")).filter_map(|x| x.attr(h("event")).and_then(|v| v.as_i64())).collect())
                .unwrap_or_default(),
            quick: e.child(H_QUICK_GOALS).map(|l| l.children_named(H_QUICK_GOAL).map(|q| QuickGoal {
                name: name(q, h("quick-goal")),
                value: q.attr(h("float_value")).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))),
                goal: q.attr(h("goal")).and_then(|v| v.as_i64()),
            }).collect()).unwrap_or_default(),
        }).collect()).unwrap_or_default();
        (name(p, h("name")), Personality { parent: name(p, H_PARENT), events })
    }).collect()
}

impl Game {
    /// Personality `name`'s entry for `event`: its own table's, else its parent's, up the chain
    /// (a child's entry for an event replaces its parent's: h_0bb586cf's heavy entry,
    /// KNOCKDOWN only, over the 1 : 1 : 3 its ancestors have; inferred from the data, the lookup
    /// itself isn't traced). None if no personality on the way has one.
    pub fn personality_entry(&self, name: u32, event: i64) -> Option<&EventEntry> {
        let mut at = name;
        // (a chain is a few deep; the bound guards against a loop in the data)
        for _ in 0..32 {
            let p = self.personalities.get(&at)?;
            if let Some(e) = p.events.iter().find(|e| e.event == event) {
                return Some(e);
            }
            at = p.parent;
            if at == 0 {
                return None;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #318: an EVT_IDLE_VARY_DATA entry's pauses take their lengths from its h_ed46a645 quick
    /// goals by goal (the follow personality h_169b2725's: crouch 15 s, cautious 12 s, pause 6 s),
    /// and an entry's quick goals are found by name.
    #[test]
    fn idle_pauses_and_quick_goals() {
        let q = |name, value, goal| QuickGoal { name, value, goal };
        let e = EventEntry {
            event: EVT_IDLE_VARY_DATA,
            goals: vec![(GOAL_PAUSE_CROUCH, 1.0), (GOAL_PAUSE_CAUTIOUS, 1.0), (GOAL_PAUSE, 1.0)],
            quick: vec![q(QG_IDLE_TIME, Some(15.0), Some(99)), q(QG_IDLE_TIME, Some(12.0), Some(98)), q(QG_IDLE_TIME, Some(6.0), Some(97))],
            ..Default::default()
        };
        assert_eq!(e.pauses(), vec![(99, 1.0, 15.0), (98, 1.0, 12.0), (97, 1.0, 6.0)]);
        assert!(e.has_quick(QG_IDLE_TIME));
        assert!(!e.has_quick(QG_ALLOW_COMBAT));
        let grant = EventEntry { event: EVT_SQUAD_MATE_FIRING, quick: vec![q(QG_ALLOW_COMBAT, None, None)], ..Default::default() };
        assert!(grant.has_quick(QG_ALLOW_COMBAT));
        assert!(!grant.has_quick(QG_ALLOW_COVER));
    }
}
