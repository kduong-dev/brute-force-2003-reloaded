//! The mission title card (#248; viewer/docs/cutscenes.md): the mission's title and chapter in
//! the top right of the screen as a level opens, "Brute Force / Welcome to the 23rd!" in the
//! tutorial, "Rescue Brutus / Chapter 1" in the first zone of mission 1, "Ripe for Plunder" on e34.
//!
//! It is the HUD's own widget (name 0x1ac66450, FUN_000b6510), given the level's `credits` block
//! at every level load: the end of the load (`end_load`, FUN_000d6520) calls FUN_000b8b40, which
//! passes the block and the zone's chapter number to FUN_000b6730 (code-map.md). It is on only
//! with game-options' HUD `<main h_f6ede62d="true">` (always, in the shipped file), in a mission
//! (a multiplayer level has `dm-data`, not a `credits` block) and with a `Title` string.
//!
//! What it shows (FUN_000b6730):
//!  * line 1, the credits block's `Title`, in the body font at 16 points (play_fonts.rs
//!    `BODY16`, record 0x3c4ac8), split at '\n' into lines of one record line height (25 units)
//!    each: the tutorial's is the three lines "Brute Force", "", "Welcome to the 23rd!";
//!  * line 2, "Chapter n" at 12 points (`BODY12`), 26 units under the first line's top; n is the
//!    zone's 1-based index in its mission when it has two or more zones
//!    (`Mission::chapter_of`); a one-zone mission has no line 2;
//!  * each of the two widgets right-aligned to the safe area's right edge (640 less
//!    `Game::hud_safe`'s 48 = 592) by its own width, which for the multi-line title is its
//!    widest line, the lines left-aligned to one another (take03: "Brute Force" starts where
//!    "Welcome to the 23rd!" does; take12: "Chapter 1" ends 2 px short of "Rescue Brutus"). From
//!    y 90 (game-options' `title-y-offset`, h_09c6b3f8); in the HUD's colour (192, 215, 255)
//!    with the one-unit shadow (0a 0a 0a at half the text's alpha);
//!  * its clock: 16 s = 2 x 6 + 4 from the level's start (game-options' `<h_0edb8b91>`: 6, 10,
//!    1, 90, 55), the alpha a triangle in the time left t: clamp((6 - |t - 6|) x 1, 0, 1). So 4 s
//!    of nothing, a 1 s linear fade in, 10 s full and a 1 s fade out (take03 5.8-6.8 s and
//!    17.0-18.1 s on a clock that started at 1.8 s; take12 583.4-584.5 s and 594.9-596.0 s from a
//!    start at 579.4; #196 take27's fade out 1.0 s).
//!
//! It stays at screen y 90 under the cutscene letterbox (the game's widget sits at 90 less the
//! bars' 70 lines, in a viewport the bars push down by as many; with the bars full the takes
//! have it one unit higher, `LETTERBOX_NUDGE`) and isn't hidden with the HUD
//! (its parent is [hud+0x135c], which FUN_000bb170 exempts from the letterbox's alpha): so it
//! is a node of its own, not a `HudRoot`. The credits roll after the title never shows
//! (FUN_000b6730 sets its group index to INT_MAX), so it isn't made.
//!
//! The 16-point glyphs are the game's own record, drawn as every other record text is
//! (play_fonts.rs, FUN_000ccd20).
//!
//! Test hooks:
//!  * BF_TITLE_LOG=1: at the level's start the card's lines, place and the mission it came from
//!    (or why there is none); then the clock at each change of its phase (fade in, full, fade
//!    out, gone).
//!  * BF_TITLE_AT=<s>: start the card's clock at s (4.0 skips the four seconds of nothing; 10
//!    shows the fade out's start at 16): for captures of a point of the card.

use std::collections::HashMap;

use super::fonts::{record_text, Across, Record, RecordText, BODY12, BODY16};
use super::*;

