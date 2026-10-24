//! A level as solid ground and walls, indexed by 2 m cell: the game's own collision surfaces
//! (objects-<lvl>.ipn, see bf::collision) of the terrain blocks, the invisible blockers and the
//! placed objects' parts (door leaves switch off while open). Each triangle keeps its material:
//! the world-material id, the slide flag and the indoor flag; and the placed object it's part
//! of, if any (what the crosshair's centre ray is on, `ray_object`; whether the floor is indoors,
//! `floor_kind`). A level without collision data falls back to the
//! drawn triangles (cut-outs left out). Used to stand characters on the ground (or on objects'
//! floors), sweep their moves against the walls and take them out of a rock (`move_body`,
//! `out_of_rock`, #285), push other bodies out of walls, cast shots, settle pickups and keep the
//! follow camera out of walls (`hull_triangles`, `convex_meet`: its wedge, play_camera.rs).
//!
//! A triangle faces the way its winding does (`Tri::front`), and the game meets it from that side
//! only; most of the data's come twinned, so they block both ways (`BodyShape`, #285), but for a
//! rock's faces that face into it (`Tri::inner`), which are neither floors nor walls.
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
    /// the way it faces by its winding, (v1 - v0) x (v2 - v0) (unit): its front, the one side the
    /// game's sweep meets it from (FUN_001516c0 makes the normal so, FUN_00152d20 leaves out a
    /// triangle whose plane has the body behind it). A lone triangle of Ipion's data comes with a
    /// twin facing the other way (`BodyShape`), so most faces block from both sides
    front: Vec3,
    /// met from either side whatever its winding: the drawn triangles of a level without
    /// collision data (`model_tris`), whose winding is the renderer's
    two_sided: bool,
    /// the second of a twin pair (the same corners wound the other way, `BodyShape`): left out
    /// where crossings of a body's surface are counted (`Arena::inside`), so a face counts once
    duplicate: bool,
    /// facing into its own body's solid (`Arena::new_body`, #285): the twin of a rock's face that
    /// faces in, as the twin of its underside faces up inside it. Neither a floor nor a wall: the
    /// game's sweep never lets a body into a rock to meet one from its front (FUN_00153810), and
    /// the demo's met one put a body in a rock (stood on, at a rock's foot) or held it there
    inner: bool,
    /// the twin facing down of a floor that is stood on (its twin faces up and isn't `inner`):
    /// it faces into the ground under the floor. Its plane is tested at the body's middle, as a
    /// ceiling's (`WallSection::base`, the demo's rule)
    floor_back: bool,
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
    /// the physics body it stands for (`Arena::ray_shot`): one per terrain block, per placed
    /// object, per blocker, per broken object's remains and per piece the test tools put down.
    /// The game's ray queries report a body once (FUN_000dd8e0 fills one result per body), and a
    /// bolt that passes through an object leaves that body out of its next sweep (FUN_00225440's
    /// +0x40)
    body: u32,
}

/// What a shot's ray met in the world (`Arena::ray_shot`): how far along, the struck triangle's
/// normal (facing back along the ray), its world material (`ray_material`'s), whether it's the
/// terrain (the static world: the ray result's kind 1, FUN_000dd8e0 for a body of PID 2,
/// TERRAIN) or an object's body (kind 2, PID 0: every placed object's parts, a broken object's
/// remains, the test tools' pieces), its body (`Tri::body`) and the breakable object it is, if
/// it's one.
#[derive(Clone, Copy, Debug)]
pub struct ShotHit {
    pub t: f32,
    pub normal: Vec3,
    pub material: u8,
    pub terrain: bool,
    pub body: u32,
    pub breakable: Option<usize>,
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
    /// the bodies numbered so far (`Tri::body`); the flat floor's ground is body FLAT_BODY
    bodies: u32,
    /// each body's triangles (a range of `tris`: a body's are added together, `new_body`)
    spans: Vec<(u32, u32)>,
    /// each body: whether its faces close round a solid (`shell`), so that a point can be inside
    /// it (`inside`): no edge left open, a twin pair counted once, and no door leaf or liquid
    /// among them (part of it would come and go). #285: e34's, e40's and the tutorial's rocks
    solid_shell: Vec<bool>,
}

/// The flat test floor's ground's body (`Tri::body`, `Arena::ray_shot`): numbered past any a map
/// would use.
pub const FLAT_BODY: u32 = u32::MAX;

/// How many times a character's move is swept on after it meets a wall (`Arena::move_body`):
/// FUN_00153810's passes (its context's +0xb4: 3 for a character, set at 0x12d1a7; 4 under the
/// session record's +0x14 bit 1, which isn't identified). What's left after the last is dropped.
const SWEEP_PASSES: usize = 3;
/// A move goes into a contact when its direction's dot product with the contact's normal is
/// under this (FUN_00152d20: the move's direction normalised by FUN_00039410, against the
/// contact's +0x30, with 0x3c0280, -0.01): a wall it grazes or leaves doesn't hold it.
const SWEEP_ENTERS: f32 = -0.01;
/// How far short of a wall (m) a swept body stops, so the next pass starts clear of it: the
/// demo's. (The game's stop is coarser: FUN_00152f20 steps the move in halves, its context's
/// +0xbc 2, each half tested from the pass's start, and keeps the last free one.)
const SWEEP_SKIN: f32 = 0.002;
/// Within this (m) beyond its radius a body touches a wall at a pass's start (`move_body`), so
/// the wall it stopped at last pass (SWEEP_SKIN short) still holds the rest of the move: the
/// demo's.
const SWEEP_TOUCH: f32 = 0.004;
/// A triangle whose normal's y, by its winding, is at or under this faces down: a ceiling, cos
/// 130 degrees (0x46d2f4, set up from the double 0x3a5550 at 0x30c450); between it and
/// FLOOR_NORMAL a triangle is steep (FUN_00152d20, 0x152d62-0x152d6f). The game tests a steep
/// triangle's plane at the body's base and any other's at its middle (`WallSection::base`).
/// The demo also tests the twin facing down of a floor that's stood on (`Tri::floor_back`) at
/// the middle: by the game's split that twin of a 50-65 degree slope is steep, and where the
/// slope eases ahead its plane is over the feet of a body walking up to the junction, so the
/// sweep met it there and held the body (#285's review: a 60-degree slope easing to 53, held at
/// the junction less its radius). The middle is over that plane. A rock's underside, whose twin
/// facing up is inside the rock (`Tri::inner`), keeps the game's split.
const CEILING_NORMAL: f32 = -0.642_787_6;
/// How far in front of a wall's plane (m) the body's point must be for the wall to meet it
/// (`WallSection::faces`): the demo's. The game asks only that it isn't behind (FUN_00152d20),
/// but its response leaves a body 1 mm off the ground it stands on (FUN_00151170 adds 0.001 m
/// along a ground-like contact's normal), behind the twin of that ground facing down into it.
/// The demo stands the feet on the ground's plane, where without the margin the rounding decided
/// whether that twin (steep by the game's split on a slope over 50 degrees) was met.
const PLANE_MARGIN: f32 = 0.01;
/// How far in front of a face (m) `Arena::new_body` asks whether a point is in the face's own
/// solid (`Tri::inner`): the demo's, well clear of the rounding of a rock's corners (welded to
/// the millimetre, `shell`) and well inside its thinnest part.
const INNER_OFFSET: f32 = 0.005;
/// Where on a face (barycentric weights of its corners) `Arena::new_body` asks it (`Tri::inner`):
/// its middle and two points off it, so one vertical line that runs down another face's edge
/// (crossings counted twice, `inside`) doesn't decide alone. The demo's.
const INNER_PROBES: [[f32; 3]; 3] = [[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0], [0.6, 0.2, 0.2], [0.2, 0.6, 0.2]];
/// How far (m) a step up may take a body into a wall beside it (`Arena::clear`): the demo's. The
/// sweep leaves a body touching a wall it walks along, and where the wall leans over the slope a
/// rise of a few centimetres brings its part over the step inside the radius: without slack the
/// climb was refused at each step (#285's test, sdm_e34's outcrop h_f5270370 at (31.0, 45.4): held
/// at a 2 cm rise). The next frame's sweep pushes the overlap out (`move_body`).
const CLIMB_SLACK: f32 = 0.05;
/// How many ways out of a rock `Arena::escape` tries, evenly round: the demo's.
const ESCAPE_WAYS: usize = 32;
/// How far (m) beyond its radius past a rock's surface `Arena::escape` puts a body: the demo's.
const ESCAPE_CLEAR: f32 = 0.02;

