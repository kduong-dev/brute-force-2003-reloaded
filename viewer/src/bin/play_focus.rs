//! The mouse across a lost and regained window focus (the product owner's report, 2026-11-01:
//! "game aim locks showing cursor unless I fire my weapon", "it only happens when I alt tab").
//!
//! The demo grabs the mouse with `CursorGrabMode::Locked` and hides the cursor (play.rs
//! `mouse_capture`). On Windows winit has no lock, so bevy_winit falls back to a one-pixel
//! `ClipCursor` rectangle, and the system drops that clip when another window takes the
//! foreground. winit 0.30 doesn't put it back when the focus returns (its `WM_SETFOCUS` only sends
//! `Focused(true)`), and bevy_winit 0.16 re-applies the grab only when `Window::cursor_options`
//! differs from the value it applied last, which it still believes is `Locked`. So after an
//! alt-tab the window stays marked as captured while the OS cursor is free and visible; a left
//! click set the same values again (no change, nothing re-applied), which is why the aim only
//! seemed to work after the first shot.
//!
//! The fix keeps bevy_winit's picture true: on `WindowFocused { focused: false }` the grab and the
//! cursor's visibility are saved and released (`None`, visible), and on `focused: true` they are
//! put back, so the grab differs from what bevy_winit applied last and is applied again. A mouse
//! freed by Escape (or by the editor, or a menu) is saved as free and comes back free; its
//! recapture is a click of either button (play.rs `mouse_capture`).
//!
//! A real alt-tab can't be made under `BF_BACKGROUND` (the window is never focused), so
//! `BF_TEST_FOCUS=<capture>,<lose>,<gain>` plays one: at those frames (counted from the start) the mouse is captured as a click would (the click itself is dropped under
//! `BF_BACKGROUND`), then `WindowFocused` events lose and regain the focus; each step prints the
//! window's grab mode and cursor visibility. Under `BF_BACKGROUND` winit never actually grabs an
//! unfocused window, so this tests the demo's state, not the OS cursor.

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};

/// The mouse as it was when the focus left: the grab mode and whether the cursor was visible.
/// `None` while the window has focus (or the focus left with the mouse already free).
#[derive(Resource, Default)]
struct Saved(Option<(CursorGrabMode, bool)>);

/// Adds the focus handling (and the test hook).
pub fn plugin(app: &mut App) {
    app.init_resource::<Saved>().add_systems(Update, (test_focus, on_focus).chain());
}

/// Releases a captured mouse as the window loses focus and takes it back as the focus returns
/// (see the module's notes). A mouse that was free stays free; a grab made meanwhile (nothing
/// reads input unfocused, but a menu's close does set one) isn't overwritten.
fn on_focus(mut events: EventReader<WindowFocused>, mut saved: ResMut<Saved>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut w) = windows.single_mut() else { return };
    for e in events.read() {
        if e.focused {
            if let Some((grab, visible)) = saved.0.take() {
                if w.cursor_options.grab_mode == CursorGrabMode::None {
                    w.cursor_options.grab_mode = grab;
                    w.cursor_options.visible = visible;
                }
            }
        } else if w.cursor_options.grab_mode != CursorGrabMode::None {
            saved.0 = Some((w.cursor_options.grab_mode, w.cursor_options.visible));
            w.cursor_options.grab_mode = CursorGrabMode::None;
            w.cursor_options.visible = true;
        }
    }
}

/// BF_TEST_FOCUS=<capture>,<lose>,<gain> (frames): see the module's notes. Prints the mouse's state at
/// each step and one frame after it.
fn test_focus(mut frame: Local<u32>, mut writer: EventWriter<WindowFocused>, mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
              mut step: Local<usize>, mut wait: Local<u32>) {
    let Ok(v) = std::env::var("BF_TEST_FOCUS") else { return };
    *frame += 1;
    let at: Vec<u32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let Ok((entity, mut w)) = windows.single_mut() else { return };
    let state = |w: &Window| format!("grab {:?}, cursor {}", w.cursor_options.grab_mode, if w.cursor_options.visible { "visible" } else { "hidden" });
    if *wait > 0 {
        *wait -= 1;
        if *wait == 0 {
            println!("frame {}: BF_TEST_FOCUS: after step {}: {}", *frame, *step, state(&w));
        }
        return;
    }
    if *step >= 3 || at.get(*step).is_none_or(|&t| *frame < t) {
        return;
    }
    *step += 1;
    match *step {
        1 => {
            w.cursor_options.grab_mode = CursorGrabMode::Locked;
            w.cursor_options.visible = false;
        }
        2 => {
            writer.write(WindowFocused { window: entity, focused: false });
        }
        _ => {
            writer.write(WindowFocused { window: entity, focused: true });
        }
    }
    println!("frame {}: BF_TEST_FOCUS: step {} ({}) sent: {}", *frame, *step, ["capture", "focus lost", "focus gained"][*step - 1], state(&w));
    *wait = 2;
}
