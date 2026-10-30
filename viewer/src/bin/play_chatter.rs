//! The chatter gate and the world's one chatter queue (#155). Every voice line a character says
//! goes through one gate, FUN_00140a60, into the world's chatter manager (world+0xbe8, made by
//! FUN_00140870): default.xbe calls the gate from all 50 of its line call sites (the analyst's
//! todo/36-squad-voice/leads.md §1, §3 and todo/35-special-skills/answer-120-chatter.md §1.3-1.5).
//! Each line's values are its block's in the speaker's chatter file (`ChatterBlock`,
//! bf/character.rs). The functions are in viewer/docs/code-map.md ("Sound"); the page is
//! viewer/docs/chatter.md.
//!
//!  - **Asking** (`say`, `hurt_line`, `death`): the line's caller asks it of the speaker
//!    (`Player::asks`), and `update` runs the gate for each, in the order asked, later in the same
//!    frame. The game calls the gate on the spot, but what it decides only takes effect at the
//!    manager's update (FUN_00140d70), which `update` runs right after the asks.
//!  - **The gate** (`Chatter::gate`), the checks in the code's order: a timed mute running
//!    (mgr+0x24); the world's cutscene bit (+0xc50 bit 4); the speaker not in the world (+0x26
//!    bit 3); its replay delay for the tag (char+0x7f4, FUN_001263e0); no block for the tag in its
//!    chatter set (type+0x1f4, FUN_001353c0); beyond 45 m of the player's character, with
//!    h_19be8951 (FUN_001409a0); the condition (FUN_001372c0: the value of the block's kind,
//!    FUN_00137360, in a variant's range); the roll (`probability` %, FUN_001374c0); the priority,
//!    a step less beyond 15 m with h_f1cb4e63 (FUN_001409c0), 0 refused; the wait behind the
//!    playing lines (FUN_00141020, up to h_183b88ed); a waiting line of its priority or more
//!    (FUN_00140a40). Then the waiting line is dropped (FUN_00141000), the line built
//!    (FUN_001400d0) and played now (FUN_00140cd0: the replay delay starts, h_eecb5362 cuts off
//!    the playing lines of its priority or lower, FUN_00140f80) or left waiting (mgr+4, +8).
//!  - **The lines the demo says**: the hurt lines by FUN_00125ad0's table (`hurt_tag`), the death
//!    cry and the killer's line (FUN_00125f40, `Ask::Death`), the follow line at a hand-over
//!    (FUN_0005cc20, play.rs `squad_control`), the Move To arrival's (FUN_00045fa0,
//!    play_orders.rs `arrive`), the medkit's (0x146f40, play_pickups.rs), the
//!    refused gun's (FUN_000a9910, play_weapon_pickups.rs), the skill lines (FUN_0010e470 /
//!    FUN_0010e3e0, play_skills.rs) and Brutus's charge hit (FUN_0013c1c0, play_vengar.rs). A
//!    type of its own (an enemy's) says only its death cry: the #129 takes' Shrikes said nothing
//!    on their hits. **But** the #36 takes 07-10 log a spawned Shrike's hurt lines through the
//!    gate (e856009f, fab0be98, 01e1dcd0, 0ae8104e), so the rule is the demo's until #328
//!    settles which enemies say them (the enemies' other lines are #110's and #128's).
//!  - **A line** (`Line`) is the events of its parts (a sound, a response), each with its start
//!    and its duration, and the running total where the next part starts (line+0x28). Two seams
//!    for the next tickets: `Line::add_part` is FUN_001400d0, which FUN_00140060 calls to join a
//!    second speaker's block onto a line already made, with no roll, replay delay or distance
//!    check (the order lines, #320); and the response event, made with the line (FUN_001406f0)
//!    and fired when its speaker's sound ends (FUN_0013fbf0: `Chatter::respond`), appends the
//!    responder's block to the same line (the replies, #322: only the death cries' responders,
//!    the nearest teammate, are built here).
//!  - **The manager's update** (`Chatter::step`, FUN_00140d70): the timed mutes run down; each
//!    playing line's events count down their start, then their duration (FUN_0013f960); a line
//!    whose last event has ended is dropped; the waiting line plays once its wait is below 0. The
//!    cutscene bit stops every line and drops the waiting one.
//!  - **The voice** (`Chatter::voice_start`, `Chatter::voice_stop`): one per character (its face
//!    controller's, char+0x5b8). A sound starting on it stops the one it was saying (FUN_001435d0
//!    calls FUN_00143570 first), a dead speaker is silent unless its block says otherwise, and a
//!    voice on a line of higher priority skips the new sound (FUN_0013fab0). A cut-off stops the
//!    voice (the sound event's stop, 0x13fb90). `Player::speaking` is its time left (the HUD's
//!    speech icon), `Player::voice` the sound it's on. The two functions start and stop the face
//!    controller's line too (play_faces.rs `speak` / `stop`: the lip-sync, #325), which runs its
//!    own copy of the time left.
//!  - **The timed mutes** (`Chatter::hush`, FUN_00141100 / FUN_00141180): the triggers' lines
//!    that hush the chatter (h_e725a1a5: a sound-trigger's sound, the dialog queue's lines, a
//!    dialog-trigger's line, play_sound_triggers.rs) refuse every line while one runs, and the
//!    first one cuts off the lines playing (priority 10 or under).
//!  - **The draws** (`Lcg`): the roll, the variant and the response's picks take the game's LCG
//!    step, from a state of the chatter's own (0 at each map's start), not the characters' random
//!    states that drive the squad AI: so a line's roll never moves a fight, and two runs of the
//!    same test hooks match.
//!
//! Test hooks: `BF_CHATTER_LOG=1` prints each line asked (who, the tag, the check that refused it
//! or the variant said and when; the same check refusing a character's tag again within
//! QUIET_GAP s, a hazard's every frame or a burst's pellets, once), what the manager does with it
//! (a wait over, a sound started or skipped, a cut-off, a response, a mute) and the queue after
//! each; `BF_CHATTER_CHANCE=<percent>`
//! sets every line's chance (100: each one passes the roll); `BF_TEST_HURT=<character>,<amount>
//! [:fall],<s>[,<s>...][;...]` hits that squad member for `amount` at those times (the trainer's
//! `hit`, shot list A1; `:fall` a landing's record); `BF_TEST_SPEAK=<s>,<who>,<sound>[;...]` has
//! a character say a sound on its voice at a time, with no gate (`Chatter::test_speak`: the
//! lip-sync's test). `BF_SKILL_CHATTER` (play_skills.rs) and `BF_TEST_HEALED` (play_pickups.rs)
//! set their own lines' chance.

use super::*;
use bf_viewer::bf::character::{ChatterBlock, ChatterResponse, CHATTER_CONDITION_NONE};

/// The death cry (FUN_00125f40 at 0x125fb4, from the CBT_DEAD handler FUN_00115f90): the victim
/// says e1213623, or ed35ffae when its movement state (+0x1f0) is 4, the state the end of a
/// landing leaves (FUN_0012b740): a death from a fall (inferred; the demo's stand-in is a fall's
/// hit, `Cause::fall`). Flint has no ed35ffae block: silent. Both blocks carry `flags`
/// h_eb640008 (said though dead), probability 100, priority 9, no wait, no distance rule.
pub const DEATH: u32 = 0xE121_3623;
/// ed35ffae, the death cry of a death on landing.
pub const DEATH_FALL: u32 = 0xED35_FFAE;
/// The killer's line (FUN_00125f40, 0x126030-0x126051): a character hostile to the victim
/// (0x3ffd80) that isn't the victim, one of the two a player's (their player slots differ,
/// 0x125f82-0x125f9a), says 13601f7b when its skill's vtable +8 says so (`skills::special_kill`:
/// Tex's and Flint's while on), else 02da3fee (probability 10, replay 20 s, priority 2; its
/// variants split on the enemies it fights, "fighting": Tex's, Brutus's and Flint's all need 1 to
/// 100, which the victim makes in the death frame, `fighting`).
pub const KILL: u32 = 0x02DA_3FEE;
/// 13601f7b, the kill line of a skill's kill (and Brutus's charge hit, play_vengar.rs).
pub const KILL_SKILL: u32 = 0x1360_1F7B;
/// The hurt lines, FUN_00125ad0's table (0x125e1c-0x125ecf; the code map's row): a hit with no
/// attacker character and the record's +0x5c bit 2 (falls, liquids), not gas, ef32191d; the
/// victim's own, ff44e83f; a friend's (the attacker's +0x1b4 vtable +0xc4 on the victim's team,
/// 0x3d2c0: the same team, one not hostile to itself), e2d9fb13; else DTYPE_GAS 0feee721, then by the hit points: crossing a quarter of the max
/// downward 01e1dcd0, a half fab0be98, else a hit of 15% of the max or less e856009f and a
/// bigger one 0ae8104e. Tex's blocks (todo/36-squad-voice/leads.md §2.2: probability / replay /
/// priority / wait): ef32191d 50 / 20 / 4 / 0; ff44e83f 75 / 40 / 3 / 5; e2d9fb13 10 / 30 / 4 / 5;
/// 0feee721 100 / 20 / 8 / 5; 01e1dcd0 100 / 30 / 8 / 2.5; fab0be98 75 / 30 / 8 / 5; 0ae8104e
/// 50 / 10 / 3 / 0 and e856009f 25 / 10 / 3 / 0 (neither cuts off, waits for or is cut off).
const HURT_ENVIRONMENT: u32 = 0xEF32_191D;
/// ff44e83f: the victim's own hit (its blast, its gas).
const HURT_SELF: u32 = 0xFF44_E83F;
/// e2d9fb13: a friend's hit, friendly fire.
const HURT_FRIEND: u32 = 0xE2D9_FB13;
/// 0feee721: DTYPE_GAS, the gas cough.
const HURT_GAS: u32 = 0x0FEE_E721;
/// 01e1dcd0: the hit points crossing a quarter of the max downward, low health.
const HURT_QUARTER: u32 = 0x01E1_DCD0;
/// fab0be98: the hit points crossing half the max downward.
const HURT_HALF: u32 = 0xFAB0_BE98;
/// 0ae8104e: a hit over 15% of the max, "I'm hit".
const HURT_HIT: u32 = 0x0AE8_104E;
/// e856009f: a hit of 15% of the max or less, a pain grunt.
const HURT_GRUNT: u32 = 0xE856_009F;
/// FUN_00125ad0's shares of the max hit points: the quarter and the half a hit crosses, and the
/// most a grunt's hit takes (0.25 at 0x3a5028, 0.5, 0.15; 0x125e5a-0x125eb1).
const QUARTER: f32 = 0.25;
/// The half of the max hit points a hit crosses.
const HALF: f32 = 0.5;
/// The most of the max hit points a grunt's hit takes.
const GRUNT_SHARE: f32 = 0.15;
/// DTYPE_GAS: FUN_00125ad0's damage type 4 (the record's +4), the gas cough's.
const DTYPE_GAS: i64 = 4;

