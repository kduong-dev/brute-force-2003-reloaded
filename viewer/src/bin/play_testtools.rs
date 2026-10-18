//! The test map's developer tools (#111): demo tooling for trying features out, not part of the
//! game (no footage behind it; the keys, speeds and layout are the demo's choices). Only in a
//! test session (`cargo run --bin bf_play -- --test`), on the test map or any level it switched
//! to (play_testworld.rs: the sky, music and level menus, Y, U, L). None of the game's controls
//! use these keys.
//!
//!  - F: a free camera. The mouse looks (once captured), WASD flies along the view, Space up,
//!    Ctrl down, Shift four times faster; the character stands still meanwhile. Left click
//!    teleports the controlled character to the ground under the crosshair, right click to
//!    the ground under the camera, both back to the normal camera; F leaves it where it was.
//!  - N: the NPC menu. Up / Down (or the wheel) picks the character type, J the team (friend or
//!    enemy) and B the behaviour (dummy or fight) for the next one; Enter or a click spawns it on
//!    the ground under the crosshair, facing the controlled character. N or Esc closes it.
//!    The types (#128): the squad's four, then every other character type the loaded archives
//!    define (`Game::character_types`), by display name with its archive and HP.
//!    Spawned characters are `Player`s at the end of `Squad` with `npc` set: everything that
//!    acts on the squad acts on them (shots, blasts, Sentries, ragdolls, blood), but they're not
//!    in the formation, can't be taken control of and don't show on the HUD's portraits.
//!    - A type of the squad's is that squad slot's character. Any other type is slot OTHER: its
//!      model, clips, HP, weapons, flesh, blood decals and damage (its factors, hit parts and
//!      damage-multiplier, play_damage.rs) are its type's; what's still looked up by the
//!      squad's names (chatter, skills) reads nothing for it, so it's silent and has no skill
//!      (the species tickets, #129-#134).
//!    - Teams: the friend side is the squad's (team 0), the enemy side ENEMY_TEAM; the hooks can
//!      put one on any of the game's ten (`neutral`, NEUTRAL_TEAM, among them). An enemy sets
//!      off the squad's Sentries (play_sentry.rs); the crosshair turns red on one in its lock box
//!      and green on a friend under its centre (play_reticle.rs).
//!    - A dummy stands still where it was put (it doesn't dive from grenades), a target for the
//!      player only: the AI never fires at it (the product owner's choice: the squad shoots only
//!      enemies set to fight). One set to fight stays put (but dives from a grenade, as the
//!      squad does) and aims and fires at any hostile in sight with the squad AI's fire
//!      (`ai_fire`), and the squad AI fires back (`hostile_in_sight`): a squadmate's shots stop
//!      at the first body in their way, as the player's do. NPCs take no pickups.
//!    - Outside the menu J switches the team of the NPC under the crosshair, B its behaviour,
//!      Delete removes it; Shift+Delete removes every NPC.
//!  - O: the object menu, a list of every pickup type, every hand weapon (the rack's list),
//!    each grenade type and the breakable scenery. Up / Down picks (Page Up / Down jumps a
//!    group); a see-through copy follows the crosshair on the ground; the wheel turns it 15
//!    degrees a notch; a click (or Enter) puts one there, and the menu stays open for more. O or
//!    Esc closes it. In the free camera the copy hangs in the air on the crosshair's ray
//!    (AIR_DISTANCE, Shift + the wheel), a line down to where it will land: a loose one placed
//!    there drops (play_pickups.rs's physics), scenery hangs where it's put (#116).
//!    - A pickup is a level's inventory-object, as the test map's grid (play_pickups.rs:
//!      `LatePickup`): medkits and fruit can be taken, everything is loose.
//!    - A weapon lies on its side; walking into it takes it, as from the rack (play_testmap.rs),
//!      and it's gone.
//!    - A grenade lies as modelled; walking over it adds one of its type, up to its stack-limit
//!      (the test map starts with a full stack, so only once some are used).
//!    - Breakable scenery (BREAKABLES: the radiation barrel, the missile rack, the supply crate)
//!      joins play_scenery.rs's list as a map's does (`LateBreakable`): shots, blasts and other
//!      objects' damage areas break it, it explodes and chains. With no collision on the flat
//!      floor, shots meet its model's box and characters walk through it.
//!  - P: the scenery menu (play_testscenery.rs, #185): any level's buildings and static scenery,
//!    with the object menu's see-through copy, turn and air placement, drawn as its level draws
//!    it and joining the map's collision.
//!  - F5: the map editor's edit mode (play_editor.rs, #191): the squad leaves the world and the
//!    camera flies free (the right mouse button held looks), the tools acting at the mouse
//!    pointer; F5 again plays from where the pointer is. The menus' keys are the editor's tool
//!    registry's (`editor::TOOLS`), and what they put down is listed in its `Placements`.
//!
//! Not done: the enemy species' own behaviour (#110's sub-issues: their reactions, blood, death
//! clips), moving NPCs, removing placed objects, saving a layout.
//!
//! NPCs share the squad's characters, so what reaches a body later tells them apart by
//! `Player::who` (play_energy.rs's bolts, play_sonic.rs's rings), and a blast's thrower share
//! (play_grenade.rs, play_gas.rs) is the squad thrower's only.

use super::*;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::Placed;

/// The tools' line in the controls panel (H).
pub const HELP: &str = "tools:   F5 edit mode (the map editor: the squad out, a free camera, the tools at the mouse pointer; F5 again plays from there)\n\
                        F free camera (WASD fly, Space / Ctrl up / down, Shift faster; left click: teleport to the crosshair, right click: under the camera)\n\
                        N spawn NPC   O place object (in the free camera: in the air, Shift + wheel: distance)   J / B / Delete: team / fight / remove the NPC under the crosshair (Shift+Delete: all)\n\
                        P place any level's buildings and scenery   Y sky   U music (Left / Right: previous / next)   L switch level";
/// The enemy side's team (`Player::team`): any number but the squad's 0 and the game's
/// self-hostile 7 (play_sentry.rs); BF_TEST_HOSTILE uses the same.
pub const ENEMY_TEAM: u8 = 1;
/// A neutral NPC's team (the hooks' `neutral`): the game's ALIGN_NEUTRAL (the ALIGN_ table
/// 0x3bc94c). In the team table only PC_NO_TEAM (7) and it are hostile to each other, so the
/// squad's crosshair stays blue on one (#141 take08 2.7-5.6 s).
pub const NEUTRAL_TEAM: u8 = 2;
/// The free camera's speed (m/s), Shift's factor, and the mouse's turn (rad per pixel: the
/// follow camera's, read_input). The editor's wheel changes the speed (`Tools::speed`).
const FLY_SPEED: f32 = 6.0;
const FLY_FAST: f32 = 4.0;
const LOOK: f32 = 0.004;
/// The free camera looks no further up or down than this (rad).
const FLY_PITCH: f32 = 1.5;
/// How far (m) the crosshair reaches for the ground, an NPC or a placement.
pub(super) const PICK_RANGE: f32 = 300.0;
/// An NPC within this far (m, across) of the crosshair's ground point counts as under it (a
/// body on the ground, or a near miss).
const NPC_REACH: f32 = 1.2;
/// A notch of the wheel turns the object this far (rad): 15 degrees.
pub(super) const TURN_STEP: f32 = std::f32::consts::PI / 12.0;
/// The object's see-through copy: its colour's alpha, and a glow added so it reads as a preview
/// on dark and light ground alike.
pub(super) const GHOST_ALPHA: f32 = 0.6;
pub(super) const GHOST_GLOW: LinearRgba = LinearRgba::rgb(0.05, 0.15, 0.2);
/// How near (m, across; up or down) the feet must come to take a grenade off the ground:
/// play_pickups.rs's REACH / REACH_UP.
const GRENADE_REACH: f32 = 1.0;
const GRENADE_REACH_UP: f32 = 1.5;
/// How many lines of the object list show at once.
pub(super) const LIST_LINES: usize = 9;
/// How long a tool's message shows (s): the test map's.
pub(super) const MESSAGE_TIME: f32 = 2.0;
/// How far along the crosshair (m) the free camera puts an object in the air at first (the
/// demo's choice).
const AIR_DISTANCE: f32 = 4.0;
/// How far (m) a notch of Shift + the wheel moves it along the crosshair (the demo's).
pub(super) const AIR_STEP: f32 = 0.5;
/// The nearest and furthest (m) it goes along the crosshair (the demo's).
pub(super) const AIR_RANGE: (f32, f32) = (1.0, 40.0);
/// How far short (m) of what the crosshair's ray meets first it stops, so it isn't put inside a
/// wall or the floor (the demo's).
const AIR_CLEAR: f32 = 0.3;
/// An object whose lowest point would be less than this (m) above the ground is put on it.
pub(super) const AIR_MIN_HEIGHT: f32 = 0.05;
/// The preview's mark for an object that drops (the demo's): a cyan line down to where it lands
/// and a ring there.
const DROP_MARK: Color = Color::srgb(0.3, 0.85, 1.0);
/// The preview's mark for one that hangs where it's put (scenery; the demo's): a yellow ring
/// round its foot.
pub(super) const HANG_MARK: Color = Color::srgb(1.0, 0.8, 0.25);
/// The marks' ring radius (m, the demo's).
pub(super) const MARK_RING: f32 = 0.35;


/// A character the NPC tool spawned (`Player::npc`): its number (1, 2, ... in the order
/// spawned; the hooks' `<n>`), whether it fights, and the way it was put facing.
#[derive(Clone, Copy, Debug)]
pub struct Npc {
    pub id: u32,
    pub fight: bool,
    pub face: f32,
}

/// The demo's yaw (0 faces -z, `Player::yaw`) for a facing given as the xemu watch logs give
/// it: a bearing in degrees, atan2(dz, dx) of the way it faces, so 0 faces +x and 90 faces +z
/// (xemu.md: the character's yaw +0x434 is 90 degrees less the bearing). BF_TEST_NPC's
/// optional facing (#167).
fn bearing_yaw(bearing: f32) -> f32 {
    let (dz, dx) = bearing.to_radians().sin_cos();
    (-dx).atan2(-dz)
}

/// The free camera: where it is and where it looks (yaw about +Y, 0 down -z; pitch up). The
/// play mode's F and the map editor's edit camera (play_editor.rs).
#[derive(Clone, Copy)]
pub(super) struct FreeCam {
    pub(super) pos: Vec3,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
}

impl FreeCam {
    pub(super) fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    /// The ray through the HUD's crosshair, which sits CROSSHAIR_UP above the middle (as
    /// `aim_ray`).
    fn ray(&self) -> Vec3 {
        self.ray_through(Vec2::new(0.0, CROSSHAIR_UP), 1.0)
    }

