//! The tutorial's panels (#224, step 5 of #28; viewer/docs/hints.md): the menu-trigger and the
//! game-options menus it opens over the game, BASIC CONTROLS, HEADS-UP DISPLAY (HUD), ADVANCED
//! HUD, WEAPONS, ADVANCED WEAPONS, INVENTORY, SPECIAL ABILITIES, SELECTING CHARACTERS and SQUAD
//! COMMANDS (470 x 360 at (85, 60)). A child of play_triggers.rs: the class is the runtime's; this
//! module's systems draw a panel when one opens, stop the game's clock under it and take its
//! buttons. Written from default.xbe's code (the addresses are in the comments and in
//! viewer/docs/code-map.md); take04, take13 and take14 confirm what the buttons do and the clock.
//!
//! The menu-trigger (vtable 0x396108; parser FUN_00208140: `menu`, `Title`, `message`):
//!  * OPERATE (its handler 0x2082d0) opens its menu (FUN_002081d0: the menu system's open, the
//!    session's vtable +0x78, slot 0, with the trigger's callback at the menu's +0xac and the
//!    menu's id kept at the trigger's +0x4c; the two names 0xf3b06f04 and 0x0ec41ef2 open a message
//!    box of its Title and message instead), then sends OPERATED (its iD, the event's subject and
//!    data) to its targets;
//!  * the menu's buttons call the callback (vtable 0x3961d4), which sends to the trigger's targets
//!    (iD 0, no subject, data -1): A MENU_ACCEPT (+0, once the menu has closed on its h_1966467f
//!    item: 0x100ea0), B MENU_BACK_UP (+4), X MENU_X_BUTTON (+8), Y MENU_Y_BUTTON (+0xc, which then
//!    closes the menu: the menu system's +0x14). On the tutorial's panels only A and Y close it
//!    (take14: B, BACK, X and RB on BASIC CONTROLS leave it up, the clock stopped at 31082 ms);
//!  * its update (0x208430) sends FINISHED (25, its iD) once its menu isn't open any more (the
//!    menu system's open list, its vtable +0x64), then its FIRST_UPDATE while enabled. A name that
//!    game-options doesn't define (the tutorial's trigger_menu509, h_194eab9a) opens nothing:
//!    OPERATED, never FINISHED.
//!
//! The panels all send to router h_0d29c80f, whose routers pass MENU_Y_BUTTON on as USER1 to every
//! panel (h_10820d7c: each one's USER1 disables it, "Y skip tutorial menus") and as USER2 to
//! timer464 (router524: 2 s, ADVANCED HUD's and the hint chain's); a sound-trigger plays on it.
//! take13: Y on BASIC CONTROLS, no HUD, ADVANCED HUD or WEAPONS panel after it, the hint chain
//! from timer464.
//!
//! The game's clock stops while a panel is up, as under the objectives menu (take04's watch log:
//! clk 31081 from 4.9 s to the A at 12.2 s under BASIC CONTROLS, 44033 under HUD, 46003 under
//! ADVANCED HUD): `frame_dt` 0, no press reaches the character or the triggers' buttons. It shows
//! whole in its first frame (take04 4.9 s). The frame a trigger opens one in runs to its end (the
//! world updated with the frame's time, as the game's world update goes on past the trigger's
//! handler); the clock stops from the next frame's start, and runs again from the start of the
//! frame the panel closes in, so the triggers' timers and the player's clock lose the same frames.
//!
//! A panel is drawn from game-options' data (bf/menus.rs): its pictures first (the shade over the
//! whole screen: no texture, its style h_0db074a5's colour, black at 180, take04's background at
//! 0.28 of its value under it; the frame h_0d0ce2e2, the objectives menu's; the sample pictures:
//! the controller, the gun, the rings), then its texts in their styles' colours (240 the
//! headings, 155 the lines, orange the "10/20" sample), laid out in their boxes as the game's
//! static text does (FUN_0010a880, by the records' own advances: the lines break where take04's
//! do), a line every (the record's line height + 1) units, then its legends, "A Accept" and "Y
//! skip tutorial menus" (the button in the icon font's 12-point glyphs, on their own atlas, as the
//! game draws them). The words are drawn in the demo's HUD font (play_text.rs) at capital heights
//! measured on take04, not the game's 10 / 12 / 16-point glyphs, a few percent wider: their exact
//! look is a later ticket's.
//!
//! Controls: A is Enter, Space or the pad's A; Y is Q (the demo's key for the Y button) or the
//! pad's Y. While a panel is up no key, mouse or pad press reaches the game.
//!
//! Test hook: BF_TEST_PANEL=<a|y>[,<open s>][;...]: the n-th panel to show is closed with that
//! button once it's been up that long (1 s by default; real time, or the capture's steps under
//! BF_CAPTURE, as the game's clock stands still); the last entry stands for the panels after it.

