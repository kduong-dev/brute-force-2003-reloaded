//! Match rules and the match's clock (#420, the demo's): the time limit, the score limit, the
//! character and the bot squads the front end's MATCH SETUP screen sets (play_multiplayer_menu.rs
//! `Setup`), carried into the map as `Rules` and run here (the bot squads are play_bots.rs').
//!
//! The game has no such screen on the demo's path: its DEATHMATCH and SQUAD DEATHMATCH panels
//! (game-options h_1854a812, h_ec00d7fa) lead to the mode and a map, and a match's length and score
//! on an Xbox live in the session's own tables, which #420 didn't read. So the screen, its values
//! and the rules below are the demo's, the product owner's choices, and they do only what the
//! demo can see:
//!  - **The clock** runs on the app's frame time while a multiplayer map plays. Not the game's
//!    (`Player::sim_time`, which stops with the world in a cutscene and under the menu's cover):
//!    a match's length is the front end's, as the menu's own settings are.
//!  - **A kill** is a character of another team (`Player::team`: 0 the squad's) that was alive at
//!    the last frame and isn't now. The demo keeps no kill record and #420 traced none of the
//!    game's own score, so a kill counts for the player's side whoever fired it: a stand-in, a
//!    guess. The squad's own deaths don't count.
//!  - **A credited kill** (`BF_MATCH_KILLS`, the test hook) is one the match starts with, so a
//!    score limit can be reached with no enemy to kill it: a multiplayer map places none.
//!  - **At a limit** the match ends as Backspace does (`back_to_menu`, play.rs): the front end, on
//!    that map's SELECT MISSION (`menu::Return`), after one line in the log.
//!  - **A round** (squad deathmatch): when one squad's characters are all dead, the next round
//!    starts at once: both squads come back at the map's start points, a fresh character each, the
//!    tally kept (the kills counted so far, and the clock). The demo's reading of the game's rule
//!    (#420 traced none); a deathmatch has no round (one character, and no squad against it).
//!
//! Only a map the front end's MATCH SETUP started has rules (`start_loading` inserts them: a start
//! with `BF_MAP`, the test tools' switches or the map editor plays as it did), and only while a
//! multiplayer map is played (`damage::multiplayer_map`): a mission's map never sees them.

use std::collections::HashMap;

use super::*;

/// The rules the front end set for the match about to start (play_menu.rs `start_loading`, from
/// the MATCH SETUP screen's choices). `Default`: an open match with no limit and no character
/// picked (one started any other way).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct Rules {
    /// the time limit (s), None for none
    pub time_limit: Option<f32>,
    /// the score limit (kills), None for none
    pub score_limit: Option<u32>,
    /// the squad slot the player controls (`CHARACTERS` index), None for `start_character`'s own
    pub character: Option<usize>,
    /// the bot squads the match puts up (play_bots.rs), 0 for none
    pub bots: usize,
    /// the two teams' squads: each a place in the front end's squad list (play_menu.rs
    /// `Menu::squads`: the game's five, then the demo's three; play_bots.rs `team_types`), the
    /// player's team then the other's; None for no squad (a deathmatch: its join page picks the
    /// single character)
    pub squads: [Option<usize>; 2],
}

impl Rules {
    /// A limit in minutes (the screen's choice) in seconds.
    pub fn minutes(m: u32) -> Option<f32> {
        Some(m as f32 * 60.0)
    }

    /// Whether it sets a limit (a character alone doesn't make a match of it).
    pub fn any(&self) -> bool {
        self.time_limit.is_some() || self.score_limit.is_some()
    }

    /// Whether a match at `clock` (s) with `kills` has reached one of `self`'s limits, and which:
    /// the time one first when both have (`update` ends the match on it either way).
    pub fn reached(&self, clock: f32, kills: u32) -> Option<End> {
        if self.time_limit.is_some_and(|t| clock >= t) {
            Some(End::Time)
        } else if self.score_limit.is_some_and(|s| kills >= s) {
            Some(End::Score)
        } else {
            None
        }
    }