    /// The ray through a point of the view: `ndc` from -1 (left, bottom) to 1 (right, top), the
    /// view `aspect` (width / height) wide, FOV_Y high (the editor's mouse pointer).
    pub(super) fn ray_through(&self, ndc: Vec2, aspect: f32) -> Vec3 {
        let t = (FOV_Y * 0.5).tan();
        (self.rotation() * Vec3::new(ndc.x * t * aspect, ndc.y * t, -1.0)).normalize()
    }

    /// From where a camera is, looking its way (no roll).
    pub(super) fn from_transform(t: &Transform) -> FreeCam {
        let f = t.forward();
        FreeCam { pos: t.translation, yaw: (-f.x).atan2(-f.z), pitch: f.y.clamp(-1.0, 1.0).asin() }
    }
}

/// Which menu is open (Sky, Music, Levels: play_testworld.rs; Scenery: play_testscenery.rs).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(super) enum Panel {
    #[default]
    None,
    Npcs,
    Objects,
    Sky,
    Music,
    Levels,
    Scenery,
}

/// The player's camera and crouch, and whether the mouse was captured, before `read_input`
/// (the free camera keeps the character's camera as it was; the menus keep the wheel's zoom;
/// the click that captures the mouse doesn't place or spawn).
#[derive(Clone, Copy, Default)]
pub(super) struct Before {
    yaw: f32,
    pitch: f32,
    distance: f32,
    crouch: bool,
    pub(super) captured: bool,
}

/// The tools' state (a map's: `reset` as each map starts).
#[derive(Resource, Default)]
pub(super) struct Tools {
    pub(super) freecam: Option<FreeCam>,
    pub(super) panel: Panel,
    /// the NPC menu: the entry picked (an index into `NpcList`), the team and behaviour of the
    /// next one
    npc_type: usize,
    npc_team: u8,
    npc_fight: bool,
    /// the NPCs spawned so far (the last one's number)
    spawned: u32,
    /// the object menu: the entry picked (into `Catalogue`) and its turn (rad)
    object: usize,
    yaw: f32,
    /// the see-through copy and the entry it shows
    ghost: Option<(Entity, usize)>,
    /// how far along the crosshair (m) the free camera puts an object in the air (AIR_DISTANCE
    /// at first; Shift + the wheel): the object menu's and the scenery menu's
    pub(super) air: f32,
    pub(super) before: Before,
    /// BF_TEST_FLY: until when (sim time), the move (right, up, forward), fast
    fly: Option<(f32, Vec3, bool)>,
    /// the map editor's edit mode is on (play_editor.rs `EditorMode::Edit`, set from it each
    /// frame): `freecam` is the edit camera, and the tools act at the mouse pointer
    pub(super) edit: bool,
    /// in edit mode, where the mouse pointer is in the view (-1..1 across, left to right, and up,
    /// bottom to top; None: no pointer, so the crosshair's place, as under BF_BACKGROUND), and
    /// the view's width over its height
    pub(super) pointer: Option<Vec2>,
    pub(super) aspect: f32,
    /// a click of the left button for the world this frame: in play, not the one that captures
    /// the mouse; in edit mode, not one on the tools' panel (`before_input`)
    pub(super) click: bool,
    /// the free camera's speed (m/s; FLY_SPEED at first, the editor's wheel changes it)
    pub(super) speed: f32,
}

/// What an object entry places (see the module's notes).
#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    /// an inventory-object type (a `Game::items` key)
    Pickup(u32),
    /// a hand weapon (a `Game::weapons` key)
    Weapon(u32),
    /// a grenade type (a `GrenadeKits` index)
    Grenade(usize),
    /// breakable scenery (a `Game::breakable` key, see BREAKABLES)
    Breakable(u32),
}

/// The breakable scenery the object menu has: sdm_e34's three kinds (play_scenery.rs' notes),
/// by object type, with the names the demo gives them.
const BREAKABLES: [(&str, u32); 3] = [("Radiation barrel", 0xE04E_5A0E), ("Missile rack", 0xFBDC_D828), ("Supply crate", 0x09A6_856D)];

/// One entry of the object menu: its name, its group's name, what it places, its model, the
/// pose it's put in (`rest`, before its turn) and how far its origin sits above the ground in
/// that pose (its lowest vertex).
struct Entry {
    label: String,
    group: &'static str,
    kind: Kind,
    model: WeaponModel,
    rest: Quat,
    lift: f32,
}

/// The object menu's entries, in groups: pickups, weapons, grenades, scenery (built once the
/// grenade types are loaded).
#[derive(Resource, Default)]
pub(super) struct Catalogue(Vec<Entry>);

/// A grenade put on the ground by the object tool: its type (a `GrenadeKits` index).
#[derive(Component)]
struct GroundGrenade(usize);

/// The see-through copy that follows the crosshair.
#[derive(Component)]
struct Ghost;

/// The tools' panel (right of the middle of the screen, clear of the HUD's radar and panels).
#[derive(Component)]
pub(super) struct ToolPanel;

/// Which NPC a hook (or key) acts on: the n-th spawned, the one under the crosshair, or all.
#[derive(Clone, Copy, Debug)]
pub(super) enum Pick {
    Nth(u32),
    Aim,
    All,
}

/// A test hook's action (see `Script`; Sky to Level: play_testworld.rs).
#[derive(Clone, Debug)]
pub(super) enum Hook {
    Help,
    Freecam(Option<FreeCam>),
    FreecamOff,
    Fly(f32, Vec3, bool),
    Teleport { camera: bool },
    NpcMenu,
    Close,
    /// `character`: the type as the hook names it (see `npc_kind`); `at`: x, z, the height the
    /// ground is looked for under (None: 50 m over the controlled character) and the yaw it faces
    /// (the demo's, radians, from the hook's bearing: `bearing_yaw`; None: facing the controlled
    /// character); `frozen`: a dummy whose brain is off (play_enemies.rs: no hit reactions)
    Npc { character: String, team: u8, fight: bool, frozen: bool, at: Option<(Vec2, Option<f32>, Option<f32>)> },
    /// the team to put them on (None: switch friend <-> enemy, as J does)
    Team(Pick, Option<u8>),
    /// the radar-contact to give them (`Player::radar_contact`)
    Radar(Pick, u8),
    /// drive them in a motion type to a point (play_enemies.rs `drive`): the motion type, the
    /// point (x, z; None: in place), the ready-script's clips (else the script's)
    Motion(Pick, i64, Option<Vec2>, bool),
    /// the hit points to give them, both the current and the maximum (#35 take04's trainer `hp
    /// spawned 999 999`, so a target outlives a long burst)
    Health(Pick, f32),
    Fight(Pick),
    Remove(Pick),
    Placer { object: String, yaw: f32 },
    /// `at`: x, z, yaw and a height above the ground (in the air, to drop)
    Place { object: Option<String>, at: Option<(Vec2, f32, f32)> },
    /// the free camera's air distance (m)
    Air(f32),
    /// P: the scenery menu (play_testscenery.rs)
    SceneryMenu,
    /// P with a piece picked (`<level>:<piece>`, play_testscenery.rs `find_piece`), turned `yaw`
    Scenery { piece: String, yaw: f32 },
    /// the scenery menu's click (None: the picked piece at the crosshair), or that piece put at
    /// x, z turned `yaw`, a height above the ground (in the air)
    SceneryPlace { piece: Option<String>, at: Option<(Vec2, f32, f32)> },
    SkyMenu,
    MusicMenu,
    LevelMenu,
    /// a sky by level (`none`, `map`), a music bank by level (`off`, `map`, `next`, `prev`),
    /// a level to switch to
    Sky(String),
    Music(String),
    Level(String),
    /// the map editor (play_editor.rs): edit mode, the edit camera put there (None: where it
    /// last was, else where the camera is)
    Edit(Option<FreeCam>),
    /// play from where the pointer is; `start`: from the map's start
    Play { start: bool },
    /// the edit mode's mouse pointer put at this place of the window (0..1 across and down
    /// from the top left), as if the mouse were there
    Pointer(Vec2),
}

impl std::fmt::Debug for FreeCam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1} yaw {:.2} pitch {:.2}", self.pos, self.yaw, self.pitch)
    }
}

