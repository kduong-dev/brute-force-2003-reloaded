//! The level's triggers, run (#220, the tutorial's step 1 of #28; viewer/docs/triggers.md).
//! Every trigger the level file places (bf/triggers.rs: 500 in the tutorial) lives here as the
//! game keeps it in its object, and acts as default.xbe's trigger code does. Written from the
//! code (the addresses are in the comments and in viewer/docs/code-map.md); nothing is fitted.
//!
//! Events. An event is {signal, the sender's iD, a subject object, data} (the game's 0x14-byte
//! event, vtable 0x3a3fb4). A trigger sends one to its targets (h_e91cf6a8) only while it's
//! enabled and alive (FUN_001ff0a0); an object sends one to the triggers listening to it (those
//! that list it in h_0b6b92ba: FUN_001fe3d0). A trigger answers with its entries (FUN_001feb60):
//! those whose signal and match-id are the event's and whose use count is under their limit,
//! in order. An entry that acts counts a use (FUN_001feac0); a trigger whose entry is spent
//! with h_170a9b80, or any of whose entries acts with h_137926ff, sends DISABLED and disables
//! itself. Enabling a trigger zeroes its entries' counts (0x1fefc0). After any event but
//! FIRST_UPDATE, h_f657f799 kills a trigger and h_1eec0e75 disables it, once (FUN_001ff000).
//!
//! The base's actions (every class, FUN_001ff0f0): ENABLE / DISABLE / TOGGLE_ENABLE (ENABLED to
//! the targets: the DISABLED a DISABLE sends is lost, as the trigger is off by then) and
//! USER_SIG1-8 (USER_SIGNAL1-8 to the targets). FIRST_UPDATE: each enabled trigger's first
//! update sends it to the trigger itself, once (FUN_001fe610 / FUN_001fe660); the level's start
//! sends it to every script-trigger first (FUN_001fe260). A disabled trigger runs only its base
//! actions.
//!
//! The classes run here (their handlers, vtable +0x60, and updates, +0x1c):
//!  * router (0x208fc0): passes every event but FIRST_UPDATE to its targets, or, with
//!    h_f0d6857f, those its OPERATE entries accept, as `out-signal` when that isn't -1.
//!  * timer (0x200190, 0x2002f0): OPERATE starts it, TIMER_FULL_RESET stops and reloads it,
//!    RESET_COUNTDOWN_TIME reloads it; running and enabled it counts down and at 0 sends
//!    TIMER_PULSE, then stops and reloads (h_f92b0b15) or reloads (enable-looping).
//!  * calculator (0x1ff5f0): INC / DEC / MODIFY (by the entry's value) / SET_FROM_SIG_DATA,
//!    then its test against the target: CALCULATOR and TRUE, or FALSE, with the counter.
//!  * spawn-trigger (0x209ac0, 0x209bd0, FUN_002099e0): OPERATE spawns its template at its own
//!    transform (after h_eab6952f s if set) while armed; it's armed once, SPAWN_RESET re-arms a
//!    re-armable one (h_0f95f576). A character copy stands on the ground under the transform
//!    (the position setter's ray, `on_ground`). A squad-type character's copy the session
//!    doesn't count in the squad is taken away at once (FUN_00209900, `Triggers::member_kept`,
//!    reading the save-game slots of play_flags.rs). The copy sends SPAWNED to its listeners,
//!    the trigger OPERATED to its targets; the triggers that list the spawn-trigger listen to its
//!    copies.
//!  * gamevar-trigger (0x1ffc40, #221): OPERATE works out its operation (h_015914ab: ==, <, <=,
//!    >, >=, A, the out flag's default, +, -, x, /) of its operands (a game flag, a constant,
//!    random, the event's data) and writes it to its out operand (a game flag: play_flags.rs's
//!    `Campaign`), sending VALUE_CHANGED when that changed it, then TRUE (the result not 0) or
//!    FALSE. The tutorial's set the objectives' `state` and test the squad's member flags.
//!  * score-trigger (0x20ba00, #221): OPERATE adds its amount to the campaign squad's cash, x
//!    1 + 0.1 x the difficulty (h_11819686).
//!  * plate- and proximity-detector (FUN_001f9680, FUN_001f8db0): the characters its filter
//!    admits (FUN_001fb980) in its volume; one inside 0.1 s sends ENTER_DETECTOR (and
//!    ENTER_DETECTOR_FIRST if none was), one out of it 0.2 s EXIT_DETECTOR, the last gone
//!    EMPTIED_DETECTOR. The plate's volume is its box (`size` its half-extents: the phantom
//!    surface h_051ac63e is that box), the proximity detector's a sphere of `radius`; the
//!    character is the demo's cylinder (BODY_RADIUS, BODY_HEIGHT), not its Ipion body.
//!  * anim-trigger (0x204d00, FUN_00204bb0): OPERATE plays its entry's clip on its first object,
//!    forward or backward (wwit-debugging) at its time-scale, ANIM_START / ANIM_BACK_START to its
//!    targets when the way it plays changes. The doors (play.rs `doors`) are what it moves.
//!  * sound-trigger and dialog-trigger (#223, play_sound_triggers.rs; 0x206450 / 0x206690,
//!    0x206d20 / 0x206ee0): PLAY_SOUND plays a sound library entry (alike everywhere, or at its
//!    place with its distances; only near a player with h_15bc1769), PLAY_DIALOG queues it on
//!    the world's dialog queue (one line at a time), STOP / PAUSE / RESUME; when the sound's
//!    length has run (the bank's size over its rate) it sends FINISHED to its targets. A
//!    dialog-trigger's character says its line, FINISHED at its end.
//!  * message, input, menu and event triggers (#224, play_hints.rs and play_panels.rs; 0x2078e0,
//!    0x2008a0, 0x2082d0 / 0x208430, 0x202dd0): OPERATE shows or clears the HUD's two hint lines
//!    ("Press [BACK] to check mission objectives.") and sends OPERATED; TRUE every frame a virtual
//!    button is held; OPERATE opens a game-options panel over the game (the clock stopped),
//!    OPERATED, then MENU_ACCEPT / MENU_Y_BUTTON from its buttons and FINISHED once it's closed;
//!    an AI event about a listened object (EVT_HEALTH_PCT_25 ...) passed on as EVENT_MATCH.
//!  * destroyer (0x200d70, 0x200e40): OPERATE marks it, and its update removes its objects (a
//!    spawn-trigger's copy for a spawn-trigger), then it disables itself. The update runs the
//!    base update first, so an OPERATE on its own FIRST_UPDATE removes them in that same update.
//!  * script-, letterbox- and warp-trigger (#222, with play_script.rs; 0x205000, 0x207480,
//!    0x200650): OPERATE asks for its script to play (by its entry's `<script command>`: play,
//!    pause, resume, stop), brings the letterbox in (its STOP takes it out), warps its objects to
//!    its world point. A script's 103 sends its signal straight to a trigger's handler.
//!  * level-over and zone triggers (#226, play_level_over.rs; 0x207fe0, 0x207de0): OPERATE ends
//!    the mission (h_18f66101) or the zone, and names the zone to go to; the mission flow
//!    (play_mission.rs) does the rest.
//!  * every other class (radar, property, the damage plate ...): its base actions; its own
//!    actions are logged as left to their step (`bf::triggers::later_step`). Waypoint and
//!    evaluate triggers: base only.
//!
//! Object events from the world: a character's death (DESTROYED, from its trigger name: a spawn
//! copy's), a named breakable object broken (DESTROYED), a wall panel used (USE_OBJECT, play.rs
//! `doors`), a pickup taken (PICKUP_OBJECT, play_pickups.rs).
//!
//! Test hooks:
//!  * BF_TRIGGER_LOG=1: every event a trigger or an object sends (who, the signal, its subject
//!    and data, to whom), what each class does with it, and at the start the level's counts,
//!    the squad's spawns and the detectors. Names are the level's cutscene scripts' where they
//!    name them (Game::script_names), else the editor's `trigger_spawn<n>`-style ones that hash
//!    to them, else h_xxxxxxxx.
//!  * BF_TRIGGER_FIRE=<trigger>,<signal>[,<s>][;...]: sends the signal (a number or a TRIG_SIG_
//!    name: USER_SIGNAL1, USER1, ENTER_DETECTOR ...) to that trigger at that time (default 0),
//!    as if a sender with iD 0 had; `@<object>,<signal>` sends it from an object to its
//!    listeners (a copy's DESTROYED). A trigger is named by its name (hashed) or h_xxxxxxxx.

use std::collections::{HashMap, HashSet};

use bf_viewer::bf::character::{Game, OBJECT_EFFECT};
use bf_viewer::bf::hash::h;
use bf_viewer::bf::level::Level;
use bf_viewer::bf::triggers::{self as data, act, op, operand, sig, Class, Filter, Operand, Trigger, Volume};
use bf_viewer::level_scene::Placed;

use super::flags::Campaign;
use super::*;

/// The triggers' state and the world's events for them, set up when a map starts (after
/// play.rs `setup` has placed the level, and the campaign's state its squad: play_flags.rs);
/// their frame after the squad has moved (the detectors see where it is now) and before the
/// doors move (an anim-trigger's clip starts in its frame).
pub fn plugin(app: &mut App) {
    app.init_resource::<TriggerEvents>()
        .init_resource::<Triggers>()
        .add_systems(OnEnter(AppState::Playing), start.after(setup).after(flags::begin_level))
        .add_systems(Update, run.after(squad_control).before(doors).run_if(in_state(AppState::Playing)))
        // (the hint lines, button gates, AI events and panels: #224)
        .add_plugins((hints::plugin, panels::plugin));
}

/// An object's event from the world (the game's FUN_001fe3d0: to the triggers listening to the
/// object): a wall panel used (USE_OBJECT, play.rs `doors`), a pickup taken (PICKUP_OBJECT,
/// play_pickups.rs). `from` and `subject` are names (the user, 0 none); `data` the event's
/// +0x10 (a panel's h_f1660a0d).
pub struct ObjectEvent {
    pub from: u32,
    pub signal: i64,
    pub subject: u32,
    pub data: i64,
}

/// The world's object events waiting for the runtime (`run` delivers them).
#[derive(Resource, Default)]
pub struct TriggerEvents(pub Vec<ObjectEvent>);

/// A squad member's start from its spawn-trigger (#102: the squad is spawned by spawn-triggers
/// at its start points; the tutorial's Tex by trigger_spawn8 at (65.5, 28.5, 74.0)): its squad
/// slot, its template placement (h_1299668b), the trigger, its copy's name (h_14f15a34: "tex_1",
/// what the level's triggers and scripts call it), where its feet go (on the ground under the
/// trigger's position: `on_ground`, the tutorial's Tex at y 25.25 as in take03), the way it
/// faces (the trigger's -z: `yaw_of`) and the template's inventory: the copy is the placement,
/// which carries it (the character's property handler 0x112eb0 reads it, FUN_001130e0). Its
/// `<weapon>`s: the tutorial's Tex two MK-ASLTs (h_0c3db625, 48 a clip: take03's 48 / 500 and its
/// icon); its `<h_10dd6db2 name count>` items: his 500 rounds of ammo-type 2 (h_02735ad0), the
/// squad's reserve (play_ammo.rs `stock`), and grenades. A member placed in the world instead
/// (m09_a's) has its placement for all of these, `trigger` 0.
#[derive(Clone, Debug)]
pub struct SquadSpawn {
    pub slot: usize,
    pub template: u32,
    pub trigger: u32,
    pub copy: u32,
    pub feet: Vec3,
    pub yaw: f32,
    pub weapons: Option<Vec<u32>>,
    pub items: Vec<(u32, i64)>,
}

impl SquadSpawn {
    /// What a member started from this spawn carries: the name the level's triggers know it by
    /// and its template's weapons (play.rs `spawn_player` arms a character with
    /// `enemy.weapons` when it's set).
    pub fn dress(&self, p: &mut Player) {
        p.trigger_name = self.copy;
        p.enemy.weapons = self.weapons.clone();
    }
}

/// The level's squad spawns (`squad_spawns`), one per squad member it spawns or places; empty
/// on a map without (the deathmatch maps' start points, the flat test floor).
#[derive(Resource, Default, Clone)]
pub struct SquadSpawns(pub Vec<SquadSpawn>);

impl SquadSpawns {
    /// Squad slot `slot`'s spawn, if the level has one.
    pub fn of(&self, slot: usize) -> Option<&SquadSpawn> {
        self.0.iter().find(|s| s.slot == slot)
    }

    /// Who leads: `wanted` (BF_CHARACTER's) if the level spawns it or spawns no squad, else the
    /// first it spawns (the tutorial: Tex).
    pub fn leader(&self, wanted: usize) -> usize {
        if self.0.is_empty() || self.of(wanted).is_some() { wanted } else { self.0[0].slot }
    }
}

