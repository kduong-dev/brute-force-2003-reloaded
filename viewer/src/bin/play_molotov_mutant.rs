//! The Molotov Mutant's skill (#211): CSKILL_MOLOTOV_MUTANT 20, the campaign's e01, e23, m03_a and
//! m03_b mutants who run at the squad with a lit bottle in the left hand and blow themselves up
//! within 5 m (the e23 type h_f0c5e50e: stamina cost 3, recovery 1.5, 55 HP). One of the effect
//! skills (play_effect_skills.rs: the spawner and the damage areas are #200's, whose three dispatch
//! points each have an arm for it); its own part is here: the class, its bottle, and what it does
//! to the character.
//!
//! **What the game does** (AI only: no character a player controls has the skill; the code is in
//! viewer/docs/skills/molotov-mutant.md and todo/npc-skills/answer.md §3.1, todo/132-mutants
//! leads §4):
//!  - **The class** (vtable 0x39cd68, constructor FUN_0013db80): the boss base's (FUN_0013b770, the
//!    Supermutant's and Shadoon's too) with the `<molotov-mutant>` block of game-options
//!    (`MolotovSkill`): its `<effect-data>` (the cloud h_1a256542, count 1, reach 1), the
//!    animation-duration 3 and the bottle (h_e8eb77f8) and hardpoint (h_f8474385) of its skill item
//!    holder (+0x68 0x13dbc0 makes it, +0x5c 0x13dc10 fills it at the character's creation: the
//!    bottle is always in the left hand, lit).
//!  - **The toggle** (0x13b830, CT_TOGGLE_SPECIAL; sent by GOAL_ATTACK_MOLOTOV, FUN_000370b0, once
//!    the squared distance between the mutant's and its target's root nodes is under 25: 5 m): off,
//!    no motion request pending, a current MOTION with a motion type 45 transition, the overlay
//!    test (as the Wrath's) and a stamina of at least the cost x animation-duration (3 x 3 = 9; equal
//!    passes); then FUN_0010e470 (on) and return 2: the MOTION h_fe5d7332 `molotov`
//!    (`Sc_w1_molotov`, 0.73 s, `transition="0.1"`, holds the character where it stands).
//!  - **The clip's events** (the boss base's handler 0x13b800): 19f8311b at 0.57 s (nothing here),
//!    **04592c53 at 0.70 s**, which calls the class's +0x138 (FUN_0013dce0: the holder's bottle is let
//!    go, FUN_00223160(500.0, DTYPE_FLAME) with no attacker: 500 FLAME to the mutant itself, through
//!    its damage handler) and then the spawner FUN_0013e410(0) at the boss base's target point
//!    (+0x134 0x13b7c0: the character's own root node), the cloud h_1a256542: ALE h_e49f4396, light
//!    h_ef1056ea, sound h_02bcfca8, **78 FLAME a second for 1 s within 7 m, linear falloff**, which
//!    hurts anyone in range, the mutant's side too; and ead033a3 at 0.73 s, the generic toggle
//!    event: off.
//!  - **Its death** (+0x70 FUN_0013dc90, from the ragdoll build FUN_00127290 and the vaporise
//!    FUN_001252b0): a bottle still held is let go and the spawner called, so a mutant shot dead on
//!    its run bursts where it falls (todo/132-mutants take11: the kill's score at 5.60 s, the
//!    fireball at 5.67). A mutant killed after the event holds none.
//!  - **The footage** (todo/132-mutants take02, 09, 10: the squad's HP round the burst): the cloud is
//!    centred on the mutant's own node, not on where the clip's root travel leaps its picture to
//!    (the 2.37 m forward the animation carries, which the picture shows but the node doesn't take:
//!    take02's mutant leaps to Tex's shoulder 4.2 m from the burst, which hurt Tex 18.3 of 78 x
//!    (1 - 4.2 / 7) x 0.6, one cloud and no more).
//!
//! **What the demo does** (the product owner's design for a player who controls a Molotov Mutant,
//! 2026-11-08, #211: demo-own, the game's version is the AI's):
//!  - **Press V**: he lights the bottle and holds it in his left hand (`Bottle`, the model and flame
//!    of the throwable Molotov's, h_0157c126 and h_f61f99b7, held at the hardpoint the game's holder
//!    is at by the bottle's foregrip), the game's toggle gates in front of it (an MT 45 transition from his motion, free to
//!    act, a stamina of 9 and the shared 10% of the maximum), the skill on: the HUD names it
//!    ("Self-Destruct", the demo's pick: the boss base's name slot returns 0, the string table's
//!    only skill name for the idea is CSKILL_SELF_DESTRUCT 11's, FUN_0013ef10), the stamina drains at
//!    the class's cost, 3 a second, about 33 s from full.
//!  - **Holding**: he can move, jump, dodge or cancel, and nothing else: no fire and no aim (the
//!    controlled character's `fire` and `aim` are cleared before his step, `hands_full`), and he
//!    isn't free to act (`lets_act`: no reload, weapon switch, grenade, medkit, pickup or crouch
//!    key, the game's rule for a skill that's on, as the Wrath's). An AI one holds fire.
//!  - **Press V again**: the bottle is put away (the skill's toggle while it's on, the same command
//!    the stamina's 0 sends): the stamina spent is gone and he refills at the recovery.
//!  - **A hostile within 5 m** (`watch_foes`: the game's compare, on the root nodes of the two): the
//!    lunge, the game's own toggle (`Toggle::Pose`: the MT 45 motion alone, he's on already): he
//!    turns to the nearest, the `molotov` clip plays whole-body (`Toggling::pose`, held where he
//!    stands), and at its 04592c53 event (0.70 s) the bottle is let go and he takes 500 FLAME
//!    (`self_burst`: through his damage handler, `Hit::land`, no attacker) and the cloud starts at
//!    his root node (`Skill::fire`, `SkillKind::own_point`): the others in 7 m take 78 a second x
//!    (1 - d / 7) for a second, their own factors and step. At the clip's 0.73 s he's off.
//!  - **The stamina runs out while holding** (the command the stamina update sends at 0): the bottle
//!    fizzles, put away, no burst. **Killed while holding** (`SkillKind::off` at the death): the bottle
//!    bursts where he falls, as FUN_0013dc90's.
//!
//! The bottle that falls loose at the event isn't built: the game's holder lets it go (holder +0x38)
//! and the grenade rules would set it off at its first contact (fuse 0, 13-26 FLAME within 7 m, h_13fe011e,
//! and the same cloud again: the analyst's reading, not traced), but the takes' damage is one cloud
//! and nothing more (take02: Tex 4.22 m off lost 18.29, 78 x 0.397 x 0.6 = 18.6; Brutus 4.72 m,
//! 16.21 against 15.3; take09 and take10 likewise), so the demo's bottle goes with the burst.
//!
//! Test hooks: `BF_TEST_SQUAD_PRESET="0.3,mutant_horde"` (the Molotov Mutant is slot 1, the player's)
//! or `BF_TEST_SQUAD_SLOT="0.3,me,molotov mutant@e23"`, `BF_TEST_GIVE_SKILL=20` (the cost 3 and
//! recovery 1.5), `BF_TEST_SKILL=<s>[,<s>...]` for the V presses, `BF_TEST_NPC` for the hostiles,
//! `BF_SKILL_LOG=1` for the toggles, the lunge and its burst, the stamina's half seconds and the cloud
//! started, `BF_DAMAGE_LOG=1` / `BF_COMBAT_LOG=1` for what the cloud and the burst do to whom.

