//! Brute Force playable demo: run around as one of the squad, driven by the game's own
//! locomotion animations, root motion and sounds (read straight from Brute Force/data/*.tgz).
//!
//!   cargo run --bin bf_play
//!   cargo run --bin bf_play -- --test    the test map (every weapon and pickup, instant kill,
//!                                        the controls panel; see play_testmap.rs)
//!
//! Keyboard / mouse (no gamepad for now)
//!   WASD        move (camera-relative)
//!   Shift       sprint
//!   Ctrl        walk
//!   Space       jump
//!   C           dodge roll / sidestep
//!   Z           crouch / stand (kneel standing still, crouch walk on the move)
//!   Right mouse aim: upper body turns to the camera direction, legs keep running;
//!               backpedal when moving away
//!   Mouse       look (click to capture, Esc to release)
//!   Wheel       zoom
//!   Left mouse  fire (hold)
//!   Q           switch weapon
//!   R           reload
//!   G           use the item in the item box (grenade: hold to charge, release to throw;
//!               Roller / Sentry: set down at the press)
//!   T           next grenade type carried
//!   Tab        next item; hold: the item list (wheel picks)
//!   E (hold)    use (a gate's wall panel)
//!   M           next ground surface (footstep / landing sounds)
//!   Backspace   back to the map menu (in the same window)
//!   1-4         Brutus / Flint / Hawk / Tex
//!
//! Movement is root motion: each frame the character moves by exactly what the playing clip's
//! root channel moves. Clips are looked up by their real names (animation names are hashes of
//! the motion-sequence script names, e.g. "Sc_w1_run"); jumps follow the game's MOTION chain
//! crouch -> launch -> fall (loop) -> land. Sounds: footsteps are triggered when a foot plants
//! (from the animated pose) and use the character's footstep type on the current surface; the
//! jump grunt comes from the character's movement-sounds; landing / dodge use the surface's
//! jump-land / slide sets.
//!
//! Weapons: each character carries their two starting weapons from the game's inventory
//! (objecttypes), modelled from the level's weapon archetypes. The held one is joined to the
//! hand by hardpoints (see bf::weapon), the other is stowed on its slot's back / hip hardpoint. Firing uses the weapon's rate and fire sounds; the spine twists so the muzzle (not
//! the chest) lines up with the crosshair, and shots fly from the muzzle to what the crosshair
//! is on (ground or pillars).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::{
    audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume},
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    math::Affine2,
    prelude::*,
    pbr::NotShadowCaster,
    render::mesh::skinning::SkinnedMeshInverseBindposes,
    window::{CursorGrabMode, PrimaryWindow},
};
use bf_viewer::bf::character::{Character, Game, LiquidType, LIQUID_BIG_SPLASH, LIQUID_SHOT, LIQUID_SPLASH, LIQUID_WADE};
use bf_viewer::bf::locomotion::{self, Locomotion};
use bf_viewer::bf::weapon::{self, Hardpoint, WeaponDef, WeaponModel};
use bf_viewer::scene::{spawn_model, spawn_static, ModelAssets};
use bf_viewer::bf::level::Level;

const CHARACTERS: [&str; 4] = ["brutus", "flint", "hawk", "tex"];
const FADE: f32 = 0.2;
const TURN_RATE: f32 = 9.0;
/// yaw (radians) the spine takes when aiming standing; the feet turn for the rest
const AIM_SPINE_YAW: f32 = 0.2;
/// the gun aims at least this far out along the camera heading (looking straight down, the
/// crosshair point would otherwise sit under the gun and swing its yaw around)
const MIN_AIM_REACH: f32 = 1.5;
/// The character's pelvis (skeleton origin) sits about this high above its feet.
const GROUND: f32 = -1.05;
/// The game's gravity and jump take-off speed (default.xbe: FUN_0012cd10 subtracts 18 m/s^2
/// from the vertical speed while falling; FUN_0012b630 launches at 5.8 m/s): apex 0.93 m.
const GRAVITY: f32 = 18.0;
const JUMP_SPEED: f32 = 5.8;
/// In the air the horizontal speed keeps 0.99 of itself per game frame (taken as 30 Hz).
const AIR_DRAG: f32 = 0.99;
/// A foot counts as planted below its resting height plus this.
const FOOT_CONTACT: f32 = 0.035;

fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("BF_DATA_DIR") {
        return d.into();
    }
    let here = std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_else(|_| ".".into());
    for cand in [here.join("../Brute Force/data"), PathBuf::from("Brute Force/data"), PathBuf::from("../Brute Force/data")] {
        if cand.join("common.tgz").exists() {
            return cand;
        }
    }
    here.join("../Brute Force/data")
}

/// Where bf_play is: the front end (its menus, and the loading screen while a map loads on a
/// thread) or a map. One window throughout: Backspace in a map tears it down and goes back to
/// the menu.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    /// starting up: the loading screen while the game data is read, then the intro
    #[default]
    Boot,
    Intro,
    Menu,
    Loading,
    Playing,
}

/// The game mode being played: deathmatch is alone, without the squad's radar.
#[derive(Resource, Clone, Copy, Default)]
pub struct Mode {
    pub deathmatch: bool,
}

/// The game data for a map: the shared archives, the default level audio (surfaces and their
/// footstep / landing sounds come from the first mission; the multiplayer bank has everyone's
/// footsteps) and the squad's voice lines.
pub fn load_game() -> Result<Game, String> {
    let mut game = Game::load(&data_dir())?;
    let level = std::env::var("BF_LEVEL").unwrap_or_else(|_| "e01".into());
    if let Err(e) = game.load_level(&data_dir(), &level) {
        eprintln!("no level audio from {level}: {e}");
    }
    if level != "mp1" {
        if let Err(e) = game.load_extra_sounds(&data_dir(), "mp1") {
            eprintln!("no extra sounds from mp1: {e}");
        }
    }
    if let Err(e) = game.load_voices(&data_dir(), "en") {
        eprintln!("no voice lines: {e}");
    }
    Ok(game)
}