/// Where a body taken out of a rock goes (`Arena::escape`): its place (x, z) and the floor there
/// (the highest under its head; None for none), which its feet go on.
#[derive(Debug, PartialEq)]
pub struct Moved {
    pub at: Vec2,
    pub floor: Option<f32>,
}

/// What a body's collision triangles make (`Arena::survey`, #285): how many, their bounds, and
/// whether they close round a volume. Corners are welded to the millimetre. A triangle with a twin
/// (the same corners wound the other way) faces both ways: Ipion keeps a lone triangle as a flat
/// ledge of its two sides, and the terrain's, the blockers' and most placed objects' faces come
/// so. Closed: the faces (a twin pair counted once) leave no edge open (used by one face).
/// Wound against its neighbour: an edge whose two faces, without twins, run along it the same
/// way. The volume is signed by the winding ((v1 - v0) x (v2 - v0) out: positive), the sum over
/// the triangles of the tetrahedra they make with the bounds' centre: 0 for a twinned body.
pub struct BodyShape {
    pub body: u32,
    pub terrain: bool,
    pub blocker: bool,
    /// a liquid's surface (not solid) or a door's leaves (not solid while open)
    pub liquid: bool,
    pub door: bool,
    /// the placed object it's part of (its index in `Level::objects`), if any
    pub object: Option<u32>,
    pub triangles: usize,
    pub lo: Vec3,
    pub hi: Vec3,
    /// edges used by a single face (a twin pair counted once)
    pub open_edges: usize,
    /// edges used by more than two faces (a twin pair counted once)
    pub shared_edges: usize,
    /// of the untwinned faces' edges, those whose two faces run along them the same way
    pub flipped_edges: usize,
    /// triangles with a twin
    pub twins: usize,
    /// triangles facing into the body's own solid (`Tri::inner`)
    pub inner: usize,
    /// m^3, by the winding
    pub volume: f32,
    /// every corner on or behind every triangle's plane (within 1 cm), by the winding (not tried
    /// over 2000 triangles)
    pub convex: bool,
}

impl BodyShape {
    fn of(body: u32, tris: &[&Tri]) -> BodyShape {
        type Key = bevy::math::IVec3;
        let key = |v: Vec3| (v * 1000.0).round().as_ivec3();
        let (lo, hi) = tris.iter().flat_map(|t| t.v).fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(l, h), v| (l.min(v), h.max(v)));
        let mid = (lo + hi) * 0.5;
        // (each face by its corners from its least, keeping the winding: a twin is the same
        // corners the other way round)
        let mut faces: HashMap<[Key; 3], u32> = HashMap::new();
        let mut volume = 0.0;
        for t in tris {
            let k = t.v.map(key);
            let first = (0..3).min_by_key(|&i| k[i].to_array()).unwrap_or(0);
            *faces.entry([k[first], k[(first + 1) % 3], k[(first + 2) % 3]]).or_default() += 1;
            let (a, b, c) = (t.v[0] - mid, t.v[1] - mid, t.v[2] - mid);
            volume += a.dot(b.cross(c)) / 6.0;
        }
        let twinned = |f: &[Key; 3]| faces.contains_key(&[f[0], f[2], f[1]]);
        let twins = faces.iter().filter(|(f, _)| twinned(f)).map(|(_, &n)| n as usize).sum();
        // (each undirected edge of the faces, a twin pair once: how many faces use it, and of the
        // untwinned how often each way)
        let mut edges: HashMap<(Key, Key), (u32, u32, u32)> = HashMap::new();
        for f in faces.keys() {
            let lone = !twinned(f);
            if !lone && f[1].to_array() > f[2].to_array() {
                continue;
            }
            for i in 0..3 {
                let (a, b) = (f[i], f[(i + 1) % 3]);
                let forward = a.to_array() < b.to_array();
                let e = edges.entry(if forward { (a, b) } else { (b, a) }).or_default();
                e.0 += 1;
                if lone && forward { e.1 += 1 } else if lone { e.2 += 1 }
            }
        }
        let open_edges = edges.values().filter(|e| e.0 == 1).count();
        let shared_edges = edges.values().filter(|e| e.0 > 2).count();
        let flipped_edges = edges.values().filter(|e| e.1 > 1 || e.2 > 1).count();
        let corners: Vec<Vec3> = tris.iter().flat_map(|t| t.v).collect();
        // (not tried on big ones, the terrain's blocks: never convex)
        let convex = tris.len() <= 2000 && tris.iter().all(|t| {
            let n = (t.v[1] - t.v[0]).cross(t.v[2] - t.v[0]).normalize_or_zero();
            corners.iter().all(|&p| (p - t.v[0]).dot(n) <= 0.01)
        });
        BodyShape { body, terrain: tris.iter().any(|t| t.terrain), blocker: tris.iter().any(|t| t.blocker), liquid: tris.iter().any(|t| t.liquid.is_some()),
                    door: tris.iter().any(|t| t.door.is_some()), object: tris.iter().find_map(|t| t.object),
                    triangles: tris.len(), lo, hi, open_edges, shared_edges, flipped_edges, twins, inner: tris.iter().filter(|t| t.inner).count(),
                    volume, convex }
    }
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

/// A wall (a triangle that isn't a floor, `Arena::wall_sections`) cut to the heights of a body's
/// wall test and seen from above (`Arena::move_body`, #285): the corners (x, z) of its part
/// between the heights, in order round it (a convex polygon of up to five; a segment or a point
/// for one seen edge on). The game sweeps a vertical cylinder against the whole triangle
/// (FUN_00167a80: GJK with the cylinder's rim as its support, FUN_001678a0 / FUN_00167810); the
/// cylinder meets the triangle where its circle meets this section, so the demo's sweep is a
/// circle's against it, in two dimensions.
struct WallSection {
    corners: [Vec2; 6],
    count: usize,
    /// which edges (corner k to k + 1) the heights cut across the triangle's face, not along one
    /// of its own edges: a body against one meets the face (`closest`)
    cut: [bool; 6],
    /// the triangle's normal by its winding and a corner: its plane, which side meets it
    /// (`faces`) and the way a face contact pushes (`normal`); and whether it's met from either
    /// side (`Tri::two_sided`)
    front: Vec3,
    corner: Vec3,
    two_sided: bool,
    /// the height of the body's axis its plane is tested at: the feet for a steep triangle, the
    /// body's middle for a ceiling (FUN_00152d20 raises the point by half the height for a
    /// triangle that isn't steep) and for the twin facing down of a floor that's stood on
    /// (`Tri::floor_back`, CEILING_NORMAL: the demo's)
    base: f32,
    /// a second height it's tested at, the body's middle, for a steep face of a rock (not the
    /// terrain's): met when either is in front (the demo's). A body on a rock's flank
    /// (`Arena::flank_below`, #285) has its feet on the face's plane, so at the feet alone the
    /// face over the step was left out, and the body slid down the flank with its middle over the
    /// rock, past its widest part and under it (#285's test of falls onto sdm_e34's boulder); met,
    /// the face pushes it off (`move_body`)
    also: Option<f32>,
}

