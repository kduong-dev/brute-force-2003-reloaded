//! A level file (levels-<name>.xmb, loaded with `Game::load_level`): the terrain mesh, the
//! placed objects, the sky, fog and lighting colours, and the level's own cameras.
//!
//!  Terrain     mesh-name: a mesh of terrain vertices (see `character::TERRAIN_VERTEX`), one
//!              geoset per texture layer; the first is the ground, the others are blended over
//!              it by their material's "alpha" mask, one 128 px mask across the 8 x 8 blocks of
//!              32 m (x and z -128..128; mask v runs toward -z).
//!  objects     every element with an object type (h_ff8051ee) and a <transform> (3 x 3,
//!              then the position): game objects, weapon and inventory pickups, start points...
//!              The type's objecttypes entry names its mesh archetype; light-usage says whether
//!              it stands indoors (`Placement::light_usage`).
//!  sky         <object mesh-name> at <position>, <background-color>
//!  fog         h_1a467f02: <color> (0-255), start-distance, end-distance
//!  cameras     camera-object transforms (the level's flyby cameras); by name with their FOV,
//!              the cutscenes' cameras (`LevelCamera`), and the path-objects their scripts move
//!              them along (`LevelPath`, #222)
//!  lights      light-object, by its type's `light-type` (objecttypes `<light>`,
//!              `Game::light_types`): directional (3) or point (1); colour h_f08eb2f3 (0-1), a
//!              directional one shining down the transform's -z. h_e02aba1c puts a light in the
//!              indoor pair, h_1f416687 in the outdoor pair (`LevelLight`); an object takes the
//!              pair its light-usage says (`Placement::light_usage`, `Geoset::indoor`), its lamps
//!              too (`LevelLamp`). Ambient: Terrain <ambient>, <object-ambient>,
//!              <character-ambient>.
//!  collision   the terrain blocks' collision surfaces (Terrain > blocks h_051ac63e) and the
//!              invisible blockers (blocker h_051ac63e + transform), see bf::collision
//!  buttons     world-button-object (h_16f22d4b, reticule-action 1): a wall panel that sends
//!              its signal (h_eaf8a35b, 14 on every one) when used
//!  triggers    every element built on the game's trigger base (routers, timers, calculators,
//!              spawn-triggers, detectors, anim-triggers ...), read by `bf::triggers` and run by
//!              bf_play's trigger runtime (play_triggers.rs). A gate's panels reach its
//!              anim-triggers through a router (see `Level::button_opens`).
//!  radar       each placement's radar-contact (h_1905f381, else its type's), and the
//!              radar-triggers (radar-contact N on their h_0b6b92ba objects); those that act at
//!              the level's start (signal 13) are applied as it loads (`RadarTrigger`)
//!  patrols     the patrol points (`<patrol-point>`, and `<patrol-first-point>` the head of a
//!              chain), linked forward and back, grouped by their iD (`PatrolPoint`): the AI's
//!              patrol routes (bf_play's play_brain.rs, #227)

use bevy::math::{Mat4, Vec3};

use super::bxml::Element;
use super::character::{Game, Geoset, H_PHYSICS_NAME, H_RADAR_CONTACT, LT_DIRECTIONAL, LT_POINT, LT_SPOT};
use super::hash::h;
use super::triggers;
use super::weapon;

/// Battle of Bulgar's terrain half-extent (8 blocks of 16 cells of 2 m); see `Level::terrain_half`.
pub const TERRAIN_HALF: f32 = 128.0;
/// The terrain grid the vertex decoder assumes (`character::TERRAIN_*`: 65 vertices a row, 2 m):
/// the level's own grid is undone from it, see `Level::load`.
const DECODED_ROW: f32 = 65.0;
const DECODED_CELL: f32 = 2.0;
const H_TYPE: u32 = 0xFF80_51EE;
const H_FOG: u32 = 0x1A46_7F02;
/// A terrain block's baked light: 16 x 16 cell values.
const H_BAKED_LIGHT: u32 = 0xE2D7_9BAD;
/// Terrain cell size in metres (1, 2 or 4).
const H_TERRAIN_CELL: u32 = 0xE023_CDC9;
/// A light-object's colour (h_f08eb2f3, 0-1: the light's +0xf0..+0xf8, by its placement's
/// parser 0x14e700-0x14e8b0).
const H_LIGHT_COLOR: u32 = 0xF08E_B2F3;
/// A light-object's h_e02aba1c: the light's +0x94 bit 1 (0x14e7b4), what puts it in the indoor
/// pair (`LevelLight::indoor`).
const H_LIGHT_INDOOR: u32 = 0xE02A_BA1C;
/// A light-object's h_1f416687: +0x94 bit 2 (0x14e7f0), the outdoor pair (`LevelLight::outdoor`).
const H_LIGHT_OUTDOOR: u32 = 0x1F41_6687;

