//! Grenades, every type the loaded levels define, as in the game's data and the xemu recordings
//! (issues #74-#81; the measurements are the reference agents' from the user's recordings).
//!
//! Data (objecttypes-<level>, `inventory-grenade`, read into `WeaponDef`): each type has
//!  - `timer`: the fuse (s), counted from the first contact with the ground (the Frag's 1.5 s:
//!    1.52 s from landing to blast in the recording); 0 goes off on the first contact, without a
//!    bounce (the Sonic);
//!  - h_053c429f: the explosion's weapon definition: `<Damage max min damage-type radius>` and a
//!    bullet whose h_ec23d593 is the effect type played where it goes off (its ALE effects, a
//!    light effect and a sound, see `EffectType`), h_e5618348 an impact sound (the Frag's
//!    fb4b3604, the Light's ignition h_1080aaf1; empty on most) and h_06a27365 a ground decal
//!    (the Frag's scorch h_ff1b711e);
//!  - h_18b6ab72's first slot: the effect type attached to it (its trail: grenade_trail's smoke
//!    and a sound, the Frag's hiss 10318b29; none on the Roller and the Sentry);
//!  - h_e5ec3f1f its HUD icon, stringtable-name its label, mesh-name its model, stack-limit,
//!    function-type (0 thrown, 8 IFSET_PROXIMITY_EXPLOSIVE: Sentry, 13 IFSET_ROLLING_BOMB:
//!    Roller) and h_1ee2f4ed how it's used (3 IOU_THROW_TO_USE, 2 IOU_PLACE_ON_GROUND);
//!  - an object event (state 7, h_fa04e025 = e43166d1): the sound heard as the meter appears.
//!
//! Thrown types: hold the button to charge (the HUD meter, full in 0.6 s), let go to throw: the
//! count drops at once (one game frame after the button in the recordings), the stance's throw
//! clip takes the grenade in hand at its reach event (its trail and hiss start then: ~0.33 s
//! after the button) and lets go at its release event. It bounces silently and goes off when its
//! fuse ends. Place-on-ground types (Roller, Sentry) are set down at the feet with the stance's
//! place_hi clip (no meter, the count drops at the press); the Roller rolls off straight ahead.
//!
//! The blast plays the effect type's ALE effects and light (ale_fx.rs) and sound and the impact
//! sound, leaves the decal, and hurts everyone in its radius (and throws loose pickups) unless
//! it does no damage or deals it over time. Hurt by it, the player's 3D view goes red for a
//! moment (see `TINT_LOW`).
//!
//! The Gas's damage over time is play_gas.rs's (its blast is handed over there, see
//! `fly_grenades`), the Energy's comes with its bolts (play_energy.rs). The Light does no
//! damage: its canister stays where it lies while phosphor_grenade and light_phosphor burn
//! (30 s, see `stays`). Not done yet (each type's own ticket): the
//! Sonic's ring that carries the damage out, the Roller's seeking and the Sentry's trigger (it
//! lies there until its 9999 s timer or BF_TEST_DETONATE).

use super::*;
use bf_viewer::bf::character::EffectType;