/// The test hooks' actions, by time (the controlled character's sim time), read when the map
/// starts; `now` holds those due this frame. The hooks (times in s; several entries of one
/// hook are separated by `;`; yaw and pitch in degrees):
///  - BF_TEST_HELP=<s>[;<s>...]: H.
///  - BF_TEST_FREECAM=<s>[,<x>,<y>,<z>,<yaw>,<pitch>]: F, the free camera from where the camera
///    is, or put there. BF_TEST_FREECAM_OFF=<s>: F again (no teleport).
///  - BF_TEST_FLY=<from>,<to>,<right>,<up>,<forward>[,fast]: fly that way (each -1..1) then.
///  - BF_TEST_TELEPORT=<s>[,camera]: the left click in the free camera (teleport to the ground
///    under the crosshair); `camera`: the right click (under the camera).
///  - BF_TEST_NPC_MENU=<s>: N (the menu opens). BF_TEST_TOOL_CLOSE=<s>: Esc (a menu closes).
///  - BF_TEST_NPC=<s>,<character>,<friend|enemy|neutral|team>,<dummy|fight|frozen>[,<x>,<z>[,<y>[,<bearing>]]]: spawn
///    one on the ground at x, z, or under the crosshair, facing the controlled character, or
///    `bearing` (degrees, the xemu watch logs' atan2(dz, dx): 0 faces +x, 90 faces +z; an empty y
///    keeps its default: `x,z,,90`; #167) when given. Its team:
///    friend 0, enemy ENEMY_TEAM, neutral NEUTRAL_TEAM, or a number 0-9 (`team_of`). The character: a
///    squad name or 0-3, or any loaded character type (#128) by its display name (any case; the
///    first loaded of that name, e.g. `shrike soldier`), `name@archive` (`shrike soldier@m09_a`)
///    or `name@library` (`shrike soldier@sdm_shrike`), its `<base name>` (`mp_brutus`) or its key
///    `h_xxxxxxxx` (`npc_kind`). The ground is the
///    first floor under y (world height), by default 50 m above the controlled character: a y
///    just over the floor wanted puts it under a roof or a gate's top (#73's takes 14, 16, 22
///    and 23 at sdm_e34's gate).
///  - BF_TEST_NPC_TEAM / BF_TEST_NPC_FIGHT / BF_TEST_NPC_REMOVE=<s>,<n|aim|all>: switch the
///    team / the behaviour of, or remove, the n-th NPC spawned, the one under the crosshair, or
///    every one. BF_TEST_NPC_TEAM=<s>,<n|aim|all>,<team> puts them on that team instead (as
///    BF_TEST_NPC names one).
///  - BF_TEST_NPC_RADAR=<s>,<n|aim|all>,<value>: gives them that radar-contact (every character
///    has 1; 2 or 4 turns the crosshair yellow on them, play_reticle.rs), as #141's take08 rewrote
///    the Shrike's.
///  - BF_TEST_NPC_MOTION=<s>,<n|aim|all>,<motion>[,<x>,<z>[,relaxed]]: drives an enemy NPC (a type
///    of its own, #129) in that motion type (MT_ name or number: walk, run, dash, move_left,
///    move_right_back_slow, backpedal, backstep, dizzy, knockdown, dodge_left ...) to x, z, or in
///    place: its MOTION table's clip (the ready-script's, the gun raised as the takes' Shrikes
///    hold it; `relaxed`: the script's) at its type's speed for the motion type
///    (play_enemies.rs). A motion that plays once (a reaction, a dodge) plays through. The
///    fighting AI that moves enemies in the game isn't in the demo.
///  - BF_TEST_NPC's character may also be a placement of a loaded level by its name
///    (`h_1c5780f8`: `<character-object name>`, #129): its type, with the placement's weapons and
///    personality (e34's Shrikes' A50s, Saryl-45s, Rail RVRs, and h_0bb586cf's knock-downs).
///  - BF_TEST_NPC_HP=<s>,<n|aim|all>,<hp>: gives them that many hit points, now and at most (once
///    they're loaded: a spawn's own come with its model the frame after), as #35 take04's trainer
///    gave its Shrike 999. (An enemy's heavy-hit and blood-decal thresholds are shares of its max
///    hit points, so at 999 a Shrike takes no heavy hit and lays almost no decals: play_enemies.rs,
///    play_fx.rs.)
///  - BF_TEST_PLACER=<s>,<object>[,<yaw>]: O with that object picked (by its label, any case;
///    else the first whose label contains it), turned that way: its copy follows the crosshair.
///  - BF_TEST_PLACE=<s>[,<object>,<x>,<z>[,<yaw>[,<height>]]]: a click with the object menu open
///    (the picked object at the crosshair: in the air in the free camera); or that object put at
///    x, z turned that way, `height` m above the ground (in the air: it drops, scenery hangs).
///  - BF_TEST_AIR=<s>,<m>: the free camera's air distance (Shift + the wheel).
///  - BF_TEST_SCENERY_MENU=<s>: P (the scenery menu opens, play_testscenery.rs).
///    BF_TEST_SCENERY=<s>,<level>:<piece>[,<yaw>]: P with that level's piece picked (its type or
///    archetype `h_xxxxxxxx`, or its place in the level's list, 0 the largest), turned that way.
///    BF_TEST_SCENERY_PLACE=<s>[,<level>:<piece>[,<x>,<z>[,<yaw>[,<height>]]]]: the scenery
///    menu's click (the picked piece at the crosshair), that piece at the crosshair, or that piece
///    put at x, z, `height` m up (it hangs there); each placed once its archive is loaded and
///    it's built.
///  - BF_TEST_SKY_MENU / BF_TEST_MUSIC_MENU / BF_TEST_LEVEL_MENU=<s>: Y / U / L (the menu opens).
///  - BF_TEST_SKY=<s>,<level|none|map>: that level's sky (by its archive's name, e.g. sdm_e10),
///    none, or the map's own.
///  - BF_TEST_MUSIC=<s>,<level|off|map|next|prev>: that level's music bank, none, the map's
///    own, or the next / previous bank of the list (the music menu's Right / Left).
///  - BF_TEST_LEVEL=<s>,<level>[;<s>,<level>...]: switch to that level (`flat` or `test`: the
///    test map). Each entry is for one map in turn: the first is due on the first map, the second
///    on the map it went to, and so on, each on its own map's clock.
///  - BF_TEST_EDIT=<s>[,<x>,<y>,<z>,<yaw>,<pitch>]: F5 in play, the map editor's edit mode
///    (play_editor.rs), the edit camera put there (else where it last was, else where the camera
///    is). BF_TEST_PLAY=<s>[,start]: F5 in edit mode, play from where the pointer is (`start`:
///    Shift+F5, from the map's start). BF_TEST_POINTER=<s>,<x>,<y>: the edit mode's pointer at
///    that place of the window (0..1 across and down from the top left), as if the mouse were
///    there. In edit mode BF_TEST_FREECAM's pose moves the edit camera.
///
/// Every hook but BF_TEST_LEVEL acts on one map of the session only (a switch would otherwise
/// replay them on each map): the first, or the n-th with BF_TEST_ON_MAP=<n> (the tools on a map
/// a switch went to). BF_TOOLS_LOG=1 prints each step as it's due, and what it did (a
/// teleport's place, an NPC spawned, switched or removed, an object placed, a sky or music
/// bank, a switch and what each map starts and ends with).
#[derive(Resource, Default)]
pub(super) struct Script {
    steps: Vec<(f32, Hook)>,
    next: usize,
    pub(super) now: Vec<Hook>,
}

/// The test session (from `--test` to the window closing, across level switches): how many
/// maps have started, the first 1.
#[derive(Resource, Default)]
pub(super) struct Session {
    pub(super) maps: usize,
}

/// The tools' systems: the hooks and the mouse first, then (after `read_input`) the menus' keys
/// (the editor's tool registry, `editor::open_tools`) and the tools, and late the free camera
/// and the panel.
pub fn plugin(app: &mut App) {
    app.init_resource::<Tools>().init_resource::<Catalogue>().init_resource::<NpcList>().init_resource::<Script>().init_resource::<Session>()
        .add_systems(OnEnter(AppState::Playing), (reset, spawn_panel).after(setup).run_if(resource_exists::<TestMap>))
        .add_systems(Update, (run_script, before_input).chain().before(read_input).run_if(active))
        .add_systems(Update, (gate_input, super::editor::open_tools, camera_tool, npc_tool, object_tool).chain().after(read_input).before(squad_control)
            .run_if(active))
        .add_systems(Update, take_grenades.after(update_player).run_if(active))
        .add_systems(Update, (fly, show_panel).chain().after(deathcam::death_cam).before(update_hud).run_if(active));
}

/// The tools run on the test map, while it's played.
pub(super) fn active(state: Res<State<AppState>>, test: Option<Res<TestMap>>) -> bool {
    *state.get() == AppState::Playing && test.is_some()
}

use super::testmap::TestMap;

/// A new map: the tools closed, nothing spawned, the object and NPC lists to be built again (a
/// level switch loads more data), the hooks read (all of them on the session's first map, else
/// only its BF_TEST_LEVEL entry).
pub(super) fn reset(mut tools: ResMut<Tools>, mut script: ResMut<Script>, mut hits: ResMut<AiHits>, mut list: ResMut<Catalogue>,
                    mut npcs: ResMut<NpcList>, mut session: ResMut<Session>) {
    *tools = Tools { npc_team: ENEMY_TEAM, air: AIR_DISTANCE, speed: FLY_SPEED, aspect: 1.0, ..default() };
    // (built again once this map's grenade types are in: its entries index them)
    list.0.clear();
    npcs.0.clear();
    hits.0.clear();
    session.maps += 1;
    *script = Script { steps: read_hooks(session.maps), next: 0, now: vec![] };
    if !script.steps.is_empty() {
        println!("test tools: {} hook steps: {:?}", script.steps.len(), script.steps);
    }
}

/// The character type a hook or the menu names: a squad slot's (0-3) or any loaded type by name
/// (`Game::find_character_type`: brutus, mp_brutus, a display name, `name@archive` or
/// `name@library`, h_xxxxxxxx).
fn npc_kind(game: &Game, s: &str) -> Option<u32> {
    let s = s.trim();
    s.parse::<usize>().ok().filter(|&i| i < SQUAD).map(squad_type).or_else(|| game.find_character_type(s))
}

/// A placement of the loaded levels named by a hook (`h_xxxxxxxx`, its `<character-object name>`)
/// whose type is loaded: what BF_TEST_NPC spawns with its weapons and personality (#129).
fn npc_placement(game: &Game, s: &str) -> Option<bf_viewer::bf::character_type::Placement> {
    let k = s.trim().strip_prefix("h_").and_then(|x| u32::from_str_radix(x, 16).ok())?;
    game.placements.iter().find(|p| p.name == k && game.character_type(p.kind).is_some()).cloned()
}

/// The squad slot of character type `kind`: the squad member's whose type it is, else OTHER.
fn slot_of(kind: u32) -> usize {
    (0..SQUAD).find(|&c| squad_type(c) == kind).unwrap_or(OTHER)
}

/// `n`, `aim` or `all`.
fn pick(s: &str) -> Option<Pick> {
    match s.trim().to_ascii_lowercase().as_str() {
        "aim" => Some(Pick::Aim),
        "all" => Some(Pick::All),
        n => n.parse().ok().map(Pick::Nth),
    }
}