pub struct Placement {
    /// the element's name hash (game-object, start-point, ...)
    pub tag: u32,
    /// the object's own name (what triggers refer to it by)
    pub name: u32,
    /// object type name
    pub kind: u32,
    /// its mesh archetype, if it has one
    pub archetype: Option<u32>,
    pub transform: Mat4,
    /// its radar-contact (`character::H_RADAR_CONTACT`, the object's +0x7a) as the level starts:
    /// its own attribute (the placement's handler 0x15646f -> FUN_0015b310), else its type's
    /// default (`ObjectType::radar_contact`), then the radar-triggers that act at the level's
    /// start (`Level::radar_triggers`). e34's five missile racks and three h_e8589e7e devices 4,
    /// its device h_00bc61ab 2 (as read live in #141); 0 on every squad deathmatch map
    pub radar_contact: u8,
    /// its light-usage (the object's +0x44; the LU_ names at 0x3bf954): LU_INDOOR, LU_OUTDOOR,
    /// LU_GEOSET (each triangle says, `collision::MATERIAL_INDOOR`), 0 (LU_NONE) without one.
    /// Every game-object has one (m09_c: 52 indoor, 7 outdoor, 124 by geoset). It picks the light
    /// pair the object is drawn with (0x9f3c0) and whether a character standing on it is indoors
    /// (FUN_0012dab0), which the indoor and outdoor sound-triggers follow
    pub light_usage: u8,
    /// it has an orientation of its own (a <transform>, or an <ORIENTATION> with its <position>):
    /// a direction-point does, a world-point (a <position> alone) doesn't. A warp-trigger puts
    /// what it moves at such a point's whole transform, else at its position only (FUN_00200480,
    /// the point's +0xc interface asked for one)
    pub oriented: bool,
    /// a point's h_e6564e8a (H_POINT_RADIUS, a world point's +0x20 by its attribute handler
    /// FUN_000ecdc0): the arrival radius a route's walk takes at it when it's over 0 (FUN_00026d80);
    /// -1 without one (the tutorial's world points: -1 765 times, 1 165 times, a few 0.3-0.6)
    pub radius: f32,
    /// an item object's h_1cb7d5f2 (name not recovered): set down and live, as if placed by a
    /// character (the item object's attribute handler 0x14a0a0 sets +0x1bc bit 1 and calls its
    /// item set's +0x10: for a Sentry the mine is armed, play_sentry.rs). True on the tutorial's
    /// three mines (`<h_1d1d4f05>` item spawns of the Sentry h_e5f1f063), false on its pickups
    pub placed: bool,
    /// an item object's `item-alignment` (0x14a0c4: +0x1c0, the team a mine counts as its own,
    /// the ALIGN_ numbers): 1 (the enemies') on the tutorial's mines, -1 on its pickups; -1
    /// without one
    pub alignment: i64,
}

/// A level camera-object (`<camera-object name h_1435f288 h_19bc8fdf h_125978e4><transform>`,
/// its attribute handler 0x6cb90): what a cutscene script's CAMERA entity of that name is bound to
/// (FUN_00175090 finds it among the world's objects by its name hash), looking down its -z.
/// `half_fov` is h_1435f288, the horizontal half angle in degrees: the handler hands the camera
/// twice it (FUN_0006e450 takes the whole horizontal angle; the aspect is 4:3); 30-45 on the
/// tutorial's. `near` / `far`: h_19bc8fdf and h_125978e4 (0.005 and 500 on the tutorial's).
#[derive(Clone, Debug)]
pub struct LevelCamera {
    pub name: u32,
    /// its type (h_ff8051ee): `camera` for the scripts' cameras; END_CAMERA for the views the end
    /// screen picks from (FUN_001092e0)
    pub kind: u32,
    pub transform: Mat4,
    pub half_fov: f32,
    pub near: f32,
    pub far: f32,
}

/// The camera type the end screen cuts the view to: as the end screen opens, FUN_001092e0 picks
/// one of the level's camera-objects whose type's name (+0xc, +8) is h_ee3b0d8c at random (the
/// game's random state 0x46cfd8) and shows the world through it. The tutorial has one,
/// h_046b169f at (-12.8, 27.4, -5.6): take12's view of the base behind MISSION SUCCESSFUL, its
/// camera point (-12.82, -5.63) in the watch log from 359.6 s.
pub const END_CAMERA: u32 = 0xEE3B_0D8C;

/// A level path-object (`<path-object name><transform><points><points time><transform>..`, its
/// attribute handler 0x16cca0): what a script's MOTION_PATH entity of that name is bound to. Its
/// keys (time in s, the 3 x 3 and position in the path's own frame: `transform` places them) are
/// what a START_PATH_ANIMATION follows (play_script.rs).
#[derive(Clone, Debug)]
pub struct LevelPath {
    pub name: u32,
    pub transform: Mat4,
    pub keys: Vec<(f32, Mat4)>,
}

/// A patrol point of the level: a `<patrol-point>` (type patrol_point, h_efd397a1: the world
/// point factory FUN_000ee070 makes it with FUN_000ec180, vtable 0x39e608, kind 11 by its +0x3c
/// slot 0x1031c0) or a `<patrol-first-point>` (h_09528bf8, type patrol_first_point h_0eef0757:
/// FUN_000ec5a0, vtable 0x39eac0, kind 12, slot 0x1f88a0), the head of a chain. The world keeps
/// both kinds, and only them, in one list (world+0x1d0: the add by kind at 0xdab13, jump table
/// 0xdb374), which GOAL_PATROL's start walks (FUN_0002af70). The tutorial has two chains: iD 1, a
/// head at (18.8, 35.2) and ten points north to (52.2, 76.9); iD 2, a head by the base's gate at
/// (-37.7, 0.1) and thirteen points through the base to (-78.6, 47.6).
#[derive(Clone, Debug)]
pub struct PatrolPoint {
    pub name: u32,
    /// its `<position>`
    pub at: Vec3,
    /// a chain's head (kind 12), else a patrol point (kind 11)
    pub head: bool,
    /// `iD`: the low 12 bits of the object's +0x24 (the world point attribute handler
    /// FUN_000ecdc0, h_f3dd6750): the chain it belongs to. GOAL_PATROL takes the head whose iD is
    /// the nearest point's (FUN_0002af70 compares `(+0x24 ^ +0x24) & 0xfff`)
    pub group: i64,
    /// forward-object and backward-object (FUN_000eddd0: +0x3c / +0x44, the links of the chain;
    /// 0 none) and trigger-object (+0x4c: a trigger sent an event as a patroller reaches the point,
    /// FUN_00026940; none in the tutorial)
    pub forward: u32,
    pub backward: u32,
    pub trigger: u32,
    /// h_e6564e8a (+0x20, FUN_000ecdc0; -1 none, 0.3 at least when 0-0.25): the point's own
    /// arrival radius, which replaces the walk's when it's above 0 (FUN_00026d80). The tutorial's
    /// base chain has 1 on its four points by the gate, the rest -1
    pub radius: f32,
}

