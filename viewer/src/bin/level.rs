//! Brute Force level viewer: a level's terrain, placed objects and sky from the original archives.
//!
//!   cargo run --release --bin bf_level -- [level]      (default sdm_e34, "Battle of Bulgar")
//!
//! Controls: right mouse drag looks around, W/A/S/D move, Q/E down/up, Shift faster;
//! 1-9 jump to the level's own cameras (its flyby camera objects), 0 the overview; F toggles the
//! level's fog; P prints the camera position.
//!
//! Test hooks: BF_LEVEL_CAMERA=<n> starts at the level's camera n (0 = overview),
//! BF_VIEW=x,y,z,yaw,pitch starts there (degrees), BF_NO_FOG=1 starts without fog, BF_SCREENSHOT=<file.png> saves a screenshot
//! once everything has loaded, then quits. BF_PART_DUMP=<hex archetype> prints its parts' geosets
//! (bounds, normals, winding, UV direction) and exits; with BF_PART_VERTS=1 every vertex, not the first 6.
//! BF_TEX_LOG=1 prints where each texture asked for came from (`Game::texture_entry`);
//! BF_TEX_LOG=squad prints the squad's skin textures on the level (archive, bytes) and exits.

use std::path::PathBuf;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    pbr::{DistanceFog, FogFalloff},
    prelude::*,
};
use bf_viewer::bf::character::Game;
use bf_viewer::bf::level::Level;
use bf_viewer::level_scene::spawn_level;

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

#[derive(Resource)]
struct World {
    game: Game,
    level: Level,
    name: String,
}

/// The level's fog (F toggles it).
#[derive(Resource)]
struct LevelFog(Option<DistanceFog>);

#[derive(Resource, Default)]
struct Fly {
    yaw: f32,
    pitch: f32,
    frames: u32,
}

/// BF_SCRIPT_CHECK=1: every data/*.tgz archive's cutscene scripts (scripts/*.gscr) through the
/// reader (bf/gscr.rs, #222): a line per script (its duration, entities, events, the USER_EVENT
/// codes and other event types it has), the ones that don't read, and the totals: 179 scripts in
/// the 39 archives that have any, all read.
fn script_check(dir: &std::path::Path) {
    use bf_viewer::bf::gscr::{EventKind, Script};
    let mut archives: Vec<PathBuf> = std::fs::read_dir(dir).map(|d| d.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "tgz")).collect()).unwrap_or_default();
    archives.sort();
    let (mut ok, mut bad) = (0, 0);
    let mut codes: std::collections::BTreeMap<String, usize> = Default::default();
    for path in archives {
        let Ok(ar) = bf_viewer::bf::archive::Archive::open(&path, |n| n.ends_with(".gscr")) else { continue };
        let level = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        for f in ar.find("", ".gscr") {
            let text = String::from_utf8_lossy(ar.get(&f).unwrap_or(&[])).into_owned();
            match Script::parse(f.trim_end_matches(".gscr"), &text) {
                Ok(s) => {
                    ok += 1;
                    let mut mine: std::collections::BTreeMap<String, usize> = Default::default();
                    for e in &s.events {
                        let k = match &e.kind {
                            EventKind::User(u) => u.code.to_string(),
                            EventKind::Path(_) => "path".to_string(),
                            EventKind::Other(k, _) => format!("type {k}"),
                        };
                        *mine.entry(k.clone()).or_default() += 1;
                        *codes.entry(k).or_default() += 1;
                    }
                    println!("{level}/{f}: {:.3} s, {} entities, {} events: {}", s.duration, s.entities.len(), s.events.len(),
                             mine.iter().map(|(k, n)| format!("{k} x{n}")).collect::<Vec<_>>().join(", "));
                }
                Err(e) => {
                    bad += 1;
                    println!("{level}/{f}: does not read: {e}");
                }
            }
        }
    }
    println!("scripts: {ok} read, {bad} not; events by code / type: {}", codes.iter().map(|(k, n)| format!("{k} x{n}")).collect::<Vec<_>>().join(", "));
}

