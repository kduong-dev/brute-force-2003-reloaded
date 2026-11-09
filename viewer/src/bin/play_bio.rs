//! The bioreactive death's vaporise (#147): the bio effect at the body and the invisible carrier
//! the game leaves there, whose explosion is the blast that chains.
//!
//! A BIOREACTIVE (type 3) hit that kills vaporises the victim (FUN_001252b0; its no-ragdoll,
//! no-corpse half is #366's: `Player::vaporised`, `ragdoll::hide_vaporised`). Two things follow,
//! and this module does them:
//!
//! * **The effect.** The type **h_eecb39f9** at the body: ALE bio-spray, bio-shrap, bio-flash and
//!   bio-spines_master, with the light light_hit_bio (common/objecttypes-common.xml line 677, so
//!   every level loads it). FUN_001252b0 starts it at the character's transform (vtable +0x40) and
//!   gives it the user parameter min(1, 150 / max HP), which the demo's effects don't take.
//! * **The carrier and its blast.** 0.2 s later the invisible carrier goes off: the
//!   inventory-grenade **h_0569a8b4** (its `timer` 0.2, its explosion h_1f4a5a0a), which FUN_001252b0
//!   places at the body and arms, owned by the record's attacker, its factor word the victim's
//!   max HP. So the blast is **1.1-1.5 x the body's max HP, BIOREACTIVE, radius 7 m**, and a body it
//!   kills is vaporised in turn and leaves its own carrier: **the chain**, 0.25-0.45 s a link
//!   (todo/147-bioreactive/leads.md §3-§5, shotlist.md A3-A4).
//!
//! The blast is play_missile.rs's `deal_blasts` (a blast object as FUN_0022bfc0 makes it: the
//! Damage core rule's taper, the cover ray, the difficulty step and the victim's type factor, so
//! a mutant's BIOREACTIVE x0 leaves it untouched), pushed with `MissileState::carrier`. The carrier
//! is only placed when the level loads the explosion's type: m03_a has the barrels and the effect
//! and no carrier, and vaporises with the effect alone (shotlist.md E1).
//!
//! **Not here** (#147's next slice): the bio build-up at char+0x554 and its decay, and with it the
//! rule for a body already dead (a type-3 hit on a corpse vaporises it once the build-up passes
//! half its max HP), which needs hits on corpses the demo doesn't deal; and the effect's user
//! parameter. The guns' and the item's drop is any death's already.
//!
//! **The build-up** (char+0x554, #419): every type-3 hit adds what the body took after every
//! factor (`take_hit`; 0x1257dc), FUN_00125210's fall takes 0.5 a second off it for the squad and
//! 0.15 for an NPC's, clamped to 0..max HP, and a vaporise zeroes it. The game's only reader, the
//! corpse rule (a body already dead vaporises once it passes half its max HP), is the next slice.
//!
//! **Its sound** is the blast's, not the vaporise's: as FUN_0022cea0 makes the blast it plays the
//! explosion's impact sound (**h_f421f43e**, the weapon's `h_e5618348`, leads.md §3), which the
//! demo's `go_off` does for every other blast. A carrier never goes through `go_off`, so
//! `take_vaporise` queues the sound with the same fuse. The death itself has no voice line
//! (leads.md §2: the code shows no sound at t0), so `take_hit` skips both the killing hit's hurt
//! line and the death's lines for a vaporised body, and a line it was already saying is cut off
//! on the frame the body goes (`Chatter::voice_stop`, FUN_00143570).
//!
//! Test hook `BF_BIO_LOG=1`: each body vaporised, the effect it plays, and the carrier's blast;
//! `BF_MISSILE_LOG=1` adds the blast's own pass over the bodies.

use super::*;

