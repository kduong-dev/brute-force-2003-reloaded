//! A level's start behind the LOADING screen (#311).
//!
//! What the game does: its LOADING screen hides the load, so the level is whole when it shows
//! (the tutorial and deathmatch takes in todo/ never show an unfinished one). What the demo did:
//! `finish_loading` (play_menu.rs) ends the front end's LOADING as soon as the map's data is read,
//! and the level is spawned on the next frame, but Bevy only draws a mesh once the render
//! pipeline for its material has compiled, and compiles each one on a worker thread the first
//! time a mesh of that kind is queued. For the first level of a session (the compiled pipelines
//! stay for the rest of it) the first ~0.7-1 s showed the character and the HUD, whose pipelines
//! are small and quick, over the sky's flat clear colour with the terrain still missing.
//! BF_CAPTURE hid it: its warm-up waits for the character, so a capture's first frames came late.
//!
//! The fix: keep the LOADING screen up until the render world has nothing waiting to compile.
//! A **cover** (a black full-window node over everything, with the LOADING ring and word as the
//! front end draws them, `menu::spawn_loading_art`) goes up as the map starts (`AppState::Playing`
//! entered) and the game clock is held (`PAUSE_LOADING`: the level's script and the AI start when
//! the screen goes, not behind it). The render world's `probe` counts the pipelines still
//! queued or being compiled (`PipelineCache::waiting_pipelines`) and how many there are in all;
//! the cover goes once the count has been zero and the total unchanged for `STABLE_FRAMES`
//! frames in a row, and at least `MIN_FRAMES` frames after the map started (the level's first
//! materials are queued over its first frames), or after `GIVE_UP` s whatever the count says.
//!
//! Details found in the review rounds:
//!  - The cover is UI, and bevy_ui's own pipeline (`ui_pipeline`) compiles on the first UI node drawn:
//!    from the front end it is there already (its LOADING screen compiled it), but on a start with
//!    no front end before it (`BF_MAP`, `--test`) it compiled while the level was already drawn, so
//!    the cover was missing for 0-14 frames and the level or the sky's blue showed (seen in a
//!    failing run's render-world log: `ui_pipeline: creating` for the first three frames). The probe
//!    reports it (`ui_ready`); until it has compiled the main camera is held inactive (its frames
//!    don't count towards the wait) and the clear colour is black while the cover is up (what the
//!    map's setup chose is put back when it goes): the first frames are black, then the ring.
//!  - The cover holds the clock through `PAUSE_LOADING`, but `read_input` (play.rs) is not stopped
//!    by that bit, only by a menu's: the test hooks' steering and the first turn are worked out
//!    behind the cover as they were behind the capture's warm-up, so the first frames of a capture
//!    equal the ones with no cover (a tutorial `BF_TEST_GOTO` capture: mean luma difference 0.00
//!    in all 14 frames; with the input stopped it was 10.9, 9.9, 7.1, 1.7 ... down to 0 by frame 7).
//!
//! The game's own timing isn't known (no take shows its load past the first LOADING screen), so
//! the three numbers are guesses: a few frames of margin, which the captures in menu.md's
//! "Verified" showed enough (no blue frame after the cover; the first level's wait 0.45-1.2 s, a
//! later level's the 6-frame minimum, 0.17 s).
//!
//! Test hooks: `BF_WARMUP_LOG=1` prints when the cover goes up and comes down and how long the
//! wait was and how many pipelines the level needed; `BF_CAPTURE_EARLY=1` (play.rs `capture`)
//! captures from the map's first frame instead of after the warm-up, so the cover shows in
//! the frames; without it a capture starts once the cover has gone.

use bevy::prelude::*;
use bevy::render::{render_resource::PipelineCache, Render, RenderApp, RenderSet};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use super::menu;
use super::{AppState, MainCamera, PAUSE_LOADING};

