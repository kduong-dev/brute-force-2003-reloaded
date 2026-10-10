//! The test tools' world menus (#116), beside play_testtools.rs's (#111): a sky picker, a music
//! picker and a level switch. Demo tooling, not the game: the keys and the layout are the demo's
//! choices, none of them used by the game's controls. In a test session (`--test`) on any map.
//!
//!  - Y: the sky menu. "none", the map's own, or any level's sky: every level whose level file
//!    names a sky mesh (`Level::sky_mesh`, read from the archives when the session starts). The
//!    chosen level's sky (its layers, as `level_scene::spawn_level` draws them) replaces the
//!    map's, and its background colour the clear colour; on the flat test floor the camera then
//!    sees as far as a level's (SKY_FAR) to reach it.
//!  - U: the music menu. "off", the map's own, or any level's music bank (data/sounds/<level>.xwb
//!    with tracks), mixed as the game mixes a level's (play.rs `music_mix`: the music, its
//!    ambience bed under it, by the Types of that level's sound bank). Left / Right: the previous
//!    / next bank at once.
//!  - L: the level menu: the test map, then the game's level list (common campaign-bf.xmb: the
//!    squad deathmatch maps, the deathmatch arenas, the missions' zones). Enter loads the chosen
//!    one without restarting: AppState::Switching tears the map down (`end_play`, as Backspace
//!    does), loads the next on a thread as the front end does (with the test map's weapon data,
//!    so the object tool has its whole list) and plays it. The tools, instant kill and the help
//!    panel stay. The deathmatch arenas are played alone, as the front end plays them; the rest
//!    with the squad. A map that won't load (no level file) puts the test map up instead, saying
//!    so.
//!
//! Up / Down or the wheel picks in each menu, a click or Enter takes the pick; the key again or
//! Esc closes it.

use std::sync::{mpsc::Receiver, Mutex};

use super::*;
use super::testtools::{Hook, Panel, Script, Session, Tools};
use bf_viewer::bf::bxml::SchemaSet;
use bf_viewer::bf::character::{Geoset, LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};
use bf_viewer::level_scene::{SkyLayer, SkyLayers};
use super::testmap::TestMap;

/// How many lines of a list show at once.
const LIST_LINES: usize = 10;
/// How long a message shows (s): the test map's.
const MESSAGE_TIME: f32 = 2.0;
/// When a map's start is logged (s of the controlled character's clock, BF_TOOLS_LOG; the
/// demo's): once its scenery and pickups are listed.
const AUDIT_AT: f32 = 0.5;
/// How much of a music bank's file is read to list its tracks (bytes): its header, entry table
/// and names come first (in data/sounds the names end by byte 868 at most: m09_b.xwb's, 9 entries).
const XWB_HEAD: u64 = 16 * 1024;
/// How far the camera sees (m) under another level's sky: as far as on a level (setup's
/// 5000). The flat floor's camera has Bevy's 1000, short of sdm_e34's sky (3.2 km across).
const SKY_FAR: f32 = 5000.0;

/// A level the menus list: its archive's name ("flat": the test map), the name the game gives
/// it (`CampaignLevel::name`; a mission's zones by their archive too), its group, and whether
/// the front end plays it alone (the deathmatch arenas).
#[derive(Clone, Debug)]
struct Entry {
    zone: String,
    title: String,
    group: &'static str,
    deathmatch: bool,
}

/// What the scan of the levels found, per level (an index into `Lists::levels`): whether its
/// level file names a sky mesh, and its music bank's tracks; and how long it took (s).
struct Scan(Vec<(usize, bool, Vec<String>)>, f32);

/// The session's lists, kept across level switches: the levels, those with a sky and those with a
/// music bank (once the scan is in), and the skies made ready so far (by level: their layers and
/// background colour).
#[derive(Resource, Default)]
struct Lists {
    levels: Vec<Entry>,
    skies: Vec<usize>,
    banks: Vec<(usize, Vec<String>)>,
    scanned: bool,
    scan: Option<Mutex<Receiver<Scan>>>,
    ready: HashMap<String, (SkyLayers, [f32; 3])>,
}

