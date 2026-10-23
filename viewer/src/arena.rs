//! A level as solid ground and walls, indexed by 2 m cell: the game's own collision surfaces
//! (objects-<lvl>.ipn, see bf::collision) of the terrain blocks, the invisible blockers and the
//! placed objects' parts (door leaves switch off while open). Each triangle keeps its material:
//! the world-material id, the slide flag and the indoor flag; and the placed object it's part
//! of, if any (what the crosshair's centre ray is on, `ray_object`; whether the floor is indoors,
//! `floor_kind`). A level without collision data falls back to the
//! drawn triangles (cut-outs left out). Used to stand characters on the ground (or on objects'
//! floors), push them out of walls, cast shots, settle pickups and keep the follow camera out of
//! walls (`hull_triangles`, `convex_meet`: its wedge, play_camera.rs).
//!
//! Liquids (lava, toxic rivers, water: placed objects of class h_04366a6a) have a flat
//! collision plane at their surface; it isn't solid: characters wade and sink through it to
//! the bed, and `liquid_at` tells how deep they are in what.
//!
//! Pickups (weapon / inventory objects and items) are placed at their spawn height in the level
//! file (the game lets them fall); `settle_pickups` drops them onto the surface below.
//!
//! Breakable objects (game objects with hit points, `Game::shootable`, #263: those with a debris
//! list, `Game::breakable`, break) are numbered in level object order: shots on their collision
//! find them (`ray_breakable`), their own triangles and their level blocker (`<blocker
//! object-instance>`) block until they're broken (`set_broken`), and the game objects they leave
//! behind (the missile rack's stand) are put in from the start, blocking only once they're
//! broken. One without a debris list dies (`set_dead`) and stays as it is.
//!
//! A spawn-trigger's template (its h_1299668b) isn't in the world: the trigger takes it out as it
//! links (FUN_000dfe90), so it adds nothing here. (The tutorial's Tex template put a surface
//! 1.7 m above the ground where his spawn stands him, #220.) Nor does any character placement
//! (`character-object`): a character's body is its own physics object, not part of the static
//! collision (#230; m09_a's squad started on their own placements' bodies, 0.8-1.3 m up, and
//! stepped off them, and e34's enemies left surfaces where they stood).
//!
//! The test tools' scenery (#185, play_testscenery.rs) joins the map's collision as it's placed
//! (`Arena::place`): a piece's own triangles, solid like a placed object's. The flat test floor
//! has no arena until then; placing a solid one gives it an arena of its flat ground
//! (`Arena::flat`, a plane at the floor's height everywhere, counted as terrain) with the pieces
//! on it.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// How finely a shot's path is looked along for a liquid's surface (m).
const LIQUID_STEP: f32 = 0.25;
/// How much farther (m) than the first face a ray meets a face met from its front may be and
/// still count as the one struck (`Arena::ray_material`): the twins' distances differ by their
/// rounding only. The demo's.
const FRONT_SLACK: f32 = 0.01;

use bevy::math::{Mat4, Vec2, Vec3};

use crate::bf::character::{Game, LiquidType, H_LIQUID_OBJECT, OBJECT_GAME};
use crate::bf::collision::{MATERIAL_ID, MATERIAL_SLIDE};
use crate::bf::hash::h;
use crate::bf::level::Level;
use crate::bf::weapon::WeaponModel;
use crate::level_scene::{visible, CUTOUT_TYPES};

const CELL: f32 = 2.0;
/// Surfaces at least this level (normal y) are floors; steeper ones are walls. The game counts
/// a contact as ground within 65 degrees of up (cos 65, set up at 0x30c440 in default.xbe).
const FLOOR_NORMAL: f32 = 0.422_618_26;
/// A terrain triangle whose normal's y is under this is taken as upright (a wall face, not
/// ground to fall down: `slope_below`): about 87 degrees. The demo's.
const SLOPE_UPRIGHT: f32 = 0.05;
/// Pickup element: weapon and inventory objects, and the level's item spawns (h_1d1d4f05).
const PICKUP_ITEM: u32 = 0x1D1D_4F05;

