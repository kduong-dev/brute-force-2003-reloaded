//! The screen's depth of field (#38): the scope's blur and the low-health blur, one pass
//! (FUN_000702a0's camera block, 0x7173c-0x7188d; the renderer's stage 8, FUN_000e7d40, whose
//! composite FUN_000e5730 draws shader 0x1e in mode 2 — the ticket's second comment has the
//! reads). The game works it out on the follow camera and copies it to the renderer:
//!
//!  - **Scoped and alive** (cam+0x4c bit 3): S (+0x117) 1, mode 2 (0x40de14), bias (+0x120) 0,
//!    and the focus (+0x118, a depth byte) is the aim ray's hit distance (+0x1f8, clamped to the
//!    clip planes and eased at 1000 m/s). Here: `reticle::Reticle::depth`, the centre ray's depth
//!    the scope view already sizes its frame by (play_scope_view.rs).
//!  - **Low health** (the hit points under half the max, not scoped): S 0.8 (0x40de18), mode 2,
//!    the focus the character's own depth, and bias = hp / 0.5 - 1.
//!  - **The shader** (0x1e): rgb = 1/4 sharp + 1/2 of a half-size and 1/4 of a quarter-size copy;
//!    alpha = clamp(64 (2S - 1) |z - focus| + bias, 0, 1), which blends that over the picture.
//!    The slope (64 a metre at S 1) makes the alpha reach 1 about a centimetre past the focus, so
//!    the pass reads as: the focus distance sharp, everything else blurred; at low health the
//!    character stays sharp and the world either side of him blurs as the hit points fall.
//!
//! Here bevy's own depth-of-field pass (`DepthOfField`, `DepthOfFieldMode::Gaussian`) draws it,
//! placed before the tonemapping like the game's stage: the same focus distance, and an aperture
//! small enough that the circle of confusion saturates just past the focus as 0x1e's slope does.
//! **Not the game's shader**: the fixed quarter/half mix and the bias's small shift are not
//! reproduced (bevy's lens model is a smooth fall-off round a focal plane), so the strength is a
//! close approximation rather than the formula; the exact blend is a later step. The sky is left
//! as it is (`max_depth`), as the game's 8-bit depth byte leaves it at the far value it quantises
//! to. MSAA is no obstacle: bevy reads a multisampled depth texture (dof/mod.rs
//! `prepare_depth_of_field_view_bind_group_layouts`).
//!
//! Test hooks: `BF_DOF_LOG=1` prints each change of the effect (when it comes on and off, the
//! focus and the health), `BF_TEST_HURT` (play_chatter.rs) takes the player's hit points down to
//! try the low-health side, and the scope's own hooks (play/scope.md) the scope's.

use super::*;
use bevy::core_pipeline::dof::{DepthOfField, DepthOfFieldMode};

/// The share of the max hit points the low-health blur starts under (FUN_000702a0's 0.5, the
/// 0x3a51a4 comparison: `hp < 0.5 x max`, not scoped).
const DOF_LOW_HEALTH: f32 = 0.5;
/// The character's own depth for the focus at low health (m): the follow camera's boom, the
/// campaign camera block's 3.2 m at 1 degree behind the head (character-types.md, the classes'
/// `camera` blocks). A stand-in: the game takes his depth from the view, the demo's boom is the
/// follow camera's own (`play_camera.rs`), so a run with a closer or further camera has the focus
/// a little off. Demo-own, 2026-11-09.
const DOF_SELF_DEPTH: f32 = 3.2;
/// The focus's ease in the scope (m a second, cam+0x118's 1000 at 0x40de20), as the scope view's
/// frame sizes itself by; the low-health focus is the character's, which doesn't move.
const DOF_FOCUS_RATE: f32 = 1000.0;
/// The follow camera's near plane (m; the clip-plane-near h_04570f05, 0.1 in every level), which
/// the game clamps the focus to.
const DOF_NEAR: f32 = 0.1;
/// The far plane when the map names no follow camera (m; play_scope_view.rs's FAR).
const DOF_FAR: f32 = 250.0;
/// Bevy's aperture: small enough that the circle of confusion reaches its cap a centimetre or two
/// past the focus, the game's 0x1e slope's read (alpha 1 at 1/64 m at S 1). Demo-own, tuned by
/// eye against the shape above.
const DOF_APERTURE: f32 = 0.12;
/// The largest blur bevy may draw (px): the game's quarter-size copy is a four-pixel box at the
/// console's 640 x 480, so a wide cap here would be more than the effect ever was. Demo-own.
const DOF_MAX_COC: f32 = 16.0;

