//! The front end's MULTIPLAYER submenu (#310; viewer/docs/menu.md § MULTIPLAYER): the demo's own
//! main menu entry that groups the game's DEATHMATCH and SQUAD DEATHMATCH, which the game lists
//! on its main menu itself (play_menu.rs `MAIN`). A child of play_menu.rs.
//!
//! The screen is CAMPAIGN TYPE's (game-options h_f9728897, the menu the game's CAMPAIGN entry
//! opens: play_campaign_menus.rs), made from that menu's own items so it looks and moves as the
//! game's sub-menus do: the panel (picture set h_ff6b96da), the typed title (style h_189b1ae9,
//! 484 x 38 at (90, 50)), the SINGLE XBOX CONSOLE heading (h_1cc9f852, a band at (70, 118) in
//! style h_e8b4c2d6), the group's first two buttons (200 x 20 at (80, 152) and (80, 180), style
//! h_ea5acf71: no font of its own, so the body font at 14 points; `color-normal`, the selected
//! one in `color-selected` with its echo), each button's description (216 x 254 at (314, 118),
//! style h_15405823, shown with its button: its buddy-control) and the A SELECT and B BACK
//! legends (86, 410) and (242, 410). Their animations are the items' own (translation-
//! anims sliding the buttons in, color-anim fading the texts in, the title typed, select-anim
//! echoing the selected button) and play on the menu's clock as CAMPAIGN TYPE's do; so do the
//! sounds: the first button's slide sound as the screen opens, h_ed2b5e99 as the selection moves
//! (the error sound at the ends), select and open sounds going on to a map list.
//!
//! What differs from CAMPAIGN TYPE, all of it the demo's choice (the game has no MULTIPLAYER
//! entry):
//!  * the title says MULTIPLAYER (the demo's text, `play_menu.rs` `MULTIPLAYER`);
//!  * the group has DEATHMATCH and SQUAD DEATHMATCH in the places of CAMPAIGN and QUICK CAMPAIGN,
//!    in the game's words: the labels of the first buttons of the game's own DEATHMATCH and SQUAD
//!    DEATHMATCH panels (game-options h_1854a812 and h_ec00d7fa: h_f4475e09 and h_0e51acdd; the
//!    main menu's SQUAD DEATHMATCH, h_18653c47, is two lines for its two-line box), and the
//!    descriptions the game's main menu gives the two entries (h_163f7c9d, h_1e9ff6eb), their line
//!    breaks (made for the main menu's 270-unit box) made spaces so the 216-unit box wraps them;
//!  * no RESUME LAST, no System Link group: the demo has no saves and no network play.
//! Each entry goes on to SELECT MISSION for its mode (the game's mode panel, split-screen /
//! System Link, is skipped, as it was from the main menu). Back from SELECT MISSION returns
//! here, on that mode; Back from here returns to the main menu on MULTIPLAYER.
//!
//! The mouse works here as on the main menu (CAMPAIGN TYPE's screens take no mouse): each button
//! has an invisible target on its own box; pointing at it selects it, a click selects and goes on.
//!
//! Test hooks: play_menu.rs's BF_MENU_SCREEN=multiplayer with BF_MENU_PICK=<0|1> shows the screen on
//! that entry; BF_MENU_PRESS drives it (with BF_CAPTURE: menu_NNN.png).

use std::collections::HashMap;

use bevy::prelude::*;
use bf_viewer::bf::character::{LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};
use bf_viewer::bf::menus::{GameMenu, ItemKind};

use super::campaign::{self, CampaignMenus};
use super::{play, Asked, Assets2, Item, Menu, Screen, Sfx, MAIN, MULTIPLAYER, SND_ERROR, SND_OPEN, SND_SELECT, SND_SLIDE};