/// A level's sky loaded on a thread: the game data holding its meshes and textures (dropped
/// once they're made ready), its layers, where they sit and its background colour.
type SkyLoad = Result<(Game, Vec<Geoset>, Vec3, [f32; 3]), String>;
/// A level's music bank decoded and mixed on a thread (`music_mix`).
type MusicLoad = Result<Vec<(String, Vec<u8>, &'static str, f32)>, String>;
/// A level loaded on a thread for the switch, and a note to show (when it fell back).
type LevelLoad = Result<(LoadedMap, Option<String>), String>;

/// The map's state of the menus (`start_map` as each map starts): the picks, what's shown and
/// playing, the map's own clear colour (before a sky changed it), what's loading.
#[derive(Resource, Default)]
struct World {
    sky_pick: usize,
    music_pick: usize,
    level_pick: usize,
    sky: String,
    music: String,
    own_clear: Option<Color>,
    sky_loading: Option<(String, Mutex<Receiver<SkyLoad>>)>,
    music_loading: Option<(String, Mutex<Receiver<MusicLoad>>)>,
    /// the bank playing (an index into `Lists::banks`), for next / previous
    bank: Option<usize>,
    /// whether the map's start has been logged (BF_TOOLS_LOG)
    audited: bool,
}

/// The level switch asked for, and (once it's loading) the thread's result; where from; and the
/// levels that failed to load on the way (the fall back: the map it came from, then the test
/// map), with why.
#[derive(Resource)]
struct Switch {
    to: Entry,
    from: String,
    rx: Option<Mutex<Receiver<LevelLoad>>>,
    failed: Vec<String>,
}

/// A note for the next map's first frame (a level that didn't load).
#[derive(Resource, Default)]
struct Notice(Option<String>);

/// The open world menu's text, shown by play_testtools.rs's panel.
#[derive(Resource, Default)]
pub(super) struct WorldText(pub(super) String);

/// The level switch's loading screen.
#[derive(Component)]
struct SwitchPart;

pub fn plugin(app: &mut App) {
    app.init_resource::<Lists>().init_resource::<World>().init_resource::<WorldText>().init_resource::<Notice>()
        .add_systems(OnEnter(AppState::Playing), start_map.after(setup).run_if(resource_exists::<TestMap>))
        .add_systems(Update, (world_menus, finish_loads, audit, world_text).chain().after(update_player).before(super::testtools::show_panel)
            .run_if(super::testtools::active))
        .add_systems(OnEnter(AppState::Switching), start_switch)
        .add_systems(Update, finish_switch.run_if(in_state(AppState::Switching)))
        .add_systems(OnExit(AppState::Switching), end_switch);
}

/// Whether `zone` is a deathmatch arena by the game's level list (campaign-bf.xmb Type
/// LEVEL_DEATHMATCH): the front end plays those alone, and so does a test session, whether it
/// starts on one (`--test` with BF_MAP, play.rs `main`) or switches to one.
pub fn arena(game: &Game, zone: &str) -> bool {
    game.campaign.iter().any(|l| l.kind == LEVEL_DEATHMATCH && (l.file == zone || l.zones.iter().any(|z| z == zone)))
}

/// The game's level list as the level menu shows it: the test map, then the squad deathmatch
/// maps, the deathmatch arenas and the missions (each zone), each group in the game's order.
fn level_list(game: &Game) -> Vec<Entry> {
    let mut out = vec![Entry { zone: "flat".into(), title: "Test map".into(), group: "test", deathmatch: false }];
    let s = |id: u32| game.strings.get(&id).cloned().filter(|t| !t.trim().is_empty());
    for (kind, group) in [(LEVEL_SQUAD_DEATHMATCH, "squad deathmatch"), (LEVEL_DEATHMATCH, "deathmatch"), (0, "missions")] {
        for l in game.campaign.iter().filter(|l| if kind == 0 { l.kind != LEVEL_DEATHMATCH && l.kind != LEVEL_SQUAD_DEATHMATCH } else { l.kind == kind }) {
            let zones = if l.zones.is_empty() { vec![l.file.clone()] } else { l.zones.clone() };
            for z in &zones {
                if out.iter().any(|e| &e.zone == z) {
                    continue;
                }
                let title = s(l.name).map(|t| t.trim().to_string()).unwrap_or_else(|| z.clone());
                out.push(Entry { zone: z.clone(), title, group, deathmatch: arena(game, z) });
            }
        }
    }
    out
}

/// Which levels have a sky and a music bank (see `Scan`): each one's level file alone (its
/// archive read up to it) and the start of its music bank.
fn scan(schemas: SchemaSet, levels: Vec<String>) -> Scan {
    let t = std::time::Instant::now();
    let dir = data_dir();
    let out = levels.iter().enumerate().filter(|(_, z)| *z != "flat").map(|(i, z)| {
        let sky = Game::level_file_of(&schemas, &dir, z).ok().flatten().is_some_and(|r| bf_viewer::bf::level::Level::sky_mesh(&r) != 0);
        let mut head = vec![];
        if let Ok(f) = std::fs::File::open(dir.join("sounds").join(format!("{z}.xwb"))) {
            use std::io::Read;
            let _ = f.take(XWB_HEAD).read_to_end(&mut head);
        }
        (i, sky, bf_viewer::bf::audio::xwb_names(&head))
    }).collect();
    Scan(out, t.elapsed().as_secs_f32())
}

/// A map starts: the menus' state anew (nothing loading carries over), the level list (once a
/// session) and the scan started; the level menu on this map.
fn start_map(mut world: ResMut<World>, mut lists: ResMut<Lists>, game: Res<GameData>, current: Res<CurrentMap>, mut text: ResMut<WorldText>) {
    *world = World::default();
    text.0.clear();
    if lists.levels.is_empty() {
        lists.levels = level_list(&game.0);
    }
    if !lists.scanned && lists.scan.is_none() {
        let (tx, rx) = std::sync::mpsc::channel();
        let (schemas, levels) = (game.0.schemas.clone(), lists.levels.iter().map(|e| e.zone.clone()).collect());
        std::thread::spawn(move || {
            let _ = tx.send(scan(schemas, levels));
        });
        lists.scan = Some(Mutex::new(rx));
    }
    world.level_pick = lists.levels.iter().position(|e| e.zone == current.0).unwrap_or(0);
    world.sky = if current.0 == "flat" { "none".into() } else { format!("{}'s own", current.0) };
    world.music = world.sky.clone();
}

/// A level by name: its archive's name, else the first whose title contains it (any case).
fn find_level(lists: &Lists, name: &str) -> Option<usize> {
    let n = name.trim().to_ascii_lowercase();
    let n = if n == "test" { "flat".to_string() } else { n };
    lists.levels.iter().position(|e| e.zone == n).or_else(|| lists.levels.iter().position(|e| e.title.to_ascii_lowercase().contains(&n)))
}

/// What a menu's pick means: entry 0 "none" / "off", 1 the map's own, then the list's levels.
enum Pick {
    None,
    Own,
    Level(usize),
}

/// The menus (Y, U, L) and their hooks: picking, and asking for a sky, a bank or a switch.
#[allow(clippy::too_many_arguments)]
fn world_menus(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>,
               scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>, mut world: ResMut<World>, mut lists: ResMut<Lists>,
               script: Res<Script>, current: Res<CurrentMap>, game: Res<GameData>, mut status: ResMut<UsePanel>,
               mut next: ResMut<NextState<AppState>>, mut clear: ResMut<ClearColor>, skies: Query<Entity, With<SkyLayer>>,
               music: Query<Entity, With<super::LevelMusic>>) {
    // the scan's lists, once in
    let got = lists.scan.as_ref().and_then(|r| r.lock().ok().and_then(|r| r.try_recv().ok()));
    if let Some(Scan(found, secs)) = got {
        lists.skies = found.iter().filter(|f| f.1).map(|f| f.0).collect();
        lists.banks = found.into_iter().filter(|f| !f.2.is_empty()).map(|f| (f.0, f.2)).collect();
        lists.scanned = true;
        lists.scan = None;
        println!("test tools: {} levels, {} with a sky, {} with a music bank (listed in {secs:.1} s)", lists.levels.len() - 1, lists.skies.len(), lists.banks.len());
    }
    let toggle = |p: Panel, tools: &mut Tools| tools.panel = if tools.panel == p { Panel::None } else { p };
    if keys.just_pressed(KeyCode::KeyY) {
        toggle(Panel::Sky, &mut *tools);
    }
    if keys.just_pressed(KeyCode::KeyU) {
        toggle(Panel::Music, &mut *tools);
    }
    if keys.just_pressed(KeyCode::KeyL) {
        toggle(Panel::Levels, &mut *tools);
    }
    let mut sky: Option<Pick> = None;
    let mut bank: Option<Pick> = None;
    let mut step_bank = 0;
    let mut switch: Option<usize> = None;
    for hook in &script.now {
        match hook {
            Hook::SkyMenu => tools.panel = Panel::Sky,
            Hook::MusicMenu => tools.panel = Panel::Music,
            Hook::LevelMenu => tools.panel = Panel::Levels,
            Hook::Sky(n) => match n.to_ascii_lowercase().as_str() {
                "none" => sky = Some(Pick::None),
                "map" => sky = Some(Pick::Own),
                _ => match find_level(&lists, n) {
                    Some(i) => sky = Some(Pick::Level(i)),
                    None => println!("test tools: no level {n}"),
                },
            },
            Hook::Music(n) => match n.to_ascii_lowercase().as_str() {
                "off" => bank = Some(Pick::None),
                "map" => bank = Some(Pick::Own),
                "next" => step_bank = 1,
                "prev" => step_bank = -1,
                _ => match find_level(&lists, n) {
                    Some(i) => bank = Some(Pick::Level(i)),
                    None => println!("test tools: no level {n}"),
                },
            },
            Hook::Level(n) => match find_level(&lists, n) {
                Some(i) => switch = Some(i),
                None => println!("test tools: no level {n}"),
            },
            _ => {}
        }
    }
    // the open menu's keys
    let step = |k: KeyCode| keys.just_pressed(k) as i32;
    let wheel = if scroll.delta.y < 0.0 { 1 } else if scroll.delta.y > 0.0 { -1 } else { 0 };
    let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp) + wheel;
    let take = keys.just_pressed(KeyCode::Enter) || (mouse.just_pressed(MouseButton::Left) && tools.before.captured);
    let pick_of = |k: usize, list: &[usize]| match k {
        0 => Pick::None,
        1 => Pick::Own,
        k => Pick::Level(list[k - 2]),
    };
    match tools.panel {
        Panel::Sky => {
            let n = lists.skies.len() + 2;
            world.sky_pick = (world.sky_pick as i32 + d).rem_euclid(n as i32) as usize;
            if take {
                sky = Some(pick_of(world.sky_pick, &lists.skies));
            }
        }
        Panel::Music => {
            let banks: Vec<usize> = lists.banks.iter().map(|b| b.0).collect();
            let n = banks.len() + 2;
            world.music_pick = (world.music_pick as i32 + d).rem_euclid(n as i32) as usize;
            if take {
                bank = Some(pick_of(world.music_pick, &banks));
            }
            step_bank += step(KeyCode::ArrowRight) - step(KeyCode::ArrowLeft);
        }
        Panel::Levels => {
            let n = lists.levels.len().max(1);
            world.level_pick = (world.level_pick as i32 + d).rem_euclid(n as i32) as usize;
            if take {
                switch = Some(world.level_pick);
            }
        }
        _ => {}
    }
    if step_bank != 0 {
        if lists.banks.is_empty() {
            status.message = Some(("Still listing the music banks".into(), MESSAGE_TIME));
        } else {
            // (from the bank playing: one picked, else the map's own, else the list's ends)
            let n = lists.banks.len() as i32;
            let own = lists.banks.iter().position(|b| lists.levels[b.0].zone == current.0);
            let k = match world.bank.or(own.filter(|_| world.music != "off")) {
                Some(k) => (k as i32 + step_bank).rem_euclid(n),
                None if step_bank > 0 => 0,
                None => n - 1,
            } as usize;
            world.music_pick = k + 2;
            bank = Some(Pick::Level(lists.banks[k].0));
        }
    }
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    // a sky: none, or a level's (made ready already, or loaded on a thread). While one loads,
    // other picks are ignored (each load reads the game data anew: they'd pile up)
    if let (Some(_), Some((z, _))) = (&sky, &world.sky_loading) {
        status.message = Some((format!("Still loading the sky of {z}"), MESSAGE_TIME));
        if log {
            println!("test tools: sky pick ignored: still loading the sky of {z}");
        }
        sky = None;
    }
    if let Some(p) = sky {
        let zone = match p {
            Pick::None => None,
            Pick::Own => Some(current.0.clone()).filter(|m| m != "flat"),
            Pick::Level(i) => Some(lists.levels[i].zone.clone()),
        };
        world.sky_loading = None;
        match zone {
            None => {
                let own = *world.own_clear.get_or_insert(clear.0);
                for e in &skies {
                    commands.entity(e).despawn();
                }
                clear.0 = own;
                world.sky = "none".into();
                if log {
                    println!("test tools: sky: none");
                }
                status.message = Some(("Sky: none".into(), MESSAGE_TIME));
            }
            Some(z) if lists.ready.contains_key(&z) => {
                world.own_clear.get_or_insert(clear.0);
                world.sky = z.clone();
                let (layers, bg) = &lists.ready[&z];
                show_sky(&mut commands, layers, *bg, &skies, &mut clear);
                if log {
                    println!("test tools: sky: {z} ({} layers, ready)", layers.len());
                }
                status.message = Some((format!("Sky: {z}"), MESSAGE_TIME));
            }
            Some(z) => {
                let (tx, rx) = std::sync::mpsc::channel();
                let name = z.clone();
                std::thread::spawn(move || {
                    let dir = data_dir();
                    let load = (|| {
                        let mut game = Game::load(&dir)?;
                        game.load_level(&dir, &name)?;
                        let (sky, at, bg) = bf_viewer::bf::level::Level::load_sky(&game)?;
                        Ok((game, sky, at, bg))
                    })();
                    let _ = tx.send(load);
                });
                world.sky_loading = Some((z.clone(), Mutex::new(rx)));
                status.message = Some((format!("Loading the sky of {z}"), MESSAGE_TIME));
            }
        }
    }
    // music: off, or a level's bank (decoded and mixed on a thread)
    if let (Some(_), Some((z, _))) = (&bank, &world.music_loading) {
        status.message = Some((format!("Still loading the music of {z}"), MESSAGE_TIME));
        if log {
            println!("test tools: music pick ignored: still loading the music of {z}");
        }
        bank = None;
    }
    if let Some(p) = bank {
        let zone = match p {
            Pick::None => None,
            Pick::Own => Some(current.0.clone()).filter(|m| m != "flat"),
            Pick::Level(i) => Some(lists.levels[i].zone.clone()),
        };
        world.music_loading = None;
        world.bank = zone.as_ref().and_then(|z| lists.banks.iter().position(|b| &lists.levels[b.0].zone == z));
        match zone {
            None => {
                for e in &music {
                    commands.entity(e).despawn();
                }
                world.music = "off".into();
                if log {
                    println!("test tools: music off");
                }
                status.message = Some(("Music off".into(), MESSAGE_TIME));
            }
            Some(z) => {
                let (tx, rx) = std::sync::mpsc::channel();
                let (name, schemas) = (z.clone(), game.0.schemas.clone());
                std::thread::spawn(move || {
                    let dir = data_dir();
                    let load = Game::load_music(&dir, &name).map(|tracks| {
                        // (the bank's Types: that level's sound bank; without it the names decide)
                        let bank = Game::sound_bank_of(&schemas, &dir, &name).unwrap_or_default();
                        super::music_mix(tracks, &bank)
                    });
                    let _ = tx.send(load);
                });
                world.music_loading = Some((z.clone(), Mutex::new(rx)));
                status.message = Some((format!("Loading the music of {z}"), MESSAGE_TIME));
            }
        }
    }
    // a switch: the map goes (end_play), the next loads (start_switch)
    if let Some(i) = switch {
        let to = lists.levels[i].clone();
        if log {
            println!("test tools: switching from {} to {} ({})", current.0, to.zone, to.title);
        }
        tools.panel = Panel::None;
        commands.insert_resource(Switch { to, from: current.0.clone(), rx: None, failed: vec![] });
        next.set(AppState::Switching);
    }
}

