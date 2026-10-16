//! The map editor (#191, the home of the map creation epic #190): the test map's own edit mode,
//! out of the gunplay, and a switch to play the map and back. Demo tooling, not the game: the
//! keys, the camera and the layout are the demo's choices (the camera's controls are the level
//! viewer's, bf_level: the right button held looks, WASD flies, E / Q up and down).
//!
//!  - **The modes** (`EditorMode`, a Bevy state): Play, the test map as it was (the squad, the
//!    follow camera, the HUD; the tools' menus too), and Edit. In Edit no character is in the
//!    world: the squad leaves it (`bench`: their bodies go, the squad's members leave `Squad`,
//!    the NPCs stay) and a stand-in without a body takes the controlled character's place
//!    (play.rs `Player::stand_in`: counted dead, so everything that acts on characters passes it
//!    over). No HUD, crosshair or controls panel; the camera is a free one (play_testtools.rs's
//!    `FreeCam`, the edit camera); `read_input` and the follow camera don't run. The world goes
//!    on meanwhile: NPCs set to fight fight each other, things fall, effects play (freezing it is
//!    #192's).
//!  - **The controls**: the right mouse button held looks (the cursor hidden, and put back where
//!    it was when it's let go); WASD flies along the view, E / Space up, Q / Ctrl down, Shift
//!    four times faster, the wheel (no menu open) the speed. The tools act at the mouse pointer
//!    (the ray through it, `Tools::pointer`), where their see-through copies follow it; with no
//!    pointer (the cursor off the window, BF_BACKGROUND) at the crosshair's place, marked with a
//!    small cross. A click on the tools' panel isn't one in the world. The panel lists the keys
//!    (H hides it). Esc closes a menu and does nothing else: no key but F5 leaves edit mode, and
//!    Backspace (leaving for the front end) wants a second press (play.rs `back_to_menu`).
//!  - **The tools** (`TOOLS`, the registry): a key and the menu it opens (P scenery, O objects,
//!    N NPCs, Y sky, U music, L the level to build on); `open_tools` opens and closes them, in
//!    both modes, and the edit mode's help lists them. A later tool (#186 liquids, #189 lights,
//!    #187's selection, #194, #195) adds an entry and its `Panel`.
//!  - **The placed things** (`Placements`, the one list that owns them): what the tools have put
//!    on the map, each with its number, what it is (enough to make it again: #188's file, #187's
//!    undo), where it was put and turned, and its root in the world (`PlacedId` on it) or, for an
//!    NPC, its number in `Squad`. Kept up to date each frame (`track`: here, dead, or gone:
//!    taken, broken, or removed in play). A map's: a level switch starts the next one empty.
//!  - **Play from here** (F5): the squad is put where the pointer meets the ground (else under
//!    the camera), facing the camera's way, the leader the character who last led, the others in
//!    formation round them (`squad_around`), out of the walls; Shift+F5 at the map's start
//!    instead. A fresh squad each time: full health, a full clip, the start's grenades, the
//!    reserve stocked again. The camera flies down from the edit camera to the follow camera
//!    behind the leader (FLIGHT_TIME), the HUD showing once it's there. **Back to edit** (F5 in
//!    play): the squad leaves, and the camera flies back up to the edit camera where it was.
//!  - **What carries over**: the map as it is, both ways. Nothing placed is lost; what was taken,
//!    broken or killed in play stays so (the NPCs' bodies lie where they fell, `Placements` marks
//!    them). The squad isn't the map's: it leaves with its bodies (the guns dropped and the blood
//!    stay), and comes back fresh.
//!  - **The start**: a test session starts in play (`--test`, so every test hook runs as it
//!    did); `--edit` in its place starts it in edit mode, the edit camera behind and above the
//!    map's start. F5 or the hooks switch.
//!
//! Test hooks (play_testtools.rs `read_hooks`, on the controlled character's clock, which the
//! stand-in carries on): BF_TEST_EDIT=<s>[,<x>,<y>,<z>,<yaw>,<pitch>] (F5 in play: edit mode,
//! the camera put there), BF_TEST_PLAY=<s>[,start] (F5 in edit mode, or Shift+F5),
//! BF_TEST_POINTER=<s>,<x>,<y> (the pointer at that place of the window, 0..1 across and down from
//! the top left). BF_TOOLS_LOG=1 prints each switch (who leaves and comes, where, the camera, the
//! placements) and each placement's change.

use super::*;
use super::testmap::TestMap;
use super::testtools::{aim_word, crosshair, ground_point, FreeCam, Hook, Panel, Script, Tools};

