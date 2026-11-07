*Part of [maps/](README.md), the former page `maps.md`.*

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
    * The player and the hinters' faces (#341; `BF_FIND_HINTER=1`, `BF_MOVE_LOG=frame`, 15 fps):
      the scan finds 203 cells on m02_b where the AI's and the player's ground differ (all flank,
      none floor), 5438 on m09_a (2549 floor) and 5954 on m09_x (2677 floor). m02_b's body 89,
      at (125.5, 117.5) (floor 25.0, the band's face 30.19 over it):
      `BF_MAP=m02_b BF_TEST_GOTO=125.5,117.5,125.5,117.5 BF_TEST_FALL=10`: the player falls
      through the band and lands on the floor at 25.0 (the first JumpLand at t=1.067, frame 16); with the filter left
      out of `flank_below` it came down "on a face" at 30.15 and slid to (125.42, 117.99).
      m09_a (`BF_TEST_GOTO=-206,-215.7,-199,-215.7`): the feet stay at 57.64-57.67 across
      x -204.7 to -202.8 where the AI's floor (hinter body 39's) rises to 58.8-59.5 (the old
      step-up); the run now ends held at (-201.25, -215.28) by body 182 (object 171, ice
      crystals), where main got past on the hinter floor. On m09_x the player has no ground at all where the hinter's floor is (the
      terrain's hole under it: "no floor, face or flank", held as any hole, #296). The AI's
      is unchanged: the tutorial's lava yard (`BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999`
      with `BF_BRAIN_LOG`) has 29 home at 1.53 s and 30 at 2.53 s as before, and sdm_e13's band
      run (`BF_TEST_GOTO=90,-91,130,-91`) the player at (124.50, -95.15) at 11 s. The player's
      other queries leave the hinters out as well (the review's round): a knocked-down
      character's ragdoll body is `Mover::of(controlled).body()`, set again every frame it
      lies (`Ragdoll::set_body`, so a squad switch gives the member switched to or from the right
      one), the rock escape (`Arena::out_of_rock`, `escape`) and the warps' and spawn copies'
      `on_ground` take the mover; the squad's spawn table, made before it's known who is
      controlled, asks as the AI. Checked: `BF_MAP=m02_b BF_TEST_GOTO=125.5,117.5,125.5,117.5
      BF_TEST_FALL=10 BF_TEST_KNOCK=0.3 BF_TEST_HEALTH=5000`, Tex knocked down falling through
      body 89's band, gets up on the floor at 25.0 (JumpLand at 2.53 s), not on the band's 30.
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
    until it's spawned ([triggers.md](../triggers/README.md), #220; nor is one drawn but a pickup, #230), nor a placed
    character (`character-object`): a character's body is its own physics object, not the
    level's collision (#230; m09_a's squad started on their own placements' bodies).
  * Fallback: a level without collision data uses the drawn triangles instead.
