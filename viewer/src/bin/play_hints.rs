//! The tutorial's HUD hints and button gates (#224, step 5 of #28; viewer/docs/hints.md): the
//! message trigger h_e9b3ae8e, the input trigger h_e2b9c3a2, the event-trigger and the AI events
//! it hears, and the HUD's two hint lines. A child of play_triggers.rs: the three classes are the
//! runtime's (their handlers and updates are `Triggers`' and dispatched from there); this
//! module's systems feed it the frame's virtual buttons and AI events and draw the lines. Written
//! from default.xbe's code (the addresses are in the comments and in viewer/docs/code-map.md);
//! take04 confirms the line's place, font and blink to a unit and a frame.
//!
//! The message trigger (vtable 0x395bf0; parser FUN_002077b0, handler 0x2078e0): OPERATE sends
//! OPERATED (its iD, the event's subject and data) to its targets, then sets each player's two
//! hint lines (the widgets at the player view's +0x3c and +0x40, class 0x39f0f8): a line it shows
//! gets the kind, button and text of the trigger's line and starts blinking; one it doesn't is
//! hidden at once.
//!
//! A hint line (constructor FUN_000cb1c0, update 0xcb220; the glyph line's three texts made by
//! 0xc9b50: +0x68 "Press " / "Hold ", +0x64 the button's glyph, +0x6c the text):
//!  * it stands at the HUD's safe area's left (51) and its top (39) + 30 + 16 full screen:
//!    (51, 85); the second line 20 lower. The skill's name below the bars moves 20 lower for each
//!    line up (0xc73b4, play_hud.rs);
//!  * kind 1 is "Press " then the glyph of the pad button bound to its virtual button, kind 2
//!    "Hold " then the glyph (FUN_000ca000); the glyph 3 units lower, the text 4 units after it.
//!    A stick or a d-pad direction is a word instead of a glyph ("D-Pad Up": FUN_000ca1a0); with
//!    no button (kind 0's VB 0) the text alone, from the line's start (FUN_000c9d70);
//!  * the words in the body font at 14 points (h_0c626fd5, the record 0x3c35a8, as the briefings'
//!    subtitles), pale blue (192, 215, 255) over a shadow 1 unit down and right (10, 10, 10) at
//!    half the alpha (FUN_000c99d0); the glyph in the button icon font at 16 (h_0e38a4ca: the atlas
//!    h_0b630034, the record 0x3c6920), white;
//!  * a line just shown blinks: its timer (+0x78) is set to 3 s and counted down by the frame's
//!    time, and it's drawn while the fraction of 2 x the timer is under a half (FUN_000cb410:
//!    +0x7c 4 x 0.5): off 0.25 s, on 0.25 s, six times, then steady. take04's BACK hint: off from
//!    49.77 s, six off phases 0.51 s apart, steady from 52.60 s (the watched take's clock runs
//!    at ~0.95).
//!
//! The input trigger (vtable 0x3954a8; update 0x2008a0): while it's enabled, every frame its
//! virtual button's value isn't 0 on a controller's pad, TRUE (iD 0, no subject, data -1) to its
//! targets. Its `controller` isn't read: any pad counts. A virtual button's value comes from the
//! pad's bindings (0x1e57d0: game-options' `<action>` block and the hard-coded START, BACK and
//! d-pad; FUN_001e7b30 counts a mode 0 binding while it's held, a mode 1 one the frame it goes
//! down). The demo's keys stand for the pad's buttons (`BINDINGS`); none is read while the
//! game's clock is stopped under a menu, as the world isn't updated then.
//!
//! The event-trigger (vtable 0x395440; handler 0x202dd0): an AI event about an object it listens
//! to (FUN_001fe440 hands the object's AI events to its event-triggers: signal 17, the object as
//! the subject, the EVT_ number at the event's +0x14) whose number is its `event-id` (its default
//! filter h_fe4bc988, FUN_0004c800) goes to its targets as EVENT_MATCH, and it disables itself.
//! The AI events the demo posts (`ai_events`): a character's health falling under 75, 50 and 25
//! percent of its most (EVT_HEALTH_PCT_75 / 50 / 25: the damage handler, 0x116180), and
//! EVT_SOUND_ENEMY_NOISE as a stub.
//!
//! Test hooks: BF_TEST_VB=<vb>,<s>[,<hold s>][;...] holds a virtual button from that time (the
//! game's clock) for that long (0.1 s by default, and at least the first frame from that time),
//! as a pad would; BF_HINT_LOG=1 prints the hint
//! lines shown and cleared, the AI events posted, the presses that reach an input trigger and
//! the panels (play_panels.rs).

use std::collections::HashMap;

use bevy::input::InputSystem;

use super::super::briefing::{self, FontAtlas};
use super::super::hud::HudRoot;
use super::*;

