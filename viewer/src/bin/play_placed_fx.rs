//! The level's placed effect objects (#258; viewer/docs/maps.md "Placed effect objects"): the
//! ambient particles a level places as effect objects (object-type 17), started with the map
//! (ale_fx `spawn_placed_effects`: the tutorial's 18 swarms of flies, 7 of butterflies and the
//! lava yard's 3 clouds of embers; other levels' fires, torches, steam and dust storms), and
//! taken away as the game takes them:
//!  * by a destroyer-trigger that names one (e34's six, e40's io tablet glow ...): the trigger
//!    runtime (play_triggers.rs) holds them with its copies, by name;
//!  * with the breakable object one is attached to (a `<parent Type="1">` link: e01's barrel
//!    fires on its barrels), once that object has gone (FUN_0021a700);
//!  * stopped when the object whose hardpoint one sits at has gone (a `<parent Type="2">` link:
//!    the torches and sconces, FUN_0015a1d0), #308.
//!
//! Test hooks: `BF_NO_PLACED_FX=1` leaves them out; `BF_PLACED_FX_VARIANTS=1` starts the ones
//! held out (ale_fx `PLACED_VARIANTS`); `BF_LEVEL_DUMP=1` lists each one started or held out;
//! `BF_LINK_LOG=1` logs each one taken away or stopped with its object, and each lamp put out
//! (play_lamps.rs); with `BF_LINK_AT=<x>,<z>` also, ten times a second, the point lights alive
//! within 3 m of that place and their summed intensity (the ALE light effects' particles among
//! them), to see the lights go with their object.

use super::*;
use bf_viewer::ale_fx::{AleEffect, StartAfter, TakenAway};

/// The placed effect objects started on this map, by name (`spawn`): play_triggers.rs takes
/// their entities into its list of effects a destroyer removes (`Triggers::effects`), and
/// `parents_gone` takes away those whose breakable object has gone.
#[derive(Resource, Default)]
pub struct PlacedEffects(pub Vec<Placed>);

/// A placed effect object started: its name, its entities (ale_fx `PlacedEffect`), the breakable
/// object it's attached to (`Arena::breakable` of its parent) and the one whose hardpoint it sits
/// at (of its holder), if any.
pub struct Placed {
    pub name: u32,
    pub entities: Vec<Entity>,
    pub breakable: Option<usize>,
    pub held_by: Option<usize>,
}

/// The placed effects' resource (filled by play.rs `setup`, `spawn`) and the system that takes
/// those on a gone object away, while playing.
pub fn plugin(app: &mut App) {
    app.init_resource::<PlacedEffects>()
        .add_systems(Update, parents_gone.after(super::scenery::SceneryStep).run_if(in_state(AppState::Playing)))
        .add_systems(Update, lights_log.run_if(in_state(AppState::Playing)));
}

/// How near (m, across the ground) a point light is to BF_LINK_AT's place to be counted.
const LIGHT_LOG_NEAR: f32 = 3.0;

/// BF_LINK_LOG with BF_LINK_AT=x,z: ten times a second, how many point lights are alive (the ALE
/// light effects' particles among them) within LIGHT_LOG_NEAR of that place and their summed
/// intensity, to see them go with their object.
fn lights_log(player: Res<Player>, lights: Query<(&PointLight, &GlobalTransform)>, mut last: Local<f32>) {
    if std::env::var("BF_LINK_LOG").is_err() {
        return;
    }
    let Some(at) = std::env::var("BF_LINK_AT").ok().and_then(|v| {
        let f: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        (f.len() >= 2).then(|| Vec2::new(f[0], f[1]))
    }) else { return };
    let now = player.sim_time;
    if (now * 10.0).floor() != (*last * 10.0).floor() {
        let near: Vec<_> = lights.iter().filter(|(_, g)| g.translation().xz().distance(at) <= LIGHT_LOG_NEAR).collect();
        println!("t {now:.2}: {} point lights near, intensity {:.0}", near.len(), near.iter().fold(0.0, |sum, (l, _)| sum + l.intensity));
    }
    *last = now;
}