/// The condition kinds FUN_00137360 tells apart (a block's h_ff361b85): none (0; the default,
/// FUN_0019cd70); the enemies the speaker's brain is fighting (FUN_0001b460); char+0x5a8, which
/// default.xbe only ever sets to 0; the hit points in % (+0x54 / +0x50 x 100); the alert value
/// (the brain's +0x770); the ammo ratio (FUN_001265c0 x 100); the brain's mode (+0x58). Any other
/// kind takes the morale mix (FUN_001264a0 x 100).
const CONDITION_FIGHTING: u32 = 0x12B7_4A88;
/// h_e7fad8d2: char+0x5a8, always 0.
const CONDITION_COUNTER: u32 = 0xE7FA_D8D2;
/// h_e63846de: the hit points in %.
const CONDITION_HEALTH: u32 = 0xE638_46DE;
/// h_e8f64b20: the alert value (ALERT_VALUE).
const CONDITION_ALERT: u32 = 0xE8F6_4B20;
/// h_19690a85: the ammo ratio (`ammo_ratio`) x 100.
const CONDITION_AMMO: u32 = 0x1969_0A85;
/// h_e3bdaee0: the brain's mode (`mode_value`).
const CONDITION_MODE: u32 = 0xE3BD_AEE0;
/// The alert value (FUN_00137360's h_e8f64b20): a brain's +0x770 0 gives 10, 3 gives 20, no
/// brain 0. +0x770 is 0 from the brain's constructor (0x16b86) and 3 only after GOAL_PATROL's
/// spotting (FUN_0002b5b0 at 0x2b7a6), which the demo's brains don't run; and every character
/// the demo has has a brain in the game, the controlled one too (char+0x830 is set only by the
/// brain's constructor, through the character's vtable +0x164 at 0x16e9e, and cleared only by its
/// destructors, 0x111ede / 0x112e80). So 10 for everyone, and the "X is down" replies' 0..0
/// variants never fit (leads §2.4).
const ALERT_VALUE: i64 = 10;
/// FUN_00137360's value for the brain's mode (h_e3bdaee0, brain+0x58): FOLLOW (Cover Me) 30,
/// GUARD (Stand Ground) 20, WINGMAN (Fire At Will) 40, MOVE_OUT (Move To) 10, FREEWILL and
/// FORMATION 0. A squad member's mode is its `Player::command`'s (play_brain.rs `Command`, set by
/// the orders, play_orders.rs); other characters' modes aren't kept there (0). No squad block
/// uses the kind (todo/36-squad-voice leads.md §2.5).
fn mode_value(mode: brain::Mode) -> i64 {
    match mode {
        brain::Mode::Follow => 30,
        brain::Mode::Guard => 20,
        brain::Mode::Wingman => 40,
        brain::Mode::MoveOut => 10,
        brain::Mode::Freewill | brain::Mode::Formation => 0,
    }
}
/// The morale mix's weights (FUN_001264a0): the hit points 0.4 (0x3a51fc), the stamina 0.2
/// (0x3a4f90), the clip and the skill 0.1 (0x3a4ff0) each, char+0x5a8 x 0.2 0.2 (always 0); the
/// clip term without a gun in hand, and the skill's while off, 0.75 (0x3f400000).
const MORALE_HEALTH: f32 = 0.4;
/// The stamina's share's weight (0x3a4f90).
const MORALE_STAMINA: f32 = 0.2;
/// The clip term's weight (0x3a4ff0).
const MORALE_CLIP: f32 = 0.1;
/// The skill term's weight (0x3a4ff0).
const MORALE_SKILL: f32 = 0.1;
/// The clip term without a gun in hand, and the skill term while it's off (0x3f400000).
const MORALE_UNSET: f32 = 0.75;
/// FUN_001265c0: a recharging gun's clip counts five times over its size (`clip x 5 / size`),
/// and the ratio is x 0.2 (0x3a4f90), held to 0..1.
const AMMO_CLIPS: f32 = 5.0;
/// The ratio's scale (0x3a4f90).
const AMMO_SCALE: f32 = 0.2;

/// The gate's distances from the speaker to the nearest player-controlled character (0x8d660):
/// beyond FAR_STEP a line with h_f1cb4e63 loses a step of priority (FUN_001409c0, 15.0 at
/// 0x3a500c), beyond FAR_CUT one with h_19be8951 isn't said (FUN_001409a0, 45.0 at 0x3a5010).
const FAR_STEP: f32 = 15.0;
/// The cut's distance (m).
const FAR_CUT: f32 = 45.0;
/// A line's sound lasts the sound's length + this (FUN_00140590, 0.2 at 0x3a4f90).
const SOUND_TAIL: f32 = 0.2;
/// FUN_00141020 adds this to a playing line's time left (0.2 at 0x40c9d8).
const WAIT_TAIL: f32 = 0.2;
/// The gap FUN_00140a60 adds to a wait (0.3 at 0x40c9dc).
const WAIT_GAP: f32 = 0.3;
/// The top priority: FUN_00140a60 measures the wait behind every line with FUN_00141020(10), and
/// the first timed mute cuts off the lines of 10 or under (FUN_00141100 -> FUN_00140f80(10)).
const TOP_PRIORITY: i64 = 10;
/// A response event's duration (FUN_001406f0's +8 = 60.0): moot, its start fires it and it goes
/// (0x13fe50 returns 1).
const RESPONSE_SPAN: f32 = 60.0;
/// `valid_targets` h_07b939a0 (the table at 0x3c0488, 0x135e20): every character of the speaker's
/// team in the world but the speaker, the dead too, nearest first. The death cries' responders.
const NEAREST_TEAMMATES: u32 = 0x07B9_39A0;
/// BF_CHATTER_LOG's quiet time (s, the demo's): the same check refusing a character's tag again
/// within it isn't logged, so a gas cloud's, a liquid's or a damage plate's hit, which asks its
/// hurt line every frame, logs one refusal (its replay delay's) a run.
const QUIET_GAP: f32 = 0.5;

/// The chatter's random state. Each of its draws is the game's LCG step (the state at 0x46cfd8, x
/// 0x19660d + 0x3c6ef35f), the new state mod a count, as FUN_001374c0 rolls a line (mod 100, + 1
/// against the probability; no draw at 100 or more), FUN_00136fc0 picks a variant,
/// FUN_0013fde0 a response entry and FUN_001367e0 a responder's tag, and FUN_0013fbf0 rolls a
/// response (mod 100, no + 1). In the game that state is the one every random choice shares;
/// the demo's chatter keeps a copy of its own, from the .bss's 0 at each map's start (play.rs
/// `setup` puts a new `Chatter` in), so its draws don't move the squad AI's (`Player::random`)
/// or the shots' (`GameRandom`): a line's roll changes no fight, and two runs of the same test
/// hooks match.
#[derive(Clone, Copy, Debug, Default)]
struct Lcg(u32);

impl Lcg {
    /// The next state mod `n` (a draw even for 1, as the game's).
    fn draw(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
        (self.0 % n.max(1) as u32) as usize
    }
}

/// What a damaging hit's record tells FUN_00125ad0 (its +0x14 attacker, its +0x5c bit 2, its
/// damage type +4), and whether it's a fall's (the demo's stand-in for the victim's movement
/// state 4 at its death, FUN_00125f40). Each call of play.rs `hurt` names its hit's.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cause {
    /// who dealt it, if a character did (the record's +0x14 when its type is 3, OT_CHARACTER)
    pub by: Option<Who>,
    /// the record's +0x5c bit 2: a fall's (FUN_0012b740) and a liquid's (0x22171c); clear on the
    /// rest (FUN_00222f60 clears it: a damage plate's, a weapon's)
    pub environment: bool,
    /// the record's damage type (+4)
    pub damage_type: i64,
    /// a fall's hit (the death cry ed35ffae)
    pub landing: bool,
}

impl Cause {
    /// A weapon's, a grenade's, a gas cloud's or a skill's hit: its shooter and damage type
    /// (`damage::Hit::by`, `source`).
    pub fn hit(hit: &damage::Hit) -> Cause {
        Cause { by: hit.by, damage_type: hit.damage_type, ..Default::default() }
    }

    /// A landing's (FUN_0012b740): no attacker, +0x5c bit 2, DTYPE_NORMAL.
    pub fn fall(damage_type: i64) -> Cause {
        Cause { environment: true, damage_type, landing: true, ..Default::default() }
    }

    /// A liquid's burn (0x221610 -> 0x221780): no attacker, +0x5c bit 2 (0x22171c), its damage
    /// type (DTYPE_FLAME).
    pub fn liquid(damage_type: i64) -> Cause {
        Cause { environment: true, damage_type, ..Default::default() }
    }

    /// A bare record with no attacker and none of its flags (the trainer's `hit`, a damage plate's,
    /// FUN_00222f60), of `damage_type`.
    pub fn bare(damage_type: i64) -> Cause {
        Cause { damage_type, ..Default::default() }
    }
}

/// A line asked of the gate (`Player::asks`), run by `update` in the order asked.
#[derive(Clone, Copy, Debug)]
pub enum Ask {
    /// the gate for a tag its caller names; `chance` a test hook's own chance (%) for it
    Tag { tag: u32, chance: Option<i64> },
    /// a damaging hit's hurt line (FUN_00125ad0 picks the tag): its cause, the hit points before
    /// it (char+0x54 as CBT_DAMAGED comes, before FUN_002232d0 takes them down) and its amount
    /// (the record's +8)
    Hurt { cause: Cause, before: f32, amount: f32 },
    /// a death's lines (FUN_00125f40): the death cry, then the killer's line
    Death { killer: Option<Who>, landing: bool },
}

