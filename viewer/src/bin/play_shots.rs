//! The shot path of a gun, from default.xbe's weapon class (0x22b000-0x233000; the data each
//! function reads is `weapon::ShotData`):
//!
//!  - **Fire timing** (FUN_0022f2a0, the fire loop, run once a game frame by FUN_0022ee30 ->
//!    FUN_0022efe0 while the weapon's fire flag +0x224 bit 4 is set):
//!    a shot sets the cooldown (weapon+0x218) to 1 / rate (FUN_0022f0e0; set, not added, so
//!    the rest of the frame is dropped); it counts down by the frame's time (FUN_0022ec90) and
//!    the trigger fires again once it's <= 0. The demo runs this in 30 Hz game frames, so an
//!    interval is the rate's period rounded up to whole frames: MK 8/s -> 4 frames, 7.5/s;
//!    Minigun 15/s -> 2-3 frames (the float sum lands either side of 0, see
//!    `TICK_JITTER`), ~12/s. The takes
//!    measured 7.0-7.4 and 11.4-11.7 (the emulator ran at 27-29 fps). While the holder's
//!    +0x7a8 is set (the scope; medium confidence) the rate is h_019c314a (MK 6/s).
//!  - **Bursts and pellets**: each shot counts weapon+0x28c; below the burst count the next
//!    cooldown is the burst-delay instead (FUN_0022f0e0). With a delay of 0 the loop goes on in
//!    the same frame and the shots after the first take no round (FUN_0022fe00): the Bower's 6
//!    pellets a shell. Each of those is turned by a uniform random yaw and pitch within the
//!    burst-spread (FUN_0022e1d0), in place, so the pellets walk away from the first.
//!  - **Accuracy** (the object at weapon+0x1d0: current +0xc = weapon+0x1dc, cap +0x10): the cap
//!    is the data's max, or its scoped cap while the holder's +0x7a8 is set (FUN_001241b0 ->
//!    FUN_00222ac0); each trigger cycle takes recoil-per-shot off (FUN_00222c10; halved, and the
//!    floor min + 15, while the holder crouches, +0x1f8 == 6); each frame the cooldown is out it
//!    gains recovery x dt (FUN_00222b30, doubled crouched). It's used once a frame, before the
//!    fire loop (FUN_0022ee30 -> FUN_0022e4c0): the aim from each muzzle (FUN_0022e5b0) is turned
//!    about the three world axes by uniform random angles within +-S degrees (FUN_002229b0),
//!    S = (100 - clamp(A + bonus, 0, 100)) x 0.03 (FUN_000c4e10). The bonus (accuracy object
//!    +8) is -1 unless the holder's script property h_ec5337fb is set, then its h_072837bb
//!    (-100..100, FUN_00112d90's neighbour at 0x11302c): -1 here.
//!  - **Tracers**: an instant ray (bullet-type 4, FUN_002317e0) spawns its flight effect only
//!    when the counter weapon+0x290 is 0, which then restarts at h_eeb9e75a: the first shot and
//!    every (N+1)th after (MK every 3rd, Minigun every 4th). Other bullet types always fly theirs.
//!  - **Muzzle effect** h_fd88830d at the muzzle hardpoint, set off with each shot
//!    (FUN_0022f170).
//!  - **Shells**: each trigger cycle adds h_fdc93b33 to weapon+0x208 (FUN_0022f2a0); each whole
//!    one throws a casing object (h_19b21bf5, the casing h_16415157) from the h_ea1abbbe
//!    hardpoint (FUN_0022eee0 -> FUN_00231b10): 5 cm back along the hardpoint's z, at 2-3 m/s
//!    along its z turned a quarter turn about the vertical, plus up to 0.5 m/s along each of the
//!    other two axes (turned +-15 degrees about z).
//!  - **Damage falloff**: an instant hit's damage is scaled by FUN_00223780(Damage h_fb124e6c,
//!    1 - distance / range) (FUN_00230040): the Bower's mode 2 is (1 - d/range)^2, 0.60 at 9 m.

use super::*;
use bf_viewer::bf::weapon::ShotData;

