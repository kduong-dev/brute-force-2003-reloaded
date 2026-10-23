//! The static scenery every level places (#185): what the test map's scenery menu
//! (play_testscenery.rs) offers, read from the level archives without loading their levels
//! (`scan_all`), and a picked piece's archive loaded on its own to build it from
//! (`Game::load_scenery`).
//!
//! Which placements are static scenery: a level file's `<game-object>` whose object type
//! (h_ff8051ee) has `<base object-type>` 2, OT_STATIC_GEOMETRY by the game's OT_ name table
//! (0x3be87c, {length, name, value} entries: 0 OT_BODY, 3 OT_CHARACTER, 23 OT_COMPOUND_TYPE ...),
//! and a mesh archetype (`mesh-name`). The game fixes such an object's physics bodies in place (its
//! parts on fixed joints, h_0fb850ce, at 100 x the type's mass) and never puts it on the world's
//! list of objects that move (FUN_001590a0: object types 2, 26 OT_PLATFORM and 31
//! OT_WORLDBUTTON). Left out of those:
//!  - the types that break: a non-empty debris list (h_197caf14, `Game::breakable`), queued when
//!    their hitpoints run out (FUN_00157260): the object menu's kind (play_scenery.rs);
//!  - the doors and gates: an archetype with a part on a sliding joint (h_fd8f670c), which only
//!    the door machinery opens (level_scene.rs `Door`).
//!
//! Every other model a level places has an element of its own (characters, pickups, weapons,
//! liquids h_04366a6a, world buttons h_16f22d4b, platforms, turrets), and its other
//! game-objects are OT_BODY (object-type 0, loose bodies: 77 in all). Counted in the 53 level
//! files: 23,670 game-objects, 19,671 of them static scenery (3,823 break, 99 are doors), of 675
//! object types; 2 (mp3) to 80 (m03_b) types a level, 2,412 counted level by level.
//!
//! A level's static scenery is drawn from its own archive (each of the 53 checked): its objects,
//! materials, textures and collision surfaces, but for two of common's types (placed in m02_a,
//! m04_a, m04_c, m05_b and m05_y: common's objects, as the game looks types up in common's
//! library first, the level file's `<data-libraries>` "common <level>") and 16 references to
//! common's textures (m01_a, m05_a, m05_b).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use bevy::math::Vec3;

use super::archive::Archive;
use super::bxml::{self, Element, SchemaSet};
use super::character::{Game, H_PHYSICS, H_PHYSICS_NAME};
use super::hash::h;

/// A placement's object type (`<game-object h_ff8051ee=..>`, level.rs's H_TYPE).
const H_TYPE: u32 = 0xFF80_51EE;
/// `<base object-type>` OT_STATIC_GEOMETRY (the OT_ name table 0x3be87c; FUN_001590a0's 2).
pub const OT_STATIC_GEOMETRY: i64 = 2;
/// A type's debris list (h_197caf14): it breaks (character.rs `ObjectType::debris`).
const H_DEBRIS_LIST: u32 = 0x197C_AF14;
/// An archetype's parts (`<h_e487418a><PART part-name archetype-name>`) and the joints that place
/// them (`<h_151ace78 child-part><h_002cbac5 Type><parent-point><child-point>`), as
/// weapon.rs reads them.
const H_PARTS: u32 = 0xE487_418A;
const H_JOINT: u32 = 0x151A_CE78;
/// A sliding joint's type: door and gate leaves (weapon.rs's SLIDING_JOINT).
const SLIDING_JOINT: u32 = 0xFD8F_670C;
/// An archetype's bounding radius (m), `h_1ab09530` (0 on a compound: its parts carry theirs).
const H_RADIUS: u32 = 0x1AB0_9530;
/// At most this many archives are read at once (`scan_all`): each holds its level file parsed,
/// a few MB to tens of MB, so the scan's peak stays bounded on a many-core PC.
const SCAN_THREADS: usize = 8;

