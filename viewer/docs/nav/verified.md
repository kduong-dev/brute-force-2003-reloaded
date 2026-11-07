*Part of [nav/](README.md), the former page `nav.md`.*

## Verified

* Unit tests (`cargo test`, `bf::nav::tests`): links one way as stored; invalid and avoided
  points gone round, and no route when every way is through them; the start's first neighbour
  exempt from the valid bit; a closed point opened again (a chain of links under 0.5 m, g 0,
  reaching it after it came off the heap: it comes off twice and the route goes the chain's way);
  g truncated (two 2.49 m links, 8, beat one 4.6 m link, 9); the radius limit; the first-entry
  drop with the corridor clear and kept with it blocked, and the one-entry route; the walk's
  radii, the look-ahead skip and its NO_SKIP stop, the corner rule's hold to 0.6 m, the end's
  reset; the pass test; the own point's first pick, its deeper search, the near walk within the
  0.25 s, and the far pick; the heap's order.
* The tutorial's lava yard (`BF_ROUTE`, both ways): from h_0e4f1e60 (48.7, -14.1) to the
  terrorists' end of the yard, h_187143b0 (71.0, -44.7), 12 entries: h_19d4b7b6, h_0a6c9ab1,
  h_1862fb9d, h_f5015ea8, then the walkway along the south pit to h_ee1bb73f and h_187143b0;
  60.5 m along it for 37.8 m straight across the pit; the own point dropped (the corridor to
  h_19d4b7b6 clear). Back the other way: h_f5015ea8 -> h_1862fb9d -> h_0a6c9ab1 -> h_19d4b7b6,
  62.1 m. Every leg's corridor is clear from its point (`BF_ROUTE_RAYS`); 1 m off the walkway's
  line the rays meet its hinters.
* sdm_e34 (`BF_ROUTE`): from the valley (-49.9, 43.0, y 30.3) to the high ground at (2.0, -15.2,
  y 40.2), 21 entries up the ramp (31.4, 32.8, 34.4, 36.6, 38.2 m ...), 95.0 m for 78.6 straight.
  To the sniper walkway: no route (the island above).
* e34: from the start valley (-60, -70) to the plateau at (-10.4, -16.9, y 38.7), 83 entries,
  332 m for 73 m straight: the graph reaches the plateau only from the north-east, through two
  door points. The cost above.
* In bf_play (`BF_TEST_ROUTE`, the commands above, captured at 15 fps): on the tutorial the
  waypoint triggers' ten changes logged at t 0.00 (and the save point's, kind 0x1d, not kept, at
  0.07); the plan at 1 s the same route as bf_level's (33-69 us); Tex runs h_19d4b7b6 (2.33 s),
  h_0a6c9ab1, h_1862fb9d (4.00 s), h_f5015ea8 and h_e507cadc (each held a frame at the corner,
  a hinter 1.2 / 2.0 m along the right ray), h_ec00fa4a ... h_e4b186f7 (10.07 s), skips
  h_1a6a0ec5 (neither it nor the next has h_ed6a7490, and the corridor on was clear), and ends by
  h_187143b0 at 13.20 s, the walkway's points each reached within its own 1 m and the last two
  within 2 m; his own point follows by the 6-link search. From above (yaw 0,
  pitch -1, 16 m) he crosses on the dark strip between the two lava pools at 3-5 s and follows
  the walkway round the south pit, never in lava. On sdm_e34 he climbs the ramp from the valley
  (y 30) to the high ground (y 40) in 19.7 s, the look-ahead skipping eight points (sdm_e34 has
  only 24 no-skip points) and the corner rule holding him to 0.6 m of h_1195e686, where a placed
  object stands in the next leg's left ray.
