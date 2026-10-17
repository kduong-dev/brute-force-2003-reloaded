//! The front end's campaign path (#226; viewer/docs/menu.md § Campaign): CAMPAIGN on the main
//! menu, then CAMPAIGN TYPE, SELECT PROFILE, SELECT CAMPAIGN, ENTER NAME, SELECT DIFFICULTY and
//! JOIN CAMPAIGN, as take01 goes through them (todo/28-tutorial/, 6.8-35.4 s); START on JOIN
//! CAMPAIGN starts a new campaign at the difficulty chosen: the first mission's briefing movies
//! (play_briefing.rs), LOADING and its first zone, the tutorial. A child of play_menu.rs.
//!
//! The screens are game-options' menus, drawn from their data (bf/menus.rs) in the game's font
//! records (play_fonts.rs), the static texts as the end screen draws them (play_endmenu.rs
//! `text_item`):
//!  * CAMPAIGN TYPE (h_f9728897): five buttons under the headings SINGLE XBOX CONSOLE and SYSTEM
//!    LINK, each button's description its buddy-control. CAMPAIGN's event-script (event 2, action
//!    1) runs the menu sequence h_0ff0b19d: SELECT PROFILE (h_14761942), SELECT CAMPAIGN
//!    (h_e61ae053), SELECT DIFFICULTY (h_1a6884c5), SELECT MISSION (h_16ddaa42), JOIN CAMPAIGN
//!    (h_0130d4e8). A new campaign has one mission open: take01 goes from SELECT DIFFICULTY to
//!    JOIN CAMPAIGN. ENTER NAME (h_e37b5c35) opens from SELECT CAMPAIGN's "New Campaign".
//!  * SELECT PROFILE and SELECT CAMPAIGN: a listbox (display-count rows in its height, each row
//!    centred) the code fills: the profiles DEFAULT, INVERTED (profile-default.xmb's and
//!    profile-inverted.xmb's profile-id) and "New Profile"; the campaigns saved, then "New
//!    Campaign" (FUN_000f1430).
//!  * ENTER NAME: the keyboard item h_f1eabb1b, its keys from the data, the name a new campaign
//!    gets: "CAMP%03d" from 1, the first not taken (0x7e1f0; format 0x40dc5c, h_eedaa81e "CAMP").
//!    DONE saves the campaign: "Saving..." on the status line (take01 21.00-21.98 s).
//!  * SELECT DIFFICULTY: STANDARD, HARD, BRUTAL, opening on the session's (FUN_000f1340: a new
//!    campaign's STANDARD); A stores game-options' -4, 0 or 4 (FUN_000f13c0).
//!  * JOIN CAMPAIGN: the panel and four quadrant menus (232 x 129) the controller places: the
//!    first controller's character choice h_ebad668d (its profile's name, Tex's picture, "Select
//!    Character" and the character spinner), after A its ready line h_e3cb03c9 ("Controller 1
//!    ready! Press START to begin!", FUN_000f2700), the others' h_1ca7cb7b ("Press START to Join").
//!
//! START on JOIN CAMPAIGN starts the campaign: its flags anew with the difficulty written
//! (play_flags.rs `Campaign::start_new`), the status line's "Saving...", "Saving Campaign...",
//! "Saving..." (take01 35.85-38.38 s: 1, 1 and 0.5 s) over the first mission's pre-movies, then
//! LOADING and the zone. The take freezes the screen for about half a second after DONE and after
//! START while the game saves (20.56-20.97 s, 35.35-35.83 s): the demo saves nothing (#27) and
//! goes on at once.
//!
//! Stubbed (profiles and saves are #27): the profiles' settings (INVERTED inverts nothing), New
//! Profile and the delete buttons (the error sound), no saved campaigns listed; QUICK CAMPAIGN,
//! RESUME LAST and the System Link entries give the error sound. The game's animations: the
//! buttons slide in from y 240 (translation-anims h_e2027fa6, h_f87fb560: 0.25 s), the texts fade
//! in (color-anim h_17494bb2: 0.5 s), the titles type out (style h_189b1ae9: 0.125 s a letter), the
//! selected button or row echoes (select-anim h_e150ead3), as play_menu.rs's.
//!
//! Test hook: play_menu.rs's BF_MENU_PRESS drives it (with BF_CAPTURE: menu_NNN.png).

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bf_viewer::bf::character::Game;
use bf_viewer::bf::menus::{GameMenu, GameMenus, ItemKind, Keyboard, MenuItem};

use super::super::briefing::{self, List, Then};
use super::super::fonts::{self, record_text, Across, Record, RecordText, HEADING24};
use super::super::mission::{self, endmenu};
use super::super::triggers::panels::{legend_icon, ICON12_ATLAS};
use super::{play, Asked, Clock, Menu, Screen, Sfx, ECHO_COLOR, ECHO_PULSE, ECHO_SCALE, ECHO_TIME, FADE_TIME, SLIDE_FROM, SLIDE_TIME,
            SND_ERROR, SND_MOVE, SND_OPEN, SND_SELECT, SND_SLIDE, TYPE_TIME};

