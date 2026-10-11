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
//!    (on; off with stamina left; off at 0).
//!  - **What ends a skill**: the toggle again, stamina at 0, death (the death handler calls the
//!    deactivate, 0x115df5), and each skill's own ends (its `update`). Handing control over
//!    doesn't: the skill stays on and drains on the character you left (take19).
//!
//! Squad deathmatch maps (`sdm_*`) take the deathmatch squad's rates (play.rs `load_map`,
//! `Game::load_squad_library`): take21 drains Tex at 6.55/s on sdm_e34.
//!
//! Test hooks: `BF_TEST_SKILL=<s>[,<s>...]` presses the skill key at those times;
//! `BF_TEST_STAMINA=<n>[@<s>][,...]` sets the controlled character's stamina (at those times,
//! at the start without one); `BF_SKILL_CHATTER=<percent>` sets the skill lines' chance (100:
//! every time); `BF_SKILL_LOG=1` logs each command, switch, sound and line, and every
//! character's stamina each half second.

use super::*;
use bf_viewer::bf::character::SkillData;

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
/// The chatter tags (FUN_00140a60): switched on (FUN_0010e470, 0x10e4c7); off with stamina
/// left (FUN_0010e3e0, 0x10e425); off at 0 (0x10e442). Heard in the takes on 13 of 35 ons,
/// 3 of 23 offs with stamina left, 2 of 9 offs at 0 (spec Part 1 section 4).
const LINE_ON: u32 = 0x1C05_14BD;
const LINE_OFF: u32 = 0x0908_7C9C;
const LINE_EMPTY: u32 = 0xEE0A_5CBC;
/// A skill line starts this long after the switch (the takes: 0.03-0.5 s, mostly 0.05-0.2 s).
const LINE_DELAY: f32 = 0.1;
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

/// What a skill's toggle makes of the command (the skill's vtable +0x04; in the game its return
/// picks the motion: 1 the type's OMT_SPECIAL_ON / OFF overlay, 2 MT_TOGGLE_SPECIAL).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Toggle {
    /// nothing happens (the skill's own refusal)
    Refused,
    /// switch now; with `overlay` the motion set's on / off overlay plays as well, if it has one
    Now { overlay: bool },
    /// play the on / off overlay; the skill switches at the clip's event (the id), handled as
    /// FUN_00117660 does
    AtEvent(u32),
}

