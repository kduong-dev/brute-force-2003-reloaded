//! The enemy species' own behaviour (#129: the Shrike Soldier and Shrike Heavy, the first of
//! them), for a character of a type other than the squad's (`OTHER`: a test map NPC of any
//! type). The squad keeps its own rules (its root-motion locomotion and its ragdoll knock-down).
//! Sources: the spec and footage in `todo/129-shrikes/` (spec.md, notes.md), the leads in
//! `todo/110-enemy-species/leads.md` and `todo/128-enemy-types/leads.md`, and the functions cited
//! (viewer/docs/code-map.md).
//!
//!  - **Its motions** (`Motions`): the type's motion set's MOTION table, each motion type's clips
//!    by FUN_00130c50's rule (`Motion::clip`: the script, or with the gun up its ready-script; the
//!    w1_ stance, enemies holding one gun). Its gait's clips come from the table too
//!    (`table_clips`): the default motion (idle_stop) and rp_idle, walk / rp_walk, the run (the
//!    hf's rp_run: it has no relaxed one), the dash, the back steps and the walking side steps.
//!  - **Moving by its type's speeds** (FUN_0012c6c0): the motion type picks one of `speeds`
//!    (crouch walk, walk, run, dash), the ground velocity eases toward it (4.5 a second, 7 slowing
//!    down: 0x3c05b8 / 0x3c05bc), and a walking clip plays at that speed over its own (at most
//!    x1.25, 0x3c05cc). Not by root motion, which only the reactions, dodges, dives and surprise
//!    use (FUN_0012cb40's list). The demo's NPCs only move by the test hook (`Drive`): the
//!    fighting AI that moves them in the game is out of scope.
//!  - **Hit reactions** (`damaged`; the analyst's reading of the brain, todo/129-shrikes/answers.md
//!    Q1 and Q2): a non-lethal hit that reaches the hit points goes on the brain's damage record;
//!    one that brings the last 2.7 s of it over half the max hit points is EVT_DAMAGE_HEAVY (the
//!    record emptied), any other EVT_DAMAGE_LIGHT. Its personality's entry draws a goal by weight
//!    (`default`'s heavy entry: KNOCKDOWN 1 : STUN 1 : NOOP 3; an entry under priority 18 never
//!    acts in a fight). A brain that knows no enemy (the demo's dummies) gets
//!    EVT_DAMAGED_NO_ENEMIES instead, whose entry posts EVT_USER2: KNOCKDOWN 1 : STUN 1. The two
//!    goals refuse to start in their cooldowns (a refused one struck out and the draw repeated).
//!    GOAL_STUN plays MT_DIZZY (stunned / stunned2 at random), GOAL_KNOCKDOWN MT_KNOCKDOWN
//!    (knockdown / knockdown2), once, at rate 1, by their clip's root, the state lasting the clip.
//!    The knock-down is a clip, not a ragdoll (take 14).
//!  - **The stamina shield** (`absorb`): the deathmatch Shrikes' (shield-type 2, 0x223690) takes a
//!    hit from the stamina first, the rest goes to the hit points in the same hit; the stamina
//!    refills at the type's stamina-recovery a second (`refill`, FUN_0011a840 for a type with
//!    h_e78efbb2).
//!  - **Death** (answers.md Q6, Q8): the game's ragdoll, the same for every character since #259
//!    (play_ragdoll.rs: bodies for the hit parts and their ancestors, FUN_00127290, the bones
//!    below them carried rigidly, the joints' ranges FUN_00113120's); the death line starts in the
//!    death frame, none of a type's three repeated before all are said (`death_cry`, FUN_001371f0).
//!
//! Test hooks: `BF_TEST_NPC_MOTION=<s>,<n|aim|all>,<motion>[,<x>,<z>[,relaxed]]` (play_testtools.rs
//! reads it) drives an NPC in a motion type to a point; `BF_TEST_NPC`'s `frozen` behaviour stands
//! an NPC with its brain off (the takes' trainer freeze: no reactions); `BF_ENEMY_LOG=1` prints
//! the table, the draws, the motions, the drive (every half second), the shield and the death
//! line; `BF_RAGDOLL_LOG=1` the ragdoll's bodies and limits (play_ragdoll.rs).

use super::*;
use bf_viewer::bf::character_type::{CharacterType, MOTION_TYPES};
use bf_viewer::bf::personality::{EVT_DAMAGED_NO_ENEMIES, EVT_DAMAGE_HEAVY, EVT_DAMAGE_LIGHT, EVT_USER2, GOAL_KNOCKDOWN, GOAL_STUN};

/// The motion types this module names (default.xbe's MT_ name table 0x3bfdac; `MOTION_TYPES`).
pub const MT_BACKPEDAL: i64 = 3;
pub const MT_BACKSTEP: i64 = 4;
pub const MT_CROUCH: i64 = 6;
pub const MT_CROUCH_WALK: i64 = 7;
pub const MT_DASH: i64 = 8;
pub const MT_DIZZY: i64 = 14;
pub const MT_IDLE: i64 = 19;
pub const MT_KNOCKDOWN: i64 = 25;
pub const MT_MOVE_LEFT_BACK_SLOW: i64 = 28;
pub const MT_MOVE_LEFT_SLOW: i64 = 29;
pub const MT_MOVE_RIGHT_BACK_SLOW: i64 = 32;
pub const MT_MOVE_RIGHT_SLOW: i64 = 33;
pub const MT_RUN: i64 = 34;
pub const MT_WALK: i64 = 47;

/// FUN_00126a40: the running motion types (BACKPEDAL, MOVE_LEFT, _LEFT_BACK, _RIGHT, _RIGHT_BACK,
/// RUN), moved at the type's run speed (`speeds[2]`).
const RUNNING: [i64; 6] = [3, 26, 27, 30, 31, 34];
/// FUN_00126b40: the walking ones (BACKSTEP, the four MOVE_*_SLOW, WALK), at the walk speed
/// (`speeds[1]`), their clip at that speed over its own.
const WALKING: [i64; 6] = [4, 28, 29, 32, 33, 47];
/// FUN_0012cb40: the motion types that move by their clip's root, not by the speeds: DIVE,
/// DIZZY, the four dodges, KNOCKDOWN, SURPRISE (0xd-0x12, 0x19, 0x2c).
const BY_ROOT: [i64; 8] = [13, 14, 15, 16, 17, 18, 25, 44];
/// FUN_0012c6c0's ground velocity eases toward the wanted one at this rate a second (0x3c05b8),
/// or this one slowing down (0x3c05bc).
const SPEED_UP: f32 = 4.5;
const SLOW_DOWN: f32 = 7.0;
/// A walking clip plays at the walk speed over its own speed (`AnimDef::move_speed`), at most
/// this fast (0x3c05cc); a clip whose own speed is under play.rs's NO_CLIP_SPEED (0x3a4fe4)
/// plays at rate 1.
const MAX_WALK_RATE: f32 = 1.25;

