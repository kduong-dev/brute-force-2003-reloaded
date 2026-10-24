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
//!   Left mouse  fire (hold)
//!   Q           switch weapon
//!   R           reload early (the demo's own: the game reloads only by itself, see play_ammo.rs)
//!   G           use the item in the item box (grenade: hold to charge, release to throw;
//!               Roller / Sentry: set down at the press)
//!   Tab         the game's B: tap for the next item; hold 0.25 s for the item list (the game
//!               pauses; W/D or the wheel up the next item, S/A or the wheel down the previous)
//!   B           the game's Black: the Medkit shortcut (play_itembox.rs)
//!   E (hold)    use (a gate's wall panel)
//!   M           next ground surface (footstep / landing sounds)
//!   Backspace   back to the map menu (in the same window)
//!   1-4         Brutus / Flint / Hawk / Tex
//!   V           the special skill on / off (the game's White button; play_skills.rs)
//!   I           in a mission, the objectives menu (the game's BACK; play_objectives.rs)
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
use bf_viewer::bf::character::{Character, Game, LIQUID_SHOT};
use bf_viewer::bf::locomotion::{self, Locomotion};
use bf_viewer::bf::weapon::{self, Hardpoint, WeaponDef, WeaponModel};
use bf_viewer::scene::{spawn_model, spawn_static, ModelAssets};
use bf_viewer::bf::level::Level;

/// The squad's four by squad slot (`Player::character`): the names their motion sets, their
/// types (common's objecttypes) and the name-keyed `Game::character_*` tables go by; then OTHER's.
const CHARACTERS: [&str; 5] = ["brutus", "flint", "hawk", "tex", ""];
/// How many of CHARACTERS are the squad: the slots control passes between.
const SQUAD: usize = 4;
/// The slot of a character of any other type (a test map NPC, #128), whose own data is its type's
/// (`Player::kind`: its model, HP, weapons, flesh, decals, and its damage, play_damage.rs). Its
/// name is empty: what's still looked up by slot name (chatter, skills, the scope and camera)
/// finds nothing there, so it says nothing and has no skill.
const OTHER: usize = 4;
/// The crossfade between two gait or action clips (s): the demo's (the game's MOTION
/// `transition`s aren't read here); a change between the carried and the ready set takes the
/// ready times instead (`ready_times`, #140).
const FADE: f32 = 0.2;
/// The ready times (in-time, out-time; s) of a character whose type has no motion set loaded
/// (every type in the data names one): the demo's raise before #140 (0.2 s, fitted to the takes'
/// input-to-log times, which hold the rig's delay and the log's lateness: todo/140 answer §2) and
/// the crossfade.
const READY_FALLBACK: (f32, f32) = (0.2, FADE);

/// `p`'s ready times (in-time, out-time; s), `l` its loaded character: its motion set's
/// `pc-transition-ready` while a human player controls it, else its `npc-transition-ready`
/// (FUN_0010fce0 for the raise: 4.0 + type+0x230 or +0x238; FUN_0011a440 by FUN_00112800 for the
/// ready pose's blend). The squad's: pc 0.1 / 0.2, npc 0.2 / 0.2 (Brutus 0.3 / 0.2); e40's
/// Shrikes pc 0.01 / 0.2, npc 0.4 / 0.3.
fn ready_times(p: &Player, l: &Loaded) -> (f32, f32) {
    if p.controlled { l.ready_pc } else { l.ready_npc }
}

/// Raise the gun before a first shot (FUN_00123ea0 with the ready timer out): FUN_00123f60 starts
/// the timer at 4 + the in-time (FUN_0010fce0) and the fire command (CT_FIRE_WEAPON, 0x1225d7)
/// fires nothing while it's over 4, so the first round waits the in-time. The game tests the
/// timer in the command (0x1225d7) before the item update counts it down (FUN_0011a440,
/// 0x11a636), where the demo's fire loop counts a cooldown down and then tests it in one game
/// frame: so both hands' cooldowns are held at the in-time and one game frame (shots::TICK), and
/// with 0.1 s the round comes on the 3rd or 4th 30 Hz frame after the press (as the frames' time
/// wobbles), 0.10-0.13 s, as the game's timer gives it (todo/140 answer §1.1). The ready timer
/// runs from here (`FireState::raise`), shot or not. BF_SHOT_LOG logs it.
fn raise(p: &mut Player, in_time: f32) {
    p.fire_state.raise(in_time);
    let wait = in_time + shots::TICK;
    p.cooldown = p.cooldown.max(wait);
    if let Some(s) = p.second.as_mut() {
        s.cooldown = s.cooldown.max(wait);
    }
    if std::env::var("BF_SHOT_LOG").is_ok() {
        println!("t={:.2} {}: raises the gun: the first round in {in_time} s ({}-transition-ready in-time)", p.sim_time, unit_name(p),
                 if p.controlled { "pc" } else { "npc" });
    }
}
/// Whether `p`'s motion can aim now (FUN_0011e000's "may aim", char+0x260 bit 3, from the motion's
/// aim flag, aim="false" in the motion set being its +0x250 bit 6, and an overlay's +0x3d bit 0):
/// not in the crouch's transitions (stand2crouch, crouch2stand; the crouch itself can), a dash, a
/// jump, a dodge, a dive, a slide or a grenade's throw (ms_tex.xml: 43 of its 83 motions can't),
/// nor a reload (its overlay's, inferred from #35 take29), a weapon switch (FUN_00121670) or dead
/// (FUN_00115d50). While it can't, the ready timer is held at 0 (`FireState::lower`; todo/126
/// answers.md Q7, Q11).
fn can_aim(p: &Player) -> bool {
    let dash = matches!((p.action, p.gait), (Action::None, Gait::Sprint));
    let moving = matches!(p.action, Action::Crouching | Action::Rising | Action::Dodge { .. } | Action::Dive
        | Action::JumpCrouch | Action::JumpLaunch | Action::JumpFall | Action::JumpLand);
    !(p.dead || dash || moving || p.slide_on || p.throwing.is_some() || p.reloading.is_some() || p.switching.is_some())
}
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
    /// the test tools' level switch (play_testworld.rs): between one map and the next, the next
    /// one loading on a thread (no menu)
    Switching,
    /// a mission's briefing movies with their subtitles (play_briefing.rs)
    Briefing,
}

/// The game mode being played: deathmatch is alone, without the squad's radar.
#[derive(Resource, Clone, Copy, Default)]
pub struct Mode {
    pub deathmatch: bool,
}

