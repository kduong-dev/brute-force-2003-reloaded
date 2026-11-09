//! The front end's MULTIPLAYER submenu (#310; viewer/docs/menu.md § MULTIPLAYER): the demo's own
//! main menu entry that groups the game's DEATHMATCH and SQUAD DEATHMATCH, which the game lists
//! on its main menu itself (play_menu.rs `MAIN`). A child of play_menu.rs.
//!
//! The screen is CAMPAIGN TYPE's (game-options h_f9728897, the menu the game's CAMPAIGN entry
//! opens: play_campaign_menus.rs), made from that menu's own items so it looks and moves as the
//! game's sub-menus do: the panel (picture set h_ff6b96da), the typed title (style h_189b1ae9,
//! 484 x 38 at (90, 50)), the group's first two buttons (200 x 20 at (80, 152) and (80, 180),
//! style h_ea5acf71: no font of its own, so the body font at 14 points; `color-normal`, the
//! selected one in `color-selected` with its echo), each button's description (216 x 254 at
//! (314, 118), style h_15405823, shown with its button: its buddy-control) and the A SELECT and
//! B BACK legends (86, 410) and (242, 410). Their animations are the items' own (translation-
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
//!  * no RESUME LAST, no System Link group: the demo has no saves and no network play;
//!  * no heading band: CAMPAIGN TYPE's SINGLE XBOX CONSOLE band (h_1cc9f852, at (70, 118) in
//!    style h_e8b4c2d6) is dropped along with its SYSTEM LINK one: the product owner's choice.
//!    The descriptions are the game's main-menu texts and they mention System Link ("... or up
//!    to 8 players over System Link"), which a SINGLE XBOX CONSOLE band over them would
//!    contradict. The buttons stay where CAMPAIGN TYPE's data puts them, (80, 152) and (80, 180),
//!    not moved up into the band's place.
//! Each entry goes on to SELECT MISSION for its mode (the game's mode panel, split-screen /
//! System Link, is skipped, as it was from the main menu). Back from SELECT MISSION returns
//! here, on that mode; Back from here returns to the main menu on MULTIPLAYER. It opens on
//! DEATHMATCH each time it is entered from the main menu (the product owner's choice; CAMPAIGN
//! TYPE too opens on its first button).
//!
//! The mouse works here as on the main menu (CAMPAIGN TYPE's screens take no mouse): each button
//! has an invisible target on its own box; pointing at it selects it, a click selects and goes on.
//!
//! **MATCH SETUP** (#420, the demo's): a third screen, between the submenu and SELECT MISSION,
//! where a match's time limit, score limit and the character you play are chosen. The game has no
//! such screen here (its panels lead to a mode and a map), so its words, its rows and their values
//! are the demo's, the product owner's choices: drawn as the submenu is (CAMPAIGN TYPE's panel, an
//! outlined typed title, the menu's fonts and colours), with the three rows' labels left and their
//! values right, the selected row in the menu's selected colour. Left / Right change the row's
//! value, Up / Down move between the rows, A goes on to SELECT MISSION, B back to the submenu. The
//! choices are kept in the `Menu` across screens, and `start_loading` hands them to the match as
//! `match_rules::Rules` (play_match.rs runs them).
//!
//! Test hooks: play_menu.rs's BF_MENU_SCREEN=multiplayer with BF_MENU_PICK=<0|1> shows the screen on
//! that entry (`setup` shows MATCH SETUP on it); BF_MENU_PRESS drives it (with BF_CAPTURE:
//! menu_NNN.png).

use std::collections::HashMap;

use bevy::prelude::*;
use bf_viewer::bf::character::{LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};
use bf_viewer::bf::menus::{GameMenu, ItemKind};

use super::campaign::{self, CampaignMenus};
use super::{play, Asked, Assets2, Item, Menu, Screen, Sfx, MAIN, MULTIPLAYER, SND_CHANGE, SND_ERROR, SND_MOVE, SND_OPEN, SND_SELECT, SND_SLIDE};

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
/// The style of CAMPAIGN TYPE's two group headings (h_e8b4c2d6: a fill band under a right-aligned
/// text): SINGLE XBOX CONSOLE (item h_1cc9f852, at (70, 118) in game-options h_f9728897) and SYSTEM
/// LINK (h_0e49a355, at (70, 246)). The submenu has neither: no network play, and the product
/// owner dropped the first too, since the descriptions it would head mention System Link.
const STYLE_HEADING: u32 = 0xE8B4_C2D6;
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

