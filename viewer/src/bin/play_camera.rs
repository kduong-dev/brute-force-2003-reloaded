//! The follow camera (#126) as default.xbe's does it: its update FUN_000702a0 (the follow
//! camera's vtable 0x3a2640 +0x6c), from the parameters the character it follows gives it each
//! frame (0x128280, the character's vtable +0x12c), and the body's fade as the camera comes close
//! (FUN_0011be80). The field of view it draws with is play.rs's `FOV_Y` (#269). Sources: the
//! spec, the leads, the analyst's answers (Q1-Q11) and the takes in `todo/126-follow-camera/`
//! (takes 01-21, each with the game's own memory logged at 20 Hz: `notes.md`) and the functions
//! cited (viewer/docs/code-map.md, "Camera"); the page is viewer/docs/camera.md.
//!
//!  - **The view is the aim** (`Player::cam_yaw`, `cam_pitch`: the game's aim frame, char+0x5bc):
//!    the camera looks along the aim, not at the character, from behind the pivot on the boom in
//!    that frame. The pitch is held to +-60 degrees (FUN_0011e7c0). No orbit, no zoom wheel. The
//!    game turns its camera toward the aim at up to 360 degrees/s (FUN_00073e40), which never
//!    binds; the mouse turns it at once.
//!  - **The pivot** (`wanted_pivot`): the character's x, z; its feet + its head's height at spawn
//!    (`Player::head_height`), x 0.7 crouched. A 0.25 m leash smooths it (FUN_00073710) and a
//!    0.25 m dead zone its height (FUN_00073970); the camera reads their inner points (cam+0x154,
//!    +0x17c), so it catches up with a running character within 0.5 s (take07: a 0.29 m bump as he
//!    sets off, none at constant speed or on a stop).
//!  - **The boom** (`params`): the type's offset-walk (distance, elevation) x (1 - 0.4 pitch),
//!    plus the aim shift: 1.75 m x s x min(1, 1 - 0.4 pitch) along the crosshair's ray, s from the
//!    shift timer (char+0x430: held at 3 s while the gun is raised or scoped, 0 to 1 over its last
//!    0.67 s). Scoped it blends, by the scope's progress, toward 0.5 m straight behind a point
//!    0.5 m above the root node (0x12887f-0x128ebe: every character's).
//!  - **Walls** (`wall_test`, FUN_000718f0): a wedge of seven points from the pivot back along the
//!    boom; where the world meets it the distance steps to 0.75 of it, 0.5, then 0.4 m. The world:
//!    the static mesh and the collidable objects as they stand, not characters, weapons or the
//!    level's blockers (`Arena::hull_triangles`).
//!  - **The distance eases** to that (FUN_00070090: a cubic re-planned each frame over 0.2 s out,
//!    0.14 s in) and the boom's direction turns toward the wanted one at its angle / 0.2 s a second
//!    (FUN_00073f90). A wanted place 10 m off, or a cut (a hand-over, a teleport, a script giving
//!    the view back), puts the camera there at once.
//!  - **The body** (`step_alpha`): see-through (alpha 0.65 to 0.5) while its bounding sphere covers
//!    60% of the screen or more (FUN_0006d240), at once, and back over 0.25 s; the guns it holds
//!    and carries with it (0x212d60); hidden in the scope and within 0.15 m of the near plane
//!    (0x126bc0). Another character between the camera and the one it follows fades by its depth
//!    (FUN_00116340; `body_alpha`): code only, a guess's.
//!  - **The scope's re-aim** (`reaim`): entering and leaving the scope, the point the reticle was
//!    on is kept, and while the button's lockout runs the aim's pitch keeps the crosshair's ray on
//!    it as the view moves (FUN_00122fa0, FUN_0011caa0 at 0x11d0e6; take15 / take16: the pitch
//!    -0.019 to +0.206 / +0.123 over the move in).
//!
//! Test hooks: `BF_CAMERA_LOG=1` prints the camera every 0.05 s (`log`; `=2` also the triangles the
//! wedge meets), `BF_CAMERA_AIM_AT=x,y,z` turns the aim so the crosshair's ray passes through that
//! point (`aim_at`); play.rs's `BF_CAMERA_PITCH`, `BF_CAMERA_YAW`, `BF_CAMERA_DISTANCE` (the
//! boom's distance for every character) and `BF_VIEW_YAW` (the drawn view only).

use super::*;

