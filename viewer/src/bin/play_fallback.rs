//! ATTACK_FALLBACK and MOVE_BACK, the walk away from a target that came too close (#383, part B of
//! #26; viewer/docs/brain.md "The fallback", nav.md "The link walk"): what a brain does when
//! EVT_TGT_TOO_CLOSE (85) is posted for a contact within its too-close radius, and the same 7 m walk
//! GOAL_MOVE_BACK takes. Every function cited is in viewer/docs/code-map.md, "AI"; the reading is
//! todo/26-enemy-behaviour/leads.md §5, checked against the asm, and take07 of todo/132-mutants
//! (F2b).
//!
//!  - **The post** (FUN_000447b0's entry walk, 0x44b57-0x44cc0; `post`): for a contact that is a
//!    character seen now, once the brain's re-arm timer (+0x118) is out and the squared 3D distance
//!    between the two characters' position records is under FUN_0002e210's radius squared (`radius`:
//!    max(the type's `<ai>` h_f1dbdb99, 1.7 x the held gun's Damage radius); 3 m for the Red Hand and
//!    the mutants), the timer is set to 1.5 s (REARM) and EVT_TGT_TOO_CLOSE is posted unless one is
//!    queued, and one is queued **while ATTACK_FALLBACK runs** (`State::queued`): a pursued enemy
//!    posts once and walks its 7 m, it does not start again every 1.5 s. take07: posts at 9.60 and
//!    11.19 game seconds, 1.59 s apart, then none until 22.25 s. **The cover goal's hook keeps the
//!    event while the cover holds** (`Held`, `cover::takes`; FUN_000313e0 via the message handler's
//!    ask of the running goal, FUN_0003d890): nothing is queued or drawn, and the next post, 1.5 s on,
//!    goes through again. take07's first post drew nothing, the cover failed on its 8 m rule, the second
//!    drew the fallback.
//!  - **The draw** (`take`): the personality's entry for event 85 (`default`'s and h_0bb586cf's:
//!    priority 20.17, ATTACK_FALLBACK 1 : ATTACK_COVER 0), over the fight's 18 and under
//!    PATH_TOWARD_HOME's 20.19 (the brain's `past_home` comes first). A draw of ATTACK_FALLBACK that
//!    refuses is struck out and the draw made again (FUN_00054bc0), which comes to ATTACK_COVER: its
//!    start is the cover pick (`cover::start`), and started it runs as the character's attack goal
//!    (play_cover.rs); a test map NPC has no cover to pick and nothing then. The Supermutant's entry
//!    draws its pound (116) first and the Shadoon's 117, which the demo doesn't have (#212's): they do
//!    nothing, and the Molotov Mutant's entry has priority -1 (NOOP).
//!  - **The start** (0x31610 -> FUN_00029a00; `start`): the direction me - target (the two position
//!    records, 3D, **not normalised**), and from the character's place (its position and own point)
//!    the link walk of `Graph::away` with cos 0.6 and length 7.0 over its route (brain+0x930);
//!    refused when it finds none. On a map without world points (the test map's NPCs) the demo
//!    walks straight along the direction, 7 m. The line fe56cd97 is said (#36's: not here). The
//!    cover goal ends with it (`cover::end`, 0x2f900): its point let go, the crouch cleared.
//!  - **The update** (0x316d0; `walk`): done (2) once the squared distance to the target exceeds
//!    (the radius + 5)^2; else the look is the direction to the target, it fires at it (25 degrees;
//!    the caller's `ai_fire`), the pace record's +0x24 is set to 0.1 (`STEP_TIME`: the ramp climbs a
//!    notch a tenth of a second) and the walk update 0x29920 runs: the route walked at the arrival
//!    radius brain+0x4f8 (1.5 m, the constructor's), steered (FUN_000255d0) and moved at mode 3 held
//!    to the pace level with the pace near the route's end (FUN_0004d970(.., 3, 1, 1)); at the route's
//!    end done. The view turns to the look, so the move away is a back step: the motion is picked
//!    as FUN_001195c0 picks it (`facing_mt`: a back or side step with the stick at 65 percent or
//!    over is the run's variant, CT_MOVE_BACK 3 BACKPEDAL, below it 4 BACKSTEP; 0x1221ef), and a
//!    type with no BACKPEDAL keeps BACKSTEP, which it asked for at the walk level before the ramp's
//!    run: the request for a motion type the current motion has no transition to fails and changes
//!    nothing (FUN_0012eab0). The speed is the walk's x0.8, FUN_0012cb40's backward case
//!    (`Drive::back`): take07's Ugly backs off at 1.82 m/s (2.3 x 0.8) for the 3 s it falls back,
//!    however the pace climbs.
//!  - **GOAL_MOVE_BACK** (48; start 0x29a80, update the same 0x29920): the start's direction is the
//!    character's own backward axis (the third column of its transform, a unit vector: 53 degrees
//!    either way), the walk and the end the fallback's but for the done distance. ATTACK_VARY's
//!    RUN_BACK / WALK_BACK (#26 part C) start it; the demo has no draw for them, so BF_TEST_MOVE_BACK
//!    does.
//!
//! Test hooks: BF_BRAIN_LOG=1 (the post, the draw, the start with its route or its refusal, the end
//! and why; BF_ROUTE_LOG=1 the route's points), BF_GOAL_WATCH=<s> (the goal reads ATTACK_FALLBACK or
//! MOVE_BACK, and its facing and motion), BF_TEST_MOVE_BACK=<s>[,<name>[,<x>,<z>]] (at s seconds a level
//! character with a brain, whose name contains `name`, the nearest to (x, z), starts MOVE_BACK),
//! BF_TEST_FALLBACK_REFUSE=1 (ATTACK_FALLBACK's start refuses, so the draw comes to ATTACK_COVER);
//! `BF_AWAY=<x>,<z>,<dx>,<dz>` in bf_level prints the link walk from a place.

