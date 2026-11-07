//! The mission flow (#226, step 7 of #28; viewer/docs/mission.md): how a mission ends and what
//! follows, as default.xbe's campaign game mode, the world's end step and the session run it.
//! Written from the code (the addresses are in the comments and in viewer/docs/code-map/README.md);
//! take01, take05, take10 and take12 (todo/28-tutorial/) confirm the timing.
//!
//! The campaign mode (FUN_00089a50, vtable 0x3a21fc; made with each mission level by FUN_000d6490
//! into world+0xc4c): its +0x24 flags (bit 0 the level is over, bit 1 the squad is dead, bit 2 a
//! restart asked, bit 4 its countdown not run yet) and +0x20 the squad-dead countdown, 3.0 s. Its
//! update (FUN_00089c90) runs the countdown (FUN_00089d30): while the squad has members but none
//! alive (FUN_0005baf0) and no cutscene has the world (world+0xc50 bit 4), it counts down by the
//! frame's time; at 0 it sets bit 1 and bit 0 (its +0x14). A level-over-trigger with h_18f66101
//! sets bit 0 too (play_level_over.rs). take05's watch log: Tex dead at clk 324330, +0x24 48 ->
//! 32 at once (bit 4 gone: the countdown runs), 35 at clk 327269 (bits 0 and 1), 2.94 s later.
//!
//! The world's end step (FUN_000e0410, each world update): while world+0xc50 bit 0 is clear it
//! runs the mode's update and, once the mode says over (its +8), sets bit 0, closes every menu
//! (the menu system's +0x1c), starts a 0.5 s fade to black (FUN_00069780, ECX 1) and sets bit 4
//! (the cutscene bit: no input, no damage, no menus). Afterwards, once the fade is done
//! (FUN_00087d70) and bit 1 is clear, it opens the mode's end menu (+0xc, h_e8e14fcf) with its
//! sequence (+0x10, h_081933d7) and sets bit 1. take12: the fade from 357.42 to 357.90 s, MISSION
//! SUCCESSFUL over black by 358.42 s (the half second between is the game saving: no wait in the
//! code). The end screen is play_endmenu.rs.
//!
//! A level-over without h_18f66101 ends the zone instead (0x207fe0): a 0.5 s fade and the
//! session's +0x1f4 (0x7ed30: the squad and the campaign saved, "Saving Campaign..." on the status
//! line for 1 s); once the fade is done its update (0x207f70) has the session play the zone's
//! post-movies and go to the zone the last zone-trigger named (its pre-movies, LOADING, the zone).
//!
//! A level's start (FUN_0007f960, the session's level start, which runs for every level load): once
//! the level is loaded and the LOADING screen's callback taken down, it starts a 1.0 s fade from
//! black (FUN_00069780(1.0) at 0x7fa2f: ECX 0, the colour a zeroed record, black). The fade is the
//! app's screen fade (FUN_000694f0 draws it each frame): a full-screen black quad of alpha 1 - k, k
//! = the app clock's seconds since the start, so the picture's values rise linearly from black in
//! one second of the game's clock, which runs under it (the world is live: the script, the AI and
//! the title card's clock all start with it). take01 (the tutorial) and take12 (m01_a) show it as a
//! line: k = 0.94 x (t - 269.69 s) and 0.855 x (t - 579.20 s) in the video's seconds, the mean luma
//! of the picture within 0.02 of it over 42 and 52 frames; against take12's watch log the picture's
//! k is 0.93 x (the app clock's seconds since 2235249 ms) + 0.03 (rms 0.02): one second of the app
//! clock, which the guest ran at 0.94-0.95 of the wall's there. #232's takes (take01-take04 of
//! todo/232-campaign-cutscenes, a gdb tap on the function) read the same call live at every level
//! start (type 0, colour 0 0 0, 1.000 s) and the clock under it (take02, m01_a: the picture's k
//! follows the app clock to 0.03), and film the scripts' fades of other colours, ways and times
//! (code 120): `Fade::start_colour` is their machinery. See `LEVEL_FADE`, `Fade::waiting` and
//! viewer/docs/mission.md "A level's start".
//!
//! What follows the end screen (FUN_00085200 by the screen's selector, which ends by starting a
//! 1 s fade to black, ECX 1; Replay Mission's FUN_00084da0 and Replay Chapter's FUN_00084e80 start
//! one of their own, Quit's FUN_00084f50 none). Only Clone Squad's fade is ever seen in the takes:
//! take12's Continue and take10's Quit cut from the picture to black in one frame (Replay isn't
//! filmed). Why a Continue's fade never shows is open. FUN_000694f0 cancels a fade while the
//! session isn't in play, and by FUN_00085200's code a Continue to a next mission queues flags
//! bits 1 and 2 (no bit 0), which 0x7f730 sends to the level start FUN_0007f960, which frees the
//! world at once; only the last mission's Continue and Quit (flags `| 9`) take bit 0, 0x7f670:
//! read from the disassembly, not traced to the picture.
//!  * Continue (the mission successful, selector 0): the zone's post-movies, then the next
//!    mission's (the campaign's next `<level>`): its pre-movies, LOADING and its first zone
//!    (take12: A at 413.4 s, mb02 from 413.8 s, then Brutus_Intro_Final, LOADING and Rescue
//!    Brutus); after the last mission, the title;
//!  * Clone Squad (the squad killed, selector 3): the mode's bit 2, whose restart (FUN_00089da0)
//!    waits for the choice's fade, then reclones the squad at its last save point (world+0xbec
//!    +0x6c) and fades from black (ECX 0, 1 s): the level stays in play, so take05 shows the end
//!    view darken linearly for 1.05 s once the menu has gone and the new squad come out of black
//!    1 s. The demo fades out the same (the menu hidden, `MissionMode::pending`), then restarts the
//!    zone behind LOADING, whose start fades in as every level's (the faithful reclone and its
//!    menu are #41's);
//!  * Replay Mission (X, after "Are you sure you want to restart the mission?"; FUN_00084da0):
//!    the mission again from its pre-movies;
//!  * Quit (B, after "Are you sure you want to quit?"; FUN_00084f50): the front end's main menu
//!    (take10: B, then A on the box, the main menu by 228 s).
//!
//! The status line: the menu system (session+0x5c4, FUN_000fdd80) keeps a queue of texts, each
//! shown for its time in turn (its +8 by string id and +0xc, 0xff280 / 0xff2c0; the item
//! FUN_000fcfd0 makes from game-options' `<system-data><console-text>`: (72, 378) 464 x 20, style
//! h_f57d9843: 153 209 251, a black shadow, the body font at 12 points). The session's saves put
//! "Saving..." (h_07246856) and "Saving Campaign..." (h_fa0f9ecf) there: take01 after ENTER NAME
//! (21.00-21.98 s), after JOIN CAMPAIGN's START (35.85-36.82 "Saving...", 36.85-37.83 "Saving
//! Campaign...", 37.87-38.38 "Saving..."), take12 as MISSION SUCCESSFUL opens (358.37-359.40,
//! 359.43-360.45 "Saving...", 360.48-361.48 "Saving Campaign..."). The demo saves nothing (#27):
//! the line only shows.
//!
//! Test hooks: BF_MISSION_LOG=1 (the mode's flags, the countdown, the fades, the end screen, what
//! follows it); the end screen's BF_END_PRESS (play_endmenu.rs); BF_NO_FADE=1 (#266: no fade from
//! black as a level starts: `begin`; a capture's first second as it was before).

