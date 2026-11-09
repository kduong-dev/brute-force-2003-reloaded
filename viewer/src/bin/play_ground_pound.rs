//! The Supermutant's ground pound (#212): CSKILL_BOSS_SUPERMUTANT 17, the skill of e23's two
//! Supermutants (`h_ece04235`, `h_1ffe2615`: 669 HP, stamina cost 1, recovery 1.5): "Pounds the
//! ground when the target is too close". One of the effect skills (play_effect_skills.rs: the
//! spawner, the target point and the effect's damage areas are #200's and #373's); its own part is
//! here: a toggle that switches on and returns 2 (the MT 45 `ground_pound` motion, whole-body), an
//! update that calls the spawner at the clip's event and switches the skill off at the clip's end.
//! Read from default.xbe (the rows are in viewer/docs/code-map/skills-and-stamina.md: 0x13b940, the
//! boss base 0x13b770, 0x13b830, 0x13b800, 0x13b7c0; the vtable 0x39c730), the analyst's answers
//! (todo/npc-skills/answer.md §3.1, §5) and the data (`game-options-en.xml` lines 169-171,
//! `ms_mutant.xml` line 629, e23's `objecttypes-e23.xml` line 1742).
//!
//!  - **The class** (FUN_0013b940, on the boss base FUN_0013b770): the `<boss-supermutant
//!    animation-duration="3">` block's `<effect-data>` (`BossSupermutantSkill::effect`: one
//!    h_ece33110, no scatter, no pitch, reach 1, the character's own point) and its
//!    animation-duration (skill+0x3c, 3), the multiplier the toggle's stamina gate is measured by.
//!    **The name**: the data has none (the boss base's +0x4c, 0x2bf0a0, returns 0, and the English
//!    string table has no "Ground Pound" entry), so the demo's own *Ground Pound* is the HUD's
//!    caption (`NAME`, put in `Game::strings` by `name_skill` as the corpse eater's *Feast* is).
//!  - **The toggle** (0x13b830, the boss base's): refused while the skill is on, with a motion
//!    request pending, without a current motion or an MT 45 transition in it, on the overlay-slot
//!    states, or with stamina under the animation-duration x cost (3 x 1 = 3; the shipped
//!    Supermutant's stamina stays at 100 because its type's h_e78efbb2 is false, so in the game it
//!    never bites). Else it switches the skill on itself (FUN_0010e470: the 10% of the maximum, the
//!    start sound, the line) and returns 2: MT_TOGGLE_SPECIAL, the MT 45 motion h_eb4a1ed4
//!    (`script="ground_pound"`, prefix-type 1, `transition="0.1"`, `block-overlay="true"`,
//!    `h_f6a410e6="true"`), whose clip is `Sc_w1_ground_pound`, **1.97 s**, no root motion.
//!    `Toggle::Motion` (`start_pose`).
//!  - **The burst** (the class's animation-event handler 0x13b800): on the clip's **04592c53**,
//!    0.97 s, it calls the class's +0x138 (0x50b40, nothing for the Supermutant) and the spawner
//!    FUN_0013e410(0). The target point is the boss base's own (+0x134, 0x13b7c0: the character's
//!    root node), so the effect starts where he stands (`Skill::fire`, `SkillKind::own_point`).
//!    The effect h_ece33110 (`BF_TEST_EFFECT`'s, already built by #373): ALE h_18058148, light
//!    h_ef1056ea, sound h_1ea097fa (2.5 s), **50 POWERBLADE (11) a second for 1 s within 7.5 m**,
//!    no falloff, push h_ed582b3c 2.5, and a camera shake from 1 s for 2 s within 10 m (falloff 2,
//!    h_f944a10f 0.7). The areas hurt his team and everyone else in range at their POWERBLADE
//!    factors (play_damage.rs), and the shake rides the controlled character's camera
//!    (play_shake.rs); **his own class is spared** (the demo's `SkillKind::spares_owner`): the
//!    game's Supermutant takes 0.059 x 50, ~3 HP of 669, its type's factor, which the class rule
//!    strips, and 50 of the class's 95 would be half its health a pound.
//!  - **The end**: the clip's **ead033a3** at 1.97 s switches the skill off, as the game's event
//!    handler does; nothing in the class otherwise ends it. The demo's update asks for the command
//!    when its clock reaches the end, and the toggle, on with the clip over, answers `Toggle::Now`.
//!  - **Who it's for**: a type whose `<combat-skill skill-set="17">` has it, through the family's
//!    three dispatch points (play_effect_skills.rs `kind`, `own_type_skill`, `give_defaults`): the
//!    campaign's two e23 Supermutants and the demo's Supermutant class (`mp_supermutant`). The demo
//!    turns the stamina update on for the class (the game's type flag is false, so its stamina
//!    never drains), as it does for the Molotov Mutant's and the corpse eater's: a press costs 3,
//!    the clip drains 1 a second and the 1.5 a second recovery refills it, on the game's numbers.
//!
//! Not built: the AI's own use (the personality h_08cf55a2's EVT_TGT_TOO_CLOSE -> 
//! GOAL_ATTACK_SM_GROUNDSHAKE 116, whose start 0x37440 sends the toggle; the sensor that posts the
//! event is #26), and the gutterball (115). The demo's squad-order goal refuses an attack skill
//! (play_orders.rs), so the pound is asked for by V or `BF_TEST_SKILL`.
//!
//! Checked: unit tests here and in play_effect_skills.rs; a live test-map run of the class as the
//! controlled character against NPCs at 3 m and 10 m (`BF_SKILL_LOG=1`, `BF_DAMAGE_LOG=1`,
//! `BF_SHAKE_LOG=1`): the clip and pose, the effect at 0.97 s, 50 damage over the second at 3 m,
//! nothing at 10 m, the owner unhurt, the shake, off at 1.97 s. No xemu take shows the pound (the
//! ticket's S1/S2 are unfilmed), so the timing and the numbers are the code's and the data's, not
//! footage's.

