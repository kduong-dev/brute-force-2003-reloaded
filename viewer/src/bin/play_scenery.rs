//! Interactive scenery (issue #85): placed objects that break, explode and hurt, from the game's
//! data and code, checked against the xemu takes in todo/85-interactive-scenery/ (and the first
//! capture, todo/interactive scenery objects.mp4).
//!
//! Which objects: a placed game object whose type has a debris list (h_197caf14, `Game::breakable`;
//! 467 game types have one). On sdm_e34: 23 radiation barrels (h_e04e5a0e, 1 hp), 6 supply
//! crates (h_09a6856d, 25 hp) and a missile rack (h_fbdcd828, 1 hp), among others.
//!
//! Damage (FUN_002232d0, the combat-target's take-damage): value x the type's factor for its
//! damage-type (the combat-target's own h_142be76f list: the three above take Type 7 x5, 10 x10,
//! 2 x0.25, 3 / 4 / 9 x0), off its hitpoints; at 0 it breaks (message 0x4e). What does damage:
//!  - shots (anyone's): the weapon's Damage min..max at random, when the shot gets there (a
//!    bolt flies at its speed), on the object's own collision (`Arena::ray_breakable`);
//!  - grenade blasts: the explosion's Damage max, falling to nothing at its radius, as the
//!    characters take a Frag's, on the same frame (play_grenade.rs hands them over,
//!    `ObjectBlasts`). Every damaging type is dealt so, the Energy and the Sonic too, whose
//!    characters' damage comes with their bolts and ring instead: a simplification;
//!  - the damage areas of other broken objects' effects (chain reactions, see below).
//!
//! Breaking (FUN_00157260 on message 0x4e, then the countdown FUN_0015af20 each frame): the
//! debris list is queued and a countdown starts at its longest h_1b6a0ede; each frame it drops
//! by the frame's time and the first queued entry whose h_1b6a0ede is at least what's left is
//! spawned (FUN_0015ac90; one a frame); on a frame with nothing left to spawn and the countdown
//! at 0 the object goes (FUN_0015ae30). So the longest delay comes first: the missile rack's
//! effect (0.2) at once, its pieces and stand (0) from 0.2 s on, a frame apart, then the model
//! goes; the take's light and fireball come before the model goes (take13: 0.067 s apart; the
//! first capture: ~0.17 s). A barrel: its pieces on the first frame, its effect on the next, the
//! model gone on the third. Its collision and level blocker go at once (a blast from where it stood
//! sees past it), what it leaves behind blocks from then on (`Arena::set_broken`).
//! What each entry spawns, by its type's object-type:
//!  - 0, a compound (archetype-type 9): its parts fly apart as loose tumbling bodies
//!    (play_pickups.rs' rigid bodies) and are gone after DEBRIS_LIFE;
//!  - 2, a game object: it stays, with its collision (the missile rack's stand h_f77cc3a0);
//!  - 17, an effect object: its ALE effects and light effect where the object stood, its
//!    sounds, and its `<Damage>` areas.
//!
//! Damage areas (an effect's `<Damage amount Type duration range falloff>`): from AREA_DELAY
//! after the effect, for `duration` s, everyone within `range` of the object's centre (3D,
//! to their point CENTRE_UP above the feet) loses amount x dt x their factor a frame, with no
//! line-of-sight test: amount x duration in all (the barrel's 80 x 0.5 = 40 HP, the rack's 200 x
//! 0.5 = 100). Breakable objects in range take it too, times their own factor: the rack's
//! explosion (Type 10, x10) sets off barrels 4-5 m away at once, the barrel's (Type 3, x0) does
//! nothing to barrels or crates. This is the effect object's own damage code, FUN_0021ade0: per
//! frame, for each `<Damage>` record (the effect type's +0xe4 vector, 0x1c a record) while
//! h_f724cb8c < the effect's age <= h_f724cb8c + duration, every target near the effect (a grid
//! query of `range`) whose factor for the Type is above 0 and whose hitpoints are above 0, within
//! `range` of the effect's place (3D, to the target's place), takes amount x dt, x (1 - d /
//! range) for falloff 1, (1 - d / range)^2 x (amount x dt)^2 for 2, in full for 0 and 3 (the
//! rack's HALF_LIFE: the takes' flat damage at 4 and 7 m agrees; see `falloff`). It sends no
//! ray: no line of sight (the weapons' damage areas, FUN_00224a90, test one). h_ed582b3c is
//! the push its message carries, scaled by ((1 - w) x 0.4 + 0.6), w the world's load measure at
//! +0xc58 (recomputed every second from counts of active characters and such, FUN_000d9090,
//! and 0 in multiplayer): a push, not the damage, so the campaign's ~0.58 of the
//! damage (see DAMAGE_SCALE) isn't this. Hurt by one, the player's view goes red (ScreenTint)
//! as for a grenade.
//!
//! The test map's object tool (play_testtools.rs) puts these down on the flat floor too
//! (`LateBreakable`): they join the list after the map's, and with no arena their model's box
//! stands in for their collision (`ray_placed`, which play.rs's `ray_hit` stops at).

use super::*;
use super::grenade::{ScreenTint, BLAST_HEARD, BLAST_QUIET};
use bf_viewer::bf::character::{AreaDamage, Debris, OBJECT_COMPOUND, OBJECT_EFFECT, OBJECT_GAME};
use bf_viewer::level_scene::Placed;