/// throw speed range (m/s) over the charge, and extra upward angle over the crosshair (radians).
/// Fitted to the Frag recording, not from the data: the grenade leaves the hand 0.60-0.63 s
/// after the button and goes off 1.65-1.78 s after that, so with the 1.5 s fuse from the first
/// contact it's down 0.13-0.28 s after it leaves the hand, a few metres ahead where the camera
/// looks down (frag/a 588-602): a fast throw, aimed below the crosshair (a negative lob).
const MIN_THROW: f32 = 12.0;
const MAX_THROW: f32 = 20.0;
const THROW_LOB: f32 = -0.25;
const GRENADE_GRAVITY: f32 = 9.8;
pub(super) const GRENADE_RADIUS: f32 = 0.06;
/// bounce: vertical restitution, horizontal speed kept per bounce (the demo's; the friction
/// fitted so a fast throw comes to rest near where it lands, as the recording's does: its trail
/// puffs from about where it came down, frag/a 600-690)
const RESTITUTION: f32 = 0.35;
const GROUND_FRICTION: f32 = 0.3;
/// h_1ee2f4ed: IOU_PLACE_ON_GROUND (Roller, Sentry; the XBE's IOU_ enum)
const USE_PLACE: i64 = 2;
/// function-type IFSET_ROLLING_BOMB (the Roller: rolls off once set down)
const ROLLING_BOMB: i64 = 13;
/// The Roller's speed once down (m/s; 4.4-5.0 measured, straight ahead along the thrower's
/// facing, no target in the recording).
const ROLL_SPEED: f32 = 4.7;
/// While it rolls, the Roller's object sound (h_19dbc65d) plays again every ROLL_SOUND_EVERY s
/// (the recording: a loop every ~0.97 s), heard fully within ROLL_SOUND_NEAR m and fading to
/// nothing at ROLL_SOUND_FAR m (the sound's own `<Sound falloff=5 h_fd40e332=25>` in
/// sounds-mp1.xml, read as full-volume and silent distances).
const ROLL_SOUND_EVERY: f32 = 0.97;
const ROLL_SOUND_NEAR: f32 = 5.0;
const ROLL_SOUND_FAR: f32 = 25.0;
/// The thrower's own damage from their blast: SELF_DAMAGE x a roll of the explosion's Damage
/// min..max, at any distance within the radius. Measured, not the game's formula (unknown): the
/// Frag recording's six blasts each took 12.5-13.3 HP of Tex's 115 (~11%) near or far, with no
/// falloff; 58.5-65 x 0.2 fits.
const SELF_DAMAGE: f32 = 0.2;
/// The red damage tint, hurt by a blast (the Frag recording, all six blasts): the 3D picture's
/// green and blue are multiplied by TINT_CURVE's factors, one per game frame (1/30 s; the
/// reference agent's measurement of the recording), red kept, the HUD untouched, as strong
/// near or far; TINT_LOW is the deepest, TINT_TIME how long until it's gone.
const TINT_CURVE: [f32; 9] = [0.20, 0.29, 0.39, 0.51, 0.61, 0.69, 0.78, 0.92, 1.0];
const TINT_LOW: f32 = 0.2;
const TINT_TIME: f32 = 0.27;
/// A blast's damage (and with it the tint) lands this long after it goes off: the recordings'
/// tint starts 3 game frames after the flash (Frag 693 -> 699, Sonic 229-231 -> 235). Measured;
/// the game's reason (the blast's spread, a damage tick) isn't known.
const DAMAGE_DELAY: f32 = 0.1;
/// A blast's decal is drawn with its width / height read as half sizes, as the blood decals'
/// are (play_fx.rs DECAL_SCALE, fitted to the blood captures): the Frag's scorch h_ff1b711e
/// (3 x 3) is a 6 m quad, whose soft texture is half dark over ~3.7 m - the recording's scorch
/// is half dark over ~4.1 m (the reference agent's measurement). At the data's size it came out
/// ~1.9 m.
const GRENADE_DECAL_SCALE: f32 = 2.0;
/// The decal is laid this long after the blast, once the fireball has gone (the recording's
/// bright part is gone by 0.63 s, and the scorch is first seen under the smoke). The demo's
/// choice: laid at once, the dark decal showed through the added fireball as a hard dark disc.
const DECAL_DELAY: f32 = 0.7;
/// How far in front of the camera the tint quad sits (m; inside the near plane's 0.1 is cut).
const TINT_DEPTH: f32 = 0.15;
/// The tint's levels (materials) over TINT_TIME.
const TINT_STEPS: usize = 16;
/// The stored value the tint's factor is exact for (see `tint_factor`).
const TINT_GREY: f32 = 0.4;
/// Blast sounds are this loud at the blast, falling to BLAST_QUIET at BLAST_HEARD m (the demo's).
const BLAST_HEARD: f32 = 60.0;
const BLAST_QUIET: f32 = 0.3;
/// The squad's grenade definitions, in the order the demo's grenade key (T) steps through them:
/// Frag (#75), Energy (#74), Gas (#76), Light (#77), Sonic (#81), Roller (#79), Sentry (#80).
/// Some labels have several definitions (the Sentry three, the Roller two): these are the ones
/// the recordings show. The squad carries only these: the other labelled grenade with an icon,
/// the Molotov (e01's h_f42e0faa), is the mutants' weapon (#78); its definition is read like
/// any other (`Game::weapons`) for an AI thrower to use.
const SQUAD_GRENADES: [u32; 7] = [0x060F_95F7, 0x0CC3_C2C8, 0xE8C6_7904, 0xFD1A_966D, 0xEC0B_F28F, 0xFBA7_C475, 0xE5F1_F063];
/// The Frag (the main game's starting grenades).
const FRAG: u32 = 0x060F_95F7;
/// grenades carried at the start of the main game (the capture's HUD shows 3 Frags)
const START_GRENADES: i64 = 3;

/// Throw velocity for a charge of `power` (0..1) toward the crosshair ray, lowered by
/// THROW_LOB, aimed between 0.8 rad down and 1.2 rad up.
pub fn throw_velocity(ray: Vec3, power: f32) -> Vec3 {
    let flat = Vec3::new(ray.x, 0.0, ray.z).normalize_or(Vec3::NEG_Z);
    let pitch = (ray.y.clamp(-1.0, 1.0).asin() + THROW_LOB).clamp(-0.8, 1.2);
    (flat * pitch.cos() + Vec3::Y * pitch.sin()) * (MIN_THROW + (MAX_THROW - MIN_THROW) * power.clamp(0.0, 1.0))
}

/// One grenade type: its item definition, its explosion, their effect types and its model.
pub struct GrenadeKit {
    /// the inventory-grenade definition (timer, icon, label, use type ...)
    pub def: WeaponDef,
    /// the explosion's weapon definition (Damage, the bullet's effect type, impact sound, decal)
    pub blast: WeaponDef,
    /// the explosion's effect type and the attached (trail) effect type
    pub blast_fx: EffectType,
    pub trail_fx: EffectType,
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Vec3)>,
}