/// FUN_0011e7c0 (0x11e7c8): the aim's pitch, which the camera's is, is held to +-60 degrees (+
/// up). take01 / 10 / 11 / 12 reach exactly +-1.0472 and stay.
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_3;
/// 0x1283eb: the boom's distance is x (1 + DOLLY_PER_PITCH x the aim pitch) (0x3a5380): x0.58 at
/// the up limit, x1.42 at the down limit (take01: 5.534 m for Tex's 3.9 at -1.0472).
const DOLLY_PER_PITCH: f32 = -0.4;
/// 0x128316: crouched (motion state 6 or 7, or 6 queued) the pivot is at this share of the head's
/// height (0x3a5154). take07: 30.577 = 29.187 + 0.7 x 1.985.
const CROUCH_PIVOT: f32 = 0.7;
/// The aim shift (0x1284e1-0x1287fe): the boom gains SHIFT_REACH m (0x40cbd4) along the
/// crosshair's ray x clamp(SHIFT_RAMP x the timer, 0, 1) (0x3a4fb4) x min(1, the dolly). The timer
/// (char+0x430) is set to SHIFT_HOLD s (0x40cbd8) every frame the gun is raised (the ready timer
/// char+0x7dc runs) or scoped (char+0x7a8), for a character a player controls, and counts down by
/// dt after, SHIFT_DASH x dt dashing (0x11ccfc-0x11cd23: MT_DASH with char+0x5f0 above 0.1). So a
/// shot keeps the camera in for the ready timer's 4 s and 2.33 s more, out over the last 0.67 s
/// (take02: out from 6.44 s after the tap, done by 7.18; take03 6.28 / 7.06 after the last shot).
const SHIFT_REACH: f32 = 1.75;
const SHIFT_HOLD: f32 = 3.0;
const SHIFT_RAMP: f32 = 1.5;
const SHIFT_DASH: f32 = 3.0;
/// A type whose offset-walk is all zero gets this boom straight back (FUN_0012eca0: 1.55 m along
/// (0, 0, 1)).
const DEFAULT_BOOM: f32 = 1.55;
/// The leash on the pivot (FUN_00073660(0.25), FUN_00073710) and the dead zone on its height
/// (FUN_00073970, the same 0.25 at cam+0x174): their radius (m). Outside it the camera's point
/// catches up over CATCH_UP s (the leash's 0.5 at 0x73732; the dead zone's own +0x18c, the same 0.5
/// in the leads: take07's crouch and jumps follow it within ~0.3 s); inside it the point follows
/// by its time inside, capped at INSIDE_CAP s, so less and less.
const LEASH: f32 = 0.25;
const CATCH_UP: f32 = 0.5;
const INSIDE_CAP: f32 = 6.0;
/// FUN_00073970 keeps its outer point this share of the radius off the inner one (0.9999), so a
/// caught-up height is inside the dead zone on the next frame.
const DEAD_ZONE_EDGE: f32 = 0.9999;
/// The game's frame, which the per-frame weights below are the game's for (its 30 Hz,
/// shots::TICK): take07's run-start lag series is the leash's at 30 Hz (0.27 m peak against the
/// take's 0.29, back to 0 by 0.6 s), and a demo frame of dt takes `game_frames` of them.
const GAME_FRAME: f32 = shots::TICK;
/// FUN_00070090 eases the distance toward the wall test's result along a cubic, re-planned every
/// frame, over EASE_OUT s (0x3f7f14), or EASE_OUT x EASE_IN (0x3a5154, 0.14 s) when the result is
/// nearer (0x70b02-0x70b26); a frame longer than that snaps to it. take02's first step in (8.7% of
/// the way at -11 m/s) fits 0.140; the takes' moves out fit 0.14-0.17 better than the code's 0.2
/// (the spec's C3): the code's is kept.
const EASE_OUT: f32 = 0.2;
const EASE_IN: f32 = 0.7;
/// FUN_00073f90 (from 0x708f0, with FUN_0005a6e0's angle / 0x3f7f14): the boom's direction turns
/// toward the wanted one by their angle x dt / TURN_TIME a frame, all the way once that's all of
/// it. take02: the elevation 0.9 -> 6.4 degrees over ~0.8 s while the distance is in by 0.25 s.
const TURN_TIME: f32 = 0.2;
/// The wedge (FUN_000718f0): from the pivot, two points straight back along the boom at its
/// distance and four at half of it (WEDGE_SIDE_REACH) turned WEDGE_SIDE either way (0x3f060a92),
/// each pair WEDGE_TILT above and below the boom (0.08726647).
const WEDGE_SIDE: f32 = 30.0 * std::f32::consts::PI / 180.0;
const WEDGE_TILT: f32 = 5.0 * std::f32::consts::PI / 180.0;
const WEDGE_SIDE_REACH: f32 = 0.5;
/// Where the world meets the wedge it's shrunk by a quarter of the distance and tested again
/// (`param_6 * 0.25`): clear at 0.75 or 0.5 of it, the camera sits there; still meeting it at 0.5,
/// WEDGE_FLOOR m (0x3f7f18), whatever the quarter-size wedge meets.
const WEDGE_STEPS: [f32; 2] = [0.75, 0.5];
const WEDGE_FLOOR: f32 = 0.4;
/// A wanted camera place this far (m) from where the camera is cuts to it (100, squared, at
/// 0x3a4fec).
const CUT_DISTANCE: f32 = 10.0;
/// The scope's camera (0x128899-0x128d6a, with a gun in hand while the scope's progress char+0x7cc
/// is above 0): a pivot SCOPE_PIVOT m above the root node (0x3a4f74) and the camera SCOPE_BOOM m
/// straight behind it on the aim's axis (0x3f000000 at 0x12894c), both blended in by the progress.
/// take15 / take16: Flint's boom 0.500 m, up 0.000, the camera ~0.07 m under her head's pivot.
const SCOPE_PIVOT: f32 = 0.5;
const SCOPE_BOOM: f32 = 0.5;
/// FUN_0011be80: the body's alpha (char+0x72c) is 1 while its sphere covers less than COVER_FROM of
/// the screen (0x40cc0c); from there to COVER_FULL (0x40cc08) ALPHA_STEP - (ALPHA_STEP -
/// ALPHA_FLOOR) x (1 - x^2), x = 1 - (cover - COVER_FROM) / (COVER_FULL - COVER_FROM) (0x40cc04,
/// 0x40cc00): a step from 1 to 0.65, down to 0.5 at full cover. It drops to that at once; rising,
/// a timer (char+0x730) runs from 0 to RISE_TIME (0x40cbfc) and each frame the alpha closes (timer
/// - (RISE_TIME - RISE_SPAN)) / RISE_SPAN of the gap (0x40cbf8), all of it once the timer is out.
/// The takes' alpha (a72c) against this model over 3,190 steady rows of Tex's: rms 0.003.
const COVER_FROM: f32 = 0.6;
const COVER_FULL: f32 = 1.0;
const ALPHA_STEP: f32 = 0.65;
const ALPHA_FLOOR: f32 = 0.5;
const RISE_TIME: f32 = 0.25;
const RISE_SPAN: f32 = 0.5;
/// 0x126bc0 (0x3a51e4): the body isn't drawn by its own camera while the camera's distance less the
/// near plane is under this (m); the wall test's 0.4 m floor keeps it out of reach but in a dip of
/// the ease (take19: 0.33 at least).
const HIDE_NEAR: f32 = 0.15;
/// The follow camera's near plane (clip-plane-near h_04570f05: 0.1 in every level's archetype;
/// bevy's default too, which the demo's camera keeps).
const NEAR: f32 = 0.1;
/// The scope's re-aim (FUN_0011caa0 at 0x11d4ac) turns the aim's heading too when the kept point
/// lies more than this (the sideways part of its unit direction in the camera's frame) to the side;
/// else the pitch alone.
const REAIM_SIDE: f32 = 0.025;
/// The point the crosshair is on without a gun in hand (the re-aim takes the gun's range,
/// FUN_002324c0): where its ray meets the world within this (m), else that far along it. The
/// reticle's 256 m, the demo's.
const REAIM_RANGE: f32 = 256.0;
/// BF_CAMERA_LOG: a line this often (s), the watch logs' 20 Hz.
const LOG_EVERY: f32 = 0.05;

/// The wall test's result (FUN_000718f0), for the log: clear, one of the steps, or the floor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Wall {
    #[default]
    Clear,
    Step(f32),
    Floor,
}

/// FUN_00073710's leash on the pivot (the follow camera's +0x14c): `inner` (+0x154) is the point
/// the camera's pivot is on, `outer` (+0x160) the leash's other end. Outside the leash the inner
/// point catches up (`catch` of the gap a game frame, by its time outside) and the outer one hangs
/// LEASH behind it; inside, both follow by their time inside, t / (t + dt) kept a frame.
#[derive(Clone, Debug, Default)]
struct Leash {
    inside: bool,
    inner: Vec3,
    outer: Vec3,
    time_in: f32,
    time_out: f32,
}