use std::collections::HashMap;

use bevy::input::InputSystem;
use bf_viewer::bf::menus::{GameMenu, ItemKind, PicturePiece};

use super::super::briefing;
use super::super::hud;
use super::super::text::{game_text, Align, GameFont};
use super::*;

/// The two menu names FUN_002081d0 opens as a message box of the trigger's Title and message
/// (the menu system's +0x34 and +0x38) instead of a game-options menu. No tutorial trigger names
/// them; not drawn here.
const MESSAGE_BOXES: [u32; 2] = [0xF3B0_6F04, 0x0EC4_1EF2];
/// The panels over the HUD and the hint lines, as the objectives menu (20).
const PANEL_Z: i32 = 20;
/// BF_TEST_PANEL's default time up (s).
const TEST_UP: f32 = 1.0;
/// A panel text's look by its style's point size (the body font h_0c626fd5): the capitals'
/// height in the demo's HUD font, the line pitch (the game's record's line height + 1:
/// FUN_000ccae0, FUN_0010abb0; the records 0x3c0b68, 0x3c2088, 0x3c35a8, 0x3c4ac8 have 16, 19,
/// 23, 25) and where the capitals start below a line's top. Measured on take04's BASIC CONTROLS
/// (10.0 s, 1.5 px a unit): 16 points (the title, the headings) capitals 15.3 tall from 5.3 under
/// their boxes ("BASIC CONTROLS", "Movement": the demo's 15.5 spans them to a unit, as the
/// objectives menu's title fit); 12 (the lines, "TIP:") 12.0 tall from 4.0-4.3; 10 (the tips)
/// 10.0 from 4.0; 14 the objectives menu's (13.3, 5.3: play_objectives.rs).
const TEXT_SIZES: [(f32, f32, f32, f32); 4] = [(10.0, 10.0, 17.0, 4.0), (12.0, 12.0, 20.0, 4.2), (14.0, 13.3, 24.0, 5.3), (16.0, 15.5, 26.0, 5.3)];
/// The words' first ink right of their box's left edge (take04 BASIC CONTROLS: "Use", "Movement"
/// 1.3, "TIP:" 0.0, "Enemies" 0.7): a fit.
const TEXT_INSET: f32 = 1.0;
/// The body font's pen advances (each character's offset plus advance, by which FUN_000ccae0
/// measures) for the characters 32-255 of its 10, 12 and 16-point records (0x3c0b68, 0x3c2088,
/// 0x3c4ac8, read from default.xbe; the 14-point record's are the briefings' table,
/// `briefing::glyph`): what a panel's text wraps by, as the game's static text does
/// (FUN_0010a880), so its lines break where take04's do. The words are drawn in the demo's HUD
/// font, a few percent wider at 10 and 12 points ("Use Left thumbstick to" 154.7 units to the
/// take's 145.3): a line can run a little past its box.
const ADVANCE_10: [u8; 224] = [
    3, 4, 5, 7, 7, 9, 8, 3, 5, 5, 6, 7, 4, 4, 3, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 4, 4, 7, 7, 7, 6,
    11, 7, 8, 7, 8, 7, 6, 8, 8, 4, 6, 7, 6, 9, 8, 8, 7, 8, 7, 7, 6, 8, 7, 10, 7, 6, 6, 4, 6, 4, 7, 7,
    8, 6, 7, 6, 7, 6, 4, 7, 7, 3, 3, 6, 3, 10, 7, 7, 7, 7, 5, 5, 4, 7, 6, 9, 6, 6, 5, 4, 7, 4, 7, 13,
    13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13,
    3, 4, 7, 7, 7, 7, 7, 7, 8, 8, 5, 7, 7, 4, 8, 7, 5, 7, 5, 5, 8, 7, 8, 4, 8, 5, 5, 7, 10, 10, 10, 6,
    7, 7, 7, 7, 7, 7, 10, 7, 7, 7, 7, 7, 4, 4, 4, 4, 8, 8, 8, 8, 8, 8, 8, 7, 8, 8, 8, 8, 8, 6, 7, 7,
    6, 6, 6, 6, 6, 6, 9, 6, 6, 6, 6, 6, 3, 3, 3, 3, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 7, 6,
];
/// (the 12-point record's, see ADVANCE_10)
const ADVANCE_12: [u8; 224] = [
    4, 4, 6, 8, 8, 12, 10, 3, 6, 6, 8, 8, 4, 5, 4, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 5, 4, 8, 8, 8, 7,
    13, 9, 9, 9, 10, 8, 8, 9, 10, 5, 7, 9, 7, 11, 10, 10, 9, 10, 9, 8, 7, 10, 9, 12, 8, 8, 8, 5, 7, 5, 8, 8,
    10, 8, 8, 7, 8, 8, 5, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 8, 6, 7, 5, 8, 7, 11, 8, 8, 7, 5, 8, 5, 8, 16,
    16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
    4, 4, 8, 8, 8, 9, 8, 8, 10, 10, 6, 9, 8, 5, 10, 8, 6, 8, 6, 6, 10, 8, 9, 5, 10, 6, 6, 9, 13, 13, 13, 7,
    9, 9, 9, 9, 9, 9, 13, 9, 8, 8, 8, 8, 5, 5, 5, 5, 10, 10, 10, 10, 10, 10, 10, 8, 10, 10, 10, 10, 10, 8, 9, 9,
    8, 8, 8, 8, 8, 8, 11, 7, 8, 8, 8, 8, 4, 4, 4, 4, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
];
/// (the 16-point record's, see ADVANCE_10)
const ADVANCE_16: [u8; 224] = [
    5, 6, 8, 11, 11, 15, 12, 4, 7, 7, 10, 11, 6, 7, 5, 9, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 6, 6, 11, 11, 11, 10,
    17, 12, 12, 12, 13, 11, 10, 12, 13, 6, 10, 12, 10, 14, 13, 13, 11, 13, 12, 11, 10, 13, 12, 16, 11, 10, 10, 6, 9, 6, 11, 11,
    13, 10, 11, 10, 11, 10, 6, 11, 11, 6, 6, 10, 6, 16, 11, 11, 11, 11, 8, 9, 7, 11, 9, 14, 10, 10, 9, 7, 11, 7, 11, 21,
    21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
    5, 6, 11, 11, 11, 12, 11, 11, 13, 13, 8, 11, 11, 7, 13, 11, 7, 11, 9, 9, 13, 11, 12, 6, 13, 9, 8, 11, 17, 17, 17, 10,
    12, 12, 12, 12, 12, 12, 16, 12, 11, 11, 11, 11, 6, 6, 6, 6, 13, 13, 13, 13, 13, 13, 13, 11, 13, 13, 13, 13, 13, 10, 11, 12,
    10, 10, 10, 10, 10, 10, 15, 10, 10, 10, 10, 10, 6, 6, 6, 6, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 10, 11, 10,
];
/// The legends' button icons (style h_e35bb628's first font: the icon font h_0e38a4ca at 12
/// points): FUN_000cd5d0 registers that size's record, 0x3c5fe8, on an atlas of its own,
/// h_0e233fad (128 x 256, v as stored), whose discs are smaller and outlined in black, as take04's
/// and take11's legends are (not the front end's or the hint lines' 16-point discs). The
/// objectives menu's legends are the same style's (play_objectives.rs).
pub(crate) const ICON12_ATLAS: u32 = 0x0E23_3FAD;
/// The 12-point record's line height (its +4): a glyph's cell is one more tall (FUN_000ccd20).
const ICON12_LINE: f32 = 16.0;
/// The 12-point record's glyphs 'A' to 'J' (from its +0x1c, 0x18 bytes each, read from
/// default.xbe): the cell's corners on the atlas in pixels, one texel left of and above the
/// record's (FUN_000ccd20 takes one of the record's texels, 128 x 256 here, off each corner:
/// (coordinate x size - 1) / size), then the offset, width and advance. A, B, C, D are the A, B, X
/// and Y buttons, as the 16-point glyphs (play_hints.rs ICON_GLYPHS). The discs reach above their
/// cells, so they're drawn cut flat on top, as the takes show them. E's cell, starting a texel left
/// of the atlas, is clamped to its edge.
const ICON12_GLYPHS: [(char, f32, f32, f32, f32, i8, u8, u8); 10] = [
    ('A', 27.22, 33.13, 44.35, 50.20, 1, 16, 17),
    ('B', 45.36, 33.13, 62.50, 50.20, 1, 16, 17),
    ('C', 63.50, 33.13, 80.64, 50.20, 1, 16, 17),
    ('D', 81.65, 33.13, 98.78, 50.20, 1, 16, 17),
    ('E', 0.00, 50.20, 12.10, 67.27, 1, 12, 13),
    ('F', 13.11, 50.20, 26.21, 67.27, 1, 12, 13),
    ('G', 27.22, 50.20, 52.42, 67.27, 0, 24, 26),
    ('H', 53.43, 50.20, 74.59, 67.27, 3, 20, 23),
    ('I', 75.60, 50.20, 90.72, 67.27, 1, 14, 15),
    ('J', 91.72, 50.20, 106.84, 67.27, 1, 14, 15),
];
/// A legend (style h_e35bb628 at (12, 332) and (95, 332) in the panels): the icon's pen from the
/// item's corner (its cell drawn from the pen plus its offset, (width + 1) x 17 units, as
/// FUN_000ccd20 draws a glyph), then the word, its capitals 12 tall (the style's second font, the
/// body font at 12 points: take04's "Accept" and "skip tutorial menus" 12.0) from 4.0 down, its
/// first ink 18.3 in (take04 10.0 s). The pen is a fit: the cells drawn over take04's BASIC
/// CONTROLS and HUD and take11's SQUAD COMMANDS match best 1 unit down and a half unit left of the
/// corner (the A disc's best, the Y disc's a half unit right of it; the demo's frames, set side by
/// side with the takes', line up to the pixel), the shifted cells fitting better than the record's
/// own. The 1 down may be the 17-unit glyph centred in the item's 20 (a guess).
const ICON_PEN: Vec2 = Vec2::new(-0.5, 1.0);
const WORD_IN: Vec2 = Vec2::new(18.3, 4.0);
const LEGEND_CAP: f32 = 12.0;