/// How long after the hit the brain takes the damage event (s): the reaction is drawn then, and
/// its motion starts (`step`). A fit: the takes' take-damage row to the goal +0.28-0.38 s and to
/// the motion +0.37-0.46 s, in their column's time, which ran ahead of the game's clock after each
/// hit (single-console play steps the game by the frame's time, capped at 0.1 s: FUN_00068d20,
/// FUN_00069270; the hit log's stalls dropped the rest); the delay is the queued event waiting
/// for the attack goal to be interruptible, not readable statically. answers.md: about 0.3 s.
const REACTION_DELAY: f32 = 0.3;
/// The brain's damage record keeps the hits of the last this many seconds (brain+0x238 = 2.7,
/// 0x402ccccd, written once by the brain constructor FUN_000163f0 at 0x16719; FUN_0003da40 drops
/// the older), and a hit that brings its total over this share of the max hit points (0x3a4f74,
/// strictly more) is heavy (0x3e95f-0x3e9c1).
const DAMAGE_WINDOW: f32 = 2.7;
const HEAVY_SHARE: f32 = 0.5;
/// EVT_ENEMY_SEEN's priority in a fight (18, "constantly updated to be pri 18 if sight is kept"):
/// a damage entry below it never reaches the top of the brain's event heap while the character
/// fights (`default`'s light entry, -20.2 "TESTING"; answers.md Q1).
const FIGHT_PRIORITY: f32 = 18.0;
/// The reactions' cooldowns, [knock-down, stun] (s), set when one starts (FUN_000124f0 overwrites
/// them): a knock-down's start (goal 65, 0x40870) sets 6 and 3, a stun's (goal 63, 0x3f460) 2 and
/// 4; a goal refuses to start while its own runs (brain+0xd4, +0xe4). (They set the dive's and
/// the dodge's too, which the demo's enemies don't use.)
const KNOCKDOWN_COOLDOWNS: [f32; 2] = [6.0, 3.0];
const STUN_COOLDOWNS: [f32; 2] = [2.0, 4.0];
/// How far short of a drive's goal (m) the hook lets go, besides what the slide covers (the
/// demo's).
const ARRIVE: f32 = 0.15;
/// The chatter lines said so far, per chatter library and tag: the chatter manager's history
/// (mgr+0x28, FUN_001411e0; FUN_00136fc0 -> FUN_001371f0 picks among the lines not said yet, and
/// once only said ones are left clears it, FUN_00141580, and picks again). One per map, as the
/// manager is per world: emptied as a map starts (`new_world`). The skill lines keep the same
/// rule in play_skills.rs's `LineHistory`; this one is a static because `hurt`, which says the
/// death line, has no resources at hand (and a system of its own to empty it moved the other
/// map-start systems' order, and with it a few pixels of the squad's captures). Only the
/// enemies' death lines go through it (`death_cry`); the squad's picks stay the demo's (play.rs
/// `say`).
static SAID: std::sync::Mutex<Vec<(String, u32, u32)>> = std::sync::Mutex::new(Vec::new());

/// A character's MOTION table resolved to its clips (see the module notes), built at spawn.
#[derive(Default, Debug)]
pub struct Motions {
    /// per motion type, its MOTIONs in the file's order
    by_type: HashMap<i64, Vec<MotionClips>>,
}

/// One MOTION's clips: [slot 0, slot 1] x [script, ready-script] (`Motion::clip`, w1_ stance), as
/// indices into the model's clips; the second slot is `stunned2`, `knockdown2`. And whether it
/// loops, and the direction its root moves over the clip (model space, -Z forward; None when it
/// stays put): the way its clip carries the body, which a drive turns the body by.
#[derive(Clone, Debug)]
pub struct MotionClips {
    pub clips: [[Option<usize>; 2]; 2],
    pub looping: bool,
    pub heading: [Option<f32>; 2],
}

impl Motions {
    /// The clip of motion type `mt` (its first MOTION's), slot `variant`, the ready-script's when
    /// `ready` (the script's when that one has none), and its MOTION.
    pub fn clip(&self, mt: i64, variant: usize, ready: bool) -> Option<(usize, &MotionClips)> {
        let m = self.by_type.get(&mt)?.first()?;
        let slot = &m.clips[variant.min(1)];
        let c = if ready { slot[1].or(slot[0]) } else { slot[0].or(slot[1]) }?;
        Some((c, m))
    }

    /// How many slots of motion type `mt` have a clip (1 or 2: `stunned` and `stunned2`).
    pub fn slots(&self, mt: i64) -> usize {
        self.by_type.get(&mt).and_then(|v| v.first()).map_or(0, |m| m.clips.iter().filter(|s| s.iter().any(|c| c.is_some())).count())
    }
}