use super::*;
use super::skills::{Outside, SkillKind, Toggle};
use bf_viewer::bf::character::{CombatSkills, EffectData, MolotovSkill};
use bf_viewer::ale_fx::{AleAssets, AleEffect, Compiled};
use bevy::render::mesh::skinning::SkinnedMeshInverseBindposes;

/// The Molotov Mutant's skill-set (CSKILL_MOLOTOV_MUTANT, the name table 0x3bd814; the factory
/// FUN_0012e590 makes it with FUN_0013db80).
pub const CSKILL_MOLOTOV_MUTANT: i64 = 20;
/// The e23 Molotov Mutant's cost and recovery (objecttypes-e23.xml's h_f0c5e50e, `<combat-skill
/// skill-set="20" h_105c13aa="3" h_f2a2beb9="1.5">`; e01's h_fc30a700 and m03's h_10a12c64 the same),
/// which BF_TEST_GIVE_SKILL=20 gives by default (play_effect_skills.rs `give_defaults`) and the
/// demo's Molotov Mutant class keeps (character_class.rs `MOLOTOV`).
pub const MUTANT_COST: f32 = 3.0;
pub const MUTANT_RECOVERY: f32 = 1.5;
/// The HUD's name for it: "Self-Destruct", h_01fdf1ca in common/stringtable/string-table-en.xml. The
/// boss base's name slot (+0x4c, 0x2bf0a0) returns 0, so the game's skill shows none; the demo's
/// player design needs one, and this is the table's one skill name for the idea: CSKILL_SELF_DESTRUCT
/// 11's own (its vtable 0x39c870 has nothing but the name, 0x13ef10 returning 0x1fdf1ca; no character
/// has the class). The demo's pick.
const NAME: u32 = 0x01FD_F1CA;
/// How close a hostile's root node must come for the lunge, m: GOAL_ATTACK_MOLOTOV's run
/// (FUN_000370b0) sends the toggle once the squared distance between its character's and its
/// target's root nodes is under 25 (the compare at 0x37240).
const FOE_REACH: f32 = 5.0;
/// The animation event the boss base's handler (0x13b800: `cmp eax, 0x4592c53`) answers: the
/// `molotov` clip's, at 0.70 s, where the bottle is let go (FUN_0013dce0) and the cloud spawned.
const EV_BURST: u32 = 0x0459_2C53;
/// The generic toggle event (the code map's 0x118bbc: it switches an on skill off) at the clip's
/// end, 0.73 s into `molotov`; the same hash Hawk's activate_stealth carries (play_skills.rs
/// `EV_STEALTH`).
const EV_END: u32 = 0xEAD0_33A3;
/// The two events' times in the `molotov` clip (`Sc_w1_molotov`, 0.73 s: `BF_ANIM_PROBE`, the
/// leads' §4.2), for a character whose pose clip has no such event (BF_TEST_GIVE_SKILL on a squad
/// member, whose own overlay stands in for the type's motion).
const BURST_AT: f32 = 0.70;
const END_AT: f32 = 0.73;
/// What the bottle's burst deals the mutant: FUN_0013dce0 pushes 0x43fa0000, 500.0, and the damage
/// type 5 (DTYPE_FLAME) for FUN_00223160, with no attacker (EDI 0: 0x13dd16 `xor edi, edi`).
const SELF_FLAME: f32 = 500.0;
const DTYPE_FLAME: i64 = 5;
/// How long the lunge's pose blends in over: the MOTION's `transition` ("0.1", h_fe5d7332), and it
/// doesn't blend out before the clip's end (the burst kills him at 0.70 s, the clip ends at 0.73:
/// FADE's 0.2 s out would have taken the pose back to the idle by the death).
const POSE_BLEND_IN: f32 = 0.1;
/// The bottle's flame sound (the effect type h_f61f99b7's one `<Sound sound-id="h_f6dbdb17"
/// play-length="0.51">`, played as the effect starts) is queued this loud, as a thrown grenade's
/// trail sounds are (play_grenade.rs `trail`).
const FLAME_VOLUME: f32 = 0.8;

