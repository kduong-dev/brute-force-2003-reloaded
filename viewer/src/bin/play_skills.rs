//! Special skills: stamina and the skill toggle (#120). Each squad character has a skill that
//! stamina pays for (Tex's Berserker, Brutus's Spirit of Vengar, Hawk's Stealth, Flint's
//! Advanced Targeting). This module is the part they share, the character's stamina update and
//! the game's skill base class, and the interface each skill fills in with its own effect
//! (`SkillKind`; #121-#124). Measured against the takes in todo/35-special-skills (spec Part 1);
//! the default.xbe functions are in viewer/docs/code-map.md ("Skills and stamina").
//!
//!  - **Stamina** (FUN_0011a840, from the character update, for every character with a skill,
//!    the player and the AI alike): while the skill is on it drains by the skill's drain (the
//!    base class's: `stamina-cost` x dt); at 0 the update sends the toggle command itself, every
//!    frame while the skill stays on. While it's off it refills by `stamina-recovery` x dt,
//!    x0.25 dashing (the demo's sprint), x2 kneeling still, x1 otherwise, from the frame it's
//!    off (no delay). Clamped to 0..max. The dead don't change.
//!  - **The toggle** (CT_TOGGLE_SPECIAL, 8): V, the game's White button, for the controlled
//!    character. The skill's own toggle answers the command: refused, switch now, or play its
//!    overlay from the motion set and switch at that clip's event (Tex 0.50 s, Hawk 0.83 s).
//!  - **On and off** (FUN_0010e470 / FUN_0010e3e0, shared): switching on needs 10% of the
//!    maximum stamina, and a refused press shows and plays nothing; each switch plays the type's
//!    start / end sound for the controlled character only (FUN_0010e7a0) and sends a chatter tag
//!    (on; off with stamina left; off at 0) through the game's chatter gate (`skill_line`: each
//!    block's probability, replay delay, condition, distance and priority).
//!  - **What ends a skill**: the toggle again, stamina at 0, death (the death handler calls the
//!    deactivate, 0x115df5), and each skill's own ends (its `update`). Handing control over
//!    doesn't: the skill stays on and drains on the character you left (take19).
//!  - **Tex's Berserker** (#121, `Berserker`): the press tops the other weapon slot's gun up from
//!    the reserve, takes the berserk motions (play.rs `SpecialClips`) and plays `Sc_w1_bz_start`;
//!    at its 0.50 s event that gun is drawn into his left hand (`Player::second`) and fires with
//!    the gun in hand, each from its own clip at its own rate (play_shots.rs). The drain halves
//!    while the gun in hand is empty; berserk ends when the second gun runs dry, and the switch key
//!    (Q, the game's Y) and the reload key (R, the game's X) end it instead of switching or
//!    reloading. While it's on no other overlay starts: no reload, switch, throw or item use
//!    (`overlays_blocked`).
//!  - **Flint's Advanced Targeting** (#124, `Targeting`): on and off at the press (with a rifle
//!    her motions become the sniper set, play.rs `SpecialClips`), while on the crosshair's lock box
//!    grows to 0.4 of the screen (play_reticle.rs), so the guns, which aim at the lock, find the
//!    enemies near the crosshair; her HUD marker shows the box or the locked target
//!    (play_targeting.rs) and the crosshair fades out. Her rolls are doubled 15% of the time while
//!    on and x1.5 scoped (play_damage.rs), her rifles reach x1.25, she walks, and the empty clip,
//!    the switch key and the reload key end it.
//!
//! Squad deathmatch maps (`sdm_*`) take the deathmatch squad's rates (play.rs `load_map`,
//! `Game::load_squad_library`): take21 drains Tex at 6.55/s on sdm_e34.
//!
//! Test hooks: `BF_TEST_SKILL=<s>[,<s>...]` presses the skill key at those times;
//! `BF_TEST_STAMINA=<n>[@<s>][,...]` sets the controlled character's stamina (at those times,
//! at the start without one); `BF_SKILL_CHATTER=<percent>` sets the skill lines' chance (100:
//! every time); `BF_TEST_DOUBLE=<percent>` sets the chance of Flint's doubled rolls (100: every
//! one); `BF_SKILL_LOG=1` logs each command, switch, sound and line, Tex's top-ups, refusals and
//! second gun, Flint's refusals, rifle mark and doubled rolls, and every character's stamina each
//! half second.

use super::*;
use bf_viewer::bf::character::{CombatSkills, SkillData, Tracking};

/// Squad deathmatch's Brute Force squad library (game-options-en.xml line 84, `<squad
/// h_fed0689a="mp_tex mp_brutus mp_hawk mp_flint" library-name="sdm_bf">`): the rates sdm maps
/// use (take21).
pub const SDM_SQUAD: &str = "sdm_bf";
/// Switching on needs this share of the maximum stamina (FUN_0010e470, 0x10e482-0x10e4a6: 0.1
/// at 0x3a4ff0, stamina >= it). take03: refused at 9.2, on at ~11.2; take11: refused at 9.06.
const ACTIVATE_SHARE: f32 = 0.1;
/// The refill's factors (FUN_0011a840, 0x11a95a-0x11a9b3): dashing, MT_DASH (0.25 at 0x3a5028;
/// take02 x0.250, take08 x0.248), and crouched still, MT_CROUCH (x2; take02 x2.006, take11
/// x2.000).
const DASH_REFILL: f32 = 0.25;
const CROUCH_REFILL: f32 = 2.0;
/// The chatter tags, sent through the chatter gate (FUN_00140a60, see `skill_line`): switched
/// on (FUN_0010e470, 0x10e4c7); off with stamina left (FUN_0010e3e0, 0x10e425); off at 0
/// (0x10e442). Heard in the takes on 13 of 35 ons (Tex, Hawk and Flint 13 of 26, Brutus 0 of 9),
/// 3 of 23 offs with stamina left (all Flint's), 2 of 9 offs at 0 (spec Part 1 section 4).
const LINE_ON: u32 = 0x1C05_14BD;
const LINE_OFF: u32 = 0x0908_7C9C;
const LINE_EMPTY: u32 = 0xEE0A_5CBC;
/// The condition kinds of the off lines (a block's h_ff361b85, FUN_00137360): the off line with
/// stamina left tests char+0x5a8, which only ever gets 0 in default.xbe (the constructor 0x11118a,
/// FUN_0010e470 at 0x10e4ed, FUN_00119560 at 0x1195ac: answer-120-chatter section 2), so its
/// variants needing 1 or 2 and up (Tex, Brutus, Hawk) are never said and Flint's (no range) are;
/// the line at 0 tests FUN_0001b460, a count of the speaker's brain's targets, read as the
/// enemies it's fighting (inferred).
const CONDITION_COUNTER: u32 = 0xE7FA_D8D2;
const CONDITION_FIGHTING: u32 = 0x12B7_4A88;
/// The gate's distances from the speaker to the nearest player-controlled character
/// (0x8d660): beyond LINE_FAR_STEP a line loses a step of priority (FUN_001409c0, 15.0 at
/// 0x3a500c; a priority-1 line, the on and off lines, is then refused), beyond LINE_FAR_CUT it
/// isn't said (FUN_001409a0, 45.0 at 0x3a5010). The player's own lines are at 0.
const LINE_FAR_STEP: f32 = 15.0;
const LINE_FAR_CUT: f32 = 45.0;
/// The skill sounds' volume (0x10e5b0 / 0x10e5e0 play them at 100, x0.01 at 0x3a4f98).
const SOUND_VOLUME: f32 = 1.0;
/// The animation events that switch a skill (FUN_00117660, the event handler): Tex's on
/// (eaba3d48, 0x118ae7) and off (14ee2201, 0x1180bb), both 0.50 s into `Sc_w1_bz_start` /
/// `bz_stop`; Hawk's either way (ead033a3, 0x118b8f), 0.83 s into `Sc_w1/w2_activate_stealth`.
const EV_BERSERK_ON: u32 = 0xEABA_3D48;
const EV_BERSERK_OFF: u32 = 0x14EE_2201;
const EV_STEALTH: u32 = 0xEAD0_33A3;
/// Tex's toggle timer (skill+0x20, 0x13d490): 1.0 s from an accepted press; presses are refused
/// until it runs out, the commands the stamina update sends at 0 included (take02: his stop clip
/// plays once, on at 0 for its half second).
const BERSERK_TIMER: f32 = 1.0;
/// The skills by `skill-set` (CSKILL_, default.xbe's name table 0x3bd814; the factory
/// FUN_0012e590 makes the skill object by it).
const CSKILL_BRUTUS: i64 = 1;
const CSKILL_FLINT: i64 = 2;
const CSKILL_TEX: i64 = 3;
const CSKILL_HAWK: i64 = 4;
/// The skills' names (each skill's vtable +0x4c: string ids in
/// common/stringtable/string-table-en.xml lines 1112-1115).
const NAME_BERSERKER: u32 = 0x0DEE_4365;
const NAME_VENGAR: u32 = 0x055B_624B;
const NAME_STEALTH: u32 = 0xEC52_79A7;
const NAME_TARGETING: u32 = 0x19E6_36BD;
/// Flint's +0x38 while on (0x13ceb0, 0.55 at 0x3a5208): the movement (FUN_001195c0) takes the
/// smaller of the stick's length and this, as it caps a scoped character's at 0.55 (0x11962e).
const TARGETING_MOVE_CAP: f32 = 0.55;
/// FUN_001195c0: a stick length under this moves at the walk (0.65, the compare after the caps);
/// the demo's walk key stands for it (`move_capped`).
const RUN_FROM: f32 = 0.65;
/// The lock box's scale Flint's +0x100 gives while on (0x13cd1d: 1.0 at 0x3a4f78), in place of
/// the gun's own h_e1e41c2d (the L-Shot-75's 0.1, the Jax-iP's 0.6).
const TARGETING_LOCK_BOX: f32 = 1.0;