use std::collections::VecDeque;

use bf_viewer::bf::hash::h;
use bf_viewer::bf::level::END_CAMERA;

use super::briefing::{self, List, Request, Then};
use super::fonts::{self, Across, BODY12};
use super::*;

/// The campaign levels' kind in the level list (campaign-bf.xmb `Type`, h_17506391): the session's
/// level kind for which FUN_00087dd0 makes the campaign mode (and the first of which a new
/// campaign starts on: play_campaign_menus.rs).
pub(crate) const LEVEL_MISSION: u32 = 0x1750_6391;
/// The squad-dead countdown (s): the campaign mode's +0x20, 3.0 (FUN_00089a50 writes 0x40400000);
/// take05: 2.94 s of clock from Tex's death to +0x24's 35.
const SQUAD_DEAD_COUNTDOWN: f32 = 3.0;
/// The level's end fade to black (s): FUN_000e0410's FUN_00069780(0.5); take12: 357.42-357.90 s.
const END_FADE: f32 = 0.5;
/// A level's start fade from black (s): FUN_0007f960's FUN_00069780(1.0) at 0x7fa2f (ECX 0: from
/// black, the colour record zeroed), run as the level's load ends. take01 and take12 fit a line
/// (0.94 and 0.855 of the video's seconds, k = 1 at 270.76 and 580.37 s); against take12's watch
/// log the picture's k is 0.93 x the app clock's seconds since the level's first clock value
/// + 0.03, rms 0.02: one second of the app clock.
const LEVEL_FADE: f32 = 1.0;
/// The fade an end screen choice starts (s): FUN_00085200's FUN_00069780(1.0) at 0x85304 (ECX 1: to
/// black), also FUN_00084da0's (0x84e6d) and FUN_00084e80's (0x84f3a). Only Clone Squad's is seen
/// (take05: the end view's mean luma falls linearly from 33.3 at 223.50 s to 0.06 at 224.55 s,
/// 1.05 s): FUN_00089da0 waits for it, and the level stays in play. take12's Continue and take10's
/// Quit (which starts none) cut to black in a frame; why a fade never shows is open (the module
/// notes).
const CHOICE_FADE: f32 = 1.0;
/// The save's stall at a successful mission's end (the end screen's opening saves its results,
/// FUN_00106d10, before the frame is drawn): the picture stays as the fade's last frame left it
/// until the end screen comes. A fit to take12, no wait in the code: the fade linear from 39.8 to
/// 3.56 mean luma by 357.90 s (0.91 of the way), held there until the end screen at 358.37 s.
/// The demo holds its fade at SAVE_AT for SAVE_STALL s.
const SAVE_AT: f32 = 0.92;
const SAVE_STALL: f32 = 0.47;
/// The game draws the fade's colour over the picture's own (gamma) values, k of the way (its draw,
/// FUN_000694f0: k x 255 as the quad's alpha, the quad's colour the fade's): take12's luma falls
/// linearly. Bevy blends in linear light, so the demo's black takes the alpha that darkens this
/// grey (an sRGB value) by k in its own values (`linear_alpha`), and any other colour the colour
/// that, with that alpha, makes a grey of GREY_COLOUR come out as the game's blend of it
/// (`quad_colour`). A fit: on the tutorial's end view the frame's mean luma then falls within 1% of linear (a power law
/// 1 - (1 - k)^2.2 left 2% at 0.92, the take 9%); on a darker picture (m01_a's start, mean luma 27)
/// it sits up to 0.07 under (docs/mission.md "A level's start"; its measured limits and #379, the
/// stored-value blend, are in "Not done").
const GREY: f32 = 0.3;

/// An sRGB value in linear light.
fn linear(v: f32) -> f32 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

/// The alpha of a black blended in linear light that darkens GREY as the game's fade by `k`
/// does (see GREY).
fn linear_alpha(k: f32) -> f32 {
    (1.0 - linear(GREY * (1.0 - k)) / linear(GREY)).clamp(0.0, 1.0)
}