/// A legend's button icon (style h_e35bb628, see ICON12_GLYPHS): the node of `letter`'s cell on the
/// 12-point atlas `image`, its pen at the legend item's `corner` (+ ICON_PEN, + the glyph's
/// offset), (width + 1) x 17 units; None for a letter the font hasn't got. The panels' legends and
/// the objectives menu's (play_objectives.rs) draw theirs so.
pub(crate) fn legend_icon(image: Handle<Image>, corner: Vec2, letter: char) -> Option<(Node, ImageNode)> {
    let &(_, u0, v0, u1, v1, offset, width, _) = ICON12_GLYPHS.iter().find(|g| g.0 == letter)?;
    let at = corner + ICON_PEN + Vec2::new(offset as f32, 0.0);
    Some((hud::at(at.x, at.y, width as f32 + 1.0, ICON12_LINE + 1.0), ImageNode { rect: Some(Rect::new(u0, v0, u1, v1)), ..ImageNode::new(image) }))
}

/// A menu-trigger's own fields (+0x40-+0x4c): its menu, Title and message (string ids), whether
/// game-options defines the menu, and whether it's open (+0x4c holds the menu's id while it is).
#[derive(Clone, Debug)]
pub(crate) struct MenuTrigger {
    menu: u32,
    title: u32,
    message: u32,
    exists: bool,
    open: bool,
}