impl Leash {
    /// FUN_000736c0 (a cut): both ends on `at`, inside, its time inside from 0.
    fn reset(&mut self, at: Vec3) {
        *self = Leash { inside: true, inner: at, outer: at, time_in: 0.0, time_out: 0.0 };
    }

    /// One frame of FUN_00073710 toward `want`: the inner point.
    fn step(&mut self, want: Vec3, dt: f32) -> Vec3 {
        let off = self.outer - want;
        let d = off.length();
        if d > LEASH {
            if self.inside {
                self.inside = false;
                self.time_out = 0.0;
            }
            self.time_out += dt;
            self.inner = if self.time_out < CATCH_UP {
                self.inner.lerp(want, game_frames(self.time_out / CATCH_UP, dt))
            } else {
                want
            };
            self.outer = self.inner + off / d * LEASH;
        } else if self.inside {
            let t = self.time_in + dt;
            let keep = if t > 0.0 { self.time_in / t } else { 0.0 };
            self.inner = self.inner * keep + want * (1.0 - keep);
            self.outer = self.outer * keep + self.inner * (1.0 - keep);
            self.time_in = t.min(INSIDE_CAP);
        } else {
            self.inside = true;
            self.time_in = 0.0;
            self.inner = want;
        }
        self.inner
    }
}

/// FUN_00073970's dead zone on the pivot's height (the follow camera's +0x174), the leash's scheme
/// in one dimension: `inner` (+0x17c) is the camera's pivot height.
#[derive(Clone, Debug, Default)]
struct DeadZone {
    inside: bool,
    inner: f32,
    outer: f32,
    time_in: f32,
    time_out: f32,
}

impl DeadZone {
    /// The cut's reset (0x70668: +0x17c and +0x180 to the wanted height, inside, time 0).
    fn reset(&mut self, y: f32) {
        *self = DeadZone { inside: true, inner: y, outer: y, time_in: 0.0, time_out: 0.0 };
    }

    /// One frame of FUN_00073970 toward height `y`: the inner height.
    fn step(&mut self, y: f32, dt: f32) -> f32 {
        if (self.outer - y).abs() <= LEASH {
            if self.inside {
                let t = dt + self.time_in;
                let keep = if t > 0.0 { self.time_in / t } else { 0.0 };
                self.inner = (1.0 - keep) * y + keep * self.inner;
                self.outer = keep * self.outer + self.inner * (1.0 - keep);
                self.time_in = t.min(INSIDE_CAP);
            } else {
                self.inside = true;
                self.time_in = 0.0;
                self.inner = y;
            }
            return self.inner;
        }
        if self.inside {
            self.inside = false;
            self.time_out = 0.0;
        }
        self.time_out += dt;
        self.inner = if self.time_out < CATCH_UP {
            self.inner + (y - self.inner) * game_frames(self.time_out / CATCH_UP, dt)
        } else {
            y
        };
        self.outer = self.inner + if self.outer - y > 0.0 { DEAD_ZONE_EDGE * LEASH } else { -DEAD_ZONE_EDGE * LEASH };
        self.inner
    }
}

/// The follow camera of the character the player controls: the camera object's state
/// (FUN_000702a0) and the character's fields it drives (the aim shift, the body's alpha). Each
/// character carries one; the controlled one's runs (`update`), and a hand-over cuts the new one's.
#[derive(Clone, Debug)]
pub struct Rig {
    /// put the camera at its place at once next frame, no smoothing (the cut flag, cam+0x1f4 bit 1:
    /// set as the camera is made, here also on a hand-over, a teleport, a script's camera handing
    /// back)
    pub cut: bool,
    leash: Leash,
    height: DeadZone,
    /// the smoothed pivot the camera hangs from (the leash's x, z, the dead zone's y)
    pivot: Vec3,
    /// cam+0x19c: the boom's direction in the aim frame (+z back), turned toward the wanted one
    dir: Vec3,
    /// cam+0x190 / cam+0x1f0: the eased distance (m) and its speed (m/s)
    distance: f32,
    speed: f32,
    /// this frame's wanted distance before the wall test (D'), the wall test's result and the
    /// distance it eased toward (for the log)
    wanted: f32,
    wall: Wall,
    target: f32,
    /// the camera as placed (the view: its place and turn, BF_VIEW_YAW aside)
    pub at: Vec3,
    pub turn: Quat,
    /// char+0x430: the aim shift timer (s, see SHIFT_HOLD)
    pub shift: f32,
    /// char+0x72c / +0x730: the body's alpha by its own camera, and the rise's timer
    pub alpha: f32,
    alpha_timer: f32,
    /// the screen share the body's sphere covered last frame (for the log)
    cover: f32,
    /// the scope's re-aim: the point kept under the crosshair while the view moves in or out of the
    /// scope, relative to the root node (char+0x7bc), and how long it has left (s: the scope
    /// button's lockout, char+0x7c8)
    reaim: Option<(Vec3, f32)>,
    /// the turn (yaw, pitch) the re-aims have given the aim, which the test hooks that set the aim
    /// every frame (BF_TEST_GOTO's BF_CAMERA_PITCH, BF_TEST_SWEEP) add to theirs, so a scope's
    /// re-aim holds under them as it does under the mouse
    pub hook_turn: Vec2,
    /// in the scope last frame (its level above 0)
    scoped: bool,
    /// BF_CAMERA_LOG's next line (sim time)
    log_at: f32,
}

impl Default for Rig {
    fn default() -> Self {
        Rig { cut: true, leash: Leash::default(), height: DeadZone::default(), pivot: Vec3::ZERO, dir: Vec3::Z, distance: 0.0, speed: 0.0,
              wanted: 0.0, wall: Wall::Clear, target: 0.0, at: Vec3::ZERO, turn: Quat::IDENTITY, shift: 0.0, alpha: 1.0,
              alpha_timer: RISE_TIME, cover: 0.0, reaim: None, hook_turn: Vec2::ZERO, scoped: false, log_at: 0.0 }
    }
}

/// The camera plugin: the BF_CAMERA_AIM_AT hook, after the input and the hooks that turn the aim
/// (`sweep`), before the character steps on the aim it gives. The camera itself is placed in
/// play.rs's `follow_camera` (`update`), after the step.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, aim_at.after(squad_control).before(update_player).run_if(in_state(AppState::Playing)));
}