/// The bottle's flame on its model: the attached-effect list of the inventory-grenade h_e8eb77f8
/// (objecttypes-e23.xml 5847) names the hardpoint h_e9680caa, which the model's archetype h_0157c126
/// (objects-e23.xml 12026) doesn't have: it has two, `FOREGRIP` (h_ea5e02f2, on the body: where the
/// support hand holds it) and h_05517dfe, 0.215 m up its axis (the bottle spans -0.234 to 0.234). The
/// game finds the entry's place by a node's name (FUN_000cdd10 through FUN_0023c4e0), and with none of
/// that name the effect hangs on the object's own node: the model's origin, in its own frame. The
/// flame effect agrees: boss_mutant_bottlefire's emitter (`common` ALE library, h_fd5dee85) sits
/// 0.16 m along +y of its origin (its xform's y, 0.1603), just under the neck, and its smoke and
/// particles rise along +y, the bottle's axis. So the demo hangs it at the model's origin, unturned.
const FLAME_AT: Vec3 = Vec3::ZERO;

/// The skill object of a skill-set 20 `<combat-skill>`'s character, with game-options' block
/// (FUN_0013db80): play_effect_skills.rs `kind`'s arm for the class.
pub fn kind(settings: &CombatSkills) -> Box<dyn SkillKind> {
    Box::new(MolotovMutant { s: settings.molotov_mutant, ..Default::default() })
}

/// Whether `u`'s current motion has a motion type 45 transition, which the toggle asks of its MOTION
/// at 0x13b862 (+0x534; none, and it refuses). In ms_mutant2.xml (the Filthy, Molotov and Ugly
/// Mutants' set) the idles (MT 19), the walk, the run, the side steps and the back walk, the
/// crouch idle (MT 6) and the dashes (MT 8) have one; the stand-to-crouch and back (MT 46), the
/// throw (MT 42), the dodges (MT 15-18), the dive, the stun, the knock-down and the jumps (MT 20,
/// 21, 24; MT 22's landing has one, MT 23's none) don't. The demo's states: standing, walking,
/// running, dashing and kneeling, not sliding or knocked down, and with no grenade in the hand
/// (charging, thrown or set down, its release pending: the demo's reading of the throw).
fn has_transition(u: &Player) -> bool {
    matches!(u.action, Action::None | Action::Crouched) && !u.slide_on && !knocked_down(u)
        && u.throwing.is_none() && u.charge <= 0.0 && u.pending_release.is_none()
}

/// Whether `u` has an overlay under way that the toggle's own test (0x13b86f-0x13b8a7: the current
/// MOTION blocks overlays and both overlay slots are busy) would refuse on, as the demo reads it,
/// for any overlay (play_wrath.rs: a simplification, `skills::free_to_act` less the skill's own
/// `lets_act`, which is false while the bottle is held). A reload, a switch, a throw, a medkit, a
/// weapon pickup or a skill's pose.
fn overlay_busy(u: &Player) -> bool {
    u.reloading.is_some() || u.throwing.is_some() || u.using.is_some() || u.switching.is_some() || u.gun_use.is_some() || u.skill.toggling.is_some()
}

/// Where the skill is: off, holding the bottle (on, draining), or in the lunge (the `molotov`
/// clip playing, the bottle in the hand until its event).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Off,
    Holding,
    Lunging,
}

/// The Molotov Mutant (vtable 0x39cd68, constructor FUN_0013db80): see the module notes.
#[derive(Default)]
struct MolotovMutant {
    /// game-options' block, as the constructor copies it
    s: MolotovSkill,
    phase: Phase,
    /// the bottle is in the left hand: the holder's item (holder+0x20), from the first press, let go
    /// at the clip's event or put away. FUN_0013dc90's test at the death
    bottle: bool,
    /// `update` has found a hostile within the reach and asks the lunge of the toggle (set before
    /// it returns true, the command's way into `toggle`)
    asked: bool,
    /// the lunge's clock, s: 0 in the frame the toggle starts the clip (the command comes out of
    /// `update`, which isn't called again in that step) and counted by each `update` after it, as the
    /// clip's own time is (`Toggling::time`, advanced at the top of each step)
    clock: f32,
    /// the lunge's two events' times, from the clip (`times`)
    burst_at: f32,
    end_at: f32,
    /// the burst (04592c53) has happened in this lunge
    burst: bool,
    /// the clip's end (ead033a3) has come: the toggle answers the command with the off
    finished: bool,
}

