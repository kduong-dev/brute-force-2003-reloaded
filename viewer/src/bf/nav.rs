//! AI navigation over a level's world points (#334, the first slice of #242; viewer/docs/nav.md):
//! the graph the AI's routes run on, the route search, a character's own world point (where its
//! routes start), the route's first-entry drop and the walk along a route, as default.xbe's AI
//! does them. Every function cited is in viewer/docs/code-map.md ("AI"); the reading is
//! todo/242-ai-routes/leads.md, with the corrections nav.md lists. Nothing here moves a
//! character: a walk gives the point to steer for and the caller steers (bf_play's play_nav.rs
//! runs the waypoint triggers' valid bits, BF_ROUTE_LOG and the BF_TEST_ROUTE walk; bf_level's
//! BF_ROUTE prints a route).
//!
//!  - **The graph** (`Graph`): one node per world point of every kind (`Level::points`, the
//!    world's one list at world+0x110), its links (h_e3439383) one way each, as the data lists
//!    them (FUN_0020d220 adds no reverse), and the run-time flags of its +0x28 (`VALID`, `AVOID`
//!    ...). A waypoint trigger's `set-valid` sets or clears a point's `VALID` (FUN_00202ba0,
//!    `Graph::waypoint`); links never change.
//!  - **The search** (`Graph::search`, FUN_00059000): A* forward from the start along each
//!    point's own links. A binary heap on f; g grows by twice the link's 3D length, truncated to
//!    whole numbers at each step (the game keeps g through `cvttss2si`); h is the 3D distance to
//!    the goal; a neighbour must be `VALID` and not `AVOID`; a closed node is opened again on a
//!    lower g; no node cap; an optional radius round the start (GOAL_ATTACK's approach budget).
//!  - **A route** (`Graph::plan`, FUN_000416a0 -> FUN_0005a440): from a temporary start at the
//!    character's position linked to its own world point, to the destination's point; the path's
//!    first node (the temporary) is dropped, then its first entry too when the 0.8 m corridor
//!    from the character to the second is clear (FUN_00041320 -> FUN_000410a0).
//!  - **The own point** (`OwnPoint`, FUN_0014f860): kept fresh every frame: the nearest of its
//!    point and the valid points one or two links on, a deeper search every 0.25 s when that
//!    changes or the height is off, and beyond 20 m the nearest valid point of all
//!    (FUN_0014f7a0 -> FUN_000ab260).
//!  - **The walk** (`Walker`, FUN_00025730 / FUN_00026d80): the current entry is the target,
//!    reached within the caller's radius across (2 m), less at a corner whose next leg isn't
//!    clear (0.6), the point's own h_e6564e8a when it has one, at most 1 m inside a prism; at a
//!    plain point, the one after is taken at once when the corridor to it is clear.
//!
//! The corridor ray (FUN_0004ce30) is the caller's (`Clear`): the world's ray against the
//! terrain, placed geometry and the blockers, which passes through liquids (object type 22,
//! OT_WATERLAVAFLOW, is in its skip mask 0x435a1018). Nothing in the route code sees lava: the
//! graph's layout keeps the routes off it.

use std::collections::HashMap;

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::hash::h;
use super::level::WorldPoint;
use super::triggers::{act, sig, Trigger};

/// +0x28 bit 0: claimed, a point the follow's state 4 picked near the leader (FUN_000225d0); a
/// waypoint trigger's h_06d92de9 sets it (none in the data does).
pub const CLAIMED: u32 = 1 << 0;
/// +0x28 bit 1: some route's destination (FUN_00025730 sets it on a started route's last point,
/// FUN_00026d80 and FUN_0004e8f0 clear it).
pub const DESTINATION: u32 = 1 << 1;
/// +0x28 bit 2: valid, a point the search may expand (FUN_00059000), the own point's walks may
/// step onto (FUN_0004bf30) and FUN_000ab260 may pick; every point starts with it (FUN_00059b60)
/// and the waypoint triggers' set-valid switch it (FUN_00202ba0).
pub const VALID: u32 = WorldPoint::VALID;
/// +0x28 bit 3: avoid, never expanded (FUN_00059000); FUN_00041870 sets it round a squadmate in
/// the way (#335's), FUN_0004e8f0 clears it on a reset route's points.
pub const AVOID: u32 = 1 << 3;
/// +0x28 bit 4: a non-empty h_ed6a7490 (`WorldPoint::NO_SKIP`): the walk's look-ahead skip is
/// never taken from or to such a point (FUN_00026d80).
pub const NO_SKIP: u32 = WorldPoint::NO_SKIP;

/// The kinds of world point, each point's vtable +0x3c: what the route code tells apart. Read
/// from the world point factory FUN_000ee070, which makes each element type with its own
/// constructor, and each constructor's vtable's +0x3c (`mov eax, n; ret`; the plain point's
/// vtable 0x3a3f30 has 0x2bf0a0, `xor eax, eax`). The element names are the data's.
pub mod kind {
    /// `<world-point>` (type h_f95afb93, FUN_00059b60): the plain navigation point.
    pub const WORLD: u8 = 0;
    /// `<start-point>` (h_1bafdc0d, FUN_000ec910).
    pub const START: u8 = 1;
    /// `<hide-point>` (h_1038c10e, FUN_000ebf10).
    pub const HIDE: u8 = 2;
    /// `<sniper-point>` (h_1fc9ee4d, FUN_000ebe90).
    pub const SNIPER: u8 = 5;
    /// `<door-point>` (h_1ff1d816, FUN_000eb730): the walk hands it to FUN_00026820; no corner
    /// rule and no first-entry drop at one.
    pub const DOOR: u8 = 6;
    /// `<cover-point>` (h_126f47a3, FUN_000ec070): set-valid writes its +0x64 bit 5, not +0x28.
    pub const COVER: u8 = 10;
    /// `<patrol-point>` (h_efd397a1, FUN_000ec180) and `<patrol-first-point>` (h_0eef0757,
    /// FUN_000ec5a0): the walk hands them to FUN_00026940.
    pub const PATROL: u8 = 11;
    pub const PATROL_HEAD: u8 = 12;
    /// `<direction-point>` (h_0cec69df, FUN_000eb470).
    pub const DIRECTION: u8 = 16;
    /// h_16308887 (FUN_000ed0a0): set-valid writes a byte of its own (+0x90), not +0x28.
    pub const FLAGGED: u8 = 0x13;
    /// `<progress-point>` (h_f5c3ac30, FUN_000ec6e0).
    pub const PROGRESS: u8 = 24;
    /// h_f87c8847 (`<h_11d1b897>`, FUN_000ec9f0): a jump's take-off (FUN_00026520 jumps to a
    /// following 0x1b / 0x1c); no corner rule and no first-entry drop at one.
    pub const JUMP: u8 = 0x1a;
    /// h_e22441fa (`<h_e8f50262>`, FUN_000eca80): a landing (FUN_000267b0); as JUMP for the rules.
    pub const LANDING: u8 = 0x1b;
    /// h_085b36d2 (`<h_0ea2850c>`, FUN_000ecb50): a landing JUMP leads to.
    pub const LANDING_2: u8 = 0x1c;
    /// `<save-point>` (h_1815b5c2, FUN_000ec2d0): set-valid goes to the world (its vtable +0x88).
    pub const SAVE: u8 = 0x1d;
}

/// A point element type's kind (`kind`), from FUN_000ee070's switch; None for a type the factory
/// doesn't make (none in the game's 54 levels: every point type there is one of these).
pub fn kind_of(ty: u32) -> Option<u8> {
    Some(match ty {
        0xF95A_FB93 => kind::WORLD,
        0x1BAF_DC0D => kind::START,
        0x1038_C10E => kind::HIDE,
        0x1FC9_EE4D => kind::SNIPER,
        0x1FF1_D816 => kind::DOOR,
        0x1EE1_B020 => 7,
        0x0F79_22F8 => 8,
        0x0282_A91F => 9,
        0x126F_47A3 => kind::COVER,
        0xEFD3_97A1 => kind::PATROL,
        0x0EEF_0757 => kind::PATROL_HEAD,
        0x0A60_658A | 0xF49F_ED25 => 13,
        0xEDB1_1148 => 14,
        0xE538_F1B2 => 15,
        0x0CEC_69DF => kind::DIRECTION,
        0x10B3_AC7B => 17,
        0xE3B2_8B0A => 18,
        0x1630_8887 => kind::FLAGGED,
        0xF93E_CBC1 => 20,
        0x00B9_C638 | 0x0375_0EF9 => 21,
        0xF4BD_2EC2 => 22,
        0x1367_7B6D => 23,
        0xF5C3_AC30 => kind::PROGRESS,
        0x1814_0CF6 => 25,
        0xF87C_8847 => kind::JUMP,
        0xE224_41FA => kind::LANDING,
        0x085B_36D2 => kind::LANDING_2,
        0x1815_B5C2 => kind::SAVE,
        _ => return None,
    })
}

