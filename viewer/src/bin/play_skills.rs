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
//!    character; for an AI member the Squad Command menu's Toggle Ability, whose goal sends the
//!    same command every frame until the skill has switched (GOAL_TOGGLE_COMBAT_SKILL,
//!    play_orders.rs `toggle_skill`, #333). The skill's own toggle answers the command: refused,
//!    switch now, or play its overlay from the motion set and switch at that clip's event (Tex
//!    0.50 s, Hawk 0.83 s).
//!  - **On and off** (FUN_0010e470 / FUN_0010e3e0, shared): switching on needs 10% of the
//!    maximum stamina, and a refused press shows and plays nothing; each switch plays the type's
//!    start / end sound for the controlled character only (FUN_0010e7a0) and sends a chatter tag
//!    (on; off with stamina left; off at 0) through the game's chatter gate (play_chatter.rs: each
//!    block's probability, replay delay, condition, distance and priority, and the world's one
//!    queue).
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
//!  - **Hawk's Stealth** (#123, `Stealth`): her `Sc_w1/w2_activate_stealth` switches it at its
//!    0.83 s event, and her invisibility controller (`Skill::cloak`, `Cloak`) fades her out over
//!    1.0 s, her skin's (`Skill::skin`, `Skin`) following it (her look, play_stealth.rs, #198).
//!    Her visibility, 1 less her level, scaled by a factor a shot drops to 0, is what the AI's
//!    sight (play.rs `nearest_hostile`) and the crosshair (play_reticle.rs) read; while she's
//!    stealthed a round fired costs 2 x the gun's Damage max and an item 5 or 15, her footsteps
//!    are a quarter as loud, and an AI Hawk holds fire. A press mid-fade is refused.
//!  - **Flint's Advanced Targeting** (#124, `Targeting`): on and off at the press (with a rifle
//!    her motions become the sniper set, play.rs `SpecialClips`), while on the crosshair's lock box
//!    grows to 0.4 of the screen (play_reticle.rs), so the guns, which aim at the lock, find the
//!    enemies near the crosshair; her HUD marker shows the box or the locked target
//!    (play_targeting.rs) and the crosshair fades out. Her gun stays raised and the camera in
//!    while it's on and for a while after (`Skill::raised`; the camera's aim shift follows it,
//!    play_camera.rs). Her rolls are
//!    doubled 15% of the time while on and x1.5 scoped (play_damage.rs), her rifles reach x1.25,
//!    she walks, and the empty clip, the switch key and the reload key end it.
//!
//! Squad deathmatch maps (`sdm_*`) take the deathmatch squad's rates (play.rs `load_map`,
//! `Game::load_squad_library`): take21 drains Tex at 6.55/s on sdm_e34.
//!
//! Test hooks: `BF_TEST_SKILL=<s>[,<s>...]` presses the skill key at those times;
//! `BF_TEST_SQUAD_SKILL=<character>,<s>[,<s>...]` gives a squadmate the Squad Command menu's
//! Toggle Ability at those times without the menu (its goal, #333);
//! `BF_TEST_STAMINA=<n>[@<s>][,...]` sets the controlled character's stamina (at those times, at
//! the start without one);
//! `BF_SKILL_CHATTER=<percent>` sets the skill lines' chance (100: every time);
//! `BF_TEST_DOUBLE=<percent>` sets the chance of Flint's doubled rolls (100: every one);
//! `BF_TEST_STEALTH_FLASH=<s>[,<s>...]` gives every stealthed Hawk a round's flash at those times
//! (her factor to 0, as a shot, without the shot or its cost: #198's check of the refraction's
//! flash from a squadmate's view, where an AI Hawk holds fire);
//! `BF_SKILL_LOG=1` logs each command, switch, sound and line, Tex's top-ups, refusals and second
//! gun, Hawk's refusals and costs, Flint's refusals, rifle mark and doubled rolls, and every
//! character's stamina each half second (with Hawk's level, visibility, skin and refraction while
//! it shows).