/// The hint line's place below the HUD's safe area's top: 30 ([0x3a221c], added at 0xcb280) and
/// 16 more full screen (0xcb2b7, in a view at least 600 x 400), so (51, 85) with game-options'
/// margins (the safe area's left and top: FUN_000b2870 / FUN_000b2800, the margins 48 and 36
/// + SAFE_IN).
const LINE_DOWN: f32 = 30.0 + 16.0;
/// The safe area's edge inside the margins (FUN_000b2870 / FUN_000b2800: + 3.0).
const SAFE_IN: f32 = 3.0;
/// The second line is this much lower full screen (0xcb37b: its +0x74 is 2).
const LINE_STEP: f32 = 20.0;
/// The gap between the glyph and the text: +0x70, 4.0 (FUN_000c9b10), added by FUN_000ca000.
const GLYPH_GAP: f32 = 4.0;
/// The glyph stands this much lower than the words (FUN_000ca000 sets its +6 to 3).
const GLYPH_DROP: f32 = 3.0;
/// A line just shown blinks this long (s): the handler sets +0x78 to at least 3.0 (0x207b3f).
const BLINK_TIME: f32 = 3.0;
/// The blink: shown while the fraction of BLINK_RATE x 0.5 x the timer is under a half
/// (FUN_000cb410 with +0x7c, 4.0 from the constructor): two blinks a second.
const BLINK_RATE: f32 = 4.0;
/// The words' colour: bytes c0 d7 ff (0x4a5ca0, set by 0xc9c0a; the draw's ARGB puts +0x38 red,
/// +0x3a green, +0x39 blue). take04 measures 200-212, 215-228, 251-255.
const TEXT_COLOUR: [u8; 3] = [0xC0, 0xD7, 0xFF];
/// The words' shadow: 1 unit down and right in 0a 0a 0a at half the line's alpha (FUN_000c99d0:
/// alpha x 128). The console blends in stored sRGB values, Bevy's UI in linear light: a dark
/// shadow at a share p is drawn at 1 - (1 - p)^2.2 (as play_stealth.rs `srgb_alpha`).
const SHADOW_COLOUR: [u8; 3] = [10, 10, 10];
const SHADOW_SHARE: f32 = 0.5;
/// "Press " and "Hold " (string ids h_ff43bc83 and h_1cc51f04: FUN_000ca000 for kinds 1 and 2).
const S_PRESS: u32 = 0xFF43_BC83;
const S_HOLD: u32 = 0x1CC5_1F04;
/// The use prompt (a HUD widget of its own, vtable 0x3a02b4, its update 0xaa130): a glyph line as
/// the hint lines are (the same three texts, 0xc9b50; the body font at 14 points full screen,
/// FUN_000ca2f0): for a target in reach whose use takes a hold (0xaad20), "Hold " and the
/// ACTION_BUTTON's glyph (VB 6, the X button: FUN_000ca290 at 0xaa19c), then its text
/// (FUN_000aa600): a world button's "to activate %s." (h_f9d29692, 0xaa801) with its name. take09
/// and take11 show it at the first hint line's place, steady.
const PROMPT_KIND: i64 = 2;
const PROMPT_VB: i64 = 6;
const S_TO_ACTIVATE: u32 = 0xF9D2_9692;
/// A lying gun's lines (FUN_000aa600's weapon case, #17): the character can take it (FUN_000aad20
/// -> FUN_000a9910 agreeing), "Hold [X] to swap %s." (h_f1285f72, 0xaa789) or, for a gun with no
/// magazine prop and no ammo-regen, "Hold [X] to swap %s. (%d shots)" (h_e456c40f, 0xaa774); it
/// can't, "%s cannot pick up %s." (h_0ff85d29, 0xaa7e6), with no "Hold " and no glyph
/// (FUN_000ca000 with the empty glyph): the words from the line's start (#61 take04 3.25 s).
const S_TO_SWAP: u32 = 0xF128_5F72;
const S_TO_SWAP_SHOTS: u32 = 0xE456_C40F;
const S_CANNOT_PICK_UP: u32 = 0x0FF8_5D29;
/// The use prompt stands at the first hint line's place, 20 lower while the HUD's message widget
/// (hud+0xfc: the skill's name) shows, and 20 lower again for each of the update's two tests of
/// the player view's +0x3c, the first hint line (0xaa292-0xaa2e8: the second test reads +0x3c again
/// where +0x40, the second line, would be expected; so 40 under a first line, none for a second).
const PROMPT_STEP: f32 = 20.0;
/// The button icon font's atlas, h_0b630034 (256 x 512; FUN_000cd5d0 registers its 16-point
/// record with v x 0.5 + 0.684).
const ICON_ATLAS: u32 = 0x0B63_0034;
/// The icon font's 16-point line height (the record 0x3c6920's +4): a glyph's cell is one more.
const ICON_LINE: f32 = 21.0;
/// The icon font's 16-point glyphs the hint lines use, 'A' to 'J' (the record 0x3c6920 from its
/// +0x1c, 0x18 bytes each, read from default.xbe): the cell's corners on the atlas in pixels (u x
/// 256, (v x 0.5 + 0.68359375) x 512 - 1), then the offset, width and advance. A, B, C, D are the
/// A, B, X and Y buttons, E and F the left and right triggers, G START, H BACK, I White, J Black
/// (the atlas's own pictures; take04 BACK, X, B, Y, take08 Black). The renderer takes one of the
/// record's texels off each corner (FUN_000ccd20: (coordinate x size - 1) / size, the record's
/// size, 0x40d644-0x40d650): one atlas row for the 14-point body font, whose cells the demo draws
/// as the record has them (as the briefings do: take01 matches), and two here, the record being 256
/// tall on a 512 atlas; so these are a row higher than the record's. take03 / take04 show it: the
/// top row of START's and BACK's letters drawn, no stray texel of the row below, the discs' top
/// three rows cut (take04's B disc 30 px, 20 rows).
const ICON_GLYPHS: [(char, f32, f32, f32, f32, i8, u8, u8); 10] = [
    ('A', 40.16, 371.09, 63.25, 393.17, 1, 22, 22),
    ('B', 64.25, 371.09, 87.34, 393.17, 1, 22, 22),
    ('C', 88.35, 371.09, 111.44, 393.17, 1, 22, 22),
    ('D', 112.44, 371.09, 135.53, 393.17, 1, 22, 22),
    ('E', 136.53, 371.09, 153.60, 393.17, 1, 16, 17),
    ('F', 154.60, 371.09, 171.67, 393.17, 1, 16, 17),
    ('G', 172.67, 371.09, 204.80, 393.17, 1, 31, 33),
    ('H', 0.00, 393.17, 26.10, 415.26, 4, 25, 30),
    ('I', 27.11, 393.17, 45.18, 415.26, 2, 17, 19),
    ('J', 46.18, 393.17, 64.25, 415.26, 2, 17, 19),
];

