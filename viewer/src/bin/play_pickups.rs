//! Health pickups (#20), from the levels' inventory-objects and their objecttypes `<inventory>`
//! entries (`Game::items`):
//!
//!  - Medkit (h_f5123ace, function-type 5 = IFSET_GENERIC_HEALING): walking over one takes it
//!    into the squad's shared inventory, up to its stack-limit (25). Medkits are an item of the
//!    item box (see `Item`): the first one taken selects it, marked NEW; with it selected the
//!    use key (G) uses one, or says "No need to heal" at full health (the game's string). Full:
//!    "<name> cannot pick up Medkit." (the reticule-action 2 path of the game's prompt code)
//!    and it stays where it is.
//!  - Healing Garo Fruit (h_e711067e, function-type 19 = IFSET_POWERUP_MEDKIT): eaten as it's
//!    walked over; left alone at full health. The HEALTH / STAMINA POWER power-ups share
//!    function-type 19 but have an idle effect: they're #15's, not handled here.
//!
//! Each pickup adds a line under the character, "<n>x <name>" (counted up while more of the
//! same are taken: the capture's "22x 11mm Ammo", "3x Medkit" -> "4x Medkit"), gone after
//! FEED_TIME s without one. Each group of medkits has one soft green glow over its middle, all
//! the time (todo/medkits glow.png: a marker for where medkits are).
//!
//! Using a medkit (todo/medic + intenvory use case.mp4) plays the stance's use_item overlay
//! (Sc_w1_/Sc_w2_use_item, ~1 s): the red used medkit (the carried type's own model) is in the
//! throwing hand from its 0a6e8f79 event; at its 19f8311b event it heals, plays MEDKIT_SOUND,
//! the character now and then says a "healed" line, and the medkit is let go: it falls and
//! bounces to rest on the ground, and stays.
//!
//! Every pickup on the ground (and every used medkit) is loose: it lies tilted to the ground
//! under it, a character walking into it kicks it along, a grenade blast throws it, and it
//! tumbles, bounces and slides to rest (simple rigid-body steps, see `physics`; loose ones
//! push off each other so they don't sink into one another). Pickups with an idle effect
//! (the hovering power-ups) are left alone.
//!
//! Each heals its item type's h_0a811e94 (see `ItemType`), up to the character's maximum: the
//! fruit 40; a medkit what the carried item's says. The placed Medkit (h_f5123ace, 60) gives,
//! by its pickup-archetype, the inventory Medkit h_192d5337 (80): so using one heals 80. (What
//! the placed one's own 60 is for isn't known.) Taking one plays its pickup-sound; it comes back
//! after RESPAWN s (the game's respawn time isn't found). Only the player picks up for now.

use super::*;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::Placed;
use bf_viewer::scene::ModelAssets;
use bevy::render::mesh::MeshAabb;