/// Start the map's placed effect objects (play.rs `setup`), unless BF_NO_PLACED_FX is set.
/// Returns them, with the breakable object each is attached to or held by (found by its
/// parent's name among the level's objects).
pub fn spawn(commands: &mut Commands, game: &mut Game, level: &Level, ale: &mut bf_viewer::ale_fx::AleAssets,
             images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> PlacedEffects {
    if std::env::var("BF_NO_PLACED_FX").is_ok() {
        return PlacedEffects::default();
    }
    let started = bf_viewer::ale_fx::spawn_placed_effects(commands, game, level, ale, images, materials);
    let arena = world::arena();
    let object = |name: u32| (name != 0).then(|| level.objects.iter().position(|o| o.name == name)).flatten();
    let breakable = |i: usize| arena.as_ref().and_then(|a| a.breakable(i));
    PlacedEffects(started.into_iter().map(|p| Placed {
        breakable: object(p.parent).and_then(breakable),
        // (the holder stops only the effects it lists: FUN_0015a1d0 walks its +0x88 pointers,
        // `Placement::static_effects`)
        held_by: object(p.holder).filter(|&i| level.objects[i].static_effects.contains(&p.name)).and_then(breakable),
        name: p.name,
        entities: p.entities,
    }).collect())
}

/// An effect object whose object has gone (taken out of the world: `Arena::is_gone`, the end of a
/// break's countdown or a destroyer's removal), as the game treats it:
///  * attached by a type 1 link (e01's barrel fires): FUN_0015a1d0, run as the object leaves the
///    world (FUN_001586a0, vtable +0xa0 of FUN_000db3e0), stops its static effects, and the
///    object's destructor clears the pointers to it; the effect's next tick finds its link's
///    object gone, FUN_0021a700 returns 0 (the placed ones' +0x10c bit 1 is clear: set, it
///    detaches instead), and the world destroys it (FUN_0021a4d0 -> FUN_0021c700 with AL 0),
///    every node of its ALE effects and its light effect with it (`TakenAway`): the fire, the
///    smoke and the lights are gone on that frame. The light-object hung on it (its tick,
///    0x14f1d0) goes one tick after. The game takes a frame for each step; here they go
///    together;
///  * on a hardpoint, type 2 (the torches and sconces): FUN_0021a700 acts on a type 1 link only,
///    so nothing destroys the effect, but the stop of FUN_0015a1d0 stays:
///    FUN_0021c700 with AL 1 takes the effect's emitters out of the ALE manager and sets its
///    appearances' time to the end (0x2abcd0 with the mask 1, 0x2ab9a0 with FLT_MAX and the
///    mask 4), so the flame makes no more particles and those out finish their lives, and the
///    light effect likewise: the flame dies down and its light fades with its last particles.
///    The effect object stays in the world, empty. Here its effects go inactive
///    (`AleEffect::active`), which is the same.
///
/// Where a placed effect isn't attached to an object that can go, nothing happens.
fn parents_gone(mut commands: Commands, mut placed: ResMut<PlacedEffects>, player: Res<Player>, mut effects: Query<&mut AleEffect>) {
    let Some(arena) = world::arena() else { return };
    let log = std::env::var("BF_LINK_LOG").is_ok();
    let now = player.sim_time;
    for p in placed.0.iter_mut().filter(|p| !p.entities.is_empty()) {
        if p.breakable.is_some_and(|b| arena.is_gone(b)) {
            if log {
                println!("t {now:.2}: placed effect h_{:08x} ({} effects) taken away with its object", p.name, p.entities.len());
            }
            for e in p.entities.drain(..) {
                commands.entity(e).try_insert(TakenAway);
            }
        } else if p.held_by.is_some_and(|b| arena.is_gone(b)) {
            if log {
                println!("t {now:.2}: placed effect h_{:08x} ({} effects) stopped with the object that holds it", p.name, p.entities.len());
            }
            for &e in &p.entities {
                if let Ok(mut fx) = effects.get_mut(e) {
                    fx.active = false;
                }
                // (one still waiting for its creation-time stays out)
                commands.entity(e).try_remove::<StartAfter>();
            }
            p.held_by = None;
        }
    }
}