/// The MOTION table of type `t` resolved against `model`'s clips (see `Motions`); empty without
/// its motion set.
pub fn motions(model: &Character, game: &Game, t: &CharacterType) -> Motions {
    let mut out = Motions::default();
    let Some(ms) = game.motion_sets.get(&t.motion_set) else { return out };
    let has = |c: u32| model.anims.iter().any(|a| a.name == c);
    for m in &ms.motions {
        let clip = |v: usize, ready: bool| m.clip(v, ready, 1, 0, has).and_then(|n| model.anim_by_name(&n));
        let clips = [[clip(0, false), clip(0, true)], [clip(1, false), clip(1, true)]];
        let heading = |c: Option<usize>| c.and_then(|c| {
            let d = model.anims[c].duration;
            let r = locomotion::root_position(model, game, c, d * 0.999) - locomotion::root_position(model, game, c, 0.0);
            (Vec2::new(r.x, r.z).length() > 0.05).then(|| (-r.x).atan2(-r.z))
        });
        let heading = [heading(clips[0][1].or(clips[0][0])), heading(clips[0][0].or(clips[0][1]))];
        out.by_type.entry(m.motion_type).or_default().push(MotionClips { clips, looping: m.looping, heading });
    }
    // BF_ENEMY_LOG: the table, each clip with its length, own speed (h_f9628571), root travel
    // over the clip and its heading
    if std::env::var("BF_ENEMY_LOG").is_ok() {
        let mut types: Vec<&i64> = out.by_type.keys().collect();
        types.sort();
        for mt in types {
            for mc in &out.by_type[mt] {
                let show = |c: Option<usize>| c.map_or("-".to_string(), |c| {
                    let a = &model.anims[c];
                    let r = locomotion::root_position(model, game, c, a.duration * 0.999) - locomotion::root_position(model, game, c, 0.0);
                    format!("{c} {:.2}s speed {:?} root ({:.2}, {:.2}) {:.2} m", a.duration, a.move_speed.map(|s| (s * 100.0).round() / 100.0), r.x, r.z,
                            Vec2::new(r.x, r.z).length())
                });
                println!("type h_{:08x} {mt:2} {:<22} script {} | ready {} | 2nd {} / {} | loop {} heading {:?}", t.key, mt_name(*mt), show(mc.clips[0][0]),
                         show(mc.clips[0][1]), show(mc.clips[1][0]), show(mc.clips[1][1]), mc.looping, mc.heading.map(|h| h.map(|h| h.to_degrees().round())));
            }
        }
    }
    out
}

/// The gait clips of a character of type `OTHER` from its MOTION table (`motions`), over what
/// `pick_clips` found by the squad's names: per stance slot, the carried set from the scripts and
/// the ready set from the ready-scripts. The idle is the sequence's default motion (idle_stop,
/// FUN_00130c50's type+0x2b8: what a spawned enemy stands in; the takes' frozen Heavy stands in its
/// pose, gun low across the body, take 01 at 35 s), the ready one its ready-script (rp_idle). (The
/// squad's idle is its general idle, idle_general; an unalerted enemy's idle variations are the
/// AI's, out of scope.)
/// walk and run MT_WALK and MT_RUN (the Shrike Heavy's MT_RUN plays rp_run both ways), the sprint
/// MT_DASH, the back steps MT_BACKSTEP (walk) and MT_BACKPEDAL (run), the side steps the walking
/// MOVE_*_SLOW (the Shrike Soldier has none: its sides are run strafes, which only the hook
/// plays). A motion the table lacks keeps `pick_clips`' choice.
pub fn table_clips(clips: &mut Clips, m: &Motions, game: &Game, t: &CharacterType) {
    let default = game.motion_sets.get(&t.motion_set).map_or(0, |ms| ms.default_motion);
    let ms = game.motion_sets.get(&t.motion_set);
    // (the default motion by its MOTION name, else MT_IDLE's first)
    let idle = |ready: bool| -> Option<usize> {
        let named = ms.and_then(|ms| ms.motions.iter().position(|x| x.name == default)).and_then(|i| {
            let ms = ms?;
            let mt = ms.motions[i].motion_type;
            let k = ms.motions[..i].iter().filter(|x| x.motion_type == mt).count();
            m.by_type.get(&mt)?.get(k).and_then(|c| if ready { c.clips[0][1].or(c.clips[0][0]) } else { c.clips[0][0].or(c.clips[0][1]) })
        });
        named.or(m.clip(MT_IDLE, 0, ready).map(|c| c.0))
    };
    let get = |mt: i64, ready: bool| m.clip(mt, 0, ready).map(|c| c.0);
    for slot in clips.slots.iter_mut() {
        for (set, ready) in [(&mut slot.carry, false), (&mut slot.rp, true)] {
            set.idle = idle(ready).or(set.idle);
            set.walk = get(MT_WALK, ready).or(set.walk);
            set.run = get(MT_RUN, ready).or(set.run);
            set.sprint = get(MT_DASH, ready).or(set.sprint);
            set.walk_back = get(MT_BACKSTEP, ready).or(set.walk_back);
            set.run_back = get(MT_BACKPEDAL, ready).or(set.run_back);
        }
        let sides = [MT_MOVE_LEFT_SLOW, MT_MOVE_RIGHT_SLOW, MT_MOVE_LEFT_BACK_SLOW, MT_MOVE_RIGHT_BACK_SLOW];
        for (k, mt) in sides.into_iter().enumerate() {
            slot.rp_side[k] = get(mt, true).or(slot.rp_side[k]);
        }
    }
    clips.loco.idle = idle(false).or(clips.loco.idle);
    clips.loco.walk = get(MT_WALK, false).or(clips.loco.walk);
    clips.loco.run = get(MT_RUN, false).or(clips.loco.run);
    clips.loco.walk_back = get(MT_BACKSTEP, false).or(clips.loco.walk_back);
    clips.loco.run_back = get(MT_BACKPEDAL, false).or(clips.loco.run_back);
}