/// A static scenery type one level places (see the module's rule).
#[derive(Clone, Debug)]
pub struct Piece {
    /// its object type (`<base name>`)
    pub kind: u32,
    /// its mesh archetype
    pub archetype: u32,
    /// how many the level places
    pub placed: u32,
    /// its parts (1 for an archetype of one mesh)
    pub parts: u32,
    /// how far across it is (m): twice its bounding radius about its origin, each part's
    /// radius (h_1ab09530) plus how far its joint puts it out
    pub size: f32,
    /// whether a part names a collision surface (without one it's walked and shot through, as
    /// in its level: grass, hanging cables)
    pub solid: bool,
    /// its archetype is common's (two types, see the module's notes): built from common.tgz,
    /// not from the level's archive
    pub common: bool,
}

/// An objecttypes entry as the scan reads it: its `<base object-type>`, mesh archetype and
/// whether it breaks.
#[derive(Clone, Copy)]
struct TypeInfo {
    object_type: i64,
    mesh: u32,
    breaks: bool,
}

/// Object types and archetypes by name: common's, which every level falls back on.
#[derive(Default)]
struct Library {
    types: HashMap<u32, TypeInfo>,
    archetypes: HashMap<u32, Element>,
}

/// An element's `name` (0 without one).
fn name_of(e: &Element) -> u32 {
    e.attr(h("name")).and_then(|v| v.as_hash()).unwrap_or(0)
}

/// An objecttypes file's entries that carry `<base object-type>`: (type name, what the scan
/// needs). The mesh sits on the entry (`<h_e275fb80 mesh-name>`) or on its base.
fn types_of(root: &Element) -> Vec<(u32, TypeInfo)> {
    let mut out = vec![];
    for t in root.walk() {
        let Some(b) = t.child(h("base")).filter(|b| b.attr(h("object-type")).is_some()) else { continue };
        let name = name_of(b);
        if name == 0 || name == h("") {
            continue;
        }
        let mesh = [t, b].iter().find_map(|e| e.attr(h("mesh-name")).and_then(|v| v.as_hash())).filter(|&m| m != h("")).unwrap_or(0);
        out.push((name, TypeInfo {
            object_type: b.attr(h("object-type")).and_then(|v| v.as_i64()).unwrap_or(-1),
            mesh,
            breaks: t.child(H_DEBRIS_LIST).is_some_and(|d| !d.children.is_empty()),
        }));
    }
    out
}

/// An objects file's archetypes by name.
fn archetypes_of(root: &Element) -> HashMap<u32, &Element> {
    root.children_named(h("archetype-library")).flat_map(|l| l.children_named(h("archetype"))).map(|a| (name_of(a), a)).collect()
}

/// What the scan says of an archetype: parts, size, collision, and whether a part slides (a door).
struct Shape {
    parts: u32,
    size: f32,
    solid: bool,
    sliding: bool,
}

/// An archetype's `Shape`, its parts looked up with `find`.
fn shape<'a>(arch: &'a Element, find: &dyn Fn(u32) -> Option<&'a Element>) -> Shape {
    let radius = |a: &Element| a.attr(H_RADIUS).and_then(|v| v.as_f32()).unwrap_or(0.0);
    let solid = |a: &Element| a.child(H_PHYSICS).and_then(|p| p.attr(H_PHYSICS_NAME)).and_then(|v| v.as_hash()).is_some_and(|s| s != 0 && s != h(""));
    let parts: Vec<(Option<u32>, u32)> = arch.child(H_PARTS).map(|p| p.children_named(h("PART"))
        .map(|e| (e.attr(h("part-name")).and_then(|v| v.as_hash()), e.attr(h("archetype-name")).and_then(|v| v.as_hash()).unwrap_or(0)))
        .collect()).unwrap_or_default();
    if parts.is_empty() {
        return Shape { parts: 1, size: 2.0 * radius(arch), solid: solid(arch), sliding: false };
    }
    let joints: Vec<&Element> = arch.walk().into_iter().filter(|e| e.name == H_JOINT).collect();
    let mut out = Shape { parts: parts.len() as u32, size: 0.0, solid: false, sliding: false };
    for (part, pa) in parts {
        let Some(a) = find(pa) else { continue };
        let joint = joints.iter().find(|j| j.attr(h("child-part")).and_then(|v| v.as_hash()) == part).and_then(|j| j.children.first());
        let point = |n: &str| joint.and_then(|d| d.child(h(n))).and_then(|e| e.text.as_ref()).map(|t| t.floats())
            .filter(|f| f.len() >= 3).map_or(Vec3::ZERO, |f| Vec3::new(f[0], f[1], f[2]));
        out.size = out.size.max(2.0 * ((point("parent-point") + point("child-point")).length() + radius(a)));
        out.solid |= solid(a);
        out.sliding |= joint.and_then(|d| d.attr(h("Type"))).and_then(|v| v.as_hash()) == Some(SLIDING_JOINT);
    }
    out
}

