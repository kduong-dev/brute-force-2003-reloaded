//! Interactive scenery (issue #85): placed objects that break, explode and hurt, from the game's
//! data and code, checked against the xemu takes in todo/85-interactive-scenery/ (and the first
//! capture, todo/interactive scenery objects.mp4).
//!
//! Which objects: a placed game object whose type has hit points (`Game::shootable`, #263). Those
//! whose type has a debris list (h_197caf14, `Game::breakable`; 467 game types have one) break: on
//! sdm_e34 23 radiation barrels (h_e04e5a0e, 1 hp), 6 supply crates (h_09a6856d, 25 hp) and a
//! missile rack (h_fbdcd828, 1 hp), among others. The others die and stay as they are (State::
//! Dead): FUN_00157260 sends their DESTROYED and hides nothing, queues nothing, when the type has
//! no list, so only their listeners act (e23's door h_054428e4, 40 hp, opens on its own DESTROYED:
//! its anim-trigger listens to it). Whether a placed object's +0x1ac mode 1, which makes
//! FUN_00157260 take an object without a list away, is ever set isn't traced.
//!
//! Damage (FUN_002232d0, the combat-target's take-damage): value x the type's factor for its
//! damage-type (the combat-target's own h_142be76f list: the three above take Type 7 x5, 10 x10,
//! 2 x0.25, 3 / 4 / 9 x0), off its hitpoints; at 0 it breaks (message 0x4e). What does damage:
//!  - shots (anyone's): the weapon's Damage min..max at random, when the shot gets there (a
//!    bolt's roll made at its launch, on the sweep that meets it: play_bolts.rs), on the
//!    object's own collision (`Arena::ray_breakable`);
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
//!    sounds, and its `<Damage>` areas and shakes: a request for play_effects.rs
//!    (`EffectSpawn`, #373), which starts the effect as it does any effect type.
//!
//! Damage areas (an effect's `<Damage amount Type duration range falloff>`) are play_effects.rs's
//! since #373 (the barrels' were this module's before it generalised them): from AREA_DELAY
//! after the effect, for `duration` s, everyone within `range` of the object's centre (3D,
//! to their point CENTRE_UP above the feet) loses amount x dt x their factor a frame, with no
//! line-of-sight test: amount x duration in all (the barrel's 80 x 0.5 = 40 HP, the rack's 200 x
//! 0.5 = 100). Breakable objects in range take it too, times their own factor
//! (`Scenery::area_at_work`): the rack's explosion (Type 10, x10) sets off barrels 4-5 m away at
//! once, the barrel's (Type 3, x0) does nothing to barrels or crates. The rest of what the
//! effect object's damage code does (FUN_0021ade0), the push h_ed582b3c and the character's
//! difficulty step are in play_effects.rs's notes. The objects aren't stepped.
//!
//! The test map's object tool (play_testtools.rs) puts these down on the flat floor too
//! (`LateBreakable`): they join the list after the map's, and with no arena their model's box
//! stands in for their collision (`ray_placed`, which play.rs's `ray_hit` stops at).

use super::*;
use bf_viewer::bf::character::{AreaDamage, Debris, OBJECT_COMPOUND, OBJECT_EFFECT, OBJECT_GAME};
use bf_viewer::level_scene::Placed;

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

/// A blast for the scenery (play_grenade.rs adds a grenade's when it goes off, play_missile.rs a
/// missile's): it lands after `left` s, with its Damage max at `at` falling to nothing at
/// `radius` (or `damage` flat, `flat`), of `damage_type`.
pub struct ObjectBlast {
    pub left: f32,
    pub at: Vec3,
    pub radius: f32,
    pub damage: f32,
    pub damage_type: i64,
    /// the grenade's label (BF_SCENERY_LOG)
    pub label: String,
    /// `damage` anywhere inside the radius, without the falloff: a missile's blast object, whose
    /// share is flat (play_missile.rs, FUN_0022c4d0 with Damage h_011cb154 false)
    pub flat: bool,
}

/// Grenade blasts on their way to the scenery.
#[derive(Resource, Default)]
pub struct ObjectBlasts(pub Vec<ObjectBlast>);

