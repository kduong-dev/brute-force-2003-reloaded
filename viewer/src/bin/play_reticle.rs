//! The crosshair's colour (#141): red while the reticle holds a lock on a hostile, else by what is
//! under the crosshair's centre: yellow on anything whose radar-contact is 2 or 4, green on a
//! living character of the player's own team, blue otherwise. Read from default.xbe (every
//! function cited is in docs/code-map.md; todo/141-crosshair-colours/spec.md) and checked against
//! the takes in todo/141-crosshair-colours/.
//!
//! Each frame, as the game's target selection (its selector B, vtable 0x39f754, run over the
//! world's living targets by 0xbd110) and the reticle controller (0xc5720) do it:
//!
//!  1. **The lock** (`select`). Every living character but the player is a candidate
//!     (FUN_000bf7a0) when:
//!     - it's hostile: the team table's row of the player's team, the column of theirs
//!       (FUN_000bf6d0 on 0x3ffd80, `damage::hostile`). The squad (team 0) locks teams 1, 7 and
//!       8 only: a neutral (team 2) or a teammate never;
//!     - its aim point is within reach of the gun's muzzle: game-options' 0.7 (`ReticuleOptions::
//!       lock_share`) x the gun's reach (FUN_00232520, `reach`). The aim point is the point of
//!       its aim capsule (`AimCapsule`, the character's vtable +0x134) nearest the crosshair's
//!       ray (FUN_0015b450). Only the gun in hand's reach counts: the demo's choice.
//!       FUN_000bf7a0 takes the smaller of it and that of the item in the holder's +0x178 slot
//!       (+0x198; FUN_0014d310, Tex's second gun in his berserk), which the demo doesn't have;
//!     - projected on the HUD's 640 x 480 (in front of the camera, at most DEPTH_MAX deep) the aim
//!       point is inside a box round the crosshair's centre (FUN_000beb40): 33.6 x s units each
//!       way, s the gun's `WeaponDef::lock_box`, x KEEP_BOX for last frame's lock (0xbfae9), so a
//!       lock is kept further out than it's taken;
//!     - a ray from the camera to it (SIGHT_RANGE m) meets that character first, or nothing
//!       (FUN_000becf0).
//!
//!     Of the candidates, each has to beat the best "outer" score so far (`box_score`, over a box
//!     OUTER_BOX times as big) and then the best score so far (FUN_000bf7a0); of those the
//!     lowest cost, depth x (0.3 + 0.8 (1 - score)), under COST_MAX takes the lock
//!     (FUN_000bfe40): the most central, the nearer at a tie. They're tried in the squad's order:
//!     the demo's choice. The game walks the world's target list (0xba1c0), whose order isn't
//!     traced, and with the running best scores that order decides between close candidates.
//!  2. **The centre ray** (0xc5720), with no lock: the first body or object on the ray through
//!     the crosshair's centre, within the gun's range (256 m with no gun in hand), decides:
//!     - a hostile (FUN_000bf550): locked at once (red) with no gun in hand, in the scope, or when
//!       the gun's lock reach outruns its range (FUN_00232590 > FUN_002324c0: a mesh
//!       projectile's, the Therm Sweeper's 133 m against its 20 m); otherwise blue, whatever its
//!       radar-contact (the radar test is skipped, 0xc5943 -> 0xc5a3e);
//!     - a living character of the player's team, a team not hostile to itself (FUN_0004b960):
//!       green;
//!     - then anything else whose radar-contact is 2 or 4 (0xc59d7; `Player::radar_contact`,
//!       `Placement::radar_contact`): a teammate, a neutral, a placed object: yellow, over green;
//!     - anything else, or nothing: blue.
//!  3. **The tint** (0xc6380, FUN_000bbd70): red with a lock, else as the centre ray said; a
//!     step on the frame it changes (the takes: no fade).
//!
//! Not in: the line-of-fire ray (0xc54d0: an obstacle between the gun and the locked target shows
//! its own colour after two frames; not filmed), Hawk's stealth (a target below 0.1 visibility
//! isn't a candidate: the demo's skills have no visibility yet), Flint's x1.25 range with a rifle
//! (FUN_002324c0 through his skill's +0x54: the demo's shots don't have it either), the alpha's
//! blink (controller +0x94 bit 1: what sets it isn't traced) and Flint's targeting box (#124).
//! The demo's bodies are solid cylinders (`ray_body`), so the centre ray doesn't slip between an
//! arm and the body as the game's does (take04's 2-3 frame blue gaps), and the dead aren't hit
//! by it (the game's ray meets a body and tests its radar-contact). Two choices of the demo's
//! (step 1): only the gun in hand's reach counts, and the candidates are tried in the squad's
//! order.
//!
//! Test hooks:
//!  - BF_RETICLE_LOG=1 prints the tint, the lock and what the centre ray meets when they change
//!    (=2 every frame), with each hostile's aim point on the HUD and the box it's tested against.
//!  - BF_TEST_SWEEP=<deg>|<from>,<to>,<deg/s>[;...] turns the camera, to sweep the crosshair
//!    across a target (`sweep`).
//!  - BF_TEST_RADAR=<object>,<value>[;...] sets a level object's radar-contact by its name
//!    (h_xxxxxxxx) as the map starts (`test_radar`). Test map NPCs: play_testtools.rs's
//!    BF_TEST_NPC_RADAR.

