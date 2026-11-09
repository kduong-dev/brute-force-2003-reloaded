//! The corpse eater (#210): CSKILL_CORPSE_EATER 16, the skill of m03_a's and
//! m03_b's Filthy Mutants (h_1662a599, h_0b10f748). In the game it is set dressing the level's
//! script plays; the demo gives it to its Filthy and Ugly Mutant classes as a player skill (the product
//! owner's design, 2026-11-09: demo-own, labelled so in `character_class.rs`).
//!
//! **What the game does** (todo/npc-skills/answer.md §3.3, todo/132-mutants/leads.md §7):
//!  - **The class** (FUN_0013c800, vtable 0x39d408, 0x20 bytes) has one override only: +0x18
//!    0x13c830, the animation-event handler (FUN_00117660 hands every event to it first,
//!    0x11768d). No toggle, no drain, no update, no name (its +0x4c is the base's, which returns 0).
//!  - **The event** e8262ae5 (FUN_0013c850) spawns the `<corpse-eater>` block's effect
//!    (h_e698b665 -> h_1eeb36f7: one ALE effect h_1d563f48, no damage, no light, no sound,
//!    max-life FLT_MAX) attached to the character at the block's hardpoint (h_0e13b0f0,
//!    `LEFT_HAND` h_f8474385), effect vtable +0xdc, flag |= 2, FUN_0021c380(1.0).
//!  - **The times** sit in `Sc_w1_eat_idle`, a 9.13 s loop, at 0.37, 1.70 and 6.13 s. The chain is
//!    MT 37 `eat_start` (1.13 s: the body drops 0.43 m, the kneel), next-motion MT 41 `eat_idle`
//!    (the loop) and MT 40 `eat_stop` (0.83 s, the stand-up). take05 and take13 (todo/132-mutants)
//!    show the kneel, the loop and the red drip at the hand.
//!  - **Nothing heals** in the class, and nothing in the data gives a rate: the ticket's E3 wasn't
//!    filmed either. The stamina cost, 3 a second with 1.5 recovery, is real (the m03 types'
//!    `<combat-skill>`).
//!
//! **What the demo does** (the product owner's design):
//!  - **V next to a body**: he kneels over it (the `eat_start` clip, then the `eat_idle` loop), its
//!    gore effect on his left hand at each of the loop's three events, and feeds.
//!  - **The body is whatever lies at his feet** (`watch_bodies`, `nearest_body`): any dead
//!    character's — the squad's, a test NPC's, a level enemy's, either side, any type — at the point
//!    the corpse is at (its ragdoll's root, `Ragdoll::root`, else where it died), within
//!    CORPSE_REACH **across** and CORPSE_LEVEL **in height** (a body on a ledge below or above isn't
//!    his). The game's script has no check to copy: it plays the motion at a placed mutant.
//!  - **While feeding he regains health**, HEAL_PER_SECOND: a demo value, not the game's (there is
//!    none), and the class's own stamina drain of 3 a second is what limits it (about 33 s from a
//!    full bar; at 0 it fizzles, as the Molotov Mutant's bottle does).
//!  - **Exposed**: no fire (`hands_busy`, the player's `fire` and `aim` cleared; `holds_fire` for
//!    an AI one) and not free to act (`lets_act`: no reload, switch, grenade, medkit, pickup or
//!    crouch key). **V again stands him up** (`eat_stop`) and switches the skill off, the stick's
//!    move and turn held while that clip plays (`SkillKind::holds_still`), so the cancel's
//!    animation finishes before he can walk off. **A full health bar** stands him up too, the press
//!    refused on one (there is nothing left to give; the product owner's ask of 2026-11-09).
//!    **Moving off the body** does the same, and so does the **body's own going** (a blast that
//!    throws it out of the reach, a trigger that removes it), a knock-down, the stamina's 0 or the
//!    death.
//!
//! The extension is `skills::SkillKind`, the same one the Molotov Mutant (#211) uses; the three
//! family dispatch points in `play_effect_skills.rs` (`kind`, `own_type_skill`, `give_defaults`)
//! give it to a type that carries skill-set 16, and `character_class::CORPSE_EATER_DEMO` gives it
//! to the demo's Filthy and Ugly Mutants. The clips the pose needs are `play.rs`'s `Loaded::eat_clips`
//! (MT 37 / 41 / 40, read from the type's motion set); the skill only times them, `feeding`.
//!
//! Test hooks: `BF_TEST_SQUAD_SLOT="0.3,me,mp_mutant_filthy"` (or `BF_TEST_SQUAD_PRESET="0.3,
//! mutant_horde"` and switch to the Filthy Mutant), `BF_TEST_SKILL=<s>` for the V presses,
//! `BF_TEST_NPC` for a body to kneel over (kill it first: any side's), `BF_SKILL_LOG=1` for the
//! presses, the chain, the gore, the heal and the body found or lost.
//!
//! **Not verified in play**: the gore needs a map whose archives define h_1eeb36f7 (m03_a's or
//! m03_b's objecttypes; `load_gore` says when this map's don't) and no capture of the chain's frames
//! has been taken yet (see docs/skills/corpse-eater.md).

