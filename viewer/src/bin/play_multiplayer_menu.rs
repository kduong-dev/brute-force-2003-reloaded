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
//! **The match's screens** (#420), in the order the game's own do (a capture of its SQUAD
//! DEATHMATCH flow, docs/menu.md): SELECT MISSION first (the map, from the submenu), then the
//! mode's rules panel (the score and time limits), then SELECT SQUAD (two teams, each assigned one
//! of the game's squad deathmatch squads: Brute Force, Ferguson Platoon and the rest), then JOIN
//! SQUAD DEATHMATCH (the team the player joins and the member of that squad they play). The game's
//! own words are used where the string table has them (the rules panel's labels and lines, the
//! join page's), so what the demo adds is its drawing of them, its rows and values:
//! drawn as the submenu is (CAMPAIGN TYPE's panel, an outlined typed title, the menu's fonts and
//! colours), each screen's rows' labels left and their values right, the selected row in the
//! menu's selected colour. Up / Down move between rows, Left / Right change the row's value, A
//! goes on and B back; the choices are kept in the `Menu` across screens, and `start_loading`
//! hands them to the match as `match_rules::Rules` (play_match.rs runs them, play_bots.rs puts the
//! other team up).
//!
//! The squads come from the game's own list (game-options' `<squad>` entries, the five the game
//! offers, then the demo's own three (character_class::SQUADS); a member is one of a squad's
//! four (game-options' character names), and the member the player picks is the squad slot they
//! play (`Rules::character`), the other three their squadmates. The other team is a squad of
//! `Player`s on the team table's TEAM1, set to fight (play_bots.rs).
//!
//! Test hooks: play_menu.rs's BF_MENU_SCREEN=multiplayer with BF_MENU_PICK=<0|1> shows the submenu
//! on that entry (`setup`, `teams` and `members` show those screens); BF_MENU_PRESS drives them
//! (with BF_CAPTURE: menu_NNN.png).

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
            // (the map first, as the game's own flow has it: SELECT MISSION, then its rules)
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

// ---- MATCH SETUP (#420, the demo's) ----

/// The rows of the match's limits screen, in order: the two limits, as the game's own SQUAD
/// DEATHMATCH rules panel has them (score above time), by their string-table labels.
pub const ROWS: [u32; 2] = [S_SCORE_LIMIT, S_TIME_LIMIT];
/// The time limits it offers (min): 5 to 30, then none (the game opens on 5:00).
pub const TIMES: [Option<u32>; 6] = [Some(5), Some(10), Some(15), Some(20), Some(30), None];
/// The score limits it offers (points): none, then 5 to 25 (the game opens on NO LIMIT).
pub const SCORES: [Option<u32>; 6] = [None, Some(5), Some(10), Some(15), Some(20), Some(25)];
/// The rows' tops on the 640 x 480 screen, under the title and the rule line.
const ROW_Y: [f32; 2] = [196.0, 240.0];

/// The game's own words of its match screens (string-table ids, from the SQUAD DEATHMATCH sequence
/// of a capture of the game: see docs/menu.md): the rules panel's labels and lines, and the join
/// page's.
const S_SCORE_LIMIT: u32 = 0xE031_7C75; // "SCORE LIMIT:"
const S_TIME_LIMIT: u32 = 0x1258_7991; // "TIME LIMIT:"
const S_RULES_SCORE: u32 = 0xE46F_F524; // "Match ends when score limit or time limit is reached."
const S_RULES_KILL: u32 = 0xEDD5_70FA; // "Match ends when kill limit or time limit is reached."
const S_SQUAD_TIP: u32 = 0xF020_1E83; // "Kill all members of the enemy squad to score a point!"
const S_SELECT_SQUAD: u32 = 0xE8FD_0364; // "SELECT SQUAD"
const S_JOIN_SQUAD: u32 = 0xEE68_F808; // "JOIN SQUAD DEATHMATCH"
const S_JOIN_DM: u32 = 0x189B_ABE2; // "JOIN DEATHMATCH"
const S_SELECT_PROFILE: u32 = 0xE24A_4C5A; // "Select Profile"
const S_SELECT_TEAM: u32 = 0xFF2C_953C; // "Select Team"
const S_SELECT_CHARACTER: u32 = 0xEB9E_AE95; // "Select Character"
const S_DEFAULT: u32 = 0x0A6D_E845; // "DEFAULT"
const S_VS: u32 = 0x109C_A2FD; // "VS"
const S_READY: u32 = 0xE74A_AC22; // "Controller %d is ready!\nPress START to begin!"