use super::*;
use bf_viewer::bf::character::ReticuleOptions;
use bf_viewer::bf::hash::h;

/// The crosshair's alpha: 229 / 255, FUN_000c6280's 0.9 (0xc6290) x 255 (0x3a50e8), truncated
/// into the sub-widget's +0x10 (0xc6373); read live at every draw in #141, 10 HP too.
const ALPHA: f32 = 229.0 / 255.0;
/// FUN_000bbd70's tints (jump table 0xbbe34; bytes R G B, alpha 255): style 0 blue 50 150 255,
/// 1 red 160 32 32, 5 green 68 220 68, 6 yellow 200 200 0. The weapon's crosshair texture is
/// drawn in them at ALPHA, and the console blends in the stored sRGB values. Bevy's UI blends in
/// linear light, where the same colour at that alpha comes out brighter (the blue's B 243
/// against 232 over the takes' dark ground), so each is drawn as the colour that gives the
/// game's result over take08's ground (14, 16, 26), as `METER_ORANGE` is: (49, 143, 243),
/// (152, 32, 32), (66, 209, 67), (190, 190, 0) (spec 7.1).
pub const BLUE: Color = Color::srgba(49.0 / 255.0, 143.0 / 255.0, 243.0 / 255.0, ALPHA);
/// See BLUE: the game's 160 32 32.
pub const RED: Color = Color::srgba(152.0 / 255.0, 32.0 / 255.0, 32.0 / 255.0, ALPHA);
/// See BLUE: the game's 68 220 68.
pub const GREEN: Color = Color::srgba(66.0 / 255.0, 209.0 / 255.0, 67.0 / 255.0, ALPHA);
/// See BLUE: the game's 200 200 0.
pub const YELLOW: Color = Color::srgba(190.0 / 255.0, 190.0 / 255.0, 0.0, ALPHA);

