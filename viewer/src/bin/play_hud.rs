//! The in-game HUD, laid out as in Brute Force: health / energy bars top left, the weapon panel
//! (icon, clip / reserve, a full list for a moment after a switch) top right, radar with the
//! character's portrait bottom left, and the crosshair above centre. (The item box, bottom
//! right, is play_itembox.rs's.)
//!
//! Positions are in the game's 640 x 480 screen, measured from an xemu capture, inside a 4:3 area
//! centred in the window (so it scales with the window like the game). Textures are the game's
//! own (data/common.tgz), identified by the HUD page of the options screen
//! (common/game-options-en.xml: radar 132e4ee1 + 17d0ee37, weapon panel e11fb292, portraits) and
//! the weapon definitions' icon (h_e5ec3f1f); like all the game's textures they are stored
//! upside down, so they are drawn flipped. Text uses Bevy's font, not the game's bitmap font.

use super::*;
use super::fonts::{record_text, Across, RecordText, BODY12, HEADING24};
use super::text::{game_text, Align, GameText, AMMO_ORANGE, MESSAGE_BLUE};

/// radar frame (octagon) and its disc
const RADAR_FRAME: u32 = 0x132E_4EE1;
const RADAR_DISC: u32 = 0x17D0_EE37;
/// weapon panel background (128 x 32)
const WEAPON_PANEL: u32 = 0xE11F_B292;
const CROSSHAIR_TEX: u32 = 0x1D2A_68D0;
const ENERGY_ICON: u32 = 0x1AC4_1530;

/// The target ring (the ring widget, vtable 0x3a02e8): its size once zoomed in (+0x94, 48.0 from
/// its constructor 0xa9b90; take11's ring 48 units across on the L-Shot-50 and 47 at the gate's
/// panel, #17, where this was 60 from an earlier fit), and how much bigger it starts: its update
/// (0xa9bf0) draws it 48 x (1 + 1.5 (1 - c^4)) across at alpha c^4, c its zoom (`Ring::open`).
const RING_SIZE: f32 = 48.0;
const RING_GROW: f32 = 1.5;
/// Its colours (0xa9bf0's bytes, the sprite's +0x2c red, +0x2d green, +0x2e blue): 23 53 d4 for a
/// target the character can use (take11: (33-36, 80, 206) through the video), a0 20 20 for one it
/// can't (#61 take04's red ring on the Jax-iP), 3c 69 dc while it's being used (take11 300.6-301.1:
/// (60, 101, 210)).
const RING_BLUE: Color = Color::srgb(0x23 as f32 / 255.0, 0x53 as f32 / 255.0, 0xD4 as f32 / 255.0);
const RING_RED: Color = Color::srgb(0xA0 as f32 / 255.0, 0x20 as f32 / 255.0, 0x20 as f32 / 255.0);
const RING_IN_USE: Color = Color::srgb(0x3C as f32 / 255.0, 0x69 as f32 / 255.0, 0xDC as f32 / 255.0);
/// Pickup lines shown at once, and where they go: fixed on the screen, centred under the
/// character's usual place, the status message above them (the capture's spacing: lines 29
/// and 45 under the message).
const FEED_LINES: usize = 4;
const MESSAGE_Y: f32 = 290.0;
const FEED_FIRST: f32 = 29.0;
const FEED_STEP: f32 = 16.5;
/// The message line (the HUD entry's widget +0x114, name 0xe2548d48, constructor FUN_000b32d0):
/// the body font at 12 points (FUN_000cd9c0(0xc626fd5, 12): play_fonts.rs BODY12) in
/// (192, 255, 255) (its constructor's bytes from FUN_000b87a0's c0 d7 ff ff, the green taken from
/// the third byte: take10's "No need to heal" (192, 245, 244)) with a shadow (+0x3b bit 4), centred
/// at x 320; its alpha 255 x min(1, 2 x time left) (its update 0xb33b0: a 0.5 s fade). Its top is
/// a fit: the takes' glyph tops at 292.7-294.7 (take10 8.5 s, take21 4.5 s, take11 3.2 s, take26
/// 7.0 s, #196 spec AC-27); at 289 the demo's "No need to heal" spanned y 293.3-305.3 against
/// take10's 292.0-304.0 (8.0 s; x 269.3-370.0 in both), so 1.3 higher.
const MESSAGE_RGB: [u8; 3] = [192, 255, 255];
const MESSAGE_TOP: f32 = 287.7;
/// Strings: "switch to %s." (after the Y button's icon), "Regen".
const S_SWITCH_TO: u32 = 0xFF01_7DF6;
const S_REGEN: u32 = 0xE246_DE1D;
/// A pickup line that took nothing, its stack full (#17): "<name> (full)", the word h_f1416a07
/// (FUN_000ad790 at 0xad7d9), in FUN_000b8820's colour, bytes a0 20 20 (the line's +0x38 red,
/// +0x3a green, +0x39 blue, as the message lines take them): the DNA video's "Bio Ammo (full)",
/// measured (170, 42, 30) through the video's compression.
const S_FULL: u32 = 0xF141_6A07;
const FULL_RED: Color = Color::srgb(160.0 / 255.0, 32.0 / 255.0, 32.0 / 255.0);
/// A weapon row's icon size (HUD units) where its item names none: 128 x 64, the rifles' and
/// heavy guns'. The five pistols (weapon-type 0: the Saryl-23, Foley, MNR-7 Bio, LZR-10 and
/// Jax-iP) name 64 x 64 (`WeaponDef::icon_size`, #64). Its top left is (463, the row's y - 25).
const WEAPON_ICON: [f32; 2] = [128.0, 64.0];
/// "[Y] switch to %s." once the held gun's clip and the reserve are both empty (FUN_00120d40
/// posts message 0x52): top left under the health bar, pale blue (todo/49 take02 19 s; the same
/// line as the pickup prompt). Here the key (Q) stands for the button's icon (the wall panel's
/// prompt draws the game's glyph line: play_hints.rs). Measured there: "switch"'s letters span y 90-104 (14 units; with CAP 11.5 ours
/// spanned 12.5, so CAP is scaled to 13), and TOP (the text line's top) puts them at 90.
const SWITCH_HINT_X: f32 = 53.0;
const SWITCH_HINT_TOP: f32 = 85.8;
const SWITCH_HINT_CAP: f32 = 13.0;
/// portraits by character (CHARACTERS order: Brutus, Flint, Hawk, Tex) and the centres of the
/// faces round the radar: the whole squad, Tex top, Hawk left, Flint right, Brutus bottom (game
/// screenshot, positions refined by template matching against it; same order as the options
/// screen's HUD page). The faces are packed into a corner
/// of their 32 x 32 textures (Flint and Hawk 21 x 27, Brutus 27 x 24, Tex 27 x 30), so each is
/// cropped to its opaque pixels and drawn 1:1 (a texel per screen unit) centred there.
const PORTRAITS: [u32; 4] = [0xFDFB_4331, 0xEB2F_DA05, 0x0823_79F7, 0x076B_684B];
const PORTRAIT_CENTRE: [(f32, f32); 4] = [(122.2, 428.5), (180.5, 368.2), (64.1, 368.5), (122.2, 303.8)];
/// The command tab beside each portrait (#333): the tab widget FUN_000c3d10 (vtable 0x39f4f0, its
/// update 0xc3f80 at +0x14 and draw 0xc4130 at +0x18; the HUD factory makes one a portrait at
/// 0xb9af8), an 8-unit sprite whose texture and size its update sets each frame by the command
/// mode of its slot's member's brain (+0x58, its jump table 0xc40e0 on the mode less 1; the
/// constructor loads the five textures): FOLLOW (Cover Me, the squad's start) h_eeaa5549, green
/// arrows, 16 x 8; GUARD (Stand Ground) h_fa29b7a4, yellow bars, 8 x 16; WINGMAN (Fire At Will)
/// h_f6722366, a ring of red and orange dots, 16 x 16; MOVE_OUT (Move To) h_eed3270a, a blue arrow,
/// 16 x 16; FORMATION and any other mode (FREEWILL, the frozen 6) none: hidden (#35 take18, the
/// trainer's squad in mode 6: Brutus's and Flint's tabs empty). The fifth, h_e878453f (+0x40), is
/// loaded and never shown. Measured: take03 1.0 s the four green arrows; Hawk's and Flint's
/// yellow bars from 5.25 s after Stand Ground; take08 Hawk's blue arrow from 25.8 s after Move To,
/// her bars from 31.1 s at the point; #33 take05 Fire At Will's ring on Flint's, Brutus's and
/// Hawk's tabs from 5.4 s.
const TAB_FOLLOW: (u32, f32, f32) = (0xEEAA_5549, 16.0, 8.0);
const TAB_GUARD: (u32, f32, f32) = (0xFA29_B7A4, 8.0, 16.0);
const TAB_WINGMAN: (u32, f32, f32) = (0xF672_2366, 16.0, 16.0);
const TAB_MOVE_OUT: (u32, f32, f32) = (0xEED3_270A, 16.0, 16.0);
/// Each tab's top left (FUN_000c3f40 from the update: the table 0x3c8ea8, (x, y) by the game slot
/// less 1, 4 on for a small screen), from the radar panel's corner, by character (CHARACTERS:
/// Brutus slot 3, Flint 2, Hawk 4, Tex 1). Each icon's art fills its texture's top left. The
/// corner is the one the demo draws the radar frame and the brackets from (RADAR_FRAME_AT), so
/// each icon sits in its box of the frame as in the takes. The code's corner on a full screen is
/// (56, 301) (FUN_000c2be0: the HUD safe area's left + 5, its bottom - 10 less the panel's 128),
/// where take03's and take08's icons start (Hawk's arrows, bars and arrow from (61, 382), Tex's
/// arrows from (94, 306)); the demo's frame is drawn 1.5 right and 4 lower than that (take03 1.0
/// s: Tex's box's edges at x 90.7 / 104.7, y 302.7 / 317.3 against the demo's 92.0 / 106.0, 306.0
/// / 320.7), and the tabs with it.
const TAB_AT: [(f32, f32); 4] = [(82.0, 114.0), (115.0, 37.0), (5.0, 81.0), (38.0, 5.0)];
/// The tab's flash at a new icon: the update sets +0x4c and zeroes +0x48 whenever the texture
/// changes (0xc4002-0xc4099, the first one set included), FUN_000c4100 adds 1.5 (0x40d6b0) x the
/// HUD's dt and ends it past 1.0 (0.67 s). Its draw (0xc4130) puts a copy of the icon under it
/// first: the widget's scale set to s = 1 + 1.8 t (0x3a5234; its +0x18 through the vtable's
/// +0x1c 0xa9270), the sprite draw (0xba950, which takes the size and the offset times that scale)
/// at the offset -1.8 (s - 1) each way (0x40d6ac, 0x40d6a8) and alpha 255 x clamp(2 (1 - t)^2, 0, 1)
/// (0x3a50e8); then the scale back to 1 and the icon (the base draw 0xbb300). So a copy that grows
/// to 2.8 times, whole for its first 0.2 s and gone at 0.67. take03: Hawk's bars a 9.3 x 12 blob
/// 5.35-5.55 s against their 7 x 9.3 after; the green arrows bold at 14.2-14.3 s after Cover Me.
const TAB_FLASH_RATE: f32 = 1.5;
const TAB_FLASH_GROW: f32 = 1.8;
const TAB_FLASH_SHIFT: f32 = -1.8;
const SQUAD_ICON: u32 = 0xE064_867A;
const HEALTH_ICON: u32 = 0x1BFA_5678;
/// bar fills (4 x 8 gradients): health red, energy blue
const HEALTH_FILL: u32 = 0xE801_997B;
/// The charge meter (the recordings, every grenade): a blue outlined bar x 354.0-367.7,
/// y 162.0-221.7 right of the crosshair (todo/119 take01: outline y 161.3-222.0, fill y 164.0-218.7,
/// tick y 178-180); its fill x 355.3-365.7 grows linearly from
/// y 218.7 to 164.3 (full), sRGB (255, 160, 53) at ~0.65 opacity; a dark divider at y ~179.
/// (left, top, right, bottom)
const METER_BOX: (f32, f32, f32, f32) = (354.0, 162.0, 367.7, 221.7);
/// (left, right, top when full, bottom)
const METER_FILL: (f32, f32, f32, f32) = (355.3, 365.7, 164.3, 218.7);
const METER_TICK_Y: f32 = 178.5;
/// The fill: sRGB (255, 160, 53) at 0.65, blended by the console in stored values (over the
/// recording's grey ground (63, 63, 55) it gives (188, 126, 54)). The UI blends in linear light,
/// so it's drawn as the colour that gives the same result over that ground: (226, 148, 54) at
/// 0.65 (fitted over that one ground colour; lighter or darker ground comes out a little off).
const METER_ORANGE: Color = Color::srgba(226.0 / 255.0, 148.0 / 255.0, 54.0 / 255.0, 0.65);
/// The dark bar across the meter at METER_TICK_Y (the frame texture h_e4d89656's grey-63 bar,
/// 0xadac0; the demo's colour).
const METER_TICK: Color = Color::srgb(0.08, 0.12, 0.25);
/// The meter's fade, drawn in linear light. The game's widget alpha (0xad950: clamp(2 x timer,
/// 0, 1)^3) blends in stored sRGB values, so the strength the take measures, the share of the
/// change from the ground to the full meter, is that alpha; the UI blends in linear light, where
/// the same alpha looks stronger. So the demo's alpha is the one that gives that strength over
/// take01's ground inside the box (sRGB (37, 27, 20), #119 spec §5.3), as luminances with a 2.2
/// gamma: METER_GROUND that ground's, METER_FULL the full fill's over it in the demo (188, 122,
/// 45). Against the exact sRGB curves per channel it's within 0.007 (#162); over a lighter
/// ground the demo's fade comes out a little fainter than the game's.
const METER_GROUND: f32 = 28.0 / 255.0;
const METER_FULL: f32 = 130.0 / 255.0;
/// radar: centre of the disc (screen units; the radar widget's place + 64, its +0xd4 / +0xd8)
const RADAR_CENTRE: (f32, f32) = (121.5, 369.0);
/// The radar's max-range circle in screen units: its +0x88, 39 on a full screen (set at 0xadc70
/// and by FUN_000adf20; 29 on a small or split one). FUN_000b0550 turns a world offset by the view
/// and scales it by this over the game options' max-range (+0x94 = +0x88 / +0x90,
/// 0xae255-0xae26a): 120 m across 39 units, 0.325 a metre. take07 agrees: the waypoint 17.4 m off
/// drawn 5.4 units from the middle (5.65 by the code), the next, 55.5 m off, 17.0 (18.0). (The
/// demo's 40 m across 44 units before #228 had no source.)
const RADAR_UNITS: f32 = 39.0;
/// The squad's blips: FUN_000ae7c0's colour for a friend (a character on the viewer's team, a team
/// not hostile to itself: FUN_0004b960), 90 200 40 (0xae9a9-0xae9b9); take11's marines are green.
/// (The demo's yellow before #228 had no source.)
const SQUAD_GREEN: Color = Color::srgb(90.0 / 255.0, 200.0 / 255.0, 40.0 / 255.0);
/// The viewer's own dot in the radar's middle: 180 180 180 (FUN_000ae7c0's colour for the HUD's
/// own character, 0xae7f0 / 0xae934, in every mode; take03 and take07 show it pale grey). The
/// squad deathmatch recordings' green middle is its squadmates' dots over it: within a few metres
/// of the viewer at 0.325 units a metre (the demo draws its squad's blips over it too).
const VIEWER_GREY: Color = Color::srgb(180.0 / 255.0, 180.0 / 255.0, 180.0 / 255.0);
/// FUN_000ae7c0's colours for the other characters on the radar (radar-contact 1, or 6 for a
/// boss: the second of each pair): a friend 90 200 40 / 40 100 20 (0xae992-0xae9b9); one whose
/// team the viewer's is hostile to (FUN_000a8520) 200 40 90 / 100 20 40 (0xaea3d-0xaea84: take08
/// and take11's terrorists are red); anyone else 70 70 70 (0xaea9d).
const CHARACTER_FRIEND: [[f32; 3]; 2] = [[90.0 / 255.0, 200.0 / 255.0, 40.0 / 255.0], [40.0 / 255.0, 100.0 / 255.0, 20.0 / 255.0]];
const CHARACTER_HOSTILE: [[f32; 3]; 2] = [[200.0 / 255.0, 40.0 / 255.0, 90.0 / 255.0], [100.0 / 255.0, 20.0 / 255.0, 40.0 / 255.0]];
const CHARACTER_NEUTRAL: [f32; 3] = [70.0 / 255.0; 3];
/// The radar's dot texture h_ecea7a68 (the radar's +0x34, looked up at 0xadd15): an 8 x 8 soft
/// white disc (A8), what each of its contact records is drawn with (0xb01b0).
const RADAR_DOT: u32 = 0xECEA_7A68;
/// An objective's record lives this long when its object wasn't seen in the last SEEN_TIME s (the
/// radar's +0xa4, 1.0) and PING_LIFE (+0xa0, 0.3) otherwise (FUN_000aece0 picks by FUN_000ae360).
const PING_NEW_LIFE: f32 = 1.0;
const PING_LIFE: f32 = 0.3;
/// How long an object counts as seen after its last record (the radar's +0xa8, 3.0: FUN_000ae360
/// sets it, FUN_000ae300 counts it down).
const SEEN_TIME: f32 = 3.0;
/// A record's alpha x min(life / PING_FADE, 1) (+0x98, 0.25: 0xb02bd); it grows by PING_GROWTH
/// a second (+0x9c, 8.0 on a full screen: FUN_000ae3f0) from a half-size of 1 unit (+0xdc).
const PING_FADE: f32 = 0.25;
const PING_GROWTH: f32 = 8.0;
/// An objective beyond the max-range: its rim marker. FUN_000aece0 hands FUN_000b0ad0 its point on
/// the circle (A) and 5.0 x +0xac (1.0 on a full screen): B, this far in from A on the way to the
/// object. FUN_000af840 draws the record (type 1, 0xafaa0) as a line list from A to B pushed out
/// to each side by as much, square to A-B (0xaf9a5-0xaf9c5): an "L" with its corner on the circle,
/// its arms 5 in and 5 across (7.1 units long). take11 at 330 s and the squad deathmatch recordings
/// show it so.
const RIM_DEPTH: f32 = 5.0;
/// The rim marker's line width (units): the game's is a line list, a pixel of its 640 x 480;
/// take11's and the recordings' are ~2 px of their 1280 x 720 frames, 1.33 units (a fit).
const RIM_LINE: f32 = 1.33;
/// The blink's dim alpha: 255 >> 6 (0xaedd5), on the half of each 2 s where frac(t / 2) < 0.5.
const BLINK_DIM: f32 = 3.0;
/// The objectives' colour, 255 255 0, and the important pickup's (7: the memory chip), 30 90 30
/// (FUN_000aece0's record +0x10 / +0x14 / +0x18, which the draw packs as D3DCOLOR ARGB with them
/// as R G B, 0xb033c-0xb0374; take07's dot is pure yellow).
const OBJECTIVE_YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
const PICKUP_GREEN: [f32; 3] = [30.0 / 255.0, 90.0 / 255.0, 30.0 / 255.0];
/// The radar overlay's pixels per screen unit (its image is 128 units square).
const OVERLAY_PX: usize = 2;
/// Squad health bars fill the radar frame's own dark channels on its diagonals (texels of the
/// frame texture, display rows, darker than HEALTH_DARK). By character (Brutus, Flint, Hawk,
/// Tex): the channel's quadrant (x right, y down) and the portrait notch it's anchored at (frame
/// texels), so the bar empties toward the member's portrait: Brutus bottom-left toward the
/// bottom, Flint bottom-right toward the right, Hawk top-left toward the left, Tex top-right
/// toward the top.
const HEALTH_BARS: [((bool, bool), (f32, f32)); 4] = [
    ((false, true), (63.0, 115.0)),
    ((true, true), (115.0, 65.0)),
    ((false, false), (12.0, 63.0)),
    ((true, false), (65.0, 12.0)),
];
const HEALTH_DARK: u32 = 27;
/// where the radar frame sits (screen units, 128 x 128), the corner the brackets and the command
/// tabs are placed from. (#333 found the game's 1.5 left and 4 higher, at the radar panel's corner
/// (56, 301) by FUN_000c2be0: take03's frame and tabs there; not moved here, see TAB_AT)
const RADAR_FRAME_AT: (f32, f32) = (57.5, 305.0);
const HEALTH_PINK: Color = Color::srgb(0.96, 0.45, 0.42);
/// The radar panel's slide while the Squad Command menu is open (#319): the radar and its
/// portraits are one widget (0xc2d40, 128 x 128, its update 0xc2ac0), which FUN_000c2be0 places
/// each frame between its corner, the HUD safe area's left + 5 and bottom - 10 less its height
/// (56, 301), and the screen's middle less its width and 20 across, (480 - 128) / 2 + 5 down
/// (172, 181), by 1 - (1 - fade / 0.3)^3 of the menu's fade (play_squad_menu.rs
/// `SquadMenu::radar_slide`). take01: its middle dot 114 right and 124 up with the menu open
/// (the dots' blips blur it a unit or two).
const RADAR_SLIDE: (f32, f32) = (116.0, -120.0);
/// The radar panel over the menu's backdrop (play_squad_menu.rs MENU_Z, 2), which darkens the rest
/// of the HUD but not it (take01 4.0 s: the radar and its faces as bright as before), under the
/// menus (20).
const RADAR_Z: i32 = 3;
/// A selected member on the radar (FUN_000c86d0 true): FUN_000ae7c0 draws its record 200 200 200
/// (take03: the middle dot pale with members selected) ...
const SELECTED_GREY: Color = Color::srgb(200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0);
/// ... its portrait in FUN_000b87e0's ff a0 6a (0xc4280 gives the portrait's colour bytes that
/// while its flag is set) ...: not as a plain multiply, though: take01 and take03's selected faces
/// are the face's brightness in that orange, their red risen (Brutus's face (55, 80, 106) went
/// (96, 61, 42), Hawk's (102, 114, 139) (145, 92, 61): both its luminance x the orange x 1.28).
/// How the sprite draw makes it isn't traced: SELECTED_GAIN is that fit (`selected_face`).
const SELECTED_TINT: [f32; 3] = [1.0, 160.0 / 255.0, 106.0 / 255.0];
const SELECTED_GAIN: f32 = 1.28;
/// The selected faces' images in HudImages: under their texture's name xor this.
const SELECTED_KEY: u32 = 0x0519_E1EC;
/// ... and a bracket round its portrait shown, the part 0x1207a43b (FUN_000c3a50, its update
/// FUN_000c3bc0): h_ff5d4d4f for Tex and Brutus (slots 1 and 3, 0xc3c79 / 0xc3c8d), h_fdec2fc6 for
/// Flint and Hawk, placed by the table 0x3c8ca8 (FUN_000c3cc0: x, y, width, height from the radar
/// panel's corner, a negative size mirrored), by slot as CHARACTERS: Brutus (49, 133, 32, -32),
/// Flint (133, 49, -32, 32), Hawk (-4, 49, 32, 32), Tex (49, -5, 32, 32). take03: Hawk's "]" and
/// Flint's "[" closed toward the radar, Brutus's open below.
const SELECT_FRAMES: [(u32, (f32, f32, f32, f32)); 4] = [(0xFF5D_4D4F, (49.0, 133.0, 32.0, -32.0)), (0xFDEC_2FC6, (133.0, 49.0, -32.0, 32.0)),
                                                         (0xFDEC_2FC6, (-4.0, 49.0, 32.0, 32.0)), (0xFF5D_4D4F, (49.0, -5.0, 32.0, 32.0))];
