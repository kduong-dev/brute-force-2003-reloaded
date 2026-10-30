//! The front end, after the game's own (todo/ video): the title (START), the main menu
//! (CAMPAIGN, the demo's MULTIPLAYER and EDITOR, OPTIONS, CREDITS) and SELECT MISSION, a
//! carousel of the chosen mode's maps from the game's level list (campaign-bf.xmb `Type`: the
//! deathmatch arenas mp*, the squad deathmatch maps sdm_*) with each one's name, planet and
//! description.
//! Over the menu's background movie (data/movies/menuBack.bik, as frames in
//! ../decompiled/movies/menuBack), with the game's logo, title bar, fonts, button icons,
//! strings, style colours (common/game-options-en.xmb), menu music (menu_dub1) and menu sounds
//! (the global-sounds of game-options, played from the common sound bank).
//!
//! CAMPAIGN leads on to the campaign path (CAMPAIGN TYPE to JOIN CAMPAIGN, then a new campaign's
//! first mission: play_campaign_menus.rs, #226). MULTIPLAYER (#310, the demo's own entry: the game
//! lists DEATHMATCH and SQUAD DEATHMATCH on its main menu) opens a submenu drawn like CAMPAIGN
//! TYPE (play_multiplayer_menu.rs) with those two; each goes on to its maps (the game's mode
//! panel, split-screen / System Link, is skipped). Choosing a map shows the LOADING screen while
//! the map loads on a thread, then plays it in the same window (deathmatch alone, without the
//! radar). In the game, Backspace comes back here, to the map it left: SELECT MISSION, and Back
//! from it the submenu on that mode, and Back from the submenu the main menu on MULTIPLAYER.
//! EDITOR (#193, the demo's own entry) shows LOADING too, then opens the map editor
//! (play_editor.rs) on the test map in its edit mode, as `--edit` does; leaving it (Backspace
//! twice) comes back to the main menu on EDITOR.
//!
//! Keys: arrows / WASD / d-pad move, Enter / Space / A select, Esc / Backspace / B back (Esc at
//! the title quits), the pad's START is START (JOIN CAMPAIGN); the mouse picks and clicks (the
//! main menu and the MULTIPLAYER submenu, not the campaign path).
//!
//! Test hook BF_MENU_PRESS=<key>@<s>[;...] presses keys (a, b, x, y, up, down, left, right, start)
//! that many seconds after the menu first opens; with BF_CAPTURE=<dir> each menu frame is saved as
//! menu_NNN.png (a fixed 1/15 s step a frame), and the run goes on through a briefing (its
//! brief_NNN.png) into the map (its frame_NNN.png, BF_CAPTURE_FRAMES of them, then it quits);
//! BF_MENU_FRAMES=<n> quits after n menu frames.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume},
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};

use bf_viewer::bf::character::{Game, LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};

/// The game's menus are laid out on a 640 x 480 screen; this one is scaled to fit the window.
const SCREEN: Vec2 = Vec2::new(640.0, 480.0);

/// Strings (common string table) the menu shows.
const S_START: u32 = 0xF768_BB5F;
const S_DEMOS: u32 = 0xFB27_0AD2;
/// The title's copyright line ("© & (P) 2003 Microsoft Corporation.  All rights reserved."):
/// game-options' static text at (50, 418), 542 x 20, style h_e76e1577 (centred, 153 209 251 at
/// alpha 200, black outline, body font 12 pt), fading in with the title.
const S_COPYRIGHT: u32 = 0xE360_3AE4;
const COPYRIGHT_BOX: Vec2 = Vec2::new(50.0, 418.0);
const COPYRIGHT_SIZE: Vec2 = Vec2::new(542.0, 20.0);
const COPYRIGHT_CAP: f32 = 8.5;
const COPYRIGHT_COLOR: Color = Color::srgba(153.0 / 255.0, 209.0 / 255.0, 251.0 / 255.0, 200.0 / 255.0);
const S_SELECT_MISSION: u32 = 0xE7B4_FD5A;
const S_PLANET: u32 = 0xFA56_34C8;
const S_RECOMMENDED: u32 = 0x1512_A125;
const S_LOADING: u32 = 0x0F28_682D;
const S_SELECT: u32 = 0x15A0_733A;
const S_BACK: u32 = 0xF1E0_6461;
/// The main menu's CAMPAIGN (its words' string): the campaign path (play_campaign_menus.rs).
const S_CAMPAIGN: u32 = 0xEA6C_B1B3;
/// A main menu entry's words: a string of the game's string table, or the demo's own (EDITOR,
/// which the game hasn't: #193; MULTIPLAYER, its own grouping of the game's two deathmatch
/// modes: #310).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Say {
    Game(u32),
    Demo(&'static str),
}

impl Say {
    /// The words: the string table's, or the demo's.
    fn text(self, strings: &HashMap<u32, String>) -> String {
        match self {
            Say::Game(id) => strings.get(&id).cloned().unwrap_or_default(),
            Say::Demo(s) => s.to_string(),
        }
    }
}

/// The main menu's MULTIPLAYER (#310): the demo's own entry, second after CAMPAIGN. The game
/// lists DEATHMATCH and SQUAD DEATHMATCH on its main menu; the demo groups them under this one,
/// which opens a submenu of the two (play_multiplayer_menu.rs). Its word is the demo's (the game's
/// own "Multiplayer", h_eb279513, is a debug-config label), drawn as the game's entries are.
const MULTIPLAYER: Say = Say::Demo("MULTIPLAYER");
/// MULTIPLAYER's description line: the demo's own text, shown where the game's entries show theirs.
const MULTIPLAYER_ABOUT: Say = Say::Demo("Play a deathmatch in an arena, or lead your squad in a squad deathmatch.");
/// The main menu's EDITOR (#193): the demo's own entry, after MULTIPLAYER. It opens the map
/// editor (play_editor.rs) on the test map. Its word is the demo's (the game has no editor),
/// drawn as the game's entries are.
const EDITOR: Say = Say::Demo("EDITOR");
/// EDITOR's description line: the demo's own text, shown where the game's entries show theirs.
const EDITOR_ABOUT: Say = Say::Demo("Build your own map on the test level: place buildings, objects and characters, then play it with your squad.");
/// The main menu: label and description. CAMPAIGN opens the campaign path, MULTIPLAYER its
/// submenu (DEATHMATCH and SQUAD DEATHMATCH, which lead on to their maps), EDITOR the map editor
/// and CREDITS the credits; OPTIONS isn't in the demo (the error sound).
const MAIN: [(Say, Say); 5] = [
    (Say::Game(0xEA6C_B1B3), Say::Game(0xF35A_957D)), // CAMPAIGN
    (MULTIPLAYER, MULTIPLAYER_ABOUT),                 // MULTIPLAYER (the demo's)
    (EDITOR, EDITOR_ABOUT),                           // EDITOR (the demo's)
    (Say::Game(0x1EB0_7CA1), Say::Game(0xFB11_75B6)), // OPTIONS
    (Say::Game(0xE94A_3EF6), Say::Game(0x0CEB_37E1)), // CREDITS
];
/// Layout (game-options, 640 x 480): the menu words are right-aligned in boxes ending at
/// x 279 (main) / 278 (title), centred in them: (top, height) of each box. The game's tops are
/// 100, 160, 220 (SQUAD DEATHMATCH, two lines, 80 tall), 320 and 380 (OPTIONS, CREDITS): a
/// one-line entry's box is 40 tall, 60 from its neighbour's. The demo's five entries (#310) are
/// all one line, so they take that 60 pitch from the game's first top, evenly: 100, 160, 220, 280,
/// 340. CAMPAIGN, MULTIPLAYER (where DEATHMATCH was) and EDITOR (where SQUAD DEATHMATCH began) sit
/// at the game's tops; OPTIONS and CREDITS at 280 and 340, not the game's 320 and 380: the game's
/// 20-unit gap under SQUAD DEATHMATCH's second line is gone with it, and keeping 320 and 380 would
/// leave the entries unevenly spaced (or start CAMPAIGN at 140). CREDITS' box ends at 380, 100
/// above the screen's bottom edge (#193 had pushed it to 440-480, where its echo was clipped).
const MAIN_BOXES: [(f32, f32); 5] = [(100.0, 40.0), (160.0, 40.0), (220.0, 40.0), (280.0, 40.0), (340.0, 40.0)];
const TITLE_BOXES: [(f32, f32); 2] = [(194.0, 40.0), (246.0, 40.0)];
const MAIN_RIGHT: f32 = 279.0;
const TITLE_RIGHT: f32 = 278.0;
/// The logo, drawn at its own size (512 x 256): on the title, and on the main menu (it slides
/// up from the title's place, translation-anim h_0ba62894).
const LOGO_SIZE: Vec2 = Vec2::new(512.0, 256.0);
const LOGO_TITLE: Vec2 = Vec2::new(194.0, 106.0);
const LOGO_MAIN: Vec2 = Vec2::new(194.0, 18.0);
/// The description box (static-text, style h_15405823).
const ABOUT_AT: Vec2 = Vec2::new(312.0, 240.0);
const ABOUT_WIDTH: f32 = 270.0;
/// Menu animations (game-options): items slide in from y 240 (translation-anim h_e2027fa6)
/// and fade in (color-anim h_17494bb2) as a menu opens; the selected word's echo
/// (select-anim h_e150ead3) grows to 1.7x and fades, in its colour; panel titles type out
/// (menu-style h_189b1ae9, 0.125 s a letter).
const SLIDE_FROM: f32 = 240.0;
const SLIDE_TIME: f32 = 0.25;
const FADE_TIME: f32 = 0.5;
const ECHO_TIME: f32 = 0.5;
const ECHO_SCALE: f32 = 1.7;
/// The echo repeats while the word stays selected (or hovered), each repeat this long, back
/// to back (the first one takes the game's ECHO_TIME).
const ECHO_PULSE: f32 = 0.3;
const ECHO_COLOR: [u8; 4] = [50, 185, 250, 150];
const TYPE_TIME: f32 = 0.125;
/// Text outline (the styles' h_ed70ff4f: black) and drop shadow, in screen units.
const STROKE: f32 = 1.0;
const SHADOW: f32 = 1.5;
/// The opening animations' sounds (splash_screen bank): the first item sliding in, the logo
/// sliding up.
const SND_SLIDE: u32 = 0xE8AC_E091;
const SND_LOGO: u32 = 0xE1AB_D007;

