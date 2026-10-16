//! The test tools' scenery menu (#185, the first part of the map creation epic #190): any level's
//! buildings and static scenery, put on the test map or on whatever level a test session is on.
//! Demo tooling, not the game: the key (P), the layout and the limits are the demo's choices.
//!
//!  - The list: the levels of the game's list (play_testworld.rs `level_list`), each with its
//!    static scenery by bf/pieces.rs's rule (the game-objects whose type is OT_STATIC_GEOMETRY and
//!    neither breaks nor opens), by its type's name (a hash: the data names none), the largest
//!    first, with its size, its parts and how many its level places. Read from the archives on
//!    threads (`pieces::scan_all`) the first time the menu opens, once a session.
//!  - The menu: Up / Down picks, Page Up / Down jumps a level, the wheel turns the copy 15
//!    degrees a notch, a click or Enter puts one there (the menu stays open for more), P or Esc
//!    closes it. As the object menu's (O): a see-through copy stands at the crosshair; in the free
//!    camera it hangs in the air on the crosshair's ray (Shift + the wheel: how far), a yellow
//!    ring at its foot, and it's put where it hangs (scenery never moves).
//!  - A piece's origin goes where it's put, as its level places it: a building's floor on the
//!    ground (what its level sinks into its terrain goes under the floor).
//!  - Loading: a picked piece is built from its own archive, read on a thread
//!    (`Game::load_scenery`: its objects, materials and textures), and drawn as its level draws
//!    its objects (level_scene.rs `object_parts`: its materials, cut-outs, glows, grass), lit by
//!    the map's lights and lamps, casting no shadow.
//!  - Collision: its parts' collision surfaces (arena.rs `physics_tris`) join the map's
//!    (`Arena::place`; the first solid one gives the flat test floor an arena of its ground): the
//!    squad walks on and around it, shots, grenades, missiles and the crosshair meet it. A piece
//!    with none is walked and shot through, as in its level.
//!  - Memory: one archive's data is held at a time (the last one a piece was built from), and
//!    the pieces built (their meshes, materials and collision): every placed one, and at most
//!    KEEP_UNPLACED others, the least recently picked dropped first.
//!
//! Not done: moving and deleting placed pieces (#187), saving a layout (#188).

use std::sync::{mpsc::Receiver, Mutex};
use std::time::Instant;

use super::*;
use super::testtools::{crosshair, spot_at, Hook, Panel, Script, Spot, Tools};
use bf_viewer::bf::pieces::Piece;
use bf_viewer::level_scene::{Lamps, LevelMaterial, ObjectPart};

/// How many pieces built but not placed are kept (their meshes, materials and collision), the
/// least recently picked dropped first: what browsing holds beyond the one archive (the demo's).
const KEEP_UNPLACED: usize = 6;
/// The hooks' placements at x, z look for the floor under this height above the controlled
/// character (m): the object menu's 50 (play_testtools.rs `object_tool`).
const FLOOR_FROM: f32 = 50.0;

/// A level of the list and its static scenery, from the scan.
struct LevelPieces {
    zone: String,
    title: String,
    pieces: Vec<Piece>,
}

/// The scan's result: per level of the list (its zone and title) its pieces or why its archive
/// couldn't be read, and how long it took (s).
struct Scan(Vec<(String, String, Result<Vec<Piece>, String>)>, f32);

/// The session's list (kept across level switches): the levels with static scenery and the
/// menu's entries (a level and its piece, by index) in order; the scan while it runs.
#[derive(Resource, Default)]
struct Lists {
    levels: Vec<LevelPieces>,
    entries: Vec<(usize, usize)>,
    scan: Option<Mutex<Receiver<Scan>>>,
    scanned: bool,
}

/// An archive read on a thread to build pieces from: a Game holding its data, how many bytes of
/// vertex, collision and texture data that is, and how long it took (s).
type ArchiveLoad = Result<(Game, usize, f32), String>;

/// The archive held to build from: its name, its data and its size (bytes, `ArchiveLoad`), and
/// the materials made from it so far (shared by its pieces).
struct Held {
    archive: String,
    game: Game,
    bytes: usize,
    cache: HashMap<u32, Handle<LevelMaterial>>,
}

