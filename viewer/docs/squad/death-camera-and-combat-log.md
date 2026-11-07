*Part of [squad/](README.md), the former page `squad.md`.*

* **Death camera** (`play_deathcam.rs`, from the reference recording `Friendly Fire 2 + Death
  Cam.mp4`, 28.3-32.4 s). On the frame the character you control dies, the view cuts (no
  blend) to a camera on the body and the reticle goes. The camera uses the character type's
  `offset-dead`: camera block `h_0d41e5f1`, element f13fb0c4, `3.5 5 0` for the squad. The
  objecttypes parser `FUN_0018fa80` stores the seven offsets at +0x1b0 in the character type,
  and all seven are now read into `Game::character_camera`. The camera sits 3.5 m above the
  ground under the pelvis and 5 m back. It aims 11 degrees above and 6 degrees left of the
  pelvis, so the body sits low and right of centre, at about (380, 350) of 640 x 480 as in
  the recording (both angles measured). It circles the body counter-clockwise seen from above,
  travelling to its own right, at a steady 60 degrees per second. It starts from the follow
  camera's heading. The rate is an estimate: 55-66 degrees per second measured through the
  recording's gas smoke.

  Walls: the game's own camera collision here isn't recorded, so this is the demo's choice.
  The camera sits on the line of sight from the body (0.3 m above the pelvis) to where
  `offset-dead` puts it, and keeps 0.3 m in front of whatever the level has on that line, so
  rock never hides the body. It eases in ahead of a wall, looking 20 degrees along the orbit.
  It never comes closer than 2.5 m: where a wall leaves less room, that part of the circle is
  skipped with a cut ahead to the next heading with room.

  On the death frame a squad switch picked just before is dropped, so the death camera always
  runs its full 4 s. A scope ends at once (no zoom easing out, the body not hidden).

  At 3.5 s the next living member's name appears over it, at its final size from its first
  frame, as in the recording (f1910 and f1911 are the same), as on a manual switch (**The name
  label**, #88). At exactly 4.0 s the squad switch cuts to that member's follow camera, and the
  name swells and fades across the cut. Keys 1-4 do nothing while you are dead. With nobody left to take
  over (the whole squad down (#41), or deathmatch) the follow camera stays on the body as
  before; on the test map the player then respawns where it stood 4 s after the death (#302,
  the demo's rule there: [testtools.md](../testtools/the-squad.md) "The squad").

  The hand-over, for a death or a manual switch, cuts to the camera straight behind the new
  character's facing, the whole body in view, as the recording's cut to Hawk does: the follow
  camera put at its place at once (#126: every character's camera is cut to its place each
  frame while no player controls it, so take21's hand-over to Brutus jumps without an ease;
  [camera.md](../camera.md)). If the follow camera's wall test leaves less than its whole boom
  there, the heading swings in 10 degree steps to the nearest one either side that leaves it
  (the demo's choice: the game's hand-over with a wall behind isn't filmed). The new
  character's aim, fire, walking and squad-AI idling (the leader's heading it turned to,
  kneeling) are dropped, so the cut is to the plain follow camera with them standing still.
  Standing a kneeling member up is the demo's guess.

  After the cut the follow camera is the game's ([camera.md](../camera.md)): turned toward a close
  wall its wall test steps it to 0.75, then 0.5 of its boom, then 0.4 m, and the body turns
  see-through as the camera comes close (#126, which took #89's walls in).

  Not done: the HUD's red tint and fade, the red score popup, steering with the right stick.
  `BF_DEATHCAM_LOG=1` prints the camera every frame (heading, how much was skipped at walls,
  distance and room). `BF_TEST_DIE=<s>` kills the player once per level.

  Verified at 15 fps on sdm_e34 (`BF_MAP=sdm_e34 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=<n>`, plus):
  - `BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`:
    - frame 15 has the reticle on;
    - frame 16: Tex dies, the view cuts and the reticle goes;
    - frames 16-75: 60 frames turning 4 degrees each;
    - frame 69: "BRUTUS" (t = 3.53 s);
    - frame 76: the cut to Brutus, from behind and whole. After that the test hook's autopilot
      turns the camera toward its goal.
  - `BF_TEST_GOTO=16,37,16,37 BF_TEST_DIE=0.5`, against the cliff:
    - the body stays in view at 5.5-5.9 m;
    - at t = 3.6 s (frame 62) the orbit skips 130 degrees of cliff and lands 2.8 m out;
    - frame 68: the cut to Brutus from behind.
  - `BF_TEST_DIE=1` from the map's start: the cut at frame 76 is to Brutus's back, and he
    stays facing away.
  - `BF_CHARACTER=1 BF_TEST_SCOPE=1 BF_TEST_DIE=2`: Flint scoped through frame 29; frame 30
    cuts to the unzoomed death camera with the body in view.
  - `BF_TEST_SELECT=0 BF_TEST_DIE=1.2`: the manual label shows at frames 16-17, Tex dies at
    18, and the label goes. The death camera runs its 60 frames. "BRUTUS" is the same size at
    frames 71, 72 and 73, and the cut is at frame 78.
  - `BF_TEST_SELECT=0`: the manual switch's label (frame 16; whole from its first frame since
    #88), and frame 25 cuts to Brutus from behind.
  - `BF_TEST_GOTO=16.5,37,16.5,37 BF_TEST_DIE=0.7`, a cliff behind Brutus at the hand-over:
    - frame 71: the cut, from behind with his whole body in view;
    - from frame 72 the hook's autopilot (a `BF_TEST_GOTO` whose goal is its start) sets the
      camera heading to 0 every frame, which points it at the cliff, 1.8 m away;
    - the follow camera's clamp then pulled it in close to him (frames 73-89; before #126: its
      wall test now steps it in, and his body turns see-through). That comes from the hook
      and the walls, not the hand-over.
  - With `BF_DEATHMATCH=1` there is no death camera.
* `BF_COMBAT_LOG=1` prints hits (each with its factors: the hit part struck, the own side's
  x0.7, the difficulty step; play.md **Damage**), knockdowns and dives.
