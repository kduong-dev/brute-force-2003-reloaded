//! Brute Force character viewer, reading the ORIGINAL game files (Brute Force/data/*.tgz).
//!
//!   cargo run                          open the viewer
//!   cargo run -- --dump brutus 20 0.5  print bone positions (for checking against char_render.py)
//!
//! Controls:
//!   1-4 / Tab          switch character (Brutus, Flint, Hawk, Tex)
//!   Right / Left       next / previous animation; PageDown / PageUp jump 10
//!   Space              pause / resume;  [ / ] slower / faster
//!   Q / E              previous / next facial pose (lip-sync / expression clips)
//!   L                  toggle game shading (specular mask + 3-point light) vs flat
//!   Left-drag, wheel   orbit, zoom;  F reset camera;  G ground
//!
//! Data directory: $BF_DATA_DIR, else ../Brute Force/data next to this crate.


use std::path::PathBuf;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::mesh::skinning::SkinnedMeshInverseBindposes,
};
use bf_viewer::bf::character::{Character, Game};
use bf_viewer::scene::{spawn_model, ModelAssets, RootBone};

const CHARACTERS: [&str; 4] = ["brutus", "flint", "hawk", "tex"];

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let t0 = std::time::Instant::now();
    let game = match Game::load(&data_dir()) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("failed to load game data from {}: {e}", data_dir().display());
            std::process::exit(1);
        }
    };
    println!("loaded common.tgz in {:.2}s: {} characters, {} animation channels",
             t0.elapsed().as_secs_f32(), game.characters.len(), game.channels.len());

    if args.get(1).map(String::as_str) == Some("--dump") {
        dump(game, &args[2..]);
        return;
    }

    let start = std::env::var("BF_CHARACTER").ok().and_then(|s| s.parse().ok()).unwrap_or(0usize) % CHARACTERS.len();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Brute Force character viewer (original data)".into(), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.12, 0.13, 0.16)))
        .insert_resource(AmbientLight { brightness: 120.0, ..default() })
        .insert_resource(GameData(game))
        .insert_resource(Viewer { character: start, anim: 0, time: 0.0, face: None, face_time: 0.0, paused: false, speed: 1.0,
                                 shading: std::env::var("BF_FLAT").is_err(), spawned: None })
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_character, controls, animate, apply_lighting, orbit_camera, update_hud, auto_screenshot).chain())
        .run();
}

/// `--dump <character> [anim] [time]`: bone count, mesh stats, and model-space bone positions.
fn dump(game: Game, args: &[String]) {
    let name = args.first().map(String::as_str).unwrap_or("brutus");
    let anim: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let t: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let ch = Character::load(&game, name, 0).unwrap_or_else(|e| panic!("{e}"));
    let tris: usize = ch.geosets.iter().map(|g| g.indices.len() / 3).sum();
    println!("{name}: {} bones, {} geosets, {tris} triangles, {} animations", ch.bones.len(), ch.geosets.len(), ch.anims.len());
    let bind = ch.world(&ch.bind_local());
    let err = bind.iter().zip(&ch.inverse_bind).map(|(w, ib)| (*w * *ib - Mat4::IDENTITY).abs().to_cols_array()
        .into_iter().fold(0f32, f32::max)).fold(0f32, f32::max);
    println!("bind check: max |world * inverse_bind - I| = {err:.2e}");
    let w = ch.world(&ch.pose(&game, anim, t, ch.default_face, t));
    println!("anim {anim} at t={t}: bone positions (model space)");
    for (i, m) in w.iter().enumerate() {
        let p = m.w_axis;
        println!("{i:3} {:08x} {:10.6} {:10.6} {:10.6}", ch.bones[i], p.x, p.y, p.z);
    }
}

#[derive(Resource)]
struct GameData(Game);

#[derive(Resource)]
struct Viewer {
    character: usize,
    anim: usize,
    time: f32,
    /// face clip index (None = raw bind face); reset to the character's neutral face on load
    face: Option<usize>,
    face_time: f32,
    /// game shading (specular mask, glossiness, 3-point lighting) vs the plain flat look
    shading: bool,
    paused: bool,
    speed: f32,
    spawned: Option<Spawned>,
}

struct Spawned {
    index: usize,
    shading: bool,
    root: Entity,
    joints: Vec<Entity>,
    model: Character,
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct Ground;

/// Lights only used by the game-shading mode.
#[derive(Component)]
struct ExtraLight;

#[derive(Component)]
struct KeyLight;

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    target: Vec3,
}