/// What a skill's toggle makes of the command (the skill's vtable +0x04). In the game the toggle
/// switches the skill itself or not, and its return picks the motion (the command handler,
/// 0x1224e4): 0 none, 1 the type's overlay, 2 MT_TOGGLE_SPECIAL. For a 1 the handler looks the
/// overlay up by the skill's state after the toggle, OMT_SPECIAL_OFF (43) if it's on and
/// OMT_SPECIAL_ON (44) if not (0x122500-0x122564), and queues it only into a free overlay slot
/// (char+0x218, or +0x244 by the overlay record's +0x3d bit 2: 0x12253a-0x122587).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Toggle {
    /// nothing happens (the skill's own refusal)
    Refused,
    /// switch now; `overlay`: the toggle returns 1 after switching, so the handler's overlay
    /// plays too (none of the four does: Brutus, Flint and Hawk at 0 return 0)
    Now { overlay: bool },
    /// return 1 without switching: the overlay plays and the skill switches at the clip's event
    /// (the id), handled as FUN_00117660 does (Tex, Hawk)
    AtEvent(u32),
}

/// What a skill's own methods reach outside its character: the squad's ammo reserve (Tex's
/// clip top-up at the press, 0x13d643-0x13d692, and his second gun's load at the switch,
/// FUN_0022daf0), and whether to log (BF_SKILL_LOG).
pub struct Outside<'a> {
    pub reserve: &'a mut ammo::Reserve,
    pub log: bool,
}

/// A skill's own part, as the game's skill classes override their base class: the four below
/// (`Berserker`, `Vengar`, `Stealth`, `Targeting`), each of which its ticket fills in with its
/// effect. Each method's default is the base class's. The skill is taken out of its character
/// while these run, so they get the character itself (`u.skill.on`, `u.stamina`...).
pub trait SkillKind: Send + Sync {
    /// The skill's name, a string table id (vtable +0x4c): the HUD shows it while it's on.
    fn name(&self) -> u32;
    /// The toggle command (vtable +0x04), with the skill on or off (`u.skill.on`).
    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle;
    /// The stamina used over `dt` while on (vtable +0xbc); the base: `cost` x dt.
    fn drain(&self, _u: &Player, cost: f32, dt: f32) -> f32 {
        cost * dt
    }
    /// Switched on (vtable +0x120; for Tex, his switch event's own work, 0x118ae7): the skill's
    /// effects start.
    fn on(&mut self, _u: &mut Player, _out: &mut Outside) {}
    /// Switched off (vtable +0x124; for Tex, his off event's, 0x1180bb): its effects end.
    fn off(&mut self, _u: &mut Player) {}
    /// Every frame, on or off, while alive (vtable +0xc0). True sends the toggle command: the
    /// skill's own ends (Tex's other gun dry, Flint's clip dry: their tickets).
    fn update(&mut self, _u: &mut Player, _dt: f32) -> bool {
        false
    }
    /// Whether the switch key, the reload key and the black button end the skill while it's on,
    /// instead of what they do (vtable +0x7c, asked by the player controller at 0x134367,
    /// 0x134c50 and 0x134db4). The base says no (0x20e200); Tex's and Flint's yes (0x213690).
    fn ended_by_actions(&self) -> bool {
        false
    }
    /// Whether the character's other overlays may start while the skill is on (the overlay
    /// filter, vtable +0xa4, asked for each queued overlay by the item update FUN_0011a440 at
    /// 0x11a507, which drops a refused one). The base lets all through (0x262a80); Tex's
    /// (0x13d850) only his own off overlay (OMT_SPECIAL_OFF 43, which the toggle plays here).
    fn lets_overlays_play(&self) -> bool {
        true
    }
    /// Whether the character may start an action now: the last word of FUN_00117210, which ends
    /// by asking the skill's vtable +0x14. The game asks it before a reload starts (FUN_00120d40,
    /// 0x120e16), a weapon switch (FUN_0011e130, 0x11e135; CT_TOGGLE_WEAPON, 0x12259a), an item
    /// use other than IOU_JUST_USE (FUN_001210c0, 0x121139: a grenade thrown or placed, the
    /// medkit, IOU_USE_IN_HAND), a melee attack (CT_ATTACK, 0x1226d2) and Tex's and Hawk's
    /// toggles. Tex's, Brutus's and Hawk's say yes (0x213690); Flint's (0x13b980) only while her
    /// skill is off.
    fn lets_act(&self, _u: &Player) -> bool {
        true
    }
    /// The shooter's own factor on a damage roll (vtable +0xec: the roll FUN_0022c940 asks a
    /// character shooter's skill at 0x22c9a3, a blast's roll too). The base's 1.0 (0x10e3d0);
    /// Flint's (0x13cc70) doubles some while on and gives x1.5 scoped.
    fn roll_factor(&mut self, _u: &mut Player) -> f32 {
        1.0
    }
    /// A gun's range in the character's hands, `range` its bullet's (vtable +0x54, asked by
    /// FUN_002324c0 for the shot's ray, the falloff, the crosshair's centre ray and the lock's
    /// reach). The base leaves it (0x10e3a0); Flint's lengthens her rifles' (0x13ce80).
    fn range(&self, _u: &Player, _def: &WeaponDef, range: f32) -> f32 {
        range
    }
    /// The most of the move stick's length the character goes by, `m` the length (vtable +0x38,
    /// asked by the movement FUN_001195c0 at 0x119668, which keeps the smaller). The base passes
    /// it on (0x10e8f0); Flint's caps it while on.
    fn move_cap(&self, _u: &Player, m: f32) -> f32 {
        m
    }
    /// The crosshair's lock box scale, `scale` the gun's h_e1e41c2d (vtable +0x100, asked by
    /// FUN_000c5d20 at 0xc5d82 each frame, 0x11cfd5). The base passes it on (0x10e8f0); Flint's
    /// is 1.0 while on.
    fn lock_box(&self, _u: &Player, scale: f32) -> f32 {
        scale
    }
    /// The lock box's shares of the screen's width and height that the skill puts in the target
    /// selection (vtable +0xc4, which the selection runs for the controlled character's skill each
    /// frame, 0xbd0c6); None leaves the reticle's own (0.14 x 0.75 and 0.14, play_reticle.rs).
    /// The base's +0xc4 does nothing (0x2894f0); Flint's (0x13cd30) writes hers while on and the
    /// reticle's own back while off.
    fn lock_shares(&self, _u: &Player) -> Option<[f32; 2]> {
        None
    }
    /// The crosshair's alpha, `alpha` the reticle's 0.9 and `fill` the skill's marker's fill
    /// (vtable +0xfc, asked by FUN_000c6280 at 0xc62a7). The base passes it on (0x10e8f0);
    /// Flint's fades it out as her marker fills.
    fn crosshair_alpha(&self, _u: &Player, alpha: f32, _fill: f32) -> f32 {
        alpha
    }
    /// Whether the skill has the targeting marker, the HUD widget h_121cf7bb its +0xc4 makes for
    /// the controlled character (0x13cd74; play_targeting.rs): Flint's only.
    fn has_marker(&self) -> bool {
        false
    }
    /// Whether the skill keeps the gun raised now (FUN_00123ea0 every frame: Flint's update
    /// 0x13cbf0 while on), which the demo shows as the ready pose without the aiming camera.
    fn keeps_gun_up(&self, _u: &Player) -> bool {
        false
    }
    /// Whether the scope is barred now: the demo's choice for Tex's berserk motions (see
    /// `Berserker`), none of the game's.
    fn bars_scope(&self, _u: &Player) -> bool {
        false
    }
}