use bf_viewer::bf::character::Game;
use bf_viewer::bf::nav::{Graph, Place};

use super::goals::{Foe, Move};
use super::*;

/// The hook's system: BF_TEST_MOVE_BACK.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, move_back_hook.after(read_input).before(update_player).run_if(in_state(AppState::Playing)));
}

/// EVT_TGT_TOO_CLOSE (85, the EVT_ name table 0x3bdb9c), GOAL_ATTACK_FALLBACK (107) and
/// GOAL_MOVE_BACK (48, the GOAL_ table 0x3bcbb4).
pub const EVT_TGT_TOO_CLOSE: i64 = 85;
pub const GOAL_ATTACK_FALLBACK: i64 = 107;
pub const GOAL_MOVE_BACK: i64 = 48;
/// The goals a draw of EVT_TGT_TOO_CLOSE can come to besides the fallback: ATTACK_COVER (19, the
/// entry's second goal, weight 0: drawn once the fallback is struck out; play_cover.rs runs it,
/// #384), and the ones this module doesn't run, for the log: the Supermutant's pound (116) and the
/// Shadoon's (117).
const GOAL_ATTACK_COVER: i64 = cover::GOAL_ATTACK_COVER;
/// EVT_ENEMY_SEEN's priority in a fight (18, "constantly updated to be pri 18 if sight is kept"):
/// an entry has to be over it to interrupt the attack goal (play_enemies.rs `FIGHT_PRIORITY`).
const FIGHT_PRIORITY: f32 = 18.0;
/// The re-arm of the sense step's post: +0x118 and +0x120 get 1.5 (0x3fc00000) at 0x44c79-0x44c85
/// each time the distance test passes, posted or not.
const REARM: f32 = 1.5;
/// FUN_0002e210's share of the held gun's Damage radius (1.7, 0x3a....): the too-close radius is
/// at least this much of it.
const BLAST_SHARE: f32 = 1.7;
/// ATTACK_FALLBACK's update (0x31765): done once the distance is over the radius plus this (5.0,
/// [0x3a4fd4]).
const DONE_BEYOND: f32 = 5.0;
/// FUN_00029a00's link walk: the cone's cosine 0.6 (0x3f19999a, 53 degrees for a unit direction)
/// and the length 7.0 (0x40e00000) the legs must pass.
const WALK_COS: f32 = 0.6;
const WALK_LENGTH: f32 = 7.0;
/// The walk update's arrival radius (0x29920: brain+0x4f8, which the brain's constructor sets to
/// 1.5, 0x3fc00000, at 0x1693e; ATTACK_VARY's lateral draws set it to 0.6, #26 part C).
const WALK_RADIUS: f32 = 1.5;
/// The pace record's +0x24 the fallback's update sets (0x31899: 0x3dcccccd, 0.1): each notch of the
/// ramp takes this long (`Router::step_time`).
pub const STEP_TIME: f32 = 0.1;
/// The walk's mode (0x299d0: FUN_0004d970(brain, dir, 3, 1, 1): a run, held to the pace level, with
/// the pace near the route's end).
const WALK_MODE: i32 = 3;
/// MOTION types the back and side steps pick between (the MT_ name table, 0x3bfdac): the run's
/// side steps MOVE_LEFT / MOVE_RIGHT (26 / 30) and their backward ones (27 / 31), the walk's
/// MOVE_*_SLOW (29 / 33) and MOVE_*_BACK_SLOW (28 / 32), BACKPEDAL (3) and BACKSTEP (4).
const MT_MOVE_LEFT: i64 = 26;
const MT_MOVE_LEFT_BACK: i64 = 27;
const MT_MOVE_RIGHT: i64 = 30;
const MT_MOVE_RIGHT_BACK: i64 = 31;

/// A character's fallback state (`Goals::fallback`): the re-arm timer of EVT_TGT_TOO_CLOSE (brain
/// +0x118) and the event while it waits for the brain to take it, and the goal's walk when it runs.
#[derive(Clone, Debug, Default)]
pub struct State {
    /// brain+0x118 with +0x11c bit 0: seconds until the sense step may post again; 0 armed. The demo
    /// counts it down inside `run` only, so it waits while PATH_TOWARD_HOME or the stuck
    /// ATTACK_GUARD have the character (the game's sense step runs every frame; negligible: it only
    /// delays the next post).
    rearm: f32,
    /// EVT_TGT_TOO_CLOSE is in the brain's queue and not taken yet, taken once no hit reaction runs
    /// (GOAL_STUN and GOAL_KNOCKDOWN are not interrupted); see `State::queued` for the rest of its
    /// time there
    posted: bool,
    /// the walk away, while the goal runs
    pub walk: Option<Walk>,
}

