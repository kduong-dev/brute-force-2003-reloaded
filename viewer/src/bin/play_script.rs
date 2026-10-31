//! The level's cutscenes (#222, the tutorial's step 3 of #28; viewer/docs/cutscenes.md): the
//! .gscr scripts (bf/gscr.rs) played as default.xbe's script player plays them, their cameras,
//! the letterbox and the holds on the characters. Written from the code (the addresses are in
//! the comments and in viewer/docs/code-map.md, "Cutscenes"); the HUD's hiding is a fit.
//!
//! The player. Every script of the level has its own player (FUN_00174180 registers one per
//! scripts/*.gscr at load, FUN_00174460). A script-trigger's OPERATE asks it to start
//! (FUN_00204ec0: its entry's `<script command>` 0 or 1 plays it from the start, 2 pauses, 3
//! resumes, 4 stops); the start itself happens in the player's next update (FUN_00174930 ->
//! FUN_00174b30), so two OPERATEs in one frame (the opening's two FIRST_UPDATEs, play_triggers.rs)
//! start it once. The start holds every character the script names (FUN_00174e10). Each update
//! advances its clock by the frame's milliseconds (FUN_00174270) and runs the events that are due,
//! in order; the running path animations follow (the list at +0x98); at `duration` it stops
//! (FUN_00174bd0), releasing the characters it held (FUN_00174e70). An event whose signal asks
//! something of the world (a 103 to a spawn- or warp-trigger) has it done before the next event
//! (`step`'s passes), as the game's signals act at once.
//!
//! The events (`gscr::EventKind`). USER_EVENT goes to its entity's controller (FUN_00175090 binds
//! the entities by name: a MARKER "trig" to the level trigger, "script" to the script itself, a
//! CAMERA to the level's camera-object, a COMPOUND to the character or spawn copy, a MOTION_PATH
//! to the level's path-object; a spawn copy is bound as it appears, FUN_002096e0; one not in the
//! world isn't bound, and its events are lost):
//!  * 103 on a trigger: its signal straight to the trigger's handler, from iD 0, no subject, data
//!    -1 (0x1772b0);
//!  * 101 on a camera with 0: the view cuts to it (FUN_00173270: the view's +0x8c, its cut flag);
//!    with 1-3 on a camera or the script itself: back to the game's camera (FUN_00072f00: the
//!    follow camera snaps). Only in a mission (the world's +0xbec +0x18);
//!  * 119 on a character: the cutscene flags (0x17206d, the fields FUN_001753f0 reads): the hold
//!    (char+0x834 bit 0: its AI and its player's controls off, FUN_00129470), the scripted
//!    movement (bits 0 and 1, FUN_001294a0) or the release (FUN_001294c0); a held character is
//!    released when the script stops. The tutorial's 119s all hold;
//!  * 105 on a character: it walks to the listed world points, held (0x171f7a; a straight walk
//!    here, the ticket's stub);
//!  * 100 on a character: a command to its command handler (0x1722cc: vtable +0x15c), the weapon
//!    switch or the reload (#281: the openings' guns in hand, `command_100`; the crouch's toggle
//!    is logged);
//!  * 122 / 123 on the script: the world's cutscene bit on / off (0x176227 / 0x176258: world+0xc50
//!    bit 4): while it's on no input reaches the game (no menu opens, FUN_0008c8c0; the player's
//!    character doesn't move), nobody takes damage (the damage handler 0x125600) and the squad's
//!    chatter is quiet (FUN_00140a60, not done here);
//!  * every other code (104, 110, 111, 113-115, 120: #232) is logged, not run.
//! START_PATH_ANIMATION moves its first entity (a camera here) along its second (a path) over its
//! `duration` (the event class at 0x392520: 0x275290 each frame): see `PathCurve`.
//!
//! The letterbox (the view's +0x78..+0x80, FUN_00073460 / FUN_000734b0, its step 0x733ee): the
//! letterbox-trigger's OPERATE brings the bars in over its show-duration, its STOP takes them out
//! over its hide-duration, linearly; at full they cover 145.83 / 1000 of the screen's height at the
//! top and at the bottom (FUN_00073110: the 3D view's viewport cropped; take03's frames at 26.0
//! and 26.8 s show the same picture under the bars, so bars over the full view are the same).
//!
//! The squad's farcast. A squad member a spawn-trigger spawns isn't in the world until then (its
//! template is taken out of it as the trigger links, FUN_000dfe90): `Player::absent` (play.rs
//! `player_at_start`), until `spawn_copy` (play_triggers.rs) brings it in. The tutorial's Tex comes
//! with the opening's farcast at 17.09 s (take03's watch log: clk 17102).
//!
//! Test hooks:
//!  * BF_SCRIPT=<name>[,<s>]: plays that script at that time (sim s, default 0), as a script-trigger
//!    would. Then (and with BF_TEST_GOTO) the level's start doesn't play its scripts (the opening,
//!    which would hold the player for its 26 s) and the squad is in the world from the start, as
//!    before #222. BF_SCRIPT=off: no script plays at all.
//!  * BF_SCRIPT_LOG=1: each script's start and end, every event as it runs (the camera cuts, the
//!    paths, the holds, the signals sent), the letterbox and the cutscene bit.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bf_viewer::bf::gscr::{self, flags, EntityKind, EventKind, PathAnimation, Script, UserEvent};
use bf_viewer::bf::hash::h;
use bf_viewer::bf::triggers::sig;

use super::triggers::Triggers;
use super::*;

/// The cutscenes' state, a new one per map (`start`); their frame is the trigger runtime's
/// (play_triggers.rs `run` calls `step` after the triggers' updates and what they asked of the
/// world, so a script a trigger starts plays its first events in that frame, and what its events
/// send reaches the triggers at once); the view (the camera, the bars, the HUD) after the follow
/// camera (`view`).
pub fn plugin(app: &mut App) {
    app.init_resource::<Cutscenes>()
        .add_systems(OnEnter(AppState::Playing), start.after(setup))
        .add_systems(OnExit(AppState::Playing), finish)
        .add_systems(Update, view.after(update_hud).run_if(in_state(AppState::Playing)));
}

/// The world's cutscene bit (world+0xc50 bit 4: set by a script's 122, cleared by its 123,
/// 0x176227 / 0x176258). take03's watch log: 137456 from the level's start, 137440 at clk 26188
/// (the opening's 123 at 26.176 s). Read by the systems the game's readers stand for
/// (`in_cutscene`).
static CUTSCENE: AtomicBool = AtomicBool::new(false);

/// Whether a cutscene has the world (see CUTSCENE): no menu opens (FUN_0008c8c0 opens none of the
/// players' menus while the bit is set, the objectives menu among them), the player's controls
/// are off, nobody is hurt (the damage handler 0x125600's early exit).
pub fn in_cutscene() -> bool {
    CUTSCENE.load(Ordering::Relaxed)
}

/// Sets or clears the world's cutscene bit from outside a script (#226): the level's end sets it
/// as its fade starts (FUN_000e0410, world+0xc50 |= 0x10) and the end screen as it opens
/// (0x106ef0); a restart clears it (FUN_00089da0).
pub fn set_cutscene(on: bool) {
    CUTSCENE.store(on, Ordering::Relaxed);
}