/// Load a map (its level, streamed sounds and collision, made the current arena); "flat" or a
/// map that won't load: the flat test floor (None).
pub fn load_map(game: &mut Game, map: &str) -> Option<Level> {
    world::Arena::uninstall();
    if map == "flat" {
        return None;
    }
    match game.load_level(&data_dir(), map).and_then(|_| Level::load(game)) {
        Ok(mut l) => {
            // door and gate sounds stream from the map's language wave bank
            if let Err(e) = game.load_level_streams(&data_dir(), map, "en") {
                eprintln!("no streamed sounds for {map}: {e}");
            }
            let a = world::Arena::install(game, &l);
            a.settle_pickups(game, &mut l);
            // test hook: BF_FIND_STEEP=1 lists spots on slide surfaces steeper than 50 degrees
            if std::env::var("BF_FIND_STEEP").is_ok() {
                for z in (-100..100).step_by(6) {
                    for x in (-100..100).step_by(6) {
                        let (x, z) = (x as f32, z as f32);
                        if let Some((y, n, m)) = a.floor_at(x, z, 200.0) {
                            let deg = n.y.clamp(-1.0, 1.0).acos().to_degrees();
                            if deg > 50.0 && world::Arena::slides(m) {
                                println!("steep {x},{z} y {y:.1} {deg:.0} deg, slide surface, down {:.2},{:.2}", n.x, n.z);
                            }
                            // a ledge: flat here, much lower 2 m away
                            if deg < 20.0 {
                                for (dx, dz) in [(2.0, 0.0), (-2.0, 0.0), (0.0, 2.0), (0.0, -2.0)] {
                                    if let Some((y2, _, _)) = a.floor_at(x + dx, z + dz, y + 0.5) {
                                        if y - y2 > 6.0 {
                                            println!("ledge {x},{z} y {y:.1} drops {:.1} m toward {},{}", y - y2, x + dx, z + dz);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                std::process::exit(0);
            }
            let (n, terrain, slide, doors) = a.stats();
            println!("map {map}: collision {n} triangles ({terrain} terrain, {slide} slide-flagged, {doors} door leaves; {} blockers), {} start points",
                     l.blockers.len(), a.starts.len());
            // BF_LIGHT_LOG: each start point's nearest point lights
            if std::env::var("BF_LIGHT_LOG").is_ok() {
                for (i, (at, _)) in a.starts.iter().enumerate() {
                    let mut near: Vec<(f32, f32)> = l.lamps.iter().map(|p| (p.at.distance(*at), p.range)).collect();
                    near.sort_by(|x, y| x.0.total_cmp(&y.0));
                    let list: Vec<String> = near.iter().take(3).map(|(d, r)| format!("{d:.1} m (range {r})")).collect();
                    println!("start {i} at {:.1} {:.1} {:.1}: nearest point lights {}", at.x, at.y, at.z, list.join(", "));
                }
            }
            Some(l)
        }
        Err(e) => {
            eprintln!("no map {map}: {e}; using the flat test floor");
            None
        }
    }
}

/// A loaded map, ready to play: the data, its level, and the mode.
pub struct LoadedMap {
    pub game: Game,
    pub level: Option<Level>,
    pub deathmatch: bool,
    pub map: String,
}

/// The map being played (Backspace returns to it in the menu).
#[derive(Resource)]
struct CurrentMap(String);

/// Put a loaded map in place to play (its resources; `setup` and the plugins spawn it as
/// AppState::Playing begins).
pub fn begin_play(commands: &mut Commands, map: LoadedMap) {
    let LoadedMap { game, level, deathmatch, map } = map;
    commands.insert_resource(CurrentMap(map));
    let start = std::env::var("BF_CHARACTER").ok().and_then(|s| s.parse().ok()).unwrap_or(3usize) % CHARACTERS.len();
    commands.insert_resource(player_at_start(start));
    commands.insert_resource(squad_at_start(start, deathmatch));
    commands.insert_resource(GameData(game));
    commands.insert_resource(MapLevel(level));
    commands.insert_resource(Mode { deathmatch });
    commands.insert_resource(SoundCache::default());
    commands.insert_resource(TestDied::default());
}

/// Whether BF_TEST_DIE has killed the player yet on this level (once per level: the member who
/// takes over lives on).
#[derive(Resource, Default)]
struct TestDied(bool);

/// The player at the map's start point (BF_START=<n> picks another), with the test hooks'
/// starting camera and place.
fn player_at_start(start: usize) -> Player {
    // test hooks: starting camera (yaw in radians, 0 = behind the character)
    let mut p = Player::new(start);
    let env = |k: &str| std::env::var(k).ok().and_then(|s| s.parse::<f32>().ok());
    if let Some(v) = env("BF_CAMERA_YAW") { p.cam_yaw = v; }
    if let Some(v) = env("BF_CAMERA_PITCH") { p.cam_pitch = v; }
    if let Some(v) = env("BF_CAMERA_DISTANCE") { p.cam_distance = v; }
    // on a map: at its first start point (BF_START=<n> picks another), facing its way
    if let Some(a) = world::arena() {
        let n = std::env::var("BF_START").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        if let Some(&(at, yaw)) = a.starts.get(n % a.starts.len().max(1)) {
            p.position = Vec3::new(at.x, at.y + 1.0 - GROUND, at.z);
            p.yaw = yaw;
            p.cam_yaw = yaw;
        }
    }
    if let Some(g) = test_goto() {
        // a 5th value starts at that height (on the floor below it), e.g. on a roof
        let ground = match g.get(4) {
            Some(&y) => world::arena().and_then(|a| a.floor_below(g[0], g[1], y)).map_or(y, |f| f.0),
            None => world::arena().and_then(|a| a.ground(g[0], g[1])).unwrap_or(GROUND),
        };
        p.position = Vec3::new(g[0], ground + 1.0 - GROUND, g[1]);
        p.yaw = (-(g[2] - g[0])).atan2(-(g[3] - g[1]));
        p.cam_yaw = p.yaw;
    }
    p.position.y = floor_y(p.position.x, p.position.z, p.position.y + GROUND) - GROUND;
    p.prev_xz = Vec2::new(p.position.x, p.position.z);
    p
}

/// The rest of the squad, in formation behind the start character (none in deathmatch).
fn squad_at_start(start: usize, deathmatch: bool) -> Squad {
    // the rest of the squad, in formation behind the start character
    let origin = world::arena().and_then(|a| a.starts.get(std::env::var("BF_START").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0) % a.starts.len().max(1)).copied());
    let members = (0..CHARACTERS.len()).filter(|&c| c != start).enumerate().map(|(slot, c)| {
        let mut m = Player::new(c);
        let f = FORMATION[slot % FORMATION.len()];
        let top = origin.map_or(GROUND, |(at, _)| at.y + 1.0);
        m.position = match origin {
            Some((at, yaw)) => Vec3::new(at.x, 0.0, at.z) + Quat::from_rotation_y(yaw) * Vec3::new(f.x * 0.5, 0.0, f.y * 0.5),
            None => Vec3::new(f.x, 0.0, f.y),
        };
        if let Some(a) = world::arena() {
            let feet = floor_y(m.position.x, m.position.z, top);
            let q = a.push_out(Vec2::new(m.position.x, m.position.z), BODY_RADIUS, feet, feet + BODY_HEIGHT, STEP_UP);
            m.position = Vec3::new(q.x, 0.0, q.y);
        }
        // test hook: BF_TEST_TARGET=1 stands the first squadmate 8 m in front of a
        // BF_TEST_GOTO start (in the line of fire, for BF_TEST_FIRE)
        let mut top = top;
        if let (0, true, Some(g)) = (slot, std::env::var("BF_TEST_TARGET").is_ok(), test_goto()) {
            m.position = Vec3::new(g[0], 0.0, g[1] - 8.0);
            top = 200.0;
        }
        m.position.y = floor_y(m.position.x, m.position.z, top) - GROUND;
        m.prev_xz = Vec2::new(m.position.x, m.position.z);
        m.surface = usize::MAX;
        m
    }).collect();
    Squad(if deathmatch { vec![] } else { members })
}

/// The entities there were when a map began: everything else is the map's, removed when it
/// ends.
#[derive(Resource, Default)]
struct Before(std::collections::HashSet<Entity>);

fn snapshot_entities(mut commands: Commands, all: Query<Entity>) {
    commands.insert_resource(Before(all.iter().collect()));
}

/// Leaving a map: remove everything it spawned (its roots; children go with them), and its
/// collision.
fn end_play(mut commands: Commands, before: Option<Res<Before>>, all: Query<(Entity, Option<&ChildOf>), Without<Window>>,
            mut cursor: Query<&mut Window, With<PrimaryWindow>>) {
    let before = before.map(|b| b.0.clone()).unwrap_or_default();
    for (e, parent) in &all {
        let new = |e: Entity| !before.contains(&e);
        if new(e) && parent.is_none_or(|p| !new(p.parent())) {
            commands.entity(e).despawn();
        }
    }
    world::Arena::uninstall();
    commands.insert_resource(ClearColor(Color::BLACK));
    if let Ok(mut w) = cursor.single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::None;
        w.cursor_options.visible = true;
    }
}

fn main() {
    // the menu first: the window opens at once and the game data is read behind its loading
    // screen (AppState::Boot); a map straight away (BF_MAP, test hooks): read here first
    let test_map = testmap::requested();
    let menu_first = !test_map && menu::wanted();
    let map = if test_map { "flat".into() } else { std::env::var("BF_MAP").unwrap_or_else(|_| "sdm_e34".into()) };
    let mut loaded = None;
    if !menu_first {
        let mut game = load_game().unwrap_or_else(|e| {
            eprintln!("failed to load game data from {}: {e}", data_dir().display());
            std::process::exit(1)
        });
        let level = load_map(&mut game, &map);
        if test_map {
            testmap::load_weapon_data(&mut game);
        }
        let start = std::env::var("BF_CHARACTER").ok().and_then(|s| s.parse().ok()).unwrap_or(3usize) % CHARACTERS.len();
        // BF_DUMP_MUSIC=<file.wav>  write the map's music track and exit
        if let Ok(out) = std::env::var("BF_DUMP_MUSIC") {
            let tracks = Game::load_music(&data_dir(), &map).unwrap_or_default();
            if let Some((name, wav)) = tracks.first() {
                std::fs::write(&out, wav).ok();
                println!("{name} -> {out}");
            }
            std::process::exit(0);
        }
        // BF_DUMP_SOUND_IDS=<dir>:<hex id>,<hex id>...  decode sound ids (or `all`) to WAV files and exit
        if let Some((dir, ids)) = std::env::var("BF_DUMP_SOUND_IDS").ok().as_deref().and_then(|v| v.rsplit_once(':')) {
            std::fs::create_dir_all(dir).ok();
            let ids: Vec<u32> = if ids == "all" { game.sounds.ids() } else {
                ids.split(',').filter_map(|x| u32::from_str_radix(x.trim().trim_start_matches("h_"), 16).ok()).collect()
            };
            for id in ids {
                match game.sounds.pcm(id) {
                    Some((rate, pcm)) => {
                        let path = format!("{dir}/{id:08x}.wav");
                        std::fs::write(&path, bf_viewer::bf::audio::wav_bytes(rate, &pcm)).ok();
                        println!("{id:08x} {rate} Hz {:.2} s", pcm.len() as f32 / rate as f32);
                    }
                    None => println!("{id:08x} not found"),
                }
            }
            return;
        }
        // BF_DUMP_TEXTURE=<path>:<hex id>  the decoded RGBA of a texture, as the viewer uploads it
        // (raw: u32 width, u32 height, then rows), and exit
        if let Some((path, id)) = std::env::var("BF_DUMP_TEXTURE").ok().as_deref().and_then(|v| v.rsplit_once(':')) {
            if let Some((w, h, px)) = u32::from_str_radix(id, 16).ok().and_then(|id| game.texture_rgba(id)) {
                std::fs::write(path, [w.to_le_bytes(), h.to_le_bytes()].concat().into_iter().chain(px).collect::<Vec<u8>>()).ok();
            }
            return;
        }
        // BF_ANIM_PROBE=<name>,<name>...  each character's clips by script name: duration and events
        if let Ok(names) = std::env::var("BF_ANIM_PROBE") {
            for (name, _) in game.characters.clone() {
                let Ok(model) = Character::load(&game, &name, 0) else { continue };
                for n in names.split(',') {
                    match model.anim_by_name(n) {
                        Some(c) => {
                            let a = &model.anims[c];
                            let ev: Vec<String> = game.events(a.event_channel).iter().map(|(t, e)| format!("{t:.2}:{e:08x}")).collect();
                            let root = locomotion::root_delta(&model, &game, c, 0.0, a.duration - 1e-3);
                            println!("{name:7} {n:24} clip {c:3} {:.2}s bones {:3} root {:.2} events [{}]", a.duration, a.targets.len(), root, ev.join(" "));
                        }
                        None => println!("{name:7} {n:24} -"),
                    }
                }
            }
            return;
        }
        // BF_DUMP_HITPOINTS=1: each character's hitpoints, and exit
        if std::env::var("BF_DUMP_HITPOINTS").is_ok() {
            let mut hp: Vec<_> = game.character_hitpoints.iter().collect();
            hp.sort_by(|a, b| a.0.cmp(b.0));
            for (name, hp) in hp {
                println!("{name:12} {hp}");
            }
            return;
        }
        if std::env::var("BF_DUMP_WEAPONS").is_ok() {
            dump_weapons(&game);
            return;
        }
        if let Ok(dir) = std::env::var("BF_DUMP_SOUNDS") {
            dump_sounds(&mut game, start, &dir);
            return;
        }
        let deathmatch = std::env::var("BF_NO_SQUAD").is_ok() || std::env::var("BF_DEATHMATCH").is_ok();
        loaded = Some(LoadedMap { game, level, deathmatch, map: map.clone() });
    }
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Brute Force".into(), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight { brightness: 220.0, ..default() })
        .init_state::<AppState>();
    bf_viewer::level_scene::plugin(&mut app);
    // the front end; its pictures and sounds come from the game data
    menu::plugin(&mut app);
    if let Some(mut l) = loaded {
        let data = menu::gather(&mut l.game, &data_dir());
        menu::install(app.world_mut(), data);
        // straight into the map (BF_MAP or a test hook)
        app.insert_state(AppState::Playing);
        if test_map {
            app.insert_resource(testmap::TestMap { instant_kill: true });
        }
        let mut commands = app.world_mut().commands();
        begin_play(&mut commands, l);
        app.world_mut().flush();
    }
    let playing = in_state(AppState::Playing);
    app.add_plugins((hud::plugin, grenade::plugin, fx::plugin, pickups::plugin, text::plugin, deathcam::plugin, testmap::plugin, bf_viewer::ale_fx::plugin))
        .init_resource::<UsePanel>()
        .add_systems(OnEnter(AppState::Playing), (snapshot_entities, setup).chain())
        .add_systems(OnExit(AppState::Playing), end_play)
        .add_systems(Update, (spawn_player, read_input, squad_control, doors, update_player, play_sounds, follow_camera, update_weapons,
                               move_tracers, projectiles, update_hud, slide_dust, capture).chain().run_if(playing.clone()))
        .add_systems(Update, lamp_characters.after(update_player).run_if(playing.clone()))
        .add_systems(Update, spawn_splashes.after(update_player).run_if(playing.clone()))
        .add_systems(Update, (back_to_menu, audio_probe).run_if(playing))
        .add_systems(PostUpdate, bf_viewer::level_scene::follow_sky.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}

#[path = "play_hud.rs"]
mod hud;
#[path = "play_grenade.rs"]
mod grenade;
#[path = "play_fx.rs"]
mod fx;
#[path = "play_pickups.rs"]
mod pickups;
#[path = "play_text.rs"]
mod text;
#[path = "play_deathcam.rs"]
mod deathcam;
#[path = "play_testmap.rs"]
mod testmap;
#[path = "play_menu.rs"]
mod menu;
use bf_viewer::arena as world;

/// The map's level (drawn at startup), if playing on one.
#[derive(Resource)]
struct MapLevel(Option<Level>);

/// The map's doors.
#[derive(Resource)]
struct Doors(Vec<DoorState>);

struct DoorState {
    door: bf_viewer::level_scene::Door,
    /// how far into its opening animation it is (s), and whether it's opening
    t: f32,
    opening: bool,
    /// the wall panels that open it (world buttons whose signal reaches its anim-triggers):
    /// each one's button and the way it faces (world); such a door opens only from them
    panels: Vec<(Vec3, Vec3)>,
    /// opened from a panel: its anim-triggers act once (h_f136d22d 1), so it stays open
    latched: bool,
}

/// The use key (E) and what it would act on now (see `doors`), and the status message shown at
/// the character (text, seconds left).
#[derive(Resource, Default)]
pub struct UsePanel {
    /// the prompt's object ("panel": the game's "Hold [X] to activate %s."), while one's usable
    pub prompt: Option<&'static str>,
    /// where the blue target ring goes (the panel's button, world)
    pub target: Option<Vec3>,
    /// how long use has been held on it (s)
    pub held: f32,
    pub message: Option<(String, f32)>,
}

/// Doors open for anyone within DOOR_RANGE m (their animation: leaves slide along their joint's
/// axis) and close again (the animation backwards); open doors stop blocking. The level's sound
/// triggers beside them play as they start opening or closing, heard up to DOOR_HEARD m.
/// Gates with wall panels (sdm_e34's big gate) open only when a panel is used: the player's feet
/// within USE_REACH m of its button, looking within USE_ANGLE of it (the panels' reticule-action
/// 1: the reticle on them), holding use for HOLD_TIME s (the game's prompt says "Hold"). The
/// capture shows no progress bar and no gate message. Reach, angle and hold time are guesses.
const DOOR_RANGE: f32 = 5.0;
const USE_REACH: f32 = 2.5;
const USE_ANGLE: f32 = 0.8;
const HOLD_TIME: f32 = 0.5;
/// The gate panels' button: hardpoint h_10f81d34 of their archetype (the green glow quad).
const PANEL_BUTTON: u32 = 0x10F8_1D34;
/// Music volume (linear) under the game's sounds.
const MUSIC_VOLUME: f32 = 0.5;
/// A level's ambience bed, under its music (of the music's volume).
const AMBIENCE_VOLUME: f32 = 0.8;
const DOOR_HEARD: f32 = 15.0;

fn doors(time: Res<Time>, mut player: ResMut<Player>, squad: Res<Squad>, doors: Option<ResMut<Doors>>,
         mut use_panel: ResMut<UsePanel>, mut transforms: Query<&mut Transform>) {
    let Some(mut doors) = doors else { return };
    let dt = frame_dt(&time);
    // a panel's button in reach and in view (of a gate that isn't open yet)
    let look = Vec2::new(-player.cam_yaw.sin(), -player.cam_yaw.cos());
    let feet = player.position + Vec3::Y * GROUND;
    // (from in front of it: the button faces out of the wall)
    let usable = |&(p, facing): &(Vec3, Vec3)| {
        let to = Vec2::new(p.x - feet.x, p.z - feet.z);
        !player.dead && to.length() < USE_REACH && (-0.5..2.5).contains(&(p.y - feet.y)) && to.normalize_or_zero().dot(look) > USE_ANGLE.cos()
            && (feet - p).dot(facing) > 0.0
    };
    let target = doors.0.iter().enumerate().filter(|(_, d)| !d.latched)
        .find_map(|(i, d)| d.panels.iter().find(|p| usable(p)).map(|p| (i, p.0)));
    use_panel.prompt = target.map(|_| "panel");
    use_panel.target = target.map(|t| t.1);
    if let Some((_, left)) = &mut use_panel.message {
        *left -= dt;
        if *left <= 0.0 {
            use_panel.message = None;
        }
    }
    use_panel.held = if target.is_some() && player.use_held { use_panel.held + dt } else { 0.0 };
    if let (Some((i, _)), true) = (target, use_panel.held >= HOLD_TIME) {
        doors.0[i].latched = true;
        use_panel.held = 0.0;
    }
    let mut sounds = vec![];
    for (i, d) in doors.0.iter_mut().enumerate() {
        let DoorState { door, t, opening, .. } = d;
        let near = if d.panels.is_empty() {
            std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.dead)
                .any(|u| Vec2::new(u.position.x - door.centre.x, u.position.z - door.centre.z).length() < DOOR_RANGE)
        } else {
            d.latched
        };
        if near != *opening {
            *opening = near;
            if let Some(id) = if near { door.open_sound } else { door.close_sound } {
                let k = 1.0 - door.centre.distance(player.position) / DOOR_HEARD;
                if k > 0.0 {
                    sounds.push((id, k));
                }
            }
        }
        let was = *t;
        *t = if near { (*t + dt).min(door.duration) } else { (*t - dt).max(0.0) };
        if *t != was {
            for leaf in &door.leaves {
                if let Ok(mut tr) = transforms.get_mut(leaf.entity) {
                    tr.translation = leaf.closed.translation + leaf.axis * leaf.slide(*t, door.duration);
                }
            }
            // BF_DOOR_LOG: a panel gate's leaves, each second of its opening
            if !d.panels.is_empty() && (was == 0.0 || *t == door.duration || t.floor() != was.floor()) && std::env::var("BF_DOOR_LOG").is_ok() {
                println!("gate h_{:08x} t {:.2}/{:.2} leaves slid {:?} m", door.name, *t, door.duration,
                         door.leaves.iter().map(|l| (l.slide(*t, door.duration) * 100.0).round() / 100.0).collect::<Vec<_>>());
            }
        }
        if let Some(a) = world::arena() {
            a.set_door_open(i, *t > 0.6 * door.duration);
        }
    }
    player.sound_queue.extend(sounds);
}

/// The level's point lights on the characters. The scenery takes them per surface (its
/// `LevelMaterial`); a character's materials are bevy's own, so each takes the light at its chest
/// from every direction, as its texture added to the picture (emissive), at CHARACTER_LAMP of it
/// (about the average N.L over a body).
fn lamp_characters(map: Res<MapLevel>, player: Res<Player>, squad: Res<Squad>, children: Query<&Children>,
                   parts: Query<&MeshMaterial3d<StandardMaterial>>, mut materials: ResMut<Assets<StandardMaterial>>,
                   mut last: Local<HashMap<Entity, Vec3>>) {
    let Some(level) = &map.0 else { return };
    if level.lamps.is_empty() {
        return;
    }
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let Some(l) = &u.loaded else { continue };
        let light = bf_viewer::level_scene::lamp_light_at(level, u.position + Vec3::Y * CHEST) * CHARACTER_LAMP;
        // (only when it changes: a changed material is sent to the GPU again)
        if last.get(&l.root).is_some_and(|v| v.distance(light) < 0.005) {
            continue;
        }
        last.insert(l.root, light);
        for e in children.iter_descendants(l.root) {
            let Ok(m) = parts.get(e) else { continue };
            let Some(mat) = materials.get_mut(&m.0) else { continue };
            if mat.unlit {
                continue;
            }
            mat.emissive = LinearRgba::rgb(light.x, light.y, light.z);
            if mat.emissive_texture.is_none() {
                mat.emissive_texture = mat.base_color_texture.clone();
            }
        }
    }
}

/// See `lamp_characters`: the share of the lamps' light a body takes, and where it's measured.
const CHARACTER_LAMP: f32 = 0.5;
const CHEST: f32 = 1.2;

/// The ALE effects' clock (fixed steps while capturing), and each character's slide dust: a
/// slide_puff effect at their feet, on while they slide.
fn slide_dust(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
              game: Res<GameData>, ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
              mut effects: Query<(&mut bf_viewer::ale_fx::AleEffect, &mut Transform)>) {
    commands.insert_resource(bf_viewer::ale_fx::AleClock(frame_dt(&time)));
    let Some(mut ale) = ale else { return };
    let Some(&effect) = game.0.effect_types.get(&SLIDE_EFFECT).and_then(|l| l.first()) else { return };
    for (i, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        let feet = Vec3::new(u.position.x, u.position.y + GROUND, u.position.z);
        match u.slide_fx.and_then(|e| effects.get_mut(e).ok()) {
            Some((mut fx, mut tr)) => {
                fx.active = u.slide_on && !u.dead;
                tr.translation = feet;
                tr.rotation = Quat::from_rotation_y(u.yaw);
            }
            None if u.slide_on => {
                // compiled at setup; no images or materials needed here
                if let Some(fx) = ale.cached(effect) {
                    u.slide_fx = Some(commands.spawn((Transform::from_translation(feet), Visibility::default(),
                        bf_viewer::ale_fx::AleEffect::new(fx, 0.0, 0x51DE_0000 + i as u32))).id());
                }
            }
            None => {}
        }
    }
}

/// BF_AUDIO_LOG=1: every 2 s, how many sounds are playing (sinks exist only once the audio
/// output is open) and how many are waiting for one.
fn audio_probe(time: Res<Time>, mut last: Local<f32>, sinks: Query<&AudioSink>, players: Query<(), With<AudioPlayer>>) {
    if std::env::var("BF_AUDIO_LOG").is_err() {
        return;
    }
    let t = time.elapsed_secs();
    if t - *last >= 2.0 {
        *last = t;
        let playing = sinks.iter().filter(|s| !s.is_paused() && !s.empty()).count();
        println!("audio t={t:.0}: {} players, {} sinks, {playing} playing, muted {:?}", players.iter().count(), sinks.iter().count(),
                 sinks.iter().next().map(|s| s.is_muted()));
    }
}

/// Backspace: back to the map menu (bf_play starts again there).
fn back_to_menu(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>, mode: Res<Mode>, map: Res<CurrentMap>,
                mut commands: Commands, mut frames: Local<u32>) {
    // test hook: BF_TEST_BACK=<n> goes back after n frames of the map
    *frames += 1;
    let test = std::env::var("BF_TEST_BACK").ok().and_then(|v| v.parse::<u32>().ok()) == Some(*frames);
    if keys.just_pressed(KeyCode::Backspace) || test {
        *frames = 0;
        commands.insert_resource(menu::Return { map: Some(map.0.clone()), deathmatch: mode.deathmatch });
        next.set(AppState::Menu);
    }
}

fn test_goto() -> Option<Vec<f32>> {
    std::env::var("BF_TEST_GOTO").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect::<Vec<f32>>())
        .filter(|g| g.len() >= 4)
}

/// The floor's height at (x, z) no higher than `top`: the map's terrain or an object's floor,
/// else the flat test floor.
fn floor_y(x: f32, z: f32, top: f32) -> f32 {
    world::arena().and_then(|a| a.floor_below(x, z, top)).map_or(GROUND, |f| f.0)
}

#[derive(Resource)]
struct GameData(Game);

/// One animation playing in the crossfade stack.
struct Layer {
    clip: usize,
    time: f32,
    weight: f32,
    /// one-shot: plays to its end instead of looping
    once: bool,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Gait {
    #[default]
    Idle,
    Walk,
    Run,
    Sprint,
    WalkBack,
    RunBack,
    /// aiming, stepping sideways with the body to the crosshair: left or right, the forward or
    /// the back diagonal (the motion scripts' f_side_walk / b_side_walk)
    Side { left: bool, back: bool },
}

/// Aiming and moving: a side step when the way the character moves is off the crosshair's
/// heading by SIDE_FROM to BACK_FROM radians (`rel`: positive is to the left), the forward
/// diagonal up to SIDE_BACK_FROM, the back one past it, if the set has that clip. Within
/// SIDE_FROM it walks or runs forward, past BACK_FROM it backpedals, as before.
fn side_step(rel: f32, set: &[Option<usize>; 4], armed: bool) -> Option<Gait> {
    let a = rel.abs();
    if !armed || !(SIDE_FROM..BACK_FROM).contains(&a) {
        return None;
    }
    let (left, back) = (rel > 0.0, a > SIDE_BACK_FROM);
    set[side_index(left, back)].map(|_| Gait::Side { left, back })
}

/// See `side_step` (radians: 30, 100 and 150 degrees).
const SIDE_FROM: f32 = 0.52;
const SIDE_BACK_FROM: f32 = 1.75;
const BACK_FROM: f32 = 2.62;

/// A side step's clip in a set of four (forward left, forward right, back left, back right).
fn side_index(left: bool, back: bool) -> usize {
    (back as usize) * 2 + (!left as usize)
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum Action {
    #[default]
    None,
    Dodge { left: bool },
    JumpCrouch,
    JumpLaunch,
    JumpFall,
    JumpLand,
    /// kneeling down (stand2crouch), kneeling (crouch idle, loops), getting up (crouch2stand)
    Crouching,
    Crouched,
    Rising,
    /// diving away (the motion script's dive)
    Dive,
}

impl Action {
    fn airborne(self) -> bool {
        matches!(self, Action::JumpLaunch | Action::JumpFall)
    }
}

/// Clips a character uses, by game name where the name is known.
#[derive(Default, Debug)]
struct Clips {
    loco: Locomotion,
    jump_crouch: Option<usize>,
    jump_launch: Option<usize>,
    jump_fall: Option<usize>,
    jump_land: Option<usize>,
    /// four-legged jump (Brutus sprints on all fours)
    leg4_launch: Option<usize>,
    leg4_fall: Option<usize>,
    leg4_land: Option<usize>,
    /// sliding down a slope too steep to stand on (motion "slide", script Sc_w1_slide_idle)
    slide: Option<usize>,
    /// per weapon slot (Sc_w1_* holds the first weapon, Sc_w2_* the second): carried and ready
    /// (rp_*, raised and pointing ahead, used while aiming or firing)
    slots: [SlotClips; 2],
}

#[derive(Default, Debug)]
struct SlotClips {
    carry: Locomotion,
    rp: Locomotion,
    stand2crouch: Option<usize>,
    crouch_idle: Option<usize>,
    crouch2stand: Option<usize>,
    /// crouch walk forward / back (the motion scripts' cr_walk, cr_back_walk)
    cr_walk: Option<usize>,
    cr_back_walk: Option<usize>,
    /// side steps while aiming, standing (rp_) and crouched (cr_): forward left, forward right,
    /// back left, back right (`side_index`)
    rp_side: [Option<usize>; 4],
    cr_side: [Option<usize>; 4],
    dive: Option<usize>,
}

/// A weapon the character carries.
struct HeldWeapon {
    def: WeaponDef,
    /// the stance (motion set Sc_w1_* = 0, Sc_w2_* = 1) for its class, see `stance_for`
    stance: usize,
    /// weapon root (child of the hand or back joint)
    entity: Entity,
    /// spinning parts (minigun barrel): entity, axis, offset
    spinners: Vec<(Entity, Vec3, Vec3)>,
    light: Entity,
    flash: Entity,
    muzzle: Hardpoint,
    /// fire direction in the weapon's frame
    fire_dir: Vec3,
    /// placement under a joint (bone, rotation, translation): in hand / stowed for its slot
    in_hand: Option<(usize, Quat, Vec3)>,
    stowed: Option<(usize, Quat, Vec3)>,
}

/// A one-shot overlay clip (reload, grenade throw): played on its bones over the locomotion.
struct OverlayClip {
    clip: usize,
    duration: f32,
    events: Vec<(f32, u32)>,
    /// every bone the clip animates (used standing still)
    mask: Vec<bool>,
    /// the same without the legs and pelvis (used on the move, so the legs keep running)
    upper: Vec<bool>,
}

impl OverlayClip {
    fn event(&self, id: u32) -> Option<f32> {
        self.events.iter().find(|e| e.1 == id).map(|e| e.0)
    }
}

/// reload clip event: magazine in
/// use_item clip events: the item in the hand, and used (the medkit's heal, then let go).
const EV_ITEM_IN_HAND: u32 = 0x0A6E_8F79;
const EV_ITEM_USED: u32 = 0x19F8_311B;
const EV_MAG_IN: u32 = 0x1B2E_C99E;
/// throw clip events: hand reaches the grenade (also in reloads), grenade leaves the hand
const EV_REACH: u32 = 0x1A6B_4920;
const EV_RELEASE: u32 = 0x1186_6F3A;
/// place_hi clip (Sc_w1/w2_place_hi: a placed grenade set down underhand) events: the hand
/// reaches it (EV_REACH, 0.30-0.47 s: the recordings' "in hand 0.38-0.47 s") and lets it go
/// (19f8311b, the use_item clip's "used" event, 0.50-0.60 s); it drops from the hand to the
/// ground (the recordings: on the ground ~0.8 s (Sentry) / ~1.05 s (Roller) after the press)
const EV_PLACED: u32 = EV_ITEM_USED;
/// An inventory item: a grenade type (an index into `grenade::GrenadeKits`, the same as into
/// `Player::grenades`) or the medkit.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Item {
    Grenade(usize),
    Medkit,
}
/// Tab held this long (s) opens the item list instead of stepping.
const ITEM_LIST_HOLD: f32 = 0.3;

/// The inventory items, in the item box's order: every grenade type, then the medkit.
fn items(p: &Player) -> Vec<Item> {
    (0..p.grenades.len()).map(Item::Grenade).chain([Item::Medkit]).collect()
}

/// How many of an item the squad carries.
fn item_count(p: &Player, item: Item) -> i64 {
    match item {
        Item::Grenade(k) => p.grenades.get(k).copied().unwrap_or(0),
        Item::Medkit => p.medkits,
    }
}

/// Whether the controlled character can use the item now (the item box shows it red when not:
/// a medkit at full health; an item their character can't use, e.g. Brutus and OrgSen, once
/// those items are in).
fn item_usable(p: &Player, item: Item) -> bool {
    !p.dead && item_count(p, item) > 0 && match item {
        Item::Grenade(_) => true,
        Item::Medkit => p.health < p.max_health,
    }
}

/// Select the next (or previous) item carried.
fn step_item(p: &mut Player, step: i32) {
    let list = items(p);
    let n = list.len() as i32;
    let here = list.iter().position(|&i| i == p.item).unwrap_or(0) as i32;
    for k in 1..=n {
        let i = list[(here + step * k).rem_euclid(n) as usize];
        if item_count(p, i) > 0 {
            if i != p.item {
                p.item = i;
                p.item_new = 0.0;
            }
            return;
        }
    }
}

/// Select the next grenade type carried (the demo's grenade key, T: the game's button for it
/// isn't in the recordings). From another item, the first type carried.
fn step_grenade(p: &mut Player) {
    let n = p.grenades.len();
    let here = match p.item { Item::Grenade(k) => k, _ => n.saturating_sub(1) };
    for k in 1..=n {
        let i = (here + k) % n;
        if p.grenades[i] > 0 {
            if Item::Grenade(i) != p.item {
                p.item = Item::Grenade(i);
                p.item_new = 0.0;
            }
            return;
        }
    }
}

/// The selected item ran out: the first grenade type carried if it was a grenade (the Sentry
/// recording: the last Sentry set down, the box shows Frag 10; the Frag is the first type),
/// else the next item carried.
fn item_ran_out(p: &mut Player) {
    if let (Item::Grenade(_), Some(k)) = (p.item, p.grenades.iter().position(|&n| n > 0)) {
        p.item = Item::Grenade(k);
        p.item_new = 0.0;
    } else {
        step_item(p, 1);
    }
}
/// The highest ledge a character steps onto; a bigger drop is a fall.
const STEP_UP: f32 = 0.5;
/// Sliding, as the game does it (default.xbe FUN_0012c020 / FUN_0012bc60, see
/// decompiled/xbe/ghidra/README.md "Player movement"): only on collision triangles flagged as
/// slide surfaces. A slide amount (0-1) grows while the character moves on ground steeper than
/// 50 degrees (to full rate at 55), 8 per second; at 1 the slide starts. Sliding, it keeps
/// growing (at 1 per second) down to 45 degrees and drains below. Flagged ground counts at least
/// half rate. The slide velocity is pushed downhill by (1 - normal y) x 8 at the start and
/// eases toward that much more (3 per second) every second after, plus 5 m/s^2 down, up to
/// 40 m/s; the character moves by it times the amount. Under an amount of 0.2 the player's own
/// movement blends back in.
const SLIDE_COS: [f32; 3] = [0.707_106_8, 0.642_787_6, 0.573_576_4];   // cos 45, 50, 55
const SLIDE_RATE: [f32; 2] = [8.0, 1.0];
const SLIDE_PUSH: f32 = 8.0;
const SLIDE_DOWN: f32 = -5.0;
const SLIDE_EASE: f32 = 3.0;
const SLIDE_MAX_SPEED: f32 = 40.0;
const SLIDE_RELEASE: f32 = 0.2;
/// A slide that achieves less than this part of its move is blocked and stops.
const SLIDE_BLOCKED: f32 = 0.1;
/// The slide clip shows for this long past the slide (keeps it from flickering).
const SLIDE_HOLD: f32 = 0.25;
/// A liquid this deep (m above the feet: the head under) kills, if it's a harmful one (any of
/// its LiquidType::damage values set: lava, toxic). From a capture of sdm_e13: Brutus waded
/// waist-deep through lava for 10 s at full health, then died the moment he went under.
const LIQUID_UNDER: f32 = 1.6;
/// A harmful liquid (any damage rate set: lava, toxic) burns from the first touch: LIQUID_TOUCH
/// of its highest rate (per second) at the surface, rising with depth to all of it at
/// LIQUID_UNDER; dealt every LIQUID_TICK s. (A capture of sdm_e13 had Brutus wade waist-deep
/// through lava unhurt; this is the demo's choice: touching it hurts.)
const LIQUID_TOUCH: f32 = 0.2;
const LIQUID_TICK: f32 = 0.4;
/// Falls (FUN_0012b740): from 5 m a landing hurts, 100 x ((drop - 5) / 25)^2, up to 100 at 30 m.
const FALL_HURT: (f32, f32, f32) = (5.0, 30.0, 100.0);
/// Effect type of the dust kicked up sliding (objecttypes <effect>: slide_puff).
const SLIDE_EFFECT: u32 = 0x0151_3B0F;
/// seconds to charge a throw fully (the recordings: a linear fill, full in 0.60 s, then held)
const CHARGE_TIME: f32 = 0.6;
/// After the button is let go the meter holds its level METER_HOLD s, then fades over
/// METER_FADE s (the recordings: ~0.5 s and ~0.1 s).
pub const METER_HOLD: f32 = 0.5;
pub const METER_FADE: f32 = 0.1;
/// seconds the chosen member's name shows before control passes to them
const SELECT_TIME: f32 = 0.6;
/// chatter line set the character you hand over from says as they drop back into the squad
/// ("I'm right behind you", "Following at a distance", ...: line_tag f415ab02 in all four
/// <name>_chatter files; the capture's switches transcribe and match to these, said by the
/// previous character right at the camera cut)
const FOLLOW_CHATTER: u32 = 0xF415_AB02;
/// pain grunts ("Ugh", "Ahh": line_tag e856009f, transcribed) said when hurt by a blast
const HURT_CHATTER: u32 = 0xE856_009F;
/// "I'm hit!" (0ae8104e) and, shot by a teammate, "Careful!" / "Stop shooting at me!" (e2d9fb13,
/// friendly_fire)
const HIT_CHATTER: u32 = 0x0AE8_104E;
const FRIENDLY_FIRE_CHATTER: u32 = 0xE2D9_FB13;
/// the death cry: line_tag e1213623 ("No!", "They finally got me.", "Sorry guys." - the
/// capture's two deaths match Brutus's us00569 and Hawk's us0010 in this set). Its block's
/// response names the line a surviving squadmate answers with ("Brutus is down!", "They've taken
/// out Hawk."...: 0ace9296 / 0cceddf8 / ea52e942 / efc7d9fe), said at the data's 50% chance
const DEATH_CHATTER: u32 = 0xE121_3623;
const DEATH_RESPONSE_CHANCE: usize = 2;
/// a single hit this hard can knock a character down (the L-Shot's 23-27 and Bower's 16-20 can,
/// the rifles' 2-13 can't; capture: L-Shot hits floor a squadmate for ~0.6 s before they get up)
const KNOCKDOWN_DAMAGE: f32 = 15.0;
const KNOCK_DOWN_TIME: f32 = 0.7;
/// ragdoll physics substep (s)
const RAGDOLL_STEP: f32 = 1.0 / 60.0;
/// The ragdoll's collision: limb bones are spheres this size that don't pass through each other
/// (between bones more than RAGDOLL_SELF_HOPS apart in the skeleton) or walls; floors are the
/// level's (no higher than this above a bone).
const RAGDOLL_SELF_RADIUS: f32 = 0.09;
const RAGDOLL_SELF_HOPS: usize = 3;
/// How the body falls: a bone keeps this much of its speed each substep; on the ground it loses
/// this part of its sliding; it rests after this many still substeps. Non-hinge joints hold their
/// bend this stiffly (fingers, toes, the spine's links outside the torso pieces); the waist joins
/// the hips and chest this stiffly. Knees fold to at most this part of their straight length
/// (about 120 degrees), elbows a little more.
const RAGDOLL_DAMPING: f32 = 0.985;
const RAGDOLL_FRICTION: f32 = 0.6;
const RAGDOLL_SETTLE: u32 = 20;
/// A body come to rest on its side (its chest's left-right axis more than ROLL_SIDE upright) is
/// rolled on: its upper shoulder pushed ROLL_PUSH m/s toward its back (or front, if it leans
/// that way), up to ROLL_TRIES times. The demo's choice: bodies don't stay balanced on a
/// shoulder.
const ROLL_SIDE: f32 = 0.6;
const ROLL_PUSH: f32 = 2.0;
const ROLL_TRIES: u8 = 3;
/// A shot into a body on the ground: the push at the bone it hits (m/s) and how far round that
/// bone others are pushed too, less with distance (m). The demo's choice.
const SHOT_SHOVE: f32 = 3.0;
const SHOT_SHOVE_REACH: f32 = 0.6;
const RAGDOLL_BEND_STIFFNESS: f32 = 0.25;
const RAGDOLL_WAIST_STIFFNESS: f32 = 0.35;
const RAGDOLL_KNEE_FOLD: f32 = 0.5;
const RAGDOLL_ELBOW_FOLD: f32 = 0.4;
const RAGDOLL_STEP_UP: f32 = 0.4;
const GET_UP_TIME: f32 = 0.45;
/// grenade landing this close makes a squadmate dive away (EVT_GRENADE_NEAR -> GOAL_DIVE)
pub const DIVE_RADIUS: f32 = 6.0;
/// characters' height for shots (m above the ground)
const BODY_HEIGHT: f32 = 1.9;
const QUOTE_DELAY: f32 = 0.05;
/// characters' collision radius (m)
const BODY_RADIUS: f32 = 0.4;
/// wingman places (x right, z back, in the frame of the leader's heading), by member: the
/// captures' squad keeps 4-12 m ahead of the leader (measured from character sizes on screen:
/// running 5-6 m ahead +-2 m, stopped 4-12 m ahead up to 7 m aside)
const FORMATION: [Vec2; 3] = [Vec2::new(-3.0, -6.5), Vec2::new(3.5, -8.0), Vec2::new(-1.6, -11.5)];
/// while the leader aims or fires, squadmates keep out of this wide a lane along the aim
/// (EVT_SHOT_BLOCKED_BY_FRIEND -> GOAL_REPOSITION: step aside rather than block the shot)
const FIRE_LANE: f32 = 1.8;
/// a squadmate sets off when this far from their place and settles within SLOT_STOP; runs, and
/// dashes when far behind (the AI's leash: past squad-leash-dist + 10 m it must dash)
const SLOT_START: f32 = 3.0;
const SLOT_STOP: f32 = 1.0;
const SLOT_DASH: f32 = 8.0;
/// standing still this long (+ up to 1.5 s) a squadmate kneels (crouch idle, weapon ready);
/// Brutus stays on his feet (both as in the captures)
const KNEEL_AFTER: f32 = 1.2;
/// The squad's idle variations while holding position (wingman personality h_ea20a17d,
/// EVT_IDLE_VARY_DATA): IDLE_VARY_DATA_PAUSE_CROUCH and IDLE_VARY_DATA_PAUSE_CAUTIOUS, equally
/// likely, lasting 15 and 12 s. The first pause is the crouch (the captures); then they're drawn.
/// A cautious pause: standing, weapon ready, looking left and right of the heading.
const IDLE_CROUCH_TIME: f32 = 15.0;
const IDLE_CAUTIOUS_TIME: f32 = 12.0;
const CAUTIOUS_LOOK: f32 = 0.6;
const CAUTIOUS_SWEEP: f32 = 0.35;
/// Shot at (EVT_SHOT_AT, pri 20.15): a squadmate the player's shot passes within this distance of
/// (without hitting) sidesteps with the table's odds, GOAL_DODGE x1 against GOAL_NOOP x7. (The
/// distance is not in the data; a near miss is taken as 1.5 m.)
const SHOT_AT_RADIUS: f32 = 1.5;
const SHOT_AT_DODGE_ODDS: usize = 8;

/// A reload in progress: the clip is refilled at the clip's magazine-in event.
struct Reload {
    weapon: usize,
    time: f32,
    fill: i64,
    filled: bool,
}

/// A grenade throw in progress (or a placed one being set down).
struct Throw {
    /// charge when let go (0..1): how hard it is thrown
    power: f32,
    time: f32,
    slot: usize,
    grabbed: bool,
    released: bool,
    /// the grenade type (a `grenade::GrenadeKits` index)
    kind: usize,
    /// set down at the feet (IOU_PLACE_ON_GROUND): the stance's place_hi clip, not the throw
    place: bool,
}

/// Weapon-switch sounds, hard-coded in default.xbe (0xcbfda): the first when a switch starts, the
/// second when it completes; in a capture of the game they play 0.63-0.66 s apart.
const SWITCH_SOUNDS: [u32; 2] = [0xFF82_0FDA, 0xE1E9_7460];
const SWITCH_SOUND_GAP: f32 = 0.65;
/// Handing control to another squad member: one sound as the camera cuts to them (common bank,
/// 0.18 s; a capture has it at all five hand-overs, lined up with the cut).
const SQUAD_SWITCH_SOUND: u32 = 0x0794_9F0D;
/// seconds the HUD lists every weapon after a switch (as in the game capture)
const HUD_LIST_TIME: f32 = 2.5;
/// reload length when the definition has none (the capture shows ~0.7 s)
const RELOAD_TIME: f32 = 0.7;
/// the game's crosshair sits above the screen centre: y = 187 of 480, i.e. this fraction of the
/// half-height above the middle; shots and aiming go through it
const CROSSHAIR_UP: f32 = (240.0 - 187.0) / 240.0;
/// Aiming a weapon that zooms (WeaponDef::zoom > 1) goes into the scope: the camera moves to the
/// character's eyes (this far above and in front of the camera target, m), the field of view
/// narrows by the weapon's zoom, the character's model is hidden, and its snipe-sound breathing
/// plays in a loop (BREATH_GAP between breaths) while it stays there.
const EYE_ABOVE_TARGET: f32 = 0.45;
const EYE_FORWARD: f32 = 0.35;
const SCOPE_RATE: f32 = 10.0;
const BREATH_GAP: f32 = 0.6;
const BREATH_VOLUME: f32 = 0.6;
const SCOPE_SOUND_VOLUME: f32 = 0.8;
/// Flint, the squad's sniper (a synthetic), holds her scope dead still (a capture); the others'
/// aim wanders while they look through one: a slow drift of up to SWAY radians.
const STILL_SNIPER: usize = 1;
/// Flint's scope has a second step: the weapon's zoom doubled (L-Shot-50: 5x, then 10x - the
/// L-Shot-75's). The data has one zoom per weapon; the doubling is a choice.
const SNIPER_SECOND_STEP: f32 = 2.0;
const SWAY: f32 = 0.012;
/// In the scope the eye keeps this far off walls (it sits in front of the head otherwise and
/// would look through a barrier close by).
const EYE_CLEARANCE: f32 = 0.25;
/// vertical field of view (Bevy's default perspective)
const FOV_Y: f32 = std::f32::consts::FRAC_PI_4;

/// A weapon-switch overlay clip (upper body) and its events.
struct SwitchClip {
    clip: usize,
    duration: f32,
    /// "drop_weapon": the held gun goes to its stow point
    drop: f32,
    /// the hand takes the other gun
    grab: f32,
    /// bones the overlay moves
    mask: Vec<bool>,
}

/// A switch in progress.
struct Switching {
    to: usize,
    time: f32,
    dropped: bool,
    grabbed: bool,
}

struct Loaded {
    index: usize,
    root: Entity,
    joints: Vec<Entity>,
    model: Character,
    clips: Clips,
    aim_chain: Vec<usize>,
    /// spine bones carrying the trigger arm (not the head, not above the legs): turned to aim guns
    arm_chain: Vec<usize>,
    /// left / right lowest foot bones and their planted (resting) heights
    feet: [usize; 2],
    /// how far the model is raised so its soles stand on the floor: the floor is GROUND under
    /// the root, the idle pose's lowest skinned vertex is this much lower (see `sole_lift`)
    lift: f32,
    /// the weapon in hand has been let go (dead: see `update_weapons`)
    weapon_dropped: bool,
    /// the crouch clips' own lift (see `clip_lift`): a kneel's lowest point isn't the standing
    /// soles', so `lift` would leave it floating or sunk
    crouch_lift: HashMap<usize, Vec<f32>>,
    foot_rest: [f32; 2],
    /// per locomotion clip: lowest and highest height of each foot over the clip
    foot_range: HashMap<usize, [(f32, f32); 2]>,
    footstep_type: i64,
    jump_sound: u32,
    weapons: Vec<HeldWeapon>,
    /// switch clips by the slot being switched to (0: from the second weapon, 1: to it)
    switch_clips: [Option<SwitchClip>; 2],
    /// reload and grenade-throw clips by stance slot (Sc_w1_ / Sc_w2_)
    reload_clips: [Option<OverlayClip>; 2],
    /// the stance's use_item overlay (Sc_w1_/Sc_w2_use_item: a medkit used)
    use_clips: [Option<OverlayClip>; 2],
    throw_clips: [Option<OverlayClip>; 2],
    /// the stance's place_hi overlay (Sc_w1/w2_place_hi: a Roller or Sentry set down)
    place_clips: [Option<OverlayClip>; 2],
    /// the throwing hand: bone and the grip point in its frame
    throw_hand: Option<(usize, Vec3)>,
}

/// A shot for the effects system: from the muzzle along `dir`, `dist` metres to a hit (or range).
/// `hit`: it ends on the world (a spark there); a body hit gets the flesh effects instead.
/// A shot on its way to a squad member: it lands after `delay` s (blood at `local` from them).
#[derive(Clone, Copy)]
struct PendingHit {
    delay: f32,
    member: usize,
    amount: f32,
    dir: Vec3,
    local: Vec3,
    ammo: i64,
}

#[derive(Clone, Copy)]
struct Shot {
    origin: Vec3,
    dir: Vec3,
    dist: f32,
    hit: bool,
    speed: f32,
    /// the weapon's effect types: in flight, and where the shot lands (0: none)
    flight: u32,
    hit_fx: u32,
}

#[derive(Resource)]
struct Player {
    character: usize,
    loaded: Option<Loaded>,
    position: Vec3,
    /// facing angle about +Y; the model faces -Z at yaw 0
    yaw: f32,
    layers: Vec<Layer>,
    gait: Gait,
    action: Action,
    /// seconds left in the current one-shot action clip
    action_left: f32,
    on_all_fours: bool,
    // jump physics
    height: f32,
    vy: f32,
    air_velocity: Vec3,
    last_velocity: Vec3,
    face_time: f32,
    sim_time: f32,
    // input (filled by read_input)
    move_input: Vec2,
    sprint: bool,
    walk: bool,
    aim: bool,
    jump_pressed: bool,
    /// a jump pressed during another action fires when it ends (seconds left)
    jump_buffer: f32,
    dodge_pressed: bool,
    next_surface: bool,
    fire: bool,
    switch_pressed: bool,
    twist: f32,
    // weapons
    weapon: usize,
    weapon_dirty: bool,
    /// false between a switch's drop and grab: both guns are stowed
    holding: bool,
    switching: Option<Switching>,
    /// per weapon: rounds in the clip, rounds in reserve
    ammo: Vec<[i64; 2]>,
    /// reload in progress: (weapon, seconds left, clip it fills to)
    reloading: Option<Reload>,
    /// grenades carried, by type (`grenade::GrenadeKits` order; filled by play_grenade.rs's
    /// stock_inventory once the kits are loaded)
    grenades: Vec<i64>,
    /// medkits carried (shared by the squad, like the grenades; see play_pickups.rs)
    medkits: i64,
    /// the inventory item in the item box, how long it's been NEW (s left),
    /// whether the item list is open (Tab held), how long Tab's been down, and the use key
    /// (G) pressed for a non-grenade item
    item: Item,
    item_new: f32,
    item_list: bool,
    tab_down: f32,
    item_use: bool,
    /// a medkit being used: time into the use_item clip; `item_in_hand` while the clip has it
    /// (its 0a6e8f79 event to its 19f8311b), `item_used` set the frame it's used (heal, sound,
    /// drop: see play_pickups.rs)
    using: Option<f32>,
    item_in_hand: bool,
    item_used: bool,
    /// the inventory item they are (what using one heals)
    medkit_kind: u32,
    test_medkit_used: bool,
    /// throw button held: charging (the HUD meter shows `charge`)
    throw_held: bool,
    charge: f32,
    /// the meter after the button is let go: its level and how long it still shows (s)
    meter_after: (f32, f32),
    /// a place-on-ground type was set down on this press (wait for the button to come up)
    place_latch: bool,
    throwing: Option<Throw>,
    /// set at a throw's release: (power, type, placed); the grenade leaves the hand once the
    /// pose is known
    pending_release: Option<(f32, usize, bool)>,
    /// released grenades to spawn (play_grenade.rs)
    thrown: Vec<grenade::Thrown>,
    /// control is passing to this squad member (character index) in this many seconds
    select: Option<(usize, f32)>,
    /// a voice line to say in this many seconds
    quote_in: Option<(f32, u32)>,
    /// seconds left of the voice line being said (the HUD shows their speech icon)
    speaking: f32,
    /// hitpoints (the character's combat-target hitpoints at the start)
    health: f32,
    max_health: f32,
    dead: bool,
    /// the body once dead (physics on the skeleton)
    ragdoll: Option<Ragdoll>,
    /// push given to the body when it dies (world space, m/s)
    death_push: Vec3,
    /// the last animated pose's bone matrices (model space): where the ragdoll starts
    last_world: Vec<Mat4>,
    /// the crosshair is on a teammate (the HUD turns it green)
    aim_friend: bool,
    /// seconds before the next "I'm hit" line
    hurt_quiet: f32,
    /// knocked down: limp, then getting up (requested with the hit's push)
    knock: Option<Knock>,
    knock_request: Option<Vec3>,
    knock_cooldown: f32,
    /// AI stance and reactions: kneel wanted, seconds standing still (+ this member's extra
    /// delay), heading to face when idle, dodge (left?) / dive away from a point
    crouch_wanted: bool,
    still_time: f32,
    kneel_jitter: f32,
    face_yaw: Option<f32>,
    dodge_request: Option<bool>,
    dive_from: Option<Vec3>,
    /// the idle variation going on while holding position (cautious, else crouch) and how long
    /// it has left (see IDLE_CROUCH_TIME)
    idle_cautious: bool,
    idle_left: f32,
    /// the AI's squad-leash-dist (m)
    leash: f32,
    /// hits to show (blood, sparks, hit sound): hit point, shot direction (world), ammo type (-1:
    /// a blast)
    blood: Vec<(Vec3, Vec3, i64)>,
    /// the body has hit the ground (thud played) / where it lies
    thud: bool,
    body_at: Option<Vec3>,
    /// seconds since death; the DNA and the blood pool under the body are out
    dead_for: f32,
    dna_done: bool,
    /// where they stood last frame (x, z)
    prev_xz: Vec2,
    /// seconds left of sliding down steep ground (set while the slope carries them downhill),
    /// the way down (yaw), whether the slide clip is on, and the slide puff at their feet
    sliding: f32,
    /// the game's slide state: amount (0-1), whether the slide is on, its velocity
    slide_amount: f32,
    slide_active: bool,
    slide_vel: Vec3,
    slide_yaw: f32,
    /// feet height (world) where the current jump or fall began, and whether they were in the
    /// air last frame
    fall_from: f32,
    was_air: bool,
    slide_on: bool,
    slide_fx: Option<Entity>,
    pool_done: bool,
    /// the line a surviving squadmate says about this death, due in this many seconds
    death_response: Option<(f32, u32)>,
    /// squad AI fire: wait before joining in, and the current burst / pause
    ai_delay: f32,
    ai_burst: bool,
    ai_phase: f32,
    reload_pressed: bool,
    /// the use key (E: a gate's wall panel), held
    use_held: bool,
    /// a reload asked for while busy (switching, already reloading) starts when it can
    reload_wanted: bool,
    /// seconds the HUD keeps the full weapon list open (after a switch)
    hud_list: f32,
    show_help: bool,
    /// how far the model is raised this frame: its stance's lift (`Loaded::lift`), or the
    /// crouch clips' own, by their weights (see `clip_lift`)
    lift_now: f32,
    /// seconds until the switch's closing sound (SWITCH_SOUNDS[1]); negative when none is due
    switch_sound_in: f32,
    cooldown: f32,
    /// seconds the upper body keeps aiming after the last shot
    aim_hold: f32,
    /// model-space yaw of the held weapon's fire direction (from the animation), smoothed
    muzzle_off: f32,
    /// 0..1 blend of gun aiming (turning the arm chain onto the crosshair)
    aim_weight: f32,
    /// yaw the spine still has to turn to put the gun on the crosshair (last frame); standing,
    /// the feet take all but a little of it so the body doesn't stay twisted
    aim_residual: f32,
    recoil: f32,
    flash: f32,
    spin: f32,
    spin_angle: f32,
    shots: Vec<Shot>,
    /// the player's shots still flying toward squad members
    pending_hits: Vec<PendingHit>,
    shots_fired: u32,
    pending_shot: bool,
    // sound
    surface: usize,
    foot_prev: [f32; 2],
    sound_queue: Vec<(u32, f32)>,
    rng: u32,
    /// seconds of no footsteps after a landing (its thud covers the feet touching down)
    step_mute: f32,
    // camera
    cam_yaw: f32,
    cam_pitch: f32,
    cam_distance: f32,
    cam_target: Vec3,
    /// 0..1 blend of the over-the-shoulder aiming camera
    shoulder: f32,
    /// the scope: 0..1 blend into the sniping view (the character's `offset-snipe`, at its
    /// eyes) and the zoom it gives (the held weapon's, 1 none)
    scope: f32,
    zoom: f32,
    /// the character's breathing in the scope (its snipe-sound, how long it lasts) and when it's
    /// next due
    snipe_sound: Option<(u32, f32)>,
    breath_in: f32,
    /// the scope's zoom in / out sounds, and whether the scope was up last frame
    scope_sounds: (u32, u32),
    was_scoped: bool,
    /// the scope's step (0 out; 1 in; 2 closer, Flint only), toggled by right mouse
    scope_level: u8,
    /// (BF_TEST_SELECT done)
    test_selected: bool,
    /// the scoped aim's wander (yaw, pitch; radians)
    sway: Vec2,
    /// in a liquid (its surface above the feet), the time to the next wading ring, and the
    /// splashes made this frame (where, ALE effect); see `splashes`
    wet: bool,
    wade_in: f32,
    splashes: Vec<(Vec3, u32)>,
    /// a harmful liquid's damage not yet dealt, and the time to the next dealing (see
    /// LIQUID_TOUCH)
    burn: f32,
    burn_in: f32,
}

impl Player {
    fn new(character: usize) -> Self {
        Self {
            character, loaded: None, position: Vec3::ZERO, yaw: 0.0, layers: vec![], gait: Gait::Idle,
            action: Action::None, action_left: 0.0, on_all_fours: false,
            height: 0.0, vy: 0.0, air_velocity: Vec3::ZERO, last_velocity: Vec3::ZERO, face_time: 0.0, sim_time: 0.0,
            move_input: Vec2::ZERO, sprint: false, walk: false, aim: false, jump_pressed: false, jump_buffer: 0.0, dodge_pressed: false,
            next_surface: false, fire: false, switch_pressed: false, twist: 0.0,
            weapon: 0, weapon_dirty: true, holding: true, switching: None, ammo: vec![], reloading: None, grenades: vec![], medkits: 0, item: Item::Grenade(0), item_new: 0.0, item_list: false, tab_down: -1.0, item_use: false, using: None, item_in_hand: false, item_used: false, medkit_kind: 0, test_medkit_used: false, throw_held: false, charge: 0.0, meter_after: (0.0, 0.0), place_latch: false, throwing: None, pending_release: None, thrown: vec![], select: None, quote_in: None, speaking: 0.0, health: 100.0, max_health: 100.0, dead: false, ragdoll: None, death_push: Vec3::ZERO, last_world: vec![], aim_friend: false, hurt_quiet: 0.0, knock: None, knock_request: None, knock_cooldown: 0.0, crouch_wanted: false, still_time: 0.0, kneel_jitter: 0.0, face_yaw: None, dodge_request: None, dive_from: None, idle_cautious: false, idle_left: 0.0, leash: 15.0, blood: vec![], thud: false, body_at: None, dead_for: 0.0, dna_done: false, prev_xz: Vec2::ZERO, sliding: 0.0, slide_amount: 0.0, slide_active: false, slide_vel: Vec3::ZERO, slide_yaw: 0.0, fall_from: 0.0, was_air: false, slide_on: false, slide_fx: None, pool_done: false, death_response: None, ai_delay: 0.0, ai_burst: false, ai_phase: 0.0, reload_pressed: false, use_held: false, reload_wanted: false, hud_list: 0.0, show_help: false, switch_sound_in: -1.0, cooldown: 0.0, aim_hold: 0.0, muzzle_off: 0.0, aim_weight: 0.0, aim_residual: 0.0, recoil: 0.0, flash: 0.0, lift_now: 0.0,
            spin: 0.0, spin_angle: 0.0, shots: vec![], pending_hits: vec![], shots_fired: 0, pending_shot: false,
            surface: usize::MAX, foot_prev: [1.0; 2], sound_queue: vec![], rng: 0x1234_5678, step_mute: 0.0,
            cam_yaw: 0.0, cam_pitch: -0.18, cam_distance: 3.6, cam_target: Vec3::new(0.0, 0.3, 0.0),
            shoulder: 0.0, scope: 0.0, zoom: 1.0, snipe_sound: None, breath_in: 0.0, scope_sounds: (0, 0), was_scoped: false, scope_level: 0, test_selected: false, sway: Vec2::ZERO,
            wet: false, wade_in: 0.0, splashes: vec![], burn: 0.0, burn_in: 0.0,
        }
    }

    /// (see `splashes`)
    fn splash(&mut self, at: Vec3, liquid: &LiquidType, slot: usize) {
        if liquid.effects[slot] != 0 {
            self.splashes.push((at, liquid.effects[slot]));
        }
        if liquid.sounds[slot] != 0 {
            self.sound_queue.push((liquid.sounds[slot], 1.0));
        }
    }

    fn random(&mut self, n: usize) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng as usize) % n.max(1)
    }

    fn queue_any(&mut self, ids: &[u32], volume: f32) {
        if !ids.is_empty() {
            let id = ids[self.random(ids.len())];
            self.sound_queue.push((id, volume));
        }
    }
}

/// The squad members not under control: AI that follows in formation and fights alongside.
#[derive(Resource, Default)]
struct Squad(Vec<Player>);

/// Squad AI: keep to a formation place round the leader (run / sprint to catch up, walk the
/// last bit) and look where the leader looks. They shoot only at enemies they can see (none
/// yet). Reloads happen by themselves (empty clip).
fn squad_ai(m: &mut Player, leader: &Player, slot: usize, heading: f32, dt: f32) {
    let f = FORMATION[slot % FORMATION.len()];
    let mut goal = leader.position + Quat::from_rotation_y(heading) * Vec3::new(f.x, 0.0, f.y);
    // out of the leader's line of fire: a place in the lane moves to its side, and anyone
    // standing in it steps out
    let mut in_lane = false;
    if leader.aim || leader.aim_hold > 0.0 {
        let lf = Vec3::new(-leader.cam_yaw.sin(), 0.0, -leader.cam_yaw.cos());
        let lr = Vec3::new(leader.cam_yaw.cos(), 0.0, -leader.cam_yaw.sin());
        let side_of = |p: Vec3| { let r = p - leader.position; (r.dot(lf), r.dot(lr)) };
        let (ma, ms) = side_of(m.position);
        in_lane = ma > 0.0 && ms.abs() < FIRE_LANE;
        let (ga, gs) = side_of(goal);
        if ga > 0.0 && gs.abs() < FIRE_LANE + 0.5 {
            let side = if ms.abs() > 0.2 { ms.signum() } else if gs.abs() > 0.2 { gs.signum() } else { 1.0 };
            goal += lr * (side * (FIRE_LANE + 1.2) - gs);
        }
    }
    let to = Vec3::new(goal.x - m.position.x, 0.0, goal.z - m.position.z);
    let dist = to.length();
    // their "camera" is the leader's, so movement and aim share the leader's frame
    m.cam_yaw = leader.cam_yaw;
    m.cam_pitch = leader.cam_pitch;
    m.cam_distance = leader.cam_distance;
    let fwd = Vec3::new(-m.cam_yaw.sin(), 0.0, -m.cam_yaw.cos());
    let right = Vec3::new(m.cam_yaw.cos(), 0.0, -m.cam_yaw.sin());
    let moving = m.move_input.length() > 0.1;
    m.jump_pressed = false;
    m.dodge_pressed = false;
    // shot by the player: sidestep (EVT_DAMAGED_BY_PC -> GOAL_DODGE)
    if let Some(left) = m.dodge_request.take() {
        m.dodge_pressed = true;
        m.move_input = Vec2::new(if left { -1.0 } else { 1.0 }, 0.0);
        m.crouch_wanted = false;
        return;
    }
    // wingman follow: set off when this far from the place, settle when close; dash when far
    // behind (or past the data's leash + 10 m), walk the last bit
    let go = (if moving { dist > SLOT_STOP } else { dist > SLOT_START }) || (in_lane && dist > 0.3);
    if go {
        let d = to / dist;
        m.move_input = Vec2::new(d.dot(right), d.dot(fwd));
        m.sprint = dist > SLOT_DASH || dist > m.leash + 10.0;
        m.walk = dist < 2.5;
        m.still_time = 0.0;
        m.crouch_wanted = false;
        m.idle_cautious = false;
        m.idle_left = 0.0;
    } else {
        m.move_input = Vec2::ZERO;
        m.sprint = false;
        m.walk = false;
        m.still_time += dt;
        // after a moment, the idle variations: a crouch first, then crouch or cautious pauses
        // drawn from the personality's table (Brutus doesn't kneel: his crouch pause stands)
        if m.still_time > KNEEL_AFTER + m.kneel_jitter {
            if m.idle_left <= 0.0 {
                // (exactly 0: the first pause since moving)
                let first = m.idle_left == 0.0;
                m.idle_cautious = !first && m.random(2) == 0;
                m.idle_left = if m.idle_cautious { IDLE_CAUTIOUS_TIME } else { IDLE_CROUCH_TIME };
                if std::env::var("BF_AI_LOG").is_ok() {
                    println!("{} idles: {} for {} s", CHARACTERS[m.character], if m.idle_cautious { "cautious" } else { "crouch" }, m.idle_left);
                }
            }
            m.idle_left -= dt;
        }
        m.crouch_wanted = m.character != 0 && m.still_time > KNEEL_AFTER + m.kneel_jitter && !m.idle_cautious;
    }
    m.face_yaw = Some(heading);
    let cautious = m.idle_cautious && m.move_input == Vec2::ZERO;
    if cautious {
        // look about: the ready weapon sweeps across the heading
        let sweep = CAUTIOUS_LOOK * (m.still_time * CAUTIOUS_SWEEP * std::f32::consts::TAU + slot as f32).sin();
        m.cam_yaw = leader.cam_yaw + sweep;
        m.face_yaw = Some(heading + sweep);
    }
    // squadmates only aim and shoot at an enemy they can see - never just because the player
    // fires (the demo has no enemies yet, so they hold fire). When engaged, each picks a random
    // moment to open up, then fires in bursts and pauses of its own length.
    let enemy_in_sight = false;
    m.aim = enemy_in_sight || cautious;
    if enemy_in_sight {
        if m.ai_delay > 0.0 {
            m.ai_delay -= dt;
            m.fire = false;
        } else {
            m.ai_phase -= dt;
            if m.ai_phase <= 0.0 {
                m.ai_burst = !m.ai_burst;
                let r = m.random(1000) as f32 / 1000.0;
                m.ai_phase = if m.ai_burst { 0.4 + 0.9 * r } else { 0.25 + 0.7 * r };
            }
            m.fire = m.ai_burst;
        }
    } else {
        m.fire = false;
        m.ai_burst = false;
        m.ai_phase = 0.0;
        m.ai_delay = 0.2 + 0.6 * (m.random(1000) as f32 / 1000.0);
    }
    m.switch_pressed = false;
    m.reload_pressed = false;
    m.throw_held = false;
    m.next_surface = false;
}

/// Hand control to another squad member: their name shows over them for SELECT_TIME (as in the
/// game), then they become the player and the camera cuts behind them; the previous character
/// joins the squad AI.
fn squad_control(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut test_died: ResMut<TestDied>) {
    let dt = frame_dt(&time);
    // voice lines due (whoever says them; the HUD shows their speech icon while they talk)
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        u.speaking -= dt;
        if let Some((left, id)) = u.quote_in {
            u.quote_in = if left - dt > 0.0 { Some((left - dt, id)) } else {
                u.sound_queue.push((id, 1.0));
                u.speaking = game.0.sounds.pcm(id).map_or(1.0, |(rate, s)| s.len() as f32 / rate as f32);
                None
            };
        }
    }
    // a squadmate answers a death ("Brutus is down!"): a living member who isn't talking
    let mut answers = vec![];
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if let Some((left, tag)) = u.death_response {
            u.death_response = if left - dt > 0.0 { Some((left - dt, tag)) } else { answers.push((u.character, tag)); None };
        }
    }
    for (dead, tag) in answers {
        let mut alive: Vec<&mut Player> = std::iter::once(&mut *player).chain(squad.0.iter_mut())
            .filter(|u| !u.dead && u.character != dead && u.speaking <= 0.0 && u.quote_in.is_none()).collect();
        if !alive.is_empty() {
            let n = alive.len();
            let k = alive[0].random(n);
            say(alive[k], &game.0, tag, 0.0);
        }
    }
    // test hook: BF_TEST_KILL=<character> - from 1 s, turn to that squad member and keep firing
    if let Some(c) = std::env::var("BF_TEST_KILL").ok().and_then(|v| v.parse::<usize>().ok()) {
        if let Some(m) = squad.0.iter().find(|m| m.character == c) {
            if player.sim_time > 1.0 && !player.dead {
                let d = m.position - player.position;
                player.cam_yaw = (-d.x).atan2(-d.z);
                player.cam_pitch = -0.12;
                player.fire = !m.dead;
                player.aim = !m.dead;
            }
        }
    }
    // test hook: BF_TEST_DIE=<s> - the player drops dead at that time (BF_TEST_DIE=1: at 1 s),
    // shoved sideways. Once: the member who takes over lives on.
    let die_at = std::env::var("BF_TEST_DIE").ok().map(|v| v.parse::<f32>().unwrap_or(1.0));
    if die_at.is_some_and(|at| player.sim_time > at) && !player.dead && !test_died.0 {
        test_died.0 = true;
        let (at, side) = (player.position + Vec3::Y, Vec3::new(player.yaw.cos(), 0.0, -player.yaw.sin()));
        let all = player.health;
        hurt(&mut player, &game.0, all, HURT_CHATTER, side * 5.0 + Vec3::Y * 2.0, at, -1);
    }
    // dead: the death camera (play_deathcam) picks the next living member near its end
    let Some((c, left)) = player.select else { return };
    let Some(i) = squad.0.iter().position(|m| m.character == c && m.loaded.is_some() && !m.dead) else {
        player.select = None;
        return;
    };
    if left - dt > 0.0 {
        player.select = Some((c, left - dt));
        return;
    }
    let (pitch, distance, help) = (player.cam_pitch, player.cam_distance, player.show_help);
    // the squad shares one inventory: the grenades and medkits go with control
    let inventory = (player.grenades.clone(), player.medkits, player.medkit_kind, player.item, player.item_new);
    std::mem::swap(&mut *player, &mut squad.0[i]);
    (player.grenades, player.medkits, player.medkit_kind, player.item, player.item_new) = inventory;
    // the switch sound, at the cut (a capture: the same sound at every hand-over)
    player.sound_queue.push((SQUAD_SWITCH_SOUND, 0.8));
    squad.0[i].select = None;
    player.select = None;
    player.cam_pitch = pitch;
    player.cam_distance = distance;
    player.cam_target = player.position + Vec3::new(0.0, 0.35 + player.height * 0.5, 0.0);
    // the cut lands behind them, clear of walls (the reference recording's cut to Hawk: from
    // straight behind, the whole body in view)
    player.cam_yaw = clear_follow_yaw(player.cam_target, player.yaw, pitch, distance);
    player.shoulder = 0.0;
    // the squad AI's aim, fire, walking and idling don't carry over: the cut is to the plain
    // follow camera (not pulled in over the shoulder), and they stand where they face instead
    // of walking on, turning to the old leader's heading (face_yaw) or kneeling on (the demo's
    // choice: the recording's Hawk stands at the cut, but whether a kneeling member stands up is
    // a guess)
    player.move_input = Vec2::ZERO;
    player.aim = false;
    player.fire = false;
    player.aim_hold = 0.0;
    player.face_yaw = None;
    player.crouch_wanted = false;
    player.idle_cautious = false;
    player.show_help = help;
    player.weapon_dirty = true;
    player.hud_list = 0.0;
    for m in squad.0.iter_mut() {
        m.move_input = Vec2::ZERO;
        m.aim = false;
        m.fire = false;
    }
    // the character handed over from drops back with a follow line (unless they're dead)
    let left_behind = &mut squad.0[i];
    if !left_behind.dead {
        say(left_behind, &game.0, FOLLOW_CHATTER, QUOTE_DELAY);
    }
}