/// A walk away from the target: which goal it is, and on a map without world points where the demo's
/// straight walk ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Walk {
    pub goal: i64,
    end: Option<Vec3>,
}

impl Walk {
    /// The goal's name, for the log and BF_GOAL_WATCH.
    pub fn name(&self) -> &'static str {
        if self.goal == GOAL_MOVE_BACK { "MOVE_BACK" } else { "ATTACK_FALLBACK" }
    }
}

impl State {
    /// The fight is over (no contact left): the timer and the walk dropped (the caller resets the
    /// route).
    pub fn fight_over(&mut self) {
        *self = State::default();
    }

    /// Whether the brain's queue holds an EVT_TGT_TOO_CLOSE, which makes the sense step skip its post
    /// (the walk over the queue at 0x44c8b-0x44cbd finds an event 0x55): one posted and not taken yet,
    /// and **the one whose goal ATTACK_FALLBACK is running**. The second is inferred, not read to its
    /// end: the goal machine takes the front event once and, while it is the running goal's own, only
    /// refreshes the goal's copy of it (FUN_0003abe0 at 0x3b3bc-0x3b437, FUN_00022e90), FUN_0003b580
    /// does not pop it when the goal starts (only a NOOP draw, FUN_0003aa10, and a draw that every
    /// goal refused, FUN_0003b750 -> FUN_00016150, do), and the entry's decay is 0, so the priority
    /// never takes it out. take07 agrees: its breakpoint on the post (0x44cc0) logged 9.60 and 11.19
    /// game seconds and then nothing until 22.25 (Flint), although from 12.69 to 13.1 s the Ugly stood
    /// 2.19 to 3.05 m from Tex with the timer out. A MOVE_BACK walk (not started by the event) holds
    /// nothing.
    fn queued(&self) -> bool {
        self.posted || self.walk.is_some_and(|w| w.goal == GOAL_ATTACK_FALLBACK)
    }
}

/// What a frame of the goal came to for the caller (play_brain.rs `think`, play_goals.rs
/// `npc_fight`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Out {
    /// not on: the attack goal goes on
    Off,
    /// it walks away facing the target (`look`: its place; a MOVE_BACK with no target has none)
    /// and fires at it when it sees it: the caller faces and fires, as the update's FUN_00024f80
    /// at 25 degrees does
    Walking { look: Option<Vec3> },
}

/// Whether BF_BRAIN_LOG is set.
fn log_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_BRAIN_LOG").is_ok())
}

/// BF_TEST_FALLBACK_REFUSE=1: ATTACK_FALLBACK's start refuses as if the link walk found no 7 m way
/// (the staging for the draw's second goal, ATTACK_COVER: a level's geometry rarely gives a refusal
/// where a cover point is to be had; e23's ledge is a dead end, but its Ugly has no cover point in
/// its home area).
fn refuse_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_TEST_FALLBACK_REFUSE").is_ok())
}

/// FUN_0002e210, the too-close radius of `m` (EAX the brain): the larger of its type's `<ai>`
/// h_f1dbdb99 (the AI settings' +0x70) and BLAST_SHARE x the Damage radius of the gun it holds
/// (weapon data +0x24; none without a gun).
pub fn radius(m: &Player, game: &Game) -> f32 {
    let setting = game.character_type(m.kind).map_or(0.0, |t| t.ai.too_close);
    let blast = m.loaded.as_ref().and_then(|l| l.weapons.get(m.weapon)).map_or(0.0, |w| w.def.blast_radius);
    setting.max(BLAST_SHARE * blast)
}

/// The sense step's test (0x44c4a-0x44c62): the squared 3D distance `d2` between the two characters'
/// position records under the radius squared.
fn too_close(d2: f32, r: f32) -> bool {
    d2 < r * r
}

/// ATTACK_FALLBACK's end test (0x31765-0x31795): the squared distance over (radius + DONE_BEYOND)^2.
fn done(d2: f32, r: f32) -> bool {
    d2 > (r + DONE_BEYOND) * (r + DONE_BEYOND)
}

/// A level character's brain as the fallback meets it (play_cover.rs, #384): its cover fields
/// (brain+0x510..) and the frame's cover context (`think`'s `cover_ctx`: its target, the enemies it
/// knows, its home area). The cover goal's hook keeps EVT_TGT_TOO_CLOSE while the cover holds
/// (`cover::takes`, FUN_000313e0), the fallback's start ends the cover goal (`cover::end`, 0x2f900:
/// the point let go, the crouch cleared), and the entry's second goal, ATTACK_COVER, starts the
/// cover (`cover::start`). A test map NPC has no brain to hold a cover point: `None`.
pub struct Held<'a, 'c> {
    pub cover: &'a mut cover::Cover,
    pub ctx: &'a cover::Ctx<'c>,
}