/// A skill's own part, as the game's skill classes override their base class: the four below
/// (`Berserker`, `Vengar`, `Stealth`, `Targeting`), each of which its ticket fills in with its
/// effect. Each method's default is the base class's. The skill is taken out of its character
/// while these run, so they get the character itself (`u.skill.on`, `u.stamina`...).
pub trait SkillKind: Send + Sync {
    /// The skill's name, a string table id (vtable +0x4c): the HUD shows it while it's on.
    fn name(&self) -> u32;
    /// The toggle command (vtable +0x04), with the skill on or off (`u.skill.on`).
    fn toggle(&mut self, u: &Player) -> Toggle;
    /// The stamina used over `dt` while on (vtable +0xbc); the base: `cost` x dt.
    fn drain(&self, _u: &Player, cost: f32, dt: f32) -> f32 {
        cost * dt
    }
    /// Switched on (vtable +0x120): the skill's effects start.
    fn on(&mut self, _u: &mut Player) {}
    /// Switched off (vtable +0x124): its effects end.
    fn off(&mut self, _u: &mut Player) {}
    /// Every frame, on or off, while alive (vtable +0xc0). True sends the toggle command: the
    /// skill's own ends (Tex's other gun dry, Flint's clip dry: their tickets).
    fn update(&mut self, _u: &mut Player, _dt: f32) -> bool {
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
}

impl Skill {
    /// The skill of a character type (the spawn, FUN_0012f440, has the factory FUN_0012e590 make
    /// it by skill-set).
    pub fn new(data: Option<&SkillData>) -> Self {
        let kind: Option<Box<dyn SkillKind>> = match data.map(|d| d.skill_set) {
            Some(CSKILL_TEX) => Some(Box::new(Berserker::default())),
            Some(CSKILL_BRUTUS) => Some(Box::new(Vengar)),
            Some(CSKILL_HAWK) => Some(Box::new(Stealth)),
            Some(CSKILL_FLINT) => Some(Box::new(Targeting)),
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

/// Whether `u` is free to start a toggle (FUN_00117210: alive, not in a blocking state, no item
/// held; in the demo: not knocked down and no overlay under way, a reload, throw, item, weapon
/// switch or toggle). Tex's and Hawk's toggles ask it.
pub fn free_to_act(u: &Player) -> bool {
    !u.dead && !knocked_down(u) && u.reloading.is_none() && u.throwing.is_none() && u.using.is_none()
        && u.switching.is_none() && u.skill.toggling.is_none()
}

/// Whether `u` has the stamina switching on needs (FUN_0010e560's check, the one FUN_0010e470
/// makes).
pub fn can_activate(u: &Player) -> bool {
    u.stamina >= ACTIVATE_SHARE * u.max_stamina
}

/// Tex's Berserker (CSKILL_TEX; skill vtable 0x39d058, toggle 0x13d490). The toggle plays
/// `Sc_w1_bz_start` / `Sc_w1_bz_stop` and switches at their 0.50 s events (measured 0.53 s after
/// the toggle frame, on and off, takes 01-05, 21, 25, 26); its 1.0 s timer refuses presses
/// meanwhile, which keeps it on at 0 for that half second. His other refusals (crouched,
/// scoped, no second gun, the clip top-up), the second gun and the drain's x0.5 are #121's.
#[derive(Default)]
struct Berserker {
    /// the toggle timer (skill+0x20, s left)
    timer: f32,
}

impl SkillKind for Berserker {
    fn name(&self) -> u32 {
        NAME_BERSERKER
    }

    fn toggle(&mut self, u: &Player) -> Toggle {
        // switching on also needs him free (FUN_00117210) and 10% (take03 2.0 s: nothing at all)
        if self.timer > 0.0 || (!u.skill.on && (!free_to_act(u) || !can_activate(u))) {
            return Toggle::Refused;
        }
        self.timer = BERSERK_TIMER;
        Toggle::AtEvent(if u.skill.on { EV_BERSERK_OFF } else { EV_BERSERK_ON })
    }

    fn update(&mut self, _u: &mut Player, dt: f32) -> bool {
        self.timer = (self.timer - dt).max(0.0);
        false
    }
}

/// Brutus's Spirit of Vengar (CSKILL_BRUTUS; skill vtable 0x39d678, toggle FUN_0013bfe0): on or
/// off at the press (takes 06-09, 22, 23, 27, 30: in the first log sample after it); the motion
/// set's `spirit` overlay (`Sc_w1/w2_spirit`, 1.50 s) plays at the on, there's no off one. The
/// Vengar view, regeneration, damage taken and the charge are #122's.
struct Vengar;

impl SkillKind for Vengar {
    fn name(&self) -> u32 {
        NAME_VENGAR
    }

    fn toggle(&mut self, _u: &Player) -> Toggle {
        Toggle::Now { overlay: true }
    }
}

/// Hawk's Stealth (CSKILL_HAWK; skill vtable 0x39d190, toggle FUN_0013cf90). At stamina 0 or
/// dead it goes off at once, without a clip (take11 15.26-15.31); otherwise the press is a
/// request: refused unless she's free (FUN_00117210; her clip under way is a request pending)
/// and, switching on, at 10% (take11 52.0 s: nothing at 9.06); accepted, her
/// `Sc_w1/w2_activate_stealth` plays, on and off, and switches at its 0.83 s event (measured
/// 0.89-0.97 s after the press was sent, less the rig's input delay). Her fade (and its
/// mid-fade refusal), the firing and item costs and the AI's view of her are #123's.
struct Stealth;

impl SkillKind for Stealth {
    fn name(&self) -> u32 {
        NAME_STEALTH
    }

    fn toggle(&mut self, u: &Player) -> Toggle {
        if u.skill.on && (u.stamina <= 0.0 || u.dead) {
            return Toggle::Now { overlay: false };
        }
        if !free_to_act(u) || (!u.skill.on && !can_activate(u)) {
            return Toggle::Refused;
        }
        Toggle::AtEvent(EV_STEALTH)
    }
}

/// Flint's Advanced Targeting (CSKILL_FLINT; skill vtable 0x39d2c8, toggle FUN_0013cad0): on or
/// off at the press (takes 15-17, 29, 31), no overlay (her motion set has none; with a rifle
/// the game asks for MT_TOGGLE_SPECIAL, which never showed in the takes). Her toggle's need of a
/// gun with rounds in hand, the marker, aim and damage are #124's.
struct Targeting;

impl SkillKind for Targeting {
    fn name(&self) -> u32 {
        NAME_TARGETING
    }

    fn toggle(&mut self, _u: &Player) -> Toggle {
        Toggle::Now { overlay: false }
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

/// The skill key and test hooks before the movement, every character's skill after it (so the
/// refill sees this frame's motion, and its sounds play this frame).
pub fn plugin(app: &mut App) {
    app.init_resource::<SkillHooks>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| commands.insert_resource(SkillHooks::default()))
        .add_systems(Update, (
            skill_input.after(read_input).before(update_player),
            update_skills.after(update_player).before(play_sounds),
        ).run_if(in_state(AppState::Playing)));
}

/// The skill key: V, the game's White button (VB_SPECIAL_ABILITY, game-options-en.xml line 151;
/// the player controller sends CT_TOGGLE_SPECIAL at 0x134cff), for the controlled character,
/// alive; and the test hooks BF_TEST_SKILL (presses) and BF_TEST_STAMINA (pokes).
fn skill_input(keys: Res<ButtonInput<KeyCode>>, mut player: ResMut<Player>, mut hooks: ResMut<SkillHooks>) {
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
                println!("t={t:.2} {}: stamina set to {:.2}", CHARACTERS[player.character % CHARACTERS.len()], player.stamina);
            }
        }
    }
}

/// Every character's skill and stamina, after the movement (so the refill sees this frame's
/// motion): the controlled one first, then the squad (the AI's, as the game runs it for
/// everyone). Logs each half second with BF_SKILL_LOG.
fn update_skills(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut last_log: Local<f32>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    step(&mut player, &game.0, true, dt, log);
    for m in squad.0.iter_mut().filter(|m| m.in_squad()) {
        step(m, &game.0, false, dt, log);
    }
    // the stamina of each, every half second
    let t = player.sim_time;
    if log && (t * 2.0).floor() > (*last_log * 2.0).floor() {
        let list: Vec<String> = std::iter::once(&*player).chain(squad.0.iter().filter(|m| m.in_squad())).map(|u| {
            let state = if u.skill.on { "on".to_string() } else { format!("off x{}", refill_factor(u)) };
            format!("{} {:.2} {state}", CHARACTERS[u.character % CHARACTERS.len()], u.stamina)
        }).collect();
        println!("t={t:.2} stamina: {}", list.join(" | "));
    }
    *last_log = t;
}

/// The refill's factor for what `u` is doing (FUN_0011a840, by the motion state char+0x1f8):
/// dashing, MT_DASH (the demo's sprint, the player's and the AI's catching up) x0.25; crouched
/// still, MT_CROUCH (kneeling, not the crouch walk) x2; anything else x1, the kneel's way down
/// and up (MT_TRANSITION, measured x1: take02, take11) and walking included.
fn refill_factor(u: &Player) -> f32 {
    match (u.action, u.gait) {
        (Action::None, Gait::Sprint) => DASH_REFILL,
        (Action::Crouched, Gait::Idle) => CROUCH_REFILL,
        _ => 1.0,
    }
}

/// One character's skill this frame (`controlled`: the one the player controls, whose skill
/// sounds play): the death handler's off, the overlay's switch, the toggle command asked for,
/// then the stamina update as FUN_0011a840 runs it.
fn step(u: &mut Player, game: &Game, controlled: bool, dt: f32, log: bool) {
    let Some(mut kind) = u.skill.kind.take() else { return };
    let k = kind.as_mut();
    if u.dead {
        // the death handler switches it off (0x115df5 -> FUN_0010e3e0: its end sound and line,
        // take30 off in the frame of death); the dead don't change (take30: 94.11 for 9.4 s)
        u.skill.toggling = None;
        u.skill.request = false;
        if u.skill.on {
            deactivate(u, k, game, controlled, log);
        }
        u.skill.kind = Some(kind);
        return;
    }
    // knocked down, an overlay is cut short, and the switch it carried with it
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
                activate(u, k, game, controlled, log);
            } else {
                deactivate(u, k, game, controlled, log);
            }
        }
    }
    if std::mem::take(&mut u.skill.request) {
        command(u, k, game, controlled, "key", log);
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
            command(u, k, game, controlled, "at 0", log);
        }
    }
    // the skill's update, on or off (vtable +0xc0): its own ends
    if k.update(u, dt) {
        command(u, k, game, controlled, "its own end", log);
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
fn command(u: &mut Player, k: &mut dyn SkillKind, game: &Game, controlled: bool, why: &str, log: bool) {
    let was = u.skill.on;
    let answer = k.toggle(u);
    if log && (why != "at 0" || answer != Toggle::Refused) {
        println!("t={:.2} {}: toggle ({why}) at stamina {:.2}, {}: {answer:?}", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()],
                 u.stamina, if was { "on" } else { "off" });
    }
    match answer {
        Toggle::Refused => {}
        Toggle::Now { overlay } => {
            let switched = if was {
                deactivate(u, k, game, controlled, log);
                true
            } else {
                activate(u, k, game, controlled, log)
            };
            if switched && overlay {
                start_overlay(u, !was, None, log);
            }
        }
        Toggle::AtEvent(event) => {
            // (without the clip or its event: at once)
            if !start_overlay(u, !was, Some(event), log) {
                if was {
                    deactivate(u, k, game, controlled, log);
                } else {
                    activate(u, k, game, controlled, log);
                }
            }
        }
    }
}