/// A door point or a jump's (`kind::DOOR`, `JUMP`, `LANDING`): FUN_00041320 keeps the first entry
/// and FUN_00026d80 skips the corner rule at one (both test 6 and 0x1a-0x1b).
fn special(kind: u8) -> bool {
    kind == kind::DOOR || (kind::JUMP..=kind::LANDING).contains(&kind)
}

/// The corridor's half-width (m): FUN_00041320 and FUN_00026d80 hand FUN_000410a0 0.8
/// (0x3f4ccccd), and FUN_00026a50 offsets its two rays 0.8.
pub const CORRIDOR: f32 = 0.8;
/// FUN_000410a0 casts its rays the straight length + 0.1 at most.
const CORRIDOR_SLACK: f32 = 0.1;
/// FUN_00026a50's rays: the length + 0.001 at most, both ends 0.08 above the points.
const CORNER_SLACK: f32 = 0.001;
const CORNER_LIFT: f32 = 0.08;
/// FUN_000410a0 casts nothing (and says clear) for a length under 1.42e-14 ([0x3a5150]) or a
/// sideways part under 1.19e-7 (both the game's floats).
const TINY_LENGTH: f32 = 1.421_085_5e-14;
const TINY_SIDE: f32 = 1.192_092_9e-7;

/// The arrival radius every route walk's caller passes FUN_00026d80 (m, across): 2.0 for the
/// follow, MOVE_OUT, PATROL and RETURN_HOME (FUN_0002b490, FUN_00034690, FUN_00045fa0 ...).
pub const ARRIVE: f32 = 2.0;
/// The radius at the corner point (brain+0x9dc): at most 0.6 (0x3f19999a, `minss`).
pub const CORNER_RADIUS: f32 = 0.6;
/// A point's own h_e6564e8a replaces the radius when it's over 0 and the radius over this
/// (0x3a4f74).
const OWN_RADIUS_OVER: f32 = 0.5;
/// Inside one of the level's prisms (brain+0x3c4 bit 10, FUN_0004cb60): at most 1.0.
pub const PRISM_RADIUS: f32 = 1.0;
/// The corner rule runs for a radius over 0.7 (0x3a5154).
const CORNER_OVER: f32 = 0.7;
/// Under this radius (1.0, 0x3a4f78) a point the character passed beside counts as reached
/// (FUN_00025870).
const PASS_UNDER: f32 = 1.0;
/// FUN_00025870 takes no direction for a move under this (0.01 m).
const PASS_STILL: f32 = 0.01;

/// The own point's upkeep (FUN_0014f860): its point is kept by the walks within 20 m (400, the
/// square, 3D: [0x3a5528]); beyond, FUN_0014f7a0.
const OWN_NEAR: f32 = 20.0;
/// FUN_0014f7a0 keeps a point within 5 m (25, the square: [0x3a5030]).
const OWN_KEEP: f32 = 5.0;
/// The height the own point is compared at: the character's position less 0.25 ([0x3a5028]).
const OWN_DROP: f32 = 0.25;
/// The deeper search waits 0.25 s since the last ([0x3a5028] again: timer +0x258).
const OWN_RECHECK: f32 = 0.25;
/// A point more than 1.5 below that height or 0.5 above it is off ([0x3a4fb4], [0x3a4f7c]).
const OWN_UP: f32 = 1.5;
const OWN_DOWN: f32 = -0.5;
/// After 1 s (timer +0x254, [0x3a4f78]) a point more than 2.5 below or 1.5 above
/// ([0x3a5044], [0x3a52f4]) asks for the widest search.
const OWN_DEEP_AFTER: f32 = 1.0;
const OWN_FAR_UP: f32 = 2.5;
const OWN_FAR_DOWN: f32 = -1.5;
/// The deeper searches' depths (links) and the widest one's sight tries (FUN_0014f5c0's
/// arguments: 6 and 0, or 10 and 6).
const OWN_HOPS: u32 = 6;
const OWN_WIDE_HOPS: u32 = 10;
const OWN_TRIES: usize = 6;
/// With +0x250 bit 6 a refresh is forced once its timer (+0x25c) passes 2 s ([0x3a5008]).
const OWN_FORCE_AFTER: f32 = 2.0;
/// FUN_0014f5c0's sight ray: the distance + 0.1 at most.
const OWN_RAY_SLACK: f32 = 0.1;

/// The world's ray as the route code asks it (FUN_0004ce30 -> FUN_0004cd10): true when the two
/// points are no farther apart than the length given and nothing the corridor query meets
/// (the terrain, placed geometry, the blockers and hinters; not characters, items or liquids:
/// its object-type mask 0x435a1018) lies between them.
pub type Clear<'a> = &'a dyn Fn(Vec3, Vec3, f32) -> bool;

/// One world point in the graph (`Level::points`' element of the same index).
#[derive(Clone, Debug)]
pub struct Node {
    pub name: u32,
    /// its vtable +0x3c (`kind`); 0xff for a type the factory doesn't make (none in the data),
    /// which stays out of the graph: no links to it or from it, never valid
    pub kind: u8,
    pub at: Vec3,
    /// +0x24 (`WorldPoint::flags`)
    pub flags: u32,
    /// h_e6564e8a (+0x20): its own arrival radius, -1 none
    pub radius: f32,
    /// the points its h_e3439383 names, by node index, in the data's order
    pub links: Vec<u32>,
    /// +0x28: `CLAIMED`, `DESTINATION`, `VALID`, `AVOID`, `NO_SKIP`
    pub state: u32,
    /// a cover point's +0x64 bit 5 (its constructor FUN_000ec070 sets it, `| 0x33`): what its
    /// set-valid writes (FUN_00202ba0); true for the other kinds, which have none
    pub cover: bool,
}

/// A level's world points as the route code sees them (see the module notes).
#[derive(Clone, Debug, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    by_name: HashMap<u32, u32>,
}

/// Where a search starts (FUN_0005a440's from side): a point, or a temporary point at a position
/// with one link, to a point (a character's place: FUN_00059b60 on the stack, linked by
/// FUN_00038440 to its char+0x24c).
#[derive(Clone, Copy, Debug)]
pub enum Start {
    Point(u32),
    Temp { at: Vec3, link: u32 },
}

/// What a route goes to (the to-place of FUN_000416a0): a point (FUN_00058b60's place, bit 0
/// clear), or a character (FUN_0011dfa0's place: its position and its own world point, bit 0
/// set): a character is routed to its point, not its position (FUN_0005a440).
#[derive(Clone, Copy, Debug)]
pub enum Dest {
    Point(u32),
    Character { at: Vec3, point: Option<u32> },
}

/// One entry of a route (the 0x18-byte place, FUN_00058c50): a point, or a bare position (the
/// one-entry route to a character whose point the walker already has, FUN_00022d70).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    pub at: Vec3,
    pub point: Option<u32>,
}

/// A route found (`Graph::plan`): its entries, start to end, and the entry FUN_00041320 dropped
/// (the character's own point, when it could go straight to the second), for the log.
#[derive(Clone, Debug, Default)]
pub struct Route {
    pub entries: Vec<Entry>,
    pub dropped: Option<u32>,
}

/// Why `Graph::plan` found no route.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoRoute {
    /// the character has no world point: the game links a temporary to up to 6 visible points
    /// within 20 m (FUN_00059f70), not done here (its per-kind radius and height limits, vtable
    /// +0x44 / +0x4c, aren't read)
    NoStart,
    /// the destination has no point: FUN_00059f70's temporary has links out only, which the
    /// forward search can't reach (leads §6, inferred)
    NoGoal,
    /// the heap ran dry (FUN_00059000 returns 0): every way there blocked by invalid or avoided
    /// points, or one-way links
    Unreachable,
}

