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
//!  cameras     camera-object transforms (the level's flyby cameras)
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
//!  triggers    anim-trigger / router-trigger: the signals they act on (h_031cd9f5 > base or
//!              > anim > base: signal, h_f136d22d = how many times, 2147483647 = always; an
//!              anim's animation-archetype is the clip it plays), the triggers they pass signals
//!              on to (h_e91cf6a8 objects) and their objects (h_0b6b92ba: a router's are the
//!              buttons it listens to, an anim-trigger's the object it animates). A gate's
//!              panels reach its anim-triggers through a router (see `Level::button_opens`).

use bevy::math::{Mat4, Vec3};

use super::bxml::{Element, Value};
use super::character::{Game, Geoset, H_PHYSICS_NAME};
use super::hash::h;
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
}

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

/// A wall panel (world-button-object): its name, placement and the signal it sends when used.
pub struct Button {
    pub name: u32,
    /// its object type (h_ee4c83c9 for the gate panels)
    pub kind: u32,
    pub transform: Mat4,
    pub signal: i64,
}

/// An anim-trigger or router-trigger (see the module notes).
pub struct Trigger {
    /// anim-trigger or router-trigger
    pub tag: u32,
    pub name: u32,
    /// (signal, animation clip (anim-triggers), times it acts: 2147483647 = always)
    pub on: Vec<(i64, Option<u32>, i64)>,
    /// triggers it passes signals on to
    pub to: Vec<u32>,
    /// its objects (a router's buttons, an anim-trigger's animated object)
    pub objects: Vec<u32>,
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
    pub triggers: Vec<Trigger>,
}

fn floats(e: Option<&Element>) -> Vec<f32> {
    e.and_then(|e| e.text.as_ref()).map(|t| t.floats()).unwrap_or_default()
}

fn hash(e: &Element, name: u32) -> u32 {
    e.attr(name).and_then(|v| v.as_hash()).unwrap_or(0)
}

/// A trigger's object list (<h_e91cf6a8 objects="h_.. h_..">): the names in it (stored as one
/// List value or as the attribute repeated, one per name).
fn object_list(e: &Element, list: u32) -> Vec<u32> {
    let Some(l) = e.child(list) else { return vec![] };
    l.attrs.iter().filter(|(k, _)| *k == h("objects")).flat_map(|(_, v)| match v {
        Value::List(v) => v.iter().filter_map(|x| x.as_hash()).collect(),
        v => v.as_hash().into_iter().collect::<Vec<_>>(),
    }).filter(|&x| x != 0 && x != h("")).collect()
}

const H_WORLD_BUTTON: u32 = 0x16F2_2D4B;
const H_BUTTON_SIGNAL: u32 = 0xEAF8_A35B;
const H_SIGNALS: u32 = 0x031C_D9F5;
const H_SIGNAL_COUNT: u32 = 0xF136_D22D;
const H_SENDS_TO: u32 = 0xE91C_F6A8;
const H_TRIGGER_OBJECTS: u32 = 0x0B6B_92BA;

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
            if t.len() < 12 {
                let (o, p) = (floats(e.child(h("ORIENTATION"))), floats(e.child(h("position"))));
                if p.len() >= 3 {
                    t = if o.len() >= 9 { o[..9].to_vec() } else { vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] };
                    t.extend_from_slice(&p[..3]);
                }
            }
            let Some(m) = transform(&t) else { continue };
            let kind = hash(e, H_TYPE);
            objects.push(Placement { tag: e.name, name: hash(e, h("name")), kind, archetype: game.object_meshes.get(&kind).copied(), transform: m });
        }
        // sky
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
            })).collect();
        let triggers = all.iter().filter(|e| e.name == h("anim-trigger") || e.name == h("router-trigger")).map(|e| {
            let mut on = vec![];
            for s in e.child(H_SIGNALS).map(|s| s.children.as_slice()).unwrap_or_default() {
                // <base signal> or <anim animation-archetype><base signal></anim>
                let (base, clip) = if s.name == h("anim") { (s.child(h("base")), Some(hash(s, h("animation-archetype")))) } else { (Some(s), None) };
                let Some(b) = base else { continue };
                let Some(signal) = b.attr(h("signal")).and_then(|v| v.as_i64()) else { continue };
                on.push((signal, clip, b.attr(H_SIGNAL_COUNT).and_then(|v| v.as_i64()).unwrap_or(i64::from(i32::MAX))));
            }
            Trigger { tag: e.name, name: hash(e, h("name")), on, to: object_list(e, H_SENDS_TO), objects: object_list(e, H_TRIGGER_OBJECTS) }
        }).collect();
        Ok(Level { terrain, objects, sky, sky_at, background, fog, ambient, cameras, sounds, terrain_collision, blockers, lights, lamps, terrain_ambient,
                   terrain_half, terrain_light, buttons, triggers })
    }

    /// What using a button does to the level's animated objects: (object name, clip, times)
    /// for each anim-trigger its signal reaches (the routers listening to the button, and on
    /// through the triggers they send to).
    pub fn button_opens(&self, button: &Button) -> Vec<(u32, u32, i64)> {
        let mut todo: Vec<&Trigger> = self.triggers.iter().filter(|t| t.tag == h("router-trigger") && t.objects.contains(&button.name)).collect();
        let mut seen = vec![];
        let mut out = vec![];
        while let Some(t) = todo.pop() {
            if seen.contains(&t.name) {
                continue;
            }
            seen.push(t.name);
            for &(signal, clip, times) in &t.on {
                if let (true, Some(clip)) = (signal == button.signal, clip) {
                    out.extend(t.objects.iter().map(|&o| (o, clip, times)));
                }
            }
            todo.extend(self.triggers.iter().filter(|n| t.to.contains(&n.name)));
        }
        out
    }
}