#[derive(Resource, Default)]
struct SoundCache(HashMap<u32, Option<Handle<AudioSource>>>);

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct MainCamera;


#[derive(Component)]
struct Tracer {
    dir: Vec3,
    origin: Vec3,
    travelled: f32,
    dist: f32,
    speed: f32,
    len: f32,
    hit: bool,
}

#[derive(Component)]
struct Impact(f32);

/// Shared meshes / materials for shot effects.
#[derive(Resource)]
struct Fx {
    tracer: Handle<Mesh>,
    tracer_mat: Handle<StandardMaterial>,
    spark: Handle<Mesh>,
    spark_mat: Handle<StandardMaterial>,
}

/// Pillars of the test ground (centres); they stop shots.
fn pillars() -> impl Iterator<Item = Vec3> {
    (-3..=3).flat_map(|i| (-3..=3).map(move |j| (i, j))).filter(|_| world::arena().is_none())
        .filter(|&(i, j): &(i32, i32)| (i + j) % 2 == 0 && (i, j) != (0, 0))
        .map(|(i, j)| Vec3::new(i as f32 * 9.0, GROUND + 1.5, j as f32 * 9.0))
}

/// Distance along a ray to the ground or a pillar, if within `max`.
fn ray_hit(origin: Vec3, dir: Vec3, max: f32) -> Option<f32> {
    if let Some(a) = world::arena() {
        return a.ray(origin, dir, max);
    }
    let mut best = if dir.y < -1e-4 { Some((GROUND - origin.y) / dir.y).filter(|t| *t > 0.0) } else { None };
    for c in pillars() {
        let (lo, hi) = (c - Vec3::new(0.3, 1.5, 0.3), c + Vec3::new(0.3, 1.5, 0.3));
        let (mut t0, mut t1) = (0.0f32, f32::MAX);
        let mut ok = true;
        for k in 0..3 {
            if dir[k].abs() < 1e-6 {
                ok &= origin[k] >= lo[k] && origin[k] <= hi[k];
            } else {
                let (a, b) = ((lo[k] - origin[k]) / dir[k], (hi[k] - origin[k]) / dir[k]);
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
        }
        if ok && t0 <= t1 && t0 > 0.0 && best.is_none_or(|b| t0 < b) {
            best = Some(t0);
        }
    }
    best.filter(|t| *t <= max)
}

/// Camera position and the point it looks at, from the player's camera state. Aiming and firing
/// dolly the camera in along its view (as the game does: the background and crosshair stay put,
/// the character grows ~1.8x, a little right of centre). Looking steeply down the
/// game keeps its distance, so the zoom fades out below ~35 degrees of pitch.
fn camera_pose(p: &Player, view_yaw: f32) -> (Vec3, Vec3) {
    let rot = Quat::from_euler(EulerRot::YXZ, p.cam_yaw + view_yaw + p.sway.x, p.cam_pitch + p.sway.y, 0.0);
    let s = p.shoulder * ((p.cam_pitch + 1.05) / 0.45).clamp(0.0, 1.0);
    let target = p.cam_target + rot * Vec3::new(0.2, 0.3, 0.0) * s;
    let pos = target + rot * Vec3::new(0.0, 0.0, p.cam_distance * (1.0 - 0.42 * s));
    if p.scope <= 0.0 {
        return (pos, target);
    }
    // the scope: the camera moves to the eyes (offset-snipe 0.001 0 0) and looks along the aim
    let head = p.cam_target + Vec3::Y * EYE_ABOVE_TARGET;
    let fwd = rot * Vec3::NEG_Z;
    // not into a wall: no further forward than the clearance before what's in front
    let reach = world::arena().and_then(|a| a.ray(head, fwd, EYE_FORWARD + EYE_CLEARANCE))
        .map_or(EYE_FORWARD, |t| (t - EYE_CLEARANCE).clamp(-EYE_CLEARANCE, EYE_FORWARD));
    let eye = head + fwd * reach;
    let ahead = eye + fwd * 10.0;
    (pos.lerp(eye, p.scope), target.lerp(ahead, p.scope))
}

/// Headings tried either side of straight behind when the cut to a squad member finds a wall
/// there: this step, up to half a turn (the demo's choice; the game's own placement isn't
/// recorded near walls).
const HANDOVER_YAW_STEP: f32 = 10.0 * std::f32::consts::PI / 180.0;
/// The follow camera keeps this far in front of a wall (its clamp in `follow_camera`).
const FOLLOW_WALL_MARGIN: f32 = 0.3;

/// The follow camera's heading for a cut to a character facing `facing`, its view on `target`:
/// straight behind them if nothing is in the way back to the camera at `distance`; otherwise
/// the nearest heading either side that is clear (the follow camera's clamp would otherwise
/// pull it into the body); with none clear, the one with the most room.
fn clear_follow_yaw(target: Vec3, facing: f32, pitch: f32, distance: f32) -> f32 {
    let Some(a) = world::arena() else { return facing };
    let need = distance + FOLLOW_WALL_MARGIN;
    let room = |yaw: f32| {
        let back = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z;
        a.ray(target, back, need).unwrap_or(f32::MAX)
    };
    let mut best = (facing, room(facing));
    if best.1 >= need {
        return facing;
    }
    for k in 1..=18 {
        for side in [1.0, -1.0] {
            let yaw = wrap_angle(facing + side * k as f32 * HANDOVER_YAW_STEP);
            let r = room(yaw);
            if r >= need {
                return yaw;
            }
            if r > best.1 {
                best = (yaw, r);
            }
        }
    }
    best.0
}

/// The vertical field of view at a zoom.
fn fov_at(zoom: f32) -> f32 {
    2.0 * ((FOV_Y * 0.5).tan() / zoom.max(1.0)).atan()
}

/// Camera position and direction of the ray through the crosshair.
fn aim_ray(p: &Player) -> (Vec3, Vec3) {
    let (cam, look) = camera_pose(p, 0.0);
    let fwd = (look - cam).normalize();
    let right = fwd.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(fwd);
    // (in the scope the crosshair is in the middle)
    (cam, (fwd + up * CROSSHAIR_UP * (1.0 - p.scope) * (fov_at(p.zoom) * 0.5).tan()).normalize())
}

/// The flat test floor: checkered ground, pillars, camera and lights (BF_MAP=flat).
fn test_floor(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>,
              images: &mut Assets<Image>) {
    commands.spawn((MainCamera, Camera3d::default(), Transform::from_xyz(0.0, 1.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
                    bf_viewer::level_scene::console_look(),
                    DistanceFog { color: Color::srgb(0.47, 0.55, 0.66), falloff: FogFalloff::Linear { start: 25.0, end: 90.0 }, ..default() }));
    // a key and a fill light in the levels' manner (see level_scene::spawn_lighting)
    commands.spawn((
        DirectionalLight { illuminance: std::f32::consts::PI, color: Color::srgb(0.85, 0.82, 0.76), shadows_enabled: true, ..default() },
        Transform::from_xyz(4.0, 8.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight { illuminance: std::f32::consts::PI, color: Color::srgb(0.2, 0.24, 0.3), ..default() },
        Transform::from_xyz(-3.0, 2.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight { color: Color::srgb(0.2, 0.22, 0.26), brightness: 1.0, ..default() });

    // checkered ground so movement is easy to read
    let n = 64u32;
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let c: u8 = if ((x / 32) + (y / 32)) % 2 == 0 { 120 } else { 96 };
            let c = if x % 32 == 0 || y % 32 == 0 { 70 } else { c };
            px.extend_from_slice(&[c, c + 6, c + 4, 255]);
        }
    }
    let mut img = Image::new(bevy::render::render_resource::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
                             bevy::render::render_resource::TextureDimension::D2, px,
                             bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                             bevy::asset::RenderAssetUsages::default());
    img.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::Repeat,
        address_mode_v: bevy::image::ImageAddressMode::Repeat,
        ..bevy::image::ImageSamplerDescriptor::linear()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(img)),
            uv_transform: Affine2::from_scale(Vec2::splat(100.0)),   // 2 m squares
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, GROUND, 0.0),
    ));
    let pillar = meshes.add(Cuboid::new(0.6, 3.0, 0.6));
    let stone = materials.add(StandardMaterial { base_color: Color::srgb(0.55, 0.5, 0.45), perceptual_roughness: 0.9, ..default() });
    for i in -3..=3 {
        for j in -3..=3 {
            if (i + j) % 2 == 0 && (i, j) != (0, 0) {
                commands.spawn((Mesh3d(pillar.clone()), MeshMaterial3d(stone.clone()),
                                Transform::from_xyz(i as f32 * 9.0, GROUND + 1.5, j as f32 * 9.0)));
            }
        }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
         mut images: ResMut<Assets<Image>>, mut game: ResMut<GameData>, map: Res<MapLevel>,
         mut sources: ResMut<Assets<AudioSource>>, mode: Res<Mode>, current: Res<CurrentMap>,
         mut level_materials: ResMut<Assets<bf_viewer::level_scene::LevelMaterial>>,
         mut liquids: ResMut<Assets<bf_viewer::level_scene::LiquidMaterial>>,
         mut buffers: ResMut<Assets<bevy::render::storage::ShaderStorageBuffer>>) {
    let mut ale = bf_viewer::ale_fx::AleAssets::new(&mut meshes);
    info!("ALE library: {} effects, {} nodes", game.0.effects.effects.len(), game.0.effects.nodes.len());
    // the slide dust, ready before anyone slides
    if let Some(&fx) = game.0.effect_types.get(&SLIDE_EFFECT).and_then(|l| l.first()) {
        ale.load(&mut game.0, &mut images, &mut materials, fx);
    }
    if let Some(level) = &map.0 {
        // the map: its terrain, objects and sky, fog, background and light
        let doors = bf_viewer::level_scene::spawn_level(&mut commands, &mut game.0, level, &mut meshes, &mut level_materials, &mut liquids, &mut buffers, &mut images);
        for d in doors.iter().filter(|_| std::env::var("BF_LEVEL_DUMP").is_ok()) {
            println!("door at {:.1}: {:.2} s, leaves slide {:?} m, sounds {:x?} / {:x?}", d.centre, d.duration,
                     d.leaves.iter().map(|l| l.slide(d.duration, d.duration)).collect::<Vec<_>>(), d.open_sound, d.close_sound);
        }
        // the gates' wall panels: a button whose signal reaches the door's anim-triggers
        // (where each one's button is: its archetype's PANEL_BUTTON hardpoint, else 1.4 m up)
        let opens: Vec<((Vec3, Vec3), Vec<(u32, u32, i64)>)> = level.buttons.iter().map(|b| {
            let button = game.0.object_meshes.get(&b.kind).and_then(|&a| bf_viewer::bf::weapon::WeaponModel::load(&game.0, a).ok())
                .and_then(|m| m.hardpoints.get(&PANEL_BUTTON).map(|h| h.point)).unwrap_or(Vec3::Y * 1.4);
            // (its front: the button quad's normal, the model's +z)
            let facing = b.transform.transform_vector3(Vec3::Z).normalize_or_zero();
            ((b.transform.transform_point3(button), facing), level.button_opens(b))
        }).collect();
        let doors: Vec<DoorState> = doors.into_iter().map(|door| {
            let panels = opens.iter().filter(|(_, o)| o.iter().any(|x| x.0 == door.name)).map(|(at, _)| *at).collect();
            DoorState { door, t: 0.0, opening: false, panels, latched: false }
        }).collect();
        if std::env::var("BF_DOOR_LOG").is_ok() {
            for (b, (at, o)) in level.buttons.iter().zip(&opens) {
                println!("panel h_{:08x} button at {:.1} facing {:.2} signal {}: opens {:x?}", b.name, at.0, at.1, b.signal, o);
            }
            for d in doors.iter().filter(|d| !d.panels.is_empty()) {
                println!("gate h_{:08x} at {:.1}: {} panels, {} leaves", d.door.name, d.door.centre, d.panels.len(), d.door.leaves.len());
            }
        }
        commands.insert_resource(Doors(doors));
        commands.insert_resource(UsePanel::default());
        // the power-ups' icons (their idle effects)
        let n = if std::env::var("BF_NO_IDLE_FX").is_ok() { 0 } else {
            bf_viewer::ale_fx::spawn_idle_effects(&mut commands, &mut game.0, level, &mut ale, &mut images, &mut materials)
        };
        println!("{n} idle effects (power-up icons)");
        let fog = level.fog.map(|(c, start, end)| DistanceFog { color: Color::srgb(c[0], c[1], c[2]), falloff: FogFalloff::Linear { start, end }, ..default() });
        let mut cam = commands.spawn((MainCamera, Camera3d::default(), Projection::Perspective(PerspectiveProjection { far: 5000.0, ..default() }),
                                      Transform::default(), bf_viewer::level_scene::console_look()));
        if let Some(f) = fog {
            cam.insert(f);
        }
        let bg = level.background;
        commands.insert_resource(ClearColor(Color::srgb(bg[0], bg[1], bg[2])));
        // the level's own lights (key, fill, ambient) as the console used them
        bf_viewer::level_scene::spawn_lighting(&mut commands, level);
    } else {
        test_floor(&mut commands, &mut meshes, &mut materials, &mut images);
    }
    commands.insert_resource(ale);
    // the level's music, looping (BF_MUTE / BF_NO_MUSIC: none; BF_MUSIC_VOLUME, default 0.5);
    // deathmatch plays without music
    if map.0.is_some() && !mode.deathmatch && std::env::var("BF_MUTE").is_err() && std::env::var("BF_NO_MUSIC").is_err() {
        let name = current.0.clone();
        match Game::load_music(&data_dir(), &name) {
            Ok(tracks) => {
                let volume = std::env::var("BF_MUSIC_VOLUME").ok().and_then(|v| v.parse().ok()).unwrap_or(MUSIC_VOLUME);
                // a level's bank holds its music and its ambience bed (sdm_e13: amb_singe_02, then
                // tmp_full-on1a-f): the first of each, by the sounds' Types (names the banks don't
                // define: an "amb" prefix is ambience); the bed plays under the music
                let ambient = |n: &str| match game.0.sounds.sound_type(bf_viewer::bf::hash::h(n)) {
                    Some(t) => t != bf_viewer::bf::audio::MUSIC_TYPE,
                    None => n.to_ascii_lowercase().starts_with("amb"),
                };
                // (a bank without music, sdm_m07's: all its beds, ambiencehellish-2ch its score)
                let (beds, music): (Vec<_>, Vec<_>) = tracks.into_iter().partition(|(n, _)| ambient(n));
                let bed_count = if music.is_empty() { beds.len() } else { 1 };
                for ((track, wav), (label, k)) in music.into_iter().take(1).map(|t| (t, ("music", 1.0)))
                    .chain(beds.into_iter().take(bed_count).map(|t| (t, ("ambience", AMBIENCE_VOLUME)))) {
                    println!("{label}: {track}");
                    let source = sources.add(AudioSource { bytes: Arc::from(wav.into_boxed_slice()) });
                    commands.spawn((AudioPlayer::new(source), PlaybackSettings { volume: Volume::Linear(volume * k), ..PlaybackSettings::LOOP },
                                    Name::new(label)));
                }
            }
            Err(e) => eprintln!("no music for {name}: {e}"),
        }
    }
    commands.insert_resource(Fx {
        tracer: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        tracer_mat: materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.85, 0.4), emissive: LinearRgba::rgb(12.0, 8.0, 2.5), unlit: true, ..default() }),
        spark: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
        spark_mat: materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.7, 0.3), emissive: LinearRgba::rgb(8.0, 4.0, 1.0), unlit: true, ..default() }),
    });
    // debug / controls text (H), bottom centre, clear of the game HUD
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        TextLayout::new_with_justify(JustifyText::Center),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(6.0), left: Val::Px(0.0), right: Val::Px(0.0), ..default() },
    ));
}