/// Whether the player's controls don't reach `p` this frame: during a cutscene, while a script
/// holds it (char+0x834 bit 0, FUN_00129530 turns its player's control off; a 105's walk holds
/// it too, bits 0 and 1), before it's in the world (`Player::absent`). A walk left over once the
/// script has let it go doesn't: its release clears both bits (FUN_00174e70, FUN_001294c0).
pub fn blocks_input(p: &Player) -> bool {
    in_cutscene() || p.held || p.absent
}

/// Whether the level's start has run (play_triggers.rs `run`'s first frame: `level_started`);
/// cleared as a map starts and as it ends.
static LEVEL_STARTED: AtomicBool = AtomicBool::new(false);

/// The level's start has run: its first FIRST_UPDATEs are out (play_triggers.rs `run`).
pub fn level_started() {
    LEVEL_STARTED.store(true, Ordering::Relaxed);
}

/// Whether a squad member a spawn-trigger spawns waits out of the world until it does: yes at a
/// map's start, but not with BF_TEST_GOTO or BF_SCRIPT (whose runs don't play the level's start
/// scripts, which would spawn it: see the module notes), nor once the level's start has run (the
/// map editor's Shift+F5 brings a fresh squad to the map's start after the triggers that spawn it
/// may have fired, and a spent once-only spawn-trigger never fires again). play.rs
/// `player_at_start` and `squad_at_start` ask.
pub fn spawns_wait() -> bool {
    !LEVEL_STARTED.load(Ordering::Relaxed) && std::env::var("BF_TEST_GOTO").is_err() && std::env::var("BF_SCRIPT").is_err()
}

/// The letterbox's bars at full, each of the top and the bottom: 145.8333 / 1000 of the screen's
/// height (FUN_00073110 at 0x731d9: the viewport's top, and 1000 less it its bottom, in
/// thousandths; 70 of 480 lines).
const LETTERBOX_BAR: f32 = 145.833_33 / 1000.0;
/// The longest the bars take to come or go (FUN_00073460 / FUN_000734b0 clamp the duration to
/// 0..10 s).
const LETTERBOX_MAX_TIME: f32 = 10.0;
/// The HUD is hidden while the bars are in beyond this share: a fit, its code not found (the
/// letterbox-trigger's other call, the world's +0xbec +0x54 / +0x58, is the co-op join, not the
/// HUD). take03: no HUD at 26.3 s with the bars at 0.37 of full, faint at 26.4 s (0.17), whole at
/// 26.6 s (gone); take09 at 39.95 s half there at 0.14; take15: hidden through show_patrol_2,
/// which has no 122 / 123, so it's the bars and not the cutscene bit.
const HUD_HIDE_AT: f32 = 0.25;
/// The game's screen: its cameras' aspect is forced to 4:3 (the camera's attach 0x6ba30: view+0x28
/// = 0.75 x view+0x24), so a camera's horizontal FOV gives the vertical one the demo's projection
/// takes.
const SCREEN_ASPECT: f32 = 4.0 / 3.0;
/// The samples a path's curve is cut into per segment for its length (the spline's +8, 16, and
/// +0xc 1/16: FUN_0028c430, FUN_0028c750).
const CURVE_SAMPLES: usize = 16;
/// Where the curve's two made-up end points come from: the curve through the first two points
/// (each doubled) at 0.1, and through the last two at 0.9, mirrored about the end points
/// (FUN_0028c430: FUN_0028b860 with 0x3dcccccd / 0x3f666666): 0.064 of the end segment out.
const CURVE_END_IN: f32 = 0.1;
const CURVE_END_OUT: f32 = 0.9;
/// A scripted walker has reached its world point within this (m): a guess (the walk is the
/// ticket's stub, a straight line).
const WALK_ARRIVE: f32 = 0.6;
/// A scripted walker that hasn't come WALK_PROGRESS m closer to its point for WALK_STALL s is
/// taken as blocked and gives that point up (on to the next, or it stands): a guess, the straight
/// walk's stand-in for the brain's route round what's in its way (FUN_00171710). m09_a's startup
/// sends Brutus to cover_point413 through a wall.
const WALK_PROGRESS: f32 = 0.1;
const WALK_STALL: f32 = 1.0;

/// A 105's walk (play.rs `Player::script_walk`): the world points left, in turn, and for the
/// first the closest the walker has come and how long since it last came WALK_PROGRESS closer.
#[derive(Clone, Debug)]
pub struct Walk {
    points: Vec<Vec3>,
    best: f32,
    stalled: f32,
}

impl Default for Walk {
    fn default() -> Walk {
        Walk { points: vec![], best: f32::INFINITY, stalled: 0.0 }
    }
}

impl Walk {
    /// A walk to these points.
    fn to(points: Vec<Vec3>) -> Walk {
        Walk { points, ..Default::default() }
    }

    /// Whether it has no point left.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The first point given up or reached: on to the next.
    fn next(&mut self) {
        self.points.remove(0);
        (self.best, self.stalled) = (f32::INFINITY, 0.0);
    }

    /// No walk.
    pub fn clear(&mut self) {
        *self = Walk::default();
    }
}

/// The cutscenes' state (see the module notes).
#[derive(Resource, Default)]
pub struct Cutscenes {
    /// the level's scripts by name hash (`Game::scripts`)
    scripts: HashMap<u32, Arc<Script>>,
    /// the scripts that have been started (one player each), in the order they first were
    players: Vec<Playing>,
    /// the level's camera-objects by name: where each is now (paths move them) and its horizontal
    /// FOV (degrees)
    cameras: HashMap<u32, (Mat4, f32)>,
    /// the level's paths by name
    paths: HashMap<u32, PathCurve>,
    /// the level's world points by name (105's): where, and their orientation if they have one
    points: HashMap<u32, (Vec3, Option<Quat>)>,
    /// the camera the view shows (the view's +0x8c), None the game's own
    shown: Option<u32>,
    /// the letterbox: how far in (0..1), where it's going, the time left to get there (the view's
    /// +0x78, +0x7c, +0x80)
    bars: f32,
    bars_to: f32,
    bars_left: f32,
    /// requests of its own this frame (BF_SCRIPT's start; the triggers' come through
    /// `Triggers::take_cutscene`)
    requests: Vec<Request>,
    /// BF_SCRIPT_LOG
    log: bool,
    /// BF_SCRIPT=off: nothing plays
    off: bool,
    /// BF_TEST_GOTO or BF_SCRIPT: the level's start doesn't play its scripts
    skip_start: bool,
    /// BF_SCRIPT=<name>[,<s>]: the script, when, and whether it's been asked
    hook: Option<(u32, f32, bool)>,
    /// the script whose update this frame stopped after an event that asked something of the
    /// world (`step`), to go on from
    resume: Option<usize>,
    /// whether the player's controls were off last frame (`blocks_input`), for the log
    controls_off: Option<bool>,
    /// whether `view` has hidden the HUD under the bars (it shows it again as they go, once)
    hud_hidden: bool,
    /// a mission (101's cuts need one: the world's +0xbec +0x18)
    mission: bool,
    /// the sim time, for the log
    clock: f32,
    /// names for the log
    names: HashMap<u32, String>,
    /// the models of squad members `view` hid while they weren't in the world, shown again as
    /// they come (the player's own is the follow camera's to show, play.rs `follow_camera`)
    hidden: Vec<Entity>,
}