/// DEATHMATCH's word: the label of the first button of the game's own DEATHMATCH panel
/// (game-options h_1854a812, at (80, 152)), one line as CAMPAIGN TYPE's buttons are (the main
/// menu's h_1d317bba is the same word).
const S_DEATHMATCH: u32 = 0xF447_5E09;
/// SQUAD DEATHMATCH's word: the label of the first button of the game's own SQUAD DEATHMATCH panel
/// (game-options h_ec00d7fa, at (80, 152)), "SQUAD DEATHMATCH" in one line; the main menu's
/// h_18653c47 is the same with a line break for its two-line box.
const S_SQUAD_DEATHMATCH: u32 = 0x0E51_ACDD;
/// DEATHMATCH's description: the one the game's main menu shows for it (h_163f7c9d, "Play
/// head-to-head, up to 4 players on one Xbox console, or up to 8 players over System Link.").
const S_DEATHMATCH_ABOUT: u32 = 0x163F_7C9D;
/// SQUAD DEATHMATCH's description: the one the game's main menu shows for it (h_1e9ff6eb, "Fight
/// against friends, each commanding different squads. ...").
const S_SQUAD_DEATHMATCH_ABOUT: u32 = 0x1E9F_F6EB;
/// CAMPAIGN TYPE's SYSTEM LINK heading (item name h_0e49a355, at (70, 246) in game-options
/// h_f9728897): the second group, which the submenu leaves out.
const I_SYSTEM_LINK: u32 = 0x0E49_A355;
/// The style of CAMPAIGN TYPE's five descriptions (h_15405823), the static texts shown with a button.
const STYLE_ABOUT: u32 = 0x1540_5823;

/// An entry of the submenu: its button's words, its description (both string ids) and the level
/// kind of the maps it leads to (SELECT MISSION's list).
pub struct Mode {
    word: u32,
    about: u32,
    pub kind: u32,
}

/// The submenu's entries, in order (the menu's `multi`): DEATHMATCH, SQUAD DEATHMATCH.
pub const MODES: [Mode; 2] = [
    Mode { word: S_DEATHMATCH, about: S_DEATHMATCH_ABOUT, kind: LEVEL_DEATHMATCH },
    Mode { word: S_SQUAD_DEATHMATCH, about: S_SQUAD_DEATHMATCH_ABOUT, kind: LEVEL_SQUAD_DEATHMATCH },
];

/// The entry whose maps are a level kind's (the list SELECT MISSION shows); an unknown kind gives
/// the first, DEATHMATCH.
pub fn mode_of(kind: u32) -> usize {
    MODES.iter().position(|m| m.kind == kind).unwrap_or(0)
}

/// The main menu's MULTIPLAYER chosen: the submenu, on its first entry (as CAMPAIGN TYPE opens on
/// its first button).
pub fn open(menu: &mut Menu) {
    menu.screen = Screen::Multiplayer;
    menu.multi = 0;
}

/// Back from SELECT MISSION: the submenu on the mode whose maps those were, and the main menu's
/// selection on MULTIPLAYER for the next Back.
pub fn back_to(menu: &mut Menu) {
    menu.screen = Screen::Multiplayer;
    menu.multi = mode_of(menu.kind);
    menu.main = main_entry();
}

/// MULTIPLAYER's place on the main menu.
fn main_entry() -> usize {
    MAIN.iter().position(|m| m.0 == MULTIPLAYER).unwrap_or(0)
}

/// The submenu's menu, made of CAMPAIGN TYPE's (`base`): its items but for the buttons past the
/// first `MODES.len()` and their descriptions, and the SYSTEM LINK heading; the kept buttons say
/// the modes' words and their descriptions the modes' texts. Everything else of an item (place,
/// size, style, animations, buddies) is CAMPAIGN TYPE's.
fn derive(base: &GameMenu) -> GameMenu {
    let buddies: Vec<&Vec<u32>> = base.items.iter().filter_map(|i| match &i.kind {
        ItemKind::Button { buddies, .. } => Some(buddies),
        _ => None,
    }).collect();
    // (a button's description is its first buddy)
    let kept: Vec<u32> = buddies.iter().take(MODES.len()).filter_map(|b| b.first().copied()).collect();
    let dropped: Vec<u32> = buddies.iter().skip(MODES.len()).flat_map(|b| b.iter().copied()).collect();
    let mut button = 0;
    let mut items = vec![];
    for item in &base.items {
        let mut item = item.clone();
        match &mut item.kind {
            ItemKind::Button { text, .. } => {
                let Some(mode) = MODES.get(button) else { continue };
                button += 1;
                *text = mode.word;
            }
            ItemKind::Text(id) => {
                if let Some(k) = kept.iter().position(|&n| n == item.name) {
                    *id = MODES[k].about;
                } else if dropped.contains(&item.name) || item.name == I_SYSTEM_LINK {
                    continue;
                }
            }
            ItemKind::Picture { .. } | ItemKind::Legend { .. } => {}
            _ => continue,
        }
        items.push(item);
    }
    GameMenu { name: base.name, at: base.at, size: base.size, items }
}

