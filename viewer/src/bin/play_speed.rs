//! How fast a squad member covers the ground in the run and the dash (#345).
//!
//! The game doesn't move a running character by its clip's root motion. FUN_0012c6c0 sets its
//! ground speed from the character type's `speeds` (crouch walk, walk, run, dash; `CharacterType::
//! speeds`) by the motion type, plays the run and dash clips at rate 1, and eases the velocity
//! toward the speed (4.5 a second, 7 slowing down). What multiplies the type's speed (char+0x720
//! and the callers' factors) depends on who drives the character:
//!
//!  - **A character with a human player** (FUN_00112800's test, repeated at 0x12cf6a-0x12cf8d in
//!    FUN_0012cd10, the ground step) calls FUN_0012c6c0 directly with char+0x720 = 1 (the movement
//!    reset FUN_0012b560 sets it at 0x12b61a, a player taking the character puts the brain's
//!    speed record back, FUN_00018150 at 0x181d3, and the brain's off update keeps it at 1.0,
//!    0x1a2a5): the type's speed as it stands.
//!    #264 take01 (Tex dashing east on e34, the game's clock): the velocity eases up to 8.45 m/s
//!    (type 8.5); take02 (Brutus) 8.7 (8.7); #129 take34 (a player's deathmatch Soldier, run 6.5
//!    dash 8.5): 6.49 and 8.44.
//!  - **Anyone else** goes through FUN_0012cb40, which multiplies char+0x720 by 0.8 (0x3a5014)
//!    for a type with h_e78efbb2 (type+0x1a8 bit 0: the four and the marines), and by 0.8 again
//!    while the byte char+0x800 is 0 (FUN_001195c0, 0x119b4e: moving backwards or sideways),
//!    around its call (0x12cbb4-0x12cc07). #33 take08 (Hawk's Move To, the game's clock): the
//!    dash levels at 6.84 m/s (type 8.5 x 0.8 = 6.8).
//!  - **The follow's boost**: the follow's update FUN_00020b10 sets the brain's speed record
//!    (brain+0x89c: flag +0x89d, value +0x8a0, timer +0x8a4) to 1.25 and active every update
//!    (0x20ece-0x20edb), and its chase state FUN_00022130 takes it back to 1.0 and off while the
//!    leader is within the follow's second distance (14 m in FOLLOW, 10 m in WINGMAN). The AI
//!    update FUN_0001afe0 hands the record to FUN_00048270 (at 0x1b3cd), which puts the value in
//!    char+0x720 while the pace level (the pace record at brain+0x1a8, +0: 1 stand, 2 walk, 3
//!    run, 4 dash) is above 2, a run or a dash alike, the value is at least 1, the record's timer
//!    has run out and the brain isn't crouched, else 1.0. So a member chasing a leader farther
//!    than that at a run or a dash moves at 0.8 x 1.25 = 1.0 x the type's speed for it, and
//!    inside it at 0.8 x: #242 take01 (Flint, Brutus and Hawk right after the release, the game's
//!    clock) 8.3-8.7 m/s on the first straight (types 8.6, 8.7, 8.5; the leader 48-50 m away);
//!    #33 take04 (the followers of Tex's dash) Brutus and Hawk hold 13.7-14.5 m behind him at 8-9
//!    m/s while farther and slow to 6.3-7.1 inside it. Move To (GOAL_MOVE_OUT) writes nothing to
//!    the record, and the follow goal's end puts it back to 1.0 and off (FUN_00020a10, 0x20a5c and
//!    0x20a62): the 0.8.
//!
//! **What isn't here.** The pace level is #335's (play_routes.rs `Router::cap`): the demo's starts
//! at the dash, the routes' stuck detours, the way round a friend and the follow's hold set it
//! down with their timers, and the ramp climbs it back (FUN_00058670: a notch every 0.17 s for a
//! squad member in FOLLOW or WINGMAN, 1.0 s otherwise); a member moves at the lower of it and the
//! move it asks, and the boost is keyed on that (`squad_ai`: 3 or 4). Left to #344: the ramp's
//! start from a stand and its step down after the move asked (the game's level follows it both
//! ways: walk, run, dash as take04's motion numbers go 47, 34, 8), and the feelers' drops
//! (FUN_00018c40: a dash hit takes the boost off for 1.5 s, the timer at brain+0x8a4, and the next
//! hit drops to a run for 1.5 s), a hard turn to a walk for 1 s and a door step to a stand for
//! 0.5 s (todo/344-ai-avoidance/leads.md §6). The follow's other states set the record too, and
//! the demo has neither: state 5 (FUN_00022790) 1.0 / off and then FUN_0002fba0's value near an
//! arrival, at most 1.0, which only a walk takes (the level 2 test: the slow-down on arrival; the
//! demo's walk is the clip's), and state 6 (FUN_000210c0) 1.5 / on, entered only with a route
//! point, so in WINGMAN (FUN_00020b10's row: FOLLOW never leaves the states 0-5). The members of
//! #242 take01 spent their route on the game's clock: 12-35% of their moving time at the boosted
//! dash (8.4-8.9 m/s), 20-29% at the run without the boost (6.2-6.3 = 0.8 x 7.8 / 7.9), 19-27% in
//! between (the climbs back up the ramp, the dash without the boost, 6.8-7.0) and 26-36% under
//! 5.9 (walks after the turns, corners): a mean over the moving time of 6.1 (Flint), 6.6 (Brutus)
//! and 5.9 (Hawk) m/s. A mean of that kind (the ticket's "6.5-6.9" is of that order) says nothing
//! of the dash on a straight, which is 8.5. `Player::speed_boost` is where the feelers' timer
//! lands.
//!
//! The demo moves the squad by root motion (the clips are looked up by name, the root decides
//! the step), so the run and the dash take the game's speed by scaling the clip's root step: each
//! layer of the crossfade by `target / the clip's own root speed` (`scale`), the animation itself
//! untouched (rate 1, as the game's). Only those two gaits, forward, and only the four squad types
//! (`enemies::enemy` ones keep the test hooks' drive, and the level characters their own
//! walks): the walk, the crouch walk, the back and side steps keep their clip's root (the game's
//! walking speed follows the stick; not done here), and so does a cutscene (scripted play moves
//! everyone by the root, FUN_0012cb40's other branch). The ease and the slide to a stop (4.5 and
//! 7 a second) aren't added: the crossfade's 0.2 s ramp stands for them, as before.
//!
//! Test hooks: `BF_SPEED_LOG=1` prints, every 0.25 s, each running or dashing character's target
//! (the type's speed, who drives it, the boost), the speed it covers and where it is;
//! `BF_ROOT_SPEED=1` moves the run and the dash by their clips' root, as before #345.