    /// `BF_MATCH="<minutes>,<kills>"`, the test hook: a time limit in minutes (a fraction for a
    /// short match) and a score limit, `-` or empty for none (`"0.05,-"` is three seconds and no
    /// score limit). The front end doesn't set it; `start_loading` inserts the screen's rules.
    pub fn parse(spec: &str) -> Rules {
        let mut parts = spec.split(',');
        let minutes = parts.next().unwrap_or_default().trim().parse::<f32>().ok().filter(|m| *m > 0.0);
        let score = parts.next().unwrap_or_default().trim().parse::<u32>().ok().filter(|s| *s > 0);
        let bots = parts.next().unwrap_or_default().trim().parse::<usize>().ok().unwrap_or(0);
        // (the two teams' squads, then: "1,5,0,1,2" is Brute Force's squad against the Feral
        // Hunting Pack's, as the squad page's rows pick them)
        let squads = [parts.next().and_then(|v| v.trim().parse().ok()), parts.next().and_then(|v| v.trim().parse().ok())];
        Rules { time_limit: minutes.map(|m| m * 60.0), score_limit: score, character: None, bots, squads }
    }

    /// The rules a start takes: `BF_MATCH`'s where it is set (a map started with `BF_MAP`), else
    /// none (`Default`).
    pub fn from_env() -> Rules {
        std::env::var("BF_MATCH").ok().map_or_else(Rules::default, |s| Rules::parse(&s))
    }
}

/// Which limit ended a match (`Rules::reached`), for the line `update` prints.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum End {
    Time,
    Score,
}

/// The match in progress on a multiplayer map.
#[derive(Resource, Default)]
pub struct Match {
    /// how long it has run (s, the app's frame time)
    pub clock: f32,
    /// the player's side's kills
    pub kills: u32,
    /// each character living at the last frame and its team, so a death is seen once
    alive: HashMap<Who, i64>,
    /// the round being played (squad deathmatch: 1 at the start, one more each time a squad is
    /// wiped out)
    pub round: u32,
    /// its limit was reached: it has ended (the menu is on its way)
    pub over: bool,
}

impl Match {
    /// A match starting with `BF_MATCH_KILLS`'s kills (the test hook), none by default.
    pub fn new() -> Match {
        let credited = std::env::var("BF_MATCH_KILLS").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
        Match { kills: credited, round: 1, ..Match::default() }
    }

    /// This frame's deaths: the characters `player` and `squad` have that were alive at the last
    /// frame and aren't now, those of another team (`player.team`) counted. `alive` is brought up
    /// to date, so a character seen dead never counts again and one not seen yet counts for
    /// nothing.
    fn deaths(&mut self, player: &Player, squad: Option<&Squad>) -> u32 {
        let mine = player.team as i64;
        let mut living: HashMap<Who, i64> = HashMap::new();
        for u in std::iter::once(player).chain(squad.into_iter().flat_map(|s| s.0.iter())) {
            // (one not in the world yet is no death when it goes: it was never seen alive)
            if u.dead || u.absent {
                continue;
            }
            living.insert(u.who(), u.team as i64);
        }
        let kills = self.alive.iter().filter(|(who, &team)| team != mine && !living.contains_key(who)).count() as u32;
        self.alive = living;
        kills
    }
}

/// Which side has nobody left (squad deathmatch's round's end): 0 the player's squad, 1 the one put
/// up against it, None while both have someone. A character out of the world (`absent`) or dead
/// counts for nobody; the player's side is its character and the squad's own slots, the other its
/// team's characters (`play_bots::spawn_enemy`'s, `damage::BOT_TEAMS[0]`).
fn wiped(player: &Player, squad: &Squad) -> Option<usize> {
    let alive = |p: &Player| !p.dead && !p.absent;
    let mut foes = squad.0.iter().filter(|m| m.team == damage::BOT_TEAMS[0]);
    if foes.clone().count() == 0 {
        return None; // no other squad was put up: nothing to wipe
    }
    let mine = alive(player) || squad.0.iter().any(|m| m.in_squad() && m.team == player.team && alive(m));
    let foe = foes.any(alive);
    match (mine, foe) {
        (true, true) => None,
        (false, _) => Some(0),
        (_, false) => Some(1),
    }
}

/// The next round (`wiped`): the player and both squads' characters made again at the map's start
/// points (`play_bots::place`), a fresh character of its type each (`testsquad::fresh`; its body
/// made by `spawn_player`), so the tally and the clock go on. The other squads a test hook adds
/// (`Rules::bots`) aren't brought back.
fn next_round(commands: &mut Commands, player: &mut Player, squad: &mut Squad) {
    let starts = world::arena().map_or_else(Vec::new, |a| a.starts.clone());
    let fresh = testsquad::fresh(player.character, player.kind, player, true);
    if let Some(l) = player.loaded.take() {
        commands.entity(l.root).despawn();
    }
    *player = fresh;
    if !starts.is_empty() {
        play_bots::place(player, &starts, 0);
    }
    let mut n = 1;
    for m in squad.0.iter_mut() {
        if !(m.in_squad() || m.team == damage::BOT_TEAMS[0]) {
            continue;
        }
        let mut fresh = testsquad::fresh(m.character, m.kind, m, false);
        if let Some(l) = m.loaded.take() {
            commands.entity(l.root).despawn();
        }
        fresh.npc = m.npc;
        if !starts.is_empty() {
            play_bots::place(&mut fresh, &starts, n);
        }
        *m = fresh;
        n += 1;
    }
}

