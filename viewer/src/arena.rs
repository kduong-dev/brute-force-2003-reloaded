//! A level as solid ground and walls, indexed by 2 m cell: the game's own collision surfaces
//! (objects-<lvl>.ipn, see bf::collision) of the terrain blocks, the invisible blockers and the
//! placed objects' parts (door leaves switch off while open). Each triangle keeps its material:
//! the world-material id and the slide flag. A level without collision data falls back to the
//! drawn triangles (cut-outs left out). Used to stand characters on the ground (or on objects'
//! floors), push them out of walls, cast shots and settle pickups.
//!
//! Liquids (lava, toxic rivers, water: placed objects of class h_04366a6a) have a flat
//! collision plane at their surface; it isn't solid: characters wade and sink through it to
//! the bed, and `liquid_at` tells how deep they are in what.
//!
//! Pickups (weapon / inventory objects and items) are placed at their spawn height in the level
//! file (the game lets them fall); `settle_pickups` drops them onto the surface below.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

/// How finely a shot's path is looked along for a liquid's surface (m).
const LIQUID_STEP: f32 = 0.25;

use bevy::math::{Mat4, Vec2, Vec3};

use crate::bf::character::{Game, LiquidType, H_LIQUID_OBJECT};
use crate::bf::collision::MATERIAL_SLIDE;
use crate::bf::hash::h;
use crate::bf::level::Level;
use crate::bf::weapon::WeaponModel;
use crate::level_scene::{visible, CUTOUT_TYPES};

const CELL: f32 = 2.0;
/// Surfaces at least this level (normal y) are floors; steeper ones are walls. The game counts
/// a contact as ground within 65 degrees of up (cos 65, set up at 0x30c440 in default.xbe).
const FLOOR_NORMAL: f32 = 0.422_618_26;
/// Pickup element: weapon and inventory objects, and the level's item spawns (h_1d1d4f05).
const PICKUP_ITEM: u32 = 0x1D1D_4F05;

#[derive(Clone, Copy)]
struct Tri {
    v: [Vec3; 3],
    /// facing up (floors), or sideways for walls
    n: Vec3,
    terrain: bool,
    /// collision material: world-material id (low 5 bits) and MATERIAL_SLIDE
    material: u8,
    /// a door leaf's: the door's index (its triangles don't count while it's open)
    door: Option<u32>,
    /// a liquid's surface (index into `liquids`): not solid
    liquid: Option<u32>,
}

pub struct Arena {
    cells: HashMap<(i32, i32), Vec<u32>>,
    tris: Vec<Tri>,
    /// the level's start points (position, facing yaw)
    pub starts: Vec<(Vec3, f32)>,
    /// doors in level object order (as `level_scene::spawn_level` returns them): open or not
    doors: Vec<AtomicBool>,
    /// the placed liquids' types
    liquids: Vec<LiquidType>,
}

/// The current map's arena. A new map installs its own (bf_play loads maps one after another
/// in one window); an old one is leaked, a few MB a map.
static ARENA: RwLock<Option<&'static Arena>> = RwLock::new(None);

pub fn arena() -> Option<&'static Arena> {
    *ARENA.read().unwrap_or_else(|e| e.into_inner())
}

fn cell(x: f32, z: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (z / CELL).floor() as i32)
}

fn is_pickup(tag: u32) -> bool {
    tag == h("inventory-object") || tag == h("weapon-object") || tag == PICKUP_ITEM
}

/// Closest point on triangle `t` to `p` (Ericson, Real-Time Collision Detection 5.1.5).
fn closest_on_tri(p: Vec3, t: &[Vec3; 3]) -> Vec3 {
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 { return a; }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 { return b; }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 { return a + ab * (d1 / (d1 - d3)); }
    let cp = p - c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 { return c; }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 { return a + ac * (d2 / (d2 - d6)); }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    a + ab * (vb * denom) + ac * (vc * denom)
}

