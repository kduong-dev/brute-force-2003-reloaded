//! A level's triggers (levels-<lvl>.xmb, #220): every element built on the game's trigger base,
//! read as data. `bf_play` runs them (src/bin/play_triggers.rs, viewer/docs/triggers.md).
//!
//! The base (its attribute parser FUN_001fedb0, constructor FUN_001fe080):
//!
//!   <router-trigger name enabled h_137926ff h_1eec0e75 h_f657f799 h_0a82e377 h_f090ebbb iD ...>
//!     <h_031cd9f5>                     the entries, in order
//!       <base Type signal action match-id h_170a9b80 h_f136d22d h_e2da3955/>
//!       <anim command wwit-debugging animation-archetype time-scale ..><base ../></anim>
//!       <calculator value><base ../></calculator>      <script command time-scale><base/></script>
//!     </h_031cd9f5>
//!     <h_e91cf6a8 objects="..."/>       its targets: the triggers it sends to (FUN_001ff0a0)
//!     <h_0884288c objects="..."/>       the editor's back-references, read and dropped
//!     <h_0b6b92ba objects="..."/>       its objects: what it listens to or acts on
//!     <transform>..</transform>         (detectors, spawn-triggers, sound-triggers ...)
//!
//! An element is a trigger when it carries the base's h_137926ff (all 500 of the tutorial's: 58
//! routers, 54 timers, 47 spawn-triggers, 27 plates, 14 proximity detectors ...). Its class is
//! its element's name (`Trigger::tag`); each class has its own vtable (0x395168-0x3967c8, the
//! parser at +0x14, the update at +0x1c, the signal handler at +0x60: code map, "Levels and
//! triggers").

use bevy::math::{Mat4, Vec3};

use super::bxml::{Element, Value};
use super::hash::h;

/// The game's trigger signals, TRIG_SIG_ (the name table 0x3bef5c, read whole for #28): what an
/// event carries (its +4) and what an entry answers.
pub mod sig {
    /// ENTER_DETECTOR: a detector's occupant has been inside 0.1 s (FUN_001f8db0)
    pub const ENTER_DETECTOR: i64 = 0;
    /// EXIT_DETECTOR: an occupant has been out of it 0.2 s
    pub const EXIT_DETECTOR: i64 = 1;
    /// TRUE: a calculator's or a gamevar-trigger's test held
    pub const TRUE: i64 = 2;
    /// FALSE: a calculator's or a gamevar-trigger's test failed
    pub const FALSE: i64 = 3;
    /// ENABLED: a trigger switched on, to its targets (FUN_001ff0f0)
    pub const ENABLED: i64 = 4;
    /// DISABLED: a trigger switched off, to its targets (FUN_001ff0f0, FUN_001feac0)
    pub const DISABLED: i64 = 5;
    /// DESTROYED: an object (a character's death, a spawn copy's) to its listeners
    pub const DESTROYED: i64 = 6;
    /// TIMER_PULSE: a timer ran out (0x2002f0)
    pub const TIMER_PULSE: i64 = 8;
    /// CALCULATOR: a calculator's test held (0x1ff5f0), before its TRUE
    pub const CALCULATOR: i64 = 10;
    /// USER_SIGNAL1: a trigger's USER_SIG1 action, or a cutscene script's "103, 1, <signal>"
    /// (leads §4.1). USER_SIGNAL2 follows it; 3-8 are 32-37 (USER_SIGNAL3)
    pub const USER_SIGNAL1: i64 = 11;
    /// USER_SIGNAL2: a trigger's USER_SIG2 action, or a cutscene script's
    pub const USER_SIGNAL2: i64 = 12;
    /// FIRST_UPDATE: each enabled trigger to itself on its first update (FUN_001fe660)
    pub const FIRST_UPDATE: i64 = 13;
    /// USE_OBJECT: a world button used (its h_eaf8a35b, 14 on every one; 0x163060)
    pub const USE_OBJECT: i64 = 14;
    /// PICKUP_OBJECT: a pickup taken, to its listeners
    pub const PICKUP_OBJECT: i64 = 15;
    /// EVENT_MATCH: an event-trigger's AI event seen (step 5)
    pub const EVENT_MATCH: i64 = 16;
    /// SPAWNED: a spawn copy, to its listeners as it appears (FUN_002096e0)
    pub const SPAWNED: i64 = 24;
    /// FINISHED: a sound-trigger's sound ended, a menu closed (steps 4, 5)
    pub const FINISHED: i64 = 25;
    /// ANIM_START: an anim-trigger starts its clip forward (FUN_00204bb0), to its targets (the
    /// door sounds)
    pub const ANIM_START: i64 = 31;
    /// ANIM_BACK_START: an anim-trigger starts its clip backward (FUN_00204bb0), to its targets
    pub const ANIM_BACK_START: i64 = 49;
    /// USER_SIGNAL3: the first of USER_SIGNAL3-8 (32-37), a trigger's USER_SIG3-8 actions
    pub const USER_SIGNAL3: i64 = 32;
    /// ENTER_DETECTOR_TO_SELF: to a detector's own handler, with each ENTER_DETECTOR it sends
    /// (FUN_001f8db0)
    pub const ENTER_TO_SELF: i64 = 39;
    /// EXIT_DETECTOR_TO_SELF: to a detector's own handler, with each EXIT_DETECTOR it sends
    pub const EXIT_TO_SELF: i64 = 40;
    /// VALUE_CHANGED: a gamevar-trigger's write changed its out operand (0x1fff6c), before its
    /// TRUE / FALSE
    pub const VALUE_CHANGED: i64 = 41;
    /// EMPTIED_DETECTOR: a detector's last occupant gone
    pub const EMPTIED_DETECTOR: i64 = 42;
    /// OPERATED: a spawn-trigger's copy made (FUN_002099e0), a menu opened
    pub const OPERATED: i64 = 46;
    /// ENTER_DETECTOR_FIRST: the first occupant of an empty detector, with its ENTER_DETECTOR
    pub const ENTER_FIRST: i64 = 50;

