*Part of [squad/](README.md), the former page `squad.md`.*

* **Standing on the floor.** The floor is 1.05 m (`GROUND`) under each character's pelvis, but
  the four aren't the same height. So each is raised or lowered by how far the lowest
  skinned vertex of their idle pose is from it (`sole_lift`, up to 0.3 m either way):
  * Tex: raised 0.17 m (his soles went through the floor);
  * Brutus: lowered 0.07 m (floated);
  * Flint: about right (0.01 m);
  * Hawk: lowered 0.15 m (floated).

  That's for standing. A kneel's lowest point (the knee) isn't the standing soles', so each
  crouch clip has its own lift (`clip_lift`): `stand2crouch` and `crouch2stand` through their
  length (16 samples), and `rp_crouch_idle` and the crouch walks as the most over their cycle.
  The model is lifted by the weighted mix of what's playing, so a kneel or a crouch walk
  stands on the floor too. With the standing lift alone, Hawk and Flint knelt in the air. This
  covers the squad's kneels as well.
* **Scopes.** With a weapon that zooms, right mouse is the scope button (in, out; other weapons
  still aim while it's held). How many levels is the holder's skill's: one for Tex, Hawk and
  Brutus, two for Flint, who steps in, closer, out, with any gun that zooms; her second level is
  z + max(10, z + 5) (the L-Shot-75 10 then 25, the L-Shot-50 5 then 15), the code's rule, which
  the takes confirm (play.md, "The scope"). The zoom ramps linearly, 0.15 s a level, and the
  button waits out a short lockout after going in or out. Switching weapons or reloading drops
  out of the scope, and Tex's berserk keeps him out of it ([skills.md](../skills/texs-berserker.md)).
  `BF_TEST_SCOPE=<level>[,<from s>]` holds a level (from that time on), `BF_TEST_SCOPE_PRESS`
  presses the button (play.md). A scoped shot leaves from the eye along the crosshair's ray (from the
  muzzle, a little lower, it would land low through the zoom). In the scope the camera
  moves in by the scope's progress to 0.5 m behind a point 0.5 m above the character's root
  node, every character's (the follow camera's own scope rig, 0x128280; the types'
  `offset-snipe`, 0.001 0 0, isn't read: [camera.md](../camera.md), "The scope"), and the aim
  turns so the crosshair stays on its point as the view moves. The field of view narrows by the
  level's zoom (`h_efce7f77` on the weapon's definition: MK-ASLT 2, Kman Auto 2, Psionic Blast 2,
  Foley 356 Tact 3, L-Shot-50 5, L-Shot-75 10; 0 on the rest), the model is hidden, and mouse
  look slows by the same factor. Shots still go through the crosshair. While scoped, the
  character's own breathing loops: each character's `snipe-sound` (`h_12501487`, beside its
  radar-range), a 2D sound of the common bank (Brutus `h_01a51dd2` 2.9 s, Flint `h_15545426`
  0.7 s, Hawk `h_1ca2a830` 1.7 s, Tex `h_0ce9616b` 1.8 s), with 0.6 s between breaths.
  `BF_TEST_AIM=1` holds aim.

  Two kinds of scope view (#283, `src/bin/play_scope_view.rs`). Which one is the character's
  skill's (its +0xa8): with bit 0 set (Flint's, 0x2d5e40) her overlay, with it clear the frame
  (everyone else's, Tex's 8 too: nothing reads bit 3), so it follows her skill whatever gun she
  scopes with (#62). Both are HUD widgets the crosshair widget's update makes in code (0xc6030;
  they aren't in the HUD's layout data), under the rest of the HUD:

  | Who | View |
  |---|---|
  | Flint (the sniper) | Fills the screen, with her overlay (FUN_000b3520) added over it: a cyan-teal smear round the picture's corners and along its edges (`h_f5b6a9f8`) and an ellipse of teal and yellow-green dots near the edges (`h_f0d8a0a7`). It comes in over 0.17 s from the press, shrinking from 3x its size to its scoped 2x as it brightens, the same at both levels, and goes the same way out. Her aim is dead steady (she's a synthetic). |
  | The others | The view sits inside the game's scope frame `h_1807ec04`: a 256 × 128 quarter (rounded corner, tick marks) drawn 1:1 from the middle and mirrored four ways, the opening x 164-476, y 127-353 of the 640 × 480 screen. Outside the rim the picture shows at 40% of its brightness, on the rim's line at 20%, the tick marks at 76%. The frame is whole at the press; leaving the scope it grows 3x about the middle in 0.33 s, fading over the last half. |

  How they're drawn (the leads, `todo/283-scope-hud/leads.md` §2-3; every function in
  [code-map.md](../code-map/hud.md), "HUD"):
  * **The frame** (update FUN_000b4070, draw 0xb4240): scoped (char+0x7a8) with bit 0 clear, its
    scale is 1 at once; else it eases toward 3 at 6 a second, its alpha 1 - clamp(scale - 2, 0, 1).
    Four quads from the middle, the texture's first column and last row there, texture stage 0
    set to CLAMP (FUN_0006a610 on its ADDRESSU / ADDRESSV), so the texture's outer grey
    (153) carries on to the screen's edges. The sprite draw (0xba950) blends it in its mode 5,
    ZERO / ONE_MINUS_SRC_COLOR: the picture times 1 - the texture x the alpha.
  * **Flint's overlay** (update 0xb3700, draw FUN_000b3970): scoped, its scale eases toward 2,
    else 3, at the frame's rate and with its fade. Its target in the scope is (1 - the reticle
    controller's +0x74) + 1, and the controller clears +0x74 every frame (its +0x58,
    FUN_000c52f0), so 2. Each texture in four mirrored quads of 160 × 120 units x its own scale
    (floored) x the widget's, the texture's first column and last row at the picture's corners:
    the ring at a scale from 1.5 to 1.2, then the smear at 1.1 to 1.0, by how deep the scope's
    focus lies between the follow camera's near plane (0.1 m) and its far plane (the
    `clip-plane-far` of the follow camera archetype the level file names, h_e8eb1b7b:
    `Level::follow_camera`, `Game::follow_cameras`; 250 m on 35 levels, e34's and e40's among
    them, 100 on 12, up to 800 on m14_c and m14_d). The focus is the centre ray's point's depth
    (`Reticle::depth`), taken at the scope's first frame and followed at 1000 m a second
    (FUN_000702a0's cam+0x1f8): 11.3 m at e34's range spot, ring 1.49. Mode 2 adds them, ONE /
    ONE, their texels times the vertex colours, grey 30 and 50, times the alpha. Not drawn in a
    vision mode (the camera's +0xd8: Brutus's Vengar view).
  * The console blends in the frame buffer's values (sRGB); Bevy's UI blends in linear light,
    where the added colour comes out much fainter over a lit scene. So both are one pass over
    the main camera's picture (`src/scope_view.wgsl`), after the Vengar view's world pass and
    before the HUD, in those values. It goes with the HUD: not under `BF_NO_HUD`, nor while the
    HUD is hidden (a cutscene's letterbox, the map editor).
  * In a window wider than the game's 4:3 the sides go on as CLAMP addressing would carry them:
    the frame's grey to the window's edge, the overlay's outer texels past its quads (the demo's
    choice; the game's screen ends at 4:3).
  * Before #283 the frame's quarter was drawn 397 × 269 (55% too big) as a dark tint at the
    texture's alpha, blended in linear light (lighter than the game's multiply), and Flint had a
    cyan rim of the demo's own over the whole window.

  `BF_SCOPE_VIEW_LOG=1` prints both widgets' scale and fade, the focus and its share, and the
  ring's and smear's scales on each frame they change.

  Verified (`BF_CAPTURE_FPS=30`, `BF_MUTE=1 BF_BACKGROUND=1`):
  * The frame: Hawk with the Kman Auto at #60 take07's spot, `BF_MAP=e40 BF_CHARACTER=2
    BF_TEST_WEAPON="Kman Auto" BF_TEST_GOTO=0.29,166.87,-0.16,166.66 BF_CAMERA_PITCH=-0.02
    BF_NO_SQUAD=1 BF_TEST_SCOPE_PRESS=2.0,4.0`. Frame 90 beside take07 at 4.0 s: the rim, the
    rounded corners and the tick marks where the take has them. Against the same run with
    `BF_NO_HUD=1` (no frame), the picture comes out at 0.399 of itself outside the rim, 0.220 on
    its line, 0.764 on the ticks, 1.000 in the opening (the texture's 0.400, 0.216, 0.761, 1), the
    opening x 164.0-476.0 and y 127.3-352.7 units. The game's own surround (#60 take01, its first
    grow-out frame against the last whole one, the opening's own change taken out): 0.37-0.40. The
    frame is whole in the first scoped frame (frame 61; take01 26.267 s) and grows 1.2, 1.4, ...
    2.2 a frame on leaving (take01 31.28-31.47 s: 1.20, 1.40, 1.60, 1.82, 2.02, 2.22).
  * Flint's overlay: #126 take16's spot, `BF_MAP=e34 BF_CHARACTER=1 BF_TEST_WEAPON=L-Shot-75
    BF_TEST_GOTO=-52.96,30.96,-52.96,30.96 BF_TEST_SWEEP=164.4 BF_CAMERA_PITCH=-0.0199
    BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_TEST_SCOPE_PRESS=2.0,5.0,8.0`.
    Frames 105 and 195 beside take16 at 3.5 and 6.5 s: the smear at the corners and the ring's
    dots in the same places; at level 2, the scene near black, the left and top edges read
    (5, 47, 48) and (4, 49, 58) against the take's (7, 49, 50) and (6, 51, 57), the corners
    (2, 21, 37) against (2, 20, 30). In: full in the 5th frame from the scope's first (the
    take's 6th); out: the corners and edges fall as the take's do frame by frame (the take's left
    edge 49 -> 24 -> 9 green, the demo's 47 -> 20 -> 9). #62 take06's spot on e40
    (`BF_TEST_GOTO=-16.16,152.62,-16.65,152.53`, presses 2.5 and 4.0): the same overlay over a
    lit cliff (the scene itself is the demo's, lighter than the take's).
  * The far plane by level: e10's follow camera (h_f727cfb9) gives 100 m (`BF_SCOPE_VIEW_LOG`).

  The others' aim wanders while scoped: a slow drift of up to about 0.7° that moves the view
  and the shots together. In the scope everyone can still move, but only at a walk. The
  crosshair moves to the middle of the screen at the same size, 64 units (see play.md, **Ammo,
  reload and the crosshair**). The eye stays
  0.25 m off whatever is in front (by the collision), so a close wall can't be looked through.

  Each level gone up plays the character's in sound and leaving the scope its out sound. They're
  stored beside its snipe-sound: `h_00dd1fe0` (the type's +0x260, which the press plays,
  FUN_00124340) and `h_02c94d34` (+0x264, FUN_001249b0's), about 0.3 s each. Flint has her own
  pair (`h_1fd08ea5`, `h_03e0936d`); the others share `h_135ec2ce`, `h_f9ffc972`.