/// The HUD's screen in units (FUN_000beb40 sizes the box by the view's, the owner's +0x20 /
/// +0x22: 640 x 480 full screen; FUN_000ab390 puts the crosshair's centre at half its width).
const SCREEN: Vec2 = Vec2::new(640.0, 480.0);
/// 0xc50b0: the box's width share is game-options' x 0.75 (0x3a510c, the 4:3 aspect), so the box
/// is square on the screen.
const BOX_ASPECT: f32 = 0.75;
/// FUN_000beb40's outer box: 2.5 x the box (0x3c0554).
const OUTER_BOX: f32 = 2.5;
/// 0xbfae9: the box is 1.2 x (0x3f99999a) for the character locked last frame, 1.0 for another.
const KEEP_BOX: f32 = 1.2;
/// FUN_000bfe40's cost of a candidate, depth x (COST_BASE + COST_OFF x (1 - score)): its share of
/// the depth however central it is, 0.3 (0x3a50b0, at 0xbfe57).
const COST_BASE: f32 = 0.3;
/// FUN_000bfe40: the share of the depth a candidate at the box's edge adds over one at its
/// centre, 0.8 (0x3a5014, at 0xbfe4e).
const COST_OFF: f32 = 0.8;
/// The highest cost that takes the lock (m): selector B's +0x48 (0x439a8000 = 309, set at
/// 0xbcbeb), which FUN_000beae0 starts each frame's best at.
const COST_MAX: f32 = 309.0;
/// FUN_000bf7a0: an aim point deeper than this (m, 0x3a503c) isn't a candidate.
const DEPTH_MAX: f32 = 206.0;
/// The lock's sight ray (m): FUN_000bf7a0 hands FUN_000becf0 256.0 (0x43800000).
const SIGHT_RANGE: f32 = 256.0;
/// The centre ray's length with no gun in hand (m): 0xc57d9 (256.0, 0x43800000).
const NO_GUN_RANGE: f32 = 256.0;
/// FUN_00232520: a burst of pellets (no delay between its shots, more than one: the Bower 20's
/// six) reaches 0.9 of its range.
const PELLET_REACH: f32 = 0.9;
/// FUN_00232520: a mesh projectile that splits at its apogee (WM_MIRV_ON_APOGEE) adds 0.75 of
/// its range.
const APOGEE_REACH: f32 = 0.75;
/// WM_MIRV_ON_APOGEE (the WM_ name table 0x3be7f4, the bullet's `<mirv>` mode): FUN_00232520
/// adds APOGEE_REACH of the range for it.
const MIRV_ON_APOGEE: i64 = 1;
/// WM_MIRV_ON_EXPLODE (0x3be7f4): FUN_00232520 adds the whole range again for it (the Therm
/// Sweeper's).
const MIRV_ON_EXPLODE: i64 = 2;
/// The bullet-type whose reach FUN_00232520 lengthens: BTYPE_MESH_PROJECTILE (0x3bd63c).
const MESH_PROJECTILE: i64 = 2;
/// 0x121d00, a character's aim capsule (its vtable +0x134): its top end is this share of the way
/// from its "Bip01 Head" node to its root node (0x3ecccccd, 0x121d73).
const CAPSULE_TOP: f32 = 0.4;
/// 0x121d00: the capsule's bottom end is this share of the head-to-root length past the root,
/// below the hips (0x3f000000, 0x121dc8).
const CAPSULE_BELOW: f32 = 0.5;
/// 0x121d00: the capsule's radius, as a share of the head-to-root length (0x3a51e4, 0x121e91).
const CAPSULE_THICK: f32 = 0.15;
/// The radar-contacts that turn the crosshair yellow (0xc59da, 0xc59df): RCONTACT_OBJECTIVE
/// and RCONTACT_OBJECTIVE_SECONDARY (0x3beaac).
const YELLOW_CONTACTS: [u8; 2] = [2, 4];

/// The crosshair's tint this frame: the reticle sub-widget's style (+0x30), set by 0xc6380.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Tint {
    /// style 0: no lock, nothing of note under the centre
    #[default]
    Blue,
    /// style 1: a lock
    Red,
    /// style 5: the reticle widget's state 1, a teammate under the centre (0xc59c4)
    Green,
    /// style 6: its state 2, a radar-contact 2 or 4 under the centre (0xc59e4)
    Yellow,
}

impl Tint {
    /// The colour the HUD draws it in (BLUE, RED, GREEN, YELLOW).
    pub fn color(self) -> Color {
        match self {
            Tint::Blue => BLUE,
            Tint::Red => RED,
            Tint::Green => GREEN,
            Tint::Yellow => YELLOW,
        }
    }
}