/// A piece built: its menu entry, its drawn parts, its collision triangles (its own frame, with
/// their materials), how many are placed and when it was last wanted (`Menu::clock`).
struct Built {
    entry: usize,
    parts: Vec<ObjectPart>,
    collision: Arc<Vec<([Vec3; 3], u8)>>,
    placed: u32,
    used: u64,
}

/// Where a placement goes: the crosshair's spot when it was asked for (a click), or the hooks'
/// x, z and height above the floor there.
#[derive(Clone, Copy)]
enum Where {
    Spot(Spot),
    At(Vec2, f32),
}

/// A placement waiting for its piece to be built: the entry, where, its turn (rad).
struct Pending {
    entry: usize,
    at: Where,
    yaw: f32,
}

/// The menu on this map (`start_map` makes it anew): the entry picked and its turn, the
/// see-through copy, the hooks waiting for the list, the placements waiting for their piece, the
/// archive loading and the one held, the pieces built, the map's lamps, a clock for the pieces'
/// use, and how many pieces are placed.
#[derive(Resource, Default)]
struct Menu {
    pick: usize,
    yaw: f32,
    ghost: Option<(Entity, usize)>,
    want_pick: Option<(String, f32)>,
    want_place: Vec<(Option<String>, Option<(Vec2, f32, f32)>)>,
    pending: Vec<Pending>,
    loading: Option<(String, Mutex<Receiver<ArchiveLoad>>)>,
    held: Option<Held>,
    built: Vec<Built>,
    lamps: Option<Lamps>,
    clock: u64,
    placed: u32,
}

/// The menu's text, shown by play_testtools.rs's panel.
#[derive(Resource, Default)]
pub(super) struct SceneryText(pub(super) String);

/// A piece of scenery put down: the level it's from and its type.
#[derive(Component)]
pub(super) struct PlacedScenery {
    pub(super) zone: String,
    pub(super) kind: u32,
}

/// The see-through copy at the crosshair.
#[derive(Component)]
struct SceneryGhost;

/// What building a piece makes: meshes, the level materials (and their textures), the lamps'
/// buffer.
#[derive(bevy::ecs::system::SystemParam)]
struct SceneAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<LevelMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    buffers: ResMut<'w, Assets<bevy::render::storage::ShaderStorageBuffer>>,
}

/// The scenery menu: its list (the session's), its state on each map, and the menu's system
/// (after the tool registry's keys, `editor::open_tools`: P is its key there).
pub fn plugin(app: &mut App) {
    app.init_resource::<Lists>().init_resource::<Menu>().init_resource::<SceneryText>()
        .add_systems(OnEnter(AppState::Playing), start_map.after(setup).run_if(resource_exists::<super::testmap::TestMap>))
        .add_systems(Update, scenery_tool.after(read_input).after(super::editor::open_tools).before(squad_control)
            .run_if(super::testtools::active));
}

/// A map starts: nothing built, held or waiting (what the last map built went with it).
fn start_map(mut menu: ResMut<Menu>, mut text: ResMut<SceneryText>) {
    *menu = Menu::default();
    text.0.clear();
}

/// A thread's result, if it's in: its own, or `stopped` if the thread ended without one (it
/// panicked), so nothing waits on it for ever.
fn poll<T>(rx: &Mutex<Receiver<T>>, stopped: impl FnOnce() -> T) -> Option<T> {
    match rx.lock().ok()?.try_recv() {
        Ok(x) => Some(x),
        Err(std::sync::mpsc::TryRecvError::Empty) => None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(stopped()),
    }
}

/// An entry by name: `<level>:<piece>`, the piece its type's or its archetype's `h_xxxxxxxx` or
/// its place in the level's list (0 the largest); without a level, the first level's that has
/// that type.
fn find_piece(lists: &Lists, s: &str) -> Option<usize> {
    let (zone, piece) = s.split_once(':').map_or((None, s), |(z, p)| (Some(z.trim()), p.trim()));
    let key = piece.strip_prefix("h_").and_then(|x| u32::from_str_radix(x, 16).ok());
    let nth = piece.parse::<usize>().ok();
    lists.entries.iter().position(|&(l, p)| {
        let level = &lists.levels[l];
        zone.is_none_or(|z| level.zone.eq_ignore_ascii_case(z)) && match (key, nth) {
            (Some(k), _) => level.pieces[p].kind == k || level.pieces[p].archetype == k,
            (None, Some(n)) => p == n,
            _ => false,
        }
    })
}