/// The grey a fade's colour is fitted at (see `quad_colour`): what a fade toward a colour adds to a
/// picture, a constant energy in linear light, weighs most on its darker pixels, so this is lower
/// than GREY, which a darkening is fitted at. A fit on the demo's own frames (the tutorial's
/// opening, mean luma 47, and m01_a's, 27.6; the takes' scenes are 26-47 too): with the black's
/// alpha at GREY, the colour term solved for a picture value of 0.12 leaves the mean of a white
/// fade within 0.039 and 0.023 of the game's blend and a red one (191 0 0) within 0.014 and
/// 0.017; solved at GREY it was 0.077 and 0.116 (the whiteout arriving early).
const GREY_COLOUR: f32 = 0.12;

/// The demo's quad for the game's fade of `colour` (FUN_00069780's three ints, 0-255) at quad alpha
/// `a` (FUN_000694f0: out = (1 - a) v + a c on the picture's gamma values), blended by Bevy in
/// linear light: the alpha of `linear_alpha` (the darkening, exact for a picture value of GREY)
/// and, for each channel the colour has, the colour that makes a picture value of GREY_COLOUR
/// come out as the game's blend of it. A black quad (or a channel the colour hasn't) stays
/// black; white and red come in with the colour's own weight in the picture's values (take03's red
/// fade of the letterbox bars: 191 x a; take04's whiteout).
fn quad_colour(colour: [u8; 3], a: f32) -> Color {
    let alpha = linear_alpha(a);
    if alpha <= 1e-4 {
        return Color::NONE;
    }
    let channel = |c: u8| {
        if c == 0 {
            return 0.0;
        }
        let c = c as f32 / 255.0;
        let aim = (1.0 - a) * GREY_COLOUR + a * c;
        ((linear(aim) - (1.0 - alpha) * linear(GREY_COLOUR)) / alpha).clamp(0.0, 1.0)
    };
    Color::linear_rgba(channel(colour[0]), channel(colour[1]), channel(colour[2]), alpha)
}

/// One of BF_TEST_FADE's fades: when (the game's clock, s), its colour, whether it goes to the
/// colour (else from it) and its time (s): the arguments of FUN_00069780 (see `Fade::start_colour`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct TestFade {
    at: f32,
    colour: [u8; 3],
    out: bool,
    secs: f32,
}

/// Test hook `BF_TEST_FADE=<s>,<r>,<g>,<b>,<to|from>,<secs>[;...]` (#266): screen fades of any
/// colour, way and time, started that many seconds of the game's clock into the level (one second
/// is `from,...,0`: a script's `dur=0`), as the script controller's code 120 will start them
/// (#232): `0,255,255,255,to,0.5;0.5,255,255,255,from,0.5` is take04's whiteout, `11,191,0,0,to,1`
/// take03's red fade. A part that doesn't read is left out.
fn test_fades(spec: &str) -> Vec<TestFade> {
    let mut fades: Vec<TestFade> = spec.split(';').filter_map(|part| {
        let f: Vec<&str> = part.split(',').map(str::trim).collect();
        if f.len() != 6 {
            return None;
        }
        let out = match f[4].to_ascii_lowercase().as_str() { "to" => true, "from" => false, _ => return None };
        Some(TestFade { at: f[0].parse().ok()?, colour: [f[1].parse().ok()?, f[2].parse().ok()?, f[3].parse().ok()?], out, secs: f[5].parse().ok()? })
    }).collect();
    fades.sort_by(|a, b| a.at.total_cmp(&b.at));
    fades
}
/// A zone end's fade (s): the level-over's FUN_00069780(0.5) at 0x208092.
const ZONE_FADE: f32 = 0.5;
/// The status line's strings (the string table): "Saving..." and "Saving Campaign...".
pub const S_SAVING: u32 = 0x0724_6856;
pub const S_SAVING_CAMPAIGN: u32 = 0xFA0F_9ECF;
/// The status line's place (game-options' `<system-data><console-text>`: (72, 378), 464 x 20,
/// left and top) and colours (its style h_f57d9843: 153 209 251, shadow 0 0 0). take01's
/// "Saving..." ink from x 270 / 1.5 - 107 = 72.7 (the 4:3 picture starts at 160 of 1280).
const CONSOLE_AT: Vec2 = Vec2::new(72.0, 378.0);
const CONSOLE_COLOUR: Color = Color::srgb(153.0 / 255.0, 209.0 / 255.0, 251.0 / 255.0);
/// Above everything (the menus' 20, the end screen's 25).
const CONSOLE_Z: i32 = 200;
/// The fade's place in the order: over the world, the HUD and the letterbox (10), under the
/// panels (20) and the end screen (25), as take12 shows MISSION SUCCESSFUL over the black.
const FADE_Z: i32 = 15;

/// The flow's systems: the status line in every state; the mode, the end step and the fade in a
/// map.
pub fn plugin(app: &mut App) {
    app.init_resource::<Console>()
        .init_resource::<MissionMode>()
        .init_resource::<Fade>()
        .add_systems(Startup, spawn_console)
        .add_systems(Update, console)
        .add_systems(OnEnter(AppState::Playing), begin.after(flags::begin_level))
        .add_systems(Update, frame.after(doors).after(flags::deaths).run_if(in_state(AppState::Playing)))
        .add_systems(Update, fade.after(frame).run_if(in_state(AppState::Playing)))
        .add_plugins(endmenu::plugin);
}

/// The menu system's status line and its queue (see the module notes): the texts waiting, each
/// with its time (s), and the one shown with its time left.
#[derive(Resource, Default)]
pub struct Console {
    queue: VecDeque<(String, f32)>,
    shown: Option<(String, f32)>,
    /// the line's text node (`spawn_console`), and the state its glyphs were made in (a map's end
    /// takes the glyphs it made: they're made again)
    text: Option<Entity>,
    state: Option<AppState>,
}