/// common's object types and archetypes (common.tgz's objecttypes-common.xmb and
/// objects-common.xmb), for the levels to fall back on.
fn read_common(schemas: &SchemaSet, data_dir: &Path) -> Result<Library, String> {
    let keep = |n: &str| n.ends_with(".xmb") && (n.starts_with("objects-") || n.starts_with("objecttypes"));
    let ar = Archive::open_until(&data_dir.join("common.tgz"), keep, 2)?;
    let mut lib = Library::default();
    for n in ar.find("objecttypes", ".xmb") {
        let root = bxml::parse(ar.get(&n).unwrap_or(&[]), schemas).map_err(|e| format!("{n}: {e}"))?;
        for (k, t) in types_of(&root) {
            lib.types.entry(k).or_insert(t);
        }
    }
    for n in ar.find("objects-", ".xmb") {
        let root = bxml::parse(ar.get(&n).unwrap_or(&[]), schemas).map_err(|e| format!("{n}: {e}"))?;
        for (k, a) in archetypes_of(&root) {
            lib.archetypes.entry(k).or_insert_with(|| a.clone());
        }
    }
    Ok(lib)
}

/// One level archive's static scenery (see the module's rule), the largest first: its level
/// file, objects and objecttypes (read up to the objecttypes, the last of the three in every
/// level archive; its models, textures and sounds aren't kept). Common's types and archetypes
/// first, as the game looks them up (FUN_00160da0, libraries "common <level>"). An archive with
/// no level file has none.
fn scan_archive(schemas: &SchemaSet, data_dir: &Path, archive: &str, common: &Library) -> Result<Vec<Piece>, String> {
    let keep = |n: &str| n.ends_with(".xmb") && (n.starts_with("levels-") || n.starts_with("objects-") || n.starts_with("objecttypes"));
    let ar = Archive::open_until(&data_dir.join(format!("{archive}.tgz")), keep, 3)?;
    let parse = |n: &str| bxml::parse(ar.get(n).unwrap_or(&[]), schemas).map_err(|e| format!("{n}: {e}"));
    let Some(level) = ar.find("levels-", ".xmb").first().map(|n| parse(n)).transpose()? else { return Ok(vec![]) };
    let objects: Vec<Element> = ar.find("objects-", ".xmb").iter().map(|n| parse(n)).collect::<Result<_, _>>()?;
    let mut types: HashMap<u32, TypeInfo> = HashMap::new();
    for n in ar.find("objecttypes", ".xmb") {
        for (k, t) in types_of(&parse(&n)?) {
            types.entry(k).or_insert(t);
        }
    }
    let archetypes: HashMap<u32, &Element> = objects.iter().flat_map(archetypes_of).collect();
    let find = |a: u32| common.archetypes.get(&a).or_else(|| archetypes.get(&a).copied());
    // how many of each type the level places
    let mut placed: HashMap<u32, u32> = HashMap::new();
    for e in level.walk().into_iter().filter(|e| e.name == h("game-object")) {
        if let Some(k) = e.attr(H_TYPE).and_then(|v| v.as_hash()) {
            *placed.entry(k).or_default() += 1;
        }
    }
    let mut out = vec![];
    for (kind, count) in placed {
        let Some(t) = common.types.get(&kind).or_else(|| types.get(&kind)).copied() else { continue };
        if t.object_type != OT_STATIC_GEOMETRY || t.breaks || t.mesh == 0 {
            continue;
        }
        let Some(arch) = find(t.mesh) else { continue };
        let s = shape(arch, &find);
        if s.sliding {
            continue;
        }
        out.push(Piece { kind, archetype: t.mesh, placed: count, parts: s.parts, size: s.size, solid: s.solid,
                         common: common.archetypes.contains_key(&t.mesh) });
    }
    out.sort_by(|a, b| b.size.total_cmp(&a.size).then(a.kind.cmp(&b.kind)));
    Ok(out)
}

