//! The Wrath of Garner (#208): the Outcast Shaman's special skill (CSKILL_WRATH 13; the deathmatch
//! type h_e973af9e and the Feral Hunting Pack's h_130090e3: stamina cost 33, recovery 2.8, 70 HP),
//! "Calls a blast on the crosshair's target and holds a spirit pose". One of the effect skills
//! (play_effect_skills.rs: the spawner, the target point and the damage areas are #200's); its own
//! part is here: a toggle that switches on and stays on for 3 s, draining the stamina all the
//! while, and a character who isn't free to act meanwhile. Read from default.xbe (the rows are in
//! viewer/docs/code-map/skills-and-stamina.md: FUN_0013f1c0, FUN_0013f240, FUN_0013f2f0, 0x13b980,
//! the vtable 0x39c378), the analyst's answers (todo/200-effect-skills/answers.md §5, §8) and the
//! takes 08-11 and 18 in todo/200-effect-skills (spec.md §4).
//!
//!  - **The class** (vtable 0x39c378, constructor FUN_0013f1c0): the `<effect-data>` of game-options'
//!    block h_085b2f26 (`WrathSkill::effect`: one h_1e72e858, no scatter, reach 50, FUN_0013de50's
//!    copy, so the spawner and the target point are the Orbital Beam's) and its `attack-duration`
//!    (3 s, skill+0x40, `WrathSkill::duration`). The drain is the base class's, `stamina-cost` x dt
//!    (0x13c690: 33 a second, `SkillKind::drain`'s default); no sound or overlay of its own, and the
//!    type's start / end sounds through the shared on / off.
//!  - **The toggle** (FUN_0013f240): while the skill is on it returns 0 at once, so a second press
//!    does nothing (and neither does the command the stamina update sends at 0: the skill ends by
//!    its timer alone). Off, it needs: no motion request pending (char+0x748 bit 3), a motion state,
//!    the current MOTION with a motion type 45 transition (`has_transition`), the overlay test
//!    (below: the demo's is stricter than the game's) and a stamina of at least `stamina-cost` x
//!    `attack-duration`, 33 x 3 =
//!    99 (the compare passes on equal, `fcompp` / `test ah, 0x41` / `jp` at 0x13f2c8-0x13f2cf: the
//!    code map's "equality refused" read the C's `<` and `==` the wrong way; takes 09: refused at
//!    98.87, fired at 99.66; 08 and 18 at 100). Then, in this order: the spawner FUN_0013e410(0) (the
//!    blast starts at once, in the press's frame: `Skill::fire`), the activate FUN_0010e470 (on, the
//!    start sound, the line), the timer (skill+0x3c) = the attack-duration, and the return 2: the
//!    command handler asks for MT_TOGGLE_SPECIAL, the Shaman's `spirit` pose (`Toggle::Motion`).
//!    **The overlay test** (0x13f286-0x13f2ba) passes when the current MOTION has +0x250 bit 2 or
//!    lacks +0x252 bit 1 (an ordinary motion), or when either overlay slot, char+0x214 or +0x218, is
//!    empty or can be interrupted (+0x3c bit 0): it refuses only a MOTION that blocks overlays with
//!    both slots busy. So a reload, a weapon switch or a throw alone, which take one slot, don't
//!    stop the Wrath in the game. The demo's is stricter: it refuses while any overlay is under way
//!    (`skills::free_to_act`, FUN_00117210's reading, which asks more than this test does), a
//!    simplification, for nothing says what the pose does to an overlay in flight (not filmed:
//!    #200's shot S1 doesn't ask) and the demo doesn't know which slot each of its overlays takes.
//!  - **The pose**: the MOTION h_e327688a (motion type 45, script `spirit`, prefix-type 1: the clip
//!    `Sc_w1_spirit` / `Sc_w2_spirit` of Brutus's animation set that the ferals play, 1.50 s, no
//!    events, `aim="false"`, `block-overlay="true"`, h_f6a410e6 `true`): a whole-body motion that
//!    holds him where he stands (the stick's move and turn are skipped, `skills::holds_still`) and
//!    takes no jump, dodge or crouch from him, then the idle (its next motion). Take08: the motion
//!    state 45 for 1.45-1.59 s of the game's clock.
//!  - **The update** (FUN_0013f2f0, the skill's +0xc0, after the stamina update's drain step in the
//!    same frame as FUN_0011a840 orders them): while on, the timer at or under this frame's time
//!    ends it (FUN_0010e3e0: the off sound, the line; the timer to 0), else the time is taken off.
//!    So it is on for the timer over the frame's time, 3 s (take08: 2.94-2.96 s of the log's clock,
//!    which ran 1.0127 game seconds to its second: 2.98-3.0), and the drain, taken in each frame it
//!    is on, leaves 100 - 99 = ~1 (take08 0.22, take10 1.05, take09 1.15; the off frame refills,
//!    2.8 a second, FUN_0011a840's third step). The demo ends it through the toggle command, which is
//!    the only way its skills switch off: the update asks for it and the toggle, finding the timer
//!    out while on, answers `Toggle::Now`.
//!  - **Not free to act** (+0x14, 0x13b980: true only while the skill is off): FUN_00117210 says no
//!    for the whole 3 s, not just the pose's 1.5. So no weapon switch, reload, item use (the medkit,
//!    grenades, the OrgSen), weapon pickup or use walk, and the crouch key is lost (the controller
//!    sends the command only if free, 0x134df6: Z and the test hooks ask `lets_act` in `read_input`
//!    and `test_trigger`, each of the others in its own module). The X hold on a usable or a lying
//!    gun arms nothing in the demo, its simplification: the game's hold isn't gated, its completed
//!    use is (FUN_00120290 at 0x1207b9), deferred while char+0x620 is above 0 or cancelled. Not
//!    gated: the stick's move and turn after the pose, firing (the controller's fire section posts
//!    the command without asking) and the White button, which returns 0 at once.
//!  - **The effect** h_1e72e858 (common's objecttypes line 592): the ALE effects `exp_hbomb`,
//!    `_flash`, `_flash2`, `_ring` and `_smoke`; the sound h_e866e1d9, which plays whole, 4.35 s
//!    (`play-length` 1.75 only picks the path and the window); two damage areas, **650 EXPLOSION a
//!    second for 0.1 s within 5 m, then 30 a second for 1 s within 15 m from 2 s**, no falloff, push
//!    0.2, so rates and not lumps: 58.7-64.2 in the first 0.1 s (two or three frames of a 30 Hz clock,
//!    take08 58.66, take09 59.60, take11 63.76, take18 64.22) and 29.9-30.4 in the second, 88.7-94 in
//!    all, the owner and his team hurt like anyone within range (take11: the teammate 63.76 + 29.86,
//!    the owner at 14.8 m 29.86; spec §1.5), a 70-HP dm Tex dead 0.2 s into the second area (take18).
//!  - **Sounds** (answers.md Q5): the type's start sound (ee753485, Brutus's 5.98 s rumble) at the
//!    on, the end sound (190cd1f6) at the off, 2D for the controlled character (FUN_0010e7a0), as the
//!    squad's skills (`skills::switched`); the takes (08, 10, 11, 18) hear neither under the effect's
//!    own 4.35 s boom and nothing of the rumble after it. The analyst reads FUN_0010e750 (the first
//!    thing FUN_0010e7a0 does: the skill's current voice released with 0 if it still plays) as the
//!    end sound cutting the start sound; #35 take08 measured Brutus's end sound, the same call, not
//!    cutting his rumble (the rest of it still matches after the off, todo/35-special-skills
//!    spec.md section 3), so what release(h, 0) does is open and the demo keeps the squad's: the
//!    start sound plays on.
//!
//! The class is one of the effect skills' family (play_effect_skills.rs), whose three dispatch
//! points each have an arm for it: `kind` (the skill object by skill-set: `kind` here), `own_type_skill`
//! (a type whose `<combat-skill skill-set="13">` has it) and `give_defaults` (BF_TEST_GIVE_SKILL's
//! cost and recovery: `SHAMAN_COST` and `SHAMAN_RECOVERY` here).
//!
//! In the demo the Shaman is any squad slot of a test session that holds his type
//! (`BF_TEST_SQUAD_SLOT="0.3,me,h_130090e3"` on the Arena, `BF_MAP=mp1`; his model, clips and set
//! load as any type's, #394), whose `<combat-skill>` gives him the class; `BF_TEST_GIVE_SKILL=13`
//! gives it to the character controlled at a map's start (the cost 33 and recovery 2.8 the
//! Shaman's; Brutus stands in with his `spirit` overlay, which is the same clip). Press V or
//! `BF_TEST_SKILL=<s>`; `BF_SKILL_LOG=1` prints the toggle, its refusals, the pose and the target
//! point, `BF_DAMAGE_LOG=1` the effect's ticks.