/// A search's record of a point (FUN_00058e10's 0x18 bytes in DAT_004a5900: parent, g, h, f,
/// open, closed), made the first time the point is met.
#[derive(Clone, Copy, Debug, Default)]
struct Record {
    parent: Option<u32>,
    g: f32,
    h: f32,
    f: f32,
    open: bool,
    closed: bool,
    made: bool,
}

/// A search's result (`Graph::search`): the path, start to goal, and how many points it opened
/// and popped (for BF_ROUTE_LOG's cost).
#[derive(Clone, Debug, Default)]
pub struct Found {
    pub path: Vec<u32>,
    pub popped: usize,
    pub met: usize,
}

impl Graph {
    /// The graph of `points` (`Level::points`): a node each, in the same order (a point's index
    /// is its node's), the links by name (a name that isn't a point, or a point of no kind, is
    /// dropped: none in the data). The first of two points with one name takes it (none in the
    /// data either).
    pub fn new(points: &[WorldPoint]) -> Graph {
        let mut by_name = HashMap::new();
        for (i, p) in points.iter().enumerate() {
            if kind_of(p.kind).is_some() {
                by_name.entry(p.name).or_insert(i as u32);
            }
        }
        let nodes = points.iter().map(|p| {
            let kind = kind_of(p.kind);
            Node {
                name: p.name,
                kind: kind.unwrap_or(0xff),
                at: p.at,
                flags: p.flags,
                radius: p.radius,
                links: if kind.is_some() { p.links.iter().filter_map(|n| by_name.get(n).copied()).collect() } else { vec![] },
                state: if kind.is_some() { p.state } else { 0 },
                cover: true,
            }
        }).collect();
        Graph { nodes, by_name }
    }

    /// The node of the point named `name`.
    pub fn find(&self, name: u32) -> Option<u32> {
        self.by_name.get(&name).copied()
    }

    /// Whether point `i` has `VALID`.
    pub fn valid(&self, i: u32) -> bool {
        self.nodes.get(i as usize).is_some_and(|n| n.state & VALID != 0)
    }

    /// FUN_00202ba0, a waypoint trigger's OPERATE on its point `i`: `claim` (its h_06d92de9) not -1
    /// sets `CLAIMED` to it; `valid` (its set-valid) not -1 sets, by the point's kind, a cover
    /// point's +0x64 bit 5 (`Node::cover`), kind 0x13's byte +0x90 (not kept), a save point's
    /// world call (vtable +0x88, not read), or else `VALID`. What it did, for the log.
    pub fn waypoint(&mut self, i: u32, valid: i64, claim: i64) -> Waypoint {
        let Some(n) = self.nodes.get_mut(i as usize) else { return Waypoint::None };
        if claim != -1 {
            n.state = if claim != 0 { n.state | CLAIMED } else { n.state & !CLAIMED };
        }
        if valid == -1 {
            return if claim != -1 { Waypoint::Claim } else { Waypoint::None };
        }
        let on = valid != 0;
        match n.kind {
            kind::COVER => {
                n.cover = on;
                Waypoint::Cover(on)
            }
            kind::FLAGGED => Waypoint::Other(n.kind),
            kind::SAVE => Waypoint::Other(n.kind),
            _ => {
                n.state = if on { n.state | VALID } else { n.state & !VALID };
                Waypoint::Valid(on)
            }
        }
    }

    /// FUN_000ab260: the valid point nearest `at` (3D) of the whole list, any kind; the first in
    /// the list's order of two as near. FUN_0014f7a0's pick for a character beyond its point.
    pub fn nearest_valid(&self, at: Vec3) -> Option<u32> {
        let mut best = (None, f32::MAX);
        for (i, n) in self.nodes.iter().enumerate() {
            let d = n.at.distance_squared(at);
            if n.state & VALID != 0 && d < best.1 {
                best = (Some(i as u32), d);
            }
        }
        best.0
    }

    /// FUN_0004bf30 without marks (FUN_0014f860's walk): the nearest to `at` (3D, squared) of
    /// point `i` and, `hops` deep, its valid neighbours, not stepping straight back to `parent`.
    fn nearest_near(&self, at: Vec3, i: u32, parent: Option<u32>, hops: u32, best: &mut (Option<u32>, f32)) {
        let n = &self.nodes[i as usize];
        let d = n.at.distance_squared(at);
        if d < best.1 {
            *best = (Some(i), d);
        }
        if hops == 0 {
            return;
        }
        for &k in &n.links {
            if self.nodes[k as usize].state & VALID != 0 && Some(k) != parent {
                self.nearest_near(at, k, Some(i), hops - 1, best);
            }
        }
    }

    /// FUN_0004bf30 with marks (FUN_0014f5c0's): depth first from point `i`, each point once (the
    /// first time it's met, whatever the depth: its record marks it), `hops` deep through valid
    /// neighbours not straight back to `parent`; every point met into `met` with its squared
    /// distance to `at`, and the nearest into `best`.
    fn nearest_marked(&self, at: Vec3, i: u32, parent: Option<u32>, hops: u32, best: &mut (Option<u32>, f32), seen: &mut [bool],
                      met: &mut Vec<(u32, f32)>) {
        if seen[i as usize] {
            return;
        }
        let n = &self.nodes[i as usize];
        let d = n.at.distance_squared(at);
        if d < best.1 {
            *best = (Some(i), d);
        }
        if hops == 0 {
            return;
        }
        seen[i as usize] = true;
        met.push((i, d));
        for &k in &n.links {
            if self.nodes[k as usize].state & VALID != 0 && Some(k) != parent {
                self.nearest_marked(at, k, Some(i), hops - 1, best, seen, met);
            }
        }
    }

    /// FUN_0014f5c0 `(slot, character, current, hops, tries)`: from point `from`, `hops` deep
    /// (`nearest_marked`, from the character's position `at`): with `tries` 0 the nearest point
    /// met; else the points met nearest first (FUN_0004e150 sorts their squared distances), the
    /// first of `tries` the sight ray from (x, y - 0.25, z) reaches (`clear`, the distance + 0.1
    /// at most). None when it finds none.
    fn deeper(&self, at: Vec3, from: u32, hops: u32, tries: usize, clear: Clear) -> Option<u32> {
        let mut seen = vec![false; self.nodes.len()];
        let mut met = vec![];
        let mut best = (None, f32::MAX);
        self.nearest_marked(at, from, None, hops, &mut best, &mut seen, &mut met);
        if tries == 0 {
            return best.0;
        }
        met.sort_by(|a, b| a.1.total_cmp(&b.1));
        let eye = at - Vec3::Y * OWN_DROP;
        met.iter().take(tries).map(|&(i, _)| i).find(|&i| {
            let p = self.nodes[i as usize].at;
            clear(eye, p, p.distance(eye) + OWN_RAY_SLACK)
        })
    }

    /// The position of a search node: a point's, or the temporary start's (`Start::Temp`).
    fn pos(&self, i: u32, start: &Start) -> Vec3 {
        match (start, i as usize == self.nodes.len()) {
            (Start::Temp { at, .. }, true) => *at,
            _ => self.nodes[i as usize].at,
        }
    }