const MEDKIT: i64 = 5;
const FRUIT: i64 = 19;
/// The sound of a medkit used (chosen by ear): the Sound whose file is h_15331992 (named as
/// 92193315, its bytes as stored in sounds-<level>.xmb), id h_1538baad in all 54 levels'
/// banks, 1.5 s.
const MEDKIT_SOUND: u32 = 0x1538_BAAD;
/// The relieved line after a medkit: line_tag 12c35d69 in each <name>_chatter file, whose
/// block's h_ea21ae4b is 30 (read as the chance, %). In todo/medic + intenvory use case.mp4
/// Tex says his e707f108 with the use (spectrogram match), Brutus nothing at his.
const HEALED_CHATTER: u32 = 0x12C3_5D69;
const HEALED_CHANCE: usize = 30;
/// How long a taken pickup stays away (s): a guess.
const RESPAWN: f32 = 30.0;
/// How near (m) the player's feet must come: across, and up or down.
const REACH: f32 = 1.0;
const REACH_UP: f32 = 1.5;
/// The medkit spots' glow (todo/medkits glow.png): one per group (medkits within GLOW_GROUP m
/// across of another, and less than a metre up or down), a soft green halo over the group's middle, GLOW_UP m above the ground, facing
/// the camera (added to the picture, unlit), GLOW_SIZE m across. It's drawn up to GLOW_NEAR m
/// (at most GLOW_NEAR_PART of the way) toward the camera along its line of sight, made smaller
/// to look the same, so the ground in front of the medkits doesn't cut it.
const GLOW_GROUP: f32 = 3.5;
const GLOW_SIZE: f32 = 1.6;
const GLOW_UP: f32 = 0.3;
const GLOW_NEAR: f32 = 1.0;
const GLOW_NEAR_PART: f32 = 0.4;
const GLOW_GREEN: Color = Color::srgba(0.3, 0.9, 0.4, 0.35);
/// A group's medkits are laid out in an even grid round its middle (todo/medkits glow.png: four
/// in two rows, nearly touching), each turned the way its first one is give or take up to
/// GROUP_TWIST (rad, fixed per medkit: not too neat), GROUP_GAP of a medkit's size apart edge
/// to edge so none overlap even turned; the levels place them up to ~3 m apart.
const GROUP_GAP: f32 = 0.3;
const GROUP_TWIST: f32 = 0.2;
/// A medkit's footprint (m, across and deep) if its mesh can't be measured.
const MEDKIT_FOOTPRINT: Vec2 = Vec2::new(0.5, 0.3);
/// The used medkit let go: its push (m/s, forward of the character and up), gravity, bounce,
/// how many stay on the ground at once.
const DROP_FORWARD: f32 = 0.8;
const DROP_UP: f32 = 0.6;
const DROP_GRAVITY: f32 = 9.8;
const DROP_RADIUS: f32 = 0.06;
const DROP_BOUNCE: f32 = 0.3;
const DROP_FRICTION: f32 = 0.55;
const DROPS_KEPT: usize = 12;
/// Loose pickups: how near (m, across) a character's feet must come to kick one, and how high
/// above or below; the kick (the character's speed carried, a push away, a hop up, m/s); a
/// blast's reach (its radius times this) and throw (m/s at its middle); how far apart (m)
/// the steepest they lie tilted (rad); slower than this (m/s) a bounce
/// settles. A kick sends it out ahead faster than the character and off to the side it was
/// on (KICK_SIDE: how much the side it's off the character's line counts).
const KICK_REACH: f32 = 0.45;
const KICK_UP_REACH: f32 = 0.8;
const KICK_CARRY: f32 = 1.15;
const KICK_AWAY: f32 = 1.2;
const KICK_SIDE: f32 = 2.5;
const KICK_HOP: f32 = 1.4;
/// After a kick it can't be kicked again for this long (s), so it isn't pushed along.
const KICK_AGAIN: f32 = 0.6;
const BLAST_REACH: f32 = 1.5;
const BLAST_THROW: f32 = 9.0;
const BODY_APART: f32 = 0.28;
const MAX_TILT: f32 = 0.45;
const SETTLE: f32 = 0.35;
/// Landing faster than this (m/s, down) bounces; slower it's sliding on the ground, slowed by
/// SLIDE_FRICTION (m/s per s).
const BOUNCE_SPEED: f32 = 1.0;
const SLIDE_FRICTION: f32 = 5.0;
/// The used medkit's cross: its texture's blue (the cross) turned red; the case keeps its colour.
/// How long a pickup message shows (s), a pickup's line stays, and an item stays NEW.
const MESSAGE_TIME: f32 = 2.0;
const FEED_TIME: f32 = 3.0;
const NEW_TIME: f32 = 3.0;

/// The pickup lines under the character: (item name, how many taken, seconds left).
#[derive(Resource, Default)]
pub struct PickupFeed(pub Vec<(String, i64, f32)>);

struct Pickup {
    entity: Entity,
    kind: u32,
    at: Vec3,
    /// seconds until it's back (0: there)
    gone: f32,
    /// the "cannot pick up" message has shown since the player came near
    told: bool,
}

#[derive(Resource, Default)]
struct Pickups {
    list: Vec<Pickup>,
    ready: bool,
}

/// The used medkit's model (the carried type's archetype): parts and their offsets.
#[derive(Resource, Default)]
struct UsedMedkit {
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Vec3)>,
    /// the one in the hand, while the use has it
    held: Option<Entity>,
    /// the ones let go, oldest first
    dropped: Vec<Entity>,
}

/// A medkit group's glow: where it is, faced to the camera each frame.
#[derive(Component)]
struct Glow(Vec3);

/// A loose pickup (or used medkit): its motion, how high its origin sits above the ground at
/// rest, and whether it's moving (at rest it's left alone until kicked or thrown).
#[derive(Component)]
struct Body {
    velocity: Vec3,
    spin: Vec3,
    lift: f32,
    moving: bool,
    /// seconds until it can be kicked again
    kicked: f32,
}

/// Grenade blasts this frame (where, radius), for the loose pickups (play_grenade.rs adds them).
#[derive(Resource, Default)]
pub struct Blasts(pub Vec<(Vec3, f32)>);

/// A used medkit's cross glow: red, at full strength.
const USED_GLOW: Color = Color::srgb(0.9, 0.12, 0.08);