/// The game frame the fire loop runs in (s): the takes' rates are whole frames at ~30 Hz.
pub const TICK: f32 = 1.0 / 30.0;
/// How much a game frame's time varies, as a share of TICK (+-). Most rates' periods are a
/// whole number of frames (Minigun 1/15 = 2, Foley 1/3 = 10, LZR-50 1/3.75 = 8, Jax-iC 1/2 =
/// 15): there the cooldown lands on 0 and the frame time's own wobble decides whether the shot
/// goes on that frame or the next, about half the time each. The takes measured just that:
/// Minigun 0.086 s (2.5 frames: 0.083), Foley 0.352 (10.5: 0.350), LZR-50 0.284 (8.5: 0.283),
/// Jax-iC 0.52 (15.5: 0.517), while the MK's 3.75 frames is always 4 (0.142 measured, 0.133).
/// A model of the frame clock's jitter, small enough not to move any period that isn't a tie.
const TICK_JITTER: f32 = 1e-4;
/// The game's degrees-to-radians factor in FUN_002229b0 and FUN_0022e1d0 (0x3a52e0).
const DEG: f32 = 0.017444;
/// FUN_000c4e10: degrees of spread per point of accuracy below 100.
const SPREAD_PER_POINT: f32 = 0.03;
/// The accuracy object's +8 bonus without the holder's script property (FUN_00222b30).
const ACCURACY_BONUS: f32 = -1.0;
/// Crouched (holder +0x1f8 == 6): the floor of the recoil is min + this (FUN_00222c10).
const CROUCH_FLOOR: f32 = 15.0;
/// FUN_00231b10: a casing starts this far (m) back along the hardpoint's z, leaves at
/// SHELL_SPEED + up to SHELL_SPEED_MORE m/s along its z turned a quarter turn about the
/// vertical, with up to SHELL_JITTER m/s along each other axis, the frame turned up to
/// SHELL_ROLL degrees either way about z.
const SHELL_BACK: f32 = 0.05;
const SHELL_SPEED: f32 = 2.0;
const SHELL_SPEED_MORE: f32 = 1.0;
const SHELL_JITTER: f32 = 0.5;
const SHELL_ROLL: f32 = 15.0;
/// How long a casing stays (s), how it falls (m/s^2) and bounces. Guesses: the casing is a
/// pooled physics object (FUN_002291d0) whose life and material aren't traced.
const SHELL_LIFE: f32 = 2.0;
const SHELL_GRAVITY: f32 = 9.81;
const SHELL_BOUNCE: f32 = 0.3;

/// The game's random numbers (FUN_00233b90: x * 0x19660d + 0x3c6ef35f, the top 23 bits as a
/// float in [0, 1)).
fn random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
    f32::from_bits((*seed >> 9) | 0x3F80_0000) - 1.0
}

/// One weapon object's own fire state (each gun keeps its own in the game).
#[derive(Clone, Default)]
struct WeaponFire {
    /// current accuracy (weapon+0x1dc); None until the gun is first used: it starts at its cap
    accuracy: Option<f32>,
    /// tracer counter (weapon+0x290)
    tracer: i64,
    /// casings owed (weapon+0x208)
    shells: f32,
}

/// One shot the fire loop let off this frame, for the shot builder (play.rs).
#[derive(Clone, Copy, Debug)]
pub struct Pellet {
    /// the game frame it went in (the accuracy turn is once a frame)
    tick: u32,
    /// the accuracy spread then (degrees, FUN_000c4e10)
    spread: f32,
    /// a zero-delay burst's later shot: turned by the burst-spread from the one before
    walk: bool,
    /// it carries the bullet's flight effect (tracer interval)
    pub tracer: bool,
}

/// A character's fire loop state (kept on `Player`).
#[derive(Default)]
pub struct FireState {
    /// time not yet run in game frames
    clock: f32,
    tick: u32,
    /// shots into the current burst (weapon+0x28c)
    burst: i64,
    weapons: Vec<WeaponFire>,
    /// the shots let off since the shot builder last took them
    pub pellets: Vec<Pellet>,
    /// muzzle effects and casings owed to the effects (`shot_fx`)
    pub flashes: u32,
    pub shells: u32,
    seed: u32,
    /// a trigger pull still owed its first shot (see `pull`), and the trigger last frame
    latched: bool,
    was_held: bool,
}