* #338: bf_play loaded e01's data before the map's, and a collision surface loaded first wins its
  name, so on every map but e01 a blocker or hinter named as one of e01's had e01's shape (the
  tutorial's five hinters, 5 of sdm_e34's 14 blockers, 1 of e34's 17: maps.md, "Which level's
  shapes"). These walks had to run with `BF_LEVEL=<the map>`, which takes the default data from
  the map itself: without it the tutorial's walk held at h_1862fb9d from 4.9 s to the end of a
  20 s capture, the left ray meeting a hinter of e01's shape that bf_level's collision doesn't
  have (the same two rays replayed in bf_level, `BF_ROUTE_PROBE`, meet nothing). The map's own
  data loads first now: the commands above run as written, the tutorial's walk ends at 13.20 s
  and sdm_e34's at 19.67 s (eight points skipped), and the tutorial's with `BF_LEVEL=tutorial`
  gives the same log line for line (the search's microseconds aside) and the same frames to the
  noise.

* **The squad on routes (#335)**, `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps. The times
  below are game seconds (the demo's clock); the take's were measured off its video at first,
  wall time, which ran at 0.71 of the game's clock in xemu. The e13 staging of
  `todo/242-ai-routes` take01 (Tex on the north rim facing south, the three on the south rim):
  `BF_MAP=e13 BF_TEST_GOTO=37.3,25.6,37.3,25.6 "BF_TEST_WARP=0,me,37.3,25.6,180;
  0,brutus,44.6,73.8;0,flint,43.1,75.8;0,hawk,45.7,75.6" BF_TEST_NO_HOSTILES=1 BF_ROUTE_LOG=1
  BF_ORDER_LOG=2 BF_LIQUID_LOG=1 BF_CAMERA_PITCH=-1.2 BF_CAMERA_DISTANCE=70`, 900 frames:
  * each member's first route goes round by the west shore through the take's points in the
    take's order, h_0f143f8f (37.3, 73.6), h_141d6e35, h_1fe8f610, h_fb8c63b3, h_f079fb96 (9.3,
    73.6), h_e9ba9a68, h_f2b3cbd2, h_029e248d (2.3, 57.6), h_efb7f462, h_f3024e36, h_1df21e91,
    h_1c93b982, h_ef15c18a, h_f6599a9a, h_1469ef23, h_1b3a3faf, h_e41a0444, as the game's three
    (notes.md: Brutus's and Hawk's 20 points, Flint's 19); then Brutus's to his place beside
    h_e7a4bd37, and Flint's and Hawk's on by h_ed1d34d2 to Tex's own point h_f6146568, where the
    game's three ended: their places, 8 and 11.5 m out from him, lie on the slope down to the
    lava, 3.3 m under his feet (more than 2 m: since the fix round a place no one could stand on
    is given up for the leader's point). bf_level's route from Brutus's spot to Tex's point is the take's 20
    to the point, 116.8 m for 48.7 straight. Two differences: the game's three keep their own
    point as the first entry, which the demo drops (both corridor rays clear from Brutus's spot
    in the demo: `BF_ROUTE_PROBE`; what the game's met isn't found); and the corners;
  * the corners: the game held its members (take01's brain+0x9dc) at h_f079fb96, h_e9ba9a68,
    h_029e248d and h_1469ef23, as the demo does, but not at h_f2b3cbd2 or h_f3024e36, where the
    demo's rays meet e13's hinter band h_09910e17. The band runs along the west walkway with
    both points just beyond it (h_f2b3cbd2 0.21 m, h_f3024e36 about 1.5 m: `BF_ROUTE_FACES`);
    the game's members kept about a metre off it (their avoidance, FUN_000255d0's feelers, not
    done) and the game's Flint skipped h_f3024e36 from h_efb7f462 (his steering target h_1df2
    from 17.08 s), where the demo's, steering straight at the points, slide along the band, are
    held at those corners and stop 0.6 m short of them against it. The stuck handling then moves
    them on (with the fix round's 5 cm: round 1 counted only an equal distance as none closer,
    and the review saw a centimetre's jitter start the time again for good): Brutus's detour at
    9.7 s at h_f2b3cbd2 takes the -0.7 m lane (h_1021c864, h_ee8f64b4, h_f79d787b, h_0ec4ef6b,
    h_eaf9b73f, h_fec3851a, h_ef0679a7), the lane the game's Brutus took after his own detour,
    and he is at his place at 21 s; Flint's at 11.8 s takes it too, and he is at Tex's point at
    24.1 s; Hawk is detoured twice at each of h_f2b3cbd2, h_029e248d, h_f3024e36 and h_1df21e91
    and put on the point the third time (FUN_000425b0, the route's current point), about 10 s a
    corner, and is at Tex's point at 58.1 s. The game's three were there at 16.5 s (Flint), 18.5
    s (Hawk) and 23.0 s (Brutus) by the game's clock (the 25-32 s once given here was the take's
    wall time). With `BF_ROUTE_NO_HINTERS=1` (round 1's run; not the game: its rays meet the
    hinters) the holds match the game's but for the brief ones at h_f079fb96, h_e9ba9a68 and
    h_029e248d, h_f3024e36 is skipped from h_efb7f462 and h_f2b3cbd2 reached at 2 m as the
    game's were, and all three arrive in 14-18 s: the pinning is the missing feelers' (#344);
  * none enters lava: `BF_LIQUID_LOG` never fires (the routes' safety net never acts);
  * **with #345's ground speeds** (the squad's run and dash by the types' speeds: the dash 8.5-8.7
    m/s while the leader is farther than 14 m and 6.8-7.0 inside it, the run 7.8-7.9 and 6.2-6.3;
    play.md "Ground speed"; `BF_ROOT_SPEED=1` runs the staging as the numbers above were): the
    same first routes round the west shore through the same points, and never into lava
    (`BF_LIQUID_LOG` silent). The arrival times move: Flint's route ends at Tex's point at 18.6 s
    (24.1 s; the game's 16.5 s), Hawk's at 57.5 s (58.1 s; the game's 18.5 s), and Brutus reaches
    his place beside h_e7a4bd37 at 42.5 s (21 s; the game's 23.0 s). The members' dash on the
    first straight is the take's (Brutus 8.72, Flint 8.62, Hawk 8.52 m/s over 5 s on the test
    floor from 80-90 m off the leader; take01's 8.59-8.62, 8.53-8.55, 8.42-8.46). The pinning at
    the band's corners isn't removed, it moves from member to member with the speeds (#344's
    feelers): Brutus is now stuck at h_f2b3cbd2 (10.1 and 15.1 s, through on the second detour), at
    h_f3024e36 (22.1, 25.1 and 28.2 s, put on the point the third time) and at h_1df21e91 (31.5,
    34.6 and 37.7 s, put on it), where he was detoured once; Flint is stuck once, at h_f2b3cbd2
    (8.5 s); Hawk as before at the same four corners (put on the point at 22.1, 32.1, 42.7 and
    52.2 s).
  * **with #344's avoidance** (the feelers, the push, the pace ramp from a stand and down, the hard
    turn, the game's stuck test; the same command with `BF_AVOID_LOG=1`; `BF_ORDER_LOG=2`'s 15 Hz
    rows against take01's 10 Hz watch rows, both from the release, "within 10.5 m of Tex" as the
    arrival, the take's state 3 being 9.8-10.0 m off him): Flint 17.6 s (the take's 16.6), Hawk
    18.1 s (18.6) and Brutus 18.9 s (23.1; the game's Brutus took a detour lane after a stuck, 145 m
    to his 128, the demo's none). Along the west walkway, the first time each is past z 64, 50 and
    38 and then on the north rim (x over 20): Hawk 8.9, 11.5, 14.1, 17.2 s (the take's 9.1, 11.7,
    13.8, 17.5), Flint 8.4, 10.8, 13.6, 16.7 (7.8, 10.1, 12.3, 15.8: the game's Flint ran the first
    straight on the lane of points at z 77.6, his own point carried from before the freeze, the
    demo's spawn picks h_06130f19 on the z 73.6 one, whose hinter band 1-4 m south of it his feelers
    meet twice at the dash), Brutus 9.3, 13.2, 14.9, 18.0. No stuck at all (21 before, 6 of them
    put on the point), no detour, no lava (`BF_LIQUID_LOG` silent), 116 frames of feelers steering
    and 70 of the push, two hard turns' walks. The corner holds: h_f079fb96, h_e9ba9a68,
    h_029e248d and h_1469ef23 for all three, the take's four, plus one member each at h_f2b3cbd2
    (Hawk) and h_f3024e36 (Flint), which the game's didn't hold; each held 0.3-1.4 s and passed
    within its 0.6 m. A track plot against `BF_ROUTE_FACES`' hinter faces shows the three down the
    west walkway on the band's outer side as the take's, a metre or so off it, wavering at
    h_e9ba9a68-h_f2b3cbd2 where the take's curve is smooth (the push and the square steers turn
    the demo's members, which move along their facing). With `BF_AVOID_OFF=1` (the plain direction
    alone, the rest as #344's): Hawk pinned at the band for good, 8 stucks, Flint at 40.5 s.
  * **with #344's fix round** (the move's turn, FUN_00019af0; the stuck test's `progress`; the
    follow's hold within d0): Flint 18.8 s (the take's 16.6: +13%), Hawk 17.1 s (18.6: -8%) and
    Brutus 21.9 s (23.1: -5%), no stuck, no detour, no lava. The weave (the tester's measure, 0-25
    s from the release, the track resampled every metre: changes of side over 0.3 rad a 10 m, the
    turn a metre): Flint 0.42 and 0.158 rad, Hawk 0.25 and 0.123, Brutus 0.74 and 0.182; take01's
    0.28 and 0.138, 0.36 and 0.133, 0.42 and 0.155; the first round's 0.27 and 0.117, 1.18 and
    0.183, 1.13 and 0.192. The spread off the track's 9 m mean grows (0.77, 0.68 and 0.88 m; the
    take's 0.59, 0.59 and 0.67; the first round's 0.66, 0.60, 0.67): the turn's wider arcs. Over
    six staging shifts (all three 2 m west, swapped round, 2 m toward the pit, bunched, 14 and 24 m
    west along the rim) all arrive in 13.7-20.5 s, with one stuck in three of them and three in
    one, each a detour at a corner of the band (h_f079fb96, h_e9ba9a68, h_f2b3cbd2), none put on a
    point; with `BF_AVOID_FREE_TURN=1` none (the first round's numbers to the frame).
* Move To across the pit (e13, the squad and Tex on the south rim: `BF_MAP=e13
  BF_TEST_GOTO=44.3,73.6,44.3,73.6 "BF_TEST_WARP=0,me,44.3,73.6,0;0,brutus,45.7,75.8;
  0,flint,42.3,75.8;0,hawk,43.0,72.0" BF_TEST_NO_HOSTILES=1 "BF_TEST_ORDER=hawk,move,34.3,29.6@2"`,
  1800 frames): a 20-entry route round the west shore to h_e7a4bd37 on the north rim. The same
  four corners pin her against the hinter band; at each the stuck handling detours her twice and
  the third stuck puts her on the point (20.4, 31.1, 42.5 and 52.9 s), each detour's drop to a
  stand climbing back a notch a second (MOVE_OUT's step); she reaches the point at 58.3 s, 1.9 m
  off: GUARD and e635ecc6. Never into lava. With #345's speeds (her dash 6.8 m/s, 8.5 x 0.8,
  as take08's plateau 6.84; the clips' 7.2 before) the same four corners pin her (put on the point at
  20.5, 30.6, 42.1 and 52.2 s) and she reaches it at 57.9 s. (The review's run, round 1: held 3.69 m short of
  h_f2b3cbd2 for good, the jitter starting the stuck time again.) With #344's avoidance: no stuck,
  held at five corners for a moment each, two hard turns' walks; she reaches the point at 20.2 s,
  18.2 s after the order, 1.8 m off: GUARD. With the fix round: held at seven corners for a moment
  each, no stuck; she reaches the point at 22.7 s, 20.7 s after the order, 1.8 m off. Brutus and
  Flint, sent to places 7 and 9 m out in front of the standing Tex that lie across each other's
  way, run south side by side 1.2 m apart: each turning toward its place brings the other into the
  half in front, whose push turns it back, and at 200 degrees a second neither breaks away (with
  the free turn both do). They run on past their places (Brutus 9 m) down the slope, where the
  feelers' blockers stop them, stuck at 3.9 and 4.1 s, 16 and 13 m off Tex; held there, as the
  fix round's first build did, they stood so for good. Beyond d0 the follow plans again: Flint
  reaches his place, 8.6 m off Tex, and Brutus, stuck twice more short of h_f7672590, is put on it
  (9.1 s) and holds there, 10.1 m off Tex, 3.1 m short of his place. With the second fix round's
  pace near the route's points (`pace_near`, `BF_ROUTE_LOG=2`): the level dropped to a walk by
  the corners at h_f079fb96, h_f2b3cbd2 and h_1b3a3faf and a notch at h_1c93b982 (FUN_000587d0,
  each with the timer at 1.5 s, then the ramp's notch a second), so she walks most of the west
  walkway, whose corners lie 5 m apart; a run from 5.9 m off the point, a walk from 2.9 m; she
  reaches it at 27.9 s, 25.9 s after the order, 1.9 m off, no stuck (no footage of this one).
  On e34 (#318's take08 recipe
  with Tex on her way, `BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999
  "BF_TEST_ORDER=hawk,move,-89.5,-87.8@1;hawk,move,-70.8,-79.5@10"`): a 5-entry route past Tex,
  at h_ff113737 3.13 s after the order, GUARD 1.8 m short (#318: 3.1 s and 1.8 m; take08 4.2 s,
  1.9-2.0 m), the same in round 1 and the fix round; with #345's speeds 3.26 s and 1.9 m short
  (her dash 6.8 m/s), the same route, no stuck; with #344's 3.86 s and 1.9 m short (Move To's ramp
  from a stand: a walk, the run after 1 s, the dash after 2), the same with the fix round; with
  the second fix round's pace near the route's points 4.13 s and 2.0 m short, the take's 4.2 s
  and 1.9-2.0 m: she dashes from 12.2 m out, runs from 5.1 m out and walks from 2.7 m out (the
  watch rows at 7.5 Hz; take08's 5.6 and 2.8 m at 10 Hz, FUN_00058590's 6.0 and 3.2 m), where she
  had dashed to 1.7 m.
* The follow on e34 (`BF_TEST_GOTO=-83.39,-82.79,-30,-80,27 BF_TEST_GO_AT=4 BF_TEST_NO_HOSTILES=1
  BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, 300 frames, Tex running east past the rocks):
  8-12 plans a member from 4.5 to 16 s (4-5 at most in any 2 s), 1-2 holds each at points short
  of a place behind a rock (Object 464 by h_f6632343), 0.6-0.9 s each; of the 87 watch rows in
  that time, standing 8 (Brutus), 10 (Flint) and 5 (Hawk) (round 1: 3, 5 and 4; with d3 as the
  hold's distance 21, 15 and 12, Brutus ending 10 m behind Tex); all three at their places when
  Tex stops (6.4-10.7 m off him), no stuck. With #345's speeds (the follow's dash 8.5-8.7 beyond
  14 m of Tex, 6.8-7.0 inside; Tex's run 7.8 m/s where it was the clip's 4.8): 7-8 plans a member
  in that time, 0-1 holds each, standing 12 (Brutus), 6 (Flint) and 4 (Hawk) of the 87 rows, all
  three at their places when Tex stops, 6.5, 7.9 and 10.9 m off him, no stuck, no detour. With
  #344's avoidance: no stuck, no detour, the same final distances (6.5, 7.8, 10.8 m), the members
  never closer than 1.3 m to each other (0.72 before: the push), the lag behind Tex while he runs
  a mean of 5.8 (Brutus), 9.5 (Flint) and 10.1 m (Hawk) against 5.0, 8.4 and 6.7 (Hawk's back-step
  to the point behind her at 5.4 s, #335's follow, comes a little sooner, and her hold there
  ends later). With the fix round: no stuck, no detour, the same final distances (6.4, 7.8 and
  10.8 m), the members never closer than 0.81 m, the lag a mean of 5.9, 12.9 and 11.3 m: Flint's
  route at 6.9 s goes back to his own point behind him, and turning round at 200 degrees a second
  takes over a second at more than 2.5 rad/s, which ends in the hard turn's walk (8.3 s), 21 m
  behind Tex at most; Hawk's back-step the same. (The tester's sprint recipe, `BF_MAP=e34
  BF_SCRIPT=off BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4
  BF_TEST_SPRINT=4`, 375 frames: a mean lag of 3.7, 5.0 and 7.0 m, the first round's 3.6, 5.1 and
  7.8, main's 3.4, 6.9 and 8.8; never closer than 0.78 m.) The second fix round changes neither
  (the follow's walk has no pace near its points). Both back-steps are #335's follow: main makes
  them too (Hawk's route at 5.5 s to h_e90ebcab, Flint's at 6.2 s to h_e0098c3d, toward the
  place's point behind them), turning round at once; the game's follow routes to the leader's own
  point (state 0) and wouldn't (#331). Over the rows before each one's back-step the lag is 4.8
  (Flint) and 8.3 m (Hawk), main's 5.4 and 8.3; Brutus, who has none, 5.9 (main 5.0: a 2 s walk
  at 8.7 s, the long rays' pick held to the target's side, FUN_00018b50). A plain U-turn at 200
  degrees a second (0.9 s) doesn't set off the hard turn's walk; Flint's and Hawk's went on round
  their corner points (h_e0098c3d, closing to 0.6 m) past 1 s. (Tried and dropped, not the game's:
  the pace near the route's points for the follow too. Hawk then goes through sdm_e34's door at a
  dash, but on e13 she arrives at 25.9 s, 39% over the take, m07's Flint walks 23 m at rest, the
  sprint recipe's lag grows to 6.4, 9.7 and 7.6 m, and the e34 run's stays at 5.9, 11.3 and 10.5.)
  Stand Ground holds where ordered while Tex goes 32 m
  off, Fire At Will and Cover Me follow (`"BF_TEST_ORDER=hawk,guard@3;flint,fire@3"`, round 1).
  sdm_e13 (#1's recipe, the controlled Brutus walking into the lava): the three stay on the
  shore's points, 8-9 m from him (round 1). The tutorial's marines (brain.md's recipe) follow Tex
  on routes (round 1).
* The leader in the lava (#335's review: e13, `BF_TEST_GOTO=143.9,68.7,120,100 BF_TEST_HEALTH=5000
  BF_TEST_NO_HOSTILES=1`, 450 frames: Tex runs into the lava at 6.4 s and goes round and round his
  goal there, within 2 m of (125.3, 95.1)): from 10 to 30 s no plans at all and none of the three
  moves more than 0.2 m. They wait on the shore at his own point, 11-14 m from him, as their
  places are in the lava. The review's run (round 1): 34-49 plans each in those 20 s, 11-13 in
  2 s, each pacing 2-5 m; keyed on the leader with the 3 m hold but the place's nearest point,
  1 in 2 s at most, but Hawk ran to and fro between h_ebea0733 and h_00bdf10f, 21 m apart. None
  enters the lava (`BF_LIQUID_LOG`). The same with #345's speeds: no plan from 10 to 30 s, none of
  the three moves more than 0.2 m. The same with #344's fix round: no plan and no stuck from 10
  to 30 s, none moves more than 0.2 m.
* #344's review recipes, on the fix round's build (the tester's, `BF_MUTE=1 BF_BACKGROUND=1`):
  * the tutorial's lava yard (`BF_MAP=tutorial BF_TEST_GOTO=-15.6,-87.4,-15.6,-87.4
    BF_TEST_NO_HOSTILES=1`, the base gate's ten waypoint triggers fired, `BF_TRIGGER_FIRE`, and its
    panels pressed, `BF_TEST_PANEL`, Tex sent down the yard by `BF_TEST_ROUTE=73.1,-44.6,16`, 3000
    frames, and from 10, 24 and 40 s, 2400 frames): the two marines' routes from the upper ledge
    go by the jump h_0656d0f4 -> h_03090779; each is stuck there three times, the detours finding
    no other way down, and put on h_03090779 below (Cogburn 75.0 s, Stout 77.5 s in the first);
    within 8 m of the yard's end (73.1, -44.6) at 87 and 90 s, 82 and 84 (10 s), 94 and 97 (24 s),
    and at 112 s for Stout (40 s; Cogburn stops at his place 8.4 m off it at 98 s). main: 84-86,
    78-81, 91-92, 100-107 s; #344's first build: held at the edge for good;
  * m07_a (`BF_MAP=m07_a BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-341.5,39.1,-341.5,39.1
    BF_TEST_ROUTE=-270.4,38.0,2`, 1000 frames, Tex standing from about 20 s, his test walk held at
    a corner by an object):
    from 30 to 66 s Brutus, Flint and Hawk walk 0.0, 0.8 and 0.0 m (main 1.1, 8.9, 0.3; #344's
    first build 124.7, 47.3, 1.7), each held where the stuck handling acted near Tex;
  * sdm_e34's door pair at a run and at a dash (`BF_MAP=sdm_e34 BF_CHARACTER=0 BF_TEST_NO_HOSTILES=1
    BF_TEST_GOTO=-44.6,18.6,-44.6,18.6 BF_TEST_ROUTE=-58.3,3.4,2 "BF_TEST_WARP=0,hawk,-47,21,90;
    0,flint,-46,23,90;0,tex,-47,16,90"`, and `BF_TEST_SPRINT=2`, 900 frames; Brutus led through
    h_f519ca12 / h_117d5fb1 into the building): from 15 to 60 s Flint, Hawk and Tex walk 0.6, 1.5
    and 1.8 m at a run (main 0.5, 0.1, 0.3; the first build 1.0, 0.2, 2.1), and 0.7, 42.5 and 1.6
    m at the dash (main 1.5, 0.2, 1.1; the first build 8.6, 3.2, 0.8): Hawk, on through at the
    first door point at 6.3 s, swings wide of the doorway at 200 degrees a second, is stuck outside
    it at 8.9 s, and the detour takes her round the building by its other doors, at her point at
    22.9 s (with `BF_AVOID_FREE_TURN=1` through in 2 s, 3.2 m walked after 15 s);
  * e34's Fire At Will then Cover Me (`BF_MAP=e34 BF_TEST_GOTO=-60,-79,-60,-79,30
    "BF_TEST_ORDER=all,fire@1;all,cover@6"`, 450 frames): two stucks (Brutus and Hawk, 8.4 s),
    each held at once where it stood near Tex (main none; the first build six and a place given
    up).
* **The game's follow (#331)**, the recipes above on its build against main's (`b3f6a9c`), `BF_MUTE=1
  BF_BACKGROUND=1`, the lag the mean over the 15 Hz rows with the leader moving (the takes' own
  numbers are in [squad.md](../squad/the-follow.md)):
  * the e34 run (`BF_TEST_GOTO=-83.39,-82.79,-30,-80,27 BF_TEST_GO_AT=4`, 300 frames): a lag of
    14.7 (Brutus), 13.5 (Flint) and 13.3 m (Hawk), at most 16.3, 16.1 and 16.0, never ahead of Tex,
    and at rest 9.3, 9.0 and 9.2 m behind him; main 5.9, 12.9 and 11.3, at most 13.7, 21.6 and 16.5
    (Flint's and Hawk's back-steps to their places' points behind them), 17-21 rows of 87 ahead of
    him, at rest 6.4, 7.8 and 10.8. 20 plans in all (main 28). The lag is the game's: the chase
    holds a member about 14 m behind a leader at its own top speed (take04's 13.7-14.4 m); main's
    was less because its places ran beside and ahead of him;
  * the sprint recipe (`BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4 BF_TEST_SPRINT=4`):
    15.3, 16.7 and 13.7 m, at rest 8.2, 9.2 and 9.1 (main 3.7, 5.0 and 7.0, 15-21 rows ahead);
  * #242 take01's staging on e13: arrivals within 10.5 m of Tex at 18.8 (Flint), 17.1 (Hawk) and
    21.9 s (Brutus), main's to the frame (the game's 16.6, 18.6 and 23.1: +13%, -8%, -5%), the paths
    116, 113 and 126 m (main 124, 121, 137; the game's 109, 112, 145); each stops 9.4-9.7 m off him
    (the take's state 3 at 9.8-10.0 m); no stuck, no lava, no warp;
  * e13rev (Tex running the route to the south rim, `BF_TEST_ROUTE=44.3,73.6,2` from the north rim,
    700 frames): a lag of 16.2, 15.2 and 14.6 m, at most 33.9, 23.8 and 18.3 (main 23.9, 21.7 and
    20.6, at most 46.6-49.9; #344's tester's main before #344: 18.5, 11.5 and 14.1); one corner
    hold each at h_1df21e91 (#344's Brutus held there four times, 5 s), Brutus held twice at
    h_029e248d; at rest 9.3, 9.4 and 8.5 m; no stuck, no lava;
  * take08's Move To: at the point 4.13 s after the order, 1.9 m short (main 4.13 s, 2.0 m);
  * m07_a (Tex standing from about 20 s): from 30 to 66 s the three walk 0.6, 0.2 and 1.0 m (main
    0.0, 0.8, 0.0), at rest 9.0, 9.4 and 9.2 m off Tex where main's Flint and Hawk stood 1.4 and 2.5 m
    off him (its places' fallback to his point); no stuck;
  * the leader in the lava (e13, `BF_TEST_GOTO=143.9,68.7,120,100 BF_TEST_HEALTH=5000`): from 10 to
    30 s the three walk 0.2, 4.0 and 0.2 m on the shore, 11-15 m off him, none into the lava (main
    0.9, 0.6, 0.1). Without `play_follow.rs` `in_burning` (the demo's: no route to a leader's position
    in a liquid that burns) they walked 105-134 m in those 20 s, 500 plans: the chase plans every
    update with him farther than d3 from his own point, and the route of a member on his own point
    is his position, in the lava, which the shore guard held them out of as they slid along it;
    sdm_e13's Brutus in the lava: the three on the shore 9.3-10.6 m off him (main 9.4-11.6);
  * the tutorial's lava yard (t335test's recipe): the marines stuck at the jump's edge and put on
    h_03090779 below as main's (Cogburn 74.3 s, Stout 77.0 s), within d2 of Tex at the yard's end at
    85.7 and 88.9 s (main's within 8 m of the end at 87.1 and 90.1 s), Cogburn to a cover point;
  * sdm_e34's door pair at a run: from 15 to 20 s Brutus, Flint and Hawk walk 0.0 m (main 0.2, 0.0,
    0.6); e34's Fire At Will then Cover Me with its enemies: 3 stucks (main 6), the squad fights
    (16 rows firing, main 76; main's Hawk died at 12.5 s, here no one: the members hold 9-15 m off
    Tex, not 4-9);
  * the test map's floor (no world points, `--test BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,0,0,14
    BF_TEST_SPRINT=2`): the straight chase, 12.3-13.4 m behind Tex's dash, at rest 9.2-9.3 m (main
    1.3-3.4 m running beside him, at rest 6.3-10.7). A goal Tex can't reach (`0,0,0,60`: he's
    held at z 17.3) keeps his stick held, so he's running to the follow and the three close
    to 1-2 m of him (FUN_00020ad0 reads the motion, not the speed: take04's Tex too, pushing at the
    rocks).
* **#363: the command heading, the view and the motion** (`BF_MUTE=1 BF_BACKGROUND=1`, the 15 Hz
  watch rows and `BF_GAIT_LOG=1`; main is `a897a30`):
  * **The fight** (`BF_MAP=e34 BF_SCRIPT=off BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4`,
    620 frames; Hawk is killed at 3.3 s by the start valley's enemies in both builds): Flint and
    Brutus keep up with Tex's run through the fight, dashing at 6.9-8.6 m/s and running at 6.3 m/s
    with the gun on their target, and are within 10 m of him from 11.9 s (he stops at 11.1 s),
    holding 9.1-9.3 m off; main's Flint crawled at 0.4 m/s and was within 10 m at 26.7 s, its
    Brutus at 20.0 s; before #344, 18.3 and 17.6 s. Their gait changes once or twice a second
    moving with a target (main: 12-13), their clip layers 1.1-1.6 deep (max 3-4, a crossfade;
    main 10-11), their view, body and move within a degree or two of each other on a straight. From
    13 s they lose sight of their target where they hold and turn to its last place, gun up (the
    contacts' turn, play_brain.rs). With Fire At Will (`"BF_TEST_ORDER=all,fire@0.5"`) Flint and
    Brutus stay within 8 m of Tex from his stop (5.9 and 6.7 m off at 11.9 s), Hawk, who follows
    Flint, 12.3 m off; main's two within 8 m at 23.3 and 21.7 s, its Hawk never. On the test map's
    fight (`--test BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,0,0,70 BF_TEST_GO_AT=3 "BF_TEST_NPC=0.5,shrike
    soldier,enemy,fight,-22,25" BF_TEST_NPC_HP=1.5,1,9999`) the three chase Tex dashing, guns turned
    onto the Shrike by the upper body (frames 102-114 looking back, `BF_VIEW_YAW=3.1416`), 0.2-0.9
    gait changes a second, layers 1.1 deep; Hawk backs off for a moment on a reversal (12% of her
    moving frames). They reach Tex where he's held at z 17 by a pillar, in the Shrike's fire, and
    Brutus and Hawk die there; main's crawled 20-42 m behind and lived.
  * **The sway**: the paths' wobble (the tester's measure, the 10 Hz track's heading about its 1 s
    mean) in the fight 12.8 (Flint) and 5.7 degrees (Brutus) RMS at 1.4 and 1.0 crossings a second,
    on the no-hostiles runs (the run to -43, -80; the sprint; take04's) 5-19 degrees at 0.9-1.7;
    #33 take04's 13-18 degrees at 0.7-1.25; main's 5-17 at 0.7-2.2. Where it remains, the members
    run bunched within 1.5 m (the run to -43, -80: Flint with Hawk 1.4 m ahead of him for half a
    second at a time), and the push (the game's stage) swerves the move until the neighbour is
    out of the half in front: the squad's start bunched 1.3-3 m apart is #363's out of scope.
  * **Unchanged**: take08's Move To (GUARD 4.06 s after the order, 2.0 m short; main 4.13 s, 1.9 m;
    the take 4.2 s, 1.9-2.0 m); the e34 run (a lag of 12.1, 13.6 and 13.5 m, at rest 9.3-9.9; main
    14.7, 13.5, 13.3 and 9.0-9.3), the sprint recipe (11.8, 13.2, 13.9 m; main 15.3, 16.7, 13.7), the
    walk (9.8-10.5 m as main's) and take04's (13.0, 18.8, 18.6 m, at rest 9.0-9.3; main 13.0, 19.7,
    18.6 and 8.7-10.6); the yard (the marines put on h_03090779 at 75.7 and 78.7 s, within d2 of Tex
    at the yard's end at 87.1 and 90.1 s; main 77.0 / 74.3 and 88.9 / 85.7); the tutorial's follow
    (to the row); m07_a (from 30 to 66 s the three walk 0.0, 0.6 and 0.6 m; main 0.6, 0.2, 1.0); the
    leader in the lava (at most 0.1 m walked, no lava); sdm_e34's door pair (at most 1.2 m walked
    from 15 to 60 s, run and dash; one stuck at the run, main two); the door step (to the frame as
    main's); with the door shut the first stuck ends the Move To at 9.8 s, 5.5 m short (main's went
    round to a stuck at 14.0 s, 8.5 m off); e34's Fire At Will then Cover Me with its enemies: no
    stuck (main 2), Hawk killed at 5.7 s after a knock-down (main's lived).
  * **e13**: take01's staging, arrivals within 10.5 m of Tex at 18.5 (Flint), 18.8 (Brutus) and
    17.5 s (Hawk) (main 18.8, 21.9, 17.1; the take 16.6, 23.1, 18.6), two stucks (Flint 8.3 s and
    Brutus 9.2 s short of h_f2b3cbd2, a detour each onto the -0.7 m lane), no lava; the weave 0.08,
    0.16 and 0.44 changes of side a 10 m (Flint, Brutus, Hawk; main 0.45, 0.82, 0.27; the take 0.28,
    0.42, 0.36). Over three staging shifts as well (all 2 m west, swapped round, bunched) the mean
    arrival is 20.3, 17.6 and 17.3 s (main 16.9, 18.8, 18.4): Flint late in two (22.1, 24.7 s), each
    time held at the west walkway's band. There a member's move can be caught facing straight away
    from its next point: the left feeler's tip (1.0-1.2 m at a walk) meets the band every other
    frame, so the steering alternates between the feelers' square steer and the way to the point
    behind it, each frame's turn reverses the last (the rate dropped at each change of side,
    FUN_00019a20), and the move holds where it is till the stuck handling moves it on; the view,
    which has both on one side, turns away meanwhile, so a side step and a backpedal show. The
    game's logic, met where the demo's members hug e13's hinter band (#344's open end). e13rev
    over five stagings: a mean lag of 11.9, 20.0 and 21.5 m (Brutus, Flint, Hawk; main 13.6, 17.3,
    22.9), each build with runs where one falls 34-49 m behind at the walkway's corners.
* The door step (#346; sdm_e34's pair h_f519ca12 (-49.2, 11.8) / h_117d5fb1 (-53.1, 10.0), door 0
  h_06537687 between them, opened by the anim-trigger h_134c9934 on a detector; Hawk's Move To to
  h_f4e3da16 (-56.0, 8.2) inside: `BF_MAP=sdm_e34 BF_CHARACTER=0 BF_TEST_NO_HOSTILES=1
  BF_TEST_GOTO=-36,20,-36,20 "BF_TEST_WARP=0,me,-36,20,90;0,hawk,-38,18,90;0,flint,-37,22,90;
  0,brutus,-38,21,90" "BF_TEST_ORDER=hawk,move,-56.0,8.2@3" BF_ROUTE_LOG=1 BF_DOOR_LOG=1
  BF_AVOID_LOG=1`, 300 frames): her route goes through the pair; the detector opens the door at
  4.33 s as she nears it, she reaches the first door point at 5.07 s, the corridor between the
  two is clear by then (the leaves past 0.6 of their 1.21 s clip), "on through", the far point at
  5.73 s, GUARD at the Move To point at 6.60 s. With `BF_TEST_DOOR_SHUT=h_06537687` (the same
  run): at 5.07 s "the corridor between them blocked: stands 0.5 s, its feelers off 1.5 s"; no
  feeler steers until 6.60 s (1.5 s on), then they turn her along the leaves; with the route's next
  entry its last, the first stuck (8.73 s) ends it (FUN_00042280) and the Move To is over, GUARD
  7.6 m short. (Tex warped beside the door instead, the follow walks Hawk through it at 1.13 s with
  the door already open.) The frames of these runs show the tower beside the building, not the
  door: the logs are the check.
* The cost (the plans' times in the logs, opt-level 2): 4-45 us a search, at most 225 points off
  the heap (e13), the worst frame three plans in 90 us.
* `BF_TEST_PRISM=1` (e34, Move To with Tex on Hawk's line, a squadmate on the other way round):
  the blocker route runs; with two friends standing on the two ways round it, as read, swaps
  between them every update, each new route starting from the member's own point, which holds
  it there, and growing by the flood's way each time (FUN_00041810 appends the old route's tail
  after the first entry of the next point).
* Unit tests (`cargo test`): the route to a position beside a point (through the point or a
  neighbour, the position alone from the point itself); the flood's order (a link further each
  pass, the last met first, the radius, the avoided points); the catch-up; the stuck detour (the
  three points off for one search, the bits back, the pace level) and the third stuck's warp onto
  the route's current point; the stuck test without a margin (#344: a jitter that comes back to
  its best never stuck, a drift away stuck after 2 s; since the fix round one asking to move and
  held to the bit stuck, one standing by choice not), and the follow's place given up on the
  third stuck; the stuck handling's marks for the follow, which holds within d0 of its leader and
  plans again beyond it; the pace near a route's points (the second fix round: FUN_00058590 at,
  within and beyond its two distances, a walk asked not raised; FUN_000587d0's square, bent and
  gentle corners, moving its way or off it, beyond 2.4 m, standing, past the walk distance in 3D,
  at a stand's level; on a route, the corner and then the end, and a Move To's distances against
  the follow's); the pace ramp (a notch every 0.17 s in FOLLOW, every 1.0 s on a Move To, once the
  drop's timer runs out; no higher than asked; the move at the lower of the two) and, since #344,
  its start at a stand with 0.3 s and its steps down after the mode asked; the follow's points on
  the leader's side, the nearest the place first. The avoidance (`cargo test --bin bf_play
  avoid`): the feelers' lengths by the pace and the speed factor (x0.945 for a factor of 1.05, as
  read, not max(1, 0.9 f)), a side block's square steer, a wall ahead's long rays and the
  hysteresis (the missed long ray's slot as read), both sides blocked, the dash's drops, walkable
  terrain, the tilt; FUN_00018b50's side; the push; the side-step; the plain direction; the move's
  turn (FUN_00019af0: at once from a stand, else at most 3.49 rad/s with the rate eased a third of
  the way a game frame, the same ease over two at 15 fps, the heading within a degree, a stand's
  direction as it is). The door
  step (#346): blocked between two door points, a stand and the feelers off; clear, nothing; a door
  point followed by a plain one, nothing.

* **#336, the enemies on routes** (`BF_MUTE=1 BF_BACKGROUND=1`, `BF_BRAIN_LOG=1 BF_ROUTE_LOG=1
  BF_GOAL_WATCH=1`): the tutorial with every spawn router fired and Tex away at the mine field
  (`BF_TEST_GOTO=89,-55,89,-55`, 90 s): every walk home arrives but 9's, terrorists 21, 13, 6 and 3
  among them (main stalled 21, 13 and 6 against the base's and the third building's walls; 17's
  router isn't among those fired, so it isn't in this run): 21 at 7.7 s, 3 at 9.4, 13 at 10.5, 0
  at 10.8, 6 at 28.0 s (66.5 m, a detour round a stuck corner at 23.5 s) and then its patrol; 9
  stops at the base building's door, shut in this staging (the stuck detour on the route's last
  entry), its leash then on: 7.2 m off its home with a 7 m area it walks PATH_TOWARD_HOME and stops
  inside. The three patrols (2, 18, 6) walk their chains to the end of the run. m09_a's start
  (40 s): the two Hounds home at 15.8 s (main: stalled 23 and 29 m short). e40 with its 28 spawn
  triggers fired (90 s): eight walks home arrive, as before the home area; a patroller with a 15 m
  area (h_f490d291, chain iD 5, a loop reaching 19.5 m from its home) is called back by
  PATH_TOWARD_HOME at its fourth point each round and starts the chain again (the game's rule as
  read; no e40 footage of it). The test map (`-- --test`, a Filthy Mutant h_0b5f6e21 set to fight
  45 m from Tex): it dashes in and stands at 25.9 m (under its 26.25), fires from 4.7 s. e23 staged
  as todo/132-mutants' F1 (Tex at the level's start, warped to (-57.7, 64.8) at 25 s once the four
  live mutants are home): the home areas are the data's (the Uglys' 9 or 7 m across, 7 m high; the
  Molotov Mutant's 100 by 20), the Uglys within their stand-off stand and fire (16.8 m), and the
  one on the ledge (home (-62.4, 32.5), 7 m) approaches round the ledge, leaves its area, walks
  back by PATH_TOWARD_HOME and does it again: take08's 832f6110 does ATTACK_VARY then 19 s of
  PATH_TOWARD_HOME the same way, and in take07 the Ugly's PATH_TOWARD_HOME starts as it crosses its
  9 m edge (9.05 m) and ends 7.01 m inside (FUN_000235d0's 7.0). The lava yard (take08's staging):
  its two run home round the pit by the walkway; who sees Tex there was unchanged by #336 (the
  hinter band stopped the demo's sight ray; since #244 it doesn't, [brain.md](../brain/verified.md)
  "Verified"). Unit tests: `bf::nav` (the far pick without patrol points, FUN_0004c2e0's pick, a route's
  length, a chain's two ends) and `play_goals` (the budget, the stop short, the leash, the margin
  and the edge, which goals approach).
* **#336, the first patrol against take07** (`BF_TEST_GOTO=44,91,34.4,94.3 BF_TEST_PANEL=a,1
  BF_COMBAT_LOG=1 BF_SHOT_LOG=1 BF_GOAL_WATCH=0.5`; times from tutorial_show_patrol's start, the
  game's clock 2.27 s): main and this build see Tex at 16.0 s, 35.3 m off. Main fires where its
  leader stands and first hits him at 18.3 s. Here the leader approaches by a route of 10 entries
  (40.1 m, under its budget of 62 m), holding fire as the game's approach does
  ([brain.md](../brain/goals-on-routes.md) "Its goals on routes"), and stops 25.9 m off at 17.7 s; the demo's ray from
  its eyes to Tex's chest has been cut since 16.5 s, so it doesn't fire there: it pursues at
  20.2 s, sees him at 20.8 s (24.4 m off), fires from 21.1 s and first hits him at 22.7 s. The
  second sees him at 17.3 s, stops 26.2 m off, pursues at 21.5 s and stands 22.4 m off at 22.9 s.
  take07's first hit comes at 14.2 s, before either build sees him: most of the gap is the sighting
  (the game's ray between the characters' points, #197 / #244) and the patrol's walk (its leader
  at its second point 0.8 s after take07's, [brain.md](../brain/verified.md) "Verified"); take05's patrol
  closes to 2-5 m, its cover and strafes #26's.
  **Since #244** (the sight ray between the two characters' head nodes, hinters and blockers left
  out, [brain.md](../brain/sight-and-contacts.md) "The sight ray"), with Tex on the plate's spot: the leader sees him at
  14.3 s, 40.0 m off, stops 26.2 m off at 16.7 s with its ray clear (no pursuit; the second, 1 m
  behind, loses the ray from 18.1 s and pursues at 21.8 s), fires from 17.1 s and first hits him at
  21.7 s. take07's blip shows at 12.8 s, ~43 m out, and its leader stands there firing (hit at
  14.2 s) instead of walking in: that is the attack goal's approach. At take07's own spot the
  tester has main seeing him at 12.66 s and this build at 11.60 s, and the terrorists' rounds
  striking the slope (4 hits in 40 s against main's 27): [brain.md](../brain/verified.md) "Verified", #244.
  **Since #244's fix round** (the AI fires only with a clear muzzle corridor and aims at the hips,
  [brain.md](../brain/sight-and-contacts.md) "The muzzle corridor"), on the plate's spot: the leader sees him at 14.3 s,
  40.0 m off, stops 26.2 m off at 19.0 s and fires from 17.1 s; the second, 26.1 m off behind it,
  holds fire from 19.4 s with the leader in its corridor, posts EVT_SHOT_BLOCKAGE at 23.1 s (its
  muzzle line blocked 3.8 s), walks in 2 s (GOAL_PURSUE, a reposition's stand-in: it comes to 21.2
  m) and fires from 25.3 s; in main's run the two fired through each other (3 hits by one on the
  other). First hit on Tex 21.7 s (main 22.7 s), 11 in 43 s (main 15). At take07's own spot the
  leader's approach ends 0.9 m from where main's does (it sees Tex 1.1 s earlier, on another
  stretch of its patrol), with a ridge in its line: 32 of its 120 rounds strike it and Tex is hit
  once in 43 s (main: 4 of 120 and 32 hits), whether it aims at the hips or the chest
  (brain.md "Verified"). **With #384's cover** (DEBUG1 takes cover against Tex, it being
  outnumbered) the first patrol's two no longer walk in to their stand-off at take07's spot: they
  take the cover points h_1e75e701 and h_1772d797, 43.3 and 40.8 m from Tex, where take07's
  leader stands, and no round strikes the slope; only a terrorist whose pick finds no point (the
  second at the plate's spot) still approaches to 26 m (brain.md "Verified", the fix round on
  #384's cover).

* **#383, the link walk against take07 (F2b)** (todo/132-mutants; the Ugly 83302960 crouched on its cover
  point h_157fd721 (-66.62, 37.66), Tex teleported to 1.12 m, game seconds): `BF_AWAY=-66.57,37.81,-0.48,-1.01`
  gives the route h_157fd721 (the own point, in the cone: dot 1.11), h_0dcdca06 (-68.07, 37.00; +1.6 m), h_1eb2ac93
  (-70.76, 36.11; +2.9 m) and h_f90e1c8c (-73.63, 33.99; +3.6 m), 8.0 m between its points, taking the
  link h_0dcdca06 (dot 0.90) before the last one in the list, h_01510b25 (dot 1.12, straight away). The
  footage's Ugly backs off from 11.92 to 15.0 s (BACKSTEP, 1.82 m/s) along exactly that line: its 8 Hz
  positions stay within 0.39 m of the polyline of the four points (0.00-0.14 m along the first two legs,
  0.18-0.39 m round the corner at h_1eb2ac93, where the walk turns toward the next point within 1.5
  m: the model's cut there is 0.33 m; with 0.6 m it would be 0.17, with 2.0 m 0.40), through (-68.00,
  37.06) at 12.91 s, (-70.00, 36.06) at 14.14 and (-71.36, 35.34) at 15.0. The goal then gives way to PATH_TOWARD_HOME (the Ugly is 10.6 m from its
  home, the 9 m area), as the demo's brain does outside its area. The demo's staging (e23, Tex warped
  2 m north of the third Ugly, no cover): TOO_CLOSE at once, the walk to h_f0804248 (-69.3, 53.5)
  by a one-entry route (its own point is in front of it, so no entry, and the first leg from it is
  9.1 m), BACKSTEP at 1.21, 1.69, 1.80, 1.83, 1.84 m/s over the first 1.3 s (the footage's
  0.50, 1.13, 1.46, 1.61, 1.71, 1.76, 1.79, 1.80, 1.81, 1.82 from the first BACKSTEP row), facing Tex
  and firing at him (hits of 2.3-4.5 on him about every 0.5 s while it backs off, the footage's -1.7 to -4.3
  every 0.5-0.6 s), done 8.0 m from him after 3.5 s (post at 21.93, done at 25.47). The tutorial's wall sentry (a guard, BACKPEDAL
  at 4.4 m/s = 0.8 x its 5.5) falls back along the walkway through three points, 6.9 m between them
  (3.2 m from its own point, behind it, to the first). The re-arm and the hold: on the test map
  a Filthy Mutant set to fight with Tex walking after it (`BF_TEST_WALK=1`, 2 m off at the start) posts
  at 0.0 s, holds the tests that pass 1.5 s apart while its walk runs (1.47 and 3.00 s: no post, no
  restart), runs its 7 m straight to the route's end at 4.20 s and posts again at 4.53 s (then 9.13,
  13.73 and 18.33 s: a walk of 4.1-4.2 s a post; the first build re-posted every 1.53 s and started the
  walk again each time). On e23 a second warp of Tex 1.55 m from the third Ugly at 23.45 s, its timer
  out and its walk running, gives "not posted" at 23.47 s, and the walk goes on to its route's end
  at 25.80 s. **With the cover goals merged in (#384):** F2b's staging on e23 (the Ugly crouched at
  its cover point h_157fd721, Tex warped 1.5 m from it) takes the same route from the cover point,
  h_157fd721, h_0dcdca06, h_1eb2ac93, h_f90e1c8c, after the cover's hook has kept the first post
  and the 8 m rule has failed the cover; the numbers are in [brain.md](../brain/verified.md) "Verified".
  Unchanged: the first patrol (the `BF_BRAIN_LOG` log line for line, no post in 15 s of fight), the tutorial's routes
  recipe (every line the same but the search's microseconds), sdm_e34's autopilot (298 of 300 frames
  identical, main's own 296-297), e23's F1 (no post).
