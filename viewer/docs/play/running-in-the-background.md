*Part of [play/](README.md), the former page `play.md`.*

### Running in the background (#163)

For agents' runs while someone uses the PC (`src/bin/play_background.rs`; tooling, not game
behaviour):

* `BF_BACKGROUND=1` (any value but `0`):
  * No gamepad is read: bevy's `GilrsPlugin` is left out, so no `Gamepad` exists. (Pads only
    drive the front end, `play_menu.rs`; in-game control is keyboard and mouse.)
  * The mouse's motion, button, wheel and pointer-move events are dropped at the start of each
    frame, before bevy_input, the UI or `read_input` see them, and bevy_picking's mouse input
    is off. So the cursor is never grabbed or hidden (the two places that lock it,
    `read_input`'s click and the test tools' Esc, also check the hook). The cursor position
    bevy_winit stores on the window, which the UI's hover reads, isn't touched; the menu only
    acts on a hover in a frame with a pointer-move event.
  * The window opens without taking focus, at the bottom of the z-order behind the other
    windows. bevy 0.16 shows a new window with focus, so on Windows it's created hidden and
    shown with user32 `SetWindowPos(HWND_BOTTOM, SWP_SHOWWINDOW | SWP_NOACTIVATE)`. winit still
    thinks it's hidden: a later change to the window's mode, decorations, resizability, level,
    buttons or hit test, or a maximize / minimize, would hide it for good (the demo makes none).
  * Updates run flat out, as a focused window's do.
  * The keyboard and raw mouse motion are left to the focus: winit only delivers them to a
    focused window. Clicking the window (or its taskbar button) brings it forward as usual.

  Use it with `BF_MUTE=1` and hook-driven runs (`BF_TEST_GOTO`, `BF_CAPTURE`, ...).
* `BF_WINDOW_MONITOR=<n>`: opens the window centred on monitor n (0-based, in the order Windows
  lists them; on this PC 0 is the main monitor and 1 the left one), with or without
  `BF_BACKGROUND`. Unset: where it opened before. Past the last monitor bevy logs "Couldn't get
  monitor selected with: Index(n)" and opens it at the default position.

Without either variable nothing changes. Checked (`BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20
BF_CAPTURE_FRAMES=60 BF_MUTE=1`): with `BF_BACKGROUND=1`, the foreground window (polled every
0.1 s with `GetForegroundWindow`) stayed the full-screen game in front for the whole run, the
demo's window came up visible at the bottom of the z-order, and the captures render normally
while it's covered. A background run gave the same 60 frames, pixel for pixel, as a run
without the hook (run on a separate, never-shown desktop so it couldn't take focus). Frames 2-5
differ between some runs either way, with or without the hook: the terrain and the weapon
appear a frame earlier or later depending on when their GPU pipelines finish compiling. With
`BF_WINDOW_MONITOR=1` the window opened centred on the left monitor. Not tried with live input
(a pad or mouse in use during a run).