/// A script's player (the 0x130-byte object FUN_00174620 makes): the script, whether it's
/// playing, a start asked (+0xa4), paused (+0x7c), its clock (ms, +0x88), the next event, its
/// running path animations (+0x98) and the characters it holds (its list at +0xc4).
struct Playing {
    script: u32,
    playing: bool,
    restart: bool,
    paused: bool,
    ms: i64,
    /// the clock in seconds, kept whole (the manager's own, FUN_00174270, turned into ms)
    secs: f64,
    next: usize,
    paths: Vec<Running>,
    held: Vec<u32>,
}

/// A START_PATH_ANIMATION under way: who moves (an entity name's hash), along which path, from
/// when (ms), its parameters.
struct Running {
    target: u32,
    path: u32,
    from: i64,
    anim: PathAnimation,
}

/// What the triggers ask of the cutscenes (play_triggers.rs pushes them; `step` does them).
pub enum Request {
    /// a script-trigger's OPERATE (FUN_00204ec0): its script (`script-name`), its entry's
    /// `<script command>` (0 / 1 play from the start, 2 pause, 3 resume, 4 stop), and the signal
    /// it came on (the level's start, FIRST_UPDATE, is skipped under BF_TEST_GOTO / BF_SCRIPT)
    Script { trigger: u32, script: u32, command: i64, signal: i64 },
    /// a letterbox-trigger's OPERATE (show) or STOP (hide), over that many seconds
    Letterbox { trigger: u32, show: bool, secs: f32 },
}

/// A map starts: the level's scripts, cameras, paths and points; the hooks read.
fn start(mut commands: Commands, game: Res<GameData>, map: Res<MapLevel>, current: Res<CurrentMap>) {
    CUTSCENE.store(false, Ordering::Relaxed);
    LEVEL_STARTED.store(false, Ordering::Relaxed);
    let mut cs = Cutscenes { log: std::env::var("BF_SCRIPT_LOG").is_ok(), bars_to: 0.0, ..Default::default() };
    let hook = std::env::var("BF_SCRIPT").ok();
    cs.off = hook.as_deref().is_some_and(|v| matches!(v.trim(), "off" | "none" | "0"));
    cs.skip_start = !spawns_wait();
    if let Some(v) = hook.filter(|_| !cs.off) {
        let mut it = v.split(',').map(str::trim);
        let name = it.next().unwrap_or("");
        let at = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        cs.hook = Some((h(name), at, false));
    }
    cs.mission = map.0.is_some() && !damage::multiplayer_map(&game.0, &current.0);
    if let Some(level) = map.0.as_ref() {
        // the map's own scripts (e01's are loaded too, with its archive's audio, after the map's:
        // `load_default_level`; a name in both is the map's, `Game::scripts`)
        cs.scripts = game.0.scripts.iter().filter(|(_, s)| s.level == current.0).map(|(k, s)| (*k, Arc::new(s.clone()))).collect();
        cs.cameras = level.camera_objects.iter().map(|c| (c.name, (c.transform, 2.0 * c.half_fov))).collect();
        cs.paths = level.paths.iter().map(|p| (p.name, PathCurve::new(p))).collect();
        cs.points = level.objects.iter().filter(|o| o.name != 0)
            .map(|o| (o.name, (o.transform.w_axis.truncate(), o.oriented.then(|| Quat::from_mat4(&o.transform))))).collect();
    }
    if cs.log {
        cs.names = game.0.script_names.clone();
        let mut names: Vec<&str> = cs.scripts.values().map(|s| s.name.as_str()).collect();
        names.sort();
        println!("cutscenes: {} scripts ({}), {} cameras, {} paths{}", names.len(), names.join(", "), cs.cameras.len(), cs.paths.len(),
                 if cs.off { "; BF_SCRIPT=off: none plays" } else if cs.skip_start { "; the level's start plays none (BF_TEST_GOTO / BF_SCRIPT)" } else { "" });
    }
    commands.insert_resource(cs);
    // the letterbox's two bars, over everything of the game's (but the menus)
    for top in [true, false] {
        let mut node = Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), height: Val::Percent(0.0), ..default() };
        if top { node.top = Val::Px(0.0); } else { node.bottom = Val::Px(0.0); }
        commands.spawn((node, BackgroundColor(Color::BLACK), GlobalZIndex(10), LetterboxBar, Name::new("letterbox bar")));
    }
}

/// Leaving the map: the cutscene bit off and the level's start undone (the next map starts
/// without either; its squad is made, `begin_play`, before it starts).
fn finish() {
    CUTSCENE.store(false, Ordering::Relaxed);
    LEVEL_STARTED.store(false, Ordering::Relaxed);
}

/// One of the letterbox's two bars (`start`).
#[derive(Component)]
struct LetterboxBar;

impl Cutscenes {
    /// A name for the log.
    fn label(&self, n: u32) -> String {
        self.names.get(&n).cloned().unwrap_or_else(|| format!("h_{n:08x}"))
    }

    /// Where the shown cutscene camera is, if one is (the view's +0x8c): the sound's listener is
    /// there while it's shown (FUN_00178c10, play_sound_triggers.rs `listener`).
    pub fn camera_at(&self) -> Option<Vec3> {
        self.shown.and_then(|c| self.cameras.get(&c)).map(|c| c.0.w_axis.truncate())
    }

    /// The end screen's view (#226, FUN_001092e0 as the end screen opens): the view cut to the
    /// level camera `camera` (one of the level's end cameras, play_mission.rs picks it; None
    /// keeps the view) and the letterbox in at once (FUN_00073460 with 0 s).
    pub fn end_view(&mut self, camera: Option<u32>) {
        if let Some(c) = camera.filter(|c| self.cameras.contains_key(c)) {
            self.shown = Some(c);
        }
        self.letterbox(true, 0.0);
    }

    /// The letterbox going in (FUN_00073460: toward 1 over the show time, scaled by what's left)
    /// or out (FUN_000734b0: toward 0, scaled by what's in).
    fn letterbox(&mut self, show: bool, secs: f32) {
        let secs = secs.clamp(0.0, LETTERBOX_MAX_TIME);
        if show {
            self.bars_to = 1.0;
            self.bars_left = (1.0 - self.bars) * secs;
        } else {
            self.bars_to = 0.0;
            self.bars_left = self.bars * secs;
        }
    }

    /// The letterbox's step (0x733ee): linearly to its target, there at once once the time left
    /// is under the frame's.
    fn letterbox_step(&mut self, dt: f32) {
        if self.bars == self.bars_to {
            return;
        }
        if self.bars_left <= dt {
            self.bars = self.bars_to;
            self.bars_left = 0.0;
        } else {
            self.bars += (self.bars_to - self.bars) * dt / self.bars_left;
            self.bars_left -= dt;
        }
    }
}

