//! Frag grenades, as in the game capture: hold the throw button to charge (the HUD meter), let go
//! to throw - the charge sets how far. Charging starts with the Frag's event sound (e43166d1,
//! heard as the meter appears in the capture), the grenade whooshes off (its object sound),
//! bounces silently and its fuse starts when it lands (release to blast ~2.5 s in the
//! capture: about a second of flight plus the definition's 1.5 s). The blast: a white flash, the
//! game's fireball flipbook (wide), brown smoke that hangs for a couple of seconds, a scorch mark
//! left on the ground, and the screen tinted red when it goes off close by.
//!
//! Data (objecttypes-<level>, `inventory-grenade`): the Frag item has `timer` (fuse, 1.5 s),
//! mesh-name (model), sound-name (10318b29, the throw whoosh - matched against the capture),
//! an object event sound (h_fa04e025, e43166d1: the charge / gauge sound) and h_053c429f -> the explosion's
//! weapon definition (Damage radius 8, bullet impact sound h_e5618348 = fb4b3604). Throw speeds
//! and lob are the demo's own.

use super::*;

/// throw speed range (m/s) over the charge, and extra upward angle over the crosshair (radians)
const MIN_THROW: f32 = 5.0;
const MAX_THROW: f32 = 15.0;
const THROW_LOB: f32 = 0.3;
const GRENADE_GRAVITY: f32 = 9.8;
const GRENADE_RADIUS: f32 = 0.06;
/// bounce: vertical restitution, horizontal speed kept per bounce
const RESTITUTION: f32 = 0.35;
const GROUND_FRICTION: f32 = 0.6;
/// the game's fireball-to-smoke flipbook (common textures, 4 x 4 frames, stored order) and a
/// smoke cloud (tinted brown, as the capture's smoke)
const FIREBALL: u32 = 0x184B_75CD;
const SMOKE: u32 = 0xF755_A49B;
const EXPLOSION_TIME: f32 = 1.1;
/// fireball height as a fraction of the blast radius; it is drawn this much wider than tall
const FIREBALL_SIZE: f32 = 0.7;
const FIREBALL_WIDE: f32 = 1.5;
const SMOKE_TIME: f32 = 2.4;
const SCORCH_TIME: f32 = 20.0;

/// Throw velocity for a charge of `power` (0..1) toward the crosshair ray.
pub fn throw_velocity(ray: Vec3, power: f32) -> Vec3 {
    let flat = Vec3::new(ray.x, 0.0, ray.z).normalize_or(Vec3::NEG_Z);
    let pitch = (ray.y.clamp(-1.0, 1.0).asin() + THROW_LOB).clamp(-0.3, 1.2);
    (flat * pitch.cos() + Vec3::Y * pitch.sin()) * (MIN_THROW + (MAX_THROW - MIN_THROW) * power.clamp(0.0, 1.0))
}

fn rgba_image(w: u32, h: u32, px: Vec<u8>) -> Image {
    Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, px,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
               bevy::asset::RenderAssetUsages::default())
}

/// Radial soft disc (white, alpha by `alpha(r)` for r in 0..1), for the flash and the scorch.
fn radial(n: u32, alpha: impl Fn(f32, f32) -> f32) -> Image {
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 + 0.5) / n as f32 * 2.0 - 1.0, (y as f32 + 0.5) / n as f32 * 2.0 - 1.0);
            let a = alpha((dx * dx + dy * dy).sqrt(), dy.atan2(dx));
            px.extend_from_slice(&[255, 255, 255, (a.clamp(0.0, 1.0) * 255.0) as u8]);
        }
    }
    rgba_image(n, n, px)
}

/// The frag grenade: definition, explosion definition, model and effect textures.
pub struct GrenadeKit {
    pub def: WeaponDef,
    pub blast_radius: f32,
    /// damage at the centre of the blast (falls off to 0 at the radius)
    pub damage: f32,
    pub explosion_sound: u32,
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Vec3)>,
    quad: Handle<Mesh>,
    floor_quad: Handle<Mesh>,
    fireball: Option<Handle<Image>>,
    smoke: Option<Handle<Image>>,
    flash: Handle<Image>,
    scorch: Handle<Image>,
}

