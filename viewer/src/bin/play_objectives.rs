//! The MISSION OBJECTIVES menu (#221; viewer/docs/campaign.md): the game's BACK button
//! (VB 62, OPEN_OBJECTIVES_MENU) opens it over the game in a mission, and the game's clock stops
//! under it (take04: open from 62.6 s to the B at 74.5 s, the clock held at 60275). It's the
//! game-options menu h_0cfeaf6f, 470 x 360 at (85, 60) on the 640 x 480 screen:
//!  * its picture h_0d0ce2e2 (game-options' h_1544c137): the bar h_f5392b60 at (0, 0) at its own
//!    size and alpha 180, the body h_ee1efd50 stretched to 470 x 296 at (0, 32), the bar again
//!    flipped upside down at (0, 328);
//!  * "MISSION OBJECTIVES" (h_08300dcd) centred in (2, 2) 246 x 24 (style h_e53b67a8);
//!    "Primary Objectives:" (h_e7ba5b50) at (8, 40) and "Secondary Objectives:" (h_e4ff251d) at
//!    (8, 182), 200 x 24 (h_0a959b2e); all 153 209 251 with a black outline;
//!  * the lists `primary-objectives` at (33, 76) 400 x 100 and `secondary-objectives` at
//!    (33, 214) 400 x 106 (h_ff54aaed: 30 110 150, outlined, centred up and down): each line
//!    FUN_000f8480 / FUN_000f8540 make, one per objective of the mission whose `enabled` flag is
//!    true (FUN_000f8310: its text, " (completed)" for state 1, " (failed)" for -1; play_flags.rs
//!    `Campaign::objective_line`), each followed by a "\n", word-wrapped to the box;
//!  * the legends "A Accept" (h_18291278) at (12, 332) and "B Back" (h_f1e06461) at (136, 332),
//!    110 x 20 (h_e35bb628: the button icon, then the word, 153 209 251).
//! It shows at once (take04: whole in the first frame after the press, 0.17 s) over the HUD,
//! which stays drawn.
//!
//! Controls: I (the demo's key for BACK) or the pad's Back opens it and closes it again; Enter /
//! Space or the pad's A or B close it. While it's up the keys and the mouse go to it: none of
//! their presses reaches the game (not the one that closes it, nor Backspace).
//!
//! Test hook: BF_TEST_OBJECTIVES=<s>[,<open s>[,<key>]]: opens it at that time (the game's clock),
//! for that long (2 s by default; counted in real time, or the capture's steps under BF_CAPTURE,
//! as the game's clock stands still meanwhile); with a key (space, enter or i) that key's press
//! closes it, put into the frame's input as the player's would be (the press must not reach the
//! game).

use super::flags::Campaign;
use super::text::{game_text, Align, GameFont, GameText};
use super::triggers::panels::{legend_icon, ICON12_ATLAS};
use super::*;