/// The command-line flag that starts a test session in edit mode (in place of `--test`).
const FLAG: &str = "--edit";
/// How long the camera's flight between the edit camera and the follow camera takes (s; the
/// demo's choice, eased in and out).
const FLIGHT_TIME: f32 = 0.75;
/// The edit camera at a map's start: the first of these places (m behind the start point, the
/// way it faces, and above it) with nothing of the map between it and the start's eyes, looking
/// at the start (the demo's choice: the start and what's round it in view; higher and closer
/// where a wall stands behind, as at sdm_e34's start in its canyon). With none clear, the one
/// with the most room, pulled in START_CLEAR short of what's in the way (as the follow camera
/// keeps in front of a wall).
const START_VIEWS: [(f32, f32); 4] = [(9.0, 5.0), (6.0, 9.0), (3.0, 13.0), (1.0, 18.0)];
const START_CLEAR: f32 = 0.3;
/// The start's eyes, from which the start views are tested (m above its feet: the AI's, AI_EYE).
const START_EYE: f32 = 1.6;
/// A notch of the wheel changes the edit camera's speed by this factor, within this range (m/s;
/// the demo's, round play_testtools.rs's FLY_SPEED).
const SPEED_STEP: f32 = 1.25;
const SPEED_RANGE: (f32, f32) = (1.5, 96.0);
/// Where F5 will put the squad's leader, drawn in edit mode with no menu open: a ring this wide
/// and an arrow this long (m) the way they'll face, in this colour (the demo's: the object menu's
/// drop mark's cyan).
const SPOT_RING: f32 = 0.5;
const SPOT_ARROW: f32 = 1.2;
const SPOT_MARK: Color = Color::srgb(0.3, 0.85, 1.0);
/// The pointer's cross where the cursor doesn't show (looking, or no cursor): half its length
/// and its thickness (px), and its colour (the demo's).
const MARK_HALF: f32 = 7.0;
const MARK_THICK: f32 = 2.0;
const MARK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.9);
/// How long a switch's message shows (s): the tools' (play_testtools.rs MESSAGE_TIME).
const MESSAGE_TIME: f32 = 2.0;

/// The editor's mode: playing the map (the squad in control, as the test map always was) or
/// editing it (no character in the world, the edit camera). Play in the main game.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EditorMode {
    #[default]
    Play,
    Edit,
}

/// Whether the command line asks to start in edit mode (`--edit`, a test session).
pub fn requested() -> bool {
    std::env::args().any(|a| a == FLAG)
}

/// An editing tool: its key (and how the help names it), the panel it opens, its name and what
/// it's for (the edit mode's help).
pub(super) struct Tool {
    pub(super) key: KeyCode,
    pub(super) key_name: &'static str,
    pub(super) panel: Panel,
    pub(super) name: &'static str,
    pub(super) about: &'static str,
}

/// The tool registry: every editing tool's key and menu, in the help's order. The keys are the
/// test tools' (#111, #116, #185), none of them the game's controls. A new tool adds its entry
/// here and its `Panel`.
pub(super) const TOOLS: [Tool; 6] = [
    Tool { key: KeyCode::KeyP, key_name: "P", panel: Panel::Scenery, name: "scenery", about: "any level's buildings and static scenery" },
    Tool { key: KeyCode::KeyO, key_name: "O", panel: Panel::Objects, name: "objects", about: "pickups, weapons, grenades, breakables" },
    Tool { key: KeyCode::KeyN, key_name: "N", panel: Panel::Npcs, name: "NPCs", about: "any character type, friend or enemy" },
    Tool { key: KeyCode::KeyY, key_name: "Y", panel: Panel::Sky, name: "sky", about: "any level's sky" },
    Tool { key: KeyCode::KeyU, key_name: "U", panel: Panel::Music, name: "music", about: "any level's music" },
    Tool { key: KeyCode::KeyL, key_name: "L", panel: Panel::Levels, name: "level", about: "the map to build on (what's placed goes)" },
];

/// A switch between the modes, asked for (F5, a hook) and done as the state changes (`to_edit`,
/// `to_play`): to edit mode, the edit camera put there (None: where it was last, else where the
/// camera is); to play, the leader at a place facing a way (None: the map's start).
#[derive(Clone, Copy, Debug)]
enum Switch {
    ToEdit(Option<FreeCam>),
    ToPlay(Option<(Vec3, f32)>),
}

/// The camera's flight between the modes: where it set off from, and how long it's been going
/// (s). Each frame it's eased from there to wherever the mode's own camera puts the view.
#[derive(Clone, Copy)]
struct Flight {
    from: Transform,
    t: f32,
}