/// BF_TEX_LOG=squad: the textures the squad's four models (LOD 0) are drawn with on `level`, one
/// line per member and texture: the archive it came from and its bytes' fingerprint
/// (`Game::texture_origin`), which tells the level's outfit (#284: five sets, the same grouping
/// for all eight skins). The model's materials are common's; the textures the level's library,
/// and on a multiplayer map mp_common's library of the match after it, as bf_play's `load_map`
/// loads it: `dm` on a deathmatch arena, the Brute Force squad's `sdm_bf` on a squad
/// deathmatch map (game-options-en.xml).
fn squad_skins(game: &mut Game, level: &str) {
    use bf_viewer::bf::character::{LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};
    let kind = game.campaign.iter().find(|l| l.file == level || l.zones.iter().any(|z| z == level)).map(|l| l.kind);
    let library = match kind {
        Some(LEVEL_DEATHMATCH) => Some("dm"),
        Some(LEVEL_SQUAD_DEATHMATCH) => Some("sdm_bf"),
        _ => None,
    };
    if let Some(library) = library {
        if let Err(e) = game.load_match_textures(&data_dir(), library) {
            println!("{level}: no {library} textures from mp_common: {e}");
        }
    }
    for name in ["hawk", "tex", "brutus", "flint"] {
        let Ok(model) = bf_viewer::bf::character::Character::load(game, name, 0) else {
            println!("{level} {name}: no model");
            continue;
        };
        let mut seen: Vec<u32> = vec![];
        for t in model.geosets.iter().filter_map(|g| game.material_texture(g.material)).collect::<Vec<_>>() {
            if seen.contains(&t) {
                continue;
            }
            seen.push(t);
            match game.texture_origin(t) {
                Some((archive, bytes)) => println!("{level} {name} h_{t:08x} <- {archive} bytes {bytes:08x}"),
                None => println!("{level} {name} h_{t:08x} missing"),
            }
        }
    }
}

