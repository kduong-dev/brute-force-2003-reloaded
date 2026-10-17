//! The end screen (#226; viewer/docs/mission.md): game-options' menu h_e8e14fcf, which the
//! world's end step opens once the level's end fade is done (play_mission.rs), drawn from its
//! data and filled as its controller fills it (vtable 0x39e1cc; FUN_001070b0 sets up each item by
//! its name, 0x107720 is its update, 0x1074c0 / 0x107800 / 0x107960 / 0x107010 its A, X, Y and B).
//! A child of play_mission.rs.
//!
//! What it shows (take12: MISSION SUCCESSFUL; take05 and take10: SQUAD KILLED):
//!  * the title, by its selector (FUN_00107e50): MISSION SUCCESSFUL when every enabled primary
//!    objective is complete (FUN_00107dd0: `state` > 0), else SQUAD KILLED when the squad died
//!    (FUN_00107a40: the campaign mode's bit 1), else MISSION FAILED (0x400dc4's three strings);
//!    the A legend's word with it (0x400d7c: Continue, Replay Mission, Replay Chapter for a
//!    mission of more than one zone, Clone Squad);
//!  * three pages, the menu's three button items h_194fde78, h_006044ef, h_008c2555 with their
//!    buddy-controls: the summary first, Y going round them (0x107960, with the scroll sound
//!    h_1dffc5a1; take12's four Y presses: objectives, cash, summary, objectives);
//!  * the summary: "Objectives n / n" (the enabled objectives, primary and secondary, complete
//!    over all: FUN_00107c70), "Cash $x / $y" (x what the squad earned: its penalties, plus its
//!    awards capped at y, FUN_00106860; y the zone's max-score, BRUTAL's, at this difficulty: 11760
//!    / 1.4 x 0.6 = 5040 on STANDARD, FUN_00106b50), "Operatives Lost n", "DNA Sequences
//!    Recovered" with "<name> DNA" or "No DNA Found"; the failed primaries (" (failed)") under
//!    "Objectives" when the mission failed;
//!  * the objectives page: each enabled objective's line (`^c0` its own colour when complete, `^c1`
//!    the red 251 100 100 when not: FUN_00106c80 gives the texts that second colour) and beside
//!    it its cash x the difficulty's scale ("+ $ 3000") or "Failed" (a primary not complete; a
//!    secondary failed) or "Incomplete";
//!  * the cash page: "Cash Awards" (the awards capped, FUN_00106be0) with the entries above 0
//!    ("24 x Red Hand" "+ $ 1440"), "Cash Penalties" with those below ("1 x Tex" "- $ 3000");
//!  * the numbers count up as it opens (FUN_001090e0 from the update): its +0xc clock runs to 10 s
//!    and each number shows clock / 10 x the greater of a step and its target, cut to the target
//!    (FUN_00107e90: 20 for the objectives and the operatives, y for the cash): take12's cash
//!    $50 at 358.42 s, $2040 by 362.4 s; take05's from $-96 to $-3000 by 190 s ("$-1526" in the
//!    footage notes is a frame of that count). Its +8 clock shows the awards' icons one by one
//!    every 0.5 s (only when the mission succeeded: the objectives' by the difficulty, h_ef0224c4 /
//!    h_12e4a006 / h_001bdb97 below / at / above HARD's 0; the cash's when all of y was earned; the
//!    DNA's h_08b1797b when a DNA was found). A or X while it counts finishes the count instead;
//!  * the texts in their styles' disabled colours (FUN_000ef3a0: a static text isn't enabled),
//!    each after its shadow one unit down and right (FUN_0010a420); the title's box filled with
//!    its style's h_f679aef5 (153 209 251 at 100); the legends A Continue, X Replay Mission, Y More
//!    Stats, B Quit.
//!
//! As it opens (0x106ef0): the cutscene bit set; on success the mission's results saved
//! (FUN_00106d10; the status line's "Saving...", "Saving...", "Saving Campaign...", 1 s each:
//! take12, nothing saved here); the view cut to the level's end camera, the letterbox in at once
//! and a 1 s fade from black (FUN_001092e0; take12: 358.42-359.40 s).
//!
//! Its buttons: A on Continue, Clone Squad (or Replay Mission / Chapter on a failure) goes on
//! (FUN_00085200, play_mission.rs `go`); X asks "Are you sure you want to restart the mission?",
//! B "Are you sure you want to quit?" in the message box h_0ec41ef2 (the menu system's +0x38,
//! centred at (184, 138), the end screen hidden under it: take10 at 209-226 s), A Accept going
//! on (FUN_00084da0, FUN_00084f50), B Back to the end screen. While it's up no press reaches the
//! game.
//!
//! Keys: A is Enter, Space or the pad's A; B is Escape, Backspace or the pad's B; X is R or the
//! pad's X; Y is Q or the pad's Y.
//!
//! Test hook: BF_END_PRESS=<a|b|x|y>@<s>[;...]: those buttons pressed that long after the end
//! screen opened (real time, or the capture's steps under BF_CAPTURE), in order; the message box's
//! A or B too.
//!
//! Not done: the count's ticking sound (h_1395a95e at 0.2 while a number counts) and the icons'
//! sound; the system-link variants (only the host may go on: h_180269c9 ...); the mission's results
//! written to the save (FUN_00106d10, the session's +0x1e0: score, max-score, the award bits).