impl GrenadeKit {
    fn load(game: &mut Game, assets: &mut ModelAssets, def: WeaponDef) -> Self {
        let blast = game.weapons.get(&def.projectile).cloned().unwrap_or_default();
        let fx = |game: &Game, t: u32| game.effect_type_defs.get(&t).cloned().unwrap_or_default();
        let (blast_fx, trail_fx) = (fx(game, blast.blast_effect), fx(game, def.attached_effect));
        let mut parts = vec![];
        match WeaponModel::load(game, def.archetype) {
            Ok(m) => for p in &m.parts {
                for (mesh, mat) in bf_viewer::scene::static_meshes(game, &p.geosets, assets, true) {
                    parts.push((mesh, mat, p.offset));
                }
            },
            Err(e) => warn!("grenade {} model: {e}", def.label),
        }
        if parts.is_empty() {
            let mat = assets.materials.add(StandardMaterial { base_color: Color::srgb(0.35, 0.3, 0.2), ..default() });
            parts.push((assets.meshes.add(Sphere::new(GRENADE_RADIUS).mesh().ico(2).unwrap()), mat, Vec3::ZERO));
        }
        info!("grenade {} (h_{:08x}): timer {}, use {}, function {}, explosion h_{:08x} {}-{} type {} radius {}, effect h_{:08x} {:x?} light {:08x} sounds {:x?}, impact {:08x}, decal {:08x}, trail h_{:08x} {:x?} {:x?}, icon {:08x}, {} parts",
              def.label, def.name, def.fuse, def.use_type, def.function_type, def.projectile, blast.damage_min, blast.damage, blast.damage_type,
              blast.blast_radius, blast.blast_effect, blast_fx.effects, blast_fx.light, blast_fx.sounds, blast.impact_sound, blast.decal,
              def.attached_effect, trail_fx.effects, trail_fx.sounds, def.icon, parts.len());
        GrenadeKit { def, blast, blast_fx, trail_fx, parts }
    }

    /// Goes off on the first contact (timer 0: the Sonic).
    pub fn on_contact(&self) -> bool {
        self.def.fuse <= 0.0
    }

    /// Set down at the feet rather than thrown (IOU_PLACE_ON_GROUND: Roller, Sentry).
    pub fn placed(&self) -> bool {
        self.def.use_type == USE_PLACE
    }

    /// The model under `parent` at `transform`.
    pub fn spawn_model(&self, commands: &mut Commands, transform: Transform, parent: Option<Entity>) -> Entity {
        let e = commands.spawn((transform, Visibility::Inherited, Name::new("grenade"))).id();
        if let Some(p) = parent {
            commands.entity(e).insert(ChildOf(p));
        }
        for (mesh, mat, offset) in &self.parts {
            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::from_translation(*offset), ChildOf(e)));
        }
        e
    }
}

/// Every grenade type the loaded levels define (see SQUAD_GRENADES), loaded once per map.
#[derive(Resource, Default)]
pub struct GrenadeKits(pub Vec<GrenadeKit>);

impl GrenadeKits {
    /// The type labelled `label` (any case).
    pub fn find(&self, label: &str) -> Option<usize> {
        self.0.iter().position(|k| k.def.label.eq_ignore_ascii_case(label))
    }
}

#[derive(Component)]
struct Grenade {
    kind: usize,
    velocity: Vec3,
    fuse: f32,
    /// the fuse runs once the grenade has come down
    landed: bool,
    spin: Vec3,
    /// the character who threw it (its blast hurts them by SELF_DAMAGE)
    thrower: usize,
    /// a Roller: rolls this way (flat, unit) along the ground once it's down; its rolling sound
    /// is due again in this many seconds
    rolling: Option<Vec3>,
    roll_sound: f32,
    /// when it left the hand (sim time; BF_GRENADE_LOG prints the flight to the first contact)
    launched: f32,
}

/// The red damage tint: seconds since the player was hurt by a blast (TINT_TIME or more: none).
/// (play_energy.rs sets it when a bolt strikes the player.)
#[derive(Resource)]
pub struct ScreenTint(pub f32);

/// The tint's quad in front of the camera (multiplied into the 3D picture).
#[derive(Component)]
struct TintQuad(Vec<Handle<StandardMaterial>>);

/// A canister left where it went off while its effects burn (see `stays`): seconds left, and its
/// type's label for BF_GRENADE_LOG ("canister gone").
#[derive(Component)]
struct Spent {
    left: f32,
    label: String,
}

/// A grenade in the hand between the throw clip's reach and release events, and its type.
#[derive(Resource, Default)]
struct Held(Option<(Entity, usize)>);

/// Blast sounds waiting for their effect type's delay: (seconds left, sound id, volume); and
/// blast decals waiting for the fireball to clear: (seconds left, decal, where).
#[derive(Resource, Default)]
struct DelayedBlastParts(Vec<(f32, u32, f32)>, Vec<(f32, u32, Vec3)>, Vec<PendingDamage>);