/// The squad switch's name label (#88): the widget FUN_000c92e0 makes (vtable 0x39f1e8), the
/// name in the heading font at 24 points (its text's font 0x97ecc77 at 0x18: the record 0x3c7b90 on
/// h_e0afcd52, `fonts::HEADING24`), centred on the screen (FUN_000c9610: at half its parent's
/// size, the HUD entry's 640 x 480). Shown for LABEL_SHOW + LABEL_OUT from each selection
/// (0xc9507: its timer +0x4c = +0x50 + +0x54), whole for the first, then over the last its alpha
/// falls to 0 and it swells (its update 0xc9690). The recordings agree: "HAWK" from 31.847 s to
/// about 32.5 s in `Friendly Fire 2 + Death Cam.mp4`, fading from about 32.2 s; "BRUTUS" from
/// 22.867 s, fading from 23.267 s.
const LABEL_SHOW: f32 = 0.4;
/// (+0x54: see LABEL_SHOW) the zoom-out's length (s)
const LABEL_OUT: f32 = 0.3;
/// +0x58: its alpha while whole (0.8: the recordings' (209, 138, 86) over the gas)
const LABEL_ALPHA: f32 = 0.8;
/// +0x5c and +0x60: its scale, 1 + 3 k^3 with k the zoom-out's share gone (0xc973f-0xc974b): 4x
/// at its end (the recordings: 1.13, 1.28, 1.6, 2.1 and 2.73 times "HAWK"'s width at 32.30,
/// 32.335, 32.37, 32.40 and 32.435 s, its alpha falling with it)
const LABEL_SCALE: f32 = 1.0;
const LABEL_SWELL: f32 = 3.0;
/// Its colour: FUN_000b87e0's, bytes ff a0 6a (0xc9581-0xc9599 put them in its text's red, green,
/// blue), orange. (The label of a character the select widget already holds, its +0x68, takes
/// 0x4a5c90's 3c 69 dc, blue: the demo never draws that.)
const LABEL_ORANGE: Color = Color::srgb(1.0, 160.0 / 255.0, 106.0 / 255.0);
/// Its centre on the 640 x 480 screen (FUN_000c9610; the recordings' "HAWK" ink 272.0-368.7 x
/// 230-251)
const LABEL_CENTRE: (f32, f32) = (320.0, 240.0);
/// the selection arrow: the game's chevron texture (158f87c1, drawn mirrored / turned to point at
/// the portrait), three of them 5 units apart, 30 units square, the first 28 units short of the
/// portrait's centre (capture), red, lit one after another
const CHEVRON: u32 = 0x158F_87C1;
const CHEVRON_RED: Color = Color::srgb(1.0, 0.33, 0.2);
/// A d-pad press's chevron flash (#319): the flash widget (0xf5c0f471, FUN_000c48c0, its draw
/// 0xc4ba0) draws its three chevrons at FUN_000c4cd0's alpha, clamp((level - 0.5 - 0.1 n) x
/// 3.333, 0, 1) x 254 (0x3a4f74, 0x3a4ff0, 0x3a5174, 0x3a516c), n 0 for the one nearest the
/// portrait: the level falling from 1 (play_squad_menu.rs `SquadMenu::flash`), the outer two go
/// first and the last 0.25 s after the press (take01: all three at 2.10-2.15 s, the inner one
/// alone at 2.20, gone by 2.30).
const PRESS_FADE: (f32, f32, f32) = (0.5, 0.1, 3.333);

/// A press flash's chevron `k` (the demo's: 0 the farthest from the portrait) at the flash's
/// `level` (PRESS_FADE).
fn press_alpha(level: f32, k: usize) -> f32 {
    let n = 2usize.saturating_sub(k) as f32;
    ((level - PRESS_FADE.0 - PRESS_FADE.1 * n) * PRESS_FADE.2).clamp(0.0, 1.0) * 254.0 / 255.0
}

/// The speech icon's size on the 640 x 480 screen: its texture (SQUAD_ICON, h_e064867a, 16 x 16 DXT3)
/// drawn 1:1, as the game draws its HUD textures. Before #332 it was drawn 10 x 10, a third too
/// small (take01's bubble ink is 12 units wide).
const BUBBLE_SIZE: f32 = 16.0;
/// Speech icons by character (Brutus, Flint, Hawk, Tex): the top-left corner of each, inside the
/// radar ring on the portrait's inner side (#332). Measured from the squad-voice takes
/// (`todo/36-squad-voice`, 1280 x 720 with the 4:3 picture at x 160-1120, 1.5 px a unit): the
/// icon's white body (every channel over 185) at Hawk take01 17.6 s (x 78.0-89.3, y 344.7-352.0),
/// Tex take12 3.9 s (x 105.3-114.7, y 324.7-332.7), Brutus take12 12.6 s (x 103.3-114.7, y
/// 397.3-403.3) and Flint take08 4.9 s (x 151.3-162.7, y 344.7-351.3), put back to the texture's
/// corner by where its body sits in the 16 x 16 (flipped: x 2-13, y 3-13) and checked by
/// rendering the demo's and comparing the bodies the same way. The fit is to about 1 unit: the
/// game's own placement code isn't found (the table is in no function the code map lists).
const BUBBLE_AT: [(f32, f32); 4] = [(101.5, 393.5), (149.5, 340.5), (76.0, 341.0), (102.0, 321.0)];
/// a dead member's portrait: their own skull (the character's h_00c51907 icon; Brutus's is the
/// wide one), dimmed; their command tab and radar health bar go. Fallbacks if the data lacks one:
const SKULL_HUMAN: u32 = 0x1834_3E47;
const SKULL_BRUTUS: u32 = 0x19DA_7869;
const SKULL_TINT: Color = Color::srgb(0.6, 0.66, 0.6);
/// The test map's squad slots (#302, play_testsquad.rs) hold any type: a member whose type has no
/// portrait (`CharacterType::portrait`: the campaign's enemies) shows a "?" in its place instead,
/// this tall (units) with its line's top this many heights above the place's centre, which
/// centres the glyph on the place (the demo's choice: the game never shows one; at 0.55 the
/// captures had it 2.5-3 units low).
const UNKNOWN_CAP: f32 = 14.0;
const UNKNOWN_TOP: f32 = 0.75;

/// The crosshair: the held weapon's 64 x 64 texture drawn 1:1, centred at (320, 192); in the
/// scope it moves to the middle, (320, 240), the same size. Measured for all 11 weapons in their
/// take01s (todo/49-68, 2.5 s: each texture's alpha box matches the screen box to 1-2 units),
/// the same standing, firing and strafing (no growth); scoped in the Foley's, MK's and L-Shot's
/// (both zoom steps). (The scope's frame and Flint's overlay are play_scope_view.rs's.)
const CROSSHAIR_SIZE: f32 = 64.0;
const CROSSHAIR_Y: f32 = 192.0;

/// The health / energy frame's top-left corner (common h_0cdf5876, 32 x 16: the notched outer
/// edge and the bars' inset channel), mirrored into the other three, its last column stretched
/// between.
const BAR_FRAME_CORNER: u32 = 0x0CDF_5876;
/// the frame's width in texels (its height is two corners: 32), for its 208 x 37 place
const BAR_FRAME_WIDTH: u32 = 180;
/// its opacity (the capture's frame is glass: the scene shows through it)
const BAR_FRAME_ALPHA: f32 = 0.55;
/// where the frame is drawn (screen units), and its height in texels (two corners of 16)
const BAR_FRAME_AT: (f32, f32) = (64.0, 40.0);
const BAR_FRAME_SIZE: (f32, f32) = (208.0, 37.0);
const BAR_FRAME_TEXELS_HIGH: f32 = 32.0;
/// the frame's two channels, health then stamina: their open interiors in texel rows (upright:
/// the corner's lines are rows 5 and 12, mirrored below to 19 and 26)
const BAR_CHANNELS: [(f32, f32); 2] = [(6.0, 12.0), (20.0, 26.0)];
/// and their open interiors in texel columns (the corner's vertical line is column 4, mirrored
/// to 175): the bars fill them end to end (inset by hand, they left gaps at both ends)
const BAR_CHANNEL_COLS: (f32, f32) = (5.0, 175.0);
/// The bar fills (HEALTH_FILL, ENERGY_FILL: 4 x 8 texels) are coloured in their texel rows
/// 2-7 only (rows 0-1, as stored, are clear): just those are drawn, filling the channel's 6 open
/// rows exactly (drawn whole, the clear rows left the colour off-centre in its box)
const BAR_FILL_RECT: (f32, f32, f32, f32) = (0.0, 2.0, 4.0, 8.0);

/// The bars' left edge and full width on screen (the channels' open columns).
fn bar_span() -> (f32, f32) {
    let k = BAR_FRAME_SIZE.0 / BAR_FRAME_WIDTH as f32;
    (BAR_FRAME_AT.0 + BAR_CHANNEL_COLS.0 * k, (BAR_CHANNEL_COLS.1 - BAR_CHANNEL_COLS.0) * k)
}

/// The open interior of the frame's channel `i` (0 health, 1 stamina) on screen: top and height.
fn bar_channel(i: usize) -> (f32, f32) {
    let (top, bottom) = BAR_CHANNELS[i];
    let k = BAR_FRAME_SIZE.1 / BAR_FRAME_TEXELS_HIGH;
    (BAR_FRAME_AT.1 + top * k, (bottom - top) * k)
}

/// The original's health / stamina frame: the tutorial's picture of it (tutorial h_17f34cbe,
/// 256 x 64, the + and bolt icons, the frame with its corner tabs and divider, both bars baked
/// in). Cut in three across: the left cap with the icons, a middle that stretches, the right cap.
/// Its baked bars are cleared (TB_EMPTY) and the live bars drawn over them. Upright texel columns
/// and rows:
const TUTORIAL_BAR: u32 = 0x17F3_4CBE;
/// the picture's frame (icons to the right tab) and its pieces: the icons (drawn solid), then the
/// frame's left cap, middle and right cap (drawn as glass)
const TB_ROWS: (u32, u32) = (12, 51);
const TB_ICONS: (u32, u32) = (34, 46);
const TB_LEFT: (u32, u32) = (46, 75);
const TB_MIDDLE: (u32, u32) = (75, 185);
const TB_RIGHT: (u32, u32) = (185, 212);
/// the bars (health, stamina): rows, and the columns both span
const TB_BAR_ROWS: [(u32, u32); 2] = [(22, 28), (35, 41)];
const TB_BAR_COLS: (u32, u32) = (56, 200);
/// The bars' empty part is see-through, the channel's inside clear (#35 spec 5.1: the sky shows
/// through the empty stamina channel, take10 14 s; the dark scene through both, take07). It was
/// the picture's own empty-bar navy (0 20 107) before the takes.
const TB_EMPTY: [u8; 4] = [0, 20, 107, 0];
/// where the picture's icons and frame go on the 640 x 480 screen (their top-left, native size: its body
/// is 38 texels high, the old frame's 37)
const TB_AT: (f32, f32) = (52.0, 39.0);
/// how opaque the frame is drawn (the picture is opaque, baked on the tutorial's screen; the
/// capture's frame is glass, the sky showing through)
const TB_ALPHA: f32 = 0.45;
/// The picture row where the frame's lower half begins: the stamina widget's own frame (its
/// pieces h_1289112b / h_f5678943 / h_0565a96e, 0xb7cb0), which the low-stamina blink tints
/// (#35 spec 5.4: from y 57.3 down, take02; TB_AT puts picture row 12 at y 39, so 57.3 is row
/// ~30).
const TB_LOWER: u32 = 30;

/// The bars' left edge on the screen: the take's fill starts at x 73.0-73.3 in every take (#35
/// spec 5.1; the picture's channel starts at 74, so the bar covers its first column).
const BAR_X: f32 = 73.0;
/// The bars' length per hit point of the character's maximum health: the bar widget's length is
/// value / +0x6c (150) x +0x84 (249) units (FUN_000b7050), its value hit points for the health
/// bar and stamina / max-stamina x max health for the stamina bar (FUN_000b7fb0, 0xb817d): 1.66
/// units a hit point. Measured at full (#35 spec 5.1): Tex 190.0, Brutus 174.0, Flint 148.7,
/// Hawk 106.7 (the code's 190.9, 174.3, 149.4, 107.9).
const BAR_PER_HP: f32 = 249.0 / 150.0;
/// A bar above 0 is drawn at least this long (FUN_000b7050: 2 units; the takes' sliver at 0.4
/// stamina, take02 16.3-17.0 s).
const BAR_MIN: f32 = 2.0;

/// The bars' full length (screen units) for a maximum health.
fn bar_length(max_health: f32) -> f32 {
    max_health.max(1.0) * BAR_PER_HP
}

/// A bar's drawn length for its value (in hit points; FUN_000b7050).
fn bar_fill(value: f32) -> f32 {
    if value > 0.0 { (value * BAR_PER_HP).max(BAR_MIN) } else { 0.0 }
}

/// The throw meter's strength (0-1, its parts' alpha over their full alpha) for the time left on
/// its timer (`Player::meter_after`, METER_TIME): the meter's update (0xad950, 0xada4f-0xadaa0)
/// sets the widget's alpha to clamp(2 x timer, 0, 1)^3, so it holds full 0.5 s and fades to
/// nothing at 1.0 s, drawn in linear light as METER_GROUND says.
fn meter_strength(left: f32) -> f32 {
    let w = (2.0 * left).clamp(0.0, 1.0).powi(3);
    let lin = |v: f32| v.powf(2.2);
    (lin(METER_GROUND + w * (METER_FULL - METER_GROUND)) - lin(METER_GROUND)) / (lin(METER_FULL) - lin(METER_GROUND))
}

/// How far the frame's middle stretches and its right cap moves for a maximum health: the bars'
/// end against the picture's own bar end (column TB_BAR_COLS.1).
fn frame_stretch(max_health: f32) -> f32 {
    BAR_X + bar_length(max_health) - (TB_AT.0 + (TB_BAR_COLS.1 - TB_ICONS.0) as f32)
}