/// The reticle this frame: its tint and lock (the controller's +0x9c, copied from the
/// selection's by 0xc52c0 or taken under the centre ray), and the selection's lock of last
/// frame (FUN_000beae0 keeps it at the selector's +8), whose box is KEEP_BOX bigger.
#[derive(Resource, Default)]
pub struct Reticle {
    pub tint: Tint,
    pub lock: Option<Who>,
    kept: Option<Who>,
    /// the tint, lock and what's under the centre that BF_RETICLE_LOG printed last
    logged: String,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Reticle>()
        .add_systems(OnEnter(AppState::Playing), ((|mut commands: Commands| commands.insert_resource(Reticle::default())), test_radar).after(setup))
        .add_systems(Update, sweep.after(read_input).before(squad_control).run_if(in_state(AppState::Playing)))
        // (once the characters have moved and the camera is where it's drawn; before the HUD)
        .add_systems(Update, update_reticle.after(follow_camera).before(update_weapons).run_if(in_state(AppState::Playing)));
}

/// The camera as drawn: where it is, its turn, the tangent of half its vertical field of view
/// and its near plane. The HUD's 640 x 480 is 4:3 centred in the window at its height, so a unit
/// is the same across and down: 240 units are that tangent at unit depth.
#[derive(Clone, Copy)]
struct View {
    at: Vec3,
    turn: Quat,
    tan: f32,
    near: f32,
}

impl View {
    /// The world direction through a HUD point, with unit depth along the view (FUN_0006dfc0's
    /// ray, divided by the near plane as FUN_000bf7a0 does).
    fn through(&self, hud: Vec2) -> Vec3 {
        self.turn * Vec3::new((hud.x - 320.0) / 240.0 * self.tan, (240.0 - hud.y) / 240.0 * self.tan, -1.0)
    }

    /// How deep a world point is along the view (m).
    fn depth(&self, p: Vec3) -> f32 {
        -(self.turn.inverse() * (p - self.at)).z
    }

    /// A world point on the HUD and its depth, if it's in front of the near plane (FUN_0006e270).
    fn project(&self, p: Vec3) -> Option<(Vec2, f32)> {
        let v = self.turn.inverse() * (p - self.at);
        let depth = -v.z;
        (depth > self.near).then(|| (Vec2::new(320.0 + v.x / depth / self.tan * 240.0, 240.0 - v.y / depth / self.tan * 240.0), depth))
    }
}

/// A character's aim capsule (0x121d00, the character's vtable +0x134): a segment from `top`
/// to `bottom`, `radius` thick. FUN_000bf7a0 takes its point nearest the crosshair's ray as the
/// character's aim point.
struct AimCapsule {
    top: Vec3,
    bottom: Vec3,
    radius: f32,
}

impl AimCapsule {
    /// `u`'s: with d = its root node (`missile::root_point`, the hips) less its "Bip01 Head"
    /// node, from the head + CAPSULE_TOP x d to the root + CAPSULE_BELOW x d, CAPSULE_THICK x |d|
    /// thick; with no head node a point at the root (0x121ea6).
    fn of(u: &Player) -> AimCapsule {
        let root = missile::root_point(u);
        let Some(head) = head_point(u) else { return AimCapsule { top: root, bottom: root, radius: 0.0 } };
        let d = root - head;
        AimCapsule { top: head + d * CAPSULE_TOP, bottom: root + d * CAPSULE_BELOW, radius: d.length() * CAPSULE_THICK }
    }

    fn middle(&self) -> Vec3 {
        (self.top + self.bottom) * 0.5
    }

    /// FUN_0015b450: the point of the capsule nearest `p`: the segment's nearest point, moved the
    /// radius toward `p`.
    fn nearest(&self, p: Vec3) -> Vec3 {
        let ab = self.bottom - self.top;
        let (t, len2) = (ab.dot(p - self.top), ab.length_squared());
        let on = if t <= 0.0 { self.top } else if t >= len2 { self.bottom } else { self.top + ab * (t / len2) };
        let to = p - on;
        let d = to.length();
        if d > 1e-6 { on + to * (self.radius / d) } else { on }
    }
}

/// Where `u`'s "Bip01 Head" node is (char+0x3e8, found at 0x12f95d): its bone in the pose as
/// drawn (play_damage.rs's `model_root` x `last_world`), or in its ragdoll while it lies limp.
fn head_point(u: &Player) -> Option<Vec3> {
    let l = u.loaded.as_ref()?;
    let i = l.model.bones.iter().position(|&b| b == h("Bip01 Head"))?;
    if let Some(r) = u.knock.as_ref().map(|k| &k.ragdoll).or(u.ragdoll.as_ref()) {
        return r.pos.get(i).map(|&p| r.origin + Quat::from_rotation_y(r.yaw) * p);
    }
    let root = Mat4::from_rotation_translation(Quat::from_rotation_y(u.yaw), u.position + Vec3::Y * (u.height + u.lift_now));
    Some(root.transform_point3(u.last_world.get(i)?.w_axis.truncate()))
}