/// A string of the string table (the game's), or `fallback` where the data has none.
fn word(strings: &HashMap<u32, String>, id: u32, fallback: &str) -> String {
    strings.get(&id).filter(|w| !w.is_empty()).cloned().unwrap_or_else(|| fallback.to_string())
}

/// The heading font's capital height a title needs to fit the panel (the game's own titles are
/// smaller where they're long: JOIN SQUAD DEATHMATCH is the longest of these).
fn title_cap(font: &super::AtlasFont, word: &str) -> f32 {
    let w = font.width(word, font.cap);
    if w > 560.0 {
        font.cap * 560.0 / w
    } else {
        font.cap
    }
}

/// The MATCH SETUP screen's choices: the row the keys are on, and each row's index in its table.
/// `Default` is an open match (no limit either way).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Setup {
    pub row: usize,
    pub time: usize,
    pub score: usize,
}

impl Default for Setup {
    fn default() -> Self {
        Setup { row: 0, time: 0, score: 0 }
    }
}

impl Setup {
    /// A row's value as the screen says it (the table's entry, held to it): the game's NO LIMIT, a
    /// score, or a time as m:00.
    pub fn value(&self, row: usize) -> String {
        match row {
            0 => SCORES[self.score % SCORES.len()].map_or_else(|| "NO LIMIT".into(), |s| s.to_string()),
            _ => TIMES[self.time % TIMES.len()].map_or_else(|| "NO LIMIT".into(), |m| format!("{m}:00")),
        }
    }

    /// The limits a map started from this screen plays by (play_match.rs); the squads and the
    /// member are the squad pages' (play_menu.rs `Menu::rules`).
    pub fn rules(&self) -> crate::match_rules::Rules {
        crate::match_rules::Rules {
            time_limit: TIMES[self.time % TIMES.len()].and_then(crate::match_rules::Rules::minutes),
            score_limit: SCORES[self.score % SCORES.len()],
            ..Default::default()
        }
    }
}

/// Draw the match's limits screen as the game's own SQUAD DEATHMATCH rules panel (see the module
/// notes): the panel and the typed title (the mode's own word, the game's), the game's rule line
/// under it, the two rows with their labels left and their values right, squad deathmatch's tip
/// line, and the keys' line.
pub fn build_setup(commands: &mut Commands, screen: Entity, assets: &Assets2, strings: &HashMap<u32, String>, sfx: &Sfx,
                   setup: &Setup, mode: usize, opened: bool) {
    let at = |x: f32, y: f32| Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), ..default() };
    panel(commands, screen, assets);
    let cap = assets.font.cap;
    let mode = mode.min(MODES.len() - 1);
    let title_word = strings.get(&MODES[mode].word).cloned().unwrap_or_default();
    let title = assets.font.text(commands, &title_word, title_cap(&assets.font, &title_word), super::Look { color: super::ABOUT, outline: None, typed: true }, false,
                                 at(90.0, 50.0 + (38.0 - assets.font.height("A", cap)) * 0.5));
    commands.entity(title).insert(ChildOf(screen));
    // the rule line, the game's (kill limit in deathmatch, score limit in squad deathmatch)
    let rules = word(strings, if mode == 0 { S_RULES_KILL } else { S_RULES_SCORE }, "");
    let line = assets.body.text(commands, &rules, super::BODY_CAP, super::Look::outlined(super::ABOUT), false, at(96.0, 132.0));
    commands.entity(line).insert(ChildOf(screen));
    for (row, id) in ROWS.iter().enumerate() {
        let colour = if row == setup.row { super::SELECTED } else { super::TEAL };
        let label = assets.font.text(commands, &word(strings, *id, ""), cap, super::Look::outlined(colour), true, at(96.0, ROW_Y[row]));
        commands.entity(label).insert(ChildOf(screen));
        let value = assets.font.text(commands, &setup.value(row), cap, super::Look::outlined(colour), true,
            Node { position_type: PositionType::Absolute, right: Val::Px(super::SCREEN.x - 508.0), top: Val::Px(ROW_Y[row]), ..default() });
        commands.entity(value).insert(ChildOf(screen));
    }
    if mode == 1 {
        // (squad deathmatch's tip, the game's)
        let tip = assets.body.text(commands, &word(strings, S_SQUAD_TIP, ""), super::BODY_CAP,
                                   super::Look::outlined(super::ABOUT), false, at(96.0, 330.0));
        commands.entity(tip).insert(ChildOf(screen));
    }
    let keys = assets.body.text(commands, "Left / Right change    Enter NEXT    Backspace BACK", super::BODY_CAP,
                                super::Look::outlined(super::ABOUT), false, at(96.0, 402.0));
    commands.entity(keys).insert(ChildOf(screen));
    if opened {
        play(commands, sfx, SND_SLIDE);
    }
}