/// The low-stamina blink's red: a red copy of the frame's lower half, each texel's luminance in
/// red and this share of it in green, no blue, its own alpha. The takes: a red-phase pixel is
/// ~ (L, 0.15 L + 14, ~24) of the normal pixel's luminance L (rms 6-11 levels, take02; the
/// game's tint, bytes a0 20 20 ff at 0xb720a, combines in a way not traced). The bar channel's
/// inside stays as it is.
const LOW_GREEN: f32 = 0.15;

/// One piece of the tutorial picture, its columns (upright, all its rows TB_ROWS), the bars
/// emptied; `red`: the low-stamina blink's copy of it, its lower half (from TB_LOWER) in red
/// (LOW_GREEN), the stamina channel's inside kept. (The halves are one image, not two nodes: two
/// nodes left a seam between them.)
fn tutorial_piece(w: u32, h: u32, px: &[u8], (x0, x1): (u32, u32), red: bool) -> Image {
    let at = |x: u32, y: u32| -> [u8; 4] {
        let in_bar = TB_BAR_ROWS.iter().any(|&(a, b)| (a..b).contains(&y)) && (TB_BAR_COLS.0..TB_BAR_COLS.1).contains(&x);
        if in_bar {
            return TB_EMPTY;
        }
        let i = (((h - 1 - y) * w + x) * 4) as usize;
        let p = [px[i], px[i + 1], px[i + 2], px[i + 3]];
        if red && y >= TB_LOWER {
            let l = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            return [l.round() as u8, (l * LOW_GREEN).round() as u8, 0, p[3]];
        }
        p
    };
    let mut out = vec![];
    for y in TB_ROWS.0..TB_ROWS.1 {
        for x in x0..x1 {
            out.extend_from_slice(&at(x, y));
        }
    }
    Image::new(bevy::render::render_resource::Extent3d { width: x1 - x0, height: TB_ROWS.1 - TB_ROWS.0, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, out,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
}

/// The tutorial picture cut up (see TUTORIAL_BAR): the icons; the frame's left cap, middle and
/// right cap, and their copies with the lower half red for the blink.
struct TutorialFrame {
    icons: Handle<Image>,
    pieces: [Handle<Image>; 3],
    red: [Handle<Image>; 3],
}

/// A piece of the frame: its image and its copy with the lower half (the stamina widget's own
/// frame) red, shown while the low-stamina blink is red.
#[derive(Component)]
struct FrameBlink(Handle<Image>, Handle<Image>);

/// The health / energy frame from its corner and edge (see BAR_FRAME_CORNER).
fn bar_frame(cw: u32, ch: u32, px: &[u8]) -> Image {
    let (w, h) = (BAR_FRAME_WIDTH.max(2 * cw), 2 * ch);
    // the pieces, upright
    let corner = |x: u32, y: u32| -> [u8; 4] {
        let i = (((ch - 1 - y) * cw + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    // the cap's own blank columns at its inner end give way to the edge
    let cap = (0..cw).rev().find(|&x| (0..ch).any(|y| corner(x, y)[3] > 0)).map_or(cw, |x| x + 1);
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let cy = if y < ch { y } else { h - 1 - y };
        for x in 0..w {
            let cx = if x < cap { Some(x) } else if x >= w - cap { Some(w - 1 - x) } else { None };
            // the middle continues the cap's inner column: the cap holds the step down from its
            // raised outer end to the middle's level (the edge-like texture h_eee38815 sits
            // higher: stretched there, the frame looked inside out)
            out.extend_from_slice(&corner(cx.unwrap_or(cap - 1), cy));
        }
    }
    Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, out,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
}

/// A portrait's own image (to show again after the skull).
#[derive(Component)]
struct PortraitArt(Handle<Image>, Rect);

/// A face texture `id` as the HUD draws it: its image and its opaque box (texels with alpha over
/// 40, rows as stored). The faces are packed into a corner of their 32 x 32 textures, so each is
/// drawn cropped to that box, 1:1, centred on its place (PORTRAIT_CENTRE). Kept in `faces` by
/// texture; None when the texture isn't found or is clear.
fn face_art(game: &mut Game, images: &mut Assets<Image>, cache: &mut HudImages, faces: &mut HashMap<u32, Option<(Handle<Image>, Rect)>>,
            id: u32) -> Option<(Handle<Image>, Rect)> {
    if let Some(art) = faces.get(&id) {
        return art.clone();
    }
    let art = (|| {
        let (w, h, px) = game.texture_rgba(id)?;
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
        for (k, p) in px.chunks_exact(4).enumerate() {
            if p[3] > 40 {
                let (x, y) = (k as u32 % w, k as u32 / w);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
            }
        }
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let handle = texture(game, images, cache, id)?;
        Some((handle, Rect::new(x0 as f32, y0 as f32, x1 as f32, y1 as f32)))
    })();
    faces.insert(id, art.clone());
    art
}

/// A face texture `id` as a selected member's portrait shows it (#319, SELECTED_TINT): each texel's
/// luminance (0.3, 0.59, 0.11, as `texture_grey`) x SELECTED_GAIN in the selection's orange, its
/// alpha kept; cached (SELECTED_KEY). The same texels as the face, so `face_art`'s box fits it.
fn selected_face(game: &mut Game, images: &mut Assets<Image>, cache: &mut HudImages, id: u32) -> Option<Handle<Image>> {
    cache.0.entry(id ^ SELECTED_KEY).or_insert_with(|| {
        let (w, h, mut px) = game.texture_rgba(id)?;
        for p in px.chunks_exact_mut(4) {
            let l = 0.3 * p[0] as f32 + 0.59 * p[1] as f32 + 0.11 * p[2] as f32;
            for (k, t) in SELECTED_TINT.iter().enumerate() {
                p[k] = (l * SELECTED_GAIN * t).min(255.0) as u8;
            }
        }
        Some(images.add(Image::new(
            bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2, px,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default())))
    }).clone()
}

/// What a squad slot's portrait place shows (`update_squad_hud`): its own face (`PortraitArt`:
/// its own type); another face (#302: a squad type's in another's place, or another type's
/// portrait from its data, `CharacterType::portrait`); nothing (a slot with no member in the
/// world, `slot_member`: the members a zone doesn't have, #303, or the test map's emptied slots);
/// or a "?" (a member whose type has no portrait). A dead member's skull goes over any of them.
#[derive(Clone)]
enum Face {
    Own,
    Other(Handle<Image>, Rect),
    Empty,
    Unknown,
}

/// Squad slot `c`'s member in the world: the controlled character or one of the squad's (not an
/// NPC), alive or dead, but not one out of the world (`absent`: a member whose spawn-trigger
/// hasn't spawned it, or whose copy the session took away, FUN_00209900). None for a slot whose
/// member the zone doesn't have (#303: the level spawns only the zone's members, so the
/// tutorial's and m01_a's squad is Tex alone, the zone's h_ed54d255 "brutus hawk flint"), and for
/// the test map's emptied slots (#302). Such a slot has no face, follow tab or health channel.
/// The game's HUD keeps a squad list of five entries (+0x34), one per slot by the character's
/// type +0x150 (1-4): each frame FUN_000bd790 drops the characters no longer on the list it
/// reads (FUN_000c8570) and adds the new ones (FUN_000c84e0), so a slot holds a character only
/// once one is in the world (which list it reads, and that the portraits draw from it, aren't
/// traced). take04 (`todo/28-tutorial`, 60 s) shows Tex's face alone, no tabs or channels.
fn slot_member<'a>(player: &'a Player, squad: &'a Squad, c: usize) -> Option<&'a Player> {
    std::iter::once(player).chain(squad.0.iter()).find(|u| u.character == c && u.in_squad() && !u.absent)
}

/// Generated HUD images: the squad health bars, the selection chevrons and the speech icon.
#[derive(Resource, Default)]
struct HudGen {
    /// the health / stamina frame is the original's (TUTORIAL_BAR), its length following the
    /// character's maximum health
    tutorial_frame: bool,
    /// per character: its health bar image (frame-sized), the channel texels (index, how far
    /// from the portrait end 0-1) and the fill drawn last
    bars: [Handle<Image>; 4],
    bar_texels: [Vec<(usize, f32)>; 4],
    bar_fill: [f32; 4],
    /// the game's chevron, white, pointing right / left / up / down
    chevrons: [Handle<Image>; 4],
    /// speech icon
    bubble: Handle<Image>,
    /// skulls for the dead, by character
    skulls: [Handle<Image>; 4],
    /// the faces drawn so far, by texture (`face_art`): the squad's four, and on the test map any
    /// type's portrait a squad slot's member shows (#302)
    faces: HashMap<u32, Option<(Handle<Image>, Rect)>>,
    /// the stamina bar's fill (ENERGY_FILL), remade mixed toward white by its brighten, and the
    /// texture's own texels it's made from
    stamina_fill: Handle<Image>,
    stamina_texels: Vec<u8>,
    /// the health bar's fill (HEALTH_FILL) and its copy in the "full" flash's colour
    /// (`flash_texels`, `BarFlash`), swapped in while the flash is tinted
    health_fill: Option<Handle<Image>>,
    health_flash: Option<Handle<Image>>,
}

/// The radar frame's four diagonal health channels, by character (see HEALTH_BARS): texel index
/// (display rows) and its place along the channel from the portrait end (0-1). A channel is the
/// frame's dark texels in that quadrant (pieces of 15 or more), with the 1-texel highlight lines
/// between dark texels closed over.
fn radar_channels(w: usize, h: usize, px: &[u8]) -> [Vec<(usize, f32)>; 4] {
    // display rows: the texture is stored bottom row first
    let at = |x: usize, y: usize| &px[((h - 1 - y) * w + x) * 4..((h - 1 - y) * w + x) * 4 + 4];
    let dark = |x: usize, y: usize| { let p = at(x, y); p[3] > 128 && (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 <= HEALTH_DARK };
    let mut mask = vec![false; w * h];
    let mut seen = vec![false; w * h];
    for start in 0..w * h {
        if seen[start] || !dark(start % w, start / w) {
            continue;
        }
        // flood fill one dark piece; keep it if it's big enough
        let (mut piece, mut stack) = (vec![], vec![start]);
        seen[start] = true;
        while let Some(i) = stack.pop() {
            piece.push(i);
            let (x, y) = (i % w, i / w);
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if !seen[j] && dark(nx as usize, ny as usize) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        if piece.len() >= 15 {
            piece.into_iter().for_each(|i| mask[i] = true);
        }
    }
    // close 1-texel gaps (the channel's inner highlight line): dark on both sides
    let closed: Vec<bool> = (0..w * h).map(|i| {
        let (x, y) = (i % w, i / w);
        let m = |dx: i32, dy: i32| {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 && mask[ny as usize * w + nx as usize]
        };
        mask[i] || (at(x, y)[3] > 128 && ((m(-1, 0) && m(1, 0)) || (m(0, -1) && m(0, 1))))
    }).collect();
    HEALTH_BARS.map(|((right, down), (ax, ay))| {
        // the channel is a band across its quadrant's diagonal: from the dark texels found, its
        // distance from the centre (s, the middle 80% of them) and its extent along it (t); every
        // texel in that band belongs to it, also the lighter ones the darkness test misses (the
        // channel's highlight lines left gaps: Hawk's and Brutus's bars)
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let (sx, sy) = (if right { 1.0 } else { -1.0 }, if down { 1.0 } else { -1.0 });
        let st = |i: usize| {
            let (x, y) = ((i % w) as f32 + 0.5 - cx, (i / w) as f32 + 0.5 - cy);
            ((x * sx + y * sy) * std::f32::consts::FRAC_1_SQRT_2, (x * sx - y * sy) * std::f32::consts::FRAC_1_SQRT_2)
        };
        let quadrant = |i: usize| ((i % w) * 2 >= w) == right && ((i / w) * 2 >= h) == down;
        let found: Vec<usize> = (0..w * h).filter(|&i| closed[i] && quadrant(i)).collect();
        let mut ss: Vec<f32> = found.iter().map(|&i| st(i).0).collect();
        ss.sort_by(f32::total_cmp);
        let pick = |q: f32| ss.get(((ss.len() as f32 - 1.0) * q).round() as usize).copied().unwrap_or(0.0);
        let (s_lo, s_hi) = (pick(0.1), pick(0.9));
        let (t_lo, t_hi) = found.iter().map(|&i| st(i).1).fold((f32::MAX, f32::MIN), |(a, b), t| (a.min(t), b.max(t)));
        let texels: Vec<(usize, f32)> = (0..w * h)
            .filter(|&i| quadrant(i) && (closed[i] || {
                let (s, t) = st(i);
                at(i % w, i / w)[3] > 128 && (s_lo..=s_hi).contains(&s) && (t_lo..=t_hi).contains(&t)
            }))
            .map(|i| (i, ((i % w) as f32 + 0.5 - ax).hypot((i / w) as f32 + 0.5 - ay))).collect();
        let (lo, hi) = texels.iter().fold((f32::MAX, f32::MIN), |(lo, hi), t| (lo.min(t.1), hi.max(t.1)));
        texels.into_iter().map(|(i, d)| (i, ((d - lo) / (hi - lo).max(1.0)).max(1e-3))).collect()
    })
}

const ENERGY_FILL: u32 = 0xE11C_7D04;

/// The stamina bar widget (the HUD's bar widget, base class FUN_000b6cc0, stamina subclass
/// FUN_000b7bf0; #35 spec 5.2-5.4). The fill brightens toward white when the value changes:
/// set to 0.5 on a rise (+0xb8) and 1.0 on a fall (+0xbc) by FUN_000b7850, fading by 0.5 a
/// second (FUN_000b75a0). So it's white while a skill drains it and pale while it refills (take01
/// `f163`, `f1124`), and pale fades to teal in 1 s once full (take25 11.53-12.48 s).
const BRIGHTEN_RISE: f32 = 0.5;
const BRIGHTEN_FALL: f32 = 1.0;
const BRIGHTEN_FADE: f32 = 0.5;
/// The bar's shown value (the trail's end) follows the value up at +0x70 8 and down at +0x74 12
/// bar units a second (FUN_000b75a0), 12 x 1.66 = 19.9 screen units a second for everyone (take03
/// after the poke: 19.7 a second), after a delay of +0x64 / +0x60 (0 for stamina: FUN_000b7bf0;
/// the health bar's fall delay is 1.0).
const TRAIL_UP: f32 = 8.0;
const TRAIL_DOWN: f32 = 12.0;
/// The low-stamina blink (FUN_000b76c0): while the value is at most +0x68 0.15 of the max
/// (FUN_000b7bf0) a triangle wave +0x40 runs between -0.5 and +0.5 of +0xac (1.0) at 1 a second,
/// reset to 0 heading down whenever the value is above it; the frame's lower half is tinted
/// while it's below 0 (the draw at 0xb71d0). So 1.0 s red, 1.0 s normal, starting red, stopping at
/// once above 15% (take02, take03, take07, take11, take17: red 0.98-1.08 s, gaps 1.00-1.07 s).
const LOW_STAMINA: f32 = 0.15;
const BLINK_SPAN: f32 = 1.0;
/// The bars' "full" flash (#15): a power-up the character can't take (its item set's "can take",
/// 0x148d40: HEALTH POWER at full health, STAMINA POWER at full stamina) starts the flash of the
/// bar it would fill, on the HUD of the player whose character it is (0x6ef00), through 0x146900:
/// when the bar widget's timer +0x48 is 0 it's set to [0x40c8fc] 4 x +0xb0 0.3 + +0xb4 5.0 = 6.2
/// s (FUN_000b6cc0 sets +0xb0 and +0xb4 for both bars). The bar's update counts it down to 0
/// (FUN_000b76c0); while it's above +0xb4 the fill is drawn in +0x59's colour (0xff3caae6, both
/// bars: FUN_000b6cc0, FUN_000b7bf0) on the half of each +0xb0 period FUN_000b73d0 picks
/// (`flash_tinted`), so four 0.15 s flashes over 1.2 s, then 5 s before a walk-over can start
/// another. #35 take20 (STAMINA POWER walked over at full stamina): the stamina fill amber at
/// 3.50-3.60, 3.80-3.90, 4.10-4.25 and 4.45-4.55 s, 0.3 s apart.
const FLASH_START: f32 = 4.0 * 0.3 + 5.0;
const FLASH_QUIET: f32 = 5.0;
const FLASH_PERIOD: f32 = 0.3;
/// The flash's colour, the bytes of 0xff3caae6 as the HUD's draw takes them (R e6, G aa, B 3c: the
/// low-stamina blink's a0 20 20 reads as red the same way), and how it tints the fill: each texel's
/// luminance x the colour x FLASH_GAIN. A fit to take20: the amber fill against the teal one in the
/// next frames (3.50 against 3.65 s), 1.64 over 600 texels (rms 15, 6, 19 levels), the same
/// combine the low-stamina blink's red copy fits (LOW_GREEN: L x (160, 32, 32) / 255 x ~1.6).
/// The game's colour operation isn't traced.
const FLASH_TINT: [f32; 3] = [230.0 / 255.0, 170.0 / 255.0, 60.0 / 255.0];
const FLASH_GAIN: f32 = 1.64;

/// The bars' "full" flash timers (the bar widgets' +0x48, see FLASH_START): the health bar's and
/// the stamina bar's, s, 0 when not flashing. play_powerups.rs starts them.
#[derive(Resource, Default)]
pub(crate) struct BarFlash {
    pub health: f32,
    pub stamina: f32,
}

impl BarFlash {
    /// 0x146900 on a bar's timer: a flash starts only when none runs.
    pub fn start(timer: &mut f32) {
        if *timer == 0.0 {
            *timer = FLASH_START;
        }
    }
}

/// Whether a bar whose flash timer is `t` draws its fill in the flash's colour (FUN_000b73d0):
/// above FLASH_QUIET, the part past it taken down by FLASH_PERIOD into (0, period], tinted when
/// that's at least half a period below the period (|f - period| >= period / 2): the second half
/// of each period as the timer counts down, so the first flash comes 0.15 s after the walk-over.
fn flash_tinted(t: f32) -> bool {
    if t <= FLASH_QUIET {
        return false;
    }
    let mut f = t - FLASH_QUIET;
    while f > FLASH_PERIOD {
        f -= FLASH_PERIOD;
    }
    (f - FLASH_PERIOD).abs() >= FLASH_PERIOD * 0.5
}

/// A fill's texels in the flash's colour (FLASH_TINT): each texel's luminance x the colour x
/// FLASH_GAIN, its alpha kept.
fn flash_texels(px: &[u8]) -> Vec<u8> {
    px.chunks_exact(4).flat_map(|c| {
        let l = 0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32;
        let t = |k: usize| (l * FLASH_TINT[k] * FLASH_GAIN).round().clamp(0.0, 255.0) as u8;
        [t(0), t(1), t(2), c[3]]
    }).collect()
}

/// The skill's name under the bars (the HUD message widget h_1a5ceeb9, FUN_000c72b0 at hud+0xfc;
/// text FUN_000c75c0, its font h_0c626fd5 at size 14), in the message blue (the takes' letter
/// cores 189-197 206-212 242-249): the left edge, the line's top and the capital height, fitted
/// so "Berserker"'s bright letter cores span the take's box, x 56.7-130.0 and y 90.0-103.3
/// (take01 3.5 s, take03 7.93 s; the demo's font is the body atlas, a little narrower and
/// taller: at the switch hint's size it spanned 55.3-126.7, and now spans x 56.7-130.0, y
/// 89.6-103.6).
const LABEL_X: f32 = 56.9;
const LABEL_TOP: f32 = 85.5;
const LABEL_CAP: f32 = 13.4;
/// The label moves this much lower for each other message widget showing (0xc73b4-0xc73cb: 20
/// units full screen): the player view's two hint lines (+0x3c, +0x40: play_hints.rs, #224).
/// The switch hint, which sits in the first one's place, counts as it: **a guess** (the takes
/// never show the two together).
const LABEL_STACK: f32 = 20.0;
/// After the off the label stays at full for LABEL_HOLD, then fades at LABEL_FADE a second (the
/// widget's alpha, 3.0 at 0x39a848, 0xc7473). The hold is measured, 0.13-0.18 s after the bar
/// turns pale (takes 01, 02, 10, 15, 16, 17); what holds it in the code isn't traced.
const LABEL_HOLD: f32 = 0.15;
const LABEL_FADE: f32 = 3.0;

/// The stamina bar widget's state (see BRIGHTEN_RISE ...), its fill's brighten as last drawn,
/// and the skill's name label. Control passing resets it to the new character's at once, no
/// trail or brighten carried over and the label shown or hidden (FUN_000b7940; take19 4.867 s).
#[derive(Resource, Default)]
pub(crate) struct StaminaBar {
    /// the character it shows: its squad slot and type (a type change on the test map is a new one, #302)
    who: Option<(usize, u32)>,
    /// the value (+0x34) and the shown value (+0x38, the trail's end), in hit points
    value: f32,
    shown: f32,
    /// the brighten (+0x4c, 0..1) and the one the fill image was last made with
    brighten: f32,
    drawn: f32,
    /// the blink's wave (+0x40) and its way (+0x44)
    blink: f32,
    blink_dir: f32,
    /// the fill image was last made in the "full" flash's colour (`BarFlash`)
    drawn_flash: bool,
    /// the label: the skill's name (a string id), its opacity and the hold left after the off
    label: u32,
    label_alpha: f32,
    label_hold: f32,
}

impl StaminaBar {
    /// Whether the skill's name shows (the HUD's message widget hud+0xfc, which the use prompt
    /// stands 20 lower for: play_hints.rs).
    pub(crate) fn label_shown(&self) -> bool {
        self.label_alpha > 0.0 && self.label != 0
    }
}

/// weapon rows: the first at y 43, the next 45 below
const WEAPON_ROWS: usize = 3;
/// A weapon's name over its count (after a pickup, and in the list after a switch): its left
/// edge, how far above the panel's top its line starts, and its size. todo/49 take04 7.5 s:
/// "Bower 20"'s B spans x 465.3, y 43.0-56.8 (13.8 units; with the old size 11 ours was 10.3
/// tall, so the size is scaled by 1.34), the panel's top at 43.
const WEAPON_NAME_X: f32 = 465.0;
const WEAPON_NAME_UP: f32 = 5.7;
const WEAPON_NAME_CAP: f32 = 14.7;
/// How opaque the weapon's icon is under its name after a pickup (todo/49 take04 7.0-9.5 s: a
/// faint outline; the level is judged by eye, a guess).
const ICON_UNDER_NAME: f32 = 0.2;
const PALE: Color = Color::srgb(0.78, 0.85, 1.0);
const HUD_BLUE: Color = Color::srgb(0.24, 0.52, 1.0);

#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    Panel(usize),
    Icon(usize),
    Name(usize),
    Ammo(usize),
    Crosshair,
    /// "Q switch to <other weapon>." while the held gun and its reserve are empty
    SwitchHint,
    /// the special skill's name while it's on (and a moment after), under the bars
    SkillLabel,
    /// the pickup lines under the message line (`MessageLine`)
    Feed(usize),
    /// the blue target ring on a usable panel's button
    Ring,
}

/// The message line at the character ("No need to heal", "Inventory is empty", "Tex cannot pick
/// up Medkit."): a record text in the game's font (MESSAGE_RGB).
#[derive(Component)]
struct MessageLine;

/// A portrait's command tab (TAB_FOLLOW ...: `update_tabs`), by character: its icon, or the
/// flash's growing copy of it drawn under the icon (`copy`, TAB_FLASH_RATE).
#[derive(Component, Clone, Copy, PartialEq)]
struct CommandTab {
    slot: usize,
    copy: bool,
}

/// The tab widgets' state (`update_tabs`), by character: the icon each shows (the widget's texture
/// +0x20, with its size; None before its first, as FUN_000ba5b0 makes it) and its flash's share
/// done (+0x48) while one runs (+0x4c). A resource, reset with each level.
#[derive(Resource, Default)]
struct Tabs([(Option<(u32, f32, f32)>, Option<f32>); 4]);

/// The tab's icon for a member in command mode `mode` (TAB_FOLLOW ...): the texture and its size
/// (width, height), None where the tab is hidden.
fn tab_icon(mode: super::brain::Mode) -> Option<(u32, f32, f32)> {
    use super::brain::Mode;
    match mode {
        Mode::Follow => Some(TAB_FOLLOW),
        Mode::Guard => Some(TAB_GUARD),
        Mode::Wingman => Some(TAB_WINGMAN),
        Mode::MoveOut => Some(TAB_MOVE_OUT),
        Mode::Formation | Mode::Freewill => None,
    }
}

/// One frame of a tab's state (0xc3f80's switch and FUN_000c4100): a new icon starts the flash
/// from 0 (the first one too), which then runs TAB_FLASH_RATE a second on the HUD's clock (`dt`)
/// and ends past 1. `icon` None: no member in the slot (the update returns before the switch):
/// the state is kept and nothing runs. (A member in a mode without a tab would still run its
/// flash in the game, 0xc40b3 going on to FUN_000c4100: the squad never is in one here.)
fn tab_step(state: &mut (Option<(u32, f32, f32)>, Option<f32>), icon: Option<(u32, f32, f32)>, dt: f32) {
    let Some(icon) = icon else { return };
    if state.0 != Some(icon) {
        *state = (Some(icon), Some(0.0));
    }
    if let Some(t) = state.1.as_mut() {
        *t += TAB_FLASH_RATE * dt;
        if *t > 1.0 {
            state.1 = None;
        }
    }
}

/// The flash's copy at its share done `t` (0xc4130): its scale s, its offset from the tab's place
/// (each way, x the scale as the sprite draw takes it) and its alpha (0-1).
fn tab_flash(t: f32) -> (f32, f32, f32) {
    let s = 1.0 + TAB_FLASH_GROW * t;
    let offset = TAB_FLASH_SHIFT * (s - 1.0) * s;
    let alpha = (2.0 * (1.0 - t) * (1.0 - t)).clamp(0.0, 1.0);
    (s, offset, alpha)
}

/// The HUD's clock step for the tabs (s): the HUD's widgets are updated through the Squad Command
/// menu's and the item list's pauses (their fades and slides: play_squad_menu.rs and
/// play_itembox.rs run on the same clock; take03's bars flash as the menu fades out), not while
/// another menu stops the game (the HUD isn't updated under it); the capture's fixed step while
/// capturing (none in its warm-up).
fn hud_step(time: &Time) -> f32 {
    if paused_by_other(PAUSE_SQUAD | PAUSE_ITEMS) {
        return 0.0;
    }
    if std::env::var("BF_CAPTURE").is_ok() {
        if WARMUP.load(std::sync::atomic::Ordering::Relaxed) > 0 { 0.0 } else { capture_step() }
    } else {
        time.delta_secs().min(0.1)
    }
}

/// The command tabs each frame (the tab widget's update 0xc3f80 and draw 0xc4130, TAB_FOLLOW ...):
/// for each slot's member in the world (`slot_member`: the controlled character's too, take03's
/// Tex), its icon by its brain's command mode (`tab_icon`), at TAB_AT from the radar panel's
/// corner (RADAR_FRAME_AT; the panel's slide moves it with the radar) at its own size, hidden
/// while the member is dead (its +0x70 bit 0, 0xc40bb) or has no tab in its mode; and its flash's
/// copy under it (`tab_flash`). The icons drawn whole, as the base draw's 255 (0xbb300); the copy's alpha raised
/// to 2.2, as the HUD's other fades are here: the console blends its stored sRGB values, Bevy in
/// linear light. BF_ORDER_LOG (play_orders.rs) prints each tab's change of icon too.
#[allow(clippy::too_many_arguments)]
fn update_tabs(time: Res<Time>, player: Res<Player>, squad: Res<Squad>, mut tabs: ResMut<Tabs>, mut game: ResMut<GameData>,
               mut images: ResMut<Assets<Image>>, mut cache: ResMut<HudImages>,
               mut nodes: Query<(&CommandTab, &mut Visibility, &mut Node, &mut ImageNode)>) {
    let dt = hud_step(&time);
    let log = std::env::var("BF_ORDER_LOG").is_ok();
    for c in 0..tabs.0.len() {
        let member = slot_member(&player, &squad, c);
        let icon = member.and_then(|u| tab_icon(u.command.mode));
        let was = tabs.0[c].0;
        tab_step(&mut tabs.0[c], icon, dt);
        if log && tabs.0[c].0 != was {
            println!("t={:.2} hud: {}'s command tab: h_{:08x} ({}), flashing", player.sim_time, CHARACTERS[c],
                     tabs.0[c].0.map_or(0, |i| i.0), member.map_or("none", |u| u.command.mode.name()));
        }
    }
    for (tab, mut vis, mut node, mut img) in &mut nodes {
        let c = tab.slot;
        let Some(&(icon, flash)) = tabs.0.get(c) else { continue };
        let member = slot_member(&player, &squad, c);
        // (shown for a member in the world, alive, in a mode with a tab; its copy while flashing)
        let shown = member.is_some_and(|u| !u.dead && tab_icon(u.command.mode).is_some());
        let (x, y) = (RADAR_FRAME_AT.0 + TAB_AT[c].0, RADAR_FRAME_AT.1 + TAB_AT[c].1);
        let (place, want) = match (icon, shown, tab.copy, flash) {
            (Some((id, w, h)), true, false, _) => ((x, y, w, h, 1.0), Some(id)),
            (Some((id, w, h)), true, true, Some(t)) => {
                let (s, off, a) = tab_flash(t);
                ((x + off, y + off, w * s, h * s, a.powf(2.2)), Some(id))
            }
            _ => ((0.0, 0.0, 0.0, 0.0, 0.0), None),
        };
        let handle = want.and_then(|id| texture(&mut game.0, &mut images, &mut cache, id));
        let on = handle.is_some();
        let v = if on { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != v {
            *vis = v;
        }
        let Some(handle) = handle else { continue };
        if img.image != handle {
            img.image = handle;
        }
        let colour = Color::srgba(1.0, 1.0, 1.0, place.4);
        if img.color != colour {
            img.color = colour;
        }
        let at = at_box(place.0, place.1, place.2, place.3);
        if node.left != at.left || node.top != at.top || node.width != at.width || node.height != at.height {
            (node.left, node.top, node.width, node.height) = (at.left, at.top, at.width, at.height);
        }
    }
}

/// Squad and grenade widgets (see update_squad_hud).
#[derive(Component, Clone, Copy, PartialEq)]
enum SquadPart {
    /// squad member's radar blip (by squad slot)
    Blip(usize),
    /// a squad member's health bar in its radar diagonal channel (by character)
    Health(usize),
    /// the controlled character's health (top-left red bar)
    PlayerHealth,
    /// the stamina bar and its trail (see `StaminaBar`), and the frame's stretching middle and
    /// its right cap (the tutorial picture's frame: they follow the character's maximum health,
    /// see `bar_length`)
    PlayerStamina,
    StaminaTrail,
    FrameMiddle,
    FrameRight,
    /// the selection's three chevrons on the radar
    Chevron(usize),
    /// speech icon beside a member's portrait while they talk (by character)
    Bubble(usize),
    /// portrait (by squad slot: its member's face, `Face`): its skull when dead
    Portrait(usize),
    /// the "?" in a slot's place whose member's type has no portrait (UNKNOWN_CAP, #302)
    Unknown(usize),
    /// the bracket round a selected member's portrait (by slot: SELECT_FRAMES, #319)
    Selected(usize),
    MeterBox,
    MeterFill,
    MeterTick,
}

/// The HUD's textures as UI images, by name (`texture`; a grey one under the name's complement,
/// `texture_grey`).
#[derive(Resource, Default)]
pub(super) struct HudImages(HashMap<u32, Option<Handle<Image>>>);

/// The squad switch's name label (LABEL_SHOW): the selection it was last shown for (the
/// character being selected last frame, to tell a new selection), whose name it shows and its
/// timer. A resource, reset with each level, so a label doesn't carry into the next one.
#[derive(Resource, Default)]
struct NameLabel {
    selecting: Option<usize>,
    character: usize,
    timer: f32,
}

/// The name label's record text (`NameLabel`).
#[derive(Component)]
struct NameLabelText;

pub fn plugin(app: &mut App) {
    app.init_resource::<HudImages>()
        .init_resource::<NameLabel>()
        .init_resource::<StaminaBar>()
        .init_resource::<BarFlash>()
        .init_resource::<RadarMarks>()
        .init_resource::<Tabs>()
        .add_systems(OnEnter(AppState::Playing), (setup_hud.after(snapshot_entities).in_set(HudSetup),
            (|mut commands: Commands| {
                commands.insert_resource(NameLabel::default());
                commands.insert_resource(StaminaBar::default());
                commands.insert_resource(BarFlash::default());
                commands.insert_resource(Tabs::default());
            }).after(setup)))
        .init_resource::<HudGen>()
        // (after the death camera: its label and reticle show the frame it decides them; the
        // tabs after the frame's orders, the menu's and the AI's: `update_player` sets a Move To's
        // arrival)
        .add_systems(Update, (update_stamina_bar.before(update_hud_widgets).before(update_squad_hud),
                              update_hud_widgets, update_squad_hud, update_message, update_tabs.after(update_player))
            .after(update_weapons).after(deathcam::death_cam)
            .run_if(in_state(AppState::Playing)))
        // the radar's objective dots, once the radar-triggers' contacts are in (#228)
        .add_systems(Update, update_radar_marks.after(super::triggers::radar::RadarApply).after(update_player).run_if(in_state(AppState::Playing)));
}

/// The HUD's set-up as a map starts: what draws over the HUD (Flint's targeting marker,
/// play_targeting.rs) is spawned after it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HudSetup;

/// The node the whole HUD hangs under (the window, with the game's 4:3 screen in it): the map
/// editor hides it in edit mode, where there's no character to show (play_editor.rs, #191).
#[derive(Component)]
pub(super) struct HudRoot;

/// The radar panel: the radar, its portraits and what's drawn on them (the game's one widget,
/// 0xc2d40), a node the size of the screen that slides with the Squad Command menu (RADAR_SLIDE).
#[derive(Component)]
struct RadarPanel;

/// A game texture as a UI image (cached; None if the texture isn't found).
pub(super) fn texture(game: &mut Game, images: &mut Assets<Image>, cache: &mut HudImages, name: u32) -> Option<Handle<Image>> {
    cache.0.entry(name).or_insert_with(|| {
        let (w, h, px) = game.texture_rgba(name)?;
        Some(images.add(Image::new(
            bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2, px,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default())))
    }).clone()
}

/// A game texture in grey (its luminance; cached under the name's complement): an item icon
/// while the item can't be used (capture: the medkit's at full health; #196 take10 2.55 s, its
/// saturation 0.04 against 0.38 usable; the icon's update FUN_000ace10 flags it, +0x13 bit 3).
pub(super) fn texture_grey(game: &mut Game, images: &mut Assets<Image>, cache: &mut HudImages, name: u32) -> Option<Handle<Image>> {
    cache.0.entry(!name).or_insert_with(|| {
        let (w, h, mut px) = game.texture_rgba(name)?;
        for p in px.chunks_exact_mut(4) {
            let l = (0.3 * p[0] as f32 + 0.59 * p[1] as f32 + 0.11 * p[2] as f32) as u8;
            (p[0], p[1], p[2]) = (l, l, l);
        }
        Some(images.add(Image::new(
            bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2, px,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default())))
    }).clone()
}

/// Game textures are stored bottom row first: draw them flipped.
pub(super) fn flipped(h: Handle<Image>) -> ImageNode {
    ImageNode { flip_y: true, ..ImageNode::new(h) }
}

/// Absolute placement in 640 x 480 screen units (as `at`, for moving nodes).
fn at_box(x: f32, y: f32, w: f32, h: f32) -> Node {
    at(x, y, w, h)
}

/// Absolute placement in 640 x 480 screen units.
pub(super) fn at(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(x / 6.4), top: Val::Percent(y / 4.8),
        width: Val::Percent(w / 6.4), height: Val::Percent(h / 4.8),
        ..default()
    }
}


/// The radar's view cone and the lit circle round the player, white with soft edges (the radar
/// turns with the camera, so the cone always points up).
fn radar_cone() -> Image {
    let n = 128u32;
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    let smooth = |e0: f32, e1: f32, x: f32| { let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 + 0.5) / n as f32 * 2.0 - 1.0, (y as f32 + 0.5) / n as f32 * 2.0 - 1.0);
            let r = (dx * dx + dy * dy).sqrt();
            let angle = dx.atan2(-dy).abs();
            let circle = 1.0 - smooth(0.36, 0.40, r);
            let cone = (1.0 - smooth(0.50, 0.56, angle)) * (1.0 - smooth(0.92, 0.98, r));
            let a = (circle.max(cone) * 0.18 * 255.0) as u8;
            px.extend_from_slice(&[255, 255, 255, a]);
        }
    }
    Image::new(bevy::render::render_resource::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, px,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
               bevy::asset::RenderAssetUsages::default())
}

fn setup_hud(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, mut cache: ResMut<HudImages>,
             mut gen: ResMut<HudGen>, mode: Res<Mode>) {
    // (no radar overlay until the radar is made below)
    commands.insert_resource(RadarMarks::default());
    if std::env::var("BF_NO_HUD").is_ok() {
        return;
    }
    let cone = images.add(radar_cone());
    // the chevron texture made white (its brightness kept for the tint) and turned four ways
    if let Some((w, h, px)) = game.0.texture_rgba(CHEVRON) {
        let at = |x: u32, y: u32| { let i = ((y * w + x) * 4) as usize; let l = px[i].max(px[i + 1]).max(px[i + 2]); [l, l, l, px[i + 3]] };
        let n = w.min(h);
        let make = |f: &dyn Fn(u32, u32) -> (u32, u32)| {
            let mut out = Vec::with_capacity((n * n * 4) as usize);
            for y in 0..n { for x in 0..n { let (sx, sy) = f(x, y); out.extend_from_slice(&at(sx, sy)); } }
            Image::new(bevy::render::render_resource::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
                       bevy::render::render_resource::TextureDimension::D2, out,
                       bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                       bevy::asset::RenderAssetUsages::default())
        };
        let m = n - 1;
        // the texture points left: right = mirrored, up / down = turned
        gen.chevrons = [images.add(make(&|x, y| (m - x, y))), images.add(make(&|x, y| (x, y))),
                        images.add(make(&|x, y| (y, x))), images.add(make(&|x, y| (m - y, x)))];
    }
    gen.bubble = texture(&mut game.0, &mut images, &mut cache, SQUAD_ICON).unwrap_or_default();
    gen.skulls = [0, 1, 2, 3].map(|c| {
        let id = game.0.character_icons.get(CHARACTERS[c]).map(|i| i.1).filter(|&i| i != 0)
            .unwrap_or(if c == 0 { SKULL_BRUTUS } else { SKULL_HUMAN });
        texture(&mut game.0, &mut images, &mut cache, id).unwrap_or_default()
    });
    if let Some((w, h, px)) = game.0.texture_rgba(RADAR_FRAME) {
        gen.bar_texels = radar_channels(w as usize, h as usize, &px);
        let n = (w * h) as usize;
        gen.bars = [0, 1, 2, 3].map(|_| images.add(Image::new(
            bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2, [255, 255, 255, 0].repeat(n),
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default())));
        gen.bar_fill = [-1.0; 4];
        info!("radar health channels: {:?} texels", gen.bar_texels.iter().map(|t| t.len()).collect::<Vec<_>>());
    }
    let (chevrons, bars) = (gen.chevrons.clone(), gen.bars.clone());
    let game = &mut game.0;
    game.load_textures_for(&super::data_dir(), "tutorial", TUTORIAL_BAR);
    let tutorial = game.texture_rgba(TUTORIAL_BAR).map(|(w, h, px)| {
        let mut piece = |cols: (u32, u32), red: bool| images.add(tutorial_piece(w, h, &px, cols, red));
        TutorialFrame {
            icons: piece(TB_ICONS, false),
            pieces: [TB_LEFT, TB_MIDDLE, TB_RIGHT].map(|c| piece(c, false)),
            red: [TB_LEFT, TB_MIDDLE, TB_RIGHT].map(|c| piece(c, true)),
        }
    });
    // the stamina bar's own fill image, remade as it brightens (see `StaminaBar`)
    if let Some((w, h, px)) = game.texture_rgba(ENERGY_FILL) {
        gen.stamina_fill = images.add(Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                                                 bevy::render::render_resource::TextureDimension::D2, px.clone(),
                                                 bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                                 bevy::asset::RenderAssetUsages::default()));
        gen.stamina_texels = px;
    }
    // the health bar's fill in the "full" flash's colour (see `BarFlash`)
    gen.health_flash = game.texture_rgba(HEALTH_FILL).map(|(w, h, px)| {
        images.add(Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                              bevy::render::render_resource::TextureDimension::D2, flash_texels(&px),
                              bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                              bevy::asset::RenderAssetUsages::default()))
    });
    let stamina_fill = gen.stamina_fill.clone();
    let bar_frame = game.texture_rgba(BAR_FRAME_CORNER).map(|(w, h, px)| images.add(bar_frame(w, h, &px)));
    let mut tex = |name: u32| texture(game, &mut images, &mut cache, name);
    let (frame, disc, panel, cross, bolt) =
        (tex(RADAR_FRAME), tex(RADAR_DISC), tex(WEAPON_PANEL), tex(CROSSHAIR_TEX), tex(ENERGY_ICON));
    let image = |h: &Option<Handle<Image>>, color: Color| flipped(h.clone().unwrap_or_default()).with_color(color);

    // full window, with the game's 4:3 screen centred in it
    let window = commands.spawn((HudRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                                 justify_content: JustifyContent::Center, ..default() })).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();
    // (the scope's frame and Flint's overlay, under the rest of the HUD: a pass over the picture,
    // play_scope_view.rs)

    // ---- health (red) and energy (blue), top left: in the game's glossy frame ----
    let (red, blue, plus) = (tex(HEALTH_FILL), tex(ENERGY_FILL), tex(HEALTH_ICON));
    gen.health_fill = red.clone();
    let (x0, y0, x1, y1) = BAR_FILL_RECT;
    let fill = |h: &Option<Handle<Image>>| ImageNode { rect: Some(Rect::new(x0, y0, x1, y1)), ..image(h, Color::WHITE) };
    gen.tutorial_frame = tutorial.is_some();
    if let Some(t) = tutorial {
        // the original's frame (see TUTORIAL_BAR): left cap with the icons, the middle stretched
        // and the right cap placed for the character (update_squad_hud), the bars over it; each
        // piece has a copy with its lower half red for the low-stamina blink
        let glass = Color::srgba(1.0, 1.0, 1.0, TB_ALPHA);
        let piece = |(a, b): (u32, u32)| (b - a) as f32;
        let (top, high) = (TB_AT.1, (TB_ROWS.1 - TB_ROWS.0) as f32);
        let left_x = TB_AT.0 + piece(TB_ICONS);
        let mid_x = left_x + piece(TB_LEFT);
        commands.spawn((ChildOf(screen), at(TB_AT.0, top, piece(TB_ICONS), high), ImageNode::new(t.icons)));
        for (k, (cols, x)) in [(TB_LEFT, left_x), (TB_MIDDLE, mid_x), (TB_RIGHT, mid_x + piece(TB_MIDDLE))].into_iter().enumerate() {
            let node = commands.spawn((ChildOf(screen), at(x, top, piece(cols), high), ImageNode::new(t.pieces[k].clone()).with_color(glass),
                                       FrameBlink(t.pieces[k].clone(), t.red[k].clone()))).id();
            if let Some(p) = [None, Some(SquadPart::FrameMiddle), Some(SquadPart::FrameRight)][k] {
                commands.entity(node).insert(p);
            }
        }
        let row = |i: usize| (TB_AT.1 + (TB_BAR_ROWS[i].0 - TB_ROWS.0) as f32, (TB_BAR_ROWS[i].1 - TB_BAR_ROWS[i].0) as f32);
        let ((hy, hh), (sy, sh)) = (row(0), row(1));
        let full = piece(TB_BAR_COLS);
        commands.spawn((ChildOf(screen), at(BAR_X, hy, full, hh), fill(&red), SquadPart::PlayerHealth));
        // the stamina bar's trail (the lost part, in the plain texture) under its fill
        commands.spawn((ChildOf(screen), at(BAR_X, sy, 0.0, sh), fill(&blue), SquadPart::StaminaTrail, Visibility::Hidden));
        commands.spawn((ChildOf(screen), at(BAR_X, sy, full, sh), fill(&Some(stamina_fill.clone())), SquadPart::PlayerStamina));
    } else {
    match bar_frame {
        Some(f) => {
            // see-through glass, as in the capture: a faint blue tint inside, the frame over it
            commands.spawn((ChildOf(screen), at(67.0, 42.0, 202.0, 33.0), BackgroundColor(Color::srgba(0.15, 0.3, 0.7, 0.18))));
            commands.spawn((ChildOf(screen), at(BAR_FRAME_AT.0, BAR_FRAME_AT.1, BAR_FRAME_SIZE.0, BAR_FRAME_SIZE.1),
                            ImageNode::new(f).with_color(Color::srgba(1.0, 1.0, 1.0, BAR_FRAME_ALPHA))));
        }
        None => {
            commands.spawn((ChildOf(screen), BackgroundColor(Color::srgba(0.08, 0.18, 0.45, 0.45)), BorderColor(HUD_BLUE),
                            Node { border: UiRect::all(Val::Px(1.5)), ..at(64.0, 40.0, 208.0, 37.0) }));
        }
    }
    // (without the tutorial's picture: the frame built from its corner, a fixed length)
    // each bar fills its channel's open interior, its icon level with it
    let ((health_y, health_h), (stamina_y, stamina_h)) = (bar_channel(0), bar_channel(1));
    let (bar_x, bar_w) = bar_span();
    commands.spawn((ChildOf(screen), at(bar_x, health_y, bar_w, health_h), fill(&red), SquadPart::PlayerHealth));
    commands.spawn((ChildOf(screen), at(bar_x, stamina_y, bar_w, stamina_h), fill(&blue)));
    commands.spawn((ChildOf(screen), at(49.0, health_y + health_h * 0.5 - 6.5, 13.0, 13.0), image(&plus, Color::WHITE)));
    commands.spawn((ChildOf(screen), at(50.0, stamina_y + stamina_h * 0.5 - 8.0, 12.0, 16.0), image(&bolt, PALE)));
    }

    // ---- weapons, top right ----
    for row in 0..WEAPON_ROWS {
        let y = 43.0 + 45.0 * row as f32;
        commands.spawn((ChildOf(screen), at(463.0, y, 128.0, 32.0), image(&panel, Color::WHITE), Part::Panel(row), Visibility::Hidden));
        commands.spawn((ChildOf(screen), at(463.0, y - 25.0, WEAPON_ICON[0], WEAPON_ICON[1]), flipped(Handle::default()), Part::Icon(row), Visibility::Hidden));
        commands.spawn((ChildOf(screen), game_text("", WEAPON_NAME_X, y - WEAPON_NAME_UP, WEAPON_NAME_CAP, AMMO_ORANGE, Align::Left), Part::Name(row), Visibility::Hidden));
        commands.spawn((ChildOf(screen), game_text("", 475.0, y + 9.0, 14.0, AMMO_ORANGE, Align::Left), Part::Ammo(row), Visibility::Hidden));
    }

    // ---- radar, bottom left (centre 121, 368): disc, view cone, frame, player, portrait; in
    // deathmatch there's no squad, and no radar (capture) ----
    let squad = !mode.deathmatch;
    // (the radar and its portraits on a panel of their own, the screen's size, which slides with
    // the Squad Command menu and draws over its backdrop: RADAR_SLIDE, RADAR_Z)
    let radar = commands.spawn((ChildOf(screen), RadarPanel, GlobalZIndex(RADAR_Z), Name::new("radar panel"),
                                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0),
                                       height: Val::Percent(100.0), ..default() })).id();
    if squad {
        commands.spawn((ChildOf(radar), at(51.0, 298.5, 141.0, 141.0), image(&disc, Color::srgba(1.0, 1.0, 1.0, 0.85))));
        commands.spawn((ChildOf(radar), at(75.0, 322.5, 93.0, 93.0), ImageNode::new(cone)));
        commands.spawn((ChildOf(radar), at(57.5, 305.0, 128.0, 128.0), image(&frame, Color::WHITE)));
        commands.spawn((ChildOf(radar), at(119.5, 367.0, 4.0, 4.0), BackgroundColor(VIEWER_GREY),
                                  BorderRadius::all(Val::Percent(50.0))));
        // the objectives' dots and rim markers (#228): drawn into an image over the radar each
        // frame (`update_radar_marks`)
        let n = (128 * OVERLAY_PX) as u32;
        let overlay = images.add(Image::new(bevy::render::render_resource::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
                                            bevy::render::render_resource::TextureDimension::D2, vec![0; (n * n * 4) as usize],
                                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                            bevy::asset::RenderAssetUsages::default()));
        let dot = game.texture_rgba(RADAR_DOT).map(|(w, h, px)| (w as usize, h as usize, px.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect()));
        if dot.is_none() {
            warn!("radar dot texture h_{RADAR_DOT:08x} not found: the objectives' dots are drawn as squares");
        }
        commands.insert_resource(RadarMarks { image: overlay.clone(), dot, ..default() });
        commands.spawn((ChildOf(radar), at(RADAR_FRAME_AT.0, RADAR_FRAME_AT.1, 128.0, 128.0), ImageNode::new(overlay), ZIndex(2)));
        // the frame's four portrait notches are open in its texture; the game fills them dark behind
        // the faces (texel boxes in the frame texture: top, right, bottom, left)
        for (x0, y0, x1, y1) in [(50.0, 8.0, 80.0, 17.0), (110.0, 50.0, 119.0, 80.0), (48.0, 110.0, 78.0, 119.0), (8.0, 48.0, 17.0, 78.0)] {
            commands.spawn((ChildOf(radar), at(57.5 + x0, 305.0 + y0, x1 - x0, y1 - y0), BackgroundColor(Color::srgba(0.16, 0.2, 0.45, 0.9))));
        }
        for (i, &(cx, cy)) in PORTRAIT_CENTRE.iter().enumerate() {
            let Some((handle, rect)) = face_art(game, &mut images, &mut cache, &mut gen.faces, PORTRAITS[i]) else { continue };
            let (bw, bh) = (rect.width(), rect.height());
            commands.spawn((ChildOf(radar), at(cx - bw / 2.0, cy - bh / 2.0, bw, bh),
                            ImageNode { rect: Some(rect), ..flipped(handle.clone()) }, ZIndex(1),
                            SquadPart::Portrait(i), PortraitArt(handle, rect)));
            // (a test map squad member whose type has no portrait: a "?" in its place, #302)
            commands.spawn((ChildOf(radar), game_text("?", cx, cy - UNKNOWN_CAP * UNKNOWN_TOP, UNKNOWN_CAP, MESSAGE_BLUE, Align::Centre),
                            ZIndex(1), SquadPart::Unknown(i), Visibility::Hidden));
        }
        // a selected member's bracket (SELECT_FRAMES, #319): a negative size is the game's
        // mirror, a positive one the textures' usual flip (`flipped`)
        for (c, &(tex_id, (x, y, w, h))) in SELECT_FRAMES.iter().enumerate() {
            let Some(handle) = texture(game, &mut images, &mut cache, tex_id) else { continue };
            let (left, top) = (RADAR_FRAME_AT.0 + x.min(x + w), RADAR_FRAME_AT.1 + y.min(y + h));
            commands.spawn((ChildOf(radar), at(left, top, w.abs(), h.abs()), ZIndex(2), SquadPart::Selected(c), Visibility::Hidden,
                            ImageNode { flip_x: w < 0.0, flip_y: h > 0.0, ..ImageNode::new(handle) }));
        }
        // each portrait's command tab (`update_tabs`): the flash's copy first, so the icon draws
        // over it (0xc4130)
        for slot in 0..TAB_AT.len() {
            for copy in [true, false] {
                commands.spawn((ChildOf(radar), at(0.0, 0.0, 0.0, 0.0), flipped(Handle::default()), CommandTab { slot, copy }, Visibility::Hidden));
            }
        }
    }

    // (the item box, bottom right, is play_itembox.rs's)

    // ---- a gate's wall panel or a lying gun in reach (#17): the target ring (the reticle texture
    // h_1d2a68d0) on the panel's button or the gun; its prompt, "Hold [X] to activate panel.",
    // "Hold [X] to swap L-Shot-50.", is a glyph line drawn with the hint lines (play_hints.rs
    // `draw_prompt`) ----
    commands.spawn((ChildOf(screen), game_text("", SWITCH_HINT_X, SWITCH_HINT_TOP, SWITCH_HINT_CAP, MESSAGE_BLUE, Align::Left), Part::SwitchHint, Visibility::Hidden));
    // the special skill's name, under the bars while it's on (see `StaminaBar`)
    commands.spawn((ChildOf(screen), game_text("", LABEL_X, LABEL_TOP, LABEL_CAP, MESSAGE_BLUE, Align::Left), Part::SkillLabel, Visibility::Hidden));
    let ring = texture(game, &mut images, &mut cache, CROSSHAIR_TEX);
    commands.spawn((ChildOf(screen), at(0.0, 0.0, RING_SIZE, RING_SIZE), image(&ring, RING_BLUE), Part::Ring, Visibility::Hidden));

    // ---- below the middle of the screen: the status message, the pickup lines under it ----
    let message_rgb = Color::srgb_u8(MESSAGE_RGB[0], MESSAGE_RGB[1], MESSAGE_RGB[2]);
    commands.spawn((ChildOf(screen), MessageLine, Visibility::Hidden,
                    record_text("", 320.0, MESSAGE_TOP, &BODY12, vec![message_rgb], Some(Color::BLACK), Across::Centre)));
    for i in 0..FEED_LINES {
        commands.spawn((ChildOf(screen), game_text("", 320.0, MESSAGE_Y + FEED_FIRST + FEED_STEP * i as f32, 11.5, MESSAGE_BLUE, Align::Centre),
                        Part::Feed(i), Visibility::Hidden));
    }

    // ---- crosshair, above centre (320, 192) ----
    // the held weapon's own crosshair (its definition's reticule-prefix texture), 64 units across,
    // in the reticle's tint (play_reticle.rs)
    commands.spawn((ChildOf(screen), at(320.0 - CROSSHAIR_SIZE / 2.0, CROSSHAIR_Y - CROSSHAIR_SIZE / 2.0, CROSSHAIR_SIZE, CROSSHAIR_SIZE),
                    image(&cross, super::reticle::BLUE), Part::Crosshair));

    // ---- grenade throw: the charge meter, right of the crosshair (capture: box 354..367 x 163..221,
    // orange fill from the bottom, dark tick at ~72%). A thrown grenade has no crosshair of its own:
    // the gun's crosshair stays as it is meanwhile (todo/119 take01) ----
    // (the inside of the box is clear: the game draws only the fill and the frame, 0xadac0)
    commands.spawn((ChildOf(screen), SquadPart::MeterBox, Visibility::Hidden, BorderColor(HUD_BLUE),
                    Node { border: UiRect::all(Val::Px(1.5)), ..at(METER_BOX.0, METER_BOX.1, METER_BOX.2 - METER_BOX.0, METER_BOX.3 - METER_BOX.1) }));
    commands.spawn((ChildOf(screen), SquadPart::MeterFill, Visibility::Hidden, BackgroundColor(METER_ORANGE),
                    at(METER_FILL.0, METER_FILL.3, METER_FILL.1 - METER_FILL.0, 0.0)));
    commands.spawn((ChildOf(screen), SquadPart::MeterTick, Visibility::Hidden, BackgroundColor(METER_TICK),
                    at(METER_FILL.0, METER_TICK_Y, METER_FILL.1 - METER_FILL.0, 1.5)));

    // ---- squad: radar blips, selection marker, name over the chosen member ----
    if !squad {
        return;
    }
    for slot in 0..3 {
        commands.spawn((ChildOf(radar), SquadPart::Blip(slot), Visibility::Hidden, BackgroundColor(SQUAD_GREEN),
                        BorderRadius::all(Val::Percent(50.0)), ZIndex(2), at(0.0, 0.0, 4.0, 4.0)));
    }
    // three red chevrons on the radar pointing at the portrait of the member being given control
    for k in 0..3 {
        commands.spawn((ChildOf(radar), SquadPart::Chevron(k), Visibility::Hidden, ImageNode::new(chevrons[0].clone()).with_color(CHEVRON_RED),
                        ZIndex(2 + k as i32), at(0.0, 0.0, 30.0, 30.0)));
    }
    // the chosen member's name label (LABEL_SHOW): the heading font's record text
    commands.spawn((ChildOf(screen), NameLabelText, Visibility::Hidden, ZIndex(3),
                    record_text("", LABEL_CENTRE.0, LABEL_CENTRE.1, &HEADING24, vec![LABEL_ORANGE], None, Across::Centre)));
    for (c, &(x, y)) in BUBBLE_AT.iter().enumerate() {
        commands.spawn((ChildOf(radar), SquadPart::Bubble(c), Visibility::Hidden, flipped(Handle::default()), ZIndex(2), at(x, y, BUBBLE_SIZE, BUBBLE_SIZE)));
    }
    // squad health in the radar frame's diagonal channels (the frame's dark channel is the track)
    for (c, bar) in bars.into_iter().enumerate() {
        commands.spawn((ChildOf(radar), SquadPart::Health(c), ZIndex(1), at(RADAR_FRAME_AT.0, RADAR_FRAME_AT.1, 128.0, 128.0),
                        ImageNode::new(bar).with_color(HEALTH_PINK)));
    }
}