/// MULTIPLAYER's place on the main menu (every lookup of it goes through here, so they agree).
pub(super) fn main_entry() -> usize {
    MAIN.iter().position(|m| m.0 == MULTIPLAYER).unwrap_or(0)
}

/// The submenu's menu, made of CAMPAIGN TYPE's (`base`): its items but for the buttons past the
/// first `MODES.len()` and their descriptions, and the group headings (STYLE_HEADING); the kept
/// buttons say the modes' words and their descriptions the modes' texts. Everything else of an
/// item (place, size, style, animations, buddies) is CAMPAIGN TYPE's.
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
                } else if dropped.contains(&item.name) || item.style == STYLE_HEADING {
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
            // (its MATCH SETUP screen first: #420)
            menu.screen = Screen::Setup;
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

// ---- MATCH SETUP (#420, the demo's) ----

/// The rows of the MATCH SETUP screen, in order: what their labels say.
pub const ROWS: [&str; 4] = ["TIME LIMIT", "SCORE LIMIT", "BOT SQUADS", "CHARACTER"];
/// The time limits it offers (min): none, then 5 to 30 (the demo's values, the product owner's).
pub const TIMES: [Option<u32>; 6] = [None, Some(5), Some(10), Some(15), Some(20), Some(30)];
/// The score limits it offers (kills): none, then 5 to 25.
pub const SCORES: [Option<u32>; 6] = [None, Some(5), Some(10), Some(15), Some(20), Some(25)];
/// The character you play: the squad's four, in `CHARACTERS`' order (the places keys 1-4 take
/// control of: Brutus, Flint, Hawk, Tex).
pub const MEMBERS: [&str; 4] = ["BRUTUS", "FLINT", "HAWK", "TEX"];
/// The bot squads it offers (play_bots.rs): none, then one to three more squads to fight.
pub const BOTS: [usize; 4] = [0, 1, 2, 3];
/// The rows' tops on the 640 x 480 screen, under the title and the mode's word.
const ROW_Y: [f32; 4] = [176.0, 220.0, 264.0, 308.0];

/// The MATCH SETUP screen's choices: the row the keys are on, and each row's index in its table.
/// `Default` is an open match (no limit either way) and Tex, the campaign's start character.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Setup {
    pub row: usize,
    pub time: usize,
    pub score: usize,
    pub bots: usize,
    pub member: usize,
}

impl Default for Setup {
    fn default() -> Self {
        Setup { row: 0, time: 0, score: 0, bots: 0, member: 3 }
    }
}

impl Setup {
    /// A row's value as the screen says it (the table's entry, held to it).
    pub fn value(&self, row: usize) -> String {
        match row {
            0 => TIMES[self.time % TIMES.len()].map_or_else(|| "NONE".into(), |m| format!("{m} MIN")),
            1 => SCORES[self.score % SCORES.len()].map_or_else(|| "NONE".into(), |s| format!("{s} KILLS")),
            2 => match BOTS[self.bots % BOTS.len()] {
                0 => "NONE".to_string(),
                1 => "1 SQUAD".to_string(),
                n => format!("{n} SQUADS"),
            },
            _ => MEMBERS[self.member % MEMBERS.len()].to_string(),
        }
    }

    /// The rules a map started from this screen plays by (play_match.rs).
    pub fn rules(&self) -> crate::match_rules::Rules {
        crate::match_rules::Rules {
            time_limit: TIMES[self.time % TIMES.len()].and_then(crate::match_rules::Rules::minutes),
            score_limit: SCORES[self.score % SCORES.len()],
            character: Some(self.member % MEMBERS.len()),
            bots: BOTS[self.bots % BOTS.len()],
        }
    }
}