/// EVT_HEALTH_PCT_75, _50 and _25 (the EVT_ table 0x3bdb9c), each with the share of the most it
/// marks: posted when a character's health, in whole percent of its most, falls from at least
/// the mark to under it (a character's damage, FUN_00115f90 at 0x116180: 0x39, 0x38, 0x37 against
/// 75, 50, 25).
const HEALTH_EVENTS: [(i32, i64); 3] = [(75, 57), (50, 56), (25, 55)];
/// EVT_SOUND_ENEMY_NOISE (49): the tutorial's second building's guards hear Tex
/// (event-trigger156). Its sources (the noise system) aren't traced: the demo posts it, **a
/// stub**, when one of them starts to see a hostile or is hurt (the ticket's reading).
const EVT_SOUND_ENEMY_NOISE: i64 = 49;
/// BF_TEST_VB's default hold (s).
const TEST_HOLD: f32 = 0.1;
/// The hint lines over the rest of the HUD (whose root has no z-index), under the menus (20).
const HINT_Z: i32 = 1;

/// What stands for a pad button in a hint line (FUN_000c9d70's table at 0xc9ee4 by the pad input
/// bound to the virtual button): a letter of the icon font (the face buttons, the triggers,
/// White, Black, START, BACK) or, for a stick or a d-pad direction, a string's words.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Glyph {
    Icon(char),
    Word(u32),
}

/// A virtual button (the VB_ name table 0x3bf39c, its records from 0x3bf398: 5 VB_FIRE, 6
/// VB_ACTION_BUTTON, 7 VB_USE_INV ... 62 VB_OPEN_OBJECTIVES_MENU, 63 VB_USE_HEALTHPACK) as the
/// pad's default bindings give it and the
/// demo's controls stand for it: its number, whether its binding counts the frame it goes down
/// (mode 1) or every frame it's held (mode 0), what its hint shows, and the demo's keys, mouse
/// button and pad buttons (a pad isn't read under BF_BACKGROUND).
struct Binding {
    vb: i64,
    press: bool,
    glyph: Glyph,
    keys: &'static [KeyCode],
    mouse: Option<MouseButton>,
    pads: &'static [GamepadButton],
}

/// The bindings (0x1e57d0 builds them; the modes are game-options' `<action>` h_efb3b9f5 and the
/// hard-coded records'): the pad input each is bound to picks the glyph (0xc9ee4: 9 A 'A', 10 B
/// 'B', 11 X 'C', 12 Y 'D', 13 White 'I', 14 Black 'J', 15 left trigger 'E', 16 right trigger
/// 'F', 17 / 18 the stick clicks' words, 19 START 'G', 20 BACK 'H', 5-8 the d-pad's words). The
/// demo's keys are its controls for the same actions (viewer/docs/play.md): the left mouse fires,
/// E and R are the X button (use, reload), G the left trigger, V White, Q Y, Z the left stick's
/// click, Tab B, the digits the d-pad (4 up to Tex's portrait, 2 right to Flint's, 1 down to
/// Brutus's, 3 left to Hawk's), Escape START, the right mouse the right stick's click, Space A, I
/// BACK; B stands for Black (its Medkit shortcut is play_itembox.rs's). On the pad,
/// White and Black are the shoulder buttons (as xemu maps them: take08 RB, take09 LB).
const BINDINGS: [Binding; 16] = [
    Binding { vb: 5, press: false, glyph: Glyph::Icon('F'), keys: &[], mouse: Some(MouseButton::Left), pads: &[GamepadButton::RightTrigger2] },
    Binding { vb: 6, press: false, glyph: Glyph::Icon('C'), keys: &[KeyCode::KeyE, KeyCode::KeyR], mouse: None, pads: &[GamepadButton::West] },
    Binding { vb: 7, press: false, glyph: Glyph::Icon('E'), keys: &[KeyCode::KeyG], mouse: None, pads: &[GamepadButton::LeftTrigger2] },
    Binding { vb: 9, press: true, glyph: Glyph::Icon('I'), keys: &[KeyCode::KeyV], mouse: None, pads: &[GamepadButton::LeftTrigger] },
    Binding { vb: 10, press: true, glyph: Glyph::Icon('D'), keys: &[KeyCode::KeyQ], mouse: None, pads: &[GamepadButton::North] },
    Binding { vb: 11, press: false, glyph: Glyph::Word(0x0DB4_9C4B), keys: &[KeyCode::KeyZ], mouse: None, pads: &[GamepadButton::LeftThumb] },
    Binding { vb: 12, press: false, glyph: Glyph::Icon('B'), keys: &[KeyCode::Tab], mouse: None, pads: &[GamepadButton::East] },
    Binding { vb: 13, press: false, glyph: Glyph::Word(0x1E25_1D69), keys: &[KeyCode::Digit4], mouse: None, pads: &[GamepadButton::DPadUp] },
    Binding { vb: 14, press: false, glyph: Glyph::Word(0xF48C_E0DC), keys: &[KeyCode::Digit2], mouse: None, pads: &[GamepadButton::DPadRight] },
    Binding { vb: 15, press: false, glyph: Glyph::Word(0xF1B3_F1AA), keys: &[KeyCode::Digit1], mouse: None, pads: &[GamepadButton::DPadDown] },
    Binding { vb: 16, press: false, glyph: Glyph::Word(0xFB51_5609), keys: &[KeyCode::Digit3], mouse: None, pads: &[GamepadButton::DPadLeft] },
    Binding { vb: 17, press: true, glyph: Glyph::Icon('G'), keys: &[KeyCode::Escape], mouse: None, pads: &[GamepadButton::Start] },
    Binding { vb: 34, press: true, glyph: Glyph::Word(0x1327_F9ED), keys: &[], mouse: Some(MouseButton::Right), pads: &[GamepadButton::RightThumb] },
    Binding { vb: 36, press: true, glyph: Glyph::Icon('A'), keys: &[KeyCode::Space], mouse: None, pads: &[GamepadButton::South] },
    Binding { vb: 62, press: true, glyph: Glyph::Icon('H'), keys: &[KeyCode::KeyI], mouse: None, pads: &[GamepadButton::Select] },
    Binding { vb: 63, press: false, glyph: Glyph::Icon('J'), keys: &[KeyCode::KeyB], mouse: None, pads: &[GamepadButton::RightTrigger] },
];