/// The hooks' steps (see `Script`) for the session's `map`-th map (the first is 1), by time.
fn read_hooks(map: usize) -> Vec<(f32, Hook)> {
    let mut out = vec![];
    let entries = |k: &str| -> Vec<Vec<String>> {
        std::env::var(k).ok().map(|v| v.split(';').map(|e| e.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>())
            .filter(|e| !e[0].is_empty()).collect()).unwrap_or_default()
    };
    let num = |e: &[String], i: usize| e.get(i).and_then(|x| x.parse::<f32>().ok());
    let mut add = |at: Option<f32>, hook: Option<Hook>| if let (Some(at), Some(hook)) = (at, hook) { out.push((at, hook)) };
    // (the map's own switch; the rest are one map's: the first, or BF_TEST_ON_MAP's)
    if let Some(e) = entries("BF_TEST_LEVEL").get(map.wrapping_sub(1)) {
        add(num(e, 0), e.get(1).map(|l| Hook::Level(l.clone())));
    }
    if map != std::env::var("BF_TEST_ON_MAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1) {
        out.sort_by(|a, b| a.0.total_cmp(&b.0));
        return out;
    }
    for e in entries("BF_TEST_HELP") {
        for t in &e {
            add(t.parse().ok(), Some(Hook::Help));
        }
    }
    // (a pose: x, y, z, yaw and pitch in degrees, from the entry's second value)
    let pose = |e: &[String]| (|| Some(FreeCam { pos: Vec3::new(num(e, 1)?, num(e, 2)?, num(e, 3)?), yaw: num(e, 4)?.to_radians(), pitch: num(e, 5)?.to_radians() }))();
    for e in entries("BF_TEST_FREECAM") {
        add(num(&e, 0), Some(Hook::Freecam(pose(&e))));
    }
    for e in entries("BF_TEST_EDIT") {
        add(num(&e, 0), Some(Hook::Edit(pose(&e))));
    }
    for e in entries("BF_TEST_PLAY") {
        add(num(&e, 0), Some(Hook::Play { start: e.get(1).is_some_and(|x| x.eq_ignore_ascii_case("start")) }));
    }
    for e in entries("BF_TEST_POINTER") {
        add(num(&e, 0), num(&e, 1).zip(num(&e, 2)).map(|(x, y)| Hook::Pointer(Vec2::new(x, y))));
    }
    for e in entries("BF_TEST_FREECAM_OFF") {
        add(num(&e, 0), Some(Hook::FreecamOff));
    }
    for e in entries("BF_TEST_FLY") {
        let v = (|| Some(Vec3::new(num(&e, 2)?, num(&e, 3)?, num(&e, 4)?)))();
        add(num(&e, 0), v.zip(num(&e, 1)).map(|(v, to)| Hook::Fly(to, v, e.get(5).is_some_and(|x| x == "fast"))));
    }
    for e in entries("BF_TEST_TELEPORT") {
        add(num(&e, 0), Some(Hook::Teleport { camera: e.get(1).is_some_and(|x| x == "camera") }));
    }
    for e in entries("BF_TEST_NPC_MENU") {
        add(num(&e, 0), Some(Hook::NpcMenu));
    }
    for e in entries("BF_TEST_TOOL_CLOSE") {
        add(num(&e, 0), Some(Hook::Close));
    }
    for e in entries("BF_TEST_NPC") {
        // (a team it doesn't know: a friend, as before the team numbers, with a warning)
        if let Some(t) = e.get(2).filter(|t| team_of(t).is_none()) {
            warn!("BF_TEST_NPC: {t} isn't friend, enemy, neutral or a team 0-9; spawned as a friend");
        }
        let hook = (|| Some(Hook::Npc {
            character: e.get(1)?.clone(),
            team: team_of(e.get(2)?).unwrap_or(0),
            fight: e.get(3)?.eq_ignore_ascii_case("fight"),
            frozen: e.get(3)?.eq_ignore_ascii_case("frozen"),
            at: num(&e, 4).zip(num(&e, 5)).map(|(x, z)| (Vec2::new(x, z), num(&e, 6), num(&e, 7).map(bearing_yaw))),
        }))();
        add(num(&e, 0), hook);
    }
    for e in entries("BF_TEST_NPC_TEAM") {
        // (a team named: that one, or nothing if it isn't one; none: the J key's switch)
        let team = match e.get(2) {
            Some(t) => match team_of(t) {
                Some(team) => Some(team),
                None => {
                    warn!("BF_TEST_NPC_TEAM: {t} isn't friend, enemy, neutral or a team 0-9; skipped");
                    continue;
                }
            },
            None => None,
        };
        add(num(&e, 0), e.get(1).and_then(|p| pick(p)).map(|p| Hook::Team(p, team)));
    }
    for e in entries("BF_TEST_NPC_RADAR") {
        let value = e.get(2).and_then(|v| v.parse::<u8>().ok());
        add(num(&e, 0), e.get(1).and_then(|p| pick(p)).zip(value).map(|(p, v)| Hook::Radar(p, v)));
    }
    for e in entries("BF_TEST_NPC_MOTION") {
        let mt = e.get(2).and_then(|m| super::enemies::motion_type(m));
        if mt.is_none() {
            warn!("BF_TEST_NPC_MOTION: {:?} isn't a motion type; skipped", e.get(2));
        }
        let goal = num(&e, 3).zip(num(&e, 4)).map(|(x, z)| Vec2::new(x, z));
        let ready = !e.iter().any(|x| x.eq_ignore_ascii_case("relaxed"));
        add(num(&e, 0), e.get(1).and_then(|p| pick(p)).zip(mt).map(|(p, mt)| Hook::Motion(p, mt, goal, ready)));
    }
    for e in entries("BF_TEST_NPC_HP") {
        add(num(&e, 0), e.get(1).and_then(|p| pick(p)).zip(num(&e, 2)).map(|(p, hp)| Hook::Health(p, hp)));
    }
    for (k, make) in [("BF_TEST_NPC_FIGHT", Hook::Fight as fn(Pick) -> Hook), ("BF_TEST_NPC_REMOVE", Hook::Remove)] {
        for e in entries(k) {
            add(num(&e, 0), e.get(1).and_then(|p| pick(p)).map(make));
        }
    }
    for e in entries("BF_TEST_PLACER") {
        add(num(&e, 0), e.get(1).map(|o| Hook::Placer { object: o.clone(), yaw: num(&e, 2).unwrap_or(0.0).to_radians() }));
    }
    for e in entries("BF_TEST_PLACE") {
        let at = num(&e, 2).zip(num(&e, 3)).map(|(x, z)| (Vec2::new(x, z), num(&e, 4).unwrap_or(0.0).to_radians(), num(&e, 5).unwrap_or(0.0)));
        add(num(&e, 0), Some(Hook::Place { object: e.get(1).cloned(), at }));
    }
    for e in entries("BF_TEST_AIR") {
        add(num(&e, 0), num(&e, 1).map(Hook::Air));
    }
    for e in entries("BF_TEST_SCENERY_MENU") {
        add(num(&e, 0), Some(Hook::SceneryMenu));
    }
    for e in entries("BF_TEST_SCENERY") {
        add(num(&e, 0), e.get(1).map(|p| Hook::Scenery { piece: p.clone(), yaw: num(&e, 2).unwrap_or(0.0).to_radians() }));
    }
    for e in entries("BF_TEST_SCENERY_PLACE") {
        let at = num(&e, 2).zip(num(&e, 3)).map(|(x, z)| (Vec2::new(x, z), num(&e, 4).unwrap_or(0.0).to_radians(), num(&e, 5).unwrap_or(0.0)));
        add(num(&e, 0), Some(Hook::SceneryPlace { piece: e.get(1).cloned(), at }));
    }
    for (k, hook) in [("BF_TEST_SKY_MENU", Hook::SkyMenu), ("BF_TEST_MUSIC_MENU", Hook::MusicMenu), ("BF_TEST_LEVEL_MENU", Hook::LevelMenu)] {
        for e in entries(k) {
            add(num(&e, 0), Some(hook.clone()));
        }
    }
    for e in entries("BF_TEST_SKY") {
        add(num(&e, 0), e.get(1).map(|l| Hook::Sky(l.clone())));
    }
    for e in entries("BF_TEST_MUSIC") {
        add(num(&e, 0), e.get(1).map(|l| Hook::Music(l.clone())));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// The hooks due this frame into `Script::now`.
pub(super) fn run_script(player: Res<Player>, mut script: ResMut<Script>) {
    script.now.clear();
    while let Some((at, hook)) = script.steps.get(script.next).cloned() {
        if player.sim_time < at {
            break;
        }
        script.next += 1;
        if std::env::var("BF_TOOLS_LOG").is_ok() {
            println!("t {:.2}: hook {hook:?}", player.sim_time);
        }
        script.now.push(hook);
    }
}

/// Keeps the player's camera, crouch and the mouse's capture as they are before `read_input`,
/// and whether the left button clicked in the world (`Tools::click`): in play, not the click
/// that captures the mouse; in edit mode (the mouse free), not one on the tools' panel.
pub(super) fn before_input(mut tools: ResMut<Tools>, player: Res<Player>, mouse: Res<ButtonInput<MouseButton>>,
                           windows: Query<&Window, With<PrimaryWindow>>, panel: Query<&bevy::ui::RelativeCursorPosition, With<ToolPanel>>) {
    let captured = windows.single().is_ok_and(|w| w.cursor_options.grab_mode != CursorGrabMode::None);
    tools.before = Before {
        yaw: player.cam_yaw,
        pitch: player.cam_pitch,
        distance: player.cam_distance,
        crouch: player.crouch_wanted,
        captured,
    };
    let on_panel = panel.single().is_ok_and(|p| p.mouse_over());
    tools.click = mouse.just_pressed(MouseButton::Left) && if tools.edit { !on_panel } else { captured };
}

/// Undoes what `read_input` took that a tool has: in the free camera, the character stands
/// still (no move, aim, fire, jump, item; its camera stays); with a menu open, the click doesn't
/// fire and the wheel doesn't zoom. (In edit mode `read_input` doesn't run: nothing to undo.)
/// The test hooks that act as input (play_ammo.rs `test_trigger`, play_brain.rs `aim_hostile`)
/// run after it, so they drive the character under the free camera too.
pub(super) fn gate_input(tools: Res<Tools>, mut player: ResMut<Player>) {
    let b = tools.before;
    if tools.edit {
        return;
    }
    if tools.freecam.is_some() {
        player.cam_yaw = b.yaw;
        player.cam_pitch = b.pitch;
        player.cam_distance = b.distance;
        player.crouch_wanted = b.crouch;
        player.move_input = Vec2::ZERO;
        player.sprint = false;
        player.walk = false;
        player.aim = false;
        player.fire = false;
        player.jump_pressed = false;
        player.dodge_pressed = false;
        player.switch_pressed = false;
        player.reload_pressed = false;
        player.throw_held = false;
        player.item_use = false;
        player.use_held = false;
        player.next_surface = false;
    } else if tools.panel != Panel::None {
        player.fire = false;
        player.cam_distance = b.distance;
    }
}

/// The crosshair's ray: the free camera's (in edit mode, through the mouse pointer if there is
/// one), else the player's.
pub(super) fn crosshair(player: &Player, tools: &Tools) -> (Vec3, Vec3) {
    match (tools.freecam, tools.pointer.filter(|_| tools.edit)) {
        (Some(c), Some(p)) => (c.pos, c.ray_through(p, tools.aspect)),
        (Some(c), None) => (c.pos, c.ray()),
        (None, _) => aim_ray(player),
    }
}

/// What the tools act at, for their panels: the mouse pointer in edit mode, else the crosshair.
pub(super) fn aim_word(tools: &Tools) -> &'static str {
    if tools.edit && tools.pointer.is_some() { "the pointer" } else { "the crosshair" }
}

/// The ground point under the crosshair (where its ray meets the world, dropped to the floor
/// there), if the ray meets anything within PICK_RANGE.
pub(super) fn ground_point(origin: Vec3, dir: Vec3) -> Option<Vec3> {
    let t = ray_hit(origin, dir, PICK_RANGE)?;
    let p = origin + dir * t;
    Some(Vec3::new(p.x, floor_y(p.x, p.z, p.y + 0.5), p.z))
}

/// Puts the controlled character on the ground at `to`, facing `yaw` with the camera behind
/// (a standing start: no fall, no slide).
fn teleport(player: &mut Player, to: Vec3, yaw: f32) {
    player.position = Vec3::new(to.x, to.y - GROUND, to.z);
    player.prev_xz = Vec2::new(to.x, to.z);
    player.height = 0.0;
    player.vy = 0.0;
    player.air_velocity = Vec3::ZERO;
    player.last_velocity = Vec3::ZERO;
    player.fall_from = to.y;
    player.was_air = false;
    player.slide_vel = Vec3::ZERO;
    player.slide_amount = 0.0;
    if player.action.airborne() {
        player.action = Action::None;
    }
    player.yaw = yaw;
    player.cam_yaw = yaw;
    player.cam_target = player.position + Vec3::new(0.0, 0.35, 0.0);
}

/// F and the free camera's teleports (left click: the crosshair's ground; right click: the
/// camera's), H's hook, and Esc closing a menu (the mouse stays captured). In the map editor's
/// edit mode (play_editor.rs) the camera is the edit camera, always on, and there's no character
/// to teleport: F and the clicks do nothing there, BF_TEST_FREECAM's pose moves the edit camera,
/// and Esc only closes the menu (the mouse is free already).
#[allow(clippy::too_many_arguments)]
fn camera_tool(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut tools: ResMut<Tools>, mut player: ResMut<Player>,
               script: Res<Script>, mut status: ResMut<UsePanel>, cam: Query<&Transform, With<MainCamera>>,
               mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let edit = tools.edit;
    let mut toggle = keys.just_pressed(KeyCode::KeyF) && !edit;
    let mut set: Option<FreeCam> = None;
    let mut port = None;
    let mut close = false;
    if tools.freecam.is_some() && tools.before.captured && !edit {
        if mouse.just_pressed(MouseButton::Left) {
            port = Some(false);
        } else if mouse.just_pressed(MouseButton::Right) {
            port = Some(true);
        }
    }
    for hook in &script.now {
        match hook {
            // (the edit mode's help is the editor's)
            Hook::Help if !edit => player.show_help = !player.show_help,
            Hook::Freecam(Some(pose)) if edit => tools.freecam = Some(*pose),
            Hook::Freecam(pose) if !edit => {
                toggle = tools.freecam.is_none();
                set = *pose;
            }
            Hook::FreecamOff if !edit => toggle = tools.freecam.is_some(),
            Hook::Fly(until, v, fast) => tools.fly = Some((*until, *v, *fast)),
            Hook::Teleport { camera } if !edit => port = Some(*camera),
            Hook::Teleport { .. } => println!("test tools: no teleport in edit mode (no character to put there)"),
            Hook::Close => close = true,
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::Escape) && tools.panel != Panel::None {
        close = true;
        // (read_input let the mouse go: a menu's Esc only closes it. Never grabbed under
        // BF_BACKGROUND, play_background.rs; nor in edit mode, where the mouse is free)
        if let (false, false, Ok(mut w)) = (super::background::on(), edit, windows.single_mut()) {
            w.cursor_options.grab_mode = CursorGrabMode::Locked;
            w.cursor_options.visible = false;
        }
    }
    if close {
        tools.panel = Panel::None;
    }
    if toggle {
        tools.freecam = match tools.freecam {
            Some(_) => None,
            // from where the camera is, looking its way
            None => Some(set.unwrap_or_else(|| FreeCam::from_transform(&cam.single().copied().unwrap_or_default()))),
        };
        status.message = Some((if tools.freecam.is_some() { "Free camera" } else { "Camera back" }.into(), MESSAGE_TIME));
    }
    if let (Some(camera), Some(c)) = (port, tools.freecam) {
        let to = if camera {
            Some(Vec3::new(c.pos.x, floor_y(c.pos.x, c.pos.z, c.pos.y), c.pos.z))
        } else {
            ground_point(c.pos, c.ray())
        };
        match to {
            Some(to) => {
                teleport(&mut player, to, c.yaw);
                tools.freecam = None;
                status.message = Some((format!("Teleported to {:.1}, {:.1}", to.x, to.z), MESSAGE_TIME));
                if std::env::var("BF_TOOLS_LOG").is_ok() {
                    println!("t {:.2}: teleported {} to {to:.2}", player.sim_time, CHARACTERS[player.character]);
                }
            }
            None => status.message = Some(("Nothing under the crosshair".into(), MESSAGE_TIME)),
        }
    }
}

/// The NPC menu's entries (label, character type), built once a map's data is in (`npc_list`).
#[derive(Resource, Default)]
pub(super) struct NpcList(Vec<(String, u32)>);

/// The character types the NPC menu offers (#128): the squad's four by name (each its slot's
/// type), then every other type the loaded archives define whose model is loaded, by display
/// name with where it's defined and its HP (41 militia definitions share 7 names: those tell
/// them apart), and its key when two still read the same (as the object menu's entries).
fn npc_list(game: &Game) -> Vec<(String, u32)> {
    let mut out: Vec<(String, u32)> = (0..SQUAD).map(|c| (title(CHARACTERS[c]), squad_type(c))).collect();
    let mut others: Vec<(String, u32)> = game.character_type_order.iter().copied().filter(|&k| slot_of(k) == OTHER).filter_map(|k| {
        let t = game.character_type(k)?;
        game.has_model(t).then(|| (format!("{} ({}, {} hp)", game.type_label(k), t.place(), t.hitpoints), k))
    }).collect();
    let labels: Vec<String> = others.iter().map(|o| o.0.clone()).collect();
    for (label, k) in others.iter_mut() {
        if labels.iter().filter(|l| l.as_str() == label.as_str()).count() > 1 {
            *label = format!("{label} h_{k:08x}");
        }
    }
    others.sort_by(|a, b| a.0.cmp(&b.0));
    out.extend(others);
    out
}

/// An NPC's name for the menu and the messages: its squad name ("Brutus") or its type's.
fn npc_name(game: &Game, m: &Player) -> String {
    if m.character == OTHER { game.type_label(m.kind) } else { title(CHARACTERS[m.character]) }
}

/// "brutus" -> "Brutus".
fn title(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// The NPC under the crosshair (a `Squad` index): a standing one the ray passes through before
/// the world, else the nearest within NPC_REACH of where the ray meets the ground (a body lying
/// there, or a near miss).
fn npc_at(squad: &Squad, origin: Vec3, dir: Vec3) -> Option<usize> {
    let wall = ray_hit(origin, dir, PICK_RANGE).unwrap_or(PICK_RANGE);
    let npcs = || squad.0.iter().enumerate().filter(|(_, m)| m.npc.is_some());
    npcs().filter(|(_, m)| !m.dead).filter_map(|(i, m)| ray_body(origin, dir, m, wall).map(|t| (i, t)))
        .min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
        .or_else(|| {
            let g = origin + dir * wall;
            npcs().map(|(i, m)| (i, Vec2::new(g.x, g.z).distance(m.body_at.unwrap_or(m.position).xz())))
                .filter(|&(_, d)| d < NPC_REACH).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
        })
}

/// The NPCs a pick means (`Squad` indices).
fn picked(squad: &Squad, pick: Pick, aimed: Option<usize>) -> Vec<usize> {
    match pick {
        Pick::Nth(n) => squad.0.iter().position(|m| m.npc.is_some_and(|c| c.id == n)).into_iter().collect(),
        Pick::Aim => aimed.into_iter().collect(),
        Pick::All => squad.0.iter().enumerate().filter(|(_, m)| m.npc.is_some()).map(|(i, _)| i).collect(),
    }
}

/// "friend", "enemy", "neutral" or "team n".
fn side(team: u8) -> String {
    match team {
        0 => "friend".into(),
        ENEMY_TEAM => "enemy".into(),
        NEUTRAL_TEAM => "neutral".into(),
        n => format!("team {n}"),
    }
}

/// A team as a hook names it: `friend` (the squad's 0), `enemy` (ENEMY_TEAM), `neutral`
/// (NEUTRAL_TEAM) or a number of the game's team table, 0-9 (play_damage.rs's `TEAM_TABLE`).
fn team_of(s: &str) -> Option<u8> {
    match s.trim().to_ascii_lowercase().as_str() {
        "friend" => Some(0),
        "enemy" => Some(ENEMY_TEAM),
        "neutral" => Some(NEUTRAL_TEAM),
        n => n.parse::<u8>().ok().filter(|&t| t < 10),
    }
}

/// The NPC menu (N: its key is the tool registry's, `editor::open_tools`) and the NPC keys (J,
/// B, Delete) and hooks: spawn, switch team or behaviour, remove. Each NPC spawned is listed in
/// the editor's `Placements` (its team and behaviour kept up to date there), and one removed is
/// struck off.
#[allow(clippy::too_many_arguments)]
fn npc_tool(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>,
            scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
            mut hits: ResMut<AiHits>, script: Res<Script>, mut status: ResMut<UsePanel>, game: Res<GameData>, mut npcs: ResMut<NpcList>,
            mut placements: ResMut<super::editor::Placements>) {
    if npcs.0.is_empty() {
        npcs.0 = npc_list(&game.0);
        // BF_TOOLS_LOG: the menu's entries, once a map's list is made
        if std::env::var("BF_TOOLS_LOG").is_ok() {
            println!("NPC menu: {} entries", npcs.0.len());
            for (label, k) in &npcs.0 {
                println!("  {label} (h_{k:08x})");
            }
        }
    }
    let types = &npcs.0;
    let open = tools.panel == Panel::Npcs;
    let mut spawns: Vec<(u32, u8, bool, bool, Option<(Vec2, Option<f32>, Option<f32>)>, Option<bf_viewer::bf::character_type::Placement>)> = vec![];
    let mut acts: Vec<Hook> = vec![];
    if script.now.iter().any(|h| matches!(h, Hook::NpcMenu)) {
        tools.panel = if open { Panel::None } else { Panel::Npcs };
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if open {
        let step = |k: KeyCode| keys.just_pressed(k) as i32;
        let wheel = if scroll.delta.y < 0.0 { 1 } else if scroll.delta.y > 0.0 { -1 } else { 0 };
        let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp) + wheel;
        tools.npc_type = (tools.npc_type as i32 + d).rem_euclid(types.len().max(1) as i32) as usize;
        if keys.just_pressed(KeyCode::KeyJ) {
            tools.npc_team = if tools.npc_team == 0 { ENEMY_TEAM } else { 0 };
        }
        if keys.just_pressed(KeyCode::KeyB) {
            tools.npc_fight = !tools.npc_fight;
        }
        if keys.just_pressed(KeyCode::Enter) || tools.click {
            spawns.push((types[tools.npc_type].1, tools.npc_team, tools.npc_fight, false, None, None));
        }
    } else {
        if keys.just_pressed(KeyCode::KeyJ) {
            acts.push(Hook::Team(Pick::Aim, None));
        }
        if keys.just_pressed(KeyCode::KeyB) {
            acts.push(Hook::Fight(Pick::Aim));
        }
    }
    if keys.just_pressed(KeyCode::Delete) && matches!(tools.panel, Panel::None | Panel::Npcs) {
        acts.push(Hook::Remove(if shift { Pick::All } else { Pick::Aim }));
    }
    for hook in &script.now {
        match hook {
            Hook::Npc { character, team, fight, frozen, at } => match (npc_kind(&game.0, character), npc_placement(&game.0, character)) {
                (Some(kind), _) => spawns.push((kind, *team, *fight, *frozen, *at, None)),
                (None, Some(placed)) => spawns.push((placed.kind, *team, *fight, *frozen, *at, Some(placed))),
                (None, None) => {
                    println!("test tools: no character type {character} in the loaded data");
                    status.message = Some((format!("No character type {character}"), MESSAGE_TIME));
                }
            },
            Hook::Team(..) | Hook::Radar(..) | Hook::Health(..) | Hook::Fight(_) | Hook::Remove(_) | Hook::Motion(..) => acts.push(hook.clone()),
            _ => {}
        }
    }
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let (origin, dir) = crosshair(&player, &tools);
    for (kind, team, fight, frozen, at, placed) in spawns {
        let ground = match at {
            Some((p, from, _)) => Some(Vec3::new(p.x, floor_y(p.x, p.y, from.unwrap_or(player.position.y + GROUND + 50.0)), p.y)),
            None => ground_point(origin, dir),
        };
        let Some(ground) = ground else {
            status.message = Some(("Nothing under the crosshair".into(), MESSAGE_TIME));
            continue;
        };
        tools.spawned += 1;
        let id = tools.spawned;
        // a squad type is its slot's character; any other is OTHER with its own type
        let mut m = Player::new(slot_of(kind));
        m.kind = kind;
        m.position = Vec3::new(ground.x, ground.y - GROUND, ground.z);
        m.prev_xz = Vec2::new(ground.x, ground.z);
        m.fall_from = ground.y;
        // facing the hook's bearing, else the controlled character
        let to = player.position - m.position;
        m.yaw = at.and_then(|a| a.2).unwrap_or((-to.x).atan2(-to.z));
        m.cam_yaw = m.yaw;
        m.face_yaw = Some(m.yaw);
        m.team = team;
        m.npc = Some(Npc { id, fight, face: m.yaw });
        // (its own random stream: hit reactions, bursts)
        m.rng ^= id.wrapping_mul(0x9E37_79B9) | 1;
        // its brain off (no hit reactions: play_enemies.rs)
        m.enemy.frozen = frozen;
        // a placement's: its weapons and personality (play_enemies.rs; its type's otherwise), and
        // its sight (its `<ai>`'s vision and field-of-view, play_brain.rs `Sensor`)
        if let Some(pl) = &placed {
            m.enemy.weapons = Some(pl.weapons.iter().map(|w| w.0).collect());
            m.enemy.personality = pl.ai.personalities[0];
            m.enemy.ai = Some(pl.ai.clone());
        }
        // (its sensor from the spawn on: the model's load sets it again, play.rs `spawn_unit`)
        m.sensor.set(game.0.character_type(kind), m.enemy.ai.as_ref());
        let label = format!("{} {} ({})", side(team), npc_name(&game.0, &m), if fight { "fight" } else if frozen { "frozen" } else { "dummy" });
        if log {
            let from = placed.as_ref().map_or(String::new(), |pl| format!(", placement h_{:08x} [{}] weapons {:?} personality h_{:08x}", pl.name, pl.archive,
                pl.weapons.iter().map(|(w, _)| game.0.weapons.get(w).map_or(format!("h_{w:08x}"), |d| d.label.clone())).collect::<Vec<_>>(), pl.ai.personalities[0]));
            println!("t {:.2}: NPC {id}: {label} at {ground:.2}, type h_{kind:08x}{from}", player.sim_time);
        }
        status.message = Some((format!("Spawned {label}"), MESSAGE_TIME));
        let thing = super::editor::Thing::Npc { id, kind, team, fight, frozen, placement: placed.as_ref().map(|pl| pl.name) };
        placements.add(thing, ground, m.yaw, false, None);
        squad.0.push(m);
    }
    let aimed = npc_at(&squad, origin, dir);
    for act in acts {
        match act {
            Hook::Motion(p, mt, goal, ready) => {
                for i in picked(&squad, p, aimed) {
                    let m = &mut squad.0[i];
                    super::enemies::drive(m, mt, goal, ready);
                    if log {
                        println!("t {:.2}: NPC {}: motion {mt} to {goal:?}{}", player.sim_time, m.npc.map_or(0, |n| n.id), if ready { "" } else { " (relaxed)" });
                    }
                }
            }
            Hook::Team(p, _) | Hook::Radar(p, _) | Hook::Health(p, _) | Hook::Fight(p) => {
                let list = picked(&squad, p, aimed);
                if list.is_empty() {
                    status.message = Some(("No NPC under the crosshair".into(), MESSAGE_TIME));
                }
                for i in list {
                    let m = &mut squad.0[i];
                    let Some(mut npc) = m.npc else { continue };
                    match act {
                        Hook::Team(_, Some(team)) => m.team = team,
                        Hook::Team(_, None) => m.team = if m.team == 0 { ENEMY_TEAM } else { 0 },
                        Hook::Radar(_, value) => m.radar_contact = value,
                        Hook::Health(_, hp) => (m.health, m.max_health) = (hp, hp),
                        _ => npc.fight = !npc.fight,
                    }
                    m.npc = Some(npc);
                    // (its listing keeps what it's set to: the edited map's)
                    placements.set_npc(npc.id, m.team, npc.fight);
                    // (its radar-contact when a hook has changed it from every character's 1)
                    let radar = if m.radar_contact != 1 { format!(", radar-contact {}", m.radar_contact) } else { String::new() };
                    let hp = if matches!(act, Hook::Health(..)) { format!(", {:.0} HP", m.health) } else { String::new() };
                    let label = format!("NPC {} ({}): {} {}{radar}{hp}", npc.id, npc_name(&game.0, m), side(m.team), if npc.fight { "fight" } else { "dummy" });
                    if log {
                        println!("t {:.2}: {label}", player.sim_time);
                    }
                    status.message = Some((label, MESSAGE_TIME));
                }
            }
            Hook::Remove(p) => {
                let list = picked(&squad, p, aimed);
                if list.is_empty() {
                    status.message = Some(("No NPC under the crosshair".into(), MESSAGE_TIME));
                    continue;
                }
                let mut k = 0;
                squad.0.retain_mut(|m| {
                    let gone = list.contains(&k);
                    k += 1;
                    if gone {
                        if let Some(l) = m.loaded.take() {
                            commands.entity(l.root).despawn();
                        }
                        if let Some(e) = m.slide_fx.take() {
                            commands.entity(e).despawn();
                        }
                        // (taken off the map, not lost in play: struck off its listing)
                        if let Some(n) = m.npc {
                            placements.remove_npc(n.id);
                        }
                    }
                    !gone
                });
                // (the shots on their way are counted by place in the squad)
                hits.0.clear();
                player.pending_hits.clear();
                if log {
                    println!("t {:.2}: removed {} NPCs, {} left", player.sim_time, list.len(), squad.0.iter().filter(|m| m.npc.is_some()).count());
                }
                status.message = Some((format!("Removed {} NPC{}", list.len(), if list.len() == 1 { "" } else { "s" }), MESSAGE_TIME));
            }
            _ => {}
        }
    }
}

/// The test map's NPC "AI" (see the module's notes), in place of the squad AI: it stands where
/// it was put; a dummy faces the way it was put and holds fire; one set to fight aims and fires
/// at `target` (a hostile in sight, from play_brain.rs `glance`: its type's sight distance and
/// field of view, or its placement's) with the squad AI's `ai_fire`; with nothing to fire at,
/// one set to fight turns to `toward` (a known enemy's last place: one that shot it from behind).
pub(super) fn npc_ai(m: &mut Player, target: Option<Vec3>, toward: Option<Vec3>, dt: f32) {
    let Some(npc) = m.npc else { return };
    m.move_input = Vec2::ZERO;
    m.sprint = false;
    m.walk = false;
    m.jump_pressed = false;
    m.dodge_pressed = false;
    m.dodge_request = None;
    m.switch_pressed = false;
    m.reload_pressed = false;
    m.throw_held = false;
    m.item_use = false;
    m.use_held = false;
    m.next_surface = false;
    m.crouch_wanted = false;
    m.cam_yaw = npc.face;
    m.face_yaw = Some(npc.face);
    let target = target.filter(|_| npc.fight);
    // (one set to fight, with nothing to fire at, turns to a known enemy's last place, gun up:
    // `toward`, play_brain.rs `glance`)
    let turned = npc.fight && target.is_none() && toward.is_some_and(|p| super::face_toward(m, p));
    m.aim = target.is_some() || turned;
    ai_fire(m, target, dt);
}

/// The object menu's entries (see `Catalogue`).
fn catalogue(game: &Game, kits: &grenade::GrenadeKits) -> Vec<Entry> {
    // how far below its origin a model reaches, turned `rest`
    let lift = |model: &WeaponModel, rest: Quat| -model.parts.iter()
        .flat_map(|p| p.geosets.iter().flat_map(move |g| g.positions.iter().map(move |v| (rest * (p.offset + p.rotation * Vec3::from(*v))).y)))
        .fold(f32::MAX, f32::min).min(1e3);
    let mut out = vec![];
    for (kind, label, model) in super::testmap::pickup_types(game) {
        out.push(Entry { label, group: "pickups", kind: Kind::Pickup(kind), lift: lift(&model, Quat::IDENTITY), rest: Quat::IDENTITY, model });
    }
    // a weapon lies on its side (the demo's choice: modelled as held, upright, it would stand
    // on its grip)
    let side = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    for (key, def, model) in super::testmap::hand_weapons(game, false) {
        out.push(Entry { label: def.label, group: "weapons", kind: Kind::Weapon(key), lift: lift(&model, side), rest: side, model });
    }
    for (i, k) in kits.0.iter().enumerate() {
        let Ok(model) = WeaponModel::load(game, k.def.archetype) else { continue };
        out.push(Entry { label: k.def.label.clone(), group: "grenades", kind: Kind::Grenade(i), lift: lift(&model, Quat::IDENTITY), rest: Quat::IDENTITY, model });
    }
    for (label, kind) in BREAKABLES {
        let model = game.breakable(kind).and(game.object_meshes.get(&kind)).and_then(|&a| WeaponModel::load(game, a).ok());
        match model {
            Some(model) => out.push(Entry { label: label.into(), group: "scenery", kind: Kind::Breakable(kind), lift: lift(&model, Quat::IDENTITY),
                                            rest: Quat::IDENTITY, model }),
            None => println!("test tools: no {label} (h_{kind:08x}) in the loaded data"),
        }
    }
    // (two of a name - the placed and the carried Medkit, two Bio Ammo - by their hash too)
    let labels: Vec<String> = out.iter().map(|e| e.label.clone()).collect();
    for e in out.iter_mut() {
        if labels.iter().filter(|l| **l == e.label).count() > 1 {
            let key = match e.kind { Kind::Pickup(k) | Kind::Weapon(k) | Kind::Breakable(k) => k, Kind::Grenade(k) => k as u32 };
            e.label = format!("{} h_{key:08x}", e.label);
        }
        if !e.lift.is_finite() || e.lift.abs() > 5.0 {
            e.lift = 0.0;
        }
    }
    out
}

/// An entry by name: its label in any case, else the first whose label contains it.
fn find_entry(list: &[Entry], name: &str) -> Option<usize> {
    let n = name.to_ascii_lowercase();
    list.iter().position(|e| e.label.to_ascii_lowercase() == n)
        .or_else(|| list.iter().position(|e| e.label.to_ascii_lowercase().contains(&n)))
}

/// Spawns entry `e`'s model under `root` (its parts as modelled); `ghost`: in see-through
/// copies of its materials, casting no shadow.
fn spawn_parts(commands: &mut Commands, game: &mut Game, assets: &mut ModelAssets, e: &Entry, root: Entity, ghost: bool) {
    for part in &e.model.parts {
        let pe = commands.spawn((Transform::from_translation(part.offset).with_rotation(part.rotation), Visibility::Inherited, ChildOf(root))).id();
        if !ghost {
            spawn_static(commands, game, &part.geosets, assets, pe, true);
            continue;
        }
        for (mesh, mat) in bf_viewer::scene::static_meshes(game, &part.geosets, assets, true) {
            let see_through = assets.materials.get(&mat).cloned().map(|mut m| {
                // an opaque surface's texture alpha is no coverage (a shine or glow mask: the
                // medkit's is about 0, so its copy didn't show at all): its colours only
                if m.alpha_mode == AlphaMode::Opaque {
                    let solid = m.base_color_texture.as_ref().and_then(|t| assets.images.get(t)).cloned().and_then(|mut img| {
                        use bevy::render::render_resource::TextureFormat;
                        let four = matches!(img.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm);
                        img.data.as_mut().filter(|_| four)?.chunks_exact_mut(4).for_each(|p| p[3] = 255);
                        Some(assets.images.add(img))
                    });
                    if solid.is_some() {
                        m.base_color_texture = solid;
                    }
                }
                m.base_color = m.base_color.with_alpha(GHOST_ALPHA);
                m.alpha_mode = AlphaMode::Blend;
                m.emissive = GHOST_GLOW;
                assets.materials.add(m)
            }).unwrap_or(mat);
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(see_through), Transform::default(), NotShadowCaster, ChildOf(pe)));
        }
    }
}

/// Where entry `e` stands put on the ground at `at` turned `yaw`.
fn placed_at(e: &Entry, at: Vec3, yaw: f32) -> Transform {
    Transform::from_translation(at + Vec3::Y * e.lift).with_rotation(Quat::from_rotation_y(yaw) * e.rest)
}

/// Where an object goes: on the ground at a point, or (the free camera) in the air with its
/// lowest point at `at`, the ground under it at `below`.
#[derive(Clone, Copy, Debug)]
pub(super) enum Spot {
    Ground(Vec3),
    Air { at: Vec3, below: Vec3 },
}

impl Spot {
    /// Where the object's lowest point goes.
    pub(super) fn at(self) -> Vec3 {
        match self {
            Spot::Ground(p) | Spot::Air { at: p, .. } => p,
        }
    }
}

/// Whether entry kind `k` drops when put in the air (the loose ones: pickups, weapons and
/// grenades fall under play_pickups.rs's loose-body physics); breakable scenery hangs where it's
/// put, as a level's scenery never moves (the ticket's choice, #116).
fn drops(k: Kind) -> bool {
    !matches!(k, Kind::Breakable(_))
}

/// Where the crosshair puts an object. Following the character: on the ground under it. In the
/// free camera: in the air, `air` m along its ray (short of what the ray meets first, by
/// AIR_CLEAR), unless that's on (or under) the ground there.
pub(super) fn spot_at(origin: Vec3, dir: Vec3, freecam: bool, air: f32) -> Option<Spot> {
    if !freecam {
        return ground_point(origin, dir).map(Spot::Ground);
    }
    let wall = ray_hit(origin, dir, PICK_RANGE).unwrap_or(PICK_RANGE);
    let t = air.min(wall - AIR_CLEAR).max(0.0);
    let at = origin + dir * t;
    let below = Vec3::new(at.x, floor_y(at.x, at.z, at.y + AIR_MIN_HEIGHT), at.z);
    Some(if at.y - below.y < AIR_MIN_HEIGHT { Spot::Ground(below) } else { Spot::Air { at, below } })
}

/// Puts entry `i` at `spot` turned `yaw` (see the module's notes for each kind), and lists it in
/// the editor's `Placements` (its root marked with its number). In the air, a loose one drops
/// from there (`DropIn`, or `Thrown` from rest: play_pickups.rs's physics) and scenery hangs.
#[allow(clippy::too_many_arguments)]
fn place(commands: &mut Commands, game: &mut Game, assets: &mut ModelAssets, placements: &mut super::editor::Placements, list: &[Entry], i: usize,
         spot: Spot, yaw: f32) {
    let e = &list[i];
    let root = commands.spawn((placed_at(e, spot.at(), yaw), Visibility::default(), Name::new(format!("placed {}", e.label)))).id();
    let id = placements.add(super::editor::Thing::Object { label: e.label.clone(), kind: e.kind }, spot.at(), yaw, matches!(spot, Spot::Air { .. }),
                            Some(root));
    commands.entity(root).insert(super::editor::PlacedId(id));
    let air = matches!(spot, Spot::Air { .. });
    let fall = || super::pickups::Thrown { velocity: Vec3::ZERO, spin: Vec3::ZERO };
    match e.kind {
        Kind::Pickup(kind) => {
            commands.entity(root).insert((Placed { tag: h("inventory-object"), kind, name: 0 }, super::pickups::LatePickup));
            if air {
                commands.entity(root).insert(super::pickups::DropIn);
            }
        }
        Kind::Weapon(key) => {
            commands.entity(root).insert((super::testmap::RackWeapon(key), super::testmap::Lying));
            if air {
                commands.entity(root).insert(fall());
            }
        }
        Kind::Grenade(k) => {
            commands.entity(root).insert(GroundGrenade(k));
            if air {
                commands.entity(root).insert(fall());
            }
        }
        Kind::Breakable(kind) => {
            commands.entity(root).insert(super::scenery::LateBreakable(kind));
        }
    }
    spawn_parts(commands, game, assets, e, root, false);
}

/// The object menu (O: its key is the tool registry's, `editor::open_tools`): picking, the
/// see-through copy at the crosshair, the wheel's turn, a click placing; and the placing hooks.
#[allow(clippy::too_many_arguments)]
fn object_tool(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>,
               scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>, player: Res<Player>, mut game: ResMut<GameData>,
               kits: Option<Res<grenade::GrenadeKits>>, mut list: ResMut<Catalogue>, script: Res<Script>, mut status: ResMut<UsePanel>,
               (mut ghosts, mut gizmos): (Query<(&mut Transform, &mut Visibility), With<Ghost>>, Gizmos),
               mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
               mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>, mut placements: ResMut<super::editor::Placements>) {
    if list.0.is_empty() {
        let Some(kits) = kits else { return };
        list.0 = catalogue(&game.0, &kits);
        println!("test tools: {} objects to place", list.0.len());
    }
    let n = list.0.len();
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let mut click = false;
    let mut direct: Vec<(usize, Spot, f32)> = vec![];
    for hook in &script.now {
        match hook {
            Hook::Air(m) => tools.air = m.clamp(AIR_RANGE.0, AIR_RANGE.1),
            Hook::Placer { object, yaw } => match find_entry(&list.0, object) {
                Some(i) => {
                    tools.panel = Panel::Objects;
                    tools.object = i;
                    tools.yaw = *yaw;
                }
                None => println!("test tools: no object {object}"),
            },
            Hook::Place { object: None, .. } => click = true,
            Hook::Place { object: Some(o), at } => match (find_entry(&list.0, o), at) {
                (Some(i), Some((p, yaw, up))) => {
                    let below = Vec3::new(p.x, floor_y(p.x, p.y, player.position.y + GROUND + 50.0), p.y);
                    let spot = if *up > AIR_MIN_HEIGHT { Spot::Air { at: below + Vec3::Y * *up, below } } else { Spot::Ground(below) };
                    direct.push((i, spot, *yaw));
                }
                (Some(i), None) => {
                    tools.object = i;
                    click = true;
                }
                (None, _) => println!("test tools: no object {o}"),
            },
            _ => {}
        }
    }
    let open = tools.panel == Panel::Objects;
    if open && n > 0 {
        let step = |k: KeyCode| keys.just_pressed(k) as i32;
        let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp);
        tools.object = (tools.object as i32 + d).rem_euclid(n as i32) as usize;
        // Page Up / Down: the first entry of the previous / next group
        let group = list.0[tools.object].group;
        if keys.just_pressed(KeyCode::PageDown) {
            tools.object = (tools.object..n).find(|&i| list.0[i].group != group).unwrap_or(0);
        }
        if keys.just_pressed(KeyCode::PageUp) {
            let start = (0..=tools.object).rev().find(|&i| list.0[i].group != group).map_or(0, |i| i + 1);
            let prev = if start == 0 { n - 1 } else { start - 1 };
            let g = list.0[prev].group;
            tools.object = (0..=prev).rev().find(|&i| list.0[i].group != g).map_or(0, |i| i + 1);
        }
        // the wheel turns it; with Shift, in the free camera, it moves it along the crosshair
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let wheel = if scroll.delta.y != 0.0 { scroll.delta.y } else { scroll.delta.x };
        if wheel != 0.0 && shift && tools.freecam.is_some() {
            tools.air = (tools.air + AIR_STEP * wheel.signum()).clamp(AIR_RANGE.0, AIR_RANGE.1);
        } else if scroll.delta.y != 0.0 {
            tools.yaw = wrap_angle(tools.yaw + TURN_STEP * scroll.delta.y.signum());
        }
        click |= keys.just_pressed(KeyCode::Enter) || tools.click;
    }
    let (origin, dir) = crosshair(&player, &tools);
    let spot = spot_at(origin, dir, tools.freecam.is_some(), tools.air);
    if click && n > 0 {
        match spot {
            Some(at) => direct.push((tools.object, at, tools.yaw)),
            None => status.message = Some(("Nothing under the crosshair".into(), MESSAGE_TIME)),
        }
    }
    for (i, at, yaw) in direct {
        place(&mut commands, &mut game.0, &mut assets, &mut placements, &list.0, i, at, yaw);
        status.message = Some((format!("Placed {}", list.0[i].label), MESSAGE_TIME));
        if log {
            let how = match at {
                Spot::Ground(_) => "on the ground".to_string(),
                Spot::Air { at, below } => format!("{:.2} m up ({})", at.y - below.y, if drops(list.0[i].kind) { "drops" } else { "hangs" }),
            };
            println!("t {:.2}: placed {} ({:?}) at {:.2} {how} turned {:.0} deg", player.sim_time, list.0[i].label, list.0[i].kind, at.at(), yaw.to_degrees());
        }
    }
    // the see-through copy: the picked entry, at the crosshair
    let want = (open && n > 0).then_some(tools.object);
    if tools.ghost.map(|g| g.1) != want {
        if let Some((e, _)) = tools.ghost.take() {
            commands.entity(e).despawn();
        }
        if let Some(i) = want {
            let root = commands.spawn((Transform::default(), Visibility::Hidden, Ghost, Name::new("ghost"))).id();
            spawn_parts(&mut commands, &mut game.0, &mut assets, &list.0[i], root, true);
            tools.ghost = Some((root, i));
        }
    }
    if let Some((e, i)) = tools.ghost {
        if let Ok((mut t, mut v)) = ghosts.get_mut(e) {
            match spot {
                Some(at) => {
                    *t = placed_at(&list.0[i], at.at(), tools.yaw);
                    *v = Visibility::Inherited;
                }
                None => *v = Visibility::Hidden,
            }
        }
        // in the air: where it will land (a line down, a ring), or a ring at its foot where it
        // will hang
        let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        if let Some(Spot::Air { at, below }) = spot {
            if drops(list.0[i].kind) {
                gizmos.line(at, below, DROP_MARK);
                gizmos.circle(Isometry3d::new(below + Vec3::Y * 0.02, flat), MARK_RING, DROP_MARK);
            } else {
                gizmos.circle(Isometry3d::new(at, flat), MARK_RING, HANG_MARK);
            }
        }
    }
}