/// A damage area starts hurting this long after its effect is spawned: the takes' first HP
/// loss is ~0.1 s after the object's hitpoints reach 0, for the barrel (takes 01-04: 2.04 ->
/// 2.13-2.14 s) and the rack alike (take07). Measured; the same as a grenade's DAMAGE_DELAY.
const AREA_DELAY: f32 = 0.1;
/// A character's point that a damage area measures to (m above the feet): the barrel's range of
/// 3 m from its centre (0.435 m up) reached Tex 2.7 m away along the ground and not 2.9 m (takes
/// 04 / 05), so a point 1.2-1.74 m up. The middle of that: a fit.
const CENTRE_UP: f32 = 1.45;
/// The campaign's squad took ~0.58 of a blast (campaign e34, takes 01-09) where sdm_e34 took
/// the whole (take29: 39.4 of 40). Not traced: the ((1 - w) x 0.4 + 0.6) in FUN_00157260 and
/// FUN_0021ade0 scales the push (h_ed582b3c), not the damage (see the module notes); the
/// characters' own take-damage runs a per-target modifier first (FUN_002232d0's
/// combat-target[6]), not followed. The demo's maps are the squad deathmatch ones: 1.
const DAMAGE_SCALE: f32 = 1.0;
/// How long a broken object's loose pieces stay (s), and the last part of that over which they
/// shrink away. A guess: the takes lose them out of frame (crate panels leave it 0.4-0.6 s
/// after the break, still whole); the first capture's crate pieces are gone after ~0.8 s,
/// fading over ~0.13 s (low confidence).
const DEBRIS_LIFE: f32 = 1.2;
const DEBRIS_SHRINK: f32 = 0.13;
/// A broken object's pieces fly off (m/s): out from where it was hit (DEBRIS_OUT), along the
/// hit (DEBRIS_ALONG) and up (DEBRIS_UP), each up to DEBRIS_JITTER more or less, tumbling at up
/// to DEBRIS_SPIN rad/s. An explosive one's (with a damage area) are thrown out from its centre
/// instead by DEBRIS_BLAST x (1.2 - d / range)^2, the push the area tick gives loose bodies
/// (FUN_00224a90: (1.2 - d / range)^2 x damage x 1.5), and up by DEBRIS_BLAST_UP: take26's
/// barrel pieces go low and fast. Fitted to take25 (the crate's panels go 2-3 m to the
/// side and ~1 m up, out of frame by 0.5 s) and take26 (the barrel's pieces low and fast, out of
/// frame by 0.45 s); not the game's numbers (the push on the pieces is the hit's direction x
/// its damage, FUN_00156e50, on bodies of unknown mass).
const DEBRIS_OUT: f32 = 2.0;
const DEBRIS_ALONG: f32 = 4.0;
const DEBRIS_UP: f32 = 3.0;
const DEBRIS_JITTER: f32 = 0.35;
const DEBRIS_SPIN: f32 = 9.0;
const DEBRIS_BLAST: f32 = 6.0;
const DEBRIS_BLAST_UP: f32 = 1.0;
/// Test hook BF_TEST_HIT's hit: 50 ballistic (damage-type 1), what the takes' gdb call gave the
/// objects through the game's own take-damage (notes.md: `!hit <object> 50 1`).
const TEST_HIT_DAMAGE: f32 = 50.0;
const TEST_HIT_TYPE: i64 = 1;
/// A shot counts as hitting a breakable object when the object's collision is this close to
/// where the shot ended (m).
const SHOT_SLACK: f32 = 0.05;

/// A grenade blast for the scenery (play_grenade.rs adds them when it goes off): it lands after
/// `left` s, with its Damage max at `at` falling to nothing at `radius`, of `damage_type`.
pub struct ObjectBlast {
    pub left: f32,
    pub at: Vec3,
    pub radius: f32,
    pub damage: f32,
    pub damage_type: i64,
    /// the grenade's label (BF_SCENERY_LOG)
    pub label: String,
}

/// Grenade blasts on their way to the scenery.
#[derive(Resource, Default)]
pub struct ObjectBlasts(pub Vec<ObjectBlast>);

/// Where a breaking object was hit and which way (the push its main debris gets, the object's
/// +0xe8..+0x100 that FUN_00156e50 fills from the damage message).
#[derive(Clone, Copy)]
struct Push {
    at: Vec3,
    dir: Vec3,
}

/// A breakable object's state.
enum State {
    Intact,
    /// broken: the countdown (s) and the debris entries not spawned yet
    Breaking { countdown: f32, queue: Vec<Debris> },
    Gone,
}

/// A placed breakable object (in the arena's breakable order).
struct Breakable {
    /// its name, type and placement
    name: u32,
    kind: u32,
    place: Transform,
    /// where the game has it: its origin moved to its archetype's centre (`Game::
    /// archetype_centre`, the physics body's; the barrel's 0.435 m up). Its effects are spawned
    /// and its damage areas measured from here, and other blasts measured to it. An inference:
    /// at the origin, on the ground, the barrel's burst (exp-radio-flash and spray) was half
    /// buried and showed late and faint, where take26's is full on its first frame
    centre: Vec3,
    /// its drawn model (the placed root)
    entity: Option<Entity>,
    hp: f32,
    state: State,
    push: Push,
    /// it explodes (an effect in its debris list has a damage area)
    explosive: bool,
    /// its model's box (its own frame) for shots, when it has no collision in the arena: one
    /// put down by the test map's object tool (`LateBreakable`). None: the arena's, at the same
    /// index (`Arena::ray_breakable`, `Arena::set_broken`).
    bounds: Option<(Vec3, Vec3)>,
}

