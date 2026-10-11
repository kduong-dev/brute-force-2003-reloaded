//! Test hooks for running the demo beside someone at the PC (#163). Not game behaviour: an
//! agent's capture run mustn't take the user's controller, mouse or focus.
//!
//!  - `BF_BACKGROUND=1` (any value but 0): no gamepad is read (bevy's gilrs plugin is left out,
//!    so no `Gamepad` entity exists), the mouse's motion, buttons, wheel and pointer moves are
//!    dropped before anything reads them (so the cursor is never grabbed or hidden either), and
//!    the window opens without focus, at the bottom of the z-order, behind the other windows.
//!    The keyboard is left alone: it only reaches a focused window, and this one is never
//!    focused unless the user clicks it. Frames render (and `BF_CAPTURE` saves them) whether
//!    the window is covered or not.
//!  - `BF_WINDOW_MONITOR=<n>`: opens the window centred on monitor n (0-based, in the order the
//!    system lists them), with or without `BF_BACKGROUND`. Unset: where it opened before.
//!
//! Without either variable nothing changes.

use bevy::{
    ecs::event::EventUpdates,
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorMoved, MonitorSelection, PrimaryWindow, RawHandleWrapper, WindowPosition},
    winit::{UpdateMode, WinitSettings, WinitWindows},
};

/// Whether `BF_BACKGROUND` is set (to anything but 0). Read once.
pub fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_BACKGROUND").is_ok_and(|v| v.trim() != "0"))
}

/// `BF_WINDOW_MONITOR=<n>`: the monitor to open the window on (0-based), if set.
fn monitor() -> Option<usize> {
    std::env::var("BF_WINDOW_MONITOR").ok().and_then(|v| v.trim().parse().ok())
}

/// The demo's primary window. Under `BF_BACKGROUND` it's created hidden and unfocused, and
/// `show_window` shows it without activating it: bevy 0.16 creates every window with winit's
/// default `active: true` (`Window::focused` isn't passed on at creation), and winit's first
/// show of such a window is a `ShowWindow(SW_SHOW)`, which brings it to the front with focus.
pub fn window() -> Window {
    let mut w = Window { title: "Brute Force".into(), ..default() };
    if let Some(n) = monitor() {
        w.position = WindowPosition::Centered(MonitorSelection::Index(n));
    }
    if on() {
        w.visible = false;
        w.focused = false;
    }
    w
}

/// Adds the background mode's systems (nothing without `BF_BACKGROUND`). After DefaultPlugins:
/// its `WinitSettings` replaces theirs.
pub fn plugin(app: &mut App) {
    if !on() {
        return;
    }
    // the window is never focused, so the default unfocused mode (an update at most every
    // 1/60 s) would hold every run to that; run flat out as a focused window does
    app.insert_resource(WinitSettings { focused_mode: UpdateMode::Continuous, unfocused_mode: UpdateMode::Continuous })
        .add_systems(First, drop_mouse.before(EventUpdates))
        .add_systems(Update, show_window);
}

/// Drops the frame's mouse events before anything reads them: the motion (raw device events,
/// which arrive whatever window has focus), the buttons, the wheel and the pointer's moves. With
/// them go `ButtonInput<MouseButton>`, `AccumulatedMouseMotion` / `AccumulatedMouseScroll`
/// (bevy_input), the UI's clicks and the menu's hover (`play_menu::input`), and so the left
/// click that grabs the cursor (`read_input`). Runs in First before the event buffers swap,
/// ahead of bevy_picking's input (also in First) and bevy_input's systems (PreUpdate).
fn drop_mouse(mut motion: ResMut<Events<MouseMotion>>, mut buttons: ResMut<Events<MouseButtonInput>>,
              mut wheel: ResMut<Events<MouseWheel>>, mut cursor: ResMut<Events<CursorMoved>>) {
    motion.clear();
    buttons.clear();
    wheel.clear();
    cursor.clear();
}

/// Shows the hidden window once it exists, at the bottom of the z-order and without
/// activating it (`SetWindowPos` with `SWP_SHOWWINDOW | SWP_NOACTIVATE`). On the main thread
/// (the `WinitWindows` parameter): the window belongs to it, and a synchronous call from a
/// worker could wait on the main thread while it waits on the schedule. Not on Windows the
/// window stays hidden (untested there).
fn show_window(_main_thread: NonSend<WinitWindows>, windows: Query<&RawHandleWrapper, With<PrimaryWindow>>,
               mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(handle) = windows.single() else { return };
    *done = true;
    #[cfg(windows)]
    if let raw_window_handle::RawWindowHandle::Win32(h) = handle.get_window_handle() {
        // SAFETY: the handle is our own live window (bevy keeps it while the entity has its
        // RawHandleWrapper), and the call is made on the thread that owns it.
        let ok = unsafe { win32::SetWindowPos(h.hwnd.get(), win32::HWND_BOTTOM, 0, 0, 0, 0, win32::SWP_FLAGS) };
        if ok == 0 {
            warn!("BF_BACKGROUND: SetWindowPos failed; the window stays hidden");
        }
    }
    #[cfg(not(windows))]
    let _ = handle;
}

/// The one user32 call `show_window` needs, declared here rather than pulling in a Windows
/// bindings crate. Values from the Windows SDK's WinUser.h.
#[cfg(windows)]
mod win32 {
    /// `HWND_BOTTOM`: the bottom of the z-order.
    pub const HWND_BOTTOM: isize = 1;
    /// `SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_SHOWWINDOW`.
    pub const SWP_FLAGS: u32 = 0x0001 | 0x0002 | 0x0010 | 0x0040;

    #[link(name = "user32")]
    extern "system" {
        /// user32's `SetWindowPos` (HWNDs passed as isize, the size of a pointer).
        pub fn SetWindowPos(hwnd: isize, insert_after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    }
}