/// light-usage LU_INDOOR (the LU_ name table 0x3bf954: LU_NONE 0, LU_INDOOR 1, LU_OUTDOOR 2,
/// LU_GEOSET 3).
pub const LU_INDOOR: u8 = 1;
/// light-usage LU_OUTDOOR.
pub const LU_OUTDOOR: u8 = 2;
/// light-usage LU_GEOSET: indoors where its triangles say so (`collision::MATERIAL_INDOOR`).
pub const LU_GEOSET: u8 = 3;

/// A radar-trigger (`<radar-trigger radar-contact=N h_e2145a6c=T h_12b33046=B>`): on
/// TRIG_ACT_OPERATE its action handler (0x207230) sets radar-contact N, with a time T, on each
/// object in its h_0b6b92ba list (FUN_0015b310 at 0x207321). Only those acting on the level's
/// start are run (`Level::load`); the others wait for the level's trigger runtime.
pub struct RadarTrigger {
    pub name: u32,
    /// `enabled` (+0x3d bit 0): a disabled one runs only the base actions (0x207236)
    pub enabled: bool,
    /// `radar-contact` (+0x40): the value it sets
    pub contact: u8,
    /// h_e2145a6c (+0x44): the time it gives the objects (s; the radar's blink, #160)
    pub time: f32,
    /// h_12b33046 (+0x48): with a 2 or a 3, a cue follows (FUN_00179d60; inferred to be a sound)
    pub cue: bool,
    /// its entries (h_031cd9f5 > base): (signal, action), the TRIG_SIG_ and TRIG_ACT_ enums
    pub on: Vec<(i64, i64)>,
    /// the objects it sets (h_0b6b92ba objects)
    pub objects: Vec<u32>,
}

/// TRIG_SIG_FIRST_UPDATE (the TRIG_SIG_ name table 0x3bef5c): every enabled trigger sends it to
/// itself once, at the level's start (FUN_001fe610 / FUN_001fe660).
pub const TRIG_SIG_FIRST_UPDATE: i64 = 13;
/// TRIG_ACT_OPERATE (the TRIG_ACT_ name table 0x3bed0c): what a radar-trigger sets its value on.
pub const TRIG_ACT_OPERATE: i64 = 1;

/// A directional light of the level: a light-object whose type's `light-type` is
/// LT_DIRECTIONAL (h_e1cac6f7, h_f5302dc6, h_157b3160, h_e8d6efe4 in the game's data; 146 + 15
/// placements). The game lights an object with two of them, a pair (FUN_000dcc10): its
/// light-usage picks the pair (the wrapper material's draw 0x9f3c0: LU_INDOOR the indoor one,
/// LU_OUTDOOR the outdoor one, LU_GEOSET each geoset's own flag), and a light belongs to the pair
/// its flags say, either or both (the tutorial's fill lights both). The three types' records are
/// alike field for field: nothing in them says key or fill.
#[derive(Clone, Copy, Debug)]
pub struct LevelLight {
    /// its type (h_ff8051ee) and its own name
    pub kind: u32,
    pub name: u32,
    pub color: [f32; 3],
    /// the way the light shines (world, unit length): minus the third column of its transform,
    /// which FUN_000dcc10 hands the shaders as the way toward the light (the matrix's bytes 8,
    /// 0x14 and 0x20)
    pub dir: Vec3,
    /// that column's length: the shaders take N.L against the column as it is (vertex program
    /// 0x19's `max(0, n . c16)`; FUN_000dcc10 copies it unnormalised), so a hand-made transform
    /// whose column is short lights that much less (e40's key 0.944, its outdoor fill 0.834; 34
    /// of the 161 placements are off by over 2%)
    pub strength: f32,
    /// h_e02aba1c: one of the indoor pair
    pub indoor: bool,
    /// h_1f416687: one of the outdoor pair
    pub outdoor: bool,
}

impl LevelLight {
    /// Its colour as it lights a surface facing it: `color` x `strength` (0-1, gamma terms).
    pub fn lit_color(&self) -> [f32; 3] {
        self.color.map(|c| c * self.strength)
    }
}

/// A point light: a light-object whose type's `light-type` is LT_POINT (h_ea460e64, h_e81ba579,
/// h_087ae059, h_09020c30, h_f1557cc8, h_ff66877a: 1290 + 959 placements). Its place, colour
/// (0-1), the distance it reaches (`range`, m) and its `falloff` (an enum: 3 or 4 on every one;
/// unnamed in the schema). An object takes the lamps of its pair only: FUN_0009f0b0 gathers a
/// block's lamps from those with the light's +0x94 bit 2 (h_1f416687) for the outdoor pair, bit
/// 1 (h_e02aba1c) for the indoor one (its fourth argument, the pair 0x9f3c0 picked, 0x9f688); one
/// with neither lights nothing (32 on m09_c).
pub struct LevelLamp {
    /// its type (h_ff8051ee) and its own name
    pub kind: u32,
    pub name: u32,
    pub at: Vec3,
    pub color: [f32; 3],
    pub range: f32,
    pub falloff: i64,
    /// h_e02aba1c: lights what the indoor pair lights
    pub indoor: bool,
    /// h_1f416687: lights what the outdoor pair lights (characters, LU_OUTDOOR on 535 of the
    /// 542 placed ones)
    pub outdoor: bool,
}

