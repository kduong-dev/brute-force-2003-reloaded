*Part of [play/](README.md), the former page `play.md`.*

### The mouse across a lost focus (product owner's report, 2026-11-01)

"Game aim locks showing cursor unless I fire my weapon", which only happened after an alt-tab
(`src/bin/play_focus.rs`; the demo's own mouse, the game has none). Not the reticle: its lock
and the target marker run whenever a gun is in hand, aiming or not (`select` in play_reticle.rs
needs the gun in hand, `player.holding`, never the raised state), and the demo's right button
holds the gun raised as before.

* **Cause.** The capture is `CursorGrabMode::Locked` with the cursor hidden. On Windows winit has
  no lock, so bevy_winit falls back to a one-pixel `ClipCursor`, which the system drops when
  another window takes the foreground; winit 0.30 doesn't put it back at `WM_SETFOCUS`, and
  bevy_winit re-applies a grab only when `Window::cursor_options` differs from what it applied
  last (still `Locked`). So after an alt-tab the demo believed the mouse captured while the OS
  cursor was free and visible, and a click wrote the same values again (nothing applied).
* **Fix.** On `WindowFocused { focused: false }` the grab and the cursor's visibility are saved
  and released; on `focused: true` they are put back (so bevy_winit applies them again). A mouse
  that was free (Esc, the editor) stays free. `Saved` is cleared as a map ends (`end_play`), so a
  refocus in the front end can't lock the window with a grab the last map saved.
* **Order.** The focus systems run after `read_input`. Windows can deliver the focus gain and the
  click that brings the window back in one frame; if the grab were put back first, `read_input`
  would find the mouse already captured and the click would shoot. Read first, the click
  recaptures without firing, and the restore then finds the grab set and leaves it alone.
  (Review fix after 5eafecf, which had no ordering.)
* **Either mouse button recaptures** (play.rs `mouse_capture`, which Esc still frees, in cutscenes
  too). The left click's shot is dropped as before (`fire` needs the mouse captured before the
  frame); the right button aims at once (`aim` never needed the capture) and scopes at once
  (`scope_pressed` reads the capture after the press), except in the test tools' free camera,
  where the right button is the camera tool's and steps no scope. The Squad Command menu and its Move To
  cursor take the frame's presses before `read_input` (`read_buttons`), so Esc closing a menu,
  the right button cancelling the cursor and the left button sending the order recapture
  nothing; the editor has its own look-around capture (play_editor.rs) and `read_input` doesn't
  run there.
* **`BF_TEST_FOCUS=<capture>,<lose>,<gain>`** (frames): plays an alt-tab, which can't be done
  under `BF_BACKGROUND` (the window is never focused): at the first frame the mouse is captured
  as a click would, then `WindowFocused` events lose and regain the focus, printing the grab mode
  and cursor after each. Verified with `BF_MAP=sdm_e34 BF_TEST_GOTO=0,8,0,8 BF_TEST_FOCUS=10,20,30
  BF_MUTE=1 BF_BACKGROUND=1`: Locked and hidden after the capture, None and visible after the
  loss, Locked and hidden after the gain. **Not tested:** the real OS cursor (an unfocused window
  is never grabbed by winit, and a real alt-tab needs the user's screen), nor a real click of the
  right button; a run by hand is the check.