/// A share `w` of the gap closed in one of the game's frames (GAME_FRAME), spread over a demo frame
/// of `dt`: 1 - (1 - w)^(dt / GAME_FRAME), the game's own at 30 fps.
fn game_frames(w: f32, dt: f32) -> f32 {
    1.0 - (1.0 - w.clamp(0.0, 1.0)).powf(dt / GAME_FRAME)
}

/// The aim frame (the game's char+0x5bc): yaw, then pitch (+ up), with the scope's wander.
pub fn aim_turn(p: &Player) -> Quat {
    Quat::from_euler(EulerRot::YXZ, p.cam_yaw + p.sway.x, p.cam_pitch + p.sway.y, 0.0)
}

/// The tangent of the crosshair's ray above the view's axis: CROSSHAIR_UP of the half-height in the
/// view at its zoom (FUN_000ab390's point through FUN_0006dfc0), at the centre in the scope (its
/// progress moves it there).
fn crosshair_tan(p: &Player) -> f32 {
    CROSSHAIR_UP * (1.0 - p.scope) * (fov_at(p.zoom) * 0.5).tan()
}

/// The crosshair's ray in the camera's frame (unit; -z ahead): 5.996 degrees up at the game's
/// field of view, unscoped.
pub fn crosshair_local(p: &Player) -> Vec3 {
    Vec3::new(0.0, crosshair_tan(p), -1.0).normalize()
}

/// The camera as `p`'s rig placed it, turned to `p`'s aim now (the aim moves before the camera is
/// placed again: the character's step reads it): its place and its turn. Its view and the
/// crosshair's ray (play.rs `aim_ray`).
pub fn view(p: &Player) -> (Vec3, Quat) {
    let turn = aim_turn(p);
    (p.rig.pivot + turn * (p.rig.dir * p.rig.distance), turn)
}

/// `character`'s boom from its type's offset-walk (the camera block h_0d41e5f1, FUN_0012eca0): its
/// distance (m; BF_CAMERA_DISTANCE overrides it) and its direction in the aim frame, (sin yaw cos
/// el, sin el, cos yaw cos el) from its elevation and yaw (degrees). Tex 3.9 m at 0.9, Brutus 3.4
/// at 7, Flint 3.2 at 1, Hawk 2.9 at 1.5 (take01 / 10-12: 3.915 / 3.420 / 3.225 / 2.919 level).
fn boom_of(game: &Game, character: usize) -> (f32, Vec3) {
    let walk = CHARACTERS.get(character).and_then(|n| game.character_camera.get(*n)).map(|c| c.walk).unwrap_or([0.0; 3]);
    let (d, dir) = if walk == [0.0; 3] {
        (DEFAULT_BOOM, Vec3::Z)
    } else {
        let (el, yaw) = (walk[1].to_radians(), walk[2].to_radians());
        (walk[0], Vec3::new(yaw.sin() * el.cos(), el.sin(), yaw.cos() * el.cos()))
    };
    let d = std::env::var("BF_CAMERA_DISTANCE").ok().and_then(|v| v.parse::<f32>().ok()).filter(|d| *d > 0.0).unwrap_or(d);
    (d, dir)
}

/// The pivot the character wants (0x128280): its x, z; its feet's height (char+0x438) + its head's
/// height (char+0x48c), x CROUCH_PIVOT kneeling or going down to it (the motion state 6 or 7, or 6
/// queued: take07's pivot goes down with the crouch's transition and up with the stand's).
fn wanted_pivot(p: &Player) -> Vec3 {
    let crouched = matches!(p.action, Action::Crouching | Action::Crouched);
    let feet = p.position.y + GROUND + p.height;
    Vec3::new(p.position.x, feet + p.head_height * if crouched { CROUCH_PIVOT } else { 1.0 }, p.position.z)
}

/// The character's camera parameters this frame (0x128280): the pivot it wants and the boom in the
/// aim frame (its length the distance before the wall test). The boom: the offset x the dolly,
/// plus the aim shift along the crosshair's ray (in the aim frame, as the camera's own frame is);
/// scoped, both blended by the scope's progress toward the scope's camera.
fn params(p: &Player, game: &Game) -> (Vec3, Vec3) {
    let (d, dir) = boom_of(game, p.character);
    let dolly = 1.0 + DOLLY_PER_PITCH * p.cam_pitch;
    let mut boom = dir * (d * dolly);
    let s = (p.rig.shift * SHIFT_RAMP).clamp(0.0, 1.0);
    if s > 0.0 {
        boom += crosshair_local(p) * (SHIFT_REACH * s * dolly.min(1.0));
    }
    let mut pivot = wanted_pivot(p);
    if p.scope > 0.0 && p.holding {
        let turn = aim_turn(p);
        let near = missile::root_point(p) + Vec3::Y * SCOPE_PIVOT;
        let at = (pivot + turn * boom).lerp(near + turn * (Vec3::Z * SCOPE_BOOM), p.scope);
        pivot = pivot.lerp(near, p.scope);
        boom = turn.inverse() * (at - pivot);
    }
    (pivot, boom)
}

/// FUN_00073f90: `from` turned toward `to` (both unit) by their angle (FUN_0005a6e0) x dt /
/// TURN_TIME, or `to` once that's all of it.
fn turn_toward(from: Vec3, to: Vec3, dt: f32) -> Vec3 {
    let angle = from.angle_between(to);
    let step = angle * dt / TURN_TIME;
    let axis = from.cross(to);
    if step >= angle || axis.length_squared() < 1e-12 {
        return to;
    }
    Quat::from_axis_angle(axis.normalize(), step) * from
}

/// FUN_00070090: one step of `dt` along the cubic from (`x`, speed `v`) to (`target`, 0) over `t`
/// s: the new place and speed.
fn ease(x: f32, v: f32, target: f32, t: f32, dt: f32) -> (f32, f32) {
    let d = target - x;
    let a = (3.0 * d - 2.0 * v * t) / (t * t);
    let b = (v * t - 2.0 * d) / (t * t * t);
    (x + (v + (a + b * dt) * dt) * dt, v + (2.0 * a + 3.0 * b * dt) * dt)
}