/// The character a script's COMPOUND entity is bound to: the unit of the world by that name
/// (`Player::trigger_name`: the spawn copy's), with whether it's the controlled one. None for one
/// not in the world (`Player::absent`, a spawn's template): the game binds an entity only to an
/// object of its name in the world (FUN_00175090 at the level's load) or to a spawn's copy as
/// it's made (FUN_002096e0 -> FUN_00174fe0), so an event for one not yet there reaches no
/// controller and is lost (the V-squad's 105s at 0.259 s, before its marines' spawns at 7.57 s).
fn unit<'a>(player: &'a mut Player, squad: &'a mut Squad, name: u32) -> Option<(&'a mut Player, bool)> {
    if name == 0 {
        return None;
    }
    if player.trigger_name == name && !player.stand_in {
        return (!player.absent).then_some((player, true));
    }
    squad.0.iter_mut().find(|m| m.trigger_name == name && !m.absent).map(|m| (m, false))
}

/// Where the cutscenes' frame is (`step`): to begin, to go on after an event whose signal asked
/// something of the world (which play_triggers.rs `run` has done since), or done.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pass {
    First,
    Resume,
    Done,
}

/// The cutscenes' frame (from play_triggers.rs `run`, after the triggers' updates and what they
/// asked of the world): the requests, then each script's update (FUN_00174930), each followed by
/// the requests its events made (a 103 to a letterbox-trigger). A script's event whose signal
/// asks something of the world (a 103 to a spawn- or warp-trigger) ends the pass there
/// (Pass::Resume): `run` does it, then the next pass goes on from that script's next event, as
/// the game's signals act at once (a spawn's copy is bound to the scripts as it's made,
/// FUN_002096e0, for the events after it in the same frame).
pub(super) fn step(cs: &mut Cutscenes, tr: &mut Triggers, player: &mut Player, squad: &mut Squad, dt: f32, pass: Pass) -> Pass {
    let (from, resumed) = match (pass, cs.resume.take()) {
        (Pass::Resume, Some(k)) => (k, true),
        _ => {
            cs.clock = player.sim_time;
            // BF_SCRIPT_LOG: the player's controls going off or coming back (last frame's)
            let off = blocks_input(player);
            if cs.log && cs.controls_off != Some(off) {
                let why = [(in_cutscene(), "the cutscene bit"), (player.held, "held"), (player.absent, "not in the world")]
                    .iter().filter(|w| w.0).map(|w| w.1).collect::<Vec<_>>().join(", ");
                println!("t {:.2}: the player's controls {}", cs.clock, if off { format!("off ({why})") } else { "back".to_string() });
            }
            cs.controls_off = Some(off);
            // test hook: BF_SCRIPT=<name>[,<s>]
            if let Some((script, at, false)) = cs.hook {
                if cs.clock >= at {
                    cs.hook = Some((script, at, true));
                    cs.requests.push(Request::Script { trigger: 0, script, command: 0, signal: -1 });
                }
            }
            take_requests(cs, tr, player, squad);
            (0, false)
        }
    };
    for k in from..cs.players.len() {
        if update(cs, k, tr, player, squad, dt, resumed && k == from) {
            cs.resume = Some(k);
            return Pass::Resume;
        }
        take_requests(cs, tr, player, squad);
    }
    Pass::Done
}

/// A character a script holds, on its list (once) for its release at the end (FUN_00174dc0 ->
/// FUN_0005eab0: the list at +0xc4).
fn hold_list(p: &mut Playing, name: u32) {
    if !p.held.contains(&name) {
        p.held.push(name);
    }
}

/// The triggers' requests (see `Request`): a script to play, pause, resume or stop (its start
/// waits for its update this frame, FUN_00174930), the letterbox.
fn take_requests(cs: &mut Cutscenes, tr: &mut Triggers, player: &mut Player, squad: &mut Squad) {
    let mut list = std::mem::take(&mut cs.requests);
    list.extend(tr.take_cutscene());
    for r in list {
        match r {
            Request::Letterbox { trigger, show, secs } => {
                cs.letterbox(show, secs);
                if cs.log {
                    println!("t {:.2}: {}: the letterbox {} over {secs} s", cs.clock, cs.label(trigger), if show { "in" } else { "out" });
                }
            }
            Request::Script { trigger, script, command, signal } => {
                let by = if trigger == 0 { "BF_SCRIPT".to_string() } else { cs.label(trigger) };
                if cs.off || (cs.skip_start && signal == sig::FIRST_UPDATE) {
                    if cs.log {
                        println!("t {:.2}: {by} asks for {}: not played ({})", cs.clock, cs.label(script),
                                 if cs.off { "BF_SCRIPT=off" } else { "the level's start, under BF_TEST_GOTO / BF_SCRIPT" });
                    }
                    continue;
                }
                if !cs.scripts.contains_key(&script) {
                    if cs.log {
                        println!("t {:.2}: {by}: no script {} on this level", cs.clock, cs.label(script));
                    }
                    continue;
                }
                let k = match cs.players.iter().position(|p| p.script == script) {
                    Some(k) => k,
                    None => {
                        cs.players.push(Playing { script, playing: false, restart: false, paused: false, ms: 0, secs: 0.0, next: 0, paths: vec![], held: vec![] });
                        cs.players.len() - 1
                    }
                };
                let p = &mut cs.players[k];
                match command {
                    // play from the start: +0xa4, done by the update (FUN_00174930)
                    0 | 1 => p.restart = true,
                    2 => p.paused = true,
                    3 => p.paused = false,
                    _ => {
                        stop(cs, k, player, squad);
                        continue;
                    }
                }
                if cs.log {
                    let what = match command { 0 | 1 => "plays", 2 => "pauses", _ => "resumes" };
                    println!("t {:.2}: {by} {what} {}", cs.clock, cs.label(script));
                }
            }
        }
    }
}

/// Script `k`'s update (FUN_00174930): its start if asked (FUN_00174b30: the characters it names
/// held, FUN_00174e10), its clock on by the frame (not while paused), the events now due, its
/// paths, and its end at `duration` (FUN_00174bd0). True when it stopped after an event that asked
/// something of the world (`step`); `resumed`, it goes on from there (its start and clock done).
fn update(cs: &mut Cutscenes, k: usize, tr: &mut Triggers, player: &mut Player, squad: &mut Squad, dt: f32, resumed: bool) -> bool {
    let script = cs.players[k].script;
    if !resumed && !cs.players[k].restart && (!cs.players[k].playing || cs.players[k].paused) {
        return false;
    }
    let Some(s) = cs.scripts.get(&script).cloned() else { return false };
    if !resumed && cs.players[k].restart {
        let held = std::mem::take(&mut cs.players[k].held);
        release(&held, player, squad);
        let p = &mut cs.players[k];
        (p.restart, p.playing, p.paused, p.ms, p.secs, p.next) = (false, true, false, 0, 0.0, 0);
        p.paths.clear();
        // every character it names that's in the world, held (FUN_00174e10 -> FUN_00174cd0; one
        // not in the world yet isn't bound to it)
        let mut held: Vec<u32> = vec![];
        for e in s.entities.iter().filter(|e| e.kind == EntityKind::Compound) {
            if let Some((u, _)) = unit(player, squad, e.hash) {
                u.held = true;
                if !held.contains(&e.hash) {
                    held.push(e.hash);
                }
            }
        }
        if cs.log {
            println!("t {:.2}: {} starts ({:.3} s, {} events){}", cs.clock, s.name, s.duration, s.events.len(),
                     if held.is_empty() { String::new() } else { format!(": holds {}", held.iter().map(|&n| cs.label(n)).collect::<Vec<_>>().join(", ")) });
        }
        cs.players[k].held = held;
    }
    if !cs.players[k].playing || cs.players[k].paused {
        return false;
    }
    // the clock in whole ms (FUN_00174270 carries the fraction over)
    let p = &mut cs.players[k];
    if !resumed {
        p.secs += dt as f64;
        p.ms = (p.secs * 1000.0) as i64;
    }
    let now = p.ms;
    while let Some(ev) = s.events.get(cs.players[k].next) {
        if ((ev.time as f64) * 1000.0).round() as i64 > now {
            break;
        }
        cs.players[k].next += 1;
        event(cs, k, &s, ev, tr, player, squad);
        // (a spawn, a warp asked: done before the next event)
        if tr.has_acts() {
            return true;
        }
    }
    // the path animations under way (the list at +0x98: each one's update, 0x275290, until its end)
    let paths = std::mem::take(&mut cs.players[k].paths);
    let mut keep = vec![];
    for r in paths {
        let done = follow(cs, &r, now);
        if !done {
            keep.push(r);
        } else if cs.log {
            println!("t {:.2}: {}: {} reaches the end of {}", cs.clock, s.name, cs.label(r.target), cs.label(r.path));
        }
    }
    // (a path started by this frame's events is in the player's list already)
    let p = &mut cs.players[k];
    keep.append(&mut p.paths);
    p.paths = keep;
    if now >= (s.duration as f64 * 1000.0).round() as i64 {
        stop(cs, k, player, squad);
    }
    false
}