/// The stamina bar widget each frame (see `StaminaBar`): FUN_000b7850 sets the value (the
/// brighten on a change), FUN_000b75a0 fades the brighten and moves the trail's end, FUN_000b76c0
/// runs the blink and counts the "full" flash down (both bars' timers, `BarFlash`). Then the fill
/// image is remade mixed toward white by the brighten in sRGB (the takes' rising bar, (139-145,
/// 183-186, 200-206), is the texture half way to white in sRGB, (144, 188, 206); Bevy mixes UI
/// colours in linear light, which gives (188, 202, 214)), or in the flash's colour while it's
/// tinted (`flash_tinted`), the frame's lower half is red while the blink is, and the skill's name
/// label is kept.
#[allow(clippy::too_many_arguments)]
fn update_stamina_bar(time: Res<Time>, player: Res<Player>, mut bar: ResMut<StaminaBar>, gen: Res<HudGen>, mut flash: ResMut<BarFlash>,
                      mut images: ResMut<Assets<Image>>, mut frame: Query<(&FrameBlink, &mut ImageNode)>) {
    let dt = frame_dt(&time);
    let flash = &mut *flash;
    for t in [&mut flash.health, &mut flash.stamina] {
        *t = (*t - dt).max(0.0);
    }
    let tinted = flash_tinted(flash.stamina);
    let max = player.max_health.max(1.0);
    // the HUD's value: stamina / max-stamina x max health, at least 1 when not 0 (FUN_000b7fb0)
    let v = player.stamina / player.max_stamina.max(1e-3) * max;
    let v = if v > 0.0 { v.clamp(1.0, max) } else { 0.0 };
    let name = player.skill.name().unwrap_or(0);
    if bar.who != Some((player.character, player.kind)) {
        // control passed, the type changed (or the map began): the new character's, at once
        let on = player.skill.on;
        *bar = StaminaBar { who: Some((player.character, player.kind)), value: v, shown: v, blink_dir: -1.0, drawn: -1.0, label: name,
                            label_alpha: if on { 1.0 } else { 0.0 }, label_hold: if on { LABEL_HOLD } else { 0.0 }, ..default() };
    } else {
        if v != bar.value {
            bar.brighten = if v > bar.value { BRIGHTEN_RISE } else { BRIGHTEN_FALL };
            bar.value = v;
        }
        bar.brighten = (bar.brighten - dt * BRIGHTEN_FADE).max(0.0);
        if bar.shown < bar.value {
            bar.shown = (bar.shown + dt * TRAIL_UP).min(bar.value);
        } else if bar.shown > bar.value {
            bar.shown = (bar.shown - dt * TRAIL_DOWN).max(bar.value);
        }
        let half = BLINK_SPAN * 0.5;
        if bar.value > LOW_STAMINA * max {
            bar.blink = 0.0;
            bar.blink_dir = -1.0;
        } else {
            bar.blink += dt * bar.blink_dir;
            if bar.blink <= -half || bar.blink >= half {
                bar.blink_dir = -bar.blink_dir;
                bar.blink = bar.blink.clamp(-half, half);
            }
        }
        // the label: full while on, then the hold, then the fade
        if player.skill.on {
            bar.label = name;
            bar.label_alpha = 1.0;
            bar.label_hold = LABEL_HOLD;
        } else if bar.label_hold > 0.0 {
            bar.label_hold -= dt;
        } else {
            bar.label_alpha = (bar.label_alpha - dt * LABEL_FADE).max(0.0);
        }
    }
    if (bar.brighten - bar.drawn).abs() > 1e-3 || tinted != bar.drawn_flash {
        bar.drawn = bar.brighten;
        bar.drawn_flash = tinted;
        let b = bar.brighten;
        let flashed = tinted.then(|| flash_texels(&gen.stamina_texels));
        if let Some(data) = images.get_mut(&gen.stamina_fill).and_then(|i| i.data.as_mut()) {
            match flashed {
                Some(f) if f.len() == data.len() => data.copy_from_slice(&f),
                _ => {
                    for (o, i) in data.chunks_exact_mut(4).zip(gen.stamina_texels.chunks_exact(4)) {
                        for c in 0..3 {
                            o[c] = (i[c] as f32 + (255.0 - i[c] as f32) * b).round() as u8;
                        }
                        o[3] = i[3];
                    }
                }
            }
        }
    }
    let red = bar.blink < 0.0;
    for (f, mut img) in &mut frame {
        let want = if red { &f.1 } else { &f.0 };
        if img.image != *want {
            img.image = want.clone();
        }
    }
}

