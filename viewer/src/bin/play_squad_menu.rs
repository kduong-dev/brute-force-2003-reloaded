//! The Squad Command menu, the members' selection and the Move To cursor (#319, the second slice
//! of #33; todo/33-squad-commands leads §2 and §4, its takes take01-take03, take08, take09 and #35
//! take18). The orders it gives are #318's command modes (play_orders.rs `order`); the lines they
//! say are #320's. Every function cited is in viewer/docs/code-map.md ("HUD").
//!
//!  - **Selecting** (the HUD's squad list, FUN_000c7cf0, its d-pad handler 0xc7e20): each d-pad
//!    direction is a radar portrait's member (up Tex, right Flint, down Brutus, left Hawk: the
//!    type's slot, h_072ab4af; the demo's keys 4, 2, 1 and 3, as play_hints.rs maps them). A press
//!    of a member that may be selected (FUN_000c8470: there, alive, in the world; the controlled
//!    one too, but not while the cursor is up) plays h_0c0d7e4d and flashes the chevrons at its
//!    portrait (FUN_000c4dd0). Held 0.25 s it toggles the member's flag (FUN_000c8320 mode 0,
//!    FUN_000c8640: h_fac5d6b4 on, h_0d93fbd9 off); let go before, a tap: with the menu closed it
//!    switches to that member (mode 1: the hand-over, play.rs `squad_control`), with it open it
//!    toggles. Each action locks its direction out for 0.3 s. A flagged member that may no
//!    longer be selected (dead) drops out with h_0d93fbd9.
//!  - **The menu** (the squad command widget, FUN_000c0040, its update FUN_000c0fb0): it opens
//!    as soon as a member is flagged (FUN_000c0940 -> FUN_000c10d0), pausing the game (single
//!    player), with h_1bdc87d1, and fades in over 0.3 s (FUN_000c0cf0); the radar slides to the
//!    middle (FUN_000c2be0, play_hud.rs), the crosshair and the item box fade out (FUN_000c17c0).
//!    The orders (FUN_000c0d90): the first pressed of B Fire At Will, A Cover Me, Y Stand Ground,
//!    X Move To, Black Medkit, White Toggle Ability (table 0x3c9008 -> FUN_000bffb0); Move To,
//!    Medkit and Toggle first drop the player's own character from the selection; with someone
//!    left, Move To brings up the cursor and the rest give the order (FUN_000c1280), with
//!    h_11d6f86a; then the menu fades out (h_15262e1b unless the cursor is up) and the game runs
//!    again at the fade's end (FUN_000c1150). BACK clears the selection: it closes with h_15262e1b
//!    and h_ffbe0bb9.
//!  - **The cursor** (HUD+0x44, FUN_000bd860, its update 0xbdfb0 and input 0xbe130; the game
//!    runs meanwhile): each frame the world point nearest the centre ray's point whose flags pass
//!    (+0x24 bit 14 set, bit 15 clear, +0x28 bit 2) and that projects inside the screen
//!    (FUN_0004d630 with the filter 0xbdef0); with none, the controlled character's place. A
//!    spinning marker stands on it (the object type h_18709e83, 0xbe5e0 / 0xbe440); the rows "Look
//!    to select position", "[A] accept", "[B] cancel" fade in as the menu goes. A sends Move To
//!    with the point (or, on the character, Cover Me: +0xcc), B cancels (FUN_000bd930).
//!
//! The demo's keys (the product owner's call): 1-4 the d-pad (a tap switches, held 0.25 s
//! selects, open a tap toggles), I Stand Ground, L Fire At Will, K Cover Me, J Move To, B Medkit
//! and V Toggle Ability (the demo's Black and White, play_hints.rs `BINDINGS`), Escape or
//! Backspace BACK; the cursor: the mouse aims, the left button or E accepts, the right button or
//! Escape cancels. A pad (not under BF_BACKGROUND) has the game's own: the d-pad, Y B A X, RB
//! (Black) and LB (White) as xemu maps them, Select (BACK); A and B on the cursor.
//!
//! Test hooks: `BF_TEST_COMMAND="<s>:<what>[:<hold s>];..."` presses at each time (the hooks'
//! clock: the sim time plus the time the menu's pause held it) `<what>`: a member (brutus /
//! flint / hawk / tex or 0-3) holds its d-pad direction for `<hold>` s (0.1 by default, a tap;
//! 0.4 selects), `stand` / `fire` / `cover` / `move` / `medkit` / `toggle` (or `y` / `b` / `a` /
//! `x` / `black` / `white`) presses that order's button, `back` BACK, `accept` / `cancel` the
//! cursor's A and B; `BF_TEST_CURSOR_AT=<x>,<z>` takes the floor there as the cursor's centre ray
//! point (a stand-in for aiming at it); `BF_SQUAD_MENU_LOG=1` logs the presses, the flags, the
//! menu's opening, orders and close, and the cursor's point.

use bevy::ecs::system::SystemParam;
use bf_viewer::bf::level::WorldPoint;
use bf_viewer::level_scene::LevelMaterial;
use brain::Mode;

use super::fonts::{record_text, Across, RecordText, Record, BODY10, BODY12, BODY16};
use super::hud::{flipped, texture, HudImages, HudRoot};
use super::*;

// ---- the squad list (vtable 0x39f264: FUN_000c7cf0) ----

/// A d-pad direction held this long (s) is a hold: the list's +0x7c, 0.25 (FUN_000c7cf0), which
/// the handler's held time +0x78 is compared with (0xc8141). take02: 200 ms presses switched, 350
/// ms ones selected.
const HOLD: f32 = 0.25;
/// Each action locks its direction out for this long (s): FUN_000c8320 sets the slot's +0x60
/// to 0.3 (0x3e99999a), which the handler counts down and reads before it takes a press.
const LOCKOUT: f32 = 0.3;
/// The d-pad's directions in the order the handler reads them (0xc7f6a-0xc8017): right (VB 14,
/// slot 2 Flint), up (13, slot 1 Tex), left (16, slot 4 Hawk), down (15, slot 3 Brutus), as the
/// demo's slots (CHARACTERS: 1 Flint, 3 Tex, 2 Hawk, 0 Brutus) and its keys for them.
const DPAD: [(usize, KeyCode, GamepadButton); 4] = [(1, KeyCode::Digit2, GamepadButton::DPadRight), (3, KeyCode::Digit4, GamepadButton::DPadUp),
                                                    (2, KeyCode::Digit3, GamepadButton::DPadLeft), (0, KeyCode::Digit1, GamepadButton::DPadDown)];
/// The press (0xc81ea: a member that may be selected), the selection on and off (FUN_000c8320
/// after FUN_000c8640; off also when a flagged member drops out, 0xc82dc): common's 2D sounds.
const S_PRESS: u32 = 0x0C0D_7E4D;
const S_ON: u32 = 0xFAC5_D6B4;
const S_OFF: u32 = 0x0D93_FBD9;
/// The chevrons' flash at a press (FUN_000c4dd0 on the portrait's flash widget, its +0x30 set to
/// 1.0): its level falls 2.0 a second (0x40d6a4, 0xc4a5d) and it shows while above 0.
const FLASH_FALL: f32 = 2.0;

// ---- the squad command widget (vtable 0x39f708: FUN_000c0040, FUN_000c0180, FUN_000c0ae0) ----