/// A character's special skill: what the game's skill base class holds (on, the toggle under
/// way) and the skill's own part. The stamina is the character's (`Player::stamina`), as the
/// game's is (char+0x490).
#[derive(Default)]
pub struct Skill {
    /// the type's skill data (rates, sounds); None: no skill, and the stamina stays put
    pub data: Option<SkillData>,
    /// the skill's own part (None while its methods run)
    pub kind: Option<Box<dyn SkillKind>>,
    /// switched on (skill+0x1c bit 0, read by FUN_00023c80)
    pub on: bool,
    /// the toggle's overlay under way, and the switch it carries
    pub toggling: Option<Toggling>,
    /// the toggle command, asked for (the key; a squad order, #33) and not yet carried out
    pub request: bool,
    /// the motion set's special motions taken (play.rs `SpecialClips`: Tex's berserk set). His
    /// toggle asks for them at an accepted press switching on, and for the normal ones back at
    /// one switching off (the motion request MT_TOGGLE_SPECIAL, FUN_0012eab0(type, 45, 1) at
    /// 0x13d6ca); so they change at the press, under the toggle's overlay, not at its event
    pub special_motions: bool,
    /// the character's chatter replay delays: tag -> seconds before it can say that tag again
    /// (the game's map at char+0x7f4: started when a line is said, FUN_00126460, counted down by
    /// FUN_0011c100 from the character update). Only the skill lines use it so far; it's here,
    /// not on `Player`, for that reason alone.
    pub replay: HashMap<u32, f32>,
    /// a roll was doubled since the targeting marker last looked (Flint's +0xec sets its marker's
    /// flag 4, 0x13ccd2; the marker's corners show it for a frame and its update clears it,
    /// 0xb49cd: play_targeting.rs)
    pub double_hit: bool,
}

impl Skill {
    /// The skill of a character type (the spawn, FUN_0012f440, has the factory FUN_0012e590 make
    /// it by skill-set), with game-options' `<combat-skills>` tuning (each constructor copies its
    /// block).
    pub fn new(data: Option<&SkillData>, settings: &CombatSkills) -> Self {
        let kind: Option<Box<dyn SkillKind>> = match data.map(|d| d.skill_set) {
            Some(CSKILL_TEX) => Some(Box::new(Berserker { timer: 0.0, empty_drain: settings.berserk_empty_drain })),
            Some(CSKILL_BRUTUS) => Some(Box::new(Vengar)),
            Some(CSKILL_HAWK) => Some(Box::new(Stealth)),
            Some(CSKILL_FLINT) => Some(Box::new(Targeting { rifle: false, cfg: settings.tracking })),
            _ => None,
        };
        Skill { data: data.cloned().filter(|_| kind.is_some()), kind, ..default() }
    }

    /// The skill's name, a string table id (None: no skill).
    pub fn name(&self) -> Option<u32> {
        self.kind.as_ref().map(|k| k.name())
    }
}

/// A toggle overlay under way: one of the character's `Loaded::skill_clips`.
#[derive(Clone, Copy, Debug)]
pub struct Toggling {
    /// the on overlay (OMT_SPECIAL_ON) or the off one
    pub on: bool,
    /// its stance (`Sc_w1_` / `Sc_w2_`), fixed at its start
    pub stance: usize,
    /// seconds into it, and its length
    pub time: f32,
    pub duration: f32,
    /// when it switches the skill (s into it: its event), until it has
    pub switch_at: Option<f32>,
}

impl Toggling {
    /// The overlay's place in `Loaded::skill_clips`: 0 on, 1 off.
    pub fn slot(&self) -> usize {
        if self.on { 0 } else { 1 }
    }
}

/// Whether `u` is free to start a toggle (FUN_00117210: no action in the slot char+0x214 or a
/// finished one, +0x225 clear, alive, no item held in +0x1fc / +0x200 of kind 0x2e or flagged
/// +0x252 bit 1, then the skill's +0x14, `lets_act`). In the demo: alive, not knocked down, no
/// grenade held (charging, `charge`), no reload, throw, medkit, weapon switch or toggle under way,
/// and the skill's word. Only Tex's toggle (switching on, 0x13d5c4) and Hawk's (0x13cfd2) ask it;
/// Brutus's (0x13bfe0) and Flint's (0x13cad0) don't.
pub fn free_to_act(u: &Player) -> bool {
    !u.dead && !knocked_down(u) && u.charge <= 0.0 && u.reloading.is_none() && u.throwing.is_none() && u.using.is_none()
        && u.switching.is_none() && u.skill.toggling.is_none() && lets_act(u)
}

/// Whether `u`'s skill lets `u` start an action now (`SkillKind::lets_act`, the skill's +0x14 that
/// FUN_00117210 ends with): not while Flint's Advanced Targeting is on. A reload, a weapon switch,
/// a grenade thrown or placed and the medkit wait for it (play_ammo.rs, play.rs,
/// play_pickups.rs).
pub fn lets_act(u: &Player) -> bool {
    u.skill.kind.as_ref().is_none_or(|k| k.lets_act(u))
}

/// Whether `u` has the stamina switching on needs (FUN_0010e560's check, the one FUN_0010e470
/// makes).
pub fn can_activate(u: &Player) -> bool {
    u.stamina >= ACTIVATE_SHARE * u.max_stamina
}

/// Whether `u`'s skill refuses its other overlays now (`SkillKind::lets_overlays_play`, while
/// it's on): in Tex's berserk no reload, weapon switch, grenade throw or item use starts (#35
/// take05: no reload; take26 20.5 s: LT throws nothing).
pub fn overlays_blocked(u: &Player) -> bool {
    u.skill.on && u.skill.kind.as_ref().is_some_and(|k| !k.lets_overlays_play())
}

/// Whether `u`'s skill is on and the switch key, the reload key and the black button end it
/// (`SkillKind::ended_by_actions`).
pub fn ended_by_actions(u: &Player) -> bool {
    u.skill.on && u.skill.kind.as_ref().is_some_and(|k| k.ended_by_actions())
}

/// The factor `u`'s skill puts on a damage roll `u` makes (FUN_0022c940 asks a character
/// shooter's skill, +0xec, after the roll of Damage min..max: play_damage.rs `Hit::roll`); 1
/// without a skill.
pub fn roll_factor(u: &mut Player) -> f32 {
    let Some(mut k) = u.skill.kind.take() else { return 1.0 };
    let f = k.roll_factor(u);
    u.skill.kind = Some(k);
    f
}

/// How far gun `def` reaches in `u`'s hands (FUN_002324c0: the bullet's range through the
/// holder's skill, +0x54): the shot's ray and its falloff (play.rs), the crosshair's centre ray
/// and the lock's reach (play_reticle.rs).
pub fn weapon_range(u: &Player, def: &WeaponDef) -> f32 {
    u.skill.kind.as_ref().map_or(def.range, |k| k.range(u, def, def.range))
}

/// Whether `u`'s skill holds the move stick under the run (FUN_001195c0 takes the smaller of the
/// stick and the skill's +0x38; under RUN_FROM the character walks): the demo's walk then, and
/// no sprint, as in the scope.
pub fn move_capped(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.move_cap(u, 1.0) < RUN_FROM)
}

/// The crosshair's lock box scale for `u`, `scale` the gun's (FUN_000c5d20 through the skill's
/// +0x100).
pub fn lock_box(u: &Player, scale: f32) -> f32 {
    u.skill.kind.as_ref().map_or(scale, |k| k.lock_box(u, scale))
}

/// The lock box's shares of the screen `u`'s skill puts in the target selection, if it does
/// (`SkillKind::lock_shares`).
pub fn lock_shares(u: &Player) -> Option<[f32; 2]> {
    u.skill.kind.as_ref().and_then(|k| k.lock_shares(u))
}

/// The crosshair's alpha for `u` (FUN_000c6280 through the skill's +0xfc), `fill` the
/// targeting marker's.
pub fn crosshair_alpha(u: &Player, alpha: f32, fill: f32) -> f32 {
    u.skill.kind.as_ref().map_or(alpha, |k| k.crosshair_alpha(u, alpha, fill))
}