impl MolotovMutant {
    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: molotov refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }

    /// The times of the pose clip's burst and end events, for the stance he holds his gun in: the
    /// `molotov` clip's 04592c53 (0.70 s) and ead033a3 (0.73 s); a character whose pose clip lacks
    /// them (a squad member the hook gave the skill: his overlay stands in) gets the clip's numbers.
    fn times(u: &Player) -> (f32, f32) {
        let clip = u.loaded.as_ref().and_then(|l| {
            let stance = stance_of(l, u.weapon);
            l.pose_clips[stance].as_ref().or(l.skill_clips[0][stance].as_ref())
        });
        let burst = clip.and_then(|c| c.event(EV_BURST)).unwrap_or(BURST_AT);
        let end = clip.and_then(|c| c.event(EV_END)).unwrap_or(burst + END_AT - BURST_AT).max(burst);
        (burst, end)
    }

    /// The lunge's start (the toggle's return 2: the pose is asked of the command handler): he turns
    /// to the hostile that brought it on, and the clock starts.
    fn start_lunge(&mut self, u: &mut Player) {
        if let Some(foe) = u.skill.foe {
            let d = foe - u.position;
            if d.x != 0.0 || d.z != 0.0 {
                u.yaw = (-d.x).atan2(-d.z);
            }
        }
        (self.burst_at, self.end_at) = Self::times(u);
        (self.phase, self.clock, self.burst, self.finished) = (Phase::Lunging, 0.0, false, false);
    }

    /// The clip's 04592c53 event (0x13b800 -> +0x138 FUN_0013dce0, then the spawner FUN_0013e410(0)): the
    /// bottle is let go and he takes 500 FLAME (`Skill::self_damage`, `self_burst`), and the cloud
    /// is asked of the spawner (`Skill::fire`, at his own root node: `own_point`).
    fn let_go(&mut self, u: &mut Player) {
        self.burst = true;
        self.bottle = false;
        u.skill.self_damage = Some((SELF_FLAME, DTYPE_FLAME));
        u.skill.fire = true;
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: the clip's 04592c53 at {:.2} s: the bottle is let go, {SELF_FLAME} FLAME to himself, the cloud h_{:08x} at his root node",
                     u.sim_time, unit_name(u), self.clock, self.s.effect.effect);
        }
    }
}

impl SkillKind for MolotovMutant {
    fn name(&self) -> u32 {
        NAME
    }

    /// 0x13b830 and the demo's player design. **Off**: a motion with a motion type 45 transition
    /// (`has_transition`), no overlay under way (`overlay_busy`) and a stamina of at least the cost x
    /// animation-duration (3 x 3 = 9, equal passes: `fcompp` / `test ah, 0x41` / `jp` at 0x13b8b3);
    /// then the shared switch on (the 10% of the maximum, the line), which lights the bottle (`on`).
    /// **On, holding**: the lunge `update` asked for (the game's own toggle with the skill on already:
    /// the same gates, then the pose, `Toggle::Pose`), else a press or the stamina's 0: the bottle is
    /// put away (`Toggle::Now`, the off). **On, in the lunge**: refused, as the game's toggle refuses
    /// while on (0x13b830's first test), but the clip's end (`finished`), which switches him off.
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on {
            return match self.phase {
                Phase::Lunging if self.finished => {
                    self.finished = false;
                    Toggle::Now { overlay: false }
                }
                Phase::Lunging => Self::refused(u, out, "the lunge is under way"),
                Phase::Holding if self.asked => {
                    self.asked = false;
                    if !has_transition(u) || overlay_busy(u) {
                        return Self::refused(u, out, "no motion type 45 transition from his motion, or busy");
                    }
                    self.start_lunge(u);
                    if out.log {
                        println!("t={:.2} {}: molotov: a hostile within {FOE_REACH} m: the lunge ({:.2} s to the burst)", u.sim_time, unit_name(u), self.burst_at);
                    }
                    Toggle::Pose
                }
                _ => {
                    if out.log {
                        println!("t={:.2} {}: molotov: the bottle is put away at stamina {:.2}{}", u.sim_time, unit_name(u), u.stamina,
                                 if u.stamina <= 0.0 { " (out of stamina: it fizzles)" } else { "" });
                    }
                    Toggle::Now { overlay: false }
                }
            };
        }
        if !has_transition(u) {
            return Self::refused(u, out, "no motion type 45 transition from his motion");
        }
        if overlay_busy(u) {
            return Self::refused(u, out, "busy");
        }
        let cost = u.skill.data.as_ref().map_or(f32::MAX, |d| d.cost);
        let needs = cost * self.s.duration;
        if u.stamina < needs {
            return Self::refused(u, out, &format!("stamina {:.2} under {cost} x {} = {needs}", u.stamina, self.s.duration));
        }
        Toggle::Now { overlay: false }
    }

    /// The skill switched on (FUN_0010e470 inside the toggle): the bottle is lit and in his hand
    /// (the holder filled, 0x13dc10).
    fn on(&mut self, u: &mut Player, _out: &mut Outside) {
        (self.phase, self.bottle, self.asked, self.finished, self.burst) = (Phase::Holding, true, false, false, false);
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: molotov: the bottle is lit and in his left hand, stamina {:.2} draining {} a second", u.sim_time, unit_name(u), u.stamina,
                     u.skill.data.as_ref().map_or(0.0, |d| d.cost));
        }
    }

    /// The skill switched off: put away or fizzled (the toggle while holding, the stamina at 0), the
    /// lunge over, or the death (the handler 0x115df5 -> FUN_0010e3e0). At the death a bottle still
    /// held is let go and the spawner called (FUN_0013dc90, +0x70: the cloud at the character); one
    /// let go at the event, or put away, isn't.
    fn off(&mut self, u: &mut Player) {
        if u.dead && self.bottle {
            u.skill.fire = true;
            if std::env::var("BF_SKILL_LOG").is_ok() {
                println!("t={:.2} {}: molotov: killed holding the bottle: it bursts where he falls (the cloud h_{:08x})", u.sim_time, unit_name(u),
                         self.s.effect.effect);
            }
        }
        if !u.dead && self.bottle && u.stamina <= 0.0 && std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: molotov: out of stamina, the bottle fizzles: no burst", u.sim_time, unit_name(u));
        }
        (self.phase, self.bottle, self.asked, self.finished) = (Phase::Off, false, false, false);
    }

    /// Every frame, on or off, while alive (0xc0). Holding: a hostile within the reach
    /// (`Skill::foe`, `watch_foes`) asks for the lunge, once his motion can take it (the command is
    /// sent, `toggle` answers). In the lunge: the clock; the burst at the clip's 04592c53, the end
    /// at its ead033a3 (the command, which `toggle` answers with the off); a knock-down cuts the
    /// clip short (play_skills.rs `step` drops the pose), and he's holding again.
    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        match self.phase {
            Phase::Off => false,
            Phase::Holding => {
                if u.skill.foe.is_some() && has_transition(u) && !overlay_busy(u) {
                    self.asked = true;
                    return true;
                }
                false
            }
            Phase::Lunging => {
                if knocked_down(u) && !self.burst {
                    self.phase = Phase::Holding;
                    return false;
                }
                self.clock += dt;
                if !self.burst && self.clock >= self.burst_at {
                    self.let_go(u);
                }
                if self.clock >= self.end_at {
                    self.finished = true;
                    return true;
                }
                false
            }
        }
    }

    /// (+0x14, 0x13b980 as the Wrath's: free to act only while the skill is off: while he holds the
    /// bottle his hands are full, so no reload, weapon switch, grenade, medkit, pickup or crouch key)
    fn lets_act(&self, u: &Player) -> bool {
        !u.skill.on
    }

    /// (+0x108: the AI's attack check refuses while it's true; the demo's: it holds fire while the
    /// bottle is in its hand, as the player's is held back, `hands_full`)
    fn holds_fire(&self, _u: &Player) -> bool {
        self.bottle
    }

    fn effect_data(&self) -> Option<&EffectData> {
        Some(&self.s.effect)
    }

    /// (+0x10c, 0x213690: an attack skill; the demo's squad-order goal refuses it,
    /// play_orders.rs `toggle_ability`: GOAL_ATTACK_MOLOTOV's own use is #374's)
    fn attack_skill(&self) -> bool {
        true
    }

    /// (+0x134, 0x13b7c0: the boss base's target point is the character's own root node)
    fn own_point(&self) -> bool {
        true
    }

    fn foe_reach(&self) -> Option<f32> {
        Some(FOE_REACH)
    }

    fn bottle(&self) -> bool {
        self.bottle
    }

    /// The `molotov` MOTION's `transition` in, and no blend out: see `POSE_BLEND_IN`.
    fn pose_blend(&self) -> (f32, f32) {
        (POSE_BLEND_IN, 0.0)
    }
}