use super::*;
use bf_viewer::bf::character::{CombatSkills, SkillData, StealthSkill, Tracking, VengarSkill};

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
/// The chatter tags, sent through the chatter gate (FUN_00140a60, play_chatter.rs): switched on
/// (FUN_0010e470, 0x10e4c7); off with stamina left (FUN_0010e3e0, 0x10e425); off at 0 (0x10e442).
/// Heard in the takes on 13 of 35 ons (Tex, Hawk and Flint 13 of 26, Brutus 0 of 9), 3 of 23 offs
/// with stamina left (all Flint's), 2 of 9 offs at 0 (spec Part 1 section 4). The off line with
/// stamina left tests char+0x5a8, which only ever gets 0 in default.xbe (answer-120-chatter
/// section 2), so its variants needing 1 or 2 and up (Tex, Brutus, Hawk) are never said and
/// Flint's (no range) are; the line at 0 tests the enemies the speaker fights (FUN_0001b460).
const LINE_ON: u32 = 0x1C05_14BD;
const LINE_OFF: u32 = 0x0908_7C9C;
const LINE_EMPTY: u32 = 0xEE0A_5CBC;
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
/// FUN_001195c0: a stick length under this moves at the walk (0.65 at 0x3a53f0, the compare after
/// the caps at 0x11973a); the demo's walk key stands for it (`move_capped`).
const RUN_FROM: f32 = 0.65;
/// The lock box's scale Flint's +0x100 gives while on (0x13cd1d: 1.0 at 0x3a4f78), in place of
/// the gun's own h_e1e41c2d (the L-Shot-75's 0.1, the Jax-iP's 0.6).
const TARGETING_LOCK_BOX: f32 = 1.0;
/// Flint's scope past its first level (her +0x2c, 0x13c930, for a gun's zoom over 1): the zoom
/// plus the larger of this (the 10.0 stored at 0x13c94f) and the zoom + SECOND_ZOOM_MORE. So her
/// second level is 25 with the L-Shot-75 (zoom 10: x2.5 the first), 15 with the L-Shot-50 (5:
/// x3), 13 with the Foley (3), 12 with the MK-ASLT and the Kman Auto (2). The takes measure the
/// second level against the first at x2.45-2.52 (the L-Shot-75, todo/62 takes 01, 06, 08, 10) and
/// x2.97 (the L-Shot-50, todo/61 take01).
const SECOND_ZOOM_LEAST: f32 = 10.0;
/// ... (0x13c957 adds [0x3a4fd4] = 5.0 to the zoom)
const SECOND_ZOOM_MORE: f32 = 5.0;
/// The item use types (the IOU_ name table 0x3be61c) that FUN_00121170 hands a skill's +0x104
/// (at 0x12123b and the like): IOU_USE_IN_HAND 0 (the medkit, its h_1ee2f4ed in the level
/// objecttypes), IOU_PLACE_ON_GROUND 2 (Rollers, Sentries), IOU_THROW_TO_USE 3 (thrown grenades),
/// IOU_SKILL_MELEE_ATTACK 8 (the Power Blade). A grenade kit's own is its data's `use_type`.
pub const IOU_USE_IN_HAND: i64 = 0;
const IOU_PLACE_ON_GROUND: i64 = 2;
const IOU_THROW_TO_USE: i64 = 3;
const IOU_SKILL_MELEE_ATTACK: i64 = 8;
/// The use types Brutus's skill refuses (+0x1c, 0x13c160: `cmp eax, 1` / `cmp eax, 7` on the
/// stack's use type for him, FUN_001463c0): IOU_ATTACH_TO_FACE (the OrgSen's h_1ee2f4ed) and
/// IOU_ATTACH_TO_EQUIP_SLOT (the Hat's).
const IOU_ATTACH_TO_FACE: i64 = 1;
const IOU_ATTACH_TO_EQUIP_SLOT: i64 = 7;
/// What Hawk's +0x104 (0x13d340) takes from her stamina for an item used while stealthed: 5.0
/// (0x3a4fd4) for any, and 10.0 (0x3a4fdc) more for one placed or thrown (#35 take13: a Sonic
/// thrown, 15.1 in one step); nothing for IOU_SKILL_MELEE_ATTACK.
const STEALTH_ITEM_COST: f32 = 5.0;
const STEALTH_THROW_COST: f32 = 10.0;
/// What Hawk's +0xb0 (0x13d0a0) takes from her stamina for each round fired while stealthed: the
/// weapon data's +0x18, the `<Damage max>`, x 2 (`fadd st(0), st(0)`, 0x13d0d0). The Saryl-23's
/// 13.5: 27 (takes 12, 13: 27.1 a shot); the Jax-iR's 34.125: 68.25 (take14: her last 67.35 in
/// one shot).
const STEALTH_SHOT_COST: f32 = 2.0;
/// The material controllers' ramps (s): the invisibility controller's init (0x212380) sets both
/// its fade-in time (+8, 0x212398) and its fade-out time (+0xc, 0x21239b) to 1.0, and the body
/// controller's (kind 1, 0x212cb0) its own two to 1.0 as well. #35 takes 11-14, 18, 19: Hawk's
/// level 0 -> 1 and 1 -> 0 in 1.0 s, linearly (0.05 a 50 ms log row); take11 her body's presence
/// falling linearly over the same second (todo/198 leads §6).
const FADE_IN: f32 = 1.0;
const FADE_OUT: f32 = 1.0;
/// Where a controller switched off starts its fade-out (0x210ac1: its time set to 0x38d1b717,
/// 1e-4 s), so its level reads just under 1 from the frame it's switched off.
const FADE_OUT_START: f32 = 1e-4;
/// Where the invisibility controller's fades change halves (FUN_00212430), as a share of the
/// fade: 0.5 for the fade-in (0x407070) and for the fade-out (0x40706c).
const HALF_IN: f32 = 0.5;
const HALF_OUT: f32 = 0.5;
/// The material's lookup offset (FUN_00212430, state 1): 0.15 (0x40707c) x her visibility + 0.1
/// (0x407078) x sin(2.5 t) (0x3a51f0, a double), t the seconds since the fade-in ended. Her
/// visibility 0, the wobble alone: a period of 2.51 s (take01 4-14 s: the picture inside her
/// outline repeats every 2.6-2.8 s, todo/198 leads §6).
const OFFSET_SEEN: f32 = 0.15;
const WOBBLE: f32 = 0.1;
const WOBBLE_RATE: f32 = 2.5;
/// The least change of her visibility that FUN_00210950 pushes into the controller (2e-05).
const VIS_STEP: f32 = 2e-5;
/// The log's name for the toggle command GOAL_TOGGLE_COMBAT_SKILL sends (`Skill::ordered`): the
/// Squad Command menu's Toggle Ability (#333).
const ORDERED: &str = "the order's goal";

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
    /// Whether the character's kill is the skill's (vtable +8, which the death's lines FUN_00125f40
    /// ask of the killer's skill at 0x126030): its kill line is then h_13601f7b, else h_02da3fee
    /// (play_chatter.rs). Brutus's says no (0x20e200, the base's here); Tex's and Flint's while on
    /// (0x13c8e0: +0x1c bit 0 with an owner); Hawk's (0x13d300) while on with the inventory's
    /// +0x130 item of the type her +0x134 names, which the demo doesn't carry: no.
    fn special_kill(&self, _u: &Player) -> bool {
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
    /// The factor on the damage the character takes (vtable +0xd8, which the damage handler
    /// 0x125600 multiplies in at 0x125747, after the difficulty step: play_damage.rs `Hit::on`).
    /// The base gives 1.0 (0x10e3d0); Brutus's Vengar 0.85 while on (0x13c140).
    fn damage_factor(&self, _u: &Player) -> f32 {
        1.0
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
    /// The scope's level count, `n` the count asked about (vtable +0x28: the scope's level count
    /// FUN_00124840 asks it with 1 for a gun that zooms, and the way out of the scope FUN_0011af10
    /// for the top level). The base returns it (0x262a40): one level for everyone's skill but
    /// Flint's (0x13c910).
    fn scope_levels(&self, n: i64) -> i64 {
        n
    }
    /// The zoom at scope level `level` of a gun whose zoom is `zoom` (vtable +0x2c, asked by
    /// FUN_00124920 for a level above 0). The base gives the gun's zoom (0x222cf0); Flint's
    /// (0x13c930) more past her first level.
    fn scope_zoom(&self, zoom: f32, _level: i64) -> f32 {
        zoom
    }
    /// The scope view's flags (vtable +0xa8): with bit 0 set the scope frame isn't shown (the HUD
    /// widget FUN_000b4010, texture h_1807ec04: its update FUN_000b4070 shows it only while the
    /// character is scoped and this has bit 0 clear; play_hud.rs `update_scope`). The base gives
    /// 0 (0x20e200, and the other skills' 0x2bf0a0): the frame; Flint's 1, Tex's 8 (below).
    fn scope_view(&self) -> u32 {
        0
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
    /// 0x13cbf0 while on), which holds the gun's ready timer (`Skill::raised`): the ready pose,
    /// and the controlled character's camera in (the aim shift, play_camera.rs).
    fn keeps_gun_up(&self, _u: &Player) -> bool {
        false
    }
    /// Whether the scope is barred now: the demo's choice for Tex's berserk motions (see
    /// `Berserker`), none of the game's.
    fn bars_scope(&self, _u: &Player) -> bool {
        false
    }
    /// A round fired by the character's gun `def` (vtable +0xb0, which FUN_0022f640 calls from
    /// the weapon's fire loop FUN_0022f2a0 at the end of each trigger cycle that takes a round:
    /// a pellet burst once, play_shots.rs). The base does nothing (0x2c60c0); Hawk's (0x13d0a0)
    /// shows her and takes stamina while she's stealthed.
    fn fired(&mut self, _u: &mut Player, _def: &WeaponDef) {}
    /// An item used, by its use type (IOU_*, vtable +0x104, which the item use FUN_00121170 calls
    /// once the use starts: a throw, a placement, a medkit). The base does nothing (0x2c60c0);
    /// Hawk's (0x13d340) takes stamina while she's stealthed.
    fn used_item(&mut self, _u: &mut Player, _use_type: i64) {}
    /// Whether the character can use an item of use type `use_type` (IOU_*; vtable +0x1c, which
    /// the item function objects' "can use" 0x146c70 asks for the stack's type through its user's
    /// skill: no, and the item box shows it red and the use is refused). The base says yes
    /// (0x262a80: Tex's, Hawk's, Flint's); Brutus's (0x13c160) says no to IOU_ATTACH_TO_FACE and
    /// IOU_ATTACH_TO_EQUIP_SLOT (the OrgSen, the Hat), posting "Cannot use" (play_inventory.rs
    /// `use_other`; #196 take26).
    fn uses_item(&self, _use_type: i64) -> bool {
        true
    }
    /// How visible the character is, 0 to 1 (vtable +0xe4): the stamina update pushes it into its
    /// combat target every frame (0x11a934-0x11a94a), where the target selection
    /// (FUN_000bf6d0) and the AI's sight (FUN_0004d140) read it. The base's 1.0 (0x10e3d0);
    /// Hawk's (0x13d200) falls as she fades out.
    fn visibility(&self, _u: &Player) -> f32 {
        1.0
    }
    /// The factor on the character's footsteps' volume (FUN_00119020 at 0x11918e) and its
    /// movement noise's radius (FUN_0001aeb0) (vtable +0xe8). The base's 1.0 (0x10e3d0); Hawk's
    /// (0x13d210) a quarter stealthed.
    fn noise(&self, _u: &Player) -> f32 {
        1.0
    }
    /// Whether an AI-controlled character holds fire (vtable +0x108: the AI's attack check
    /// FUN_00024f80 refuses at 0x2507a while it's true). The base says no (0x20e200); Hawk's
    /// (0x13d3f0) while she's stealthed.
    fn holds_fire(&self, _u: &Player) -> bool {
        false
    }
    /// Whether the skill's toggle wants its character standing (vtable +0x11c, which only
    /// GOAL_TOGGLE_COMBAT_SKILL asks: its start 0x36e80 has the brain stand, 0x3f0e0 with 0, and
    /// its update 0x36f60 sends no toggle while the character kneels or crouch-walks, motion state
    /// 6 / 7). The base says no (0x20e200: Brutus's, Hawk's); Tex's and Flint's yes (0x213690).
    fn stands_to_toggle(&self) -> bool {
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
    /// the toggle command, asked for (the key, the black button) and not yet carried out
    pub request: bool,
    /// the toggle command GOAL_TOGGLE_COMBAT_SKILL sent this frame (the Squad Command menu's
    /// Toggle Ability, play_orders.rs `toggle_skill`, #333): carried out as the key's is; the goal
    /// sends it every frame until the skill has switched, so its refusals go unlogged, as the
    /// command at 0's do
    pub ordered: bool,
    /// the motion set's special motions taken (play.rs `SpecialClips`: Tex's berserk set, Flint's
    /// sniper set). Their toggles ask for them (the motion request MT_TOGGLE_SPECIAL, motion type
    /// 45) at an accepted press switching on, and for the normal ones back at one switching off:
    /// Tex's at 0x13d6ca (FUN_0012eab0(type, 45, 1)), under his toggle's overlay, not at its event;
    /// Flint's by returning 2 with a rifle (0x13cbd5; the command handler's request, 0x1224ee)
    pub special_motions: bool,
    /// stamina the skill's own action took this frame, drained with the next frame's drain: Brutus's
    /// charge hit (skill+0x2c = the h_0461a627 100 at 0x13c474; his drain 0x13c570 adds it, his
    /// update clears it, 0x13c0fd). Set by play_vengar.rs, which reaches the other characters
    pub pending: f32,
    /// a roll was doubled since the targeting marker last looked (Flint's +0xec sets its marker's
    /// flag 4, 0x13ccd2; the marker's corners show it for a frame and its update clears it,
    /// 0xb49cd: play_targeting.rs)
    pub double_hit: bool,
    /// the gun's ready timer as the skill holds it, s (char+0x7dc): Flint's update raises the gun
    /// every frame while on (FUN_00123ea0 from 0x13cbf0), which keeps the timer at 4 s or more
    /// (FUN_00123f60, 0x3c05fc), and it runs down after the off (`hold`). The ready pose shows
    /// while it runs (play.rs `upper_aim`): take17, off at 11.55 s, the rifle down at 15.5-15.8.
    /// Kept apart from the shot's own (`FireState`'s, play_shots.rs), as the demo's pose after a
    /// shot is its own (`Player::aim_hold`); the follow camera's aim shift reads both
    /// (play_camera.rs: held while either runs, 3 s more after)
    pub raised: f32,
    /// the character's invisibility material controller (its controllers of kind 2, char+0x54c),
    /// which Hawk's on and off switch and whose level her update reads back (`Cloak`); her
    /// refraction is drawn from it (play_stealth.rs). Off for everyone else
    pub cloak: Cloak,
    /// the character's body controller (kind 1, the map's primary), whose alpha is how much of
    /// her own skin is drawn (`Skin`, `opacity`): on for everyone, faded out while Hawk's cloak
    /// fades in or is on
    pub skin: Skin,
}

/// A material controller's fade: the base class of the controllers a character's model carries
/// (the map at char+0x54c), its state (+4) and the seconds in it (+0x10), with the base's switch
/// (+0x18, 0x210a90), frame (FUN_00210b90) and level (+0x20, 0x210b10). The demo has two of
/// them: Hawk's invisibility controller (`Cloak`, kind 2) and every character's body (`Skin`,
/// kind 1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fade {
    /// its state (+4)
    pub state: FadeState,
    /// seconds in it (+0x10)
    pub time: f32,
}

/// A material controller's states (+4, FUN_00210b90).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FadeState {
    /// 0: switched on, fading in over FADE_IN, then 1
    In,
    /// 1: on
    On,
    /// 2: switched off, fading out over FADE_OUT, then 3
    Out,
    /// 3: off, as made (0x210201: +4 = 3)
    #[default]
    Off,
}

impl Fade {
    /// The switch (the controller's +0x18, 0x210a90, through FUN_002107d0): on, an off one starts
    /// its fade-in from 0 and one fading out is on at once (its jump table 0x210ae4: state 3 -> 0,
    /// 2 -> 1); off, one on or fading in starts its fade-out (0 and 1 -> 2, at FADE_OUT_START), so
    /// a fade-in cut short jumps to the full level and fades from there (the game's).
    pub fn switch(&mut self, on: bool) {
        match (on, self.state) {
            (true, FadeState::Off) => *self = Fade { state: FadeState::In, time: 0.0 },
            (true, FadeState::Out) => *self = Fade { state: FadeState::On, time: 0.0 },
            (false, FadeState::In | FadeState::On) => *self = Fade { state: FadeState::Out, time: FADE_OUT_START },
            _ => {}
        }
    }

    /// One frame (FUN_00210b90): off, nothing; a fade whose time is out (at or past its length)
    /// moves on with its time back to 0 (in -> on, out -> off); then the time runs by the frame's.
    pub fn step(&mut self, dt: f32) {
        match self.state {
            FadeState::Off => return,
            FadeState::In if self.time >= FADE_IN => *self = Fade { state: FadeState::On, time: 0.0 },
            FadeState::Out if self.time >= FADE_OUT => *self = Fade { state: FadeState::Off, time: 0.0 },
            _ => {}
        }
        self.time += dt;
    }

    /// Its level (the controller's +0x20, 0x210b10, which FUN_00210880 reads for Hawk's
    /// skill+0x24): the fade-in's share done, 1 on, the fade-out's share left, 0 off. Not clamped:
    /// a fade's last frame reads a little past its end before the step moves it on (the takes'
    /// logs: 1.01-1.03 for a tick at the end of the fade-in).
    pub fn level(&self) -> f32 {
        match self.state {
            FadeState::In => self.time / FADE_IN,
            FadeState::On => 1.0,
            FadeState::Out => 1.0 - self.time / FADE_OUT,
            FadeState::Off => 0.0,
        }
    }
}

/// A character's material controller of kind 2: the BF_InvisMaterial controller (vtable
/// 0x394e64, 0x54 bytes), which FUN_002101d0 makes for a type whose controller mask has bit 2:
/// FUN_00210060 gives the type named "hawk" the mask 7 (kinds 1, 2, 4; mp_hawk 0x1f, Brutus 0x101,
/// most 0x19), so Hawk's model alone carries it. Made off (state 3). Its init (0x212380) makes the
/// material (class 0xf3bf04d1; its texture h_ff43a70b is loaded and never drawn) and sets both
/// fades to 1.0 s; its update (`update`: FUN_00212430, after the base's FUN_00210b90) sets the
/// material's alpha and lookup offset from its state, its fade's share and her visibility, which
/// its +0x24 (0x2126e0) hands the material before each draw (the offset to +0x28, the colour to
/// +0x2c..+0x38). The refraction is drawn by play_stealth.rs. Not here: its body alpha (+0x14 /
/// +0x18: her shadow's strength, FUN_002109b0, and the Power Blade's, which the demo hasn't) and
/// the second offset (+0x30 / +0x3c, 1.0 (0x407074) x her visibility x sin(5 t)), which the
/// material gets at +0x20 and never reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cloak {
    /// its fade (+4, +0x10)
    pub fade: Fade,
    /// her visibility as last pushed in (+0x1c, FUN_00210950 from her update)
    vis: f32,
    /// the offset the fade-in and the on state set (+0x2c), which the fade-out dies from
    base: f32,
    /// the material's lookup offset now (+0x38, the vertex shader's c24.w)
    pub offset: f32,
    /// the material's alpha now (+0x50, c26.w)
    pub alpha: f32,
}