/// Ask `tag` of `u` (FUN_00140a60 from a caller that names its tag). A type of its own says no
/// line but its death cry (see the module notes).
pub fn say(u: &mut Player, tag: u32) {
    say_with(u, tag, None);
}

/// `say` with a test hook's chance for it (% in place of the block's `probability`).
pub fn say_with(u: &mut Player, tag: u32, chance: Option<i64>) {
    if !u.own_type() {
        u.asks.push(Ask::Tag { tag, chance });
    }
}

/// A damaging hit's hurt line (FUN_00125ad0, which every hit that reaches the hit points runs:
/// CBT_DAMAGED, posted by FUN_002232d0 when the amount isn't 0), asked before the hit points go
/// down. Not by a type of its own (see the module notes).
pub fn hurt_line(u: &mut Player, cause: Cause, amount: f32) {
    if amount > 0.0 && !u.own_type() {
        u.asks.push(Ask::Hurt { cause, before: u.health, amount });
    }
}

/// A death's lines (FUN_00125f40 from the CBT_DEAD handler): `u`'s death cry, then its killer's
/// line.
pub fn death(u: &mut Player, cause: Cause) {
    u.asks.push(Ask::Death { killer: cause.by, landing: cause.landing });
}

/// The world's chatter manager (world+0xbe8; FUN_00140870 makes it, 0x34 bytes): one per map
/// (play.rs `setup` puts a new one in).
#[derive(Resource, Default)]
pub struct Chatter {
    /// the lines playing (mgr+0xc's list: head +0x10, count +0x14)
    playing: Vec<Line>,
    /// the one waiting line and its wait (s; mgr+4, mgr+8)
    waiting: Option<(Line, f32)>,
    /// the timed mutes: key and seconds left (mgr+0x1c's list, count +0x24)
    mutes: Vec<(u32, f32)>,
    /// the mutes asked for since the last update (`hush`, `unhush`), applied by it
    hushes: Vec<(u32, Option<f32>)>,
    /// the variants said, by chatter set and tag (mgr+0x28: FUN_001412c0 marks one as its sound
    /// starts, FUN_001411e0 tests, FUN_00141580 clears a tag's)
    said: HashMap<(String, u32), Vec<u32>>,
    /// lines made so far (their numbers in the log)
    made: u32,
    /// the voices to start and stop at the end of the update
    acts: Vec<VoiceAct>,
    /// the game's time since the map started (s), for the log: the controlled character's clock
    /// stops at its death
    clock: f32,
    /// the draws' random state (`Lcg`)
    rng: Lcg,
    /// BF_CHATTER_LOG: when each check (its number in `gate`) last refused each speaker's tag, to
    /// log a run of the same refusal once (QUIET_GAP)
    quiet: HashMap<(Who, u32, u8), f32>,
}

/// A chatter line (0x50 bytes, FUN_0013feb0): its parts' events and its state.
#[derive(Clone, Debug)]
struct Line {
    /// its number, for the log
    n: u32,
    /// line+8: its priority (the first part's after the 15 m step; a response's part sets it
    /// again, FUN_001409e0)
    priority: i64,
    /// line+0xc bit 2 (the record's h_1800c848): it makes no new line wait (FUN_00141020)
    nobody_waits: bool,
    /// line+0xc bit 3 (h_f4744ab4): it can't be cut off (FUN_00140f80)
    uncuttable: bool,
    /// its first part's speaker and tag, and its block's replay delay and h_eecb5362 (line+0x4c,
    /// the first record: what FUN_00140cd0 reads as it plays)
    speaker: Who,
    tag: u32,
    replay_delay: f32,
    cuts_off: bool,
    /// its events (line+0x24's list)
    events: Vec<Event>,
    /// line+0x28: the running total of its parts' sounds, where the next part starts; it runs
    /// down with the time, to 0 (FUN_0013f800)
    total: f32,
}

/// One of a line's events (the base 0x39c308; the sound's vtable 0x39c33c, the response's
/// 0x39c324): its start, then its duration, counted down by FUN_0013f960.
#[derive(Clone, Debug)]
struct Event {
    /// +4: seconds before it starts (the line's running total as it was added, FUN_0013f7b0)
    start: f32,
    /// +8: seconds it lasts once started
    duration: f32,
    /// +0x1c: started
    started: bool,
    /// +0x10: whose part it is
    speaker: Who,
    /// its block's chatter set and tag (the history's key: the record's +0x60, +0x5c)
    set: String,
    tag: u32,
    /// said though the speaker is dead (the block's h_eb640008: the sound's +0x28, the response's
    /// +0x44 bit 0)
    dead_speaks: bool,
    kind: EventKind,
}

/// What an event does as it starts.
#[derive(Clone, Debug)]
enum EventKind {
    /// a sound, the variant picked (+0x24): FUN_0013fab0 plays it on the speaker's voice
    Sound(u32),
    /// a response (FUN_001406f0): the block's `<response>` entries whose range held the speaker's
    /// value; 0x13fe50 fires one through FUN_0013fbf0
    Response(Vec<ChatterResponse>),
}

/// A voice to start or stop at the end of the update: its speaker's, a sound and how loud.
#[derive(Clone, Copy, Debug)]
enum VoiceAct {
    Start { who: Who, id: u32, volume: f32 },
    Stop { who: Who },
}

/// A character's voice playing a chatter sound (an entity with its AudioPlayer): whose, so the
/// next sound on that voice or a cut-off stops it.
#[derive(Component)]
struct ChatterVoice(Who);

/// What the gate and the manager read of the world this frame.
struct Ctx<'a> {
    game: &'a Game,
    /// who can see whom (play.rs `sight_list`: the player, then the squad), for "fighting"
    sight: &'a [Seen],
    /// the player's character's place: the nearest player-controlled character to anyone
    /// (0x8d660, one player)
    here: Vec3,
    /// the squad's ammo (the ammo ratio)
    reserve: &'a ammo::Reserve,
    /// the clock, for the log (`Chatter::clock`)
    now: f32,
    /// the frame's time (s), the characters' step's too: how long before now, in a face
    /// controller's clock, a line starting here began in the game (`voice_start`)
    dt: f32,
    /// BF_CHATTER_CHANCE
    chance: Option<i64>,
    /// BF_CHATTER_LOG
    log: bool,
}

/// Whether FUN_00125ad0 counts the hit's attacker as none, the victim itself, a friend or a foe.
#[derive(Clone, Copy, Debug, PartialEq)]
enum By {
    Nothing,
    Itself,
    Friend,
    Foe,
}

impl By {
    /// What it is, for the log.
    fn label(self) -> &'static str {
        match self {
            By::Nothing => "no attacker",
            By::Itself => "its own",
            By::Friend => "a friend's",
            By::Foe => "a foe's",
        }
    }
}

/// The chatter's frame (`update`): the dialog-triggers' lines go on their speakers' faces after it
/// (play_sound_triggers.rs `speakers`), as the game's triggers run after its manager.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatterSystems;

/// The chatter manager each map (reset by play.rs `setup`); the test hook's hits, then the asks
/// and the manager's update after the movement and the skills (whose lines it runs in the same
/// frame), before the frame's sounds.
pub fn plugin(app: &mut App) {
    app.init_resource::<Chatter>()
        .add_systems(Update, (
            test_hurt.after(update_player),
            update.in_set(ChatterSystems).after(test_hurt).after(skills::update_skills).before(play_sounds),
        ).run_if(in_state(AppState::Playing)));
}

/// The chatter's frame: the mutes the triggers asked for, the replay delays and the voices' time
/// running down (FUN_0011c100, the face controller's tick FUN_001437d0), each character's asks
/// through the gate (the player's, then the squad's), then the manager's update (FUN_00140d70),
/// then the voices it started and stopped. BF_SOUND_LOG prints each voice started.
#[allow(clippy::too_many_arguments)]
fn update(mut commands: Commands, time: Res<Time>, mut chatter: ResMut<Chatter>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
          game: Res<GameData>, reserve: Res<ammo::Reserve>, mut cache: ResMut<SoundCache>, mut sources: ResMut<Assets<AudioSource>>,
          voices: Query<(Entity, &ChatterVoice)>, mut speaks: Local<Option<Vec<TestSpeak>>>) {
    let dt = frame_dt(&time);
    chatter.clock += dt;
    let sight = sight_list(std::iter::once(&*player).chain(squad.0.iter()));
    let ctx = Ctx {
        game: &game.0,
        sight: &sight,
        here: player.position,
        reserve: &reserve,
        now: chatter.clock,
        dt,
        chance: std::env::var("BF_CHATTER_CHANCE").ok().and_then(|v| v.parse::<i64>().ok()),
        log: std::env::var("BF_CHATTER_LOG").is_ok(),
    };
    let chatter = &mut *chatter;
    let mut all: Vec<&mut Player> = std::iter::once(&mut *player).chain(squad.0.iter_mut()).collect();
    chatter.apply_hushes(&mut all, &ctx);
    for u in all.iter_mut() {
        // the replay delays (FUN_0011c100, from the character update)
        u.replay.retain(|_, left| {
            *left -= dt;
            *left > 0.0
        });
        voice_tick(u, dt);
    }
    for i in 0..all.len() {
        for ask in std::mem::take(&mut all[i].asks) {
            chatter.ask(&mut all, i, ask, &ctx);
        }
    }
    chatter.step(&mut all, dt, &ctx);
    chatter.test_speak(&mut all, speaks.get_or_insert_with(test_speaks), &ctx);
    // the voices
    if chatter.acts.is_empty() {
        return;
    }
    let names: HashMap<Who, String> = all.iter().map(|u| (u.who(), unit_name(u))).collect();
    let (mute, log) = (std::env::var("BF_MUTE").is_ok(), std::env::var("BF_SOUND_LOG").is_ok());
    let mut started: Vec<(Who, Entity)> = vec![];
    for act in std::mem::take(&mut chatter.acts) {
        let who = match act {
            VoiceAct::Start { who, .. } | VoiceAct::Stop { who } => who,
        };
        // (a voice says one sound at a time: whatever it was saying stops, FUN_00143570)
        for (e, v) in &voices {
            if v.0 == who {
                commands.entity(e).try_despawn();
            }
        }
        for (_, e) in started.iter().filter(|(w, _)| *w == who) {
            commands.entity(*e).try_despawn();
        }
        started.retain(|(w, _)| *w != who);
        let VoiceAct::Start { id, volume, .. } = act else { continue };
        if log {
            println!("t={:6.2} sound {id:08x} vol {volume:.2} (the voice of {}: chatter)", ctx.now, names.get(&who).map_or("?", String::as_str));
        }
        if mute {
            continue;
        }
        let handle = cache.0.entry(id).or_insert_with(|| {
            game.0.sounds.wav(id).map(|w| sources.add(AudioSource { bytes: Arc::from(w.into_boxed_slice()) }))
        });
        match handle.clone() {
            Some(h) => {
                let e = commands.spawn((AudioPlayer::new(h), PlaybackSettings { volume: Volume::Linear(volume), ..PlaybackSettings::DESPAWN }, ChatterVoice(who))).id();
                started.push((who, e));
            }
            None => if log {
                println!("  sound {id:08x}: no sample data in the loaded banks");
            }
        }
    }
}