/// A direct hit on a breakable object that isn't found from a shot at its shot (play.rs's
/// `Shot`s find their own): a missile that strikes it (play_missile.rs, FUN_0022c4d0's roll on the
/// object struck), a bolt's sweep that meets it (play_bolts.rs: the roll made at its launch). Its
/// object (`Arena::breakable`'s number), the amount before the object's damage-type factor, the
/// damage type, where it struck and which way.
pub struct ObjectHit {
    pub target: usize,
    pub amount: f32,
    pub damage_type: i64,
    pub at: Vec3,
    pub dir: Vec3,
}

/// This frame's direct hits on breakable objects (`ObjectHit`).
#[derive(Resource, Default)]
pub struct ObjectHits(pub Vec<ObjectHit>);

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
    /// dead without a debris list (#263): its DESTROYED sent, still drawn and solid
    Dead,
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
    /// its type has a debris list (`Game::breakable`): it breaks at 0 hit points; else it dies
    /// and stays (`State::Dead`, #263)
    breaks: bool,
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

impl Scenery {
    /// The drawn model of breakable `b` (its list index: the arena's breakable number, or past
    /// them a test tool's) until it's gone: whole, dead without a debris list (#263), or breaking
    /// until its countdown ends (`break_scenery`). What a decal laid on it goes with (play_fx.rs
    /// `tie_decals`, #301): the game's object lives until then (FUN_0015ae30), so the shot that
    /// breaks it lays its hole on it too.
    pub(super) fn model(&self, b: usize) -> Option<Entity> {
        self.list.get(b).filter(|x| !matches!(x.state, State::Gone)).and_then(|x| x.entity)
    }

    /// Whether breakable `b` is gone: its break over (`break_scenery`), or taken away by a
    /// destroyer-trigger as the level started (`find_scenery`). A shot's hole laid on it
    /// afterwards goes at once (play_fx.rs `tie_decals`).
    pub(super) fn gone(&self, b: usize) -> bool {
        self.list.get(b).is_some_and(|x| matches!(x.state, State::Gone))
    }

    /// A damage area at work this frame (play_effects.rs `area_damage`, #373), for what the
    /// scenery holds: its reach is noted for the Sentries that are down (`set_off_mines`), and
    /// every intact breakable object but `source` (the one the effect came from) within
    /// `area.range` of `at`, measured to the object's centre, is noted in `hits` to take `value`
    /// (the area's amount x the part of the frame it was at work) by the area's falloff
    /// (`effects::falloff`): FUN_0021ade0 deals to every target in range, objects included. It's
    /// what sets off the barrels around a missile rack. `deal_area_hits` deals them, once every
    /// area at work this frame has been looked at (the objects' states are the frame's start's).
    pub(super) fn area_at_work(&mut self, area: &AreaDamage, at: Vec3, value: f32, source: Option<usize>, hits: &mut AreaHits) {
        self.reach.push((at, area.range, area.damage_type));
        for (i, b) in self.list.iter().enumerate() {
            let d = b.centre.distance(at);
            if Some(i) != source && d < area.range && matches!(b.state, State::Intact) {
                hits.0.push((i, super::effects::falloff(area, value, d), area.damage_type, Push { at, dir: (b.centre - at).normalize_or(Vec3::Y) }));
            }
        }
    }

    /// The breakable objects' share of this frame's damage areas (`AreaHits`), dealt: each takes
    /// its value times its factor for the damage type (`damage`), and breaks at 0 hit points.
    pub(super) fn deal_area_hits(&mut self, game: &Game, hits: AreaHits, now: f32) {
        for (i, value, kind, push) in hits.0 {
            damage(self, game, i, value, kind, push, now);
        }
    }
}

/// What the damage areas at work in a frame do to the breakable objects (`Scenery::area_at_work`
/// notes it, `Scenery::deal_area_hits` deals it): per object, its list index, the amount, the
/// damage type and the push.
#[derive(Default)]
pub(super) struct AreaHits(Vec<(usize, f32, i64, Push)>);

/// A loose piece of a broken object: seconds left, and its own scale (it shrinks away at the end).
#[derive(Component)]
struct DebrisPiece {
    left: f32,
    scale: Vec3,
}

/// The scenery's systems on a frame (`plugin`): what hangs on an object taken out of the world
/// (play_placed_fx.rs, play_lamps.rs) runs after them, on the frame the object goes.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SceneryStep;