/// Textures: the logo (splash_screen), the title bar of a panel, and (common) the fonts: the
/// "heading" font (menu words, titles), the body font (descriptions; glyphs from `!` on in
/// reading order, in two sizes, the first used) and the controller button icons.
const LOGO: u32 = 0xF1E0_AC39;
/// The panel (game-options h_1544c137 h_e462c760), drawn at (66, 50): its pieces (splash_screen
/// textures) at their places, at their own size unless given, all at alpha 180.
const PANEL_AT: Vec2 = Vec2::new(66.0, 50.0);
const PANEL: [(u32, f32, f32, Option<(f32, f32)>); 7] = [
    (0x190A_4EA7, 2.0, 0.0, None),               // title bar
    (0xFA67_B919, 2.0, 43.0, None),              // the strip under it
    (0x05D9_9DF9, 2.0, 65.0, Some((467.0, 260.0))), // body
    (0x1529_2AE1, 0.0, 325.0, None),             // bottom
    (0x1EE5_9403, 471.0, 0.0, None),             // top right corner
    (0x0D56_02E0, 471.0, 51.0, None),            // right side
    (0x15C6_FB99, 469.0, 65.0, None),            // its edge
];
const PANEL_ALPHA: f32 = 180.0 / 255.0;
/// The carousel (h_f619981e, style h_03268196): 3 pictures of 128 in (106, 102)-(500, 242),
/// the middle one outlined, between the style's arrow (texture-1, h_158f87c1, stretched to the
/// pictures' height; the right one mirrored).
const CAROUSEL_X: [f32; 3] = [106.0, 239.0, 372.0];
const CAROUSEL_Y: f32 = 108.0;
const ARROW: u32 = 0x158F_87C1;
const ARROW_X: [f32; 2] = [76.0, 509.0];
const ARROW_SIZE: Vec2 = Vec2::new(20.0, 128.0);
/// The texts (static-texts of that menu): title (90, 50) 484 x 38 (style h_189b1ae9: 153 209
/// 251, the heading font, no outline, typed); the map's name (74, 245) (h_0e3b5974: 153 209
/// 251, body font 16 pt); recommended players (74, 270), planet (74, 295) and description
/// (75, 317) 450 wide (h_12c208b8: 148 198 149, 14 pt).
const NAME_CAP: f32 = 12.0;
const INFO_CAP: f32 = 10.0;
const INFO: Color = Color::srgb_u8(148, 198, 149);
/// The LOADING menu (game-options h_f6551df5): a 256 x 256 ring at (192, 112) on black (no
/// movie, no music), drawn as four mirrored quarters of a 128 x 128 texture (data flags: flip x
/// h_e3e50283, flip y h_06712921, the latter inverted for our decoded rows), twice: h_06477be9
/// and h_1ec0f086: the same ring with its big blocks on the diagonals / on the axes, shown in
/// turn every 0.1 s (a capture). "LOADING" (h_0f28682d) centred at (0, 118)
/// 256 x 20 in it (style h_f5cbc6f0: the heading font, outlined).
const RING_AT: Vec2 = Vec2::new(192.0, 112.0);
const RING: [u32; 2] = [0x0647_7BE9, 0x1EC0_F086];
const RING_QUARTERS: [(f32, f32, bool, bool); 4] = [(0.0, 0.0, false, true), (128.0, 0.0, true, true), (0.0, 128.0, false, false), (128.0, 128.0, true, false)];
const RING_FRAME: f32 = 0.1;
const FONT_ATLAS: u32 = 0xE0AF_CD52;
const BODY_FONT: u32 = 0xE4E4_D2F4;
pub(crate) const BUTTONS: u32 = 0x0B63_0034;
/// The button icons' row on BUTTONS (atlas y range): A, B, X, Y, L, R.
pub(crate) const BUTTON_ROW: (u32, u32) = (66, 98);
/// The atlas's glyph rows (in ASCII order; row 6 goes on past `~`), and in each the glyph
/// whose foot is the row's baseline.
const FONT_ROWS: [(&str, usize); 7] = [("./0123456789:;", 2), ("<=>?@ABCDEF", 5), ("GHIJKLMNOPQ", 0), ("RSTUVWXYZ[", 0),
                                       ("\\]^_`abcdefghij", 5), ("klmnopqrstuvw", 0), ("xyz{|}~", 0)];

/// Menu sounds: game-options' global-sounds (event -> sound object in sounds-common), with the
/// events the menu code (default.xbe) sends: 0x1038e0 sends h_ebf26601 (h_1dffc5a1) when a list
/// moves, h_f06bc0ab when A selects, menu_error when it can't; 0x100250 sends h_e29fd995
/// (h_ed2b5e99); 0x10a9xx h_f97ceaf0 when a menu opens. A capture settles which plays when
/// the main menu's selection moves: h_ed2b5e99 (at both moves, 0.75 correlation; h_1dffc5a1
/// isn't heard). The carousel's scrolling matched nothing clearly; it keeps h_1dffc5a1.
/// Going back plays no sound of its own: the menu it returns to plays its opening sounds (a
/// capture: the main menu's item slide h_e8ace091 and logo slide h_e1abd007; h_e78073da, the
/// sound of event h_1d899c36, isn't heard).
/// the menu words' selection moving (heard in the capture)
const SND_MOVE: u32 = 0xED2B_5E99;
const SND_SELECT: u32 = 0xE6B8_BF54;
const SND_ERROR: u32 = 0xE476_68BC;
/// the carousel scrolling (not confirmed)
const SND_CHANGE: u32 = 0x1DFF_C5A1;
const SND_OPEN: u32 = 0xF3A5_B12B;
/// The music player's track (splash_screen sound h_fa71a2f8, streamed from the language bank).
const MUSIC: &str = "menu_dub1";
const MUSIC_VOLUME: f32 = 0.5;

/// Body text's capital height and the button icons' height (screen units).
const BODY_CAP: f32 = 9.0;
const BUTTON_SIZE: f32 = 20.0;

/// Background movie frame rate (the frames were made at 15 per second).
const MOVIE_FPS: f32 = 15.0;

/// game-options menu-style colours: normal, selected (main menu words), selected (others)
const TEAL: Color = Color::srgb_u8(30, 110, 150);
const SELECTED: Color = Color::srgb_u8(215, 236, 251);
/// description text (style h_15405823)
const ABOUT: Color = Color::srgb_u8(153, 209, 251);
const WHITE: Color = Color::WHITE;
const OUTLINE: Color = Color::srgb(0.62, 0.86, 1.0);

/// One map: archive, name, recommended players (deathmatch), planet, description, preview.
struct Map {
    file: String,
    title: String,
    players: String,
    planet: String,
    text: String,
    preview: Option<Handle<Image>>,
}

/// The front end's screens.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Screen {
    #[default]
    Title,
    Main,
    Missions,
    /// the map is loading (bf_play starting on it): game-options' LOADING menu
    Loading,
    /// a screen of the campaign path (play_campaign_menus.rs)
    Campaign(campaign::Step),
    /// the MULTIPLAYER submenu (#310, play_multiplayer_menu.rs): DEATHMATCH and SQUAD DEATHMATCH
    Multiplayer,
}

/// The front end's state: the screen, the main menu's selection, the MULTIPLAYER submenu's,
/// SELECT MISSION's level kind and its map per kind, and the campaign path's choices.
#[derive(Resource, Default)]
struct Menu {
    screen: Screen,
    main: usize,
    /// the MULTIPLAYER submenu's entry (`multiplayer::MODES`: 0 DEATHMATCH, 1 SQUAD DEATHMATCH)
    multi: usize,
    kind: u32,
    pick: HashMap<u32, usize>,
    path: campaign::Path,
}

#[derive(Resource)]
struct Assets2 {
    maps: HashMap<u32, Vec<Map>>,
    strings: HashMap<u32, String>,
    font: AtlasFont,
    body: AtlasFont,
    /// the A and B button icons (on `buttons`)
    buttons: Handle<Image>,
    button_rects: Vec<Rect>,
    logo: Option<Handle<Image>>,
    /// the panel's pieces: picture, place, size
    panel: Vec<(Handle<Image>, Vec2, Vec2)>,
    arrow: Option<Handle<Image>>,
    ring: Vec<Handle<Image>>,
}

#[derive(Resource, Default)]
struct Sfx(HashMap<u32, Handle<AudioSource>>);

/// A movie's frames (JPEG): held, or read from their files as they're shown (the credits: 163 MB,
/// so they start at once).
enum Frames {
    Held(Vec<Vec<u8>>),
    Files(Vec<std::path::PathBuf>),
}

impl Frames {
    fn len(&self) -> usize {
        match self {
            Frames::Held(f) => f.len(),
            Frames::Files(f) => f.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn image(&self, i: usize) -> Option<Image> {
        match self {
            Frames::Held(f) => f.get(i).and_then(|b| decode_jpeg(b)),
            Frames::Files(f) => f.get(i).and_then(|p| std::fs::read(p).ok()).and_then(|b| decode_jpeg(&b)),
        }
    }
}

/// The background movie: its frames (JPEG), the shown one and the image they go into.
#[derive(Resource)]
struct Movie {
    frames: Arc<Frames>,
    clock: f32,
    shown: usize,
    image: Handle<Image>,
    /// frames a second; round and round (the menu's) or once (the intro)
    fps: f32,
    looping: bool,
}

/// The 640 x 480 screen everything is placed on, and the movie behind it.
#[derive(Resource)]
struct Ui {
    screen: Entity,
    movie: Entity,
}

/// Something to point at: a main menu entry, an entry of the MULTIPLAYER submenu (its index in
/// `multiplayer::MODES`), or a map of the carousel (offset from the middle).
#[derive(Component, Clone, Copy)]
enum Item {
    Main(usize),
    Mode(usize),
    Carousel(i32),
}

/// The front end's data, gathered from the game data (on the boot thread, or before a map
/// started straight away): pictures, strings, sounds, music and the background movie.
pub struct MenuData {
    maps: HashMap<u32, Vec<Row>>,
    strings: HashMap<u32, String>,
    logo: Option<Rgba>,
    atlas: Option<Rgba>,
    arrow: Option<Rgba>,
    ring: Vec<Rgba>,
    panel: Vec<(Rgba, Vec2, Option<(f32, f32)>)>,
    body: Option<Rgba>,
    buttons: Option<Rgba>,
    waves: HashMap<u32, Vec<u8>>,
    music: Option<Vec<u8>>,
    frames: Vec<Vec<u8>>,
    /// the campaign path's menus and pictures (play_campaign_menus.rs)
    campaign: campaign::CampaignData,
}
type Rgba = (u32, u32, Vec<u8>);
/// a map of the menu: file, title, players, planet, text, preview
type Row = (String, String, String, String, String, Option<Rgba>);

/// Gather the front end's data (see MenuData).
pub fn gather(game: &mut Game, data_dir: &Path) -> MenuData {
    let rgba = |game: &mut Game, t: u32| game.texture_rgba(t);
    let s = |game: &Game, id: u32| game.strings.get(&id).cloned().unwrap_or_default();
    // the maps of each kind, in the game's order
    let mut maps: HashMap<u32, Vec<Row>> = HashMap::new();
    let levels: Vec<_> = game.campaign.iter().filter(|l| l.kind == LEVEL_DEATHMATCH || l.kind == LEVEL_SQUAD_DEATHMATCH)
        .map(|l| (l.kind, l.file.clone(), l.name, l.planet, l.description, l.preview, l.players)).collect();
    for (kind, file, name, planet, about, preview, (lo, hi)) in levels {
        let title = Some(s(game, name)).filter(|t| !t.is_empty()).unwrap_or(file.clone());
        let planet = s(game, S_PLANET).replace("%s", &s(game, planet));
        // "Recommended for %d - %d players." (none on the squad deathmatch maps: 0 0)
        let players = if hi > 0 { s(game, S_RECOMMENDED).replacen("%d", &lo.to_string(), 1).replacen("%d", &hi.to_string(), 1) } else { String::new() };
        let px = rgba(game, preview);
        maps.entry(kind).or_default().push((file, title, players, planet, s(game, about).trim().to_string(), px));
    }
    let (logo, atlas, arrow) = (rgba(game, LOGO), rgba(game, FONT_ATLAS), rgba(game, ARROW));
    let ring: Vec<_> = RING.iter().filter_map(|&t| rgba(game, t)).collect();
    let panel: Vec<_> = PANEL.iter().filter_map(|&(t, x, y, size)| rgba(game, t).map(|p| (p, Vec2::new(x, y), size))).collect();
    let (body, buttons) = (rgba(game, BODY_FONT), rgba(game, BUTTONS));
    let strings = game.strings.clone();
    let _ = game.load_extra_sounds(data_dir, "splash_screen");
    let waves: HashMap<u32, Vec<u8>> = [SND_MOVE, SND_SELECT, SND_ERROR, SND_CHANGE, SND_OPEN, SND_SLIDE, SND_LOGO].into_iter()
        .filter_map(|id| game.sounds.wav(id).map(|w| (id, w))).collect();
    if waves.len() < 7 {
        eprintln!("menu: {} of the 7 menu sounds found", waves.len());
    }
    let music = Game::load_language_waves(data_dir, "splash_screen", "en").ok()
        .and_then(|m| m.into_iter().find(|(n, _)| n == MUSIC)).map(|m| m.1);
    match &music {
        Some(m) => eprintln!("menu: music {MUSIC} ({} KB)", m.len() / 1024),
        None => eprintln!("menu: no music ({MUSIC} not in ml-sounds/en/splash_screen-en.tgz)"),
    }
    let frames = movie_frames("menuBack");
    if frames.is_empty() {
        eprintln!("menu: no background movie frames in ../decompiled/movies/menuBack (see docs/menu.md)");
    }
    let campaign = campaign::gather(game);
    MenuData { maps, strings, logo, atlas, arrow, ring, panel, body, buttons, waves, music, frames, campaign }
}

/// Put the front end's data in the world (its pictures and sounds as assets).
pub fn install(world: &mut World, data: MenuData) {
    let MenuData { maps, strings, logo, atlas, arrow, ring, panel, body, buttons, mut waves, mut music, frames, campaign } = data;
    campaign::install(world, campaign);
    let mut images = world.resource_mut::<Assets<Image>>();
    let image = |images: &mut Assets<Image>, (w, h, px): (u32, u32, Vec<u8>)| images.add(Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px,
        TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default()));
    let maps = maps.into_iter().map(|(k, v)| (k, v.into_iter().map(|(file, title, players, planet, text, px)| Map {
        file, title, players, planet, text, preview: px.map(|p| image(&mut images, p)),
    }).collect())).collect();
    let font = match atlas {
        Some((w, h, px)) => {
            let glyphs = AtlasFont::glyphs(w, h, &px);
            AtlasFont::new(image(&mut images, (w, h, px)), glyphs)
        }
        None => AtlasFont::new(Handle::default(), HashMap::new()),
    };
    let body = match body {
        Some((w, h, px)) => {
            let glyphs = AtlasFont::sequential(w, h, &px);
            AtlasFont::new(image(&mut images, (w, h, px)), glyphs)
        }
        None => AtlasFont::new(Handle::default(), HashMap::new()),
    };
    let (buttons, button_rects) = match buttons {
        Some((w, h, px)) => {
            let rects = AtlasFont::runs(w, &px, BUTTON_ROW.0, BUTTON_ROW.1);
            (image(&mut images, (w, h, px)), rects)
        }
        None => (Handle::default(), vec![]),
    };
    let assets = Assets2 {
        maps, strings, font, body, buttons, button_rects,
        logo: logo.map(|p| image(&mut images, p)),
        panel: panel.into_iter().map(|((w, h, px), at, size)| {
            let size = size.map_or(Vec2::new(w as f32, h as f32), |(sw, sh)| Vec2::new(sw, sh));
            (image(&mut images, (w, h, px)), PANEL_AT + at, size)
        }).collect(),
        arrow: arrow.map(|p| image(&mut images, p)),
        ring: ring.into_iter().map(|p| image(&mut images, p)).collect(),
    };
    let mut sources = world.resource_mut::<Assets<AudioSource>>();
    let sfx = Sfx(waves.drain().map(|(id, w)| (id, sources.add(AudioSource { bytes: Arc::from(w.into_boxed_slice()) }))).collect());
    let music = music.take().map(|m| sources.add(AudioSource { bytes: Arc::from(m.into_boxed_slice()) }));
    world.insert_resource(assets);
    world.insert_resource(sfx);
    world.insert_resource(MenuMedia { music, frames: Arc::new(Frames::Held(frames)) });
}

/// The front end, in bf_play's app: starting up (AppState::Boot: the loading screen at once,
/// the game data read on a thread), the intro (AppState::Intro) and the menu (Menu, Loading).
/// Its data is installed by the boot (or, starting a map straight away, by `install`).
pub fn plugin(app: &mut App) {
    let in_menu = in_state(super::AppState::Menu).or(in_state(super::AppState::Loading));
    app.init_resource::<Clock>()
        // screenshots (BF_MENU_SHOT) step a fixed 1/60 s a frame, so a frame number is a moment
        // of the animations; a capture driven by BF_MENU_PRESS the capture's step (1/15 s)
        .insert_resource(if std::env::var("BF_MENU_SHOT").is_ok() {
            bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f32(1.0 / 60.0))
        } else if menu_capture() {
            bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f32(super::capture_step()))
        } else {
            bevy::time::TimeUpdateStrategy::Automatic
        })
        .insert_resource(Presses::read())
        .add_systems(OnEnter(super::AppState::Boot), start_boot)
        .init_resource::<Opening>()
        .add_systems(Update, (fit, finish_boot, pulse_emblem, leave_boot).chain().run_if(in_state(super::AppState::Boot)))
        .add_systems(Update, (fit, play_movie, end_intro).chain().run_if(in_state(super::AppState::Intro)))
        .add_systems(OnEnter(super::AppState::Menu), (close_boot, open_menu).chain())
        .add_systems(OnExit(super::AppState::Loading), close_menu)
        // (a briefing's movies take the screen: what the boot or the menu put up goes)
        .add_systems(OnEnter(super::AppState::Briefing), (close_boot, close_menu))
        .add_systems(Update, (fit, play_movie, input, build, animate, campaign::animate, finish_loading, finish_credits).chain().run_if(in_menu.clone()))
        // (the screenshot hook runs on into a map, to see it)
        .add_systems(Update, shot.after(finish_loading))
        .add_systems(Update, capture.after(campaign::animate).run_if(in_menu));
}