/// The cooldown a trigger cycle, a dry click or a switch sets: 1 / rate, or 1 / the scoped rate
/// h_019c314a while the holder's +0x7a8 is set (FUN_0022f0e0, FUN_002327f0, FUN_0022dc00 all
/// read it through vtable +0x15c).
pub fn period(def: &WeaponDef, scoped: bool) -> f32 {
    let rate = if scoped && def.shots.scoped_rate > 0.0 { def.shots.scoped_rate } else { def.rate };
    1.0 / rate.max(0.1)
}

/// What the trigger and the holder are doing this frame.
pub struct Trigger {
    pub held: bool,
    /// rounds in the clip
    pub clip: i64,
    /// the holder's +0x7a8 (the scope)
    pub scoped: bool,
    /// the holder's +0x1f8 == 6 (crouched)
    pub crouched: bool,
}

/// The spread (degrees either way, on each axis) at accuracy `a` (FUN_000c4e10).
pub fn spread_at(a: f32) -> f32 {
    (100.0 - (a + ACCURACY_BONUS).clamp(0.0, 100.0)) * SPREAD_PER_POINT
}

impl FireState {
    fn weapon(&mut self, i: usize) -> &mut WeaponFire {
        if self.weapons.len() <= i {
            self.weapons.resize(i + 1, WeaponFire::default());
        }
        &mut self.weapons[i]
    }

    /// A reload starting (FUN_0022daf0), a dry click (FUN_002327f0) or a switch (FUN_0022dc00):
    /// each clears the burst counter weapon+0x28c, so the next pull starts a fresh cycle.
    pub fn reset_burst(&mut self) {
        self.burst = 0;
    }

    /// The trigger for the fire loop this frame: held, or pressed earlier and still owed its first
    /// shot. The game's request outlives the button: the press sets the holder's +0x260 bit 8,
    /// which sets the weapon's fire bits each frame (0x122614-0x122655) until an animation event
    /// clears it (0x0f8408d5, FUN_00117660 at 0x117b12), so an 80 ms tap fires 0.19-0.24 s later
    /// (the takes). The clip that clears it isn't placed; the demo clears the request at the
    /// pull's first shot, or when the gun can't fire.
    pub fn pull(&mut self, held: bool, can_fire: bool) -> bool {
        if held && !self.was_held {
            self.latched = true;
        }
        self.was_held = held;
        if !can_fire {
            self.latched = false;
        }
        (held || self.latched) && can_fire
    }

    /// The current accuracy of weapon `i` (for logs and tests).
    pub fn accuracy(&self, i: usize) -> Option<f32> {
        self.weapons.get(i).and_then(|w| w.accuracy)
    }

