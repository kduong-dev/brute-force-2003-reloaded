//! The Air Strike (CSKILL_AIR_STRIKE 6, #203): the special skill of the deathmatch Militia (type
//! `h_055acb5f`, objecttypes-dm.xml line 355) and of the Ferguson Platoon's (sdm_ferg, line 175):
//! while it's on, every 0.75 s a strike is called on the crosshair's target, 20 stamina each, and
//! the shell lands 2 s later. One of the effect skills (play_effect_skills.rs: the spawner
//! FUN_0013e410, the target point 0x13df70, the damage areas of the effect type), whose own part
//! here is when it calls the spawner: five times from a full bar, then it switches itself off.
//! Checked against todo/200-effect-skills (spec.md §3; take05 and take17 the strikes at a locked
//! target at 20 m, take06 the second press, take07 no lock, take13 a teammate; answers.md Q4, Q6,
//! Q7); the default.xbe functions are in viewer/docs/code-map/skills-and-stamina.md.
//!
//!  - **The class** (constructor FUN_0013b5e0, vtable 0x39da28): game-options' `<air-strike
//!    trigger-frequency="0.75">` (h_1657bd33, settings +0x2f0 -> skill+0x40: `AirStrikeSkill::
//!    frequency`) and its `<effect-data>` (settings +0x2c4, FUN_0013de50: `EffectData::air_strike`):
//!    one effect h_007aa97d, at the target point plus up to 2 m either way on x and z (the XZ offset,
//!    FUN_0013e740), turned by the pitch 1 rad and a random angle about the vertical (FUN_0013e800,
//!    FUN_0001fa20: `effect_skills::turn`), the aim's ray reaching 50 m, and h_e272cd2c 1, the
//!    rise of a point found on the terrain (`target_point`). The skill's drain is 0 (+0xbc, 0x13b5d0:
//!    nothing is taken while it's on but what each call costs).
//!  - **The toggle** (FUN_0013b660, vtable +4; the White button, V): with the skill on, off at once
//!    (FUN_0010e3e0) and the timer (skill+0x3c) cleared: the strikes already called are effect objects
//!    of their own and land all the same (take06: the second press at 3.7 s after two calls, no third,
//!    both shells land). Off, with stamina at least the cost (20, equality in: take05's fifth call
//!    is made from exactly 20.00): the spawner (FUN_0013e410(0): `Skill::fire`, whose target point is
//!    found anew at each call: `effect_skills::fire_effects`), the cost off (FUN_001258d0), the timer set to
//!    the frequency and the skill's switch on (FUN_0010e470), whose gate, 10% of the maximum stamina,
//!    is tested **after** the payment: a press at 20-29.9 pays and fires one strike and the skill
//!    never goes on (from the code only: not filmed). Under the cost nothing happens. It answers
//!    `Toggle::Now`: the shared switch does the on and the off (their chatter tags; the Militia's
//!    type names no start or end sound), and a press the 10% gate refuses leaves the strike called.
//!  - **The update** (FUN_0013b6d0, vtable +0xc0, after the stamina rule's drain step): while on,
//!    the timer counts down by the frame's time, and in the frame it is at or under that time
//!    (`timer <= dt`: the code's compare is "less than" xor "equal"; the takes read it as 0.0077,
//!    0.0034 and 0.0016 just before a call, never under 0) the next call is made as the toggle's,
//!    with stamina for it, and the timer is set to the frequency again (no part of the frame's
//!    time carried over: 23 frames at the game's 30 Hz, 0.767 s, where the record says 0.75); without
//!    the stamina the skill is switched off (FUN_0010e3e0: `update`'s true, the toggle command).
//!    From a full bar the fifth call leaves 0, and the next frame's drain step ends the skill by the
//!    stamina rule's at-0 toggle (`skills::step`; the takes' 0.03 stamina in the sample of the
//!    off is that frame's refill), so the update's own end comes only from a bar that isn't a
//!    multiple of 20 (70: calls at 70, 50, 30, then off at 10 left; not filmed).
//!  - **The look**: the effect type's eight ALE effects, two at once (the smoke trail and its flare)
//!    and six after 2 s (the shell's burst), its sound h_065168c9 at 2 s and its damage area, 40
//!    EXPLOSION a second from 1 s for 0.5 s within 20 m with a linear falloff (play_effects.rs:
//!    `Rules::Game`), so each strike's hit points go 1.0-1.5 s after its call, 16-20 of them at a
//!    target within 2.5 m of it, and its picture and sound come at 2.0 s (spec §3.3: the HP before the
//!    picture). The owner and his team are hurt as anyone is (take05: the owner 20 m off took 1.9).
//!  - **The HUD name** "Air Strike" (h_f91b25db; vtable +0x4c, 0x13b5c0) shows while it's on, as any
//!    skill's does (its base update, FUN_0010e500, holds the name widget's timer at 0.2 s each frame
//!    it's on: play_hud.rs).
//!
//! Test hooks: the Militia on the test map is `BF_TEST_SQUAD="3:militia@sdm_ferg"` (he takes Tex's
//! place and leads) or on any map of a test session `BF_TEST_SQUAD_SLOT="0.3,me,militia@sdm_ferg"`;
//! `BF_TEST_GIVE_SKILL=6` gives the controlled character the class with the Militia's cost 20 and
//! recovery 1 (`6,<cost>,<recovery>` for others), as play_effect_skills.rs gives the Orbital Beam.
//! V or `BF_TEST_SKILL=<s>[,<s>...]` presses the key; `BF_SKILL_LOG=1` prints each call, its stamina
//! and the target point (`effect_skills::fire_effects`), each switch on and off with why.