/// A breakable object put down after the map started (the test map's object tool,
/// play_testtools.rs), its type (a `Game::breakable` key): `late_scenery` adds it to the list
/// with its placed Transform. It has no collision (the flat test floor has no arena): shots
/// meet its model's box instead, characters walk through it and what it leaves behind.
#[derive(Component)]
pub struct LateBreakable(pub u32);

/// A damage area at work (see the module notes).
struct Area {
    /// the breakable it came from, the effect type, where it is (the object's origin)
    source: usize,
    effect: u32,
    at: Vec3,
    damage: AreaDamage,
    /// seconds until it starts, then its age since it started
    wait: f32,
    age: f32,
    /// per character it reached: (character, its age when first and last hurting them, HP)
    victims: Vec<(usize, f32, f32, f32)>,
}

/// A shot on its way to a breakable object: lands after `delay` s.
struct PendingHit {
    delay: f32,
    target: usize,
    amount: f32,
    damage_type: i64,
    push: Push,
}

/// One part of a debris model: its placement in the model, the middle of its meshes (part
/// frame) and its meshes.
struct PieceModel {
    offset: Vec3,
    rotation: Quat,
    middle: Vec3,
    meshes: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
}

/// The map's breakable objects and what's going on with them.
#[derive(Resource, Default)]
pub(super) struct Scenery {
    list: Vec<Breakable>,
    hits: Vec<PendingHit>,
    areas: Vec<Area>,
    /// effect sounds waiting for their delay: (s left, id, where)
    sounds: Vec<(f32, u32, Vec3)>,
    /// debris models by object type (compounds and the game objects left behind)
    models: HashMap<u32, Vec<PieceModel>>,
    ready: bool,
    /// BF_TEST_HIT's hits done so far
    test_hits: usize,
    /// grenade blasts on their way (from `ObjectBlasts`, from the frame after they went off)
    blasts: Vec<ObjectBlast>,
    /// this frame's grenade blasts landing and areas at work: (where, how far, damage-type), for
    /// `set_off_mines`
    reach: Vec<(Vec3, f32, i64)>,
}

/// A loose piece of a broken object: seconds left, and its own scale (it shrinks away at the end).
#[derive(Component)]
struct DebrisPiece {
    left: f32,
    scale: Vec3,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Scenery>()
        .init_resource::<ObjectBlasts>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| {
            commands.insert_resource(Scenery::default());
            PLACED_BOXES.lock().unwrap().clear();
            commands.insert_resource(ObjectBlasts::default());
        }).after(setup))
        // (after the grenades: a blast's damage reaches the scenery on the frame the
        // characters take it)
        // (the mines set off before they're handed to the next frame's shots)
        .add_systems(Update, (find_scenery, late_scenery, hit_scenery, break_scenery, area_damage, set_off_mines, debris_life).chain()
            .after(super::grenade::GrenadeSystems).before(super::sentry::publish_targets).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// The map's breakable objects, once its objects are spawned: their placed models, hitpoints,
/// and their debris models and effects made ready (the effects' render pipelines built ahead,
/// as the grenades' are).
#[allow(clippy::too_many_arguments)]
fn find_scenery(mut commands: Commands, mut scenery: ResMut<Scenery>, mut game: ResMut<GameData>, map: Res<MapLevel>,
                placed: Query<(Entity, &Placed, &Transform)>, mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
                mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>, mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>) {
    if scenery.ready {
        return;
    }
    let (Some(level), Some(arena)) = (map.0.as_ref(), world::arena()) else {
        scenery.ready = true;
        return;
    };
    // (the level's objects are spawned by `setup`; wait for them)
    if placed.is_empty() {
        return;
    }
    let log = std::env::var("BF_SCENERY_LOG").is_ok();
    let mut list = vec![];
    for (i, o) in level.objects.iter().enumerate() {
        let Some(b) = arena.breakable(i) else { continue };
        let Some(t) = game.0.breakable(o.kind).cloned() else { continue };
        let place = Transform::from_matrix(o.transform);
        let entity = placed.iter().filter(|(_, p, tr)| p.kind == o.kind && tr.translation.distance(place.translation) < 0.01)
            .map(|(e, _, _)| e).next();
        let explosive = t.debris.iter().any(|d| game.0.object_types.get(&d.kind).is_some_and(|k| k.object_type == OBJECT_EFFECT)
            && game.0.effect_type_defs.get(&d.kind).is_some_and(|e| !e.damage.is_empty()));
        if log {
            println!("breakable {b}: h_{:08x} type h_{:08x} at {:.2}, {} hp, factors {:?}, debris {:?}{}", o.name, o.kind, place.translation, t.hitpoints,
                     t.factors, t.debris.iter().map(|d| format!("h_{:08x} ({}{}, delay {})", d.kind,
                         game.0.object_types.get(&d.kind).map_or(-1, |k| k.object_type), if d.main { ", main" } else { "" }, d.delay)).collect::<Vec<_>>(),
                     if entity.is_none() { ", no model" } else { "" });
        }
        debug_assert_eq!(b, list.len());
        let centre = place.transform_point(game.0.object_meshes.get(&o.kind).and_then(|&a| game.0.archetype_centre(a)).unwrap_or(Vec3::ZERO));
        list.push(Breakable { name: o.name, kind: o.kind, place, centre, entity, hp: t.hitpoints, state: State::Intact,
                              push: Push { at: centre, dir: Vec3::ZERO }, explosive, bounds: None });
        let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
        prepare_debris(&mut commands, &mut scenery, &mut game.0, &mut assets, ale.as_deref_mut(), &t.debris);
    }
    println!("{} breakable objects", list.len());
    scenery.list = list;
    scenery.ready = true;
}

/// What a breakable object breaks into, made ready: its effects compiled (their render pipelines
/// built ahead) and its pieces' and remains' models.
fn prepare_debris(commands: &mut Commands, scenery: &mut Scenery, game: &mut Game, assets: &mut ModelAssets,
                  ale: Option<&mut bf_viewer::ale_fx::AleAssets>, debris: &[Debris]) {
    let mut ale = ale;
    for d in debris {
        let Some(k) = game.object_types.get(&d.kind).map(|k| k.object_type) else { continue };
        if k == OBJECT_EFFECT {
            if let Some(ale) = ale.as_deref_mut() {
                let fx = game.effect_type_defs.get(&d.kind).cloned().unwrap_or_default();
                for &e in fx.effects.iter().chain([&fx.light]).filter(|&&e| e != 0) {
                    if ale.cached_recorded(e).is_none() {
                        if let Some(c) = ale.load_recorded(game, assets.images, assets.materials, e) {
                            bf_viewer::ale_fx::warm_up(commands, ale, &c);
                        }
                    }
                }
            }
        } else if !scenery.models.contains_key(&d.kind) {
            let model = debris_model(game, assets, d.kind);
            scenery.models.insert(d.kind, model);
        }
    }
}

/// Breakable objects put down since the map started (`LateBreakable`), once the map's are in:
/// each joins the list as a level's would (its hitpoints, centre, debris made ready), with its
/// model's box for shots.
#[allow(clippy::too_many_arguments)]
fn late_scenery(mut commands: Commands, mut scenery: ResMut<Scenery>, mut game: ResMut<GameData>,
                placed: Query<(Entity, &LateBreakable, &Transform)>, mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
                mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>, mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>) {
    if !scenery.ready {
        return;
    }
    for (e, late, &place) in &placed {
        commands.entity(e).remove::<LateBreakable>();
        let kind = late.0;
        let Some(t) = game.0.breakable(kind).cloned() else {
            println!("h_{kind:08x} doesn't break");
            continue;
        };
        let arch = game.0.object_meshes.get(&kind).copied();
        let centre = place.transform_point(arch.and_then(|a| game.0.archetype_centre(a)).unwrap_or(Vec3::ZERO));
        let bounds = arch.and_then(|a| WeaponModel::load(&game.0, a).ok()).map(|m| m.parts.iter()
            .flat_map(|p| p.geosets.iter().flat_map(move |g| g.positions.iter().map(move |v| p.offset + p.rotation * Vec3::from(*v))))
            .fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(v), u.max(v)))).filter(|(l, u)| l.x <= u.x);
        let explosive = t.debris.iter().any(|d| game.0.object_types.get(&d.kind).is_some_and(|k| k.object_type == OBJECT_EFFECT)
            && game.0.effect_type_defs.get(&d.kind).is_some_and(|e| !e.damage.is_empty()));
        if std::env::var("BF_SCENERY_LOG").is_ok() {
            println!("breakable {} (placed): type h_{kind:08x} at {:.2}, {} hp, box {:.2?}", scenery.list.len(), place.translation, t.hitpoints, bounds);
        }
        if let Some(bx) = bounds {
            PLACED_BOXES.lock().unwrap().push((scenery.list.len(), place, bx));
        }
        scenery.list.push(Breakable { name: kind, kind, place, centre, entity: Some(e), hp: t.hitpoints, state: State::Intact,
                                      push: Push { at: centre, dir: Vec3::ZERO }, explosive, bounds });
        let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
        prepare_debris(&mut commands, &mut scenery, &mut game.0, &mut assets, ale.as_deref_mut(), &t.debris);
    }
}