/// Whether `u`'s skill has the targeting marker (Flint's; play_targeting.rs).
pub fn has_marker(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.has_marker())
}

/// Whether `u`'s skill keeps the gun raised now (`SkillKind::keeps_gun_up`): play.rs shows the
/// ready pose.
pub fn keeps_gun_up(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.keeps_gun_up(u))
}

/// Whether `u`'s skill bars the scope now (`SkillKind::bars_scope`, the demo's).
pub fn bars_scope(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.bars_scope(u))
}

/// The gun in `u`'s hand (FUN_0014d2e0, inventory +0x164): its data, while one is held (not
/// between a switch's drop and grab).
fn gun_in_hand(u: &Player) -> Option<&WeaponDef> {
    u.loaded.as_ref().and_then(|l| l.weapons.get(u.weapon)).filter(|_| u.holding).map(|w| &w.def)
}

/// The gun of `u`'s other weapon slot, the one berserk draws into the second hand: FUN_0014d090
/// takes slot 2's with slot 1 in hand and slot 1's with slot 2 (inventory +0x1cc), and the
/// toggle's top-up finds the same gun as the one a slot still holds (FUN_0014d430 slot 1, else
/// FUN_0014d450 slot 2: the gun in hand has left its slot). None with a single gun.
pub fn other_gun(u: &Player) -> Option<usize> {
    let n = u.loaded.as_ref()?.weapons.len();
    let w = if u.weapon == 0 { 1 } else { 0 };
    (u.weapon < 2 && w < n).then_some(w)
}

/// Top the clip of `u`'s weapon `w` up from the squad's reserve: min(clip size - clip, reserve)
/// rounds (Tex's toggle at the press, 0x13d643-0x13d692; take25: the Jax-iC 2 -> 36 and the
/// reserve 200 -> 166 at the press). A recharging gun has no reserve (FUN_0022da80 finds no item
/// of its ammo-type: the LZR-50 stays as it is). The rounds moved.
fn top_up(u: &mut Player, w: usize, reserve: &mut ammo::Reserve) -> i64 {
    let Some(def) = u.loaded.as_ref().and_then(|l| l.weapons.get(w)).map(|h| &h.def) else { return 0 };
    let (Some(&[clip, _]), false) = (u.ammo.get(w), def.ammo_regen > 0.0) else { return 0 };
    let need = (def.ammo.max(1) - clip).max(0);
    let took = if u.npc.is_some() { need } else { reserve.take(def.ammo_type, need) };
    u.ammo[w][0] += took;
    took
}

/// Whether `u`'s current motion has a MT_TOGGLE_SPECIAL transition, which Tex's toggle needs
/// either way (the motion record at char+0x1fc, its +0x534: the transition for motion type 45,
/// 0x13d4b8-0x13d4ce): in ms_tex.xml only the stand, walk, run, side and back steps and the
/// dash have one, and their berserk versions one back. So not crouched or getting there, in a
/// jump, a dodge, a dive or a slide, nor knocked down.
fn special_transition(u: &Player) -> bool {
    matches!(u.action, Action::None) && !u.slide_on && !knocked_down(u)
}

/// Tex's Berserker (CSKILL_TEX; skill vtable 0x39d058, constructor FUN_0013d440): he fires the
/// gun of his other weapon slot with the one in hand ("Uses two weapons at once.").
///
///  - **The toggle** (0x13d490): refused unless his motion has a berserk transition
///    (`special_transition`). Switching on is also refused with a gun already in the second hand,
///    crouched (MT 6/7), scoped (char+0x7a8), when he isn't free (FUN_00117210, 0x13d5c4), under
///    10% (0x13d5d3; take03 2.0 s: nothing at all), within the timer, without a second gun, or
///    when that gun's clip is still 0 after its top-up (take25 10.0 s); switching off with no gun
///    in the second hand or within the timer. Accepted, it tops the other gun up from the reserve
///    (switching on, `top_up`), asks for the berserk motions (or the normal ones back), starts its
///    1.0 s timer and returns 1, so `Sc_w1_bz_start` / `bz_stop` play and switch at their 0.50 s
///    events (measured 0.53 s after the toggle frame, on and off, takes 01-05, 21, 25, 26). The
///    timer refuses presses meanwhile, which keeps it on at 0 for that half second (take02).
///    "Out of Ammo" (h_ff471882, 0x13d6e2) is unreachable: the code refuses without a second gun
///    before it gets there.
///  - **On** (the event eaba3d48, 0x118ae7): the other slot's gun into his left hand
///    (FUN_0014d090), loaded from the reserve at once (FUN_0022daf0); the weapon box lists both.
///  - **Off** (14ee2201, 0x1180bb; and death): it goes back to its slot (FUN_0014d1a0).
///  - **Drain** (0x13d720): `cost` x dt, x h_1469a0df (0.5) while the gun in hand's clip is 0
///    (take05: -2.22/s, then -1.12/s).
///  - **Update** (FUN_0013d780): the timer runs down; once it's out, the second gun's clip at 0
///    sends the toggle (take26 30.23 s: the LZR-50 dry, berserk off 0.5 s later).
///  - The switch key, the reload key and the black button end it (vtable +0x7c), and no other
///    overlay plays while it's on (+0xa4, 0x13d850).
///  - The demo's choices until #35's shots T2 / T3 show the game's: no scope while his berserk
///    motions or second gun are in (play.rs `follow_camera`), and a crouch pressed in the motions
///    dropped, not kept for after the off (`step_player`).
struct Berserker {
    /// the toggle timer (skill+0x20, s left)
    timer: f32,
    /// the drain's factor while the gun in hand is empty (skill+0x24, `CombatSkills`)
    empty_drain: f32,
}

impl Berserker {
    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: berserk refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }
}

impl SkillKind for Berserker {
    fn name(&self) -> u32 {
        NAME_BERSERKER
    }

    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if !special_transition(u) {
            return Self::refused(u, out, "no berserk transition from his motion");
        }
        if u.skill.on {
            if u.second.is_none() {
                return Self::refused(u, out, "no gun in the second hand");
            }
            if self.timer > 0.0 {
                return Self::refused(u, out, "its timer runs");
            }
        } else {
            // (crouched is in the motion check already)
            let why = if u.second.is_some() { "a gun in the second hand already" }
                else if u.scope_level > 0 { "scoped" }
                else if !free_to_act(u) { "busy" }
                else if !can_activate(u) { "under 10%" }
                else if self.timer > 0.0 { "its timer runs" }
                else { "" };
            if !why.is_empty() {
                return Self::refused(u, out, why);
            }
            let Some(w) = other_gun(u) else { return Self::refused(u, out, "no second gun") };
            let took = top_up(u, w, out.reserve);
            let clip = u.ammo.get(w).map_or(0, |a| a[0]);
            if out.log {
                println!("t={:.2} {}: berserk top-up of weapon {w}: {took} rounds, clip {clip}", u.sim_time, unit_name(u));
            }
            if clip == 0 {
                return Self::refused(u, out, "the second gun has no rounds");
            }
        }
        u.skill.special_motions = !u.skill.on;
        self.timer = BERSERK_TIMER;
        Toggle::AtEvent(if u.skill.on { EV_BERSERK_OFF } else { EV_BERSERK_ON })
    }

    fn drain(&self, u: &Player, cost: f32, dt: f32) -> f32 {
        let empty = u.ammo.get(u.weapon).is_some_and(|a| a[0] == 0);
        cost * dt * if empty { self.empty_drain } else { 1.0 }
    }

    fn on(&mut self, u: &mut Player, out: &mut Outside) {
        let Some(w) = other_gun(u) else { return };
        u.second = Some(SecondGun::new(w));
        // (FUN_0022daf0 also clears its burst counter)
        let took = top_up(u, w, out.reserve);
        u.fire_state.reset_burst(1);
        u.weapon_dirty = true;
        // the weapon box names both a moment (take01: from the switch, 2.750 s, to 5.417 s)
        u.hud_list = HUD_LIST_TIME;
        if out.log {
            println!("t={:.2} {}: second gun drawn: weapon {w} (loaded {took} more, clip {})", u.sim_time, unit_name(u),
                     u.ammo.get(w).map_or(0, |a| a[0]));
        }
    }

    fn off(&mut self, u: &mut Player) {
        u.skill.special_motions = false;
        if u.second.take().is_some() {
            u.weapon_dirty = true;
            // the weapon box lists the weapons again, the stowed one in blue (take01 18.767 s)
            u.hud_list = HUD_LIST_TIME;
        }
    }

    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        // (the dry check waits for the timer: 0x13d783-0x13d7c2)
        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
            return false;
        }
        // (a toggle's overlay cut short by a knock-down, the demo's choice in `step`, leaves the
        // motions as the skill is)
        if u.skill.toggling.is_none() {
            u.skill.special_motions = u.skill.on;
        }
        u.skill.on && u.second.is_some_and(|s| u.ammo.get(s.weapon).is_some_and(|a| a[0] == 0))
    }

    fn ended_by_actions(&self) -> bool {
        true
    }

    fn lets_overlays_play(&self) -> bool {
        false
    }

    /// No scope from the accepted press (his berserk motions; the second gun, play.rs
    /// `follow_camera`, until it's stowed): the demo's choice until #35's shot T2.
    fn bars_scope(&self, u: &Player) -> bool {
        u.skill.special_motions
    }
}