/// Walking over a grenade put on the ground takes it: one more of its type, up to the type's
/// stack-limit ("Took <type>"; full, it stays and "<type>: full" shows once).
fn take_grenades(mut commands: Commands, mut player: ResMut<Player>, kits: Option<Res<grenade::GrenadeKits>>, mut status: ResMut<UsePanel>,
                 lying: Query<(Entity, &GroundGrenade, &Transform)>, mut told: Local<Option<Entity>>) {
    let Some(kits) = kits else { return };
    if player.dead {
        return;
    }
    let feet = player.position + Vec3::Y * GROUND;
    let near = lying.iter().find(|(_, _, t)| t.translation.xz().distance(feet.xz()) < GRENADE_REACH && (t.translation.y - feet.y).abs() < GRENADE_REACH_UP);
    let Some((e, g, _)) = near else {
        *told = None;
        return;
    };
    let Some(kit) = kits.0.get(g.0) else { return };
    let limit = kit.def.stack_limit.max(1);
    let label = kit.def.label.clone();
    match player.grenades.get_mut(g.0) {
        Some(n) if *n < limit => {
            *n += 1;
            commands.entity(e).despawn();
            status.message = Some((format!("Took {label}"), MESSAGE_TIME));
        }
        _ if *told != Some(e) => {
            *told = Some(e);
            status.message = Some((format!("{label}: full"), MESSAGE_TIME));
        }
        _ => {}
    }
}