use super::*;
use super::skills::{Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{CombatSkills, CorpseEaterSkill};
use bf_viewer::ale_fx::{AleAssets, AleEffect, Compiled};

/// The Filthy Mutants' skill-set (CSKILL_CORPSE_EATER, default.xbe's name table 0x3bd814; the
/// factory FUN_0012e590 makes it with FUN_0013c800, vtable 0x39d408).
pub const CSKILL_CORPSE_EATER: i64 = 16;
/// The m03 Filthy Mutants' cost and recovery (`<combat-skill skill-set="16" h_105c13aa="3"
/// h_f2a2beb9="1.5">`, objecttypes-m03_a.xml's h_1662a599 and m03_b's h_0b10f748), which the demo's
/// class keeps and BF_TEST_GIVE_SKILL=16 gives by default.
pub const MUTANT_COST: f32 = 3.0;
pub const MUTANT_RECOVERY: f32 = 1.5;
/// The HUD's caption while it feeds, *Feast*: the demo's own (`NAME`'s id is the demo's, outside
/// the string table's hash space, and `load_gore` puts the text in `Game::strings` as the demo's
/// class names are). The game's class names nothing (its +0x4c is the base's, 0x2bf0a0, which
/// returns 0) and the table has no corpse- or eating-word at all, so the caption is a demo pick like
/// the Molotov Mutant's "Self-Destruct" (h_01fdf1ca): demo-own, the product owner's choice (2026-11-09:
/// `Feast`, over the first pick `Scavenge` and the table's `Meatgrinder`).
/// How far across from a body's own point the press finds it, m: the demo's reach, as the game's
/// script has no check to copy (it plays the motion at a placed mutant). Measured in the ground's
/// plane from the character's feet: the body is what he stands over, whichever way he faces.
const CORPSE_REACH: f32 = 2.2;
/// How far above or below his feet a body's own point may lie and still be his, m: a corpse lies on
/// the ground, so what counts is the same floor (a body on a ledge below or above isn't his), not the
/// height its point is at (a corpse's root point is near the floor, a standing body's at the hips).
const CORPSE_LEVEL: f32 = 1.8;
/// What feeding gives back, health a second. A demo value: the class holds no heal and the data no
/// rate (#210's E3 was never filmed). The stamina's 3 a second is what caps a feed.
const HEAL_PER_SECOND: f32 = 6.0;
/// The three clips' lengths, s (`BF_ANIM_PROBE`, todo/132-mutants/leads.md §7): `Sc_w1_eat_start`
/// 1.13, `Sc_w1_eat_idle` 9.13 (the loop) and `Sc_w1_eat_stop` 0.83.
const START: f32 = 1.13;
const LOOP: f32 = 9.13;
const STOP: f32 = 0.83;
/// The animation event the class answers (0x13c830: `cmp` against 0xe8262ae5) and the three times it
/// sits at in `eat_idle`, s into the loop.
/// The caption's id: `Feast`'s in `Game::strings`, the demo's own (its text put there by
/// `load_gore`; no string table has it, the table's ids being the hashes of their text).
const NAME: u32 = 0xFF00_0001;
const EV_GORE: u32 = 0xE826_2AE5;
const GORE_AT: [f32; 3] = [0.37, 1.70, 6.13];
/// How far the character may drift from where he knelt before the feed stops, m (moving off the body
/// ends it, the product owner's design).
const MOVED: f32 = 0.02;

/// Where a dead character's body is now: its ragdoll's root once the death has built one (0x1271a0
/// writes it into the character's node each step, the point the death camera, the blood pool and the
/// memory chip go by: `Ragdoll::root`), else where it died (a body with no ragdoll yet, or one whose
/// death never made one: `missile::root_point`, the node the game's own attachments and blasts
/// read). Not the character's `position`, which stops at the death.
fn body_point(u: &Player) -> Vec3 {
    u.ragdoll.as_ref().map_or_else(|| missile::root_point(u), |r| r.root())
}

/// The nearest of `bodies` (their own points) to a character standing at `at` (his feet): the one
/// the press finds there. Across the ground's plane within CORPSE_REACH, and of the same floor
/// (CORPSE_LEVEL up or down): a body at his feet counts whichever way he faces, and one below a ledge
/// or above a balcony doesn't.
fn nearest_body(at: Vec3, bodies: impl Iterator<Item = Vec3>) -> Option<Vec3> {
    let across = |p: &Vec3| (p.x - at.x).powi(2) + (p.z - at.z).powi(2);
    bodies
        .filter(|p| across(p) <= CORPSE_REACH * CORPSE_REACH && (p.y - at.y).abs() <= CORPSE_LEVEL)
        .min_by(|a, b| across(a).total_cmp(&across(b)))
}

/// The skill object of a skill-set 16 `<combat-skill>`'s character, with game-options' `<corpse-eater>`
/// block (play_effect_skills.rs `kind`'s arm for the class).
pub fn kind(settings: &CombatSkills) -> Box<dyn SkillKind> {
    Box::new(CorpseEater { s: settings.corpse_eater, ..Default::default() })
}

/// Where the feed is: off, the kneel (the start clip), the loop (feeding, the events) or the
/// stand-up (the stop clip; the skill switches off when it ends).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Off,
    Start,
    Idle,
    Stop,
}