/// Draw the MATCH SETUP screen (see the module notes): the panel and the typed title as the other
/// panels' (CAMPAIGN TYPE's, SELECT MISSION's), the mode's word under it, the three rows with
/// their labels left and their values right, and the keys' line.
pub fn build_setup(commands: &mut Commands, screen: Entity, assets: &Assets2, strings: &HashMap<u32, String>, sfx: &Sfx,
                   setup: &Setup, mode: usize, opened: bool) {
    let at = |x: f32, y: f32| Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), ..default() };
    for (picture, place, size) in &assets.panel {
        commands.spawn((ImageNode { flip_y: true, ..ImageNode::new(picture.clone()) }, super::Tint::new(Color::WHITE.with_alpha(super::PANEL_ALPHA)),
                        Node { width: Val::Px(size.x), height: Val::Px(size.y), ..at(place.x, place.y) }, ChildOf(screen)));
    }
    let cap = assets.font.cap;
    let title = assets.font.text(commands, "MATCH SETUP", cap, super::Look { color: super::ABOUT, outline: None, typed: true }, false,
                                 at(90.0, 50.0 + (38.0 - assets.font.height("A", cap)) * 0.5));
    commands.entity(title).insert(ChildOf(screen));
    // the mode's word, the game's (the submenu's button)
    let word = strings.get(&MODES[mode.min(MODES.len() - 1)].word).cloned().unwrap_or_default();
    let sub = assets.body.text(commands, &word, super::BODY_CAP, super::Look::outlined(super::ABOUT), false, at(90.0, 104.0));
    commands.entity(sub).insert(ChildOf(screen));
    for (row, label) in ROWS.iter().enumerate() {
        let colour = if row == setup.row { super::SELECTED } else { super::TEAL };
        let label = assets.font.text(commands, &format!("{label}:"), cap, super::Look::outlined(colour), true, at(96.0, ROW_Y[row]));
        commands.entity(label).insert(ChildOf(screen));
        let value = assets.font.text(commands, &setup.value(row), cap, super::Look::outlined(colour), true,
            Node { position_type: PositionType::Absolute, right: Val::Px(super::SCREEN.x - 508.0), top: Val::Px(ROW_Y[row]), ..default() });
        commands.entity(value).insert(ChildOf(screen));
    }
    let keys = assets.body.text(commands, "Left / Right change    Enter START    Backspace BACK", super::BODY_CAP,
                                super::Look::outlined(super::ABOUT), false, at(96.0, 402.0));
    commands.entity(keys).insert(ChildOf(screen));
    if opened {
        play(commands, sfx, SND_SLIDE);
    }
}