/// A light-object the demo doesn't draw: its type, its name and why.
pub struct OtherLight {
    pub kind: u32,
    pub name: u32,
    pub why: &'static str,
}

/// Why a light-object isn't drawn (`OtherLight`): an effect object's light. 373 point lights
/// (all 159 h_09020c30, 214 h_e81ba579) are `<parent Type="1">` of an effect-object, at its
/// origin: the effect types that carry one name its type (their h_f8c6a8b4) and an ALE
/// light-effect, which drives it (an inference, as `ale_fx` reads the effects' light; their
/// colour is 1 1 1 and range 10 on every one). The demo doesn't run placed effect objects (#258),
/// so their lights wait for that.
pub const EFFECT_LIGHT: &str = "its effect object's light (run by its light-effect; #258)";
/// Why a light-object isn't drawn: a spot light (LT_SPOT; none in the game's data).
pub const SPOT_LIGHT: &str = "a spot light";
/// Why a light-object isn't drawn: its type isn't a light type the loaded objecttypes know.
pub const NO_LIGHT_TYPE: &str = "no light type of that name";
/// Why a light-object isn't drawn: its parent (`<parent Type="1">`) isn't among the level's
/// placements.
pub const NO_PARENT: &str = "its parent isn't placed";

/// A light-object's `<parent Type>` (the object link FUN_0015f0b0 reads into the light's +0xa4,
/// updated by FUN_0015ed30): 1 sits at its parent object, its transform the parent's times the
/// link's offset, identity when the `<parent>` names none (none does); 2 a named node of its
/// parent, and with no node named (none is) the light keeps its own transform; 0 none.
const PARENT_OBJECT: i64 = 1;

/// A wall panel (world-button-object): its name, placement and the event it sends when used:
/// its signal h_eaf8a35b (+0x1b0) and data h_f1660a0d (+0x1b4; -1 when not given, its
/// constructor FUN_00162e90), through 0x163060 to its listeners.
pub struct Button {
    pub name: u32,
    /// its object type (h_ee4c83c9 for the gate panels)
    pub kind: u32,
    pub transform: Mat4,
    pub signal: i64,
    /// h_f1660a0d: the event's data (-1 when not given)
    pub data: i64,
}

pub struct Level {
    /// terrain layers: geoset (positions in world space) and its mask texture (None: the ground)
    pub terrain: Vec<(Geoset, Option<u32>)>,
    pub objects: Vec<Placement>,
    /// sky mesh geosets and where they sit
    pub sky: Vec<Geoset>,
    pub sky_at: Vec3,
    pub background: [f32; 3],
    /// colour (0-1), start, end (m)
    pub fog: Option<([f32; 3], f32, f32)>,
    pub ambient: [f32; 3],
    pub cameras: Vec<Mat4>,
    /// collision: the terrain blocks' surfaces (world frame) and the invisible blockers
    /// (<blocker h_051ac63e=surface object-instance=owner><transform>): surface, placement and
    /// the placed object it belongs to (its name; 0 for none). A breakable object's blocker goes
    /// with it (sdm_e34's missile rack h_128f3de2: blocker h_e1a3f88e, physics-id 4)
    pub terrain_collision: Vec<u32>,
    pub blockers: Vec<(u32, Mat4, u32)>,
    /// directional lights (`LevelLight`), in the level file's order, and the terrain's ambient
    /// (`ambient` is the objects')
    pub lights: Vec<LevelLight>,
    /// point lights (`LevelLamp`), in the file's order
    pub lamps: Vec<LevelLamp>,
    /// the light-objects that aren't drawn, and why (`OtherLight`)
    pub other_lights: Vec<OtherLight>,
    pub terrain_ambient: [f32; 3],
    /// `<character-ambient>` (h_e36353c3, the level settings' +0x44, parser 0x198690): the skinned
    /// meshes' ambient, c15 (0xda19b), where the static meshes take `ambient` (c14). Read, not
    /// drawn: characters take `ambient` here
    pub character_ambient: [f32; 3],
    /// half the terrain's width (m): its grid is width blocks x 16 cells of h_e023cdc9 m,
    /// centred on the origin; the layer masks span it
    pub terrain_half: f32,
    /// the terrain's baked light (Terrain > blocks h_e2d79bad): blocks per side, cell size (m)
    /// and per block 16 x 16 cell values 0-255 (sunlit ~250, shadowed ~25)
    pub terrain_light: Option<(usize, f32, Vec<Vec<u8>>)>,
    pub buttons: Vec<Button>,
    /// every trigger of the level, in the file's order (`bf::triggers`)
    pub triggers: Vec<triggers::Trigger>,
    /// the level's radar-triggers (see `RadarTrigger`); those acting at its start have set
    /// `Placement::radar_contact` already
    pub radar_triggers: Vec<RadarTrigger>,
    /// the level's camera-objects by name (the cutscenes' cameras, `LevelCamera`; `cameras` has
    /// their transforms alone, for the level viewer)
    pub camera_objects: Vec<LevelCamera>,
    /// the level's path-objects (the cutscenes' camera paths, `LevelPath`)
    pub paths: Vec<LevelPath>,
    /// `<compiled max-score>`: the most cash the zone can pay, at BRUTAL (its parser 0x1982a0
    /// keeps it at the level record's +0x120; the tutorial's 11760 = (5000 + 1000 + 24 x 100) x
    /// 1.4). The end screen shows it at the difficulty played ("Cash $x / $5040" on STANDARD,
    /// take05 and take12). 0 when the file has none
    pub max_score: f32,
    /// the level's patrol points, both kinds, in the file's order (`PatrolPoint`)
    pub patrols: Vec<PatrolPoint>,
}