/// The yaw a spawn-trigger's copy faces at the trigger's transform `m`: down its -z, as the
/// level's cameras look. Checked on the tutorial's Tex (trigger_spawn8, whose +z points east):
/// the game's follow camera after the opening (take03 at 32 s) looks west past him, at the dark
/// peak and the palms, which the demo shows facing -z; along +z (the start points' way,
/// arena.rs) he faced the rock face east of the start.
fn yaw_of(m: &Mat4) -> f32 {
    let f = -m.z_axis.truncate();
    (-f.x).atan2(-f.z)
}

/// How far below the point it's given the game looks for the ground to stand a character on:
/// the position setter (character vtable +0x38, 0x1154b0) asks FUN_0012dc00 with 50.0 (pushed
/// at 0x115567), which casts FUN_000dddd0's ray straight down from the point that far.
const GROUND_PROBE: f32 = 50.0;

/// Where a character put at `at` stands: on the highest floor below the point within
/// GROUND_PROBE m, else at the point. Every character the game places goes through this: the
/// place call (vtable +0x50, 0x1159c0) runs the position setter 0x1154b0 first, which puts the
/// feet at the ray's hit (plus the type's height-offset, type+0x2ac: 0 on every character here)
/// unless the world's +0xc50 bit 2 is set (not traced; not the case in the takes). So the
/// tutorial's Tex, spawned at y 28.53, stands at 25.25 (take03: no fall, 69 ms from the spawn),
/// and e40's Brutus 10.9 m below his trigger (xemu.md's live read). The demo's floor (the
/// arena's floors no steeper than 65 degrees, as its movement stands on) stands for the ray's
/// hit.
pub fn on_ground(at: Vec3) -> Vec3 {
    let y = world::arena().and_then(|a| a.floor_below(at.x, at.z, at.y)).map(|f| f.0).filter(|&y| at.y - y <= GROUND_PROBE);
    Vec3::new(at.x, y.unwrap_or(at.y), at.z)
}

/// A level object's character placement (`Game::placements`, with its inventory and AI): the
/// one of that name at that place.
fn placement_of<'a>(game: &'a Game, o: &bf_viewer::bf::level::Placement) -> Option<&'a bf_viewer::bf::character_type::Placement> {
    game.placements.iter().find(|p| p.name == o.name && p.transform.w_axis.distance(o.transform.w_axis) < 0.01)
}

/// The level's squad spawns: each spawn-trigger whose template is a character placement of a
/// squad slot's type (brutus, flint, hawk, tex), the first per slot in the file's order (e40,
/// the tutorial); then a squad placement no spawn-trigger takes as its template is in the world
/// from the start, where it's placed (m09_a's four: its warp-triggers move them, step 3), named
/// by its own name, `trigger` 0. A template stays out of the world until spawned (FUN_000dfe90
/// takes it out as its spawn-trigger links). Either way the feet go on the ground below
/// (`on_ground`: a placed character goes through the same place call, FUN_0008b300).
pub fn squad_spawns(game: &Game, level: &Level) -> SquadSpawns {
    let mut out: Vec<SquadSpawn> = vec![];
    let slot_of = |kind: u32| (0..SQUAD).find(|&c| squad_type(c) == kind);
    let weapons = |o: &bf_viewer::bf::level::Placement| placement_of(game, o).map(|p| p.weapons.iter().map(|w| w.0).collect::<Vec<_>>())
        .filter(|w| !w.is_empty());
    let items = |o: &bf_viewer::bf::level::Placement| placement_of(game, o).map_or(vec![], |p| p.items.clone());
    let mut templates = HashSet::new();
    for t in &level.triggers {
        let (Class::Spawn { template, copy_name, .. }, Some(m)) = (&t.class, t.transform) else { continue };
        templates.insert(*template);
        let Some(o) = level.objects.iter().find(|o| o.name == *template && o.tag == h("character-object")) else { continue };
        let Some(slot) = slot_of(o.kind) else { continue };
        if out.iter().any(|s| s.slot == slot) {
            continue;
        }
        out.push(SquadSpawn { slot, template: *template, trigger: t.name, copy: *copy_name, feet: on_ground(m.w_axis.truncate()), yaw: yaw_of(&m),
                              weapons: weapons(o), items: items(o) });
    }
    for o in level.objects.iter().filter(|o| o.tag == h("character-object") && o.name != 0 && !templates.contains(&o.name)) {
        let Some(slot) = slot_of(o.kind) else { continue };
        if out.iter().any(|s| s.slot == slot) {
            continue;
        }
        out.push(SquadSpawn { slot, template: o.name, trigger: 0, copy: o.name, feet: on_ground(o.transform.w_axis.truncate()),
                              yaw: yaw_of(&o.transform), weapons: weapons(o), items: items(o) });
    }
    SquadSpawns(out)
}

/// The sound- and dialog-triggers, the world's dialog queue and the voices they play (#223): a
/// child of this module, so that its classes reach the runtime's own state and sends.
#[path = "play_sound_triggers.rs"]
pub(crate) mod sounds;

/// The message, input and event triggers, the AI events they hear, the virtual buttons and the
/// HUD's two hint lines (#224): a child of this module, as the sound side is.
#[path = "play_hints.rs"]
pub(crate) mod hints;

/// The menu-triggers and the game-options panels they open over the game (#224).
#[path = "play_panels.rs"]
pub(crate) mod panels;

/// The level-over and zone triggers, what they ask of the mission flow (#226).
#[path = "play_level_over.rs"]
pub(crate) mod level_over;

/// An event as the game's triggers pass it (FUN_001fe660 builds one, vtable 0x3a3fb4): its
/// signal (+4), its sender's iD (+8, what an entry's match-id answers), its subject (+0xc: who
/// entered a detector, a spawn's copy ...; by name here, 0 none) and its data (+0x10: a
/// calculator's counter, a panel's h_f1660a0d; -1 none).
#[derive(Clone, Copy, Debug)]
struct Ev {
    signal: i64,
    id: i64,
    subject: u32,
    data: i64,
}

impl Ev {
    /// A new event from a sender with iD `id`, about nothing (FUN_001fe660's: subject 0, data -1).
    fn new(signal: i64, id: i64) -> Ev {
        Ev { signal, id, subject: 0, data: -1 }
    }

    /// This event passed on as another signal from another sender (FUN_001fdf20: its subject and
    /// data kept).
    fn with(self, signal: i64, id: i64) -> Ev {
        Ev { signal, id, ..self }
    }
}

/// A character the detectors see this frame (`actors`): its name (its trigger name, else a
/// stand-in, see `actors`), type, whether its type is a squad type (h_e78efbb2) and it has a
/// human player, its team (the game's alignment numbers), feet, height and radius.
struct Actor {
    name: u32,
    kind: u32,
    squad_type: bool,
    human: bool,
    team: i64,
    feet: Vec3,
    height: f32,
    radius: f32,
}

/// A detector's occupant (the list at +0xdc / +0xe0, FUN_001f8db0): how long it's been one (s;
/// -1 once its ENTER_DETECTOR is sent) and how long it's been out of the volume (never reset:
/// the game sums it, FUN_001f8db0).
struct Occupant {
    name: u32,
    inside: f32,
    absent: f32,
}

/// What a class keeps besides the base (its object's own fields, see `data::Class`).
enum ClassState {
    Other,
    /// a sound-trigger's fields from +0x40 (play_sound_triggers.rs)
    Sound(Box<sounds::Sound>),
    /// a dialog-trigger's +0x40-+0x48 (play_sound_triggers.rs)
    Dialog(sounds::Dialog),
    /// a message trigger's two hint lines, +0x40-+0x5c (play_hints.rs)
    Message([data::HintLine; 2]),
    /// an input trigger's virtual button, +0x48 (play_hints.rs)
    Input { vb: i64 },
    /// a menu-trigger's menu and whether it's open, +0x40-+0x4c (play_panels.rs)
    Menu(panels::MenuTrigger),
    /// an event-trigger's EVT_ number and filter, +0x40 / +0x44 (play_hints.rs)
    Event { event: i64, filter: u32 },
    Router { filters: bool, out_signal: i64 },
    /// +0x48 running, +0x49 looping, +0x4a stops, +0x40 initial, +0x44 current, +0x50 the subject
    /// its OPERATE came with (its pulse's)
    Timer { running: bool, looping: bool, stops: bool, initial: f32, current: f32, subject: u32 },
    Calc { counter: i64, target: i64, test: i64 },
    /// armed (+0xe4 bit 3, set by its constructor, 0x1fd680-0x1fd6e7), the delay running (+0xec), its current
    /// copy (+0xc4, by name); `member`: its template's character type when that's a squad type
    /// (h_e78efbb2, type+0x1a8 bit 0) other than the demo's squad members' (whose spawns move the
    /// member, `spawn_copy`), what FUN_00209900 checks; `check` its h_0fa4f510 (+0xe4 bit 5)
    Spawn { armed: bool, rearm: bool, pending: f32, delay: f32, template: u32, copy_name: u32, fade: f32, copy: u32, member: Option<u32>, check: bool },
    Detector { volume: Volume, filter: Filter, keep: bool, occupants: Vec<Occupant> },
    /// +0x5c bit 0: its clip last played backward (set by its constructor 0x204db0: the first
    /// forward play sends ANIM_START)
    Anim { backward: bool },
    /// +0x4c: OPERATE asked, done by its update (0x200e40)
    Destroyer { pending: bool },
    /// +0x50 the operation, the operands out, A, B (+0x40 + 4i, +0x4c + i): `out`'s constant is
    /// live (a CONSTANT out keeps the last result)
    Gamevar { op: i64, operands: [Operand; 3] },
    /// +0x40 the amount, +0x44 the key, +0x48 bit 2 keyed, bit 1 scaled
    Score { amount: f32, key: u32, keyed: bool, scaled: bool },
    /// script-trigger (#222): +0x40 its `script-name` (the script's name hash, FUN_00204e40)
    Script { script: u32 },
    /// letterbox-trigger (#222): +0x40 `show-duration`, +0x44 `hide-duration` (FUN_00207410)
    Letterbox { show: f32, hide: f32 },
    /// warp-trigger (#222): its `world-point` (+0x40, the point linked at +0x44) and h_0ede4719
    /// (+0x48: the fade-in a warped character gets, 0x200400)
    Warp { point: u32, fade: f32 },
    /// level-over-trigger (#226): h_18f66101 (+0x40)
    LevelOver { ends_mission: bool },
    /// zone-trigger (#226): h_14b0089f (+0x44), the next zone's file name
    Zone { next: String },
}

/// Which class a trigger runs as (for `update` and `deliver`).
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Other,
    Sound,
    Dialog,
    Message,
    Input,
    Menu,
    Event,
    Router,
    Timer,
    Calc,
    Spawn,
    Detector,
    Anim,
    Destroyer,
    Gamevar,
    Score,
    Script,
    Letterbox,
    Warp,
    LevelOver,
    Zone,
}

/// A trigger as the game keeps it: its data (the entries' use counts are live) and +0x3d's bits:
/// bit 0 enabled, bit 4 dead, bit 1 dies after its first event, bit 2 disables after it, bit 5
/// its FIRST_UPDATE sent.
struct Live {
    t: Trigger,
    enabled: bool,
    dead: bool,
    die_once: bool,
    disable_once: bool,
    first_done: bool,
    state: ClassState,
}

/// What the triggers ask of the world this frame (`run` does it after their updates).
enum Act {
    /// an anim-trigger's clip on its object (FUN_00204bb0's request to world+0x1f4): which way,
    /// how fast, its AC_ command
    Anim { trigger: usize, object: u32, clip: u32, backward: bool, rate: f32, command: i64 },
    /// a spawn-trigger's copy of its template at its transform, named `copy`; characters fade
    /// in over `fade` s (char+0x73c = h_eeae96e7, FUN_00209620 -> FUN_0011bd00)
    Spawn { trigger: usize, template: u32, copy: u32, at: Mat4, fade: f32 },
    /// a destroyer's object (FUN_00200d00 -> FUN_00200b70)
    Destroy { trigger: usize, object: u32 },
    /// a warp-trigger's OPERATE (FUN_00200480): its objects to its world point (`warp`)
    Warp { trigger: usize },
}

/// A BF_TRIGGER_FIRE entry: when (sim time), an object's event (`@`) or a trigger's, whose, the
/// signal.
struct Fire {
    at: f32,
    object: bool,
    name: u32,
    signal: i64,
    done: bool,
}