/// Locomotion and jump clips, preferring the game's own names.
fn pick_clips(model: &Character, game: &Game) -> Clips {
    let mut loco = locomotion::pick(&locomotion::stats(model, game));
    let named = |n: &str| model.anim_by_name(n);
    loco.idle = named("Sc_w1_idle_general").or(loco.idle);
    loco.walk = named("Sc_w1_walk").or(loco.walk);
    loco.run = named("Sc_w1_run").or(loco.run);
    loco.walk_back = named("Sc_w1_back_walk").or(loco.walk_back);
    loco.run_back = named("Sc_w1_back_run").or(loco.run_back);
    // a slot's set, by the motion scripts' names (prefix + motion), falling back to the first
    // set; Brutus's sprint stays the four-legged run picked from the data
    let four_legged = named("Sc_4leg_w1_jump_launch").is_some();
    let slot = |pre: &str| {
        let n = |m: &str| named(&format!("{pre}{m}"));
        SlotClips {
            carry: Locomotion {
                idle: n("idle_general").or(loco.idle),
                walk: n("walk").or(loco.walk),
                run: n("run").or(loco.run),
                sprint: if four_legged { loco.sprint } else { n("dash").or(loco.sprint) },
                walk_back: n("back_walk").or(loco.walk_back),
                run_back: n("back_run").or(loco.run_back),
                dodge_left: loco.dodge_left,
                dodge_right: loco.dodge_right,
            },
            rp: Locomotion {
                idle: n("rp_idle"),
                walk: n("rp_walk"),
                run: n("rp_run"),
                walk_back: n("rp_back_walk"),
                run_back: n("rp_back_run"),
                ..default()
            },
            stand2crouch: n("stand2crouch"),
            crouch_idle: n("rp_crouch_idle").or(n("crouch_idle")),
            crouch2stand: n("crouch2stand"),
            cr_walk: n("cr_walk"),
            cr_back_walk: n("cr_back_walk"),
            rp_side: ["rp_f_side_walk_l", "rp_f_side_walk_r", "rp_b_side_walk_l", "rp_b_side_walk_r"].map(n),
            cr_side: ["cr_f_side_walk_l", "cr_f_side_walk_r", "cr_b_side_walk_l", "cr_b_side_walk_r"].map(n),
            dive: n("dive"),
        }
    };
    let slots = [slot("Sc_w1_"), slot("Sc_w2_")];
    Clips {
        slots,
        loco,
        jump_crouch: named("Sc_w1_jump_crouch"),
        jump_launch: named("Sc_w1_jump_launch"),
        jump_fall: named("Sc_w1_jump_fall"),
        jump_land: named("Sc_w1_jump_land"),
        leg4_launch: named("Sc_4leg_w1_jump_launch"),
        leg4_fall: named("Sc_4leg_w1_jump_fall"),
        leg4_land: named("Sc_4leg_w1_jump_land"),
        slide: named("Sc_w1_slide_idle"),
    }
}

/// The lowest bone on each side (toes / feet) and how high it sits when planted.
fn find_feet(model: &Character, game: &Game, idle: Option<usize>) -> ([usize; 2], [f32; 2]) {
    let bind = model.world(&model.bind_local());
    let lowest = |side: f32| -> usize {
        (0..bind.len()).filter(|&i| bind[i].w_axis.x * side > 0.02)
            .min_by(|&a, &b| bind[a].w_axis.y.total_cmp(&bind[b].w_axis.y)).unwrap_or(0)
    };
    let feet = [lowest(-1.0), lowest(1.0)];
    let pose = model.world(&match idle {
        Some(i) => model.pose(game, i, 0.0, None, 0.0),
        None => model.bind_local(),
    });
    (feet, [pose[feet[0]].w_axis.y, pose[feet[1]].w_axis.y])
}

/// How far to raise a character so the soles of its idle pose stand on the floor (GROUND under
/// the root): GROUND minus its lowest skinned vertex in that pose (each vertex skinned by its
/// joints' idle world matrices and inverse binds, as the GPU does). GROUND is one height for
/// everyone; Tex's idle soles are about 0.14 m lower, so his feet went through the floor.
/// Limited to MAX_SOLE_LIFT either way, in case a model's lowest vertex isn't a sole.
fn sole_lift(model: &Character, game: &Game, idle: Option<usize>) -> f32 {
    let world = model.world(&match idle {
        Some(i) => model.pose(game, i, 0.0, None, 0.0),
        None => model.bind_local(),
    });
    let lowest = lowest_vertex(model, &world);
    if lowest == f32::MAX { 0.0 } else { (GROUND - lowest).clamp(-MAX_SOLE_LIFT, MAX_SOLE_LIFT) }
}

/// A clip's lift through its length, like `sole_lift` (GROUND minus the pose's lowest skinned
/// vertex), at LIFT_SAMPLES even times. A loop gets one value, the most (its lowest point over
/// the cycle on the floor: a walk always has a foot down, a kneel its knee). Without these, a
/// kneel was lifted by the standing soles' amount: Hawk and Flint knelt in the air.
fn clip_lift(model: &Character, game: &Game, clip: usize, looping: bool) -> Vec<f32> {
    let d = model.anims[clip].duration.max(1e-3);
    let lifts: Vec<f32> = (0..LIFT_SAMPLES).map(|k| {
        let mut pose = model.pose(game, clip, d * k as f32 / (LIFT_SAMPLES - 1) as f32, None, 0.0);
        pose[0].1.x = 0.0;
        pose[0].1.z = 0.0;
        let lowest = lowest_vertex(model, &model.world(&pose));
        if lowest == f32::MAX { 0.0 } else { (GROUND - lowest).clamp(-MAX_CLIP_LIFT, MAX_CLIP_LIFT) }
    }).collect();
    if looping { vec![lifts.iter().copied().fold(f32::MIN, f32::max)] } else { lifts }
}

/// `clip_lift`'s samples, and the most a clip's lift may move a character (m): a kneel drops the
/// body further than a standing correction.
const LIFT_SAMPLES: usize = 16;
const MAX_CLIP_LIFT: f32 = 0.6;

/// The lowest skinned vertex (model space) in a pose given as world matrices, f32::MAX if none.
fn lowest_vertex(model: &Character, world: &[Mat4]) -> f32 {
    let skin: Vec<Mat4> = world.iter().zip(&model.inverse_bind).map(|(w, ib)| *w * *ib).collect();
    let mut lowest = f32::MAX;
    for g in &model.geosets {
        for (k, v) in g.positions.iter().enumerate() {
            let (j, w) = (g.joints.get(k), g.weights.get(k));
            let at = match (j, w) {
                (Some(j), Some(w)) => (0..4).filter(|&n| w[n] > 0.0)
                    .filter_map(|n| skin.get(j[n] as usize).map(|m| m.transform_point3(Vec3::from(*v)) * w[n]))
                    .sum::<Vec3>(),
                _ => Vec3::from(*v),
            };
            lowest = lowest.min(at.y);
        }
    }
    lowest
}

/// The most `sole_lift` moves a character (m).
const MAX_SOLE_LIFT: f32 = 0.3;

/// Each foot's height range over the locomotion clips. A run is carried higher than the idle
/// stance (the toes never come back down to it), so contact is judged per clip.
fn foot_ranges(model: &Character, game: &Game, clips: &Clips, feet: [usize; 2]) -> HashMap<usize, [(f32, f32); 2]> {
    let lo = &clips.loco;
    let mut out = HashMap::new();
    let sets = clips.slots.iter().flat_map(|s| [&s.carry, &s.rp]);
    for c in [lo.walk, lo.run, lo.sprint, lo.walk_back, lo.run_back, lo.dodge_left, lo.dodge_right].into_iter()
        .chain(sets.flat_map(|l| [l.walk, l.run, l.sprint, l.walk_back, l.run_back]))
        .chain(clips.slots.iter().flat_map(|s| [s.cr_walk, s.cr_back_walk]))
        .chain(clips.slots.iter().flat_map(|s| s.rp_side.into_iter().chain(s.cr_side))).flatten() {
        let d = model.anims[c].duration;
        let mut r = [(f32::MAX, f32::MIN); 2];
        for k in 0..24 {
            let w = model.world(&model.pose(game, c, d * k as f32 / 24.0, None, 0.0));
            for f in 0..2 {
                let y = w[feet[f]].w_axis.y;
                r[f] = (r[f].0.min(y), r[f].1.max(y));
            }
        }
        out.insert(c, r);
    }
    out
}

/// The stance a weapon is held in: the motion set (Sc_w1_* / Sc_w2_*) of the slot whose class
/// (the character's limit-weapon) matches the weapon's class, so Brutus holds his medium Bower 20
/// in his medium stance though it's his second weapon; by its slot when both or neither match.
fn stance_for(classes: Option<[i64; 2]>, class: i64, slot: usize) -> usize {
    match classes {
        Some([a, b]) if a != b && class == a => 0,
        Some([a, b]) if a != b && class == b => 1,
        _ => slot.min(1),
    }
}

/// The held weapon's stance (0 when unarmed).
fn stance_of(l: &Loaded, weapon: usize) -> usize {
    l.weapons.get(weapon).map_or(0, |w| w.stance)
}

const EV_DROP_WEAPON: u32 = 0xE2A3_FFF9;
/// events where the hand takes the new gun: to slot 2, to slot 1, Tex's weapon change
const EV_GRAB: [u32; 3] = [0xE141_F815, 0xFA48_A9AF, 0xFBC0_1DF5];

/// The overlay that switches to weapon slot `to`, by its motion-script name: Flint and Hawk
/// Sc_w1_2_w2 / Sc_w2_2_w1, Brutus Sc_w1_2_w2rifle|cannon / Sc_w2rifle|cannon_2_w1, Tex
/// Sc_wc_<from>2<to> (rifle / cannon by weapon type). Drop and grab times come from the clip's
/// events (drop_weapon, then the grab event); in the original clips the trigger hand is then at
/// the old and the new weapon's stow point.
fn switch_clip(model: &Character, game: &Game, weapons: &[HeldWeapon], to: usize) -> Option<SwitchClip> {
    let class = |i: usize| if weapons.get(i).is_some_and(|w| w.def.weapon_type == 2) { "cannon" } else { "rifle" };
    let names = if to == 1 {
        [format!("Sc_w1_2_w2"), format!("Sc_w1_2_w2{}", class(1)), format!("Sc_wc_{}2{}", class(0), class(1))]
    } else {
        [format!("Sc_w2_2_w1"), format!("Sc_w2{}_2_w1", class(1)), format!("Sc_wc_{}2{}", class(1), class(0))]
    };
    let clip = names.iter().find_map(|n| model.anim_by_name(n))?;
    let a = &model.anims[clip];
    let ev = game.events(a.event_channel);
    let drop = ev.iter().find(|e| e.1 == EV_DROP_WEAPON).map(|e| e.0).unwrap_or(a.duration * 0.3);
    let grab = ev.iter().find(|e| EV_GRAB.contains(&e.1) && e.0 > drop).map(|e| e.0).unwrap_or(a.duration * 0.65);
    let mut mask = vec![false; model.bones.len()];
    for (bone, _) in &a.targets {
        if let Some(i) = model.bones.iter().position(|b| b == bone) {
            mask[i] = model.parent[i].is_some();
        }
    }
    Some(SwitchClip { clip, duration: a.duration, drop, grab, mask })
}

/// A reload / throw clip by name with its events, and its bone masks (all animated bones; and
/// without the legs and pelvis: everything from the feet up to where the two legs meet).
fn overlay_clip(model: &Character, game: &Game, name: &str, feet: [usize; 2]) -> Option<OverlayClip> {
    let clip = model.anim_by_name(name)?;
    let a = &model.anims[clip];
    let ancestors = |mut b: usize| { let mut v = vec![b]; while let Some(p) = model.parent[b] { v.push(p); b = p; } v };
    let (l, r) = (ancestors(feet[0]), ancestors(feet[1]));
    let hips: Vec<usize> = l.iter().copied().filter(|b| r.contains(b)).collect();
    let legs: Vec<usize> = l.iter().chain(&r).copied().filter(|b| !hips.contains(b)).collect();
    let mut mask = vec![false; model.bones.len()];
    let mut upper = vec![false; model.bones.len()];
    for (bone, _) in &a.targets {
        if let Some(i) = model.bones.iter().position(|b| b == bone) {
            mask[i] = model.parent[i].is_some();
            upper[i] = mask[i] && !hips.contains(&i) && !ancestors(i).iter().any(|b| legs.contains(b));
        }
    }
    Some(OverlayClip { clip, duration: a.duration, events: game.events(a.event_channel), mask, upper })
}

/// Blend an overlay clip at time `t` onto `pose`, fading in and out over FADE.
fn apply_overlay(pose: &mut [(Quat, Vec3)], model: &Character, game: &Game, c: &OverlayClip, t: f32, full: bool, face_time: f32) {
    let t = t.min(c.duration - 1e-3);
    let k = (t / FADE).min((c.duration - t) / FADE).clamp(0.0, 1.0);
    let over = model.pose(game, c.clip, t, model.default_face, face_time);
    for (b, m) in (if full { &c.mask } else { &c.upper }).iter().enumerate() {
        if *m {
            let (q0, t0) = pose[b];
            let q1 = if q0.dot(over[b].0) < 0.0 { -over[b].0 } else { over[b].0 };
            pose[b] = (q0.lerp(q1, k).normalize(), t0.lerp(over[b].1, k));
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    mut game: ResMut<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    for p in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if p.loaded.as_ref().is_some_and(|l| l.index == p.character) {
            continue;
        }
        if let Some(old) = p.loaded.take() {
            commands.entity(old.root).despawn();
        }
        spawn_unit(&mut commands, p, &mut game.0, &mut assets);
    }
}

/// Load a character and spawn it with its weapons and clips into `p`.
fn spawn_unit(commands: &mut Commands, p: &mut Player, game: &mut Game, assets: &mut ModelAssets) {
    let name = CHARACTERS[p.character];
    let model = match Character::load(game, name, 0) {
        Ok(m) => m,
        Err(e) => {
            error!("{name}: {e}");
            return;
        }
    };
    let clips = pick_clips(&model, game);
    info!("{name} clips: {clips:?}");
    let (feet, foot_rest) = find_feet(&model, game, clips.loco.idle);
    let (footstep_type, jump_sound) = game.character_audio.get(name).copied().unwrap_or((-1, 0));
    let root = commands.spawn((Transform::from_translation(p.position), Visibility::default(), Name::new(name))).id();
    let joints = spawn_model(commands, game, &model, assets, root, true);
    p.layers.clear();
    p.gait = Gait::Idle;
    p.action = Action::None;
    p.height = 0.0;
    let index = p.character;
    let aim_chain = locomotion::aim_chain(&model);
    // pick a surface that has footsteps for this character's footstep type
    let has = |s: &bf_viewer::bf::audio::Surface| s.footsteps.get(&footstep_type).is_some_and(|v| v.iter().any(|&i| game.sounds.has(i)));
    if p.surface >= game.surfaces.len() || !has(&game.surfaces[p.surface]) {
        p.surface = game.surfaces.iter().position(has).unwrap_or(0);
    }
    let foot_range = foot_ranges(&model, game, &clips, feet);

    // weapons: the starting inventory, joined to the hand / back hardpoints
    let model_bones = model.bones.clone();
    let arm_chain: Vec<usize> = match model.hardpoints.get(&weapon::TRIGGER_HAND) {
        Some(&(hand, _)) => {
            let ancestors = |mut b: usize| { let mut v = vec![]; while let Some(p) = model.parent[b] { v.push(p); b = p; } v };
            let (arm, legs) = (ancestors(hand), [ancestors(feet[0]), ancestors(feet[1])].concat());
            aim_chain.iter().copied().filter(|b| arm.contains(b) && !legs.contains(b)).collect()
        }
        None => vec![],
    };
    let flash_mesh = assets.meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap());
    let flash_mat = assets.materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.8, 0.4),
        emissive: LinearRgba::rgb(30.0, 18.0, 5.0), unlit: true, ..default() });
    let mut weapons = vec![];
    for (slot, w) in game.character_weapons.get(name).cloned().unwrap_or_default().into_iter().enumerate() {
        let Some(def) = game.weapons.get(&w).cloned() else { continue };
        let wm = match WeaponModel::load(game, def.archetype) {
            Ok(m) => m,
            Err(e) => {
                warn!("{}: {e}", def.label);
                continue;
            }
        };
        let entity = commands.spawn((Transform::default(), Visibility::Hidden, Name::new(def.label.clone()), ChildOf(root))).id();
        let mut spinners = vec![];
        for part in &wm.parts {
            let pe = commands.spawn((Transform::from_translation(part.offset).with_rotation(part.rotation), Visibility::Inherited, ChildOf(entity))).id();
            spawn_static(commands, game, &part.geosets, assets, pe, true);
            if let Some(axis) = part.spin_axis {
                spinners.push((pe, axis, part.offset));
            }
        }
        let muzzle = wm.hardpoints.get(&def.muzzle).copied().unwrap_or(Hardpoint { point: Vec3::ZERO, rot: Quat::IDENTITY });
        let fire_dir = muzzle.rot * Vec3::NEG_Z;
        let flash = commands.spawn((
            Mesh3d(flash_mesh.clone()), MeshMaterial3d(flash_mat.clone()), NotShadowCaster,
            Transform::from_translation(muzzle.point + fire_dir * 0.07).looking_to(fire_dir, Vec3::Y)
                .with_scale(Vec3::new(0.045, 0.045, 0.11)),
            Visibility::Hidden, ChildOf(entity))).id();
        let light = commands.spawn((
            PointLight { intensity: 0.0, range: 6.0, color: Color::srgb(1.0, 0.75, 0.4), ..default() },
            Transform::from_translation(muzzle.point + fire_dir * 0.1), ChildOf(entity))).id();
        let in_hand = model.hardpoints.get(&weapon::TRIGGER_HAND).zip(wm.hardpoints.get(&weapon::TRIGGER_GRIP))
            .map(|(&(bone, hand), grip)| { let (r, t) = hand.mount(grip); (bone, r, t) });
        let stance = stance_for(game.character_weapon_classes.get(name).copied(), def.weapon_type, slot);
        weapons.push(HeldWeapon {
            stance,
            in_hand,
            stowed: weapon::STOW.get(slot).and_then(|k| model.hardpoints.get(k)).zip(wm.hardpoints.get(&weapon::HOLSTER))
                .map(|(&(bone, stow), h)| { let (r, t) = stow.mount(h); (bone, r, t) }),
            def, entity, spinners, light, flash, muzzle, fire_dir,
        });
    }
    info!("{name} weapons: {:?}", weapons.iter().map(|w| format!("{} (class {}, stance w{}) held by bone {}", w.def.label, w.def.weapon_type,
        w.stance + 1, w.in_hand.map(|(b, ..)| format!("{:08x}", model_bones[b])).unwrap_or("-".into()))).collect::<Vec<_>>());
    for (k, set) in clips.slots.iter().enumerate() {
        info!("{name} slot {} clips: carry {:?} / ready {:?}", k + 1, set.carry, set.rp);
    }
    // test hook: BF_START_WEAPON=1 starts with the second weapon in hand
    p.weapon = std::env::var("BF_START_WEAPON").ok().and_then(|v| v.parse().ok()).unwrap_or(0).min(weapons.len().saturating_sub(1));
    p.weapon_dirty = true;
    p.holding = true;
    p.switching = None;
    p.reloading = None;
    // a full clip and ten more in reserve
    p.ammo = weapons.iter().map(|w| { let c = w.def.ammo.max(1); [c, c * 10] }).collect();
    let switch_clips = [switch_clip(&model, game, &weapons, 0), switch_clip(&model, game, &weapons, 1)];
    let reload_clips = ["Sc_w1_reload", "Sc_w2_reload"].map(|n| overlay_clip(&model, game, n, feet));
    let use_clips = ["Sc_w1_use_item", "Sc_w2_use_item"].map(|n| overlay_clip(&model, game, n, feet));
    let throw_clips = ["Sc_w1_throw_grenade", "Sc_w2_throw_grenade"].map(|n| overlay_clip(&model, game, n, feet));
    let place_clips = ["Sc_w1_place_hi", "Sc_w2_place_hi"].map(|n| overlay_clip(&model, game, n, feet));
    // the throwing hand: whichever hand moves further between reaching for the grenade and letting go
    let throw_hand = throw_clips[0].as_ref().and_then(|c| {
        let at = |t: f32| model.world(&model.pose(game, c.clip, t, model.default_face, 0.0));
        let (reach, release) = (c.event(EV_REACH).unwrap_or(0.2), c.event(EV_RELEASE).unwrap_or(c.duration * 0.55));
        let (a, b) = (at(reach), at(release));
        [weapon::TRIGGER_HAND, weapon::SUPPORT_HAND].iter().filter_map(|k| model.hardpoints.get(k))
            .map(|&(bone, hp)| (a[bone].transform_point3(hp.point).distance(b[bone].transform_point3(hp.point)), bone, hp.point))
            .max_by(|x, y| x.0.total_cmp(&y.0)).map(|(_, bone, point)| (bone, point))
    });
    info!("{name} reload {:?} / throw {:?} clips, throwing hand {:?}",
          reload_clips.iter().map(|c| c.as_ref().map(|c| c.duration)).collect::<Vec<_>>(),
          throw_clips.iter().map(|c| c.as_ref().map(|c| c.duration)).collect::<Vec<_>>(), throw_hand.map(|h| h.0));
    p.throwing = None;
    // (play_grenade.rs's stock_inventory fills it again: a respawn starts with the start stock)
    p.grenades.clear();
    p.max_health = game.character_hitpoints.get(name).copied().unwrap_or(100.0);
    p.health = p.max_health;
    p.leash = game.squad_leash.get(name).copied().unwrap_or(15.0);
    p.scope_sounds = game.character_scope_sounds.get(name).copied().unwrap_or((0, 0));
    p.snipe_sound = game.character_snipe_sound.get(name).and_then(|&id| game.sounds.pcm(id).map(|(r, pcm)| (id, pcm.len() as f32 / r as f32)));
    p.kneel_jitter = 1.5 * p.random(1000) as f32 / 1000.0;
    for (to, s) in switch_clips.iter().enumerate() {
        if let Some(s) = s {
            info!("{name} switch to weapon {}: clip {} ({:.2}s), drop {:.2}s, grab {:.2}s", to + 1, s.clip, s.duration, s.drop, s.grab);
        }
    }
    let lift = sole_lift(&model, game, clips.loco.idle);
    let mut crouch_lift = HashMap::new();
    for set in &clips.slots {
        let sides = set.cr_side.map(|c| (c, true));
        for (clip, looping) in [(set.stand2crouch, false), (set.crouch2stand, false), (set.crouch_idle, true), (set.cr_walk, true), (set.cr_back_walk, true)].into_iter().chain(sides) {
            if let Some(c) = clip {
                crouch_lift.entry(c).or_insert_with(|| clip_lift(&model, game, c, looping));
            }
        }
    }
    p.lift_now = lift;
    info!("{name}: soles {lift:.3} m below the floor, raised by that");
    p.loaded = Some(Loaded { index, root, joints, model, clips, aim_chain, arm_chain, feet, lift, weapon_dropped: false, crouch_lift, foot_rest, foot_range, footstep_type, jump_sound,
                                  weapons, switch_clips, reload_clips, use_clips, throw_clips, place_clips, throw_hand });
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut player: ResMut<Player>,
) {
    // 1-4: hand control to that squad member (after their name shows, see squad_control); not
    // while dead: the death camera hands over when it ends (whether the game lets you cut it
    // short isn't recorded: a guess)
    for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
        if keys.just_pressed(*k) && i != player.character && !player.dead {
            player.select = Some((i, SELECT_TIME));
        }
    }
    player.next_surface = keys.just_pressed(KeyCode::KeyM);
    // test hook: BF_TEST_USE=<s> holds use (E) for a second from that time
    player.use_held = std::env::var("BF_TEST_USE").ok().and_then(|v| v.parse::<f32>().ok())
        .is_some_and(|at| player.sim_time >= at && player.sim_time - at < 1.0);
    // test hook: BF_TEST_ITEM_LIST=1 holds the item list open
    if std::env::var("BF_TEST_ITEM_LIST").is_ok() {
        player.item_list = true;
    }
    if autopilot(&mut player) {
        return;
    }
    let mut captured = false;
    let mut was_captured = false;
    if let Ok(mut win) = windows.single_mut() {
        was_captured = win.cursor_options.grab_mode != CursorGrabMode::None;
        if mouse.just_pressed(MouseButton::Left) {
            win.cursor_options.grab_mode = CursorGrabMode::Locked;
            win.cursor_options.visible = false;
        }
        if keys.just_pressed(KeyCode::Escape) {
            win.cursor_options.grab_mode = CursorGrabMode::None;
            win.cursor_options.visible = true;
        }
        captured = win.cursor_options.grab_mode != CursorGrabMode::None;
    }
    let mut mv = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) { mv.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { mv.y -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { mv.x += 1.0; }
    if keys.pressed(KeyCode::KeyA) { mv.x -= 1.0; }
    // (finer in the scope: the turn is divided by the zoom)
    let look = if captured { motion.delta * 0.004 / player.zoom.max(1.0) } else { Vec2::ZERO };
    let sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let walk = keys.pressed(KeyCode::ControlLeft);
    // right mouse: aim while held; with a weapon that zooms it toggles the scope instead (Flint
    // steps in twice: in, closer, out)
    let zoomable = player.loaded.as_ref().and_then(|l| l.weapons.get(player.weapon)).is_some_and(|w| w.def.zoom > 1.0);
    if !zoomable {
        player.scope_level = 0;
    } else if was_captured && mouse.just_pressed(MouseButton::Right) {
        let steps = if player.character == STILL_SNIPER { 2 } else { 1 };
        player.scope_level = (player.scope_level + 1) % (steps + 1);
    }
    if let Some(l) = std::env::var("BF_TEST_SCOPE").ok().and_then(|v| v.parse().ok()) {
        player.scope_level = l;
    }
    let aim = if zoomable { player.scope_level > 0 } else { mouse.pressed(MouseButton::Right) };
    // test hook: BF_TEST_SELECT=<character> hands control over at 1 s
    if let Some(c) = std::env::var("BF_TEST_SELECT").ok().and_then(|v| v.parse::<usize>().ok()) {
        if player.sim_time > 1.0 && player.select.is_none() && c != player.character && !player.test_selected {
            player.select = Some((c, SELECT_TIME));
            player.test_selected = true;
        }
    }
    let jump = keys.just_pressed(KeyCode::Space);
    let dodge = keys.just_pressed(KeyCode::KeyC);
    // the click that captures the mouse doesn't fire
    let fire = was_captured && mouse.pressed(MouseButton::Left);
    let switch = keys.just_pressed(KeyCode::KeyQ);
    let reload = keys.just_pressed(KeyCode::KeyR);
    let use_key = keys.pressed(KeyCode::KeyE) || player.use_held;
    // the item box (the game's B button): Tab tapped steps to the next item carried; held, it
    // opens the item list, where the wheel picks one. G uses the item: a grenade is held to
    // charge and thrown on release, a medkit is used at once.
    let tab = keys.pressed(KeyCode::Tab);
    if tab {
        player.tab_down = if player.tab_down < 0.0 { 0.0 } else { player.tab_down + frame_dt(&time) };
        player.item_list = player.tab_down >= ITEM_LIST_HOLD;
        if player.item_list && scroll.delta.y != 0.0 {
            let step = if scroll.delta.y < 0.0 { 1 } else { -1 };
            step_item(&mut player, step);
        }
    } else {
        if (0.0..ITEM_LIST_HOLD).contains(&player.tab_down) {
            step_item(&mut player, 1);
        }
        player.tab_down = -1.0;
        player.item_list = false;
    }
    // T: the next grenade type carried (the demo's key)
    if keys.just_pressed(KeyCode::KeyT) {
        step_grenade(&mut player);
    }
    let grenade = matches!(player.item, Item::Grenade(_));
    let throw = grenade && keys.pressed(KeyCode::KeyG);
    player.item_use = !grenade && keys.just_pressed(KeyCode::KeyG);
    if keys.just_pressed(KeyCode::KeyH) {
        player.show_help = !player.show_help;
    }
    // Z: crouch (kneel standing, crouch walk on the move) or stand; jumping, dodging or
    // sprinting stands up
    if keys.just_pressed(KeyCode::KeyZ) {
        player.crouch_wanted = !player.crouch_wanted;
    }
    if jump || dodge || sprint {
        player.crouch_wanted = false;
    }
    player.move_input = if mv.length() > 1.0 { mv.normalize() } else { mv };
    player.sprint = sprint;
    player.walk = walk;
    // test hook: BF_TEST_AIM=1 holds aim
    player.aim = aim || std::env::var("BF_TEST_AIM").is_ok();
    // in the scope one can still move, slowly: a walk, no sprint
    if player.scope > 0.5 {
        player.sprint = false;
        player.walk = true;
    }
    player.jump_pressed = jump;
    player.dodge_pressed = dodge;
    player.fire = fire;
    player.switch_pressed = switch;
    player.reload_pressed = reload;
    player.use_held = use_key;
    player.throw_held = throw;
    player.cam_yaw -= look.x;
    player.cam_pitch = (player.cam_pitch - look.y).clamp(-1.2, 0.5);
    if scroll.delta.y != 0.0 && !player.item_list {
        player.cam_distance = (player.cam_distance * (1.0 - scroll.delta.y * 0.1)).clamp(1.5, 12.0);
    }
}

/// BF_AUTOPILOT=1: scripted input for automated captures. Returns true while it is driving.
fn autopilot(player: &mut Player) -> bool {
    // test hook: BF_TEST_GOTO=x,z,x2,z2 - start at (x, z) and run toward (x2, z2), the camera
    // behind (also starts there; see main); x2,z2 = x,z stands still; a 5th value is the start
    // height (the floor below it). BF_TEST_FIRE=1 (or 2: the second weapon) fires as well.
    if let Some(g) = test_goto() {
        let to = Vec2::new(g[2] - player.position.x, g[3] - player.position.z);
        player.cam_yaw = (-to.x).atan2(-to.y);
        // (BF_CAMERA_PITCH: another pitch, e.g. to fire down at something)
        player.cam_pitch = std::env::var("BF_CAMERA_PITCH").ok().and_then(|v| v.parse().ok()).unwrap_or(-0.2);
        player.aim = std::env::var("BF_TEST_AIM").is_ok();
        // a goal on the start itself: stand there (and look down -z)
        let stay = g[0] == g[2] && g[1] == g[3];
        if stay {
            player.cam_yaw = 0.0;
        }
        player.move_input = if player.sim_time > 1.0 && to.length() > 0.8 && !stay { Vec2::new(0.0, 1.0) } else { Vec2::ZERO };
        // BF_TEST_STRAFE=<x>[,<y>]: move that way instead (camera-relative: -1,0 left, 1,0
        // right, 1,-1 back right), from 1 s on; the camera keeps facing the goal
        if let Some(v) = std::env::var("BF_TEST_STRAFE").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<f32>>()) {
            if player.sim_time > 1.0 && !v.is_empty() {
                player.move_input = Vec2::new(v[0], v.get(1).copied().unwrap_or(0.0)).normalize_or_zero();
            }
        }
        // BF_TEST_FIRE=1: aim and fire (after a second); =2 with the second weapon
        if let Ok(w) = std::env::var("BF_TEST_FIRE") {
            player.aim = player.sim_time > 0.8;
            player.fire = player.sim_time > 1.0;
            player.switch_pressed = w == "2" && (0.3..0.4).contains(&player.sim_time);
        }
        player.sprint = false;
        // BF_TEST_CROUCH=<s>: crouch from that time on
        if let Some(at) = std::env::var("BF_TEST_CROUCH").ok().and_then(|v| v.parse::<f32>().ok()) {
            player.crouch_wanted = player.sim_time >= at;
        }
        // BF_TEST_THROW=<s>[,<hold s>][,<s>,<hold s>...]: hold the grenade button from s for hold
        // s (default 0.6: a full charge), as many times as given
        if let Ok(v) = std::env::var("BF_TEST_THROW") {
            let v: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            let t = player.sim_time;
            player.throw_held = v.chunks(2).any(|c| (c[0]..c[0] + c.get(1).copied().unwrap_or(CHARGE_TIME)).contains(&t));
        }
        // BF_TEST_NEXT_GRENADE=<s>[,<s>...]: press the grenade-type key (T) at those times
        if let Ok(v) = std::env::var("BF_TEST_NEXT_GRENADE") {
            let (t, was) = (player.sim_time, player.sim_time - capture_step());
            if v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).any(|s| was < s && t >= s) {
                step_grenade(player);
            }
        }
        return true;
    }
    if std::env::var("BF_AUTOPILOT").is_err() {
        return false;
    }
    let t = player.sim_time;
    let (mv, sprint, aim) = match t {
        t if t < 1.5 => (Vec2::ZERO, false, false),
        t if t < 4.0 => (Vec2::new(0.0, 1.0), false, false),       // run
        t if t < 6.0 => (Vec2::new(0.8, 0.6), false, false),       // turn
        t if t < 7.5 => (Vec2::new(-0.6, 0.8), true, false),       // sprint
        t if t < 9.0 => (Vec2::ZERO, false, false),                // stop
        t if t < 11.0 => (Vec2::new(1.0, 0.0), false, true),       // aimed: run right, torso to camera
        t if t < 12.5 => (Vec2::new(0.0, -1.0), false, true),      // aimed backpedal
        t if t < 13.5 => (Vec2::new(-1.0, 0.0), false, false),     // dodge left (pressed at 13.0)
        t if t < 15.8 => (Vec2::ZERO, false, false),               // standing jump at 14.6
        t if t < 19.5 => (Vec2::new(0.0, 1.0), false, false),     // running jump at 17.0
        t if t < 21.5 => (Vec2::ZERO, false, true),                // aim and fire standing
        t if t < 23.6 => (Vec2::ZERO, false, false),               // switch weapon at 21.8 (standing)
        t if t < 26.2 => (Vec2::new(0.0, 1.0), false, false),     // fire from the hip while running; switch back at 25.5
        _ => (Vec2::ZERO, false, false),
    };
    player.move_input = mv;
    player.sprint = sprint;
    player.walk = false;
    player.aim = aim;
    let was = t - capture_step();
    let at = |s: f32| was < s && t >= s;
    player.dodge_pressed = at(13.0);
    player.jump_pressed = at(14.6) || at(17.0);
    player.fire = (19.8..21.5).contains(&t) || (23.8..25.4).contains(&t);
    // BF_AUTOPILOT_SPIN=<rad/s>: during the standing fire, look down and turn the camera
    if let Some(spin) = std::env::var("BF_AUTOPILOT_SPIN").ok().and_then(|s| s.parse::<f32>().ok()) {
        if (19.5..21.5).contains(&t) {
            player.cam_pitch = -1.0;
            player.cam_yaw = wrap_angle(player.cam_yaw + spin / 15.0);
        }
    }
    player.switch_pressed = at(21.8) || at(25.5);
    player.reload_pressed = at(26.5);
    player.throw_held = (29.0..29.5).contains(&t);
    if at(31.0) {
        player.select = Some((1, SELECT_TIME));                // hand over to Flint
    }
    true
}