impl Console {
    /// A text to show for `secs` s after those waiting (the menu system's +0xc).
    pub fn say(&mut self, text: &str, secs: f32) {
        self.queue.push_back((text.to_string(), secs));
    }
}

/// The status line's node, made at the start so that it outlives every state's own parts: a
/// window-wide root with the game's 4:3 screen in it and the line's text.
fn spawn_console(mut commands: Commands, mut console: ResMut<Console>) {
    let root = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                      justify_content: JustifyContent::Center, ..default() }, GlobalZIndex(CONSOLE_Z),
                               Name::new("status line"))).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(root))).id();
    let text = commands.spawn((fonts::record_text("", CONSOLE_AT.x, CONSOLE_AT.y, &BODY12, vec![CONSOLE_COLOUR], Some(Color::BLACK), Across::Left),
                               ChildOf(screen))).id();
    console.text = Some(text);
}

/// The status line's frame, in real time (the capture's steps under BF_CAPTURE): the text shown
/// runs out, the next takes its place; its node shows it.
fn console(time: Res<Time<Real>>, state: Res<State<AppState>>, mut console: ResMut<Console>, mut texts: Query<&mut fonts::RecordText>) {
    let dt = if std::env::var("BF_CAPTURE").is_ok() { capture_step() } else { time.delta_secs().min(0.1) };
    let c = &mut *console;
    if let Some((_, left)) = &mut c.shown {
        *left -= dt;
        if *left <= 0.0 {
            c.shown = None;
        }
    }
    if c.shown.is_none() {
        c.shown = c.queue.pop_front();
        if let (Some((text, secs)), true) = (&c.shown, std::env::var("BF_MISSION_LOG").is_ok()) {
            println!("status line: \"{text}\" for {secs:.2} s");
        }
    }
    let Some(mut t) = c.text.and_then(|e| texts.get_mut(e).ok()) else { return };
    let want = c.shown.as_ref().map_or(String::new(), |s| s.0.clone());
    // (a state's end takes the glyphs made in it with its own parts: made again in the next)
    let now = *state.get();
    if c.state != Some(now) {
        c.state = Some(now);
        t.redraw();
    }
    if t.text != want {
        t.text = want;
    }
}

/// The app's screen fade (FUN_00069780 starts one: app+0xf0 on, +0xf4 its way, +0xf8.. its colour,
/// +0x104 one over its time, +0x108 when it began; FUN_00087d70 is true while it runs): to its
/// colour (out) or from it (in), over its time; after a fade out the screen stays that colour
/// until another fade starts. A fade is a colour, a way and a time (`start_colour`; `start` is
/// the black one the level's starts and ends use): the level's start, the level's end, the end
/// screen's choices and, as the script controller's code 120 (0x1761e5: type 0 from the colour,
/// 1 to it, colour 0 black or the script's r g b ints, dur in ms with 0 for 1 s: #232), the
/// scripts'. It covers the whole frame, the letterbox's bars too (take03's red fade, take04's
/// whiteout). Its clock is the app's, the game's (0x67d00: app+0x38 as seconds, which
/// FUN_00068d20 advances by each step's time and which stands still while the game is paused:
/// `frame_dt`, the capture's steps under BF_CAPTURE).
#[derive(Resource, Default)]
pub struct Fade {
    /// how far it has come (0..1) and how fast (one over its time)
    progress: f32,
    rate: f32,
    /// to its colour (ECX 1), else from it
    out: bool,
    /// its colour (FUN_00069780's EAX record, +0xf8: three ints, 0-255)
    colour: [u8; 3],
    /// one has been started on this map
    on: bool,
    /// BF_TEST_FADE's fades still to start, by the game's clock (s), and that clock
    hook: Vec<TestFade>,
    hook_clock: f32,
    /// the level's start fade is still to begin: the screen is black (the node opaque) until the
    /// LOADING cover (play_warmup.rs) comes down, as the game's first frame is, and then
    /// `LEVEL_FADE` starts (FUN_0007f960)
    waiting: bool,
    /// the level's start fade is the one running: the game's clock and the real time (s) it has
    /// run for, for BF_MISSION_LOG's line when it's done
    level_ran: Option<(f32, f32)>,
    /// the black node over the world
    node: Option<Entity>,
    /// a save's stall to come (SAVE_AT, SAVE_STALL): the progress the picture stays at and for
    /// how long (s)
    stall: Option<(f32, f32)>,
}

impl Fade {
    /// FUN_00069780(secs) with ECX `out` and the colour record zeroed: black.
    pub fn start(&mut self, secs: f32, out: bool) {
        self.start_colour([0, 0, 0], secs, out);
    }

    /// FUN_00069780 itself: a fade to (`out`, ECX 1) or from (ECX 0) `colour` (EAX: three ints,
    /// 0-255) over `secs` (the float argument; under 1.19e-7 it is one second, +0x104 = 1.0: a
    /// script's `dur=0`), starting now and replacing the fade under way (a script's own at a
    /// level's start replaces the level's: e34's 2.0 s).
    pub fn start_colour(&mut self, colour: [u8; 3], secs: f32, out: bool) {
        self.on = true;
        self.waiting = false;
        self.level_ran = None;
        self.out = out;
        self.colour = colour;
        self.progress = 0.0;
        self.rate = if secs > 1.192_092_9e-7 { 1.0 / secs } else { 1.0 };
        self.stall = None;
    }

    /// FUN_00069780(secs) with ECX `out`, then the game stalled by a save once it's `at` of the
    /// way, for `secs_stalled` (SAVE_AT): the picture stays there, then the fade is over.
    pub fn start_stalled(&mut self, secs: f32, out: bool, at: f32, secs_stalled: f32) {
        self.start(secs, out);
        self.stall = Some((at, secs_stalled));
    }

    /// FUN_00087d70: a fade is under way (through a stall too).
    pub fn running(&self) -> bool {
        self.on && self.progress < 1.0
    }

