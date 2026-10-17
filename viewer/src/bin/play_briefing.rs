//! The campaign's briefing movies with their subtitles (#225): a mission zone's movie list
//! (campaign-bf.xmb's `pre-movies`, or its `post-movies`) played one after another, each
//! skippable, with the list's subtitles typed over them in the game's body font.
//!
//! The game plays a list in a game-options menu (default.xbe): the session's +0x30 (0x80220)
//! opens the menu h_f5750fe9, a full-screen movie player and a static text, queues the zone's
//! pre-movies in the player and gives the text each movie's subtitles; its +0x34 (FUN_00080490)
//! does the same with the post-movies in the menu h_1d6f7c4e. When a movie starts, the text gets
//! that movie's subtitles as a sequence (0x10b0f0); the sequence shows each in turn (0x10be50) and
//! the text types it out and lays it out (0x10a630, FUN_0010a880), drawn by the game's text
//! renderer (FUN_000ccd20) with its markup: `^cN` the colour N, `^f(N` .. `^)` a blink at N a
//! second, tabs, line breaks. A or START skips the movie playing (the movie item's input,
//! 0x1026a0); after the last one the menu closes and the game goes on.
//!
//! The tutorial (m00) plays Campaign_Intro, Tex_Intro_Final and mb01; Rescue Brutus (m01) mb02
//! and Brutus_Intro_Final. The movies are converted once into frames and a mixed-down sound, as
//! the front end's (docs/briefing.md); one that wasn't converted is left out.
//!
//! `start` is the entry point (the mission flow's): play a zone's list, then hand over (`Then`).
//! Test hook `BF_MOVIES=<mission>[,<zone>[,post]]` plays one list straight after the boot, then
//! the title; with it `BF_MOVIES_AT`, `BF_MOVIES_SKIP`, `BF_MOVIES_LOG` and `BF_CAPTURE` (see
//! `requested`, `Playing`, `capture`).

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    audio::{AudioPlayer, AudioSource, PlaybackSettings},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};

use bf_viewer::bf::character::{CampaignLevel, Game, Subtitle};
use bf_viewer::bf::hash::h;

use super::AppState;