/// Script `k` stops (FUN_00174bd0): its paths dropped, the characters it held released
/// (FUN_00174e70: char+0x834 bits 0 and 1 cleared).
fn stop(cs: &mut Cutscenes, k: usize, player: &mut Player, squad: &mut Squad) {
    let p = &mut cs.players[k];
    let was = p.playing;
    p.playing = false;
    p.restart = false;
    p.paths.clear();
    let held = std::mem::take(&mut p.held);
    let script = p.script;
    release(&held, player, squad);
    if cs.log && was {
        println!("t {:.2}: {} ends{}", cs.clock, cs.label(script),
                 if held.is_empty() { String::new() } else { format!(": releases {}", held.iter().map(|&n| cs.label(n)).collect::<Vec<_>>().join(", ")) });
    }
}

/// The characters a script held, released (FUN_00174e70: char+0x834 bits 0 and 1 cleared).
fn release(held: &[u32], player: &mut Player, squad: &mut Squad) {
    for &n in held {
        if let Some((u, controlled)) = unit(player, squad, n) {
            let_go(u, controlled);
        }
    }
}

/// One character let go (bits 0 and 1 cleared, FUN_00129530): the controlled one's walk ends
/// with it, its player's controls back; another's goes on, the brain's goal (FUN_00171710) that
/// the release lets it pursue (until there or blocked, `held_step`).
fn let_go(u: &mut Player, controlled: bool) {
    u.held = false;
    if controlled {
        // (and its walk's stride: the controls set it again from the next frame)
        u.script_walk.clear();
        (u.move_input, u.walk) = (Vec2::ZERO, false);
    }
}

/// One event of script `s` (player `k`): see the module notes.
fn event(cs: &mut Cutscenes, k: usize, s: &Script, ev: &gscr::Event, tr: &mut Triggers, player: &mut Player, squad: &mut Squad) {
    let first = ev.entities.first().map(String::as_str).unwrap_or("");
    let entity = s.entity(first);
    let name = h(first);
    let say = |cs: &Cutscenes, what: String| {
        if cs.log {
            println!("t {:.2}: {} {:.3}: {first}: {what}", cs.clock, s.name, ev.time);
        }
    };
    match &ev.kind {
        EventKind::Path(anim) => {
            let path = ev.entities.get(1).map(|p| h(p)).unwrap_or(0);
            let movable = cs.cameras.contains_key(&name);
            if !movable || !cs.paths.contains_key(&path) {
                say(cs, format!("START_PATH_ANIMATION along {}: not run ({})", ev.entities.get(1).map_or("-", String::as_str),
                                if movable { "no such path on the level" } else { "only cameras are moved (#232)" }));
                return;
            }
            say(cs, format!("START_PATH_ANIMATION along {} over {:.3} s ({:.2}..{:.2}, flags {}{})", ev.entities[1], anim.duration, anim.start,
                            anim.stop, anim.flags, if anim.event_flags != 0 { ", by the keys' times" } else { "" }));
            let from = (ev.time as f64 * 1000.0).round() as i64;
            let r = Running { target: name, path, from, anim: anim.clone() };
            // its first step now (the event's update runs in the frame it starts)
            let now = cs.players[k].ms;
            if !follow(cs, &r, now) {
                cs.players[k].paths.push(r);
            }
        }
        EventKind::Other(kind, _) => say(cs, format!("event type {kind}: not run (#232)")),
        EventKind::User(u) => {
            let kind = entity.map_or(EntityKind::Unknown, |e| e.kind);
            let template = entity.map_or("", |e| e.template.as_str());
            let own = kind == EntityKind::Marker && template == "script";
            match u.code {
                // 103: the signal straight to the trigger's handler (0x1772b0)
                103 if kind == EntityKind::Marker && template == "trig" => {
                    let signal = u.int(0);
                    say(cs, format!("103: {} to the trigger", sig::name(signal)));
                    if !tr.script_signal(name, signal) {
                        say(cs, "103: no such trigger on the level".into());
                    }
                }
                // 101: a cut to this camera (FUN_00173270), or back to the game's (FUN_00072f00)
                101 => {
                    let arg = u.int(0);
                    if !cs.mission {
                        say(cs, "101: not in a mission: no cut (the world's +0xbec +0x18)".into());
                    } else if arg == 0 && kind == EntityKind::Camera {
                        if !cs.cameras.contains_key(&name) {
                            // (no level camera of that name: the script's own copy stands in,
                            // its fovh taken as the whole horizontal angle: a guess)
                            if let Some(e) = entity {
                                let mut m = Mat4::from_mat3(e.orient);
                                m.w_axis = e.pos.extend(1.0);
                                cs.cameras.insert(name, (m, e.camera.map_or(70.0, |c| c.fovh)));
                            }
                        }
                        cs.shown = Some(name);
                        say(cs, format!("101: the view cuts to it (FOV {:.0} across)", cs.cameras.get(&name).map_or(0.0, |c| c.1)));
                    } else if (own && (1..=3).contains(&arg)) || (arg == 1 && kind == EntityKind::Camera) {
                        // (the script's own 1-3, 0x1761b0; a camera's 1 hands it to the script's,
                        // 0x1732c0, whose 2 and 3 do nothing)
                        cs.shown = None;
                        // the follow camera snaps (the view's cut flag, +0xc8: play_camera.rs
                        // `Rig::cut`)
                        player.rig.cut = true;
                        say(cs, format!("101 {arg}: back to the game's camera"));
                    } else {
                        say(cs, format!("101 {arg}: nothing (a {kind:?})"));
                    }
                }
                122 if own => {
                    CUTSCENE.store(true, Ordering::Relaxed);
                    say(cs, "122: the cutscene bit on (no input, no damage, no menus)".into());
                }
                123 if own => {
                    CUTSCENE.store(false, Ordering::Relaxed);
                    say(cs, "123: the cutscene bit off".into());
                }
                119 if kind == EntityKind::Compound => flags_119(cs, k, name, u, player, squad, &say),
                105 if kind == EntityKind::Compound => walk_105(cs, k, name, u, player, squad, &say),
                100 if kind == EntityKind::Compound => command_100(cs, name, u, player, squad, &say),
                c => say(cs, format!("{c} ({}): not run (#232)", u.raw)),
            }
        }
    }
}