/// The free camera: the mouse turns it (once captured; in edit mode while the right button is
/// held, play_editor.rs), WASD fly it, Space / E up and Ctrl / Q down (E / Q as the level
/// viewer's, bf_level), Shift faster (BF_TEST_FLY), at `Tools::speed`; and it's the view (after
/// the follow and death cameras have put theirs).
#[allow(clippy::too_many_arguments)]
fn fly(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, motion: Res<AccumulatedMouseMotion>, mut tools: ResMut<Tools>, player: Res<Player>,
       windows: Query<&Window, With<PrimaryWindow>>, mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>) {
    let Some(mut c) = tools.freecam else { return };
    let dt = frame_dt(&time);
    if windows.single().is_ok_and(|w| w.cursor_options.grab_mode != CursorGrabMode::None) {
        c.yaw = wrap_angle(c.yaw - motion.delta.x * LOOK);
        c.pitch = (c.pitch - motion.delta.y * LOOK).clamp(-FLY_PITCH, FLY_PITCH);
    }
    let key = |k: KeyCode| keys.pressed(k) as i32 as f32;
    let up = key(KeyCode::Space).max(key(KeyCode::KeyE)) - key(KeyCode::ControlLeft).max(key(KeyCode::KeyQ));
    let mut v = Vec3::new(key(KeyCode::KeyD) - key(KeyCode::KeyA), up, key(KeyCode::KeyW) - key(KeyCode::KeyS));
    let mut fast = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if let Some((until, hv, hf)) = tools.fly {
        if player.sim_time < until {
            v = hv;
            fast = hf;
        } else {
            tools.fly = None;
        }
    }
    let r = c.rotation();
    let speed = tools.speed * if fast { FLY_FAST } else { 1.0 };
    c.pos += (r * Vec3::X * v.x + Vec3::Y * v.y + r * Vec3::NEG_Z * v.z) * speed * dt;
    tools.freecam = Some(c);
    if let Ok((mut t, mut p)) = cam.single_mut() {
        *t = Transform::from_translation(c.pos).with_rotation(r);
        if let Projection::Perspective(pp) = p.as_mut() {
            if (pp.fov - FOV_Y).abs() > 1e-4 {
                pp.fov = FOV_Y;
            }
        }
    }
}