/// The editor's state. The session's: whether the help shows. The map's (`start_map`): the
/// switch asked for, the edit camera kept while playing, the camera's flight, where the cursor
/// was when the right button went down, and the hook's pointer.
#[derive(Resource)]
pub(super) struct Editor {
    switch: Option<Switch>,
    camera: Option<FreeCam>,
    flight: Option<Flight>,
    help: bool,
    look_from: Option<Vec2>,
    test_pointer: Option<Vec2>,
}

impl Default for Editor {
    fn default() -> Self {
        Editor { switch: None, camera: None, flight: None, help: true, look_from: None, test_pointer: None }
    }
}

/// The edit mode's help, shown in the tools' panel with no menu open (play_testtools.rs
/// `show_panel`).
#[derive(Resource, Default)]
pub(super) struct EditText(pub(super) String);

/// What a placed thing is: enough to make it again.
#[derive(Clone, Debug)]
pub(super) enum Thing {
    /// an object menu entry (play_testtools.rs): its label (the menu's, with its hash where two
    /// read the same) and what it places
    Object { label: String, kind: super::testtools::Kind },
    /// a level's static scenery (play_testscenery.rs): the level it's listed under, its object
    /// type and the archetype it's built from
    Scenery { zone: String, kind: u32, archetype: u32 },
    /// an NPC (play_testtools.rs): its number (`Npc::id`), character type, team and behaviour
    /// (fight, else a dummy; `frozen`: a dummy with its brain off), and the level placement it was
    /// spawned from (its weapons and personality), if one
    Npc { id: u32, kind: u32, team: u8, fight: bool, frozen: bool, placement: Option<u32> },
}

/// Where a placed thing is now (`track`): on the map; dead (an NPC's body); or gone (taken,
/// broken: its root is no more; an NPC no longer in `Squad`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Now {
    Here,
    Dead,
    Gone,
}

/// One placed thing: its number (in the order placed), what it is, where it was put (an object's
/// or a piece's lowest point or origin as the tool put it, an NPC's feet) and its turn (rad),
/// whether it was put in the air (to drop, or hang), its root in the world (objects and scenery;
/// an NPC is `Squad`'s, by its number), where it is now and whether its root has been seen yet
/// (it's made by the tool's commands).
pub(super) struct Placement {
    pub(super) id: u32,
    pub(super) thing: Thing,
    pub(super) at: Vec3,
    pub(super) yaw: f32,
    pub(super) air: bool,
    pub(super) entity: Option<Entity>,
    pub(super) now: Now,
    seen: bool,
}

/// Everything the tools have put on this map, the one list that owns it (see the module's
/// notes): the tools add to it, the NPC tool keeps an NPC's team and behaviour and strikes off
/// one it removes, `track` follows what play does to them. #187 (select, move, delete, undo) and
/// #188 (save, load) work on it.
#[derive(Resource, Default)]
pub(super) struct Placements {
    next: u32,
    pub(super) list: Vec<Placement>,
}

impl Placements {
    /// Lists a thing just placed; its number (1, 2, ...).
    pub(super) fn add(&mut self, thing: Thing, at: Vec3, yaw: f32, air: bool, entity: Option<Entity>) -> u32 {
        self.next += 1;
        let id = self.next;
        let p = Placement { id, thing, at, yaw, air, entity, now: Now::Here, seen: entity.is_none() };
        if std::env::var("BF_TOOLS_LOG").is_ok() {
            println!("placement {}", listing(&p));
        }
        self.list.push(p);
        id
    }

    /// An NPC's team or behaviour changed (J, B, the hooks): its listing keeps it.
    pub(super) fn set_npc(&mut self, npc: u32, to_team: u8, to_fight: bool) {
        for p in self.list.iter_mut() {
            if let Thing::Npc { id, team, fight, .. } = &mut p.thing {
                if *id == npc {
                    (*team, *fight) = (to_team, to_fight);
                }
            }
        }
    }

    /// An NPC taken off the map (Delete, the hooks): struck off the list.
    pub(super) fn remove_npc(&mut self, npc: u32) {
        self.list.retain(|p| !matches!(p.thing, Thing::Npc { id, .. } if id == npc));
    }

    /// "4 placed: 2 scenery, 1 object, 1 NPC (1 gone, 1 dead)".
    fn summary(&self) -> String {
        let n = |f: &dyn Fn(&Placement) -> bool| self.list.iter().filter(|p| f(p)).count();
        let scenery = n(&|p| matches!(p.thing, Thing::Scenery { .. }));
        let objects = n(&|p| matches!(p.thing, Thing::Object { .. }));
        let npcs = n(&|p| matches!(p.thing, Thing::Npc { .. }));
        let (gone, dead) = (n(&|p| p.now == Now::Gone), n(&|p| p.now == Now::Dead));
        let plural = |k: usize, one: &str, many: &str| format!("{k} {}", if k == 1 { one } else { many });
        let mut s = format!("{} placed: {} scenery, {}, {}", self.list.len(), scenery, plural(objects, "object", "objects"), plural(npcs, "NPC", "NPCs"));
        if gone + dead > 0 {
            s += &format!(" ({gone} gone, {dead} dead)");
        }
        s
    }
}