pub fn plugin(app: &mut App) {
    app.init_resource::<PickupFeed>()
        .init_resource::<Blasts>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| {
            commands.insert_resource(Pickups::default());
            commands.insert_resource(PickupFeed::default());
            commands.insert_resource(UsedMedkit::default());
            commands.insert_resource(Blasts::default());
        }).after(setup))
        .add_systems(Update, (find_pickups, physics, take_pickups, use_medkit, medkit_used, hold_medkit, face_glows).chain()
            .after(update_player).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// The level's health pickups, once its objects are spawned (their own Transform: placed objects
/// are roots, and on their first frame the GlobalTransform isn't worked out yet), the medkit
/// spots' glow, and the used medkit's model.
#[allow(clippy::too_many_arguments)]
fn find_pickups(mut commands: Commands, mut game: ResMut<GameData>, mut pickups: ResMut<Pickups>, mut player: ResMut<Player>,
                mut used: ResMut<UsedMedkit>, mut placed: Query<(Entity, &Placed, &mut Transform)>,
                children: Query<&Children>, parts: Query<(&Transform, Option<&Mesh3d>), Without<Placed>>,
                mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
                mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>) {
    if pickups.ready || player.loaded.is_none() {
        return;
    }
    pickups.ready = true;
    // test hook: BF_TEST_HEALTH=<hp> starts the player hurt
    if let Some(hp) = std::env::var("BF_TEST_HEALTH").ok().and_then(|v| v.parse::<f32>().ok()) {
        player.health = hp.min(player.max_health);
    }
    pickups.list = placed.iter()
        .filter(|(_, p, _)| p.tag == h("inventory-object") && !game.0.idle_effects.contains_key(&p.kind)
            && game.0.items.get(&p.kind).is_some_and(|i| i.function == MEDKIT || i.function == FRUIT))
        .map(|(entity, p, t)| Pickup { entity, kind: p.kind, at: t.translation, gone: 0.0, told: false })
        .collect();
    // one glow over each group of medkits (it stays when they're taken: it marks where they are)
    let kits: Vec<usize> = (0..pickups.list.len())
        .filter(|&i| game.0.items.get(&pickups.list[i].kind).is_some_and(|t| t.function == MEDKIT)).collect();
    let spots: Vec<Vec3> = kits.iter().map(|&i| pickups.list[i].at).collect();
    let mut group = (0..spots.len()).collect::<Vec<_>>();
    fn root(g: &mut [usize], mut i: usize) -> usize {
        while g[i] != i {
            g[i] = g[g[i]];
            i = g[i];
        }
        i
    }
    for i in 0..spots.len() {
        for j in i + 1..spots.len() {
            if spots[i].xz().distance(spots[j].xz()) < GLOW_GROUP && (spots[i].y - spots[j].y).abs() < 1.0 {
                let (a, b) = (root(&mut group, i), root(&mut group, j));
                group[a] = b;
            }
        }
    }
    let mut middles: HashMap<usize, (Vec3, f32)> = HashMap::new();
    for i in 0..spots.len() {
        let m = middles.entry(root(&mut group, i)).or_insert((Vec3::ZERO, 0.0));
        m.0 += spots[i];
        m.1 += 1.0;
    }
    // each group's medkits in an even grid round its middle, turned alike
    let mut members: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..spots.len() {
        members.entry(root(&mut group, i)).or_default().push(i);
    }
    for (r, mut list) in members {
        if list.len() < 2 {
            continue;
        }
        // stable order: by where the level put them
        list.sort_by(|&a, &b| spots[a].z.total_cmp(&spots[b].z).then(spots[a].x.total_cmp(&spots[b].x)));
        let c = middles[&r].0 / middles[&r].1;
        let Ok((_, _, first)) = placed.get(pickups.list[kits[list[0]]].entity) else { continue };
        let (turn, scale) = (first.rotation, first.scale);
        let size = footprint(pickups.list[kits[list[0]]].entity, &children, &parts, &meshes, scale).unwrap_or(MEDKIT_FOOTPRINT);
        let (yaw, _, _) = turn.to_euler(EulerRot::YXZ);
        let (across, deep) = (Quat::from_rotation_y(yaw) * Vec3::X, Quat::from_rotation_y(yaw) * Vec3::Z);
        let n = list.len();
        let cols = (n as f32).sqrt().ceil() as usize;
        let rows = n.div_ceil(cols);
        let step = size * (1.0 + GROUP_GAP);
        for (k, &i) in list.iter().enumerate() {
            let (row, col) = (k / cols, k % cols);
            // a short last row is centred
            let in_row = if row == rows - 1 { n - row * cols } else { cols };
            let off = across * (col as f32 - (in_row as f32 - 1.0) / 2.0) * step.x
                + deep * (row as f32 - (rows as f32 - 1.0) / 2.0) * step.y;
            let pickup = &mut pickups.list[kits[i]];
            let (x, z) = (c.x + off.x, c.z + off.z);
            let y = pickup.at.y - floor_y(pickup.at.x, pickup.at.z, pickup.at.y + 0.5) + floor_y(x, z, pickup.at.y + 0.5);
            pickup.at = Vec3::new(x, y, z);
            if let Ok((_, _, mut t)) = placed.get_mut(pickup.entity) {
                t.translation = pickup.at;
                // a fixed twist per medkit, from where the level put it
                let seed = (spots[i].x * 12.9898 + spots[i].z * 78.233).sin() * 43758.547;
                t.rotation = Quat::from_rotation_y((seed.fract() * 2.0 - 1.0) * GROUP_TWIST) * turn;
            }
        }
    }
    let halo = images.add(radial_halo(64));
    let glow = materials.add(StandardMaterial { base_color: GLOW_GREEN, base_color_texture: Some(halo), unlit: true,
                                                alpha_mode: AlphaMode::Add, double_sided: true, cull_mode: None, fog_enabled: false,
                                                ..default() });
    let quad = meshes.add(Rectangle::new(GLOW_SIZE, GLOW_SIZE));
    for (sum, n) in middles.into_values() {
        let c = sum / n;
        let at = Vec3::new(c.x, floor_y(c.x, c.z, c.y + 0.5) + GLOW_UP, c.z);
        commands.spawn((Mesh3d(quad.clone()), MeshMaterial3d(glow.clone()), Transform::from_translation(at), Glow(at),
                        bevy::pbr::NotShadowCaster, Name::new("medkit glow")));
    }
    // every pickup on the ground is loose, lying tilted to the ground under it
    for (e, p, mut t) in &mut placed {
        if p.tag != h("inventory-object") || game.0.idle_effects.contains_key(&p.kind) {
            continue;
        }
        let ground = floor_y(t.translation.x, t.translation.z, t.translation.y + 0.5);
        let (yaw, _, _) = t.rotation.to_euler(EulerRot::YXZ);
        t.rotation = lie(t.translation, yaw);
        commands.entity(e).insert(Body { velocity: Vec3::ZERO, spin: Vec3::ZERO, lift: (t.translation.y - ground).max(0.0), moving: false, kicked: 0.0 });
    }
    if std::env::var("BF_PICKUP_LOG").is_ok() {
        for p in &pickups.list {
            let i = &game.0.items[&p.kind];
            println!("pickup h_{:08x} {} (function {}, stack {}, heals {}) at {:.1}", p.kind, i.label, i.function, i.stack_limit, i.health, p.at);
        }
    }
    // the used medkit: the carried type's model (the placed medkit's pickup-archetype)
    let carried = game.0.items.values().find(|i| i.function == MEDKIT && i.gives != 0).map(|i| i.gives);
    if let Some(arch) = carried.and_then(|k| game.0.object_meshes.get(&k).copied()) {
        if let Ok(m) = bf_viewer::bf::weapon::WeaponModel::load(&game.0, arch) {
            let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
            for part in &m.parts {
                let meshes_of = bf_viewer::scene::static_meshes(&mut game.0, &part.geosets, &mut assets, true);
                for ((mesh, mat), g) in meshes_of.into_iter().zip(&part.geosets) {
                    // its cross's glow (the flat glow quad): red, full strength
                    if bf_viewer::level_scene::is_glow_decal(&game.0, g) {
                        let glow = assets.materials.add(StandardMaterial { base_color: USED_GLOW, unlit: true, alpha_mode: AlphaMode::Add,
                                                                           depth_bias: 1.0, fog_enabled: false, ..default() });
                        used.parts.push((mesh, glow, part.offset));
                        continue;
                    }
                    // used, its cross is red (todo/medic + intenvory use case.mp4: the dropped
                    // ones on the ground; the model's own cross is blue)
                    let red = assets.materials.get(&mat).cloned().map(|mut m| {
                        let tex = m.base_color_texture.as_ref().and_then(|t| assets.images.get(t)).cloned();
                        if let Some(mut img) = tex {
                            red_cross(&mut img);
                            m.base_color_texture = Some(assets.images.add(img));
                        }
                        assets.materials.add(m)
                    }).unwrap_or(mat);
                    used.parts.push((mesh, red, part.offset));
                }
            }
        }
    }
}

fn take_pickups(time: Res<Time>, game: Res<GameData>, mut pickups: ResMut<Pickups>, mut player: ResMut<Player>,
                mut status: ResMut<UsePanel>, mut feed: ResMut<PickupFeed>, mut vis: Query<&mut Visibility>) {
    let dt = frame_dt(&time);
    for line in &mut feed.0 {
        line.2 -= dt;
    }
    feed.0.retain(|l| l.2 > 0.0);
    player.item_new = (player.item_new - dt).max(0.0);
    // the selected item ran out (the last grenade thrown): the next one carried
    if item_count(&player, ITEMS[player.item]) <= 0 {
        step_item(&mut player, 1);
    }
    let feet = player.position + Vec3::Y * GROUND;
    for p in &mut pickups.list {
        if p.gone > 0.0 {
            p.gone -= dt;
            if p.gone <= 0.0 {
                if let Ok(mut v) = vis.get_mut(p.entity) { *v = Visibility::Inherited; }
            }
            continue;
        }
        let near = !player.dead && Vec2::new(p.at.x - feet.x, p.at.z - feet.z).length() < REACH && (p.at.y - feet.y).abs() < REACH_UP;
        if !near {
            p.told = false;
            continue;
        }
        let Some(item) = game.0.items.get(&p.kind) else { continue };
        let taken = match item.function {
            MEDKIT if player.medkits < item.stack_limit => {
                // a first medkit: the item box shows it, NEW
                if player.medkits == 0 {
                    player.item = ITEMS.iter().position(|&i| i == Item::Medkit).unwrap_or(0);
                    player.item_new = NEW_TIME;
                }
                player.medkits = (player.medkits + item.amount.max(1)).min(item.stack_limit);
                player.medkit_kind = if item.gives != 0 { item.gives } else { p.kind };
                true
            }
            MEDKIT => {
                if !p.told {
                    let name = CHARACTERS[player.character % CHARACTERS.len()];
                    let mut c = name.chars();
                    let name = c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
                    status.message = Some((format!("{name} cannot pick up {}.", item.label), MESSAGE_TIME));
                    p.told = true;
                }
                false
            }
            _ if player.health < player.max_health => {
                player.health = (player.health + item.health).min(player.max_health);
                true
            }
            _ => false,
        };
        if taken {
            p.gone = RESPAWN;
            let n = item.amount.max(1);
            match feed.0.iter_mut().find(|l| l.0 == item.label) {
                Some(l) => { l.1 += n; l.2 = FEED_TIME; }
                None => feed.0.push((item.label.clone(), n, FEED_TIME)),
            }
            if let Ok(mut v) = vis.get_mut(p.entity) { *v = Visibility::Hidden; }
            if item.sound != 0 {
                player.sound_queue.push((item.sound, 1.0));
            }
            if std::env::var("BF_PICKUP_LOG").is_ok() {
                println!("took {} at {:.1}: health {:.0}/{:.0}, medkits {}", item.label, p.at, player.health, player.max_health, player.medkits);
            }
        }
    }
}

/// The use key (G) with the medkit selected: start using one (the game's "Use Health Pack"),
/// or say why not.
fn use_medkit(mut player: ResMut<Player>, mut status: ResMut<UsePanel>) {
    // test hook: BF_TEST_MEDKIT=<s> uses one at that time
    let test = std::env::var("BF_TEST_MEDKIT").ok().and_then(|v| v.parse::<f32>().ok())
        .is_some_and(|at| player.sim_time >= at && player.sim_time - at < 0.1 && !player.test_medkit_used);
    let selected = ITEMS[player.item] == Item::Medkit;
    if !((player.item_use && selected) || test) || player.dead || player.using.is_some() {
        return;
    }
    player.test_medkit_used |= test;
    if player.medkits <= 0 {
        status.message = Some(("No Medkits".into(), MESSAGE_TIME));
    } else if player.health >= player.max_health {
        status.message = Some(("No need to heal".into(), MESSAGE_TIME));
    } else if player.reloading.is_none() && player.throwing.is_none() && player.switching.is_none() {
        player.using = Some(0.0);
    }
}

/// The use_item clip's use event: heal, the sound, now and then a "healed" line, and let the
/// medkit go.
fn medkit_used(mut commands: Commands, game: Res<GameData>, mut player: ResMut<Player>, mut used: ResMut<UsedMedkit>,
               joints: Query<&GlobalTransform>) {
    if !player.item_used || player.medkits <= 0 {
        return;
    }
    player.medkits -= 1;
    let heal = game.0.items.get(&player.medkit_kind).map_or(60.0, |i| i.health);
    player.health = (player.health + heal).min(player.max_health);
    player.sound_queue.push((MEDKIT_SOUND, 1.0));
    // test hook: BF_TEST_HEALED=1 always says it
    if player.random(100) < HEALED_CHANCE || std::env::var("BF_TEST_HEALED").is_ok() {
        say(&mut player, &game.0, HEALED_CHATTER, 0.0);
    }
    // the medkit leaves the hand: forward of the character, a little up, tumbling
    let hand = player.loaded.as_ref().and_then(|l| l.throw_hand.map(|(bone, point)| (l.joints[bone], point)))
        .and_then(|(joint, point)| joints.get(joint).ok().map(|t| t.transform_point(point)))
        .unwrap_or(player.position + Vec3::Y * (CHEST + GROUND));
    let forward = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
    let k = player.random(1000) as f32 / 1000.0;
    let e = spawn_model(&mut commands, &used, Transform::from_translation(hand), None);
    commands.entity(e).insert(Body { velocity: forward * DROP_FORWARD + Vec3::Y * DROP_UP, spin: Vec3::new(4.0 + 3.0 * k, 2.0 * k, 3.0 - 2.0 * k),
                                     lift: DROP_RADIUS, moving: true, kicked: 0.0 });
    used.dropped.push(e);
    if used.dropped.len() > DROPS_KEPT {
        let old = used.dropped.remove(0);
        commands.entity(old).despawn();
    }
    if std::env::var("BF_PICKUP_LOG").is_ok() {
        println!("used a medkit: health {:.0}/{:.0}, {} left; sound h_{MEDKIT_SOUND:08x} (in the bank: {}), healed line: {:?}, dropped at {hand:.2}",
                 player.health, player.max_health, player.medkits, game.0.sounds.has(MEDKIT_SOUND), player.quote_in);
    }
}

/// A placed object's footprint (m, along its own x and z), from its meshes' bounds.
fn footprint(entity: Entity, children: &Query<&Children>, parts: &Query<(&Transform, Option<&Mesh3d>), Without<Placed>>,
             meshes: &Assets<Mesh>, scale: Vec3) -> Option<Vec2> {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let mut stack = vec![(entity, Transform::from_scale(scale))];
    while let Some((e, at)) = stack.pop() {
        if let Ok((t, mesh)) = parts.get(e) {
            let at = at * *t;
            if let Some(b) = mesh.and_then(|m| meshes.get(&m.0)).and_then(|m| m.compute_aabb()) {
                let (mn, mx) = (Vec3::from(b.min()), Vec3::from(b.max()));
                for corner in [mn, mx, Vec3::new(mn.x, mn.y, mx.z), Vec3::new(mx.x, mx.y, mn.z),
                               Vec3::new(mn.x, mx.y, mn.z), Vec3::new(mx.x, mn.y, mx.z), Vec3::new(mn.x, mx.y, mx.z), Vec3::new(mx.x, mn.y, mn.z)] {
                    let p = at.transform_point(corner);
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
            }
            if let Ok(kids) = children.get(e) {
                stack.extend(kids.iter().map(|k| (k, at)));
            }
        } else if let Ok(kids) = children.get(e) {
            stack.extend(kids.iter().map(|k| (k, at)));
        }
    }
    (lo.x <= hi.x).then(|| Vec2::new(hi.x - lo.x, hi.z - lo.z))
}

/// A soft round halo: white, alpha falling from the middle to nothing at the edge.
fn radial_halo(n: u32) -> Image {
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 + 0.5) / n as f32 * 2.0 - 1.0, (y as f32 + 0.5) / n as f32 * 2.0 - 1.0);
            let r = (dx * dx + dy * dy).sqrt();
            let a = (1.0 - r).clamp(0.0, 1.0).powf(2.2);
            px.extend_from_slice(&[255, 255, 255, (a * 255.0) as u8]);
        }
    }
    Image::new(bevy::render::render_resource::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, px,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
}

