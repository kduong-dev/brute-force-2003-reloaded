//! The in-game HUD, laid out as in Brute Force: health / energy bars top left, the weapon panel
//! (icon, clip / reserve, a full list for a moment after a switch) top right, radar with the
//! character's portrait bottom left, grenade box bottom right, and the crosshair above centre.
//!
//! Positions are in the game's 640 x 480 screen, measured from an xemu capture, inside a 4:3 area
//! centred in the window (so it scales with the window like the game). Textures are the game's
//! own (data/common.tgz), identified by the HUD page of the options screen
//! (common/game-options-en.xml: radar 132e4ee1 + 17d0ee37, weapon panel e11fb292, portraits) and
//! the weapon definitions' icon (h_e5ec3f1f); like all the game's textures they are stored
//! upside down, so they are drawn flipped. Text uses Bevy's font, not the game's bitmap font.

use super::*;
use super::text::{game_text, Align, GameText, AMMO_ORANGE, ITEM_RED, MESSAGE_BLUE};

/// radar frame (octagon) and its disc
const RADAR_FRAME: u32 = 0x132E_4EE1;
const RADAR_DISC: u32 = 0x17D0_EE37;
/// weapon panel background (128 x 32)
const WEAPON_PANEL: u32 = 0xE11F_B292;
const CROSSHAIR_TEX: u32 = 0x1D2A_68D0;
const ENERGY_ICON: u32 = 0x1AC4_1530;
/// the Frag item's own HUD icon (its definition's h_e5ec3f1f; the ribbed grenade in the capture),
/// for a grenade type without one
const FRAG_ICON: u32 = 0xFE20_B919;
/// The item box's count: top right (the medkit captures), or for a grenade at the right
/// middle (the Frag and Light recordings: the digit's right edge at 588, its top at 410-412; the
/// line's top is ~3 units above the digit's: the reference agent measured the digit at
/// y 410-421 in the recording, 406-418 with the line at 403), and hidden while only one is carried (Light, Sentry recordings).
const ITEM_COUNT_Y: f32 = 378.0;
/// A grenade's icon in the item box (left, top, size, units): the Frag recording's icon spans x 553-568,
/// y 384-425; drawn 50 units square at (535, 379) it spanned x 554-566, y 385-418, so it's
/// drawn 1.25x larger, its middle on the recording's.
const ITEM_ICON: (f32, f32, f32) = (529.3, 376.4, 62.5);
/// The medkit's icon (left, top, size, units), fitted to the medkit captures (todo/medkits.mp4).
const MEDKIT_ICON_AT: (f32, f32, f32) = (535.0, 379.0, 50.0);
const GRENADE_COUNT_Y: f32 = 407.0;
/// The item's name (the line's top): the medkit captures', and a grenade's a little lower (the
/// Frag and Light recordings: its capitals' tops at 423-425).
const ITEM_NAME_Y: f32 = 417.0;
const GRENADE_NAME_Y: f32 = 420.0;
/// The item list's slots: ITEM_SLOTS_LEFT leftward along the bottom (the items that aren't
/// grenades), the rest up the box's column (the other grenade types: the Frag recording's
/// inventory overview and the Sentry one's).
const ITEM_SLOTS_LEFT: usize = 2;
const ITEM_SLOTS: usize = 7;
/// the Medkit's HUD icon (its item type's h_e5ec3f1f), if the carried type has none
const MEDKIT_ICON: u32 = 0xF647_BBEF;
/// The item box (top left, units) and the step between the item list's slots (capture: the
/// slots 52 px apart at 480 px across, 69 units; a box is 61).
const ITEM_BOX: (f32, f32) = (530.0, 376.0);
const SLOT_STEP: f32 = 67.0;

/// The target ring: its size (units) and tint (capture: blue, ~63 units across at the panel).
const RING_SIZE: f32 = 60.0;
const RING_BLUE: Color = Color::srgba(0.16, 0.42, 1.0, 0.95);
/// Pickup lines shown at once, and where they go: fixed on the screen, centred under the
/// character's usual place, the status message above them (the capture's spacing: lines 29
/// and 45 under the message).
const FEED_LINES: usize = 4;
const MESSAGE_Y: f32 = 290.0;
const FEED_FIRST: f32 = 29.0;
const FEED_STEP: f32 = 16.5;
/// Strings: "Hold ", "to activate %s.", "NEW", "switch to %s." (after the Y button's icon),
/// "Regen".
const S_HOLD: u32 = 0x1CC5_1F04;
const S_TO_ACTIVATE: u32 = 0xF9D2_9692;
const S_NEW: u32 = 0x0BA4_1874;
const S_SWITCH_TO: u32 = 0xFF01_7DF6;
const S_REGEN: u32 = 0xE246_DE1D;
/// "[Y] switch to %s." once the held gun's clip and the reserve are both empty (FUN_00120d40
/// posts message 0x52): top left under the health bar, pale blue (todo/49 take02 19 s; the same
/// line as the pickup prompt). Here the key (Q) stands for the button's icon, as E does in the
/// panel prompt. Measured there: "switch"'s letters span y 90-104 (14 units; with CAP 11.5 ours
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
/// the small tab beside each portrait: the follow-order arrows, or the speech icon while that
/// member is speaking
const TAB_CENTRE: [(f32, f32); 4] = [(144.8, 423.3), (178.8, 345.0), (67.2, 389.3), (100.5, 312.7)];
const SQUAD_ICON: u32 = 0xE064_867A;
const PLAYER_ICON: u32 = 0xEEAA_5549;
const HEALTH_ICON: u32 = 0x1BFA_5678;
/// bar fills (4 x 8 gradients): health red, energy blue
const HEALTH_FILL: u32 = 0xE801_997B;
/// grenade reticle (two brackets) shown while a throw charges
const GRENADE_RETICLE: u32 = 0x0EF1_1EE8;
/// The charge meter (the recordings, every grenade): a blue outlined bar x 354.0-367.7,
/// y 162.0-221.7 beside the reticle brackets; its fill x 355.3-365.7 grows linearly from
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
/// radar: centre of the disc, its radius in screen units and the range it shows (m)
const RADAR_CENTRE: (f32, f32) = (121.5, 369.0);
const RADAR_RADIUS: f32 = 44.0;
const RADAR_RANGE: f32 = 40.0;
const SQUAD_YELLOW: Color = Color::srgb(1.0, 0.88, 0.2);
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
/// where the radar frame sits (screen units, 128 x 128)
const RADAR_FRAME_AT: (f32, f32) = (57.5, 305.0);
const HEALTH_PINK: Color = Color::srgb(0.96, 0.45, 0.42);
/// the selected member's name: shown during the selection, then at the camera cut it zooms to
/// 3x and fades out over this long (capture)
const NAME_ZOOM_TIME: f32 = 0.4;
/// ... and swells to this many times its size as it fades (exaggerated from the game's ~3x)
const NAME_ZOOM_TO: f32 = 6.0;
/// The name is drawn this much larger than the capture's, and pops in: from NAME_POP times its
/// size down to it over NAME_POP_TIME as the selection starts.
const NAME_BIG: f32 = 1.6;
const NAME_POP: f32 = 1.9;
const NAME_POP_TIME: f32 = 0.15;
/// the HUD font atlas (stored upright; glyph rows in ASCII order) and the capital letters' rows
const FONT_ATLAS: u32 = 0xE0AF_CD52;
const FONT_ROWS: [(usize, &str); 3] = [(1, "<=>?@ABCDEF"), (2, "GHIJKLMNOPQ"), (3, "RSTUVWXYZ[")];
/// name size: screen units per atlas pixel (capture: "FLINT" 18 units tall, 21 px glyphs),
/// colour, and centre
const NAME_SCALE: f32 = 18.0 / 21.0;
const NAME_ORANGE: Color = Color::srgb(0.93, 0.56, 0.3);
const NAME_CENTRE_Y: f32 = 238.0;
const NAME_SLOTS: usize = 8;
/// the selection arrow: the game's chevron texture (158f87c1, drawn mirrored / turned to point at
/// the portrait), three of them 5 units apart, 30 units square, the first 28 units short of the
/// portrait's centre (capture), red, lit one after another
const CHEVRON: u32 = 0x158F_87C1;
const CHEVRON_RED: Color = Color::srgb(1.0, 0.33, 0.2);
/// speech icons: beside each portrait, on the side away from its command tab (by character)
const BUBBLE_AT: [(f32, f32); 4] = [(96.0, 426.0), (176.0, 383.0), (45.0, 344.0), (136.0, 292.0)];
const FRIEND_GREEN: Color = Color::srgba(0.3, 1.0, 0.35, 0.95);
/// a dead member's portrait: their own skull (the character's h_00c51907 icon; Brutus's is the
/// wide one), dimmed; their command tab and radar health bar go. Fallbacks if the data lacks one:
const SKULL_HUMAN: u32 = 0x1834_3E47;
const SKULL_BRUTUS: u32 = 0x19DA_7869;
const SKULL_TINT: Color = Color::srgb(0.6, 0.66, 0.6);

