//! The test map (`cargo run --bin bf_play -- --test`): the flat test floor, straight in with no
//! boot screen or menu, for trying things out. Only reachable by that command.
//!
//!  - Every hand weapon (a `Game::weapons` definition with a clip, whose model loads; one per
//!    model), from the first mission's and the multiplayer levels' data, floats on a rack in
//!    front of the start, turning slowly. Walking into one puts it in the held weapon's slot
//!    (the character is respawned with it: full ammo, the weapon in hand).
//!  - One of every pickup type with a model (`Game::items`, one per model) is dropped in a
//!    grid behind the rack, upright but tipped a little, settling as it would, a
//!    level's inventory-object as far as play_pickups.rs is concerned: medkits and fruit are
//!    taken and used as on a map; the rest are loose objects.
//!  - Instant kill: the player's shots and grenades kill any squad member they hurt, in one
//!    hit. On at the start; K turns it on and off.
//!  - X kills the controlled character outright (for the death camera and the hand-over to
//!    the next squad member).
//!  - The controls panel (H) lives here; the main game doesn't show it.

use super::*;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::Placed;

/// The command-line flag that opens the test map.
const FLAG: &str = "--test";
/// The weapon rack: how many to a row, the spacing (m), where its first row is (z, ahead of
/// the start), how high they float and how fast they turn (rad/s).
const RACK_ROW: usize = 8;
const RACK_STEP: f32 = 1.6;
const RACK_Z: f32 = -6.0;
const RACK_UP: f32 = 0.9;
const RACK_SPIN: f32 = 0.8;
/// How near (m, across) the player must come to take a weapon.
const TAKE_REACH: f32 = 0.9;
/// The pickups' grid, behind the rack: how many to a row, the spacing (m) and its first row (z).
const ITEMS_ROW: usize = 8;
const ITEMS_STEP: f32 = 1.4;
const ITEMS_Z: f32 = -12.0;
/// The levels whose weapons the test map loads (see `load_weapon_data`).
/// The multiplayer archives hold 24 hand weapons with models, against the first mission's 10.
/// The campaign adds three: A10 Bioreactive, Confed LZR-50 and Jax-iP, all in m02_a (found by
/// loading every level with BF_TESTMAP_LOG). Every level together loads in about 6 s and
/// adds nothing more; these take about 0.2 s.
const TEST_LEVELS: [&str; 9] = ["mp_common", "mp1", "mp2", "mp3", "mp4", "mp6", "mp7", "mp8", "m02_a"];
/// How high (m) the pickups are dropped from, and how far (rad) each is tipped either way.
const DROP_HEIGHT: f32 = 0.3;
const DROP_TIP: f32 = 0.2;
/// How long a test-map message shows (s).
const MESSAGE_TIME: f32 = 2.0;

/// The test map is being played (inserted only when it was asked for): instant kill on or off.
#[derive(Resource, Clone, Copy)]
pub struct TestMap {
    pub instant_kill: bool,
}

/// Whether the test map was asked for on the command line.
pub fn requested() -> bool {
    std::env::args().any(|a| a == FLAG)
}

/// Every level's weapon definitions and models, on top of the first mission's (only the
/// definitions with a clip are hand weapons). The test map's levels: BF_TESTMAP_LEVELS
/// (comma-separated) overrides the list.
pub fn load_weapon_data(game: &mut bf_viewer::bf::character::Game) {
    let t = std::time::Instant::now();
    let surfaces = game.surfaces.clone();
    let list = std::env::var("BF_TESTMAP_LEVELS").unwrap_or_else(|_| TEST_LEVELS.join(","));
    for level in list.split(',').filter(|l| !l.is_empty()) {
        if let Err(e) = game.load_level(&data_dir(), level) {
            eprintln!("test map: no data from {level}: {e}");
        }
    }
    // (the first mission's surfaces and their footsteps, as on any map)
    game.surfaces = surfaces;
    let hand = game.weapons.values().filter(|d| d.ammo > 0).count();
    println!("test map: weapon data from {list} in {:.1} s: {hand} hand weapons", t.elapsed().as_secs_f32());
}

/// A weapon on the rack: its definition (a `Game::weapons` key).
#[derive(Component)]
struct RackWeapon(u32);

/// A weapon taken from the rack: the slot to put in hand once the character is respawned.
#[derive(Resource, Default)]
struct PendingSlot(Option<usize>);