/// A virtual button's binding, if it has one.
fn binding(vb: i64) -> Option<&'static Binding> {
    BINDINGS.iter().find(|b| b.vb == vb)
}

/// One of the HUD's two hint lines as the message triggers leave it (the widget's fields):
/// shown (+0x13 bit 0 clear), what the trigger that last showed it set (its line), the blink's
/// timer (+0x78), who set it, and a count of its changes (the drawing rebuilds on one).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LineState {
    pub(crate) shown: bool,
    line: data::HintLine,
    timer: f32,
    by: u32,
    serial: u32,
}

/// A BF_TEST_VB entry: the virtual button, from when and for how long (the game's clock), and
/// whether its press has gone down yet (a mode 1 button counts that frame only).
#[derive(Clone, Copy, Debug)]
struct TestPress {
    vb: i64,
    at: f32,
    hold: f32,
    down: bool,
}

/// What a character looked like to the AI events last frame (`ai_events`): its health in whole
/// percent of its most, its health and its most, and whether it saw a hostile.
#[derive(Clone, Copy, Debug)]
struct Watch {
    percent: i32,
    health: f32,
    most: f32,
    saw: bool,
}

/// The hint side of the triggers' state (`Triggers::hints`), one per level: the HUD's two hint
/// lines, the virtual buttons held this frame (the `buttons` system's), this frame's AI events
/// (`ai_events`'), what those watch, and the panels (play_panels.rs).
#[derive(Default)]
pub(crate) struct HintWorld {
    pub(crate) lines: [LineState; 2],
    held: Vec<i64>,
    /// the AI events of this frame: (the object's name, the EVT_ number)
    ai_events: Vec<(u32, i64)>,
    watch: HashMap<u32, Watch>,
    /// BF_TEST_VB's entries, read at the level's first frame
    tests: Option<Vec<TestPress>>,
    /// BF_HINT_LOG is set
    pub(crate) log: bool,
    /// the virtual buttons held last frame (BF_HINT_LOG logs a hold the frame it starts)
    logged: Vec<i64>,
    pub(crate) panels: super::panels::PanelWorld,
}

/// Which hint lines are up (shown, blinking or not), for the skill's name under the bars, which
/// stands 20 lower for each (0xc73b4-0xc73cb; play_hud.rs).
#[derive(Resource, Default)]
pub(crate) struct HintLines(pub(crate) [bool; 2]);

/// The hint lines' nodes and atlases (`build`): each line's node, the icon atlas, the safe
/// area's corner, and the change each line was last drawn for; the use prompt's node and the
/// text and drop it was last drawn with.
#[derive(Resource)]
struct HintHud {
    lines: [Entity; 2],
    icons: Option<Handle<Image>>,
    corner: Vec2,
    drawn: [u32; 2],
    prompt: Entity,
    prompt_drawn: Option<(String, f32)>,
}

/// The frame's virtual buttons once a tutorial panel has taken the frame's presses (none reaches
/// the gates under one) and before the objectives menu does (its opening on BACK must still let
/// the BACK gate see it: take04's START hint 3 s of clock after the BACK press), and the AI events
/// before the triggers' frame; the lines after it, and after the use prompt's target is known
/// (play.rs `doors`, play_weapon_pickups.rs `use_target`).
pub fn plugin(app: &mut App) {
    app.init_resource::<HintLines>()
        .add_systems(OnEnter(AppState::Playing), build.after(setup))
        .add_systems(PreUpdate, buttons.after(InputSystem).after(super::panels::input).before(super::super::objectives::ObjectivesControls)
                     .run_if(in_state(AppState::Playing)))
        .add_systems(Update, (ai_events.before(super::run), draw.after(super::run).after(super::super::weapon_pickups::use_target))
                     .run_if(in_state(AppState::Playing)));
}

