//! The OrgSen (#21): putting it on, taking it off, and vision mode 1. Read from default.xbe (the
//! addresses are in the comments and in viewer/docs/code-map.md; the reading is
//! todo/18-ammo-pickups/leads.md §4) and checked against that folder's take09 (taking it) and
//! take10 (using it, sdm_e13, xemu). Picking it up is play_item_pickups.rs's; the view it gives is
//! play_vengar.rs's (mode 1 beside Vengar's mode 2), and the lock through walls is
//! play_reticle.rs's.
//!
//! **The item.** "OrgSen" ("Easily detects concealed organic signatures even through most
//! obstacles.") is an `<inventory>` item of function-type 12, IFSET_VISION_ASSIST (group-type 6
//! IG_ENHANCEMENT, stack-limit 1, use type 1 IOU_ATTACH_TO_FACE), which the placements of the
//! campaign's squads carry and sdm_e13's two spawn-triggers hand out as a pickup (play_item_pickups.rs).
//! The item box lists it ("OrgSen", the icon h_1b73c691, no count below 2) in the pale blue for the
//! characters whose skill takes its use type and the red for Brutus's, whose skill refuses it with
//! "Cannot use" (play_inventory.rs `use_other`, #196 take26).
//!
//! **Using it** (LT, the item box's use; FUN_000abf60 -> FUN_00121170's case 1), for Tex, Hawk and
//! Flint, whose skills say yes (0x262a80):
//!  - with nothing on the face (the inventory's face holder, +0x88): the stance's `bring2face`
//!    overlay (motion type 10 OMT_BRING_TO_FACE, `Sc_w1_` or `Sc_w2_` by the weapon in hand: Tex's
//!    1.80 s, Hawk's 3.00, Flint's 3.43, Brutus's 3.00), the upper body only, the character free to
//!    move meanwhile. At its reach event (1a6b4920) the OrgSen is in the hand holder, a copy: the
//!    stack keeps its count (FUN_0014abc0 doesn't take from it for use types 1 and 7); at its
//!    **attach_item** event (0x0775fceb, FUN_00117660 at 0x11774c) the item moves to the face holder
//!    and its set's "put on" hook (0x1479f0) sets the character's vision mode 1: char+0x644 = 1 and
//!    its camera's +0xd8 = 1. take10: LT at 3.0 s of video, the hand holder holding it at 3.8 and the
//!    face holder, with vision mode 1, from 4.66 s (the clip's 0.57 and 1.33 s of Tex's, run at
//!    the game's 0.9 of video time). The box keeps "OrgSen" in the pale blue all the while (its
//!    colour is the skill's, not "a use is pending");
//!  - with it on: FUN_00121010 takes it off **at once**, no clip: it sets the item's state 5
//!    (FUN_0014a610, which plays the type's event record for that state: its sound h_feaf631d at the
//!    item's place, the wearer's), then its set's "taken off" hook (0x147a20) calls
//!    FUN_00126df0(character, 0), vision mode 0 (take10: LT again at 17.0 s, mode 0 and the normal
//!    picture the frame after, and a broadband transient in the audio 40 ms after the press, +25 dB
//!    above 1 kHz). The put-on sets the item's place state 2 (FUN_0014a880), for which the type has
//!    no event record: no sound, as take10's audio has none at 4.66 s.
//!  Both go through FUN_001210c0's gate first: the character free (no reload, throw, weapon switch
//!  or other item use under way, no gun in the second hand, the skill's word), and a Hawk who's
//!  stealthed pays 5 stamina for the put-on (the skill's +0x104). The scope is left first
//!  (FUN_001249b0).
//!
//! **Vision mode 1** is a state of the character (`Player::vision_assist`; Vengar's mode 2 is his
//! skill's), which the view of the player who controls it draws as play_vengar.rs says. It is the
//! character's own: another squad member's camera has its own mode, so passing control to one who
//! hasn't it on shows the normal picture, and back shows this one's again; the AI never reads it
//! (no reader found in its code). A dead character's is dropped here (the death camera is another
//! view: a guess, not filmed).
//!
//! **What it does besides the picture**: with a gun in hand whose bullet passes through what it
//! hits (the Rail RVR, the Rail CLVR, the Saryl-45: h_11caf57b) the crosshair locks on a
//! character within 75 m without the sight ray, through walls, and the guns aim at the lock's
//! point (FUN_000bf5a0, FUN_000bfeb0, 0xc5720: play_reticle.rs). Flint's scope overlay isn't
//! drawn while a mode is on (FUN_000b3970: not done here).
//!
//! Test hooks: BF_TEST_INV=OrgSen:1 carries one, BF_TEST_ITEM=OrgSen selects it and
//! BF_TEST_ITEM_USE=<s>[,<s>...] presses the use button at those times (the put-on, then the
//! take-off); BF_PICKUP_LOG=1 prints each use and the vision mode's changes.