use bevy::input::InputSystem;
use bf_viewer::bf::character::Game;
use bf_viewer::bf::hash::h;
use bf_viewer::bf::menus::{GameMenu, ItemKind, MenuItem, MenuStyle};

use super::super::fonts::{self, record_text, Across, Record, BODY12};
use super::super::triggers::panels::{legend_icon, ICON12_ATLAS};
use super::*;

/// The end screen's menu (the campaign mode's +0xc) and the confirmation box (the menu system's
/// +0x38 opens h_0ec41ef2: "Attention", a message, A Accept, B Back).
const END_MENU: u32 = 0xE8E1_4FCF;
const BOX_MENU: u32 = 0x0EC4_1EF2;
/// Where the menu system puts the box: centred on the 640 x 480 screen ((640 - 272) / 2, (480 -
/// 204) / 2); take10's box from x 192 (its picture's 8 in) and y 139.
const BOX_AT: Vec2 = Vec2::new(184.0, 138.0);
/// The box's texts: "Are you sure you want to quit?" (h_fa796b36, 0x400db0) after B and "Are you
/// sure you want to restart the mission?" (h_0c0ef012, 0x1078b3) after X, under "Attention"
/// (h_0cc05846).
const S_QUIT_ASK: u32 = 0xFA79_6B36;
const S_RESTART_ASK: u32 = 0x0C0E_F012;
const S_ATTENTION: u32 = 0x0CC0_5846;
/// The titles by selector (0x400dc4): MISSION SUCCESSFUL, SQUAD KILLED, MISSION FAILED.
const TITLES: [u32; 3] = [0xEF09_CA83, 0xFC20_40F1, 0xEDA4_A8DD];
/// The A legend's word by selector (0x400d7c): Continue, Replay Mission, Replay Chapter, Clone
/// Squad.
const A_WORDS: [u32; 4] = [0x167A_4774, 0xE2A7_E3B3, 0x19F8_8621, 0x0FAD_FC8C];
/// The strings the items' texts are made with: "%d / %d", "$%d / $%d", "%d", " DNA", "No DNA
/// Found", " (failed)", "Failed", "Incomplete", "+ $ %d", "- $ %d", "%d x %s".
const S_OF: u32 = 0x1F28_106B;
const S_CASH_OF: u32 = 0x1FD5_DB3D;
const S_COUNT: u32 = 0x0C28_8E26;
const S_DNA: u32 = 0x0FD5_0A6D;
const S_NO_DNA: u32 = 0x0ED8_605A;
const S_FAILED_SUFFIX: u32 = 0x07AE_48A2;
const S_FAILED: u32 = 0x0FEA_A729;
const S_INCOMPLETE: u32 = 0x0CEB_196B;
const S_PLUS: u32 = 0x1A06_DA6D;
const S_MINUS: u32 = 0x0BF7_9BEE;
const S_TIMES: u32 = 0x1612_1172;
/// The items FUN_001070b0 fills, by their item-data names (see the module notes).
const I_TITLE: u32 = 0xF3CD_882F;
const I_OBJECTIVES: u32 = 0xFE27_2709;
const I_FAILED: u32 = 0xFC4B_414A;
const I_CASH: u32 = 0xEB8D_434B;
const I_LOST: u32 = 0x0F07_0A22;
const I_DNA: u32 = 0x0605_6D29;
const I_PRIMARY: u32 = 0x153E_3B76;
const I_PRIMARY_STATUS: u32 = 0x1957_B14B;
const I_SECONDARY: u32 = 0xF110_1A50;
const I_SECONDARY_STATUS: u32 = 0x0FBE_1A97;
const I_AWARDS: u32 = 0xF219_35F5;
const I_PENALTIES: u32 = 0x0254_2B8F;
const I_AWARD_LIST: u32 = 0xFA6E_46F9;
const I_AWARD_AMOUNTS: u32 = 0x1689_514D;
const I_PENALTY_LIST: u32 = 0x040E_BAE8;
const I_PENALTY_AMOUNTS: u32 = 0x0847_4EEE;
/// The rows FUN_001081a0 moves down under the failed objectives' lines (the labels and values of
/// Cash, Operatives Lost and DNA): by (lines - 1) x 16 + 5.
const MOVED_BY_FAILED: [u32; 6] = [0xE589_6D33, I_CASH, 0xF878_A063, I_LOST, 0x0228_8451, I_DNA];
/// The icons, in the slots' order (FUN_00108f90, FUN_00108fd0, FUN_00109080): objectives, cash,
/// DNA; and the legends A, X, Y, B.
const I_ICON_OBJECTIVES: u32 = 0xFA12_5125;
const I_ICON_CASH: u32 = 0x14A0_8C29;
const I_ICON_DNA: u32 = 0xEF9A_D673;
const I_LEGEND_A: u32 = 0x10B8_A5CF;
/// The objectives icon by the difficulty against HARD's (0x400dd0: below, at, above), the cash
/// icon's (0x400ddc) and the DNA icon (0x1090b8, h_08b1797b).
const ICON_OBJECTIVES: [u32; 3] = [0xEF02_24C4, 0x12E4_A006, 0x001B_DB97];
const ICON_CASH: [u32; 3] = [0xE3FA_BE34, 0x1E1C_3AF6, 0x0CE3_4167];
const ICON_DNA: u32 = 0x08B1_797B;
/// HARD's difficulty, n (game-options' h_ffc1becb `normal`; FUN_00106b10 compares the session's
/// with the app's +0x54).
const NORMAL_DIFFICULTY: i64 = 0;
/// FUN_001090e0's clocks: the icons one every 0.5 s up to 2 s, the count to 10 s; the steps the
/// objectives and operatives counts take (FUN_00107f60, FUN_001082a0: 20.0).
const ICON_EVERY: f32 = 0.5;
const ICONS_FOR: f32 = 2.0;
const COUNT_FOR: f32 = 10.0;
const COUNT_STEP: i32 = 20;
/// The red second colour FUN_00106c80 gives a text (0xfffb6464).
const RED: Color = Color::srgb(251.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0);
/// The end screen over the fade (15) and the panels (20).
const MENU_Z: i32 = 25;
/// A legend's word after its button: the icon font's A-D glyphs' offset and advance (1 + 17, the
/// 12-point record 0x3c5fe8: play_panels.rs ICON12_GLYPHS), the pen going on into the word.
const LEGEND_WORD_IN: f32 = 18.0;