/// The card's whole time on screen (s): the countdown FUN_000b6730 sets, +0x78 = 2 x T + 4, T
/// being game-options' h_1a4bd85f (6).
const SPAN: f32 = 16.0;
/// The half-span T of the alpha's triangle (s): game-options' `<h_0edb8b91 h_1a4bd85f="6">`, the
/// widget's +0x80.
const HALF: f32 = 6.0;
/// The alpha's slope, 1 per second (game-options' h_0b44ad21 = 1.0, the widget's +0x88): the
/// fades take 1 s.
const SLOPE: f32 = 1.0;
/// The widget's y (units): game-options' `title-y-offset` (h_09c6b3f8 = 90), the title line's top.
const TOP: f32 = 90.0;
/// How far under the title's top the chapter line's top is: FUN_000b6730 puts it at y 26.
const CHAPTER_DOWN: f32 = 26.0;
/// The text's colour: 0xffffd7c0 at [0x4a5ca0], c0 d7 ff (FUN_000bb600): (192, 215, 255).
const COLOUR: [u8; 3] = [192, 215, 255];
/// The shadow's colour (0a 0a 0a) and its share of the text's alpha (half), drawn one unit down
/// and right (FUN_000c99d0; todo/248 leads §3). `RecordText` draws the unit offset.
const SHADOW: [f32; 4] = [10.0 / 255.0, 10.0 / 255.0, 10.0 / 255.0, 0.5];
/// The chapter lines' strings, Chapter 1 to 6 (FUN_000b6730's table at 0xb68e0, string-table-en).
const CHAPTERS: [u32; 6] = [0x1B03_A2F8, 0x000A_F342, 0x090D_C3D4, 0xED69_5677, 0xE46E_66E1, 0xFF67_375B];
/// How far the card rises (units) while the letterbox is full: its capitals' tops are at 94.0-94.7
/// units in the three takes, all under full bars, against 95.7-96 for a pen at y 90 drawn as the
/// demo draws a record text (take03 142 px, the demo 144 px of a 1.5 px unit: 1.3 units at 16
/// points, 0.7 at 12). The explanation, not read from the code: the widget's y is 90 less
/// trunc(bars x 70) in a viewport whose top is the truncated 0.145833 x 480 = 69.99984 = 69
/// (FUN_00073110), so a full letterbox leaves it a unit short of 90 (and a half-open one
/// whenever the two truncations differ). A fit of one unit with that reading, not measured out
/// of the bars.
const LETTERBOX_NUDGE: f32 = 1.0;
/// Where the card draws in the order: over the radar (3), under the letterbox (10), the panels
/// (20) and the menus. (Not a guess about the game; the demo's layering.)
const CARD_Z: i32 = 4;

/// Where the card is in its time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    /// the first four seconds: the alpha is 0
    Waiting,
    /// the one second fade in
    FadeIn,
    /// ten seconds at full
    Full,
    /// the one second fade out
    FadeOut,
    /// over (the widget hides at the countdown's end)
    Gone,
}

/// The level's card: its clock and what it shows.
#[derive(Resource, Default)]
struct Card {
    /// the lines' text nodes (the title's lines, then the chapter) and each one's y under the
    /// widget's top: each frame sets their alpha and place
    lines: Vec<(Entity, f32)>,
    /// seconds of the game's clock since the card was set (the widget's countdown run from
    /// SPAN); None when the level has no card
    elapsed: Option<f32>,
    /// the phase last logged
    phase: Option<Phase>,
    /// BF_TITLE_LOG is set
    log: bool,
}

/// One of the card's two text widgets (FUN_000b65a0's +0x64 and +0x68): its text (the title's own
/// '\n's make its lines), its font record and its y under the card's top.
struct Widget {
    text: String,
    record: &'static Record,
    down: f32,
}

/// The card's systems: set at a level's start, stepped each frame.
pub fn plugin(app: &mut App) {
    app.init_resource::<Card>()
        .add_systems(OnEnter(AppState::Playing), begin.after(flags::begin_level))
        .add_systems(Update, step.run_if(in_state(AppState::Playing)));
}

/// The card's alpha with `elapsed` seconds gone (FUN_000b69a0's update): the triangle in the time
/// left t = SPAN - elapsed, clamp((HALF - |t - HALF|) x SLOPE, 0, 1); nothing once the countdown
/// is spent.
fn alpha_at(elapsed: f32) -> f32 {
    let left = SPAN - elapsed;
    if left <= 0.0 {
        return 0.0;
    }
    ((HALF - (left - HALF).abs()) * SLOPE).clamp(0.0, 1.0)
}