/// A placed thing in words, for the logs.
fn describe(thing: &Thing) -> String {
    match thing {
        Thing::Object { label, kind } => format!("object {label} ({kind:?})"),
        Thing::Scenery { zone, kind, archetype } => format!("scenery {zone} h_{kind:08x} (archetype h_{archetype:08x})"),
        Thing::Npc { id, kind, team, fight, frozen, placement } => {
            let how = if *fight { "fight" } else if *frozen { "frozen" } else { "dummy" };
            let from = placement.map_or(String::new(), |p| format!(", placement h_{p:08x}"));
            format!("NPC {id} (type h_{kind:08x}, team {team}, {how}{from})")
        }
    }
}

/// A placement in words, for the logs: its number, what it is, where it was put and turned, and
/// where it is now.
fn listing(p: &Placement) -> String {
    format!("{} {} at {:.2}{} turned {:.0} deg: {:?}", p.id, describe(&p.thing), p.at, if p.air { " (in the air)" } else { "" }, p.yaw.to_degrees(), p.now)
}

/// BF_TOOLS_LOG at a switch: what's placed, each thing with where it is now (nothing placed is
/// lost by a switch: the list is the same either side of it).
fn log_placements(placements: &Placements) {
    println!("  {}", placements.summary());
    for p in &placements.list {
        println!("    placement {}", listing(p));
    }
}

/// The placed thing a root in the world is (its `Placements` number).
#[derive(Component)]
pub(super) struct PlacedId(pub(super) u32);

/// The edit mode's status line (the tools' messages; the HUD's message line is hidden there).
#[derive(Component)]
struct EditStatus;

/// The pointer's cross, where the cursor doesn't show.
#[derive(Component)]
struct PointerMark;

/// The editor: its state (EditorMode::Edit at the start with `--edit`), the switches, its input,
/// the registry's keys, the placements' life, its help and its view. In a test session only.
pub fn plugin(app: &mut App) {
    app.insert_state(if requested() { EditorMode::Edit } else { EditorMode::Play })
        .init_resource::<Editor>().init_resource::<EditText>().init_resource::<Placements>()
        .add_systems(OnEnter(AppState::Playing), start_map.after(super::testtools::reset).run_if(resource_exists::<TestMap>))
        .add_systems(OnEnter(EditorMode::Edit), to_edit.run_if(super::testtools::active))
        .add_systems(OnEnter(EditorMode::Play), to_play.run_if(super::testtools::active))
        .add_systems(Update, edit_input.after(super::testtools::run_script).before(super::testtools::before_input).run_if(super::testtools::active))
        .add_systems(Update, track.after(update_player).run_if(super::testtools::active))
        .add_systems(Update, (help_text.before(super::testtools::show_panel), edit_view.after(super::testtools::show_panel).before(update_hud))
            .run_if(super::testtools::active));
}

/// The registry's keys (`TOOLS`): each opens its tool's menu, or closes it if it's the one open
/// (before the tools read their menus' keys).
pub(super) fn open_tools(keys: Res<ButtonInput<KeyCode>>, mut tools: ResMut<Tools>) {
    for t in &TOOLS {
        if keys.just_pressed(t.key) {
            tools.panel = if tools.panel == t.panel { Panel::None } else { t.panel };
        }
    }
}

/// The edit camera at a map's start (see START_VIEWS): behind and above the controlled
/// character's start, clear of the map's walls, looking at the start.
fn start_camera(player: &Player) -> FreeCam {
    let feet = player.position + Vec3::Y * GROUND;
    let eye = feet + Vec3::Y * START_EYE;
    let turn = Quat::from_rotation_y(player.yaw);
    // each view: where it is, and how far along the line from the eyes it's clear
    let views = START_VIEWS.map(|(back, up)| {
        let at = feet + turn * Vec3::new(0.0, 0.0, back) + Vec3::Y * up;
        let (dir, len) = ((at - eye).normalize_or(Vec3::Y), (at - eye).length());
        let room = world::arena().and_then(|a| a.ray(eye, dir, len)).unwrap_or(f32::MAX);
        (at, dir, len, room)
    });
    let at = match views.iter().find(|v| v.3 >= v.2) {
        Some(v) => v.0,
        None => {
            let v = views.iter().max_by(|a, b| a.3.total_cmp(&b.3)).expect("START_VIEWS");
            eye + v.1 * (v.3 - START_CLEAR).max(0.0)
        }
    };
    FreeCam::from_transform(&Transform::from_translation(at).looking_at(feet + Vec3::Y, Vec3::Y))
}