/// An enemy's own state (`Player::enemy`; the squad's stays at its default, but for the weapons
/// and `<ai>` of the placement a squad member starts from: play_triggers.rs `SquadSpawn::dress`).
#[derive(Default)]
pub struct State {
    /// its personality (a `Game::personalities` name): the placement's personality-freewill
    /// (char+0x1bc, FUN_00110cd0) when it was spawned from one, else its type's; 0 none
    pub personality: u32,
    /// the weapons of the placement it was spawned from (its `<inventory><weapon>`, the second
    /// slot empty on every enemy: e34's A50s, Saryl-45s, Rail RVRs); None: its type's (the
    /// character's property handler 0x112eb0 gives a placement's inventory, FUN_001130e0)
    pub weapons: Option<Vec<u32>>,
    /// the `<ai>` of the placement it came from (FUN_00110cd0 copies it to char+0x1b8, the
    /// sensor's): its sight distance and field of view (play_brain.rs `Sensor::set`, #197); None
    /// from none (a squad member on a map that doesn't place it, a test map NPC of a type)
    pub ai: Option<bf_viewer::bf::character_type::PlacementAi>,
    /// the motion playing in place of its gait (a reaction, or the hook's)
    pub motion: Option<Playing>,
    /// a damage event the brain takes in this many seconds (REACTION_DELAY), when its reaction
    /// is drawn: the event, and the yaw that faces the shooter (the clip turned away from the
    /// hit: see `start_once`)
    pub pending: Option<(f32, i64, Option<f32>)>,
    /// the brain's damage record (brain+0x224): (sim time, amount) of the hits that reached the
    /// hit points in the last DAMAGE_WINDOW s
    pub record: Vec<(f32, f32)>,
    /// the reactions' cooldowns left (s): knock-down, stun (brain+0xd4, +0xe4)
    pub cooldowns: [f32; 2],
    /// damage events posted during a reaction, waiting for it to end: (the entry's priority, the
    /// event, the yaw that faces the shooter). The brain's event heap (brain+0x404, FUN_0001d800)
    /// keeps them: the damage entries don't decay (h_f167afaa 86.86, and EVT_USER2's 0)
    pub queued: Vec<(f32, i64, Option<f32>)>,
    /// its brain is off (BF_TEST_NPC's `frozen`: the takes' trainer freeze, brain mode 6, which
    /// gets no damage message, 0x3dd83): no reactions
    pub frozen: bool,
    /// the test hook's drive (BF_TEST_NPC_MOTION)
    pub drive: Option<Drive>,
    /// the ground velocity (world, m/s; the game's char+0x470), eased toward the wanted one
    pub velocity: Vec3,
    /// its type's stamina-recovery (a second; 0 none) and whether its stamina shields it (the
    /// type's shield-type 2), set at spawn
    pub recovery: f32,
    pub stamina_shield: bool,
    /// the playing clip begins anew this frame, though the top of the stack is the same clip (a
    /// second knock-down)
    pub restart: bool,
}

/// A motion played in place of the gait (`State::motion`).
#[derive(Clone, Copy, Debug)]
pub struct Playing {
    /// its motion type (the MT_ enum) and clip
    pub mt: i64,
    pub clip: usize,
    /// plays once (a reaction), or loops (a drive's)
    pub once: bool,
    /// seconds into it, and how long the motion lasts (a reaction's whole state)
    pub time: f32,
    pub hold: f32,
    /// it moves by its clip's root (FUN_0012cb40's list), not by the speeds
    pub by_root: bool,
    /// its clip's rate (a walking clip's speed over its own)
    pub rate: f32,
}

/// The test hook's drive: a motion type held until the goal is reached (or for good).
#[derive(Clone, Copy, Debug)]
pub struct Drive {
    pub mt: i64,
    /// where to (x, z); None: in place
    pub goal: Option<Vec2>,
    /// the ready-script's clips (the gun raised, as every Shrike in the takes), else the script's
    pub ready: bool,
}

/// What `step` tells `step_player`: the clip to play instead of the gait's, once or looping, at
/// what rate, and whether it moves the body by its root (else by `State::velocity`).
pub struct Scripted {
    pub clip: usize,
    pub once: bool,
    pub rate: f32,
    pub by_root: bool,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, refill.after(update_player).run_if(in_state(AppState::Playing)));
}

/// Whether `u` is an enemy of a type of its own (none of the squad's four: `Player::own_type`),
/// which this module runs: a test map NPC, a level's character, or on the test map a squad member
/// of such a type (#302), which reacts, shields and dies as its type does whoever controls it.
pub fn enemy(u: &Player) -> bool {
    u.own_type()
}

/// A map starts (play.rs `setup`): the chatter manager is the world's, so its history (`SAID`)
/// starts empty.
pub fn new_world() {
    SAID.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

/// `u`'s death line (chatter tag `tag`, e1213623), started now: the game says it in the death
/// frame (the CBT_DEAD handler FUN_00115f90, after the death FUN_00115d50, calls FUN_00125f40 at
/// 0x116028, which says it through the chatter gate FUN_00140a60; no
/// delay, answers.md Q8: the takes' 0.15-0.35 s was the capture's audio latency). From its type's
/// chatter-library (#129: `shrike_light_chatter`, `dm_shrike_heavy_chatter`...; the entry's flag
/// h_eb640008 lets the dead speak), picked the game's way (FUN_001371f0): at random among the
/// lines not said yet on the map (`SAID`), so a Shrike's three all come before one repeats.
pub fn death_cry(u: &mut Player, game: &Game, tag: u32) {
    let Some(library) = game.character_type(u.kind).map(|t| t.chatter.trim_end_matches("_chatter").to_string()) else { return };
    let lines: Vec<u32> = game.chatter.get(&library).and_then(|c| c.get(&tag))
        .map(|v| v.iter().copied().filter(|&id| game.sounds.has(id)).collect()).unwrap_or_default();
    if lines.is_empty() {
        return;
    }
    let mut said = SAID.lock().unwrap_or_else(|e| e.into_inner());
    let unsaid = |said: &Vec<(String, u32, u32)>| -> Vec<u32> {
        lines.iter().copied().filter(|&id| !said.iter().any(|(l, t, s)| *l == library && *t == tag && *s == id)).collect()
    };
    let mut left = unsaid(&said);
    if left.is_empty() {
        said.retain(|(l, t, _)| !(*l == library && *t == tag));
        left = lines.clone();
    }
    let id = left[u.random(left.len())];
    said.push((library.clone(), tag, id));
    if std::env::var("BF_ENEMY_LOG").is_ok() {
        println!("{}'s death line {id:08x} of {} ({} not said yet)", unit_name(u), lines.len(), left.len());
    }
    u.sound_queue.push((id, 1.0));
    u.speaking = game.sounds.pcm(id).map_or(1.0, |(rate, s)| s.len() as f32 / rate as f32);
}

/// Whether `u` is reacting to a hit (its event due, or the reaction playing): a hit meanwhile
/// starts no reaction, its event waits for this one's end (`State::queued`; take 12's and take
/// 14's hits during a stun drew none).
pub fn reacting(u: &Player) -> bool {
    u.enemy.pending.is_some() || u.enemy.motion.is_some_and(|m| matches!(m.mt, MT_DIZZY | MT_KNOCKDOWN))
}

/// The speed (m/s) motion type `mt` moves at, by FUN_0012c6c0 with the stick (the drive's
/// request) at full: the type's crouch walk for MT_CROUCH_WALK, walk for the walking types, run
/// for the running ones, dash for MT_DASH, 0 for the rest.
pub fn speed(t: &CharacterType, mt: i64) -> f32 {
    match mt {
        MT_CROUCH_WALK => t.speeds[0],
        MT_DASH => t.speeds[3],
        mt if WALKING.contains(&mt) => t.speeds[1],
        mt if RUNNING.contains(&mt) => t.speeds[2],
        _ => 0.0,
    }
}

/// A motion type by name (MT_ without the prefix, any case: `walk`, `move_left`, `dizzy`) or
/// number.
pub fn motion_type(s: &str) -> Option<i64> {
    let s = s.trim().to_ascii_uppercase();
    let s = s.strip_prefix("MT_").unwrap_or(&s);
    s.parse::<i64>().ok().filter(|&n| (0..MOTION_TYPES.len() as i64).contains(&n))
        .or_else(|| MOTION_TYPES.iter().position(|&n| n == s).map(|n| n as i64))
}

/// The motion type's name, for logs.
fn mt_name(mt: i64) -> &'static str {
    MOTION_TYPES.get(mt as usize).copied().unwrap_or("?")
}