/// The end screen (see the module notes) while it's up.
#[derive(Resource, Default)]
pub struct EndMenu {
    open: bool,
    root: Option<Entity>,
    /// FUN_00107e50's selector: 0 success, 1 failed (Replay Mission), 2 failed (Replay Chapter), 3
    /// squad killed
    selector: usize,
    /// the page shown, and each page's items' nodes
    page: usize,
    pages: [Vec<Entity>; 3],
    /// its +8 and +0xc clocks
    icons_t: f32,
    count_t: f32,
    /// the icons shown so far, in order, and the counting texts with their targets and steps
    icons: Vec<Entity>,
    counts: Vec<Count>,
    /// the box up (its nodes and what it asks)
    confirm: Option<(Entity, Ask)>,
    /// how long it's been up (real time, the capture's steps) and BF_END_PRESS's presses left
    up: f32,
    presses: Vec<(Press, f32)>,
}

/// A number counting up (FUN_00107e90): its text node, the format ("%d / %d" ...), its target,
/// the step and the second number.
struct Count {
    node: Entity,
    format: String,
    target: i32,
    step: i32,
    of: Option<i32>,
}

/// What the box asks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ask {
    Quit,
    Replay,
}

/// The end screen's buttons.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Press {
    A,
    B,
    X,
    Y,
}

/// The end screen: built when the flow opens it, its buttons as soon as the frame's input is in
/// (before the game's systems see it), its clocks each frame.
pub fn plugin(app: &mut App) {
    app.init_resource::<EndMenu>()
        .add_systems(OnEnter(AppState::Playing), reset)
        .add_systems(PreUpdate, input.after(InputSystem).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (build, animate).chain().after(frame).run_if(in_state(AppState::Playing)));
}

/// A map starts: no end screen (its nodes went with the last map's).
fn reset(mut menu: ResMut<EndMenu>) {
    *menu = EndMenu::default();
}

/// FUN_00107e90: a count's value `t` s into the count (`COUNT_FOR`): t / 10 x the greater of the
/// step and the target (below 0: the lesser of -step and the target), cut toward 0 and kept to
/// the target.
fn counted(t: f32, target: i32, step: i32) -> i32 {
    let k = (t / COUNT_FOR).min(1.0);
    if target >= 0 {
        ((k * step.max(target) as f32) as i32).min(target)
    } else {
        ((k * (-step).min(target) as f32) as i32).max(target)
    }
}

/// A count's text: its format's first "%d" the value, its second (if it has one) `of`.
fn count_text(format: &str, value: i32, of: Option<i32>) -> String {
    let text = format.replacen("%d", &value.to_string(), 1);
    match of {
        Some(of) => text.replacen("%d", &of.to_string(), 1),
        None => text,
    }
}

/// A style's look for a static text: its record (`fonts::record_of`), its colour (disabled:
/// FUN_000ef3a0), its shadow (h_ed70ff4f) and its alignment.
fn look(style: &MenuStyle) -> (&'static Record, Color, Option<Color>, (i64, i64)) {
    let rgba = |c: [u8; 4]| Color::srgba_u8(c[0], c[1], c[2], c[3]);
    let colour = style.disabled.or(style.color).map_or(Color::WHITE, rgba);
    (fonts::record_of(style), colour, style.edge.map(rgba), style.align)
}

/// The lines of a static text item's string in its box: its "\n"s, each laid out to the box's width
/// as the game's static text does (FUN_0010a880, `briefing::lay_out_with`), by the record's
/// advances.
pub(crate) fn lines(text: &str, width: f32, record: &Record) -> Vec<String> {
    text.split('\n').flat_map(|line| {
        let chars: Vec<char> = line.chars().collect();
        let laid = briefing::lay_out_with(&chars, width, &|c| record.measure(&c.to_string()));
        laid.into_iter().map(|l| l.into_iter().collect::<String>().trim_end().to_string()).collect::<Vec<_>>()
    }).collect()
}