/// The path's menus (game-options): CAMPAIGN TYPE, SELECT PROFILE, SELECT CAMPAIGN, ENTER NAME,
/// SELECT DIFFICULTY and JOIN CAMPAIGN (see the module notes).
const M_TYPE: u32 = 0xF972_8897;
const M_PROFILE: u32 = 0x1476_1942;
const M_SLOT: u32 = 0xE61A_E053;
const M_NAME: u32 = 0xE37B_5C35;
const M_DIFFICULTY: u32 = 0x1A68_84C5;
const M_JOIN: u32 = 0x0130_D4E8;
/// JOIN CAMPAIGN's quadrant menus: a controller choosing its character (h_ebad668d), ready
/// (h_e3cb03c9), and a free controller's (h_1ca7cb7b).
const M_CHARACTER: u32 = 0xEBAD_668D;
const M_READY: u32 = 0xE3CB_03C9;
const M_FREE: u32 = 0x1CA7_CB7B;
/// Where JOIN CAMPAIGN's controller puts the four quadrant menus (232 x 129 each, the first
/// controller's at the top left): h_1ca7cb7b's own place, (68, 114), and its neighbours. take01
/// at 31.5 s: the quadrants' lines at x 68, 300 and 531, y 114, 243 and 371.
const QUADRANTS: [Vec2; 4] = [Vec2::new(68.0, 114.0), Vec2::new(300.0, 114.0), Vec2::new(68.0, 243.0), Vec2::new(300.0, 243.0)];
/// The quadrant items the code fills: the profile's name (h_f420b712), the character's picture
/// (h_0aecc0f9), the lock over a locked one (h_139b76d9, left out: Tex isn't) and the character
/// spinner (h_f39f1075, a value control h_fbe6d17c).
const I_PROFILE_NAME: u32 = 0xF420_B712;
const I_PORTRAIT: u32 = 0x0AEC_C0F9;
const I_LOCK: u32 = 0x139B_76D9;
const SPINNER: u32 = 0xFBE6_D17C;
/// Tex's picture (campaign-bf.xmb `<squad-char char-id="tex" h_1b561f3f>`, 128 x 128) and his
/// name (h_eccb7800 "Tex", mp_tex's h_07f2f670 in game-options): the tutorial's squad is Tex alone.
const TEX_PICTURE: u32 = 0x0F35_D3D4;
const S_TEX: u32 = 0xECCB_7800;
/// The rows the code puts in the lists: the profiles DEFAULT (h_0a6de845) and INVERTED
/// (h_15a3164e), "New Profile" (h_e9fd23e1, 0xf66d8); "New Campaign" (h_ebb41e5c, FUN_000f1430);
/// and a new campaign's name, "CAMP" (h_eedaa81e) and its number (0x7e1f0).
const S_DEFAULT: u32 = 0x0A6D_E845;
const S_INVERTED: u32 = 0x15A3_164E;
const S_NEW_PROFILE: u32 = 0xE9FD_23E1;
const S_NEW_CAMPAIGN: u32 = 0xEBB4_1E5C;
const S_CAMP: u32 = 0xEEDA_A81E;
/// The keyboard's special keys' labels: BACK, EXTRA ALPHA, SPACE, DONE.
const KEY_BACK: u32 = 0x0451_5786;
const KEY_EXTRA: u32 = 0xFC72_6615;
const KEY_SPACE: u32 = 0xE978_2CC5;
const KEY_DONE: u32 = 0x043D_9074;
/// The animations the items name (game-options): the select-anim echoing the selected item.
const ANIM_ECHO: u32 = 0xE150_EAD3;
/// The color-anim fading an item in, and the translation-anims sliding one in (the first with
/// its sound, h_e8ace091).
const ANIM_FADE: u32 = 0x1749_4BB2;
const ANIM_SLIDE: [u32; 2] = [0xE202_7FA6, 0xF87F_B560];
/// The typed titles' style (h_189b1ae9, its h_f78a2264: 0.125 s a letter).
const STYLE_TYPED: u32 = 0x189B_1AE9;
/// The keyboard's name field: the box under the name (the default name stands selected), as take01
/// measures it at 20.6 s (199 199 254 over the text's width: a fit, the colour isn't in the style);
/// the selected key's word pulses between the style's disabled and selected colours once a
/// second (take01 18.0-20.6 s: brightest at 18.45, 19.45, 20.43 s: a fit), its key drawn in the
/// style's h_ed70ff4f (white), the others in its h_f679aef5 (100 100 100: take01's dim rims).
const FIELD_BOX: Color = Color::srgb(199.0 / 255.0, 199.0 / 255.0, 254.0 / 255.0);
const KEY_PULSE: f32 = 1.0;
/// A key is drawn 2 units in from each side of its slot (take01: keys 36 wide, 40 apart).
const KEY_INSET: f32 = 2.0;
/// A picture item whose texture isn't in the data (the quadrants' frame, h_19647c51) draws its
/// border, this wide, in its style's h_f679aef5 (153 209 251 at 100): take01's faint lines round
/// the quadrants (a reading of the footage: the code isn't traced).
const BORDER: f32 = 1.0;
/// A legend's word after its button: the 12-point icon glyph's offset and advance (1 + 17, as the
/// end screen's: play_endmenu.rs LEGEND_WORD_IN).
const LEGEND_WORD_IN: f32 = 18.0;
/// The legend for deleting (SELECT PROFILE's Delete Profile, SELECT CAMPAIGN's Delete Campaign,
/// h_1b0d073a): disabled with nothing to delete (a built-in profile, no campaign saved).
const I_DELETE: u32 = 0x1B0D_073A;

/// The path's screens (play_menu.rs `Screen::Campaign`), in take01's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Type,
    Profile,
    Slot,
    Name,
    Difficulty,
    Join,
}

impl Step {
    /// Its menu.
    fn menu(self) -> u32 {
        match self {
            Step::Type => M_TYPE,
            Step::Profile => M_PROFILE,
            Step::Slot => M_SLOT,
            Step::Name => M_NAME,
            Step::Difficulty => M_DIFFICULTY,
            Step::Join => M_JOIN,
        }
    }
}

/// Where the player is on the path: each screen's choice.
#[derive(Clone, Default, Debug)]
pub struct Path {
    /// CAMPAIGN TYPE's button (0 CAMPAIGN .. 4 JOIN SESSION)
    kind: usize,
    /// SELECT PROFILE's row (0 DEFAULT, 1 INVERTED, 2 New Profile) and the profile taken
    profile: usize,
    /// SELECT CAMPAIGN's row
    slot: usize,
    /// ENTER NAME: the name, whether it's still the default (shown selected: the first key typed
    /// replaces it), the key selected (its index in the data) and the EXTRA set on
    name: String,
    fresh: bool,
    key: usize,
    extra: bool,
    /// SELECT DIFFICULTY's button (0 STANDARD, 1 HARD, 2 BRUTAL)
    difficulty: usize,
    /// JOIN CAMPAIGN: the first controller has picked its character
    joined: bool,
}

/// A texture's pixels: its width, height and RGBA rows.
type Rgba = (u32, u32, Vec<u8>);

/// The path's data, gathered with the front end's (play_menu.rs `gather`): game-options' menus,
/// the textures they draw, the difficulties and the first mission.
pub struct CampaignData {
    /// game-options' menus, styles and picture sets
    menus: GameMenus,
    /// the textures the screens draw, by name: the panels' pieces, the list arrows (h_1cc0d333),
    /// the spinner's (h_158f87c1), the keys (h_e8d2f863, h_f942b4b2, h_e38f7784), the pad
    /// (h_e8047c6d), Tex's picture and the legends' icon font (ICON12_ATLAS)
    textures: Vec<(u32, Rgba)>,
    /// see CampaignMenus
    difficulties: [i64; 3],
    first: Option<(u32, String)>,
}