/// The menu's place and size on the 640 x 480 screen (game-options h_0cfeaf6f's `<data>`).
const MENU_AT: Vec2 = Vec2::new(85.0, 60.0);
/// Its picture's pieces (h_0d0ce2e2): the bar (512 x 32 at its own size, alpha 180) at the top
/// and, flipped upside down, at y 328; the body stretched to 470 x 296 at y 32, opaque.
const BAR: u32 = 0xF539_2B60;
const BODY: u32 = 0xEE1E_FD50;
const BAR_SIZE: Vec2 = Vec2::new(512.0, 32.0);
const BODY_AT: f32 = 32.0;
const BODY_SIZE: Vec2 = Vec2::new(470.0, 296.0);
const FOOT_AT: f32 = 328.0;
const BAR_ALPHA: f32 = 180.0 / 255.0;
/// The words (string ids): the title, the two headings, the legends.
const S_TITLE: u32 = 0x0830_0DCD;
const S_PRIMARY: u32 = 0xE7BA_5B50;
const S_SECONDARY: u32 = 0xE4FF_251D;
const S_ACCEPT: u32 = 0x1829_1278;
const S_BACK: u32 = 0xF1E0_6461;
/// The styles' colours: h_e53b67a8, h_0a959b2e and h_e35bb628's 153 209 251 (the title, the
/// headings, the legends); h_ff54aaed's 30 110 150 (the objective lines). Every one outlined
/// black (h_ed70ff4f), as `game_text` draws.
const LIGHT: Color = Color::srgb(153.0 / 255.0, 209.0 / 255.0, 251.0 / 255.0);
const TEAL: Color = Color::srgb(30.0 / 255.0, 110.0 / 255.0, 150.0 / 255.0);
/// The title's box (2, 2) 246 x 24, its words centred both ways (h_e53b67a8 alignment 2 2).
const TITLE_BOX: (Vec2, Vec2) = (Vec2::new(2.0, 2.0), Vec2::new(246.0, 24.0));
/// The headings' boxes, (8, 40) and (8, 182) (200 x 24), and the lists', (33, 76) 400 x 100 and
/// (33, 214) 400 x 106: primary, then secondary.
const HEADINGS: [Vec2; 2] = [Vec2::new(8.0, 40.0), Vec2::new(8.0, 182.0)];
const LISTS: [(Vec2, Vec2); 2] = [(Vec2::new(33.0, 76.0), Vec2::new(400.0, 100.0)), (Vec2::new(33.0, 214.0), Vec2::new(400.0, 106.0))];
/// The capitals' heights (units), measured on take04's frame at 63.0 s (1.5 px a unit): the
/// headings' P and S 13.3 (14 pt), the lines' I 13.3 and E 14.0 (14 pt: taken as the
/// headings'), the legends' B 12.0 (12 pt, from 396.0 to 408.0: the tutorial panels' legends',
/// the same style; 10.0 before #224 was a misreading). The title (16 pt): its glyphs are 21 px
/// tall in the take where the demo's font drew them 19 at 14.0, so 15.5 (a fit; the take spaces
/// its letters wider still: 183 units across to the demo's 176).
const TITLE_CAP: f32 = 15.5;
const TEXT_CAP: f32 = 13.3;
const LEGEND_CAP: f32 = 12.0;
/// Where a top-aligned line's capitals start below its box's top (take04: the headings' 105.3
/// and 247.3 in boxes from 100 and 242): a fit.
const CAP_BELOW: f32 = 5.3;
/// A list's line pitch: 24 units, the height the menu gives a line of 14 pt (its heading's box).
/// The list (its lines and the "\n" after the last, an empty line) is centred up and down in its
/// box: take04's single lines have their capitals at 166.7 and 308.0, the rule gives 167.3 and
/// 308.3. A fit on one line each; more lines aren't in the takes.
const LINE_PITCH: f32 = 24.0;
/// The words' first ink right of their box's left edge (take04: the headings' 95.3 and 94.7 in
/// boxes from 93, the lines' 121.3 and 120.7 in boxes from 118): measured.
const HEADING_INSET: f32 = 2.3;
const LINE_INSET: f32 = 3.3;
/// The legends (style h_e35bb628): the A item at (12, 332), the B item at (136, 332), each 110 x
/// 20 (so from 392 on the screen), with the button's letter in the icon font at 12 points: its
/// cell drawn as the tutorial panels' legends' (play_panels.rs `legend_icon`: take04's discs,
/// outlined and cut flat on top). The word as the panels' (play_panels.rs WORD_IN): its node 18.3
/// in, its capitals from 4.0 below the item's top (take04 at 63.0 s: "Accept" from 115.3, "Back"
/// from 240.0, both capitals 396.0-408.0). Measured.
const LEGENDS: [(f32, char, u32); 2] = [(12.0, 'A', S_ACCEPT), (136.0, 'B', S_BACK)];
/// The legends' item top (MENU_AT's 60 + 332).
const LEGEND_TOP: f32 = 392.0;
const WORD_IN: f32 = 18.3;
const WORD_CAP_TOP: f32 = LEGEND_TOP + 4.0;
/// How many rows each list has nodes for (the most a mission has is m01's four primaries, some
/// of which wrap).
const MAX_LINES: usize = 10;
/// The test hook's default time open (s).
const TEST_OPEN: f32 = 2.0;
/// Above the HUD (its nodes have no z-index of their own).
const MENU_Z: i32 = 20;

/// The menu's state.
#[derive(Resource, Default)]
pub struct ObjectivesMenu {
    /// it's open (the game's clock stopped)
    pub open: bool,
    /// how long it's been open, in real time (the capture's steps under BF_CAPTURE)
    held: f32,
    /// BF_TEST_OBJECTIVES: (when, for how long), and whether it's been done
    test: Option<(f32, f32)>,
    test_done: bool,
    /// BF_TEST_OBJECTIVES's closing key, pressed as the player would (None: it closes itself),
    /// and that key while it's held down (let go the frame after)
    test_key: Option<KeyCode>,
    test_key_held: Option<KeyCode>,
    /// closed from outside (#226: the level's end closes every menu, the menu system's +0x1c in
    /// FUN_000e0410): done by `toggle` in the next frame
    pub close: bool,
}