/// Where the feet face while standing and aiming: toward the camera heading (less the held gun's
/// yaw) until the gun is up, then wherever the gun is actually aimed. Looking down, the crosshair
/// lands near the character and the gun's direction drifts from the camera heading as the view
/// turns; the spine keeps only a small share (AIM_SPINE_YAW) and the feet turn for the rest.
fn stand_aim_yaw(p: &Player, camera_goal: f32) -> f32 {
    if p.aim_weight < 0.5 {
        return wrap_angle(camera_goal);
    }
    let excess = p.aim_residual - p.aim_residual.clamp(-AIM_SPINE_YAW, AIM_SPINE_YAW);
    wrap_angle(p.yaw + excess)
}

fn wrap_angle(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

#[allow(clippy::too_many_arguments)]
fn update_player(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
                 mut transforms: Query<&mut Transform>, mut heading: Local<Option<f32>>, test: Option<Res<testmap::TestMap>>,
                 kits: Option<Res<grenade::GrenadeKits>>) {
    let dt = frame_dt(&time);
    let kits: &[grenade::GrenadeKit] = kits.as_ref().map_or(&[], |k| &k.0);
    // the squad's heading: the leader's facing, smoothed (formation places turn with it)
    let h = heading.get_or_insert(player.yaw);
    *h = wrap_angle(*h + wrap_angle(player.yaw - *h) * (1.0 - (-1.5 * dt).exp()));
    let heading = *h;
    // take the loaded character out so the step can borrow it and mutate the player freely
    if !player.dead && !knocked_down(&player) {
        if let Some(l) = player.loaded.take() {
            step_player(&mut player, &l, &game.0, kits, dt, &mut transforms);
            player.loaded = Some(l);
        }
    }
    for (slot, m) in squad.0.iter_mut().enumerate() {
        if m.dead || knocked_down(m) {
            continue;
        }
        // (BF_TEST_TARGET's squadmate stands still in the line of fire)
        if slot != 0 || std::env::var("BF_TEST_TARGET").is_err() {
            squad_ai(m, &player, slot, heading, dt);
        }
        let Some(l) = m.loaded.take() else { continue };
        step_player(m, &l, &game.0, kits, dt, &mut transforms);
        m.loaded = Some(l);
    }
    // friendly fire: the player's shots this frame stop at the first teammate in their way
    // the weapon's Damage min..max per shot
    let held = player.loaded.as_ref().and_then(|l| l.weapons.get(player.weapon));
    let damage = held.map_or((8.0, 10.0), |w| if w.def.damage > 0.0 { (w.def.damage_min.min(w.def.damage), w.def.damage) } else { (8.0, 10.0) });
    let ammo = held.map_or(1, |w| w.def.ammo_type);
    let mut shots = std::mem::take(&mut player.shots);
    for shot in shots.iter_mut() {
        let hit = squad.0.iter().enumerate().filter(|(_, m)| !m.dead)
            .filter_map(|(i, m)| ray_body(shot.origin, shot.dir, m, shot.dist).map(|t| (i, t)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((i, t)) = hit {
            shot.dist = t;
            shot.hit = false;
            // it lands when the shot gets there (a bolt flies at its weapon's speed)
            let (lo, hi) = damage;
            let m = &mut squad.0[i];
            let amount = lo + (hi - lo) * m.random(1000) as f32 / 1000.0;
            let local = shot.origin + shot.dir * t - m.position;
            player.pending_hits.push(PendingHit { delay: t / shot.speed.max(1.0), member: i, amount, dir: shot.dir, local, ammo });
        }
        // shot at: the others it passes close to may sidestep (see SHOT_AT_RADIUS)
        for (j, m) in squad.0.iter_mut().enumerate() {
            if m.dead || hit.is_some_and(|(i, _)| i == j) || m.knock.is_some() || m.dodge_request.is_some() {
                continue;
            }
            let chest = m.position + Vec3::Y * (GROUND + m.height + 1.0);
            let t = (chest - shot.origin).dot(shot.dir).clamp(0.0, shot.dist);
            if (shot.origin + shot.dir * t).distance(chest) < SHOT_AT_RADIUS && m.random(SHOT_AT_DODGE_ODDS) == 0 {
                m.dodge_request = Some(m.random(2) == 0);
                if std::env::var("BF_AI_LOG").is_ok() {
                    println!("{} shot at: dodges", CHARACTERS[m.character]);
                }
            }
        }
    }
    player.shots = shots;
    // shots arriving: the hit lands on the member (blood where the shot meets them)
    let mut due = vec![];
    player.pending_hits.retain_mut(|h| {
        h.delay -= dt;
        if h.delay <= 0.0 {
            due.push(*h);
        }
        h.delay > 0.0
    });
    for h in due {
        let Some(m) = squad.0.get_mut(h.member) else { continue };
        // mostly grunts, now and then "I'm hit" or a word about friendly fire
        let tag = match m.random(6) { 0 => FRIENDLY_FIRE_CHATTER, 1 => HIT_CHATTER, _ => HURT_CHATTER };
        let at = m.position + h.local;
        // the test map's instant kill: one hit is enough
        let amount = if test.as_ref().is_some_and(|t| t.instant_kill) { m.health.max(h.amount) } else { h.amount };
        hurt(m, &game.0, amount, tag, Vec3::new(h.dir.x, 0.0, h.dir.z).normalize_or(Vec3::Z) * 4.0 + Vec3::Y * 1.5, at, h.ammo);
        // shot by the player: sidestep one time in three (EVT_DAMAGED_BY_PC -> GOAL_DODGE)
        if !m.dead && m.knock_request.is_none() && m.random(3) == 0 {
            m.dodge_request = Some(m.random(2) == 0);
        }
    }
    // is the crosshair on a teammate (before the ground or a pillar)?
    let (cam, ray) = aim_ray(&player);
    let wall = ray_hit(cam, ray, 150.0).unwrap_or(150.0);
    player.aim_friend = !player.dead && squad.0.iter().any(|m| !m.dead && ray_body(cam, ray, m, wall).is_some());
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        u.hurt_quiet -= dt;
    }
    // shots hitting a body on the ground push it (anyone's: the dead, and the knocked down)
    let shots: Vec<(Vec3, Vec3, f32)> = std::iter::once(&*player).chain(squad.0.iter())
        .flat_map(|u| u.shots.iter().map(|s| (s.origin, s.dir, s.dist))).collect();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let body = match (u.ragdoll.as_mut(), u.knock.as_mut()) {
            (Some(r), _) => r,
            (None, Some(k)) => &mut k.ragdoll,
            _ => continue,
        };
        for &(origin, dir, dist) in &shots {
            if let Some(at) = body.shove(origin, dir, dist) {
                // it bleeds where it's hit, as the living do (a bullet's hit: ammo type 1)
                u.blood.push((at, dir, 1));
                if std::env::var("BF_RAGDOLL_LOG").is_ok() {
                    println!("shot {}'s body at {at:.2}", CHARACTERS[u.character]);
                }
            }
        }
    }
    // bodies: the dead fall limp; the knocked down go limp, then get back up
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        u.knock_cooldown -= dt;
        let Some(l) = u.loaded.take() else { continue };
        let surface_land = game.0.surfaces.get(u.surface).map(|s| s.jump_land.clone()).unwrap_or_default();
        if let (Some(push), false) = (u.knock_request.take(), u.dead) {
            if !u.last_world.is_empty() {
                let push = Quat::from_rotation_y(-u.yaw) * (push + u.last_velocity);
                // (the model stands raised by its sole lift: the ragdoll starts where it's drawn)
                let ragdoll = Ragdoll::new(&l.model, &u.last_world, GROUND - u.height - u.lift_now, push).placed(u.position + Vec3::Y * (u.height + u.lift_now), u.yaw);
                u.knock = Some(Knock { ragdoll, time: 0.0, lying: None });
                u.thud = false;
                u.action = Action::None;
                u.crouch_wanted = false;
                u.switching = None;
                u.throwing = None;
                u.charge = 0.0;
            }
        }
        if u.dead {
            if u.ragdoll.is_none() && !u.last_world.is_empty() {
                let push = Quat::from_rotation_y(-u.yaw) * (u.death_push + u.last_velocity);
                // the knocked down die where they lie
                u.ragdoll = Some(match u.knock.take() {
                    Some(k) => k.ragdoll,
                    None => Ragdoll::new(&l.model, &u.last_world, GROUND - u.height - u.lift_now, push).placed(u.position + Vec3::Y * (u.height + u.lift_now), u.yaw),
                });
            }
            if let Some(r) = u.ragdoll.as_mut() {
                let local = r.step(&l.model, dt);
                if std::env::var("BF_RAGDOLL_LOG").is_ok() && (u.dead_for * 2.0).fract() < dt * 2.0 {
                    let closest = r.pairs.iter().map(|&(a, b)| r.pos[a].distance(r.pos[b])).fold(f32::MAX, f32::min);
                    let turn = Quat::from_rotation_y(r.yaw);
                    let below = r.pos.iter().map(|&p| { let w = r.origin + turn * p;
                        world::arena().and_then(|a| a.floor_below(w.x, w.z, w.y + 0.4)).map_or(0.0, |f| w.y - f.0) }).fold(f32::MAX, f32::min);
                    let worst = r.hinge_sides.iter().filter_map(|&(a, b, c, f, side)| {
                        let axes = r.frames[f].and_then(|fr| torso_axes(&r.pos, fr))?;
                        let n = axes.0 * side.x + axes.1 * side.y + axes.2 * side.z;
                        let along = (r.pos[c] - r.pos[a]).normalize_or_zero();
                        Some(hinge_offset(&r.pos, a, b, c).dot((n - along * n.dot(along)).normalize_or_zero()))
                    }).fold(f32::MAX, f32::min);
                    println!("ragdoll {} t {:.1}: pelvis at {:.2}, {} pairs, closest {:.3} m, lowest bone {:.3} m above its floor, {} hinges, worst bend {:.3} m (negative: wrong way)",
                             CHARACTERS[u.character], u.dead_for, r.origin + turn * r.pos[0], r.pairs.len(), closest, below, r.hinge_sides.len(), worst);
                }
                apply_pose(&l, &local, &mut transforms);
                body_thud(u.position, u.yaw, u.height + u.lift_now, r, &mut u.thud, &mut u.body_at, &surface_land, &mut u.sound_queue, &mut u.rng);
            }
        } else if let Some(mut k) = u.knock.take() {
            k.time += dt;
            if k.time < KNOCK_DOWN_TIME {
                let local = k.ragdoll.step(&l.model, dt);
                apply_pose(&l, &local, &mut transforms);
                body_thud(u.position, u.yaw, u.height + u.lift_now, &k.ragdoll, &mut u.thud, &mut u.body_at, &surface_land, &mut u.sound_queue, &mut u.rng);
                u.knock = Some(k);
            } else {
                // getting up: stand where the body lies, blend the lying pose into the animation
                if k.lying.is_none() {
                    let mut lying = k.ragdoll.step(&l.model, 0.0);
                    let r = k.ragdoll.pos[0];
                    u.position += Quat::from_rotation_y(u.yaw) * Vec3::new(r.x, 0.0, r.z);
                    lying[0].1.x -= r.x;
                    lying[0].1.z -= r.z;
                    k.lying = Some(lying);
                    u.layers.clear();
                    u.gait = Gait::Idle;
                }
                let w = ((k.time - KNOCK_DOWN_TIME) / GET_UP_TIME).clamp(0.0, 1.0);
                let w = w * w * (3.0 - 2.0 * w);
                let still = std::mem::replace(&mut u.move_input, Vec2::ZERO);
                step_player(u, &l, &game.0, &[], dt, &mut transforms);
                u.move_input = still;
                if let Some(lying) = &k.lying {
                    for (e, (q0, t0)) in l.joints.iter().zip(lying) {
                        if let Ok(mut tr) = transforms.get_mut(*e) {
                            let q1 = if q0.dot(tr.rotation) < 0.0 { -tr.rotation } else { tr.rotation };
                            tr.rotation = q0.slerp(q1, w);
                            tr.translation = t0.lerp(tr.translation, w);
                        }
                    }
                }
                if k.time < KNOCK_DOWN_TIME + GET_UP_TIME {
                    u.knock = Some(k);
                } else {
                    u.knock_cooldown = 2.0;
                    u.thud = false;
                    u.body_at = None;
                }
            }
        }
        u.loaded = Some(l);
    }
    let player_first = !player.dead && !knocked_down(&player);
    let mut units: Vec<&mut Player> = std::iter::once(&mut *player).chain(squad.0.iter_mut())
        .filter(|u| !u.dead && !knocked_down(u)).collect();
    collide(&mut units, player_first);
    // on the map: stand on the floor under the feet (terrain, or an object's floor within a
    // step; the game's collision surfaces, floors up to 65 degrees), fall off ledges, slide down
    // slide surfaces as the game does, and don't walk off the map
    if let Some(a) = world::arena() {
        for u in units.iter_mut() {
            let air = u.action.airborne();
            let feet = u.position.y + GROUND + u.height.max(0.0);
            u.sliding -= dt;
            let Some((g, n, material)) = a.floor_at(u.position.x, u.position.z, feet + STEP_UP) else {
                u.position.x = u.prev_xz.x;
                u.position.z = u.prev_xz.y;
                continue;
            };
            if air {
                // in the air: keep the height above the world, over whatever floor is below
                u.height = feet - g;
                u.position.y = g - GROUND;
                u.was_air = true;
            } else if feet - g > STEP_UP && u.knock.is_none() {
                // walked off a ledge: fall from here
                u.action = Action::JumpFall;
                u.vy = 0.0;
                u.height = feet - g;
                u.air_velocity = Vec3::new(u.last_velocity.x, 0.0, u.last_velocity.z);
                u.fall_from = feet;
                u.position.y = g - GROUND;
                u.was_air = true;
                if std::env::var("BF_SLIDE_LOG").is_ok() {
                    println!("{} walks off a ledge at {:.1}: {:.1} m to the floor", CHARACTERS[u.character], u.position, feet - g);
                }
            } else {
                u.position.y = g - GROUND;
                if u.was_air {
                    // landed: a long drop hurts
                    u.was_air = false;
                    let drop = u.fall_from - g;
                    if std::env::var("BF_SLIDE_LOG").is_ok() {
                        println!("{} lands at {:.1} after a {drop:.1} m drop", CHARACTERS[u.character], u.position);
                    }
                    let (lo, hi, most) = FALL_HURT;
                    // a liquid breaks the fall
                    let wet = a.liquid_at(u.position.x, u.position.z, g + 0.5).is_some();
                    if drop > lo && !wet {
                        let k = ((drop - lo) / (hi - lo)).min(1.0);
                        let at = u.position + Vec3::Y * 0.2;
                        hurt(u, &game.0, most * k * k, HURT_CHATTER, Vec3::Y * 2.0, at, -1);
                    }
                    u.fall_from = g;
                }
                slide(u, a, g, n, material, dt);
            }
            // in a liquid: wading is harmless; going under a harmful one (lava, toxic) kills
            let feet = u.position.y + GROUND + u.height.max(0.0);
            let liquid_here = a.liquid_at(u.position.x, u.position.z, feet);
            splashes(u, liquid_here, dt);
            if let Some((surface, liquid)) = liquid_here {
                let deep = surface - feet;
                let rate = liquid.damage.iter().copied().fold(0.0, f32::max);
                let at = Vec3::new(u.position.x, surface, u.position.z);
                if deep > LIQUID_UNDER && rate > 0.0 {
                    let all = u.health;
                    hurt(u, &game.0, all, HURT_CHATTER, Vec3::Y, at, -1);
                } else if rate > 0.0 {
                    // burning: from LIQUID_TOUCH of the liquid's rate at a touch up to all of
                    // it chest-deep, dealt every LIQUID_TICK s (no blood: it burns)
                    let k = (deep / LIQUID_UNDER).clamp(0.0, 1.0);
                    u.burn += rate * (LIQUID_TOUCH + (1.0 - LIQUID_TOUCH) * k) * dt;
                    u.burn_in -= dt;
                    if u.burn_in <= 0.0 {
                        let (amount, bled) = (u.burn, u.blood.len());
                        hurt(u, &game.0, amount, HURT_CHATTER, Vec3::Y, at, -1);
                        u.blood.truncate(bled);
                        u.burn = 0.0;
                        u.burn_in = LIQUID_TICK;
                    }
                }
                if std::env::var("BF_SLIDE_LOG").is_ok() {
                    println!("{} in liquid {} {deep:.2} m deep: {} hp", CHARACTERS[u.character], liquid.kind, u.health);
                }
            }
            u.prev_xz = Vec2::new(u.position.x, u.position.z);
        }
    }
    for u in &units {
        if let Some(tr) = u.loaded.as_ref().and_then(|l| transforms.get_mut(l.root).ok()).as_mut() {
            tr.translation = u.position + Vec3::Y * (u.height + u.lift_now);
        }
    }
}

/// A character and a liquid (its surface height and type, if the feet are under it): going in
/// makes a splash (a big one falling in faster than SPLASH_FAST m/s), wading leaves a ring every
/// WADE_EVERY s while moving; the liquid type's effects and sounds (see `LiquidType`). They're
/// spawned by `spawn_splashes`.
fn splashes(u: &mut Player, liquid: Option<(f32, LiquidType)>, dt: f32) {
    let Some((surface, l)) = liquid else {
        u.wet = false;
        return;
    };
    let at = Vec3::new(u.position.x, surface, u.position.z);
    if !u.wet {
        let slot = if u.vy < -SPLASH_FAST { LIQUID_BIG_SPLASH } else { LIQUID_SPLASH };
        u.splash(at, &l, slot);
        u.wade_in = WADE_EVERY;
    } else {
        let moved = Vec2::new(u.position.x, u.position.z).distance(u.prev_xz);
        u.wade_in -= dt;
        if u.wade_in <= 0.0 && moved > WADE_MOVING * dt && moved < 1.0 {
            u.splash(at, &l, LIQUID_WADE);
            u.wade_in = WADE_EVERY;
        }
    }
    u.wet = true;
}

/// See `splashes`: a fall speed (m/s) that makes a big splash; how often a wader leaves a ring
/// (s), moving faster than WADE_MOVING m/s.
const SPLASH_FAST: f32 = 5.0;
const WADE_EVERY: f32 = 0.5;
const WADE_MOVING: f32 = 0.5;

/// The splashes the characters made this frame (see `splashes`), and shots striking a liquid's
/// surface (see `projectiles`): each liquid effect once, at the surface.
#[allow(clippy::too_many_arguments)]
fn spawn_splashes(mut commands: Commands, mut game: ResMut<GameData>, ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
                  mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>,
                  mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let Some(mut ale) = ale else { return };
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        for (at, effect) in u.splashes.drain(..) {
            spawn_once(&mut commands, &mut game.0, &mut ale, &mut images, &mut materials, effect, Transform::from_translation(at));
        }
    }
}

/// An ALE effect run once at `at` (removed when it's done); nothing if the library lacks it.
fn spawn_once(commands: &mut Commands, game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
              materials: &mut Assets<StandardMaterial>, effect: u32, at: Transform) {
    if let Some(fx) = ale.load(game, images, materials, effect) {
        let life = fx.duration();
        let seed = (at.translation.x * 977.0 + at.translation.z * 131.0) as u32;
        commands.spawn((at, Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx, 0.0, seed), AleExpire(life)));
    }
}