/// One frame of the fallback (and MOVE_BACK) for `m`, a character in a fight whose target is `foe`
/// (None: the brain has none, which fails the goal): `g` the level's points (None on the test map's
/// floor), `personality` its brain's (or the NPC's), `everyone` the frame's characters (where each
/// is), `label` its name for the log, `held` its brain's cover goals (None for a test map NPC). In
/// the order of the game's AI update: the re-arm timer runs and the sense step posts
/// EVT_TGT_TOO_CLOSE (`post`: not while its cover goal holds, whose hook keeps the event), the goal
/// machine takes the event (`take`), and the goal running updates (`walk`). `Out::Walking` says the
/// caller fires and faces the target this frame.
#[allow(clippy::too_many_arguments)]
pub fn run(m: &mut Player, mut g: Option<&mut Graph>, game: &Game, personality: u32, foe: Option<Foe>, everyone: &[follow::Lead], label: &str,
           mut held: Option<Held>, dt: f32) -> Out {
    // (the demo moves an enemy type by its drive; a squad type's brain, a test map NPC of one, has
    // none, so a walk would move nothing: it stands and fires as before, play_goals.rs `go`)
    if !enemies::enemy(m) {
        return Out::Off;
    }
    let r = radius(m, game);
    if let Some(f) = foe {
        m.goals.note_foe(f.who, f.at.distance(m.position));
    }
    let kept = held.as_ref().is_some_and(|h| cover::takes(h.cover, EVT_TGT_TOO_CLOSE));
    post(m, r, everyone, label, dt, kept);
    if m.goals.fallback.posted && !enemies::reacting(m) {
        m.goals.fallback.posted = false;
        take(m, g.as_deref_mut(), game, personality, foe, label, held.as_mut());
    }
    let Some(w) = m.goals.fallback.walk else { return Out::Off };
    // (a hit reaction takes the character over: its goal is not the walk's any more)
    if enemies::reacting(m) {
        end(m, g, label, "a hit reaction takes over", false);
        return Out::Off;
    }
    // (ATTACK_FALLBACK's update needs a target, the brain's else the goal's: none fails it, and it
    // is done beyond its radius + 5; MOVE_BACK's walk asks for none)
    if w.goal == GOAL_ATTACK_FALLBACK {
        let Some(foe) = foe else {
            end(m, g, label, "no target", true);
            return Out::Off;
        };
        if done(foe.at.distance_squared(m.position), r) {
            let d = foe.at.distance(m.position);
            end(m, g, label, &format!("its target is {d:.1} m off, over {:.1} m (the too-close radius {r:.1} + {DONE_BEYOND})", r + DONE_BEYOND), true);
            return Out::Off;
        }
    }
    let look = foe.map(|f| f.at);
    if step(m, g.as_deref_mut(), w, look, dt) {
        Out::Walking { look }
    } else {
        end(m, g, label, "the route's end", w.goal == GOAL_ATTACK_FALLBACK);
        Out::Off
    }
}

/// The stuck handling pushed GOAL_ATTACK_GUARD (play_goals.rs `stuck_guard`, FUN_00042620), which
/// marks the goal that ran done: a walk away ends, and the caller does not call `run` while the
/// guard stands (it has no target to give the fallback, and an event posted meanwhile would only be
/// refused for want of one, drawing for nothing).
pub fn guarded(m: &mut Player, g: Option<&mut Graph>, label: &str) {
    if m.goals.fallback.walk.is_some() {
        end(m, g, label, "the stuck handling's ATTACK_GUARD takes over", false);
    }
}

/// The sense step's post (FUN_000447b0 at 0x44b57-0x44cc0): the re-arm timer runs; once it is out, a
/// contact seen now (a character, +0x80 bits 5 and 2) whose position record is within the radius
/// `r` of `m`'s (3D) sets the timer to REARM and posts EVT_TGT_TOO_CLOSE, unless one is queued
/// already (`State::queued`: one posted and not taken yet, or the one whose ATTACK_FALLBACK still
/// walks; the timer is set either way, 0x44c68-0x44c85 before the queue is looked at). The first
/// contact in the brain's map that passes does it, whoever the brain's target is. A post goes to
/// the brain's message handler (0x3dce0), which asks the running goal's hook first (FUN_0003d890):
/// `kept`, the cover goal's (FUN_000313e0 keeps EVT_TGT_TOO_CLOSE while the cover holds,
/// `cover::takes`), swallows it, so nothing is queued and nothing drawn, and the next test, 1.5 s on,
/// goes through again. take07 (F2b): the post at 9.60 s drew nothing, the cover failed on its 8 m rule,
/// the one at 11.19 s drew ATTACK_FALLBACK.
fn post(m: &mut Player, r: f32, everyone: &[follow::Lead], label: &str, dt: f32, kept: bool) {
    let me = m.position;
    let fb = &mut m.goals.fallback;
    fb.rearm = (fb.rearm - dt).max(0.0);
    if fb.rearm > 0.0 {
        return;
    }
    let near = m.sensor.seen_now_who().filter_map(|w| everyone.iter().find(|l| l.who == w)).find(|l| too_close(l.at.distance_squared(me), r));
    let Some(near) = near else { return };
    let fb = &mut m.goals.fallback;
    fb.rearm = REARM;
    if fb.queued() {
        if fb.walk.is_some() && log_on() {
            println!("t={:.2} brain {label}: EVT_TGT_TOO_CLOSE (85) not posted: {:?} {:.2} m off, within {r:.1} m, but one is queued while its ATTACK_FALLBACK walks",
                     m.sim_time, near.who, near.at.distance(me));
        }
        return;
    }
    if kept {
        if log_on() {
            println!("t={:.2} brain {label}: EVT_TGT_TOO_CLOSE (85): {:?} {:.2} m off, within {r:.1} m: kept by its cover goal's hook (FUN_000313e0), nothing drawn",
                     m.sim_time, near.who, near.at.distance(me));
        }
        return;
    }
    fb.posted = true;
    if log_on() {
        println!("t={:.2} brain {label}: EVT_TGT_TOO_CLOSE (85): {:?} {:.2} m off, within {r:.1} m", m.sim_time, near.who, near.at.distance(me));
    }
}