/// The scope frame (h_1807ec04, 256 x 128: the top right quarter) and its size on screen: its
/// rim where the capture shows it (77% of the width, nearly the full height), the quarters
/// running past the screen's edge.
const SCOPE_FRAME: u32 = 0x1807_EC04;
/// The crosshair: the held weapon's 64 x 64 texture drawn 1:1, centred at (320, 192); in the
/// scope it moves to the middle, (320, 240), the same size. Measured for all 11 weapons in their
/// take01s (todo/49-68, 2.5 s: each texture's alpha box matches the screen box to 1-2 units),
/// the same standing, firing and strafing (no growth); scoped in the Foley's, MK's and L-Shot's
/// (both zoom steps).
const CROSSHAIR_SIZE: f32 = 64.0;
const CROSSHAIR_Y: f32 = 192.0;
const SCOPE_QUARTER: (f32, f32) = (397.0, 269.0);
/// the frame's grey (153 155 153, alpha 153) drawn as a darkening, and the same beyond it
const SCOPE_SHADE: Color = Color::srgb(0.12, 0.13, 0.12);
const SCOPE_FILL_COLOR: Color = Color::srgba(0.02, 0.02, 0.02, 0.6);
const SCOPE_FILL: f32 = 400.0;

/// The health / energy frame's top-left corner (common h_0cdf5876, 32 x 16: the notched outer
/// edge and the bars' inset channel), mirrored into the other three, its last column stretched
/// between; and the glossy see-through panel under the grenade (h_10891a71, 64 x 32, cut top-left
/// corner), drawn 9-sliced (corners PANEL_SLICE texels).
const BAR_FRAME_CORNER: u32 = 0x0CDF_5876;
const ITEM_PANEL: u32 = 0x1089_1A71;
const PANEL_SLICE: f32 = 10.0;
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

/// An image the right way up (HUD textures are stored bottom row first).
fn upright(w: u32, h: u32, px: Vec<u8>) -> Image {
    let row = (w * 4) as usize;
    let flipped: Vec<u8> = px.chunks_exact(row).rev().flatten().copied().collect();
    Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, flipped,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
}

/// The original's health / stamina frame: the tutorial's picture of it (tutorial h_17f34cbe,
/// 256 x 64, the + and bolt icons, the frame with its corner tabs and divider, both bars baked
/// in). Cut in three across: the left cap with the icons, a middle that stretches, the right cap.
/// Its baked bars are painted the empty-bar navy it shows itself (the stamina bar's empty end)
/// and the live bars drawn over them. Upright texel columns and rows:
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
const TB_EMPTY: [u8; 4] = [0, 20, 107, 255];
/// where the picture's icons and frame go on the 640 x 480 screen (their top-left, native size: its body
/// is 38 texels high, the old frame's 37)
const TB_AT: (f32, f32) = (52.0, 39.0);
/// The bars' length for a character's maximum health: Tex's 115 gives TB_FULL (the frame's
/// length to height then matches a capture, 5.4; the picture's own bars are 144 texels).
const TB_FULL: f32 = 184.0;
const TB_FULL_HEALTH: f32 = 115.0;
/// how opaque the frame is drawn (the picture is opaque, baked on the tutorial's screen; the
/// capture's frame is glass, the sky showing through)
const TB_ALPHA: f32 = 0.45;

/// The bars' length (screen units) for a maximum health.
fn bar_length(max_health: f32) -> f32 {
    TB_FULL * max_health.max(1.0) / TB_FULL_HEALTH
}