/// Every archive of `archives`' static scenery (`scan_archive`), in the list's order: common
/// read first, then the archives on up to SCAN_THREADS threads, each taking the next. Per
/// archive its pieces or why it couldn't be read.
pub fn scan_all(schemas: &SchemaSet, data_dir: &Path, archives: &[String]) -> Vec<(String, Result<Vec<Piece>, String>)> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let common = match read_common(schemas, data_dir) {
        Ok(c) => c,
        Err(e) => return archives.iter().map(|a| (a.clone(), Err(format!("common: {e}")))).collect(),
    };
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, SCAN_THREADS.min(archives.len().max(1)));
    let mut found: Vec<Option<Result<Vec<Piece>, String>>> = (0..archives.len()).map(|_| None).collect();
    std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads).map(|_| s.spawn(|| {
            let mut mine = vec![];
            loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some(a) = archives.get(i) else { break };
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scan_archive(schemas, data_dir, a, &common)))
                    .unwrap_or_else(|_| Err("its reader stopped".into()));
                mine.push((i, r));
            }
            mine
        })).collect();
        for w in workers {
            for (i, r) in w.join().unwrap_or_default() {
                found[i] = Some(r);
            }
        }
    });
    archives.iter().zip(found).map(|(a, r)| (a.clone(), r.unwrap_or_else(|| Err("not read".into())))).collect()
}

impl Game {
    /// A Game to build one archive's scenery from (`load_scenery`): this one's schemas and the
    /// textures it has found (common's among them), shared, and nothing else; no archives to
    /// search for a texture it lacks (that search unpacks archive after archive).
    pub fn scenery_base(&self) -> Game {
        let mut g = Game::empty(self.schemas.clone());
        g.share_textures(self);
        g
    }

    /// Load `archive`'s objects (each objects file whole, with its .ivd and .ipn: its meshes
    /// and collision surfaces point into them), materials and textures (textures-*.xmb and the
    /// .tex, kept whole), what its static scenery is drawn and collided from (see the module's
    /// notes). Entries already loaded win, but for textures: the archive's library replaces any
    /// shared from the played game that aren't common's, as a level's own does in the game
    /// (`Game::load_level`, #284), so a piece wears its own level's copy. How many bytes of
    /// vertex, collision and texture data it keeps.
    pub fn load_scenery(&mut self, data_dir: &Path, archive: &str) -> Result<usize, String> {
        let keep = |n: &str| n.starts_with("objects-") || n.starts_with("materials-") || n.starts_with("textures-");
        let ar = Archive::open(&data_dir.join(format!("{archive}.tgz")), keep)?;
        let mut kept = 0;
        for n in ar.find("objects-", ".xmb") {
            let root = bxml::parse(ar.get(&n).unwrap_or(&[]), &self.schemas).map_err(|e| format!("{n}: {e}"))?;
            let blob = |ext: &str| Arc::new(ar.get(&n.replace(".xmb", ext)).unwrap_or(&[]).to_vec());
            let (ivd, ipn) = (blob(".ivd"), blob(".ipn"));
            kept += ivd.len() + ipn.len();
            self.add_objects(root, ivd, ipn);
        }
        for n in ar.find("materials-", ".xmb") {
            let root = bxml::parse(ar.get(&n).unwrap_or(&[]), &self.schemas).map_err(|e| format!("{n}: {e}"))?;
            self.add_materials(&root, |_| true);
        }
        self.load_textures(&ar, archive, true)?;
        kept += ar.find("textures-", ".tex").iter().filter_map(|n| ar.get(n)).map(|d| d.len()).sum::<usize>();
        Ok(kept)
    }
}