/// The menu's fade (s): +0x9c, 0.3 (FUN_000c0040), counted up opening and down closing
/// (FUN_000c0cf0). take01: fade 0.034 -> 0.300 in 0.3 s.
const FADE: f32 = 0.3;
/// The menu's sounds (FUN_000bb220, the HUD's 2D player): open (FUN_000c0940), an order
/// (FUN_000c0d90, the cursor's A), closing (FUN_000c1120, FUN_000c0d90's close and FUN_000bd930,
/// none while the cursor is up) and the selection emptied (FUN_000c0940's second).
const S_OPEN: u32 = 0x1BDC_87D1;
const S_ORDER: u32 = 0x11D6_F86A;
const S_CLOSE: u32 = 0x1526_2E1B;
const S_EMPTIED: u32 = 0xFFBE_0BB9;
/// The menu's texts (FUN_000c0180): "Squad Command", "D-Pad Select Char", "Give Order", "Close
/// Menu".
const S_TITLE: u32 = 0xECE7_1B9D;
const S_DPAD: u32 = 0xFEAE_B4FE;
const S_GIVE_ORDER: u32 = 0xF5B1_4961;
const S_CLOSE_MENU: u32 = 0xE645_0981;
/// The six entries in the order index the orders go by (FUN_000c1840's EAX): their labels and
/// 64 x 64 icons (common's textures), the demo's names for the log and hooks.
const ENTRY_LABELS: [u32; 6] = [0x1C8E_3180, 0x1941_EB0E, 0xF09A_0387, 0xFC5C_D98A, 0x02E3_7319, 0x1956_5285];
const ENTRY_ICONS: [u32; 6] = [0xE1CE_A7E9, 0x19ED_0CE8, 0xF8DD_F854, 0xF822_C386, 0xE6D6_7FA3, 0xFE0C_0F73];
const ENTRY_NAMES: [&str; 6] = ["Stand Ground", "Fire At Will", "Cover Me", "Move To", "Medkit", "Toggle Ability"];
/// The order indices: what FUN_000c1280 does with each.
const STAND: usize = 0;
const FIRE: usize = 1;
const COVER: usize = 2;
const MOVE: usize = 3;
const MEDKIT: usize = 4;
const TOGGLE: usize = 5;
/// The entries' places (ints at 0x3c8fa8, a screen of 600 x 400 or more: FUN_000c0ae0 puts each
/// entry's top left there), relative to the widget, which stands at (width / 2 + 20, height / 2).
/// take01 4.0 s: Stand Ground's icon at (390, 129)-(454, 193) units, as these put it.
const ENTRY_AT: [(f32, f32); 6] = [(50.0, -111.0), (110.0, -32.0), (50.0, 47.0), (-10.0, -32.0), (150.0, 52.0), (120.0, 84.0)];
const WIDGET_AT: Vec2 = Vec2::new(320.0 + 20.0, 240.0);
/// An entry's size (FUN_000c1840: 0x40), halved for the two with the flag 1 and +0x70 bit 1
/// (Medkit and Toggle Ability, 0xc0842 / 0xc08b7; their update 0xc19d0): their icons are the
/// Black and White buttons' 32 x 32, their labels beside them, 8 in (bit 2: width / 4), centred on
/// the icon's height; the other four's under the icon, centred (take01: "Toggle Ability" over its
/// disc's right).
const ENTRY_SIZE: f32 = 64.0;
const SMALL_ENTRY: f32 = 32.0;
/// The texts' colour: bytes c0 d7 ff (0x4a5ca0, set by FUN_000c0180 and FUN_000c1840), pale blue;
/// with a shadow (their +0x3b bit 4: black, a unit down and right, as the HUD's other texts).
const BLUE: [u8; 3] = [0xC0, 0xD7, 0xFF];
/// The title stands 30 above Stand Ground's top (FUN_000c0ae0: its y, -111 - 30), centred on the
/// widget less 20 (the screen's middle).
const TITLE_UP: f32 = 30.0;
/// The two side rows stand 38 under Cover Me's icon (its y + 64 + 38): "Give Order" centred on
/// Cover Me, "D-Pad Select Char" as far the other side of the widget less 20; "Close Menu" 26
/// below them, centred (FUN_000c0ae0).
const ROWS_DOWN: f32 = 38.0;
const CLOSE_DOWN: f32 = 26.0;
/// A glyph row (0xf94d225, FUN_000c9b10): its glyphs 3 units lower than its words (FUN_000ca000
/// sets their +6 to 3) and its words GLYPH_GAP after them (+0x70, 4.0).
const GLYPH_DROP: f32 = 3.0;
const GLYPH_GAP: f32 = 4.0;
/// "Give Order"'s glyphs (FUN_000c0180's list {9, 10, 11, 12, 14, 13}: A, B, X, Y, Black, White
/// by FUN_000c9d70's table 0xc9ee4) and "Close Menu"'s ({20}: BACK), as the icon font's letters.
const GIVE_GLYPHS: [char; 6] = ['A', 'B', 'C', 'D', 'J', 'I'];
const CLOSE_GLYPHS: [char; 1] = ['H'];
/// The button icon font at 12 points (FUN_000ca360 with 0: h_0e38a4ca's record 0x3c5fe8, its
/// line 16, registered on the atlas h_0e233fad, 128 x 256, at v x 1 + 0 by FUN_000cd5d0): its
/// glyphs for A, B, X (C), Y (D), BACK (H), White (I) and Black (J), read from default.xbe: the
/// cell's corners on the atlas in pixels, each taken a texel in as the renderer samples them
/// ((size x coordinate - 1) / size, FUN_000ccd20), then the offset, width and advance.
const ICON_ATLAS_12: u32 = 0x0E23_3FAD;
const ICON_LINE_12: f32 = 16.0;
const ICON_GLYPHS_12: [(char, f32, f32, f32, f32, i8, u8, u8); 7] = [
    ('A', 27.22, 33.13, 44.35, 50.20, 1, 16, 17),
    ('B', 45.36, 33.13, 62.50, 50.20, 1, 16, 17),
    ('C', 63.50, 33.13, 80.64, 50.20, 1, 16, 17),
    ('D', 81.65, 33.13, 98.78, 50.20, 1, 16, 17),
    ('H', 53.43, 50.20, 74.59, 67.27, 3, 20, 23),
    ('I', 75.60, 50.20, 90.72, 67.27, 1, 14, 15),
    ('J', 91.72, 50.20, 106.84, 67.27, 1, 14, 15),
];
/// The backdrop (0x17f9dd32, FUN_000c67b0: the whole screen, no texture) multiplies the picture
/// under it, the HUD's bars and panels too, by a grey (0xba950 mode 3: DESTCOLOR / ZERO) of
/// (1 - k) x 175 + 80 with k = (fade / 0.3)^2 (FUN_000c09a0): 80 / 255 when open. take01 measured
/// the sky and the health bar 0.29-0.31 times as bright under it.
const BACKDROP_GREY: (f32, f32) = (175.0, 80.0);
/// The console multiplies the stored sRGB values; Bevy's UI blends in linear light, where the
/// same darkening is a multiply by the grey to this power: drawn as black at 1 - that (a gamma of
/// 2.2, as play_hud.rs's fits).
const SRGB_GAMMA: f32 = 2.2;
/// The menu over the HUD (its root has no z-index) and the hint lines (1), under the radar,
/// which shows over it (play_hud.rs RADAR_Z, take01: the radar undimmed by the backdrop), and the
/// menus (20).
const MENU_Z: i32 = 2;

// ---- the cursor (HUD+0x44, FUN_000bd860, 0xbd970, 0xbdfb0, 0xbe130) ----

/// The cursor's rows (0xbd970): "Look to select position", "accept" with A's glyph and "cancel"
/// with B's (FUN_000ca290 with VB 20 MENU_SELECT and 23 MENU_BACKUP, 0xbddb5 / 0xbddd1).
const S_LOOK: u32 = 0x0226_8885;
const S_ACCEPT: u32 = 0x09C6_306A;
const S_CANCEL: u32 = 0xF719_037C;
/// Where they stand (0xbdce0): the cursor widget sits at the crosshair's centre (0xab390) and
/// places its rows at the HUD safe area's left (51) less its own x + 140 and the safe area's
/// bottom (439) less its own y and 18: so "accept" at (191, 421) on the screen, "Look to select
/// position" 18 above it and "cancel" 15 after "accept"'s row. take08 4.0 s: "Look ..." from x
/// 191.3, the A glyph at 193.
const CURSOR_ROWS_AT: Vec2 = Vec2::new(51.0 + 140.0, 439.0 - 18.0);
const CURSOR_ROW_STEP: f32 = 18.0;
const CURSOR_ROW_GAP: f32 = 15.0;
/// The marker (0xbe5e0): a world object of the type h_18709e83 (its mesh h_fa39b375: a
/// three-sided upside-down pyramid, its tip at the point and 2 m tall, a blue outline round a
/// dark see-through fill), put at the cursor's point each frame (0xbe440) and turned about y by
/// the frame's time (FUN_00039380 with dt): a turn a radian a second (take08 5.0-6.4 s: its
/// inner edge going round).
const MARKER_TYPE: u32 = 0x1870_9E83;
const MARKER_TURN: f32 = 1.0;
/// The 640 x 480 the filter keeps a point's projection in (0xbdef0: 0 to 640 across, 0 to 480
/// down, [0x3a5180] / [0x3a517c]).
const SCREEN: Vec2 = Vec2::new(640.0, 480.0);
/// A world point's +0x28 bit 2, which the filter and FUN_0004d630 ask for: FUN_00059b60 sets it
/// for every point and nothing in the data clears it (what does at run time isn't traced; take08
/// read 4 on every point the cursor met, 6 once taken). The demo takes every point as having it.
const POINT_USABLE: bool = true;

/// The plain world point's type (h_f95afb93, FUN_000ee070's FUN_00059b60 case), for the log:
/// the other kinds (cover, door, patrol points ...) are in the cursor's list too.
const WORLD_POINT_TYPE: u32 = 0xF95A_FB93;
/// The pause's holder bit (play.rs PAUSED): the menu's own hold on the game's clock.
const PAUSE_SQUAD: u32 = super::PAUSE_SQUAD;

