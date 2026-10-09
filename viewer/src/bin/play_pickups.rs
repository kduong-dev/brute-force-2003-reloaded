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
//! after RESPAWN s (the game's respawn time isn't found). Squad members running over one take it
//! too: a medkit into the squad's shared inventory (the player's), a fruit for their own health.

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
/// The used medkit let go: its push (m/s, forward of the character and up), gravity (for every
/// loose body), its size if its model can't be measured (m), how many stay on the ground at once.
const DROP_FORWARD: f32 = 0.8;
const DROP_UP: f32 = 0.6;
const DROP_GRAVITY: f32 = 9.8;
const DROP_RADIUS: f32 = 0.06;
const DROPS_KEPT: usize = 12;
/// Loose pickups: how near (m, across) a character's feet must come to kick one, and how high
/// above or below; the kick (the character's speed carried, a push away, a hop up, m/s); a
/// blast's reach (its radius times this) and throw (m/s at its middle); how far apart (m)
/// loose ones push to; the steepest they lie tilted (rad). A kick sends it out ahead faster than the character and off to the side it was
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
/// The tumble (see `rigid_step`): a corner landing faster than BOUNCE_SPEED (m/s) bounces back
/// at RIGID_BOUNCE of it; friction at a corner is at most RIGID_FRICTION of its bounce; on the
/// ground the turning slows by RIGID_ROLL_DRAG per second; RIGID_STEPS sub-steps per 1/15 s.
/// Slower than RIGID_STILL m/s and RIGID_STILL_SPIN rad/s on the ground for RIGID_QUIET s, it's
/// at rest. A kick rolls it KICK_ROLL rad/s per m/s it's sent. All the demo's choices.
const BOUNCE_SPEED: f32 = 1.0;
const RIGID_BOUNCE: f32 = 0.3;
const RIGID_FRICTION: f32 = 0.6;
const RIGID_ROLL_DRAG: f32 = 1.5;
const RIGID_STEPS: usize = 8;
const RIGID_STILL: f32 = 0.12;
const RIGID_STILL_SPIN: f32 = 0.6;
const RIGID_QUIET: f32 = 0.25;
const KICK_ROLL: f32 = 2.5;
/// A shot through a loose body: the push along the shot and the hop (m/s), and the turn per m of
/// how far off its middle it's hit (rad/s). The demo's choices.
const SHOT_PUSH: f32 = 2.5;
const SHOT_HOP: f32 = 1.0;
const SHOT_SPIN: f32 = 60.0;
/// Balanced, not resting (see `balanced`): the touching points within CONTACT_GAP (m) of the
/// lowest, their narrower spread under TIP_WIDTH (m); the push over (rad/s).
const CONTACT_GAP: f32 = 0.01;
const TIP_WIDTH: f32 = 0.02;
const TOPPLE_PUSH: f32 = 1.5;
/// How fast a body that has stopped turns into its lying pose (per second, exponential: most
/// of the way in about 0.25 s), so it doesn't snap flat on the frame it stops.
const SETTLE_TURN: f32 = 12.0;
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
    /// its bounding box corners (for its tumble when let go)
    corners: Option<Vec<Vec3>>,
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
    /// the pose it's settling into after coming to rest (eased to, see SETTLE_TURN)
    rest: Option<Quat>,
    /// the outermost points of its meshes (in its own frame, see `local_points`): it touches
    /// the ground with these, whichever way up it is; without them, `lift` is how high its
    /// origin sits
    corners: Option<Vec<Vec3>>,
    /// how long it has been all but still on the ground (s): at RIGID_QUIET it comes to rest
    quiet: f32,
}

impl Body {
    /// How high its origin sits above the ground turned `rotation`: its lowest corner's depth.
    fn contact(&self, rotation: Quat) -> f32 {
        match &self.corners {
            Some(c) => -c.iter().map(|v| (rotation * *v).y).fold(f32::MAX, f32::min),
            None => self.lift,
        }
    }
}

/// Something let go to fall and tumble as a loose body from where it is, e.g. a dead
/// character's gun (play.rs): its starting motion. Made a `Body` once its meshes are measured.
#[derive(Component)]
pub struct Thrown {
    pub velocity: Vec3,
    pub spin: Vec3,
}