/// Where F5 puts the squad's leader in edit mode, and the way they face: where the pointer's ray
/// meets the ground (`ground_point`; on a roof or a ledge if that's what it meets), else the
/// ground under the edit camera; facing the camera's way.
fn play_spot(player: &Player, tools: &Tools) -> Option<(Vec3, f32)> {
    let c = tools.freecam?;
    let (origin, dir) = crosshair(player, tools);
    let at = ground_point(origin, dir).unwrap_or_else(|| Vec3::new(c.pos.x, floor_y(c.pos.x, c.pos.z, c.pos.y), c.pos.z));
    Some((at, c.yaw))
}

/// The squad leaves the world for edit mode: the controlled character's body and the squad's go
/// (and their slide puffs), the squad's members leave `Squad` (its NPCs stay, in their order),
/// and the stand-in takes the controlled character's place, at `at` (play.rs `Player::stand_in`;
/// it leads when play starts again). The hits on their way go: they're counted by place in the
/// squad.
fn bench(commands: &mut Commands, player: &mut Player, squad: &mut Squad, hits: &mut AiHits, at: Vec3) {
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut().filter(|m| m.in_squad())) {
        if let Some(l) = u.loaded.take() {
            commands.entity(l.root).despawn();
        }
        if let Some(e) = u.slide_fx.take() {
            commands.entity(e).despawn();
        }
    }
    squad.0.retain(|m| !m.in_squad());
    hits.0.clear();
    *player = Player::stand_in(player.character, player.sim_time, at);
}

/// Frees the mouse (edit mode: the cursor shows) or captures it (play: the mouse looks), unless
/// it's never to be grabbed (BF_BACKGROUND, play_background.rs).
fn set_capture(windows: &mut Query<&mut Window, With<PrimaryWindow>>, capture: bool) {
    let (false, Ok(mut w)) = (super::background::on(), windows.single_mut()) else { return };
    w.cursor_options.grab_mode = if capture { CursorGrabMode::Locked } else { CursorGrabMode::None };
    w.cursor_options.visible = !capture;
}

/// A map starts: nothing placed, no switch or flight under way, the edit camera from before
/// dropped (it was another map's); the edit mode's status line and pointer cross made. Starting
/// in edit mode (`--edit`, or a level switch while editing): the squad leaves at once, the edit
/// camera behind and above the start.
#[allow(clippy::too_many_arguments)]
fn start_map(mut commands: Commands, mode: Res<State<EditorMode>>, mut editor: ResMut<Editor>, mut placements: ResMut<Placements>,
             mut tools: ResMut<Tools>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut hits: ResMut<AiHits>,
             mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    *placements = Placements::default();
    *editor = Editor { help: editor.help, ..default() };
    commands.spawn((
        EditStatus,
        Text::new(""),
        TextFont { font_size: 15.0, ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.92)),
        TextLayout::new_with_justify(JustifyText::Center),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(10.0), left: Val::Px(0.0), right: Val::Px(0.0), display: Display::None,
               ..default() },
    ));
    let mark = commands.spawn((PointerMark, Visibility::Hidden,
                               Node { position_type: PositionType::Absolute, width: Val::Px(0.0), height: Val::Px(0.0), ..default() })).id();
    for (left, top, width, height) in [(-MARK_HALF, -MARK_THICK * 0.5, MARK_HALF * 2.0, MARK_THICK), (-MARK_THICK * 0.5, -MARK_HALF, MARK_THICK, MARK_HALF * 2.0)] {
        commands.spawn((ChildOf(mark), BackgroundColor(MARK_COLOR),
                        Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(top), width: Val::Px(width),
                               height: Val::Px(height), ..default() }));
    }
    tools.edit = *mode.get() == EditorMode::Edit;
    if !tools.edit {
        return;
    }
    let cam = start_camera(&player);
    bench(&mut commands, &mut player, &mut squad, &mut hits, cam.pos);
    tools.freecam = Some(cam);
    set_capture(&mut windows, false);
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        println!("t {:.2}: edit mode from the start: the edit camera at {:.2} yaw {:.0} pitch {:.0} deg", player.sim_time, cam.pos,
                 cam.yaw.to_degrees(), cam.pitch.to_degrees());
    }
}

