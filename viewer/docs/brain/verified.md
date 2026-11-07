*Part of [brain/](README.md), the former page `brain.md`.*

## Verified

Muted runs in the background window (`BF_MUTE=1 BF_BACKGROUND=1`), captured at 15 fps, against
todo/28-tutorial's takes:

* **The wall sentry** (`BF_MAP=tutorial`, the level's start with tutorial_start_long):
  TUTORIAL_terrorist_5 walks west along the walkway through the first shot by its walk clip's
  root (scripted play) and is home, within wpoint1493's 1 m, 5.1 s into the 6.1 s shot. In
  take03 it's still walking at ~5.9 s (the game's route home isn't the straight line).
* **The first patrol** (`BF_TEST_GOTO=44,91,34.4,94.3`, take07): TUTORIAL_terrorist_2 and _18
  come with tutorial_show_patrol and patrol chain iD 1 from its head (2.5 and 6.6 m off it),
  through the shot at the walk clip's root speed, 1.86 m/s: the leader passes its second point
  6.5 s into the script (take07: ~5.7 s). With Tex standing on the plate (`BF_TEST_PANEL=a,1
  BF_COMBAT_LOG=1 BF_SHOT_LOG=1`), the leader sees him at 35.5 m 15.3 s after the spawn, stops
  and fires: 10 of its 59 shots hit him (17%; 0.7 to 1.7 HP a hit; take07 1.1 to 1.8). Take07's
  first hit came 14.2 s after the script's start, the demo's 17.2 s (#227's build: it fired where
  it saw him; since #336 it walks in first, and since #244 it sees him sooner and farther: the
  table under "#244" below). The second stops right
  behind it (its muzzle 0.1 m from the leader's body) and fired through it (#227's build; since
  #244's fix round it holds fire, "Verified" below): of its 88 shots 5 hit
  the leader (x0.7: 2.1, 1.4, a 16.6 head shot, 1.7, 2.0), killing it 13 s in, and 3 hit Tex once
  the leader was down (3.4%). With the aim helper (`BF_TEST_AIM_HOSTILE=14`) Tex kills both:
  +$60 each.
* **The patrol walked out** (Tex away at the mine field, the router fired): the chain to
  patrol_point11 and back, turned round at each end, ~66 s a round trip, on and on.
* **The marines** (`BF_TEST_GOTO=-15.6,-87.4,-60,-75 BF_TEST_GO_AT=14`, take11): they farcast
  in at 7.57 and 7.87 s of tutorial_farcast_vsquad_short, held until it ends, stand on their
  spawn points until Tex moves, then run with him (`squad_ai`) and, when he stops at
  (-60, -75), fight terrorists 31 and 32 at 20-40 m (they see and shoot the marines too): both
  terrorists die to the marines' fire 10 and 12 s after they spawn (+$60 each). Stout hit with 7
  of his 11 shots, Cogburn with 8 of 10 (accuracy 0.6); one of Stout's went into Cogburn, in his
  line (x0.7: 1.6 HP). The terrorists hit with 3 of 16 and 1 of 12.
* **The lava yard** (`BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999`, Tex on the plate
  h_f42fa5b8, take08): the plate's router spawns 29 and 30, who run home across the pit (1.5
  and 2.3 s). Before #244 nobody saw Tex in 60 s: the demo's sight ray took the hinters for
  blockers, and the band h_ee99c575 (body 64) lies twice across the line from their homes to the
  plate (with #338's own hinters in; on e01's shapes before, 30 saw Tex at 26.9 m on its way,
  stopped and fired, -1.7 a hit, and died to the aim helper). Since #244 the ray leaves the
  hinters out and runs head to head ("The sight ray"): 30 sees Tex at 28.5 m at 2.67 s, still on
  its run home, stops 26.0 m off at 3.2 s and fires, first hit at 8.9 s (-1.7 to -2.9, 9 hits in
  the 60 s); 29, home at 2.53 s, stands facing along its run and sees no one. **take08 doesn't
  agree with that:** nobody fired on Tex while he stood on the plate (93.9-106 s) nor in the 20 s
  after he'd walked on to (46.3, -25.9); his first hits (-1.5 to -2.8) came at 129.68 s, 35.8 s
  after he stepped on the plate.
  The game's ray was clear for long stretches of it: its radar shows red blips 12 px (25 m, at
  2.05 m a pixel) ahead of him at 95.5-96.0 s, as a ping, and 10 px (20-22 m) from 104.5 s to 132
  s, with gaps (the two at their homes, 19-21 m off, through the band), and the frame at 118 s has
  one standing in the open facing him. So the divergence is a **hold upstream of the ray**: the
  pair's ray is clear from 104.5 s, the terrorists are in view, and nothing is fired until 129.68
  s, 25 s later. The demo has no such hold, and what makes it in the game isn't found (the
  brains' state at their spawn, allow_combat's grant and a script are all unread). The demo's 30
  fires on its way home where take08's wait 35 s: the match the hinter band gave was by the wrong
  cause. **Corrected since** (the analyst's reading of take08's frames, todo/244-ai-sight/
  answers.md §2.2): the terrorists do fire in take08, from at least 112 s (a tracer across the pit
  at 112.0 s, a muzzle flash at 127.5 s), and miss until 129.68 s, when one moves (ATTACK_COVER's
  23 s re-pick, #384's): no hold upstream of the ray; the fix round's lava yard has the demo's
  numbers.
* **The base** (staged: its routers fired with `BF_TRIGGER_FIRE`, Tex standing at the gate, in
  the plaza and at the third building with the aim helper): terrorists 0, 6, 8, 9, 10, 11, 13 and
  15 were killed, most having seen and fired on Tex (the sniper 11 hit him for 8.6-10 from 51 m
  and killed him once, Tex standing at the third building); every death paid +$60 and counted on
  h_e8ea7ba2 (router99's twelve). The wall sentry (on its walkway), the sniper 7 and the third
  building's 12 and 21 (inside its walls) were out of the aim helper's sight from the spots
  tried, so no run cleared all twelve (take12's clear had the marines and healing).
* **Every spawn at once** (Tex away, every router fired): who arrives home and who stalled on
  the straight walks before #336 (the next bullet: since #336 they walk routes); the paces from
  `locomotion` as logged.
* **#336, the goals on routes** (nav.md's "Verified" has the runs): the tutorial's walks home
  arrive (21, 13, 6 and 3 included, which main's straight walks stalled), 9 stopped at its shut
  door and brought inside its area by PATH_TOWARD_HOME; the patrols walk their chains; m09_a's
  Hounds are home at 15.8 s; the first patrol approaches to ~26 m, pursues when it loses Tex and
  first hits him at 24.9 s (main 20.6); the test map's Filthy Mutant closes from 45 to 25.9 m and
  fires; e23's mutants keep their home areas as in todo/132-mutants; testtools.md's NPC fight
  tells the same story (the NPCs within their stand-off stand, as before; the hits after the
  first few differ, the attack goal's draw taking a random number).
* **No regressions** (main's build, e5d2f74, against this one): e40's first 40 s (the same log,
  no frame differing by more than 24 levels), sdm_e34's autopilot (the same log, 296 of 300
  frames the same: the first four differ between two runs of one build too), the test map's NPC
  fight of testtools.md (the same combat log, 222 of 225 frames the same and the other three by
  1-2 pixels). m09_a's four Shrikes now run home at its start (two Hounds stall 23 and 29 m
  short, off their ledge): seen in the opening shot, where they stood before.
* **After main's #224 merged in** (main 9686256's build against this one): the hooks under the
  free camera, `-- --test` e40's take04 scene (`BF_TEST_TRIGGER=3.0-3.15` with
  `BF_TEST_FREECAM`: 69 of 70 frames the same, the Therm Sweeper's 80 -> 79 on the same frame)
  and the test map (`BF_TEST_TRIGGER=2.0-2.3`: 50 of 50; `BF_TEST_FIRE=1 BF_TEST_SWITCH=2.2`: 75
  of 75, the fire held off by the gate in both, the switch at 2.2 s in both); sdm_e34's
  autopilot (the same log, 297 of 300); the NPC fight (the same combat log, 221 of 225: one
  tracer two frames earlier, where main had it before #224, and 1-2 pixels). The first patrol
  (with `BF_TEST_PANEL=a,1`) and the marines' fight as above.
* **The shared sensor** (#197; main 403630b's build against this one): the first patrol's run
  above, 40 s, gives the same brain and combat log line for line. The squad's and the NPCs'
  sight on the test map (an e40 Shrike set to fight at (4.5, 4.3), Tex at z = 84.3 / 114.3, i.e.
  80 / 110 m): no sighting, no shot, as main since #123; Tex walking in from 95 m: a contact at
  74.8 m and fire; walking out from 70 m: fired on out to 81.6 m, lost at ~86 m (75 x 1.15,
  7.53 s), forgotten 3.75 s later (11.27). take14's recipe (skills.md): lost at 5.87 as main,
  "forgets Squad(2) (stealthed)". testtools.md's NPC fight: the same story (enemy Brutus fights
  the squad and dies, Flint set to fight fires back, set to friend stops), the hits' order
  shifted by the squad's field of view.
* **#244, the sight ray's end points** (main d61b190's build against this one; `BF_MUTE=1
  BF_BACKGROUND=1`, 15 fps). The first patrol (the recipe in "Test hooks": Tex walks onto the plate,
  (34.4, 94.3), `BF_TEST_PANEL=a,1 BF_COMBAT_LOG=1 BF_SHOT_LOG=1`; times from
  tutorial_show_patrol's start, sim 2.27 s; take07's from its letterbox, video 41.77 s, where Tex
  stood at (32.67, 93.7), 1.8 m from the plate's spot):

  | | main | #244 | take07 |
  |---|---|---|---|
  | the leader sees Tex | 16.0 s, 35.3 m | 14.3 s, 40.0 m | radar blip at 12.8 s (video 54.6), 20.8 px from the middle: 43 m (the radar's 120 m across 39 units, 2.05 m a pixel in the 1280 x 720 frame; #245's 39-42 m) |
  | the second sees him | 17.3 s | 15.7 s | |
  | the leader's approach stops | 17.7 s, 25.9 m | 16.7 s, 26.2 m | it doesn't: its blip stays at 20.2-21.3 px, ~41 m, from 54.6 to 68.8 s (7.4 s of game time, the WEAPONS panel's stop taken out) |
  | its first round / first hit | 21.1 s / 22.7 s | 17.1 s / 21.7 s | tracer 13.2 s (video 55.0) / hit 14.2 s (video 56.0) |

  On the plate's spot the new end points put the leader's sighting 4.7 m farther and 1.7 s sooner
  than main's (40.0 m, against take07's 41-43 m), and its ray stays clear once it has stopped (main:
  cut from 16.5 s, so it pursued at 20.2 s; now only the second, 1 m behind it, loses the ray, from
  18.1 s, and pursues at 21.8 s). The leader's head is 1.76 m over its feet at the sighting, Tex's
  2.01 (the sighting line of `BF_BRAIN_LOG`). The lava yard is above: the terrorists now see
  across the pit, and the demo's 30 sees Tex at 2.7 s and hits him from 8.9 s where take08's wait
  35 s.

  **The tester's findings** (this build against main's, on the same recipes):
  * **At take07's own spot** (Tex standing at (32.67, 93.7)) the sighting is not nearer the take
    than main's: main's leader sees him at 12.66 s, 42.8 m; this build's at 11.60 s, 45.3 m;
    take07's blip shows at 12.8 s, 42.6-43.7 m. There the old ray was the nearer, and the new one is
    1.2 s early and about 2 m far. The pair table's delay ("Not modelled", above: an unalerted pair
    is tested about every 1.5 s) would put the game's sighting after the geometry's by about that, a
    reading and not a measurement. The "#244" column of the table is the plate's spot alone.
  * **Rounds into the slope.** At take07's spot the terrorists now land 4 hits in 40 s where main's
    land 27, and 31 of 143 and 58 of 183 of their rounds (two runs) strike the terrain less than 20
    m from the shooter, against main's 0 and 3. The AI aims at the demo's chest, 1.1 m over the feet
    (`AI_AIM_HEIGHT`, play.rs `ai_fire`), where its ray now clears at head height, so a chest line
    the slope hides is fired on as one it sees. (FUN_00024f80 aims at the target's vtable +0x40
    position, its root node, about 1 m over the feet, plus FUN_00024c00's miss; so the game's AI
    also fires a hips line under a head-height ray, and why the demo's rounds strike the slope so
    much oftener is open.) A follow-up: **which point the game's AI aims at** (FUN_0022dc60 /
    FUN_00024c00). The first hit stays behind take07's besides because take07's leader stands ~41 m
    off and fires 0.4 s after the blip, where the demo's walks in to 26 m holding fire (the attack
    goal's approach, GOAL_DEBUG1's mapping to ATTACK_VARY, #26's). (**Settled by the fix round,
    below:** the game aims at the hips, which the demo does now, and its muzzle corridor is in; the
    rounds in the slope stayed, because they come from where the leader's approach ends, not from
    the aim or the ray.)
  * **The lava yard's divergence** is a missing hold upstream of the ray: take08's radar has the
    blips from 104.5 s to 132 s and nothing is fired at Tex until 129.68 s ("The lava yard" above).
    (**Settled in part by the fix round:** the terrorists do fire in take08 from 112 s, and miss
    until 129.68 s, by the analyst's reading of the frames; no hold.)
  * **e23's ledge Ugly** is now shot by the squad within ~1.1 s, so that staging no longer shows the
    PATH_TOWARD_HOME loop (in my run it makes one out-and-back, at 34.7 s and 39.0 s, where main's
    makes three, at 33.6 / 37.9, 50.8 and 66.2 / 70.5 s).
  * **Frame time** is up 10% in a 16-Shrike fight: the ray is cast every frame for every candidate
    in reach and in view (the pair table's schedule, in "Not done", would cut it).

  **The fix round** (the muzzle corridor, the hips as the aim, EVT_SHOT_BLOCKAGE; the first #244
  build above, main d61b190's and this one; `BF_MUTE=1 BF_BACKGROUND=1 BF_NO_FADE=1`, 15 fps;
  times from the first patrol's script start, sim 2.27 s). Unit tests: `the_muzzle_corridor_meets_a_third_character`,
  `the_muzzle_line_goes_in_the_games_order`, `the_muzzle_bit_and_its_blocked_clock`,
  `the_fire_gate_holds_a_blocked_line` (play_brain.rs), `shot_blockage_needs_the_clocks_and_the_goal`
  (play_goals.rs).

  * **The first patrol at take07's own spot** (`BF_MAP=tutorial BF_TEST_GOTO=44,91,32.67,93.7
    BF_TEST_PANEL=a,1 BF_BRAIN_LOG=1 BF_COMBAT_LOG=1 BF_SHOT_LOG=1`, 43 s of the fight):

    | | main | first #244 build | this | take07 |
    |---|---|---|---|---|
    | the leader sees Tex | 12.7 s, 42.8 m | 11.6 s, 45.3 m | 11.6 s, 45.4 m | blip 12.8 s, ~43 m |
    | the leader's approach stops | 15.4 s, 26.2 m | | 15.0 s, 26.0 m | it doesn't (~41 m) |
    | rounds: leader / second | 120 / 110 | 120 / 110 | 120 / 107 | |
    | rounds striking the world under 20 m | 4 / 0 | 35 / 37 | 32 / 35 | |
    | hits on Tex | 32, first 16.7 s | 4, first 21.4 s | 1, first 22.9 s | first 14.2 s |

    **The corridor and the hips did not bring the rounds out of the slope at this spot, and they
    are not what put them there.** The first #244 build, with no corridor and the chest aim, has
    them as this one does (35 and 37 of its rounds). The first round of each build, at the same
    sim time (18.00 s) and the same elevation (dir.y 0.313, 0.310 and 0.313), tells the story:
    main's leader fires along a bearing of -142 degrees and the round meets the hill behind Tex,
    33.5 m out; the other two's along -146, 4 degrees to one side, and meet the slope 13.7-13.8 m
    out, at (26.8, 22.3, 84.9), 10.6 m from Tex. The leader's `BF_GOAL_WATCH` rows put it, when
    its approach has ended, at (18.05, 74.08) in main's run and (18.86, 73.67) in this one, 0.9 m
    apart, 25.5 and 25.4 m from Tex: it sees Tex 1.1 s earlier and 2.6 m farther, on another
    stretch of the patrol, so the end of its route lies elsewhere across the road, and from the
    second place the ridge is in its line. What the log's `muzzle corridor` lines add: at the sighting, 45.4
    m off, the leader walks with its gun down (muzzle 1.06 m over its feet) and both corridors are
    blocked (reason 4); while the gun rises the bit is "taken as clear"; once it is up the
    corridor to Tex's head (2.01 m) is the sight ray's twin and **is clear**, the one to his hips
    (1.19 m: his clip's root-height, above the old 1.1 m chest, not below it) blocked by the ridge
    from 14.9 s on (now and then both are blocked for a second or so). FUN_00057360 needs either,
    so the bit is set, and the gun is given the hips: the accurate rounds (the Red Hand's 12.5%)
    strike the ridge and the misses, aimed 0.85-1.2 m over the hips, go over Tex's head. So the
    gate holds a target that is seen only for a third character in the line (below) or a gun
    still down; what would bring the rounds out of the slope here is where the leader stops, the
    approach's end (#336's routes, #26's: take07's leader stands ~41 m off and never walks in).
  * **The plate's spot** (`BF_TEST_GOTO=44,91,34.4,94.3`, the same flags; main's run against
    this): the leader sees Tex at 14.3 s, 40.0 m (main 16.0 s, 35.3 m), fires from 17.1 s, first
    hit 21.7 s (main 22.7 s), 11 hits in 43 s (main 15). **The friend in the line holds fire:** the
    second, 26.1 m off behind the leader, has both corridors blocked by it from 19.4 s ("head
    blocked, hips blocked: the attack check refuses it, reason 4"), posts EVT_SHOT_BLOCKAGE at
    23.1 s (its muzzle line blocked 3.8 s, the ray 2.9 s), tries GOAL_PURSUE, which walks it 2 s to
    21.2 m (it sees a contact again at 25.3 s) and it fires from 25.3 s, 93 rounds, none into the
    slope and none into the leader. Main's two fired through each other: 3 hits by one on the
    other.
  * **The lava yard** (`BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999`, Tex on the plate, and a
    variant that walks him to take08's spot (46.26, -25.92) at 5 s): 30 sees Tex at 28.5 m at
    2.67 s, still running home, stops 26.0 m off at 3.2 s and fires from there; its corridors are
    both blocked at the sighting (the gun down), clear (head and hips) from 3.0 s on; 50 rounds,
    9 hits on Tex from 6.7 s (-1.7 to -2.9) with the walk and without it, 2 of the 50 striking the
    rim under 20 m (16 of the 50 with Tex on the plate). 29 stands facing along its run and sees
    no one. **It is not take08's**: there the terrorists fire from their homes from 112 s (Tex
    walked off the plate at 107.1 s) and miss until the first hit at 129.7 s, when one moves
    (ATTACK_COVER's re-pick, #384's), where the demo's 30 stops 26 m off on its run home (the
    ATTACK_VARY approach, as ATTACK_COVER is refused) and has a clear hips line from there. Main's
    60 s: no sighting, no shot. Whether the rounds from the home spots strike the rim as the
    game's do can't be taken before the cover logic: the demo's terrorist doesn't fire from its
    home spot but from where its approach ends.
  * **Regressions** (main d61b190's build against this one; the first #244 build against this
    for what this round alone changed): e40's first 40 s (the same log, all 11 lines) and
    sdm_e34's autopilot (the same log, the three frames compared identical); e23 staged as
    todo/132-mutants' F1: the ledge Ugly makes one out-and-back (PATH_TOWARD_HOME at 35.3 s), as
    the first #244 build's (36.3 s) and not main's four (35.1, 39.5, 52.3 and 59.1 s) as the squad
    shoots it within ~1 s; the test map's NPC fight of testtools.md (`-- --test`, 225 frames):
    the same story (the enemy Brutus opens up on Hawk, the squad answers, it is knocked down),
    with **the squad's friendly fire gone**: the squad's Flint and Hawk hitting their own Brutus
    through the line were 5 hits in main's run, 5 in the first #244 build's and none in this
    one (a body in the corridor holds the fire); the enemy Flint dummy in the squad Brutus's
    line still takes 3 hits (main 3, the first build 5), rounds off the corridor's line. Take14's
    recipe (skills.md: `-- --test BF_CHARACTER=2 BF_NO_SQUAD=1 ...`) and the sensor run (an e40
    Shrike set to fight at (4.5, 4.3), Tex walking in from z = 99.3): the same logs in the three
    builds, 16 hits from 1.47 s, "forgets Squad(2) (stealthed)" at 5.87 s, a contact again at
    18.20 s; and a contact at 74.8 m at 3.73 s, 26 hits from 7.80 s.
  * **Frame time**: a 16-Shrike fight on the test map (`-- --test`, `BF_TEST_FULL_SQUAD=1`, 16
    `shrike soldier@m09_a` set to fight in a row 38 m in front of the squad, 600 frames, the sight and
    combat logs on): 25.3 and 28.8 s of wall time for this build, 26.3 and 26.8 s for the first
    #244 build, 25.9 s for main (a first run of main took 40.2 s, cold): the spread of the runs is
    wider than any difference, and the render and the frame capture dominate; the corridor's cost
    (a ray and the bodies' cylinders for each contact seen, the hips' ray too with a log on) isn't
    resolved at this resolution.

  **Unchanged**: `cargo test` (`scan_by_the_team_table`, #175's table; the new
  `the_sight_ray_runs_between_two_points`); the test map's NPC fight of testtools.md, 270 frames, and
  the sensor run above (Brutus set to fight at (4.5, 4.3), Tex walking in from 95 m): the same
  sighting lines (the first at 77.5 m, "sees unit 4 at 42.2 m" ...), the hits' counts within what
  two runs of main's alone differ by (the load times move the frames a runs' events fall in);
  take14's recipe (skills.md): "forgets Squad(2) (stealthed)" at 5.87 and a contact again at 18.20
  in both; e23 staged as todo/132-mutants' F1: the same mutants act (the Ugly at 16.9 m stands and
  fires; the ledge Ugly approaches, leaves its 7 m area and walks back by PATH_TOWARD_HOME, but the
  squad shoots it too soon for it to loop, above), their contacts made from farther (a mutant sees
  the squad at 56.9 m where main's saw Tex at 48.0); e40's first 40 s (the same log, a frame 18
  pixels off) and sdm_e34's autopilot (the same log, 300 frames the same). What did change is in
  the tester's findings above.
  **The fix round on #384's cover** (main 51d4658's build against this branch merged with it;
  `BF_MUTE=1 BF_BACKGROUND=1 BF_NO_FADE=1 BF_COVER_LOG=1`, 15 fps; times from the first patrol's
  script start, sim 2.27 s). The merge brought three things in: `Sensor::sees_at`, the flank
  test's sight (FUN_0002f960), measures to the enemy's hips and casts the sensor's own
  head-to-head ray (`cover::Contact::head`), where #384's measured to a chest 1.1 m up and cast an
  eye-to-chest ray; the cover goals' hips (`cover::profiles`, its `Ctx`) come from the frame's
  `Seen::hips`, the one source of the characters' hips (`missile::root_point`) the aim and the
  sensor take them from too; and FUN_0002e330's T is 1.5 s for a target whose own brain is in
  cover or dodging (`brain::quick`, `Seen::quick`, `Lead::quick`: ATTACK_COVER, DEBUG1's cover
  half, a stun, a knock-down, a dodge playing), 3.75 s otherwise, for both of its clocks. Unit
  test `the_blocked_shot_time_follows_the_targets_goal`.

  * **The first patrol at take07's own spot** (`BF_MAP=tutorial BF_TEST_GOTO=44,91,32.67,93.7
    BF_TEST_PANEL=a,1`, 50 s of the fight). With cover in, both terrorists take the cover points
    41-43 m from Tex where take07's leader stands, and no round strikes the terrain under 20 m in
    either build:

    | | main 51d4658 | this | take07 |
    |---|---|---|---|
    | the leader sees Tex | 12.7 s, 42.8 m | 11.6 s, 45.4 m | blip 12.8 s, ~43 m |
    | the leader goes to | h_1e75e701 (0.6, 65.2), 43.3 m from Tex, 11.6 m off; there 17.3 s | the same point, 14.1 m off; there 14.7 s | stands ~41 m off |
    | the second goes to | h_1772d797 (-0.6, 70.6), 40.8 m from Tex, 16.2 m off; there 19.3 s | the same point, 19.0 m off; there 17.1 s | |
    | first round (leader / second) | 18.1 / 20.1 s | 15.6 / 17.9 s | tracer 13.2 s |
    | first hit on Tex | 21.0 s | 16.9 s | 14.0 s |
    | hits on Tex in 50 s, their damage | 29, 38.6 | 33, 40.3 | |
    | rounds into terrain under 20 m (leader / second) | 0 / 0 | 0 / 0 | |

    This build sees Tex 1.1 s sooner (the head-to-head ray) and its leader is 2.5 m farther from
    its point, so it is there 2.6 s sooner and its first round and first hit come 2.5 and 4.1 s
    before main's: 2.9 s behind take07's first hit where main's is 7 s behind. What is left of the
    gap is the walk: the game's leader fires before it is at its point (take07's first tracer comes
    0.4 s after its blip), the demo's holds fire while it moves (#384's Not done: the fire on the
    walk to cover, FUN_00024f80 at 20 and 35 degrees). The corridor gate holds nothing at the
    points: the corridors from them are clear (head and hips), now and then both blocked for a few
    frames as a terrorist dodges out.
  * **The plate's spot** (`BF_TEST_GOTO=44,91,34.4,94.3`): the leader takes h_1772d797 (42.8 m
    from Tex) as main's does; the second finds no point facing Tex and runs ATTACK_VARY, which
    approaches to 26.2 m and fires from there: 38 of its 133 rounds strike the ridge under 20 m
    (main's: 5 of 118, its eye-to-chest ray cut there so it pursued to 24.5 m and fired from where
    the ridge isn't in its line), 14 hits on Tex in 50 s (main 30), the first at 20.9 s (main
    22.7 s). So the rounds into the slope are left for a terrorist that finds no cover and walks
    in to its stand-off: the ridge at 26 m, as at take07's spot before #384, not the gate or the
    aim.
  * **The lava yard** (`BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999`: Tex on the plate; and
    `BF_TEST_GOTO=49.5,-15.5,46.26,-25.92 BF_TEST_GO_AT=5`: he walks to take08's spot at 5 s. The
    recipe without `BF_TEST_GO_AT` runs him into the pit's lava from 1 s: 30 sees him at 2.27 s
    and he is gone at 3.0 s, no round in either build).
    Main's build: nobody sees Tex across the pit (its eye-to-chest ray meets the hinters), no
    round and no hit in 80 s. This build: 29 is home at 2.5 s and stands facing along its run, as
    before, and sees no one; 30 sees Tex at 2.67 s (28.5 m) and takes 29's home cover point
    h_1ab28748 (65.2, -34.3), 5.4 m off and 24.5 m from his hips, is there at 6.2 s and the goal
    fails: "2.31 m off its cover point" (29 stands 0.8 m from the point; the two bodies'
    collision is the likely cause, the 0.9 m rule); it re-picks h_fd7b13b5 (82.3, -47.4), 19.5 m
    off and 42.1 m from Tex (at the rim; 45.8 m with Tex on the plate), is there at 11.5 s, and
    from there on moves between it and h_f47c2323 (83.2, -55.3) at its re-pick after 10 s at the
    point, dodging and stepping out. With Tex at the rim, whose walk starts at 5 s, its first
    round is at 12.1 s and its first hit at 13.4 s (7.1 and 8.4 s after Tex set off), 67 rounds
    and 14 hits in 70 s, all its rounds striking the terrain 50-70 m out, none the near rim. With
    Tex on the plate it never sees him from there, forgets him at 18.1 s, walks home (a 155 m
    route) and sees him at 22.0 m at 34.5 s: no point faces him, ATTACK_VARY, it stands 21.8 m off
    and fires from 35.2 s, 8 hits from 38.2 s. **It is not take08's**: there the terrorists fire
    from their homes, 20 m off, from 112 s (Tex set off at 107.1 s, 4.9 s before) and miss until
    129.68 s (22.6 s after), when one moves (the 23 s re-pick). The demo's 29, the one whose home
    is the cover point, never engages (its look is along its run: the idle looks are #26's), so
    30 takes its point, fails it and goes to a far one; nothing fires from the home cover point,
    so there are no rounds on the near rim to compare.
  * **Regressions** (main 51d4658's build against this one): e40's first 40 s, take14's stealth
    (16 hits from 1.47 s, "forgets Squad(2) (stealthed)" at 5.87 s, a contact again at 18.20 s)
    and the sensor run (a contact at 74.8 m at 3.73 s, 26 hits from 7.80 s) give the same logs;
    the test map's NPC fight (225 frames) tells the same story with **the squad's friendly fire
    gone** (Flint's and Hawk's hits on their own Brutus, 6 in main's run, none in this); e23's
    staging has the same cover story (h_f5f8f2ca at h_faf90c97, h_e187e355 at h_040fc844 ...: the
    Uglys' corridors from their points are clear, one contact's, Squad(1)'s, both blocked), and
    the ledge Ugly makes two PATH_TOWARD_HOME legs (34.9 and 39.2 s) where main's makes four
    (38.3, 42.7, 55.5 and 62.3 s), as the first #244 build's run did (the squad sees it sooner;
    not re-checked). Frame time, a 16-Shrike fight (600 frames,
    the combat log on): 11.8, 13.0 and 11.6 s of wall time for this build, 11.6, 12.3 and 11.5 s
    for main's (+2.5%, inside the runs' spread).

* **#384, the cover** (muted, in the background window, `BF_NO_FADE=1`; the runs are the test hooks'
  above; the fix round's, after the pick measured from the hips): take06's staging (Tex warped in
  first: the warp sets him on top of the block at that spot, the first floor it finds, and he drops
  to the ground in about half a second; warped in the frame the Ugly first looks, he was still up
  there, 5.7 m from h_eed0869f, and the Ugly took that point): the Ugly sees Tex at 3.9 m, refuses
  h_eed0869f (Tex 3.9 m from it, under 5), takes h_157fd721 (DEBUG1 in cover: its
  threat 0.33, Tex's 1.50), is there 4.5 s later and crouches, 13.5 m from Tex, facing him and
  firing; at 10 s it finds no other point and dodges left out of its home area, comes back by
  GOAL_PATH_TOWARD_HOME and takes the point again 2.9 s later, then steps out left the next times,
  every ~16 s. The three hits: at 500 hit points, hit for 97.3, 97.7 and 109.9 at the point, the
  cover is blown on the third (34.40 s, blacklisted) and DEBUG1 runs ATTACK_VARY, which fires 13.4 m
  from Tex until it's past its home area's edge (35.07 s, 1 m on from the point) and walks home.
  Flanked: with Tex walking off from 37 s it's flanked at its point at 38.60 s, done, ATTACK_VARY.
  take08's staging is "Cover" above: crouched at h_eed0869f 1.0 s after it saw Tex, 18.8 m from him
  (the first round, from the feet, sent it to h_157fd721 9.5 m off). The first patrol at take07's
  spot (`BF_TEST_GOTO=44,91,34.4,94.3`, Tex on the plate 8-9 m above them, the aim helper from 14
  s): TUTORIAL_terrorist_2, shot at 37.5 m, takes h_1772d797 (13.0 m off, 42.8 m from Tex) and is
  there at 23.8 s; _18 finds no point facing Tex (h_1e75e701 now just outside its 30-degree half
  cone by Tex's hips, h_1772d797 claimed) and approaches to 26.1 m as main's does; Tex's aim helper
  kills both by 25 s. The first round's had _2 at h_1e75e701 and _18 at h_1772d797. Not checked
  against the tutorial's footage. Against main's build (2138fb3), the first round: the tutorial's
  routes (every spawn router fired) and m09_a's start give the same brain log line for line; the
  test map's NPC fight draws the same goals (DEBUG1 for both, no cover for an NPC) and tells the
  same story, its hits' rolls differing between runs of either build.
* **#383, the fallback** (`BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures; the numbers and take07's
  comparison are in [nav.md](../nav/verified.md) "Verified"): on e23 the third Ugly with Tex 2 m off posts
  EVT_TGT_TOO_CLOSE at once, starts ATTACK_FALLBACK (a one-entry route of the link walk, 9.1 m
  from its own point), backs off facing him and firing, BACKSTEP at 1.84 m/s (take07's 1.82-1.83),
  and is done 8.0 m from him after 3.5 s; the tutorial's wall sentry backs off along its walkway
  at 4.4 m/s (BACKPEDAL), is taken home by PATH_TOWARD_HOME and falls back again every 4 s (the review
  round's run: posts at 8.07, 11.47, 15.47 and 19.47 s, each fallback done 8.0-8.1 m off at 9.73, 13.73,
  17.73 and 21.73 s, as before); a test map NPC with Tex walking after it (`BF_TEST_WALK=1`, 2 m off)
  posts once, holds the tests that pass 1.5 s apart while its walk runs (its clock 1.47 and 3.00 s:
  "one is queued while its ATTACK_FALLBACK walks") and runs its 7 m straight to the route's end at
  4.20 s, then posts again at 4.53 s: a walk of 4.1-4.2 s a post, posts at 0.0, 4.5, 9.1, 13.7 and
  18.3 s (before the hold it re-posted every 1.53 s and started the walk again each time); on e23 a
  second warp of Tex 1.55 m from the third Ugly in the middle of its walk (23.45 s, its timer out)
  gives the same "not posted" at 23.47 s and the walk goes on to its route's end at 25.80 s;
  `BF_TEST_MOVE_BACK` walks the Ugly's 7 m behind it facing Tex, done at the route's end. The
  first patrol (`BF_BRAIN_LOG`), the tutorial's routes recipe, sdm_e34's autopilot (298 of 300
  frames) and e23's F1 are unchanged.
* **#383 with the cover** (main's 51d4658 merged in; `BF_NO_FADE=1`, muted, in the background; main's
  build `bf_play_51d4658.exe` for the comparisons): **F2b on take06's staging** (the Ugly crouched
  at h_157fd721 from 30.0 s, Tex warped 1.5 m from it at 36 s): the cover's hook keeps the first
  EVT_TGT_TOO_CLOSE (35.93 s, 1.35 m off: "kept by its cover goal's hook"), the cover fails in the
  same frame on its 8 m rule (the target 1.5 m off at its point, blacklisted; DEBUG1 runs
  ATTACK_VARY), and the next post 1.54 s on (37.47 s; take07's 10.11 and 11.74 s, 1.63 apart) draws
  ATTACK_FALLBACK, which ends the cover (the point let go) and walks the link walk's route
  h_157fd721, h_0dcdca06, h_1eb2ac93, h_f90e1c8c (8.0 m between its points, the one the footage's
  Ugly took) at BACKSTEP 1.20, 1.60, 1.78, 1.82, 1.84 m/s, through (-67.9, 37.1) at 38.53 s,
  (-69.95, 35.98) at 39.80 and (-71.14, 35.35) at 40.53, where the footage has (-68.00, 37.06) at
  12.91, (-70.00, 36.06) at 14.14 and (-71.36, 35.34) at 15.0: the same line, a steady 0.5-0.6 s
  ahead (the footage's stand-up). PATH_TOWARD_HOME takes over at 40.47 s (the Ugly 10.6 m from its
  home), as at 14.997 s there. Main's build on the same staging has the same log to the second warp
  (the cover failed, ATTACK_VARY) and no fallback. The plain e23 staging of the first round (Tex 2 m
  north, no cover point to be had: DEBUG1 runs ATTACK_VARY): the post at once, the walk done 8.0 m
  off after 3.5 s, as before the merge; the test map's chase and the wall sentry the same as before
  (posts at 0.0, 4.5, 9.1, 13.7, 18.3 s; 8.07, 11.47, 15.47, 19.47 s). **Unchanged against main:**
  #384's take06 and take08 stagings (the same brain and cover log line for line, 49 and 51 lines;
  take06's frames equal but for the sky: 345 of 750 identical, the rest differing in under 700 pixels
  each, all of them in the sky's upper right), #336's tutorial routes recipe (the same 288 lines but
  the search's microseconds). **The draw's second goal:** e23's ledge Ugly (a dead end: Tex warped
  2 m west of it, the walk away east is 5.2 m of links): ATTACK_FALLBACK refused ("no link walk of
  7 m"), ATTACK_COVER's pick refused too (every point is outside its 7 m home area but the one it
  stands on, 0.6 m off), at every post;
  with `BF_TEST_FALLBACK_REFUSE` the same on F2b's staging (Tex stands between the Ugly and the
  points it could take). A refused start leaves the cover fields as they were (the first run
  wiped DEBUG1's state at each post, `cover::start` resetting them before its pick, and its 5 s timer
  began again each time). **Not staged:** the draw's ATTACK_COVER *started* (a refused fallback with a
  cover point to be had): it needs a point that passes the filter with the target within 3 m and
  the cover not holding at the post, which e23 doesn't give by itself.