fn main() {
    if std::env::var("BF_SCRIPT_CHECK").is_ok() {
        script_check(&data_dir());
        return;
    }
    let name = std::env::args().nth(1).or_else(|| std::env::var("BF_LEVEL").ok()).unwrap_or_else(|| "sdm_e34".into());
    let mut game = Game::load(&data_dir()).unwrap_or_else(|e| {
        eprintln!("failed to load game data from {}: {e}", data_dir().display());
        std::process::exit(1)
    });
    if let Err(e) = game.load_level(&data_dir(), &name) {
        eprintln!("failed to load level {name}: {e}");
        std::process::exit(1)
    }
    if std::env::var("BF_TEX_LOG").is_ok_and(|v| v == "squad") {
        squad_skins(&mut game, &name);
        return;
    }
    let mut level = Level::load(&game).unwrap_or_else(|e| {
        eprintln!("{name}: {e}");
        std::process::exit(1)
    });
    if let Some(arch) = std::env::var("BF_PART_DUMP").ok().and_then(|v| u32::from_str_radix(&v, 16).ok()) {
        if let Ok(m) = bf_viewer::bf::weapon::WeaponModel::load(&game, arch) {
            for p in &m.parts {
                for g in &p.geosets {
                    let (mut lo, mut hi, mut n) = (Vec3::MAX, Vec3::MIN, Vec3::ZERO);
                    for (v, nn) in g.positions.iter().zip(&g.normals) {
                        lo = lo.min(Vec3::from(*v));
                        hi = hi.max(Vec3::from(*v));
                        n += Vec3::from(*nn);
                    }
                    let mut wind = 0.0;
                    for t in g.indices.chunks_exact(3) {
                        let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(g.positions[i as usize]));
                        let nn: Vec3 = [t[0], t[1], t[2]].iter().map(|&i| Vec3::from(g.normals[i as usize])).sum();
                        wind += (b - a).cross(c - a).dot(nn).signum();
                    }
                    let (mx, mu) = (g.positions.iter().map(|v| v[0]).sum::<f32>() / g.positions.len() as f32, g.uvs.iter().map(|u| u[0]).sum::<f32>() / g.uvs.len() as f32);
                    let corr: f32 = g.positions.iter().zip(&g.uvs).map(|(v, u)| (v[0] - mx) * (u[0] - mu)).sum();
                    let front: Vec<String> = g.positions.iter().zip(&g.normals).zip(&g.uvs).take(if std::env::var("BF_PART_VERTS").is_ok() { usize::MAX } else { 6 }).map(|((v, n), u)| format!("p{:.2?} n{:.2?} uv{:.2?}", v, n, u)).collect();
                    println!("  u-vs-x {corr:.2}; {}", front.join(" | "));
                    println!("part h_{:08x} at {:.2} rot {:.2} mat h_{:08x}: {} verts, box {:.2}..{:.2}, normal sum {:.1}, winding agree {wind}/{}",
                             p.name, p.offset, p.rotation, g.material, g.positions.len(), lo, hi, n, g.indices.len() / 3);
                }
            }
        }
        std::process::exit(0);
    }
    // pickups sit at their spawn height in the file; drop them onto the floor below
    bf_viewer::arena::Arena::install(&game, &level).settle_pickups(&game, &mut level);
    let with_mesh = level.objects.iter().filter(|o| o.archetype.is_some()).count();
    if std::env::var("BF_LEVEL_DUMP").is_ok() {
        for g in &level.sky {
            let (lo, hi) = g.positions.iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(l, u), p| (l.min(Vec3::from(*p)), u.max(Vec3::from(*p))));
            println!("sky geoset material h_{:08x} type h_{:08x} texture {:?}: {} vertices {} indices, bounds {lo:.0} .. {hi:.0}, sky at {:.0}",
                     g.material, game.material_type(g.material), game.material_texture(g.material).map(|t| format!("h_{t:08x}")),
                     g.positions.len(), g.indices.len(), level.sky_at);
        }
        for l in &level.lights {
            println!("light h_{:08x} type h_{:08x}{}{} colour {:.2?} dir {:.2}", l.name, l.kind, if l.indoor { " indoor" } else { "" },
                     if l.outdoor { " outdoor" } else { "" }, l.color, l.dir);
        }
        println!("ambient objects {:.2?} characters {:.2?} terrain {:.2?}", level.ambient, level.character_ambient, level.terrain_ambient);
        // which way the baked light map lies: correlate it with the slope light of a light marked
        // h_e02aba1c (read as the terrain's before #143; any of the level's would do)
        if let (Some((g, _)), Some(key)) = (level.terrain.first(), level.lights.iter().find(|l| l.indoor).or(level.lights.first())) {
            for o in 0..8u8 {
                let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut n) = (0f64, 0f64, 0f64, 0f64, 0f64, 0f64);
                for (p, nn) in g.positions.iter().zip(&g.normals) {
                    let Some(b) = level.baked_light(p[0], p[2], o) else { continue };
                    let l = (-Vec3::from(*nn).dot(key.dir)).max(0.0) as f64;
                    let b = b as f64;
                    sx += l; sy += b; sxx += l * l; syy += b * b; sxy += l * b; n += 1.0;
                }
                let r = (n * sxy - sx * sy) / ((n * sxx - sx * sx).sqrt() * (n * syy - sy * sy).sqrt()).max(1e-9);
                println!("baked light orientation {o}: correlation with N.L {r:.3} over {n} vertices");
            }
        }
        for (k, (g, mask)) in level.terrain.iter().enumerate() {
            let tex = game.material_texture(g.material);
            println!("terrain layer {k}: material h_{:08x} type h_{:08x} texture {:?} (decodes: {}) mask {:?} (decodes: {}), {} triangles",
                     g.material, game.material_type(g.material), tex.map(|t| format!("h_{t:08x}")),
                     tex.is_some_and(|t| game.texture_rgba(t).is_some()), mask.map(|m| format!("h_{m:08x}")),
                     mask.is_some_and(|m| game.texture_rgba(m).is_some()), g.indices.len() / 3);
        }
    }
    println!("{name}: terrain {} layers ({} triangles), {} placed objects ({with_mesh} with a mesh), sky {} geosets, {} cameras",
             level.terrain.len(), level.terrain.iter().map(|(g, _)| g.indices.len() / 3).sum::<usize>(),
             level.objects.len(), level.sky.len(), level.cameras.len());
    // BF_LEVEL_DUMP=1: every placed object with a mesh: element, type, archetype, position,
    // parts, untextured geosets
    if std::env::var("BF_LEVEL_DUMP").is_ok() {
        let tags: std::collections::HashMap<u32, &str> = ["game-object", "weapon-object", "inventory-object", "light-object",
            "effect-object", "start-point", "camera-object", "door-point", "world-point", "cover-point", "sniper-point",
            "spawn-trigger", "anim-trigger", "plate-detector", "blocker"].iter().map(|n| (bf_viewer::bf::hash::h(n), *n)).collect();
        for o in &level.objects {
            let Some(arch) = o.archetype else { continue };
            let p = o.transform.w_axis;
            let (parts, bare) = match bf_viewer::bf::weapon::WeaponModel::load(&game, arch) {
                Ok(m) => (m.parts.len(), m.parts.iter().flat_map(|p| &p.geosets).filter(|g| game.material_texture(g.material).is_none())
                    .map(|g| { print!("[untextured material h_{:08x} type h_{:08x}] ", g.material, game.material_type(g.material)); }).count()),
                Err(_) => (0, 0),
            };
            println!("{:16} type h_{:08x} arch h_{arch:08x} at {:7.2} {:6.2} {:7.2} parts {parts} untextured {bare} det {:.2}",
                     tags.get(&o.tag).copied().unwrap_or("?"), o.kind, p.x, p.y, p.z, o.transform.determinant());
        }
    }
    let bg = level.background;
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: format!("Brute Force level viewer - {name}"), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(bg[0], bg[1], bg[2])))
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 400.0, ..default() })
        .insert_resource(World { game, level, name })
        .init_resource::<Fly>()
        .add_plugins((bf_viewer::ale_fx::plugin, bf_viewer::level_scene::plugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (fly, screenshot, move_test_fx, turn_sky))
        .add_systems(PostUpdate, bf_viewer::level_scene::follow_sky.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}

fn setup(mut commands: Commands, mut world: ResMut<World>, mut fly: ResMut<Fly>, mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
         mut level_materials: ResMut<Assets<bf_viewer::level_scene::LevelMaterial>>,
         mut liquids: ResMut<Assets<bf_viewer::level_scene::LiquidMaterial>>,
         mut buffers: ResMut<Assets<bevy::render::storage::ShaderStorageBuffer>>) {
    let World { game, level, .. } = &mut *world;
    let _doors = spawn_level(&mut commands, game, level, &mut meshes, &mut level_materials, &mut liquids, &mut buffers, &mut images);
    // the power-ups' icons and other idle effects
    let mut fx = bf_viewer::ale_fx::AleAssets::new(&mut meshes);
    let n = bf_viewer::ale_fx::spawn_idle_effects(&mut commands, game, level, &mut fx, &mut images, &mut materials);
    println!("{n} idle effects");
    // the level's placed effect objects: its flies, embers, fires, dust (#258); BF_NO_PLACED_FX
    // leaves them out
    if std::env::var("BF_NO_PLACED_FX").is_err() {
        let placed = bf_viewer::ale_fx::spawn_placed_effects(&mut commands, game, level, &mut fx, &mut images, &mut materials);
        println!("{} placed effect objects", placed.len());
    }
    let test_fx = std::env::var("BF_FX_TEST").ok().and_then(|name| fx.load(game, &mut images, &mut materials, bf_viewer::bf::hash::h(&name)));
    commands.insert_resource(fx);
    // the level's own lights (key, fill, ambient) as the console used them
    bf_viewer::level_scene::spawn_lighting(&mut commands, level);
    // camera, with the level's fog
    let fog = level.fog.map(|(c, start, end)| DistanceFog {
        color: Color::srgb(c[0], c[1], c[2]),
        falloff: FogFalloff::Linear { start, end },
        ..default()
    });
    let mut cam = commands.spawn((Camera3d::default(), Projection::Perspective(PerspectiveProjection { far: 5000.0, ..default() }),
                                  Transform::default(), bf_viewer::level_scene::console_look()));
    if let Some(f) = fog.clone().filter(|_| std::env::var("BF_NO_FOG").is_err()) {
        cam.insert(f);
    }
    let cam = cam.id();
    // start view
    let view: Vec<f32> = std::env::var("BF_VIEW").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default();
    let tr = if view.len() >= 5 {
        fly.yaw = view[3].to_radians();
        fly.pitch = view[4].to_radians();
        Transform::from_xyz(view[0], view[1], view[2]).with_rotation(Quat::from_euler(EulerRot::YXZ, fly.yaw, fly.pitch, 0.0))
    } else {
        let n = std::env::var("BF_LEVEL_CAMERA").ok().and_then(|v| v.parse().ok()).unwrap_or(0usize);
        start_view(level, n, &mut fly)
    };
    commands.entity(cam).insert(tr);
    commands.insert_resource(LevelFog(fog));
    // test hook: BF_FX_TEST=<ALE effect name> runs that effect 4 m ahead of the start view, its
    // +y up and -z (a shot's way) to the right; BF_FX_MOVE=<m/s> sends it that way
    if let Some(fx) = test_fx {
        let at = tr.translation + tr.forward() * 4.0;
        let turn = Transform::default().looking_to(*tr.right(), Vec3::Y).rotation;
        let speed = std::env::var("BF_FX_MOVE").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        commands.spawn((Transform::from_translation(at).with_rotation(turn), Visibility::default(),
                        bf_viewer::ale_fx::AleEffect::new(fx, 0.0, 1), FxMover(speed, at)));
    }
}

/// BF_FX_TEST: the test effect's speed (m/s, along its -z) and where it started (it loops over 6 m).
#[derive(Component)]
struct FxMover(f32, Vec3);

fn move_test_fx(time: Res<Time>, mut q: Query<(&FxMover, &mut Transform)>) {
    for (m, mut tr) in &mut q {
        if m.0 > 0.0 {
            let fwd = tr.forward();
            let d = ((tr.translation - m.1).length() + m.0 * time.delta_secs()) % 6.0;
            tr.translation = m.1 + fwd * d - fwd * 3.0;
        }
    }
}

/// Level camera `n` (1-based; 0 = an overview from above the south edge).
fn start_view(level: &Level, n: usize, fly: &mut Fly) -> Transform {
    let tr = match n.checked_sub(1).and_then(|i| level.cameras.get(i)) {
        Some(m) => Transform::from_matrix(*m),
        None => Transform::from_xyz(0.0, 140.0, 190.0).looking_at(Vec3::new(0.0, 30.0, 20.0), Vec3::Y),
    };
    let (yaw, pitch, _) = tr.rotation.to_euler(EulerRot::YXZ);
    fly.yaw = yaw;
    fly.pitch = pitch;
    tr
}

#[allow(clippy::too_many_arguments)]
fn fly(mut commands: Commands, time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, buttons: Res<ButtonInput<MouseButton>>,
       motion: Res<AccumulatedMouseMotion>, world: Res<World>, mut fly: ResMut<Fly>, level_fog: Option<Res<LevelFog>>,
       mut cam: Query<(Entity, &mut Transform, Has<DistanceFog>), With<Camera3d>>) {
    let Ok((cam_entity, mut tr, fogged)) = cam.single_mut() else { return };
    if keys.just_pressed(KeyCode::KeyF) {
        match (fogged, level_fog.and_then(|f| f.0.clone())) {
            (true, _) => { commands.entity(cam_entity).remove::<DistanceFog>(); }
            (false, Some(f)) => { commands.entity(cam_entity).insert(f); }
            _ => {}
        }
    }
    let digits = [KeyCode::Digit0, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
                  KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9];
    for (n, k) in digits.iter().enumerate() {
        if keys.just_pressed(*k) {
            *tr = start_view(&world.level, n, &mut fly);
        }
    }
    if buttons.pressed(MouseButton::Right) {
        fly.yaw -= motion.delta.x * 0.003;
        fly.pitch = (fly.pitch - motion.delta.y * 0.003).clamp(-1.5, 1.5);
        tr.rotation = Quat::from_euler(EulerRot::YXZ, fly.yaw, fly.pitch, 0.0);
    }
    let speed = if keys.pressed(KeyCode::ShiftLeft) { 60.0 } else { 15.0 } * time.delta_secs();
    let (fwd, right) = (*tr.forward(), *tr.right());
    let mut d = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { d += fwd; }
    if keys.pressed(KeyCode::KeyS) { d -= fwd; }
    if keys.pressed(KeyCode::KeyD) { d += right; }
    if keys.pressed(KeyCode::KeyA) { d -= right; }
    if keys.pressed(KeyCode::KeyE) { d += Vec3::Y; }
    if keys.pressed(KeyCode::KeyQ) { d -= Vec3::Y; }
    tr.translation += d * speed;
    if keys.just_pressed(KeyCode::KeyP) {
        let p = tr.translation;
        println!("{}: BF_VIEW={:.1},{:.1},{:.1},{:.0},{:.0}", world.name, p.x, p.y, p.z, fly.yaw.to_degrees(), fly.pitch.to_degrees());
    }
}

/// The sky's clock on the frame's time, which turns it (level_scene.rs `follow_sky`; the game's
/// FUN_00177ba0 runs from the world step with its dt).
fn turn_sky(time: Res<Time>, mut sky: Query<&mut bf_viewer::level_scene::SkyLayer>) {
    for mut s in &mut sky {
        s.run(time.delta_secs());
    }
}

fn screenshot(mut commands: Commands, mut fly: ResMut<Fly>, mut exit: EventWriter<AppExit>) {
    let Ok(file) = std::env::var("BF_SCREENSHOT") else { return };
    fly.frames += 1;
    if fly.frames == 90 {
        commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(file));
    } else if fly.frames > 110 {
        exit.write(AppExit::Success);
    }
}