impl MenuTrigger {
    /// A menu-trigger as the level starts (its constructor 0x2083c0: closed).
    pub(crate) fn new(menu: u32, title: u32, message: u32, game: &Game) -> MenuTrigger {
        MenuTrigger { menu, title, message, exists: game.menus.menus.contains_key(&menu), open: false }
    }
}

/// The buttons that close a tutorial panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Button {
    /// A: the menu closes on its accept item, then MENU_ACCEPT
    Accept,
    /// Y: MENU_Y_BUTTON, then the menu closes
    Y,
}

/// A panel open: the trigger that opened it, its menu, its nodes once drawn, which panel it is
/// (the n-th to show, for BF_TEST_PANEL), and how long it's been up (real time).
struct Panel {
    trigger: usize,
    menu: u32,
    root: Option<Entity>,
    nth: usize,
    up: f32,
}

/// The panels' side of the triggers' state (`HintWorld::panels`), one per level: the panels open
/// (the last on top), the ones closed since the triggers' last frame with their buttons, whether
/// the clock was stopped for them, BF_TEST_PANEL's entries and how many panels have shown.
#[derive(Default)]
pub(crate) struct PanelWorld {
    stack: Vec<Panel>,
    closed: Vec<(usize, Button)>,
    paused: bool,
    tests: Option<Vec<(Button, f32)>>,
    shown: usize,
}