/// The goal machine takes EVT_TGT_TOO_CLOSE (FUN_0003b580): the personality's entry, which has to
/// have a priority over the fight's (an entry at -1 is NOOP); its draw (FUN_00054c40) by weight;
/// ATTACK_FALLBACK starts (`start`), and one that refuses is struck out and the draw made again
/// (FUN_00054bc0): which comes to the entry's second goal, ATTACK_COVER (weight 0, so only then),
/// whose start is the cover pick (`cover::start`, 0x2f8e0: a brain with its cover context; a test
/// map NPC's refuses): started, the cover goal runs as the character's attack goal from here
/// (`Goals::attack`, as `think`'s own draw leaves it), and when it is over the attack goal is drawn
/// again; refused, struck out in its turn, and the draw is spent. The goals the demo doesn't have
/// (the Supermutant's pound and the Shadoon's) do nothing: the attack goal goes on.
fn take(m: &mut Player, mut g: Option<&mut Graph>, game: &Game, personality: u32, foe: Option<Foe>, label: &str, mut held: Option<&mut Held>) {
    let Some(entry) = game.personality_entry(personality, EVT_TGT_TOO_CLOSE).filter(|e| e.priority > FIGHT_PRIORITY) else { return };
    let mut refused: Vec<i64> = vec![];
    loop {
        let u = m.random(1_000_000) as f32 / 1_000_000.0;
        let Some(goal) = entry.draw(u, &refused) else { return };
        if goal == GOAL_ATTACK_FALLBACK {
            if start(m, g.as_deref_mut(), foe, GOAL_ATTACK_FALLBACK, label, held.as_mut().map(|h| &mut *h.cover)) {
                return;
            }
        } else if goal == GOAL_ATTACK_COVER {
            // (a start that refuses leaves the goal beneath as it was: `cover::start` resets the cover
            // fields before its pick, which would end a DEBUG1 running ATTACK_VARY and its 5 s timer)
            let started = held.as_mut().is_some_and(|h| {
                let before = h.cover.clone();
                let ok = cover::start(h.cover, m, g.as_deref_mut(), GOAL_ATTACK_COVER, h.ctx);
                if !ok {
                    *h.cover = before;
                }
                ok
            });
            if started {
                m.goals.fight_paused();
                m.goals.attack = Some(GOAL_ATTACK_COVER);
                if log_on() {
                    println!("t={:.2} brain {label}: EVT_TGT_TOO_CLOSE draws ATTACK_COVER (the fallback refused): its cover goal runs (play_cover.rs)", m.sim_time);
                }
                return;
            }
        } else {
            if log_on() {
                println!("t={:.2} brain {label}: EVT_TGT_TOO_CLOSE draws goal {goal}: not run by the demo", m.sim_time);
            }
            return;
        }
        refused.push(goal);
    }
}