/// BF_MENU_PRESS's presses (see the module notes): each key and when (s from the menu's first
/// opening), and the clock since then (the menu's: real time, or the capture's steps).
#[derive(Resource, Default)]
struct Presses {
    left: std::collections::VecDeque<(String, f32)>,
    clock: Option<f32>,
}

impl Presses {
    /// BF_MENU_PRESS read (none without it).
    fn read() -> Presses {
        let left = std::env::var("BF_MENU_PRESS").ok().map(|v| v.split([';', ',']).filter_map(|e| {
            let (k, at) = e.trim().split_once('@')?;
            Some((k.trim().to_ascii_lowercase(), at.trim().parse::<f32>().ok()?))
        }).collect()).unwrap_or_default();
        Presses { left, clock: None }
    }

    /// The presses due by the clock this frame, as the frame's asks.
    fn due(&mut self, dt: f32, asked: &mut Asked) {
        let Some(clock) = self.clock.as_mut() else {
            self.clock = Some(0.0);
            return;
        };
        *clock += dt;
        let now = *clock;
        while self.left.front().is_some_and(|p| p.1 <= now) {
            let (k, _) = self.left.pop_front().unwrap_or_default();
            match k.as_str() {
                "a" => asked.select = true,
                "b" => asked.back = true,
                "up" => asked.up = true,
                "down" => asked.down = true,
                "left" => asked.left = true,
                "right" => asked.right = true,
                "start" => asked.start = true,
                other => eprintln!("menu: BF_MENU_PRESS: no key \"{other}\""),
            }
            // (one a frame, as a pad's presses come)
            break;
        }
    }
}

/// A capture of the front end (BF_CAPTURE with BF_MENU_PRESS): see the module notes.
fn menu_capture() -> bool {
    std::env::var("BF_CAPTURE").is_ok() && std::env::var("BF_MENU_PRESS").is_ok()
}

/// BF_CAPTURE with BF_MENU_PRESS: each menu frame saved as menu_NNN.png (numbered on through the
/// menu's later visits: the LOADING screen after a briefing), after a few frames' warm-up; with
/// BF_MENU_FRAMES=<n> the app quits after n of them.
fn capture(mut commands: Commands, mut frame: Local<usize>, mut warm: Local<usize>, mut exit: EventWriter<AppExit>) {
    if !menu_capture() {
        return;
    }
    let Ok(dir) = std::env::var("BF_CAPTURE") else { return };
    if *warm < 10 {
        *warm += 1;
        return;
    }
    let most: Option<usize> = std::env::var("BF_MENU_FRAMES").ok().and_then(|v| v.parse().ok());
    if most.is_some_and(|m| *frame >= m) {
        if *frame > most.unwrap_or(0) + 20 {
            exit.write(AppExit::Success);
        }
        *frame += 1;
        return;
    }
    commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(format!("{dir}/menu_{:03}.png", *frame)));
    *frame += 1;
}

/// A path under ../decompiled (the extracted textures, the converted movies), if it's there; or,
/// with BF_DATA_DIR set (a git worktree has no decompiled/ of its own), beside the "Brute Force"
/// folder that holds the data.
pub(crate) fn decompiled(sub: &str) -> Option<std::path::PathBuf> {
    let here = std::env::var("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| ".".into());
    let beside_data = std::env::var("BF_DATA_DIR").ok().map(|d| std::path::PathBuf::from(d).join("../../decompiled"));
    [here.join("../decompiled"), "decompiled".into(), "../decompiled".into()].into_iter().chain(beside_data)
        .map(|d| d.join(sub)).find(|d| d.exists())
}

/// A movie's frames (decompiled/movies/<name>/*.jpg), if they've been made (see docs/menu.md).
fn movie_frames(name: &str) -> Vec<Vec<u8>> {
    movie_files(name).iter().filter_map(|f| std::fs::read(f).ok()).collect()
}

/// A movie's frame files, in order.
pub(crate) fn movie_files(name: &str) -> Vec<std::path::PathBuf> {
    let Some(dir) = decompiled(&format!("movies/{name}")).filter(|d| d.is_dir()) else { return vec![] };
    let mut files: Vec<_> = std::fs::read_dir(dir).map(|r| r.flatten().map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("jpg"))).collect()).unwrap_or_default();
    files.sort();
    files
}

/// A converted movie's frame (JPEG) as an image.
pub(crate) fn decode_jpeg(bytes: &[u8]) -> Option<Image> {
    Image::from_buffer(bytes, ImageType::Extension("jpg"), CompressedImageFormats::NONE, true, ImageSampler::Default,
                       RenderAssetUsages::default()).ok()
}

/// Scale the 640 x 480 screen to the window, and let the movie cover the window (4:3).
fn fit(window: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>, ui: Option<Res<Ui>>, mut nodes: Query<&mut Node>,
       state: Res<State<super::AppState>>) {
    let (Ok(w), Some(ui)) = (window.single(), ui) else { return };
    let size = w.size();
    let k = (size.x / SCREEN.x).min(size.y / SCREEN.y).max(0.01);
    if (scale.0 - k).abs() > 1e-4 {
        scale.0 = k;
    }
    let (sw, sh) = (size.x / k, size.y / k);
    // (the menu's background covers the window; a movie played on its own fits inside it, whole,
    // between black bars)
    let bw = if *state.get() == super::AppState::Intro { sw.min(sh * 4.0 / 3.0) } else { sw.max(sh * 4.0 / 3.0) };
    let bh = bw * 0.75;
    if let Ok(mut n) = nodes.get_mut(ui.movie) {
        let (l, t) = (Val::Px((sw - bw) * 0.5), Val::Px((sh - bh) * 0.5));
        if n.width != Val::Px(bw) || n.left != l || n.top != t {
            (n.width, n.height, n.left, n.top) = (Val::Px(bw), Val::Px(bh), l, t);
        }
    }
}

fn play_movie(time: Res<Time>, movie: Option<ResMut<Movie>>, mut images: ResMut<Assets<Image>>) {
    let Some(mut movie) = movie else { return };
    if movie.frames.is_empty() {
        return;
    }
    movie.clock += time.delta_secs();
    let at = (movie.clock * movie.fps) as usize;
    let at = if movie.looping { at % movie.frames.len() } else { at.min(movie.frames.len() - 1) };
    if at != movie.shown {
        movie.shown = at;
        if let Some(img) = movie.frames.image(at) {
            let _ = images.insert(&movie.image, img);
        }
    }
}

/// What the player asked for this frame (`start`: the pad's START, which JOIN CAMPAIGN tells from
/// A; elsewhere it selects).
#[derive(Default, PartialEq)]
struct Asked {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    select: bool,
    back: bool,
    start: bool,
}