/// The pictures the panels draw, loaded once: each texture as an image with its size (the
/// legends' icon atlas among them).
#[derive(Resource, Default)]
struct PanelImages {
    textures: HashMap<u32, Option<(Handle<Image>, u32, u32)>>,
}

/// A panel's nodes (its root).
#[derive(Component)]
struct PanelRoot;

/// The panels' buttons as soon as the frame's input is in (before the objectives menu, which
/// mustn't open on a BACK pressed over a panel: take14), and a panel drawn once its trigger has
/// opened it (after the triggers' frame).
pub fn plugin(app: &mut App) {
    app.init_resource::<PanelImages>()
        .add_systems(OnExit(AppState::Playing), close_on_exit)
        .add_systems(PreUpdate, input.after(InputSystem).before(super::super::objectives::ObjectivesControls).run_if(in_state(AppState::Playing)))
        .add_systems(Update, show.after(super::run).run_if(in_state(AppState::Playing)));
}

/// BF_TEST_PANEL's entries (see the module notes).
fn read_tests() -> Vec<(Button, f32)> {
    let Ok(v) = std::env::var("BF_TEST_PANEL") else { return vec![] };
    v.split(';').filter_map(|e| {
        let mut it = e.split(',').map(str::trim);
        let button = match it.next()?.to_ascii_lowercase().as_str() {
            "a" => Button::Accept,
            "y" => Button::Y,
            _ => return None,
        };
        Some((button, it.next().and_then(|s| s.parse().ok()).unwrap_or(TEST_UP)))
    }).collect()
}

/// A panel's title for the log: its "Title" text, else its name.
fn title(game: &Game, menu: u32) -> String {
    game.menus.menus.get(&menu).and_then(|m| m.items.iter().find(|i| i.name == h("Title"))).and_then(|i| match i.kind {
        ItemKind::Text(id) => game.strings.get(&id).cloned(),
        _ => None,
    }).map_or(format!("h_{menu:08x}"), |t| format!("{t} (h_{menu:08x})"))
}

/// The top panel's buttons (see the module notes), as soon as the frame's input is in: the
/// game's clock stops at the start of the first frame after a panel was drawn; A or Y closes it
/// (its nodes go, the press is kept for its trigger's update: `menu_update`); every key, mouse
/// and pad press of the frame is taken, so nothing under it sees one (the hint lines' buttons
/// read the frame after this). When the last panel closes, the game's clock runs again.
#[allow(clippy::too_many_arguments)]
pub(super) fn input(mut commands: Commands, mut tr: ResMut<Triggers>, mut keys: ResMut<ButtonInput<KeyCode>>,
                    mut mouse: ResMut<ButtonInput<MouseButton>>, mut pads: Query<&mut Gamepad>, time: Res<Time>, game: Res<GameData>) {
    let clock = tr.clock;
    let log = tr.hints.log;
    let p = &mut tr.hints.panels;
    if p.tests.is_none() {
        p.tests = Some(read_tests());
    }
    if !p.stack.last().is_some_and(|top| top.root.is_some()) {
        return;
    }
    if !p.paused {
        p.paused = true;
        set_paused(PAUSE_PANELS, true);
    }
    let Some(top) = p.stack.last_mut() else { return };
    top.up += if std::env::var("BF_CAPTURE").is_ok() { capture_step() } else { time.delta_secs() };
    let pad = |b: GamepadButton| pads.iter().any(|g| g.just_pressed(b));
    let mut button = if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) || pad(GamepadButton::South) {
        Some(Button::Accept)
    } else if keys.just_pressed(KeyCode::KeyQ) || pad(GamepadButton::North) {
        Some(Button::Y)
    } else {
        None
    };
    let tests = p.tests.as_deref().unwrap_or(&[]);
    if let Some(&(b, after)) = tests.get(top.nth.saturating_sub(1)).or(tests.last()) {
        if top.up >= after {
            button = button.or(Some(b));
        }
    }
    // (the frame's presses are the panel's)
    let pressed: Vec<KeyCode> = keys.get_just_pressed().copied().collect();
    for k in pressed {
        keys.clear_just_pressed(k);
    }
    let clicked: Vec<MouseButton> = mouse.get_just_pressed().copied().collect();
    for b in clicked {
        mouse.clear_just_pressed(b);
    }
    for mut g in &mut pads {
        let down: Vec<GamepadButton> = g.digital().get_just_pressed().copied().collect();
        for b in down {
            g.digital_mut().clear_just_pressed(b);
        }
    }
    let Some(button) = button else { return };
    let Some(panel) = p.stack.pop() else { return };
    if let Some(root) = panel.root {
        commands.entity(root).despawn();
    }
    if log {
        println!("t {clock:.2}: panel {} closed with {} after {:.2} s", title(&game.0, panel.menu), if button == Button::Y { "Y" } else { "A" }, panel.up);
    }
    p.closed.push((panel.trigger, button));
    if p.stack.is_empty() && p.paused {
        p.paused = false;
        set_paused(PAUSE_PANELS, false);
    }
}