/// Frames the cover stays up at least once the map has started: the level's entities reach the
/// render world on the first, and the pipelines for them are queued over the next few. A guess
/// (not measured past the first runs' frames).
const MIN_FRAMES: u32 = 6;
/// Frames in a row with nothing compiling and no new pipeline before the level counts as ready.
/// A guess: it only has to outlast the frames between queueing a pipeline and the next one.
const STABLE_FRAMES: u32 = 4;
/// Seconds after which the cover goes whatever the render world says (a pipeline that never
/// finishes, a window with no GPU to count): no reason to wait this long is known, so it is long.
const GIVE_UP: f32 = 15.0;
/// The cover's draw order: over every HUD and menu layer (the highest is the console's 200,
/// play_mission.rs `CONSOLE_Z`).
const COVER_Z: i32 = 1000;

/// What the render world tells the main one each frame (both hold the same `Arc`): the pipelines
/// waiting (queued or being compiled), how many there are, and for how many frames in a row
/// nothing was waiting and the total hadn't changed.
#[derive(Resource, Clone, Default)]
struct Probe {
    inner: Arc<ProbeData>,
}

/// See `Probe`.
#[derive(Default)]
struct ProbeData {
    waiting: AtomicU32,
    total: AtomicU32,
    stable: AtomicU32,
    /// the total last frame (the stable count starts over when it changes)
    last_total: AtomicU32,
    /// the UI's pipeline has compiled (the cover can be drawn)
    ui_ready: AtomicBool,
}

/// The cover's state while a map starts: its entity, how many frames and seconds the wait has
/// lasted (the game's clock is held meanwhile, so these are the app's own).
#[derive(Resource, Default)]
struct Cover {
    entity: Option<Entity>,
    frames: u32,
    seconds: f32,
    log: bool,
    /// the clear colour the map's setup chose, put back when the cover goes
    clear: Color,
    /// `clear` has been taken (the first frame of the wait)
    saved: bool,
}

/// Render world, every frame: the pipeline cache's counts into the `Probe` (after the frame's
/// pipeline queue has been processed, `RenderSet::Cleanup`).
fn probe(cache: Res<PipelineCache>, probe: Res<Probe>) {
    // the UI's own pipeline (bevy_ui's, labelled "ui_pipeline") has compiled: until it has, nothing
    // of the cover can be drawn (it compiles on the first UI node drawn, so on a start that
    // comes with no front end before it, `BF_MAP` or `--test`, it is a first-use compile like
    // the level's)
    let ui_ready = cache.pipelines().any(|p| matches!(p.state, bevy::render::render_resource::CachedPipelineState::Ok(_))
        && matches!(&p.descriptor, bevy::render::render_resource::PipelineDescriptor::RenderPipelineDescriptor(d) if d.label.as_deref() == Some("ui_pipeline")));
    probe.inner.ui_ready.store(ui_ready, Ordering::Relaxed);
    let waiting = cache.waiting_pipelines().count() as u32;
    let total = cache.pipelines().count() as u32;
    let p = &probe.inner;
    p.waiting.store(waiting, Ordering::Relaxed);
    p.total.store(total, Ordering::Relaxed);
    let last = p.last_total.swap(total, Ordering::Relaxed);
    if waiting == 0 && last == total {
        p.stable.fetch_add(1, Ordering::Relaxed);
    } else {
        p.stable.store(0, Ordering::Relaxed);
    }
}

/// One cover: black over the window with the LOADING ring and word (when the front end's art is
/// there), drawn through the default UI camera (the heat camera).
fn spawn_cover(commands: &mut Commands, assets: Option<&menu::Assets2>, k: f32) -> Entity {
    let root = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                      justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                               BackgroundColor(Color::BLACK), GlobalZIndex(COVER_Z), Name::new("loading cover"))).id();
    if let Some(assets) = assets {
        let screen = commands.spawn((Node { width: Val::Px(640.0 * k), height: Val::Px(480.0 * k), ..default() }, ChildOf(root))).id();
        menu::spawn_loading_art(commands, assets, screen, k);
    }
    root
}