impl GrenadeKit {
    /// The level's Frag (a weapon definition with a fuse and the "Frag" name), if any.
    pub fn load(game: &mut Game, assets: &mut ModelAssets) -> Option<Self> {
        let def = game.weapons.values().filter(|w| w.fuse > 0.0 && w.label == "Frag").min_by_key(|w| w.name)
            .or_else(|| game.weapons.values().filter(|w| w.fuse > 0.0).min_by_key(|w| w.name))?.clone();
        let blast = game.weapons.get(&def.projectile).cloned().unwrap_or_default();
        let mut parts = vec![];
        match WeaponModel::load(game, def.archetype) {
            Ok(m) => for p in &m.parts {
                for (mesh, mat) in bf_viewer::scene::static_meshes(game, &p.geosets, assets, true) {
                    parts.push((mesh, mat, p.offset));
                }
            },
            Err(e) => warn!("frag model: {e}"),
        }
        if parts.is_empty() {
            let mat = assets.materials.add(StandardMaterial { base_color: Color::srgb(0.35, 0.3, 0.2), ..default() });
            parts.push((assets.meshes.add(Sphere::new(GRENADE_RADIUS).mesh().ico(2).unwrap()), mat, Vec3::ZERO));
        }
        info!("grenade {} (h_{:08x}): fuse {:.1}s, blast radius {}, sounds throw {:08x} charge {:08x} explode {:08x}, {} parts",
              def.label, def.name, def.fuse, blast.blast_radius, def.object_sound, def.arm_sound, blast.impact_sound, parts.len());
        let mut tex = |id: u32| game.texture_rgba(id).map(|(w, h, px)| assets.images.add(rgba_image(w, h, px)));
        let (fireball, smoke) = (tex(FIREBALL), tex(SMOKE));
        let flash = assets.images.add(radial(64, |r, _| (1.0 - r).powf(1.5)));
        // scorch: dark in the middle, ragged soft edge
        let scorch = assets.images.add(radial(128, |r, a| {
            let edge = 0.75 + 0.12 * (a * 5.0).sin() + 0.08 * (a * 11.0 + 1.3).sin();
            0.85 * (1.0 - ((r - 0.25) / (edge - 0.25)).clamp(0.0, 1.0)).powf(0.8)
        }));
        Some(Self { blast_radius: if blast.blast_radius > 0.0 { blast.blast_radius } else { 8.0 },
                    damage: if blast.damage > 0.0 { blast.damage } else { 65.0 },
                    explosion_sound: blast.impact_sound, def, parts, fireball, smoke, flash, scorch,
                    quad: assets.meshes.add(Rectangle::new(1.0, 1.0)),
                    floor_quad: assets.meshes.add(Plane3d::default().mesh().size(1.0, 1.0)) })
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

#[derive(Component)]
struct Grenade {
    velocity: Vec3,
    fuse: f32,
    /// the fuse runs once the grenade has come down
    landed: bool,
    spin: Vec3,
}

/// One piece of a blast, with its own lifetime and look.
#[derive(Component)]
struct Effect {
    kind: EffectKind,
    age: f32,
    life: f32,
    size: f32,
    material: Handle<StandardMaterial>,
}

#[derive(Clone, Copy)]
enum EffectKind {
    Flash,
    Fireball { light: Entity },
    Smoke { delay: f32 },
    Scorch,
}

/// Red full-screen tint after a blast close by (0..1).
#[derive(Resource, Default)]
struct ScreenTint(f32);

#[derive(Component)]
struct TintOverlay;

/// A grenade in the hand between the throw clip's reach and release events.
#[derive(Resource, Default)]
struct Held(Option<Entity>);

pub fn plugin(app: &mut App) {
    app.init_resource::<Held>()
        .init_resource::<ScreenTint>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| {
            commands.spawn((TintOverlay, GlobalZIndex(-1), BackgroundColor(Color::NONE),
                            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
        }).after(snapshot_entities))
        .add_systems(Update, (hold_grenade, launch_grenades, fly_grenades, effects).chain().after(update_player).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// Show the grenade in the throwing hand while the throw has it.
fn hold_grenade(mut commands: Commands, player: Res<Player>, mut held: ResMut<Held>) {
    let Some(l) = player.loaded.as_ref() else { return };
    let want = player.throwing.as_ref().is_some_and(|t| t.grabbed && !t.released);
    match (want, held.0, &l.grenade, l.throw_hand) {
        (true, None, Some(kit), Some((bone, point))) => {
            held.0 = Some(kit.spawn_model(&mut commands, Transform::from_translation(point), Some(l.joints[bone])));
        }
        (false, Some(e), ..) => {
            commands.entity(e).despawn();
            held.0 = None;
        }
        _ => {}
    }
}

/// Spawn the grenades released this frame (with the throw whoosh).
fn launch_grenades(mut commands: Commands, mut player: ResMut<Player>, mut next_test: Local<f32>) {
    let mut throws = std::mem::take(&mut player.thrown);
    // test hook: BF_TEST_EXPLOSION=1 drops a grenade 6 m ahead every 1.5 s, exploding on contact
    let test = std::env::var("BF_TEST_EXPLOSION").is_ok();
    if test && player.sim_time >= *next_test {
        *next_test = player.sim_time + 1.5;
        throws.push((Vec3::new(player.position.x, floor_y(player.position.x, player.position.z - 6.0, player.position.y + GROUND + 1.0) + GRENADE_RADIUS, player.position.z - 6.0), Vec3::ZERO));
    }
    let Some(kit) = player.loaded.as_ref().and_then(|l| l.grenade.as_ref()) else { return };
    let whoosh = kit.def.object_sound;
    let mut launched = 0;
    for (pos, velocity) in throws {
        let e = kit.spawn_model(&mut commands, Transform::from_translation(pos), None);
        let spin = velocity.cross(Vec3::Y).normalize_or(Vec3::X) * -12.0;
        commands.entity(e).insert(Grenade { velocity, fuse: if test { 0.0 } else { kit.def.fuse }, landed: test, spin });
        launched += !test as usize;
    }
    if whoosh != 0 {
        for _ in 0..launched {
            player.sound_queue.push((whoosh, 0.8));
        }
    }
}

/// Grenade flight: gravity, bounces off the ground and the pillars, the blast when the fuse ends.
fn fly_grenades(
    mut commands: Commands,
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    game: Res<GameData>,
    mut tint: ResMut<ScreenTint>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grenades: Query<(Entity, &mut Grenade, &mut Transform)>,
    mut blasts: ResMut<super::pickups::Blasts>,
) {
    let dt = frame_dt(&time);
    let Some(l) = player.loaded.take() else { return };
    let p = &mut *player;
    if let Some(kit) = l.grenade.as_ref() {
        for (e, mut g, mut tr) in &mut grenades {
            if g.landed {
                g.fuse -= dt;
            }
            if g.fuse <= 0.0 {
                let at = tr.translation;
                commands.entity(e).despawn();
                let near = at.distance(p.position);
                p.sound_queue.push((kit.explosion_sound, (1.0 - near / 60.0).clamp(0.3, 1.0)));
                tint.0 = tint.0.max(0.6 * (1.0 - near / (kit.blast_radius * 1.4)).clamp(0.0, 1.0));
                blast(&mut commands, &mut materials, kit, at, p.random(1000) as f32 / 1000.0);
                blasts.0.push((at, kit.blast_radius));
                // the blast hurts everyone in range (Damage max at the centre, nothing at the
                // radius); the hurt say a pain grunt
                for u in std::iter::once(&mut *p).chain(squad.0.iter_mut()) {
                    let d = u.position.distance(at);
                    if d < kit.blast_radius {
                        let k = 1.0 - d / kit.blast_radius;
                        let away = Vec3::new(u.position.x - at.x, 0.0, u.position.z - at.z).normalize_or(Vec3::X);
                        hurt(u, &game.0, kit.damage * k, HURT_CHATTER, (away * 6.0 + Vec3::Y * 4.0) * k, u.position + Vec3::Y * 1.0, -1);
                    }
                }
                continue;
            }
            g.velocity.y -= GRENADE_GRAVITY * dt;
            let mut pos = tr.translation + g.velocity * dt;
            let floor = floor_y(pos.x, pos.z, tr.translation.y) + GRENADE_RADIUS;
            let mut hit = 0.0f32;
            if pos.y < floor {
                pos.y = floor;
                if !g.landed {
                    // squadmates close by dive away (EVT_GRENADE_NEAR -> GOAL_DIVE)
                    for m in squad.0.iter_mut().filter(|m| !m.dead && m.position.distance(pos) < DIVE_RADIUS) {
                        m.dive_from = Some(pos);
                    }
                }
                g.landed = true;
                if g.velocity.y < 0.0 {
                    hit = hit.max(-g.velocity.y);
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
                    if half - d.x.abs() < half - d.z.abs() {
                        pos.x = c.x + half * d.x.signum();
                        hit = hit.max(g.velocity.x.abs());
                        g.velocity.x = -g.velocity.x * 0.5;
                    } else {
                        pos.z = c.z + half * d.z.signum();
                        hit = hit.max(g.velocity.z.abs());
                        g.velocity.z = -g.velocity.z * 0.5;
                    }
                }
            }
            // the map's objects: push out and bounce off
            if let Some(a) = world::arena() {
                let at = Vec2::new(pos.x, pos.z);
                let q = a.push_out(at, GRENADE_RADIUS, pos.y - GRENADE_RADIUS, pos.y + GRENADE_RADIUS, 0.0);
                if q != at {
                    let n = (q - at).normalize_or_zero();
                    let v = Vec2::new(g.velocity.x, g.velocity.z);
                    let vn = v.dot(n);
                    if vn < 0.0 {
                        hit = hit.max(-vn);
                        let v = v - n * vn * 1.5;
                        g.velocity.x = v.x;
                        g.velocity.z = v.y;
                    }
                    pos.x = q.x;
                    pos.z = q.y;
                }
            }
            let _ = hit;                                   // bounces are silent (the clank is the gauge's)
            tr.translation = pos;
            let spin = g.spin * dt;
            if spin.length_squared() > 1e-8 {
                tr.rotation = Quat::from_scaled_axis(spin) * tr.rotation;
            }
        }
    }
    player.loaded = Some(l);
}

/// Spawn a blast at `at`: flash, fireball (+ light), two smoke clouds, scorch mark.
fn blast(commands: &mut Commands, materials: &mut Assets<StandardMaterial>, kit: &GrenadeKit, at: Vec3, rnd: f32) {
    let ground = Vec3::new(at.x, at.y.max(floor_y(at.x, at.z, at.y + 0.5)), at.z);
    let size = kit.blast_radius * FIREBALL_SIZE;
    let mut spawn = |commands: &mut Commands, kind: EffectKind, mesh: &Handle<Mesh>, mat: StandardMaterial, pos: Vec3, size: f32, life: f32| {
        let material = materials.add(mat);
        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                        Transform::from_translation(pos).with_scale(Vec3::splat(0.001)),
                        Effect { kind, age: 0.0, life, size, material }));
    };
    let billboard = |tex: Option<Handle<Image>>, color: Color, alpha_mode: AlphaMode| StandardMaterial {
        base_color: color, base_color_texture: tex, unlit: true, alpha_mode, double_sided: true, cull_mode: None, ..default()
    };
    spawn(commands, EffectKind::Flash, &kit.quad, billboard(Some(kit.flash.clone()), Color::srgb(3.0, 3.0, 2.6), AlphaMode::Add),
          ground + Vec3::Y * 0.8, size * 0.9, 0.15);
    let light = commands.spawn((PointLight { intensity: 4.0e6 * bf_viewer::level_scene::POINT_LIGHT_SCALE, range: kit.blast_radius * 3.0, color: Color::srgb(1.0, 0.7, 0.35),
                                            shadows_enabled: false, ..default() },
                                Transform::from_translation(ground + Vec3::Y * 0.8))).id();
    let fire_color = if kit.fireball.is_some() { Color::srgb(2.5, 2.2, 2.0) } else { Color::srgba(1.0, 0.6, 0.2, 0.8) };
    spawn(commands, EffectKind::Fireball { light }, &kit.quad,
          StandardMaterial { uv_transform: frame_uv(0), ..billboard(kit.fireball.clone(), fire_color, AlphaMode::Blend) },
          ground + Vec3::Y * size * 0.3, size, EXPLOSION_TIME);
    for (k, dx) in [(0.25, -0.35), (0.45, 0.3)] {
        let side = Vec3::new(dx * size * (0.8 + 0.4 * rnd), 0.0, 0.0);
        spawn(commands, EffectKind::Smoke { delay: k }, &kit.quad,
              billboard(kit.smoke.clone(), Color::srgba(0.45, 0.3, 0.18, 0.0), AlphaMode::Blend),
              ground + side + Vec3::Y * size * 0.35, size * 1.1, SMOKE_TIME);
    }
    let mut scorch = billboard(Some(kit.scorch.clone()), Color::srgba(0.05, 0.04, 0.035, 0.9), AlphaMode::Blend);
    scorch.unlit = false;
    scorch.perceptual_roughness = 1.0;
    scorch.depth_bias = 50.0;
    spawn(commands, EffectKind::Scorch, &kit.floor_quad, scorch, Vec3::new(at.x, floor_y(at.x, at.z, at.y + 0.5) + 0.01, at.z),
          kit.blast_radius * 0.55, SCORCH_TIME);
}

/// UV window of flipbook frame `i` (row by row; unlike the HUD icons these frames are upright).
fn frame_uv(i: usize) -> bevy::math::Affine2 {
    let (col, row) = ((i % 4) as f32, (i / 4).min(3) as f32);
    bevy::math::Affine2::from_scale_angle_translation(Vec2::splat(0.25), 0.0, Vec2::new(col * 0.25, row * 0.25))
}

/// Blast pieces over their lifetimes (billboards face the camera), and the red screen tint.
fn effects(
    mut commands: Commands,
    time: Res<Time>,
    mut tint: ResMut<ScreenTint>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(Entity, &mut Effect, &mut Transform), Without<MainCamera>>,
    camera: Query<&Transform, With<MainCamera>>,
    mut lights: Query<&mut PointLight>,
    mut overlay: Query<&mut BackgroundColor, With<TintOverlay>>,
) {
    let dt = frame_dt(&time);
    let facing = camera.single().map(|c| c.rotation).unwrap_or_default();
    for (e, mut x, mut tr) in &mut q {
        x.age += dt;
        let k = (x.age / x.life).min(1.0);
        let s = x.size;
        let mut color = None;
        match x.kind {
            EffectKind::Flash => {
                tr.rotation = facing;
                tr.scale = Vec3::splat(s * (0.6 + 0.8 * k));
                color = Some(Color::srgb(3.0 * (1.0 - k), 3.0 * (1.0 - k), 2.6 * (1.0 - k)));
            }
            EffectKind::Fireball { light } => {
                tr.rotation = facing;
                tr.scale = Vec3::new(s * FIREBALL_WIDE, s, 1.0) * (0.75 + 0.25 * k);
                if let Some(m) = materials.get_mut(&x.material) {
                    m.uv_transform = frame_uv((k * 16.0) as usize);
                }
                if let Ok(mut l) = lights.get_mut(light) {
                    l.intensity = 4.0e6 * bf_viewer::level_scene::POINT_LIGHT_SCALE * (1.0 - (k * 2.5).min(1.0)).powi(2);
                }
                if k >= 1.0 {
                    commands.entity(light).despawn();
                }
            }
            EffectKind::Smoke { delay } => {
                tr.rotation = facing;
                let t = ((x.age - delay) / (x.life - delay)).clamp(0.0, 1.0);
                tr.scale = Vec3::new(s * 1.3, s, 1.0) * (0.6 + 0.6 * t);
                tr.translation.y += dt * 0.5 * (x.age > delay) as i32 as f32;
                let a = if x.age < delay { 0.0 } else { (t / 0.15).min(1.0) * (1.0 - t).powf(1.3) * 0.85 };
                color = Some(Color::srgba(0.45, 0.3, 0.18, a));
            }
            EffectKind::Scorch => {
                tr.scale = Vec3::new(s, 1.0, s);
                let fade = ((x.life - x.age) / 5.0).clamp(0.0, 1.0);
                color = Some(Color::srgba(0.05, 0.04, 0.035, 0.9 * fade));
            }
        }
        if let (Some(c), Some(m)) = (color, materials.get_mut(&x.material)) {
            m.base_color = c;
        }
        if k >= 1.0 {
            commands.entity(e).despawn();
        }
    }
    tint.0 *= (-3.0 * dt).exp();
    if let Ok(mut bg) = overlay.single_mut() {
        bg.0 = Color::srgba(0.9, 0.08, 0.02, tint.0);
    }
}