/// The medkit texture's cross (its blue: blue well over red and green) made red, as bright.
fn red_cross(img: &mut Image) {
    let Some(px) = img.data.as_mut() else { return };
    for c in px.chunks_exact_mut(4) {
        let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
        if b > r + 30 && b > g + 15 {
            c[0] = b as u8;
            c[1] = (g * 2 / 5) as u8;
            c[2] = (r * 2 / 5) as u8;
        }
    }
}

/// The medkit glows facing the camera, drawn a little toward it.
fn face_glows(camera: Query<&GlobalTransform, With<MainCamera>>, mut glows: Query<(&Glow, &mut Transform)>) {
    let Ok(cam) = camera.single() else { return };
    let eye = cam.translation();
    for (g, mut tr) in &mut glows {
        let d = eye.distance(g.0);
        if d < 0.01 {
            continue;
        }
        let pull = GLOW_NEAR.min(d * GLOW_NEAR_PART);
        tr.translation = g.0 + (eye - g.0) / d * pull;
        tr.scale = Vec3::splat((d - pull) / d);
        tr.look_at(eye, Vec3::Y);
    }
}

/// The used medkit's model under `parent` at `transform`.
fn spawn_model(commands: &mut Commands, used: &UsedMedkit, transform: Transform, parent: Option<Entity>) -> Entity {
    let e = commands.spawn((transform, Visibility::Inherited, Name::new("used medkit"))).id();
    if let Some(p) = parent {
        commands.entity(e).insert(ChildOf(p));
    }
    for (mesh, mat, offset) in &used.parts {
        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::from_translation(*offset), ChildOf(e)));
    }
    e
}

