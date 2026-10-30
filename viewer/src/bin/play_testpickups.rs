//! The test map's pickup reset: Home brings back every pickup taken on the map (the product
//! owner's request on #18's review, with #19 and #21). Demo tooling, not the game: no footage
//! behind it, and the key is the demo's choice.
//!
//! Why it is wanted. A taken pickup is gone for good: FUN_0011c170 removes what it takes
//! (FUN_000db3e0) and nothing re-adds a level's own pickup (pickups.md, "A taken pickup is gone
//! for good"; only a spawn-trigger's timer makes a copy, play_pickup_spawns.rs). The test map's
//! pickup grid and the pickups the object tool puts down could then be taken once each, and
//! tried again only by putting them down again.
//!
//!  - **Home** (`RESET_KEY`; the hook BF_TEST_RESET_PICKUPS) puts back every pickup of the
//!    `Shelf` that is gone from the world, each where it was put and posed as it was: the grid's
//!    drop pose (the same 0.3 m fall, tipped and turned, so it settles as it did), the object
//!    tool's spot turned the way the copy showed (on the ground, or in the air to drop). One
//!    still there is left alone (not moved, not doubled). A message says how many came back.
//!  - **Which pickups**: those the test map put out and listed on the `Shelf`: the grid
//!    (play_testmap.rs `spawn_test_map`) and the object tool's (play_testtools.rs `place`, its
//!    pickups only: a lying gun or a grenade on the ground is the tool's own, and the rack's guns
//!    are restocked as they are taken). On a level the test session went to (the L menu) the
//!    shelf holds only what the object tool put there: the level's own pickups keep the game's
//!    rule, gone for good. A new map starts with an empty shelf.
//!  - **As a pickup placed after the map started**: a put-back pickup is made by `spawn_pickup`,
//!    the grid's own spawn, plus `LatePickup`, so `late_pickups` seats it, gives it its loose
//!    body and lists the medkits and fruit (and DNA canisters) as takeable, and `take_items`,
//!    `take_powerups` and the rest find it as they find any placed pickup. One the object tool
//!    put down keeps its number in the editor's `Placements` (`remake`: it was `Gone`, it is
//!    `Here` again). A character standing in reach when it comes back takes it at once, as the
//!    game's box does when its trigger's timer returns it.
//!
//! Test hook (play_testtools.rs `read_hooks`, on the controlled character's clock):
//! BF_TEST_RESET_PICKUPS=<s>[;<s>...] presses Home at those times. BF_TOOLS_LOG=1 prints what
//! came back ("t 6.00: 2 pickups put back: Medkit at [..], Ammo Box at [..]").

use super::*;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::Placed;
use super::testtools::{active, Hook, Script, MESSAGE_TIME};
use super::pickups::{DropIn, LatePickup};

/// The key that brings the taken pickups back: Home. Every letter (the walk, the tools, the
/// skills), the digits 1-4 (taking control), F5 (the map editor), Space, Enter, Tab, Delete,
/// Backspace, Escape, the arrows, Page Up / Down and the modifiers are used by the demo
/// (a search of the `KeyCode`s in play*.rs and the editor's tool registry), and nothing reads
/// Home. The demo's choice.
pub(super) const RESET_KEY: KeyCode = KeyCode::Home;

/// One pickup the test map put out: what it is, how it was put, and the entity that is it now.
#[derive(Clone, Debug)]
struct Piece {
    /// the item type (a `Game::items` key); its model is the type's `object_meshes` archetype
    kind: u32,
    /// the root's pose when it was put out: the grid's drop pose, or the object tool's spot
    pose: Transform,
    /// it falls from `pose` (`DropIn`: the grid's, and the object tool's in the free camera's
    /// air), else it lies where it is put
    drop_in: bool,
    /// the root's `Name`, for the entity lists and the logs
    name: String,
    /// the root now; gone from the world once the pickup is taken
    entity: Entity,
    /// the object tool's placement number in the editor's `Placements`, kept by the put-back
    placement: Option<u32>,
}

/// Every pickup the test map put out on this map, in the order put out: the grid's and the
/// object tool's (see the module's notes). Cleared as each map starts (`spawn_test_map`).
#[derive(Resource, Default)]
pub(super) struct Shelf(Vec<Piece>);

impl Shelf {
    /// A new map starts: nothing is on the shelf.
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    /// Lists a pickup just put out: item type `kind`, its root `entity`, put out at `pose` (`drop_in`:
    /// to fall from there), named `name`, and the object tool's placement number if it has one.
    pub(super) fn put(&mut self, kind: u32, pose: Transform, drop_in: bool, name: &str, entity: Entity, placement: Option<u32>) {
        self.0.push(Piece { kind, pose, drop_in, name: name.to_string(), entity, placement });
    }