/// After the triggers' frame: a panel just opened is drawn (unless another menu has the game's
/// clock stopped, the objectives menu: then it waits for that one to close); the clock stops at
/// the next frame's start (`input`).
///
/// A panel isn't held back while a cutscene has the world: the menu-trigger opens its menu
/// straight through the menu system (FUN_002081d0), without the cutscene bit test (+0xc50 & 0x10)
/// that the players' own menus go through (FUN_0008c8c0, the objectives and pause menus:
/// play_script.rs `in_cutscene`). No take shows one opening during a cutscene (WEAPONS opens at
/// the end of a voice line, h_eb8d3d96's FINISHED, after the patrol's script: take05, take07).
fn show(mut commands: Commands, mut tr: ResMut<Triggers>, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>,
        mut cache: ResMut<PanelImages>, font: Option<Res<GameFont>>) {
    let Some(font) = font else { return };
    if !tr.hints.panels.stack.iter().any(|p| p.root.is_none()) || paused_by_other(PAUSE_PANELS) {
        return;
    }
    let (clock, log) = (tr.clock, tr.hints.log);
    let p = &mut tr.hints.panels;
    for k in 0..p.stack.len() {
        if p.stack[k].root.is_some() {
            continue;
        }
        let Some(menu) = game.0.menus.menus.get(&p.stack[k].menu).cloned() else { continue };
        p.shown += 1;
        p.stack[k].nth = p.shown;
        p.stack[k].root = Some(build(&mut commands, &mut game.0, &mut images, &mut cache, &font, &menu));
        if log {
            println!("t {clock:.2}: panel {} shows (the game's clock stops)", title(&game.0, menu.name));
        }
    }
}

/// Leaving the map: the panels' hold on the clock let go if they had it (their nodes go with the
/// map's).
fn close_on_exit(mut tr: ResMut<Triggers>) {
    if tr.hints.panels.paused {
        tr.hints.panels.paused = false;
        set_paused(PAUSE_PANELS, false);
    }
    tr.hints.panels.stack.clear();
}

/// A game texture as a UI image with its size, loaded once (None if it isn't found).
fn texture(cache: &mut PanelImages, game: &mut Game, images: &mut Assets<Image>, name: u32) -> Option<(Handle<Image>, u32, u32)> {
    cache.textures.entry(name).or_insert_with(|| {
        let (w, hgt, px) = game.texture_rgba(name)?;
        Some((images.add(Image::new(bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 },
                                    bevy::render::render_resource::TextureDimension::D2, px,
                                    bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())), w, hgt))
    }).clone()
}

/// A text's look by its point size (TEXT_SIZES: the nearest): the size it's taken as, the
/// capitals' height, the line pitch and where the capitals start.
fn text_size(points: f32) -> (f32, f32, f32, f32) {
    TEXT_SIZES.iter().min_by(|a, b| (a.0 - points).abs().total_cmp(&(b.0 - points).abs())).copied().unwrap_or(TEXT_SIZES[1])
}

/// How far a character moves the pen in the body font at `points` (ADVANCE_10 ...; 14 points
/// the briefings' record); out of the records' range (32-255) it's 0x82's, as FUN_000ccae0 takes
/// it.
fn advance(points: f32, c: char) -> f32 {
    let code = if (32..=255).contains(&(c as u32)) { c as u32 } else { 0x82 };
    let i = (code - 32) as usize;
    match points as i32 {
        10 => ADVANCE_10[i] as f32,
        12 => ADVANCE_12[i] as f32,
        16 => ADVANCE_16[i] as f32,
        _ => briefing::glyph(c).1,
    }
}