/// Move the limits screen's selected row's value one step round its table (its Left / Right), in
/// `ROWS`' order: the score's first, the time's second.
fn step_setup(s: &mut Setup, d: isize) {
    let step = |v: usize, d: isize, n: usize| ((v as isize + d).rem_euclid(n as isize)) as usize;
    match s.row {
        0 => s.score = step(s.score, d, SCORES.len()),
        _ => s.time = step(s.time, d, TIMES.len()),
    }
}

/// The MATCH SETUP screen takes the frame's presses (see the module notes): Up / Down the row (the
/// error sound at the ends), Left / Right its value (round the table), A on to SELECT SQUAD (the
/// error sound if the mode has no map) and B back to SELECT MISSION, with no sound of its own.
pub fn setup_input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, assets: &Assets2) {
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
        step_setup(&mut s, if asked.left { -1 } else { 1 });
        play(commands, sfx, SND_CHANGE);
    } else if asked.select {
        if assets.maps.get(&menu.kind).is_some_and(|m| !m.is_empty()) {
            play(commands, sfx, SND_SELECT);
            play(commands, sfx, SND_OPEN);
            // (deathmatch has no squads to pick: its join page's one character is the pick)
            menu.screen = if menu.kind == LEVEL_DEATHMATCH { Screen::Members } else { Screen::Teams };
        } else {
            play(commands, sfx, SND_ERROR);
        }
    } else if asked.back {
        menu.screen = Screen::Missions;
    }
    menu.setup = s;
}

// ---- SELECT SQUAD and SELECT MEMBER (#420, the demo's) ----

/// A squad the front end's squad page offers, as the game data has it (#420, play_menu.rs
/// `gather`): its name, the names of its four members (in the game's own order, the order their
/// types go in when a map starts: play_bots.rs `team_types`) — game-options' five squad
/// deathmatch squads, then the demo's three (character_class::SQUADS) — and its portrait's texture
/// (`GameSquad::image`, or the Kingman Posse's cut picture).
pub struct SquadData {
    pub name: String,
    pub members: Vec<String>,
    pub picture: Option<super::Rgba>,
}

/// A squad as the page draws it (play_menu.rs `install`): `SquadData` with its portrait an image,
/// at its size in the page's picture box.
pub struct SquadRow {
    pub name: String,
    pub members: Vec<String>,
    pub picture: Option<Handle<Image>>,
    pub picture_size: Vec2,
}

/// The box a squad's portrait fits in on the 640 x 480 page: the panel's right-hand column, one
/// picture on each team's row (the game's stand there at about 86 x 105: four members together).
pub const SQUAD_PICTURE: Vec2 = Vec2::new(132.0, 100.0);
/// The pictures' column's centre across the page (the game's: 380 to 466).
const SQUAD_PICTURE_X: f32 = 423.0;