impl Default for Cloak {
    /// As made (FUN_002101d0: off, its +0x1c 1.0) and set up (0x212380: the offsets 0, the colour
    /// and alpha 1).
    fn default() -> Self {
        Cloak { fade: Fade::default(), vis: 1.0, base: 0.0, offset: 0.0, alpha: 1.0 }
    }
}

impl Cloak {
    /// Switched on or off (`Fade::switch`): Hawk's on and off (0x13c650 / 0x13c670); then its
    /// frame's values worked out again with no time (`update`), so the switch's own frame draws
    /// the new state's: switched on from off, alpha 0 and no offset. Its update has already run
    /// this frame (`step`: the controllers' frame, then the toggle clip's event that switches it),
    /// so without this the switch frame drew what its update had left, the as-made alpha 1 on a
    /// map's first switch-on: one frame of a full-alpha tinted copy of her (#219). The game's
    /// order of the clip's event against the controllers' frame (FUN_00210560) isn't traced: with
    /// the event first, its first frame of the fade-in is at (dt / 0.5)^2, about 0 as here; with
    /// it after, the game would draw that frame with the values its update left too. #198 take01
    /// shows no such frame at its switch-on (her body's mean colour steps once at 3.04 s and then
    /// fades smoothly, frame by frame at 58 fps; whether it was that map's first switch-on isn't
    /// known), so the demo takes the switch's state.
    pub fn switch(&mut self, on: bool) {
        self.fade.switch(on);
        self.update(0.0);
    }

    /// Its level (`Fade::level`).
    pub fn level(&self) -> f32 {
        self.fade.level()
    }