/// A static text item's nodes (one record text a line) on `screen`: placed in its box (its style's
/// alignment: across 0 left, 1 right, 2 centred; down 0 top, 2 centred), a line every (line + 1),
/// its box filled with the style's h_f679aef5; `dy` moves it down (FUN_001081a0). Returns the
/// nodes (the first is the fill when there is one). The front end's campaign screens draw theirs
/// so too (play_campaign_menus.rs).
#[allow(clippy::too_many_arguments)]
pub(crate) fn text_item(commands: &mut Commands, screen: Entity, menu_at: Vec2, item: &MenuItem, style: &MenuStyle, text: &str, colours: Option<Vec<Color>>,
             dy: f32) -> Vec<Entity> {
    let (record, colour, shadow, (across, down)) = look(style);
    let corner = menu_at + item.at + Vec2::new(0.0, dy);
    let mut out = vec![];
    if let Some(f) = style.fill {
        out.push(commands.spawn((hud::at(corner.x, corner.y, item.size.x, item.size.y), BackgroundColor(Color::srgba_u8(f[0], f[1], f[2], f[3])),
                                 ChildOf(screen))).id());
    }
    let lines = lines(text, item.size.x, record);
    let pitch = record.height();
    let top = match down {
        2 => corner.y + ((item.size.y - lines.len() as f32 * pitch) * 0.5).floor(),
        1 => corner.y + item.size.y - lines.len() as f32 * pitch,
        _ => corner.y,
    };
    let (x, how) = match across {
        1 => (corner.x + item.size.x, Across::Right),
        2 => (corner.x + item.size.x * 0.5, Across::Centre),
        _ => (corner.x, Across::Left),
    };
    let colours = colours.unwrap_or_else(|| vec![colour, RED]);
    for (n, line) in lines.iter().enumerate() {
        out.push(commands.spawn((record_text(line, x, top + n as f32 * pitch, record, colours.clone(), shadow, how), ChildOf(screen))).id());
    }
    out
}

/// A picture item's node (its own piece: the icons), or None.
fn picture(commands: &mut Commands, screen: Entity, game: &mut Game, images: &mut Assets<Image>, menu_at: Vec2, item: &MenuItem, texture: u32) -> Option<Entity> {
    let (w, hgt, px) = game.texture_rgba(texture)?;
    let image = images.add(Image::new(bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 },
                                      bevy::render::render_resource::TextureDimension::D2, px,
                                      bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default()));
    let at = menu_at + item.at;
    // (game pictures are stored bottom row first: drawn flipped)
    Some(commands.spawn((hud::at(at.x, at.y, item.size.x, item.size.y), ImageNode { flip_y: true, ..ImageNode::new(image) }, ChildOf(screen))).id())
}

/// A legend item's nodes: its button's glyph in the 12-point icon font (play_panels.rs
/// `legend_icon`) and its word after it in the 12-point body font, in its style's colour.
#[allow(clippy::too_many_arguments)]
fn legend(commands: &mut Commands, screen: Entity, icons: &Option<Handle<Image>>, game: &Game, menu_at: Vec2, item: &MenuItem, style: &MenuStyle,
          button: u32, word: &str) -> Vec<Entity> {
    let corner = menu_at + item.at;
    let mut out = vec![];
    let letter = game.strings.get(&button).and_then(|s| s.chars().next()).unwrap_or(' ');
    if let Some((node, image)) = icons.clone().and_then(|image| legend_icon(image, corner, letter)) {
        out.push(commands.spawn((node, image, ChildOf(screen))).id());
    }
    let rgba = |c: [u8; 4]| Color::srgba_u8(c[0], c[1], c[2], c[3]);
    let colour = style.color.map_or(Color::WHITE, rgba);
    out.push(commands.spawn((record_text(word, corner.x + LEGEND_WORD_IN, corner.y, &BODY12, vec![colour], style.edge.map(rgba), Across::Left),
                             ChildOf(screen))).id());
    out
}

/// A screen root over the game: the whole window with the 640 x 480 screen 4:3 at its height.
fn root(commands: &mut Commands, name: &str, z: i32) -> (Entity, Entity) {
    let root = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                      justify_content: JustifyContent::Center, ..default() }, GlobalZIndex(z), Name::new(name.to_string()))).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(root))).id();
    (root, screen)
}

/// What the end screen's texts say, worked out as FUN_001070b0's fillers do (see the module notes).
struct Numbers {
    /// the enabled objectives complete, and all of them
    done: i32,
    total: i32,
    /// the cash earned and the zone's max at this difficulty
    earned: i32,
    max: i32,
    /// the awards (capped) and the penalties (above 0)
    awards: i32,
    penalties: i32,
    lost: i32,
}

