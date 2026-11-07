*Part of [maps/](README.md), the former page `maps.md`.*

  * Steep ground with no floor under it (#264; the code: todo/264-steep-slopes/answer.md). The
    terrain is a height field, so a pit's wall or a cliff face steeper than 65° is still the
    ground where there's no floor below it (`Arena::slope_below`: terrain only, not an upright
    face, normal y under 0.05; a rock's steep side is #285's, below; nothing under it at all is a
    hole, #292's, below). Before #264 no floor under
    the feet put a character back where it was, as at the edge of the map: in the tutorial's lava
    yard Tex ran in place at the east pit's rim (z -26.5), whose north wall is 66° (normal y 0.40)
    down to the bed at ~10.5 m, under lava at 14.40 m. Now, as the game does it:
    * **The ledge grace.** A floor within 0.8 x the body's radius of its middle and no more than
      0.1 m under the feet still holds it (FUN_0012b950's overlap probe, a capsule of 0.8 x radius,
      FUN_00153780; the 0.1 m is the ground contact's window, FUN_00153410; the demo tests eight
      points round the middle and takes the lowest floor among them, as the game's ground contact
      is the lowest-lying, `Arena::floor_around`). That keeps a sliver of steep ground beside a
      floor from starting a fall and a landing every frame. (A grace that took floors a whole step
      down let a character walk down a bumpy face from patch to patch, the further the more often
      it was tested.)
    * **Followed in steps.** A grounded character's move each frame is followed along the ground
      in steps of at most 0.12 m (`ground_walk`), and a move down a face the same way
      (`face_walk`): where the floor stops carrying them (a ledge, a face, its foot) is found
      within that, not at the frame's end, so it doesn't hang on the frame rate. The demo steps
      once a frame where the game's physics steps at a fixed rate with substeps.
    * **A fall, not a slide.** Past that, stepping onto the face is a fall: JUMP_FALL (MT 21),
      the take-off height kept (FUN_0012b950; take08's Tex runs off the rim in MT 21). The game
      has no slide state on ground steeper than 65° (its slide runs on slide-flagged floors only).
    * **On the face** (`slide_on_face`, `face_move`). Once the feet reach it the character is on
      it (`Player::face`): the free fall goes on (vy -= 18 m/s² a second, x/z x 0.99 a 1/30 s
      frame, FUN_0012cd10), and each frame's move has its part into the face taken off, with no
      friction (the sweep's impulses along the contact normals only, FUN_00150f90 /
      FUN_00168950 / FUN_00151170). The velocity itself isn't changed (the game never feeds the
      sweep's result back), so from rest the speed along the face grows at g sin θ (16.4 m/s² at
      66°) and the descent at g sin² θ. The feet stay on its surface (no hover), it isn't pushed
      off it as a wall (`Arena::push_out_sliding` leaves out the terrain's steep triangles facing
      the same way), and there's no landing on it. Where it falls away faster than the move
      brought them down (a cliff past it) they leave it, falling on. Every fall's move is taken at
      the frame's mean speed (vy - g dt / 2), so a fall's path doesn't hang on the frame rate.
    * **A gully** (two steep faces with no floor between them) keeps a body falling in the demo:
      it's held on one face and pushed off the other. The game's sweep lands a wedged body (two
      contacts whose normals oppose, pushing down and barely moving: FUN_00153810's bit 5), which
      the demo doesn't do.
    * **At the bottom** they land as from any fall: the drop counted from where they took off
      (char+0x6e8, written once when the fall starts, FUN_0012b740), so a 20 m run down a 66°
      face costs what a 20 m free fall does (36 HP before the difficulty's step); into a liquid
      too, by the drop to its bed (FUN_0012b740 has no liquid check, #152).
    * **Splashes** going into a liquid off a face are as any other entry's: by each part's
      bounding radius, not the speed (#5, levels.md "What a liquid plays").
    * **The AI** (followers, enemies, a cutscene's walk; not one a fall, a slide or a knock is
      carrying) keeps to ground with a floor under its middle and to the frame's own test, as
      before #264, and stops at a drop ahead: the ground within 5 m under points 0.4, 0.8 and
      1.2 m ahead, none at the first or a change of more than 2.75 m at both the others
      (`ledge_ahead`, FUN_00115b00's numbers, its rays cast from 2.75 m over the feet so that a
      ramp rising ahead is met from above). One on a slide-flagged floor, or building up a slide
      there, isn't probed: the game's slide takes it down (e40's Shrikes at (-12.9, 56.7) slide as
      before). The game's AI has no such refusal (FUN_00115b00's verdict is thrown away by its only
      caller): its routes over the level's world points keep it off cliffs, and the demo's brains
      don't walk those yet, so this is the demo's stand-in. Without it m09_a's followers walked
      off the start platform down a 72° face, ~30 m into the liquid.
    * The game's step is 0.4 m on the ground and 0.1 m in the air (FUN_00152d20); the demo's
      `STEP_UP` stays 0.5 m.
    * Checked:
      * `BF_MAP=tutorial BF_TEST_GOTO=46.26,-25.92,45.9,-34 BF_SLIDE_LOG=1`: "tex steps onto a
        steep face at [46.2, 16.1, -26.5] (66 deg)" (take08: off the rim at (46.2, 15.2, -26.4)
        in MT 21), running clear of the face into the lava (levels.md, "What a liquid does to a
        character": with this goal he runs on over the bed and out of the pit's south side,
        burned to 4.6 HP; to take08's own stop at z -30.4 he dies there).
      * Starting on the pit wall (`BF_TEST_GOTO=46.2,-26.9,46.2,-26.9,15.5`: a start height with
        no floor under it stands on the steep face there) the slide is the same at 15 and 60 fps
        (`BF_CAPTURE_FPS=60`): the feet 0.40 m down after 0.27 s at 15 fps, 0.44 m after 0.25 s at
        60 (g sin² θ = 15.1 m/s², 0.47-0.51 m), with no gap between the feet and the face.
      * The frame rate (`BF_CAPTURE_FPS=15/30/60`, `BF_TEST_SPRINT=0.5`): e40's s3 route
        (71.3, 103.0 to 92.1, 91.0) takes off at y 55.5 / 55.5 / 55.6 and lands after 20.9 / 20.3 /
        21.0 m (24.2 / 22.6 / 24.6 HP); s2 (-55.9, 84.6 to -32.1, 88.0) after 19.4 / 17.6 / 17.8 m
        (19.9 / 15.2 / 15.8 HP). s2's 15 fps differs in its run up, before any steep ground: the
        sprint gets going and turns differently at 15 fps (1.5 s in: 3.2 m covered, yaw -98, against
        2.0 m and yaw -102 at 60), so it reaches the rim 1.2 m further along it. Started 2.4 m
        before the rim (-53.5, 84.95) it lands after 18.4 / 19.0 / 19.0 m (17.3 / 18.8 / 18.9 HP).
      * `BF_MAP=m09_a BF_TEST_GOTO=-177.12,-223.64,-175.7,-217.2 BF_SLIDE_LOG=1` and
        `BF_SCRIPT=m09_a_startup,0`: no squad member falls. The NPC ramp drives on sdm_e34
        (`BF_TEST_NPC=0.5,shrike soldier@dm,enemy,dummy,<x>,<z>`, `BF_TEST_NPC_MOTION=1.0,1,run,..`)
        go as on main; e34's followers reach Tex again.