fn floats(e: Option<&Element>) -> Vec<f32> {
    e.and_then(|e| e.text.as_ref()).map(|t| t.floats()).unwrap_or_default()
}

fn hash(e: &Element, name: u32) -> u32 {
    e.attr(name).and_then(|v| v.as_hash()).unwrap_or(0)
}

/// A world button, world-button-object (h_16f22d4b): a wall panel.
const H_WORLD_BUTTON: u32 = 0x16F2_2D4B;
/// A world button's signal (h_eaf8a35b, +0x1b0: 14, USE_OBJECT, on every one).
const H_BUTTON_SIGNAL: u32 = 0xEAF8_A35B;
/// A world button's data, sent with its signal (h_f1660a0d, +0x1b4: FUN_00162fc0).
const H_BUTTON_DATA: u32 = 0xF166_0A0D;
/// A trigger's entries list (`bf::triggers::H_ENTRIES`), for the radar-triggers.
const H_SIGNALS: u32 = triggers::H_ENTRIES;
/// A trigger's objects (`bf::triggers::H_OBJECTS`), for the radar-triggers.
const H_TRIGGER_OBJECTS: u32 = triggers::H_OBJECTS;
/// A camera-object's h_1435f288: its horizontal half angle of view in degrees (the attribute
/// handler 0x6cb90 passes twice it to FUN_0006e450, which takes the whole angle).
const H_CAMERA_HALF_FOV: u32 = 0x1435_F288;
/// A camera-object's h_19bc8fdf: its near plane (0x6cb90 -> FUN_0006ac90).
const H_CAMERA_NEAR: u32 = 0x19BC_8FDF;
/// A camera-object's h_125978e4: its far plane (0x6cb90, the view's +0x20).
const H_CAMERA_FAR: u32 = 0x1259_78E4;
/// The patrol points' elements: `patrol-point` (h_ffcf1f13) and `patrol-first-point`
/// (h_09528bf8, names recovered by hashing; their types patrol_point h_efd397a1 and
/// patrol_first_point h_0eef0757 are the strings at 0x40ceac and 0x40ce98).
const H_PATROL_POINT: u32 = 0xFFCF_1F13;
const H_PATROL_HEAD: u32 = 0x0952_8BF8;
/// A world point's h_e6564e8a (+0x20, FUN_000ecdc0): its own arrival radius (-1 none).
const H_POINT_RADIUS: u32 = 0xE656_4E8A;
/// An item object's h_1cb7d5f2 (its attribute handler 0x14a0a0 at 0x14a0f7): placed and live
/// (`Placement::placed`).
const H_ITEM_PLACED: u32 = 0x1CB7_D5F2;

/// A level <transform>: the 3 x 3 rotation row by row (columns are the object's axes: the
/// level's cameras come out level and looking down -z), then the position.
pub fn transform(v: &[f32]) -> Option<Mat4> {
    (v.len() >= 12).then(|| {
        let c = |i: usize| Vec3::new(v[i], v[i + 3], v[i + 6]).extend(0.0);
        Mat4::from_cols(c(0), c(1), c(2), Vec3::new(v[9], v[10], v[11]).extend(1.0))
    })
}

impl Level {
    /// The terrain's baked light (0-1) at world (x, z), by one of 8 grid orientations
    /// (bit 0: x runs the other way, bit 1: z does, bit 2: rows and columns swap); see
    /// `TERRAIN_LIGHT_ORIENTATION`.
    pub fn baked_light(&self, x: f32, z: f32, orientation: u8) -> Option<f32> {
        let (blocks, cell, maps) = self.terrain_light.as_ref()?;
        let n = (blocks * 16) as i64;
        let mut cx = ((x + self.terrain_half) / cell).floor() as i64;
        let mut cz = ((z + self.terrain_half) / cell).floor() as i64;
        if !(0..n).contains(&cx) || !(0..n).contains(&cz) {
            return None;
        }
        if orientation & 1 != 0 { cx = n - 1 - cx; }
        if orientation & 2 != 0 { cz = n - 1 - cz; }
        if orientation & 4 != 0 { std::mem::swap(&mut cx, &mut cz); }
        let (bx, bz) = ((cx / 16) as usize, (cz / 16) as usize);
        let map = maps.get(bz * blocks + bx)?;
        map.get(((cz % 16) * 16 + cx % 16) as usize).map(|&v| v as f32 / 255.0)
    }

    /// A level's sky from its level file's elements (`all`): the `<sky><object mesh-name>`
    /// layers, where they sit (its `<position>`) and the `<background-color>`. No layers without
    /// a sky mesh (sdm_m07).
    pub fn sky_of(game: &Game, all: &[&Element]) -> (Vec<Geoset>, Vec3, [f32; 3]) {
        let sky_el = all.iter().find(|e| e.name == h("sky"));
        let sky_obj = sky_el.and_then(|s| s.child(h("object")));
        // a mesh or an archetype (sdm_e34's is an archetype 3.2 km across)
        let sky = match sky_obj.map(|o| hash(o, h("mesh-name"))) {
            Some(m) if m != 0 => weapon::mesh_geosets(game, m).ok().filter(|g| !g.is_empty())
                .or_else(|| weapon::WeaponModel::load(game, m).ok().map(|w| w.parts.into_iter().flat_map(|p| p.geosets).collect()))
                .unwrap_or_default(),
            _ => vec![],
        };
        let p = floats(sky_obj.and_then(|o| o.child(h("position"))));
        let sky_at = if p.len() >= 3 { Vec3::new(p[0], p[1], p[2]) } else { Vec3::ZERO };
        let bg = floats(sky_el.and_then(|s| s.child(h("background-color"))));
        let background = if bg.len() >= 3 { [bg[0], bg[1], bg[2]] } else { [0.5, 0.6, 0.7] };
        (sky, sky_at, background)
    }

