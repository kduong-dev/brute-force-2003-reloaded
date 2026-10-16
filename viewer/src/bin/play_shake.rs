//! The camera shake (issue #73), as default.xbe's camera does it: the camera object keeps a
//! shake (its +0xdc block: amplitude, fall rate, a one-frame amplitude, hold time) that blasts
//! kick and that turns the view a little every game frame.
//!
//!  - **A kick** (FUN_0006bdd0: amplitude, hold, fall time) replaces the shake only if it's
//!    stronger than what's left of it; the amplitude then holds for the hold time and falls to
//!    nothing over the fall time (FUN_00073a80). A kick with no fall time lasts one frame.
//!  - **Each game frame** (the camera's update at 0x6c780: FUN_00073d20, FUN_00073c00, then
//!    FUN_00073a80): s = the amplitude, at most 1; the view turns by a random pitch and yaw
//!    within +-0.025 x s rad and a roll within half that (FUN_00072010, FUN_000720d0,
//!    FUN_00073d80 turn it about its x, y and z), new each frame, from the game's random
//!    numbers (FUN_00233b90's generator).
//!
//! Kicked by the missiles' blasts only (play_missile.rs, FUN_00228c70, whose only callers are
//! the missile's two explosion paths at 0x227867 and 0x227b1d): grenades and other blasts don't
//! shake the view through it (their effect types' shake-effects lists aren't played). The view
//! only, the demo's choice: the aim keeps the unshaken camera (play.rs's `camera_pose`). The
//! game multiplies the turn into the camera's own frame (0x6c54a, FUN_00073b20, the camera's
//! vtable +0x3c); whether its crosshair ray reads that frame isn't traced.
//!
//! Test hook: `BF_SHAKE_LOG=1` prints each kick and when a shake ends.

use super::*;

/// FUN_00073c00: the pitch and yaw reach this many radians at full amplitude...
const SHAKE_TURN: f32 = 0.025;
/// ... and the roll this share of that.
const SHAKE_ROLL: f32 = 0.5;
/// FUN_00073d20: the amplitude counts up to this much.
const SHAKE_MAX: f32 = 1.0;
/// The camera's update runs once per game frame: the turn is drawn anew this often (s), the
/// game's 30 Hz (shots::TICK).
const SHAKE_TICK: f32 = super::shots::TICK;

/// The shake's state (the camera object's +0xdc block).
#[derive(Resource, Default)]
pub struct CameraShake {
    /// +0xdc: the amplitude
    amp: f32,
    /// +0xe0: how fast it falls once its hold is over (per s)
    rate: f32,
    /// +0xe4: an amplitude for one frame (a kick with no fall time)
    once: f32,
    /// +0xe8: how long the amplitude holds before it falls (s)
    hold: f32,
    /// game time not yet run in ticks (s)
    clock: f32,
    /// the turn drawn on the last tick
    turn: Quat,
    seed: u32,
}

impl CameraShake {
    /// FUN_0006bdd0: shake with `amp`, held `hold` s and then falling to nothing over `fall` s,
    /// unless what's left of the current shake is stronger.
    pub fn kick(&mut self, amp: f32, hold: f32, fall: f32) {
        if self.amp >= amp {
            return;
        }
        if fall > 0.0 {
            self.amp = amp;
            self.hold = hold;
            self.rate = amp / fall;
        } else {
            self.once = amp;
        }
        if std::env::var("BF_SHAKE_LOG").is_ok() {
            println!("shake: {amp:.3} held {hold:.2} s, falling over {fall:.2} s");
        }
    }

    /// The game's random numbers in [0, 1) (shots::random), from the shake's own seed.
    fn random(&mut self) -> f32 {
        if self.seed == 0 {
            self.seed = 0x5AC3_0001;
        }
        super::shots::random(&mut self.seed)
    }

    /// One game frame: the turn from the amplitude (FUN_00073d20, FUN_00073c00), then the
    /// hold and the fall (FUN_00073a80).
    fn tick(&mut self, dt: f32) {
        let s = self.amp.max(self.once).min(SHAKE_MAX);
        self.turn = if s > 0.0 {
            let k = s * SHAKE_TURN;
            let pitch = (self.random() * 2.0 - 1.0) * k;
            let yaw = (self.random() * 2.0 - 1.0) * k;
            let roll = (self.random() * 2.0 - 1.0) * k * SHAKE_ROLL;
            Quat::from_rotation_x(pitch) * Quat::from_rotation_y(yaw) * Quat::from_rotation_z(roll)
        } else {
            Quat::IDENTITY
        };
        let was = self.amp > 0.0;
        if self.hold > dt {
            self.hold -= dt;
        } else {
            self.hold = 0.0;
            self.amp = if self.rate != 0.0 { (self.amp - dt * self.rate).max(0.0) } else { 0.0 };
        }
        self.once = 0.0;
        if was && self.amp <= 0.0 && std::env::var("BF_SHAKE_LOG").is_ok() {
            println!("shake: over");
        }
    }
}

/// The shake resource, run on the view after every camera has put it (PostUpdate, before the
/// transforms are propagated: the follow, death and free cameras set the view each frame). Not
/// on the map editor's camera in edit mode (play_editor.rs, #191): an editor's view holds still.
pub fn plugin(app: &mut App) {
    app.init_resource::<CameraShake>()
        .add_systems(OnEnter(AppState::Playing), |mut shake: ResMut<CameraShake>| *shake = CameraShake::default())
        .add_systems(PostUpdate, apply_shake.before(bevy::transform::TransformSystem::TransformPropagate)
            .run_if(in_state(AppState::Playing).and(not(in_state(super::editor::EditorMode::Edit)))));
}

/// The shake's turn on the view, its game frames run in SHAKE_TICK steps.
fn apply_shake(time: Res<Time>, mut shake: ResMut<CameraShake>, mut cam: Query<&mut Transform, With<MainCamera>>) {
    shake.clock += frame_dt(&time);
    while shake.clock >= SHAKE_TICK {
        shake.clock -= SHAKE_TICK;
        shake.tick(SHAKE_TICK);
    }
    if shake.turn != Quat::IDENTITY {
        if let Ok(mut tr) = cam.single_mut() {
            tr.rotation *= shake.turn;
        }
    }
}
