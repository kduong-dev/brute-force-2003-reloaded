//! A level file (levels-<name>.xmb, loaded with `Game::load_level`): the terrain mesh, the
//! placed objects, the sky, fog and lighting colours, and the level's own cameras.
//!
//!  Terrain     mesh-name: a mesh of terrain vertices (see `character::TERRAIN_VERTEX`), one
//!              geoset per texture layer; the first is the ground, the others are blended over
//!              it by their material's "alpha" mask, one 128 px mask across the 8 x 8 blocks of
//!              32 m (x and z -128..128; mask v runs toward -z).
//!  objects     every element with an object type (h_ff8051ee) and a <transform> (3 x 3,
//!              then the position): game objects, weapon and inventory pickups, start points...
//!              The type's objecttypes entry names its mesh archetype.
//!  sky         <object mesh-name> at <position>, <background-color>
//!  fog         h_1a467f02: <color> (0-255), start-distance, end-distance
//!  cameras     camera-object transforms (the level's flyby cameras); by name with their FOV,
//!              the cutscenes' cameras (`LevelCamera`), and the path-objects their scripts move
//!              them along (`LevelPath`, #222)
//!  lights      light-object (types h_e1cac6f7 key, h_f5302dc6 fill: directional; h_ea460e64
//!              point): colour h_f08eb2f3 (0-1), shining down the transform's -z; h_e02aba1c
//!              marks the pair that lights the terrain (the others light objects and characters).
//!              Ambient: Terrain <ambient>, <object-ambient>, <character-ambient>.
//!  collision   the terrain blocks' collision surfaces (Terrain > blocks h_051ac63e) and the
//!              invisible blockers (blocker h_051ac63e + transform), see bf::collision
//!  sounds      sound-trigger: a sound id (sound-object) played on a signal (h_031cd9f5 > base
//!              signal: 31 beside each door when it opens, 49 when it closes) at its position
//!  buttons     world-button-object (h_16f22d4b, reticule-action 1): a wall panel that sends
//!              its signal (h_eaf8a35b, 14 on every one) when used
//!  triggers    every element built on the game's trigger base (routers, timers, calculators,
//!              spawn-triggers, detectors, anim-triggers ...), read by `bf::triggers` and run by
//!              bf_play's trigger runtime (play_triggers.rs). A gate's panels reach its
//!              anim-triggers through a router (see `Level::button_opens`).
//!  radar       each placement's radar-contact (h_1905f381, else its type's), and the
//!              radar-triggers (radar-contact N on their h_0b6b92ba objects); those that act at
//!              the level's start (signal 13) are applied as it loads (`RadarTrigger`)

use bevy::math::{Mat4, Vec3};

use super::bxml::Element;
use super::character::{Game, Geoset, H_PHYSICS_NAME, H_RADAR_CONTACT};
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
const H_KEY_LIGHT: u32 = 0xE1CA_C6F7;
const H_FILL_LIGHT: u32 = 0xF530_2DC6;
const H_POINT_LIGHT: u32 = 0xEA46_0E64;
const H_LIGHT_COLOR: u32 = 0xF08E_B2F3;
const H_LIGHTS_TERRAIN: u32 = 0xE02A_BA1C;

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
    /// it has an orientation of its own (a <transform>, or an <ORIENTATION> with its <position>):
    /// a direction-point does, a world-point (a <position> alone) doesn't. A warp-trigger puts
    /// what it moves at such a point's whole transform, else at its position only (FUN_00200480,
    /// the point's +0xc interface asked for one)
    pub oriented: bool,
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
    pub transform: Mat4,
    pub half_fov: f32,
    pub near: f32,
    pub far: f32,
}

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

/// A directional light of the level (see the module notes).
#[derive(Clone, Copy, Debug)]
pub struct LevelLight {
    pub color: [f32; 3],
    /// the way the light shines (world)
    pub dir: Vec3,
    /// the key light (else a fill)
    pub key: bool,
    /// lights the terrain (else objects and characters)
    pub terrain: bool,
}