/// How deep events may nest (a send whose handler sends ...): the level data has no loop, so
/// this only guards against one.
const MAX_DEPTH: u32 = 64;
/// A spawn-trigger's delay counts only from this (0x209b5e compares h_eab6952f with
/// 4.76837e-07 at 0x3a54f4: under it the copy is made at once).
const MIN_DELAY: f32 = 4.768_37e-7;
/// A detector's entry time (s): an occupant counts as in after 0.1 s in its volume
/// (FUN_001f8db0 reads it at 0x3a4ff0).
const ENTER_TIME: f32 = 0.1;
/// A detector's exit time (s): an occupant leaves after 0.2 s out of its volume, summed
/// (FUN_001f8db0 reads it at 0x3a4f90).
const EXIT_TIME: f32 = 0.2;
/// The first of the names the runtime makes up for a re-armable spawn-trigger's copies (the
/// game names them none: FUN_00209620 passes no name then), counted up from here.
const MADE_UP_COPY: u32 = 0x7E00_0000;
/// The first of the names the runtime makes up for characters no trigger named, so the
/// detectors can tell them apart (see `actors`): + the squad slot, or + 100 + the NPC number.
const MADE_UP_UNIT: u32 = 0x7F00_0000;
/// A gamevar-trigger's == : |A - B| under this (0x3a4fd8, compared at 0x1ffd44).
const GAMEVAR_EQUAL: f32 = 1e-5;
/// A gamevar-trigger's result under this (absolute) sends FALSE, else TRUE (0x3a4fb8, 0x1fff8f).
const GAMEVAR_ZERO: f32 = 1e-6;
/// The random numbers' generator (FUN_00233b90): seed x 0x19660d + 0x3c6ef35f, its top 23 bits a
/// mantissa in [1, 2), less 1. Its seed (DAT_0046cfd8) as the demo starts it: a guess (the
/// game's isn't traced).
const RANDOM_MUL: u32 = 0x0019_660D;
const RANDOM_ADD: u32 = 0x3C6E_F35F;
const RANDOM_SEED: u32 = 0x2210_0221;
/// The numbers of the NPCs the triggers spawn (`testtools::Npc::id`), clear of the test tools'
/// own (counted from 1), and small enough for the shots' seed (play.rs `step_player`: id x
/// 104729 in a u32, under 41000).
const NPC_IDS: u32 = 20_000;
/// The editor's names for level objects (the families the analyst's dump hashed: leads §0),
/// for the log: trigger_spawn8, wpoint194 ...
const NAME_FAMILIES: [&str; 31] = ["trigger_spawn", "trigger_router", "trigger_timer", "trigger_letterbox", "trigger_level_over",
    "trigger_damage_plate", "trigger_menu", "trigger_warp", "trigger_sound", "trigger_gamevar", "trigger_calculator", "trigger_radar",
    "trigger_plate", "trigger_proximity", "trigger_anim", "trigger_script", "trigger_dialog", "trigger_event", "trigger_evaluate",
    "trigger_score", "trigger_destroyer", "wpoint", "cover_point", "patrol_point", "door_point", "save_point", "direction_point",
    "progress_point", "path", "camera", "TUTORIAL_terrorist_"];

/// The level's triggers as they run (see the module notes).
#[derive(Resource, Default)]
pub struct Triggers {
    /// every trigger of the level, in the file's order (the order they update in)
    live: Vec<Live>,
    /// trigger name -> its place in `live` (the first, if two share a name)
    by_name: HashMap<u32, usize>,
    /// object name -> the triggers listening to it (their h_0b6b92ba lists it: FUN_001febb0)
    listeners: HashMap<u32, Vec<usize>>,
    /// spawn-trigger -> the triggers listing it, which listen to its copies (FUN_001fe8a0 keeps
    /// them at its +0xf0; FUN_002096e0 adds them to each copy)
    copy_listeners: HashMap<usize, Vec<usize>>,
    /// a copy's name -> its spawn-trigger
    copy_of: HashMap<u32, usize>,
    /// every object name of the level (a template that isn't one makes no copy: FUN_00209620)
    known: HashSet<u32>,
    /// what this frame's triggers asked of the world (`run` does it after their updates)
    acts: Vec<Act>,
    /// the sound side (play_sound_triggers.rs): the world's dialog queue, the voices asked for,
    /// where the listener and the characters were
    sound: sounds::SoundWorld,
    /// the message, input, menu and event triggers' side (play_hints.rs, play_panels.rs): the
    /// HUD's hint lines, the virtual buttons held this frame, this frame's AI events, the panels
    hints: hints::HintWorld,
    /// the sim time (s), for the log and BF_TRIGGER_FIRE
    clock: f32,
    /// how deep the events in flight are nested (MAX_DEPTH)
    depth: u32,
    /// BF_TRIGGER_LOG is set
    log: bool,
    /// names for the log (`names`; filled only with BF_TRIGGER_LOG)
    names: HashMap<u32, String>,
    /// the level's start has been sent to its script-triggers
    started: bool,
    /// BF_TRIGGER_FIRE's events
    fires: Vec<Fire>,
    /// characters whose death has been sent, by name
    dead_seen: HashSet<u32>,
    /// the named breakable level objects someone listens to: (object index, name)
    watched: Vec<(usize, u32)>,
    /// the watched objects whose DESTROYED has been sent, by object index
    broken_seen: HashSet<usize>,
    /// copies named so far (MADE_UP_COPY + n)
    copies_made: u32,
    /// NPCs spawned so far (NPC_IDS + n)
    npcs_made: u32,
    /// the effect copies spawned, by copy name (a destroyer removes them)
    effects: HashMap<u32, Vec<Entity>>,
    /// the MAX_DEPTH warning has been printed
    deep_told: bool,
    /// a mission, not a multiplayer map: the session's level kind 0x17506391, which
    /// FUN_00209900's member check needs
    mission: bool,
    /// the controlled character's type (the player slots' check in 0x83ee0: a member a player
    /// controls is kept whatever its slot says)
    player_kind: u32,
    /// the campaign's state (play_flags.rs: the game flags, the squad slots, the cash), held here
    /// while `run` runs a mission's triggers (the gamevar and score triggers and the member check
    /// use it) and handed back after; an empty one between frames, and all along on a
    /// multiplayer map (no flags, no cash: see `run`)
    campaign: Campaign,
    /// each spawn template's count (the placement's +0x24 short, FUN_000dfe90): one per
    /// spawn-trigger that names it, -1 once a re-armable one does; a spawn (FUN_000dfd90) clones
    /// the template while the count, less one, isn't 0 (or it's -1), else takes the template
    /// itself. Only the member flags tell the two apart here (a clone's set-up sets its flag)
    template_uses: HashMap<u32, i32>,
    /// the random numbers' seed (FUN_00233b90's DAT_0046cfd8): the gamevar's random operands
    seed: u32,
    /// what the script and letterbox triggers ask of the cutscenes this frame (play_script.rs
    /// `step` takes them)
    cutscene: Vec<script::Request>,
    /// what the level-over and zone triggers ask of the mission flow (play_level_over.rs,
    /// play_mission.rs takes it)
    flow: level_over::Flow,
}

/// A map starts: its triggers as the level file has them, the listeners linked (FUN_001febb0),
/// the squad member flags set up, the test hooks read.
fn start(mut commands: Commands, map: Res<MapLevel>, game: Res<GameData>, spawns: Option<Res<SquadSpawns>>, current: Res<CurrentMap>) {
    commands.insert_resource(TriggerEvents::default());
    let Some(level) = map.0.as_ref() else {
        commands.insert_resource(Triggers::default());
        return;
    };
    let log = std::env::var("BF_TRIGGER_LOG").is_ok();
    let squad_spawns = spawns.as_deref().map_or(&[][..], |s| &s.0[..]);
    let mut tr = Triggers { log, mission: !damage::multiplayer_map(&game.0, &current.0), seed: RANDOM_SEED, ..Default::default() };
    // a template that's a squad-type character the demo doesn't have in its squad: the member
    // FUN_00209900 checks
    let member_of = |template: u32| {
        let o = level.objects.iter().find(|o| o.name == template && o.tag == h("character-object"))?;
        let squad = game.0.character_type(o.kind).is_some_and(|t| t.player_type);
        (squad && !squad_spawns.iter().any(|s| s.template == template)).then_some(o.kind)
    };
    for t in &level.triggers {
        let state = match &t.class {
            Class::Router { filters, out_signal } => ClassState::Router { filters: *filters, out_signal: *out_signal },
            Class::Sound(s) => ClassState::Sound(Box::new(sounds::Sound::new(t, s, level, &game.0))),
            Class::Dialog { dialog, hush } => ClassState::Dialog(sounds::Dialog::new(*dialog, *hush, &game.0)),
            Class::Message { lines } => ClassState::Message(*lines),
            Class::Input { vb, .. } => ClassState::Input { vb: *vb },
            Class::Menu { menu, title, message } => ClassState::Menu(panels::MenuTrigger::new(*menu, *title, *message, &game.0)),
            Class::Event { event, filter } => ClassState::Event { event: *event, filter: *filter },
            Class::Timer { running, looping, stops, initial, current } => ClassState::Timer { running: *running, looping: *looping, stops: *stops,
                                                                                             initial: *initial, current: *current, subject: 0 },
            Class::Calculator { counter, target, test } => ClassState::Calc { counter: *counter, target: *target, test: *test },
            Class::Spawn { template, copy_name, fade, delay, rearm, member_check, .. } => ClassState::Spawn {
                armed: true, rearm: *rearm, pending: 0.0, delay: *delay, template: *template, copy_name: *copy_name, fade: *fade, copy: 0,
                member: member_of(*template), check: *member_check },
            Class::Detector { volume, filter, keep_when_disabled, .. } => ClassState::Detector { volume: *volume, filter: filter.clone(), keep: *keep_when_disabled,
                                                                                                occupants: vec![] },
            Class::Anim => ClassState::Anim { backward: true },
            Class::Destroyer => ClassState::Destroyer { pending: false },
            Class::Gamevar { op, operands } => ClassState::Gamevar { op: *op, operands: operands.clone() },
            Class::Score { amount, key, keyed, scaled, .. } => ClassState::Score { amount: *amount, key: *key, keyed: *keyed, scaled: *scaled },
            Class::LevelOver { ends_mission } => ClassState::LevelOver { ends_mission: *ends_mission },
            Class::Zone { next } => ClassState::Zone { next: next.clone() },
            Class::Other => cutscene_class(t),
        };
        // FUN_000dfe90 as the spawn-trigger links: its template counted once more, or -1 for
        // good from a re-armable one
        if let Class::Spawn { template, rearm, .. } = &t.class {
            let uses = tr.template_uses.entry(*template).or_insert(0);
            if *uses != -1 {
                *uses = if *rearm { -1 } else { *uses + 1 };
            }
        }
        if t.name != 0 {
            tr.by_name.entry(t.name).or_insert(tr.live.len());
        }
        tr.live.push(Live { t: t.clone(), enabled: t.enabled && !t.dead, dead: t.dead, die_once: t.die_after_event, disable_once: t.disable_after_event,
                            first_done: false, state });
    }
    // the listeners: a trigger listed is linked directly (a spawn-trigger: its copies), any
    // other object takes the lister as its listener
    for (i, l) in tr.live.iter().enumerate() {
        for &o in &l.t.objects {
            match tr.by_name.get(&o) {
                Some(&k) if matches!(tr.live[k].state, ClassState::Spawn { .. }) => tr.copy_listeners.entry(k).or_default().push(i),
                Some(_) => {}
                None => tr.listeners.entry(o).or_default().push(i),
            }
        }
    }
    tr.known = level.objects.iter().map(|o| o.name).filter(|&n| n != 0).collect();
    // the squad started at its spawns (play.rs `player_at_start`): their copies are those
    // spawn-triggers', so their deaths reach the triggers listening to them
    for s in spawns.as_deref().map_or(&[][..], |s| &s.0[..]) {
        if let Some(&k) = tr.by_name.get(&s.trigger) {
            tr.copy_of.insert(s.copy, k);
        }
    }
    if let Some(a) = bf_viewer::arena::arena() {
        tr.watched = level.objects.iter().enumerate()
            .filter(|(i, o)| o.name != 0 && tr.listeners.contains_key(&o.name) && a.breakable(*i).is_some()).map(|(i, o)| (i, o.name)).collect();
    }
    tr.fires = read_fires();
    if log {
        tr.names = names(&game.0);
        let mut counts: HashMap<String, usize> = HashMap::new();
        for l in &tr.live {
            *counts.entry(data::class_name(l.t.tag)).or_default() += 1;
        }
        let mut counts: Vec<_> = counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("triggers: {} ({})", tr.live.len(), counts.iter().map(|(k, n)| format!("{n} {k}")).collect::<Vec<_>>().join(", "));
        for s in squad_spawns {
            let from = if s.trigger == 0 { "placed in the world".to_string() } else { format!("from {}", tr.label(s.trigger)) };
            let weapons = s.weapons.as_ref().map_or("its type's".to_string(), |w| w.iter().map(|&k| game.0.weapons.get(&k).map_or(format!("h_{k:08x}"), |d| d.label.clone()))
                .collect::<Vec<_>>().join(", "));
            println!("squad spawn: {} as {} {from} at {:.2} (on the ground) facing {:.0} deg, weapons {weapons}", CHARACTERS[s.slot], tr.label(s.copy), s.feet,
                     s.yaw.to_degrees());
        }
        if !tr.mission {
            println!("squad member check: none (a multiplayer map)");
        }
        for l in &tr.live {
            if let (ClassState::Detector { volume, filter, .. }, Some(m)) = (&l.state, l.t.transform) {
                let what = match volume { Volume::Plate(s) => format!("plate +-{:.1} x {:.1} x {:.1}", s.x, s.y, s.z), Volume::Sphere(r) => format!("sphere r {r}") };
                println!("detector {} at {:.1}: {what}, filter {}{}", tr.label(l.t.name), m.w_axis.truncate(), filter_text(filter, &tr),
                         if l.enabled { "" } else { " (disabled)" });
            }
        }
        for f in &tr.fires {
            println!("BF_TRIGGER_FIRE: {} to {}{} at {:.2} s", sig::name(f.signal), if f.object { "object " } else { "" }, tr.label(f.name), f.at);
        }
    }
    commands.insert_resource(tr);
}