use super::*;
use super::skills::{free_to_act, Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{BossSupermutantSkill, CombatSkills, EffectData};

/// The Supermutant's skill-set (CSKILL_BOSS_SUPERMUTANT 17, the name table 0x3bd814; the factory
/// FUN_0012e590 makes it with FUN_0013b940).
pub const CSKILL_BOSS_SUPERMUTANT: i64 = 17;
/// The Supermutant's cost and recovery (`<combat-skill skill-set="17" h_105c13aa="1"
/// h_f2a2beb9="1.5">`, e23's two Supermutants), which BF_TEST_GIVE_SKILL=17 gives by default.
pub const SUPERMUTANT_COST: f32 = 1.0;
pub const SUPERMUTANT_RECOVERY: f32 = 1.5;
/// The clip's burst event (04592c53) at 0.97 s and its end event (ead033a3) at 1.97 s, the times
/// `animations-*.chnl` gives `Sc_w1_ground_pound` (answer.md §3.1; the Molotov Mutant's clip uses
/// the same handler). The clip is 1.97 s long.
const BURST_AT: f32 = 0.97;
const END_AT: f32 = 1.97;
/// The MT 45 motion's `transition`, which the pose blends in and out over (s).
const TRANSITION: f32 = 0.1;
/// The HUD's caption while he pounds, *Ground Pound*: the demo's own (`NAME`'s id is the demo's,
/// outside the string table's hash space, and `name_skill` puts the text in `Game::strings` as the
/// corpse eater's `load_gore` puts "Feast" there). The class's +0x4c is the boss base's (0x2bf0a0,
/// which returns 0) and the English table has no ground- or pound-word for it, so the caption is a
/// demo pick like the Molotov Mutant's "Self-Destruct": the product owner's choice (2026-11-09),
/// the ticket's own word for the move.
const NAME: u32 = 0xFF00_0002;

/// The Supermutant's ground pound and its HUD caption (`name_skill`).
pub fn plugin(app: &mut App) {
    app.add_systems(Update, name_skill.run_if(in_state(AppState::Playing)));
}

/// Puts `NAME`'s text in `Game::strings` once a map has a character with the skill (a type's own,
/// the demo class's, or BF_TEST_GIVE_SKILL's): the HUD draws the name only from the table
/// (play_hud.rs `Part::SkillLabel`), so a press would show none without it. The corpse eater's
/// `load_gore` does the same for its "Feast".
fn name_skill(mut game: ResMut<GameData>, player: Res<Player>, squad: Res<Squad>) {
    let has = |u: &Player| u.skill.data.as_ref().is_some_and(|d| d.skill_set == CSKILL_BOSS_SUPERMUTANT);
    if std::iter::once(&*player).chain(squad.0.iter()).any(has) {
        game.0.strings.entry(NAME).or_insert_with(|| "Ground Pound".into());
    }
}

/// The Supermutant's skill, with game-options' `<boss-supermutant>` block (FUN_0013b940): the
/// play_effect_skills.rs `kind`'s arm for the class.
pub fn kind(settings: &CombatSkills) -> Box<dyn SkillKind> {
    Box::new(GroundPound { s: settings.boss_supermutant, clock: 0.0, fired: false })
}

/// The Supermutant's ground pound (vtable 0x39c730): see the module notes.
struct GroundPound {
    /// the `<boss-supermutant>` block, as the constructor copies it (the effect, the duration)
    s: BossSupermutantSkill,
    /// seconds into the clip the toggle started, its events read off it
    clock: f32,
    /// whether the clip's burst event has been taken (once a clip)
    fired: bool,
}

impl GroundPound {
    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: ground pound refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }
}