/// A piece's line in the menu: its type, size, parts, how many its level places, and whether it
/// has no collision.
fn label(p: &Piece) -> String {
    let size = if p.size < 10.0 { format!("{:.1} m", p.size) } else { format!("{:.0} m", p.size) };
    let parts = if p.parts > 1 { format!(", {} parts", p.parts) } else { String::new() };
    let solid = if p.solid { "" } else { ", no collision" };
    format!("h_{:08x}  {size}{parts}{solid}  (x{} in its level)", p.kind, p.placed)
}

/// The archive a piece is built from: its level's, or common's for the two types it shares
/// (`Piece::common`).
fn source(lists: &Lists, entry: usize) -> (&str, &Piece) {
    let (l, p) = lists.entries[entry];
    let piece = &lists.levels[l].pieces[p];
    (if piece.common { "common" } else { lists.levels[l].zone.as_str() }, piece)
}

/// A see-through copy of a level material (the object menu's look: GHOST_ALPHA, a faint glow);
/// an additive one (a glow shell) is see-through already.
fn see_through(materials: &mut Assets<LevelMaterial>, m: &Handle<LevelMaterial>) -> Handle<LevelMaterial> {
    let Some(mut x) = materials.get(m).cloned() else { return m.clone() };
    if x.base.alpha_mode == AlphaMode::Add {
        return m.clone();
    }
    x.base.base_color = x.base.base_color.with_alpha(super::testtools::GHOST_ALPHA);
    x.base.alpha_mode = AlphaMode::Blend;
    x.base.emissive = super::testtools::GHOST_GLOW;
    materials.add(x)
}