    /// The game's name of a signal (the TRIG_SIG_ table without its prefix: -1 INVALID, the
    /// data's "any"), for logs.
    pub fn name(s: i64) -> String {
        let n = match s {
            -1 => "INVALID", 0 => "ENTER_DETECTOR", 1 => "EXIT_DETECTOR", 2 => "TRUE", 3 => "FALSE", 4 => "ENABLED", 5 => "DISABLED",
            6 => "DESTROYED", 7 => "ALARM", 8 => "TIMER_PULSE", 9 => "DAMAGED", 10 => "CALCULATOR", 11 => "USER_SIGNAL1", 12 => "USER_SIGNAL2",
            13 => "FIRST_UPDATE", 14 => "USE_OBJECT", 15 => "PICKUP_OBJECT", 16 => "EVENT_MATCH", 17 => "RESERVE_4EVT", 18 => "BEFORE_USE_OBJECT",
            19 => "DEBUG_MENU", 20 => "DEBUG_RESERVE1", 21 => "DEBUG_RESERVE2", 22 => "CRASHED",
            23 => "GOAL_FINISHED", 24 => "SPAWNED", 25 => "FINISHED", 26 => "PC_RETICULE_TRACK", 27 => "PC_LOOKING_AT",
            28 => "ELEVATOR_CALL", 29 => "ELEVATOR_UP_FLOOR", 30 => "ELEVATOR_DN_FLOOR", 31 => "ANIM_START", 32 => "USER_SIGNAL3",
            33 => "USER_SIGNAL4", 34 => "USER_SIGNAL5", 35 => "USER_SIGNAL6", 36 => "USER_SIGNAL7", 37 => "USER_SIGNAL8",
            38 => "ANIM_END", 39 => "ENTER_DETECTOR_TO_SELF", 40 => "EXIT_DETECTOR_TO_SELF", 41 => "VALUE_CHANGED",
            42 => "EMPTIED_DETECTOR", 43 => "SPAWN_SQUAD_EXTRA1", 44 => "SPAWN_SQUAD_EXTRA2", 45 => "SPAWN_SQUAD_EXTRA3",
            46 => "OPERATED", 47 => "RECLONE_SQUAD", 48 => "ABORT_SPECIAL_USE", 49 => "ANIM_BACK_START", 50 => "ENTER_DETECTOR_FIRST",
            51 => "MENU_ACCEPT", 52 => "MENU_BACK_UP", 53 => "MENU_X_BUTTON", 54 => "MENU_Y_BUTTON",
            _ => return format!("signal {s}"),
        };
        n.to_string()
    }

    /// A signal by its number or its name (with or without "TRIG_SIG_", any case; "USER1" for
    /// USER_SIGNAL1), for the test hooks.
    pub fn parse(s: &str) -> Option<i64> {
        let s = s.trim();
        if let Ok(n) = s.parse::<i64>() {
            return Some(n);
        }
        let up = s.to_ascii_uppercase();
        let up = up.trim_start_matches("TRIG_SIG_");
        let up = match up.strip_prefix("USER").and_then(|n| n.parse::<i64>().ok()) {
            Some(n) => format!("USER_SIGNAL{n}"),
            None => up.to_string(),
        };
        (-1..64).find(|&k| name(k) == up)
    }
}

/// The game's trigger actions, TRIG_ACT_ (the name table 0x3bed0c, read whole for #28): what an
/// entry does.
pub mod act {
    /// OPERATE: the class's own action (a router passes the signal on, a timer starts, a
    /// spawn-trigger spawns, an anim-trigger plays its clip ...)
    pub const OPERATE: i64 = 1;
    /// ENABLE: the base's: on, ENABLED to the targets (FUN_001ff0f0)
    pub const ENABLE: i64 = 2;
    /// DISABLE: the base's: off, its DISABLED lost (FUN_001ff0f0)
    pub const DISABLE: i64 = 3;
    /// STOP: a sound-trigger's stop (0x206450, as STOP_SOUND), a dialog-trigger's (0x206d20, as
    /// STOP_DIALOG), a letterbox-trigger's (the bars out over its hide-duration, 0x207480; #222)
    pub const STOP: i64 = 4;
    /// TOGGLE_ENABLE: the base's: ENABLE when off, DISABLE when on (FUN_001ff0f0)
    pub const TOGGLE_ENABLE: i64 = 5;
    /// PLAY_SOUND: a sound-trigger plays its sound (0x206450, as OPERATE)
    pub const PLAY_SOUND: i64 = 6;
    /// STOP_SOUND: a sound-trigger stops it (its voice fading out)
    pub const STOP_SOUND: i64 = 7;
    /// PAUSE_SOUND: a sound-trigger pauses its voice and its clock
    pub const PAUSE_SOUND: i64 = 8;
    /// RESUME_SOUND: a sound-trigger resumes them
    pub const RESUME_SOUND: i64 = 9;
    /// PLAY_DIALOG: a sound-trigger queues its sound on the world's dialog queue; a
    /// dialog-trigger's character says its line (0x206d20, as OPERATE)
    pub const PLAY_DIALOG: i64 = 10;
    /// STOP_DIALOG: a sound-trigger's stop, a dialog-trigger's line stopped
    pub const STOP_DIALOG: i64 = 11;
    /// PAUSE_DIALOG: a dialog-trigger counts it and does nothing (0x206d20's table)
    pub const PAUSE_DIALOG: i64 = 12;
    /// RESUME_DIALOG: the same
    pub const RESUME_DIALOG: i64 = 13;
    /// INC_COUNTER: a calculator's counter + 1 (0x1ff5f0)
    pub const INC_COUNTER: i64 = 14;
    /// DEC_COUNTER: a calculator's counter - 1
    pub const DEC_COUNTER: i64 = 15;
    /// MODIFY_COUNTER: a calculator's counter + the entry's value
    pub const MODIFY_COUNTER: i64 = 16;
    /// SET_COUNTER_FROM_SIG_DATA: a calculator's counter set to the event's data
    pub const SET_COUNTER_FROM_SIG_DATA: i64 = 17;
    /// DISPLAY_TIMER: a timer's (by its name, the countdown shown); counted but doing nothing
    /// here (0x200190)
    pub const DISPLAY_TIMER: i64 = 26;
    /// UNDISPLAY_TIMER: a timer's (by its name, the countdown hidden); counted but doing nothing
    /// here
    pub const UNDISPLAY_TIMER: i64 = 27;
    /// TIMER_FULL_RESET: a timer's: stop and reload (0x200190)
    pub const TIMER_FULL_RESET: i64 = 28;
    /// RESET_COUNTDOWN_TIME: a timer's: reload, running on if it was
    pub const RESET_COUNTDOWN_TIME: i64 = 29;
    /// SPAWN_RESET: a re-armable spawn-trigger armed again (0x209ac0)
    pub const SPAWN_RESET: i64 = 36;
    /// USER_SIG1: the first of USER_SIG1-8 (37-44): the base's, USER_SIGNAL1-8 to the targets
    pub const USER_SIG1: i64 = 37;
    /// USER_SIG8: the last of USER_SIG1-8
    pub const USER_SIG8: i64 = 44;