    /// The sky of the level loaded last (`Game::load_level`), alone (see `sky_of`).
    pub fn load_sky(game: &Game) -> Result<(Vec<Geoset>, Vec3, [f32; 3]), String> {
        let root = game.levels.last().ok_or("no level file loaded")?;
        Ok(Self::sky_of(game, &root.walk()))
    }

    /// The sky mesh a level file names (its `<sky><object mesh-name>`; 0: none), to list the
    /// levels that have a sky without loading them. 51 of the 54 level files name one; sdm_m07,
    /// m07_c and splash_screen have no `<object>` in their `<sky>`.
    pub fn sky_mesh(root: &Element) -> u32 {
        root.walk().into_iter().find(|e| e.name == h("sky")).and_then(|s| s.child(h("object"))).map_or(0, |o| hash(o, h("mesh-name")))
    }

    /// The level loaded last (`Game::load_level`).
    pub fn load(game: &Game) -> Result<Self, String> {
        let root = game.levels.last().ok_or("no level file loaded")?;
        let all = root.walk();
        // terrain
        let mut terrain = vec![];
        let mut terrain_half = TERRAIN_HALF;
        if let Some(t) = all.iter().find(|e| e.name == h("Terrain")) {
            let mesh = hash(t, h("mesh-name"));
            // (sdm_m07 names a terrain mesh that's on no disc archive: the map is all objects)
            let mut geosets = weapon::mesh_geosets(game, mesh).unwrap_or_else(|e| {
                eprintln!("terrain: {e}; going on without it");
                vec![]
            });
            // the terrain vertex shader splits |short a| into row and column by a per-level
            // constant (8 x blocks + 1: half the grid a side) and scales them by the cell size
            // (h_e023cdc9); the decoder used Battle of Bulgar's (65, 2 m), so re-split
            let blocks = t.attr(h("width")).and_then(|v| v.as_i64()).unwrap_or(8).max(1) as f32;
            let cell = t.attr(H_TERRAIN_CELL).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(2.0);
            let row = 8.0 * blocks + 1.0;
            terrain_half = 8.0 * blocks * cell;
            if (row, cell) != (DECODED_ROW, DECODED_CELL) {
                for g in &mut geosets {
                    for p in &mut g.positions {
                        let a = (p[2].abs() / DECODED_CELL).round() * DECODED_ROW + (p[0].abs() / DECODED_CELL).round();
                        let (r, c) = ((a / row).floor(), a % row);
                        p[0] = p[0].signum() * c * cell;
                        p[2] = p[2].signum() * r * cell;
                    }
                }
            }
            for (i, g) in geosets.into_iter().enumerate() {
                let mask = (i > 0).then(|| game.material_param_texture(g.material, h("alpha"))).flatten();
                terrain.push((g, mask));
            }
        }
        // placed objects
        let mut objects = vec![];
        for e in all.iter().filter(|e| e.attr(H_TYPE).is_some() && e.name != h("camera-object")) {
            // a <transform>, or (start points, ...) an <ORIENTATION> and a <position>
            let mut t = floats(e.child(h("transform")));
            let mut oriented = t.len() >= 12;
            if t.len() < 12 {
                let (o, p) = (floats(e.child(h("ORIENTATION"))), floats(e.child(h("position"))));
                if p.len() >= 3 {
                    oriented = o.len() >= 9;
                    t = if o.len() >= 9 { o[..9].to_vec() } else { vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] };
                    t.extend_from_slice(&p[..3]);
                }
            }
            let Some(m) = transform(&t) else { continue };
            let kind = hash(e, H_TYPE);
            let radar_contact = e.attr(H_RADAR_CONTACT).and_then(|v| v.as_i64()).map(|v| v.clamp(0, 255) as u8)
                .or_else(|| game.object_types.get(&kind).map(|t| t.radar_contact)).unwrap_or(0);
            let light_usage = e.attr(h("light-usage")).and_then(|v| v.as_i64()).map_or(0, |v| v.clamp(0, 255) as u8);
            objects.push(Placement { tag: e.name, name: hash(e, h("name")), kind, archetype: game.object_meshes.get(&kind).copied(), transform: m,
                                     radar_contact, light_usage, oriented,
                                     radius: e.attr(H_POINT_RADIUS).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(-1.0),
                                     placed: e.attr(H_ITEM_PLACED).and_then(|v| v.as_i64()).is_some_and(|v| v != 0),
                                     alignment: e.attr(h("item-alignment")).and_then(|v| v.as_i64()).unwrap_or(-1) });
        }
        // the radar-triggers; those that operate on the level's start (an entry on
        // TRIG_SIG_FIRST_UPDATE with TRIG_ACT_OPERATE, enabled) set their objects' value now, in
        // the level file's order
        let flag = |e: &Element, k: u32| e.attr(k).and_then(|v| v.as_i64()).is_some_and(|v| v != 0);
        let radar_triggers: Vec<RadarTrigger> = all.iter().filter(|e| e.name == h("radar-trigger")).map(|e| RadarTrigger {
            name: hash(e, h("name")),
            enabled: flag(e, h("enabled")),
            contact: e.attr(h("radar-contact")).and_then(|v| v.as_i64()).map_or(0, |v| v.clamp(0, 255) as u8),
            time: e.attr(0xE214_5A6C).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(0.0),
            cue: flag(e, 0x12B3_3046),
            on: e.child(H_SIGNALS).map(|s| s.children_named(h("base")).filter_map(|b| {
                Some((b.attr(h("signal")).and_then(|v| v.as_i64())?, b.attr(h("action")).and_then(|v| v.as_i64()).unwrap_or(TRIG_ACT_OPERATE)))
            }).collect()).unwrap_or_default(),
            objects: triggers::object_list(e, H_TRIGGER_OBJECTS),
        }).collect();
        for t in radar_triggers.iter().filter(|t| t.enabled && t.on.contains(&(TRIG_SIG_FIRST_UPDATE, TRIG_ACT_OPERATE))) {
            for o in objects.iter_mut().filter(|o| o.name != 0 && t.objects.contains(&o.name)) {
                o.radar_contact = t.contact;
            }
        }
        let (sky, sky_at, background) = Self::sky_of(game, &all);
        // fog and ambient
        let fog = all.iter().find(|e| e.name == H_FOG).and_then(|f| {
            let c = floats(f.child(h("color")));
            let d = |n: &str| f.attr(h(n)).and_then(|v| v.as_f32());
            (c.len() >= 3).then(|| ([c[0] / 255.0, c[1] / 255.0, c[2] / 255.0], d("start-distance").unwrap_or(50.0), d("end-distance").unwrap_or(200.0)))
        });
        let a = floats(all.iter().find(|e| e.name == h("object-ambient")).copied());
        let ambient = if a.len() >= 3 { [a[0], a[1], a[2]] } else { [0.15; 3] };
        let cameras = all.iter().filter(|e| e.name == h("camera-object"))
            .filter_map(|c| transform(&floats(c.child(h("transform"))))).collect();
        // the cameras by name with their FOV, and the path-objects with their keys (#222)
        let number = |e: &Element, k: u32, or: f32| e.attr(k).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(or);
        let camera_objects = all.iter().filter(|e| e.name == h("camera-object")).filter_map(|c| Some(LevelCamera {
            name: hash(c, h("name")),
            kind: hash(c, H_TYPE),
            transform: transform(&floats(c.child(h("transform"))))?,
            half_fov: number(c, H_CAMERA_HALF_FOV, 35.0),
            near: number(c, H_CAMERA_NEAR, 0.005),
            far: number(c, H_CAMERA_FAR, 500.0),
        })).collect();
        let paths = all.iter().filter(|e| e.name == h("path-object")).filter_map(|p| Some(LevelPath {
            name: hash(p, h("name")),
            transform: transform(&floats(p.child(h("transform"))))?,
            keys: p.child(h("points")).map(|list| list.children_named(h("points")).filter_map(|k| {
                Some((number(k, h("time"), 0.0), transform(&floats(k.child(h("transform"))))?))
            }).collect()).unwrap_or_default(),
        })).collect();
        let surface = |e: &Element| hash(e, H_PHYSICS_NAME);
        let terrain_collision = all.iter().filter(|e| e.name == h("blocks")).map(|e| surface(e))
            .filter(|&n| n != 0 && n != h("")).collect();
        let blockers = all.iter().filter(|e| e.name == h("blocker"))
            .filter_map(|e| Some((surface(e), transform(&floats(e.child(h("transform"))))?, hash(e, h("object-instance")))))
            .filter(|(n, _, _)| *n != 0 && *n != h("")).collect();
        // the light-objects, each by its type's light-type (`Game::light_types`)
        let (mut lights, mut lamps, mut other_lights) = (vec![], vec![], vec![]);
        for e in all.iter().filter(|e| e.name == h("light-object")) {
            let (kind, name) = (hash(e, H_TYPE), hash(e, h("name")));
            let other = |why| OtherLight { kind, name, why };
            let c = floats(e.child(H_LIGHT_COLOR));
            let color = if c.len() >= 3 { [c[0], c[1], c[2]] } else { [1.0; 3] };
            let flag = |k: u32| e.attr(k).and_then(|v| v.as_i64()).is_some_and(|v| v != 0);
            let (indoor, outdoor) = (flag(H_LIGHT_INDOOR), flag(H_LIGHT_OUTDOOR));
            // where it sits: on a `<parent Type="1">` its parent's transform times the link's
            // offset (the `<parent>`'s own `<transform>`), else its own (PARENT_OBJECT)
            let parent = e.child(h("parent")).filter(|p| p.attr(h("Type")).and_then(|v| v.as_i64()) == Some(PARENT_OBJECT))
                .map(|p| (hash(p, h("parent")), transform(&floats(p.child(h("transform")))).unwrap_or(Mat4::IDENTITY)))
                .filter(|(n, _)| *n != 0 && *n != h(""));
            let placed = match parent {
                Some((p, offset)) => match objects.iter().find(|o| o.name == p) {
                    // (an effect's light is the effect's to run: EFFECT_LIGHT)
                    Some(o) if o.tag == h("effect-object") => { other_lights.push(other(EFFECT_LIGHT)); continue; }
                    Some(o) => o.transform * offset,
                    None => { other_lights.push(other(NO_PARENT)); continue; }
                },
                None => match transform(&floats(e.child(h("transform")))) {
                    Some(m) => m,
                    None => continue,
                },
            };
            match game.light_types.get(&kind).copied() {
                Some(LT_DIRECTIONAL) => lights.push(LevelLight {
                    kind, name, color, indoor, outdoor,
                    dir: -placed.z_axis.truncate().normalize_or_zero(),
                    strength: placed.z_axis.truncate().length(),
                }),
                Some(LT_POINT) => {
                    let num = |n: &str| e.attr(h(n)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
                    lamps.push(LevelLamp {
                        kind, name, color, indoor, outdoor,
                        at: placed.w_axis.truncate(),
                        range: num("range").unwrap_or(10.0),
                        falloff: num("falloff").map_or(3, |f| f as i64),
                    });
                }
                Some(LT_SPOT) => other_lights.push(other(SPOT_LIGHT)),
                _ => other_lights.push(other(NO_LIGHT_TYPE)),
            }
        }
        let ca = floats(all.iter().find(|e| e.name == h("character-ambient")).copied());
        let character_ambient = if ca.len() >= 3 { [ca[0], ca[1], ca[2]] } else { ambient };
        let ta = floats(all.iter().find(|e| e.name == h("Terrain")).and_then(|t| t.child(h("ambient"))));
        let terrain_ambient = if ta.len() >= 3 { [ta[0], ta[1], ta[2]] } else { ambient };
        let terrain_light = all.iter().find(|e| e.name == h("Terrain")).map(|t| {
            let blocks = t.attr(h("width")).and_then(|v| v.as_i64()).unwrap_or(8).max(1) as usize;
            let cell = t.attr(H_TERRAIN_CELL).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(2.0);
            let maps = t.children_named(h("blocks")).map(|b| {
                b.child(H_BAKED_LIGHT).and_then(|e| e.text.as_ref()).map(|v| v.ints().into_iter().map(|i| i.clamp(0, 255) as u8).collect()).unwrap_or_default()
            }).collect();
            (blocks, cell, maps)
        });
        // no sky and a terrain mesh that's on no disc (sdm_m07's cavern): the background shows
        // through where the terrain should be, so let it be the fog's colour, not a bright sky's
        let background = match fog {
            Some((c, ..)) if sky.is_empty() && terrain.is_empty() && all.iter().any(|e| e.name == h("Terrain")) => c,
            _ => background,
        };
        let buttons = all.iter().filter(|e| e.name == H_WORLD_BUTTON && e.attr(h("reticule-action")).and_then(|v| v.as_i64()) == Some(1))
            .filter_map(|e| Some(Button {
                name: hash(e, h("name")),
                kind: hash(e, H_TYPE),
                transform: transform(&floats(e.child(h("transform"))))?,
                signal: e.attr(H_BUTTON_SIGNAL).and_then(|v| v.as_i64())?,
                data: e.attr(H_BUTTON_DATA).and_then(|v| v.as_f32()).map_or(-1, |v| v as i64),
            })).collect();
        let triggers = triggers::read(&all);
        let max_score = all.iter().find(|e| e.name == h("compiled")).and_then(|e| e.attr(h("max-score")))
            .and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(0.0);
        // the patrol points (#227): both kinds, by their attribute handler's names (FUN_000ecdc0,
        // FUN_000eddd0)
        let patrols = all.iter().filter(|e| e.name == H_PATROL_POINT || e.name == H_PATROL_HEAD).filter_map(|e| {
            let p = floats(e.child(h("position")));
            // (an empty link is the empty name's hash: none)
            let link = |k: u32| Some(hash(e, k)).filter(|&n| n != h("")).unwrap_or(0);
            Some(PatrolPoint {
                name: hash(e, h("name")),
                at: (p.len() >= 3).then(|| Vec3::new(p[0], p[1], p[2]))?,
                head: e.name == H_PATROL_HEAD,
                group: e.attr(h("iD")).and_then(|v| v.as_i64()).unwrap_or(0) & 0xfff,
                forward: link(h("forward-object")),
                backward: link(h("backward-object")),
                trigger: link(h("trigger-object")),
                radius: number(e, H_POINT_RADIUS, -1.0),
            })
        }).collect();
        Ok(Level { terrain, objects, sky, sky_at, background, fog, ambient, cameras, terrain_collision, blockers, lights, lamps, other_lights,
                   terrain_ambient, character_ambient, terrain_half, terrain_light, buttons, triggers, radar_triggers, camera_objects, paths,
                   max_score, patrols })
    }