/// The game's menus are laid out on a 640 x 480 screen; the movie fills it and it's scaled to fit
/// the window whole (between black bars, as the takes show it on a wide screen).
const SCREEN: Vec2 = Vec2::new(640.0, 480.0);
/// Frames a second of the converted movies (docs/briefing.md: made at 30 a second from the Bink
/// files' 29.97).
const MOVIE_FPS: f32 = 30.0;
/// The subtitles' text box: the static text h_1ad19da6 (game-options) of the pre-movies' menu
/// h_f5750fe9 at (60, 100), 520 x 360, and of the post-movies' menu h_1d6f7c4e at (60, 60),
/// 520 x 320. Lines start at its left and top and wrap at its width (FUN_0010a880). take01's
/// first lines: ink from x 60-61 and y 104 (the glyphs' cells start 4-5 units above their
/// capitals).
const PRE_BOX: (Vec2, f32) = (Vec2::new(60.0, 100.0), 520.0);
/// (the post-movies' box: see PRE_BOX)
const POST_BOX: (Vec2, f32) = (Vec2::new(60.0, 60.0), 520.0);
/// The text's own colour, the first of its colour list (`^c0`): 0x10be50 puts the static
/// text's colour first and the subtitle's style colours after it (so `^c1` is the style's
/// first). The static text's style is h_1d46e79b, `color-normal` 153 209 251 (game-options);
/// take01 measures 156 207 252 (the style colour 60 80 251 measures 63 80 249).
const TEXT_COLOUR: [u8; 4] = [153, 209, 251, 255];
/// The body font h_e4e4d2f4's atlas (common textures, 256 x 512): the style's font h_0c626fd5 at
/// 14 points is the record default.xbe keeps at 0x3c35a8 (FUN_000cd5d0 registers it with this
/// texture at scale 1, offset 0; FUN_000cca40 reads it), the atlas's top, larger size.
const FONT_ATLAS: u32 = 0xE4E4_D2F4;
/// The record's line height, 23, plus one: a glyph's cell is this tall (FUN_000ccd20 draws
/// (23 + 1) units) and a line of the text is this far below the one before (the line's height
/// FUN_000ccae0 measures; FUN_0010abb0 places line n at n x it). take01: lines 24 units apart.
/// (The HUD's hint lines draw this record too: play_hints.rs.)
pub(crate) const LINE: f32 = 24.0;
/// The record's characters: 32 to 255 (its +0x14 and +0x16).
const FIRST_CHAR: u32 = 32;
/// (the last: see FIRST_CHAR)
const LAST_CHAR: u32 = 255;
/// A character the record lacks is drawn as 0x82 (FUN_000ccd20, FUN_000ccae0).
const MISSING_CHAR: u32 = 0x82;
/// Tab stops every 50 units: FUN_000ccd20 counts them from the screen's left, FUN_000ccae0
/// (measuring a line to wrap it) from the line's start. take01: "From:", two tabs, then "Deputy"
/// at 200.
const TAB: f32 = 50.0;
/// The glyphs of the record at 0x3c35a8 for the characters 32 to 255 (0x18 bytes each from its
/// +0x1c): the cell's left and top on the atlas (its u x 255 and v x 511: whole pixels), the
/// offset the pen moves before the glyph (+0x10), its width (+0x12; the cell is one wider and
/// LINE tall) and the advance after it (+0x14). Read from default.xbe; take01's thirteen line
/// widths match them to a unit.
const GLYPHS: [(u8, u8, i8, u8, u8); 224] = [
    (0, 0, 0, 1, 4), (3, 0, 1, 4, 4), (9, 0, 0, 8, 8), (19, 0, 0, 10, 10), (31, 0, 1, 8, 9), (41, 0, 0, 14, 14), (57, 0, 0, 11, 11), (70, 0, 0, 4, 4),
    (76, 0, 0, 7, 7), (85, 0, 0, 7, 7), (94, 0, 0, 9, 9), (105, 0, 1, 8, 9), (115, 0, 0, 5, 5), (122, 0, 0, 6, 6), (130, 0, 0, 4, 5), (136, 0, 0, 9, 8),
    (147, 0, 0, 10, 10), (159, 0, 1, 6, 9), (167, 0, 1, 8, 9), (177, 0, 0, 10, 10), (189, 0, 0, 10, 10), (201, 0, 0, 10, 10), (213, 0, 0, 10, 10), (225, 0, 0, 10, 10),
    (0, 24, 0, 10, 10), (12, 24, 0, 10, 10), (24, 24, 0, 4, 6), (30, 24, 0, 5, 5), (37, 24, 1, 8, 9), (47, 24, 1, 8, 9), (57, 24, 1, 8, 9), (67, 24, 0, 9, 9),
    (78, 24, 1, 14, 14), (94, 24, 0, 11, 11), (107, 24, 1, 10, 10), (119, 24, 1, 10, 10), (131, 24, 1, 10, 11), (143, 24, 1, 8, 9), (153, 24, 1, 8, 8), (163, 24, 1, 9, 10),
    (174, 24, 1, 9, 11), (185, 24, 1, 3, 4), (190, 24, 0, 8, 9), (200, 24, 1, 10, 9), (212, 24, 1, 8, 8), (222, 24, 1, 11, 12), (0, 48, 1, 9, 10), (11, 48, 1, 10, 11),
    (23, 48, 1, 9, 9), (34, 48, 1, 10, 11), (46, 48, 1, 10, 9), (58, 48, 0, 9, 10), (69, 48, 0, 9, 9), (80, 48, 1, 9, 10), (91, 48, 0, 11, 11), (104, 48, 0, 15, 15),
    (121, 48, 0, 10, 10), (133, 48, 0, 10, 9), (145, 48, 0, 9, 9), (156, 48, 1, 4, 5), (162, 48, 0, 9, 8), (173, 48, 1, 4, 5), (179, 48, 0, 10, 10), (191, 48, 0, 10, 10),
    (203, 48, 3, 5, 9), (210, 48, 0, 8, 9), (220, 48, 1, 8, 9), (230, 48, 0, 9, 9), (0, 72, 0, 9, 10), (11, 72, 0, 9, 9), (22, 72, 0, 7, 6), (31, 72, 0, 9, 10),
    (42, 72, 1, 8, 9), (52, 72, 1, 3, 4), (57, 72, -1, 5, 6), (64, 72, 1, 9, 8), (75, 72, 1, 3, 4), (80, 72, 1, 13, 14), (95, 72, 1, 8, 9), (105, 72, 0, 9, 10),
    (116, 72, 1, 8, 9), (126, 72, 0, 9, 10), (137, 72, 1, 6, 6), (145, 72, 0, 8, 8), (155, 72, 0, 7, 6), (164, 72, 1, 8, 9), (174, 72, 0, 9, 8), (185, 72, 0, 13, 13),
    (200, 72, 0, 9, 9), (211, 72, 0, 9, 9), (222, 72, 0, 8, 8), (232, 72, 0, 6, 6), (0, 96, 4, 2, 6), (4, 96, 0, 6, 6), (12, 96, 0, 10, 10), (24, 96, 2, 15, 17),
    (41, 96, 2, 15, 17), (58, 96, 2, 15, 17), (75, 96, 2, 15, 17), (92, 96, 2, 15, 17), (109, 96, 2, 15, 17), (126, 96, 2, 15, 17), (143, 96, 2, 15, 17), (160, 96, 2, 15, 17),
    (177, 96, 2, 15, 17), (194, 96, 2, 15, 17), (211, 96, 2, 15, 17), (0, 120, 2, 15, 17), (17, 120, 2, 15, 17), (34, 120, 2, 15, 17), (51, 120, 2, 15, 17), (68, 120, 2, 15, 17),
    (85, 120, 2, 15, 17), (102, 120, 2, 15, 17), (119, 120, 2, 15, 17), (136, 120, 2, 15, 17), (153, 120, 2, 15, 17), (170, 120, 2, 15, 17), (187, 120, 2, 15, 17), (204, 120, 2, 15, 17),
    (0, 144, 2, 15, 17), (17, 144, 2, 15, 17), (34, 144, 2, 15, 17), (51, 144, 2, 15, 17), (68, 144, 2, 15, 17), (85, 144, 2, 15, 17), (102, 144, 2, 15, 17), (119, 144, 2, 15, 17),
    (136, 144, 0, 1, 4), (139, 144, 0, 4, 5), (145, 144, 1, 8, 9), (155, 144, 1, 9, 9), (166, 144, 0, 10, 10), (178, 144, 0, 10, 11), (190, 144, 4, 2, 6), (194, 144, 0, 10, 10),
    (206, 144, 2, 7, 10), (215, 144, 0, 12, 12), (229, 144, 1, 5, 6), (0, 168, 0, 10, 10), (12, 168, 1, 8, 9), (22, 168, 0, 6, 6), (30, 168, 0, 12, 12), (44, 168, 0, 10, 10),
    (56, 168, 0, 6, 7), (64, 168, 1, 8, 9), (74, 168, 1, 6, 7), (82, 168, 0, 7, 8), (91, 168, 3, 5, 9), (98, 168, 0, 10, 10), (110, 168, 0, 11, 11), (123, 168, 1, 4, 5),
    (129, 168, 3, 5, 9), (136, 168, 1, 4, 7), (142, 168, 0, 7, 7), (151, 168, 0, 10, 10), (163, 168, 0, 14, 15), (179, 168, 0, 15, 15), (196, 168, -1, 16, 16), (214, 168, 0, 9, 9),
    (225, 168, 0, 11, 11), (0, 192, 0, 11, 11), (13, 192, 0, 11, 11), (26, 192, 0, 11, 11), (39, 192, 0, 11, 11), (52, 192, 0, 11, 11), (65, 192, -1, 16, 16), (83, 192, 1, 10, 10),
    (95, 192, 1, 8, 9), (105, 192, 1, 8, 9), (115, 192, 1, 8, 9), (125, 192, 1, 8, 9), (135, 192, -1, 5, 6), (142, 192, 0, 5, 5), (149, 192, -1, 7, 6), (158, 192, -1, 7, 6),
    (167, 192, 0, 11, 11), (180, 192, 1, 9, 10), (191, 192, 1, 10, 11), (203, 192, 1, 10, 11), (215, 192, 1, 10, 11), (227, 192, 1, 10, 11), (0, 216, 1, 10, 11), (12, 216, 0, 10, 10),
    (24, 216, 0, 12, 12), (38, 216, 1, 9, 10), (49, 216, 1, 9, 10), (60, 216, 1, 9, 10), (71, 216, 1, 9, 10), (82, 216, 0, 10, 9), (94, 216, 1, 9, 9), (105, 216, 1, 9, 10),
    (116, 216, 0, 8, 9), (126, 216, 0, 8, 9), (136, 216, 0, 8, 9), (146, 216, 0, 8, 9), (156, 216, 0, 8, 9), (166, 216, 0, 8, 9), (176, 216, 0, 13, 13), (191, 216, 0, 9, 9),
    (202, 216, 0, 9, 9), (213, 216, 0, 9, 9), (224, 216, 0, 9, 9), (235, 216, 0, 9, 9), (0, 240, -1, 5, 6), (7, 240, 0, 5, 5), (14, 240, -1, 7, 6), (23, 240, -1, 7, 6),
    (32, 240, 0, 9, 10), (43, 240, 1, 8, 9), (53, 240, 0, 9, 10), (64, 240, 0, 9, 10), (75, 240, 0, 9, 10), (86, 240, 0, 9, 10), (97, 240, 0, 9, 10), (108, 240, 1, 8, 9),
    (118, 240, 0, 10, 10), (130, 240, 1, 8, 9), (140, 240, 1, 8, 9), (150, 240, 1, 8, 9), (160, 240, 1, 8, 9), (170, 240, 0, 9, 9), (181, 240, 1, 8, 9), (191, 240, 0, 9, 9),
];