    /// Runs the weapon's fire loop over `dt` in whole game frames: the cooldown (weapon+0x218,
    /// kept by play.rs), bursts, accuracy, tracers and casings. Returns the rounds taken from
    /// the clip; the shots themselves wait in `pellets`.
    pub fn run(&mut self, cooldown: &mut f32, weapon: usize, def: &WeaponDef, t: Trigger, seed: u32, dt: f32) -> i64 {
        if self.seed == 0 {
            self.seed = seed | 1;
        }
        let d: &ShotData = &def.shots;
        let [min, max, scoped_cap, recoil, recovery] = d.accuracy;
        let cap = if t.scoped { scoped_cap } else { max };
        let cycle = period(def, t.scoped);
        let mut rounds = 0;
        self.clock += dt;
        while self.clock >= TICK {
            self.clock -= TICK;
            self.tick = self.tick.wrapping_add(1);
            // FUN_0022ec90 (with the frame time's wobble, TICK_JITTER)
            let frame = TICK * (1.0 + (random(&mut self.seed) * 2.0 - 1.0) * TICK_JITTER);
            if *cooldown > 0.0 {
                *cooldown -= frame;
            }
            // FUN_00222ac0: held within [min, cap]
            let mut a = self.weapon(weapon).accuracy.unwrap_or(cap).max(min).min(cap);
            // FUN_0022e4c0: the aim's turn this frame, from the accuracy before the shots
            let spread = spread_at(a);
            // FUN_0022f2a0
            while t.held && *cooldown <= 0.0 && t.clip - rounds > 0 {
                let walk = self.burst > 0 && d.burst_delay == 0.0;
                // FUN_0022f0e0
                self.burst += 1;
                if self.burst < d.burst_count {
                    *cooldown = d.burst_delay;
                } else {
                    *cooldown = cycle;
                    self.burst = 0;
                }
                // FUN_0022fe00: a zero-delay burst's later shots take no round
                if !walk {
                    rounds += 1;
                }
                self.latched = false;
                // FUN_002317e0: instant rays carry the flight effect every (N+1)th shot
                let tracer = if d.bullet_type == 4 {
                    let w = self.weapon(weapon);
                    if w.tracer == 0 {
                        w.tracer = d.tracer_gap;
                        true
                    } else {
                        w.tracer -= 1;
                        false
                    }
                } else {
                    true
                };
                self.pellets.push(Pellet { tick: self.tick, spread, walk, tracer });
                self.flashes += 1;
                // the end of a trigger cycle: casings, recoil (FUN_0022f2a0 -> FUN_00222c10)
                if self.burst == 0 || d.burst_delay != 0.0 {
                    let (lost, floor) = if t.crouched { ((recoil as i64 / 2) as f32, (min + CROUCH_FLOOR).min(cap)) } else { (recoil, min) };
                    a = (a - lost).max(floor).min(cap);
                    if d.shell_object != 0 && d.shell_hardpoint != 0 {
                        self.weapon(weapon).shells += d.shell_rate;
                    }
                }
                if *cooldown > 0.0 {
                    break;
                }
            }
            // FUN_0022eee0: a casing for each whole one owed
            let w = self.weapon(weapon);
            let owed = w.shells.floor().max(0.0);
            w.shells -= owed;
            self.shells += owed as u32;
            // FUN_00222b30: back toward the cap while the cooldown is out
            if *cooldown <= 0.0 {
                a += recovery * frame * if t.crouched { 2.0 } else { 1.0 };
            }
            self.weapon(weapon).accuracy = Some(a.max(min).min(cap));
        }
        rounds
    }

    /// The directions of the shots waiting, from the aim `base`: each game frame's turned once
    /// by the accuracy (FUN_002229b0), a zero-delay burst's later shots each turned again from
    /// the one before by the burst-spread (FUN_0022e1d0).
    pub fn directions(&mut self, base: Vec3, burst_spread: f32) -> Vec<(Pellet, Vec3)> {
        let pellets = std::mem::take(&mut self.pellets);
        let mut out = Vec::with_capacity(pellets.len());
        let (mut tick, mut dir) = (None, base);
        for p in pellets {
            if tick != Some(p.tick) {
                tick = Some(p.tick);
                let s = p.spread * DEG;
                let turn = |seed: &mut u32| (random(seed) * 2.0 - 1.0) * s;
                let (x, y, z) = (turn(&mut self.seed), turn(&mut self.seed), turn(&mut self.seed));
                dir = (Quat::from_rotation_z(z) * Quat::from_rotation_y(y) * Quat::from_rotation_x(x) * base).normalize();
            }
            if p.walk && burst_spread > 0.0 {
                let s = burst_spread * DEG;
                let yaw = (random(&mut self.seed) * 2.0 - 1.0) * s;
                let pitch = (random(&mut self.seed) * 2.0 - 1.0) * s;
                let right = dir.cross(Vec3::Y).normalize_or(Vec3::X);
                dir = (Quat::from_axis_angle(right, pitch) * Quat::from_rotation_y(yaw) * dir).normalize();
            }
            out.push((p, dir));
        }
        out
    }
}