/// A placed pickup that starts falling where it is, in the pose it's given, rather than lying
/// seated on the ground (the test map drops its pickups so they land on a face).
#[derive(Component)]
pub struct DropIn;

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
        .add_systems(Update, (find_pickups, throw, physics, take_pickups, use_medkit, medkit_used, hold_medkit, face_glows).chain()
            .after(update_player).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// The level's health pickups, once its objects are spawned (their own Transform: placed objects
/// are roots, and on their first frame the GlobalTransform isn't worked out yet), the medkit
/// spots' glow, and the used medkit's model.
#[allow(clippy::too_many_arguments)]
fn find_pickups(mut commands: Commands, mut game: ResMut<GameData>, mut pickups: ResMut<Pickups>, mut player: ResMut<Player>,
                mut used: ResMut<UsedMedkit>, mut placed: Query<(Entity, &Placed, &mut Transform, Has<DropIn>)>,
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
        .filter(|(_, p, _, _)| p.tag == h("inventory-object") && !game.0.idle_effects.contains_key(&p.kind)
            && game.0.items.get(&p.kind).is_some_and(|i| i.function == MEDKIT || i.function == FRUIT))
        .map(|(entity, p, t, _)| Pickup { entity, kind: p.kind, at: t.translation, gone: 0.0, told: false })
        .collect();
    // one glow over each group of medkits (it stays when they're taken: it marks where they are)
    let kits: Vec<usize> = (0..pickups.list.len())
        .filter(|&i| game.0.items.get(&pickups.list[i].kind).is_some_and(|t| t.function == MEDKIT))
        // (dropped in ones fall where they will)
        .filter(|&i| !placed.get(pickups.list[i].entity).is_ok_and(|p| p.3)).collect();
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
        let Ok((_, _, first, _)) = placed.get(pickups.list[kits[list[0]]].entity) else { continue };
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
            if let Ok((_, _, mut t, _)) = placed.get_mut(pickup.entity) {
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
    for (e, p, mut t, drop_in) in &mut placed {
        if p.tag != h("inventory-object") || game.0.idle_effects.contains_key(&p.kind) {
            continue;
        }
        let ground = floor_y(t.translation.x, t.translation.z, t.translation.y + 0.5);
        let mut body = Body { velocity: Vec3::ZERO, spin: Vec3::ZERO, lift: (t.translation.y - ground).max(0.0), moving: drop_in, kicked: 0.0,
                              rest: None, corners: local_points(e, &children, &parts, &meshes, t.scale), quiet: 0.0 };
        if drop_in {
            // falling from where it is, as it's turned
            body.lift = 0.0;
        } else {
            let (yaw, _, _) = t.rotation.to_euler(EulerRot::YXZ);
            t.rotation = lie(t.translation, yaw);
            // sitting on its lowest point, not on its origin (a model's origin can be its middle)
            t.translation.y = ground + body.contact(t.rotation);
        }
        commands.entity(e).insert(body);
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
                    if let Some(b) = assets.meshes.get(&mesh).and_then(|m| m.compute_aabb()) {
                        let (lo, hi) = (Vec3::from(b.min()) + part.offset, Vec3::from(b.max()) + part.offset);
                        let (l, u) = used.corners.as_ref().map_or((lo, hi), |c| c.iter().fold((lo, hi), |(l, u), v| (l.min(*v), u.max(*v))));
                        used.corners = Some(box_corners(l, u).to_vec());
                    }
                    used.parts.push((mesh, red, part.offset));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn take_pickups(time: Res<Time>, game: Res<GameData>, mut pickups: ResMut<Pickups>, mut player: ResMut<Player>,
                mut squad: ResMut<Squad>, mut status: ResMut<UsePanel>, mut feed: ResMut<PickupFeed>, mut vis: Query<&mut Visibility>) {
    let dt = frame_dt(&time);
    for line in &mut feed.0 {
        line.2 -= dt;
    }
    feed.0.retain(|l| l.2 > 0.0);
    player.item_new = (player.item_new - dt).max(0.0);
    // the selected item ran out (the last grenade thrown): another grenade type carried, else
    // the next item carried
    if item_count(&player, player.item) <= 0 {
        item_ran_out(&mut player);
    }
    // who's near enough: the player first, then any squad member running over it (their
    // medkits go in the squad's shared inventory, the player's; a fruit heals whoever eats it)
    let feet: Vec<Option<Vec3>> = std::iter::once(&*player).chain(squad.0.iter())
        .map(|u| (!u.dead).then(|| u.position + Vec3::Y * GROUND)).collect();
    for p in &mut pickups.list {
        if p.gone > 0.0 {
            p.gone -= dt;
            if p.gone <= 0.0 {
                if let Ok(mut v) = vis.get_mut(p.entity) { *v = Visibility::Inherited; }
            }
            continue;
        }
        let near = |f: &Option<Vec3>| f.is_some_and(|f| Vec2::new(p.at.x - f.x, p.at.z - f.z).length() < REACH && (p.at.y - f.y).abs() < REACH_UP);
        let Some(who) = feet.iter().position(near) else {
            p.told = false;
            continue;
        };
        let Some(item) = game.0.items.get(&p.kind) else { continue };
        let taken = match item.function {
            // a fruit, eaten by a squad member: their own health
            FRUIT if who > 0 => {
                let m = &mut squad.0[who - 1];
                let hurt = m.health < m.max_health;
                if hurt {
                    m.health = (m.health + item.health).min(m.max_health);
                }
                hurt
            }
            // a medkit: into the shared inventory, whoever runs over it; full, only the player
            // is told
            MEDKIT if who > 0 && player.medkits >= item.stack_limit => false,
            MEDKIT if player.medkits < item.stack_limit => {
                // a first medkit: the item box shows it, NEW
                if player.medkits == 0 {
                    player.item = Item::Medkit;
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
                let by = if who == 0 { player.character } else { squad.0[who - 1].character };
                println!("{} took {} at {:.1}: health {:.0}/{:.0}, medkits {}", CHARACTERS[by], item.label, p.at, player.health, player.max_health, player.medkits);
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
    let selected = player.item == Item::Medkit;
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
                                     lift: DROP_RADIUS, moving: true, kicked: 0.0, rest: None, corners: used.corners.clone(), quiet: 0.0 });
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

/// Thrown things become loose bodies, measured by their meshes.
fn throw(mut commands: Commands, thrown: Query<(Entity, &Thrown, &Transform)>, children: Query<&Children>,
         parts: Query<(&Transform, Option<&Mesh3d>), Without<Placed>>, meshes: Res<Assets<Mesh>>) {
    for (e, t, tr) in &thrown {
        commands.entity(e).remove::<Thrown>().insert(Body {
            velocity: t.velocity, spin: t.spin, lift: DROP_RADIUS, moving: true, kicked: KICK_AGAIN, rest: None,
            corners: local_points(e, &children, &parts, &meshes, tr.scale), quiet: 0.0,
        });
    }
}

/// A placed object's outermost points in its own frame (with its `scale`): of all its meshes'
/// vertices, the furthest out along each of 26 directions (toward a box's faces, edges and
/// corners), about its convex hull. Unlike its bounding box this has its real shape at the
/// bottom: a fruit's pointed end touches the ground at one point and topples, a crate's flat
/// base stays put.
fn local_points(entity: Entity, children: &Query<&Children>, parts: &Query<(&Transform, Option<&Mesh3d>), Without<Placed>>,
                meshes: &Assets<Mesh>, scale: Vec3) -> Option<Vec<Vec3>> {
    let mut all = vec![];
    // (its children, in its frame: the root's own transform is where it is, not its shape)
    let mut stack: Vec<(Entity, Transform)> = children.get(entity)
        .map(|k| k.iter().map(|c| (c, Transform::from_scale(scale))).collect()).unwrap_or_default();
    while let Some((e, at)) = stack.pop() {
        if let Ok((t, mesh)) = parts.get(e) {
            let at = at * *t;
            if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(v)) =
                mesh.and_then(|m| meshes.get(&m.0)).and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)) {
                all.extend(v.iter().map(|p| at.transform_point(Vec3::from(*p))));
            }
            if let Ok(kids) = children.get(e) {
                stack.extend(kids.iter().map(|k| (k, at)));
            }
        } else if let Ok(kids) = children.get(e) {
            stack.extend(kids.iter().map(|k| (k, at)));
        }
    }
    if all.is_empty() {
        return None;
    }
    let mut points: Vec<Vec3> = vec![];
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let d = Vec3::new(x as f32, y as f32, z as f32);
                if d == Vec3::ZERO {
                    continue;
                }
                let p = all.iter().copied().max_by(|a, b| a.dot(d).total_cmp(&b.dot(d))).unwrap_or_default();
                if !points.iter().any(|q| q.distance(p) < 1e-4) {
                    points.push(p);
                }
            }
        }
    }
    Some(points)
}

/// The eight corners of the box `lo`..`hi`.
fn box_corners(lo: Vec3, hi: Vec3) -> [Vec3; 8] {
    [lo, hi, Vec3::new(lo.x, lo.y, hi.z), Vec3::new(hi.x, hi.y, lo.z),
     Vec3::new(lo.x, hi.y, lo.z), Vec3::new(hi.x, lo.y, hi.z), Vec3::new(lo.x, hi.y, hi.z), Vec3::new(hi.x, lo.y, lo.z)]
}

/// A placed object's footprint (m, along its own x and z), from its meshes' bounds.
fn footprint(entity: Entity, children: &Query<&Children>, parts: &Query<(&Transform, Option<&Mesh3d>), Without<Placed>>,
             meshes: &Assets<Mesh>, scale: Vec3) -> Option<Vec2> {
    let c = local_points(entity, children, parts, meshes, scale)?;
    let (lo, hi) = c.iter().fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(*v), u.max(*v)));
    Some(Vec2::new(hi.x - lo.x, hi.z - lo.z))
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

