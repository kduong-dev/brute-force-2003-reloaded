//! The death camera, as in the reference recording `Friendly Fire 2 + Death Cam.mp4` (Brutus
//! killed by his own gas grenade at 28.333 s, control passing to Hawk):
//!
//! * the frame the controlled character dies, the view cuts (no blend) to a camera framing the
//!   body: the character type's `offset-dead` (camera block `h_0d41e5f1`, `3.5 5 0` for the
//!   squad), read as 3.5 m above the ground the body lies on and 5 m back from it, aiming a
//!   little above and left of the pelvis (measured) so the body sits low and right of centre;
//! * it circles the body at a steady rate, travelling to its own right (counter-clockwise seen
//!   from above), starting from the follow camera's heading at the moment of death; near walls it
//!   pulls in along the line of sight to the body, and skips (cuts past) the part of the circle
//!   where a wall leaves less than DEATH_CAM_MIN_DISTANCE (the demo's choice: the game's own
//!   collision there isn't recorded);
//! * a squad switch picked just before the death is dropped, and a scope's zoom ends at once;
//! * at 3.5 s the next living squad member's name pops on over it (the squad switch's label),
//!   and at exactly 4.0 s the view cuts to that member's follow camera: the squad switch
//!   (`squad_control`) hands control over and the label grows and fades across the cut.
//!
//! With nobody left to hand over to (the whole squad down, or deathmatch, which has no squad)
//! there's no death camera: the follow camera stays on the body as before (#41 covers that case).
//! The red HUD tint and fade, the score popup and steering the orbit with the right stick are not
//! done here.

use super::*;

/// Seconds from the death to the cut to the next member (reference recording: the health bar
/// empties at 28.333 s and the cut to Hawk is at 32.333 s).
const DEATH_CAM_TIME: f32 = 4.0;
/// Seconds after the death at which the next member's name pops on over the death camera
/// (reference recording: "HAWK" appears at 31.833 s).
const DEATH_LABEL_AT: f32 = 3.5;
/// Orbit rate (radians per second, positive: counter-clockwise from above). An estimate: the
/// recording's gas smoke hides most of the background, and the turn measured through it is
/// 55-66 degrees per second. To be refined from a clearer recording.
const DEATH_CAM_RATE: f32 = 60.0 * std::f32::consts::PI / 180.0;
/// The view aims at the body's pelvis, turned up and left of it by these (radians), so the body
/// sits a little below and right of the centre. Measured from the recording (with the game's FOV
/// 60 taken as horizontal over the 4:3 picture): the body stays at about (380, 350) of 640 x 480
/// whether it stands (right after the cut), kneels or lies (29.3-30.0 s): 11 degrees below the
/// centre and 6 right. Right after the cut the horizon is about 80 of 360 rows above the centre,
/// which fits a standing pelvis 1.2 m up seen from 3.5 m up and 5 m back with the view turned up
/// 11 degrees from it. Measurements, not data.
const DEATH_CAM_LOOK_UP: f32 = 11.0 * std::f32::consts::PI / 180.0;
const DEATH_CAM_LOOK_LEFT: f32 = 6.0 * std::f32::consts::PI / 180.0;
/// The pelvis's height above the feet before the body has gone limp (a guess: the death frame of
/// a kill dealt after the bodies' update, see `body_at`).
const PELVIS_HEIGHT: f32 = 1.1;
/// `offset-dead` for a character type without one: the squad's `3.5 5 0`.
const DEFAULT_OFFSET_DEAD: [f32; 3] = [3.5, 5.0, 0.0];
/// How fast the orbit's centre and the point the view aims at follow the body as it falls and
/// slides (per second; the follow camera's own rate for its target, see `follow_camera`).
const BODY_FOLLOW_RATE: f32 = 12.0;
/// Walls. The game's own camera collision here isn't recorded (the recording's camera does end
/// up closer and lower near objects later on), so this is the demo's choice. The camera sits on
/// the line of sight from the body (this high above the pelvis) to where `offset-dead` puts it,
/// and keeps CLAMP_MARGIN (the follow camera's 0.3 m) in front of whatever the level has on
/// that line, so the body is never hidden behind rock.
const SIGHT_ABOVE: f32 = 0.3;
const CLAMP_MARGIN: f32 = 0.3;
/// ... It pulls in rather than collapse onto the body: never closer than this (the body and its
/// surroundings stay in view; a line of sight shorter than this, from a short `offset-dead`,
/// needs only its own length). Where the wall leaves less room, that part of the orbit is
/// skipped, a cut ahead to the next heading with room (searched in SKIP_STEP steps).
const DEATH_CAM_MIN_DISTANCE: f32 = 2.5;
const SKIP_STEP: f32 = 5.0 * std::f32::consts::PI / 180.0;
/// ... It eases in ahead of a wall, looking this far along the orbit, at PULL_IN_RATE per second,
/// and back out at PULL_OUT_RATE (always at once inside the room there is).
const LOOK_AHEAD: f32 = 20.0 * std::f32::consts::PI / 180.0;
const PULL_IN_RATE: f32 = 4.0;
const PULL_OUT_RATE: f32 = 2.0;