/// The figures (FUN_00107c70, FUN_00106860, FUN_00106b50, FUN_00106be0, FUN_00106c30, the session's
/// +0x1d4): from the campaign's flags and cash, and the zone's max-score at BRUTAL scaled to the
/// difficulty played.
fn numbers(campaign: &flags::Campaign, max_score: f32) -> Numbers {
    let (mut done, mut total) = (0, 0);
    for o in &campaign.objectives {
        let enabled = campaign.flags.value(&[o.path.as_slice(), &[h("enabled")]].concat()).is_some_and(|v| v.as_bool());
        if enabled {
            total += 1;
            if campaign.flags.value(&[o.path.as_slice(), &[h("state")]].concat()).is_some_and(|v| v.as_int() > 0) {
                done += 1;
            }
        }
    }
    let cash = &campaign.cash;
    // (FUN_00106b50: the max-score is BRUTAL's (FUN_00106660 sets its difficulty to 4): scaled
    // back to this difficulty's)
    let max = (campaign.cash_scale() * (max_score / (1.0 + 0.1 * 4.0))) as i32;
    let positive: f32 = cash.entries.iter().filter(|e| e.sum > 0.0).map(|e| e.sum).sum();
    let negative: f32 = cash.entries.iter().filter(|e| e.sum < 0.0).map(|e| e.sum).sum();
    let awards = (cash.awards + positive).min(max as f32);
    let penalties = cash.penalties + negative;
    Numbers { done, total, earned: (penalties + awards) as i32, max, awards: awards as i32, penalties: (-penalties) as i32, lost: cash.lost as i32 }
}