/// One BF_TEST_SPEAK entry: at `at` s (the controlled character's clock), `who` says `sound`.
struct TestSpeak {
    at: f32,
    who: String,
    sound: u32,
}

/// BF_TEST_SPEAK's entries (`<s>,<who>,<sound>`, `;` between them; the sound in hex, `h_` or not);
/// one that doesn't parse is skipped with a warning.
fn test_speaks() -> Vec<TestSpeak> {
    let Ok(v) = std::env::var("BF_TEST_SPEAK") else { return vec![] };
    v.split(';').filter(|e| !e.trim().is_empty()).filter_map(|e| {
        let f: Vec<&str> = e.split(',').map(str::trim).collect();
        let hook = (|| Some(TestSpeak { at: f.first()?.parse().ok()?, who: f.get(1)?.to_lowercase(),
                                         sound: u32::from_str_radix(f.get(2)?.trim_start_matches("h_").trim_start_matches("0x"), 16).ok()? }))();
        if hook.is_none() {
            warn!("BF_TEST_SPEAK: {e:?} isn't <s>,<who>,<sound id>; skipped");
        }
        hook
    }).collect()
}

/// The voice's time (the face controller's tick, FUN_001437d0: its time left +0x50 runs down and
/// it stops speaking at 0 or under); while it isn't speaking the line priority is 0 (char+0x81c,
/// 0x11cc37-0x11cc6c). This is the chatter's copy (the HUD's speech icon, the priority test of
/// `start_sound`); the face controller runs its own in its tick (play_faces.rs `Face::tick`),
/// where its line ends.
fn voice_tick(u: &mut Player, dt: f32) {
    if u.voice.is_some() {
        u.speaking -= dt;
        if u.speaking > 0.0 {
            return;
        }
        u.voice = None;
    }
    u.speaking = 0.0;
    u.voice_priority = 0;
}

/// Whether `u` is in the world (the gate's +0x26 bit 3): not a squad member its spawn-trigger
/// hasn't brought (`Player::absent`), nor the map editor's stand-in.
fn in_world(u: &Player) -> bool {
    !u.absent && !u.stand_in
}

/// `u`'s chatter set (the type's +0x1f4 map): a squad type's file by its name (`tex`), a type of
/// its own's `chatter-library` (play_enemies.rs: `shrike_light`). None without one.
fn set_of(u: &Player, game: &Game) -> Option<String> {
    if u.own_type() {
        game.character_type(u.kind).map(|t| t.chatter.trim_end_matches("_chatter").to_string()).filter(|s| !s.is_empty())
    } else {
        Some(u.data_name().to_string())
    }
}

/// The place in `all` (the player, then the squad) of the character `who`.
fn find(all: &[&mut Player], who: Who) -> Option<usize> {
    all.iter().position(|u| u.who() == who)
}

/// The distance from `all[i]` to the nearest player-controlled character (0x8d660): 0 for the
/// player's own.
fn distance(all: &[&mut Player], i: usize, ctx: &Ctx) -> f32 {
    if i == 0 { 0.0 } else { all[i].position.distance(ctx.here) }
}

/// A line's priority after the speaker's distance: a step less beyond FAR_STEP with h_f1cb4e63
/// (FUN_001409c0).
fn stepped(block: &ChatterBlock, v: f32) -> i64 {
    block.priority - (block.far_step && v > FAR_STEP) as i64
}

/// The condition value of `kind` for `u`, unit `me` of `ctx.sight` (FUN_00137360; 0 for one not
/// in the world); `held` a kill's victim, still among the killer's contacts (`fighting`).
fn condition(u: &Player, me: usize, kind: u32, held: Option<Who>, ctx: &Ctx) -> i64 {
    if !in_world(u) {
        return 0;
    }
    match kind {
        CHATTER_CONDITION_NONE => 0,
        CONDITION_FIGHTING => fighting(u, me, held, ctx),
        // char+0x5a8, which only ever gets 0 (answer-120-chatter §2)
        CONDITION_COUNTER => 0,
        CONDITION_HEALTH => if u.max_health > 0.0 { (u.health / u.max_health * 100.0) as i64 } else { 0 },
        CONDITION_ALERT => ALERT_VALUE,
        CONDITION_AMMO => (ammo_ratio(u, ctx.reserve) * 100.0) as i64,
        CONDITION_MODE => if u.in_squad() { mode_value(u.command.mode) } else { 0 },
        _ => (morale(u) * 100.0) as i64,
    }
}

/// FUN_0001b460's count, the enemies `u`'s brain fights, for the "fighting" condition: the
/// entries of its contact map (brain+0x358) of kind 1, an enemy character, with bit 2 (set once
/// by the scan, FUN_00043000 -> FUN_00054f90: the contact within 15 m, else a test of the two's
/// states, FUN_00054e00 against FUN_00054ec0, not read).
/// The controlled character's brain keeps its contacts too: the world's brain loop (FUN_000d9770)
/// runs a brain that's off through its vtable +8 (0x1a220), which for a squad type (brain+0x3c4
/// bit 8, from type+0x1a8 bit 0) a human player controls (FUN_00112800), in brain mode 0 to 5,
/// runs the same sense step as an AI's (FUN_00044480 at 0x1a360, the scan FUN_00044140).
///
/// The demo's stand-in, **a guess**: the more of the hostiles its sensor knows (an AI's
/// contacts; the demo's controlled character's sensor knows no one) and the hostiles it sees
/// within its gun's reach (play.rs `nearest_hostile`'s scan, which leaves out the dead), and one
/// more for `held`, a kill's victim: FUN_00125f40 runs in the death frame, before the killer's
/// brain handles the body (FUN_00043470, from FUN_00044140's dead branch: where the kind-1
/// entry goes then isn't shown, inferred), so the kill still counts its victim. Counted whatever
/// the distance, and even when the killer's scan never sighted it (the entry's bit 2 isn't
/// modelled: a victim that never came within 15 m nor passed the other test, or was never a
/// contact at all, such as a grenade kill round a corner, wouldn't count in the game).
fn fighting(u: &Player, me: usize, held: Option<Who>, ctx: &Ctx) -> i64 {
    let seen = u.sensor.scan(u, me, ctx.sight).iter().filter(|&&(_, d)| d < ai_range(u)).count();
    // (the victim taken out of what the sensor knows, then counted once)
    let known = u.sensor.known() - held.is_some_and(|w| u.sensor.knows(w)) as usize;
    (seen.max(known) + held.is_some() as usize) as i64
}

/// The gun in `u`'s hand (none between a switch's drop and grab): the inventory's +0x164 of
/// object type 20, OT_WEAPON (FUN_001265c0, FUN_001264a0).
fn gun_in_hand(u: &Player) -> Option<&WeaponDef> {
    u.loaded.as_ref().and_then(|l| l.weapons.get(u.weapon)).filter(|_| u.holding).map(|w| &w.def)
}

/// FUN_001265c0, the ammo ratio of the gun in hand: a gun that doesn't recharge (the weapon data's
/// +0x14c, ammo-regen, 0) its reserve over its clip size (FUN_0022da80's count over +0xe4), one
/// that does its clip x 5 over its size; x 0.2, held to 0..1. 1 x 0.2 without a gun. (The code
/// map's row had the two the other way round.)
fn ammo_ratio(u: &Player, reserve: &ammo::Reserve) -> f32 {
    let x = gun_in_hand(u).filter(|d| d.ammo > 0).map_or(1.0, |d| {
        if d.ammo_regen == 0.0 {
            reserve.get(d.ammo_type) as f32 / d.ammo as f32
        } else {
            u.ammo.get(u.weapon).map_or(0, |a| a[0]) as f32 * AMMO_CLIPS / d.ammo as f32
        }
    });
    (x * AMMO_SCALE).clamp(0.0, 1.0)
}

/// FUN_001264a0, the morale mix: 0.4 x the hit points' share + 0.2 x the stamina's + 0.1 x the
/// clip term (the clip over its size in whole numbers, `idiv` at 0x126503: 1 full, else 0; 0.75
/// without a gun) + 0.1 x the skill term (1 on, 0.75 off) + 0.2 x char+0x5a8 x 0.2 (0).
fn morale(u: &Player) -> f32 {
    let health = if u.max_health > 0.0 { u.health / u.max_health } else { 0.0 };
    let stamina = if u.max_stamina > 0.0 { u.stamina / u.max_stamina } else { 0.0 };
    let clip = gun_in_hand(u).map_or(MORALE_UNSET, |d| {
        if d.ammo > 0 { (u.ammo.get(u.weapon).map_or(0, |a| a[0]) / d.ammo) as f32 } else { 0.0 }
    });
    let skill = if u.skill.on { 1.0 } else { MORALE_UNSET };
    health * MORALE_HEALTH + stamina * MORALE_STAMINA + clip * MORALE_CLIP + skill * MORALE_SKILL
}

/// Whom FUN_00125ad0 takes the hit on `all[i]` by `by` (its shooter) for: none (no character of
/// `all`), the victim itself, a friend or a foe. A friend is what the attacker's +0x1b4 vtable
/// +0xc4 says of the victim's team (0x125de1-0x125e10): 0x3d2c0, friends as FUN_0004b960 has
/// them, the same team and one not hostile to itself. Not merely "not hostile": one of e34's
/// Shrike Hounds is placed NEUTRAL (team 2, hostile to neither side), and its bites are a foe's.
fn attacker(all: &[&mut Player], i: usize, by: Option<Who>) -> By {
    match by.and_then(|w| find(all, w)) {
        None => By::Nothing,
        Some(k) if k == i => By::Itself,
        Some(k) if all[k].team == all[i].team && !damage::hostile(all[k].team, all[k].team) => By::Friend,
        Some(_) => By::Foe,
    }
}