/// The bottle's model and flame, loaded once a map (`load_bottle`): the skill's item h_e8eb77f8's
/// archetype (the throwable Molotov's mesh h_0157c126), the ALE effect of its attached effect type
/// h_f61f99b7 and that type's sound. The flame hangs at the model's origin (`FLAME_AT`).
#[derive(Resource, Default)]
struct Bottle {
    /// the model's meshes and materials with each part's place in the model's frame
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform)>,
    /// the attached effect type's ALE effects, compiled with the game's count and shapes
    /// (`AleAssets::load_recorded_facing`, as the hits' and the triggers' effects are)
    flames: Vec<Arc<Compiled>>,
    /// where the support hand holds it (the model's `FOREGRIP`): the bottle is mounted by it, as a
    /// gun is by its grip; None without one (the bottle's origin at the hand)
    grip: Option<Hardpoint>,
    /// the attached effect type's sounds, queued as it starts
    sounds: Vec<u32>,
}

/// A bottle in the left hand of the character `Who` (the model, and its flame as a child).
#[derive(Component)]
struct BottleOf(Who);

/// The systems: `hands_full` after the controls and the test hooks that press or aim, before the
/// characters' step; `watch_foes` after it (the roots are where the step left them), before the
/// skills' frame (whose `update` reads the foe); `self_burst` and `show_bottle` after the skills'
/// frame; and the bottle's load as a map starts, after the setup.
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), load_bottle.after(setup))
        .add_systems(Update, (hands_full.after(read_input).after(squad_control).after(ammo::test_trigger).before(update_player),
                              watch_foes.after(update_player).before(skills::update_skills),
                              (self_burst, show_bottle).after(skills::update_skills))
            .run_if(in_state(AppState::Playing)));
}

/// While the controlled character holds the bottle his hands are full: no fire and no aim (the
/// product owner's design: "bottle only", he can move or cancel). Before his step, after every
/// control and hook that sets either.
fn hands_full(mut player: ResMut<Player>) {
    if skills::holds_bottle(&player) {
        player.fire = false;
        player.aim = false;
    }
}

/// The nearest hostile character within the skill's reach of each character whose skill names
/// one and is on (`SkillKind::foe_reach`), into `Skill::foe` for the skill's `update`: living, in the
/// world and drawn, on a side hostile to his (`damage::hostile`, the team table 0x3ffd80 FUN_000bf6d0
/// reads), and its root node (`missile::root_point`, the vtable +0x40 record GOAL_ATTACK_MOLOTOV's
/// compare reads, FUN_000370b0) under the reach by the squared distance in 3D.
fn watch_foes(mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let all: Vec<(Who, u8, Vec3)> = std::iter::once(&*player).chain(squad.0.iter())
        .filter(|u| !u.dead && !u.absent && !u.stand_in && u.loaded.is_some())
        .map(|u| (u.who(), u.team, missile::root_point(u))).collect();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        u.skill.foe = None;
        let Some(reach) = skills::foe_reach(u) else { continue };
        if !u.skill.on || u.dead || u.loaded.is_none() {
            continue;
        }
        let (me, who, team) = (missile::root_point(u), u.who(), u.team);
        u.skill.foe = all.iter().filter(|&&(w, t, p)| w != who && damage::hostile(team, t) && p.distance_squared(me) < reach * reach)
            .map(|&(_, _, p)| p).min_by(|a, b| a.distance_squared(me).total_cmp(&b.distance_squared(me)));
    }
}