use super::*;
use super::skills::{free_to_act, Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{CombatSkills, EffectData, WrathSkill};

/// The Wrath's skill-set (CSKILL_WRATH, the name table 0x3bd814; the factory FUN_0012e590 makes it
/// with FUN_0013f1c0).
pub const CSKILL_WRATH: i64 = 13;
/// Its name (its vtable +0x4c, 0x13f1b0): "Wrath of Garner", h_1f405090 in
/// common/stringtable/string-table-en.xml. The HUD shows it while the skill is on.
const NAME_WRATH: u32 = 0x1F40_5090;
/// The Outcast Shaman's cost and recovery (`<combat-skill skill-set="13" h_105c13aa="33"
/// h_f2a2beb9="2.8">`, objecttypes-dm.xml's h_e973af9e and objecttypes-sdm_feral.xml's h_130090e3
/// alike), which BF_TEST_GIVE_SKILL=13 gives by default (play_effect_skills.rs `give_defaults`).
pub const SHAMAN_COST: f32 = 33.0;
pub const SHAMAN_RECOVERY: f32 = 2.8;

/// The Wrath of a skill-set 13 `<combat-skill>`'s character, with game-options' block (FUN_0013f1c0):
/// play_effect_skills.rs `kind`'s arm for the class.
pub fn kind(settings: &CombatSkills) -> Box<dyn SkillKind> {
    Box::new(Wrath { s: settings.wrath, timer: 0.0 })
}

/// Whether `u`'s current motion has a motion type 45 transition, which the toggle asks of its
/// MOTION at 0x13f278 (+0x534; none, and it refuses). In the feral set (ms_feral_colonist.xml,
/// which the Shaman plays) the idles, walk, run, dash, back and side steps, the crouch idle and
/// the crouch walks, and the stand-to-crouch and back (MT 46) have one; the jumps, dodges, the
/// dive, the slide, the throw (MT 42), the stun and the knock-down have none. The demo's states:
/// standing, kneeling down or up and kneeling, not sliding or knocked down, and with no grenade in
/// the hand (charging, thrown or set down, its release pending: the demo's reading of the throw).
fn has_transition(u: &Player) -> bool {
    matches!(u.action, Action::None | Action::Crouching | Action::Crouched | Action::Rising) && !u.slide_on && !knocked_down(u)
        && u.throwing.is_none() && u.charge <= 0.0 && u.pending_release.is_none()
}

/// The Wrath of Garner (vtable 0x39c378, constructor FUN_0013f1c0): see the module notes.
struct Wrath {
    /// game-options' block, as the constructor copies it (the effect, the attack-duration)
    s: WrathSkill,
    /// the timer (skill+0x3c): the attack-duration from the toggle, counted down while on
    timer: f32,
}

impl Wrath {
    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: wrath refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }
}