#[derive(Clone, Copy)]
struct Tri {
    v: [Vec3; 3],
    /// facing up (floors), or sideways for walls
    n: Vec3,
    terrain: bool,
    /// collision material: world-material id (low 5 bits), MATERIAL_SLIDE and MATERIAL_INDOOR
    material: u8,
    /// a door leaf's: the door's index (its triangles don't count while it's open)
    door: Option<u32>,
    /// a liquid's surface (index into `liquids`): not solid
    liquid: Option<u32>,
    /// a breakable object's (its index, see `breakable`) and whether it blocks once broken (what
    /// it leaves behind) or until then (the object itself)
    piece: Option<(u32, bool)>,
    /// the placed object it's part of (its index in `Level::objects`): its own parts' triangles
    /// and its level blocker's (`<blocker object-instance>`); None for the terrain, a blocker of
    /// no object and what a broken object leaves behind (`ray_object`)
    object: Option<u32>,
    /// one of a level's blockers (`<blocker physics-id ...>`, the invisible walls round its
    /// objects), which the follow camera's wedge passes through (`hull_triangles`)
    blocker: bool,
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
    /// breakable objects, in level object order: broken or not
    broken: Vec<AtomicBool>,
    /// breakable objects, in level object order: dead or not (their hit points run out: broken
    /// for one with a debris list, still standing for one without, #263)
    dead: Vec<AtomicBool>,
    /// level object index -> its breakable index
    breakable_of: HashMap<usize, u32>,
    /// the flat test floor's ground: a plane at this height under every point, terrain (an
    /// arena made for the flat floor when the test tools put scenery on it, `Arena::flat`)
    flat: Option<f32>,
}

/// The current map's arena. A new map installs its own (bf_play loads maps one after another
/// in one window), and the old one is freed once nothing holds it (it was leaked, several MB a
/// map: the test tools' level switch made that add up, #116).
static ARENA: RwLock<Option<Arc<Arena>>> = RwLock::new(None);