use bf_viewer::bf::character_type::CharacterType;

use super::*;

/// The squad types' factor for a character no human drives: FUN_0012cb40 (0x12cbca-0x12cbd6)
/// multiplies char+0x720 by the float at 0x3a5014 when the type has h_e78efbb2. (A second 0.8 for
/// moving backwards or sideways, char+0x800 = 0, isn't used: the demo's run and dash go forward.)
pub const AI_SQUAD_FACTOR: f32 = 0.8;

/// The follow's speed record while it chases the leader: 1.25 (0x3fa00000, stored at 0x20ed4 by
/// FUN_00020b10 every update), which FUN_00048270 puts in char+0x720 at any pace level above a
/// walk. 0.8 x 1.25 = 1.
pub const FOLLOW_BOOST: f32 = 1.25;

/// The leader's distance (m) inside which the follow's chase state takes the record back to 1.0
/// (FUN_00022130: the leader's distance on the follow's stack, brain-relative +4, at most the
/// second distance, +0xc, sets brain+0x8a0 = 1.0 and the flag off): 14 m in FOLLOW, Cover Me, the
/// squad's default (FUN_00020b10's 1.4 x set 11.2, 14, 9.8, 9.8, 5.6; 10 m in WINGMAN, scaled in
/// the demo by `orders::spacing`). #33 take04: Brutus and Hawk dash after Tex's dash at 13.7-14.5
/// m, 8-9 m/s while farther and 6.3-7.1 m/s inside it (0.8 x 8.7 = 6.96).
pub const FOLLOW_D1: f32 = 14.0;