/// The vaporise effect type h_eecb39f9 (common/objecttypes-common.xml line 677, in every level):
/// the bio spray, shrapnel, flash and spines with a green light, no sound and no damage area.
/// FUN_001252b0 starts it at the body (0x12533d-0x12537e).
const VAPORISE: u32 = 0xEECB_39F9;
/// The carrier's explosion weapon type h_1f4a5a0a (WTYPE_DATA, play_shots.rs's `WTYPE_DATA`): Damage
/// 1.1-1.5 BIOREACTIVE, radius 7 m, h_011cb154 true (the whole roll within half the radius, then
/// (1 - 2 (d / r - 0.5))^2 of it to the edge: the shot list's A3 numbers, 192.5-262.5 at 3.5 m
/// down to 3.9-5.4 at 6.5 m for a 125 HP body). The data reuses the iKhan-GPL's name and
/// description for it (#55).
const BLAST: u32 = 0x1F4A_5A0A;
/// Seconds from the vaporise to the blast (s): the carrier's `timer`, 0.2, plus the blast object's
/// own delay (0.05 + 0.2 U, FUN_0022bfc0 at 0x22c0b7), of which the demo takes the low end: the
/// chain's links land 0.25-0.45 s apart in the game (todo/147-bioreactive/leads.md §5).
const FUSE: f32 = 0.25;

/// The bio build-up's fall for a player's squad (s^-1): FUN_00125210 subtracts dt x 0.5 (0x3a4f74)
/// every frame, x0.3 more (0x3a50b0) for a squad without a player (`NPC_DECAY`), so a squad's is
/// 0.5 a second and an NPC's 0.15 (#419).
const SQUAD_DECAY: f32 = 0.5;
/// The bio build-up's fall for a squad without a player (#419): 0.5 x 0.3.
const NPC_DECAY: f32 = 0.15;

/// The bio build-up after `dt` s of FUN_00125210's fall: `value` less `rate` x `dt`, clamped to
/// 0..`max` (the game's clamp is to 0..max HP, char+0x50).
pub(super) fn decay(value: f32, rate: f32, dt: f32, max: f32) -> f32 {
    (value - rate * dt).clamp(0.0, max)
}

/// A character's fall rate: the squad's 0.5 a second, an NPC's 0.15. The game asks whether the
/// character's squad (char+0x59c) has a player (FUN_0005c0a0); the demo's test NPCs stand in for a
/// squad without one.
pub(super) fn rate(u: &Player) -> f32 {
    if u.npc.is_some() { NPC_DECAY } else { SQUAD_DECAY }
}

/// One frame's fall of `u`'s build-up (`decay`), logging the moment it empties under `BF_BIO_LOG`.
pub(super) fn tick(u: &mut Player, dt: f32, log: bool) {
    if u.bio <= 0.0 {
        return;
    }
    let before = u.bio;
    u.bio = decay(u.bio, rate(u), dt, u.max_health);
    // (a line for each whole point it loses, and for the last of it: how the fall shows in a run)
    if log && (before.floor() > u.bio.floor() || u.bio <= 0.0) {
        println!("t {:.2}: {} bio {:.1} of {:.0}", u.sim_time, unit_name(u), u.bio, u.max_health);
    }
}

pub fn plugin(app: &mut App) {
    // (before the chatter's update, so a voice the vaporise cuts off stops on that frame)
    app.add_systems(Update, take_vaporise.after(update_player).before(chatter::ChatterSystems).run_if(in_state(AppState::Playing)));
}

/// The carrier blast's roll: the explosion's Damage roll (FUN_0022c940 at 0x22bfc0, `damage_roll`)
/// x the vaporised body's max HP, the carrier's factor word (+0x1de, the game's (int) max HP): so
/// 1.1-1.5 x max HP for the carrier's weapon as written, whatever the body. `draw` is the game's
/// one random number for the roll.
fn carrier_roll(def: &WeaponDef, max_hp: f32, draw: f32) -> f32 {
    missile::damage_roll(def, draw) * max_hp
}