/// FUN_00125ad0's hurt line for a hit (see the HURT_ constants): by whom (`By`), its cause, the hit
/// points before it, its amount and the victim's max hit points.
fn hurt_tag(by: By, cause: &Cause, before: f32, amount: f32, max: f32) -> u32 {
    let first = match by {
        By::Nothing if cause.environment && cause.damage_type != DTYPE_GAS => Some(HURT_ENVIRONMENT),
        By::Itself => Some(HURT_SELF),
        By::Friend => Some(HURT_FRIEND),
        _ => None,
    };
    if let Some(tag) = first {
        return tag;
    }
    if cause.damage_type == DTYPE_GAS {
        return HURT_GAS;
    }
    let after = before - amount;
    if !(max * QUARTER <= after || before <= max * QUARTER) {
        HURT_QUARTER
    } else if !(max * HALF <= after || before <= max * HALF) {
        HURT_HALF
    } else if amount <= max * GRUNT_SHARE {
        HURT_GRUNT
    } else {
        HURT_HIT
    }
}

/// The wait (s) a line of priority `p` that waits at most `most` would take behind the lines
/// playing (FUN_00140a60, 0x140b51-0x140c10), None refused: B = `behind(p)`, the longest time
/// left of the lines of `p` or under, D = `behind(10)` - B the longer lines' time beyond them; B +
/// 0.3 when B is over 0 and that fits, else B when it fits (0 with nothing of `p` or under
/// playing), else D + 0.3 or D by the same two tests.
fn wait(b: f32, a: f32, most: f32) -> Option<f32> {
    let d = a - b;
    if b > 0.0 && b + WAIT_GAP <= most {
        Some(b + WAIT_GAP)
    } else if b <= most {
        Some(b)
    } else if d > 0.0 && d + WAIT_GAP <= most {
        Some(d + WAIT_GAP)
    } else if d <= most {
        Some(d)
    } else {
        None
    }
}

/// A variant for a part (FUN_00136fc0 over FUN_001371f0's): a draw of `rng` (the LCG mod their
/// count) among those whose range holds `value` and that the history hasn't marked; when all that
/// fit are marked their marks are cleared (FUN_00141580) and any of them may come. None, and no
/// draw, when none fits.
fn pick(said: &mut HashMap<(String, u32), Vec<u32>>, set: &str, tag: u32, block: &ChatterBlock, value: i64, rng: &mut Lcg) -> Option<u32> {
    let fits: Vec<u32> = block.variants.iter().filter(|&&(_, lo, hi)| (lo..=hi).contains(&value)).map(|&(id, _, _)| id).collect();
    if fits.is_empty() {
        return None;
    }
    let marked = said.entry((set.to_string(), tag)).or_default();
    let mut fresh: Vec<u32> = fits.iter().copied().filter(|id| !marked.contains(id)).collect();
    if fresh.is_empty() {
        marked.clear();
        fresh = fits;
    }
    Some(fresh[rng.draw(fresh.len())])
}

impl Line {
    /// A new line of `priority` for `block`'s tag `tag`, said by `speaker` (FUN_0013feb0; the gate
    /// sets +8 and +0xc, FUN_001400d0 +0x4c).
    fn new(n: u32, priority: i64, block: &ChatterBlock, speaker: Who, tag: u32) -> Line {
        Line { n, priority, nobody_waits: block.nobody_waits, uncuttable: block.uncuttable, speaker, tag, replay_delay: block.replay_delay,
               cuts_off: block.cuts_off, events: vec![], total: 0.0 }
    }

    /// FUN_001400d0: one part, `block` said by `u` (its chatter set `set`, the tag `tag`, its
    /// condition value `value`): its sound (FUN_00140590: a variant drawn from `rng`, lasting its
    /// length + 0.2 s) and its response (FUN_001406f0: the block's `<response>`s whose range holds
    /// `value`, from the sound's end), each starting where the running total stands
    /// (FUN_0013f7b0); with `adds` the sound's time joins the total. The part's variant, None when
    /// none fits. The gate makes a line's first part with it; FUN_00140060 joins another
    /// speaker's (#320) and FUN_0013fbf0 a responder's with no roll of their own.
    #[allow(clippy::too_many_arguments)]
    fn add_part(&mut self, said: &mut HashMap<(String, u32), Vec<u32>>, rng: &mut Lcg, block: &ChatterBlock, u: &Player, set: &str, tag: u32,
                value: i64, adds: bool, game: &Game) -> Option<u32> {
        let who = u.who();
        let id = pick(said, set, tag, block, value, rng);
        let mut length = 0.0;
        if let Some(id) = id {
            let sound = game.sounds.length(id).unwrap_or(0.0);
            let duration = if sound > 0.0 { sound + SOUND_TAIL } else { sound };
            self.events.push(Event { start: self.total, duration, started: false, speaker: who, set: set.to_string(), tag, dead_speaks: block.dead_speaks,
                                     kind: EventKind::Sound(id) });
            length = duration.max(0.0);
        }
        let entries: Vec<ChatterResponse> = block.responses.iter().filter(|r| (r.min..=r.max).contains(&value)).cloned().collect();
        if !entries.is_empty() {
            self.events.push(Event { start: self.total + length, duration: RESPONSE_SPAN, started: false, speaker: who, set: set.to_string(), tag,
                                     dead_speaks: block.dead_speaks, kind: EventKind::Response(entries) });
        }
        if adds {
            self.total += length;
        }
        id
    }

    /// FUN_0013f750: its time left, the latest of its events' start + duration (0 with none).
    fn left(&self) -> f32 {
        self.events.iter().map(|e| e.start + e.duration).fold(0.0, f32::max)
    }
}

impl Chatter {
    /// FUN_00141100: a timed mute keyed `key` for `length` s, or (None) FUN_00141180: the mutes
    /// keyed `key` taken off; applied at the next update. The triggers' lines that hush the
    /// chatter (h_e725a1a5) ask it (play_sound_triggers.rs).
    pub fn hush(&mut self, key: u32, length: Option<f32>) {
        self.hushes.push((key, length));
    }

    /// The mutes asked for: one is added unless its key is on the list already (the empty id
    /// adds none), and the first cuts off the lines playing at the top priority or under
    /// (FUN_00140f80(10)); a key's are taken off.
    fn apply_hushes(&mut self, all: &mut [&mut Player], ctx: &Ctx) {
        for (key, length) in std::mem::take(&mut self.hushes) {
            match length {
                Some(length) => {
                    if key == 0 || self.mutes.iter().any(|m| m.0 == key) {
                        continue;
                    }
                    if ctx.log {
                        println!("t={:6.2} chatter: a timed mute h_{key:08x} for {length:.2} s{}", ctx.now, if self.mutes.is_empty() { ": the lines playing are cut off" } else { "" });
                    }
                    if self.mutes.is_empty() {
                        self.cut(all, TOP_PRIORITY, ctx, "a timed mute");
                    }
                    self.mutes.push((key, length));
                }
                None => {
                    if ctx.log && self.mutes.iter().any(|m| m.0 == key) {
                        println!("t={:6.2} chatter: the timed mute h_{key:08x} taken off", ctx.now);
                    }
                    self.mutes.retain(|m| m.0 != key);
                }
            }
        }
    }

    /// FUN_00141020: the longest time left (+ 0.2) of the lines playing of priority `p` or under
    /// that make lines wait; 0 with none.
    fn behind(&self, p: i64) -> f32 {
        self.playing.iter().filter(|l| l.priority <= p && !l.nobody_waits).map(|l| l.left() + WAIT_TAIL).fold(0.0, f32::max)
    }

    /// One ask of `all[i]`.
    fn ask(&mut self, all: &mut [&mut Player], i: usize, ask: Ask, ctx: &Ctx) {
        match ask {
            Ask::Tag { tag, chance } => self.gate(all, i, tag, chance, None, ctx, String::new()),
            Ask::Hurt { cause, before, amount } => {
                let by = attacker(all, i, cause.by);
                let tag = hurt_tag(by, &cause, before, amount, all[i].max_health);
                let what = format!(", a hit of {amount:.1} ({before:.0} of {:.0} before; {}{})", all[i].max_health, by.label(),
                                   if cause.environment { ", environment" } else { "" });
                self.gate(all, i, tag, None, None, ctx, what);
            }
            Ask::Death { killer, landing } => {
                // the victim's cry (0x125fb4)
                self.gate(all, i, if landing { DEATH_FALL } else { DEATH }, None, None, ctx, ", its death".into());
                // the killer's line (0x125fce-0x126051): a character, not the victim, whose player
                // slot isn't the victim's (0x125f82-0x125f9a: the victim's slot, world+0xbec
                // vtable +0xc, against the record's +0x1c, the shooter's slot as the weapon code
                // fills it: so an AI's kill of an AI, neither with a slot, draws none; #36 takes
                // 07-10, the squad's kills of the Shrike, asked none), hostile to the victim (the
                // team table, the killer's row). One player: the controlled character (`all[0]`)
                // has the slot, the killer's as it is now (the demo's stand-in for the shooter's
                // at the shot). A type of its own says none (see the module notes)
                let Some(k) = killer.and_then(|w| find(all, w)) else { return };
                if k == i || (k == 0) == (i == 0) || !damage::hostile(all[k].team, all[i].team) || all[k].own_type() {
                    return;
                }
                let tag = if skills::special_kill(all[k]) { KILL_SKILL } else { KILL };
                let victim = unit_name(all[i]);
                // (its victim still among its contacts: `fighting`)
                let held = Some(all[i].who());
                self.gate(all, k, tag, None, held, ctx, format!(", the kill of {victim}"));
            }
        }
    }

