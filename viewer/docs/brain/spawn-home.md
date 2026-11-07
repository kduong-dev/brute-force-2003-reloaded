*Part of [brain/](README.md), the former page `brain.md`.*

## The spawn: its home

A spawn-trigger's copy gets its brain as it's made (FUN_000dfd90 -> FUN_00017b60, then
FUN_002096e0 -> FUN_00018510):

* **Its home** (brain+0x1ec): the trigger's `home-point` (h_1b747c8d). Without one, the trigger's
  world point h_131de144, kept when it's within 5 m of where the copy stands (FUN_0014f7a0 tests
  25, the square); otherwise the game takes a nav point near the copy, the demo the copy's own
  spot.
* **Home first.** With a home-point the brain is sent there at once: FUN_00018510 posts
  EVT_RETURN_HOME (31), whose personality entry (pri 1.5, over EVT_BASE's 1) is GOAL_RETURN_HOME
  (its update FUN_00034690), home within the point's own h_e6564e8a when it has one over 0
  (FUN_00026d80; the wall sentry's wpoint1493: 1 m), else 2 m. Its pace is the trigger's
  `locomotion` (h_f27d26c9, +0x84: FUN_00018510's 5th argument, stored as the goal's +0x94 at
  0x18750), the speed mode FUN_0004d970 takes while the goal is on: 0 and 2 a walk (the stick at
  half), 3 a run, 4 the dash, and 1 the stick at 0.01, read as staying put (the tutorial's 31 and
  32, who spawn on their homes).
* **Its home area**: the trigger's h_f63b86a2 across and h_062fbaf0 high round the home
  (brain+0x214 / +0x218: each at least 7 when both are given, else 100 and 20), on from the spawn
  for a copy without a home-point and once home for one with (since #336: "Its goals on
  routes").

The tutorial's: the opening's wall sentry, TUTORIAL_terrorist_5, walks 10.5 m west along the
walkway to wpoint1493 as the level starts (its trigger at FIRST_UPDATE); the base's patroller,
TUTORIAL_terrorist_6, is sent 55 m into the base to wpoint1123 before it patrols (since #336 by a
route of 18 points, 66.5 m, home 27 s after its spawn); the lava yard's 29 and 30, the second
building's 3 and 17, the base's 7, 9, 13 and 21 run home (locomotion 3); the rest walk or are
home already.
