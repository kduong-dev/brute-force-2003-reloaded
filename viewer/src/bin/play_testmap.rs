//! The test map (`cargo run --bin bf_play -- --test`): the flat test floor, straight in with no
//! boot screen or menu, for trying things out. Only reachable by that command.
//!
//!  - Every hand weapon (a `Game::weapons` definition with a clip, whose model loads; one per
//!    model), from the first mission's and the multiplayer levels' data, floats on a rack in
//!    front of the start, turning slowly. Walking into one puts it in the held weapon's slot
//!    (the character is respawned with it: a full clip, the weapon in hand; the reserve is
//!    the squad's, play_ammo.rs).
//!  - One of every pickup type with a model (`Game::items`, one per model) is dropped in a
//!    grid behind the rack, upright but tipped a little, settling as it would, a
//!    level's inventory-object as far as play_pickups.rs is concerned: medkits and fruit are
//!    taken and used as on a map; the rest are loose objects.
//!  - The squad carries a full stack (stack-limit) of every grenade type the levels define
//!    (play_grenade.rs: Frag, Energy, Gas, Light, Sonic, Roller, Sentry); T steps through them.
//!  - Instant kill: the player's shots and grenades kill any squad member they hurt, in one
//!    hit. On at the start; K turns it on and off.
//!  - X kills the controlled character outright (for the death camera and the hand-over to
//!    the next squad member).
//!  - The controls panel (H) lives here; the main game doesn't show it. It starts hidden, with
//!    an "H: help" hint.
//!  - The developer tools (play_testtools.rs): a free camera with a teleport, an NPC spawner
//!    and an object spawner; and (play_testworld.rs) a sky and a music picker and a level
//!    switch. On a level the switch went to (or `--test` with BF_MAP) everything here but the
//!    rack and the grid holds: the tools, instant kill, X, the grenade stacks, the help panel.

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
/// adds nothing more; these take about 0.2 s. m09_a adds the Light grenade (h_fd1a966d is only
/// in m09_a/b/c/x; m09_a is the smallest): with the multiplayer levels (Sentry, Roller, Gas,
/// Energy, Sonic, Frag) every squad grenade type is defined. sdm_e34 adds the missile rack
/// (h_fbdcd828) for the object tool's breakable scenery (play_testtools.rs; the barrel and the
/// crate are in the others too): about 0.2 s more, the rack and the pickups unchanged.
const TEST_LEVELS: [&str; 11] = ["mp_common", "mp1", "mp2", "mp3", "mp4", "mp6", "mp7", "mp8", "m02_a", "m09_a", "sdm_e34"];
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

/// A weapon on the rack, or one the object tool laid on the ground (`Lying`): its definition
/// (a `Game::weapons` key).
#[derive(Component)]
pub(super) struct RackWeapon(pub(super) u32);

/// A weapon laid on the ground by the object tool (play_testtools.rs): it doesn't turn, and it's
/// gone once taken.
#[derive(Component)]
pub(super) struct Lying;

/// A weapon taken from the rack: the slot to put in hand once the character is respawned.
#[derive(Resource, Default)]
struct PendingSlot(Option<usize>);

pub fn plugin(app: &mut App) {
    super::testtools::plugin(app);
    super::testworld::plugin(app);
    app.init_resource::<PendingSlot>()
        .add_systems(OnEnter(AppState::Playing), spawn_test_map.after(setup).run_if(resource_exists::<TestMap>))
        .add_systems(Update, (toggle_instant_kill, suicide, take_weapons, spin_rack).chain().after(update_player)
            .run_if(in_state(AppState::Playing).and(resource_exists::<TestMap>)));
}