/// The game's slide on the ground under `u` (normal `n`, collision `material`), see SLIDE_COS.
/// Moves `u` while sliding (its own movement this frame replaced, or blended back in as the
/// slide fades) and keeps the slide clip on.
fn slide(u: &mut Player, a: &world::Arena, g: f32, n: Vec3, material: u8, dt: f32) {
    let here = Vec2::new(u.position.x, u.position.z);
    let own = here - u.prev_xz;
    let moving = own.length() > 1e-3 && own.length() < 1.0;     // (not a spawn's jump)
    let flagged = world::Arena::slides(material);
    let s = u.slide_active as usize;
    let lo = SLIDE_COS[1 - s];
    let mut f = (n.y - lo) / (SLIDE_COS[2] - lo);
    if flagged {
        f = f.max(0.5);
    }
    if f <= 0.0 || moving {
        u.slide_amount += f * SLIDE_RATE[s] * dt;
    }
    // downhill along the ground: gravity's part in the ground plane
    let along = (Vec3::NEG_Y + n * n.y).normalize_or_zero();
    let push = along * (1.0 - n.y) * SLIDE_PUSH;
    if u.slide_amount > 1.0 {
        u.slide_amount = 1.0;
        if flagged && !u.slide_active {
            u.slide_active = true;
            if std::env::var("BF_SLIDE_LOG").is_ok() {
                println!("  slide surface {:.0} deg (material {material:#04x})", n.y.clamp(-1.0, 1.0).acos().to_degrees());
            }
            u.slide_vel = push + Vec3::Y * SLIDE_DOWN;
        }
    } else if u.slide_amount <= 0.0 {
        u.slide_amount = 0.0;
        u.slide_active = false;
        u.slide_vel = Vec3::ZERO;
    }
    if !u.slide_active {
        return;
    }
    let target = u.slide_vel + push + Vec3::Y * (SLIDE_DOWN * dt);
    u.slide_vel += (target - u.slide_vel) * (SLIDE_EASE * dt).min(1.0);
    u.slide_vel = u.slide_vel.clamp_length_max(SLIDE_MAX_SPEED);
    let mv = u.slide_vel * dt * u.slide_amount;
    let slid = u.prev_xz + Vec2::new(mv.x, mv.z);
    // fading out: the player's own movement comes back, the more the more they face downhill
    let to = if u.slide_amount >= SLIDE_RELEASE { slid } else {
        let facing = Vec2::new(-u.yaw.sin(), -u.yaw.cos());
        let w = (facing.dot(Vec2::new(u.slide_vel.x, u.slide_vel.z).normalize_or_zero()).max(0.0)
                 * (SLIDE_RELEASE - u.slide_amount) / SLIDE_RELEASE).clamp(0.0, 1.0);
        slid + (here - slid) * w
    };
    // blocked: the game ends the slide when the sweep achieves under a tenth of the move asked
    // (FUN_0012cd10 -> FUN_0012bf40): a wall, the edge of the map, or flat slide-flagged
    // ground, where the slide only presses down into the floor
    let floor = a.floor_at(to.x, to.y, g + STEP_UP);
    let across = Vec2::new(mv.x, mv.z).length();
    match floor {
        Some((g2, _, _)) if across >= SLIDE_BLOCKED * mv.length() => u.position = Vec3::new(to.x, g2 - GROUND, to.y),
        _ => {
            u.slide_active = false;
            u.slide_amount = 0.0;
            u.slide_vel = Vec3::ZERO;
            return;
        }
    }
    let down = Vec2::new(along.x, along.z);
    if down.length() > 1e-3 {
        u.slide_yaw = (-down.x).atan2(-down.y);
    }
    u.sliding = SLIDE_HOLD;
}

/// Lying limp (knocked down, before getting up).
fn knocked_down(u: &Player) -> bool {
    u.knock.as_ref().is_some_and(|k| k.time < KNOCK_DOWN_TIME)
}

/// Write a pose (local transforms) to a character's joints.
fn apply_pose(l: &Loaded, local: &[(Quat, Vec3)], transforms: &mut Query<&mut Transform>) {
    for (e, (q, t)) in l.joints.iter().zip(local) {
        if let Ok(mut tr) = transforms.get_mut(*e) {
            tr.rotation = *q;
            tr.translation = *t;
        }
    }
}

/// Where a ragdoll's body lies (world), and its thud on the ground (the surface's landing sound,
/// once, when the pelvis comes down).
#[allow(clippy::too_many_arguments)]
fn body_thud(position: Vec3, yaw: f32, height: f32, r: &Ragdoll, thud: &mut bool, body_at: &mut Option<Vec3>,
             land: &[u32], queue: &mut Vec<(u32, f32)>, rng: &mut u32) {
    let root = Transform::from_translation(position + Vec3::Y * height).with_rotation(Quat::from_rotation_y(yaw));
    let pelvis = root.transform_point(r.pos[0]);
    *body_at = Some(pelvis);
    if !*thud && r.pos[0].y <= r.floor + 0.25 {
        *thud = true;
        if !land.is_empty() {
            *rng ^= *rng << 13;
            *rng ^= *rng >> 17;
            *rng ^= *rng << 5;
            queue.push((land[*rng as usize % land.len()], 0.9));
        }
    }
}

/// Have `u` say a random line of a chatter set (after `delay` s), unless they are already talking.
fn say(u: &mut Player, game: &Game, tag: u32, delay: f32) {
    if u.speaking > 0.0 || u.quote_in.is_some() {
        return;
    }
    let lines: Vec<u32> = game.chatter.get(CHARACTERS[u.character]).and_then(|c| c.get(&tag))
        .map(|v| v.iter().copied().filter(|&id| game.sounds.has(id)).collect()).unwrap_or_default();
    if !lines.is_empty() {
        let k = u.random(lines.len());
        u.quote_in = Some((delay, lines[k]));
    }
}

/// Damage `u` (hit at `at`, world, by `ammo`, -1 for a blast): blood; at 0 they die (death cry, the body goes limp with
/// `push`, a surviving squadmate may answer); a hard hit can knock them down; otherwise they
/// grunt or say a line of `tag` now and then.
fn hurt(u: &mut Player, game: &Game, amount: f32, tag: u32, push: Vec3, at: Vec3, ammo: i64) {
    if u.dead || amount <= 0.0 {
        return;
    }
    u.blood.push((at, push.normalize_or(Vec3::Y), ammo));
    u.health = (u.health - amount).max(0.0);
    if std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("hit {} for {amount:.1} -> {:.1} hp at {at:.2} by ammo type {ammo} (knock cooldown {:.2})", CHARACTERS[u.character], u.health, u.knock_cooldown);
    }
    if u.health <= 0.0 {
        u.dead = true;
        u.death_push = push;
        u.fire = false;
        u.aim = false;
        u.quote_in = None;
        u.speaking = 0.0;
        u.knock = None;
        say(u, game, DEATH_CHATTER, 0.0);
        // the answer comes once the cry is over
        let cry = u.quote_in.map_or(1.5, |(_, id)| game.sounds.pcm(id).map_or(1.5, |(r, s)| s.len() as f32 / r as f32));
        if let Some(&r) = game.chatter_response.get(CHARACTERS[u.character]).and_then(|m| m.get(&DEATH_CHATTER)) {
            if u.random(DEATH_RESPONSE_CHANCE) == 0 {
                u.death_response = Some((cry + 0.4, r));
            }
        }
        return;
    }
    // a hard hit floors them now and then (not right after getting up)
    if amount >= KNOCKDOWN_DAMAGE && u.knock.is_none() && u.knock_cooldown <= 0.0 && !u.action.airborne() && u.random(3) != 0 {
        u.knock_request = Some(push);
        if std::env::var("BF_COMBAT_LOG").is_ok() {
            println!("  knocked down: {}", CHARACTERS[u.character]);
        }
    }
    if u.hurt_quiet <= 0.0 {
        say(u, game, tag, 0.05);
        u.hurt_quiet = 0.8;
    }
}

/// Distance along a ray to a standing character (an upright cylinder of BODY_RADIUS from the
/// ground to BODY_HEIGHT), if hit within `max`.
fn ray_body(origin: Vec3, dir: Vec3, u: &Player, max: f32) -> Option<f32> {
    let (o, d) = (Vec2::new(origin.x - u.position.x, origin.z - u.position.z), Vec2::new(dir.x, dir.z));
    let a = d.length_squared();
    if a < 1e-8 {
        return None;
    }
    let (b, c) = (o.dot(d), o.length_squared() - BODY_RADIUS * BODY_RADIUS);
    let disc = b * b - a * c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / a;
    let t = if t < 0.0 { (-b + disc.sqrt()) / a } else { t };
    let y = origin.y + dir.y * t - (u.position.y + GROUND + u.height);
    (t > 0.05 && t < max && (0.0..BODY_HEIGHT).contains(&y)).then_some(t)
}

/// Knocked down: limp (the ragdoll) for KNOCK_DOWN_TIME, then back up: the pose blends from where
/// the body lay into the animation over GET_UP_TIME.
struct Knock {
    ragdoll: Ragdoll,
    time: f32,
    /// the lying pose the get-up starts from (local, root moved under the character)
    lying: Option<Vec<(Quat, Vec3)>>,
}