    /// The gate (FUN_00140a60) for `tag` from `all[i]`, with a test hook's `chance` and `held`, a
    /// kill's victim the condition counts (`fighting`); `what` says why, for the log. See the
    /// module notes for the checks.
    #[allow(clippy::too_many_arguments)]
    fn gate(&mut self, all: &mut [&mut Player], i: usize, tag: u32, chance: Option<i64>, held: Option<Who>, ctx: &Ctx, what: String) {
        let head = format!("t={:6.2} chatter: {} {tag:08x} ({}){what}", ctx.now, unit_name(all[i]), tag_name(tag));
        let who = all[i].who();
        // 1, 2: a timed mute, the world's cutscene bit
        if !self.mutes.is_empty() {
            let why = format!("a timed mute runs (h_{:08x}, {:.1} s left)", self.mutes[0].0, self.mutes[0].1);
            return self.refused(&head, (who, tag, 1), why, ctx);
        }
        if script::in_cutscene() {
            return self.refused(&head, (who, tag, 2), "the world's cutscene bit".into(), ctx);
        }
        // 3: in the world
        if !in_world(all[i]) {
            return self.refused(&head, (who, tag, 3), "not in the world".into(), ctx);
        }
        // 4: its replay delay for the tag
        if let Some(left) = all[i].replay.get(&tag) {
            return self.refused(&head, (who, tag, 4), format!("its replay delay ({left:.1} s left)"), ctx);
        }
        // 5: a block for the tag
        let Some(set) = set_of(all[i], ctx.game) else { return self.refused(&head, (who, tag, 5), "no chatter set".into(), ctx) };
        let Some(block) = ctx.game.chatter_blocks.get(&set).and_then(|b| b.get(&tag)) else {
            return self.refused(&head, (who, tag, 5), format!("no block in {set}_chatter"), ctx);
        };
        // 6: 45 m
        let v = distance(all, i, ctx);
        if block.far_cut && v > FAR_CUT {
            return self.refused(&head, (who, tag, 6), format!("{v:.0} m from the player"), ctx);
        }
        // 7: the condition
        let value = condition(all[i], i, block.condition, held, ctx);
        if !block.variants.iter().any(|&(_, lo, hi)| (lo..=hi).contains(&value)) {
            let why = format!("no variant for the condition value {value} (kind h_{:08x})", block.condition);
            return self.refused(&head, (who, tag, 7), why, ctx);
        }
        // 8: the roll (FUN_001374c0: none at 100 or more)
        let p = chance.or(ctx.chance).unwrap_or(block.probability);
        if p < 100 && self.rng.draw(100) as i64 + 1 > p {
            return self.refused(&head, (who, tag, 8), format!("not rolled ({p}%)"), ctx);
        }
        // 9: the priority, a step less far away; 0 refused
        let priority = stepped(block, v);
        if priority == 0 {
            return self.refused(&head, (who, tag, 9), format!("priority 0 ({v:.0} m from the player)"), ctx);
        }
        // 10: the wait behind the lines playing
        let (b, a) = (self.behind(priority), self.behind(TOP_PRIORITY));
        let Some(r) = wait(b, a, block.max_wait) else {
            let why = format!("it would wait over its {:.1} s (behind {b:.2} s, {a:.2} s in all)", block.max_wait);
            return self.refused(&head, (who, tag, 10), why, ctx);
        };
        // 11: one waiting line
        if let Some((w, _)) = &self.waiting {
            if w.priority >= priority {
                let why = format!("line #{} waits, priority {}", w.n, w.priority);
                return self.refused(&head, (who, tag, 11), why, ctx);
            }
        }
        // the waiting line goes (FUN_00141000), the line is built (FUN_001400d0)
        if let Some((w, _)) = self.waiting.take() {
            if ctx.log {
                println!("t={:6.2} chatter: the waiting line #{} ({} {:08x}) dropped for a new one", ctx.now, w.n, who_label(all, w.speaker), w.tag);
            }
        }
        self.made += 1;
        let mut line = Line::new(self.made, priority, block, who, tag);
        let id = line.add_part(&mut self.said, &mut self.rng, block, &*all[i], &set, tag, value, true, ctx.game);
        let n = line.n;
        if r > 0.0 {
            self.waiting = Some((line, r));
        } else {
            self.play_now(all, line, ctx);
        }
        if ctx.log {
            let variant = id.map_or("no variant".to_string(), |id| format!("variant {id:08x}"));
            let when = if r > 0.0 { format!("waits {r:.2} s") } else { "now".into() };
            println!("{head}: said, line #{n} {variant} {when} ({p}%, priority {priority}{}, replay {:.0} s, value {value}, {v:.0} m) [{}]",
                     if priority != block.priority { " after the 15 m step" } else { "" }, block.replay_delay, self.state());
        }
    }

    /// BF_CHATTER_LOG's line for the line `head` the gate refused, `why`, with the queue; `key` its
    /// speaker, tag and the check's number. Not again while the same check refuses the same tag of
    /// the same speaker within QUIET_GAP s of the last time (a hazard's every frame).
    fn refused(&mut self, head: &str, key: (Who, u32, u8), why: String, ctx: &Ctx) {
        if !ctx.log {
            return;
        }
        if self.quiet.insert(key, ctx.now).is_some_and(|last| ctx.now - last < QUIET_GAP) {
            return;
        }
        println!("{head}: refused, {why} [{}]", self.state());
    }

    /// FUN_00140cd0: `line` plays now: its first speaker's replay delay starts when its block's
    /// isn't 0 (FUN_00126460), with h_eecb5362 the lines playing of its priority or under are cut
    /// off (FUN_00140f80), and it joins the lines playing.
    fn play_now(&mut self, all: &mut [&mut Player], line: Line, ctx: &Ctx) {
        if line.replay_delay != 0.0 {
            if let Some(k) = find(all, line.speaker) {
                all[k].replay.insert(line.tag, line.replay_delay);
            }
        }
        if line.cuts_off {
            self.cut(all, line.priority, ctx, &format!("line #{}", line.n));
        }
        self.playing.push(line);
    }

    /// FUN_00140f80: the lines playing of priority `p` or under that can be cut off are stopped
    /// (their started events, FUN_0013f8d0) and dropped.
    fn cut(&mut self, all: &mut [&mut Player], p: i64, ctx: &Ctx, by: &str) {
        for line in std::mem::take(&mut self.playing) {
            if line.priority <= p && !line.uncuttable {
                if ctx.log {
                    println!("t={:6.2} chatter: line #{} ({} {:08x}, priority {}) cut off by {by}", ctx.now, line.n, who_label(all, line.speaker), line.tag,
                             line.priority);
                }
                self.stop_line(all, &line);
            } else {
                self.playing.push(line);
            }
        }
    }

    /// FUN_0013f8d0: a line's started events stopped: a sound's stop (0x13fb90) stops its
    /// speaker's voice if it's speaking (FUN_00143570), whatever it says.
    fn stop_line(&mut self, all: &mut [&mut Player], line: &Line) {
        for ev in line.events.iter().filter(|e| e.started && matches!(e.kind, EventKind::Sound(_))) {
            if let Some(k) = find(all, ev.speaker) {
                if all[k].speaking > 0.0 {
                    self.voice_stop(&mut *all[k]);
                }
            }
        }
    }

    /// The manager's update (FUN_00140d70): with the cutscene bit the waiting line is dropped and
    /// every line stopped; the timed mutes run down (FUN_00141090); each line's events step
    /// (FUN_0013f800) and a line with none left is dropped; the waiting line's wait runs down and
    /// it plays once below 0 (FUN_00140cd0).
    fn step(&mut self, all: &mut [&mut Player], dt: f32, ctx: &Ctx) {
        let cutscene = script::in_cutscene();
        if cutscene {
            if let Some((w, _)) = self.waiting.take() {
                if ctx.log {
                    println!("t={:6.2} chatter: the waiting line #{} dropped (the cutscene bit)", ctx.now, w.n);
                }
            }
        }
        self.mutes.retain_mut(|m| {
            m.1 -= dt;
            m.1 > 0.0
        });
        for mut line in std::mem::take(&mut self.playing) {
            if !cutscene {
                self.step_line(all, &mut line, dt, ctx);
                if !line.events.is_empty() {
                    self.playing.push(line);
                    continue;
                }
            }
            if ctx.log {
                println!("t={:6.2} chatter: line #{} ({} {:08x}) {}", ctx.now, line.n, who_label(all, line.speaker), line.tag,
                         if cutscene { "stopped by the cutscene bit" } else { "ends" });
            }
            self.stop_line(all, &line);
        }
        let due = match self.waiting.as_mut() {
            Some((_, wait)) => {
                *wait -= dt;
                *wait < 0.0
            }
            None => false,
        };
        if due {
            if let Some((line, _)) = self.waiting.take() {
                if ctx.log {
                    println!("t={:6.2} chatter: the waiting line #{} ({} {:08x}) plays", ctx.now, line.n, who_label(all, line.speaker), line.tag);
                }
                self.play_now(all, line, ctx);
            }
        }
    }

    /// FUN_0013f800: each of a line's events steps (FUN_0013f960: its start counts down, it starts
    /// at 0 and may go at once; then its duration counts down and it ends), the ones a start
    /// appends among them; then its running total runs down, to 0.
    fn step_line(&mut self, all: &mut [&mut Player], line: &mut Line, dt: f32, ctx: &Ctx) {
        let mut k = 0;
        while k < line.events.len() {
            let ev = &mut line.events[k];
            let gone = if !ev.started {
                ev.start -= dt;
                if ev.start > 0.0 {
                    false
                } else {
                    ev.started = true;
                    let ev = ev.clone();
                    self.start_event(all, line, &ev, ctx)
                }
            } else {
                // (a sound's end, 0x2c60c0, does nothing: it has played out)
                ev.duration -= dt;
                ev.duration <= 0.0
            };
            if gone {
                line.events.remove(k);
            } else {
                k += 1;
            }
        }
        line.total = (line.total - dt).max(0.0);
    }

    /// An event's start: true when it goes at once.
    fn start_event(&mut self, all: &mut [&mut Player], line: &mut Line, ev: &Event, ctx: &Ctx) -> bool {
        match &ev.kind {
            EventKind::Sound(id) => self.start_sound(all, line, ev, *id, ctx),
            EventKind::Response(entries) => {
                self.respond(all, line, ev, entries, ctx);
                true
            }
        }
    }