/// Into edit mode (F5 in play, BF_TEST_EDIT): the squad leaves the world (`bench`), the menus
/// close, the mouse is freed, and the camera flies from where it is to the edit camera: the one
/// asked for, else where it was when play began, else where the camera is.
#[allow(clippy::too_many_arguments)]
fn to_edit(mut commands: Commands, mut editor: ResMut<Editor>, mut tools: ResMut<Tools>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
           mut hits: ResMut<AiHits>, mut status: ResMut<UsePanel>, placements: Res<Placements>, cam: Query<&Transform, With<MainCamera>>,
           mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let Some(Switch::ToEdit(pose)) = editor.switch.take() else { return };
    let from = cam.single().copied().unwrap_or_default();
    let camera = pose.or(editor.camera).unwrap_or_else(|| FreeCam::from_transform(&from));
    let (leader, members) = (player.character, squad.0.iter().filter(|m| m.in_squad()).count());
    bench(&mut commands, &mut player, &mut squad, &mut hits, camera.pos);
    tools.freecam = Some(camera);
    tools.panel = Panel::None;
    tools.edit = true;
    editor.flight = Some(Flight { from, t: 0.0 });
    // (the death camera's orbit, if the leader was dead, is over)
    commands.insert_resource(super::deathcam::DeathCam::default());
    set_capture(&mut windows, false);
    status.message = Some(("Editing: F5 plays from the pointer".into(), MESSAGE_TIME));
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        println!("t {:.2}: edit mode: {} and the squad ({members}) out of the world, {} NPCs stay; the edit camera at {:.2} yaw {:.0} pitch {:.0} deg",
                 player.sim_time, CHARACTERS[leader], squad.0.len(), camera.pos, camera.yaw.to_degrees(), camera.pitch.to_degrees());
        log_placements(&placements);
    }
}

/// Into play (F5 in edit mode, BF_TEST_PLAY): a fresh squad, its leader the character who last
/// led, at the place asked for (`player_at`, the others round them, `squad_around`) or at the
/// map's start (`player_at_start`, `squad_at_start`); the NPCs stay after them in `Squad`. The
/// edit camera is kept for coming back, the menus close, the squad's reserve is stocked again
/// (play_ammo.rs `stock`), the mouse is captured, and the camera flies down to the follow camera.
#[allow(clippy::too_many_arguments)]
fn to_play(mut commands: Commands, mut editor: ResMut<Editor>, mut tools: ResMut<Tools>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
           mut hits: ResMut<AiHits>, mut status: ResMut<UsePanel>, mode: Res<Mode>, placements: Res<Placements>,
           cam: Query<&Transform, With<MainCamera>>, mut windows: Query<&mut Window, With<PrimaryWindow>>,
           spawns: Option<Res<super::triggers::SquadSpawns>>) {
    let Some(Switch::ToPlay(spot)) = editor.switch.take() else { return };
    let from = cam.single().copied().unwrap_or_default();
    editor.camera = tools.freecam.take();
    tools.panel = Panel::None;
    tools.edit = false;
    tools.pointer = None;
    let leader = player.character;
    // (the level's squad spawns, play_triggers.rs: who comes and the names its triggers know)
    let none = super::triggers::SquadSpawns::default();
    let spawns = spawns.as_deref().unwrap_or(&none);
    let (mut p, mut members) = match spot {
        Some((at, yaw)) => {
            let mut p = super::player_at(leader, at, yaw);
            p.trigger_name = spawns.of(leader).map_or(0, |s| s.copy);
            (p, super::squad_around(leader, Some((at, yaw)), mode.deathmatch, spawns))
        }
        None => (super::player_at_start(leader, spawns), super::squad_at_start(leader, mode.deathmatch, spawns)),
    };
    p.sim_time = player.sim_time;
    p.show_help = player.show_help;
    let (fresh, npcs) = (members.0.len(), squad.0.len());
    members.0.append(&mut squad.0);
    *squad = members;
    *player = p;
    hits.0.clear();
    commands.insert_resource(super::ammo::Reserve::default());
    commands.insert_resource(super::deathcam::DeathCam::default());
    editor.flight = Some(Flight { from, t: 0.0 });
    set_capture(&mut windows, true);
    status.message = Some(("Playing: F5 goes back to editing".into(), MESSAGE_TIME));
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        let place = spot.map_or("the map's start".to_string(), |(at, yaw)| format!("{at:.2} facing {:.0} deg", yaw.to_degrees()));
        println!("t {:.2}: play from {place}: {} leads at {:.2}, {fresh} more in the squad, {npcs} NPCs", player.sim_time, CHARACTERS[leader],
                 player.position + Vec3::Y * GROUND);
        log_placements(&placements);
    }
}