/// The corpse eater (FUN_0013c800, vtable 0x39d408): see the module notes.
#[derive(Default)]
struct CorpseEater {
    /// game-options' block, as the constructor copies it
    s: CorpseEaterSkill,
    phase: Phase,
    /// seconds into the clip under way
    clock: f32,
    /// which of the loop's gore events the last frame was before (0-3: 3, all of them)
    next: usize,
    /// where he knelt: moving off it ends the feed
    at: Vec3,
    /// the stop clip has finished: the toggle's next answer is the off
    ended: bool,
}

impl CorpseEater {
    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: corpse eater refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }

    /// Whether an overlay is under way that a toggle waits for (the demo's one overlay at a time:
    /// a reload, a switch, a throw, a medkit, a weapon pickup or a skill's toggle).
    fn busy(u: &Player) -> bool {
        u.reloading.is_some() || u.throwing.is_some() || u.using.is_some() || u.switching.is_some()
            || u.gun_use.is_some() || u.skill.toggling.is_some()
    }

    /// Stand him up: the stop clip, then the off (`update` asks the command for it at its end).
    fn stand(&mut self, u: &Player, why: &str) {
        if self.phase == Phase::Stop {
            return;
        }
        self.phase = Phase::Stop;
        self.clock = 0.0;
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: corpse eater: {why}, standing up ({STOP} s)", u.sim_time, unit_name(u));
        }
    }
}

impl SkillKind for CorpseEater {
    /// The game's class has no name (its +0x4c is the base's, 0x2bf0a0, which returns 0), so the HUD
    /// shows none; the demo keeps it that way rather than borrow another skill's.
    fn name(&self) -> u32 {
        NAME
    }