/// A gamevar-trigger's operation, for the log (`bf::triggers::op`).
fn op_name(op: i64) -> &'static str {
    match op {
        op::EQUAL => "A == B",
        op::LESS => "A < B",
        op::LESS_EQUAL => "A <= B",
        op::GREATER => "A > B",
        op::GREATER_EQUAL => "A >= B",
        op::ASSIGN => "A",
        op::DEFAULT => "the default",
        op::ADD => "A + B",
        op::SUBTRACT => "A - B",
        op::MULTIPLY => "A x B",
        op::DIVIDE => "A / B",
        _ => "an unknown operation",
    }
}

/// A filter, for the log.
fn filter_text(f: &Filter, tr: &Triggers) -> String {
    let mut parts = vec![];
    if f.squad_types { parts.push("squad types".to_string()); }
    if f.other_types { parts.push("other types".to_string()); }
    if f.characters { parts.push("any character".to_string()); }
    if f.human { parts.push("a human player's".to_string()); }
    if f.alignment != -1 { parts.push(format!("team {}", f.alignment)); }
    if !f.objects.is_empty() { parts.push(format!("objects {}", f.objects.iter().map(|&o| tr.label(o)).collect::<Vec<_>>().join(" "))); }
    if !f.object_types.is_empty() { parts.push(format!("types {}", f.object_types.iter().map(|&o| tr.label(o)).collect::<Vec<_>>().join(" "))); }
    if parts.is_empty() { parts.push("anything".into()); }
    format!("{}{}", if f.invert { "not " } else { "" }, parts.join(", "))
}

/// The names the log uses: the scripts' (`Game::script_names`), then the editor's families
/// hashed (NAME_FAMILIES 0-999), the squad's own.
fn names(game: &Game) -> HashMap<u32, String> {
    let mut n = game.script_names.clone();
    for fam in NAME_FAMILIES {
        for k in 0..1000 {
            let s = format!("{fam}{k}");
            n.entry(h(&s)).or_insert(s);
        }
    }
    for s in ["tex", "brutus", "hawk", "flint", "tex_0", "tex_1", "brutus_0", "brutus_1", "hawk_0", "hawk_1", "flint_0", "flint_1",
              "inventory-object", "weapon-object", "game-object", "character-object", "effect-object", "world-point"] {
        n.entry(h(s)).or_insert(s.to_string());
    }
    n
}

/// BF_TRIGGER_FIRE's entries (see the module notes).
fn read_fires() -> Vec<Fire> {
    let Ok(v) = std::env::var("BF_TRIGGER_FIRE") else { return vec![] };
    v.split(';').filter_map(|e| {
        let parts: Vec<&str> = e.split(',').map(str::trim).collect();
        let first = *parts.first()?;
        let (name, object) = match first.strip_prefix('@') { Some(n) => (n, true), None => (first, false) };
        let name = match name.strip_prefix("h_") { Some(x) => u32::from_str_radix(x, 16).ok()?, None if !name.is_empty() => h(name), None => return None };
        let signal = sig::parse(parts.get(1)?)?;
        let at = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        Some(Fire { at, object, name, signal, done: false })
    }).collect()
}

/// The characters the detectors can see: the living player and squad (the test map's NPCs and
/// the ones the triggers spawned among them) in the world (a squad member waiting for its
/// spawn-trigger isn't yet: `Player::absent`, #222). One no trigger named gets a name of its own
/// for the detectors' occupant lists (MADE_UP_UNIT + its squad slot, or + 100 + its NPC number).
fn actors(player: &Player, squad: &Squad, game: &Game) -> Vec<Actor> {
    std::iter::once((player, true)).chain(squad.0.iter().map(|m| (m, false))).filter(|(u, _)| !u.dead && !u.stand_in && !u.absent).map(|(u, human)| {
        let name = if u.trigger_name != 0 { u.trigger_name } else {
            match u.who() {
                Who::Squad(c) => MADE_UP_UNIT + c as u32,
                Who::Npc(n, _) => MADE_UP_UNIT + 100 + n,
            }
        };
        Actor { name, kind: u.kind, squad_type: game.character_type(u.kind).is_some_and(|t| t.player_type), human, team: i64::from(u.team),
                feet: u.position + Vec3::Y * GROUND, height: BODY_HEIGHT, radius: BODY_RADIUS }
    }).collect()
}

/// FUN_001fb980: every part of a detector's filter passes (each as its function has it, all
/// actors being characters with a brain): the team (FUN_001fb770), the other types
/// (FUN_001fb810), the squad types (FUN_001fb850), a human player's (FUN_001fb7d0), any character
/// (FUN_001fb890), the named objects (FUN_001fb8b0) and the object types (FUN_001fb910); each
/// turned over by h_ebfd2b20.
fn passes(f: &Filter, a: &Actor) -> bool {
    let test = |on: bool, yes: bool| !on || (yes != f.invert);
    (f.alignment == -1 || ((a.team == f.alignment) != f.invert))
        && test(f.other_types, !a.squad_type)
        && test(f.squad_types, a.squad_type)
        && test(f.human, a.human)
        && test(f.characters, true)
        && test(!f.objects.is_empty(), f.objects.contains(&a.name))
        && test(!f.object_types.is_empty(), f.object_types.contains(&a.kind))
}

/// Whether a character's body (an upright cylinder of `radius` from its feet to `height`, the
/// demo's: the game tests its Ipion body against the detector's phantom) meets a detector's
/// volume at `at`: a plate's box of half-extents `size` in its frame (sampled along the body,
/// each point within the box grown by the radius), a proximity detector's sphere (the body's
/// axis within the radius plus the body's).
fn inside(volume: Volume, at: &Mat4, inv: &Mat4, a: &Actor) -> bool {
    let (lo, hi) = (a.feet + Vec3::Y * a.radius.min(a.height * 0.5), a.feet + Vec3::Y * (a.height - a.radius).max(a.height * 0.5));
    match volume {
        Volume::Plate(size) => (0..=4).any(|k| {
            let p = inv.transform_point3(lo.lerp(hi, k as f32 / 4.0));
            p.x.abs() <= size.x + a.radius && p.y.abs() <= size.y + a.radius && p.z.abs() <= size.z + a.radius
        }),
        Volume::Sphere(r) => {
            let c = at.w_axis.truncate();
            let y = c.y.clamp(lo.y, hi.y);
            Vec3::new(a.feet.x, y, a.feet.z).distance(c) <= r + a.radius
        }
    }
}

/// The triggers' frame (see the module notes): the world's events (deaths, broken objects,
/// panels, pickups, the test hook's), each trigger's update in the level file's order, then what
/// they asked of the world.
#[allow(clippy::too_many_arguments)]
fn run(mut commands: Commands, time: Res<Time>, mut tr: ResMut<Triggers>, mut events: ResMut<TriggerEvents>, mut player: ResMut<Player>,
       mut squad: ResMut<Squad>, mut game: ResMut<GameData>, map: Res<MapLevel>, mut doors: Option<ResMut<Doors>>, spawns: Option<Res<SquadSpawns>>,
       mut fx: (Option<ResMut<bf_viewer::ale_fx::AleAssets>>, ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
       mut placed: Query<(&Placed, &mut Visibility)>, mut hits: ResMut<AiHits>, mut campaign: ResMut<Campaign>,
       mut cutscenes: ResMut<script::Cutscenes>) {
    let Some(level) = map.0.as_ref() else { return };
    if tr.live.is_empty() {
        events.0.clear();
        return;
    }
    let dt = frame_dt(&time);
    tr.clock = player.sim_time;
    tr.player_kind = player.kind;
    // the campaign's state, held while the triggers run (handed back at the end of `run`: no
    // return in between). Only in a mission: a multiplayer map's triggers keep their own empty
    // one, no flags (their gamevars read 0 and write nothing, as on a flag the tree lacks) and no
    // cash. That a match has none of the campaign's flags is inferred: the session's difficulty
    // getter reads 0 in a split-screen match (live, #129), which it does only without a tree or
    // without the flag (0x7c720), where the campaign's default flags would give -4. (No
    // multiplayer map's gamevar names a flag in the data: this only keeps a match off the
    // campaign's state.)
    let mission = tr.mission;
    if mission {
        std::mem::swap(&mut tr.campaign, &mut *campaign);
        tr.campaign.player_kind = player.kind;
    }
    // the level's start: FIRST_UPDATE to every script-trigger, then its update (FUN_001fe260)
    if !tr.started {
        tr.started = true;
        // (a squad made at the map's start from now on is in the world at once, play_script.rs
        // `spawns_wait`: the map editor's Shift+F5)
        script::level_started();
        for i in 0..tr.live.len() {
            if tr.live[i].t.tag == h("script-trigger") {
                tr.deliver(i, Ev::new(sig::FIRST_UPDATE, 0));
                tr.update(i, 0.0, &[]);
            }
        }
    }
    // deaths: a character's DESTROYED to its listeners (its spawn-trigger's)
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        if u.dead && u.trigger_name != 0 && tr.dead_seen.insert(u.trigger_name) {
            let n = u.trigger_name;
            tr.object_event(n, Ev { signal: sig::DESTROYED, id: 0, subject: n, data: -1 });
        }
    }
    // named breakable objects broken (play_scenery.rs breaks them)
    if let Some(a) = bf_viewer::arena::arena() {
        let broke: Vec<(usize, u32)> = tr.watched.iter().copied().filter(|(i, _)| a.breakable(*i).is_some_and(|b| a.is_broken(b))).collect();
        for (i, n) in broke {
            if tr.broken_seen.insert(i) {
                tr.object_event(n, Ev { signal: sig::DESTROYED, id: 0, subject: n, data: -1 });
            }
        }
    }
    for e in std::mem::take(&mut events.0) {
        tr.object_event(e.from, Ev { signal: e.signal, id: 0, subject: e.subject, data: e.data });
    }
    // test hook: BF_TRIGGER_FIRE
    let now = tr.clock;
    let due: Vec<(bool, u32, i64)> = tr.fires.iter_mut().filter(|f| !f.done && now >= f.at).map(|f| { f.done = true; (f.object, f.name, f.signal) }).collect();
    for (object, name, signal) in due {
        if object {
            tr.object_event(name, Ev::new(signal, 0));
        } else if let Some(&i) = tr.by_name.get(&name) {
            if tr.log {
                println!("t {now:.2}: BF_TRIGGER_FIRE {} to {} ({})", sig::name(signal), tr.label(name), data::class_name(tr.live[i].t.tag));
            }
            tr.deliver(i, Ev::new(signal, 0));
        } else {
            println!("BF_TRIGGER_FIRE: no trigger {} on this level", tr.label(name));
        }
    }
    let actors = actors(&player, &squad, &game.0);
    for i in 0..tr.live.len() {
        tr.update(i, dt, &actors);
    }
    // what they asked of the world, then the cutscenes' frame (#222, play_script.rs: the scripts
    // the triggers started, their events due now), which stops after an event whose signal asked
    // something of the world (a 103 to a spawn- or warp-trigger) for that to be done before its
    // next event: the game's signals act at once (a spawn's copy is in the world, and bound to
    // the scripts, FUN_002096e0, as its 103 returns: the opening's 119 on tex_1 at 17.106 s holds
    // the Tex its 103 at 17.091 s spawned, in the same frame)
    let none = SquadSpawns::default();
    let spawns = spawns.as_deref().unwrap_or(&none);
    let mut pass = script::Pass::First;
    loop {
        let acts = std::mem::take(&mut tr.acts);
        for a in acts {
            match a {
                Act::Anim { trigger, object, clip, backward, rate, command } => {
                    let Some(d) = doors.as_deref_mut().and_then(|d| d.list.iter_mut().find(|d| d.door.name == object)) else {
                        if tr.log {
                            println!("t {now:.2}: {}: clip h_{clip:08x} on {}: not an object the demo animates", tr.label(tr.live[trigger].t.name), tr.label(object));
                        }
                        continue;
                    };
                    if command != data::AC_PLAY && command != data::AC_RESUME {
                        if tr.log {
                            println!("t {now:.2}: {}: anim command {command} on {}: not run (AC_PLAY and AC_RESUME only)", tr.label(tr.live[trigger].t.name), tr.label(object));
                        }
                        continue;
                    }
                    // (its sound is its sound-trigger's, which the anim-trigger's ANIM_START /
                    // ANIM_BACK_START reaches: play_sound_triggers.rs)
                    let opening = !backward;
                    d.opening = opening;
                    d.rate = rate;
                    if tr.log || std::env::var("BF_DOOR_LOG").is_ok() {
                        println!("t {now:.2}: door {} {} (clip h_{clip:08x} x{rate}) by {}", tr.label(object), if opening { "opens" } else { "closes" },
                                 tr.label(tr.live[trigger].t.name));
                    }
                }
                Act::Spawn { trigger, template, copy, at, fade } => {
                    spawn_copy(&mut commands, &mut tr, &mut player, &mut squad, &mut game.0, level, spawns, &mut fx, trigger, template, copy, at, fade);
                }
                Act::Destroy { trigger, object } => {
                    let by = tr.label(tr.live[trigger].t.name);
                    // (a character the triggers spawned; never a squad member the player can
                    // switch to: no level's destroyer lists one, and the demo's squad isn't the
                    // triggers' to take away)
                    if let Some(k) = squad.0.iter().position(|m| m.trigger_name == object && m.npc.is_some()) {
                        let mut m = squad.0.remove(k);
                        if let Some(l) = m.loaded.take() {
                            commands.entity(l.root).despawn();
                        }
                        if let Some(e) = m.slide_fx.take() {
                            commands.entity(e).despawn();
                        }
                        // (the shots on their way are counted by place in the squad)
                        hits.0.clear();
                        player.pending_hits.clear();
                        if tr.log {
                            println!("t {now:.2}: {by} removes {}", tr.label(object));
                        }
                    } else if let Some(list) = tr.effects.remove(&object) {
                        for e in list {
                            commands.entity(e).try_despawn();
                        }
                        if tr.log {
                            println!("t {now:.2}: {by} removes the effect {}", tr.label(object));
                        }
                    } else if std::iter::once(&*player).chain(squad.0.iter()).any(|m| m.trigger_name == object) {
                        if tr.log {
                            println!("t {now:.2}: {by}: {} is a squad member: not removed", tr.label(object));
                        }
                    } else if let Some(i) = level.objects.iter().position(|o| o.name == object) {
                        // a placed object: no longer drawn; a breakable's collision goes with it
                        for (p, mut v) in &mut placed {
                            if p.name == object {
                                *v = Visibility::Hidden;
                            }
                        }
                        if let Some(b) = bf_viewer::arena::arena().and_then(|a| a.breakable(i).map(|b| (a, b))) {
                            b.0.set_broken(b.1, true);
                        }
                        if tr.log {
                            println!("t {now:.2}: {by} removes the placed {} (h_{:08x})", tr.label(object), level.objects[i].kind);
                        }
                    } else if tr.log {
                        println!("t {now:.2}: {by}: nothing called {} to remove", tr.label(object));
                    }
                }
                Act::Warp { trigger } => warp(&mut tr, trigger, level, spawns, &mut player, &mut squad),
            }
        }
        if pass == script::Pass::Done {
            break;
        }
        pass = script::step(&mut cutscenes, &mut tr, &mut player, &mut squad, dt, pass);
    }
    if mission {
        std::mem::swap(&mut tr.campaign, &mut *campaign);
    }
}