    /// The game's name of an action (the TRIG_ACT_ table without its prefix), for logs.
    pub fn name(a: i64) -> String {
        let n = match a {
            1 => "OPERATE", 2 => "ENABLE", 3 => "DISABLE", 4 => "STOP", 5 => "TOGGLE_ENABLE", 6 => "PLAY_SOUND", 7 => "STOP_SOUND",
            8 => "PAUSE_SOUND", 9 => "RESUME_SOUND", 10 => "PLAY_DIALOG", 11 => "STOP_DIALOG", 12 => "PAUSE_DIALOG", 13 => "RESUME_DIALOG",
            14 => "INC_COUNTER", 15 => "DEC_COUNTER", 16 => "MODIFY_COUNTER", 17 => "SET_COUNTER_FROM_SIG_DATA", 18 => "PLAY_MOVIE",
            19 => "STOP_MOVIE", 20 => "PAUSE_MOVIE", 21 => "UNPAUSE_MOVIE", 22 => "TOGGLE_PAUSE_MOVIE", 23 => "UP", 24 => "DOWN", 25 => "CALL",
            26 => "DISPLAY_TIMER", 27 => "UNDISPLAY_TIMER", 28 => "TIMER_FULL_RESET", 29 => "RESET_COUNTDOWN_TIME", 30 => "MOVE_FORWARD",
            31 => "MOVE_FORWARD_LOOP", 32 => "MOVE_FORWARD_OSCILLATE", 33 => "MOVE_BACKWARD", 34 => "MOVE_BACKWARD_LOOP",
            35 => "MOVE_BACKWARD_OSCILLATE", 36 => "SPAWN_RESET", 37 => "USER_SIG1", 38 => "USER_SIG2", 39 => "USER_SIG3", 40 => "USER_SIG4",
            41 => "USER_SIG5", 42 => "USER_SIG6", 43 => "USER_SIG7", 44 => "USER_SIG8", 45 => "RESERVE1", 46 => "RESERVE2", 47 => "LOOP_MOVIE",
            _ => return format!("action {a}"),
        };
        n.to_string()
    }
}

/// An entry's Type (the SH_ table, 0x3bf924): SH_ANIM, an anim-trigger's (its `<anim>`).
pub const SH_ANIM: i64 = 1;
/// An `<anim>` entry's play modes (wwit-debugging, the AM_ table 0x3bd41c): 3-5 play backward
/// (AM_BACKWARD, _LOOP, _OSCILLATE), 0-2 forward.
pub const AM_BACKWARD: i64 = 3;
/// An `<anim>` entry's command AC_PLAY (the AC_ table 0x3bd3c4), one of the two FUN_00204bb0
/// sends ANIM_START / ANIM_BACK_START for.
pub const AC_PLAY: i64 = 0;
/// An `<anim>` entry's command AC_RESUME (the AC_ table 0x3bd3c4), the other.
pub const AC_RESUME: i64 = 3;