impl Arena {
    fn new(game: &Game, level: &Level) -> Arena {
        let mut a = Arena { cells: HashMap::new(), tris: vec![], starts: vec![], doors: vec![], liquids: vec![] };
        // the game's collision: terrain blocks (world frame) and invisible blockers
        let physics = !level.terrain_collision.is_empty();
        for &s in &level.terrain_collision {
            for (t, m) in game.collision(s) {
                a.add(t, true, None, m);
            }
        }
        for (s, at) in &level.blockers {
            for (t, m) in game.collision(*s) {
                a.add(t.map(|v| at.transform_point3(v)), false, None, m);
            }
        }
        if !physics {
            if let Some((g, _)) = level.terrain.first() {
                for t in g.indices.chunks_exact(3) {
                    a.add([0, 1, 2].map(|k| Vec3::from(g.positions[t[k] as usize])), true, None, 0);
                }
            }
        }
        let mut solid: HashMap<u32, Vec<([Vec3; 3], bool, u8)>> = HashMap::new();
        for o in &level.objects {
            if o.tag == h("start-point") {
                let p = o.transform.w_axis.truncate();
                let f = o.transform.z_axis.truncate();
                a.starts.push((p, (-f.x).atan2(-f.z)));
            }
            let Some(arch) = o.archetype else { continue };
            if is_pickup(o.tag) {
                continue;
            }
            let tris = solid.entry(arch).or_insert_with(|| if physics { physics_tris(game, arch) } else { model_tris(game, arch) });
            // doors in level_scene's order: any object with a sliding leaf part
            let has_leaf = WeaponModel::load(game, arch).is_ok_and(|m| m.parts.iter().any(|p| p.slide_axis.is_some()));
            let door = has_leaf.then(|| {
                a.doors.push(AtomicBool::new(false));
                a.doors.len() as u32 - 1
            });
            let liquid = (o.tag == H_LIQUID_OBJECT).then(|| {
                a.liquids.push(game.liquids.get(&o.kind).copied().unwrap_or_default());
                a.liquids.len() as u32 - 1
            });
            let first = a.tris.len();
            for (t, leaf, m) in tris.iter() {
                let w = t.map(|v| o.transform.transform_point3(v));
                a.add(w, false, door.filter(|_| *leaf), *m);
            }
            for t in &mut a.tris[first..] {
                t.liquid = liquid;
            }
            if liquid.is_some() && std::env::var("BF_LIQUID_LOG").is_ok() {
                let ys = a.tris[first..].iter().flat_map(|t| t.v.map(|v| v.y));
                let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(l, h), y| (l.min(y), h.max(y)));
                eprintln!("liquid collision h_{:08x} at {:.1}: {} triangles, y {lo:.2} .. {hi:.2} ({})", o.kind, o.transform.w_axis.truncate(),
                          a.tris.len() - first, if physics { "physics mesh" } else { "model" });
            }
        }
        a
    }

    fn add(&mut self, v: [Vec3; 3], terrain: bool, door: Option<u32>, material: u8) {
        let n = (v[1] - v[0]).cross(v[2] - v[0]);
        if n.length_squared() < 1e-10 {
            return;
        }
        // facing up: collision triangles' winding varies; floors are what's under the feet
        let n = n.normalize();
        let n = if n.y < 0.0 { -n } else { n };
        let i = self.tris.len() as u32;
        self.tris.push(Tri { v, n, terrain, material, door, liquid: None });
        let (lo, hi) = (v[0].min(v[1]).min(v[2]), v[0].max(v[1]).max(v[2]));
        let (c0, c1) = (cell(lo.x, lo.z), cell(hi.x, hi.z));
        for x in c0.0..=c1.0 {
            for z in c0.1..=c1.1 {
                self.cells.entry((x, z)).or_default().push(i);
            }
        }
    }

    /// Build the level's arena and make it the shared one (replacing the last map's).
    pub fn install(game: &Game, level: &Level) -> &'static Arena {
        let a: &'static Arena = Box::leak(Box::new(Arena::new(game, level)));
        *ARENA.write().unwrap_or_else(|e| e.into_inner()) = Some(a);
        a
    }

    /// No map: the flat test floor.
    pub fn uninstall() {
        *ARENA.write().unwrap_or_else(|e| e.into_inner()) = None;
    }

    fn near(&self, x: f32, z: f32) -> &[u32] {
        self.cells.get(&cell(x, z)).map_or(&[], |v| v.as_slice())
    }

    /// Open or close door `i` (its leaves stop or start blocking).
    pub fn set_door_open(&self, i: usize, open: bool) {
        if let Some(d) = self.doors.get(i) {
            d.store(open, Ordering::Relaxed);
        }
    }

    /// A triangle that blocks now (not an open door's leaf).
    fn solid(&self, t: &Tri) -> bool {
        t.liquid.is_none() && t.door.is_none_or(|d| !self.doors[d as usize].load(Ordering::Relaxed))
    }

    /// The liquid at (x, z) whose surface is above `feet`: its surface height and type.
    pub fn liquid_at(&self, x: f32, z: f32, feet: f32) -> Option<(f32, LiquidType)> {
        let mut best: Option<(f32, LiquidType)> = None;
        for &i in self.near(x, z) {
            let t = &self.tris[i as usize];
            let Some(l) = t.liquid else { continue };
            if let Some(y) = Self::height_on(t, x, z).filter(|y| *y > feet) {
                if best.is_none_or(|(b, _)| y > b) {
                    best = Some((y, self.liquids[l as usize]));
                }
            }
        }
        best
    }

    /// Where the line from `from` to `to` first goes into a liquid (through its surface, from
    /// above), and the liquid: sampled every LIQUID_STEP m.
    pub fn liquid_crossing(&self, from: Vec3, to: Vec3) -> Option<(Vec3, LiquidType)> {
        let length = from.distance(to);
        let dir = (to - from).normalize_or_zero();
        if self.liquid_at(from.x, from.z, from.y).is_some() {
            return None;
        }
        let steps = (length / LIQUID_STEP).ceil() as usize;
        (1..=steps).find_map(|i| {
            let p = from + dir * (i as f32 * LIQUID_STEP).min(length);
            self.liquid_at(p.x, p.z, p.y).map(|(y, l)| (Vec3::new(p.x, y, p.z), l))
        })
    }

    /// Height of triangle `t` at (x, z), if (x, z) is over it.
    fn height_on(t: &Tri, x: f32, z: f32) -> Option<f32> {
        let p = Vec2::new(x, z);
        let [a, b, c] = t.v.map(|v| Vec2::new(v.x, v.z));
        let d = (b - a).perp_dot(c - a);
        if d.abs() < 1e-8 {
            return None;
        }
        let u = (p - a).perp_dot(c - a) / d;
        let v = (b - a).perp_dot(p - a) / d;
        (u >= -1e-4 && v >= -1e-4 && u + v <= 1.0001).then(|| t.v[0].y + (t.v[1].y - t.v[0].y) * u + (t.v[2].y - t.v[0].y) * v)
    }

    /// The terrain's height at (x, z), if there is terrain there.
    pub fn ground(&self, x: f32, z: f32) -> Option<f32> {
        self.near(x, z).iter().map(|&i| &self.tris[i as usize]).filter(|t| t.terrain)
            .filter_map(|t| Self::height_on(t, x, z)).reduce(f32::max)
    }

    /// The highest floor at (x, z) no higher than `top`: terrain or an object's surface no
    /// steeper than 65 degrees, with its normal.
    pub fn floor_below(&self, x: f32, z: f32, top: f32) -> Option<(f32, Vec3)> {
        self.floor_at(x, z, top).map(|f| (f.0, f.1))
    }

    /// `floor_below` with the floor's collision material (world-material id | MATERIAL_SLIDE).
    pub fn floor_at(&self, x: f32, z: f32, top: f32) -> Option<(f32, Vec3, u8)> {
        let mut best: Option<(f32, Vec3, u8)> = None;
        for &i in self.near(x, z) {
            let t = &self.tris[i as usize];
            if t.n.y < FLOOR_NORMAL || !self.solid(t) {
                continue;
            }
            if let Some(y) = Self::height_on(t, x, z).filter(|y| *y <= top) {
                if best.is_none_or(|(b, _, _)| y > b) {
                    best = Some((y, t.n, t.material));
                }
            }
        }
        best
    }

    /// (triangles, of them terrain, slide-flagged, door leaves), for the log.
    pub fn stats(&self) -> (usize, usize, usize, usize) {
        let n = |f: &dyn Fn(&Tri) -> bool| self.tris.iter().filter(|t| f(t)).count();
        (self.tris.len(), n(&|t| t.terrain), n(&|t| Self::slides(t.material)), n(&|t| t.door.is_some()))
    }

    /// Whether a floor material is one characters slide down (when steep enough).
    pub fn slides(material: u8) -> bool {
        material & MATERIAL_SLIDE != 0
    }

    /// Push a body (radius `radius`, standing from `feet` to `head`) out of the walls: spheres
    /// at a few heights against the steep triangles near it. Returns the corrected (x, z).
    pub fn push_out(&self, p: Vec2, radius: f32, feet: f32, head: f32, step: f32) -> Vec2 {
        let mut p = p;
        let lo = feet + step;
        let n = 3;
        for _ in 0..2 {
            let mut seen = HashSet::new();
            for dx in -1..=1 {
                for dz in -1..=1 {
                    for &i in self.near(p.x + dx as f32 * CELL, p.y + dz as f32 * CELL) {
                        if !seen.insert(i) {
                            continue;
                        }
                        let t = &self.tris[i as usize];
                        if t.n.y.abs() >= FLOOR_NORMAL || !self.solid(t) {
                            continue;
                        }
                        let ty = (t.v[0].y.min(t.v[1].y).min(t.v[2].y), t.v[0].y.max(t.v[1].y).max(t.v[2].y));
                        if ty.1 < lo || ty.0 > head {
                            continue;
                        }
                        for k in 0..n {
                            let y = lo + (head - lo) * (k as f32 + 0.5) / n as f32;
                            let c = Vec3::new(p.x, y, p.y);
                            let q = closest_on_tri(c, &t.v);
                            let d = Vec2::new(c.x - q.x, c.z - q.z);
                            if (q.y - y).abs() < radius && d.length() < radius {
                                let dir = if d.length() > 1e-4 { d.normalize() } else { Vec2::new(t.n.x, t.n.z).normalize_or(Vec2::X) };
                                p += dir * (radius - d.length());
                            }
                        }
                    }
                }
            }
        }
        p
    }

    /// Distance along a ray to the terrain or an object, if within `max`.
    pub fn ray(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<f32> {
        let mut best = max;
        let mut hit = false;
        let mut seen = HashSet::new();
        let flat = Vec2::new(dir.x, dir.z).length();
        let step = if flat > 1e-4 { (CELL * 0.5 / flat).min(max) } else { max };
        let mut t = 0.0;
        loop {
            let p = origin + dir * t;
            for &i in self.near(p.x, p.z) {
                if !seen.insert(i) {
                    continue;
                }
                let tri = &self.tris[i as usize];
                if !self.solid(tri) {
                    continue;
                }
                // Moller-Trumbore, both sides
                let (e1, e2) = (tri.v[1] - tri.v[0], tri.v[2] - tri.v[0]);
                let pv = dir.cross(e2);
                let det = e1.dot(pv);
                if det.abs() < 1e-9 {
                    continue;
                }
                let tv = origin - tri.v[0];
                let u = tv.dot(pv) / det;
                if !(0.0..=1.0).contains(&u) {
                    continue;
                }
                let qv = tv.cross(e1);
                let v = dir.dot(qv) / det;
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }
                let d = e2.dot(qv) / det;
                if d > 1e-3 && d < best {
                    best = d;
                    hit = true;
                }
            }
            if t >= best || t >= max {
                break;
            }
            t = (t + step).min(max);
        }
        hit.then_some(best)
    }

    /// Drop the level's pickups onto the floor below them (their mesh's bottom on it).
    pub fn settle_pickups(&self, game: &Game, level: &mut Level) {
        let mut bottoms: HashMap<u32, f32> = HashMap::new();
        for o in level.objects.iter_mut().filter(|o| is_pickup(o.tag)) {
            let Some(arch) = o.archetype else { continue };
            let bottom = *bottoms.entry(arch).or_insert_with(|| {
                model_tris(game, arch).iter().flat_map(|(t, _, _)| t).map(|v| v.y).fold(f32::MAX, f32::min)
            });
            if bottom == f32::MAX {
                continue;
            }
            let at = o.transform.w_axis;
            if let Some((y, _)) = self.floor_below(at.x, at.z, at.y + 0.5) {
                o.transform.w_axis.y = y - bottom * o.transform.y_axis.y;
            }
        }
    }
}