    /// FUN_00059000 `(start, goal, &list)`: the A* (see the module notes). `within`: the radius
    /// round the start a neighbour must lie in (DAT_004021a8, -1.0 in the binary: None; GOAL_ATTACK's
    /// approach sets it for one search, FUN_0002e800). The path, start to goal (a temporary start
    /// is the node after the last point), or None when the heap runs dry.
    ///
    /// As the game's: g is truncated to a whole number at each step (0x591e1, `cvttss2si`, and
    /// `fild` back), so a link under 0.5 m costs nothing; f = h + g with h the straight 3D
    /// distance (FUN_00058ef0) and each link 2 x its 3D length (FUN_00058f50, whose 0.25 x for
    /// h_09aeb0fa points is behind the debug byte 0x4021b4, 0: off); a neighbour needs `VALID`
    /// unless the records still hold only the start (the start's first neighbours met before any
    /// passes are let through without it: a character whose own point is invalid still gets
    /// out), and never `AVOID`; a point open or closed is taken again only for a strictly lower
    /// g, a closed one opened again, but an open one isn't moved in the heap (its f changes in
    /// place, FUN_00059920 isn't called for it). The heap is the game's STL one (`push_heap`,
    /// `pop_heap`), so equal f's come off in its order.
    pub fn search(&self, start: Start, goal: u32, within: Option<f32>) -> Option<Found> {
        let n = self.nodes.len();
        if goal as usize >= n {
            return None;
        }
        let first = match start {
            Start::Point(i) if (i as usize) < n => i,
            Start::Temp { link, .. } if (link as usize) < n => n as u32,
            _ => return None,
        };
        let temp_links = match start {
            Start::Temp { link, .. } => vec![link],
            _ => vec![],
        };
        let links = |i: u32| if i as usize == n { &temp_links[..] } else { &self.nodes[i as usize].links[..] };
        let start_at = self.pos(first, &start);
        let goal_at = self.nodes[goal as usize].at;
        let limit = within.filter(|r| *r > 0.0).map(|r| r * r);
        let mut rec = vec![Record::default(); n + 1];
        let mut made = 1usize;
        let h0 = start_at.as_dvec3().distance(goal_at.as_dvec3());
        rec[first as usize] = Record { parent: None, g: 0.0, h: h0 as f32, f: h0 as f32, open: true, closed: false, made: true };
        let mut heap: Vec<u32> = vec![];
        push_heap(&mut heap, first, |i| rec[i as usize].f);
        let mut found = Found::default();
        while let Some(&top) = heap.first() {
            pop_heap(&mut heap, |i| rec[i as usize].f);
            found.popped += 1;
            rec[top as usize].open = false;
            if top == goal {
                let mut path = vec![goal];
                let mut at = goal;
                while let Some(p) = rec[at as usize].parent {
                    path.push(p);
                    at = p;
                    if path.len() > n + 1 {
                        break;
                    }
                }
                path.reverse();
                found.path = path;
                found.met = made;
                return Some(found);
            }
            let here = self.pos(top, &start);
            for &k in links(top) {
                let node = &self.nodes[k as usize];
                if node.state & AVOID != 0 || (node.state & VALID == 0 && made != 1) {
                    continue;
                }
                if limit.is_some_and(|r2| node.at.distance_squared(start_at) > r2) {
                    continue;
                }
                if !rec[k as usize].made {
                    rec[k as usize].made = true;
                    made += 1;
                }
                let cost = 2.0 * here.as_dvec3().distance(node.at.as_dvec3());
                let g = ((cost + rec[top as usize].g as f64) as f32) as i32 as f32;
                let r = rec[k as usize];
                if (r.open || r.closed) && g >= r.g {
                    continue;
                }
                let h = node.at.as_dvec3().distance(goal_at.as_dvec3());
                let r = &mut rec[k as usize];
                r.parent = Some(top);
                r.g = g;
                r.h = h as f32;
                r.f = (h + g as f64) as f32;
                r.closed = false;
                if !r.open {
                    r.open = true;
                    push_heap(&mut heap, k, |i| rec[i as usize].f);
                }
            }
            rec[top as usize].closed = true;
        }
        None
    }

    /// FUN_000416a0 `(to, from, brain; route)` with FUN_0005a440: a route from a character at
    /// `at` whose own world point is `own` (char+0x24c) to `to`, or why there's none.
    ///  1. the same point on both sides (both none too, as the game compares them): the
    ///     one-entry route to `to` (the point, or the character's position);
    ///  2. (FUN_000413f0, a bit-15 destination linked to the nearest valid point within 40 m of
    ///     it: with its filter as read that's the destination itself whenever it's valid, and an
    ///     invalid one stays unexpandable, so nothing changes; not done);
    ///  3. the search from a temporary start at `at` linked to `own` to the destination's point
    ///     (`search`, `within` as there), the temporary dropped (FUN_0005a690): an entry per point;
    ///  4. FUN_00041320: with two entries or more and the first point not a door or a jump's,
    ///     the first dropped when the corridor (`corridor_blocked`) from the character, at that
    ///     point's height, to the second is clear: it goes straight to the second.
    pub fn plan(&self, at: Vec3, own: Option<u32>, to: Dest, within: Option<f32>, clear: Clear) -> Result<(Route, Found), NoRoute> {
        let goal = match to {
            Dest::Point(p) => Some(p),
            Dest::Character { point, .. } => point,
        };
        if own == goal {
            let entry = match to {
                Dest::Point(p) => Entry { at: self.nodes[p as usize].at, point: Some(p) },
                Dest::Character { at, .. } => Entry { at, point: None },
            };
            return Ok((Route { entries: vec![entry], dropped: None }, Found::default()));
        }
        let own = own.ok_or(NoRoute::NoStart)?;
        let goal = goal.ok_or(NoRoute::NoGoal)?;
        let found = self.search(Start::Temp { at, link: own }, goal, within).ok_or(NoRoute::Unreachable)?;
        let mut entries: Vec<Entry> = found.path.iter().skip(1).map(|&i| Entry { at: self.nodes[i as usize].at, point: Some(i) }).collect();
        let mut dropped = None;
        if entries.len() >= 2 && entries[0].point.is_none_or(|p| !special(self.nodes[p as usize].kind)) {
            let from = Vec3::new(at.x, entries[0].at.y, at.z);
            if !corridor_blocked(from, entries[1].at, CORRIDOR, clear) {
                dropped = entries[0].point;
                entries.remove(0);
            }
        }
        Ok((Route { entries, dropped }, found))
    }
}

/// A waypoint trigger (`<h_10ecb459>`, vtable 0x395e20) as its parser 0x202af0 reads it after the
/// base's: `world-point` (h_0fe604ae, through FUN_000ee560: the point's handle +0x40, the point
/// +0x44), `set-valid` (h_e77cb64e, +0x48) and h_06d92de9 (+0x4c). -1 is "leave it"; a trigger
/// without a number keeps -1 (the constructor's isn't read; every one in the data has both).
/// The tutorial has 62, 31 of them with a world-point; sdm_e34 16; e34 21. Its OPERATE (the
/// handler 0x202c50) is `Graph::waypoint`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaypointTrigger {
    pub point: u32,
    pub valid: i64,
    pub claim: i64,
}

impl WaypointTrigger {
    /// The class's element.
    pub const TAG: u32 = 0x10EC_B459;

    /// Trigger `t`'s, if it's a waypoint trigger.
    pub fn of(t: &Trigger) -> Option<WaypointTrigger> {
        if t.tag != Self::TAG {
            return None;
        }
        let e = &t.element;
        let int = |k: u32| e.attr(k).and_then(|v| v.as_i64().or(v.as_f32().map(|f| f as i64))).unwrap_or(-1);
        Some(WaypointTrigger {
            point: e.attr(h("world-point")).and_then(|v| v.as_hash()).filter(|&n| n != h("")).unwrap_or(0),
            valid: int(h("set-valid")),
            claim: int(0x06D9_2DE9),
        })
    }
}

impl Graph {
    /// The waypoint triggers that act as the level starts, applied as bf_play's trigger runtime
    /// applies them in its first frame: each enabled one's entries on its own FIRST_UPDATE (signal
    /// 13 with its iD, FUN_001fe660) whose action is OPERATE. For bf_level's route dump, which
    /// runs no triggers. The points they changed, and how.
    pub fn level_start(&mut self, triggers: &[Trigger]) -> Vec<(u32, Waypoint)> {
        let mut out = vec![];
        for t in triggers.iter().filter(|t| t.enabled && !t.dead) {
            let Some(w) = WaypointTrigger::of(t) else { continue };
            let acts = t.entries.iter().any(|e| e.signal == sig::FIRST_UPDATE && e.match_id == t.id && e.limit > 0 && e.action == act::OPERATE);
            if let (true, Some(i)) = (acts, self.find(w.point)) {
                let did = self.waypoint(i, w.valid, w.claim);
                if did != Waypoint::None {
                    out.push((i, did));
                }
            }
        }
        out
    }
}

/// What a waypoint trigger's OPERATE did (`Graph::waypoint`), for the log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Waypoint {
    None,
    Claim,
    /// `VALID` set (true) or cleared
    Valid(bool),
    /// a cover point's +0x64 bit 5 (not the graph's)
    Cover(bool),
    /// kind 0x13's byte or a save point's world call: not kept
    Other(u8),
}

/// FUN_001611d0 then FUN_00059920: `v` onto the end of the heap and up while its parent's f is
/// over its own (the STL's push_heap with a "greater" compare: a min-heap on f).
fn push_heap(heap: &mut Vec<u32>, v: u32, f: impl Fn(u32) -> f32) {
    heap.push(v);
    let mut hole = heap.len() - 1;
    while hole > 0 {
        let parent = (hole - 1) / 2;
        if f(heap[parent]) <= f(v) {
            break;
        }
        heap[hole] = heap[parent];
        hole = parent;
    }
    heap[hole] = v;
}