/// The trigger base's h_137926ff: present on every trigger element (what `read` finds them by),
/// and when true the trigger disables itself after any entry acts (+0x3d bit 3, FUN_001feac0).
const H_DISABLE_AFTER_ENTRY: u32 = 0x1379_26FF;
/// h_1eec0e75 (+0x3d bit 2): disables itself after its first event (FUN_001ff000).
const H_DISABLE_AFTER_EVENT: u32 = 0x1EEC_0E75;
/// h_f657f799 (+0x3d bit 1): dies after its first event (FUN_001ff000: bit 4 set).
const H_DIE_AFTER_EVENT: u32 = 0xF657_F799;
/// h_f090ebbb (+0x3d bit 4): dead from the start: it neither acts nor sends.
const H_DEAD: u32 = 0xF090_EBBB;
/// A trigger's entries list, h_031cd9f5 (FUN_001fedb0).
pub const H_ENTRIES: u32 = 0x031C_D9F5;
/// A trigger's targets, h_e91cf6a8: the triggers it sends to (FUN_001ff0a0).
pub const H_TARGETS: u32 = 0xE91C_F6A8;
/// A trigger's objects, h_0b6b92ba: what it listens to (FUN_001febb0) or acts on.
pub const H_OBJECTS: u32 = 0x0B6B_92BA;
/// An entry's h_170a9b80 (the parse record's +0x10: 0x18e060; the runtime entry's +0x18:
/// 0x1fb1c0): disable the trigger once spent.
const H_DISABLE_WHEN_SPENT: u32 = 0x170A_9B80;
/// An entry's h_f136d22d (the parse record's +0x14; the runtime entry's +0x10): its use limit.
const H_LIMIT: u32 = 0xF136_D22D;
/// An entry's h_e2da3955 (the parse record's +0x18; the runtime entry's +0x14): its uses.
const H_USED: u32 = 0xE2DA_3955;
/// A router's h_f0d6857f (+0x40, FUN_00208f50): its entries filter what it passes (else it
/// passes everything but FIRST_UPDATE, its entries unread: 0x208fc0).
const H_ROUTER_FILTERS: u32 = 0xF0D6_857F;
/// A timer's h_f92b0b15 (+0x4a, FUN_00200080): at its pulse it stops and reloads (0x2002f0).
const H_TIMER_STOPS: u32 = 0xF92B_0B15;
/// A calculator's test, h_015914ab (+0x48, FUN_001ff560): 0 ==, 1 <, 2 >, 3 <=, 4 >= the target.
const H_CALC_TEST: u32 = 0x0159_14AB;
/// A spawn-trigger's template, h_1299668b (its parser 0x209250: +0xc8).
const H_TEMPLATE: u32 = 0x1299_668B;
/// A spawn-trigger's copy's name, h_14f15a34 (+0xd0).
const H_COPY_NAME: u32 = 0x14F1_5A34;
/// A spawn-trigger's world point, h_131de144 (+0xd4; not its facing, see `Class::Spawn`).
const H_FACE_POINT: u32 = 0x131D_E144;
/// A spawn-trigger's fade-in time, h_eeae96e7 (+0xdc; FUN_00209620 hands it to the copy).
const H_SPAWN_FADE: u32 = 0xEEAE_96E7;
/// A spawn-trigger's delay, h_eab6952f (+0xe0; 0x209b5e).
const H_SPAWN_DELAY: u32 = 0xEAB6_952F;
/// A spawn-trigger's re-armable flag, h_0f95f576 (+0xe4 bit 2; SPAWN_RESET re-arms it). It
/// doesn't leave the trigger unarmed: the constructor arms every one (+0xe4 = (+0xe4 & 0xe8) |
/// 0x28 at 0x1fd6ba-0x1fd6d6) and the parser only sets bit 2 (0x209386-0x20939e).
const H_REARM: u32 = 0x0F95_F576;
/// A spawn-trigger's h_0fa4f510 (+0xe4 bit 5, set by the constructor too, cleared by the data's
/// false): a squad-type character's copy is kept only if its member flag says it's in the
/// squad (FUN_00209900); without it the copy sets that flag (the session's vtable +0xbc).
const H_MEMBER_CHECK: u32 = 0x0FA4_F510;
/// A detector's h_e25d95e3 (+0xf4, FUN_001f8c90): disabled, it still keeps its occupants and
/// enables itself when one leaves (FUN_001f9680, FUN_001f8db0), sending nothing meanwhile.
const H_KEEP_WHEN_DISABLED: u32 = 0xE25D_95E3;
/// A detector's physics phantom, h_051ac63e (+0x104): the plate's box as an Ipion surface of the
/// level's objects-<lvl>.ipn (FUN_001f99a0), the size's box exactly (tutorial: h_e3a770e9 30 x 10
/// x 4 -> +-30 +-10 +-4; the building plates' 1.5 x 2 x 2 -> +-1.5 +-2 +-2).
const H_PHANTOM: u32 = 0x051A_C63E;
/// A `<filter>`'s squad types, h_19656b35 (its parser 0x18b0f0: +0).
const H_SQUAD_TYPES: u32 = 0x1965_6B35;
/// A `<filter>`'s other types (not the squad's), h_1d083f6a (+1).
const H_OTHER_TYPES: u32 = 0x1D08_3F6A;
/// A `<filter>`'s any character, h_115c7b01 (+2).
const H_ANY_CHARACTER: u32 = 0x115C_7B01;
/// A `<filter>`'s a human player's, h_e707ecc1 (+3).
const H_HUMAN: u32 = 0xE707_ECC1;
/// A `<filter>`'s invert, h_ebfd2b20 (+4): every test turned over.
const H_INVERT: u32 = 0xEBFD_2B20;
/// A gamevar-trigger's operation, h_015914ab (+0x50; its parser 0x1ffac0): see `Class::Gamevar`.
const H_GAMEVAR_OP: u32 = 0x0159_14AB;
/// A gamevar-trigger's operands, in the order the trigger keeps them (0x1ffac0 hands each to
/// 0x1ff9d0 with its index: `out` 0, A h_1554ca7a 1, B h_0e5d9bc0 2; +0x40 + 4i the value, +0x4c + i
/// the kind byte).
const H_OPERANDS: [u32; 3] = [0xF39E_8D99, 0x1554_CA7A, 0x0E5D_9BC0];
/// An operand's kind, h_18bbf97f (FUN_0018cbe0, the byte's low 7 bits): see `Operand`.
const H_OPERAND_KIND: u32 = 0x18BB_F97F;
/// An operand's invert, h_1fe1ecd1 (FUN_0018cbe0, the byte's bit 7): the value is 1 - it.
const H_OPERAND_INVERT: u32 = 0x1FE1_ECD1;
/// A score-trigger's key, h_19ca8cf3 (+0x44; FUN_0020b900): the cash record's entry it adds to;
/// the null name (the data's "") is the record's own totals.
const H_SCORE_KEY: u32 = 0x19CA_8CF3;
/// A score-trigger's h_08f457d8 (+0x48 bit 2, set by the constructor 0x20bb73): add under its key
/// (h_19ca8cf3), else under the null name.
const H_SCORE_KEYED: u32 = 0x08F4_57D8;
/// A score-trigger's h_1747360f (+0x48 bit 0, set by the constructor): not read by its handler
/// (0x20ba00); false on five of the data's 75 (e06's two 300s, e16's 1200, e23's 280).
const H_SCORE_FLAG0: u32 = 0x1747_360F;
/// A score-trigger's h_11819686 (+0x48 bit 1, set by the constructor): the amount scaled by
/// 1 + 0.1 x the difficulty (0x20babd-0x20bacd).
const H_SCORE_SCALED: u32 = 0x1181_9686;

/// The gamevar-trigger's operations (h_015914ab; its handler's jump table 0x200010 over 0-10).
pub mod op {
    /// A == B (|A - B| under 1e-5, 0x3a4fd8)
    pub const EQUAL: i64 = 0;
    /// A < B
    pub const LESS: i64 = 1;
    /// A <= B
    pub const LESS_EQUAL: i64 = 2;
    /// A > B
    pub const GREATER: i64 = 3;
    /// A >= B
    pub const GREATER_EQUAL: i64 = 4;
    /// A (the tutorial's objective states: `state` = 1 on completion, 0 at the start)
    pub const ASSIGN: i64 = 5;
    /// the out flag's default (FUN_001fdeb0 on the out operand's record)
    pub const DEFAULT: i64 = 6;
    /// B + A
    pub const ADD: i64 = 7;
    /// A - B
    pub const SUBTRACT: i64 = 8;
    /// B x A
    pub const MULTIPLY: i64 = 9;
    /// A / B (FLT_MAX for B 0)
    pub const DIVIDE: i64 = 10;
}