/// The hint lines' nodes, hidden, in a HUD root of their own, and the icon atlas.
///
/// The lines are a HUD widget and go with the HUD: their root is a `HudRoot`, which the editor's
/// edit mode hides and the letterbox hides while its bars are in (play_script.rs `view`: from a
/// quarter in, shown again as they go out). A line set under the bars shows once the HUD is back,
/// its blink timer having run on meanwhile (whether the game's widget counts down while the HUD is
/// off isn't traced).
fn build(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, mut shown: ResMut<HintLines>) {
    let icons = game.0.texture_rgba(ICON_ATLAS).map(|(w, h, px)| images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, px,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())));
    let window = commands.spawn((HudRoot, Name::new("hint lines"), GlobalZIndex(HINT_Z),
                                 Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center, ..default() })).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();
    let full = || Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0),
                         height: Val::Percent(100.0), ..default() };
    let lines = [commands.spawn((full(), Visibility::Hidden, ChildOf(screen))).id(), commands.spawn((full(), Visibility::Hidden, ChildOf(screen))).id()];
    let prompt = commands.spawn((full(), Visibility::Hidden, ChildOf(screen))).id();
    let corner = Vec2::new(game.0.hud_safe[0] + SAFE_IN, game.0.hud_safe[1] + SAFE_IN + LINE_DOWN);
    commands.insert_resource(HintHud { lines, icons, corner, drawn: [0; 2], prompt, prompt_drawn: None });
    shown.0 = [false; 2];
}

/// BF_TEST_VB's entries (see the module notes).
fn read_tests() -> Vec<TestPress> {
    let Ok(v) = std::env::var("BF_TEST_VB") else { return vec![] };
    v.split(';').filter_map(|e| {
        let n: Vec<f32> = e.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        Some(TestPress { vb: *n.first()? as i64, at: *n.get(1)?, hold: n.get(2).copied().unwrap_or(TEST_HOLD), down: false })
    }).collect()
}

/// The frame's virtual buttons (FUN_001e7b30 for each binding): a mode 0 one while any of its
/// keys, its mouse button or its pad buttons is held, a mode 1 one the frame one goes down; the
/// mouse's buttons only while it's captured (the click that captures it doesn't fire); the left
/// one also when the autopilot fires (BF_TEST_FIRE); and BF_TEST_VB's. None while the game's
/// clock is stopped (a menu or a panel over the game: the world isn't updated) or in the map
/// editor's edit mode. Logged with BF_HINT_LOG when one reaches an enabled input trigger
/// (`input_update`).
#[allow(clippy::too_many_arguments)]
fn buttons(mut tr: ResMut<Triggers>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<&Gamepad>,
           windows: Query<&Window, With<PrimaryWindow>>, player: Res<Player>, editing: Option<Res<State<editor::EditorMode>>>) {
    let h = &mut tr.hints;
    // (last frame's, for the log: a hold is logged the frame it starts)
    h.logged = std::mem::take(&mut h.held);
    if h.tests.is_none() {
        h.tests = Some(read_tests());
        h.log = std::env::var("BF_HINT_LOG").is_ok();
    }
    if paused() || editing.is_some_and(|m| *m.get() == editor::EditorMode::Edit) {
        return;
    }
    let captured = windows.single().is_ok_and(|w| w.cursor_options.grab_mode != bevy::window::CursorGrabMode::None);
    for b in &BINDINGS {
        let on = if b.press {
            keys.any_just_pressed(b.keys.iter().copied()) || b.mouse.is_some_and(|m| captured && mouse.just_pressed(m))
                || pads.iter().any(|p| b.pads.iter().any(|&x| p.just_pressed(x)))
        } else {
            keys.any_pressed(b.keys.iter().copied()) || b.mouse.is_some_and(|m| captured && mouse.pressed(m))
                || pads.iter().any(|p| b.pads.iter().any(|&x| p.pressed(x)))
                || (b.vb == 5 && player.fire)
        };
        if on {
            h.held.push(b.vb);
        }
    }
    let t = player.sim_time;
    for p in h.tests.iter_mut().flatten() {
        // (held for at least the first frame from its time, a hold shorter than a frame too)
        let within = t >= p.at && (t < p.at + p.hold || !p.down);
        let press = binding(p.vb).is_some_and(|b| b.press);
        if within && (!press || !p.down) && !h.held.contains(&p.vb) {
            h.held.push(p.vb);
        }
        p.down |= within;
    }
}

/// The AI events of this frame, from each character the triggers know by name (the player and
/// the squad, the copies the triggers spawned among them) against last frame: its health in whole
/// percent of its most (truncated, as the damage handler's cvttss2si) falling from at least 75,
/// 50 or 25 to under it (`HEALTH_EVENTS`); and the EVT_SOUND_ENEMY_NOISE stub, for a character
/// the player doesn't control, the frame it starts to see a hostile (its AI's target) or is hurt.
/// The game posts them in its damage handler; here they're a frame later, before the triggers'
/// frame, which hands them to the event-triggers listening (`event_update`).
fn ai_events(mut tr: ResMut<Triggers>, player: Res<Player>, squad: Res<Squad>) {
    if tr.live.is_empty() {
        return;
    }
    let mut events = vec![];
    let mut seen = HashMap::new();
    for (u, controlled) in std::iter::once((&*player, true)).chain(squad.0.iter().map(|m| (m, false))) {
        let name = u.trigger_name;
        // (a character still being set up, its body not loaded, isn't watched yet: its health is
        // set from its type then)
        if name == 0 || u.loaded.is_none() {
            continue;
        }
        let percent = if u.max_health > 0.0 { (u.health.max(0.0) * 100.0 / u.max_health) as i32 } else { 0 };
        let now = Watch { percent, health: u.health, most: u.max_health, saw: u.ai_target.is_some() };
        if let Some(was) = tr.hints.watch.get(&name).filter(|w| w.most == u.max_health) {
            for &(mark, event) in &HEALTH_EVENTS {
                if was.percent >= mark && percent < mark {
                    events.push((name, event, format!("health {}% -> {percent}%", was.percent)));
                }
            }
            if !controlled && !u.dead && ((now.saw && !was.saw) || now.health < was.health) {
                let why = if now.health < was.health { format!("hurt, {:.1} -> {:.1}", was.health, now.health) } else { "sees a hostile".into() };
                events.push((name, EVT_SOUND_ENEMY_NOISE, why));
            }
        }
        seen.insert(name, now);
    }
    if tr.hints.log {
        for (name, event, why) in &events {
            println!("t {:.2}: AI event {} about {} ({why})", tr.clock, evt_name(*event), tr.label(*name));
        }
    }
    let events = events.into_iter().map(|(name, event, _)| (name, event)).collect();
    tr.hints.watch = seen;
    tr.hints.ai_events = events;
}