/// The four characters the front end offers, in the demo's CHARACTERS order (play.rs: Brutus,
/// Flint, Hawk and Tex): a deathmatch's single pick names them so, and they are the squad page's
/// fallback where a squad's members aren't in the data. The index is the squad slot the player
/// controls (`Rules::character`).
pub const CHARACTERS: [&str; 4] = ["Brutus", "Flint", "Hawk", "Tex"];
/// The squad page's rows: the player's team (team 0's squad) and the other's, the game's words.
pub const TEAM_ROWS: [&str; 2] = ["Team 1", "Team 2"];
/// The squad page's rows' tops, under the title (VS between them), roomy enough for a picture on
/// each row between them (the game's lines are at 191 and 269 on its 640 x 480 screen).
const TEAM_ROW_Y: [f32; 2] = [185.0, 295.0];
/// JOIN SQUAD DEATHMATCH's rows' tops (Select Team, Select Character).
const JOIN_ROW_Y: [f32; 2] = [196.0, 240.0];

/// The squad page's choices, kept in the `Menu` across screens: the row the keys are on, the squad
/// each team plays as (a place in the menu's squad list, play_menu.rs `Assets2::squads`), the team
/// the player joins (0 or 1: the join page's Select Team) and the member of that team's squad they
/// play as (3, Tex, the campaign's start character).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Teams {
    pub row: usize,
    pub squads: [usize; 2],
    pub team: usize,
    pub member: usize,
}

impl Default for Teams {
    fn default() -> Self {
        Teams { row: 0, squads: [0, 1], team: 0, member: 3 }
    }
}

impl Teams {
    /// The squad row `row`'s team plays as (its place held to the list's length).
    fn squad<'a>(&self, row: usize, squads: &'a [SquadRow]) -> Option<&'a SquadRow> {
        squads.get(self.squads[row % 2] % squads.len().max(1))
    }

    /// The team the player joins (0 or 1).
    fn mine(&self) -> usize {
        self.team % 2
    }

    /// A row's value as the screen says it: its squad's name (the game's own words).
    pub fn value(&self, row: usize, squads: &[SquadRow]) -> String {
        self.squad(row, squads).map_or_else(|| "NONE".into(), |s| s.name.clone())
    }

    /// The player's squad's members (empty with no squad list).
    pub fn members<'a>(&self, squads: &'a [SquadRow]) -> &'a [String] {
        self.squad(self.mine(), squads).map_or(&[], |s| s.members.as_slice())
    }

    /// The member the keys are on (`member`, held to the player's squad's members).
    pub fn picked(&self, squads: &[SquadRow]) -> usize {
        self.member % self.members(squads).len().max(1)
    }

    /// The picks held to the squad list (`BF_MENU_GO`, a list of one).
    fn resolve(&mut self, squads: &[SquadRow]) {
        let n = squads.len().max(1);
        self.squads = [self.squads[0] % n, self.squads[1] % n];
        self.team %= 2;
        self.member = self.picked(squads);
    }
}

/// The panel the demo's screens are drawn on (the menu's pieces at their places, alpha 180).
fn panel(commands: &mut Commands, screen: Entity, assets: &Assets2) {
    let at = |x: f32, y: f32| Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), ..default() };
    for (picture, place, size) in &assets.panel {
        commands.spawn((ImageNode { flip_y: true, ..ImageNode::new(picture.clone()) }, super::Tint::new(Color::WHITE.with_alpha(super::PANEL_ALPHA)),
                        Node { width: Val::Px(size.x), height: Val::Px(size.y), ..at(place.x, place.y) }, ChildOf(screen)));
    }
}