/// The operand kinds (h_18bbf97f, its value enum read by FUN_001853b0; FUN_001ffb80 evaluates
/// them).
pub mod operand {
    /// a game flag, by its path (`gameflag`): its value as a float (FUN_00077e80); 0 without it
    pub const GAMEFLAG: i64 = 0;
    /// `constant`
    pub const CONSTANT: i64 = 1;
    /// a random float in [0, 1) x `constant` (FUN_00233b90)
    pub const RANDOM: i64 = 2;
    /// the event's data (+0x10, its bits read as a float)
    pub const SIGNAL_DATA: i64 = 3;
    /// a random whole number: floor(random x `constant`)
    pub const RANDOM_INT: i64 = 4;
}

/// A gamevar-trigger's operand (`<h_1554ca7a>` A, `<h_0e5d9bc0>` B, `<out>`; FUN_0018cbe0).
#[derive(Clone, Debug, Default)]
pub struct Operand {
    /// h_18bbf97f: the kind (`operand::`); `out` is written for GAMEFLAG, kept for CONSTANT
    pub kind: i64,
    /// h_1fe1ecd1: 1 - the value (on `out`: 1 - the result, before it's written)
    pub invert: bool,
    /// `gameflag`: the flag's path (missions/m00/h_eaf792f4/state, tex/h_03761692 ...)
    pub path: Vec<u32>,
    /// `constant` (+0x40 + 4i): CONSTANT's value, RANDOM's and RANDOM_INT's scale
    pub constant: f32,
}

/// One entry of a trigger: its `<base>` (0x18e060's record, copied into the runtime entry by its
/// class's vtable +4: 0x1fb1c0 base, 0x1fb320 anim, 0x1fb5e0 calc) with its container's own
/// attributes (`<anim>`'s, `<calculator>`'s, `<script>`'s).
#[derive(Clone, Debug)]
pub struct Entry {
    /// Type: the SH_ enum (0 BASE, 1 ANIM, 3 CALC, 6 SCRIPT)
    pub kind: i64,
    /// signal (+4): the TRIG_SIG_ it answers. The match (FUN_001feb60) is exact: the data's -1
    /// ("any") entries never match an event, and the routers that have them pass everything
    /// without reading their entries (0x208fc0)
    pub signal: i64,
    /// action (+8): the TRIG_ACT_ it takes
    pub action: i64,
    /// match-id (+0xc): the sender's iD it answers (0 on every one, as every trigger's iD is)
    pub match_id: i64,
    /// h_170a9b80 (+0x18): once spent, the trigger sends DISABLED and disables itself
    /// (FUN_001feac0)
    pub disable_when_spent: bool,
    /// h_f136d22d (+0x10): how many times it acts (2147483647: always; 1: once)
    pub limit: i64,
    /// h_e2da3955 (+0x14): how many times it has (0 in the data; enabling the trigger zeroes
    /// it, 0x1fefc0)
    pub used: i64,
    /// its container's attributes (an `<anim>`'s command, wwit-debugging, animation-archetype,
    /// time-scale ...; a `<calculator>`'s value; a `<script>`'s command, time-scale); empty for a
    /// plain `<base>`
    pub extra: Vec<(u32, Value)>,
}

impl Entry {
    /// Its container's attribute `k`, as a whole number.
    fn extra_i64(&self, k: u32) -> Option<i64> {
        self.extra.iter().find(|(n, _)| *n == k).and_then(|(_, v)| v.as_i64())
    }

    /// Its container's attribute `k`, as a float.
    fn extra_f32(&self, k: u32) -> Option<f32> {
        self.extra.iter().find(|(n, _)| *n == k).and_then(|(_, v)| v.as_f32())
    }

    /// An `<anim>` entry's clip (animation-archetype, the anim entry's +0x1c), 0 none.
    pub fn anim_clip(&self) -> u32 {
        self.extra.iter().find(|(n, _)| *n == h("animation-archetype")).and_then(|(_, v)| v.as_hash()).unwrap_or(0)
    }

    /// An `<anim>` entry's command (AC_, the anim entry's +0x24).
    pub fn anim_command(&self) -> i64 {
        self.extra_i64(h("command")).unwrap_or(AC_PLAY)
    }

    /// An `<anim>` entry's play mode (wwit-debugging, AM_, +0x28).
    pub fn anim_mode(&self) -> i64 {
        self.extra_i64(h("wwit-debugging")).unwrap_or(0)
    }

    /// An `<anim>` entry's time-scale (+0x2c): how fast the clip plays (the tutorial gate's 2.5).
    pub fn time_scale(&self) -> f32 {
        self.extra_f32(h("time-scale")).filter(|s| *s > 0.0).unwrap_or(1.0)
    }

    /// A `<calculator>` entry's value (+0x1c): MODIFY_COUNTER's step.
    pub fn value(&self) -> i64 {
        self.extra_i64(h("value")).unwrap_or(0)
    }

    /// A `<script>` entry's command (the script-trigger entry's +0x1c, FUN_00204ec0's switch): 0
    /// or 1 play the script from its start, 2 pause it, 3 resume it, 4 stop it (#222).
    pub fn script_command(&self) -> i64 {
        self.extra_i64(h("command")).unwrap_or(0)
    }
}

/// A detector's `<filter>` (0x18b0f0; the test FUN_001fb980: every part must pass).
#[derive(Clone, Debug, Default)]
pub struct Filter {
    /// h_19656b35: a character of a squad type (type h_e78efbb2, type+0x1a8 bit 0: FUN_001fb850)
    pub squad_types: bool,
    /// h_1d083f6a: a character of any other type (FUN_001fb810)
    pub other_types: bool,
    /// h_115c7b01: any character (FUN_001fb890)
    pub characters: bool,
    /// h_e707ecc1: a character with a human player (FUN_001fb7d0 -> FUN_00112800)
    pub human: bool,
    /// h_ebfd2b20: each test inverted
    pub invert: bool,
    /// alignment (+8): the object's team, -1 any (FUN_001fb770)
    pub alignment: i64,
    /// `object`: named objects (FUN_001fb8b0); `object-type`: object types (FUN_001fb910)
    pub objects: Vec<u32>,
    pub object_types: Vec<u32>,
}

