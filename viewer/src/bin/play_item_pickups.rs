//! The walk-over's item path for the pickups that put something in the squad's store: the ammo
//! boxes (#18) and, in the commits after it, the grenade packs (#19) and the OrgSen (#21). Read
//! from default.xbe (the addresses are in the comments and in viewer/docs/code-map.md; the reading
//! is todo/18-ammo-pickups/leads.md's) and checked against that folder's takes 01-10 (sdm_e34 and
//! sdm_e13, xemu): the amounts, the lines, the full stack, the AI squadmate and the respawns.
//!
//! A pickup lying in the world (an `inventory-object`, a level's or a spawn-trigger's copy) is
//! taken by walking over it, the same FUN_0011c540 -> FUN_0011c170 that takes the medkits, the
//! fruit, the DNA canisters, the power-ups and the guns' rounds (play_pickups.rs,
//! play_powerups.rs, play_weapon_pickups.rs): every living member of a player's squad is tried
//! each frame, the AI's too, by its place against the pickup's: at most 1.8 m below and 1.0 m
//! above the character's position and under 1.34 m across (`powerups::in_reach`). Then:
//!
//!  - the pickup's set says whether it can be taken (+0x38): for the ammo boxes, the packs and
//!    the OrgSen it always says yes (0x280040, `mov al, 1`), so nothing refuses a box because no
//!    squad gun uses the ammo (take03: a Rail box taken by a squad with no Rail gun and no Rail
//!    stack: "20x Rail Ammo", the stack took its type) or because the stack is full;
//!  - `pickup-amount` of the type it gives (its pickup-archetype, else itself: FUN_00146340) go
//!    into the squad's store (FUN_0014ce90 with its multiplier flag set, FUN_0014bfe0): the stack
//!    of that type's key, held to its type's stack-limit; what doesn't fit is lost (take02: a
//!    stack at 195 of 200 and a box of 25 give "5x Particle Ammo", and the box is gone);
//!  - **nothing fits** (the stack full): the line "<name> (full)" in red, and the pickup stays
//!    where it is, taken again every frame the character stands in reach (take02, take05, take09:
//!    the line stays up for as long as he stands there). It goes to the HUD of the player who
//!    controls the character alone (FUN_0011c6a0 with AL clear): an AI squadmate standing on a
//!    full box shows nothing (take08, 14-22 s);
//!  - **something fitted**: its pickup sound at the character (FUN_0014a490 -> FUN_00146190:
//!    h_e4097f62 for the ammo, the packs and the OrgSen), PICKUP_OBJECT from the pickup (the
//!    spawn-triggers' relays time the next copy on it, #278), the line "<n>x <name>" (n alone is
//!    the name) on the HUD, and the pickup is taken out of the world for good (FUN_000db3e0). A
//!    level's own pickup never comes back (nothing re-adds it); a spawn-trigger's copy does when
//!    its trigger's timer says (take01: the Particle box 10 s after, take03: the Rail boxes
//!    20 s, each with the green rings of the spawn flash).
//!
//! The line is the HUD's pickup feed (play_pickups.rs `PickupFeed`: two pickups of one kind
//! within its 4 s are one line with the sum: take03's two Rail boxes "20x Rail Ammo").
//!
//! Test hooks: BF_PICKUP_LOG=1 prints each pickup taken or refused (who, what, how many went in,
//! the stack); BF_TEST_AMMO (play_ammo.rs) sets the reserves; the test map's object menu (O) and
//! its row of pickups carry every type.

use super::*;
use bf_viewer::bf::character::ItemType;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::{Placed, SpawnTemplate};

/// IFSET_AMMO_BOX (default.xbe's IFSET_ name table at 0x3be3cc, counted from IFSET_NONE 0): an ammo
/// item's function-type, whose h_e5f51266 is the ammo-type it feeds (play_ammo.rs `STACK_LIMITS`).
const IFSET_AMMO_BOX: i64 = 14;

/// The squad's stack a pickup fills (its key: the given type's group-type and h_e5f51266,
/// FUN_00149d30).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Stack {
    /// rounds of an ammo-type: the reserve (play_ammo.rs `Reserve`), category 1 IG_AMMO
    Rounds(i64),
}

/// What walking over a pickup of item type `kind` gives (`gift`): the stack, how many of it
/// (FUN_0014ce90's multiplier: the pickup type's +0x190, its pickup-amount), the line's name and
/// the sound.
#[derive(Clone, Debug, PartialEq)]
struct Gift {
    stack: Stack,
    amount: i64,
    name: String,
    sound: u32,
}