/// FUN_00223780: the damage factor at `x` = 1 - distance / range, by the Damage's falloff mode
/// (h_fb124e6c).
pub fn falloff(mode: i64, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    match mode {
        1 => x,
        2 => x * x,
        3 if x < 0.5 => 2.0 * x,
        _ => 1.0,
    }
}

impl Shot {
    /// The share of the weapon's damage a hit `t` m along this shot deals (`falloff`, as
    /// FUN_00230040 applies it to an instant hit with the bullet's range).
    pub fn falloff_at(&self, t: f32) -> f32 {
        if self.range <= 0.0 { 1.0 } else { falloff(self.falloff, 1.0 - t / self.range) }
    }
}

/// Where a ray meets the world, and the surface's normal facing back along it (for the hole it
/// leaves): the map's collision, or on the flat test floor the ground, a pillar's side, or (a
/// placed object's box, `ray_hit`) facing the shot.
pub fn hit_normal(origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Vec3)> {
    if let Some(a) = world::arena() {
        return a.ray_normal(origin, dir, max);
    }
    let t = ray_hit(origin, dir, max)?;
    let at = origin + dir * t;
    if (at.y - GROUND).abs() < 0.01 {
        return Some((t, Vec3::Y));
    }
    for c in pillars() {
        let d = (at - c) / Vec3::new(0.3, 1.5, 0.3);
        if d.abs().max_element() < 1.01 {
            let k = if d.x.abs() >= d.y.abs().max(d.z.abs()) { 0 } else if d.y.abs() >= d.z.abs() { 1 } else { 2 };
            let mut n = Vec3::ZERO;
            n[k] = d[k].signum();
            return Some((t, n));
        }
    }
    Some((t, -dir))
}

/// A casing thrown from a gun: it falls and bounces off the floor, then goes.
#[derive(Component)]
struct Shell {
    velocity: Vec3,
    spin: Vec3,
    life: f32,
}

/// The casing objects' meshes, by object type (loaded the first time one is thrown); None if it
/// has no model.
#[derive(Resource, Default)]
struct ShellModels(HashMap<u32, Option<Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform)>>>);

pub fn plugin(app: &mut App) {
    app.init_resource::<ShellModels>()
        .add_systems(Update, test_weapon.before(spawn_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (shot_fx, move_shells).chain().after(update_weapons).before(projectiles)
            .run_if(in_state(AppState::Playing)));
}

/// The muzzle effect and the casings for the shots each character let off this frame.
#[allow(clippy::too_many_arguments)]
fn shot_fx(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
           ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut models: ResMut<ShellModels>, globals: Query<&GlobalTransform>,
           mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
           mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>) {
    let Some(mut ale) = ale else { return };
    for p in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let (flashes, shells) = (std::mem::take(&mut p.fire_state.flashes), std::mem::take(&mut p.fire_state.shells));
        if flashes == 0 && shells == 0 {
            continue;
        }
        let Some(w) = p.loaded.as_ref().and_then(|l| l.weapons.get(p.weapon)) else { continue };
        let d = &w.def.shots;
        // the muzzle effect, once a frame however many shots (the Bower's pellets are one)
        if flashes > 0 && d.muzzle_effect != 0 {
            let at = Transform::from_translation(w.muzzle.point).with_rotation(Quat::from_rotation_arc(Vec3::Y, w.fire_dir));
            for fx in effects_of(&mut game.0, &mut ale, &mut images, &mut materials, d.muzzle_effect) {
                let life = fx.duration();
                let seed = p.fire_state.next_seed();
                commands.spawn((at, Visibility::default(), bf_viewer::ale_fx::AleEffect::once(fx, 0.0, seed), AleExpire(life), ChildOf(w.entity)));
            }
        }
        let (Some(port), Ok(g)) = (w.shell, globals.get(w.entity)) else { continue };
        if shells == 0 {
            continue;
        }
        let parts = models.0.entry(d.shell_object).or_insert_with(|| {
            let arch = *game.0.object_meshes.get(&d.shell_object)?;
            let model = WeaponModel::load(&game.0, arch).ok()?;
            let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
            Some(model.parts.iter().flat_map(|part| {
                let place = Transform::from_translation(part.offset).with_rotation(part.rotation);
                bf_viewer::scene::static_meshes(&mut game.0, &part.geosets, &mut assets, true).into_iter().map(move |(m, mat)| (m, mat, place))
            }).collect())
        }).clone();
        let Some(parts) = parts else { continue };
        let frame = g.compute_transform();
        let rot = frame.rotation * port.rot;
        let at = frame.transform_point(port.point);
        for _ in 0..shells {
            let s = &mut p.fire_state;
            let quarter = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            let roll = (random(&mut s.seed) * 2.0 * SHELL_ROLL - SHELL_ROLL).to_radians();
            let turned = quarter * rot * Quat::from_rotation_z(roll);
            let speed = SHELL_SPEED + random(&mut s.seed) * SHELL_SPEED_MORE;
            let velocity = -(turned * Vec3::Z) * speed + turned * Vec3::Y * (random(&mut s.seed) * SHELL_JITTER)
                + turned * Vec3::X * (random(&mut s.seed) * SHELL_JITTER);
            let spin = Vec3::new(random(&mut s.seed) - 0.5, random(&mut s.seed) - 0.5, random(&mut s.seed) - 0.5) * 30.0;
            let root = commands.spawn((Transform::from_translation(at - rot * Vec3::Z * SHELL_BACK).with_rotation(rot),
                                       Visibility::default(), Shell { velocity, spin, life: SHELL_LIFE })).id();
            for (mesh, material, place) in &parts {
                commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), *place, NotShadowCaster, ChildOf(root)));
            }
        }
    }
}