/// 119 on a character (0x17206d; its twelve fields as FUN_001753f0 stores them, v0..v11 in the
/// data's order). With v0 or v9 set (0x172121) it joins the script's list, released at its end
/// (FUN_00174dc0), then (0x17214b-0x17218e) v0 without v1, or v9 without v10, hold it
/// (FUN_00129470), else v0 with v1 hands it to the script's movement (FUN_001294a0); and apart
/// from those, v9 with v10 release it (FUN_001294c0), last. The others (v6 an entity flag, v2 /
/// v7 the character's FUN_001268b0, v8, v11 its brain reset) are logged. One not in the world
/// gets nothing (`unit`).
#[allow(clippy::too_many_arguments)]
fn flags_119(cs: &mut Cutscenes, k: usize, name: u32, u: &UserEvent, player: &mut Player, squad: &mut Squad,
             say: &dyn Fn(&Cutscenes, String)) {
    let v: Vec<bool> = (0..12).map(|i| u.int(i) != 0).collect();
    // (the scripted movement, v0 with v1, holds it as the hold does: its brain the script's, which
    // nothing moves here but a 105)
    let hold = (v[0] && !v[1]) || (v[9] && !v[10]) || (v[0] && v[1]);
    let free = v[9] && v[10];
    let others: Vec<usize> = [2, 6, 7, 8, 11].into_iter().filter(|&i| v[i]).collect();
    let rest = if others.is_empty() { String::new() } else { format!("; v{others:?} not run") };
    if !(v[0] || v[9]) {
        say(cs, format!("119: no hold{rest}"));
        return;
    }
    match unit(player, squad, name) {
        Some((u, controlled)) => {
            hold_list(&mut cs.players[k], name);
            if hold {
                u.held = true;
            }
            if free {
                let_go(u, controlled);
            }
            let what = match (hold, free) { (true, true) => "held, then released", (true, false) => "held", (false, true) => "released", _ => "on the list" };
            say(cs, format!("119: {what}{rest}"));
        }
        None => say(cs, "119: not in the world (its entity isn't bound): nothing".into()),
    }
}

/// CT_TOGGLE_WEAPON and CT_RELOAD_WEAPON (the CT_ name table 0x3c0360: the command handler
/// 0x122040's jump table 0x12276c, cases 9 and 17): two of the three commands the scripts' 100s
/// send (twenty uses: e34_startup, e40_start, E01_Startup, e05_start, e13_intro, e16_startup,
/// m09_c_intro). The third, CT_TOGGLE_CROUCH (15, 0x12247b: e34's Hawk at 16.39 and 20.69 s,
/// e13's), isn't run: the demo's squad AI sets the crouch every frame for a member the script
/// doesn't hold (e34's Hawk isn't held at 16.39 s: her copy comes after the script's start), so
/// the first toggle was lost and the second crouched her for the opening's last seconds.
const CT_TOGGLE_WEAPON: i64 = 9;
const CT_RELOAD_WEAPON: i64 = 17;

/// 100 on a character (0x1722cc): its first value is a command for the character's command
/// handler (vtable +0x15c, 0x122040, with 0), as its player or its brain would send it:
///  * CT_TOGGLE_WEAPON (9; 0x12258c): the switch to its other weapon (play.rs `toggle_weapon`).
///    The openings switch guns this way: e34's Flint (13.06 s) and Tex (17.98 s), e40's Hawk,
///    Tex and Brutus (7.2-8.0 s), so a fresh start has Tex's LZR-50 / Jax-iC in hand (xemu's
///    reads at the first controllable frame, todo/196 notes, and take27 / take30's HUD);
///  * CT_RELOAD_WEAPON (17; 0x122069): with a gun in hand (FUN_0014d2e0: none between a switch's
///    put-away and its draw, `Player::holding`) and none in the second (FUN_0014d310), the clip
///    down a round when it isn't empty (0x12209a-0x1220c1), then the reload (FUN_00120d40,
///    play_ammo.rs): a full clip takes one round from the reserve, e40's Flint (11mm 40 -> 39) and
///    Tex (Sonic 200 -> 199: take30's "36 / 199"; xemu's read: types 1 and 3 at 39 and 199).
///
/// Any other command is logged (#232). One not in the world gets nothing (`unit`).
fn command_100(cs: &Cutscenes, name: u32, u: &UserEvent, player: &mut Player, squad: &mut Squad, say: &dyn Fn(&Cutscenes, String)) {
    let command = u.int(0);
    let Some((c, _)) = unit(player, squad, name) else {
        say(cs, format!("100 ({command}): not in the world (its entity isn't bound): nothing"));
        return;
    };
    match command {
        CT_TOGGLE_WEAPON => {
            let started = super::toggle_weapon(c);
            say(cs, format!("100: CT_TOGGLE_WEAPON: {}", if started { "the switch to its other weapon" } else { "busy, or one weapon: no switch" }));
        }
        CT_RELOAD_WEAPON => {
            let w = c.weapon;
            let reloads = c.holding && c.second.is_none();
            if reloads {
                if let Some(a) = c.ammo.get_mut(w).filter(|a| a[0] > 0) {
                    a[0] -= 1;
                }
                c.reload_wanted = true;
            }
            say(cs, format!("100: CT_RELOAD_WEAPON: {}", if reloads { "a round out of the clip, then the reload" }
                                                           else { "no gun in hand, or one in the second hand: nothing" }));
        }
        n => say(cs, format!("100 ({n}): not run (#232)")),
    }
}

/// 105 on a character (0x171f7a): it's handed to the script's movement and walks to the world
/// points listed after the ':' ("105, 4, 0, 1, 1, 0, 1:wpoint1380": one point), in turn: a
/// straight walk (`held_step`), the ticket's stub for the brain's goal (FUN_00171710). One not in
/// the world gets nothing (`unit`).
#[allow(clippy::too_many_arguments)]
fn walk_105(cs: &mut Cutscenes, k: usize, name: u32, u: &UserEvent, player: &mut Player, squad: &mut Squad,
            say: &dyn Fn(&Cutscenes, String)) {
    let list = u.raw.split(':').nth(1).unwrap_or("");
    let points: Vec<Vec3> = list.split_whitespace().filter_map(|p| cs.points.get(&h(p)).map(|q| q.0)).collect();
    if points.is_empty() {
        say(cs, format!("105: no world point in '{list}'"));
        return;
    }
    match unit(player, squad, name) {
        Some((u, _)) => {
            u.script_walk = Walk::to(points);
            u.held = true;
            hold_list(&mut cs.players[k], name);
            say(cs, format!("105: walks to {}", list.trim()));
        }
        None => say(cs, format!("105 to {}: not in the world (its entity isn't bound): nothing", list.trim())),
    }
}