/// The menu, the selection and the cursor's state (the squad list's, the squad command
/// widget's and the cursor widget's fields), which the HUD reads (play_hud.rs: the radar's place
/// and highlight, the crosshair's alpha; play_itembox.rs: the box's alpha).
#[derive(Resource)]
pub struct SquadMenu {
    /// the selection flags by slot (CHARACTERS index): the list's +0x58..+0x5c
    flags: [bool; SQUAD],
    /// each slot's lockout (s): +0x60..
    lockout: [f32; SQUAD],
    /// the direction held (+0x80: its slot), how long (+0x78), and whether its press was used
    /// (+0x84: a hold done, or a tap; cleared once nothing is held)
    held: Option<usize>,
    hold: f32,
    consumed: bool,
    /// the press's chevron flash: the slot and its level (FUN_000c4dd0, falling FLASH_FALL a
    /// second)
    flash: Option<(usize, f32)>,
    /// the widget's +0xbc: open (bit 0), the game paused by it (bit 1), fading in (bit 2),
    /// closing (bit 3)
    open: bool,
    paused_game: bool,
    fading_in: bool,
    closing: bool,
    /// its fade (+0x98, 0 to FADE)
    fade: f32,
    /// the cursor is up (+0x7c)
    cursor: bool,
    /// the order last given (+0xa0, an entry index), which stays whole as the rest fades out
    last: Option<usize>,
    /// the cursor's point (+0xac), the world point it's on (+0xc4: an index into the level's
    /// `points`), and whether it's on the controlled character instead (+0xcc: no point found)
    cursor_at: Vec3,
    cursor_point: Option<usize>,
    on_player: bool,
    /// the marker's turn about y (rad)
    marker_turn: f32,
    /// the time the menu's pause has held the game's clock (s): the hooks' clock is the sim time
    /// plus it
    paused_time: f32,
    /// BF_TEST_COMMAND's entries (read once) and whether each has started
    tests: Option<Vec<(TestPress, bool)>>,
    /// BF_SQUAD_MENU_LOG
    log: bool,
}

impl Default for SquadMenu {
    fn default() -> Self {
        SquadMenu { flags: [false; SQUAD], lockout: [0.0; SQUAD], held: None, hold: 0.0, consumed: false, flash: None, open: false, paused_game: false,
                    fading_in: false, closing: false, fade: 0.0, cursor: false, last: None, cursor_at: Vec3::ZERO, cursor_point: None, on_player: false,
                    marker_turn: 0.0, paused_time: 0.0, tests: None, log: std::env::var("BF_SQUAD_MENU_LOG").is_ok() }
    }
}

impl SquadMenu {
    /// Whether slot `c`'s member is selected (FUN_000c86d0): the radar draws it 200, 200, 200 and
    /// tints its portrait (play_hud.rs).
    pub fn selected(&self, c: usize) -> bool {
        self.flags.get(c).copied().unwrap_or(false)
    }

    /// The share of their alpha the crosshair and the item box keep (FUN_000c17c0, from the
    /// reticle's FUN_000c6280 and the item box's 0xabc60): none while the cursor is up, 1 less the
    /// fade's share while the menu is open, else all.
    pub fn hud_share(&self) -> f32 {
        if self.cursor {
            0.0
        } else if self.open {
            1.0 - self.fade / FADE
        } else {
            1.0
        }
    }

    /// How far the radar has slid toward the menu's middle, 0 to 1: FUN_000c2be0's 1 - (1 - fade
    /// / 0.3)^3 (a cubic ease: take01's radar most of the way there by the fade's first third).
    pub fn radar_slide(&self) -> f32 {
        let t = 1.0 - self.fade / FADE;
        1.0 - t * t * t
    }

    /// The portrait the chevrons flash toward after a press (FUN_000c4dd0) and the flash's level
    /// (1 at the press, falling FLASH_FALL a second), while it lasts.
    pub fn flash(&self) -> Option<(usize, f32)> {
        self.flash.filter(|f| f.1 > 0.0)
    }

    /// Whether the menu or the cursor is up (the HUD draws its highlight meanwhile).
    pub fn up(&self) -> bool {
        self.open || self.cursor
    }

    /// The flags as the takes' watch logs show them (`sel` 00101: slots 0-4 left to right, the
    /// game's slot n by its type: Tex 1, Flint 2, Brutus 3, Hawk 4).
    fn sel_string(&self) -> String {
        let game_slot = |n: usize| match n { 1 => self.flags[3], 2 => self.flags[1], 3 => self.flags[0], 4 => self.flags[2], _ => false };
        (0..5).map(|n| if game_slot(n) { '1' } else { '0' }).collect()
    }
}

/// What a step of the list or the widget asks of the world: a sound (the HUD's, 2D), a flash of
/// the chevrons at a portrait, a hand-over to a slot's member.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Act {
    Sound(u32),
    Flash(usize),
    Switch(usize),
}

/// What FUN_000c8320 does with a slot (the game options' record +0x160..+0x16c, all 0 but the
/// closed menu's tap, +0x160 = 1: FUN_00068800 zeroes the other three): toggle its flag or
/// switch to it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum SlotAct {
    Toggle,
    Switch,
}

impl SquadMenu {
    /// One step of the d-pad handler (0xc7e20) with the direction held this frame (`dir`, a slot),
    /// the HUD's `dt`, whether the d-pad is read at all (`live`: the controls on, no hand-over
    /// under way, no list or menu over the game; else nothing is held and a release isn't a
    /// tap), whether the controlled character is dead (then a press does nothing: the demo leaves
    /// the hand-over to the death camera, play_deathcam.rs; the game switches at once, 0xc809c)
    /// and whether a slot's member may be selected (`may`, FUN_000c8470).
    fn dpad(&mut self, dir: Option<usize>, dt: f32, live: bool, dead: bool, may: &dyn Fn(usize) -> bool, acts: &mut Vec<Act>) {
        let mut dir = dir.filter(|_| live);
        let mut tap_ok = live;
        if !live {
            (self.hold, self.held) = (0.0, None);
        }
        // a direction still locked out is no press (0xc802c-0xc8048)
        if dir.is_some_and(|d| self.lockout[d] > 0.0) {
            dir = None;
            tap_ok = false;
        }
        for l in &mut self.lockout {
            *l = (*l - dt).max(0.0);
        }
        if let Some((_, level)) = self.flash.as_mut() {
            *level = (*level - FLASH_FALL * dt).max(0.0);
        }
        if dead {
            return;
        }
        // a tap toggles while the menu is open (fading out too: +0xbc bit 0), switches while it's
        // closed; a hold toggles either way
        let tap = if self.open { SlotAct::Toggle } else { SlotAct::Switch };
        match dir {
            None => match self.held.take() {
                Some(h) => {
                    self.consumed = true;
                    self.hold = 0.0;
                    if tap_ok {
                        self.act(h, tap, may, acts);
                    }
                }
                None => self.consumed = false,
            },
            Some(d) if self.held == Some(d) => {
                self.hold += dt;
                if self.hold >= HOLD {
                    (self.hold, self.held, self.consumed) = (0.0, None, true);
                    self.act(d, SlotAct::Toggle, may, acts);
                }
            }
            // another direction while one was held: the first's tap (0xc8198)
            Some(_) if self.held.is_some() => {
                let h = self.held.take().unwrap_or(0);
                (self.consumed, self.hold) = (true, 0.0);
                if tap_ok {
                    self.act(h, tap, may, acts);
                }
            }
            Some(d) => {
                if !self.consumed {
                    (self.held, self.hold) = (Some(d), dt);
                    if may(d) {
                        acts.push(Act::Sound(S_PRESS));
                        acts.push(Act::Flash(d));
                        // FUN_000c4dd0: the level to 1.0 from none, back up to 0.8 when it has
                        // fallen under 0.6
                        let level = self.flash.filter(|f| f.0 == d).map_or(0.0, |f| f.1);
                        self.flash = Some((d, if level <= 0.0 { 1.0 } else if level < 0.6 { 0.8 } else { level }));
                    }
                }
            }
        }
        // a flagged member that may no longer be selected drops out (0xc828b-0xc82f0)
        for c in 0..SQUAD {
            if self.flags[c] && !may(c) {
                self.flags[c] = false;
                acts.push(Act::Sound(S_OFF));
            }
        }
    }

    /// FUN_000c8320: a slot's action, unless it's locked out; it locks the slot out for LOCKOUT.
    /// A toggle (FUN_000c8640) of a member that may be selected sets or clears its flag with
    /// h_fac5d6b4 or h_0d93fbd9; a switch to one (FUN_000c8410 -> FUN_000c88c0) starts the
    /// hand-over and clears every flag.
    fn act(&mut self, c: usize, what: SlotAct, may: &dyn Fn(usize) -> bool, acts: &mut Vec<Act>) {
        if self.lockout[c] > 0.0 {
            return;
        }
        self.lockout[c] = LOCKOUT;
        if !may(c) {
            return;
        }
        match what {
            SlotAct::Toggle => {
                self.flags[c] = !self.flags[c];
                acts.push(Act::Sound(if self.flags[c] { S_ON } else { S_OFF }));
            }
            SlotAct::Switch => {
                acts.push(Act::Switch(c));
                self.flags = [false; SQUAD];
            }
        }
    }

    /// The widget opens (FUN_000c10d0): the game paused (single player: the mode's player count,
    /// 0x8cd20; the demo has no split screen; `squad_menu` holds the clock while `paused_game`),
    /// open and fading in, the open sound (FUN_000c0940).
    fn open_menu(&mut self, acts: &mut Vec<Act>) {
        self.paused_game = true;
        (self.open, self.fading_in, self.closing) = (true, true, false);
        acts.push(Act::Sound(S_OPEN));
    }

    /// The widget starts to close (FUN_000c1120 and FUN_000c0d90's tail): fading out, with
    /// h_15262e1b unless the cursor is up.
    fn close_menu(&mut self, acts: &mut Vec<Act>) {
        if self.open {
            (self.fading_in, self.closing) = (false, true);
            if !self.cursor {
                acts.push(Act::Sound(S_CLOSE));
            }
        }
    }