/// FUN_00059850: the root off the heap, as the STL's pop_heap: the last element held, the root
/// moved to the end, the hole sifted down to a leaf by the smaller child (FUN_000599b0: the left
/// one only when its f is strictly under the right's), the held one pushed up from there
/// (FUN_00059920), the end dropped.
fn pop_heap(heap: &mut Vec<u32>, f: impl Fn(u32) -> f32) {
    let len = heap.len();
    if len > 1 {
        let value = heap[len - 1];
        heap[len - 1] = heap[0];
        let n = len - 1;
        let mut hole = 0;
        let mut child = 2;
        while child < n {
            if f(heap[child - 1]) < f(heap[child]) {
                child -= 1;
            }
            heap[hole] = heap[child];
            hole = child;
            child = 2 * child + 2;
        }
        if child == n {
            heap[hole] = heap[n - 1];
            hole = n - 1;
        }
        while hole > 0 {
            let parent = (hole - 1) / 2;
            if f(heap[parent]) <= f(value) {
                break;
            }
            heap[hole] = heap[parent];
            hole = parent;
        }
        heap[hole] = value;
    }
    heap.pop();
}

/// FUN_000410a0 `(half-width; EDX from, ECX to)`: whether the corridor from `a` to `b` is blocked:
/// two rays (`clear`) offset `half` either side across (in x and z), each the straight length +
/// 0.1 at most; blocked when either meets something. A length under 1.42e-14, or a way that's
/// all but straight up, casts nothing: clear. (The game's sideways offset comes out `half` long
/// whatever its first normalisation, which takes the reciprocal square root of the length, not
/// of its square: only that last test feels it.)
pub fn corridor_blocked(a: Vec3, b: Vec3, half: f32, clear: Clear) -> bool {
    let d = b - a;
    let len = d.length();
    if len <= TINY_LENGTH {
        return false;
    }
    let side = Vec2::new(d.z, -d.x);
    if side.length_squared() / len <= TINY_SIDE {
        return false;
    }
    let off = side.normalize() * half;
    let off = Vec3::new(off.x, 0.0, off.y);
    let max = len + CORRIDOR_SLACK;
    !clear(a + off, b + off, max) || !clear(a - off, b - off, max)
}

/// FUN_00026a50 `(brain, point, next)`: the corner rule's test, whether the corridor from the
/// character (at `point`'s height) to `next` is blocked: two rays offset 0.8 m across, both ends
/// 0.08 m above their points, each the length + 0.001 at most. Clear without a point or a next.
fn corner_blocked(me: Vec3, point: Option<Vec3>, next: Option<Vec3>, clear: Clear) -> bool {
    let (Some(p), Some(n)) = (point, next) else { return false };
    let a = Vec3::new(me.x, p.y + CORNER_LIFT, me.z);
    let b = n + Vec3::Y * CORNER_LIFT;
    let side = Vec2::new(b.z - a.z, -(b.x - a.x));
    if side.length_squared() <= 0.0 {
        return false;
    }
    let off = side.normalize() * CORRIDOR;
    let off = Vec3::new(off.x, 0.0, off.y);
    let max = a.distance(b) + CORNER_SLACK;
    !clear(a + off, b + off, max) || !clear(a - off, b - off, max)
}

/// FUN_00025870 `(radius; EAX from, ESI to, ECX the target)`: whether the target, across, lies
/// within `r` of the line the character moved along from `from` to `to`, short of `to`'s end
/// (the projection under the move's length; behind `from` counts too, as the game has no lower
/// bound). `from` is the character's vtable +0x10 position, read as where it was at its last
/// update (inferred).
fn passed(from: Vec2, to: Vec2, target: Vec2, r: f32) -> bool {
    let d = to - from;
    let len = d.length();
    let dir = if len <= PASS_STILL { Vec2::ZERO } else { d / len };
    let v = target - from;
    let along = v.dot(dir);
    if along >= len {
        return false;
    }
    let side = v.length_squared() - along * along;
    side >= 0.0 && side < r * r
}

/// A character's own world point (char+0x24c), its flags (+0x250) and timers (+0x254 the deep
/// search's, +0x258 the re-check's, +0x25c the forced refresh's), kept by `update` (FUN_0014f860):
/// a route's start, and a character destination's point.
#[derive(Clone, Copy, Debug, Default)]
pub struct OwnPoint {
    pub point: Option<u32>,
    pub flags: u32,
    pub deep: f32,
    pub check: f32,
    pub forced: f32,
}

/// What `OwnPoint::update` did, for the log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OwnChange {
    /// the point picked by the near walk (one or two links on)
    Near(u32),
    /// the deeper search's (6 links), or the widest one's (10 links, the first of 6 in sight)
    Deep(u32),
    Wide(u32),
    /// FUN_0014f7a0's: the nearest valid point of all
    Far(u32),
}

impl OwnPoint {
    /// +0x250 bit 0: the point is wanted (FUN_0011dfa0, the first time a place of the character
    /// is made; FUN_00017b60 at a spawn; FUN_00025730 at a route's start): only then is it kept.
    pub const WANTED: u32 = 1 << 0;
    /// +0x250 bits 1-2: the near walk runs (bit 2 set at a spawn by FUN_00017b60; bit 1 a
    /// one-frame ask, cleared at each update's end).
    pub const ASKED: u32 = 1 << 1;
    pub const WALK: u32 = 1 << 2;
    /// +0x250 bits 3-5: a frame count; the near walk goes two links on when it's 0, one else.
    const COUNT: u32 = 0x38;
    /// +0x250 bit 6: refresh at once once +0x25c passes 2 s (FUN_0014f7a0's pick).
    pub const FORCE: u32 = 1 << 6;

    /// A spawned character's (FUN_00017b60: +0x250 |= 1 | 4), its point not picked yet.
    pub fn spawned() -> OwnPoint {
        OwnPoint { flags: Self::WANTED | Self::WALK, ..Default::default() }
    }

    /// FUN_0014f860, one frame (`dt` s) of a living character with a brain at `at` (its position:
    /// the skeleton's origin, the hips, as the game's vtable +0x40 transform), `jumping` while
    /// its motion state is a jump's (FUN_00126a90: MT_JUMP_CROUCH, _FALL, _LAND, _LAUNCH, 20-22
    /// and 24). Not wanted: nothing. Its timers run; with no point, a forced refresh due, or its
    /// point over 20 m away (3D), FUN_0014f7a0 (`keep_near`). Else, with the walk on and not
    /// jumping: the nearest to `at` of its point and the valid points one link on (two every
    /// eighth frame, the +0x250 count at 0); when that's another point, or its height off (more
    /// than 1.5 below `at` - 0.25 or 0.5 above), and 0.25 s have passed since the last deeper
    /// search: that search instead (6 links deep from the current point, the nearest; after 1 s,
    /// and the height off by over 2.5 / 1.5: 10 links deep, the first of the 6 nearest in sight),
    /// which keeps the point when it finds none; within the 0.25 s the near walk's point is
    /// taken as it is. The count moves on and the one-frame ask clears every frame. (The game
    /// skips it all on a deathmatch arena, mode 0x120785ef: the caller's.)
    pub fn update(&mut self, graph: &Graph, at: Vec3, dt: f32, jumping: bool, clear: Clear) -> Option<OwnChange> {
        if self.flags & Self::WANTED == 0 {
            return None;
        }
        self.deep += dt;
        self.check += dt;
        self.forced += dt;
        let mut forced = false;
        if self.flags & Self::FORCE != 0 && self.forced > OWN_FORCE_AFTER {
            self.flags &= !Self::FORCE;
            self.forced = 0.0;
            forced = true;
        }
        let near = self.point.filter(|&p| !forced && graph.nodes[p as usize].at.distance_squared(at) <= OWN_NEAR * OWN_NEAR);
        let change = match near {
            None => self.keep_near(graph, at),
            Some(current) if self.flags & (Self::ASKED | Self::WALK) != 0 && !jumping => {
                let hops = if self.flags & Self::COUNT == 0 { 2 } else { 1 };
                let mut best = (None, f32::MAX);
                graph.nearest_near(at, current, None, hops, &mut best);
                best.0.and_then(|found| {
                    let dy = at.y - OWN_DROP - graph.nodes[found as usize].at.y;
                    let mut pick = Some(found);
                    let mut how = OwnChange::Near(found);
                    if self.check > OWN_RECHECK && (found != current || dy > OWN_UP || dy < OWN_DOWN) {
                        self.check = 0.0;
                        let wide = self.deep > OWN_DEEP_AFTER && (dy > OWN_FAR_UP || dy < OWN_FAR_DOWN);
                        if wide {
                            self.deep = 0.0;
                        }
                        let (hops, tries) = if wide { (OWN_WIDE_HOPS, OWN_TRIES) } else { (OWN_HOPS, 0) };
                        pick = graph.deeper(at, current, hops, tries, clear);
                        how = match pick {
                            Some(p) if wide => OwnChange::Wide(p),
                            Some(p) => OwnChange::Deep(p),
                            None => how,
                        };
                    }
                    let p = pick?;
                    let was = self.point;
                    self.point = Some(p);
                    (was != Some(p)).then_some(how)
                })
            }
            Some(_) => None,
        };
        // (0x14fa91: bit 1 cleared, bits 3-5 one more, the rest kept)
        let f = self.flags & !Self::ASKED;
        self.flags = ((((f & !7) + 8) ^ f) & Self::COUNT) ^ f;
        change
    }