/// A point light (light-object h_ea460e64): its place, colour (0-1), the distance it reaches
/// (`range`, m) and its `falloff` (an enum: 3 or 4 on every one; unnamed in the schema).
pub struct LevelLamp {
    pub at: Vec3,
    pub color: [f32; 3],
    pub range: f32,
    pub falloff: i64,
    /// lights the terrain too
    pub terrain: bool,
}

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
    /// sound triggers: (sound id, signal, position)
    pub sounds: Vec<(u32, i64, Vec3)>,
    /// collision: the terrain blocks' surfaces (world frame) and the invisible blockers
    /// (<blocker h_051ac63e=surface object-instance=owner><transform>): surface, placement and
    /// the placed object it belongs to (its name; 0 for none). A breakable object's blocker goes
    /// with it (sdm_e34's missile rack h_128f3de2: blocker h_e1a3f88e, physics-id 4)
    pub terrain_collision: Vec<u32>,
    pub blockers: Vec<(u32, Mat4, u32)>,
    /// directional lights and the terrain's ambient (`ambient` is the objects')
    pub lights: Vec<LevelLight>,
    /// point lights (light-object h_ea460e64)
    pub lamps: Vec<LevelLamp>,
    pub terrain_ambient: [f32; 3],
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
            objects.push(Placement { tag: e.name, name: hash(e, h("name")), kind, archetype: game.object_meshes.get(&kind).copied(), transform: m,
                                     radar_contact, oriented });
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
        let sounds = all.iter().filter(|e| e.name == h("sound-trigger")).filter_map(|e| {
            let signal = e.child(0x031C_D9F5).and_then(|s| s.child(h("base"))).and_then(|b| b.attr(h("signal"))).and_then(|v| v.as_i64())?;
            let at = transform(&floats(e.child(h("transform"))))?.w_axis.truncate();
            Some((hash(e, h("sound-object")), signal, at))
        }).collect();
        let surface = |e: &Element| hash(e, H_PHYSICS_NAME);
        let terrain_collision = all.iter().filter(|e| e.name == h("blocks")).map(|e| surface(e))
            .filter(|&n| n != 0 && n != h("")).collect();
        let blockers = all.iter().filter(|e| e.name == h("blocker"))
            .filter_map(|e| Some((surface(e), transform(&floats(e.child(h("transform"))))?, hash(e, h("object-instance")))))
            .filter(|(n, _, _)| *n != 0 && *n != h("")).collect();
        let lights = all.iter().filter(|e| e.name == h("light-object")).filter_map(|e| {
            let kind = hash(e, H_TYPE);
            let key = kind == H_KEY_LIGHT;
            if !key && kind != H_FILL_LIGHT {
                return None;
            }
            let c = floats(e.child(H_LIGHT_COLOR));
            let m = transform(&floats(e.child(h("transform"))))?;
            Some(LevelLight {
                color: if c.len() >= 3 { [c[0], c[1], c[2]] } else { [1.0; 3] },
                dir: -m.z_axis.truncate().normalize_or_zero(),
                key,
                terrain: e.attr(H_LIGHTS_TERRAIN).and_then(|v| v.as_i64()) == Some(1),
            })
        }).collect();
        let lamps = all.iter().filter(|e| e.name == h("light-object") && hash(e, H_TYPE) == H_POINT_LIGHT).filter_map(|e| {
            let c = floats(e.child(H_LIGHT_COLOR));
            let num = |n: &str| e.attr(h(n)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
            Some(LevelLamp {
                at: transform(&floats(e.child(h("transform"))))?.w_axis.truncate(),
                color: if c.len() >= 3 { [c[0], c[1], c[2]] } else { [1.0; 3] },
                range: num("range").unwrap_or(10.0),
                falloff: num("falloff").map_or(3, |f| f as i64),
                terrain: e.attr(H_LIGHTS_TERRAIN).and_then(|v| v.as_i64()) == Some(1),
            })
        }).collect();
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
        Ok(Level { terrain, objects, sky, sky_at, background, fog, ambient, cameras, sounds, terrain_collision, blockers, lights, lamps, terrain_ambient,
                   terrain_half, terrain_light, buttons, triggers, radar_triggers, camera_objects, paths })
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