    /// The fade's end (FUN_000c1150): the game runs again, the selection cleared unless the cursor
    /// is up, the last order forgotten, the widget hidden.
    fn hidden(&mut self) {
        self.paused_game = false;
        (self.open, self.closing) = (false, false);
        if !self.cursor {
            self.flags = [false; SQUAD];
        }
        self.last = None;
    }

    /// The fade (FUN_000c0cf0) by `dt`: none while closed, up to FADE opening, down to 0 closing
    /// (then `hidden`).
    fn run_fade(&mut self, dt: f32) {
        if !self.open {
            self.fade = 0.0;
        } else if self.fading_in {
            self.fade += dt;
            if self.fade >= FADE {
                (self.fade, self.fading_in) = (FADE, false);
            }
        } else if self.closing {
            self.fade -= dt;
            if self.fade <= 0.0 {
                self.fade = 0.0;
                self.hidden();
            }
        }
    }

    /// The cursor goes (FUN_000bd930): hidden, the selection cleared, h_15262e1b.
    fn hide_cursor(&mut self, acts: &mut Vec<Act>) {
        if self.cursor {
            self.cursor = false;
            self.flags = [false; SQUAD];
            (self.cursor_point, self.on_player) = (None, false);
            acts.push(Act::Sound(S_CLOSE));
        }
    }
}

/// The menu's buttons this frame (`menu_input`): the d-pad directions held and pressed this frame
/// (by slot), the first order button pressed (an entry index, in the game's order: table
/// 0x3c9008), BACK, and the cursor's A and B.
#[derive(Resource, Default, Clone, Copy, Debug)]
struct MenuInput {
    dpad: [bool; SQUAD],
    dpad_down: [bool; SQUAD],
    order: Option<usize>,
    back: bool,
    accept: bool,
    cancel: bool,
}

/// The order buttons in the order FUN_000c0d90 reads them (0x3c9008: VB 38 B, 37 A, 40 Y, 39 X,
/// 63 Black, 9 White -> FUN_000bffb0's entries 1, 2, 0, 3, 4, 5), with the demo's keys and the
/// pad's buttons (White and Black on the shoulders, as xemu has them: play_hints.rs).
const ORDER_BUTTONS: [(usize, KeyCode, GamepadButton); 6] = [(FIRE, KeyCode::KeyL, GamepadButton::East), (COVER, KeyCode::KeyK, GamepadButton::South),
                                                            (STAND, KeyCode::KeyI, GamepadButton::North), (MOVE, KeyCode::KeyJ, GamepadButton::West),
                                                            (MEDKIT, KeyCode::KeyB, GamepadButton::RightTrigger),
                                                            (TOGGLE, KeyCode::KeyV, GamepadButton::LeftTrigger)];

/// A BF_TEST_COMMAND entry: when (the hooks' clock), what, and for how long a direction is held.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TestPress {
    at: f32,
    what: TestWhat,
    hold: f32,
}

/// What a BF_TEST_COMMAND entry presses.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TestWhat {
    Member(usize),
    Order(usize),
    Back,
    Accept,
    Cancel,
}

/// A BF_TEST_COMMAND direction's hold when none is given (s): a tap.
const TEST_TAP: f32 = 0.1;

/// BF_TEST_COMMAND's entries (see the module notes); a bad one is logged and skipped.
fn parse_tests(v: &str) -> Vec<TestPress> {
    let mut out = vec![];
    for e in v.split(';').map(str::trim).filter(|e| !e.is_empty()) {
        let parts: Vec<&str> = e.split(':').map(str::trim).collect();
        let at = parts.first().and_then(|t| t.parse::<f32>().ok());
        let what = parts.get(1).and_then(|w| test_what(w));
        let hold = parts.get(2).and_then(|h| h.parse::<f32>().ok()).unwrap_or(TEST_TAP);
        match (at, what) {
            (Some(at), Some(what)) => out.push(TestPress { at, what, hold }),
            _ => println!("BF_TEST_COMMAND: \"{e}\" isn't <s>:<member|order|back|accept|cancel>[:<hold s>]: skipped"),
        }
    }
    out
}

/// A BF_TEST_COMMAND entry's `<what>`.
fn test_what(w: &str) -> Option<TestWhat> {
    let w = w.to_ascii_lowercase();
    Some(match w.as_str() {
        "stand" | "y" => TestWhat::Order(STAND),
        "fire" | "b" => TestWhat::Order(FIRE),
        "cover" | "a" => TestWhat::Order(COVER),
        "move" | "x" => TestWhat::Order(MOVE),
        "medkit" | "black" => TestWhat::Order(MEDKIT),
        "toggle" | "white" => TestWhat::Order(TOGGLE),
        "back" => TestWhat::Back,
        "accept" => TestWhat::Accept,
        "cancel" => TestWhat::Cancel,
        m => TestWhat::Member(testsquad::slot_named(m, usize::MAX).filter(|&s| s < SQUAD)?),
    })
}

/// The HUD's clock step (s): the menu and the cursor run through the menu's own pause (take01:
/// the fade while the session's clock is still): the real frame time, the capture's fixed step
/// while capturing (none in its warm-up), none while another menu stops the game's clock (the
/// HUD isn't updated under it).
fn hud_dt(time: &Time) -> f32 {
    if paused_by_other(PAUSE_SQUAD) {
        return 0.0;
    }
    if std::env::var("BF_CAPTURE").is_ok() {
        if WARMUP.load(std::sync::atomic::Ordering::Relaxed) > 0 { 0.0 } else { capture_step() }
    } else {
        time.delta_secs().min(0.1)
    }
}

/// The hooks' clock: the sim time, which stops under the menu's pause, plus the time the pause
/// held it.
fn hook_time(p: &Player, m: &SquadMenu) -> f32 {
    p.sim_time + m.paused_time
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SquadMenu>()
        .init_resource::<MenuInput>()
        .add_systems(OnEnter(AppState::Playing), (
            (|mut commands: Commands| {
                commands.insert_resource(SquadMenu::default());
                commands.insert_resource(MenuInput::default());
            }).after(setup),
            spawn_menu.after(snapshot_entities).in_set(super::hud::HudSetup),
            spawn_marker.after(setup),
        ))
        .add_systems(OnExit(AppState::Playing), |mut m: ResMut<SquadMenu>| {
            m.open = false;
            m.paused_game = false;
            set_paused(PAUSE_SQUAD, false);
        })
        // (the menu takes the frame's presses before the objectives menu, the test tools and the
        // keys' other readers see them)
        .add_systems(PreUpdate, menu_input.after(bevy::input::InputSystem).before(super::objectives::ObjectivesControls)
            .run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            squad_menu.after(read_input).before(squad_control),
            find_point.after(super::reticle::update_reticle).before(update_hud),
            draw.after(find_point),
        ).run_if(in_state(AppState::Playing)));
}

/// The frame's menu buttons (`MenuInput`) from the keys, a pad and the hooks; and, while the menu
/// is open, the frame's presses taken from everything else (as the objectives menu does: Escape,
/// Backspace or I closing it don't free the mouse, leave the map or open the objectives); while
/// the cursor is up its own buttons (E, Escape, the two mouse buttons) taken and let go, so they
/// neither use, free the mouse, fire nor aim.
fn menu_input(mut keys: ResMut<ButtonInput<KeyCode>>, mut mouse: ResMut<ButtonInput<MouseButton>>, pads: Query<&Gamepad>, player: Res<Player>,
              mut menu: ResMut<SquadMenu>, mut input: ResMut<MenuInput>) {
    let m = &mut *menu;
    let mut i = MenuInput::default();
    let pad_held = |b: GamepadButton| pads.iter().any(|p| p.pressed(b));
    let pad = |b: GamepadButton| pads.iter().any(|p| p.just_pressed(b));
    // (not while a cutscene holds the world: the handler's world+0xc50 bit 4, play_script.rs)
    if !script::in_cutscene() {
        for (c, key, button) in DPAD {
            i.dpad[c] = keys.pressed(key) || pad_held(button);
            i.dpad_down[c] = keys.just_pressed(key) || pad(button);
        }
        i.order = ORDER_BUTTONS.iter().find(|(_, k, b)| keys.just_pressed(*k) || pad(*b)).map(|o| o.0);
        i.back = keys.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) || pad(GamepadButton::Select);
        i.accept = keys.just_pressed(KeyCode::KeyE) || mouse.just_pressed(MouseButton::Left) || pad(GamepadButton::South);
        i.cancel = keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) || pad(GamepadButton::East);
    }
    // the hooks, on their clock (a direction held its time, from at least the first frame; a
    // button the frame its time comes)
    let now = hook_time(&player, m);
    let tests = m.tests.get_or_insert_with(|| std::env::var("BF_TEST_COMMAND").map_or(vec![], |v| parse_tests(&v).into_iter().map(|t| (t, false)).collect()));
    for (t, started) in tests.iter_mut() {
        if now < t.at {
            continue;
        }
        let first = !*started;
        *started = true;
        match t.what {
            TestWhat::Member(c) if first || now < t.at + t.hold => {
                i.dpad[c] = true;
                i.dpad_down[c] |= first;
            }
            TestWhat::Order(o) if first => i.order = Some(o),
            TestWhat::Back if first => i.back = true,
            TestWhat::Accept if first => i.accept = true,
            TestWhat::Cancel if first => i.cancel = true,
            _ => {}
        }
    }
    *input = i;
    if m.open {
        let pressed: Vec<KeyCode> = keys.get_just_pressed().copied().collect();
        for k in pressed {
            keys.clear_just_pressed(k);
        }
        let clicked: Vec<MouseButton> = mouse.get_just_pressed().copied().collect();
        for b in clicked {
            mouse.clear_just_pressed(b);
        }
    } else if m.cursor {
        for k in [KeyCode::KeyE, KeyCode::Escape] {
            keys.clear_just_pressed(k);
            keys.release(k);
        }
        for b in [MouseButton::Left, MouseButton::Right] {
            mouse.clear_just_pressed(b);
            mouse.release(b);
        }
    }
}