/// The boxes of the intact breakable objects put down without collision (`LateBreakable`): their
/// list index, placement and box. The flat floor's `ray_hit` (play.rs) stops at them as it does
/// at the pillars, so shots and the crosshair meet them as a map's collision would.
static PLACED_BOXES: std::sync::Mutex<Vec<(usize, Transform, (Vec3, Vec3))>> = std::sync::Mutex::new(Vec::new());

/// How many placed breakables' boxes are in `PLACED_BOXES`, and how many breakable objects the
/// map has (its own and the placed): for the test tools' log of what each map starts with
/// (play_testworld.rs: nothing carried over from the map before).
pub(super) fn counts(scenery: &Scenery) -> (usize, usize) {
    (PLACED_BOXES.lock().unwrap().len(), scenery.list.len())
}

/// How far along a ray (from `origin` along unit `dir`) it meets a placed breakable's box
/// (`PLACED_BOXES`) within `max`, if it does.
pub(super) fn ray_placed(origin: Vec3, dir: Vec3, max: f32) -> Option<f32> {
    PLACED_BOXES.lock().unwrap().iter().filter_map(|(_, place, bx)| ray_box(place, *bx, origin, dir))
        .filter(|&t| t > 0.0 && t <= max).min_by(|a, b| a.total_cmp(b))
}