/// The held gun's muzzle (FUN_00232180): its hardpoint on the hand as posed, as play.rs's shots
/// leave from it; None while it isn't in the hand.
fn muzzle(p: &Player, w: &HeldWeapon) -> Option<Vec3> {
    let (hb, r, t) = w.in_hand?;
    let root = Mat4::from_rotation_translation(Quat::from_rotation_y(p.yaw), p.position + Vec3::Y * p.height);
    Some((root * *p.last_world.get(hb)?).transform_point3(r * w.muzzle.point + t))
}

/// FUN_00232520: how far a gun reaches for the lock, before game-options' share. Its range
/// (FUN_002324c0: the bullet's `range` through the holder's skill, which the demo's leave as it
/// is), x PELLET_REACH for a burst of pellets; a mesh projectile's adds its second timer x its
/// speed, and its range again (MIRV_ON_EXPLODE) or APOGEE_REACH of it (MIRV_ON_APOGEE). The
/// Therm Sweeper: 20 + 20 + 5 x 30 = 190 m (read live in #141), its lock 133 m.
fn reach(def: &WeaponDef) -> f32 {
    let range = def.range;
    if def.shots.bullet_type != MESH_PROJECTILE {
        return if def.shots.burst_delay == 0.0 && def.shots.burst_count > 1 { range * PELLET_REACH } else { range };
    }
    let flight = def.bullet_after * def.bullet_speed;
    match def.mirv_mode {
        MIRV_ON_EXPLODE => range + range + flight,
        MIRV_ON_APOGEE => range * APOGEE_REACH + range + flight,
        _ => range + flight,
    }
}

/// FUN_000beb40: how central the HUD point `at` is in the box of half-size `half` round the
/// crosshair's centre: (score, outer). Each is a smoothstep, (3 - 2w) w², of w = the square root
/// of (1 - dx / x)(1 - dy / y): the score over the box itself (0 outside it), the outer one over
/// a box OUTER_BOX times as big (0 outside that).
fn box_score(at: Vec2, centre: Vec2, half: Vec2) -> (f32, f32) {
    let d = (at - centre).abs();
    let outer = half * OUTER_BOX;
    if half.x <= 0.0 || half.y <= 0.0 || d.x >= outer.x || d.y >= outer.y {
        return (0.0, 0.0);
    }
    let smooth = |w: f32| (3.0 - 2.0 * w) * w * w;
    let wide = smooth(((1.0 - d.x / outer.x) * (1.0 - d.y / outer.y)).sqrt());
    let score = if d.x < half.x && d.y < half.y { smooth(((1.0 - d.x / half.x) * (1.0 - d.y / half.y)).sqrt()) } else { 0.0 };
    (score, wide)
}

/// FUN_000becf0 from FUN_000bf7a0: whether the ray from the camera through `point`
/// (SIGHT_RANGE m) meets `target`'s body before anything else, or meets nothing. The world or
/// another body first hides it.
fn in_sight(from: Vec3, point: Vec3, target: Who, squad: &Squad) -> bool {
    let dir = (point - from).normalize_or_zero();
    let wall = ray_hit(from, dir, SIGHT_RANGE);
    let body = squad.0.iter().filter(|u| !u.dead).filter_map(|u| ray_body(from, dir, u, SIGHT_RANGE).map(|t| (t, u.who())))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match (body, wall) {
        (Some((t, who)), Some(w)) => t <= w && who == target,
        (Some((_, who)), None) => who == target,
        (None, wall) => wall.is_none(),
    }
}

/// What the crosshair's centre ray meets first (0xc5720's FUN_000becf0): a living character (a
/// `Squad` index) or a level's placed object (its `Level::objects` index; None for the terrain,
/// the flat floor and anything that isn't one).
#[derive(Clone, Copy, Debug)]
enum Under {
    Body(usize),
    Object(Option<usize>),
}