/// What a detector watches: a plate's box or a proximity detector's sphere.
#[derive(Clone, Copy, Debug)]
pub enum Volume {
    /// plate-detector: its `size` is the box's half-extents in the trigger's frame (the phantom
    /// surface h_051ac63e is that box: see H_PHANTOM)
    Plate(Vec3),
    /// proximity-detector: a sphere of `radius` (the phantom FUN_001fa330 makes)
    Sphere(f32),
}

/// A sound-trigger's h_15bc1769 (+0x8d bit 0; parser 0x205e7c): its sound plays only while a
/// player's character is within its max-distance (FUN_00206b50).
const H_SOUND_NEAR: u32 = 0x15BC_1769;
/// A sound-trigger's h_e725a1a5 (+0x8c; 0x205f45), also the dialog-trigger's (+0x48): the
/// squad's chatter is hushed while the line plays (FUN_00141100 on the chatter manager).
const H_SOUND_HUSH: u32 = 0xE725_A1A5;
/// A sound-trigger's h_1f625953 (+0x90; 0x205eb1): its voice's fade-in (s), to the sound
/// system's vtable +0x50 (FUN_002060c0).
const H_SOUND_FADE_IN: u32 = 0x1F62_5953;
/// A sound-trigger's h_05cad875 (+0x94; 0x205ed9): its voice's fade-out (s), the same call.
const H_SOUND_FADE_OUT: u32 = 0x05CA_D875;
/// A sound-trigger's h_0269d394 (+0x9c; 0x205efd): its min distance (the DirectSound buffer's,
/// FUN_0017eb90), -1 its library entry's.
const H_SOUND_MIN_DISTANCE: u32 = 0x0269_D394;

/// A sound-trigger's own attributes (vtable 0x395d50; its parser 0x205d00 after the base's,
/// FUN_001fedb0), with its constructor's values where the data has none (0x1fce50).
#[derive(Clone, Debug)]
pub struct SoundTrigger {
    /// `sound-object` (+0x40): the sound library entry it plays
    pub sound: u32,
    /// `enable-looping` (+0x4c): its voice loops, and it never ends by itself (no FINISHED)
    pub looping: bool,
    /// `Type` (+0x54): 0 heard alike everywhere (2D), 1 at its place (3D: its first object's,
    /// else its transform's), 2-5 the four kinds whose volume follows the players' share of the
    /// listeners (FUN_00206880) and that send no FINISHED (FUN_001fcf00); 4 and 5 at their place too
    pub kind: i64,
    /// `HARDPOINT` (+0x58): empty in the tutorial's; what reads it isn't traced
    pub hardpoint: u32,
    /// h_15bc1769 (+0x8d bit 0, H_SOUND_NEAR)
    pub near: bool,
    /// h_e725a1a5 (+0x8c, H_SOUND_HUSH)
    pub hush: bool,
    /// h_1f625953, h_05cad875 (+0x90, +0x94): its voice's fade-in and fade-out (s)
    pub fade_in: f32,
    pub fade_out: f32,
    /// `max-distance` (+0x98): the voice's max distance, and the range of h_15bc1769's start and
    /// stop; -1 its library entry's
    pub max_distance: f32,
    /// h_0269d394 (+0x9c): the voice's min distance; -1 its library entry's
    pub min_distance: f32,
    /// `volume` (h_eff0deab, +0xa8): 0-100, -1 its library entry's (FUN_00206250)
    pub volume: f32,
    /// `priority` (h_e146dc86, +0xac): -1 on every tutorial one; not read by the paths here
    pub priority: i64,
}

/// A trigger's class-specific attributes, for the classes the runtime runs (the rest keep their
/// element, `Trigger::element`, for the steps that run them).
#[derive(Clone, Debug)]
pub enum Class {
    /// router-trigger (parser FUN_00208f50): h_f0d6857f, `out-signal` (+0x44; -1 the same)
    Router { filters: bool, out_signal: i64 },
    /// sound-trigger (vtable 0x395d50, parser 0x205d00): see `SoundTrigger`
    Sound(SoundTrigger),
    /// dialog-trigger (vtable 0x3952a0, parser FUN_00206cb0): `dialog` (+0x40, the line: a sound
    /// library entry) and h_e725a1a5 (+0x48, true unless the data says not: its constructor
    /// 0x1fd430)
    Dialog { dialog: u32, hush: bool },
    /// timer-trigger (FUN_00200080): `enable-timer` (+0x48 running), `enable-looping` (+0x49),
    /// h_f92b0b15 (+0x4a), `initial-value` (+0x40), `current-value` (+0x44)
    Timer { running: bool, looping: bool, stops: bool, initial: f32, current: f32 },
    /// calculator-trigger (FUN_001ff560): `counter` (+0x40), `target` (+0x44), h_015914ab (+0x48)
    Calculator { counter: i64, target: i64, test: i64 },
    /// spawn-trigger (0x209250): see the H_ constants
    Spawn { template: u32, copy_name: u32, face_point: u32, fade: f32, delay: f32, rearm: bool, member_check: bool },
    /// plate-detector, proximity-detector (FUN_001f8c90 and their own parsers)
    Detector { volume: Volume, filter: Filter, keep_when_disabled: bool, phantom: u32 },
    /// anim-trigger (vtable 0x395ef0: the base's attributes only; its entries are `<anim>`)
    Anim,
    /// destroyer-trigger (vtable 0x396170, parser 0x200a10: duration, start, end)
    Destroyer,
    /// gamevar-trigger (vtable 0x3960a0, parser 0x1ffac0, handler 0x1ffc40): OPERATE works out
    /// h_015914ab (`op::`) of A and B, writes it to `out`, sends VALUE_CHANGED when that changed
    /// it, then TRUE (the result not 0) or FALSE. `operands` as the trigger keeps them: out, A, B
    Gamevar { op: i64, operands: [Operand; 3] },
    /// score-trigger (vtable 0x3953d8, parser FUN_0020b900, handler 0x20ba00): OPERATE adds
    /// `amount` (x 1 + 0.1 x the difficulty with h_11819686) to the campaign squad's cash, under
    /// `key` with h_08f457d8 (else the null name)
    Score { amount: f32, key: u32, keyed: bool, flag0: bool, scaled: bool },
    /// every other class: its base actions only, until its step runs it
    Other,
}