    /// FUN_0014f7a0 (EAX the position, EBX the slot): wanted, the point is kept within 5 m of
    /// `at` (3D), else replaced by the nearest valid point of all (FUN_000ab260).
    pub fn keep_near(&mut self, graph: &Graph, at: Vec3) -> Option<OwnChange> {
        if self.flags & Self::WANTED == 0 {
            return None;
        }
        if self.point.is_some_and(|p| graph.nodes[p as usize].at.distance_squared(at) < OWN_KEEP * OWN_KEEP) {
            return None;
        }
        let was = self.point;
        self.point = graph.nearest_valid(at);
        self.point.filter(|&p| was != Some(p)).map(OwnChange::Far)
    }
}

/// A route walked (the route record, `route+0x..`, with the brain's two fields it uses: the
/// steering target brain+0x478 and the corner point brain+0x9dc), started by `Walker::start`
/// (FUN_00025730) and stepped by `Walker::step` (FUN_00026d80).
#[derive(Clone, Debug, Default)]
pub struct Walker {
    /// the entries (+4..+0xc)
    pub entries: Vec<Entry>,
    /// the current entry (+0x10), -1 none
    pub index: i32,
    /// +0x14: FUN_00025730's third argument 0 clears it (a route's end then looks round 20 m, not
    /// done)
    pub keep: bool,
    /// +0x18: the best squared distance to the current target so far (FLT_MAX at each advance;
    /// the follow's stuck handling reads it, #335)
    pub best: f32,
    /// +0x1c: seconds without progress (the follow's, #335)
    pub stuck: f32,
    /// +0x20: done (the stuck handling's: the next step resets and ends)
    pub done: bool,
    /// +0x24: the point the look-ahead skipped
    pub skipped: Option<u32>,
    /// brain+0x9dc: the point the corner rule holds the character to 0.6 m of
    pub corner: Option<u32>,
    /// brain+0x478: the steering target
    pub target: Vec3,
}

/// One step of a walk (`Walker::step`): its return value and what happened, for the caller and
/// the log.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Step {
    /// FUN_00026d80's 1: the walk goes on toward `target`; 0: it's over (the end, or done)
    pub walking: bool,
    pub target: Vec3,
    /// the point reached this step and its kind (the game hands kinds 6, 11 / 12, 0x1a, 0x1b to
    /// FUN_00026820, FUN_00026940, FUN_00026520, FUN_000267b0: the caller's), and whether the
    /// entry was a bare position
    pub reached: Option<(Option<u32>, u8)>,
    /// the corner rule held the character at this point (its next leg blocked)
    pub corner: Option<u32>,
    /// the look-ahead skipped this point
    pub skipped: Option<u32>,
    /// the radius the current entry was tested with
    pub radius: f32,
}

impl Walker {
    /// FUN_00025730 `(route, ?, keep)` (EAX the brain): index 0 (-1 for an empty route), best
    /// FLT_MAX, no skip, not done; the steering target the first entry's position; +0x14 cleared
    /// unless `keep`; the last point marked `DESTINATION`; and a character standing on a jump's
    /// take-off (its own point `own` of kind 0x1a) whose route starts at a landing (0x1b / 0x1c)
    /// gets its own point put first, so the jump is made.
    pub fn start(graph: &mut Graph, route: &Route, keep: bool, own: Option<u32>) -> Walker {
        let mut entries = route.entries.clone();
        if let (Some(o), Some(Some(first))) = (own, entries.first().map(|e| e.point)) {
            let k = graph.nodes[first as usize].kind;
            if graph.nodes[o as usize].kind == kind::JUMP && (k == kind::LANDING || k == kind::LANDING_2) {
                entries.insert(0, Entry { at: graph.nodes[o as usize].at, point: Some(o) });
            }
        }
        if let Some(Some(last)) = entries.last().map(|e| e.point) {
            graph.nodes[last as usize].state |= DESTINATION;
        }
        let target = entries.first().map_or(Vec3::ZERO, |e| Self::position(graph, e));
        Walker { index: if entries.is_empty() { -1 } else { 0 }, entries, keep, best: f32::MAX, target, ..Default::default() }
    }

    /// FUN_000233d0: an entry's position: its point's (live), else its own.
    fn position(graph: &Graph, e: &Entry) -> Vec3 {
        e.point.map_or(e.at, |p| graph.nodes[p as usize].at)
    }

    /// The current entry's point (FUN_00026d80 reads +0x14 of entry +0x10).
    pub fn current(&self) -> Option<u32> {
        usize::try_from(self.index).ok().and_then(|i| self.entries.get(i)).and_then(|e| e.point)
    }

    /// FUN_0004e8f0: the reset: the last point's `DESTINATION` and every entry's `AVOID` cleared,
    /// no current entry, the entries gone, not done.
    pub fn reset(&mut self, graph: &mut Graph) {
        if let Some(Some(last)) = self.entries.last().map(|e| e.point) {
            graph.nodes[last as usize].state &= !DESTINATION;
        }
        for e in &self.entries {
            if let Some(p) = e.point {
                graph.nodes[p as usize].state &= !AVOID;
            }
        }
        self.index = -1;
        self.best = f32::MAX;
        self.skipped = None;
        self.entries.clear();
        self.done = false;
    }