    /// Whether it's switched on or fading (0x210b10's answer, states 0-2): the renderer draws its
    /// material then (FUN_000e51e0 puts her on the refraction list).
    pub fn active(&self) -> bool {
        self.fade.state != FadeState::Off
    }

    /// Her visibility pushed in (FUN_00210950, from her update at 0x13d173): only a change of
    /// VIS_STEP or more.
    pub fn push_visibility(&mut self, v: f32) {
        if (v - self.vis).abs() >= VIS_STEP {
            self.vis = v;
        }
    }

    /// One frame (its +0x14): the base's step (FUN_00210b90), then FUN_00212430 with x the share
    /// of its fade done:
    ///  - fading in: the alpha (x / 0.5)^2 and no offset over the first half; then the alpha 1
    ///    and the offset (1 - (1 - y)^2) x her visibility x 0.15, y the second half's share;
    ///  - on: the alpha 1, the offset 0.15 x her visibility + 0.1 x sin(2.5 t) (a shot, which
    ///    shows her, strengthens it for the quarter second her visibility takes to fall back);
    ///  - fading out: the alpha 1 and the offset dying as 1 - (x / 0.5)^2 from the last one the
    ///    fade-in or the on state set (+0x2c, which this state doesn't write: no wobble); then no
    ///    offset and the alpha (1 - y)^2.
    pub fn update(&mut self, dt: f32) {
        self.fade.step(dt);
        match self.fade.state {
            FadeState::In => {
                let x = self.fade.time / FADE_IN;
                if x >= HALF_IN {
                    let y = ((x - HALF_IN) / (1.0 - HALF_IN)).min(1.0);
                    self.alpha = 1.0;
                    self.base = (1.0 - (1.0 - y) * (1.0 - y)) * self.vis * OFFSET_SEEN;
                } else {
                    let a = (x / HALF_IN).min(1.0);
                    self.alpha = a * a;
                    self.base = 0.0;
                }
                self.offset = self.base;
            }
            FadeState::On => {
                self.alpha = 1.0;
                self.base = OFFSET_SEEN * self.vis + WOBBLE * (WOBBLE_RATE * self.fade.time).sin();
                self.offset = self.base;
            }
            FadeState::Out => {
                // (the halves' split, 1 - 0x40706c, kept at 0x4a7120)
                let x = self.fade.time / FADE_OUT;
                let split = 1.0 - HALF_OUT;
                if x >= split {
                    let y = ((x - split) / HALF_OUT).min(1.0);
                    self.alpha = (1.0 - y) * (1.0 - y);
                    self.offset = 0.0;
                } else {
                    let s = (x / split).min(1.0);
                    self.alpha = 1.0;
                    self.offset = (1.0 - s * s) * self.base;
                }
            }
            FadeState::Off => {}
        }
    }
}

/// A character's material controller of kind 1, its body (vtable 0x394e18, 0x28 bytes): the
/// controller map's primary kind (map+0x14) for every type the demo has (FUN_00210060), switched
/// straight to on as the map is made (FUN_00210840). It draws the model with its own materials at
/// its alpha (0x213120: `ctx+0x90 *= +0x14`), so its alpha is how much of her skin shows. The
/// map's frame switches it off while a controller of a kind in 0xc2 fades in or is on (Hawk's
/// cloak), so her skin fades out linearly as she fades out (her presence in take11, todo/198
/// leads §6) and back as the cloak goes off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skin(Fade);

impl Default for Skin {
    /// On, as made (FUN_00210840).
    fn default() -> Self {
        Skin(Fade { state: FadeState::On, time: 0.0 })
    }
}

impl Skin {
    /// The controller map's frame for it (FUN_00210560, from the character update at 0x11cc80),
    /// after the cloak's own update: its update with the frame's time (0x212cd0, through the
    /// base's FUN_00210b90); switched off if the cloak is fading in or on, on otherwise
    /// (FUN_002107d0 with the primary kind); then its update again with no time, which moves a
    /// fade at its end on. So switched off mid fade-in, the cloak starts its fade-out and her skin,
    /// then fading out, is on at once (the switch takes state 2 to 1): her body snaps back while
    /// the refraction dies.
    pub fn update(&mut self, cloak: &Cloak, dt: f32) {
        self.0.step(dt);
        self.0.switch(!matches!(cloak.fade.state, FadeState::In | FadeState::On));
        self.0.step(0.0);
    }

    /// Its alpha (+0x14 = +0x18, 0x212cd0): the fade-in's share done (+0x10 / +8), 1 on, the
    /// fade-out's share left (1 - +0x10 / +0xc), 0 off: its level (0x3a4f78 1.0, 0x3a4f88 0).
    pub fn alpha(&self) -> f32 {
        self.0.level()
    }
}

