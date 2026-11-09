//! The front end's bot squads (#420, the demo's): the squad page's second team
//! (play_multiplayer_menu.rs `Teams`, carried as `Rules::squads` in play_match.rs) is a squad of
//! the game's own, put up to fight the player's, and `BF_MATCH`'s third value may add up to three
//! more (`Rules::bots`), each on a team of its own.
//!
//! The demo's, not the game's: a multiplayer map places no opponents (mp1's and sdm_e40's only
//! characters are the squad's four), and #420 traced no AI opponents in the game's own match. A
//! bot is a `Player` of the squad's four types with `Player::npc`'s fight AI — the test map's NPC
//! tools' (play_testtools.rs `Npc`, `npc_ai`), a brainless level character's goals (play_goals.rs
//! `npc_fight`) — pushed into `Squad`: `spawn_player` gives it its body, weapons and clips, and
//! `update_player` drives it as it drives a test map NPC.
//!
//!  - **Its team**: the team table's TEAM1-TEAM3 (3, 4, 5: damage::BOT_TEAMS), which the game's
//!    table makes hostile to each other already. The squad's team 0 has no row against them, so a
//!    bot match adds it (`damage::set_bot_match`, the demo's rule) while its map plays: the
//!    player's squad and the bots see each other as hostiles, the crosshair locks them, and a
//!    bot's death is the player's side's kill (play_match.rs `Match::deaths`).
//!  - **Where it starts**: the map's start points, one a character, after the player's own
//!    (`BF_START`'s), a squad's members on consecutive points. A map with too few points puts the
//!    rest in the flat floor's formation.
//!  - **What it does**: `npc_fight` walks it in to its gun's range and `npc_ai` aims and fires at
//!    a hostile in its sight (its type's sight distance and field of view), as a test map NPC set
//!    to fight: a squad stands where it starts until one of them sees an enemy.
//!
//! The squads are the game's own (game-options' `<squad>` list, the five the front end's squad
//! page offers first: Brute Force, Feral Hunting Pack, Shrike Raiders, Seer Coven, Ferguson
//! Platoon), each member the type its entry's character names in the squad's library, and the
//! demo's three after them (character_class::SQUADS: the Terrorist Cell, the Kingman Posse and the
//! Mutant Horde), each member the type of its seat (`team_types`).
//!
//! Test hook: `BF_MATCH`'s third value (play_match.rs `Rules::parse`), `"0.5,10,2"` for two extra
//! squads, and its fourth and fifth the two teams' squads (`team_types`: `"0.5,10,0,1,2"` with
//! Brute Force's against the Feral Hunting Pack's, the squad page's rows).

use super::*;

/// The characters in a bot squad: the game's four to a squad (`SQUAD`), one of the squad's own
/// types each, in `CHARACTERS`' order.
pub const SQUAD_SIZE: usize = SQUAD;

/// How many bot squads the front end may add: one to each of the table's TEAM1-TEAM3.
pub const SQUADS_MAX: usize = damage::BOT_TEAMS.len();

/// Put `bots` squads (up to SQUADS_MAX) into the match about to start: each of SQUAD_SIZE
/// characters of the squad's own types, on team `damage::BOT_TEAMS[s]`, set to fight, at the map's
/// start points after the player's own. Called by `begin_play` before the map plays: their
/// bodies, weapons and clips come from `spawn_player` (`spawn_unit`), as any character's.
pub fn spawn(squad: &mut Squad, bots: usize) {
    for (s, &team) in damage::BOT_TEAMS.iter().enumerate().take(bots.min(SQUADS_MAX)) {
        add(squad, &[None; SQUAD_SIZE], team, 1 + s * SQUAD_SIZE, 1 + s * SQUAD_SIZE);
        println!("bots: squad {} of {bots}: {SQUAD_SIZE} characters on team {team}", s + 1);
    }
}

/// Put the other team's squad into the match (`begin_play`): the SQUAD_SIZE types the squad page
/// picked for it (`team_types`), on `team`, set to fight, at the map's start points from `at` on
/// (after the player's and the extra bot squads').
pub fn spawn_enemy(squad: &mut Squad, types: &[Option<u32>; SQUAD_SIZE], team: u8, at: usize) {
    add(squad, types, team, at, at);
    println!("bots: the other team's squad: {SQUAD_SIZE} characters on team {team}");
}

/// A squad of `types` on `team` (None: the slot's own type, as the campaign squad's), at the map's
/// start points from `at` on, fighting, their NPC numbers from `id`.
fn add(squad: &mut Squad, types: &[Option<u32>; SQUAD_SIZE], team: u8, at: usize, id: usize) {
    let starts = world::arena().map_or_else(Vec::new, |a| a.starts.clone());
    for (j, kind) in types.iter().enumerate() {
        let mut b = Player::new(j % SQUAD);
        if let Some(k) = *kind {
            b.kind = k;
        }
        b.team = team;
        b.npc = Some(testtools::Npc { id: 1 + id as u32 + j as u32, fight: true, face: 0.0, follow: false });
        b.surface = usize::MAX;
        place(&mut b, &starts, at + j);
        squad.0.push(b);
    }
}