/// The scenery menu (P, the tool registry's key) and its hooks: the list, picking, the
/// see-through copy, loading and building the pieces wanted, placing them (each listed in the
/// editor's `Placements`).
#[allow(clippy::too_many_arguments)]
fn scenery_tool(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, mut placements: ResMut<super::editor::Placements>,
                scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>, player: Res<Player>, game: Res<GameData>, map: Res<MapLevel>,
                script: Res<Script>, mut status: ResMut<UsePanel>, (mut lists, mut menu, mut text): (ResMut<Lists>, ResMut<Menu>, ResMut<SceneryText>),
                (mut ghosts, mut gizmos): (Query<(&mut Transform, &mut Visibility), With<SceneryGhost>>, Gizmos),
                placed: Query<&PlacedScenery>, mut assets: SceneAssets) {
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let menu = &mut *menu;
    // the list, once the scan is in
    let got = lists.scan.as_ref().and_then(|r| poll(r, || Scan(vec![], 0.0)));
    if let Some(Scan(found, secs)) = got {
        lists.scan = None;
        lists.scanned = true;
        let (mut placements, mut kinds) = (0, std::collections::HashSet::new());
        for (zone, title, r) in found {
            match r {
                Ok(pieces) if !pieces.is_empty() => {
                    placements += pieces.iter().map(|p| p.placed).sum::<u32>();
                    kinds.extend(pieces.iter().map(|p| p.kind));
                    // (BF_TOOLS_LOG: each level's count, and each piece as the menu lists it)
                    if log {
                        println!("scenery: {zone}: {} pieces, {} placed in its level", pieces.len(), pieces.iter().map(|p| p.placed).sum::<u32>());
                        for (i, p) in pieces.iter().enumerate() {
                            println!("  {zone}:{i} {} arch h_{:08x}{}", label(p), p.archetype, if p.common { " (common's)" } else { "" });
                        }
                    }
                    lists.levels.push(LevelPieces { zone, title, pieces });
                }
                Ok(_) => {}
                Err(e) => eprintln!("scenery: no list from {zone}: {e}"),
            }
        }
        lists.entries = lists.levels.iter().enumerate().flat_map(|(l, v)| (0..v.pieces.len()).map(move |p| (l, p))).collect();
        let mb = super::testworld::process_mb().map_or(String::new(), |(ws, _)| format!("; process {ws:.0} MB"));
        println!("test tools: scenery of {} levels, {} pieces ({placements} placements of {} types), listed in {secs:.1} s{mb}",
                 lists.levels.len(), lists.entries.len(), kinds.len());
    }
    // the hooks (P is the tool registry's)
    for hook in &script.now {
        match hook {
            Hook::SceneryMenu => tools.panel = Panel::Scenery,
            Hook::Scenery { piece, yaw } => {
                tools.panel = Panel::Scenery;
                menu.want_pick = Some((piece.clone(), *yaw));
            }
            Hook::SceneryPlace { piece, at } => menu.want_place.push((piece.clone(), *at)),
            _ => {}
        }
    }
    let open = tools.panel == Panel::Scenery;
    // the scan, the first time the list is wanted
    if (open || menu.want_pick.is_some() || !menu.want_place.is_empty()) && !lists.scanned && lists.scan.is_none() {
        let (tx, rx) = std::sync::mpsc::channel();
        let schemas = game.0.schemas.clone();
        let zones: Vec<(String, String)> = super::testworld::level_list(&game.0).into_iter().filter(|e| e.zone != "flat")
            .map(|e| (e.zone, e.title)).collect();
        std::thread::spawn(move || {
            let t = Instant::now();
            let names: Vec<String> = zones.iter().map(|z| z.0.clone()).collect();
            let found = bf_viewer::bf::pieces::scan_all(&schemas, &data_dir(), &names);
            let out = zones.into_iter().zip(found).map(|((zone, title), (_, r))| (zone, title, r)).collect();
            let _ = tx.send(Scan(out, t.elapsed().as_secs_f32()));
        });
        lists.scan = Some(Mutex::new(rx));
    }
    let (origin, dir) = crosshair(&player, &tools);
    let spot = spot_at(origin, dir, tools.freecam.is_some(), tools.air);
    let n = lists.entries.len();
    // the hooks' picks and placements, once the list is in
    if lists.scanned {
        if let Some((p, yaw)) = menu.want_pick.take() {
            match find_piece(&lists, &p) {
                Some(i) => {
                    menu.pick = i;
                    menu.yaw = yaw;
                }
                None => println!("test tools: no scenery {p}"),
            }
        }
        for (p, at) in std::mem::take(&mut menu.want_place) {
            let entry = match &p {
                Some(p) => match find_piece(&lists, p) {
                    Some(i) => i,
                    None => {
                        println!("test tools: no scenery {p}");
                        continue;
                    }
                },
                None if n > 0 => menu.pick,
                None => continue,
            };
            match (at, spot) {
                (Some((xz, yaw, up)), _) => menu.pending.push(Pending { entry, at: Where::At(xz, up), yaw }),
                (None, Some(s)) => {
                    if p.is_some() {
                        menu.pick = entry;
                    }
                    menu.pending.push(Pending { entry, at: Where::Spot(s), yaw: menu.yaw });
                }
                (None, None) => status.message = Some(("Nothing under the crosshair".into(), super::testtools::MESSAGE_TIME)),
            }
        }
    }
    // the open menu's keys
    if open && n > 0 {
        let step = |k: KeyCode| keys.just_pressed(k) as i32;
        let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp);
        menu.pick = (menu.pick as i32 + d).rem_euclid(n as i32) as usize;
        // Page Up / Down: the first piece of the previous / next level
        let level = lists.entries[menu.pick].0;
        if keys.just_pressed(KeyCode::PageDown) {
            menu.pick = (menu.pick..n).find(|&i| lists.entries[i].0 != level).unwrap_or(0);
        }
        if keys.just_pressed(KeyCode::PageUp) {
            let before = if level == 0 { lists.levels.len() - 1 } else { level - 1 };
            menu.pick = lists.entries.iter().position(|e| e.0 == before).unwrap_or(0);
        }
        // the wheel turns it; with Shift, in the free camera, it moves it along the crosshair
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let wheel = if scroll.delta.y != 0.0 { scroll.delta.y } else { scroll.delta.x };
        let (air_step, air_range) = (super::testtools::AIR_STEP, super::testtools::AIR_RANGE);
        if wheel != 0.0 && shift && tools.freecam.is_some() {
            tools.air = (tools.air + air_step * wheel.signum()).clamp(air_range.0, air_range.1);
        } else if scroll.delta.y != 0.0 {
            menu.yaw = wrap_angle(menu.yaw + super::testtools::TURN_STEP * scroll.delta.y.signum());
        }
        if keys.just_pressed(KeyCode::Enter) || tools.click {
            match spot {
                Some(s) => menu.pending.push(Pending { entry: menu.pick, at: Where::Spot(s), yaw: menu.yaw }),
                None => status.message = Some(("Nothing under the crosshair".into(), super::testtools::MESSAGE_TIME)),
            }
        }
    }
    // the archive loading, once in: held in place of the last
    let got = menu.loading.as_ref().and_then(|(a, r)| poll(r, || Err("the loading thread stopped".into())).map(|x| (a.clone(), x)));
    if let Some((archive, load)) = got {
        menu.loading = None;
        match load {
            Ok((g, bytes, secs)) => {
                if log {
                    let mb = super::testworld::process_mb().map_or(String::new(), |(ws, _)| format!("; process {ws:.0} MB"));
                    println!("scenery: {archive} loaded in {secs:.2} s ({:.1} MB of vertex, collision and texture data){mb}", bytes as f32 / 1048576.0);
                }
                menu.held = Some(Held { archive, game: g, bytes, cache: HashMap::new() });
            }
            Err(e) => {
                eprintln!("scenery: couldn't load {archive}: {e}");
                status.message = Some((format!("No scenery from {archive}"), super::testtools::MESSAGE_TIME));
                // (what waits on it is dropped: it would ask again for ever)
                menu.pending.retain(|p| source(&lists, p.entry).0 != archive);
            }
        }
    }
    // build what's wanted: the placements waiting first, then the pick; one a frame, from the
    // archive held (else that archive is loaded first, one at a time)
    menu.clock += 1;
    let wanted: Vec<usize> = menu.pending.iter().map(|p| p.entry).chain((open && n > 0).then_some(menu.pick)).collect();
    for &e in &wanted {
        if let Some(b) = menu.built.iter_mut().find(|b| b.entry == e) {
            b.used = menu.clock;
            continue;
        }
        let (archive, piece) = source(&lists, e);
        if menu.held.as_ref().is_some_and(|h| h.archive == archive) {
            let t = Instant::now();
            let lamps = menu.lamps.get_or_insert_with(|| bf_viewer::level_scene::lamps_of(map.0.as_ref(), &mut assets.buffers)).clone();
            let held = menu.held.as_mut().expect("held");
            let parts = bf_viewer::level_scene::object_parts(&mut held.game, piece.archetype, &mut assets.meshes, &mut assets.materials,
                                                             &mut assets.images, &lamps, &mut held.cache);
            let collision: Vec<([Vec3; 3], u8)> = bf_viewer::arena::physics_tris(&held.game, piece.archetype).into_iter().map(|(t, _, m)| (t, m)).collect();
            if log {
                let image_mb = assets.images.iter().map(|(_, i)| i.data.as_ref().map_or(0, |d| d.len())).sum::<usize>() as f32 / 1048576.0;
                let mb = super::testworld::process_mb().map_or(String::new(), |(ws, _)| format!("; process {ws:.0} MB"));
                let (lo, hi) = collision.iter().flat_map(|(t, _)| t).fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(*v), u.max(*v)));
                println!("scenery: built {archive} h_{:08x} in {:.0} ms: {} meshes, {} collision triangles (its frame {lo:.1} .. {hi:.1}); held {archive} ({:.1} MB), {} built ({} placed); meshes {}, images {} ({image_mb:.0} MB), level materials {}{mb}",
                         piece.kind, t.elapsed().as_secs_f32() * 1000.0, parts.len(), collision.len(), held.bytes as f32 / 1048576.0,
                         menu.built.len() + 1, menu.built.iter().filter(|b| b.placed > 0).count(), assets.meshes.len(), assets.images.len(),
                         assets.materials.len());
            }
            if parts.is_empty() {
                println!("test tools: {archive} h_{:08x} has nothing to draw", piece.archetype);
            }
            menu.built.push(Built { entry: e, parts, collision: Arc::new(collision), placed: 0, used: menu.clock });
            break;
        }
        if menu.loading.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            let t = Instant::now();
            let base = game.0.scenery_base();
            let base_ms = t.elapsed().as_secs_f32() * 1000.0;
            let name = archive.to_string();
            std::thread::spawn(move || {
                let t = Instant::now();
                let mut g = base;
                let load = g.load_scenery(&data_dir(), &name).map(|bytes| (g, bytes, t.elapsed().as_secs_f32()));
                let _ = tx.send(load);
            });
            if log {
                println!("scenery: loading {archive} (its Game started in {base_ms:.1} ms)");
            }
            status.message = Some((format!("Loading the scenery of {archive}"), super::testtools::MESSAGE_TIME));
            menu.loading = Some((archive.to_string(), Mutex::new(rx)));
        }
        break;
    }
    // the pieces built but not placed nor wanted: at most KEEP_UNPLACED, the least recently
    // wanted dropped first (their meshes and materials go with their handles)
    loop {
        let spare: Vec<usize> = (0..menu.built.len()).filter(|&i| menu.built[i].placed == 0 && !wanted.contains(&menu.built[i].entry)).collect();
        if spare.len() <= KEEP_UNPLACED {
            break;
        }
        let Some(&oldest) = spare.iter().min_by_key(|&&i| menu.built[i].used) else { break };
        let b = menu.built.remove(oldest);
        if log {
            let (archive, piece) = source(&lists, b.entry);
            println!("scenery: dropped {archive} h_{:08x} (not placed)", piece.kind);
        }
    }
    // the placements whose piece is built
    let mut k = 0;
    while k < menu.pending.len() {
        let Some(b) = menu.built.iter().position(|b| b.entry == menu.pending[k].entry) else {
            k += 1;
            continue;
        };
        let p = menu.pending.remove(k);
        let at = match p.at {
            Where::Spot(s) => s,
            Where::At(xz, up) => {
                let below = Vec3::new(xz.x, floor_y(xz.x, xz.y, player.position.y + GROUND + FLOOR_FROM), xz.y);
                if up > super::testtools::AIR_MIN_HEIGHT { Spot::Air { at: below + Vec3::Y * up, below } } else { Spot::Ground(below) }
            }
        };
        let (zone, piece) = (lists.levels[lists.entries[p.entry].0].zone.clone(), source(&lists, p.entry).1);
        let place = Transform::from_translation(at.at()).with_rotation(Quat::from_rotation_y(p.yaw));
        let root = commands.spawn((place, Visibility::default(), PlacedScenery { zone: zone.clone(), kind: piece.kind },
                                   Name::new(format!("scenery {zone} h_{:08x}", piece.kind)))).id();
        let thing = super::editor::Thing::Scenery { zone: zone.clone(), kind: piece.kind, archetype: piece.archetype };
        let id = placements.add(thing, at.at(), p.yaw, matches!(at, Spot::Air { .. }), Some(root));
        commands.entity(root).insert(super::editor::PlacedId(id));
        let built = &mut menu.built[b];
        bf_viewer::level_scene::spawn_object_parts(&mut commands, &built.parts, root, |part| part.material.clone());
        // its collision into the map's (a piece with none leaves the arena as it is)
        let t = Instant::now();
        let arena = if built.collision.is_empty() { world::arena() } else { Some(world::Arena::place(&built.collision, place.compute_matrix(), GROUND)) };
        let arena_ms = t.elapsed().as_secs_f32() * 1000.0;
        built.placed += 1;
        built.used = menu.clock;
        menu.placed += 1;
        status.message = Some((format!("Placed {zone} h_{:08x}", piece.kind), super::testtools::MESSAGE_TIME));
        if log {
            let how = match at {
                Spot::Ground(_) => "on the ground".to_string(),
                Spot::Air { at, below } => format!("{:.2} m up (hangs)", at.y - below.y),
            };
            println!("t {:.2}: placed scenery {zone} h_{:08x} (arch h_{:08x}, {:.1} m) at {:.2} {how} turned {:.0} deg: {} collision triangles; arena {} triangles (made in {arena_ms:.0} ms); {} placed on this map",
                     player.sim_time, piece.kind, piece.archetype, piece.size, at.at(), p.yaw.to_degrees(), built.collision.len(),
                     arena.map_or(0, |a| a.stats().0), menu.placed);
        }
    }
    // the see-through copy: the picked piece, once built, at the crosshair
    let want = (open && n > 0 && menu.built.iter().any(|b| b.entry == menu.pick)).then_some(menu.pick);
    if menu.ghost.map(|g| g.1) != want {
        if let Some((e, _)) = menu.ghost.take() {
            commands.entity(e).despawn();
        }
        if let Some(i) = want {
            let root = commands.spawn((Transform::default(), Visibility::Hidden, SceneryGhost, Name::new("scenery ghost"))).id();
            let parts = &menu.built.iter().find(|b| b.entry == i).expect("built").parts;
            let mut copies: HashMap<AssetId<LevelMaterial>, Handle<LevelMaterial>> = HashMap::new();
            let materials = &mut assets.materials;
            let copy = |p: &ObjectPart| copies.entry(p.material.id()).or_insert_with(|| see_through(materials, &p.material)).clone();
            bf_viewer::level_scene::spawn_object_parts(&mut commands, parts, root, copy);
            menu.ghost = Some((root, i));
        }
    }
    if let Some((e, _)) = menu.ghost {
        if let Ok((mut t, mut v)) = ghosts.get_mut(e) {
            match spot {
                Some(s) => {
                    *t = Transform::from_translation(s.at()).with_rotation(Quat::from_rotation_y(menu.yaw));
                    *v = Visibility::Inherited;
                }
                None => *v = Visibility::Hidden,
            }
        }
        // in the air: a ring at its foot, where it will hang
        if let Some(Spot::Air { at, .. }) = spot {
            let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            gizmos.circle(Isometry3d::new(at, flat), super::testtools::MARK_RING, super::testtools::HANG_MARK);
        }
    }
    // the panel's text
    let s = if open { panel_text(&lists, menu, &tools, &placed) } else { String::new() };
    if text.0 != s {
        text.0 = s;
    }
}

