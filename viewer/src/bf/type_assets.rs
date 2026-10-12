//! Character types from other archives, without their levels (#180): what a type needs to be
//! spawned (its definition, model, motion set and clips, chatter, weapons and sounds), taken
//! from a level archive whose level isn't loaded. The test map uses it to offer the types its
//! own levels don't define (play_testmap.rs `load_types`).
//!
//! Every campaign level defines the types it places in its own objecttypes; a type defined in
//! several archives is the same in each (#128, character-types.md), and the first one loaded
//! wins (FUN_00160da0). Per archive, only the types not loaded yet (and that the caller keeps)
//! are added, with:
//!  - the weapon definitions they carry that aren't loaded (and what those fire, their
//!    `projectile`), and the archive's effect and decal types (looked up by name; first wins);
//!  - from its objects file, only the archetypes and meshes their models and those weapons'
//!    models reach (PARTs, LOD meshes, hardpoints...), with their collision surfaces: not the
//!    level's own geometry. The file's vertex and collision data (.ivd, .ipn) are kept whole, as
//!    the meshes and surfaces point into them by offset;
//!  - the materials those meshes name (their textures are found on demand, as everyone's:
//!    `Game::texture_rgba`);
//!  - their motion sets, their animation and face sets with only the channels those use, and
//!    their chatter files;
//!  - the archive's sound bank (first loaded wins, as for any level: their weapons' and voices'
//!    in-memory sounds).
//!
//! Placements, items, object types, the level file and its geometry aren't read, so nothing
//! the test map lists (the rack, the pickups, the object menu) changes.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use super::archive::Archive;
use super::bxml::{self, Element, SchemaSet, Value};
use super::character::{Game, H_PHYSICS, H_PHYSICS_FILE, H_PHYSICS_NAME};
use super::character_type::{parse_types, CharacterType};
use super::hash::h;
use super::weapon::{self, WeaponDef};

/// The members of a level archive a type's assets come from (by base name): its objecttypes,
/// objects (with their .ivd / .ipn), materials, animations (.chnl), sound bank (.mem), motion
/// sets and chatter. Not its textures (read on demand) nor its levels-*.xmb.
fn wanted(name: &str) -> bool {
    ["objecttypes", "objects-", "materials-", "animations-", "sounds-", "ms_"].iter().any(|p| name.starts_with(p))
        || name.ends_with("_chatter.xmb")
}

/// Every name hash an element and its descendants carry in their attributes (a hash, a string's
/// hash, the entries of a list): the references an archetype or a mesh makes to others.
fn references(e: &Element, out: &mut HashSet<u32>) {
    fn value(v: &Value, out: &mut HashSet<u32>) {
        match v {
            Value::List(l) => l.iter().for_each(|v| value(v, out)),
            v => {
                if let Some(x) = v.as_hash() {
                    out.insert(x);
                }
            }
        }
    }
    for x in e.walk() {
        x.attrs.iter().for_each(|(_, v)| value(v, out));
    }
}

/// An element's `name` attribute's hash (0 without one).
fn name_of(e: &Element) -> u32 {
    e.attr(h("name")).and_then(|v| v.as_hash()).unwrap_or(0)
}