    /// The quad's colour and alpha now in the demo's blend (`quad_colour`).
    fn quad(&self) -> Color {
        quad_colour(self.colour, self.alpha())
    }

    /// The fade's alpha now, as the game draws it over the picture's own values (see GREY): 1 (a
    /// whole black) while the level start waits for the cover.
    fn alpha(&self) -> f32 {
        if self.waiting {
            return 1.0;
        }
        if !self.on {
            return 0.0;
        }
        let k = self.progress.clamp(0.0, 1.0);
        if self.out { k } else { 1.0 - k }
    }

    /// One frame of the fade's clock: on to the end, or to its stall and through it.
    fn step(&mut self, dt: f32) {
        if !self.on || self.progress >= 1.0 {
            return;
        }
        match self.stall {
            Some((at, left)) if self.progress >= at => {
                // (stalled: the picture stays, then the fade's time is past its end)
                if left - dt <= 0.0 {
                    self.stall = None;
                    self.progress = 1.0;
                } else {
                    self.stall = Some((at, left - dt));
                }
            }
            Some((at, _)) => self.progress = (self.progress + dt * self.rate).min(at),
            None => self.progress += dt * self.rate,
        }
    }
}

/// The campaign game mode of a mission level and the world's end step (see the module notes),
/// made anew with each level (`begin`).
#[derive(Resource, Default)]
pub struct MissionMode {
    /// a mission (a multiplayer map has the deathmatch modes: none of this runs)
    pub(crate) active: bool,
    /// +0x24 bit 0: the level is over
    pub(crate) over: bool,
    /// +0x24 bit 1: the squad is dead (the countdown ran out)
    pub(crate) squad_dead: bool,
    /// +0x24 bit 4: the countdown hasn't run yet (its first frame clears it)
    fresh: bool,
    /// +0x20: the countdown (s)
    countdown: f32,
    /// world+0xc50 bit 0: the end step has taken the level's end (the fade started)
    ended: bool,
    /// world+0xc50 bit 1: the end menu opened (play_endmenu.rs builds it)
    pub(crate) menu_open: bool,
    /// a zone's end under way (a level-over without h_18f66101): waiting for its fade, then the
    /// zone it goes to
    zone_end: Option<String>,
    /// an end screen choice waiting for its fade to black to finish (Clone Squad's: FUN_00089da0
    /// returns until FUN_00087d70 says the fade is done), then what it leads to (`go`)
    pub(crate) pending: Option<Go>,
    /// the mission and the zone of this level (`<level h_046391fd>` and its zone's index)
    pub(crate) mission: u32,
    pub(crate) zone: usize,
    /// the end camera the end screen cuts to (one of the level's END_CAMERA camera-objects)
    pub(crate) end_camera: Option<u32>,
    /// BF_MISSION_LOG
    pub(crate) log: bool,
    /// the level's clock for the log (the game's, which runs on while the player lies dead)
    clock: f32,
}

/// A map starts: the campaign mode made anew (FUN_000d6490) in a mission, its mission and zone
/// known by the level's file; the last fade gone and the level's start fade to come (FUN_0007f960,
/// `LEVEL_FADE`: the screen is black until the LOADING cover goes). Not in the map editor's edit
/// mode (not the game's), nor with BF_NO_FADE set (a test hook: the first second of a capture
/// without the fade).
#[allow(clippy::too_many_arguments)]
fn begin(mut mode: ResMut<MissionMode>, mut fade: ResMut<Fade>, mut commands: Commands, game: Res<GameData>, map: Res<MapLevel>,
         current: Res<CurrentMap>, campaign: Res<flags::Campaign>, editing: Option<Res<State<super::editor::EditorMode>>>) {
    let log = std::env::var("BF_MISSION_LOG").is_ok();
    let waiting = std::env::var("BF_NO_FADE").is_err() && !editing.is_some_and(|m| *m.get() == super::editor::EditorMode::Edit);
    let (mission, zone) = game.0.campaign.iter().filter(|l| l.kind == LEVEL_MISSION)
        .find_map(|l| l.zones.iter().position(|z| *z == current.0.to_lowercase()).map(|z| (l.id, z))).unwrap_or((0, 0));
    // FUN_001092e0's pick among the level's end cameras: the game's random state (0x46cfd8) isn't
    // the demo's, so the first (the tutorial has one, e01 four)
    let end_camera = map.0.as_ref().and_then(|l| l.camera_objects.iter().find(|c| c.kind == END_CAMERA)).map(|c| c.name);
    *mode = MissionMode { active: campaign.in_mission, fresh: true, countdown: SQUAD_DEAD_COUNTDOWN, mission, zone, end_camera, log, ..default() };
    // the fade's black, over the world (a map's end takes it with the map's parts): whole black
    // from the first frame when the level's start fade is to come
    let node = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                               BackgroundColor(if waiting { Color::BLACK } else { Color::NONE }), GlobalZIndex(FADE_Z), Name::new("screen fade"))).id();
    let hook = std::env::var("BF_TEST_FADE").map(|v| test_fades(&v)).unwrap_or_default();
    *fade = Fade { node: Some(node), waiting, hook, ..default() };
    if log {
        println!("mission flow: {} (mission h_{mission:08x} zone {zone}), {}; end camera {}{}", current.0,
                 if campaign.in_mission { "the campaign mode" } else { "not a mission: no flow" },
                 end_camera.map_or("none".to_string(), |c| format!("h_{c:08x}")),
                 if waiting { format!("; the level's start fade ({LEVEL_FADE} s from black) waits for the LOADING cover") } else { String::new() });
    }
}