    /// **Off**: a dead body (`Skill::corpse`, `watch_bodies`), health below its max (a full bar has
    /// nothing to feed for), no overlay under way, not knocked down, and the class's stamina cost.
    /// **On**: the press stands him up (`Stays`: the stop clip is asked for, the skill stays on
    /// until it ends); in the stop clip a press is ignored; once the clip has ended the answer is
    /// the off (`Now`).
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on {
            if self.phase == Phase::Stop {
                if self.ended {
                    self.ended = false;
                    return Toggle::Now { overlay: false };
                }
                return Toggle::Stays;
            }
            self.stand(u, "the press");
            return Toggle::Stays;
        }
        if u.skill.corpse.is_none() {
            return Self::refused(u, out, &format!("no body within {CORPSE_REACH} m across and {CORPSE_LEVEL} m of his floor"));
        }
        if u.health >= u.max_health {
            return Self::refused(u, out, "his health is full");
        }
        if knocked_down(u) || Self::busy(u) {
            return Self::refused(u, out, "busy or knocked down");
        }
        if !skills::can_activate(u) {
            return Self::refused(u, out, &format!("stamina {:.2} under the 10% a skill needs", u.stamina));
        }
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        if u.stamina < cost {
            return Self::refused(u, out, &format!("stamina {:.2} under its cost {cost}", u.stamina));
        }
        Toggle::Now { overlay: false }
    }

    /// The kneel: he turns to the body he found and the start clip begins.
    fn on(&mut self, u: &mut Player, _out: &mut Outside) {
        (self.phase, self.clock, self.next, self.ended, self.at) = (Phase::Start, 0.0, 0, false, u.position);
        if let Some(c) = u.skill.corpse {
            let d = c - u.position;
            if d.x != 0.0 || d.z != 0.0 {
                u.yaw = (-d.x).atan2(-d.z);
            }
        }
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: corpse eater: the kneel over the body at {:.2} m, health {:.0}/{:.0}, stamina {:.2} draining {} a second; its block's effect h_{:08x} at the hardpoint h_{:08x}",
                     u.sim_time, unit_name(u), u.skill.corpse.map_or(0.0, |c| (c - u.position).xz().length()),
                     u.health, u.max_health, u.stamina,
                     u.skill.data.as_ref().map_or(0.0, |d| d.cost), self.s.effect, self.s.hardpoint);
        }
    }

    /// The off: put away (the stand-up's end, the stamina's 0, the death, a hand-over).
    fn off(&mut self, u: &mut Player) {
        if std::env::var("BF_SKILL_LOG").is_ok() && !u.dead {
            println!("t={:.2} {}: corpse eater: the feed is over, health {:.0}/{:.0}", u.sim_time, unit_name(u), u.health, u.max_health);
        }
        (self.phase, self.clock, self.next, self.ended) = (Phase::Off, 0.0, 0, false);
    }

    /// The chain's clock: the kneel, then the loop (its three gore events, the heal), then the
    /// stand-up, which returns true when its clip has played whole (the toggle's off). Movement off
    /// `at`, the body's own going (out of the reach, removed from the world), a full health bar, a
    /// knock-down or the death stands him up; at 0 stamina the base's command does the same
    /// through `toggle`.
    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        if matches!(self.phase, Phase::Off) || u.dead {
            return false;
        }
        if knocked_down(u) {
            self.stand(u, "knocked down");
            return false;
        }
        if self.phase != Phase::Stop && u.skill.corpse.is_none() {
            self.stand(u, "the body is gone");
            return false;
        }
        // (a bar with nothing left to give ends the feed: the heal has nothing to add, and the
        // drain would go on buying it)
        if self.phase != Phase::Stop && u.health >= u.max_health {
            self.stand(u, "his health is full");
            return false;
        }
        if self.phase != Phase::Stop && u.position.distance_squared(self.at) > MOVED * MOVED {
            self.stand(u, "he moved off the body");
            return false;
        }
        self.clock += dt;
        match self.phase {
            Phase::Off => false,
            Phase::Start => {
                if self.clock >= START {
                    (self.phase, self.clock, self.next) = (Phase::Idle, self.clock - START, 0);
                }
                false
            }
            Phase::Idle => {
                // the loop restarts every LOOP seconds; the events are counted within the cycle
                while self.clock >= LOOP {
                    self.clock -= LOOP;
                    self.next = 0;
                }
                while self.next < GORE_AT.len() && self.clock >= GORE_AT[self.next] {
                    self.next += 1;
                    u.skill.gore = true;
                    if std::env::var("BF_SKILL_LOG").is_ok() {
                        println!("t={:.2} {}: corpse eater: the loop's e8262ae5 (h_{EV_GORE:08x}) at {:.2} s: the gore effect at his left hand",
                                 u.sim_time, unit_name(u), self.clock);
                    }
                }
                let before = u.health;
                u.health = (u.health + HEAL_PER_SECOND * dt).min(u.max_health);
                if std::env::var("BF_SKILL_LOG").is_ok() && (before as i32) != (u.health as i32) {
                    println!("t={:.2} {}: corpse eater: feeding, health {:.0}/{:.0}", u.sim_time, unit_name(u), u.health, u.max_health);
                }
                false
            }
            Phase::Stop => {
                if self.clock >= STOP {
                    self.ended = true;
                    return true;
                }
                false
            }
        }
    }

    /// (vtable +0x14: the base's 0x213690 says free to act always; the demo's design makes feeding
    /// exclusive, as the Molotov Mutant's bottle: no reload, switch, grenade, medkit, pickup or
    /// crouch key while it's on)
    fn lets_act(&self, u: &Player) -> bool {
        !u.skill.on
    }

    /// (+0x108: the AI's attack check refuses while it's true; the player's `fire` and `aim` are
    /// cleared by `hands_busy`, as the Molotov Mutant's are)
    fn holds_fire(&self, u: &Player) -> bool {
        u.skill.on
    }

    /// The stand-up holds him where he stands until its clip has played (`skills::holds_still`,
    /// FUN_001195c0's skip of the stick's move and turn): a cancel isn't cut short by walking, so
    /// the animation finishes before he's free to move. The kneel and the loop don't: moving off
    /// the body is one of the feed's ends (the product owner's design).
    fn holds_still(&self) -> bool {
        self.phase == Phase::Stop
    }

    /// The clip the pose plays: the kneel's start, the loop or the stand-up, and the seconds into it
    /// (`play.rs` `Loaded::eat_clips`).
    fn feeding(&self) -> Option<(usize, f32)> {
        match self.phase {
            Phase::Off => None,
            Phase::Start => Some((0, self.clock)),
            Phase::Idle => Some((1, self.clock)),
            Phase::Stop => Some((2, self.clock)),
        }
    }
}