/// The path's data in the world (see CampaignData): the textures as images with their sizes.
#[derive(Resource)]
pub struct CampaignMenus {
    /// game-options' menus, styles and picture sets
    menus: GameMenus,
    /// each texture's image and its own size
    images: HashMap<u32, (Handle<Image>, Vec2)>,
    /// game-options' `<difficulty-settings>`: STANDARD, HARD, BRUTAL's n (-4, 0, 4)
    difficulties: [i64; 3],
    /// the first mission: its level's id and its first zone's archive (m00, the tutorial)
    first: Option<(u32, String)>,
}

/// Gather the path's data (see CampaignData).
pub fn gather(game: &mut Game) -> CampaignData {
    let menus = game.menus.clone();
    let mut wanted: Vec<u32> = vec![TEX_PICTURE, ICON12_ATLAS];
    for m in [M_TYPE, M_PROFILE, M_SLOT, M_NAME, M_DIFFICULTY, M_JOIN, M_CHARACTER, M_READY, M_FREE] {
        let Some(menu) = menus.menus.get(&m) else { continue };
        for item in &menu.items {
            if let ItemKind::Picture { set, piece } = &item.kind {
                wanted.extend(menus.pictures.get(set).into_iter().flatten().map(|p| p.texture));
                wanted.extend(piece.iter().map(|p| p.texture));
            }
            if let Some(s) = menus.styles.get(&item.style) {
                wanted.extend(s.textures);
            }
        }
    }
    wanted.retain(|&t| t != 0);
    wanted.sort();
    wanted.dedup();
    let textures = wanted.into_iter().filter_map(|t| game.texture_rgba(t).map(|px| (t, px))).collect();
    let first = game.campaign.iter().find(|l| l.kind == mission::LEVEL_MISSION).map(|l| (l.id, l.zones.first().cloned().unwrap_or_else(|| l.file.clone())));
    CampaignData { menus, textures, difficulties: game.difficulty_settings.unwrap_or([-4, 0, 4]), first }
}

/// Put the path's data in the world (its textures as images).
pub fn install(world: &mut World, data: CampaignData) {
    let CampaignData { menus, textures, difficulties, first } = data;
    let mut assets = world.resource_mut::<Assets<Image>>();
    let images = textures.into_iter().map(|(t, (w, hgt, px))| (t, (assets.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, px, bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default())), Vec2::new(w as f32, hgt as f32)))).collect();
    world.insert_resource(CampaignMenus { menus, images, difficulties, first });
}

/// What the path's input needs besides the front end's own: its data, the status line, the
/// campaign's flags (a new campaign) and the state to go on to.
#[derive(SystemParam)]
pub struct PathParams<'w> {
    cm: Option<Res<'w, CampaignMenus>>,
    console: ResMut<'w, mission::Console>,
    campaign: ResMut<'w, super::super::flags::Campaign>,
}

/// A text sliding in to its place as its screen opens (a translation-anim: from y 240 to its top,
/// `home`).
#[derive(Component)]
pub(super) struct SlideIn {
    home: f32,
}
/// A text fading in as its screen opens (color-anim h_17494bb2).
#[derive(Component)]
pub(super) struct FadeIn;
/// A static text's fill fading in with it (its colour at full).
#[derive(Component)]
pub(super) struct FadeFill(Color);
/// A title typing out (style h_189b1ae9): its whole text.
#[derive(Component)]
pub(super) struct Typed(String);
/// The selected item's echo (select-anim h_e150ead3): its centre, when the echo began and whether
/// it's past its first (pulsing).
#[derive(Component)]
pub(super) struct Echo {
    centre: Vec2,
    born: f32,
    pulsing: bool,
}
/// The selected key's word, pulsing (see KEY_PULSE).
#[derive(Component)]
pub(super) struct Pulse;

/// A colour from a style's RGBA.
fn rgba(c: [u8; 4]) -> Color {
    Color::srgba_u8(c[0], c[1], c[2], c[3])
}

/// A string of the string table (empty if it isn't there).
fn string(strings: &HashMap<u32, String>, id: u32) -> String {
    strings.get(&id).cloned().unwrap_or_default()
}

/// The first word of a key's label (its character; "EXTRA" of "EXTRA ALPHA"), or the EXTRA set's
/// (the label's second word) when that's on and the key has one.
fn key_word(strings: &HashMap<u32, String>, label: u32, extra: bool) -> String {
    let s = string(strings, label);
    let mut words = s.split_whitespace();
    let first = words.next().unwrap_or_default().to_string();
    match words.next() {
        Some(second) if extra && label != KEY_EXTRA => second.to_string(),
        _ => first,
    }
}

/// A line of text in a record on `screen` with its shadow, placed in a box by an alignment (across
/// 0 left, 1 right, 2 centred; down 0 top, 1 bottom, 2 centred, as play_endmenu.rs `text_item`):
/// the node and the line's centre.
#[allow(clippy::too_many_arguments)]
fn line_in(commands: &mut Commands, screen: Entity, text: &str, corner: Vec2, size: Vec2, record: &'static Record, colour: Color, shadow: Option<Color>,
           align: (i64, i64)) -> (Entity, Vec2) {
    let pitch = record.height();
    let top = match align.1 {
        2 => corner.y + ((size.y - pitch) * 0.5).floor(),
        1 => corner.y + size.y - pitch,
        _ => corner.y,
    };
    let width = record.measure(text);
    let (x, how, left) = match align.0 {
        1 => (corner.x + size.x, Across::Right, (corner.x + size.x - width).floor()),
        2 => (corner.x + size.x * 0.5, Across::Centre, (corner.x + size.x * 0.5 - width * 0.5).floor()),
        _ => (corner.x, Across::Left, corner.x),
    };
    let e = commands.spawn((record_text(text, x, top, record, vec![colour], shadow, how), ChildOf(screen))).id();
    (e, Vec2::new(left + width * 0.5, top + pitch * 0.5))
}

/// An echo of a line (select-anim h_e150ead3): a copy of it growing from its centre and fading.
fn echo(commands: &mut Commands, screen: Entity, text: &str, record: &'static Record, centre: Vec2, now: f32) {
    let [r, g, b, a] = ECHO_COLOR;
    commands.spawn((record_text(text, centre.x, centre.y - record.height() * 0.5, record, vec![Color::srgba_u8(r, g, b, a)], None, Across::Centre),
                    Echo { centre, born: now, pulsing: false }, ChildOf(screen)));
}

/// A picture on `screen` at a place and size, flipped as the game's pictures are stored (bottom
/// row first), in a tint.
fn picture(commands: &mut Commands, screen: Entity, image: &Handle<Image>, at: Vec2, size: Vec2, tint: Color, flip_x: bool, flip_y: bool) -> Entity {
    commands.spawn((super::super::hud::at(at.x, at.y, size.x, size.y), ImageNode { flip_x, flip_y, ..ImageNode::new(image.clone()) }.with_color(tint),
                    ChildOf(screen))).id()
}