/// How far along a ray (from `origin` along unit `dir`) it enters a box (`lo`..`hi` in the frame
/// `place`), if it does.
fn ray_box(place: &Transform, (lo, hi): (Vec3, Vec3), origin: Vec3, dir: Vec3) -> Option<f32> {
    let inv = place.compute_affine().inverse();
    let (o, d) = (inv.transform_point3(origin), inv.transform_vector3(dir));
    let (mut t0, mut t1) = (0.0f32, f32::MAX);
    for k in 0..3 {
        if d[k].abs() < 1e-8 {
            if o[k] < lo[k] || o[k] > hi[k] {
                return None;
            }
        } else {
            let (a, b) = ((lo[k] - o[k]) / d[k], (hi[k] - o[k]) / d[k]);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
    }
    // (the local ray's length per world metre is |d|: the same t, as the transform has no scale
    // but the object's own; a scaled one's t is in its units, close enough for a hit test)
    (t0 <= t1).then_some(t0)
}

/// A debris type's model: each part's placement, middle and meshes.
fn debris_model(game: &mut Game, assets: &mut ModelAssets, kind: u32) -> Vec<PieceModel> {
    let Some(&arch) = game.object_meshes.get(&kind) else { return vec![] };
    let Ok(m) = WeaponModel::load(game, arch) else { return vec![] };
    m.parts.iter().map(|p| {
        let (lo, hi) = p.geosets.iter().flat_map(|g| &g.positions).fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(Vec3::from(*v)), u.max(Vec3::from(*v))));
        let middle = if lo.x <= hi.x { (lo + hi) * 0.5 } else { Vec3::ZERO };
        PieceModel { offset: p.offset, rotation: p.rotation, middle, meshes: bf_viewer::scene::static_meshes(game, &p.geosets, assets, true) }
    }).filter(|p| !p.meshes.is_empty()).collect()
}

/// Damage reaching the scenery this frame: the test hook's hits, shots (when they get there)
/// and grenade blasts.
///
/// Test hook: BF_TEST_HIT=<s>[,<s>...] hits the intact breakable object nearest the controlled
/// character (or BF_TEST_HIT_NEAR=<x>,<z>) at those times with TEST_HIT_DAMAGE ballistic, as the
/// takes' gdb call did.
fn hit_scenery(time: Res<Time>, mut scenery: ResMut<Scenery>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               mut blasts: ResMut<ObjectBlasts>) {
    let dt = frame_dt(&time);
    let s = &mut *scenery;
    // grenade blasts: Damage max at the blast, nothing at the radius (as the characters take
    // it), to the object's origin. Their countdown starts on the frame after the blast, as the
    // characters' does (play_grenade.rs counts its pending damage down before new blasts), so
    // both land on the same frame
    let mut landed = vec![];
    s.blasts.retain_mut(|b| {
        b.left -= dt;
        if b.left <= 0.0 {
            landed.push((b.at, b.radius, b.damage, b.damage_type, b.label.clone()));
        }
        b.left > 0.0
    });
    s.blasts.append(&mut blasts.0);
    // (on every map: they reach the Sentries too, `set_off_mines`)
    for &(at, radius, _, kind, _) in &landed {
        s.reach.push((at, radius, kind));
    }
    if !s.ready || s.list.is_empty() {
        return;
    }
    let log = std::env::var("BF_SCENERY_LOG").is_ok();
    let now = player.sim_time;
    // the test hook's hits
    if let Some(times) = std::env::var("BF_TEST_HIT").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<_>>()) {
        if s.test_hits < times.len() && now >= times[s.test_hits] {
            s.test_hits += 1;
            let near = std::env::var("BF_TEST_HIT_NEAR").ok().and_then(|v| {
                let f: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                (f.len() >= 2).then(|| Vec2::new(f[0], f[1]))
            }).unwrap_or(player.position.xz());
            let target = s.list.iter().enumerate().filter(|(_, b)| matches!(b.state, State::Intact))
                .min_by(|a, b| a.1.place.translation.xz().distance(near).total_cmp(&b.1.place.translation.xz().distance(near))).map(|(i, _)| i);
            if let Some(i) = target {
                let at = s.list[i].place.translation;
                let dir = Vec3::new(at.x - player.position.x, 0.0, at.z - player.position.z).normalize_or(Vec3::X);
                if log {
                    println!("t {now:.2}: test hit on breakable {i} (h_{:08x}), {:.1} m away", s.list[i].name, at.xz().distance(player.position.xz()));
                }
                damage(s, &game.0, i, TEST_HIT_DAMAGE, TEST_HIT_TYPE, Push { at: at + Vec3::Y * 0.5, dir }, now);
            }
        }
    }
    // shots: each stops on the first breakable object's collision in its way
    if let Some(arena) = world::arena() {
        for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
            let shots: Vec<Shot> = u.shots.clone();
            for shot in shots {
                let Some((t, Some(b))) = arena.ray_breakable(shot.origin, shot.dir, shot.dist + SHOT_SLACK) else { continue };
                if t > shot.dist + SHOT_SLACK || !matches!(s.list.get(b).map(|b| &b.state), Some(State::Intact)) {
                    continue;
                }
                let [lo, hi] = if shot.damage[1] > 0.0 { shot.damage } else { [8.0, 10.0] };
                // (the gun's falloff over its range, play_shots.rs)
                let amount = (lo + (hi - lo) * u.random(1000) as f32 / 1000.0) * shot.falloff_at(t);
                let push = Push { at: shot.origin + shot.dir * t, dir: shot.dir };
                s.hits.push(PendingHit { delay: t / shot.speed.max(1.0), target: b, amount, damage_type: shot.damage_type, push });
            }
        }
    }
    // ... or on the box of one put down without collision (`LateBreakable`)
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let shots: Vec<Shot> = u.shots.clone();
        for shot in shots {
            let first = s.list.iter().enumerate().filter(|(_, b)| matches!(b.state, State::Intact))
                .filter_map(|(i, b)| b.bounds.and_then(|bx| ray_box(&b.place, bx, shot.origin, shot.dir)).map(|t| (i, t)))
                .filter(|&(_, t)| t <= shot.dist + SHOT_SLACK).min_by(|a, b| a.1.total_cmp(&b.1));
            let Some((b, t)) = first else { continue };
            let [lo, hi] = if shot.damage[1] > 0.0 { shot.damage } else { [8.0, 10.0] };
            let amount = (lo + (hi - lo) * u.random(1000) as f32 / 1000.0) * shot.falloff_at(t);
            let push = Push { at: shot.origin + shot.dir * t, dir: shot.dir };
            s.hits.push(PendingHit { delay: t / shot.speed.max(1.0), target: b, amount, damage_type: shot.damage_type, push });
        }
    }
    let mut due = vec![];
    s.hits.retain_mut(|h| {
        h.delay -= dt;
        if h.delay <= 0.0 {
            due.push((h.target, h.amount, h.damage_type, h.push));
        }
        h.delay > 0.0
    });
    for (b, amount, kind, push) in due {
        damage(s, &game.0, b, amount, kind, push, now);
    }
    // ... to the objects' centres
    for (at, radius, max, kind, label) in landed {
        for i in 0..s.list.len() {
            let o = s.list[i].centre;
            let d = o.distance(at);
            if d >= radius || !matches!(s.list[i].state, State::Intact) {
                continue;
            }
            if log {
                println!("t {now:.2}: {label} blast {d:.1} m from breakable {i} (h_{:08x})", s.list[i].name);
            }
            let dir = (o - at).normalize_or(Vec3::Y);
            damage(s, &game.0, i, max * (1.0 - d / radius), kind, Push { at, dir }, now);
        }
    }
}