/// The effect's state: the focus it's on (m) and whether it has taken one since it came on (the
/// game's cam+0x1f4 bit 0: the first scoped frame's focus is taken at once, then followed).
#[derive(Resource, Default)]
pub struct Focus {
    depth: f32,
    taken: bool,
    /// (BF_DOF_LOG: what it printed last, so a still picture prints once)
    logged: String,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Focus>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| commands.insert_resource(Focus::default())).after(setup))
        .add_systems(Update, update.after(follow_camera).after(reticle::update_reticle).before(update_weapons)
            .run_if(in_state(AppState::Playing)));
}

/// `from` moved toward `to` by at most `step`.
fn ease_toward(from: f32, to: f32, step: f32) -> f32 {
    from + (to - from).clamp(-step, step)
}

/// The effect's settings this frame, and the camera's component put on or taken off with it.
fn update(mut commands: Commands, time: Res<Time>, player: Res<Player>, reticle: Res<reticle::Reticle>,
          game: Res<GameData>, map: Res<MapLevel>, mut focus: ResMut<Focus>,
          mut cams: Query<(Entity, Option<&mut DepthOfField>), With<MainCamera>>) {
    let dt = frame_dt(&time);
    let hp = (player.health / player.max_health.max(1.0)).clamp(0.0, 1.0);
    let scoped = player.scope_level > 0 && !player.dead;
    let low = !scoped && !player.dead && hp < DOF_LOW_HEALTH;
    let log = std::env::var("BF_DOF_LOG").is_ok();
    if !scoped && !low {
        focus.taken = false;
        if focus.logged != "off" {
            focus.logged = "off".into();
            for (e, on) in &mut cams {
                if on.is_some() {
                    commands.entity(e).remove::<DepthOfField>();
                }
            }
            if log {
                println!("t={:.2} depth of field off (health {:.0}%{})", player.sim_time, hp * 100.0,
                         if player.scope_level > 0 { ", dead in the scope" } else { "" });
            }
        }
        return;
    }
    let far = map.0.as_ref().and_then(|l| game.0.follow_cameras.get(&l.follow_camera)).copied().unwrap_or(DOF_FAR).max(DOF_NEAR + 1e-3);
    // the focus: the aim ray's depth in the scope (the scope view's), else his own depth
    let target = if scoped { reticle.depth.clamp(DOF_NEAR, far) } else { DOF_SELF_DEPTH.min(far) };
    focus.depth = if focus.taken { ease_toward(focus.depth, target, DOF_FOCUS_RATE * dt) } else { target };
    focus.taken = true;
    // (BF_DOF_APERTURE / BF_DOF_COC: the two lens figures, to tune them live)
    let aperture = std::env::var("BF_DOF_APERTURE").ok().and_then(|v| v.parse().ok()).unwrap_or(DOF_APERTURE);
    let max_coc = std::env::var("BF_DOF_COC").ok().and_then(|v| v.parse().ok()).unwrap_or(DOF_MAX_COC);
    let settings = DepthOfField {
        mode: DepthOfFieldMode::Gaussian,
        focal_distance: focus.depth,
        aperture_f_stops: aperture,
        max_circle_of_confusion_diameter: max_coc,
        max_depth: far,
        ..default()
    };
    if log {
        let line = format!("{} {:.1} m, health {:.0}%, {} camera(s)", if scoped { "scope" } else { "low health" }, focus.depth, hp * 100.0,
                           cams.iter().count());
        if line != focus.logged {
            focus.logged = line.clone();
            println!("t={:.2} depth of field: {line}", player.sim_time);
        }
    }
    for (e, on) in &mut cams {
        match on {
            Some(mut d) => *d = settings,
            None => {
                commands.entity(e).insert(settings);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The effect comes on under half health and in the scope, and the focus takes its first value
    /// at once, then follows at FOCUS_RATE (the game's cam+0x1f4 bit 0 and 0x40de20).
    #[test]
    fn the_focus_is_taken_at_once_and_then_followed() {
        let mut focus = Focus::default();
        assert!(!focus.taken);
        focus.depth = ease_toward(3.2, 40.0, DOF_FOCUS_RATE / 30.0);
        assert!((focus.depth - (3.2 + 1000.0 / 30.0)).abs() < 1e-3);
        // (a metre is a thirtieth of a second at the game's 30 and a frame at 1000 a second)
        assert!((ease_toward(0.0, 40.0, 1000.0 / 1000.0) - 1.0).abs() < 1e-6);
    }
}