/// A menu's picture items (its sets' pieces or its own piece) at `origin`: a texture at its own
/// size or stretched, in its piece's colour or `tint`; one the data hasn't got, its border
/// (BORDER). `fill` gives the code's picture for an item without a texture (JOIN's character
/// picture).
fn pictures(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, menu: &GameMenu, origin: Vec2, fill: &dyn Fn(u32) -> Option<u32>,
            tint: Option<Color>) {
    for item in &menu.items {
        let ItemKind::Picture { set, piece } = &item.kind else { continue };
        let corner = origin + item.at;
        let pieces = if *set != 0 { cm.menus.pictures.get(set).cloned().unwrap_or_default() } else { piece.iter().cloned().collect() };
        let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
        for pc in pieces {
            let texture = if pc.texture == 0 { fill(item.name) } else { Some(pc.texture) };
            let Some(texture) = texture else { continue };
            let at = corner + pc.at;
            let colour = tint.unwrap_or(rgba(pc.color));
            match cm.images.get(&texture) {
                Some((image, own)) => {
                    let size = if pc.stretched { pc.size } else { *own };
                    picture(commands, screen, image, at, size, colour, false, !pc.flipped);
                }
                None => {
                    // (no such texture: its border)
                    let Some(c) = style.fill else { continue };
                    let c = rgba(c);
                    for (x, y, w, hgt) in [(at.x, at.y, pc.size.x, BORDER), (at.x, at.y + pc.size.y - BORDER, pc.size.x, BORDER),
                                           (at.x, at.y, BORDER, pc.size.y), (at.x + pc.size.x - BORDER, at.y, BORDER, pc.size.y)] {
                        commands.spawn((super::super::hud::at(x, y, w, hgt), BackgroundColor(c), ChildOf(screen)));
                    }
                }
            }
        }
    }
}

/// A legend item (style h_e35bb628): its button's icon in the 12-point icon font and its word in
/// the body font at 12 points, in the style's colour, or its disabled colour (a delete with nothing
/// to delete: take01's grey "Delete Profile").
#[allow(clippy::too_many_arguments)]
fn legend(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, origin: Vec2, item: &MenuItem, button: u32,
          label: u32, enabled: bool) {
    let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
    let corner = origin + item.at;
    let letter = string(strings, button).chars().next().unwrap_or(' ');
    let grey = style.disabled.map(rgba).unwrap_or(Color::srgb(0.4, 0.4, 0.4));
    if let Some((node, mut image)) = cm.images.get(&ICON12_ATLAS).and_then(|(i, _)| legend_icon(i.clone(), corner, letter)) {
        if !enabled {
            image.color = grey;
        }
        commands.spawn((node, image, ChildOf(screen)));
    }
    let colour = if enabled { style.color.map_or(Color::WHITE, rgba) } else { grey };
    commands.spawn((record_text(&string(strings, label), corner.x + LEGEND_WORD_IN, corner.y, &fonts::BODY12, vec![colour], style.edge.map(rgba), Across::Left),
                    ChildOf(screen)));
}

/// A screen's static texts (see `endmenu::text_item`), but for the buttons' descriptions not
/// selected (`hidden`) and the ones the code fills (`filled`: their text, or left out): each with
/// its animations (fade in; the title typed), which run on from when the screen opened.
#[allow(clippy::too_many_arguments)]
fn texts(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, menu: &GameMenu, origin: Vec2, hidden: &[u32],
         filled: &dyn Fn(u32) -> Option<Option<String>>) {
    for item in &menu.items {
        let ItemKind::Text(id) = item.kind else { continue };
        if hidden.contains(&item.name) {
            continue;
        }
        let text = match filled(item.name) {
            Some(Some(t)) => t,
            Some(None) => continue,
            None => string(strings, id),
        };
        if text.is_empty() {
            continue;
        }
        let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let nodes = endmenu::text_item(commands, screen, origin, item, &style, &text, None, 0.0);
        let fades = item.anims.contains(&ANIM_FADE);
        let has_fill = style.fill.is_some();
        for (k, &e) in nodes.iter().enumerate() {
            if k == 0 && has_fill {
                if fades {
                    commands.entity(e).insert(FadeFill(rgba(style.fill.unwrap_or([0; 4]))));
                }
                continue;
            }
            if fades {
                commands.entity(e).insert(FadeIn);
            }
            if item.style == STYLE_TYPED {
                commands.entity(e).insert(Typed(text.clone()));
            }
        }
    }
}

/// The buttons of a button menu (CAMPAIGN TYPE, SELECT DIFFICULTY) in their styles: the selected
/// one in `color-selected` with its echo, the others in `color-normal`; each sliding and fading
/// in as the screen opens (its translation- and color-anims). Returns the buttons' buddies (the
/// descriptions not selected, hidden).
fn buttons(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, menu: &GameMenu, pick: usize, now: f32) -> Vec<u32> {
    let mut hidden = vec![];
    let all: Vec<&MenuItem> = menu.items.iter().filter(|i| matches!(i.kind, ItemKind::Button { .. })).collect();
    for (k, item) in all.iter().enumerate() {
        let ItemKind::Button { text, buddies } = &item.kind else { continue };
        if k != pick {
            hidden.extend(buddies);
        }
        let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let record = fonts::record_of(&style);
        let colour = if k == pick { style.selected.or(style.color) } else { style.color }.map_or(Color::WHITE, rgba);
        let word = string(strings, *text);
        let (e, centre) = line_in(commands, screen, &word, menu.at + item.at, item.size, record, colour, style.edge.map(rgba), style.align);
        if item.anims.iter().any(|a| ANIM_SLIDE.contains(a)) {
            let home = centre.y - record.height() * 0.5;
            commands.entity(e).insert(SlideIn { home });
        }
        if item.anims.contains(&ANIM_FADE) {
            commands.entity(e).insert(FadeIn);
        }
        if k == pick && item.anims.contains(&ANIM_ECHO) {
            echo(commands, screen, &word, record, centre, now);
        }
    }
    hidden
}