impl SkillKind for Wrath {
    fn name(&self) -> u32 {
        NAME_WRATH
    }

    /// FUN_0013f240. On: nothing (returns 0), but for the end of the timer, which the update has
    /// found and asks the command for (the module notes: the demo's way to FUN_0010e3e0). Off: the
    /// motion, the overlay test and the stamina (>= the cost x the attack-duration), then the
    /// spawner, the activate and the timer, returning 2 (`Toggle::Motion`).
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on {
            return if self.timer <= 0.0 { Toggle::Now { overlay: false } } else { Toggle::Refused };
        }
        if !has_transition(u) {
            return Self::refused(u, out, "no motion type 45 transition from his motion");
        }
        // (stricter than the game's, 0x13f286-0x13f2ba: that refuses only with the MOTION blocking
        // overlays and both overlay slots busy, so a reload alone passes there. The demo's whole
        // free-to-act test, the module notes say why)
        if !free_to_act(u) {
            return Self::refused(u, out, "busy");
        }
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        let needs = cost * self.s.duration;
        if u.stamina < needs {
            return Self::refused(u, out, &format!("stamina {:.2} under {cost} x {} = {needs}", u.stamina, self.s.duration));
        }
        u.skill.fire = true;
        self.timer = self.s.duration;
        if out.log {
            println!("t={:.2} {}: wrath of garner at stamina {:.2}: its effect h_{:08x} asked for, on for {} s", u.sim_time, unit_name(u), u.stamina,
                     self.s.effect.effect, self.s.duration);
        }
        Toggle::Motion
    }

    /// FUN_0013f2f0, every frame (the base's FUN_0010e500 after it does nothing here): on, the timer
    /// at or under this frame's time ends it (true: the command, which the toggle answers while on
    /// with the timer out), else the frame's time comes off it.
    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        if !u.skill.on {
            return false;
        }
        if self.timer <= dt {
            self.timer = 0.0;
            return true;
        }
        self.timer -= dt;
        false
    }

    /// (+0x14, 0x13b980: free to act only while the skill is off)
    fn lets_act(&self, u: &Player) -> bool {
        !u.skill.on
    }

    fn effect_data(&self) -> Option<&EffectData> {
        Some(&self.s.effect)
    }

    /// (+0x10c, 0x213690: an attack skill; the demo's goal refuses it, play_orders.rs
    /// `toggle_ability`)
    fn attack_skill(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::character::SkillData;

    /// The Shaman as the demo makes him with the skill: the dm type's cost and recovery, the
    /// settings' block.
    fn shaman() -> Player {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_WRATH, max_stamina: 100.0, cost: SHAMAN_COST, recovery: SHAMAN_RECOVERY,
                                                        stamina_update: true, ..Default::default() }), &CombatSkills::default());
        u.max_stamina = 100.0;
        u.stamina = 100.0;
        u
    }

    /// FUN_0013f240's gate: the stamina must reach 33 x 3 = 99, equal passing (the compare at
    /// 0x13f2c8): take09 refused at 98.87 and fired at 99.66; a press while on does nothing.
    #[test]
    fn the_toggle_needs_ninety_nine() {
        let mut u = shaman();
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut k = u.skill.kind.take().unwrap();
        u.stamina = 98.87;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
        assert!(!u.skill.fire);
        u.stamina = 99.0;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Motion);
        assert!(u.skill.fire);
        u.skill.fire = false;
        // (on, with the timer running: a press does nothing)
        u.skill.on = true;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
        assert!(!u.skill.fire);
        assert_eq!((k.name(), k.attack_skill()), (0x1F40_5090, true));
        assert_eq!(k.effect_data().map(|d| d.effect), Some(0x1E72_E858));
    }

    /// The toggle's other tests: a motion with no motion type 45 transition (a jump, a slide, a
    /// knock-down, a grenade in the hand) or a busy overlay refuses it. The busy refusal is the
    /// demo's own, stricter than FUN_0013f240's (0x13f286-0x13f2ba), which refuses only a MOTION that
    /// blocks overlays with both overlay slots busy: in the game a reload alone, which takes one
    /// slot, passes. The reload case below pins the demo's rule, not the game's (the module notes).
    #[test]
    fn the_toggle_needs_a_transition_and_free_overlays() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let refusals: [(&str, fn(&mut Player)); 5] = [
            ("a jump", |u| u.action = Action::JumpFall),
            ("a dodge", |u| u.action = Action::Dodge { left: true }),
            ("a slide", |u| u.slide_on = true),
            ("a grenade charging", |u| u.charge = 0.3),
            ("a reload (the demo's rule: the game passes one overlay alone)", |u| u.reloading = Some(Reload { weapon: 0, time: 0.0, fill: 10, filled: false })),
        ];
        for (what, set) in refusals {
            let mut u = shaman();
            set(&mut u);
            let mut k = u.skill.kind.take().unwrap();
            assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused, "{what}");
            assert!(!u.skill.fire, "{what}");
        }
        // (kneeling down, kneeling and getting up have one)
        for action in [Action::Crouching, Action::Crouched, Action::Rising, Action::None] {
            let mut u = shaman();
            u.action = action;
            let mut k = u.skill.kind.take().unwrap();
            assert_eq!(k.toggle(&mut u, &mut out), Toggle::Motion, "{action:?}");
        }
    }

    /// The timer: set to the attack-duration at the toggle, taken down by the frame's time while on;
    /// it ends the skill in the first frame it's at or under that time, so 3 s on at the game's
    /// 30 Hz is 90 frames (the last asking for the end), and the toggle then answers the command
    /// with `Now` (the switch off). Off, the update does nothing.
    #[test]
    fn it_is_on_for_three_seconds() {
        let mut u = shaman();
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut k = u.skill.kind.take().unwrap();
        assert!(!k.update(&mut u, 1.0 / 30.0));
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Motion);
        u.skill.on = true;
        let dt = 1.0 / 30.0;
        let mut frames = 0;
        while !k.update(&mut u, dt) {
            frames += 1;
            assert!(frames < 200, "never ends");
        }
        // (the press's frame counts too: 89 frames taking time off, the 90th ends it, or the 91st
        // when the repeated subtraction leaves the last 1/30 a rounding over it)
        assert!((90..=91).contains(&(frames + 1)), "{}", frames + 1);
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
        // 60 Hz: 180 frames
        let mut u = shaman();
        let mut k = u.skill.kind.take().unwrap();
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Motion);
        u.skill.on = true;
        let mut n = 0;
        while !k.update(&mut u, 1.0 / 60.0) {
            n += 1;
        }
        assert!((179..=181).contains(&(n + 1)), "{}", n + 1);
    }

    /// Free to act only while off (+0x14, 0x13b980); the drain is the base's cost x dt, 33 a second.
    #[test]
    fn it_isnt_free_to_act_while_on() {
        let mut u = shaman();
        let k = u.skill.kind.take().unwrap();
        assert!(k.lets_act(&u));
        u.skill.on = true;
        assert!(!k.lets_act(&u));
        assert!((k.drain(&u, SHAMAN_COST, 1.0 / 30.0) - 1.1).abs() < 1e-5);
        assert!(!skills::ended_by_actions(&u) && k.lets_overlays_play());
    }

    /// The factory makes the Wrath for a skill-set 13 (through the family's `kind`, play_effect_skills.rs;
    /// its own dispatch test is there), with game-options' block: its name and its effect.
    #[test]
    fn the_factory_makes_it() {
        let shaman = SkillData { skill_set: CSKILL_WRATH, cost: SHAMAN_COST, recovery: SHAMAN_RECOVERY, ..Default::default() };
        let skill = skills::Skill::new(Some(&shaman), &CombatSkills::default());
        let k = skill.kind.as_ref().unwrap();
        assert_eq!((k.name(), k.effect_data().map(|d| d.effect)), (NAME_WRATH, Some(0x1E72_E858)));
        assert_eq!((skill.data.as_ref().map(|d| d.cost), skill.on), (Some(33.0), false));
    }
}