/// A zone's two movie lists (see `ZoneMovies`): before the zone loads, in the menu h_f5750fe9,
/// or after it, in h_1d6f7c4e.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum List {
    Pre,
    Post,
}

/// What follows the last movie: the hand-over to whoever started the list.
#[derive(Clone, Debug)]
pub enum Then {
    /// the front end, at its title (the test hook's)
    Menu,
    /// another state of the app, which the caller has set up: the mission flow's (#226: the
    /// zone loading), which doesn't call this yet
    #[allow(dead_code)]
    State(AppState),
    /// leave the app (a capture run)
    Quit,
}

/// The list asked for: a mission (its level's id, `h("m00")`), a zone of it, which list and what
/// follows.
#[derive(Resource, Clone, Debug)]
pub struct Request {
    /// the level's id (`CampaignLevel::id`)
    pub mission: u32,
    /// the zone's index in the level (`CampaignLevel::movies`)
    pub zone: usize,
    pub list: List,
    pub then: Then,
}

/// Play a mission zone's movie list (`mission` is its level's id, `h("m00")` for the tutorial),
/// then `then`: the entry point. The front end's own parts go as the briefing opens
/// (play_menu.rs); a movie that wasn't converted is left out, and with none the hand-over comes
/// at once.
pub fn start(commands: &mut Commands, next: &mut NextState<AppState>, mission: u32, zone: usize, list: List, then: Then) {
    commands.insert_resource(Request { mission, zone, list, then });
    next.set(AppState::Briefing);
}

/// Test hook: `BF_MOVIES=<mission>[,<zone>[,post]]` plays that zone's list (zone 0's pre-movies
/// unless told) straight after the boot, then the title (with `BF_CAPTURE`: quits); `m00` is the
/// tutorial's three movies, `m01` Rescue Brutus's two. With it `BF_MOVIES_AT=<s>` starts the list
/// s seconds in (`open`).
pub fn requested() -> Option<Request> {
    let v = std::env::var("BF_MOVIES").ok()?;
    let mut parts = v.split(',').map(str::trim);
    let mission = h(parts.next().filter(|m| !m.is_empty())?);
    let zone = parts.next().and_then(|z| z.parse().ok()).unwrap_or(0);
    let list = if parts.next().is_some_and(|l| l.eq_ignore_ascii_case("post")) { List::Post } else { List::Pre };
    let then = if std::env::var("BF_CAPTURE").is_ok() { Then::Quit } else { Then::Menu };
    Some(Request { mission, zone, list, then })
}

/// The campaign's movie lists and their subtitles' texts (the level list campaign-bf.xmb and the
/// string table), installed at boot or before a map started straight away.
#[derive(Resource, Default)]
pub struct Campaign {
    levels: Vec<CampaignLevel>,
    texts: HashMap<u32, String>,
}

/// The body font's atlas (FONT_ATLAS), installed at boot; the HUD's hint lines draw from it too
/// (play_hints.rs).
#[derive(Resource)]
pub(crate) struct FontAtlas(pub(crate) Handle<Image>);

/// What the briefings need from the game data, gathered on the boot thread: the levels with
/// movies, their subtitles' texts and the font's atlas (RGBA).
pub struct BriefingData {
    levels: Vec<CampaignLevel>,
    texts: HashMap<u32, String>,
    font: Option<(u32, u32, Vec<u8>)>,
}