    /// The pieces (indices) whose pickup is gone from the world: `alive` says whether an entity
    /// still is in it.
    fn missing(&self, alive: impl Fn(Entity) -> bool) -> Vec<usize> {
        (0..self.0.len()).filter(|&i| !alive(self.0[i].entity)).collect()
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Shelf>()
        .add_systems(Update, bring_back.after(super::testtools::run_script).after(super::item_pickups::take_items).run_if(active));
}

/// Makes a pickup of item type `kind` as the test map's grid makes its own: a level's
/// inventory-object (`Placed`, no name) at `pose` with its model's parts under it; `drop_in`: it
/// falls from there (`DropIn`); `late`: it joins the map's pickups the frame it is there
/// (`LatePickup`, as one placed after the map started does: the grid's own are found at the
/// start, the put-back ones are not). Returns its root.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_pickup(commands: &mut Commands, game: &mut Game, assets: &mut ModelAssets, kind: u32, model: &WeaponModel, pose: Transform,
                           drop_in: bool, late: bool, name: &str) -> Entity {
    let root = commands.spawn((pose, Visibility::default(), Placed { tag: h("inventory-object"), kind, name: 0 }, Name::new(name.to_string()))).id();
    if drop_in {
        commands.entity(root).insert(DropIn);
    }
    if late {
        commands.entity(root).insert(LatePickup);
    }
    for part in &model.parts {
        let pe = commands.spawn((Transform::from_translation(part.offset).with_rotation(part.rotation), Visibility::Inherited, ChildOf(root))).id();
        spawn_static(commands, game, &part.geosets, assets, pe, true);
    }
    root
}

/// Home (or the hook): every pickup on the shelf that is gone is made again at its place (see
/// the module's notes), its piece now naming the new root so it can be taken and brought back
/// again. A message says how many; `BF_TOOLS_LOG` lists them.
#[allow(clippy::too_many_arguments)]
fn bring_back(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, script: Res<Script>, mut shelf: ResMut<Shelf>, mut game: ResMut<GameData>,
              mut placements: ResMut<super::editor::Placements>, mut status: ResMut<UsePanel>, player: Res<Player>, world: Query<(), With<Placed>>,
              mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
              mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>) {
    let hook = script.now.iter().any(|step| matches!(step, Hook::ResetPickups));
    if !keys.just_pressed(RESET_KEY) && !hook {
        return;
    }
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    let mut back: Vec<String> = vec![];
    for i in shelf.missing(|e| world.contains(e)) {
        let piece = shelf.0[i].clone();
        // (the type's model, loaded as the grid and the object menu load it: pickup_types)
        let model = game.0.object_meshes.get(&piece.kind).copied().and_then(|arch| WeaponModel::load(&game.0, arch).ok());
        let Some(model) = model else {
            println!("test map: no model for pickup h_{:08x}: not put back", piece.kind);
            continue;
        };
        let root = spawn_pickup(&mut commands, &mut game.0, &mut assets, piece.kind, &model, piece.pose, piece.drop_in, true, &piece.name);
        if let Some(id) = piece.placement {
            commands.entity(root).insert(super::editor::PlacedId(id));
            placements.remake(id, root);
        }
        shelf.0[i].entity = root;
        let label = game.0.items.get(&piece.kind).map_or_else(|| format!("h_{:08x}", piece.kind), |it| it.label.clone());
        back.push(format!("{label} at {:.2}", piece.pose.translation));
    }
    status.message = Some((match back.len() {
        0 => "No pickup to put back".to_string(),
        1 => "1 pickup put back".to_string(),
        n => format!("{n} pickups put back"),
    }, MESSAGE_TIME));
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        println!("t {:.2}: {} of {} pickups put back{}{}", player.sim_time, back.len(), shelf.0.len(), if back.is_empty() { "" } else { ": " }, back.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pieces missing are those whose entity the world no longer has, in the order put out,
    /// and a piece made again (its new entity put on it) isn't missing any more.
    #[test]
    fn a_taken_pickup_is_missing_until_it_is_put_back() {
        let mut shelf = Shelf::default();
        let (a, b, c) = (Entity::from_raw(1), Entity::from_raw(2), Entity::from_raw(3));
        for (n, e) in [a, b, c].into_iter().enumerate() {
            shelf.put(n as u32, Transform::IDENTITY, true, "test pickup", e, None);
        }
        assert!(shelf.missing(|_| true).is_empty());
        assert_eq!(shelf.missing(|e| e != b), vec![1]);
        assert_eq!(shelf.missing(|e| e == b), vec![0, 2]);
        // (put back: the new root takes its place on the shelf)
        let again = Entity::from_raw(9);
        shelf.0[1].entity = again;
        assert!(shelf.missing(|e| e != b).is_empty());
        shelf.clear();
        assert!(shelf.missing(|_| false).is_empty());
    }
}