    /// FUN_00026d80 `(brain, route, &target, ?, radius)`, one update of a character at `me` (its
    /// position; `last` where it was at its previous update), `prism` while it stands in one of
    /// the level's prisms (brain+0x3c4 bit 10), with the caller's `radius` (`ARRIVE`):
    ///  1. done: reset, over;
    ///  2. the target is the current entry's position; the radius at most 0.6 at the corner
    ///     point, the point's own when it has one over 0 (and the radius over 0.5), at most 1 in a
    ///     prism;
    ///  3. reached: within the radius across (strictly), or, for a radius under 1, the target
    ///     passed beside (`passed`); not reached, on toward it;
    ///  4. the corner rule: at a point that isn't a door's or a jump's, with a radius over 0.7,
    ///     the corridor from the character to the next point (`corner_blocked`) blocked: this is
    ///     the corner point, on toward it (the next step closes to 0.6 m);
    ///  5. the point is reached (its kind's handler: the caller's);
    ///  6. the next entry: with one, the target it; at a plain point (kind 0) without `NO_SKIP`
    ///     followed by a point without it, the corridor from the character (at the reached
    ///     point's height) to the one after clear (`corridor_blocked`): that one is taken at once,
    ///     the skipped point kept (+0x24) and this step's target left the skipped one's;
    ///  7. past the end: the last point's `DESTINATION` cleared, reset, over.
    pub fn step(&mut self, graph: &mut Graph, me: Vec3, last: Vec3, prism: bool, radius: f32, clear: Clear) -> Step {
        let mut out = Step { radius, ..Default::default() };
        let Some(i) = usize::try_from(self.index).ok().filter(|&i| i < self.entries.len() && !self.done) else {
            self.reset(graph);
            return out;
        };
        let e = self.entries[i];
        let target = Self::position(graph, &e);
        self.target = target;
        out.target = target;
        let mut r = radius;
        if let Some(p) = e.point {
            if Some(p) == self.corner {
                r = r.min(CORNER_RADIUS);
            }
            let own = graph.nodes[p as usize].radius;
            if own > 0.0 && r > OWN_RADIUS_OVER {
                r = own;
            }
        }
        if prism {
            r = r.min(PRISM_RADIUS);
        }
        out.radius = r;
        let (dx, dz) = (me.x - target.x, me.z - target.z);
        let mut reached = dx * dx + dz * dz < r * r;
        if r < PASS_UNDER && !reached {
            reached = passed(last.xz(), me.xz(), target.xz(), r);
        }
        if !reached {
            out.walking = true;
            return out;
        }
        let kind = e.point.map_or(0, |p| graph.nodes[p as usize].kind);
        if !special(kind) && r > CORNER_OVER {
            let next = self.entries.get(i + 1).and_then(|n| n.point).map(|n| graph.nodes[n as usize].at);
            if corner_blocked(me, e.point.map(|p| graph.nodes[p as usize].at), next, clear) {
                self.corner = e.point;
                out.corner = e.point;
                out.walking = true;
                return out;
            }
        }
        self.corner = None;
        out.reached = Some((e.point, kind));
        self.index += 1;
        self.best = f32::MAX;
        self.skipped = None;
        let i = i + 1;
        if let Some(n) = self.entries.get(i).copied() {
            let target = Self::position(graph, &n);
            self.target = target;
            out.target = target;
            let plain = n.point.filter(|&c| graph.nodes[c as usize].state & NO_SKIP == 0 && graph.nodes[c as usize].kind == kind::WORLD);
            let after = self.entries.get(i + 1).and_then(|a| a.point).filter(|&a| graph.nodes[a as usize].state & NO_SKIP == 0);
            if let (Some(c), Some(a)) = (plain, after) {
                let height = e.point.map_or(me.y, |p| graph.nodes[p as usize].at.y);
                if !corridor_blocked(Vec3::new(me.x, height, me.z), graph.nodes[a as usize].at, CORRIDOR, clear) {
                    self.index += 1;
                    self.best = f32::MAX;
                    self.skipped = Some(c);
                    out.skipped = Some(c);
                }
            }
            out.walking = true;
            return out;
        }
        if let Some(p) = e.point {
            graph.nodes[p as usize].state &= !DESTINATION;
        }
        self.reset(graph);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain world point named `name` at (x, 0, z) linking to `links` (by name).
    fn point(name: u32, x: f32, z: f32, links: &[u32]) -> WorldPoint {
        WorldPoint { name, kind: 0xF95A_FB93, at: Vec3::new(x, 0.0, z), flags: 0xffff_4000, radius: -1.0, links: links.to_vec(), state: VALID }
    }

    /// Nothing in the way.
    fn open(_: Vec3, _: Vec3, _: f32) -> bool {
        true
    }

    /// The names of a path's nodes.
    fn names(g: &Graph, path: &[u32]) -> Vec<u32> {
        path.iter().map(|&i| g.nodes[i as usize].name).collect()
    }

    /// The links are one way, as stored: 1 -> 2 -> 3 and 3 -> 1 only. From 1 the search reaches 3
    /// through 2; from 3 it reaches 2 only through 1 (no link 3 -> 2, though 2 -> 3 exists).
    #[test]
    fn links_are_one_way() {
        let g = Graph::new(&[point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[3]), point(3, 20.0, 0.0, &[1])]);
        let found = g.search(Start::Point(0), 2, None).unwrap();
        assert_eq!(names(&g, &found.path), [1, 2, 3]);
        let found = g.search(Start::Point(2), 1, None).unwrap();
        assert_eq!(names(&g, &found.path), [3, 1, 2]);
        // a point nothing links to can't be reached
        let g = Graph::new(&[point(1, 0.0, 0.0, &[]), point(2, 10.0, 0.0, &[1])]);
        assert!(g.search(Start::Point(0), 1, None).is_none());
    }

    /// A search from a temporary start just beside point `i` (as `Graph::plan` starts one).
    fn beside(g: &Graph, i: u32) -> Start {
        Start::Temp { at: g.nodes[i as usize].at + Vec3::new(-0.5, 0.0, 0.0), link: i }
    }

    /// An invalid point isn't expanded: the way round is taken; with the only ways through
    /// invalid or avoided points there's no route.
    #[test]
    fn invalid_and_avoided_points_are_gone_round() {
        // 1 - 2 - 4 straight (2 x 10 m), 1 - 3 - 4 round (2 x 14.1 m)
        let pts = [point(1, 0.0, 0.0, &[2, 3]), point(2, 10.0, 0.0, &[1, 4]), point(3, 10.0, 10.0, &[1, 4]), point(4, 20.0, 0.0, &[2, 3])];
        let mut g = Graph::new(&pts);
        assert_eq!(names(&g, &g.search(beside(&g, 0), 3, None).unwrap().path[1..]), [1, 2, 4]);
        assert_eq!(g.waypoint(1, 0, -1), Waypoint::Valid(false));
        assert_eq!(names(&g, &g.search(beside(&g, 0), 3, None).unwrap().path[1..]), [1, 3, 4]);
        g.nodes[2].state |= AVOID;
        assert!(g.search(beside(&g, 0), 3, None).is_none());
        assert_eq!(g.waypoint(1, 1, -1), Waypoint::Valid(true));
        assert_eq!(names(&g, &g.search(beside(&g, 0), 3, None).unwrap().path[1..]), [1, 2, 4]);
    }