/// The frame's presses (keys, pads, the mouse, BF_MENU_PRESS) on the screen shown.
#[allow(clippy::too_many_arguments)]
fn input(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, pads: Query<&Gamepad>, menu: Option<ResMut<Menu>>,
         assets: Option<Res<Assets2>>, sfx: Option<Res<Sfx>>, items: Query<(&Interaction, &Item), Changed<Interaction>>,
         mut cursor: EventReader<CursorMoved>, mut exit: EventWriter<AppExit>, mut next: ResMut<NextState<super::AppState>>,
         time: Res<Time>, mut presses: ResMut<Presses>, mut path: campaign::PathParams) {
    let (Some(mut menu), Some(assets), Some(sfx)) = (menu, assets, sfx) else { return };
    let any = |k: &[KeyCode], b: &[GamepadButton]| keys.any_just_pressed(k.iter().copied()) || pads.iter().any(|p| p.any_just_pressed(b.iter().copied()));
    // (START selects, as A does, but on JOIN CAMPAIGN, where it begins)
    let start = any(&[], &[GamepadButton::Start]);
    let join = menu.screen == Screen::Campaign(campaign::Step::Join);
    let mut asked = Asked {
        up: any(&[KeyCode::ArrowUp, KeyCode::KeyW], &[GamepadButton::DPadUp]),
        down: any(&[KeyCode::ArrowDown, KeyCode::KeyS], &[GamepadButton::DPadDown]),
        left: any(&[KeyCode::ArrowLeft, KeyCode::KeyA], &[GamepadButton::DPadLeft]),
        right: any(&[KeyCode::ArrowRight, KeyCode::KeyD], &[GamepadButton::DPadRight]),
        select: any(&[KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space], &[GamepadButton::South]),
        back: any(&[KeyCode::Escape, KeyCode::Backspace], &[GamepadButton::East]),
        start,
    };
    presses.due(time.delta_secs(), &mut asked);
    if asked.start && !join {
        asked.start = false;
        asked.select = true;
    }
    // the mouse: pointing picks (when it moves), clicking selects
    let moved = cursor.read().count() > 0;
    for (interaction, item) in &items {
        let click = *interaction == Interaction::Pressed;
        if !click && !(moved && *interaction == Interaction::Hovered) {
            continue;
        }
        match (*item, menu.screen) {
            (Item::Main(i), Screen::Main) if i != menu.main => {
                menu.main = i;
                play(&mut commands, &sfx, SND_MOVE);
            }
            (Item::Mode(i), Screen::Multiplayer) if i != menu.multi => {
                menu.multi = i;
                play(&mut commands, &sfx, SND_MOVE);
            }
            (Item::Carousel(d), Screen::Missions) if click && d != 0 => {
                asked.left = d < 0;
                asked.right = d > 0;
                continue;
            }
            _ => {}
        }
        asked.select |= click;
    }
    if asked == Asked::default() {
        return;
    }
    match menu.screen {
        Screen::Title => {
            if asked.select {
                menu.screen = Screen::Main;
                play(&mut commands, &sfx, SND_SELECT);
                play(&mut commands, &sfx, SND_OPEN);
            } else if asked.back {
                exit.write(AppExit::Success);
            }
        }
        Screen::Main => {
            if asked.up || asked.down {
                let to = if asked.up { menu.main.checked_sub(1) } else { Some(menu.main + 1).filter(|&i| i < MAIN.len()) };
                match to {
                    Some(i) => {
                        menu.main = i;
                        play(&mut commands, &sfx, SND_MOVE);
                    }
                    None => play(&mut commands, &sfx, SND_ERROR),
                }
            } else if asked.select && MAIN[menu.main].0 == Say::Game(S_CREDITS) {
                play(&mut commands, &sfx, SND_SELECT);
                start_credits(&mut commands);
            } else if asked.select && MAIN[menu.main].0 == EDITOR {
                // the map editor on the test map, behind the LOADING screen (#193)
                play(&mut commands, &sfx, SND_SELECT);
                start_editor(&mut commands, &mut menu, &mut next);
            } else if asked.select && MAIN[menu.main].0 == Say::Game(S_CAMPAIGN) {
                // the campaign path (play_campaign_menus.rs)
                let (screen, p) = campaign::open();
                menu.screen = screen;
                menu.path = p;
                play(&mut commands, &sfx, SND_SELECT);
                play(&mut commands, &sfx, SND_OPEN);
            } else if asked.select && MAIN[menu.main].0 == MULTIPLAYER {
                // the submenu of the two deathmatch modes, on its first (play_multiplayer_menu.rs)
                multiplayer::open(&mut menu);
                play(&mut commands, &sfx, SND_SELECT);
                play(&mut commands, &sfx, SND_OPEN);
            } else if asked.select {
                // (OPTIONS: not in the demo)
                play(&mut commands, &sfx, SND_ERROR);
            } else if asked.back {
                menu.screen = Screen::Title;
            }
        }
        Screen::Multiplayer => multiplayer::input(&mut commands, &mut menu, &asked, &sfx, &assets),
        Screen::Missions => {
            let n = assets.maps.get(&menu.kind).map_or(0, |m| m.len());
            if n == 0 {
                return;
            }
            let kind = menu.kind;
            let at = menu.pick.get(&kind).copied().unwrap_or(0) % n;
            if asked.left || asked.right {
                let to = if asked.left { (at + n - 1) % n } else { (at + 1) % n };
                menu.pick.insert(kind, to);
                play(&mut commands, &sfx, SND_CHANGE);
            } else if asked.select {
                play(&mut commands, &sfx, SND_SELECT);
                let map = assets.maps[&kind][at].file.clone();
                start_loading(&mut commands, &mut menu, &mut next, &map, kind == LEVEL_DEATHMATCH, false);
            } else if asked.back {
                // (the submenu, on the mode these maps are of)
                multiplayer::back_to(&mut menu);
            }
        }
        Screen::Loading => {}
        Screen::Campaign(_) => campaign::input(&mut commands, &mut menu, &asked, &sfx, &mut path, &assets.strings, &mut next),
    }
}

/// The menu music (stopped for the loading screen).
#[derive(Component)]
struct Music;

/// The menu's own entities: its camera, screens and music, removed when a map starts.
#[derive(Component)]
struct MenuPart;

/// The menu's lasting data: its music and the movie's frames.
#[derive(Resource)]
struct MenuMedia {
    music: Option<Handle<AudioSource>>,
    frames: Arc<Frames>,
}

/// Coming back from a map: which, so SELECT MISSION shows it; or (`main`, the end screen's Quit,
/// #226) the main menu, on CAMPAIGN (take10 at 228 s); or (`editor`, leaving the map editor
/// opened from EDITOR, #193) the main menu, on EDITOR.
#[derive(Resource, Default)]
pub struct Return {
    pub map: Option<String>,
    pub deathmatch: bool,
    pub main: bool,
    pub editor: bool,
}

/// A zone to load behind the LOADING screen as the menu opens (the mission flow's, #226: after a
/// briefing's movies, a restart): the menu opens straight onto LOADING (`open_menu`).
#[derive(Resource)]
struct LoadNext(String);

/// Load the zone in archive `map` behind the front end's LOADING screen and play it (from a
/// briefing, or from a map: its state ends first).
pub fn load_next(commands: &mut Commands, next: &mut NextState<super::AppState>, map: &str) {
    commands.insert_resource(LoadNext(map.to_string()));
    next.set(super::AppState::Menu);
}

/// The map loading on a thread (its result arrives here) and which it is; `editor`: the map
/// editor's test map (EDITOR, #193).
#[derive(Resource)]
struct Pending {
    result: std::sync::Mutex<std::sync::mpsc::Receiver<Result<super::LoadedMap, String>>>,
    map: String,
    deathmatch: bool,
    editor: bool,
}

/// Show the loading screen and load `map` on a thread (the game data, the level, its collision);
/// `finish_loading` plays it once it's there. `editor`: the map editor's session (EDITOR), with
/// the test map's data on top as `--test` loads it (play_testmap.rs `load_weapon_data`: every
/// weapon, grenade and character type its tools offer).
fn start_loading(commands: &mut Commands, menu: &mut Menu, next: &mut NextState<super::AppState>, map: &str, deathmatch: bool, editor: bool) {
    menu.screen = Screen::Loading;
    next.set(super::AppState::Loading);
    let (tx, rx) = std::sync::mpsc::channel();
    let name = map.to_string();
    std::thread::spawn(move || {
        let loaded = super::load_game().map(|mut game| {
            let level = super::load_map(&mut game, &name);
            if editor {
                super::testmap::load_weapon_data(&mut game);
            }
            super::LoadedMap { game, level, deathmatch, map: name, test: editor }
        });
        let _ = tx.send(loaded);
    });
    commands.insert_resource(Pending { result: std::sync::Mutex::new(rx), map: map.to_string(), deathmatch, editor });
}

/// EDITOR (#193): the LOADING screen, and the map editor's test map (the flat floor, as `--edit`
/// opens it) loading behind it; `finish_loading` opens the editor on it (play_editor.rs
/// `open_from_menu`).
fn start_editor(commands: &mut Commands, menu: &mut Menu, next: &mut NextState<super::AppState>) {
    start_loading(commands, menu, next, super::editor::TEST_LEVEL, false, true);
}

/// Starting up: the game data (and the menu's, and the opening movies) being read on a thread.
#[derive(Resource)]
struct Booting(std::sync::Mutex<std::sync::mpsc::Receiver<Result<Booted, String>>>);

struct Booted {
    menu: MenuData,
    movies: Vec<Clip>,
    /// the campaign's movie lists (play_briefing.rs)
    briefing: super::briefing::BriefingData,
    /// the font records' atlases (play_fonts.rs)
    fonts: super::fonts::FontData,
}

/// An opening movie: its frames and its sound.
struct Clip {
    frames: Frames,
    audio: Option<Vec<u8>>,
}

/// What the boot and the opening movies put up (gone as the menu opens), and the boot screen's
/// emblem (it fades out as the loading ends) and the screen itself.
#[derive(Component)]
struct BootPart;
#[derive(Component)]
struct BootEmblem;
#[derive(Resource)]
struct BootScreen(Entity);

/// The data is in: the emblem fading out (s so far), then the movies still to play.
#[derive(Resource)]
struct BootDone(f32);
#[derive(Resource, Default)]
struct Opening(std::collections::VecDeque<Clip>);

/// The boot screen (a capture: before the game data is read, the game shows a dim "B" emblem
/// with "Loading" over it at the screen's bottom left, the emblem fading out as the loading ends;
/// it isn't in the data files or the XBE's images, so decompiled/boot holds it as captured, 1:1
/// with the 640 x 480 screen: see docs/menu.md): where, how big, how long the fade.
const BOOT_AT: Vec2 = Vec2::new(44.0, 364.0);
const BOOT_SIZE: Vec2 = Vec2::new(72.0, 88.0);
const BOOT_FADE: f32 = 0.15;
/// The emblem pulses (a capture, the emblem's brightness against its brightest, frame by frame):
/// dim, darker, up to bright and held, back down, every BOOT_PULSE s. (time in the cycle,
/// brightness as seen; the picture holds the brightest.)
const BOOT_PULSE: f32 = 2.0;
/// the emblem's grey at its brightest (57 / 255)
const BOOT_EMBLEM_GREY: f32 = 0.224;
const BOOT_PULSE_KEYS: [(f32, f32); 7] = [(0.0, 0.29), (0.33, 0.12), (0.5, 0.36), (0.8, 1.0), (1.7, 1.0), (1.97, 0.30), (2.0, 0.29)];
/// The opening movies, in order (data/movies/*.bik, converted: see docs/menu.md), and their rate.
/// Each is skipped on its own (any key, click or button), as in a capture.
const OPENING: [&str; 3] = ["MGS_Logo_Final", "DA_Logo_Final1", "Intro_Montage"];
const OPENING_FPS: f32 = 30.0;

/// Whether to play the opening movies: not with BF_NO_INTRO or BF_MOVIES (a briefing's list
/// plays instead), nor under the menu's screenshot and press hooks (unless BF_SHOT_EARLY: they
/// then count frames from the start, to see the boot and movies).
fn intro_wanted() -> bool {
    std::env::var("BF_NO_INTRO").is_err() && std::env::var("BF_MOVIES").is_err()
        && (std::env::var("BF_SHOT_EARLY").is_ok()
            || (std::env::var("BF_MENU_SHOT").is_err() && std::env::var("BF_MENU_GO").is_err() && std::env::var("BF_MENU_PRESS").is_err()))
}