/// The d-pad direction the handler reads this frame (the first in DPAD's order): one held, or,
/// while the menu is open, only one pressed this frame. The open menu locks the player's input
/// object (FUN_000c11f0 -> FUN_001e7820, let go by FUN_000c1150 at the fade-out's end), and what
/// the d-pad then reads comes as presses: take01's LEFT held 1.2 s read as no
/// direction from the frame the menu opened (dir 0 from 2.34 s to the release at 3.2, Hawk kept),
/// where a held direction would have pressed again once its lockout ran out; take03's 100 ms
/// presses with it open toggled at once. (Which binding mode does it, the second input object's
/// press-only d-pad, is inferred from the takes.)
fn read_dir(open: bool, input: &MenuInput) -> Option<usize> {
    let read = if open { &input.dpad_down } else { &input.dpad };
    DPAD.iter().map(|d| d.0).find(|&c| read[c])
}

/// The squad's members by slot: the controlled character or one of the squad, in the world
/// (play_hud.rs `slot_member`'s rule).
fn member<'a>(player: &'a Player, squad: &'a Squad, c: usize) -> Option<&'a Player> {
    std::iter::once(player).chain(squad.0.iter()).find(|u| u.character == c && u.in_squad() && !u.absent)
}

/// FUN_000c8470: whether slot `c`'s member may be selected (or taken): there, loaded and alive
/// (char+0x70 bit 0); the controlled character only while the cursor isn't up; none held by
/// another player's controller (the demo has one player).
fn may_select(player: &Player, squad: &Squad, cursor: bool, c: usize) -> bool {
    member(player, squad, c).is_some_and(|u| u.loaded.is_some() && !u.dead && !u.stand_in && !(u.character == player.character && cursor))
}

/// A HUD sound (FUN_000bb220): 2D, at its library entry's volume.
fn hud_sound(p: &mut Player, game: &Game, id: u32) {
    let volume = game.sounds.params(id).map_or(1.0, |s| s.volume);
    p.sound_queue.push((id, volume));
}

/// One frame of the squad list, the menu and the cursor's input, on the HUD's clock (after the
/// controls are read, before the hand-over's countdown: a tap's hand-over starts this frame).
/// Nothing while another menu holds the game's clock (the HUD isn't updated under one).
fn squad_menu(time: Res<Time>, input: Res<MenuInput>, mut menu: ResMut<SquadMenu>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
              game: Res<GameData>) {
    if paused_by_other(PAUSE_SQUAD) {
        return;
    }
    let dt = hud_dt(&time);
    let m = &mut *menu;
    if m.paused_game {
        m.paused_time += dt;
    }
    let now = hook_time(&player, m);
    let log = m.log;
    let mut acts = vec![];
    let before = (m.sel_string(), m.open, m.cursor);
    // no character (the editor's stand-in, none spawned yet): no selection (0xc82fc)
    if player.loaded.is_none() || player.stand_in {
        m.flags = [false; SQUAD];
    } else {
        // the d-pad's read (0xc7f06-0xc7f5a): no hand-over under way (the select widget's +0x68
        // is its +0x70), no script holding the character, and a squad with more members than
        // humans (0xc7f1b: the squad's count against the mode's vtable +0x7c): an AI member in
        // the world (none in a deathmatch, nor on the test map's squad of one). (Another menu over
        // the game, the item list's among them, stops the whole step above.)
        let ai_member = squad.0.iter().any(|u| u.in_squad() && !u.absent);
        let live = player.select.is_none() && !script::blocks_input(&player) && ai_member;
        let dir = read_dir(m.open, &input);
        let cursor = m.cursor;
        let may = |c: usize| may_select(&player, &squad, cursor, c);
        m.dpad(dir, dt, live, player.dead, &may, &mut acts);
    }
    // the widget (FUN_000c0fb0): its character gone or dead, it closes and the selection empties
    let alive = player.loaded.is_some() && !player.dead && !player.stand_in;
    if !alive {
        m.close_menu(&mut acts);
        m.flags = [false; SQUAD];
    }
    let selected = m.flags.iter().any(|&f| f);
    if selected {
        if !m.open && !m.cursor {
            m.open_menu(&mut acts);
        }
        if m.open && !m.closing && !m.cursor {
            menu_orders(m, &input, &mut player, &mut squad, now, &mut acts);
        }
    } else if m.open && !m.closing {
        m.close_menu(&mut acts);
        acts.push(Act::Sound(S_EMPTIED));
    }
    m.run_fade(dt);
    // the game's clock held while the menu has it paused (play.rs PAUSED)
    set_paused(PAUSE_SQUAD, m.paused_game);
    // the cursor's input (0xbe130)
    if m.cursor {
        if input.accept {
            // A: Move To with the point, or Cover Me with no point (on the character: +0xcc)
            let (order, point) = if m.on_player { (COVER, None) } else { (MOVE, Some(m.cursor_at)) };
            if log {
                println!("t={now:.2} squad menu: accept on {}: {}", if m.on_player { "the controlled character (no point)".to_string() }
                         else { format!("({:.1}, {:.1}, {:.1})", m.cursor_at.x, m.cursor_at.y, m.cursor_at.z) }, ENTRY_NAMES[order]);
            }
            give(m, order, point, &mut player, &mut squad, now);
            acts.push(Act::Sound(S_ORDER));
            m.hide_cursor(&mut acts);
        } else if input.cancel {
            if log {
                println!("t={now:.2} squad menu: the cursor cancelled");
            }
            m.hide_cursor(&mut acts);
        } else if !m.flags.iter().any(|&f| f) {
            m.hide_cursor(&mut acts);
        }
    }
    // what the steps asked
    for a in acts {
        match a {
            Act::Sound(id) => hud_sound(&mut player, &game.0, id),
            Act::Flash(_) => {}
            Act::Switch(c) => {
                if c != player.character {
                    player.select = Some((c, SELECT_TIME));
                }
                if log {
                    println!("t={now:.2} squad menu: tap: switch to slot {c} ({})", CHARACTERS[c]);
                }
            }
        }
    }
    if log {
        let after = (m.sel_string(), m.open, m.cursor);
        if after != before {
            println!("t={now:.2} squad menu: sel {} open {} cursor {} fade {:.3}", after.0, after.1, after.2, m.fade);
        }
    }
}

/// The open menu's input (FUN_000c0d90): BACK clears the selection; else the first order button
/// pressed: Move To, Medkit and Toggle Ability first drop the controlled character from the
/// selection; with anyone left Move To brings up the cursor, the others are given
/// (FUN_000c1280), with h_11d6f86a when anyone took it; then the menu closes.
fn menu_orders(m: &mut SquadMenu, input: &MenuInput, player: &mut Player, squad: &mut Squad, now: f32, acts: &mut Vec<Act>) {
    if input.back {
        if m.log {
            println!("t={now:.2} squad menu: BACK");
        }
        m.flags = [false; SQUAD];
        return;
    }
    let Some(order) = input.order else { return };
    let me = player.character;
    if order >= MOVE && me < SQUAD {
        m.flags[me] = false;
    }
    if m.log {
        println!("t={now:.2} squad menu: {} ({} selected)", ENTRY_NAMES[order], m.flags.iter().filter(|&&f| f).count());
    }
    if m.flags.iter().any(|&f| f) {
        let taken = if order == MOVE {
            m.cursor = true;
            (m.cursor_point, m.on_player) = (None, false);
            true
        } else {
            give(m, order, None, player, squad, now)
        };
        if taken {
            m.last = Some(order);
            acts.push(Act::Sound(S_ORDER));
        }
    }
    m.close_menu(acts);
}

/// FUN_000c1280: the order to each selected member (by slot: every squad member has a brain).
/// Stand Ground, Fire At Will, Cover Me and Move To set its command mode (play_orders.rs
/// `order`: GUARD, WINGMAN, FOLLOW, MOVE_OUT with the point); the leader they make the ordering
/// player (FUN_0005bda0) is the demo's already. Toggle Ability asks its skill to toggle
/// (GOAL_TOGGLE_COMBAT_SKILL 127 at priority 2: play_skills.rs `Skill::request`, as #35 take18's
/// Hawk). Medkit pushes GOAL_HEAL (20) in the game, whose behaviour isn't traced (#329): here
/// it's only logged. The selection is cleared after. True when any member took it.
fn give(m: &mut SquadMenu, order: usize, point: Option<Vec3>, player: &mut Player, squad: &mut Squad, now: f32) -> bool {
    let mut any = false;
    let flags = m.flags;
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if !u.in_squad() || u.absent || u.character >= SQUAD || !flags[u.character] {
            continue;
        }
        any = true;
        match order {
            STAND => { orders::order(u, Mode::Guard, None, now); }
            FIRE => { orders::order(u, Mode::Wingman, None, now); }
            COVER => { orders::order(u, Mode::Follow, None, now); }
            MOVE => { orders::order(u, Mode::MoveOut, point, now); }
            TOGGLE => {
                if u.skill.data.is_some() && !u.dead {
                    u.skill.request = true;
                }
            }
            _ => {}
        }
        if m.log {
            match order {
                MEDKIT => println!("t={now:.2} squad menu: Medkit to {}: GOAL_HEAL (20) pushed in the game; its heal isn't done (#329)", unit_name(u)),
                TOGGLE => println!("t={now:.2} squad menu: Toggle Ability to {}: its skill asked to toggle", unit_name(u)),
                _ => {}
            }
        }
    }
    m.flags = [false; SQUAD];
    any
}