/// A drive (the test hook): start motion type `mt` toward `goal` (or in place). A motion that
/// plays once (a reaction's, a dodge) plays through and ends.
pub fn drive(u: &mut Player, mt: i64, goal: Option<Vec2>, ready: bool) {
    u.enemy.drive = Some(Drive { mt, goal, ready });
    u.enemy.motion = None;
    u.enemy.pending = None;
}

/// A non-lethal hit of `amount` (after every factor and the stamina shield: what take-damage,
/// FUN_002232d0, sends with CBT_DAMAGED) reached `u`'s hit points from the way `way` (the shot's,
/// world): the brain's damage message and event, its personality's draw (answers.md Q1; the brain's
/// handler 0x3dce0, CBT_DAMAGED at 0x3e528 posting EVT_DAMAGED, handled at 0x3e756):
///  1. the hit goes on the damage record (FUN_0003dab0), which keeps the last DAMAGE_WINDOW s;
///  2. a brain that knows an enemy (the demo: an NPC set to fight with a hostile in sight, or a
///     level character's brain with a contact, play_brain.rs) posts
///     EVT_DAMAGE_HEAVY when the record's total is over HEAVY_SHARE of the max hit points (the
///     record emptied), else EVT_DAMAGE_LIGHT; an entry under FIGHT_PRIORITY never acts;
///  3. one that knows none (brain+0x3c4 bit 7; the demo's dummies) posts EVT_DAMAGED_NO_ENEMIES,
///     whose entry posts EVT_USER2 ("Involuntary response", KNOCKDOWN 1 : STUN 1 at 100);
///  4. REACTION_DELAY s later the brain takes the event (`step`): the entry's draw
///     (`take_event`); STUN or KNOCKDOWN starts then, NOOP does nothing.
/// A frozen brain gets nothing (mode 6). While an event is due or a reaction plays the event waits
/// (`State::queued`): STUN and KNOCKDOWN are uninterruptible goals (class 0x30 in the goal table
/// 0x4a5908, filled by FUN_000560c0; the event processing tests 0x20), and the brain takes the
/// queued events, highest priority first, once the reaction's goal is done (`step`), when the
/// cooldowns mostly refuse them (not a knock-down 2.0 s into the Heavy's 2.17 s stunned2).
pub fn damaged(u: &mut Player, game: &Game, amount: f32, way: Vec3) {
    if !enemy(u) || u.dead || u.enemy.frozen || amount <= 0.0 {
        return;
    }
    let now = u.sim_time;
    u.enemy.record.retain(|&(t, _)| now - t <= DAMAGE_WINDOW);
    u.enemy.record.push((now, amount));
    let log = std::env::var("BF_ENEMY_LOG").is_ok();
    let personality = u.enemy.personality;
    // (a level character's brain knows an enemy when it has a contact, play_brain.rs; the test
    // map's NPCs when they have a hostile in sight)
    let knows = u.ai_target.is_some() || brain::knows_enemy(u);
    let (event, entry) = if knows {
        let sum: f32 = u.enemy.record.iter().map(|r| r.1).sum();
        let event = if sum > HEAVY_SHARE * u.max_health { EVT_DAMAGE_HEAVY } else { EVT_DAMAGE_LIGHT };
        if event == EVT_DAMAGE_HEAVY {
            u.enemy.record.clear();
        }
        if log {
            println!("t={now:.2} {}: {amount:.1} damage, record {sum:.1} of max {}: {}", unit_name(u), u.max_health,
                     if event == EVT_DAMAGE_HEAVY { "heavy" } else { "light" });
        }
        (event, game.personality_entry(personality, event).filter(|e| e.priority >= FIGHT_PRIORITY))
    } else {
        let user2 = game.personality_entry(personality, EVT_DAMAGED_NO_ENEMIES).filter(|e| e.spawns.contains(&EVT_USER2))
            .and_then(|_| game.personality_entry(personality, EVT_USER2));
        if log {
            println!("t={now:.2} {}: {amount:.1} damage, no enemy known: EVT_DAMAGED_NO_ENEMIES -> EVT_USER2 {}", unit_name(u),
                     if user2.is_some() { "" } else { "(none)" });
        }
        (EVT_USER2, user2)
    };
    let Some(entry) = entry else { return };
    let back = Vec2::new(way.x, way.z);
    let face = (back.length() > 1e-3).then(|| back.x.atan2(back.y));
    if reacting(u) {
        if log {
            println!("  (reacting: event {event} waits)");
        }
        u.enemy.queued.push((entry.priority, event, face));
        return;
    }
    u.enemy.pending = Some((REACTION_DELAY, event, face));
}