/// The burst's damage to the mutant himself (`Skill::self_damage`, FUN_0013dce0's FUN_00223160(500.0,
/// FLAME) with no attacker): through his damage handler, as any hit reaches a character (0x125600:
/// the difficulty step, his skill's factor, his factor for the type), a bare record (no hit part, no
/// shooter's factors), so a kill by it is shoved straight up with the up-kick merged at the cap
/// (play_ragdoll.rs `death_shoves`, the code map's FUN_00223160 row: #366), his corpse thrown as the
/// footage shows it (take02 7.13 s: the body in the middle of the fireball, limbs out).
fn self_burst(mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, difficulty: Res<damage::Difficulty>) {
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let Some((amount, kind)) = u.skill.self_damage.take() else { continue };
        if u.dead {
            continue;
        }
        let mut hit = damage::Hit::new(amount, "its own burst", kind);
        let dealt = hit.land(u, &game.0, &difficulty, u.sim_time);
        let at = missile::root_point(u);
        let before = u.health;
        hurt(u, &game.0, dealt, chatter::Cause::hit(&hit), Vec3::ZERO, at, -1);
        if u.dead {
            ragdoll::death_shoves(u, ragdoll::Target::None, at, Vec3::ZERO, dealt, kind, ragdoll::area_push(1.0));
        }
        if log {
            println!("t={:.2} {}: takes {dealt:.1} of its own burst ({before:.1} -> {:.1} hp){}", u.sim_time, unit_name(u), u.health, if u.dead { ", dead" } else { "" });
        }
    }
}

/// The bottle's model and flame (`Bottle`) as a map starts: the skill item's definition
/// (`MolotovSkill::item`, the weapon table's h_e8eb77f8) for its archetype and attached effect
/// type, the model's parts as the grenades' are made (`GrenadeKit::load`), the effect's ALE effects
/// compiled and warmed up as the grenades' trail effects are (`load_kits`). One it can't find is
/// said and the bottle is left unseen.
#[allow(clippy::too_many_arguments)]
fn load_bottle(mut commands: Commands, mut game: ResMut<GameData>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
               mut images: ResMut<Assets<Image>>, mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>, ale: Option<ResMut<AleAssets>>) {
    let mut bottle = Bottle::default();
    let item = game.0.combat_skills.molotov_mutant.item;
    match game.0.weapons.get(&item).cloned() {
        None => warn!("molotov mutant: no item h_{item:08x} in the weapon table: no bottle to show"),
        Some(def) => {
            match WeaponModel::load(&game.0, def.archetype) {
                Ok(m) => {
                    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
                    for p in &m.parts {
                        for (mesh, mat) in bf_viewer::scene::static_meshes(&mut game.0, &p.geosets, &mut assets, true) {
                            bottle.parts.push((mesh, mat, Transform::from_translation(p.offset).with_rotation(p.rotation)));
                        }
                    }
                    bottle.grip = m.hardpoints.get(&weapon::FOREGRIP).copied();
                    if std::env::var("BF_SKILL_LOG").is_ok() {
                        for (k, h) in &m.hardpoints {
                            println!("molotov mutant: the bottle's hardpoint h_{k:08x} at {:.3} turned {:.3}", h.point, h.rot);
                        }
                    }
                }
                Err(e) => warn!("molotov mutant: the bottle's model: {e}"),
            }
            if let Some(fx) = game.0.effect_type_defs.get(&def.attached_effect).cloned() {
                bottle.sounds = fx.sounds.iter().filter(|s| s.1 <= 0.0).map(|s| s.0).collect();
                if let Some(mut ale) = ale {
                    for &e in fx.effects.iter().filter(|&&e| e != 0) {
                        if let Some(c) = ale.load_recorded_facing(&mut game.0, &mut images, &mut materials, e) {
                            bf_viewer::ale_fx::warm_up(&mut commands, &ale, &c);
                            bottle.flames.push(c);
                        }
                    }
                }
            }
        }
    }
    info!("molotov mutant: the bottle h_{item:08x}: {} meshes, {} flame effects, sounds {:x?}", bottle.parts.len(), bottle.flames.len(), bottle.sounds);
    commands.insert_resource(bottle);
}