/// The running death camera: whose death, how long since, the orbit's heading at the death plus
/// the parts of the orbit skipped at walls, the point on the ground it circles, the pelvis it
/// aims at and how far it is from the body along the line of sight.
struct Orbit {
    character: usize,
    t: f32,
    yaw: f32,
    skip: f32,
    centre: Vec3,
    pelvis: Vec3,
    distance: f32,
}

impl Orbit {
    /// Where `offset-dead` (`up`, `back`) puts the camera at heading `yaw`, as the line of sight
    /// from the body: its start, direction, full length and the room along it before a wall.
    fn sight(&self, yaw: f32, up: f32, back: f32) -> (Vec3, Vec3, f32, f32) {
        let from = self.pelvis + Vec3::Y * SIGHT_ABOVE;
        let want = self.centre + Vec3::Y * up + Quat::from_rotation_y(yaw) * Vec3::new(0.0, 0.0, back);
        let (dir, full) = ((want - from).normalize_or(Vec3::Z), (want - from).length());
        let room = world::arena().and_then(|a| a.ray(from, dir, full)).map_or(full, |hit| (hit - CLAMP_MARGIN).max(0.0));
        (from, dir, full, room)
    }
}

/// The death camera's state: the orbit while it runs, and the last character seen dead in
/// control (so a death starts the camera once, not again after it gave up).
#[derive(Resource, Default)]
pub(super) struct DeathCam {
    orbit: Option<Orbit>,
    seen_dead: Option<usize>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DeathCam>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| commands.insert_resource(DeathCam::default())).after(setup))
        .add_systems(Update, death_cam.after(projectiles).before(update_hud).run_if(in_state(AppState::Playing)));
}

/// The body's pelvis (as the ragdoll lies, see `body_thud`; before it has gone limp, PELVIS_HEIGHT
/// above the feet) and the ground below it, which the orbit circles.
fn body(p: &Player) -> (Vec3, Vec3) {
    let pelvis = p.body_at.unwrap_or(p.position + Vec3::Y * (p.height + GROUND + PELVIS_HEIGHT));
    let floor = world::arena().and_then(|a| a.floor_below(pelvis.x, pelvis.z, pelvis.y + 0.5)).map_or(p.position.y + GROUND, |f| f.0);
    (pelvis, Vec3::new(pelvis.x, floor, pelvis.z))
}

/// The next living squad member control passes to (the same choice as before: the first in the
/// squad's order).
fn next_member(squad: &Squad) -> Option<usize> {
    squad.0.iter().find(|m| !m.dead && m.in_squad() && m.loaded.is_some()).map(|m| m.character)
}

