*Part of [maps/](README.md), the former page `maps.md`.*

* **HUD text** (`play_text.rs`) is drawn in the game's own body font, atlas h_e4e4d2f4 (as the
  menu draws it), its glyphs 1 atlas pixel apart over a black outline and shadow, as in the
  captures. Colours sampled from them: messages and prompts pale blue (186 210 249), ammo and
  NEW orange (254 165 104), an item that can't be used red. Capitals are ~11.5 units (14 for the ammo).

* **Movement constants from default.xbe** (see `decompiled/xbe/ghidra/README.md`, "Player
  movement"):
  * **Gravity** is 18 m/s² (0x3c0584): FUN_0012cd10 takes 18 x dt off the vertical speed each
    step while the falling flag is set (#270). Nothing else acts on the vertical speed: no drag,
    no top speed. In the air the horizontal speed keeps 0.99 of itself a step (0x3c058c).
  * **The step.** In single play the game steps the world once a frame with the frame's time
    (the main loop 0x68d20; network play steps a fixed 1/30 s), 30 frames a second on the
    console (`GAME_FRAME`). The #264 falls show it: their vertical speed drops 17.9, 18.2 and
    18.3 m/s a second of the game's clock (take02, take05, take03's watch logs), 0.47-0.70 a
    step, and their horizontal speed keeps 0.99 a step 30.5-31 times a second.
  * **The jump** launches at 5.8 m/s (FUN_0012b630, 0x3c0588). Each step moves by the speed it
    has before taking gravity off, and the first step after the launch takes none: the falling
    flag that gravity waits for is set by that step's own sweep, once it finds no ground
    (FUN_00153810). So the game's heights after n steps are 5.8 n dt - 9 dt² (n - 1)(n - 2), the
    parabola of a 6.7 m/s take-off, which the demo flies (`JUMP_TAKE_OFF`): apex 1.25 m, 0.74 s
    in the air. Before #270 it flew the bare 5.8 m/s: apex 0.93 m, 0.64 s, three frames short.
    * Checked against #126's take07 (Tex's two standing jumps on e34, its 20 Hz watch log):
      `BF_MAP=e34 BF_CHARACTER=3 BF_TEST_GOTO=-51.654,35.515,-51.654,35.515 BF_TEST_JUMP=2.0,4.0
      BF_MOVE_LOG=frame BF_CAPTURE_FPS=30 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05
      BF_NO_SQUAD=1`. Fitted alike (a parabola through the airborne rows): the demo's gravity
      18.00, take-off 6.70 m/s, apex 1.247 m, 0.744 s in the air; the take's 17.8 / 18.0, 6.46 /
      6.63 m/s, 1.18 / 1.22 m, 0.727 / 0.737 s. The press to the take-off: 0.23 s (the take's
      0.24-0.28 s once the pad's delay is taken off).
    * Not like the take: the landing. The demo stands again 0.23 s after touching down, the take
      after about 0.45 s (MT_JUMP_LAND rows): the landing clip's hold, not the fall.
  * **The falls** keep their code (18 m/s² from the speed they leave with): a 37 m `BF_TEST_FALL`
    lands after 2.03 s (sqrt(2 x 37 / 18)). The game's own step is an explicit Euler step (the
    move before gravity), whose heights trail the exact parabola by half a step (17 ms over
    37 m): left out.
    * Not like the game yet: leaving the ground. The game's grounded move follows the ground's
      plane and pushes down 0.1 x the step's move + 0.01 m (0x3c05d4 / 0x3c05d8); ground that
      falls away faster leaves the character in the air, with the falling flag on and gravity
      building the speed even before the fall state starts (FUN_0012b950's ledge grace). The demo
      snaps the feet onto any floor within 0.5 m (`STEP_UP`) each frame. So over the tutorial's
      pit rim the demo's Tex runs down the rim's 46-degree slope on his feet (2.3 m in 0.27 s)
      and drops the last 1.8 m, where #264's take03 has him fly off at 15.05 m and fall 3.98 m;
      off e40's plateau lip the demo's Brutus slides down the 50-degree slope, where take02's
      falls 0.9 s (2.3 damage). A follow-up: it changes every character's ground following
      (#264's `ground_walk`).
  * Test hooks: `BF_TEST_JUMP=<s>[,<s>...]` presses the jump at those times (with
    `BF_TEST_GOTO`); `BF_MOVE_LOG=frame` prints the controlled character every frame (the feet,
    the vertical speed, the height over the floor and the action) once the ground loop has set
    them.
  * Walking off a drop of more than 0.5 m is a fall.
  * Landing after a drop of more than 5 m costs 100 × ((drop − 5) / 25)² health, with no upper
    limit (FUN_0012b740; #157): 16 at 15 m, 100 at 30 m, 144 at 35 m, 164 at 37 m (m09_a's
    waste pit), before the difficulty step. So a long fall kills Brutus (105) and Tex (115) too;
    the demo stopped at 100 from 30 m before. A 10.6 m fall cost 4.9.
    * `BF_TEST_FALL=<m>` starts the controlled character that many metres above the floor it
      would stand on, falling from rest. Checked with `BF_MAP=sdm_e34
      BF_TEST_GOTO=5.47,35.95,5.47,35.95 BF_TEST_FALL=37 BF_SLIDE_LOG=1 BF_COMBAT_LOG=1`: "tex
      lands ... after a 37.0 m drop", "fall 163.84 x none (n 0) ... = 163.84" at 2.07 s, and
      Tex dies (frames 0-30 show the fall onto the start's slope). With
      `BF_DIFFICULTY=standard` and 35 m: "fall 144.00 x STANDARD (n -4) squad 0.60 ... = 86.40",
      28.6 HP left.
* **Sliding** works as the game does it, and only on collision triangles flagged as slide
  surfaces (the canyon walls).
  * Building up: a slide amount (0–1) grows while you move. On ground steeper than 50° it grows
    at up to 8 per second, at the full rate from 55°; flagged ground always counts at least half
    rate. At 1 the slide starts.
    * "Move" is the step's own ground move, more than 1 mm (FUN_0012c020's third argument, from
      FUN_0012c6c0 or FUN_0012cb40): standing there's none, as the idle motion has no ground
      speed and isn't moved by its clip's root. The demo moves a standing body by its idle clip's
      root sway (a centimetre or two), and pushes from squadmates and walls move it too; before
      #294 those counted, so a character left standing on flagged ground built a slide up and slid
      off, turned downhill, with no input: on sdm_e34's 36° face at (23, 10) Flint after 42 s and
      Tex after 83 s in real time (some 190 frames a second), Flint at 1.3 s at the capture's 15
      fps, where a frame's sway is longer (the same at 62b6cd9, before #285). Now a character
      standing on its feet or kneeling, not sliding nor knocked, has no move (play.rs
      `stands_still`).
  * Sliding: it carries on until the ground eases under 45°.
  * Speed: the slide velocity starts at (1 − normal y) × 8 downhill plus 5 m/s down, then
    builds by that push 3 times a second, up to 40 m/s. You move by it times the amount.
  * Ending: under an amount of 0.2 your own movement blends back in. A slide that's blocked (a
    wall, or flat flagged ground where it only presses down) ends.
  * What you see and hear: the slide clip (motion "slide", script Sc_w1_slide_idle), facing
    down the slope, the surface's slide sound, and the slide dust (effect type h_01513b0f, ALE
    effect slide_puff) at your feet.
  * Steep but not flagged: unflagged slopes up to 65° can be walked up.