use super::*;
use super::skills::{Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{AirStrikeSkill, EffectData};

/// The Air Strike's skill-set (CSKILL_AIR_STRIKE, the name table 0x3bd814, 6; the factory FUN_0012e590
/// makes it with FUN_0013b5e0).
pub const CSKILL_AIR_STRIKE: i64 = 6;
/// Its name (its vtable +0x4c, 0x13b5c0): "Air Strike", h_f91b25db in
/// common/stringtable/string-table-en.xml (line 778).
const NAME_AIR_STRIKE: u32 = 0xF91B_25DB;

/// The cost of a call and the recovery that BF_TEST_GIVE_SKILL gives by default: the Militia's
/// `<combat-skill skill-set="6" h_105c13aa="20" h_f2a2beb9="1">` (objecttypes-dm.xml's h_055acb5f and
/// objecttypes-sdm_ferg.xml's h_1563301d alike; the takes: 20 a call, +1.01 a second after the off).
pub const MILITIA_COST: f32 = 20.0;
pub const MILITIA_RECOVERY: f32 = 1.0;

/// The Air Strike (vtable 0x39da28): the skill's own part of the shared interface. Its slots past
/// the base's: +4 the toggle FUN_0013b660, +0x4c its name, +0xbc its drain 0x13b5d0 (0), +0xc0 its
/// update FUN_0013b6d0, +0x10c 0x213690 (an attack skill for the AI), +0x134 0x13df70 (the target
/// point); the free-to-act slot (+0x14) is the base's yes, so it blocks nothing.
pub struct AirStrike {
    /// the `<air-strike>` block: the seconds between calls and the effect record (FUN_0013b5e0 keeps
    /// settings +0x2f0 at skill+0x40 and FUN_0013de50 copies the record)
    settings: AirStrikeSkill,
    /// skill+0x3c: the seconds to the next call while on, set to `settings.frequency` at each call and
    /// to 0 by the toggle that switches it off (the constructor sets it to 0)
    timer: f32,
    /// the strikes called since the last press that began them, for the log: the game keeps none
    calls: u32,
}

impl AirStrike {
    /// The skill with the `<air-strike>` settings `settings` (its constructor: the timer 0).
    pub fn new(settings: AirStrikeSkill) -> Self {
        AirStrike { settings, timer: 0.0, calls: 0 }
    }

    /// One call, as the toggle and the update both make it: the spawner asked for (FUN_0013e410(0):
    /// `Skill::fire`, which `effect_skills::fire_effects` carries out this frame with the target point
    /// found anew), the cost off the stamina (FUN_001258d0, clamped to 0..max) and the timer set to
    /// the frequency.
    fn call(&mut self, u: &mut Player, cost: f32, why: &str) {
        u.skill.fire = true;
        u.stamina = (u.stamina - cost).clamp(0.0, u.max_stamina);
        self.timer = self.settings.frequency;
        self.calls += 1;
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: air strike call {} ({why}): stamina -{cost} -> {:.2}, its effect h_{:08x} asked for", u.sim_time, unit_name(u), self.calls,
                     u.stamina, self.settings.effect.effect);
        }
    }
}