/// GOAL_ATTACK_FALLBACK's start (0x31610) or GOAL_MOVE_BACK's (0x29a80), both FUN_00029a00: a route
/// of the link walk (`Graph::away`: cos 0.6, length 7.0) from the character's place (its position
/// and own point), the route record reset first and kept reset when none is found; found, the move's
/// heading memory is dropped (brain+0x3c4 bit 14, which FUN_00019af0 takes as "start the move
/// afresh") and the route is walked from here on (FUN_00025730(route, 1, 1)). The fallback's
/// direction is me - the target (the position records, 3D: `Graph::away` takes its x and z, and the
/// vector as it is, not normalised: its length across scales the dot, so a target 1.2 m off widens
/// the cone to cos^-1 (0.6 / 1.2) = 60 degrees a side, 53 being a unit direction's, and one 0.6 m
/// off or less leaves no link at all), MOVE_BACK's the character's backward axis (unit). It needs
/// the target for the fallback (FUN_00049ca0, else the start returns); the demo's straight walk on a
/// floor without points (`g` None) is the same 7 m along that direction. Whether it started.
///
/// `cover`: the brain's cover fields, when it has a brain (#384). A fallback that starts while the
/// cover goal is up (take07's Ugly was crouched at its cover point when it fell back, the cover
/// failed on its 8 m rule and DEBUG1 running ATTACK_VARY, its point still claimed) ends the cover
/// with it (`cover::end`, 0x2f900): the point let go, the crouch cleared, the cover's route reset.
/// It ends before the walk's route starts (it resets the route record), and only once the walk is
/// found: a start that refuses leaves the cover as it was.
pub fn start(m: &mut Player, g: Option<&mut Graph>, foe: Option<Foe>, goal: i64, label: &str, cover: Option<&mut cover::Cover>) -> bool {
    let log = log_on();
    // (BF_TEST_MOVE_BACK reaches here without `run`'s check of the type)
    if !enemies::enemy(m) {
        if log {
            println!("t={:.2} brain {label}: {} refused: the demo's walk moves enemy types only, a squad type has no drive", m.sim_time, Walk { goal, end: None }.name());
        }
        return false;
    }
    if goal == GOAL_ATTACK_FALLBACK && refuse_on() {
        if log {
            println!("t={:.2} brain {label}: ATTACK_FALLBACK refused: BF_TEST_FALLBACK_REFUSE", m.sim_time);
        }
        return false;
    }
    let dir = if goal == GOAL_MOVE_BACK {
        Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos())
    } else {
        let Some(foe) = foe else {
            if log {
                println!("t={:.2} brain {label}: ATTACK_FALLBACK refused: no target", m.sim_time);
            }
            return false;
        };
        m.position - foe.at
    };
    let name = Walk { goal, end: None }.name();
    let across = Vec2::new(dir.x, dir.z);
    let walk = match g {
        Some(g) => {
            let place = Place::character(m.position, m.router.own.point);
            match g.away(place, dir, WALK_COS, WALK_LENGTH, None) {
                Some(entries) => {
                    // (the cover goal beneath ends first: its end resets the route record, which the
                    // walk's route then takes over)
                    if let Some(c) = cover {
                        cover::end(c, m, Some(&mut *g));
                    }
                    routes::start(m, g, entries, name);
                    Some(Walk { goal, end: None })
                }
                None => {
                    if m.router.own.point.is_some() {
                        routes::stop(m, g);
                    }
                    if log {
                        println!("t={:.2} brain {label}: {name} refused: no link walk of {WALK_LENGTH} m within {WALK_COS} of its direction (me - target {:.2} m across, own point {})",
                                 m.sim_time, across.length(), m.router.own.point.map_or("none".to_string(), |p| format!("h_{:08x}", g.nodes[p as usize].name)));
                    }
                    None
                }
            }
        }
        None => {
            // (the demo's: a floor with no world points, so no link to walk: 7 m along the direction,
            // the end set WALK_RADIUS farther so the walk is over once it has covered them)
            let along = across.normalize_or_zero();
            if along == Vec2::ZERO {
                if log {
                    println!("t={:.2} brain {label}: {name} refused: no direction", m.sim_time);
                }
                None
            } else {
                if let Some(c) = cover {
                    cover::end(c, m, None);
                }
                let end = m.position + Vec3::new(along.x, 0.0, along.y) * (WALK_LENGTH + WALK_RADIUS);
                m.router.stop = Some(end);
                if log {
                    println!("t={:.2} brain {label}: {name}: a straight walk of {WALK_LENGTH} m on a floor with no world points, to ({:.1}, {:.1})", m.sim_time, end.x, end.z);
                }
                Some(Walk { goal, end: Some(end) })
            }
        }
    };
    let Some(walk) = walk else { return false };
    // (the attack goal is interrupted: its approach and pursuit start again after, the goal's own
    // brain+0x398 timers are the ATTACK_VARY draw's: #26 part C. The approach's stop point,
    // brain+0x1b4, is zeroed by the AI update before every goal, 0x1b2c5: the pace near the end
    // goes by this route's, but for the demo's straight walk, which sets its own)
    m.goals.fight_paused();
    if walk.end.is_none() {
        m.router.stop = None;
    }
    m.goals.fallback.walk = Some(walk);
    m.router.avoid.dir_was = Vec2::ZERO;
    m.router.avoid.dir_now = Vec2::ZERO;
    if log {
        println!("t={:.2} brain {label}: {name} starts{}{}", m.sim_time, foe.map_or(String::new(), |f| format!(": {:.1} m from its target", f.at.distance(m.position))),
                 if goal == GOAL_ATTACK_FALLBACK { ", the line fe56cd97 not said (#36)" } else { "" });
    }
    true
}

/// The goal is over: its route reset (FUN_0004e8f0, the goal's end 0x316c0 -> 0x28e70), the pace
/// record's mode, and for the fallback the attack goal drawn again (`done`: the update's
/// return 2 / 0 ends the goal and the personality's EVT_ENEMY_SEEN draws anew).
fn end(m: &mut Player, g: Option<&mut Graph>, label: &str, why: &str, redraw: bool) {
    if log_on() {
        println!("t={:.2} brain {label}: {} over: {why}", m.sim_time, m.goals.fallback.walk.map_or("the fallback", |w| w.name()));
    }
    m.goals.fallback.walk = None;
    m.router.stop = None;
    if let Some(g) = g {
        routes::stop(m, g);
    }
    goals::stand(m);
    if redraw {
        m.goals.redraw_attack();
    }
}

/// The walk update (0x316d0's tail into 0x29920) for `walk` after the target at `target`: the pace
/// record's +0x24 = STEP_TIME (the fallback's update sets it every frame; MOVE_BACK's start doesn't,
/// its own update never touches it), the route walked at WALK_RADIUS (FUN_00026d80 with brain+0x4f8),
/// steered and moved at WALK_MODE held to the pace level, with the pace near the route's end, the
/// motion picked against the view toward `target` (`goals::go_facing`). Whether the walk goes on
/// (false: the route's end, or none: the update returns done / fails). The demo's straight walk on a
/// floor without points ends within WALK_RADIUS of its end.
fn step(m: &mut Player, g: Option<&mut Graph>, walk: Walk, target: Option<Vec3>, dt: f32) -> bool {
    if walk.goal == GOAL_ATTACK_FALLBACK {
        m.router.step_time = STEP_TIME;
    }
    match (g, walk.end) {
        (Some(g), _) => {
            if !m.router.walking() {
                return false;
            }
            let s = routes::step(m, g, WALK_RADIUS, WALK_RADIUS);
            if !s.walking {
                return false;
            }
            goals::go_facing(m, Some(&*g), s.target, Move::held(WALK_MODE), target, true, dt);
        }
        (None, Some(end)) => {
            if m.position.xz().distance(end.xz()) <= WALK_RADIUS {
                return false;
            }
            goals::go_facing(m, None, end, Move::held(WALK_MODE), target, true, dt);
        }
        (None, None) => return false,
    }
    true
}