impl SkillKind for GroundPound {
    /// The HUD's caption: *Ground Pound*, the demo's own string (`NAME`), as the boss base's +0x4c
    /// names nothing.
    fn name(&self) -> u32 {
        NAME
    }

    /// 0x13b830. On: the clip's end alone asks for the off (0x13b800's ead033a3), and a press
    /// meanwhile does nothing. Off: an MT 45 transition, the overlay test and the stamina (>= the
    /// animation-duration x cost, 3), then the switch and the pose (`Toggle::Motion`); nothing is
    /// started until the clip's own event.
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on {
            return if self.clock >= END_AT { Toggle::Now { overlay: false } } else { Toggle::Refused };
        }
        if !wrath::has_transition(u) {
            return Self::refused(u, out, "no motion type 45 transition from his motion");
        }
        // (the game's overlay-slot test, read through the demo's free-to-act one, as the Wrath's
        // and the Molotov Mutant's are: a simplification, see play_wrath.rs's notes)
        if !free_to_act(u) {
            return Self::refused(u, out, "busy");
        }
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        let needs = cost * self.s.duration;
        if u.stamina < needs {
            return Self::refused(u, out, &format!("stamina {:.2} under {cost} x {} = {needs}", u.stamina, self.s.duration));
        }
        if out.log {
            println!("t={:.2} {}: ground pound at stamina {:.2}: the MT 45 ground_pound, the burst at {BURST_AT} s (h_{:08x})",
                     u.sim_time, unit_name(u), u.stamina, self.s.effect.effect);
        }
        Toggle::Motion
    }

    /// The skill switched on: the clip starts, so its clock does too (FUN_0010e470, inside the
    /// toggle's return 2).
    fn on(&mut self, _u: &mut Player, _out: &mut Outside) {
        (self.clock, self.fired) = (0.0, false);
    }

    /// The skill switched off: the clip's end, a knock-down, the death or the key (the handler
    /// 0x115df5 -> FUN_0010e3e0). The next press starts a fresh clip.
    fn off(&mut self, _u: &mut Player) {
        (self.clock, self.fired) = (0.0, false);
    }

    /// Every frame while on (0x13b800's events read off the clip's clock): the burst at 04592c53
    /// (`Skill::fire`, the effect at the character's root), then at ead033a3 the command that ends
    /// it (the toggle, on with the clip over, answers the off). A knock-down cuts the clip short
    /// (play_skills.rs `step` drops the pose; the skill is off at once), so no late burst.
    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        if !u.skill.on {
            return false;
        }
        if knocked_down(u) {
            // (the clip is cut by the knock, play_skills.rs `step` drops the pose: nothing more
            // fires, and the off is asked for rather than waiting for an end event that will
            // never come)
            self.clock = END_AT;
            return true;
        }
        self.clock += dt;
        if !self.fired && self.clock >= BURST_AT {
            self.fired = true;
            u.skill.fire = true;
            if std::env::var("BF_SKILL_LOG").is_ok() {
                println!("t={:.2} {}: ground pound: the clip's 04592c53 at {:.2} s, the effect h_{:08x} at his root node",
                         u.sim_time, unit_name(u), self.clock, self.s.effect.effect);
            }
        }
        if self.clock >= END_AT {
            return true;
        }
        false
    }

    fn effect_data(&self) -> Option<&EffectData> {
        Some(&self.s.effect)
    }

    /// (+0x10c, 0x213690: an attack skill. The demo's squad-order goal refuses one, play_orders.rs;
    /// the AI's own goal 116 is #26's, not built)
    fn attack_skill(&self) -> bool {
        true
    }

    /// (+0x134, 0x13b7c0: the boss base's target point is the character's own root node)
    fn own_point(&self) -> bool {
        true
    }

    /// (the demo's, not the game's: the game's Supermutant stands in its own blast and takes 50 x
    /// its type's 0.059 POWERBLADE factor, ~3 HP of its 669; the demo's class has none by the class
    /// rule and would take 50 of its 95, half its health a pound. So the class's own effect leaves
    /// it alone, while his squadmates and everyone else in the 7.5 m take it as the data says.)
    fn spares_owner(&self) -> bool {
        true
    }

    /// The MT 45 motion's `transition`, in and out.
    fn pose_blend(&self) -> (f32, f32) {
        (TRANSITION, TRANSITION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::character::SkillData;

    /// The Supermutant as the demo makes him with the skill: the campaign type's cost and recovery.
    fn mutant() -> Player {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_BOSS_SUPERMUTANT, max_stamina: 100.0, cost: SUPERMUTANT_COST,
                                                        recovery: SUPERMUTANT_RECOVERY, stamina_update: true, ..Default::default() }),
                                   &CombatSkills::default());
        u.max_stamina = 100.0;
        u.stamina = 100.0;
        u
    }

    /// 0x13b830's gate: the animation-duration x the cost, 3 x 1, equal passing; a press while the
    /// clip runs does nothing.
    #[test]
    fn the_toggle_needs_three_stamina() {
        let mut u = mutant();
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut k = u.skill.kind.take().unwrap();
        u.stamina = 2.99;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
        assert!(!u.skill.fire);
        u.stamina = 3.0;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Motion);
        assert!(!u.skill.fire);
        assert_eq!(k.name(), NAME);
        u.skill.on = true;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
    }

    /// The clip's two events: nothing at the press, the burst at 0.97 s (once), the off asked for
    /// at 1.97 s; a fresh clip after it fires again.
    #[test]
    fn the_burst_fires_at_the_clips_event_and_the_clip_ends_it() {
        let mut u = mutant();
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut k = u.skill.kind.take().unwrap();
        u.skill.on = true;
        let mut fired_at = None;
        let mut ended_at = None;
        for i in 1..=40 {
            let t = i as f32 * 0.05;
            let done = k.update(&mut u, 0.05);
            if u.skill.fire && fired_at.is_none() {
                fired_at = Some(t);
                u.skill.fire = false;
            }
            if done {
                ended_at = Some(t);
                break;
            }
        }
        let fired_at = fired_at.expect("the burst");
        assert!((fired_at - BURST_AT).abs() <= 0.05, "{fired_at}");
        assert!((ended_at.expect("the end") - END_AT).abs() <= 0.05);
        // the toggle, on with the clip over, is the off; then the next press starts a new clip
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
        k.off(&mut u);
        u.skill.on = false;
        assert_eq!(k.update(&mut u, 0.05), false);
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Motion);
    }

    /// The record is the data's: one h_ece33110, reach 1, and it starts at the character's own
    /// point; the class is an attack skill.
    #[test]
    fn the_effect_is_the_pounds() {
        let k = kind(&CombatSkills::default());
        assert_eq!(k.effect_data(), Some(&EffectData { count: 1, effect: 0xECE3_3110, scatter: 0.0, pitch: 0.0, reroll: false, reach: 1.0, rise: 0.0 }));
        assert!(k.own_point() && k.attack_skill() && k.spares_owner());
        assert_eq!(k.pose_blend(), (TRANSITION, TRANSITION));
    }
}