/// Draw SELECT SQUAD as the game's page (see the module notes): the panel and its typed title (the
/// game's word), TEAM 1 and TEAM 2 with their squads' names between arrows (on the selected row),
/// VS between them, their squads' portraits (the game's `<squad>` pictures, each team's in the
/// panel's right-hand column) and the keys' line. Their locks aren't drawn: the game's are only
/// for a squad another player has.
pub fn build_teams(commands: &mut Commands, screen: Entity, assets: &Assets2, strings: &HashMap<u32, String>, teams: &Teams,
                   sfx: &Sfx, opened: bool) {
    let at = |x: f32, y: f32| Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), ..default() };
    panel(commands, screen, assets);
    let cap = assets.font.cap;
    let title_word = word(strings, S_SELECT_SQUAD, "SELECT SQUAD");
    let title = assets.font.text(commands, &title_word, title_cap(&assets.font, &title_word),
                                 super::Look { color: super::ABOUT, outline: None, typed: true }, false,
                                 at(90.0, 50.0 + (38.0 - assets.font.height("A", cap)) * 0.5));
    commands.entity(title).insert(ChildOf(screen));
    for row in 0..TEAM_ROWS.len() {
        let on = row == teams.row % TEAM_ROWS.len();
        let colour = if on { super::SELECTED } else { super::TEAL };
        // (the team's word small at the left, as the game has it; the squad's name between the
        // arrows the game puts round the selected one)
        let label = assets.body.text(commands, TEAM_ROWS[row], super::BODY_CAP, super::Look::outlined(colour), false,
                                     at(161.0, TEAM_ROW_Y[row]));
        commands.entity(label).insert(ChildOf(screen));
        let value = teams.value(row, &assets.squads);
        let w = assets.body.width(&value, super::BODY_CAP);
        let x = 292.0 - w * 0.5;
        let name = assets.body.text(commands, &value, super::BODY_CAP, super::Look::outlined(colour), false, at(x, TEAM_ROW_Y[row]));
        commands.entity(name).insert(ChildOf(screen));
        if on {
            let left = assets.body.text(commands, "<", super::BODY_CAP, super::Look::outlined(colour), true, at(x - 30.0, TEAM_ROW_Y[row]));
            commands.entity(left).insert(ChildOf(screen));
            let right = assets.body.text(commands, ">", super::BODY_CAP, super::Look::outlined(colour), true, at(x + w + 12.0, TEAM_ROW_Y[row]));
            commands.entity(right).insert(ChildOf(screen));
        }
        // (the squad's portrait, the game's: its four members together, in the panel's right-hand
        // column, centred on the team's own row)
        if let Some(squad) = teams.squad(row, &assets.squads) {
            if let Some(picture) = &squad.picture {
                let p = squad.picture_size;
                let top = TEAM_ROW_Y[row] + 4.0 - p.y * 0.5;
                commands.spawn((ImageNode { flip_y: true, ..ImageNode::new(picture.clone()) }, super::Tint::new(Color::WHITE),
                                Node { width: Val::Px(p.x), height: Val::Px(p.y), ..at(SQUAD_PICTURE_X - p.x * 0.5, top) },
                                ChildOf(screen)));
            }
        }
    }
    let vs = word(strings, S_VS, "VS");
    let w = assets.font.width(&vs, cap);
    let vs = assets.font.text(commands, &vs, cap, super::Look::outlined(super::ABOUT), true, at(super::SCREEN.x * 0.5 - w * 0.5, 229.0));
    commands.entity(vs).insert(ChildOf(screen));
    let keys = assets.body.text(commands, "Left / Right change    Enter NEXT    Backspace BACK", super::BODY_CAP,
                                super::Look::outlined(super::ABOUT), false, at(96.0, 402.0));
    commands.entity(keys).insert(ChildOf(screen));
    if opened {
        play(commands, sfx, SND_SLIDE);
    }
}