/// An EVT_ number's name, for the log (the table 0x3bdb9c's, the few the demo posts).
fn evt_name(e: i64) -> String {
    match e {
        49 => "EVT_SOUND_ENEMY_NOISE".into(),
        55 => "EVT_HEALTH_PCT_25".into(),
        56 => "EVT_HEALTH_PCT_50".into(),
        57 => "EVT_HEALTH_PCT_75".into(),
        n => format!("EVT {n}"),
    }
}

/// A piece of a hint line: a glyph's cell from one of the two atlases, where it goes (units from
/// the line's corner), its size, and its colour.
struct Cell {
    icon: bool,
    rect: Rect,
    at: Vec2,
    size: Vec2,
    colour: Color,
}

/// A text in the body font at 14 points as FUN_000ccd20 draws it: each glyph's cell, (width + 1)
/// x 24, at the pen + its offset (a space's left empty), its shadow first; the pen then moved by
/// the offset and the advance. Returns how far the pen went (FUN_000ccae0's width: its furthest).
fn words(cells: &mut Vec<Cell>, text: &str, from: Vec2) -> f32 {
    let shadow = Color::srgba_u8(SHADOW_COLOUR[0], SHADOW_COLOUR[1], SHADOW_COLOUR[2], 255)
        .with_alpha(1.0 - (1.0 - SHADOW_SHARE).powf(2.2));
    let colour = Color::srgb_u8(TEXT_COLOUR[0], TEXT_COLOUR[1], TEXT_COLOUR[2]);
    let (mut pen, mut width) = (0.0f32, 0.0f32);
    let mut glyphs = vec![];
    for c in text.chars() {
        let ((gx, gy, offset, w, _), advance) = briefing::glyph(c);
        if c != ' ' {
            let rect = Rect::new(gx as f32, gy as f32, gx as f32 + w as f32 + 1.0, gy as f32 + briefing::LINE);
            glyphs.push((rect, Vec2::new(from.x + pen + offset as f32, from.y), Vec2::new(w as f32 + 1.0, briefing::LINE)));
        }
        pen += advance;
        width = width.max(pen);
    }
    for &(rect, at, size) in &glyphs {
        cells.push(Cell { icon: false, rect, at: at + Vec2::ONE, size, colour: shadow });
    }
    for (rect, at, size) in glyphs {
        cells.push(Cell { icon: false, rect, at, size, colour });
    }
    width
}

/// A glyph line's cells (a hint line's or the use prompt's; see the module notes): "Press " or
/// "Hold " and the glyph of virtual button `vb` for kinds 1 and 2 (the glyph alone for kind 0),
/// or the word of a stick or a d-pad direction, then `words` 4 units after them; the words alone
/// for a line with no button.
fn layout(kind: i64, vb: i64, words_text: &str, strings: &HashMap<u32, String>) -> Vec<Cell> {
    let text = |id: u32| strings.get(&id).cloned().unwrap_or_default();
    let mut cells = vec![];
    let mut x = 0.0;
    match binding(vb).map(|b| b.glyph) {
        Some(Glyph::Icon(c)) => {
            let lead = match kind {
                1 => text(S_PRESS),
                2 => text(S_HOLD),
                _ => String::new(),
            };
            let pen = if lead.is_empty() { 0.0 } else { words(&mut cells, &lead, Vec2::ZERO) };
            if let Some(&(_, x0, y0, x1, y1, offset, _, advance)) = ICON_GLYPHS.iter().find(|g| g.0 == c) {
                cells.push(Cell { icon: true, rect: Rect::new(x0, y0, x1, y1), at: Vec2::new(pen + offset as f32, GLYPH_DROP),
                                  size: Vec2::new(x1 - x0, ICON_LINE + 1.0), colour: Color::WHITE });
                x = pen + GLYPH_GAP + (offset as f32 + advance as f32);
            }
        }
        Some(Glyph::Word(id)) => {
            x = words(&mut cells, &text(id), Vec2::ZERO) + GLYPH_GAP;
        }
        None => {}
    }
    words(&mut cells, words_text, Vec2::new(x, 0.0));
    cells
}

/// A glyph line's cells as nodes under `parent`, from `corner` (units on the 640 x 480 screen):
/// the icon font's from `icons`, the body font's from `font`.
fn spawn_cells(commands: &mut Commands, parent: Entity, cells: Vec<Cell>, corner: Vec2, icons: &Option<Handle<Image>>, font: &Handle<Image>) {
    for c in cells {
        let image = if c.icon { icons.clone() } else { Some(font.clone()) };
        let Some(image) = image else { continue };
        let at = corner + c.at;
        commands.spawn((ImageNode { rect: Some(c.rect), color: c.colour, ..ImageNode::new(image) },
                        Node { position_type: PositionType::Absolute, left: Val::Percent(at.x / 6.4), top: Val::Percent(at.y / 4.8),
                               width: Val::Percent(c.size.x / 6.4), height: Val::Percent(c.size.y / 4.8), ..default() },
                        ChildOf(parent)));
    }
}