/// The tutorial picture's pieces (icons, left, middle, right), upright, the bars emptied.
fn tutorial_frame(w: u32, h: u32, px: &[u8]) -> [Image; 4] {
    let at = |x: u32, y: u32| -> [u8; 4] {
        let in_bar = TB_BAR_ROWS.iter().any(|&(a, b)| (a..b).contains(&y)) && (TB_BAR_COLS.0..TB_BAR_COLS.1).contains(&x);
        if in_bar {
            return TB_EMPTY;
        }
        let i = (((h - 1 - y) * w + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    [TB_ICONS, TB_LEFT, TB_MIDDLE, TB_RIGHT].map(|(x0, x1)| {
        let mut out = vec![];
        for y in TB_ROWS.0..TB_ROWS.1 {
            for x in x0..x1 {
                out.extend_from_slice(&at(x, y));
            }
        }
        Image::new(bevy::render::render_resource::Extent3d { width: x1 - x0, height: TB_ROWS.1 - TB_ROWS.0, depth_or_array_layers: 1 },
                   bevy::render::render_resource::TextureDimension::D2, out,
                   bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
    })
}

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

/// A scope overlay: the frame (false) or Flint's sniper rim (true).
#[derive(Component)]
struct ScopeOverlay(bool);

/// Flint's sniper view: clear in the middle, a cyan-teal glow toward the edges (the capture's
/// look, approximated: the game smears the edges with an effect).
fn sniper_rim() -> Image {
    let n = 256u32;
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let (u, v) = (x as f32 / (n - 1) as f32 * 2.0 - 1.0, y as f32 / (n - 1) as f32 * 2.0 - 1.0);
            let r = (u * u + v * v).sqrt();
            let k = ((r - 0.7) / 0.6).clamp(0.0, 1.0);
            let a = k * k * (3.0 - 2.0 * k) * 0.5;
            px.extend_from_slice(&[40, 150, 170, (a * 255.0) as u8]);
        }
    }
    Image::new(bevy::render::render_resource::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, px,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
}

/// Show the scope's overlay while the scope is up: Flint's rim for her, the frame for the others.
fn update_scope(player: Res<Player>, mut overlays: Query<(&ScopeOverlay, &mut Visibility)>) {
    let up = player.scope > 0.6;
    let flint = player.character == STILL_SNIPER;
    for (o, mut v) in &mut overlays {
        let want = if up && o.0 == flint { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

/// A portrait's own image (to show again after the skull).
#[derive(Component)]
struct PortraitArt(Handle<Image>, Rect);

/// Glyph rectangles (atlas pixels) of the capital letters: the atlas cut at empty rows and
/// columns; rows 1-3 hold `<=>?@ABCDEF`, `GHIJKLMNOPQ`, `RSTUVWXYZ[`.
fn font_glyphs(w: u32, h: u32, px: &[u8]) -> HashMap<char, Rect> {
    let solid = |x: u32, y: u32| px[((y * w + x) * 4 + 3) as usize] > 40;
    let mut bands = vec![];
    let mut y = 0;
    while y < h {
        if (0..w).any(|x| solid(x, y)) {
            let y0 = y;
            while y < h && (0..w).any(|x| solid(x, y)) { y += 1; }
            bands.push((y0, y));
        }
        y += 1;
    }
    let mut out = HashMap::new();
    for (row, chars) in FONT_ROWS {
        let Some(&(y0, y1)) = bands.get(row) else { continue };
        let mut cols = vec![];
        let mut x = 0;
        while x < w {
            if (y0..y1).any(|y| solid(x, y)) {
                let x0 = x;
                while x < w && (y0..y1).any(|y| solid(x, y)) { x += 1; }
                cols.push((x0, x));
            }
            x += 1;
        }
        if cols.len() != chars.chars().count() {
            continue;
        }
        for (c, (x0, x1)) in chars.chars().zip(cols) {
            // tight vertical extent of this glyph
            let ys: Vec<u32> = (y0..y1).filter(|&y| (x0..x1).any(|x| solid(x, y))).collect();
            let (ty0, ty1) = (ys[0], ys[ys.len() - 1] + 1);
            out.insert(c, Rect::new(x0 as f32, ty0 as f32, x1 as f32, ty1 as f32));
        }
    }
    out
}

/// Generated HUD images: the squad health bars, the selection chevrons, the name font and the
/// speech icon.
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
    font: Handle<Image>,
    glyphs: HashMap<char, Rect>,
    /// speech icon
    bubble: Handle<Image>,
    /// skulls for the dead, by character
    skulls: [Handle<Image>; 4],
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
    Tab(usize),
    Crosshair,
    /// "Hold E to activate panel." while a gate's wall panel is in reach and in view
    UsePrompt,
    /// "Q switch to <other weapon>." while the held gun and its reserve are empty
    SwitchHint,
    /// the item box (bottom right, while anything's carried): its panel, the grenade's icon, the
    /// item's name, how many, and NEW for a newly taken kind; the item list above it (Tab held)
    ItemBox,
    ItemIcon,
    ItemName,
    ItemCount,
    ItemNew,
    /// the item list around the box while Tab is held (todo/medic + intenvory use case.mp4):
    /// slots 0, 1 the other items leftward along the bottom, 2.. the other grenade types up the
    /// right edge (see ITEM_SLOTS); each its icon, name and count
    Slot(usize, SlotPart),
    /// the status message at the character ("No need to heal", "Tex cannot pick up Medkit.")
    /// and the pickup lines under it
    Message,
    Feed(usize),
    /// the blue target ring on a usable panel's button
    Ring,
}

#[derive(Clone, Copy, PartialEq)]
enum SlotPart {
    Icon,
    Name,
    Count,
}

/// Where item list slot `k` sits (its box's top left, units).
fn slot_at(k: usize) -> (f32, f32) {
    if k < ITEM_SLOTS_LEFT {
        (ITEM_BOX.0 - SLOT_STEP * (k + 1) as f32, ITEM_BOX.1)
    } else {
        (ITEM_BOX.0, ITEM_BOX.1 - SLOT_STEP * (k + 1 - ITEM_SLOTS_LEFT) as f32)
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
    /// the stamina bar, and the frame's stretching middle and its right cap (the tutorial
    /// picture's frame: they follow the character's maximum health, see `bar_length`)
    PlayerStamina,
    FrameMiddle,
    FrameRight,
    /// the selection's three chevrons on the radar
    Chevron(usize),
    /// the chosen member's name in the game's font: letter slots
    NameGlyph(usize),
    /// speech icon beside a member's portrait while they talk (by character)
    Bubble(usize),
    /// portrait (by character): dimmed when dead
    Portrait(usize),
    /// that member's name over their head (window space)
    SelectName,
    MeterBox,
    MeterFill,
    MeterTick,
    Reticle,
}

#[derive(Resource, Default)]
struct HudImages(HashMap<u32, Option<Handle<Image>>>);

/// The selected member's name label: the member being selected and how long their name has
/// shown, then, after the cut, the member and how far into the zoom-out it is. A resource, reset
/// with each level, so a label doesn't carry into the next one.
#[derive(Resource, Default)]
struct NameLabel {
    pending: Option<(usize, f32)>,
    zooming: Option<(usize, f32)>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<HudImages>()
        .init_resource::<NameLabel>()
        .add_systems(OnEnter(AppState::Playing), (setup_hud.after(snapshot_entities),
            (|mut commands: Commands| commands.insert_resource(NameLabel::default())).after(setup)))
        .init_resource::<HudGen>()
        // (after the death camera: its label and reticle show the frame it decides them)
        .add_systems(Update, (update_hud_widgets, update_squad_hud, update_scope).after(update_weapons).after(deathcam::death_cam)
            .run_if(in_state(AppState::Playing)));
}

/// A game texture as a UI image (cached; None if the texture isn't found).
fn texture(game: &mut Game, images: &mut Assets<Image>, cache: &mut HudImages, name: u32) -> Option<Handle<Image>> {
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
/// while the item can't be used (capture: the medkit's at full health).
fn texture_grey(game: &mut Game, images: &mut Assets<Image>, cache: &mut HudImages, name: u32) -> Option<Handle<Image>> {
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
fn flipped(h: Handle<Image>) -> ImageNode {
    ImageNode { flip_y: true, ..ImageNode::new(h) }
}

/// Absolute placement in 640 x 480 screen units (as `at`, for moving nodes).
fn at_box(x: f32, y: f32, w: f32, h: f32) -> Node {
    at(x, y, w, h)
}

/// Absolute placement in 640 x 480 screen units.
fn at(x: f32, y: f32, w: f32, h: f32) -> Node {
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
    if let Some((w, h, px)) = game.0.texture_rgba(FONT_ATLAS) {
        gen.glyphs = font_glyphs(w, h, &px);
        gen.font = images.add(Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                                         bevy::render::render_resource::TextureDimension::D2, px,
                                         bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                         bevy::asset::RenderAssetUsages::default()));
        info!("HUD font: {} capital glyphs", gen.glyphs.len());
    }
    let (chevrons, font, bars) = (gen.chevrons.clone(), gen.font.clone(), gen.bars.clone());
    let game = &mut game.0;
    let rim = images.add(sniper_rim());
    game.load_textures_for(&super::data_dir(), "tutorial", TUTORIAL_BAR);
    let tutorial = game.texture_rgba(TUTORIAL_BAR).map(|(w, h, px)| tutorial_frame(w, h, &px).map(|i| images.add(i)));
    let bar_frame = game.texture_rgba(BAR_FRAME_CORNER).map(|(w, h, px)| images.add(bar_frame(w, h, &px)));
    let item_panel = game.texture_rgba(ITEM_PANEL).map(|(w, h, px)| images.add(upright(w, h, px)));
    let mut tex = |name: u32| texture(game, &mut images, &mut cache, name);
    let (frame, disc, panel, cross, bolt, frag) =
        (tex(RADAR_FRAME), tex(RADAR_DISC), tex(WEAPON_PANEL), tex(CROSSHAIR_TEX), tex(ENERGY_ICON), tex(FRAG_ICON));
    let image = |h: &Option<Handle<Image>>, color: Color| flipped(h.clone().unwrap_or_default()).with_color(color);

    // full window, with the game's 4:3 screen centred in it
    let window = commands.spawn(Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                       justify_content: JustifyContent::Center, ..default() }).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();

    // ---- the scope (under the rest of the HUD): for most, the game's frame h_1807ec04, a
    // quarter (top right: outer grey, rounded corner, tick marks) mirrored four ways, the
    // shading outside drawn dark as in the capture; Flint's sniper view has a cyan rim instead
    // (an effect in the game, not a texture: approximated) ----
    let quarter = tex(SCOPE_FRAME);
    let (qw, qh) = SCOPE_QUARTER;
    for (x, y, fx, fy) in [(320.0, 240.0 - qh, false, false), (320.0 - qw, 240.0 - qh, true, false),
                           (320.0, 240.0, false, true), (320.0 - qw, 240.0, true, true)] {
        commands.spawn((ChildOf(screen), at(x, y, qw, qh), ScopeOverlay(false), Visibility::Hidden,
                        ImageNode { flip_x: fx, flip_y: fy, ..ImageNode::new(quarter.clone().unwrap_or_default()) }.with_color(SCOPE_SHADE)));
    }
    // beyond the frame's texture (wide windows): the same shading
    for (x, w) in [(-SCOPE_FILL, 320.0 - qw + SCOPE_FILL), (320.0 + qw, SCOPE_FILL)] {
        commands.spawn((ChildOf(screen), at(x, 240.0 - qh, w, 2.0 * qh), ScopeOverlay(false), Visibility::Hidden,
                        BackgroundColor(SCOPE_FILL_COLOR)));
    }
    // (over the whole window, not just the 4:3 screen)
    commands.spawn((ChildOf(window), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                    ScopeOverlay(true), Visibility::Hidden, ImageNode::new(rim)));

    // ---- health (red) and energy (blue), top left: in the game's glossy frame ----
    let (red, blue, plus) = (tex(HEALTH_FILL), tex(ENERGY_FILL), tex(HEALTH_ICON));
    let (x0, y0, x1, y1) = BAR_FILL_RECT;
    let fill = |h: &Option<Handle<Image>>| ImageNode { rect: Some(Rect::new(x0, y0, x1, y1)), ..image(h, Color::WHITE) };
    gen.tutorial_frame = tutorial.is_some();
    if let Some([icons, left, middle, right]) = tutorial {
        // the original's frame (see TUTORIAL_BAR): left cap with the icons, the middle stretched
        // and the right cap placed for the character (update_squad_hud), the bars over it
        let glass = Color::srgba(1.0, 1.0, 1.0, TB_ALPHA);
        let piece = |(a, b): (u32, u32)| (b - a) as f32;
        let (top, high) = (TB_AT.1, (TB_ROWS.1 - TB_ROWS.0) as f32);
        let left_x = TB_AT.0 + piece(TB_ICONS);
        let mid_x = left_x + piece(TB_LEFT);
        commands.spawn((ChildOf(screen), at(TB_AT.0, top, piece(TB_ICONS), high), ImageNode::new(icons)));
        commands.spawn((ChildOf(screen), at(left_x, top, piece(TB_LEFT), high), ImageNode::new(left).with_color(glass)));
        commands.spawn((ChildOf(screen), at(mid_x, top, piece(TB_MIDDLE), high), ImageNode::new(middle).with_color(glass), SquadPart::FrameMiddle));
        commands.spawn((ChildOf(screen), at(mid_x + piece(TB_MIDDLE), top, piece(TB_RIGHT), high), ImageNode::new(right).with_color(glass),
                        SquadPart::FrameRight));
        let bar_x = TB_AT.0 + (TB_BAR_COLS.0 - TB_ICONS.0) as f32;
        let row = |i: usize| (TB_AT.1 + (TB_BAR_ROWS[i].0 - TB_ROWS.0) as f32, (TB_BAR_ROWS[i].1 - TB_BAR_ROWS[i].0) as f32);
        let ((hy, hh), (sy, sh)) = (row(0), row(1));
        let full = piece(TB_BAR_COLS);
        commands.spawn((ChildOf(screen), at(bar_x, hy, full, hh), fill(&red), SquadPart::PlayerHealth));
        commands.spawn((ChildOf(screen), at(bar_x, sy, full, sh), fill(&blue), SquadPart::PlayerStamina));
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
        commands.spawn((ChildOf(screen), at(463.0, y - 25.0, 128.0, 64.0), flipped(Handle::default()), Part::Icon(row), Visibility::Hidden));
        commands.spawn((ChildOf(screen), game_text("", WEAPON_NAME_X, y - WEAPON_NAME_UP, WEAPON_NAME_CAP, AMMO_ORANGE, Align::Left), Part::Name(row), Visibility::Hidden));
        commands.spawn((ChildOf(screen), game_text("", 475.0, y + 9.0, 14.0, AMMO_ORANGE, Align::Left), Part::Ammo(row), Visibility::Hidden));
    }

    // ---- radar, bottom left (centre 121, 368): disc, view cone, frame, player, portrait; in
    // deathmatch there's no squad, and no radar (capture) ----
    let squad = !mode.deathmatch;
    if squad {
        commands.spawn((ChildOf(screen), at(51.0, 298.5, 141.0, 141.0), image(&disc, Color::srgba(1.0, 1.0, 1.0, 0.85))));
        commands.spawn((ChildOf(screen), at(75.0, 322.5, 93.0, 93.0), ImageNode::new(cone)));
        commands.spawn((ChildOf(screen), at(57.5, 305.0, 128.0, 128.0), image(&frame, Color::WHITE)));
        commands.spawn((ChildOf(screen), at(119.5, 367.0, 4.0, 4.0), BackgroundColor(Color::srgb(0.55, 1.0, 0.35)),
                                  BorderRadius::all(Val::Percent(50.0))));
        // the frame's four portrait notches are open in its texture; the game fills them dark behind
        // the faces (texel boxes in the frame texture: top, right, bottom, left)
        for (x0, y0, x1, y1) in [(50.0, 8.0, 80.0, 17.0), (110.0, 50.0, 119.0, 80.0), (48.0, 110.0, 78.0, 119.0), (8.0, 48.0, 17.0, 78.0)] {
            commands.spawn((ChildOf(screen), at(57.5 + x0, 305.0 + y0, x1 - x0, y1 - y0), BackgroundColor(Color::srgba(0.16, 0.2, 0.45, 0.9))));
        }
        for (i, &(cx, cy)) in PORTRAIT_CENTRE.iter().enumerate() {
            let Some((w, h, px)) = game.texture_rgba(PORTRAITS[i]) else { continue };
            // opaque bounding box (texture rows as stored)
            let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
            for (k, p) in px.chunks_exact(4).enumerate() {
                if p[3] > 40 {
                    let (x, y) = (k as u32 % w, k as u32 / w);
                    (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
                }
            }
            if x1 <= x0 || y1 <= y0 { continue }
            let handle = texture(game, &mut images, &mut cache, PORTRAITS[i]).unwrap_or_default();
            let (bw, bh) = ((x1 - x0) as f32, (y1 - y0) as f32);
            let rect = Rect::new(x0 as f32, y0 as f32, x1 as f32, y1 as f32);
            commands.spawn((ChildOf(screen), at(cx - bw / 2.0, cy - bh / 2.0, bw, bh),
                            ImageNode { rect: Some(rect), ..flipped(handle.clone()) }, ZIndex(1),
                            SquadPart::Portrait(i), PortraitArt(handle, rect)));
        }
        for (i, &(cx, cy)) in TAB_CENTRE.iter().enumerate() {
            commands.spawn((ChildOf(screen), at(cx - 5.0, cy - 5.0, 10.0, 10.0), flipped(Handle::default()), Part::Tab(i)));
        }
    }

    // ---- the item box, bottom right: one item at a time (the game's B button steps through
    // them), on the game's glossy panel (its cut corner top left). The captures: a grenade with
    // its icon; a medkit as its name in red at the bottom, the count top right, NEW (orange)
    // over the name when newly taken ----
    let item_box = commands.spawn((ChildOf(screen), at(530.0, 376.0, 61.0, 61.0), Part::ItemBox, Visibility::Hidden)).id();
    match item_panel.clone() {
        Some(p) => {
            commands.spawn((ChildOf(item_box), Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, ImageNode {
                image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(PANEL_SLICE), center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 1.0 }),
                ..ImageNode::new(p)
            }));
        }
        None => {
            commands.spawn((ChildOf(item_box), BackgroundColor(Color::srgba(0.10, 0.22, 0.50, 0.40)), BorderColor(HUD_BLUE),
                            Node { border: UiRect::all(Val::Px(1.0)), width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
        }
    }
    commands.spawn((ChildOf(screen), at(ITEM_ICON.0, ITEM_ICON.1, ITEM_ICON.2, ITEM_ICON.2), image(&frag, Color::WHITE), Part::ItemIcon, Visibility::Hidden));
    commands.spawn((ChildOf(screen), game_text("", 561.0, ITEM_NAME_Y, 12.0, ITEM_RED, Align::Centre), Part::ItemName, Visibility::Hidden));
    commands.spawn((ChildOf(screen), game_text("", 588.0, ITEM_COUNT_Y, 12.0, ITEM_RED, Align::Right), Part::ItemCount, Visibility::Hidden));
    commands.spawn((ChildOf(screen), game_text("", 545.0, 401.0, 12.0, AMMO_ORANGE, Align::Left), Part::ItemNew, Visibility::Hidden));
    for k in 0..ITEM_SLOTS {
        let (x, y) = slot_at(k);
        commands.spawn((ChildOf(screen), at(x + 5.0, y + 3.0, 50.0, 50.0), image(&frag, Color::WHITE), Part::Slot(k, SlotPart::Icon), Visibility::Hidden));
        commands.spawn((ChildOf(screen), game_text("", x + 31.0, y + 41.0, 12.0, MESSAGE_BLUE, Align::Centre), Part::Slot(k, SlotPart::Name), Visibility::Hidden));
        commands.spawn((ChildOf(screen), game_text("", x + 58.0, y + 2.0, 12.0, MESSAGE_BLUE, Align::Right), Part::Slot(k, SlotPart::Count), Visibility::Hidden));
    }

    // ---- a gate's wall panel in reach: "Hold E to activate panel." under the health bar (the
    // game's "Hold " + its button icon + "to activate %s."; here the key), and the blue
    // target ring (the reticle texture h_1d2a68d0) on the panel's button ----
    commands.spawn((ChildOf(screen), game_text("", 52.0, 68.0, 11.5, MESSAGE_BLUE, Align::Left), Part::UsePrompt, Visibility::Hidden));
    commands.spawn((ChildOf(screen), game_text("", SWITCH_HINT_X, SWITCH_HINT_TOP, SWITCH_HINT_CAP, MESSAGE_BLUE, Align::Left), Part::SwitchHint, Visibility::Hidden));
    let ring = texture(game, &mut images, &mut cache, CROSSHAIR_TEX);
    commands.spawn((ChildOf(screen), at(0.0, 0.0, RING_SIZE, RING_SIZE), image(&ring, RING_BLUE), Part::Ring, Visibility::Hidden));

    // ---- below the middle of the screen: the status message, the pickup lines under it ----
    commands.spawn((ChildOf(screen), game_text("", 320.0, MESSAGE_Y, 11.5, MESSAGE_BLUE, Align::Centre), Part::Message, Visibility::Hidden));
    for i in 0..FEED_LINES {
        commands.spawn((ChildOf(screen), game_text("", 320.0, MESSAGE_Y + FEED_FIRST + FEED_STEP * i as f32, 11.5, MESSAGE_BLUE, Align::Centre),
                        Part::Feed(i), Visibility::Hidden));
    }

    // ---- crosshair, above centre (320, 192) ----
    // the held weapon's own crosshair (its definition's reticule-prefix texture), 64 units across
    commands.spawn((ChildOf(screen), at(320.0 - CROSSHAIR_SIZE / 2.0, CROSSHAIR_Y - CROSSHAIR_SIZE / 2.0, CROSSHAIR_SIZE, CROSSHAIR_SIZE),
                    image(&cross, Color::srgba(0.2, 0.55, 1.0, 0.95)), Part::Crosshair));

    // ---- grenade throw: bracket reticle and the charge meter right of it (capture: box 354..367 x
    // 163..221, orange fill from the bottom, dark tick at ~72%) ----
    let reticle = texture(game, &mut images, &mut cache, GRENADE_RETICLE);
    commands.spawn((ChildOf(screen), at(302.0, 169.0, 36.0, 36.0), image(&reticle, Color::srgba(0.2, 0.55, 1.0, 0.95)),
                    SquadPart::Reticle, Visibility::Hidden));
    commands.spawn((ChildOf(screen), SquadPart::MeterBox, Visibility::Hidden, BorderColor(HUD_BLUE), BackgroundColor(Color::srgba(0.0, 0.05, 0.15, 0.25)),
                    Node { border: UiRect::all(Val::Px(1.5)), ..at(METER_BOX.0, METER_BOX.1, METER_BOX.2 - METER_BOX.0, METER_BOX.3 - METER_BOX.1) }));
    commands.spawn((ChildOf(screen), SquadPart::MeterFill, Visibility::Hidden, BackgroundColor(METER_ORANGE),
                    at(METER_FILL.0, METER_FILL.3, METER_FILL.1 - METER_FILL.0, 0.0)));
    commands.spawn((ChildOf(screen), SquadPart::MeterTick, Visibility::Hidden, BackgroundColor(Color::srgb(0.08, 0.12, 0.25)),
                    at(METER_FILL.0, METER_TICK_Y, METER_FILL.1 - METER_FILL.0, 1.5)));

    // ---- squad: radar blips, selection marker, name over the chosen member ----
    if !squad {
        return;
    }
    for slot in 0..3 {
        commands.spawn((ChildOf(screen), SquadPart::Blip(slot), Visibility::Hidden, BackgroundColor(SQUAD_YELLOW),
                        BorderRadius::all(Val::Percent(50.0)), ZIndex(2), at(0.0, 0.0, 4.0, 4.0)));
    }
    // three red chevrons on the radar pointing at the portrait of the member being given control
    for k in 0..3 {
        commands.spawn((ChildOf(screen), SquadPart::Chevron(k), Visibility::Hidden, ImageNode::new(chevrons[0].clone()).with_color(CHEVRON_RED),
                        ZIndex(2 + k as i32), at(0.0, 0.0, 30.0, 30.0)));
    }
    // the chosen member's name in the game's font, in the middle of the screen (capture: centred
    // at 320, 238); a row of letter images, bottom-aligned
    let name_box = commands.spawn((ChildOf(screen), SquadPart::SelectName, Visibility::Hidden, ZIndex(3),
                                   Node { justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                          column_gap: Val::Px(2.0), ..at(20.0, NAME_CENTRE_Y - 40.0, 600.0, 80.0) })).id();
    for k in 0..NAME_SLOTS {
        commands.spawn((ChildOf(name_box), SquadPart::NameGlyph(k), Visibility::Hidden,
                        ImageNode::new(font.clone()).with_color(NAME_ORANGE), Node::default()));
    }
    for (c, &(x, y)) in BUBBLE_AT.iter().enumerate() {
        commands.spawn((ChildOf(screen), SquadPart::Bubble(c), Visibility::Hidden, flipped(Handle::default()), ZIndex(2), at(x, y, 10.0, 10.0)));
    }
    // squad health in the radar frame's diagonal channels (the frame's dark channel is the track)
    for (c, bar) in bars.into_iter().enumerate() {
        commands.spawn((ChildOf(screen), SquadPart::Health(c), ZIndex(1), at(RADAR_FRAME_AT.0, RADAR_FRAME_AT.1, 128.0, 128.0),
                        ImageNode::new(bar).with_color(HEALTH_PINK)));
    }
}

/// Weapon rows (the held weapon first; every weapon with names for a moment after a switch, the
/// held one's name for a moment after a pickup), ammo ("clip / reserve", the squad's reserve of
/// its ammo-type; "clip Regen" for a recharging gun), red panel on an empty clip, the switch
/// hint, portrait, crosshair.
#[allow(clippy::too_many_arguments)]
fn update_hud_widgets(
    player: Res<Player>,
    reserve: Res<super::ammo::Reserve>,
    held_name: Res<super::ammo::HeldName>,
    switch_hint: Res<super::ammo::SwitchHint>,
    squad: Res<Squad>,
    use_panel: Res<super::UsePanel>,
    feed: Res<super::pickups::PickupFeed>,
    mut game: ResMut<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<HudImages>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut parts: Query<(&Part, &mut Visibility, Option<&mut ImageNode>, Option<&mut GameText>, Option<&mut Node>)>,
    kits: Option<Res<super::grenade::GrenadeKits>>,
) {
    let Some(l) = player.loaded.as_ref() else { return };
    let list = player.hud_list > 0.0;
    let mut order: Vec<usize> = (0..l.weapons.len()).collect();
    order.sort_by_key(|&i| i != player.weapon);
    order.truncate(if list { WEAPON_ROWS } else { 1 });
    // a world point on the HUD's 640 x 480 screen (4:3, centred in the window)
    let on_screen = |p: Vec3| -> Option<Vec2> {
        let (cam, at) = camera.single().ok()?;
        let win = windows.single().ok()?;
        let px = cam.world_to_viewport(at, p).ok()?;
        let unit = win.height() / 480.0;
        Some(Vec2::new((px.x - (win.width() - 640.0 * unit) / 2.0) / unit, px.y / unit))
    };
    // the item box: what's carried, the selected one (a grenade type: its definition's label
    // and HUD icon h_e5ec3f1f)
    let carried: Vec<Item> = super::items(&player).into_iter().filter(|&i| super::item_count(&player, i) > 0).collect();
    let item = player.item;
    let count = super::item_count(&player, item);
    let kit = |k: usize| kits.as_ref().and_then(|g| g.0.get(k));
    let name_of = |game: &GameData, i: Item| match i {
        Item::Grenade(k) => kit(k).map_or("Frag".to_string(), |g| g.def.label.clone()),
        Item::Medkit => game.0.items.get(&player.medkit_kind).map(|t| t.label.clone()).filter(|s| !s.is_empty()).unwrap_or("Medkit".into()),
    };
    // pale while it can be used, red while it can't (capture: a medkit at full health)
    let usable = super::item_usable(&player, item);
    let item_color = if usable { MESSAGE_BLUE } else { ITEM_RED };
    let grenade = matches!(item, Item::Grenade(_));
    // the item list's slots: the other carried grenade types up the column (in order after the
    // selected one), the other items leftward
    let (mut up, left): (Vec<Item>, Vec<Item>) = carried.iter().copied().filter(|&i| i != item).partition(|i| matches!(i, Item::Grenade(_)));
    if let Some(at) = up.iter().position(|&i| matches!((i, item), (Item::Grenade(a), Item::Grenade(b)) if a > b)) {
        up.rotate_left(at);
    }
    let slot_item = |k: usize| -> Option<Item> {
        if !player.item_list {
            return None;
        }
        if k < ITEM_SLOTS_LEFT { left.get(k).copied() } else { up.get(k - ITEM_SLOTS_LEFT).copied() }
    };
    let icon_of = |game: &GameData, i: Item| match i {
        Item::Grenade(k) => kit(k).map(|g| g.def.icon).filter(|&x| x != 0).unwrap_or(FRAG_ICON),
        Item::Medkit => game.0.items.get(&player.medkit_kind).map_or(MEDKIT_ICON, |t| if t.icon != 0 { t.icon } else { MEDKIT_ICON }),
    };
    let new_word = game.0.strings.get(&S_NEW).cloned().unwrap_or("NEW".into());
    let hold = game.0.strings.get(&S_HOLD).cloned().unwrap_or("Hold ".into());
    let to_activate = game.0.strings.get(&S_TO_ACTIVATE).cloned().unwrap_or("to activate %s.".into());
    let regen_word = game.0.strings.get(&S_REGEN).cloned().unwrap_or("Regen".into());
    // the switch hint (play_ammo.rs's SwitchHint) names the other weapon
    let other = (l.weapons.len() > 1).then(|| &l.weapons[(player.weapon + 1) % l.weapons.len()].def.label).filter(|_| switch_hint.0);
    let switch_to = game.0.strings.get(&S_SWITCH_TO).cloned().unwrap_or("switch to %s.".into());
    // after a pickup: the held weapon's name over its count, its icon hidden meanwhile
    let name_shown = held_name.0 > 0.0;
    // its fade at the end (1: full), and the icon faint under it meanwhile, coming back as it
    // fades
    let name_alpha = (held_name.0 / super::ammo::NAME_FADE).min(1.0);
    let feed_shown: Vec<String> = feed.0.iter().rev().take(FEED_LINES).rev().map(|(name, n, _)| if *n > 0 { format!("{n}x {name}") } else { name.clone() }).collect();

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
                let def = &l.weapons[w].def;
                let clip = player.ammo.get(w).map_or(0, |a| a[0]);
                let empty = clip == 0;
                let tint = if w == player.weapon { AMMO_ORANGE } else { MESSAGE_BLUE };
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
            Part::Tab(i) => {
                let speaking = if i == player.character % PORTRAITS.len() { player.speaking > 0.0 }
                    else { squad.0.iter().any(|m| m.character == i && m.in_squad() && m.speaking > 0.0) };
                // every tab shows the member's command (follow); speech has its own icon beside the portrait
                let _ = speaking;
                let icon = Some(PLAYER_ICON);
                let h = icon.and_then(|icon| texture(&mut game.0, &mut images, &mut cache, icon));
                let dead = std::iter::once(&*player).chain(squad.0.iter()).any(|u| u.character == i && u.in_squad() && u.dead);
                show(&mut vis, h.is_some() && !dead);
                if let (Some(mut img), Some(h)) = (img, h) {
                    if img.image != h { img.image = h; }
                }
            }
            Part::ItemBox => show(&mut vis, !carried.is_empty()),
            Part::ItemIcon => {
                // the item's HUD icon: the grenade type's, the carried Medkit's own (h_e5ec3f1f)
                let icon = icon_of(&game, item);
                let h = (count > 0).then(|| if usable { texture(&mut game.0, &mut images, &mut cache, icon) }
                                            else { texture_grey(&mut game.0, &mut images, &mut cache, icon) }).flatten();
                show(&mut vis, h.is_some());
                if let (Some(mut img), Some(h)) = (img, h) {
                    if img.image != h { img.image = h; }
                }
                // a grenade's icon as in the Frag recording, the medkit's as in its captures
                let (x, y, size) = if grenade { ITEM_ICON } else { MEDKIT_ICON_AT };
                if let Some(mut n) = node {
                    let want = at(x, y, size, size);
                    if n.left != want.left || n.width != want.width {
                        (n.left, n.top, n.width, n.height) = (want.left, want.top, want.width, want.height);
                    }
                }
            }
            Part::ItemName => {
                show(&mut vis, count > 0);
                let y = if grenade { GRENADE_NAME_Y } else { ITEM_NAME_Y };
                if let Some(t) = text.as_mut() {
                    if t.y != y {
                        t.y = y;
                    }
                }
                set(text, &name_of(&game, item), Some(item_color));
            }
            Part::ItemCount => {
                show(&mut vis, count > 0 && !(grenade && count == 1));
                let y = if grenade { GRENADE_COUNT_Y } else { ITEM_COUNT_Y };
                if let Some(mut t) = text {
                    if t.y != y {
                        t.y = y;
                    }
                    if t.text != count.to_string() {
                        t.text = count.to_string();
                    }
                    if t.color != item_color {
                        t.color = item_color;
                    }
                }
            }
            Part::ItemNew => {
                show(&mut vis, count > 0 && player.item_new > 0.0);
                set(text, &new_word, None);
            }
            Part::Slot(k, sub) => {
                let Some(i) = slot_item(k) else { show(&mut vis, false); continue };
                show(&mut vis, true);
                let ok = super::item_usable(&player, i);
                match sub {
                    SlotPart::Icon => {
                        let id = icon_of(&game, i);
                        let h = if ok { texture(&mut game.0, &mut images, &mut cache, id) } else { texture_grey(&mut game.0, &mut images, &mut cache, id) };
                        if let (Some(mut img), Some(h)) = (img, h) {
                            if img.image != h { img.image = h; }
                        }
                    }
                    SlotPart::Name => set(text, &name_of(&game, i), Some(if ok { MESSAGE_BLUE } else { ITEM_RED })),
                    SlotPart::Count => set(text, &super::item_count(&player, i).to_string(), Some(if ok { MESSAGE_BLUE } else { ITEM_RED })),
                }
            }
            Part::UsePrompt => {
                show(&mut vis, use_panel.prompt.is_some() && player.scope < 0.5);
                if let Some(what) = use_panel.prompt {
                    set(text, &format!("{hold}E {}", to_activate.replace("%s", what)), None);
                }
            }
            Part::SwitchHint => {
                show(&mut vis, other.is_some());
                if let Some(name) = other {
                    set(text, &format!("Q {}", switch_to.replace("%s", name)), None);
                }
            }
            Part::Ring => {
                // a usable gate panel's button only
                let at = use_panel.target.filter(|_| player.scope < 0.5).and_then(on_screen);
                show(&mut vis, at.is_some());
                if let (Some(c), Some(mut n)) = (at, node) {
                    let want = at_box(c.x - RING_SIZE / 2.0, c.y - RING_SIZE / 2.0, RING_SIZE, RING_SIZE);
                    if n.left != want.left || n.top != want.top || n.width != want.width {
                        (n.left, n.top, n.width, n.height) = (want.left, want.top, want.width, want.height);
                    }
                }
            }
            Part::Message => {
                show(&mut vis, use_panel.message.is_some());
                if let (Some((m, _)), Some(mut t)) = (&use_panel.message, text) {
                    if t.text != *m { t.text = m.clone(); }
                }
            }
            Part::Feed(k) => {
                let line = feed_shown.get(k);
                show(&mut vis, line.is_some());
                if let (Some(line), Some(mut t)) = (line, text) {
                    if t.text != *line { t.text = line.clone(); }
                }
            }
            Part::Crosshair => {
                // (none once dead: the reference recording's reticle goes on the death frame; none
                // during a weapon switch from the drop to the grab: the takes' crosshair goes
                // ~0.5 s after Y and the new one shows at ~0.95 s, todo/49 and 58 take02)
                show(&mut vis, !l.weapons.is_empty() && player.charge <= 0.0 && !player.dead && player.holding);
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
                    img.color = if player.aim_friend { FRIEND_GREEN } else { Color::srgba(0.2, 0.55, 1.0, 0.95) };
                }
            }
        }
    }
}

/// Squad blips on the radar (turning with the camera), the selection marker and the name (centre
/// of the screen) while control passes to another member, and the grenade meter / reticle while a throw charges.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_squad_hud(
    time: Res<Time>,
    player: Res<Player>,
    squad: Res<Squad>,
    mut gen: ResMut<HudGen>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut label: ResMut<NameLabel>,
    mut parts: Query<(&SquadPart, &mut Visibility, &mut Node, Option<&mut Text>, Option<&mut TextFont>, Option<&mut TextColor>, Option<&mut ImageNode>, Option<&PortraitArt>)>,
) {
    let dt = frame_dt(&time);
    let font_k = windows.single().map(|w| w.height() / 480.0).unwrap_or(1.5);
    let _ = &font_k;
    // name: pending while selecting (with how long it has shown: the pop counts from its start);
    // when control passes, zoom it out. Over the death camera (the player dead) it doesn't pop:
    // the reference recording's "HAWK" is at its final size from its first frame (f1910 and
    // f1911 are the same)
    let NameLabel { pending, zooming } = &mut *label;
    if let Some((c, _)) = player.select {
        let start = if player.dead { NAME_POP_TIME } else { 0.0 };
        let shown = pending.filter(|(p, _)| *p == c).map_or(start, |(_, s)| s) + dt;
        *pending = Some((c, shown));
    } else if let Some((c, _)) = pending.take() {
        if player.character == c {
            *zooming = Some((c, 0.0));
        }
    }
    if let Some((_, t)) = zooming.as_mut() {
        *t += dt;
    }
    if zooming.is_some_and(|(_, t)| t > NAME_ZOOM_TIME) {
        *zooming = None;
    }
    let health = |c: usize| std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.character == c && u.in_squad())
        .map(|u| (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0));
    let show = |v: &mut Visibility, on: bool| { let want = if on { Visibility::Inherited } else { Visibility::Hidden }; if *v != want { *v = want; } };
    let pct = |x: f32, y: f32, n: &mut Node| { n.left = Val::Percent(x / 6.4); n.top = Val::Percent(y / 4.8); };
    let fwd = Vec3::new(-player.cam_yaw.sin(), 0.0, -player.cam_yaw.cos());
    let right = Vec3::new(player.cam_yaw.cos(), 0.0, -player.cam_yaw.sin());
    let charging = player.charge > 0.0;
    // the meter: while charging, then held at its level after the button (METER_HOLD) before it
    // fades (METER_FADE)
    let (after, left) = player.meter_after;
    // (the fade is drawn as a cut halfway through it: two to three frames)
    let meter = charging || left > super::METER_FADE * 0.5;
    let level = if charging { player.charge } else { after };
    let selected = player.select.and_then(|(c, _)| squad.0.iter().find(|m| m.character == c && m.in_squad()).map(|m| (c, m.position)));
    let dead = |c: usize| std::iter::once(&*player).chain(squad.0.iter()).any(|u| u.character == c && u.in_squad() && u.dead);
    for (part, mut vis, mut node, _text, _font, _color, img, art) in &mut parts {
        match *part {
            SquadPart::Blip(slot) => {
                let Some(m) = squad.0.get(slot).filter(|m| !m.dead) else { show(&mut vis, false); continue };
                let rel = m.position - player.position;
                let mut v = Vec2::new(rel.dot(right), rel.dot(fwd)) * (RADAR_RADIUS / RADAR_RANGE);
                if v.length() > RADAR_RADIUS {
                    v = v.normalize() * RADAR_RADIUS;
                }
                show(&mut vis, true);
                pct(RADAR_CENTRE.0 + v.x - 2.0, RADAR_CENTRE.1 - v.y - 2.0, &mut node);
            }
            SquadPart::Chevron(k) => {
                show(&mut vis, selected.is_some());
                if let Some((c, _)) = selected {
                    let (px, py) = PORTRAIT_CENTRE[c % PORTRAIT_CENTRE.len()];
                    let u = Vec2::new(px - RADAR_CENTRE.0, py - RADAR_CENTRE.1).normalize_or(Vec2::X);
                    let at = Vec2::new(px, py) - u * (28.0 - 5.0 * k as f32);
                    pct(at.x - 15.0, at.y - 15.0, &mut node);
                    let dir = if u.x.abs() > u.y.abs() { if u.x > 0.0 { 0 } else { 1 } } else if u.y < 0.0 { 2 } else { 3 };
                    // lit one after another toward the portrait, then all dim again
                    let step = (time.elapsed_secs() * 10.0) as usize % 4;
                    let lit = k < step;
                    if let Some(mut img) = img {
                        if img.image != gen.chevrons[dir] { img.image = gen.chevrons[dir].clone(); }
                        img.color = if lit { CHEVRON_RED } else { CHEVRON_RED.darker(0.25).with_alpha(0.75) };
                    }
                }
            }
            SquadPart::SelectName => {
                show(&mut vis, selected.is_some() || zooming.is_some());
            }
            SquadPart::NameGlyph(k) => {
                let (c, scale, alpha) = match (selected, *zooming) {
                    (Some((c, _)), _) => {
                        // popping in: from big down to size, quickly
                        let e = pending.map_or(1.0, |(_, shown)| (shown / NAME_POP_TIME).clamp(0.0, 1.0));
                        (Some(c), NAME_BIG * (1.0 + (NAME_POP - 1.0) * (1.0 - e) * (1.0 - e)), 1.0)
                    }
                    (None, Some((c, t))) => {
                        let z = (t / NAME_ZOOM_TIME).min(1.0);
                        (Some(c), NAME_BIG * (1.0 + (NAME_ZOOM_TO - 1.0) * z * (2.0 - z)), (1.0 - z) * (1.0 - z))
                    }
                    _ => (None, 1.0, 0.0),
                };
                let glyph = c.and_then(|c| CHARACTERS[c].to_uppercase().chars().nth(k)).and_then(|ch| gen.glyphs.get(&ch).copied());
                show(&mut vis, glyph.is_some());
                // unused slots leave the row entirely (hidden nodes still take up space)
                let display = if glyph.is_some() { Display::Flex } else { Display::None };
                if node.display != display { node.display = display; }
                if let Some(r) = glyph {
                    let k = NAME_SCALE * scale * font_k;
                    node.width = Val::Px(r.width() * k);
                    node.height = Val::Px(r.height() * k);
                    if let Some(mut img) = img {
                        img.rect = Some(r);
                        img.color = NAME_ORANGE.with_alpha(alpha);
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
                if let (Some(mut img), Some(art)) = (img, art) {
                    let (want, rect, color) = if dead(c) {
                        (gen.skulls[c].clone(), None, SKULL_TINT)
                    } else {
                        (art.0.clone(), Some(art.1), Color::WHITE)
                    };
                    if img.image != want { img.image = want; }
                    if img.rect != rect { img.rect = rect; }
                    img.color = color;
                }
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
                let full = if gen.tutorial_frame { bar_length(player.max_health) } else { bar_span().1 };
                node.width = Val::Percent(full * f / 6.4);
            }
            SquadPart::PlayerStamina => node.width = Val::Percent(bar_length(player.max_health) / 6.4),
            SquadPart::FrameMiddle => {
                // the middle takes up what the bars' length adds to (or takes from) the picture's
                let extra = bar_length(player.max_health) - (TB_BAR_COLS.1 - TB_BAR_COLS.0) as f32;
                node.width = Val::Percent(((TB_MIDDLE.1 - TB_MIDDLE.0) as f32 + extra).max(1.0) / 6.4);
            }
            SquadPart::FrameRight => {
                let extra = bar_length(player.max_health) - (TB_BAR_COLS.1 - TB_BAR_COLS.0) as f32;
                node.left = Val::Percent((TB_AT.0 + (TB_RIGHT.0 - TB_ICONS.0) as f32 + extra) / 6.4);
            }
            SquadPart::MeterBox | SquadPart::MeterTick | SquadPart::Reticle => show(&mut vis, meter),
            SquadPart::MeterFill => {
                show(&mut vis, meter);
                let h = (METER_FILL.3 - METER_FILL.2) * level;
                node.top = Val::Percent((METER_FILL.3 - h) / 4.8);
                node.height = Val::Percent(h / 4.8);
            }
        }
    }
}