/// The follow's speed record (`Player::speed_boost`) for a member moving at pace level `level`
/// (1 stand, 2 walk, 3 run, 4 dash: the lower of the move it asks and the level its drops leave
/// it, play_routes.rs `pace`) with its leader `gap` m away and `d1` the follow's second distance
/// (`FOLLOW_D1` x the mode's spacing): FOLLOW_BOOST beyond d1 at a run or a dash, 1.0 otherwise.
/// FUN_00020b10 sets 1.25 each update and FUN_00022130 takes it back to 1.0 while the gap is at
/// most d1 (0x221b6); FUN_00048270 hands the value to char+0x720 only at a level over 2, a run
/// as well as a dash (a walk takes only a value under 1.0: the arrival's slow-down, not done).
pub fn follow_boost(level: i32, gap: f32, d1: f32) -> f32 {
    if level > 2 && gap > d1 { FOLLOW_BOOST } else { 1.0 }
}

/// A locomotion clip's gait for the ground speed: the type's `speeds[2]` or `speeds[3]`
/// (FUN_0012c6c0: a running motion type, MT_RUN among them; MT_DASH).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Run,
    Dash,
}

/// The ground speeds (m/s) a character's run and dash move it at now.
#[derive(Clone, Copy, Debug)]
pub struct Targets {
    pub run: f32,
    pub dash: f32,
}

impl Targets {
    /// The speed of `kind`.
    pub fn of(&self, kind: Kind) -> f32 {
        match kind {
            Kind::Run => self.run,
            Kind::Dash => self.dash,
        }
    }
}

/// The ground speed (m/s) of `kind` for a character of type `t`: its `speeds` entry, x1 under a
/// human player (FUN_0012c6c0 called directly), else x AI_SQUAD_FACTOR for a type with
/// h_e78efbb2 (`CharacterType::player_type`), x `boost` (the brain's speed record, at least 1:
/// FUN_00048270 takes it for a run or a dash only from 1.0 up; FOLLOW_BOOST in the follow).
pub fn ground_speed(t: &CharacterType, kind: Kind, human: bool, boost: f32) -> f32 {
    let speed = match kind {
        Kind::Run => t.speeds[2],
        Kind::Dash => t.speeds[3],
    };
    if human {
        return speed;
    }
    speed * if t.player_type { AI_SQUAD_FACTOR } else { 1.0 } * boost.max(1.0)
}

/// BF_ROOT_SPEED=1 (test hook): the run and the dash move by their clips' root, as before #345,
/// to put a run beside the game's speeds.
fn root_speed() -> bool {
    static ROOT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ROOT.get_or_init(|| std::env::var("BF_ROOT_SPEED").is_ok())
}

/// The run and dash speeds of `p` now, None for a character they don't move (a type of its own,
/// whose gait the test hooks' drive and the level's brains set, play_enemies.rs; a cutscene, which
/// moves everyone by the clip's root; BF_ROOT_SPEED).
pub fn targets(p: &Player, game: &Game) -> Option<Targets> {
    if root_speed() {
        return None;
    }
    speeds_of(p, game)
}

/// `targets` without BF_ROOT_SPEED (for the log).
fn speeds_of(p: &Player, game: &Game) -> Option<Targets> {
    if enemies::enemy(p) || script::in_cutscene() {
        return None;
    }
    let t = game.character_type(p.kind)?;
    Some(Targets { run: ground_speed(t, Kind::Run, p.controlled, p.speed_boost), dash: ground_speed(t, Kind::Dash, p.controlled, p.speed_boost) })
}

/// The gait of the clip `p.gait` plays: Run for a run, Dash for a sprint (the demo's name for the
/// game's dash), None for the rest.
pub fn gait_kind(gait: Gait) -> Option<Kind> {
    match gait {
        Gait::Run => Some(Kind::Run),
        Gait::Sprint => Some(Kind::Dash),
        _ => None,
    }
}

/// Each run and dash clip of the slots' sets (carried and ready) with its gait and its own ground
/// speed, root displacement over the cycle (m/s, over `MIN_OWN` or left out). The same speed as
/// the clip's `AnimDef::move_speed` (h_f9628571) within 1%.
pub fn clip_speeds(model: &Character, game: &Game, slots: &[SlotClips; 2]) -> HashMap<usize, (Kind, f32)> {
    let mut out = HashMap::new();
    for set in slots.iter().flat_map(|s| [&s.carry, &s.rp]) {
        for (clip, kind) in [(set.run, Kind::Run), (set.sprint, Kind::Dash)] {
            let Some(c) = clip else { continue };
            let d = model.anims[c].duration.max(1e-3);
            let root = locomotion::root_delta(model, game, c, 0.0, d - 1e-3);
            let own = Vec2::new(root.x, root.z).length() / d;
            if own >= MIN_OWN {
                out.insert(c, (kind, own));
            }
        }
    }
    out
}