/// Puts a sky up in place of the one there (`layers`, the clear colour its `background`).
fn show_sky(commands: &mut Commands, layers: &SkyLayers, background: [f32; 3], skies: &Query<Entity, With<SkyLayer>>, clear: &mut ClearColor) {
    for e in skies {
        commands.entity(e).despawn();
    }
    bf_viewer::level_scene::spawn_sky(commands, layers);
    clear.0 = Color::srgb(background[0], background[1], background[2]);
}

/// A thread's result, if it's in: its own, or an error if the thread ended without one (it
/// panicked), so nothing waits on it for ever.
fn poll<T>(rx: &Mutex<Receiver<Result<T, String>>>) -> Option<Result<T, String>> {
    match rx.lock().ok()?.try_recv() {
        Ok(x) => Some(x),
        Err(std::sync::mpsc::TryRecvError::Empty) => None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Err("the loading thread stopped".into())),
    }
}

/// A sky or a music bank loaded: the sky made ready (kept for the session) and put up, the

/// camera seeing as far as SKY_FAR; the music in place of what played.
#[allow(clippy::too_many_arguments)]
fn finish_loads(mut commands: Commands, mut world: ResMut<World>, mut lists: ResMut<Lists>, mut status: ResMut<UsePanel>,
                mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<bf_viewer::level_scene::LevelMaterial>>,
                mut buffers: ResMut<Assets<bevy::render::storage::ShaderStorageBuffer>>, mut images: ResMut<Assets<Image>>,
                mut sources: ResMut<Assets<AudioSource>>, mut clear: ResMut<ClearColor>, skies: Query<Entity, With<SkyLayer>>,
                music: Query<Entity, With<super::LevelMusic>>, mut cam: Query<&mut Projection, With<MainCamera>>) {
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let got = world.sky_loading.as_ref().and_then(|(z, r)| poll(r).map(|x| (z.clone(), x)));
    if let Some((z, load)) = got {
        world.sky_loading = None;
        match load {
            Ok((mut g, sky, at, bg)) if !sky.is_empty() => {
                let layers = bf_viewer::level_scene::sky_layers(&mut g, &sky, at, &mut meshes, &mut materials, &mut buffers, &mut images);
                world.own_clear.get_or_insert(clear.0);
                show_sky(&mut commands, &layers, bg, &skies, &mut clear);
                if log {
                    println!("test tools: sky: {z} ({} layers at {at:.0}, background {bg:.2?})", layers.len());
                }
                lists.ready.insert(z.clone(), (layers, bg));
                world.sky = z.clone();
                status.message = Some((format!("Sky: {z}"), MESSAGE_TIME));
            }
            Ok(_) => status.message = Some((format!("{z} has no sky"), MESSAGE_TIME)),
            Err(e) => {
                eprintln!("test tools: no sky from {z}: {e}");
                status.message = Some((format!("No sky from {z}"), MESSAGE_TIME));
            }
        }
    }
    // (far enough for a sky's layers wherever one is up)
    if !skies.is_empty() {
        for mut p in &mut cam {
            if let Projection::Perspective(pp) = p.as_mut() {
                if pp.far < SKY_FAR {
                    pp.far = SKY_FAR;
                }
            }
        }
    }
    let got = world.music_loading.as_ref().and_then(|(z, r)| poll(r).map(|x| (z.clone(), x)));
    if let Some((z, load)) = got {
        world.music_loading = None;
        match load {
            Ok(tracks) => {
                for e in &music {
                    commands.entity(e).despawn();
                }
                let names: Vec<String> = tracks.iter().map(|t| format!("{} {}", t.2, t.0)).collect();
                if log {
                    println!("test tools: music of {z}: {}", names.join(", "));
                }
                if std::env::var("BF_MUTE").is_err() {
                    for (_, wav, label, k) in tracks {
                        super::spawn_music(&mut commands, &mut sources, wav, label, k);
                    }
                }
                world.music = format!("{z}: {}", names.join(", "));
                status.message = Some((format!("Music: {z}"), MESSAGE_TIME));
            }
            Err(e) => {
                eprintln!("test tools: no music from {z}: {e}");
                status.message = Some((format!("No music from {z}"), MESSAGE_TIME));
            }
        }
    }
}