/// A text's lines in a box `width` wide at `points` (the briefings' FUN_0010a880 by this size's
/// advances: `briefing::lay_out_with`), each without its trailing spaces and line break.
fn wrap(text: &str, width: f32, points: f32) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    briefing::lay_out_with(&chars, width, &|c| advance(points, c)).into_iter()
        .map(|l| l.into_iter().collect::<String>().trim_end().to_string()).collect()
}

/// A dark colour's alpha as Bevy's UI blends it (in linear light) for the console's look (which
/// blends the stored sRGB values): 1 - (1 - a)^2.2, as play_stealth.rs `srgb_alpha`.
fn dark_alpha(a: f32) -> f32 {
    1.0 - (1.0 - a.clamp(0.0, 1.0)).powf(2.2)
}

/// A panel's nodes, drawn from its menu's data (see the module notes): in a root over the game,
/// its 640 x 480 screen 4:3 at the window's height.
fn build(commands: &mut Commands, game: &mut Game, images: &mut Assets<Image>, cache: &mut PanelImages, font: &GameFont, menu: &GameMenu) -> Entity {
    let root = commands.spawn((PanelRoot, Name::new("tutorial panel"), GlobalZIndex(PANEL_Z),
                               Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                      justify_content: JustifyContent::Center, ..default() })).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(root))).id();
    let place = |at: Vec2, size: Vec2| hud::at(at.x, at.y, size.x, size.y);
    // the pictures, in the data's order
    for item in &menu.items {
        let ItemKind::Picture { set, piece } = &item.kind else { continue };
        let corner = menu.at + item.at;
        let pieces: Vec<PicturePiece> = if *set != 0 { game.menus.pictures.get(set).cloned().unwrap_or_default() } else { piece.iter().cloned().collect() };
        for pc in pieces {
            let at = corner + pc.at;
            if pc.texture == 0 {
                // (no texture: a quad in its item's style's h_ed70ff4f, else its own colour)
                let c = game.menus.styles.get(&item.style).and_then(|s| s.edge).unwrap_or(pc.color);
                let colour = Color::srgb_u8(c[0], c[1], c[2]).with_alpha(dark_alpha(c[3] as f32 / 255.0));
                commands.spawn((place(at, pc.size), BackgroundColor(colour), ChildOf(screen)));
                continue;
            }
            let Some((image, w, hgt)) = texture(cache, game, images, pc.texture) else { continue };
            let size = if pc.stretched { pc.size } else { Vec2::new(w as f32, hgt as f32) };
            let tint = Color::srgba_u8(pc.color[0], pc.color[1], pc.color[2], pc.color[3]);
            // (game pictures are stored bottom row first: drawn flipped, and a flipped piece not,
            // as the objectives menu's frame)
            commands.spawn((place(at, size), ImageNode { flip_y: !pc.flipped, ..ImageNode::new(image) }.with_color(tint), ChildOf(screen)));
        }
    }
    // the texts
    for item in &menu.items {
        let ItemKind::Text(id) = item.kind else { continue };
        let Some(text) = game.strings.get(&id).filter(|t| !t.is_empty()) else { continue };
        let style = game.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let (points, cap, pitch, cap_below) = text_size(style.font.map_or(12.0, |f| f.1));
        let rgba = style.color.unwrap_or([255; 4]);
        let colour = Color::srgba_u8(rgba[0], rgba[1], rgba[2], rgba[3]);
        let lines = wrap(text, item.size.x, points);
        let corner = menu.at + item.at;
        let top = match style.align.1 {
            2 => corner.y + (item.size.y - lines.len() as f32 * pitch) / 2.0,
            1 => corner.y + item.size.y - lines.len() as f32 * pitch,
            _ => corner.y,
        };
        let (x, align) = match style.align.0 {
            2 => (corner.x + item.size.x / 2.0, Align::Centre),
            1 => (corner.x + item.size.x, Align::Right),
            _ => (corner.x + TEXT_INSET, Align::Left),
        };
        for (n, line) in lines.iter().enumerate() {
            let y = top + n as f32 * pitch + cap_below - font.cap_drop(cap);
            commands.spawn((game_text(line, x, y, cap, colour, align), ChildOf(screen)));
        }
    }
    // the legends: the button's icon (its string's letter in the 12-point icon font), then the word
    let icons = texture(cache, game, images, ICON12_ATLAS).map(|(image, ..)| image);
    for item in &menu.items {
        let ItemKind::Legend { button, label } = item.kind else { continue };
        let corner = menu.at + item.at;
        let letter = game.strings.get(&button).and_then(|s| s.chars().next()).unwrap_or(' ');
        if let Some((node, image)) = icons.clone().and_then(|image| legend_icon(image, corner, letter)) {
            commands.spawn((node, image, ChildOf(screen)));
        }
        let style = game.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let rgba = style.color.unwrap_or([255; 4]);
        let word = game.strings.get(&label).cloned().unwrap_or_default();
        let y = corner.y + WORD_IN.y - font.cap_drop(LEGEND_CAP);
        commands.spawn((game_text(&word, corner.x + WORD_IN.x, y, LEGEND_CAP, Color::srgba_u8(rgba[0], rgba[1], rgba[2], rgba[3]), Align::Left),
                        ChildOf(screen)));
    }
    root
}