/// A path animation's step at `now` (ms; 0x275290): its target (a camera) to the path's point and
/// turn for the time. Done (true) once `now` is past its end (it has been put at its end then).
fn follow(cs: &mut Cutscenes, r: &Running, now: i64) -> bool {
    let (Some(path), Some(cam)) = (cs.paths.get(&r.path), cs.cameras.get(&r.target)) else { return true };
    let a = &r.anim;
    let elapsed = (now - r.from).max(0);
    let dur = (a.duration as f64 * 1000.0).round() as i64;
    // the path's share now (0x275630): no stretch, its end; else the start plus the elapsed share
    let span = a.stop - a.start;
    let p = if span == 0.0 { 1.0 } else if dur <= 0 { a.stop } else { a.start + elapsed as f32 / dur as f32 * span };
    let mut m = cam.0;
    let (scale, mut rot, mut pos) = m.to_scale_rotation_translation();
    if a.flags & flags::POSITION != 0 {
        let local = if a.event_flags != 0 { path.keyed(elapsed as f32) } else { path.at(p) };
        pos = path.frame.transform_point3(local + a.offset);
    }
    if a.flags & flags::ORIENTATION != 0 {
        rot = if a.flags & flags::LOOK_AT != 0 {
            // along the path (FUN_0028b000 from `up` and `front`: front = NEG_Z, as a camera looks)
            let ahead = path.frame.transform_vector3(path.at((p + 0.01).min(1.0)) - path.at((p - 0.01).max(0.0)));
            Transform::default().looking_to(ahead.normalize_or(Vec3::NEG_Z), gscr::axis(a.up)).rotation
        } else {
            let ms = if a.event_flags != 0 { elapsed as f32 } else { (p * path.span() as f32).trunc() };
            Quat::from_mat4(&path.frame) * path.turn(ms)
        };
    }
    m = Mat4::from_scale_rotation_translation(scale, rot, pos);
    if let Some(c) = cs.cameras.get_mut(&r.target) {
        c.0 = m;
    }
    // (USE_SCRIPT_DURATION: until the script's end)
    a.flags & flags::USE_SCRIPT_DURATION == 0 && now >= r.from + dur
}

/// A level path as the script player follows it (the path-object, 0x16cca0 / 0x16ce20):
///  * by a share of its length (a START_PATH_ANIMATION without event_flags): the CR spline the
///    path makes of its keys' positions (CV_CRSplinePath h_f28ddde7, FUN_0028c430): uniform
///    Catmull-Rom (FUN_0028b860) through them, with a made-up point before the first and after the
///    last (CURVE_END_IN / OUT), cut into CURVE_SAMPLES pieces a segment, and the point at a share
///    of its length is the one that far along those pieces (0x28d2e0: FUN_0028d480 between two
///    samples); 0 or less the first key, 1 or more the last;
///  * by the keys' own times (with event_flags, the path controller's +0x8c, FUN_001765a0): the
///    keyed spline FUN_0016c190 builds (the ends mirrored), Catmull-Rom between the two keys the
///    time falls between (FUN_0016c4d0, FUN_0016b5c0), the end keys outside them;
///  * its turn: the keys' rotations by their times (ms, +0x30), slerped between the two either
///    side (FUN_0016f080 with two keys; more keys, the game's own blend isn't read: slerp too),
///    the end keys outside; by a share p, the time p x (the last key's - the first's) (0x16cbf0,
///    FUN_00170f00), compared with the keys' own times as they are.
/// Positions are in the path's frame (`frame`, the path-object's transform).
struct PathCurve {
    frame: Mat4,
    /// the sampled curve: (the share of the length so far, the point)
    samples: Vec<(f32, Vec3)>,
    /// the keys (ms, position) and (ms, rotation)
    keys: Vec<(f32, Vec3)>,
    turns: Vec<(f32, Quat)>,
}

/// Catmull-Rom between `b` and `c` at `t` (FUN_0028b860, FUN_0016b5c0): 0.5 (2b + (c - a) t +
/// (2a - 5b + 4c - d) t^2 + (-a + 3b - 3c + d) t^3).
fn catmull_rom(a: Vec3, b: Vec3, c: Vec3, d: Vec3, t: f32) -> Vec3 {
    let (t2, t3) = (t * t, t * t * t);
    0.5 * (2.0 * b + (c - a) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3)
}

impl PathCurve {
    /// The curve of a level path (see `PathCurve`). Key times are whole ms (0x16ce20 truncates
    /// time x 1000).
    fn new(p: &bf_viewer::bf::level::LevelPath) -> PathCurve {
        let keys: Vec<(f32, Vec3)> = p.keys.iter().map(|(t, m)| ((t * 1000.0).trunc(), m.w_axis.truncate())).collect();
        let turns: Vec<(f32, Quat)> = p.keys.iter().map(|(t, m)| ((t * 1000.0).trunc(), Quat::from_mat4(m).normalize())).collect();
        let pts: Vec<Vec3> = keys.iter().map(|k| k.1).collect();
        let mut samples = vec![];
        if pts.len() >= 2 {
            let n = pts.len();
            let mut q = Vec::with_capacity(n + 2);
            let first = catmull_rom(pts[0], pts[0], pts[1], pts[1], CURVE_END_IN);
            q.push(2.0 * pts[0] - first);
            q.extend_from_slice(&pts);
            let last = catmull_rom(pts[n - 2], pts[n - 2], pts[n - 1], pts[n - 1], CURVE_END_OUT);
            q.push(2.0 * pts[n - 1] - last);
            let mut total = 0.0;
            let mut prev = pts[0];
            samples.push((0.0, prev));
            for s in 1..n {
                for j in 1..=CURVE_SAMPLES {
                    let at = catmull_rom(q[s - 1], q[s], q[s + 1], q[s + 2], j as f32 / CURVE_SAMPLES as f32);
                    total += at.distance(prev);
                    samples.push((total, at));
                    prev = at;
                }
            }
            if total > 0.0 {
                for s in &mut samples {
                    s.0 /= total;
                }
            }
        } else if let Some(&only) = pts.first() {
            samples.push((0.0, only));
        }
        PathCurve { frame: p.transform, samples, keys, turns }
    }

    /// The point at share `p` of the curve's length (0x28d2e0).
    fn at(&self, p: f32) -> Vec3 {
        let Some(first) = self.samples.first() else { return Vec3::ZERO };
        if p <= 0.0 || self.samples.len() == 1 {
            return first.1;
        }
        let last = self.samples[self.samples.len() - 1];
        if p >= 1.0 {
            return last.1;
        }
        let i = self.samples.partition_point(|s| s.0 <= p).clamp(1, self.samples.len() - 1);
        let (a, b) = (self.samples[i - 1], self.samples[i]);
        let f = if b.0 > a.0 { (p - a.0) / (b.0 - a.0) } else { 0.0 };
        a.1.lerp(b.1, f)
    }