/// The MATCH SETUP screen takes the frame's presses (see the module notes): Up / Down the row (the
/// error sound at the ends), Left / Right its value (round the table), A on to SELECT MISSION (the
/// error sound if the mode has no map) and B back to the submenu, with no sound of its own.
pub fn setup_input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, assets: &Assets2) {
    let step = |v: usize, d: isize, n: usize| ((v as isize + d).rem_euclid(n as isize)) as usize;
    let mut s = menu.setup;
    if asked.up || asked.down {
        let to = if asked.up { s.row.checked_sub(1) } else { Some(s.row + 1).filter(|&r| r < ROWS.len()) };
        match to {
            Some(r) => {
                s.row = r;
                play(commands, sfx, SND_MOVE);
            }
            None => play(commands, sfx, SND_ERROR),
        }
    } else if asked.left || asked.right {
        let d = if asked.left { -1 } else { 1 };
        match s.row {
            0 => s.time = step(s.time, d, TIMES.len()),
            1 => s.score = step(s.score, d, SCORES.len()),
            2 => s.bots = step(s.bots, d, BOTS.len()),
            _ => s.member = step(s.member, d, MEMBERS.len()),
        }
        play(commands, sfx, SND_CHANGE);
    } else if asked.select {
        if assets.maps.get(&menu.kind).is_some_and(|m| !m.is_empty()) {
            play(commands, sfx, SND_SELECT);
            play(commands, sfx, SND_OPEN);
            menu.screen = Screen::Missions;
        } else {
            play(commands, sfx, SND_ERROR);
        }
    } else if asked.back {
        menu.screen = Screen::Multiplayer;
    }
    menu.setup = s;
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
                text(0x08E7_9DB2, campaign::STYLE_TYPED, 0xF6AD_E596), text(0x1CC9_F852, STYLE_HEADING, 0xEE9A_A66D), text(0x0E49_A355, STYLE_HEADING, 0xF30D_DF1D),
                text(0xEA3D_AF3C, STYLE_ABOUT, 0x186B_7D31), text(0xF4A5_6B36, STYLE_ABOUT, 0xF7C4_F81F), text(0x1632_AC3F, STYLE_ABOUT, 0x1A83_55C0),
                text(0x1412_FA83, STYLE_ABOUT, 0x1919_859B), text(0x0744_F955, STYLE_ABOUT, 0x0A4F_864D),
                item(0x0FBE_73AA, 0, ItemKind::Picture { set: 0xFF6B_96DA, piece: None }, &[]),
                item(0x0499_CE0B, 0xE35B_B628, ItemKind::Legend { button: 1, label: 2 }, &[]),
                item(0x0E82_04A4, 0xE35B_B628, ItemKind::Legend { button: 3, label: 4 }, &[]),
            ],
        }
    }

    /// The derived menu keeps CAMPAIGN TYPE's first two buttons, in their places (not moved up
    /// into the first heading's), as DEATHMATCH and SQUAD DEATHMATCH with their descriptions, and
    /// the title, the panel and the legends; the other three buttons, their descriptions and both
    /// heading bands (SINGLE XBOX CONSOLE and SYSTEM LINK) go.
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
        for gone in [0x1632_AC3F, 0x1412_FA83, 0x0744_F955, 0x1CC9_F852, 0x0E49_A355, 0x1B74_4C5F, 0xFE2C_7D14, 0xED7A_7EC2] {
            assert!(menu.items.iter().all(|i| i.name != gone), "{gone:08x} stays");
        }
        assert!(menu.items.iter().all(|i| i.style != STYLE_HEADING));
        for kept in [0x08E7_9DB2, 0x0FBE_73AA, 0x0499_CE0B, 0x0E82_04A4] {
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

    /// The MATCH SETUP screen's choices become the match's rules (#420): no limit and Tex by
    /// default, then the tables' entries in rows 0 and 1, round the tables.
    #[test]
    fn setup_choices_are_the_rules() {
        let mut s = Setup::default();
        assert_eq!((s.row, s.time, s.score, s.bots, s.member), (0, 0, 0, 0, 3), "an open match, no bots, Tex");
        assert!(!s.rules().any());
        assert_eq!(s.rules().character, Some(3));
        s.time = 2;
        s.score = 3;
        assert_eq!(s.rules().time_limit, Some(600.0), "10 minutes");
        assert_eq!(s.rules().score_limit, Some(15));
        assert_eq!((s.value(0), s.value(1), s.value(3)), ("10 MIN".to_string(), "15 KILLS".to_string(), "TEX".to_string()));
        s.time = 0;
        s.member = 0;
        assert_eq!((s.value(0), s.value(3)), ("NONE".to_string(), "BRUTUS".to_string()));
        // (a row's index past its table is held to it)
        s.time = TIMES.len() + 1;
        assert_eq!((s.value(0), s.rules().time_limit), ("5 MIN".to_string(), Some(300.0)));
    }

    /// The BOT SQUADS row (#420): none by default, one to three more squads, and the rules carry
    /// the count (play_bots.rs spawns them).
    #[test]
    fn the_bot_squads_row_is_the_rule() {
        let mut s = Setup::default();
        assert_eq!((s.bots, s.value(2), s.rules().bots), (0, "NONE".to_string(), 0));
        s.bots = 1;
        assert_eq!((s.value(2), s.rules().bots), ("1 SQUAD".to_string(), 1));
        s.bots = 3;
        assert_eq!((s.value(2), s.rules().bots), ("3 SQUADS".to_string(), 3));
        // (a row's index past its table is held to it)
        s.bots = BOTS.len();
        assert_eq!((s.value(2), s.rules().bots), ("NONE".to_string(), 0));
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
