//! Gunthar Ghent's Cloak (#205): the special skill of the deathmatch's and the Ferguson Platoon's
//! Gunthar Ghent (CSKILL_CLOAK 8; the types h_1dc1aa6d of objecttypes-dm and h_e714d929 of
//! objecttypes-sdm_ferg: stamina cost 6, recovery 1.5, 80 HP, no start or end sound), Hawk's
//! invisibility without her clip, her item costs and her hold fire. Read from default.xbe (the rows
//! are in viewer/docs/code-map/skills-and-stamina.md: FUN_0013c590, 0x13c5f0, 0x13c6a0,
//! FUN_0013c710, FUN_0013c770, 0x13c7d0, 0x13c7e0 and the vtable 0x39d540) and the game's data.
//! **Not filmed**: no take shows it (todo/npc-skills/answer.md §6; #205's shots C1-C5 were never
//! recorded), so every number below is the code's or the data's and none is a fit to footage. Its
//! page is viewer/docs/skills/ghents-cloak.md.
//!
//!  - **The class** (vtable 0x39d540, constructor FUN_0013c590): the settings of game-options'
//!    `<combat-skills>` block `<cloak h_f726177e="2" h_1d194665="0.25"/>` (h_e806e6f2 by the
//!    element dispatch at 0x19f50c, its handler FUN_0019e900: `CloakSettings`): the recovery rate,
//!    2 a second (settings +0x368 -> skill+0x28; Hawk's block has 4), and the noise floor, 0.25
//!    (+0x36c -> +0x2c); the level (skill+0x20) 0 and the factor (+0x24) 1.0. The vtable differs
//!    from the base class's in the toggle (+4, 0x13c5f0), the name (+0x4c, 0x13c580), a round
//!    fired (+0xb0, 0x13c6a0), the update (+0xc0, FUN_0013c710), the visibility (+0xe4, 0x13c7d0),
//!    the noise (+0xe8, 0x13c7e0) and Hawk's on and off (+0x120 / +0x124, 0x13c650 / 0x13c670); the
//!    rest is the base's, compared slot by slot with Hawk's vtable 0x39d190 (the table is in the
//!    code map): the drain is `stamina-cost` x dt (+0xbc, 0x13c690: 6 a second), the character is
//!    always free to act (+0x14, 0x213690), no item costs anything (+0x104, 0x2c60c0: no
//!    `used_item` here) and an AI character keeps firing while it is on (+0x108, 0x20e200: no
//!    `holds_fire`).
//!  - **The toggle** (0x13c5f0), no clip, no request byte and no FUN_00117210, returning 0 either
//!    way (so no overlay and no motion): while the skill is on it is switched off at once
//!    (FUN_0010e3e0), at **any** level, a fade-in still under way included; off, it is switched on
//!    (FUN_0010e470, which asks the 10% again) only with the stamina at least 10% of the maximum
//!    (0x3a4ff0, equal allowed) **and** the level exactly 1.0 or 0.0 (a bitwise compare with 1.0
//!    and an `fcomp` with 0.0), else nothing at all. The level of an off skill is 0 or on its way
//!    there, the fade-out of the second after an off, so a press in that second is refused; the
//!    1.0 is the same test as Hawk's and cannot hold for an off skill (the level is read after the
//!    switch). Hawk's toggle refuses a press mid-fade either way.
//!  - **On and off** (0x13c650 / 0x13c670, Hawk's): the character's invisibility controller, kind
//!    2, switched on or off (FUN_002107d0, `Skill::cloak`): 1.0 s to fade in and 1.0 s to fade out
//!    (0x212380), from the press. The look is Hawk's (play_stealth.rs: her skin's controller and
//!    the refraction), asked for through `SkillKind::cloaks`. The game gives the controller to a
//!    model by its type's name (FUN_00210060: the mask 0x1f for h_1dc1aa6d and h_e714d929); the demo
//!    gives every character one, so the demo's Terrorist (`character_class::CLOAK_DEMO`, a class of
//!    the demo's) has it too.
//!  - **The update** (FUN_0013c710), every frame, on or off: the level read back from the
//!    controllers (FUN_00210880, 0 when none is active), the visibility pushed into them
//!    (FUN_00210950, with the last frame's factor), then the factor (skill+0x24) climbing at the
//!    recovery rate, 2 a second, clamped 0..1 (FUN_0013c770), then the base update (FUN_0010e500).
//!  - **Visibility** (+0xe4, 0x13c7d0) 1 - factor x level: what the AI's sight and the crosshair's
//!    selection read (play_brain.rs `Sensor`, play_reticle.rs), as for Hawk. **Noise** (+0xe8,
//!    0x13c7e0) the floor + (1 - the floor) x the visibility: 0.25 + 0.75 x it (his footsteps,
//!    play_liquids.rs `step`).
//!  - **A round** (+0xb0, 0x13c6a0), each one the fire loop takes while the skill is on: the
//!    factor to 0 (so he is seen as the AI and the crosshair see it, and the refraction's ripple
//!    jumps, for the 0.5 s the factor takes to climb back at 2 a second) and the stamina less the
//!    gun's `<Damage max>` (the weapon data's +0x18) **once**, clamped at 0 (FUN_001258d0). Hawk's
//!    cost is twice that (`fadd st(0), st(0)`, 0x13d0d0). A shot that empties the bar switches the
//!    skill off by the stamina update's command at 0.
//!  - **The drain** is the base class's, `stamina-cost` x dt (`SkillKind::drain`'s default): 6 a
//!    second. At 0 the stamina update sends the toggle, which switches it off.
//!
//! The class is one of the effect skills' family only in the demo's wiring: it is handed out through
//! play_effect_skills.rs's three dispatch points (`kind`, `own_type_skill`, `give_defaults`), one arm
//! each, as the Wrath's and the Air Strike's are. In the demo Gunthar Ghent is a type of its own in
//! a squad slot of the test map (`BF_TEST_SQUAD_SLOT="0.3,me,gunthar ghent@sdm_ferg"` on any map of a
//! test session, or `BF_TEST_SQUAD="3:gunthar ghent@sdm_ferg"` on the test map); the demo's Terrorist
//! class carries it (#205: the product owner's request, `character_class::TERRORIST`). Press V or
//! `BF_TEST_SKILL=<s>`; `BF_TEST_GIVE_SKILL=8` gives the class to the character controlled at a map's
//! start (Gunthar's cost 6 and recovery 1.5 unless given); `BF_SKILL_LOG=1` prints each press and its
//! refusal, each shot with its cost and when the factor is back to 1.