/// The ground's up under `at`: its slope, or straight up where it's steeper than MAX_TILT (a
/// step or a wall edge, not a slope).
fn ground_up(at: Vec3) -> Vec3 {
    let e = 0.15;
    let top = at.y + 0.5;
    let dx = floor_y(at.x + e, at.z, top) - floor_y(at.x - e, at.z, top);
    let dz = floor_y(at.x, at.z + e, top) - floor_y(at.x, at.z - e, top);
    let n = Vec3::new(-dx / (2.0 * e), 1.0, -dz / (2.0 * e)).normalize();
    if n.angle_between(Vec3::Y) > MAX_TILT { Vec3::Y } else { n }
}

/// The rest pose for something placed at `at` turned `yaw`: upright, tilted to the ground
/// under it.
fn lie(at: Vec3, yaw: f32) -> Quat {
    Quat::from_rotation_arc(Vec3::Y, ground_up(at)) * Quat::from_rotation_y(yaw)
}

/// The rest pose for something that has tumbled to a stop at `at` turned `now`: on whichever
/// face is nearest the ground (its local axis pointing most nearly up), turned the least way
/// that sits that face on the ground. Its heading and the side it landed on are kept, so it
/// settles by a few degrees instead of being stood back upright.
fn settle(at: Vec3, now: Quat) -> Quat {
    let up = [Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z].into_iter()
        .map(|a| now * a).max_by(|a, b| a.y.total_cmp(&b.y)).unwrap_or(Vec3::Y);
    (Quat::from_rotation_arc(up, ground_up(at)) * now).normalize()
}