/// Start up: the boot screen straight away, and the game data, the menu's and the opening
/// movies read on a thread.
fn start_boot(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(ClearColor(Color::BLACK));
    commands.spawn((Camera2d, BootPart));
    let back = commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BootPart)).id();
    // (the movies' picture, covering the window as the menu's movie does: hidden till they play)
    let movie = commands.spawn((ImageNode::default(), Node { position_type: PositionType::Absolute, ..default() }, Visibility::Hidden,
                                ChildOf(back))).id();
    let front = commands.spawn((Node {
        position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
        justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
    }, ZIndex(1), BootPart)).id();
    let screen = commands.spawn((Node { width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() }, ChildOf(front))).id();
    commands.insert_resource(Ui { screen, movie });
    let boot = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() },
                               ChildOf(screen))).id();
    commands.insert_resource(BootScreen(boot));
    let png = |name: &str| decompiled(&format!("boot/{name}.png")).and_then(|p| std::fs::read(p).ok())
        .and_then(|b| Image::from_buffer(&b, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::nearest(),
                                         RenderAssetUsages::default()).ok());
    let place = Node { position_type: PositionType::Absolute, left: Val::Px(BOOT_AT.x), top: Val::Px(BOOT_AT.y),
                       width: Val::Px(BOOT_SIZE.x), height: Val::Px(BOOT_SIZE.y), ..default() };
    match png("loading_emblem") {
        Some(e) => { commands.spawn((ImageNode::new(images.add(e)), place.clone(), BootEmblem, ChildOf(boot))); }
        None => eprintln!("menu: no boot screen in ../decompiled/boot (see docs/menu.md)"),
    }
    if let Some(t) = png("loading_text") {
        commands.spawn((ImageNode::new(images.add(t)), place, ChildOf(boot)));
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let dir = super::data_dir();
        let booted = Game::load(&dir).map(|mut game| {
            let menu = gather(&mut game, &dir);
            let briefing = super::briefing::gather(&mut game);
            let fonts = super::fonts::gather(&mut game);
            let movies = if intro_wanted() {
                OPENING.iter().filter_map(|&name| {
                    let frames = movie_frames(name);
                    if frames.is_empty() {
                        eprintln!("menu: no frames in ../decompiled/movies/{name} (see docs/menu.md)");
                        return None;
                    }
                    Some(Clip { frames: Frames::Held(frames), audio: decompiled(&format!("movies/{name}/audio.wav")).and_then(|p| std::fs::read(p).ok()) })
                }).collect()
            } else {
                vec![]
            };
            Booted { menu, movies, briefing, fonts }
        });
        let _ = tx.send(booted);
    });
    commands.insert_resource(Booting(std::sync::Mutex::new(rx)));
}

/// The game data is in: install the menu's; the emblem starts fading.
fn finish_boot(world: &mut World) {
    let got = world.get_resource::<Booting>().and_then(|b| b.0.lock().ok().and_then(|r| r.try_recv().ok()));
    let Some(got) = got else { return };
    world.remove_resource::<Booting>();
    let booted = got.unwrap_or_else(|e| {
        eprintln!("failed to load game data from {}: {e}", super::data_dir().display());
        std::process::exit(1)
    });
    install(world, booted.menu);
    super::briefing::install(world, booted.briefing);
    super::fonts::install(world, booted.fonts);
    world.insert_resource(Opening(booted.movies.into()));
    world.insert_resource(BootDone(0.0));
}

/// The emblem fades out; then the boot screen goes and the opening movies play (or the menu opens).
/// The emblem's pulse (BOOT_PULSE_KEYS), and its fade as the loading ends.
fn pulse_emblem(time: Res<Time>, mut clock: Local<f32>, done: Option<Res<BootDone>>, mut emblem: Query<&mut ImageNode, With<BootEmblem>>) {
    *clock += time.delta_secs();
    let t = *clock % BOOT_PULSE;
    let seen = BOOT_PULSE_KEYS.windows(2).find(|w| t <= w[1].0)
        .map_or(BOOT_PULSE_KEYS[0].1, |w| w[0].1 + (w[1].1 - w[0].1) * (t - w[0].0) / (w[1].0 - w[0].0));
    let fade = done.map_or(1.0, |d| (1.0 - d.0 / BOOT_FADE).max(0.0));
    // (the captured greys are the picture's colours, opaque: dimming is darkening; brightness as
    // seen is sRGB, the tint linear: the tint that takes the emblem's grey to that part of it)
    let linear = |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    let k = linear(BOOT_EMBLEM_GREY * seen * fade) / linear(BOOT_EMBLEM_GREY);
    for mut e in &mut emblem {
        e.color = Color::linear_rgb(k, k, k);
    }
}

fn leave_boot(mut commands: Commands, time: Res<Time>, done: Option<ResMut<BootDone>>,
              screen: Option<Res<BootScreen>>, mut opening: ResMut<Opening>, mut next: ResMut<NextState<super::AppState>>,
              mut images: ResMut<Assets<Image>>, mut sources: ResMut<Assets<AudioSource>>, ui: Res<Ui>) {
    let Some(mut done) = done else { return };
    done.0 += time.delta_secs();
    if done.0 < BOOT_FADE {
        return;
    }
    commands.remove_resource::<BootDone>();
    if let Some(s) = screen {
        commands.entity(s.0).despawn();
        commands.remove_resource::<BootScreen>();
    }
    // test hook BF_MOVIES: a briefing's list instead of the opening movies and the menu
    if let Some(r) = super::briefing::requested() {
        super::briefing::start(&mut commands, &mut next, r.mission, r.zone, r.list, r.then);
        return;
    }
    let playing = next_movie(&mut commands, &mut opening, &mut images, &mut sources, &ui);
    next.set(if playing { super::AppState::Intro } else { super::AppState::Menu });
}

/// Play the next opening movie with its sound (the last one's sound stops); false if none is left.
fn next_movie(commands: &mut Commands, opening: &mut Opening, images: &mut Assets<Image>, sources: &mut Assets<AudioSource>, ui: &Ui) -> bool {
    let Some(clip) = opening.0.pop_front() else { return false };
    let Some(first) = clip.frames.image(0) else { return false };
    let image = images.add(first);
    commands.entity(ui.movie).insert((ImageNode::new(image.clone()), Visibility::Inherited));
    if let Some(a) = clip.audio {
        let sound = sources.add(AudioSource { bytes: Arc::from(a.into_boxed_slice()) });
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN, BootPart, OpeningSound));
    }
    commands.insert_resource(Movie { frames: Arc::new(clip.frames), clock: 0.0, shown: 0, image, fps: OPENING_FPS, looping: false });
    true
}

#[derive(Component)]
struct OpeningSound;

/// A movie ends at its last frame, or at any key, click or button: the next plays, or the menu opens.
fn end_intro(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<&Gamepad>,
             movie: Option<Res<Movie>>, mut opening: ResMut<Opening>, mut next: ResMut<NextState<super::AppState>>,
             mut images: ResMut<Assets<Image>>, mut sources: ResMut<Assets<AudioSource>>, ui: Res<Ui>,
             sounds: Query<Entity, With<OpeningSound>>) {
    let skip = keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some()
        || pads.iter().any(|p| p.digital().get_just_pressed().next().is_some());
    let done = movie.is_none_or(|m| m.clock * m.fps >= m.frames.len() as f32);
    if !(skip || done) {
        return;
    }
    for e in &sounds {
        commands.entity(e).despawn();
    }
    if !next_movie(&mut commands, &mut opening, &mut images, &mut sources, &ui) {
        next.set(super::AppState::Menu);
    }
}

/// The main menu's CREDITS (its words' string) and the movie it plays (data/movies/credits.bik,
/// converted: see docs/menu.md).
const S_CREDITS: u32 = 0xE94A_3EF6;
const CREDITS: &str = "credits";

/// CREDITS was selected (the movie starts this frame), and the menu to come back to after it.
#[derive(Resource)]
struct CreditsAsked;
#[derive(Resource)]
struct FromCredits;

/// CREDITS: the movie starts (finish_credits, after the input).
fn start_credits(commands: &mut Commands) {
    commands.insert_resource(CreditsAsked);
}

/// CREDITS: the menu goes and the credits play as the opening movies do (their frames read from
/// disk as they're shown; any key, click or button stops them), then the menu opens again on
/// CREDITS.
#[allow(clippy::too_many_arguments)]
fn finish_credits(mut commands: Commands, asked: Option<Res<CreditsAsked>>, parts: Query<Entity, With<MenuPart>>,
                  mut opening: ResMut<Opening>, mut next: ResMut<NextState<super::AppState>>, mut images: ResMut<Assets<Image>>,
                  mut sources: ResMut<Assets<AudioSource>>) {
    if asked.is_none() {
        return;
    }
    commands.remove_resource::<CreditsAsked>();
    let files = movie_files(CREDITS);
    let clip = (!files.is_empty()).then(|| Clip {
        frames: Frames::Files(files), audio: decompiled(&format!("movies/{CREDITS}/audio.wav")).and_then(|p| std::fs::read(p).ok()),
    });
    for e in &parts {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<Movie>();
    commands.insert_resource(FromCredits);
    commands.spawn((Camera2d, BootPart));
    let back = commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BootPart)).id();
    let movie = commands.spawn((ImageNode::default(), Node { position_type: PositionType::Absolute, ..default() }, Visibility::Hidden,
                                ChildOf(back))).id();
    let ui = Ui { screen: movie, movie };
    match clip {
        Some(clip) => {
            opening.0.push_back(clip);
            next_movie(&mut commands, &mut opening, &mut images, &mut sources, &ui);
        }
        // (none: the menu opens again straight away)
        None => eprintln!("menu: no frames in ../decompiled/movies/{CREDITS} (see docs/menu.md)"),
    }
    commands.insert_resource(ui);
    next.set(super::AppState::Intro);
}