/// The wedge's seven points (FUN_000718f0) at distance `d` from `pivot` along `back` (unit, world):
/// the pivot; at `d` straight back and at d x WEDGE_SIDE_REACH WEDGE_SIDE either way, each
/// WEDGE_TILT above and below the boom's elevation.
fn wedge(pivot: Vec3, back: Vec3, d: f32) -> [Vec3; 7] {
    let yaw = back.x.atan2(back.z);
    let el = back.y.clamp(-1.0, 1.0).asin();
    let mut points = [pivot; 7];
    let mut k = 1;
    for tilt in [WEDGE_TILT, -WEDGE_TILT] {
        for (side, reach) in [(-WEDGE_SIDE, WEDGE_SIDE_REACH), (0.0, 1.0), (WEDGE_SIDE, WEDGE_SIDE_REACH)] {
            let (y, e) = (yaw + side, el + tilt);
            points[k] = pivot + Vec3::new(y.sin() * e.cos(), e.sin(), y.cos() * e.cos()) * (d * reach);
            k += 1;
        }
    }
    points
}

/// FUN_000718f0, the wall test: the distance the camera may have along `back` from `pivot`, of `d`.
/// The world's triangles that meet the wedge at `d` (`Arena::hull_triangles`; none: `d`); then the
/// wedge at 0.75 and 0.5 of it against those, the first clear one's; none clear, WEDGE_FLOOR. On
/// the flat test floor without an arena, its ground at GROUND. take04 / take19: steps begin with
/// the gate panel at d, 0.75 d and 0.5 d behind the pivot, within a 20 Hz row.
fn wall_test(pivot: Vec3, back: Vec3, d: f32) -> (f32, Wall) {
    if d <= 0.0 {
        return (d, Wall::Clear);
    }
    let points = wedge(pivot, back, d);
    let tris = match world::arena() {
        Some(a) => a.hull_triangles(&points),
        None => world::Arena::flat(GROUND).hull_triangles(&points),
    };
    if tris.is_empty() {
        return (d, Wall::Clear);
    }
    if std::env::var("BF_CAMERA_LOG").is_ok_and(|v| v == "2") {
        for t in tris.iter().take(4) {
            println!("camera: the wedge meets {:.2} {:.2} {:.2}", t[0], t[1], t[2]);
        }
    }
    for k in WEDGE_STEPS {
        let smaller = wedge(pivot, back, d * k);
        if !tris.iter().any(|t| world::convex_meet(&smaller, t)) {
            return (d * k, Wall::Step(k));
        }
    }
    (WEDGE_FLOOR, Wall::Floor)
}

/// The room the wall test leaves `p`'s camera at heading `yaw` (its pitch now, its wanted pivot,
/// no aim shift): the distance it may have and the boom it wants (play.rs `clear_follow_yaw`, the
/// hand-over's choice of heading).
pub fn room_at(p: &Player, game: &Game, yaw: f32) -> (f32, f32) {
    let (d, dir) = boom_of(game, p.character);
    let full = d * (1.0 + DOLLY_PER_PITCH * p.cam_pitch);
    let turn = Quat::from_euler(EulerRot::YXZ, yaw, p.cam_pitch, 0.0);
    (wall_test(wanted_pivot(p), turn * dir, full).0, full)
}

/// FUN_0006d240: how much of the game's 4:3 screen a sphere (centre, radius) covers seen from `at`
/// turned `turn`: its centre in the view, +- its radius across and up, over its depth and the
/// view's half tangents (70 degrees across, GAME_TAN_ACROSS, and 3/4 of that down), the right and
/// top edges held to 1 and the left and bottom to -1 as the game holds them, (x1 - x0)(y1 - y0) / 4
/// within 0..1. The game's own picture, whatever the demo's window.
fn cover(at: Vec3, turn: Quat, centre: Vec3, radius: f32) -> f32 {
    let v = turn.inverse() * (centre - at);
    let depth = -v.z;
    if depth.abs() < 1e-4 {
        return 1.0;
    }
    let (across, down) = (depth * targeting::GAME_TAN_ACROSS, depth * targeting::GAME_TAN_ACROSS * 0.75);
    let (x1, x0) = (((v.x + radius) / across).min(1.0), ((v.x - radius) / across).max(-1.0));
    let (y1, y0) = (((v.y + radius) / down).min(1.0), ((v.y - radius) / down).max(-1.0));
    let c = (x1 - x0) * (y1 - y0) * 0.25;
    if c.is_finite() { c.clamp(0.0, 1.0) } else { 1.0 }
}

/// FUN_0011be80's alpha for a cover `c` (see COVER_FROM).
fn alpha_for(c: f32) -> f32 {
    if c < COVER_FROM {
        return 1.0;
    }
    let x = 1.0 - (c.min(COVER_FULL) - COVER_FROM) / (COVER_FULL - COVER_FROM);
    (ALPHA_FLOOR - ALPHA_STEP) * (1.0 - x * x) + ALPHA_STEP
}

/// The bounding sphere's radius the body's fade takes (m), by squad slot (Brutus, Flint, Hawk,
/// Tex), round the scene node (`sphere`). Fits [F] to the takes' alpha (a72c) through the model
/// (FUN_0006d240 + FUN_0011be80) over their steady rows, the centre on the node: Tex 1.28 (takes
/// 01-05, 09, 13, 14, 17, 20: 3,190 rows, rms 0.0035, none misjudged), Flint 1.13 (take11: rms
/// 0.0005, 1.0-1.16 all fit), Hawk 0.97 (take12: 0.85-1.0 fit), Brutus 1.0 (take10: 1.1 and up
/// misjudge rows). The game's is its scene node's (0x158ca0 -> FUN_00236670: the union of the
/// model's part spheres in the node's frame, todo/126 answers.md Q5); the demo's union of them
/// (`Character::bounds`) sits 0.13 m lower for Tex and misjudges 193 rows (rms 0.10), so the fit.
const FADE_RADIUS: [f32; SQUAD] = [1.0, 1.13, 0.97, 1.28];

/// A character's bounding sphere for the body's fade, in the world (centre, radius): round its
/// scene node (`missile::root_point`: the feet + its clip's root-height, the hips), FADE_RADIUS for
/// the squad's; another's the model's union (`Character::bounds`) radius.
fn sphere(u: &Player) -> Option<(Vec3, f32)> {
    let l = u.loaded.as_ref()?;
    let radius = FADE_RADIUS.get(u.character).copied().or(l.model.bounds.map(|b| b.1))?;
    Some((missile::root_point(u), radius))
}