/// Gather the briefings' data (see BriefingData).
pub fn gather(game: &mut Game) -> BriefingData {
    let levels: Vec<CampaignLevel> = game.campaign.iter()
        .filter(|l| l.movies.iter().any(|z| !z.pre.is_empty() || !z.post.is_empty())).cloned().collect();
    let texts = levels.iter().flat_map(|l| l.movies.iter()).flat_map(|z| z.pre.iter().chain(&z.post))
        .flat_map(|m| m.subtitles.iter()).filter_map(|s| game.strings.get(&s.text).map(|t| (s.text, t.clone()))).collect();
    BriefingData { levels, texts, font: game.texture_rgba(FONT_ATLAS) }
}

/// Put the briefings' data in the world: the movie lists and the font's atlas as an image.
pub fn install(world: &mut World, data: BriefingData) {
    let BriefingData { levels, texts, font } = data;
    if let Some((w, hgt, px)) = font {
        let image = world.resource_mut::<Assets<Image>>().add(Image::new(
            Extent3d { width: w, height: hgt, depth_or_array_layers: 1 }, TextureDimension::D2, px,
            TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default()));
        world.insert_resource(FontAtlas(image));
    } else {
        eprintln!("briefing: no font atlas h_{FONT_ATLAS:08x}; the subtitles won't show");
    }
    world.insert_resource(Campaign { levels, texts });
}

/// The briefing's systems (AppState::Briefing).
pub fn plugin(app: &mut App) {
    let on = in_state(AppState::Briefing);
    app.add_systems(OnEnter(AppState::Briefing), open)
        .add_systems(OnExit(AppState::Briefing), close)
        .add_systems(Update, (fit, advance, subtitles, capture).chain().run_if(on));
}

/// One movie to play: its name, frame files, sound file and subtitles.
struct Reel {
    name: String,
    frames: Vec<PathBuf>,
    audio: Option<PathBuf>,
    subtitles: Vec<Subtitle>,
}

/// The list playing. Test hooks: `BF_MOVIES_SKIP=<s>[,<s>..]` presses skip at those times on the
/// list's clock (seconds since it began playing); `BF_MOVIES_LOG=1` prints each movie's start and
/// end and each subtitle's showing and going.
#[derive(Resource)]
struct Playing {
    /// the movies still to come, and the one on screen
    queue: VecDeque<Reel>,
    reel: Option<Reel>,
    /// counts the movies started (a new one's text starts afresh)
    serial: usize,
    /// the movie's clock (s), the frame of it shown and the image the frames go into
    clock: f32,
    shown: usize,
    image: Handle<Image>,
    /// the list's clock (s): the blink's and the hooks'
    total: f32,
    /// the text box's top left and width (PRE_BOX or POST_BOX)
    text_box: (Vec2, f32),
    /// the hand-over after the last movie
    then: Then,
    /// BF_MOVIES_SKIP's times still to come, and BF_MOVIES_LOG
    skips: VecDeque<f32>,
    log: bool,
    /// capture frames still warming up (shaders compiling): the clocks wait for them
    warm: usize,
    /// the text on screen: the movie's serial, its subtitle and how many of its characters
    text: Option<(usize, usize, usize)>,
}

/// Parts of the briefing's screen, gone when it closes.
#[derive(Component)]
struct BriefingPart;
/// The layer on the 640 x 480 screen the subtitles' glyphs go on (over the movie).
#[derive(Component)]
struct TextLayer;
/// A movie's sound.
#[derive(Component)]
struct ReelSound;
/// A glyph inside a `^f(N` .. `^)` span: shown in the first half of each 1/N s, hidden in the
/// second (FUN_000cd510: hidden while the fraction of the clock x N is over 0.5). take01's
/// "<<LOADING>>" (N = 1): 0.5 s on, 0.5 s off.
#[derive(Component)]
struct Blink(f32);

/// Capture's warm-up frames before the first saved one (shaders compiling).
const CAPTURE_WARMUP: usize = 30;

/// The briefing opens: its camera and screen, and the list asked for (the movies that were
/// converted); with nothing to play, the hand-over at once. Test hook `BF_MOVIES_AT=<s>`: the
/// list starts s seconds in, the movies before that point left out (the sound starts there too).
#[allow(clippy::too_many_arguments)]
fn open(mut commands: Commands, request: Option<Res<Request>>, campaign: Option<Res<Campaign>>, mut images: ResMut<Assets<Image>>,
        mut sources: ResMut<Assets<AudioSource>>, mut next: ResMut<NextState<AppState>>, mut exit: EventWriter<AppExit>) {
    let Some(request) = request.as_deref().cloned() else {
        eprintln!("briefing: no list asked for");
        next.set(AppState::Menu);
        return;
    };
    commands.remove_resource::<Request>();
    commands.insert_resource(ClearColor(Color::BLACK));
    commands.spawn((Camera2d, BriefingPart));
    let root = commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center,
                                      align_items: AlignItems::Center, ..default() }, BriefingPart)).id();
    let screen = commands.spawn((Node { width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() }, ChildOf(root))).id();
    let image = images.add(Image::default());
    commands.spawn((ImageNode::new(image.clone()), Node { position_type: PositionType::Absolute, width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() },
                    ChildOf(screen)));
    commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Px(SCREEN.x), height: Val::Px(SCREEN.y), ..default() },
                    TextLayer, ChildOf(screen)));
    // the zone's list, its movies that were converted
    let movies = campaign.as_ref().and_then(|c| c.levels.iter().find(|l| l.id == request.mission))
        .and_then(|l| l.movies.get(request.zone))
        .map(|z| if request.list == List::Pre { z.pre.clone() } else { z.post.clone() });
    if movies.is_none() {
        eprintln!("briefing: no movie list for mission h_{:08x} zone {} ({:?})", request.mission, request.zone, request.list);
    }
    let mut queue: VecDeque<Reel> = movies.unwrap_or_default().into_iter().filter_map(|m| {
        let name = movie_name(&m.file);
        let frames = super::menu::movie_files(&name);
        if frames.is_empty() {
            eprintln!("briefing: no frames in ../decompiled/movies/{name} (see docs/briefing.md)");
            return None;
        }
        let audio = super::menu::decompiled(&format!("movies/{name}/audio.wav"));
        Some(Reel { name, frames, audio, subtitles: m.subtitles })
    }).collect();
    // BF_MOVIES_AT: the movies it passes are left out
    let mut at = std::env::var("BF_MOVIES_AT").ok().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0).max(0.0);
    while let Some(r) = queue.front() {
        let length = r.frames.len() as f32 / MOVIE_FPS;
        if at < length {
            break;
        }
        at -= length;
        queue.pop_front();
    }
    let skips = std::env::var("BF_MOVIES_SKIP").ok().map(|v| v.split(',').filter_map(|s| s.trim().parse::<f32>().ok()).collect()).unwrap_or_default();
    let text_box = if request.list == List::Pre { PRE_BOX } else { POST_BOX };
    let warm = if std::env::var("BF_CAPTURE").is_ok() { CAPTURE_WARMUP } else { 0 };
    let mut playing = Playing { queue, reel: None, serial: 0, clock: 0.0, shown: usize::MAX, image, total: 0.0, text_box, then: request.then,
                                skips, log: std::env::var("BF_MOVIES_LOG").is_ok(), warm, text: None };
    if next_reel(&mut commands, &mut playing, &mut images, &mut sources, at) {
        commands.insert_resource(playing);
    } else {
        hand_over(&playing.then, &mut next, &mut exit);
    }
}