/// The four types the squad at `place` of the front end's squad list plays as (play_menu.rs
/// `Assets2::squads`): game-options' squad deathmatch squads in the file's order
/// (`Game::option_squads`: Brute Force, Feral Hunting Pack, Shrike Raiders, Seer Coven, Ferguson
/// Platoon), each member the type its `<squad>` entry's character names (`GameCharacter::squad_type`,
/// the type of that name in the squad's library: #339), then the demo's three squads of classes
/// (`character_class::SQUADS`: Terrorist Cell, Kingman Posse, Mutant Horde), each member the type
/// of its seat (`Squad::type_key`). A type the map hasn't loaded with its model keeps its slot's
/// own (the squad menu's rule: play_testsquad.rs `presets`).
pub fn team_types(game: &Game, place: usize) -> Option<[Option<u32>; SQUAD_SIZE]> {
    let loaded = |k: u32| game.character_type(k).is_some_and(|t| game.has_model(t));
    if let Some(s) = game.option_squads.get(place) {
        let mut types = [None; SQUAD_SIZE];
        for (c, kind) in types.iter_mut().enumerate() {
            *kind = s.members.get(c)
                .and_then(|a| game.option_characters.iter().find(|ch| ch.archetype == *a))
                .map(|ch| ch.squad_type)
                .filter(|&k| loaded(k));
        }
        return Some(types);
    }
    let s = bf_viewer::bf::character_class::SQUADS.get(place.checked_sub(game.option_squads.len())?)?;
    Some(std::array::from_fn(|c| s.members.get(c).map(|m| s.type_key(m)).filter(|&k| loaded(k))))
}

/// Puts bot `b` at the map's start point `n`, a start point a character as `player_at_start` puts
/// the player (its feet on the ground there), facing its way; with no start points, in the flat
/// floor's formation at place `n`.
pub(super) fn place(b: &mut Player, starts: &[(Vec3, f32)], n: usize) {
    let Some(&(at, yaw)) = starts.get(n % starts.len().max(1)) else {
        place_in_formation(b, None, n);
        return;
    };
    b.position = Vec3::new(at.x, at.y + 1.0 - GROUND, at.z);
    b.fall_from = at.y;
    b.yaw = yaw;
    b.cam_yaw = yaw;
    b.prev_xz = Vec2::new(at.x, at.z);
    if let Some(npc) = b.npc.as_mut() {
        npc.face = yaw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three squads are the game's four characters each, on the table's TEAM1, TEAM2 and TEAM3,
    /// all fighting and none of them a squad slot (they can't be handed control or walked into
    /// the formation), each with an NPC number of its own.
    #[test]
    fn bot_squads_are_four_fighting_characters_a_team() {
        let mut squad = Squad::default();
        spawn(&mut squad, 3);
        assert_eq!(squad.0.len(), 3 * SQUAD_SIZE);
        for (s, &team) in damage::BOT_TEAMS.iter().enumerate() {
            let of = squad.0.iter().filter(|b| b.team == team).count();
            assert_eq!(of, SQUAD_SIZE, "squad {} is on team {team}", s + 1);
        }
        assert!(squad.0.iter().all(|b| b.npc.is_some_and(|n| n.fight)), "every one fights");
        assert!(squad.0.iter().all(|b| !b.in_squad()), "none is a squad slot");
        let mut ids: Vec<u32> = squad.0.iter().filter_map(|b| b.npc.map(|n| n.id)).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 3 * SQUAD_SIZE, "a number each");
    }

    /// The other team's squad plays the types the squad page picked for it, on its team, fighting
    /// (play_bots.rs `spawn_enemy`).
    #[test]
    fn the_other_team_plays_the_types_it_is_given() {
        let mut squad = Squad::default();
        spawn_enemy(&mut squad, &[Some(7), Some(8), Some(9), Some(10)], damage::BOT_TEAMS[0], 1);
        assert_eq!(squad.0.len(), SQUAD_SIZE);
        assert_eq!(squad.0.iter().map(|b| b.kind).collect::<Vec<_>>(), [7, 8, 9, 10]);
        assert!(squad.0.iter().all(|b| b.team == damage::BOT_TEAMS[0] && b.npc.is_some_and(|n| n.fight)));
        assert!(squad.0.iter().all(|b| !b.in_squad()), "none is a squad slot");
    }

    /// As many squads as asked, up to the three teams; past them it stops at three.
    #[test]
    fn as_many_squads_as_asked() {
        let mut squad = Squad::default();
        spawn(&mut squad, 2);
        assert_eq!(squad.0.len(), 2 * SQUAD_SIZE);
        assert!(squad.0.iter().all(|b| b.team != damage::BOT_TEAMS[2]), "no third squad's team");
        let mut squad = Squad::default();
        spawn(&mut squad, SQUADS_MAX + 6);
        assert_eq!(squad.0.len(), SQUADS_MAX * SQUAD_SIZE);
    }
}
