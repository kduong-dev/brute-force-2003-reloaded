//! Brute Force character viewer.
//!
//! Loads the characters exported by `char_render.py gltf` (decompiled/characters/*.glb)
//! and plays their animations.
//!
//! Controls:
//!   1-4 / Tab        switch character (Brutus, Flint, Hawk, Tex)
//!   Right / Left     next / previous animation
//!   PageDown / PageUp  jump 10 animations
//!   Space            pause / resume
//!   [ / ]            slower / faster
//!   Left-drag        orbit camera, mouse wheel zooms, F re-frames
//!   G                toggle ground plane

use bevy::{
    gltf::Gltf,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

const CHARACTERS: [&str; 4] = ["brutus", "flint", "hawk", "tex"];

#[derive(Resource)]
struct Viewer {
    character: usize,
    gltf: Handle<Gltf>,
    scene: Option<Entity>,
    /// (animation name, clip) sorted by name; names start with the export index ("020_h_...")
    clips: Vec<String>,
    graph: Option<Handle<AnimationGraph>>,
    nodes: Vec<AnimationNodeIndex>,
    anim: usize,
    paused: bool,
    speed: f32,
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct Ground;

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
        let pos = self.target + rot * Vec3::new(0.0, 0.0, self.distance);
        Transform::from_translation(pos).looking_at(self.target, Vec3::Y)
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        // characters face -Z in the exported files, so look from that side (front three-quarter)
        Self { yaw: std::f32::consts::PI + 0.45, pitch: -0.12, distance: 3.6, target: Vec3::new(0.0, -0.1, 0.0) }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // relative to this crate when run with `cargo run`
            file_path: "../decompiled/characters".into(),
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window { title: "Brute Force character viewer".into(), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.12, 0.13, 0.16)))
        .insert_resource(AmbientLight { brightness: 600.0, ..default() })
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_when_loaded, attach_player, no_culling, controls, orbit_camera, update_hud, auto_screenshot))
        .run();
}

/// Test hook: with BF_SCREENSHOT=<file.png> set, save a screenshot ~2 s after the character
/// starts animating, then quit. (BF_CHARACTER=0-3 and BF_ANIM=<n> pick what is shown.)
fn auto_screenshot(
    mut commands: Commands,
    time: Res<Time>,
    mut viewer: ResMut<Viewer>,
    mut players: Query<&mut AnimationPlayer>,
    mut state: Local<(Option<f32>, bool, bool)>,
    mut exit: EventWriter<AppExit>,
) {
    let Ok(path) = std::env::var("BF_SCREENSHOT") else { return };
    let (start, configured, shot) = &mut *state;
    if !*configured {
        if viewer.nodes.is_empty() || players.is_empty() {
            return;
        }
        viewer.anim = std::env::var("BF_ANIM").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
        for mut p in &mut players {
            play(&viewer, &mut p);
        }
        *configured = true;
        *start = Some(time.elapsed_secs());
        return;
    }
    let t = time.elapsed_secs() - start.unwrap_or(0.0);
    if !*shot && t > 2.0 {
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path));
        *shot = true;
    } else if *shot && t > 3.5 {
        exit.write(AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cam = OrbitCamera::default();
    commands.spawn((Camera3d::default(), cam.transform(), cam));
    commands.spawn((
        DirectionalLight { illuminance: 6000.0, shadows_enabled: true, ..default() },
        Transform::from_xyz(2.0, 4.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.27, 0.3),
            perceptual_roughness: 1.0,
            ..default()
        })),
        // character origins sit at the pelvis; feet are about 1 unit below
        Transform::from_xyz(0.0, -1.05, 0.0),
    ));
    commands.spawn((
        Hud,
        Text::new("loading..."),
        TextFont { font_size: 16.0, ..default() },
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(12.0), ..default() },
    ));
    // BF_CHARACTER=0-3 selects the starting character (also used by the screenshot test hook)
    let start: usize = std::env::var("BF_CHARACTER").ok().and_then(|s| s.parse().ok()).unwrap_or(0)
        % CHARACTERS.len();
    commands.insert_resource(Viewer {
        character: start,
        gltf: assets.load(format!("{}.glb", CHARACTERS[start])),
        scene: None,
        clips: vec![],
        graph: None,
        nodes: vec![],
        anim: 0,
        paused: false,
        speed: 1.0,
    });
}

