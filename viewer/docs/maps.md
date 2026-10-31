# Playing on a map

`bf_play` now runs on Battle of Bulgar by default (`BF_MAP=<level>` picks another level,
`BF_MAP=flat` brings back the checkered test floor, `BF_START=<n>` picks a start point,
`BF_TEST_GOTO=x,z,x2,z2[,y]` starts the player at one point (on the floor below height y if
given, e.g. a roof; with no floor below it, on the steep face there if there's one, #264;
without y on the terrain, or where there's none on the highest floor, #292; kept on the levels
whose spawn-triggers place the squad, #292) and runs them toward another from 1 s on (`BF_TEST_GO_AT=<s>` sets off at
that time instead: #35 take08's dash after standing 4 s, skills.md), or stands still when both
points are the same;
`BF_FIND_STEEP=1` lists slide surfaces steeper than 50° and ledges with a drop of more than 6 m,
then exits; `BF_FIND_BODIES=1` lists the collision bodies (size, closed or open, twinned, how
many faces face into the body; blockers and hinters, a placed object's object-type, its faces by
the way they face) and exits, `BF_FIND_BODIES=b<n>` lists body n's triangles (#292);
`BF_FIND_HOLES=1` or `=<file>.ppm` scans the map for holes in its collision and for the hinters'
walls and exits (#292, "Holes" below); `BF_COLLISION_CHECK=1` counts the collision surfaces the
map uses (terrain blocks, blockers, hinters, placed objects' parts) that aren't the map's own
and exits (#338, "Which level's shapes" below); `BF_HELD_LOG=1` prints what stops the controlled
character's move (#292); `BF_INSIDE_LOG=1` prints a character in a rock and one taken out (#285);
`BF_SLIDE_LOG=1` prints slides, falls and landings; `BF_ALE_LOG=1` prints particle emission;
`BF_NO_IDLE_FX=1` leaves out the power-ups' effects; `BF_NO_PLACED_FX=1` the level's placed
effect objects, `BF_PLACED_FX_VARIANTS=1` starts the ones held out, `BF_LEVEL_DUMP=1` lists
them):

* **Spawning.** You start at the level's start point. Points 0–3 are one squad's spawn on the
  hill, 4–7 the other's, and 9 is in the base. A level whose spawn-triggers spawn the squad (the
  tutorial, e40) starts each member on the ground under its own spawn-trigger, and one that
  places the squad in the world (m09_a) at its placements, each with its template's weapons and
  ammo ([triggers.md](triggers.md), "Spawn-triggers", #102).
* **The level's triggers** run: detectors, routers, timers, calculators, spawn-triggers, the
  doors' anim-triggers, destroyers ([triggers.md](triggers.md), #220).
* **Collision.** It uses the game's own collision surfaces (`src/bf/collision.rs`,
  `src/arena.rs`). On sdm_e34 that's 43,444 triangles.
  * Where they come from: `objects-<level>.ipn` holds them in the Ipion physics engine's format.
    The terrain blocks, the 23 invisible blockers and every placed object's parts each name
    theirs. Door leaves have their own and stop blocking while open. A blocker is a wall to
    every character or, a hinter, to the AI's alone (#292, "Blockers and hinters" below), and to
    nothing else: grenades, ragdolls, pickups, shots and decals pass through (#298).
  * **Which level's shapes (#338).** A surface is found by the name its owner gives it, and the
    levels number their blockers and hinters alike. Of the 9057 surface names in the 53 level
    archives' tables 1967 are in more than one archive and 291 of those have more than one shape:
    `h_fcc5d473` is in 42 archives with 39 shapes, `h_ee99c575` in 38 with 37, `h_09910e17` in 37
    with 36, `h_1bcd1f11` in 36 with 35, `h_fee0807e` in 32 with 31, and e01's shape of each is no
    other archive's (34 of e01's 492 surfaces are shaped otherwise somewhere, and 46 of the other
    52 archives have at least one that e01 shapes otherwise). The game loads common's data and
    the level's and nothing of another level (FUN_000d61a0), so a blocker is the shape its own
    archive has. bf_play loaded e01's data (the first mission's, `BF_LEVEL`: for its sounds)
    before the map's, and the first surface of a name wins (`Game::add_objects`), so on every map
    but e01 a blocker or hinter named as one of e01's took e01's shape. `BF_COLLISION_CHECK` over
    the 53 maps found 774 blocker surfaces (2768 placed) and 184 hinter surfaces (184 placed); 290
    and 142 had a name of e01's, and 155 (on 37 maps) and 136 (on 24) took another shape than the
    map's own: 291 of the 958. The terrain blocks (5960 surfaces) and the placed objects' parts
    (3851, 771 of them named in e01 too) never did. Examples (surfaces, those with a name of
    e01's, those that took another shape): the tutorial's blockers and hinters 24, 11 (6 blockers
    that were the same, and the 5 hinters), 5; sdm_e34's 14, 9, 5; e34's 17, 5, 1; e13's 6, 5, 5;
    m03_c's 32, 20, 11; m09_a's 18, 11, 11; sdm_e40's 5, 2, 2.
    * Now the map's data is loaded first, right after common's, and e01's after it, to fill in
      what the map lacks (`load_map`, `load_default_level`; the extras of the test map come after
      that): the check finds none on all 53 maps, and every surface a map names has a shape in its
      own data. e01's own map is as it was.
    * What else the levels name alike is the same in both, but for six archives (levels.md,
      "Which level's data"): so the order changes nothing for archetypes, meshes, materials,
      object types, sounds, animations and scripts, and gives e16, e23, m03_a, m04_a, m07_b and
      m09_b their own material, effect type and animation sets.
    * `BF_COLLISION_CHECK=1` (bf_play): reads the map's own data and e01's again alone (common's
      with the map's, as bf_level loads it) and prints for the terrain blocks, the blockers, the
      hinters and the placed objects' parts: the surfaces and their uses, how many `BF_LEVEL`'s
      level (e01) also has under the name, how many the map has no shape for, and how many the
      loaded data gives another shape than the map's own (with the first names); then exits.
    * What changed on screen. sdm_e34's `h_fee0807e` blocker at (14.7, 99.8) was e01's 19.8 x
      12.0 x 22.3 m box and is the map's own 7.1 x 8.0 x 10.3 m one: Tex running north along
      x 14.7 from z 84 (`BF_MAP=sdm_e34 BF_TEST_GOTO=14.7,84,14.7,112 BF_HELD_LOG=1`) was held
      for the rest of the run at (13.60, 36.46, 92.71) by body 51, a blocker, and now walks on,
      held a moment by the new body 51's west face (x 11.89) at (11.59, 37.78, 102.33). The
      tutorial's five hinters (the AI's alone) span 9-38 m where e01's spanned 3-22 m: four of
      them lie round the lava yard's pits and walkways. Terrorist 30 of the yard's plate trigger
      (`BF_MAP=tutorial BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999 BF_BRAIN_LOG=1
      BF_COMBAT_LOG=1 BF_TEST_AIM_HOSTILE=14`) used to see Tex across the pit at 26.9 m and stop
      at 2.33 s, still on its way, and fire (it died at 15 s to the aim helper); now it reaches
      its home (62.1, -38.2) at 2.53 s and sees no one for the 40 s: the demo's sight rays stop at
      a hinter's band as at any blocker (`Arena::ray`; which blockers the game's own sight ray
      leaves out isn't traced, #298): the line from its home to Tex on the plate crosses body 64,
      h_ee99c575's band at (48.6, -28.1), twice. That agrees with todo/28-tutorial take08, where
      nobody fired on Tex on the plate (brain.md).
      The other runs compared, before against after (`BF_MUTE=1 BF_BACKGROUND=1`, 15 fps, the
      frames saved at every 3rd to 15th): e01's start (48 of 60 frames identical, the rest
      differing in at most 0.008% of the pixels by over 8 levels; two runs of one build differ
      so, and in their first frames when the load is slow) and its autopilot run (46 of 60, at
      most 0.002%), sdm_e34's autopilot run (60 of 60), the tutorial's marines (63 of 70, at most
      0.005%) and first patrol (34 of 40, at most 0.001%), e34's squad follow (29 of 60, at most
      0.023%) and its fight with the Shrike Hound (49 of 50): the same to the particles' noise,
      the largest difference 15 levels, and their logs the same line for line but the map's
      triangle count.
  * Material: every triangle carries one, a world-material id plus a slide flag.
  * Standing: characters stand on surfaces up to 65° (the game's ground limit), facing up by
    their winding, within a 0.5 m step; steeper surfaces are walls (#285: so is a surface facing
    down, under the body).
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
  * Rocks and boulders (#285; the code: FUN_00153810's sweep, [code-map.md](code-map.md)
    "Characters and motion"). Before #285 a character could run into a boulder and not get out.
    * The rocks: placed scenery models (game-objects; `BF_FIND_BODIES=1` lists every body of a
      map and exits), each a closed shell of 330-660 triangles (h_e729625c, h_0a4ac769,
      h_05736b8b, h_e117fe28, ...; down to 40 for the small h_09a6856d), not convex hulls. Like
      nearly everything in the data (all the terrain, every blocker, most placed objects) their
      triangles come twinned: each also wound the other way, as Ipion keeps a lone triangle as a
      flat ledge of its two sides, so of each pair one faces out of the rock and one into it. The
      terrain is a height field with no rocks of its own; there are no physics blocks among them.
    * Why a body got in: the wall test pushed the body's spheres out from the nearest point of
      each steep triangle where the move ended. A move that carried the body's middle past a face
      in one frame (a sprint, a dive, a knock: more than its 0.4 m radius at 15 fps) left it
      behind the face, where the push sent it further in; inside, the inner faces held it, and the
      twins of the rock's underside facing up into it were floors to stand on. A thin wall went
      the same way: sprinting slantwise at sdm_e34's barracks
      (`BF_TEST_GOTO=-70,-12,-90,8 BF_TEST_SPRINT=1.0`) took Tex through its wall into the
      building at (-75, -8).
    * Which (before, with `BF_INSIDE_LOG=1`: frames the body ended inside): sdm_e34's boulder
      h_e729625c at (0.1, 69.5) let Tex in sprinting (trapped from 1.7 s), diving (5 frames) and
      knocked into it (trapped); e34's h_09a6856d at (-59.9, 34.2) sprinting (trapped); the
      tutorial's h_e729625c at (54.0, 16.6) sprinting and diving, and its h_0a4ac769 at (56.7, 5.1)
      sprinting (trapped). A run (5 m/s) didn't get in, nor did a dodge. e40's low boulders
      (h_034df7ff, h_142e34fd, h_0f18f5e6) are walked onto, before and after.
    * **Swept, as the game does it.** The game moves a character's body (a vertical cylinder)
      along its move and stops it before it passes into a face: nothing lets it in, and nothing
      pushes it out (FUN_00153810, FUN_00152f20, FUN_00152d20). Now a character's move each frame
      is swept from where the last frame left it (`Arena::move_body`): a circle of BODY_RADIUS
      against each wall's part between STEP_UP over the feet and the head, seen from above. It
      stops at the first wall the move goes into (its direction into the contact under -0.01),
      however far the move goes, and the rest of the move slides along it (its part into the
      wall taken out, no friction: FUN_00150f90 / FUN_00168950 / FUN_00151170), up to the
      game's passes: 4 times for the character a player controls, 3 for the AI's (#292,
      FUN_0012cd10; it was 3 for every one). Each push of the squad apart is swept too, and a
      slide's move, at the height of the feet where it starts. What's left overlapping a wall
      after (a door shut on a body) is pushed out from the front, as before, by the body's radius
      at most, unless that puts the body in a rock. One in the air whose move across would end in a rock's solid (into
      a rock's slope, which isn't a wall, rising over its feet faster than it falls) stays where it
      was across and comes down on the slope. The game raises the cylinder's bottom 0.4 m on the
      ground and 0.1 m in the air for a steep triangle only and meets any other with the whole
      cylinder (FUN_00152d20); the demo cuts every wall at STEP_UP (0.5 m) over the feet, so a
      ceiling lower than that over the feet isn't met.
    * **Walls by their winding.** A triangle is met from its front only (the game leaves out one
      whose plane has the body's point behind it: the feet for a steep triangle, the middle for
      others), its front by the winding ((v1 - v0) x (v2 - v0), FUN_001516c0). A contact with a
      wall's face pushes along the face's normal across, as the game's does (FUN_00150a10), and
      one with its edge or corner away from it; the demo asks for the body's point to be 1 cm in
      front, and leaves out a face contact whose normal faces away from the body, so the twin of
      a slope facing into the ground never holds a body walking up it (a bump ahead can put the
      feet in front of it). A lone face (a few small hulls: sdm_e34's h_fc2033e6 and h_04c527a0,
      e34's h_f809f8de and h_fbad6fba) blocks only from its front. A wall is any triangle that
      isn't a floor by its winding: a steep face, and the underside of an overhang (a ceiling,
      normal y at or under cos 130), which the game meets with the whole cylinder.
      * A rock's faces that face into it (`Tri::inner`: of a twin pair in a closed body, the one
        with a point 5 mm in front of it inside the rock) are neither walls nor floors. The game
        never has a body inside a rock to meet them from their front; the demo met them: the twin
        of a rock's underside, facing up inside it within the step from the ground, was a floor
        that a body walking at the rock's foot stepped up onto, inside (#285's test: sdm_e34's
        boulder h_e729625c from the north, `BF_TEST_GOTO=0.10,65.50,0.10,77.50
        BF_TEST_SPRINT=1.0`, went in and out 57 times in 120 frames; e40's h_05736b8b at
        `113.78,68.74,134.64,76.34`; dives and sprints at sdm_e34's boulder), and from inside
        they held a body there. Of a pair that tests in both ways or neither (a doubled or thin
        part: one triangle of each of e40's and the tutorial's low boulders h_0f18f5e6 and
        h_142e34fd, the blockers round some objects) both are kept. On sdm_e34, e34, e40, the
        tutorial and m09_a, 6268, 12200, 14098, 8620 and 8632 triangles face in.
      * The twin facing down of a floor that's stood on (`Tri::floor_back`: the terrain's, a
        rock top's) is tested at the body's middle, as a ceiling's: the demo's rule. By the game's
        split that twin of a 50-65° slope is steep and tested at the feet, and where the slope eases
        ahead its plane is over the feet, so the sweep met it and held a body walking up to the
        junction (#285's review: a 60° slope easing to 53°, and a 55°-52°-45° flank, held 0.4 m
        short of the junction; `cargo test --lib up_a_twinned_slope`). A rock's underside keeps the
        game's split (its twin facing up is inside the rock).
      * A floor faces up by its winding, so a lone hull's underside holds no one, and the floor
        under the feet is the twin facing up, which carries the material and the slide flag
        (`Arena::ray_material`), as the game's ground is: its ground contact is the lowest contact
        whose normal is within 65° of up (FUN_00153410), and that contact's triangle header gives
        the material and the slide flag (FUN_0012dab0); the twin facing down has its normal down
        and its plane behind the body, so it's never one. Before, both twins were floors and a
        rounding tie picked either: the twin facing down (material 0, no slide flag) wherever its
        height came out higher (on e40's canyon rim at (-54.23, 84.80), by 4 µm). So on every
        level slides on slide-flagged ground start as the game's would, sooner than before (#285's
        test: `BF_MAP=sdm_e34 BF_TEST_GOTO=30,10,-30,10` slides at 2.47 s on a 36° face, where main
        slid at 3.00 s on a 57° one; e34's `5.47,35.95,30,60` at 2.93 s against 3.13 s; 26,878 of
        e34's 30,962 terrain triangles carry the flag).
    * **Climbing onto a rock.** The game's sweep moves the body up a slope with its move, so a
      rock overhanging the slope stops it. The demo moves level and raises the feet to the floor
      after, so a step up that would raise the body more than 5 cm into a wall isn't taken
      (`Arena::clear`, `CLIMB_SLACK`, in `ground_walk` and the ground loop: the sweep leaves a body
      touching a wall it walks along, and where the wall leans over the slope a rise of a few
      centimetres brings it inside the radius; with no slack each such step was refused, #285's
      test on sdm_e34's outcrop h_f5270370), and a body on its feet that the floor carried into a
      rock's solid (a dome rising faster than the step along one move) goes back where it stood.
    * **A rock's steep side.** A body with its middle over a rock's face too steep to stand on,
      more than a step over the floor under it (`Arena::flank_below`: a face of a placed object or
      a blocker facing out, normal y from 0.05 to cos 65), is on it as on terrain too steep to
      stand on (#264's `slide_on_face`; not one the AI steers): stepping onto it starts a fall,
      and it slides down the face to the floor below. Before, it fell straight through the rock
      to the floor under it and was taken out at the bottom, half a metre or more sideways at
      once (#285's test: 108 drops of 4 and 9 m onto the tops and sides of six rocks of sdm_e34,
      the tutorial, e40 and e34, 5 taken out; now none). The game's sweep meets the body with the
      face and moves it off along it. A rock's steep face is met from where the body's middle is
      in front of it as well as its feet (`WallSection::also`, the demo's), so the face over the
      step pushes a body on the flank off, and the face's steps (`face_walk`) follow a rock's
      flank as they do steep terrain.
    * **Out of a rock.** One in a rock's solid anyway (a spawn or a test teleport there, or a
      fall's drop past a rock's side that it slid off under the rock's widest part) is taken out
      at the start of the collision, before it's drawn, to the nearest place clear of it
      (`Arena::escape`: of 32 ways round, the one where the rock's surface is nearest at the
      heights of its body that are inside, then its radius beyond; its feet on the floor there,
      unless it's in the air). The game's sweep would slide a falling body off along the rock;
      here it's one step sideways.
      Inside is told by the rock's surface (`Arena::inside`): a body whose faces, a twin pair
      counted once, close round a solid (no edge left open; not a door's, not a liquid's) and that
      the vertical lines below and above the point each cross an odd number of times.
      `BF_INSIDE_LOG=1` logs a character ending a frame inside one and each one taken out.
    * The fall onto a rock's top is the air step's, which lands on the floor it was over (a 6 m
      and an 8 m drop onto sdm_e34's boulders land on their tops).
    * Test hooks for the moves (with `BF_TEST_GOTO`): `BF_TEST_DODGE=<s>[,<s>...]` presses the dodge
      at those times (to the right, standing), `BF_TEST_DIVE=<s>` dives toward the goal then (as
      from a grenade behind); `BF_TEST_SPRINT`, `BF_TEST_KNOCK` (thrown back) as before.
    * Checked (15 fps captures, `BF_INSIDE_LOG=1 BF_NO_SQUAD=1`):
      * From 6-7 m out sprinting at a rock's middle, or 3.4-4.2 m out dodging, diving or knocked
        at it (27 runs over sdm_e34, e34, e40 and the tutorial): before #285, 7 ended in a rock;
        now none, and Tex runs on the spot against the boulder (frames 26-60 of
        `BF_MAP=sdm_e34 BF_TEST_GOTO=0.10,76.50,0.10,69.50 BF_TEST_SPRINT=1.0`). Started on a
        rock's middle (`BF_TEST_GOTO=0.1,69.5,0.1,69.5`; the tutorial's 54.0,16.6; e34's
        -59.9,34.2) he's taken out on the first frame, beside it: 1.9, 2.4 and 1.2 m out.
      * #285's test's sets (its scripts, run again): 108 drops of 4 and 9 m onto the tops and
        sides of six rocks, none taken out (5 before this round); 67 sprints at 14 rocks, 16 at
        sdm_e34's boulder from the north, 46 at pairs and clusters of rocks, 64 dives at sdm_e34's
        boulder from 8 sides and 0.7-1.45 m clear, and 16 walks and sprints over open ground: none
        ends a frame inside or is taken out, but for a run that starts in a rock (before this
        round 1, 6, 2 and 11 of the sprints, the sprints from the north, the pairs and the dives
        ended a frame inside). Its 192 dives at three rocks, run on the build before the steep
        sides: none either (12 before).
      * The loops it found: sdm_e34's boulder from the north (`BF_TEST_GOTO=0.10,65.50,0.10,77.50
        BF_TEST_SPRINT=1.0`, and from 63.0) is climbed and left over its far side, to (0.16, 77.59)
        where main's run ends at (0.06, 77.54); e40's h_05736b8b (`113.78,68.74,134.64,76.34`) ends
        at (128.7, 79.6), main's at (127.9, 79.8); sdm_e34's outcrop h_f5270370
        (`29.23,44.02,38.53,47.40`) holds Tex at its crest, where main ran him through it (its
        frame 23 is inside the rock) to the goal.
      * The test tools' ridge: [testtools.md](testtools.md).
      * `cargo test --lib arena`: a 2 m twinned box holds a body out in one 5 m move and in 0.6 m
        steps, slides it along, takes one put inside out to its nearest side; its faces that face
        in are neither floors nor walls; a lone face holds from its front only; a low rock is
        stepped over; an overhang, one leaning 15° out and a ceiling at 1.2 m hold; a 55° twinned
        slope doesn't, nor one easing from 60° to 53° or from 55° to 52° to 45°, walked up from its
        foot.
    * Against main, the same runs (15 fps): the tutorial's base door identical, its second
      building within 0.06 m; sdm_e34's barracks door closes a frame later, Tex within 0.36 m.
      #264's checks: the tutorial's rim and pit wall identical; e40's s3 takes off and lands
      (20.9 m) as before, Tex's slide after it starting 0.13 s later and ending 0.04 m off, the
      Shrikes' slides within a frame; m09_a's squad doesn't fall (Flint steps down a 1.2 m ledge
      on the way), its Shrike Hound's slide starts a frame sooner and doesn't start again at 6.8
      s. e40's s2 steps onto the 66° face where it did but lands after 17.2 m, not 19.4 m: on the
      rim the floor is now the twin with the slide flag (above), so the slide starts a frame
      sooner and he leaves the face slower (#264's 30 and 60 fps runs landed after 17.6 and
      17.8 m). Sprinting slantwise at sdm_e34's barracks he's held at its wall (-75.1, -9.7),
      where main took him through it. The squad following Tex on e34, and past the boulders of
      sdm_e34 and e40 (`BF_TEST_GOTO=6,60,6,90`, `-5,-150,-5,-180`): within 1.22, 0.04 and
      0.43 m of main's (on e34 Tex ends this run sliding: slides start sooner, above). #285's
      test's 16 walks and sprints over open ground on e34, sdm_e34, e40, the tutorial and m09_a
      end where main's do (within 0.25 m) but where a slide on slide-flagged ground starts
      sooner: e34's sprint 2.4 m off, sdm_e34's flat run and sprint 1.5 and 3.4 m.
  * **Blockers and hinters (#292).** A level's `<blocker>` carries a `physics-id`, the PID_ kind
    of its body (its attribute handler FUN_00197e40 reads it through the PID_ names,
    FUN_00186320): 2768 of the 2952 blockers of the 54 levels are 4, PID_BLOCKER, and 184 on 28
    levels are 6, PID_HINTER. A character's sweep meets the world's blockers by a mask of kinds
    (FUN_001525f0) that its physics step builds (FUN_0012cd10, code map): the blockers always,
    the hinters only for a character no player controls (one whose brain has +0x3c4 bit 12, set
    while its route jumps between world points, FUN_00026520, passes them too). A hinter is a wall
    the AI keeps to and the player walks through. 181 of the data's 184 hinters are open bands
    of upright faces (no floor, flank or face looking down among them), none is closed
    (`BF_FIND_BODIES` over the 28 levels, #338: #292's survey said all of them, with e01's shapes
    where a hinter's name was e01's, "Which level's shapes" above); the three others, m02_b's
    body 89 (28 steep and 28 downward faces among its 72 triangles) and m09_a's body 39 and
    m09_x's body 49 (6 floors, 16 steep, 22 downward among 48), are shaped otherwise, and the
    demo's floor queries for a character (`Body::Character`) take their faces in as any blocker's.
    The bands run round ground the AI shouldn't leave: sdm_e13's 2.7 km
    band (body 71, 25 m tall) traces its arena's outer lava shore, m14_c's longest (body 49,
    375 m) the edge of its icy plateau.
    * The demo made every blocker a wall to everyone. Before #285 the push-out let a body run
      through a thin band; #285's sweep holds a body at it, so the hinters became walls to the
      player: on sdm_e13 (`BF_TEST_GOTO=90,-91,130,-91`) Tex runs down to the shore and is held
      at the band at (94.9, -102.0), sliding along it, on 018e360 (#285 merged) and on main
      051dce2; on 62b6cd9 (before #285) he went through it into the lava. On m14_c (the free
      camera's teleport to (59.1, 75.4), running to (64.0, 81.7)) main held him at (61.5, 78.4),
      short of the plateau's edge.
    * Now the sweep and the step-up test (`Arena::move_body`, `Arena::clear`) take who moves
      (`arena::Mover`, from `Player::controlled`): the hinters' walls (`Tri::hinter`) are left out
      for the controlled character and kept for the AI's (the wingmen, the enemies, another
      member's cutscene walk; the controlled character's own cutscene walk stays the player's,
      whether or not the game's hold clears the player slot's flag; the demo's AI walks no routes,
      so the hinters always hold it). The player's move is swept in 4 passes, the AI's in 3, as
      FUN_0012cd10 sets them. On sdm_e13 the band holds the wingmen at the outer shore as the
      game's AI is held; the demo's own shore stop for them (squad.md, "Liquids that burn") is
      unchanged.
    * Where (`BF_FIND_HOLES`: the hinters' walls on ground the player can walk to, in metres of
      floor, an upper bound; measured again after #338, which gave the maps their own hinters
      and blockers: the numbers before in brackets where they changed): e13 2613 (18), sdm_e13 2607 (1532), m14_a 1608 (400), m07_b 1338 (352), m02_a 1323
      (670), m02_c 1223 (426), m14_b 1065 (333), m07_a 973 (494), m04_c 963 (304), m02_b 665 (225),
      m09_a 567 (49), m14_c 523 (289), m09_x 512 (210), m09_b 489 (none), e05 366, sdm_m03 337
      (19), m04_a 308 (276), m03_b 293 (294), m03_a 246 (232), e01 212, the tutorial 198 (51), e16
      117 (31), e40 78 (16), sdm_e40 10 (11), m09_c 8 (14), m07_c none (1); m05_y's one hinter is on
      no such ground; sdm_m07's six (a level with no terrain) aren't scanned. The fix takes the
      kind, so it covers every one.
    * **Only a character meets them (#298).** Ipion's body-pair table at 0x3c01e8 (FUN_00169ab0:
      a byte per pair of PID kinds, 0 never collide, 1 always, 255 ask the pair's own check;
      code-map.md) pairs PID_BLOCKER (4) and PID_HINTER (6) with PID_CHARACTER_PHANTOM (10)
      alone, and the one world query that takes them in is a character's sweep (the weapons'
      leaves them out: play.md, "Bolts and their hits"). So the demo's other queries pass through
      every blocker and hinter (`arena::Body::Thing`): a grenade's flight and its bounces off the
      walls, a corpse's ragdoll bodies (the faces they meet and the floor under them, ragdoll.md;
      a living character knocked down keeps a character's, as the demo stands it up where its
      ragdoll's pelvis lies and the game's knockdown moves the character itself), the floors that pickups, lying guns,
      casings and memory chips settle and land on, a missile's and an instant shot's hit
      (`Arena::ray_normal`, `ray_material`, `ray_breakable`: a breakable object's own blocker no
      longer takes the shots meant for it, its collision does), the decals' surfaces and the
      blasts' ground. What a character's body meets stays as it was (`Body::Character`): its sweep
      (`Mover`), the floors it stands on, the spawns' places, the AI's ledge probe; so do the test
      tools' and the editor's spots, which put down characters and objects alike. The cameras'
      and the sight lines' rays
      (`Arena::ray`: the crosshair's, the AI's eyes, a blast's line to a body, the death camera)
      still stop at a blocker: which of them the game's filters leave out isn't traced.
      * Before, a grenade or a Roller that ended a frame within its radius of a blocker's wall was
        pushed out and bounced back (often not: at a grenade's speed a thin band is crossed between
        two frames), and a blocker's top was a floor to a grenade, a ragdoll, a pickup and a
        decal (sdm_e34's vehicles h_fc0ef12e have their blockers' tops 0.14 m over their roofs).
      * Checked: `cargo test --lib arena` (`a_blocker_meets_characters_alone`: a blocker's and a
        hinter's top, walls and rays against a thing and a character, and a plain object's box
        against both). sdm_e13's band at the shore (`-- --test BF_MAP=sdm_e13 BF_NO_SQUAD=1
        BF_NO_HUD=1 BF_TEST_GOTO=99.64,-92.92,115.84,-104.52 BF_TEST_GO_AT=100
        BF_TEST_GRENADE_TYPE=Roller BF_TEST_THROW=1.0,0.1 BF_GRENADE_LOG=1 BF_CAPTURE_FPS=120
        BF_TEST_FREECAM=0.5,104.82,35,-87.31,21.6,-24`): the Roller set down 3 m short of the
        band rolls down the shore through it and into the lava at (105.88, -97.90) at 3.75 s;
        with the walls met (the old rule, the same run) it was pushed out at the band at 2.68 s,
        (101.78, -94.96), and rolled back up the shore. No pickup or lying gun of the 53 levels
        starts over a blocker's floor (the floor under each is the same for a character and a
        thing), so where the levels lay them is unchanged. sdm_e34's squad run
        (`BF_TEST_GOTO=6,60,6,90`, 120 frames): the same frames as before, pixel for pixel.
  * **Holes (#292: the game's rule found, not applied; #296).** Where a character has nothing
    under it (no floor, no terrain too steep to stand on, no rock's flank: a hole in the level's
    collision, or past the terrain's edge), the demo puts it back where it stood, as at the edge
    of the map: an invisible wall round the hole, older than #285 (62b6cd9 the same).
    * sdm_e34's chasm, between the mountains east of the valley (x 44-70, z -5 to 16, the valley
      floor beside it at 29-32 m), is such a hole, with no blocker or trigger near it: running
      north from (56, 32) Tex stops at its rim, (55.95, 15.67), and sliding down the mountain's
      face west of it (`BF_TEST_GOTO=30,5,60,5`) he's held at its foot, (40.52, 8.53). The drawn
      terrain has the hole too: the background's colour shows through.
    * What the game does at a hole, read in its code: it has no edge. Its sweep meets nothing
      there and sets the falling flag (FUN_00153810, 0x1539de), and past the ledge grace
      (FUN_0012b950) the character falls. A living character under -500 m (0x3a5524) is out of
      its world: a game object's update (FUN_00157c40) refuses it and the character's
      (FUN_0011caa0) returns before its physics step, so it hangs there.
    * Not applied. #292's first round let a character fall through (a JUMP_FALL to a hang at
      -500 m, taken out); the test against it found the chasm's fall a flat field of the
      background's colour and then a hang with no way on, and new falls near the levels' starts:
      sdm_e34 about 10 m south-west of its start, at (-5.25, 44.93), m02_a's west wall, twice on
      m14_a and once on m14_b. Those holes look like terrain the demo doesn't load: 17 of
      sdm_e34's 64 terrain blocks name no collision surface at all (`<blocks h_051ac63e="">`),
      and its silo stands over a hole. #296 looks for it; until then the put-back stays for
      every character.
    * Where (`BF_FIND_HOLES`): every level with terrain but e10, e13, e23, m01_a, m01_b, m05_b,
      m05_y, m07_c, mp4, mp6, mp7, mp8, sdm_e10 and sdm_e13 has holes, nearly all of them the
      terrain's outer edge behind the mountains round the level (sdm_e34 908 rim metres, m09_c
      3122). The scan's upper bound (it climbs any slope a floor allows, as no slide holds it)
      counts most of each edge as reachable; in play the slides on the slide-flagged slopes keep
      the player off most of them.
  * Test hooks (#292):
    * `BF_HELD_LOG=1`: each frame the sweep stops the controlled character's move more than 1 cm
      short, where and the bodies that held it (`Arena::holding`: the terrain, a blocker, a
      hinter or object n, by `BF_FIND_BODIES`' numbers), and each time the ground loop puts it
      back (no floor, face or flank under it: the edge of the map or a hole; a step up into a
      wall, `Arena::clear`; a floor that carried it into a rock).
    * `BF_FIND_HOLES=1` or `=<file>.ppm`: the map a metre at a time over its terrain's extent
      (`find_holes`): floors, steep terrain, flanks and holes, the walls by what they are, what
      can be walked to from the start points, the spawn-triggers and the placed characters (an
      upper bound: a step up a floor of up to 2.15 m a metre, any drop, no wall across the way of
      a 0.25 m body, the slides not counted), then in runs the holes' rims and the hinters' walls
      on that ground; the picture: floors grey by height (darker where not reached), steep
      terrain red, flanks orange, holes black, the hinters' walls magenta, the blockers' blue, the
      objects' yellow, -z up, a pixel a metre.
    * `BF_FIND_BODIES=b<n>`: body n's triangles and the way each faces.
    * `BF_TEST_GOTO`'s start is kept on the 25 levels whose spawn-triggers place the squad at
      the level's start (m01_b, m02_a-c, m03_a-c, m04_c, m05_a-d, m05_x, m05_y, m07_b, m07_c,
      m08_a, m08_b, m09_b, m09_c, m09_x and m14_a-d): the spawn names the controlled character
      and brings it into the world where the hook put it (play_triggers.rs `spawn_copy`); it
      moved it to the spawn before, so the hook's start was lost there. The squad's other
      members still go to theirs. Recipes on those levels written before now start where their
      `BF_TEST_GOTO` says, not at the spawn: play.md's Rail CLVR runs on m14_a (the spawn had
      put Tex 10 m off), and levels.md's and skills.md's m03_a views, 80 m off, now written with
      the spawns they ran at (the same view, the body turned to the goal).
    * Without a height `BF_TEST_GOTO` starts on the terrain there, and now, where there's no
      terrain, on the highest floor; with one, on the floor below it, as before. m14_c's squad
      spawn at (-167.14, 87.95) stands on an object's floor at 40.1 m outside its terrain: there
      the feet came out at -1.05 m, under the floor (and fell, with #292's first round); now
      they're on the highest floor there, 56.1 m up, over the spawn's, and with `,41` on the
      spawn's.
  * Checked (15 fps captures, `BF_MUTE=1 BF_BACKGROUND=1`, `BF_NO_SQUAD=1` but where a squad is
    named, against main 051dce2's build; 62b6cd9's and 018e360's for when):
    * sdm_e13's band: `BF_MAP=sdm_e13 BF_TEST_GOTO=90,-91,130,-91 BF_TEST_HEALTH=5000`: Tex runs
      down the slope, through the band at (95, -102.3) and into the lava (feet 17.0 m at 4.5 s),
      on to (124.50, -95.15) at 11 s, where main slid him along it to (106.94, -87.51).
    * m14_c's plateau: `BF_MAP=m14_c BF_TEST_GOTO=59.09,75.35,64.01,81.65`: Tex crosses the band
      at (61.5, 78.5), runs off the plateau's edge at (63.5, 48.0, 81.0) and down its 75° cliff.
    * m14: 35 runs across the big hinters of m14_a, m14_b and m14_c, from 4 m on one side of a
      segment's middle to 4 m on the other, with the first round's build: 20 crossed (m14_a's
      bodies 37, 39 and 40, m14_b's 42, 43 and 44, m14_c's 44 and 49); of the rest 5 started over a
      hole (held there now), 6 were held by an object, a blocker or the terrain beside the band,
      and 4 went off another way down a slope. Main can't start those runs at the bands (the
      spawn moved Tex, above); its free camera's teleport run held him at m14_c's plateau edge
      (above).
    * sdm_e34's chasm, as main: from (56, 32) Tex is held at the rim, ending at (55.95, 15.42)
      (`BF_HELD_LOG`: "no floor, face or flank under it" from 4.53 s), and down the face west of it
      (`30,5,60,5`) at its foot, ending at (42.65, 7.16) as main's.
    * No change where nothing of this is met (the same runs on main's build and the first
      round's, whose fall none of them met, 150 frames, end in the same place with the same log
      lines): #285's rocks (sdm_e34's boulder from the north, (0.25, 71.63); Tex put in it, the
      tutorial's and e34's taken out to (1.43, 70.83), (51.81, 15.69) and (-61.10, 34.20); over
      the boulder to (0.17, 77.59); e40's
      h_05736b8b, (126.12, 80.86); the outcrop's crest, (31.53, 46.16); the barracks' wall,
      (-75.07, -9.82); the dive at the boulder), #264's slopes (the tutorial's rim and pit wall,
      e40's s3 and s2 landing after 20.9 and 18.4 m, m09_a's start with Flint's 1.2 m step and the
      Shrike Hound's slide), the slides (sdm_e34's at 2.47 s on 36°, e34's at 2.93 s), the
      liquids (#146's take01: 45, then 9, dead 0.54 s after the first part in; the tutorial's pit,
      five hits of 18; the wingmen waiting at sdm_e13's shore, 48 lines alike) and the squads
      following on e34 and e40. The player's fourth pass changed none of them.
    * sdm_e34's start pit (`BF_MAP=sdm_e34 BF_TEST_GOTO=5.47,35.95,-8,48`, from start point 0
      toward the hole south-west of it): held at its rim, sliding on and off there, ending at
      (-3.89, 42.50) as main's run does; m14_c's spawn (`-167.14,87.95,-167.14,87.95`) stands
      on the highest floor there, 56.1 m, and with `,41` on the spawn's, 40.1 m: neither falls
      since the put-back is back.
  * Not solid: grass, wires and other undrawn or cut-out parts have no collision surface, so
    they don't block. Nor does a spawn-trigger's template, which the game takes out of the world
    until it's spawned ([triggers.md](triggers.md), #220; nor is one drawn but a pickup, #230), nor a placed
    character (`character-object`): a character's body is its own physics object, not the
    level's collision (#230; m09_a's squad started on their own placements' bodies).
  * Fallback: a level without collision data uses the drawn triangles instead.
* **Doors and gates.** Their leaves are the parts of a placed object that its clips move: parts
  on sliding joints (joint type h_fd8f670c: sdm_e34's and the tutorial's doors and gates) and,
  since #261, parts on transform joints (h_f35d5b3c) and hinges (h_e94eb66a): m09's and m14's
  Shrike doors, m01's hatches and m01_a's gate, m05's and e06's doors, m05_y's hut, the 8-part
  doors of m08, e10, sdm_e10 and mp8, e10's ramp (Motion, below). A door moves
  only when one of the level's anim-triggers that lists it plays its clip (#220, #125:
  [triggers.md](triggers.md)): forward opens it, backward closes it (the opening played back), at
  the entry's time-scale; an open door stops blocking. There's no door flag: a door nothing
  signals stays shut. What signals them, from the data:
  * sdm_e34's eight doors: their plate (a box, `size` its half-extents: e.g. h_f214c88d, +-4 x 2
    x 2 round the doorway) admits any character; its ENTER_DETECTOR_FIRST, through a router,
    opens the door, its EMPTIED_DETECTOR closes it once the last one has been off it 0.2 s. The
    5 m circle the demo used before is gone, and an enemy on a campaign map no longer opens a
    door its plate doesn't admit (#125's e34 gate).
  * the tutorial's two building doors: their plates (squad types), once: h_eefcb023 opens the
    base's h_06537687, h_020415dc the second building's h_0f544611 27 m away; the door
    h_fa29ecfd opens by itself at the level's start (FIRST_UPDATE); the gate h_14690535 from the
    end of its panel's sound (its sound-trigger's FINISHED, #223: [triggers.md](triggers.md),
    "Sound- and dialog-triggers").
  * gates with wall panels (below).
  * the campaign's other doors: the door survey (todo/door-survey/survey.md) traces every
    one's chain.
  * Motion: the level's animations-<level>.xmb has an archetype-set named after each door's
    archetype, with one target per leaf (by part name) naming a data channel. The game plays a
    channel only on a joint of its kind (FUN_002340f0), sets the joint's value from it
    (FUN_00234dc0) and places the part by the joint's type (FUN_00234f90; the rows are in
    [code-map.md](code-map.md), "Levels and triggers"). The demo does the same for three kinds:
    * A sliding joint takes a float channel (data type h_e0b5650b): metres along the joint's
      axis per frame at 24 fps. sdm_e34's doors take 1.21 s (upper leaf 1.89 m, lower 1.70 m);
      the gate takes 4.96 s (3.92 m). A sliding leaf without a channel slides its own length,
      eased (none in the data).
    * A transform joint takes a transform channel (h_06901441, `DT_XFORM`; each one in the data
      is compressed, 18 bytes a frame: a translation and a packed rotation; sampled, or keyed
      with a time before each frame): the part sits at its joint's child-point plus the
      channel's translation, turned by the channel's rotation after the joint's ORIENTATION.
      Between frames the translation goes straight and the rotation by a normalised blend
      (FUN_00233b10). A Shrike door's eight segments (h_e509629f, 43 keys each) turn 60° and
      retract about 6 m into the ring over 3.29 s; its other kind (h_0fb9cdec) rises 6 m over
      3.97 s; m08's 8-part doors (h_01ea42aa) roll up, each slat rising to stack under the
      lintel, over 3.79 s; m05's and e06's palisade gates (h_f3e957e1) rise 5.86 m over 3.97 s;
      m05_y's hut doors (h_0d8aa8f7) swing out 80° on their hinges in 0.8 s (a translation and a
      turn together); m01_a's gate (h_f8f66fa8) slides its two leaves 2.5 m along its own z over
      4.96 s, under the craft-like structure it's part of; m01's hatches (h_fda9208e) turn their
      ring 11.4° in 1.47 s.
    * A hinge takes an angle channel (h_fed99c09, radians; between keys the short way round,
      0x292230), clamped to the joint's limits (`<h_fab3703e min max>`): the part turns about
      the joint's axis with its child-point on its parent-point. e10's ramp h_f3d6c977 (type
      h_1e50b5e0) lowers from the hull to the ground, 16° (0.279 rad) over 1.47 s.
    * Before its clip first plays, a leaf sits at its joint's rest, as the level draws it (a new
      joint's value is zero, FUN_00235c00); played back to the start, it's at the clip's first
      frame (a Shrike segment's is within 5 cm and 1° of the rest).
    * Which objects are doors: one with a sliding part, as before. One whose parts on transform
      joints or hinges have channels of their kind is a door when an anim-trigger of its level
      plays it once (AM_FORWARD or AM_BACKWARD, with AC_PLAY or AC_RESUME:
      `level_scene::played_once`). Those the level only loops (FORWARD_LOOP: the fans and other
      props of e01, e16, e23, m03_a and the tutorial, m08_b's dish), which the runtime would play
      once (triggers.md "Not done"), stay as placed, as they did. So does m09_b's AA gun
      (h_0849d1d8): its gun hangs from its turning mount, and the demo places a compound's parts
      one level deep. The other objects played once move as doors do and stop blocking past 0.6
      of their clip: m07_c's dome, e16's prop h_e1dc5844, m08_b's special-use panel (never
      played: #145).
    * Not done: a door's clips run on one clock, at the time-scale of the anim entry that played
      last. m01_a's gate has two anim-triggers off the same plate playing its leaves at x5.5 and
      x1; the demo opens both at x1 (the one that runs second), where the game plays each clip
      on its own. And a moving part's collision doesn't move with it: a door's leaves stop
      blocking past 0.6 of its clip, the transform and hinge doors' as the sliding ones'.
  * Sound: a door's anim-trigger sends ANIM_START (31) as it starts opening and ANIM_BACK_START
    (49) as it starts closing to sound-triggers beside it, which play them (#223,
    [triggers.md](triggers.md)): on sdm_e34 h_1edc357e (0.93 s) opening, metal_gate_05_close
    (h_f25b1597, 1.47 s) closing, each played while the player is within its `max-distance`: 7 m
    for three of the eight doors (h_196659f4, h_10616962, h_0b6838d8; full within the min distance,
    3 m, then quieter along the rolloff curve to nothing at 7), 15 m for the other five (full
    within 7 m), and
    the gate's metal_door_5 (h_e1b1f167, 8.5 s) within its 15 m (7 m full). These waves are streamed (resource-type
    1) from the language wave bank `ml-sounds/en/<level>-en.tgz` at the offsets in the level's
    sound bank. (Before #223 the demo played the sound-trigger nearest the door itself.)
  * The gate's two leaves have a clip each in its set (h_10d3fde3 and h_0a48ec76, 4.96 s, one
    target each); each leaf takes its channel from whichever clip names its part (a transform or
    hinge part: the first whose channel is of its joint's kind).
  * Verified (#261; `--test` for the free camera, `BF_NO_SQUAD=1 BF_NO_HUD=1 BF_DOOR_LOG=1`,
    each door's anim-trigger fired at 2 s with `BF_TRIGGER_FIRE=<trigger>,<signal>,2`, 15 fps):
    * `BF_DOOR_LOG` on m09_b, m09_c, m08_a, m01_a, m05_a, m05_y, e06 and e10 lists every door of
      the survey's tables for them, with "n on transform joints, m on hinges"; m09_b's AA gun
      and the looping props aren't doors.
    * m09_c's Shrike door h_1ced0204 (`BF_MAP=m09_c BF_TEST_GOTO=2,-271.6,2,-271.6
      BF_TEST_FREECAM=0.3,0,12,-271.6,-90,-3 BF_TRIGGER_FIRE=h_faf72c55,0,2`): open from 2.00 to
      5.27 s (its 3.29 s clip); frames 29-52 show the iris opening from its centre, its segments
      turning as they retract into the ring, a little further each frame, the tunnel behind
      clear from frame 62. Its other kind, h_e3b56e9c (trigger h_19088a3c, the camera at
      144.44,47.60,194.55,124,4.1), rises from 2.00 to 5.93 s. Closing: m09_c's h_f4e541da,
      opened at x10 by the level's start, closes backward at x2 on its USER_SIGNAL1
      (`BF_TRIGGER_FIRE=h_17ed0d14,11,3`, the camera at -62.90,11.97,-259.19,134.8,-5.7) from
      3.07 to 4.67 s, its segments closing in from the ring (frames 52-70), back on its clip's
      first frame.
    * m08_a's 8-part door h_027fe272 (trigger h_f2bc037b, x2; the camera at
      -62.33,26.71,11.24,75.8,-2.6): rolls up from 2.00 to 3.87 s, its 3.79 s clip at x2.
    * e10's ramp h_f3d6c977 (trigger h_120ddb37; the camera at 76,24,18,80,-10): lowers from
      the hull to the ground between 2.00 and 3.40 s.
    * m01_a's hatch h_06b98a8e (h_f62e3e2c; a round mosaic door in a circular hut's wall, seen at 80.3,108,-16.6,0,-3, the #261 tester's camera) slides about 2.9 m along the wall in 1.47 s;
      its gate h_fa445160 (h_1267e76e and h_15c8b4ad, signal 50; from under it at
      59.5,133,-49.5,105,-12) slides its leaves apart; m05_y's hut h_f0ce0274 (h_ff133988)
      swings its doors open in 0.8 s; m05_a's gate h_fcb796db (h_0f77a93c) rises 5.86 m.
    * Regression, run with the build before #261 and after: sdm_e34's door walk-through and gate
      panel and the tutorial's gate panel and base door (the commands in
      [triggers.md](triggers.md), "Verified") log the same door lines, and their frames are the
      same but for the world streaming in (frames 1-6) and a few pixels round the walking
      character that also differ between two runs of the same build. The door lists (`BF_DOOR_LOG`,
      before and after, all 53 levels): every door listed before is listed after, in the same
      order; the tutorial, the squad deathmatch maps and mp1-mp7 are the same; the campaign gains
      the survey's 69 doors on transform joints and hinges and e16's prop, m07_c's dome and
      m08_b's panel; sdm_e10 gains e10's two 8-part doors and mp8 its eight (opened at its start:
      FIRST_UPDATE, x1 and x100). m07_c's dome moves by the log (2.13 m over 6.17 s), but no
      camera spot tried showed it.
* **Gates and their wall panels.** A gate with wall panels opens from them:
  * The panels are `world-button-object`s (h_16f22d4b, type h_ee4c83c9, `reticule-action` 1).
    Used, a panel sends its signal (h_eaf8a35b: 14, USE_OBJECT) and data (h_f1660a0d) to the
    triggers listening to it (0x163060; the trigger runtime, [triggers.md](triggers.md)).
  * A `router-trigger` lists the panels among its objects (h_0b6b92ba) and passes their signal
    on (h_e91cf6a8) to the gate's `anim-trigger`s. Those animate the gate object (their own
    h_0b6b92ba) with one clip per leaf on signal 14, once only (h_f136d22d 1, where the
    proximity doors' signal 50 / 42 entries say 2147483647), and send on to the gate's sound
    trigger. So a used gate opens and stays open. On sdm_e34: panels h_14643319 / h_1d63038f ->
    router h_1c61561d -> anim-triggers h_f993d7ef / h_e8ad85ec -> gate h_14690535. The
    tutorial's panels go through their sound-triggers instead: the panel's terminal_03 plays
    (1.04 s), and its end (FINISHED) opens the gate's two leaves (2.0 s at time-scale 2.5) and
    plays the gate's grind (#223, take11).
  * Using one (capture todo/gate button.mp4): from in front of its button (hardpoint
    h_10f81d34, facing the model's +z), within 2.5 m and looking within ~45° of it, hold E for
    0.5 s. Reach, angle and hold time are guesses; the capture shows no progress bar and no
    gate message. While it's usable, "Hold [X] to activate panel." shows under the health bar:
    the game's use prompt, "Hold " + the X button's glyph + "to activate %s." with "panel", a
    glyph line drawn as the hint lines are, at the first one's place ([hints.md](hints.md), "The
    use prompt"; take11 matches it to a unit). E is the demo's X. The blue target ring (the four-notch reticle texture
    h_1d2a68d0, tinted) sits on the button. The prompt goes once the panel is used, or once the
    doors its signal reaches unchanged (`Level::button_opens`: sdm_e34's gate) are open (the
    demo's choice: the game's panel stays usable until a property trigger says otherwise).
  * The button glows green: material h_f87dca06 is shader type h_1df292a1 (851 materials), a
    lit shader whose glow texture h_e01baa40 is tinted by its constant of the same name
    (0.276 0.914 0.220 here) and added as light.
  * `BF_DOOR_LOG=1` lists the doors and the panels (what each opens unchanged; a door on
    transform joints or hinges with how many leaves are on each), and logs each door the
    triggers open or close and its leaves each second: how far each sliding leaf has slid, or,
    for a door on transform joints or hinges, how far each leaf is from its closed place (m) and
    how far it has turned (degrees). `BF_TEST_USE=<s>` holds use from that time for a second.
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
* **Music.** The game streams `data/sounds/<level>.xwb` by level name (sdm_e34's is
  caspian_action20, 59.7 s, stereo Xbox ADPCM, 22050 Hz). A bank can hold a music track and an
  ambience bed; the level's sound bank gives each its Type (`h_f92fb2b9` music, `ambient`; for
  names it doesn't define, an `amb` prefix is ambience). The music loops at half volume with the
  bed under it at 80% of that; a bank without music plays all its beds. Before, the first wave
  played, which on sdm_e13 and sdm_m03 was the bed, so their music was missing:

  | Map | Music | Ambience |
  |---|---|---|
  | sdm_e01 | caspian_action1 | |
  | sdm_e10 | ferix_action8 | amb_ferix_02 |
  | sdm_e13 | tmp_full-on1a-f | amb_singe_02 |
  | sdm_e34 | caspian_action20 | |
  | sdm_e40 | creepy_estuary_mutant_01 | light_bird_bed_1 |
  | sdm_m03 | caspian_action5 | amb_casp_04 |
  | sdm_m07 | | amb_shad_int2, ambiencehellish-2ch |

  `BF_NO_MUSIC=1` turns it off, `BF_MUSIC_VOLUME=<0-1>` sets the level, and
  `BF_DUMP_MUSIC=<file.wav>` writes the track and exits.

  **Since #223 the level's sound-triggers play it.** The bank's tracks are sounds of the level's
  sound bank (resource-type 2; a track's name hashes to its sound id: tutorial_01 is
  h_f5862d3f), and every level with a bank has sound-triggers that play them, most at
  FIRST_UPDATE (sdm_e34's h_1844a2b7: caspian_action20, looping, a 1 s fade-in), some as cues
  (the tutorial's: tutorial_01 from the start, stopped by a 168 s timer with a 3 s fade-out;
  caspian_tut_mellow1 in the mine field for 72 s; tutorial_02 when the gate has opened, for 238
  s; caspian_victory1 when the base is cleared). For such a level (`play_sound_triggers.rs`
  `plays_music`) `setup` plays nothing itself: the triggers do, at the music volume times the
  sound's own (the trigger's `volume`, else its library entry's: 0.8 for tutorial_01), with the
  trigger's fades ([triggers.md](triggers.md), "Sound- and dialog-triggers"). **One music
  track at a time** (#254, the game's 0x17eff0): a track (Type h_f92fb2b9) that starts while
  another still plays fades that one out over its fade-out as it fades in, and its own trigger
  loses hold of it, so its stop timer does nothing. In a fast tutorial run (the mine field
  before 168 s) mellow plays on past its 72 s until tutorial_02 replaces it at the gate, and
  tutorial_02 then past its 238 s until the victory sting replaces it. That is the game's own
  behaviour as its code reads; no take shows it (triggers.md, "One music voice"). The table above
  is the bank's mix the test tools' music picker still plays. Deathmatch still plays no
  music-bank track. The ambience beds are the levels' outdoor and indoor sound-triggers (Types 2
  and 3): an outdoor bed (sdm_e40's light_bird_bed_1, sdm_e34's wind, m01_a's amb_ferix_06) is
  full while the player stands outdoors and down at its `volume` (15-35%) indoors, an indoor one
  (m09_c's amb_shrike_int_01) the other way (triggers.md, "Types 2-5").
* **Shots.** Each weapon's bullet definition names its own effects, and they're played from
  the effect library: `effect-name` is the shot in flight, h_08d28037 is where it lands.
  | Weapon | Speed | In flight | Hit |
  |---|---|---|---|
  | LZR-23 laser | 150 m/s | laser_bolt_s + laser_bolt_rifle_s, a glowing bolt | laserhit_s, laserhitb_* |
  | Laser pistol | 100 m/s | laser_bolt_s | laser hits |
  | Brutus's cutter | 120 m/s | cuttertrail_ribbon, a bluish ribbon trailing the shot, and cutter_flare | — |
  | Minigun, ballistic guns (bullet-type 4) | instant | `tracer`, one particle shot from the muzzle at 113.6 m/s along the shot, on every (h_eeb9e75a + 1)th shot (MK every 3rd, Minigun every 4th) | sminigun_hit |
  | L-Shot | instant | tracer_snipe, a white beam drawn 200 m down the line in 0.38 s, smoke, rings | sminigun_hit |

  * Bolts carry their effect with them: the emitters flagged "attached" (h_e2999ffd, the
    demo's reading) and, since #57, every emitter of a bolt's flight effect but its trails
    have their particles move with the shot and go with it; a trail (an emitter under the
    library's world node h_ee223b51 with its appearance at the root) leaves its particles in
    the world. See play.md, **Bolts and their hits**.
  * Hit effects play on world hits with the struck material's debris, hit sound and the
    effect's light, each turned by its alignment-type; on bodies the flesh row's effect plays
    (made by hand for the squad's ballistic, cutter and laser rows, from the data for the rest)
    with the blood (play_fx).
  * World hits also leave the bullet's h_06a27365 hole or scorch on the collision triangle they
    meet, and each gun has its own muzzle effect and casings; the fire timing, pellets and
    accuracy spread are in [play.md](play.md), "Shots".
  * A shot's damage (friendly fire) lands when it reaches the body: an instant ray's at the
    distance / the demo's 400 m/s, a bolt's on the sweep that finds it, k x 0.1 s later
    (play.md, **Bolts and their hits**; its
    factors: the hit part struck, x0.7 on the own side, the difficulty step, none on the
    multiplayer maps as read live in a match, STANDARD on the missions: play.md **Damage**).
  * Beam appearances (class h_1f55f13e) are ribbons through their emitter's live particles in
    birth order, facing the camera. They have their own colour, alpha and width parameters, and
    their texture runs along v.
  * A frame's new particles are spread over the frame: along the emitter's animated offset
    (the sniper beam's sweep) and the effect's own movement (trails behind fast shots).
  * Effect libraries refer to their nodes by the case-kept name hash (e.g.
    "laserhit_Cone.emt"). Matching that recovered the emitters of 262 effect parts, the laser
    hits among them.
  * Test hooks:
    * `BF_TEST_FIRE=1` (or `2` for the second weapon) makes a `BF_TEST_GOTO` player aim and
      fire.
    * `BF_TEST_TARGET=1` stands the first squadmate still, 8 m in the line of fire.
    * `BF_CAPTURE_FPS=<n>` sets the capture's fixed step (default 15).
    * `bf_level` with `BF_FX_TEST=<effect name>` (and `BF_FX_MOVE=<m/s>`) runs one effect in
      front of the view.

* **Look.** The game lights each surface `texture × (ambient + light₁·N·L + light₂·N·L)` with
  the level's own light colours, in gamma space, with no tone mapping. The demo does the same
  (`level_scene::spawn_lighting` and `console_look`):
  * Lights: the level's outdoor pair of directional lights (the brightest casting the
    characters' shadow; the scenery and the terrain cast none) and its object ambient, given as
    sRGB colours; scenery placed indoors takes the indoor pair instead ([levels.md](levels.md),
    "Light objects", #143).
  * Camera: tone mapping off, and an exposure where a light of 1.0 leaves a texture as it is.
  * Terrain: tinted by the hue of the lights marked h_e02aba1c (a bluish 0.44, 0.57, 0.65 on
    sdm_e34), read as the terrain's own before #143; the game's code has them as the indoor
    pair, and what lights its terrain is #181's.

  Before this, an invented 9000 lux sun, a strong ambient and Bevy's default tone mapper (which
  desaturates) made everything pale. Point lights (muzzle flash, hit glows, explosions) are
  scaled to the new exposure by `POINT_LIGHT_SCALE`.
  * Textures (#257): every world texture (terrain, scenery, sky, liquids, characters, weapons,
    effects, decals) is drawn with the mip levels its archive ships, linear within and between
    levels and with no anisotropy, as the console samples them; before, each was its top level
    alone, so the ground sparkled past a few metres and showed raw texels close up. The HUD and
    the menus are unchanged. The terrain also mixes in its detail texture near the camera: the
    same texture mostly 3-5 times finer (0-16 over the data), fading out mostly 15-25 m away
    (0-35). Both are in
    [levels.md](levels.md) (Terrain layers), with their hooks (`BF_NO_DETAIL=1`,
    `BF_ANISOTROPY=<1-16>`).
* **Power-ups.** Each power-up type's idle event (`<events><event state="1" h_ed0c9fac=…>`)
  names an effect type. Its objecttypes `<effect>` entry lists the ALE effects:
  | Power-up | Effect | What it draws |
  |---|---|---|
  | Brute (quad) | multi_quad_icon | godray.tga rays |
  | Force (shield) | multi_shield_icon | brokenring.tga ring, plus the animated texture "arcb" (4 × 4 frames at 30 fps) |
  | Health | multi_health_icon | red-flare.tga |
  | Stamina | multi_stamina_icon | blue-flare.tga |

  `src/ale_fx.rs` runs them from the effect library's own parameters (see that file).
  * These are "perp" particles: flat in the emitter's frame. The shield's arcs emitter is turned
    −87.6° about x to stand its arcs up.
  * Each particle keeps the appearance's turn at the moment it's born (about 220°/s), so the
    particles fan out round the icon.
  * **Not the game's look** (#15): the icons' appearances are GeneralApp_FacingType 2 (square to
    the particle's motion) with AngleFadeOut, on sphere emitters that throw their particles out
    from the middle (0.025 m/s, 0.17-0.21 m out): the game's flares face out every way and fade
    near edge-on, a glowing ball (#35 take20 at 2.0 s: an oval glow ~1 m across), where the
    demo's lie flat, a disc thin from a low camera. The facing types are #286's.
  * Walking over one takes it ([pickups.md](pickups.md), "Power-ups", #15).
* **ALE blends (#228).** An appearance's blend pair (h_1daea8c0) is the PC's D3DBLEND (src,
  dest), which the game turns into the console's through the table at 0x4036b4 (0x2a6f00): 5,2
  (SRCALPHA, ONE) on 518 appearances, 5,6 (SRCALPHA, INVSRCALPHA) on 207, 3,2 (SRCCOLOR, ONE) on
  33 (the lava's eruptions slava-erupt*, its wading rings and splashes env_sl_*, the teleport and
  ion beams), 2,2 (ONE, ONE) on 13 (slava-fuzz, the laser and flesh hits, campfire sparks), and
  a few 3,6 / 6,6 / 2,6 / 4,6. Every pair with a dest of ONE adds the particle to the picture
  (before #228 only 5,2 did: the others were alpha blends, and the tutorial's lava bursts came
  out as brown balls). The particle's colour is the texel x its vertex colour and its alpha the
  texel's x its vertex alpha, side by side (pixel shader 0x3b89e0, which FUN_0029f5d0 sets for
  any pair whose dest isn't INVSRCALPHA; the vertex colour is the appearance's colour with its
  alpha curve in the top byte, FUN_002a1a50, and the particle vertex shader 0x3b8bf8 passes the
  colour through): so the alpha curve fades a SRCALPHA particle only. With ONE the particle adds
  texel x colour whatever its alpha, with SRCCOLOR (texel x colour) squared: drawn here with the
  alpha at 1, the texture's alpha opaque and, for SRCCOLOR, the texels and the colour squared
  (`ale_fx.rs` `Source`). A beam's ribbon keeps its colour x alpha whatever its pair (its own
  draw isn't read).
* **Effects the triggers make** (the tutorial's lava bursts, its farcast flashes) are compiled
  with the game's particle rules, the grenades' and the Therm Sweeper's (`load_recorded_facing`:
  the shapes, the count once a game frame, the facing; [triggers.md](triggers.md),
  "Spawn-triggers"). take08's lava burst (slava-erupt-jet: small-flare at 3,2, 100 a second for
  0.6 s, 0.47 -> 1.7 m) is a bright fire column that rises and fades within a second; with the
  demo's own rules (half-size quads, its count) it was a string of small blobs.
* **Placed effect objects (#258)** (`ale_fx.rs` `spawn_placed_effects`, `play_placed_fx.rs`;
  `bf/level.rs` `EffectObject`). The air particles a level places: the tutorial's 18 swarms of
  flies (xfly), 7 of butterflies (xbfly) and the lava yard's 3 clouds of embers (slava-fuzz),
  and other levels' fires, torches, sconces, steam, dust storms and power-up spot glows (1453
  placements over the levels; m01_a has 65).
  * **What the game does.** An `<effect-object>` (object-type 17) is an object of an effect type
    (objecttypes `<effect>`). The world ticks it from the level's start (0x21b7a0), and it made
    its ALE effects as it was set up from its type (0x21a180, FUN_00219fc0): it runs its effect
    type's ALE effects and its light effect where it's placed. A spawn-trigger's template isn't
    in the world (FUN_000dfe90): only its copies run (the tutorial's lava jet and farcast pads,
    "Effects the triggers make"). Nothing else starts these: no trigger names the tutorial's
    flies, and their `lifetime` is FLT_MAX.
  * **The object.** Its life is the placement's `lifetime`, else its type's `max-life`: a 0 is
    taken away at the first tick (e01's exp_gasmain, never seen); every other placed life is 90
    days or more. Its type's h_0c88c46c is the user parameter it starts with (FUN_0021c380, 1
    on nearly every type), its type's bounding sphere its distance LOD. Each emitter runs once,
    for its lifespan (0x2a5b10): the ambient ones for 1e10 s, the few explosions levels place go
    off once at the start. An alchemy entry with a creation-time starts that much later (m14_c's
    meteor strike, 2 s).
  * **Camera-attached types** (h_1e40fd68, the type's flag: the placement's own isn't read,
    0x21c000) are kept at the camera each frame, h_0e528ccb m along its view (FUN_0021a700). The
    levels place eight, all at 0 m: six start with their level (e01's dust storm, m03_a/b/c's,
    m14_a/b's meteor rain), and m09_b/c's two are spawn-trigger templates the demo doesn't
    spawn yet. Their particles stay where they're born.
  * **Taken away** by a destroyer-trigger that names one (play_triggers.rs takes them in with
    its copies: e34's six tutorial pointers, e40's io tablet glow ...), or with the breakable
    object it's attached to (`<parent Type="1">`, FUN_0021a700: e01's barrel fires go when
    their barrels have gone, on the frame the model does). Either way the effect object is
    destroyed, and every particle and light it had made goes with it on that frame (#308:
    `ale_fx::TakenAway`; the game's destructor takes every node of its ALE effects out of the
    manager, FUN_0021a4d0 -> FUN_0021c700 with AL 0, so none lives out). A hardpoint link
    (Type 2: the torches and sconces) doesn't take it away (FUN_0021a700 acts on a type 1 link
    only), but the holder's removal stops the effects it lists (FUN_0015a1d0): the emitters go and
    the particles out finish, so the flame dies down and its light fades. What the game does, and
    how it was checked: [scenery.md](scenery.md) "What hangs on an object when it goes", #308.
  * **Drawn** with the game's particle rules (the triggers' effects' shapes, count once a game
    frame, facing and LOD; the shapes are the table at 0x402a4c, where a shape-2 quad spans -1 to
    +1 of its Scale, twice the demo's own quad, and a shape-3 one stands on its particle) and
    more read from the game's code for these (`Compiled::placed`):
    * every particle starts in its emitter's box, which reaches its Width / Height / Depth to
      either side (both cube classes' spawns draw [-1, 1) of each: 0x2b17c0, 0x64e30; a game
      cube's OffsetHeight h_e2999ffd from the emitter up), and none is attached; FxCubeEmitter's
      polar spread is MinSpread + r x (MaxSpread - MinSpread) with r in [-1, 1). So xfly's flies
      start anywhere in 39 x 18 x 39 m, centred 12 m above their object;
    * a sphere emitter's particles start MinRadius + r x (MaxRadius - MinRadius) out along their
      direction (0x2a8a90) and a cone's the same out along their own way, its polar angle
      MinSpread + r x (MaxSpread - MinSpread) (0x2aa190), each r in [-1, 1). So a smoke
      column's puffs (0.04-0.21 m, spread up to 30 degrees) start within 21 cm of their emitter
      along their rise, where the demo's own reading (the other effects') puts them out flat
      from the cone's axis;
    * every facing-3 appearance is drawn stretched along its motion and turned about it to face
      the camera (`Pair::axial`): the draw picks the shader's facing by GeneralApp_FacingType
      alone (0x2a1de6), where the other effects do so only for their AngleFadeOut ones. So
      e01's barrel and trash fires, its campfires and m01_a's braziers are flame tongues rising
      from their coals (spun at random about their foot they were a flat star of flame), and
      the godrays (e06, m01_a/b: 4 m quads up to 70 times as tall, falling at 0.06 m/s from 171
      m up) hang as pale shafts;
    * turbulence fields run (FxTurbulenceField, 0x2a9440 / 0x2a9590): once a game frame each
      particle's velocity is pulled by dt x Approach x 4 of the way toward a new random
      velocity, a random direction x r x Magnitude with r in [-1, 1). xfly's 96.8 and 0.73
      keep the flies darting about at ~13 m/s (the root mean square in the demo), turning
      within a few tenths of a second; the butterflies' 24-35 and 0.20 at ~1-2 m/s.

    The other effects (the grenades', the guns', the triggers') keep their own checked rules:
    the box and the spawns for all of them are #164, the fields #101.
  * **Held out:** the five placed effects whose curves vary with the user parameter
    (s-lavalight, s-lavapool: the lava pools' glows; slava-cavern-splash, duststorm_med,
    env_fl_mdsplash). The game draws them at the parameter their objects start with, 1:
    FUN_0021c380 sets it before FUN_00219fc0 makes the slots, FUN_0008f5d0 hands it to each ALE
    effect (the effect's +0x18, 0x2abaa0), which sets it on each node (0x66a00, the +0x88 its
    curves are read at). There s-lavalight is 14 a second 4.9 m sheets of sidedistortion2 up
    to 21 m tall over a 23 x 22 m box, added at an alpha of 0.108: a faint sheet. The demo draws
    it far too strongly, an orange curtain over the whole view of the tutorial's lava yard that
    take08 doesn't show (100-134 s: the far hills stay clear behind it); the likely cause is the
    faint added sheet taken as linear light (`Source`, #165's blend). They stay out until that's
    drawn as the console does (`BF_PLACED_FX_VARIANTS=1` runs them anyway).
  * **Budget:** the placed effects' particles have their own cap (4000; m01_a keeps ~2000
    alive, the tutorial ~300, e40 ~600, sdm_e34 ~430), so they never crowd out a fight's
    (3000). Both are counted afresh each frame from the particles there: a count kept from frame
    to frame missed a map's end, which takes them all at once, so m01_a loaded a second time
    drew half its torches and a third time none. Frame time (bf_play, 40 s, mean): the tutorial
    5.5 ms against 5.2 without them, m01_a 5.6 against 5.1, e40 5.3 against 5.1.
  * **Not done:** their sounds (the fires' and torches' loops) and damage areas (e34's exp-oxy),
    a finite life's end and the start over of a type flagged h_0d5e990b (FUN_0021c500: none
    is placed with a life under 90 days), each alchemy entry's own max-life (the slot's +8,
    which FUN_0021a850 reads: FLT_MAX on nearly every entry, 10 s on h_f90c728a's in m01_a/b
    and m05_b/y, 207 and 600 s on a few in e01, m01_a and sdm_e01; the demo runs them on), the
    lights levels hang on them (`<parent>` light objects of type h_09020c30: #143's; the game
    drives one such light-object from the light effect, FUN_0021d270, where the demo draws a
    point light per particle: levels.md "Linked lights").
  * **Checked** (scratchpad t258dev):
    * The tutorial's landing spot (take03 30-33 s: `BF_MAP=tutorial
      BF_TEST_GOTO=65.55,74.03,55.6,73.05 BF_TEST_GO_AT=999`): dark winged flies dart about the
      sky over the hill and the palms, one to a few in view at once, as in take03.
    * The lava yard (take08 45 s: `BF_TEST_GOTO=50.68,17.53,55.8,8.9 BF_TEST_GO_AT=999
      BF_CAMERA_PITCH=0.25`): the sky above the pits full of drifting light specks, the embers,
      with flies and butterflies; take08's are as many by a dot count through the takes'
      encoding (the demo's frames scaled to 480 lines, x264 crf 16: ~80 specks of contrast 25
      or more in the same region, take08 ~80), but the demo's are warmer and crisper.
    * The pool at 120 s (`BF_TEST_GOTO=46.26,-25.92,51.4,-34.5 BF_CAMERA_PITCH=0.12`): no
      curtain, as take08 (the orange light on the hill there is #143's).
    * e34: `BF_TRIGGER_FIRE=h_f211fe52,6,4.0` removes the tutorial pointer by (69.2, -92.5)
      ("removes the effect h_f3bee12f"). e01: `BF_TEST_HIT=5.0 BF_TEST_HIT_NEAR=-110.96,51.77`
      breaks the fire barrel, and its fire goes with it. Since #308 the particles and lights
      go on the same frame (scenery.md, "Verified": before, the pointer's glow faded over 0.9 s
      after its removal and a barrel's lights for 1.3 s after its break).
    * The tutorial's opening: the lava bursts and the farcast come once each, as before (the
      templates are left to the triggers); e40, sdm_e34 and the test map play as before.
    * m01_a loaded three times in one test session (`--test` with
      `BF_TEST_LEVEL=1,m01_a;30,m01_a;30,m01_a BF_ALE_LOG=1`): each load starts from 0
      particles and keeps ~1950 alive, the same each time; so does a map played after
      `BF_TEST_BACK` has gone back to the menu.
    * e01's fire barrel (`BF_MAP=e01 BF_TEST_GOTO=-110.96,64,-114,50`, frames 100-140): flame
      tongues stand up out of the coals and over the rim. The bright flat disc over the barrel's
      mouth is its light (light_barrelfire: several point lights 0.3 m up inside the barrel,
      4.7-7.3 m reach), see below.
    * m01_a (`BF_TEST_GOTO=30.6,29,30.6,32`, frame 89): the godrays hang as pale shafts in the
      mist beyond the trees.
  * **Doesn't match yet:**
    * The flies look about 1.65 times as big as take03's and take08's and cross the picture
      about twice as fast; their speed for their size on screen matches within 25%, so the
      turbulence (read from the code: ~12.6 m/s root mean square) is likely right and their size
      is off. 1.27 of it was the demo's narrower view, the game's since #269
      ([camera.md](camera.md): about 1.30 should be left, not measured again); the rest wants a shot of a fly at a
      known distance in xemu. Their size is xfly-basic's Scale x 2 (shape 2, 0x402a4c).
    * The embers are warmer and crisper (the demo's additive blend in linear light at 720
      lines, #165), and they come and go in waves of 5-10 s (1 to 42 specks within 6 s).
      Nothing in slava-fuzz is periodic: its rate (4.68 a second), life (7.97 s), speed and
      transform are constants, and each particle fades in over its first quarter and out over
      its last fifth. It keeps ~37 alive, each a 14-35 m sheet of noise2's dots; the waves are
      the few sheets in front of the camera coming and going. Not checked against a take's
      steadiness.
    * e01's fire barrels' light (light_barrelfire, ~7 point lights at once 0.3 m up inside the
      barrel, 4.7-7.3 m reach) burns the coals at the barrel's mouth out to a flat bright
      yellow disc: the demo's point lights (inverse-square, `LIGHT_LUMENS`) against the game's
      light falloff, not traced. Every effect's light is drawn so; not this ticket's.
* **Pickups.** They're stored at their spawn height in the level file; they now settle onto the
  floor below.
* **Compound joints.** A compound object's joints carry a rotation (a w-x-y-z quaternion),
  applied to the part (the wall sets turn pieces by 90° and 180°). A part sits at the joint's
  parent-point + child-point, both in the parent's frame; the rotation turns only the part about
  its own origin. Bulgar's compound `h_0baf091a` (hangar, bunk rooms: 19 pieces) is the one placed
  object with non-zero child-points, and only this reading puts every piece inside the building:
  read as parent-point − rotation × child-point, its roof bays landed outside the hangar and its
  turned bunks went through the walls (issue #12). `BF_OVERLAP_LOG=1` (bf_level) lists large
  objects whose boxes overlap much.
* **Winding.** Object triangles are wound to face along their stored normals, so a mirrored mesh
  isn't culled from the front.
* **The gate (h_0792d16a) and texture address modes.** Its two leaves (h_e783bd32 → mesh
  h_e01d9aa6, h_fcd25d86 → h_178bc697) are the same panel shifted 0.996 m, with matching UVs
  and no joint rotation. The mirroring is in the texture's sampling: each leaf runs its
  texture u 0..2 across, and the texture entry's `address-u` / `address-v` (h_1c82a17b /
  h_078bf0c1) is `TAM_MIRROR` (h_18712765), so the second half is the first mirrored and each
  leaf shows one symmetric brace. The other modes are `TAM_WRAP` (h_e3012d9e, most textures)
  and `TAM_CLAMP` (h_e041e0b9). Every level and model texture is now sampled by its own modes
  (`Game::texture_address`); 24 levels use mirroring somewhere.
* **Invisible or glowing materials.**
  * Power-ups (Brute / Force / Health / Stamina) use a placeholder cube whose material no level
    defines; it isn't drawn.
  * Untextured glow materials (type h_f539fe8c, e.g. the medkits' shell) are drawn as their
    glow colour at the wrapper material's opacity.
* **Hits.** Shots, grenades, decals, the memory chip and the follow camera use the same collision.