/// The menu opens: what the boot and the movies put up goes (their sound with it).
fn close_boot(mut commands: Commands, parts: Query<Entity, With<BootPart>>) {
    for e in &parts {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<BootScreen>();
}

/// The map has loaded: play it, or (EDITOR) open the map editor on it (or, if it couldn't, back
/// to its list, or to EDITOR).
fn finish_loading(mut commands: Commands, pending: Option<Res<Pending>>, mut next: ResMut<NextState<super::AppState>>,
                  mut mode: ResMut<NextState<super::editor::EditorMode>>) {
    let Some(pending) = pending else { return };
    let got = pending.result.lock().ok().and_then(|r| r.try_recv().ok());
    let Some(got) = got else { return };
    commands.remove_resource::<Pending>();
    match got {
        Ok(map) => {
            if pending.editor {
                super::editor::open_from_menu(&mut commands, &mut mode);
            }
            super::begin_play(&mut commands, map);
            next.set(super::AppState::Playing);
        }
        Err(e) => {
            eprintln!("menu: couldn't load {}: {e}", pending.map);
            let map = (!pending.editor).then(|| pending.map.clone());
            commands.insert_resource(Return { map, deathmatch: pending.deathmatch, main: false, editor: pending.editor });
            next.set(super::AppState::Menu);
        }
    }
}

/// Open the menu: its camera, the movie behind, the 640 x 480 screen in front, the music; the
/// title, or (back from a map) that map's SELECT MISSION, or the main menu; or, a zone to load
/// (`LoadNext`), the LOADING screen at once, with no music.
#[allow(clippy::too_many_arguments)]
fn open_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>, media: Res<MenuMedia>, assets: Res<Assets2>,
             ret: Option<Res<Return>>, credits: Option<Res<FromCredits>>, load: Option<Res<LoadNext>>, mut next: ResMut<NextState<super::AppState>>) {
    commands.insert_resource(ClearColor(Color::BLACK));
    if let (Some(m), None) = (&media.music, &load) {
        commands.spawn((AudioPlayer::new(m.clone()), PlaybackSettings { volume: Volume::Linear(MUSIC_VOLUME), ..PlaybackSettings::LOOP },
                        Music, MenuPart));
    }
    let first = media.frames.image(0).unwrap_or_default();
    let movie_image = images.add(first);
    commands.spawn((Camera2d, MenuPart));
    let back = commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, MenuPart)).id();
    let movie = commands.spawn((ImageNode::new(movie_image.clone()), Node { position_type: PositionType::Absolute, ..default() },
                                ChildOf(back))).id();
    let front = commands.spawn((Node {
        position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
        justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
    }, ZIndex(1), MenuPart)).id();
    let screen = commands.spawn((Node { width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() }, ChildOf(front))).id();
    commands.insert_resource(Ui { screen, movie });
    commands.insert_resource(Movie { frames: media.frames.clone(), clock: 0.0, shown: 0, image: movie_image, fps: MOVIE_FPS, looping: true });
    // (the main menu opens on its first entry, CAMPAIGN: take01 at 2 s)
    let mut menu = Menu { screen: Screen::Title, main: 0, kind: LEVEL_SQUAD_DEATHMATCH, pick: HashMap::new(), ..default() };
    if let Some(Return { map: Some(map), deathmatch, .. }) = ret.as_deref() {
        // back from a map: its SELECT MISSION (Back from there goes to the MULTIPLAYER submenu on
        // that map's mode, and from there to the main menu on MULTIPLAYER: #310). It's spent: a
        // later way into the menu that leaves nothing (a campaign's last briefing, play_briefing.rs
        // `Then::Menu`) must open on the title, not on this map's list again
        commands.remove_resource::<Return>();
        menu.screen = Screen::Missions;
        menu.kind = if *deathmatch { LEVEL_DEATHMATCH } else { LEVEL_SQUAD_DEATHMATCH };
        menu.main = multiplayer::main_entry();
        menu.multi = multiplayer::mode_of(menu.kind);
        let at = assets.maps.get(&menu.kind).and_then(|m| m.iter().position(|x| &x.file == map)).unwrap_or(0);
        menu.pick.insert(menu.kind, at);
    }
    if ret.as_deref().is_some_and(|r| r.main) {
        // the end screen's Quit: the main menu, on CAMPAIGN
        commands.remove_resource::<Return>();
        menu.screen = Screen::Main;
        menu.main = 0;
    }
    if ret.as_deref().is_some_and(|r| r.editor) {
        // back from the map editor opened from EDITOR (#193): the main menu, on EDITOR
        commands.remove_resource::<Return>();
        menu.screen = Screen::Main;
        menu.main = MAIN.iter().position(|m| m.0 == EDITOR).unwrap_or(0);
    }
    if credits.is_some() {
        // back from the credits: the main menu, on CREDITS
        commands.remove_resource::<FromCredits>();
        menu.screen = Screen::Main;
        menu.main = MAIN.iter().position(|m| m.0 == Say::Game(S_CREDITS)).unwrap_or(0);
    }
    if let Some(LoadNext(map)) = load.as_deref() {
        // a zone to load: LOADING at once
        commands.remove_resource::<LoadNext>();
        start_loading(&mut commands, &mut menu, &mut next, map, false, false);
    }
    commands.insert_resource(menu);
}

/// A map starts: the menu goes (its UI scale with it: the HUD lays itself out).
fn close_menu(mut commands: Commands, parts: Query<Entity, With<MenuPart>>, mut scale: ResMut<UiScale>) {
    for e in &parts {
        commands.entity(e).despawn();
    }
    scale.0 = 1.0;
}

fn play(commands: &mut Commands, sfx: &Sfx, sound: u32) {
    if let Some(s) = sfx.0.get(&sound) {
        commands.spawn((AudioPlayer::new(s.clone()), PlaybackSettings::DESPAWN));
    }
}

/// Lay the current screen out again when the menu changes.
#[allow(clippy::too_many_arguments)]
fn build(mut commands: Commands, menu: Option<Res<Menu>>, assets: Option<Res<Assets2>>, ui: Option<Res<Ui>>, sfx: Option<Res<Sfx>>,
         time: Res<Time>, mut clock: ResMut<Clock>, mut last: Local<Option<(Screen, usize)>>, music: Query<Entity, With<Music>>,
         cm: Option<Res<campaign::CampaignMenus>>) {
    let (Some(menu), Some(assets), Some(ui), Some(sfx)) = (menu, assets, ui, sfx) else { return };
    if !menu.is_changed() {
        return;
    }
    // a new screen opens (its items slide and fade in)
    let came_from = last.map(|l| l.0);
    let opened = came_from != Some(menu.screen);
    *last = Some((menu.screen, menu.main));
    clock.now = time.elapsed_secs();
    if opened {
        clock.opened = clock.now;
    }
    commands.entity(ui.screen).despawn_related::<Children>();
    let s = |id: u32| assets.strings.get(&id).cloned().unwrap_or_default();
    let at = |x: f32, y: f32| Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), ..default() };
    let screen = ui.screen;
    match menu.screen {
        Screen::Campaign(step) => {
            if let Some(cm) = cm.as_deref() {
                campaign::build(&mut commands, screen, cm, &assets.strings, &sfx, step, &menu.path, clock.now, opened);
            }
        }
        Screen::Multiplayer => {
            if let Some(cm) = cm.as_deref() {
                multiplayer::build(&mut commands, screen, cm, &assets.strings, &sfx, menu.multi, clock.now, opened);
            }
        }
        Screen::Title | Screen::Main => {
            let main = menu.screen == Screen::Main;
            if let Some(logo) = &assets.logo {
                let home = if main { LOGO_MAIN } else { LOGO_TITLE };
                let from = if main { LOGO_TITLE.y } else { home.y };
                commands.spawn((ImageNode { flip_y: true, ..ImageNode::new(logo.clone()) }, Tint::new(WHITE),
                                Node { width: Val::Px(LOGO_SIZE.x), height: Val::Px(LOGO_SIZE.y), ..at(home.x, home.y) },
                                Slide { from, home: home.y }, ChildOf(screen)));
            }
            let (words, boxes, right): (Vec<(String, bool)>, &[(f32, f32)], f32) = if main {
                (MAIN.iter().enumerate().map(|(i, m)| (m.0.text(&assets.strings), i == menu.main)).collect(), &MAIN_BOXES, MAIN_RIGHT)
            } else {
                (vec![(s(S_START), true), (s(S_DEMOS), false)], &TITLE_BOXES, TITLE_RIGHT)
            };
            let cap = assets.font.cap;
            for (i, ((word, on), &(top, h))) in words.iter().zip(boxes).enumerate() {
                let y = top + (h - assets.font.height(word, cap)) * 0.5;
                let look = Look::outlined(if *on { SELECTED } else { TEAL });
                let w = assets.font.text(&mut commands, word, cap, look, true,
                    Node { position_type: PositionType::Absolute, right: Val::Px(SCREEN.x - right), top: Val::Px(y), ..default() });
                commands.entity(w).insert((Button, Item::Main(i), ZIndex(1), Slide { from: y + SLIDE_FROM - top, home: y }, ChildOf(screen)));
                if *on {
                    // the selected word's echo: grows from its middle and fades
                    commands.spawn((Echo { word: word.clone(), right, centre: top + h * 0.5, cap, born: clock.now, shown: None, pulsing: false },
                                    Node { position_type: PositionType::Absolute, width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() },
                                    ChildOf(screen)));
                }
            }
            if !main {
                // the copyright line, centred in its box
                let line = s(S_COPYRIGHT);
                let (w, h) = (assets.body.width(&line, COPYRIGHT_CAP), assets.body.height(&line, COPYRIGHT_CAP));
                let c = assets.body.text(&mut commands, &line, COPYRIGHT_CAP, Look::outlined(COPYRIGHT_COLOR), false,
                    at(COPYRIGHT_BOX.x + (COPYRIGHT_SIZE.x - w) * 0.5, COPYRIGHT_BOX.y + (COPYRIGHT_SIZE.y - h) * 0.5));
                commands.entity(c).insert(ChildOf(screen));
            }
            if main {
                let about = assets.body.paragraph(&mut commands, &MAIN[menu.main].1.text(&assets.strings), BODY_CAP, Look::outlined(ABOUT), ABOUT_WIDTH,
                                                  at(ABOUT_AT.x, ABOUT_AT.y));
                commands.entity(about).insert(ChildOf(screen));
            }
            if opened {
                play(&mut commands, &sfx, SND_SLIDE);
                if main {
                    play(&mut commands, &sfx, SND_LOGO);
                }
            }
        }
        Screen::Loading => {
            // black: the movie and the music stop
            commands.entity(ui.movie).insert(Visibility::Hidden);
            for e in &music {
                commands.entity(e).despawn();
            }
            for (layer, ring) in assets.ring.iter().enumerate() {
                for (x, y, flip_x, flip_y) in RING_QUARTERS {
                    let mut quarter = commands.spawn((ImageNode { flip_x, flip_y: !flip_y, ..ImageNode::new(ring.clone()) },
                                                      Node { width: Val::Px(128.0), height: Val::Px(128.0), ..at(RING_AT.x + x, RING_AT.y + y) },
                                                      ChildOf(screen)));
                    quarter.insert(RingFrame(layer));
                }
            }
            let word = s(S_LOADING);
            let cap = assets.font.cap;
            let w = assets.font.text(&mut commands, &word, cap, Look::outlined(SELECTED), false,
                at(RING_AT.x + (256.0 - assets.font.width(&word, cap)) * 0.5, RING_AT.y + 118.0 + (20.0 - assets.font.height(&word, cap)) * 0.5));
            commands.entity(w).insert(ChildOf(screen));
        }
        Screen::Missions => {
            let maps = assets.maps.get(&menu.kind).map(|v| v.as_slice()).unwrap_or_default();
            let n = maps.len().max(1);
            let pick = menu.pick.get(&menu.kind).copied().unwrap_or(0) % n;
            if opened {
                play(&mut commands, &sfx, SND_SLIDE);
            }
            // the panel
            for (picture, place, size) in &assets.panel {
                commands.spawn((ImageNode { flip_y: true, ..ImageNode::new(picture.clone()) }, Tint::new(WHITE.with_alpha(PANEL_ALPHA)),
                                Node { width: Val::Px(size.x), height: Val::Px(size.y), ..at(place.x, place.y) }, ChildOf(screen)));
            }
            let title_look = Look { color: ABOUT, outline: None, typed: true };
            let title = assets.font.text(&mut commands, &s(S_SELECT_MISSION), assets.font.cap, title_look, false,
                                         at(90.0, 50.0 + (38.0 - assets.font.height("A", assets.font.cap)) * 0.5));
            commands.entity(title).insert(ChildOf(screen));
            // the carousel: the previous map, this one (outlined), the next, between the arrows
            for (d, x) in (-1..=1i32).zip(CAROUSEL_X) {
                let Some(m) = maps.get((pick as i32 + d).rem_euclid(n as i32) as usize) else { continue };
                let mut img = ImageNode { flip_y: true, ..default() };
                if let Some(p) = &m.preview {
                    img.image = p.clone();
                }
                let edge = if d == 0 { 2.0 } else { 0.0 };
                let frame = commands.spawn((Node { border: UiRect::all(Val::Px(edge)), ..at(x - edge, CAROUSEL_Y - edge) }, BorderColor(OUTLINE),
                                            Button, Item::Carousel(d), ChildOf(screen))).id();
                commands.spawn((img, Tint::new(WHITE), Node { width: Val::Px(128.0), height: Val::Px(128.0), ..default() }, ChildOf(frame)));
            }
            if let Some(arrow) = &assets.arrow {
                for (d, x) in [(-1, ARROW_X[0]), (1, ARROW_X[1])] {
                    commands.spawn((ImageNode { flip_x: d > 0, ..ImageNode::new(arrow.clone()) }, Tint::new(WHITE),
                                    Node { width: Val::Px(ARROW_SIZE.x), height: Val::Px(ARROW_SIZE.y), ..at(x, CAROUSEL_Y) },
                                    Button, Item::Carousel(d), ChildOf(screen)));
                }
            }
            if let Some(m) = maps.get(pick) {
                let plain = |color| Look { color, outline: None, typed: false };
                let line = |y: f32, h: f32, cap: f32| at(74.0, y + (h - assets.body.height("A", cap)) * 0.5);
                let name = assets.body.text(&mut commands, &m.title, NAME_CAP, plain(ABOUT), false, line(245.0, 22.0, NAME_CAP));
                let players = assets.body.text(&mut commands, &m.players, INFO_CAP, plain(INFO), false, line(270.0, 20.0, INFO_CAP));
                let planet = assets.body.text(&mut commands, &m.planet, INFO_CAP, plain(INFO), false, line(295.0, 20.0, INFO_CAP));
                let about = assets.body.paragraph(&mut commands, &m.text, INFO_CAP, plain(INFO), 450.0, at(75.0, 317.0));
                for e in [name, players, planet, about] {
                    commands.entity(e).insert(ChildOf(screen));
                }
            }
            // the buttons (style h_e35bb628): the game's A and B icons and their words
            for (x, button, label) in [(86.0, 0, S_SELECT), (242.0, 1, S_BACK)] {
                let row = commands.spawn((Node { align_items: AlignItems::Center, column_gap: Val::Px(4.0), ..at(x, 410.0) }, ChildOf(screen))).id();
                if let Some(&r) = assets.button_rects.get(button) {
                    let k = BUTTON_SIZE / r.height();
                    commands.spawn((ImageNode { rect: Some(r), ..ImageNode::new(assets.buttons.clone()) }, Tint::new(WHITE),
                                    Node { width: Val::Px(r.width() * k), height: Val::Px(BUTTON_SIZE), ..default() }, ChildOf(row)));
                }
                let word = assets.body.text(&mut commands, &s(label), INFO_CAP, Look::outlined(ABOUT), false, Node::default());
                commands.entity(word).insert(ChildOf(row));
            }
        }
    }
}