impl Triggers {
    /// 0x2082d0: a menu-trigger. Disabled, its base actions; enabled, each entry that answers:
    /// OPERATE opens its menu (`open_menu`) and sends OPERATED (its iD, the event's subject and
    /// data); the rest the base's; the entry counted.
    pub(super) fn menu(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            self.open_menu(i);
            let id = self.live[i].t.id;
            self.send(i, Ev { signal: sig::OPERATED, id, ..ev });
            self.entry_used(i, j, &ev);
        }
    }

    /// FUN_002081d0: menu-trigger `i` opens its menu over the game, shown by `show` after the
    /// triggers' frame; a message box (MESSAGE_BOXES) isn't drawn, and a name game-options doesn't
    /// have opens nothing. Asked again while it's open, nothing more opens (the demo keeps one
    /// panel a trigger).
    fn open_menu(&mut self, i: usize) {
        let ClassState::Menu(m) = &mut self.live[i].state else { return };
        let (menu, title, message, exists, open) = (m.menu, m.title, m.message, m.exists, m.open);
        let why = if MESSAGE_BOXES.contains(&menu) {
            Some(format!("a message box (Title h_{title:08x}, message h_{message:08x}): not drawn"))
        } else if !exists {
            Some(format!("game-options has no menu h_{menu:08x}: nothing opens"))
        } else if open {
            Some("its menu is open already".into())
        } else {
            m.open = true;
            None
        };
        if let Some(why) = why {
            if self.log || self.hints.log {
                println!("t {:.2}: {} OPERATE: {why}", self.clock, self.who(i));
            }
            return;
        }
        self.hints.panels.stack.push(Panel { trigger: i, menu, root: None, nth: 0, up: 0.0 });
        if self.log {
            println!("t {:.2}: {} opens the panel h_{menu:08x}", self.clock, self.who(i));
        }
    }

    /// 0x208430: a menu-trigger's update. Its menu open and closed now: the buttons it was closed
    /// with go out first (the callback's sends at the press: MENU_ACCEPT or MENU_Y_BUTTON, iD 0,
    /// no subject, data -1), then FINISHED (its iD); then its FIRST_UPDATE while enabled.
    pub(super) fn menu_update(&mut self, i: usize) {
        let open = matches!(&self.live[i].state, ClassState::Menu(m) if m.open);
        if open && !self.hints.panels.stack.iter().any(|p| p.trigger == i) {
            let presses: Vec<Button> = self.hints.panels.closed.iter().filter(|c| c.0 == i).map(|c| c.1).collect();
            self.hints.panels.closed.retain(|c| c.0 != i);
            for b in presses {
                let signal = if b == Button::Y { sig::MENU_Y_BUTTON } else { sig::MENU_ACCEPT };
                self.send(i, Ev { signal, id: 0, subject: 0, data: -1 });
            }
            if let ClassState::Menu(m) = &mut self.live[i].state {
                m.open = false;
            }
            let id = self.live[i].t.id;
            self.send(i, Ev { signal: sig::FINISHED, id, subject: 0, data: -1 });
        }
        self.base_update(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SQUAD COMMANDS' two 12-point paragraphs (boxes 181 and 187 wide) break where take11's
    /// (123.2 s) do, by the 12-point record's advances.
    #[test]
    fn squad_commands_wrap_as_the_take() {
        assert_eq!(wrap("Press and hold the D-pad in any direction to access the Squad Commands Menu.", 181.0, 12.0),
                   ["Press and hold the D-pad in", "any direction to access the", "Squad Commands Menu."]);
        assert_eq!(wrap("Press the D-pad in the direction of the character icon on the radar to toggle their selection.", 187.0, 12.0),
                   ["Press the D-pad in the", "direction of the character", "icon on the radar to toggle", "their selection."]);
    }
}