    /// The directional lights FUN_000dcc10 puts in the indoor pair (`indoor`) or the outdoor one:
    /// those with the pair's flag (`LevelLight::indoor`, `outdoor`), two at most. It walks the
    /// world's lights (+0x22c) and fills the pair's two slots in the list's order (a slot rule on
    /// the type's +0x50 bit 3 never applies: that byte is the parsed bool h_ee2faf73, 0 or 1); no
    /// level of the game has more than two lights in either pair, so the order never decides.
    pub fn pair(&self, indoor: bool) -> Vec<&LevelLight> {
        self.lights.iter().filter(|l| if indoor { l.indoor } else { l.outdoor }).take(2).collect()
    }

    /// What using a button does to the level's animated objects, read from the data alone: (object
    /// name, clip, times) for each anim-trigger entry its signal reaches unchanged (the routers
    /// listening to the button, and on through the triggers they send to). The trigger runtime
    /// (play_triggers.rs) is what opens them; this is for the panel's prompt (play.rs `doors`).
    /// A chain that changes the signal on the way (the tutorial gate's: the panel's sound-trigger,
    /// then its FINISHED) isn't followed.
    pub fn button_opens(&self, button: &Button) -> Vec<(u32, u32, i64)> {
        let mut todo: Vec<&triggers::Trigger> = self.triggers.iter().filter(|t| t.tag == h("router-trigger") && t.objects.contains(&button.name)).collect();
        let mut seen = vec![];
        let mut out = vec![];
        while let Some(t) = todo.pop() {
            if seen.contains(&t.name) {
                continue;
            }
            seen.push(t.name);
            for e in &t.entries {
                let clip = e.anim_clip();
                if e.signal == button.signal && clip != 0 {
                    out.extend(t.objects.iter().map(|&o| (o, clip, e.limit)));
                }
            }
            todo.extend(self.triggers.iter().filter(|n| t.targets.contains(&n.name)));
        }
        out
    }
}