/// The vaporised bodies' vaporise, taken once each (`Player::vaporise`, set by the hit that
/// killed): the effect at the body, then the carrier's blast `FUSE` s later, owned by the killer,
/// dealt by play_missile.rs's `deal_blasts`. Bodies the same frame's blast kills set their own,
/// taken on the frames after, so the chain runs a link a frame.
fn take_vaporise(
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    game: Res<GameData>,
    mut state: ResMut<missile::MissileState>,
    mut spawns: ResMut<effects::EffectSpawns>,
    mut chatter: ResMut<chatter::Chatter>,
    mut seed: Local<u32>,
) {
    let log = std::env::var("BF_BIO_LOG").is_ok();
    // (the bodies that died this frame; the hit lists come out of the characters first, since the
    // blast's team is another character's)
    let mut bodies: Vec<(Vec3, Option<Who>, f32, u8)> = vec![];

    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if let Some((at, killer, max_hp)) = u.vaporise.take() {
            // (the game zeroes char+0x554 as it vaporises, 0x1255a9)
            let bio = u.bio;
            u.bio = 0.0;
            // the body goes with its voice: a line it was saying is cut off where it is (the game's
            // stop, FUN_00143570), and the death asks no new one (take_hit)
            if u.voice.is_some() {
                if log {
                    println!("  {} mid-line: its voice stops with the body", unit_name(u));
                }
                chatter.voice_stop(u);
            }
            bodies.push((at, killer, max_hp, u.team));
            if log {                println!("t {:.2}: {} vaporised at {at:.2}, max HP {max_hp:.0}, bio {bio:.1} -> 0, killer {}", u.sim_time,
                         unit_name(u), killer.map_or("none".to_string(), |w| format!("{w:?}")));
            }
        }
    }

    if bodies.is_empty() {
        return;
    }

    let blaster = game.0.weapons.get(&BLAST);
    for (at, killer, max_hp, own_team) in bodies {
        // the killer's side, for FUN_0004b9c0's x0.7 on its own bodies (`Hit::by`); the body's
        // when the killer isn't in the world, the blast's when a barrel's cloud has none
        let team = killer
            .and_then(|w| {
                std::iter::once(&*player)
                    .chain(squad.0.iter())
                    .find(|k| k.who() == w)
                    .map(|k| k.team)
            })
            .unwrap_or(own_team);

        spawns.0.push(effects::EffectSpawn::new(VAPORISE, at));

        let Some(def) = blaster else {
            if log {
                println!("  no carrier in this level (no h_{BLAST:08x}): the effect alone");
            }
            continue;
        };

        if max_hp <= 0.0 {
            continue;
        }

        let roll = carrier_roll(def, max_hp, shots::random(&mut seed));
        let id = state.carrier(FUSE, at, BLAST, roll, killer, team);
        // the blast's sound, when the game makes the blast (FUN_0022cea0): the explosion's impact
        // sound, h_f421f43e on this weapon
        state.sound_after(FUSE, def.impact_sound, effects::blast_volume(at, player.position));
        if log {
            println!("  carrier {id}: h_{BLAST:08x} roll {roll:.2} ({max_hp:.0} HP), {FUSE:.2} s from {at:.2}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The carrier's factor is the body's max HP and the weapon's Damage rolls 1.1-1.5: a 125 HP
    /// body's blast is 137.5-187.5 before the victim's factors, which on STANDARD is 192.5-262.5
    /// for an enemy (x1.4) and 82.5-112.5 for the squad (x0.6), the shot list's A3 / A5 numbers.
    #[test]
    fn the_carrier_rolls_the_weapon_times_the_bodys_max_hp() {
        let def = WeaponDef { damage: 1.5, damage_min: 1.1, ..Default::default() };
        assert!((carrier_roll(&def, 125.0, 0.0) - 137.5).abs() < 1e-4);
        assert!((carrier_roll(&def, 125.0, 1.0) - 187.5).abs() < 1e-4);
        assert!((carrier_roll(&def, 125.0, 0.5) - 162.5).abs() < 1e-4);
        // a bigger body's blast is bigger, the roll the same
        assert!((carrier_roll(&def, 173.0, 0.0) - 190.3).abs() < 1e-3);
    }

    /// The build-up falls at the game's two rates and stays within 0..max HP (#419, FUN_00125210).
    #[test]
    fn the_bio_build_up_falls_at_the_games_rates() {
        // 0.5 a second for a player's squad, 0.15 for a squad without a player
        assert!((decay(10.0, SQUAD_DECAY, 2.0, 20.0) - 9.0).abs() < 1e-6);
        assert!((decay(10.0, NPC_DECAY, 2.0, 20.0) - 9.7).abs() < 1e-5);
        // it stops at 0, and never sits above max HP
        assert_eq!(decay(0.3, SQUAD_DECAY, 1.0, 20.0), 0.0);
        assert_eq!(decay(19.9, 0.0, 1.0, 20.0), 19.9);
        assert_eq!(decay(25.0, 0.0, 1.0, 20.0), 20.0);
    }
}