/// Build the submenu (play_menu.rs `build` calls it when the menu changes), with entry `pick`
/// selected: CAMPAIGN TYPE's drawing (`campaign::build`'s order: the panel, the buttons, the
/// texts, the legends) of the derived menu, its animations running from when the screen opened
/// (the menu's Clock); the first button's slide sound when it has just `opened`.
#[allow(clippy::too_many_arguments)]
pub fn build(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, sfx: &Sfx, pick: usize, now: f32,
             opened: bool) {
    let Some(base) = cm.menus.menus.get(&campaign::M_TYPE) else {
        eprintln!("menu: game-options has no menu h_{:08x}", campaign::M_TYPE);
        return;
    };
    let menu = derive(base);
    let none = |_: u32| None;
    campaign::pictures(commands, screen, cm, &menu, menu.at, &none, None);
    let hidden = campaign::buttons(commands, screen, cm, strings, &menu, pick, now);
    // the title is the demo's word; a description's line breaks (the main menu's box is wider)
    // become spaces
    let filled = |name: u32| -> Option<Option<String>> {
        let item = menu.items.iter().find(|i| i.name == name)?;
        match item.kind {
            ItemKind::Text(_) if item.style == campaign::STYLE_TYPED => Some(Some(MULTIPLAYER.text(strings))),
            ItemKind::Text(id) if item.style == STYLE_ABOUT => Some(Some(super::paragraph(&campaign::string(strings, id)))),
            _ => None,
        }
    };
    campaign::texts(commands, screen, cm, strings, &menu, menu.at, &hidden, &filled);
    // the mouse's targets: each button's own box
    for (i, item) in menu.items.iter().filter(|i| matches!(i.kind, ItemKind::Button { .. })).enumerate() {
        let at = menu.at + item.at;
        commands.spawn((Button, Item::Mode(i), super::super::hud::at(at.x, at.y, item.size.x, item.size.y), ChildOf(screen)));
    }
    for item in &menu.items {
        if let ItemKind::Legend { button, label } = item.kind {
            campaign::legend(commands, screen, cm, strings, menu.at, item, button, label, true);
        }
    }
    if opened && menu.items.iter().any(|i| i.anims.first() == Some(&campaign::ANIM_SLIDE[0])) {
        // the first item's translation-anim sound (h_e2027fa6's h_e8ace091)
        play(commands, sfx, SND_SLIDE);
    }
}

