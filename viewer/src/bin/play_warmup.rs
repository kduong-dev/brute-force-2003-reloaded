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
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use super::menu;
use super::{AppState, PAUSE_LOADING};

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
}

/// The cover's state while a map starts: its entity, how many frames and seconds the wait has
/// lasted (the game's clock is held meanwhile, so these are the app's own).
#[derive(Resource, Default)]
struct Cover {
    entity: Option<Entity>,
    frames: u32,
    seconds: f32,
    log: bool,
}

/// Render world, every frame: the pipeline cache's counts into the `Probe` (after the frame's
/// pipeline queue has been processed, `RenderSet::Cleanup`).
fn probe(cache: Res<PipelineCache>, probe: Res<Probe>) {
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

/// The map has started: hold the game's clock and put the cover up (the LOADING screen the front
/// end took down as the map's data arrived), with the ring when the front end's art is there (a
/// map started with `BF_MAP` has no front end: black only).
fn begin(mut commands: Commands, mut cover: ResMut<Cover>, probe: Res<Probe>, assets: Option<Res<menu::Assets2>>,
         window: Query<&Window, With<bevy::window::PrimaryWindow>>) {
    super::set_paused(PAUSE_LOADING, true);
    probe.inner.stable.store(0, Ordering::Relaxed);
    let size = window.single().map(|w| w.size()).unwrap_or(Vec2::new(640.0, 480.0));
    // (the 640 x 480 screen the ring is laid out on, fitted to the window as the front end's
    // UI scale does)
    let k = (size.x / 640.0).min(size.y / 480.0).max(0.01);
    let root = commands.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                      justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                               BackgroundColor(Color::BLACK), GlobalZIndex(COVER_Z), Name::new("loading cover"))).id();
    if let Some(assets) = assets {
        let screen = commands.spawn((Node { width: Val::Px(640.0 * k), height: Val::Px(480.0 * k), ..default() }, ChildOf(root))).id();
        menu::spawn_loading_art(&mut commands, &assets, screen, k);
    }
    cover.entity = Some(root);
    cover.frames = 0;
    cover.seconds = 0.0;
    cover.log = std::env::var("BF_WARMUP_LOG").is_ok();
    if cover.log {
        println!("warmup: cover up");
    }
}

/// Each frame of the wait: the cover comes down once the render world is ready (see the module
/// notes), and the game's clock goes on.
fn wait(mut commands: Commands, mut cover: ResMut<Cover>, probe: Res<Probe>, time: Res<Time<Real>>) {
    let Some(entity) = cover.entity else { return };
    cover.frames += 1;
    cover.seconds += time.delta_secs();
    let p = &probe.inner;
    let ready = cover.frames >= MIN_FRAMES && p.stable.load(Ordering::Relaxed) >= STABLE_FRAMES;
    if ready || cover.seconds >= GIVE_UP {
        if cover.log {
            println!("warmup: cover down after {} frames, {:.2} s ({} pipelines, {} waiting{})", cover.frames, cover.seconds,
                     p.total.load(Ordering::Relaxed), p.waiting.load(Ordering::Relaxed), if ready { "" } else { ", gave up" });
        }
        commands.entity(entity).despawn();
        cover.entity = None;
        super::set_paused(PAUSE_LOADING, false);
    }
}

/// The map ends (or the app leaves it) while the cover is up: it goes and the clock is let go.
fn end(mut commands: Commands, mut cover: ResMut<Cover>) {
    if let Some(e) = cover.entity.take() {
        commands.entity(e).despawn();
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