impl WallSection {
    /// Triangle `t` between the heights lo and hi, if any of it is, for a body with its feet at
    /// `feet` and its middle at `mid`.
    fn of(t: &Tri, lo: f32, hi: f32, feet: f32, mid: f32) -> Option<WallSection> {
        // (Sutherland-Hodgman: above lo, then below hi; up to five corners, each with the height
        // it was cut at, if any: 1 lo, 2 hi)
        let clip = |poly: &[(Vec3, u8)], y: f32, above: bool, tag: u8| -> ([(Vec3, u8); 6], usize) {
            let inside = |p: Vec3| if above { p.y >= y } else { p.y <= y };
            let (mut out, mut m) = ([(Vec3::ZERO, 0u8); 6], 0);
            for k in 0..poly.len() {
                let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
                if inside(a.0) && m < 6 {
                    out[m] = a;
                    m += 1;
                }
                if inside(a.0) != inside(b.0) && m < 6 {
                    out[m] = (a.0 + (b.0 - a.0) * ((y - a.0.y) / (b.0.y - a.0.y)), tag);
                    m += 1;
                }
            }
            (out, m)
        };
        let (low, n) = clip(&t.v.map(|v| (v, 0)), lo, true, 1);
        let (both, n) = clip(&low[..n], hi, false, 2);
        if n == 0 {
            return None;
        }
        let mut corners = [Vec2::ZERO; 6];
        let mut tags = [0u8; 6];
        let mut count = 0;
        for &(p, tag) in &both[..n] {
            let q = Vec2::new(p.x, p.z);
            if count > 0 && corners[count - 1].distance_squared(q) <= 1e-10 {
                tags[count - 1] |= tag;
                continue;
            }
            corners[count] = q;
            tags[count] = tag;
            count += 1;
        }
        if count > 1 && corners[0].distance_squared(corners[count - 1]) <= 1e-10 {
            tags[0] |= tags[count - 1];
            count -= 1;
        }
        let cut = std::array::from_fn(|k| k < count && tags[k] & tags[(k + 1) % count] != 0);
        let steep = t.two_sided || (t.front.y > CEILING_NORMAL && !t.floor_back);
        let base = if steep { feet } else { mid };
        let also = (steep && !t.two_sided && !t.terrain).then_some(mid);
        Some(WallSection { corners, count, cut, front: t.front, corner: t.v[0], two_sided: t.two_sided, base, also })
    }

    /// Whether a body at `at` meets the wall: from its front (FUN_00152d20 leaves out a triangle
    /// whose plane, by the winding, has the body's point, `base`, behind it), more than
    /// PLANE_MARGIN in front (or at the `also` height); or either way for a drawn triangle.
    fn faces(&self, at: Vec2) -> bool {
        let in_front = |y: f32| (Vec3::new(at.x, y, at.y) - self.corner).dot(self.front) > PLANE_MARGIN;
        self.two_sided || in_front(self.base) || self.also.is_some_and(in_front)
    }

    /// Whether `p` is inside the section seen from above (a sliver never is).
    fn contains(&self, p: Vec2) -> bool {
        let c = &self.corners[..self.count];
        if c.len() < 3 {
            return false;
        }
        let area: f32 = (0..c.len()).map(|k| c[k].perp_dot(c[(k + 1) % c.len()])).sum();
        if area.abs() < 1e-6 {
            return false;
        }
        (0..c.len()).all(|k| (c[(k + 1) % c.len()] - c[k]).perp_dot(p - c[k]) * area.signum() >= 0.0)
    }

    /// The section's point nearest `p`, and whether that's a contact with the triangle's face
    /// (`p` over the section, or nearest a cut edge between its ends) rather than with one of its
    /// edges or corners (FUN_00150a10 tells the kinds apart by the contact point's distance from
    /// the edges).
    fn closest(&self, p: Vec2) -> (Vec2, bool) {
        let c = &self.corners[..self.count];
        if c.len() == 1 {
            return (c[0], false);
        }
        if self.contains(p) {
            return (p, true);
        }
        let mut best = (f32::MAX, c[0], false);
        let edges = if c.len() == 2 { 1 } else { c.len() };
        for k in 0..edges {
            let (a, b) = (c[k], c[(k + 1) % c.len()]);
            let q = closest_on_segment(p, a, b);
            if p.distance_squared(q) < best.0 {
                let inner = q.distance_squared(a) > 1e-8 && q.distance_squared(b) > 1e-8;
                best = (p.distance_squared(q), q, self.cut[k] && inner);
            }
        }
        (best.1, best.2)
    }

    /// The way a contact with the section pushes a body whose axis is at `p`, its nearest point
    /// `q` on it (`closest`): a face contact along the triangle's normal across (FUN_00150a10
    /// takes a steep face's contact normal across; for one met from either side, the side the
    /// body's point, `base`, is on), an edge or corner contact away from it across. None for a
    /// face lying flat (the game's bump on the head) or one facing away from the body: the twin
    /// facing into a slope it's on, met where a bump in the ground has its feet in front of it.
    fn normal(&self, p: Vec2, q: Vec2, face: bool) -> Vec2 {
        let across = Vec2::new(self.front.x, self.front.z);
        let across = if self.two_sided && (Vec3::new(p.x, self.base, p.y) - self.corner).dot(self.front) < 0.0 { -across } else { across };
        let on = p.distance_squared(q) <= 1e-10;
        if face || on {
            let n = if across.length() > 0.05 { across.normalize() } else { Vec2::ZERO };
            return if !on && (p - q).dot(n) < 0.0 { Vec2::ZERO } else { n };
        }
        (p - q).normalize()
    }

    /// How far along unit `u` a circle of radius `r` from `a` (clear of the section) goes before
    /// it meets the section, if it does.
    fn meets(&self, a: Vec2, u: Vec2, r: f32) -> Option<f32> {
        let c = &self.corners[..self.count];
        let edges = if c.len() <= 2 { 1 } else { c.len() };
        (0..edges).filter_map(|k| circle_meets_segment(a, u, c[k], c[(k + 1) % c.len()], r)).reduce(f32::min)
    }
}

/// The point of segment a..b nearest `p` (a point when they're the same).
fn closest_on_segment(p: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let d = b - a;
    let l = d.length_squared();
    if l < 1e-12 {
        return a;
    }
    a + d * ((p - a).dot(d) / l).clamp(0.0, 1.0)
}

/// How far along unit `u` a circle of radius `r` from `a`, clear of segment e0..e1 (more than
/// `r` from it), goes before it touches it: the first of its end points' circles and the band
/// along it.
fn circle_meets_segment(a: Vec2, u: Vec2, e0: Vec2, e1: Vec2, r: f32) -> Option<f32> {
    let end = |c: Vec2| {
        let m = a - c;
        let (b, k) = (m.dot(u), m.length_squared() - r * r);
        let disc = b * b - k;
        (k > 0.0 && b < 0.0 && disc >= 0.0).then(|| -b - disc.sqrt())
    };
    let mut best = [end(e0), end(e1)].into_iter().flatten().reduce(f32::min);
    let d = e1 - e0;
    let l = d.length_squared();
    if l > 1e-12 {
        let n = Vec2::new(-d.y, d.x) / l.sqrt();
        let (h, v) = ((a - e0).dot(n), u.dot(n));
        let s = if h > r && v < 0.0 { Some((h - r) / -v) } else if h < -r && v > 0.0 { Some((-r - h) / v) } else { None };
        if let Some(s) = s.filter(|&s| (0.0..=1.0).contains(&((a + u * s - e0).dot(d) / l))) {
            best = Some(best.map_or(s, |b| b.min(s)));
        }
    }
    best
}

/// Move `m` with its part into each of the walls `normals` (across, unit) taken out, as the
/// game's response takes the move out of every plane it touches (FUN_00150f90 builds the
/// system, FUN_00168950 solves it, FUN_00151170 adds the impulses along the normals: no
/// friction). Projected in turn a few times; one still going into a wall (a corner it's pressed
/// into) stops.
fn slide_off(m: Vec2, normals: &[Vec2]) -> Vec2 {
    let mut m = m;
    for _ in 0..4 {
        let mut changed = false;
        for n in normals {
            let into = m.dot(*n);
            if into < 0.0 {
                m -= *n * into;
                changed = true;
            }
        }
        if !changed {
            return m;
        }
    }
    if normals.iter().any(|n| m.dot(*n) < -1e-5) { Vec2::ZERO } else { m }
}