/// The end screen opens (0x106ef0 and FUN_001070b0, see the module notes): when the flow says so.
#[allow(clippy::too_many_arguments)]
fn build(mut commands: Commands, mode: Res<MissionMode>, mut menu: ResMut<EndMenu>, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>,
         campaign: Res<flags::Campaign>, map: Res<MapLevel>, mut cutscenes: ResMut<script::Cutscenes>, mut fade: ResMut<Fade>,
         mut console: ResMut<Console>) {
    if !mode.menu_open || menu.open {
        return;
    }
    let Some(data) = game.0.menus.menus.get(&END_MENU).cloned() else {
        eprintln!("end screen: game-options has no menu h_{END_MENU:08x}");
        menu.open = true;
        return;
    };
    let game = &mut game.0;
    let success = !mode.squad_dead && campaign.objectives.iter().filter(|o| o.primary).all(|o| {
        let p = |k: &str| [o.path.as_slice(), &[h(k)]].concat();
        !campaign.flags.value(&p("enabled")).is_some_and(|v| v.as_bool()) || campaign.flags.value(&p("state")).is_some_and(|v| v.as_int() > 0)
    });
    let zones = game.campaign.iter().find(|l| l.id == mode.mission).map_or(1, |l| l.zones.len());
    let selector = if success { 0 } else if mode.squad_dead { 3 } else if zones > 1 { 2 } else { 1 };
    let n = numbers(&campaign, map.0.as_ref().map_or(0.0, |l| l.max_score));
    let scale = campaign.cash_scale();
    let s = |game: &Game, id: u32| string(game, id);
    // the failed primaries (FUN_00108020: a failure that isn't the squad's death)
    let failed: Vec<String> = if selector == 1 || selector == 2 {
        campaign.objectives.iter().filter(|o| o.primary && campaign.flags.value(&[o.path.as_slice(), &[h("state")]].concat()).is_some_and(|v| v.as_int() < 0))
            .map(|o| format!("{}{}", s(game, o.text), s(game, S_FAILED_SUFFIX))).collect()
    } else {
        vec![]
    };
    let moved = if failed.is_empty() { 0.0 } else { (failed.len() as f32 - 1.0) * 16.0 + 5.0 };
    // the objectives' lists and their status column (FUN_00108430 .. FUN_00108950)
    let list = |primary: bool| -> (String, String) {
        let (mut names, mut status) = (vec![], vec![]);
        for o in campaign.objectives.iter().filter(|o| o.primary == primary) {
            let p = |k: &str| [o.path.as_slice(), &[h(k)]].concat();
            if !campaign.flags.value(&p("enabled")).is_some_and(|v| v.as_bool()) {
                continue;
            }
            let state = campaign.flags.value(&p("state")).map_or(0, |v| v.as_int());
            let complete = state > 0;
            names.push(format!("{}{}", if complete { "^c0" } else { "^c1" }, s(game, o.text)));
            status.push(if complete {
                format!("^c0{}", s(game, S_PLUS).replacen("%d", &((o.cash as f32 * scale) as i32).to_string(), 1))
            } else if primary || state < 0 {
                format!("^c1{}", s(game, S_FAILED))
            } else {
                format!("^c0{}", s(game, S_INCOMPLETE))
            });
        }
        (names.join("\n"), status.join("\n"))
    };
    let (primary_names, primary_status) = list(true);
    let (secondary_names, secondary_status) = list(false);
    // the cash entries (FUN_00108c60, FUN_00108de0): above 0 the awards, below the penalties
    let label = |game: &Game, key: u32| game.strings.get(&key).cloned().unwrap_or_else(|| format!("h_{key:08x}"));
    let entries = |above: bool| -> (String, String) {
        let picked: Vec<_> = campaign.cash.entries.iter().filter(|e| if above { e.sum > 0.0 } else { e.sum < 0.0 }).collect();
        (picked.iter().map(|e| s(game, S_TIMES).replacen("%d", &e.count.to_string(), 1).replacen("%s", &label(game, e.key), 1)).collect::<Vec<_>>().join("\n"),
         picked.iter().map(|e| if above { format!("^c0{}", s(game, S_PLUS).replacen("%d", &(e.sum as i32).to_string(), 1)) } else {
             format!("^c1{}", s(game, S_MINUS).replacen("%d", &((-e.sum) as i32).to_string(), 1))
         }).collect::<Vec<_>>().join("\n"))
    };
    let (award_names, award_amounts) = entries(true);
    let (penalty_names, penalty_amounts) = entries(false);
    // the icons (success only), in slot order
    let difficulty = campaign.difficulty;
    let variant = if difficulty < NORMAL_DIFFICULTY { 0 } else if difficulty > NORMAL_DIFFICULTY { 2 } else { 1 };
    let dna: Option<u32> = None;
    let icon = |name: u32| -> Option<u32> {
        if !success {
            return None;
        }
        match name {
            I_ICON_OBJECTIVES => Some(ICON_OBJECTIVES[variant]),
            I_ICON_CASH => (n.earned >= n.max).then_some(ICON_CASH[variant]),
            I_ICON_DNA => dna.map(|_| ICON_DNA),
            _ => None,
        }
    };

    let (root, screen) = root(&mut commands, "end screen", MENU_Z);
    let icons_atlas = game.texture_rgba(ICON12_ATLAS).map(|(w, hgt, px)| images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 }, bevy::render::render_resource::TextureDimension::D2, px,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())));
    // which page each item is on (the three buttons' buddy-controls; the rest on every page)
    let pages: Vec<Vec<u32>> = data.items.iter().filter_map(|i| match &i.kind { ItemKind::Button { buddies, .. } => Some(buddies.clone()), _ => None }).collect();
    let page_of = |name: u32| pages.iter().position(|p| p.contains(&name));
    let mut out = EndMenu { open: true, root: Some(root), selector, ..default() };
    for item in &data.items {
        let style = game.menus.styles.get(&item.style).cloned().unwrap_or_default();
        let dy = if MOVED_BY_FAILED.contains(&item.name) { moved } else { 0.0 };
        let nodes: Vec<Entity> = match &item.kind {
            ItemKind::Text(id) => {
                let count = |format: &str, target: i32, step: i32, of: Option<i32>| (format.to_string(), target, step, of);
                let (text, counting) = match item.name {
                    I_TITLE => (s(game, TITLES[title_of(selector)]), None),
                    I_OBJECTIVES => (String::new(), Some(count(&s(game, S_OF), n.done, COUNT_STEP, Some(n.total)))),
                    I_CASH => (String::new(), Some(count(&s(game, S_CASH_OF), n.earned.min(n.max), n.max, Some(n.max)))),
                    I_LOST => (String::new(), Some(count(&s(game, S_COUNT), n.lost, COUNT_STEP, None))),
                    I_DNA => (dna.map_or_else(|| s(game, S_NO_DNA), |d| format!("{}{}", label(game, d), s(game, S_DNA))), None),
                    I_FAILED => (failed.join("\n"), None),
                    I_PRIMARY => (primary_names.clone(), None),
                    I_PRIMARY_STATUS => (primary_status.clone(), None),
                    I_SECONDARY => (secondary_names.clone(), None),
                    I_SECONDARY_STATUS => (secondary_status.clone(), None),
                    I_AWARDS => (s(game, S_PLUS).replacen("%d", &n.awards.to_string(), 1), None),
                    I_PENALTIES => (s(game, S_MINUS).replacen("%d", &n.penalties.to_string(), 1), None),
                    I_AWARD_LIST => (award_names.clone(), None),
                    I_AWARD_AMOUNTS => (award_amounts.clone(), None),
                    I_PENALTY_LIST => (penalty_names.clone(), None),
                    I_PENALTY_AMOUNTS => (penalty_amounts.clone(), None),
                    _ => (s(game, *id), None),
                };
                // (a count starts at its value at 0 s: one line to write its numbers into)
                let text = counting.as_ref().map_or(text, |(format, target, step, of)| count_text(format, counted(0.0, *target, *step), *of));
                let nodes = text_item(&mut commands, screen, data.at, item, &style, &text, None, dy);
                if let (Some((format, target, step, of)), Some(&node)) = (counting, nodes.last()) {
                    out.counts.push(Count { node, format, target, step, of });
                }
                nodes
            }
            ItemKind::Picture { piece: Some(_), .. } => match icon(item.name).and_then(|t| picture(&mut commands, screen, game, &mut images, data.at, item, t)) {
                Some(e) => {
                    commands.entity(e).insert(Visibility::Hidden);
                    out.icons.push(e);
                    vec![e]
                }
                None => vec![],
            },
            ItemKind::Legend { button, label: word } => {
                let word = if item.name == I_LEGEND_A { s(game, A_WORDS[selector]) } else { s(game, *word) };
                legend(&mut commands, screen, &icons_atlas, game, data.at, item, &style, *button, &word)
            }
            _ => vec![],
        };
        if let Some(p) = page_of(item.name) {
            out.pages[p].extend(nodes);
        }
    }
    for p in 1..3 {
        for &e in &out.pages[p] {
            commands.entity(e).insert(Visibility::Hidden);
        }
    }
    out.presses = read_presses();
    *menu = out;
    // 0x106ef0: the cutscene bit; on success the results saved (the status line: take12's three
    // lines); FUN_001092e0: the end camera, the letterbox in at once, a 1 s fade from black
    script::set_cutscene(true);
    if success {
        for (text, secs) in [(S_SAVING, 1.0), (S_SAVING, 1.0), (S_SAVING_CAMPAIGN, 1.0)] {
            console.say(&s(game, text), secs);
        }
    }
    cutscenes.end_view(mode.end_camera);
    fade.start(1.0, false);
    if mode.log {
        println!("t {:.2}: end screen: {} (selector {selector}): objectives {} / {}, cash ${} / ${} (awards {}, penalties {}), operatives lost {}",
                 mode.clock, s(game, TITLES[title_of(selector)]), n.done, n.total, n.earned, n.max,
                 n.awards, n.penalties, n.lost);
    }
}