/// The phase `alpha_at` is in at `elapsed` seconds.
fn phase_at(elapsed: f32) -> Phase {
    let left = SPAN - elapsed;
    match alpha_at(elapsed) {
        _ if left <= 0.0 => Phase::Gone,
        a if a >= 1.0 => Phase::Full,
        _ if left > HALF => if elapsed <= SPAN - 2.0 * HALF { Phase::Waiting } else { Phase::FadeIn },
        _ => Phase::FadeOut,
    }
}

/// The card's widgets for the mission `title` (a string id) and `chapter`: the title at 16
/// points, then "Chapter n" at 12, 26 units under it (FUN_000b6730). None when the title is
/// missing: the card shows nothing without one (+0x9c).
fn widgets_of(strings: &HashMap<u32, String>, title: u32, chapter: Option<usize>) -> Option<Vec<Widget>> {
    let text = strings.get(&title).filter(|_| title != 0)?;
    let mut widgets = vec![Widget { text: text.replace('\r', ""), record: &BODY16, down: 0.0 }];
    if let Some(n) = chapter.filter(|n| (1..=CHAPTERS.len()).contains(n)) {
        let name = strings.get(&CHAPTERS[n - 1]).cloned().unwrap_or_else(|| format!("Chapter {n}"));
        widgets.push(Widget { text: name, record: &BODY12, down: CHAPTER_DOWN });
    }
    Some(widgets)
}

/// The card's top (units): game-options' `title-y-offset`, a unit less under a full letterbox
/// (`LETTERBOX_NUDGE`).
fn top(bars: f32) -> f32 {
    if bars >= 1.0 { TOP - LETTERBOX_NUDGE } else { TOP }
}

/// A level starts (FUN_000d6520 -> FUN_000b8b40): in a mission, the card is made with the
/// mission's title and the zone's chapter, its clock at 0 (BF_TITLE_AT). A map that isn't a
/// campaign zone (a test map, a multiplayer level) gets none.
fn begin(mut commands: Commands, mut card: ResMut<Card>, game: Res<GameData>, current: Res<CurrentMap>, campaign: Res<flags::Campaign>) {
    *card = Card { log: std::env::var("BF_TITLE_LOG").is_ok(), ..default() };
    let mission = game.0.missions.mission_of(&current.0).map(|(m, _)| m).filter(|_| campaign.in_mission);
    let Some(mission) = mission else {
        if card.log {
            println!("title card: none on {} (not a campaign mission)", current.0);
        }
        return;
    };
    let chapter = mission.chapter_of(&current.0);
    let Some(widgets) = widgets_of(&game.0.strings, mission.title, chapter) else {
        if card.log {
            println!("title card: none on {} (mission h_{:08x} has no Title)", current.0, mission.id);
        }
        return;
    };
    // (a node over the whole window with the game's 4:3 screen centred in it, as the HUD's roots)
    let window = commands.spawn((Name::new("title card"), GlobalZIndex(CARD_Z),
                                 Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center, ..default() })).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();
    let start = std::env::var("BF_TITLE_AT").ok().and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(0.0);
    let rgb = Color::srgb_u8(COLOUR[0], COLOUR[1], COLOUR[2]);
    let shadow = Color::srgba(SHADOW[0], SHADOW[1], SHADOW[2], SHADOW[3]);
    // each widget's x: the safe area's right edge less its width, its widest line's: the lines
    // of the title left-aligned to one another (FUN_000b6730: (right - left) - the width)
    let right = 640.0 - game.0.hud_safe[0];
    let mut placed = vec![];
    for w in &widgets {
        let width = w.text.split('\n').map(|l| w.record.measure(l)).fold(0.0f32, f32::max);
        let left = right - width;
        for (i, l) in w.text.split('\n').enumerate() {
            // (an empty line, the tutorial's middle one, draws nothing and takes its 25 units)
            if l.is_empty() {
                continue;
            }
            let down = w.down + w.record.line * i as f32;
            let mut line = commands.spawn((ChildOf(screen), record_text(l, left, top(0.0) + down, w.record, vec![rgb], Some(shadow), Across::Left)));
            line.entry::<RecordText>().and_modify(move |mut t| t.alpha = alpha_at(start));
            card.lines.push((line.id(), down));
            placed.push((l.to_string(), left, down));
        }
    }
    card.elapsed = Some(start);
    if card.log {
        println!("title card: {} mission h_{:08x}, chapter {}: {:?} (text, x, y under {TOP}), {SPAN} s, clock at {start}", current.0, mission.id,
                 chapter.map_or("none".to_string(), |n| n.to_string()), placed);
    }
}