use super::*;
use super::skills::{can_activate, Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{CloakSettings, CombatSkills};

/// The Cloak's skill-set (CSKILL_CLOAK, the name table 0x3bd814; the factory FUN_0012e590 makes it
/// with FUN_0013c590: its jump table 0x12e810 entry for skill-set 8, 0x12e662, allocates 0x30 bytes
/// and calls it).
pub const CSKILL_CLOAK: i64 = 8;
/// Its name (its vtable +0x4c, 0x13c580: `mov eax, 0xfd42048b`): "Cloak", h_fd42048b in
/// common/stringtable/string-table-en.xml line 775. The HUD shows it while the skill is on.
const NAME_CLOAK: u32 = 0xFD42_048B;
/// Gunthar Ghent's cost and recovery (`<combat-skill skill-set="8" h_105c13aa="6" h_f2a2beb9="1.5">`,
/// objecttypes-dm.xml line 415, h_1dc1aa6d, and objecttypes-sdm_ferg.xml line 235, h_e714d929, alike),
/// which BF_TEST_GIVE_SKILL=8 gives by default (play_effect_skills.rs `give_defaults`) and the demo's
/// Terrorist class carries (`character_class::CLOAK_DEMO`).
pub const GHENT_COST: f32 = 6.0;
pub const GHENT_RECOVERY: f32 = 1.5;
/// The level at which a switch-on is allowed besides 0 (0x13c625: `cmp [ecx+0x20], 0x3f800000`, the
/// bits of 1.0): a skill whose level is exactly 1.0 or exactly 0.0 (0x13c62e: `fcomp` against 0.0,
/// 0x3a4f88). Anything between, or a fade's last frame a little past 1 or under 0 (the controller's
/// level isn't clamped, 0x210b10), is mid-fade.
fn settled(level: f32) -> bool {
    level == 1.0 || level == 0.0
}

/// The Cloak of a skill-set 8 `<combat-skill>`'s character, with game-options' block (FUN_0013c590):
/// play_effect_skills.rs `kind`'s arm for the class.
pub fn kind(settings: &CombatSkills) -> Box<dyn SkillKind> {
    Box::new(CloakSkill::new(settings.cloak))
}

/// Gunthar Ghent's Cloak (vtable 0x39d540, constructor FUN_0013c590): see the module notes. (Named
/// for the class: `skills::Cloak` is the material controller it switches, Hawk's too.)
struct CloakSkill {
    /// the level as the last update read it (skill+0x20): 0 visible, 1 cloaked, between mid-fade
    level: f32,
    /// the factor (skill+0x24): 1, dropped to 0 by a round, climbing back at the recovery rate
    factor: f32,
    /// game-options' `<cloak>` block, copied at the constructor (skill+0x28, +0x2c)
    cfg: CloakSettings,
    /// when the last round dropped the factor (the log's measure of its climb), if it is still
    /// below 1
    shot_at: Option<f32>,
}

impl CloakSkill {
    /// The skill as made (0x13c5e0: the level 0, the factor 1.0), over the block's values.
    fn new(cfg: CloakSettings) -> Self {
        CloakSkill { level: 0.0, factor: 1.0, cfg, shot_at: None }
    }

    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: cloak refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }
}