/// The first body or object on the ray from `from` along `dir`, within `range` (m), and how far.
fn under_ray(from: Vec3, dir: Vec3, range: f32, squad: &Squad) -> Option<(f32, Under)> {
    let world = match world::arena() {
        Some(a) => a.ray_object(from, dir, range).map(|(t, o)| (t, Under::Object(o))),
        None => ray_hit(from, dir, range).map(|t| (t, Under::Object(None))),
    };
    let body = squad.0.iter().enumerate().filter(|(_, u)| !u.dead)
        .filter_map(|(i, u)| ray_body(from, dir, u, range).map(|t| (t, Under::Body(i))))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match (body, world) {
        (Some(b), Some(w)) => Some(if b.0 <= w.0 { b } else { w }),
        (b, w) => b.or(w),
    }
}

/// A character's name for the log.
fn name(u: &Player) -> String {
    format!("{}{}", unit_name(u), u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)))
}

/// The selection's lock this frame (see the module's notes, step 1); with `notes` (BF_RETICLE_LOG)
/// a note on each hostile for the log, made only then.
#[allow(clippy::too_many_arguments)]
fn select(player: &Player, squad: &Squad, view: &View, centre: Vec2, gun: Option<(&HeldWeapon, Vec3)>, options: &ReticuleOptions,
          kept: Option<Who>, mut notes: Option<&mut Vec<String>>) -> Option<Who> {
    // (FUN_000bf7a0 needs a gun in hand)
    let (w, muzzle) = gun?;
    let reach = options.lock_share * reach(&w.def);
    let half = Vec2::new(SCREEN.x * options.box_share * BOX_ASPECT, SCREEN.y * options.box_share) * 0.5 * w.def.lock_box;
    let (mut best_wide, mut best_score, mut best_cost) = (0.0, 0.0, COST_MAX);
    let mut lock = None;
    let along = view.through(centre);
    for u in squad.0.iter().filter(|u| !u.dead && damage::hostile(player.team, u.team)) {
        let capsule = AimCapsule::of(u);
        // the crosshair's ray at the depth of the capsule's middle (FUN_000a0c60)
        let aim = capsule.nearest(view.at + along * view.depth(capsule.middle()));
        let f = if kept == Some(u.who()) { KEEP_BOX } else { 1.0 };
        // (the log's note, its text made only when the log is on)
        let mut note = |why: &dyn Fn() -> String| if let Some(n) = notes.as_deref_mut() { n.push(format!("{} {}", name(u), why())) };
        if muzzle.distance(aim) > reach {
            note(&|| format!("out of reach: {:.1} m from the muzzle, reach {reach:.1}", muzzle.distance(aim)));
            continue;
        }
        let Some((at, depth)) = view.project(aim).filter(|&(_, d)| d <= DEPTH_MAX) else {
            note(&|| "behind the camera or too deep".into());
            continue;
        };
        let (score, wide) = box_score(at, centre, half * f);
        let place = || {
            let off = at - centre;
            format!("at ({:+.1}, {:+.1}) units, box {:.1}{}", off.x, off.y, half.x * f, if f > 1.0 { " (kept)" } else { "" })
        };
        if wide <= best_wide {
            note(&|| format!("{}: outside, or not first", place()));
            continue;
        }
        if !in_sight(view.at, aim, u.who(), squad) {
            note(&|| format!("{}: hidden", place()));
            continue;
        }
        best_wide = wide;
        if score <= best_score {
            note(&|| format!("{}: outside the box", place()));
            continue;
        }
        best_score = score;
        let cost = depth * (COST_BASE + COST_OFF * (1.0 - score));
        if cost < best_cost {
            best_cost = cost;
            lock = Some(u.who());
            note(&|| format!("{}: locks (score {score:.2}, {depth:.1} m deep)", place()));
        }
    }
    lock
}