/// A movie's folder under decompiled/movies: the list's file name without ".bik".
fn movie_name(file: &str) -> String {
    let lower = file.to_ascii_lowercase();
    if lower.ends_with(".bik") { file[..file.len() - 4].to_string() } else { file.to_string() }
}

/// The next movie of the list starts `at` seconds in (its first frame shown, its sound playing:
/// none under BF_MUTE); false if none is left.
fn next_reel(commands: &mut Commands, p: &mut Playing, images: &mut Assets<Image>, sources: &mut Assets<AudioSource>, at: f32) -> bool {
    let Some(reel) = p.queue.pop_front() else { return false };
    p.clock = at;
    p.shown = usize::MAX;
    p.serial += 1;
    show_frame(p, &reel, images);
    if std::env::var("BF_MUTE").is_err() {
        if let Some(bytes) = reel.audio.as_ref().and_then(|a| std::fs::read(a).ok()) {
            let bytes = if at > 0.0 { wav_from(&bytes, at) } else { bytes };
            let sound = sources.add(AudioSource { bytes: Arc::from(bytes.into_boxed_slice()) });
            commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN, ReelSound, BriefingPart));
        }
    }
    if p.log {
        println!("briefing: t {:7.2}  {} starts{} ({:.2} s, {} subtitles)", p.total, reel.name,
                 if at > 0.0 { format!(" at {at:.2} s") } else { String::new() }, reel.frames.len() as f32 / MOVIE_FPS, reel.subtitles.len());
    }
    p.reel = Some(reel);
    true
}

/// Show the movie's frame for its clock (read from its file and decoded when it changes).
fn show_frame(p: &mut Playing, reel: &Reel, images: &mut Assets<Image>) {
    let at = ((p.clock * MOVIE_FPS) as usize).min(reel.frames.len().saturating_sub(1));
    if at == p.shown {
        return;
    }
    p.shown = at;
    if let Some(img) = std::fs::read(&reel.frames[at]).ok().and_then(|b| super::menu::decode_jpeg(&b)) {
        let _ = images.insert(&p.image, img);
    }
}

/// A WAV's sound from `seconds` in: its header kept, the data chunk cut (the converted movies'
/// sound is plain PCM). A data chunk that claims more than the file holds is cut to what's there.
fn wav_from(bytes: &[u8], seconds: f32) -> Vec<u8> {
    let u16_at = |i: usize| bytes.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize);
    let u32_at = |i: usize| bytes.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize);
    let (mut i, mut frame, mut rate) = (12, 0, 0);
    while let (Some(id), Some(size)) = (bytes.get(i..i + 4), u32_at(i + 4)) {
        if id == b"fmt " {
            rate = u32_at(i + 12).unwrap_or(0);
            frame = u16_at(i + 20).unwrap_or(0);
        } else if id == b"data" && frame > 0 {
            let end = (i + 8).saturating_add(size).min(bytes.len());
            let start = (i + 8).saturating_add((seconds * rate as f32) as usize * frame).min(end);
            let mut out = bytes[..i + 8].to_vec();
            out[i + 4..i + 8].copy_from_slice(&((end - start) as u32).to_le_bytes());
            out.extend_from_slice(&bytes[start..end]);
            let riff = (out.len() - 8) as u32;
            out[4..8].copy_from_slice(&riff.to_le_bytes());
            return out;
        }
        i = i.saturating_add(8 + size + (size & 1));
    }
    bytes.to_vec()
}

/// After the last movie: the hand-over (see Then).
fn hand_over(then: &Then, next: &mut NextState<AppState>, exit: &mut EventWriter<AppExit>) {
    match then {
        Then::Menu => next.set(AppState::Menu),
        Then::State(s) => next.set(*s),
        Then::Quit => {
            exit.write(AppExit::Success);
        }
    }
}