/// The current map's arena, if it has one (a handle: hold it no longer than the frame).
pub fn arena() -> Option<Arc<Arena>> {
    ARENA.read().unwrap_or_else(|e| e.into_inner()).clone()
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

/// How many rounds `convex_meet` takes before it calls the pair apart: FUN_00167c40 counts its
/// rounds from 1 and gives up past 100 (0x167dc0), a miss.
const GJK_ROUNDS: u32 = 100;

/// FUN_00167c40's closeness: a search vector (the simplex's point nearest the origin) of squared
/// length at or under this (0x3c0278, 0.0011: about 0.033 m) counts as a meeting. So a wedge that
/// comes within a few centimetres of a triangle can meet it (#275), but only when the search gets
/// that close before it finds a separating direction; nothing makes it a fixed margin.
const GJK_CLOSE_SQ: f32 = 0.0011;

/// Whether the convex hull of the points `a` (the follow camera's wedge, play_camera.rs) meets
/// the triangle `b`, as the game's FUN_00167c40 tests it (#275): GJK on the triangle less the hull
/// with the Johnson sub-algorithm (FUN_00166c60) for the simplex's point nearest the origin, in
/// f32 as the game. The search vector starts at the triangle's first corner less the hull's
/// first point; each round takes the triangle's support point against it and the hull's along it
/// (FUN_00167750, FUN_00167920); a new point that doesn't get past the origin is a miss, so is a
/// simplex the sub-algorithm can't resolve and a 101st round; four points round the origin are a
/// meeting, and so is a search vector within GJK_CLOSE_SQ. The points are taken relative to `a`'s
/// first, so the sums stay small where the world's coordinates are large (the game works in the
/// world's; the order of the arithmetic is otherwise the same).
pub fn convex_meet(a: &[Vec3], b: &[Vec3]) -> bool {
    let (Some(&o), Some(&b0)) = (a.first(), b.first()) else { return false };
    // the support points: the triangle's farthest along `d` (FUN_00167750) and the hull's
    // (FUN_00167920), the first of the farthest on a tie
    let far = |s: &[Vec3], d: Vec3| {
        let mut best = s[0] - o;
        let mut at = best.dot(d);
        for &p in &s[1..] {
            let q = p - o;
            if q.dot(d) > at {
                (best, at) = (q, q.dot(d));
            }
        }
        best
    };
    let mut gjk = Johnson::default();
    let mut v = b0 - o;
    let mut last = 0;
    for round in 1..=GJK_ROUNDS {
        let w = far(b, -v) - far(a, v);
        if w.dot(v) > 0.0 {
            return false;
        }
        match gjk.add(last, w) {
            Some(nearest) => v = nearest,
            None => return false,
        }
        if gjk.bits == 0xf {
            return true;
        }
        // the next free slot after the last, round the four
        loop {
            last = (last + 1) & 3;
            if gjk.bits & (1 << last) == 0 {
                break;
            }
        }
        if round == GJK_ROUNDS {
            return false;
        }
        if v.length_squared() <= GJK_CLOSE_SQ {
            return true;
        }
    }
    false
}

/// The simplex of up to four points of a GJK search and Johnson's sub-algorithm for its point
/// nearest the origin (FUN_00166c60, as in van den Bergen's GJK): the dot products of the points
/// (`dot`), the determinants of every subset (`det[subset][point]`), the points in use (`bits`).
#[derive(Default)]
struct Johnson {
    /// the points, by slot
    y: [Vec3; 4],
    /// y[i] . y[j]
    dot: [[f32; 4]; 4],
    /// the sub-algorithm's determinant of each subset (bit mask) for each of its points
    det: [[f32; 4]; 16],
    /// the slots of the simplex in use
    bits: u32,
}

impl Johnson {
    /// Puts `w` in slot `last` (not in use) and finds the smallest subset with it whose nearest
    /// point to the origin lies inside it (FUN_00166ad0's test), which becomes the simplex: that
    /// point (FUN_00166b30, the determinants' weights), or None when no subset passes.
    fn add(&mut self, last: usize, w: Vec3) -> Option<Vec3> {
        let last_bit = 1u32 << last;
        self.y[last] = w;
        for i in 0..4 {
            if self.bits & (1 << i) != 0 {
                let d = self.y[i].dot(w);
                self.dot[i][last] = d;
                self.dot[last][i] = d;
            }
        }
        self.dot[last][last] = w.dot(w);
        self.dets(last, last_bit);
        let mut s = self.bits;
        while s != 0 {
            if s & self.bits == s && self.valid(s | last_bit, last_bit) {
                self.bits = s | last_bit;
                return Some(self.point(self.bits));
            }
            s -= 1;
        }
        if self.valid(last_bit, last_bit) {
            self.bits = last_bit;
            return Some(w);
        }
        None
    }

    /// The determinants of the subsets that hold the new point `last` (the old ones stand).
    fn dets(&mut self, last: usize, last_bit: u32) {
        let (dp, det) = (&self.dot, &mut self.det);
        det[last_bit as usize][last] = 1.0;
        for j in 0..4 {
            let bj = 1u32 << j;
            if self.bits & bj == 0 {
                continue;
            }
            let s2 = (bj | last_bit) as usize;
            det[s2][j] = dp[last][last] - dp[last][j];
            det[s2][last] = dp[j][j] - dp[j][last];
            for k in 0..j {
                let bk = 1u32 << k;
                if self.bits & bk == 0 {
                    continue;
                }
                let s3 = (bk | s2 as u32) as usize;
                let (kj, kl, jl) = ((bk | bj) as usize, (bk | last_bit) as usize, s2);
                det[s3][k] = det[jl][j] * (dp[j][j] - dp[j][k]) + det[jl][last] * (dp[last][j] - dp[last][k]);
                det[s3][j] = det[kl][k] * (dp[k][k] - dp[k][j]) + det[kl][last] * (dp[last][k] - dp[last][j]);
                det[s3][last] = det[kj][k] * (dp[k][k] - dp[k][last]) + det[kj][j] * (dp[j][k] - dp[j][last]);
            }
        }
        if self.bits | last_bit == 0xf {
            det[15][0] = det[14][1] * (dp[1][1] - dp[1][0]) + det[14][2] * (dp[2][1] - dp[2][0]) + det[14][3] * (dp[3][1] - dp[3][0]);
            det[15][1] = det[13][0] * (dp[0][0] - dp[0][1]) + det[13][2] * (dp[2][0] - dp[2][1]) + det[13][3] * (dp[3][0] - dp[3][1]);
            det[15][2] = det[11][0] * (dp[0][0] - dp[0][2]) + det[11][1] * (dp[1][0] - dp[1][2]) + det[11][3] * (dp[3][0] - dp[3][2]);
            det[15][3] = det[7][0] * (dp[0][0] - dp[0][3]) + det[7][1] * (dp[1][0] - dp[1][3]) + det[7][2] * (dp[2][0] - dp[2][3]);
        }
    }

    /// FUN_00166ad0: whether subset `s`'s nearest point lies inside it: every one of its points'
    /// determinants above 0, and none of the other points in play (the simplex and the new one)
    /// with a positive determinant in `s` with it.
    fn valid(&self, s: u32, last_bit: u32) -> bool {
        (0..4).all(|i| {
            let bit = 1u32 << i;
            if (self.bits | last_bit) & bit == 0 {
                true
            } else if s & bit != 0 {
                self.det[s as usize][i] > 0.0
            } else {
                self.det[(s | bit) as usize][i] <= 0.0
            }
        })
    }

    /// FUN_00166b30: the point of subset `s` nearest the origin, its points weighted by their
    /// determinants.
    fn point(&self, s: u32) -> Vec3 {
        let (mut sum, mut p) = (0.0, Vec3::ZERO);
        for i in 0..4 {
            if s & (1 << i) != 0 {
                sum += self.det[s as usize][i];
                p += self.y[i] * self.det[s as usize][i];
            }
        }
        p / sum
    }
}

impl Arena {
    fn new(game: &Game, level: &Level) -> Arena {
        let mut a = Arena { cells: HashMap::new(), tris: vec![], starts: vec![], doors: vec![], liquids: vec![], broken: vec![], dead: vec![],
                            breakable_of: HashMap::new(), flat: None };
        // breakable objects (with hit points, #263), numbered in level object order (and by name,
        // for their blockers)
        let mut by_name: HashMap<u32, u32> = HashMap::new();
        for (i, o) in level.objects.iter().enumerate() {
            if o.archetype.is_some() && !is_pickup(o.tag) && game.shootable(o.kind).is_some() {
                let b = a.broken.len() as u32;
                a.broken.push(AtomicBool::new(false));
                a.dead.push(AtomicBool::new(false));
                a.breakable_of.insert(i, b);
                if o.name != 0 {
                    by_name.insert(o.name, b);
                }
            }
        }
        // the game's collision: terrain blocks (world frame) and invisible blockers
        let physics = !level.terrain_collision.is_empty();
        for &s in &level.terrain_collision {
            for (t, m) in game.collision(s) {
                a.add(t, true, None, m);
            }
        }
        // (every named object by name, for the blockers that belong to one)
        let object_of: HashMap<u32, u32> = level.objects.iter().enumerate().filter(|(_, o)| o.name != 0).map(|(i, o)| (o.name, i as u32)).collect();
        for (s, at, owner) in &level.blockers {
            let first = a.tris.len();
            for (t, m) in game.collision(*s) {
                a.add(t.map(|v| at.transform_point3(v)), false, None, m);
            }
            let piece = by_name.get(owner).map(|&b| (b, false));
            let object = object_of.get(owner).copied().filter(|_| *owner != 0);
            for t in &mut a.tris[first..] {
                t.piece = piece;
                t.object = object;
                t.blocker = true;
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
        // (per archetype: its parts on transform joints and hinges that its clips move; an
        // object of it is a door when an anim-trigger plays it once: level_scene `animates`)
        let mut animated: HashMap<u32, HashMap<u32, crate::level_scene::Animated>> = HashMap::new();
        let played = crate::level_scene::played_once(level);
        // (the spawn-triggers' templates: out of the world until spawned, FUN_000dfe90)
        let templates = level.spawn_templates();
        for (i, o) in level.objects.iter().enumerate() {
            if o.tag == h("start-point") {
                let p = o.transform.w_axis.truncate();
                let f = o.transform.z_axis.truncate();
                a.starts.push((p, (-f.x).atan2(-f.z)));
            }
            let Some(arch) = o.archetype else { continue };
            // (a character's body is its own physics object, not the level's collision: #230,
            // m09_a's squad stood on their own placements' bodies and stepped off them)
            if is_pickup(o.tag) || templates.contains(&o.name) || o.tag == h("character-object") {
                continue;
            }
            let tris = solid.entry(arch).or_insert_with(|| if physics { physics_tris(game, arch) } else { model_tris(game, arch) });
            // doors in level_scene's order: any object with a sliding leaf part, or with parts
            // its clips move that an anim-trigger plays once (`spawn_level`'s test)
            let has_leaf = WeaponModel::load(game, arch).is_ok_and(|m| m.parts.iter().any(|p| p.slide_axis.is_some()));
            let moving = animated.entry(arch).or_insert_with(|| crate::level_scene::animated_parts(game, arch));
            let door = (has_leaf || crate::level_scene::animates(moving, &played, o.name)).then(|| {
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
            let piece = a.breakable_of.get(&i).copied();
            for t in &mut a.tris[first..] {
                t.liquid = liquid;
                t.piece = piece.map(|b| (b, false));
                t.object = Some(i as u32);
            }
            // what a breakable one leaves behind (its debris list's game objects: the missile
            // rack's stand), blocking once it's broken
            if let Some(b) = piece {
                let remains: Vec<u32> = game.breakable(o.kind).map(|t| t.debris.iter().map(|d| d.kind)
                    .filter(|k| game.object_types.get(k).is_some_and(|t| t.object_type == OBJECT_GAME)).collect()).unwrap_or_default();
                for arch in remains.into_iter().filter_map(|k| game.object_meshes.get(&k).copied()) {
                    let first = a.tris.len();
                    let tris = if physics { physics_tris(game, arch) } else { model_tris(game, arch) };
                    for (t, _, m) in tris {
                        a.add(t.map(|v| o.transform.transform_point3(v)), false, None, m);
                    }
                    for t in &mut a.tris[first..] {
                        t.piece = Some((b, true));
                    }
                }
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
        self.tris.push(Tri { v, n, terrain, material, door, liquid: None, piece: None, object: None, blocker: false });
        let (lo, hi) = (v[0].min(v[1]).min(v[2]), v[0].max(v[1]).max(v[2]));
        let (c0, c1) = (cell(lo.x, lo.z), cell(hi.x, hi.z));
        for x in c0.0..=c1.0 {
            for z in c0.1..=c1.1 {
                self.cells.entry((x, z)).or_default().push(i);
            }
        }
    }

    /// Build the level's arena and make it the shared one (replacing the last map's).
    pub fn install(game: &Game, level: &Level) -> Arc<Arena> {
        let a = Arc::new(Arena::new(game, level));
        *ARENA.write().unwrap_or_else(|e| e.into_inner()) = Some(a.clone());
        a
    }

    /// No map: the flat test floor.
    pub fn uninstall() {
        *ARENA.write().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// The flat test floor as an arena: nothing but its ground, a plane at `ground` under every
    /// point (see `flat`).
    pub fn flat(ground: f32) -> Arena {
        Arena { cells: HashMap::new(), tris: vec![], starts: vec![], doors: vec![], liquids: vec![], broken: vec![], dead: vec![],
                breakable_of: HashMap::new(), flat: Some(ground) }
    }

    /// Whether this is the flat test floor's arena (`Arena::flat`): what the flat floor has of its
    /// own (its pillars, the boxes of the breakables the test tools put down) isn't in it.
    pub fn is_flat(&self) -> bool {
        self.flat.is_some()
    }

    /// A copy of this arena (its doors' and breakables' state as they are now).
    fn duplicate(&self) -> Arena {
        let flags = |v: &[AtomicBool]| v.iter().map(|b| AtomicBool::new(b.load(Ordering::Relaxed))).collect();
        Arena { cells: self.cells.clone(), tris: self.tris.clone(), starts: self.starts.clone(), doors: flags(&self.doors), liquids: self.liquids.clone(),
                broken: flags(&self.broken), dead: flags(&self.dead), breakable_of: self.breakable_of.clone(), flat: self.flat }
    }

    /// Puts a placed object's collision into the current map's arena (the test tools' scenery,
    /// #185): `tris` (corners in the object's own frame, `physics_tris`, and their material)
    /// placed by `at`, solid as a level's placed objects are, belonging to no level object. On
    /// the flat test floor (no arena) an arena of its ground at `ground` is made first. The shared
    /// arena is read-only, so a copy with the triangles added takes its place: whoever holds the
    /// old one this frame keeps it until they let it go.
    pub fn place(tris: &[([Vec3; 3], u8)], at: Mat4, ground: f32) -> Arc<Arena> {
        let mut slot = ARENA.write().unwrap_or_else(|e| e.into_inner());
        let mut a = slot.as_deref().map_or_else(|| Arena::flat(ground), Arena::duplicate);
        for (t, m) in tris {
            a.add(t.map(|v| at.transform_point3(v)), false, None, *m);
        }
        let a = Arc::new(a);
        *slot = Some(a.clone());
        a
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

    /// A triangle that blocks now (not an open door's leaf, nor a broken object's, nor the
    /// remains of one not broken yet).
    fn solid(&self, t: &Tri) -> bool {
        t.liquid.is_none() && t.door.is_none_or(|d| !self.doors[d as usize].load(Ordering::Relaxed))
            && t.piece.is_none_or(|(b, after)| self.broken[b as usize].load(Ordering::Relaxed) == after)
    }

    /// The breakable index of level object `object` (its place in `Level::objects`), if it has hit
    /// points (`Game::shootable`; whether it breaks is `Game::breakable`'s).
    pub fn breakable(&self, object: usize) -> Option<usize> {
        self.breakable_of.get(&object).map(|&b| b as usize)
    }

    /// Break breakable `b` (or mend it): its own triangles and blocker stop blocking, what it
    /// leaves behind starts.
    pub fn set_broken(&self, b: usize, broken: bool) {
        if let Some(x) = self.broken.get(b) {
            x.store(broken, Ordering::Relaxed);
        }
    }

    /// Whether breakable `b` is broken (the trigger runtime's DESTROYED from a level object,
    /// play_triggers.rs).
    pub fn is_broken(&self, b: usize) -> bool {
        self.broken.get(b).is_some_and(|x| x.load(Ordering::Relaxed))
    }

    /// Breakable `b`'s hit points have run out (play_scenery.rs): the trigger runtime sends its
    /// DESTROYED (FUN_00157260), whether it breaks or stays (#263).
    pub fn set_dead(&self, b: usize) {
        if let Some(x) = self.dead.get(b) {
            x.store(true, Ordering::Relaxed);
        }
    }

    /// Whether breakable `b` has died (`set_dead`).
    pub fn is_dead(&self, b: usize) -> bool {
        self.dead.get(b).is_some_and(|x| x.load(Ordering::Relaxed))
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

    /// The terrain's height at (x, z), if there is terrain there (the flat floor's ground
    /// everywhere).
    pub fn ground(&self, x: f32, z: f32) -> Option<f32> {
        self.near(x, z).iter().map(|&i| &self.tris[i as usize]).filter(|t| t.terrain)
            .filter_map(|t| Self::height_on(t, x, z)).chain(self.flat).reduce(f32::max)
    }

    /// The highest floor at (x, z) no higher than `top`: terrain or an object's surface no
    /// steeper than 65 degrees, with its normal.
    pub fn floor_below(&self, x: f32, z: f32, top: f32) -> Option<(f32, Vec3)> {
        self.floor_at(x, z, top).map(|f| (f.0, f.1))
    }

    /// Where there's no floor (`floor_below`), the terrain under (x, z) no higher than `top` that
    /// is too steep to stand on (steeper than 65 degrees, not upright), with its normal: the
    /// terrain is a height field, so a pit's wall or a cliff's face is still the ground there,
    /// which a character falls onto and slides down (#264: the tutorial's lava pit's north wall is
    /// 66 degrees, normal y 0.40, with no floor under it; take08's Tex runs off its rim in
    /// JUMP_FALL, MT 21, and comes down it into the lava). The highest such triangle.
    pub fn slope_below(&self, x: f32, z: f32, top: f32) -> Option<(f32, Vec3)> {
        let mut best: Option<(f32, Vec3)> = None;
        for &i in self.near(x, z) {
            let t = &self.tris[i as usize];
            if !t.terrain || t.n.y >= FLOOR_NORMAL || t.n.y < SLOPE_UPRIGHT || !self.solid(t) {
                continue;
            }
            if let Some(y) = Self::height_on(t, x, z).filter(|y| *y <= top) {
                if best.is_none_or(|(b, _)| y > b) {
                    best = Some((y, t.n));
                }
            }
        }
        best
    }

    /// The lowest floor between `low` and `top` under any of eight points `radius` m round (x, z)
    /// (each point's highest floor no higher than `top`), with its normal and material: what a
    /// body that wide rests on where the ground under its middle is too steep to stand on (a
    /// sliver of steep ground beside a floor, a rim it's half over: #264). The lowest, as the
    /// game's ground contact is the lowest-lying of its contacts (FUN_00153410).
    pub fn floor_around(&self, x: f32, z: f32, radius: f32, low: f32, top: f32) -> Option<(f32, Vec3, u8)> {
        (0..8).filter_map(|k| {
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            self.floor_at(x + radius * a.cos(), z + radius * a.sin(), top).filter(|f| f.0 >= low)
        }).reduce(|a, b| if b.0 < a.0 { b } else { a })
    }

    /// `floor_below` with the floor's collision material (world-material id | MATERIAL_SLIDE).
    /// The flat floor's ground is level, of material 0.
    pub fn floor_at(&self, x: f32, z: f32, top: f32) -> Option<(f32, Vec3, u8)> {
        self.floor_tri(x, z, top).map(|(y, t)| t.map_or((y, Vec3::Y, 0), |t| (y, t.n, t.material)))
    }

    /// The floor `floor_at` finds as a character's ground record keeps it (FUN_0012dab0): its
    /// collision material, whether it's the terrain's, and the level object it's part of (an
    /// index into `Level::objects`). The flat floor counts as terrain, of material 0.
    pub fn floor_kind(&self, x: f32, z: f32, top: f32) -> Option<(u8, bool, Option<u32>)> {
        self.floor_tri(x, z, top).map(|(_, t)| t.map_or((0, true, None), |t| (t.material, t.terrain, t.object)))
    }

    /// The highest floor at (x, z) no higher than `top` and its triangle (None: the flat floor).
    fn floor_tri(&self, x: f32, z: f32, top: f32) -> Option<(f32, Option<&Tri>)> {
        let mut best: Option<(f32, Option<&Tri>)> = self.flat.filter(|&g| g <= top).map(|g| (g, None));
        for &i in self.near(x, z) {
            let t = &self.tris[i as usize];
            if t.n.y < FLOOR_NORMAL || !self.solid(t) {
                continue;
            }
            if let Some(y) = Self::height_on(t, x, z).filter(|y| *y <= top) {
                if best.is_none_or(|(b, _)| y > b) {
                    best = Some((y, Some(t)));
                }
            }
        }
        best
    }

    /// How many triangles of each world-material id (MATERIAL_ID) the terrain has and the rest
    /// (the placed objects' and blockers'), by id, for the log (BF_SHOT_LOG: what a shot's hit
    /// takes its sounds and debris from, play.rs `world_hit`).
    pub fn material_counts(&self) -> Vec<(u8, usize, usize)> {
        let mut out: Vec<(u8, usize, usize)> = vec![];
        for t in &self.tris {
            let id = t.material & MATERIAL_ID;
            match out.iter_mut().find(|e| e.0 == id) {
                Some(e) => if t.terrain { e.1 += 1 } else { e.2 += 1 },
                None => out.push((id, t.terrain as usize, !t.terrain as usize)),
            }
        }
        out.sort();
        out
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
        self.push_out_sliding(p, radius, feet, head, step, None)
    }

    /// `push_out` for a body sliding down a steep terrain face of normal `face` (#264): the
    /// terrain's steep triangles that face the same way (their across normal within 90 degrees
    /// of the face's) are what it slides on, not walls to push it off; others (the far side of a
    /// gully, an object's wall) still push.
    pub fn push_out_sliding(&self, p: Vec2, radius: f32, feet: f32, head: f32, step: f32, face: Option<Vec3>) -> Vec2 {
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
                        if face.is_some_and(|f| t.terrain && t.n.x * f.x + t.n.z * f.z > 0.0) {
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
        self.ray_tri(origin, dir, max).map(|(t, _)| t)
    }

    /// `ray`, with the hit triangle's normal turned to face back along the ray (where a bullet
    /// hole lies, play_shots.rs); the flat floor's ground faces up.
    pub fn ray_normal(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Vec3)> {
        self.ray_tri(origin, dir, max).map(|(t, i)| {
            let Some(i) = i else { return (t, Vec3::Y) };
            let v = &self.tris[i].v;
            let n = (v[1] - v[0]).cross(v[2] - v[0]).normalize_or(Vec3::Y);
            (t, if n.dot(dir) > 0.0 { -n } else { n })
        })
    }

    /// `ray`, with the world-material id (the collision material's low bits, MATERIAL_ID) of the
    /// triangle met from its front: what a shot's hit takes its effect, debris and sounds from
    /// (FUN_0022cd80, FUN_0022b5b0: the hit record's +0x54). The terrain's triangles come in
    /// twins with the same corners: the one facing up by its winding ((v1 - v0) x (v2 - v0))
    /// carries the material and the flags, its back 0 and none (e34: all 15480 pairs DIRT 5 over
    /// 0, the slide flag on 10462 of the fronts; e40's 20278 the same), and the ray, testing
    /// both sides, met either. A face met from its front within FRONT_SLACK of the first face
    /// met is taken; else that first face (a lone triangle wound the other way). The flat
    /// floor's ground is material 0.
    pub fn ray_material(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, u8)> {
        let id = |i: Option<usize>| i.map_or(0, |i| self.tris[i].material & MATERIAL_ID);
        let first = self.ray_tri(origin, dir, max)?;
        let front = self.ray_tri_where(origin, dir, max, |t| (t.v[1] - t.v[0]).cross(t.v[2] - t.v[0]).dot(dir) < 0.0)
            .filter(|(t, _)| *t <= first.0 + FRONT_SLACK);
        Some(front.map_or((first.0, id(first.1)), |(t, i)| (t, id(i))))
    }

    /// `ray`, with the breakable object it meets first (its index, see `breakable`), if the
    /// nearest thing on the ray is one.
    pub fn ray_breakable(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Option<usize>)> {
        self.ray_tri(origin, dir, max).map(|(t, i)| (t, i.and_then(|i| self.tris[i].piece.filter(|p| !p.1).map(|p| p.0 as usize))))
    }

    /// `ray`, with the placed object it meets first (its index in `Level::objects`), if the
    /// nearest thing on the ray is part of one: what the crosshair's centre ray is on
    /// (play_reticle.rs).
    pub fn ray_object(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Option<usize>)> {
        self.ray_tri(origin, dir, max).map(|(t, i)| (t, i.and_then(|i| self.tris[i].object.map(|o| o as usize))))
    }

    /// Distance along a ray to the terrain alone (no objects or blockers), if within `max`: the
    /// sight test of the game's target search, whose world query (FUN_000df0a0 with the flags
    /// 0xfffffffa that FUN_002237c0 sets) takes the terrain and some dynamic objects but none of
    /// the level's placed geometry.
    pub fn ray_terrain(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<f32> {
        self.ray_tri_where(origin, dir, max, |t| t.terrain).map(|(t, _)| t)
    }

    /// The solid triangles that meet the convex hull of `points` (`convex_meet`), and the flat
    /// floor's ground under them where the hull reaches down to it (a triangle of it round the
    /// hull): the follow camera's wedge test (play_camera.rs), where FUN_00152a50 gathers the
    /// world's triangles round the wedge (its query mode 1: FUN_001516c0 keeps a leaf's triangle
    /// when FUN_00167c40 says it meets the wedge) from the static mesh and the collidable objects
    /// in their current pose, not characters or weapons (FUN_0006ef40's mask, the filter 0x6efb0:
    /// todo/126 answers.md Q8). The level's blockers aren't met: the takes' camera passes the
    /// hangar's on e34 (take17: 1.902 m at +0.061, where its blocker 1.2 m behind Tex would leave
    /// 0.4) and the fence's at the range spot; which class keeps them out isn't traced (one of the
    /// mask's types 15 / 19 / 24, or no collidable flag: a guess).
    pub fn hull_triangles(&self, points: &[Vec3]) -> Vec<[Vec3; 3]> {
        let Some(&first) = points.first() else { return vec![] };
        let (lo, hi) = points.iter().fold((first, first), |(lo, hi), &p| (lo.min(p), hi.max(p)));
        let mut out = vec![];
        if let Some(g) = self.flat.filter(|&g| lo.y <= g) {
            let size = (hi - lo).max_element() + 1.0;
            let corner = Vec3::new(lo.x - 1.0, g, lo.z - 1.0);
            let ground = [corner, corner + Vec3::X * size * 3.0, corner + Vec3::Z * size * 3.0];
            if convex_meet(points, &ground) {
                out.push(ground);
            }
        }
        let (c0, c1) = (cell(lo.x, lo.z), cell(hi.x, hi.z));
        let mut seen = HashSet::new();
        for x in c0.0..=c1.0 {
            for z in c0.1..=c1.1 {
                for &i in self.cells.get(&(x, z)).map_or(&[][..], |v| v.as_slice()) {
                    if !seen.insert(i) {
                        continue;
                    }
                    let t = &self.tris[i as usize];
                    if !self.solid(t) || t.blocker {
                        continue;
                    }
                    let (tlo, thi) = (t.v[0].min(t.v[1]).min(t.v[2]), t.v[0].max(t.v[1]).max(t.v[2]));
                    if tlo.cmpgt(hi).any() || thi.cmplt(lo).any() {
                        continue;
                    }
                    if convex_meet(points, &t.v) {
                        out.push(t.v);
                    }
                }
            }
        }
        out
    }

    /// `ray`, with the triangle hit (None: the flat floor's ground).
    fn ray_tri(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Option<usize>)> {
        self.ray_tri_where(origin, dir, max, |_| true)
    }

    /// `ray_tri` against the solid triangles `keep` takes, and the flat floor's ground (terrain,
    /// which every `keep` here takes) met from above, as the flat floor's own rays meet it
    /// (play.rs `ray_hit`).
    fn ray_tri_where(&self, origin: Vec3, dir: Vec3, max: f32, keep: impl Fn(&Tri) -> bool) -> Option<(f32, Option<usize>)> {
        let mut best = max;
        let mut hit = None;
        if let Some(g) = self.flat.filter(|_| dir.y < -1e-4) {
            let d = (g - origin.y) / dir.y;
            if d > 1e-3 && d < best {
                best = d;
                hit = Some(None);
            }
        }
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
                if !self.solid(tri) || !keep(tri) {
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
                    hit = Some(Some(i as usize));
                }
            }
            if t >= best || t >= max {
                break;
            }
            t = (t + step).min(max);
        }
        hit.map(|i| (best, i))
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
/// part), flagged if they belong to a part that moves as a door leaf (`leaf_part`), with their
/// material.
pub fn physics_tris(game: &Game, arch: u32) -> Vec<([Vec3; 3], bool, u8)> {
    let Ok(m) = WeaponModel::load(game, arch) else { return vec![] };
    let moving = crate::level_scene::animated_parts(game, arch);
    let mut out = vec![];
    for p in &m.parts {
        let Some(s) = p.physics else { continue };
        let place = Mat4::from_rotation_translation(p.rotation, p.offset);
        for (t, mat) in game.collision(s) {
            out.push((t.map(|v| place.transform_point3(v)), leaf_part(p, &moving), mat));
        }
    }
    out
}

/// An archetype's drawn triangles in its own frame (cut-out and undrawn materials left out),
/// for levels without collision data, flagged if they belong to a part that moves as a door leaf
/// (`leaf_part`).
fn model_tris(game: &Game, arch: u32) -> Vec<([Vec3; 3], bool, u8)> {
    let Ok(m) = WeaponModel::load(game, arch) else { return vec![] };
    let moving = crate::level_scene::animated_parts(game, arch);
    let mut out = vec![];
    for p in &m.parts {
        let place = Mat4::from_rotation_translation(p.rotation, p.offset);
        for g in p.geosets.iter().filter(|g| !CUTOUT_TYPES.contains(&game.material_type(g.material)) && visible(game, g)) {
            for t in g.indices.chunks_exact(3) {
                out.push(([0, 1, 2].map(|k| place.transform_point3(Vec3::from(g.positions[t[k] as usize]))), leaf_part(p, &moving), 0));
            }
        }
    }
    out
}

/// Whether a part moves as a door's leaf: a sliding one, or one its archetype's clips move on a
/// transform joint or a hinge (`moving`: level_scene `animated_parts`). Its triangles stop
/// blocking while its door is open (an object that isn't a door has no door to open).
fn leaf_part(p: &crate::bf::weapon::WeaponPart, moving: &HashMap<u32, crate::level_scene::Animated>) -> bool {
    p.slide_axis.is_some() || moving.contains_key(&p.name)
}