    /// The point at `ms` by the keys' own times (FUN_0016c4d0 over FUN_0016c190's keys: the ends
    /// mirrored, P0 - (P1 - P0) and Pn + (Pn - Pn-1)).
    fn keyed(&self, ms: f32) -> Vec3 {
        let n = self.keys.len();
        if n == 0 {
            return Vec3::ZERO;
        }
        if n == 1 || ms <= self.keys[0].0 {
            return self.keys[0].1;
        }
        if ms >= self.keys[n - 1].0 {
            return self.keys[n - 1].1;
        }
        let i = self.keys.partition_point(|k| k.0 <= ms).clamp(1, n - 1);
        let p = |j: isize| -> Vec3 {
            if j < 0 { 2.0 * self.keys[0].1 - self.keys[1].1 } else if j as usize >= n { 2.0 * self.keys[n - 1].1 - self.keys[n - 2].1 } else { self.keys[j as usize].1 }
        };
        let (t0, t1) = (self.keys[i - 1].0, self.keys[i].0);
        let t = if t1 > t0 { (ms - t0) / (t1 - t0) } else { 0.0 };
        let i = i as isize;
        catmull_rom(p(i - 2), p(i - 1), p(i), p(i + 1), t)
    }

    /// The keys' time span (ms): the last key's less the first's (FUN_00170f00).
    fn span(&self) -> f32 {
        match (self.turns.first(), self.turns.last()) {
            (Some(a), Some(b)) => b.0 - a.0,
            _ => 0.0,
        }
    }

    /// The turn at `ms` (FUN_0016f080; see `PathCurve`).
    fn turn(&self, ms: f32) -> Quat {
        let n = self.turns.len();
        if n == 0 {
            return Quat::IDENTITY;
        }
        if n == 1 || ms < self.turns[0].0 {
            return self.turns[0].1;
        }
        if ms > self.turns[n - 1].0 {
            return self.turns[n - 1].1;
        }
        let i = self.turns.partition_point(|k| k.0 <= ms).clamp(1, n - 1);
        let (a, b) = (self.turns[i - 1], self.turns[i]);
        let t = if b.0 > a.0 { (ms - a.0) / (b.0 - a.0) } else { 1.0 };
        a.1.slerp(b.1, t)
    }
}

/// A held or walking character's frame (from play.rs: `read_input` for the player, `update_player`
/// for the others, in place of their controls or AI; dt the frame's): nothing reaches it but a
/// script's walk (105), which takes it to each of its points in turn at a walk, facing the way,
/// giving a point up once it's stuck (WALK_STALL); at the last it stands.
pub fn held_step(m: &mut Player, dt: f32) {
    (m.jump_pressed, m.dodge_pressed, m.switch_pressed, m.reload_pressed) = (false, false, false, false);
    (m.item_use, m.next_surface, m.fire, m.aim, m.throw_held, m.sprint) = (false, false, false, false, false, false);
    m.ai_target = None;
    while let Some(&goal) = m.script_walk.points.first() {
        let to = Vec2::new(goal.x - m.position.x, goal.z - m.position.z);
        let d = to.length();
        let w = &mut m.script_walk;
        if d < w.best - WALK_PROGRESS {
            (w.best, w.stalled) = (d, 0.0);
        } else {
            w.stalled += dt;
        }
        if d > WALK_ARRIVE && w.stalled < WALK_STALL {
            m.cam_yaw = (-to.x).atan2(-to.y);
            m.face_yaw = Some(m.cam_yaw);
            m.move_input = Vec2::new(0.0, 1.0);
            m.walk = true;
            return;
        }
        if d > WALK_ARRIVE && std::env::var("BF_SCRIPT_LOG").is_ok() {
            println!("t {:.2}: a script's walk is stuck {d:.1} m short of its point: given up", m.sim_time);
        }
        w.next();
    }
    m.move_input = Vec2::ZERO;
    m.walk = false;
}

/// The view (after the follow camera and the HUD): the letterbox's step and bars, the HUD hidden
/// under them, the shown cutscene camera in place of the follow camera, the squad members not
/// yet in the world not drawn. In the map editor's edit mode the camera, the HUD and the screen
/// are the editor's (play_editor.rs): no bars, no cutscene camera, the HUD left alone. The scripts
/// run on meanwhile, as the triggers that start them do (their pause in edit mode is #230's), so
/// a cutscene under way when play comes back has the view again. The HUD is shown again once,
/// as the bars go, not every frame: other systems hide it too (the editor's view).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn view(time: Res<Time>, mut cs: ResMut<Cutscenes>, player: Res<Player>, squad: Res<Squad>,
        mode: Option<Res<State<super::editor::EditorMode>>>, mut hidden: ResMut<super::hud::HudHidden>,
        mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
        mut bars: Query<&mut Node, With<LetterboxBar>>,
        mut hud: Query<&mut Visibility, (With<super::hud::HudRoot>, Without<LetterboxBar>)>,
        mut roots: Query<&mut Visibility, (Without<super::hud::HudRoot>, Without<LetterboxBar>, Without<MainCamera>)>) {
    let dt = frame_dt(&time);
    let editing = mode.is_some_and(|m| *m.get() == super::editor::EditorMode::Edit);
    cs.letterbox_step(dt);
    // (the bars' share is an eased ratio: one that isn't finite is 0 and logged once, #349)
    let height = super::ui_guard::percent("a letterbox bar's height", if editing { 0.0 } else { cs.bars * LETTERBOX_BAR * 100.0 });
    for mut n in &mut bars {
        if n.height != height {
            n.height = height;
        }
    }
    // the HUD: hidden while the bars are in (each frame: the editor's view shows it in play);
    // shown again once as they go; in edit mode the editor's
    let want = if editing {
        cs.hud_hidden = false;
        None
    } else if cs.bars > HUD_HIDE_AT {
        cs.hud_hidden = true;
        Some(Visibility::Hidden)
    } else if cs.hud_hidden {
        cs.hud_hidden = false;
        Some(Visibility::Inherited)
    } else {
        None
    };
    for mut v in &mut hud {
        if let Some(want) = want.filter(|w| *v != *w) {
            *v = want;
        }
    }
    // (whether the bars have it hidden, for what draws only with the HUD: play_scope_view.rs)
    if hidden.letterbox != cs.hud_hidden {
        hidden.letterbox = cs.hud_hidden;
    }
    if let Some((m, fov_across)) = cs.shown.filter(|_| !editing).and_then(|c| cs.cameras.get(&c)).copied() {
        if let Ok((mut tr, mut projection)) = cam.single_mut() {
            *tr = Transform::from_matrix(m);
            if let Projection::Perspective(pp) = projection.as_mut() {
                // the game's horizontal angle on its 4:3 screen, as the demo's vertical one
                let fov = 2.0 * ((fov_across.to_radians() * 0.5).tan() / SCREEN_ASPECT).atan();
                if (pp.fov - fov).abs() > 1e-4 {
                    pp.fov = fov;
                }
            }
        }
    }
    // the squad members not in the world yet aren't drawn; a member that has come is again (the
    // player's own model: the follow camera's, which shows it unless scoped)
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let Some(l) = u.loaded.as_ref() else { continue };
        let Ok(mut v) = roots.get_mut(l.root) else { continue };
        if u.absent {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
            if !cs.hidden.contains(&l.root) {
                cs.hidden.push(l.root);
            }
        } else if let Some(i) = cs.hidden.iter().position(|&e| e == l.root) {
            cs.hidden.swap_remove(i);
            *v = Visibility::Inherited;
        }
    }
}