/// The cursor's update (0xbdfb0), after the reticle's centre ray (0xc5720, play_reticle.rs): the
/// world point nearest the ray's point (FUN_0004d630 over the level's points) that passes the
/// filter (0xbdef0: +0x24 bit 14 set and bit 15 clear, +0x28 bit 2, its projection inside the 640
/// x 480); with none, the controlled character's place (and Cover Me for A, +0xcc). Also the
/// marker's turn. BF_TEST_CURSOR_AT=<x>,<z> takes the floor there as the ray's point.
fn find_point(time: Res<Time>, mut menu: ResMut<SquadMenu>, player: Res<Player>, reticle: Res<super::reticle::Reticle>, map: Res<MapLevel>) {
    let m = &mut *menu;
    if !m.cursor {
        return;
    }
    m.marker_turn = (m.marker_turn + MARKER_TURN * frame_dt(&time)).rem_euclid(std::f32::consts::TAU);
    let ray = std::env::var("BF_TEST_CURSOR_AT").ok().and_then(|v| {
        let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        (n.len() >= 2).then(|| Vec3::new(n[0], floor_y(n[0], n[1], player.position.y + 50.0, world::Body::Character), n[1]))
    }).unwrap_or(reticle.point);
    let points = map.0.as_ref().map_or(&[][..], |l| &l.points[..]);
    let view = reticle.view;
    let passes = |p: &WorldPoint| {
        p.flags & WorldPoint::BIT_14 != 0 && p.flags & WorldPoint::BIT_15 == 0 && POINT_USABLE
            && view.and_then(|v| v.project(p.at)).is_some_and(|(s, _)| (0.0..=SCREEN.x).contains(&s.x) && (0.0..=SCREEN.y).contains(&s.y))
    };
    let best = points.iter().enumerate().filter(|(_, p)| passes(p)).min_by(|a, b| a.1.at.distance_squared(ray).total_cmp(&b.1.at.distance_squared(ray)));
    let was = m.cursor_point;
    match best {
        Some((i, p)) => (m.cursor_at, m.cursor_point, m.on_player) = (p.at, Some(i), false),
        None => (m.cursor_at, m.cursor_point, m.on_player) = (player.position, None, true),
    }
    if m.log && m.cursor_point != was {
        match best {
            Some((i, p)) => println!("t={:.2} squad menu: cursor on point {i} h_{:08x} (type h_{:08x}, flags {:08x}) at ({:.1}, {:.1}, {:.1}), {:.1} m from the ray's point ({:.1}, {:.1}, {:.1})",
                                     hook_time(&player, m), p.name, p.kind, p.flags, p.at.x, p.at.y, p.at.z, p.at.distance(ray), ray.x, ray.y, ray.z),
            None => println!("t={:.2} squad menu: cursor on no point: on the controlled character", hook_time(&player, m)),
        }
    }
}

// ---- drawing ----

/// The menu's and the cursor's nodes (`spawn_menu`): the root (over the HUD, MENU_Z), the
/// backdrop, the menu's texts and entries, the cursor's rows; the record texts and glyph cells
/// with the alpha each takes.
#[derive(Resource)]
struct MenuNodes {
    root: Entity,
    backdrop: Entity,
    menu: Entity,
    cursor: Entity,
    /// the record texts that fade as the menu's texts do (title, rows, labels by entry)
    texts: Vec<(Entity, Fades)>,
    /// the glyph cells and icons: their entity and how they fade
    images: Vec<(Entity, Fades)>,
}

/// How a node fades (FUN_000c09a0 for the menu's title and rows, the entries' update 0xc19d0,
/// 0xbdce0 for the cursor's rows).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Fades {
    /// the title and the rows: min(2k, 1), k = (fade / 0.3)^2
    Text,
    /// entry n's icon and label: k, whole while it's the order just given
    Entry(usize),
    /// the cursor's rows: 1 - fade / 0.3
    Cursor,
}

/// An icon font glyph's cell, by its letter (ICON_GLYPHS_12).
fn icon_glyph(c: char) -> Option<&'static (char, f32, f32, f32, f32, i8, u8, u8)> {
    ICON_GLYPHS_12.iter().find(|g| g.0 == c)
}

/// A string of icon glyphs' width (the pen's travel: offset and advance each, FUN_000ccae0).
fn glyphs_width(glyphs: &[char]) -> f32 {
    glyphs.iter().filter_map(|&c| icon_glyph(c)).map(|g| g.5 as f32 + g.7 as f32).sum()
}

/// A glyph row's width (the row's +0xe, which the menu centres it by): its glyphs, the gap and
/// its words; its words alone without glyphs.
fn row_width(glyphs: &[char], words: &str, record: &Record) -> f32 {
    let w = record.measure(words);
    if glyphs.is_empty() { w } else { glyphs_width(glyphs) + GLYPH_GAP + w }
}

/// A glyph row's nodes under `parent` with its line's top left at `at` (FUN_000ca000 with no
/// lead word): the glyphs GLYPH_DROP lower, from the line's start, the words GLYPH_GAP after them;
/// pushed onto `texts` and `images` with `fades`.
#[allow(clippy::too_many_arguments)]
fn spawn_row(commands: &mut Commands, parent: Entity, at: Vec2, glyphs: &[char], words: &str, icons: &Option<Handle<Image>>, fades: Fades,
             texts: &mut Vec<(Entity, Fades)>, images: &mut Vec<(Entity, Fades)>) {
    let mut pen = at.x;
    if let Some(icons) = icons {
        for &c in glyphs {
            let Some(&(_, x0, y0, x1, y1, offset, width, advance)) = icon_glyph(c) else { continue };
            let x = pen + offset as f32;
            let e = commands.spawn((ImageNode { rect: Some(Rect::new(x0, y0, x1, y1)), ..ImageNode::new(icons.clone()) },
                                    super::hud::at(x, at.y + GLYPH_DROP, width as f32 + 1.0, ICON_LINE_12 + 1.0), ChildOf(parent))).id();
            images.push((e, fades));
            pen = x + advance as f32;
        }
    }
    let x = if glyphs.is_empty() { at.x } else { at.x + glyphs_width(glyphs) + GLYPH_GAP };
    let blue = Color::srgb_u8(BLUE[0], BLUE[1], BLUE[2]);
    let e = commands.spawn((record_text(words, x, at.y, &BODY12, vec![blue], Some(Color::BLACK), Across::Left), ChildOf(parent))).id();
    texts.push((e, fades));
}

