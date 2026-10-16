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
//!    re-armable one (h_0f95f576). The copy sends SPAWNED to its listeners, the trigger
//!    OPERATED to its targets; the triggers that list the spawn-trigger listen to its copies.
//!  * plate- and proximity-detector (FUN_001f9680, FUN_001f8db0): the characters its filter
//!    admits (FUN_001fb980) in its volume; one inside 0.1 s sends ENTER_DETECTOR (and
//!    ENTER_DETECTOR_FIRST if none was), one out of it 0.2 s EXIT_DETECTOR, the last gone
//!    EMPTIED_DETECTOR. The plate's volume is its box (`size` its half-extents: the phantom
//!    surface h_051ac63e is that box), the proximity detector's a sphere of `radius`; the
//!    character is the demo's cylinder (BODY_RADIUS, BODY_HEIGHT), not its Ipion body.
//!  * anim-trigger (0x204d00, FUN_00204bb0): OPERATE plays its entry's clip on its first object,
//!    forward or backward (wwit-debugging) at its time-scale, ANIM_START / ANIM_BACK_START to its
//!    targets when the way it plays changes. The doors (play.rs `doors`) are what it moves.
//!  * destroyer (0x200d70, 0x200e40): OPERATE removes its objects (a spawn-trigger's copy for a
//!    spawn-trigger) on its next update, then it disables itself.
//!  * every other class (gamevar, script, letterbox, warp, sound, dialog, menu, message, input,
//!    event, radar, level-over ...): its base actions; its own actions are logged as left to
//!    their step (`bf::triggers::later_step`). Waypoint and evaluate triggers: base only.
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
use bf_viewer::bf::triggers::{self as data, act, sig, Class, Filter, Trigger, Volume};
use bf_viewer::level_scene::Placed;

use super::*;

/// The triggers' state and the world's events for them, set up when a map starts (after
/// play.rs `setup` has placed the level); their frame after the squad has moved (the
/// detectors see where it is now) and before the doors move (an anim-trigger's clip starts in
/// its frame).
pub fn plugin(app: &mut App) {
    app.init_resource::<TriggerEvents>()
        .init_resource::<Triggers>()
        .add_systems(OnEnter(AppState::Playing), start.after(setup))
        .add_systems(Update, run.after(squad_control).before(doors).run_if(in_state(AppState::Playing)));
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
/// what the level's triggers and scripts call it), where its feet go (the trigger's position)
/// and the way it faces (the trigger's -z: `yaw_of`). A member placed in the world instead
/// (m09_a's) has its placement for all of these, `trigger` 0.
#[derive(Clone, Copy, Debug)]
pub struct SquadSpawn {
    pub slot: usize,
    pub template: u32,
    pub trigger: u32,
    pub copy: u32,
    pub feet: Vec3,
    pub yaw: f32,
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

/// The level's squad spawns: each spawn-trigger whose template is a character placement of a
/// squad slot's type (brutus, flint, hawk, tex), the first per slot in the file's order (e40,
/// the tutorial); then a squad placement no spawn-trigger takes as its template is in the world
/// from the start, where it's placed (m09_a's four: its warp-triggers move them, step 3), named
/// by its own name, `trigger` 0. A template stays out of the world until spawned (FUN_000dfe90
/// takes it out as its spawn-trigger links).
pub fn squad_spawns(_game: &Game, level: &Level) -> SquadSpawns {
    let mut out: Vec<SquadSpawn> = vec![];
    let slot_of = |kind: u32| (0..SQUAD).find(|&c| squad_type(c) == kind);
    let mut templates = HashSet::new();
    for t in &level.triggers {
        let (Class::Spawn { template, copy_name, .. }, Some(m)) = (&t.class, t.transform) else { continue };
        templates.insert(*template);
        let Some(o) = level.objects.iter().find(|o| o.name == *template && o.tag == h("character-object")) else { continue };
        let Some(slot) = slot_of(o.kind) else { continue };
        if out.iter().any(|s| s.slot == slot) {
            continue;
        }
        out.push(SquadSpawn { slot, template: *template, trigger: t.name, copy: *copy_name, feet: m.w_axis.truncate(), yaw: yaw_of(&m) });
    }
    for o in level.objects.iter().filter(|o| o.tag == h("character-object") && o.name != 0 && !templates.contains(&o.name)) {
        let Some(slot) = slot_of(o.kind) else { continue };
        if out.iter().any(|s| s.slot == slot) {
            continue;
        }
        out.push(SquadSpawn { slot, template: o.name, trigger: 0, copy: o.name, feet: o.transform.w_axis.truncate(), yaw: yaw_of(&o.transform) });
    }
    SquadSpawns(out)
}

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
    Router { filters: bool, out_signal: i64 },
    /// +0x48 running, +0x49 looping, +0x4a stops, +0x40 initial, +0x44 current, +0x50 the subject
    /// its OPERATE came with (its pulse's)
    Timer { running: bool, looping: bool, stops: bool, initial: f32, current: f32, subject: u32 },
    Calc { counter: i64, target: i64, test: i64 },
    /// armed (+0xe4 bit 3, set by its constructor, 0x1fd680-0x1fd6e7), the delay running (+0xec), its current
    /// copy (+0xc4, by name)
    Spawn { armed: bool, rearm: bool, pending: f32, delay: f32, template: u32, copy_name: u32, fade: f32, copy: u32 },
    Detector { volume: Volume, filter: Filter, keep: bool, occupants: Vec<Occupant> },
    /// +0x5c bit 0: its clip last played backward (set by its constructor 0x204db0: the first
    /// forward play sends ANIM_START)
    Anim { backward: bool },
    /// +0x4c: OPERATE asked, done on its next update
    Destroyer { pending: bool },
}

/// Which class a trigger runs as (for `update` and `deliver`).
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Other,
    Router,
    Timer,
    Calc,
    Spawn,
    Detector,
    Anim,
    Destroyer,
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
}