/// Whether a line blinking with `timer` s left is drawn now (FUN_000cb410: the fraction of
/// BLINK_RATE x 0.5 x the timer under a half; steady once the timer is out).
fn blink_on(timer: f32) -> bool {
    (BLINK_RATE * 0.5 * timer).fract() < 0.5
}

/// After the triggers' frame: each line's blink counted down by the frame's time (the game's
/// clock: still under a menu), its glyphs rebuilt when it changed, shown while it's up and its
/// blink is on; then the use prompt (`draw_prompt`).
#[allow(clippy::too_many_arguments)]
fn draw(mut commands: Commands, time: Res<Time>, mut tr: ResMut<Triggers>, game: Res<GameData>, font: Option<Res<FontAtlas>>,
        hud: Option<ResMut<HintHud>>, mut nodes: Query<&mut Visibility>, mut shown: ResMut<HintLines>,
        (use_panel, player, stamina): (Res<UsePanel>, Res<Player>, Res<super::super::hud::StaminaBar>)) {
    let (Some(mut hud), Some(font)) = (hud, font) else { return };
    let dt = frame_dt(&time);
    for n in 0..2 {
        let state = &mut tr.hints.lines[n];
        state.timer = (state.timer - dt).max(0.0);
        let (line, serial, up, timer) = (state.line, state.serial, state.shown, state.timer);
        shown.0[n] = up;
        if serial != hud.drawn[n] {
            hud.drawn[n] = serial;
            let parent = hud.lines[n];
            commands.entity(parent).despawn_related::<Children>();
            let corner = hud.corner + Vec2::Y * LINE_STEP * n as f32;
            let words_text = game.0.strings.get(&line.text).cloned().unwrap_or_default();
            spawn_cells(&mut commands, parent, layout(line.kind, line.vb, &words_text, &game.0.strings), corner, &hud.icons, &font.0);
        }
        if let Ok(mut v) = nodes.get_mut(hud.lines[n]) {
            let want = if up && blink_on(timer) { Visibility::Inherited } else { Visibility::Hidden };
            if *v != want {
                *v = want;
            }
        }
    }
    draw_prompt(&mut commands, &mut hud, &font.0, &game.0.strings, &mut nodes, &use_panel, &player, &stamina, shown.0[0]);
}

/// The prompt line's kind (2: "Hold " and the X glyph; 0: the words alone) and words for a
/// prompt (`Prompt`, FUN_000aa600's cases), from the string table.
fn prompt_words(prompt: &Prompt, strings: &HashMap<u32, String>) -> (i64, String) {
    let text = |id: u32, fallback: &str| strings.get(&id).cloned().unwrap_or_else(|| fallback.to_string());
    match prompt {
        Prompt::Activate(what) => (PROMPT_KIND, text(S_TO_ACTIVATE, "to activate %s.").replacen("%s", what, 1)),
        Prompt::Swap { name, shots: None } => (PROMPT_KIND, text(S_TO_SWAP, "to swap %s.").replacen("%s", name, 1)),
        Prompt::Swap { name, shots: Some(n) } => (PROMPT_KIND, text(S_TO_SWAP_SHOTS, "to swap %s. (%d shots)").replacen("%s", name, 1)
            .replacen("%d", &n.to_string(), 1)),
        Prompt::Refuse { who, name } => (0, text(S_CANNOT_PICK_UP, "%s cannot pick up %s.").replacen("%s", who, 1).replacen("%s", name, 1)),
    }
}

/// The use prompt (see PROMPT_KIND): a gate's wall panel in reach and in view (play.rs `doors`),
/// "Hold [X] to activate panel.", or a lying gun's line (play_weapon_pickups.rs, #17: "Hold [X]
/// to swap L-Shot-50.", "Tex cannot pick up Jax-iP."); none while scoped, as before. At the first
/// hint line's place, lowered for the skill's name and a first hint line (PROMPT_STEP), steady;
/// rebuilt when its text or place changes.
#[allow(clippy::too_many_arguments)]
fn draw_prompt(commands: &mut Commands, hud: &mut HintHud, font: &Handle<Image>, strings: &HashMap<u32, String>, nodes: &mut Query<&mut Visibility>,
               use_panel: &UsePanel, player: &Player, stamina: &super::super::hud::StaminaBar, first_line: bool) {
    let want = use_panel.prompt.as_ref().filter(|_| player.scope < 0.5).map(|prompt| {
        let (kind, text) = prompt_words(prompt, strings);
        let drop = PROMPT_STEP * (stamina.label_shown() as u32 + 2 * first_line as u32) as f32;
        (kind, text, drop)
    });
    if let Some((kind, text, drop)) = want.as_ref().filter(|w| hud.prompt_drawn.as_ref() != Some(&(w.1.clone(), w.2))) {
        let parent = hud.prompt;
        commands.entity(parent).despawn_related::<Children>();
        let vb = if *kind == 0 { 0 } else { PROMPT_VB };
        spawn_cells(commands, parent, layout(*kind, vb, text, strings), hud.corner + Vec2::Y * *drop, &hud.icons, font);
        hud.prompt_drawn = Some((text.clone(), *drop));
    }
    if let Ok(mut v) = nodes.get_mut(hud.prompt) {
        let vis = if want.is_some() { Visibility::Inherited } else { Visibility::Hidden };
        if *v != vis {
            *v = vis;
        }
    }
}