/// The submenu takes the frame's presses: up and down move the selection (the error sound at the
/// ends, as CAMPAIGN TYPE's), A goes on to SELECT MISSION for the mode (the error sound if the
/// level list has none), B goes back to the main menu on MULTIPLAYER (no sound: the main menu
/// plays its opening ones).
pub fn input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, assets: &Assets2) {
    let mut pick = menu.multi.min(MODES.len() - 1);
    if campaign::step_pick(commands, sfx, &mut pick, MODES.len(), asked) {
        menu.multi = pick;
    } else if asked.select {
        let kind = MODES[pick].kind;
        if assets.maps.get(&kind).is_some_and(|m| !m.is_empty()) {
            menu.kind = kind;
            menu.screen = Screen::Missions;
            play(commands, sfx, SND_SELECT);
            play(commands, sfx, SND_OPEN);
        } else {
            play(commands, sfx, SND_ERROR);
        }
    } else if asked.back {
        menu.screen = Screen::Main;
        menu.main = main_entry();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::menus::MenuItem;

    /// An item of a CAMPAIGN TYPE-like menu.
    fn item(name: u32, style: u32, kind: ItemKind, anims: &[u32]) -> MenuItem {
        MenuItem { kind, name, style, at: Vec2::new(name as f32, 1.0), size: Vec2::new(200.0, 20.0), anims: anims.to_vec() }
    }

    /// CAMPAIGN TYPE's shape (game-options h_f9728897): five buttons with a buddy each, the
    /// title, the two group headings, the five descriptions, the panel and the legends.
    fn campaign_type() -> GameMenu {
        let button = |name: u32, text: u32, buddy: u32| item(name, 0xEA5A_CF71, ItemKind::Button { text, buddies: vec![buddy] }, &[0xE202_7FA6]);
        let text = |name: u32, style: u32, id: u32| item(name, style, ItemKind::Text(id), &[]);
        GameMenu {
            name: campaign::M_TYPE, at: Vec2::ZERO, size: Vec2::new(640.0, 480.0),
            items: vec![
                button(0x02B3_42D3, 0x06E5_E6D8, 0xEA3D_AF3C), button(0x0024_5356, 0xF7E9_5A5E, 0xF4A5_6B36), button(0x1B74_4C5F, 0xE558_0964, 0x1632_AC3F),
                button(0xFE2C_7D14, 0xEF9D_D11D, 0x1412_FA83), button(0xED7A_7EC2, 0xFCCB_D2CB, 0x0744_F955),
                text(0x08E7_9DB2, campaign::STYLE_TYPED, 0xF6AD_E596), text(0x1CC9_F852, 0xE8B4_C2D6, 0xEE9A_A66D), text(I_SYSTEM_LINK, 0xE8B4_C2D6, 0xF30D_DF1D),
                text(0xEA3D_AF3C, STYLE_ABOUT, 0x186B_7D31), text(0xF4A5_6B36, STYLE_ABOUT, 0xF7C4_F81F), text(0x1632_AC3F, STYLE_ABOUT, 0x1A83_55C0),
                text(0x1412_FA83, STYLE_ABOUT, 0x1919_859B), text(0x0744_F955, STYLE_ABOUT, 0x0A4F_864D),
                item(0x0FBE_73AA, 0, ItemKind::Picture { set: 0xFF6B_96DA, piece: None }, &[]),
                item(0x0499_CE0B, 0xE35B_B628, ItemKind::Legend { button: 1, label: 2 }, &[]),
                item(0x0E82_04A4, 0xE35B_B628, ItemKind::Legend { button: 3, label: 4 }, &[]),
            ],
        }
    }

    /// The derived menu keeps CAMPAIGN TYPE's first two buttons, in their places, as DEATHMATCH and
    /// SQUAD DEATHMATCH with their descriptions, and the title, the first heading, the panel and
    /// the legends; the other three buttons, their descriptions and the SYSTEM LINK heading go.
    #[test]
    fn derives_from_campaign_type() {
        let base = campaign_type();
        let menu = derive(&base);
        let buttons: Vec<_> = menu.items.iter().filter(|i| matches!(i.kind, ItemKind::Button { .. })).collect();
        assert_eq!(buttons.len(), 2);
        assert_eq!((buttons[0].name, buttons[1].name), (0x02B3_42D3, 0x0024_5356));
        assert_eq!((buttons[0].at, buttons[1].at), (base.items[0].at, base.items[1].at));
        let words: Vec<u32> = buttons.iter().map(|b| match &b.kind { ItemKind::Button { text, .. } => *text, _ => 0 }).collect();
        assert_eq!(words, [S_DEATHMATCH, S_SQUAD_DEATHMATCH]);
        let about = |name: u32| menu.items.iter().find(|i| i.name == name).map(|i| match i.kind { ItemKind::Text(id) => id, _ => 0 });
        assert_eq!(about(0xEA3D_AF3C), Some(S_DEATHMATCH_ABOUT));
        assert_eq!(about(0xF4A5_6B36), Some(S_SQUAD_DEATHMATCH_ABOUT));
        for gone in [0x1632_AC3F, 0x1412_FA83, 0x0744_F955, I_SYSTEM_LINK, 0x1B74_4C5F, 0xFE2C_7D14, 0xED7A_7EC2] {
            assert!(about(gone).is_none(), "{gone:08x} stays");
        }
        for kept in [0x08E7_9DB2, 0x1CC9_F852, 0x0FBE_73AA, 0x0499_CE0B, 0x0E82_04A4] {
            assert!(menu.items.iter().any(|i| i.name == kept), "{kept:08x} goes");
        }
        // (the first button keeps its slide-in animation, the one with the sound)
        assert_eq!(buttons[0].anims, [0xE202_7FA6]);
    }

    /// The entries lead to the two deathmatch map lists, and a map list's entry is found by its
    /// kind (an unknown kind: the first).
    #[test]
    fn modes_lead_to_their_maps() {
        assert_eq!(MODES[0].kind, LEVEL_DEATHMATCH);
        assert_eq!(MODES[1].kind, LEVEL_SQUAD_DEATHMATCH);
        assert_eq!((mode_of(LEVEL_DEATHMATCH), mode_of(LEVEL_SQUAD_DEATHMATCH), mode_of(0)), (0, 1, 0));
    }

    /// Back from SELECT MISSION lands on the submenu on that mode and the main menu's selection on
    /// MULTIPLAYER; opening from the main menu starts on the first entry.
    #[test]
    fn back_returns_to_the_mode() {
        let mut menu = Menu { screen: Screen::Missions, kind: LEVEL_SQUAD_DEATHMATCH, ..default() };
        back_to(&mut menu);
        assert_eq!((menu.screen, menu.multi, MAIN[menu.main].0), (Screen::Multiplayer, 1, MULTIPLAYER));
        menu.screen = Screen::Main;
        open(&mut menu);
        assert_eq!((menu.screen, menu.multi), (Screen::Multiplayer, 0));
    }
}