/// The flow's frame (after the triggers', the doors' and play_flags.rs's count of the deaths):
/// what the level-over and zone triggers asked, the campaign mode's update (the squad-dead
/// countdown) and the world's end step (see the module notes).
#[allow(clippy::too_many_arguments)]
fn frame(mut commands: Commands, time: Res<Time>, mut mode: ResMut<MissionMode>, mut tr: ResMut<triggers::Triggers>, player: Res<Player>,
         squad: Res<Squad>, mut fade: ResMut<Fade>, mut console: ResMut<Console>, mut objectives: ResMut<objectives::ObjectivesMenu>,
         game: Res<GameData>, mut next: ResMut<NextState<AppState>>, campaign: Res<flags::Campaign>, current: Res<CurrentMap>) {
    if !mode.active {
        return;
    }
    // an end screen choice that waits for its fade (Clone Squad: the mode's restart, FUN_00089da0,
    // returns while the fade runs): once it's done, what the choice leads to
    if let (Some(to), false) = (mode.pending, fade.running()) {
        mode.pending = None;
        go(&mut commands, &mut next, &game.0, &mode, &current.0, to);
        return;
    }
    let dt = frame_dt(&time);
    mode.clock += dt;
    let t = mode.clock;
    let log = mode.log;
    let flow = tr.take_flow();
    if flow.over && !mode.over {
        // the game mode's +0x14 (0x89d20)
        mode.over = true;
        if log {
            println!("t {t:.2}: mission flow: the level-over: the mode is over (+0x24 bit 0)");
        }
    }
    if flow.zone_end && mode.zone_end.is_none() {
        // 0x207fe0's false path: the fade, then the session's +0x1f4 (the checkpoint's save and
        // "Saving Campaign..." for 1 s, 0x7ed30)
        fade.start(ZONE_FADE, true);
        console.say(&string(&game.0, S_SAVING_CAMPAIGN), 1.0);
        mode.zone_end = Some(flow.next_zone.clone().unwrap_or_default());
        if log {
            println!("t {t:.2}: mission flow: the zone's end: a {ZONE_FADE} s fade, \"Saving Campaign...\", then {}",
                     flow.next_zone.as_deref().unwrap_or("no zone (none named)"));
        }
    }
    // the zone's end, its fade done (0x207f70): the zone's post-movies, then the next zone
    if let (Some(next_zone), false) = (mode.zone_end.clone(), fade.running()) {
        mode.zone_end = None;
        mode.active = false;
        let to = game.0.campaign.iter().filter(|l| l.kind == LEVEL_MISSION).find_map(|l| l.zones.iter().position(|z| *z == next_zone).map(|z| (l.id, z)));
        let then = match to {
            Some((m, z)) => Then::List(Box::new(Request { mission: m, zone: z, list: List::Pre, then: Then::Zone(next_zone.clone()) })),
            None => Then::Menu,
        };
        if log {
            println!("t {t:.2}: mission flow: the zone's post-movies, then {}", if to.is_some() { next_zone.as_str() } else { "the title (no zone named)" });
        }
        briefing::start(&mut commands, &mut next, mode.mission, mode.zone, List::Post, then);
        return;
    }
    if !mode.ended {
        // the mode's update (FUN_00089c90): the squad-dead countdown (FUN_00089d30), while none
        // of the squad record's members lives (FUN_0005baf0: the joined marines count) and its
        // +0x20 has a death counted (outside a cutscene, `Cash::lost`)
        if !mode.over {
            let living = std::iter::once(&*player).chain(squad.0.iter()).filter(|u| campaign.in_squad_record(&game.0, u) && !u.dead).count();
            if living == 0 && campaign.cash.lost > 0 && !script::in_cutscene() {
                if mode.fresh {
                    // (bit 4 cleared; its FUN_00179e10, the world's music stopped, isn't done)
                    mode.fresh = false;
                    if log {
                        println!("t {t:.2}: mission flow: the squad is dead: a {SQUAD_DEAD_COUNTDOWN} s countdown (+0x24 bit 4 cleared)");
                    }
                }
                mode.countdown -= dt;
                if mode.countdown <= 0.0 {
                    mode.squad_dead = true;
                    mode.over = true;
                    if log {
                        println!("t {t:.2}: mission flow: the countdown ran out: the squad is dead and the level over (+0x24 bits 1 and 0)");
                    }
                }
            }
        }
        // FUN_000e0410: over, the end taken: the menus closed, the fade, the cutscene bit; a
        // successful mission's results are saved as the end screen opens (FUN_00106d10): the
        // picture stalls near black first (SAVE_AT)
        if mode.over {
            mode.ended = true;
            objectives.close = true;
            let saves = succeeded(&mode, &campaign);
            if saves {
                fade.start_stalled(END_FADE, true, SAVE_AT, SAVE_STALL);
            } else {
                fade.start(END_FADE, true);
            }
            script::set_cutscene(true);
            if log {
                println!("t {t:.2}: mission flow: the level's end: the menus closed, a {END_FADE} s fade to black{}, the cutscene bit (world+0xc50 bits 0 and 4)",
                         if saves { format!(", the save's stall at {SAVE_AT} of it for {SAVE_STALL} s") } else { String::new() });
            }
        }
    } else if !mode.menu_open && !fade.running() {
        // the fade done: the end menu (h_e8e14fcf, play_endmenu.rs)
        mode.menu_open = true;
        if log {
            println!("t {t:.2}: mission flow: the fade is done: the end screen opens (world+0xc50 bit 1)");
        }
    }
}

/// Whether the mission ended in success (FUN_00107e50's selector 0): not the squad's death, and
/// every enabled primary objective complete (FUN_00107dd0: `state` > 0).
pub(crate) fn succeeded(mode: &MissionMode, campaign: &flags::Campaign) -> bool {
    !mode.squad_dead && campaign.objectives.iter().filter(|o| o.primary).all(|o| {
        let p = |k: &str| [o.path.as_slice(), &[h(k)]].concat();
        !campaign.flags.value(&p("enabled")).is_some_and(|v| v.as_bool()) || campaign.flags.value(&p("state")).is_some_and(|v| v.as_int() > 0)
    })
}