/// The card's frame (FUN_000b69a0): the clock on by the game's frame time (stopped while the
/// game is paused, as the HUD's is not traced: an assumption), every line given the alpha and
/// the place the letterbox leaves.
fn step(time: Res<Time>, mut card: ResMut<Card>, cutscenes: Res<script::Cutscenes>, mut texts: Query<&mut RecordText>) {
    let Some(elapsed) = card.elapsed else { return };
    let elapsed = elapsed + frame_dt(&time);
    card.elapsed = Some(elapsed);
    let alpha = alpha_at(elapsed);
    let top = top(cutscenes.bars());
    for &(e, down) in &card.lines {
        if let Ok(mut t) = texts.get_mut(e) {
            if t.alpha != alpha {
                t.alpha = alpha;
            }
            if t.y != top + down {
                t.y = top + down;
            }
        }
    }
    let phase = phase_at(elapsed);
    if card.log && card.phase != Some(phase) {
        println!("title card: {phase:?} at {elapsed:.2} s (alpha {alpha:.2}, bars {:.2})", cutscenes.bars());
    }
    card.phase = Some(phase);
    if phase == Phase::Gone {
        card.elapsed = None;
        for (e, _) in std::mem::take(&mut card.lines) {
            if let Ok(mut c) = texts.get_mut(e) {
                c.alpha = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fade is the measured one: nothing for 4 s, up over the 5th, full until the 15th, down
    /// over the 16th (take03, take12, #196 take27).
    #[test]
    fn alpha_is_the_takes_triangle() {
        assert_eq!(alpha_at(0.0), 0.0);
        assert_eq!(alpha_at(4.0), 0.0);
        assert!((alpha_at(4.5) - 0.5).abs() < 1e-5);
        assert_eq!(alpha_at(5.0), 1.0);
        assert_eq!(alpha_at(10.0), 1.0);
        assert_eq!(alpha_at(15.0), 1.0);
        assert!((alpha_at(15.5) - 0.5).abs() < 1e-5);
        assert_eq!(alpha_at(16.0), 0.0);
        assert_eq!(alpha_at(20.0), 0.0);
    }

    /// The phases follow the alpha.
    #[test]
    fn phases() {
        assert_eq!(phase_at(1.0), Phase::Waiting);
        assert_eq!(phase_at(4.5), Phase::FadeIn);
        assert_eq!(phase_at(8.0), Phase::Full);
        assert_eq!(phase_at(15.5), Phase::FadeOut);
        assert_eq!(phase_at(16.5), Phase::Gone);
    }

    /// The tutorial's title is one widget of three lines (the middle one empty); "Chapter n" is a
    /// second, 26 under the top, at 12 points.
    #[test]
    fn widgets() {
        let mut strings: HashMap<u32, String> = HashMap::new();
        strings.insert(1, "Brute Force\n\nWelcome to the 23rd!".into());
        strings.insert(CHAPTERS[1], "Chapter 2".into());
        let w = widgets_of(&strings, 1, None).unwrap();
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].text.split('\n').collect::<Vec<_>>(), vec!["Brute Force", "", "Welcome to the 23rd!"]);
        let w = widgets_of(&strings, 1, Some(2)).unwrap();
        assert_eq!((w[1].text.as_str(), w[1].down, w[1].record.line), ("Chapter 2", 26.0, 19.0));
        assert!(widgets_of(&strings, 0, None).is_none());
    }

    /// The card is a unit higher only under a full letterbox.
    #[test]
    fn place() {
        assert_eq!(top(0.0), 90.0);
        assert_eq!(top(0.7), 90.0);
        assert_eq!(top(1.0), 89.0);
    }
}