/// The brain takes damage event `event` (its personality's entry) and draws its goal
/// (FUN_0003b580): the draw (FUN_00054c40), and a goal whose start refuses it struck out and
/// drawn again (FUN_00054bc0), at most once per goal of the entry. GOAL_KNOCKDOWN's start
/// (0x40870) and GOAL_STUN's (0x3f460) refuse while their cooldown runs, or when the type has no
/// MOTION of their motion type (MT_KNOCKDOWN, MT_DIZZY: type+0x1a8 bits 6 and 7, from its MOTION
/// table, FUN_00130c50; `ms_shadoon` has a DIZZY and no KNOCKDOWN). The reaction drawn, if any.
fn take_event(u: &mut Player, game: &Game, event: i64, log: bool) -> Option<i64> {
    let personality = u.enemy.personality;
    let entry = game.personality_entry(personality, event)?;
    let t = game.character_type(u.kind);
    let mut out: Vec<i64> = vec![];
    let goal = loop {
        let draw = u.random(1_000_000) as f32 / 1_000_000.0;
        let Some(g) = entry.draw(draw, &out) else { break None };
        let refused = match g {
            GOAL_KNOCKDOWN => u.enemy.cooldowns[0] > 0.0 || !has_motion(game, t, MT_KNOCKDOWN),
            GOAL_STUN => u.enemy.cooldowns[1] > 0.0 || !has_motion(game, t, MT_DIZZY),
            _ => false,
        };
        if !refused {
            break Some(g);
        }
        out.push(g);
        if out.len() >= entry.goals.len() {
            break None;
        }
    };
    if log {
        println!("t={:.2} {}: personality h_{personality:08x} event {event}: goals {:?}, refused {out:?}, drew {goal:?} (cooldowns {:?})",
                 u.sim_time, unit_name(u), entry.goals, u.enemy.cooldowns);
    }
    goal.filter(|&g| matches!(g, GOAL_STUN | GOAL_KNOCKDOWN))
}

/// Whether type `t` has a MOTION of motion type `mt` in its motion set's MOTION table (what the
/// reaction goals' starts test, by the type flags FUN_00130c50 sets from it).
fn has_motion(game: &Game, t: Option<&CharacterType>, mt: i64) -> bool {
    t.and_then(|t| game.motion_sets.get(&t.motion_set)).is_some_and(|ms| ms.motions.iter().any(|m| m.motion_type == mt))
}

/// Start reaction `goal` (GOAL_STUN or GOAL_KNOCKDOWN): its motion type's clip, one of its two at
/// random (`start_once`), the body first turned to face the shooter (`face`), so the clip throws
/// it away from the hit.
fn react(u: &mut Player, l: &Loaded, goal: i64, face: Option<f32>, log: bool) {
    if let Some(f) = face {
        u.yaw = f;
    }
    start_once(u, l, if goal == GOAL_STUN { MT_DIZZY } else { MT_KNOCKDOWN }, log);
}

/// One frame of `u`'s own motions, before `step_player` picks its clip: a drawn reaction comes
/// due, a playing motion runs on and ends, the drive steers. While a motion plays the character's
/// own inputs are dropped (it doesn't fire, aim or turn by them). What to play, if anything.
pub fn step(u: &mut Player, l: &Loaded, game: &Game, dt: f32) -> Option<Scripted> {
    if !enemy(u) {
        return None;
    }
    let log = std::env::var("BF_ENEMY_LOG").is_ok();
    let t = game.character_type(u.kind);
    // the cooldowns run down (the brain's timers, in game time)
    for c in u.enemy.cooldowns.iter_mut() {
        *c = (*c - dt).max(0.0);
    }
    // a damage event comes due: the brain takes it now, drawing its reaction with the cooldowns
    // as they are now (the goals' starts test them)
    if let Some((left, event, face)) = u.enemy.pending.take() {
        let left = left - dt;
        if left > 0.0 {
            u.enemy.pending = Some((left, event, face));
        } else if let Some(goal) = take_event(u, game, event, log) {
            react(u, l, goal, face, log);
        }
    }
    // the reaction done, the events that waited, highest priority first, until one draws a
    // reaction (the rest wait on)
    if !reacting(u) && !u.enemy.queued.is_empty() {
        u.enemy.queued.sort_by(|a, b| a.0.total_cmp(&b.0));
        while let Some((_, event, face)) = u.enemy.queued.pop() {
            if let Some(goal) = take_event(u, game, event, log) {
                react(u, l, goal, face, log);
                break;
            }
        }
    }
    // the drive: a motion it holds (looping), or plays once
    if u.enemy.motion.is_none() || u.enemy.motion.is_some_and(|m| !reaction(m.mt)) {
        if let Some(d) = u.enemy.drive {
            steer(u, l, t, d, dt, log);
        }
    }
    let m = u.enemy.motion.as_mut()?;
    m.time += dt * m.rate;
    let m = *m;
    if m.once && m.time >= m.hold {
        if log {
            println!("t={:.2} {}: {} ends after {:.2} s", u.sim_time, unit_name(u), mt_name(m.mt), m.time);
        }
        let done = m.mt;
        u.enemy.motion = None;
        // (a drive's motion that plays once is done with it)
        if u.enemy.drive.is_some_and(|d| d.mt == done) {
            u.enemy.drive = None;
        }
        // standing where the clip left it, facing its way (the hook's NPC keeps it)
        if let Some(n) = u.npc.as_mut() {
            n.face = u.yaw;
        }
        u.face_yaw = Some(u.yaw);
        return None;
    }
    // its own inputs dropped meanwhile (a reaction: no fire, no aim; a drive: the hook's)
    u.move_input = Vec2::ZERO;
    u.fire = false;
    u.aim = false;
    u.aim_hold = 0.0;
    u.dodge_pressed = false;
    u.jump_pressed = false;
    u.crouch_wanted = false;
    u.face_yaw = Some(u.yaw);
    Some(Scripted { clip: m.clip, once: m.once, rate: m.rate, by_root: m.by_root })
}

/// Whether a motion type is a hit reaction.
fn reaction(mt: i64) -> bool {
    matches!(mt, MT_DIZZY | MT_KNOCKDOWN)
}