/// The menu's text (see `SceneryText`): a window of the list round the pick, by level, and what's
/// loading.
fn panel_text(lists: &Lists, menu: &Menu, tools: &Tools, placed: &Query<&PlacedScenery>) -> String {
    let mut s = String::from("PLACE SCENERY   (P / Esc: close)\n");
    if !lists.scanned {
        s += "  (listing every level's scenery...)\n";
        return s;
    }
    let n = lists.entries.len();
    if n == 0 {
        s += "  (no scenery found)\n";
        return s;
    }
    let lines = super::testtools::LIST_LINES;
    let first = menu.pick.saturating_sub(lines / 2).min(n.saturating_sub(lines));
    let mut level = usize::MAX;
    for i in first..(first + lines).min(n) {
        let (l, p) = lists.entries[i];
        if l != level {
            level = l;
            s += &format!("  [{}  {}]\n", lists.levels[l].zone, lists.levels[l].title);
        }
        s += &format!("{} {}\n", if i == menu.pick { ">" } else { " " }, label(&lists.levels[l].pieces[p]));
    }
    s += &format!("  ({} of {n} pieces, level {} of {})\n", menu.pick + 1, lists.entries[menu.pick].0 + 1, lists.levels.len());
    s += &format!("Up / Down pick   Page Up / Down level   wheel turn ({:.0} deg)\n", menu.yaw.to_degrees());
    let at = super::testtools::aim_word(tools);
    s += &if tools.freecam.is_some() {
        format!("click / Enter: place IN THE AIR, {:.1} m along {at} (Shift + wheel); it hangs there\n", tools.air)
    } else {
        format!("click / Enter: place on the ground at {at}\n")
    };
    let (archive, _) = source(lists, menu.pick);
    if let Some((a, _)) = &menu.loading {
        s += &format!("loading {a}...\n");
    } else if !menu.built.iter().any(|b| b.entry == menu.pick) {
        s += &format!("building ({archive})...\n");
    }
    // what's placed on this map, by piece
    let mut counts: Vec<(&str, u32, usize)> = vec![];
    for p in placed {
        match counts.iter_mut().find(|c| c.0 == p.zone && c.1 == p.kind) {
            Some(c) => c.2 += 1,
            None => counts.push((&p.zone, p.kind, 1)),
        }
    }
    if !counts.is_empty() {
        let list: Vec<String> = counts.iter().take(4).map(|(z, k, c)| format!("{z} h_{k:08x} x{c}")).collect();
        s += &format!("placed on this map: {}{}", list.join(", "), if counts.len() > 4 { ", ..." } else { "" });
    }
    s
}
