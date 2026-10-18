//! The level's placed effect objects (#258; viewer/docs/maps.md "Placed effect objects"): the
//! ambient particles a level places as effect objects (object-type 17), started with the map
//! (ale_fx `spawn_placed_effects`: the tutorial's 18 swarms of flies, 7 of butterflies and the
//! lava yard's 3 clouds of embers; other levels' fires, torches, steam and dust storms), and
//! taken away as the game takes them:
//!  * by a destroyer-trigger that names one (e34's six, e40's io tablet glow ...): the trigger
//!    runtime (play_triggers.rs) holds them with its copies, by name;
//!  * with the breakable object one is attached to (a `<parent Type="1">` link: e01's barrel
//!    fires on its barrels), once that object breaks (FUN_0021a700).
//!
//! Test hooks: `BF_NO_PLACED_FX=1` leaves them out; `BF_PLACED_FX_VARIANTS=1` starts the ones
//! held out (ale_fx `PLACED_VARIANTS`); `BF_LEVEL_DUMP=1` lists each one started or held out.

use super::*;

/// The placed effect objects started on this map, by name (`spawn`): play_triggers.rs takes
/// their entities into its list of effects a destroyer removes (`Triggers::effects`), and
/// `parents_gone` takes away those whose breakable object has broken.
#[derive(Resource, Default)]
pub struct PlacedEffects(pub Vec<Placed>);

/// A placed effect object started: its name, its entities (ale_fx `PlacedEffect`) and the
/// breakable object it's attached to, if any (`Arena::breakable` of its parent).
pub struct Placed {
    pub name: u32,
    pub entities: Vec<Entity>,
    pub breakable: Option<usize>,
}

/// The placed effects' resource (filled by play.rs `setup`, `spawn`) and the system that takes
/// those on a broken object away, while playing.
pub fn plugin(app: &mut App) {
    app.init_resource::<PlacedEffects>()
        .add_systems(Update, parents_gone.run_if(in_state(AppState::Playing)));
}

/// Start the map's placed effect objects (play.rs `setup`), unless BF_NO_PLACED_FX is set.
/// Returns them, with the breakable object each is attached to (found by its parent's name
/// among the level's objects).
pub fn spawn(commands: &mut Commands, game: &mut Game, level: &Level, ale: &mut bf_viewer::ale_fx::AleAssets,
             images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> PlacedEffects {
    if std::env::var("BF_NO_PLACED_FX").is_ok() {
        return PlacedEffects::default();
    }
    let started = bf_viewer::ale_fx::spawn_placed_effects(commands, game, level, ale, images, materials);
    let arena = world::arena();
    let breakable = |parent: u32| (parent != 0).then(|| level.objects.iter().position(|o| o.name == parent)).flatten()
        .and_then(|i| arena.as_ref().and_then(|a| a.breakable(i)));
    PlacedEffects(started.into_iter().map(|p| Placed { breakable: breakable(p.parent), name: p.name, entities: p.entities }).collect())
}

/// FUN_0021a700: an effect object attached to an object (FUN_0015ed30's link type 1) goes when
/// that object has gone (its link's object cleared), unless the effect object's +0x10c bit 1
/// keeps it (what sets that bit isn't traced; the instance's set-up 0x21a180 copies only bit 0
/// from its type). Here: when the breakable object it's attached to breaks
/// (`Arena::is_broken`, which a destroyer that removes the object sets too). The game takes the
/// object away at the end of its break's countdown (FUN_0015af20: its debris's longest delay, 0
/// for e01's fire barrels), the fire with it; this takes the fire at the break.
fn parents_gone(mut commands: Commands, mut placed: ResMut<PlacedEffects>) {
    let Some(arena) = world::arena() else { return };
    for p in placed.0.iter_mut().filter(|p| !p.entities.is_empty()) {
        if p.breakable.is_some_and(|b| arena.is_broken(b)) {
            for e in p.entities.drain(..) {
                commands.entity(e).try_despawn();
            }
        }
    }
}