/// A listbox's rows (SELECT PROFILE, SELECT CAMPAIGN): `display-count` rows in its height, each
/// centred in its place, the selected one in `color-selected` with its echo; the list's arrows
/// (its style's texture-1) above and below it.
fn list(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, menu: &GameMenu, rows: &[String], pick: usize, now: f32) {
    for item in &menu.items {
        let ItemKind::List { rows: shown, .. } = item.kind else { continue };
        let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let record = fonts::record_of(&style);
        let corner = menu.at + item.at;
        let pitch = item.size.y / shown.max(1) as f32;
        for (k, row) in rows.iter().enumerate().take(shown as usize) {
            let colour = if k == pick { style.selected.or(style.color) } else { style.color }.map_or(Color::WHITE, rgba);
            // (a black shadow, as take01's rows have at 17.0 s: the style's h_ed70ff4f is white)
            let (e, centre) = line_in(commands, screen, row, Vec2::new(corner.x, corner.y + k as f32 * pitch), Vec2::new(item.size.x, pitch), record, colour,
                                      Some(Color::BLACK), (2, 2));
            if item.anims.contains(&ANIM_FADE) {
                commands.entity(e).insert(FadeIn);
            }
            if k == pick && item.anims.contains(&ANIM_ECHO) {
                echo(commands, screen, row, record, centre, now);
            }
        }
        // the arrows: the texture as the game draws it points down (below the list); mirrored,
        // up (above it). take01 12.2 s: their ink at y 119-139 and 353-370
        if let Some((image, own)) = cm.images.get(&style.textures[0]) {
            let x = corner.x + (item.size.x - own.x) * 0.5;
            picture(commands, screen, image, Vec2::new(x, corner.y - own.y), *own, Color::WHITE, false, false);
            picture(commands, screen, image, Vec2::new(x, corner.y + item.size.y), *own, Color::WHITE, false, true);
        }
    }
}

/// The keyboard's slots: each key's box (its first slot's place to its last's), the slots centred
/// in the item's width and in its height under the name field (a fit: take01's first key at
/// (105.3, 168)).
fn key_boxes(kb: &Keyboard, corner: Vec2, size: Vec2) -> Vec<(Vec2, Vec2)> {
    let across = kb.across.max(1);
    let left = corner.x + (size.x - across as f32 * kb.slot) * 0.5;
    let top = corner.y + kb.field + (size.y - kb.field - kb.rows as f32 * kb.slot) * 0.5;
    kb.keys.iter().map(|k| {
        let (row, col) = (k.slot / across, k.slot % across);
        (Vec2::new(left + col as f32 * kb.slot + KEY_INSET, top + row as f32 * kb.slot + KEY_INSET),
         Vec2::new(k.width as f32 * kb.slot - 2.0 * KEY_INSET, kb.slot - 2.0 * KEY_INSET))
    }).collect()
}

/// ENTER NAME's keyboard (see the module notes): the name in its field (in the style's selected
/// colour with a black shadow, take01's at 20.6 s), each key's picture (texture-1 one slot wide;
/// a wider one texture-2's end, texture-3 stretched, texture-2 mirrored) and word, the selected
/// key's lit and its word pulsing.
fn keyboard(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, menu: &GameMenu, path: &Path) {
    for item in &menu.items {
        let ItemKind::Keyboard(kb) = &item.kind else { continue };
        let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let corner = menu.at + item.at;
        let record: &'static Record = fonts::record_of(&style);
        // the name, centred at the top in the field's height, over its box while it's the default
        let width = record.measure(&path.name);
        let top = corner.y + ((kb.field - record.height()) * 0.5).floor();
        let left = (corner.x + (item.size.x - width) * 0.5).floor();
        if path.fresh && !path.name.is_empty() {
            commands.spawn((super::super::hud::at(left, top, width, record.height()), BackgroundColor(FIELD_BOX), ChildOf(screen)));
        }
        let selected = style.selected.map_or(Color::WHITE, rgba);
        commands.spawn((record_text(&path.name, corner.x + item.size.x * 0.5, top, record, vec![selected], Some(Color::BLACK), Across::Centre),
                        ChildOf(screen)));
        let lit = style.edge.map_or(Color::WHITE, rgba);
        let dim = style.fill.map_or(Color::srgb(0.4, 0.4, 0.4), rgba);
        let normal = style.color.map_or(Color::WHITE, rgba);
        let tex = |k: usize| cm.images.get(&style.textures[k]);
        for (k, (key, (at, size))) in kb.keys.iter().zip(key_boxes(kb, corner, item.size)).enumerate() {
            let on = k == path.key;
            let tint = if on { lit } else { dim };
            if key.width <= 1 {
                if let Some((image, _)) = tex(0) {
                    picture(commands, screen, image, at, size, tint, false, true);
                }
            } else if let (Some((end, _)), Some((middle, _))) = (tex(1), tex(2)) {
                let cap = size.y;
                picture(commands, screen, end, at, Vec2::new(cap, size.y), tint, false, true);
                picture(commands, screen, middle, at + Vec2::new(cap, 0.0), Vec2::new(size.x - 2.0 * cap, size.y), tint, false, true);
                picture(commands, screen, end, at + Vec2::new(size.x - cap, 0.0), Vec2::new(cap, size.y), tint, true, true);
            }
            let word = key_word(strings, key.label, path.extra);
            let (e, _) = line_in(commands, screen, &word, at, size, &HEADING24, normal, None, (2, 2));
            if on {
                commands.entity(e).insert(Pulse);
            }
        }
    }
}