/// The briefing closes: its parts and its sound go, and the UI scale is the HUD's again.
fn close(mut commands: Commands, parts: Query<Entity, With<BriefingPart>>, mut scale: ResMut<UiScale>) {
    for e in &parts {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<Playing>();
    scale.0 = 1.0;
}

/// Scale the 640 x 480 screen to fit the window.
fn fit(window: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    let Ok(w) = window.single() else { return };
    let k = (w.width() / SCREEN.x).min(w.height() / SCREEN.y).max(0.01);
    if (scale.0 - k).abs() > 1e-4 {
        scale.0 = k;
    }
}

/// The movie runs on its clock; A or START skips it (the movie item's input 0x1026a0 takes the
/// menu's buttons 0 and 1, FUN_000fff00's +0x80 and +0x90; take02: A and START each skip the
/// movie playing, B and X don't), as do Enter, Space and a click; past its last frame, or
/// skipped, the next starts, and after the last comes the hand-over. Under BF_CAPTURE the clock
/// steps a fixed 1/15 s a frame (BF_CAPTURE_FPS).
#[allow(clippy::too_many_arguments)]
fn advance(mut commands: Commands, time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<&Gamepad>,
           playing: Option<ResMut<Playing>>, mut images: ResMut<Assets<Image>>, mut sources: ResMut<Assets<AudioSource>>,
           sounds: Query<Entity, With<ReelSound>>, mut next: ResMut<NextState<AppState>>, mut exit: EventWriter<AppExit>) {
    let Some(mut p) = playing else { return };
    let dt = if std::env::var("BF_CAPTURE").is_ok() {
        if p.warm > 0 { 0.0 } else { super::capture_step() }
    } else {
        time.delta_secs().min(0.1)
    };
    p.clock += dt;
    p.total += dt;
    let mut skip = keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) || mouse.just_pressed(MouseButton::Left)
        || pads.iter().any(|g| g.any_just_pressed([GamepadButton::South, GamepadButton::Start]));
    if p.skips.front().is_some_and(|&t| p.total >= t) {
        p.skips.pop_front();
        skip = true;
    }
    let Some(reel) = p.reel.take() else { return };
    let over = p.clock * MOVIE_FPS >= reel.frames.len() as f32;
    if !(skip || over) {
        show_frame(&mut p, &reel, &mut images);
        p.reel = Some(reel);
        return;
    }
    if p.log {
        println!("briefing: t {:7.2}  {} {} at {:.2} s", p.total, reel.name, if over { "ends" } else { "skipped" }, p.clock);
    }
    for e in &sounds {
        commands.entity(e).despawn();
    }
    if !next_reel(&mut commands, &mut p, &mut images, &mut sources, 0.0) {
        let then = p.then.clone();
        commands.remove_resource::<Playing>();
        hand_over(&then, &mut next, &mut exit);
    }
}

/// The subtitle on screen `t` seconds into a movie, and how long it has been up: the list's
/// subtitles follow each other, each taking its start + shown seconds (FUN_0010c060 takes the
/// slots off the movie's time until it's inside one), and one is on screen once the time is past
/// its start in its slot (0x10be50; before that, and past the last slot, the text is
/// empty). take01: Campaign_Intro's "From: Deputy Director Operations" from 18.5 s to 35.5 s of
/// the movie, its "Accessing Video Cloning Archives" from 37.5 s to 47.5 s, mb01's text from
/// 13.5 s to 30.5 s, each to within a frame or two.
fn on_screen(subtitles: &[Subtitle], t: f32) -> Option<(usize, f32)> {
    let mut t = t;
    for (i, s) in subtitles.iter().enumerate() {
        let slot = s.start + s.shown;
        if slot < t {
            t -= slot;
            continue;
        }
        return (t > s.start).then_some((i, t - s.start));
    }
    None
}

/// The subtitle's text, typed out and laid out, as glyphs on the text layer: rebuilt when the
/// subtitle or the number of its characters shown changes; the blinking ones shown or hidden
/// every frame.
fn subtitles(mut commands: Commands, playing: Option<ResMut<Playing>>, campaign: Option<Res<Campaign>>, atlas: Option<Res<FontAtlas>>,
             layer: Query<Entity, With<TextLayer>>, mut blinks: Query<(&Blink, &mut Visibility)>) {
    let (Some(mut p), Some(campaign), Some(atlas), Ok(layer)) = (playing, campaign, atlas, layer.single()) else { return };
    let Some(reel) = p.reel.as_ref() else { return };
    let current = on_screen(&reel.subtitles, p.clock);
    // (the text types out a character every `letter` s from when it appears: FUN_0010a240 restarts
    // the static text's typing, 0x10a630 adds a character each time its clock passes another
    // `letter`; markup and line breaks count as characters)
    let want = current.map(|(i, up)| {
        let s = &reel.subtitles[i];
        let length = campaign.texts.get(&s.text).map_or(0, |t| t.chars().count());
        let typed = if s.letter > 0.0 { ((up / s.letter) as usize).min(length) } else { length };
        (p.serial, i, typed)
    });
    if want != p.text {
        if p.log {
            match (p.text, want) {
                (Some((_, a, _)), Some((_, b, _))) if a == b => {}
                (_, Some((_, b, _))) => println!("briefing: t {:7.2}  {} subtitle {} h_{:08x} shows at {:.2} s", p.total, reel.name, b,
                                                 reel.subtitles[b].text, p.clock),
                (Some(_), None) => println!("briefing: t {:7.2}  {} subtitle goes at {:.2} s", p.total, reel.name, p.clock),
                (None, None) => {}
            }
        }
        commands.entity(layer).despawn_related::<Children>();
        if let Some((_, i, typed)) = want {
            let s = &reel.subtitles[i];
            let chars: Vec<char> = campaign.texts.get(&s.text).map(|t| t.chars().take(typed).collect()).unwrap_or_default();
            // the colour list: the text's own, then the style's (0x10be50)
            let colours: Vec<[u8; 4]> = std::iter::once(TEXT_COLOUR).chain(s.colours.iter().copied()).collect();
            let (corner, width) = p.text_box;
            for (n, line) in lay_out(&chars, width).iter().enumerate() {
                draw_line(&mut commands, layer, &atlas.0, line, Vec2::new(corner.x, corner.y + n as f32 * LINE), &colours, p.total);
            }
        }
        p.text = want;
    }
    for (blink, mut vis) in &mut blinks {
        let v = blink_visibility(blink.0, p.total);
        if *vis != v {
            *vis = v;
        }
    }
}