/// Every hand weapon the test map has, by label: each `Game::weapons` definition with a clip
/// (the rest are pickups, props and level objects) whose model loads and is at least 0.1 m,
/// one per model, with its model. The rack's list, and the object tool's (play_testtools.rs).
/// `log`: print each, and each hand weapon left out with why (BF_TESTMAP_LOG).
pub(super) fn hand_weapons(game: &bf_viewer::bf::character::Game, log: bool) -> Vec<(u32, WeaponDef, WeaponModel)> {
    let mut weapons: Vec<(u32, WeaponDef)> = game.weapons.iter().map(|(&k, d)| (k, d.clone())).collect();
    weapons.sort_by(|a, b| a.1.label.cmp(&b.1.label).then(a.0.cmp(&b.0)));
    let mut out = vec![];
    let mut models = std::collections::HashSet::new();
    for (key, def) in weapons {
        let skip = |why: &str| if log && def.ammo > 0 {
            println!("skipped h_{key:08x} {:24} arch h_{:08x}: {why}", def.label, def.archetype);
        };
        if def.ammo <= 0 || !models.insert(def.archetype) {
            skip("its model is already on the rack");
            continue;
        }
        let Ok(model) = WeaponModel::load(game, def.archetype) else {
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
        out.push((key, def, model));
    }
    out
}

/// Every pickup type the test map has, by label: each `Game::items` type with a model, one per
/// model: (item type, label, its model). The grid's list, and the object tool's.
pub(super) fn pickup_types(game: &bf_viewer::bf::character::Game) -> Vec<(u32, String, WeaponModel)> {
    let mut items: Vec<(u32, String)> = game.items.iter().filter(|(k, _)| game.object_meshes.contains_key(k))
        .map(|(&k, i)| (k, i.label.clone())).collect();
    items.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let mut models = std::collections::HashSet::new();
    items.into_iter().filter_map(|(kind, label)| {
        let arch = *game.object_meshes.get(&kind)?;
        if !models.insert(arch) {
            return None;
        }
        Some((kind, label, WeaponModel::load(game, arch).ok()?))
    }).collect()
}

/// The rack of every weapon and the row of every pickup, on the flat floor only (the test tools
/// can switch to a level, play_testworld.rs). The controls panel starts hidden (H).
#[allow(clippy::too_many_arguments)]
fn spawn_test_map(mut commands: Commands, mut game: ResMut<GameData>, mut player: ResMut<Player>, current: Res<CurrentMap>, mut pending: ResMut<PendingSlot>,
                  mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
                  mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>) {
    player.show_help = false;
    // (a weapon taken just before a level switch isn't put in the next map's hand)
    pending.0 = None;
    if current.0 != "flat" {
        return;
    }
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    // every weapon whose model loads, by name
    let mut n = 0;
    for (key, def, model) in hand_weapons(&game.0, std::env::var("BF_TESTMAP_LOG").is_ok()) {
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
    let mut placed = 0;
    for (kind, _, model) in pickup_types(&game.0) {
        let (row, col) = (placed / ITEMS_ROW, placed % ITEMS_ROW);
        let x = (col as f32 - (ITEMS_ROW as f32 - 1.0) / 2.0) * ITEMS_STEP;
        // dropped from a little height, upright as modelled but turned and tipped a little, so
        // it settles as it would: a crate on its base, the Garo fruit (modelled on its point)
        // and the cards (on their edge) over onto their sides
        let at = Vec3::new(x, GROUND + DROP_HEIGHT, ITEMS_Z - row as f32 * ITEMS_STEP);
        let seed = |k: f32| ((placed as f32 + 1.0) * k).sin().fract().abs() * 2.0 - 1.0;
        let turn = Quat::from_euler(EulerRot::YXZ, seed(12.9898) * std::f32::consts::PI, seed(78.233) * DROP_TIP, seed(37.719) * DROP_TIP);
        let root = commands.spawn((Transform::from_translation(at).with_rotation(turn), Visibility::default(),
                                   Placed { tag: h("inventory-object"), kind }, super::pickups::DropIn,
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
/// (test hook: BF_TEST_TOGGLE_KILL=<s>[,<s>...] presses it at those times: e.g. a squadmate
/// killed by BF_TEST_KILL, then instant kill off, for a grenade that hurts the living and throws
/// the dead)
fn toggle_instant_kill(keys: Res<ButtonInput<KeyCode>>, mut test: ResMut<TestMap>, mut status: ResMut<UsePanel>, player: Res<Player>,
                       mut pressed: Local<usize>) {
    let times: Vec<f32> = std::env::var("BF_TEST_TOGGLE_KILL").ok()
        .map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default();
    let hook = times.get(*pressed).is_some_and(|&at| player.sim_time >= at);
    *pressed += hook as usize;
    if keys.just_pressed(KeyCode::KeyK) || hook {
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
#[allow(clippy::too_many_arguments)]
fn take_weapons(mut commands: Commands, mut game: ResMut<GameData>, mut player: ResMut<Player>, mut pending: ResMut<PendingSlot>,
                mut status: ResMut<UsePanel>, rack: Query<(Entity, &RackWeapon, &GlobalTransform, Has<Lying>)>,
                mut held_name: ResMut<super::ammo::HeldName>) {
    // back from a respawn: the new weapon in hand, its name on the HUD a moment (as after the
    // game's pickup)
    if let (Some(slot), Some(l)) = (pending.0, player.loaded.as_ref()) {
        let slot = slot.min(l.weapons.len().saturating_sub(1));
        player.weapon = slot;
        player.weapon_dirty = true;
        pending.0 = None;
        held_name.0 = super::ammo::NAME_TIME;
        return;
    }
    // walking into it, not standing on it (taking over a squad member who stands on the rack)
    if pending.0.is_some() || player.dead || player.loaded.is_none() || player.move_input == Vec2::ZERO {
        return;
    }
    let feet = player.position;
    let Some((key, _, entity, lying)) = rack.iter().map(|(e, w, t, lying)| (w.0, t.translation(), e, lying))
        .find(|(_, at, _, _)| Vec2::new(at.x - feet.x, at.z - feet.z).length() < TAKE_REACH) else { return };
    let name = CHARACTERS[player.character];
    let slot = player.weapon;
    let list = game.0.character_weapons.entry(name.to_string()).or_default();
    if list.get(slot) == Some(&key) {
        return;
    }
    if slot < list.len() { list[slot] = key } else { list.push(key) }
    let label = game.0.weapons.get(&key).map(|d| d.label.clone()).unwrap_or_default();
    status.message = Some((format!("Took {label}"), MESSAGE_TIME));
    // (one laid on the ground is gone; the rack's stay)
    if lying {
        commands.entity(entity).despawn();
    }
    if let Some(old) = player.loaded.take() {
        commands.entity(old.root).despawn();
    }
    pending.0 = Some(slot);
}

/// The rack's weapons turn slowly.
fn spin_rack(time: Res<Time>, mut rack: Query<&mut Transform, (With<RackWeapon>, Without<Lying>)>) {
    let turn = Quat::from_rotation_y(RACK_SPIN * frame_dt(&time));
    for mut t in &mut rack {
        t.rotation = turn * t.rotation;
    }
}