/// Whether a still body is only balanced, not resting: the points it touches the ground with (its
/// outer points within CONTACT_GAP of the lowest) are one point or one line (their narrower
/// spread under TIP_WIDTH m), like the Garo fruit on its point or a card on its edge. Then
/// the way it leans (across, from those points toward its middle), to fall over that way.
fn balanced(b: &Body, tr: &Transform) -> Option<Vec3> {
    let points = b.corners.as_ref()?;
    let (lo, hi) = points.iter().fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(*v), u.max(*v)));
    let middle = tr.transform_point((lo + hi) * 0.5);
    let world: Vec<Vec3> = points.iter().map(|p| tr.transform_point(*p)).collect();
    let height = |p: &Vec3| p.y - floor_y(p.x, p.z, p.y + 0.5);
    let low = world.iter().map(height).fold(f32::MAX, f32::min);
    let touching: Vec<Vec2> = world.iter().filter(|p| height(p) < low + CONTACT_GAP).map(|p| p.xz()).collect();
    let n = touching.len() as f32;
    let centre = touching.iter().sum::<Vec2>() / n;
    // the spread of the touching points: their covariance's narrower axis
    let (mut xx, mut xz, mut zz) = (0.0, 0.0, 0.0);
    for p in &touching {
        let d = *p - centre;
        xx += d.x * d.x / n;
        xz += d.x * d.y / n;
        zz += d.y * d.y / n;
    }
    let (sum, diff) = (xx + zz, ((xx - zz) * (xx - zz) * 0.25 + xz * xz).sqrt());
    let narrow = (sum * 0.5 - diff).max(0.0).sqrt();
    if narrow >= TIP_WIDTH {
        return None;
    }
    // the narrow axis (across the edge), and which way along it the middle is
    let across = if xz.abs() > 1e-6 { Vec2::new(sum * 0.5 - diff - zz, xz).normalize_or(Vec2::X) }
                 else if xx < zz { Vec2::X } else { Vec2::Y };
    let lean = middle.xz() - centre;
    let way = if lean.length() > 1e-3 { lean.normalize() } else { across * if across.dot(lean) < 0.0 { -1.0 } else { 1.0 } };
    Some(Vec3::new(way.x, 0.0, way.y))
}