/// The match's frame: its clock, its score, its rounds and its end at a limit (see the module
/// notes). It runs while a map plays; with no rules, on a map that isn't a multiplayer one, or once
/// it has ended, it does nothing.
#[allow(clippy::too_many_arguments)]
pub(super) fn update(time: Res<Time>, rules: Option<Res<Rules>>, data: Option<Res<GameData>>, map: Option<Res<CurrentMap>>,
                     mode: Option<Res<Mode>>, player: Option<ResMut<Player>>, squad: Option<ResMut<Squad>>, mut state: ResMut<Match>,
                     mut commands: Commands, mut next: ResMut<NextState<AppState>>) {
    let (Some(rules), Some(data), Some(map)) = (rules, data, map) else { return };
    if !rules.any() || state.over || !damage::multiplayer_map(&data.0, &map.0) {
        return;
    }
    let (Some(mut player), Some(mut squad)) = (player, squad) else { return };
    state.clock += time.delta_secs();
    state.kills += state.deaths(&player, Some(&squad));
    // (squad deathmatch's round: a squad with nobody left brings the next one on, the tally kept)
    if rules.squads[0].is_some() && rules.squads[1].is_some() && mode.as_ref().is_none_or(|m| !m.deathmatch) {
        if let Some(side) = wiped(&player, &squad) {
            state.round += 1;
            println!("match: round {}: {} wiped: the next round", state.round,
                     if side == 0 { "your squad" } else { "the other squad" });
            next_round(&mut commands, &mut player, &mut squad);
        }
    }
    let Some(end) = rules.reached(state.clock, state.kills) else { return };
    state.over = true;
    let kills = if state.kills == 1 { "kill" } else { "kills" };
    println!("match over: {} {} in {:.1} s (the {}): back to the menu", state.kills, kills, state.clock,
             match end { End::Time => "time limit", End::Score => "score limit" });
    let deathmatch = mode.as_ref().is_none_or(|m| m.deathmatch);
    commands.insert_resource(menu::Return { map: Some(map.0.clone()), deathmatch, main: false, editor: false });
    next.set(AppState::Menu);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A character of squad slot `c` on team `t`, alive.
    fn member(c: usize, t: u8) -> Player {
        let mut p = Player::new(c);
        p.team = t;
        p
    }

    /// A character of another team that dies is one kill, counted once; the squad's own deaths
    /// aren't, and one not seen alive before it dies counts for nothing.
    #[test]
    fn a_death_of_another_team_is_one_kill() {
        let me = member(3, 0);
        let foe = member(1, 1);
        let mut m = Match::default();
        let mut squad = Squad(vec![foe]);
        assert_eq!(m.deaths(&me, Some(&squad)), 0, "the first frame only sees them");
        squad.0[0].dead = true;
        assert_eq!(m.deaths(&me, Some(&squad)), 1, "a foe's death is a kill");
        assert_eq!(m.deaths(&me, Some(&squad)), 0, "counted once");
        // a teammate's death isn't one
        let mut mate = Squad(vec![member(2, 0)]);
        assert_eq!(m.deaths(&me, Some(&mate)), 0, "seen alive");
        mate.0[0].dead = true;
        assert_eq!(m.deaths(&me, Some(&mate)), 0, "the squad's own");
        // one already dead when first seen: nothing
        let mut gone = Squad(vec![member(1, 1)]);
        gone.0[0].dead = true;
        assert_eq!(m.deaths(&me, Some(&gone)), 0);
        // and the player's own death isn't a kill of theirs
        let mut dead_me = member(3, 0);
        dead_me.dead = true;
        assert_eq!(m.deaths(&dead_me, Some(&Squad::default())), 0);
    }

    /// A squad deathmatch's round is over when one squad has nobody left: the player's side is its
    /// character and the squad's slots, the other side its team's characters.
    #[test]
    fn a_squad_with_nobody_left_ends_the_round() {
        let me = member(3, 0);
        let mut squad = Squad(vec![member(2, 0), member(1, damage::BOT_TEAMS[0])]);
        assert_eq!(wiped(&me, &squad), None, "both sides stand");
        squad.0[0].dead = true;
        let mut dead_me = member(3, 0);
        dead_me.dead = true;
        assert_eq!(wiped(&dead_me, &squad), Some(0), "the player's side: its character and the squad's slots");
        assert_eq!(wiped(&me, &squad), None, "the player stands");
        squad.0[1].dead = true;
        assert_eq!(wiped(&me, &squad), Some(1), "the other squad's last character");
        assert_eq!(wiped(&dead_me, &squad), Some(0), "both: the player's side first");
    }

    /// The next round brings both squads back: a fresh character of its type each (no body yet:
    /// `spawn_player` makes it), full health and alive, the enemy still fighting; on the test's
    /// empty world nobody moves.
    #[test]
    fn the_next_round_brings_both_squads_back() {
        let mut world = World::new();
        let mut player = member(3, 0);
        player.dead = true;
        player.health = 0.0;
        let mut foe = member(1, damage::BOT_TEAMS[0]);
        foe.dead = true;
        foe.health = 0.0;
        foe.npc = Some(testtools::Npc { id: 9, fight: true, face: 0.0, follow: false });
        let mut squad = Squad(vec![member(2, 0), foe]);
        squad.0[0].dead = true;
        let mut commands = world.commands();
        next_round(&mut commands, &mut player, &mut squad);
        world.flush();
        assert!(!player.dead && player.health == player.max_health, "the player comes back");
        assert_eq!(player.kind, member(3, 0).kind, "its type kept");
        assert!(squad.0.iter().all(|m| !m.dead && m.health == m.max_health), "the squads come back");
        assert_eq!(squad.0[1].team, damage::BOT_TEAMS[0]);
        assert!(squad.0[1].npc.is_some_and(|n| n.id == 9 && n.fight), "the enemy still fights");
    }

    /// `BF_MATCH`'s two limits: minutes in seconds, `-` or 0 for none.
    #[test]
    fn bf_match_parses_its_two_limits() {
        assert_eq!(Rules::parse("1,5"), Rules { time_limit: Some(60.0), score_limit: Some(5), ..Rules::default() });
        assert_eq!(Rules::parse("0.5,-"), Rules { time_limit: Some(30.0), score_limit: None, ..Rules::default() });
        // (the two teams' squads, the squad page's picks)
        assert_eq!(Rules::parse("1,5,0,1,2").squads, [Some(1), Some(2)]);
        assert_eq!(Rules::parse("1,5").squads, [None, None]);
        assert_eq!(Rules::parse("0.5,10,2").bots, 2, "two bot squads (play_bots.rs)");
        assert_eq!(Rules::parse("0.5,10,x").bots, 0, "nothing it can read: none");
        assert!(!Rules::parse(",").any(), "neither: an open match");
        assert_eq!(Rules::parse("2,0").score_limit, None, "a 0 is no limit");
        assert_eq!(Rules::parse("x,y"), Rules::default(), "nothing it can read: none");
    }

    /// A score limit ends the match at its count, a time limit at its seconds, and neither where
    /// it isn't set (a match with only a score limit runs on whatever its clock says).
    #[test]
    fn either_limit_ends_a_match() {
        let both = Rules { time_limit: Some(30.0), score_limit: Some(2), ..default() };
        assert_eq!(both.reached(0.0, 0), None);
        assert_eq!(both.reached(10.0, 1), None);
        assert_eq!(both.reached(10.0, 2), Some(End::Score), "two kills: the score limit");
        assert_eq!(both.reached(30.0, 0), Some(End::Time), "half a minute: the time limit");
        assert_eq!(both.reached(30.0, 2), Some(End::Time), "both at once: the time one");
        let score_only = Rules { score_limit: Some(1), ..default() };
        assert_eq!(score_only.reached(9999.0, 0), None, "no time limit");
        assert_eq!(score_only.reached(0.0, 1), Some(End::Score));
        let time_only = Rules { time_limit: Some(2.0), ..default() };
        assert_eq!(time_only.reached(1.99, 99), None, "no score limit");
        assert_eq!(time_only.reached(2.0, 0), Some(End::Time));
    }

    /// A limit in minutes is seconds; no rules, no limit; a character alone sets no limit.
    #[test]
    fn rules_are_what_the_screen_set() {
        assert_eq!(Rules::minutes(10), Some(600.0));
        assert_eq!(Rules::minutes(0), Some(0.0));
        let open = Rules { character: Some(1), ..default() };
        assert!(!open.any());
        let limited = Rules { time_limit: Rules::minutes(15), score_limit: Some(10), ..default() };
        assert!(limited.any());
        assert_eq!(limited.time_limit, Some(900.0));
    }
}