/// Weapon rows (the held weapon first; every weapon with names for a moment after a switch, the
/// held one's name for a moment after a pickup), ammo ("clip / reserve", the squad's reserve of
/// its ammo-type; "clip Regen" for a recharging gun), red panel on an empty clip, the switch
/// hint, the skill's name, crosshair. (The portraits' command tabs are `update_tabs`'.)
#[allow(clippy::too_many_arguments)]
fn update_hud_widgets(
    player: Res<Player>,
    reserve: Res<super::ammo::Reserve>,
    held_name: Res<super::ammo::HeldName>,
    (switch_hint, hint_lines): (Res<super::ammo::SwitchHint>, Res<super::triggers::hints::HintLines>),
    use_panel: Res<super::UsePanel>,
    feed: Res<super::pickups::PickupFeed>,
    mut game: ResMut<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<HudImages>,
    camera: Query<(&Camera, &Transform), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut parts: Query<(&Part, &mut Visibility, Option<&mut ImageNode>, Option<&mut GameText>, Option<&mut Node>)>,
    stamina: Res<StaminaBar>,
    reticle: Res<super::reticle::Reticle>,
) {
    let Some(l) = player.loaded.as_ref() else { return };
    let list = player.hud_list > 0.0;
    // the gun in hand's row, then (in Tex's berserk) the second hand's: both in a hand, both
    // tinted as held, two rows for as long as it's drawn (#35 take01 6.0 s, take04 8.5 s: "99
    // Regen" over "36 / 200"); the list after a switch adds the rest
    let second = player.second.map(|s| s.weapon).filter(|&w| w < l.weapons.len());
    let held = |w: usize| w == player.weapon || Some(w) == second;
    let mut order: Vec<usize> = (0..l.weapons.len()).collect();
    order.sort_by_key(|&i| (i != player.weapon, Some(i) != second));
    order.truncate(if list { WEAPON_ROWS } else if second.is_some() { 2 } else { 1 });
    // a world point on the HUD's 640 x 480 screen (4:3, centred in the window), through the view
    // this frame draws: the camera's Transform as the cameras have just set it (it's a root, so
    // that's its place in the world), not its GlobalTransform, which is the last frame's until
    // PostUpdate propagates it and left the ring a frame behind its target while the view moves
    // (#17's review; the shake, play_shake.rs, is added after and isn't followed)
    let on_screen = |p: Vec3| -> Option<Vec2> {
        let (cam, at) = camera.single().ok()?;
        let win = windows.single().ok()?;
        let px = cam.world_to_viewport(&GlobalTransform::from(*at), p).ok()?;
        let unit = win.height() / 480.0;
        Some(Vec2::new((px.x - (win.width() - 640.0 * unit) / 2.0) / unit, px.y / unit))
    };
    let regen_word = game.0.strings.get(&S_REGEN).cloned().unwrap_or("Regen".into());
    // the switch hint (play_ammo.rs's SwitchHint) names the other weapon
    let other = (l.weapons.len() > 1).then(|| &l.weapons[(player.weapon + 1) % l.weapons.len()].def.label).filter(|_| switch_hint.0);
    let switch_to = game.0.strings.get(&S_SWITCH_TO).cloned().unwrap_or("switch to %s.".into());
    // after a pickup: the held weapon's name over its count, its icon hidden meanwhile
    let name_shown = held_name.0 > 0.0;
    // its fade at the end (1: full), and the icon faint under it meanwhile, coming back as it
    // fades
    let name_alpha = (held_name.0 / super::ammo::NAME_FADE).min(1.0);
    // (FUN_000ad790: "%dx %s" from two in a row, the name alone for one, 0x40d734; nothing taken
    // with a stack full, "%s %s" (0x40d744) with "(full)", in FUN_000b8820's red, #17)
    let full_word = game.0.strings.get(&S_FULL).cloned().unwrap_or("(full)".into());
    let feed_shown: Vec<(String, Color)> = feed.0.iter().rev().take(FEED_LINES).rev().map(|l| match (l.full, l.count) {
        (true, _) => (format!("{} {full_word}", l.name), FULL_RED),
        (false, n) if n > 1 => (format!("{n}x {}", l.name), MESSAGE_BLUE),
        _ => (l.name.clone(), MESSAGE_BLUE),
    }).collect();

    for (part, mut vis, img, mut text, node) in &mut parts {
        let show = |v: &mut Visibility, on: bool| { let want = if on { Visibility::Inherited } else { Visibility::Hidden }; if *v != want { *v = want; } };
        let set = |t: Option<Mut<GameText>>, s: &str, color: Option<Color>| {
            if let Some(mut t) = t {
                if t.text != s { t.text = s.to_string(); }
                if let Some(c) = color { if t.color != c { t.color = c; } }
            }
        };
        match *part {
            Part::Panel(r) | Part::Icon(r) | Part::Name(r) | Part::Ammo(r) => {
                let Some(&w) = order.get(r) else { show(&mut vis, false); continue };
                // (a weapon pickup's release to its grab: nothing in the hand, the held row's panel
                // empty, in its usual tint, take11 300.8-301.1 s; #17. The faint grey shape the
                // take has over it there isn't drawn)
                let gap = r == 0 && player.gun_use.as_ref().is_some_and(|u| u.hand_empty());
                if gap && !matches!(*part, Part::Panel(_)) {
                    show(&mut vis, false);
                    continue;
                }
                let def = &l.weapons[w].def;
                let clip = player.ammo.get(w).map_or(0, |a| a[0]);
                let empty = clip == 0 && !gap;
                let tint = if held(w) { AMMO_ORANGE } else { MESSAGE_BLUE };
                match *part {
                    Part::Panel(_) => {
                        show(&mut vis, true);
                        if let Some(mut img) = img {
                            img.color = if empty { Color::srgb(1.0, 0.35, 0.3) } else { Color::WHITE };
                        }
                    }
                    Part::Icon(r) => {
                        let h = (def.icon != 0).then(|| texture(&mut game.0, &mut images, &mut cache, def.icon)).flatten();
                        show(&mut vis, h.is_some());
                        // at its own size (`WeaponDef::icon_size`, the item's h_e2bc2c81 x
                        // h_f71189bb) from the slot's top left: the MNR-7's 64 x 64, as #287
                        // take18 draws it, where the fixed 128 x 64 slot stretched it twice as wide
                        // (#64); WEAPON_ICON where the data gives none
                        if let Some(mut n) = node {
                            let [w, h] = match def.icon_size { [w, h] if w > 0.0 && h > 0.0 => [w, h], _ => WEAPON_ICON };
                            let (width, height) = (Val::Percent(w / 6.4), Val::Percent(h / 4.8));
                            if n.width != width || n.height != height {
                                n.width = width;
                                n.height = height;
                            }
                        }
                        let faint = if r == 0 && name_shown && !list { ICON_UNDER_NAME + (1.0 - ICON_UNDER_NAME) * (1.0 - name_alpha) } else { 1.0 };
                        if let (Some(mut img), Some(h)) = (img, h) {
                            if img.image != h { img.image = h; }
                            let c = Color::srgba(1.0, 1.0, 1.0, faint);
                            if img.color != c { img.color = c; }
                        }
                    }
                    Part::Name(r) => {
                        show(&mut vis, list || (r == 0 && name_shown));
                        let tint = if list { tint } else { tint.with_alpha(name_alpha) };
                        set(text, &def.label, Some(tint));
                    }
                    _ => {
                        show(&mut vis, true);
                        let count = if def.ammo_regen > 0.0 { format!("{clip} {regen_word}") }
                            else { format!("{clip} / {}", reserve.get(def.ammo_type)) };
                        set(text, &count, Some(tint));
                    }
                }
            }
            Part::SwitchHint => {
                show(&mut vis, other.is_some());
                if let Some(name) = other {
                    set(text, &format!("Q {}", switch_to.replace("%s", name)), None);
                }
            }
            Part::SkillLabel => {
                let name = game.0.strings.get(&stamina.label).cloned().unwrap_or_default();
                show(&mut vis, stamina.label_alpha > 0.0 && !name.is_empty());
                // (lower for each hint line up, the switch hint as the first: LABEL_STACK)
                let lines = (hint_lines.0[0] || other.is_some()) as usize + hint_lines.0[1] as usize;
                let y = LABEL_TOP + lines as f32 * LABEL_STACK;
                if let Some(t) = text.as_mut() {
                    if t.y != y {
                        t.y = y;
                    }
                }
                // (the console blends the stored sRGB values, so its fade is linear in them; Bevy
                // blends in linear light, where the same alpha looks brighter: raised to 2.2 the
                // fade matches take01's, 18.87-19.17 s)
                set(text, &name, Some(MESSAGE_BLUE.with_alpha(stamina.label_alpha.powf(2.2))));
            }
            Part::Ring => {
                // on a gate panel's button or a lying gun (#17), zooming in and out (`Ring`)
                let ring = use_panel.ring;
                let at = ring.at.filter(|_| player.scope < 0.5 && ring.open > 0.0).and_then(on_screen);
                show(&mut vis, at.is_some());
                let c4 = ring.open.powi(4);
                let size = RING_SIZE * (1.0 + RING_GROW * (1.0 - c4));
                if let (Some(c), Some(mut n)) = (at, node) {
                    let want = at_box(c.x - size / 2.0, c.y - size / 2.0, size, size);
                    if n.left != want.left || n.top != want.top || n.width != want.width {
                        (n.left, n.top, n.width, n.height) = (want.left, want.top, want.width, want.height);
                    }
                }
                if let Some(mut img) = img {
                    // (the console blends the stored sRGB values, Bevy in linear light, where the
                    // same alpha looks brighter: raised to 2.2, as the skill's name's fade is)
                    let colour = match ring.colour {
                        super::RingColour::Usable => RING_BLUE,
                        super::RingColour::Refused => RING_RED,
                        super::RingColour::InUse => RING_IN_USE,
                    }.with_alpha(c4.powf(2.2));
                    if img.color != colour {
                        img.color = colour;
                    }
                }
            }
            Part::Feed(k) => {
                let line = feed_shown.get(k);
                show(&mut vis, line.is_some());
                if let Some((line, colour)) = line {
                    set(text, line, Some(*colour));
                }
            }
            Part::Crosshair => {
                // (none once dead: the reference recording's reticle goes on the death frame; none
                // during a weapon switch from the drop to the grab: the takes' crosshair goes
                // ~0.5 s after Y and the new one shows at ~0.95 s, todo/49 and 58 take02). Kept through a
                // grenade throw's charge, release and motion: todo/119 take01 has the Therm Sweeper's
                // box x 294.7-346.7, y 172.0-210.7 on every frame of three full-charge Frag throws.
                show(&mut vis, !l.weapons.is_empty() && !player.dead && player.holding);
                // in the scope: in the middle of the screen, the same size
                let s = player.scope;
                let size = CROSSHAIR_SIZE;
                let (cx, cy) = (320.0, CROSSHAIR_Y + (240.0 - CROSSHAIR_Y) * s);
                if let Some(mut n) = node {
                    let want = at(cx - size / 2.0, cy - size / 2.0, size, size);
                    if n.top != want.top || n.width != want.width {
                        (n.left, n.top, n.width, n.height) = (want.left, want.top, want.width, want.height);
                    }
                }
                let id = l.weapons.get(player.weapon).map(|w| w.def.reticle).filter(|&r| r != 0).unwrap_or(CROSSHAIR_TEX);
                if let (Some(mut img), Some(h)) = (img, texture(&mut game.0, &mut images, &mut cache, id)) {
                    if img.image != h { img.image = h; }
                    // red with a lock, green on a teammate, yellow on a radar-contact 2 or 4, else
                    // blue: switched on the frame it changes; at the reticle's alpha, which Flint's
                    // targeting fades out (play_reticle.rs)
                    let tint = reticle.tint.color().with_alpha(reticle.alpha);
                    if img.color != tint { img.color = tint; }
                }
            }
        }
    }
}