/// Brutus's Spirit of Vengar (CSKILL_BRUTUS; skill vtable 0x39d678, toggle FUN_0013bfe0): on or
/// off at the press (takes 06-09, 22, 23, 27, 30: in the first log sample after it), with no
/// motion. The toggle activates or deactivates and returns 0 either way (0x13c00e), and the
/// command handler plays an overlay only for a 1 (0x1224e4): the handler at 0x122500-0x122587 is
/// the only lookup of the overlay types 43 / 44, so his motion set's `spirit` overlay
/// (`Sc_w1/w2_spirit`, OMT_SPECIAL_ON) never plays from the White button. It doesn't ask
/// FUN_00117210 either. The Vengar view, regeneration, damage taken and the charge are #122's.
struct Vengar;

impl SkillKind for Vengar {
    fn name(&self) -> u32 {
        NAME_VENGAR
    }

    fn toggle(&mut self, _u: &mut Player, _out: &mut Outside) -> Toggle {
        Toggle::Now { overlay: false }
    }
}

/// Hawk's Stealth (CSKILL_HAWK; skill vtable 0x39d190, toggle FUN_0013cf90). At stamina 0 or
/// dead it goes off at once, without a clip, returning 0 (take11 15.26-15.31); otherwise the
/// press is a request: refused unless she's free (FUN_00117210, 0x13cfd2; her clip under way
/// counts as a request pending) and, switching on, at 10% (take11 52.0 s: nothing at 9.06);
/// accepted, it returns 1 and her `Sc_w1/w2_activate_stealth` plays, on and off, and switches at
/// its 0.83 s event (measured 0.89-0.97 s after the press was sent, less the rig's input delay).
/// Her fade (and its mid-fade refusal), the firing and item costs and the AI's view of her are
/// #123's.
struct Stealth;

impl SkillKind for Stealth {
    fn name(&self) -> u32 {
        NAME_STEALTH
    }

    fn toggle(&mut self, u: &mut Player, _out: &mut Outside) -> Toggle {
        if u.skill.on && (u.stamina <= 0.0 || u.dead) {
            return Toggle::Now { overlay: false };
        }
        if !free_to_act(u) || (!u.skill.on && !can_activate(u)) {
            return Toggle::Refused;
        }
        Toggle::AtEvent(EV_STEALTH)
    }
}

/// Whether `u`'s current motion has a motion type 45 transition, which Flint's toggle needs
/// either way (the motion record at char+0x1fc, its +0x534, 0x13caf3-0x13cb09): in ms_flint.xml
/// her idles, walk, run, side and back steps, dash, crouch idle and crouch walks have one, and the
/// sniper motions one back; the stand-to-crouch and back (MT 46), the jumps, dodges, dive, slide
/// and the throw have none. So not getting down or up, in a jump, a dodge, a dive or a slide, nor
/// knocked down.
fn sniper_transition(u: &Player) -> bool {
    matches!(u.action, Action::None | Action::Crouched) && !u.slide_on && !knocked_down(u)
}

/// Flint's Advanced Targeting (CSKILL_FLINT; skill vtable 0x39d2c8, constructor FUN_0013c9e0,
/// the tuning `Tracking`, game-options' block h_0725cdb1): "Automatically targets enemies."
///
///  - **The toggle** (FUN_0013cad0), on or off at the press (takes 15-17, 29, 31: in the first
///    log sample after it), no overlay: refused unless her motion has a motion type 45
///    transition (`sniper_transition`). Off: the deactivate. On: refused with a gun in the second
///    hand, without a gun in hand, or with its clip empty (0x13cb70-0x13cbaf); then the activate
///    (10%), and `on`. Its own other refusals (char+0x748 bit 3, a motion state of 0, and the
///    motion record's +0x250 / +0x252 flags with both action slots busy, 0x13cad8-0x13cb46) aren't
///    modelled: what sets them isn't traced.
///  - **On**: with a gun of the tracking class in hand (`Tracking::weapon_type`, the rifle) the
///    toggle marks skill+0x48 and returns 2, motion type 45 (0x13cbbc-0x13cbd5): her motions
///    become the sniper set (play.rs `SpecialClips`: `Sc_w1_sniper_idle` and the `Sc_w1_sn_*`
///    walks; take15 2.5 s, the L-Shot raised to the shoulder). The motion state stays 19, an idle,
///    which is why the takes' logs never showed 45. Otherwise it raises the gun (FUN_00123ea0) and
///    returns 0.
///  - **Off**: the toggle returns 2 again when the mark is set (0x13cb64-0x13cb6c): her own
///    motions back.
///  - **Update** (0x13cbf0), while on: the gun kept raised (FUN_00123ea0 every frame), and the
///    gun in hand's clip at 0 sends the toggle (take29: off in the tick the clip read 0, before
///    the reload).
///  - **The crosshair's lock** (+0x100 and +0xc4, play_reticle.rs): while on the lock box is 0.4
///    of the screen each way at scale 1.0, in place of 0.14 x 0.75 / 0.14 x the gun's
///    h_e1e41c2d, so an enemy up to 128 units across and 96 down from the crosshair is locked, and
///    the guns, which aim at the lock (play.rs), hit it (take28: 1 hit in 8 off, 7 in 8 on).
///  - **Damage** (+0xec, 0x13cc70, in the roll FUN_0022c940): x1.5 scoped (char+0x7a8), on or off;
///    while on, a roll of the game's random numbers at most 0.15 doubles it and sets the marker's
///    flag 4 (`Skill::double_hit`). The two stack. take28: one hit of seven doubled (80.6 against
///    42.7), the scoped ones x1.36-1.59.
///  - **Range** (+0x54, 0x13ce80): her rifles' x1.25, on or off.
///  - **Walk** (+0x38, 0x13ceb0): 0.55 of the stick while on, as scoped: a walk.
///  - **The crosshair** (+0xfc, 0x13cce0): its 0.9 alpha x (1 - the marker's fill) while on, so
///    it fades out as the marker fills (take15 2.2-2.8 s) and is back at the off.
///  - The switch key, the reload key and the black button end it (+0x7c, 0x213690); her overlays
///    play (+0xa4, the base's).
///  - **Not free to act** (+0x14, 0x13b980: "off"): while on, FUN_00117210 says no, so no reload,
///    weapon switch, grenade throw or placement, or medkit starts (`lets_act`); the keys that end
///    it come first, and a reload asked for follows the off.
///  - Not used: +0xf8, the tracking-reticule-speed 2 (0x13c9b0), how fast the crosshair moves to
///    a lock (FUN_000c5e20), hidden while she tracks; +0x58, 0.8 in place of the base's 0.2 for
///    the scoped lock box (0x11cfb5): the demo's scope keeps the gun's box (play_reticle.rs).
struct Targeting {
    /// skill+0x48 bit 0: switched on with the rifle in hand (its toggle asked for motion type 45
    /// then, and asks for it again switching off)
    rifle: bool,
    /// game-options' tuning, copied at the constructor
    cfg: Tracking,
}

impl Targeting {
    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: targeting refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }
}