/// One frame of a loose body as a rigid box (its meshes' bounding box, mass 1): gravity, then
/// each corner that has gone into the ground gets an impulse at that corner, a bounce along the
/// ground's up (only from a real landing, RIGID_BOUNCE) and friction across it (up to
/// RIGID_FRICTION of the bounce). Because the push lands on a corner, not the middle, it turns
/// the box too: a box sliding on one edge tips over and rolls, one landing on a corner flips. In
/// RIGID_STEPS sub-steps a frame, so it holds together at 15 fps. Returns whether it's touching
/// the ground.
fn rigid_step(b: &mut Body, tr: &mut Transform, dt: f32) -> bool {
    let corners = b.corners.clone().unwrap_or_else(|| {
        let r = b.lift.max(0.05);
        box_corners(Vec3::splat(-r), Vec3::splat(r)).to_vec()
    });
    let (lo, hi) = corners.iter().fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(*v), u.max(*v)));
    let centre = (lo + hi) * 0.5;
    let half = ((hi - lo) * 0.5).max(Vec3::splat(0.02));
    // a box's inverse inertia (mass 1) along its own axes
    let inv_inertia = Vec3::new(3.0 / (half.y * half.y + half.z * half.z), 3.0 / (half.x * half.x + half.z * half.z),
                                3.0 / (half.x * half.x + half.y * half.y));
    let mut rot = tr.rotation;
    let mut com = tr.translation + rot * centre;
    let up = ground_up(com);
    let steps = ((dt * RIGID_STEPS as f32 * 15.0).ceil() as usize).clamp(1, 4 * RIGID_STEPS);
    let h = dt / steps as f32;
    let mut touching = false;
    for _ in 0..steps {
        b.velocity.y -= DROP_GRAVITY * h;
        com += b.velocity * h;
        if b.spin.length_squared() > 1e-10 {
            rot = (Quat::from_scaled_axis(b.spin * h) * rot).normalize();
        }
        let inv_i = |x: Vec3| rot * (inv_inertia * (rot.inverse() * x));
        let mut deepest = 0.0f32;
        for c in &corners {
            let r = rot * (*c - centre);
            let p = com + r;
            let depth = floor_y(p.x, p.z, p.y + 0.5) - p.y;
            if depth <= 0.0 {
                continue;
            }
            touching = true;
            deepest = deepest.max(depth);
            let vp = b.velocity + b.spin.cross(r);
            let vn = vp.dot(up);
            if vn >= 0.0 {
                continue;
            }
            // the bounce at this corner
            let k = 1.0 + inv_i(r.cross(up)).cross(r).dot(up);
            let bounce = if vn < -BOUNCE_SPEED { RIGID_BOUNCE } else { 0.0 };
            let jn = -(1.0 + bounce) * vn / k;
            b.velocity += up * jn;
            b.spin += inv_i(r.cross(up * jn));
            // friction across the ground at this corner
            let vp = b.velocity + b.spin.cross(r);
            let across = vp - up * vp.dot(up);
            let speed = across.length();
            if speed > 1e-4 {
                let t = across / speed;
                let kt = 1.0 + inv_i(r.cross(t)).cross(r).dot(t);
                let jt = (speed / kt).min(RIGID_FRICTION * jn);
                b.velocity -= t * jt;
                b.spin -= inv_i(r.cross(t * jt));
            }
        }
        // out of the ground
        com += up * deepest;
        if deepest > 0.0 {
            b.spin *= (1.0 - RIGID_ROLL_DRAG * h).max(0.0);
        }
    }
    tr.rotation = rot;
    tr.translation = com - rot * centre;
    touching
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
    // this frame's shots (anyone's): each knocks the first loose body on its line
    let shots: Vec<(Vec3, Vec3, f32)> = std::iter::once(&*player).chain(squad.0.iter())
        .flat_map(|u| u.shots.iter().map(|s| (s.origin, s.dir, s.dist))).collect();
    let mut struck: HashMap<Entity, (Vec3, Vec3)> = HashMap::new();
    for &(origin, dir, dist) in &shots {
        let hit = bodies.iter().filter(|(_, _, _, v)| **v != Visibility::Hidden).filter_map(|(e, b, t, _)| {
            let (lo, hi) = b.corners.as_ref()?.iter().fold((Vec3::MAX, Vec3::MIN), |(l, u), v| (l.min(*v), u.max(*v)));
            let (centre, radius) = (t.transform_point((lo + hi) * 0.5), ((hi - lo) * 0.5).length().max(0.05));
            // where the line passes closest to its middle, if within its size and the shot's reach
            let along = (centre - origin).dot(dir);
            let miss = (origin + dir * along).distance(centre);
            (along > 0.0 && along < dist + radius && miss < radius).then(|| (along, e, origin + dir * (along - (radius * radius - miss * miss).sqrt()), centre))
        }).min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, e, at, centre)) = hit {
            if std::env::var("BF_PICKUP_LOG").is_ok() {
                println!("shot struck {e} at {at:.2}");
            }
            struck.insert(e, (dir, at - centre));
        }
    }
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
            // rolling the way it's sent (a ball rolling along v turns about up x v), a little
            // askew and turning, per body
            let along = Vec3::new(b.velocity.x, 0.0, b.velocity.z);
            b.spin = Vec3::Y.cross(along) * KICK_ROLL + Vec3::new(2.0 * (j - 0.5), 4.0 * (j - 0.5), 2.0 * (0.5 - j));
            b.moving = true;
            b.rest = None;
            b.quiet = 0.0;
            if std::env::var("BF_PICKUP_LOG").is_ok() {
                println!("kicked {e} at {at:.2} by character {i} going {:.1} m/s: {:.2}", v.length(), b.velocity);
            }
        }
        // shot: knocked along the shot, and turned by where it was hit
        if let Some(&(dir, off)) = struck.get(&e) {
            b.velocity += dir * SHOT_PUSH + Vec3::Y * SHOT_HOP;
            b.spin += off.cross(dir) * SHOT_SPIN;
            b.moving = true;
            b.rest = None;
            b.quiet = 0.0;
        }
        for &(c, r) in &blasts {
            let d = at.distance(c);
            if d < r * BLAST_REACH {
                let k = 1.0 - d / (r * BLAST_REACH);
                let away = Vec3::new(at.x - c.x, 0.0, at.z - c.z).normalize_or(Vec3::X);
                b.velocity += (away * 0.8 + Vec3::Y) * BLAST_THROW * k;
                b.spin += Vec3::Y.cross(away) * BLAST_THROW * KICK_ROLL * k + Vec3::new(6.0 * j, 4.0, 5.0 * (1.0 - j)) * k;
                    b.moving = true;
                b.rest = None;
                b.quiet = 0.0;
            }
        }
        if !b.moving {
            // coming to rest: turning into its lying pose, not snapping to it
            if let Some(rest) = b.rest {
                tr.rotation = tr.rotation.slerp(rest, (1.0 - (-SETTLE_TURN * dt).exp()).min(1.0));
                // (still on the ground as it turns)
                tr.translation.y = floor_y(at.x, at.z, at.y + 0.3) + b.contact(tr.rotation);
                if tr.rotation.angle_between(rest) < 0.01 {
                    tr.rotation = rest;
                    b.rest = None;
                }
            }
            continue;
        }
        let touching = rigid_step(&mut b, &mut tr, dt);
        b.quiet = if touching && b.velocity.length() < RIGID_STILL && b.spin.length() < RIGID_STILL_SPIN { b.quiet + dt } else { 0.0 };
        // balanced on a point or an edge isn't resting: it falls the way it leans
        if b.quiet >= RIGID_QUIET {
            if let Some(lean) = balanced(&b, &tr) {
                b.spin += Vec3::Y.cross(lean) * TOPPLE_PUSH;
                b.quiet = 0.0;
            }
        }
        if b.quiet >= RIGID_QUIET {
            // at rest: lying on the face it came to, sat flush on the ground
            b.velocity = Vec3::ZERO;
            b.spin = Vec3::ZERO;
            b.moving = false;
            b.quiet = 0.0;
            b.rest = Some(settle(tr.translation, tr.rotation));
            if std::env::var("BF_PICKUP_LOG").is_ok() {
                println!("{e} came to rest at {:.2} at t {:.2}; the player at {:.2}", tr.translation, player.sim_time, feet[0]);
            }
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