/// The menu's and the cursor's nodes as a map starts (with the HUD; none under BF_NO_HUD), hidden:
/// the backdrop over the whole window, the rest on the game's 4:3 screen, placed as FUN_000c0ae0
/// and 0xbdce0 place them on a full screen (see the constants).
fn spawn_menu(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, mut cache: ResMut<HudImages>) {
    if std::env::var("BF_NO_HUD").is_ok() {
        return;
    }
    let text = |id: u32, fallback: &str| game.0.strings.get(&id).cloned().unwrap_or_else(|| fallback.to_string());
    let (title, dpad_row, give_row, close_row) = (text(S_TITLE, "Squad Command"), text(S_DPAD, "D-Pad Select Char"), text(S_GIVE_ORDER, "Give Order"),
                                                  text(S_CLOSE_MENU, "Close Menu"));
    let (look, accept, cancel) = (text(S_LOOK, "Look to select position"), text(S_ACCEPT, "accept"), text(S_CANCEL, "cancel"));
    let labels: Vec<String> = ENTRY_LABELS.iter().zip(ENTRY_NAMES).map(|(&id, name)| text(id, name)).collect();
    let icons = game.0.texture_rgba(ICON_ATLAS_12).map(|(w, h, px)| images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, px,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())));
    let entry_icons: Vec<Option<Handle<Image>>> = ENTRY_ICONS.iter().map(|&id| texture(&mut game.0, &mut images, &mut cache, id)).collect();
    let full = || Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0),
                         height: Val::Percent(100.0), ..default() };
    let root = commands.spawn((HudRoot, Name::new("squad command menu"), GlobalZIndex(MENU_Z), Visibility::Hidden,
                               Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                      justify_content: JustifyContent::Center, ..default() })).id();
    let backdrop = commands.spawn((full(), BackgroundColor(Color::NONE), Visibility::Hidden, ChildOf(root))).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(root))).id();
    let menu = commands.spawn((full(), Visibility::Hidden, ChildOf(screen))).id();
    let cursor = commands.spawn((full(), Visibility::Hidden, ChildOf(screen))).id();
    let blue = Color::srgb_u8(BLUE[0], BLUE[1], BLUE[2]);
    let (mut texts, mut imgs) = (vec![], vec![]);
    // the title, centred on the screen's middle, 30 above Stand Ground
    let e = commands.spawn((record_text(&title, WIDGET_AT.x - 20.0, WIDGET_AT.y + ENTRY_AT[STAND].1 - TITLE_UP, &BODY16, vec![blue], Some(Color::BLACK),
                                        Across::Centre), ChildOf(menu))).id();
    texts.push((e, Fades::Text));
    // the entries: the four in the face buttons' places, 64 x 64 with their labels under them;
    // Medkit and Toggle Ability 32 x 32 with theirs beside them
    for (n, &(x, y)) in ENTRY_AT.iter().enumerate() {
        let small = n >= MEDKIT;
        let size = if small { SMALL_ENTRY } else { ENTRY_SIZE };
        let at = WIDGET_AT + Vec2::new(x, y);
        let icon = commands.spawn((flipped(entry_icons[n].clone().unwrap_or_default()), super::hud::at(at.x, at.y, size, size), ChildOf(menu))).id();
        imgs.push((icon, Fades::Entry(n)));
        let (lx, ly, across) = if small {
            (at.x + (size / 4.0).trunc(), at.y + ((size - BODY10.height()) / 2.0).trunc(), Across::Left)
        } else {
            (at.x + size / 2.0, at.y + size, Across::Centre)
        };
        let e = commands.spawn((record_text(&labels[n], lx, ly, &BODY10, vec![blue], Some(Color::BLACK), across), ChildOf(menu))).id();
        texts.push((e, Fades::Entry(n)));
    }
    // the rows: "Give Order" centred on Cover Me 38 under its icon, "D-Pad Select Char" as far the
    // other side of the screen's middle, "Close Menu" 26 lower in the middle
    let cover = WIDGET_AT + Vec2::new(ENTRY_AT[COVER].0, ENTRY_AT[COVER].1);
    let rows_y = cover.y + ENTRY_SIZE + ROWS_DOWN;
    let side = ENTRY_AT[COVER].0 + ENTRY_SIZE / 2.0;
    let centre_left = |w: f32, centre: f32| centre - (w / 2.0).trunc();
    let give_w = row_width(&GIVE_GLYPHS, &give_row, &BODY12);
    spawn_row(&mut commands, menu, Vec2::new(centre_left(give_w, WIDGET_AT.x + side), rows_y), &GIVE_GLYPHS, &give_row, &icons, Fades::Text, &mut texts, &mut imgs);
    let dpad_w = row_width(&[], &dpad_row, &BODY12);
    spawn_row(&mut commands, menu, Vec2::new(centre_left(dpad_w, WIDGET_AT.x - 20.0 - side), rows_y), &[], &dpad_row, &icons, Fades::Text, &mut texts, &mut imgs);
    let close_w = row_width(&CLOSE_GLYPHS, &close_row, &BODY12);
    spawn_row(&mut commands, menu, Vec2::new(centre_left(close_w, WIDGET_AT.x - 20.0), rows_y + CLOSE_DOWN), &CLOSE_GLYPHS, &close_row, &icons, Fades::Text,
              &mut texts, &mut imgs);
    // the cursor's rows
    spawn_row(&mut commands, cursor, CURSOR_ROWS_AT - Vec2::Y * CURSOR_ROW_STEP, &[], &look, &icons, Fades::Cursor, &mut texts, &mut imgs);
    spawn_row(&mut commands, cursor, CURSOR_ROWS_AT, &['A'], &accept, &icons, Fades::Cursor, &mut texts, &mut imgs);
    let accept_w = row_width(&['A'], &accept, &BODY12);
    spawn_row(&mut commands, cursor, CURSOR_ROWS_AT + Vec2::X * (accept_w + CURSOR_ROW_GAP), &['B'], &cancel, &icons, Fades::Cursor, &mut texts, &mut imgs);
    commands.insert_resource(MenuNodes { root, backdrop, menu, cursor, texts, images: imgs });
}

/// The cursor's marker (0xbe5e0): its type's model, the level's objects' way (level_scene.rs
/// `object_parts`, lit by the map's lamps), hidden under a root of its own until the cursor is up.
#[derive(Resource)]
struct Marker(Entity);

/// The marker's model as a map starts.
#[derive(SystemParam)]
struct MarkerAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<LevelMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    buffers: ResMut<'w, Assets<bevy::render::storage::ShaderStorageBuffer>>,
}

/// The marker's root and its parts (see `Marker`); with BF_SQUAD_MENU_LOG, the level's world
/// points and how many the cursor's flags take.
fn spawn_marker(mut commands: Commands, mut game: ResMut<GameData>, map: Res<MapLevel>, mut assets: MarkerAssets) {
    if std::env::var("BF_SQUAD_MENU_LOG").is_ok() {
        let points = map.0.as_ref().map_or(&[][..], |l| &l.points[..]);
        let usable = points.iter().filter(|p| p.flags & WorldPoint::BIT_14 != 0 && p.flags & WorldPoint::BIT_15 == 0).count();
        let plain = points.iter().filter(|p| p.kind == WORLD_POINT_TYPE).count();
        println!("squad menu: the level has {} world points ({plain} plain), {usable} the cursor may take", points.len());
    }
    let root = commands.spawn((Transform::default(), Visibility::Hidden, Name::new("move to marker"))).id();
    commands.insert_resource(Marker(root));
    let Some(&arch) = game.0.object_meshes.get(&MARKER_TYPE) else {
        warn!("the Move To marker's type h_{MARKER_TYPE:08x} has no mesh: no marker");
        return;
    };
    let lamps = bf_viewer::level_scene::lamps_of(map.0.as_ref(), &mut assets.buffers);
    let mut cache = HashMap::new();
    let parts = bf_viewer::level_scene::object_parts(&mut game.0, arch, &mut assets.meshes, &mut assets.materials, &mut assets.images, &lamps, &mut cache);
    bf_viewer::level_scene::spawn_object_parts(&mut commands, &parts, root, |p| p.material.clone());
}

