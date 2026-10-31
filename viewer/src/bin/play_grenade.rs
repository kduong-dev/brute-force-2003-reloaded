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
//! Thrown types: hold the button to charge (the HUD meter, full in 0.625 s), let go to throw (15
//! degrees above the aim, at the charge's push over the type's mass: `throw_velocity`, #161): the
//! count drops at once (one game frame after the button in the recordings), the stance's throw
//! clip takes the grenade in hand at its reach event (its trail and hiss start then: ~0.33 s
//! after the button) and lets go at its release event. It bounces silently and goes off when its
//! fuse ends. Place-on-ground types (Roller, Sentry, Thermo) are set down at the feet with the
//! stance's place_hi clip (no meter, the count drops at the press); the Roller then hops off after
//! enemies (play_roller.rs).
//!
//! The fuse (the grenade update 0x149420) counts only once the grenade is out of the hand, armed
//! by its first contact (`Grenade::landed`), and while no cutscene holds the world (its +0xc50 bit
//! 4, a script's 122 / 123: play_script.rs `in_cutscene`), and goes off when it's out.
//!
//! The blast plays the effect type's ALE effects and light (ale_fx.rs) and sound and the impact
//! sound, leaves the decal, and hurts everyone in its radius (and throws loose pickups) unless
//! it does no damage or deals it over time. Hurt by it, the player's 3D view goes red for a
//! moment (see `TINT_LOW`).
//!
//! The Gas's damage over time is play_gas.rs's (its blast is handed over there, see
//! `fly_grenades`), the Energy's comes with its bolts (play_energy.rs), the Sonic's with its
//! ring (play_sonic.rs). The Light does no damage: its canister stays where it lies while
//! phosphor_grenade and light_phosphor burn (30 s, see `stays`). The Roller's seeking is
//! play_roller.rs's: its proximity rule sets its fuse to 0. The Sentry's trigger (a hostile within
//! 3 m, no friend near; or a shot) is play_sentry.rs's: it sets a mine that's due off at once
//! (`Grenade::set_off`).

use super::*;
use super::damage::{footage_squad_step, Difficulty, Hit};
use bf_viewer::bf::character::EffectType;

/// A throw's impulse at no charge and its charge term (N s): FUN_00122ea0 returns 16 + 7 x for
/// x = p^2 (0x3a4ff4 16, 0x3a501c 7), the release event's push (`throw_velocity`). The AI's 12
/// + 18 x (0x3a5020, 0x3a5024, with char+0x748 bit 2, set only by its grenade goal at 0x353ce) is
/// the enemies' (#110, #78), not done.
const THROW_IMPULSE: f32 = 16.0;
const THROW_IMPULSE_CHARGE: f32 = 7.0;
/// How far above the aim a throw leaves (rad): the release event (FUN_00117660 at 0x117be4)
/// turns (0, sin 15, -cos 15) (0x3e84816f 0.2588, 0xbf774539 -0.9659) by the aim frame
/// (char+0x5bc, FUN_0001f9c0): 15 degrees above the aim's forward, whatever the charge.
const THROW_UP: f32 = 15.0 * std::f32::consts::PI / 180.0;
/// The release moves the grenade this far (m) along (0, sin 15, -cos 15) in the world's axes
/// before the push (0x3ecccccd 0.4): the offset isn't turned by the aim, a quirk of the code
/// (#161's leads), so it goes 0.39 m toward the world's -z and 0.10 m up whichever way it's
/// thrown.
const THROW_NUDGE: f32 = 0.4;
/// The physics world's gravity (m/s^2): FUN_0016a250 makes the Ipion world with (0, -9.81, 0)
/// (0x16a4a7, 0x16a4c7).
const GRENADE_GRAVITY: f32 = 9.81;
pub(super) const GRENADE_RADIUS: f32 = 0.06;
/// bounce: vertical restitution, horizontal speed kept per bounce (the demo's; the friction
/// fitted so a fast throw comes to rest near where it lands, as the recording's does: its trail
/// puffs from about where it came down, frag/a 600-690)
const RESTITUTION: f32 = 0.35;
const GROUND_FRICTION: f32 = 0.3;
/// h_1ee2f4ed: IOU_PLACE_ON_GROUND (Roller, Sentry, Thermo; the XBE's IOU_ enum)
const USE_PLACE: i64 = 2;
/// A level's item spawn element (h_1d1d4f05: the inventory-objects it puts down, the arena's
/// PICKUP_ITEM; as pickups, or set down live with h_1cb7d5f2: the tutorial's and e16's, e34's and
/// e40's mines, m09_b's Thermo): `load_kits` makes a kit for the grenade and explosive types among
/// them.
const ITEM_SPAWN: u32 = 0x1D1D_4F05;
/// While it rolls, the Roller's object sound (h_19dbc65d) plays again every ROLL_SOUND_EVERY s
/// (the recording: a loop every ~0.97 s), heard fully within ROLL_SOUND_NEAR m and fading to
/// nothing at ROLL_SOUND_FAR m (the sound's own `<Sound falloff=5 h_fd40e332=25>` in
/// sounds-mp1.xml, read as full-volume and silent distances).
const ROLL_SOUND_EVERY: f32 = 0.97;
const ROLL_SOUND_NEAR: f32 = 5.0;
const ROLL_SOUND_FAR: f32 = 25.0;
/// The thrower's own damage from their blast: SELF_DAMAGE x a roll of the explosion's Damage
/// min..max, at any distance within the radius. Measured: the Frag recording's six blasts each
/// took 12.5-13.3 HP of Tex's 115 (~11%, campaign) near or far, with no falloff; 58.5-65 x 0.2
/// fits. The game's rule gives 0.21 (half the roll, h_011cb154 false, x0.7 own side x0.6 the
/// squad's step on STANDARD: todo/drafts/comment-112-blast-rules.md), not in yet (#112). The
/// amount holds STANDARD's step already (`hurt_by_blast` divides it out before the step).
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
pub(super) const DAMAGE_DELAY: f32 = 0.1;
/// A blast's decal is drawn with its width / height read as half sizes, as the blood decals'
/// are (play_fx.rs DECAL_SCALE, fitted to the blood captures): the Frag's scorch h_ff1b711e
/// (3 x 3) is a 6 m quad, whose soft texture is half dark over ~3.7 m - the recording's scorch
/// is half dark over ~4.1 m (the reference agent's measurement). At the data's size it came out
/// ~1.9 m.
pub(super) const GRENADE_DECAL_SCALE: f32 = 2.0;
/// The decal is laid this long after the blast, once the fireball has gone (the recording's
/// bright part is gone by 0.63 s, and the scorch is first seen under the smoke). The demo's
/// choice: laid at once, the dark decal showed through the added fireball as a hard dark disc.
pub(super) const DECAL_DELAY: f32 = 0.7;
/// How far in front of the camera the tint quad sits (m; inside the near plane's 0.1 is cut).
const TINT_DEPTH: f32 = 0.15;
/// The tint's levels (materials) over TINT_TIME.
const TINT_STEPS: usize = 16;
/// The stored value the tint's factor is exact for (see `tint_factor`).
const TINT_GREY: f32 = 0.4;
/// Blast sounds are this loud at the blast, falling to BLAST_QUIET at BLAST_HEARD m (the demo's).
pub(super) const BLAST_HEARD: f32 = 60.0;
pub(super) const BLAST_QUIET: f32 = 0.3;
/// The squad's grenade definitions, in the game's store order, (category, index): the group-type
/// and h_e5f51266 of each (FUN_00149d30's key; the store and the item box run through it in that
/// order, FUN_0014c230): Frag (2, 0; #75), Gas (2, 1; #76), Sonic (2, 4; #81), Energy (2, 5; #74),
/// Light (2, 32; #77), Sentry (3, 1; #80), Roller (3, 4; #79). The kits are kept in that order
/// (`load_kits` sorts them by it), which is the order the demo's grenade key (T) steps through
/// them and the first one carried the item box shows: e34's Sonic, e16's and e23's Gas.
/// Some labels have several definitions (the Sentry three, the Roller two): these are the ones
/// the recordings show; a level that defines another of a label its squad's placements carry
/// takes that one (`load_kits`: m14_c's and m14_d's Sentry h_f73de83d, (3, 3)). The squad carries
/// only these: the other labelled grenade with an icon, the Molotov (e01's h_f42e0faa), is the
/// mutants' weapon (#78); its definition is read like any other (`Game::weapons`) for an AI
/// thrower to use.
const SQUAD_GRENADES: [u32; 7] = [0x060F_95F7, 0xE8C6_7904, 0xEC0B_F28F, 0x0CC3_C2C8, 0xFD1A_966D, 0xE5F1_F063, 0xFBA7_C475];