/// BF_TEST_OBJECTIVES's closing key by its name: space, enter or i.
fn test_key(name: &str) -> Option<KeyCode> {
    match name.trim().to_ascii_lowercase().as_str() {
        "space" => Some(KeyCode::Space),
        "enter" => Some(KeyCode::Enter),
        "i" => Some(KeyCode::KeyI),
        _ => None,
    }
}

/// The menu's root node (hidden while closed).
#[derive(Component)]
struct MenuRoot;

/// The menu's controls (`toggle`), for what must take a frame's presses before it: the tutorial's
/// panels over the game and the hint lines' virtual buttons (play_panels.rs, play_hints.rs).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ObjectivesControls;

/// An objective line's text node: which list (0 primary, 1 secondary) and which line.
#[derive(Component)]
struct ObjectiveLine(usize, usize);

/// The menu as a map starts: built hidden, the test hook read; its controls each frame as soon as
/// the frame's input is in (PreUpdate, after Bevy's input systems), so that what it takes reaches
/// no system of the game's (`toggle`); its texts in the frame.
pub fn plugin(app: &mut App) {
    app.init_resource::<ObjectivesMenu>()
        .add_systems(OnEnter(AppState::Playing), build.after(setup))
        .add_systems(OnExit(AppState::Playing), close_on_exit)
        .add_systems(PreUpdate, toggle.in_set(ObjectivesControls).after(bevy::input::InputSystem).run_if(in_state(AppState::Playing)))
        .add_systems(Update, fill.run_if(in_state(AppState::Playing)));
}

/// A game texture as a UI image (None if it isn't found).
fn texture(game: &mut bf_viewer::bf::character::Game, images: &mut Assets<Image>, name: u32) -> Option<(Handle<Image>, Vec<u8>, u32, u32)> {
    let (w, h, px) = game.texture_rgba(name)?;
    let image = images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, px.clone(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default()));
    Some((image, px, w, h))
}

/// Builds the menu, hidden (see the module notes). Game textures are stored bottom row first:
/// drawn with flip_y, and the data's flipped piece (h_06712921) without.
fn build(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, mut menu: ResMut<ObjectivesMenu>) {
    let hook: Vec<String> = std::env::var("BF_TEST_OBJECTIVES").map(|v| v.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default();
    let num = |i: usize| hook.get(i).and_then(|x| x.parse::<f32>().ok());
    *menu = ObjectivesMenu {
        test: num(0).map(|at| (at, num(1).unwrap_or(TEST_OPEN))),
        test_key: hook.get(2).and_then(|k| test_key(k)),
        ..Default::default()
    };
    let game = &mut game.0;
    let s = |id: u32| game.strings.get(&id).cloned().unwrap_or_default();
    let (title, primary, secondary) = (s(S_TITLE), s(S_PRIMARY), s(S_SECONDARY));
    let legends: Vec<String> = LEGENDS.iter().map(|&(_, _, w)| s(w)).collect();
    let window = commands.spawn((MenuRoot, Visibility::Hidden, GlobalZIndex(MENU_Z),
                                 Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center, ..default() })).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();
    let at = |p: Vec2, size: Vec2| hud::at(MENU_AT.x + p.x, MENU_AT.y + p.y, size.x, size.y);
    if let Some((bar, ..)) = texture(game, &mut images, BAR) {
        let tint = Color::srgba(1.0, 1.0, 1.0, BAR_ALPHA);
        commands.spawn((ChildOf(screen), at(Vec2::ZERO, BAR_SIZE), ImageNode { flip_y: true, ..ImageNode::new(bar.clone()) }.with_color(tint)));
        commands.spawn((ChildOf(screen), at(Vec2::new(0.0, FOOT_AT), BAR_SIZE), ImageNode::new(bar).with_color(tint)));
    }
    if let Some((body, ..)) = texture(game, &mut images, BODY) {
        commands.spawn((ChildOf(screen), at(Vec2::new(0.0, BODY_AT), BODY_SIZE), ImageNode { flip_y: true, ..ImageNode::new(body) }));
    }
    // (the texts' places are set as the menu opens, `place`, once the font is known)
    commands.spawn((ChildOf(screen), game_text(&title, MENU_AT.x + TITLE_BOX.0.x + TITLE_BOX.1.x / 2.0, 0.0, TITLE_CAP, LIGHT, Align::Centre), Heading(None)));
    for (k, text) in [primary, secondary].into_iter().enumerate() {
        commands.spawn((ChildOf(screen), game_text(&text, MENU_AT.x + HEADINGS[k].x + HEADING_INSET, 0.0, TEXT_CAP, LIGHT, Align::Left), Heading(Some(k))));
        for n in 0..MAX_LINES {
            commands.spawn((ChildOf(screen), game_text("", MENU_AT.x + LISTS[k].0.x + LINE_INSET, 0.0, TEXT_CAP, TEAL, Align::Left), ObjectiveLine(k, n)));
        }
    }
    // the legends: the button's icon (the 12-point icon font's cell), the word
    let icons = texture(game, &mut images, ICON12_ATLAS).map(|(image, ..)| image);
    for (k, &(x, letter, _)) in LEGENDS.iter().enumerate() {
        if let Some((node, image)) = icons.clone().and_then(|image| legend_icon(image, Vec2::new(MENU_AT.x + x, LEGEND_TOP), letter)) {
            commands.spawn((ChildOf(screen), node, image));
        }
        commands.spawn((ChildOf(screen), game_text(&legends[k], MENU_AT.x + x + WORD_IN, 0.0, LEGEND_CAP, LIGHT, Align::Left), Heading(Some(2 + k))));
    }
}