/// Whether a glyph blinking at `rate` a second shows at `clock` (see Blink).
fn blink_visibility(rate: f32, clock: f32) -> Visibility {
    if (clock * rate).fract() > 0.5 { Visibility::Hidden } else { Visibility::Inherited }
}

/// A character's glyph in the record (see GLYPHS) and how far it moves the pen, its offset plus
/// its advance (FUN_000ccd20, FUN_000ccae0); a character out of the record's range is 0x82's.
pub(crate) fn glyph(c: char) -> ((u8, u8, i8, u8, u8), f32) {
    let code = if (FIRST_CHAR..=LAST_CHAR).contains(&(c as u32)) { c as u32 } else { MISSING_CHAR };
    let g = GLYPHS[(code - FIRST_CHAR) as usize];
    (g, (g.2 as i32 + g.4 as i32) as f32)
}

/// How wide a line is as the wrap measures it (FUN_000ccae0 on one line), in this record (the
/// tests').
#[cfg(test)]
fn measure(line: &[char]) -> f32 {
    measure_with(line, &|c| glyph(c).1)
}

/// How wide a line is as the wrap measures it (FUN_000ccae0 on one line) in a font whose
/// characters move the pen by `advance` (their offset plus advance): the pen's furthest point
/// after a glyph, tab stops from the line's start, the markup taking no room, the line's end at a
/// line break.
fn measure_with(line: &[char], advance: &dyn Fn(char) -> f32) -> f32 {
    let (mut x, mut width) = (0.0f32, 0.0f32);
    let mut i = 0;
    while i < line.len() {
        match line[i] {
            '\n' => break,
            '\t' => {
                x += TAB - (x as i32 % TAB as i32) as f32;
                i += 1;
            }
            '^' => match line.get(i + 1) {
                Some('c') => {
                    if i + 2 >= line.len() {
                        break;
                    }
                    i += 3;
                }
                Some('f') => {
                    let mut j = i + 2;
                    if line.get(j) == Some(&'(') {
                        j += 1;
                    }
                    if j >= line.len() {
                        break;
                    }
                    i = j + 1;
                }
                Some(')') => i += 2,
                _ => i += 1,
            },
            c => {
                x += advance(c);
                width = width.max(x);
                i += 1;
            }
        }
    }
    width
}

/// The static text's lines once it holds `text`, in this record (`lay_out_with`).
fn lay_out(text: &[char], width: f32) -> Vec<Vec<char>> {
    lay_out_with(text, width, &|c| glyph(c).1)
}

/// The static text's lines once it holds `text` (FUN_0010a880 takes the characters one at a time)
/// in a font whose characters move the pen by `advance`: each goes on the last line; a line break
/// starts a new one; a character other than a space or a tab that makes the line wider than the
/// box moves what follows the line's last space, hyphen or tab (looked for from the end back to
/// its third character; none: just that character) to a new line. take01: mb01's text wraps
/// after "procedure" and "Ship". (The tutorial's panels lay their texts out with it too, in their
/// records' advances: play_panels.rs.)
pub(crate) fn lay_out_with(text: &[char], width: f32, advance: &dyn Fn(char) -> f32) -> Vec<Vec<char>> {
    let mut lines: Vec<Vec<char>> = vec![];
    for &c in text {
        if lines.is_empty() {
            lines.push(vec![]);
        }
        let line = lines.last_mut().expect("a line");
        line.push(c);
        if c == '\n' {
            lines.push(vec![]);
            continue;
        }
        if c == ' ' || c == '\t' || measure_with(line, advance) <= width {
            continue;
        }
        let mut cut = line.len() - 1;
        let mut i = line.len() - 1;
        while i > 1 {
            if matches!(line[i], ' ' | '-' | '\t') {
                cut = i + 1;
                break;
            }
            i -= 1;
        }
        let rest = line.split_off(cut);
        lines.push(rest);
    }
    lines
}

/// One line of the text as glyph nodes on `layer`, its pen from `at` (the line's top left), as
/// FUN_000ccd20 draws it: each glyph's cell at the pen plus its offset, the pen then moved by its
/// advance; tab stops from the screen's left; `^cN` takes colour N of the list (none past its
/// end); `^f(N` .. `^)` blinks what's between at N a second when the line holds its `^)`; a
/// markup cut short (the text still typing) ends the line. `clock` is the blink's (see Blink).
#[allow(clippy::too_many_arguments)]
fn draw_line(commands: &mut Commands, layer: Entity, atlas: &Handle<Image>, line: &[char], at: Vec2, colours: &[[u8; 4]], clock: f32) {
    let mut x = at.x;
    let mut colour = colours[0];
    let mut blink: Option<f32> = None;
    let mut i = 0;
    while i < line.len() {
        match line[i] {
            '\n' => break,
            '\t' => {
                x += TAB - (x as i32 % TAB as i32) as f32;
                i += 1;
            }
            '^' => match line.get(i + 1) {
                Some('c') => {
                    let Some(d) = line.get(i + 2) else { break };
                    if let Some(c) = d.to_digit(10).and_then(|n| colours.get(n as usize)) {
                        colour = *c;
                    }
                    i += 3;
                }
                Some(')') => {
                    blink = None;
                    i += 2;
                }
                Some('f') => {
                    let mut j = i + 2;
                    // with "(", the span blinks only if the line holds its "^)"
                    let mut closed = true;
                    if line.get(j) == Some(&'(') {
                        j += 1;
                        closed = line[j..].windows(2).any(|w| w == ['^', ')']);
                    }
                    let Some(d) = line.get(j) else { break };
                    if let Some(rate) = d.to_digit(10) {
                        if closed {
                            blink = Some(rate as f32);
                        }
                        j += 1;
                    }
                    i = j;
                }
                _ => i += 1,
            },
            c => {
                let ((gx, gy, offset, w, _), advance) = glyph(c);
                // (a space's cell is empty: not drawn)
                if c != ' ' {
                    let left = x + offset as f32;
                    let cell = Rect::new(gx as f32, gy as f32, gx as f32 + w as f32 + 1.0, gy as f32 + LINE);
                    let [r, gr, b, a] = colour;
                    let mut e = commands.spawn((ImageNode { rect: Some(cell), color: Color::srgba_u8(r, gr, b, a), ..ImageNode::new(atlas.clone()) },
                                                Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(at.y),
                                                       width: Val::Px(w as f32 + 1.0), height: Val::Px(LINE), ..default() },
                                                ChildOf(layer)));
                    if let Some(rate) = blink {
                        e.insert((Blink(rate), blink_visibility(rate, clock)));
                    }
                }
                x += advance;
                i += 1;
            }
        }
    }
}

