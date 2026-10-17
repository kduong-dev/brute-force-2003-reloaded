//! The mission flow (#226, step 7 of #28; viewer/docs/mission.md): how a mission ends and what
//! follows, as default.xbe's campaign game mode, the world's end step and the session run it.
//! Written from the code (the addresses are in the comments and in viewer/docs/code-map.md);
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
//! What follows the end screen (FUN_00085200 by the screen's selector, then a 1 s fade, ECX 1):
//!  * Continue (the mission successful, selector 0): the zone's post-movies, then the next
//!    mission's (the campaign's next `<level>`): its pre-movies, LOADING and its first zone
//!    (take12: A at 413.4 s, mb02 from 413.8 s, then Brutus_Intro_Final, LOADING and Rescue
//!    Brutus); after the last mission, the title;
//!  * Clone Squad (the squad killed, selector 3): the mode's bit 2, whose restart (FUN_00089da0)
//!    reclones the squad at its last save point (world+0xbec +0x6c): the demo restarts the zone
//!    instead, behind LOADING (the faithful reclone and its menu are #41's);
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
//! follows it); the end screen's BF_END_PRESS (play_endmenu.rs).

use std::collections::VecDeque;

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
        .add_systems(Update, frame.after(doors).run_if(in_state(AppState::Playing)))
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
/// +0x104 one over its time, +0x108 when it began; FUN_00087d70 is true while it runs): to black
/// (out) or from it (in), over its time; after a fade out the screen stays black until a fade in.
/// Its clock is the app's (FUN_000cd4c0's +0x44: real time, the capture's steps under BF_CAPTURE).
#[derive(Resource, Default)]
pub struct Fade {
    /// how far it has come (0..1) and how fast (one over its time)
    progress: f32,
    rate: f32,
    /// to black (ECX 1), else from it
    out: bool,
    /// one has been started on this map
    on: bool,
    /// the black node over the world
    node: Option<Entity>,
}

impl Fade {
    /// FUN_00069780(secs) with ECX `out`.
    pub fn start(&mut self, secs: f32, out: bool) {
        self.on = true;
        self.out = out;
        self.progress = 0.0;
        self.rate = if secs > 1.192_092_9e-7 { 1.0 / secs } else { 1.0 };
    }

    /// FUN_00087d70: a fade is under way.
    pub fn running(&self) -> bool {
        self.on && self.progress < 1.0
    }

    /// The black's alpha now.
    fn alpha(&self) -> f32 {
        if !self.on {
            return 0.0;
        }
        let k = self.progress.clamp(0.0, 1.0);
        if self.out { k } else { 1.0 - k }
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
/// known by the level's file; the fade gone.
fn begin(mut mode: ResMut<MissionMode>, mut fade: ResMut<Fade>, mut commands: Commands, game: Res<GameData>, map: Res<MapLevel>,
         current: Res<CurrentMap>, campaign: Res<flags::Campaign>) {
    let log = std::env::var("BF_MISSION_LOG").is_ok();
    let (mission, zone) = game.0.campaign.iter().filter(|l| l.kind == LEVEL_MISSION)
        .find_map(|l| l.zones.iter().position(|z| *z == current.0.to_lowercase()).map(|z| (l.id, z))).unwrap_or((0, 0));
    // FUN_001092e0's pick among the level's end cameras: the game's random state (0x46cfd8) isn't
    // the demo's, so the first (the tutorial has one, e01 four)
    let end_camera = map.0.as_ref().and_then(|l| l.camera_objects.iter().find(|c| c.kind == END_CAMERA)).map(|c| c.name);
    *mode = MissionMode { active: campaign.in_mission, fresh: true, countdown: SQUAD_DEAD_COUNTDOWN, mission, zone, end_camera, log, ..default() };
    // the fade's black, over the world (a map's end takes it with the map's parts)
    let node = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                               BackgroundColor(Color::NONE), GlobalZIndex(FADE_Z), Name::new("screen fade"))).id();
    *fade = Fade { node: Some(node), ..default() };
    if log {
        println!("mission flow: {} (mission h_{mission:08x} zone {zone}), {}; end camera {}", current.0,
                 if campaign.in_mission { "the campaign mode" } else { "not a mission: no flow" },
                 end_camera.map_or("none".to_string(), |c| format!("h_{c:08x}")));
    }
}

/// The flow's frame (after the triggers' and the doors'): what the level-over and zone triggers
/// asked, the campaign mode's update (the squad-dead countdown) and the world's end step (see
/// the module notes).
#[allow(clippy::too_many_arguments)]
fn frame(mut commands: Commands, time: Res<Time>, mut mode: ResMut<MissionMode>, mut tr: ResMut<triggers::Triggers>, player: Res<Player>,
         squad: Res<Squad>, mut fade: ResMut<Fade>, mut console: ResMut<Console>, mut objectives: ResMut<objectives::ObjectivesMenu>,
         game: Res<GameData>, mut next: ResMut<NextState<AppState>>) {
    if !mode.active {
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
        // the mode's update (FUN_00089c90): the squad-dead countdown (FUN_00089d30)
        if !mode.over {
            let members: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).filter(|u| u.in_squad() && !u.absent && !u.stand_in).collect();
            let living = members.iter().filter(|u| !u.dead).count();
            if !members.is_empty() && living == 0 && !script::in_cutscene() {
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
        // FUN_000e0410: over, the end taken: the menus closed, the fade, the cutscene bit
        if mode.over {
            mode.ended = true;
            objectives.close = true;
            fade.start(END_FADE, true);
            script::set_cutscene(true);
            if log {
                println!("t {t:.2}: mission flow: the level's end: the menus closed, a {END_FADE} s fade to black, the cutscene bit (world+0xc50 bits 0 and 4)");
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

/// The fade's frame: its clock (the app's: real time, the capture's steps), its black's alpha.
fn fade(time: Res<Time<Real>>, mut fade: ResMut<Fade>, mut nodes: Query<&mut BackgroundColor>) {
    let dt = if std::env::var("BF_CAPTURE").is_ok() { capture_step() } else { time.delta_secs().min(0.1) };
    if fade.on && fade.progress < 1.0 {
        fade.progress += dt * fade.rate;
    }
    let alpha = fade.alpha();
    if let Some(mut c) = fade.node.and_then(|n| nodes.get_mut(n).ok()) {
        let want = Color::BLACK.with_alpha(alpha);
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
    /// Clone Squad: the zone again (the demo's stand-in for the reclone, #41)
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
            commands.insert_resource(menu::Return { map: None, deathmatch: false, main: true });
            next.set(AppState::Menu);
        }
    }
}

/// The end screen (h_e8e14fcf): its drawing, counting, pages and choices.
#[path = "play_endmenu.rs"]
pub(crate) mod endmenu;