/// A fixed text's node: the title (None), a heading (Some(0) primary, Some(1) secondary) or a
/// legend's word (Some(2), Some(3)), placed by `place`.
#[derive(Component)]
struct Heading(Option<usize>);

/// The lists as they are now (FUN_000f8480 and FUN_000f8540: each enabled objective's line), each
/// word-wrapped to its box's width as the menu's text box does (the takes don't show a wrapped
/// one: with " (completed)" the tutorial's primary is wider than its 400): the rows of each.
fn rows(campaign: &Campaign, game: &bf_viewer::bf::character::Game, font: &GameFont) -> [Vec<String>; 2] {
    let mut out: [Vec<String>; 2] = [vec![], vec![]];
    for o in &campaign.objectives {
        let k = if o.primary { 0 } else { 1 };
        let Some(line) = campaign.objective_line(o, game) else { continue };
        let room = LISTS[k].1.x - LINE_INSET;
        let mut row = String::new();
        for word in line.split(' ') {
            let longer = if row.is_empty() { word.to_string() } else { format!("{row} {word}") };
            if !row.is_empty() && font.width(&longer, TEXT_CAP) > room {
                out[k].push(std::mem::take(&mut row));
                row = word.to_string();
            } else {
                row = longer;
            }
        }
        out[k].push(row);
    }
    out
}

/// The texts and their places, by where take04 has their capitals (`GameFont::cap_drop`: from a
/// capital's top to its line's): the title centred up and down in its box, the headings
/// CAP_BELOW under their boxes' tops, the legends' words at WORD_CAP_TOP, and each list's rows
/// (with the empty one after them) centred in its box at LINE_PITCH. Kept up to date while the
/// menu is closed too, so it's whole in the frame it opens (as take04's).
fn fill(campaign: Res<Campaign>, game: Res<GameData>, font: Option<Res<GameFont>>,
        mut headings: Query<(&Heading, &mut GameText), Without<ObjectiveLine>>, mut lines: Query<(&ObjectiveLine, &mut GameText), Without<Heading>>) {
    let Some(font) = font else { return };
    let rows = rows(&campaign, &game.0, &font);
    for (which, mut t) in headings.iter_mut() {
        let cap_top = match which.0 {
            None => MENU_AT.y + TITLE_BOX.0.y + (TITLE_BOX.1.y - TITLE_CAP) / 2.0,
            Some(k @ 0..=1) => MENU_AT.y + HEADINGS[k].y + CAP_BELOW,
            Some(_) => WORD_CAP_TOP,
        };
        let y = cap_top - font.cap_drop(t.cap);
        if t.y != y {
            t.y = y;
        }
    }
    for (l, mut t) in lines.iter_mut() {
        let (at, size) = LISTS[l.0];
        let block = (rows[l.0].len() + 1) as f32 * LINE_PITCH;
        let top = MENU_AT.y + at.y + (size.y - block) / 2.0;
        let y = top + l.1 as f32 * LINE_PITCH + CAP_BELOW - font.cap_drop(t.cap);
        let text = rows[l.0].get(l.1).cloned().unwrap_or_default();
        if t.text != text {
            t.text = text;
        }
        if t.y != y {
            t.y = y;
        }
    }
}