/// The `<corpse-eater>` block's effect, compiled when a character with the skill first appears on a
/// map (`load_gore`): its ALE effects, their one-shot lengths and the type's bounding sphere.
#[derive(Resource, Default)]
struct Gore {
    bursts: Vec<(std::sync::Arc<Compiled>, f32)>,
    bounds: Option<(Vec3, f32)>,
}

/// The systems: `hands_busy` after the controls and the test hooks that press or aim, before the
/// characters' step; `watch_bodies` after it (the corpses are where the step and their ragdolls left
/// them), before the skills' frame (whose toggle reads the body); `load_gore` before `show_gore`;
/// `show_gore` after the skills' frame (whose update sets `Skill::gore`). As
/// play_molotov_mutant.rs's, none is an `OnEnter` system: one there reorders the map's start and
/// moves the rack's pickups' entity indices (#211's tester).
pub fn plugin(app: &mut App) {
    app.add_systems(Update, (hands_busy.after(read_input).after(squad_control).after(ammo::test_trigger).before(update_player),
                             watch_bodies.after(update_player).before(skills::update_skills),
                             load_gore.before(show_gore),
                             show_gore.after(skills::update_skills))
        .run_if(in_state(AppState::Playing)));
}

/// While the controlled character feeds his hands are full: no fire and no aim (the product
/// owner's design: V again or moving stops it). Before his step, after every control and hook that
/// sets either.
fn hands_busy(mut player: ResMut<Player>) {
    if player.skill.on && player.skill.data.as_ref().is_some_and(|d| d.skill_set == CSKILL_CORPSE_EATER) {
        player.fire = false;
        player.aim = false;
    }
}

/// The nearest dead body within reach of each character with the skill, into `Skill::corpse` for its
/// toggle: **any dead character's** — the squad's, a test NPC's, a level enemy's, either side, any
/// type — at its own point (`body_point`), across CORPSE_REACH of his feet and of his floor
/// (`nearest_body`). A body thrown out of the reach, or taken out of the world, leaves the press
/// without one, and a feed under way without its body (`CorpseEater::update`).
fn watch_bodies(mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut shown: Local<Option<Vec3>>) {
    let bodies: Vec<Vec3> = std::iter::once(&*player).chain(squad.0.iter())
        .filter(|u| u.dead && !u.vaporised && !u.stand_in && !u.absent)
        .map(body_point)
        .collect();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        u.skill.corpse = None;
        if u.dead || u.loaded.is_none() || !u.skill.data.as_ref().is_some_and(|d| d.skill_set == CSKILL_CORPSE_EATER) {
            continue;
        }
        u.skill.corpse = nearest_body(u.position, bodies.iter().copied());
    }
    // (the log says when the controlled character's body comes and goes: what the press reads)
    if std::env::var("BF_SKILL_LOG").is_ok()
        && (shown.is_some() != player.skill.corpse.is_some() || shown.zip(player.skill.corpse).is_some_and(|(a, b)| a.distance_squared(b) > 0.25)) {
        match player.skill.corpse {
            Some(c) => println!("t={:.2} {}: corpse eater: a body at {:.2} m across, {:.2} m {} his feet", player.sim_time, unit_name(&player),
                                (c - player.position).xz().length(), (c.y - player.position.y).abs(),
                                if c.y >= player.position.y { "over" } else { "under" }),
            None => println!("t={:.2} {}: corpse eater: no body within {CORPSE_REACH} m across and {CORPSE_LEVEL} m of his floor", player.sim_time, unit_name(&player)),
        }
        *shown = player.skill.corpse;
    }
}