/// Draw JOIN SQUAD DEATHMATCH, the game's join page for the one player the demo has (see the module
/// notes): the panel and its typed title, the profile line (Select Profile DEFAULT), Select Team
/// (which of the two squads you play in, with its team's number) and Select Character (one of that
/// squad's four; arrows on the selected row, as the game's quadrant has), the game's "ready" line
/// and the keys' line. A deathmatch's page (`squad`: false) has Select Character alone, one of the
/// four characters: its mode has no squads.
pub fn build_join(commands: &mut Commands, screen: Entity, assets: &Assets2, strings: &HashMap<u32, String>, teams: &Teams,
                  squad: bool, sfx: &Sfx, opened: bool) {
    let at = |x: f32, y: f32| Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), ..default() };
    panel(commands, screen, assets);
    let cap = assets.font.cap;
    // (the game's own word for each mode's join page)
    let (title_id, title_fallback) = if squad { (S_JOIN_SQUAD, "JOIN SQUAD DEATHMATCH") } else { (S_JOIN_DM, "JOIN DEATHMATCH") };
    let title_word = word(strings, title_id, title_fallback);
    let title = assets.font.text(commands, &title_word, title_cap(&assets.font, &title_word),
                                 super::Look { color: super::ABOUT, outline: None, typed: true }, false,
                                 at(90.0, 50.0 + (38.0 - assets.font.height("A", cap)) * 0.5));
    commands.entity(title).insert(ChildOf(screen));
    let profile = format!("{}  {}", word(strings, S_SELECT_PROFILE, "Select Profile"), word(strings, S_DEFAULT, "DEFAULT"));
    let line = assets.body.text(commands, &profile, super::BODY_CAP, super::Look::outlined(super::ABOUT), false, at(96.0, 140.0));
    commands.entity(line).insert(ChildOf(screen));
    // (the picks: the team you join, then a member of its squad; a deathmatch's one character)
    let picks: Vec<(String, String)> = if squad {
        let picked = teams.picked(&assets.squads);
        let member = teams.members(&assets.squads).get(picked).cloned()
            .unwrap_or_else(|| CHARACTERS[picked % CHARACTERS.len()].to_string());
        vec![(word(strings, S_SELECT_TEAM, "Select Team"), format!("{} ({})", teams.value(teams.mine(), &assets.squads), teams.mine() + 1)),
             (word(strings, S_SELECT_CHARACTER, "Select Character"), member)]
    } else {
        vec![(word(strings, S_SELECT_CHARACTER, "Select Character"), CHARACTERS[teams.member % CHARACTERS.len()].to_string())]
    };
    let rows = picks.len();
    for (row, (label, value)) in picks.into_iter().enumerate() {
        let on = row == teams.row % rows;
        let colour = if on { super::SELECTED } else { super::TEAL };
        // (the game's quadrant draws these small, in the body font: the team's number's brackets
        // aren't in the heading font's set either)
        let label = assets.body.text(commands, &label, super::BODY_CAP, super::Look::outlined(colour), false,
                                     at(96.0, JOIN_ROW_Y[row] + 6.0));
        commands.entity(label).insert(ChildOf(screen));
        let w = assets.body.width(&value, super::BODY_CAP);
        let shown = assets.body.text(commands, &value, super::BODY_CAP, super::Look::outlined(colour), false,
                                     at(508.0 - w, JOIN_ROW_Y[row] + 6.0));
        commands.entity(shown).insert(ChildOf(screen));
        if on {
            let left = assets.body.text(commands, "<", super::BODY_CAP, super::Look::outlined(colour), false, at(508.0 - w - 30.0, JOIN_ROW_Y[row] + 6.0));
            commands.entity(left).insert(ChildOf(screen));
            let right = assets.body.text(commands, ">", super::BODY_CAP, super::Look::outlined(colour), false, at(516.0, JOIN_ROW_Y[row] + 6.0));
            commands.entity(right).insert(ChildOf(screen));
        }
    }
    let ready = word(strings, S_READY, "Controller 1 is ready! Press START to begin!").replace("%d", "1").replace('\n', " ");
    let line = assets.body.text(commands, &ready, super::BODY_CAP, super::Look::outlined(super::ABOUT), false, at(96.0, 352.0));
    commands.entity(line).insert(ChildOf(screen));
    let keys = assets.body.text(commands, "Left / Right change    Enter START    Backspace BACK", super::BODY_CAP,
                                super::Look::outlined(super::ABOUT), false, at(96.0, 402.0));
    commands.entity(keys).insert(ChildOf(screen));
    if opened {
        play(commands, sfx, SND_SLIDE);
    }
}