/// BF_CAPTURE=<dir> on a briefing (with BF_MOVIES): after a short warm-up, saves every frame as
/// <dir>/frame_NNN.png while the clock steps a fixed 1/15 s a frame, then quits after
/// BF_CAPTURE_FRAMES (200) frames (or when the list ends).
fn capture(mut commands: Commands, playing: Option<ResMut<Playing>>, mut frame: Local<usize>, mut exit: EventWriter<AppExit>) {
    let Ok(dir) = std::env::var("BF_CAPTURE") else { return };
    let Some(mut p) = playing else { return };
    if p.warm > 0 {
        p.warm -= 1;
        return;
    }
    let frames: usize = std::env::var("BF_CAPTURE_FRAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(200);
    if *frame < frames {
        commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(format!("{dir}/frame_{:03}.png", *frame)));
    } else if *frame > frames + 20 {
        exit.write(AppExit::Success);
    }
    *frame += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A subtitle with only its timing.
    fn timed(start: f32, shown: f32) -> Subtitle {
        Subtitle { text: 0, start, shown, letter: 0.033, colours: vec![] }
    }

    /// A text as the layout takes it.
    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn subtitles_follow_each_other() {
        // Campaign_Intro's two (campaign-bf.xmb): 18.5 s for 17 s, then 2 s for 10 s
        let s = [timed(18.5, 17.0), timed(2.0, 10.0)];
        assert_eq!(on_screen(&s, 18.4), None);
        assert_eq!(on_screen(&s, 18.6).map(|x| x.0), Some(0));
        assert_eq!(on_screen(&s, 35.4).map(|x| x.0), Some(0));
        assert_eq!(on_screen(&s, 36.0), None);
        assert_eq!(on_screen(&s, 37.6).map(|x| x.0), Some(1));
        assert_eq!(on_screen(&s, 47.6), None);
    }

    #[test]
    fn markup_takes_no_room_and_tabs_stop_every_50() {
        let w = |s: &str| measure(&chars(s));
        assert_eq!(w("^c1AB^c0"), w("AB"));
        assert_eq!(w("^f(1AB^)"), w("AB"));
        assert_eq!(w("A\tB"), TAB + w("B"));
        assert_eq!(w("AB\nCDEF"), w("AB"));
    }

    #[test]
    fn a_long_line_wraps_after_its_last_space_or_hyphen() {
        let lines = lay_out(&chars(&"abcd ".repeat(40)), 520.0);
        assert!(lines.len() > 1);
        for l in &lines {
            let words: String = l.iter().collect();
            assert!(measure(&chars(words.trim_end())) <= 520.0);
            assert_eq!(l.first(), Some(&'a'));
        }
        let lines = lay_out(&chars(&"abcd-".repeat(40)), 520.0);
        assert!(lines.len() > 1 && lines.iter().all(|l| l.first() == Some(&'a')));
        // a line break starts a new line; the break stays on the line it ends
        assert_eq!(lay_out(&chars("ab\ncd"), 520.0), vec![chars("ab\n"), chars("cd")]);
    }

    #[test]
    fn a_wav_cut_keeps_its_header() {
        // 1 s of 16-bit stereo at 100 Hz: 400 bytes of sound
        let mut w = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&2u16.to_le_bytes());
        w.extend_from_slice(&100u32.to_le_bytes());
        w.extend_from_slice(&400u32.to_le_bytes());
        w.extend_from_slice(&4u16.to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&400u32.to_le_bytes());
        w.extend((0..400).map(|i| i as u8));
        let riff = (w.len() - 8) as u32;
        w[4..8].copy_from_slice(&riff.to_le_bytes());
        let cut = wav_from(&w, 0.5);
        assert_eq!(cut.len(), w.len() - 200);
        assert_eq!(u32::from_le_bytes([cut[4], cut[5], cut[6], cut[7]]) as usize, cut.len() - 8);
        assert_eq!(&cut[8..40], &w[8..40]);
        assert_eq!(u32::from_le_bytes([cut[40], cut[41], cut[42], cut[43]]), 200);
        assert_eq!(cut[44], 200);
        // a data chunk claiming more than the file holds: cut to what's there, no panic
        let short = &w[..44 + 100];
        let cut = wav_from(short, 0.5);
        assert_eq!(cut.len(), 44);
        assert_eq!(u32::from_le_bytes([cut[40], cut[41], cut[42], cut[43]]), 0);
        assert_eq!(wav_from(short, 0.1).len(), 44 + 60);
    }
}