/// The message line (`MessageLine`): the status message's text while it has time left, at the
/// line's alpha 255 x min(1, 2 x time left) (its update 0xb33b0: full, then out over the last
/// 0.5 s; take15's message fades from 7.9 to 8.3 s video).
fn update_message(use_panel: Res<super::UsePanel>, mut line: Query<(&mut Visibility, &mut RecordText), With<MessageLine>>) {
    for (mut vis, mut t) in &mut line {
        let want = if use_panel.message.is_some() { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        if let Some((m, left)) = &use_panel.message {
            if t.text != *m {
                t.text = m.clone();
            }
            let a = (2.0 * left).clamp(0.0, 1.0);
            if (t.alpha - a).abs() > 1e-3 {
                t.alpha = a;
            }
        }
    }
}

/// Squad blips on the radar (turning with the camera, at the radar's scale: RADAR_UNITS), the
/// selection marker and the name (centre of the screen) while control passes to another member,
/// and the grenade meter while a throw charges.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_squad_hud(
    time: Res<Time>,
    player: Res<Player>,
    squad: Res<Squad>,
    mut game: ResMut<GameData>,
    mut gen: ResMut<HudGen>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<HudImages>,
    mut label: ResMut<NameLabel>,
    stamina: Res<StaminaBar>,
    flash: Res<BarFlash>,
    mut parts: Query<(&SquadPart, &mut Visibility, &mut Node, Option<&mut Text>, Option<&mut TextFont>, Option<&mut TextColor>, Option<&mut ImageNode>, Option<&PortraitArt>,
                      Option<&mut BackgroundColor>, Option<&mut BorderColor>)>,
    mut name_text: Query<(&mut RecordText, &mut Visibility), (With<NameLabelText>, Without<SquadPart>)>,
    (menu, mut panel): (Res<super::squad_menu::SquadMenu>, Query<&mut Node, (With<RadarPanel>, Without<SquadPart>)>),
) {
    let dt = frame_dt(&time);
    // the radar panel's slide toward the middle while the Squad Command menu is open
    // (FUN_000c2be0, RADAR_SLIDE: #319)
    if let Ok(mut n) = panel.single_mut() {
        let k = menu.radar_slide();
        let (left, top) = (Val::Percent(RADAR_SLIDE.0 * k / 6.4), Val::Percent(RADAR_SLIDE.1 * k / 4.8));
        if n.left != left || n.top != top {
            (n.left, n.top) = (left, top);
        }
    }
    // squad slot c's member in the world (the controlled character or one of the squad's, not an
    // NPC: `slot_member`)
    let member = |c: usize| slot_member(&player, &squad, c);
    // the name label (LABEL_SHOW): each new selection shows the name for LABEL_SHOW + LABEL_OUT
    // (0xc9440 from the select widget's choice, FUN_000c8cc0, its own timer after: it runs on past
    // the hand-over, which comes SELECT_TIME after the selection); whole from its first frame,
    // over the death camera and on a manual switch alike (the recordings' "HAWK" and "BRUTUS")
    let selecting = player.select.map(|(c, _)| c);
    if selecting.is_some_and(|c| label.selecting != Some(c)) {
        (label.character, label.timer) = (selecting.unwrap_or(0), LABEL_SHOW + LABEL_OUT);
    } else {
        label.timer = (label.timer - dt).max(0.0);
    }
    label.selecting = selecting;
    if let Ok((mut t, mut vis)) = name_text.single_mut() {
        let want = if label.timer > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        if label.timer > 0.0 {
            // 0xc9690: alpha 0.8 x min(1, timer / LABEL_OUT), scale 1 + 3 k^3, k = 1 - that share
            let share = (label.timer / LABEL_OUT).min(1.0);
            let k = 1.0 - share;
            let scale = LABEL_SCALE + LABEL_SWELL * k * k * k;
            // (the slot's member's name: on the test map any type's, #302)
            let name = member(label.character).map_or_else(|| CHARACTERS[label.character % CHARACTERS.len()].to_string(),
                                                            |u| display_name(u, &game.0)).to_uppercase();
            if t.text != name {
                t.text = name;
            }
            // (centred both ways on LABEL_CENTRE as it swells)
            let y = LABEL_CENTRE.1 - HEADING24.height() * scale * 0.5;
            if t.scale != scale || t.y != y {
                (t.scale, t.y) = (scale, y);
            }
            t.alpha = LABEL_ALPHA * share;
        }
    }
    let health = |c: usize| member(c)
        .map(|u| (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0));
    let show = |v: &mut Visibility, on: bool| { let want = if on { Visibility::Inherited } else { Visibility::Hidden }; if *v != want { *v = want; } };
    let pct = |x: f32, y: f32, n: &mut Node| { n.left = Val::Percent(x / 6.4); n.top = Val::Percent(y / 4.8); };
    let fwd = Vec3::new(-player.cam_yaw.sin(), 0.0, -player.cam_yaw.cos());
    let right = Vec3::new(player.cam_yaw.cos(), 0.0, -player.cam_yaw.sin());
    let charging = player.charge > 0.0;
    // the meter (0xad950): shown while its timer runs (METER_TIME from the last charging frame),
    // at its level, at clamp(2 x timer, 0, 1)^3 strength
    let (after, left) = player.meter_after;
    let meter = charging || left > 0.0;
    let level = if charging { player.charge } else { after };
    let strength = if charging { 1.0 } else { meter_strength(left) };
    // beside the crosshair, which the scope moves to the middle (FUN_000ab390)
    let meter_shift = super::reticle::crosshair_centre(&player).y - CROSSHAIR_Y;
    // the chevrons point at the member control is passing to (FUN_000c4d80) or, a moment after a
    // d-pad press, at the one pressed (FUN_000c4dd0, #319: its level for their fade, `press_alpha`)
    let selected = player.select.and_then(|(c, _)| squad.0.iter().find(|m| m.character == c && m.in_squad()).map(|m| (c, m.position)));
    let press = if selected.is_none() { menu.flash() } else { None };
    let selected = selected.or(press.map(|(c, _)| (c, Vec3::ZERO)));
    let dead = |c: usize| std::iter::once(&*player).chain(squad.0.iter()).any(|u| u.character == c && u.in_squad() && u.dead);
    // each slot's place: its member's face (`Face`), and its skull once it's dead (its type's: a
    // squad type's own, another type's from its data, `CharacterType::skull`, else the human one)
    // (a slot with no member in the world shows nothing: #303)
    // (and, selected for the Squad Command menu, its face in the selection's orange: `selected_face`)
    let looks: Vec<(Face, Option<Handle<Image>>, Option<Handle<Image>>)> = (0..PORTRAITS.len()).map(|c| {
        let m = member(c);
        let mut face_id = PORTRAITS[c];
        let face = match m {
            None => Face::Empty,
            Some(u) if u.kind == squad_type(c) => Face::Own,
            Some(u) => {
                let id = match u.data_slot() {
                    OTHER => game.0.character_type(u.kind).map_or(0, |t| t.portrait),
                    d => PORTRAITS[d],
                };
                face_id = id;
                match (id != 0).then(|| face_art(&mut game.0, &mut images, &mut cache, &mut gen.faces, id)).flatten() {
                    Some((h, r)) => Face::Other(h, r),
                    None => Face::Unknown,
                }
            }
        };
        let skull = m.filter(|u| u.dead).map(|u| match u.data_slot() {
            OTHER => {
                let id = game.0.character_type(u.kind).map_or(0, |t| t.skull);
                texture(&mut game.0, &mut images, &mut cache, if id != 0 { id } else { SKULL_HUMAN }).unwrap_or_default()
            }
            d => gen.skulls[d].clone(),
        });
        let lit = (menu.selected(c) && skull.is_none() && matches!(face, Face::Own | Face::Other(..)))
            .then(|| selected_face(&mut game.0, &mut images, &mut cache, face_id)).flatten();
        (face, skull, lit)
    }).collect();
    // the squad's blips: its living members in the world, not the NPCs the triggers or the test
    // tools spawned (the tutorial's terrorists showed as squad blips before #228)
    let members: Vec<&Player> = squad.0.iter().filter(|m| m.in_squad() && !m.dead && !m.absent && !m.stand_in).collect();
    // (the radar's scale, FUN_000b0550: see RADAR_UNITS)
    let scale = RADAR_UNITS / game.0.radar.max_range.max(1.0);
    for (part, mut vis, mut node, _text, _font, _color, img, art, background, border) in &mut parts {
        match *part {
            SquadPart::Blip(slot) => {
                let Some(m) = members.get(slot) else { show(&mut vis, false); continue };
                let rel = m.position - player.position;
                // (beyond the max-range the game pins the squad to the rim, FUN_000ae7c0 through
                // FUN_000b0ad0: here on the circle; within it a member shows only within the
                // player's reach, FUN_000b0670)
                let mut v = Vec2::new(rel.dot(right), rel.dot(fwd)) * scale;
                if v.length() > RADAR_UNITS {
                    v = v.normalize() * RADAR_UNITS;
                } else if radar_reach(&player, &game.0, m.position, game.0.radar.max_range.max(1.0)) <= 0.0 {
                    show(&mut vis, false);
                    continue;
                }
                show(&mut vis, true);
                pct(RADAR_CENTRE.0 + v.x - 2.0, RADAR_CENTRE.1 - v.y - 2.0, &mut node);
                // (selected for the Squad Command menu: 200 200 200, FUN_000c86d0)
                let want = if menu.selected(m.character) { SELECTED_GREY } else { SQUAD_GREEN };
                if let Some(mut c) = background.filter(|c| c.0 != want) {
                    c.0 = want;
                }
            }
            SquadPart::Selected(c) => {
                show(&mut vis, menu.selected(c) && member(c).is_some_and(|u| !u.dead));
            }
            SquadPart::Chevron(k) => {
                show(&mut vis, selected.is_some());
                if let Some((c, _)) = selected {
                    let (px, py) = PORTRAIT_CENTRE[c % PORTRAIT_CENTRE.len()];
                    let u = Vec2::new(px - RADAR_CENTRE.0, py - RADAR_CENTRE.1).normalize_or(Vec2::X);
                    let at = Vec2::new(px, py) - u * (28.0 - 5.0 * k as f32);
                    pct(at.x - 15.0, at.y - 15.0, &mut node);
                    let dir = if u.x.abs() > u.y.abs() { if u.x > 0.0 { 0 } else { 1 } } else if u.y < 0.0 { 2 } else { 3 };
                    // lit one after another toward the portrait, then all dim again; after a d-pad
                    // press, the game's fade (`press_alpha`)
                    let step = (time.elapsed_secs() * 10.0) as usize % 4;
                    let lit = k < step;
                    if let Some(mut img) = img {
                        if img.image != gen.chevrons[dir] { img.image = gen.chevrons[dir].clone(); }
                        img.color = match press {
                            Some((_, level)) => CHEVRON_RED.with_alpha(press_alpha(level, k)),
                            None if lit => CHEVRON_RED,
                            None => CHEVRON_RED.darker(0.25).with_alpha(0.75),
                        };
                    }
                }
            }
            SquadPart::Bubble(c) => {
                let speaking = std::iter::once(&*player).chain(squad.0.iter()).any(|u| u.character == c && u.in_squad() && u.speaking > 0.0);
                show(&mut vis, speaking);
                if let Some(mut img) = img {
                    if img.image != gen.bubble { img.image = gen.bubble.clone(); }
                }
            }
            SquadPart::Portrait(c) => {
                if let (Some(mut img), Some(art), Some((face, skull, lit))) = (img, art, looks.get(c)) {
                    // (the face and its box: the slot's own, or another one, centred in the place)
                    let (own, box_) = match face {
                        Face::Other(h, r) => ((h.clone(), *r), *r),
                        _ => ((art.0.clone(), art.1), art.1),
                    };
                    show(&mut vis, skull.is_some() || matches!(face, Face::Own | Face::Other(..)));
                    // (selected for the Squad Command menu: in the selection's orange, 0xc4280, #319)
                    let own = (lit.clone().unwrap_or(own.0), own.1);
                    let (want, rect, color) = match skull {
                        Some(s) => (s.clone(), None, SKULL_TINT),
                        None => (own.0, Some(own.1), Color::WHITE),
                    };
                    if img.image != want { img.image = want; }
                    if img.rect != rect { img.rect = rect; }
                    img.color = color;
                    let (cx, cy) = PORTRAIT_CENTRE[c % PORTRAIT_CENTRE.len()];
                    let want = at(cx - box_.width() / 2.0, cy - box_.height() / 2.0, box_.width(), box_.height());
                    if node.width != want.width || node.height != want.height || node.left != want.left || node.top != want.top {
                        (node.left, node.top, node.width, node.height) = (want.left, want.top, want.width, want.height);
                    }
                }
            }
            SquadPart::Unknown(c) => {
                show(&mut vis, looks.get(c).is_some_and(|(face, skull, _)| matches!(face, Face::Unknown) && skull.is_none()));
            }
            SquadPart::Health(c) => {
                let Some(f) = health(c) else { show(&mut vis, false); continue };
                show(&mut vis, !dead(c) && f > 0.0);
                // light the channel's texels from the portrait end up to the health fraction
                if (gen.bar_fill[c] - f).abs() > 1e-3 {
                    gen.bar_fill[c] = f;
                    if let Some(image) = images.get_mut(&gen.bars[c]) {
                        if let Some(data) = image.data.as_mut() {
                            for &(i, t) in &gen.bar_texels[c] {
                                data[i * 4 + 3] = if t <= f { 255 } else { 0 };
                            }
                        }
                    }
                }
            }
            SquadPart::PlayerHealth => {
                let f = (player.health / player.max_health.max(1.0)).clamp(0.0, 1.0);
                let width = if gen.tutorial_frame { bar_fill(player.health.clamp(0.0, player.max_health)) } else { bar_span().1 * f };
                node.width = Val::Percent(width / 6.4);
                // in the "full" flash's colour while it's tinted (`BarFlash`: a HEALTH POWER
                // walked over at full health)
                let want = if flash_tinted(flash.health) { gen.health_flash.clone() } else { gen.health_fill.clone() };
                if let (Some(mut img), Some(want)) = (img, want) {
                    if img.image != want {
                        img.image = want;
                    }
                }
            }
            // the stamina bar's value and its trail, as `update_stamina_bar` keeps them
            SquadPart::PlayerStamina => {
                node.width = Val::Percent(bar_fill(stamina.value) / 6.4);
                show(&mut vis, stamina.value > 0.0);
            }
            SquadPart::StaminaTrail => {
                let (from, to) = (bar_fill(stamina.value), bar_fill(stamina.shown));
                show(&mut vis, to > from);
                node.left = Val::Percent((BAR_X + from) / 6.4);
                node.width = Val::Percent((to - from).max(0.0) / 6.4);
            }
            SquadPart::FrameMiddle => {
                // the middle takes up what the bars' length adds to (or takes from) the picture's
                node.width = Val::Percent(((TB_MIDDLE.1 - TB_MIDDLE.0) as f32 + frame_stretch(player.max_health)).max(1.0) / 6.4);
            }
            SquadPart::FrameRight => {
                node.left = Val::Percent((TB_AT.0 + (TB_RIGHT.0 - TB_ICONS.0) as f32 + frame_stretch(player.max_health)) / 6.4);
            }
            SquadPart::MeterBox => {
                show(&mut vis, meter);
                node.top = Val::Percent((METER_BOX.1 + meter_shift) / 4.8);
                let want = HUD_BLUE.with_alpha(strength);
                if let Some(mut c) = border.filter(|c| c.0 != want) {
                    c.0 = want;
                }
            }
            SquadPart::MeterTick => {
                show(&mut vis, meter);
                node.top = Val::Percent((METER_TICK_Y + meter_shift) / 4.8);
                let want = METER_TICK.with_alpha(strength);
                if let Some(mut c) = background.filter(|c| c.0 != want) {
                    c.0 = want;
                }
            }
            SquadPart::MeterFill => {
                show(&mut vis, meter);
                let h = (METER_FILL.3 - METER_FILL.2) * level;
                node.top = Val::Percent((METER_FILL.3 - h + meter_shift) / 4.8);
                node.height = Val::Percent(h / 4.8);
                let want = METER_ORANGE.with_alpha(METER_ORANGE.alpha() * strength);
                if let Some(mut c) = background.filter(|c| c.0 != want) {
                    c.0 = want;
                }
            }
        }
    }
}