use super::*;

/// IOU_ATTACH_TO_FACE, the OrgSen's use type (h_1ee2f4ed 1; the IOU_ name table at 0x3be61c,
/// FUN_00121170's case 1): worn on the face, on the face holder.
pub const IOU_ATTACH_TO_FACE: i64 = 1;

/// The sound of the OrgSen's state 5 event, h_feaf631d (`<event state="5" h_fa04e025=...>` in its
/// type's `<events>`): what FUN_0014a610 plays when FUN_00121010 sets the worn item's state 5, at
/// its place (the wearer's). take10's audio has a short broadband transient at the take-off.
const SOUND_TAKEN_OFF: u32 = 0xFEAF_631D;

/// The OrgSen's use and its put-on, after the characters' step (the clip's event frame is that
/// step's) and before the sounds.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, (use_orgsen, put_on).chain().after(update_player).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// The vision mode `u`'s view is in (char+0x644 and its camera's +0xd8): 2 while Brutus's Vengar is
/// on (his on, 0x13c020), 1 with the OrgSen on its face (0x1479f0), else 0. Brutus's skill refuses
/// the OrgSen, so the two never meet in one character; the order only settles a test map's
/// stand-ins.
pub fn mode(u: &Player) -> u8 {
    if skills::vengar_on(u) {
        2
    } else if u.vision_assist && !u.dead {
        1
    } else {
        0
    }
}

/// Whether the character can start an item use now (FUN_001210c0's gate, as play_pickups.rs
/// `use_medkit` has it): alive, free to act, no other overlay under way, nothing in the second hand,
/// the skill's word (`skills::lets_act`).
fn free_for_item(p: &Player) -> bool {
    !p.dead && p.using.is_none() && p.gun_use.is_none() && p.reloading.is_none() && p.throwing.is_none() && p.switching.is_none()
        && p.skill.toggling.is_none() && p.second.is_none() && !skills::overlays_blocked(p) && skills::lets_act(p)
}

/// LT with the OrgSen in the item box (play_itembox.rs `item_controls` raises `Player::item_use`):
/// on the face already, off at once (FUN_00121010); else the bring2face clip starts
/// (FUN_00121170's case 1, the overlay of motion type 10) with the stance's skill cost
/// (FUN_0014abc0's +0x104). Brutus's refusal and its "Cannot use" are `inventory::use_other`'s
/// (his skill's +0x1c says no to the use type before any of this).
fn use_orgsen(mut player: ResMut<Player>, game: Res<GameData>) {
    let Some(Item::Other(kind)) = player.item else { return };
    if !player.item_use || player.dead {
        return;
    }
    let Some(c) = player.others.iter().find(|c| c.kind == kind && c.count > 0).copied() else { return };
    if c.use_type != IOU_ATTACH_TO_FACE || !skills::uses_item(&player, c.use_type) || !free_for_item(&player) {
        return;
    }
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    if player.vision_assist {
        // the face holder holds it: taken off at once, no clip (FUN_00121010: the set's +0x1c
        // calls FUN_00126df0 with 0)
        player.vision_assist = false;
        player.sound_queue.push((SOUND_TAKEN_OFF, 1.0));
        if log {
            println!("t={:.2} {}: the OrgSen is taken off: vision mode 0, its state 5 sound h_{SOUND_TAKEN_OFF:08x} (in the bank: {})", player.sim_time,
                     unit_name(&player), game.0.sounds.has(SOUND_TAKEN_OFF));
        }
        return;
    }
    // (out of the scope first, FUN_001249b0, as an item's use does)
    player.leave_scope();
    player.using = Some(0.0);
    player.use_face = true;
    skills::used_item(&mut player, c.use_type);
    if log {
        println!("t={:.2} {}: the OrgSen's bring2face clip starts", player.sim_time, unit_name(&player));
    }
}

/// The clip's attach_item event (`Player::item_used` with `use_face`: play.rs `step_player` flags
/// the frame): the item moves to the face holder and its put-on hook sets vision mode 1
/// (0x1479f0). A character that has died drops it (see the module notes).
fn put_on(mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if u.dead {
            u.vision_assist = false;
            continue;
        }
        if u.item_used && u.use_face {
            u.vision_assist = true;
            if log {
                println!("t={:.2} {}: the OrgSen is on (attach_item): vision mode 1", u.sim_time, unit_name(u));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The view's mode: Vengar's skill is 2 (not built here), the OrgSen's 1 while its wearer
    /// lives, none otherwise.
    #[test]
    fn the_mode_follows_the_orgsen() {
        let mut p = Player::new(3);
        assert_eq!(mode(&p), 0);
        p.vision_assist = true;
        assert_eq!(mode(&p), 1);
        p.dead = true;
        assert_eq!(mode(&p), 0);
    }
}