/// Start motion type `mt` played once (a reaction, or the hook's dodge or dive): one of its slots
/// at random, uniform (FUN_00150670 at 0x15079d: the LCG modulo the MOTION's count of scripts,
/// kept at char+0x224), at rate 1, the state lasting the clip's length (nothing holds it longer:
/// the clip's end event 0xfaea99c9 clears it and the next-motion, idle_stop, starts; answers.md
/// Q2). A reaction sets the cooldowns (`KNOCKDOWN_COOLDOWNS`, `STUN_COOLDOWNS`). The game turns the
/// clip by char+0x434 + pi (0x150845); that char+0x434 is the hit's way is the analyst's guess, so
/// a reaction faces the shooter first (`react`). The MOTION's blend-in (`transition`,
/// 0.2 s hf, 0.25 s lf) is the demo's 0.2 s crossfade (FADE).
fn start_once(u: &mut Player, l: &Loaded, mt: i64, log: bool) {
    let slots = l.motions.slots(mt).max(1);
    let v = u.random(slots);
    let ready = u.enemy.drive.is_none_or(|d| d.ready);
    let Some((clip, _)) = l.motions.clip(mt, v, ready).or(l.motions.clip(mt, 0, ready)) else {
        if log {
            println!("t={:.2} {}: no clip for {}", u.sim_time, unit_name(u), mt_name(mt));
        }
        return;
    };
    let len = l.model.anims[clip].duration;
    if log {
        println!("t={:.2} {}: {} plays clip {} slot {} ({:.2} s)", u.sim_time, unit_name(u), mt_name(mt), clip, v + 1, len);
    }
    match mt {
        MT_KNOCKDOWN => u.enemy.cooldowns = KNOCKDOWN_COOLDOWNS,
        MT_DIZZY => u.enemy.cooldowns = STUN_COOLDOWNS,
        _ => {}
    }
    u.enemy.motion = Some(Playing { mt, clip, once: true, time: 0.0, hold: len, by_root: BY_ROOT.contains(&mt), rate: 1.0 });
    u.enemy.restart = true;
    u.enemy.velocity = Vec3::ZERO;
    if reaction(mt) {
        u.enemy.drive = None;
        // (the gun lowered, a throw or reload cut short as the knock-down's ragdoll cut them)
        u.reloading = None;
        u.switching = None;
        u.cancel_throw();
    }
}

/// The drive's frame: its motion's clip (looping, or once), the body turned at the type's
/// turn-rate to where the clip carries it along the way to the goal, and the velocity eased
/// toward the motion type's speed that way (FUN_0012c6c0). At the goal it lets go: the motion
/// turns IDLE at once and the body slides to a stop (B1: ~0.8 s).
fn steer(u: &mut Player, l: &Loaded, t: Option<&CharacterType>, d: Drive, dt: f32, log: bool) {
    let Some(t) = t else { return };
    let Some((clip, mc)) = l.motions.clip(d.mt, 0, d.ready) else {
        if log {
            println!("t={:.2} {}: no clip for {}: drive dropped", u.sim_time, unit_name(u), mt_name(d.mt));
        }
        u.enemy.drive = None;
        return;
    };
    if !mc.looping && !WALKING.contains(&d.mt) && !RUNNING.contains(&d.mt) && d.mt != MT_DASH && d.mt != MT_CROUCH_WALK {
        // (a one-shot: a reaction, dodge or dive, in place)
        if u.enemy.motion.is_none_or(|m| m.mt != d.mt) {
            start_once(u, l, d.mt, log);
        }
        return;
    }
    let here = Vec2::new(u.position.x, u.position.z);
    let way = d.goal.map(|g| g - here);
    let speed = speed(t, d.mt);
    let slide = u.enemy.velocity.length() / SLOW_DOWN;
    if way.is_some_and(|w| w.length() <= ARRIVE + slide) {
        if log {
            println!("t={:.2} {}: drive {} reached {:?} at ({:.2}, {:.2})", u.sim_time, unit_name(u), mt_name(d.mt), d.goal, u.position.x, u.position.z);
        }
        u.enemy.drive = None;
        u.enemy.motion = None;
        if let Some(n) = u.npc.as_mut() {
            n.face = u.yaw;
        }
        return;
    }
    // the body faces the way minus the clip's own heading (a strafe left faces 90 degrees right of
    // its way), turning at the type's turn-rate (type+0x2d0, the AI's)
    let ready = d.ready;
    let heading = mc.heading[if ready { 0 } else { 1 }];
    if let Some(w) = way.filter(|w| w.length() > 1e-3) {
        let way_yaw = (-w.x).atan2(-w.y);
        let want = wrap_angle(way_yaw - heading.unwrap_or(0.0));
        let rate = t.turn_rate.to_radians().max(0.1);
        u.yaw = wrap_angle(u.yaw + wrap_angle(want - u.yaw).clamp(-rate * dt, rate * dt));
    }
    // the velocity: along the way (the stick's direction, not the clip's), eased
    let dir = way.map_or(Vec2::ZERO, |w| w.normalize_or_zero());
    let target = Vec3::new(dir.x, 0.0, dir.y) * speed;
    let k = if target.length_squared() < u.enemy.velocity.length_squared() { SLOW_DOWN } else { SPEED_UP };
    u.enemy.velocity += (target - u.enemy.velocity) * (k * dt).min(1.0);
    // a walking clip at the speed over its own (FUN_0012c6c0: the clip record's +0x28)
    let own = l.model.anims[clip].move_speed.unwrap_or(0.0);
    let rate = if WALKING.contains(&d.mt) && own >= NO_CLIP_SPEED { (speed / own).min(MAX_WALK_RATE) } else { 1.0 };
    // in scripted play (the world's cutscene bit, from a script's 122 to its 123: play_script.rs)
    // every character moves by its clip's root instead, and FUN_0012c6c0, which sets the clip's
    // rate, isn't run (FUN_0012cb40): the first patrol walks so through tutorial_show_patrol (#227)
    let rooted = super::script::in_cutscene();
    match u.enemy.motion.as_mut() {
        Some(m) if m.mt == d.mt && m.clip == clip => {
            if !rooted {
                m.rate = rate;
            }
            m.by_root = rooted;
        }
        _ => {
            if log {
                println!("t={:.2} {}: drive {} clip {} ({:.2} s, own speed {own:.2}) at {speed:.2} m/s, rate {rate:.2}, heading {:?}{}", u.sim_time,
                         unit_name(u), mt_name(d.mt), clip, l.model.anims[clip].duration, heading.map(|h| h.to_degrees().round()),
                         if rooted { " (scripted play: by the clip's root, rate 1)" } else { "" });
            }
            u.enemy.motion = Some(Playing { mt: d.mt, clip, once: false, time: 0.0, hold: f32::INFINITY, by_root: rooted, rate: if rooted { 1.0 } else { rate } });
        }
    }
    if log && (u.sim_time * 2.0).floor() != ((u.sim_time - dt) * 2.0).floor() {
        let v = Vec2::new(u.enemy.velocity.x, u.enemy.velocity.z);
        println!("t={:.2} {}: drive {} at ({:.2}, {:.2}) speed {:.2} yaw {:.0}", u.sim_time, unit_name(u), mt_name(d.mt), u.position.x, u.position.z,
                 v.length(), u.yaw.to_degrees());
    }
}