/// FUN_001195c0's motion for a walk that keeps its gaze on `face` (a place): the move `way` (the
/// steered direction) against the view toward `face` (`avoid::pick`, the demo's body as the view's
/// stand-in for it) is forward (the pace's own motion `by_pace`: MT_WALK or MT_RUN), a side step, or
/// backing off; each at the walk or the run by `by_pace`, and where the type's MOTION table lacks
/// the run's variant the walking one stands for it, which is take07's Ugly: no BACKPEDAL, so its
/// back step is BACKSTEP at its walk speed whatever the pace, and no MOVE_*_BACK, so MOVE_*_BACK_SLOW.
/// A pick that gives no command (the body turning) keeps the motion it has. Also whether the pick is
/// the backward case (char+0x800 = 0), which takes 0.8 off the speed (`Drive::back`).
pub fn facing_mt(m: &Player, by_pace: i64, way: Vec2, face: Vec3) -> (i64, bool) {
    let to = face - m.position;
    let view = avoid::yaw_of(Vec2::new(to.x, to.z));
    let pick = avoid::pick(view, m.yaw, avoid::yaw_of(way), true, false);
    let has = |mt: i64| m.loaded.as_ref().is_some_and(|l| l.motions.clip(mt, 0, true).is_some());
    let mt = motion_for(pick.motion, by_pace, has, m.enemy.drive.map(|d| d.mt));
    (mt.unwrap_or(by_pace), !pick.forward)
}

/// `facing_mt`'s mapping from FUN_001195c0's command (`pick`) to a motion type: forward is the pace's
/// own (`by_pace`); a side step or a back step is the run's variant when `by_pace` is a run (or the
/// dash) and the table has it (`has`: the type's MOTION table), else the walking one, and at the
/// walk the walking one first; no command keeps the motion it has (`current`). None when the table
/// has neither of the two (the caller keeps the pace's own).
fn motion_for(motion: avoid::Motion, by_pace: i64, has: impl Fn(i64) -> bool, current: Option<i64>) -> Option<i64> {
    let running = by_pace != enemies::MT_WALK;
    let first = |c: [i64; 2]| c.into_iter().find(|&mt| has(mt));
    use avoid::Motion::*;
    match motion {
        Run => Some(by_pace),
        Side { left } => match (left, running) {
            (true, true) => first([MT_MOVE_LEFT, enemies::MT_MOVE_LEFT_SLOW]),
            (false, true) => first([MT_MOVE_RIGHT, enemies::MT_MOVE_RIGHT_SLOW]),
            (true, false) => first([enemies::MT_MOVE_LEFT_SLOW, MT_MOVE_LEFT]),
            (false, false) => first([enemies::MT_MOVE_RIGHT_SLOW, MT_MOVE_RIGHT]),
        },
        Back => if running { first([enemies::MT_BACKPEDAL, enemies::MT_BACKSTEP]) } else { first([enemies::MT_BACKSTEP, enemies::MT_BACKPEDAL]) },
        SideBack { left } => match (left, running) {
            (true, true) => first([MT_MOVE_LEFT_BACK, enemies::MT_MOVE_LEFT_BACK_SLOW]),
            (false, true) => first([MT_MOVE_RIGHT_BACK, enemies::MT_MOVE_RIGHT_BACK_SLOW]),
            (true, false) => first([enemies::MT_MOVE_LEFT_BACK_SLOW, MT_MOVE_LEFT_BACK]),
            (false, false) => first([enemies::MT_MOVE_RIGHT_BACK_SLOW, MT_MOVE_RIGHT_BACK]),
        },
        Keep => current,
    }
}