/// A string's lines (the string table breaks them with "\r\n", "\n" or "\r").
fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.split(['\r', '\n']).filter(|l| !l.is_empty())
}

/// One paragraph of a string: its line breaks made spaces.
fn paragraph(text: &str) -> String {
    lines(text).map(str::trim).collect::<Vec<_>>().join(" ")
}

/// The game's font: glyphs cut from its atlas (white, alpha-shaped), set on a baseline.
pub(crate) struct AtlasFont {
    pub(crate) image: Handle<Image>,
    glyphs: HashMap<char, Glyph>,
    /// atlas pixels: capital height, rise above and drop below the baseline
    pub(crate) cap: f32,
    pub(crate) rise: f32,
    pub(crate) drop: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub(crate) rect: Rect,
    /// the baseline (atlas y)
    pub(crate) base: f32,
}

impl AtlasFont {
    pub(crate) fn new(image: Handle<Image>, glyphs: HashMap<char, Glyph>) -> AtlasFont {
        let cap = glyphs.get(&'A').map_or(20.0, |g| g.rect.height());
        let rise = glyphs.values().map(|g| g.base - g.rect.min.y).fold(cap, f32::max);
        let drop = glyphs.values().map(|g| g.rect.max.y - g.base).fold(0.0, f32::max);
        AtlasFont { image, glyphs, cap, rise, drop }
    }

    /// The glyphs of the atlas: rows cut at empty lines, glyphs at empty columns (the rows of
    /// FONT_ROWS), each trimmed to its ink.
    fn glyphs(w: u32, h: u32, px: &[u8]) -> HashMap<char, Glyph> {
        let solid = |x: u32, y: u32| px[((y * w + x) * 4 + 3) as usize] > 40;
        let runs = |n: u32, ink: &dyn Fn(u32) -> bool| {
            let mut out = vec![];
            let mut i = 0;
            while i < n {
                if ink(i) {
                    let i0 = i;
                    while i < n && ink(i) {
                        i += 1;
                    }
                    out.push((i0, i));
                }
                i += 1;
            }
            out
        };
        let bands = runs(h, &|y| (0..w).any(|x| solid(x, y)));
        let mut out = HashMap::new();
        for (&(y0, y1), (chars, foot)) in bands.iter().zip(FONT_ROWS) {
            let cols = runs(w, &|x| (y0..y1).any(|y| solid(x, y)));
            if cols.len() < chars.chars().count() {
                continue;
            }
            let rects: Vec<Rect> = cols.iter().map(|&(x0, x1)| {
                let ys: Vec<u32> = (y0..y1).filter(|&y| (x0..x1).any(|x| solid(x, y))).collect();
                Rect::new(x0 as f32, ys[0] as f32, x1 as f32, (ys[ys.len() - 1] + 1) as f32)
            }).collect();
            let base = rects[foot].max.y;
            for (c, rect) in chars.chars().zip(rects) {
                out.insert(c, Glyph { rect, base });
            }
        }
        out
    }

    /// The ink runs (glyph columns) of the atlas rows y0..y1, each trimmed to its ink.
    pub(crate) fn runs(w: u32, px: &[u8], y0: u32, y1: u32) -> Vec<Rect> {
        let solid = |x: u32, y: u32| px[((y * w + x) * 4 + 3) as usize] > 40;
        let mut out = vec![];
        let mut x = 0;
        while x < w {
            if (y0..y1).any(|y| solid(x, y)) {
                let x0 = x;
                while x < w && (y0..y1).any(|y| solid(x, y)) {
                    x += 1;
                }
                let ys: Vec<u32> = (y0..y1).filter(|&y| (x0..x).any(|x| solid(x, y))).collect();
                out.push(Rect::new(x0 as f32, ys[0] as f32, x as f32, (ys[ys.len() - 1] + 1) as f32));
            }
            x += 1;
        }
        out
    }

    /// A font whose glyphs run from `!` on in reading order across the rows (the body fonts),
    /// the atlas's first size only (rows until a wide gap); each row's baseline is where most
    /// of its glyphs end.
    pub(crate) fn sequential(w: u32, h: u32, px: &[u8]) -> HashMap<char, Glyph> {
        let solid = |x: u32, y: u32| px[((y * w + x) * 4 + 3) as usize] > 40;
        let mut bands: Vec<(u32, u32)> = vec![];
        let mut y = 0;
        while y < h {
            if (0..w).any(|x| solid(x, y)) {
                let y0 = y;
                while y < h && (0..w).any(|x| solid(x, y)) {
                    y += 1;
                }
                if bands.last().is_some_and(|b| y0 - b.1 > 20) {
                    break;
                }
                bands.push((y0, y));
            }
            y += 1;
        }
        let mut all = vec![];
        for (y0, y1) in bands {
            let rects = Self::runs(w, px, y0, y1);
            let mut feet: HashMap<i32, usize> = HashMap::new();
            for r in &rects {
                *feet.entry(r.max.y as i32).or_default() += 1;
            }
            let base = feet.into_iter().max_by_key(|f| (f.1, -f.0)).map_or(y1 as f32, |f| f.0 as f32);
            all.extend(rects.into_iter().map(|rect| Glyph { rect, base }));
        }
        // '!' to '~' first; then boxes (the codes without a glyph); the set ends with Latin-1's
        // printable half, '¡' (0xA1) to 'ÿ' (0xFF): counted back from the last glyph
        let mut out: HashMap<char, Glyph> = ('!'..='~').zip(all.iter().copied()).collect();
        let latin = 0xFF - 0xA1 + 1;
        if all.len() >= 94 + latin {
            out.extend((0xA1u8..=0xFF).map(char::from).zip(all[all.len() - latin..].iter().copied()));
        }
        out
    }

    /// `text` as one paragraph (its line breaks made spaces), wrapped at `width`.
    fn paragraph(&self, commands: &mut Commands, text: &str, cap: f32, look: Look, width: f32, node: Node) -> Entity {
        let k = cap / self.cap;
        let advance = |c: char| self.glyph(c).map_or(self.cap * 0.4, |g| g.rect.width() + 2.0) * k;
        let wide = |l: &str| l.chars().map(advance).sum::<f32>();
        let mut lines: Vec<String> = vec![];
        let mut line = String::new();
        for word in paragraph(text).split(' ').filter(|w| !w.is_empty()) {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if !line.is_empty() && wide(&candidate) > width {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = candidate;
            }
        }
        lines.push(line);
        self.text(commands, &lines.join("\n"), cap, look, false, node)
    }

    pub(crate) fn glyph(&self, c: char) -> Option<Glyph> {
        self.glyphs.get(&c).or_else(|| self.glyphs.get(&c.to_ascii_uppercase())).copied()
    }

    /// The height of `text` (its lines) with capitals `cap` units tall.
    fn height(&self, text: &str, cap: f32) -> f32 {
        let k = cap / self.cap;
        lines(text).count().max(1) as f32 * (self.rise + self.drop) * k
    }

    /// The width of `text` (its widest line) with capitals `cap` units tall.
    fn width(&self, text: &str, cap: f32) -> f32 {
        let k = cap / self.cap;
        lines(text).map(|l| l.chars().map(|c| self.glyph(c).map_or(self.cap * 0.4, |g| g.rect.width() + 2.0) * k).sum::<f32>())
            .fold(0.0, f32::max)
    }

    /// `text` (lines; right-aligned or left) with capitals `cap` units tall, as a UI node placed
    /// by `node`, drawn by `look`: each glyph over its black outline and drop shadow.
    fn text(&self, commands: &mut Commands, text: &str, cap: f32, look: Look, right: bool, node: Node) -> Entity {
        let k = cap / self.cap;
        let block = commands.spawn(Node { flex_direction: FlexDirection::Column,
                                          align_items: if right { AlignItems::FlexEnd } else { AlignItems::FlexStart }, ..node }).id();
        let mut letter = 0;
        for line in lines(text) {
            let row = commands.spawn((Node { height: Val::Px((self.rise + self.drop) * k), ..default() }, ChildOf(block))).id();
            for c in line.chars() {
                let appear = if look.typed { letter as f32 * TYPE_TIME } else { 0.0 };
                letter += 1;
                let Some(g) = self.glyph(c) else {
                    commands.spawn((Node { width: Val::Px(self.cap * 0.4 * k), ..default() }, ChildOf(row)));
                    continue;
                };
                let (w, h) = (g.rect.width() * k, g.rect.height() * k);
                let cell = commands.spawn((Node { width: Val::Px(w), height: Val::Px(h),
                                                  margin: UiRect { top: Val::Px((self.rise - (g.base - g.rect.min.y)) * k), right: Val::Px(2.0 * k), ..default() },
                                                  ..default() }, ChildOf(row))).id();
                let mut layer = |dx: f32, dy: f32, color: Color| {
                    commands.spawn((ImageNode { rect: Some(g.rect), color, ..ImageNode::new(self.image.clone()) },
                                    Tint { base: color, appear },
                                    Node { position_type: PositionType::Absolute, left: Val::Px(dx), top: Val::Px(dy),
                                           width: Val::Px(w), height: Val::Px(h), ..default() },
                                    ChildOf(cell)));
                };
                if let Some(edge) = look.outline {
                    layer(SHADOW, SHADOW, edge);
                    for (dx, dy) in [(-STROKE, 0.0), (STROKE, 0.0), (0.0, -STROKE), (0.0, STROKE)] {
                        layer(dx, dy, edge);
                    }
                }
                layer(0.0, 0.0, look.color);
            }
        }
        block
    }
}

/// How text is drawn: its colour, outline (and drop shadow) colour, and whether it types out.
#[derive(Clone, Copy)]
struct Look {
    color: Color,
    outline: Option<Color>,
    typed: bool,
}

impl Look {
    fn outlined(color: Color) -> Look {
        Look { color, outline: Some(Color::BLACK), typed: false }
    }
}

/// The animation clock: now, and when the shown screen opened.
#[derive(Resource, Default)]
struct Clock {
    now: f32,
    opened: f32,
}

/// A picture's own colour, faded in as its screen opens (and, typed text, shown from `appear`
/// seconds in).
#[derive(Component)]
struct Tint {
    base: Color,
    appear: f32,
}

impl Tint {
    fn new(base: Color) -> Tint {
        Tint { base, appear: 0.0 }
    }
}

/// One of the loading ring's two pictures (shown in turn).
#[derive(Component)]
struct RingFrame(usize);

/// An item sliding in as its screen opens: from `from` to `home` (its top, screen units).
#[derive(Component)]
struct Slide {
    from: f32,
    home: f32,
}