impl Triggers {
    /// 0x2078e0: a message trigger. Disabled, its base actions; enabled, each entry that answers:
    /// OPERATE sends OPERATED (its iD, the event's subject and data) to its targets, then sets the
    /// player's two hint lines from its own (`set_line`); the rest the base's; the entry counted.
    pub(super) fn message(&mut self, i: usize, ev: Ev) {
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
            let id = self.live[i].t.id;
            self.send(i, Ev { signal: sig::OPERATED, id, ..ev });
            let ClassState::Message(lines) = self.live[i].state else { return };
            let by = self.live[i].t.name;
            for (n, line) in lines.iter().enumerate() {
                self.set_line(n, *line, by);
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// A message trigger's line `n` on the player's view (0x207a17-0x207b6e): shown, it takes the
    /// line's kind, button and text, and its blink starts over (the widget's update with 1.0,
    /// then +0x78 at least BLINK_TIME: BLINK_TIME); not shown, it's hidden. BF_HINT_LOG prints it.
    fn set_line(&mut self, n: usize, line: data::HintLine, by: u32) {
        let was = self.hints.lines[n];
        let state = &mut self.hints.lines[n];
        if line.show {
            state.line = line;
            state.timer = BLINK_TIME;
            state.serial += 1;
        }
        state.shown = line.show;
        state.by = by;
        if self.hints.log && (line.show || was.shown) {
            let what = if line.show { format!("kind {} VB {} h_{:08x}", line.kind, line.vb, line.text) } else { "cleared".into() };
            println!("t {:.2}: hint line {} {what} by {}", self.clock, n + 1, self.label(by));
        }
    }

    /// The handler of the input trigger (0x2007f0) and the event-trigger (0x202dd0) for anything
    /// but an AI event: disabled, the base's; enabled, OPERATE only counted, the rest the base's.
    pub(super) fn counts_operate(&mut self, i: usize, ev: Ev) {
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
            self.entry_used(i, j, &ev);
        }
    }

    /// 0x2008a0: an input trigger's update: the base update (its FIRST_UPDATE), then while it's
    /// enabled, its virtual button held this frame (`buttons`): TRUE to its targets (iD 0, no
    /// subject, data -1). BF_HINT_LOG: the first frame of each hold.
    pub(super) fn input_update(&mut self, i: usize) {
        self.base_update(i);
        let ClassState::Input { vb } = self.live[i].state else { return };
        if !self.live[i].enabled || vb == 0 || !self.hints.held.contains(&vb) {
            return;
        }
        if self.hints.log && !self.hints.logged.contains(&vb) {
            println!("t {:.2}: VB {vb} held: {} sends TRUE", self.clock, self.who(i));
        }
        self.send(i, Ev { signal: sig::TRUE, id: 0, subject: 0, data: -1 });
    }

    /// Whether trigger `i` listens to `object` (its h_0b6b92ba lists it, or the spawn-trigger whose
    /// copy it is: FUN_001fe320's list).
    fn listens(&self, i: usize, object: u32) -> bool {
        self.listeners.get(&object).is_some_and(|l| l.contains(&i))
            || self.copy_of.get(&object).is_some_and(|t| self.copy_listeners.get(t).is_some_and(|l| l.contains(&i)))
    }

    /// An event-trigger's update (0x20a5a0: the base update), then this frame's AI events about
    /// the objects it listens to, as its handler (0x202dd0) takes FUN_001fe440's signal 17:
    /// disabled, the base's; enabled, an event whose number is its `event-id` goes to its targets
    /// as EVENT_MATCH (the AI event's fields, FUN_001fdee0: iD 0, the object, data -1) and it
    /// disables itself (its vtable +0x48, the base's 0x1feff0). The filter (FUN_0004c800): with
    /// h_fe4bc988 (none) the numbers must match; with another (the tutorial's EVT_SOUND_ENEMY_NOISE
    /// trigger's h_08267484) the code compares two values of the object's brain
    /// (FUN_0003a990, not traced): taken as passing, with the numbers matching.
    pub(super) fn event_update(&mut self, i: usize) {
        self.base_update(i);
        let ClassState::Event { event, filter } = self.live[i].state else { return };
        let events: Vec<(u32, i64)> = self.hints.ai_events.iter().copied().filter(|&(o, _)| self.listens(i, o)).collect();
        for (object, number) in events {
            let ev = Ev { signal: sig::AI_EVENT, id: 0, subject: object, data: -1 };
            if !self.live[i].enabled {
                self.base_and_after(i, &ev);
                continue;
            }
            if number != event {
                continue;
            }
            if self.log || self.hints.log {
                let note = if filter == data::EVENT_FILTER_NONE { String::new() } else { format!(" (its filter h_{filter:08x} taken as passing)") };
                println!("t {:.2}: {} hears {} about {}{note}", self.clock, self.who(i), evt_name(number), self.label(object));
            }
            self.send(i, Ev { signal: sig::EVENT_MATCH, ..ev });
            self.disable(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line just shown blinks off six times, a quarter second each, then stays on (FUN_000cb410
    /// with BLINK_TIME and BLINK_RATE; take04's BACK hint: six off phases, then steady).
    #[test]
    fn a_new_line_blinks_six_times() {
        let (mut offs, mut was, mut t) = (0, true, BLINK_TIME);
        while t > 0.0 {
            let on = blink_on(t);
            if was && !on {
                offs += 1;
            }
            was = on;
            t -= 0.01;
        }
        assert_eq!(offs, 6);
        assert!(blink_on(0.0));
    }

    /// Every icon a binding names has its cell (FUN_000c9d70's letters in ICON_GLYPHS).
    #[test]
    fn every_icon_binding_has_a_cell() {
        for b in &BINDINGS {
            if let Glyph::Icon(c) = b.glyph {
                assert!(ICON_GLYPHS.iter().any(|g| g.0 == c), "VB {} glyph {c}", b.vb);
            }
        }
    }
}