/// The effect type `CorpseEaterSkill::effect` (h_1eeb36f7) compiled, once a map has a character
/// with the skill: its ALE effects as a hit's are (`hit_effect_at`: the game's facing 3, one run
/// each), each with its one-shot length. m03_a and m03_b's objecttypes define the type; a map whose
/// archive doesn't is said, and the chain then plays without the gore.
fn load_gore(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>,
             mut materials: ResMut<Assets<StandardMaterial>>, ale: Option<ResMut<AleAssets>>,
             map: Option<Res<CurrentMap>>, player: Res<Player>, squad: Res<Squad>, mut loaded_for: Local<Option<String>>) {
    let name = map.map(|m| m.0.clone()).unwrap_or_default();
    let has = |u: &Player| u.skill.data.as_ref().is_some_and(|d| d.skill_set == CSKILL_CORPSE_EATER);
    if loaded_for.as_ref() == Some(&name) || !std::iter::once(&*player).chain(squad.0.iter()).any(has) {
        return;
    }
    *loaded_for = Some(name);
    // the HUD's caption (`NAME`): the demo's own text, as its class names' are (#210)
    game.0.strings.insert(NAME, "Feast".into());
    let mut gore = Gore::default();
    let kind = game.0.combat_skills.corpse_eater.effect;
    match game.0.effect_type_defs.get(&kind).cloned() {
        None => warn!("corpse eater: the effect type h_{kind:08x} isn't in this map's objecttypes (m03_a and m03_b define it): the feed plays without the gore"),
        Some(t) => {
            gore.bounds = t.bounds;
            if let Some(mut ale) = ale {
                for &e in t.effects.iter().filter(|&&e| e != 0) {
                    if let Some(fx) = ale.load_recorded_facing(&mut game.0, &mut images, &mut materials, e) {
                        let life = fx.duration();
                        gore.bursts.push((fx, life));
                    }
                }
            }
        }
    }
    info!("corpse eater: the gore effect h_{kind:08x}: {} bursts", gore.bursts.len());
    commands.insert_resource(gore);
}