/// The fade's frame: the level's start fade begun as the LOADING cover goes (FUN_0007f960 runs it as
/// the load ends; the game's clock is held behind the cover, `PAUSE_LOADING`, so the cover's wait
/// doesn't use it up), BF_TEST_FADE's fades when their time comes, its clock (the app's: the game's,
/// `frame_dt`, the capture's steps), its quad's colour and alpha (in linear light, the game's blend
/// on the picture's own values: GREY, `quad_colour`). BF_MISSION_LOG prints when the level's start
/// fade begins and when it's done, with the game's clock and the real seconds it ran (the real
/// time is the machine's under BF_CAPTURE), and each test fade as it starts.
fn fade(time: Res<Time>, real: Res<Time<Real>>, mode: Res<MissionMode>, mut fade: ResMut<Fade>, mut nodes: Query<&mut BackgroundColor>) {
    if fade.waiting && !loading_cover() {
        fade.start(LEVEL_FADE, false);
        fade.level_ran = Some((0.0, 0.0));
        if mode.log {
            println!("level start: the LOADING cover is down: a {LEVEL_FADE} s fade from black begins");
        }
    }
    let dt = frame_dt(&time);
    fade.hook_clock += dt;
    while fade.hook.first().is_some_and(|f| f.at <= fade.hook_clock) {
        let f = fade.hook.remove(0);
        fade.start_colour(f.colour, f.secs, f.out);
        if mode.log {
            println!("test fade at {:.2} s: {} {:?} over {} s", fade.hook_clock, if f.out { "to" } else { "from" }, f.colour, f.secs);
        }
    }
    fade.step(dt);
    if let Some((clock, wall)) = fade.level_ran.as_mut() {
        *clock += dt;
        *wall += real.delta_secs();
    }
    if let (Some((clock, wall)), false) = (fade.level_ran, fade.running()) {
        fade.level_ran = None;
        if mode.log {
            println!("level start: the fade is done after {clock:.2} s of the game's clock ({wall:.2} s of real time)");
        }
    }
    let want = fade.quad();
    if let Some(mut c) = fade.node.and_then(|n| nodes.get_mut(n).ok()) {
        if c.0 != want {
            c.0 = want;
        }
    }
}

/// A string of the string table (empty if it isn't there).
pub(crate) fn string(game: &bf_viewer::bf::character::Game, id: u32) -> String {
    game.strings.get(&id).cloned().unwrap_or_default()
}

/// What the end screen's choices lead to (FUN_00085200, FUN_00084da0, FUN_00084f50).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Go {
    /// Continue: the zone's post-movies, the next mission's pre-movies, its first zone
    Continue,
    /// Clone Squad: the zone again (the demo's stand-in for the reclone, #41), after the choice's
    /// 1 s fade to black (`CHOICE_FADE`, `MissionMode::pending`)
    CloneSquad,
    /// Replay Chapter: the zone again from its pre-movies
    ReplayChapter,
    /// Replay Mission: the mission again from its first zone's pre-movies
    ReplayMission,
    /// Quit: the front end's main menu
    Quit,
}

/// Leave the level for what an end screen choice leads to (see Go): the movies, the LOADING screen
/// and the zone, or the front end. The map goes as its state ends (play.rs `end_play`).
pub fn go(commands: &mut Commands, next: &mut NextState<AppState>, game: &bf_viewer::bf::character::Game, mode: &MissionMode, current: &str, to: Go) {
    let missions: Vec<_> = game.campaign.iter().filter(|l| l.kind == LEVEL_MISSION).collect();
    let here = missions.iter().position(|l| l.id == mode.mission);
    if mode.log {
        println!("mission flow: {to:?} from {current} (mission h_{:08x} zone {})", mode.mission, mode.zone);
    }
    match to {
        Go::Continue => {
            // the zone's post-movies, then the next mission's; after the last, the title
            match here.and_then(|k| missions.get(k + 1)) {
                Some(m) => {
                    let first = m.zones.first().cloned().unwrap_or_default();
                    let then = Then::List(Box::new(Request { mission: m.id, zone: 0, list: List::Pre, then: Then::Zone(first) }));
                    briefing::start(commands, next, mode.mission, mode.zone, List::Post, then);
                }
                None => briefing::start(commands, next, mode.mission, mode.zone, List::Post, Then::Menu),
            }
        }
        Go::CloneSquad => menu::load_next(commands, next, current),
        Go::ReplayChapter => briefing::start(commands, next, mode.mission, mode.zone, List::Pre, Then::Zone(current.to_string())),
        Go::ReplayMission => {
            let first = here.and_then(|k| missions[k].zones.first().cloned()).unwrap_or_else(|| current.to_string());
            briefing::start(commands, next, mode.mission, 0, List::Pre, Then::Zone(first));
        }
        Go::Quit => {
            // FUN_00084f50: the session's end, the front end (the main menu, take10)
            commands.insert_resource(menu::Return { map: None, deathmatch: false, main: true, editor: false });
            next.set(AppState::Menu);
        }
    }
}

/// The end screen (h_e8e14fcf): its drawing, counting, pages and choices.
#[path = "play_endmenu.rs"]
pub(crate) mod endmenu;

#[cfg(test)]
mod tests {
    use super::*;