impl SkillKind for AirStrike {
    fn name(&self) -> u32 {
        NAME_AIR_STRIKE
    }

    /// FUN_0013b660. On: off (the shared switch's `deactivate` carries FUN_0010e3e0 out) and the timer
    /// cleared. Off with stamina (skill+4, the type's stamina-cost) at least the cost: a call, then
    /// the switch on, which the shared switch answers by FUN_0010e470's gate. Under the cost: nothing.
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on {
            self.timer = 0.0;
            if out.log {
                println!("t={:.2} {}: air strike switched off by the toggle after {} calls, stamina {:.2}", u.sim_time, unit_name(u), self.calls, u.stamina);
            }
            return Toggle::Now { overlay: false };
        }
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        if u.stamina < cost {
            if out.log {
                println!("t={:.2} {}: air strike refused: stamina {:.2} under its cost {cost}", u.sim_time, unit_name(u), u.stamina);
            }
            return Toggle::Refused;
        }
        self.calls = 0;
        self.call(u, cost, "the press");
        Toggle::Now { overlay: false }
    }

    /// No drain: 0 (+0xbc, 0x13b5d0).
    fn drain(&self, _u: &Player, _cost: f32, _dt: f32) -> f32 {
        0.0
    }

    /// FUN_0013b6d0, while on: with the timer at or under the frame's time, a call if the stamina is
    /// there for one (the timer set again, the cost paid), else true, which sends the toggle command
    /// that switches it off (the code's FUN_0010e3e0); otherwise the timer counts down.
    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        if !u.skill.on {
            return false;
        }
        if self.timer > dt {
            self.timer -= dt;
            return false;
        }
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        if u.stamina >= cost {
            self.call(u, cost, "the timer");
            return false;
        }
        true
    }

    fn effect_data(&self) -> Option<&EffectData> {
        Some(&self.settings.effect)
    }

    /// (+0x10c, 0x213690: the AI's goal needs a target for it, as for the Orbital Beam, which the demo's
    /// goal can't give: it refuses, play_orders.rs `toggle_ability`)
    fn attack_skill(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::character::{CombatSkills, SkillData};
    use std::time::Duration;

    /// A Militia of the test map: the Air Strike with the type's cost 20 and recovery 1, a bar of
    /// `stamina`, in a mini app that runs the skills' frame (`skills::update_skills`) on a clock the
    /// test advances.
    fn militia(stamina: f32) -> App {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_AIR_STRIKE, max_stamina: 100.0, cost: MILITIA_COST, recovery: MILITIA_RECOVERY,
                                                       stamina_update: true, ..Default::default() }), &CombatSkills::default());
        u.max_stamina = 100.0;
        u.stamina = stamina;
        let mut app = App::new();
        app.insert_resource(Time::<()>::default()).insert_resource(u).insert_resource(Squad::default())
            .insert_resource(ammo::Reserve::default()).add_systems(Update, skills::update_skills);
        app
    }

    /// One frame of `dt` s: with `press` the key's command, which `skills::skill_input` leaves as
    /// the request. Whether the skill asked for the spawner in it (and `fire_effects`' clearing it).
    fn frame(app: &mut App, dt: f32, press: bool) -> bool {
        if press {
            app.world_mut().resource_mut::<Player>().skill.request = true;
        }
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(dt));
        app.update();
        std::mem::take(&mut app.world_mut().resource_mut::<Player>().skill.fire)
    }

    /// The bar and the switch after a frame.
    fn state(app: &App) -> (f32, bool) {
        let u = app.world().resource::<Player>();
        (u.stamina, u.skill.on)
    }

    /// A press from a full bar at the game's 30 Hz and nothing after: five calls (take05), 22, 23,
    /// 23 and 23 frames apart (the first press's frame counts the timer down too: the update runs
    /// after the command), each taking 20 (80, 60, 40, 20, 0: the takes' step values, exactly), the
    /// skill on through them and off in the frame after the fifth, whose refill is that frame's
    /// (take05: 0.03), and no sixth.
    #[test]
    fn five_calls_from_a_full_bar_then_off() {
        let dt = 1.0 / 30.0;
        let mut app = militia(100.0);
        let (mut at, mut left) = (vec![], vec![]);
        let mut off = None;
        for n in 0..200 {
            let called = frame(&mut app, dt, n == 0);
            let (s, on) = state(&app);
            if called {
                at.push(n);
                left.push(s);
            }
            if !on && off.is_none() && n > 0 {
                off = Some((n, s));
            }
        }
        assert_eq!(at, [0, 22, 45, 68, 91]);
        assert_eq!(left, [80.0, 60.0, 40.0, 20.0, 0.0]);
        let (n, s) = off.unwrap();
        assert_eq!(n, 92, "off in the frame after the fifth call");
        assert!((s - dt * MILITIA_RECOVERY).abs() < 1e-5, "its refill: {s}");
        // (the 0.75 s of the record: 0.733 s the first, 0.767 s the others at 30 Hz; the takes' 0.756 s a call
        // over 17 intervals is the game's clock against the log's)
        assert!((at[1] - at[0]) as f32 * dt < 0.75 && (at[2] - at[1]) as f32 * dt >= 0.75);
    }

    /// At 60 Hz the interval is the record's 0.75 s: 45 frames, to a frame's rounding.
    #[test]
    fn at_60_hz_the_interval_is_the_records() {
        let dt = 1.0 / 60.0;
        let mut app = militia(100.0);
        let mut at = vec![];
        for n in 0..400 {
            if frame(&mut app, dt, n == 0) {
                at.push(n);
            }
        }
        assert_eq!(at.len(), 5);
        for w in at.windows(2) {
            assert!((44..=46).contains(&(w[1] - w[0])), "{at:?}");
        }
    }

    /// The press again while on (take06: two calls, then the second press at 3.7 s): off at once, the
    /// timer cleared and the stamina left as it was, no third call and no refill in that frame's
    /// first part (the refill is the off frame's own, +0.03 as take06's 60.06), and the calls made
    /// are two (their effects land by themselves).
    #[test]
    fn a_second_press_switches_it_off_at_once() {
        let dt = 1.0 / 30.0;
        let mut app = militia(100.0);
        let mut calls = 0;
        for n in 0..80 {
            // (the second press after 1.2 s, as take06's 2.5 s and 3.7 s)
            if frame(&mut app, dt, n == 0 || n == 36) {
                calls += 1;
            }
            if n == 36 {
                let (s, on) = state(&app);
                assert!(!on, "off in the press's frame");
                assert!((s - (60.0 + dt)).abs() < 1e-4, "{s}: 60 and the frame's refill");
            }
        }
        assert_eq!(calls, 2);
        assert_eq!(state(&app).1, false);
    }

    /// A press under the cost does nothing at all (no call, no switch, no payment, only the refill); one
    /// at the cost exactly calls (the code's `cost <= stamina`, equality in) and pays it all, and the
    /// switch on is refused with 0 left (under 10%): the single strike of the next test.
    #[test]
    fn the_cost_gates_the_press() {
        let dt = 1.0 / 30.0;
        let mut app = militia(19.9);
        assert!(!frame(&mut app, dt, true));
        let (s, on) = state(&app);
        assert!(!on && s > 19.9 && s < 20.0, "refused, refilling: {s}");
        let mut app = militia(MILITIA_COST);
        assert!(frame(&mut app, dt, true));
        let (s, on) = state(&app);
        assert!(!on && (s - dt * MILITIA_RECOVERY).abs() < 1e-5, "paid in full, never on: {s}");
    }

    /// A press at 20-29.9 pays and calls, and the switch on is refused (FUN_0010e470's 10% of the maximum,
    /// tested on the stamina left): one strike, never on, so no more (the code only: not filmed).
    #[test]
    fn a_press_at_twenty_something_fires_one_strike_and_never_goes_on() {
        let dt = 1.0 / 30.0;
        let mut app = militia(25.0);
        assert!(frame(&mut app, dt, true));
        let (s, on) = state(&app);
        assert!(!on, "the 5 left is under 10% of 100");
        assert!(s > 5.0 && s < 5.1, "paid 20 and the off frame's refill: {s}");
        let mut calls = 0;
        for _ in 0..200 {
            calls += frame(&mut app, dt, false) as usize;
        }
        assert_eq!(calls, 0);
        // (at 30 the 10 left is enough, 10% of the maximum exactly: it goes on)
        let mut app = militia(30.0);
        assert!(frame(&mut app, dt, true));
        assert!(state(&app).1);
    }

    /// A bar that isn't a multiple of 20 ends by the update's own check, not the stamina rule's: from
    /// 70, calls at 70, 50 and 30, and at the fourth timer's end the 10 left is under the cost so the
    /// skill is switched off with that 10 in the bar (the code's FUN_0013b6d0 else-branch; not filmed).
    #[test]
    fn a_bar_that_is_not_a_multiple_of_the_cost_ends_by_the_updates_own_check() {
        let dt = 1.0 / 30.0;
        let mut app = militia(70.0);
        let mut left = vec![];
        let mut off = None;
        for n in 0..200 {
            if frame(&mut app, dt, n == 0) {
                left.push(state(&app).0);
            }
            if off.is_none() && n > 0 && !state(&app).1 {
                off = Some(state(&app).0);
            }
        }
        assert_eq!(left, [50.0, 30.0, 10.0]);
        let s = off.unwrap();
        assert!(s >= 10.0 && s < 10.1, "{s}");
    }

    /// The spawner's turn with the Air Strike's pitch (1 rad, `orientation-pitch`): the effect's up axis
    /// leans 1 rad off the vertical (its y is cos 1) toward a side that changes from strike to strike, as
    /// each shell's flare comes in from another side in the takes (take05, 07, 13, 17). The other
    /// order of the two turns (the yaw about the effect's own axis, then the tilt) would lean every
    /// one the same way, and its up axis is the one vector.
    #[test]
    fn the_turn_leans_by_the_pitch_toward_a_random_side() {
        let data = EffectData::air_strike();
        let mut u = Player::new(3);
        let ups: Vec<Vec3> = (0..40).map(|_| effect_skills::turn(&mut u, &data) * Vec3::Y).collect();
        for up in &ups {
            assert!((up.y - data.pitch.cos()).abs() < 1e-4, "{up}: not {} rad off the vertical", data.pitch);
        }
        let azimuth = |v: &Vec3| v.z.atan2(v.x);
        let (least, most) = ups.iter().map(azimuth).fold((f32::MAX, f32::MIN), |(lo, hi), a| (lo.min(a), hi.max(a)));
        assert!(most - least > 3.0, "the sides leaned to: {least} to {most}");
        // (the fixed-side order: Rx(pitch) after Ry(yaw), whose up axis never moves with the yaw)
        let fixed: Vec<Vec3> = [0.3f32, 1.7, 4.0].iter().map(|&yaw| Quat::from_rotation_x(data.pitch) * Quat::from_rotation_y(yaw) * Vec3::Y).collect();
        assert!(fixed.iter().all(|v| (*v - fixed[0]).length() < 1e-5));
    }

    /// The class: its name is the data's "Air Strike" id, the drain is nothing, the AI's goal
    /// counts it an attack skill, the record is the Air Strike's, and the class is made for skill-set 6.
    #[test]
    fn the_class_is_the_air_strikes() {
        let k = AirStrike::new(AirStrikeSkill::default());
        let u = Player::new(3);
        assert_eq!((k.name(), k.drain(&u, 20.0, 1.0)), (0xF91B_25DB, 0.0));
        assert!(k.attack_skill());
        assert_eq!(k.effect_data(), Some(&EffectData::air_strike()));
        let made = skills::Skill::new(Some(&SkillData { skill_set: 6, cost: 20.0, recovery: 1.0, ..Default::default() }), &CombatSkills::default());
        assert_eq!(made.name(), Some(0xF91B_25DB));
    }
}