/// The menu and the cursor drawn: the root shown while either is up; the backdrop's darkening
/// and the texts' and icons' alphas by the fade (FUN_000c09a0, 0xc19d0, 0xbdce0); the marker at
/// the cursor's point, turned.
#[allow(clippy::type_complexity)]
fn draw(menu: Res<SquadMenu>, nodes: Option<Res<MenuNodes>>, marker: Option<Res<Marker>>, mut vis: Query<&mut Visibility>,
        mut texts: Query<&mut RecordText>, mut images: Query<&mut ImageNode>, mut colours: Query<&mut BackgroundColor>,
        mut places: Query<&mut Transform>) {
    let m = &*menu;
    let mut show = |e: Entity, on: bool| {
        if let Ok(mut v) = vis.get_mut(e) {
            let want = if on { Visibility::Inherited } else { Visibility::Hidden };
            if *v != want {
                *v = want;
            }
        }
    };
    if let Some(mk) = marker.as_ref() {
        show(mk.0, m.cursor);
        if m.cursor {
            if let Ok(mut t) = places.get_mut(mk.0) {
                let want = Transform::from_translation(m.cursor_at).with_rotation(Quat::from_rotation_y(m.marker_turn));
                if *t != want {
                    *t = want;
                }
            }
        }
    }
    let Some(n) = nodes else { return };
    let up = m.up();
    show(n.root, up);
    show(n.menu, m.open);
    show(n.cursor, m.cursor);
    show(n.backdrop, m.open && m.fade > 0.0);
    if !up {
        return;
    }
    // FUN_000c09a0: k = (fade / 0.3)^2; the backdrop's grey (1 - k) x 175 + 80, the texts at
    // min(2k, 1); the entries at k (the order just given whole: 0xc19ec)
    let k = (m.fade / FADE).powi(2);
    let grey = ((1.0 - k) * BACKDROP_GREY.0 + BACKDROP_GREY.1) / 255.0;
    if let Ok(mut c) = colours.get_mut(n.backdrop) {
        let want = Color::BLACK.with_alpha(1.0 - grey.powf(SRGB_GAMMA));
        if c.0 != want {
            c.0 = want;
        }
    }
    let alpha = |f: Fades| match f {
        Fades::Text => (2.0 * k).min(1.0),
        Fades::Entry(e) => if m.last == Some(e) { 1.0 } else { k },
        Fades::Cursor => if m.open { 1.0 - m.fade / FADE } else { 1.0 },
    };
    for &(e, f) in &n.texts {
        if let Ok(mut t) = texts.get_mut(e) {
            let a = alpha(f);
            if (t.alpha - a).abs() > 1e-4 {
                t.alpha = a;
            }
        }
    }
    for &(e, f) in &n.images {
        if let Ok(mut img) = images.get_mut(e) {
            let a = alpha(f);
            if (img.color.alpha() - a).abs() > 1e-4 {
                img.color.set_alpha(a);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A step of the list: `dir` held for `dt`, every member selectable, the controlled one in
    /// slot 3 (Tex).
    fn step(m: &mut SquadMenu, dir: Option<usize>, dt: f32) -> Vec<Act> {
        let mut acts = vec![];
        m.dpad(dir, dt, true, false, &|_| true, &mut acts);
        acts
    }

    /// A press plays h_0c0d7e4d and flashes; held 0.25 s it selects (h_fac5d6b4) once; the menu
    /// then open, the key still held reads as nothing (`read_dir`), so nothing more however long
    /// it's held, nor at its release (take01: the press at 2.14 s, the flag at 2.34, LEFT held to
    /// 3.2 s, Hawk kept).
    #[test]
    fn a_hold_selects() {
        let mut m = SquadMenu::default();
        let dt = 1.0 / 60.0;
        assert_eq!(step(&mut m, Some(2), dt), vec![Act::Sound(S_PRESS), Act::Flash(2)]);
        let mut frames = 1;
        while !m.flags[2] {
            let acts = step(&mut m, Some(2), dt);
            assert!(acts.is_empty() || acts == vec![Act::Sound(S_ON)]);
            frames += 1;
            assert!(frames < 30);
        }
        assert!((frames as f32 * dt - HOLD).abs() < 2.0 * dt, "selected after {frames} frames");
        m.open = true;
        let held = MenuInput { dpad: [false, false, true, false], ..default() };
        for _ in 0..60 {
            let d = read_dir(m.open, &held);
            assert!(step(&mut m, d, dt).is_empty());
        }
        assert!(step(&mut m, None, dt).is_empty());
        assert!(m.flags[2]);
        // (closed, a held key is read as held: `dpad`'s tap and hold)
        assert_eq!(read_dir(false, &held), Some(2));
        let down = MenuInput { dpad_down: [true, false, false, false], ..default() };
        assert_eq!(read_dir(true, &down), Some(0));
    }

    /// Let go before 0.25 s: with the menu closed a tap switches to the member (take02: 100 and
    /// 200 ms presses switched); with it open, a tap toggles (take03).
    #[test]
    fn a_tap_switches_or_toggles() {
        let mut m = SquadMenu::default();
        let dt = 1.0 / 15.0;
        step(&mut m, Some(2), dt);
        step(&mut m, Some(2), dt);
        assert_eq!(step(&mut m, None, dt), vec![Act::Switch(2)]);
        assert!(!m.flags.iter().any(|&f| f));
        // (the frame after a release takes no press: its +0x84 is still set, 0xc81a7)
        step(&mut m, None, dt);
        m.open = true;
        m.lockout = [0.0; SQUAD];
        step(&mut m, Some(0), dt);
        assert_eq!(step(&mut m, None, dt), vec![Act::Sound(S_ON)]);
        assert!(m.flags[0]);
        // (again: off once its lockout is over)
        for _ in 0..5 {
            step(&mut m, None, dt);
        }
        step(&mut m, Some(0), dt);
        assert_eq!(step(&mut m, None, dt), vec![Act::Sound(S_OFF)]);
        assert!(!m.flags[0]);
    }

    /// Each action locks its direction out for 0.3 s: a press then is no press at all.
    #[test]
    fn the_lockout() {
        let mut m = SquadMenu { open: true, ..default() };
        let dt = 0.05;
        step(&mut m, Some(1), dt);
        assert_eq!(step(&mut m, None, dt), vec![Act::Sound(S_ON)]);
        assert!(step(&mut m, Some(1), dt).is_empty());
        assert!(step(&mut m, None, dt).is_empty());
        assert!(m.flags[1]);
        for _ in 0..6 {
            step(&mut m, None, dt);
        }
        assert_eq!(step(&mut m, Some(1), dt), vec![Act::Sound(S_PRESS), Act::Flash(1)]);
    }

    /// A member that may not be selected (dead): no press sound, and its hold neither selects nor
    /// sounds; one flagged that dies drops out with h_0d93fbd9.
    #[test]
    fn the_dead_drop_out() {
        let mut m = SquadMenu::default();
        let not_hawk = |c: usize| c != 2;
        for _ in 0..10 {
            let mut acts = vec![];
            m.dpad(Some(2), 0.05, true, false, &not_hawk, &mut acts);
            assert!(acts.is_empty());
        }
        assert!(!m.flags[2]);
        let mut m = SquadMenu { flags: [true, false, false, false], ..default() };
        let mut acts = vec![];
        m.dpad(None, 0.1, true, false, &|c| c != 0, &mut acts);
        assert_eq!(acts, vec![Act::Sound(S_OFF)]);
        assert!(!m.flags[0]);
    }

    /// The menu: open on a selection (paused, fading in over 0.3 s), an order closes it
    /// (fading out, then the game runs again and the selection is gone); the HUD's share and the
    /// radar's slide follow the fade.
    #[test]
    fn the_fade() {
        let mut m = SquadMenu::default();
        let mut acts = vec![];
        m.open_menu(&mut acts);
        assert_eq!(acts, vec![Act::Sound(S_OPEN)]);
        assert!(m.paused_game);
        m.run_fade(0.15);
        assert!((m.hud_share() - 0.5).abs() < 1e-5);
        assert!((m.radar_slide() - 0.875).abs() < 1e-5);
        m.run_fade(0.2);
        assert_eq!((m.fade, m.fading_in), (FADE, false));
        m.flags[2] = true;
        let mut acts = vec![];
        m.close_menu(&mut acts);
        assert_eq!(acts, vec![Act::Sound(S_CLOSE)]);
        m.run_fade(0.2);
        assert!(m.open && m.paused_game);
        m.run_fade(0.2);
        assert!(!m.open && !m.paused_game);
        assert!(!m.flags[2]);
        assert_eq!(m.hud_share(), 1.0);
    }

    /// BF_TEST_COMMAND's entries: a member with or without its hold, the orders by name or button,
    /// BACK and the cursor's two; a bad one skipped.
    #[test]
    fn test_presses_parse() {
        let t = parse_tests("2:hawk:0.4; 3.5:brutus ;5:y;6:move;7:black;8:back;9:accept;x;10:nobody");
        assert_eq!(t.len(), 7);
        assert_eq!(t[0], TestPress { at: 2.0, what: TestWhat::Member(2), hold: 0.4 });
        assert_eq!(t[1], TestPress { at: 3.5, what: TestWhat::Member(0), hold: TEST_TAP });
        assert_eq!(t[2].what, TestWhat::Order(STAND));
        assert_eq!(t[3].what, TestWhat::Order(MOVE));
        assert_eq!(t[4].what, TestWhat::Order(MEDKIT));
        assert_eq!(t[5].what, TestWhat::Back);
        assert_eq!(t[6].what, TestWhat::Accept);
    }

    /// The keyboard (`menu_input`): 1-4 the d-pad (held and pressed), I L K J B V the orders in the
    /// game's order of reading (B Fire At Will before Y Stand Ground), Backspace BACK; open, the
    /// menu takes the frame's presses (I opens no objectives menu); with the cursor up, E and the
    /// mouse buttons are its own, let go (no use, no fire).
    #[test]
    fn the_keys() {
        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default()).insert_resource(ButtonInput::<MouseButton>::default())
            .insert_resource(Player::new(3)).insert_resource(SquadMenu::default()).insert_resource(MenuInput::default())
            .add_systems(Update, menu_input);
        let press = |app: &mut App, keys: &[KeyCode]| {
            let mut k = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            k.reset_all();
            for &key in keys {
                k.press(key);
            }
        };
        press(&mut app, &[KeyCode::Digit3, KeyCode::KeyI, KeyCode::KeyL]);
        app.update();
        let i = *app.world().resource::<MenuInput>();
        assert_eq!((i.dpad, i.dpad_down), ([false, false, true, false], [false, false, true, false]));
        assert_eq!(i.order, Some(FIRE));
        assert!(app.world().resource::<ButtonInput<KeyCode>>().just_pressed(KeyCode::KeyI), "closed, the presses stay");
        app.world_mut().resource_mut::<SquadMenu>().open = true;
        press(&mut app, &[KeyCode::KeyI, KeyCode::Backspace]);
        app.update();
        let i = *app.world().resource::<MenuInput>();
        assert_eq!((i.order, i.back), (Some(STAND), true));
        assert!(!app.world().resource::<ButtonInput<KeyCode>>().just_pressed(KeyCode::KeyI), "open, the menu takes them");
        {
            let mut m = app.world_mut().resource_mut::<SquadMenu>();
            (m.open, m.cursor) = (false, true);
        }
        press(&mut app, &[KeyCode::KeyE]);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        let i = *app.world().resource::<MenuInput>();
        assert!(i.accept && !i.cancel);
        assert!(!app.world().resource::<ButtonInput<KeyCode>>().pressed(KeyCode::KeyE));
        assert!(!app.world().resource::<ButtonInput<MouseButton>>().pressed(MouseButton::Left));
    }

    /// The rows' widths: "Give Order"'s six glyphs are A, B, X and Y (18 each with their offset)
    /// and Black and White (16): 104, then the gap (take01: the words 108 units after the row's
    /// start).
    #[test]
    fn glyph_rows() {
        assert_eq!(glyphs_width(&GIVE_GLYPHS), 104.0);
        assert_eq!(glyphs_width(&CLOSE_GLYPHS), 26.0);
        assert_eq!(row_width(&[], "", &BODY12), 0.0);
    }
}