impl OrbitCamera {
    fn transform(&self) -> Transform {
        let rot = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);
        Transform::from_translation(self.target + rot * Vec3::new(0.0, 0.0, self.distance)).looking_at(self.target, Vec3::Y)
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        // characters face -Z, so look from that side (front three-quarter)
        Self { yaw: std::f32::consts::PI + 0.45, pitch: -0.12, distance: 3.6, target: Vec3::new(0.0, -0.1, 0.0) }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut cam = OrbitCamera::default();
    if let Some(d) = std::env::var("BF_CAMERA_DISTANCE").ok().and_then(|s| s.parse().ok()) {
        cam.distance = d;
    }
    if std::env::var("BF_CLOSEUP").is_ok() {
        // test hook: frame the head (faces sit ~0.75 above the pelvis origin)
        cam.distance = 1.1;
        cam.target.y = 0.72;
        cam.yaw = std::f32::consts::PI + 0.25;
    }
    // test hooks for inspecting from a given angle (radians; yaw relative to the front view)
    let env = |k: &str| std::env::var(k).ok().and_then(|s| s.parse::<f32>().ok());
    if let Some(y) = env("BF_CAMERA_YAW") { cam.yaw = std::f32::consts::PI + y; }
    if let Some(p) = env("BF_CAMERA_PITCH") { cam.pitch = p; }
    if let Some(d) = env("BF_CAMERA_DISTANCE") { cam.distance = d; }
    if let Some(y) = env("BF_CAMERA_TARGET_Y") { cam.target.y = y; }
    commands.spawn((Camera3d::default(), cam.transform(), cam));
    // key: warm, from the front-right and above (characters face -Z), casts the shadows
    commands.spawn((
        KeyLight,
        DirectionalLight { illuminance: 9000.0, color: Color::srgb(1.0, 0.95, 0.88), shadows_enabled: true, ..default() },
        Transform::from_xyz(2.0, 4.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // fill: cool and soft from the front-left; rim: from behind to separate the silhouette
    commands.spawn((
        ExtraLight,
        DirectionalLight { illuminance: 2200.0, color: Color::srgb(0.75, 0.82, 1.0), ..default() },
        Transform::from_xyz(-3.0, 1.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        ExtraLight,
        DirectionalLight { illuminance: 6000.0, color: Color::srgb(1.0, 1.0, 1.0), ..default() },
        Transform::from_xyz(0.5, 2.5, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgb(0.25, 0.27, 0.3), perceptual_roughness: 1.0, ..default() })),
        Transform::from_xyz(0.0, -1.05, 0.0),
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont { font_size: 16.0, ..default() },
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(12.0), ..default() },
    ));
}

/// (Re)build the selected character from the game data when the selection changes.
#[allow(clippy::too_many_arguments)]
fn spawn_character(
    mut commands: Commands,
    mut viewer: ResMut<Viewer>,
    mut game: ResMut<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    if viewer.spawned.as_ref().is_some_and(|s| s.index == viewer.character && s.shading == viewer.shading) {
        return;
    }
    let same_character = viewer.spawned.as_ref().is_some_and(|s| s.index == viewer.character);
    if let Some(old) = viewer.spawned.take() {
        commands.entity(old.root).despawn();
    }
    let name = CHARACTERS[viewer.character];
    let model = match Character::load(&game.0, name, 0) {
        Ok(m) => m,
        Err(e) => {
            error!("{name}: {e}");
            return;
        }
    };
    let root = commands.spawn((Transform::default(), Visibility::default(), Name::new(name.to_string()))).id();
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    let joints = spawn_model(&mut commands, &mut game.0, &model, &mut assets, root, viewer.shading);
    if !same_character {
        viewer.anim = std::env::var("BF_ANIM").ok().and_then(|s| s.parse().ok()).unwrap_or(0).min(model.anims.len().saturating_sub(1));
        viewer.time = 0.0;
        viewer.face = match std::env::var("BF_FACE").as_deref() {
            Ok("none") => None,                                   // test hook: raw bind face
            Ok(n) => n.parse().ok().or(model.default_face),
            Err(_) => model.default_face,
        };
        viewer.face_time = 0.0;
    }
    let (index, shading) = (viewer.character, viewer.shading);
    viewer.spawned = Some(Spawned { index, shading, root, joints, model });
}

/// Drive the bone entities from the animation channels.
fn animate(time: Res<Time>, mut viewer: ResMut<Viewer>, game: Res<GameData>, mut transforms: Query<&mut Transform>) {
    // BF_FRAMES capture mode steps the animation a fixed 1/15 s per frame
    let step = if std::env::var("BF_FRAMES").is_ok() { 1.0 / 15.0 } else { time.delta_secs() };
    let dt = if viewer.paused { 0.0 } else { step * viewer.speed };
    let anim = viewer.anim;
    let Some(sp) = viewer.spawned.as_ref() else { return };
    let duration = sp.model.anims.get(anim).map(|a| a.duration).unwrap_or(1.0).max(1e-3);
    let t = (viewer.time + dt) % duration;
    let face_t = viewer.face_time + dt;
    let pose = sp.model.pose(&game.0, anim, t, viewer.face, face_t);
    for (e, (q, p)) in sp.joints.iter().zip(&pose) {
        if let Ok(mut tr) = transforms.get_mut(*e) {
            tr.rotation = *q;
            tr.translation = *p;
        }
    }
    viewer.time = t;
    viewer.face_time = face_t;
}

fn controls(keys: Res<ButtonInput<KeyCode>>, mut viewer: ResMut<Viewer>, mut camera: Query<&mut OrbitCamera>,
            mut ground: Query<&mut Visibility, With<Ground>>) {
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
        if keys.just_pressed(*key) {
            viewer.character = i;
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        viewer.character = (viewer.character + 1) % CHARACTERS.len();
    }
    let n = viewer.spawned.as_ref().map(|s| s.model.anims.len()).unwrap_or(0);
    if n > 0 {
        let step: isize = [(KeyCode::ArrowRight, 1), (KeyCode::ArrowLeft, -1), (KeyCode::PageDown, 10), (KeyCode::PageUp, -10)]
            .iter().filter(|(k, _)| keys.just_pressed(*k)).map(|(_, s)| *s).sum();
        if step != 0 {
            viewer.anim = (viewer.anim as isize + step).rem_euclid(n as isize) as usize;
            viewer.time = 0.0;
        }
    }
    let faces = viewer.spawned.as_ref().map(|s| s.model.face_anims.len()).unwrap_or(0);
    if faces > 0 {
        // cycle through "none" (raw bind face) and every face clip
        let cur = viewer.face.map(|f| f as isize).unwrap_or(-1);
        let step = keys.just_pressed(KeyCode::KeyE) as isize - keys.just_pressed(KeyCode::KeyQ) as isize;
        if step != 0 {
            let next = (cur + 1 + step).rem_euclid(faces as isize + 1) - 1;
            viewer.face = (next >= 0).then_some(next as usize);
            viewer.face_time = 0.0;
        }
    }
    if keys.just_pressed(KeyCode::KeyL) {
        viewer.shading = !viewer.shading;
    }
    if keys.just_pressed(KeyCode::Space) {
        viewer.paused = !viewer.paused;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        viewer.speed = (viewer.speed * 0.5).max(0.0625);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        viewer.speed = (viewer.speed * 2.0).min(4.0);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        for mut cam in &mut camera {
            *cam = OrbitCamera::default();
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        for mut vis in &mut ground {
            vis.toggle_visible_hidden();
        }
    }
}

fn orbit_camera(time: Res<Time>, buttons: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>,
                scroll: Res<AccumulatedMouseScroll>, roots: Query<&GlobalTransform, With<RootBone>>,
                mut cameras: Query<(&mut OrbitCamera, &mut Transform)>) {
    // follow the skeleton root horizontally so root motion stays in frame
    let root = roots.iter().next().map(|t| t.translation());
    for (mut cam, mut transform) in &mut cameras {
        if let Some(r) = root {
            let goal = Vec3::new(r.x, cam.target.y, r.z);
            // follow tightly; snap when a looping clip with root motion jumps back to its start
            if cam.target.distance(goal) > 1.5 {
                cam.target = goal;
            } else {
                let k = 1.0 - (-20.0 * time.delta_secs()).exp();
                cam.target = cam.target.lerp(goal, k);
            }
        }
        if buttons.pressed(MouseButton::Left) {
            cam.yaw -= motion.delta.x * 0.006;
            cam.pitch = (cam.pitch - motion.delta.y * 0.006).clamp(-1.4, 1.4);
        }
        if scroll.delta.y != 0.0 {
            cam.distance = (cam.distance * (1.0 - scroll.delta.y * 0.1)).clamp(0.8, 20.0);
        }
        *transform = cam.transform();
    }
}

/// Game shading uses a dim ambient plus key / fill / rim lights; flat mode is the old look
/// (bright ambient, one light).
fn apply_lighting(viewer: Res<Viewer>, mut ambient: ResMut<AmbientLight>,
                  mut extra: Query<&mut Visibility, With<ExtraLight>>, mut key: Query<&mut DirectionalLight, With<KeyLight>>) {
    if !viewer.is_changed() {
        return;
    }
    let (amb, key_lux, vis) = if viewer.shading { (120.0, 9000.0, Visibility::Inherited) } else { (600.0, 6000.0, Visibility::Hidden) };
    if ambient.brightness != amb {
        ambient.brightness = amb;
    }
    for mut v in &mut extra {
        *v = vis;
    }
    for mut l in &mut key {
        l.illuminance = key_lux;
    }
}

fn update_hud(viewer: Res<Viewer>, mut hud: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = hud.single_mut() else { return };
    if std::env::var("BF_NO_HUD").is_ok() {
        text.0.clear();
        return;
    }
    let name = CHARACTERS[viewer.character];
    let status = match &viewer.spawned {
        Some(sp) if !sp.model.anims.is_empty() => {
            let a = &sp.model.anims[viewer.anim];
            let face = match viewer.face {
                Some(f) => format!("face {}/{}{}", f + 1, sp.model.face_anims.len(),
                                   if Some(f) == sp.model.default_face { " (neutral)" } else { "" }),
                None => "face: raw bind".into(),
            };
            format!(
                "{name}  [{}/{}]  h_{:08x}  {:.2}/{:.2}s{}  speed x{}   {face}   (original data files)\n\
                 1-4/Tab character   Left/Right animation   PgUp/PgDn +-10   Q/E face   L shading   Space pause   [ ] speed\n\
                 drag orbit   wheel zoom   F reset camera   G ground",
                viewer.anim + 1, sp.model.anims.len(), a.name, viewer.time, a.duration,
                if viewer.paused { "  (paused)" } else { "" }, viewer.speed)
        }
        _ => format!("{name}: loading..."),
    };
    if text.0 != status {
        text.0 = status;
    }
}

/// Test hooks (also: BF_NO_HUD hides the overlay, BF_CAMERA_DISTANCE sets the start distance,
/// BF_CLOSEUP frames the head). BF_SCREENSHOT=<file.png> saves a screenshot 2 s after the character appears, then
/// quits. With BF_FRAMES=<n> it instead captures n consecutive frames <file>_000.png ... with the
/// animation stepped exactly 1/15 s per frame (for making GIFs), starting from the clip's start.
fn auto_screenshot(mut commands: Commands, time: Res<Time>, mut viewer: ResMut<Viewer>,
                   mut state: Local<(Option<f32>, bool, usize)>, mut exit: EventWriter<AppExit>) {
    let Ok(path) = std::env::var("BF_SCREENSHOT") else { return };
    if viewer.spawned.is_none() {
        return;
    }
    let frames: usize = std::env::var("BF_FRAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let (start, shot, captured) = &mut *state;
    let t = time.elapsed_secs() - *start.get_or_insert(time.elapsed_secs());
    let save = |commands: &mut Commands, file: String| {
        commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(file));
    };
    if frames > 0 {
        if t < 1.0 {
            return;                                   // let textures and shadows settle
        }
        if *captured == 0 && !*shot {
            // reset the clip this frame, start saving from the next one so frame i is at i/15 s
            viewer.time = 0.0;
            viewer.face_time = 0.0;
            *shot = true;
            return;
        }
        if *captured < frames {
            let stem = path.trim_end_matches(".png");
            save(&mut commands, format!("{stem}_{:03}.png", *captured));
            *captured += 1;
            *start = Some(time.elapsed_secs() - 1.0); // keep the exit timer behind the capture
        } else if t > 2.5 {
            exit.write(AppExit::Success);
        }
        return;
    }
    if !*shot && t > 2.0 {
        save(&mut commands, path);
        *shot = true;
    } else if *shot && t > 3.5 {
        exit.write(AppExit::Success);
    }
}