pub fn plugin(app: &mut App) {
    app.init_resource::<Scenery>()
        .init_resource::<ObjectBlasts>()
        .init_resource::<ObjectHits>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| {
            commands.insert_resource(Scenery::default());
            PLACED_BOXES.lock().unwrap().clear();
            commands.insert_resource(ObjectBlasts::default());
            commands.insert_resource(ObjectHits::default());
        }).after(setup))
        // (after the grenades: a blast's damage reaches the scenery on the frame the
        // characters take it)
        // (before the hit effects: this frame's hits are dealt before their holes are laid, as
        // the game's damage handler does, 0x22d0b0, and a model taken away takes its decals on
        // the same frame, play_fx.rs `tie_decals`, `untie_decals`, #301)
        // (the mines set off before they're handed to the next frame's shots)
        // (the effects started and their damage areas at work, play_effects.rs, run between
        // `break_scenery`, which asks for them, and `set_off_mines`, which reads their reach)
        .add_systems(Update, (find_scenery, late_scenery, hit_scenery, break_scenery, set_off_mines, debris_life).chain()
            .in_set(SceneryStep).after(super::grenade::GrenadeSystems).before(super::fx::FxSystems).before(super::sentry::publish_targets).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
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
    // (the placed models by type: thousands of objects have hit points, #263)
    let mut models: HashMap<u32, Vec<(Entity, Vec3)>> = HashMap::new();
    for (e, p, tr) in &placed {
        models.entry(p.kind).or_default().push((e, tr.translation));
    }
    for (i, o) in level.objects.iter().enumerate() {
        let Some(b) = arena.breakable(i) else { continue };
        let Some(t) = game.0.shootable(o.kind).cloned() else { continue };
        let place = Transform::from_matrix(o.transform);
        let entity = models.get(&o.kind).and_then(|m| m.iter().find(|(_, at)| at.distance(place.translation) < 0.01)).map(|(e, _)| *e);
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
        // (one a destroyer-trigger removed as the level started is gone already: the tutorial's
        // four h_0247aefb props, play_triggers.rs)
        let state = if arena.is_broken(b) { State::Gone } else { State::Intact };
        list.push(Breakable { name: o.name, kind: o.kind, place, centre, entity, hp: t.hitpoints, state, breaks: !t.debris.is_empty(),
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
        scenery.list.push(Breakable { name: kind, kind, place, centre, entity: Some(e), hp: t.hitpoints, state: State::Intact, breaks: true,
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
    rays_placed(origin, dir, max).into_iter().map(|(t, _)| t).min_by(|a, b| a.total_cmp(b))
}

/// Every placed breakable's box a ray (from `origin` along unit `dir`) meets within `max`, how far
/// along, with the breakable (its list index, as `ObjectHit::target` takes it): a bolt's sweep
/// strikes them there (play_bolts.rs).
pub(super) fn rays_placed(origin: Vec3, dir: Vec3, max: f32) -> Vec<(f32, usize)> {
    PLACED_BOXES.lock().unwrap().iter().filter_map(|(i, place, bx)| ray_box(place, *bx, origin, dir).map(|(t, _)| (t, *i)))
        .filter(|&(t, _)| t > 0.0 && t <= max).collect()
}

/// The breakable object a shot's ray (from `origin` along unit `dir`) met where it met the world,
/// `t` m on, if it was one: its list index, as `ObjectHit::target` takes it. The arena's
/// (`Arena::ray_breakable`) or on the flat test floor a test tool's box (`rays_placed`), within
/// SHOT_SLACK of `t`. Found with the shot (play.rs `Shot::breakable`), as the game's hit record
/// holds the object struck, which FUN_0022ca20 lays the shot's hole on (FUN_0022c4a0): the hole
/// goes with it (play_fx.rs `tie_decals`, #301).
pub(super) fn struck_at(origin: Vec3, dir: Vec3, t: f32) -> Option<usize> {
    let there = |x: f32| (x - t).abs() <= SHOT_SLACK;
    world::arena().and_then(|a| a.ray_breakable(origin, dir, t + SHOT_SLACK)).filter(|&(x, _)| there(x)).and_then(|(_, b)| b)
        .or_else(|| rays_placed(origin, dir, t + SHOT_SLACK).into_iter().filter(|&(x, _)| there(x)).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, i)| i))
}

/// Every crossing of a placed breakable's box a ray (from `origin` along unit `dir`) makes within
/// `max`: where it goes in and where it comes out, each with the breakable and whether it's the
/// way out. A pass-through ray's multi-hit query meets a closed object at both (play_rays.rs,
/// `Arena::ray_shot_all`, #287).
pub(super) fn crossings_placed(origin: Vec3, dir: Vec3, max: f32) -> Vec<(f32, usize, bool)> {
    PLACED_BOXES.lock().unwrap().iter().filter_map(|(i, place, bx)| ray_box(place, *bx, origin, dir).map(|(t0, t1)| (t0, t1, *i)))
        .flat_map(|(t0, t1, i)| [(t0, i, false), (t1, i, true)]).filter(|&(t, _, _)| t > 0.0 && t <= max).collect()
}

/// How far along a ray (from `origin` along unit `dir`) it enters a box (`lo`..`hi` in the frame
/// `place`) and leaves it, if it meets it.
fn ray_box(place: &Transform, (lo, hi): (Vec3, Vec3), origin: Vec3, dir: Vec3) -> Option<(f32, f32)> {
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
    (t0 <= t1).then_some((t0, t1))
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
/// takes' gdb call did: one with a debris list, or with BF_TEST_HIT_ANY=1 any with hit points
/// (#263: e23's door).
fn hit_scenery(time: Res<Time>, mut scenery: ResMut<Scenery>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               mut blasts: ResMut<ObjectBlasts>, mut direct: ResMut<ObjectHits>) {
    let dt = frame_dt(&time);
    let direct = std::mem::take(&mut direct.0);
    let s = &mut *scenery;
    // grenade blasts: Damage max at the blast, nothing at the radius (as the characters take
    // it), to the object's origin. Their countdown starts on the frame after the blast, as the
    // characters' does (play_grenade.rs counts its pending damage down before new blasts), so
    // both land on the same frame
    let mut landed = vec![];
    s.blasts.retain_mut(|b| {
        b.left -= dt;
        if b.left <= 0.0 {
            landed.push((b.at, b.radius, b.damage, b.damage_type, b.label.clone(), b.flat));
        }
        b.left > 0.0
    });
    s.blasts.append(&mut blasts.0);
    // (on every map: they reach the Sentries too, `set_off_mines`)
    for &(at, radius, _, kind, _, _) in &landed {
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
            let any = std::env::var("BF_TEST_HIT_ANY").is_ok();
            let target = s.list.iter().enumerate().filter(|(_, b)| matches!(b.state, State::Intact) && (b.breaks || any))
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
    // shots: each stops on the first breakable object's collision in its way (a bolt strikes one
    // on its sweep, play_bolts.rs, `ObjectHits`)
    if let Some(arena) = world::arena() {
        for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
            let shots: Vec<Shot> = u.shots.iter().filter(|s| !s.swept()).copied().collect();
            for shot in shots {
                let Some((t, Some(b))) = arena.ray_breakable(shot.origin, shot.dir, shot.dist + SHOT_SLACK) else { continue };
                if t > shot.dist + SHOT_SLACK || !matches!(s.list.get(b).map(|b| &b.state), Some(State::Intact)) {
                    continue;
                }
                let [lo, hi] = if shot.damage[1] > 0.0 { shot.damage } else { [8.0, 10.0] };
                // (the roll x the shooter's skill's factor, FUN_0022c940: Flint's doubled and
                // scoped rolls, and its power-ups', BRUTE's x2; the gun's falloff over its
                // range, play_shots.rs)
                let amount = (lo + (hi - lo) * u.random(1000) as f32 / 1000.0) * super::skills::roll_factor(u) * super::powerups::roll_factor(u)
                    * shot.falloff_at(t);
                let push = Push { at: shot.origin + shot.dir * t, dir: shot.dir };
                s.hits.push(PendingHit { delay: shot.delay(t), target: b, amount, damage_type: shot.damage_type, push });
            }
        }
    }
    // missiles that struck one (`ObjectHits`), at once
    for h in direct {
        if matches!(s.list.get(h.target).map(|b| &b.state), Some(State::Intact)) {
            s.hits.push(PendingHit { delay: 0.0, target: h.target, amount: h.amount, damage_type: h.damage_type, push: Push { at: h.at, dir: h.dir } });
        }
    }
    // ... or on the box of one put down without collision (`LateBreakable`)
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let shots: Vec<Shot> = u.shots.iter().filter(|s| !s.swept()).copied().collect();
        for shot in shots {
            let first = s.list.iter().enumerate().filter(|(_, b)| matches!(b.state, State::Intact))
                .filter_map(|(i, b)| b.bounds.and_then(|bx| ray_box(&b.place, bx, shot.origin, shot.dir)).map(|(t, _)| (i, t)))
                .filter(|&(_, t)| t <= shot.dist + SHOT_SLACK).min_by(|a, b| a.1.total_cmp(&b.1));
            let Some((b, t)) = first else { continue };
            let [lo, hi] = if shot.damage[1] > 0.0 { shot.damage } else { [8.0, 10.0] };
            let amount = (lo + (hi - lo) * u.random(1000) as f32 / 1000.0) * super::skills::roll_factor(u) * super::powerups::roll_factor(u)
                * shot.falloff_at(t);
            let push = Push { at: shot.origin + shot.dir * t, dir: shot.dir };
            s.hits.push(PendingHit { delay: shot.delay(t), target: b, amount, damage_type: shot.damage_type, push });
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
    for (at, radius, max, kind, label, flat) in landed {
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
            // (a missile's blast object deals its share flat, play_missile.rs)
            let amount = if flat { max } else { max * (1.0 - d / radius) };
            damage(s, &game.0, i, amount, kind, Push { at, dir }, now);
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
    // dead: its DESTROYED (FUN_00157260, the trigger runtime's from `Arena::is_dead`)
    if let Some(a) = world::arena().filter(|_| b.bounds.is_none()) {
        a.set_dead(i);
    }
    // without a debris list it stays, drawn and solid (#263)
    if !b.breaks {
        b.state = State::Dead;
        if std::env::var("BF_SCENERY_LOG").is_ok() {
            println!("t {now:.2}: breakable {i} (h_{:08x}) dies (no debris list): it stays", b.name);
        }
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
/// delay first), and the object gone once it's done (see the module notes). An effect entry is
/// handed to play_effects.rs (`EffectSpawn`), which starts it later this frame, before the
/// damage areas are at work.
pub(super) fn break_scenery(mut commands: Commands, time: Res<Time>, mut scenery: ResMut<Scenery>, player: Res<Player>, game: Res<GameData>,
                            mut spawns: ResMut<super::effects::EffectSpawns>) {
    let dt = frame_dt(&time);
    let s = &mut *scenery;
    let log = std::env::var("BF_SCENERY_LOG").is_ok();
    let now = player.sim_time;
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
                // (its ALE effects and light effect at the object's centre in the object's turn, its
                // sounds, and its damage areas and shakes measured from the centre, a kill's shove
                // from the object's origin, the object itself not hurt by them: all of an effect
                // spawn's, play_effects.rs)
                OBJECT_EFFECT => spawns.0.push(super::effects::EffectSpawn {
                    kind: d.kind, at: centre, origin: place.translation, rotation: place.rotation, source: Some(i), look: true,
                }),
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
            // (taken out of the world, FUN_0015ae30 -> FUN_000db3e0: what hangs on it goes, #308)
            if let Some(a) = world::arena() {
                a.set_gone(i);
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

/// Sentries that are down (play_sentry.rs' mines) inside a grenade blast landing this frame, or
/// inside a damage area at work, go off (their fuse set to 0: the next frame's `fly_grenades`
/// sets them off) if their own combat-target takes that damage-type (the Sentry h_e5f1f063: 1
/// hp, Type 3 / 4 x0, 9 / 2 x0.25, 7 x5, 10 x10; so not in the barrel's Type 3 cloud). The
/// demo's: one blast setting off another, as it sets off barrels; the game's handling of a
/// Sentry in a blast isn't traced (no take shows one).
pub(super) fn set_off_mines(mut scenery: ResMut<Scenery>, player: Res<Player>, game: Res<GameData>, kits: Option<Res<super::grenade::GrenadeKits>>,
                            mut mines: Query<(Entity, &mut super::grenade::Grenade, &Transform), With<super::sentry::Mine>>) {
    let reach = std::mem::take(&mut scenery.reach);
    let Some(kits) = kits else { return };
    for (e, mut g, tr) in &mut mines {
        let takes = |kind: i64| kits.0.get(g.kind).and_then(|k| game.0.object_types.get(&k.def.name)).map_or(1.0, |t| t.factor(kind)) > 0.0;
        if g.fuse > 0.0 && reach.iter().any(|(at, r, kind)| tr.translation.distance(*at) < *r && takes(*kind)) {
            g.fuse = 0.0;
            g.set_off = true;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A damage area at work (play_effects.rs, #373) reaches the intact breakable objects within
    /// its range by its falloff, but not the one it came from nor the ones out of range or
    /// already broken, and notes its reach for the Sentries: the rack's explosion sets off the
    /// barrels around it, not itself.
    #[test]
    fn a_damage_area_reaches_the_objects_in_range_but_its_source() {
        let at = |x: f32, state| Breakable { name: 0, kind: 0, place: Transform::from_xyz(x, 0.0, 0.0), centre: Vec3::new(x, 0.0, 0.0), entity: None, hp: 100.0,
                                             state, breaks: true, push: Push { at: Vec3::ZERO, dir: Vec3::ZERO }, explosive: false, bounds: None };
        let mut s = Scenery {
            list: vec![at(1.0, State::Intact), at(3.5, State::Intact), at(9.0, State::Intact), at(0.5, State::Intact), at(2.0, State::Gone)],
            ..default()
        };
        // the Molotov's cloud as the data has it: 78 FLAME a second, 7 m, falloff 1; the frame's
        // share 10
        let area = AreaDamage { amount: 78.0, damage_type: 5, push: 1.0, delay: 0.0, duration: 1.0, range: 7.0, falloff: 1 };
        let mut hits = AreaHits::default();
        s.area_at_work(&area, Vec3::ZERO, 10.0, Some(3), &mut hits);
        let got: Vec<(usize, f32, i64)> = hits.0.iter().map(|h| (h.0, h.1, h.2)).collect();
        assert_eq!(got.len(), 2, "{got:?}");
        assert_eq!((got[0].0, got[0].2), (0, 5));
        assert!((got[0].1 - 10.0 * (1.0 - 1.0 / 7.0)).abs() < 1e-4);
        assert_eq!(got[1].0, 1);
        assert!((got[1].1 - 5.0).abs() < 1e-4);
        assert_eq!(s.reach, vec![(Vec3::ZERO, 7.0, 5)]);
    }

    /// A breakable keeps its model for the decals laid on it (play_fx.rs `tie_decals`) until it's
    /// gone: whole, dead without a debris list, and breaking, as the shot that breaks it lays its
    /// hole after the break (#301). Gone, it has none, and a hole laid on it then goes at once.
    #[test]
    fn a_breaking_object_holds_its_decals_until_it_goes() {
        let model = Entity::from_raw(7);
        let breakable = |state, entity| Breakable { name: 0, kind: 0, place: Transform::IDENTITY, centre: Vec3::ZERO, entity, hp: 0.0, state,
                                                    breaks: true, push: Push { at: Vec3::ZERO, dir: Vec3::ZERO }, explosive: false, bounds: None };
        let s = Scenery {
            list: vec![breakable(State::Intact, Some(model)), breakable(State::Dead, Some(model)),
                       breakable(State::Breaking { countdown: 0.1, queue: vec![] }, Some(model)),
                       // (its break over, `break_scenery` took its model; one a destroyer-trigger
                       // removed as the level started keeps its hidden model)
                       breakable(State::Gone, None), breakable(State::Gone, Some(model))],
            ..default()
        };
        for b in 0..3 {
            assert_eq!(s.model(b), Some(model), "breakable {b}");
            assert!(!s.gone(b), "breakable {b}");
        }
        for b in 3..5 {
            assert_eq!(s.model(b), None, "breakable {b}");
            assert!(s.gone(b), "breakable {b}");
        }
        assert_eq!(s.model(9), None);
        assert!(!s.gone(9));
    }
}