/// A clip with less ground speed of its own than this (m/s) isn't scaled (a stand-in's: dividing
/// by nothing).
const MIN_OWN: f32 = 0.5;

/// The factor on a layer's root step: the clip's ground speed taken to the game's. `top` is the
/// gait of the newest layer (`gait_kind`), which sets its own kind (a sprint with the gun up plays
/// the ready run, `Gait::Sprint => lo.sprint.or(lo.run)`: the dash's speed); the layers fading out
/// keep their clip's. 1 for any other clip.
pub fn scale(ground: &HashMap<usize, (Kind, f32)>, t: Targets, clip: usize, top: Option<Kind>) -> f32 {
    match ground.get(&clip) {
        Some(&(kind, own)) => t.of(top.unwrap_or(kind)) / own,
        None => 1.0,
    }
}

/// The speed plugin: the log, after the characters' step.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, log.after(update_player).run_if(in_state(AppState::Playing)));
}

/// One character's run of a gait for the log: the path it covered and the time it took, counted
/// from LOG_SKIP s into the gait (past the crossfade's ramp), every frame.
#[derive(Default)]
struct Track {
    kind: Option<Kind>,
    since: f32,
    last: Vec2,
    last_t: f32,
    path: f32,
    time: f32,
}

/// The ramp the log leaves out of a gait's mean speed (s): the crossfade's FADE and a little.
const LOG_SKIP: f32 = 0.5;

/// BF_SPEED_LOG: each frame the path every running or dashing character covers is counted (from
/// LOG_SKIP s into the gait); every 0.25 s of the controlled character's clock each prints who
/// drives it, the type's speed, the factor and the target, the speed it covers this frame
/// (`last_velocity`, the step over its time), where it is, and the mean speed of its gait so far.
fn log(player: Res<Player>, squad: Res<Squad>, game: Res<GameData>, mut last: Local<f32>, mut tracks: Local<HashMap<Who, Track>>) {
    if std::env::var("BF_SPEED_LOG").is_err() {
        return;
    }
    let t = player.sim_time;
    let print = (t * 4.0).floor() > (*last * 4.0).floor();
    *last = t;
    for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.dead && u.loaded.is_some()) {
        let Some(kind) = gait_kind(u.gait) else {
            tracks.remove(&u.who());
            continue;
        };
        let (Some(ty), Some(g)) = (game.0.character_type(u.kind), speeds_of(u, &game.0)) else { continue };
        let here = Vec2::new(u.position.x, u.position.z);
        let k = tracks.entry(u.who()).or_default();
        if k.kind != Some(kind) {
            *k = Track { kind: Some(kind), since: u.sim_time, last: here, last_t: u.sim_time, ..Default::default() };
        }
        if u.sim_time > k.last_t {
            if u.sim_time - k.since > LOG_SKIP {
                k.path += (here - k.last).length();
                k.time += u.sim_time - k.last_t;
            }
            k.last = here;
            k.last_t = u.sim_time;
        }
        if print {
            let speed = kind_speed(ty, kind);
            println!("t={t:.2} {} ({}): {} {speed:.2} m/s x {:.2} -> {:.2}, covers {:.2} m/s at ({:.2}, {:.2}), mean {:.2} m/s over {:.2} s", unit_name(u),
                     if u.controlled { "player" } else { "AI" }, if kind == Kind::Run { "run" } else { "dash" },
                     g.of(kind) / speed.max(1e-3), g.of(kind), Vec2::new(u.last_velocity.x, u.last_velocity.z).length(), u.position.x, u.position.z,
                     if k.time > 0.0 { k.path / k.time } else { 0.0 }, k.time);
        }
    }
}