impl SkillKind for Targeting {
    fn name(&self) -> u32 {
        NAME_TARGETING
    }

    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if !sniper_transition(u) {
            return Self::refused(u, out, "no motion type 45 transition from her motion");
        }
        if !u.skill.on {
            let why = if u.second.is_some() { "a gun in the second hand" }
                else if gun_in_hand(u).is_none() { "no gun in hand" }
                else if u.ammo.get(u.weapon).is_none_or(|a| a[0] <= 0) { "the clip in hand is empty" }
                else { "" };
            if !why.is_empty() {
                return Self::refused(u, out, why);
            }
        }
        // (the activate's 10% check and the mark come after: `command`, `on`)
        Toggle::Now { overlay: false }
    }

    fn on(&mut self, u: &mut Player, out: &mut Outside) {
        // 0x13cbbc: the gun in hand's class (its data's first word, the weapon-type)
        self.rifle = gun_in_hand(u).is_some_and(|d| d.weapon_type == self.cfg.weapon_type);
        if self.rifle {
            // motion type 45: the sniper set
            u.skill.special_motions = true;
        } else {
            // FUN_00123ea0
            u.fire_state.keep_raised();
        }
        if out.log {
            println!("t={:.2} {}: targeting on, {}", u.sim_time, unit_name(u),
                     if self.rifle { "the rifle marked: the sniper motions" } else { "no rifle: the gun raised" });
        }
    }

    fn off(&mut self, u: &mut Player) {
        // (with the mark, motion type 45 back to her own motions; without, she was never in the
        // sniper set)
        u.skill.special_motions = false;
    }

    fn update(&mut self, u: &mut Player, _dt: f32) -> bool {
        if !u.skill.on {
            return false;
        }
        u.fire_state.keep_raised();
        // (only a gun in hand: none between a switch's drop and grab)
        gun_in_hand(u).is_some() && u.ammo.get(u.weapon).is_some_and(|a| a[0] == 0)
    }

    fn ended_by_actions(&self) -> bool {
        true
    }

    fn lets_act(&self, u: &Player) -> bool {
        !u.skill.on
    }

    fn roll_factor(&mut self, u: &mut Player) -> f32 {
        // (char+0x7a8, the scope: read as the scope's view being in, as play.rs's fire rate and
        // play_reticle.rs read it)
        let mut f = if u.scope > 0.5 { self.cfg.scope_factor } else { 1.0 };
        if u.skill.on {
            // the game's random numbers (the LCG at 0x46cfd8, a float in [0, 1) from its top 23
            // bits) at most the chance; BF_TEST_DOUBLE=<percent> sets the chance
            let chance = std::env::var("BF_TEST_DOUBLE").ok().and_then(|v| v.trim().parse::<f32>().ok()).map_or(self.cfg.double_chance, |p| p / 100.0);
            let r = u.random(1 << 23) as f32 / (1u32 << 23) as f32;
            if r <= chance {
                f *= self.cfg.double_factor;
                u.skill.double_hit = true;
                if std::env::var("BF_SKILL_LOG").is_ok() {
                    println!("t={:.2} {}: the roll doubled (x{})", u.sim_time, unit_name(u), self.cfg.double_factor);
                }
            }
        }
        f
    }

    fn range(&self, _u: &Player, def: &WeaponDef, range: f32) -> f32 {
        if def.weapon_type == self.cfg.range_weapon_type { range * self.cfg.range_factor } else { range }
    }

    fn move_cap(&self, u: &Player, m: f32) -> f32 {
        if u.skill.on { TARGETING_MOVE_CAP } else { m }
    }

    fn lock_box(&self, u: &Player, scale: f32) -> f32 {
        if u.skill.on { TARGETING_LOCK_BOX } else { scale }
    }

    fn lock_shares(&self, u: &Player) -> Option<[f32; 2]> {
        u.skill.on.then_some(self.cfg.box_shares)
    }

    fn crosshair_alpha(&self, u: &Player, alpha: f32, fill: f32) -> f32 {
        if u.skill.on { alpha * (1.0 - fill) } else { alpha }
    }

    fn has_marker(&self) -> bool {
        true
    }

    fn keeps_gun_up(&self, u: &Player) -> bool {
        u.skill.on
    }
}

/// The test hooks' progress on this map: the last BF_TEST_SKILL press and BF_TEST_STAMINA poke
/// carried out (their times, s).
#[derive(Resource)]
struct SkillHooks {
    last_press: f32,
    last_poke: f32,
}

impl Default for SkillHooks {
    fn default() -> Self {
        SkillHooks { last_press: -1.0, last_poke: -1.0 }
    }
}

/// The skill lines' variants already said, by character (chatter set) and tag: the game's
/// manager keeps them per chatter set and tag (mgr+0x28, FUN_001411e0 / FUN_001412c0) so a
/// character goes through all of a tag's variants before one repeats (FUN_001371f0, which clears
/// them, FUN_00141580, once every matching one is said). One per map, as the manager is per world.
#[derive(Resource, Default)]
struct LineHistory(HashMap<(usize, u32), Vec<u32>>);

/// The skill key and test hooks before the movement, every character's skill after it (so the
/// refill sees this frame's motion, and its sounds play this frame). The key comes after the
/// hand-over (`squad_control`), so a press in the frame control passes goes to the new character,
/// not the one handed over from; and after the ammo test hooks (`ammo::test_trigger`), whose
/// switch and reload presses it may turn into the toggle.
pub fn plugin(app: &mut App) {
    app.init_resource::<SkillHooks>()
        .init_resource::<LineHistory>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| {
            commands.insert_resource(SkillHooks::default());
            commands.insert_resource(LineHistory::default());
        })
        .add_systems(Update, (
            skill_input.after(read_input).after(squad_control).after(ammo::test_trigger).before(update_player),
            update_skills.after(update_player).before(play_sounds),
        ).run_if(in_state(AppState::Playing)));
}

/// The skill key: V, the game's White button (VB_SPECIAL_ABILITY, game-options-en.xml line 151;
/// the player controller sends CT_TOGGLE_SPECIAL at 0x134cff), for the controlled character,
/// alive; and the test hooks BF_TEST_SKILL (presses) and BF_TEST_STAMINA (pokes).
///
/// With a skill on that they end (`ended_by_actions`: Tex's), the player controller sends the
/// toggle instead of what two more keys do: the switch key, Q (the game's Y, VB 10, 0x134db4-
/// 0x134dde: the toggle in place of CT 9; take26 4.5 s, no switch, berserk off), and the reload
/// key, R (the game's X let go, 0x134c2b-0x134c50), when a reload could start (FUN_00120c50: the
/// gun in hand's clip below its size and a reserve for it), which also leaves the reload asked
/// for (+0xa1 bit 2), so it follows once the skill is off (take26 14.0 s: the reload at 15.19 s);
/// with no reload possible the reload key does nothing. R does it on the press, where the game's
/// X does it on the release (take26: off about 0.6 s after X came up): the demo's choice, as its
/// R reloads on the press. The black button (VB 63, 0x134367, the medkit shortcut) ends it the
/// same way in the game; the demo has no key for it.
fn skill_input(keys: Res<ButtonInput<KeyCode>>, mut player: ResMut<Player>, mut hooks: ResMut<SkillHooks>, reserve: Res<ammo::Reserve>) {
    if player.loaded.is_none() {
        return;
    }
    let t = player.sim_time;
    let mut press = keys.just_pressed(KeyCode::KeyV);
    // (once per time given, whatever the frame rate: the time it last pressed is kept)
    if let Ok(v) = std::env::var("BF_TEST_SKILL") {
        let due = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).filter(|&s| t >= s && hooks.last_press < s)
            .fold(None, |a: Option<f32>, s| Some(a.map_or(s, |a| a.max(s))));
        if let Some(s) = due {
            hooks.last_press = s;
            press = true;
        }
    }
    if ended_by_actions(&player) && !player.dead {
        let log = std::env::var("BF_SKILL_LOG").is_ok();
        if std::mem::take(&mut player.switch_pressed) {
            press = true;
            if log {
                println!("t={t:.2} {}: the switch key ends the skill", unit_name(&player));
            }
        }
        if player.reload_pressed {
            // FUN_00120c50, can reload: the gun in hand's clip below its size and its reserve not
            // empty (a recharging gun has none)
            let can = player.loaded.as_ref().and_then(|l| l.weapons.get(player.weapon)).zip(player.ammo.get(player.weapon))
                .is_some_and(|(w, a)| a[0] < w.def.ammo.max(1) && w.def.ammo_regen <= 0.0 && reserve.get(w.def.ammo_type) > 0);
            press |= can;
            // (with no reload possible the key does nothing: FUN_00120d40 refuses in berserk)
            player.reload_pressed = can;
            if log {
                println!("t={t:.2} {}: the reload key {}", unit_name(&player), if can { "ends the skill, the reload follows" } else { "does nothing" });
            }
        }
    }
    if press && !player.dead {
        player.skill.request = true;
    }
    if let Ok(v) = std::env::var("BF_TEST_STAMINA") {
        let mut pokes: Vec<(f32, f32)> = v.split(',').filter_map(|x| {
            let (n, at) = x.split_once('@').unwrap_or((x, "0"));
            Some((at.trim().parse::<f32>().ok()?, n.trim().parse::<f32>().ok()?))
        }).filter(|&(at, _)| t >= at && hooks.last_poke < at).collect();
        pokes.sort_by(|a, b| a.0.total_cmp(&b.0));
        if let Some(&(at, n)) = pokes.last() {
            hooks.last_poke = at;
            player.stamina = n.clamp(0.0, player.max_stamina);
            if std::env::var("BF_SKILL_LOG").is_ok() {
                println!("t={t:.2} {}: stamina set to {:.2}", unit_name(&player), player.stamina);
            }
        }
    }
}