/// SELECT SQUAD takes the frame's presses: Up / Down the row (the error sound at the ends),
/// Left / Right the row's squad (round the list: the game's five, then the demo's three), A on to
/// SELECT MEMBER, B back to MATCH SETUP.
pub fn teams_input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, assets: &Assets2) {
    let step = |v: usize, d: isize, n: usize| ((v as isize + d).rem_euclid(n as isize)) as usize;
    let mut t = menu.teams;
    t.resolve(&assets.squads);
    if asked.up || asked.down {
        let to = if asked.up { t.row.checked_sub(1) } else { Some(t.row + 1).filter(|&r| r < TEAM_ROWS.len()) };
        match to {
            Some(r) => {
                t.row = r;
                play(commands, sfx, SND_MOVE);
            }
            None => play(commands, sfx, SND_ERROR),
        }
    } else if asked.left || asked.right {
        let n = assets.squads.len();
        if n == 0 {
            play(commands, sfx, SND_ERROR);
        } else {
            let d = if asked.left { -1 } else { 1 };
            let row = t.row % TEAM_ROWS.len();
            t.squads[row] = step(t.squads[row], d, n);
            play(commands, sfx, SND_CHANGE);
        }
    } else if asked.select {
        if assets.squads.is_empty() {
            play(commands, sfx, SND_ERROR);
        } else {
            play(commands, sfx, SND_SELECT);
            play(commands, sfx, SND_OPEN);
            menu.screen = Screen::Members;
        }
    } else if asked.back {
        menu.screen = Screen::Setup;
    }
    menu.teams = t;
}