/// The map has started: hold the game's clock and put the cover up (the LOADING screen the front
/// end took down as the map's data arrived), with the ring when the front end's art is there (a
/// map started with `BF_MAP` installs the front end's art too, `play.rs`: the ring is there).
fn begin(mut commands: Commands, mut cover: ResMut<Cover>, probe: Res<Probe>, assets: Option<Res<menu::Assets2>>,
         window: Query<&Window, With<bevy::window::PrimaryWindow>>) {
    super::set_paused(PAUSE_LOADING, true);
    probe.inner.stable.store(0, Ordering::Relaxed);
    let size = window.single().map(|w| w.size()).unwrap_or(Vec2::new(640.0, 480.0));
    // (the 640 x 480 screen the ring is laid out on, fitted to the window as the front end's
    // UI scale does)
    let k = (size.x / 640.0).min(size.y / 480.0).max(0.01);
    let root = spawn_cover(&mut commands, assets.as_deref(), k);
    cover.entity = Some(root);
    cover.saved = false;
    cover.frames = 0;
    cover.seconds = 0.0;
    cover.log = std::env::var("BF_WARMUP_LOG").is_ok();
    if cover.log {
        println!("warmup: cover up");
    }
}

/// Each frame of the wait: the cover comes down once the render world is ready (see the module
/// notes), and the game's clock goes on.
fn wait(mut commands: Commands, mut cover: ResMut<Cover>, probe: Res<Probe>, time: Res<Time<Real>>, mut clear: ResMut<ClearColor>,
        mut main: Query<&mut Camera, With<MainCamera>>) {
    let Some(entity) = cover.entity else { return };
    // the clear colour black while the cover is up, the map's own kept for when it goes (and
    // followed if it changes)
    if !cover.saved || clear.0 != Color::BLACK {
        cover.clear = clear.0;
        cover.saved = true;
        clear.0 = Color::BLACK;
    }
    // the level isn't drawn until the UI's own pipeline has compiled, since it compiles on the first
    // UI node drawn: on a start with no front end before it (`BF_MAP`, `--test`) the cover itself
    // was missing for 1-9 frames and the level or the sky's blue showed (#311 tester); the main
    // camera waits (its frames, and the count of them below, from the frame the UI can be drawn),
    // so the first frames are the black clear colour, then the ring. From the front end the
    // pipeline is there already (its LOADING screen compiled it): no wait.
    let ui_ready = probe.inner.ui_ready.load(Ordering::Relaxed);
    for mut camera in &mut main {
        if camera.is_active != ui_ready {
            camera.is_active = ui_ready;
        }
    }
    cover.seconds += time.delta_secs();
    if ui_ready {
        cover.frames += 1;
    }
    let p = &probe.inner;
    let ready = ui_ready && cover.frames >= MIN_FRAMES && p.stable.load(Ordering::Relaxed) >= STABLE_FRAMES;
    if ready || cover.seconds >= GIVE_UP {
        if cover.log {
            println!("warmup: cover down after {} frames, {:.2} s ({} pipelines, {} waiting{})", cover.frames, cover.seconds,
                     p.total.load(Ordering::Relaxed), p.waiting.load(Ordering::Relaxed), if ready { "" } else { ", gave up" });
        }
        commands.entity(entity).try_despawn();
        for mut camera in &mut main {
            camera.is_active = true;
        }
        cover.entity = None;
        clear.0 = cover.clear;
        super::set_paused(PAUSE_LOADING, false);
    }
}

/// The map ends (or the app leaves it) while the cover is up: it goes and the clock is let go.
fn end(mut commands: Commands, mut cover: ResMut<Cover>, mut clear: ResMut<ClearColor>, mut main: Query<&mut Camera, With<MainCamera>>) {
    if let Some(e) = cover.entity.take() {
        for mut camera in &mut main {
            camera.is_active = true;
        }
        // (try: the map's teardown may have taken it already)
        commands.entity(e).try_despawn();
        clear.0 = cover.clear;
    }
    super::set_paused(PAUSE_LOADING, false);
}

/// The plugin: the probe in the render world, the cover on entering and leaving a map.
pub fn plugin(app: &mut App) {
    let shared = Probe::default();
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.insert_resource(shared.clone()).add_systems(Render, probe.in_set(RenderSet::Cleanup));
    }
    app.insert_resource(shared)
        .init_resource::<Cover>()
        .add_systems(OnEnter(AppState::Playing), begin)
        .add_systems(Update, (menu::animate_ring, wait).run_if(in_state(AppState::Playing)))
        .add_systems(OnExit(AppState::Playing), end);
}