pub fn plugin(app: &mut App) {
    app.init_resource::<PendingSlot>()
        .add_systems(OnEnter(AppState::Playing), spawn_test_map.after(setup).run_if(resource_exists::<TestMap>))
        .add_systems(Update, (toggle_instant_kill, suicide, take_weapons, spin_rack).chain().after(update_player)
            .run_if(in_state(AppState::Playing).and(resource_exists::<TestMap>)));
}

/// The rack of every weapon and the row of every pickup.
fn spawn_test_map(mut commands: Commands, mut game: ResMut<GameData>, mut player: ResMut<Player>,
                  mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
                  mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>) {
    player.show_help = true;
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    // every weapon whose model loads, by name
    let mut weapons: Vec<(u32, WeaponDef)> = game.0.weapons.iter().map(|(&k, d)| (k, d.clone())).collect();
    weapons.sort_by(|a, b| a.1.label.cmp(&b.1.label).then(a.0.cmp(&b.0)));
    let mut n = 0;
    let mut models = std::collections::HashSet::new();
    for (key, def) in weapons {
        // hand weapons only (the rest are pickups, props and level objects), one per model
        let log = std::env::var("BF_TESTMAP_LOG").is_ok();
        let skip = |why: &str| if log && def.ammo > 0 {
            println!("skipped h_{key:08x} {:24} arch h_{:08x}: {why}", def.label, def.archetype);
        };
        if def.ammo <= 0 || !models.insert(def.archetype) {
            skip("its model is already on the rack");
            continue;
        }
        let Ok(model) = WeaponModel::load(&game.0, def.archetype) else {
            skip("no model");
            continue;
        };
        let size = model.parts.iter().flat_map(|p| p.geosets.iter().flat_map(|g| g.positions.iter().map(|v| Vec3::from(*v))))
            .fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(v), u.max(v)));
        if (size.1 - size.0).max_element() < 0.1 {
            skip("under 0.1 m");
            continue;
        }
        if log {
            let (lo, hi) = model.parts.iter().flat_map(|p| p.geosets.iter().flat_map(move |g| g.positions.iter().map(move |v| p.offset + p.rotation * Vec3::from(*v))))
                .fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(v), u.max(v)));
            println!("weapon h_{key:08x} {:24} type {:3} ammo {:4} arch h_{:08x} size {:.2}", def.label, def.weapon_type, def.ammo, def.archetype, hi - lo);
        }
        let (row, col) = (n / RACK_ROW, n % RACK_ROW);
        let x = (col as f32 - (RACK_ROW as f32 - 1.0) / 2.0) * RACK_STEP;
        let at = Vec3::new(x, GROUND + RACK_UP, RACK_Z - row as f32 * RACK_STEP);
        let root = commands.spawn((Transform::from_translation(at), Visibility::default(), RackWeapon(key),
                                   Name::new(format!("rack {}", def.label)))).id();
        for part in &model.parts {
            let pe = commands.spawn((Transform::from_translation(part.offset).with_rotation(part.rotation), Visibility::Inherited, ChildOf(root))).id();
            spawn_static(&mut commands, &mut game.0, &part.geosets, &mut assets, pe, true);
        }
        n += 1;
    }
    // one of every pickup type with a model
    let mut items: Vec<(u32, String)> = game.0.items.iter().filter(|(k, _)| game.0.object_meshes.contains_key(k))
        .map(|(&k, i)| (k, i.label.clone())).collect();
    items.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let mut placed = 0;
    let mut models = std::collections::HashSet::new();
    for (kind, _) in &items {
        let Some(&arch) = game.0.object_meshes.get(kind) else { continue };
        if !models.insert(arch) {
            continue;
        }
        let Ok(model) = WeaponModel::load(&game.0, arch) else { continue };
        let (row, col) = (placed / ITEMS_ROW, placed % ITEMS_ROW);
        let x = (col as f32 - (ITEMS_ROW as f32 - 1.0) / 2.0) * ITEMS_STEP;
        // dropped from a little height, upright as modelled but turned and tipped a little, so
        // it settles as it would: a crate on its base, the Garo fruit (modelled on its point)
        // and the cards (on their edge) over onto their sides
        let at = Vec3::new(x, GROUND + DROP_HEIGHT, ITEMS_Z - row as f32 * ITEMS_STEP);
        let seed = |k: f32| ((placed as f32 + 1.0) * k).sin().fract().abs() * 2.0 - 1.0;
        let turn = Quat::from_euler(EulerRot::YXZ, seed(12.9898) * std::f32::consts::PI, seed(78.233) * DROP_TIP, seed(37.719) * DROP_TIP);
        let root = commands.spawn((Transform::from_translation(at).with_rotation(turn), Visibility::default(),
                                   Placed { tag: h("inventory-object"), kind: *kind }, super::pickups::DropIn,
                                   Name::new("test pickup"))).id();
        for part in &model.parts {
            let pe = commands.spawn((Transform::from_translation(part.offset).with_rotation(part.rotation), Visibility::Inherited, ChildOf(root))).id();
            spawn_static(&mut commands, &mut game.0, &part.geosets, &mut assets, pe, true);
        }
        placed += 1;
    }
    println!("test map: {n} weapons on the rack, {placed} pickups");
}