    /// The start's first neighbours met before any passes go through without `VALID` (the records
    /// hold only the start): a character whose own point is invalid still gets out of it, and a
    /// search from a point takes its first neighbour invalid or not; a later invalid neighbour
    /// isn't taken.
    #[test]
    fn the_start_gets_out_of_an_invalid_point() {
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3]), point(3, 20.0, 0.0, &[2])];
        let mut g = Graph::new(&pts);
        g.nodes[0].state &= !VALID;
        assert_eq!(names(&g, &g.search(beside(&g, 0), 2, None).unwrap().path[1..]), [1, 2, 3]);
        g.nodes[1].state &= !VALID;
        assert!(g.search(beside(&g, 0), 2, None).is_none());
        // from point 1 itself its first neighbour, 2, passes though it's invalid
        assert_eq!(names(&g, &g.search(Start::Point(0), 2, None).unwrap().path), [1, 2, 3]);
    }

    /// A closed point is opened again when a lower g reaches it later. S -> X directly costs 20
    /// (10 m); a chain of links under 0.5 m (each trunc(0.89) = 0) leaves S away from the goal G,
    /// turns 11.2 m out and comes back to X. X comes off the heap (f 20 + 90) before the chain's
    /// far end (0 + 111) and is closed; the chain reaches it with g 0, opens it again, and the
    /// route to G goes the chain's way.
    #[test]
    fn closed_points_are_opened_again() {
        let (s, x, goal) = (1, 2, 3);
        let mut chain = vec![];
        for k in 1..=28 {
            chain.push(Vec3::new(-0.4 * k as f32, 0.0, 0.2));
        }
        for k in 0..=52 {
            chain.push(Vec3::new(-11.2 + 0.4 * k as f32, 0.0, -0.2));
        }
        let mut pts = vec![point(s, 0.0, 0.0, &[x, 100]), point(x, 10.0, 0.0, &[goal]), point(goal, 100.0, 0.0, &[])];
        for (k, c) in chain.iter().enumerate() {
            let next = if k + 1 < chain.len() { 101 + k as u32 } else { x };
            pts.push(WorldPoint { at: *c, ..point(100 + k as u32, 0.0, 0.0, &[next]) });
        }
        let g = Graph::new(&pts);
        let found = g.search(Start::Point(0), 2, None).unwrap();
        assert_eq!(found.path.len(), 1 + chain.len() + 2);
        assert_eq!(names(&g, &found.path[found.path.len() - 2..]), [x, goal]);
        // every point off the heap once but X, twice: closed, opened again, closed
        assert_eq!(found.popped, 1 + chain.len() + 3);
    }

    /// g is truncated to a whole number at each step: S -> G straight, 4.6 m, costs trunc(9.2) =
    /// 9; S -> A -> G, two links of 2.49 m, trunc(4.98) = 4 each: 8. The longer way wins (it
    /// wouldn't untruncated: 9.96 against 9.2).
    #[test]
    fn g_is_whole_numbers() {
        let z = (2.49f32 * 2.49 - 2.3 * 2.3).sqrt();
        let pts = [point(1, 0.0, 0.0, &[3, 2]), point(2, 2.3, z, &[3]), point(3, 4.6, 0.0, &[])];
        let g = Graph::new(&pts);
        assert_eq!(names(&g, &g.search(Start::Point(0), 2, None).unwrap().path), [1, 2, 3]);
    }

    /// The radius limit round the start (DAT_004021a8): a point outside it isn't taken.
    #[test]
    fn the_radius_limit() {
        let pts = [point(1, 0.0, 0.0, &[2, 3]), point(2, 30.0, 0.0, &[4]), point(3, 5.0, 8.0, &[4]), point(4, 10.0, 0.0, &[])];
        let g = Graph::new(&pts);
        assert_eq!(names(&g, &g.search(Start::Point(0), 3, None).unwrap().path), [1, 3, 4]);
        assert_eq!(names(&g, &g.search(Start::Point(0), 3, Some(12.0)).unwrap().path), [1, 3, 4]);
        assert!(g.search(Start::Point(0), 3, Some(9.0)).is_none());
    }

    /// FUN_000416a0's plan: the temporary start dropped, the character's own point dropped too
    /// when the corridor to the second entry is clear, kept when it isn't; the same point on both
    /// sides gives the one entry.
    #[test]
    fn plan_drops_the_first_entry_when_clear() {
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3]), point(3, 20.0, 0.0, &[2])];
        let g = Graph::new(&pts);
        let at = Vec3::new(1.0, 0.0, 1.0);
        let (route, _) = g.plan(at, Some(0), Dest::Point(2), None, &open).unwrap();
        assert_eq!(route.entries.iter().map(|e| e.point).collect::<Vec<_>>(), [Some(1), Some(2)]);
        assert_eq!(route.dropped, Some(0));
        let wall = |_: Vec3, _: Vec3, _: f32| false;
        let (route, _) = g.plan(at, Some(0), Dest::Point(2), None, &wall).unwrap();
        assert_eq!(route.entries.iter().map(|e| e.point).collect::<Vec<_>>(), [Some(0), Some(1), Some(2)]);
        let (route, _) = g.plan(at, Some(2), Dest::Point(2), None, &open).unwrap();
        assert_eq!(route.entries, [Entry { at: pts[2].at, point: Some(2) }]);
        assert_eq!(g.plan(at, None, Dest::Point(2), None, &open).unwrap_err(), NoRoute::NoStart);
    }

    /// The walk: each point reached within 2 m across, the look-ahead skipping a plain point
    /// whose next leg is clear, none through a NO_SKIP point, the corner rule holding the
    /// character to 0.6 m where the next leg is blocked, the end resetting the route.
    #[test]
    fn walking_a_route() {
        let mut pts = vec![point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[3]), point(3, 20.0, 0.0, &[4]), point(4, 30.0, 0.0, &[])];
        let mut g = Graph::new(&pts);
        let route = Route { entries: (0..4).map(|i| Entry { at: g.nodes[i].at, point: Some(i as u32) }).collect(), dropped: None };
        let mut w = Walker::start(&mut g, &route, false, None);
        assert!(g.nodes[3].state & DESTINATION != 0);
        // far from the first: on toward it
        let s = w.step(&mut g, Vec3::new(-5.0, 0.0, 0.0), Vec3::new(-5.1, 0.0, 0.0), false, ARRIVE, &open);
        assert!(s.walking && s.reached.is_none() && s.target == pts[0].at);
        // within 2 m of it: reached; the next (2, plain) skipped as the leg to 3 is clear
        let s = w.step(&mut g, Vec3::new(-1.5, 0.0, 0.0), Vec3::new(-1.6, 0.0, 0.0), false, ARRIVE, &open);
        assert_eq!((s.reached, s.skipped), (Some((Some(0), 0)), Some(1)));
        assert_eq!(w.index, 2);
        // the end: 3 then 4 (no skip past the last), and the reset
        let s = w.step(&mut g, Vec3::new(19.0, 0.0, 0.0), Vec3::new(18.9, 0.0, 0.0), false, ARRIVE, &open);
        assert_eq!((s.reached, s.skipped), (Some((Some(2), 0)), None));
        let s = w.step(&mut g, Vec3::new(29.0, 0.0, 0.0), Vec3::new(28.9, 0.0, 0.0), false, ARRIVE, &open);
        assert!(!s.walking && s.reached == Some((Some(3), 0)));
        assert!(g.nodes[3].state & DESTINATION == 0 && w.index == -1 && w.entries.is_empty());
        // NO_SKIP on point 2: no skip
        pts[1].state |= NO_SKIP;
        let mut g = Graph::new(&pts);
        let mut w = Walker::start(&mut g, &route, false, None);
        let s = w.step(&mut g, Vec3::new(-1.5, 0.0, 0.0), Vec3::new(-1.6, 0.0, 0.0), false, ARRIVE, &open);
        assert_eq!((s.reached, s.skipped, w.index), (Some((Some(0), 0)), None, 1));
        // the corner rule: the leg on from point 2 blocked; within 2 m of it the character is
        // held to 0.6 m, then it's reached
        let wall = |_: Vec3, _: Vec3, _: f32| false;
        let s = w.step(&mut g, Vec3::new(8.5, 0.0, 1.0), Vec3::new(8.4, 0.0, 1.0), false, ARRIVE, &wall);
        assert_eq!((s.corner, s.reached), (Some(1), None));
        let s = w.step(&mut g, Vec3::new(9.0, 0.0, 0.5), Vec3::new(8.9, 0.0, 0.5), false, ARRIVE, &wall);
        assert!(s.radius == CORNER_RADIUS && s.reached.is_none());
        let s = w.step(&mut g, Vec3::new(9.6, 0.0, 0.2), Vec3::new(9.5, 0.0, 0.2), false, ARRIVE, &wall);
        assert_eq!(s.reached, Some((Some(1), 0)));
    }

    /// The pass test (FUN_00025870): a target the move went past within the radius, short of the
    /// move's end, counts; one ahead of it doesn't.
    #[test]
    fn passing_beside_a_point() {
        assert!(passed(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(1.0, 0.4), 0.6));
        assert!(!passed(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(1.0, 0.7), 0.6));
        assert!(!passed(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(3.0, 0.0), 0.6));
    }

    /// The own point: the nearest valid point of all at first (FUN_0014f7a0), then the near walk
    /// along the links as the character moves; beyond 20 m the nearest of all again.
    #[test]
    fn the_own_point_follows_the_character() {
        let pts = [point(1, 0.0, 0.0, &[2]), point(2, 10.0, 0.0, &[1, 3]), point(3, 20.0, 0.0, &[2]), point(4, 60.0, 0.0, &[])];
        let g = Graph::new(&pts);
        let mut own = OwnPoint::spawned();
        assert_eq!(own.update(&g, Vec3::new(1.0, 0.25, 0.0), 0.1, false, &open), Some(OwnChange::Far(0)));
        // moved to 2's side: the near walk (one link on) sees 2, another point, and the re-check's
        // 0.25 s have passed (0.1 + 0.2): the deeper search (6 links) decides, and picks it
        let c = own.update(&g, Vec3::new(9.0, 0.25, 0.0), 0.2, false, &open);
        assert_eq!(c, Some(OwnChange::Deep(1)));
        // within 0.25 s of that, the near walk's pick is taken as it is
        let c = own.update(&g, Vec3::new(19.0, 0.25, 0.0), 0.1, false, &open);
        assert_eq!(c, Some(OwnChange::Near(2)));
        // far from every point but 4: FUN_0014f7a0
        let c = own.update(&g, Vec3::new(58.0, 0.25, 0.0), 0.1, false, &open);
        assert_eq!(c, Some(OwnChange::Far(3)));
    }

    /// The heap is the STL's: after pushes and pops in any order the pops come off by f, lowest
    /// first.
    #[test]
    fn the_heap_pops_lowest_f_first() {
        let f = [5.0f32, 3.0, 9.0, 1.0, 7.0, 3.0, 0.5, 8.0];
        let mut heap = vec![];
        for i in 0..f.len() as u32 {
            push_heap(&mut heap, i, |k| f[k as usize]);
        }
        let mut out = vec![];
        while let Some(&top) = heap.first() {
            pop_heap(&mut heap, |k| f[k as usize]);
            out.push(f[top as usize]);
        }
        assert_eq!(out, [0.5, 1.0, 3.0, 3.0, 5.0, 7.0, 8.0, 9.0]);
    }
}