/// A map starts: its triggers as the level file has them, the listeners linked (FUN_001febb0),
/// the test hooks read.
fn start(mut commands: Commands, map: Res<MapLevel>, game: Res<GameData>, spawns: Option<Res<SquadSpawns>>) {
    commands.insert_resource(TriggerEvents::default());
    let Some(level) = map.0.as_ref() else {
        commands.insert_resource(Triggers::default());
        return;
    };
    let log = std::env::var("BF_TRIGGER_LOG").is_ok();
    let mut tr = Triggers { log, ..Default::default() };
    for t in &level.triggers {
        let state = match &t.class {
            Class::Router { filters, out_signal } => ClassState::Router { filters: *filters, out_signal: *out_signal },
            Class::Timer { running, looping, stops, initial, current } => ClassState::Timer { running: *running, looping: *looping, stops: *stops,
                                                                                             initial: *initial, current: *current, subject: 0 },
            Class::Calculator { counter, target, test } => ClassState::Calc { counter: *counter, target: *target, test: *test },
            Class::Spawn { template, copy_name, fade, delay, rearm, .. } => ClassState::Spawn { armed: true, rearm: *rearm, pending: 0.0, delay: *delay,
                                                                                               template: *template, copy_name: *copy_name, fade: *fade, copy: 0 },
            Class::Detector { volume, filter, keep_when_disabled, .. } => ClassState::Detector { volume: *volume, filter: filter.clone(), keep: *keep_when_disabled,
                                                                                                occupants: vec![] },
            Class::Anim => ClassState::Anim { backward: true },
            Class::Destroyer => ClassState::Destroyer { pending: false },
            Class::Other => ClassState::Other,
        };
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
        for s in spawns.as_deref().map_or(&[][..], |s| &s.0[..]) {
            let from = if s.trigger == 0 { "placed in the world".to_string() } else { format!("from {}", tr.label(s.trigger)) };
            println!("squad spawn: {} as {} {from} at {:.2} facing {:.0} deg", CHARACTERS[s.slot], tr.label(s.copy), s.feet, s.yaw.to_degrees());
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
/// the ones the triggers spawned among them). One no trigger named gets a name of its own for
/// the detectors' occupant lists (MADE_UP_UNIT + its squad slot, or + 100 + its NPC number).
fn actors(player: &Player, squad: &Squad, game: &Game) -> Vec<Actor> {
    std::iter::once((player, true)).chain(squad.0.iter().map(|m| (m, false))).filter(|(u, _)| !u.dead && !u.stand_in).map(|(u, human)| {
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
       mut placed: Query<(&Placed, &mut Visibility)>, mut hits: ResMut<AiHits>) {
    let Some(level) = map.0.as_ref() else { return };
    if tr.live.is_empty() {
        events.0.clear();
        return;
    }
    let dt = frame_dt(&time);
    tr.clock = player.sim_time;
    // the level's start: FIRST_UPDATE to every script-trigger, then its update (FUN_001fe260)
    if !tr.started {
        tr.started = true;
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
    // what they asked of the world
    let acts = std::mem::take(&mut tr.acts);
    let none = SquadSpawns::default();
    let spawns = spawns.as_deref().unwrap_or(&none);
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
                let opening = !backward;
                if opening != d.opening {
                    // its sound beside it (the level's signal 31 / 49 sound-triggers)
                    if let Some(id) = if opening { d.door.open_sound } else { d.door.close_sound } {
                        let k = 1.0 - d.door.centre.distance(player.position) / DOOR_HEARD;
                        if k > 0.0 {
                            player.sound_queue.push((id, k));
                        }
                    }
                }
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
                if let Some(k) = squad.0.iter().position(|m| m.trigger_name == object) {
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
        }
    }
}

/// A spawn-trigger's copy in the world (FUN_00209620): its template's kind decides what it is.
///  * a squad member's template (`SquadSpawns`): that member, moved to the spawn (the demo has
///    the squad in the world from the start: #102's start, see play.rs `player_at_start`);
///  * any other character placement: a character of its type with the placement's weapons,
///    personality and team (as BF_TEST_NPC spawns a placement, #129), set to fight, facing the
///    trigger's way;
///  * an effect object: its effect type's ALE effects at the transform, once each (a looping one
///    that the game keeps until a destroyer removes it plays its emitters once here);
///  * anything else: not made (logged).
/// The character's fade-in over `fade` s (char+0x734..0x73c, FUN_0011bd00) isn't drawn: it
/// appears at once.
#[allow(clippy::too_many_arguments)]
fn spawn_copy(commands: &mut Commands, tr: &mut Triggers, player: &mut Player, squad: &mut Squad, game: &mut Game, level: &Level, spawns: &SquadSpawns,
              fx: &mut (Option<ResMut<bf_viewer::ale_fx::AleAssets>>, ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
              trigger: usize, template: u32, copy: u32, at: Mat4, fade: f32) {
    let now = tr.clock;
    let by = tr.label(tr.live[trigger].t.name);
    let feet = at.w_axis.truncate();
    let yaw = yaw_of(&at);
    // a squad member: moved there
    if let Some(s) = spawns.0.iter().find(|s| s.template == template) {
        let unit = if player.character == s.slot && player.in_squad() && !player.stand_in { Some(&mut *player) } else {
            squad.0.iter_mut().find(|m| m.character == s.slot && m.in_squad())
        };
        if let Some(u) = unit {
            u.position = feet - Vec3::Y * GROUND;
            u.prev_xz = feet.xz();
            u.fall_from = feet.y;
            u.yaw = yaw;
            u.cam_yaw = yaw;
            u.vy = 0.0;
            u.height = 0.0;
            u.trigger_name = copy;
            u.cam_target = u.position + Vec3::new(0.0, 0.35, 0.0);
            if tr.log {
                println!("t {now:.2}: {by} spawns {} ({}) at {feet:.2} facing {:.0} deg (the squad member moved there; its {fade} s fade-in not drawn)",
                         tr.label(copy), CHARACTERS[s.slot], yaw.to_degrees());
            }
            return;
        }
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
        let placement = game.placements.iter().find(|p| p.name == template && p.transform.w_axis.distance(o.transform.w_axis) < 0.01).cloned();
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
            println!("t {now:.2}: {by} spawns {} ({}, team {}) at {feet:.2} facing {:.0} deg{}", tr.label(copy), game.type_label(kind), m.team, yaw.to_degrees(),
                     if fade > 0.0 { format!(" (its {fade} s fade-in not drawn)") } else { String::new() });
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
                    let seed = (feet.x * 977.0 + feet.z * 131.0) as u32 ^ copy;
                    made.push(commands.spawn((Transform::from_matrix(at), Visibility::default(), bf_viewer::ale_fx::AleEffect::once(e, 0.0, seed),
                                              AleExpire(life), Name::new(format!("trigger effect h_{effect:08x}")))).id());
                }
            }
            if tr.log {
                println!("t {now:.2}: {by} spawns {} (effect h_{:08x}: {} ALE effects) at {feet:.2}", tr.label(copy), o.kind, made.len());
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
            ClassState::Router { .. } => Kind::Router,
            ClassState::Timer { .. } => Kind::Timer,
            ClassState::Calc { .. } => Kind::Calc,
            ClassState::Spawn { .. } => Kind::Spawn,
            ClassState::Detector { .. } => Kind::Detector,
            ClassState::Anim { .. } => Kind::Anim,
            ClassState::Destroyer { .. } => Kind::Destroyer,
        }
    }

    /// FUN_001feb60: the next of trigger `i`'s entries from `from` that answers `ev`: the same
    /// signal and match-id, used fewer times than its limit.
    fn next_entry(&self, i: usize, ev: &Ev, from: usize) -> Option<usize> {
        self.live[i].t.entries.iter().enumerate().skip(from)
            .find(|(_, e)| e.signal == ev.signal && e.match_id == ev.id && e.used < e.limit).map(|(j, _)| j)
    }

    /// 0x1fefc0: on (unless dead), every entry's use count back to 0.
    fn enable(&mut self, i: usize) {
        let l = &mut self.live[i];
        if l.dead {
            return;
        }
        l.enabled = true;
        for e in &mut l.t.entries {
            e.used = 0;
        }
    }

    /// 0x1feff0: off.
    fn disable(&mut self, i: usize) {
        self.live[i].enabled = false;
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
            Kind::Router => self.router(i, ev),
            Kind::Timer => self.timer(i, ev),
            Kind::Calc => self.calculator(i, ev),
            Kind::Spawn => self.spawner(i, ev),
            // (a detector's handler is the base's itself: FUN_001ff0f0, vtable 0x3965a8 +0x60)
            Kind::Detector => self.base_actions(i, &ev),
            Kind::Anim => self.anim(i, ev),
            Kind::Destroyer => self.destroyer(i, ev),
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

    /// FUN_002099e0: spawn-trigger `i` makes its copy: no longer armed; a template the level
    /// has gives a copy, which sends SPAWNED to its listeners (FUN_002096e0), and the trigger
    /// sends OPERATED with it to its targets.
    fn spawn(&mut self, i: usize) {
        let (template, fade, rearm, copy_name) = match &mut self.live[i].state {
            ClassState::Spawn { armed, rearm, template, copy_name, fade, .. } => {
                *armed = false;
                (*template, *fade, *rearm, *copy_name)
            }
            _ => return,
        };
        if template == 0 || !self.known.contains(&template) {
            if self.log {
                println!("t {:.2}: {}: its template {} isn't on this level: no copy", self.clock, self.who(i), self.label(template));
            }
            return;
        }
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
        self.acts.push(Act::Spawn { trigger: i, template, copy, at, fade });
        self.object_event(copy, Ev::new(sig::SPAWNED, 0));
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
}