/// The tools' panel, right of the middle (hidden with nothing open). It knows where the mouse
/// is over it, so a click on it isn't one in the world (edit mode, `before_input`).
fn spawn_panel(mut commands: Commands) {
    commands.spawn((
        ToolPanel,
        bevy::ui::RelativeCursorPosition::default(),
        Text::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.92)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        Node { position_type: PositionType::Absolute, right: Val::Px(12.0), top: Val::Percent(18.0), padding: UiRect::all(Val::Px(8.0)),
               max_width: Val::Percent(36.0), display: Display::None, ..default() },
    ));
}

/// What the panel shows: the free camera's keys, or the open menu (the sky, music and level
/// menus' text from play_testworld.rs, the scenery menu's from play_testscenery.rs); in edit
/// mode with no menu open, the editor's help (play_editor.rs).
#[allow(clippy::too_many_arguments)]
pub(super) fn show_panel(tools: Res<Tools>, list: Res<Catalogue>, npcs: Res<NpcList>, squad: Res<Squad>, world: Res<super::testworld::WorldText>,
              scenery: Res<super::testscenery::SceneryText>, help: Res<super::editor::EditText>,
              mut panel: Query<(&mut Text, &mut Node), With<ToolPanel>>) {
    let Ok((mut text, mut node)) = panel.single_mut() else { return };
    let mut s = String::new();
    if tools.edit && tools.panel == Panel::None {
        s += &help.0;
    } else if tools.freecam.is_some() && !tools.edit {
        s += "FREE CAMERA   (F: back)\nmouse: look   WASD: fly\nSpace / Ctrl (E / Q): up / down   Shift: faster\nleft click: teleport to the crosshair\nright click: teleport under the camera\n";
    }
    match tools.panel {
        Panel::Npcs => {
            if !s.is_empty() {
                s += "\n";
            }
            s += "SPAWN NPC   (N / Esc: close)\n";
            // (a window of the list round the one picked, as the object menu's)
            let n = npcs.0.len();
            let first = tools.npc_type.saturating_sub(LIST_LINES / 2).min(n.saturating_sub(LIST_LINES));
            for i in first..(first + LIST_LINES).min(n) {
                s += &format!("{} {}\n", if i == tools.npc_type { ">" } else { " " }, npcs.0[i].0);
            }
            s += &format!("  ({} of {n} character types)\n", tools.npc_type + 1);
            let at = aim_word(&tools);
            s += &format!("J team: {}   B: {}\nclick / Enter: spawn at {at}\n", side(tools.npc_team).to_uppercase(),
                          if tools.npc_fight { "FIGHT" } else { "DUMMY" });
            let n = squad.0.iter().filter(|m| m.npc.is_some()).count();
            s += &format!("{n} spawned   J / B / Del on the one under {at}, Shift+Del all");
        }
        Panel::Objects => {
            if !s.is_empty() {
                s += "\n";
            }
            s += "PLACE OBJECT   (O / Esc: close)\n";
            let n = list.0.len();
            let first = tools.object.saturating_sub(LIST_LINES / 2).min(n.saturating_sub(LIST_LINES));
            let mut group = "";
            for i in first..(first + LIST_LINES).min(n) {
                let e = &list.0[i];
                if e.group != group {
                    group = e.group;
                    s += &format!("  [{group}]\n");
                }
                s += &format!("{} {}\n", if i == tools.object { ">" } else { " " }, e.label);
            }
            s += &format!("Up / Down pick   Page Up / Down group   wheel turn ({:.0} deg)\n", tools.yaw.to_degrees());
            let at = aim_word(&tools);
            s += &if tools.freecam.is_some() {
                format!("click / Enter: place IN THE AIR, {:.1} m along {at}\n(Shift + wheel); loose ones drop, scenery hangs", tools.air)
            } else {
                format!("click / Enter: place on the ground at {at}")
            };
        }
        Panel::Sky | Panel::Music | Panel::Levels => {
            if !s.is_empty() {
                s += "\n";
            }
            s += &world.0;
        }
        Panel::Scenery => {
            if !s.is_empty() {
                s += "\n";
            }
            s += &scenery.0;
        }
        Panel::None => {}
    }
    let display = if s.is_empty() { Display::None } else { Display::Flex };
    if node.display != display {
        node.display = display;
    }
    let s = s.trim_end().to_string();
    if text.0 != s {
        text.0 = s;
    }
}