/// Start the toggle's overlay, the on one or the off one, for the held weapon's stance (the
/// motion set's OMT_SPECIAL_ON / OFF, `Loaded::skill_clips`), the skill to switch at its
/// `event`. False when there's no such clip, or no such event in it: then nothing starts.
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
        println!("t={:.2} {}: {} overlay, {:.2} s{}", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()], if on { "on" } else { "off" },
                 clip.duration, switch_at.map_or(String::new(), |at| format!(", switches at {at:.2} s")));
    }
    u.skill.toggling = Some(Toggling { on, stance, time: 0.0, duration: clip.duration, switch_at });
    true
}

/// Switch `u`'s skill on (FUN_0010e470): it needs 10% stamina, else nothing happens (a refused
/// press shows and plays nothing: take03, take11); then the skill's own on, the type's
/// start-skill sound and the on line. Whether it switched.
fn activate(u: &mut Player, k: &mut dyn SkillKind, game: &Game, controlled: bool, log: bool) -> bool {
    if !can_activate(u) {
        if log {
            println!("t={:.2} {}: not switched on at stamina {:.2} (under 10%)", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()], u.stamina);
        }
        return false;
    }
    u.skill.on = true;
    if let Some(t) = u.skill.toggling.as_mut() {
        t.switch_at = None;
    }
    k.on(u);
    let sound = u.skill.data.as_ref().map_or(0, |d| d.start_sound);
    switched(u, game, controlled, sound, LINE_ON, log);
    true
}