/// How thick the body is round each bone, so the ragdoll keeps its skin, not just its bones, out
/// of the floor (bones held 5 cm up let a thigh or the chest sink 10 cm into it). For each
/// bone: of the skin vertices it moves most (skinned in the starting pose), the distance from
/// the bone's line (to the child it points at) that FLESH_SHARE of them are within, between
/// FLESH_MIN and FLESH_MAX. Bones that move no skin get FLESH_MIN.
fn flesh(model: &Character, world: &[Mat4], pos: &[Vec3], aim: &[Option<(usize, Vec3)>]) -> Vec<f32> {
    let n = pos.len();
    let skin: Vec<Mat4> = world.iter().zip(&model.inverse_bind).map(|(w, ib)| *w * *ib).collect();
    let mut near: Vec<Vec<f32>> = vec![vec![]; n];
    for g in &model.geosets {
        for (k, v) in g.positions.iter().enumerate() {
            let (Some(j), Some(w)) = (g.joints.get(k), g.weights.get(k)) else { continue };
            let at = (0..4).filter(|&i| w[i] > 0.0)
                .filter_map(|i| skin.get(j[i] as usize).map(|m| m.transform_point3(Vec3::from(*v)) * w[i])).sum::<Vec3>();
            let main = (0..4).max_by(|&a, &b| w[a].total_cmp(&w[b])).map(|i| j[i] as usize).unwrap_or(0);
            if main >= n {
                continue;
            }
            // distance from the bone's line: its point to the child it points at
            let a = pos[main];
            let d = match aim[main] {
                Some((c, _)) if c < n => {
                    let ab = pos[c] - a;
                    let t = ((at - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
                    at.distance(a + ab * t)
                }
                _ => at.distance(a),
            };
            near[main].push(d);
        }
    }
    near.into_iter().map(|mut d| {
        if d.is_empty() {
            return FLESH_MIN;
        }
        d.sort_by(f32::total_cmp);
        d[((d.len() - 1) as f32 * FLESH_SHARE) as usize].clamp(FLESH_MIN, FLESH_MAX)
    }).collect()
}

/// See `flesh`: the share of a bone's skin within its thickness, and the least and most.
const FLESH_SHARE: f32 = 0.8;
const FLESH_MIN: f32 = 0.04;
const FLESH_MAX: f32 = 0.2;

/// A limp body: every bone is a point mass (verlet) held to its parent at its bone length (and
/// loosely to its grandparent, so limbs keep some shape), falling under gravity onto the ground.
/// Bones turn with their link to their farthest child. Simulated in the character's model space.
struct Ragdoll {
    pos: Vec<Vec3>,
    prev: Vec<Vec3>,
    rest_rot: Vec<Quat>,
    /// per bone: its offset from its parent at the start, in the parent's frame (the pose keeps
    /// it: bones turn, they don't stretch - free points drifting apart stretched the skin)
    rest_offset: Vec<Vec3>,
    /// per bone: the child it points at and that direction at the start
    aim: Vec<Option<(usize, Vec3)>>,
    links: Vec<(usize, usize, f32, f32)>,
    /// joint limits: two bones kept between a nearest and a farthest distance (a knee or elbow
    /// bends, but doesn't fold flat or straighten past straight; the head doesn't loll)
    ranges: Vec<(usize, usize, f32, f32)>,
    /// the torso's two frames from its bones (hips: left / right thigh and pelvis / spine; chest:
    /// left / right upper arm and spine / neck): right-hand pair, bottom-top pair
    frames: [Option<[usize; 4]>; 2],
    /// hinges bend one way: knee / elbow (upper, joint, end), its torso frame, and the side the
    /// joint must stay on (in that frame) of the line from upper to end
    hinge_sides: Vec<(usize, usize, usize, usize, Vec3)>,
    /// substeps the body has been still for (then it rests: no more jitter)
    still: u32,
    floor: f32,
    /// where its frame (the body's root: bones are kept relative to it) is in the world, so
    /// the bones collide with the level: origin and turn about y
    origin: Vec3,
    yaw: f32,
    /// the limb bones that collide with each other (RAGDOLL_SELF_RADIUS spheres): pairs that
    /// aren't within RAGDOLL_SELF_HOPS of each other in the skeleton and didn't start touching
    pairs: Vec<(usize, usize)>,
    /// per bone: how thick the body is round it (see `flesh`): it's kept that far off the floor
    flesh: Vec<f32>,
    /// times it has been rolled off its side (see ROLL_TRIES)
    rolls: u8,
}

impl Ragdoll {
    fn new(model: &Character, world: &[Mat4], floor: f32, push: Vec3) -> Self {
        let n = model.bones.len().min(world.len());
        let (mut pos, mut rest_rot) = (Vec::with_capacity(n), Vec::with_capacity(n));
        for m in &world[..n] {
            let (_, r, t) = m.to_scale_rotation_translation();
            pos.push(t);
            rest_rot.push(r);
        }
        let mut aim = vec![None; n];
        let mut links = vec![];
        // biped bones by name (Bip01 ...)
        let bone = |name: &str| model.bones.iter().position(|&b| b == bf_viewer::bf::hash::h(name)).filter(|&i| i < n);
        let sides = ["L", "R"];
        // hinges (upper, middle, end): knees and elbows; their upper-to-end distance is limited
        // instead of held, so they bend freely within reason
        let hinges: Vec<(usize, usize, usize)> = sides.iter().flat_map(|s| [
            (bone(&format!("Bip01 {s} Thigh")), bone(&format!("Bip01 {s} Calf")), bone(&format!("Bip01 {s} Foot"))),
            (bone(&format!("Bip01 {s} UpperArm")), bone(&format!("Bip01 {s} Forearm")), bone(&format!("Bip01 {s} Hand"))),
        ]).filter_map(|t| Some((t.0?, t.1?, t.2?))).collect();
        let hinge_end = |g: usize, i: usize| hinges.iter().any(|&(a, _, c)| a == g && c == i);
        for i in 0..n {
            if let Some(p) = model.parent[i] {
                links.push((p, i, pos[i].distance(pos[p]), 1.0));
                if let Some(g) = model.parent[p].filter(|&g| !hinge_end(g, i)) {
                    links.push((g, i, pos[i].distance(pos[g]), RAGDOLL_BEND_STIFFNESS));
                }
                let d = pos[i] - pos[p];
                if d.length() > 0.03 && aim[p].is_none_or(|(c, _): (usize, Vec3)| d.length() > (pos[c] - pos[p]).length()) {
                    aim[p] = Some((i, d));
                }
            }
        }
        // a push, more on the upper body than the feet (so a hit topples the body rather than
        // sliding it), and the body sagging (so the legs buckle instead of standing like posts):
        // bones start moving with it, scaled by their height (velocity over one RAGDOLL_STEP)
        let top = pos.iter().map(|p| p.y).fold(floor + 0.5, f32::max);
        let prev = pos.iter().map(|&q| {
            let w = ((q.y - floor) / (top - floor)).clamp(0.1, 1.0);
            q - (push * w * 1.6 + Vec3::NEG_Y * 2.5 * w) * RAGDOLL_STEP
        }).collect();
        // the torso moves as two solid pieces (hips and chest), stiffly joined at the waist; the
        // head is held to the chest
        let hips: Vec<usize> = ["Bip01 Pelvis", "Bip01 Spine", "Bip01 L Thigh", "Bip01 R Thigh"].iter().filter_map(|b| bone(b)).collect();
        let chest: Vec<usize> = ["Bip01 Spine1", "Bip01 Spine2", "Bip01 Spine3", "Bip01 Neck", "Bip01 L Clavicle", "Bip01 R Clavicle",
                                 "Bip01 L UpperArm", "Bip01 R UpperArm"].iter().filter_map(|b| bone(b)).collect();
        for group in [&hips, &chest] {
            for (k, &a) in group.iter().enumerate() {
                for &b in &group[k + 1..] {
                    links.push((a, b, pos[a].distance(pos[b]), 1.0));
                }
            }
        }
        for &a in &hips {
            for &b in &chest {
                links.push((a, b, pos[a].distance(pos[b]), RAGDOLL_WAIST_STIFFNESS));
            }
        }
        // the hinges' bend side, in the torso's own frame: as they were bent at death (most die
        // mid-stride or mid-aim); nearly straight, knees forward, elbows back (forward: the toes')
        let frames = [
            (|| Some([bone("Bip01 L Thigh")?, bone("Bip01 R Thigh")?, bone("Bip01 Pelvis")?, bone("Bip01 Spine")?]))(),
            (|| Some([bone("Bip01 L UpperArm")?, bone("Bip01 R UpperArm")?, bone("Bip01 Spine1")?, bone("Bip01 Neck")?]))(),
        ];
        let forward = (|| {
            let (foot, toe) = (bone("Bip01 L Foot")?, bone("Bip01 L Toe0")?);
            Some(pos[toe] - pos[foot])
        })();
        let mut hinge_sides = vec![];
        for &(a, b, c) in &hinges {
            let knee = bone("Bip01 L Thigh") == Some(a) || bone("Bip01 R Thigh") == Some(a);
            let f = if knee { 0 } else { 1 };
            let Some(axes) = frames[f].and_then(|fr| torso_axes(&pos, fr)) else { continue };
            let off = hinge_offset(&pos, a, b, c);
            let side = if off.length() > 0.02 {
                off.normalize()
            } else {
                // nearly straight: by the body's forward
                let Some(fw) = forward else { continue };
                let fw = (fw - axes.0 * fw.dot(axes.0)).normalize_or_zero();
                if knee { fw } else { -fw }
            };
            hinge_sides.push((a, b, c, f, Vec3::new(side.dot(axes.0), side.dot(axes.1), side.dot(axes.2))));
        }
        let mut ranges = vec![];
        for &(a, b, c) in &hinges {
            let straight = pos[a].distance(pos[b]) + pos[b].distance(pos[c]);
            let near = if bone("Bip01 L Thigh") == Some(a) || bone("Bip01 R Thigh") == Some(a) { RAGDOLL_KNEE_FOLD } else { RAGDOLL_ELBOW_FOLD };
            ranges.push((a, c, straight * near, straight * 0.995));
        }
        if let (Some(head), Some(neck)) = (bone("Bip01 Head"), bone("Bip01 Neck")) {
            for &c in &chest {
                if c != neck {
                    let d = pos[head].distance(pos[c]);
                    ranges.push((c, head, d * 0.85, d * 1.02));
                }
            }
        }
        // self collision: the bones that carry a limb (a child some way off), paired with those
        // far enough along the skeleton
        let limb: Vec<usize> = (0..n).filter(|&i| aim[i].is_some_and(|(_, d): (usize, Vec3)| d.length() > 0.08)).collect();
        let hops = |a: usize, b: usize| {
            let chain = |mut i: usize| { let mut v = vec![i]; while let Some(p) = model.parent[i] { v.push(p); i = p; } v };
            let (ca, cb) = (chain(a), chain(b));
            ca.iter().enumerate().find_map(|(k, x)| cb.iter().position(|y| y == x).map(|j| k + j)).unwrap_or(usize::MAX)
        };
        let mut pairs = vec![];
        for (k, &a) in limb.iter().enumerate() {
            for &b in &limb[k + 1..] {
                if hops(a, b) > RAGDOLL_SELF_HOPS && pos[a].distance(pos[b]) > 2.0 * RAGDOLL_SELF_RADIUS {
                    pairs.push((a, b));
                }
            }
        }
        let rest_offset = (0..n).map(|i| model.parent[i].filter(|&p| p < n).map_or(pos[i], |p| rest_rot[p].inverse() * (pos[i] - pos[p]))).collect();
        let flesh = flesh(model, world, &pos, &aim);
        Ragdoll { pos, prev, rest_rot, rest_offset, aim, links, ranges, frames, hinge_sides, still: 0, floor, origin: Vec3::ZERO, yaw: 0.0, pairs, flesh, rolls: 0 }
    }

    /// Place its frame in the world (the body's root: position + height, turned by yaw), so
    /// its bones meet the level's floors and walls.
    /// A shot (world line from `origin` along `dir`, ending at `dist`) through the body: the
    /// first bone it passes within its thickness (`flesh`) of is brought up to SHOT_SHOVE m/s
    /// along the shot, and bones near it less (falling off to nothing at SHOT_SHOVE_REACH m). It wakes the
    /// body (it settles again, and can roll off its side again). Where it hit (world), if it did.
    fn shove(&mut self, origin: Vec3, dir: Vec3, dist: f32) -> Option<Vec3> {
        let back = Quat::from_rotation_y(self.yaw).inverse();
        let (o, d) = (back * (origin - self.origin), back * dir);
        let hit = self.pos.iter().enumerate().filter_map(|(i, p)| {
            let t = (*p - o).dot(d);
            let miss = (o + d * t).distance(*p);
            (t > 0.0 && t < dist + 0.2 && miss < self.flesh[i] + 0.05).then_some((t, i))
        }).min_by(|a, b| a.0.total_cmp(&b.0));
        let (along, at) = hit?;
        let centre = self.pos[at];
        for (p, q) in self.pos.iter().zip(self.prev.iter_mut()) {
            let k = 1.0 - p.distance(centre) / SHOT_SHOVE_REACH;
            // up to that speed along the shot, not on top of it: a burst jolts the body, it
            // doesn't drive it across the ground
            let going = (*p - *q).dot(d) / RAGDOLL_STEP;
            let add = (SHOT_SHOVE * k - going).max(0.0);
            if add > 0.0 {
                *q -= d * add * RAGDOLL_STEP;
            }
        }
        self.still = 0;
        self.rolls = 0;
        Some(origin + dir * along)
    }

    fn placed(mut self, origin: Vec3, yaw: f32) -> Self {
        self.origin = origin;
        self.yaw = yaw;
        self
    }

    /// Advance `dt` (in fixed RAGDOLL_STEP substeps, so it falls at the same speed whatever the
    /// frame rate); returns the bones' local (rotation, translation).
    fn step(&mut self, model: &Character, dt: f32) -> Vec<(Quat, Vec3)> {
        let steps = (dt / RAGDOLL_STEP).round().clamp(0.0, 8.0) as usize;
        for _ in 0..steps {
            self.substep();
        }
        self.local(model)
    }

    fn substep(&mut self) {
        // come to rest on its side (propped on a shoulder and a hip): it rolls on, onto its back or
        // front, whichever way it leans, as a body would
        if self.still > RAGDOLL_SETTLE && self.rolls < ROLL_TRIES {
            if let Some([l, r, lo, hi]) = self.frames[1] {
                if let Some((right, _, forward)) = torso_axes(&self.pos, [l, r, lo, hi]) {
                    if right.y.abs() > ROLL_SIDE {
                        let top = if self.pos[r].y > self.pos[l].y { r } else { l };
                        let flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
                        let way = if forward.y >= 0.0 { -flat } else { flat };
                        for (b, k) in [(top, 1.0), (hi, 0.6), (lo, 0.3)] {
                            self.prev[b] -= way * ROLL_PUSH * k * RAGDOLL_STEP;
                        }
                        self.rolls += 1;
                        self.still = 0;
                    }
                }
            }
        }
        // at rest: nothing moves (a settled body doesn't twitch)
        if self.still > RAGDOLL_SETTLE {
            return;
        }
        let h = RAGDOLL_STEP;
        let g = Vec3::new(0.0, -9.8, 0.0) * h * h;
        for (p, q) in self.pos.iter_mut().zip(self.prev.iter_mut()) {
            let v = (*p - *q) * RAGDOLL_DAMPING;
            *q = *p;
            *p += v + g;
        }
        // the level under and around each bone (world <-> the body's frame)
        let turn = Quat::from_rotation_y(self.yaw);
        let back = turn.inverse();
        let arena = world::arena();
        let floors: Vec<f32> = self.pos.iter().map(|&p| {
            let w = self.origin + turn * p;
            arena.and_then(|a| a.floor_below(w.x, w.z, w.y + RAGDOLL_STEP_UP)).map_or(self.floor, |f| f.0 - self.origin.y)
        }).collect();
        for _ in 0..10 {
            // hinges bend one way: a joint on the wrong side of its limb's line goes back to it
            for &(a, b, c, f, side) in &self.hinge_sides {
                let Some(axes) = self.frames[f].and_then(|fr| torso_axes(&self.pos, fr)) else { continue };
                let n = (axes.0 * side.x + axes.1 * side.y + axes.2 * side.z).normalize_or_zero();
                // (only across the limb: the side less its part along the limb)
                let along = (self.pos[c] - self.pos[a]).normalize_or_zero();
                let n = (n - along * n.dot(along)).normalize_or_zero();
                let wrong = hinge_offset(&self.pos, a, b, c).dot(n);
                if wrong < 0.0 {
                    // the joint moves out, the ends in (they share the correction)
                    let fix = n * -wrong;
                    self.pos[b] += fix * 0.67;
                    self.pos[a] -= fix * 0.17;
                    self.pos[c] -= fix * 0.17;
                }
            }
            // joint limits
            for &(a, b, lo, hi) in &self.ranges {
                let d = self.pos[b] - self.pos[a];
                let l = d.length();
                let want = l.clamp(lo, hi);
                if l > 1e-5 && (want - l).abs() > 1e-6 {
                    let c = d * ((l - want) / l) * 0.5;
                    self.pos[a] += c;
                    self.pos[b] -= c;
                }
            }
            // the body doesn't pass through itself
            for &(a, b) in &self.pairs {
                let d = self.pos[b] - self.pos[a];
                let l = d.length();
                let min = 2.0 * RAGDOLL_SELF_RADIUS;
                if l < min && l > 1e-5 {
                    let c = d * ((min - l) / l) * 0.5;
                    self.pos[a] -= c;
                    self.pos[b] += c;
                }
            }
            for &(a, b, len, k) in &self.links {
                let d = self.pos[b] - self.pos[a];
                let l = d.length();
                if l > 1e-5 {
                    let c = d * ((l - len) / l) * 0.5 * k;
                    self.pos[a] += c;
                    self.pos[b] -= c;
                }
            }
            for (((p, q), &floor), &flesh) in self.pos.iter_mut().zip(self.prev.iter_mut()).zip(&floors).zip(&self.flesh) {
                if p.y < floor + flesh {
                    p.y = floor + flesh;
                    // ground friction: lose most of the sliding
                    q.x += (p.x - q.x) * RAGDOLL_FRICTION;
                    q.z += (p.z - q.z) * RAGDOLL_FRICTION;
                }
            }
        }
        // settling: still while nothing moves more than a millimetre a substep
        let moving = self.pos.iter().zip(&self.prev).any(|(p, q)| p.distance_squared(*q) > 1e-6);
        self.still = if moving { 0 } else { self.still + 1 };
        // walls: each bone kept out of them (a small circle at its height)
        if let Some(a) = arena {
            for (p, q) in self.pos.iter_mut().zip(self.prev.iter_mut()) {
                let w = self.origin + turn * *p;
                let out = a.push_out(Vec2::new(w.x, w.z), RAGDOLL_SELF_RADIUS, w.y - 0.05, w.y + 0.05, 0.0);
                if (out - Vec2::new(w.x, w.z)).length_squared() > 1e-8 {
                    let moved = back * Vec3::new(out.x - w.x, 0.0, out.y - w.z);
                    *p += moved;
                    // and lose the speed into the wall
                    *q += moved;
                }
            }
        }
    }

    /// The bones' local transforms for the current positions.
    fn local(&self, model: &Character) -> Vec<(Quat, Vec3)> {
        let n = self.pos.len();
        let mut rot = vec![Quat::IDENTITY; n];
        for i in 0..n {
            let delta = match self.aim[i] {
                Some((c, d0)) => {
                    let d = self.pos[c] - self.pos[i];
                    if d.length() > 1e-4 { Quat::from_rotation_arc(d0.normalize(), d.normalize()) } else { Quat::IDENTITY }
                }
                None => model.parent[i].map_or(Quat::IDENTITY, |p| rot[p] * self.rest_rot[p].inverse()),
            };
            rot[i] = (delta * self.rest_rot[i]).normalize();
        }
        (0..n).map(|i| match model.parent[i] {
            Some(p) => (rot[p].inverse() * rot[i], self.rest_offset[i]),
            None => (rot[i], self.pos[i]),
        }).collect()
    }
}

/// A torso frame from its bones (left, right, bottom, top): right, up, and their cross.
fn torso_axes(pos: &[Vec3], [l, r, lo, hi]: [usize; 4]) -> Option<(Vec3, Vec3, Vec3)> {
    let right = (pos[r] - pos[l]).normalize_or_zero();
    let up = pos[hi] - pos[lo];
    let up = (up - right * up.dot(right)).normalize_or_zero();
    (right != Vec3::ZERO && up != Vec3::ZERO).then(|| (right, up, right.cross(up)))
}

/// Where a hinge's joint is off the straight line from its upper bone to its end.
fn hinge_offset(pos: &[Vec3], a: usize, b: usize, c: usize) -> Vec3 {
    let line = pos[c] - pos[a];
    let t = if line.length_squared() > 1e-8 { (pos[b] - pos[a]).dot(line) / line.length_squared() } else { 0.0 };
    pos[b] - (pos[a] + line * t)
}

/// Collision: characters are circles (BODY_RADIUS) on the ground: pushed out of the pillars and
/// apart from each other (the player - units[0] when `player_first` - gives way less).
fn collide(units: &mut [&mut Player], player_first: bool) {
    for _ in 0..2 {
        for u in units.iter_mut() {
            for c in pillars() {
                let half = 0.3 + BODY_RADIUS;
                let d = u.position - c;
                if d.x.abs() < half && d.z.abs() < half && u.height < 3.0 {
                    if half - d.x.abs() < half - d.z.abs() {
                        u.position.x = c.x + half * d.x.signum();
                    } else {
                        u.position.z = c.z + half * d.z.signum();
                    }
                }
            }
        }
        if let Some(a) = world::arena() {
            for u in units.iter_mut() {
                let feet = u.position.y + GROUND + u.height;
                let q = a.push_out(Vec2::new(u.position.x, u.position.z), BODY_RADIUS, feet, feet + BODY_HEIGHT, STEP_UP);
                u.position.x = q.x;
                u.position.z = q.y;
            }
        }
        for i in 0..units.len() {
            let (a, b) = units.split_at_mut(i + 1);
            let u = &mut a[i];
            for (k, v) in b.iter_mut().enumerate() {
                let d = Vec2::new(u.position.x - v.position.x, u.position.z - v.position.z);
                let dist = d.length();
                if dist < 2.0 * BODY_RADIUS {
                    let dir = if dist > 1e-4 { d / dist } else { Vec2::new(1.0, (k as f32).sin()).normalize() };
                    let push = 2.0 * BODY_RADIUS - dist;
                    let share = if i == 0 && player_first { 0.2 } else { 0.5 };
                    u.position += Vec3::new(dir.x, 0.0, dir.y) * push * share;
                    v.position -= Vec3::new(dir.x, 0.0, dir.y) * push * (1.0 - share);
                }
            }
        }
    }
}

fn step_player(p: &mut Player, l: &Loaded, game: &Game, kits: &[grenade::GrenadeKit], dt: f32, transforms: &mut Query<&mut Transform>) {
    p.sim_time += dt;
    if p.next_surface && !game.surfaces.is_empty() {
        p.surface = (p.surface + 1) % game.surfaces.len();
    }
    let surface = game.surfaces.get(p.surface).cloned().unwrap_or_default();

    // camera-relative directions (camera yaw 0 looks down -Z)
    let fwd = Vec3::new(-p.cam_yaw.sin(), 0.0, -p.cam_yaw.cos());
    let right = Vec3::new(p.cam_yaw.cos(), 0.0, -p.cam_yaw.sin());
    let input = p.move_input;
    let moving = input.length() > 0.1;
    let dir = if moving { (right * input.x + fwd * input.y).normalize() } else { Vec3::ZERO };
    let dir_yaw = (-dir.x).atan2(-dir.z);
    let turn_to = |yaw: &mut f32, want: f32, rate: f32| {
        *yaw = wrap_angle(*yaw + wrap_angle(want - *yaw).clamp(-rate * dt, rate * dt));
    };
    let dur = |c: Option<usize>| c.map(|c| l.model.anims[c].duration).unwrap_or(0.3);
    let c = &l.clips;
    let four = p.on_all_fours && c.leg4_launch.is_some();
    // the held weapon's stance set (kneel and dive clips)
    let stance = &c.slots[stance_of(l, p.weapon)];

    // ---- actions: dodge and the jump chain (crouch -> launch -> fall -> land) ----
    p.action_left -= dt;
    p.jump_buffer = if p.jump_pressed { 0.4 } else { p.jump_buffer - dt };
    let start_launch = |p: &mut Player| {
        p.action = Action::JumpLaunch;
        p.action_left = dur(if four { c.leg4_launch } else { c.jump_launch });
        p.vy = JUMP_SPEED;
        p.air_velocity = Vec3::new(p.last_velocity.x, 0.0, p.last_velocity.z);
        p.fall_from = p.position.y + GROUND;
        p.sound_queue.push((l.jump_sound, 1.0));
    };
    match p.action {
        Action::JumpCrouch if p.action_left <= 0.0 => start_launch(p),
        Action::JumpLaunch if p.action_left <= 0.0 => p.action = Action::JumpFall,
        Action::Dodge { .. } | Action::JumpLand | Action::Rising | Action::Dive if p.action_left <= FADE => p.action = Action::None,
        Action::Crouching if p.action_left <= FADE => p.action = Action::Crouched,
        // (moving, it crouch-walks where the stance has one, else stands)
        Action::Crouched if !p.crouch_wanted || (moving && stance.cr_walk.is_none()) || p.jump_buffer > 0.0 || p.dodge_pressed => {
            p.action = Action::Rising;
            p.action_left = dur(stance.crouch2stand);
            if stance.crouch2stand.is_none() {
                p.action = Action::None;
            }
        }
        _ => {}
    }
    let grounded = !p.action.airborne();
    // a grenade close by: dive away (out of a kneel too)
    if let Some(from) = p.dive_from.take() {
        if grounded && matches!(p.action, Action::None | Action::Crouching | Action::Crouched) && stance.dive.is_some() {
            let away = Vec3::new(p.position.x - from.x, 0.0, p.position.z - from.z).normalize_or(Vec3::Z);
            p.yaw = (-away.x).atan2(-away.z);
            p.action = Action::Dive;
            p.action_left = dur(stance.dive);
            if std::env::var("BF_COMBAT_LOG").is_ok() {
                println!("  dive: {} away from {from:.1}", CHARACTERS[p.character]);
            }
            p.crouch_wanted = false;
            p.still_time = 0.0;
        }
    }
    if grounded && matches!(p.action, Action::None) {
        if p.jump_buffer > 0.0 {
            p.jump_buffer = 0.0;
            if moving {
                start_launch(p);                             // running jump: straight into the launch
            } else {
                p.action = Action::JumpCrouch;
                p.action_left = dur(c.jump_crouch);
                if c.jump_crouch.is_none() {
                    start_launch(p);
                }
            }
        } else if p.crouch_wanted && moving && stance.cr_walk.is_some() && stance.crouch_idle.is_some() {
            // crouching on the move: straight into the crouch walk
            p.action = Action::Crouched;
        } else if p.crouch_wanted && !moving && stance.stand2crouch.is_some() && stance.crouch_idle.is_some() {
            p.action = Action::Crouching;
            p.action_left = dur(stance.stand2crouch);
        } else if p.dodge_pressed {
            let side = if moving { wrap_angle(dir_yaw - p.yaw).sin() } else { -input.x };
            let left = side > 0.0;
            if let Some(clip) = if left { c.loco.dodge_left } else { c.loco.dodge_right } {
                p.action = Action::Dodge { left };
                p.action_left = l.model.anims[clip].duration;
                p.queue_any(&surface.slide, 0.7);
            }
        }
    }

    // ---- weapons: switch and fire ----
    let armed = !l.weapons.is_empty();
    if p.switch_pressed && l.weapons.len() > 1 && p.switching.is_none() && p.reloading.is_none() && p.throwing.is_none() && p.using.is_none() {
        let to = (p.weapon + 1) % l.weapons.len();
        p.sound_queue.push((SWITCH_SOUNDS[0], 0.9));
        p.switch_sound_in = SWITCH_SOUND_GAP;
        p.hud_list = HUD_LIST_TIME;
        if to < 2 && l.switch_clips[to].is_some() {
            p.switching = Some(Switching { to, time: 0.0, dropped: false, grabbed: false });
            p.aim_hold = 0.0;
        } else {
            // no clip: swap at once
            p.weapon = to;
            p.weapon_dirty = true;
        }
    }
    if p.switch_sound_in >= 0.0 {
        p.switch_sound_in -= dt;
        if p.switch_sound_in < 0.0 {
            p.sound_queue.push((SWITCH_SOUNDS[1], 0.9));
        }
    }
    if let Some(sw) = p.switching.as_mut() {
        sw.time += dt;
        let clip = l.switch_clips[sw.to].as_ref().unwrap();
        let (mut drop_now, mut grab_now) = (false, false);
        if !sw.dropped && sw.time >= clip.drop {
            sw.dropped = true;
            drop_now = true;
        }
        if !sw.grabbed && sw.time >= clip.grab {
            sw.grabbed = true;
            grab_now = true;
        }
        let (to, done) = (sw.to, sw.time >= clip.duration);
        if drop_now {
            p.holding = false;                               // the held gun goes to its stow point
            p.weapon_dirty = true;
        }
        if grab_now {
            p.weapon = to;
            p.holding = true;
            p.weapon_dirty = true;
        }
        if done {
            p.switching = None;
            p.cooldown = 0.1;
        }
    }
    p.hud_list -= dt;
    // ---- ammo: a reload empties the clip at once (the HUD shows 0, in red) and plays the stance's
    // reload clip; the clip is full again at its magazine-in event
    p.reload_wanted |= p.reload_pressed;
    let clip = p.ammo.get(p.weapon).map_or(0, |a| a[0]);
    let clip_size = l.weapons.get(p.weapon).map_or(0, |w| w.def.ammo.max(1));
    if armed && p.reloading.is_none() && p.switching.is_none() && p.throwing.is_none() && p.using.is_none() && (clip == 0 || p.reload_wanted) {
        p.reload_wanted = false;
        let w = p.weapon;
        let a = &mut p.ammo[w];
        let take = (clip_size - a[0]).min(a[1]);
        if take > 0 {
            let fill = a[0] + take;
            a[1] -= take;
            a[0] = 0;
            p.reloading = Some(Reload { weapon: w, time: 0.0, fill, filled: false });
            p.aim_hold = p.aim_hold.min(0.3);
        }
    }
    if let Some(mut r) = p.reloading.take() {
        r.time += dt;
        let def = &l.weapons[r.weapon].def;
        let clip = l.reload_clips[stance_of(l, r.weapon)].as_ref();
        let duration = clip.map_or(def.reload_time.max(RELOAD_TIME), |c| c.duration);
        let mag_in = clip.and_then(|c| c.event(EV_MAG_IN)).unwrap_or(duration * 0.8);
        if !r.filled && r.time >= mag_in {
            r.filled = true;
            p.ammo[r.weapon][0] = r.fill;
            if def.reload_sound != 0 {
                p.sound_queue.push((def.reload_sound, 0.8));        // lasers have none
            }
        }
        if r.time < duration {
            p.reloading = Some(r);
        }
    }
    // ---- a medkit: the stance's use_item clip; in the hand from its 0a6e8f79 event, used (and
    // dropped) at its 19f8311b event
    p.item_used = false;
    if let Some(t0) = p.using.take() {
        let t = t0 + dt;
        let clip = l.use_clips[stance_of(l, p.weapon)].as_ref();
        let duration = clip.map_or(1.0, |c| c.duration);
        let grab = clip.and_then(|c| c.event(EV_ITEM_IN_HAND)).unwrap_or(duration * 0.2);
        let used = clip.and_then(|c| c.event(EV_ITEM_USED)).unwrap_or(duration * 0.8);
        p.item_in_hand = t >= grab && t < used;
        if t0 < used && t >= used {
            p.item_used = true;
        }
        if t < duration && !p.dead {
            p.using = Some(t);
        } else {
            p.item_in_hand = false;
        }
    }
    // ---- grenade: the selected type (play_grenade.rs). Thrown: the stance's throw clip; the
    // grenade is in the hand from the reach event and leaves it at the release event. Placed
    // (Roller, Sentry): set down at the feet at the press, no meter
    let kind = match p.item { Item::Grenade(k) if p.grenades.get(k).is_some_and(|&n| n > 0) => kits.get(k).map(|kit| (k, kit)), _ => None };
    let can_throw = kind.is_some() && l.throw_hand.is_some() && p.throwing.is_none()
        && p.reloading.is_none() && p.using.is_none() && p.switching.is_none() && matches!(p.action, Action::None) && !p.on_all_fours;
    p.meter_after.1 = (p.meter_after.1 - dt).max(0.0);
    if !p.throw_held {
        p.place_latch = false;
    }
    if let (Some((k, kit)), true) = (kind, kind.is_some_and(|(_, kit)| kit.placed())) {
        // the press sets it down: the count drops then, with the item's event sound
        let slot = stance_of(l, p.weapon);
        if p.throw_held && can_throw && !p.place_latch && l.place_clips[slot].is_some() {
            p.place_latch = true;
            p.grenades[k] -= 1;
            if kit.def.arm_sound != 0 {
                p.sound_queue.push((kit.def.arm_sound, 0.8));
            }
            p.throwing = Some(Throw { power: 0.0, time: 0.0, slot, grabbed: false, released: false, kind: k, place: true });
        }
        p.charge = 0.0;
    } else if p.throw_held && can_throw {
        // hold to charge (the HUD meter), let go to throw: the charge sets how far it goes
        if p.charge == 0.0 {
            // the item's event sound starts the gauge (capture: heard as the meter appears)
            if let Some(s) = kind.map(|(_, kit)| kit.def.arm_sound).filter(|&s| s != 0) {
                p.sound_queue.push((s, 0.8));
            }
        }
        p.charge = (p.charge + dt / CHARGE_TIME).min(1.0);
    } else if p.charge > 0.0 {
        let slot = stance_of(l, p.weapon);
        if let (true, true, Some((k, _))) = (can_throw, l.throw_clips[slot].is_some(), kind) {
            p.throwing = Some(Throw { power: p.charge.max(0.1), time: 0.0, slot, grabbed: false, released: false, kind: k,
                                      place: false });
            p.aim_hold = 0.0;
            // the count drops at the button (one game frame after it in the recordings), the
            // meter holds its level a moment
            p.grenades[k] -= 1;
            p.meter_after = (p.charge, METER_HOLD + METER_FADE);
        }
        p.charge = 0.0;
    }
    if let Some(mut t) = p.throwing.take() {
        t.time += dt;
        let (clip, let_go) = if t.place { (&l.place_clips[t.slot], EV_PLACED) } else { (&l.throw_clips[t.slot], EV_RELEASE) };
        let (reach, release, duration) = clip.as_ref()
            .map_or((0.0, 0.0, 0.0), |c| (c.event(EV_REACH).unwrap_or(c.duration * 0.2), c.event(let_go).unwrap_or(c.duration * 0.55), c.duration));
        t.grabbed |= t.time >= reach;
        if !t.released && t.time >= release {
            t.released = true;
            p.pending_release = Some((t.power, t.kind, t.place));
        }
        if t.time < duration {
            p.throwing = Some(t);
        }
    }
    p.cooldown -= dt;
    p.aim_hold -= dt;
    let can_fire = armed && p.reloading.is_none() && p.throwing.is_none() && p.ammo.get(p.weapon).is_some_and(|a| a[0] > 0)
        && p.switching.is_none() && p.holding && !matches!(p.action, Action::Dodge { .. }) && !p.on_all_fours;
    if p.fire && can_fire {
        if p.aim_hold <= 0.0 && !p.aim {
            p.cooldown = p.cooldown.max(0.2);             // raise the gun before the first shot
        }
        p.aim_hold = 0.7;
        if p.cooldown <= 0.0 {
            let def = &l.weapons[p.weapon].def;
            p.cooldown = (p.cooldown + 1.0 / def.rate.max(0.2)).max(0.02);
            let (sounds, rate) = (def.fire_sounds.clone(), def.rate);
            // one sound per shot (each fire sound is a single report); fast guns overlap several
            // (the minigun's ~0.4 s sounds at 15/s), so they play a little quieter
            p.queue_any(&sounds, if rate >= 8.0 { 0.55 } else { 0.8 });
            p.shots_fired += 1;
            p.ammo[p.weapon][0] -= 1;
            p.pending_shot = true;
            p.recoil = 1.0;
            p.flash = 0.05;
        }
    } else {
        p.cooldown = p.cooldown.max(0.0);
    }

    // ---- sliding down steep ground: the slide clip, facing downhill ----
    let sliding = p.sliding > 0.0 && matches!(p.action, Action::None) && c.slide.is_some() && !p.on_all_fours;
    if sliding != p.slide_on && std::env::var("BF_SLIDE_LOG").is_ok() {
        println!("t={:.2} {} slide {} at {:.1}", p.sim_time, CHARACTERS[p.character], if sliding { "starts" } else { "ends" }, p.position);
    }
    if sliding && !p.slide_on {
        p.queue_any(&surface.slide, 0.8);
    }
    p.slide_on = sliding;

    // ---- gait, facing and upper-body twist (on the ground, outside actions) ----
    // aiming points the gun, not the chest: the held weapon's yaw in the animated pose (muzzle_off)
    // is subtracted, so a gun carried across the body still lines up with the crosshair
    let aim_yaw = p.cam_yaw;
    let off = if armed { p.muzzle_off } else { 0.0 };
    let upper_aim = p.aim || (p.aim_hold > 0.0 && armed);
    let mut twist_goal = 0.0;
    if sliding {
        let down = p.slide_yaw;
        turn_to(&mut p.yaw, down, TURN_RATE);
        p.gait = Gait::Idle;
    } else if matches!(p.action, Action::None) || (matches!(p.action, Action::Crouched) && stance.cr_walk.is_some()) {
        p.gait = if p.aim {
            if !moving {
                let goal = stand_aim_yaw(p, aim_yaw - off);
                turn_to(&mut p.yaw, goal, TURN_RATE);
                Gait::Idle
            } else if let Some(side) = side_step(wrap_angle(dir_yaw - aim_yaw), if matches!(p.action, Action::Crouched) { &stance.cr_side } else { &stance.rp_side }, armed) {
                // stepping sideways: the side-walk clips twist the upper body about a quarter
                // turn off the root, whose motion runs straight along it (forward steps forward,
                // back steps back, as far as the walks). So the character turns until the gun is
                // on the crosshair (its measured yaw, as standing), and walks where the clip takes
                // it: across the aim, a little forward or back
                turn_to(&mut p.yaw, aim_yaw - off, TURN_RATE);
                side
            } else if wrap_angle(dir_yaw - aim_yaw).abs() > 2.0 {
                turn_to(&mut p.yaw, wrap_angle(dir_yaw + std::f32::consts::PI), TURN_RATE);
                if p.sprint { Gait::RunBack } else { Gait::WalkBack }
            } else {
                turn_to(&mut p.yaw, dir_yaw, TURN_RATE);
                if p.sprint { Gait::Run } else { Gait::Walk }
            }
        } else if moving {
            turn_to(&mut p.yaw, dir_yaw, TURN_RATE);
            if p.sprint { Gait::Sprint } else if p.walk { Gait::Walk } else { Gait::Run }
        } else {
            if upper_aim {
                let goal = stand_aim_yaw(p, aim_yaw - off);
                turn_to(&mut p.yaw, goal, TURN_RATE);
            } else if let Some(f) = p.face_yaw {
                turn_to(&mut p.yaw, f, TURN_RATE * 0.35);      // squadmates settle facing the leader's way
            }
            Gait::Idle
        };
        if upper_aim && !armed {
            twist_goal = wrap_angle(aim_yaw - p.yaw).clamp(-1.75, 1.75);
        }
        // crouched there's only the crouch walk: forward, back or sideways
        if matches!(p.action, Action::Crouched) {
            p.gait = match p.gait {
                Gait::Idle => Gait::Idle,
                Gait::WalkBack | Gait::RunBack => Gait::WalkBack,
                Gait::Side { left, back } => Gait::Side { left, back },
                _ => Gait::Walk,
            };
        }
        p.on_all_fours = p.gait == Gait::Sprint;
    } else if p.action.airborne() && moving {
        turn_to(&mut p.yaw, dir_yaw, TURN_RATE * 0.3);      // a little air steering
    }
    let k = 1.0 - (-10.0 * dt).exp();
    p.twist += (twist_goal - p.twist) * k;
    // a standing throw turns to where the crosshair is
    if p.throwing.is_some() && !moving && matches!(p.action, Action::None) {
        turn_to(&mut p.yaw, aim_yaw, TURN_RATE);
    }

    // weapon up (ready pose) while aiming or firing, carried otherwise
    // the held weapon's slot picks the locomotion set (arms carry that gun); its ready pose
    // while aiming or firing. During a switch the set changes when the new gun is grabbed.
    let set = &c.slots[if armed { stance_of(l, p.weapon) } else { 0 }];
    let lo = if upper_aim && armed && set.rp.idle.is_some() && set.rp.run.is_some() { &set.rp } else { &set.carry };
    let gait_clip = match p.gait {
        Gait::Idle => lo.idle,
        Gait::Walk => lo.walk,
        Gait::Run => lo.run,
        Gait::Sprint => lo.sprint.or(lo.run),
        Gait::WalkBack => lo.walk_back.or(lo.run_back),
        Gait::RunBack => lo.run_back.or(lo.walk_back),
        Gait::Side { left, back } => set.rp_side[side_index(left, back)].or(lo.walk),
    }.or(lo.idle);
    let (action_clip, once) = match p.action {
        Action::None => (if sliding { c.slide } else { None }, false),
        Action::Dodge { left } => (if left { c.loco.dodge_left } else { c.loco.dodge_right }, true),
        Action::JumpCrouch => (c.jump_crouch, true),
        Action::JumpLaunch => (if four { c.leg4_launch } else { c.jump_launch }, true),
        Action::JumpFall => (if four { c.leg4_fall } else { c.jump_fall }, false),
        Action::JumpLand => (if four { c.leg4_land } else { c.jump_land }, true),
        Action::Crouching => (stance.stand2crouch, true),
        Action::Crouched => (match p.gait {
            Gait::Walk => stance.cr_walk,
            Gait::WalkBack => stance.cr_back_walk.or(stance.cr_walk),
            Gait::Side { left, back } => stance.cr_side[side_index(left, back)].or(stance.cr_walk),
            _ => None,
        }.or(stance.crouch_idle), false),
        Action::Rising => (stance.crouch2stand, true),
        Action::Dive => (stance.dive, true),
    };
    let Some(want) = action_clip.or(gait_clip) else { return };

    // ---- crossfade stack ----
    if p.layers.last().map(|x| x.clip) != Some(want) {
        // loops start in phase with the current clip (keeps feet in step), one-shots from 0
        let phase = p.layers.last().map(|x| x.time / l.model.anims[x.clip].duration.max(1e-3)).unwrap_or(0.0);
        let first = p.layers.is_empty();
        let time = if once { 0.0 } else { phase * l.model.anims[want].duration };
        p.layers.push(Layer { clip: want, time, weight: if first { 1.0 } else { 0.0 }, once });
    }
    let n = p.layers.len();
    let top = (p.layers[n - 1].weight + dt / FADE).min(1.0);
    let rest: f32 = p.layers[..n - 1].iter().map(|x| x.weight).sum();
    for (i, layer) in p.layers.iter_mut().enumerate() {
        layer.weight = if i == n - 1 { top } else if rest > 0.0 { layer.weight / rest * (1.0 - top) } else { 0.0 };
    }
    p.layers.retain(|x| x.weight > 1e-3);

    // ---- advance clips: root motion and poses ----
    let mut delta = Vec3::ZERO;
    let mut poses = vec![];
    p.face_time += dt;
    for layer in p.layers.iter_mut() {
        let d = l.model.anims[layer.clip].duration.max(1e-3);
        if layer.once {
            let step = (d - layer.time).clamp(0.0, dt);
            delta += locomotion::root_delta(&l.model, game, layer.clip, layer.time, step) * layer.weight;
            layer.time = (layer.time + dt).min(d - 1e-3);
        } else {
            delta += locomotion::root_delta(&l.model, game, layer.clip, layer.time, dt) * layer.weight;
            layer.time = (layer.time + dt) % d;
        }
        let mut pose = l.model.pose(game, layer.clip, layer.time, l.model.default_face, p.face_time);
        pose[0].1.x = 0.0;                                   // root motion moves the entity instead
        pose[0].1.z = 0.0;
        poses.push((pose, layer.weight));
    }
    let mut pose = locomotion::blend(&poses);
    // the lift for what's playing: the crouch clips' own, the stance's otherwise
    let (mut lift, mut weight) = (0.0, 0.0);
    for layer in &p.layers {
        let here = match l.crouch_lift.get(&layer.clip) {
            Some(t) if t.len() > 1 => {
                let d = l.model.anims[layer.clip].duration.max(1e-3);
                let x = (layer.time / d).clamp(0.0, 1.0) * (t.len() - 1) as f32;
                let (i, f) = (x.floor() as usize, x.fract());
                t[i] + (t[(i + 1).min(t.len() - 1)] - t[i]) * f
            }
            Some(t) => t[0],
            None => l.lift,
        };
        lift += here * layer.weight;
        weight += layer.weight;
    }
    p.lift_now = if weight > 0.0 { lift / weight } else { l.lift };
    // weapon switch: the overlay clip on its bones (upper body), faded in and out over 0.2 s
    if let Some(sw) = &p.switching {
        if let Some(clip) = &l.switch_clips[sw.to] {
            let t = sw.time.min(clip.duration - 1e-3);
            let k = (t / FADE).min((clip.duration - t) / FADE).clamp(0.0, 1.0);
            let over = l.model.pose(game, clip.clip, t, l.model.default_face, p.face_time);
            for (b, m) in clip.mask.iter().enumerate() {
                if *m {
                    let (q0, t0) = pose[b];
                    let q1 = if q0.dot(over[b].0) < 0.0 { -over[b].0 } else { over[b].0 };
                    pose[b] = (q0.lerp(q1, k).normalize(), t0.lerp(over[b].1, k));
                }
            }
        }
    }
    // reload and throw: whole body standing still, upper body on the move
    let standing = !moving && matches!(p.action, Action::None);
    if let Some(r) = &p.reloading {
        if let Some(c) = &l.reload_clips[stance_of(l, r.weapon)] {
            apply_overlay(&mut pose, &l.model, game, c, r.time, standing, p.face_time);
        }
    }
    if let Some(t) = p.using {
        if let Some(c) = &l.use_clips[stance_of(l, p.weapon)] {
            apply_overlay(&mut pose, &l.model, game, c, t, standing, p.face_time);
        }
    }
    if let Some(t) = &p.throwing {
        if let Some(c) = if t.place { &l.place_clips[t.slot] } else { &l.throw_clips[t.slot] } {
            apply_overlay(&mut pose, &l.model, game, c, t.time, standing, p.face_time);
        }
    }
    if let Some(w) = l.weapons.get(p.weapon) {
        if let Some((hb, r, _)) = w.in_hand {
            let d = l.model.world(&pose)[hb].transform_vector3(r * w.fire_dir);
            let off = (-d.x).atan2(-d.z);
            p.muzzle_off = wrap_angle(p.muzzle_off + wrap_angle(off - p.muzzle_off) * (1.0 - (-8.0 * dt).exp()));
        }
    }
    // aim the gun itself: yaw and pitch from the barrel in the animated pose to what the crosshair
    // is on, turned over the spine bones that carry the trigger arm
    let held = l.weapons.get(p.weapon).and_then(|w| w.in_hand.map(|h| (w, h)));
    let want_aim = if upper_aim && armed && p.switching.is_none() && p.reloading.is_none() && p.throwing.is_none() && p.using.is_none() { 1.0 } else { 0.0 };
    p.aim_weight += (want_aim - p.aim_weight) * (1.0 - (-10.0 * dt).exp());
    match held {
        Some((w, (hb, r, t))) if p.aim_weight > 0.01 && !l.arm_chain.is_empty() => {
            let (cam, ray) = aim_ray(p);
            let mut hit = cam + ray * ray_hit(cam, ray, 150.0).unwrap_or(80.0);
            let heading = Vec3::new(-p.cam_yaw.sin(), 0.0, -p.cam_yaw.cos());
            let reach = (hit - p.position).dot(heading);
            if reach < MIN_AIM_REACH {
                hit += heading * (MIN_AIM_REACH - reach);
            }
            let root = Transform::from_translation(p.position + Vec3::Y * p.height).with_rotation(Quat::from_rotation_y(p.yaw));
            let target = root.compute_matrix().inverse().transform_point3(hit);
            let yaw_of = |d: Vec3| (-d.x).atan2(-d.z);
            let pitch_of = |d: Vec3| d.y.clamp(-1.0, 1.0).asin();
            // turning the spine also moves the muzzle, so correct twice (the second pass is small)
            for pass in 0..2 {
                let world = l.model.world(&pose);
                let muzzle = world[hb].transform_point3(r * w.muzzle.point + t);
                let barrel = world[hb].transform_vector3(r * w.fire_dir).normalize();
                let want = (target - muzzle).normalize();
                let weight = if pass == 0 { p.aim_weight } else { p.aim_weight.powi(4) };
                let full = wrap_angle(yaw_of(want) - yaw_of(barrel)).clamp(-1.9, 1.9);
                if pass == 0 {
                    p.aim_residual = full;
                }
                let dyaw = full * weight;
                locomotion::turn(&l.model, &mut pose, &l.arm_chain, Vec3::Y, dyaw);
                let heading = yaw_of(barrel) + dyaw;
                let right = Vec3::new(heading.cos(), 0.0, -heading.sin());
                let dpitch = (pitch_of(want) - pitch_of(barrel)).clamp(-0.8, 0.8) * weight;
                locomotion::turn(&l.model, &mut pose, &l.arm_chain, right, dpitch);
            }
        }
        _ => {
            p.aim_residual = 0.0;
            locomotion::twist(&l.model, &mut pose, &l.aim_chain, p.twist)
        }
    }

    // ---- move: root motion on the ground, ballistic in the air ----
    let facing = Quat::from_rotation_y(p.yaw);
    if p.action.airborne() {
        p.air_velocity *= AIR_DRAG.powf(dt * 30.0);
        p.position += p.air_velocity * dt;
        p.height += p.vy * dt;
        p.vy -= GRAVITY * dt;
        if p.height <= 0.0 && p.vy < 0.0 {
            p.height = 0.0;
            p.vy = 0.0;
            p.action = Action::JumpLand;
            p.action_left = dur(if four { c.leg4_land } else { c.jump_land });
            p.queue_any(&surface.jump_land, 0.9);
            p.step_mute = 0.3;
        }
    } else {
        let step = facing * Vec3::new(delta.x, 0.0, delta.z);
        p.position += step;
        if dt > 0.0 {
            p.last_velocity = step / dt;
        }
    }
    if let Ok(mut tr) = transforms.get_mut(l.root) {
        tr.translation = p.position + Vec3::Y * (p.height + p.lift_now);
        tr.rotation = facing;
    }
    for (e, (q, t)) in l.joints.iter().zip(&pose) {
        if let Ok(mut tr) = transforms.get_mut(*e) {
            tr.rotation = *q;
            tr.translation = *t;
        }
    }

    let world = l.model.world(&pose);
    p.last_world.clone_from(&world);

    // ---- grenade release: from the throwing hand, toward the crosshair with a lob ----
    if let Some((power, kind, placed)) = p.pending_release.take() {
        if let Some((bone, point)) = l.throw_hand {
            let root = Transform::from_translation(p.position + Vec3::Y * p.height).with_rotation(facing).compute_matrix();
            let pos = (root * world[bone]).transform_point3(point);
            let (_, ray) = aim_ray(p);
            let carry = Vec3::new(p.last_velocity.x, 0.0, p.last_velocity.z);
            // a placed one is let go from the hand and drops at the feet
            let velocity = if placed { carry } else { grenade::throw_velocity(ray, power) + carry };
            p.thrown.push(grenade::Thrown { pos, velocity, kind, landed: false });
        }
    }

    // ---- shot: from the muzzle to whatever the crosshair is on ----
    if std::mem::take(&mut p.pending_shot) {
        if let Some(w) = l.weapons.get(p.weapon) {
            if let Some((hb, r, t)) = w.in_hand {
                let root = Transform::from_translation(p.position + Vec3::Y * p.height).with_rotation(facing).compute_matrix();
                let hand = root * world[hb];
                let barrel = hand.transform_vector3(r * w.fire_dir).normalize();
                let (cam, ray) = aim_ray(p);
                // in the scope the shot leaves from the eye along the crosshair's ray (the hidden
                // muzzle sits ~0.4 m lower: through a 5-10x zoom its line lands visibly low)
                let scoped = p.scope > 0.5;
                let origin = if scoped { cam + ray * 0.3 } else { hand.transform_point3(r * w.muzzle.point + t) };
                let range = w.def.range.max(20.0);
                let target = cam + ray * ray_hit(cam, ray, range + 10.0).unwrap_or(range);
                let mut dir = if scoped { ray } else { (target - origin).normalize_or_zero() };
                if dir.dot(ray) < 0.3 || dir == Vec3::ZERO {
                    dir = if upper_aim { ray } else { barrel };
                }
                let hit = ray_hit(origin, dir, range);
                // speed 1 marks instant-hit guns (their tracer effect flies on its own); the
                // others' bolts fly at the bullet's speed
                let speed = if w.def.bullet_speed < 10.0 { INSTANT_SHOT } else { w.def.bullet_speed };
                p.shots.push(Shot { origin, dir, dist: hit.unwrap_or(range), hit: hit.is_some(), speed,
                                    flight: w.def.flight_effect, hit_fx: w.def.hit_effect });
                if std::env::var("BF_SHOT_LOG").is_ok() {
                    println!("t={:.2} yaw {:.0} aim {:.0} off {:.0} twist {:.0} residual {:.0}  barrel {:.2} ray {:.2} dir {:.2} origin {:.2} dist {:.1}",
                             p.sim_time, p.yaw.to_degrees(), p.cam_yaw.to_degrees(), p.muzzle_off.to_degrees(), p.twist.to_degrees(), p.aim_residual.to_degrees(),
                             barrel, ray, dir, origin - p.position, hit.unwrap_or(range));
                }
            }
        }
    }

    // ---- footsteps: a foot coming down to its planted height ----
    p.step_mute -= dt;
    let walking = !p.action.airborne() && p.step_mute <= 0.0 && !matches!(p.action, Action::JumpLand | Action::JumpCrouch) && (p.gait != Gait::Idle || matches!(p.action, Action::Dodge { .. }));
    let volume = match p.gait { Gait::Walk | Gait::WalkBack | Gait::Side { .. } => 0.35, Gait::Sprint => 0.85, _ => 0.6 };
    let steps = surface.footsteps.get(&l.footstep_type).cloned().unwrap_or_default();
    for f in 0..2 {
        // planted level and swing height of the current clip blend (idle stance otherwise)
        let (mut low, mut swing, mut total) = (0.0, 0.0, 0.0);
        for layer in &p.layers {
            if let Some(r) = l.foot_range.get(&layer.clip) {
                low += r[f].0 * layer.weight;
                swing += (r[f].1 - r[f].0) * layer.weight;
                total += layer.weight;
            }
        }
        let (low, swing) = if total > 1e-3 { (low / total, swing / total) } else { (l.foot_rest[f], 0.0) };
        let y = world[l.feet[f]].w_axis.y - low;
        let contact = FOOT_CONTACT.max(0.25 * swing);
        let y = y - contact + FOOT_CONTACT;                   // contact band starts at FOOT_CONTACT
        if walking && y < FOOT_CONTACT && p.foot_prev[f] >= FOOT_CONTACT {
            p.queue_any(&steps, volume);
        }
        p.foot_prev[f] = y;
    }
}

/// Decode queued sound ids (cached as in-memory WAVs) and play them.
fn play_sounds(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               mut cache: ResMut<SoundCache>, mut sources: ResMut<Assets<AudioSource>>) {
    // squad members' sounds join the player's, quieter with distance
    let here = player.position;
    for m in squad.0.iter_mut() {
        let k = (1.0 - m.position.distance(here) / 35.0).clamp(0.15, 1.0) * 0.8;
        let queued: Vec<(u32, f32)> = m.sound_queue.drain(..).map(|(id, v)| (id, v * k)).collect();
        player.sound_queue.extend(queued);
    }
    if std::env::var("BF_SOUND_LOG").is_ok() {
        for (id, volume) in &player.sound_queue {
            println!("t={:6.2} sound {id:08x} vol {volume:.2} action {:?} height {:.2}", player.sim_time, player.action, player.height);
        }
    }
    if std::env::var("BF_MUTE").is_ok() {
        player.sound_queue.clear();
        return;
    }
    let queue: Vec<(u32, f32)> = player.sound_queue.drain(..).collect();
    for (id, volume) in queue {
        let handle = cache.0.entry(id).or_insert_with(|| {
            game.0.sounds.wav(id).map(|w| sources.add(AudioSource { bytes: Arc::from(w.into_boxed_slice()) }))
        });
        match handle.clone() {
            Some(h) => {
                commands.spawn((AudioPlayer::new(h), PlaybackSettings { volume: Volume::Linear(volume), ..PlaybackSettings::DESPAWN }));
            }
            None => if std::env::var("BF_SOUND_LOG").is_ok() {
                println!("  sound {id:08x}: no sample data in the loaded banks");
            }
        }
    }
}

/// Capture frames still warming up (shaders compiling); the simulation waits for them.
static WARMUP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(60);

/// Capture's fixed step: 15 per second, or BF_CAPTURE_FPS.
fn capture_step() -> f32 {
    1.0 / std::env::var("BF_CAPTURE_FPS").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(15.0).max(1.0)
}

fn frame_dt(time: &Time) -> f32 {
    if std::env::var("BF_CAPTURE").is_ok() {
        if WARMUP.load(std::sync::atomic::Ordering::Relaxed) > 0 { 0.0 } else { capture_step() }
    } else {
        time.delta_secs().min(0.1)
    }
}

fn follow_camera(time: Res<Time>, mut player: ResMut<Player>, mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
                 mut vis: Query<&mut Visibility>) {
    let dt = frame_dt(&time);
    // the scope: aiming (held, not just after firing) a weapon that zooms
    let held_zoom = player.loaded.as_ref().and_then(|l| l.weapons.get(player.weapon)).map_or(0.0, |w| w.def.zoom);
    let busy = player.switching.is_some() || player.reloading.is_some() || player.dead;
    if busy {
        player.scope_level = 0;
    }
    let scoped = player.aim && held_zoom > 1.0 && !busy;
    // Flint's second step doubles the weapon's zoom
    let held_zoom = if player.scope_level >= 2 { held_zoom * SNIPER_SECOND_STEP } else { held_zoom };

    let want = if scoped { 1.0 } else { 0.0 };
    player.scope += (want - player.scope) * (1.0 - (-SCOPE_RATE * dt).exp());
    if (player.scope - want).abs() < 0.01 {
        player.scope = want;
    }
    // (eased, also between Flint's steps)
    let target = 1.0 + (held_zoom.max(1.0) - 1.0) * player.scope;
    player.zoom += (target - player.zoom) * (1.0 - (-SCOPE_RATE * dt).exp());
    if (player.zoom - target).abs() < 0.01 {
        player.zoom = target;
    }
    // the aim's wander in the scope (none for Flint)
    let t = player.sim_time;
    let wander = Vec2::new((0.9 * t).sin() + 0.45 * (2.3 * t + 0.7).sin(), 0.8 * (1.3 * t + 1.1).sin() + 0.35 * (2.9 * t).sin());
    player.sway = if player.character == STILL_SNIPER { Vec2::ZERO } else { wander * SWAY * player.scope };
    // going in and out: the character's scope sounds
    if scoped != player.was_scoped {
        player.was_scoped = scoped;
        let id = if scoped { player.scope_sounds.0 } else { player.scope_sounds.1 };
        if id != 0 {
            player.sound_queue.push((id, SCOPE_SOUND_VOLUME));
        }
    }
    // its breathing, in a loop while scoped
    if player.scope > 0.9 {
        player.breath_in -= dt;
        if player.breath_in <= 0.0 {
            if let Some((id, len)) = player.snipe_sound {
                player.sound_queue.push((id, BREATH_VOLUME));
                player.breath_in = len + BREATH_GAP;
            }
        }
    } else {
        player.breath_in = 0.0;
    }
    if let Some(root) = player.loaded.as_ref().map(|l| l.root) {
        if let Ok(mut v) = vis.get_mut(root) {
            let want = if player.scope > 0.6 { Visibility::Hidden } else { Visibility::Inherited };
            if *v != want {
                *v = want;
            }
        }
    }
    let goal = player.position + Vec3::new(0.0, 0.35 + player.height * 0.5, 0.0);
    let k = 1.0 - (-12.0 * dt).exp();
    player.cam_target = player.cam_target.lerp(goal, k);
    // in close while aiming or firing: the game snaps in within ~0.1 s of the first shot, holds
    // briefly after the last one (aim_hold), then drifts back out over ~2 s
    let armed = player.loaded.as_ref().is_some_and(|l| !l.weapons.is_empty());
    let want = if player.aim || (armed && player.aim_hold > 0.0) { 1.0 } else { 0.0 };
    let rate = if want > player.shoulder { 25.0 } else { 1.6 };
    player.shoulder += (want - player.shoulder) * (1.0 - (-rate * dt).exp());
    // test hook: BF_VIEW_YAW turns only the rendered view (aim and controls are unchanged)
    let view = std::env::var("BF_VIEW_YAW").ok().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let (mut pos, look) = camera_pose(&player, view);
    let back = pos - look;
    if let Some(t) = world::arena().and_then(|a| a.ray(look, back.normalize_or_zero(), back.length())) {
        pos = look + back.normalize_or_zero() * (t - 0.3).max(0.2);
    }
    if let Ok((mut tr, mut projection)) = cam.single_mut() {
        *tr = Transform::from_translation(pos).looking_at(look, Vec3::Y);
        if let Projection::Perspective(pp) = projection.as_mut() {
            let fov = fov_at(player.zoom);
            if (pp.fov - fov).abs() > 1e-4 {
                pp.fov = fov;
            }
        }
    }
}

fn update_hud(player: Res<Player>, game: Res<GameData>, mut hud: Query<&mut Text, With<Hud>>, test: Option<Res<testmap::TestMap>>) {
    let Ok(mut text) = hud.single_mut() else { return };
    // the controls panel is the test map's (cargo run --bin bf_play -- --test)
    let Some(test) = test else {
        text.0.clear();
        return;
    };
    if std::env::var("BF_NO_HUD").is_ok() {
        text.0.clear();
        return;
    }
    if !player.show_help {
        if text.0 != "H  controls" {
            text.0 = "H  controls".into();
        }
        return;
    }
    let state = match player.action {
        Action::None => match player.gait {
            Gait::Idle => "idle", Gait::Walk => "walk", Gait::Run => "run", Gait::Sprint => "sprint",
            Gait::WalkBack => "walk back", Gait::RunBack => "backpedal",
            Gait::Side { left: true, back: false } => "step left", Gait::Side { left: false, back: false } => "step right",
            Gait::Side { left: true, back: true } => "step back left", Gait::Side { left: false, back: true } => "step back right",
        },
        Action::Dodge { left: true } => "dodge left",
        Action::Dodge { left: false } => "dodge right",
        Action::JumpCrouch => "jump: crouch",
        Action::JumpLaunch => "jump: launch",
        Action::JumpFall => "jump: fall",
        Action::JumpLand => "jump: land",
        Action::Crouching => "kneel",
        Action::Crouched => match player.gait {
            Gait::Idle => "kneeling",
            Gait::Side { .. } => "crouch side step",
            _ => "crouch walk",
        },
        Action::Rising => "stand up",
        Action::Dive => "dive",
    };
    let clip = player.layers.last().map(|l| format!("clip {}", l.clip)).unwrap_or_default()
        + if player.switching.is_some() { "   [switching weapon]" } else { "" };
    let surf = game.0.surfaces.get(player.surface).map(|s| format!("surface {} (id {})", player.surface, s.id)).unwrap_or_default();
    let weapon = player.loaded.as_ref().and_then(|l| l.weapons.get(player.weapon).map(|w| format!("   weapon: {} ({}/{})",
        w.def.label, player.weapon + 1, l.weapons.len()))).unwrap_or_default();
    let s = format!(
        "{}   {}   {}{}   {}{}\n\
         WASD move   Shift sprint   Ctrl walk   Space jump   C dodge   Z crouch   Right mouse aim   Left mouse fire   Q switch weapon   R reload   G use item   Tab items   T grenade type   E use   M surface   H hide\n\
         click: mouse look, Esc release   wheel zoom   1-4 take control of Brutus / Flint / Hawk / Tex   G hold to charge a grenade\n\
         test map: walk into a weapon on the rack to take it into the held slot   K instant kill: {}   X die",
        CHARACTERS[player.character], state, clip, if player.aim { "   [aiming]" } else { "" }, surf, weapon,
        if test.instant_kill { "on" } else { "off" });
    if text.0 != s {
        text.0 = s;
    }
}

/// Weapon placement (hand / back), recoil, minigun spin, muzzle flash, and tracers for new shots,
/// for the player and the squad.
fn update_weapons(
    mut commands: Commands,
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    fx: Res<Fx>,
    mut transforms: Query<&mut Transform, Without<Tracer>>,
    mut visibility: Query<&mut Visibility>,
    mut lights: Query<&mut PointLight>,
    globals: Query<&GlobalTransform>,
) {
    let dt = frame_dt(&time);
    for p in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let Some(mut l) = p.loaded.take() else { continue };
        // the dead let go of the gun in their hand: it falls from where it is, thrown a little
        // by what killed them, and tumbles like any loose object (see pickups::Thrown)
        // (and the one on their back: it would prop the body up, and the ragdoll can't feel it)
        if p.dead && !l.weapon_dropped {
            l.weapon_dropped = true;
            for w in &l.weapons {
                if visibility.get(w.entity).is_ok_and(|v| *v == Visibility::Hidden) {
                    continue;
                }
                if let Ok(g) = globals.get(w.entity) {
                    let push = p.death_push * DROP_PUSH + Vec3::Y * DROP_HOP;
                    let spin = Vec3::new(p.random(100) as f32 - 50.0, p.random(100) as f32 - 50.0, p.random(100) as f32 - 50.0) * 0.1;
                    commands.entity(w.entity).remove::<ChildOf>()
                        .insert((g.compute_transform(), pickups::Thrown { velocity: push, spin }));
                }
                if let Ok(mut v) = visibility.get_mut(w.flash) {
                    *v = Visibility::Hidden;
                }
                if let Ok(mut light) = lights.get_mut(w.light) {
                    light.intensity = 0.0;
                }
            }
            // (no longer held: nothing places them on the body again)
            p.holding = false;
        }
        weapon_fx(&mut commands, p, &l, &fx, &mut transforms, &mut visibility, &mut lights, dt);
        p.loaded = Some(l);
    }
}

/// A dead character's dropped gun: how much of what killed them it's thrown with, and its hop
/// (m/s). The demo's choice.
const DROP_PUSH: f32 = 0.5;
const DROP_HOP: f32 = 1.5;

/// One unit's weapons: placement, recoil, spin, flash, and tracers for its new shots.
#[allow(clippy::too_many_arguments)]
fn weapon_fx(commands: &mut Commands, p: &mut Player, l: &Loaded, fx: &Fx, transforms: &mut Query<&mut Transform, Without<Tracer>>,
             visibility: &mut Query<&mut Visibility>, lights: &mut Query<&mut PointLight>, dt: f32) {
    if std::mem::take(&mut p.weapon_dirty) {
        for (i, w) in l.weapons.iter().enumerate() {
            let place = if i == p.weapon && p.holding { w.in_hand } else { w.stowed };
            match place {
                Some((bone, r, t)) => {
                    commands.entity(w.entity).insert((ChildOf(l.joints[bone]), Transform::from_translation(t).with_rotation(r)));
                    if let Ok(mut v) = visibility.get_mut(w.entity) { *v = Visibility::Inherited; }
                }
                None => if let Ok(mut v) = visibility.get_mut(w.entity) { *v = Visibility::Hidden; },
            }
        }
    }
    p.recoil *= (-16.0 * dt).exp();
    let flash_on = p.flash > 0.0;
    p.flash -= dt;
    let firing = p.aim_hold > 0.5 && p.fire;
    if let Some(w) = l.weapons.get(p.weapon) {
        // recoil: kick the gun back along its barrel (only while it's in the hand: between a
        // switch's drop and grab it hangs from its stow point)
        if let (true, Some((_, r, t)), Ok(mut tr)) = (p.holding, w.in_hand, transforms.get_mut(w.entity)) {
            tr.translation = t - r * w.fire_dir * (0.04 * p.recoil);
            tr.rotation = r;
        }
        // minigun barrel spins up while firing
        let target = if firing { 28.0 } else { 0.0 };
        p.spin += (target - p.spin) * (1.0 - (-(if firing { 6.0 } else { 1.5 }) * dt).exp());
        p.spin_angle = (p.spin_angle + p.spin * dt) % std::f32::consts::TAU;
        for (e, axis, offset) in &w.spinners {
            if let Ok(mut tr) = transforms.get_mut(*e) {
                *tr = Transform::from_translation(*offset).with_rotation(Quat::from_axis_angle(*axis, p.spin_angle));
            }
        }
        let on = flash_on;
        if let Ok(mut v) = visibility.get_mut(w.flash) {
            *v = if on { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Ok(mut f) = transforms.get_mut(w.flash) {
            f.rotation = f.rotation * Quat::from_rotation_z(1.3);      // vary the flash shape
        }
        if let Ok(mut light) = lights.get_mut(w.light) {
            light.intensity = if on { 250_000.0 * bf_viewer::level_scene::POINT_LIGHT_SCALE } else { 0.0 };
        }
    }
    for s in p.shots.drain(..) {
        // the weapon's own effects (see `projectiles`), else a plain streak
        if s.flight != 0 || s.hit_fx != 0 {
            commands.spawn((Transform::from_translation(s.origin).looking_to(s.dir, Vec3::Y), Visibility::default(),
                            Projectile { shot: s, travelled: 0.0, started: false, wait: 2 }));
            continue;
        }
        let speed = s.speed.min(s.dist.max(1.0) / 0.16);
        let len = (speed * 0.03).clamp(0.8, 4.0).min(s.dist);
        commands.spawn((
            Tracer { dir: s.dir, origin: s.origin, travelled: -1.0, dist: s.dist, speed, len, hit: s.hit },
            Mesh3d(fx.tracer.clone()), MeshMaterial3d(fx.tracer_mat.clone()), NotShadowCaster,
            Transform::from_translation(s.origin).looking_to(s.dir, Vec3::Y).with_scale(Vec3::new(0.035, 0.035, 0.001)),
        ));
    }
}

/// A shot drawn with its weapon's effects: in flight it carries the flight effect (bolts move
/// with it; an instant-hit gun's tracer is fired once from the muzzle), and where it lands on
/// the world it plays the hit effect once.
#[derive(Component)]
struct Projectile {
    shot: Shot,
    travelled: f32,
    started: bool,
    wait: u8,
}

/// An effect entity that goes after this many seconds (its particles finish on their own).
#[derive(Component)]
struct AleExpire(f32);

/// Instant-hit guns: how fast the shot reaches its hit point (their tracer particle flies at
/// its own speed, 113.6 m/s for `tracer`).
const INSTANT_SHOT: f32 = 400.0;

/// The ALE effects of an effect type, compiled.
fn effects_of(game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
              materials: &mut Assets<StandardMaterial>, effect_type: u32) -> Vec<std::sync::Arc<bf_viewer::ale_fx::Compiled>> {
    let list = game.effect_types.get(&effect_type).cloned().unwrap_or_default();
    list.into_iter().filter_map(|e| ale.load(game, images, materials, e)).collect()
}

#[allow(clippy::too_many_arguments)]
fn projectiles(mut commands: Commands, time: Res<Time>, mut game: ResMut<GameData>, ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
               mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>,
               mut shots: Query<(Entity, &mut Projectile, &mut Transform)>, mut expiring: Query<(Entity, &mut AleExpire)>) {
    let dt = frame_dt(&time);
    for (e, mut x) in &mut expiring {
        x.0 -= dt;
        if x.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
    let Some(mut ale) = ale else { return };
    for (e, mut p, mut tr) in &mut shots {
        let s = p.shot;
        let instant = s.speed >= INSTANT_SHOT;
        if !p.started {
            p.started = true;
            let seed = (s.origin.x * 977.0 + s.origin.z * 131.0) as u32;
            for fx in effects_of(&mut game.0, &mut ale, &mut images, &mut materials, s.flight) {
                if instant {
                    // the tracer: fired once from the muzzle, flying on its own
                    let life = fx.duration();
                    commands.spawn((*tr, Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx, 0.0, seed), AleExpire(life)));
                } else {
                    // the bolt rides on the shot
                    commands.spawn((Transform::default(), Visibility::default(), bf_viewer::ale_fx::AleEffect::new(fx, 0.0, seed), ChildOf(e)));
                }
            }
        }
        // the first frames only start the effects (placed, then emitting): a bolt shows even
        // when its target is near
        if p.wait > 0 {
            p.wait -= 1;
            continue;
        }
        p.travelled += s.speed * dt;
        if p.travelled < s.dist {
            tr.translation = s.origin + s.dir * p.travelled;
            continue;
        }
        // landed: the hit effect on the world (bodies bleed instead, see play_fx)
        let (hit, at, dir, hit_fx) = (s.hit, s.origin + s.dir * s.dist, s.dir, s.hit_fx);
        commands.entity(e).despawn();
        // through a liquid's surface on the way: its splash there
        if let Some((surface, liquid)) = world::arena().and_then(|a| a.liquid_crossing(s.origin, at)) {
            let effect = liquid.effects[LIQUID_SHOT];
            if effect != 0 {
                spawn_once(&mut commands, &mut game.0, &mut ale, &mut images, &mut materials, effect, Transform::from_translation(surface));
            }
        }
        if hit {
            for fx in effects_of(&mut game.0, &mut ale, &mut images, &mut materials, hit_fx) {
                let life = fx.duration();
                commands.spawn((Transform::from_translation(at - dir * 0.05).with_rotation(Quat::from_rotation_arc(Vec3::Y, -dir)),
                                Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx, 0.0, 7), AleExpire(life)));
            }
        }
    }
}

/// Tracers fly to their hit point and leave a spark; sparks fade.
fn move_tracers(mut commands: Commands, time: Res<Time>, fx: Res<Fx>,
                mut tracers: Query<(Entity, &mut Tracer, &mut Transform), Without<Impact>>,
                mut impacts: Query<(Entity, &mut Impact, &mut Transform), Without<Tracer>>) {
    let dt = frame_dt(&time);
    for (e, mut t, mut tr) in &mut tracers {
        // first frame: just the segment leaving the muzzle
        t.travelled = if t.travelled < 0.0 { t.len } else { t.travelled + t.speed * dt };
        let head = t.travelled.min(t.dist);
        let tail = (t.travelled - t.len).max(0.0);
        if tail >= t.dist {
            commands.entity(e).despawn();
            if t.hit {
                commands.spawn((Impact(0.2), Mesh3d(fx.spark.clone()), MeshMaterial3d(fx.spark_mat.clone()), NotShadowCaster,
                                Transform::from_translation(t.origin + t.dir * t.dist).with_scale(Vec3::splat(0.14))));
            }
            continue;
        }
        tr.translation = t.origin + t.dir * ((head + tail) * 0.5);
        tr.scale.z = (head - tail).max(0.001);
    }
    for (e, mut i, mut tr) in &mut impacts {
        i.0 -= dt;
        if i.0 <= 0.0 {
            commands.entity(e).despawn();
        } else {
            tr.scale = Vec3::splat(0.14 * (i.0 / 0.2) + 0.03);
        }
    }
}

/// BF_CAPTURE=<dir> (use with BF_AUTOPILOT): fixed 15 fps steps, saves every frame as
/// <dir>/frame_NNN.png, then quits.
fn capture(mut commands: Commands, player: Res<Player>, mut frame: Local<usize>, mut exit: EventWriter<AppExit>) {
    let Ok(dir) = std::env::var("BF_CAPTURE") else { return };
    if player.loaded.is_none() {
        return;
    }
    let left = WARMUP.load(std::sync::atomic::Ordering::Relaxed);
    if left > 0 {
        WARMUP.store(left - 1, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    let frames: usize = std::env::var("BF_CAPTURE_FRAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(200);
    if *frame < frames {
        commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(format!("{dir}/frame_{:03}.png", *frame)));
    } else if *frame > frames + 20 {
        exit.write(AppExit::Success);
    }
    *frame += 1;
}

/// BF_DUMP_SOUNDS=<dir>: write the character's jump sound and the first surface's footstep,
/// landing and slide sounds as WAV files (for checking the decoder), then exit.
fn dump_sounds(game: &mut Game, character: usize, dir: &str) {
    let name = CHARACTERS[character];
    let (footstep, jump) = game.character_audio.get(name).copied().unwrap_or((-1, 0));
    println!("{name}: footstep type {footstep}, jump sound {jump:08x}; {} surfaces", game.surfaces.len());
    if let Ok(model) = Character::load(game, name, 0) {
        let clips = pick_clips(&model, game);
        if let Some(fi) = model.default_face {
            let f = &model.face_anims[fi];
            let idle = clips.loco.idle.unwrap_or(0);
            let base = model.pose(game, idle, 0.0, Some(fi), 0.0);
            let motion: Vec<String> = (0..24).map(|k| {
                let t = f.duration * k as f32 / 24.0;
                let p = model.pose(game, idle, 0.0, Some(fi), t);
                let d = p.iter().zip(&base).map(|(a, b)| a.0.angle_between(b.0)).fold(0.0, f32::max);
                format!("{:.0}", d.to_degrees())
            }).collect();
            println!("face clip {fi}/{} dur {:.2}s max bone deg over clip: {}", model.face_anims.len(), f.duration, motion.join(" "));
        }
        let (feet, rest) = find_feet(&model, game, clips.loco.idle);
        if let Some(run) = clips.loco.run {
            let d = model.anims[run].duration;
            let ys: Vec<[f32; 2]> = (0..40).map(|k| {
                let w = model.world(&model.pose(game, run, d * k as f32 / 40.0, None, 0.0));
                [w[feet[0]].w_axis.y - rest[0], w[feet[1]].w_axis.y - rest[1]]
            }).collect();
            for f in 0..2 {
                let v: Vec<f32> = ys.iter().map(|y| y[f]).collect();
                println!("foot {f} bone {:08x} rest {:.3} run rel min {:.3} max {:.3}", model.bones[feet[f]], rest[f],
                         v.iter().cloned().fold(f32::MAX, f32::min), v.iter().cloned().fold(f32::MIN, f32::max));
            }
        }
    }
    let mut out: Vec<(String, u32)> = vec![(format!("{name}_jump"), jump)];
    if let Some(s) = game.surfaces.iter().find(|s| s.footsteps.get(&footstep).is_some_and(|v| !v.is_empty())) {
        for (i, id) in s.footsteps[&footstep].iter().enumerate() {
            out.push((format!("surface{}_footstep{footstep}_{i}", s.id), *id));
        }
        for (i, id) in s.jump_land.iter().enumerate() {
            out.push((format!("surface{}_land_{i}", s.id), *id));
        }
        for (i, id) in s.slide.iter().enumerate() {
            out.push((format!("surface{}_slide_{i}", s.id), *id));
        }
    }
    std::fs::create_dir_all(dir).ok();
    for (label, id) in out {
        match game.sounds.pcm(id) {
            Some((rate, pcm)) => {
                let path = format!("{dir}/{label}_{id:08x}.wav");
                std::fs::write(&path, bf_viewer::bf::audio::wav_bytes(rate, &pcm)).ok();
                println!("{label:28} {id:08x} {rate} Hz {:5.2} s -> {path}", pcm.len() as f32 / rate as f32);
            }
            None => println!("{label:28} {id:08x} not found"),
        }
    }
}

/// BF_DUMP_WEAPONS=1: each character's weapons as loaded (definition, model, hardpoints, sounds).
fn dump_weapons(game: &Game) {
    println!("{} weapon definitions, {} strings", game.weapons.len(), game.strings.len());
    for name in CHARACTERS {
        let hand = Character::load(game, name, 0).ok().map(|m| {
            [weapon::TRIGGER_HAND, weapon::SUPPORT_HAND, weapon::STOW[0], weapon::STOW[1]].iter()
                .map(|k| format!("{k:08x}:{}", m.hardpoints.get(k).map(|(b, _)| format!("bone {:08x}", m.bones[*b])).unwrap_or("-".into())))
                .collect::<Vec<_>>().join(" ")
        }).unwrap_or_default();
        println!("{name}: hand/back hardpoints {hand}");
        for w in game.character_weapons.get(name).cloned().unwrap_or_default() {
            let Some(d) = game.weapons.get(&w) else { println!("   {w:08x} (no definition)"); continue };
            let sounds: Vec<String> = d.fire_sounds.iter().map(|s| match game.sounds.pcm(*s) {
                Some((rate, pcm)) => format!("{s:08x} {:.2}s", pcm.len() as f32 / rate as f32),
                None => format!("{s:08x} (missing)"),
            }).collect();
            print!("   {:18} type {} rate {:5.2}/s ammo {:3} speed {:5.0} range {:4.0} arch {:08x} sounds [{}]",
                   d.label, d.weapon_type, d.rate, d.ammo, d.bullet_speed, d.range, d.archetype, sounds.join(" "));
            match weapon::WeaponModel::load(game, d.archetype) {
                Ok(m) => {
                    let geo: Vec<String> = m.parts.iter().map(|p| format!("{}g/{}v{}", p.geosets.len(),
                        p.geosets.iter().map(|g| g.positions.len()).sum::<usize>(), if p.spin_axis.is_some() { " spin" } else { "" })).collect();
                    let mut hps: Vec<String> = m.hardpoints.keys().map(|k| format!("{k:08x}")).collect();
                    hps.sort();
                    println!(" parts [{}] hardpoints [{}] muzzle {}", geo.join(", "), hps.join(" "),
                             if m.hardpoints.contains_key(&d.muzzle) { "ok" } else { "MISSING" });
                }
                Err(e) => println!(" model error: {e}"),
            }
        }
    }
}