/// BF_TOOLS_LOG: what each map starts with (AUDIT_AT in), to show nothing carried over
/// from the map before: the entities kept from before it and its own, the sounds playing, the
/// sky's layers, the squad and NPCs, the placed breakables' boxes and the map's breakables. And
/// a note from the switch (a level that didn't load) on the status line.
#[allow(clippy::too_many_arguments)]
fn audit(mut world: ResMut<World>, mut notice: ResMut<Notice>, mut status: ResMut<UsePanel>, session: Res<Session>, current: Res<CurrentMap>,
         before: Option<Res<Before>>, all: Query<Entity>, players: Query<(), With<AudioPlayer>>, music: Query<(), With<super::LevelMusic>>,
         skies: Query<(), With<SkyLayer>>, squad: Res<Squad>, scenery: Res<super::scenery::Scenery>, player: Res<Player>) {
    // (half a second in: the map's breakables and pickups are listed by then)
    if world.audited || player.loaded.is_none() || player.sim_time < AUDIT_AT {
        return;
    }
    world.audited = true;
    if let Some(n) = notice.0.take() {
        status.message = Some((n, MESSAGE_TIME * 2.0));
    }
    if std::env::var("BF_TOOLS_LOG").is_err() {
        return;
    }
    let kept = before.map_or(0, |b| b.0.len());
    let (boxes, breakables) = super::scenery::counts(&scenery);
    println!("test tools: map {} ({}): {} entities ({kept} from before it), {} sounds ({} music), {} sky layers, squad {} ({} NPCs), {boxes} placed boxes, {breakables} breakables",
             session.maps, current.0, all.iter().count(), players.iter().count(), music.iter().count(), skies.iter().count(),
             squad.0.len(), squad.0.iter().filter(|m| m.npc.is_some()).count());
}