/// K: instant kill on or off.
fn toggle_instant_kill(keys: Res<ButtonInput<KeyCode>>, mut test: ResMut<TestMap>, mut status: ResMut<UsePanel>) {
    if keys.just_pressed(KeyCode::KeyK) {
        test.instant_kill = !test.instant_kill;
        status.message = Some((format!("Instant kill {}", if test.instant_kill { "on" } else { "off" }), MESSAGE_TIME));
    }
}

/// X: the controlled character dies on the spot, as from any other hurt (death cry, ragdoll,
/// the death camera, the hand-over to the next squad member).
/// (test hook: BF_TEST_SUICIDE=<s> presses it at that time)
fn suicide(keys: Res<ButtonInput<KeyCode>>, game: Res<GameData>, mut player: ResMut<Player>, mut tested: Local<bool>) {
    let test = std::env::var("BF_TEST_SUICIDE").ok().and_then(|v| v.parse::<f32>().ok())
        .is_some_and(|at| player.sim_time >= at && !*tested);
    if (keys.just_pressed(KeyCode::KeyX) || test) && !player.dead {
        *tested |= test;
        let (at, back) = (player.position + Vec3::Y * 1.0, Quat::from_rotation_y(player.yaw) * Vec3::Z);
        let all = player.health;
        hurt(&mut player, &game.0, all, HURT_CHATTER, back * 2.0 + Vec3::Y, at, -1);
    }
}

/// Walking into a rack weapon: it goes into the held weapon's slot, and the character is
/// respawned carrying it (with it in hand once they're back).
fn take_weapons(mut commands: Commands, mut game: ResMut<GameData>, mut player: ResMut<Player>, mut pending: ResMut<PendingSlot>,
                mut status: ResMut<UsePanel>, rack: Query<(&RackWeapon, &GlobalTransform)>) {
    // back from a respawn: the new weapon in hand
    if let (Some(slot), Some(l)) = (pending.0, player.loaded.as_ref()) {
        let slot = slot.min(l.weapons.len().saturating_sub(1));
        player.weapon = slot;
        player.weapon_dirty = true;
        pending.0 = None;
        return;
    }
    // walking into it, not standing on it (taking over a squad member who stands on the rack)
    if pending.0.is_some() || player.dead || player.loaded.is_none() || player.move_input == Vec2::ZERO {
        return;
    }
    let feet = player.position;
    let Some((key, _)) = rack.iter().map(|(w, t)| (w.0, t.translation()))
        .find(|(_, at)| Vec2::new(at.x - feet.x, at.z - feet.z).length() < TAKE_REACH) else { return };
    let name = CHARACTERS[player.character];
    let slot = player.weapon;
    let list = game.0.character_weapons.entry(name.to_string()).or_default();
    if list.get(slot) == Some(&key) {
        return;
    }
    if slot < list.len() { list[slot] = key } else { list.push(key) }
    let label = game.0.weapons.get(&key).map(|d| d.label.clone()).unwrap_or_default();
    status.message = Some((format!("Took {label}"), MESSAGE_TIME));
    if let Some(old) = player.loaded.take() {
        commands.entity(old.root).despawn();
    }
    pending.0 = Some(slot);
}

/// The rack's weapons turn slowly.
fn spin_rack(time: Res<Time>, mut rack: Query<&mut Transform, With<RackWeapon>>) {
    let turn = Quat::from_rotation_y(RACK_SPIN * frame_dt(&time));
    for mut t in &mut rack {
        t.rotation = turn * t.rotation;
    }
}