/// The step `step_player` moves a driven or sliding enemy by (world), from its eased velocity;
/// slowing to a stop once the drive is over. None while nothing moves it this way.
pub fn glide(u: &mut Player, scripted: Option<&Scripted>, dt: f32) -> Option<Vec3> {
    if !enemy(u) || scripted.is_some_and(|s| s.by_root) {
        return None;
    }
    // (no drive: the slide to a stop, at SLOW_DOWN)
    if u.enemy.drive.is_none() {
        let v = u.enemy.velocity;
        if v.length_squared() < 1e-4 {
            u.enemy.velocity = Vec3::ZERO;
            return None;
        }
        u.enemy.velocity -= v * (SLOW_DOWN * dt).min(1.0);
    }
    Some(u.enemy.velocity * dt)
}

/// The stamina shield (0x223690): a type whose shield-type is 2 (EST_STAMINA_SHIELD: the
/// deathmatch and squad deathmatch Shrike Soldier, Shrike Heavy and Hunter Lord) takes a hit of
/// `amount` from the stamina first, min(stamina, amount) off both, and what's left goes on to the
/// hit points in the same hit (B2: the Soldier's 31.40 took its stamina 25 to 0 and its HP 50 to
/// 43.60; the Heavy's 31.24 its stamina 40 to 8.76, its HP stayed 55). What's left. The game
/// takes it before the damage type's factor (FUN_002232d0); here it's taken from the amount after
/// it, the same for the deathmatch types (every factor 1) and for the squad deathmatch ones but
/// on gas (0.75). A hit it takes some of plays the shield's hit-effect round the body
/// (`shield_look`).
pub fn absorb(u: &mut Player, game: &Game, amount: f32) -> f32 {
    if !u.enemy.stamina_shield || amount <= 0.0 {
        return amount;
    }
    let took = u.stamina.min(amount).max(0.0);
    if took <= 0.0 {
        return amount;
    }
    u.stamina -= took;
    if std::env::var("BF_ENEMY_LOG").is_ok() || std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("  stamina shield: {} took {took:.2} of {amount:.2} -> stamina {:.2}", unit_name(u), u.stamina);
    }
    shield_look(u, game);
    amount - took
}

/// The damage that kills `u` in one hit (the test map's instant kill): its hit points, and the
/// stamina its stamina shield would take first (`absorb`), so K kills the deathmatch Shrikes too.
pub fn lethal(u: &Player) -> f32 {
    u.health + if u.enemy.stamina_shield { u.stamina.max(0.0) } else { 0.0 }
}

/// The shield's hit-effect (`<shield hit-effect>`, h_1218cdc1 on the deathmatch Shrikes: an effect
/// type whose ALE effect is `pshield`, pshield_face.app with small-flare.tga), played once round
/// the body (its middle, SHIELD_LOOK_HEIGHT m over the feet) through `Player::splashes`. B2: a hit
/// the shield took showed a blue-white shimmer round the victim for ~0.5-1 s (takes 38, 39). Where
/// the game places it (the body, a bone) isn't traced: the demo's.
fn shield_look(u: &mut Player, game: &Game) {
    let Some(effect) = game.character_type(u.kind).and_then(|t| t.shield.as_ref()).map(|s| s.hit_effect).filter(|&e| e != 0) else { return };
    let at = u.position + Vec3::Y * (GROUND + SHIELD_LOOK_HEIGHT);
    let effects = game.effect_types.get(&effect).map(|v| v.as_slice()).unwrap_or(&[]);
    if std::env::var("BF_ENEMY_LOG").is_ok() {
        println!("  shield hit-effect h_{effect:08x}: ALE effects {:x?} at {at:.2}", effects);
    }
    for &fx in effects {
        u.splashes.push((Transform::from_translation(at), fx, 0.0));
    }
}

/// How high over the feet the shield's look plays (m): about the middle of a 1.9-2.0 m Shrike (the
/// demo's).
const SHIELD_LOOK_HEIGHT: f32 = 1.0;

/// The stamina's refill for an enemy whose type has the stamina update (h_e78efbb2: every
/// multiplayer type), FUN_0011a840 (0x11a95a-0x11a9b3): its stamina-recovery a second up to its
/// max, x0.25 dashing (MT_DASH), x2 crouched (MT_CROUCH), with no pause after a hit or after
/// emptying (the shield has no timer; answers.md Q5). The squad's is play_skills.rs's. The takes'
/// steps of exactly recovery / 10 (Soldier +0.5, Heavy +0.4, 0.14-0.29 s apart: B2) are frames
/// whose dt the game capped at 0.1 s (FUN_00069270) while gdb paused the console between them.
/// (The x4 while the byte char+0x5c is set isn't known: left out.)
fn refill(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let dt = frame_dt(&time);
    // (the controlled character too: on the test map it may be such a type, #302)
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()).filter(|u| enemy(u) && !u.dead && u.enemy.recovery > 0.0) {
        let k = match u.enemy.motion.map(|m| m.mt) {
            Some(MT_DASH) => DASH_REFILL,
            Some(MT_CROUCH) => CROUCH_REFILL,
            _ => 1.0,
        };
        u.stamina = (u.stamina + u.enemy.recovery * k * dt).min(u.max_stamina);
    }
}

/// FUN_0011a840's refill factors: x0.25 in MT_DASH, x2 in MT_CROUCH.
const DASH_REFILL: f32 = 0.25;
const CROUCH_REFILL: f32 = 2.0;

/// Set up an enemy's own state at spawn (`spawn_unit`): its type's personality (unless a
/// placement gave it one), its stamina refill and shield.
pub fn spawned(u: &mut Player, t: Option<&CharacterType>) {
    let Some(t) = t.filter(|_| enemy(u)) else { return };
    if u.enemy.personality == 0 {
        u.enemy.personality = t.ai.personalities[0];
    }
    u.enemy.recovery = t.skill.as_ref().filter(|s| s.stamina_update).map_or(0.0, |s| s.recovery);
    u.enemy.stamina_shield = t.shield.as_ref().is_some_and(|s| s.kind == STAMINA_SHIELD);
    u.enemy.motion = None;
    u.enemy.pending = None;
    u.enemy.record.clear();
    u.enemy.queued.clear();
    u.enemy.cooldowns = [0.0; 2];
    u.enemy.velocity = Vec3::ZERO;
}

/// EST_STAMINA_SHIELD (the shield-type enum, 0x3bf99c).
const STAMINA_SHIELD: i64 = 2;