impl SkillKind for CloakSkill {
    fn name(&self) -> u32 {
        NAME_CLOAK
    }

    /// 0x13c5f0, returning 0 (no overlay, no motion) either way: on, off at once at any level
    /// (`Toggle::Now`); off, on only at 10% of the maximum stamina and a level of exactly 1.0 or
    /// 0.0, else refused with nothing happening. (The activate asks the 10% again, FUN_0010e470.)
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on {
            return Toggle::Now { overlay: false };
        }
        if !can_activate(u) {
            return Self::refused(u, out, &format!("stamina {:.2} under 10% of {}", u.stamina, u.max_stamina));
        }
        if !settled(self.level) {
            return Self::refused(u, out, &format!("mid-fade, level {:.3}", self.level));
        }
        Toggle::Now { overlay: false }
    }

    /// 0x13c650: the invisibility controller on (FUN_002107d0, kind 2): it fades in over 1.0 s from
    /// the press.
    fn on(&mut self, u: &mut Player, _out: &mut Outside) {
        u.skill.cloak.switch(true);
    }

    /// 0x13c670: the same, off.
    fn off(&mut self, u: &mut Player) {
        u.skill.cloak.switch(false);
    }

    /// FUN_0013c710, every frame, on or off: the level read back from the controller
    /// (FUN_00210880), the visibility pushed into it (FUN_00210950) with the last frame's factor,
    /// then the factor climbing at the recovery rate (FUN_0013c770), clamped 0..1.
    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        self.level = u.skill.cloak.level();
        let v = self.visibility(u);
        u.skill.cloak.push_visibility(v);
        self.factor = (self.factor + dt * self.cfg.level_recovery).clamp(0.0, 1.0);
        if let Some(at) = self.shot_at.filter(|_| self.factor >= 1.0) {
            self.shot_at = None;
            if std::env::var("BF_SKILL_LOG").is_ok() {
                println!("t={:.2} {}: cloak: the factor is back to 1 after {:.2} s", u.sim_time, unit_name(u), u.sim_time - at);
            }
        }
        false
    }

    /// 0x13c6a0, once for each round the fire loop takes while the skill is on (with a gun and an
    /// owner): the factor to 0 and the stamina less the gun's Damage max (`WeaponDef::damage`, the
    /// weapon data's +0x18), 1x where Hawk's is 2x, clamped at 0.
    fn fired(&mut self, u: &mut Player, def: &WeaponDef) {
        if !u.skill.on {
            return;
        }
        self.factor = 0.0;
        self.shot_at = Some(u.sim_time);
        let was = u.stamina;
        u.stamina = (u.stamina - def.damage).clamp(0.0, u.max_stamina);
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: a shot while cloaked: seen (the factor 0), stamina {was:.2} - {:.2} -> {:.2}", u.sim_time,
                     unit_name(u), def.damage, u.stamina);
        }
    }

    /// 0x13c7d0: 1 - the factor x the level.
    fn visibility(&self, _u: &Player) -> f32 {
        1.0 - self.factor * self.level
    }

    /// 0x13c7e0: the floor + (1 - the floor) x the visibility, 0.25 + 0.75 x it.
    fn noise(&self, u: &Player) -> f32 {
        self.cfg.noise_floor + (1.0 - self.cfg.noise_floor) * self.visibility(u)
    }

    /// Hawk's look is his (play_stealth.rs): the body's fade and the refraction.
    fn cloaks(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::character::SkillData;

    /// Gunthar as the demo makes him with the skill: the dm type's cost and recovery, the settings'
    /// block, 100 stamina.
    fn ghent() -> Player {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_CLOAK, max_stamina: 100.0, cost: GHENT_COST, recovery: GHENT_RECOVERY,
                                                        stamina_update: true, ..Default::default() }), &CombatSkills::default());
        u.max_stamina = 100.0;
        u.stamina = 100.0;
        u
    }

    /// A gun of `damage` Damage max.
    fn gun(damage: f32) -> WeaponDef {
        WeaponDef { damage, ..Default::default() }
    }

    /// The factory makes the Cloak for a skill-set 8 (through the family's `kind`, play_effect_skills.rs;
    /// its dispatch test is there): "Cloak", the block's values, seen from the outside as a cloaking
    /// skill with no effect to spawn, off at the start and fully visible.
    #[test]
    fn the_factory_makes_it() {
        let u = ghent();
        let k = u.skill.kind.as_ref().unwrap();
        assert_eq!((k.name(), k.cloaks(), k.effect_data().is_none(), k.attack_skill()), (0xFD42_048B, true, true, false));
        assert_eq!((u.skill.data.as_ref().map(|d| (d.cost, d.recovery)), u.skill.on), (Some((6.0, 1.5)), false));
        assert_eq!((skills::visibility(&u), skills::noise(&u), skills::cloaks(&u)), (1.0, 1.0, true));
        // the block's values, game-options' `<cloak h_f726177e="2" h_1d194665="0.25">`
        let cfg = CombatSkills::default().cloak;
        assert_eq!((cfg.level_recovery, cfg.noise_floor), (2.0, 0.25));
    }

    /// 0x13c5f0 off: switched on at the press at 10% of the maximum (equal allowed) and a level of
    /// exactly 0 or 1; under 10%, or mid-fade (the fade-out of the second after an off), nothing.
    #[test]
    fn it_switches_on_at_ten_percent_and_a_settled_level() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut u = ghent();
        let mut k = u.skill.kind.take().unwrap();
        u.stamina = 9.99;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
        u.stamina = 10.0;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
        // (the level is what the last update read: mid-fade, off)
        u.stamina = 100.0;
        for level in [0.0001, 0.5, 0.9999, -0.011, 1.011] {
            u.skill.cloak.fade = skills::Fade { state: skills::FadeState::Out, time: 1.0 - level };
            k.update(&mut u, 0.0);
            assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused, "level {level}");
        }
        // (a fade over: off, level 0)
        u.skill.cloak.fade = skills::Fade::default();
        k.update(&mut u, 0.0);
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
        assert!(settled(1.0) && settled(0.0) && !settled(0.9999) && !settled(1.0001) && !settled(-0.0001));
    }

    /// 0x13c5f0 on: off at once at any level and any stamina, the fade-in under way or not (Hawk's
    /// press is refused mid-fade either way).
    #[test]
    fn it_switches_off_at_once_at_any_level() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut u = ghent();
        let mut k = u.skill.kind.take().unwrap();
        u.skill.on = true;
        k.on(&mut u, &mut out);
        // (0.4 s into the fade-in)
        for _ in 0..12 {
            u.skill.cloak.update(1.0 / 30.0);
        }
        k.update(&mut u, 1.0 / 30.0);
        assert!(u.skill.cloak.level() > 0.3 && u.skill.cloak.level() < 0.5);
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
        u.stamina = 0.0;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
    }

    /// The fades: 1.0 s in from the switch, 1.0 s out, and the level the update reads is the visibility
    /// and noise the AI and the footsteps go by: 1 - the level, a quarter at the floor.
    #[test]
    fn it_fades_in_a_second_and_out_a_second() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut u = ghent();
        let mut k = u.skill.kind.take().unwrap();
        u.skill.on = true;
        k.on(&mut u, &mut out);
        let dt = 1.0 / 30.0;
        for frame in 1..=30 {
            u.skill.cloak.update(dt);
            k.update(&mut u, dt);
            let want = frame as f32 / 30.0;
            assert!((u.skill.cloak.level() - want).abs() < 1e-3, "frame {frame}: {}", u.skill.cloak.level());
            // (the visibility is pushed from the level just read: 1 - level, the factor being 1)
            assert!((k.visibility(&u) - (1.0 - want)).abs() < 1e-3, "frame {frame}");
        }
        for _ in 0..2 {
            u.skill.cloak.update(dt);
            k.update(&mut u, dt);
        }
        assert_eq!(u.skill.cloak.level(), 1.0);
        assert_eq!((k.visibility(&u), k.noise(&u)), (0.0, 0.25));
        // off: out over 1 s from the top
        u.skill.on = false;
        k.off(&mut u);
        for _ in 0..30 {
            u.skill.cloak.update(dt);
            k.update(&mut u, dt);
        }
        assert!(u.skill.cloak.level() < 0.01, "{}", u.skill.cloak.level());
        for _ in 0..3 {
            u.skill.cloak.update(dt);
            k.update(&mut u, dt);
        }
        assert_eq!((u.skill.cloak.level(), k.visibility(&u)), (0.0, 1.0));
    }

    /// 0x13c6a0 while on: the factor to 0 (visibility 1 whatever the level, noise 1) and the stamina
    /// less the gun's Damage max, once; the factor climbs back at 2 a second (FUN_0013c770), so 0.5 s
    /// to 1. Off, nothing. Hawk's, with the same gun, takes twice as much.
    #[test]
    fn a_shot_shows_him_and_costs_the_damage() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut u = ghent();
        let mut k = u.skill.kind.take().unwrap();
        k.fired(&mut u, &gun(4.75));
        assert_eq!(u.stamina, 100.0, "off: nothing");
        u.skill.on = true;
        k.on(&mut u, &mut out);
        let dt = 1.0 / 30.0;
        for _ in 0..32 {
            u.skill.cloak.update(dt);
            k.update(&mut u, dt);
        }
        assert_eq!(k.visibility(&u), 0.0);
        k.fired(&mut u, &gun(4.75));
        assert_eq!(u.stamina, 95.25);
        assert_eq!((k.visibility(&u), k.noise(&u)), (1.0, 1.0));
        // (the factor 0 -> 1 at 2 a second: 15 frames of 1/30 s, a rounding over or under)
        let mut frames = 0;
        while k.visibility(&u) > 0.0 {
            u.skill.cloak.update(dt);
            k.update(&mut u, dt);
            frames += 1;
            assert!(frames < 40, "never back");
        }
        assert!((14..=16).contains(&frames), "{frames} frames");
        // a gun that costs more than is left empties the bar and no further
        k.fired(&mut u, &gun(300.0));
        assert_eq!(u.stamina, 0.0);
        // Hawk's pays twice for the same round
        let mut hawk = Player::new(2);
        hawk.skill = skills::Skill::new(Some(&SkillData { skill_set: 4, max_stamina: 100.0, cost: 2.5, recovery: 2.0, stamina_update: true, ..Default::default() }),
                                        &CombatSkills::default());
        hawk.max_stamina = 100.0;
        hawk.stamina = 100.0;
        hawk.skill.on = true;
        skills::fired(&mut hawk, &gun(4.75));
        assert_eq!(hawk.stamina, 90.5);
    }

    /// No +0x104 and no +0x108: an item used while cloaked costs nothing (Hawk's costs 5, and 15 for a
    /// grenade thrown, the medkit being use type 0), and an AI character keeps firing.
    #[test]
    fn items_are_free_and_the_ai_keeps_firing() {
        let mut u = ghent();
        u.skill.on = true;
        for use_type in [skills::IOU_USE_IN_HAND, 2, 3, 8] {
            skills::used_item(&mut u, use_type);
        }
        assert_eq!(u.stamina, 100.0);
        assert!(!skills::holds_fire(&u) && skills::lets_act(&u));
        let mut hawk = Player::new(2);
        hawk.skill = skills::Skill::new(Some(&SkillData { skill_set: 4, max_stamina: 100.0, cost: 2.5, recovery: 2.0, stamina_update: true, ..Default::default() }),
                                        &CombatSkills::default());
        hawk.max_stamina = 100.0;
        hawk.stamina = 100.0;
        hawk.skill.on = true;
        skills::used_item(&mut hawk, skills::IOU_USE_IN_HAND);
        assert_eq!(hawk.stamina, 95.0);
        assert!(skills::holds_fire(&hawk));
    }

    /// The base class's drain: `stamina-cost` x dt, 6 a second, and nothing of Hawk's x5 or second
    /// slot factor.
    #[test]
    fn the_drain_is_six_a_second() {
        let u = ghent();
        let k = u.skill.kind.as_ref().unwrap();
        assert!((k.drain(&u, GHENT_COST, 1.0) - 6.0).abs() < 1e-6);
        assert!((k.drain(&u, GHENT_COST, 1.0 / 30.0) - 0.2).abs() < 1e-6);
    }
}