/// Run the death camera: start it the frame the controlled character dies (if someone is left
/// to take over), circle the body, show the next member's name at DEATH_LABEL_AT and have the
/// squad switch cut to them at DEATH_CAM_TIME. Runs after `follow_camera` and overrides its view;
/// after `projectiles` too, so a blast's kill cuts on the frame the health bar empties.
#[allow(clippy::too_many_arguments)]
pub(super) fn death_cam(time: Res<Time>, mut state: ResMut<DeathCam>, mut player: ResMut<Player>, squad: Res<Squad>, game: Res<GameData>,
             mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>, mut vis: Query<&mut Visibility>) {
    let dt = frame_dt(&time);
    // control passed on (or the character came back): the camera is done
    if state.orbit.as_ref().is_some_and(|o| !player.dead || o.character != player.character) {
        state.orbit = None;
    }
    let newly_dead = player.dead && state.seen_dead != Some(player.character);
    state.seen_dead = player.dead.then_some(player.character);
    let started = newly_dead && next_member(&squad).is_some();
    if started {
        // the death frame: cut to the body, heading as the follow camera was. A switch picked
        // just before is dropped (the death camera hands over itself, at its end), and the
        // scope is gone at once: no zoom easing out, the body not hidden as in the scope
        player.select = None;
        player.scope = 0.0;
        player.zoom = 1.0;
        player.scope_level = 0;
        if let Some(Ok(mut v)) = player.loaded.as_ref().map(|l| vis.get_mut(l.root)) {
            *v = Visibility::Inherited;
        }
        let (pelvis, centre) = body(&player);
        state.orbit = Some(Orbit { character: player.character, t: 0.0, yaw: player.cam_yaw, skip: 0.0, centre, pelvis, distance: 0.0 });
    } else if let Some(o) = state.orbit.as_mut() {
        o.t += dt;
        let (pelvis, centre) = body(&player);
        let k = 1.0 - (-BODY_FOLLOW_RATE * dt).exp();
        o.centre = o.centre.lerp(centre, k);
        o.pelvis = o.pelvis.lerp(pelvis, k);
    }
    let Some(o) = state.orbit.as_mut() else { return };
    let t = o.t;
    // the next member's name, then the cut (squad_control counts the rest down and swaps; a
    // hair under the remaining time, so the swap lands on the DEATH_CAM_TIME frame)
    if t >= DEATH_LABEL_AT - 1e-4 && player.select.is_none() {
        match next_member(&squad) {
            Some(c) => player.select = Some((c, (DEATH_CAM_TIME - t - 1e-3).max(0.0))),
            None => {
                // the rest died meanwhile: nobody to hand over to, back to the follow camera
                state.orbit = None;
                return;
            }
        }
    }
    let [up, back, _] = game.0.character_camera.get(CHARACTERS[player.character]).map(|c| c.dead)
        .filter(|d| d[0] != 0.0 || d[1] != 0.0).unwrap_or(DEFAULT_OFFSET_DEAD);
    let mut yaw = o.yaw + DEATH_CAM_RATE * t + o.skip;
    // a wall leaves too little room here: skip ahead to the next heading with room (a cut); with
    // none round the whole circle, the heading with the most
    // (room enough: the minimum, or all of a line of sight shorter than it)
    let enough = |(_, _, full, room): (Vec3, Vec3, f32, f32)| room >= DEATH_CAM_MIN_DISTANCE.min(full);
    let mut cut = started;
    if !enough(o.sight(yaw, up, back)) {
        // each heading's room, once
        let steps = (std::f32::consts::TAU / SKIP_STEP) as usize;
        let around: Vec<_> = (0..steps).map(|k| (k as f32 * SKIP_STEP, o.sight(yaw + k as f32 * SKIP_STEP, up, back))).collect();
        let skip = around.iter().skip(1).find(|(_, s)| enough(*s)).map(|(a, _)| *a)
            .unwrap_or_else(|| around.iter().max_by(|a, b| a.1.3.total_cmp(&b.1.3)).map_or(0.0, |(a, _)| *a));
        o.skip += skip;
        yaw += skip;
        cut |= skip > 0.0;
    }
    let (from, dir, full, room) = o.sight(yaw, up, back);
    // ease in ahead of a wall coming up on the orbit (the parts it will skip don't count)
    let ahead = (0..=4).map(|j| o.sight(yaw + LOOK_AHEAD * j as f32 / 4.0, up, back))
        .filter(|&s| enough(s)).map(|s| s.3).fold(full, f32::min);
    o.distance = if cut {
        ahead.min(room)
    } else {
        let rate = if ahead < o.distance { PULL_IN_RATE } else { PULL_OUT_RATE };
        (o.distance + (ahead - o.distance) * (1.0 - (-rate * dt).exp())).min(room)
    };
    let eye = from + dir * o.distance;
    // test hook: BF_DEATHCAM_LOG=1 prints the camera each frame (time since the death, heading in
    // degrees and how much of it was skipped at walls, the distance from the body and the room
    // there, the camera and the body's pelvis)
    if std::env::var("BF_DEATHCAM_LOG").is_ok() {
        println!("deathcam {} t {t:.3} yaw {:.1} skipped {:.0} distance {:.2} of {full:.2} room {room:.2} eye {:.2} pelvis {:?} select {:?}",
                 CHARACTERS[o.character], yaw.to_degrees(), o.skip.to_degrees(), o.distance, eye,
                 player.body_at.map(|b| (b * 100.0).round() / 100.0), player.select);
    }
    // aim at the pelvis, then up and left of it
    let to = o.pelvis - eye;
    let (aim_yaw, aim_pitch) = ((-to.x).atan2(-to.z), (to.y / to.length().max(1e-3)).clamp(-1.0, 1.0).asin());
    let view = Quat::from_euler(EulerRot::YXZ, aim_yaw + DEATH_CAM_LOOK_LEFT, aim_pitch + DEATH_CAM_LOOK_UP, 0.0);
    if let Ok((mut tr, mut projection)) = cam.single_mut() {
        *tr = Transform::from_translation(eye).with_rotation(view);
        // the unzoomed view (follow_camera eased it this frame, from the scope's zoom)
        if let Projection::Perspective(pp) = projection.as_mut() {
            let fov = fov_at(1.0);
            if (pp.fov - fov).abs() > 1e-4 {
                pp.fov = fov;
            }
        }
    }
}