/// The editor's input, before the tools read theirs: the mode's switch (F5, Shift+F5, the
/// hooks); in edit mode the mouse (the right button held looks, the cursor hidden and put back
/// on release), the pointer (`Tools::pointer`: the cursor's place, held while looking or off the
/// window; the hook's), the wheel's speed with no menu open, and H (the help).
#[allow(clippy::too_many_arguments)]
fn edit_input(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, scroll: Res<bevy::input::mouse::AccumulatedMouseScroll>,
              mode: Res<State<EditorMode>>, mut next: ResMut<NextState<EditorMode>>, mut editor: ResMut<Editor>, mut tools: ResMut<Tools>,
              script: Res<Script>, player: Res<Player>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let edit = *mode.get() == EditorMode::Edit;
    tools.edit = edit;
    for hook in &script.now {
        if let Hook::Pointer(p) = hook {
            editor.test_pointer = Some(*p);
        }
    }
    if edit {
        if let Ok(mut w) = windows.single_mut() {
            let size = Vec2::new(w.width(), w.height()).max(Vec2::ONE);
            tools.aspect = size.x / size.y;
            let ndc = |p: Vec2| Vec2::new(p.x / size.x * 2.0 - 1.0, 1.0 - p.y / size.y * 2.0);
            // (never under BF_BACKGROUND, whose mouse events are dropped: play_background.rs)
            if !super::background::on() {
                if mouse.just_pressed(MouseButton::Right) {
                    editor.look_from = w.cursor_position();
                    w.cursor_options.grab_mode = CursorGrabMode::Locked;
                    w.cursor_options.visible = false;
                } else if !mouse.pressed(MouseButton::Right) && w.cursor_options.grab_mode != CursorGrabMode::None {
                    w.cursor_options.grab_mode = CursorGrabMode::None;
                    w.cursor_options.visible = true;
                    if let Some(p) = editor.look_from.take() {
                        w.set_cursor_position(Some(p));
                    }
                }
            }
            let looking = w.cursor_options.grab_mode != CursorGrabMode::None;
            let cursor = if super::background::on() || looking { None } else { w.cursor_position() };
            if let Some(p) = editor.test_pointer {
                tools.pointer = Some(Vec2::new(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0));
            } else if let Some(c) = cursor.or(editor.look_from.filter(|_| looking)) {
                tools.pointer = Some(ndc(c));
            }
        }
        if tools.panel == Panel::None && scroll.delta.y != 0.0 {
            tools.speed = (tools.speed * SPEED_STEP.powf(scroll.delta.y.signum())).clamp(SPEED_RANGE.0, SPEED_RANGE.1);
        }
        if keys.just_pressed(KeyCode::KeyH) || script.now.iter().any(|h| matches!(h, Hook::Help)) {
            editor.help = !editor.help;
        }
    } else {
        tools.pointer = None;
    }
    // the switch: F5 (Shift+F5 in edit mode: from the map's start) or the hooks
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let mut ask = None;
    if keys.just_pressed(KeyCode::F5) {
        ask = Some(if edit { Switch::ToPlay(if shift { None } else { play_spot(&player, &tools) }) } else { Switch::ToEdit(None) });
    }
    for hook in &script.now {
        match hook {
            Hook::Edit(pose) if !edit => ask = Some(Switch::ToEdit(*pose)),
            Hook::Play { start } if edit => ask = Some(Switch::ToPlay(if *start { None } else { play_spot(&player, &tools) })),
            Hook::Edit(_) | Hook::Play { .. } => println!("test tools: {hook:?} ignored: in {:?} mode already", mode.get()),
            _ => {}
        }
    }
    if let (Some(s), None) = (ask, editor.switch) {
        editor.switch = Some(s);
        next.set(if matches!(s, Switch::ToEdit(_)) { EditorMode::Edit } else { EditorMode::Play });
    }
}

/// Where each placed thing is now (see `Now`), logged as it changes (BF_TOOLS_LOG): an object
/// or a piece by its root (still there, marked with its number), an NPC by its number in `Squad`.
fn track(mut placements: ResMut<Placements>, squad: Res<Squad>, roots: Query<&PlacedId>, player: Res<Player>) {
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    for p in placements.list.iter_mut() {
        let now = match (&p.thing, p.entity) {
            (Thing::Npc { id, .. }, _) => match squad.0.iter().find(|m| m.npc.is_some_and(|n| n.id == *id)) {
                Some(m) if m.dead => Now::Dead,
                Some(_) => Now::Here,
                None => Now::Gone,
            },
            (_, Some(e)) if roots.get(e).is_ok_and(|n| n.0 == p.id) => {
                p.seen = true;
                Now::Here
            }
            // (not made yet: its tool's commands are still to come)
            (_, Some(_)) if !p.seen => Now::Here,
            _ => Now::Gone,
        };
        if now != p.now {
            p.now = now;
            if log {
                println!("t {:.2}: placement {}", player.sim_time, listing(p));
            }
            if now == Now::Gone {
                p.entity = None;
            }
        }
    }
}