/// The selected word's echo (redrawn each frame at its size, centred on the word): the word, its
/// box's right edge and centre, its size, when it began and the text now shown.
#[derive(Component)]
struct Echo {
    word: String,
    right: f32,
    centre: f32,
    cap: f32,
    born: f32,
    shown: Option<Entity>,
    /// past its first echo: repeating at ECHO_PULSE
    pulsing: bool,
}

/// Run the menu animations (see SLIDE_FROM ..).
fn animate(mut commands: Commands, time: Res<Time>, clock: Res<Clock>, assets: Option<Res<Assets2>>,
           mut tints: Query<(&Tint, &mut ImageNode)>, mut slides: Query<(&Slide, &mut Node)>,
           mut echoes: Query<(Entity, &mut Echo)>, mut ring: Query<(&RingFrame, &mut Visibility)>) {
    let now = time.elapsed_secs();
    let shown = (now / RING_FRAME) as usize % 2;
    for (frame, mut vis) in &mut ring {
        let v = if frame.0 == shown { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != v {
            *vis = v;
        }
    }
    let t = now - clock.opened;
    let fade = (t / FADE_TIME).clamp(0.0, 1.0);
    for (tint, mut img) in &mut tints {
        let a = if t < tint.appear { 0.0 } else { tint.base.alpha() * fade };
        if (img.color.alpha() - a).abs() > 1e-3 {
            img.color = tint.base.with_alpha(a);
        }
    }
    let k = (t / SLIDE_TIME).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - k) * (1.0 - k);
    for (slide, mut node) in &mut slides {
        let top = Val::Px(slide.from + (slide.home - slide.from) * ease);
        if node.top != top {
            node.top = top;
        }
    }
    let Some(assets) = assets else { return };
    for (e, mut echo) in &mut echoes {
        if let Some(old) = echo.shown.take() {
            commands.entity(old).despawn();
        }
        // it repeats while its word stays selected (the screen is rebuilt, and the echo with
        // it, when the selection moves)
        let length = if echo.pulsing { ECHO_PULSE } else { ECHO_TIME };
        if now - echo.born >= length {
            echo.born = now;
            echo.pulsing = true;
        }
        let k = (now - echo.born) / if echo.pulsing { ECHO_PULSE } else { ECHO_TIME };
        if k >= 1.0 {
            continue;
        }
        let cap = echo.cap * (1.0 + (ECHO_SCALE - 1.0) * k);
        let [r, g, b, a] = ECHO_COLOR;
        let color = Color::srgba_u8(r, g, b, a).with_alpha(a as f32 / 255.0 * (1.0 - k));
        // centred on the word: it grows out from the word's middle
        let top = echo.centre - assets.font.height(&echo.word, cap) * 0.5;
        let middle = echo.right - assets.font.width(&echo.word, echo.cap) * 0.5;
        let right = middle + assets.font.width(&echo.word, cap) * 0.5;
        let shown = assets.font.text(&mut commands, &echo.word, cap, Look { color, outline: None, typed: false }, true,
            Node { position_type: PositionType::Absolute, right: Val::Px(SCREEN.x - right), top: Val::Px(top), ..default() });
        commands.entity(shown).insert(ChildOf(e));
        echo.shown = Some(shown);
    }
}

/// Test hooks: BF_MENU_SHOT=<file.png> saves the menu and quits; BF_MENU_SCREEN=title | main |
/// multiplayer | dm | sdm | editor (or the campaign path's campaign | profile | slot | name |
/// difficulty | join) shows that screen and BF_MENU_PICK=<n> selects entry n of it first (`main`:
/// 0 CAMPAIGN, 1 MULTIPLAYER, 2 EDITOR, 3 OPTIONS, 4 CREDITS; `multiplayer`, the submenu: 0
/// DEATHMATCH, 1 SQUAD DEATHMATCH; `dm` and `sdm`, SELECT MISSION: the map; `editor`: the main
/// menu on EDITOR). BF_MENU_GO plays the selected map (on `multiplayer`: the picked mode's first),
/// or, on EDITOR, opens the map editor.
#[allow(clippy::too_many_arguments)]
fn shot(mut commands: Commands, mut frames: Local<u32>, menu: Option<ResMut<Menu>>, assets: Option<Res<Assets2>>,
        mut exit: EventWriter<AppExit>, mut next: ResMut<NextState<super::AppState>>, state: Res<State<super::AppState>>,
        cm: Option<Res<campaign::CampaignMenus>>) {
    let go = std::env::var("BF_MENU_GO").is_ok();
    let file = std::env::var("BF_MENU_SHOT").ok();
    if file.is_none() && !go {
        return;
    }
    // frames count from the menu (the boot takes however long it takes; then on through a movie
    // it plays), or with BF_SHOT_EARLY from the start
    if *state.get() == super::AppState::Boot && std::env::var("BF_SHOT_EARLY").is_err() {
        return;
    }
    *frames += 1;
    if let (Some(mut menu), Some(assets)) = (menu, assets) {
        if *frames == 2 {
            let pick = std::env::var("BF_MENU_PICK").ok().and_then(|v| v.parse::<usize>().ok());
            match std::env::var("BF_MENU_SCREEN").as_deref() {
                Ok("title") => menu.screen = Screen::Title,
                Ok("main") => {
                    menu.screen = Screen::Main;
                    menu.main = pick.unwrap_or(menu.main).min(MAIN.len() - 1);
                }
                Ok("loading") => menu.screen = Screen::Loading,
                Ok("credits") => {
                    menu.screen = Screen::Main;
                    menu.main = MAIN.iter().position(|m| m.0 == Say::Game(S_CREDITS)).unwrap_or(0);
                    start_credits(&mut commands);
                }
                // the main menu on EDITOR (#193; BF_MENU_GO chooses it)
                Ok("editor") => {
                    menu.screen = Screen::Main;
                    menu.main = MAIN.iter().position(|m| m.0 == EDITOR).unwrap_or(0);
                }
                // the MULTIPLAYER submenu (#310) on its entry BF_MENU_PICK
                Ok("multiplayer") => {
                    menu.screen = Screen::Multiplayer;
                    menu.main = multiplayer::main_entry();
                    menu.multi = pick.unwrap_or(0).min(multiplayer::MODES.len() - 1);
                }
                Ok(kind @ ("dm" | "sdm")) => {
                    menu.screen = Screen::Missions;
                    menu.kind = if kind == "dm" { LEVEL_DEATHMATCH } else { LEVEL_SQUAD_DEATHMATCH };
                    let k = menu.kind;
                    menu.pick.insert(k, pick.unwrap_or(0));
                }
                // the campaign path's screens (play_campaign_menus.rs)
                Ok(step @ ("campaign" | "profile" | "slot" | "name" | "difficulty" | "join")) => {
                    let (screen, path) = campaign::for_shot(step, &assets.strings, cm.as_deref());
                    menu.screen = screen;
                    menu.path = path;
                }
                _ => {}
            }
        }
        if *frames == 3 && go && menu.screen == Screen::Multiplayer {
            // (BF_MENU_GO on the submenu chooses its entry: that mode's maps, whose first the next
            // step plays)
            menu.kind = multiplayer::MODES[menu.multi.min(multiplayer::MODES.len() - 1)].kind;
            menu.screen = Screen::Missions;
        }
        if *frames == 4 && go && menu.screen == Screen::Missions {
            match assets.maps.get(&menu.kind).and_then(|v| v.get(menu.pick.get(&menu.kind).copied().unwrap_or(0) % v.len().max(1))) {
                Some(m) => {
                    let (file, solo) = (m.file.clone(), menu.kind == LEVEL_DEATHMATCH);
                    start_loading(&mut commands, &mut menu, &mut next, &file, solo, false);
                }
                None => {
                    exit.write(AppExit::Success);
                }
            }
        }
        if *frames == 4 && go && menu.screen == Screen::Main && MAIN[menu.main].0 == EDITOR {
            start_editor(&mut commands, &mut menu, &mut next);
        }
    }
    let Some(file) = file else { return };
    let at: u32 = std::env::var("BF_MENU_SHOT_FRAME").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    if *frames == at {
        commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(file));
    } else if *frames > at + 20 {
        exit.write(AppExit::Success);
    }
}

/// Whether to show the menu: no map chosen and no test hook running, or BF_MENU_PRESS driving it
/// (with BF_CAPTURE: the menu's capture).
pub fn wanted() -> bool {
    const TESTS: [&str; 11] = ["BF_MAP", "BF_CAPTURE", "BF_SCREENSHOT", "BF_TEST_GOTO", "BF_AUTOPILOT", "BF_DUMP_SOUND_IDS",
                               "BF_DUMP_MUSIC", "BF_FIND_STEEP", "BF_ANIM_PROBE", "BF_DUMP_TEXTURE", "BF_NO_MENU"];
    (std::env::var("BF_MENU_PRESS").is_ok() && std::env::var("BF_MAP").is_err()) || TESTS.iter().all(|k| std::env::var(k).is_err())
}

/// The campaign path's screens (CAMPAIGN TYPE to JOIN CAMPAIGN).
#[path = "play_campaign_menus.rs"]
mod campaign;

/// The MULTIPLAYER submenu (#310): DEATHMATCH and SQUAD DEATHMATCH, drawn like CAMPAIGN TYPE.
#[path = "play_multiplayer_menu.rs"]
mod multiplayer;

#[cfg(test)]
mod tests {
    use super::*;

    /// The main menu's entries (#193, #310): CAMPAIGN first (the menu opens on it), then the demo's
    /// MULTIPLAYER and EDITOR, then the game's OPTIONS and CREDITS.
    #[test]
    fn main_menu_order() {
        assert_eq!(MAIN.len(), MAIN_BOXES.len());
        let words: Vec<Say> = MAIN.iter().map(|m| m.0).collect();
        assert_eq!(words, [Say::Game(S_CAMPAIGN), MULTIPLAYER, EDITOR, Say::Game(0x1EB0_7CA1), Say::Game(S_CREDITS)]);
    }

    /// The main menu's five one-line entries (#310) sit at the game's 60-unit pitch from its first
    /// top, 100, with its one-line boxes' 40 height, and CREDITS' box ends well above the screen's
    /// bottom edge, where #193's pushed it.
    #[test]
    fn main_menu_is_evenly_spaced() {
        assert_eq!(MAIN_BOXES, [(100.0, 40.0), (160.0, 40.0), (220.0, 40.0), (280.0, 40.0), (340.0, 40.0)]);
        assert!(MAIN_BOXES.windows(2).all(|w| w[1].0 - w[0].0 == 60.0));
        let (top, height) = MAIN_BOXES[MAIN_BOXES.len() - 1];
        assert!(top + height <= SCREEN.y - 100.0);
    }

    /// A map's `Return` is spent as the menu opens on it (#310 review): a later way into the menu
    /// that leaves nothing behind (a campaign's last briefing, play_briefing.rs `Then::Menu`)
    /// opens on the title, not on that map's SELECT MISSION again.
    #[test]
    fn a_return_is_spent_once_the_menu_has_opened_on_it() {
        use bevy::ecs::system::RunSystemOnce;
        let font = || AtlasFont::new(Handle::default(), HashMap::new());
        let mut world = World::new();
        world.insert_resource(Assets::<Image>::default());
        world.insert_resource(NextState::<super::super::AppState>::default());
        world.insert_resource(MenuMedia { music: None, frames: Arc::new(Frames::Held(vec![])) });
        world.insert_resource(Assets2 { maps: HashMap::new(), strings: HashMap::new(), font: font(), body: font(), buttons: Handle::default(),
                                        button_rects: vec![], logo: None, panel: vec![], arrow: None, ring: vec![] });
        world.insert_resource(Return { map: Some("mp1".into()), deathmatch: true, main: false, editor: false });
        world.run_system_once(open_menu).expect("open_menu");
        let menu = world.resource::<Menu>();
        assert_eq!((menu.screen, menu.kind, menu.main), (Screen::Missions, LEVEL_DEATHMATCH, multiplayer::main_entry()));
        assert!(world.get_resource::<Return>().is_none());
        // (the next way in leaves nothing behind)
        world.run_system_once(open_menu).expect("open_menu");
        assert_eq!(world.resource::<Menu>().screen, Screen::Title);
    }
}