/// JOIN SQUAD DEATHMATCH takes the frame's presses: Up / Down the pick (Select Team, Select
/// Character; the error sound at the ends), Left / Right its value (the team you join, round the
/// two, or your squad's four members), A starts the map picked on SELECT MISSION with the rules
/// the screens set (play_menu.rs `start_loading`), B back to SELECT SQUAD.
pub fn join_input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, assets: &Assets2,
                  next: &mut NextState<super::super::AppState>, squad: bool) {
    let step = |v: usize, d: isize, n: usize| ((v as isize + d).rem_euclid(n as isize)) as usize;
    let mut t = menu.teams;
    t.resolve(&assets.squads);
    let rows = if squad { 2 } else { 1 };
    if asked.up || asked.down {
        let to = if asked.up { t.row.checked_sub(1) } else { Some(t.row + 1).filter(|&r| r < rows) };
        match to {
            Some(r) => {
                t.row = r;
                play(commands, sfx, SND_MOVE);
            }
            None => play(commands, sfx, SND_ERROR),
        }
    } else if asked.left || asked.right {
        let d = if asked.left { -1 } else { 1 };
        if !squad {
            // (deathmatch: the character alone)
            t.member = step(t.member % CHARACTERS.len(), d, CHARACTERS.len());
            play(commands, sfx, SND_CHANGE);
        } else if t.row == 0 {
            // (the team you join: the other team's squad becomes the enemy)
            t.team = step(t.mine(), d, 2);
            t.member = t.picked(&assets.squads);
            play(commands, sfx, SND_CHANGE);
        } else if t.members(&assets.squads).is_empty() {
            play(commands, sfx, SND_ERROR);
        } else {
            let n = t.members(&assets.squads).len();
            t.member = step(t.picked(&assets.squads), d, n);
            play(commands, sfx, SND_CHANGE);
        }
    } else if asked.select {
        let kind = menu.kind;
        let maps = assets.maps.get(&kind);
        let at = menu.pick.get(&kind).copied().unwrap_or(0) % maps.map_or(1, |m| m.len().max(1));
        match maps.and_then(|m| m.get(at)) {
            Some(m) => {
                let (file, solo) = (m.file.clone(), kind == LEVEL_DEATHMATCH);
                play(commands, sfx, SND_SELECT);
                play(commands, sfx, SND_OPEN);
                menu.teams = t;
                super::start_loading(commands, menu, next, &file, solo, false);
                return;
            }
            None => play(commands, sfx, SND_ERROR),
        }
    } else if asked.back {
        // (a deathmatch came straight from MATCH SETUP: it has no squad page behind it)
        menu.screen = if squad { Screen::Teams } else { Screen::Setup };
    }
    menu.teams = t;
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

    /// The limits screen's choices become the match's rules (#420), as the game's rules panel has
    /// them: NO LIMIT for the score and 5:00 for the time by default, then the tables' entries.
    #[test]
    fn setup_choices_are_the_limits() {
        let mut s = Setup::default();
        assert_eq!((s.row, s.score, s.time), (0, 0, 0), "NO LIMIT, 5:00: the game's own defaults");
        assert_eq!((s.value(0), s.value(1)), ("NO LIMIT".to_string(), "5:00".to_string()));
        assert_eq!((s.rules().score_limit, s.rules().time_limit), (None, Some(300.0)));
        s.time = 2;
        s.score = 3;
        assert_eq!(s.rules().time_limit, Some(900.0), "15 minutes");
        assert_eq!(s.rules().score_limit, Some(15));
        assert_eq!((s.value(0), s.value(1)), ("15".to_string(), "15:00".to_string()));
        s.time = TIMES.len() - 1;
        assert_eq!((s.value(1), s.rules().time_limit), ("NO LIMIT".to_string(), None));
        // (a row's index past its table is held to it)
        s.time = TIMES.len() + 1;
        assert_eq!((s.value(1), s.rules().time_limit), ("10:00".to_string(), Some(600.0)));
    }

    /// The limits screen's keys change the row they are on (#420): SCORE LIMIT's steps the score
    /// and leaves the time, TIME LIMIT's the time and leaves the score. (The screen had them the
    /// other way round, so a press on SCORE LIMIT moved the time.)
    #[test]
    fn the_limits_screen_changes_the_row_the_keys_are_on() {
        let mut s = Setup::default();
        step_setup(&mut s, 1);
        assert_eq!((s.score, s.time), (1, 0), "SCORE LIMIT: 5, the time still 5:00");
        step_setup(&mut s, -1);
        assert_eq!((s.score, s.time), (0, 0), "back to NO LIMIT");
        s.row = 1;
        step_setup(&mut s, 1);
        assert_eq!((s.score, s.time), (0, 1), "TIME LIMIT: 10:00, the score still NO LIMIT");
        s.time = TIMES.len() - 1;
        step_setup(&mut s, 1);
        assert_eq!(s.time, 0, "round the time table past NO LIMIT to 5:00");
    }

    /// The two squads pages' choices (#420): each team's squad from the menu's list, the team the
    /// player joins, and that team's squad's member — all held to what the data has.
    #[test]
    fn the_squad_pages_pick_the_two_teams() {
        let row = |name: &str, members: &[&str]| SquadRow {
            name: name.into(), members: members.iter().map(|m| m.to_string()).collect(),
            picture: None, picture_size: Vec2::ZERO,
        };
        let squads = vec![
            row("Brute Force", &["Tex", "Flint", "Brutus", "Hawk"]),
            row("Feral Hunting Pack", &["A", "B", "C", "D"]),
            row("Ferguson Platoon", &["E", "F", "G", "H"]),
        ];
        let mut t = Teams::default();
        t.resolve(&squads);
        assert_eq!((t.value(0, &squads), t.value(1, &squads)), ("Brute Force".to_string(), "Feral Hunting Pack".to_string()));
        assert_eq!(t.members(&squads).len(), 4);
        assert_eq!(t.picked(&squads), 3, "Tex, the squad's last slot");
        // (the team the player joins picks whose squad they play in)
        t.squads = [2, 1];
        t.team = 1;
        assert_eq!((t.mine(), t.value(t.mine(), &squads)), (1, "Feral Hunting Pack".to_string()));
        assert_eq!(t.members(&squads)[0], "A");
        t.team = 0;
        assert_eq!((t.mine(), t.members(&squads)[3].as_str()), (0, "H"));
        t.member = 5;
        assert_eq!(t.picked(&squads), 1, "held to the squad's four");
        // (a list of one: both teams on it; an empty list: nothing to pick)
        t.squads = [1, 2];
        t.member = 3;
        t.resolve(&squads[..1]);
        assert_eq!((t.squads, t.picked(&squads[..1])), ([0, 0], 3));
        t.squads = [1, 2];
        t.team = 5;
        t.resolve(&[]);
        assert_eq!((t.mine(), t.value(0, &[]), t.members(&[]).len()), (1, "NONE".to_string(), 0));
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