/// The type's own speed of `kind`, for the log.
fn kind_speed(t: &CharacterType, kind: Kind) -> f32 {
    match kind {
        Kind::Run => t.speeds[2],
        Kind::Dash => t.speeds[3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tex's type (objecttypes-common): speeds 3 3.6 7.8 8.5, h_e78efbb2.
    fn tex() -> CharacterType {
        CharacterType { speeds: [3.0, 3.6, 7.8, 8.5], player_type: true, ..Default::default() }
    }

    #[test]
    fn a_human_moves_at_the_types_speed() {
        assert_eq!(ground_speed(&tex(), Kind::Run, true, 1.0), 7.8);
        assert_eq!(ground_speed(&tex(), Kind::Dash, true, 1.0), 8.5);
        // (nothing of the brain's reaches a character a human drives)
        assert_eq!(ground_speed(&tex(), Kind::Dash, true, FOLLOW_BOOST), 8.5);
    }

    #[test]
    fn an_ai_squad_member_moves_at_four_fifths() {
        // #33 take08: Hawk's Move To dash levels at 6.84 (8.5 x 0.8 = 6.8)
        assert!((ground_speed(&tex(), Kind::Dash, false, 1.0) - 6.8).abs() < 1e-4);
        // #242 take01: Brutus and Hawk run at 6.2-6.3 (7.8 and 7.9 x 0.8)
        assert!((ground_speed(&tex(), Kind::Run, false, 1.0) - 6.24).abs() < 1e-4);
    }

    #[test]
    fn the_follows_dash_is_the_types() {
        // #242 take01 and #33 take04: the followers' dash on a straight is the type's 8.5-8.7
        assert!((ground_speed(&tex(), Kind::Dash, false, FOLLOW_BOOST) - 8.5).abs() < 1e-4);
        // (a record under 1 is not taken for a run or a dash)
        assert!((ground_speed(&tex(), Kind::Dash, false, 0.5) - 6.8).abs() < 1e-4);
    }

    #[test]
    fn the_follows_boost_is_at_a_run_or_a_dash_beyond_d1() {
        // FUN_00048270 takes the record at a pace level over 2: a run as well as a dash
        assert_eq!(follow_boost(4, 20.0, FOLLOW_D1), FOLLOW_BOOST);
        assert_eq!(follow_boost(3, 20.0, FOLLOW_D1), FOLLOW_BOOST);
        // a walk or a stand takes none
        assert_eq!(follow_boost(2, 20.0, FOLLOW_D1), 1.0);
        assert_eq!(follow_boost(1, 20.0, FOLLOW_D1), 1.0);
        // FUN_00022130 clears it while the leader is within d1, at d1 included
        assert_eq!(follow_boost(4, FOLLOW_D1, FOLLOW_D1), 1.0);
        assert_eq!(follow_boost(3, 10.0, FOLLOW_D1), 1.0);
        // WINGMAN's d1 is 10 m (14 x 8 / 11.2)
        assert_eq!(follow_boost(4, 12.0, FOLLOW_D1 * 8.0 / 11.2), FOLLOW_BOOST);
        // so a run beyond d1 moves at the type's own run speed: 0.8 x 1.25 x 7.8
        assert!((ground_speed(&tex(), Kind::Run, false, follow_boost(3, 20.0, FOLLOW_D1)) - 7.8).abs() < 1e-4);
    }

    #[test]
    fn a_type_without_the_bit_has_no_factor() {
        let hound = CharacterType { speeds: [0.0, 2.0, 5.5, 11.42], player_type: false, ..Default::default() };
        assert_eq!(ground_speed(&hound, Kind::Dash, false, 1.0), 11.42);
    }

    #[test]
    fn a_layer_is_scaled_by_its_clips_own_speed() {
        let mut ground = HashMap::new();
        ground.insert(7, (Kind::Run, 4.8));
        ground.insert(9, (Kind::Dash, 8.0));
        let t = Targets { run: 7.8, dash: 8.5 };
        assert!((scale(&ground, t, 7, None) - 7.8 / 4.8).abs() < 1e-5);
        assert!((scale(&ground, t, 9, None) - 8.5 / 8.0).abs() < 1e-5);
        // the newest layer of a sprint that plays the ready run moves at the dash's speed
        assert!((scale(&ground, t, 7, Some(Kind::Dash)) - 8.5 / 4.8).abs() < 1e-5);
        // any other clip keeps its root
        assert_eq!(scale(&ground, t, 3, None), 1.0);
    }
}