/// A blast's damage waiting DAMAGE_DELAY: the grenade type, where it went off, who threw it.
struct PendingDamage {
    left: f32,
    kind: usize,
    at: Vec3,
    thrower: usize,
}

/// The grenades' frame systems (throws, flight, blasts, tint): play_gas.rs's poison runs after
/// them, so a cloud starts on the frame after its blast.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GrenadeSystems;

pub fn plugin(app: &mut App) {
    app.init_resource::<Held>()
        .init_resource::<DelayedBlastParts>()
        .insert_resource(ScreenTint(TINT_TIME))
        .add_systems(OnEnter(AppState::Playing), load_kits.after(setup))
        .add_systems(Update, (stock_inventory, launch_grenades, hold_grenade, fly_grenades, super::energy::strike, tint).chain().in_set(GrenadeSystems)
            .after(update_player).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// The squad's grenade types (SQUAD_GRENADES, those the loaded levels define), their ALE
/// effects compiled and their materials' render pipelines built ahead (`ale_fx::warm_up`).
fn load_kits(mut commands: Commands, mut game: ResMut<GameData>, mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
             mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>,
             ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut held: ResMut<Held>, mut tint: ResMut<ScreenTint>,
             mut delayed: ResMut<DelayedBlastParts>) {
    held.0 = None;
    tint.0 = TINT_TIME;
    delayed.0.clear();
    delayed.1.clear();
    delayed.2.clear();
    let usable = |d: &WeaponDef| d.projectile != 0 && d.icon != 0 && !d.label.is_empty() && !d.label.starts_with("h_");
    let defs: Vec<WeaponDef> = SQUAD_GRENADES.iter().filter_map(|n| game.0.weapons.get(n)).filter(|d| usable(d)).cloned().collect();
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    let kits: Vec<GrenadeKit> = defs.into_iter().map(|d| GrenadeKit::load(&mut game.0, &mut assets, d)).collect();
    if let Some(mut ale) = ale {
        for k in &kits {
            for &e in k.blast_fx.effects.iter().chain(&k.trail_fx.effects).chain([&k.blast_fx.light]).filter(|&&e| e != 0) {
                if let Some(fx) = ale.load_recorded(&mut game.0, &mut images, &mut materials, e) {
                    bf_viewer::ale_fx::warm_up(&mut commands, &ale, &fx);
                }
            }
        }
    }
    println!("{} grenade types: {}", kits.len(), kits.iter().map(|k| k.def.label.as_str()).collect::<Vec<_>>().join(", "));
    commands.insert_resource(GrenadeKits(kits));
}

/// The squad's grenades, once a character is loaded (and after a respawn): the main game starts
/// with START_GRENADES Frags; the test map with a full stack (stack-limit) of every type.
/// Test hook: BF_TEST_GRENADE_TYPE=<label> (e.g. Sonic) selects that type, giving a stack of it
/// if none is carried.
fn stock_inventory(mut player: ResMut<Player>, kits: Option<Res<GrenadeKits>>, test: Option<Res<super::testmap::TestMap>>) {
    let Some(kits) = kits else { return };
    if player.loaded.is_none() || player.grenades.len() == kits.0.len() {
        return;
    }
    player.grenades = kits.0.iter().map(|k| match (test.is_some(), k.def.name == FRAG) {
        (true, _) => k.def.stack_limit.max(1),
        (false, true) => START_GRENADES,
        _ => 0,
    }).collect();
    // the main game's levels without the Frag: the first type
    if test.is_none() && !player.grenades.iter().any(|&n| n > 0) {
        if let Some(n) = player.grenades.first_mut() {
            *n = START_GRENADES;
        }
    }
    let first = player.grenades.iter().position(|&n| n > 0).unwrap_or(0);
    if let Some(k) = std::env::var("BF_TEST_GRENADE_TYPE").ok().and_then(|l| kits.find(&l)) {
        if player.grenades[k] <= 0 {
            player.grenades[k] = kits.0[k].def.stack_limit.max(1);
        }
        player.item = Item::Grenade(k);
    } else if matches!(player.item, Item::Grenade(_)) {
        player.item = Item::Grenade(first);
    }
}

/// A grenade let go this frame (play.rs's throw and place): where, how fast, its type, and
/// whether it's on the ground already, its fuse running (BF_TEST_EXPLOSION's).
pub struct Thrown {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub kind: usize,
    pub landed: bool,
}

/// Spawn the grenades let go this frame: the one in the hand flies on (its trail with it), or
/// drops at the feet (a placed type: the Roller rolls off ahead once down).
fn launch_grenades(mut commands: Commands, mut player: ResMut<Player>, kits: Option<Res<GrenadeKits>>, mut held: ResMut<Held>,
                   mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut next_test: Local<f32>) {
    let Some(kits) = kits else { return };
    let mut throws = std::mem::take(&mut player.thrown);
    // test hook: BF_TEST_EXPLOSION=1 drops the selected type 6 m ahead every 1.5 s, going off at
    // once
    let test = std::env::var("BF_TEST_EXPLOSION").is_ok();
    let selected = match player.item { Item::Grenade(k) => k, _ => 0 };
    if test && player.sim_time >= *next_test && !kits.0.is_empty() {
        *next_test = player.sim_time + 1.5;
        let (x, z) = (player.position.x, player.position.z - 6.0);
        throws.push(Thrown { pos: Vec3::new(x, floor_y(x, z, player.position.y + GROUND + 1.0) + GRENADE_RADIUS, z), velocity: Vec3::ZERO,
                             kind: selected.min(kits.0.len() - 1), landed: true });
    }
    for t in throws {
        let Some(kit) = kits.0.get(t.kind) else { continue };
        let e = match held.0.take() {
            // the one in the hand, with its trail
            Some((e, kind)) if kind == t.kind && !t.landed => {
                commands.entity(e).remove::<ChildOf>().insert(Transform::from_translation(t.pos));
                e
            }
            other => {
                held.0 = other;
                let e = kit.spawn_model(&mut commands, Transform::from_translation(t.pos), None);
                if !test {
                    trail(&mut commands, &mut player, kit, e, ale.as_deref_mut());
                }
                e
            }
        };
        let rolling = (kit.def.function_type == ROLLING_BOMB && !t.landed).then(|| Quat::from_rotation_y(player.yaw) * Vec3::NEG_Z);
        let spin = if kit.placed() { Vec3::ZERO } else { t.velocity.cross(Vec3::Y).normalize_or(Vec3::X) * -12.0 };
        let fuse = if t.landed { 0.0 } else { kit.def.fuse.max(0.0) };
        commands.entity(e).insert(Grenade { kind: t.kind, velocity: t.velocity, fuse, landed: t.landed, spin, thrower: player.character, rolling,
                                            roll_sound: 0.0, launched: player.sim_time });
    }
}

/// Show the grenade in the throwing hand while the throw has it; its trail and hiss start then.
fn hold_grenade(mut commands: Commands, mut player: ResMut<Player>, kits: Option<Res<GrenadeKits>>, mut held: ResMut<Held>,
                mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>) {
    let Some(kits) = kits else { return };
    let want = player.throwing.as_ref().filter(|t| t.grabbed && !t.released).map(|t| t.kind);
    let hand = player.loaded.as_ref().and_then(|l| l.throw_hand.map(|(bone, point)| (l.joints[bone], point)));
    match (want, held.0, hand) {
        (Some(kind), None, Some((joint, point))) => {
            let Some(kit) = kits.0.get(kind) else { return };
            let e = kit.spawn_model(&mut commands, Transform::from_translation(point), Some(joint));
            trail(&mut commands, &mut player, kit, e, ale.as_deref_mut());
            held.0 = Some((e, kind));
        }
        (None, Some((e, _)), _) => {
            commands.entity(e).despawn();
            held.0 = None;
        }
        _ => {}
    }
}

/// The attached effect type on grenade `e`: its ALE effects (grenade_trail's smoke) riding on it
/// and its sounds (the Frag's hiss 10318b29).
fn trail(commands: &mut Commands, player: &mut Player, kit: &GrenadeKit, e: Entity, ale: Option<&mut bf_viewer::ale_fx::AleAssets>) {
    if let Some(ale) = ale {
        for (i, &effect) in kit.trail_fx.effects.iter().enumerate() {
            if let Some(fx) = ale.cached_recorded(effect) {
                commands.spawn((Transform::default(), Visibility::default(),
                                bf_viewer::ale_fx::AleEffect::new(fx, 0.0, 0x7A11_0000 ^ e.index() ^ i as u32), ChildOf(e)));
            }
        }
    }
    for &(id, _, _, _) in &kit.trail_fx.sounds {
        player.sound_queue.push((id, 0.8));
    }
}

/// Grenade flight: gravity, bounces off the ground, the pillars and the map's objects, the
/// blast when the fuse ends (or at the first contact, timer 0).
#[allow(clippy::too_many_arguments)]
fn fly_grenades(
    mut commands: Commands,
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    mut game: ResMut<GameData>,
    kits: Option<Res<GrenadeKits>>,
    mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
    (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
    mut grenades: Query<(Entity, &mut Grenade, &mut Transform)>,
    (mut blasts, mut decals, mut delayed, mut tint, mut gas, mut bolts): (ResMut<super::pickups::Blasts>, ResMut<super::fx::DecalRequests>, ResMut<DelayedBlastParts>,
                                                               ResMut<ScreenTint>, ResMut<super::gas::GasClouds>,
                                                               ResMut<super::energy::BoltRequests>),
    test: Option<Res<super::testmap::TestMap>>,
    mut detonated: Local<bool>,
    mut trails: Query<(&ChildOf, &mut bf_viewer::ale_fx::AleEffect)>,
    mut spent: Query<(Entity, &mut Spent)>,
) {
    let dt = frame_dt(&time);
    let p = &mut *player;
    // canisters left burning (see `stays`) go once their effects have run
    for (e, mut s) in &mut spent {
        s.left -= dt;
        if s.left <= 0.0 {
            commands.entity(e).despawn();
            if std::env::var("BF_GRENADE_LOG").is_ok() {
                println!("t {:.2}: {} canister gone", p.sim_time, s.label);
            }
        }
    }
    // blast sounds and decals whose delay is up
    delayed.0.retain_mut(|(left, id, volume)| {
        *left -= dt;
        if *left <= 0.0 {
            p.sound_queue.push((*id, *volume));
        }
        *left > 0.0
    });
    delayed.1.retain_mut(|(left, decal, at)| {
        *left -= dt;
        if *left <= 0.0 {
            decals.0.push((*decal, *at, GRENADE_DECAL_SCALE));
        }
        *left > 0.0
    });
    let Some(kits) = kits else { return };
    // blasts' damage whose delay is up: everyone in range is hurt. Damage max at the centre,
    // nothing at the radius; the thrower takes SELF_DAMAGE of a roll of min..max anywhere inside
    // it (the test map's instant kill: the squad dies to any blast that reaches them)
    let instant = test.as_ref().is_some_and(|t| t.instant_kill);
    let mut due = vec![];
    delayed.2.retain_mut(|d| {
        d.left -= dt;
        if d.left <= 0.0 {
            due.push((d.kind, d.at, d.thrower));
        }
        d.left > 0.0
    });
    for (kind, at, thrower) in due {
        let Some(kit) = kits.0.get(kind) else { continue };
        let (radius, max, min) = (kit.blast.blast_radius, kit.blast.damage, kit.blast.damage_min.min(kit.blast.damage));
        let roll = p.random(1000) as f32 / 1000.0;
        let (leader, rest) = (std::iter::once((&mut *p, false, true)), squad.0.iter_mut().map(|u| (u, instant, false)));
        for (u, kill, controlled) in leader.chain(rest) {
            let d = u.position.distance(at);
            if d >= radius || u.dead {
                continue;
            }
            let k = 1.0 - d / radius;
            let away = Vec3::new(u.position.x - at.x, 0.0, u.position.z - at.z).normalize_or(Vec3::X);
            // (times the character's factor for the explosion's damage-type)
            let damage = game.0.damage_factor(CHARACTERS[u.character], kit.blast.damage_type)
                * if u.character == thrower { SELF_DAMAGE * (min + (max - min) * roll) } else { max * k };
            let damage = if kill && u.character != thrower { u.health.max(damage) } else { damage };
            hurt(u, &game.0, damage, HURT_CHATTER, (away * 6.0 + Vec3::Y * 4.0) * k, u.position + Vec3::Y * 1.0, -1);
            if controlled {
                tint.0 = 0.0;
            }
            if std::env::var("BF_COMBAT_LOG").is_ok() {
                println!("{} blast at {d:.1} m: {} takes {damage:.1} -> {:.1} / {:.0}", kit.def.label, CHARACTERS[u.character], u.health, u.max_health);
            }
        }
    }
    // test hook: BF_TEST_DETONATE=<s> sets off every grenade out at that time (the Sentry has no
    // trigger yet)
    let detonate = std::env::var("BF_TEST_DETONATE").ok().and_then(|v| v.parse::<f32>().ok())
        .is_some_and(|at| p.sim_time >= at && !*detonated);
    *detonated |= detonate;
    for (e, mut g, mut tr) in &mut grenades {
        let Some(kit) = kits.0.get(g.kind) else { continue };
        if detonate {
            g.fuse = 0.0;
            g.landed = true;
        }
        if g.landed {
            g.fuse -= dt;
        }
        if g.landed && g.fuse <= 0.0 {
            let at = tr.translation;
            let burn = blast(&mut commands, &mut game.0, ale.as_deref_mut(), &mut images, &mut materials, kit, at, p, &mut delayed);
            // its damage, DAMAGE_DELAY later. None from a blast without damage (the Light), whose
            // canister stays where it lies while its effects burn (see `stays`); one whose damage
            // is dealt over time (h_04ea9251 > 0: the Gas, whose recording shows no damage at
            // once) leaves its poison cloud instead (play_gas.rs)
            let (radius, max) = (kit.blast.blast_radius, kit.blast.damage);
            if stays(kit) {
                commands.entity(e).remove::<Grenade>().insert(Spent { left: burn, label: kit.def.label.clone() });
                for (parent, mut fx) in &mut trails {
                    if parent.parent() == e {
                        fx.active = false;
                    }
                }
                continue;
            }
            commands.entity(e).despawn();
            if max <= 0.0 || radius <= 0.0 {
                continue;
            }
            if kit.blast.damage_time > 0.0 {
                gas.release(g.kind, at, g.thrower, p.sim_time);
                continue;
            }
            // loose pickups are thrown by blasts that hurt
            blasts.0.push((at, radius));
            // the Energy's damage comes with its bolts (play_energy.rs)
            if super::energy::releases_bolts(kit) {
                bolts.0.push((g.kind, Vec3::new(at.x, floor_y(at.x, at.z, at.y + 0.5), at.z), g.thrower));
                continue;
            }
            delayed.2.push(PendingDamage { left: DAMAGE_DELAY, kind: g.kind, at, thrower: g.thrower });
            continue;
        }
        g.velocity.y -= GRENADE_GRAVITY * dt;
        let mut pos = tr.translation + g.velocity * dt;
        let floor = floor_y(pos.x, pos.z, tr.translation.y) + GRENADE_RADIUS;
        let mut hit = false;
        if pos.y < floor {
            pos.y = floor;
            hit = true;
            if !g.landed {
                if std::env::var("BF_GRENADE_LOG").is_ok() {
                    println!("t {:.2}: {} down {:.2} s after leaving the hand, {:.1} m from the thrower", p.sim_time, kit.def.label,
                             p.sim_time - g.launched, Vec2::new(pos.x - p.position.x, pos.z - p.position.z).length());
                }
                // squadmates close by dive away from a thrown one (EVT_GRENADE_NEAR ->
                // GOAL_DIVE); not from one set down (a Roller, a Sentry)
                if !kit.placed() {
                    for m in squad.0.iter_mut().filter(|m| !m.dead && m.position.distance(pos) < DIVE_RADIUS) {
                        m.dive_from = Some(pos);
                    }
                }
            }
            g.landed = true;
            if let Some(dir) = g.rolling {
                // down: it rolls off along the ground
                let flat = Vec2::new(g.velocity.x, g.velocity.z);
                let v = if flat.length() > 0.5 { flat.normalize() * ROLL_SPEED } else { Vec2::new(dir.x, dir.z) * ROLL_SPEED };
                g.velocity = Vec3::new(v.x, 0.0, v.y);
                g.spin = g.velocity.cross(Vec3::Y).normalize_or(Vec3::X) * -(ROLL_SPEED / GRENADE_RADIUS).min(40.0);
                // its rolling sound, again and again while it rolls, fading with distance
                g.roll_sound -= dt;
                if g.roll_sound <= 0.0 && kit.def.object_sound != 0 {
                    g.roll_sound = ROLL_SOUND_EVERY;
                    let d = pos.distance(p.position);
                    let volume = 0.8 * (1.0 - (d - ROLL_SOUND_NEAR) / (ROLL_SOUND_FAR - ROLL_SOUND_NEAR)).clamp(0.0, 1.0);
                    if volume > 0.0 {
                        p.sound_queue.push((kit.def.object_sound, volume));
                    }
                }
            } else if g.velocity.y < 0.0 {
                g.velocity.y = -g.velocity.y * RESTITUTION;
                g.velocity.x *= GROUND_FRICTION;
                g.velocity.z *= GROUND_FRICTION;
                g.spin *= GROUND_FRICTION;
                if g.velocity.y < 0.4 {
                    g.velocity.y = 0.0;              // resting: roll to a stop
                }
            }
        }
        for c in pillars() {
            let half = 0.3 + GRENADE_RADIUS;
            let d = pos - c;
            if d.x.abs() < half && d.z.abs() < half && pos.y < c.y + 1.5 {
                // push out along the shallower axis and reflect
                hit = true;
                if half - d.x.abs() < half - d.z.abs() {
                    pos.x = c.x + half * d.x.signum();
                    g.velocity.x = -g.velocity.x * 0.5;
                } else {
                    pos.z = c.z + half * d.z.signum();
                    g.velocity.z = -g.velocity.z * 0.5;
                }
            }
        }
        // the map's objects: push out and bounce off
        if let Some(a) = world::arena() {
            let at = Vec2::new(pos.x, pos.z);
            let q = a.push_out(at, GRENADE_RADIUS, pos.y - GRENADE_RADIUS, pos.y + GRENADE_RADIUS, 0.0);
            if q != at {
                hit = true;
                let n = (q - at).normalize_or_zero();
                let v = Vec2::new(g.velocity.x, g.velocity.z);
                let vn = v.dot(n);
                if vn < 0.0 {
                    let v = v - n * vn * 1.5;
                    g.velocity.x = v.x;
                    g.velocity.z = v.y;
                }
                pos.x = q.x;
                pos.z = q.y;
            }
        }
        // timer 0: off at the first contact, where it is (no bounce)
        if hit && kit.on_contact() {
            g.landed = true;
            g.fuse = 0.0;
        }
        tr.translation = pos;
        let spin = g.spin * dt;
        if spin.length_squared() > 1e-8 {
            tr.rotation = Quat::from_scaled_axis(spin) * tr.rotation;
        }
    }
}

/// Whether a grenade's canister stays where it lies after it goes off, until its effects have
/// burnt out: one whose explosion does no damage (the Light, Damage 0-0: the recording's flares
/// lie under their beams, lg 1062-1085). The data has no flag for it (the Light's definition
/// differs from the Frag's only in names, its item kind h_e5f51266 and its hitpoints); the
/// other types' recordings show nothing left after the blast. An inference.
fn stays(kit: &GrenadeKit) -> bool {
    kit.blast.damage <= 0.0
}

/// A blast at `at`: the explosion's effect type (its ALE effects and light effect, run once on
/// the ground below, and its sounds, after their delay) and the bullet's impact sound; its
/// decal on the ground. Returns how long its effects run (s; the Light's phosphor_grenade: its
/// 30 s emitters and their last particles' 2 s).
#[allow(clippy::too_many_arguments)]
fn blast(commands: &mut Commands, game: &mut Game, ale: Option<&mut bf_viewer::ale_fx::AleAssets>, images: &mut Assets<Image>,
         materials: &mut Assets<StandardMaterial>, kit: &GrenadeKit, at: Vec3, p: &mut Player, delayed: &mut DelayedBlastParts) -> f32 {
    let ground = Vec3::new(at.x, floor_y(at.x, at.z, at.y + 0.5), at.z);
    let mut burn: f32 = 0.0;
    if let Some(ale) = ale {
        for &effect in kit.blast_fx.effects.iter().chain([&kit.blast_fx.light]).filter(|&&e| e != 0) {
            if let Some(fx) = ale.load_recorded(game, images, materials, effect) {
                let life = fx.duration();
                burn = burn.max(life);
                let seed = (ground.x * 977.0 + ground.z * 131.0) as u32;
                commands.spawn((Transform::from_translation(ground), Visibility::default(),
                                bf_viewer::ale_fx::AleEffect::once(fx, 0.0, seed), AleExpire(life)));
            }
        }
    }
    let volume = (1.0 - at.distance(p.position) / BLAST_HEARD).clamp(BLAST_QUIET, 1.0);
    for &(id, delay, _, _) in &kit.blast_fx.sounds {
        if delay > 0.0 {
            delayed.0.push((delay, id, volume));
        } else {
            p.sound_queue.push((id, volume));
        }
    }
    if kit.blast.impact_sound != 0 {
        p.sound_queue.push((kit.blast.impact_sound, volume));
    }
    if kit.blast.decal != 0 {
        delayed.1.push((DECAL_DELAY, kit.blast.decal, ground));
    }
    if std::env::var("BF_GRENADE_LOG").is_ok() {
        println!("t {:.2}: {} blast at {ground:.2}, {} effects + light {:08x}, sounds {:x?} + {:08x}, decal {:08x}, effects run {burn:.1} s", p.sim_time,
                 kit.def.label, kit.blast_fx.effects.len(), kit.blast_fx.light, kit.blast_fx.sounds, kit.blast.impact_sound, kit.blast.decal);
    }
    burn
}

/// sRGB stored value -> linear light.
fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

/// The linear-light factor that scales a picture's stored (sRGB) values by `s`: exact for a
/// stored value of TINT_GREY (the captures' ground is ~0.35-0.45), close round it. (The picture
/// is blended in linear light; the console scaled the stored values.)
fn tint_factor(s: f32) -> f32 {
    srgb_to_linear(TINT_GREY * s) / srgb_to_linear(TINT_GREY)
}

/// The red damage tint: a quad just in front of the camera, multiplied into the 3D picture (the
/// HUD is drawn after it), its green and blue TINT_LOW of the picture's stored values right after
/// the hurt, back to 1 over TINT_TIME. (The picture blends in linear light: a stored-value factor
/// s is the linear factor srgb(s), what `Color::srgb` gives.) One material per TINT_STEPS level,
/// swapped in (a material's new colour reaches the screen a frame late, a swap at once); the
/// quad stays shown, white when there's no tint (shown on the hurt, it came a frame late too).
fn tint(mut commands: Commands, time: Res<Time>, mut tint: ResMut<ScreenTint>, mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut quads: Query<(&TintQuad, &mut MeshMaterial3d<StandardMaterial>)>,
        camera: Query<Entity, With<MainCamera>>) {
    // the recording's factor at this time (linear between its game frames)
    let f = tint.0 * 30.0;
    let i = (f as usize).min(TINT_CURVE.len() - 1);
    let s = TINT_CURVE[i] + (TINT_CURVE[(i + 1).min(TINT_CURVE.len() - 1)] - TINT_CURVE[i]) * f.fract().min(1.0);
    let k = ((s - TINT_LOW) / (1.0 - TINT_LOW)).clamp(0.0, 1.0);
    tint.0 += frame_dt(&time);
    let Ok((quad, mut mat)) = quads.single_mut() else {
        if let Ok(cam) = camera.single() {
            let steps = (0..=TINT_STEPS).map(|i| {
                let s = tint_factor(TINT_LOW + (1.0 - TINT_LOW) * i as f32 / TINT_STEPS as f32);
                materials.add(StandardMaterial { base_color: Color::linear_rgb(1.0, s, s), unlit: true, fog_enabled: false,
                                                 alpha_mode: AlphaMode::Multiply, cull_mode: None, ..default() })
            }).collect::<Vec<_>>();
            commands.spawn((Mesh3d(meshes.add(Rectangle::new(4.0, 4.0))), MeshMaterial3d(steps[0].clone()), TintQuad(steps),
                            NotShadowCaster, Transform::from_xyz(0.0, 0.0, -TINT_DEPTH), Visibility::Inherited, ChildOf(cam)));
        }
        return;
    };
    let step = &quad.0[((k * TINT_STEPS as f32).round() as usize).min(TINT_STEPS)];
    if mat.0 != *step {
        mat.0 = step.clone();
    }
}