/// The open world menu's text (see `WorldText`).
fn world_text(tools: Res<Tools>, world: Res<World>, lists: Res<Lists>, current: Res<CurrentMap>, mut text: ResMut<WorldText>) {
    let mut s = String::new();
    let title = |i: usize| -> String {
        let e = &lists.levels[i];
        if e.zone == "flat" { e.title.clone() } else { format!("{}  {}", e.zone, e.title) }
    };
    // a window of LIST_LINES round the pick
    let window = |pick: usize, n: usize| {
        let first = pick.saturating_sub(LIST_LINES / 2).min(n.saturating_sub(LIST_LINES));
        first..(first + LIST_LINES).min(n)
    };
    let mark = |on: bool| if on { ">" } else { " " };
    match tools.panel {
        Panel::Sky => {
            s += "SKY   (Y / Esc: close)\n";
            s += &format!("now: {}{}\n", world.sky, world.sky_loading.as_ref().map_or(String::new(), |l| format!("   (loading {})", l.0)));
            let n = lists.skies.len() + 2;
            for k in window(world.sky_pick, n) {
                let label = match k {
                    0 => "none".to_string(),
                    1 => format!("this map's own ({})", current.0),
                    k => title(lists.skies[k - 2]),
                };
                s += &format!("{} {label}\n", mark(k == world.sky_pick));
            }
            if !lists.scanned {
                s += "  (listing the levels' skies...)\n";
            }
            s += "Up / Down pick   click / Enter: show it";
        }
        Panel::Music => {
            s += "MUSIC   (U / Esc: close)\n";
            s += &format!("now: {}{}\n", world.music, world.music_loading.as_ref().map_or(String::new(), |l| format!("   (loading {})", l.0)));
            let n = lists.banks.len() + 2;
            for k in window(world.music_pick, n) {
                let label = match k {
                    0 => "off".to_string(),
                    1 => format!("this map's own ({})", current.0),
                    k => title(lists.banks[k - 2].0),
                };
                s += &format!("{} {label}\n", mark(k == world.music_pick));
            }
            // (the picked bank's tracks on a line of their own: in the list they'd wrap)
            if let Some((_, tracks)) = world.music_pick.checked_sub(2).and_then(|k| lists.banks.get(k)) {
                s += &format!("  tracks: {}\n", tracks.join(", "));
            }
            if !lists.scanned {
                s += "  (listing the music banks...)\n";
            }
            s += "Up / Down pick   click / Enter: play   Left / Right: previous / next";
        }
        Panel::Levels => {
            s += "SWITCH LEVEL   (L / Esc: close)\n";
            s += &format!("now: {}\n", current.0);
            let mut group = "";
            for i in window(world.level_pick, lists.levels.len()) {
                let e = &lists.levels[i];
                if e.group != group {
                    group = e.group;
                    s += &format!("  [{group}]\n");
                }
                s += &format!("{} {}\n", mark(i == world.level_pick), title(i));
            }
            s += "Up / Down pick   click / Enter: load it (the tools stay)";
        }
        _ => {}
    }
    if text.0 != s {
        text.0 = s;
    }
}