/// The charge a throw's power counts from: the player controller's 0.3 (0x3a50b0).
const POWER_FROM: f32 = 0.3;
/// The least power a throw has (0x1348c0-0x134945: the clamp's low end, 0.01).
const POWER_LEAST: f32 = 0.01;

/// A throw's power from the charge `c` the button was let go at (the player controller 0x1332a0
/// at 0x1348fd-0x134941): clamp((c - 0.3) / 0.7, 0.01, 1) (0.3 at 0x3a50b0, 1 / 0.7 the double at
/// 0x3a54c8), kept at char+0x7f0 by the item use (FUN_000abf60 -> FUN_00121170). A hold under
/// 0.1875 s throws at 0.01: the meter's bottom 30% throws no farther than a tap.
pub fn throw_power(c: f32) -> f32 {
    ((c - POWER_FROM) / (1.0 - POWER_FROM)).clamp(POWER_LEAST, 1.0)
}

/// A thrown grenade's velocity at the throw clip's release event (FUN_00117660 at 0x117be4): the
/// push J = THROW_IMPULSE + THROW_IMPULSE_CHARGE x p^2 (FUN_00122ea0, p the throw's `power`,
/// `throw_power`) along the aim frame `aim` x (0, sin 15, -cos 15) (THROW_UP), given to the
/// grenade's first body at the hand's point through Ipion's push (FUN_002b3bc0), whose speed
/// changes by J over the body's mass: the item type's `mass` (`WeaponDef::mass`, h_1a19ff7a: the
/// Frag and the Light 0.9, the Energy, the Gas and the Sonic 1.0; FUN_001590a0 builds the body
/// with it). So a Frag leaves at 17.8-25.6 m/s, an Energy at 16.0-23.0. No ray and no crosshair
/// point in it: the game doesn't aim the throw at the crosshair. The off-centre push's spin isn't
/// kept (the demo spins its grenades its own way). Not done: Ipion's speed damping, read as 0.1
/// in FUN_00158f80's body template (-10% of the speed a second; an inference from Ipion's
/// defaults, #161's leads: #119 take01 fits with or without it, 25.1 against 26.3 m).
pub fn throw_velocity(aim: Quat, power: f32, mass: f32) -> Vec3 {
    let impulse = THROW_IMPULSE + THROW_IMPULSE_CHARGE * power * power;
    let dir = aim * Vec3::new(0.0, THROW_UP.sin(), -THROW_UP.cos());
    // (a definition without a mass: 1 kg, `WeaponDef::mass`'s own default)
    dir * impulse / if mass > 0.0 { mass } else { 1.0 }
}