/// A body's triangles as a surface (#285): whether it closes (every edge of its faces, a twin
/// pair counted once, used an even number of times: no edge left open), and its twin pairs (the
/// same corners, to the millimetre, wound the other way: the first's place in `tris`, then the
/// second's). Ipion keeps a lone triangle as a flat ledge of its two sides, so a twinned face is
/// one face of the surface (`BodyShape`).
fn shell(tris: &[Tri]) -> (bool, Vec<(usize, usize)>) {
    type Key = bevy::math::IVec3;
    let key = |v: Vec3| (v * 1000.0).round().as_ivec3();
    // (each face's unpaired triangles so far)
    let mut seen: HashMap<[Key; 3], Vec<usize>> = HashMap::new();
    let mut twins = vec![];
    let mut edges: HashMap<(Key, Key), u32> = HashMap::new();
    for (i, t) in tris.iter().enumerate() {
        let k = t.v.map(key);
        let first = (0..3).min_by_key(|&j| k[j].to_array()).unwrap_or(0);
        let face = [k[first], k[(first + 1) % 3], k[(first + 2) % 3]];
        // (its twin seen already, and not yet paired: this one is the second)
        let back = [face[0], face[2], face[1]];
        if let Some(j) = seen.get_mut(&back).and_then(|v| v.pop()) {
            twins.push((j, i));
            continue;
        }
        seen.entry(face).or_default().push(i);
        for j in 0..3 {
            let (a, b) = (face[j], face[(j + 1) % 3]);
            *edges.entry(if a.to_array() < b.to_array() { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    (edges.values().all(|n| n % 2 == 0), twins)
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
                            breakable_of: HashMap::new(), flat: None, bodies: 0, spans: vec![], solid_shell: vec![] };
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
            let first = a.tris.len();
            for (t, m) in game.collision(s) {
                a.add(t, true, None, m);
            }
            a.new_body(first);
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
            a.new_body(first);
        }
        if !physics {
            if let Some((g, _)) = level.terrain.first() {
                let first = a.tris.len();
                for t in g.indices.chunks_exact(3) {
                    a.add([0, 1, 2].map(|k| Vec3::from(g.positions[t[k] as usize])), true, None, 0);
                }
                // (drawn triangles: met from either side)
                for t in &mut a.tris[first..] {
                    t.two_sided = true;
                }
                a.new_body(first);
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
                t.two_sided = !physics;
            }
            a.new_body(first);
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
                        t.two_sided = !physics;
                    }
                    a.new_body(first);
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
        // (`front` keeps the winding)
        let front = n.normalize();
        let n = if front.y < 0.0 { -front } else { front };
        let i = self.tris.len() as u32;
        self.tris.push(Tri { v, n, front, two_sided: false, duplicate: false, inner: false, floor_back: false, terrain, material, door, liquid: None, piece: None,
                             object: None, blocker: false, body: self.bodies });
        let (lo, hi) = (v[0].min(v[1]).min(v[2]), v[0].max(v[1]).max(v[2]));
        let (c0, c1) = (cell(lo.x, lo.z), cell(hi.x, hi.z));
        for x in c0.0..=c1.0 {
            for z in c0.1..=c1.1 {
                self.cells.entry((x, z)).or_default().push(i);
            }
        }
    }

    /// The triangles added since `first` are one physics body (`Tri::body`): they keep the
    /// number they were added with, and the next body takes the next. Its twins are marked
    /// (`Tri::duplicate`), whether it closes round a solid kept (`solid_shell`), then of each twin
    /// pair the one facing into that solid (`Tri::inner`: a point INNER_OFFSET in front of it is
    /// in the solid, `in_body`, at two of the three INNER_PROBES, and not so for its twin), and the
    /// twins facing down of its floors that are stood on (`Tri::floor_back`).
    fn new_body(&mut self, first: usize) {
        for t in &mut self.tris[first..] {
            t.body = self.bodies;
        }
        let (closed, twins) = shell(&self.tris[first..]);
        for &(_, i) in &twins {
            self.tris[first + i].duplicate = true;
        }
        let moving = self.tris[first..].iter().any(|t| t.door.is_some() || t.liquid.is_some());
        self.spans.push((first as u32, self.tris.len() as u32));
        let solid = closed && !moving && self.tris.len() > first;
        self.solid_shell.push(solid);
        if solid {
            // (of each twin pair, the one facing in; a pair that tests in both ways or neither,
            // a thin or doubled part, keeps both, as a lone face its own)
            let body = self.bodies;
            let faces_in = |t: &Tri| !t.two_sided && INNER_PROBES.iter().filter(|w| {
                let p = t.v[0] * w[0] + t.v[1] * w[1] + t.v[2] * w[2] + t.front * INNER_OFFSET;
                self.in_body(p, body)
            }).count() >= 2;
            let inner: Vec<usize> = twins.iter().filter_map(|&(a, b)| {
                let (a, b) = (first + a, first + b);
                match (faces_in(&self.tris[a]), faces_in(&self.tris[b])) {
                    (true, false) => Some(a),
                    (false, true) => Some(b),
                    _ => None,
                }
            }).collect();
            for i in inner {
                self.tris[i].inner = true;
            }
        }
        for (a, b) in twins {
            let (a, b) = (first + a, first + b);
            for (down, up) in [(a, b), (b, a)] {
                let floor = &self.tris[up];
                if floor.front.y >= FLOOR_NORMAL && !floor.inner {
                    self.tris[down].floor_back = true;
                }
            }
        }
        self.bodies += 1;
    }

    /// Whether `at` is in body `body`'s solid: the vertical lines below and above it each cross
    /// its surface (its faces, a twin pair once) an odd number of times (`inside`, for one body,
    /// whatever is open or broken).
    fn in_body(&self, at: Vec3, body: u32) -> bool {
        let (mut below, mut above) = (0u32, 0u32);
        for &i in self.near(at.x, at.z) {
            let t = &self.tris[i as usize];
            if t.body != body || t.duplicate {
                continue;
            }
            if let Some(y) = Self::height_on(t, at.x, at.z) {
                if y < at.y { below += 1 } else { above += 1 }
            }
        }
        below % 2 == 1 && above % 2 == 1
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
                breakable_of: HashMap::new(), flat: Some(ground), bodies: 0, spans: vec![], solid_shell: vec![] }
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
                broken: flags(&self.broken), dead: flags(&self.dead), breakable_of: self.breakable_of.clone(), flat: self.flat, bodies: self.bodies,
                spans: self.spans.clone(), solid_shell: self.solid_shell.clone() }
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
        let first = a.tris.len();
        for (t, m) in tris {
            a.add(t.map(|v| at.transform_point3(v)), false, None, *m);
        }
        a.new_body(first);
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

    /// A rock's flank under (x, z) no higher than `top` (#285): the highest face of a placed
    /// object or blocker too steep to stand on that faces up and out by its winding (normal y
    /// from SLOPE_UPRIGHT to FLOOR_NORMAL), with that normal; not one facing into the rock
    /// (`Tri::inner`). A body falling with its middle over a rock's steep side met no floor there
    /// and fell on through the rock to the floor under it, to be taken out of it at the bottom
    /// (`escape`: half a metre or more at once, #285's test of falls onto sdm_e34's boulder);
    /// the game's sweep meets the body with the face and moves it off along it. The demo stands
    /// it on the face as on terrain too steep to stand on (play.rs `slide_on_face`, #264), and
    /// the face's part over the step pushes it off (`move_body`).
    pub fn flank_below(&self, x: f32, z: f32, top: f32) -> Option<(f32, Vec3)> {
        let mut best: Option<(f32, Vec3)> = None;
        for &i in self.near(x, z) {
            let t = &self.tris[i as usize];
            if t.terrain || t.two_sided || t.inner || t.front.y >= FLOOR_NORMAL || t.front.y < SLOPE_UPRIGHT || !self.solid(t) {
                continue;
            }
            if let Some(y) = Self::height_on(t, x, z).filter(|y| *y <= top) {
                if best.is_none_or(|(b, _)| y > b) {
                    best = Some((y, t.front));
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
    /// A floor faces up by its winding (#285): the game meets a triangle only from its front
    /// (FUN_00152d20), so one wound down (the underside of a rock that isn't twinned) holds no
    /// one up, and of a twin pair the floor is the one facing up, which carries the material and
    /// the slide flag (`ray_material`; with both, a rounding tie picked either: e40's canyon rim
    /// at (-54.23, 84.80), its twin facing down 4 µm higher, material 0). One facing up into its
    /// own body's solid (`Tri::inner`: the twin of a rock's underside) isn't a floor: stood on, it
    /// took a body walking at a rock's foot into the rock. A drawn triangle of a level without
    /// collision data faces either way.
    fn floor_tri(&self, x: f32, z: f32, top: f32) -> Option<(f32, Option<&Tri>)> {
        let mut best: Option<(f32, Option<&Tri>)> = self.flat.filter(|&g| g <= top).map(|g| (g, None));
        for &i in self.near(x, z) {
            let t = &self.tris[i as usize];
            let up = if t.two_sided { t.n.y } else { t.front.y };
            if up < FLOOR_NORMAL || t.inner || !self.solid(t) {
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

    /// The body whose solid `at` is in, if any (#285): a body that closes round a solid
    /// (`solid_shell`: a rock) has its surface crossed an odd number of times by the vertical line
    /// below `at` and by the one above it (each face once, `Tri::duplicate`; two lines, so a
    /// crossing counted twice on an edge two faces share makes them disagree, and that counts as
    /// outside). Only solid faces count. The game doesn't ask this: its sweep never lets a body
    /// in (FUN_00153810).
    pub fn inside(&self, at: Vec3) -> Option<u32> {
        let mut crossings: HashMap<u32, (u32, u32)> = HashMap::new();
        for &i in self.near(at.x, at.z) {
            let t = &self.tris[i as usize];
            if t.duplicate || !self.solid_shell[t.body as usize] || !self.solid(t) {
                continue;
            }
            if let Some(y) = Self::height_on(t, at.x, at.z) {
                let c = crossings.entry(t.body).or_default();
                if y < at.y { c.0 += 1 } else { c.1 += 1 }
            }
        }
        crossings.into_iter().filter(|(_, (below, above))| below % 2 == 1 && above % 2 == 1).map(|(b, _)| b).min()
    }

    /// The placed object body `body` is part of (its index in `Level::objects`), if any: for the
    /// log (BF_INSIDE_LOG).
    pub fn body_object(&self, body: u32) -> Option<u32> {
        let (first, end) = *self.spans.get(body as usize)?;
        self.tris[first as usize..end as usize].iter().find_map(|t| t.object)
    }

    /// Every body's shape (`BodyShape`), for the survey (play.rs BF_FIND_BODIES, #285).
    pub fn survey(&self) -> Vec<BodyShape> {
        let mut by_body: HashMap<u32, Vec<&Tri>> = HashMap::new();
        for t in &self.tris {
            by_body.entry(t.body).or_default().push(t);
        }
        let mut out: Vec<BodyShape> = by_body.into_iter().map(|(body, tris)| BodyShape::of(body, &tris)).collect();
        out.sort_by_key(|s| s.body);
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
    /// at a few heights against the steep triangles near it. Returns the corrected (x, z). For
    /// what isn't a character (a grenade, a ragdoll's bones) and a character's place before it's
    /// in the world (the squad's spawn); a character's own moves are swept (`move_body`).
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

    /// A character's move this frame, from `from` to `to` (x, z), its body a vertical cylinder of
    /// `radius` from `step` over its `feet` to its `head`, as the game's sweep moves one
    /// (FUN_00153810, #285); where it ends:
    /// - the move swept against the walls (`WallSection`: what isn't a floor, by its winding,
    ///   between the cylinder's heights), as FUN_00152f20 sweeps the whole body along it: it stops
    ///   at the first one it goes into (SWEEP_ENTERS) before it passes through, however fast it
    ///   goes, and what's left of the move has its part into the wall taken out (FUN_00150f90 /
    ///   FUN_00168950 / FUN_00151170: no friction), up to SWEEP_PASSES times; a wall it already
    ///   touches that the move goes into does the same at once (FUN_00152f20 makes no progress
    ///   while one is met). A wall the body is behind where the move starts is left out, as the
    ///   game leaves out a triangle whose plane has the body behind it (FUN_00152d20; the demo
    ///   tells once, at the start: its move is level, with the feet where they were, so along a
    ///   slope they'd fall behind the plane of the ground ahead); Ipion's twinned faces are met
    ///   from both sides, but for a rock's faces that face into it (`Tri::inner`);
    /// - then out of any wall it overlaps (a door shut on it), met from its front, as `push_out`
    ///   did before (the game pushes nothing out: its sweep only stops a move), by its radius at
    ///   most, unless that would put it in a rock (`inside_span`).
    ///
    /// A body sliding down a steep terrain face of normal `face` (#264) isn't held or pushed by
    /// the terrain's steep triangles that face the same way (their across normal within 90
    /// degrees of the face's): they're what it slides on. The game raises the cylinder's bottom
    /// for a steep triangle only, 0.4 m on the ground and 0.1 m in the air, and meets any other
    /// with the whole cylinder (FUN_00152d20); the demo cuts every wall at `step` over the feet
    /// (STEP_UP, 0.5 m), so a ceiling lower than that over the feet isn't met. A body already in
    /// a rock is `out_of_rock`'s.
    #[allow(clippy::too_many_arguments)]
    pub fn move_body(&self, from: Vec2, to: Vec2, radius: f32, feet: f32, head: f32, step: f32, face: Option<Vec3>) -> Vec2 {
        let lo = feet + step;
        let walls: Vec<WallSection> = self.wall_sections(from, to, radius, feet, head, step, face).into_iter().filter(|w| w.faces(from)).collect();
        let mut at = from;
        let mut rest = to - from;
        // (the wall it stopped at last pass)
        let mut held: Option<Vec2> = None;
        for _ in 0..SWEEP_PASSES {
            if rest.length() < 1e-5 {
                break;
            }
            let u = rest.normalize();
            let mut touching: Vec<Vec2> = held.into_iter().filter(|n| u.dot(*n) < SWEEP_ENTERS).collect();
            for w in &walls {
                let (q, face) = w.closest(at);
                if at.distance(q) < radius + SWEEP_TOUCH {
                    let n = w.normal(at, q, face);
                    if u.dot(n) < SWEEP_ENTERS {
                        touching.push(n);
                    }
                }
            }
            if !touching.is_empty() {
                rest = slide_off(rest, &touching);
                if rest.length() < 1e-5 {
                    break;
                }
            }
            let len = rest.length();
            let u = rest / len;
            // the first wall the body meets on the way that the move goes into
            let mut first: Option<(f32, Vec2)> = None;
            for w in &walls {
                if at.distance(w.closest(at).0) < radius {
                    continue;
                }
                let Some(s) = w.meets(at, u, radius).filter(|&s| s <= len) else { continue };
                let p = at + u * s;
                let (q, face) = w.closest(p);
                let n = w.normal(p, q, face);
                if u.dot(n) < SWEEP_ENTERS && first.is_none_or(|f| s < f.0) {
                    first = Some((s, n));
                }
            }
            match first {
                None => {
                    at += rest;
                    rest = Vec2::ZERO;
                }
                Some((s, n)) => {
                    let go = (s - SWEEP_SKIN).max(0.0);
                    at += u * go;
                    rest = u * (len - go);
                    held = Some(n);
                }
            }
        }
        // out of what it overlaps, from the front, no further than its radius however many walls
        // it overlaps (unless that puts it in a rock)
        let swept = at;
        for _ in 0..2 {
            for w in &walls {
                let (q, face) = w.closest(at);
                let d = at.distance(q);
                if d < radius {
                    at += w.normal(at, q, face) * (radius - d);
                }
            }
        }
        let pushed = at - swept;
        if pushed.length() > radius {
            at = swept + pushed.normalize() * radius;
        }
        if at != swept && self.inside_span(at, lo, head).is_some() && self.inside_span(swept, lo, head).is_none() {
            at = swept;
        }
        at
    }

    /// A body standing at `at` with its feet at `feet` whose span from `step` over them to its
    /// `head` is in a rock's solid (`inside_span`: put there by a spawn, a test teleport, or a
    /// fall's drop, which isn't swept, past a rock's side it slid off under the rock's widest part)
    /// taken out of it (`escape`), if it is (#285).
    pub fn out_of_rock(&self, at: Vec2, radius: f32, feet: f32, head: f32, step: f32) -> Option<Moved> {
        let lo = feet + step;
        self.inside_span(at, lo, head).map(|body| self.escape(at, body, radius, feet, lo, head))
    }

    /// The walls near a move from `from` to `to` (`move_body`) cut to the heights lo..hi
    /// (`WallSection`): the solid triangles that aren't a floor by their winding (normal y under
    /// FLOOR_NORMAL: the steep ones, and the ceilings, normal y at or under cos 130, which the game
    /// meets with the whole cylinder, FUN_00152d20, and whose edge stops a move across,
    /// FUN_00150a10: a rock's underside over the step is in the way), a drawn triangle of a level
    /// without collision data steep either way up; not one facing into its own rock (`Tri::inner`)
    /// nor the face a body slides down (`face`, #264). The heights: from `step` over the `feet` to
    /// the `head`; what's under the step is the floors' to step onto.
    #[allow(clippy::too_many_arguments)]
    fn wall_sections(&self, from: Vec2, to: Vec2, radius: f32, feet: f32, head: f32, step: f32, face: Option<Vec3>) -> Vec<WallSection> {
        let (lo, mid) = (feet + step, (feet + head) * 0.5);
        let reach = radius + SWEEP_TOUCH + 0.01;
        let (a, b) = (from.min(to) - Vec2::splat(reach), from.max(to) + Vec2::splat(reach));
        let (c0, c1) = (cell(a.x, a.y), cell(b.x, b.y));
        let mut seen = HashSet::new();
        let mut out = vec![];
        for x in c0.0..=c1.0 {
            for z in c0.1..=c1.1 {
                for &i in self.cells.get(&(x, z)).map_or(&[][..], |v| v.as_slice()) {
                    if !seen.insert(i) {
                        continue;
                    }
                    let t = &self.tris[i as usize];
                    let wall = if t.two_sided { t.n.y < FLOOR_NORMAL } else { t.front.y < FLOOR_NORMAL };
                    if !wall || t.inner || !self.solid(t) || face.is_some_and(|f| t.terrain && t.n.x * f.x + t.n.z * f.z > 0.0) {
                        continue;
                    }
                    if let Some(w) = WallSection::of(t, lo, head, feet, mid) {
                        out.push(w);
                    }
                }
            }
        }
        out
    }

    /// Whether a body standing at `at` with its feet at `feet` has no wall more than CLIMB_SLACK
    /// within `radius` of it between `step` over its feet and its `head` (`move_body`'s walls, met
    /// from their front and pushing it, `WallSection::normal`): where a step up may raise it
    /// (#285). The game's sweep moves a body up a slope with its move (FUN_0012cd10 projects the
    /// move on the ground's normal before FUN_00153810), so a rock overhanging the slope stops it;
    /// the demo raises the feet to the floor after the move (play.rs), and asks this first.
    pub fn clear(&self, at: Vec2, radius: f32, feet: f32, head: f32, step: f32) -> bool {
        self.wall_sections(at, at, radius, feet, head, step, None).iter().all(|w| {
            let (q, face) = w.closest(at);
            !w.faces(at) || at.distance(q) >= radius - CLIMB_SLACK || w.normal(at, q, face) == Vec2::ZERO
        })
    }

    /// The rock body (`inside`) a body standing at `at` has its span lo..hi in: tried at its
    /// bottom, middle and top.
    pub fn inside_span(&self, at: Vec2, lo: f32, hi: f32) -> Option<u32> {
        [lo, (lo + hi) * 0.5, hi].into_iter().find_map(|y| self.inside(Vec3::new(at.x, y, at.y)))
    }

    /// A body at `at` whose span lo..hi is in rock `body` (`inside_span`) taken out to the nearest
    /// place round it clear of it (#285: "pushed out to the nearest outside point"): of
    /// ESCAPE_WAYS ways evenly round, the one where the rock's surface is nearest, measured at each
    /// height of the span that's inside it and taken at the farthest of those, then its radius
    /// and ESCAPE_CLEAR beyond. A place in another rock, or whose floor (the highest under its
    /// head) is further from its feet than its head is, comes after one without. Its feet go on
    /// that floor (`Moved::floor`). The demo's: the game never has a body inside (its sweep
    /// doesn't let one in, FUN_00153810).
    pub fn escape(&self, at: Vec2, body: u32, radius: f32, feet: f32, lo: f32, hi: f32) -> Moved {
        let heights: Vec<f32> = [lo, (lo + hi) * 0.5, hi].into_iter().filter(|&y| self.inside(Vec3::new(at.x, y, at.y)) == Some(body)).collect();
        let (first, end) = self.spans[body as usize];
        let faces: Vec<&Tri> = self.tris[first as usize..end as usize].iter().filter(|t| !t.duplicate && self.solid(t)).collect();
        // (rank: in a rock or with no floor near: after; then the distance)
        let mut best: Option<((bool, f32), Vec2, Option<f32>)> = None;
        for k in 0..ESCAPE_WAYS {
            let a = k as f32 * std::f32::consts::TAU / ESCAPE_WAYS as f32;
            let d = Vec2::new(a.cos(), a.sin());
            let out = heights.iter().map(|&y| {
                let o = Vec3::new(at.x, y, at.y);
                faces.iter().filter_map(|t| ray_triangle(&t.v, o, Vec3::new(d.x, 0.0, d.y)).filter(|&s| s > 1e-4)).fold(f32::MAX, f32::min)
            }).fold(0.0, f32::max);
            if out == f32::MAX {
                continue;
            }
            let p = at + d * (out + radius + ESCAPE_CLEAR);
            let floor = self.floor_below(p.x, p.y, hi).map(|f| f.0);
            let stuck = self.inside_span(p, lo, hi).is_some() || floor.is_none_or(|g| (g - feet).abs() > hi - feet);
            if best.is_none_or(|(r, _, _)| (stuck, out) < r) {
                best = Some(((stuck, out), p, floor));
            }
        }
        match best {
            Some((_, p, floor)) => Moved { at: p, floor },
            None => Moved { at, floor: Some(feet) },
        }
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

    /// What a shot's ray meets first within `max` (`ShotHit`), as a weapon's world query takes
    /// the world: the filter a bolt's sweep (FUN_00225440) and an instant ray (FUN_00230040,
    /// FUN_00230400) cast with (vtable 0x394848) leaves out every body kind but the PID_ list's 0,
    /// 1 and 2 (its +0x10c mask 0xfffffff8: FUN_000df0a0 tests the objects' bodies, the bones
    /// and the terrain, and skips the blockers' list, kinds 4, 6 and 8), so the level's
    /// invisible blockers aren't met; nor is body `skip` (the object a bolt last passed through,
    /// its +0x40).
    pub fn ray_shot(&self, origin: Vec3, dir: Vec3, max: f32, skip: Option<u32>) -> Option<ShotHit> {
        let keep = |t: &Tri| !t.blocker && Some(t.body) != skip;
        let (t, i) = self.ray_tri_where(origin, dir, max, keep)?;
        let Some(i) = i else {
            // (the flat floor's ground)
            return (skip != Some(FLAT_BODY)).then_some(ShotHit { t, normal: Vec3::Y, material: 0, terrain: true, body: FLAT_BODY, breakable: None });
        };
        let tri = &self.tris[i];
        let n = (tri.v[1] - tri.v[0]).cross(tri.v[2] - tri.v[0]).normalize_or(Vec3::Y);
        // (the material by `ray_material`'s rule: of a face met from its front within FRONT_SLACK)
        let front = self.ray_tri_where(origin, dir, (t + FRONT_SLACK).min(max), |x| keep(x) && (x.v[1] - x.v[0]).cross(x.v[2] - x.v[0]).dot(dir) < 0.0)
            .and_then(|(_, j)| j).unwrap_or(i);
        Some(ShotHit { t, normal: if n.dot(dir) > 0.0 { -n } else { n }, material: self.tris[front].material & MATERIAL_ID, terrain: tri.terrain,
                       body: tri.body, breakable: tri.piece.filter(|p| !p.1).map(|p| p.0 as usize) })
    }

    /// Every body a shot's ray meets within `max`, once each where it first meets it, nearest
    /// first (`ShotHit`, by `ray_shot`'s filter): the multi-hit ray of an instant ray that passes
    /// through what it hits (FUN_00230400 casts FUN_000dd8e0 with its list, which keeps one result
    /// per body, sorted). Its filter leaves the liquids out too (its +0x110, 0x400000: object type
    /// 22), as `solid` does every query's.
    pub fn ray_shot_all(&self, origin: Vec3, dir: Vec3, max: f32) -> Vec<ShotHit> {
        // the nearest crossing of each body's triangles (and whether it's met from the front)
        let mut first: HashMap<u32, (f32, usize)> = HashMap::new();
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
                if !self.solid(tri) || tri.blocker {
                    continue;
                }
                let Some(d) = ray_triangle(&tri.v, origin, dir).filter(|&d| d > 1e-3 && d <= max) else { continue };
                let e = first.entry(tri.body).or_insert((d, i as usize));
                // (of a body's faces met within FRONT_SLACK of its nearest, the one met from its
                // front: `ray_material`'s twins)
                let front = |j: usize| (self.tris[j].v[1] - self.tris[j].v[0]).cross(self.tris[j].v[2] - self.tris[j].v[0]).dot(dir) < 0.0;
                if d < e.0 - FRONT_SLACK || (d <= e.0 + FRONT_SLACK && front(i as usize) && !front(e.1)) {
                    *e = (d.min(e.0), i as usize);
                }
            }
            if t >= max {
                break;
            }
            t = (t + step).min(max);
        }
        let mut out: Vec<ShotHit> = first.into_iter().map(|(body, (t, i))| {
            let tri = &self.tris[i];
            let n = (tri.v[1] - tri.v[0]).cross(tri.v[2] - tri.v[0]).normalize_or(Vec3::Y);
            ShotHit { t, normal: if n.dot(dir) > 0.0 { -n } else { n }, material: tri.material & MATERIAL_ID, terrain: tri.terrain, body,
                      breakable: tri.piece.filter(|p| !p.1).map(|p| p.0 as usize) }
        }).collect();
        // (the flat floor's ground, met from above)
        if let Some(g) = self.flat.filter(|_| dir.y < -1e-4) {
            let d = (g - origin.y) / dir.y;
            if d > 1e-3 && d <= max {
                out.push(ShotHit { t: d, normal: Vec3::Y, material: 0, terrain: true, body: FLAT_BODY, breakable: None });
            }
        }
        out.sort_by(|a, b| a.t.total_cmp(&b.t));
        out
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
                if let Some(d) = ray_triangle(&tri.v, origin, dir).filter(|&d| d > 1e-3 && d < best) {
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

/// Where a ray (from `origin` along unit `dir`) crosses triangle `v`, met from either side
/// (Moller-Trumbore): how far along, if it does.
fn ray_triangle(v: &[Vec3; 3], origin: Vec3, dir: Vec3) -> Option<f32> {
    let (e1, e2) = (v[1] - v[0], v[2] - v[0]);
    let pv = dir.cross(e2);
    let det = e1.dot(pv);
    if det.abs() < 1e-9 {
        return None;
    }
    let tv = origin - v[0];
    let u = tv.dot(pv) / det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let qv = tv.cross(e1);
    let w = dir.dot(qv) / det;
    if w < 0.0 || u + w > 1.0 {
        return None;
    }
    Some(e2.dot(qv) / det)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A box's twelve triangles from `lo` to `hi`, each wound to face out ((v1 - v0) x (v2 - v0)
    /// away from its middle).
    fn box_tris(lo: Vec3, hi: Vec3) -> Vec<[Vec3; 3]> {
        let c = |x: bool, y: bool, z: bool| Vec3::new(if x { hi.x } else { lo.x }, if y { hi.y } else { lo.y }, if z { hi.z } else { lo.z });
        let quads = [
            [c(false, false, false), c(false, true, false), c(false, true, true), c(false, false, true)],
            [c(true, false, false), c(true, true, false), c(true, true, true), c(true, false, true)],
            [c(false, false, false), c(true, false, false), c(true, false, true), c(false, false, true)],
            [c(false, true, false), c(true, true, false), c(true, true, true), c(false, true, true)],
            [c(false, false, false), c(true, false, false), c(true, true, false), c(false, true, false)],
            [c(false, false, true), c(true, false, true), c(true, true, true), c(false, true, true)],
        ];
        let mid = (lo + hi) * 0.5;
        let mut out = vec![];
        for q in quads {
            for t in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
                let n = (t[1] - t[0]).cross(t[2] - t[0]);
                let centre = (t[0] + t[1] + t[2]) / 3.0;
                out.push(if n.dot(centre - mid) > 0.0 { t } else { [t[0], t[2], t[1]] });
            }
        }
        out
    }

    /// The flat floor at 0 with one body of `tris` on it (twinned: each also wound the other way,
    /// as Ipion's lone triangles come).
    fn arena_with(tris: &[[Vec3; 3]], twinned: bool) -> Arena {
        let mut a = Arena::flat(0.0);
        for t in tris {
            a.add(*t, false, None, 0);
            if twinned {
                a.add([t[0], t[2], t[1]], false, None, 0);
            }
        }
        a.new_body(0);
        a
    }

    /// A character's move on the flat floor: radius 0.4, 1.9 m tall, its wall test from 0.5 m up.
    fn go(a: &Arena, from: Vec2, to: Vec2) -> Vec2 {
        a.move_body(from, to, 0.4, 0.0, 1.9, 0.5, None)
    }

    /// #285: a 2 m rock (a twinned box) holds a body out however fast it comes at it: in one 5 m
    /// move or in steps, it stops 0.4 m (its radius) short of the face; a move along the face
    /// isn't held, and one at it slantwise slides along it.
    #[test]
    fn a_rock_holds_a_body_out() {
        let a = arena_with(&box_tris(Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 2.0, 1.0)), true);
        assert!(a.solid_shell[0]);
        let m = go(&a, Vec2::new(-5.0, 0.1), Vec2::new(0.0, 0.1));
        assert!((m.x - -1.4).abs() < 0.01 && (m.y - 0.1).abs() < 1e-4, "{m:?}");
        let mut at = Vec2::new(-5.0, 0.3);
        for _ in 0..20 {
            at = go(&a, at, at + Vec2::new(0.6, 0.0));
            assert!(at.x <= -1.39, "{at:?}");
        }
        let along = go(&a, Vec2::new(-1.41, -3.0), Vec2::new(-1.41, 3.0));
        assert!(along.distance(Vec2::new(-1.41, 3.0)) < 1e-4, "{along:?}");
        let slant = go(&a, Vec2::new(-3.0, -2.0), Vec2::new(0.0, 1.0));
        assert!((slant.x - -1.4).abs() < 0.01 && slant.y > -0.5, "{slant:?}");
    }

    /// #285: a point inside the rock's solid is inside it (`inside`), one beside it or over it
    /// isn't; a body put inside is taken out to the nearest side, past its radius, its feet on
    /// the floor there; one outside is left alone.
    #[test]
    fn a_body_inside_a_rock_is_taken_out() {
        let a = arena_with(&box_tris(Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 2.0, 1.0)), true);
        assert_eq!(a.inside(Vec3::new(0.2, 0.7, 0.1)), Some(0));
        assert_eq!(a.inside(Vec3::new(1.3, 0.7, 0.1)), None);
        assert_eq!(a.inside(Vec3::new(0.2, 2.5, 0.1)), None);
        let m = a.out_of_rock(Vec2::new(0.2, 0.1), 0.4, 0.0, 1.9, 0.5).expect("in the rock");
        assert_eq!(m.floor, Some(0.0));
        assert!((m.at.x - 1.42).abs() < 0.01 && (m.at.y - 0.1).abs() < 0.01, "{:?}", m.at);
        assert!(a.inside_span(m.at, 0.5, 1.9).is_none());
        assert_eq!(a.out_of_rock(Vec2::new(-3.0, 0.1), 0.4, 0.0, 1.9, 0.5), None);
    }

    /// #285's test: the faces of a twinned rock that face into it (`Tri::inner`) are neither
    /// floors nor walls. A box over the ground from 0.2 m to 2 m: its bottom's twin facing up is
    /// inside it, under the step from the ground, and isn't stood on (it was: a body walking at a
    /// rock's foot went in onto it); its top is. Half its triangles face in.
    #[test]
    fn a_rock_faces_in_are_left_out() {
        let a = arena_with(&box_tris(Vec3::new(-1.0, 0.2, -1.0), Vec3::new(1.0, 2.0, 1.0)), true);
        assert_eq!(a.tris.iter().filter(|t| t.inner).count(), 12);
        assert_eq!(a.floor_below(0.5, 0.1, 0.5).map(|f| f.0), Some(0.0));
        assert_eq!(a.floor_below(0.5, 0.1, 2.4).map(|f| f.0), Some(2.0));
        assert!(a.tris.iter().filter(|t| t.inner).all(|t| t.front.dot((t.v[0] + t.v[1] + t.v[2]) / 3.0 - Vec3::new(0.0, 1.1, 0.0)) < 0.0));
        let m = go(&a, Vec2::new(-5.0, 0.1), Vec2::new(0.0, 0.1));
        assert!((m.x - -1.4).abs() < 0.01, "{m:?}");
    }

    /// #285: a lone triangle (not twinned) is met from its front only (FUN_00152d20): a wall
    /// facing -x holds a body coming from -x and lets one through from behind. A box wound to
    /// face out isn't stood on from inside (its bottom faces down), but is on its top.
    #[test]
    fn a_lone_face_is_met_from_its_front() {
        let wall = [[Vec3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 3.0, 2.0)],
                    [Vec3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 3.0, 2.0), Vec3::new(0.0, 3.0, -2.0)]];
        let a = arena_with(&wall, false);
        assert!(a.tris.iter().all(|t| t.front.x < -0.99));
        assert!((go(&a, Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0)).x - -0.4).abs() < 0.01);
        assert!(go(&a, Vec2::new(3.0, 0.0), Vec2::new(-3.0, 0.0)).distance(Vec2::new(-3.0, 0.0)) < 1e-4);
        let b = arena_with(&box_tris(Vec3::new(-1.0, 0.5, -1.0), Vec3::new(1.0, 1.0, 1.0)), false);
        assert_eq!(b.floor_below(0.2, 0.1, 0.8).map(|f| f.0), Some(0.0));
        assert_eq!(b.floor_below(0.2, 0.1, 1.2).map(|f| f.0), Some(1.0));
    }

    /// A rock under the step (0.5 m over the feet here) isn't a wall: the floor's to step up onto
    /// it. The underside of an overhang is a wall to a body under it: one wound down 57 degrees
    /// from flat (its normal's y between cos 130 and cos 65: steep, tested at the feet) stops a
    /// body walking at it where it reaches the body's height, 0.6 m out from its foot; so does one
    /// leaning 15 degrees out past upright, and a flat one (a ceiling) 1.2 m up.
    #[test]
    fn low_rocks_and_overhangs() {
        let a = arena_with(&box_tris(Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 0.3, 1.0)), true);
        assert!(go(&a, Vec2::new(-3.0, 0.0), Vec2::new(0.0, 0.0)).distance(Vec2::ZERO) < 1e-4);
        // (from x 0 at 1 m up to x -1 at 2.52 m up, over the whole of z -2..2)
        let (p, q, r, s) = (Vec3::new(0.0, 1.0, -2.0), Vec3::new(-1.0, 2.518, -2.0), Vec3::new(-1.0, 2.518, 2.0), Vec3::new(0.0, 1.0, 2.0));
        let b = arena_with(&[[p, r, q], [p, s, r]], false);
        let n = b.tris[0].front;
        assert!(n.y < -FLOOR_NORMAL && n.y > CEILING_NORMAL && n.x < 0.0 && !b.tris[0].floor_back, "{n:?}");
        let stop = go(&b, Vec2::new(-3.0, 0.0), Vec2::new(0.5, 0.0)).x;
        assert!((stop - (-0.9 / 1.518 - 0.4)).abs() < 0.01, "{stop}");
        // (from x 0 at the ground to x -0.8 at 3 m up: 15 degrees past upright)
        let lean = 15f32.to_radians().tan() * 3.0;
        let (p, q, r, s) = (Vec3::new(0.0, 0.0, -2.0), Vec3::new(-lean, 3.0, -2.0), Vec3::new(-lean, 3.0, 2.0), Vec3::new(0.0, 0.0, 2.0));
        let c = arena_with(&[[p, r, q], [p, s, r]], false);
        let n = c.tris[0].front;
        assert!(n.y < 0.0 && n.y > CEILING_NORMAL && n.x < 0.0, "{n:?}");
        let stop = go(&c, Vec2::new(-3.0, 0.0), Vec2::new(0.5, 0.0)).x;
        assert!((stop - (-lean * 1.9 / 3.0 - 0.4)).abs() < 0.01, "{stop}");
        // (a flat underside alone, facing down, x 0..2 at 1.2 m)
        let (p, q, r, s) = (Vec3::new(0.0, 1.2, -2.0), Vec3::new(2.0, 1.2, -2.0), Vec3::new(2.0, 1.2, 2.0), Vec3::new(0.0, 1.2, 2.0));
        let ceiling = arena_with(&[[p, q, r], [p, r, s]], false);
        assert!(ceiling.tris.iter().all(|t| t.front.y < -0.99));
        assert!((go(&ceiling, Vec2::new(-3.0, 0.0), Vec2::new(1.0, 0.0)).x - -0.4).abs() < 0.01);
    }

    /// A twinned slope rising along x over z -2..2 from x 0 at the ground, in parts: each part's
    /// end x and its angle (degrees). With the height of the ground at x.
    fn slope(parts: &[(f32, f32)]) -> (Arena, impl Fn(f32) -> f32) {
        let mut tris = vec![];
        let (mut x0, mut y0) = (0.0, 0.0);
        let mut heights = vec![];
        for &(x1, deg) in parts {
            let y1 = y0 + (x1 - x0) * deg.to_radians().tan();
            let (p, q) = (Vec3::new(x0, y0, -2.0), Vec3::new(x1, y1, -2.0));
            let z = Vec3::Z * 4.0;
            tris.extend([[p, q + z, q], [p, p + z, q + z]]);
            heights.push((x0, y0, x1, y1));
            (x0, y0) = (x1, y1);
        }
        let height = move |x: f32| heights.iter().find(|h| x <= h.2).or(heights.last()).map_or(0.0, |h| h.1 + (x - h.0) * (h.3 - h.1) / (h.2 - h.0));
        (arena_with(&tris, true), height)
    }

    /// A body walked up `a` from x `from` in 0.15 m moves, its feet on the slope at each start:
    /// where it ends after `moves`.
    fn walk_up(a: &Arena, height: &dyn Fn(f32) -> f32, from: f32, moves: usize) -> f32 {
        let mut x = from;
        for _ in 0..moves {
            let feet = height(x);
            x = a.move_body(Vec2::new(x, 0.0), Vec2::new(x + 0.15, 0.0), 0.4, feet, feet + 1.9, 0.5, None).x;
        }
        x
    }

    /// A body standing on a 55-degree slope (twinned, as the terrain comes) walks on up it: the
    /// twin facing down into the ground has the feet on its plane, and isn't met (PLANE_MARGIN).
    /// Nor where the slope eases ahead, 60 degrees to 53 or 55 to 52 to 45 (#285's review: held at
    /// the junction less its radius when such a twin was tested at the feet, as the game's split
    /// has it): the twins facing into the ground are the backs of floors stood on
    /// (`Tri::floor_back`), tested at the body's middle, which is over their plane.
    #[test]
    fn up_a_twinned_slope() {
        let (a, height) = slope(&[(3.0, 55.0)]);
        let feet = height(0.5);
        let to = a.move_body(Vec2::new(0.5, 0.0), Vec2::new(0.8, 0.0), 0.4, feet, feet + 1.9, 0.5, None);
        assert!(to.distance(Vec2::new(0.8, 0.0)) < 1e-4, "{to:?}");
        let (eases, height) = slope(&[(1.0, 60.0), (3.0, 53.0)]);
        let feet = height(0.8);
        assert!(eases.tris.iter().any(|t| t.front.y < -0.6 && (Vec3::new(0.8, feet, 0.0) - t.v[0]).dot(t.front) > PLANE_MARGIN));
        let to = eases.move_body(Vec2::new(0.8, 0.0), Vec2::new(1.0, 0.0), 0.4, feet, feet + 1.9, 0.5, None);
        assert!(to.distance(Vec2::new(1.0, 0.0)) < 1e-4, "{to:?}");
        let up = walk_up(&eases, &height, 0.3, 10);
        assert!((up - 1.8).abs() < 1e-3, "{up}");
        let (flank, height) = slope(&[(1.0, 55.0), (2.0, 52.0), (4.0, 45.0)]);
        let up = walk_up(&flank, &height, 0.3, 20);
        assert!((up - 3.3).abs() < 1e-3, "{up}");
    }

    /// The response (FUN_00150f90 / FUN_00168950 / FUN_00151170): the part into a wall is taken
    /// out, the rest kept; pressed into a corner, nothing's left.
    #[test]
    fn slide_off_walls() {
        let m = slide_off(Vec2::new(1.0, 1.0), &[Vec2::new(-1.0, 0.0)]);
        assert!(m.distance(Vec2::new(0.0, 1.0)) < 1e-6);
        assert_eq!(slide_off(Vec2::new(1.0, 1.0), &[Vec2::new(-1.0, 0.0), Vec2::new(0.0, -1.0)]), Vec2::ZERO);
        assert!(slide_off(Vec2::new(-1.0, 1.0), &[Vec2::new(-1.0, 0.0)]).distance(Vec2::new(-1.0, 1.0)) < 1e-6);
    }
}