impl FireState {
    /// A seed for an effect's own random numbers.
    fn next_seed(&mut self) -> u32 {
        random(&mut self.seed);
        self.seed
    }
}

/// BF_TEST_WEAPON=<label or hex name> (e.g. "Bower 20", "RVG50 Minigun", 0c3db625): the
/// controlled character starts with that weapon in the slot BF_START_WEAPON picks (the first by
/// default), if the map's data (or the test map's, `-- --test`) defines it. For trying a gun
/// the character doesn't carry.
fn test_weapon(mut done: Local<bool>, mut game: ResMut<GameData>, player: Res<Player>) {
    if std::mem::replace(&mut *done, true) {
        return;
    }
    let Ok(want) = std::env::var("BF_TEST_WEAPON") else { return };
    let hex = u32::from_str_radix(want.trim_start_matches("h_").trim_start_matches("0x"), 16).ok();
    let Some(key) = game.0.weapons.iter().find(|(k, d)| Some(**k) == hex || d.label.eq_ignore_ascii_case(&want)).map(|(k, _)| *k) else {
        warn!("BF_TEST_WEAPON: no weapon {want}");
        return;
    };
    let slot: usize = std::env::var("BF_START_WEAPON").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let list = game.0.character_weapons.entry(CHARACTERS[player.character].to_string()).or_default();
    if slot < list.len() { list[slot] = key } else { list.push(key) }
}

/// Casings fall, bounce off the floor losing most of their speed, and go after their life.
fn move_shells(mut commands: Commands, time: Res<Time>, mut shells: Query<(Entity, &mut Shell, &mut Transform)>) {
    let dt = frame_dt(&time);
    for (e, mut s, mut tr) in &mut shells {
        s.life -= dt;
        if s.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        s.velocity.y -= SHELL_GRAVITY * dt;
        let next = tr.translation + s.velocity * dt;
        let floor = floor_y(next.x, next.z, tr.translation.y + 0.3) + 0.01;
        if next.y < floor && s.velocity.y < 0.0 {
            s.velocity = Vec3::new(s.velocity.x, -s.velocity.y, s.velocity.z) * SHELL_BOUNCE;
            s.spin *= SHELL_BOUNCE;
            tr.translation = Vec3::new(next.x, floor, next.z);
        } else {
            tr.translation = next;
        }
        let spin = s.spin * dt;
        tr.rotate(Quat::from_euler(EulerRot::XYZ, spin.x, spin.y, spin.z));
    }
}