/// The medkit in the throwing hand while the use_item clip has it.
fn hold_medkit(mut commands: Commands, player: Res<Player>, mut used: ResMut<UsedMedkit>) {
    let Some(l) = player.loaded.as_ref() else { return };
    match (player.item_in_hand, used.held, l.throw_hand) {
        (true, None, Some((bone, point))) => {
            let e = spawn_model(&mut commands, &used, Transform::from_translation(point), Some(l.joints[bone]));
            used.held = Some(e);
        }
        (false, Some(e), _) => {
            commands.entity(e).despawn();
            used.held = None;
        }
        _ => {}
    }
}

/// The rest pose for something lying at `at` turned `yaw`: tilted to the ground under it (at
/// most MAX_TILT).
fn lie(at: Vec3, yaw: f32) -> Quat {
    let e = 0.15;
    let top = at.y + 0.5;
    let dx = floor_y(at.x + e, at.z, top) - floor_y(at.x - e, at.z, top);
    let dz = floor_y(at.x, at.z + e, top) - floor_y(at.x, at.z - e, top);
    let mut n = Vec3::new(-dx / (2.0 * e), 1.0, -dz / (2.0 * e)).normalize();
    if n.angle_between(Vec3::Y) > MAX_TILT {
        // a step or a wall edge under it, not a slope
        n = Vec3::Y;
    }
    Quat::from_rotation_arc(Vec3::Y, n) * Quat::from_rotation_y(yaw)
}