/// An objects file cut down to what `roots` (archetype or mesh names) reach: the archetypes and
/// meshes they name, and those name, and so on (a compound's PARTs and LOD meshes, a weapon's
/// parts), and the collision surfaces those archetypes name (`<h_f3f067ad h_051ac63e=N>`), in
/// the file's order. With every name the kept elements carry (their materials among them).
/// None when the file has none of the roots.
fn prune_objects(root: &Element, roots: &[u32]) -> Option<(Element, HashSet<u32>)> {
    let archetypes: HashMap<u32, &Element> = root.children_named(h("archetype-library"))
        .flat_map(|l| l.children_named(h("archetype"))).map(|a| (name_of(a), a)).collect();
    let meshes: HashMap<u32, &Element> = root.children_named(h("mesh-library"))
        .flat_map(|l| l.children_named(h("mesh"))).map(|m| (name_of(m), m)).collect();
    let mut kept: HashSet<u32> = HashSet::new();
    let mut refs: HashSet<u32> = HashSet::new();
    let mut queue: Vec<u32> = roots.iter().copied().filter(|r| archetypes.contains_key(r) || meshes.contains_key(r)).collect();
    if queue.is_empty() {
        return None;
    }
    while let Some(n) = queue.pop() {
        if !kept.insert(n) {
            continue;
        }
        let Some(e) = archetypes.get(&n).or(meshes.get(&n)) else { continue };
        let mut found = HashSet::new();
        references(e, &mut found);
        queue.extend(found.iter().copied().filter(|r| !kept.contains(r) && (archetypes.contains_key(r) || meshes.contains_key(r))));
        refs.extend(found);
    }
    // the collision surfaces the kept archetypes name
    let surfaces: HashSet<u32> = kept.iter().filter_map(|n| archetypes.get(n))
        .filter_map(|a| a.child(H_PHYSICS)?.attr(H_PHYSICS_NAME)?.as_hash()).collect();
    let library = |lib: &str, item: &str| -> Vec<Element> {
        root.children_named(h(lib)).map(|l| Element {
            name: l.name, attrs: l.attrs.clone(), text: None,
            children: l.children_named(h(item)).filter(|e| kept.contains(&name_of(e))).cloned().collect(),
        }).filter(|l| !l.children.is_empty()).collect()
    };
    let mut children = library("archetype-library", "archetype");
    children.extend(library("mesh-library", "mesh"));
    children.extend(root.children_named(H_PHYSICS_FILE).map(|t| Element {
        name: t.name, attrs: t.attrs.clone(), text: None,
        children: t.children_named(H_PHYSICS).filter(|s| surfaces.contains(&name_of(s))).cloned().collect(),
    }));
    Some((Element { name: root.name, attrs: root.attrs.clone(), children, text: root.text.clone() }, refs))
}

/// The kinds of document a type's assets are read from, by member name prefix: parsed on the
/// unpacking threads (the motion sets and chatter files are small and read on demand).
const DOCUMENTS: [&str; 5] = ["objecttypes", "objects-", "materials-", "animations-", "sounds-"];

/// A level archive unpacked and its documents parsed (`DOCUMENTS`, by member name), on a thread.
struct Unpacked {
    ar: Archive,
    docs: Vec<(String, Element)>,
}

impl Unpacked {
    /// Unpack `path` (its `wanted` members) and parse its documents with `schemas`.
    fn read(path: &Path, schemas: &SchemaSet) -> Result<Self, String> {
        let ar = Archive::open(path, wanted)?;
        let mut docs = vec![];
        for n in DOCUMENTS.iter().flat_map(|p| ar.find(p, ".xmb")) {
            if let Some(d) = ar.get(&n) {
                docs.push((n.clone(), bxml::parse(d, schemas).map_err(|e| format!("{n}: {e}"))?));
            }
        }
        Ok(Self { ar, docs })
    }

    /// The parsed documents whose name starts with `prefix`, with their names.
    fn docs<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a (String, Element)> {
        self.docs.iter().filter(move |(n, _)| n.starts_with(prefix))
    }

    /// The member beside document `name` with extension `ext` (an objects file's ".ivd"), whole.
    fn beside(&self, name: &str, ext: &str) -> Arc<Vec<u8>> {
        Arc::new(self.ar.get(&name.replace(".xmb", ext)).unwrap_or(&[]).to_vec())
    }
}

impl Game {
    /// Add the character types that `archives` (level archive names, e.g. "m03_c") define and
    /// the loaded data doesn't, those `keep` passes, with what they need to be spawned (see the
    /// module's comment). The archives are unpacked and parsed on threads (as many at once as
    /// there are cores), then added in the order given: where two define a type, the first in
    /// `archives` wins. Per archive: how many types it added, or why it couldn't be read.
    pub fn load_type_assets(&mut self, data_dir: &Path, archives: &[&str], keep: impl Fn(&CharacterType) -> bool)
        -> Vec<(String, Result<usize, String>)> {
        // (the threads read with a copy of the schemas, as `self` is written meanwhile)
        let schemas = self.schemas.clone();
        std::thread::scope(|s| {
            let jobs: Vec<_> = archives.iter().map(|a| {
                let (path, schemas) = (data_dir.join(format!("{a}.tgz")), &schemas);
                s.spawn(move || Unpacked::read(&path, schemas))
            }).collect();
            archives.iter().zip(jobs).map(|(archive, job)| {
                let u = job.join().unwrap_or_else(|_| Err("its reader stopped".into()));
                let added = u.and_then(|u| self.add_type_assets(&u, archive, &keep));
                (archive.to_string(), added)
            }).collect()
        })
    }