/// Switch `u`'s skill off (FUN_0010e3e0): the skill's own off, the type's end-skill sound and a
/// line, the off line with stamina left (0x10e425) or the one at 0 (0x10e442; the stamina tested
/// at 0x10e40d).
fn deactivate(u: &mut Player, k: &mut dyn SkillKind, game: &Game, controlled: bool, log: bool) {
    u.skill.on = false;
    // (an overlay still to switch it has nothing left to do: Hawk at 0 mid-clip goes off at once)
    if let Some(t) = u.skill.toggling.as_mut() {
        t.switch_at = None;
    }
    k.off(u);
    let sound = u.skill.data.as_ref().map_or(0, |d| d.end_sound);
    let line = if u.stamina > 0.0 { LINE_OFF } else { LINE_EMPTY };
    switched(u, game, controlled, sound, line, log);
}

/// A switch's sound and line. The sound plays in 2D at full volume for the controlled
/// character only (FUN_0010e7a0 asks FUN_00112790; take18: an AI Hawk's switches are silent);
/// it isn't cut by the next switch (take08). The line (FUN_00140a60) comes at the chance its
/// chatter block's h_ea21ae4b gives read as a percentage (10 on, 20 off, 30 off at 0): **a
/// guess**, the gate isn't traced (spec Part 1 section 4, open question 1; the takes heard 13 of
/// 35, 3 of 23 and 2 of 9). BF_SKILL_CHATTER=<percent> sets it.
fn switched(u: &mut Player, game: &Game, controlled: bool, sound: u32, line: u32, log: bool) {
    if controlled && sound != 0 {
        u.sound_queue.push((sound, SOUND_VOLUME));
    }
    let name = CHARACTERS[u.character % CHARACTERS.len()];
    let chance = std::env::var("BF_SKILL_CHATTER").ok().and_then(|v| v.parse::<i64>().ok())
        .unwrap_or_else(|| game.chatter_chance.get(name).and_then(|c| c.get(&line)).copied().unwrap_or(0));
    let roll = u.random(100) as i64;
    let said = roll < chance && u.speaking <= 0.0 && u.quote_in.is_none();
    if said {
        say(u, game, line, LINE_DELAY);
    }
    if log {
        let what = if u.skill.on { "on" } else { "off" };
        let line_said = u.quote_in.filter(|_| said).map_or("none".to_string(), |(_, id)| format!("{id:08x}"));
        println!("t={:.2} {name}: skill {what} at stamina {:.2}; sound {} ; line {line:08x} ({chance}%): {line_said}", u.sim_time, u.stamina,
                 if controlled && sound != 0 { format!("{sound:08x}") } else { "none".into() });
    }
}