/// The release's move of the grenade before its push: THROW_NUDGE along (0, sin 15, -cos 15) in
/// the world's axes (0x117be4-0x117e60, unturned).
pub fn throw_nudge() -> Vec3 {
    Vec3::new(0.0, THROW_UP.sin(), -THROW_UP.cos()) * THROW_NUDGE
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
    /// the model's bounding sphere (centre in the model's frame, radius; m), from its meshes'
    /// vertices: what a shot has to pass through to strike it (play_sentry.rs)
    pub bounds: (Vec3, f32),
    /// the radius its splash goes by (#5, play_liquids.rs `object_enters`): its mesh archetype's
    /// h_1ab09530 (`Game::archetype_sphere`: the grenades' 0.081 m, the Sentry's 0.33 m; what
    /// FUN_00236670 gives its scene node), else `bounds`' (a compound's isn't in the data: the
    /// demo's)
    pub splash_radius: f32,
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
        let (mut lo, mut hi) = (Vec3::MAX, Vec3::MIN);
        for (mesh, _, offset) in &parts {
            if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(v)) = assets.meshes.get(mesh).and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)) {
                for p in v {
                    lo = lo.min(Vec3::from(*p) + *offset);
                    hi = hi.max(Vec3::from(*p) + *offset);
                }
            }
        }
        let bounds = if lo.x <= hi.x { ((lo + hi) * 0.5, ((hi - lo) * 0.5).length().max(GRENADE_RADIUS)) } else { (Vec3::ZERO, GRENADE_RADIUS) };
        let sphere = game.archetype_sphere(def.archetype);
        let splash_radius = sphere.map_or(bounds.1, |s| s.1);
        info!("grenade {} (h_{:08x}): timer {}, use {}, function {}, explosion h_{:08x} {}-{} type {} radius {}, effect h_{:08x} {:x?} light {:08x} sounds {:x?}, impact {:08x}, decal {:08x}, trail h_{:08x} {:x?} {:x?}, icon {:08x}, {} parts, bounds {:.2} - {:.2}, archetype h_{:08x} sphere {:?}",
              def.label, def.name, def.fuse, def.use_type, def.function_type, def.projectile, blast.damage_min, blast.damage, blast.damage_type,
              blast.blast_radius, blast.blast_effect, blast_fx.effects, blast_fx.light, blast_fx.sounds, blast.impact_sound, blast.decal,
              def.attached_effect, trail_fx.effects, trail_fx.sounds, def.icon, parts.len(), lo, hi, def.archetype, sphere);
        GrenadeKit { def, blast, blast_fx, trail_fx, parts, bounds, splash_radius }
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

/// A grenade out of the hand, until it goes off. (play_sentry.rs reads a mine's type, whether
/// it's down and who set it down, and sets it off; play_roller.rs sets a Roller's fuse to 0.)
#[derive(Component)]
pub(super) struct Grenade {
    pub(super) kind: usize,
    velocity: Vec3,
    pub(super) fuse: f32,
    /// armed by its first contact (+0x1dc bit 1, set by the contact handler 0x149510): the fuse
    /// runs once the grenade has come down
    pub(super) landed: bool,
    /// goes off this frame whatever its fuse and the cutscene bit: a Sentry's own check (0x147620)
    /// calls the grenade's blast (+0x15c) itself, not through the fuse (play_sentry.rs), and the
    /// demo's shot and blast on a mine do the same
    pub(super) set_off: bool,
    spin: Vec3,
    /// the character who threw it (its blast hurts them by SELF_DAMAGE)
    pub(super) thrower: usize,
    /// a Roller: its rolling sound is due again in this many seconds
    roll_sound: f32,
    /// when it left the hand (sim time; BF_GRENADE_LOG prints the flight to the first contact)
    launched: f32,
    /// in a liquid (its obj+0x104 above 0, play_liquids.rs `object_enters`): it splashes going in
    /// only from out of one
    wet: bool,
}

impl Grenade {
    /// A grenade of type `kind` that's down where the level placed it (an item object with
    /// h_1cb7d5f2, play_sentry.rs: the tutorial's enemy Sentries, m09_b's Thermo), armed (its body
    /// settles on the floor as the level loads: a first contact, todo/82 leads Round 2 Q1), its
    /// fuse the type's (the Sentry's 9999 s: what sets it off is its check; the Thermo's 15 s),
    /// thrown by no one (OTHER: no squad member's own, so no thrower's share of its blast).
    pub(super) fn placed(kind: usize, fuse: f32, now: f32) -> Grenade {
        Grenade { kind, velocity: Vec3::ZERO, fuse, landed: true, set_off: false, spin: Vec3::ZERO, thrower: OTHER, roll_sound: 0.0, launched: now, wet: false }
    }
}

/// The red damage tint: seconds since the player was hurt by a blast (TINT_TIME or more: none).
/// (play_energy.rs sets it when a bolt strikes the player.)
#[derive(Resource)]
pub struct ScreenTint(pub f32);

/// The tint's quad in front of the camera (multiplied into the 3D picture).
#[derive(Component)]
struct TintQuad(Vec<Handle<TintMaterial>>);

/// The tint's material: bevy's standard one, multiplied (AlphaMode::Multiply), leaving the
/// picture's alpha as it was (`KeepAlpha`).
type TintMaterial = bevy::pbr::ExtendedMaterial<StandardMaterial, KeepAlpha>;

/// `TintMaterial`'s extension: nothing of its own but its pipeline. The picture's alpha carries
/// the glare's keys until the glare reads them after the 3D picture (bf_viewer::glare, #289);
/// bevy's multiply blend writes the quad's alpha over them (1, over the whole view), which put
/// the glare out wherever the quad is: always, as it stays shown. The console's red damage tint
/// is the view's colour fade, drawn over the finished picture, glare and all.
#[derive(Asset, bevy::render::render_resource::AsBindGroup, Reflect, Debug, Clone, Default)]
struct KeepAlpha {}