/// What a character's skill step needs from outside the character.
struct Env<'a> {
    game: &'a Game,
    /// it's the one the player controls (its skill sounds play)
    controlled: bool,
    log: bool,
    /// for its chatter lines: its place in `sight` (0 the player, then the squad's order), who can
    /// see whom (`sight_list`), its distance to the player's character, and the variants said
    me: usize,
    sight: &'a [(u8, Vec3, bool)],
    distance: f32,
    history: &'a mut LineHistory,
    /// the squad's ammo (for the skill's own methods, `Outside`)
    reserve: &'a mut ammo::Reserve,
}

impl Env<'_> {
    /// What the skill's own methods get of it.
    fn outside(&mut self) -> Outside<'_> {
        Outside { reserve: &mut *self.reserve, log: self.log }
    }
}

/// Every character's skill and stamina, after the movement (so the refill sees this frame's
/// motion): the controlled one first, then the squad (the AI's, as the game runs it for
/// everyone). Logs each half second with BF_SKILL_LOG.
#[allow(clippy::too_many_arguments)]
fn update_skills(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut history: ResMut<LineHistory>,
                 mut reserve: ResMut<ammo::Reserve>, mut last_log: Local<f32>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    // (the order the AI's sight uses: the player, then the squad, NPCs included)
    let sight = sight_list(std::iter::once(&*player).chain(squad.0.iter()));
    let at = player.position;
    let mut env = Env { game: &game.0, controlled: true, log, me: 0, sight: &sight, distance: 0.0, history: &mut *history, reserve: &mut *reserve };
    step(&mut player, &mut env, dt);
    for (k, m) in squad.0.iter_mut().enumerate().filter(|(_, m)| m.in_squad()) {
        let mut env = Env { game: &game.0, controlled: false, log, me: k + 1, sight: &sight, distance: m.position.distance(at), history: &mut *history,
                            reserve: &mut *reserve };
        step(m, &mut env, dt);
    }
    // the stamina of each, every half second
    let t = player.sim_time;
    if log && (t * 2.0).floor() > (*last_log * 2.0).floor() {
        let list: Vec<String> = std::iter::once(&*player).chain(squad.0.iter().filter(|m| m.in_squad())).map(|u| {
            let state = if u.skill.on { "on".to_string() } else { format!("off x{}", refill_factor(u)) };
            format!("{} {:.2} {state}", unit_name(u), u.stamina)
        }).collect();
        println!("t={t:.2} stamina: {}", list.join(" | "));
    }
    *last_log = t;
}

/// The refill's factor for what `u` is doing (FUN_0011a840, by the motion state char+0x1f8):
/// dashing, MT_DASH (the demo's sprint, the player's and the AI's catching up) x0.25; crouched
/// still, MT_CROUCH (kneeling, not the crouch walk) x2; anything else x1, the kneel's way down
/// and up (MT_TRANSITION, measured x1: take02, take11), walking and knocked down (MT_KNOCKDOWN;
/// the knock-down leaves the gait as it was, a sprint included) included.
fn refill_factor(u: &Player) -> f32 {
    if knocked_down(u) {
        return 1.0;
    }
    match (u.action, u.gait) {
        (Action::None, Gait::Sprint) => DASH_REFILL,
        (Action::Crouched, Gait::Idle) => CROUCH_REFILL,
        _ => 1.0,
    }
}

/// One character's skill this frame: its chatter replay delays, the death handler's off, the
/// overlay's switch, the toggle command asked for, then the stamina update as FUN_0011a840 runs
/// it.
fn step(u: &mut Player, env: &mut Env, dt: f32) {
    // the replay delays run down (FUN_0011c100, from the character update)
    u.skill.replay.retain(|_, left| {
        *left -= dt;
        *left > 0.0
    });
    let Some(mut kind) = u.skill.kind.take() else { return };
    let k = kind.as_mut();
    if u.dead {
        // the death handler switches it off (0x115df5 -> FUN_0010e3e0: its end sound; its line
        // is silent, the speaker being dead), take30 off in the frame of death; the dead don't
        // change (take30: 94.11 for 9.4 s)
        u.skill.toggling = None;
        u.skill.request = false;
        if u.skill.on {
            deactivate(u, k, env);
        }
        // (the motions the toggle asked for go with the body)
        u.skill.special_motions = false;
        u.skill.kind = Some(kind);
        return;
    }
    // knocked down, the toggle's overlay is cut short, and the switch it carried with it: the
    // demo's choice (the knock-down's ragdoll replaces the pose; what the game does with an
    // overlay at a knock-down isn't traced)
    if knocked_down(u) {
        u.skill.toggling = None;
    }
    // the overlay under way since an earlier frame: the skill switches at its event
    // (FUN_00117660), the clip plays on. (One started this frame starts moving next frame, so the
    // switch comes 0.50 s after the press frame at the demo's rates; the takes: 0.53 s, the event
    // and about one 30 Hz frame.)
    if let Some(mut t) = u.skill.toggling.take() {
        t.time += dt;
        let due = t.switch_at.is_some_and(|at| t.time >= at);
        if due {
            t.switch_at = None;
        }
        if t.time < t.duration {
            u.skill.toggling = Some(t);
        }
        if due && t.on != u.skill.on {
            if t.on {
                activate(u, k, env);
            } else {
                deactivate(u, k, env);
            }
        }
    }
    if std::mem::take(&mut u.skill.request) {
        command(u, k, env, "key");
    }
    // the stamina update (FUN_0011a840), for types with h_e78efbb2
    let (cost, recovery) = match &u.skill.data {
        Some(d) if d.stamina_update => (d.cost, d.recovery),
        _ => {
            u.skill.kind = Some(kind);
            return;
        }
    };
    if u.skill.on {
        // FUN_001258d0 clamps every write to 0..max; at 0 (0x11a8f5-0x11a900) the command, every
        // frame while it stays on (0x11a914)
        let used = k.drain(u, cost, dt);
        u.stamina = (u.stamina - used).clamp(0.0, u.max_stamina);
        if u.stamina <= 0.0 {
            command(u, k, env, "at 0");
        }
    }
    // the skill's update, on or off (vtable +0xc0): its own ends
    if k.update(u, dt) {
        command(u, k, env, "its own end");
    }
    // off (checked again: a skill that went off this frame refills this frame, take02)
    if !u.skill.on {
        u.stamina = (u.stamina + recovery * dt * refill_factor(u)).min(u.max_stamina);
    }
    u.skill.kind = Some(kind);
}

/// The toggle command (CT_TOGGLE_SPECIAL: the key's, the stamina update's at 0, a skill's own
/// end): the character's command handler (0x1224a5) calls the skill's toggle, and its answer is
/// carried out. (`why` is for the log.)
fn command(u: &mut Player, k: &mut dyn SkillKind, env: &mut Env, why: &str) {
    let was = u.skill.on;
    // (the one at 0 comes every frame: its refusals, the skill's own reasons included, go
    // unlogged)
    let mut out = env.outside();
    out.log &= why != "at 0";
    let answer = k.toggle(u, &mut out);
    if env.log && (why != "at 0" || answer != Toggle::Refused) {
        println!("t={:.2} {}: toggle ({why}) at stamina {:.2}, {}: {answer:?}", u.sim_time, unit_name(u),
                 u.stamina, if was { "on" } else { "off" });
    }
    // (the overlay, for a toggle that returns 1, is picked by the skill's state after the toggle:
    // the off one if it's on, 0x122500-0x122564)
    match answer {
        Toggle::Refused => {}
        Toggle::Now { overlay } => {
            let switched = if was {
                deactivate(u, k, env);
                true
            } else {
                activate(u, k, env)
            };
            if switched && overlay {
                start_overlay(u, !u.skill.on, None, env.log);
            }
        }
        Toggle::AtEvent(event) => {
            // (without the clip or its event: at once)
            if !start_overlay(u, !u.skill.on, Some(event), env.log) {
                if was {
                    deactivate(u, k, env);
                } else {
                    activate(u, k, env);
                }
            }
        }
    }
}