/// Once the current .glb has loaded, spawn its scene and build an animation graph of all clips.
fn spawn_when_loaded(
    mut commands: Commands,
    mut viewer: ResMut<Viewer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    if viewer.scene.is_some() {
        return;
    }
    let Some(gltf) = gltfs.get(&viewer.gltf) else { return };
    let mut named: Vec<(String, Handle<AnimationClip>)> =
        gltf.named_animations.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    named.sort_by(|a, b| a.0.cmp(&b.0));
    let (graph, nodes) = AnimationGraph::from_clips(named.iter().map(|(_, h)| h.clone()));
    viewer.clips = named.into_iter().map(|(n, _)| n).collect();
    viewer.nodes = nodes;
    viewer.graph = Some(graphs.add(graph));
    viewer.anim = viewer.anim.min(viewer.clips.len().saturating_sub(1));
    let scene = commands.spawn(SceneRoot(gltf.scenes[0].clone())).id();
    viewer.scene = Some(scene);
}

/// Skinned meshes are culled by their bind-pose bounding box, which root motion leaves behind.
fn no_culling(mut commands: Commands, skinned: Query<Entity, Added<bevy::render::mesh::skinning::SkinnedMesh>>) {
    for e in &skinned {
        commands.entity(e).insert(bevy::render::view::NoFrustumCulling);
    }
}

/// The glTF scene creates its AnimationPlayer when spawned; give it our graph and start playing.
fn attach_player(
    mut commands: Commands,
    viewer: Res<Viewer>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let Some(graph) = viewer.graph.clone() else { continue };
        commands.entity(entity).insert(AnimationGraphHandle(graph));
        play(&viewer, &mut player);
    }
}

fn play(viewer: &Viewer, player: &mut AnimationPlayer) {
    player.stop_all();
    if let Some(&node) = viewer.nodes.get(viewer.anim) {
        player.play(node).repeat().set_speed(viewer.speed);
    }
    if viewer.paused {
        player.pause_all();
    }
}

fn controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    assets: Res<AssetServer>,
    mut viewer: ResMut<Viewer>,
    mut players: Query<&mut AnimationPlayer>,
    mut camera: Query<&mut OrbitCamera>,
    mut ground: Query<&mut Visibility, With<Ground>>,
) {
    // character switching
    let mut next_char = None;
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
        if keys.just_pressed(*key) {
            next_char = Some(i);
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        next_char = Some((viewer.character + 1) % CHARACTERS.len());
    }
    if let Some(c) = next_char {
        if c != viewer.character || viewer.scene.is_none() {
            if let Some(scene) = viewer.scene.take() {
                commands.entity(scene).despawn();
            }
            viewer.character = c;
            viewer.gltf = assets.load(format!("{}.glb", CHARACTERS[c]));
            viewer.graph = None;
            viewer.nodes.clear();
            viewer.clips.clear();
            viewer.anim = 0;
        }
        return;
    }

    // animation switching
    let n = viewer.nodes.len();
    if n > 0 {
        let mut step: isize = 0;
        if keys.just_pressed(KeyCode::ArrowRight) { step = 1; }
        if keys.just_pressed(KeyCode::ArrowLeft) { step = -1; }
        if keys.just_pressed(KeyCode::PageDown) { step = 10; }
        if keys.just_pressed(KeyCode::PageUp) { step = -10; }
        let mut changed = step != 0;
        if changed {
            viewer.anim = (viewer.anim as isize + step).rem_euclid(n as isize) as usize;
        }
        if keys.just_pressed(KeyCode::Space) {
            viewer.paused = !viewer.paused;
            changed = true;
        }
        if keys.just_pressed(KeyCode::BracketLeft) {
            viewer.speed = (viewer.speed * 0.5).max(0.0625);
            changed = true;
        }
        if keys.just_pressed(KeyCode::BracketRight) {
            viewer.speed = (viewer.speed * 2.0).min(4.0);
            changed = true;
        }
        if changed {
            for mut player in &mut players {
                if step != 0 {
                    play(&viewer, &mut player);
                } else {
                    if viewer.paused { player.pause_all(); } else { player.resume_all(); }
                    for (_, active) in player.playing_animations_mut() {
                        active.set_speed(viewer.speed);
                    }
                }
            }
        }
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

fn orbit_camera(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    bones: Query<(&Name, &GlobalTransform)>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    // follow the skeleton root horizontally so animations with root motion stay in frame
    let root = bones.iter().find(|(n, _)| n.as_str() == "root").map(|(_, t)| t.translation());
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

fn update_hud(viewer: Res<Viewer>, mut hud: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = hud.single_mut() else { return };
    let name = CHARACTERS[viewer.character];
    let status = if viewer.clips.is_empty() {
        format!("{name}: loading...")
    } else {
        format!(
            "{name}  [{}/{}]  {}{}  speed x{}\n\
             1-4/Tab character   Left/Right animation   PgUp/PgDn +-10   Space pause   [ ] speed\n\
             drag orbit   wheel zoom   F reset camera   G ground",
            viewer.anim + 1,
            viewer.clips.len(),
            viewer.clips[viewer.anim],
            if viewer.paused { "  (paused)" } else { "" },
            viewer.speed,
        )
    };
    if text.0 != status {
        text.0 = status;
    }
}