    /// FUN_0013fab0, a sound event's start: none for a speaker gone; a dead speaker is silent
    /// unless its block says otherwise (h_eb640008), and a voice saying a line of higher priority
    /// skips it (char+0x81c); else the variant is marked said (FUN_001412c0) and, its speaker in
    /// the world, plays on its voice at the line's priority. True when it goes (skipped).
    fn start_sound(&mut self, all: &mut [&mut Player], line: &Line, ev: &Event, id: u32, ctx: &Ctx) -> bool {
        let Some(k) = find(all, ev.speaker) else { return true };
        let skipped = if all[k].dead && !ev.dead_speaks {
            Some("its speaker is dead".to_string())
        } else if all[k].speaking > 0.0 && all[k].voice_priority > line.priority {
            Some(format!("its speaker's voice is on priority {}", all[k].voice_priority))
        } else {
            None
        };
        if let Some(why) = skipped {
            if ctx.log {
                println!("t={:6.2} chatter: line #{}'s sound {id:08x} ({} {:08x}) skipped: {why}", ctx.now, line.n, unit_name(all[k]), ev.tag);
            }
            return true;
        }
        let marked = self.said.entry((ev.set.clone(), ev.tag)).or_default();
        if !marked.contains(&id) {
            marked.push(id);
        }
        if in_world(all[k]) {
            if ctx.log {
                println!("t={:6.2} chatter: line #{}'s sound {id:08x} ({} {:08x}, {:.2} s) starts", ctx.now, line.n, unit_name(all[k]), ev.tag, ev.duration);
            }
            if std::env::var("BF_ENEMY_LOG").is_ok() && all[k].own_type() {
                println!("{}'s line {id:08x} ({:08x}) starts", unit_name(all[k]), ev.tag);
            }
            self.voice_start(&mut *all[k], id, line.priority, ctx);
        }
        false
    }

    /// The face hook: `u`'s voice starts saying sound `id` for a line of `priority` (FUN_0013fab0
    /// -> FUN_001435d0 at 0x13fb31: char+0x81c = the priority; the controller stops what it was
    /// saying, FUN_00143570, and speaks the sound for its length, +0x50). Every chatter sound
    /// starts here, so the face controller's lip-sync starts here too (play_faces.rs `speak`,
    /// #325): the game's manager update comes before the characters' ticks (0xd9ac7, 0xd9d0b) and
    /// the demo's after their step, so the line is put a frame's dt back in the controller's
    /// clock, and its mouth is read where the game's is from the next frame on. Its end is the
    /// face's own tick (the time out) or `voice_stop`'s (a cut-off); `voice_tick` runs this copy
    /// of the time out. Heard in 2D at full for the controlled character (FUN_001435d0 asks
    /// FUN_00112790), else as play.rs `play_sounds` hears a squad member's sounds, quieter with
    /// the distance (the demo's mixing: the game's is 3D at the character, FUN_00178e50).
    fn voice_start(&mut self, u: &mut Player, id: u32, priority: i64, ctx: &Ctx) {
        faces::speak(u, &ctx.game.sounds, id, ctx.dt, "chatter");
        u.voice_priority = priority;
        u.voice = Some(id);
        u.speaking = ctx.game.sounds.length(id).unwrap_or(0.0);
        let volume = if u.controlled {
            1.0
        } else {
            (1.0 - u.position.distance(ctx.here) / triggers::sounds::SPEAKER_HEARD).clamp(triggers::sounds::SPEAKER_FLOOR, 1.0) * triggers::sounds::SPEAKER_SHARE
        };
        self.acts.push(VoiceAct::Start { who: u.who(), id, volume });
    }

    /// BF_TEST_SPEAK=<s>,<who>,<sound>[;...]: at `s` seconds (the controlled character's clock)
    /// `who` (`me`, a squad name or 0-3, `all`; play_faces.rs `is_whom`) says `sound` (its hex
    /// id) on its voice at the top priority, as a chatter line's sound starts (`voice_start`: the
    /// voice, the speech icon, the face's lip-sync) but with no gate or line: the lip-sync's test.
    /// The sound must be in the map's loaded banks (its library entry has the tracks, its file the
    /// length): a level's own line, such as take07's h_ec50fb88 (the tutorial's), says for 0 s on
    /// another map and moves nothing; the common bank's squad lines (Tex's skill-on line
    /// 05410be5) are everywhere. A character not in the world is left out. BF_CHATTER_LOG logs
    /// each, with the sound's length.
    fn test_speak(&mut self, all: &mut [&mut Player], pending: &mut Vec<TestSpeak>, ctx: &Ctx) {
        if pending.is_empty() {
            return;
        }
        let now = all[0].sim_time;
        let (due, rest): (Vec<TestSpeak>, Vec<TestSpeak>) = std::mem::take(pending).into_iter().partition(|s| s.at <= now);
        *pending = rest;
        for s in due {
            for i in 0..all.len() {
                if !faces::is_whom(all[i], i == 0, &s.who) || !in_world(all[i]) {
                    continue;
                }
                if ctx.log {
                    println!("t={:6.2} chatter: BF_TEST_SPEAK: {} says {:08x} ({:.2} s)", ctx.now, unit_name(all[i]), s.sound,
                             ctx.game.sounds.length(s.sound).unwrap_or(0.0));
                }
                self.voice_start(&mut *all[i], s.sound, TOP_PRIORITY, ctx);
            }
        }
    }

    /// The face hook: `u`'s voice stops (a cut-off: the sound event's stop, 0x13fb90 ->
    /// FUN_00143570 at 0x13fbde, once its controller is speaking, +0x54 with +0x50 over 0: `stop_line`
    /// asks it of this copy, `Player::speaking`), the face controller's lip-sync with it
    /// (play_faces.rs `stop`, #325).
    fn voice_stop(&mut self, u: &mut Player) {
        faces::stop(u, "a chatter cut-off");
        u.voice = None;
        u.speaking = 0.0;
        u.voice_priority = 0;
        self.acts.push(VoiceAct::Stop { who: u.who() });
    }

    /// A response event's start (0x13fe50 -> FUN_0013fbf0), as its speaker's sound ends: nothing
    /// for a speaker gone, or dead unless its block says otherwise; else one of its entries (a
    /// draw, FUN_0013fde0: the LCG mod their count) picks its responders (FUN_001367e0) and a
    /// response tag for each (a draw: the LCG mod the entry's tags), and each responder's block
    /// for it is appended to this line (FUN_001400d0) after a roll on its `probability` (a draw:
    /// the LCG mod 100 <= it, no + 1), when no waiting line has its priority or more; no replay
    /// delay, distance rule or condition test of its own (its variant still needs one that fits).
    /// The line takes the responder's priority after its distance rules (FUN_001409e0) and the
    /// waiting line is dropped. Built for the death cries' responder, the nearest teammate
    /// (h_07b939a0); the other targets and `response_flags` are #322's.
    fn respond(&mut self, all: &mut [&mut Player], line: &mut Line, ev: &Event, entries: &[ChatterResponse], ctx: &Ctx) {
        let head = format!("t={:6.2} chatter: line #{}'s response to {} {:08x}", ctx.now, line.n, who_label(all, ev.speaker), ev.tag);
        let Some(s) = find(all, ev.speaker) else { return };
        if all[s].dead && !ev.dead_speaks {
            if ctx.log {
                println!("{head}: none, its speaker is dead");
            }
            return;
        }
        let e = &entries[self.rng.draw(entries.len())];
        if e.targets != NEAREST_TEAMMATES || e.flags != 0 {
            if ctx.log {
                println!("{head}: valid_targets h_{:08x}, response_flags h_{:08x}: not built (#322)", e.targets, e.flags);
            }
            return;
        }
        // 0x135e20: the speaker's teammates (its team, which isn't hostile to itself), the dead
        // too, nearest first; the first one only (FUN_001367e0 goes past it only with the entry's
        // +0x14 bit 0: `response_flags` h_fcc29cfe's "every responder", inferred)
        let (team, at) = (all[s].team, all[s].position);
        let nearest = all.iter().enumerate().filter(|(k, u)| *k != s && in_world(u) && u.team == team && !damage::hostile(team, team))
            .map(|(k, u)| (k, u.position.distance(at))).min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((r, _)) = nearest else {
            if ctx.log {
                println!("{head}: no teammate");
            }
            return;
        };
        if e.tags.is_empty() {
            return;
        }
        let tag = e.tags[self.rng.draw(e.tags.len())];
        let say = format!("{head}: {} {tag:08x} ({})", unit_name(all[r]), tag_name(tag));
        let block = set_of(all[r], ctx.game).filter(|_| !all[r].own_type())
            .and_then(|set| ctx.game.chatter_blocks.get(&set).and_then(|b| b.get(&tag)).map(|b| (set, b)));
        let Some((set, block)) = block else {
            if ctx.log {
                println!("{say}: no block");
            }
            return;
        };
        if let Some((w, _)) = &self.waiting {
            if w.priority >= block.priority {
                if ctx.log {
                    println!("{say}: none, line #{} waits at priority {}", w.n, w.priority);
                }
                return;
            }
        }
        let p = ctx.chance.unwrap_or(block.probability);
        let x = self.rng.draw(100) as i64;
        if x > p {
            if ctx.log {
                println!("{say}: not rolled ({x} over {p})");
            }
            return;
        }
        let value = condition(all[r], r, block.condition, None, ctx);
        let id = line.add_part(&mut self.said, &mut self.rng, block, &*all[r], &set, tag, value, e.target_flags == 0, ctx.game);
        let v = distance(all, r, ctx);
        line.priority = if block.far_cut && v > FAR_CUT { 0 } else { stepped(block, v) };
        if let Some((w, _)) = self.waiting.take() {
            if ctx.log {
                println!("t={:6.2} chatter: the waiting line #{} dropped for the response", ctx.now, w.n);
            }
        }
        if ctx.log {
            match id {
                Some(id) => println!("{say}: appended, variant {id:08x} ({p}%, the line's priority now {})", line.priority),
                None => println!("{say}: appended with no sound, no variant for the condition value {value} (kind h_{:08x})", block.condition),
            }
        }
    }

    /// The queue, for the log: the lines playing (number, speaker's tag, priority, time left) and
    /// the waiting one.
    fn state(&self) -> String {
        let playing: Vec<String> = self.playing.iter().map(|l| format!("#{} {:08x} p{} {:.2} s", l.n, l.tag, l.priority, l.left())).collect();
        let waiting = self.waiting.as_ref().map_or("none".to_string(), |(l, w)| format!("#{} {:08x} p{} in {w:.2} s", l.n, l.tag, l.priority));
        format!("playing: {}; waiting: {waiting}{}", if playing.is_empty() { "none".into() } else { playing.join(", ") },
                if self.mutes.is_empty() { String::new() } else { format!("; {} mute(s)", self.mutes.len()) })
    }
}