/// A spawn-trigger's copy in the world (FUN_00209620): its template's kind decides what it is.
///  * a squad member's template (`SquadSpawns`): that member, moved to the spawn (the demo has
///    the squad in the world from the start: #102's start, see play.rs `player_at_start`); its
///    weapons are already its template's (`SquadSpawn::dress`). When the member isn't there to
///    move (the map editor's stand-in leads, or it was taken away) nothing is made: the squad
///    isn't doubled with an NPC of its own;
///  * any other character placement: a character of its type with the placement's weapons,
///    personality and team (as BF_TEST_NPC spawns a placement, #129), set to fight, facing the
///    trigger's way;
///  * an effect object: its effect type's ALE effects at the transform, once each (a looping one
///    that the game keeps until a destroyer removes it plays its emitters once here);
///  * anything else: not made (logged).
/// A character stands on the ground under the transform (`on_ground`: the place call's position
/// setter, 0x1154b0); an effect stays at the transform (a game object's place, 0x159b70, has no
/// such ray). The character's fade-in over `fade` s (char+0x734..0x73c, FUN_0011bd00) isn't
/// drawn: it appears at once.
#[allow(clippy::too_many_arguments)]
fn spawn_copy(commands: &mut Commands, tr: &mut Triggers, player: &mut Player, squad: &mut Squad, game: &mut Game, level: &Level, spawns: &SquadSpawns,
              fx: &mut (Option<ResMut<bf_viewer::ale_fx::AleAssets>>, ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
              trigger: usize, template: u32, copy: u32, at: Mat4, fade: f32) {
    let now = tr.clock;
    let by = tr.label(tr.live[trigger].t.name);
    let point = at.w_axis.truncate();
    let yaw = yaw_of(&at);
    // a squad member: moved there
    if let Some(s) = spawns.0.iter().find(|s| s.template == template) {
        let unit = if player.character == s.slot && player.in_squad() && !player.stand_in { Some(&mut *player) } else {
            squad.0.iter_mut().find(|m| m.character == s.slot && m.in_squad())
        };
        let Some(u) = unit else {
            if tr.log {
                println!("t {now:.2}: {by} spawns {} ({}): that squad member isn't in the world to move: nothing made", tr.label(copy), CHARACTERS[s.slot]);
            }
            return;
        };
        let feet = on_ground(point);
        u.position = feet - Vec3::Y * GROUND;
        u.prev_xz = feet.xz();
        u.fall_from = feet.y;
        u.yaw = yaw;
        u.cam_yaw = yaw;
        u.vy = 0.0;
        u.height = 0.0;
        u.trigger_name = copy;
        u.cam_target = u.position + Vec3::new(0.0, 0.35, 0.0);
        // (in the world now, if it was waiting for this: play.rs `player_at_start`, #222)
        let came = std::mem::take(&mut u.absent);
        if tr.log {
            println!("t {now:.2}: {by} spawns {} ({}) at {feet:.2} (the trigger's {:.2} up) facing {:.0} deg (the squad member {}; its {fade} s \
                      fade-in not drawn)", tr.label(copy), CHARACTERS[s.slot], point.y - feet.y, yaw.to_degrees(),
                     if came { "comes into the world there" } else { "moved there" });
        }
        return;
    }
    let Some(o) = level.objects.iter().find(|o| o.name == template) else {
        if tr.log {
            println!("t {now:.2}: {by}: its template {} isn't on this level", tr.label(template));
        }
        return;
    };
    if o.tag == h("character-object") {
        let kind = o.kind;
        let Some(t) = game.character_type(kind).cloned() else {
            if tr.log {
                println!("t {now:.2}: {by}: no character type h_{kind:08x} loaded for {}", tr.label(template));
            }
            return;
        };
        let placement = placement_of(game, o).cloned();
        let feet = on_ground(point);
        let mut m = Player::new((0..SQUAD).find(|&c| squad_type(c) == kind).unwrap_or(OTHER));
        m.kind = kind;
        m.position = feet - Vec3::Y * GROUND;
        m.prev_xz = feet.xz();
        m.fall_from = feet.y;
        m.yaw = yaw;
        m.cam_yaw = yaw;
        m.face_yaw = Some(yaw);
        m.team = placement.as_ref().map_or(t.ai.alignment, |p| p.team(Some(&t))).clamp(0, 9) as u8;
        tr.npcs_made += 1;
        let id = NPC_IDS + tr.npcs_made;
        m.npc = Some(testtools::Npc { id, fight: true, face: yaw });
        m.rng ^= id.wrapping_mul(0x9E37_79B9) | 1;
        if let Some(pl) = &placement {
            m.enemy.weapons = Some(pl.weapons.iter().map(|w| w.0).collect());
            m.enemy.personality = pl.ai.personalities[0];
        }
        m.surface = usize::MAX;
        m.trigger_name = copy;
        if tr.log {
            println!("t {now:.2}: {by} spawns {} ({}, team {}) at {feet:.2} (the trigger's {:.2} up) facing {:.0} deg{}", tr.label(copy), game.type_label(kind),
                     m.team, point.y - feet.y, yaw.to_degrees(), if fade > 0.0 { format!(" (its {fade} s fade-in not drawn)") } else { String::new() });
        }
        squad.0.push(m);
        return;
    }
    let effects = game.object_types.get(&o.kind).filter(|k| k.object_type == OBJECT_EFFECT).and_then(|_| game.effect_types.get(&o.kind)).cloned();
    match (effects, fx.0.as_deref_mut()) {
        (Some(list), Some(ale)) => {
            let mut made = vec![];
            for effect in list {
                if let Some(e) = ale.load(game, &mut fx.1, &mut fx.2, effect) {
                    let life = e.duration();
                    let seed = (point.x * 977.0 + point.z * 131.0) as u32 ^ copy;
                    made.push(commands.spawn((Transform::from_matrix(at), Visibility::default(), bf_viewer::ale_fx::AleEffect::once(e, 0.0, seed),
                                              AleExpire(life), Name::new(format!("trigger effect h_{effect:08x}")))).id());
                }
            }
            if tr.log {
                println!("t {now:.2}: {by} spawns {} (effect h_{:08x}: {} ALE effects) at {point:.2}", tr.label(copy), o.kind, made.len());
            }
            tr.effects.entry(copy).or_default().extend(made);
        }
        _ => if tr.log {
            println!("t {now:.2}: {by}: its template {} ({}, h_{:08x}) isn't a character or an effect: not made", tr.label(template), tr.label(o.tag), o.kind);
        },
    }
}

impl Triggers {
    /// A name for the log (see `names`).
    fn label(&self, n: u32) -> String {
        match self.names.get(&n) {
            Some(s) => s.clone(),
            None if n == 0 => "-".into(),
            None => format!("h_{n:08x}"),
        }
    }

    /// A trigger's name and class, for the log.
    fn who(&self, i: usize) -> String {
        format!("{} ({})", self.label(self.live[i].t.name), data::class_name(self.live[i].t.tag))
    }

    /// An event's subject and data, for the log.
    fn about(&self, ev: &Ev) -> String {
        let mut s = String::new();
        if ev.subject != 0 {
            s += &format!(" [{}]", self.label(ev.subject));
        }
        if ev.data != -1 {
            s += &format!(" [data {}]", ev.data);
        }
        s
    }

    /// Which class trigger `i` runs as (its state's).
    fn kind(&self, i: usize) -> Kind {
        match self.live[i].state {
            ClassState::Other => Kind::Other,
            ClassState::Sound(_) => Kind::Sound,
            ClassState::Dialog(_) => Kind::Dialog,
            ClassState::Message(_) => Kind::Message,
            ClassState::Input { .. } => Kind::Input,
            ClassState::Menu(_) => Kind::Menu,
            ClassState::Event { .. } => Kind::Event,
            ClassState::Router { .. } => Kind::Router,
            ClassState::Timer { .. } => Kind::Timer,
            ClassState::Calc { .. } => Kind::Calc,
            ClassState::Spawn { .. } => Kind::Spawn,
            ClassState::Detector { .. } => Kind::Detector,
            ClassState::Anim { .. } => Kind::Anim,
            ClassState::Destroyer { .. } => Kind::Destroyer,
            ClassState::Gamevar { .. } => Kind::Gamevar,
            ClassState::Score { .. } => Kind::Score,
            ClassState::Script { .. } => Kind::Script,
            ClassState::Letterbox { .. } => Kind::Letterbox,
            ClassState::Warp { .. } => Kind::Warp,
            ClassState::LevelOver { .. } => Kind::LevelOver,
            ClassState::Zone { .. } => Kind::Zone,
        }
    }

    /// FUN_001feb60: the next of trigger `i`'s entries from `from` that answers `ev`: the same
    /// signal and match-id, used fewer times than its limit.
    fn next_entry(&self, i: usize, ev: &Ev, from: usize) -> Option<usize> {
        self.live[i].t.entries.iter().enumerate().skip(from)
            .find(|(_, e)| e.signal == ev.signal && e.match_id == ev.id && e.used < e.limit).map(|(j, _)| j)
    }

    /// 0x1fefc0: on (unless dead), every entry's use count back to 0; +0x3d bit 6 too, which
    /// puts a sound-trigger's update back in the world's loop (`sounds::Sound::listed`).
    fn enable(&mut self, i: usize) {
        let l = &mut self.live[i];
        if l.dead {
            return;
        }
        l.enabled = true;
        for e in &mut l.t.entries {
            e.used = 0;
        }
        if let ClassState::Sound(s) = &mut l.state {
            s.listed = true;
        }
    }

    /// 0x1feff0: off. A sound-trigger's own (its vtable +0x48, FUN_00206610) lets its voice play
    /// out and drops what it was waiting for (`sound_disabled`).
    fn disable(&mut self, i: usize) {
        self.live[i].enabled = false;
        if matches!(self.live[i].state, ClassState::Sound(_)) {
            self.sound_disabled(i);
        }
    }

    /// FUN_001ff0a0: `ev` to trigger `i`'s targets, while it's enabled and alive; then
    /// FUN_001ff000 for it.
    fn send(&mut self, i: usize, ev: Ev) {
        if self.live[i].dead || !self.live[i].enabled {
            return;
        }
        let targets = self.live[i].t.targets.clone();
        if self.log {
            let to: Vec<String> = targets.iter().map(|&t| self.label(t)).collect();
            println!("t {:.2}: {} sends {}{} to {}", self.clock, self.who(i), sig::name(ev.signal), self.about(&ev),
                     if to.is_empty() { "no one".to_string() } else { to.join(", ") });
        }
        for t in targets {
            if let Some(&k) = self.by_name.get(&t) {
                self.deliver(k, ev);
            }
        }
        self.after_event(i, &ev);
    }

    /// FUN_001ff000: after an event (not FIRST_UPDATE), h_f657f799 kills the trigger and
    /// h_1eec0e75 disables it, once each (the DESTROYED / DISABLED they send is lost: the
    /// trigger is off by then).
    fn after_event(&mut self, i: usize, ev: &Ev) {
        if ev.signal == sig::FIRST_UPDATE {
            return;
        }
        if self.live[i].die_once {
            self.live[i].die_once = false;
            self.live[i].dead = true;
            self.disable(i);
            if self.log {
                println!("t {:.2}: {} is spent (h_f657f799)", self.clock, self.who(i));
            }
        }
        if self.live[i].disable_once {
            self.live[i].disable_once = false;
            self.disable(i);
            if self.log {
                println!("t {:.2}: {} disables itself (h_1eec0e75)", self.clock, self.who(i));
            }
        }
    }

    /// FUN_001feac0: entry `j` of trigger `i` has acted on `ev`: one more use; spent with
    /// h_170a9b80, or any entry acting with h_137926ff: DISABLED to the targets, and off.
    fn entry_used(&mut self, i: usize, j: usize, ev: &Ev) {
        let e = &mut self.live[i].t.entries[j];
        e.used += 1;
        let spent = e.disable_when_spent && e.used >= e.limit;
        if spent || self.live[i].t.disable_after_entry {
            let id = self.live[i].t.id;
            self.send(i, ev.with(sig::DISABLED, id));
            self.disable(i);
            if self.log {
                println!("t {:.2}: {} disables itself ({})", self.clock, self.who(i), if spent { "its entry spent" } else { "h_137926ff" });
            }
        }
    }

    /// FUN_001ff0f0: trigger `i`'s base actions on `ev` (nothing while dead).
    fn base_actions(&mut self, i: usize, ev: &Ev) {
        if self.live[i].dead {
            return;
        }
        let id = self.live[i].t.id;
        let mut k = 0;
        while let Some(j) = self.next_entry(i, ev, k) {
            k = j + 1;
            let action = self.live[i].t.entries[j].action;
            let what = match action {
                act::ENABLE => {
                    self.enable(i);
                    self.send(i, ev.with(sig::ENABLED, id));
                    "enabled"
                }
                act::DISABLE => {
                    self.disable(i);
                    self.send(i, ev.with(sig::DISABLED, id));
                    "disabled"
                }
                act::TOGGLE_ENABLE => {
                    if self.live[i].enabled {
                        self.disable(i);
                        self.send(i, ev.with(sig::DISABLED, id));
                        "disabled (toggled)"
                    } else {
                        self.enable(i);
                        self.send(i, ev.with(sig::ENABLED, id));
                        "enabled (toggled)"
                    }
                }
                a if (act::USER_SIG1..=act::USER_SIG8).contains(&a) => {
                    if !self.live[i].enabled {
                        continue;
                    }
                    let n = a - act::USER_SIG1;
                    let signal = if n < 2 { sig::USER_SIGNAL1 + n } else { sig::USER_SIGNAL3 + n - 2 };
                    self.send(i, ev.with(signal, id));
                    ""
                }
                _ => continue,
            };
            if self.log && !what.is_empty() {
                println!("t {:.2}: {} {what} on {}", self.clock, self.who(i), sig::name(ev.signal));
            }
            self.entry_used(i, j, ev);
        }
    }

    /// A disabled trigger's answer to an event, as every class's handler has it: its base
    /// actions, then FUN_001ff000.
    fn base_and_after(&mut self, i: usize, ev: &Ev) {
        self.base_actions(i, ev);
        self.after_event(i, ev);
    }

    /// An event to trigger `i`: its class's handler (vtable +0x60).
    fn deliver(&mut self, i: usize, ev: Ev) {
        if self.depth >= MAX_DEPTH {
            if !self.deep_told {
                self.deep_told = true;
                println!("triggers: events nested {MAX_DEPTH} deep at {}: dropped", self.who(i));
            }
            return;
        }
        self.depth += 1;
        match self.kind(i) {
            Kind::Sound => self.sound(i, ev),
            Kind::Dialog => self.dialog(i, ev),
            Kind::Message => self.message(i, ev),
            Kind::Input | Kind::Event => self.counts_operate(i, ev),
            Kind::Menu => self.menu(i, ev),
            Kind::Router => self.router(i, ev),
            Kind::Timer => self.timer(i, ev),
            Kind::Calc => self.calculator(i, ev),
            Kind::Spawn => self.spawner(i, ev),
            // (a detector's handler is the base's itself: FUN_001ff0f0, vtable 0x3965a8 +0x60)
            Kind::Detector => self.base_actions(i, &ev),
            Kind::Anim => self.anim(i, ev),
            Kind::Destroyer => self.destroyer(i, ev),
            Kind::Gamevar => self.gamevar(i, ev),
            Kind::Score => self.score(i, ev),
            Kind::Script | Kind::Letterbox | Kind::Warp => self.cutscene_trigger(i, ev),
            Kind::LevelOver => self.level_over(i, ev),
            Kind::Zone => self.zone(i, ev),
            Kind::Other => self.other(i, ev),
        }
        self.depth -= 1;
    }

    /// An object's event to the triggers listening to it (FUN_001fe3d0): those that list it,
    /// and for a spawn copy those that list its spawn-trigger.
    fn object_event(&mut self, from: u32, ev: Ev) {
        let mut to: Vec<usize> = self.listeners.get(&from).cloned().unwrap_or_default();
        if let Some(t) = self.copy_of.get(&from) {
            to.extend(self.copy_listeners.get(t).cloned().unwrap_or_default());
        }
        if self.log {
            let list: Vec<String> = to.iter().map(|&k| self.label(self.live[k].t.name)).collect();
            println!("t {:.2}: object {} sends {}{} to {}", self.clock, self.label(from), sig::name(ev.signal), self.about(&ev),
                     if list.is_empty() { "no one".to_string() } else { list.join(", ") });
        }
        for k in to {
            self.deliver(k, ev);
        }
    }

    /// 0x208fc0: a router.
    fn router(&mut self, i: usize, ev: Ev) {
        if ev.signal == sig::FIRST_UPDATE {
            return;
        }
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let ClassState::Router { filters, out_signal } = self.live[i].state else { return };
        if !filters {
            return self.send(i, ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            self.send(i, if out_signal == -1 { ev } else { Ev { signal: out_signal, ..ev } });
            self.entry_used(i, j, &ev);
        }
    }

    /// 0x200190: a timer.
    fn timer(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let action = self.live[i].t.entries[j].action;
            let ClassState::Timer { running, current, initial, subject, .. } = &mut self.live[i].state else { return };
            let what = match action {
                act::OPERATE => {
                    if ev.subject != 0 { *subject = ev.subject; }
                    *running = true;
                    format!("starts ({current:.2} s left)")
                }
                act::TIMER_FULL_RESET => {
                    if ev.subject != 0 { *subject = ev.subject; }
                    *current = *initial;
                    *running = false;
                    "stops and reloads".into()
                }
                act::RESET_COUNTDOWN_TIME => {
                    if ev.subject != 0 { *subject = ev.subject; }
                    *current = *initial;
                    "reloads".into()
                }
                act::DISPLAY_TIMER | act::UNDISPLAY_TIMER => String::new(),
                _ => {
                    self.base_and_after(i, &ev);
                    continue;
                }
            };
            if self.log && !what.is_empty() {
                println!("t {:.2}: {} {what} on {}", self.clock, self.who(i), sig::name(ev.signal));
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// 0x1ff5f0: a calculator.
    fn calculator(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        if self.live[i].dead {
            return;
        }
        let id = self.live[i].t.id;
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let (action, step) = (self.live[i].t.entries[j].action, self.live[i].t.entries[j].value());
            let ClassState::Calc { counter, .. } = &mut self.live[i].state else { return };
            match action {
                act::INC_COUNTER => *counter += 1,
                act::DEC_COUNTER => *counter -= 1,
                act::MODIFY_COUNTER => *counter += step,
                act::SET_COUNTER_FROM_SIG_DATA => *counter = ev.data,
                _ => {
                    self.base_and_after(i, &ev);
                    continue;
                }
            }
            self.entry_used(i, j, &ev);
            let ClassState::Calc { counter, target, test } = self.live[i].state else { return };
            // (compared as unsigned: jae / jbe / ja / jb at 0x1ff6da-0x1ff702)
            let (c, t) = (counter as u32, target as u32);
            let pass = match test { 0 => c == t, 1 => c < t, 2 => c > t, 3 => c <= t, 4 => c >= t, _ => false };
            if self.log {
                println!("t {:.2}: {} {} on {}: {counter} (target {target}, test {test}): {}", self.clock, self.who(i), act::name(action),
                         sig::name(ev.signal), if pass { "true" } else { "false" });
            }
            let out = Ev { signal: sig::CALCULATOR, id, subject: ev.subject, data: counter };
            if pass {
                self.send(i, out);
                self.send(i, Ev { signal: sig::TRUE, ..out });
            } else {
                self.send(i, Ev { signal: sig::FALSE, ..out });
            }
        }
    }

    /// 0x209ac0: a spawn-trigger.
    fn spawner(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let action = self.live[i].t.entries[j].action;
            let ClassState::Spawn { armed, rearm, delay, .. } = self.live[i].state else { return };
            match action {
                act::OPERATE => {
                    if delay >= MIN_DELAY {
                        if let ClassState::Spawn { pending, .. } = &mut self.live[i].state {
                            *pending = delay;
                        }
                        if self.log {
                            println!("t {:.2}: {} spawns in {delay:.2} s", self.clock, self.who(i));
                        }
                    } else if armed {
                        self.spawn(i);
                    } else if self.log {
                        println!("t {:.2}: {} on {}: not armed (spawned already)", self.clock, self.who(i), sig::name(ev.signal));
                    }
                }
                act::SPAWN_RESET => {
                    if let (true, ClassState::Spawn { armed, .. }) = (rearm, &mut self.live[i].state) {
                        *armed = true;
                    }
                }
                _ => {
                    self.base_and_after(i, &ev);
                    continue;
                }
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// FUN_000dfd90's choice for a spawn of `template`: a clone (a new character, set up by
    /// FUN_0012f440) while its count less one isn't 0, or always for a count of -1 (a re-armable
    /// trigger's), else the template itself (`template_uses`).
    fn takes_clone(&mut self, template: u32) -> bool {
        match self.template_uses.get_mut(&template) {
            Some(n) if *n == -1 => true,
            Some(n) => {
                *n -= 1;
                *n != 0
            }
            None => true,
        }
    }

    /// FUN_00209900, on spawn-trigger `i`'s copy of a squad-type character (`member`) in a
    /// mission (and on the mission's roster, the world's +0xbec object's vtable +0x74, 0x8d430:
    /// taken to hold every squad type, an inference; and with the session's +0xcc byte 0, as it
    /// is in play). A clone has set its type's member flag on its set-up first (FUN_0012f440, the
    /// session's +0xbc). With h_0fa4f510 (`check`) the copy is kept if the session counts it in
    /// the squad (+0xb8, 0x83ee0: `Campaign::in_squad`, the save-game slots), else taken away at
    /// once: its +0x144(1), a death (FUN_00112830 clears its type's flag), its hit points to 0
    /// (FUN_0010f830), out of the world (its vtable +0xa0), and no SPAWNED goes out. Without
    /// h_0fa4f510 its type's flag is set (+0xbc) and it's kept. So the tutorial's three start
    /// marines (re-armable spawns: clones), whose save-game slots aren't alive, don't come with
    /// Tex's farcast (take03-09: never seen) and leave their flags 0; the V-squad's (h_0fa4f510
    /// false, take11) come and set theirs (take11's `1|0|1|1`).
    fn member_kept(&mut self, i: usize, member: Option<u32>, check: bool, clone: bool) -> bool {
        let Some(kind) = member.filter(|_| self.mission) else { return true };
        let (t, who) = (self.clock, self.label(self.live[i].t.name));
        if clone {
            self.campaign.set_member(kind, true, &format!("FUN_0012f440: {who}'s copy set up"), t);
        }
        if !check {
            self.campaign.set_member(kind, true, &format!("FUN_00209900: {who} has no h_0fa4f510"), t);
            return true;
        }
        let kept = self.campaign.in_squad(kind);
        if !kept {
            self.campaign.set_member(kind, false, &format!("FUN_00112830: {who}'s copy taken away"), t);
        }
        kept
    }

    /// FUN_002099e0: spawn-trigger `i` makes its copy: no longer armed; a template the level
    /// has gives a copy (a clone or the template itself, `takes_clone`), which sends SPAWNED to
    /// its listeners (FUN_002096e0) unless FUN_00209900 took it away (`member_kept`), and the
    /// trigger sends OPERATED with it to its targets.
    fn spawn(&mut self, i: usize) {
        let (template, fade, rearm, copy_name, member, check) = match &mut self.live[i].state {
            ClassState::Spawn { armed, rearm, template, copy_name, fade, member, check, .. } => {
                *armed = false;
                (*template, *fade, *rearm, *copy_name, *member, *check)
            }
            _ => return,
        };
        if template == 0 || !self.known.contains(&template) {
            if self.log {
                println!("t {:.2}: {}: its template {} isn't on this level: no copy", self.clock, self.who(i), self.label(template));
            }
            return;
        }
        let clone = self.takes_clone(template);
        // (a re-armable trigger names its copies nothing, FUN_00209620; they get one here)
        let copy = if !rearm && copy_name != 0 { copy_name } else {
            self.copies_made += 1;
            MADE_UP_COPY + self.copies_made
        };
        if let ClassState::Spawn { copy: current, .. } = &mut self.live[i].state {
            *current = copy;
        }
        self.copy_of.insert(copy, i);
        let Some(at) = self.live[i].t.transform else { return };
        if self.member_kept(i, member, check, clone) {
            self.acts.push(Act::Spawn { trigger: i, template, copy, at, fade });
            self.object_event(copy, Ev::new(sig::SPAWNED, 0));
        } else if self.log {
            println!("t {:.2}: {}: its copy {} ({}) is taken away at once: the session doesn't count it in the squad (FUN_00209900, 0x83ee0)",
                     self.clock, self.who(i), self.label(copy), self.campaign.path_name(&[member.unwrap_or(0)]));
        }
        self.send(i, Ev { signal: sig::OPERATED, id: 0, subject: copy, data: -1 });
    }

    /// 0x204d00: an anim-trigger.
    fn anim(&mut self, i: usize, ev: Ev) {
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
            self.play(i, j);
            self.entry_used(i, j, &ev);
        }
    }

    /// FUN_00204bb0: anim entry `j` of trigger `i` plays on its first object: ANIM_START when it
    /// starts forward after playing backward (or plays another mode), ANIM_BACK_START when it
    /// starts backward after playing forward, for AC_PLAY / AC_RESUME; the clip goes to the world.
    fn play(&mut self, i: usize, j: usize) {
        if !self.live[i].enabled {
            return;
        }
        let Some(&object) = self.live[i].t.objects.first() else { return };
        let e = &self.live[i].t.entries[j];
        let (command, mode, clip, rate) = (e.anim_command(), e.anim_mode(), e.anim_clip(), e.time_scale());
        let backward = mode >= data::AM_BACKWARD;
        if command == data::AC_PLAY || command == data::AC_RESUME {
            let ClassState::Anim { backward: last } = &mut self.live[i].state else { return };
            let signal = match mode {
                0 if *last => { *last = false; Some(sig::ANIM_START) }
                0 => None,
                data::AM_BACKWARD if !*last => { *last = true; Some(sig::ANIM_BACK_START) }
                data::AM_BACKWARD => None,
                _ => Some(sig::ANIM_START),
            };
            if let Some(s) = signal {
                self.send(i, Ev::new(s, 0));
            }
        }
        self.acts.push(Act::Anim { trigger: i, object, clip, backward, rate, command });
    }

    /// 0x200d70: a destroyer (its handler goes on to its entries once its base actions have
    /// enabled it).
    fn destroyer(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            self.base_and_after(i, &ev);
            if !self.live[i].enabled {
                return;
            }
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            if let ClassState::Destroyer { pending } = &mut self.live[i].state {
                *pending = true;
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// FUN_00233b90: the next random float in [0, 1).
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(RANDOM_MUL).wrapping_add(RANDOM_ADD);
        f32::from_bits((self.seed >> 9) | 0x3F80_0000) - 1.0
    }

    /// FUN_001ffb80: a gamevar operand's value, `data` the event's (its +0x10 read as a float):
    /// a game flag's value as a float (0 without it), the constant, random x the constant, the
    /// data, or floor(random x the constant); then 1 - it if inverted.
    fn operand(&mut self, o: &Operand, data: f32) -> f32 {
        let v = match o.kind {
            operand::GAMEFLAG => self.campaign.flags.value(&o.path).map_or(0.0, |v| v.as_f32()),
            operand::CONSTANT => o.constant,
            operand::RANDOM => self.random() * o.constant,
            operand::SIGNAL_DATA => data,
            operand::RANDOM_INT => (self.random() * o.constant).floor(),
            _ => 0.0,
        };
        if o.invert { 1.0 - v } else { v }
    }

    /// 0x1ffc40: a gamevar-trigger. OPERATE: its operation on A and B (evaluated in the handler's
    /// order, which only random operands notice), the entry counted (FUN_001feac0: an entry that
    /// disables the trigger silences the sends after it), the result turned over if `out` is
    /// inverted, then written: to `out`'s flag (FUN_00077eb0; a path the flags lack writes
    /// nothing), kept as `out`'s constant, or neither (the other kinds). VALUE_CHANGED goes to
    /// the targets when the result isn't the value it replaced (always for those that keep
    /// nothing), then TRUE when the result isn't 0 (|r| >= 1e-6), else FALSE; both with the
    /// result as their data and the event's subject.
    fn gamevar(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        if self.live[i].dead {
            return;
        }
        let id = self.live[i].t.id;
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            let ClassState::Gamevar { op, operands } = &self.live[i].state else { return };
            let (op, operands) = (*op, operands.clone());
            let data = f32::from_bits(ev.data as u32);
            let a = |tr: &mut Self| tr.operand(&operands[1], data);
            let b = |tr: &mut Self| tr.operand(&operands[2], data);
            let truth = |x: bool| if x { 1.0 } else { 0.0 };
            let result = match op {
                op::EQUAL => { let vb = b(self); let va = a(self); truth((va - vb).abs() < GAMEVAR_EQUAL) }
                op::LESS => { let va = a(self); truth(va < b(self)) }
                op::LESS_EQUAL => { let va = a(self); truth(va <= b(self)) }
                op::GREATER => { let va = a(self); truth(va > b(self)) }
                op::GREATER_EQUAL => { let va = a(self); truth(va >= b(self)) }
                op::ASSIGN => a(self),
                op::DEFAULT => match operands[0].kind {
                    operand::GAMEFLAG => self.campaign.flags.get(&operands[0].path).map_or(0.0, |(_, d)| d.as_f32()),
                    _ => 0.0,
                },
                op::ADD => { let vb = b(self); vb + a(self) }
                op::SUBTRACT => { let va = a(self); va - b(self) }
                op::MULTIPLY => { let vb = b(self); a(self) * vb }
                op::DIVIDE => { let vb = b(self); if vb == 0.0 { f32::MAX } else { a(self) / vb } }
                _ => 0.0,
            };
            self.entry_used(i, j, &ev);
            let out = &operands[0];
            let result = if out.invert { 1.0 - result } else { result };
            let why = format!("{} {} on {}", self.who(i), op_name(op), sig::name(ev.signal));
            let changed = match out.kind {
                operand::GAMEFLAG => match self.campaign.write(&out.path, result, &why, self.clock) {
                    Some((old, _)) => result != old.as_f32(),
                    None => true,
                },
                operand::CONSTANT => match &mut self.live[i].state {
                    ClassState::Gamevar { operands, .. } => {
                        let old = std::mem::replace(&mut operands[0].constant, result);
                        result != old
                    }
                    _ => true,
                },
                _ => true,
            };
            if self.log {
                println!("t {:.2}: {why}: {result}{}", self.clock, if changed { " (changed)" } else { "" });
            }
            let ev_out = Ev { signal: sig::VALUE_CHANGED, id, subject: ev.subject, data: i64::from(result.to_bits() as i32) };
            if changed {
                self.send(i, ev_out);
            }
            self.send(i, Ev { signal: if result.abs() < GAMEVAR_ZERO { sig::FALSE } else { sig::TRUE }, ..ev_out });
        }
    }

    /// 0x20ba00: a score-trigger. OPERATE, in a campaign (the campaign mode, world+0xc4c, with its
    /// squad): its amount, x 1 + 0.1 x the difficulty with h_11819686 (0x20baaa-0x20bad5), to the
    /// squad's cash under its key with h_08f457d8, else the null name (FUN_0005c5e0); then the
    /// entry counted.
    fn score(&mut self, i: usize, ev: Ev) {
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
            let ClassState::Score { amount, key, keyed, scaled } = self.live[i].state else { return };
            if self.campaign.in_mission {
                let scale = if scaled { self.campaign.cash_scale() } else { 1.0 };
                let why = format!("{} on {}: {amount} x {scale:.2}", self.who(i), sig::name(ev.signal));
                if self.log {
                    println!("t {:.2}: {why}: {:+.0} to the squad's cash", self.clock, amount * scale);
                }
                self.campaign.add_cash(if keyed { key } else { 0 }, amount * scale, &why, self.clock);
            } else if self.log {
                println!("t {:.2}: {} on {}: no campaign, no cash", self.clock, self.who(i), sig::name(ev.signal));
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// A class not run yet: its base actions as every handler has them; its own actions logged
    /// (and counted as used, as its handler counts them).
    fn other(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let action = self.live[i].t.entries[j].action;
            if matches!(action, act::ENABLE | act::DISABLE | act::TOGGLE_ENABLE) || (act::USER_SIG1..=act::USER_SIG8).contains(&action) {
                self.base_and_after(i, &ev);
                continue;
            }
            if self.log {
                if let Some(step) = data::later_step(self.live[i].t.tag) {
                    println!("t {:.2}: {} {} on {}{}: not run yet ({step})", self.clock, self.who(i), act::name(action), sig::name(ev.signal), self.about(&ev));
                }
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// FUN_001fe660: FIRST_UPDATE to the trigger itself, once.
    fn first_update(&mut self, i: usize) {
        if self.live[i].first_done {
            return;
        }
        self.live[i].first_done = true;
        let id = self.live[i].t.id;
        self.deliver(i, Ev::new(sig::FIRST_UPDATE, id));
    }

    /// FUN_001fe610: the base update: FIRST_UPDATE while enabled.
    fn base_update(&mut self, i: usize) {
        if self.live[i].enabled {
            self.first_update(i);
        }
    }

    /// Trigger `i`'s update (vtable +0x1c) over `dt` s.
    fn update(&mut self, i: usize, dt: f32, actors: &[Actor]) {
        match self.kind(i) {
            Kind::Sound => self.sound_update(i, dt),
            Kind::Dialog => self.dialog_update(i),
            Kind::Input => self.input_update(i),
            Kind::Menu => self.menu_update(i),
            Kind::Event => self.event_update(i),
            Kind::Detector => self.detector_update(i, dt, actors),
            Kind::Timer => self.timer_update(i, dt),
            Kind::Spawn => self.spawn_update(i, dt),
            Kind::Destroyer => self.destroyer_update(i),
            _ => self.base_update(i),
        }
    }

    /// 0x2002f0: a running timer counts down while enabled; at 0 TIMER_PULSE to its targets
    /// (with the subject its OPERATE came with), then it stops and reloads (h_f92b0b15) or
    /// reloads (enable-looping), else stays at 0 until reset.
    fn timer_update(&mut self, i: usize, dt: f32) {
        self.base_update(i);
        if !self.live[i].enabled {
            return;
        }
        let ClassState::Timer { running, looping, stops, initial, current, subject } = &mut self.live[i].state else { return };
        if !*running || *current <= 0.0 {
            return;
        }
        *current -= dt;
        if *current > 0.0 {
            return;
        }
        *current = 0.0;
        let (looping, stops, initial, subject) = (*looping, *stops, *initial, *subject);
        self.send(i, Ev { signal: sig::TIMER_PULSE, id: 0, subject, data: -1 });
        let ClassState::Timer { running, current, .. } = &mut self.live[i].state else { return };
        if stops {
            *running = false;
            *current = initial;
        }
        if looping {
            *current = initial;
        }
    }

    /// 0x209bd0: a spawn-trigger's delay runs while it's enabled; at 0 it spawns if armed.
    fn spawn_update(&mut self, i: usize, dt: f32) {
        self.base_update(i);
        if !self.live[i].enabled {
            return;
        }
        let ClassState::Spawn { pending, armed, .. } = &mut self.live[i].state else { return };
        if *pending <= 0.0 {
            return;
        }
        *pending -= dt;
        if *pending <= 0.0 && *armed {
            self.spawn(i);
        }
    }

    /// 0x200e40: a destroyer's OPERATE done: its objects removed (a listed spawn-trigger's
    /// copy: FUN_00200d00), then it's off (its DISABLED lost).
    fn destroyer_update(&mut self, i: usize) {
        self.base_update(i);
        let ClassState::Destroyer { pending } = &mut self.live[i].state else { return };
        if !*pending {
            return;
        }
        *pending = false;
        for o in self.live[i].t.objects.clone() {
            let object = match self.by_name.get(&o).map(|&k| &self.live[k].state) {
                Some(ClassState::Spawn { copy, .. }) => *copy,
                Some(_) => continue,
                None => o,
            };
            if object != 0 {
                self.acts.push(Act::Destroy { trigger: i, object });
            }
        }
        self.disable(i);
    }

    /// FUN_001f9680: a detector's update. Enabled: its FIRST_UPDATE, then who's in it now
    /// (FUN_001f94e0: the actors its filter admits in its volume) and its occupancy
    /// (FUN_001f8db0). Disabled with h_e25d95e3: the occupancy alone, silent. Else nothing.
    fn detector_update(&mut self, i: usize, dt: f32, actors: &[Actor]) {
        let silent = if self.live[i].enabled {
            self.first_update(i);
            false
        } else {
            match &self.live[i].state {
                ClassState::Detector { keep: true, .. } => true,
                _ => return,
            }
        };
        let Some(at) = self.live[i].t.transform else { return };
        let ClassState::Detector { volume, filter, .. } = &self.live[i].state else { return };
        let inv = at.inverse();
        let now: Vec<u32> = actors.iter().filter(|a| passes(filter, a) && inside(*volume, &at, &inv, a)).map(|a| a.name).collect();
        self.occupancy(i, dt, &now, silent);
    }

    /// FUN_001f8db0: a detector's occupants against who's in it `now`: one in for 0.1 s sends
    /// ENTER_DETECTOR (with ENTER_DETECTOR_FIRST when nobody was counted in before) and its own
    /// ENTER_DETECTOR_TO_SELF; one out for 0.2 s (in sum) leaves, sending EXIT_DETECTOR and
    /// EXIT_DETECTOR_TO_SELF, or, silent, enabling the detector; the last one gone sends
    /// EMPTIED_DETECTOR.
    fn occupancy(&mut self, i: usize, dt: f32, now: &[u32], silent: bool) {
        let id = self.live[i].t.id;
        let ClassState::Detector { occupants, .. } = &mut self.live[i].state else { return };
        if occupants.is_empty() && now.is_empty() {
            return;
        }
        let first = !occupants.iter().any(|o| o.inside >= ENTER_TIME || o.inside == -1.0);
        let mut left = vec![];
        for o in occupants.iter_mut() {
            if o.inside >= 0.0 {
                o.inside += dt;
            }
            if now.contains(&o.name) {
                continue;
            }
            o.absent += dt;
            if o.absent >= EXIT_TIME {
                left.push(o.name);
            }
        }
        let mut entered = vec![];
        for &n in now {
            match occupants.iter_mut().find(|o| o.name == n) {
                Some(o) => {
                    if o.inside >= ENTER_TIME {
                        o.inside = -1.0;
                        if !silent {
                            entered.push(n);
                        }
                    }
                }
                None => occupants.push(Occupant { name: n, inside: 0.0, absent: 0.0 }),
            }
        }
        let had = !occupants.is_empty();
        occupants.retain(|o| !left.contains(&o.name));
        let empty = occupants.is_empty();
        let mut exits = vec![];
        for n in left {
            if silent {
                self.enable(i);
                if self.log {
                    println!("t {:.2}: {} enables itself: {} left it (h_e25d95e3)", self.clock, self.who(i), self.label(n));
                }
            } else {
                exits.push(n);
            }
        }
        for n in entered {
            let ev = Ev { signal: sig::ENTER_DETECTOR, id, subject: n, data: -1 };
            self.send(i, ev);
            if first {
                self.send(i, Ev { signal: sig::ENTER_FIRST, ..ev });
            }
            self.deliver(i, Ev { signal: sig::ENTER_TO_SELF, ..ev });
        }
        for n in exits {
            let ev = Ev { signal: sig::EXIT_DETECTOR, id, subject: n, data: -1 };
            self.send(i, ev);
            self.deliver(i, Ev { signal: sig::EXIT_TO_SELF, ..ev });
        }
        if had && empty {
            self.send(i, Ev { signal: sig::EMPTIED_DETECTOR, id, subject: 0, data: -1 });
        }
    }

    // ---- the cutscenes' triggers (#222): script, letterbox and warp; the scripts' signals ----

    /// A script-trigger (0x205000), a letterbox-trigger (0x207480) or a warp-trigger (0x200650):
    /// disabled, its base actions; enabled, each entry that answers: OPERATE (and a letterbox's
    /// STOP) its own, the rest the base's; the entry counted after (FUN_001feac0).
    ///  * script: its script asked to play, by the entry's `<script command>` (FUN_00204ec0, for
    ///    play_script.rs);
    ///  * letterbox: OPERATE the bars in over its show-duration (FUN_00073460), STOP out over its
    ///    hide-duration (FUN_000734b0);
    ///  * warp: its objects to its world point (FUN_00200480, `warp` in `run`).
    fn cutscene_trigger(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let name = self.live[i].t.name;
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let (action, command) = (self.live[i].t.entries[j].action, self.live[i].t.entries[j].script_command());
            match (&self.live[i].state, action) {
                (ClassState::Script { script }, act::OPERATE) => {
                    self.cutscene.push(script::Request::Script { trigger: name, script: *script, command, signal: ev.signal });
                }
                (ClassState::Letterbox { show, .. }, act::OPERATE) => {
                    self.cutscene.push(script::Request::Letterbox { trigger: name, show: true, secs: *show });
                }
                (ClassState::Letterbox { hide, .. }, act::STOP) => {
                    self.cutscene.push(script::Request::Letterbox { trigger: name, show: false, secs: *hide });
                }
                (ClassState::Warp { .. }, act::OPERATE) => self.acts.push(Act::Warp { trigger: i }),
                _ => {
                    self.base_and_after(i, &ev);
                    continue;
                }
            }
            if self.log {
                println!("t {:.2}: {} {} on {}", self.clock, self.who(i), act::name(action), sig::name(ev.signal));
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// A script's 103 (0x1772b0): the signal straight to the named trigger's handler, from iD 0,
    /// with no subject and data -1. Whether the level has that trigger.
    pub(super) fn script_signal(&mut self, name: u32, signal: i64) -> bool {
        let Some(&i) = self.by_name.get(&name) else { return false };
        if self.log {
            println!("t {:.2}: a script sends {} to {}", self.clock, sig::name(signal), self.who(i));
        }
        self.deliver(i, Ev::new(signal, 0));
        true
    }

    /// What the script and letterbox triggers asked of the cutscenes since last taken.
    pub(super) fn take_cutscene(&mut self) -> Vec<script::Request> {
        std::mem::take(&mut self.cutscene)
    }

    /// Whether something has been asked of the world (a spawn, a warp, a door, a removal) that
    /// `run` hasn't done yet: a script's event that asked it waits for it (play_script.rs `update`).
    pub(super) fn has_acts(&self) -> bool {
        !self.acts.is_empty()
    }

    /// The name a warp-trigger's object is known by in the world: a spawn-trigger's copy (FUN_00200480
    /// warps the copy; the squad's start stands for its spawn's copy, `SquadSpawns`), else the object.
    fn warp_object(&self, o: u32, spawns: &SquadSpawns) -> u32 {
        match self.by_name.get(&o).map(|&k| &self.live[k].state) {
            Some(ClassState::Spawn { copy, .. }) if *copy != 0 => *copy,
            Some(ClassState::Spawn { .. }) => spawns.0.iter().find(|s| s.trigger == o).map_or(0, |s| s.copy),
            Some(_) => 0,
            None => o,
        }
    }
}

/// The class state of a trigger the data reader leaves to its step (`Class::Other`): the
/// cutscenes' three (#222) from their elements' own attributes (the parsers FUN_00204e40,
/// FUN_00207410, 0x200400), their constructors' values where one has none; anything else runs its
/// base actions alone.
fn cutscene_class(t: &Trigger) -> ClassState {
    let e = &t.element;
    let name = |k: u32| e.attr(k).and_then(|v| v.as_hash()).filter(|&n| n != h("")).unwrap_or(0);
    let num = |k: u32, or: f32| e.attr(k).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(or);
    if t.tag == h("script-trigger") {
        ClassState::Script { script: name(h("script-name")) }
    } else if t.tag == h("letterbox-trigger") {
        ClassState::Letterbox { show: num(h("show-duration"), LETTERBOX_TIME), hide: num(h("hide-duration"), LETTERBOX_TIME) }
    } else if t.tag == h("warp-trigger") {
        ClassState::Warp { point: name(h("world-point")), fade: num(H_WARP_FADE, WARP_FADE) }
    } else {
        ClassState::Other
    }
}

/// A warp-trigger's h_0ede4719 (+0x48, its parser 0x200400): the fade-in (s) a living character
/// it warps gets (FUN_00200480: char+0x73c, the fade state +0x734 = 0): 2 on the tutorial's
/// squad warps, 0 on the rest. Not drawn (as a spawn's fade isn't).
const H_WARP_FADE: u32 = 0x0EDE_4719;
/// A warp-trigger's fade-in without an h_0ede4719 (s): its constructor's +0x48, 2.0 (0x200734).
const WARP_FADE: f32 = 2.0;
/// A letterbox-trigger's show- and hide-duration without one (s): its constructor's +0x40 and
/// +0x44, 0.5 (0x207599).
const LETTERBOX_TIME: f32 = 0.5;

/// A warp-trigger's OPERATE in the world (FUN_00200480): each of its objects (a spawn-trigger's
/// copy, `Triggers::warp_object`) to its world point. A character, if alive, stands on the ground
/// under the point (the place call's ray, `on_ground`), facing down the point's -z when it has an
/// orientation (a direction-point; the place call +0x50 with its whole transform, as a spawn's
/// copy faces its trigger's -z) and as before when not (a world-point; the position setter +0x48);
/// the controlled one's camera goes with it. A dead character isn't moved. The warp's fade-in
/// (h_0ede4719), +0x264 = 5.0 and FUN_0004c5e0 (an AI reset) aren't done.
#[allow(clippy::too_many_arguments)]
fn warp(tr: &mut Triggers, i: usize, level: &Level, spawns: &SquadSpawns, player: &mut Player, squad: &mut Squad) {
    let now = tr.clock;
    let by = tr.label(tr.live[i].t.name);
    let ClassState::Warp { point, fade } = tr.live[i].state else { return };
    let Some(p) = level.objects.iter().find(|o| o.name == point && point != 0) else {
        if tr.log {
            println!("t {now:.2}: {by}: no world point {} on the level", tr.label(point));
        }
        return;
    };
    let at = p.transform.w_axis.truncate();
    let yaw = p.oriented.then(|| yaw_of(&p.transform));
    for o in tr.live[i].t.objects.clone() {
        let name = tr.warp_object(o, spawns);
        // (a squad member not spawned yet has no copy to warp)
        let unit = if player.trigger_name == name && name != 0 && !player.stand_in {
            (!player.absent).then_some((&mut *player, true))
        } else {
            squad.0.iter_mut().find(|m| m.trigger_name == name && name != 0 && !m.absent).map(|m| (m, false))
        };
        let Some((u, controlled)) = unit else {
            if tr.log {
                println!("t {now:.2}: {by}: {} isn't in the world to warp", tr.label(if name != 0 { name } else { o }));
            }
            continue;
        };
        if u.dead {
            continue;
        }
        let feet = on_ground(at);
        u.position = feet - Vec3::Y * GROUND;
        u.prev_xz = feet.xz();
        u.fall_from = feet.y;
        u.vy = 0.0;
        u.height = 0.0;
        if let Some(y) = yaw {
            u.yaw = y;
            u.face_yaw = Some(y);
            if controlled || u.npc.is_some() {
                u.cam_yaw = y;
            }
        }
        if controlled {
            u.cam_target = u.position + Vec3::new(0.0, 0.35, 0.0);
        }
        if tr.log {
            println!("t {now:.2}: {by} warps {} to {} at {feet:.2}{}{}", tr.label(name), tr.label(point),
                     yaw.map_or(" (keeping its facing)".to_string(), |y| format!(" facing {:.0} deg", y.to_degrees())),
                     if fade > 0.0 { format!(" (its {fade} s fade-in not drawn)") } else { String::new() });
        }
    }
}