/// BF_TEST_MOVE_BACK=<s>[,<name>[,<x>,<z>]]: at s seconds (the controlled character's clock) a level
/// character with a brain (not following) whose name contains `name` (any when it is empty or
/// missing), the nearest to (x, z) when they are given, starts GOAL_MOVE_BACK, which nothing draws
/// yet (ATTACK_VARY's RUN_BACK / WALK_BACK are #26 part C's): the same walk as the fallback's, 7 m
/// behind it by the link walk, on a map with points. It faces and fires at its target as
/// ATTACK_VARY's state 6 does, with one, else it walks with no look.
fn move_back_hook(player: Res<Player>, mut squad: ResMut<Squad>, mut nav: ResMut<super::nav::Nav>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(spec) = std::env::var("BF_TEST_MOVE_BACK") else {
        *done = true;
        return;
    };
    let mut parts = spec.split(',').map(str::trim);
    let Some(at) = parts.next().and_then(|s| s.parse::<f32>().ok()) else {
        println!("BF_TEST_MOVE_BACK: \"{spec}\" isn't <s>[,<name>[,<x>,<z>]]: skipped");
        *done = true;
        return;
    };
    if player.sim_time < at {
        return;
    }
    let name = parts.next().unwrap_or("").to_ascii_lowercase();
    let near = parts.next().and_then(|x| x.parse::<f32>().ok()).zip(parts.next().and_then(|z| z.parse::<f32>().ok())).map(|(x, z)| Vec2::new(x, z));
    let found = squad.0.iter_mut().filter(|m| !m.dead && !m.absent && m.brain.as_ref().is_some_and(|b| !b.follows()))
        .filter(|m| unit_name(m).to_ascii_lowercase().contains(&name))
        .min_by(|a, b| near.map_or(0.0, |p| a.position.xz().distance(p)).total_cmp(&near.map_or(0.0, |p| b.position.xz().distance(p))));
    let Some(m) = found else {
        return;
    };
    *done = true;
    let label = unit_name(m);
    // (a cover goal it holds ends with it, `start`'s `cover`: the brain is out of the character for
    // the call, as `think` has it)
    let mut brain = m.brain.take();
    start(m, nav.graph.as_mut(), None, GOAL_MOVE_BACK, &label, brain.as_mut().map(|b| &mut b.cover));
    m.brain = brain;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x44c4a-0x44c62: a seen contact strictly inside the radius is too close; at it, not.
    #[test]
    fn too_close_is_inside_the_radius() {
        assert!(too_close(8.99, 3.0));
        assert!(!too_close(9.0, 3.0));
        assert!(!too_close(25.0, 3.0));
    }

    /// 0x44c8b-0x44cbd: the sense step posts nothing while the queue holds an 85: one posted and not
    /// taken yet, or the one whose ATTACK_FALLBACK walks (take07: no post from 12.69 to 13.1 s with
    /// the Ugly inside 3 m and the timer out); a MOVE_BACK walk, which the event did not start,
    /// holds nothing.
    #[test]
    fn the_event_stays_queued_while_its_fallback_walks() {
        let walk = |goal| Some(Walk { goal, end: None });
        let mut s = State::default();
        assert!(!s.queued());
        s.posted = true;
        assert!(s.queued());
        s.posted = false;
        s.walk = walk(GOAL_ATTACK_FALLBACK);
        assert!(s.queued());
        s.walk = walk(GOAL_MOVE_BACK);
        assert!(!s.queued());
        s.walk = None;
        assert!(!s.queued());
    }

    /// 0x31765-0x31795: the fallback is done strictly beyond the radius + 5 m (8 m for the Red Hand
    /// and the mutants).
    #[test]
    fn the_fallback_is_done_five_metres_beyond() {
        assert!(!done(64.0, 3.0));
        assert!(done(64.1, 3.0));
        assert!(!done(0.0, 3.0));
        // a gun with a blast radius widens both: 1.7 x 4 = 6.8 m, done beyond 11.8 m
        let r = 3.0f32.max(BLAST_SHARE * 4.0);
        assert!(too_close(46.0, r) && !too_close(46.5, r));
        assert!(!done(11.8 * 11.8 - 0.1, r) && done(11.8 * 11.8 + 0.1, r));
    }

    /// The motion for a pick: the mutants' table (BACKSTEP, the four MOVE_*_SLOW, MOVE_LEFT / RIGHT,
    /// MOVE_*_BACK_SLOW; no BACKPEDAL, no MOVE_*_BACK, take07's Ugly) backs off at BACKSTEP whatever
    /// the pace; a type with both gets the run's at a run.
    #[test]
    fn the_back_step_is_the_walking_one_where_there_is_no_back_run() {
        use avoid::Motion::*;
        let mutant = |mt: i64| matches!(mt, 4 | 26 | 28 | 29 | 30 | 32 | 33 | 34 | 47);
        let full = |_: i64| true;
        let (walk, run) = (enemies::MT_WALK, enemies::MT_RUN);
        assert_eq!(motion_for(Back, run, mutant, None), Some(enemies::MT_BACKSTEP));
        assert_eq!(motion_for(Back, walk, mutant, None), Some(enemies::MT_BACKSTEP));
        assert_eq!(motion_for(Back, run, full, None), Some(enemies::MT_BACKPEDAL));
        assert_eq!(motion_for(Back, walk, full, None), Some(enemies::MT_BACKSTEP));
        assert_eq!(motion_for(SideBack { left: true }, run, mutant, None), Some(enemies::MT_MOVE_LEFT_BACK_SLOW));
        assert_eq!(motion_for(SideBack { left: false }, run, full, None), Some(MT_MOVE_RIGHT_BACK));
        assert_eq!(motion_for(SideBack { left: false }, walk, full, None), Some(enemies::MT_MOVE_RIGHT_BACK_SLOW));
        assert_eq!(motion_for(Side { left: true }, run, mutant, None), Some(MT_MOVE_LEFT));
        assert_eq!(motion_for(Side { left: false }, walk, mutant, None), Some(enemies::MT_MOVE_RIGHT_SLOW));
        assert_eq!(motion_for(Run, run, mutant, None), Some(run));
        // no command keeps the motion; a table with neither back step has none
        assert_eq!(motion_for(Keep, run, mutant, Some(enemies::MT_BACKSTEP)), Some(enemies::MT_BACKSTEP));
        assert_eq!(motion_for(Back, run, |mt| mt == run, None), None);
    }
}