/// A level trigger (see the module notes).
#[derive(Clone, Debug)]
pub struct Trigger {
    /// its element's name hash: the class (h("router-trigger") ...)
    pub tag: u32,
    /// its own name (what targets, scripts and other triggers call it)
    pub name: u32,
    /// `enabled` (+0x3d bit 0)
    pub enabled: bool,
    /// h_137926ff, h_1eec0e75, h_f657f799, h_f090ebbb (see their H_ constants)
    pub disable_after_entry: bool,
    pub disable_after_event: bool,
    pub die_after_event: bool,
    pub dead: bool,
    /// `iD` (+0x3c): what its events carry as their sender (+8), matched by entries' match-id
    pub id: i64,
    pub entries: Vec<Entry>,
    /// h_e91cf6a8: the triggers it sends to
    pub targets: Vec<u32>,
    /// h_0b6b92ba: its objects (a router's buttons, an anim-trigger's door, a calculator's spawn
    /// triggers whose copies it counts, a destroyer's ...)
    pub objects: Vec<u32>,
    /// its `<transform>` (detectors, spawn-triggers, sound-triggers ...)
    pub transform: Option<Mat4>,
    pub class: Class,
    /// the element itself, for the classes later steps run (their own attributes)
    pub element: Element,
}

/// An element's text as numbers (a `<transform>`, a `<size>`), none without one.
fn floats(e: Option<&Element>) -> Vec<f32> {
    e.and_then(|e| e.text.as_ref()).map(|t| t.floats()).unwrap_or_default()
}

/// A true / false attribute (true as any non-zero number); false when it isn't given.
fn flag(e: &Element, k: u32) -> bool {
    e.attr(k).and_then(|v| v.as_i64()).is_some_and(|v| v != 0)
}

/// A whole-number attribute, `or` when it isn't given.
fn int(e: &Element, k: u32, or: i64) -> i64 {
    e.attr(k).and_then(|v| v.as_i64()).unwrap_or(or)
}

/// A number attribute, `or` when it isn't given.
fn float(e: &Element, k: u32, or: f32) -> f32 {
    e.attr(k).and_then(|v| v.as_f32()).unwrap_or(or)
}

/// A name attribute's hash, 0 when it's empty or not given.
fn name_of(e: &Element, k: u32) -> u32 {
    e.attr(k).and_then(|v| v.as_hash()).filter(|&n| n != h("")).unwrap_or(0)
}

/// The names in a list attribute (`objects="h_.. h_.."`, `object-type="tex"`): one List value or
/// the attribute repeated.
fn names(e: &Element, k: u32) -> Vec<u32> {
    e.attrs.iter().filter(|(n, _)| *n == k).flat_map(|(_, v)| match v {
        Value::List(v) => v.iter().filter_map(|x| x.as_hash()).collect(),
        v => v.as_hash().into_iter().collect::<Vec<_>>(),
    }).filter(|&x| x != 0 && x != h("")).collect()
}

/// A trigger's object list (`<h_e91cf6a8 objects=..>`, `<h_0b6b92ba objects=..>`).
pub fn object_list(e: &Element, list: u32) -> Vec<u32> {
    e.child(list).map(|l| names(l, h("objects"))).unwrap_or_default()
}

/// A level <transform>: see `level::transform`.
fn transform_of(e: &Element) -> Option<Mat4> {
    super::level::transform(&floats(e.child(h("transform"))))
}

/// The entries of a trigger element, in order: each `<base>` with its container's attributes.
fn entries(e: &Element) -> Vec<Entry> {
    let Some(list) = e.child(H_ENTRIES) else { return vec![] };
    list.children.iter().filter_map(|c| {
        let (base, extra) = if c.name == h("base") { (c, vec![]) } else { (c.child(h("base"))?, c.attrs.clone()) };
        Some(Entry {
            kind: int(base, h("Type"), 0),
            signal: int(base, h("signal"), -1),
            action: int(base, h("action"), act::OPERATE),
            match_id: int(base, h("match-id"), 0),
            disable_when_spent: flag(base, H_DISABLE_WHEN_SPENT),
            limit: int(base, H_LIMIT, i64::from(i32::MAX)),
            used: int(base, H_USED, 0),
            extra,
        })
    }).collect()
}

/// A detector's `<filter>` (see `Filter`); without one it admits anything, any team.
fn filter(e: &Element) -> Filter {
    let Some(f) = e.child(h("filter")) else { return Filter { alignment: -1, ..Default::default() } };
    Filter {
        squad_types: flag(f, H_SQUAD_TYPES),
        other_types: flag(f, H_OTHER_TYPES),
        characters: flag(f, H_ANY_CHARACTER),
        human: flag(f, H_HUMAN),
        invert: flag(f, H_INVERT),
        alignment: int(f, h("alignment"), -1),
        objects: names(f, h("object")),
        object_types: names(f, h("object-type")),
    }
}