/// A bottle in the left hand of each character who holds one (`skills::holds_bottle`), and none in
/// anyone else's: the model on the joint of the bone the skill's hardpoint is on (`MolotovSkill::
/// hardpoint`, the SUPPORT_HAND of its model), at the hardpoint's point and turned as it is, with its
/// flame on the wick. Lit as it's put there: the flame's sound queued once (the effect type's
/// `<Sound>`, as a thrown grenade's trail is).
fn show_bottle(mut commands: Commands, bottle: Option<Res<Bottle>>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               shown: Query<(Entity, &BottleOf)>) {
    let Some(bottle) = bottle else { return };
    for (e, of) in &shown {
        if !std::iter::once(&*player).chain(squad.0.iter()).any(|u| u.who() == of.0 && skills::holds_bottle(u)) {
            commands.entity(e).despawn();
        }
    }
    let hardpoint = game.0.combat_skills.molotov_mutant.hardpoint;
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if !skills::holds_bottle(u) || shown.iter().any(|(_, of)| of.0 == u.who()) {
            continue;
        }
        let Some((joint, hp)) = u.loaded.as_ref().and_then(|l| l.model.hardpoints.get(&hardpoint).map(|&(bone, hp)| (l.joints[bone], hp))) else {
            continue;
        };
        let (rotation, translation) = bottle.grip.map_or((hp.rot, hp.point), |grip| hp.mount(&grip));
        let e = commands.spawn((Transform::from_translation(translation).with_rotation(rotation), Visibility::Inherited, Name::new("molotov bottle"),
                                BottleOf(u.who()), ChildOf(joint))).id();
        for (mesh, mat, at) in &bottle.parts {
            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), *at, ChildOf(e)));
        }
        for (i, fx) in bottle.flames.iter().enumerate() {
            commands.spawn((Transform::from_translation(FLAME_AT), Visibility::default(),
                            AleEffect::new(fx.clone(), 0.0, 0x4D01_0000 ^ e.index() ^ i as u32), ChildOf(e)));
        }
        for &id in &bottle.sounds {
            u.sound_queue.push((id, FLAME_VOLUME));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::skills::can_activate;
    use bf_viewer::bf::character::SkillData;

    /// A Molotov Mutant as the demo makes him: the e23 type's cost and recovery, max-stamina 100,
    /// the stamina update on (the class rule), the settings' block.
    fn mutant() -> Player {
        let mut u = Player::new(3);
        u.skill = skills::Skill::new(Some(&SkillData { skill_set: CSKILL_MOLOTOV_MUTANT, max_stamina: 100.0, cost: MUTANT_COST, recovery: MUTANT_RECOVERY,
                                                        stamina_update: true, ..Default::default() }), &CombatSkills::default());
        u.max_stamina = 100.0;
        u.stamina = 100.0;
        u
    }

    /// What the shared command does with an answer (play_skills.rs `command`), for the states the
    /// tests need: a `Now` switches the skill (on through the activate's 10% gate, off at once), the
    /// rest leave it.
    fn run(u: &mut Player, k: &mut Box<dyn SkillKind>, reserve: &mut super::super::ammo::Reserve) -> Toggle {
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

    /// FUN_0013db80 and the factory: skill-set 20 makes the class with the block's cloud, its name is
    /// the demo's "Self-Destruct", it's an attack skill whose point is its own, and a type of its own
    /// with the skill-set gets it (the three dispatch points, play_effect_skills.rs).
    #[test]
    fn the_factory_makes_it() {
        let data = SkillData { skill_set: CSKILL_MOLOTOV_MUTANT, cost: MUTANT_COST, recovery: MUTANT_RECOVERY, ..Default::default() };
        assert!(super::super::effect_skills::own_type_skill(Some(&data)).is_some());
        let skill = skills::Skill::new(Some(&data), &CombatSkills::default());
        let k = skill.kind.as_ref().expect("the Molotov Mutant");
        assert_eq!((k.name(), k.effect_data().map(|d| d.effect), k.attack_skill(), k.own_point(), k.foe_reach()), (0x01FD_F1CA, Some(0x1A25_6542), true, true, Some(5.0)));
        assert_eq!((k.bottle(), k.pose_blend()), (false, (0.1, 0.0)));
        assert_eq!(skill.data.as_ref().map(|d| (d.cost, d.recovery)), Some((3.0, 1.5)));
    }

    /// The toggle's gates off (0x13b830): the stamina must reach cost x animation-duration, 3 x 3 = 9,
    /// equal passing; a motion with no motion type 45 transition refuses (ms_mutant2's: the crouch's
    /// transitions, a dodge, a jump, a slide, a knock-down, a grenade in the hand), standing,
    /// walking and kneeling pass; a reload under way is busy.
    #[test]
    fn the_toggle_gates_are_the_games() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut out = Outside { reserve: &mut reserve, log: false };
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        u.stamina = 8.99;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused);
        u.stamina = 9.0;
        assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false });
        let refusals: [(&str, fn(&mut Player)); 7] = [
            ("rising", |u| u.action = Action::Rising),
            ("kneeling down", |u| u.action = Action::Crouching),
            ("a dodge", |u| u.action = Action::Dodge { left: true }),
            ("a jump", |u| u.action = Action::JumpFall),
            ("a slide", |u| u.slide_on = true),
            ("a grenade charging", |u| u.charge = 0.3),
            ("a reload", |u| u.reloading = Some(Reload { weapon: 0, time: 0.0, fill: 10, filled: false })),
        ];
        for (what, set) in refusals {
            let mut u = mutant();
            set(&mut u);
            assert_eq!(k.toggle(&mut u, &mut out), Toggle::Refused, "{what}");
        }
        for action in [Action::None, Action::Crouched] {
            let mut u = mutant();
            u.action = action;
            assert_eq!(k.toggle(&mut u, &mut out), Toggle::Now { overlay: false }, "{action:?}");
        }
    }

    /// The player's design, press by press: V lights the bottle (on, the bottle in the hand, no
    /// fire and no free hands), V again puts it away (off, no burst, the stamina spent stays spent),
    /// and the stamina at 0 does the same through the same command: the bottle fizzles, no cloud.
    #[test]
    fn a_press_lights_it_and_a_press_puts_it_away() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && u.skill.on);
        assert!(k.bottle() && k.holds_fire(&u) && !k.lets_act(&u));
        // (the drain is the base's: the cost a second)
        assert!((k.drain(&u, 3.0, 1.0 / 30.0) - 0.1).abs() < 1e-6);
        u.stamina = 91.0;
        // V again
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && !u.skill.on);
        assert!(!k.bottle() && k.lets_act(&u) && !u.skill.fire && u.skill.self_damage.is_none());
        assert_eq!(u.stamina, 91.0);
        // lit again, then the stamina's 0: the toggle the update sends answers the same
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && u.skill.on);
        u.stamina = 0.0;
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && !u.skill.on);
        assert!(!k.bottle() && !u.skill.fire && u.skill.self_damage.is_none(), "a fizzle bursts nothing");
        // (under 10% of the maximum the switch on is the shared gate's refusal: nothing happens)
        u.stamina = 9.5;
        assert!(run(&mut u, &mut k, &mut reserve) == Toggle::Now { overlay: false } && !u.skill.on);
    }

    /// The lunge: with no hostile near nothing; one within the reach (`Skill::foe`) asks it of the
    /// command, whose toggle turns him to it and returns the pose; the clock then runs from the
    /// next frame, the burst comes at the 0.70 s event (the bottle let go, 500 FLAME to himself, the
    /// cloud asked for, once) and the clip's end at 0.73 s switches him off; a press meanwhile is
    /// refused.
    #[test]
    fn a_hostile_within_five_metres_brings_the_lunge() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        run(&mut u, &mut k, &mut reserve);
        let dt = 1.0 / 30.0;
        assert!(!k.update(&mut u, dt));
        // a hostile off to his +x, 3 m: he turns to it (the model faces -z at yaw 0)
        u.skill.foe = Some(Vec3::new(3.0, 0.0, 0.0));
        assert!(k.update(&mut u, dt), "asks the command for the lunge");
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Pose);
        assert!((u.yaw + std::f32::consts::FRAC_PI_2).abs() < 1e-5, "{}", u.yaw);
        assert!(u.skill.on && k.bottle());
        // a press in the lunge: refused
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Refused);
        // the clock counts from the next frame's update: 0.70 s is 21 frames of 1/30 (22 when the
        // sum of 1/30 falls a rounding under it) and the burst is the first at or past it
        let mut frames = 0;
        while !u.skill.fire {
            assert!(!k.update(&mut u, dt));
            frames += 1;
            assert!(frames < 40, "the burst never comes");
        }
        assert!((21..=22).contains(&frames), "{frames}");
        assert_eq!((u.skill.self_damage, k.bottle()), (Some((500.0, 5)), false));
        // (the end comes at 0.73 s: the next frame at 30 Hz, or the one after)
        u.skill.fire = false;
        u.skill.self_damage = None;
        let mut ended = false;
        for _ in 0..3 {
            ended |= k.update(&mut u, dt);
        }
        assert!(ended && !u.skill.fire && u.skill.self_damage.is_none(), "one burst per lunge");
        assert_eq!(run(&mut u, &mut k, &mut reserve), Toggle::Now { overlay: false });
        assert!(!u.skill.on);
    }

    /// A hostile near while his motion can't take the toggle (a dodge, a jump) waits: no command is sent
    /// until it can; and one near when he isn't holding does nothing.
    #[test]
    fn the_lunge_waits_for_a_motion_that_takes_it() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        u.skill.foe = Some(Vec3::X);
        assert!(!k.update(&mut u, 1.0 / 30.0), "not holding: nothing");
        run(&mut u, &mut k, &mut reserve);
        u.action = Action::Dodge { left: false };
        assert!(!k.update(&mut u, 1.0 / 30.0));
        u.action = Action::None;
        assert!(k.update(&mut u, 1.0 / 30.0));
    }

    /// FUN_0013dc90: killed while the bottle is held, it bursts where he falls (the spawner asked
    /// for); killed after the event (it's let go), or with the bottle put away, nothing does.
    #[test]
    fn killed_holding_it_the_bottle_bursts() {
        let mut reserve = super::super::ammo::Reserve::default();
        let mut u = mutant();
        let mut k = u.skill.kind.take().unwrap();
        run(&mut u, &mut k, &mut reserve);
        u.dead = true;
        k.off(&mut u);
        assert!(u.skill.fire && u.skill.self_damage.is_none(), "the cloud, and no damage to himself");
        // killed in the lunge before the event: the bottle is still in his hand
        let mut v = mutant();
        let mut k = v.skill.kind.take().unwrap();
        run(&mut v, &mut k, &mut reserve);
        v.skill.foe = Some(Vec3::X);
        k.update(&mut v, 0.1);
        run(&mut v, &mut k, &mut reserve);
        v.dead = true;
        k.off(&mut v);
        assert!(v.skill.fire);
        // killed by the burst itself (the event let it go): no second one
        let mut w = mutant();
        let mut k = w.skill.kind.take().unwrap();
        run(&mut w, &mut k, &mut reserve);
        w.skill.foe = Some(Vec3::X);
        k.update(&mut w, 0.1);
        run(&mut w, &mut k, &mut reserve);
        for _ in 0..30 {
            k.update(&mut w, 1.0 / 30.0);
        }
        w.skill.fire = false;
        w.dead = true;
        k.off(&mut w);
        assert!(!w.skill.fire, "a bottle let go isn't let go twice");
        // put away, then killed
        let mut x = mutant();
        let mut k = x.skill.kind.take().unwrap();
        run(&mut x, &mut k, &mut reserve);
        run(&mut x, &mut k, &mut reserve);
        x.dead = true;
        k.off(&mut x);
        assert!(!x.skill.fire);
    }
}