/// Loose pickups: kicked by the characters walking into them, thrown by blasts, then gravity,
/// bounces off the ground, sliding to rest tilted to it; loose ones pushed apart. The pickups'
/// spots follow them.
#[allow(clippy::too_many_arguments)]
fn physics(time: Res<Time>, player: Res<Player>, squad: Res<Squad>, mut blasts: ResMut<Blasts>, mut pickups: ResMut<Pickups>,
           mut last: Local<Vec<Vec3>>, mut bodies: Query<(Entity, &mut Body, &mut Transform, &Visibility)>) {
    let dt = frame_dt(&time);
    if dt <= 0.0 {
        return;
    }
    // the characters' feet and how fast they're going (from last frame's place)
    let feet: Vec<Vec3> = std::iter::once(&*player).chain(squad.0.iter()).map(|u| u.position + Vec3::Y * GROUND).collect();
    let alive: Vec<bool> = std::iter::once(&*player).chain(squad.0.iter()).map(|u| !u.dead).collect();
    let speed: Vec<Vec3> = feet.iter().enumerate()
        .map(|(i, f)| last.get(i).filter(|_| last.len() == feet.len()).map_or(Vec3::ZERO, |l| (*f - *l) / dt)).collect();
    *last = feet.clone();
    let blasts = std::mem::take(&mut blasts.0);
    for (e, mut b, mut tr, vis) in &mut bodies {
        if *vis == Visibility::Hidden {
            continue;
        }
        let at = tr.translation;
        // a fixed per-body jitter for spins
        let j = ((e.index() as f32) * 12.9898).sin().fract().abs();
        b.kicked = (b.kicked - dt).max(0.0);
        for (i, f) in feet.iter().enumerate() {
            let v = Vec3::new(speed[i].x, 0.0, speed[i].z);
            let to = Vec2::new(at.x - f.x, at.z - f.z);
            if !alive[i] || to.length() > KICK_REACH || (at.y - f.y).abs() > KICK_UP_REACH || v.length() < 0.5 || v.length() > 30.0 {
                continue;
            }
            if b.kicked > 0.0 {
                continue;
            }
            // out ahead, and off to whichever side of the character's line it was on
            let ahead = Vec2::new(v.x, v.z).normalize_or_zero();
            let side = Vec2::new(-ahead.y, ahead.x);
            let off = to.dot(side);
            let off = if off.abs() < 0.05 { if j < 0.5 { -0.2 } else { 0.2 } } else { off / KICK_REACH };
            let push = ahead * KICK_AWAY + side * off * KICK_SIDE;
            b.velocity = v * KICK_CARRY + Vec3::new(push.x, 0.0, push.y) + Vec3::Y * KICK_HOP * (0.6 + 0.4 * j);
            b.kicked = KICK_AGAIN;
            b.spin = Vec3::new(3.0 + 4.0 * j, 6.0 * (j - 0.5), 4.0 - 3.0 * j) * (0.5 + v.length() / 8.0);
            b.moving = true;
            if std::env::var("BF_PICKUP_LOG").is_ok() {
                println!("kicked {e} at {at:.2} by character {i} going {:.1} m/s: {:.2}", v.length(), b.velocity);
            }
        }
        for &(c, r) in &blasts {
            let d = at.distance(c);
            if d < r * BLAST_REACH {
                let k = 1.0 - d / (r * BLAST_REACH);
                let away = Vec3::new(at.x - c.x, 0.0, at.z - c.z).normalize_or(Vec3::X);
                b.velocity += (away * 0.8 + Vec3::Y) * BLAST_THROW * k;
                b.spin += Vec3::new(12.0 * j, 8.0, 10.0 * (1.0 - j)) * k;
                b.moving = true;
            }
        }
        if !b.moving {
            continue;
        }
        b.velocity.y -= DROP_GRAVITY * dt;
        let mut pos = at + b.velocity * dt;
        let floor = floor_y(pos.x, pos.z, at.y + 0.3) + b.lift;
        if pos.y < floor {
            pos.y = floor;
            if b.velocity.y < -BOUNCE_SPEED {
                // a landing: bounce, losing some of its way
                b.velocity.y = -b.velocity.y * DROP_BOUNCE;
                b.velocity.x *= DROP_FRICTION;
                b.velocity.z *= DROP_FRICTION;
                b.spin *= DROP_FRICTION;
            } else {
                // on the ground: sliding, slowed by friction, tumbling less
                b.velocity.y = b.velocity.y.max(0.0);
                let across = Vec2::new(b.velocity.x, b.velocity.z);
                let slower = (across.length() - SLIDE_FRICTION * dt).max(0.0);
                let across = across.normalize_or_zero() * slower;
                b.velocity.x = across.x;
                b.velocity.z = across.y;
                b.spin *= (1.0 - 6.0 * dt).max(0.0);
                if slower < SETTLE {
                    // at rest: lying where it landed, tilted to the ground
                    b.velocity = Vec3::ZERO;
                    b.spin = Vec3::ZERO;
                    b.moving = false;
                    let (yaw, _, _) = tr.rotation.to_euler(EulerRot::YXZ);
                    tr.rotation = lie(pos, yaw);
                    if std::env::var("BF_PICKUP_LOG").is_ok() {
                        println!("{e} came to rest at {pos:.2}; the player at {:.2}", feet[0]);
                    }
                }
            }
        }
        tr.translation = pos;
        let spin = b.spin * dt;
        if b.moving && spin.length_squared() > 1e-8 {
            tr.rotation = Quat::from_scaled_axis(spin) * tr.rotation;
        }
    }
    // the moving ones push off the others (not into them)
    let all: Vec<(Entity, Vec3, bool)> = bodies.iter().filter(|(_, _, _, v)| **v != Visibility::Hidden)
        .map(|(e, b, t, _)| (e, t.translation, b.moving)).collect();
    for &(e, at, _) in all.iter().filter(|a| a.2) {
        let mut push = Vec2::ZERO;
        for &(o, other, _) in &all {
            let d = Vec2::new(at.x - other.x, at.z - other.z);
            if o != e && d.length() < BODY_APART && (at.y - other.y).abs() < 0.3 {
                push += d.normalize_or(Vec2::X) * (BODY_APART - d.length());
            }
        }
        if push != Vec2::ZERO {
            if let Ok((_, mut b, mut t, _)) = bodies.get_mut(e) {
                t.translation.x += push.x;
                t.translation.z += push.y;
                let n = push.normalize();
                let into = b.velocity.x * n.x + b.velocity.z * n.y;
                if into < 0.0 {
                    b.velocity.x -= into * n.x;
                    b.velocity.z -= into * n.y;
                }
            }
        }
    }
    // the pickups' spots follow them
    for p in &mut pickups.list {
        if let Ok((_, _, t, _)) = bodies.get(p.entity) {
            p.at = t.translation;
        }
    }
}