    /// A level's start: black while the cover is up (no fade under way for FUN_00087d70), then
    /// LEVEL_FADE from black to clear, linear in the game's values (FUN_000694f0: the quad's alpha
    /// 1 - k), and the fade is over after it.
    #[test]
    fn level_start_fades_from_black_in_a_second() {
        let mut f = Fade { waiting: true, ..default() };
        assert_eq!(f.alpha(), 1.0);
        assert!(!f.running());
        f.step(0.5);
        assert_eq!(f.alpha(), 1.0, "the clock doesn't run it while the cover waits");
        f.start(LEVEL_FADE, false);
        assert!(f.running());
        assert_eq!(f.alpha(), 1.0);
        for (i, want) in [0.75, 0.5, 0.25].into_iter().enumerate() {
            f.step(0.25);
            assert!((f.alpha() - want).abs() < 1e-5, "step {i}: {}", f.alpha());
            assert!(f.running());
        }
        f.step(0.25);
        assert_eq!(f.alpha(), 0.0);
        assert!(!f.running());
        f.step(1.0);
        assert_eq!(f.alpha(), 0.0, "a fade in ends clear");
    }

    /// A choice's fade to black rises linearly to black and stays there (a fade out is not turned
    /// off at its end: FUN_000694f0), done after CHOICE_FADE.
    #[test]
    fn choice_fade_goes_to_black_and_stays() {
        let mut f = Fade::default();
        assert_eq!(f.alpha(), 0.0);
        f.start(CHOICE_FADE, true);
        f.step(0.5);
        assert!((f.alpha() - 0.5).abs() < 1e-5);
        assert!(f.running());
        f.step(0.5);
        assert_eq!(f.alpha(), 1.0);
        assert!(!f.running());
        f.step(3.0);
        assert_eq!(f.alpha(), 1.0);
    }

    /// The linear-light alpha that gives the game's darkening: none at 0, whole at 1, and a grey
    /// of GREY goes to k of itself in the picture's values.
    #[test]
    fn blend_alpha_darkens_the_grey_linearly() {
        assert_eq!(linear_alpha(0.0), 0.0);
        assert_eq!(linear_alpha(1.0), 1.0);
        for k in [0.25, 0.5, 0.75] {
            let a = linear_alpha(k);
            let shown = 1.055 * (linear(GREY) * (1.0 - a)).powf(1.0 / 2.4) - 0.055;
            assert!((shown - GREY * (1.0 - k)).abs() < 1e-3, "k {k}: {shown}");
        }
    }

    /// An sRGB value from linear light (the test's way of looking at the blend's result).
    fn encoded(l: f32) -> f32 {
        if l <= 0.003_130_8 { l * 12.92 } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 }
    }

    /// The quad of any colour makes a picture value of GREY_COLOUR come out as the game's blend of
    /// it, (1 - a) v + a c, in each channel the colour has, and one of GREY darken as the game's
    /// does in the channels it hasn't, once Bevy blends it in linear light: black, white (take04's
    /// whiteout) and red 191 0 0 (take03's, the letterbox bars going red as 191 x a).
    #[test]
    fn quad_colours_blend_the_grey_as_the_game_does() {
        for colour in [[0u8, 0, 0], [255, 255, 255], [191, 0, 0]] {
            for a in [0.1, 0.25, 0.5, 0.75, 0.9] {
                let Color::LinearRgba(q) = quad_colour(colour, a) else { panic!("a linear colour") };
                let channels = [q.red, q.green, q.blue];
                for ch in 0..3 {
                    let c = colour[ch] as f32 / 255.0;
                    let (v, want) = if colour[ch] == 0 { (GREY, (1.0 - a) * GREY) } else { (GREY_COLOUR, (1.0 - a) * GREY_COLOUR + a * c) };
                    let out = encoded((1.0 - q.alpha) * linear(v) + q.alpha * channels[ch]);
                    assert!((out - want).abs() < 2e-3, "{colour:?} a {a} channel {ch}: {out} against {want}");
                }
            }
        }
        // (nothing to draw at alpha 0; a black quad has no colour)
        assert_eq!(quad_colour([255, 0, 0], 0.0), Color::NONE);
        let Color::LinearRgba(q) = quad_colour([0, 0, 0], 0.5) else { panic!("a linear colour") };
        assert_eq!((q.red, q.green, q.blue), (0.0, 0.0, 0.0));
    }

    /// A fade of a colour, a way and a time (FUN_00069780's arguments): from white in half a second,
    /// then a time of 0 is one second (a script's dur=0, rate 1.0), and a fade to a colour stays that
    /// colour.
    #[test]
    fn a_fade_has_a_colour_a_way_and_a_time() {
        let mut f = Fade::default();
        f.start_colour([255, 255, 255], 0.5, false);
        assert_eq!(f.colour, [255, 255, 255]);
        assert_eq!(f.alpha(), 1.0);
        f.step(0.25);
        assert!((f.alpha() - 0.5).abs() < 1e-5);
        f.step(0.25);
        assert_eq!(f.alpha(), 0.0);
        assert!(!f.running());
        f.start_colour([191, 0, 0], 0.0, true);
        f.step(0.5);
        assert!((f.alpha() - 0.5).abs() < 1e-5, "dur 0 is one second");
        f.step(2.0);
        assert_eq!(f.alpha(), 1.0, "a fade to the colour stays");
        assert!(!f.running());
        // (another fade replaces the one under way, the level's start waiting included)
        let mut g = Fade { waiting: true, ..default() };
        g.start_colour([0, 0, 0], 2.0, false);
        assert!(!g.waiting && g.running());
    }

    /// BF_TEST_FADE's list: the fades by their time, a part that doesn't read left out.
    #[test]
    fn test_fade_list_reads_the_whiteout() {
        let f = test_fades("0.5,255,255,255,from,0.5; 0,255,255,255,to,0.5;bad;1,2,3,sideways,1");
        assert_eq!(f, vec![TestFade { at: 0.0, colour: [255, 255, 255], out: true, secs: 0.5 },
                           TestFade { at: 0.5, colour: [255, 255, 255], out: false, secs: 0.5 }]);
        assert!(test_fades("").is_empty());
    }
}