/// Loads a level for the switch (on a thread), as the front end does, with the test map's
/// weapon data on top (testmap::load_weapon_data). One that doesn't load (no level file): the
/// test map instead, and a note saying so.
///
/// Test hook: BF_TEST_LEVEL_FAIL=<level> makes loading that level panic on its thread (for the
/// fall back in `finish_switch`).
fn load_level(to: &str, deathmatch: bool) -> LevelLoad {
    if std::env::var("BF_TEST_LEVEL_FAIL").is_ok_and(|z| z == to) {
        panic!("BF_TEST_LEVEL_FAIL: {to}");
    }
    let mut game = super::load_game()?;
    let level = super::load_map(&mut game, to);
    if to != "flat" && level.is_none() {
        let mut game = super::load_game()?;
        let level = super::load_map(&mut game, "flat");
        super::testmap::load_weapon_data(&mut game);
        return Ok((LoadedMap { game, level, deathmatch: false, map: "flat".into() }, Some(format!("{to} didn't load: the test map instead"))));
    }
    super::testmap::load_weapon_data(&mut game);
    Ok((LoadedMap { game, level, deathmatch, map: to.into() }, None))
}

/// The switch begins (the map is gone: `end_play`): what's left is logged (BF_TOOLS_LOG: only
/// what was there before the map, nothing of its own; screenshots on their way and their
/// observers aside, which go once saved), a loading screen goes up and the next level loads on
/// a thread.
#[allow(clippy::type_complexity)]
fn start_switch(mut commands: Commands, mut switch: ResMut<Switch>, before: Option<Res<Before>>,
                all: Query<(Entity, Has<bevy::render::view::screenshot::Screenshot>, Has<bevy::ecs::observer::Observer>)>,
                players: Query<(), With<AudioPlayer>>) {
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        let kept = before.as_ref().map_or(0, |b| b.0.len());
        let new = |e: &Entity| !before.as_ref().is_some_and(|b| b.0.contains(e));
        let left = all.iter().filter(|(e, shot, obs)| new(e) && !shot && !obs).count();
        let shots = all.iter().filter(|(e, shot, obs)| new(e) && (*shot || *obs)).count();
        println!("test tools: left {}: {} entities remain ({kept} from before it, {left} of its own, {shots} screenshots on their way), {} sounds",
                 switch.from, all.iter().count(), players.iter().count());
    }

    commands.spawn((Camera2d, SwitchPart));
    commands.spawn((
        SwitchPart,
        Text::new(format!("Loading {} ({})...", switch.to.title, switch.to.zone)),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Node { position_type: PositionType::Absolute, left: Val::Percent(40.0), top: Val::Percent(48.0), ..default() },
    ));
    switch.rx = Some(load_thread(&switch.to));
}

