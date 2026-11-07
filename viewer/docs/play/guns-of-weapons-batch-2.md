*Part of [play/](README.md), the former page `play.md`.*

### The guns of weapons batch 2 against their takes (#53, #65, #67)

No tester's spec was written for these; each was measured here from its takes (`todo/<n>-*/`,
the frames pulled with ffmpeg at 60 fps, the HUD's counts and the watch logs in each notes.md).
The takes' spots are reproduced on their levels with the test map's tools (`-- --test`), since a
campaign level whose squad spawn-triggers fire at once (m03_a) moves the character from a
`BF_TEST_GOTO` start: `BF_TEST_FREECAM` straight down over the spot, `BF_TEST_TELEPORT` there, and
`BF_TEST_GO_AT=100` so the autopilot only aims (from the GOTO's start toward its goal).

**Feral Cutter** (#53, Brutus; `-- --test BF_MAP=m03_a BF_CHARACTER=0 BF_TEST_WEAPON="Feral Cutter"
BF_TEST_GOTO=-182.6,-43.5,-186.90,-52.59 BF_TEST_GO_AT=100
BF_TEST_FREECAM="0.3,-182.6,45,-41.88,0,-89.9" BF_TEST_TELEPORT=0.5 BF_TEST_FREECAM_OFF=0.7
BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=3.0-3.08 BF_SHOT_LOG=1`, take01's spot, the stone tower 10 m
out; the goal is #287 take07's first hit, where its blade left along bearing -115, 9 degrees right
of Brutus's facing -124, which the recipe aimed at before #287). m03_a's squad spawn moves Brutus
some 80 m from the GOTO's start, so every m03_a recipe here needs the three free-camera hooks: the
camera straight down 45 m up over its spot at (x, z + 1.62), where the crosshair's ray, 6 degrees
above the view's axis, meets the ground at (x, z); the teleport puts him there.
* Each blade strikes the tower (an object, material 3) on its first sweep, 0.1 s after the shot
  (take01: the HUD's 29 -> 28 at f11 of 5.20 s on, the white star at f17), with the white star of
  its explosion and sparks (§ Bolts, "The bullet's explosion"), its ribbon drawn from the gun to
  the tower and bending off it. Then, as #287 take07's breakpoint log has it, blade by blade
  (the bolt's hits, normals and new ways), the blade bounces off the tower's block (breakable 46,
  type h_e989e46e, the take's 7f1de400) along [-0.22, 0.19, 0.96] (the take: [-0.19, 0.22,
  0.96]), meets the flat wall up and to the left 5.4 m on at 0.2 s (breakable 281, the take's
  7f20ee00, h_0536a5bc 5.9 m on) and turns along [0.41, 0.17, 0.89] ([0.39, 0.20, 0.90]), then a
  third object 9.6 m on at 0.4 s (breakable 336; the take's 7f21ce00) along [0.99, -0.08, -0.13]
  ([0.98, -0.06, -0.17]), and flies on to its life's end. That second hit is take01's and take06's
  big flash up-left of the tower 0.1 s after the first (#287 take06 f12 and f15 at 30 fps; the
  demo's frames 98 and 101 of a capture at 30 fps). The demo's blades had flown 35-45 m to the
  ground: the recipe aimed at Brutus's facing, 11 degrees left of the take's blade, which met
  another facet of the tower. It leaves the cutter's scorch (h_1801e63d). Run again for #287
  (`BF_SHOT_LOG`): the block at 0.1 s, the flat wall (breakable 281, 7f20ee00) 5.4 m on at 0.2 s
  (take07 5.8 m, 0.1 s after the first), the far object (breakable 336, 7f21ce00) at
  (-179.10, 37.28, -28.12), 21.6 m on from the wall, at 0.4 s (take07 (-180.29, 37.53, -28.85)
  and (-179.05, 37.75, -28.21), 20 m on).
* At the flat wall at 45 degrees (#287 take08 / take09's spot: the recipe above with
  `BF_TEST_GOTO=-180.83,-39.6,-188.07,-46.9 BF_TEST_FREECAM="0.3,-180.83,45,-37.98,0,-89.9"
  BF_CAMERA_PITCH=-0.02 BF_TEST_TRIGGER=3.0-3.08,5.0-5.08`): each blade strikes the wall
  (breakable 281) at 0.1 s and bounces along [0.58-0.60, 0.11-0.14, -0.79..-0.81] (take09
  [0.58-0.60, 0.05-0.08, -0.79..-0.81]), ending on the terrain on its third sweep, 17.6 m past
  the wall where take09's ended 13.1 m past it: the demo's aim came in 2-4 degrees steeper
  (its blades' y 0.16-0.19 against the take's 0.10-0.13), so they leave it higher.
* A blade fired straight at a wall comes back past its owner and never hurts him (#287 takes
  10-13: Brutus's 105 hp all through, take13's second blade reaching the floor at his feet; the
  sweep's filter leaves the shooter out, FUN_00225440's +0x48). The demo's, at take13's spot
  (the map, character and weapon above, `BF_TEST_GOTO=-182.73,-47.2,-188.07,-46.9
  BF_TEST_GO_AT=100 BF_TEST_FREECAM="0.3,-182.73,45,-45.58,0,-89.9" BF_TEST_TELEPORT=0.5
  BF_TEST_FREECAM_OFF=0.7 BF_CAMERA_PITCH=-0.35 BF_TEST_INSTANT_KILL=0
  BF_TEST_TRIGGER=3.0-3.08,5.0-5.08,7.0-7.08 BF_SHOT_LOG=1 BF_COMBAT_LOG=1`, the tester's):
  each blade strikes the wall (breakable 281) 4.4 m out at 0.1 s, comes straight back down
  through where Brutus stands and meets the ground at his feet 5.8-6.6 m on at 0.2 s, with no
  hit on him. The recipe before (Brutus at -46.85, pitch -0.10, the first spot's free camera)
  sent its blades back past him, missing, so it didn't test the filter.
* Held fire one blade every 0.38-0.47 s (2.5 a second; take01: 26 -> 18 in 3.5 s).
* Reload (`--test BF_CHARACTER=0 BF_TEST_WEAPON=1545a4e4 BF_TEST_CLIP=3 BF_TEST_TRIGGER=2.5-7.0
  BF_AMMO_LOG=1`; the label also names the enemies' variant h_1bb3c9c7, so the hex): the clip
  3 -> 0 a blade every 0.4 s, the reserve drops 30 when the cooldown runs out 0.4 s after the
  last blade, and the clip reads 30 14 frames (0.93 s) later. take02's watch log: 3 -> 0 at
  2.735 / 3.136 / 3.537, the reserve 100 -> 70 at 3.939, the clip 30 at 4.944.
* Damage on a frozen Filthy Mutant 9 m out in front of the tower (take03's spot; m03_a has no
  Confed trooper type, take03's spawn: the map, character and weapon above,
  `BF_TEST_GOTO=-182.6,-43.5,-186.3,-51.7 BF_TEST_GO_AT=100
  BF_TEST_FREECAM="0.3,-182.6,45,-41.88,0,-89.9" BF_TEST_TELEPORT=0.5 BF_TEST_FREECAM_OFF=0.7
  BF_TEST_NPC="0.8,h_10a12c64,enemy,frozen,-186.3,-51.7" BF_TEST_NPC_HP=1.5,1,300
  BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.27 BF_TEST_TRIGGER=3.0-3.08,4.5-4.58,6.0-6.08,7.5-7.58,9.0-11.0
  BF_COMBAT_LOG=1`): every blade lands on its chest (x1), 7.9-10.0 (the roll 5.625-7.875 x the
  enemy step 1.4), one every 0.38-0.44 s in held fire; take03's taps took 9.1, 10.2 and 9.8,
  its held fire 8-11 each every 0.4-0.5 s. The white star flashes on the body (take03 f10 of 3.15
  s on), then the flesh's cutter hit.
* Not checked again (as before #53, generic): the dry fire and its hint, the switch (take02), the
  pickup (take04).

**PU Cutter** (#65, Brutus on e40; nothing of its own to build: the bolt's flight (#136), the
bounces (#238) and the explosion (#53) are the Feral Cutter's, with its own data: 100 m/s, 80 m,
4 a second, 10.5-13.5, plut_cuttertrail_ribbon in flight and plut_cutter_hit where it strikes.
The bullet's `<mirv>` count 3 is dead data: its mode is 0, NO_MIRV, plan §2):
* take01's spot, the cliff 10 m out (`-- --test BF_MAP=e40 BF_CHARACTER=0 BF_TEST_WEAPON="PU Cutter"
  BF_TEST_GOTO=-30.73,151.62,-31.65,151.23 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.06
  BF_TEST_TRIGGER=2.0-2.08,3.5-4.5`): an orange streak from the gun to the reticle, the white star
  at the cliff with spikes after it, the dark mark; held fire one every 0.27 s (3.75 a second;
  take01 ~3.8). The cliff (the terrain: no bounce) is 10.3 m (pitch -0.14) to 11.7 m (-0.06) from
  the muzzle there, just past a first sweep's 10 m at 100 m/s, so the star comes 0.2 s after the
  shot where take01's came at 0.1 s (the HUD's 49 -> 48 at f12-f13 of 5.20 s on, the star at
  f18-f19): the take's spot or aim is a little closer, not measured.
* Reload (`--test BF_CHARACTER=0 BF_TEST_WEAPON="PU Cutter" BF_TEST_CLIP=3 BF_TEST_TRIGGER=2.5-6.0
  BF_AMMO_LOG=1`): 3 -> 0 a blade every 0.27 s, the reserve 120 -> 70 0.26 s after the last, the
  clip 50 13 frames (0.87 s) later; take02's watch log: 2.742 / 2.942 / 3.243, the reserve at
  3.545 (+0.30), the clip at 4.447 (+0.90).
* Damage at 7 m (take03's spot: `BF_TEST_GOTO=-0.5,166.98,-7.0,164.4 BF_TEST_GO_AT=100
  BF_TEST_NPC="0.5,shrike soldier@e40,enemy,frozen,-7.0,164.4" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.3 BF_TEST_TRIGGER=3.0-3.08,5.2-5.28,7.4-7.48,9.6-11.0
  BF_COMBAT_LOG=1`): the taps took 15.7, 17.5 and 16.1 (the roll x 1.4 on a x1 part), take03's
  16.0, 17.9 and 16.1; held fire 14.8-17.3 a blade, each 0.1 s after its shot. Aimed higher
  (`BF_CAMERA_PITCH=-0.2`) the taps strike the head (x5, 78-88) and the held fire's blades pass
  just over it to the standing stone 25 m out and bounce off it, as take03's held fire missed
  the enemy and its blades left the stone to the right (why the held fire's blades go higher
  isn't looked into).
* A blade on the body (#287): take03 shows the white star, then a green blotch on the chest with
  sparks for ~5 frames (f13-f18 of 3.0 s on at 30 fps). The demo washed the whole Shrike
  cyan-white for 6 frames: the two cutter lights, the explosion's (cutter_hit's light-effect)
  and the flesh hit's (wmat_fleshit_cutter's, made by hand in play_fx.rs, `CUTTER_LIGHT`). Both
  left out, the demo's hit is take03's blotch; the explosion's alone left out, the wash stayed:
  the flesh's light did it. It stood 0.3 m off the body (the demo's), where its effect type puts it 2.70 m back along
  the shot (its light-effect light_hit_cutter, alignment 0, its emitter's tz 2.70: `Glow::back`;
  the explosion's, played from the data, already stood there). There, the body shows the blotch
  and a faint light, as take03 (the capture's frames 98-105 of `BF_TEST_TRIGGER=3.0-3.08
  BF_CAPTURE_FPS=30`). The LZRs' flesh light (light_hit_laser) moved the same way, 2.67 m back.
* The bounces off e40's standing stone (take05, take03): § Ricochet and pass-through above. A 25
  m hit comes on the third sweep, 0.3 s after the shot (take07: ~0.3 s), at the same damage (a
  bolt has no falloff; take07 17-18 a hit): the sweep's rule, not run again here.
* Not checked again (generic): the dry fire, the switch (take02).

**Rail RVR** (#67, Hawk; nothing of its own beyond the beam's spawn below: the pass-through
(#238) and the beam (#237), from its data: an instant ray, 90 m, 2.5 a second, 13.25-18.25
BALLISTIC with no falloff, n1 1, n2 500 and the pass flag, no scope; railhit_s with
light_hit_rail where it strikes, rgun_perp_s, rgun_side_s and rgun_beam in flight; on e34 the
start's letterbox hides the HUD: add `BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`
to the e34 recipes below for it, camera.md, the tester's):
* The beam (§ Tracers and beams) at e34's gate 10 m out (take01's spot, `-- --test BF_MAP=e34
  BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR;Foley 356 Tact" BF_TEST_GOTO=-52.76,31.02,-52.9,31.5
  BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.08 BF_TEST_TRIGGER=2.0-2.08`; the Foley as the second gun
  so the one stowed on Hawk's back doesn't hide the view): the spark at the hit, red-violet arcs
  from it down toward the gun turning blue over ~0.15 s, then the hit's blue sparks, as take01
  f19-f41 of 3.05 s on. Its rings were all spawned within a few metres of the gun, a bright tube
  round the camera; they're now strewn along the ray as the game's cube spawn puts them
  (`Compiled::game_spawn`), but stay brighter and many more than take01's and take06's faint few
  (§ Tracers and beams, "Not matched").
* At e40's cliff 25 m out (take06's spot: `-- --test BF_MAP=e40 BF_CHARACTER=2 BF_TEST_WEAPON="Rail
  RVR;Foley 356 Tact" BF_TEST_GOTO=-16.2,152.68,-16.69,152.58 BF_TEST_GO_AT=100
  BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=2.0-2.08`): the arcs from the gun up to the reticle and
  the white-blue glow at the muzzle (take06 f10-f17 of 2.5 s on, at 30 fps); the ray goes into
  the cliff (the terrain, its one wall) and on to its 90 m, the beam's length, hidden behind it.
* Held fire one shot every 0.4 s (2.5 a second; take01: one per ~0.4 s).
* Reload (`--test BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR" BF_TEST_CLIP=3
  BF_TEST_TRIGGER=2.5-6.5 BF_AMMO_LOG=1`): 3 -> 0 a shot every 0.4 s, the reserve drops 24 0.47 s
  after the last and the clip reads 24 16 frames (1.07 s) later; take02's watch log: 2.757 /
  3.158 / 3.560, the reserve 200 -> 176 at 3.961 (+0.40), the clip 24 at 5.065 (+1.10).
* Damage at 7 m on e34's Shrike Soldier (take03's spot: `-- --test BF_MAP=e34 BF_CHARACTER=2
  BF_TEST_WEAPON="Rail RVR" BF_TEST_GOTO=-52.76,31.02,-54.78,37.72 BF_TEST_GO_AT=100
  BF_TEST_NPC="0.5,h_e96829a0,enemy,frozen,-54.78,37.72" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.32 BF_TEST_TRIGGER=3.0-3.08,5.2-5.28,7.4-7.48,9.6-12.0
  BF_COMBAT_LOG=1`): with the crosshair on its legs, as take03's (its hits flash at the knees),
  every ray strikes a calf (x0.5) and goes on through into the ground: 9.9-11.5 a hit (the roll
  x 0.5 x 1.4), take03's nine 9.75-12.77. On the head (`BF_CAMERA_PITCH=-0.12`) 101-111 (x5).
  take03's run of misses in its held fire (11.8-13.5 s) isn't reproduced: the demo's stays on.
* Pass-through can't be seen in #67's takes (take01's wall has nothing visible behind it). #287's
  R1 and R2 takes settle it: § Ricochet and pass-through ("Against #287's takes": through every
  character, stopped at the far side of the first object it enters).
* Not checked again (generic): the dry fire and switch (take02), the pickups (take04, take05),
  the far enemies (take09, take10).