/// Damage `amount` of `damage_type` to breakable `i` (times its type's factor): it breaks at 0
/// hitpoints, hit as `push` says.
fn damage(s: &mut Scenery, game: &Game, i: usize, amount: f32, damage_type: i64, push: Push, now: f32) {
    let Some(b) = s.list.get_mut(i) else { return };
    if !matches!(b.state, State::Intact) {
        return;
    }
    let factor = game.object_types.get(&b.kind).map_or(1.0, |t| t.factor(damage_type));
    let value = amount * factor;
    if value <= 0.0 {
        return;
    }
    b.hp -= value;
    if std::env::var("BF_SCENERY_LOG").is_ok() {
        println!("t {now:.2}: breakable {i} (h_{:08x}) takes {amount:.2} x {factor} (type {damage_type}) -> {:.2} hp", b.name, b.hp.max(0.0));
    }
    if b.hp > 0.0 {
        return;
    }
    // broken: its debris queued, the countdown from the longest delay (FUN_00157260)
    let queue = game.object_types.get(&b.kind).map(|t| t.debris.clone()).unwrap_or_default();
    let countdown = queue.iter().map(|d| d.delay).fold(0.0, f32::max);
    b.state = State::Breaking { countdown, queue };
    b.push = push;
    // (one put down by the test map's tools has no collision to drop)
    if let (Some(a), None) = (world::arena(), b.bounds) {
        a.set_broken(i, true);
    }
    PLACED_BOXES.lock().unwrap().retain(|p| p.0 != i);
    if std::env::var("BF_SCENERY_LOG").is_ok() {
        println!("t {now:.2}: breakable {i} (h_{:08x}) breaks; debris over {countdown:.2} s", b.name);
    }
}