impl Skill {
    /// The skill of a character type (the spawn, FUN_0012f440, has the factory FUN_0012e590 make
    /// it by skill-set), with game-options' `<combat-skills>` tuning (each constructor copies its
    /// block).
    pub fn new(data: Option<&SkillData>, settings: &CombatSkills) -> Self {
        let kind: Option<Box<dyn SkillKind>> = match data.map(|d| d.skill_set) {
            Some(CSKILL_TEX) => Some(Box::new(Berserker { timer: 0.0, empty_drain: settings.berserk_empty_drain })),
            Some(CSKILL_BRUTUS) => Some(Box::new(Vengar { s: settings.vengar })),
            Some(CSKILL_HAWK) => Some(Box::new(Stealth::new(settings.stealth))),
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
/// grenade held (charging, `charge`), no reload, throw, medkit, weapon switch, weapon pickup (#17)
/// or toggle under way, and the skill's word. Only Tex's toggle (switching on, 0x13d5c4) and Hawk's
/// (0x13cfd2) ask it; Brutus's (0x13bfe0) and Flint's (0x13cad0) don't.
pub fn free_to_act(u: &Player) -> bool {
    !u.dead && !knocked_down(u) && u.charge <= 0.0 && u.reloading.is_none() && u.throwing.is_none() && u.using.is_none()
        && u.switching.is_none() && u.gun_use.is_none() && u.skill.toggling.is_none() && lets_act(u)
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

/// The factor `u`'s skill puts on the damage `u` takes (`SkillKind::damage_factor`, the skill's
/// vtable +0xd8 at 0x125747): Vengar's 0.85 while on, else 1.
pub fn damage_factor(u: &Player) -> f32 {
    u.skill.kind.as_ref().map_or(1.0, |k| k.damage_factor(u))
}

/// Whether a kill by `u` is its skill's (`SkillKind::special_kill`, the vtable +8 FUN_00125f40
/// asks at 0x126030): its kill line h_13601f7b over h_02da3fee (play_chatter.rs). No skill, no.
pub fn special_kill(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.special_kill(u))
}

/// Whether `u`'s skill is Brutus's Spirit of Vengar and it's on (play_vengar.rs: the charge, the
/// Vengar view; the vision mode 2 his on sets, char+0x644, 0x13c032).
pub fn vengar_on(u: &Player) -> bool {
    u.skill.on && u.skill.name() == Some(NAME_VENGAR)
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

/// A round fired by `u`'s gun `def` (FUN_0022f640 -> the skill's +0xb0, once for each round
/// the fire loop takes: play.rs `step_player`): Hawk's flash and cost while she's stealthed.
pub fn fired(u: &mut Player, def: &WeaponDef) {
    let Some(mut k) = u.skill.kind.take() else { return };
    k.fired(u, def);
    u.skill.kind = Some(k);
}

/// Whether `u` can use an item of use type `use_type` (IOU_*; `SkillKind::uses_item`, the skill's
/// +0x1c): not Brutus the OrgSen or the Hat. Yes without a skill.
pub fn uses_item(u: &Player, use_type: i64) -> bool {
    u.skill.kind.as_ref().is_none_or(|k| k.uses_item(use_type))
}

/// An item of use type `use_type` (IOU_*) used by `u` (FUN_00121170 -> the skill's +0x104, as
/// the use starts: play.rs's throw and placement, play_pickups.rs's medkit): Hawk's cost while
/// she's stealthed.
pub fn used_item(u: &mut Player, use_type: i64) {
    let Some(mut k) = u.skill.kind.take() else { return };
    k.used_item(u, use_type);
    u.skill.kind = Some(k);
}

/// How visible `u` is to the AI's sight and the crosshair's selection, 0 to 1 (the skill's +0xe4,
/// pushed into the combat target each frame): Hawk's falls as she fades out; 1 without a skill.
pub fn visibility(u: &Player) -> f32 {
    u.skill.kind.as_ref().map_or(1.0, |k| k.visibility(u))
}

/// The factor on `u`'s footsteps (the skill's +0xe8, FUN_00119020 at 0x11918e): a quarter for a
/// stealthed Hawk; 1 without a skill.
pub fn noise(u: &Player) -> f32 {
    u.skill.kind.as_ref().map_or(1.0, |k| k.noise(u))
}

/// Whether `u`, under the AI, holds fire (the skill's +0x108, which the AI's attack check
/// FUN_00024f80 asks at 0x2507a): a stealthed Hawk does.
pub fn holds_fire(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.holds_fire(u))
}

/// Whether `u`'s skill wants it standing to toggle (the skill's +0x11c, `SkillKind::
/// stands_to_toggle`: Tex's and Flint's), which the Squad Command menu's Toggle Ability goal asks
/// (play_orders.rs `toggle_ability`, `toggle_skill`).
pub fn stands_to_toggle(u: &Player) -> bool {
    u.skill.kind.as_ref().is_some_and(|k| k.stands_to_toggle())
}

/// Whether `u`'s skill fades its body (Hawk's Stealth, which switches the invisibility
/// controller her type has: `Cloak`), for play_stealth.rs to ready its copies.
pub fn cloaks(u: &Player) -> bool {
    u.skill.name() == Some(NAME_STEALTH)
}

/// How opaque `u`'s own skin is drawn, 0 to 1 (play_stealth.rs): its body controller's alpha
/// (`Skill::skin`, kind 1's 0x212cd0), which the game draws the model with. Its fades are linear
/// over 1.0 s, in step with the cloak's: out as Hawk's stealth fades in, back as it fades out (1
/// less the cloak's level, but for a switch-off mid fade-in, which brings it back at once). The
/// takes agree: #35 take11, her body's difference from the stealthed picture against the
/// picture's switch, 0.13 s after the log's: 0.50 at 0.49 s, 0.33 at 0.64, 0.19 at 0.84 on; 0.16,
/// 0.32, 0.50, 0.71, 0.91 at 0.17, 0.37, 0.57, 0.77, 0.97 s off.
pub fn opacity(u: &Player) -> f32 {
    u.skill.skin.alpha().clamp(0.0, 1.0)
}

/// The invisibility material over `u` this frame, while its controller is active (`Cloak`): its
/// alpha and its lookup offset (play_stealth.rs draws it). None while it's off: everyone but a
/// stealthed or fading Hawk.
pub fn refraction(u: &Player) -> Option<(f32, f32)> {
    u.skill.cloak.active().then_some((u.skill.cloak.alpha, u.skill.cloak.offset))
}

/// How far gun `def` reaches in `u`'s hands (FUN_002324c0: the bullet's range through the
/// holder's skill, +0x54): the shot's ray and its falloff (play.rs), the crosshair's centre ray
/// and the lock's reach (play_reticle.rs).
pub fn weapon_range(u: &Player, def: &WeaponDef) -> f32 {
    u.skill.kind.as_ref().map_or(def.range, |k| k.range(u, def, def.range))
}

/// `u`'s scope's level count for `n` (the skill's +0x28, `SkillKind::scope_levels`): one for
/// `n` 1, two for Flint's (play.rs `scope_levels`); a character without a skill keeps `n`.
pub fn scope_levels(u: &Player, n: i64) -> i64 {
    u.skill.kind.as_ref().map_or(n, |k| k.scope_levels(n))
}

/// The zoom at `u`'s scope level `level` for a gun of zoom `zoom` (the skill's +0x2c,
/// `SkillKind::scope_zoom`; play.rs `level_zoom`).
pub fn scope_zoom(u: &Player, zoom: f32, level: i64) -> f32 {
    u.skill.kind.as_ref().map_or(zoom, |k| k.scope_zoom(zoom, level))
}

/// Whether `u`'s scope view is the frame (the skill's +0xa8 with bit 0 clear, `SkillKind::
/// scope_view`: everyone's but Flint's, whose view is the full screen; play_hud.rs).
pub fn scope_frame(u: &Player) -> bool {
    u.skill.kind.as_ref().map_or(0, |k| k.scope_view()) & 1 == 0
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

/// Whether `u`'s skill holds the gun raised now: while it keeps it up and the ready timer's run
/// after (`Skill::raised`): play.rs shows the ready pose.
pub fn gun_raised(u: &Player) -> bool {
    u.skill.raised > 0.0
}

/// The skill's hold on the gun this frame (`Skill::raised`): the ready timer at READY_HOLD while
/// the skill keeps the gun up, else running down. The follow camera's aim shift holds while it
/// runs (play_camera.rs `step_shift`, for the controlled character).
///
/// A reload after the off ends the ready timer at once: #35 take29, where the reload right after
/// the off left the rifle lowered once it was done (6.6 s) and the camera came out at 7.6-8.0 s,
/// 2.3-3 s after the reload began (5.2), where take17's, with no reload, held 6.6 s. The reload
/// itself (FUN_00120d40) keeps a raised gun raised (FUN_00123f60 when FUN_000d4440 says it's up,
/// 0x120e63-0x120e6e); what clears the timer is FUN_0011e000, every frame of an overlay that
/// can't aim (its +0x3d bit 0: the reload's, inferred from that take; todo/126 answers.md Q7,
/// Q11), as play.rs `can_aim` does for the shot's timer.
fn hold(u: &mut Player, k: &dyn SkillKind, dt: f32) {
    u.skill.raised = if k.keeps_gun_up(u) {
        shots::READY_HOLD
    } else if u.reloading.is_some() {
        0.0
    } else {
        (u.skill.raised - dt).max(0.0)
    };
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

    fn scope_view(&self) -> u32 {
        // 0x13d3d0: 8, bit 0 clear: the frame (what bit 3 does isn't traced)
        8
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

    /// A kill in berserk is the skill's (0x13c8e0).
    fn special_kill(&self, u: &Player) -> bool {
        u.skill.on
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

    /// (+0x11c, 0x213690: his toggle has no berserk transition from a kneel)
    fn stands_to_toggle(&self) -> bool {
        true
    }
}

/// Brutus's Spirit of Vengar (CSKILL_BRUTUS; skill vtable 0x39d678, constructor FUN_0013bef0,
/// toggle FUN_0013bfe0): "Heightens perception and regenerates health." On or off at the press
/// (takes 06-09, 22, 23, 27, 30: in the first log sample after it), with no motion. The toggle
/// activates or deactivates and returns 0 either way (0x13c00e), and the command handler plays an
/// overlay only for a 1 (0x1224e4): the handler at 0x122500-0x122587 is the only lookup of the
/// overlay types 43 / 44, so his motion set's `spirit` overlay (`Sc_w1/w2_spirit`,
/// OMT_SPECIAL_ON) never plays from the White button. It doesn't ask FUN_00117210 either.
///  - **Regeneration** (his update 0x13c0b0): while on, h_eaa78fdf (1) hit point a second, up to
///    the max (take07: +1.01/s, 50.0 -> 65.04 over the 15 s on, none after the off).
///  - **Damage taken** (+0xd8, 0x13c140): x vengar-damage-multiplier (0.85) while on, on every hit
///    the damage handler takes (play_damage.rs `Hit::on`; take27: 0.84 per Saryl-45 hit).
///  - **Drain** (0x13c570): `stamina-cost` x dt, plus a charge hit's cost the frame after it
///    (`Skill::pending`, which his update clears: take22, S+0x2c 100 for one tick, then 0 and off).
///  - **The charge** (FUN_0013c1c0, which his update runs while he dashes with the skill on), the
///    **Vengar view** (vision mode 2: his on 0x13c020, his off FUN_00126df0) and the red fade at
///    each switch reach beyond him: play_vengar.rs.
///  - **Under the AI** (the Squad Command menu's Toggle Ability, #333): his on sets the vision
///    mode (char+0x644) and then, only with a player's view on him ([char+0x5e4], which
///    FUN_0006ff00 sets on the character a player's camera follows: the controlled one), the
///    view's mode, his kind-0x100 material controllers and the red fade (0x13c038 jumps past all
///    three without one); his off (FUN_00126df0) the same. So an AI Brutus's Vengar has its rules
///    (the regeneration, the x0.85, the charge, the drain) and nothing to see or hear: the demo
///    draws the view and the fade for the controlled character alone (play_vengar.rs), as the
///    code does. No take shows an AI Brutus in Vengar.
struct Vengar {
    /// his `<combat-skills>` block (h_f5dc6aac), as his constructor copies it
    s: VengarSkill,
}

impl SkillKind for Vengar {
    fn name(&self) -> u32 {
        NAME_VENGAR
    }

    fn toggle(&mut self, _u: &mut Player, _out: &mut Outside) -> Toggle {
        Toggle::Now { overlay: false }
    }

    fn drain(&self, u: &Player, cost: f32, dt: f32) -> f32 {
        cost * dt + u.skill.pending
    }

    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        if u.skill.on {
            u.health = (u.health + self.s.regen * dt).min(u.max_health);
        }
        // (the charge's cost has been drained: 0x13c0fd)
        u.skill.pending = 0.0;
        false
    }

    fn damage_factor(&self, u: &Player) -> f32 {
        if u.skill.on { self.s.damage_taken } else { 1.0 }
    }

    /// (+0x1c, 0x13c160: no to IOU_ATTACH_TO_FACE and IOU_ATTACH_TO_EQUIP_SLOT, on or off)
    fn uses_item(&self, use_type: i64) -> bool {
        !matches!(use_type, IOU_ATTACH_TO_FACE | IOU_ATTACH_TO_EQUIP_SLOT)
    }
}

/// Hawk's Stealth (CSKILL_HAWK; skill vtable 0x39d190, constructor FUN_0013cf10, the tuning
/// `StealthSkill`, game-options' block h_0ec919f1): "Turns invisible." (h_e0282878). Takes 10-14,
/// 18 and 19 (spec Part 4).
///
///  - **The toggle** (FUN_0013cf90): at stamina 0 or dead, with the skill on, off at once, no
///    clip, returning 0 (take11 15.26-15.31). Otherwise the press is a request: refused unless
///    she's free (FUN_00117210, 0x13cfd2: her clip under way among the rest), no request is
///    pending (skill+0x20, which her update clears every frame), her level isn't mid-fade
///    (skill+0x24 exactly 0 or 1: after an off at 0 the fade back holds the key for its second)
///    and, switching on, at 10% (take11 52.0 s: nothing at 9.06). Accepted, it sets the request
///    and returns 1: `Sc_w1/w2_activate_stealth` (2.00 s) plays, on and off, and switches at its
///    0.83 s event (FUN_00117660 at 0x118b8f; measured 0.89-0.97 s after the press was sent, less
///    the rig's input delay).
///  - **On / off** (0x13c650 / 0x13c670): her invisibility controller on or off (FUN_002107d0 on
///    char+0x54c, kind 2: `Skill::cloak`), which fades its refraction in or out over 1.0 s; her
///    skin's controller (`Skill::skin`) follows it the other way.
///  - **Update** (0x13d120), every frame: her level read back from the controller
///    (FUN_00210880), the request cleared, her visibility pushed into the controller
///    (FUN_00210950: it scales the material's refraction, play_stealth.rs), and the factor
///    (skill+0x28) climbing back toward 1 at stealth-level-recovery-rate, 4 a second
///    (FUN_0013d1a0, clamped 0..1).
///  - **Visibility** (+0xe4, 0x13d200): 1 - factor x level, what the AI's sight and the target
///    selection read (play_brain.rs `Sensor::scan`, play_reticle.rs): 1 visible, about 0 stealthed.
///  - **A shot** (+0xb0, 0x13d0a0), each round while on: the factor to 0, so she's fully visible
///    to them and back in 0.25 s (takes 12, 13: the factor 0 -> 1 in 0.25-0.3 s), and her stamina
///    less 2 x the gun's Damage max (the Saryl-23 27.0: 27.1 a shot). The skill stays on unless
///    that empties the bar (take13: the third shot of the burst did, off in that tick, the burst
///    went on). Her body stays invisible: her skin's controller follows the cloak's state, not the
///    visibility (`Skin::update`); only the refraction's offset reads it (`Cloak::update`), so a
///    shot strengthens the ripple for that quarter second (#198 take07).
///  - **An item** (+0x104, 0x13d340) while on: 5, and 10 more placed or thrown (take13: a Sonic
///    thrown, 15.1; she stays invisible and the skill on); nothing for the Power Blade's
///    IOU_SKILL_MELEE_ATTACK.
///  - **Noise** (+0xe8, 0x13d210): 0.25 + 0.75 x her visibility on her footsteps (play_liquids.rs
///    `step`; and the movement noise the game's AI hears, FUN_0001aeb0, which the demo's
///    AI doesn't).
///  - **Hold fire** (+0x108, 0x13d3f0): while on, the AI's attack check refuses her (play.rs
///    `update_player` gives her no target): an AI Hawk doesn't shoot while stealthed.
///  - **Drain** (0x13d030): `stamina-cost` x dt, x h_00894f2a (1) with the second weapon slot's gun
///    in hand. Not in: the x5 with the Power Blade in hand (the demo has no Power Blade).
struct Stealth {
    /// the request (skill+0x20): set by an accepted toggle, cleared by the next update
    request: bool,
    /// her level as her last update read it (skill+0x24): 0 visible, 1 stealthed
    level: f32,
    /// the stealth factor (skill+0x28): 1, dropped to 0 by a shot
    factor: f32,
    /// game-options' tuning, copied at the constructor
    cfg: StealthSkill,
}

impl Stealth {
    /// The skill as made (0x13cf10: the request 0, the level 0, the factor 1.0).
    fn new(cfg: StealthSkill) -> Self {
        Stealth { request: false, level: 0.0, factor: 1.0, cfg }
    }

    /// A refused toggle, with its reason in the log.
    fn refused(u: &Player, out: &Outside, why: &str) -> Toggle {
        if out.log {
            println!("t={:.2} {}: stealth refused: {why}", u.sim_time, unit_name(u));
        }
        Toggle::Refused
    }
}

impl SkillKind for Stealth {
    fn name(&self) -> u32 {
        NAME_STEALTH
    }

    fn toggle(&mut self, u: &mut Player, out: &mut Outside) -> Toggle {
        if u.skill.on && (u.stamina <= 0.0 || u.dead) {
            self.request = false;
            return Toggle::Now { overlay: false };
        }
        let why = if !free_to_act(u) { "busy" }
            else if self.request { "a request pending" }
            else if self.level != 0.0 && self.level != 1.0 { "mid-fade" }
            else if !u.skill.on && !can_activate(u) { "under 10%" }
            else { "" };
        if !why.is_empty() {
            return Self::refused(u, out, why);
        }
        self.request = true;
        Toggle::AtEvent(EV_STEALTH)
    }

    fn drain(&self, u: &Player, cost: f32, dt: f32) -> f32 {
        // (the inventory's slot in hand, +0x1cc: 2, the second; the demo's weapon index 1)
        cost * dt * if u.weapon == 1 { self.cfg.second_slot_drain } else { 1.0 }
    }

    fn on(&mut self, u: &mut Player, _out: &mut Outside) {
        u.skill.cloak.switch(true);
    }

    fn off(&mut self, u: &mut Player) {
        u.skill.cloak.switch(false);
    }

    fn update(&mut self, u: &mut Player, dt: f32) -> bool {
        self.level = u.skill.cloak.level();
        self.request = false;
        // her visibility into the controller (FUN_00210950, 0x13d173), before the factor climbs
        // (FUN_0013d1a0, 0x13d17e)
        let v = self.visibility(u);
        u.skill.cloak.push_visibility(v);
        self.factor = (self.factor + dt * self.cfg.level_recovery).clamp(0.0, 1.0);
        false
    }

    fn fired(&mut self, u: &mut Player, def: &WeaponDef) {
        if !u.skill.on {
            return;
        }
        self.factor = 0.0;
        let cost = STEALTH_SHOT_COST * def.damage;
        let was = u.stamina;
        u.stamina = (u.stamina - cost).clamp(0.0, u.max_stamina);
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: a shot while stealthed: seen (the factor 0), stamina {was:.2} - {cost:.2} -> {:.2}", u.sim_time,
                     unit_name(u), u.stamina);
        }
    }

    fn used_item(&mut self, u: &mut Player, use_type: i64) {
        if !u.skill.on || use_type == IOU_SKILL_MELEE_ATTACK {
            return;
        }
        let cost = STEALTH_ITEM_COST + if matches!(use_type, IOU_PLACE_ON_GROUND | IOU_THROW_TO_USE) { STEALTH_THROW_COST } else { 0.0 };
        let was = u.stamina;
        u.stamina = (u.stamina - cost).clamp(0.0, u.max_stamina);
        if std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} {}: an item used while stealthed (use type {use_type}): stamina {was:.2} - {cost:.0} -> {:.2}", u.sim_time,
                     unit_name(u), u.stamina);
        }
    }