// ---- the radar's objectives (#228): FUN_000ae580 -> FUN_000aece0, drawn by 0xb01b0 / 0xaf840 ----

/// One of the radar's contact records (the list at the radar's +0x3c, made by FUN_000aece0, aged
/// by FUN_000ae3f0, drawn by 0xb01b0): where (screen units from the radar's middle, whole numbers:
/// FUN_000b0550 casts them), its colour, its alpha byte, the time it has left and its scale.
struct Ping {
    x: f32,
    y: f32,
    rgb: [f32; 3],
    alpha: f32,
    life: f32,
    scale: f32,
}

/// An objective beyond the max-range this update (the list at +0x48, refilled by FUN_000aece0
/// each update): its marker's inner point and its point on the circle (screen units from the
/// middle), its colour and alpha byte. Drawn as two lines from the circle point, the corner, to
/// the inner point pushed out as far to each side (0xafaa0, record type 1; see RIM_DEPTH).
struct Rim {
    inner: Vec2,
    outer: Vec2,
    rgb: [f32; 3],
    alpha: f32,
}

/// The radar's objective dots and rim markers (#228) and the image they're drawn into, over the
/// radar (set up with the HUD, `setup_hud`; a default one where there's no radar).
#[derive(Resource, Default)]
struct RadarMarks {
    pings: Vec<Ping>,
    rims: Vec<Rim>,
    /// the objects seen in the last SEEN_TIME s (the radar's map at +0x78: FUN_000ae360,
    /// FUN_000ae300), by `radar::Contact::key`
    seen: HashMap<u32, f32>,
    /// time towards the radar's next update (one a game frame: `shots::TICK`)
    tick: f32,
    /// the overlay's image (128 x 128 units at OVERLAY_PX), and whether it holds anything
    image: Handle<Image>,
    drawn: bool,
    /// the dot texture h_ecea7a68: width, height and its alpha (the texels are white)
    dot: Option<(usize, usize, Vec<f32>)>,
    /// the hostiles the viewer knows of by its own sight (FUN_000ae470's FUN_000572a0 for the
    /// viewer: the controlled character's sensor keeps none, #197), each with how long it's been
    /// unseen: kept brain::FORGET s, as a contact is (`radar_knowledge`)
    viewer_known: HashMap<Who, f32>,
}

/// FUN_000ae470 with h_14be5318 off (as shipped): whether the radar shows a hostile, `who`: the
/// viewer knows of it (`RadarMarks::viewer_known`, from its own sight: `Sensor::scan` from the
/// controlled character, kept FORGET s unseen as a contact is) or a living squadmate does (one
/// with a squad record: the squad's members and the marines following, `Sensor::knows`; the
/// squad's broadcast tells them of each other's targets, play_brain.rs).
fn radar_knows(marks: &RadarMarks, squad: &Squad, who: Who) -> bool {
    marks.viewer_known.contains_key(&who)
        || squad.0.iter().any(|m| !m.dead && !m.absent && (m.in_squad() || m.brain.as_ref().is_some_and(|b| b.follows())) && m.sensor.knows(who))
}

