*Part of [play/](README.md), the former page `play.md`.*

### Ricochet and pass-through (#238)

`src/bin/play_bolts.rs` (`Bolt::after_hit`, the bolts) and `src/bin/play_rays.rs` (the instant
rays), from default.xbe's FUN_00225440 (a bolt's flight) and FUN_00230860 / FUN_00230400 /
FUN_002318e0 (an instant ray's hits); the bullet block's three values in `bf/weapon.rs`
(`ShotData::object_hits` h_1835f15e, `character_hits` h_fe4f3035, `pass_through` h_11caf57b:
FUN_0018a8d0's bullet +0x2c / +0x30 / +0x34). The data: the Feral Cutter and the PU Cutter n1 5,
n2 0, no pass; the Ion Beamer n1 5, n2 0, pass; the Rail RVR and CLVR n1 1, n2 500, pass; every
other gun 0, 0 (the Saryl-45's pass flag has nothing to spend).

* **A bolt** (FUN_00225440 after a sweep's hit, which is dealt first as any hit is): with the pass
  flag it goes through a game object that isn't a character while n1 is left (one off it), a
  character while n2 is (one off that), on from the hit point with that thing left out of its next
  sweep (+0x40); anything else ends it, the terrain always (it has no object). Without the flag,
  while n1 or n2 is left, it bounces: a hit on an object takes one off n1 (off n2 once n1 is out),
  any other hit one off n2, and its way is turned about the surface's normal (d - 2 (d.n) n); with
  the count at 0 it ends. So **the cutters' blades bounce off up to five objects** and stop on a
  character or the terrain, and **the Ion Beamer's bolt goes through up to five objects** and
  stops at the first character or the terrain. "Objects" are the ray result's kind 2 that aren't
  characters (FUN_000dd8e0: a body of PID 0): every placed object's parts (the e40 beach's
  standing stones, buildings, crates, gates), a broken object's remains, a Sentry; the demo counts
  the test floor's pillars and the test tools' pieces so too. The terrain blocks (PID 2) are kind
  1: a blade ends on e40's cliff. Each thing on the way is dealt its hit (its effects, hole,
  sound; a breakable takes the roll).
* **An instant ray** (FUN_00230860): one hit, or n1 + n2 with the pass flag, the latter through
  the multi-hit ray (FUN_00230400, nearest first, no blockers, no liquids, at most 256 hits).
  Its collector (vtable 0x39efc8, hit method 0x2b7af0) keeps every hit the ray caster hands it,
  merging none by their body (FUN_002b8ea0 keys its table by the record), so a closed object is
  met twice, where the ray goes in and where it comes out, and the terrain's floor once
  (`Arena::ray_shot_all`: each face it crosses, a face's twin and a shared edge once; the test
  floor's pillars and placed breakables likewise, play_bolts.rs `floor_all`). A character is met
  once (`units_met`: the loop passes over one it has met, 0x230bf9-0x230c14, neither dealt nor
  counted). Each hit takes one off a budget, a character not hit yet n2, anything else n1, and is
  dealt with a roll of its own (FUN_002318e0 -> 0x22d0b0); after the hit that drives a budget
  below 0 the ray stops. So **the Rails go through every character on their line and stop at the
  far side of the first object they enter**, dealing it two rolls, and never reach a character
  behind it; fired into the ground they go through the floor (one crossing) into a character
  below and on to the next surface. Before #287 a body was met once, so a Rail went through a
  crate or a gate leaf into the enemy behind it and stopped at the second wall: #287's takes show
  otherwise (below). The demo deals a Rail's hits in its shot's frame (the game's ray deals them
  as it's cast), where its other rays land theirs at INSTANT_SHOT; the shot's `dist` becomes
  where it stopped (its beam's length, § Shots). One hit (every other gun) stays play.rs's.
* Not done: a bolt's push on the loose object it ends on and a ray's push x3 (the demo's loose
  pickups are knocked by the lines, `Sweeps`); FUN_00230400's x3 on a character while the shooter
  is scoped (no pass-through gun has a scope); the squad AI's ray given up at a squadmate in the way
  (EVT 34, plan §4.2); a bolt or a ray at a liquid's surface (the game's single ray stops there,
  the demo's go on; since #5 only an instant ray splashes, levels.md "What a liquid plays").

`BF_SHOT_LOG=1` prints a bolt's counts at its launch ("n1 5 n2 0 passing through"), each bounce
("bounces off it along [0.561, 0.190, 0.806] (n1 4 n2 0 left)") and each pass ("passes through it
(n1 4 n2 0 left)"), and each of a pass-through ray's hits ("ray of Rail RVR 6.0 m out: hits Shrike
Soldier (NPC 1) (walls 1 characters 499 left)") and where it stops ("; it stops there", or "goes
on to its full range, 90.0 m"). `cargo test --bin bf_play bolts` checks the bolt's rule (five
bounces off objects and the end on the sixth, on the terrain and on a character; n2 once n1 is
out; five passes and the end at a character); `cargo test --lib multi_hit` the multi-hit ray's
crossings (a box twice, twinned or not and along a face's diagonal; once from inside it; the flat
floor's ground once).

Against #287's takes (`todo/287-weapon-shots/`, take-log.md's hit logs: every take-damage the
game dealt, in order), on e34 with Hawk (`-- --test BF_MAP=e34 BF_CHARACTER=2
BF_TEST_WEAPON="Rail RVR;Foley 356 Tact" BF_TEST_GO_AT=100 BF_TEST_INSTANT_KILL=0
BF_TEST_NPC_HP=1.0,all,300 BF_TEST_TRIGGER=3.0-3.08,5.6-5.68 BF_SHOT_LOG=1 BF_COMBAT_LOG=1
BF_SCENERY_LOG=1`; for the HUD add `BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`,
the start's letterbox off, camera.md, the tester's: take01's recipe gives the same hits with them):
* **Two enemies in line** (take01: `BF_TEST_GOTO=-48.0,23.35,-49.61,30.16 BF_CAMERA_PITCH=-0.10
  BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-49.61,30.16;0.3,h_e96829a0,enemy,frozen,-50.72,35.96"`):
  each ray hits the Heavy 6.0 m out, the Soldier 12.0 m out, then the base gate's leaf (breakable
  56, 40 hp) where it goes in, 17.8 m out (walls 1 -> 0), and where it comes out, 18.2 m (-> -1),
  and stops there. take01's every shot: the Heavy, the Soldier (or the Soldier alone), then the
  leaf 7f1e2a00 twice. Before #287 the demo's met the leaf once and went on to a wall 38 m out.
* **An enemy behind a crate** (take02: `BF_TEST_GOTO=-43.47,20.97,-41.89,32.36
  BF_CAMERA_PITCH=-0.16 BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-41.89,32.36"`): the ray
  meets the yard's supply crate (h_e45338bd, the take's, breakable 460) 7.0 and 7.7 m out, two
  rolls (16.36, 15.79), and stops: the Heavy behind isn't hit. take02: the crate (its hp raised to
  500) took two rolls a shot, five shots, and the Heavy none. The demo's crate (25 hp) breaks on
  the first shot, and the next goes on into the Heavy.
* **An enemy behind the gate's leaf** (take03: `BF_TEST_GOTO=-52.2,34.5,-52.2,41.3
  BF_CAMERA_PITCH=-0.10 BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-52.2,43.6"`): the leaf 6.2
  and 6.6 m out (14.43, 16.59), then the ray stops; the Heavy 2.3 m behind isn't hit, as take03's
  ten leaf hits and none on the Heavy. The leaf, dead at 0 hp, stays standing and blocking (#263).
* **Into the ground** (take04's spot, `BF_TEST_GOTO=-48.0,23.35,-48.99,27.64
  BF_CAMERA_PITCH=-0.45`): the floor 4.7 m out is one crossing (walls 0) and the ray goes on to its
  90 m, so a character under the floor would be hit, as take04's Heavy 2 m under it took every
  shot. The test hooks can't put one there (`BF_TEST_NPC` stands it on a floor): not run.
* take05 (the Heavy with two objects behind it, from the side) and take17 (the CLVR, a Shrike
  then the object 7f20f200 behind it twice) follow the same rule (#66).

Verified (`-- --test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=4.5,20,4.5,20
BF_TEST_NPC_HP=1.0,all,300 BF_SHOT_LOG=1 BF_COMBAT_LOG=1 BF_CAPTURE_FPS=30`, dm Shrike Soldiers
frozen on the line):
* **Rail RVR, two Shrikes in line** (`BF_TEST_WEAPON="Rail RVR" BF_CAMERA_PITCH=-0.14
  BF_TEST_NPC="0.3,shrike soldier@dm,enemy,frozen,4.5,13;0.3,shrike soldier@dm,enemy,frozen,4.5,8"
  BF_TEST_TRIGGER=2.0-2.08,3.5-3.58`): each shot hits both, 6.0 and 11.0 m out (14.43 and 14.64 x
  their forearms 0.5 x 1.4, their stamina shields taking it), goes through the ground 67.8 m out
  (its one wall, one crossing) and on to its 90 m.
* **Rail RVR at a crate** (supply crates put down 4 and 16 m out and a Shrike 10 m out,
  `BF_TEST_PLACE="0.5,supply crate,4.5,16,0;0.5,supply crate,4.5,4,0"`, #287's rule): the first
  shot meets the near crate 2.5 and 4.2 m out (walls 0, then -1) and stops; it breaks, and the
  second goes on into the Shrike 9.1 m out and stops at the far crate's far side, 14.5 and 16.3 m.
  Before #287 the first shot went through the near crate into the Shrike.
* **Ion Beamer through two crates** (`BF_TEST_WEAPON="Ion Beamer" BF_CAMERA_PITCH=-0.2`, crates
  4 and 8 m out, Shrikes 14 and 19 m out): the bolt passes the first on its sweep at frame
  3 (n1 5 -> 4) and the second at frame 6 (-> 3), each taking the launch roll 14.44, and stops at
  the first Shrike 12.8 m out at frame 9; the second Shrike isn't touched.
* **Feral Cutter off a pillar** (`BF_TEST_WEAPON="Feral Cutter" BF_TEST_GOTO=4.5,-3,5.1,-3.8
  BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=2.0-2.08,3.5-3.58`): the blade meets the pillar at (9,
  -9)'s face 5.9 m out on its first sweep and bounces, [0.561, 0.190, -0.806] -> [0.561, 0.190,
  0.806]; frames 66-75 show its ribbon bend at the pillar and go off to the right. It flies on to
  its life's end (19 frames, 75 / 120 m/s).
* **PU Cutter on e40** (Brutus, `BF_MAP=e40 BF_CHARACTER=0 BF_TEST_WEAPON="PU Cutter"`): from 10 m
  (take01's spot, `BF_TEST_GOTO=-31.57,152.2,-32.02,151.98 BF_CAMERA_PITCH=-0.02`) every blade ends
  on the cliff, the terrain (material 5), with no bounce, as take01 shows none; held fire one
  every 8 frames (3.75 a second; the take ~3.8). From take05's 45 m spot with the crosshair on the
  standing stone (`BF_TEST_GOTO=0.01,167.27,-0.46,167.10 BF_CAMERA_PITCH=-0.10`) the blades bounce
  off it (an object, material 8) up and to either side, as take05's streaks leave the stone at
  10.2 s; in the demo the stone is 23.5 m out where take05's was ~40 m (the take's spot isn't
  reproduced exactly).