    /// One archive's part of `load_type_assets`: how many types it added.
    fn add_type_assets(&mut self, u: &Unpacked, archive: &str, keep: &impl Fn(&CharacterType) -> bool) -> Result<usize, String> {
        let types: Vec<&Element> = u.docs("objecttypes").map(|(_, r)| r).collect();
        let mut new: Vec<CharacterType> = vec![];
        for (n, root) in u.docs("objecttypes") {
            let library = n.trim_start_matches("objecttypes-").trim_end_matches(".xmb");
            for t in parse_types(root, archive, library) {
                if !self.character_types.contains_key(&t.key) && !new.iter().any(|n| n.key == t.key) && keep(&t) {
                    new.push(t);
                }
            }
        }
        if new.is_empty() {
            return Ok(0);
        }
        // the weapons they carry that aren't loaded, and what those fire
        let mut defs: HashMap<u32, WeaponDef> = HashMap::new();
        for root in &types {
            for w in weapon::parse_weapons(root, &self.strings) {
                defs.entry(w.name).or_insert(w);
            }
        }
        let mut want: Vec<u32> = new.iter().flat_map(|t| t.weapons.iter().copied()).collect();
        let mut added_weapons = vec![];
        while let Some(w) = want.pop() {
            let Some(d) = defs.get(&w).filter(|_| !self.weapons.contains_key(&w)) else { continue };
            if d.projectile != 0 {
                want.push(d.projectile);
            }
            self.weapons.insert(w, d.clone());
            added_weapons.push(w);
        }
        for root in &types {
            self.add_effect_types(root);
            self.add_decals(root);
        }
        // their models and those weapons' (the ones no loaded objects file has), cut out of the
        // archive's objects, and the materials those name
        let roots: Vec<u32> = new.iter().map(|t| t.mesh).chain(added_weapons.iter().filter_map(|w| self.weapons.get(w)).map(|d| d.archetype))
            .filter(|&a| a != 0 && self.archetype(a).is_none()).collect();
        if !roots.is_empty() {
            let mut names: HashSet<u32> = HashSet::new();
            for (n, root) in u.docs("objects-") {
                let Some((kept, refs)) = prune_objects(root, &roots) else { continue };
                names.extend(refs);
                self.add_objects(kept, u.beside(n, ".ivd"), u.beside(n, ".ipn"));
            }
            for (_, root) in u.docs("materials-") {
                self.add_materials(root, |m| names.contains(&m));
            }
        }
        // their motion sets and chatter, then the animation sets those motion sets name and their
        // face sets
        let stems: HashSet<String> = new.iter().map(|t| t.motion_archetype.clone()).collect();
        self.load_motion_sets(&u.ar, |s| stems.contains(s));
        let libraries: HashSet<String> = new.iter().map(|t| t.chatter.trim_end_matches("_chatter").to_string()).collect();
        self.load_chatter(&u.ar, |who| libraries.contains(who))?;
        let sets: HashSet<u32> = new.iter().flat_map(|t| [self.motion_sets.get(&t.motion_set).map_or(0, |m| m.anim_set), t.face_set])
            .filter(|&s| s != 0 && !self.anim_sets.contains_key(&s)).collect();
        if !sets.is_empty() {
            for (n, root) in u.docs("animations-") {
                self.add_animations(root, u.beside(n, ".chnl"), Some(&sets));
            }
        }
        // the sound bank (as `Game::load_sounds` reads one)
        for (n, root) in u.docs("sounds-") {
            self.sounds.add(root, u.beside(n, ".mem"));
        }
        let count = new.len();
        for t in new {
            self.add_character_type(t);
        }
        Ok(count)
    }
}