/// The reticle each frame (see the module's notes).
#[allow(clippy::too_many_arguments)]
fn update_reticle(player: Res<Player>, squad: Res<Squad>, game: Res<GameData>, map: Res<MapLevel>,
                  camera: Query<(&Transform, &Projection), With<MainCamera>>, mut reticle: ResMut<Reticle>) {
    let Ok((tr, Projection::Perspective(pp))) = camera.single() else { return };
    if player.dead || player.loaded.is_none() {
        reticle.tint = Tint::Blue;
        reticle.lock = None;
        reticle.kept = None;
        return;
    }
    let view = View { at: tr.translation, turn: tr.rotation, tan: (pp.fov * 0.5).tan(), near: pp.near };
    // the crosshair's centre on the HUD (FUN_000ab390: 0.4 of the height, game-options'
    // h_1a504c05, the middle in the scope), as play_hud.rs draws it
    let centre = Vec2::new(SCREEN.x * 0.5, SCREEN.y * 0.5 * (1.0 - CROSSHAIR_UP * (1.0 - player.scope)));
    let held = player.loaded.as_ref().and_then(|l| l.weapons.get(player.weapon)).filter(|_| player.holding);
    let gun = held.and_then(|w| muzzle(&player, w).map(|m| (w, m)));
    let options = game.0.reticule;
    // (BF_RETICLE_LOG: the notes on each hostile, made only for the log)
    let log = std::env::var("BF_RETICLE_LOG").ok();
    let mut notes = log.as_ref().map(|_| vec![]);
    let chosen = select(&player, &squad, &view, centre, gun, &options, reticle.kept, notes.as_mut());
    // the centre ray (0xc5720), cut at the gun's range (FUN_002324c0) or 256 m with none
    let range = held.map_or(NO_GUN_RANGE, |w| w.def.range);
    let under = under_ray(view.at, view.through(centre).normalize(), range, &squad);
    let level = map.0.as_ref();
    let mut state = Tint::Blue;
    let mut direct = None;
    if chosen.is_none() {
        match under {
            Some((_, Under::Body(i))) => {
                let u = &squad.0[i];
                if damage::hostile(player.team, u.team) {
                    // FUN_000bf550 holds: with no gun, in the scope (char+0x7a8) or when the lock's
                    // reach is past the gun's range (FUN_00232590 > FUN_002324c0), it's locked
                    let scoped = player.scope > 0.5;
                    if held.is_none_or(|w| scoped || options.lock_share * reach(&w.def) > w.def.range) {
                        direct = Some(u.who());
                    }
                } else {
                    // FUN_0004b960: the same team, not one hostile to itself; then 0xc59d7
                    if u.team == player.team && !damage::hostile(u.team, u.team) {
                        state = Tint::Green;
                    }
                    if YELLOW_CONTACTS.contains(&u.radar_contact) {
                        state = Tint::Yellow;
                    }
                }
            }
            Some((_, Under::Object(Some(i)))) => {
                if level.and_then(|l| l.objects.get(i)).is_some_and(|o| YELLOW_CONTACTS.contains(&o.radar_contact)) {
                    state = Tint::Yellow;
                }
            }
            _ => {}
        }
    }
    reticle.kept = chosen;
    reticle.lock = chosen.or(direct);
    reticle.tint = if reticle.lock.is_some() { Tint::Red } else { state };
    // BF_RETICLE_LOG: on a change of the tint, the lock or what the centre ray is on (2: every
    // frame)
    let (Some(log), Some(notes)) = (log, notes) else { return };
    let on = match under {
        Some((_, Under::Body(i))) => format!("{:?}", squad.0[i].who()),
        Some((_, Under::Object(o))) => format!("{o:?}"),
        None => String::new(),
    };
    let key = format!("{:?} {:?} {on}", reticle.tint, reticle.lock);
    if log != "2" && key == reticle.logged {
        return;
    }
    reticle.logged = key;
    let locked = reticle.lock.and_then(|who| squad.0.iter().find(|u| u.who() == who)).map_or("none".to_string(), |u| {
        format!("{}{}", name(u), if direct.is_some() { " (under the centre)" } else { "" })
    });
    let what = match under {
        Some((t, Under::Body(i))) => {
            let u = &squad.0[i];
            format!("{} at {t:.1} m, team {}, radar-contact {}", name(u), u.team, u.radar_contact)
        }
        Some((t, Under::Object(Some(i)))) => level.and_then(|l| l.objects.get(i)).map_or(format!("object {i}"), |o| {
            format!("object h_{:08x} (type h_{:08x}) at {t:.1} m, radar-contact {}", o.name, o.kind, o.radar_contact)
        }),
        Some((t, Under::Object(None))) => format!("the world at {t:.1} m"),
        None => format!("nothing within {range:.0} m"),
    };
    let ray = view.through(centre).normalize();
    println!("t {:.2}: reticle {:?}: lock {locked}; centre on {what}; {}; camera at {:.2}, its ray heading {:.1} pitch {:.1} degrees",
             player.sim_time, reticle.tint, if notes.is_empty() { "no hostile".into() } else { notes.join("; ") }, view.at,
             (-ray.x).atan2(-ray.z).to_degrees(), ray.y.asin().to_degrees());
}