/// The game data for a map: the shared archives, the default level audio (surfaces and their
/// footstep / landing sounds come from the first mission; the multiplayer bank has everyone's
/// footsteps) and the squad's voice lines. The first mission's texture library isn't loaded
/// (`load_level_data`): the map's is, by `load_map` (#284).
pub fn load_game() -> Result<Game, String> {
    let mut game = Game::load(&data_dir())?;
    let level = std::env::var("BF_LEVEL").unwrap_or_else(|_| "e01".into());
    if let Err(e) = game.load_level_data(&data_dir(), &level) {
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

/// The library the deathmatch characters name (game-options-en.xml lines 64-67, `<character
/// h_f4db1dd4="dm" archetype-name="mp_tex">` ...): mp_common's textures-dm, what an arena's
/// player wears (`Game::load_match_textures`).
const DM_LIBRARY: &str = "dm";

/// Load a map (its level, streamed sounds and collision, made the current arena); "flat" or a
/// map that won't load: the flat test floor (None). The map's texture library becomes the one
/// after common's (`Game::load_level`): the squad wears its outfit (#284); on a multiplayer map
/// mp_common's library of the match comes after it and its skins win. The flat floor loads
/// none, so the search by archive name finds the squad's skins in e01's (the tan set).
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
            // a squad deathmatch map: the deathmatch squad's skill rates (play_skills.rs)
            if map.starts_with("sdm_") {
                if let Err(e) = game.load_squad_library(&data_dir(), skills::SDM_SQUAD) {
                    eprintln!("no squad deathmatch skills: {e}");
                }
            }
            // a multiplayer map: the match's library from mp_common after the map's, the squad
            // deathmatch squad's or the deathmatch characters' (its skins win: #284)
            if damage::multiplayer_map(game, map) {
                let library = if testworld::arena(game, map) { DM_LIBRARY } else { skills::SDM_SQUAD };
                if let Err(e) = game.load_match_textures(&data_dir(), library) {
                    eprintln!("no {library} textures from mp_common: {e}");
                }
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
            // test hook: BF_FIND_BODIES=1 lists the collision bodies of the placed objects and the
            // blockers (the terrain's in one line each): their size and shape, and how many of
            // their faces face into them (`Tri::inner`) (#285), then exits
            if std::env::var("BF_FIND_BODIES").is_ok() {
                for s in a.survey() {
                    let size = s.hi - s.lo;
                    let what = match (s.terrain, s.blocker, s.object) {
                        (true, _, _) => "terrain".to_string(),
                        (_, true, o) => format!("blocker{}", o.map_or(String::new(), |o| format!(" of object {o}"))),
                        (_, _, Some(o)) => format!("object {o} type h_{:08x} arch h_{:08x}", l.objects[o as usize].kind, l.objects[o as usize].archetype.unwrap_or(0)),
                        _ => "remains".to_string(),
                    };
                    let shape = if s.open_edges == 0 { "closed" } else { "open" };
                    let what = format!("{what}{}{}", if s.liquid { " (liquid)" } else { "" }, if s.door { " (door)" } else { "" });
                    println!("body {} {what}: {} triangles, {:.2} .. {:.2} ({:.1} x {:.1} x {:.1} m), {shape} ({} open edges, {} shared by 3+, {} wound against their neighbour), {} twinned, {} facing in, volume {:.2} m3{}",
                             s.body, s.triangles, s.lo, s.hi, size.x, size.y, size.z, s.open_edges, s.shared_edges, s.flipped_edges, s.twins, s.inner, s.volume,
                             if s.convex { ", convex" } else { "" });
                }
                std::process::exit(0);
            }
            let (n, terrain, slide, doors) = a.stats();
            println!("map {map}: collision {n} triangles ({terrain} terrain, {slide} slide-flagged, {doors} door leaves; {} blockers), {} start points",
                     l.blockers.len(), a.starts.len());
            // BF_SHOT_LOG: the world materials of its collision (a shot's hit sounds and debris,
            // `world_hit`)
            if std::env::var("BF_SHOT_LOG").is_ok() {
                let list: Vec<String> = a.material_counts().iter().map(|(id, t, o)| format!("{id}: {t} terrain + {o}")).collect();
                println!("map {map}: world materials (id: triangles) {}", list.join(", "));
            }
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
    // the difficulty every hit on a character is stepped by (play_damage.rs)
    let difficulty = damage::Difficulty::for_session(&game, &map);
    if std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("difficulty on {map}: {} (n {}): enemies take x{:.2}, the squad x{:.2}", difficulty.name, difficulty.n,
                 difficulty.step(testtools::ENEMY_TEAM), difficulty.step(0));
    }
    commands.insert_resource(difficulty);
    commands.insert_resource(CurrentMap(map));
    // a level whose squad its spawn-triggers spawn starts each member at its own (#102,
    // play_triggers.rs), led by BF_CHARACTER's if the level has them
    let spawns = level.as_ref().map(|l| triggers::squad_spawns(&game, l)).unwrap_or_default();
    let start = spawns.leader(std::env::var("BF_CHARACTER").ok().and_then(|s| s.parse().ok()).unwrap_or(3usize) % SQUAD);
    commands.insert_resource(player_at_start(start, &spawns));
    commands.insert_resource(squad_at_start(start, deathmatch, &spawns));
    commands.insert_resource(spawns);
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

/// The player at the map's start: its squad spawn-trigger's transform where the level has one
/// for it (`spawns`, #102: the feet on the ground under it, as the game's position setter puts
/// a copy, and its template's inventory), else the map's start point (BF_START=<n> picks
/// another); with the test hooks' starting camera and place.
fn player_at_start(start: usize, spawns: &triggers::SquadSpawns) -> Player {
    // test hooks: starting camera (yaw in radians, 0 = behind the character)
    let mut p = Player::new(start);
    let env = |k: &str| std::env::var(k).ok().and_then(|s| s.parse::<f32>().ok());
    if let Some(v) = env("BF_CAMERA_YAW") { p.cam_yaw = v; }
    if let Some(v) = env("BF_CAMERA_PITCH") { p.cam_pitch = v.clamp(-camera::PITCH_LIMIT, camera::PITCH_LIMIT); }
    // (the level's triggers know it by its spawn's copy name, and it carries its template's
    // weapons, wherever a test hook starts it)
    if let Some(s) = spawns.of(start) {
        s.dress(&mut p);
    }
    let spawn = spawns.of(start).filter(|_| test_goto().is_none());
    if let Some(s) = spawn {
        p.position = s.feet - Vec3::Y * GROUND;
        p.fall_from = s.feet.y;
        p.yaw = s.yaw;
        p.cam_yaw = s.yaw;
        // out of the world until its spawn-trigger spawns it (the tutorial's opening at 17.1 s;
        // not under BF_TEST_GOTO / BF_SCRIPT, whose runs skip the level's start scripts:
        // play_script.rs, #222)
        p.absent = s.trigger != 0 && script::spawns_wait();
    } else if let Some(a) = world::arena() {
        // on a map: at its first start point (BF_START=<n> picks another), facing its way
        let n = std::env::var("BF_START").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        if let Some(&(at, yaw)) = a.starts.get(n % a.starts.len().max(1)) {
            p.position = Vec3::new(at.x, at.y + 1.0 - GROUND, at.z);
            p.yaw = yaw;
            p.cam_yaw = yaw;
        }
    }
    if spawn.is_some() {
        p.prev_xz = Vec2::new(p.position.x, p.position.z);
        return p;
    }
    if let Some(g) = test_goto() {
        // a 5th value starts at that height (on the floor below it), e.g. on a roof; with no
        // floor below, on the steep face there if there's one (#264: it slides down it)
        let face = g.get(4).and_then(|&y| world::arena().filter(|a| a.floor_below(g[0], g[1], y).is_none())
            .and_then(|a| a.slope_below(g[0], g[1], y)));
        let ground = match g.get(4) {
            Some(&y) => world::arena().and_then(|a| a.floor_below(g[0], g[1], y)).map_or(y, |f| f.0),
            None => world::arena().and_then(|a| a.ground(g[0], g[1])).unwrap_or(GROUND),
        };
        p.position = Vec3::new(g[0], ground + 1.0 - GROUND, g[1]);
        p.yaw = (-(g[2] - g[0])).atan2(-(g[3] - g[1]));
        p.cam_yaw = p.yaw;
        if let Some((y, _)) = face {
            p.position.y = y - GROUND;
            p.prev_xz = Vec2::new(p.position.x, p.position.z);
            return p;
        }
    }
    p.position.y = floor_y(p.position.x, p.position.z, p.position.y + GROUND) - GROUND;
    p.prev_xz = Vec2::new(p.position.x, p.position.z);
    p
}

/// Test hook: BF_TEST_FALL=<m> starts the controlled character that many metres above the
/// floor it would stand on, falling from rest, so its landing takes a fall's damage from that
/// drop (FALL_HURT, FUN_0012b740; #157: a 37 m drop deals 164). The drop is measured from the
/// height it starts at, as the game's char+0x6e8 holds the height a fall began. Called once its
/// body is loaded (`spawn_player`: loading stands a character on its feet).
fn test_fall(p: &mut Player) {
    let Some(m) = std::env::var("BF_TEST_FALL").ok().and_then(|v| v.trim().parse::<f32>().ok()).filter(|&m| m > 0.0) else { return };
    p.action = Action::JumpFall;
    p.height = m;
    p.vy = 0.0;
    p.air_velocity = Vec3::ZERO;
    p.fall_from = p.position.y + GROUND + m;
    p.was_air = true;
}

/// The rest of the squad (none in deathmatch): on a level whose spawn-triggers spawn its squad
/// (`spawns`, #102), the members it has, each at its own spawn as `player_at_start` puts the
/// leader (the tutorial has Tex alone); otherwise all four, in formation behind the start
/// character.
fn squad_at_start(start: usize, deathmatch: bool, spawns: &triggers::SquadSpawns) -> Squad {
    if !spawns.0.is_empty() {
        let members = spawns.0.iter().filter(|s| s.slot != start).map(|s| {
            let mut m = Player::new(s.slot);
            m.position = s.feet - Vec3::Y * GROUND;
            m.fall_from = s.feet.y;
            m.prev_xz = Vec2::new(m.position.x, m.position.z);
            m.yaw = s.yaw;
            m.cam_yaw = s.yaw;
            m.surface = usize::MAX;
            s.dress(&mut m);
            // (out of the world until its spawn-trigger spawns it, as the leader: #222)
            m.absent = s.trigger != 0 && script::spawns_wait();
            m
        }).collect();
        return Squad(if deathmatch { vec![] } else { members });
    }
    // the rest of the squad, in formation behind the start character
    let origin = world::arena().and_then(|a| a.starts.get(std::env::var("BF_START").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0) % a.starts.len().max(1)).copied());
    squad_around(start, origin, deathmatch, spawns)
}

/// The squad's leader `start` standing at `at` (its feet) facing `yaw`, the camera behind:
/// where the map editor's "play from here" puts it (play_editor.rs, #191). Out of any wall
/// there, as `squad_around` puts the others.
fn player_at(start: usize, at: Vec3, yaw: f32) -> Player {
    let mut p = Player::new(start);
    let xz = world::arena().map_or(at.xz(), |a| a.push_out(at.xz(), BODY_RADIUS, at.y, at.y + BODY_HEIGHT, STEP_UP));
    let feet = floor_y(xz.x, xz.y, at.y + STEP_UP);
    p.position = Vec3::new(xz.x, feet - GROUND, xz.y);
    p.prev_xz = xz;
    p.fall_from = feet;
    p.yaw = yaw;
    p.cam_yaw = yaw;
    // (the follow camera starts on them, cut there: `camera::Rig`'s first frame)
    p
}

/// The squad but `start`, in formation round `origin` (a place and the way it faces; None: the
/// flat floor's origin, facing -z), each on the floor and out of the walls there (none in
/// deathmatch): the map's start (`squad_at_start`), or where the map editor's "play from here"
/// puts the leader (play_editor.rs). On a level whose spawn-triggers spawn its squad (`spawns`)
/// only the members it has come, keeping their spawns' names.
fn squad_around(start: usize, origin: Option<(Vec3, f32)>, deathmatch: bool, spawns: &triggers::SquadSpawns) -> Squad {
    let roster: Vec<usize> = if spawns.0.is_empty() { (0..SQUAD).collect() } else { spawns.0.iter().map(|s| s.slot).collect() };
    let members = roster.into_iter().filter(|&c| c != start).enumerate().map(|(slot, c)| {
        let mut m = Player::new(c);
        if let Some(s) = spawns.of(c) {
            s.dress(&mut m);
        }
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
/// collision. A screenshot on its way (BF_CAPTURE) is left to finish, with its observer (the
/// save to disk): the renderer marks it done a frame later, and panicked when it was gone, and
/// without the observer the last frame before the test tools' level switch wasn't saved (#116).
/// The screenshot and its observer go by themselves once it's saved.
#[allow(clippy::type_complexity)]
fn end_play(mut commands: Commands, before: Option<Res<Before>>,
            all: Query<(Entity, Option<&ChildOf>), (Without<Window>, Without<bevy::render::view::screenshot::Screenshot>,
                                                    Without<bevy::ecs::observer::Observer>)>,
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
    // (BF_MOVIES plays a briefing after the boot, whatever other hooks are set)
    let menu_first = !test_map && (menu::wanted() || briefing::requested().is_some());
    // (--test with BF_MAP: the test tools on that map, play_testworld.rs)
    let map = std::env::var("BF_MAP").unwrap_or_else(|_| if test_map { "flat".into() } else { "sdm_e34".into() });
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
        let start = std::env::var("BF_CHARACTER").ok().and_then(|s| s.parse().ok()).unwrap_or(3usize) % SQUAD;
        // BF_DUMP_TYPES=1  the character types of the loaded archives (character_type.rs:
        // `character_type_report`), their placements and the squad's check, and exit; `all`:
        // every archive of data/ loaded first (the whole cast: 128 types, 542 placements);
        // BF_DUMP_TYPES_FAST=1 leaves the models unloaded
        if let Ok(which) = std::env::var("BF_DUMP_TYPES") {
            if which == "all" {
                dump_all_archives(&mut game);
            }
            for line in game.character_type_report(std::env::var("BF_DUMP_TYPES_FAST").is_err(), &CHARACTERS[..SQUAD]) {
                println!("{line}");
            }
            return;
        }
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
                            println!("{name:7} {n:24} clip {c:3} {:.2}s bones {:3} root {:.2} speed {:?} events [{}]", a.duration, a.targets.len(), root, a.move_speed, ev.join(" "));
                        }
                        None => println!("{name:7} {n:24} -"),
                    }
                }
            }
            return;
        }
        // BF_DUMP_HITPOINTS=1: each character's hitpoints and damage-type factors, and exit
        if std::env::var("BF_DUMP_HITPOINTS").is_ok() {
            let mut hp: Vec<_> = game.character_hitpoints.iter().collect();
            hp.sort_by(|a, b| a.0.cmp(b.0));
            for (name, hp) in hp {
                println!("{name:12} {hp}  damage-type factors {:?}", game.character_damage_factors.get(name).cloned().unwrap_or_default());
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
        // (a test session plays a deathmatch arena alone, as its level switch does: by the
        // game's level list, play_testworld.rs `arena`)
        let deathmatch = std::env::var("BF_NO_SQUAD").is_ok() || std::env::var("BF_DEATHMATCH").is_ok()
            || (test_map && testworld::arena(&game, &map));
        loaded = Some(LoadedMap { game, level, deathmatch, map: map.clone() });
    }
    let mut app = App::new();
    // (BF_BACKGROUND / BF_WINDOW_MONITOR shape the window, and the first leaves the gamepads
    // out: play_background.rs)
    let mut plugins = DefaultPlugins.set(WindowPlugin { primary_window: Some(background::window()), ..default() });
    if background::on() {
        plugins = plugins.disable::<bevy::gilrs::GilrsPlugin>();
    }
    app.add_plugins(plugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight { brightness: 220.0, ..default() })
        .init_state::<AppState>();
    background::plugin(&mut app);
    bf_viewer::level_scene::plugin(&mut app);
    // the front end; its pictures and sounds come from the game data
    menu::plugin(&mut app);
    briefing::plugin(&mut app);
    // the font records and the mission flow (#226: the status line shows in every state)
    fonts::plugin(&mut app);
    mission::plugin(&mut app);
    if let Some(mut l) = loaded {
        let data = menu::gather(&mut l.game, &data_dir());
        menu::install(app.world_mut(), data);
        let movies = briefing::gather(&mut l.game);
        briefing::install(app.world_mut(), movies);
        let records = fonts::gather(&mut l.game);
        fonts::install(app.world_mut(), records);
        // straight into the map (BF_MAP or a test hook)
        app.insert_state(AppState::Playing);
        if test_map {
            // (test hook: BF_TEST_INSTANT_KILL=0 starts with it off, as K turns it)
            app.insert_resource(testmap::TestMap { instant_kill: std::env::var("BF_TEST_INSTANT_KILL").map_or(true, |v| v != "0") });
        }
        let mut commands = app.world_mut().commands();
        begin_play(&mut commands, l);
        app.world_mut().flush();
    }
    let playing = in_state(AppState::Playing);
    // (the map editor's edit mode has no character to drive or follow: play_editor.rs)
    let editing = || in_state(editor::EditorMode::Edit);
    app.add_plugins((hud::plugin, grenade::plugin, gas::plugin, energy::plugin, sonic::plugin, sentry::plugin, fx::plugin, dna::plugin, pickups::plugin, text::plugin, deathcam::plugin, testmap::plugin, scenery::plugin, ammo::plugin, bf_viewer::ale_fx::plugin))
        .add_plugins(shots::plugin)
        .add_plugins(missile::plugin)
        .add_plugins(bolts::plugin)
        .add_plugins(rays::plugin)
        .add_plugins(minigun::plugin)
        .add_plugins(shake::plugin)
        .add_plugins(skills::plugin)
        .add_plugins(damage::plugin)
        .add_plugins(liquids::plugin)
        .add_plugins(reticle::plugin)
        .add_plugins(targeting::plugin)
        .add_plugins(enemies::plugin)
        .add_plugins(vengar::plugin)
        // (after the Vengar view's: its pass follows that one's in the render graph)
        .add_plugins(scope_view::plugin)
        .add_plugins(stealth::plugin)
        .add_plugins(flags::plugin)
        .add_plugins(objectives::plugin)
        .add_plugins(triggers::plugin)
        .add_plugins(triggers::sounds::plugin)
        .add_plugins(script::plugin)
        .add_plugins(brain::plugin)
        .add_plugins(placed_fx::plugin)
        .add_plugins(camera::plugin)
        .add_plugins(inventory::plugin)
        .add_plugins(itembox::plugin)
        .init_resource::<UsePanel>()
        .init_resource::<AiHits>()
        .init_resource::<GameRandom>()
        .add_systems(OnEnter(AppState::Playing), (snapshot_entities, setup).chain())
        .add_systems(OnExit(AppState::Playing), end_play)
        .add_systems(Update, (spawn_player, read_input.run_if(not(editing())), squad_control, doors, update_player, play_sounds,
                               follow_camera.run_if(not(editing())), update_weapons, move_tracers, projectiles, update_hud, slide_dust, capture)
            .chain().run_if(playing.clone()))
        .add_systems(Update, lamp_characters.after(update_player).run_if(playing.clone()))
        .add_systems(Update, spawn_splashes.after(update_player).run_if(playing.clone()))
        .add_systems(Update, turn_sky.run_if(playing.clone()))
        .add_systems(Update, run_liquid_clock.run_if(playing.clone()))
        .add_systems(Update, (back_to_menu, audio_probe).run_if(playing))
        .add_systems(PostUpdate, bf_viewer::level_scene::follow_sky.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}

#[path = "play_hud.rs"]
mod hud;
#[path = "play_grenade.rs"]
mod grenade;
#[path = "play_gas.rs"]
mod gas;
#[path = "play_energy.rs"]
mod energy;
#[path = "play_sonic.rs"]
mod sonic;
#[path = "play_sentry.rs"]
mod sentry;
#[path = "play_fx.rs"]
mod fx;
#[path = "play_dna.rs"]
mod dna;
#[path = "play_pickups.rs"]
mod pickups;
#[path = "play_inventory.rs"]
mod inventory;
#[path = "play_itembox.rs"]
mod itembox;
#[path = "play_text.rs"]
mod text;
#[path = "play_deathcam.rs"]
mod deathcam;
#[path = "play_testmap.rs"]
mod testmap;
#[path = "play_testtools.rs"]
mod testtools;
#[path = "play_testworld.rs"]
mod testworld;
#[path = "play_testscenery.rs"]
mod testscenery;
#[path = "play_editor.rs"]
mod editor;
#[path = "play_menu.rs"]
mod menu;
#[path = "play_briefing.rs"]
mod briefing;
#[path = "play_fonts.rs"]
mod fonts;
#[path = "play_mission.rs"]
mod mission;
#[path = "play_scenery.rs"]
mod scenery;
#[path = "play_ammo.rs"]
mod ammo;
#[path = "play_shots.rs"]
mod shots;
#[path = "play_missile.rs"]
mod missile;
#[path = "play_bolts.rs"]
mod bolts;
#[path = "play_rays.rs"]
mod rays;
#[path = "play_minigun.rs"]
mod minigun;
#[path = "play_shake.rs"]
mod shake;
#[path = "play_skills.rs"]
mod skills;
#[path = "play_background.rs"]
mod background;
#[path = "play_damage.rs"]
mod damage;
#[path = "play_reticle.rs"]
mod reticle;
#[path = "play_targeting.rs"]
mod targeting;
#[path = "play_enemies.rs"]
mod enemies;
#[path = "play_vengar.rs"]
mod vengar;
#[path = "play_scope_view.rs"]
mod scope_view;
#[path = "play_stealth.rs"]
mod stealth;
#[path = "play_triggers.rs"]
mod triggers;
#[path = "play_flags.rs"]
mod flags;
#[path = "play_objectives.rs"]
mod objectives;
#[path = "play_script.rs"]
mod script;
#[path = "play_brain.rs"]
mod brain;
#[path = "play_placed_fx.rs"]
mod placed_fx;
#[path = "play_camera.rs"]
mod camera;
#[path = "play_liquids.rs"]
mod liquids;
use bf_viewer::arena as world;

/// The map's level (drawn at startup), if playing on one.
#[derive(Resource)]
struct MapLevel(Option<Level>);

/// The map's doors and its wall panels.
#[derive(Resource)]
struct Doors {
    /// in level object order (`Arena::set_door_open`'s)
    list: Vec<DoorState>,
    panels: Vec<Panel>,
}

/// A door or gate as its anim-triggers play it (play_triggers.rs, #220): how far into its
/// clip it is (s of the clip at its own speed), whether it's playing forward (opening) or
/// backward (closing), and how fast (the playing anim entry's time-scale: the tutorial gate's
/// 2.5 plays its 4.96 s clip in 2.0 s).
struct DoorState {
    door: bf_viewer::level_scene::Door,
    t: f32,
    opening: bool,
    rate: f32,
}

/// A wall panel (world-button-object, reticule-action 1): its name, the event its use sends
/// (its signal h_eaf8a35b and data h_f1660a0d, to its listeners: 0x163060), its button (the
/// archetype's PANEL_BUTTON hardpoint, world) and the way that faces, the doors its signal
/// reaches unchanged (`Level::button_opens`), and whether it's been used.
struct Panel {
    name: u32,
    signal: i64,
    data: i64,
    at: Vec3,
    facing: Vec3,
    opens: Vec<usize>,
    used: bool,
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

/// Doors and gates move only when the level's anim-triggers play their clips (play_triggers.rs,
/// #220: on their detectors' signals, a panel's, a timer's ... or at the level's start): their
/// leaves move by their clips forward (opening) or back (closing), along a sliding joint's axis
/// or as a transform or hinge joint's channel sets them (#261, level_scene.rs `Leaf::pose`), and
/// an open door stops blocking. Their sounds are the sound-triggers the anim-triggers send ANIM_START /
/// ANIM_BACK_START to (play_sound_triggers.rs, #223). A wall panel is used with the player's feet
/// within USE_REACH m of its button, looking within USE_ANGLE of it (the panels'
/// reticule-action 1: the reticle on them), holding use for HOLD_TIME s (the game's prompt says
/// "Hold"): its signal goes to its listeners. The capture shows no progress bar and no gate
/// message. Reach, angle and hold time are guesses.
const USE_REACH: f32 = 2.5;
const USE_ANGLE: f32 = 0.8;
const HOLD_TIME: f32 = 0.5;
/// The gate panels' button: hardpoint h_10f81d34 of their archetype (the green glow quad).
const PANEL_BUTTON: u32 = 0x10F8_1D34;
/// Music volume (linear) under the game's sounds.
const MUSIC_VOLUME: f32 = 0.5;
/// A level's ambience bed, under its music (of the music's volume).
const AMBIENCE_VOLUME: f32 = 0.8;

/// Each frame: the panel prompt and a panel's use (see USE_REACH's notes), whose event goes to
/// the level's triggers (`triggers::TriggerEvents`); then each door's leaves where its
/// anim-triggers send them, its collision open past 0.6 of its clip.
fn doors(time: Res<Time>, player: Res<Player>, doors: Option<ResMut<Doors>>, mut use_panel: ResMut<UsePanel>,
         mut events: ResMut<triggers::TriggerEvents>, mut transforms: Query<&mut Transform>) {
    let dt = frame_dt(&time);
    // (the status message runs out on any map: the flat test floor has no doors)
    if let Some((_, left)) = &mut use_panel.message {
        *left -= dt;
        if *left <= 0.0 {
            use_panel.message = None;
        }
    }
    let Some(mut doors) = doors else { return };
    let doors = &mut *doors;
    // a panel's button in reach and in view: one not used yet, whose doors (those its signal
    // reaches unchanged) aren't all open
    let look = Vec2::new(-player.cam_yaw.sin(), -player.cam_yaw.cos());
    let feet = player.position + Vec3::Y * GROUND;
    // (from in front of it: the button faces out of the wall)
    let usable = |p: &Panel| {
        let to = Vec2::new(p.at.x - feet.x, p.at.z - feet.z);
        !player.dead && to.length() < USE_REACH && (-0.5..2.5).contains(&(p.at.y - feet.y)) && to.normalize_or_zero().dot(look) > USE_ANGLE.cos()
            && (feet - p.at).dot(p.facing) > 0.0
    };
    let list = &doors.list;
    let target = doors.panels.iter().position(|p| !p.used && !(!p.opens.is_empty() && p.opens.iter().all(|&d| list[d].opening)) && usable(p));
    use_panel.prompt = target.map(|_| "panel");
    use_panel.target = target.map(|i| doors.panels[i].at);
    use_panel.held = if target.is_some() && player.use_held { use_panel.held + dt } else { 0.0 };
    if let (Some(i), true) = (target, use_panel.held >= HOLD_TIME) {
        // its event to its listeners (0x163060: {its signal, data}, the user as the subject)
        let p = &mut doors.panels[i];
        p.used = true;
        events.0.push(triggers::ObjectEvent { from: p.name, signal: p.signal, subject: player.trigger_name, data: p.data });
        use_panel.held = 0.0;
    }
    // the leaves, where their anim-triggers send them (play_triggers.rs)
    for (i, d) in doors.list.iter_mut().enumerate() {
        let DoorState { door, t, opening, rate } = d;
        let was = *t;
        *t = if *opening { (*t + dt * *rate).min(door.duration) } else { (*t - dt * *rate).max(0.0) };
        if *t != was {
            for leaf in &door.leaves {
                if let Ok(mut tr) = transforms.get_mut(leaf.entity) {
                    *tr = leaf.pose(*t, door.duration);
                }
            }
            // BF_DOOR_LOG: a door's leaves, each second of its clip (how far each sliding one has
            // slid; how far each other one has moved and turned)
            if (was == 0.0 || *t == door.duration || *t == 0.0 || t.floor() != was.floor()) && std::env::var("BF_DOOR_LOG").is_ok() {
                let round = |x: f32| (x * 100.0).round() / 100.0;
                let animated = door.leaves.iter().any(|l| matches!(l.motion, bf_viewer::level_scene::LeafMotion::Animated(_)));
                if animated {
                    let moved: Vec<(f32, f32)> = door.leaves.iter().map(|l| l.moved(*t, door.duration)).collect();
                    println!("t {:.2}: door h_{:08x} t {:.2}/{:.2} (x{rate}) leaves moved {:?} m, turned {:?} deg", player.sim_time, door.name, *t, door.duration,
                             moved.iter().map(|m| round(m.0)).collect::<Vec<_>>(), moved.iter().map(|m| (m.1 * 10.0).round() / 10.0).collect::<Vec<_>>());
                } else {
                    println!("t {:.2}: door h_{:08x} t {:.2}/{:.2} (x{rate}) leaves slid {:?} m", player.sim_time, door.name, *t, door.duration,
                             door.leaves.iter().map(|l| round(l.slide(*t, door.duration))).collect::<Vec<_>>());
                }
            }
        }
        if let Some(a) = world::arena() {
            a.set_door_open(i, *t > 0.6 * door.duration);
        }
    }
}

/// The level's point lights on the characters. The scenery takes them per surface (its
/// `LevelMaterial`); a character's materials are bevy's own, so each takes the light at its chest
/// from every direction, as its texture added to the picture (emissive), at CHARACTER_LAMP of it
/// (about the average N.L over a body). A body's stealth copies (play_stealth.rs `BodyCopy`)
/// have materials of their own and are passed over.
#[allow(clippy::too_many_arguments)]
fn lamp_characters(map: Res<MapLevel>, player: Res<Player>, squad: Res<Squad>, children: Query<&Children>,
                   parts: Query<&MeshMaterial3d<StandardMaterial>>, drawn: Query<(), (With<Mesh3d>, Without<stealth::BodyCopy>)>,
                   mut materials: ResMut<Assets<StandardMaterial>>, mut last: Local<HashMap<Entity, Vec3>>) {
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
        // (a part drawn with another material meanwhile, the Vengar view's heat figure, keeps the
        // light unset: it's set again once the part has its own back)
        let mut all = true;
        for e in children.iter_descendants(l.root) {
            let Ok(m) = parts.get(e) else {
                all &= !drawn.contains(e);
                continue;
            };
            let Some(mat) = materials.get_mut(&m.0) else { continue };
            if mat.unlit {
                continue;
            }
            mat.emissive = LinearRgba::rgb(light.x, light.y, light.z);
            if mat.emissive_texture.is_none() {
                mat.emissive_texture = mat.base_color_texture.clone();
            }
        }
        if all {
            last.insert(l.root, light);
        }
    }
}

/// The sky's clock on the game's (`frame_dt`: stopped under a menu, fixed steps while
/// capturing), which turns it (level_scene.rs `follow_sky`; FUN_00177ba0 from the world step).
fn turn_sky(time: Res<Time>, mut sky: Query<&mut bf_viewer::level_scene::SkyLayer>) {
    let dt = frame_dt(&time);
    if dt > 0.0 {
        for mut s in &mut sky {
            s.run(dt);
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

/// How long (s, real time) a first Backspace in a test session waits for the second that
/// leaves (the demo's choice): leaving drops the map and what was built on it (#191).
const LEAVE_CONFIRM: f32 = 2.0;

/// Backspace: back to the map menu (bf_play starts again there). In a test session (the map
/// editor's, play_editor.rs) only a second press within LEAVE_CONFIRM s leaves, so a stray key
/// doesn't throw away what was built.
#[allow(clippy::too_many_arguments)]
fn back_to_menu(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>, mode: Res<Mode>, map: Res<CurrentMap>,
                mut commands: Commands, mut frames: Local<u32>, test_map: Option<Res<testmap::TestMap>>, time: Res<Time<Real>>,
                mut armed: Local<Option<f32>>, mut status: ResMut<UsePanel>) {
    // test hook: BF_TEST_BACK=<n> goes back after n frames of the map
    *frames += 1;
    let test = std::env::var("BF_TEST_BACK").ok().and_then(|v| v.parse::<u32>().ok()) == Some(*frames);
    let now = time.elapsed_secs();
    let mut leave = keys.just_pressed(KeyCode::Backspace);
    if leave && test_map.is_some() && !armed.is_some_and(|at| now - at < LEAVE_CONFIRM) {
        leave = false;
        *armed = Some(now);
        status.message = Some(("Backspace again to leave for the menu: the map isn't kept".into(), LEAVE_CONFIRM));
    }
    if leave || test {
        *armed = None;
        *frames = 0;
        commands.insert_resource(menu::Return { map: Some(map.0.clone()), deathmatch: mode.deathmatch, main: false });
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
    /// how fast it plays (1 but for an enemy's walking clip driven at its type's walk speed,
    /// play_enemies.rs)
    rate: f32,
    /// how long (s) it takes to fade in over the others: FADE, or a ready time when it's the gun
    /// going up or down (#140, `ready_times`)
    fade: f32,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
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

/// BF_MOVE_LOG=1: the controlled character's position, facing, gait and clip every half second,
/// with the clip whose root motion moves it when that's a stand-in (`root_stand_ins`).
fn move_log(p: &Player, l: &Loaded, dt: f32) {
    if (p.sim_time * 2.0).floor() == ((p.sim_time - dt) * 2.0).floor() || std::env::var("BF_MOVE_LOG").is_err() {
        return;
    }
    let top = p.layers.last().map(|x| x.clip);
    println!("t={:.2} pos ({:.2}, {:.2}) yaw {:.0} gait {:?} {:?} clip {:?} root from {:?}", p.sim_time, p.position.x, p.position.z,
             p.yaw.to_degrees(), p.gait, p.action, top, top.and_then(|t| l.clips.root_stand_in.get(&t)));
}

/// A looping clip's root motion from `t` over `dt`; a side step without root motion of its own
/// takes its stand-in walk's (`root_stand_ins`), at the same point of the cycle, one walk cycle
/// per cycle of the side step.
fn clip_root_delta(model: &Character, game: &Game, clips: &Clips, clip: usize, t: f32, dt: f32) -> Vec3 {
    match clips.root_stand_in.get(&clip) {
        Some(&walk) => {
            let k = model.anims[walk].duration / model.anims[clip].duration.max(1e-3);
            locomotion::root_delta(model, game, walk, t * k, dt * k)
        }
        None => locomotion::root_delta(model, game, clip, t, dt),
    }
}

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
    /// the motion set's special motions, where it has them (Tex's berserk set, Flint's sniper set;
    /// see `SpecialClips`)
    special: Option<SpecialClips>,
    /// side steps whose own root doesn't move, and the walk whose root motion moves them instead
    /// (`root_stand_ins`)
    root_stand_in: HashMap<usize, usize>,
}

/// The motions a motion set's MT_TOGGLE_SPECIAL (45) transitions lead to, which a skill's toggle
/// asks for (`Skill::special_motions`):
///  - Tex's berserk set (common/characters/ms_tex.xml). His toggle asks for motion type 45 at an
///    accepted press, on and off (FUN_0012eab0(type, 45, 1) at 0x13d6ca), and each of his stand,
///    walk, run, side step, back step and dash motions has a transition of that type to its
///    berserk motion, which has one back. The berserk motions are `Sc_w1_bz_*`, prefix-type 0: the
///    same for both stances, carried and aimed alike (script = ready-script). They have no
///    transition to a crouch, a dash, a dodge, a dive or a throw: in berserk he can't do those
///    (the dash stays the run). Their clips (BF_ANIM_PROBE): idle 9.97 s, walk 1.29 m/s, run 4.69
///    m/s.
///  - Flint's sniper set (ms_flint.xml), which her targeting's toggle asks for with a rifle in
///    hand (FUN_0013cad0 returns 2): her idles and crouch idle lead to `Sc_w1_sniper_idle`, the
///    walk, run, crouch walk and dash to `Sc_w1_sn_walk`, the side steps and runs to
///    `Sc_w1_sn_{f,b}_side_walk_{l,r}`, the back walk and run to `Sc_w1_sn_back_walk`, all
///    prefix-type 0, and each has a type 45 transition back. They lead on to nothing but each
///    other, the slide and the fall (types 35 and 21: the normal ones): no run, crouch, dash,
///    dodge, dive, throw or jump.
#[derive(Default, Debug)]
struct SpecialClips {
    /// idle, walk, run, back walk, back run (no sprint)
    loco: Locomotion,
    /// side steps: forward left, forward right, back left, back right (`side_index`)
    side: [Option<usize>; 4],
    jump_crouch: Option<usize>,
    jump_launch: Option<usize>,
    jump_fall: Option<usize>,
    jump_land: Option<usize>,
    slide: Option<usize>,
    /// a jump (motion types 20 / 24) can start from them: Tex's set has its own, Flint's sniper
    /// motions no transition to one
    jumps: bool,
}

/// Below this a clip's own speed (`AnimDef::move_speed`, m/s) counts as none: FUN_0012c6c0's
/// test before it divides by the speed (0x3a4fe4 = 0.001).
const NO_CLIP_SPEED: f32 = 0.001;

/// The game doesn't move a walking character by its clip: FUN_0012c6c0 sets the ground speed
/// from the type's `speeds` along the stick (the clip's root term is scaled by 0x4699e4, which
/// nothing writes: zero), and plays a clip without a speed of its own at rate 1. The demo moves by
/// root motion, so a side step whose root stays put (Tex's Sc_w1_rp_f_side_walk_l, speed 5.9e-05
/// and no root translation: #170) borrows the root motion of the walk in its direction, forward
/// (rp_walk, cr_walk) or back (rp_back_walk, cr_back_walk). Every other side step in the squad's
/// sets carries its walk's speed and length, or within 6% (Flint's and Hawk's crouched ones);
/// Tex's other three Sc_w1 side steps carry rp_walk's and rp_back_walk's exactly. His berserk
/// set's forward left side step (Sc_w1_bz_f_side_walk_l) has no root motion either: it borrows
/// Sc_w1_bz_walk's.
fn root_stand_ins(model: &Character, slots: &[SlotClips; 2], special: Option<&SpecialClips>) -> HashMap<usize, usize> {
    let speed = |c: usize| model.anims[c].move_speed.unwrap_or(f32::MAX);
    let mut out = HashMap::new();
    let special = special.map(|s| (&s.side, s.loco.walk, s.loco.walk_back));
    for (side, fwd, back) in slots.iter().flat_map(|s| [(&s.rp_side, s.rp.walk, s.rp.walk_back), (&s.cr_side, s.cr_walk, s.cr_back_walk)]).chain(special) {
        for (k, clip) in side.iter().enumerate() {
            let walk = if k < 2 { fwd } else { back };
            if let (Some(c), Some(w)) = (*clip, walk) {
                if speed(c) < NO_CLIP_SPEED && speed(w) >= NO_CLIP_SPEED {
                    out.insert(c, w);
                }
            }
        }
    }
    out
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
    /// its muzzle effect type has a light effect (light_muz_minigun, light_muz_laser and the
    /// like: each kind of gun names its own), which play_shots.rs plays with the effect out of
    /// the holder's scope and which stands in for `light`, the demo's own
    data_light: bool,
    flash: Entity,
    muzzle: Hardpoint,
    /// the shell-eject hardpoint (ShotData::shell_hardpoint), if the model has it
    shell: Option<Hardpoint>,
    /// fire direction in the weapon's frame
    fire_dir: Vec3,
    /// placement under a joint (bone, rotation, translation): in hand / stowed for its slot
    in_hand: Option<(usize, Quat, Vec3)>,
    stowed: Option<(usize, Quat, Vec3)>,
    /// and in the second hand (Tex's berserk): its trigger grip at the character's SECOND_HAND
    /// hardpoint, as the gun in hand's at TRIGGER_HAND (weapon.rs); None for a model without it
    second_hand: Option<(usize, Quat, Vec3)>,
}

/// A gun drawn into the second hand (Tex's berserk, play_skills.rs): the game's inventory
/// second-hand holder (+0x178), its gun at +0x198 (FUN_0014d310). It fires with the gun in hand
/// (play_shots.rs), as a weapon object of its own.
#[derive(Clone, Copy, Debug)]
pub struct SecondGun {
    /// its index in the character's weapons (the other weapon slot's gun, FUN_0014d090)
    pub weapon: usize,
    /// its fire cooldown (its weapon+0x218; the gun in hand's is `Player::cooldown`)
    pub cooldown: f32,
    /// its kick back along the barrel and its muzzle flash's time left, as the gun in hand's
    /// `Player::recoil` / `flash`
    pub recoil: f32,
    pub flash: f32,
}

impl SecondGun {
    /// The gun of weapon `weapon`, just drawn.
    pub fn new(weapon: usize) -> Self {
        SecondGun { weapon, cooldown: 0.0, recoil: 0.0, flash: 0.0 }
    }
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
/// reaches it (EV_REACH, 0.23-0.47 s over the squad's clips: the recordings' "in hand
/// 0.38-0.47 s") and lets it go (19f8311b, the use_item clip's "used" event, 0.50-0.60 s); it drops from the hand to the
/// ground (the recordings: on the ground ~0.8 s (Sentry) / ~1.05 s (Roller) after the press)
const EV_PLACED: u32 = EV_ITEM_USED;
/// An inventory item, one stack of the squad's store: a grenade type (an index into
/// `grenade::GrenadeKits`, the same as into `Player::grenades`), the Medkit (the 80-point (5, 0)
/// stack), or another item of the store by its item type (`Player::others`: the OrgSen, the Hat,
/// the 60-point Medkit (5, 2); play_inventory.rs). The item box's selection is one
/// (`Player::item`, play_itembox.rs).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Grenade(usize),
    Medkit,
    Other(u32),
}

/// The inventory items, in the store's (category, index) order, the item box's (FUN_0014c230's
/// walk): every grenade type (the kits are in that order, play_grenade.rs `load_kits`: categories
/// 2 and 3), then the Medkit (5, 0), then the other items the item box lists (their type's
/// h_1852ab6c: the OrgSen and the Hat, not "Alien Technology Acquired!"), kept in that order too
/// (play_inventory.rs: categories 5 and up). Each whether carried or not: the listable ones are
/// those with a count (play_itembox.rs `listable`).
fn items(p: &Player) -> Vec<Item> {
    (0..p.grenades.len()).map(Item::Grenade).chain([Item::Medkit])
        .chain(p.others.iter().filter(|c| c.listed).map(|c| Item::Other(c.kind))).collect()
}

/// How many of an item the squad carries.
fn item_count(p: &Player, item: Item) -> i64 {
    match item {
        Item::Grenade(k) => p.grenades.get(k).copied().unwrap_or(0),
        Item::Medkit => p.medkits,
        Item::Other(kind) => p.others.iter().find(|c| c.kind == kind).map_or(0, |c| c.count),
    }
}

/// Whether the controlled character can use the item now: the item box's colour (its name and
/// count blue, else red with the icon grey; 0xac980 / FUN_000ace10 ask the item function's "can
/// use", +0x3c). A Medkit (either) only while hurt (the Medkit's own +0x3c 0x147070, "No need to heal":
/// #196 take01 6.95 and 8.45 s, take10 2.55 s); an item whose use type the character's skill
/// refuses (the skill's +0x1c, which the base set's "can use" 0x146c70 asks: Brutus and the OrgSen
/// or the Hat, red in take26; play_skills.rs `uses_item`). A dead member's box isn't red (take18:
/// "Energy 5" fades out blue).
fn item_usable(p: &Player, item: Item) -> bool {
    item_count(p, item) > 0 && match item {
        Item::Grenade(_) => true,
        Item::Medkit => p.health < p.max_health,
        Item::Other(kind) => p.others.iter().find(|c| c.kind == kind)
            .is_some_and(|c| skills::uses_item(p, c.use_type) && (!c.heals || p.health < p.max_health)),
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
/// Falls (FUN_0012b740): past 5 m a landing hurts, 100 x ((drop - 5) / 25)^2 with no upper
/// clamp (5.0 at 0x3c0590, 30.0 at 0x3c0594, 100.0 at 0x3c0598): 100 at 30 m, 144 at 35 m, 164
/// at 37 m (m09_a's waste pit), so a long fall kills Brutus (105) and Tex (115) too (#157).
const FALL_HURT: (f32, f32, f32) = (5.0, 30.0, 100.0);
/// A fall's damage type: FUN_0012b740 puts 0 (DTYPE_NORMAL) in its record's +4 and sends it to
/// the character's own handler (char+0x48's vtable slot 0: 0x125600, the difficulty's step).
const FALL_DAMAGE_TYPE: i64 = 0;
/// Effect type of the dust kicked up sliding (objecttypes <effect>: slide_puff).
const SLIDE_EFFECT: u32 = 0x0151_3B0F;
/// seconds to charge a throw fully (the recordings: a linear fill, full in 0.60 s, then held)
const CHARGE_TIME: f32 = 0.6;
/// The throw meter's timer (`Player::meter_after`'s second value): the player controller sets
/// it to 1.0 with the level every frame the charge is above 0 (0x134823-0x13484e) and nothing
/// after, so the meter's update (0xad950) counts it down from the last charging frame and draws
/// the meter at clamp(2 x timer, 0, 1)^3 (play_hud.rs): full 0.5 s, gone at 1.0 s (#162).
pub const METER_TIME: f32 = 1.0;
/// Seconds from a member's selection to control passing to them: the squad select widget's
/// countdown, +0x78 set to its +0x7c, 0.5 (its constructor FUN_000c8760), as a selection is made
/// (FUN_000c8cc0) and counted down by FUN_000c90c0 to the hand-over FUN_000c8fa0 (#88). The
/// recordings agree: "BRUTUS" shows at 22.867 s and the cut is at 23.367 s in `Friendly Fire 2
/// + Death Cam.mp4`, "HAWK" at 31.847 s and 32.333 s. (It was 0.6, unmeasured.)
const SELECT_TIME: f32 = 0.5;
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
/// (about 120 degrees), elbows a little more (the squad's; an enemy's are the game's limits,
/// RAGDOLL_KNEE_LIMIT and RAGDOLL_ELBOW_LIMIT).
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
/// The game's ragdoll joint limits (FUN_00113120, by the child bone's name hash), used for
/// #129's enemies (`Ragdoll::new`'s `parts`; the squad keeps the demo's above): a knee (Calf)
/// bends one way by at most RAGDOLL_KNEE_LIMIT (its y range, -90..0 degrees; x and z none), an
/// elbow (Forearm) by RAGDOLL_ELBOW_LIMIT (y, -100..0); a hip (Thigh) swings at most
/// RAGDOLL_HIP_FORWARD forward, RAGDOLL_HIP_BACK back (its y range, -60..110) and
/// RAGDOLL_HIP_SIDE to either side (z, -20..20) of its way in the bind pose. Not traced, so the
/// demo's reading: which bend axis is which and their signs (the joints' frames come from the
/// parent node, FUN_00039510): the larger range is the forward swing, as a hip's is; a knee or
/// elbow bends the way it was bent at death, 0 being straight; the hip's zero is the bind pose.
/// The hip's twist (x, -45..45) has no counterpart in the demo's point bodies, nor have the
/// shoulders', spine's and head's limits.
const RAGDOLL_KNEE_LIMIT: f32 = std::f32::consts::FRAC_PI_2;
const RAGDOLL_ELBOW_LIMIT: f32 = 100.0 * std::f32::consts::PI / 180.0;
const RAGDOLL_HIP_FORWARD: f32 = 110.0 * std::f32::consts::PI / 180.0;
const RAGDOLL_HIP_BACK: f32 = 60.0 * std::f32::consts::PI / 180.0;
const RAGDOLL_HIP_SIDE: f32 = 20.0 * std::f32::consts::PI / 180.0;
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

/// Weapon-switch sounds, hard-coded in default.xbe (0xcbfda, the HUD weapon panel's row state
/// setter FUN_000cbec0, played through FUN_000bb220: heard for the controlled character's switch
/// only, `start_switch`): the
/// first when a switch starts, the second when it completes; in a capture of the game they play
/// 0.63-0.66 s apart.
const SWITCH_SOUNDS: [u32; 2] = [0xFF82_0FDA, 0xE1E9_7460];
const SWITCH_SOUND_GAP: f32 = 0.65;
/// Handing control to another squad member: one sound as the camera cuts to them (common bank,
/// 0.18 s; a capture has it at all five hand-overs, lined up with the cut).
const SQUAD_SWITCH_SOUND: u32 = 0x0794_9F0D;
/// seconds the HUD lists every weapon after a switch (as in the game capture)
const HUD_LIST_TIME: f32 = 2.5;
/// reload length when the definition has none (the capture shows ~0.7 s)
const RELOAD_TIME: f32 = 0.7;
/// the game's crosshair sits above the screen centre: y = 192 of 480 (play_hud.rs's CROSSHAIR_Y,
/// measured in every weapon's take01), i.e. this fraction of the half-height above the middle;
/// shots and aiming go through it
const CROSSHAIR_UP: f32 = (240.0 - 192.0) / 240.0;
/// FUN_0011aa20, which runs the scope's ramp (its zoom +0x7ac and its progress +0x7cc) and the
/// scope button's lockout (+0x7c8), is called twice a game frame for a character a player
/// controls: by the character's own tick (FUN_0011caa0, its vtable's +0x6c at 0x39de44; the call
/// at 0x11cf03) and by its follow camera's (FUN_000702a0, the camera's +0x6c, through
/// FUN_0011c0b0 at 0x70728 / 0x718c4). So game-options' 0.3 s zoom times (`ScopeOptions`) run
/// in 0.15 s, and the lockouts in half theirs: the takes measure 0.17 +- 0.03 s a level, and the
/// L-Shot-50's second level grew 2.0-2.3 zoom units a game frame (todo/61 take01) against 2.22 at
/// two calls (1.11 at one). Medium confidence on the double call (read from the code; the timing
/// from the takes).
const SCOPE_CALLS: f32 = 2.0;
/// CT_AIM_SNIPE (0x12241a): entering the scope sets its button's lockout (+0x7c8) to this, and a
/// press that would leave it waits for it to run out (0.425 s with SCOPE_CALLS). Medium
/// confidence: read from the code; no take presses that fast.
const SCOPE_IN_LOCK: f32 = 0.85;
/// FUN_001249b0 (0x124a1c): leaving the scope sets the lockout to this, and a press to enter it
/// again waits for it to run out (0.45 s). Medium confidence, as SCOPE_IN_LOCK.
const SCOPE_OUT_LOCK: f32 = 0.9;
/// A weapon that zooms (WeaponDef::zoom, h_efce7f77, above 0: its scope has levels) has a scope:
/// in it the camera moves in behind the character's head (play_camera.rs: 0.5 m behind a point
/// 0.5 m above its root node, by the scope's progress), the field of view narrows by the level's
/// zoom, the character's model is hidden, and its snipe-sound breathing plays in a loop
/// (BREATH_GAP between breaths) while it stays there.
const BREATH_GAP: f32 = 0.6;
const BREATH_VOLUME: f32 = 0.6;
const SCOPE_SOUND_VOLUME: f32 = 0.8;
/// Flint, the squad's sniper (a synthetic), holds her scope dead still (a capture); the others'
/// aim wanders while they look through one: a slow drift of up to SWAY radians. (Her scope's
/// second level is her skill's: play_skills.rs `scope_levels`, `scope_zoom`.)
const STILL_SNIPER: usize = 1;
const SWAY: f32 = 0.012;
/// The vertical field of view (rad): the game's follow camera's 70 degrees across its 4:3 picture
/// (the levels' follow-camera archetypes' fov, every one of the 54 levels' 70: the camera's
/// attach 0x6ba30 -> FUN_0006e450 halves it into the view's +0x14, the aspect held to 4:3), so
/// 2 atan(0.75 tan 35) = 55.413 degrees down. The takes' live read: fov 70, half vFOV 27.706
/// (todo/126 notes, the field-of-view table), and the barrel and Tex's feet project where the
/// picture shows them (`fov-check.png`). The demo keeps this vertical at any window's shape and
/// shows more at the sides (86.1 degrees across at 16:9, the takes' black bars), so the window's
/// central 4:3 is the game's picture; a window narrower than 4:3 shows less than the game's 70
/// across (the product owner's call, #269: kept vertical always).
const FOV_Y: f32 = 0.967_138;

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
    /// the character type it was built from (`Player::kind`)
    kind: u32,
    /// what it's made of: the world material of its parts' collision (`Game::flesh`: 21 FLESH_HUMAN,
    /// 24 SYNTHETIC for Flint, 23 SHRIKE ...; play_fx.rs picks its hit effects and blood by it)
    flesh: i64,
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
    /// its motion set's `pc-transition-ready` and `npc-transition-ready` (in-time, out-time), in
    /// seconds (bf/character_type.rs `MotionSet`, #140): the gun's raise before a first shot and
    /// the ready pose's blend in and out (`ready_times`); READY_FALLBACK for a type without a set
    ready_pc: (f32, f32),
    ready_npc: (f32, f32),
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
    /// the skill's toggle overlays, [on, off] by stance (the motion set's OMT_SPECIAL_ON / OFF,
    /// `Game::skill_overlays`; play_skills.rs): Tex's Sc_w1_bz_start / bz_stop, Hawk's
    /// Sc_w1/w2_activate_stealth. (Brutus's Sc_w1/w2_spirit is loaded too, but his toggle never
    /// plays it: it returns 0, FUN_0013bfe0.)
    skill_clips: [[Option<OverlayClip>; 2]; 2],
    /// its type's MOTION table resolved to its clips (#129, play_enemies.rs): an enemy's gait,
    /// reactions and the test hook's motions
    motions: enemies::Motions,
}

/// A shot for the effects system: from the muzzle along `dir`, `dist` metres to a hit (or range).
/// `hit`: it ends on the world (a spark there); a body hit gets the flesh effects instead.
/// A shot on its way to a squad member: it lands after `delay` s (blood at `local` from them),
/// its hit record made at the shot (the roll, the falloff, the hit part struck, the own side's
/// share: play_damage.rs), the victim's own factors taken as it lands.
#[derive(Clone, Copy)]
struct PendingHit {
    delay: f32,
    member: usize,
    hit: damage::Hit,
    dir: Vec3,
    local: Vec3,
    ammo: i64,
}

/// A hit to show on a character (`Player::blood`, play_fx.rs): where (world), the way it went (the
/// push's, world), the ammo type (-1: a blast), and what it did for the game's blood decals
/// (FUN_00124a70, #129): `a`, the amount over the victim type's damage-multiplier (0 for a shot
/// into a body on the ground), and the hit points left after it. `shielded`: a stamina shield
/// took it whole (play_enemies.rs: no blood, the shield's look). `record`: a shot's hit record
/// damage (`damage::Hit::record`), which sets the user parameter of the struck material's hit
/// effect (FUN_0022cd80, play_fx.rs `flesh_sp`); None for any other hit. `burn`: a liquid's hit
/// (`burn`): only the game's decals show it.
#[derive(Clone, Copy, Debug)]
struct Bleed {
    at: Vec3,
    dir: Vec3,
    ammo: i64,
    a: f32,
    hp_left: f32,
    shielded: bool,
    record: Option<f32>,
    burn: bool,
}

impl Bleed {
    /// A hit to show that did no damage (a shot into a body on the ground).
    fn shown(at: Vec3, dir: Vec3, ammo: i64) -> Self {
        Bleed { at, dir, ammo, a: 0.0, hp_left: 0.0, shielded: false, record: None, burn: false }
    }
}

#[derive(Clone, Copy)]
struct Shot {
    /// the gun that fired it (its `Game::weapons` key)
    weapon: u32,
    origin: Vec3,
    dir: Vec3,
    dist: f32,
    hit: bool,
    speed: f32,
    /// the weapon's effect types: in flight, and where the shot lands (0: none)
    flight: u32,
    hit_fx: u32,
    /// the weapon's Damage min..max and damage-type (play_scenery.rs: what it takes off an
    /// object it hits; `update_player`: the roll on a body), and its ammo-type (the hit's blood
    /// and sparks). Each shot carries its own gun's: with two guns firing (Tex's berserk) a frame's
    /// shots aren't all the gun in hand's
    damage: [f32; 2],
    damage_type: i64,
    ammo: i64,
    /// the bullet's hole or scorch (h_06a27365; 0 none) and the normal of the surface the shot
    /// meets (play_shots.rs)
    decal: u32,
    normal: Vec3,
    /// the Damage's falloff mode (h_fb124e6c) over the bullet's range (`Shot::falloff_at`)
    falloff: i64,
    range: f32,
    /// a bolt: bullet-type 0 or 3, the pooled object FUN_0022bac0 launches, which finds what it
    /// hits as it flies, on its sweeps (play_bolts.rs, #136): its `dist` and `hit` are only what
    /// the world held on its line at the shot (the AI's "shot at" reactions read them)
    bolt: bool,
    /// it stops on a body `dist` m out (`update_player`; `hit` is false then: the body's blood is
    /// play_fx.rs's)
    body: bool,
    /// an instant ray that passes through what it hits (bullet h_11caf57b with more than one hit
    /// to spend: the Rails, FUN_00230860), whose hits play_rays.rs finds at its shot, all along
    /// its line (#238); `dist` is then where it stops
    pass: bool,
    /// fired from the holder's scoped eye (`origin` the eye: the weapon's +0x225 bit 1), where an
    /// instant ray's beam starts off to the side (`beam_line`, FUN_002312e0)
    scoped: bool,
}

#[derive(Resource)]
struct Player {
    /// the squad slot (a CHARACTERS index): who of the four this is, or OTHER
    character: usize,
    /// its character type (#128: a `Game::character_types` key, the hash of the type's `<base
    /// name>`): the squad's own (`squad_type`), or a test map NPC's of any type. Its model,
    /// clips, HP, weapons, flesh and decals come from it
    kind: u32,
    /// its name for logs (`unit_name`), set by `spawn_unit`: its squad slot's ("brutus"), or for
    /// OTHER its type's display name (`Game::type_label`); "" before it's spawned. Kept here
    /// rather than in `Loaded`, which the update systems take out of the Player while they step it
    name: String,
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
    /// the gun drawn into the second hand (Tex's berserk, play_skills.rs), fired with the gun in
    /// hand
    second: Option<SecondGun>,
    /// per weapon: rounds in the clip, and rounds taken from the squad's reserve for the reload
    /// under way (the game's weapon +0x1f8 and +0x1e8). The reserve itself is the squad's, per
    /// ammo-type (play_ammo.rs's `Reserve`).
    ammo: Vec<[i64; 2]>,
    /// per weapon: its recharge state (the game's weapon +0x220 timer and +0x218 cooldown; see
    /// play_ammo.rs's `recharge`)
    regen: Vec<ammo::Charge>,
    /// reload in progress: (weapon, seconds left, clip it fills to)
    reloading: Option<Reload>,
    /// grenades carried, by type (`grenade::GrenadeKits` order; filled by play_inventory.rs's
    /// `stock` once the kits are loaded: the squad placements' own, #281)
    grenades: Vec<i64>,
    /// medkits carried (shared by the squad, like the grenades; the placements' at the start,
    /// play_inventory.rs; pickups, play_pickups.rs)
    medkits: i64,
    /// the squad's other items (the OrgSen, the Hat, "Alien Technology Acquired!"), in the
    /// store's (category, index) order: play_inventory.rs. Shared, as the grenades are
    others: Vec<inventory::Carried>,
    /// the squad's items have been stocked from the start (play_inventory.rs `stock`): a respawn
    /// refills the grenades only
    stocked: bool,
    /// the item box's selection: the inventory front's wanted pair (inv+0x1c0), None the game's
    /// (10, -1); it goes with control, as the game's hand-over copies it (FUN_000c8fa0). And the
    /// use key (G) pressed this frame for an item that isn't a grenade, as the item controls
    /// decide it (play_itembox.rs `item_controls`)
    item: Option<Item>,
    item_use: bool,
    /// a medkit being used: time into the use_item clip; `item_in_hand` while the clip has it
    /// (its 0a6e8f79 event to its 19f8311b), `item_used` set the frame it's used (heal, sound,
    /// drop: see play_pickups.rs)
    using: Option<f32>,
    item_in_hand: bool,
    item_used: bool,
    /// the inventory item they are (what using one heals)
    medkit_kind: u32,
    /// the Medkit stack the use under way takes from (the selected one as it started: the (5, 0)
    /// Medkit or the 60-point (5, 2) one, play_pickups.rs)
    use_stack: Item,
    test_medkit_used: bool,
    /// throw button held: charging (the HUD meter shows `charge`); for the controlled character
    /// as the item controls decide it (play_itembox.rs)
    throw_held: bool,
    charge: f32,
    /// the throw meter: its level and its timer (s; METER_TIME while charging, then counting
    /// down: the meter's +0x3c and +0x40, 0xad950)
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
    /// stamina and its maximum (the game's char+0x490 / +0x494: both the type's max-stamina at
    /// spawn, 0x12f46e), and the special skill it pays for (play_skills.rs)
    stamina: f32,
    max_stamina: f32,
    skill: skills::Skill,
    dead: bool,
    /// the body once dead (physics on the skeleton)
    ragdoll: Option<Ragdoll>,
    /// push given to the body when it dies (world space, m/s)
    death_push: Vec3,
    /// the last animated pose's bone matrices (model space): where the ragdoll starts
    last_world: Vec<Mat4>,
    /// its radar-contact (the character's +0x7a, `character::H_RADAR_CONTACT`): 1,
    /// RCONTACT_ALIGNMENT, for everyone (the squad's set-up, FUN_0008b300 at 0x8b3ef; every
    /// character in e34 read 1 live in #141). The crosshair turns yellow on a 2 or a 4
    /// (play_reticle.rs): only a test hook sets one (BF_TEST_NPC_RADAR, play_testtools.rs)
    radar_contact: u8,
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
    /// hits to show (blood, sparks, hit sound, decals: play_fx.rs)
    blood: Vec<Bleed>,
    /// the body has hit the ground (thud played) / where it lies
    thud: bool,
    body_at: Option<Vec3>,
    /// seconds since death; the memory chip (play_dna.rs) and the blood pool under the body are
    /// out
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
    /// the steep terrain face they're sliding down (its normal), in contact with it (#264: the
    /// tutorial's lava pit wall; see `slide_on_face`); None off one
    face: Option<Vec3>,
    /// how far (m) their last move down a steep face brought them down (`face_move`): how far it
    /// may fall away under them and still hold them
    face_drop: f32,
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
    /// this frame's shots of a mesh-projectile gun (bullet-type 2 steered at a steady speed: the
    /// Therm Sweeper, the Sweeper V), launched as missiles by play_missile.rs instead of flying
    /// as `shots`
    launches: Vec<missile::Launch>,
    /// the player's shots still flying toward squad members
    pending_hits: Vec<PendingHit>,
    shots_fired: u32,
    /// the fire loop: game frames, bursts, accuracy, tracers, casings (play_shots.rs)
    fire_state: shots::FireState,
    // sound
    surface: usize,
    foot_prev: [f32; 2],
    sound_queue: Vec<(u32, f32)>,
    rng: u32,
    /// seconds of no footsteps after a landing (its thud covers the feet touching down)
    step_mute: f32,
    // camera
    /// the aim (rad): its heading (0 down -z, + to the left) and pitch (+ up, held to
    /// +-camera::PITCH_LIMIT), the game's aim frame char+0x5bc, which the follow camera's view is
    /// (play_camera.rs). The pitch starts level (the takes' characters rest at -0.01..-0.02; the
    /// game's at spawn isn't recorded)
    cam_yaw: f32,
    cam_pitch: f32,
    /// the follow camera's state (play_camera.rs `Rig`): the controlled character's runs
    rig: camera::Rig,
    /// the height (m above the feet) of the head the follow camera pivots on (char+0x48c: its
    /// "Bip01 Head" bone at spawn, `camera::head_height`)
    head_height: f32,
    /// the scope: its progress, 0..1 (the game's char+0x7cc: the follow camera blends into the
    /// scope's camera by it, play_camera.rs) and the zoom it gives (+0x7ac: 1 none, the level's
    /// zoom in it), both ramped linearly (`follow_camera`)
    scope: f32,
    zoom: f32,
    /// the character's breathing in the scope (its snipe-sound, how long it lasts) and when it's
    /// next due
    snipe_sound: Option<(u32, f32)>,
    breath_in: f32,
    /// the scope's in / out sounds (the type's +0x260 / +0x264: the character's h_00dd1fe0 /
    /// h_02c94d34), and the level they were last played for (`follow_camera`)
    scope_sounds: (u32, u32),
    scope_heard: u8,
    /// the scope's level (char+0x7a8: 0 out; 1 in; 2 closer, for Flint's skill only), stepped by
    /// the scope button (right mouse, `press_scope`)
    scope_level: u8,
    /// the scope button's lockout (char+0x7c8, s): set as the scope is entered (SCOPE_IN_LOCK) and
    /// left (SCOPE_OUT_LOCK); the press that would leave or enter it again waits for it
    scope_lock: f32,
    /// (BF_TEST_SCOPE_PRESS presses done)
    test_scope_presses: usize,
    /// (BF_TEST_SELECT done)
    test_selected: bool,
    /// the scoped aim's wander (yaw, pitch; radians)
    sway: Vec2,
    /// the effects it starts this frame, each played once (where and how turned, the ALE effect,
    /// its user parameter): its liquids' splashes and rings (play_liquids.rs), a shield's look
    /// (play_enemies.rs); `spawn_splashes` spawns them
    splashes: Vec<(Transform, u32, f32)>,
    /// the sounds it made this frame with play parameters of their own (`WorldSound`: the liquids'
    /// splashes and wading steps, an instant ray through a surface; #5), which `play_sounds` hears
    /// from the listener
    world_sounds: Vec<WorldSound>,
    /// its hit parts in the liquids (play_liquids.rs: the game's wade entries and char+0x104)
    wading: liquids::Wading,
    /// the side the character is on: 0 the squad (everyone in the main game); BF_TEST_HOSTILE puts
    /// a squadmate on another (play_sentry.rs: the squad's mines go off for them), and the test
    /// map's enemy NPCs are on ENEMY_TEAM (play_testtools.rs: the squad AI fires at them)
    team: u8,
    /// held where a test hook puts it, without the squad AI (play_sentry.rs's BF_TEST_HOSTILE
    /// and BF_TEST_MINE_FRIEND)
    test_hold: bool,
    /// a character spawned by the test map's NPC tool (play_testtools.rs): not one of the squad
    /// (no formation, can't be taken control of), run by `testtools::npc_ai`
    npc: Option<testtools::Npc>,
    /// the AI's target point (a hostile's chest, see `ai_fire`): the gun is turned onto it and
    /// shots go to it, instead of to what the camera's crosshair is on. None for the player.
    ai_target: Option<Vec3>,
    /// the controlled character's aim point while the crosshair holds a lock (play_reticle.rs
    /// `Reticle::aim`, last frame's): the reticle puts its lock's point in char+0x804 (0xc5b90 ->
    /// FUN_0011e9d0), the character update aims its guns there (FUN_0011aa20 -> 0x22dc60), and
    /// each muzzle's shot goes to it (FUN_0022e5b0). None without a lock (the crosshair's own
    /// point) and for the AI
    lock_aim: Option<Vec3>,
    /// an enemy's own state (#129, play_enemies.rs: its personality, reactions, the test hook's
    /// drive, its stamina shield); the squad's stays at its default
    enemy: enemies::State,
    /// the map editor's stand-in for the controlled character while editing (`Player::stand_in`,
    /// play_editor.rs, #191): no body, never given one (`spawn_player`)
    stand_in: bool,
    /// its name as the level's triggers know it (play_triggers.rs): the name its spawn-trigger
    /// gives its copy (h_14f15a34: "tex_1", "TUTORIAL_terrorist_5"), or one the runtime makes up
    /// for a re-armable trigger's; 0 for a character no trigger spawned (the demo's own squad on a
    /// map without squad spawns, the test map's NPCs). Its death is a DESTROYED from that name
    trigger_name: u32,
    /// held by a cutscene (char+0x834 bit 0: FUN_00129470, its script's start or a 119;
    /// play_script.rs): its AI and its player's controls are off until the script releases it
    held: bool,
    /// a cutscene's walk order (code 105, play_script.rs): the world points it walks to in turn,
    /// the script moving it (char+0x834 bits 0 and 1); the controlled one's ends as the script
    /// lets it go, another's when it's there or stuck
    script_walk: script::Walk,
    /// not in the world yet: a squad member whose spawn-trigger hasn't spawned it (the game takes
    /// the template out of the world as the trigger links, FUN_000dfe90, and the tutorial's Tex
    /// comes with the opening's farcast at 17.1 s). Not drawn, not seen, no controls; a member
    /// out of the world has no AI, step, sounds or hits and takes no hand-over (a leader steps:
    /// its step runs the game's clock), until play_triggers.rs `spawn_copy` brings it (#222)
    absent: bool,
    /// the brain of a character a spawn-trigger made (#227, play_brain.rs: its home, patrol,
    /// target, aim); None for the squad and the test map's NPCs
    brain: Option<Box<brain::Brain>>,
    /// its AI's sensor (char+0x1b4, play_brain.rs `Sensor`): its sight distance and field of view,
    /// from its type and the placement it came from (#197), and the hostiles it knows
    sensor: brain::Sensor,
    /// a human player controls it (FUN_00112800's test, through FUN_0010fce0): its motion set's
    /// `pc-transition-ready` times are its, else `npc-transition-ready` (#140, `ready_times`). Set
    /// each frame by `update_player`: the controlled character's only
    controlled: bool,
    /// it stood in its ready set last frame (the gun up: `step_player`), so a change of set is
    /// blended over the ready times (#140)
    ready_pose: bool,
}

/// The character type of squad slot `c`: the campaign squad's types are named after them in
/// common's objecttypes (`<base name="brutus">` ...; no other archive defines those names). 0
/// for OTHER.
fn squad_type(c: usize) -> u32 {
    CHARACTERS.get(c).filter(|n| !n.is_empty()).map_or(0, |n| bf_viewer::bf::hash::h(n))
}

/// A character's name for logs: as spawned (`Player::name`: its squad slot's, "brutus", or for
/// OTHER its type's display name), before that its slot's name or its type's key.
fn unit_name(u: &Player) -> String {
    if !u.name.is_empty() {
        u.name.clone()
    } else if u.character == OTHER {
        format!("h_{:08x}", u.kind)
    } else {
        CHARACTERS[u.character].to_string()
    }
}

/// `unit_name` by character type alone: a squad type's slot name ("flint"), any other type's
/// display name (`Game::type_label`).
fn kind_name(kind: u32, game: &Game) -> String {
    (0..SQUAD).find(|&c| squad_type(c) == kind).map_or_else(|| game.type_label(kind), |c| CHARACTERS[c].to_string())
}

/// A character's name for logs by its identity alone (a shooter, a gas cloud's or a damage
/// area's victim): a squad member's slot name, or a test map NPC's type (`kind_name`) and number.
fn who_name(who: Who, game: &Game) -> String {
    match who {
        Who::Squad(c) => CHARACTERS[c].to_string(),
        Who::Npc(n, kind) => format!("{} (NPC {n})", kind_name(kind, game)),
    }
}

impl Player {
    fn new(character: usize) -> Self {
        Self {
            character, kind: squad_type(character), name: String::new(), loaded: None, position: Vec3::ZERO, yaw: 0.0, layers: vec![], gait: Gait::Idle,
            action: Action::None, action_left: 0.0, on_all_fours: false,
            height: 0.0, vy: 0.0, air_velocity: Vec3::ZERO, last_velocity: Vec3::ZERO, face_time: 0.0, sim_time: 0.0,
            move_input: Vec2::ZERO, sprint: false, walk: false, aim: false, jump_pressed: false, jump_buffer: 0.0, dodge_pressed: false,
            next_surface: false, fire: false, switch_pressed: false, twist: 0.0,
            weapon: 0, weapon_dirty: true, holding: true, switching: None, second: None, ammo: vec![], regen: vec![], reloading: None, grenades: vec![], medkits: 0, others: vec![], stocked: false, item: None, item_use: false, using: None, item_in_hand: false, item_used: false, medkit_kind: 0, use_stack: Item::Medkit, test_medkit_used: false, throw_held: false, charge: 0.0, meter_after: (0.0, 0.0), place_latch: false, throwing: None, pending_release: None, thrown: vec![], select: None, quote_in: None, speaking: 0.0, health: 100.0, max_health: 100.0, stamina: 100.0, max_stamina: 100.0, skill: Default::default(), dead: false, ragdoll: None, death_push: Vec3::ZERO, last_world: vec![], radar_contact: 1, hurt_quiet: 0.0, knock: None, knock_request: None, knock_cooldown: 0.0, crouch_wanted: false, still_time: 0.0, kneel_jitter: 0.0, face_yaw: None, dodge_request: None, dive_from: None, idle_cautious: false, idle_left: 0.0, leash: 15.0, blood: vec![], thud: false, body_at: None, dead_for: 0.0, dna_done: false, prev_xz: Vec2::ZERO, sliding: 0.0, slide_amount: 0.0, slide_active: false, slide_vel: Vec3::ZERO, slide_yaw: 0.0, fall_from: 0.0, was_air: false, face: None, face_drop: 0.0, slide_on: false, slide_fx: None, pool_done: false, death_response: None, ai_delay: 0.0, ai_burst: false, ai_phase: 0.0, reload_pressed: false, use_held: false, reload_wanted: false, hud_list: 0.0, show_help: false, switch_sound_in: -1.0, cooldown: 0.0, aim_hold: 0.0, muzzle_off: 0.0, aim_weight: 0.0, aim_residual: 0.0, recoil: 0.0, flash: 0.0, lift_now: 0.0,
            spin: 0.0, spin_angle: 0.0, shots: vec![], launches: vec![], pending_hits: vec![], shots_fired: 0, fire_state: Default::default(),
            surface: usize::MAX, foot_prev: [1.0; 2], sound_queue: vec![], rng: 0x1234_5678, step_mute: 0.0,
            cam_yaw: 0.0, cam_pitch: 0.0, rig: Default::default(), head_height: 2.0,
            scope: 0.0, zoom: 1.0, snipe_sound: None, breath_in: 0.0, scope_sounds: (0, 0), scope_heard: 0, scope_level: 0, scope_lock: 0.0, test_scope_presses: 0, test_selected: false, sway: Vec2::ZERO,
            splashes: vec![], world_sounds: vec![], wading: Default::default(), team: 0, test_hold: false, npc: None, ai_target: None,
            lock_aim: None,
            enemy: Default::default(),
            stand_in: false,
            trigger_name: 0,
            held: false,
            script_walk: Default::default(),
            absent: false,
            brain: None,
            sensor: Default::default(),
            controlled: false,
            ready_pose: false,
        }
    }

    /// The map editor's stand-in for the controlled character while editing (play_editor.rs,
    /// #191). In edit mode no character is in the world, but every system takes `Player` as the
    /// one in control, so this takes its place: it has no body (`loaded` stays None, and
    /// `spawn_player` gives it none) and counts as dead, so everything that acts on characters
    /// passes it over (sight and the AI's targets, shots, blasts, gas, Sentries, pickups, doors,
    /// the death camera's hand-over, the squad's AI); the marks of a death are counted as made
    /// (`dead_for` past the death's frame, the blood pool, the memory chip). `character`: the
    /// squad member who leads when play starts again; `sim_time`: the clock the test hooks run
    /// on, carried on (the editor counts it on, as `step_player` counts a living one's); `at`:
    /// where it stands in (the edit camera: the squad's and NPCs' sounds are heard from there,
    /// `play_sounds`, and the NPC menu puts its NPCs facing it).
    fn stand_in(character: usize, sim_time: f32, at: Vec3) -> Self {
        let mut p = Player::new(character);
        p.stand_in = true;
        p.dead = true;
        p.health = 0.0;
        p.dead_for = 1.0;
        p.thud = true;
        p.pool_done = true;
        p.dna_done = true;
        p.sim_time = sim_time;
        p.position = at;
        p.prev_xz = at.xz();
        p
    }

    /// A grenade throw (or set-down) cut short - knocked down, or control handed over - before
    /// the grenade left the hand: it goes back into the inventory (the count dropped at the
    /// button).
    fn cancel_throw(&mut self) {
        let mut back = self.throwing.take().filter(|t| !t.released).map(|t| t.kind);
        back = back.or(self.pending_release.take().map(|(_, k, _)| k));
        if let Some(n) = back.and_then(|k| self.grenades.get_mut(k)) {
            *n += 1;
            if std::env::var("BF_GRENADE_LOG").is_ok() {
                println!("t {:.2}: throw cut short, grenade back ({n} now)", self.sim_time);
            }
        }
        // (`thrown` is kept: a grenade already released this frame still flies)
        self.pending_release = None;
        self.charge = 0.0;
        self.meter_after = (0.0, 0.0);
    }

    /// One of the squad (Brutus, Flint, Hawk, Tex as the game has them), not a test map NPC: the
    /// one control can pass to, shown on the HUD's portraits.
    fn in_squad(&self) -> bool {
        self.npc.is_none()
    }

    /// Who this is, for effects that reach a character later (Energy bolts, Sonic rings): a
    /// squad member by character (control passing between them swaps their places), a test map
    /// NPC by its number (several can share a squad character).
    fn who(&self) -> Who {
        match self.npc {
            Some(n) => Who::Npc(n.id, self.kind),
            None => Who::Squad(self.character),
        }
    }

    fn random(&mut self, n: usize) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng as usize) % n.max(1)
    }

    /// A draw in [0, 1) of its random state (`random`): the demo's stand-in for the game's
    /// FUN_00233b90 (a unit draw of the one LCG every system shares), for a sound's pitch jitter
    /// (play_liquids.rs).
    fn unit(&mut self) -> f32 {
        self.random(1 << 24) as f32 / (1 << 24) as f32
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

/// A character's identity (`Player::who`): a squad member's character, or a test map NPC's
/// number, with its character type (fixed for the NPC: for naming it, `who_name`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Who {
    Squad(usize),
    Npc(u32, u32),
}

impl Who {
    /// A number for seeding an effect: the character, or 100 + the NPC's number.
    fn seed(self) -> u32 {
        match self {
            Who::Squad(c) => c as u32,
            Who::Npc(n, _) => 100 + n,
        }
    }
}

/// The character `who` is: the controlled one or one of `squad`, and whether it's the
/// controlled one.
fn unit_mut<'a>(player: &'a mut Player, squad: &'a mut Squad, who: Who) -> Option<(&'a mut Player, bool)> {
    if player.who() == who {
        return Some((player, true));
    }
    squad.0.iter_mut().find(|m| m.who() == who).map(|m| (m, false))
}

/// Squad AI: keep to a formation place round the leader (run / sprint to catch up, walk the
/// last bit) and look where the leader looks. They shoot only at an enemy they can see
/// (`target`, from `hostile_in_sight`). Reloads happen by themselves (empty clip).
fn squad_ai(m: &mut Player, leader: &Player, slot: usize, heading: f32, dt: f32, target: Option<Vec3>, toward: Option<Vec3>) {
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
    // behind (or past the data's leash + 10 m), walk the last bit; but not on into a liquid that
    // burns (#1, play_liquids.rs `burning_ahead`: the game's routes have no point in one): they
    // wait at its shore
    let go = (if moving { dist > SLOT_STOP } else { dist > SLOT_START }) || (in_lane && dist > 0.3);
    let shore = go && dist > 1e-4 && world::arena().is_some_and(|a| liquids::burning_ahead(&a, m, Vec2::new(to.x, to.z) / dist));
    // (BF_LIQUID_LOG: twice a second while it waits there)
    if shore && std::env::var("BF_LIQUID_LOG").is_ok() && (m.sim_time * 2.0).floor() != ((m.sim_time - dt) * 2.0).floor() {
        println!("t {:.2}: {} waits at a liquid that burns, {dist:.1} m from its place, at {:.2}", m.sim_time, unit_name(m), m.position);
    }
    let go = go && !shore;
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
                    println!("{} idles: {} for {} s", unit_name(m), if m.idle_cautious { "cautious" } else { "crouch" }, m.idle_left);
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
    // squadmates only aim and shoot at an enemy they can see (`hostile_in_sight`) - never just
    // because the player fires. The main game has no enemies yet, so they hold fire; the test
    // map's enemy NPCs (play_testtools.rs) are the only targets. (Nor while their skill says hold
    // fire: `update_player` leaves the target out then)
    // a known enemy it doesn't see (it shot from behind, or went behind cover): turned to its last
    // place with the gun up (`toward`: the brain's turn toward a known contact, FUN_0002f960), so
    // its field of view finds it
    let turned = target.is_none() && toward.is_some_and(|p| face_toward(m, p));
    m.aim = target.is_some() || cautious || turned;
    ai_fire(m, target, dt);
    m.switch_pressed = false;
    m.reload_pressed = false;
    m.throw_held = false;
    m.next_surface = false;
}

/// An AI character's fire at `target` (a hostile's chest, from `hostile_in_sight`): it faces it
/// and the gun is turned onto it (`ai_target`); it picks a random moment to open up, then fires
/// in bursts and pauses of its own length. Without a target it holds fire. (The squad AI's
/// existing burst timing; the aim is exact: the demo's choice, no spread is read.)
fn ai_fire(m: &mut Player, target: Option<Vec3>, dt: f32) {
    // a target with the gun down (its ready timer out: none started by a raise or a shot in the
    // last 4 s): the attack check raises it (FUN_00024f80 -> FUN_00123ea0 at 0x2509c), and its
    // shots go through the fire command (FUN_0004dfb0: CT_FIRE_WEAPON, 0x1225d7), which fires
    // nothing until the timer is down to 4: so its first round waits its npc in-time (#140,
    // `raise`), alongside the demo's own opening delay (`ai_delay`). (No target before its model
    // is in: it can't raise, step or fire yet)
    let up = m.loaded.as_ref().map(|l| ready_times(m, l).0);
    let target = target.filter(|_| up.is_some());
    if let (Some(_), Some(in_time)) = (target, up) {
        if !m.fire_state.raised() {
            raise(m, in_time);
        }
    }
    m.ai_target = target;
    let Some(t) = target else {
        m.fire = false;
        m.ai_burst = false;
        m.ai_phase = 0.0;
        m.ai_delay = 0.2 + 0.6 * (m.random(1000) as f32 / 1000.0);
        return;
    };
    // its "camera" looks at the target (the upper body and the feet turn to it)
    let to = t - (m.position + Vec3::Y * (GROUND + m.height + AI_EYE));
    m.cam_yaw = (-to.x).atan2(-to.z);
    m.cam_pitch = (to.y / to.length().max(1e-3)).clamp(-1.0, 1.0).asin().clamp(-camera::PITCH_LIMIT, camera::PITCH_LIMIT);
    m.face_yaw = Some(m.cam_yaw);
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
}

/// An AI character with nothing to fire at turns to `p` (a known contact's last place, play_brain.rs
/// `Sensor::toward`): its "camera" and feet onto it, as `ai_fire` does onto a target. True once
/// set (false for a point on top of it).
fn face_toward(m: &mut Player, p: Vec3) -> bool {
    let to = p - (m.position + Vec3::Y * (GROUND + m.height + AI_EYE));
    if to.xz().length_squared() < 1e-4 {
        return false;
    }
    m.cam_yaw = (-to.x).atan2(-to.z);
    m.cam_pitch = (to.y / to.length().max(1e-3)).clamp(-1.0, 1.0).asin().clamp(-camera::PITCH_LIMIT, camera::PITCH_LIMIT);
    m.face_yaw = Some(m.cam_yaw);
    true
}

/// Where an AI character looks from (m above the feet) and where it aims on a target (its
/// chest: m above the feet). The demo's choices, inside BODY_HEIGHT's standing cylinder.
const AI_EYE: f32 = 1.6;
const AI_AIM_HEIGHT: f32 = 1.1;
/// How far (m) an AI character engages a hostile without a weapon range to go by (the shots'
/// own floor, `step_player`'s `range.max(20)`).
const AI_MIN_RANGE: f32 = 20.0;

/// How a character can be seen, for `hostile_in_sight` (`sight_list`).
#[derive(Clone, Copy, Debug)]
struct Seen {
    /// who it is (a level character's brain keeps its contacts by it: play_brain.rs)
    who: Who,
    /// its team
    team: u8,
    /// its chest point, where the AI aims (AI_AIM_HEIGHT above the feet)
    chest: Vec3,
    /// a target for the AI at all: alive, on its feet, not held by a test hook, in the world (a
    /// squad member waiting for its spawn-trigger isn't: `Player::absent`), and not a test map
    /// dummy (play_testtools.rs: a dummy is the player's target; one set to fight is everyone's)
    target: bool,
    /// alive and in the world: a brain keeps it as a contact while it is, knocked down or not
    /// (the scan drops a contact only when it's dead, FUN_00044140; play_brain.rs)
    alive: bool,
    /// how visible it is (the skill's +0xe4, which the stamina update pushes into its combat
    /// target each frame, 0x11a934-0x11a94a: `skills::visibility`): 1, or a stealthed Hawk's
    /// near 0
    visibility: f32,
}

/// How each character can be seen (`Seen`), in the order `hostile_in_sight`'s `me` counts: the
/// player, then the squad.
fn sight_list<'a>(units: impl Iterator<Item = &'a Player>) -> Vec<Seen> {
    units.map(|u| Seen {
        who: u.who(),
        team: u.team,
        chest: u.position + Vec3::Y * (GROUND + u.height + AI_AIM_HEIGHT),
        target: !u.dead && !knocked_down(u) && !u.test_hold && !u.absent && u.npc.is_none_or(|n| n.fight),
        alive: !u.dead && !u.absent,
        visibility: skills::visibility(u),
    }).collect()
}

/// The nearest hostile `m` (unit `me` of `sight`: 0 the player, then the squad's order) sees now,
/// its place in `sight` and its distance, when it's within the held gun's reach (`ai_range`): by
/// its sensor (play_brain.rs `Sensor::scan`, #197: its sight distance x 1.15 for a contact x
/// the target's visibility, its field of view, the ray), without changing what it knows. For a
/// skill line's "fighting" condition (play_skills.rs); the AI itself sees by `brain::glance` or
/// `brain::sense`. Never anyone in the main game but the level's characters on another side;
/// BF_TEST_HOSTILE's held squadmate isn't a target either.
fn nearest_hostile(m: &Player, me: usize, sight: &[Seen]) -> Option<(usize, f32)> {
    brain::nearest_within(&m.sensor.scan(m, me, sight), ai_range(m))
}

/// The chest of the nearest hostile `m` can see (`nearest_hostile`).
fn hostile_in_sight(m: &Player, me: usize, sight: &[Seen]) -> Option<Vec3> {
    nearest_hostile(m, me, sight).map(|(i, _)| sight[i].chest)
}

/// How far (m) `m`'s AI fires: its held weapon's range, at least AI_MIN_RANGE. The shot's own
/// test, not sight's (FUN_00024f80 doesn't compare the distance with the gun's range;
/// todo/197-ai-sight/answer.md §2); a type with no vision-range sees this far too (the demo's).
fn ai_range(m: &Player) -> f32 {
    m.loaded.as_ref().and_then(|l| l.weapons.get(m.weapon)).map_or(AI_MIN_RANGE, |w| w.def.range.max(AI_MIN_RANGE))
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
        // (not for a test map NPC, nor by one)
        if !u.in_squad() {
            u.death_response = None;
        }
        if let Some((left, tag)) = u.death_response {
            u.death_response = if left - dt > 0.0 { Some((left - dt, tag)) } else { answers.push((u.character, tag)); None };
        }
    }
    for (dead, tag) in answers {
        let mut alive: Vec<&mut Player> = std::iter::once(&mut *player).chain(squad.0.iter_mut())
            .filter(|u| !u.dead && !u.absent && u.in_squad() && u.character != dead && u.speaking <= 0.0 && u.quote_in.is_none()).collect();
        if !alive.is_empty() {
            let n = alive.len();
            let k = alive[0].random(n);
            say(alive[k], &game.0, tag, 0.0);
        }
    }
    // test hook: BF_TEST_KILL=<character> - from 1 s, turn to that squad member and keep firing
    if let Some(c) = std::env::var("BF_TEST_KILL").ok().and_then(|v| v.parse::<usize>().ok()) {
        if let Some(m) = squad.0.iter().find(|m| m.character == c && m.in_squad()) {
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
    // (not to one out of the world yet, waiting for its spawn-trigger: the keys 1-4, BF_TEST_SELECT
    // and the autopilot's pick all come here)
    let Some(i) = squad.0.iter().position(|m| m.character == c && m.in_squad() && m.loaded.is_some() && !m.dead && !m.absent) else {
        player.select = None;
        return;
    };
    if left - dt > 0.0 {
        player.select = Some((c, left - dt));
        return;
    }
    let (pitch, help) = (player.cam_pitch, player.show_help);
    // the squad shares one inventory: the grenades and medkits go with control (a throw under
    // way is called off, its grenade back in the inventory)
    player.cancel_throw();
    // (the scope ends with control: the character left behind isn't left looking through it, its
    // shots from the eye at the scoped rate, nor its button locked out for a hand-back soon after;
    // the demo's choice, the game's hand-over isn't traced)
    player.leave_scope();
    (player.scope, player.zoom, player.scope_heard, player.scope_lock) = (0.0, 1.0, 0, 0.0);
    // (the selection too: the game's hand-over copies the old character's wanted pair into the
    // new one's front, FUN_000c8fa0; #196 take17 / take29: every member's box reads the item)
    let inventory = (player.grenades.clone(), player.medkits, player.medkit_kind, player.others.clone(), player.stocked, player.item);
    std::mem::swap(&mut *player, &mut squad.0[i]);
    (player.grenades, player.medkits, player.medkit_kind, player.others, player.stocked, player.item) = inventory;
    // (a script's walk was its brain's goal: under the player's control it has none)
    player.script_walk.clear();
    // the switch sound, at the cut (a capture: the same sound at every hand-over)
    player.sound_queue.push((SQUAD_SWITCH_SOUND, 0.8));
    squad.0[i].select = None;
    player.select = None;
    player.cam_pitch = pitch;
    // the cut lands behind them, clear of walls (the reference recording's cut to Hawk: from
    // straight behind, the whole body in view), the camera put there at once (take21: the
    // picture jumps to Brutus's camera, no ease)
    player.cam_yaw = clear_follow_yaw(&player, &game.0, player.yaw);
    player.rig.cut = true;
    // the squad AI's aim, fire, walking and idling don't carry over: the cut is to the plain
    // follow camera (no aim shift: its timer is 0 for a character no player controls), and they
    // stand where they face instead
    // of walking on, turning to the old leader's heading (face_yaw) or kneeling on (the demo's
    // choice: the recording's Hawk stands at the cut, but whether a kneeling member stands up is
    // a guess)
    player.move_input = Vec2::ZERO;
    player.aim = false;
    player.fire = false;
    player.aim_hold = 0.0;
    player.face_yaw = None;
    // (nor the target it was firing at: the gun and the body would stay on it)
    player.ai_target = None;
    player.ai_burst = false;
    player.crouch_wanted = false;
    player.idle_cautious = false;
    player.show_help = help;
    player.weapon_dirty = true;
    player.hud_list = 0.0;
    player.fire_state.drop_pull();
    for m in squad.0.iter_mut() {
        m.move_input = Vec2::ZERO;
        m.aim = false;
        m.fire = false;
        // (a tap still owed its shot doesn't go on firing for the one handed over from)
        m.fire_state.drop_pull();
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

/// Whether the map is the flat test floor: no arena, or the one the test tools' scenery gave it
/// (#185, `world::Arena::flat`: its ground and the pieces put on it).
fn flat_floor() -> bool {
    world::arena().is_none_or(|a| a.is_flat())
}

/// Pillars of the test ground (centres); they stop shots.
fn pillars() -> impl Iterator<Item = Vec3> {
    (-3..=3).flat_map(|i| (-3..=3).map(move |j| (i, j))).filter(|_| flat_floor())
        .filter(|&(i, j): &(i32, i32)| (i + j) % 2 == 0 && (i, j) != (0, 0))
        .map(|(i, j)| Vec3::new(i as f32 * 9.0, GROUND + 1.5, j as f32 * 9.0))
}

/// Distance along a ray to the ground, a pillar or a breakable object the test map's tools put
/// down (flat floor; with the scenery put on it, #185, its arena's ground and pieces), or the
/// map's collision, if within `max`.
fn ray_hit(origin: Vec3, dir: Vec3, max: f32) -> Option<f32> {
    let arena = world::arena();
    if let Some(a) = arena.as_ref().filter(|a| !a.is_flat()) {
        return a.ray(origin, dir, max);
    }
    let mut best = match &arena {
        Some(a) => a.ray(origin, dir, max),
        None => if dir.y < -1e-4 { Some((GROUND - origin.y) / dir.y).filter(|t| *t > 0.0) } else { None },
    };
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
    // (the breakable objects the test map's tools put down: play_scenery.rs)
    if let Some(t) = scenery::ray_placed(origin, dir, max).filter(|&t| best.is_none_or(|b| t < b)) {
        best = Some(t);
    }
    best.filter(|t| *t <= max)
}

/// Headings tried either side of straight behind when the cut to a squad member finds a wall
/// there: this step, up to half a turn (the demo's choice: the game's hand-over puts the camera
/// behind the new character's aim, no search found in its code, and take21 has no wall behind).
const HANDOVER_YAW_STEP: f32 = 10.0 * std::f32::consts::PI / 180.0;

/// The heading for a cut to `p`, facing `facing`: straight behind them if the follow camera's wall
/// test leaves its whole boom there (`camera::room_at`); otherwise the nearest heading either side
/// that does (else the wall's steps would put the camera at the body, see-through); with none, the
/// one with the most room. The demo's choice (see HANDOVER_YAW_STEP).
fn clear_follow_yaw(p: &Player, game: &Game, facing: f32) -> f32 {
    let (room, full) = camera::room_at(p, game, facing);
    if room >= full {
        return facing;
    }
    let mut best = (facing, room);
    for k in 1..=18 {
        for side in [1.0, -1.0] {
            let yaw = wrap_angle(facing + side * k as f32 * HANDOVER_YAW_STEP);
            let (r, full) = camera::room_at(p, game, yaw);
            if r >= full {
                return yaw;
            }
            if r > best.1 {
                best = (yaw, r);
            }
        }
    }
    best.0
}

/// The vertical field of view at a zoom: FUN_0006c720's 2 atan(tan(FOV / 2) / zoom) from FOV_Y,
/// the game's follow camera's (#269): the L-Shot-75's zoom 10 and 25 give 6.01 and 2.41 degrees
/// down, 8.01 and 3.21 across (take15 / take16's zfov).
fn fov_at(zoom: f32) -> f32 {
    2.0 * ((FOV_Y * 0.5).tan() / zoom.max(1.0)).atan()
}

/// FUN_00124840: how many levels `p`'s scope has: none without a gun in hand, with one whose zoom
/// (its data +0x16c, h_efce7f77) isn't above 0, or with a gun in the second hand (Tex's
/// berserk); else its skill's count for 1 (+0x28: one, or two for Flint's skill).
fn scope_levels(p: &Player) -> u8 {
    let zoom = p.loaded.as_ref().and_then(|l| l.weapons.get(p.weapon)).map_or(0.0, |w| w.def.zoom);
    if zoom <= 0.0 || p.second.is_some() {
        return 0;
    }
    skills::scope_levels(p, 1).clamp(0, u8::MAX as i64) as u8
}

/// FUN_00124920: the zoom at scope level `level` of `p`'s gun: 1 at level 0 (and with no gun in
/// hand, or one in the second hand), else the gun's zoom through the skill (+0x2c: the zoom, or
/// for Flint's second level `z + max(10, z + 5)`: the L-Shot-75's 10 then 25, the L-Shot-50's 5
/// then 15).
fn level_zoom(p: &Player, level: u8) -> f32 {
    match p.loaded.as_ref().and_then(|l| l.weapons.get(p.weapon)) {
        Some(w) if level > 0 && p.second.is_none() => skills::scope_zoom(p, w.def.zoom, level as i64),
        _ => 1.0,
    }
}

/// CT_AIM_SNIPE, the scope button (0x122399-0x122479): out of the scope, into its first level if
/// it has levels (`scope_levels`) and the button's lockout (+0x7c8) is out, which entering sets to
/// SCOPE_IN_LOCK; in it below the top level, up one, with no lockout test; at the top, out of it
/// (`leave_scope`) once the lockout is out. The in sound with each level gone up (FUN_00124340)
/// plays in `follow_camera`.
fn press_scope(p: &mut Player) {
    let levels = scope_levels(p);
    if p.scope_level == 0 {
        if levels > 0 && p.scope_lock <= 0.0 {
            p.scope_level = 1;
            p.scope_lock = SCOPE_IN_LOCK;
        }
    } else if p.scope_level < levels {
        p.scope_level += 1;
    } else if p.scope_lock <= 0.0 {
        p.leave_scope();
    }
}

impl Player {
    /// FUN_001249b0: out of the scope (char+0x7a8 to 0) if in it, its button's lockout set to
    /// SCOPE_OUT_LOCK (for a character with a camera: the player's); the out sound plays in
    /// `follow_camera`. A reload (FUN_00120d40) and an item's use (FUN_00121170) call it first;
    /// the demo's switch, death and Tex's berserk too (`follow_camera`).
    fn leave_scope(&mut self) {
        if self.scope_level != 0 {
            self.scope_level = 0;
            self.scope_lock = SCOPE_OUT_LOCK;
        }
    }
}

/// The scope's input this frame: the scope button (`pressed`, the right mouse in play;
/// `press_scope`), then the test hooks', under BF_TEST_GOTO's autopilot too:
/// BF_TEST_SCOPE_PRESS=<s>[,<s>...] presses the button at those times (with its lockouts;
/// BF_SCOPE_LOG=1 prints each); BF_TEST_SCOPE=<level>[,<from s>] holds that level (at most the
/// scope's top; from that time on, from the start by default); without either, an aim a hook
/// holds (`p.aim` coming in: BF_TEST_AIM, BF_TEST_FIRE; never the scope's own) with a gun that
/// has a scope is its first level, as it was before the levels. With a scope, `aim` is then
/// whether the character is in it.
fn scope_input(p: &mut Player, pressed: bool) {
    if pressed {
        press_scope(p);
    }
    let presses: Option<Vec<f32>> = std::env::var("BF_TEST_SCOPE_PRESS").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect());
    if let Some(times) = &presses {
        while times.get(p.test_scope_presses).is_some_and(|&s| p.sim_time >= s) {
            p.test_scope_presses += 1;
            press_scope(p);
            if std::env::var("BF_SCOPE_LOG").is_ok() {
                println!("t={:.2} scope press: level {} (lockout {:.3} s)", p.sim_time, p.scope_level, p.scope_lock);
            }
        }
    }
    let levels = scope_levels(p);
    let held = std::env::var("BF_TEST_SCOPE").ok().and_then(|v| {
        let mut it = v.split(',').map(str::trim);
        let level = it.next().and_then(|x| x.parse::<u8>().ok())?;
        let from = it.next().and_then(|x| x.parse::<f32>().ok()).unwrap_or(0.0);
        Some((level, from))
    });
    if let Some((level, from)) = held {
        if p.sim_time >= from {
            p.scope_level = level.min(levels);
        }
    } else if presses.is_none() && p.aim && levels > 0 && p.scope_level == 0 {
        p.scope_level = 1;
    }
    if levels > 0 {
        p.aim = p.scope_level > 0;
    }
}

/// The crosshair's ray: the follow camera's place and the direction through the crosshair
/// (play_camera.rs `view`, `crosshair_local`: CROSSHAIR_UP of the half-height above the centre,
/// in the middle in the scope). What the reticle, the guns and the throws aim by.
fn aim_ray(p: &Player) -> (Vec3, Vec3) {
    let (eye, turn) = camera::view(p);
    (eye, turn * camera::crosshair_local(p))
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

/// A level's music or ambience bed playing (`setup`, or the test tools' music picker,
/// play_testworld.rs, which stops these to play another bank).
#[derive(Component)]
pub struct LevelMusic;

/// A level's music bank (`Game::load_music`) as the game mixes it, by the sounds' Types in
/// `sounds` (the level's sound bank): (track, WAV, "music" or "ambience", its share of the music
/// volume). A bank holds its music and its ambience bed (sdm_e13: amb_singe_02, then
/// tmp_full-on1a-f): the first of each, the bed under the music at AMBIENCE_VOLUME (names the
/// sound bank doesn't define: an "amb" prefix is ambience). A bank without music (sdm_m07's)
/// plays all its beds, ambiencehellish-2ch its score.
pub fn music_mix(tracks: Vec<(String, Vec<u8>)>, sounds: &bf_viewer::bf::audio::SoundBank) -> Vec<(String, Vec<u8>, &'static str, f32)> {
    let ambient = |n: &str| match sounds.sound_type(bf_viewer::bf::hash::h(n)) {
        Some(t) => t != bf_viewer::bf::audio::MUSIC_TYPE,
        None => n.to_ascii_lowercase().starts_with("amb"),
    };
    let (beds, music): (Vec<_>, Vec<_>) = tracks.into_iter().partition(|(n, _)| ambient(n));
    let bed_count = if music.is_empty() { beds.len() } else { 1 };
    music.into_iter().take(1).map(|(n, w)| (n, w, "music", 1.0))
        .chain(beds.into_iter().take(bed_count).map(|(n, w)| (n, w, "ambience", AMBIENCE_VOLUME))).collect()
}

/// The music's volume, linear: BF_MUSIC_VOLUME, default MUSIC_VOLUME. A level's tracks play at it
/// times their own share (`music_mix`'s, or the sound's volume when a sound-trigger plays it:
/// play_sound_triggers.rs).
pub fn music_volume() -> f32 {
    std::env::var("BF_MUSIC_VOLUME").ok().and_then(|v| v.parse().ok()).unwrap_or(MUSIC_VOLUME)
}

/// Plays a level's track (`music_mix`) looping, at `k` of the music volume (`music_volume`).
pub fn spawn_music(commands: &mut Commands, sources: &mut Assets<AudioSource>, wav: Vec<u8>, label: &'static str, k: f32) -> Entity {
    let source = sources.add(AudioSource { bytes: Arc::from(wav.into_boxed_slice()) });
    commands.spawn((AudioPlayer::new(source), PlaybackSettings { volume: Volume::Linear(music_volume() * k), ..PlaybackSettings::LOOP },
                    Name::new(label), LevelMusic)).id()
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
         mut images: ResMut<Assets<Image>>, mut game: ResMut<GameData>, map: Res<MapLevel>,
         mut sources: ResMut<Assets<AudioSource>>, mode: Res<Mode>, current: Res<CurrentMap>,
         mut level_materials: ResMut<Assets<bf_viewer::level_scene::LevelMaterial>>,
         mut liquids: ResMut<Assets<bf_viewer::level_scene::LiquidMaterial>>,
         mut buffers: ResMut<Assets<bevy::render::storage::ShaderStorageBuffer>>) {
    let mut ale = bf_viewer::ale_fx::AleAssets::new(&mut meshes);
    info!("ALE library: {} effects, {} nodes", game.0.effects.effects.len(), game.0.effects.nodes.len());
    // a new world: the enemies' chatter history starts empty (play_enemies.rs)
    enemies::new_world();
    // the slide dust, ready before anyone slides
    if let Some(&fx) = game.0.effect_types.get(&SLIDE_EFFECT).and_then(|l| l.first()) {
        ale.load(&mut game.0, &mut images, &mut materials, fx);
    }
    if let Some(level) = &map.0 {
        // the map: its terrain, objects and sky, fog, background and light
        let doors = bf_viewer::level_scene::spawn_level(&mut commands, &mut game.0, level, &mut meshes, &mut level_materials, &mut liquids, &mut buffers, &mut images);
        for d in doors.iter().filter(|_| std::env::var("BF_LEVEL_DUMP").is_ok()) {
            // (a leaf on a transform joint or a hinge: how far it ends from its closed place, m)
            println!("door at {:.1}: {:.2} s, leaves slide {:?} m", d.centre, d.duration,
                     d.leaves.iter().map(|l| match l.motion {
                         bf_viewer::level_scene::LeafMotion::Slide { .. } => l.slide(d.duration, d.duration),
                         bf_viewer::level_scene::LeafMotion::Animated(_) => l.moved(d.duration, d.duration).0,
                     }).collect::<Vec<_>>());
        }
        // the doors, closed until their anim-triggers play them (play_triggers.rs)
        let list: Vec<DoorState> = doors.into_iter().map(|door| DoorState { door, t: 0.0, opening: false, rate: 1.0 }).collect();
        // the wall panels (where each one's button is: its archetype's PANEL_BUTTON hardpoint,
        // else 1.4 m up), and the doors their signal reaches unchanged, for the prompt
        let panels: Vec<Panel> = level.buttons.iter().map(|b| {
            let button = game.0.object_meshes.get(&b.kind).and_then(|&a| bf_viewer::bf::weapon::WeaponModel::load(&game.0, a).ok())
                .and_then(|m| m.hardpoints.get(&PANEL_BUTTON).map(|h| h.point)).unwrap_or(Vec3::Y * 1.4);
            // (its front: the button quad's normal, the model's +z)
            let facing = b.transform.transform_vector3(Vec3::Z).normalize_or_zero();
            let reach = level.button_opens(b);
            let opens = list.iter().enumerate().filter(|(_, d)| reach.iter().any(|x| x.0 == d.door.name)).map(|(i, _)| i).collect();
            if std::env::var("BF_DOOR_LOG").is_ok() {
                println!("panel h_{:08x} button at {:.1} facing {:.2} signal {}: opens {:x?}", b.name, b.transform.transform_point3(button), facing, b.signal, reach);
            }
            Panel { name: b.name, signal: b.signal, data: b.data, at: b.transform.transform_point3(button), facing, opens, used: false }
        }).collect();
        commands.insert_resource(Doors { list, panels });
        commands.insert_resource(UsePanel::default());
        // the power-ups' icons (their idle effects)
        let n = if std::env::var("BF_NO_IDLE_FX").is_ok() { 0 } else {
            bf_viewer::ale_fx::spawn_idle_effects(&mut commands, &mut game.0, level, &mut ale, &mut images, &mut materials)
        };
        println!("{n} idle effects (power-up icons)");
        // the level's placed effect objects: its flies, embers, fires, dust (#258,
        // play_placed_fx.rs)
        let placed = placed_fx::spawn(&mut commands, &mut game.0, level, &mut ale, &mut images, &mut materials);
        println!("{} placed effect objects ({} ALE effects; {} of the level's {} are spawn-trigger templates)", placed.0.len(),
                 placed.0.iter().map(|p| p.entities.len()).sum::<usize>(), level.spawn_templates().iter()
                     .filter(|t| level.effect_objects.iter().any(|o| o.name == **t)).count(), level.effect_objects.len());
        commands.insert_resource(placed);
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
        // (no doors: none of a level played before, after the test tools' level switch; nor its
        // placed effects)
        commands.remove_resource::<Doors>();
        commands.insert_resource(UsePanel::default());
        commands.insert_resource(placed_fx::PlacedEffects::default());
    }
    commands.insert_resource(ale);
    // the level's music, looping (BF_MUTE / BF_NO_MUSIC: none; BF_MUSIC_VOLUME, default 0.5);
    // deathmatch plays without music. A level whose sound-triggers play its music bank's tracks
    // (every level with a bank does) has them play it, cue by cue, and its ambience beds, as the
    // player stands outdoors or in (play_sound_triggers.rs, #223)
    let by_triggers = map.0.as_ref().is_some_and(|l| triggers::sounds::plays_music(l, &game.0.sounds));
    if by_triggers {
        println!("music: the level's sound-triggers play it");
    }
    if map.0.is_some() && !by_triggers && !mode.deathmatch && std::env::var("BF_MUTE").is_err() && std::env::var("BF_NO_MUSIC").is_err() {
        let name = current.0.clone();
        match Game::load_music(&data_dir(), &name) {
            Ok(tracks) => {
                for (track, wav, label, k) in music_mix(tracks, &game.0.sounds) {
                    println!("{label}: {track}");
                    spawn_music(&mut commands, &mut sources, wav, label, k);
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
    // the special motions (SpecialClips): the berserk set, for a set that has its idle (Tex's),
    // else the sniper set, for one that has the sniper idle (Flint's: the run and the back run
    // lead to the walks, ms_flint.xml)
    let bz = |m: &str| named(&format!("Sc_w1_bz_{m}"));
    let sn = |m: &str| named(&format!("Sc_w1_sn_{m}"));
    let special = bz("idle").map(|idle| SpecialClips {
        loco: Locomotion { idle: Some(idle), walk: bz("walk"), run: bz("run"), walk_back: bz("back_walk"), run_back: bz("back_run"), ..default() },
        side: ["f_side_walk_l", "f_side_walk_r", "b_side_walk_l", "b_side_walk_r"].map(bz),
        jump_crouch: bz("jump_crouch"),
        jump_launch: bz("jump_launch"),
        jump_fall: bz("jump_fall"),
        jump_land: bz("jump_land"),
        slide: bz("slide_idle"),
        jumps: true,
    }).or_else(|| named("Sc_w1_sniper_idle").map(|idle| SpecialClips {
        loco: Locomotion { idle: Some(idle), walk: sn("walk"), run: sn("walk"), walk_back: sn("back_walk"), run_back: sn("back_walk"), ..default() },
        side: ["f_side_walk_l", "f_side_walk_r", "b_side_walk_l", "b_side_walk_r"].map(sn),
        // (the fall and the slide are the normal ones)
        jump_crouch: None,
        jump_launch: None,
        jump_fall: None,
        jump_land: None,
        slide: None,
        jumps: false,
    }));
    Clips {
        root_stand_in: root_stand_ins(model, &slots, special.as_ref()),
        slots,
        special,
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
        // (the editor's stand-in has no body: play_editor.rs)
        if p.stand_in || p.loaded.as_ref().is_some_and(|l| l.index == p.character && l.kind == p.kind) {
            continue;
        }
        if let Some(old) = p.loaded.take() {
            commands.entity(old.root).despawn();
        }
        spawn_unit(&mut commands, p, &mut game.0, &mut assets);
    }
    // (the controlled character's first load: BF_TEST_FALL starts it in the air; a BF_TEST_LEVEL
    // switch makes a new Player with sim_time 0, so it drops again on each map)
    if player.sim_time == 0.0 && !player.stand_in {
        test_fall(&mut player);
    }
}

/// Load a character and spawn it with its weapons and clips into `p`: its type's model (#128),
/// and its data from the tables keyed by its slot's name where they have it (the squad's: the
/// test map's rack changes `character_weapons`, a squad deathmatch map the skills), else from its
/// type (OTHER's).
fn spawn_unit(commands: &mut Commands, p: &mut Player, game: &mut Game, assets: &mut ModelAssets) {
    let name = CHARACTERS[p.character];
    let ty = game.character_types.get(&p.kind).cloned();
    let label = if p.character == OTHER { game.type_label(p.kind) } else { name.to_string() };
    // (the squad's by type is its model by name: BF_DUMP_TYPES compares the two)
    let model = match ty.as_ref().map_or_else(|| Character::load(game, name, 0), |t| Character::load_type(game, t, 0)) {
        Ok(m) => m,
        Err(e) => {
            error!("{label}: {e}");
            return;
        }
    };
    let mut clips = pick_clips(&model, game);
    // its type's MOTION table resolved to its clips; an enemy's gait clips come from it (#129,
    // play_enemies.rs), the squad keeps the clips by name
    let mut motions = enemies::Motions::default();
    if let (OTHER, Some(t)) = (p.character, ty.as_ref()) {
        motions = enemies::motions(&model, game, t);
        enemies::table_clips(&mut clips, &motions, game, t);
        clips.root_stand_in = root_stand_ins(&model, &clips.slots, clips.special.as_ref());
    }
    info!("{label} clips: {clips:?}");
    // the head the follow camera pivots on (char+0x48c, measured at spawn: play_camera.rs)
    p.head_height = camera::head_height(&model, game, clips.loco.idle);
    if std::env::var("BF_CAMERA_LOG").is_ok() {
        println!("camera: {label}: head height {:.3} m (its idle's root-height {:?}), the model's bounding sphere {:?} (in its frame)",
                 p.head_height, clips.loco.idle.and_then(|c| model.anims.get(c)).and_then(|a| a.root_height), model.bounds);
    }
    let (feet, foot_rest) = find_feet(&model, game, clips.loco.idle);
    let (footstep_type, jump_sound) = game.character_audio.get(name).copied()
        .or(ty.as_ref().map(|t| (t.footstep_type, t.jump_sound))).unwrap_or((-1, 0));
    // what it's made of: its parts' world material (Flint's 24 SYNTHETIC, the rest of the
    // squad's 21 FLESH_HUMAN), FLESH_HUMAN when its collision isn't loaded
    let flesh = ty.as_ref().and_then(|t| game.flesh(t.mesh)).map_or(fx::FLESH, |f| f as i64);
    let root = commands.spawn((Transform::from_translation(p.position), Visibility::default(), Name::new(label.clone()))).id();
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
    // (a character from a placement holds the placement's: an enemy spawned from one,
    // play_enemies.rs, or a squad member started at its spawn, play_triggers.rs `SquadSpawn::dress`)
    let inventory = p.enemy.weapons.clone().or(game.character_weapons.get(name).cloned()).or(ty.as_ref().map(|t| t.weapons.clone())).unwrap_or_default();
    let classes = game.character_weapon_classes.get(name).copied().or(ty.as_ref().map(|t| t.weapon_classes));
    for (slot, w) in inventory.into_iter().enumerate() {
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
        let muzzle = wm.hardpoints.get(&def.muzzle()).copied().unwrap_or(Hardpoint { point: Vec3::ZERO, rot: Quat::IDENTITY });
        let fire_dir = muzzle.rot * Vec3::NEG_Z;
        let flash = commands.spawn((
            Mesh3d(flash_mesh.clone()), MeshMaterial3d(flash_mat.clone()), NotShadowCaster,
            Transform::from_translation(muzzle.point + fire_dir * 0.07).looking_to(fire_dir, Vec3::Y)
                .with_scale(Vec3::new(0.045, 0.045, 0.11)),
            Visibility::Hidden, ChildOf(entity))).id();
        let light = commands.spawn((
            PointLight { intensity: 0.0, range: 6.0, color: Color::srgb(1.0, 0.75, 0.4), ..default() },
            Transform::from_translation(muzzle.point + fire_dir * 0.1), ChildOf(entity))).id();
        let held_at = |hand: u32| model.hardpoints.get(&hand).zip(wm.hardpoints.get(&weapon::TRIGGER_GRIP))
            .map(|(&(bone, hand), grip)| { let (r, t) = hand.mount(grip); (bone, r, t) });
        let in_hand = held_at(weapon::TRIGGER_HAND);
        let stance = stance_for(classes, def.weapon_type, slot);
        weapons.push(HeldWeapon {
            stance,
            in_hand,
            second_hand: held_at(weapon::SECOND_HAND),
            stowed: weapon::STOW.get(slot).and_then(|k| model.hardpoints.get(k)).zip(wm.hardpoints.get(&weapon::HOLSTER))
                .map(|(&(bone, stow), h)| { let (r, t) = stow.mount(h); (bone, r, t) }),
            shell: wm.hardpoints.get(&def.shots.shell_hardpoint).copied(),
            data_light: game.effect_type_defs.get(&def.shots.muzzle_effect).is_some_and(|t| t.light != 0),
            def, entity, spinners, light, flash, muzzle, fire_dir,
        });
    }
    info!("{label} weapons: {:?}", weapons.iter().map(|w| format!("{} (class {}, stance w{}) held by bone {}", w.def.label, w.def.weapon_type,
        w.stance + 1, w.in_hand.map(|(b, ..)| format!("{:08x}", model_bones[b])).unwrap_or("-".into()))).collect::<Vec<_>>());
    for (k, set) in clips.slots.iter().enumerate() {
        info!("{label} slot {} clips: carry {:?} / ready {:?}", k + 1, set.carry, set.rp);
    }
    // the first slot in hand: a placement leaves none in hand (FUN_0014dd50: +0x1cc = 0), and the
    // character's activation (vtable +0xa4, FUN_00112ab0: a spawn copy's, FUN_000dfd90) draws one
    // through FUN_00123fa0, the first slot that has a gun (the second without one). A zone's opening
    // script may switch it after (play_script.rs's code 100: CT_TOGGLE_WEAPON, e34's Tex and Flint,
    // e40's Tex, Hawk and Brutus). Test hook: BF_START_WEAPON=1 starts with the second in hand
    p.weapon = std::env::var("BF_START_WEAPON").ok().and_then(|v| v.parse().ok()).unwrap_or(0).min(weapons.len().saturating_sub(1));
    p.weapon_dirty = true;
    p.holding = true;
    p.switching = None;
    p.second = None;
    p.reloading = None;
    // new weapon objects: their accuracy, tracer counters, burst and casings start afresh
    // (play_shots.rs), as a new gun's do
    p.fire_state = Default::default();
    // a full clip each (the reserve is the squad's: play_ammo.rs)
    p.ammo = weapons.iter().map(|w| [w.def.ammo.max(1), 0]).collect();
    p.regen = vec![ammo::Charge::default(); weapons.len()];
    let switch_clips = [switch_clip(&model, game, &weapons, 0), switch_clip(&model, game, &weapons, 1)];
    let reload_clips = ["Sc_w1_reload", "Sc_w2_reload"].map(|n| overlay_clip(&model, game, n, feet));
    let use_clips = ["Sc_w1_use_item", "Sc_w2_use_item"].map(|n| overlay_clip(&model, game, n, feet));
    let throw_clips = ["Sc_w1_throw_grenade", "Sc_w2_throw_grenade"].map(|n| overlay_clip(&model, game, n, feet));
    let place_clips = ["Sc_w1_place_hi", "Sc_w2_place_hi"].map(|n| overlay_clip(&model, game, n, feet));
    // the skill's toggle overlays, per stance: a prefixed script is Sc_w1_ / Sc_w2_ + script,
    // otherwise the clip's whole name (Tex's bz_start serves both his stances)
    let skill_clips = game.skill_overlays.get(name).cloned().unwrap_or_default().map(|o| [0, 1].map(|stance| {
        o.as_ref().and_then(|(script, prefixed)| {
            let clip = if *prefixed { format!("Sc_w{}_{script}", stance + 1) } else { script.clone() };
            overlay_clip(&model, game, &clip, feet)
        })
    }));
    // the throwing hand: whichever hand moves further between reaching for the grenade and letting go
    let throw_hand = throw_clips[0].as_ref().and_then(|c| {
        let at = |t: f32| model.world(&model.pose(game, c.clip, t, model.default_face, 0.0));
        let (reach, release) = (c.event(EV_REACH).unwrap_or(0.2), c.event(EV_RELEASE).unwrap_or(c.duration * 0.55));
        let (a, b) = (at(reach), at(release));
        [weapon::TRIGGER_HAND, weapon::SUPPORT_HAND].iter().filter_map(|k| model.hardpoints.get(k))
            .map(|&(bone, hp)| (a[bone].transform_point3(hp.point).distance(b[bone].transform_point3(hp.point)), bone, hp.point))
            .max_by(|x, y| x.0.total_cmp(&y.0)).map(|(_, bone, point)| (bone, point))
    });
    info!("{label} reload {:?} / throw {:?} clips, throwing hand {:?}",
          reload_clips.iter().map(|c| c.as_ref().map(|c| c.duration)).collect::<Vec<_>>(),
          throw_clips.iter().map(|c| c.as_ref().map(|c| c.duration)).collect::<Vec<_>>(), throw_hand.map(|h| h.0));
    p.throwing = None;
    // (play_inventory.rs's `stock` fills it again: a respawn starts with the start's grenades)
    p.grenades.clear();
    p.max_health = game.character_hitpoints.get(name).copied().or(ty.as_ref().map(|t| t.hitpoints)).unwrap_or(100.0);
    p.health = p.max_health;
    // stamina, full at spawn (0x12f46e), and the skill, off (play_skills.rs; the squad's only)
    p.skill = skills::Skill::new(game.character_skills.get(name), &game.combat_skills);
    p.max_stamina = p.skill.data.as_ref().map_or(ty.as_ref().map_or(100.0, |t| t.max_stamina.max(1.0)), |d| d.max_stamina.max(1.0));
    p.stamina = p.max_stamina;
    p.leash = game.squad_leash.get(name).copied().or(ty.as_ref().map(|t| t.ai.squad_leash)).unwrap_or(15.0);
    p.scope_sounds = game.character_scope_sounds.get(name).copied().unwrap_or((0, 0));
    p.snipe_sound = game.character_snipe_sound.get(name).and_then(|&id| game.sounds.pcm(id).map(|(r, pcm)| (id, pcm.len() as f32 / r as f32)));
    p.kneel_jitter = 1.5 * p.random(1000) as f32 / 1000.0;
    for (to, s) in switch_clips.iter().enumerate() {
        if let Some(s) = s {
            info!("{label} switch to weapon {}: clip {} ({:.2}s), drop {:.2}s, grab {:.2}s", to + 1, s.clip, s.duration, s.drop, s.grab);
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
    info!("{label}: soles {lift:.3} m below the floor, raised by that; flesh {flesh}");
    // an enemy's personality, stamina refill and shield (play_enemies.rs)
    enemies::spawned(p, ty.as_ref());
    p.name = label;
    // its AI's sensor: the sight distance and field of view of its type and the placement it came
    // from (#197, play_brain.rs `Sensor`)
    p.sensor.set(ty.as_ref(), p.enemy.ai.as_ref());
    // the gun's raise and the ready pose's blend (#140): its motion set's ready times
    let set = ty.as_ref().and_then(|t| game.motion_sets.get(&t.motion_set));
    let (ready_pc, ready_npc) = set.map_or((READY_FALLBACK, READY_FALLBACK), |s| (s.pc_ready, s.npc_ready));
    info!("{}: sight {} m, field of view {} deg; ready times pc {ready_pc:?}, npc {ready_npc:?}", p.name, p.sensor.vision, p.sensor.fov);
    p.loaded = Some(Loaded { index, kind: p.kind, flesh, root, joints, model, clips, aim_chain, arm_chain, feet, lift, weapon_dropped: false, crouch_lift, foot_rest, foot_range, footstep_type, jump_sound,
                                  ready_pc, ready_npc, weapons, switch_clips, reload_clips, use_clips, throw_clips, place_clips, throw_hand, skill_clips, motions });
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut player: ResMut<Player>,
) {
    // under a menu (the game's clock stopped, `paused`; the menu takes the frame's presses,
    // play_objectives.rs) or the item list (its pause: the controller's movement, look and item
    // sections are off while it's up, 0x13417b-0x13418c) the controls go to them: no press reaches
    // the character, it stops firing and aiming; the rest held stays as it was (nothing moves
    // meanwhile) and is read again once the menu has closed. (The item keys are play_itembox.rs's)
    if paused() {
        (player.jump_pressed, player.dodge_pressed, player.switch_pressed, player.reload_pressed) = (false, false, false, false);
        (player.item_use, player.next_surface, player.fire, player.aim) = (false, false, false, false);
        return;
    }
    // a cutscene has the world (its cutscene bit), a script holds the character, or it isn't in
    // the world yet (play_script.rs, #222): no press reaches it, nor the test hooks' input (take03:
    // the stick, A and START pushed through the opening moved nothing); a script's walk moves it
    if script::blocks_input(&player) {
        // (the mouse's capture isn't a control: Escape frees it here too, #256, and a click
        // takes it again by the same rule as in play; a pause menu would open here, #40)
        mouse_capture(&mut windows, &keys, &mouse, player.sim_time);
        script::held_step(&mut player, frame_dt(&time));
        return;
    }
    // 1-4: hand control to that squad member (after their name shows, see squad_control); not
    // while dead: the death camera hands over when it ends (whether the game lets you cut it
    // short isn't recorded: a guess)
    for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
        if keys.just_pressed(*k) && i != player.character && !player.dead {
            player.select = Some((i, SELECT_TIME));
        }
    }
    // (the AI's target is the AI's: whoever is controlled aims at the crosshair)
    player.ai_target = None;
    player.next_surface = keys.just_pressed(KeyCode::KeyM);
    // test hook: BF_TEST_USE=<s> holds use (E) for a second from that time
    player.use_held = std::env::var("BF_TEST_USE").ok().and_then(|v| v.parse::<f32>().ok())
        .is_some_and(|at| player.sim_time >= at && player.sim_time - at < 1.0);
    // test hook: BF_TEST_SELECT=<character>[,<s>] picks that member at 1 s (or s): control
    // passes SELECT_TIME later. Also under BF_TEST_GOTO's autopilot
    if let Some(v) = std::env::var("BF_TEST_SELECT").ok() {
        let mut it = v.split(',').map(|x| x.trim());
        let c = it.next().and_then(|x| x.parse::<usize>().ok());
        let at = it.next().and_then(|x| x.parse::<f32>().ok()).unwrap_or(1.0);
        if let Some(c) = c {
            if player.sim_time > at && player.select.is_none() && c != player.character && !player.test_selected {
                player.select = Some((c, SELECT_TIME));
                player.test_selected = true;
            }
        }
    }
    if autopilot(&mut player) {
        // (the scope's test hooks; the autopilot's aim with a gun that has a scope is its first
        // level)
        scope_input(&mut player, false);
        return;
    }
    let (was_captured, captured) = mouse_capture(&mut windows, &keys, &mouse, player.sim_time);
    let mut mv = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) { mv.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { mv.y -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { mv.x += 1.0; }
    if keys.pressed(KeyCode::KeyA) { mv.x -= 1.0; }
    // (finer in the scope: the turn is divided by the zoom)
    let look = if captured { motion.delta * 0.004 / player.zoom.max(1.0) } else { Vec2::ZERO };
    let sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let walk = keys.pressed(KeyCode::ControlLeft);
    // right mouse: with a gun whose scope has levels it's the scope button (`press_scope`: in,
    // up a level for Flint's skill, out), applied with the aim below; with any other, aim while
    // held
    let zoomable = scope_levels(&player) > 0;
    let scope_pressed = zoomable && was_captured && mouse.just_pressed(MouseButton::Right);
    let aim = !zoomable && mouse.pressed(MouseButton::Right);
    let jump = keys.just_pressed(KeyCode::Space);
    let dodge = keys.just_pressed(KeyCode::KeyC);
    // the click that captures the mouse doesn't fire
    let fire = was_captured && mouse.pressed(MouseButton::Left);
    let switch = keys.just_pressed(KeyCode::KeyQ);
    let reload = keys.just_pressed(KeyCode::KeyR);
    let use_key = keys.pressed(KeyCode::KeyE) || player.use_held;
    // (the item box's keys, Tab, G and B, are play_itembox.rs's: the game's B, LT and Black, which
    // the player controller's item section reads)
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
    // test hook: BF_TEST_AIM=1 holds aim (with a scope, its first level: `scope_input`)
    player.aim = aim || std::env::var("BF_TEST_AIM").is_ok();
    scope_input(&mut player, scope_pressed);
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
    // the mouse turns the aim, which the follow camera's view is (the game's right stick; no
    // zoom wheel: the boom is each character's own, play_camera.rs), the pitch held to +-60 degrees
    // (FUN_0011e7c0)
    player.cam_yaw -= look.x;
    player.cam_pitch = (player.cam_pitch - look.y).clamp(-camera::PITCH_LIMIT, camera::PITCH_LIMIT);
}

/// The mouse's capture, the demo's own (the game has no mouse): a left click captures it (never
/// under BF_BACKGROUND, play_background.rs, whose mouse events are dropped too), Escape frees it.
/// Read in play and while the controls are off (a cutscene, a script's hold, out of the world:
/// #256), not under a menu. Returns whether it was captured before this frame and is now. With
/// BF_SCRIPT_LOG=1 Escape's press is logged (under BF_BACKGROUND, where the mouse is never
/// captured, the only sign the key was read).
fn mouse_capture(windows: &mut Query<&mut Window, With<PrimaryWindow>>, keys: &ButtonInput<KeyCode>,
                 mouse: &ButtonInput<MouseButton>, t: f32) -> (bool, bool) {
    let Ok(mut win) = windows.single_mut() else { return (false, false) };
    let was = win.cursor_options.grab_mode != CursorGrabMode::None;
    if mouse.just_pressed(MouseButton::Left) && !background::on() {
        win.cursor_options.grab_mode = CursorGrabMode::Locked;
        win.cursor_options.visible = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        win.cursor_options.grab_mode = CursorGrabMode::None;
        win.cursor_options.visible = true;
        if std::env::var("BF_SCRIPT_LOG").is_ok() {
            println!("t {t:.2}: Escape: the mouse freed (it was {})", if was { "captured" } else { "free" });
        }
    }
    (was, win.cursor_options.grab_mode != CursorGrabMode::None)
}

/// BF_AUTOPILOT's grenade: the use button held over these sim times (s), a 0.5 s charge.
const AUTOPILOT_THROW: std::ops::Range<f32> = 29.0..29.5;

/// BF_AUTOPILOT=1: scripted input for automated captures. Returns true while it is driving.
fn autopilot(player: &mut Player) -> bool {
    // test hook: BF_TEST_GOTO=x,z,x2,z2 - start at (x, z) and run toward (x2, z2), the camera
    // behind (also starts there; see main); x2,z2 = x,z stands still; a 5th value is the start
    // height (the floor below it). BF_TEST_FIRE=1 (or 2: the second weapon) fires as well.
    if let Some(g) = test_goto() {
        let to = Vec2::new(g[2] - player.position.x, g[3] - player.position.z);
        // (BF_CAMERA_PITCH: another pitch, e.g. to fire down at something; with the turn a scope's
        // re-aim has given the aim, which the mouse would keep: play_camera.rs `Rig::hook_turn`)
        let turn = player.rig.hook_turn;
        player.cam_yaw = (-to.x).atan2(-to.y) + turn.x;
        let pitch: f32 = std::env::var("BF_CAMERA_PITCH").ok().and_then(|v| v.parse().ok()).unwrap_or(-0.2);
        player.cam_pitch = (pitch + turn.y).clamp(-camera::PITCH_LIMIT, camera::PITCH_LIMIT);
        player.aim = std::env::var("BF_TEST_AIM").is_ok();
        // a goal on the start itself: stand there (and look down -z)
        let stay = g[0] == g[2] && g[1] == g[3];
        if stay {
            player.cam_yaw = turn.x;
        }
        // (BF_TEST_GO_AT=<s>: set off at that time instead of after 1 s, as #35 take08's dash at
        // 4.0 s after standing; play_vengar.rs)
        let go_at = std::env::var("BF_TEST_GO_AT").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
        player.move_input = if player.sim_time > go_at && to.length() > 0.8 && !stay { Vec2::new(0.0, 1.0) } else { Vec2::ZERO };
        // BF_TEST_STRAFE=<x>[,<y>]: move that way instead (camera-relative: -1,0 left, 1,0
        // right, 1,-1 back right), from 1 s on; the camera keeps facing the goal
        if let Some(v) = std::env::var("BF_TEST_STRAFE").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<f32>>()) {
            if player.sim_time > 1.0 && !v.is_empty() {
                player.move_input = Vec2::new(v[0], v.get(1).copied().unwrap_or(0.0)).normalize_or_zero();
            }
        }
        // BF_TEST_FIRE=1: aim and fire (after a second); =2 with the second weapon; =hip fires
        // without aiming (no scope on a gun that zooms)
        if let Ok(w) = std::env::var("BF_TEST_FIRE") {
            player.aim = player.sim_time > 0.8 && w != "hip";
            player.fire = player.sim_time > 1.0;
            player.switch_pressed = w == "2" && (0.3..0.4).contains(&player.sim_time);
        }
        // BF_TEST_SPRINT=<s>: sprint (the game's dash) toward the goal from that time on
        player.sprint = std::env::var("BF_TEST_SPRINT").ok().and_then(|v| v.parse::<f32>().ok()).is_some_and(|at| player.sim_time >= at);
        // BF_TEST_WALK=1: walk toward it (Ctrl), as #146's takes' half-stick (motion 47, 2.15 m/s;
        // play_liquids.rs)
        player.walk = std::env::var("BF_TEST_WALK").is_ok();
        // BF_TEST_CROUCH=<s>: crouch from that time on
        if let Some(at) = std::env::var("BF_TEST_CROUCH").ok().and_then(|v| v.parse::<f32>().ok()) {
            player.crouch_wanted = player.sim_time >= at;
        }
        // (#285) BF_TEST_DODGE=<s>[,<s>...]: the dodge pressed at those times (to the side the move
        // goes, or to the right standing: `step_player`); BF_TEST_DIVE=<s>: a dive at that time
        // toward the goal, as from a grenade landing behind (`Player::dive_from`). Each once, on
        // the frame its time falls in (the capture's fixed step, as BF_AUTOPILOT's presses)
        let (t, was) = (player.sim_time, player.sim_time - capture_step());
        let due = |key: &str| std::env::var(key).ok().is_some_and(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).any(|s| was < s && t >= s));
        player.dodge_pressed = due("BF_TEST_DODGE");
        if due("BF_TEST_DIVE") {
            let away = Vec3::new(g[2] - player.position.x, 0.0, g[3] - player.position.z).normalize_or(Vec3::NEG_Z);
            player.dive_from = Some(player.position - away);
        }
        // (BF_TEST_THROW, the grenade button held, and BF_TEST_NEXT_GRENADE, Tab taps, are the item
        // controls' hooks: play_itembox.rs `item_input`)
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
    // (the throw at AUTOPILOT_THROW: the item controls' input, play_itembox.rs)
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

/// A shot's damage roll on a body: its gun's Damage min..max (`Shot::damage`; the shot also
/// carries the gun's damage-type, for the target's factor, play_damage.rs `vulnerability`, and its
/// ammo type, for the hit's effects), 8-10 for a gun without one. Each shot's own gun's: Tex's
/// berserk fires two at once.
fn shot_rolls(shot: &Shot) -> (f32, f32) {
    if shot.damage[1] > 0.0 { (shot.damage[0], shot.damage[1]) } else { (8.0, 10.0) }
}

/// The AI's shots on their way to a body (see `update_player`): `member` is the unit hit, 0 the
/// player, then the squad's order (the test map's NPC tool clears them when it removes one).
#[derive(Resource, Default)]
struct AiHits(Vec<PendingHit>);

/// A shot of an AI character's landing on `u` (`AiHits`, and a bolt's hit, play_bolts.rs): the
/// victim's step and damage-type factor taken as it lands (play_damage.rs), the hurt and its blood
/// where the shot meets them (`h.local` from their feet), the record's damage for the struck
/// material's hit effect (play_fx.rs `flesh_sp`), and a level character hit by another side knows
/// its shooter (play_brain.rs; a friend's hit posts no EVT_DAMAGED_NO_ENEMIES, 0x3e8fc).
fn land_ai_shot(u: &mut Player, h: &PendingHit, game: &Game, difficulty: &damage::Difficulty, now: f32) {
    let at = u.position + h.local;
    let mut hit = h.hit;
    let amount = hit.land(u, game, difficulty, now);
    let bled = u.blood.len();
    hurt(u, game, amount, HURT_CHATTER, Vec3::new(h.dir.x, 0.0, h.dir.z).normalize_or(Vec3::Z) * 4.0 + Vec3::Y * 1.5, at, h.ammo);
    if let Some(b) = u.blood.get_mut(bled) {
        b.record = Some(hit.record());
    }
    if let Some(by) = h.hit.by.filter(|_| h.hit.friendly >= 1.0) {
        brain::attacked(u, by, true);
    }
}

/// A shot of the controlled character's landing on squad member (or NPC) `m`
/// (`Player::pending_hits`, and a bolt's hit, play_bolts.rs): as `land_ai_shot`, but the hurt line
/// is mostly a grunt, now and then "I'm hit" or a word about friendly fire; the test map's instant
/// kill makes one hit enough (a stamina shield's share too); and shot by the player they sidestep
/// one time in three (EVT_DAMAGED_BY_PC -> GOAL_DODGE).
fn land_player_shot(m: &mut Player, h: &PendingHit, game: &Game, difficulty: &damage::Difficulty, now: f32, instant_kill: bool) {
    let tag = match m.random(6) { 0 => FRIENDLY_FIRE_CHATTER, 1 => HIT_CHATTER, _ => HURT_CHATTER };
    let at = m.position + h.local;
    let mut hit = h.hit;
    let amount = hit.land(m, game, difficulty, now);
    let amount = if instant_kill { enemies::lethal(m).max(amount) } else { amount };
    let bled = m.blood.len();
    hurt(m, game, amount, tag, Vec3::new(h.dir.x, 0.0, h.dir.z).normalize_or(Vec3::Z) * 4.0 + Vec3::Y * 1.5, at, h.ammo);
    if let Some(b) = m.blood.get_mut(bled) {
        b.record = Some(hit.record());
    }
    if let Some(by) = h.hit.by.filter(|_| h.hit.friendly >= 1.0) {
        brain::attacked(m, by, true);
    }
    if !m.dead && m.knock_request.is_none() && m.random(3) == 0 {
        m.dodge_request = Some(m.random(2) == 0);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_player(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
                 mut transforms: Query<&mut Transform>, mut heading: Local<Option<f32>>, mut saw: Local<HashMap<Who, bool>>, test: Option<Res<testmap::TestMap>>,
                 kits: Option<Res<grenade::GrenadeKits>>, mut mines: ResMut<sentry::MineTargets>, mut ai_hits: ResMut<AiHits>,
                 difficulty: Res<damage::Difficulty>, reticle: Res<reticle::Reticle>, map: Res<MapLevel>) {
    let dt = frame_dt(&time);
    let kits: &[grenade::GrenadeKit] = kits.as_ref().map_or(&[], |k| &k.0);
    // the controlled character's guns aim at the crosshair's lock (play_reticle.rs, last
    // frame's); the AI's at its own target
    player.lock_aim = reticle.aim;
    // the squad's heading: the leader's facing, smoothed (formation places turn with it)
    let h = heading.get_or_insert(player.yaw);
    *h = wrap_angle(*h + wrap_angle(player.yaw - *h) * (1.0 - (-1.5 * dt).exp()));
    let heading = *h;
    // a player sees for the controlled character, so its sensor knows no one (#197: it starts
    // afresh under the AI when control passes on); its ready times are the pc ones, the rest's
    // the npc ones (#140)
    player.sensor.clear();
    player.controlled = true;
    for m in squad.0.iter_mut() {
        m.controlled = false;
    }
    // take the loaded character out so the step can borrow it and mutate the player freely (one
    // out of the world yet, `Player::absent`, steps too: its step runs the game's clock, and no
    // input reaches it, play_script.rs `blocks_input`)
    if !player.dead && !knocked_down(&player) {
        if let Some(l) = player.loaded.take() {
            step_player(&mut player, &l, &game.0, kits, dt, &mut transforms);
            move_log(&player, &l, dt);
            player.loaded = Some(l);
        }
    }
    // who can see whom (the squad AI and the test map's NPCs fire only at a hostile in sight)
    let sight = sight_list(std::iter::once(&*player).chain(squad.0.iter()));
    let sight_log = std::env::var("BF_SIGHT_LOG").is_ok();
    // (the log's clock: the controlled character's, as the skills' log uses)
    let clock = player.sim_time;
    // the level's patrol points, for the brains' patrols (#227, play_brain.rs)
    let patrols = map.0.as_ref().map_or(&[][..], |l| &l.patrols[..]);
    // a follower's formation place comes after the squad's own members' (play_brain.rs)
    let mut follower = squad.0.iter().filter(|m| m.in_squad()).count();
    // each one's target this frame, for the squad's broadcast (play_brain.rs `broadcast`): the
    // controlled character's the nearest hostile it sees
    let mut held = vec![(0, (!player.dead).then(|| nearest_hostile(&player, 0, &sight)).flatten().map(|(i, _)| sight[i].who))];
    for (slot, m) in squad.0.iter_mut().enumerate() {
        // (the crosshair's lock is the controlled character's alone: one handed over drops it)
        m.lock_aim = None;
        if m.dead || knocked_down(m) {
            continue;
        }
        // one out of the world yet, waiting for its spawn-trigger (#222): no AI and no step, its
        // clock kept with the game's (control may pass to it once it's come)
        if m.absent {
            m.sim_time += dt;
            continue;
        }
        // (its sensor, play_brain.rs: its sight distance, field of view and contacts, #197; a
        // level character's brain keeps its target among them, the rest fire at the nearest seen)
        // (`toward`: with nothing to fire at, where a brainless AI turns to, a contact's last place)
        let (seen, toward) = if m.brain.is_some() { (brain::sense(m, slot + 1, &sight, dt), None) } else { brain::glance(m, slot + 1, &sight, dt, clock) };
        let target = seen.map(|(i, _)| sight[i].chest);
        held.push((slot + 1, if m.brain.is_some() { brain::target(m) } else { seen.map(|(i, _)| sight[i].who) }));
        // BF_SIGHT_LOG: an AI character finding or losing its target
        if sight_log && saw.insert(m.who(), target.is_some()).is_none_or(|was| was != target.is_some()) {
            let reach = m.sensor.reach(ai_range(m));
            let what = match seen {
                Some((i, d)) => format!("sees unit {i} at {d:.1} m (its visibility {:.2}, reach {reach:.1} x it, x1.15 a contact; field of view {} deg)",
                                        sight[i].visibility, m.sensor.fov),
                None => format!("sees no hostile in its gun's reach {:.1} (sight {reach:.1} x their visibility, x1.15 a contact)", ai_range(m)),
            };
            println!("t={clock:.2} {}: {what}", unit_name(m));
        }
        // no target while the character's skill says hold fire: the AI's attack check
        // FUN_00024f80 refuses at 0x2507a while the attacker's skill's +0x108 is true, a stealthed
        // Hawk's (0x13d3f0), before it raises the gun (0x2509c)
        let target = target.filter(|_| !skills::holds_fire(m));
        // (BF_TEST_TARGET's squadmate stands still in the line of fire; play_sentry.rs's test
        // hooks hold theirs where they put them; the test map's NPCs have their own; a cutscene's
        // hold or walk has its own too, play_script.rs; a level character's its brain,
        // play_brain.rs)
        let place = follower;
        if m.brain.as_ref().is_some_and(|b| b.follows()) {
            follower += 1;
        }
        if m.held || !m.script_walk.is_empty() {
            brain::held(m);
            script::held_step(m, dt);
        } else if m.brain.is_some() {
            brain::think(m, &player, place, heading, dt, target, patrols, &game.0, difficulty.n);
        } else if m.npc.is_some() {
            testtools::npc_ai(m, target, toward, dt);
        } else if (slot != 0 || std::env::var("BF_TEST_TARGET").is_err()) && !m.test_hold {
            squad_ai(m, &player, slot, heading, dt, target, toward);
        }
        let Some(l) = m.loaded.take() else { continue };
        step_player(m, &l, &game.0, kits, dt, &mut transforms);
        // (only the player's grenades are launched)
        m.thrown.clear();
        m.loaded = Some(l);
    }
    // the squad tells each other of its targets (FUN_00045090, after the sense step)
    brain::broadcast(&mut player, &mut squad.0, &held, &sight, dt, clock);
    // friendly fire: the player's shots this frame stop at the first teammate in their way
    let (shooter, team) = (player.who(), player.team);
    let now = player.sim_time;
    // the squadmates' shots stop at a Sentry on their line (play_sentry.rs: 1 hitpoint), going
    // off when the shot lands (`Shot::delay`, handed over as the distance at 1 m/s). (A bolt finds
    // what it hits as it flies, play_bolts.rs: here and below its hits aren't looked for)
    for m in squad.0.iter_mut() {
        for shot in m.shots.iter_mut().filter(|s| !s.swept()) {
            if let Some((e, t)) = mines.first_on(shot.origin, shot.dir, shot.dist) {
                shot.dist = t;
                shot.hit = true;
                mines.strike(e, shot.delay(t), 1.0, now);
            }
        }
    }
    let mut shots = std::mem::take(&mut player.shots);
    for shot in shots.iter_mut() {
        // a Sentry on the line stops the shot, unless a teammate is in the way before it
        let mine = if shot.swept() { None } else { mines.first_on(shot.origin, shot.dir, shot.dist) };
        if let Some((_, t)) = mine {
            shot.dist = t;
            shot.hit = true;
        }
        let hit = if shot.swept() { None } else {
            squad.0.iter().enumerate().filter(|(_, m)| !m.dead && !m.absent)
                .filter_map(|(i, m)| ray_body(shot.origin, shot.dir, m, shot.dist).map(|t| (i, t)))
                .min_by(|a, b| a.1.total_cmp(&b.1))
        };
        if let Some((i, t)) = hit {
            shot.dist = t;
            shot.hit = false;
            shot.body = true;
            // it lands when the shot gets there (`Shot::delay`): a roll x the shooter's skill's
            // factor (Flint's doubled and scoped rolls) x the falloff over the range
            // (play_shots.rs) x the hit part struck, x0.7 on the own side (play_damage.rs)
            let (lo, hi) = shot_rolls(shot);
            let factor = skills::roll_factor(&mut player);
            let m = &mut squad.0[i];
            let roll = lo + (hi - lo) * m.random(1000) as f32 / 1000.0;
            let bone = damage::struck_bone(m, &game.0, shot.origin, shot.dir, t);
            let hit = damage::Hit::new(roll, "roll", shot.damage_type).skill(factor).by(shooter, team, m).falloff(shot.falloff_at(t))
                .struck(m, &game.0, bone);
            let local = shot.origin + shot.dir * t - m.position;
            player.pending_hits.push(PendingHit { delay: shot.delay(t), member: i, hit, dir: shot.dir, local, ammo: shot.ammo });
        } else if let Some((e, t)) = mine {
            mines.strike(e, shot.delay(t), 1.0, now);
        }
        // shot at: the others it passes close to may sidestep (see SHOT_AT_RADIUS)
        for (j, m) in squad.0.iter_mut().enumerate() {
            if m.dead || hit.is_some_and(|(i, _)| i == j) {
                continue;
            }
            let chest = m.position + Vec3::Y * (GROUND + m.height + 1.0);
            let t = (chest - shot.origin).dot(shot.dir).clamp(0.0, shot.dist);
            if (shot.origin + shot.dir * t).distance(chest) >= SHOT_AT_RADIUS {
                continue;
            }
            // (a level character shot at by another side knows its shooter if it knows no enemy:
            // EVT_SHOT_AT_NO_ENEMIES, play_brain.rs)
            if m.team != team {
                brain::attacked(m, shooter, false);
            }
            if m.knock.is_some() || m.dodge_request.is_some() {
                continue;
            }
            if m.random(SHOT_AT_DODGE_ODDS) == 0 {
                m.dodge_request = Some(m.random(2) == 0);
                if std::env::var("BF_AI_LOG").is_ok() {
                    println!("{} shot at: dodges", unit_name(m));
                }
            }
        }
    }
    player.shots = shots;
    // the AI's shots (the squad's and the test map's NPCs', only ever fired at a hostile in
    // sight: none in the main game) stop at the first body on their line but the shooter's own,
    // anyone's - a teammate in the way takes it - and land when the shot gets there
    for k in 0..squad.0.len() {
        if squad.0[k].shots.is_empty() {
            continue;
        }
        let mut shots = std::mem::take(&mut squad.0[k].shots);
        for shot in shots.iter_mut() {
            let ((lo, hi), ammo, damage_type) = (shot_rolls(shot), shot.ammo, shot.damage_type);
            let hit = if shot.swept() { None } else {
                std::iter::once(&*player).chain(squad.0.iter()).enumerate().filter(|&(i, u)| i != k + 1 && !u.dead && !u.absent)
                    .filter_map(|(i, u)| ray_body(shot.origin, shot.dir, u, shot.dist).map(|t| (i, t, u.character, u.position)))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
            };
            if let Some((i, t, _, at)) = hit {
                shot.dist = t;
                shot.hit = false;
                shot.body = true;
                // a roll x the shooter's skill's factor x the falloff x the hit part struck, x0.7 on
                // the own side (play_damage.rs)
                let roll = lo + (hi - lo) * squad.0[k].random(1000) as f32 / 1000.0;
                let factor = skills::roll_factor(&mut squad.0[k]);
                let (shooter, team) = (squad.0[k].who(), squad.0[k].team);
                let victim = if i == 0 { &*player } else { &squad.0[i - 1] };
                let bone = damage::struck_bone(victim, &game.0, shot.origin, shot.dir, t);
                let hit = damage::Hit::new(roll, "roll", damage_type).skill(factor).by(shooter, team, victim).falloff(shot.falloff_at(t))
                    .struck(victim, &game.0, bone);
                let local = shot.origin + shot.dir * t - at;
                if std::env::var("BF_COMBAT_LOG").is_ok() {
                    let name = |u: &Player| format!("{}{}", unit_name(u), u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)));
                    println!("{} shoots {} ({t:.1} m)", name(&squad.0[k]), name(victim));
                }
                ai_hits.0.push(PendingHit { delay: shot.delay(t), member: i, hit, dir: shot.dir, local, ammo });
            }
            // a level character of another side the shot passes close to knows its shooter if it
            // knows no enemy, as with the player's shots (EVT_SHOT_AT_NO_ENEMIES, play_brain.rs)
            let (by, by_team, struck) = (squad.0[k].who(), squad.0[k].team, hit.map(|h| h.0));
            for (j, m) in squad.0.iter_mut().enumerate() {
                if j == k || struck == Some(j + 1) || m.dead || m.brain.is_none() || m.team == by_team {
                    continue;
                }
                let chest = m.position + Vec3::Y * (GROUND + m.height + 1.0);
                let t = (chest - shot.origin).dot(shot.dir).clamp(0.0, shot.dist);
                if (shot.origin + shot.dir * t).distance(chest) < SHOT_AT_RADIUS {
                    brain::attacked(m, by, false);
                }
            }
        }
        squad.0[k].shots = shots;
    }
    let mut due = vec![];
    ai_hits.0.retain_mut(|h| {
        h.delay -= dt;
        if h.delay <= 0.0 {
            due.push(*h);
        }
        h.delay > 0.0
    });
    for h in due {
        let u = if h.member == 0 { Some(&mut *player) } else { squad.0.get_mut(h.member - 1) };
        let Some(u) = u else { continue };
        land_ai_shot(u, &h, &game.0, &difficulty, now);
    }
    // shots arriving: the hit lands on the member (blood where the shot meets them)
    let mut due = vec![];
    player.pending_hits.retain_mut(|h| {
        h.delay -= dt;
        if h.delay <= 0.0 {
            due.push(*h);
        }
        h.delay > 0.0
    });
    let instant_kill = test.as_ref().is_some_and(|t| t.instant_kill);
    for h in due {
        let Some(m) = squad.0.get_mut(h.member) else { continue };
        land_player_shot(m, &h, &game.0, &difficulty, now, instant_kill);
    }
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        u.hurt_quiet -= dt;
    }
    // shots hitting a body on the ground push it (anyone's: the dead, and the knocked down; a
    // bolt's on its sweep, play_bolts.rs)
    let shots: Vec<(Vec3, Vec3, f32)> = std::iter::once(&*player).chain(squad.0.iter())
        .flat_map(|u| u.shots.iter().filter(|s| !s.swept()).map(|s| (s.origin, s.dir, s.dist))).collect();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        // (its name for the log, taken before its body is borrowed)
        let name = if std::env::var("BF_RAGDOLL_LOG").is_ok() { unit_name(u) } else { String::new() };
        let body = match (u.ragdoll.as_mut(), u.knock.as_mut()) {
            (Some(r), _) => r,
            (None, Some(k)) => &mut k.ragdoll,
            _ => continue,
        };
        for &(origin, dir, dist) in &shots {
            if let Some(at) = body.shove(origin, dir, dist) {
                // it bleeds where it's hit, as the living do (a bullet's hit: ammo type 1)
                u.blood.push(Bleed::shown(at, dir, 1));
                if !name.is_empty() {
                    println!("shot {name}'s body at {at:.2}");
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
                // (the model stands raised by its sole lift: the ragdoll starts where it's drawn;
                // an enemy's has the game's bodies and joint limits, play_enemies.rs)
                let ragdoll = Ragdoll::new(&l.model, &u.last_world, GROUND - u.height - u.lift_now, push, &enemies::ragdoll_parts(&game.0, u)).placed(u.position + Vec3::Y * (u.height + u.lift_now), u.yaw);
                u.knock = Some(Knock { ragdoll, time: 0.0, lying: None });
                u.thud = false;
                u.action = Action::None;
                u.crouch_wanted = false;
                u.switching = None;
                u.cancel_throw();
            }
        }
        if u.dead {
            if u.ragdoll.is_none() && !u.last_world.is_empty() {
                let push = Quat::from_rotation_y(-u.yaw) * (u.death_push + u.last_velocity);
                // the knocked down die where they lie
                u.ragdoll = Some(match u.knock.take() {
                    Some(k) => k.ragdoll,
                    None => Ragdoll::new(&l.model, &u.last_world, GROUND - u.height - u.lift_now, push, &enemies::ragdoll_parts(&game.0, u)).placed(u.position + Vec3::Y * (u.height + u.lift_now), u.yaw),
                });
            }
            // (its name for the log, taken before its ragdoll is borrowed)
            let name = if std::env::var("BF_RAGDOLL_LOG").is_ok() { unit_name(u) } else { String::new() };
            if let Some(r) = u.ragdoll.as_mut() {
                let local = r.step(&l.model, dt);
                if !name.is_empty() && (u.dead_for * 2.0).fract() < dt * 2.0 {
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
                    println!("ragdoll {name} t {:.1}: pelvis at {:.2}, {} pairs, closest {:.3} m, lowest bone {:.3} m above its floor, {} hinges, worst bend {:.3} m (negative: wrong way)",
                             u.dead_for, r.origin + turn * r.pos[0], r.pairs.len(), closest, below, r.hinge_sides.len(), worst);
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
        for (i, u) in units.iter_mut().enumerate() {
            let air = u.action.airborne();
            // (where they stood: a step into a rock goes back there, below)
            let y0 = u.position.y;
            let feet = u.position.y + GROUND + u.height.max(0.0);
            u.sliding -= dt;
            // a character the AI steers (a follower, an enemy, a cutscene's walk; not one a fall, a
            // slide or a knock is carrying) keeps to ground with a floor under its middle, as before
            // #264 (a steep face is as the edge of the map), and stops at a drop ahead
            // (`ledge_ahead`), where the game's routes keep its AI off a drop (#264: m09_a's
            // followers walked off the start's platform down a 72-degree face, ~30 m into the
            // liquid). The demo's stand-in for the routes; the game's AI has no refusal of its own.
            // The probe leaves alone one on a slide-flagged floor or building up a slide there,
            // which the game's slide should take down it (e40's Shrikes at (-12.9, 56.7))
            let here = a.floor_at(u.position.x, u.position.z, feet + STEP_UP);
            let steered = !(player_first && i == 0) && !air && !u.slide_active && u.knock.is_none();
            let probe = steered && u.slide_amount <= 0.0 && !here.is_some_and(|f| world::Arena::slides(f.2));
            let moved = Vec2::new(u.position.x, u.position.z) - u.prev_xz;
            let off = steered && here.is_none();
            // nor into a liquid that burns it (#1, play_liquids.rs `steps_into_burning`: the
            // game's routes have no world point in one), unless it's in one already
            let burns = steered && moved.length() > 1e-4 && moved.length() < 1.0
                && liquids::steps_into_burning(&a, u.prev_xz, feet, Vec2::new(u.position.x, u.position.z));
            if off || burns || (probe && moved.length() > 1e-4 && moved.length() < 1.0 && ledge_ahead(&a, u.prev_xz, moved.normalize(), feet)) {
                u.position.x = u.prev_xz.x;
                u.position.z = u.prev_xz.y;
                continue;
            }
            // on the ground, followed along the way they came in steps of at most GROUND_STEP, so
            // where they leave it doesn't hang on the frame rate (`ground_walk`)
            // (not one the AI steers: it keeps to the frame's own test, as before #264)
            let feet = if air || u.knock.is_some() { if u.face.is_some() { face_walk(&a, u, feet) } else { feet } }
                else if steered { feet } else { ground_walk(&a, u, feet) };
            // the floor under the feet; else one under the body's middle part (a rim it's half
            // over, a sliver of steep ground beside a floor: FUN_0012b950's ledge grace, LEDGE_GRACE,
            // #264); else terrain too steep to stand on, which they fall onto and slide down
            // (`slide_on_face`, #264); neither: the edge of the map, so back where they were. A
            // rock's steep flank under the body's middle more than a step over that floor is
            // ground too steep to stand on as well (#285, `Arena::flank_below`; not for one the AI
            // steers, which keeps to floors)
            let top = feet + STEP_UP;
            let floor = a.floor_at(u.position.x, u.position.z, top)
                .or_else(|| (!steered).then(|| a.floor_around(u.position.x, u.position.z, LEDGE_GRACE * BODY_RADIUS, feet - GRACE_DROP, top)).flatten());
            let flank = if steered { None } else {
                a.flank_below(u.position.x, u.position.z, top).filter(|f| floor.is_none_or(|g| f.0 > g.0 + STEP_UP))
            };
            let slope = if floor.is_none() { a.slope_below(u.position.x, u.position.z, top) } else { None };
            let Some((g, n, material, steep)) = flank.map(|(g, n)| (g, n, 0, true)).or(floor.map(|(g, n, m)| (g, n, m, false)))
                .or(slope.map(|(g, n)| (g, n, 0, true))) else {
                u.position.x = u.prev_xz.x;
                u.position.z = u.prev_xz.y;
                continue;
            };
            // a step up that would raise the body into a wall (a rock overhanging the slope it's
            // going up) isn't taken: back where they were (#285, `Arena::clear`). The game's sweep
            // moves the body up the slope with its move, and the overhang stops it (FUN_0012cd10,
            // FUN_00153810); here the move is swept level and the feet raised after
            if !steep && !air && u.knock.is_none() && g > feet + 1e-3
                && !a.clear(Vec2::new(u.position.x, u.position.z), BODY_RADIUS, g, g + BODY_HEIGHT, STEP_UP) {
                u.position.x = u.prev_xz.x;
                u.position.z = u.prev_xz.y;
                continue;
            }
            // (on their feet on a floor: off any face, whatever ended the fall)
            if !steep && !air && u.face.take().is_some() && std::env::var("BF_SLIDE_LOG").is_ok() {
                println!("t {:.2}: {} off the face on its feet at {:.2}", u.sim_time, unit_name(u), u.position);
            }
            if steep && (air || u.knock.is_none()) {
                slide_on_face(u, g, n, feet, air);
            } else if steep {
                // getting up on it: kept on it until they move
                u.position.y = g - GROUND;
            } else if air {
                // in the air: keep the height above the world, over whatever floor is below (off
                // a face onto a floor: falling the rest, the free fall's `vy` going on)
                if u.face.take().is_some() && std::env::var("BF_SLIDE_LOG").is_ok() {
                    println!("t {:.2}: {} off the face onto a floor at {:.2}, falling {:.1} m/s", u.sim_time, unit_name(u), u.position, -u.vy);
                }
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
                    println!("{} walks off a ledge at {:.1}: {:.1} m to the floor", unit_name(u), u.position, feet - g);
                }
            } else {
                u.position.y = g - GROUND;
                if u.was_air {
                    // landed: a long drop hurts
                    u.was_air = false;
                    let drop = u.fall_from - g;
                    if std::env::var("BF_SLIDE_LOG").is_ok() {
                        println!("{} lands at {:.1} after a {drop:.1} m drop", unit_name(u), u.position);
                    }
                    let (lo, hi, most) = FALL_HURT;
                    // (into a liquid too: the drop to its bed, FUN_0012b740 has no liquid check;
                    // #152)
                    if drop > lo {
                        // (no clamp: FUN_0012b740)
                        let k = (drop - lo) / (hi - lo);
                        let at = u.position + Vec3::Y * 0.2;
                        // (DTYPE_NORMAL, through the character's handler: the step, play_damage.rs)
                        let amount = damage::Hit::new(most * k * k, "fall", FALL_DAMAGE_TYPE).land(u, &game.0, &difficulty, now);
                        hurt(u, &game.0, amount, HURT_CHATTER, Vec3::Y * 2.0, at, -1);
                    }
                    u.fall_from = g;
                }
                slide(u, &a, g, y0 + GROUND, n, material, dt);
            }
            // a body on its feet that the floor has carried into a rock's solid (#285: a floor
            // under the step that a level move took it onto, where the rock's surface rises above
            // the step within the move; the game's sweep moves the body up the floor and the rock
            // holds it, FUN_00153810) goes back where it stood
            let feet = u.position.y + GROUND + u.height.max(0.0);
            if !u.action.airborne() && u.knock.is_none()
                && a.inside_span(Vec2::new(u.position.x, u.position.z), feet + STEP_UP, feet + BODY_HEIGHT).is_some()
                && a.inside_span(u.prev_xz, y0 + GROUND + STEP_UP, y0 + GROUND + BODY_HEIGHT).is_none() {
                u.position = Vec3::new(u.prev_xz.x, y0, u.prev_xz.y);
            }
            // (in a liquid: what it does to them, their splashes and rings are play_liquids.rs's)
            inside_log(&a, u);
            u.prev_xz = Vec2::new(u.position.x, u.position.z);
        }
    }
    for u in &units {
        if let Some(tr) = u.loaded.as_ref().and_then(|l| transforms.get_mut(l.root).ok()).as_mut() {
            tr.translation = u.position + Vec3::Y * (u.height + u.lift_now);
        }
    }
}

/// The liquids' clock (level_scene::LiquidClock: their scroll and the water's shimmer) on the
/// game's (`frame_dt`): stopped under a menu, as the game's material and texture updates are
/// skipped while it's paused (0x1ea830), and in BF_CAPTURE's fixed steps.
fn run_liquid_clock(time: Res<Time>, mut clock: ResMut<bf_viewer::level_scene::LiquidClock>) {
    let dt = frame_dt(&time);
    if dt > 0.0 {
        clock.0 += dt;
    }
}

/// BF_INSIDE_LOG=1 (#285): each frame a character ends with its body in a rock's solid
/// (`Arena::inside` at the bottom, middle and top of what its wall test spans: STEP_UP over the
/// feet to its head), which body and placed object, and where.
fn inside_log(a: &world::Arena, u: &Player) {
    if std::env::var("BF_INSIDE_LOG").is_err() {
        return;
    }
    let feet = u.position.y + GROUND + u.height.max(0.0);
    let (lo, hi) = (feet + STEP_UP, feet + BODY_HEIGHT);
    let found = [lo, (lo + hi) * 0.5, hi].into_iter().find_map(|y| a.inside(Vec3::new(u.position.x, y, u.position.z)).map(|b| (b, y)));
    if let Some((body, y)) = found {
        println!("t {:.2}: {} inside body {body} (object {:?}) at {:.2}, {:.2} m over its feet", u.sim_time, unit_name(u),
                 a.body_object(body), Vec3::new(u.position.x, feet, u.position.z), y - feet);
    }
}

/// The effects the characters started this frame (`Player::splashes`: their liquids' splashes
/// and rings, play_liquids.rs; a shield's look, play_enemies.rs), each once where it was started.
#[allow(clippy::too_many_arguments)]
fn spawn_splashes(mut commands: Commands, mut game: ResMut<GameData>, ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
                  mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>,
                  mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let Some(mut ale) = ale else { return };
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        for (at, effect, sp) in u.splashes.drain(..) {
            spawn_once(&mut commands, &mut game.0, &mut ale, &mut images, &mut materials, effect, at, sp);
        }
    }
}

/// An ALE effect run once at `at` with user parameter `sp` (removed when it's done); nothing if
/// the library lacks it.
#[allow(clippy::too_many_arguments)]
fn spawn_once(commands: &mut Commands, game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
              materials: &mut Assets<StandardMaterial>, effect: u32, at: Transform, sp: f32) {
    if let Some(fx) = ale.load(game, images, materials, effect) {
        let life = fx.duration();
        let seed = (at.translation.x * 977.0 + at.translation.z * 131.0) as u32;
        commands.spawn((at, Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx, sp, seed), AleExpire(life)));
    }
}

/// A character on terrain too steep to stand on, its surface at `g` under them with normal `n`
/// (#264; `Arena::slope_below`), their feet at `feet`, `air` if they were already in the air. On
/// a face steeper than 65 degrees the game's character is in the fall state, never a slide state
/// (todo/264-steep-slopes/answer.md): no contact steeper than cos 65 counts as ground
/// (FUN_00153410), so stepping onto one starts a fall (FUN_0012b950: the take-off height
/// char+0x6e8 kept, mode 3, MT_JUMP_FALL 21: take08's Tex runs off the tutorial's lava pit rim in
/// MT 21), carrying their speed. Once their feet reach it they're on it (`Player::face`): its
/// surface keeps them (no landing), and `step_player` moves them by the free fall's move with its
/// part into the face taken off (`face_move`). They stay on it while it falls away no faster than
/// that move brought them down (FACE_TOUCH more); faster (a cliff past it), they leave it,
/// falling on. Off it onto a floor they fall the rest and land, the drop counted from where they
/// took off (FUN_0012b740).
fn slide_on_face(u: &mut Player, g: f32, n: Vec3, feet: f32, air: bool) {
    let log = std::env::var("BF_SLIDE_LOG").is_ok();
    if !air {
        u.action = Action::JumpFall;
        u.vy = 0.0;
        u.air_velocity = Vec3::new(u.last_velocity.x, 0.0, u.last_velocity.z);
        u.fall_from = feet;
        if log {
            println!("{} steps onto a steep face at {:.1} ({:.0} deg)", unit_name(u), u.position, n.y.clamp(-1.0, 1.0).acos().to_degrees());
        }
    }
    u.was_air = true;
    u.position.y = g - GROUND;
    let gap = feet - g;
    let reach = if u.face.is_some() { FACE_TOUCH + u.face_drop } else { FACE_TOUCH };
    if gap > reach {
        // above it (falling onto it, or off it where it drops away)
        if log && u.face.is_some() {
            println!("t {:.2}: {} leaves the face at {:.2} ({gap:.2} m over it), falling {:.1} m/s", u.sim_time, unit_name(u), u.position, -u.vy);
        }
        u.face = None;
        u.height = gap;
        return;
    }
    if log && u.face.is_none() {
        println!("t {:.2}: {} on the face at {:.2} ({:.0} deg)", u.sim_time, unit_name(u), u.position, n.y.clamp(-1.0, 1.0).acos().to_degrees());
    }
    u.face = Some(n);
    u.height = 0.0;
}

/// The move `m` (m: this frame's velocity x dt) against a steep face of normal `n`: its part
/// into the face taken off, none along it (the game's sweep, FUN_00150f90 / FUN_00168950 /
/// FUN_00151170, adds impulses along the contact normals only: no friction). The velocity
/// itself isn't changed (FUN_0012cd10 never feeds the sweep's result back into char+0x470), so
/// from rest the speed along a face grows at g sin(theta): 16.4 m/s^2 at 66 degrees.
fn face_move(m: Vec3, n: Vec3) -> Vec3 {
    let into = m.dot(n);
    if into < 0.0 { m - n * into } else { m }
}

/// The ledge grace (FUN_0012b950): before a fall starts the game probes a capsule of 0.8 x the
/// character's radius (1.4 m tall, at the feet, FUN_00153780 at 0x12ba02), and any walkable
/// triangle in it (normal y above cos 65, not slide-flagged) keeps them standing. Here: a floor
/// within this share of BODY_RADIUS round their middle, no more than GRACE_DROP under the feet (`Arena::floor_around`).
const LEDGE_GRACE: f32 = 0.8;
/// How far under the feet (m) a floor round their middle may lie and still hold them in the
/// ledge grace: the game's ground contact is one under feet + 0.1 m (FUN_00153410,
/// 0x153530-0x153540). A grace that took floors a whole step down let a character walk down a
/// bumpy face from patch to patch, further the more often it was tested (#264: e40's s3 route).
const GRACE_DROP: f32 = 0.1;

/// The AI's drop probe, FUN_00115b00's numbers (todo/264-steep-slopes/answer.md): the ground
/// within LEDGE_PROBE_DOWN m below points LEDGE_PROBE m ahead along the way they're going (here
/// a ray down from LEDGE_PROBE_DROP m over their feet, so ground rising ahead, up a ramp, is met
/// from above and counts: from a step over the feet a ray started under a ramp steeper than ~32
/// degrees and met nothing, which held enemies at the foot of e34's ramps); none at the first,
/// or a change of more than
/// LEDGE_PROBE_DROP m at both the second and the third, is a drop. The game computes that verdict
/// and throws it away (its only caller, the feelers FUN_00018c40, reads only the height): its AI
/// has no ledge refusal, and what keeps it off cliffs is its route over the level's world points,
/// which the demo's brains don't walk yet. So the refusal is the demo's stand-in for those routes.
const LEDGE_PROBE: [f32; 3] = [0.4, 0.8, 1.2];
const LEDGE_PROBE_DOWN: f32 = 5.0;
const LEDGE_PROBE_DROP: f32 = 2.75;

/// Whether a character at `at` with its feet at `feet`, going `dir`, is at a drop (see
/// LEDGE_PROBE).
fn ledge_ahead(a: &world::Arena, at: Vec2, dir: Vec2, feet: f32) -> bool {
    let ground = |d: f32| {
        let p = at + dir * d;
        a.ray(Vec3::new(p.x, feet + LEDGE_PROBE_DROP, p.y), Vec3::NEG_Y, LEDGE_PROBE_DROP + LEDGE_PROBE_DOWN).map(|t| feet + LEDGE_PROBE_DROP - t)
    };
    let off = |d: f32| ground(d).is_none_or(|g| (g - feet).abs() > LEDGE_PROBE_DROP);
    ground(LEDGE_PROBE[0]).is_none() || (off(LEDGE_PROBE[1]) && off(LEDGE_PROBE[2]))
}

/// The longest step (m) a grounded character's move this frame is followed along the ground in
/// (`ground_walk`): about a sprint's move at 60 fps. The demo's: the game's physics steps at a
/// fixed rate with substeps (FUN_0012cd10), where the demo steps once a frame.
const GROUND_STEP: f32 = 0.12;

/// A grounded character's move this frame (from `prev_xz` to where it is), followed along the
/// ground in steps of at most GROUND_STEP: at each step the floor within a step of the feet (as
/// the frame's own test, the ledge grace's too) carries them on; the first point where it doesn't (a ledge, a steep
/// face, nothing) is where they stop this frame, so the ground loop starts their fall or slide
/// from there. With one test a frame, a 15 fps frame's 0.5 m sprint stepped over the floor
/// patches of e40's cliff tops that a 60 fps one walked down, and the take-off (and so the fall's
/// damage) hung on the frame rate (#264). A floor that would raise them into a wall (a rock's
/// overhang, `Arena::clear`, #285) stops them at the step before it. Their feet's height where
/// they stop.
fn ground_walk(a: &world::Arena, u: &mut Player, feet: f32) -> f32 {
    let from = u.prev_xz;
    let to = Vec2::new(u.position.x, u.position.z);
    let length = from.distance(to);
    if !(GROUND_STEP..1.0).contains(&length) {
        return feet;
    }
    let steps = (length / GROUND_STEP).ceil() as usize;
    let mut feet = feet;
    for k in 1..steps {
        let p = from.lerp(to, k as f32 / steps as f32);
        let floor = a.floor_at(p.x, p.y, feet + STEP_UP)
            .or_else(|| a.floor_around(p.x, p.y, LEDGE_GRACE * BODY_RADIUS, feet - GRACE_DROP, feet + STEP_UP));
        match floor {
            // (a step up into a wall, under a rock's overhang, isn't taken: they stop before it,
            // #285, `Arena::clear`)
            Some((g, _, _)) if g > feet + 1e-3 && !a.clear(p, BODY_RADIUS, g, g + BODY_HEIGHT, STEP_UP) => {
                let back = from.lerp(to, (k - 1) as f32 / steps as f32);
                u.position.x = back.x;
                u.position.z = back.y;
                break;
            }
            Some((g, _, _)) if feet - g <= STEP_UP => feet = g,
            _ => {
                u.position.x = p.x;
                u.position.z = p.y;
                break;
            }
        }
    }
    u.position.y = feet - GROUND;
    feet
}

/// A move down a steep face this frame (from `prev_xz` to where it is, its feet coming down
/// `face_drop` along it), followed in steps of at most GROUND_STEP as `ground_walk` follows the
/// ground: at the first point where there's a floor at the feet or no face under them (steep
/// terrain or a rock's flank, #285: the face's foot, a cliff past it) they stop this frame, their
/// feet where the move had brought them, so the ground loop lands them or lets them fall on from
/// there. Taken whole, a 15 fps frame's move carried them up to a metre past the foot before the
/// floor was seen, and the landing (and its damage) hung on the frame rate (#264). Their feet's
/// height where they stop.
fn face_walk(a: &world::Arena, u: &mut Player, feet: f32) -> f32 {
    let from = u.prev_xz;
    let to = Vec2::new(u.position.x, u.position.z);
    let length = from.distance(to);
    if !(GROUND_STEP..4.0).contains(&length) {
        return feet;
    }
    let steps = (length / GROUND_STEP).ceil() as usize;
    for k in 1..steps {
        let t = k as f32 / steps as f32;
        let p = from.lerp(to, t);
        let at = feet - u.face_drop * t;
        let floor = a.floor_at(p.x, p.y, at + FACE_TOUCH).is_some_and(|f| at - f.0 <= FACE_TOUCH + u.face_drop / steps as f32);
        // (the face: steep terrain, or a rock's flank, #285)
        let top = at + FACE_TOUCH + u.face_drop;
        if floor || (a.slope_below(p.x, p.y, top).is_none() && a.flank_below(p.x, p.y, top).is_none()) {
            u.position.x = p.x;
            u.position.z = p.y;
            u.position.y = at - GROUND;
            return at;
        }
    }
    feet
}

/// A body on a steep face (`slide_on_face`) whose feet are within this (m) of it, more than its
/// move along the face brought them down, is touching it. The demo's.
const FACE_TOUCH: f32 = 0.02;

/// The game's slide on the ground under `u` (normal `n`, collision `material`), see SLIDE_COS.
/// Moves `u` while sliding (its own movement this frame replaced, or blended back in as the
/// slide fades) and keeps the slide clip on. `g` is the ground where its own move took it, and
/// `start` the feet's height where the slide's move starts (`prev_xz`, where the last frame
/// stood it): its sweep's (#285), as the collision's sweep of the own move is.
fn slide(u: &mut Player, a: &world::Arena, g: f32, start: f32, n: Vec3, material: u8, dt: f32) {
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
    // ground, where the slide only presses down into the floor. The slide's move swept as the
    // character's own is (`Arena::move_body`, #285: not into a rock at the canyon's foot), from
    // the feet where it starts (from the ground the own move reached, down a slope, the slope's
    // twin facing into the ground would have the feet in front of it and push them back up)
    let asked = to - u.prev_xz;
    let to = a.move_body(u.prev_xz, to, BODY_RADIUS, start, start + BODY_HEIGHT, STEP_UP, None);
    let kept = if asked.length() > 1e-5 { ((to - u.prev_xz).length() / asked.length()).min(1.0) } else { 1.0 };
    let floor = a.floor_at(to.x, to.y, g + STEP_UP);
    let across = Vec2::new(mv.x, mv.z).length() * kept;
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
/// The squad's lines are their names' chatter files. An enemy (OTHER) says only its death line
/// (`enemies::death_cry`): the takes' Shrikes said nothing on the hits before the kill.
fn say(u: &mut Player, game: &Game, tag: u32, delay: f32) {
    if u.speaking > 0.0 || u.quote_in.is_some() || u.character == OTHER {
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
    take_hit(u, game, amount, tag, push, at, ammo, false, true);
}

/// A liquid's hit on `u` (play_liquids.rs), at `at` (the record's point: the character's
/// position): as `hurt`, through the same handler (each game frame's share too, as the game's
/// liquid sends every one to it, 0x221780), but it floors nobody (take03 of #146: thirteen entry
/// hits of 30 left Brutus walking) and its blood is the game's decals alone (`Bleed::burn`:
/// FUN_00124a70 runs after any hit on a character, but there's no weapon's impact: no particles,
/// no hit effect or sound). No push (the record's direction is straight down, 0x221780). `entry`:
/// the whole rate as a part goes in, which may draw a hurt line as `hurt`'s hits do; a frame's
/// share draws none (the demo's: take03 has no sound between the foot events while Brutus
/// stands burning, todo/5-liquid-sounds leads §4.1; the game's chatter gate isn't modelled here).
fn burn(u: &mut Player, game: &Game, amount: f32, at: Vec3, entry: bool) {
    take_hit(u, game, amount, HURT_CHATTER, Vec3::ZERO, at, -1, true, entry);
}

/// `hurt` and `burn`'s hit: `liquid` for a liquid's (`burn`), `speaks` whether it may draw a hurt
/// line (`tag`).
#[allow(clippy::too_many_arguments)]
fn take_hit(u: &mut Player, game: &Game, amount: f32, tag: u32, push: Vec3, at: Vec3, ammo: i64, liquid: bool, speaks: bool) {
    // (nor while a cutscene has the world: the damage handler 0x125600 leaves at once while the
    // world's +0xc50 bit 4 is set, play_script.rs; nor one not in the world yet)
    if u.dead || amount <= 0.0 || script::in_cutscene() || u.absent {
        return;
    }
    let dir = push.normalize_or(Vec3::Y);
    // an enemy's stamina shield takes it first (#129, play_enemies.rs: the deathmatch Shrikes');
    // one it takes whole draws no blood (the shield's look instead)
    let amount = enemies::absorb(u, game, amount);
    if amount <= 0.0 {
        u.blood.push(Bleed { at, dir, ammo, a: 0.0, hp_left: u.health, shielded: true, record: None, burn: liquid });
        return;
    }
    u.health = (u.health - amount).max(0.0);
    // (the game's blood decals size by the amount over the type's damage-multiplier, and by the
    // hit points left after it: FUN_00124a70, play_fx.rs)
    let a = amount / damage::damage_multiplier(game, u).max(1e-3);
    u.blood.push(Bleed { at, dir, ammo, a, hp_left: u.health, shielded: false, record: None, burn: liquid });
    if std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("hit {}{} for {amount:.1} -> {:.1} hp at {at:.2} by ammo type {ammo} (knock cooldown {:.2})", unit_name(u),
                 u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)), u.health, u.knock_cooldown);
    }
    if u.health <= 0.0 {
        u.dead = true;
        u.death_push = push;
        u.fire = false;
        u.aim = false;
        u.quote_in = None;
        u.speaking = 0.0;
        u.knock = None;
        // (an enemy's from its type's chatter, started now: play_enemies.rs)
        if enemies::enemy(u) {
            enemies::death_cry(u, game, DEATH_CHATTER);
        } else {
            say(u, game, DEATH_CHATTER, 0.0);
        }
        // the answer comes once the cry is over
        let cry = u.quote_in.map_or(1.5, |(_, id)| game.sounds.pcm(id).map_or(1.5, |(r, s)| s.len() as f32 / r as f32));
        if let Some(&r) = game.chatter_response.get(CHARACTERS[u.character]).and_then(|m| m.get(&DEATH_CHATTER)) {
            if u.random(DEATH_RESPONSE_CHANCE) == 0 {
                u.death_response = Some((cry + 0.4, r));
            }
        }
        return;
    }
    // a hard hit floors them now and then (not right after getting up); an enemy reacts by its
    // personality instead (#129: a stun or a knock-down clip, play_enemies.rs). A liquid's hit
    // does neither (take03 of #146: thirteen entry hits of 30 left Brutus walking; an enemy's
    // reaction to one isn't filmed)
    if !liquid {
        if enemies::enemy(u) {
            enemies::damaged(u, game, amount, dir);
        } else if amount >= KNOCKDOWN_DAMAGE && u.knock.is_none() && u.knock_cooldown <= 0.0 && !u.action.airborne() && u.random(3) != 0 {
            u.knock_request = Some(push);
            if std::env::var("BF_COMBAT_LOG").is_ok() {
                println!("  knocked down: {}", unit_name(u));
            }
        }
    }
    if speaks && u.hurt_quiet <= 0.0 {
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
    /// per bone: whether it has a body of its own (FUN_00127290, #129: an enemy's hit parts and
    /// their ancestors; every bone of the squad's); one without stays as posed at death, rigid
    /// with the nearest bone above it that has one
    bodied: Vec<bool>,
    /// per bone with others rigid with it: the one of them farthest off the line to the child
    /// it points at, and its offset from that line at the start (so the bone also turns about
    /// that line, as the bones it carries went)
    twist: Vec<Option<(usize, Vec3)>>,
    /// hip limits (FUN_00113120, #129's enemies; see RAGDOLL_HIP_FORWARD): thigh, knee, which
    /// way is outward on the hips' right axis (-1 left, 1 right), and the thigh's way in the
    /// bind pose the limits are about (radians forward of straight down, outward)
    swings: Vec<(usize, usize, f32, Vec2)>,
    /// which way the hips' frame's third axis (right x up) points: 1 forward, -1 back (the
    /// body faces -z in its frame at the start)
    hips_forward: f32,
}

impl Ragdoll {
    /// A ragdoll from the pose at death (`world`, the bones' model-space transforms), started
    /// with `push`. `parts`: the bones the game gives bodies to (an enemy's hit parts,
    /// `enemies::ragdoll_parts`, #129), with its knee, elbow and hip limits; empty, every bone
    /// moves, held by the demo's own limits (the squad's).
    fn new(model: &Character, world: &[Mat4], floor: f32, push: Vec3, parts: &[u32]) -> Self {
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
        // instead of held, so they bend freely within reason. Each says whether it's a leg's (the
        // hips' frame, the knee's fold; nearly straight at death, a knee bends forward, an elbow
        // back). A digitigrade leg (Thigh -> Calf -> HorseLink -> Foot: the Shrikes, #129)
        // bends at the knee, (Thigh, Calf, HorseLink); below the calf it's rigid (`bodied`).
        let hinges: Vec<(usize, usize, usize, bool)> = sides.iter().flat_map(|s| {
            let b = |part: &str| bone(&format!("Bip01 {s} {part}"));
            [(b("Thigh"), b("Calf"), b("HorseLink").or(b("Foot")), true), (b("UpperArm"), b("Forearm"), b("Hand"), false)]
        }).filter_map(|t| Some((t.0?, t.1?, t.2?, t.3))).collect();
        let hinge_end = |g: usize, i: usize| hinges.iter().any(|&(a, _, c, _)| a == g && c == i);
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
        // FUN_00127290 (#129): the game's ragdoll gives a body to each hit part and each of
        // their ancestors up to the root; a bone below the lowest hit part of its chain (a
        // Shrike's HorseLink, foot and toes under the calf, a hand under the forearm) has none
        // and stays as posed at death, rigid with the nearest bone above that has one. The demo
        // holds them rigid where that bone points at one of them (its line then carries their
        // turn: the calf at the HorseLink, the forearm at the hand); elsewhere they move as the
        // squad's do.
        let mut bodied = vec![parts.is_empty(); n];
        for i in 0..n {
            if parts.contains(&model.bones[i]) {
                let mut k = Some(i);
                while let Some(j) = k.filter(|&j| j < n && !bodied[j]) {
                    bodied[j] = true;
                    k = model.parent[j];
                }
            }
        }
        if !bodied.contains(&true) {
            bodied = vec![true; n];
        }
        let owner_of = |bodied: &[bool], i: usize| {
            let mut j = i;
            while !bodied[j] {
                match model.parent[j] {
                    Some(p) if p < n => j = p,
                    _ => break,
                }
            }
            j
        };
        let owner: Vec<usize> = (0..n).map(|i| owner_of(&bodied, i)).collect();
        for i in 0..n {
            if !bodied[i] && aim[owner[i]].is_none_or(|(c, _): (usize, Vec3)| owner[c] != owner[i]) {
                bodied[i] = true;
            }
        }
        let owner: Vec<usize> = (0..n).map(|i| owner_of(&bodied, i)).collect();
        // the rigid ones held to their bone, to the child it points at and to the one farthest
        // off that line, at full stiffness (three points fix a fourth). The part may roll about
        // its own line, which the game's knee and elbow don't let it (FUN_00113120 locks their
        // other two axes): the point bodies have no twist to lock it to (a lock tried on the
        // knee's plane fought the floor and kept the body writhing)
        let mut twist = vec![None; n];
        for b in 0..n {
            let rigid: Vec<usize> = (0..n).filter(|&i| i != b && owner[i] == b).collect();
            let Some((c, d0)) = aim[b].filter(|_| !rigid.is_empty()) else { continue };
            let line = d0.normalize();
            let off = |i: usize| { let d = pos[i] - pos[b]; d - line * d.dot(line) };
            twist[b] = rigid.iter().filter(|&&i| i != c).map(|&i| (i, off(i))).filter(|(_, o)| o.length() > 0.03)
                .max_by(|x, y| x.1.length().total_cmp(&y.1.length()));
            let anchors: Vec<usize> = [Some(b), Some(c), twist[b].map(|t| t.0)].into_iter().flatten().collect();
            for (k, &x) in anchors.iter().enumerate() {
                for &y in &anchors[k + 1..] {
                    links.push((x, y, pos[x].distance(pos[y]), 1.0));
                }
            }
            for &i in rigid.iter().filter(|i| !anchors.contains(i)) {
                for &x in &anchors {
                    links.push((x, i, pos[x].distance(pos[i]), 1.0));
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
        for &(a, b, c, leg) in &hinges {
            let f = if leg { 0 } else { 1 };
            let Some(axes) = frames[f].and_then(|fr| torso_axes(&pos, fr)) else { continue };
            let off = hinge_offset(&pos, a, b, c);
            let side = if off.length() > 0.02 {
                off.normalize()
            } else {
                // nearly straight: by the body's forward
                let Some(fw) = forward else { continue };
                let fw = (fw - axes.0 * fw.dot(axes.0)).normalize_or_zero();
                if leg { fw } else { -fw }
            };
            hinge_sides.push((a, b, c, f, Vec3::new(side.dot(axes.0), side.dot(axes.1), side.dot(axes.2))));
        }
        let mut ranges = vec![];
        for &(a, b, c, leg) in &hinges {
            let (upper, lower) = (pos[a].distance(pos[b]), pos[b].distance(pos[c]));
            let straight = upper + lower;
            let near = match (leg, parts.is_empty()) {
                // the game's knee and elbow (FUN_00113120): at most RAGDOLL_KNEE_LIMIT or
                // RAGDOLL_ELBOW_LIMIT from straight (the two bones' ends as far apart as that
                // bend puts them), or as far as it was folded at death (not snapped open)
                (_, false) => {
                    let limit = if leg { RAGDOLL_KNEE_LIMIT } else { RAGDOLL_ELBOW_LIMIT };
                    (upper * upper + lower * lower + 2.0 * upper * lower * limit.cos()).max(0.0).sqrt().min(pos[a].distance(pos[c]))
                }
                (true, true) => straight * RAGDOLL_KNEE_FOLD,
                (false, true) => straight * RAGDOLL_ELBOW_FOLD,
            };
            ranges.push((a, c, near, straight * 0.995));
        }
        // the game's hips (FUN_00113120, #129's enemies): each thigh's swing limited about its
        // way in the bind pose, in the hips' frame (forward: the body's -z, as at the start)
        let hips_forward = frames[0].and_then(|fr| torso_axes(&pos, fr)).map_or(1.0, |(_, _, t)| if t.z <= 0.0 { 1.0 } else { -1.0 });
        let mut swings = vec![];
        if !parts.is_empty() {
            let bind: Vec<Vec3> = model.inverse_bind.iter().take(n).map(|m| m.inverse().w_axis.truncate()).collect();
            if let Some((right, up, third)) = frames[0].filter(|_| bind.len() == n).and_then(|fr| torso_axes(&bind, fr)) {
                let fwd = if third.z <= 0.0 { third } else { -third };
                for (s, out) in [("L", -1.0), ("R", 1.0)] {
                    let (Some(t), Some(c)) = (bone(&format!("Bip01 {s} Thigh")), bone(&format!("Bip01 {s} Calf"))) else { continue };
                    let d = (bind[c] - bind[t]).normalize_or_zero();
                    if d != Vec3::ZERO && bodied[t] && bodied[c] {
                        swings.push((t, c, out, Vec2::new(d.dot(fwd).atan2(-d.dot(up)), (d.dot(right) * out).clamp(-1.0, 1.0).asin())));
                    }
                }
            }
        }
        if std::env::var("BF_RAGDOLL_LOG").is_ok() && !parts.is_empty() {
            let names: Vec<String> = ["Thigh", "Calf", "HorseLink", "Foot", "Toe0", "UpperArm", "Forearm", "Hand"].iter()
                .flat_map(|p| sides.iter().map(move |s| format!("Bip01 {s} {p}"))).chain(["Bip01 Head".to_string()]).collect();
            let name = |i: usize| names.iter().find(|b| bone(b) == Some(i)).cloned().unwrap_or_else(|| format!("h_{:08x}", model.bones[i]));
            let carried = |b: usize| (0..n).filter(|&i| i != b && owner[i] == b).count();
            let rigid: Vec<String> = (0..n).filter(|&b| carried(b) > 0).map(|b| format!("{} +{}", name(b), carried(b))).collect();
            let bends: Vec<String> = hinges.iter().map(|&(a, b, c, _)| {
                let bend = std::f32::consts::PI - (pos[a] - pos[b]).angle_between(pos[c] - pos[b]);
                format!("{} {:.0} deg", name(b), bend.to_degrees())
            }).collect();
            let hip: Vec<String> = swings.iter().map(|&(t, _, _, r)| format!("{} {:.0} fwd {:.0} out", name(t), r.x.to_degrees(), r.y.to_degrees())).collect();
            println!("ragdoll: {} bones, {} with bodies (FUN_00127290); rigid below: {}; bent at death: {}; hips about the bind pose: {}",
                     n, bodied.iter().filter(|&&b| b).count(), rigid.join(", "), bends.join(", "), hip.join(", "));
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
        Ragdoll { pos, prev, rest_rot, rest_offset, aim, links, ranges, frames, hinge_sides, still: 0, floor, origin: Vec3::ZERO, yaw: 0.0, pairs, flesh, rolls: 0,
                  bodied, twist, swings, hips_forward }
    }

    /// Place its frame in the world (the body's root: position + height, turned by yaw), so
    /// its bones meet the level's floors and walls.
    /// A shot (world line from `origin` along `dir`, ending at `dist`) through the body: the
    /// first bone it passes within its thickness (`flesh`) of is brought up to SHOT_SHOVE m/s
    /// along the shot, and bones near it less (falling off to nothing at SHOT_SHOVE_REACH m). It wakes the
    /// body (it settles again, and can roll off its side again). Where it hit (world), if it did.
    fn shove(&mut self, origin: Vec3, dir: Vec3, dist: f32) -> Option<Vec3> {
        let back = Quat::from_rotation_y(self.yaw).inverse();
        let d = back * dir;
        let (along, at) = self.first_bone(origin, dir, dist)?;
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

    /// The first bone a shot (world line from `origin` along `dir`, ending at `dist`) passes within
    /// its thickness (`flesh`, and 5 cm) of, up to 0.2 m past `dist`: how far along it, and the
    /// bone (`shove`; a bolt's sweep meets a body on the ground there, play_bolts.rs).
    fn first_bone(&self, origin: Vec3, dir: Vec3, dist: f32) -> Option<(f32, usize)> {
        let back = Quat::from_rotation_y(self.yaw).inverse();
        let (o, d) = (back * (origin - self.origin), back * dir);
        self.pos.iter().enumerate().filter_map(|(i, p)| {
            let t = (*p - o).dot(d);
            let miss = (o + d * t).distance(*p);
            (t > 0.0 && t < dist + 0.2 && miss < self.flesh[i] + 0.05).then_some((t, i))
        }).min_by(|a, b| a.0.total_cmp(&b.0))
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
            arena.as_ref().and_then(|a| a.floor_below(w.x, w.z, w.y + RAGDOLL_STEP_UP)).map_or(self.floor, |f| f.0 - self.origin.y)
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
            // the hips: each thigh back within its swing (the knee moved, about the hip)
            if let Some((right, up, third)) = self.frames[0].filter(|_| !self.swings.is_empty()).and_then(|fr| torso_axes(&self.pos, fr)) {
                let fwd = third * self.hips_forward;
                for &(t, c, out, rest) in &self.swings {
                    let d = self.pos[c] - self.pos[t];
                    let len = d.length();
                    if len < 1e-4 {
                        continue;
                    }
                    let (ahead, aside) = (d.dot(fwd).atan2(-d.dot(up)), (d.dot(right) * out / len).clamp(-1.0, 1.0).asin());
                    let want = (clamp_turn(ahead, rest.x - RAGDOLL_HIP_BACK, rest.x + RAGDOLL_HIP_FORWARD),
                                aside.clamp(rest.y - RAGDOLL_HIP_SIDE, rest.y + RAGDOLL_HIP_SIDE));
                    if want != (ahead, aside) {
                        let level = want.1.cos();
                        self.pos[c] = self.pos[t] + (right * (out * want.1.sin()) - up * (level * want.0.cos()) + fwd * (level * want.0.sin())) * len;
                    }
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
        if let Some(a) = arena.as_deref() {
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

    /// The bones' local transforms for the current positions. A bone without a body of its own
    /// (`bodied`) keeps its pose at death under its parent.
    fn local(&self, model: &Character) -> Vec<(Quat, Vec3)> {
        let n = self.pos.len();
        let mut rot = vec![Quat::IDENTITY; n];
        for i in 0..n {
            let delta = match self.aim[i].filter(|_| self.bodied[i]) {
                Some((c, d0)) => {
                    let d = self.pos[c] - self.pos[i];
                    if d.length() > 1e-4 {
                        let turn = Quat::from_rotation_arc(d0.normalize(), d.normalize());
                        // and about that line, as the bones rigid with it went
                        match self.twist[i] {
                            Some((m, was)) => {
                                let line = d.normalize();
                                let off = self.pos[m] - self.pos[i];
                                let (was, now) = ((turn * was).normalize_or_zero(), (off - line * off.dot(line)).normalize_or_zero());
                                Quat::from_axis_angle(line, line.dot(was.cross(now)).atan2(was.dot(now))) * turn
                            }
                            None => turn,
                        }
                    } else {
                        Quat::IDENTITY
                    }
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

/// `v` (an angle) held within `lo`..`hi`: outside, the nearer end, going round the circle.
fn clamp_turn(v: f32, lo: f32, hi: f32) -> f32 {
    if (lo..=hi).contains(&v) {
        v
    } else if wrap_angle(v - hi).abs() < wrap_angle(v - lo).abs() {
        hi
    } else {
        lo
    }
}

/// Where a hinge's joint is off the straight line from its upper bone to its end.
fn hinge_offset(pos: &[Vec3], a: usize, b: usize, c: usize) -> Vec3 {
    let line = pos[c] - pos[a];
    let t = if line.length_squared() > 1e-8 { (pos[b] - pos[a]).dot(line) / line.length_squared() } else { 0.0 };
    pos[b] - (pos[a] + line * t)
}

/// Collision: characters are circles (BODY_RADIUS) on the ground: pushed out of the pillars and
/// apart from each other (the player - units[0] when `player_first` - gives way less). On a map
/// one left in a rock is taken out of it first (`out_of_rocks`), then each one's move is swept
/// from where the last frame left it (`prev_xz`) against the walls (`walls`), and so is each
/// push apart after it, so nothing takes a body through a wall or into a rock (#285).
fn collide(units: &mut [&mut Player], player_first: bool) {
    let arena = world::arena();
    if let Some(a) = arena.as_deref() {
        out_of_rocks(a, units);
    }
    // (where each one's sweep starts: last frame's place, then where the last sweep left it)
    let mut from: Vec<Vec2> = units.iter().map(|u| u.prev_xz).collect();
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
        if let Some(a) = arena.as_deref() {
            walls(a, units, &mut from, player_first);
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
    // (the last push apart swept too)
    if let Some(a) = arena.as_deref() {
        walls(a, units, &mut from, player_first);
    }
}

/// Each of `units` moved from `from` (its own) to where it is now, swept against the walls
/// (`Arena::move_body`, #285: the game's sweep, FUN_00153810, stops a move before it passes
/// through a face, so a sprint, a dodge, a dive or a knock can't carry a body into a rock). Not
/// off the steep face they're on or stepping onto, which they slide down (#264, `slide_on_face`;
/// for the player or one in the air: one the AI steers keeps to floors, and the face stays a wall
/// that turns it along the slope). One in the air whose move across would end in a rock's solid
/// (into a rock's slope, which isn't a wall, rising over its feet faster than it falls) stays
/// where it was across and comes down on the slope, where the game's sweep takes it along it; one
/// on its feet is the ground loop's, which may yet stand it on the rock. (The fall's drop isn't
/// swept: one over a rock's steep side is stood on it by the ground loop, `Arena::flank_below`,
/// and one that drops into a rock anyway is taken out at the start of the collision,
/// `out_of_rocks`, before it's drawn.) One out of the world (`Player::absent`) stays where it is.
fn walls(a: &world::Arena, units: &mut [&mut Player], from: &mut [Vec2], player_first: bool) {
    for (i, u) in units.iter_mut().enumerate() {
        if u.absent {
            continue;
        }
        let feet = u.position.y + GROUND + u.height;
        let (x, z) = (u.position.x, u.position.z);
        let steps_on = (player_first && i == 0) || u.action.airborne();
        let face = u.face.or_else(|| steps_on.then(|| a.floor_at(x, z, feet + STEP_UP).is_none().then(|| a.slope_below(x, z, feet + STEP_UP)).flatten()).flatten().map(|s| s.1));
        let mut at = a.move_body(from[i], Vec2::new(x, z), BODY_RADIUS, feet, feet + BODY_HEIGHT, STEP_UP, face);
        if u.action.airborne() && a.inside_span(at, feet + STEP_UP, feet + BODY_HEIGHT).is_some()
            && a.inside_span(from[i], feet + STEP_UP, feet + BODY_HEIGHT).is_none() {
            at = from[i];
        }
        u.position.x = at.x;
        u.position.z = at.y;
        from[i] = at;
    }
}

/// Each of `units` in a rock's solid where the last frame left it (`Arena::out_of_rock`, #285:
/// put there by a spawn or a test teleport, or by this frame's fall, which isn't swept, dropping
/// it past a rock's side it slid off) is taken out to the nearest place clear of it, its feet on the
/// floor there unless it's in the air, and its move this frame is over (`prev_xz` there too: the
/// ground loop follows none). One out of the world (`Player::absent`) stays where it is.
/// BF_INSIDE_LOG logs it.
fn out_of_rocks(a: &world::Arena, units: &mut [&mut Player]) {
    for u in units.iter_mut() {
        if u.absent {
            continue;
        }
        let feet = u.position.y + GROUND + u.height;
        let Some(out) = a.out_of_rock(u.prev_xz, BODY_RADIUS, feet, feet + BODY_HEIGHT, STEP_UP) else { continue };
        if std::env::var("BF_INSIDE_LOG").is_ok() {
            println!("t {:.2}: {} taken out of a rock from {:.2} to {:.2} (feet {feet:.2}, the floor there {:.2?})", u.sim_time, unit_name(u),
                     u.prev_xz, out.at, out.floor);
        }
        u.position.x = out.at.x;
        u.position.z = out.at.y;
        u.prev_xz = out.at;
        if let Some(g) = out.floor.filter(|_| !u.action.airborne()) {
            u.position.y = g - GROUND;
            u.height = 0.0;
        }
    }
}

/// The seed of a shooter's fire loop (play_shots.rs `FireState::run`): the demo's own hash of
/// its character and, for a test tool's or a trigger's NPC, its number (the primes 7919 and
/// 104729 are the demo's). Wrapping, as a hash: a number above ~41,000 overflowed a u32 (#229);
/// below that the seeds are what they were.
fn shot_seed(character: usize, npc: Option<u32>) -> u32 {
    (character as u32).wrapping_mul(7919).wrapping_add(npc.map_or(0, |id| id.wrapping_mul(104_729)))
}

/// The switch to `p`'s other weapon starts (the weapon switch's press, `step_player`; a script's
/// CT_TOGGLE_WEAPON, `toggle_weapon`): its switch clip, or a swap at once without one. Not while
/// busy, nor during a skill's toggle overlay either: one overlay at a time is the demo's choice
/// (the game queues a toggle overlay only into a free overlay slot, char+0x218 or +0x244,
/// 0x12253a-0x122587; whether a switch waits for one isn't traced). Nor while the skill's overlay
/// filter refuses the switch's overlay (Tex's berserk, `skills::overlays_blocked`), nor while the
/// skill says the character isn't free (FUN_0011e130 asking FUN_00117210: Flint's targeting on,
/// `skills::lets_act`). Whether it started.
fn start_switch(p: &mut Player, l: &Loaded) -> bool {
    if !(l.weapons.len() > 1 && p.switching.is_none() && p.reloading.is_none() && p.throwing.is_none() && p.using.is_none()
        && p.skill.toggling.is_none() && !skills::overlays_blocked(p) && skills::lets_act(p)) {
        return false;
    }
    let to = (p.weapon + 1) % l.weapons.len();
    p.fire_state.reset_burst(0);                        // FUN_0022dc00
    // the two sounds are the HUD's: the weapon panel's rows (vtables 0x39f0b8 / 0x39f078, their
    // update 0xcb970) watch the slot in hand (inventory +0x1cc / +0x1d0) of their HUD entry's
    // character (entry+0xe0; every squad member has an entry, and every entry's rows update),
    // and their state setter FUN_000cbec0 plays ff820fda / e1e97460 (0xcbfda) through
    // FUN_000bb220 as a row leaves or takes the hand. FUN_000bb220 plays a squad member's entry's
    // sound only when a player slot controls that member (0x8cf60), so only the controlled
    // character's switch is heard: an AI member's, as a script's toggle of e40's Hawk and
    // Brutus, is silent (the opening plays its own sound-triggers). A cutscene doesn't silence
    // the controlled one's (#282): the entries update every frame of play (0x7a140) and the
    // cutscene's HUD hide is the HUD's alpha (FUN_000b8e30), not the rows' hidden bit, the one
    // FUN_000bb220 reads
    if p.controlled {
        p.sound_queue.push((SWITCH_SOUNDS[0], 0.9));
        p.switch_sound_in = SWITCH_SOUND_GAP;
    }
    p.hud_list = HUD_LIST_TIME;
    if to < 2 && l.switch_clips[to].is_some() {
        p.switching = Some(Switching { to, time: 0.0, dropped: false, grabbed: false });
        p.aim_hold = 0.0;
    } else {
        // no clip: swap at once
        p.weapon = to;
        p.weapon_dirty = true;
    }
    true
}

/// CT_TOGGLE_WEAPON (9) sent to `p` (the command handler 0x122040's case at 0x12258c: out of the
/// scope, then the slot change FUN_0011e130): a script's code 100 (play_script.rs). The switch
/// starts as the press starts it (`start_switch`). Whether it started.
fn toggle_weapon(p: &mut Player) -> bool {
    let Some(l) = p.loaded.take() else { return false };
    p.leave_scope();
    let started = start_switch(p, &l);
    p.loaded = Some(l);
    started
}

fn step_player(p: &mut Player, l: &Loaded, game: &Game, kits: &[grenade::GrenadeKit], dt: f32, transforms: &mut Query<&mut Transform>) {
    p.sim_time += dt;
    if p.next_surface && !game.surfaces.is_empty() {
        p.surface = (p.surface + 1) % game.surfaces.len();
    }
    let surface = game.surfaces.get(p.surface).cloned().unwrap_or_default();
    // an enemy's own motion (#129, play_enemies.rs): a hit reaction or the test hook's drive,
    // played in place of the gait, its own inputs dropped meanwhile
    let scripted = enemies::step(p, l, game, dt);
    // a skill that holds the move stick under the run (Flint's targeting, 0.55: FUN_001195c0
    // through its +0x38) walks, as in the scope: no run, no sprint. Only this frame's gait takes
    // it (the inputs stay as read: the autopilot doesn't set the walk again each frame). Two of
    // the demo's choices: the AI's Flint walks too (the game's cap is on the stick's way,
    // FUN_001195c0), and the sprint (the game's dash) is barred with the pistol as well, where the
    // game's dash command comes from the stick before the cap (0x134f52) and her own motions,
    // which the pistol keeps, have a dash (not filmed)
    let capped = skills::move_capped(p);
    let (sprint, walk) = (p.sprint && !capped, p.walk || capped);

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
    // the special motions while a skill's toggle has taken them (SpecialClips, play_skills.rs:
    // Tex's berserk set, Flint's sniper set): their own jump (Tex's) and slide; no crouch, dash,
    // dodge or dive, which they have no transition to, nor a jump in Flint's
    let special = if p.skill.special_motions { c.special.as_ref() } else { None };
    let (jump_crouch, jump_launch, jump_fall, jump_land, slide_clip) = match special {
        Some(s) => (s.jump_crouch.or(c.jump_crouch), s.jump_launch.or(c.jump_launch), s.jump_fall.or(c.jump_fall),
                    s.jump_land.or(c.jump_land), s.slide.or(c.slide)),
        None => (c.jump_crouch, c.jump_launch, c.jump_fall, c.jump_land, c.slide),
    };
    // a crouch asked for in them is dropped, not kept for after the off: the demo's choice until
    // a take shows what the game does with one (#35's shot T3); kept, he'd drop into a crouch at
    // the off. Switched on kneeling, Flint gets up through her crouch2stand into the sniper idle:
    // also the demo's, where ms_flint.xml's crouch idle has its type 45 transition straight to
    // `Sc_w1_sniper_idle` (with the motion blend; not filmed)
    if special.is_some() {
        p.crouch_wanted = false;
    }
    let crouch_wanted = p.crouch_wanted;
    let dodge_pressed = p.dodge_pressed && special.is_none();

    // ---- actions: dodge and the jump chain (crouch -> launch -> fall -> land) ----
    p.action_left -= dt;
    p.jump_buffer = if p.jump_pressed { 0.4 } else { p.jump_buffer - dt };
    let start_launch = |p: &mut Player| {
        p.action = Action::JumpLaunch;
        p.action_left = dur(if four { c.leg4_launch } else { jump_launch });
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
        Action::Crouched if !crouch_wanted || (moving && stance.cr_walk.is_none()) || p.jump_buffer > 0.0 || dodge_pressed => {
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
        if grounded && matches!(p.action, Action::None | Action::Crouching | Action::Crouched) && stance.dive.is_some() && special.is_none() {
            let away = Vec3::new(p.position.x - from.x, 0.0, p.position.z - from.z).normalize_or(Vec3::Z);
            p.yaw = (-away.x).atan2(-away.z);
            p.action = Action::Dive;
            p.action_left = dur(stance.dive);
            if std::env::var("BF_COMBAT_LOG").is_ok() {
                println!("  dive: {} away from {from:.1}", unit_name(p));
            }
            p.crouch_wanted = false;
            p.still_time = 0.0;
        }
    }
    // (a jump pressed in motions with no transition to one is dropped: Flint's sniper set)
    if special.is_some_and(|s| !s.jumps) {
        p.jump_buffer = 0.0;
    }
    if grounded && matches!(p.action, Action::None) {
        if p.jump_buffer > 0.0 {
            p.jump_buffer = 0.0;
            if moving {
                start_launch(p);                             // running jump: straight into the launch
            } else {
                p.action = Action::JumpCrouch;
                p.action_left = dur(jump_crouch);
                if jump_crouch.is_none() {
                    start_launch(p);
                }
            }
        } else if crouch_wanted && moving && stance.cr_walk.is_some() && stance.crouch_idle.is_some() {
            // crouching on the move: straight into the crouch walk
            p.action = Action::Crouched;
        } else if crouch_wanted && !moving && stance.stand2crouch.is_some() && stance.crouch_idle.is_some() {
            p.action = Action::Crouching;
            p.action_left = dur(stance.stand2crouch);
        } else if dodge_pressed {
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
    if p.switch_pressed {
        start_switch(p, l);
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
    // ---- ammo: a reload (started by play_ammo.rs, which empties the clip at once: the HUD
    // shows 0, in red) plays the stance's reload clip; the clip is full again at its
    // magazine-in event
    if let Some(mut r) = p.reloading.take() {
        r.time += dt;
        let def = &l.weapons[r.weapon].def;
        let clip = l.reload_clips[stance_of(l, r.weapon)].as_ref();
        let duration = clip.map_or(def.reload_time.max(RELOAD_TIME), |c| c.duration);
        let mag_in = clip.and_then(|c| c.event(EV_MAG_IN)).unwrap_or(duration * 0.8);
        if !r.filled && r.time >= mag_in {
            r.filled = true;
            p.ammo[r.weapon] = [r.fill, 0];
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
    let kind = match p.item { Some(Item::Grenade(k)) if p.grenades.get(k).is_some_and(|&n| n > 0) => kits.get(k).map(|kit| (k, kit)), _ => None };
    // (not during a skill's toggle overlay: the demo's choice, as for the weapon switch above;
    // nor in Tex's berserk, whose overlay filter refuses the throw's and whose motions have no
    // throw: #35 take26 20.5 s, LT throws nothing and the count stays; nor while the skill says
    // the character isn't free: the item use FUN_001210c0 asks FUN_00117210 for all but
    // IOU_JUST_USE, 0x121139, so not in Flint's targeting, `skills::lets_act`)
    let can_throw = kind.is_some() && l.throw_hand.is_some() && p.throwing.is_none()
        && p.reloading.is_none() && p.using.is_none() && p.switching.is_none() && p.skill.toggling.is_none()
        && !skills::overlays_blocked(p) && skills::lets_act(p) && matches!(p.action, Action::None) && !p.on_all_fours;
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
            // the item use's skill call (FUN_00121170 -> +0x104, IOU_PLACE_ON_GROUND): Hawk's cost
            skills::used_item(p, kit.def.use_type);
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
        // the meter: its level and a full timer every frame the charge is above 0 (0x134823)
        p.meter_after = (p.charge, METER_TIME);
    } else if p.charge > 0.0 {
        // (and the frame it ends, thrown or not: nothing writes the meter after, so it holds and
        // fades from this level, 0xad950)
        p.meter_after = (p.charge, METER_TIME);
        let slot = stance_of(l, p.weapon);
        if let (true, true, Some((k, kit))) = (can_throw, l.throw_clips[slot].is_some(), kind) {
            p.throwing = Some(Throw { power: p.charge.max(0.1), time: 0.0, slot, grabbed: false, released: false, kind: k,
                                      place: false });
            p.aim_hold = 0.0;
            // the count drops at the button (one game frame after it in the recordings)
            p.grenades[k] -= 1;
            // the item use's skill call (FUN_00121170 -> +0x104, IOU_THROW_TO_USE): Hawk's cost,
            // with the count (#35 take13: 15.1 at 6.19-6.24 s, the count at 6.21)
            skills::used_item(p, kit.def.use_type);
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
    p.aim_hold -= dt;
    // the right mouse's aim (no scope) holds the gun raised, its ready timer at 4 s or more: the
    // demo's stand-in for an aim the game has none of without a shot (the product owner's call,
    // #126), so the follow camera stays in while it's held and eases out the game's way after it
    // (play_camera.rs); scoped, the game's own raise every frame (FUN_0011a440 at 0x11a5ba). A
    // motion that can't aim lowers it every frame (`can_aim`), the aim's hold included
    if p.aim && p.controlled && armed {
        p.fire_state.keep_raised();
    }
    if !can_aim(p) {
        p.fire_state.lower();
    }
    let gun_up = armed && p.reloading.is_none() && p.throwing.is_none()
        && p.switching.is_none() && p.holding && !matches!(p.action, Action::Dodge { .. }) && !p.on_all_fours;
    let clip_of = |p: &Player, w: usize| p.ammo.get(w).map_or(0, |a| a[0]);
    // the second hand's gun (Tex's berserk): the trigger fires it too, each gun while it has a
    // round (take05: the gun in hand empty, the other fires on alone)
    let second = p.second.filter(|s| s.weapon < l.weapons.len());
    let can_fire = [gun_up && clip_of(p, p.weapon) > 0, gun_up && second.is_some_and(|s| clip_of(p, s.weapon) > 0)];
    // (an AI holds a lobbed weapon's fire while its target is within the blast's reach of itself,
    // `missile::lob_holds`: the demo's, untraced)
    let held = p.ai_target.zip(l.weapons.get(p.weapon)).is_some_and(|(t, w)| missile::lob_holds(&w.def, game, p.position, t));
    // (a tap is held for each gun until its first shot, play_shots.rs `pull`)
    let pulled = p.fire_state.pull(p.fire && !held, can_fire);
    if pulled.contains(&true) {
        if p.aim_hold <= 0.0 && !p.fire_state.raised() && !p.aim {
            // raise the gun before the first shot, for the character's own in-time (#140; both
            // guns: take04's first berserk shots went in one frame). The AI aims before it fires:
            // its raise is `ai_fire`'s
            raise(p, ready_times(p, l).0);
        }
        p.aim_hold = 0.7;
    }
    // the fire loop in 30 Hz game frames (play_shots.rs): the cooldown, bursts and pellets,
    // accuracy; each round fired takes one from its clip and plays one report. The fire command
    // sets both guns' fire flags (0x122614-0x122655): each runs its own loop
    if let Some(def) = l.weapons.get(p.weapon).map(|w| &w.def) {
        // (an empty gun's dry cycle: the button down, or a pull still owed the other gun's shot,
        // as the game's request sets both guns' fire flags)
        let dry = ((p.fire && !held) || pulled.contains(&true)) && gun_up && p.using.is_none();
        // (the accuracy object's bonus from the holder: a level character's placement's, #227,
        // play_brain.rs; -1 for the rest)
        let bonus = p.brain.as_ref().map_or(shots::ACCURACY_BONUS, |b| b.gun_bonus);
        let trigger = |hand: usize, clip: i64| shots::Trigger { held: pulled[hand], dry, clip, scoped: p.scope > 0.5, crouched: crouch_wanted, bonus };
        let seed = shot_seed(p.character, p.npc.map(|n| n.id));
        let mut guns = vec![shots::Gun { hand: 0, weapon: p.weapon, def, trigger: trigger(0, clip_of(p, p.weapon)), cooldown: p.cooldown, rounds: 0 }];
        // (its cooldown as the raise above left it)
        if let Some(s) = second.and(p.second) {
            guns.push(shots::Gun { hand: 1, weapon: s.weapon, def: &l.weapons[s.weapon].def, trigger: trigger(1, clip_of(p, s.weapon)),
                                   cooldown: s.cooldown, rounds: 0 });
        }
        p.fire_state.run(&mut guns, seed, dt);
        for g in &guns {
            if g.hand == 0 {
                p.cooldown = g.cooldown;
            } else if let Some(s) = p.second.as_mut() {
                s.cooldown = g.cooldown;
            }
            for _ in 0..g.rounds {
                // one sound per round (each fire sound is a single report); fast guns overlap
                // several (the minigun's ~0.4 s sounds at 15/s), so they play a little quieter
                p.queue_any(&g.def.fire_sounds, if g.def.rate >= 8.0 { 0.55 } else { 0.8 });
                p.shots_fired += 1;
                p.ammo[g.weapon][0] -= 1;
                match p.second.as_mut().filter(|_| g.hand == 1) {
                    Some(s) => (s.recoil, s.flash) = (1.0, 0.05),
                    None => (p.recoil, p.flash) = (1.0, 0.05),
                }
                // the skill's +0xb0 for each round taken (FUN_0022f640 from the fire loop):
                // Hawk's flash and cost while stealthed
                skills::fired(p, g.def);
            }
        }
    }

    // ---- sliding down steep ground: the slide clip, facing downhill ----
    let sliding = p.sliding > 0.0 && matches!(p.action, Action::None) && slide_clip.is_some() && !p.on_all_fours;
    if sliding != p.slide_on && std::env::var("BF_SLIDE_LOG").is_ok() {
        println!("t={:.2} {} slide {} at {:.1}", p.sim_time, unit_name(p), if sliding { "starts" } else { "ends" }, p.position);
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
    // (and while a skill holds the gun raised: FUN_00123ea0 every frame of Flint's targeting, and
    // the ready timer's 4 s after the off, `skills::gun_raised`; the camera comes in with it,
    // play_camera.rs)
    let upper_aim = p.aim || (p.aim_hold > 0.0 && armed) || (armed && skills::gun_raised(p));
    let mut twist_goal = 0.0;
    if sliding {
        let down = p.slide_yaw;
        turn_to(&mut p.yaw, down, TURN_RATE);
        p.gait = Gait::Idle;
    } else if matches!(p.action, Action::None) || (matches!(p.action, Action::Crouched) && stance.cr_walk.is_some()) {
        // (the berserk set's side steps for the aimed ones, SpecialClips)
        let sides = match special {
            Some(s) => &s.side,
            None if matches!(p.action, Action::Crouched) => &stance.cr_side,
            None => &stance.rp_side,
        };
        p.gait = if p.aim {
            if !moving {
                let goal = stand_aim_yaw(p, aim_yaw - off);
                turn_to(&mut p.yaw, goal, TURN_RATE);
                Gait::Idle
            } else if let Some(side) = side_step(wrap_angle(dir_yaw - aim_yaw), sides, armed) {
                // stepping sideways: the side-walk clips twist the upper body about a quarter
                // turn off the root, whose motion runs straight along it (forward steps forward,
                // back steps back, as far as the walks). So the character turns until the gun is
                // on the crosshair (its measured yaw, as standing), and walks where the clip takes
                // it: across the aim, a little forward or back
                turn_to(&mut p.yaw, aim_yaw - off, TURN_RATE);
                side
            } else if wrap_angle(dir_yaw - aim_yaw).abs() > 2.0 {
                turn_to(&mut p.yaw, wrap_angle(dir_yaw + std::f32::consts::PI), TURN_RATE);
                if sprint { Gait::RunBack } else { Gait::WalkBack }
            } else {
                turn_to(&mut p.yaw, dir_yaw, TURN_RATE);
                if sprint { Gait::Run } else { Gait::Walk }
            }
        } else if moving {
            turn_to(&mut p.yaw, dir_yaw, TURN_RATE);
            // (no dash in the berserk set: the dash request finds no transition, the run goes on)
            if sprint && special.is_none() { Gait::Sprint } else if walk { Gait::Walk } else { Gait::Run }
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
    // (the berserk set carried and aimed alike: its script is its ready-script)
    let set = &c.slots[if armed { stance_of(l, p.weapon) } else { 0 }];
    let ready_set = special.is_none() && upper_aim && armed && set.rp.idle.is_some() && set.rp.run.is_some();
    let lo = match special {
        Some(s) => &s.loco,
        None if ready_set => &set.rp,
        None => &set.carry,
    };
    // the gun going up or down: the ready pose blended in over the in-time and out over the
    // out-time (FUN_0011a440: FUN_00150420 with the ready times as the timer leaves 0 and as it
    // reaches it), any other change over FADE
    let fade = match (p.ready_pose, ready_set) {
        (false, true) => ready_times(p, l).0,
        (true, false) => ready_times(p, l).1,
        _ => FADE,
    };
    p.ready_pose = ready_set;
    let sides = special.map_or(&set.rp_side, |s| &s.side);
    let gait_clip = match p.gait {
        Gait::Idle => lo.idle,
        Gait::Walk => lo.walk,
        Gait::Run => lo.run,
        Gait::Sprint => lo.sprint.or(lo.run),
        Gait::WalkBack => lo.walk_back.or(lo.run_back),
        Gait::RunBack => lo.run_back.or(lo.walk_back),
        Gait::Side { left, back } => sides[side_index(left, back)].or(lo.walk),
    }.or(lo.idle);
    let (action_clip, once) = match p.action {
        Action::None => (if sliding { slide_clip } else { None }, false),
        Action::Dodge { left } => (if left { c.loco.dodge_left } else { c.loco.dodge_right }, true),
        Action::JumpCrouch => (jump_crouch, true),
        Action::JumpLaunch => (if four { c.leg4_launch } else { jump_launch }, true),
        Action::JumpFall => (if four { c.leg4_fall } else { jump_fall }, false),
        Action::JumpLand => (if four { c.leg4_land } else { jump_land }, true),
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
    // (an enemy's own motion over both, play_enemies.rs)
    let (action_clip, once) = match &scripted {
        Some(s) => (Some(s.clip), s.once),
        None => (action_clip, once),
    };
    let Some(want) = action_clip.or(gait_clip) else { return };

    // ---- crossfade stack ----
    // (a reaction played again from its start: a second knock-down on the same clip)
    let restart = std::mem::take(&mut p.enemy.restart);
    if p.layers.last().map(|x| x.clip) != Some(want) || restart {
        // loops start in phase with the current clip (keeps feet in step), one-shots from 0
        let phase = p.layers.last().map(|x| x.time / l.model.anims[x.clip].duration.max(1e-3)).unwrap_or(0.0);
        let first = p.layers.is_empty();
        let time = if once { 0.0 } else { phase * l.model.anims[want].duration };
        p.layers.push(Layer { clip: want, time, weight: if first { 1.0 } else { 0.0 }, once, rate: 1.0, fade });
    }
    if let (Some(s), Some(top)) = (&scripted, p.layers.last_mut()) {
        top.rate = s.rate;
    }
    let n = p.layers.len();
    let top = (p.layers[n - 1].weight + dt / p.layers[n - 1].fade.max(1e-3)).min(1.0);
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
        let dt = dt * layer.rate;
        if layer.once {
            let step = (d - layer.time).clamp(0.0, dt);
            delta += locomotion::root_delta(&l.model, game, layer.clip, layer.time, step) * layer.weight;
            layer.time = (layer.time + dt).min(d - 1e-3);
        } else {
            delta += clip_root_delta(&l.model, game, c, layer.clip, layer.time, dt) * layer.weight;
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
    // the skill's toggle overlay (play_skills.rs times it and switches at its event)
    if let Some(t) = &p.skill.toggling {
        if let Some(c) = &l.skill_clips[t.slot()][t.stance] {
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
    // is on, turned over the spine bones that carry the trigger arm. Not over a skill's toggle
    // overlay: the motion sets' `<overlay>` entries all say aim="false" (ms_*.xml; that the
    // game's aim follows the flag isn't traced)
    let held = l.weapons.get(p.weapon).and_then(|w| w.in_hand.map(|h| (w, h)));
    let want_aim = if upper_aim && armed && p.switching.is_none() && p.reloading.is_none() && p.throwing.is_none() && p.using.is_none()
        && p.skill.toggling.is_none() { 1.0 } else { 0.0 };
    p.aim_weight += (want_aim - p.aim_weight) * (1.0 - (-10.0 * dt).exp());
    match held {
        Some((w, (hb, r, t))) if p.aim_weight > 0.01 && !l.arm_chain.is_empty() => {
            let (cam, ray) = aim_ray(p);
            // (the AI aims at its target itself, see `ai_fire`; the controlled character at the
            // crosshair's lock, `lock_aim`, else at what the crosshair is on)
            let mut hit = p.ai_target.or(p.lock_aim).unwrap_or_else(|| cam + ray * ray_hit(cam, ray, 150.0).unwrap_or(80.0));
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
        if let Some(n) = p.face {
            // on a steep face (#264): the free fall goes on (FUN_0012cd10: vy -= 18 dt, x/z x0.99
            // a frame) and its move slides along the face (`face_move`); the ground loop keeps
            // their feet on it (`slide_on_face`). No landing on it
            // (the fall's move over the frame at its mean speed, so it doesn't hang on the frame
            // rate: the game's own steps are fixed)
            let m = face_move(Vec3::new(p.air_velocity.x, p.vy - 0.5 * GRAVITY * dt, p.air_velocity.z) * dt, n);
            p.vy -= GRAVITY * dt;
            p.position += Vec3::new(m.x, 0.0, m.z);
            p.face_drop = (-m.y).max(0.0);
            p.height = 0.0;
        } else {
            p.position += p.air_velocity * dt;
            p.height += (p.vy - 0.5 * GRAVITY * dt) * dt;
            p.vy -= GRAVITY * dt;
        }
        if p.face.is_none() && p.height <= 0.0 && p.vy < 0.0 {
            p.height = 0.0;
            p.vy = 0.0;
            p.action = Action::JumpLand;
            p.action_left = dur(if four { c.leg4_land } else { jump_land });
            // (not with a part in a liquid: the landing's dust and sound are skipped while
            // char+0x104 > 0, 0x12d842)
            if !liquids::in_liquid(p) {
                p.queue_any(&surface.jump_land, 0.9);
            }
            p.step_mute = 0.3;
        }
    } else {
        // (an enemy driven by the test hook, or sliding to a stop after, moves by its type's
        // speeds, not its clip's root: play_enemies.rs)
        let step = enemies::glide(p, scripted.as_ref(), dt).unwrap_or(facing * Vec3::new(delta.x, 0.0, delta.z));
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

    // ---- shots: from the muzzle to whatever the crosshair is on, turned by the accuracy and a
    // burst's spread (play_shots.rs); the second hand's gun's from its own muzzle to the same
    // point (take04: both guns' shots meet at the one crosshair) ----
    if !p.fire_state.pellets.is_empty() {
        let mounts = [l.weapons.get(p.weapon).and_then(|w| w.in_hand.map(|m| (p.weapon, w, m))),
                      p.second.and_then(|s| l.weapons.get(s.weapon).and_then(|w| w.second_hand.map(|m| (s.weapon, w, m))))];
        for (side, mount) in mounts.into_iter().enumerate() {
            let Some((index, w, (hb, r, t))) = mount else { continue };
            let root = Transform::from_translation(p.position + Vec3::Y * p.height).with_rotation(facing).compute_matrix();
            let hand = root * world[hb];
            let barrel = hand.transform_vector3(r * w.fire_dir).normalize();
            let (cam, ray) = aim_ray(p);
            // in the scope the shot leaves from the eye along the crosshair's ray (the hidden
            // muzzle sits ~0.4 m lower: through a 5-10x zoom its line lands visibly low), or to
            // the lock's point when the crosshair holds one
            let scoped = p.scope > 0.5 && side == 0;
            let origin = if scoped { cam + ray * 0.3 } else { hand.transform_point3(r * w.muzzle.point + t) };
            // (the bullet's range in this character's hands, FUN_002324c0: Flint's rifles x1.25)
            let reach = skills::weapon_range(p, &w.def);
            let range = reach.max(20.0);
            // to the AI's target, the crosshair's lock (FUN_0022e5b0: the muzzle to the aim point
            // the reticle gave, char+0x804), or what the crosshair is on. Every gun is steered so:
            // the demo's choice, as FUN_0022e5b0 leaves a weapon whose data +0x170 is set alone
            // (which weapons have it isn't traced)
            let target = p.ai_target.or(p.lock_aim).unwrap_or_else(|| cam + ray * ray_hit(cam, ray, range + 10.0).unwrap_or(range));
            let mut dir = if scoped && p.lock_aim.is_none() { ray } else { (target - origin).normalize_or_zero() };
            if dir.dot(ray) < 0.3 || dir == Vec3::ZERO {
                dir = if upper_aim { ray } else { barrel };
            }
            // bullet-type 4 is an instant ray whatever its speed (FUN_0022f1d0: the Bower's and
            // MK's 80 m/s aren't used; their tracer effect flies on its own); speed 1 marks
            // the others' instant hits; the rest fly at the bullet's speed
            let speed = if w.def.shots.bullet_type == 4 || w.def.bullet_speed < 10.0 { INSTANT_SHOT } else { w.def.bullet_speed };
            let accuracy = p.fire_state.accuracy(index);
            // a mesh projectile (bullet-type 2: the Therm Sweeper and the Sweeper V, steered at a
            // steady speed, and the iKhan-GPL's lobbed round, #239) is a missile of its own
            // (FUN_0022f1d0 case 2 -> FUN_0022fb50, play_missile.rs): it leaves the muzzle along
            // the shot with the aim point and the shooter's line
            let missile = missile::flies(&w.def);
            let look = if p.ai_target.is_some() { (origin, dir) } else { (cam, ray) };
            // an AI's lobbed round is pitched up to come down on its aim point (`missile::lob`:
            // the demo's stand-in for the game's AI aim, untraced); its line stays the straight one
            if p.ai_target.is_some() && missile::lobs(&w.def, game) {
                dir = missile::lob(origin, target, w.def.bullet_speed);
            }
            for (pellet, dir) in p.fire_state.directions(side, dir, w.def.shots.burst_spread) {
                if missile {
                    p.launches.push(missile::Launch { weapon: w.def.name, origin, dir, aim: target, look });
                    if std::env::var("BF_SHOT_LOG").is_ok() {
                        println!("t={:.2} missile launch: origin {:.2} dir {dir:.3} aim {target:.2} {pellet:?} accuracy {accuracy:?}", p.sim_time, origin - p.position);
                    }
                    continue;
                }
                let hit = shots::hit_normal(origin, dir, range);
                p.shots.push(Shot { weapon: w.def.name, origin, dir, dist: hit.map_or(range, |h| h.0), hit: hit.is_some(), speed,
                                    flight: if pellet.tracer { w.def.flight_effect } else { 0 }, hit_fx: w.def.hit_effect,
                                    damage: [w.def.damage_min.min(w.def.damage), w.def.damage], damage_type: w.def.damage_type, ammo: w.def.ammo_type,
                                    decal: w.def.decal, normal: hit.map_or(-dir, |h| h.1), falloff: w.def.shots.falloff,
                                    // (the falloff runs over the bullet's range in the holder's
                                    // hands, FUN_00230040 at 0x2302ef)
                                    range: reach,
                                    // (FUN_0022f1d0: types 0 and 3 to FUN_0022bac0; flying, not
                                    // instant: the speed check above)
                                    bolt: matches!(w.def.shots.bullet_type, 0 | 3) && speed < INSTANT_SHOT, body: false,
                                    // (FUN_00230860: an instant ray takes n1 + n2 hits with the
                                    // pass flag, one without; two or more go through the
                                    // multi-hit ray, play_rays.rs)
                                    pass: speed >= INSTANT_SHOT && w.def.shots.pass_through
                                        && w.def.shots.object_hits + w.def.shots.character_hits >= 2,
                                    scoped });
                // (the pellet's hand: 0 the gun in hand, 1 the second hand's; the shooter last, as
                // the combat log names it)
                if std::env::var("BF_SHOT_LOG").is_ok() {
                    println!("t={:.2} yaw {:.0} aim {:.0} off {:.0} twist {:.0} residual {:.0}  barrel {:.2} ray {:.2} dir {:.3} origin {:.2} dist {:.1} {pellet:?} accuracy {accuracy:?} by {}{}",
                             p.sim_time, p.yaw.to_degrees(), p.cam_yaw.to_degrees(), p.muzzle_off.to_degrees(), p.twist.to_degrees(), p.aim_residual.to_degrees(),
                             barrel, ray, dir, origin - p.position, hit.map_or(range, |h| h.0), unit_name(p), p.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)));
                }
            }
        }
        // (nothing to fire them from: dropped)
        p.fire_state.pellets.clear();
    }

    // ---- footsteps: a foot coming down to its planted height ----
    p.step_mute -= dt;
    let walking = !p.action.airborne() && p.step_mute <= 0.0 && !matches!(p.action, Action::JumpLand | Action::JumpCrouch) && (p.gait != Gait::Idle || matches!(p.action, Action::Dodge { .. }));
    // (x the skill's +0xe8, FUN_00119020 at 0x11918e: a quarter for a stealthed Hawk)
    let volume = skills::noise(p) * match p.gait { Gait::Walk | Gait::WalkBack | Gait::Side { .. } => 0.35, Gait::Sprint => 0.85, _ => 0.6 };
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
            // (in a liquid the wading step or nothing, FUN_00119020's branch: play_liquids.rs;
            // the foot's bone in the world, as drawn)
            let foot = Mat4::from_rotation_translation(Quat::from_rotation_y(p.yaw), p.position + Vec3::Y * (p.height + p.lift_now))
                .transform_point3(world[l.feet[f]].w_axis.truncate());
            match liquids::step(p, foot) {
                liquids::Step::Ground => p.queue_any(&steps, volume),
                liquids::Step::Wade(s) => p.world_sounds.push(s),
                liquids::Step::Silent => {}
            }
        }
        p.foot_prev[f] = y;
    }
}

/// A sound request with play parameters of its own (`Player::world_sounds`, #5): what the game's
/// FUN_00179250 takes for the liquids' sounds (ECX's play-parameter block; todo/5-liquid-sounds
/// leads §2.1), where `Player::sound_queue` holds the demo's plain (id, volume) ones.
#[derive(Clone, Copy, Debug)]
struct WorldSound {
    /// the sound id
    id: u32,
    /// where it plays from (the block's position +0 with its set byte +0xc): heard by its library
    /// entry's distances from the listener, and dropped at or beyond its max distance
    /// (0x17932a-0x179334: 15 m for every liquid sound). None plays it 2D (the controlled
    /// character's own wading step, whose point FUN_00119020 writes into the block's +0x10)
    at: Option<Vec3>,
    /// its volume (+0x48 with its set byte +0x4c), which replaces the library entry's (FUN_0017ec80:
    /// 1.0 for a splash, where the bank has 0.5); None for the entry's own (FUN_0022feb0's ray sets
    /// none)
    volume: Option<f32>,
    /// its pitch (+0x40 with +0x44) in octaves (FUN_0017f580: DirectSound's 1/4096-octave units):
    /// played at 2^pitch the speed
    pitch: f32,
    /// the request's kind, FUN_00179250's third argument (KIND_STEP, KIND_SPLASH, KIND_RAY; for
    /// BF_SOUND_LOG)
    kind: u8,
}

impl WorldSound {
    /// How loud it's heard from `listener` with its library entry `entry` (FUN_00179250, FUN_0017ec80),
    /// and how far it is: a positional one by the entry's distances (`distance_gain`; a 2D entry,
    /// min distance -1, at its volume), None at or beyond the entry's max distance or without an
    /// entry (0x17932a-0x179334); a 2D one at its volume. Its volume is its own when it has one,
    /// else the entry's.
    fn heard(&self, entry: Option<bf_viewer::bf::audio::SoundParams>, listener: Vec3) -> Option<(f32, Option<f32>)> {
        let own = self.volume.unwrap_or(entry.map_or(1.0, |e| e.volume));
        match (self.at, entry) {
            (Some(at), Some(e)) => {
                let d = listener.distance(at);
                let gain = if e.min_distance >= 0.0 { triggers::sounds::distance_gain(d, e.min_distance, e.max_distance) } else { 1.0 };
                (d < e.max_distance).then_some((own * gain, Some(d)))
            }
            (Some(_), None) => None,
            (None, _) => Some((own, None)),
        }
    }
}

/// The kinds of sound request FUN_00179250 is given (its third argument, pushed by its callers): a
/// footstep (FUN_00119020, 0x1192a2), a splash (FUN_00221af0, before 0x221dc2) and an instant ray
/// through a liquid's surface (FUN_0022feb0, 0x22ffb9), kept at the request record's +0x74. What
/// the manager does by a kind isn't traced; here they label the log.
const KIND_STEP: u8 = 4;
const KIND_SPLASH: u8 = 5;
const KIND_RAY: u8 = 7;

/// Decode queued sound ids (cached as in-memory WAVs) and play them: the plain queue
/// (`Player::sound_queue`) as it is, the world's sounds (`WorldSound`) as the game requests them.
fn play_sounds(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               mut cache: ResMut<SoundCache>, mut sources: ResMut<Assets<AudioSource>>, cutscenes: Res<script::Cutscenes>) {
    // squad members' sounds join the player's, quieter with distance (none from one not in the
    // world yet, play_script.rs: its queue is dropped; the player's own queue carries the world's
    // sounds to the listener too, so an absent leader's stays, its body making none: no step)
    let here = player.position;
    let mut world = std::mem::take(&mut player.world_sounds);
    for m in squad.0.iter_mut() {
        if m.absent {
            m.sound_queue.clear();
            m.world_sounds.clear();
            continue;
        }
        let k = (1.0 - m.position.distance(here) / 35.0).clamp(0.15, 1.0) * 0.8;
        let queued: Vec<(u32, f32)> = m.sound_queue.drain(..).map(|(id, v)| (id, v * k)).collect();
        player.sound_queue.extend(queued);
        world.append(&mut m.world_sounds);
    }
    let log = std::env::var("BF_SOUND_LOG").is_ok();
    if log {
        for (id, volume) in &player.sound_queue {
            println!("t={:6.2} sound {id:08x} vol {volume:.2} action {:?} height {:.2}", player.sim_time, player.action, player.height);
        }
    }
    // the world's sounds (FUN_00179250): a positional one is heard from the listener (FUN_00178c10:
    // the followed character's feet, or the cutscene's camera while one is shown, as the
    // sound-triggers hear theirs, play_sound_triggers.rs `listener`) by its library entry's
    // distances (`distance_gain`), and dropped where that's at or beyond the entry's max distance
    // or the library hasn't the id; a 2D one plays at its volume
    let listener = cutscenes.camera_at().unwrap_or(player.position + Vec3::Y * (GROUND + player.height.max(0.0)));
    let mut queue: Vec<(u32, f32, f32)> = player.sound_queue.drain(..).map(|(id, v)| (id, v, 1.0)).collect();
    for s in world {
        let heard = s.heard(game.0.sounds.params(s.id), listener);
        if log {
            let place = match (s.at, heard) {
                (Some(at), Some((_, Some(d)))) => format!("at {at:.2}, {d:.1} m from the listener"),
                (Some(at), _) => format!("at {at:.2}, {:.1} m from the listener: not heard (beyond the entry's max distance, or no entry)", listener.distance(at)),
                (None, _) => "2D".to_string(),
            };
            println!("t={:6.2} sound {:08x} kind {} vol {:.2} (asked {}) pitch {:+.3} oct {place}", player.sim_time, s.id, s.kind,
                     heard.map_or(0.0, |h| h.0), s.volume.map_or("the entry's".to_string(), |v| format!("{v:.2}")), s.pitch);
        }
        if let Some((volume, _)) = heard.filter(|h| h.0 > 0.0) {
            queue.push((s.id, volume, 2f32.powf(s.pitch)));
        }
    }
    if std::env::var("BF_MUTE").is_ok() {
        return;
    }
    for (id, volume, speed) in queue {
        let handle = cache.0.entry(id).or_insert_with(|| {
            game.0.sounds.wav(id).map(|w| sources.add(AudioSource { bytes: Arc::from(w.into_boxed_slice()) }))
        });
        match handle.clone() {
            Some(h) => {
                commands.spawn((AudioPlayer::new(h), PlaybackSettings { volume: Volume::Linear(volume), speed, ..PlaybackSettings::DESPAWN }));
            }
            None => if log {
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

/// The game's clock stopped under a menu: the objectives menu (play_objectives.rs), a tutorial
/// panel (play_panels.rs) or the item list (play_itembox.rs). The game's app clock [[0x469b54]+0x38] stops while one is open
/// (take04's watch log: clk 60275 from the press at 62.6 s to the B at 74.5 s). `frame_dt` is 0
/// meanwhile, so nothing in the world moves (the characters, the triggers' timers, the effects'
/// clock), and `read_input` takes no input. Each menu holds a bit of its own (PAUSE_OBJECTIVES,
/// PAUSE_PANELS): the clock runs again once none holds it, so one closing doesn't restart it
/// under the other.
static PAUSED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// The objectives menu's hold on the clock (see PAUSED).
const PAUSE_OBJECTIVES: u32 = 1;
/// The tutorial panels' hold on the clock (see PAUSED).
const PAUSE_PANELS: u32 = 2;
/// The item list's hold on the clock (play_itembox.rs): FUN_000ac080 pauses the game (the
/// session's pause through the app, 0x7a320) while the list is open in single player; #196 take02's
/// app clock still from the open to the release, take03's grenade hanging in the air.
const PAUSE_ITEMS: u32 = 4;

/// Stops (`on`) or lets go of the game's clock for one of its holders (a PAUSE_ bit; see PAUSED).
fn set_paused(by: u32, on: bool) {
    if on {
        PAUSED.fetch_or(by, std::sync::atomic::Ordering::Relaxed);
    } else {
        PAUSED.fetch_and(!by, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Whether the game's clock is stopped (see PAUSED).
fn paused() -> bool {
    PAUSED.load(std::sync::atomic::Ordering::Relaxed) != 0
}

/// Whether a holder other than `by` (a PAUSE_ bit) has the game's clock stopped.
fn paused_by_other(by: u32) -> bool {
    PAUSED.load(std::sync::atomic::Ordering::Relaxed) & !by != 0
}

fn frame_dt(time: &Time) -> f32 {
    if paused() {
        return 0.0;
    }
    if std::env::var("BF_CAPTURE").is_ok() {
        if WARMUP.load(std::sync::atomic::Ordering::Relaxed) > 0 { 0.0 } else { capture_step() }
    } else {
        time.delta_secs().min(0.1)
    }
}

fn follow_camera(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
                 mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>, mut vis: Query<&mut Visibility>) {
    let dt = frame_dt(&time);
    // the scope: none in Tex's berserk either, from the accepted press to the second gun's
    // stowing (his toggle only refuses a press made scoped, 0x13d490): the demo's choice until a
    // take shows whether the game lets him (#35's shot T2). Scoped, the second gun's muzzle effect
    // washes the view, and a shot in the start clip leaves from behind his shoulder. (Flint scopes
    // in her sniper motions: take15 14.5 s.) Out of it while busy (FUN_001249b0: a reload, the
    // switch, death) and once the gun in hand has fewer levels (none: no gun that zooms)
    let berserk = skills::bars_scope(&player) || player.second.is_some();
    let busy = player.switching.is_some() || player.reloading.is_some() || player.dead || berserk;
    if busy || player.scope_level > scope_levels(&player) {
        player.leave_scope();
    }
    let level = player.scope_level;
    // the ramp (FUN_0011aa20 at 0x11acc6-0x11ae79), SCOPE_CALLS steps a frame: in the scope the
    // progress (+0x7cc) gains dt / the in time and the zoom (+0x7ac) climbs at (the level's zoom
    // - the level below's) / the in time (FUN_0011ae80) to the level's, never past it; out of it
    // the progress loses dt / the out time and the zoom falls at the top level's / the out time
    // (FUN_0011af10: the skill's count, +0x28 with 1, through FUN_00124920) to 1; a time of 0
    // snaps. Linear in the zoom (the takes: the zoom's steps a game frame even, the last one cut
    // short), so the view's angle narrows fastest at first
    let step = dt * SCOPE_CALLS;
    let times = game.0.scope;
    let was = (player.zoom, player.scope);
    if level > 0 {
        let (z, below) = (level_zoom(&player, level), level_zoom(&player, level - 1));
        if times.zoom_in > 0.0 {
            player.scope = (player.scope + step / times.zoom_in).min(1.0);
            if z > player.zoom {
                player.zoom += ((z - below) / times.zoom_in * step).min(z - player.zoom);
            }
        } else {
            (player.scope, player.zoom) = (1.0, z);
        }
    } else {
        // (the game's FUN_0011af10 never meets a gun without a scope in hand here: its swap comes
        // at the switch clip's event, after the scope was left and its zoom fell, and with that
        // gun's zoom of 0 the fall would stop. The demo's gun can change at once (its test hooks,
        // a squad hand-back) with the zoom still up: then the zoom falls at its own rate. A gun in
        // the second hand or none in hand keeps the game's top of 1, FUN_00124920's)
        let top = level_zoom(&player, skills::scope_levels(&player, 1).clamp(0, u8::MAX as i64) as u8);
        let top = if top > 0.0 { top } else { player.zoom };
        if times.zoom_out > 0.0 {
            player.scope = (player.scope - step / times.zoom_out).max(0.0);
            if player.zoom > 1.0 {
                player.zoom -= (top / times.zoom_out * step).min(player.zoom - 1.0);
            }
        } else {
            (player.scope, player.zoom) = (0.0, 1.0);
        }
    }
    // the scope button's lockout (+0x7c8) runs down in the same calls (0x11ae42-0x11ae70)
    player.scope_lock = (player.scope_lock - step).max(0.0);
    // BF_SCOPE_LOG=1: the ramp's zoom and progress on each frame they move (and each level change
    // and hook press, below and in `scope_input`)
    if std::env::var("BF_SCOPE_LOG").is_ok() && (player.zoom, player.scope) != was {
        println!("t={:.3} scope ramp: zoom {:.3}, progress {:.3}, level {level}", player.sim_time, player.zoom, player.scope);
    }
    // the aim's wander in the scope (none for Flint)
    let t = player.sim_time;
    let wander = Vec2::new((0.9 * t).sin() + 0.45 * (2.3 * t + 0.7).sin(), 0.8 * (1.3 * t + 1.1).sin() + 0.35 * (2.9 * t).sin());
    player.sway = if player.character == STILL_SNIPER { Vec2::ZERO } else { wander * SWAY * player.scope };
    // the character's scope sounds: the in sound with each level gone up (FUN_00124340 at the
    // press: the type's +0x260), the out sound as it leaves (FUN_001249b0: +0x264)
    if level != player.scope_heard {
        let id = if level > player.scope_heard { player.scope_sounds.0 } else if level == 0 { player.scope_sounds.1 } else { 0 };
        player.scope_heard = level;
        if id != 0 {
            player.sound_queue.push((id, SCOPE_SOUND_VOLUME));
        }
        if std::env::var("BF_SCOPE_LOG").is_ok() {
            // (with the level's field of view, down and across the game's 4:3 picture)
            let (zoom, fov) = (level_zoom(&player, level), fov_at(level_zoom(&player, level)));
            let across = 2.0 * ((fov * 0.5).tan() / 0.75).atan();
            println!("t={:.2} scope level {level}: zoom {:.2} -> {zoom:.2}, {:.2} degrees down, {:.2} across (lockout {:.3} s)", player.sim_time, player.zoom,
                     fov.to_degrees(), across.to_degrees(), player.scope_lock);
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
    // the follow camera (play_camera.rs): placed after the character's step, its view the aim's.
    // Every other character's is cut to its wanted place every frame (the cut flag, set every
    // frame for a character no player controls: FUN_000702a0 at 0x70548), so the ray its aim
    // takes without a target has a camera to come from, and a hand-over finds the camera in place
    let (at, turn) = camera::update(&mut player, &game.0, dt);
    for m in squad.0.iter_mut() {
        m.rig.cut = true;
        camera::update(m, &game.0, dt);
    }
    // the body: not drawn by its own camera in the scope (the zoom above 1) or with the camera at
    // its near plane (0x126bc0); its fade by the camera is play_stealth.rs's
    if let Some(root) = player.loaded.as_ref().map(|l| l.root) {
        if let Ok(mut v) = vis.get_mut(root) {
            let want = if camera::hides_body(&player) { Visibility::Hidden } else { Visibility::Inherited };
            if *v != want {
                *v = want;
            }
        }
    }
    if let Ok((mut tr, mut projection)) = cam.single_mut() {
        *tr = Transform::from_translation(at).with_rotation(turn);
        if let Projection::Perspective(pp) = projection.as_mut() {
            let fov = fov_at(player.zoom);
            if (pp.fov - fov).abs() > 1e-4 {
                pp.fov = fov;
            }
        }
    }
}

fn update_hud(player: Res<Player>, game: Res<GameData>, mut hud: Query<(&mut Text, &mut Node, &mut TextLayout), With<Hud>>, test: Option<Res<testmap::TestMap>>) {
    let Ok((mut text, mut node, mut layout)) = hud.single_mut() else { return };
    // the "H: help" hint, bottom centre between the radar and the item box; the panel, top left
    // under the health bars, clear of the radar, the item box and the tools' panel (right)
    let (top, bottom, left, right, width, justify) = if player.show_help {
        (Val::Percent(17.0), Val::Auto, Val::Percent(1.5), Val::Auto, Val::Percent(48.0), JustifyText::Left)
    } else {
        (Val::Auto, Val::Px(6.0), Val::Px(0.0), Val::Px(0.0), Val::Auto, JustifyText::Center)
    };
    if node.top != top || node.bottom != bottom {
        (node.top, node.bottom, node.left, node.right, node.max_width) = (top, bottom, left, right, width);
        layout.justify = justify;
    }
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
        if text.0 != "H: help" {
            text.0 = "H: help".into();
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
         WASD move   Shift sprint   Ctrl walk   Space jump   C dodge   Z crouch   Right mouse aim   Left mouse fire   Q switch weapon   R reload (demo)   G use item   Tab next item (hold: the list, W/D S/A or the wheel)   B medkit   E use   V special skill   M surface   H hide\n\
         click: mouse look, Esc release   1-4 take control of Brutus / Flint / Hawk / Tex   G hold to charge a grenade\n\
         test map: walk into a weapon on the rack to take it into the held slot   K instant kill: {}   X die\n\
         {}",
        CHARACTERS[player.character], state, clip, if player.aim { "   [aiming]" } else { "" }, surf, weapon,
        if test.instant_kill { "on" } else { "off" }, testtools::HELP);
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
                    // (an enemy's drops at its feet, #129: the takes' guns lie at the standing spot)
                    let push = if enemies::enemy(p) { Vec3::ZERO } else { p.death_push * DROP_PUSH + Vec3::Y * DROP_HOP };
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
    // (not the dead's: they've let go of their guns, which fall loose, `update_weapons`)
    if std::mem::take(&mut p.weapon_dirty) && !l.weapon_dropped {
        for (i, w) in l.weapons.iter().enumerate() {
            // the gun in the hand, the one drawn into the second hand (Tex's berserk), the rest
            // at their stow points
            let drawn = i == p.weapon && p.holding;
            let second = p.second.is_some_and(|s| s.weapon == i);
            let place = if drawn { w.in_hand } else if second { w.second_hand.or(w.stowed) } else { w.stowed };
            // a gun put away (the second hand's at berserk's end) leaves no flash or light on
            if !drawn && !second {
                if let Ok(mut v) = visibility.get_mut(w.flash) { *v = Visibility::Hidden; }
                if let Ok(mut light) = lights.get_mut(w.light) { light.intensity = 0.0; }
            }
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
        // the barrel's turn: a Minigun's (IFSET_MINIGUN) by its spin level, which
        // play_minigun.rs sets `spin` from (1 at once while firing, falling over 1 s); any other
        // gun's spinning parts by the demo's own ease while firing
        if !minigun::spins(&w.def) {
            let target = if firing { minigun::BARREL_TURN } else { 0.0 };
            p.spin += (target - p.spin) * (1.0 - (-(if firing { 6.0 } else { 1.5 }) * dt).exp());
        }
        p.spin_angle = (p.spin_angle + p.spin * dt) % std::f32::consts::TAU;
        for (e, axis, offset) in &w.spinners {
            if let Ok(mut tr) = transforms.get_mut(*e) {
                *tr = Transform::from_translation(*offset).with_rotation(Quat::from_axis_angle(*axis, p.spin_angle));
            }
        }
        // (the data's muzzle effect replaces the plain flash where there is one, play_shots.rs;
        // the light stays)
        let on = flash_on && w.def.shots.muzzle_effect == 0;
        if let Ok(mut v) = visibility.get_mut(w.flash) {
            *v = if on { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Ok(mut f) = transforms.get_mut(w.flash) {
            f.rotation = f.rotation * Quat::from_rotation_z(1.3);      // vary the flash shape
        }
        if let Ok(mut light) = lights.get_mut(w.light) {
            // (off in the holder's scope: from the hidden gun under the eye it lit a hard-edged
            // disc on the ground; the demo's rule. Off for a gun whose muzzle effect brings its
            // own light: play_shots.rs plays that one)
            light.intensity = if flash_on && p.scope <= 0.5 && !w.data_light { 250_000.0 * bf_viewer::level_scene::POINT_LIGHT_SCALE } else { 0.0 };
        }
    }
    // the second hand's gun (Tex's berserk): its own kick, plain flash and light, as the gun in
    // hand's; its barrels spin with the gun in hand's (one trigger fires both)
    let spin_angle = p.spin_angle;
    if let Some(s) = p.second.as_mut() {
        s.recoil *= (-16.0 * dt).exp();
        let flash_on = s.flash > 0.0;
        s.flash -= dt;
        if let Some(w) = l.weapons.get(s.weapon) {
            if let (Some((_, r, t)), Ok(mut tr)) = (w.second_hand, transforms.get_mut(w.entity)) {
                tr.translation = t - r * w.fire_dir * (0.04 * s.recoil);
                tr.rotation = r;
            }
            for (e, axis, offset) in &w.spinners {
                if let Ok(mut tr) = transforms.get_mut(*e) {
                    *tr = Transform::from_translation(*offset).with_rotation(Quat::from_axis_angle(*axis, spin_angle));
                }
            }
            let on = flash_on && w.def.shots.muzzle_effect == 0;
            if let Ok(mut v) = visibility.get_mut(w.flash) {
                *v = if on { Visibility::Inherited } else { Visibility::Hidden };
            }
            if let Ok(mut light) = lights.get_mut(w.light) {
                light.intensity = if flash_on && !w.data_light { 250_000.0 * bf_viewer::level_scene::POINT_LIGHT_SCALE } else { 0.0 };
            }
        }
    }
    for s in p.shots.drain(..) {
        // (a bolt is play_bolts.rs's from its shot: it flies there, with its flight effect)
        if s.bolt {
            continue;
        }
        // the weapon's own effects (see `projectiles`), else a plain streak
        if s.flight != 0 || s.hit_fx != 0 || s.decal != 0 {
            // (an instant ray's hit waits two frames, `Projectile::wait`)
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

/// A shot drawn with its weapon's effects that isn't a bolt (a bolt is play_bolts.rs's): in
/// flight it carries the flight effect (an instant ray's is placed once along the ray, its beam
/// or tracer, `beam_line`; a shot flying at its speed carries it along), and where it lands on
/// the world it plays the hit (`world_hit`).
#[derive(Component)]
struct Projectile {
    shot: Shot,
    travelled: f32,
    started: bool,
    /// an instant ray's frames before it lands (its beam placed, then emitting)
    wait: u8,
}

/// An effect entity that goes after this many seconds (its particles finish on their own).
#[derive(Component)]
struct AleExpire(f32);

/// Instant-hit guns: how fast the shot reaches its hit point (their tracer particle flies at
/// its own speed, 113.6 m/s for `tracer`).
const INSTANT_SHOT: f32 = 400.0;

impl Shot {
    /// How long after its shot's frame a hit `t` m along this shot lands, for a shot whose hits
    /// are found at the shot (`update_player`, play_scenery.rs, play_sentry.rs): when it gets
    /// there, an instant ray's t / INSTANT_SHOT. (A bolt's come on its sweeps, play_bolts.rs.)
    fn delay(&self, t: f32) -> f32 {
        t / self.speed.max(1.0)
    }

    /// Whether this shot's hits are found by a module of their own rather than by the shots'
    /// readers here: a bolt's on its sweeps as it flies (play_bolts.rs, #136), a pass-through
    /// ray's all along its line (play_rays.rs, #238). The shots' other readers (`update_player`'s
    /// bodies and Sentries, play_scenery.rs, play_pickups.rs, the bodies on the ground) leave them
    /// alone.
    fn swept(&self) -> bool {
        self.bolt || self.pass
    }
}

/// default.xbe's one random state, the LCG at 0x46cfd8 (x 0x19660d + 0x3c6ef35f a draw, from the
/// .bss's 0): a weapon hit takes its damage roll from it (FUN_0022c940: the new state's bits 9-31
/// as a [0, 1) fraction) and its hit sound (FUN_0022b5b0: one of the struck material's four, by
/// its low 2 bits; the demo's by a stirred draw, see `world_hit`). In the game every random
/// choice draws from it, so its draws vary from hit to hit; the demo's copy is drawn by its shots
/// only: each shot's flight effect's seed, a bolt's roll at its launch (play_bolts.rs,
/// FUN_0022bac0), and at a hit on the world the roll (an instant ray's), the hit effects' seed and
/// the sound (`projectiles`, `world_hit`; the seeds are the demo's draws). It replaces a
/// seed from the muzzle's place, which saturated to 0 wherever x 977 + z 131 was negative (every
/// shot at e34's gate: one sound, one roll, one burst).
#[derive(Resource, Default)]
struct GameRandom(u32);

impl GameRandom {
    /// The next state (FUN_0022c940's and FUN_0022b5b0's step).
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
        self.0
    }

    /// A fraction in [0, 1) as FUN_0022c940 makes it: the next state's bits 9-31 under a float's
    /// exponent of 1 (1.0 to 2.0), less 1.
    fn unit(&mut self) -> f32 {
        f32::from_bits((self.next() >> 9) | 0x3F80_0000) - 1.0
    }

    /// The next state stirred by a 32-bit integer hash (the "lowbias32" shifts and multipliers,
    /// the demo's choice) for the demo's own uses, the effects' seeds and the hit sound's pick: a
    /// run of the LCG's states drawn the same number of times a shot repeats itself in its low
    /// bits (a period of 4: x 0x19660d is 1 mod 4, + 0x3c6ef35f is 3) and clusters in its top
    /// ones (a lone shooter's first five gate hits drew one sound of the four).
    fn stirred(&mut self) -> u32 {
        let mut x = self.next();
        x ^= x >> 16;
        x = x.wrapping_mul(0x7FEB_352D);
        x ^= x >> 15;
        x = x.wrapping_mul(0x846C_A68B);
        x ^ (x >> 16)
    }
}

/// The ALE effects of an effect type, compiled.
fn effects_of(game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
              materials: &mut Assets<StandardMaterial>, effect_type: u32) -> Vec<std::sync::Arc<bf_viewer::ale_fx::Compiled>> {
    let list = game.effect_types.get(&effect_type).cloned().unwrap_or_default();
    list.into_iter().filter_map(|e| ale.load(game, images, materials, e)).collect()
}

/// An instant ray's flight effect type's ALE effects, compiled for a beam (`AleAssets::load_beam`:
/// their kill fields run).
fn beam_effects_of(game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
                   materials: &mut Assets<StandardMaterial>, effect_type: u32) -> Vec<std::sync::Arc<bf_viewer::ale_fx::Compiled>> {
    let list = game.effect_types.get(&effect_type).cloned().unwrap_or_default();
    list.into_iter().filter_map(|e| ale.load_beam(game, images, materials, e)).collect()
}

/// The farthest an instant ray's beam reaches by its user parameter (m): FUN_002312e0 clamps the
/// ray's length to 200.0 and takes x 0.005 of it.
const BEAM_REACH: f32 = 200.0;
/// How far right of the scoped eye (m) a scoped ray's beam starts (FUN_002312e0: x 0.25 the frame
/// FUN_0001d0c0 builds from the ray's way, its first axis)...
const BEAM_SCOPED_RIGHT: f32 = 0.25;
/// ... and how far below it (m: x 0.1 its second axis, taken off).
const BEAM_SCOPED_DOWN: f32 = 0.1;

/// Where an instant ray's flight effect is placed and its user parameter (FUN_002317e0 ->
/// FUN_002312e0, #237): at the muzzle, turned along the ray (FUN_0001d0c0's frame: its -z along
/// the way, its x the world's up x that), with the user parameter min(d, BEAM_REACH) /
/// BEAM_REACH for the ray's length d (`Shot::dist`: where it hit, or stopped going through,
/// play_rays.rs; its range when it met nothing; FUN_00230860 writes it back). The effects are
/// authored so: their emitters, turned -90 degrees about x, throw and carry their particles along
/// the -z (tracer.emt at 113.6 m/s; rgun_beam.emt's offset out to 100 m in 0.24 s), and their kill
/// fields stand that far out (ale_fx.rs CLASS_KILL_FIELD). Fired from the scoped eye, the beam
/// starts BEAM_SCOPED_RIGHT right of and BEAM_SCOPED_DOWN below it and is aimed from there at the
/// ray's end. Before #237 the demo spun the effect's +y along the shot, which its emitters' turn
/// threw down instead (the tracers went into the ground at the muzzle), and gave it no length.
fn beam_line(s: &Shot) -> (Transform, f32) {
    let up = |d: Vec3| if d.y.abs() > 0.9848 { Vec3::Z } else { Vec3::Y };
    let sp = s.dist.min(BEAM_REACH) / BEAM_REACH;
    if !s.scoped {
        return (Transform::from_translation(s.origin).looking_to(s.dir, up(s.dir)), sp);
    }
    let frame = Transform::default().looking_to(s.dir, up(s.dir));
    let from = s.origin + frame.right() * BEAM_SCOPED_RIGHT - frame.up() * BEAM_SCOPED_DOWN;
    let to = s.origin + s.dir * s.dist;
    let way = (to - from).normalize_or(s.dir);
    (Transform::from_translation(from).looking_to(way, up(way)), sp)
}

/// FUN_0022cd80's user parameter for the struck material's hit effect and its debris, from the
/// hit record's damage (the damage handler 0x22d0b0 passes the record's +8, `damage::Hit::record`):
/// 0.036 of it, at most 0.9, or 1.0 when the hit killed (its second argument, the handler's kill
/// flag). The weapon's own hit effect, played where the material has none, takes 1.0. It picks
/// between the effects' variants (plasma_hit_green_spark's rate, wmat_concrete's speed, the
/// bio-flash's emit count ...).
fn hit_sp(record: f32, killed: bool) -> f32 {
    if killed { 1.0 } else { (0.036 * record).min(0.9) }
}

/// A shot's hit effect frame for an alignment-type (EffectType::aligns, FUN_0021a850): a frame
/// FUN_0001d0c0 builds from a direction d, its z axis -d and its x up x z (Bevy's `looking_to`),
/// from the shot's way (0, ALIGN_TO_EFFECT: FUN_0022cbe0 turns the effect by the hit's velocity),
/// that way reflected about the struck surface (1, inferred: FUN_0021a850 reflects the effect
/// frame's own axis, FUN_0001fdd0, about what isn't read; the data's three alchemy entries with
/// it name no effect), the surface's normal (2, the hit record's +0x10, stored as it is, outward:
/// answers-batch1.md §2, the sign inferred) or the world's axes (3). The hits' emitters are
/// turned -90 degrees about x (sonic_can_hit.emt, plasma_hit_green_spark.emt,
/// sminigun_hit.emt ...), so they fire along d: out of the surface for 2, on along the shot for
/// 0. (The demo turned every hit's +y back along the shot before, so those emitters fired across
/// it.)
fn hit_frame(at: Vec3, align: i64, dir: Vec3, normal: Vec3) -> Transform {
    let place = Transform::from_translation(at);
    let d = match align {
        1 => dir - 2.0 * dir.dot(normal) * normal,
        2 => normal,
        3 => return place,
        _ => dir,
    }.normalize_or(dir);
    // (FUN_0001d0c0's up hint: world y, or world z for a way within 10 degrees of the vertical)
    place.looking_to(d, if d.y.abs() > 0.9848 { Vec3::Z } else { Vec3::Y })
}

/// Effect type `kind` played once at a hit `at` (FUN_0022cbe0): each of its ALE effects in its
/// alignment's frame (`hit_frame`), and its light-effect (a "light_" effect: a point light, its
/// size the reach, ale_fx.rs) at the point, with the user parameter `sp` (FUN_0021c380) and the
/// type's bounding sphere for the distance LOD. Compiled with the game's emission count, shapes
/// and facing 3 (`load_recorded_facing`): the hits' bursts are emit-count keys (sonic_can_hit.emt
/// 16.3 / 17.2 / 18.4), one particle under the demo's other count.
#[allow(clippy::too_many_arguments)]
fn hit_effect_at(commands: &mut Commands, game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
                            materials: &mut Assets<StandardMaterial>, kind: u32, at: Vec3, dir: Vec3, normal: Vec3, sp: f32, seed: u32) {
    let Some(t) = game.effect_type_defs.get(&kind).cloned() else { return };
    let effects = t.effects.iter().enumerate().map(|(i, &e)| (e, t.aligns.get(i).copied().unwrap_or(0)));
    for (k, (effect, align)) in effects.chain((t.light != 0).then_some((t.light, 0))).enumerate() {
        let Some(fx) = ale.load_recorded_facing(game, images, materials, effect) else { continue };
        let life = fx.duration();
        let run = bf_viewer::ale_fx::AleEffect::once(fx, sp, seed.wrapping_add((k as u32).wrapping_mul(0x9E37_79B9))).with_lod_sphere(t.bounds);
        commands.spawn((hit_frame(at, align, dir, normal), Visibility::default(), run, AleExpire(life)));
    }
}

/// A bolt's flight effect type started on it: its ALE effects (and its light, if it has one)
/// riding on the shot's entity `on`, compiled for a flight (`load_flight`: the heads' particles
/// travel with the bolt and go with it, the trails' stay in the world; the game's facing 3).
#[allow(clippy::too_many_arguments)]
fn start_flight(commands: &mut Commands, game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
                materials: &mut Assets<StandardMaterial>, kind: u32, on: Entity, seed: u32) {
    let Some(t) = game.effect_type_defs.get(&kind).cloned() else { return };
    for (k, &effect) in t.effects.iter().chain((t.light != 0).then_some(&t.light)).enumerate() {
        let Some(fx) = ale.load_flight(game, images, materials, effect) else { continue };
        let run = bf_viewer::ale_fx::AleEffect::new(fx, 0.0, seed.wrapping_add((k as u32).wrapping_mul(0x9E37_79B9))).with_lod_sphere(t.bounds);
        commands.spawn((Transform::default(), Visibility::default(), run, ChildOf(on)));
    }
}

#[allow(clippy::too_many_arguments)]
fn projectiles(mut commands: Commands, time: Res<Time>, mut game: ResMut<GameData>, ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
               mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>,
               mut shots: Query<(Entity, &mut Projectile, &mut Transform)>, mut expiring: Query<(Entity, &mut AleExpire)>,
               mut holes: ResMut<fx::HoleRequests>, mut player: ResMut<Player>, camera: Query<&GlobalTransform, With<MainCamera>>,
               mut chance: ResMut<GameRandom>) {
    let dt = frame_dt(&time);
    for (e, mut x) in &mut expiring {
        x.0 -= dt;
        if x.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
    let Some(mut ale) = ale else { return };
    let listener = camera.iter().next().map(|c| c.translation());
    for (e, mut p, mut tr) in &mut shots {
        let s = p.shot;
        let instant = s.speed >= INSTANT_SHOT;
        if !p.started {
            p.started = true;
            // (its flight effect's seed: a draw of the shots' random state)
            let seed = chance.stirred();
            if instant {
                // the beam (#237): its effect placed once along the ray (`beam_line`), its user
                // parameter the ray's length over 200 m, where its kill fields stop it
                let (place, sp) = beam_line(&s);
                for fx in beam_effects_of(&mut game.0, &mut ale, &mut images, &mut materials, s.flight) {
                    let life = fx.duration();
                    commands.spawn((place, Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx, sp, seed), AleExpire(life)));
                }
            } else {
                for fx in effects_of(&mut game.0, &mut ale, &mut images, &mut materials, s.flight) {
                    // (a shot flying at its speed that isn't a bolt: the effect rides on it)
                    commands.spawn((Transform::default(), Visibility::default(), bf_viewer::ale_fx::AleEffect::new(fx, 0.0, seed), ChildOf(e)));
                }
            }
        }
        // the first frames only start the beam (placed, then emitting)
        if p.wait > 0 {
            p.wait -= 1;
            continue;
        }
        p.travelled += s.speed * dt;
        if p.travelled < s.dist {
            tr.translation = s.origin + s.dir * p.travelled;
            continue;
        }
        // landed: the hit on the world (bodies bleed instead, see play_fx)
        let at = s.origin + s.dir * s.dist;
        commands.entity(e).despawn();
        // through a liquid's surface on the way: an instant ray's (bullet-type 4) splash there,
        // FUN_0022feb0 from FUN_00230860's hit loop (0x230f7c): the liquid type's shot effect
        // upright at user parameter 1.0 (0x22ff6b) and its shot sound at the crossing, kind 7, at
        // the library's volume and pitch (no play parameters set, 0x22ff8f-0x22ffcf). A flying
        // shot or a bolt has no liquid code (todo/146-liquids leads §5.1). (The ocean's trail along
        // the ray under the surface, +0x168, isn't done.)
        let ray = game.0.weapons.get(&s.weapon).is_some_and(|d| d.shots.bullet_type == 4);
        if let Some((surface, liquid)) = world::arena().filter(|_| ray).and_then(|a| a.liquid_crossing(s.origin, at)) {
            let effect = liquid.effects[LIQUID_SHOT];
            if effect != 0 {
                spawn_once(&mut commands, &mut game.0, &mut ale, &mut images, &mut materials, effect, Transform::from_translation(surface), 1.0);
            }
            if liquid.sounds[LIQUID_SHOT] != 0 {
                player.world_sounds.push(WorldSound { id: liquid.sounds[LIQUID_SHOT], at: Some(surface), volume: None, pitch: 0.0, kind: KIND_RAY });
            }
            if std::env::var("BF_LIQUID_LOG").is_ok() {
                println!("t {:.2}: an instant ray through a liquid's surface (type {}) at {surface:.2}: effect h_{effect:08x}, sound {:08x}",
                         player.sim_time, liquid.kind, liquid.sounds[LIQUID_SHOT]);
            }
        }
        if s.hit {
            // the bullet's hole or scorch where it met the world (play_fx.rs)
            if s.decal != 0 {
                holes.0.push((s.decal, at, s.normal));
            }
            world_hit(&mut commands, &mut game.0, &mut ale, &mut images, &mut materials, &s, at, &mut chance, listener, &mut player, None);
        }
    }
}

/// How loud a sound at `at` is heard: the library's volume, by its distances from the listener
/// (`distance_gain`) where it has them.
fn heard_at(game: &Game, id: u32, at: Vec3, listener: Option<Vec3>) -> f32 {
    match (game.sounds.params(id), listener) {
        (Some(p), Some(l)) if p.min_distance >= 0.0 => triggers::sounds::distance_gain(l.distance(at), p.min_distance, p.max_distance) * p.volume,
        (Some(p), _) => p.volume,
        _ => 1.0,
    }
}

/// A shot's explosion at a direct hit, on the world or a body (the damage handler 0x22d0b0 ->
/// FUN_0022cfc0, before the struck material's hit effect, FUN_0022cd80): the bullet's own effect
/// type (`WeaponDef::blast_effect`, h_ec23d593, the weapon type's +0x250) through FUN_0022cbe0 at
/// user parameter 1.0, in its alignment's frame (`hit_effect_at`), and its impact sound
/// (`WeaponDef::impact_sound`, h_e5618348, +0x254) heard from the hit. Of the hand guns' shots
/// only the two cutters' come here with one (cutter_hit and plut_cutter_hit with
/// light_hit_cutter: the white star where a blade strikes, #53 take01 and take03, #65). The
/// iKhan-GPL names one too (h_e26237da, exp_mirvgrenade), and after weapons batch 2 its round,
/// then a plain shot, played it here where it hit (#287); since #239 it flies as a lobbed
/// missile, which plays it where it goes off (play_missile.rs). The grenades' and missiles'
/// explosions are their own (play_grenade.rs, play_missile.rs); the hounds' Breath Attacks, left
/// on the plain shot (`missile::flies`), still play theirs here where it meets the world. Its
/// particles' seed is a draw of `chance` when
/// there's an effect, so no other gun's draws move. Not done: the per-material overrides at
/// +0x25c, which FUN_0022cfc0 takes when the hit record's +0x58 bit 1 is set (not traced for a
/// bolt). Returns the effect type played, for the logs.
#[allow(clippy::too_many_arguments)]
fn shot_explosion(commands: &mut Commands, game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
                  materials: &mut Assets<StandardMaterial>, s: &Shot, at: Vec3, normal: Vec3, chance: &mut GameRandom,
                  listener: Option<Vec3>, player: &mut Player) -> Option<u32> {
    let (effect, sound) = game.weapons.get(&s.weapon).map_or((0, 0), |d| (d.blast_effect, d.impact_sound));
    if sound != 0 && game.sounds.has(sound) {
        let volume = heard_at(game, sound, at, listener);
        if volume > 0.0 {
            player.sound_queue.push((sound, volume));
        }
    }
    if effect == 0 {
        return None;
    }
    let seed = chance.stirred();
    hit_effect_at(commands, game, ale, images, materials, effect, at, s.dir, normal, 1.0, seed);
    Some(effect)
}

/// A shot landing on the world at `at`: the damage handler's tail (0x22d0b0 -> FUN_0022cd80,
/// FUN_0022b5b0) by the struck triangle's world material (`Arena::ray_material`; the flat test
/// floor's 0) and the shot's ammo-type, after the bullet's own explosion (`shot_explosion`, the
/// cutters' white star):
///  - the material's hit effect for the ammo-type (world-materials h_f6f08598; the world's
///    materials have none, only the fleshes), else the weapon's own (h_08d28037) at user
///    parameter 1.0;
///  - the material's debris effect (`debris-effect`: wmat_sand's tan puff, wmat_concrete's
///    grey dust ...), the record's user parameter (`hit_sp`, the roll x the falloff: no kill);
///  - each with its light-effect (the hit lights: light_hit_sonic's cyan wash, #57; the debris
///    have none) and in its alignment's frame (`hit_effect_at`);
///  - one of the material row's four hit sounds (FUN_0022b5b0: the row's +0x28 entries, picked
///    by the random state's low 2 bits), heard from the hit by the library's distances
///    (`distance_gain`: the sonic rows' full within 9 m, silent beyond 20 m).
///
/// The roll, the effects' seed and the sound's pick are three draws of `chance` (GameRandom), in
/// the handler's order: the record's roll, the effects, the sound. A bolt's hit brings its record
/// and the struck material (`swept`: the roll FUN_0022bac0 made at its launch, and the material
/// its sweep met, which leaves the level's blockers out, play_bolts.rs), so no roll is drawn for
/// it.
#[allow(clippy::too_many_arguments)]
fn world_hit(commands: &mut Commands, game: &mut Game, ale: &mut bf_viewer::ale_fx::AleAssets, images: &mut Assets<Image>,
             materials: &mut Assets<StandardMaterial>, s: &Shot, at: Vec3, chance: &mut GameRandom, listener: Option<Vec3>,
             player: &mut Player, swept: Option<(f32, i64)>) {
    let material = swept.map(|x| x.1)
        .unwrap_or_else(|| world::arena().and_then(|a| a.ray_material(s.origin, s.dir, s.dist + 0.1)).map_or(0, |(_, m)| m as i64));
    let surface = game.surfaces.iter().find(|x| x.id == material).cloned();
    // (the roll the record carries, FUN_0022c940: Damage min..max x an instant ray's falloff;
    // the world takes no hit part)
    let [lo, hi] = s.damage;
    let roll = swept.map(|x| x.0).unwrap_or_else(|| (lo + (hi - lo) * chance.unit()) * s.falloff_at(s.dist));
    let sp = hit_sp(roll, false);
    let place = at - s.dir * 0.05;
    let explosion = shot_explosion(commands, game, ale, images, materials, s, place, s.normal, chance, listener, player);
    // (the hit effects' particles: the demo's draw)
    let seed = chance.stirred();
    let row = surface.as_ref().and_then(|x| x.hit_effect.get(&s.ammo)).copied();
    let (main, main_sp) = match row {
        Some(kind) => (kind, sp),
        None => (s.hit_fx, 1.0),
    };
    let log = std::env::var("BF_SHOT_LOG").is_ok();
    if main != 0 {
        hit_effect_at(commands, game, ale, images, materials, main, place, s.dir, s.normal, main_sp, seed);
    }
    let debris = surface.as_ref().and_then(|x| x.debris_effect).filter(|&d| d != 0);
    if let Some(d) = debris {
        hit_effect_at(commands, game, ale, images, materials, d, place, s.dir, s.normal, sp, seed ^ 0x5DEE_CE66);
    }
    // (FUN_0022b5b0: one of the row's four slots for a draw. The game takes the state's low 2
    // bits, which an LCG of this kind steps through with a period of 4: there the rest of the
    // game's draws between two hits mix them, here a shot draws the same number every time and
    // the low bits would pick one slot for good, so the demo takes the top 2 of a stirred draw
    // (`GameRandom::stirred`). The row as loaded keeps its ids in order and drops empty ones, and
    // every hit row in the data has all four or none; a missing slot, or an id the level's banks
    // don't hold, is silent.)
    let sounds = surface.as_ref().and_then(|x| x.hit_sounds.get(&s.ammo)).cloned().unwrap_or_default();
    let slot = (chance.stirred() >> 30) as usize;
    let mut heard = None;
    if let Some(id) = sounds.get(slot).copied().filter(|&id| game.sounds.has(id)) {
        let volume = heard_at(game, id, at, listener);
        if volume > 0.0 {
            player.sound_queue.push((id, volume));
        }
        heard = Some((id, volume));
    }
    if log {
        let light = |k: u32| game.effect_type_defs.get(&k).map_or(0, |t| t.light);
        // (a bolt's sweep starts at its last checked point: its own line says how far out it is)
        let out = if swept.is_some() { format!("{:.1} m along its sweep", s.dist) } else { format!("{:.1} m out", s.dist) };
        println!("t={:.2} hit on world material {material} at {at:.2} {out} (ammo-type {}): {}effect h_{main:08x} sp {main_sp:.2} light h_{:08x}{}, debris {}, sound {}",
                 player.sim_time, s.ammo, explosion.map_or(String::new(), |x| format!("explosion h_{x:08x} light h_{:08x}, ", light(x))),
                 light(main), if row.is_some() { " (the material's)" } else { " (the weapon's)" },
                 debris.map_or("none".into(), |d| format!("h_{d:08x} sp {sp:.2}")),
                 heard.map_or("none".into(), |(id, v)| format!("h_{id:08x} vol {v:.2} (slot {slot})")));
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
    // (once the character is in; the map editor's stand-in never has a body to wait for)
    if player.loaded.is_none() && !player.stand_in {
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

/// BF_DUMP_TYPES=all: every archive of data/ into `game` (common's and the map's are in already;
/// one loaded twice adds nothing: first loaded wins), in name order, as the test map loads its
/// levels (`Game::load_level_data`: their data, not their texture libraries).
fn dump_all_archives(game: &mut Game) {
    let t = std::time::Instant::now();
    let mut names: Vec<String> = std::fs::read_dir(data_dir()).into_iter().flatten().filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".tgz")).map(str::to_string))
        .filter(|n| n != "common").collect();
    names.sort();
    for n in &names {
        if let Err(e) = game.load_level_data(&data_dir(), n) {
            eprintln!("{n}: {e}");
        }
    }
    println!("loaded {} archives in {:.1} s", names.len(), t.elapsed().as_secs_f32());
}

/// BF_DUMP_WEAPONS=1: each character's weapons as loaded (definition, model, hardpoints, sounds).
fn dump_weapons(game: &Game) {
    println!("{} weapon definitions, {} strings", game.weapons.len(), game.strings.len());
    for &name in &CHARACTERS[..SQUAD] {
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
                    // (each muzzle the definition names, found on the model or not)
                    let muzzles: Vec<String> = d.muzzles.iter().map(|k| format!("{k:08x} {}", if m.hardpoints.contains_key(k) { "ok" } else { "MISSING" })).collect();
                    println!(" parts [{}] hardpoints [{}] muzzles [{}]", geo.join(", "), hps.join(" "), muzzles.join(", "));
                }
                Err(e) => println!(" model error: {e}"),
            }
        }
    }
    // the definitions with more than one muzzle (the map's turrets), each found on its model or not
    let mut many: Vec<&weapon::WeaponDef> = game.weapons.values().filter(|d| d.muzzles.len() > 1).collect();
    many.sort_by_key(|d| d.name);
    for d in many {
        let model = weapon::WeaponModel::load(game, d.archetype).ok();
        let muzzles: Vec<String> = d.muzzles.iter().map(|k| format!("{k:08x} {}", match &model {
            Some(m) if m.hardpoints.contains_key(k) => "ok",
            Some(_) => "MISSING",
            None => "no model",
        })).collect();
        println!("h_{:08x} ({}, arch {:08x}): muzzles [{}]", d.name, d.label, d.archetype, muzzles.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #5 (FUN_00179250): a liquid sound's entry (min 5 m, max 15 m, volume 0.5) heard at its own
    /// volume within 5 m, by the rolloff curve to 15 m and not at all from there; at the entry's
    /// volume when it asks none; 2D at its volume wherever the listener is; none without an entry.
    #[test]
    fn a_world_sound_is_heard_by_its_entry() {
        let entry = Some(bf_viewer::bf::audio::SoundParams { min_distance: 5.0, max_distance: 15.0, volume: 0.5 });
        let splash = |x: f32| WorldSound { id: 1, at: Some(Vec3::new(x, 0.0, 0.0)), volume: Some(1.0), pitch: 0.0, kind: KIND_SPLASH };
        assert_eq!(splash(4.0).heard(entry, Vec3::ZERO), Some((1.0, Some(4.0))));
        // (the curve's second stretch, 7-9 m: 1.0 to 0.375)
        let (v, _) = splash(8.0).heard(entry, Vec3::ZERO).unwrap();
        assert!((v - 0.6875).abs() < 1e-5, "{v}");
        assert!(splash(14.9).heard(entry, Vec3::ZERO).is_some_and(|h| h.0 < 0.01));
        assert_eq!(splash(15.0).heard(entry, Vec3::ZERO), None);
        assert_eq!(splash(4.0).heard(None, Vec3::ZERO), None);
        let ray = WorldSound { volume: None, kind: KIND_RAY, ..splash(2.0) };
        assert_eq!(ray.heard(entry, Vec3::ZERO), Some((0.5, Some(2.0))));
        let own_step = WorldSound { at: None, volume: Some(0.375), kind: KIND_STEP, ..splash(0.0) };
        assert_eq!(own_step.heard(entry, Vec3::new(100.0, 0.0, 0.0)), Some((0.375, None)));
    }

    /// #229: the seed is the old product for the numbers that fitted, and a number past the old
    /// limit (2^32 / 104729, about 41,010) wraps instead of overflowing.
    #[test]
    fn shot_seed_wraps_past_the_old_limit() {
        for (c, id) in [(0usize, None), (3, None), (2, Some(1u32)), (1, Some(20_000)), (3, Some(41_000))] {
            assert_eq!(shot_seed(c, id), c as u32 * 7919 + id.map_or(0, |n| n * 104_729));
        }
        assert_eq!(shot_seed(3, Some(41_011)), (3u64 * 7919 + 41_011 * 104_729) as u32);
        assert_eq!(shot_seed(4, Some(u32::MAX)), (4u64 * 7919 + u32::MAX as u64 * 104_729) as u32);
    }

    /// #237 (FUN_002312e0): an instant ray's beam at the muzzle along the ray, its user parameter
    /// the length over 200 m and 1 past it; scoped, from 0.25 m right of the eye and 0.1 m below,
    /// aimed at the ray's end.
    #[test]
    fn a_beam_spans_the_ray() {
        let shot = |dist: f32, scoped: bool| Shot { weapon: 0, origin: Vec3::new(1.0, 2.0, 3.0), dir: Vec3::NEG_Z, dist, hit: true, speed: INSTANT_SHOT,
                                                    flight: 0, hit_fx: 0, damage: [1.0, 2.0], damage_type: 0, ammo: 0, decal: 0, normal: Vec3::Z,
                                                    falloff: 0, range: 300.0, bolt: false, body: false, pass: false, scoped };
        let (place, sp) = beam_line(&shot(27.0, false));
        assert!(place.translation.distance(Vec3::new(1.0, 2.0, 3.0)) < 1e-6);
        assert!(place.forward().as_vec3().distance(Vec3::NEG_Z) < 1e-6);
        assert!((sp - 0.135).abs() < 1e-6);
        assert_eq!(beam_line(&shot(250.0, false)).1, 1.0);
        // scoped: right is +x and up +y for a ray along -z
        let (place, sp) = beam_line(&shot(25.0, true));
        let from = Vec3::new(1.25, 1.9, 3.0);
        assert!(place.translation.distance(from) < 1e-6);
        let end = Vec3::new(1.0, 2.0, 3.0 - 25.0);
        assert!(place.forward().as_vec3().distance((end - from).normalize()) < 1e-6);
        assert!((sp - 0.125).abs() < 1e-6);
    }
}