/// The edit mode's help (see `EditText`): the camera's keys, the registry's tools, the NPC keys,
/// F5, and what's placed; with H, one line.
fn help_text(tools: Res<Tools>, editor: Res<Editor>, placements: Res<Placements>, mut text: ResMut<EditText>) {
    let mut s = String::new();
    if tools.edit {
        s += "EDIT MODE   (F5: play from here)\n";
        if editor.help {
            s += "look: hold the right mouse button\n";
            s += "fly: WASD   up / down: E / Q (or Space / Ctrl)\n";
            s += &format!("Shift: faster   wheel: speed ({:.1} m/s)\n", tools.speed);
            for t in &TOOLS {
                s += &format!("{}  {}: {}\n", t.key_name, t.name, t.about);
            }
            let at = aim_word(&tools);
            s += &format!("J / B / Delete: the NPC under {at}: team / fight / remove\n");
            s += &format!("F5: play from {at} (Shift+F5: from the map's start)\n");
            s += "Esc: close a menu   H: hide this help   Backspace twice: leave\n";
        } else {
            s += "H: help\n";
        }
        s += &placements.summary();
    }
    if text.0 != s {
        text.0 = s;
    }
}

/// The editor's view each frame: the camera's flight between the modes; the HUD and the controls
/// panel hidden in edit mode (and while the camera flies down to play); in edit mode the stand-in
/// at the edit camera with the hooks' clock running on, the status line, the pointer's cross
/// where the cursor doesn't show, and the mark of where F5 puts the squad (no menu open).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn edit_view(time: Res<Time>, mut editor: ResMut<Editor>, tools: Res<Tools>, mut player: ResMut<Player>, status: Res<UsePanel>,
             mut cam: Query<&mut Transform, With<MainCamera>>, windows: Query<&Window, With<PrimaryWindow>>,
             mut hud: Query<&mut Visibility, (Or<(With<super::hud::HudRoot>, With<Hud>)>, Without<PointerMark>)>,
             mut line: Query<(&mut Text, &mut Node), (With<EditStatus>, Without<PointerMark>)>,
             mut mark: Query<(&mut Node, &mut Visibility), (With<PointerMark>, Without<EditStatus>)>, mut gizmos: Gizmos) {
    let dt = frame_dt(&time);
    // the flight: from where the view set off to the mode's own camera, eased in and out
    if let Some(f) = editor.flight.as_mut() {
        f.t += dt;
        let k = (f.t / FLIGHT_TIME).clamp(0.0, 1.0);
        let k = k * k * (3.0 - 2.0 * k);
        if let Ok(mut t) = cam.single_mut() {
            t.translation = f.from.translation.lerp(t.translation, k);
            t.rotation = f.from.rotation.slerp(t.rotation, k);
        }
        if f.t >= FLIGHT_TIME {
            editor.flight = None;
        }
    }
    let show = !tools.edit && editor.flight.is_none();
    for mut v in &mut hud {
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
    if let Ok((mut text, mut node)) = line.single_mut() {
        let s = status.message.as_ref().filter(|_| tools.edit).map_or(String::new(), |m| m.0.clone());
        let display = if s.is_empty() { Display::None } else { Display::Flex };
        if node.display != display {
            node.display = display;
        }
        if text.0 != s {
            text.0 = s;
        }
    }
    // the cross: in edit mode where the cursor doesn't show (looking, or none)
    let cursor = windows.single().is_ok_and(|w| w.cursor_options.visible && w.cursor_position().is_some()) && !super::background::on();
    if let Ok((mut node, mut v)) = mark.single_mut() {
        let want = if tools.edit && !cursor { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
        let p = tools.pointer.unwrap_or(Vec2::new(0.0, CROSSHAIR_UP));
        let (left, top) = (Val::Percent((p.x + 1.0) * 50.0), Val::Percent((1.0 - p.y) * 50.0));
        if node.left != left || node.top != top {
            (node.left, node.top) = (left, top);
        }
    }
    if !tools.edit {
        return;
    }
    // the stand-in: where the edit camera is, the hooks' clock running on
    if let Some(c) = tools.freecam {
        player.position = c.pos;
        player.prev_xz = c.pos.xz();
    }
    player.sim_time += dt;
    if tools.panel == Panel::None {
        if let Some((at, yaw)) = play_spot(&player, &tools) {
            let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let at = at + Vec3::Y * 0.03;
            gizmos.circle(Isometry3d::new(at, flat), SPOT_RING, SPOT_MARK);
            gizmos.arrow(at, at + Vec3::new(-yaw.sin(), 0.0, -yaw.cos()) * SPOT_ARROW, SPOT_MARK);
        }
    }
}