    fn visibility(&self, _u: &Player) -> f32 {
        1.0 - self.factor * self.level
    }

    fn noise(&self, u: &Player) -> f32 {
        self.cfg.noise_floor + (1.0 - self.cfg.noise_floor) * self.visibility(u)
    }

    fn holds_fire(&self, u: &Player) -> bool {
        u.skill.on
    }
}

/// Whether `u`'s current motion has a motion type 45 transition, which Flint's toggle needs
/// either way (the motion record at char+0x1fc, its +0x534, 0x13caf3-0x13cb09): in ms_flint.xml
/// her idles, walk, run, side and back steps, dash, crouch idle and crouch walks have one, and the
/// sniper motions one back; the stand-to-crouch and back (MT 46), the jumps, dodges, dive, slide
/// and the throw (`throw_grenade`, MT 42) have none. So not getting down or up, in a jump, a
/// dodge, a dive or a slide, knocked down, nor with a grenade in the hand: charging, thrown or set
/// down, or its release pending (the demo's charge runs in her own motion, so counting it is the
/// demo's reading of the game's throw).
fn sniper_transition(u: &Player) -> bool {
    matches!(u.action, Action::None | Action::Crouched) && !u.slide_on && !knocked_down(u)
        && u.throwing.is_none() && u.charge <= 0.0 && u.pending_release.is_none()
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
///  - **The hold** (`Skill::raised`): raising the gun every frame holds its ready timer at 4 s, so
///    the ready pose stays up while on and 4 s after the off; the camera's aim shift (0x128280,
///    play_camera.rs) keeps the controlled character's camera in while that runs and 3 s more,
///    coming out over the last 0.67 s. take17: in by 2.27 s (on at 2.09), the rifle down at
///    15.5-15.8 s and the camera out over 18.13-18.6 s (off at 11.55); take29 the same way in.
///  - **The crosshair's lock** (+0x100 and +0xc4, play_reticle.rs): while on the lock box is 0.4
///    of the screen each way at scale 1.0, in place of 0.14 x 0.75 / 0.14 x the gun's
///    h_e1e41c2d, so an enemy up to 128 units across and 96 down from the crosshair is locked, and
///    the guns, which aim at the lock (play.rs), hit it. take28 had the reticle on the frozen
///    Shrike's body: 1 hit in 8 off, 7 in 8 on. The demo's bodies are solid cylinders the centre
///    ray hits, so there its shots on the body hit with the skill off too; its check aims 7
///    degrees off the body, inside the box: 0 in 8 off, 8 in 8 on.
///  - **Damage** (+0xec, 0x13cc70, in the roll FUN_0022c940): x1.5 scoped (char+0x7a8), on or off;
///    while on, a roll of the game's random numbers at most 0.15 doubles it and sets the marker's
///    flag 4 (`Skill::double_hit`). The two stack. take28: one hit of seven doubled (80.6 against
///    42.7), the scoped ones x1.36-1.59.
///  - **Range** (+0x54, 0x13ce80): her rifles' x1.25, on or off.
///  - **Walk** (+0x38, 0x13ceb0): 0.55 of the stick while on, as scoped: a walk (for the AI's Flint
///    too, and no sprint with the pistol either: the demo's choices, play.rs `step_player`).
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

    /// A kill with the targeting on is the skill's (0x13c8e0, Tex's function too).
    fn special_kill(&self, u: &Player) -> bool {
        u.skill.on
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

    fn scope_levels(&self, n: i64) -> i64 {
        // 0x13c910: one more than asked, none for none
        if n != 0 { n + 1 } else { 0 }
    }

    fn scope_zoom(&self, zoom: f32, level: i64) -> f32 {
        // 0x13c930: past the first level of a gun that zooms, its zoom plus the larger of
        // SECOND_ZOOM_LEAST and its zoom + SECOND_ZOOM_MORE
        if zoom > 1.0 && level > 1 { zoom + SECOND_ZOOM_LEAST.max(zoom + SECOND_ZOOM_MORE) } else { zoom }
    }

    fn scope_view(&self) -> u32 {
        // 0x2d5e40: no frame (her full-screen view)
        1
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

    /// (+0x11c, 0x213690, as Tex's: the order's goal stands her up first, though her own toggle
    /// takes a kneel)
    fn stands_to_toggle(&self) -> bool {
        true
    }
}

/// The test hooks' progress on this map: the last BF_TEST_SKILL press, BF_TEST_STAMINA poke,
/// BF_TEST_SQUAD_SKILL press and BF_TEST_STEALTH_FLASH flash carried out (their times, s).
#[derive(Resource)]
struct SkillHooks {
    last_press: f32,
    last_poke: f32,
    last_squad_press: f32,
    last_flash: f32,
}

impl Default for SkillHooks {
    fn default() -> Self {
        SkillHooks { last_press: -1.0, last_poke: -1.0, last_squad_press: -1.0, last_flash: -1.0 }
    }
}

/// The skill key and test hooks before the movement, every character's skill after it (so the
/// refill sees this frame's motion, and its sounds play this frame). The key comes after the
/// hand-over (`squad_control`), so a press in the frame control passes goes to the new character,
/// not the one handed over from; and after the ammo test hooks (`ammo::test_trigger`), whose
/// switch and reload presses it may turn into the toggle.
pub fn plugin(app: &mut App) {
    app.init_resource::<SkillHooks>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| {
            commands.insert_resource(SkillHooks::default());
        })
        .add_systems(Update, (
            skill_input.after(read_input).after(squad_control).after(ammo::test_trigger).before(update_player),
            update_skills.after(update_player).before(play_sounds),
        ).run_if(in_state(AppState::Playing)));
}

/// The skill key: V, the game's White button (VB_SPECIAL_ABILITY, game-options-en.xml line 151;
/// the player controller sends CT_TOGGLE_SPECIAL at 0x134cff), for the controlled character,
/// alive, with its controls on (not in a cutscene, held by a script or out of the world:
/// play_script.rs `blocks_input`, #256); and the test hooks BF_TEST_SKILL (presses, blocked the
/// same way), BF_TEST_STAMINA (pokes), BF_TEST_SQUAD_SKILL and BF_TEST_STEALTH_FLASH.
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
/// same way: play_itembox.rs's B key (#196).
fn skill_input(keys: Res<ButtonInput<KeyCode>>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut hooks: ResMut<SkillHooks>,
               reserve: Res<ammo::Reserve>) {
    if player.loaded.is_none() {
        return;
    }
    let t = player.sim_time;
    // test hook: BF_TEST_SQUAD_SKILL=<character>,<s>[,<s>...] gives that squadmate the Squad
    // Command menu's Toggle Ability at those times (once each; the character by name or 0-3): its
    // goal 127 pushed, as the menu does (FUN_000c1280 at 0xc1591; play_orders.rs
    // `toggle_ability`), without the menu
    if let Ok(v) = std::env::var("BF_TEST_SQUAD_SKILL") {
        let mut it = v.split(',').map(str::trim);
        let who = it.next().and_then(|c| c.parse::<usize>().ok().or_else(|| CHARACTERS.iter().position(|n| n.eq_ignore_ascii_case(c))));
        let due = it.filter_map(|x| x.parse::<f32>().ok()).filter(|&s| t >= s && hooks.last_squad_press < s)
            .fold(None, |a: Option<f32>, s| Some(a.map_or(s, |a| a.max(s))));
        if let (Some(c), Some(s)) = (who, due) {
            hooks.last_squad_press = s;
            if let Some(m) = squad.0.iter_mut().find(|m| m.character == c && m.in_squad() && !m.dead) {
                if std::env::var("BF_SKILL_LOG").is_ok() {
                    println!("t={t:.2} {}: the squad order's toggle (BF_TEST_SQUAD_SKILL)", unit_name(m));
                }
                orders::toggle_ability(m, t);
            }
        }
    }
    // test hook: BF_TEST_STEALTH_FLASH=<s>[,<s>...] gives every stealthed Hawk a round's flash at
    // those times: her skill's +0xb0 with a gun of no damage, so her factor drops to 0 as a
    // shot's does and nothing is fired or paid (the shot list's H5, done the trainer's way)
    if let Ok(v) = std::env::var("BF_TEST_STEALTH_FLASH") {
        let due = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).filter(|&s| t >= s && hooks.last_flash < s)
            .fold(None, |a: Option<f32>, s| Some(a.map_or(s, |a| a.max(s))));
        if let Some(s) = due {
            hooks.last_flash = s;
            let none = WeaponDef::default();
            for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()).filter(|u| cloaks(u) && u.skill.on) {
                fired(u, &none);
                if std::env::var("BF_SKILL_LOG").is_ok() {
                    println!("t={t:.2} {}: a flash (BF_TEST_STEALTH_FLASH)", unit_name(u));
                }
            }
        }
    }
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
    // not while the player's controls don't reach the character (#256): the toggle is the player
    // controller's command (0x134cff), and a script's hold switches that controller off
    // (FUN_00129530), as a cutscene does for every control (`read_input`, play_script.rs
    // `blocks_input`); nor under the item list or a menu (`paused`: the list's input locks,
    // FUN_000ac080, #196); a press then, the key's or BF_TEST_SKILL's, is lost, not kept for later
    if press && !player.dead && !script::blocks_input(&player) && !paused() {
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
    /// it's the one the player controls (its skill sounds play)
    controlled: bool,
    log: bool,
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
pub fn update_skills(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut reserve: ResMut<ammo::Reserve>,
                     mut last_log: Local<f32>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    let mut env = Env { controlled: true, log, reserve: &mut *reserve };
    step(&mut player, &mut env, dt);
    follow_cloak(&mut player, dt);
    for m in squad.0.iter_mut().filter(|m| m.in_squad()) {
        let mut env = Env { controlled: false, log, reserve: &mut *reserve };
        step(m, &mut env, dt);
        follow_cloak(m, dt);
    }
    // the stamina of each, every half second
    let t = player.sim_time;
    if log && (t * 2.0).floor() > (*last_log * 2.0).floor() {
        let list: Vec<String> = std::iter::once(&*player).chain(squad.0.iter().filter(|m| m.in_squad())).map(|u| {
            let state = if u.skill.on { "on".to_string() } else { format!("off x{}", refill_factor(u)) };
            // (Hawk's stealth while it shows: her controller's level, her visibility, her skin's
            // alpha and the refraction's alpha and offset)
            let cloak = if u.skill.cloak.active() || visibility(u) < 1.0 {
                format!(" (cloak {:?} level {:.2}, visibility {:.2}, skin {:.2}, refraction alpha {:.2} offset {:.3})", u.skill.cloak.fade.state,
                        u.skill.cloak.level(), visibility(u), u.skill.skin.alpha(), u.skill.cloak.alpha, u.skill.cloak.offset)
            } else {
                String::new()
            };
            format!("{} {:.2} {state}{cloak}", unit_name(u), u.stamina)
        }).collect();
        println!("t={t:.2} stamina: {}", list.join(" | "));
    }
    *last_log = t;
}

/// `u`'s body controller's frame (`Skin::update`), after its skill step: so a switch of the cloak
/// this frame (at the toggle clip's event, an off at 0 or at death) moves her skin in the same
/// frame, the two fades in step. The game's order of the clip's event and the controller map's
/// update isn't traced; this one keeps the skin at 1 less the cloak's level, as #123 measured.
fn follow_cloak(u: &mut Player, dt: f32) {
    let cloak = u.skill.cloak;
    u.skill.skin.update(&cloak, dt);
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

/// One character's skill this frame: the death handler's off, the overlay's switch, the toggle
/// command asked for, then the stamina update as FUN_0011a840 runs it. (Its chatter replay delays
/// are play_chatter.rs's.)
fn step(u: &mut Player, env: &mut Env, dt: f32) {
    // the material controllers' frame (FUN_00210560, from the character update at 0x11cc80,
    // before the stamina update's skill update at 0x11cedb), the dead's too: one switched off by
    // the death fades back in. The invisibility controller's here; the body's, which follows it,
    // once the skill has switched it this frame (`follow_cloak`)
    u.skill.cloak.update(dt);
    let Some(mut kind) = u.skill.kind.take() else { return };
    let k = kind.as_mut();
    if u.dead {
        // the death handler switches it off (0x115df5 -> FUN_0010e3e0: its end sound; its line
        // is silent, the speaker being dead), take30 off in the frame of death; the dead don't
        // change (take30: 94.11 for 9.4 s)
        u.skill.toggling = None;
        u.skill.request = false;
        u.skill.ordered = false;
        // (and the order's goal goes with the brain: the demo's choice, a dead character's goal
        // stack isn't traced)
        u.command.toggle = None;
        if u.skill.on {
            deactivate(u, k, env);
        }
        // (the motions the toggle asked for go with the body, and the hold: the ready timer is 0
        // for the dead, FUN_00115d50 at 0x115da4, and with it the camera's aim shift, 0x11ccb8)
        u.skill.special_motions = false;
        u.skill.raised = 0.0;
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
    // the Squad Command menu's Toggle Ability: its goal's command (play_orders.rs `toggle_skill`)
    if std::mem::take(&mut u.skill.ordered) {
        command(u, k, env, ORDERED);
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
    // the gun it keeps raised (Flint's), which the camera's aim shift follows
    hold(u, k, dt);
    // off (checked again: a skill that went off this frame refills this frame, take02)
    if !u.skill.on {
        u.stamina = (u.stamina + recovery * dt * refill_factor(u)).min(u.max_stamina);
    }
    u.skill.kind = Some(kind);
}

/// The toggle command (CT_TOGGLE_SPECIAL: the key's, the stamina update's at 0, a skill's own
/// end, the Squad Command menu's Toggle Ability goal's): the character's command handler
/// (0x1224a5) calls the skill's toggle, and its answer is carried out. (`why` is for the log.)
fn command(u: &mut Player, k: &mut dyn SkillKind, env: &mut Env, why: &str) {
    let was = u.skill.on;
    // (the one at 0 and the order's goal's come every frame: their refusals, the skill's own
    // reasons included, go unlogged, and so does a switch on the activate will refuse, under 10%:
    // Brutus's and Flint's toggles answer Now and leave the 10% to it)
    let repeated = why == "at 0" || why == ORDERED;
    let mut out = env.outside();
    out.log &= !repeated;
    let answer = k.toggle(u, &mut out);
    let does = match answer {
        Toggle::Refused => false,
        Toggle::Now { .. } => was || can_activate(u),
        Toggle::AtEvent(_) => true,
    };
    let quiet = repeated && !does;
    if env.log && !quiet && (!repeated || answer != Toggle::Refused) {
        println!("t={:.2} {}: toggle ({why}) at stamina {:.2}, {}: {answer:?}", u.sim_time, unit_name(u),
                 u.stamina, if was { "on" } else { "off" });
    }
    let log = env.log;
    env.log &= !quiet;
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
    env.log = log;
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
/// (play_chatter.rs, the same frame: its block's probability, 50 for the skill lines, or
/// BF_SKILL_CHATTER=<percent>; its replay delay, condition, distance, priority and the queue). A
/// dead speaker's (the death's off) is silent there (FUN_0013fab0).
fn switched(u: &mut Player, env: &mut Env, sound: u32, line: u32) {
    if env.controlled && sound != 0 {
        u.sound_queue.push((sound, SOUND_VOLUME));
    }
    chatter::say_with(u, line, std::env::var("BF_SKILL_CHATTER").ok().and_then(|v| v.parse::<i64>().ok()));
    if env.log {
        let what = if u.skill.on { "on" } else { "off" };
        println!("t={:.2} {}: skill {what} at stamina {:.2}; sound {}; line {line:08x} asked of the chatter gate (BF_CHATTER_LOG: what came of it)",
                 u.sim_time, unit_name(u), u.stamina,
                 if env.controlled && sound != 0 { format!("{sound:08x}") } else { "none".into() });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #219: switched on in a frame whose update has run, the cloak's values are the fade-in's
    /// start (alpha 0, no offset), not the as-made alpha 1; switched off from on, the fade-out's
    /// start (alpha 1, the offset it had).
    #[test]
    fn switch_sets_the_frame() {
        let mut c = Cloak::default();
        c.update(1.0 / 30.0);
        c.switch(true);
        assert_eq!((c.alpha, c.offset), (0.0, 0.0));
        for _ in 0..60 {
            c.update(1.0 / 30.0);
        }
        assert_eq!(c.fade.state, FadeState::On);
        let offset = c.offset;
        c.switch(false);
        assert_eq!(c.fade.state, FadeState::Out);
        assert_eq!(c.alpha, 1.0);
        assert!((c.offset - offset).abs() < 1e-3);
    }
}