/// The viewer's own knowledge for the radar (see `RadarMarks::viewer_known`): this frame's
/// sightings from the controlled character (none while it's dead) renewed, the rest a frame
/// older, the ones unseen brain::FORGET s or gone from the world dropped.
fn radar_knowledge(marks: &mut RadarMarks, player: &Player, squad: &Squad, dt: f32) {
    let sight = super::sight_list(std::iter::once(player).chain(squad.0.iter()));
    for t in marks.viewer_known.values_mut() {
        *t += dt;
    }
    if !player.dead && !player.absent {
        for (i, _) in player.sensor.scan(player, 0, &sight) {
            marks.viewer_known.insert(sight[i].who, 0.0);
        }
    }
    marks.viewer_known.retain(|w, t| *t < super::brain::FORGET && sight.iter().any(|s| s.who == *w && s.alive && s.team != player.team));
}

/// FUN_000b0670 / FUN_000b0770: how much a character `at` shows on the radar by its distance
/// across from the player's character: its type's `radar-range` (type+0x2a0, eased toward
/// +0x2a4 by char+0x57c, taken as 0: its writers aren't traced beyond the character's set-up and
/// the radar's own reset), and none beyond; within it full but for the last 2 m, (reach - d) / 2.
/// A type with no `radar-range` shows to the radar's max-range.
fn radar_reach(player: &Player, game: &Game, at: Vec3, max_range: f32) -> f32 {
    let reach = game.character_type(player.kind).and_then(|t| t.skill.as_ref()).map(|s| s.radar_range).filter(|&r| r > 0.0).unwrap_or(max_range);
    let d = Vec2::new(at.x - player.position.x, at.z - player.position.z).length();
    ((reach - d) * 0.5).clamp(0.0, 1.0)
}

/// FUN_000b0550: a world offset `rel` (from the viewer) on the radar, in screen units from its
/// middle (x right, y down), turned by the view (`fwd`, the demo's camera heading: the squad's
/// blips use the same) and scaled by `scale`; each cut to a whole number as the game's int casts do.
fn radar_point(rel: Vec3, fwd: Vec3, scale: f32) -> Vec2 {
    let x = (rel.z * fwd.x - rel.x * fwd.z) * scale;
    let y = -(rel.x * fwd.x + rel.z * fwd.z) * scale;
    Vec2::new(x.trunc(), y.trunc())
}

/// The radar's objectives each frame (#228). The radar's update runs a game frame at a time
/// (`shots::TICK`: the records' count, and so how solid a dot looks, follows the game's 30 frames
/// a second whatever the demo's), cutscene or not (take03: the first waypoint's dot is whole the
/// frame the HUD comes back after the opening); a contact it hasn't seen gets its first record on
/// the frame it shows, as the game's update would in that frame:
///  * FUN_000ae300 ages the seen map and FUN_000ae3f0 the records: one whose life is out goes,
///    the others lose the frame's time and grow by PING_GROWTH a second;
///  * FUN_000ae580 / FUN_000aece0, for each objective contact (`radar::RadarContacts`: the
///    level's objects and the characters with 2, 3 or 7): a new record where it is when it's
///    within the max-range (FUN_000b0940: across, from the viewer), living PING_NEW_LIFE if it
///    wasn't seen in the last SEEN_TIME s (FUN_000ae360) and PING_LIFE if it was; one that's beyond
///    gets a rim marker (and, new, a record at its point on the circle). Yellow, the important
///    pickup green; its alpha 255, or BLINK_DIM on the off half of its blink (its time t above 0
///    and frac(t / 2) < 0.5).
/// The other characters (FUN_000ae7c0, for radar-contact 1 and 6): one within the max-range a
/// record as an objective's, coloured by its side (CHARACTER_FRIEND, CHARACTER_HOSTILE,
/// CHARACTER_NEUTRAL); the squad's own members are play_hud.rs's blips (`update_squad_hud`).
/// A hostile shows only once the viewer or a squadmate knows of it (FUN_000ae470, h_14be5318 off:
/// `radar_knows`), and every one only within the player's reach, fading over its edge
/// (FUN_000b0670, FUN_000b0770: `radar_reach`).
/// Then the records and markers are drawn into the overlay: each record a quad of the dot texture
/// 2 x its scale units across round its point, its alpha x min(life / PING_FADE, 1); each marker
/// an "L" with its corner on the circle (RIM_DEPTH, RIM_LINE). take07 shows the result: the new
/// waypoint's dot comes in full on its first frames (the new record's alpha was the blink's dim
/// one: set with time 2, t / 2 = 1 is on the off half), holds 1.1 s, is gone 0.8 s, then stays
/// (its records fading over their last 0.25 s); the old one goes over 0.2 s as its records run
/// out.
#[allow(clippy::too_many_arguments)]
fn update_radar_marks(time: Res<Time>, contacts: Res<super::triggers::radar::RadarContacts>, player: Res<Player>, squad: Res<Squad>,
                      game: Res<GameData>, mut marks: ResMut<RadarMarks>, mut images: ResMut<Assets<Image>>) {
    if marks.image == Handle::default() {
        return;
    }
    let dt = frame_dt(&time);
    let max_range = game.0.radar.max_range.max(1.0);
    let scale = RADAR_UNITS / max_range;
    let fwd = Vec3::new(-player.cam_yaw.sin(), 0.0, -player.cam_yaw.cos());
    let viewer = player.position;
    // the other characters FUN_000ae7c0 makes records of (see the notes above): each one's key in
    // the seen map (a trigger copy's name, else its NPC number, top bits set apart from the
    // objectives'), where it is and its colour
    // (the viewer's own knowledge, this frame's sight in it, FUN_000ae470)
    radar_knowledge(&mut marks, &player, &squad, dt);
    let characters: Vec<(u32, Vec3, [f32; 3], f32)> = squad.0.iter().enumerate()
        .filter(|(_, u)| !u.dead && !u.absent && !u.stand_in && !u.in_squad() && matches!(u.radar_contact, 1 | 6))
        .filter_map(|(k, u)| {
            let rel = Vec3::new(u.position.x - viewer.x, 0.0, u.position.z - viewer.z);
            // (FUN_000b0940: one beyond the range makes no record)
            if rel.length() >= max_range {
                return None;
            }
            // FUN_000b0670: within the player's reach; FUN_000b0770: fading over its last 2 m
            let show = radar_reach(&player, &game.0, u.position, max_range);
            if show <= 0.0 {
                return None;
            }
            let boss = usize::from(u.radar_contact == 6);
            // FUN_0004b960 (friends: the same team, one not hostile to itself), else FUN_000a8520
            // (the viewer's team hostile to its); a hostile only once the viewer or a squadmate
            // knows of it (FUN_000ae580 -> FUN_000ae470)
            let rgb = if u.team == player.team && !super::damage::hostile(u.team, u.team) {
                CHARACTER_FRIEND[boss]
            } else if super::damage::hostile(player.team, u.team) {
                if !radar_knows(&marks, &squad, u.who()) {
                    return None;
                }
                CHARACTER_HOSTILE[boss]
            } else {
                CHARACTER_NEUTRAL
            };
            let key = match (u.trigger_name, u.npc.as_ref()) {
                (0, Some(n)) => 0x4000_0000 | n.id,
                (0, None) => 0x2000_0000 | k as u32,
                (name, _) => 0x8000_0000 | name,
            };
            Some((key, rel, rgb, 255.0 * show))
        }).collect();
    // a contact's record for FUN_000aece0: where (pulled onto the circle beyond the range), its
    // colour and alpha (the blink), whether it's within the range
    let record = |c: &super::triggers::radar::Contact| {
        let rel = Vec3::new(c.at.x - viewer.x, 0.0, c.at.z - viewer.z);
        let inside = rel.length() < max_range;
        let rgb = if c.contact == 7 { PICKUP_GREEN } else { OBJECTIVE_YELLOW };
        let half = 0.5 * c.time;
        let alpha = if half > 0.0 && half.fract() < 0.5 { BLINK_DIM } else { 255.0 };
        // FUN_000b0940 pulls one beyond the range onto the circle
        let on_circle = if inside { rel } else { rel.normalize_or_zero() * max_range };
        (rel, on_circle, inside, rgb, alpha)
    };
    // a contact the radar hasn't seen lately gets its record on the frame it first shows (the
    // game's radar update runs every game frame, the frame a radar-trigger sets it too: take07's
    // new waypoint, set with time 2, starts on the blink's off half and its first record is dim)
    for c in &contacts.0 {
        if marks.seen.contains_key(&c.key) {
            continue;
        }
        marks.seen.insert(c.key, SEEN_TIME);
        let (_, on_circle, _, rgb, alpha) = record(c);
        let p = radar_point(on_circle, fwd, scale);
        marks.pings.push(Ping { x: p.x, y: p.y, rgb, alpha, life: PING_NEW_LIFE, scale: 1.0 });
    }
    for &(key, rel, rgb, alpha) in &characters {
        if marks.seen.insert(key, SEEN_TIME).is_none() {
            let p = radar_point(rel, fwd, scale);
            marks.pings.push(Ping { x: p.x, y: p.y, rgb, alpha, life: PING_NEW_LIFE, scale: 1.0 });
        }
    }
    marks.tick += dt;
    while marks.tick >= super::shots::TICK {
        marks.tick -= super::shots::TICK;
        let step = super::shots::TICK;
        // FUN_000ae300: kept while its time is above the frame's
        marks.seen.retain(|_, t| { let keep = *t > step; *t -= step; keep });
        // FUN_000ae3f0: kept while its life is above the frame's; it grows meanwhile
        marks.pings.retain_mut(|p| {
            let keep = p.life > step;
            p.life -= step;
            p.scale += step * PING_GROWTH;
            keep
        });
        marks.rims.clear();
        for c in &contacts.0 {
            // FUN_000ae360: seen lately (its time set back to SEEN_TIME), or new again
            let new = marks.seen.insert(c.key, SEEN_TIME).is_none();
            let (rel, on_circle, inside, rgb, alpha) = record(c);
            if inside || new {
                let p = radar_point(on_circle, fwd, scale);
                marks.pings.push(Ping { x: p.x, y: p.y, rgb, alpha, life: if new { PING_NEW_LIFE } else { PING_LIFE }, scale: 1.0 });
            }
            if !inside {
                // FUN_000b0ad0: the inner point RIM_DEPTH units in along the way to it
                let inner = on_circle - rel.normalize_or_zero() * (RIM_DEPTH / scale);
                marks.rims.push(Rim { inner: radar_point(inner, fwd, scale), outer: radar_point(on_circle, fwd, scale), rgb, alpha });
            }
        }
        // FUN_000ae7c0: the other characters within the range (FUN_000ae360 as for the objectives)
        for &(key, rel, rgb, alpha) in &characters {
            let new = marks.seen.insert(key, SEEN_TIME).is_none();
            let p = radar_point(rel, fwd, scale);
            marks.pings.push(Ping { x: p.x, y: p.y, rgb, alpha, life: if new { PING_NEW_LIFE } else { PING_LIFE }, scale: 1.0 });
        }
    }
    // the overlay: nothing to draw and nothing drawn before, the image is left alone
    if marks.pings.is_empty() && marks.rims.is_empty() && !marks.drawn {
        return;
    }
    let n = 128 * OVERLAY_PX;
    let mut px = vec![[0.0f32; 4]; n * n];
    for p in &marks.pings {
        let a = p.alpha / 255.0 * (p.life / PING_FADE).min(1.0);
        let s = 1.0 * p.scale;
        draw_dot(&mut px, n, marks.dot.as_ref(), 64.0 + p.x, 64.0 + p.y, s, p.rgb, a);
    }
    for r in &marks.rims {
        // (0xafaa0: the corner A on the circle, its arms to B pushed out to each side, square to
        // A-B by as much)
        let d = r.inner - r.outer;
        let side = Vec2::new(-d.y, d.x);
        let corner = Vec2::splat(64.0) + r.outer;
        for end in [r.inner + side, r.inner - side] {
            draw_line(&mut px, n, corner, Vec2::splat(64.0) + end, RIM_LINE, r.rgb, r.alpha / 255.0);
        }
    }
    marks.drawn = !marks.pings.is_empty() || !marks.rims.is_empty();
    if let Some(image) = images.get_mut(&marks.image) {
        image.data = Some(px.iter().flat_map(|p| [p[0], p[1], p[2], p[3]].map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)).collect());
    }
}

/// One of the radar's records drawn into the overlay `px` (n x n, OVERLAY_PX a screen unit, straight
/// alpha): a quad `s` units each way round (`cx`, `cy`) (screen units from the radar's top left) of
/// the dot texture `dot` (its alpha, sampled bilinearly; a plain square without it), in `rgb` at
/// alpha `a`, laid over what's there (0xb01b0 draws them in the list's order, the newest last).
#[allow(clippy::too_many_arguments)]
fn draw_dot(px: &mut [[f32; 4]], n: usize, dot: Option<&(usize, usize, Vec<f32>)>, cx: f32, cy: f32, s: f32, rgb: [f32; 3], a: f32) {
    if a <= 0.0 || s <= 0.0 {
        return;
    }
    let k = OVERLAY_PX as f32;
    let (x0, x1, y0, y1) = ((cx - s) * k, (cx + s) * k, (cy - s) * k, (cy + s) * k);
    let (lo_x, hi_x) = (x0.floor().max(0.0) as usize, (x1.ceil().max(0.0) as usize).min(n));
    let (lo_y, hi_y) = (y0.floor().max(0.0) as usize, (y1.ceil().max(0.0) as usize).min(n));
    for y in lo_y..hi_y {
        for x in lo_x..hi_x {
            let (u, v) = ((x as f32 + 0.5 - x0) / (x1 - x0), (y as f32 + 0.5 - y0) / (y1 - y0));
            if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                continue;
            }
            let t = match dot {
                Some((w, h, alpha)) => {
                    // bilinear, texel centres at (i + 0.5) / w, clamped at the edges
                    let fx = (u * *w as f32 - 0.5).clamp(0.0, (*w - 1) as f32);
                    let fy = (v * *h as f32 - 0.5).clamp(0.0, (*h - 1) as f32);
                    let (ix, iy) = (fx.floor() as usize, fy.floor() as usize);
                    let (jx, jy) = ((ix + 1).min(*w - 1), (iy + 1).min(*h - 1));
                    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
                    let at = |x: usize, y: usize| alpha[y * *w + x];
                    (at(ix, iy) * (1.0 - tx) + at(jx, iy) * tx) * (1.0 - ty) + (at(ix, jy) * (1.0 - tx) + at(jx, jy) * tx) * ty
                }
                None => 1.0,
            };
            blend(&mut px[y * n + x], rgb, a * t);
        }
    }
}

/// A rim marker's line, `width` screen units wide (the game draws them as a line list, 0xaf8d6),
/// from `from` to `to` (screen units from the radar's top left) into the overlay, edges smoothed.
#[allow(clippy::too_many_arguments)]
fn draw_line(px: &mut [[f32; 4]], n: usize, from: Vec2, to: Vec2, width: f32, rgb: [f32; 3], a: f32) {
    let k = OVERLAY_PX as f32;
    let (p0, p1) = (from * k, to * k);
    let half = 0.5 * width * k;
    let (lo, hi) = (p0.min(p1) - Vec2::splat(half + 1.0), p0.max(p1) + Vec2::splat(half + 1.0));
    let along = p1 - p0;
    let len2 = along.length_squared().max(1e-6);
    for y in (lo.y.max(0.0) as usize)..(hi.y.max(0.0) as usize).min(n) {
        for x in (lo.x.max(0.0) as usize)..(hi.x.max(0.0) as usize).min(n) {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let t = ((p - p0).dot(along) / len2).clamp(0.0, 1.0);
            let d = p.distance(p0 + along * t);
            let cover = (half + 0.5 - d).clamp(0.0, 1.0);
            if cover > 0.0 {
                blend(&mut px[y * n + x], rgb, a * cover);
            }
        }
    }
}

/// `rgb` at alpha `a` laid over the overlay's pixel `dst` (straight alpha both).
fn blend(dst: &mut [f32; 4], rgb: [f32; 3], a: f32) {
    let a = a.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let out = a + dst[3] * (1.0 - a);
    for c in 0..3 {
        dst[c] = (rgb[c] * a + dst[c] * dst[3] * (1.0 - a)) / out;
    }
    dst[3] = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::brain::Mode;

    /// The tab's icon by command mode (0xc3f80's jump table): FOLLOW's green arrows 16 x 8, GUARD's
    /// bars 8 x 16, WINGMAN's ring and MOVE_OUT's arrow 16 x 16; FORMATION and FREEWILL none.
    #[test]
    fn tab_icons_by_mode() {
        assert_eq!(tab_icon(Mode::Follow), Some((0xEEAA_5549, 16.0, 8.0)));
        assert_eq!(tab_icon(Mode::Guard), Some((0xFA29_B7A4, 8.0, 16.0)));
        assert_eq!(tab_icon(Mode::Wingman), Some((0xF672_2366, 16.0, 16.0)));
        assert_eq!(tab_icon(Mode::MoveOut), Some((0xEED3_270A, 16.0, 16.0)));
        assert_eq!(tab_icon(Mode::Formation), None);
        assert_eq!(tab_icon(Mode::Freewill), None);
    }

    /// A new icon flashes for 1 / 1.5 s (the first one too); the same icon doesn't; an empty slot
    /// keeps the state.
    #[test]
    fn a_new_icon_flashes() {
        let dt = 0.125;
        let mut s = (None, None);
        tab_step(&mut s, Some(TAB_FOLLOW), dt);
        assert_eq!(s.0, Some(TAB_FOLLOW));
        assert_eq!(s.1, Some(TAB_FLASH_RATE * dt));
        let mut frames = 1;
        while s.1.is_some() {
            tab_step(&mut s, Some(TAB_FOLLOW), dt);
            frames += 1;
        }
        assert_eq!(frames, 6, "past 1 in its sixth eighth of a second: 0.67 s");
        tab_step(&mut s, None, dt);
        assert_eq!(s, (Some(TAB_FOLLOW), None));
        tab_step(&mut s, Some(TAB_GUARD), dt);
        assert!(s.0 == Some(TAB_GUARD) && s.1.is_some());
    }

    /// The flash's copy (0xc4130): the icon's size and place at the start, whole till 2 (1 - t)^2
    /// falls under 1 (t 0.29), 2.8 times the icon and gone at the end, shifted up and left by 1.8
    /// (s - 1) x s.
    #[test]
    fn the_flash_grows_and_fades() {
        assert_eq!(tab_flash(0.0), (1.0, 0.0, 1.0));
        let (_, _, a) = tab_flash(0.29);
        assert_eq!(a, 1.0);
        let (s, off, a) = tab_flash(0.5);
        assert!((s - 1.9).abs() < 1e-6 && (a - 0.5).abs() < 1e-6 && (off + 1.8 * 0.9 * 1.9).abs() < 1e-5);
        let (s, off, a) = tab_flash(1.0);
        assert!((s - 2.8).abs() < 1e-6 && a == 0.0 && (off + 1.8 * 1.8 * 2.8).abs() < 1e-5);
    }
}