/// Start a toggle's overlay, the on one (OMT_SPECIAL_ON) or the off one, for the held weapon's
/// stance (the motion set's, `Loaded::skill_clips`), the skill to switch at its `event`. False
/// when there's no such clip, or no such event in it: then nothing starts. (The game queues it
/// only into a free overlay slot, char+0x218 or +0x244, 0x12253a-0x122587; the toggles that use
/// it, Tex's and Hawk's, are refused anyway unless their character is free.)
fn start_overlay(u: &mut Player, on: bool, event: Option<u32>, log: bool) -> bool {
    let Some(l) = u.loaded.as_ref() else { return false };
    let stance = stance_of(l, u.weapon);
    let Some(clip) = l.skill_clips[if on { 0 } else { 1 }][stance].as_ref() else { return false };
    let switch_at = match event {
        Some(e) => match clip.event(e) {
            Some(at) => Some(at),
            None => return false,
        },
        None => None,
    };
    if log {
        println!("t={:.2} {}: {} overlay, {:.2} s{}", u.sim_time, unit_name(u), if on { "on" } else { "off" },
                 clip.duration, switch_at.map_or(String::new(), |at| format!(", switches at {at:.2} s")));
    }
    u.skill.toggling = Some(Toggling { on, stance, time: 0.0, duration: clip.duration, switch_at });
    true
}

/// Switch `u`'s skill on (FUN_0010e470): it needs 10% stamina, else nothing happens (a refused
/// press shows and plays nothing: take03, take11); then the skill's own on, the type's
/// start-skill sound and the on line. Whether it switched.
fn activate(u: &mut Player, k: &mut dyn SkillKind, env: &mut Env) -> bool {
    if !can_activate(u) {
        if env.log {
            println!("t={:.2} {}: not switched on at stamina {:.2} (under 10%)", u.sim_time, unit_name(u), u.stamina);
        }
        return false;
    }
    u.skill.on = true;
    if let Some(t) = u.skill.toggling.as_mut() {
        t.switch_at = None;
    }
    k.on(u, &mut env.outside());
    let sound = u.skill.data.as_ref().map_or(0, |d| d.start_sound);
    switched(u, env, sound, LINE_ON);
    true
}

/// Switch `u`'s skill off (FUN_0010e3e0): the skill's own off, the type's end-skill sound and a
/// line, the off line with stamina left (0x10e425) or the one at 0 (0x10e442; the stamina tested
/// at 0x10e40d).
fn deactivate(u: &mut Player, k: &mut dyn SkillKind, env: &mut Env) {
    u.skill.on = false;
    // (an overlay still to switch it has nothing left to do: Hawk at 0 mid-clip goes off at once)
    if let Some(t) = u.skill.toggling.as_mut() {
        t.switch_at = None;
    }
    k.off(u);
    let sound = u.skill.data.as_ref().map_or(0, |d| d.end_sound);
    let line = if u.stamina > 0.0 { LINE_OFF } else { LINE_EMPTY };
    switched(u, env, sound, line);
}

/// A switch's sound and line. The sound plays in 2D at full volume for the controlled
/// character only (FUN_0010e7a0 asks FUN_00112790; take18: an AI Hawk's switches are silent);
/// it isn't cut by the next switch (take08). The line goes through the chatter gate
/// (`skill_line`).
fn switched(u: &mut Player, env: &mut Env, sound: u32, line: u32) {
    if env.controlled && sound != 0 {
        u.sound_queue.push((sound, SOUND_VOLUME));
    }
    let said = skill_line(u, env, line);
    if env.log {
        let what = if u.skill.on { "on" } else { "off" };
        println!("t={:.2} {}: skill {what} at stamina {:.2}; sound {}; line {line:08x}: {said}", u.sim_time,
                 unit_name(u), u.stamina,
                 if env.controlled && sound != 0 { format!("{sound:08x}") } else { "none".into() });
    }
}

/// The game's chatter gate (FUN_00140a60, todo/35-special-skills/answer-120-chatter.md section
/// 1.3) for a skill line `tag` from `u`, with its block's values (`ChatterBlock`); what came of
/// it, for the log. In the code's order:
///  * the speaker's replay delay for the tag running (FUN_001263e0): refused;
///  * no block for the tag: refused;
///  * beyond 45 m of the player's character (FUN_001409a0, the block's h_19be8951): refused;
///  * the condition: its value (FUN_00137360, by the block's kind) must lie in a variant's range
///    (FUN_001372c0). The skill lines' kinds: none (0); char+0x5a8 (CONDITION_COUNTER: always 0,
///    so Tex's, Brutus's and Hawk's off lines are never said); the enemies it's fighting
///    (CONDITION_FIGHTING, FUN_0001b460), for which the demo counts 1 while a hostile is in the
///    speaker's sight (`hostile_in_sight`), **a guess** (only Hawk's variants need it, 1..100);
///  * the roll, `probability` % (FUN_001374c0; 50 for the skill lines). BF_SKILL_CHATTER=<percent>
///    sets it;
///  * the priority, a step less beyond 15 m (FUN_001409c0, h_f1cb4e63); 0 is refused (a
///    squadmate's on and off lines, priority 1);
///  * the game's one shared queue (a playing line of equal or lower priority delays it up to the
///    block's 5 s, a waiting line of equal or higher priority refuses it: FUN_00141020,
///    FUN_00140a40) isn't modelled: the demo's stand-in is the speaker's own `say()` check, refused
///    while that character is talking or has a line due;
///  * a dead speaker stays silent (FUN_0013fab0).
///
/// Said, a variant whose range holds the value is picked at random among those not said yet
/// (`LineHistory`), the speaker's replay delay for the tag starts (`replay-delay`, 10 / 20 / 30 s:
/// FUN_00140cd0 -> FUN_00126460), and it plays the next frame (the manager's update starts it;
/// the samples have no silence before the voice, 0-0.07 s).
fn skill_line(u: &mut Player, env: &mut Env, tag: u32) -> String {
    let (game, name) = (env.game, CHARACTERS[u.character % CHARACTERS.len()]);
    if u.dead {
        return "silent (the speaker is dead)".into();
    }
    if let Some(left) = u.skill.replay.get(&tag) {
        return format!("refused (its replay delay, {left:.1} s left)");
    }
    let Some(block) = game.chatter_blocks.get(name).and_then(|b| b.get(&tag)) else { return "refused (no block)".into() };
    if block.far_cut && env.distance > LINE_FAR_CUT {
        return format!("refused ({:.0} m from the player)", env.distance);
    }
    let value = match block.condition {
        CONDITION_FIGHTING => hostile_in_sight(u, env.me, env.sight).is_some() as i64,
        // char+0x5a8, never counted up in the game
        CONDITION_COUNTER => 0,
        // the default kind's value (and any other kind's: none of the skill lines has one)
        _ => 0,
    };
    let fits: Vec<u32> = block.variants.iter().filter(|&&(id, lo, hi)| (lo..=hi).contains(&value) && game.sounds.has(id))
        .map(|&(id, _, _)| id).collect();
    if fits.is_empty() {
        return format!("refused (no variant for the condition value {value}, kind {:08x})", block.condition);
    }
    let probability = std::env::var("BF_SKILL_CHATTER").ok().and_then(|v| v.parse::<i64>().ok()).unwrap_or(block.probability);
    if probability < 100 && u.random(100) as i64 + 1 > probability {
        return format!("not rolled ({probability}%)");
    }
    let priority = block.priority - (block.far_step && env.distance > LINE_FAR_STEP) as i64;
    if priority <= 0 {
        return format!("refused (priority 0, {:.0} m from the player)", env.distance);
    }
    // (the stand-in for the game's queue)
    if u.speaking > 0.0 || u.quote_in.is_some() {
        return "refused (the speaker is talking)".into();
    }
    let said = env.history.0.entry((u.character, tag)).or_default();
    let mut fresh: Vec<u32> = fits.iter().copied().filter(|id| !said.contains(id)).collect();
    if fresh.is_empty() {
        said.clear();
        fresh = fits;
    }
    let id = fresh[u.random(fresh.len())];
    said.push(id);
    if block.replay_delay > 0.0 {
        u.skill.replay.insert(tag, block.replay_delay);
    }
    u.quote_in = Some((0.0, id));
    format!("said {id:08x} ({probability}%, replay delay {:.0} s)", block.replay_delay)
}