/// The title of a selector (0x1073c0): MISSION SUCCESSFUL for 0, SQUAD KILLED for 3, MISSION FAILED
/// for 1 and 2.
fn title_of(selector: usize) -> usize {
    match selector {
        0 => 0,
        3 => 1,
        _ => 2,
    }
}

/// BF_END_PRESS's presses (see the module notes).
fn read_presses() -> Vec<(Press, f32)> {
    let Ok(v) = std::env::var("BF_END_PRESS") else { return vec![] };
    v.split([';', ',']).filter_map(|e| {
        let (b, at) = e.trim().split_once('@')?;
        let b = match b.trim().to_ascii_lowercase().as_str() { "a" => Press::A, "b" => Press::B, "x" => Press::X, "y" => Press::Y, _ => return None };
        Some((b, at.trim().parse().ok()?))
    }).collect()
}

/// The end screen's clocks (0x107720 -> FUN_001090e0): the icons one every 0.5 s, the numbers
/// counting for 10 s (see the module notes).
fn animate(time: Res<Time<Real>>, mut menu: ResMut<EndMenu>, mut texts: Query<&mut fonts::RecordText>, mut vis: Query<&mut Visibility>) {
    if !menu.open {
        return;
    }
    let dt = if std::env::var("BF_CAPTURE").is_ok() { capture_step() } else { time.delta_secs().min(0.1) };
    menu.up += dt;
    if menu.icons_t < ICONS_FOR {
        menu.icons_t += dt;
    }
    if menu.count_t < COUNT_FOR {
        menu.count_t += dt;
    }
    let shown = ((menu.icons_t / ICON_EVERY) as usize).min(menu.icons.len());
    for (k, &e) in menu.icons.iter().enumerate() {
        // (an icon on another page than the summary stays hidden: they're all on it)
        if k < shown && menu.page == 0 {
            if let Ok(mut v) = vis.get_mut(e) {
                if *v != Visibility::Inherited {
                    *v = Visibility::Inherited;
                }
            }
        }
    }
    let t = menu.count_t;
    for c in &menu.counts {
        let text = count_text(&c.format, counted(t, c.target, c.step), c.of);
        if let Ok(mut r) = texts.get_mut(c.node) {
            if r.text != text {
                r.text = text;
            }
        }
    }
}

/// The end screen's and the box's buttons (see the module notes), as soon as the frame's input
/// is in: the frame's presses are theirs.
#[allow(clippy::too_many_arguments)]
fn input(mut commands: Commands, mut menu: ResMut<EndMenu>, mut keys: ResMut<ButtonInput<KeyCode>>, mut mouse: ResMut<ButtonInput<MouseButton>>,
         mut pads: Query<&mut Gamepad>, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, mode: Res<MissionMode>, current: Res<CurrentMap>,
         mut next: ResMut<NextState<AppState>>, mut vis: Query<&mut Visibility>) {
    if !menu.open || menu.root.is_none() {
        return;
    }
    let pad = |b: GamepadButton| pads.iter().any(|g| g.just_pressed(b));
    let mut press = if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) || pad(GamepadButton::South) {
        Some(Press::A)
    } else if keys.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) || pad(GamepadButton::East) {
        Some(Press::B)
    } else if keys.just_pressed(KeyCode::KeyR) || pad(GamepadButton::West) {
        Some(Press::X)
    } else if keys.just_pressed(KeyCode::KeyQ) || pad(GamepadButton::North) {
        Some(Press::Y)
    } else {
        None
    };
    // BF_END_PRESS: the next one due
    if press.is_none() && menu.presses.first().is_some_and(|p| menu.up >= p.1) {
        press = Some(menu.presses.remove(0).0);
    }
    // (the frame's presses are the menu's)
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
    let Some(press) = press else { return };
    let log = mode.log;
    // the box: A goes on, B back to the end screen
    if let Some((node, ask)) = menu.confirm {
        match press {
            Press::A => {
                if log {
                    println!("end screen: {ask:?} accepted");
                }
                menu.open = false;
                let to = if ask == Ask::Quit { Go::Quit } else { Go::ReplayMission };
                go(&mut commands, &mut next, &game.0, &mode, &current.0, to);
            }
            Press::B => {
                commands.entity(node).despawn();
                menu.confirm = None;
                if let Some(Ok(mut v)) = menu.root.map(|r| vis.get_mut(r)) {
                    *v = Visibility::Inherited;
                }
            }
            _ => {}
        }
        return;
    }
    // A and X finish a count still running first (FUN_001091f0 then FUN_001090e0)
    let counting = menu.icons_t < ICONS_FOR || menu.count_t < COUNT_FOR;
    match press {
        Press::A | Press::X if counting => {
            menu.icons_t = ICONS_FOR;
            menu.count_t = COUNT_FOR;
            if log {
                println!("end screen: {press:?} finishes the count");
            }
        }
        Press::A => {
            let to = match menu.selector { 0 => Go::Continue, 3 => Go::CloneSquad, 2 => Go::ReplayChapter, _ => Go::ReplayMission };
            if log {
                println!("end screen: A: {to:?}");
            }
            menu.open = false;
            go(&mut commands, &mut next, &game.0, &mode, &current.0, to);
        }
        Press::Y => {
            // 0x107960: the next page
            let (from, to) = (menu.page, (menu.page + 1) % 3);
            for (p, want) in [(from, Visibility::Hidden), (to, Visibility::Inherited)] {
                for &e in &menu.pages[p] {
                    if let Ok(mut v) = vis.get_mut(e) {
                        *v = want;
                    }
                }
            }
            // (the icons are on the summary: shown again with it only once they've come)
            if to == 0 {
                let shown = ((menu.icons_t / ICON_EVERY) as usize).min(menu.icons.len());
                for (k, &e) in menu.icons.iter().enumerate() {
                    if let Ok(mut v) = vis.get_mut(e) {
                        *v = if k < shown { Visibility::Inherited } else { Visibility::Hidden };
                    }
                }
            }
            menu.page = to;
            if log {
                println!("end screen: Y: page {}", to + 1);
            }
        }
        Press::X | Press::B => {
            let ask = if press == Press::B { Ask::Quit } else { Ask::Replay };
            if let Some(node) = confirm_box(&mut commands, &mut game.0, &mut images, ask) {
                if let Some(Ok(mut v)) = menu.root.map(|r| vis.get_mut(r)) {
                    *v = Visibility::Hidden;
                }
                menu.confirm = Some((node, ask));
                if log {
                    println!("end screen: {press:?}: the box asks {ask:?}");
                }
            }
        }
    }
}