/// One frame of FUN_0011be80 for the controlled character, from the view its camera had last frame
/// (the takes' alpha trails the camera by about a game frame): the cover's alpha, dropped to at
/// once, risen to over RISE_TIME; 1 for the dead.
fn step_alpha(p: &mut Player, dt: f32) {
    if p.dead {
        p.rig.alpha = 1.0;
        return;
    }
    let Some((centre, radius)) = sphere(p) else { return };
    let c = cover(p.rig.at, p.rig.turn, centre, radius);
    p.rig.cover = c;
    let a = alpha_for(c);
    let rig = &mut p.rig;
    if a < rig.alpha {
        rig.alpha = a;
    }
    if rig.alpha_timer == RISE_TIME && rig.alpha < a {
        rig.alpha_timer = 0.0;
    }
    rig.alpha_timer = (rig.alpha_timer + dt).clamp(0.0, RISE_TIME);
    if rig.alpha_timer >= RISE_TIME {
        rig.alpha = a;
    } else if rig.alpha_timer > RISE_TIME - RISE_SPAN {
        let w = (rig.alpha_timer - (RISE_TIME - RISE_SPAN)) / RISE_SPAN;
        rig.alpha += (a - rig.alpha) * game_frames(w, dt);
    }
}

/// How much of `u`'s body is drawn by the follow camera of `followed` (FUN_00116480 draws it at
/// its spawn fade x this): for the character the camera follows its own alpha (`Rig::alpha`); for
/// another, FUN_00116340's depth fade: nearer the camera than the followed one's sphere, which is
/// in front of it, |its depth / the followed one's| (a squadmate halfway to the camera at 0.5),
/// else 1. The depth fade is code only (#126 spec E5: take06 too faint to measure), a guess's.
pub fn body_alpha(u: &Player, followed: &Player) -> f32 {
    if u.who() == followed.who() {
        return if u.dead { 1.0 } else { followed.rig.alpha };
    }
    let (Some((mine, _)), Some((theirs, _))) = (sphere(u), sphere(followed)) else { return 1.0 };
    let depth = |p: Vec3| -(followed.rig.turn.inverse() * (p - followed.rig.at)).z;
    let (z, zf) = (depth(mine), depth(theirs));
    if zf > 0.0 && z < zf { (z / zf).abs().clamp(0.0, 1.0) } else { 1.0 }
}

/// Whether the follow camera leaves `p`'s body undrawn: in the scope (the zoom isn't 1: cam+0x4c
/// bit 0) or with the camera within HIDE_NEAR of the near plane (0x126bc0). take16: no body from
/// the zoom's first frame to its last.
pub fn hides_body(p: &Player) -> bool {
    p.zoom > 1.0 || p.rig.distance - NEAR < HIDE_NEAR
}

/// The point the reticle is on for the scope's re-aim, its crosshair `up` (the tangent of its ray
/// above the view's axis) from `p`'s camera as it stands (FUN_00122fa0's: the lock's aim point,
/// the reticle's +0xac, while it holds one; else its ray's, +0xb8: where the ray meets the world
/// within the gun's range, FUN_002324c0, else at that range).
fn reticle_point(p: &Player, up: f32) -> Vec3 {
    if let Some(at) = p.lock_aim {
        return at;
    }
    let range = p.loaded.as_ref().and_then(|l| l.weapons.get(p.weapon)).map_or(REAIM_RANGE, |w| skills::weapon_range(p, &w.def));
    let (eye, turn) = view(p);
    let ray = turn * Vec3::new(0.0, up, -1.0).normalize();
    eye + ray * ray_hit(eye, ray, range).unwrap_or(range)
}

/// The aim (yaw, pitch) that puts the crosshair's ray on `point`: the camera turns with the aim
/// (its boom is in the aim frame), so a few rounds, each aiming from the place the last one gives
/// it; the crosshair's ray lies in the view's up-down plane, so its pitch is the view's + its own
/// angle.
fn aim_onto(p: &Player, point: Vec3) -> (f32, f32) {
    let (mut yaw, mut pitch) = (p.cam_yaw, p.cam_pitch);
    let up = crosshair_tan(p).atan();
    for _ in 0..AIM_ROUNDS {
        let turn = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        let eye = p.rig.pivot + turn * (p.rig.dir * p.rig.distance);
        let to = point - eye;
        if to.length_squared() < 1e-6 {
            break;
        }
        let was = (yaw, pitch);
        yaw = (-to.x).atan2(-to.z);
        pitch = ((to.y / to.length()).clamp(-1.0, 1.0).asin() - up).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        if wrap_angle(yaw - was.0).abs() < 1e-6 && (pitch - was.1).abs() < 1e-6 {
            break;
        }
    }
    (yaw, pitch)
}

/// `aim_onto`'s rounds at most: each takes the camera's swing with the aim (its boom over the
/// point's distance) off the error, so a handful settle it from any start (the demo's bound).
const AIM_ROUNDS: usize = 16;

/// The aim shift timer this frame (char+0x430, see SHIFT_HOLD): held while the gun is raised (its
/// ready timer, `FireState::raised`, or a skill's, `Skill::raised`) or the scope is up, for the
/// controlled character; else counted down, faster dashing; 0 for the dead.
fn step_shift(p: &mut Player, dt: f32) {
    let raised = p.fire_state.raised() || p.skill.raised > 0.0 || p.scope_level > 0;
    let dashing = matches!((p.action, p.gait), (Action::None, Gait::Sprint));
    p.rig.shift = if p.dead || !p.controlled {
        0.0
    } else if raised {
        SHIFT_HOLD
    } else {
        (p.rig.shift - dt * if dashing { SHIFT_DASH } else { 1.0 }).max(0.0)
    };
}