/// A character's name for the log by its identity (its place in `all`).
fn who_label(all: &[&mut Player], who: Who) -> String {
    find(all, who).map_or_else(|| format!("{who:?}"), |k| unit_name(all[k]))
}

/// What the tags the demo raises are, for the log (named by the situation that raises them:
/// todo/36-squad-voice/leads.md §2.2).
fn tag_name(tag: u32) -> &'static str {
    match tag {
        DEATH => "death cry",
        DEATH_FALL => "death cry, fall",
        KILL => "kill",
        KILL_SKILL => "kill / charge hit",
        HURT_ENVIRONMENT => "fall / burn cry",
        HURT_SELF => "hurt by own blast",
        HURT_FRIEND => "friendly fire",
        HURT_GAS => "gas cough",
        HURT_QUARTER => "low health",
        HURT_HALF => "half health",
        HURT_HIT => "I'm hit",
        HURT_GRUNT => "pain grunt",
        0xF415_AB02 => "follow",
        0x12C3_5D69 => "healed",
        0xF832_AD27 => "healed, alone",
        0xF2CE_8278 => "can't take that gun",
        0x1C05_14BD => "skill on",
        0x0908_7C9C => "skill off",
        0xEE0A_5CBC => "skill off, drained",
        0x0CCE_DDF8 => "Tex is down",
        0xEFC7_D9FE => "Hawk is down",
        0x0ACE_9296 => "Brutus is down",
        0xEA52_E942 => "Flint is down",
        _ => "-",
    }
}

/// Test hook: BF_TEST_HURT=<character>,<amount>[:fall],<s>[,<s>...][;...] hits that squad member
/// (by name or 0-3, the controlled one too) for `amount` hit points at each time, once: a bare hit
/// with no attacker, as the trainer's `hit` calls FUN_002232d0 (no difficulty step, no shooter;
/// the shot list's A1), or with `:fall` a landing's record (no attacker, +0x5c bit 2: the hurt
/// line ef32191d, the death cry ed35ffae), through play.rs `hurt` (its blood, a knock-down at 15 or
/// more, its hurt line).
fn test_hurt(mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut done: Local<Vec<(usize, u32)>>) {
    let Ok(v) = std::env::var("BF_TEST_HURT") else { return };
    let t = player.sim_time;
    for (n, entry) in v.split(';').enumerate() {
        let mut it = entry.split(',').map(str::trim);
        let who = it.next().and_then(|c| c.parse::<usize>().ok().or_else(|| CHARACTERS.iter().position(|x| x.eq_ignore_ascii_case(c))));
        let (amount, cause) = match it.next().map(|a| a.split_once(':').unwrap_or((a, ""))) {
            Some((a, "fall")) => (a.parse::<f32>().ok(), Cause::fall(0)),
            Some((a, _)) => (a.parse::<f32>().ok(), Cause::bare(0)),
            None => (None, Cause::bare(0)),
        };
        let (Some(c), Some(amount)) = (who, amount) else { continue };
        for at in it.filter_map(|x| x.parse::<f32>().ok()) {
            if t < at || done.contains(&(n, at.to_bits())) {
                continue;
            }
            done.push((n, at.to_bits()));
            let u = if player.character == c && player.in_squad() { Some(&mut *player) } else { squad.0.iter_mut().find(|m| m.character == c && m.in_squad()) };
            if let Some(u) = u.filter(|u| !u.dead) {
                if std::env::var("BF_CHATTER_LOG").is_ok() {
                    println!("t={t:6.2} chatter: BF_TEST_HURT hits {} for {amount}{}", unit_name(u), if cause.landing { " (a fall's)" } else { "" });
                }
                let at = u.position + Vec3::Y * (GROUND + BODY_HEIGHT * 0.5);
                hurt(u, &game.0, amount, cause, Vec3::ZERO, at, -1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FUN_00125ad0's table: the attacker first, then gas, then the hit points' quarter and half
    /// crossed, then the 15% hit.
    #[test]
    fn hurt_lines() {
        let bare = Cause::bare(0);
        assert_eq!(hurt_tag(By::Nothing, &Cause::fall(0), 100.0, 10.0, 100.0), HURT_ENVIRONMENT);
        assert_eq!(hurt_tag(By::Itself, &bare, 100.0, 10.0, 100.0), HURT_SELF);
        assert_eq!(hurt_tag(By::Friend, &bare, 100.0, 10.0, 100.0), HURT_FRIEND);
        assert_eq!(hurt_tag(By::Foe, &Cause::bare(DTYPE_GAS), 100.0, 10.0, 100.0), HURT_GAS);
        // (a gas cloud with no attacker isn't the environment's: bit 2 is for the rest)
        assert_eq!(hurt_tag(By::Nothing, &Cause::liquid(DTYPE_GAS), 100.0, 10.0, 100.0), HURT_GAS);
        assert_eq!(hurt_tag(By::Foe, &bare, 30.0, 10.0, 100.0), HURT_QUARTER);
        assert_eq!(hurt_tag(By::Foe, &bare, 55.0, 10.0, 100.0), HURT_HALF);
        assert_eq!(hurt_tag(By::Foe, &bare, 100.0, 15.0, 100.0), HURT_GRUNT);
        assert_eq!(hurt_tag(By::Foe, &bare, 100.0, 16.0, 100.0), HURT_HIT);
        // already under the quarter: no crossing
        assert_eq!(hurt_tag(By::Nothing, &bare, 20.0, 5.0, 100.0), HURT_GRUNT);
    }

    /// FUN_00140a60's wait: B + 0.3 when it fits, B, then the longer lines' D + 0.3 or D.
    #[test]
    fn waits() {
        let near = |r: Option<f32>, w: Option<f32>| match (r, w) {
            (Some(r), Some(w)) => (r - w).abs() < 1e-5,
            (r, w) => r.is_none() && w.is_none(),
        };
        assert!(near(wait(0.0, 0.0, 5.0), Some(0.0)));
        assert!(near(wait(1.0, 1.0, 5.0), Some(1.3)));
        assert!(near(wait(4.9, 4.9, 5.0), Some(4.9)));
        // a line of its priority or under with longer than the most left: it goes now (D = 0)
        // and, cutting off, stops that one as it plays
        assert!(near(wait(6.0, 6.0, 5.0), Some(0.0)));
        // ... unless a longer line of higher priority plays: then it waits for that one's excess
        assert!(near(wait(6.0, 8.0, 5.0), Some(2.3)));
        assert!(near(wait(6.0, 12.0, 5.0), None));
        // nothing of its priority or under: now, whatever higher plays
        assert!(near(wait(0.0, 1.7, 5.0), Some(0.0)));
        // a line that waits for nothing (0 s, the death cry): now unless a higher one plays
        // longer than its own kind
        assert!(near(wait(1.7, 1.7, 0.0), Some(0.0)));
        assert!(near(wait(1.7, 2.5, 0.0), None));
    }

    /// FUN_001371f0: no variant repeats before every one that fits has been said; a value no range
    /// holds gets none.
    #[test]
    fn variants() {
        let block = ChatterBlock {
            probability: 100, replay_delay: 0.0, priority: 5, condition: CHATTER_CONDITION_NONE, far_step: true, far_cut: true, cuts_off: true,
            max_wait: 2.0, nobody_waits: false, uncuttable: false, dead_speaks: false, variants: vec![(1, 0, 0), (2, 0, 100), (3, 1, 100)],
            responses: vec![],
        };
        let mut said = HashMap::new();
        let mut rng = Lcg::default();
        let mut seen = vec![];
        for _ in 0..2 {
            let id = pick(&mut said, "tex", 7, &block, 0, &mut rng).unwrap();
            said.get_mut(&("tex".to_string(), 7)).unwrap().push(id);
            seen.push(id);
        }
        seen.sort();
        assert_eq!(seen, vec![1, 2]);
        // both said: the marks go and either may come
        assert!(matches!(pick(&mut said, "tex", 7, &block, 0, &mut rng), Some(1 | 2)));
        // none fits: no variant, and no draw
        let before = rng.0;
        assert_eq!(pick(&mut said, "tex", 7, &block, 101, &mut rng), None);
        assert_eq!(rng.0, before);
    }

    /// FUN_00125ad0's attacker: a friend is one of the victim's team, a team not hostile to
    /// itself (0x3d2c0); a neutral's hit (e34's hounds, team 2) is a foe's, and so is a hit
    /// between two of PC_NO_TEAM (7, hostile to itself).
    #[test]
    fn friends() {
        let (mut tex, mut hawk, mut hound) = (Player::new(0), Player::new(1), Player::new(2));
        hound.team = 2;
        let all = [&mut tex, &mut hawk, &mut hound];
        assert_eq!(attacker(&all, 1, None), By::Nothing);
        assert_eq!(attacker(&all, 1, Some(Who::Squad(1))), By::Itself);
        assert_eq!(attacker(&all, 1, Some(Who::Squad(0))), By::Friend);
        assert_eq!(attacker(&all, 1, Some(Who::Squad(2))), By::Foe);
        let (mut a, mut b) = (Player::new(0), Player::new(1));
        (a.team, b.team) = (7, 7);
        assert_eq!(attacker(&[&mut a, &mut b], 1, Some(Who::Squad(0))), By::Foe);
    }

    /// The chatter's draws are the game's LCG from 0 (0x46cfd8's step), the new state mod the
    /// count: the first states are 0x3c6ef35f and 0x47502932, so the first roll is 1013904223 %
    /// 100 = 23 and the second 1196435762 % 100 = 62.
    #[test]
    fn draws() {
        let mut rng = Lcg::default();
        assert_eq!(rng.draw(100), 23);
        assert_eq!(rng.0, 0x3C6E_F35F);
        assert_eq!(rng.draw(100), 62);
        assert_eq!(rng.0, 0x4750_2932);
        // a draw of 1 still steps the state
        let s = rng.0;
        assert_eq!(rng.draw(1), 0);
        assert_ne!(rng.0, s);
    }
}