/// The message box (h_0ec41ef2 at BOX_AT): its frame (the picture set h_e78d23b5), "Attention",
/// the question in its message's style, A Accept and B Back.
fn confirm_box(commands: &mut Commands, game: &mut Game, images: &mut Assets<Image>, ask: Ask) -> Option<Entity> {
    let data: GameMenu = game.menus.menus.get(&BOX_MENU).cloned()?;
    let (root, screen) = root(commands, "end screen box", MENU_Z + 1);
    let icons_atlas = game.texture_rgba(ICON12_ATLAS).map(|(w, hgt, px)| images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 }, bevy::render::render_resource::TextureDimension::D2, px,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())));
    let question = string(game, if ask == Ask::Quit { S_QUIT_ASK } else { S_RESTART_ASK });
    // the pictures first, then the texts and legends
    for item in &data.items {
        let ItemKind::Picture { set, .. } = &item.kind else { continue };
        for pc in game.menus.pictures.get(set).cloned().unwrap_or_default() {
            let Some((w, hgt, px)) = game.texture_rgba(pc.texture) else { continue };
            let image = images.add(Image::new(bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 },
                                              bevy::render::render_resource::TextureDimension::D2, px,
                                              bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default()));
            let at = BOX_AT + item.at + pc.at;
            let size = if pc.stretched { pc.size } else { Vec2::new(w as f32, hgt as f32) };
            let tint = Color::srgba_u8(pc.color[0], pc.color[1], pc.color[2], pc.color[3]);
            commands.spawn((hud::at(at.x, at.y, size.x, size.y), ImageNode { flip_y: !pc.flipped, ..ImageNode::new(image) }.with_color(tint), ChildOf(screen)));
        }
    }
    for item in &data.items {
        let style = game.menus.styles.get(&item.style).cloned().unwrap_or_default();
        match &item.kind {
            ItemKind::Text(_) => {
                // "Title" has no style: the body font at 16 points in the menus' light blue; the
                // question in its style's normal colour, not a static text's disabled grey (take10
                // at 215 s: 153 209 251; the code sets its text: read from the footage)
                let (text, style, colours) = if item.name == h("Title") {
                    (string(game, S_ATTENTION), MenuStyle { font: Some((0, 16.0)), color: Some([153, 209, 251, 255]), edge: Some([0, 0, 0, 255]),
                                                                  ..default() }, None)
                } else {
                    let normal = style.color.map(|c| vec![Color::srgba_u8(c[0], c[1], c[2], c[3])]);
                    (question.clone(), style, normal)
                };
                text_item(commands, screen, BOX_AT, item, &style, &text, colours, 0.0);
            }
            ItemKind::Legend { button, label } => {
                let word = string(game, *label);
                legend(commands, screen, &icons_atlas, game, BOX_AT, item, &style, *button, &word);
            }
            _ => {}
        }
    }
    Some(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// take12's cash counting from 0 to 2040 of 5040 (504 a second) and take05's down to -3000.
    #[test]
    fn counts_as_the_takes() {
        assert_eq!(counted(0.1, 2040, 5040), 50);
        assert_eq!(counted(4.0, 2040, 5040), 2016);
        assert_eq!(counted(5.0, 2040, 5040), 2040);
        assert_eq!(counted(0.2, -3000, 5040), -100);
        assert_eq!(counted(10.0, -3000, 5040), -3000);
        // the objectives: 20 a 10 s, so 2 by 1 s
        assert_eq!(counted(0.6, 2, 20), 1);
        assert_eq!(counted(1.0, 2, 20), 2);
    }
}