/// The scope's re-aim (`Rig::reaim`; the spec's F2, Q10 of todo/126 answers.md): at the scope's
/// entry (CT_AIM_SNIPE, 0x1223f2) and exit (FUN_001249b0, 0x124a17) FUN_00122fa0 keeps the point
/// the reticle is on (`reticle_point`: at the entry through the crosshair's unscoped place,
/// CROSSHAIR_UP of the half-height up; at the exit through the scope's centre), relative to the
/// root node (char+0x7bc, so it rides with the character); then while the scope button's lockout
/// runs (char+0x7c8: SCOPE_IN_LOCK / SCOPE_OUT_LOCK at SCOPE_CALLS a frame, whatever started the
/// change: a hook's level too) the character update (FUN_0011caa0, 0x11d0e6-0x11d56c) sets the
/// aim's pitch so the crosshair's ray from where the camera stands passes through it (FUN_0011e7c0
/// at 0x11d4c4 / 0x11d567), and its heading too when the point lies more than REAIM_SIDE to the
/// side (0x11d4ac). take15: the pitch -0.019 -> +0.085 at +0.05 s, +0.206 at +0.2 s, as the camera
/// moves in (the gate panel's point within 0.18 degrees of the scoped view's axis); back to -0.008
/// over the move out.
fn reaim(p: &mut Player, dt: f32) {
    let scoped = p.scope_level > 0;
    // (a camera being cut has no view to keep a point from: a hand-over to a character left
    // scoped when it was last controlled starts afresh)
    if p.rig.cut {
        (p.rig.scoped, p.rig.reaim) = (scoped, None);
        return;
    }
    if scoped != p.rig.scoped && !p.dead {
        let up = if scoped { CROSSHAIR_UP * (FOV_Y * 0.5).tan() } else { 0.0 };
        let point = reticle_point(p, up);
        let lock = if scoped { SCOPE_IN_LOCK } else { SCOPE_OUT_LOCK } / SCOPE_CALLS;
        p.rig.reaim = Some((point - missile::root_point(p), lock));
        if std::env::var("BF_CAMERA_LOG").is_ok() {
            println!("t={:.3} camera: the scope {}: the aim kept on {:.2} for {lock:.3} s", p.sim_time, if scoped { "in" } else { "out" }, point);
        }
    }
    p.rig.scoped = scoped;
    let Some((offset, left)) = p.rig.reaim else { return };
    if left <= 0.0 || p.dead {
        p.rig.reaim = None;
        return;
    }
    p.rig.reaim = Some((offset, left - dt));
    let to = offset + missile::root_point(p) - p.rig.at;
    if to.length_squared() < 1e-6 {
        return;
    }
    let dir = to.normalize();
    let pitch = (dir.y.clamp(-1.0, 1.0).asin() - crosshair_tan(p).atan()).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    let yaw = if (p.rig.turn.inverse() * dir).x.abs() > REAIM_SIDE { (-dir.x).atan2(-dir.z) } else { p.cam_yaw };
    p.rig.hook_turn += Vec2::new(wrap_angle(yaw - p.cam_yaw), pitch - p.cam_pitch);
    (p.cam_yaw, p.cam_pitch) = (yaw, pitch);
}

/// The follow camera's frame (FUN_000702a0), after the character's step: the shift timer, the
/// body's alpha from last frame's view, the scope's re-aim; the parameters; a cut, or the leash,
/// the dead zone and the boom's turn; the wall test and the ease; the camera's place. Returns the
/// view to draw (its place and turn), BF_VIEW_YAW turning it about the pivot.
pub fn update(p: &mut Player, game: &Game, dt: f32) -> (Vec3, Quat) {
    step_shift(p, dt);
    // (a camera about to be cut has no view of last frame to measure the body by: its first
    // placement, a hand-over's, a teleport's)
    if !p.rig.cut {
        step_alpha(p, dt);
    }
    reaim(p, dt);
    let (want, boom) = params(p, game);
    let turn = aim_turn(p);
    let wanted = boom.length();
    let dir = if wanted > 1e-6 { boom / wanted } else { Vec3::Z };
    let place = want + turn * boom;
    let far = p.rig.at.distance_squared(place) > CUT_DISTANCE * CUT_DISTANCE;
    let rig = &mut p.rig;
    rig.wanted = wanted;
    if rig.cut || far {
        // the cut (0x70668): the leash and the dead zone on the wanted pivot, the boom as wanted;
        // the cut flag's frame ends there (no wall test, 0x70339-0x70345), a 10 m jump's goes on
        rig.leash.reset(want);
        rig.height.reset(want.y);
        rig.pivot = want;
        rig.dir = dir;
        (rig.distance, rig.speed, rig.target, rig.wall) = (wanted, 0.0, wanted, Wall::Clear);
        if rig.cut {
            rig.cut = false;
            (rig.at, rig.turn) = (place, turn);
            return drawn(p);
        }
    }
    let inner = rig.leash.step(want, dt);
    let y = rig.height.step(inner.y, dt);
    rig.pivot = Vec3::new(inner.x, y, inner.z);
    if dt > 0.0 {
        rig.dir = turn_toward(rig.dir, dir, dt);
    }
    let back = turn * rig.dir;
    let (room, wall) = wall_test(rig.pivot, back, wanted);
    let target = room.min(wanted).max(0.0);
    let t = if target < rig.distance { EASE_OUT * EASE_IN } else { EASE_OUT };
    (rig.distance, rig.speed) = if dt <= t { ease(rig.distance, rig.speed, target, t, dt) } else { (target, 0.0) };
    (rig.target, rig.wall) = (target, wall);
    (rig.at, rig.turn) = (rig.pivot + back * rig.distance, turn);
    log(p);
    drawn(p)
}

/// The view to draw: the camera's, turned BF_VIEW_YAW about the pivot (a test hook: the aim and the
/// controls keep the camera's).
fn drawn(p: &Player) -> (Vec3, Quat) {
    let view = std::env::var("BF_VIEW_YAW").ok().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    if view == 0.0 {
        return (p.rig.at, p.rig.turn);
    }
    let r = Quat::from_rotation_y(view);
    (p.rig.pivot + r * (p.rig.at - p.rig.pivot), r * p.rig.turn)
}

/// BF_CAMERA_LOG=1: the camera every LOG_EVERY s, in the watch logs' order and names (todo/126's
/// `.watch.csv`) so the two read side by side: the time, the feet, the aim's pitch and yaw, the
/// smoothed pivot, the camera, the boom before the wall test (d'), the eased distance's target
/// and itself (d190), the wall test's step, the ready and aim shift timers (t7dc, t430), the
/// cover and the body's alpha (a72c), scoped, and the view's field across (zoomed).
fn log(p: &mut Player) {
    if std::env::var("BF_CAMERA_LOG").is_err() || p.sim_time < p.rig.log_at {
        return;
    }
    p.rig.log_at = p.sim_time + LOG_EVERY;
    let r = &p.rig;
    let feet = p.position + Vec3::Y * (GROUND + p.height);
    let wall = match r.wall { Wall::Clear => "1".to_string(), Wall::Step(k) => format!("{k}"), Wall::Floor => "floor".to_string() };
    let across = 2.0 * ((fov_at(p.zoom) * 0.5).tan() * targeting::GAME_TAN_ACROSS / (FOV_Y * 0.5).tan()).atan();
    println!("t={:.3} camera: feet {:.3} {:.3} {:.3} pitch {:+.4} yaw {:.4} piv {:.3} {:.3} {:.3} cam {:.3} {:.3} {:.3} d' {:.3} target {:.3} d190 {:.3} \
              wall {wall} t7dc {:.3} t430 {:.3} cover {:.3} a72c {:.2} scoped {} zfov {:.2}",
             p.sim_time, feet.x, feet.y, feet.z, p.cam_pitch, p.cam_yaw, r.pivot.x, r.pivot.y, r.pivot.z, r.at.x, r.at.y, r.at.z, r.wanted,
             r.target, r.distance, p.fire_state.ready_left().max(p.skill.raised), r.shift, r.cover, r.alpha, p.scope_level, across.to_degrees());
}