/// The item type's display name for the pickup line (h_1dd3a49d, FUN_00146340's type), else its
/// stringtable-name: "Particle Ammo".
fn line_name(game: &Game, item: &ItemType) -> String {
    game.strings.get(&item.message).cloned().filter(|_| item.message != 0).unwrap_or_else(|| item.label.clone())
}

/// The gift of a pickup of item type `kind` (FUN_0011c170's item path, 0x11c2bd-0x11c51c), None
/// for a type this path doesn't take (the medkits, fruit and DNA are play_pickups.rs's, the
/// power-ups play_powerups.rs's).
fn gift(game: &Game, kind: u32) -> Option<Gift> {
    let item = game.items.get(&kind)?;
    // an ammo box: its h_e5f51266 is the ammo-type, the reserve's key
    if item.function == IFSET_AMMO_BOX && item.group == super::inventory::IG_AMMO && item.kind > 0 {
        return Some(Gift { stack: Stack::Rounds(item.kind), amount: item.amount.max(1), name: line_name(game, item), sound: item.sound });
    }
    None
}

/// The walk-over's system, after the other pickups' frame (the item box's frame follows it).
pub fn plugin(app: &mut App) {
    app.add_systems(Update, take_items.after(pickups::PickupSystems).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// The walk-over of the pickups above, each frame (see the module notes): every pickup in the
/// world that isn't a spawn-trigger's template, against the player's character and then each
/// squad member (the test map's NPCs take nothing): the first in reach that the stack takes some
/// from takes it; one in reach with nothing fitting shows "(full)" if it's the player's own.
#[allow(clippy::too_many_arguments)]
pub fn take_items(mut commands: Commands, game: Res<GameData>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
                  mut reserve: ResMut<ammo::Reserve>, mut feed: ResMut<pickups::PickupFeed>, mut events: ResMut<triggers::TriggerEvents>,
                  lying: Query<(Entity, &Placed, &Transform), Without<SpawnTemplate>>) {
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    for (e, p, t) in &lying {
        if p.tag != h("inventory-object") {
            continue;
        }
        let Some(gift) = gift(&game.0, p.kind) else { continue };
        let at = t.translation;
        for k in 0..=squad.0.len() {
            let u = if k == 0 { &mut *player } else { &mut squad.0[k - 1] };
            if u.dead || u.absent || !u.in_squad() || u.loaded.is_none() || !powerups::in_reach(u.position, at) {
                continue;
            }
            let added = match gift.stack {
                Stack::Rounds(ammo_type) => reserve.add(ammo_type, gift.amount),
            };
            if added == 0 {
                // nothing fits: it stays, and the player's own character is told, red, each frame
                // (FUN_0011c6a0 with AL clear; the log says it once)
                if k == 0 {
                    let shown = feed.0.iter().any(|l| l.name == gift.name && l.full);
                    feed.add(gift.name.clone(), 0);
                    if log && !shown {
                        println!("t {:.2}: {} on {} h_{:08x} at {at:.2}: the stack is full, it stays", u.sim_time, unit_name(u), gift.name, p.name);
                    }
                }
                continue;
            }
            if gift.sound != 0 {
                u.sound_queue.push((gift.sound, 1.0));
            }
            if p.name != 0 {
                events.0.push(triggers::ObjectEvent { from: p.name, signal: bf_viewer::bf::triggers::sig::PICKUP_OBJECT, subject: u.trigger_name, data: -1 });
            }
            feed.add(gift.name.clone(), added);
            commands.entity(e).despawn();
            if log {
                println!("t {:.2}: {} took {added}x {} of {} from h_{:08x} at {at:.2} ({})", u.sim_time, unit_name(u), gift.name, gift.amount, p.name,
                         match gift.stack { Stack::Rounds(t) => format!("reserve type {t} now {}", reserve.get(t)) });
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The squad's reserve with one Particle stack (ammo-type 4: limit 200).
    fn reserve(have: i64) -> ammo::Reserve {
        ammo::Reserve::with(&[(4, have, 200), (5, 0, 400)])
    }

    /// take01 / take02 / take03: a box adds its `pickup-amount` as far as the stack's limit takes
    /// it (FUN_0014bfe0's clamp), the rest lost; a full stack takes none (the box stays: nothing
    /// added); an unassigned stack (a Rail stack of 0) takes the box whole.
    #[test]
    fn a_box_fills_up_to_the_limit() {
        let mut r = reserve(100);
        assert_eq!(r.add(4, 25), 25);
        assert_eq!(r.get(4), 125);
        let mut r = reserve(195);
        assert_eq!(r.add(4, 25), 5);
        assert_eq!(r.get(4), 200);
        assert_eq!(r.add(4, 25), 0);
        assert_eq!(r.get(4), 200);
        let mut r = reserve(0);
        assert_eq!(r.add(5, 10) + r.add(5, 10), 20);
    }
}