/// JOIN CAMPAIGN's quadrants (see the module notes): the first controller's character choice or
/// its ready line, the free controllers' "Press START to Join" (their items in their styles'
/// disabled colours, as take01 shows them dim).
fn quadrants(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, path: &Path) {
    let profile = string(strings, if path.profile == 1 { S_INVERTED } else { S_DEFAULT });
    for (q, origin) in QUADRANTS.into_iter().enumerate() {
        let name = if q > 0 { M_FREE } else if path.joined { M_READY } else { M_CHARACTER };
        let Some(menu) = cm.menus.menus.get(&name) else { continue };
        let active = q == 0;
        // the pictures first: the frame, the character's picture (the lock left out), the pad (a
        // free controller's in its style's disabled colour)
        let fill = |n: u32| if n == I_PORTRAIT { Some(TEX_PICTURE) } else { None };
        for item in menu.items.iter().filter(|i| i.name != I_LOCK) {
            let one = GameMenu { items: vec![item.clone()], ..default() };
            let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
            let textured = matches!(&item.kind, ItemKind::Picture { piece: Some(p), .. } if cm.images.contains_key(&p.texture));
            let tint = if !active && textured { style.disabled.map(rgba) } else { None };
            pictures(commands, screen, cm, &one, origin, &fill, tint);
        }
        for item in &menu.items {
            let style = cm.menus.styles.get(&item.style).cloned().unwrap_or_default();
            let corner = origin + item.at;
            match &item.kind {
                ItemKind::Button { text, .. } => {
                    let colour = if active { style.selected.or(style.color) } else { style.disabled }.map_or(Color::WHITE, rgba);
                    let word = string(strings, *text).replace("%d", "1");
                    let record = fonts::record_of(&style);
                    let lines: Vec<&str> = word.split('\n').collect();
                    let pitch = record.height();
                    let top = corner.y + ((item.size.y - lines.len() as f32 * pitch) * 0.5).floor();
                    for (n, l) in lines.iter().enumerate() {
                        line_in(commands, screen, l, Vec2::new(corner.x, top + n as f32 * pitch), Vec2::new(item.size.x, pitch), record, colour,
                                style.edge.map(rgba), (2, 0));
                    }
                }
                ItemKind::Text(_) if item.name == I_PROFILE_NAME => {
                    // (in the style's normal colour, as take01 shows it at 31.5 s, not a static
                    // text's disabled grey: read from the footage, the code that fills it isn't
                    // traced)
                    let colour = style.color.map_or(Color::WHITE, rgba);
                    line_in(commands, screen, &profile, corner, item.size, fonts::record_of(&style), colour, style.edge.map(rgba), style.align);
                }
                ItemKind::Other(SPINNER) => {
                    // the character's name between the style's arrows (texture-1, the right one
                    // mirrored): take01 31.5 s, "Tex" centred at 184, the arrows' ink from 75 and 273
                    let colour = style.selected.or(style.color).map_or(Color::WHITE, rgba);
                    let shadow = Some(Color::BLACK);
                    line_in(commands, screen, &string(strings, S_TEX), corner, item.size, fonts::record_of(&style), colour, shadow, (2, 2));
                    if let Some((image, own)) = cm.images.get(&style.textures[0]) {
                        let y = corner.y + (item.size.y - own.y) * 0.5;
                        picture(commands, screen, image, Vec2::new(corner.x, y), *own, Color::WHITE, false, true);
                        picture(commands, screen, image, Vec2::new(corner.x + item.size.x - own.x, y), *own, Color::WHITE, true, true);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Build a screen of the path (play_menu.rs `build` calls it when the menu changes): its panel and
/// pictures, texts, buttons, list or keyboard, quadrants and legends, their animations running
/// from when it opened (the menu's Clock); its first item's sound when it has just `opened`.
#[allow(clippy::too_many_arguments)]
pub fn build(commands: &mut Commands, screen: Entity, cm: &CampaignMenus, strings: &HashMap<u32, String>, sfx: &Sfx, step: Step, path: &Path,
             now: f32, opened: bool) {
    let Some(menu) = cm.menus.menus.get(&step.menu()) else {
        eprintln!("menu: game-options has no menu h_{:08x}", step.menu());
        return;
    };
    let none = |_: u32| None;
    pictures(commands, screen, cm, menu, menu.at, &none, None);
    let hidden = match step {
        Step::Type => buttons(commands, screen, cm, strings, menu, path.kind, now),
        Step::Difficulty => buttons(commands, screen, cm, strings, menu, path.difficulty, now),
        _ => vec![],
    };
    // (SELECT PROFILE's detail texts are the profile's settings: left empty, #27)
    texts(commands, screen, cm, strings, menu, menu.at, &hidden, &|_| None);
    match step {
        Step::Profile => list(commands, screen, cm, menu, &profile_rows(strings), path.profile, now),
        Step::Slot => list(commands, screen, cm, menu, &slot_rows(strings), path.slot, now),
        Step::Name => keyboard(commands, screen, cm, strings, menu, path),
        Step::Join => quadrants(commands, screen, cm, strings, path),
        _ => {}
    }
    for item in &menu.items {
        if let ItemKind::Legend { button, label } = item.kind {
            // (the deletes have nothing to delete: a built-in profile, no campaign saved)
            let enabled = !(matches!(step, Step::Profile | Step::Slot) && item.name == I_DELETE);
            legend(commands, screen, cm, strings, menu.at, item, button, label, enabled);
        }
    }
    if opened {
        // the first item's translation-anim sound (h_e2027fa6's h_e8ace091)
        if menu.items.iter().any(|i| i.anims.first() == Some(&ANIM_SLIDE[0])) {
            play(commands, sfx, SND_SLIDE);
        }
    }
}

/// SELECT PROFILE's rows: the profiles, then "New Profile" (0xf66d8).
fn profile_rows(strings: &HashMap<u32, String>) -> Vec<String> {
    [S_DEFAULT, S_INVERTED, S_NEW_PROFILE].iter().map(|&s| string(strings, s)).collect()
}

/// SELECT CAMPAIGN's rows (FUN_000f1430: the campaigns saved, none here, then "New Campaign").
fn slot_rows(strings: &HashMap<u32, String>) -> Vec<String> {
    vec![string(strings, S_NEW_CAMPAIGN)]
}

/// The path's opening: CAMPAIGN TYPE on its first button.
pub fn open() -> (Screen, Path) {
    (Screen::Campaign(Step::Type), Path::default())
}

/// ENTER NAME's start: a new campaign's name (0x7e1f0: "CAMP" and the first number not taken, 1
/// with nothing saved), standing selected, the selection on DONE.
fn new_name(path: &mut Path, strings: &HashMap<u32, String>, cm: &CampaignMenus) {
    path.name = format!("{}{:03}", string(strings, S_CAMP), 1);
    path.fresh = true;
    path.extra = false;
    path.key = cm.menus.menus.get(&M_NAME).and_then(|m| m.items.iter().find_map(|i| match &i.kind {
        ItemKind::Keyboard(kb) => kb.keys.iter().position(|k| k.label == KEY_DONE),
        _ => None,
    })).unwrap_or(0);
}

/// BF_MENU_SCREEN's screens of the path (campaign, profile, slot, name, difficulty, join): the
/// screen with its first choices, as a run through the path leaves them.
pub fn for_shot(name: &str, strings: &HashMap<u32, String>, cm: Option<&CampaignMenus>) -> (Screen, Path) {
    let step = match name {
        "profile" => Step::Profile,
        "slot" => Step::Slot,
        "name" => Step::Name,
        "difficulty" => Step::Difficulty,
        "join" => Step::Join,
        _ => Step::Type,
    };
    let mut path = Path::default();
    if let Some(cm) = cm {
        new_name(&mut path, strings, cm);
    }
    (Screen::Campaign(step), path)
}

/// Step a selection by one up or down within `n`, the error sound at the ends (as the main
/// menu's): whether it moved.
fn step_pick(commands: &mut Commands, sfx: &Sfx, pick: &mut usize, n: usize, asked: &Asked) -> bool {
    if !(asked.up || asked.down) {
        return false;
    }
    let to = if asked.up { pick.checked_sub(1) } else { Some(*pick + 1).filter(|&i| i < n) };
    match to {
        Some(i) => {
            *pick = i;
            play(commands, sfx, SND_MOVE);
        }
        None => play(commands, sfx, SND_ERROR),
    }
    true
}

/// Open the next screen with the select and open sounds.
fn go_to(commands: &mut Commands, sfx: &Sfx, menu: &mut Menu, step: Step) {
    menu.screen = Screen::Campaign(step);
    play(commands, sfx, SND_SELECT);
    play(commands, sfx, SND_OPEN);
}

/// A screen of the path takes the frame's presses (see the module notes).
pub fn input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, params: &mut PathParams, strings: &HashMap<u32, String>,
             next: &mut NextState<super::super::AppState>) {
    let Screen::Campaign(step) = menu.screen else { return };
    let Some(cm) = params.cm.as_deref() else { return };
    match step {
        Step::Type => {
            let n = cm.menus.menus.get(&M_TYPE).map_or(0, |m| m.items.iter().filter(|i| matches!(i.kind, ItemKind::Button { .. })).count());
            let mut kind = menu.path.kind;
            if step_pick(commands, sfx, &mut kind, n, asked) {
                menu.path.kind = kind;
            } else if asked.select {
                // CAMPAIGN runs the sequence; the rest aren't here (#27, System Link)
                if kind == 0 {
                    menu.path.profile = 0;
                    go_to(commands, sfx, menu, Step::Profile);
                } else {
                    play(commands, sfx, SND_ERROR);
                }
            } else if asked.back {
                menu.screen = Screen::Main;
                menu.main = 0;
            }
        }
        Step::Profile => {
            let mut pick = menu.path.profile;
            if step_pick(commands, sfx, &mut pick, profile_rows(strings).len(), asked) {
                menu.path.profile = pick;
            } else if asked.select {
                // a profile (New Profile is #27's)
                if pick < 2 {
                    menu.path.slot = 0;
                    go_to(commands, sfx, menu, Step::Slot);
                } else {
                    play(commands, sfx, SND_ERROR);
                }
            } else if asked.back {
                menu.screen = Screen::Campaign(Step::Type);
            }
        }
        Step::Slot => {
            let mut pick = menu.path.slot;
            if step_pick(commands, sfx, &mut pick, slot_rows(strings).len(), asked) {
                menu.path.slot = pick;
            } else if asked.select {
                // New Campaign: ENTER NAME, the new name standing selected, on DONE
                new_name(&mut menu.path, strings, cm);
                go_to(commands, sfx, menu, Step::Name);
            } else if asked.back {
                menu.screen = Screen::Campaign(Step::Profile);
            }
        }
        Step::Name => name_input(commands, menu, asked, sfx, cm, strings, &mut params.console),
        Step::Difficulty => {
            let mut pick = menu.path.difficulty;
            if step_pick(commands, sfx, &mut pick, 3, asked) {
                menu.path.difficulty = pick;
            } else if asked.select {
                // (SELECT MISSION passed: the new campaign's one mission)
                menu.path.joined = false;
                go_to(commands, sfx, menu, Step::Join);
            } else if asked.back {
                menu.screen = Screen::Campaign(Step::Slot);
            }
        }
        Step::Join => {
            if asked.left || asked.right {
                // the spinner has Tex alone (0x1038e0's menu_error)
                if !menu.path.joined {
                    play(commands, sfx, SND_ERROR);
                }
            } else if (asked.start || asked.select) && menu.path.joined {
                begin(commands, menu, sfx, cm, strings, &mut params.console, &mut params.campaign, next);
            } else if asked.select || asked.start {
                menu.path.joined = true;
                play(commands, sfx, SND_SELECT);
            } else if asked.back {
                if menu.path.joined {
                    menu.path.joined = false;
                } else {
                    menu.screen = Screen::Campaign(Step::Difficulty);
                }
            }
        }
    }
}

/// ENTER NAME's presses: the d-pad moves over the keys (round at the edges), A types the key
/// (BACK takes the last character, SPACE adds one, EXTRA ALPHA switches the set, DONE goes on and
/// saves), B goes back.
#[allow(clippy::too_many_arguments)]
fn name_input(commands: &mut Commands, menu: &mut Menu, asked: &Asked, sfx: &Sfx, cm: &CampaignMenus, strings: &HashMap<u32, String>,
              console: &mut mission::Console) {
    let Some(kb) = cm.menus.menus.get(&M_NAME).and_then(|m| m.items.iter().find_map(|i| match &i.kind {
        ItemKind::Keyboard(kb) => Some(kb.clone()),
        _ => None,
    })) else { return };
    if kb.keys.is_empty() {
        return;
    }
    let across = kb.across.max(1);
    let path = &mut menu.path;
    let at = |slot: u32| kb.keys.iter().position(|k| (k.slot..k.slot + k.width).contains(&slot));
    let key = &kb.keys[path.key.min(kb.keys.len() - 1)];
    let (row, col) = (key.slot / across, key.slot % across);
    let rows = kb.rows.max(1);
    let to = if asked.left {
        at(row * across + (col + across - 1) % across)
    } else if asked.right {
        at(row * across + (col + key.width) % across)
    } else if asked.up {
        at(((row + rows - 1) % rows) * across + col)
    } else if asked.down {
        at(((row + 1) % rows) * across + col)
    } else {
        None
    };
    if let Some(k) = to {
        path.key = k;
        play(commands, sfx, SND_MOVE);
        return;
    }
    if asked.back {
        menu.screen = Screen::Campaign(Step::Slot);
        return;
    }
    if !asked.select {
        return;
    }
    let label = key.label;
    let longest = kb.longest.max(1) as usize;
    match label {
        KEY_DONE => {
            if path.name.trim().is_empty() {
                play(commands, sfx, SND_ERROR);
                return;
            }
            // the campaign saved (the status line; #27 saves nothing)
            console.say(&string(strings, mission::S_SAVING), 1.0);
            path.difficulty = 0;
            go_to(commands, sfx, menu, Step::Difficulty);
        }
        KEY_BACK => {
            if path.fresh {
                path.name.clear();
            } else {
                path.name.pop();
            }
            path.fresh = false;
            play(commands, sfx, SND_SELECT);
        }
        KEY_EXTRA => {
            path.extra = !path.extra;
            play(commands, sfx, SND_SELECT);
        }
        _ => {
            let c = if label == KEY_SPACE { " ".to_string() } else { key_word(strings, label, path.extra) };
            if path.fresh {
                path.name.clear();
                path.fresh = false;
            }
            if path.name.chars().count() < longest {
                path.name.push_str(&c);
                play(commands, sfx, SND_SELECT);
            } else {
                play(commands, sfx, SND_ERROR);
            }
        }
    }
}

/// JOIN CAMPAIGN's START (see the module notes): the new campaign at the difficulty chosen, the
/// status line's saves, the first mission's pre-movies, then LOADING and its first zone.
#[allow(clippy::too_many_arguments)]
fn begin(commands: &mut Commands, menu: &mut Menu, sfx: &Sfx, cm: &CampaignMenus, strings: &HashMap<u32, String>, console: &mut mission::Console,
         campaign: &mut super::super::flags::Campaign, next: &mut NextState<super::super::AppState>) {
    let Some((mission_id, zone)) = cm.first.clone() else {
        eprintln!("menu: the level list has no mission to start");
        play(commands, sfx, SND_ERROR);
        return;
    };
    let n = cm.difficulties[menu.path.difficulty.min(2)];
    campaign.start_new(n);
    for (text, secs) in [(mission::S_SAVING, 1.0), (mission::S_SAVING_CAMPAIGN, 1.0), (mission::S_SAVING, 0.5)] {
        console.say(&string(strings, text), secs);
    }
    if std::env::var("BF_MISSION_LOG").is_ok() {
        println!("menu: a new campaign \"{}\" at difficulty {n}: mission h_{mission_id:08x}, {zone}", menu.path.name);
    }
    play(commands, sfx, SND_SELECT);
    briefing::start(commands, next, mission_id, 0, List::Pre, Then::Zone(zone));
}

/// The path's animations (see the module notes), on the menu's clock: the texts sliding, fading
/// and typing in as their screen opens, the echoes growing and fading (then pulsing), the selected
/// key's word pulsing.
pub fn animate(time: Res<Time>, clock: Res<Clock>, mut texts: Query<(&mut RecordText, Option<&SlideIn>, Option<&FadeIn>, Option<&Typed>, Option<&mut Echo>,
               Option<&Pulse>)>, mut fills: Query<(&FadeFill, &mut BackgroundColor)>, cm: Option<Res<CampaignMenus>>) {
    let now = time.elapsed_secs();
    let t = now - clock.opened;
    let fade = (t / FADE_TIME).clamp(0.0, 1.0);
    let k = (t / SLIDE_TIME).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - k) * (1.0 - k);
    for (mut r, slide, fade_in, typed, echo, pulse) in &mut texts {
        if let Some(s) = slide {
            let y = SLIDE_FROM + (s.home - SLIDE_FROM) * ease;
            if r.y != y {
                r.y = y;
            }
        }
        if fade_in.is_some() && r.alpha != fade {
            r.alpha = fade;
        }
        if let Some(Typed(full)) = typed {
            let shown = ((t / TYPE_TIME) as usize + 1).min(full.chars().count());
            let text: String = full.chars().take(shown).collect();
            if r.text != text {
                r.text = text;
            }
        }
        if let Some(mut e) = echo {
            // as play_menu.rs's echo: the select-anim's 0.5 s, then a pulse every ECHO_PULSE
            let length = if e.pulsing { ECHO_PULSE } else { ECHO_TIME };
            if now - e.born >= length {
                e.born = now;
                e.pulsing = true;
            }
            let k = ((now - e.born) / if e.pulsing { ECHO_PULSE } else { ECHO_TIME }).clamp(0.0, 1.0);
            let scale = 1.0 + (ECHO_SCALE - 1.0) * k;
            let [cr, cg, cb, ca] = ECHO_COLOR;
            let colour = Color::srgba_u8(cr, cg, cb, ca).with_alpha(ca as f32 / 255.0 * (1.0 - k));
            r.scale = scale;
            r.y = e.centre.y - r.record.height() * scale * 0.5;            if r.colours.first() != Some(&colour) {
                r.colours = vec![colour];
                r.redraw();
            }
        }
        if pulse.is_some() {
            let style = cm.as_ref().and_then(|cm| cm.menus.menus.get(&M_NAME)).and_then(|m| m.items.iter().find(|i| matches!(i.kind, ItemKind::Keyboard(_))))
                .and_then(|i| cm.as_ref().and_then(|cm| cm.menus.styles.get(&i.style)).cloned()).unwrap_or_default();
            let (lo, hi) = (style.disabled.unwrap_or([100, 100, 100, 200]), style.selected.unwrap_or([255; 4]));
            let phase = (t % KEY_PULSE) / KEY_PULSE;
            let k = if phase < 0.5 { phase * 2.0 } else { 2.0 - phase * 2.0 };
            let mix = |a: u8, b: u8| a as f32 + (b as f32 - a as f32) * k;
            let colour = Color::srgba_u8(mix(lo[0], hi[0]) as u8, mix(lo[1], hi[1]) as u8, mix(lo[2], hi[2]) as u8, mix(lo[3], hi[3]) as u8);
            if r.colours.first() != Some(&colour) {
                r.colours = vec![colour];
                r.redraw();
            }
        }
    }
    for (f, mut c) in &mut fills {
        let want = f.0.with_alpha(f.0.alpha() * fade);
        if c.0 != want {
            c.0 = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::menus::Key;

    /// ENTER NAME's keys from the data's layout: ten 40-unit slots a row, centred in the 420-unit
    /// item, five rows under the 38-unit field (take01: the first key at (105.3, 168), 36 wide).
    #[test]
    fn keys_sit_where_take01_has_them() {
        let kb = Keyboard { field: 38.0, slot: 40.0, longest: 12, across: 10, rows: 5,
                            keys: vec![Key { label: 0, slot: 0, width: 1 }, Key { label: 0, slot: 47, width: 3 }] };
        let boxes = key_boxes(&kb, Vec2::new(94.0, 124.0), Vec2::new(420.0, 248.0));
        assert!((boxes[0].0.x - 105.3).abs() < 1.5, "{:?}", boxes[0]);
        assert!((boxes[0].0.y - 168.0).abs() < 1.5, "{:?}", boxes[0]);
        assert_eq!(boxes[0].1, Vec2::new(36.0, 36.0));
        // DONE: 3 slots from the eighth of the last row (take01: 385-502 across)
        assert!((boxes[1].0.x - 385.3).abs() < 1.5 && (boxes[1].1.x - 116.0).abs() < 1.5, "{:?}", boxes[1]);
    }
}