/// BF_CAMERA_AIM_AT=x,y,z: each frame the aim turns so the crosshair's ray passes through that
/// point (`aim_onto`, from the camera as last placed: it settles over the first frames as the
/// camera does), in place of BF_CAMERA_PITCH / BF_TEST_SWEEP's turn: a recipe aims at a place, not
/// at a pitch that puts the crosshair there for one camera.
fn aim_at(mut player: ResMut<Player>) {
    let Ok(v) = std::env::var("BF_CAMERA_AIM_AT") else { return };
    let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    let [x, y, z] = n[..] else { return };
    let (yaw, pitch) = aim_onto(&player, Vec3::new(x, y, z));
    (player.cam_yaw, player.cam_pitch) = (yaw, pitch);
}

/// The head's height (m above the feet) a character's camera pivots on (char+0x48c): its "Bip01
/// Head" bone at spawn (0x12fefe in FUN_0012f440), the model posed by its idle clip's first frame
/// with its root node at the clip's root-height (FUN_0023d240 / FUN_0023d4f0; the model's own,
/// GROUND below its root, without one); the constructor's 2.0 without a head (0x111aa0). The takes'
/// pivots less the feet: Tex 1.985, Brutus 1.555, Flint 1.644, Hawk 1.487; this gives 1.985, 1.559,
/// 1.645, 1.488 (the bind pose's 1.990, 1.526, 1.644, 1.481).
pub fn head_height(model: &Character, game: &Game, idle: Option<usize>) -> f32 {
    let Some(head) = model.bones.iter().position(|&b| b == bf_viewer::bf::hash::h("Bip01 Head")) else { return 2.0 };
    let Some(idle) = idle.filter(|&c| c < model.anims.len()) else { return model.bind_pos[head].y - GROUND };
    let root = model.anims[idle].root_height.unwrap_or(-GROUND);
    root + model.world(&model.pose(game, idle, 0.0, None, 0.0))[head].w_axis.y
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The convex test: a triangle across the wedge meets it, one beside it or past its tip
    /// doesn't, one just inside its tip does.
    #[test]
    fn wedge_meets() {
        let pivot = Vec3::new(10.0, 2.0, -30.0);
        let w = wedge(pivot, Vec3::Z, 4.0);
        let across = [pivot + Vec3::new(-5.0, -5.0, 3.0), pivot + Vec3::new(5.0, -5.0, 3.0), pivot + Vec3::new(0.0, 5.0, 3.0)];
        assert!(world::convex_meet(&w, &across));
        let beside = across.map(|v| v + Vec3::X * 20.0);
        assert!(!world::convex_meet(&w, &beside));
        let behind = across.map(|v| v + Vec3::Z * 1.5);
        assert!(!world::convex_meet(&w, &behind));
        let tip = [pivot + Vec3::new(-1.0, -1.0, 3.98), pivot + Vec3::new(1.0, -1.0, 3.98), pivot + Vec3::new(0.0, 1.0, 3.98)];
        assert!(world::convex_meet(&w, &tip));
        let past = tip.map(|v| v + Vec3::Z * 0.02);
        assert!(!world::convex_meet(&w, &past));
    }

    /// The wedge's steps against a wall facing the boom (FUN_000718f0; the spec's D1 table): clear
    /// past d, 0.75 d, 0.5 d, then the floor.
    #[test]
    fn wall_steps() {
        let d = 3.929;
        let step = |w: f32| {
            let wall = [Vec3::new(-50.0, -50.0, w), Vec3::new(50.0, -50.0, w), Vec3::new(0.0, 50.0, w)];
            let tris = [wall];
            let meets = |k: f32| tris.iter().any(|t| world::convex_meet(&wedge(Vec3::ZERO, Vec3::Z, d * k), t));
            if !meets(1.0) { d } else if !meets(0.75) { 0.75 * d } else if !meets(0.5) { 0.5 * d } else { WEDGE_FLOOR }
        };
        assert_eq!(step(6.0), d);
        assert_eq!(step(3.69), 0.75 * d);
        assert_eq!(step(2.69), 0.5 * d);
        assert_eq!(step(1.29), WEDGE_FLOOR);
    }

    /// FUN_0011be80's alpha at the takes' covers: 1 under 0.6, 0.65 at it, 0.54 at 0.8, 0.5 full.
    #[test]
    fn alpha_by_cover() {
        assert_eq!(alpha_for(0.59), 1.0);
        assert!((alpha_for(0.6) - 0.65).abs() < 1e-6);
        assert!((alpha_for(0.8) - 0.5375).abs() < 1e-4);
        assert!((alpha_for(1.0) - 0.5).abs() < 1e-6);
    }

    /// The leash at 30 Hz on a run from rest (to ~5.5 m/s over 0.3 s): the pivot lags as the
    /// character leaves the leash, catches up within 0.5 s of that and has no lag at a steady run
    /// (take07: a 0.29 m bump, 0 within ~0.65 s), and none after the stop.
    #[test]
    fn leash_catches_up() {
        let mut l = Leash::default();
        l.reset(Vec3::ZERO);
        l.time_in = INSIDE_CAP;
        let (mut x, mut peak) = (0.0f32, 0.0f32);
        for k in 0..45 {
            let t = (k + 1) as f32 * GAME_FRAME;
            if t < 1.2 {
                x += GAME_FRAME * (5.5 * (t / 0.3)).min(5.5);
            }
            let inner = l.step(Vec3::new(0.0, 0.0, -x), GAME_FRAME);
            let lag = x - (-inner.z);
            peak = peak.max(lag);
            if t > 0.8 {
                assert!(lag.abs() < 0.01, "lag {lag} at {t}");
            }
        }
        assert!((0.25..0.5).contains(&peak), "peak {peak}");
    }

    /// FUN_00070090's cubic from rest: the first 25 ms of a 3.915 -> 2.172 move at T 0.14 goes
    /// 8.7% of the way (take02's first row).
    #[test]
    fn ease_first_step() {
        let (x, _) = ease(3.915, 0.0, 2.172, 0.14, 0.025);
        let share = (3.915 - x) / (3.915 - 2.172);
        assert!((share - 0.087).abs() < 0.01, "share {share}");
    }

    /// The field of view: 70 degrees across the game's 4:3 picture.
    #[test]
    fn field_of_view() {
        assert!(((FOV_Y * 0.5).tan() / 0.75 - targeting::GAME_TAN_ACROSS).abs() < 1e-6);
    }
}