/// Loads `to` on a thread (`load_level`).
fn load_thread(to: &Entry) -> Mutex<Receiver<LevelLoad>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let (zone, deathmatch) = (to.zone.clone(), to.deathmatch);
    std::thread::spawn(move || {
        let _ = tx.send(load_level(&zone, deathmatch));
    });
    Mutex::new(rx)
}

/// The next level has loaded: play it (`begin_play`, as the front end does). One that failed (an
/// error, or its thread stopped without a result: a panic) is noted and the map it came from is
/// loaded instead, else the test map; only if that fails too does the program stop.
fn finish_switch(mut commands: Commands, mut switch: ResMut<Switch>, lists: Res<Lists>, mut next: ResMut<NextState<AppState>>,
                 mut notice: ResMut<Notice>, mut exit: EventWriter<AppExit>, mut text: Query<&mut Text, With<SwitchPart>>) {
    let Some(got) = switch.rx.as_ref().and_then(poll) else { return };
    match got {
        Ok((map, note)) => {
            if std::env::var("BF_TOOLS_LOG").is_ok() {
                println!("test tools: {} loaded{}", map.map, note.as_ref().map_or(String::new(), |n| format!(" ({n})")));
            }
            let failed = (!switch.failed.is_empty()).then(|| format!("{}: {} instead", switch.failed.join("; "), map.map));
            notice.0 = failed.or(note);
            commands.remove_resource::<Switch>();
            begin_play(&mut commands, map);
            next.set(AppState::Playing);
        }
        Err(e) => {
            eprintln!("test tools: couldn't load {}: {e}", switch.to.zone);
            let failed = switch.to.zone.clone();
            switch.failed.push(format!("{failed} didn't load ({e})"));
            // the map it came from, then the test map, each tried once
            let tried = |z: &str| switch.failed.iter().any(|f| f.starts_with(&format!("{z} ")));
            let back = [switch.from.clone(), "flat".to_string()].into_iter().find(|z| !tried(z));
            let Some(entry) = back.and_then(|z| lists.levels.iter().find(|e| e.zone == z).cloned()) else {
                exit.write(AppExit::error());
                return;
            };
            if std::env::var("BF_TOOLS_LOG").is_ok() {
                println!("test tools: {failed} didn't load: back to {}", entry.zone);
            }
            for mut t in &mut text {
                t.0 = format!("{failed} didn't load. Loading {} ({})...", entry.title, entry.zone);
            }
            switch.rx = Some(load_thread(&entry));
            switch.to = entry;
        }
    }
}


/// The loading screen goes.
fn end_switch(mut commands: Commands, parts: Query<Entity, With<SwitchPart>>) {
    for e in &parts {
        commands.entity(e).despawn();
    }
}