/// Breaking objects: the countdown, spawning a debris entry a frame when it's due (the longest
/// delay first), and the object gone once it's done (see the module notes). Effect sounds whose delay
/// is up play.
#[allow(clippy::too_many_arguments)]
fn break_scenery(mut commands: Commands, time: Res<Time>, mut scenery: ResMut<Scenery>, mut player: ResMut<Player>, mut game: ResMut<GameData>,
                 mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
                 mut loose: ResMut<super::pickups::Blasts>) {
    let dt = frame_dt(&time);
    let s = &mut *scenery;
    let log = std::env::var("BF_SCENERY_LOG").is_ok();
    let now = player.sim_time;
    let p = &mut *player;
    s.sounds.retain_mut(|(left, id, at)| {
        *left -= dt;
        if *left <= 0.0 {
            p.sound_queue.push((*id, blast_volume(*at, p.position)));
        }
        *left > 0.0
    });
    for i in 0..s.list.len() {
        let State::Breaking { countdown, queue } = &mut s.list[i].state else { continue };
        *countdown -= dt;
        let left = *countdown;
        // the first entry due spawns (one a frame, FUN_0015af20); none due and the countdown
        // run out: the object goes
        let due: Vec<Debris> = queue.iter().position(|d| d.delay >= left).map(|k| queue.remove(k)).into_iter().collect();
        let done = due.is_empty() && left <= 0.0;
        let (place, centre, push, explosive, name) = (s.list[i].place, s.list[i].centre, s.list[i].push, s.list[i].explosive, s.list[i].name);
        for d in due {
            let kind = game.0.object_types.get(&d.kind).map_or(-1, |k| k.object_type);
            if log {
                println!("t {now:.2}: breakable {i} (h_{name:08x}) spawns h_{:08x} (object-type {kind})", d.kind);
            }
            match kind {
                OBJECT_EFFECT => {
                    let fx = game.0.effect_type_defs.get(&d.kind).cloned().unwrap_or_default();
                    if let Some(ale) = ale.as_deref_mut() {
                        for (k, &e) in fx.effects.iter().chain([&fx.light]).filter(|&&e| e != 0).enumerate() {
                            if let Some(c) = ale.load_recorded(&mut game.0, &mut images, &mut materials, e) {
                                let life = c.duration();
                                let seed = (centre.x * 977.0 + centre.z * 131.0) as u32 ^ k as u32;
                                commands.spawn((place.with_scale(Vec3::ONE).with_translation(centre), Visibility::default(),
                                                bf_viewer::ale_fx::AleEffect::once(c, 0.0, seed), AleExpire(life)));
                            }
                        }
                    }
                    for &(id, delay, _, _) in &fx.sounds {
                        s.sounds.push((delay, id, centre));
                    }
                    for dmg in &fx.damage {
                        s.areas.push(Area { source: i, effect: d.kind, at: centre, damage: *dmg, wait: AREA_DELAY + dmg.delay, age: 0.0,
                                            victims: vec![] });
                        // loose bodies about (pickups, earlier debris) are thrown by it
                        loose.0.push((centre, dmg.range));
                    }
                }
                OBJECT_COMPOUND => {
                    let Some(model) = s.models.get(&d.kind) else { continue };
                    let range = explosive_range(&game.0, s.list[i].kind).unwrap_or(1.0);
                    for (k, part) in model.iter().enumerate() {
                        let at = place * Transform::from_translation(part.offset).with_rotation(part.rotation);
                        let middle = at.transform_point(part.middle);
                        let r = |n: u32| {
                            let x = ((i * 31 + k * 7 + n as usize) as f32 * 12.9898).sin() * 43758.547;
                            x - x.floor()
                        };
                        let jitter = |n: u32| 1.0 + DEBRIS_JITTER * (2.0 * r(n) - 1.0);
                        // out from where it was hit (an explosive one: from between that and
                        // its centre, FUN_00157260), along the hit, and up; an explosive one's
                        // thrown out from its centre instead, low (the barrel's) or up (the
                        // rack's missiles, above it), by (1.2 - d / range)^2
                        let from = if explosive { (centre + push.at) * 0.5 } else { push.at };
                        let out = Vec3::new(middle.x - from.x, 0.0, middle.z - from.z).normalize_or(Vec3::new(r(1) - 0.5, 0.0, r(2) - 0.5).normalize_or(Vec3::X));
                        let along = Vec3::new(push.dir.x, 0.0, push.dir.z);
                        let velocity = if explosive {
                            let d = middle.distance(centre);
                            let k = (1.2 - d / range).max(0.0);
                            let blast = (middle - centre).normalize_or(out);
                            out * DEBRIS_OUT * jitter(3) + blast * DEBRIS_BLAST * k * k * jitter(4) + Vec3::Y * DEBRIS_BLAST_UP * jitter(5)
                        } else {
                            out * DEBRIS_OUT * jitter(3) + along * DEBRIS_ALONG * jitter(4) + Vec3::Y * DEBRIS_UP * jitter(5)
                        };
                        let spin = Vec3::new(r(6) - 0.5, r(7) - 0.5, r(8) - 0.5) * 2.0 * DEBRIS_SPIN;
                        let e = commands.spawn((at, Visibility::default(), Name::new(format!("debris h_{:08x}", d.kind)),
                                                super::pickups::Thrown { velocity, spin }, DebrisPiece { left: DEBRIS_LIFE, scale: at.scale })).id();
                        for (mesh, mat) in &part.meshes {
                            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::default(), ChildOf(e)));
                        }
                    }
                }
                OBJECT_GAME => {
                    // left behind where it stood (its collision is the arena's already)
                    let Some(model) = s.models.get(&d.kind) else { continue };
                    let e = commands.spawn((place, Visibility::default(), Name::new(format!("remains h_{:08x}", d.kind)))).id();
                    for part in model {
                        for (mesh, mat) in &part.meshes {
                            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), NotShadowCaster,
                                            Transform::from_translation(part.offset).with_rotation(part.rotation), ChildOf(e)));
                        }
                    }
                }
                _ => {}
            }
        }
        if done {
            if let Some(e) = s.list[i].entity.take() {
                commands.entity(e).despawn();
            }
            s.list[i].state = State::Gone;
            if log {
                println!("t {now:.2}: breakable {i} (h_{name:08x}) gone");
            }
        }
    }
}

/// The range of the first damage area among `kind`'s debris effects.
fn explosive_range(game: &Game, kind: u32) -> Option<f32> {
    game.breakable(kind)?.debris.iter().filter_map(|d| game.effect_type_defs.get(&d.kind)).flat_map(|e| e.damage.iter()).map(|d| d.range).next()
}

/// A blast sound's volume heard at `listener`: full at the blast, BLAST_QUIET from BLAST_HEARD m
/// (the grenades' choice).
fn blast_volume(at: Vec3, listener: Vec3) -> f32 {
    (1.0 - at.distance(listener) / BLAST_HEARD).clamp(BLAST_QUIET, 1.0)
}