/// The class-specific attributes of a trigger element (see `Class`).
fn class(e: &Element) -> Class {
    let tag = e.name;
    if tag == h("router-trigger") {
        Class::Router { filters: flag(e, H_ROUTER_FILTERS), out_signal: int(e, h("out-signal"), -1) }
    } else if tag == h("sound-trigger") {
        // (the constructor 0x1fce50: volume, priority and both distances -1, the rest 0)
        Class::Sound(SoundTrigger {
            sound: name_of(e, h("sound-object")),
            looping: flag(e, h("enable-looping")),
            kind: int(e, h("Type"), 0),
            hardpoint: name_of(e, h("HARDPOINT")),
            near: flag(e, H_SOUND_NEAR),
            hush: flag(e, H_SOUND_HUSH),
            fade_in: float(e, H_SOUND_FADE_IN, 0.0),
            fade_out: float(e, H_SOUND_FADE_OUT, 0.0),
            max_distance: float(e, h("max-distance"), -1.0),
            min_distance: float(e, H_SOUND_MIN_DISTANCE, -1.0),
            volume: float(e, h("volume"), -1.0),
            priority: int(e, h("priority"), -1),
        })
    } else if tag == h("dialog-trigger") {
        Class::Dialog { dialog: name_of(e, h("dialog")), hush: e.attr(H_SOUND_HUSH).and_then(|v| v.as_i64()).is_none_or(|v| v != 0) }
    } else if tag == h("timer-trigger") {
        Class::Timer { running: flag(e, h("enable-timer")), looping: flag(e, h("enable-looping")), stops: flag(e, H_TIMER_STOPS),
                       initial: float(e, h("initial-value"), 0.0), current: float(e, h("current-value"), 0.0) }
    } else if tag == h("calculator-trigger") {
        Class::Calculator { counter: int(e, h("counter"), 0), target: int(e, h("target"), 0), test: int(e, H_CALC_TEST, 0) }
    } else if tag == h("spawn-trigger") {
        // (the constructor's spawn fade, +0xdc = 2.0 when the attribute isn't given: 0x1fd6e7)
        Class::Spawn { template: name_of(e, H_TEMPLATE), copy_name: name_of(e, H_COPY_NAME), face_point: name_of(e, H_FACE_POINT),
                       fade: float(e, H_SPAWN_FADE, 2.0), delay: float(e, H_SPAWN_DELAY, 0.0), rearm: flag(e, H_REARM),
                       member_check: e.attr(H_MEMBER_CHECK).and_then(|v| v.as_i64()).is_none_or(|v| v != 0) }
    } else if tag == h("plate-detector") || tag == h("proximity-detector") {
        let volume = if tag == h("plate-detector") {
            let s = floats(e.child(h("size")));
            Volume::Plate(if s.len() >= 3 { Vec3::new(s[0], s[1], s[2]) } else { Vec3::ONE })
        } else {
            Volume::Sphere(float(e, h("radius"), 1.0))
        };
        Class::Detector { volume, filter: filter(e), keep_when_disabled: flag(e, H_KEEP_WHEN_DISABLED), phantom: name_of(e, H_PHANTOM) }
    } else if tag == h("anim-trigger") {
        Class::Anim
    } else if tag == h("destroyer-trigger") {
        Class::Destroyer
    } else if tag == h("gamevar-trigger") {
        let read = |k: u32| e.child(k).map(|o| Operand {
            kind: int(o, H_OPERAND_KIND, operand::GAMEFLAG),
            invert: flag(o, H_OPERAND_INVERT),
            path: names(o, h("gameflag")),
            constant: float(o, h("constant"), 0.0),
        }).unwrap_or_default();
        Class::Gamevar { op: int(e, H_GAMEVAR_OP, op::EQUAL), operands: H_OPERANDS.map(read) }
    } else if tag == h("score-trigger") {
        // (the constructor 0x20bb30: +0x40 0, +0x44 the null name, +0x48 |= 7: every flag on
        // when the data doesn't say)
        let on = |k: u32| e.attr(k).and_then(|v| v.as_i64()).is_none_or(|v| v != 0);
        Class::Score { amount: float(e, h("amount"), 0.0), key: name_of(e, H_SCORE_KEY), keyed: on(H_SCORE_KEYED), flag0: on(H_SCORE_FLAG0),
                       scaled: on(H_SCORE_SCALED) }
    } else {
        Class::Other
    }
}

/// Every trigger of a level file's elements (`all`, its walk), in the file's order.
pub fn read(all: &[&Element]) -> Vec<Trigger> {
    all.iter().filter(|e| e.attr(H_DISABLE_AFTER_ENTRY).is_some()).map(|e| Trigger {
        tag: e.name,
        name: name_of(e, h("name")),
        enabled: flag(e, h("enabled")),
        disable_after_entry: flag(e, H_DISABLE_AFTER_ENTRY),
        disable_after_event: flag(e, H_DISABLE_AFTER_EVENT),
        die_after_event: flag(e, H_DIE_AFTER_EVENT),
        dead: flag(e, H_DEAD),
        id: int(e, h("iD"), 0),
        entries: entries(e),
        targets: object_list(e, H_TARGETS),
        objects: object_list(e, H_OBJECTS),
        transform: transform_of(e),
        class: class(e),
        element: (*e).clone(),
    }).collect()
}

/// A short name of a trigger class for logs: the element's name without "-trigger", or what the
/// unnamed classes do (leads §5).
pub fn class_name(tag: u32) -> String {
    /// The class elements named in the data (their names hash to the tags).
    const NAMED: [&str; 22] = ["router-trigger", "timer-trigger", "calculator-trigger", "spawn-trigger", "plate-detector", "proximity-detector",
        "anim-trigger", "destroyer-trigger", "sound-trigger", "dialog-trigger", "script-trigger", "letterbox-trigger", "warp-trigger",
        "radar-trigger", "menu-trigger", "gamevar-trigger", "score-trigger", "evaluate-trigger", "event-trigger", "camera-trigger",
        "effect-trigger", "ai-trigger"];
    if let Some(n) = NAMED.iter().find(|n| h(n) == tag) {
        return n.trim_end_matches("-trigger").to_string();
    }
    match tag {
        0xE9B3_AE8E => "message".into(),
        0xE2B9_C3A2 => "input".into(),
        0x10EC_B459 => "waypoint".into(),
        0x1F07_BE84 => "level-over".into(),
        0x14CC_CFAA => "property".into(),
        0x1308_18C0 => "damage-plate".into(),
        t => format!("h_{t:08x}"),
    }
}

/// Which step of #28 runs a class the runtime doesn't run yet (plan.md §2; the steps' tickets
/// #222-#228), for the log; None for the classes it runs (the gamevar and score triggers since
/// step 2, #221; the sound- and dialog-triggers since step 4, #223), and for those that do
/// nothing a player sees (waypoint, evaluate: their base actions only).
pub fn later_step(tag: u32) -> Option<&'static str> {
    let is = |n: &str| h(n) == tag;
    Some(if is("script-trigger") || is("letterbox-trigger") || is("warp-trigger") {
        "step 3, #222"
    } else if is("menu-trigger") || is("event-trigger") || tag == 0xE9B3_AE8E || tag == 0xE2B9_C3A2 {
        "step 5, #224"
    } else if tag == 0x1F07_BE84 {
        "step 7, #226"
    } else if is("radar-trigger") || tag == 0x1308_18C0 {
        "step 9, #228"
    } else if tag == 0x14CC_CFAA {
        "a later step (object properties)"
    } else {
        return None;
    })
}