/// BF_TEST_SWEEP=<deg>|<from>,<to>,<deg/s>[;...]: turns the camera, so the crosshair sweeps across
/// what's in front: an entry of one number turns it that far (degrees, + to the left), one of
/// three turns it at that rate between those times (s, the controlled character's clock). With
/// BF_TEST_GOTO, from the heading from its start to its goal (down -z when they're the same), so
/// the camera doesn't follow the character's drift from the start (the autopilot aims it from
/// where the character is: with the goal 0.5 m away, a kneel's few centimetres turned it 2.6
/// degrees); else from the camera's heading as the run started.
fn sweep(mut player: ResMut<Player>, mut applied: Local<f32>) {
    let Ok(v) = std::env::var("BF_TEST_SWEEP") else { return };
    let t = player.sim_time;
    let turn = v.split(';').map(|e| {
        let n: Vec<f32> = e.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        match n.as_slice() {
            [deg] => *deg,
            [from, to, rate] => rate * (t.min(*to) - from).max(0.0),
            _ => 0.0,
        }
    }).sum::<f32>().to_radians();
    match test_goto() {
        Some(g) => {
            let base = if g[0] == g[2] && g[1] == g[3] { 0.0 } else { (-(g[2] - g[0])).atan2(-(g[3] - g[1])) };
            player.cam_yaw = wrap_angle(base + turn);
        }
        None => player.cam_yaw = wrap_angle(player.cam_yaw + turn - *applied),
    }
    *applied = turn;
}

/// BF_TEST_RADAR=<object>,<value>[;...]: as the map starts, the level's objects of that name
/// (h_xxxxxxxx) take that radar-contact (to see the yellow on a map whose level gives none, e.g.
/// sdm_e34's missile rack h_128f3de2). With BF_RETICLE_LOG, the map's objects that turn the
/// crosshair yellow are listed.
fn test_radar(mut map: ResMut<MapLevel>) {
    let Some(level) = map.0.as_mut() else { return };
    if let Ok(v) = std::env::var("BF_TEST_RADAR") {
        for e in v.split(';') {
            let mut it = e.split(',').map(|x| x.trim());
            let name = it.next().and_then(|n| u32::from_str_radix(n.trim_start_matches("h_").trim_start_matches("0x"), 16).ok());
            let value = it.next().and_then(|x| x.parse::<u8>().ok());
            let (Some(name), Some(value)) = (name, value) else {
                warn!("BF_TEST_RADAR: {e} isn't <object>,<value>");
                continue;
            };
            let mut n = 0;
            for o in level.objects.iter_mut().filter(|o| o.name == name) {
                o.radar_contact = value;
                n += 1;
            }
            println!("BF_TEST_RADAR: h_{name:08x} radar-contact {value} ({n} objects)");
        }
    }
    if std::env::var("BF_RETICLE_LOG").is_ok() {
        for o in level.objects.iter().filter(|o| YELLOW_CONTACTS.contains(&o.radar_contact)) {
            println!("radar-contact {}: h_{:08x} (type h_{:08x}) at {:.2}", o.radar_contact, o.name, o.kind, o.transform.w_axis.truncate());
        }
    }
}