/// A gore burst on the left hand of each character whose skill has one due (`Skill::gore`, set at
/// the loop's events, 0xe8262ae5): the effect's entities children of the hand's joint, at the
/// block's hardpoint (`CorpseEaterSkill::hardpoint`), so they ride with the hand as the game's
/// attached effect does; each goes when its particles have run (`AleExpire`, the type's max-life
/// being FLT_MAX).
fn show_gore(mut commands: Commands, gore: Option<Res<Gore>>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>) {
    let Some(gore) = gore else { return };
    if gore.bursts.is_empty() {
        return;
    }
    let hardpoint = game.0.combat_skills.corpse_eater.hardpoint;
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if !std::mem::take(&mut u.skill.gore) {
            continue;
        }
        let Some((joint, hp)) = u.loaded.as_ref().and_then(|l| l.model.hardpoints.get(&hardpoint).map(|&(bone, hp)| (l.joints[bone], hp))) else {
            continue;
        };
        let seed = u.sim_time.to_bits();
        for (i, (fx, life)) in gore.bursts.iter().enumerate() {
            let run = AleEffect::once(fx.clone(), 1.0, seed.wrapping_add((i as u32).wrapping_mul(0x9E37_79B9))).with_lod_sphere(gore.bounds);
            commands.spawn((Transform::from_translation(hp.point).with_rotation(hp.rot), Visibility::default(), Name::new("corpse eater gore"),
                            run, AleExpire(*life), ChildOf(joint)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::skills::can_activate;
    use bf_viewer::bf::character::SkillData;

    /// A Filthy Mutant as the demo makes him: the m03 types' cost and recovery, max-stamina 100,
    /// the stamina update on (the class rule), the settings' block, and half his health to fill.
    fn mutant() -> Player {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_CORPSE_EATER, max_stamina: 100.0, cost: MUTANT_COST, recovery: MUTANT_RECOVERY,
                                                        stamina_update: true, ..Default::default() }), &CombatSkills::default());
        u.max_stamina = 100.0;
        u.stamina = 100.0;
        u.health = 50.0;
        u
    }

    /// What the shared command does with an answer (play_skills.rs `command`), for the states the
    /// tests need: a `Now` switches the skill (on through the activate's 10% gate, off at once), the
    /// rest leave it.
    fn run(u: &mut Player, k: &mut Box<dyn SkillKind>, reserve: &mut ammo::Reserve) -> Toggle {
        let mut out = Outside { reserve, log: false };
        let was = u.skill.on;
        let answer = k.toggle(u, &mut out);
        if let Toggle::Now { .. } = answer {
            if was {
                u.skill.on = false;
                k.off(u);
            } else if can_activate(u) {
                u.skill.on = true;
                k.on(u, &mut out);
            }
        }
        answer
    }

    /// FUN_0013c800 and the factory: skill-set 16 makes the class with the block's effect, it has no
    /// name and nothing feeds until it's on, and a type of its own with the skill-set gets it (the
    /// three dispatch points, play_effect_skills.rs).
    #[test]
    fn the_factory_makes_it() {
        let data = SkillData { skill_set: CSKILL_CORPSE_EATER, cost: MUTANT_COST, recovery: MUTANT_RECOVERY, ..Default::default() };
        assert!(super::super::effect_skills::own_type_skill(Some(&data)).is_some());
        let skill = skills::Skill::new(Some(&data), &CombatSkills::default());
        let k = skill.kind.as_ref().expect("the corpse eater");
        assert_eq!((k.name(), k.feeding().is_none()), (NAME, true), "the demo's caption");
        assert_eq!((k.lets_act(&Player::new(3)), k.holds_fire(&Player::new(3))), (true, false));
        assert_eq!(skill.data.as_ref().map(|d| (d.cost, d.recovery)), Some((3.0, 1.5)));
    }

    /// **Any body at his feet**: across the ground's plane within the reach and of his floor, the
    /// nearest one, whatever its side, its type or the height its own point is at (a standing body's
    /// is at the hips, a corpse's near the floor); one 3 m off, or 2.4 m up a ledge, isn't his, nor
    /// one below it.
    #[test]
    fn the_nearest_body_at_his_feet_is_the_one() {
        let me = Vec3::new(0.0, 0.0, 0.0);
        let one = |p: Vec3| std::iter::once(p);
        assert_eq!(nearest_body(me, one(Vec3::new(3.0, 0.2, 0.0))), None, "3 m off");
        assert_eq!(nearest_body(me, one(Vec3::new(0.0, 2.4, 0.0))), None, "a ledge above");
        assert_eq!(nearest_body(me, one(Vec3::new(0.0, -2.4, 0.0))), None, "a ledge below");
        // at his feet, whichever way he faces
        assert_eq!(nearest_body(me, one(Vec3::new(0.0, 0.0, -0.7))), Some(Vec3::new(0.0, 0.0, -0.7)));
        // (a standing body's point is at the hips, a corpse's near the floor: both are his)
        assert_eq!(nearest_body(me, one(Vec3::new(1.2, 1.05, 0.0))), Some(Vec3::new(1.2, 1.05, 0.0)));
        // two: the nearer across, though the further one's point is lower
        let near = Vec3::new(1.0, 0.9, 0.0);
        let far = Vec3::new(2.0, 0.1, 0.0);
        assert_eq!(nearest_body(me, [far, near].into_iter()), Some(near));
    }

    /// No body within the reach refuses the press; one there kneels him over it (the start clip's
    /// slot), turning him to it, and V again stands him up (the stop clip), after which the skill
    /// switches off once the clip has played.
    #[test]
    fn a_press_next_to_a_body_kneels_and_a_press_stands_him_up() {
        let mut reserve = ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Refused, "no body");
        // a body 1.5 m off to his +x: `watch_bodies` would have set it
        u.skill.corpse = Some(Vec3::new(1.5, 0.0, 0.0));
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && u.skill.on);
        assert!((u.yaw + std::f32::consts::FRAC_PI_2).abs() < 1e-5, "{}", u.yaw);
        assert_eq!(k.feeding(), Some((0, 0.0)));
        assert!(!k.lets_act(&u) && k.holds_fire(&u));
        assert!(!k.holds_still(), "the kneel doesn't hold him: moving off ends the feed");
        // V again: the stand-up, and the skill stays on until its clip has played
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Stays);
        assert!(u.skill.on && k.feeding().is_some_and(|(slot, _)| slot == 2));
        assert!(k.holds_still(), "the stand-up holds him until its clip ends");
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Stays, "still standing up");
        let dt = 1.0 / 30.0;
        let mut ended = false;
        for _ in 0..40 {
            ended |= k.update(&mut u, dt);
        }
        assert!(ended, "the stop clip's end asks the command for the off");
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Now { overlay: false });
        assert!(!u.skill.on && k.feeding().is_none() && k.lets_act(&u));
        assert!(!k.holds_still(), "free to move once the stand-up is over");
    }

    /// **A full bar ends it**: the press is refused on full health (nothing left to give), and a
    /// feed that fills the bar stands him up there and then, with nothing healing on the way out.
    #[test]
    fn a_full_bar_ends_the_feed() {
        let mut reserve = ammo::Reserve::default();
        let mut u = mutant();
        u.skill.corpse = Some(Vec3::new(1.0, 0.0, 0.0));
        u.health = u.max_health;
        let mut k = u.skill.kind.take().unwrap();
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Refused, "full health: nothing to feed for");
        // one point short: it kneels, and the loop fills the bar
        u.health = u.max_health - 1.0;
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && u.skill.on);
        let dt = 1.0 / 30.0;
        let mut stood = false;
        for _ in 0..120 {
            k.update(&mut u, dt);
            if k.feeding().is_some_and(|(slot, _)| slot == 2) {
                stood = true;
                break;
            }
        }
        assert!(stood, "the full bar stands him up");
        assert_eq!(u.health, u.max_health);
        let full = u.health;
        for _ in 0..40 {
            k.update(&mut u, dt);
        }
        assert_eq!(u.health, full, "nothing heals on the way out");
        assert!(k.holds_still(), "the stand-up holds him while its clip plays");
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Now { overlay: false });
        assert!(!u.skill.on && !k.holds_still(), "free to move once it has played");
    }

    /// The loop: the kneel first (no heal, no gore), then `eat_idle` from 1.13 s, the loop's three
    /// e8262ae5 events at 0.37, 1.70 and 6.13 s into it, each asking for the gore once, and the heal
    /// at 6 HP a second while it plays.
    #[test]
    fn the_loop_feeds_and_its_events_ask_for_the_gore() {
        let mut reserve = ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        u.skill.corpse = Some(Vec3::new(1.0, 0.0, 0.0));
        run(&mut u, &mut k, &mut reserve);
        let dt = 1.0 / 30.0;
        let mut gore = 0;
        // the kneel: 1.13 s, and no gore or heal in it
        for _ in 0..33 {
            k.update(&mut u, dt);
            gore += std::mem::take(&mut u.skill.gore) as usize;
        }
        assert_eq!((gore, u.health), (0, 50.0));
        // the loop: 0.37, 1.70 and 6.13 s in, one event a frame at most
        let mut at = Vec::new();
        let mut t = 0.0;
        while at.len() < 3 {
            k.update(&mut u, dt);
            t += dt;
            if std::mem::take(&mut u.skill.gore) {
                at.push(t);
            }
            assert!(t < 7.0, "the third event never came");
        }
        for (i, want) in [0.37, 1.70, 6.13].iter().enumerate() {
            assert!((at[i] - want).abs() <= dt * 1.5, "{at:?} against {want}");
        }
        assert!(u.health > 50.0 + HEAL_PER_SECOND * (6.13 - dt) , "{:.2}", u.health);
        // (the loop's clock is the kneel's own time less START, so it trails the frames counted here
        // by the kneel's overshoot: the slot is what matters)
        assert_eq!(k.feeding().map(|(slot, _)| slot), Some(1));
    }

    /// Moving off the body or being knocked down stands him up (the stop clip), and nothing feeds on
    /// the way out.
    #[test]
    fn moving_off_the_body_stands_him_up() {
        let mut reserve = ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        u.skill.corpse = Some(Vec3::new(1.0, 0.0, 0.0));
        run(&mut u, &mut k, &mut reserve);
        let before = u.health;
        u.position.x += 0.5;
        assert!(!k.update(&mut u, 1.0 / 30.0));
        assert_eq!(k.feeding().map(|(slot, _)| slot), Some(2), "the stand-up");
        assert_eq!(u.health, before, "nothing feeds after he moves");
        // the stop clip ends: the command's off
        for _ in 0..40 {
            k.update(&mut u, 1.0 / 30.0);
        }
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Now { overlay: false });
        assert!(!u.skill.on);
    }

    /// The feed is about the body: one that goes (a blast throws it out of the reach, a trigger takes
    /// it out of the world) stands him up in the loop, as moving off it does, and nothing heals after.
    #[test]
    fn a_body_gone_stands_him_up() {
        let mut reserve = ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        u.skill.corpse = Some(Vec3::new(1.0, 0.0, 0.0));
        run(&mut u, &mut k, &mut reserve);
        let dt = 1.0 / 30.0;
        // into the loop (the kneel is 1.13 s) and the first whole points of the heal
        for _ in 0..40 {
            k.update(&mut u, dt);
        }
        assert_eq!(k.feeding().map(|(slot, _)| slot), Some(1));
        assert!(u.health > 50.0);
        // the body is gone: the next frame's body scan finds none
        u.skill.corpse = None;
        let before = u.health;
        assert!(!k.update(&mut u, dt));
        assert_eq!(k.feeding().map(|(slot, _)| slot), Some(2), "the stand-up");
        for _ in 0..40 {
            k.update(&mut u, dt);
        }
        assert_eq!(u.health, before, "nothing heals on the way out");
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Now { overlay: false });
        assert!(!u.skill.on);
    }
}