/// An archetype's collision triangles in its own frame (each part's surface, placed as the
/// part), flagged if they belong to a sliding door leaf, with their material.
pub(crate) fn physics_tris(game: &Game, arch: u32) -> Vec<([Vec3; 3], bool, u8)> {
    let Ok(m) = WeaponModel::load(game, arch) else { return vec![] };
    let mut out = vec![];
    for p in &m.parts {
        let Some(s) = p.physics else { continue };
        let place = Mat4::from_rotation_translation(p.rotation, p.offset);
        for (t, mat) in game.collision(s) {
            out.push((t.map(|v| place.transform_point3(v)), p.slide_axis.is_some(), mat));
        }
    }
    out
}

/// An archetype's drawn triangles in its own frame (cut-out and undrawn materials left out),
/// for levels without collision data, flagged if they belong to a sliding door leaf.
fn model_tris(game: &Game, arch: u32) -> Vec<([Vec3; 3], bool, u8)> {
    let Ok(m) = WeaponModel::load(game, arch) else { return vec![] };
    let mut out = vec![];
    for p in &m.parts {
        let place = Mat4::from_rotation_translation(p.rotation, p.offset);
        for g in p.geosets.iter().filter(|g| !CUTOUT_TYPES.contains(&game.material_type(g.material)) && visible(game, g)) {
            for t in g.indices.chunks_exact(3) {
                out.push(([0, 1, 2].map(|k| place.transform_point3(Vec3::from(g.positions[t[k] as usize]))), p.slide_axis.is_some(), 0));
            }
        }
    }
    out
}