impl bevy::pbr::MaterialExtension for KeepAlpha {
    /// The blend's alpha keeps what's there (ZERO / ONE); the colour's is bevy's multiply.
    fn specialize(_pipeline: &bevy::pbr::MaterialExtensionPipeline, descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
                  _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef, _key: bevy::pbr::MaterialExtensionKey<Self>)
                  -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation};
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for blend in fragment.targets.iter_mut().flatten().filter_map(|t| t.blend.as_mut()) {
                blend.alpha = BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::One, operation: BlendOperation::Add };
            }
        }
        Ok(())
    }
}

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
    app.add_plugins(MaterialPlugin::<TintMaterial> { prepass_enabled: false, shadows_enabled: false, ..default() })
        .init_resource::<Held>()
        .init_resource::<DelayedBlastParts>()
        .insert_resource(ScreenTint(TINT_TIME))
        .add_systems(OnEnter(AppState::Playing), load_kits.after(setup))
        .add_systems(Update, (super::inventory::stock, launch_grenades, hold_grenade, super::sentry::trip, super::roller::seek, fly_grenades, super::energy::strike,
                              super::sonic::ring, tint).chain().in_set(GrenadeSystems)
            .after(update_player).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// The squad's grenade types (SQUAD_GRENADES, those the loaded levels define), then any other
/// grenade or explosive whose label none of those has that the level's squad placements carry (a
/// Sentry defined only as h_f73de83d: m14_c and m14_d) or that the level puts down itself (its
/// item spawns, #82), their ALE effects compiled and their materials' render pipelines built ahead
/// (`ale_fx::warm_up`). The item spawns' rule is general; run on every level that puts down a type
/// outside SQUAD_GRENADES (28: the campaign's and the arenas'), it adds a kit on four: m09_b's
/// Thermo, and the Sentry h_f73de83d on e16, e34 and e40, which those three lay as live mines
/// (h_1cb7d5f2): with its kit made, `place_mines` (play_sentry.rs) now arms them as the level
/// loads (14, 6 and 7), where before #82 they had no kit and weren't placed. The types of the
/// level's live-placed items get their own kit by type hash, not by label (#317): m09_b's and
/// m09_x's Sentry h_f73de83d and e23's h_ecfe0d8c share the label of the squad's h_e5f1f063, which
/// the label rule above kept out; `place_mines` (play_sentry.rs) now sets down 1 more mine on
/// m09_b (its one of h_f73de83d), 3 more on m09_x (4 in all) and 3 on e23 (before: none), and no other level's
/// count changes (`BF_SENTRY_LOG=1` over the 53 maps, before and after).
#[allow(clippy::too_many_arguments)]
fn load_kits(mut commands: Commands, mut game: ResMut<GameData>, mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
             mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>,
             ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut held: ResMut<Held>, mut tint: ResMut<ScreenTint>,
             mut delayed: ResMut<DelayedBlastParts>, spawns: Option<Res<super::triggers::SquadSpawns>>, map: Option<Res<MapLevel>>) {
    held.0 = None;
    tint.0 = TINT_TIME;
    delayed.0.clear();
    delayed.1.clear();
    delayed.2.clear();
    let usable = |d: &WeaponDef| d.projectile != 0 && d.icon != 0 && !d.label.is_empty() && !d.label.starts_with("h_");
    let mut defs: Vec<WeaponDef> = SQUAD_GRENADES.iter().filter_map(|n| game.0.weapons.get(n)).filter(|d| usable(d)).cloned().collect();
    let carried = spawns.iter().flat_map(|s| s.0.iter()).flat_map(|s| s.items.iter().map(|i| i.0));
    // and those the level puts down itself (its item spawns, h_1d1d4f05, as pickups or set down
    // live with h_1cb7d5f2): m09_b's Thermo (h_e04e05e1, the type's only placement, set down, #82)
    // and e16's, e34's and e40's Sentry mines (h_f73de83d), so a level's own mine is armed
    // (`place_mines`) and a picked-up stack has its kit (the pickups that hand them out are #17's)
    let laid: Vec<u32> = map.iter().flat_map(|m| m.0.iter()).flat_map(|l| l.objects.iter()).filter(|o| o.tag == ITEM_SPAWN).map(|o| o.kind).collect();
    for n in carried.chain(laid) {
        let other = game.0.weapons.get(&n).filter(|d| matches!(d.group_type, 2 | 3) && usable(d))
            .filter(|d| !defs.iter().any(|k| k.label.eq_ignore_ascii_case(&d.label)));
        if let Some(d) = other.cloned() {
            defs.push(d);
        }
    }
    // and the types of the level's live-placed items (`Placement::placed`, h_1cb7d5f2: `place_mines`
    // sets them down as the level loads), taken by their own type hash, not by label (#317): two
    // types can share a label (the Sentry has three definitions, h_e5f1f063, h_f73de83d and e23's
    // h_ecfe0d8c, all labelled Sentry), so m09_b's and m09_x's live h_f73de83d and e23's live
    // h_ecfe0d8c got no kit behind the squad's own Sentry h_e5f1f063 and were never put down. The
    // squad's kit keeps its place first (the labels' `GrenadeKits::find` takes the first of a label,
    // and the sort below is stable). A level's own Roller is left to #316, a spawn-trigger's template
    // isn't in the world (`place_mines`'s rules).
    for l in map.iter().filter_map(|m| m.0.as_ref()) {
        let templates = l.spawn_templates();
        for o in l.objects.iter().filter(|o| o.placed && !templates.contains(&o.name)) {
            let own = game.0.weapons.get(&o.kind)
                .filter(|d| matches!(d.group_type, 2 | 3) && usable(d) && d.function_type != super::roller::ROLLING_BOMB)
                .filter(|d| !defs.iter().any(|k| k.name == d.name));
            if let Some(d) = own.cloned() {
                defs.push(d);
            }
        }
    }
    // (BF_TEST_INV's grenade types the level defines but neither SQUAD_GRENADES, its squad nor its
    // item spawns bring in, the first by name. #196's nine-stack pool on e34 (play_inventory.rs)
    // asks for a Sentry, whose kit, h_f73de83d, e34's own mines now bring in above)
    if let Ok(v) = std::env::var("BF_TEST_INV") {
        for label in v.split(',').filter_map(|e| e.split_once(':')).map(|(n, _)| n.trim()) {
            if defs.iter().any(|k| k.label.eq_ignore_ascii_case(label)) {
                continue;
            }
            let mut named: Vec<(&u32, &WeaponDef)> = game.0.weapons.iter()
                .filter(|(_, d)| matches!(d.group_type, 2 | 3) && usable(d) && d.label.eq_ignore_ascii_case(label)).collect();
            named.sort_by_key(|(n, _)| **n);
            if let Some((_, d)) = named.first() {
                defs.push((*d).clone());
            }
        }
    }
    // (in the store's (category, index) order, as SQUAD_GRENADES: an alternate taken above goes in
    // its place)
    defs.sort_by_key(|d| (d.group_type, d.item_index));
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

/// A grenade let go this frame (play.rs's throw and place): where, how fast, its type, and
/// whether it's on the ground already, its fuse running (BF_TEST_EXPLOSION's).
pub struct Thrown {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub kind: usize,
    pub landed: bool,
}

/// Spawn the grenades let go this frame: the one in the hand flies on (its trail with it), or
/// drops at the feet (a placed type). A Roller let go gets its seek helper (the item set's let-go
/// 0x148640 -> FUN_00148720, play_roller.rs): its point 50 m along the thrower's aim.
fn launch_grenades(mut commands: Commands, mut player: ResMut<Player>, kits: Option<Res<GrenadeKits>>, mut held: ResMut<Held>,
                   mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut next_test: Local<f32>) {
    let Some(kits) = kits else { return };
    let mut throws = std::mem::take(&mut player.thrown);
    // test hook: BF_TEST_EXPLOSION=1 drops the selected type 6 m ahead every 1.5 s, going off at
    // once
    let test = std::env::var("BF_TEST_EXPLOSION").is_ok();
    let selected = match player.item { Some(Item::Grenade(k)) => k, _ => 0 };
    if test && player.sim_time >= *next_test && !kits.0.is_empty() {
        *next_test = player.sim_time + 1.5;
        let (x, z) = (player.position.x, player.position.z - 6.0);
        throws.push(Thrown { pos: Vec3::new(x, floor_y(x, z, player.position.y + GROUND + 1.0, world::Body::Thing) + GRENADE_RADIUS, z), velocity: Vec3::ZERO,
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
        // (the placer's horizontal aim: its aim frame's forward flattened, 0x10fbf0; the demo's aim
        // frame is the camera's turn, `camera::aim_turn`)
        if kit.def.function_type == super::roller::ROLLING_BOMB && !t.landed {
            let aim = super::camera::aim_turn(&player) * Vec3::NEG_Z;
            let roller = super::roller::Roller::let_go(kit, t.pos, Some(aim), Vec3::Z, player.team, Some(player.who()), player.sim_time);
            if std::env::var("BF_ROLLER_LOG").is_ok() {
                println!("t {:.2}: Roller let go at {:.2}, its point {:.2} ({:.1} degrees, the logs' atan2(dz, dx))", player.sim_time, t.pos, roller.point,
                         (roller.point.z - t.pos.z).atan2(roller.point.x - t.pos.x).to_degrees());
            }
            commands.entity(e).insert(roller);
        }
        let spin = if kit.placed() { Vec3::ZERO } else { t.velocity.cross(Vec3::Y).normalize_or(Vec3::X) * -12.0 };
        let fuse = if t.landed { 0.0 } else { kit.def.fuse.max(0.0) };
        if std::env::var("BF_GRENADE_LOG").is_ok() && !t.landed {
            let v = t.velocity;
            let feet = floor_y(player.position.x, player.position.z, player.position.y + GROUND + 1.0, world::Body::Player);
            println!("t {:.2}: {} leaves the hand {:.2} m over the floor at the thrower's feet at {:.1} m/s, {:.1} degrees up (mass {:.1})", player.sim_time,
                     kit.def.label, t.pos.y - feet, v.length(), v.y.atan2(Vec2::new(v.x, v.z).length()).to_degrees(), kit.def.mass);
        }
        commands.entity(e).insert(Grenade { kind: t.kind, velocity: t.velocity, fuse, landed: t.landed, set_off: false, spin, thrower: player.character,
                                            roll_sound: 0.0, launched: player.sim_time, wet: false });
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

/// Grenade flight: gravity, bounces off the ground, the pillars and the map's objects (a Roller
/// hops toward its drive and stops at walls, play_roller.rs), the blast when the fuse ends (or at
/// the first contact, timer 0), or when something sets it off (`Grenade::set_off`).
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
    mut grenades: Query<(Entity, &mut Grenade, &mut Transform, Option<&super::roller::Roller>)>,
    (mut blasts, mut decals, mut delayed, mut tint, mut gas, mut bolts, mut rings, mut objects): (ResMut<super::pickups::Blasts>, ResMut<super::fx::DecalRequests>,
                                                               ResMut<DelayedBlastParts>, ResMut<ScreenTint>, ResMut<super::gas::GasClouds>,
                                                               ResMut<super::energy::BoltRequests>, ResMut<super::sonic::RingRequests>,
                                                               ResMut<super::scenery::ObjectBlasts>),
    test: Option<Res<super::testmap::TestMap>>,
    mut detonated: Local<bool>,
    mut trails: Query<(&ChildOf, &mut bf_viewer::ale_fx::AleEffect)>,
    mut spent: Query<(Entity, &mut Spent)>,
    difficulty: Res<Difficulty>,
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
        let (max, min) = (kit.blast.damage, kit.blast.damage_min.min(kit.blast.damage));
        let roll = p.random(1000) as f32 / 1000.0;
        let own = SELF_DAMAGE * (min + (max - min) * roll);
        let now = p.sim_time;
        if hurt_by_blast(p, &game.0, kit, at, own, thrower, false, &difficulty, now).is_some() {
            tint.0 = 0.0;
        }
        for u in squad.0.iter_mut() {
            hurt_by_blast(u, &game.0, kit, at, own, thrower, instant, &difficulty, now);
        }
    }
    // test hook: BF_TEST_DETONATE=<s> sets off every grenade out at that time (a Sentry with
    // nobody hostile about, a Roller before its fuse)
    let detonate = std::env::var("BF_TEST_DETONATE").ok().and_then(|v| v.parse::<f32>().ok())
        .is_some_and(|at| p.sim_time >= at && !*detonated);
    *detonated |= detonate;
    // (the world's cutscene bit, world+0xc50 bit 4: a script's 122 sets it, its 123 clears it)
    let cutscene = super::script::in_cutscene();
    for (e, mut g, mut tr, roller) in &mut grenades {
        let Some(kit) = kits.0.get(g.kind) else { continue };
        if detonate {
            g.fuse = 0.0;
            g.landed = true;
        }
        // the fuse (the grenade update 0x149420): it counts, and ends in the blast, only once the
        // grenade is out of the hand (+0x1bc bit 1, 0x149454: it's a Grenade once let go), armed by
        // a contact (+0x1dc bit 1, 0x1494b3: `landed`) and with no cutscene (0x1494be returns before
        // the subtraction while the world's cutscene bit is set: m09_b's Thermo waits out its
        // startup cinematic, todo/82 leads Round 2 Q1). A Sentry's own check sets it off directly
        let counts = g.landed && !cutscene;
        if counts {
            g.fuse -= dt;
        }
        if g.set_off || (counts && g.fuse <= 0.0) {
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
            // loose pickups are thrown by blasts that hurt, and breakable scenery takes their
            // damage DAMAGE_DELAY later (play_scenery.rs; the Energy's and the Sonic's too, as a
            // plain blast: a simplification)
            blasts.0.push((at, radius));
            objects.0.push(super::scenery::ObjectBlast { left: DAMAGE_DELAY, at, radius, damage: max, damage_type: kit.blast.damage_type,
                                                        label: kit.def.label.clone(), flat: false });
            // the Energy's damage comes with its bolts (play_energy.rs)
            if super::energy::releases_bolts(kit) {
                bolts.0.push((g.kind, Vec3::new(at.x, floor_y(at.x, at.z, at.y + 0.5, world::Body::Thing), at.z), g.thrower));
                continue;
            }
            // the Sonic's comes with its ring (play_sonic.rs)
            if super::sonic::carries_damage(kit) {
                rings.0.push((g.kind, at, g.thrower));
                continue;
            }
            delayed.2.push(PendingDamage { left: DAMAGE_DELAY, kind: g.kind, at, thrower: g.thrower });
            continue;
        }
        // (a Roller rests on the ground and meets walls by its own ball's radius, its archetype's
        // sphere: 0.19 m; the others by GRENADE_RADIUS)
        let radius = roller.map_or(GRENADE_RADIUS, |r| r.radius_m);
        // (a wall stops a Roller with no bounce: take17 / take18 stop it within 0.05 s at the gate's
        // panel, it settles back 0.15 m at most and stays against it, pressed toward its point)
        let rebound = if roller.is_some() { 0.0 } else { 0.5 };
        g.velocity.y -= GRENADE_GRAVITY * dt;
        let speed = g.velocity.length();
        let mut pos = tr.translation + g.velocity * dt;
        let floor = floor_y(pos.x, pos.z, tr.translation.y, world::Body::Thing) + radius;
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
                // GOAL_DIVE); not from one set down (a Roller, a Sentry). The test map's NPCs set
                // to fight dive too; its dummies stand where they were put
                if !kit.placed() {
                    for m in squad.0.iter_mut().filter(|m| !m.dead && m.npc.is_none_or(|n| n.fight) && m.position.distance(pos) < DIVE_RADIUS) {
                        m.dive_from = Some(pos);
                    }
                }
            }
            g.landed = true;
            if let Some(r) = roller {
                // a Roller hops off again, its way along the ground taken toward its drive
                // (play_roller.rs `hop`: the takes' ball flies straight between ground contacts)
                let before = Vec2::new(g.velocity.x, g.velocity.z).length();
                if super::roller::hop(&mut g.velocity, r.drive, super::roller::ground_normal(pos.x, pos.z, tr.translation.y))
                    && std::env::var("BF_ROLLER_LOG").is_ok() {
                    println!("t {:.2}: Roller contact at {pos:.2}: {before:.2} -> {:.2} m/s along the ground, drive {:.2} m/s", p.sim_time,
                             Vec2::new(g.velocity.x, g.velocity.z).length(), r.drive.length());
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
        if let Some(r) = roller {
            // its rolling sound, again and again once it's down, fading with distance; and its
            // turn as it rolls, with its drive (the game's ball spins with its push)
            g.spin = super::roller::spin(r);
            if g.landed {
                g.roll_sound -= dt;
                if g.roll_sound <= 0.0 && kit.def.object_sound != 0 {
                    g.roll_sound = ROLL_SOUND_EVERY;
                    let d = pos.distance(p.position);
                    let volume = 0.8 * (1.0 - (d - ROLL_SOUND_NEAR) / (ROLL_SOUND_FAR - ROLL_SOUND_NEAR)).clamp(0.0, 1.0);
                    if volume > 0.0 {
                        p.sound_queue.push((kit.def.object_sound, volume));
                    }
                }
            }
        }
        for c in pillars() {
            let half = 0.3 + radius;
            let d = pos - c;
            if d.x.abs() < half && d.z.abs() < half && pos.y < c.y + 1.5 {
                // push out along the shallower axis and reflect
                hit = true;
                if half - d.x.abs() < half - d.z.abs() {
                    pos.x = c.x + half * d.x.signum();
                    g.velocity.x = -g.velocity.x * rebound;
                } else {
                    pos.z = c.z + half * d.z.signum();
                    g.velocity.z = -g.velocity.z * rebound;
                }
            }
        }
        // a Roller's move along the ground is swept for walls first: pushed out after the move, a
        // ball whose middle went past the middle of a thin wall (e34's gate panel, take17 / take18)
        // came out on the far side
        if let (Some(_), Some(a)) = (roller, world::arena()) {
            let from = tr.translation;
            let step = Vec3::new(pos.x - from.x, 0.0, pos.z - from.z);
            let len = step.length();
            if len > 1e-4 {
                let dir = step / len;
                if let Some((t, n)) = a.ray_normal(from, dir, len + radius).filter(|(_, n)| n.y.abs() < super::roller::WALL_NORMAL) {
                    hit = true;
                    let back = (t - radius).clamp(0.0, len);
                    pos.x = from.x + dir.x * back;
                    pos.z = from.z + dir.z * back;
                    let n = Vec2::new(n.x, n.z).normalize_or_zero();
                    let v = Vec2::new(g.velocity.x, g.velocity.z);
                    let vn = v.dot(n);
                    if vn < 0.0 {
                        let v = v - n * vn;
                        g.velocity.x = v.x;
                        g.velocity.z = v.y;
                    }
                }
            }
        }
        // the map's objects: push out and bounce off
        if let Some(a) = world::arena() {
            let at = Vec2::new(pos.x, pos.z);
            let q = a.push_out(at, radius, pos.y - radius, pos.y + radius, 0.0, world::Body::Thing);
            if q != at {
                hit = true;
                let n = (q - at).normalize_or_zero();
                let v = Vec2::new(g.velocity.x, g.velocity.z);
                let vn = v.dot(n);
                if vn < 0.0 {
                    let v = v - n * vn * (1.0 + rebound);
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
        // into a liquid (#5): its splash, by its mesh's radius (play_liquids.rs; the liquids aren't
        // solid, so it goes on down to the bed)
        if let Some(a) = world::arena() {
            if let Some((t, at, slot)) = super::liquids::object_enters(&a, &mut g.wet, tr.translation, pos, radius, kit.splash_radius, speed) {
                super::liquids::splash(p, &t, slot, at);
                if std::env::var("BF_GRENADE_LOG").is_ok() || std::env::var("BF_LIQUID_LOG").is_ok() {
                    println!("t {:.2}: {} into a liquid (type {}) at {speed:.1} m/s: a {} splash at {at:.2} (r {:.3}): effect h_{:08x}, sound {:08x}",
                             p.sim_time, kit.def.label, t.kind, if slot == bf_viewer::bf::character::LIQUID_BIG_SPLASH { "big" } else { "small" }, kit.splash_radius,
                             t.effects[slot], t.sounds[slot]);
                }
            }
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

/// One character hurt by a blast of `kit` at `at`, if they're alive and within its radius:
/// Damage max falling to nothing at the radius, the thrower `own` anywhere inside it (the
/// test map's instant kill, `kill`, kills anyone else), times the character's difficulty step
/// and factor for the explosion's damage-type (play_damage.rs: a blast reaches a character's
/// handler 0x125600 as every hit does); thrown back and up by as much. `own` is an amount
/// measured on the takes' squad (STANDARD), which holds their step already: it's divided by it
/// (`damage::footage_squad_step`) and comes out as measured there. A corpse isn't hurt: its
/// ragdoll takes the blast's push (`ragdoll::blast_corpse`). Returns the damage dealt (None if
/// none: out of reach, dead, or no damage).
#[allow(clippy::too_many_arguments)]
pub(super) fn hurt_by_blast(u: &mut Player, game: &Game, kit: &GrenadeKit, at: Vec3, own: f32, thrower: usize, kill: bool,
                            difficulty: &Difficulty, now: f32) -> Option<f32> {
    let (radius, max) = (kit.blast.blast_radius, kit.blast.damage);
    // a corpse: the blast's push on its ragdoll's root body when the explosion pushes
    // (FUN_00224a90, its Damage max as the blast's damage: play_ragdoll.rs `blast_corpse`, #259).
    // With no cover test, as the grenades' blasts hurt the living here (their rule is the demo's
    // until #112 gives them the game's: its ray from the blast to the body)
    if u.dead {
        ragdoll::blast_corpse(u, at, radius, max, kit.blast.push, |_| true);
        return None;
    }
    let d = u.position.distance(at);
    if d >= radius {
        return None;
    }
    let k = 1.0 - d / radius;
    let away = Vec3::new(u.position.x - at.x, 0.0, u.position.z - at.z).normalize_or(Vec3::X);
    // (the thrower is the squad's: a test map NPC of their character isn't them)
    let own_blast = u.who() == Who::Squad(thrower);
    let mut hit = if own_blast {
        Hit::new(own / footage_squad_step(game), "the thrower's share", kit.blast.damage_type)
    } else {
        Hit::new(max * k, "Damage max x (1 - d / r)", kit.blast.damage_type)
    }.source(Who::Squad(thrower));
    let dealt = hit.on(u, game, difficulty);
    let dealt = if kill && !own_blast { enemies::lethal(u).max(dealt) } else { dealt };
    if dealt <= 0.0 {
        return None;
    }
    hit.report(u, game, now);
    hurt(u, game, dealt, chatter::Cause::hit(&hit), (away * 6.0 + Vec3::Y * 4.0) * k, u.position + Vec3::Y * 1.0, -1);
    // (a kill's own shoves on the body, behind ragdoll::DEATH_SHOVE: the blast's record has no
    // part, and a push of 2 when its weapon's is 0, FUN_0022c4d0)
    if u.dead {
        let push = if kit.blast.push > 0.0 { kit.blast.push } else { 2.0 };
        ragdoll::death_shoves(u, None, u.position + Vec3::Y * 1.0, away, dealt, kit.blast.damage_type, push);
    }
    if std::env::var("BF_COMBAT_LOG").is_ok() {
        println!("{} blast at {d:.1} m: {} takes {dealt:.1} -> {:.1} / {:.0}", kit.def.label, unit_name(u), u.health, u.max_health);
    }
    Some(dealt)
}

/// A blast at `at`: the explosion's effect type (its ALE effects and light effect, run once on
/// the ground below, and its sounds, after their delay) and the bullet's impact sound; its
/// decal on the ground. Returns how long its effects run (s; the Light's phosphor_grenade: its
/// 30 s emitters and their last particles' 2 s).
#[allow(clippy::too_many_arguments)]
fn blast(commands: &mut Commands, game: &mut Game, ale: Option<&mut bf_viewer::ale_fx::AleAssets>, images: &mut Assets<Image>,
         materials: &mut Assets<StandardMaterial>, kit: &GrenadeKit, at: Vec3, p: &mut Player, delayed: &mut DelayedBlastParts) -> f32 {
    // (where it is, when a level has no floor under it: a grenade over a rock's face too steep to
    // be a floor found none, and its blast went to the flat floor's height, -1.05 m: a Roller on
    // e34's rock, #79)
    let floor = match world::arena().filter(|a| !a.is_flat()) {
        Some(a) => a.floor_below(at.x, at.z, at.y + 0.5, world::Body::Thing).map_or(at.y, |f| f.0),
        None => floor_y(at.x, at.z, at.y + 0.5, world::Body::Thing),
    };
    let ground = Vec3::new(at.x, floor, at.z);
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
/// quad stays shown, white when there's no tint (shown on the hurt, it came a frame late too),
/// leaving the picture's alpha (the glare's keys) alone (`TintMaterial`).
fn tint(mut commands: Commands, time: Res<Time>, mut tint: ResMut<ScreenTint>, mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<TintMaterial>>,
        mut quads: Query<(&TintQuad, &mut MeshMaterial3d<TintMaterial>)>,
        camera: Query<Entity, With<MainCamera>>) {
    // the recording's factor at this time (linear between its game frames)
    let f = tint.0 * 30.0;
    let i = (f as usize).min(TINT_CURVE.len() - 1);
    let s = TINT_CURVE[i] + (TINT_CURVE[(i + 1).min(TINT_CURVE.len() - 1)] - TINT_CURVE[i]) * f.fract().min(1.0);
    let k = ((s - TINT_LOW) / (1.0 - TINT_LOW)).clamp(0.0, 1.0);
    tint.0 += frame_dt(&time);
    // (the level now: a quad made for a new camera, e.g. a new map, starts at it, not the deepest)
    let now = ((k * TINT_STEPS as f32).round() as usize).min(TINT_STEPS);
    let Ok((quad, mut mat)) = quads.single_mut() else {
        if let Ok(cam) = camera.single() {
            let steps = (0..=TINT_STEPS).map(|i| {
                let s = tint_factor(TINT_LOW + (1.0 - TINT_LOW) * i as f32 / TINT_STEPS as f32);
                materials.add(TintMaterial {
                    base: StandardMaterial { base_color: Color::linear_rgb(1.0, s, s), unlit: true, fog_enabled: false,
                                             alpha_mode: AlphaMode::Multiply, cull_mode: None, ..default() },
                    extension: KeepAlpha {},
                })
            }).collect::<Vec<_>>();
            commands.spawn((Mesh3d(meshes.add(Rectangle::new(4.0, 4.0))), MeshMaterial3d(steps[now].clone()), TintQuad(steps),
                            NotShadowCaster, Transform::from_xyz(0.0, 0.0, -TINT_DEPTH), Visibility::Inherited, ChildOf(cam)));
        }
        return;
    };
    let step = &quad.0[now];
    if mat.0 != *step {
        mat.0 = step.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #161: the power by the charge (0x1348fd-0x134941): 0.01 for any charge up to 0.3 (a hold
    /// under 0.19 s at 1.6 a second), 1 at the full meter, linear between.
    #[test]
    fn a_throws_power_counts_from_three_tenths_of_the_charge() {
        assert_eq!(throw_power(0.0), POWER_LEAST);
        assert_eq!(throw_power(0.3), POWER_LEAST);
        assert!((throw_power(0.64) - 0.4857).abs() < 1e-3);
        assert!((throw_power(1.0) - 1.0).abs() < 1e-6);
    }

    /// #161: a full Frag (0.9 kg) leaves at 25.6 m/s and a tap at 17.8, an Energy (1 kg) full at
    /// 23.0, each 15 degrees above the aim: level aim, 15 up; aimed 9 degrees down, 6 up; turned
    /// with the aim's yaw.
    #[test]
    fn a_throw_leaves_fifteen_degrees_above_the_aim_at_its_push_over_its_mass() {
        let up = |v: Vec3| v.y.atan2(Vec2::new(v.x, v.z).length()).to_degrees();
        let level = throw_velocity(Quat::IDENTITY, 1.0, 0.9);
        assert!((level.length() - 25.556).abs() < 0.01, "{}", level.length());
        assert!((up(level) - 15.0).abs() < 1e-3 && level.z < 0.0 && level.x.abs() < 1e-5);
        assert!((throw_velocity(Quat::IDENTITY, 0.01, 0.9).length() - 17.778).abs() < 0.01);
        assert!((throw_velocity(Quat::IDENTITY, 1.0, 1.0).length() - 23.0).abs() < 1e-3);
        let down = throw_velocity(Quat::from_euler(EulerRot::YXZ, 0.0, -9f32.to_radians(), 0.0), 1.0, 0.9);
        assert!((up(down) - 6.0).abs() < 1e-3);
        let turned = throw_velocity(Quat::from_euler(EulerRot::YXZ, std::f32::consts::FRAC_PI_2, 0.0, 0.0), 1.0, 0.9);
        assert!(turned.x < -24.0 && turned.z.abs() < 1e-4, "{turned}");
        // (no mass in the definition: 1 kg)
        assert!((throw_velocity(Quat::IDENTITY, 1.0, 0.0).length() - 23.0).abs() < 1e-3);
    }
}