/// The menu's controls (see the module notes) and the test hook: open, it shows and stops the
/// game's clock; closed, the clock runs again. A menu over the game takes the controls: the keys
/// and mouse buttons pressed in a frame it's up (the one it opens in and the one it closes in
/// too) are taken from the frame's input, so no system of the game's sees them (Space or Enter
/// closing it doesn't make Tex jump or click a test tool's panel). A tutorial panel over the game
/// takes the frame's presses before this runs (play_panels.rs `input`, ordered by
/// `ObjectivesControls`), so BACK on one opens nothing (take14).
#[allow(clippy::too_many_arguments)]
fn toggle(mut keys: ResMut<ButtonInput<KeyCode>>, mut mouse: ResMut<ButtonInput<MouseButton>>, pads: Query<&Gamepad>, time: Res<Time>,
          player: Res<Player>, campaign: Res<Campaign>, game: Res<GameData>, font: Option<Res<GameFont>>, mut menu: ResMut<ObjectivesMenu>,
          mut root: Query<&mut Visibility, With<MenuRoot>>, editing: Option<Res<State<editor::EditorMode>>>) {
    // (the test hook's closing key, let go the frame after its press)
    if let Some(k) = menu.test_key_held.take() {
        keys.release(k);
    }
    let real = if std::env::var("BF_CAPTURE").is_ok() { capture_step() } else { time.delta_secs() };
    if menu.open {
        menu.held += real;
    }
    let test_open = !menu.test_done && menu.test.is_some_and(|(at, _)| player.sim_time >= at);
    let test_close = menu.open && menu.test_done && menu.test.is_some_and(|(_, open)| menu.held >= open);
    // the test hook's closing key, pressed now as the player's would be
    let mut closes_itself = test_close;
    if let (true, Some(k)) = (test_close, menu.test_key) {
        keys.press(k);
        menu.test_key_held = Some(k);
        closes_itself = false;
    }
    let pad = |b: GamepadButton| pads.iter().any(|p| p.just_pressed(b));
    let back = keys.just_pressed(KeyCode::KeyI) || pad(GamepadButton::Select);
    let close = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || pad(GamepadButton::South) || pad(GamepadButton::East);
    let playing = editing.is_none_or(|m| *m.get() != editor::EditorMode::Edit);
    let forced = std::mem::take(&mut menu.close);
    // (not while a cutscene has the world: FUN_0008c8c0 opens no menu while its +0xc50 bit 4 is
    // set, play_script.rs; the press is lost, as take03's START in the opening. Nor under another
    // menu that holds the game's clock: the item list and the Squad Command menu lock the player's
    // game input, BACK's binding among it (FUN_000ac080, FUN_000c11f0 -> FUN_001e7820), and a
    // tutorial panel takes the frame's presses; the pad's BACK closing the Squad Command menu
    // opens nothing, #319)
    let want = if menu.open { !(back || close || closes_itself || forced) } else {
        (back || test_open) && campaign.in_mission && playing && !script::in_cutscene() && !paused_by_other(PAUSE_OBJECTIVES)
    };
    if test_open {
        menu.test_done = true;
    }
    if menu.open || want {
        let pressed: Vec<KeyCode> = keys.get_just_pressed().copied().collect();
        for k in pressed {
            keys.clear_just_pressed(k);
        }
        let clicked: Vec<MouseButton> = mouse.get_just_pressed().copied().collect();
        for b in clicked {
            mouse.clear_just_pressed(b);
        }
    }
    if want == menu.open {
        return;
    }
    menu.open = want;
    menu.held = 0.0;
    set_paused(PAUSE_OBJECTIVES, want);
    if let Ok(mut v) = root.single_mut() {
        *v = if want { Visibility::Inherited } else { Visibility::Hidden };
    }
    if std::env::var("BF_FLAG_LOG").is_ok() {
        match (want, font) {
            (true, Some(f)) => {
                let rows = rows(&campaign, &game.0, &f);
                println!("t {:.2}: objectives menu: primary {:?}, secondary {:?}", player.sim_time, rows[0], rows[1]);
            }
            (false, _) => println!("t {:.2}: objectives menu closed{}", player.sim_time, if closes_itself { " (the test hook's time)" } else { " by a press" }),
            _ => {}
        }
    }
}

/// Leaving the map: the menu closed and its hold on the game's clock let go (the menu's nodes go
/// with the map's).
fn close_on_exit(mut menu: ResMut<ObjectivesMenu>) {
    menu.open = false;
    set_paused(PAUSE_OBJECTIVES, false);
}
