//! The front end's bot squads (#420, the demo's): the MATCH SETUP screen's BOT SQUADS row
//! (play_multiplayer_menu.rs `Setup`, carried as `Rules::bots` in play_match.rs) puts up to three
//! more squads into a match, each on its own team, out to fight the player's squad and each other.
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
//! Test hook: `BF_MATCH`'s third value (play_match.rs `Rules::parse`), `"0.5,10,2"` for two
//! squads.

use super::*;

/// The characters in a bot squad: the game's four to a squad (`SQUAD`), one of the squad's own
/// types each, in `CHARACTERS`' order.
pub const SQUAD_SIZE: usize = SQUAD;

/// How many bot squads the front end may add: one to each of the table's TEAM1-TEAM3.
pub const SQUADS_MAX: usize = damage::BOT_TEAMS.len();

/// Put `bots` squads (up to SQUADS_MAX) into the match about to start: each of SQUAD_SIZE
/// characters of the squad's types, on team `damage::BOT_TEAMS[s]`, set to fight, at the map's
/// start points after the player's own. Called by `begin_play` before the map plays: their
/// bodies, weapons and clips come from `spawn_player` (`spawn_unit`), as any character's.
pub fn spawn(squad: &mut Squad, bots: usize) {
    let starts = world::arena().map_or_else(Vec::new, |a| a.starts.clone());
    let mut id = 0;
    for (s, &team) in damage::BOT_TEAMS.iter().enumerate().take(bots.min(SQUADS_MAX)) {
        for j in 0..SQUAD_SIZE {
            let mut b = Player::new(j % SQUAD);
            b.team = team;
            b.npc = Some(testtools::Npc { id: 1 + id as u32, fight: true, face: 0.0, follow: false });
            b.surface = usize::MAX;
            place(&mut b, &starts, 1 + s * SQUAD_SIZE + j);
            squad.0.push(b);
            id += 1;
        }
        println!("bots: squad {} of {bots}: {SQUAD_SIZE} characters on team {team}", s + 1);
    }
}

/// Puts bot `b` at the map's start point `n`, a start point a character as `player_at_start` puts
/// the player (its feet on the ground there), facing its way; with no start points, in the flat
/// floor's formation at place `n`.
fn place(b: &mut Player, starts: &[(Vec3, f32)], n: usize) {
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