/// The damage areas: from their start for their duration, amount x dt x factor a frame to every
/// character within range with a clear line from the object, and to the breakable objects in
/// range (chain reactions). See the module notes.
#[allow(clippy::too_many_arguments)]
fn area_damage(time: Res<Time>, mut scenery: ResMut<Scenery>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               mut tint: ResMut<ScreenTint>) {
    let dt = frame_dt(&time);
    let s = &mut *scenery;
    let now = player.sim_time;
    let combat = std::env::var("BF_COMBAT_LOG").is_ok();
    let mut hits = vec![];
    for a in s.areas.iter_mut() {
        // the part of this frame the area is at work
        let before = a.wait;
        a.wait -= dt;
        if a.wait > 0.0 {
            continue;
        }
        let from = a.age;
        a.age += if before > 0.0 { -a.wait } else { dt };
        let inside = (a.age.min(a.damage.duration) - from).max(0.0);
        if inside <= 0.0 {
            continue;
        }
        let value = a.damage.amount * inside * DAMAGE_SCALE;
        s.reach.push((a.at, a.damage.range, a.damage.damage_type));
        // the characters in range (FUN_0021ade0 measures each target's place to the effect's,
        // with no ray test: the weapons' blast areas test one, FUN_00224a90, not these)
        for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
            if u.dead {
                continue;
            }
            let middle = u.position + Vec3::Y * (GROUND + u.height + CENTRE_UP);
            let d = middle.distance(a.at);
            if d >= a.damage.range {
                continue;
            }
            let amount = falloff(&a.damage, value, d) * game.0.damage_factor(CHARACTERS[u.character], a.damage.damage_type);
            if amount <= 0.0 {
                continue;
            }
            let first = !a.victims.iter().any(|v| v.0 == u.character);
            if first || amount >= u.health {
                // hurt as by a blast (a grunt, blood); dies from it as from any hurt
                let away = Vec3::new(u.position.x - a.at.x, 0.0, u.position.z - a.at.z).normalize_or(Vec3::X);
                hurt(u, &game.0, amount, HURT_CHATTER, away * 3.0 + Vec3::Y * 2.0, middle, -1);
                if k == 0 {
                    tint.0 = 0.0;
                }
            } else {
                u.health -= amount;
            }
            if combat {
                println!("t {now:.2}: area h_{:08x} at {d:.2} m: {} takes {amount:.2} -> {:.1} / {:.0}", a.effect, CHARACTERS[u.character], u.health, u.max_health);
            }
            match a.victims.iter_mut().find(|v| v.0 == u.character) {
                Some(v) => { v.2 = a.age; v.3 += amount; }
                None => a.victims.push((u.character, a.age, a.age, amount)),
            }
        }
        // the breakable objects in range (to their origin)
        for (i, b) in s.list.iter().enumerate() {
            let d = b.centre.distance(a.at);
            if i != a.source && d < a.damage.range && matches!(b.state, State::Intact) {
                hits.push((i, falloff(&a.damage, value, d), a.damage.damage_type, Push { at: a.at, dir: (b.centre - a.at).normalize_or(Vec3::Y) }));
            }
        }
    }
    for (i, value, kind, push) in hits {
        damage(s, &game.0, i, value, kind, push, now);
    }
    // areas done
    s.areas.retain(|a| {
        let over = a.wait <= 0.0 && a.age >= a.damage.duration;
        if over && (combat || std::env::var("BF_SCENERY_LOG").is_ok()) {
            println!("t {now:.2}: area h_{:08x} over ({} x {} s, range {}, type {}, falloff {})", a.effect, a.damage.amount, a.damage.duration,
                     a.damage.range, a.damage.damage_type, a.damage.falloff);
            for &(who, first, last, total) in &a.victims {
                println!("  {} hurt from +{first:.2} to +{last:.2} s: {total:.1} HP", CHARACTERS[who]);
            }
        }
        !over
    });
}

/// An area's damage this frame (`value`: amount x the frame's part of its time) at `d` m by its
/// falloff, as FUN_0021ade0 deals it: 1 (DFALL_LINEAR) x (1 - d / range); 2 (DFALL_EXPONENTIAL)
/// (1 - d / range)^2 x value^2 (squared value and all, as the code has it); the others, 0
/// (NONE) and 3 (HALF_LIFE: the rack's), in full. The takes' flat rack damage at 4 and 7 m
/// agrees.
fn falloff(damage: &AreaDamage, value: f32, d: f32) -> f32 {
    let k = 1.0 - d / damage.range.max(1e-3);
    match damage.falloff {
        1 => value * k,
        2 => k * k * value * value,
        _ => value,
    }
}

/// Sentries that are down (play_sentry.rs' mines) inside a grenade blast landing this frame, or
/// inside a damage area at work, go off (their fuse set to 0: the next frame's `fly_grenades`
/// sets them off) if their own combat-target takes that damage-type (the Sentry h_e5f1f063: 1
/// hp, Type 3 / 4 x0, 9 / 2 x0.25, 7 x5, 10 x10; so not in the barrel's Type 3 cloud). The
/// demo's: one blast setting off another, as it sets off barrels; the game's handling of a
/// Sentry in a blast isn't traced (no take shows one).
fn set_off_mines(mut scenery: ResMut<Scenery>, player: Res<Player>, game: Res<GameData>, kits: Option<Res<super::grenade::GrenadeKits>>,
                 mut mines: Query<(Entity, &mut super::grenade::Grenade, &Transform), With<super::sentry::Mine>>) {
    let reach = std::mem::take(&mut scenery.reach);
    let Some(kits) = kits else { return };
    for (e, mut g, tr) in &mut mines {
        let takes = |kind: i64| kits.0.get(g.kind).and_then(|k| game.0.object_types.get(&k.def.name)).map_or(1.0, |t| t.factor(kind)) > 0.0;
        if g.fuse > 0.0 && reach.iter().any(|(at, r, kind)| tr.translation.distance(*at) < *r && takes(*kind)) {
            g.fuse = 0.0;
            if std::env::var("BF_SCENERY_LOG").is_ok() {
                println!("t {:.2}: a blast sets off the Sentry {e}", player.sim_time);
            }
        }
    }
}

/// Loose pieces of broken objects shrink away and go at the end of their DEBRIS_LIFE.
fn debris_life(mut commands: Commands, time: Res<Time>, mut pieces: Query<(Entity, &mut DebrisPiece, &mut Transform)>) {
    let dt = frame_dt(&time);
    for (e, mut p, mut tr) in &mut pieces {
        p.left -= dt;
        if p.left <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        if p.left < DEBRIS_SHRINK {
            tr.scale = p.scale * (p.left / DEBRIS_SHRINK);
        }
    }
}
