*Part of [brain/](README.md), the former page `brain.md`.*

## Test hooks

* `BF_BRAIN_LOG=1`: each brain at its spawn (home, its pace there, sight, field of view,
  accuracy, gun bonus, personality), its base goal and the draw, its patrol route (iD, points,
  where it starts), each point reached, home reached or given up short of (and how far), each
  goal's start, refusal and end (#336), each contact seen (with the heights over their feet the
  sight ray ran between, their hips and its muzzle, #244), the muzzle corridor to each contact
  each time it changes (the head's and the hips', reason 4: "The muzzle corridor"),
  EVT_SHOT_BLOCKAGE when it posts, its target, contacts forgotten (and why), a shooter it learns
  of, stopping to fight. The copies are named as the triggers' log names them. `BF_SIGHT_LOG=1`
  prints the corridor lines for the squad's AI and the NPCs the same way.
* `BF_TEST_AIM_HOSTILE=<s>`: from that time the controlled character's camera turns onto the
  nearest living hostile it can see (within 120 m, nothing of the world in between: its chest,
  else its head over cover) and it aims and fires at it: the takes' aim helper
  (todo/28-tutorial/notes.md), to stage a fight. With `BF_TEST_GOTO` it walks on meanwhile,
  toward the hostile while it aims (the walk follows the camera). It runs after the other input
  hooks (its fire wins over `BF_TEST_TRIGGER`'s) and after the free camera's input gate. With a
  gun that has a scope it aims through the scope's first level, as the other hooks' aim does
  (play.md, The scope), and leaves the scope it went into once no hostile is in sight.
* `BF_GOAL_WATCH=<s>` (#336): every s seconds a row for each level character with a brain
  (not following) and each test map NPC set to fight: its goal (ATTACK_FALLBACK and MOVE_BACK
  since #383), where it is and its facing (degrees, the demo's yaw), its own point, its route's
  entry, its target and how far, its pace, and the motion it plays ([nav.md](../nav/test-hooks.md), test hooks).
* `BF_TEST_MOVE_BACK=<s>[,<name>[,<x>,<z>]]` (#383): at s seconds a level character with a brain (not
  following) whose name contains `name` (any when empty), the nearest to (x, z) when they're given,
  starts GOAL_MOVE_BACK, which nothing draws yet; with a target in sight it backs off facing and
  firing at it. `BF_BRAIN_LOG=1` also logs the post (and one kept by the cover's hook or held while
  the walk runs), the draw, the start with its route (or why it was refused) and the end (and why);
  `BF_ROUTE_LOG=1` the route's points.
* `BF_TEST_FALLBACK_REFUSE=1` (#383): ATTACK_FALLBACK's start refuses as if the link walk found no 7 m
  way, so the entry's draw comes to its second goal, ATTACK_COVER's pick (a level's geometry rarely
  gives a refusal where a cover point could be had: e23's ledge is a dead end, but its Ugly has no
  cover point inside its home area).
* The tutorial's panels (#224) stop the clock until closed: `BF_TEST_PANEL=a,1` closes each
  after 1 s (in the first patrol's run below the WEAPONS panel comes up at ~17 s).
* `BF_COVER_LOG=1` (#384): the cover goals' events: each pick (the point, how far, from the
  target, its cone and bits), DEBUG1's halves with FUN_00036970's numbers (the hit points' share,
  its own threat at 25 m, its target's, the sum), the states, the peeks, the exits, the hits that
  blow the cover. `=2` also why each cover point the flood met was passed over, every retry of a
  DEBUG1 waiting in ATTACK_VARY, and a row every half second of each cover goal (`cover t=...`:
  its state, point, how far off it, the hold, the time at the point, the peek and re-pick timers,
  the crouch, the motion, where it is, its facing and how far its target).
* `BF_TEST_TURN=<s>[,<r>]` (#384): at s every level character with a brain within r m (60 by
  default) of the controlled character turns to face it at once: the takes' `!yaw` staging, so
  its sensor can see him.
* `BF_TEST_PUT=<s>,<name>,<x>,<z>[,<heading>[,<hp>]][;...]` (#384): at s the level character whose
  brain is named `name` (as BF_BRAIN_LOG names it) is put on the floor at x, z (its heading in
  degrees, 0 facing -z; its hit points, now and at most): the takes' `!place` and `hp`.

```sh
# the first patrol from its plate: the script, the walk past the camera, the fight
BF_MAP=tutorial BF_TEST_GOTO=44,91,34.4,94.3 BF_TEST_PANEL=a,1 BF_BRAIN_LOG=1 BF_COMBAT_LOG=1 BF_TEST_AIM_HOSTILE=14 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the first patrol walked to and fro, Tex far away at the mine field
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router_spawn_first_patrol,USER1,1" BF_BRAIN_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#336) every spawn router of the tutorial fired, Tex away: the walks home by route, the patrols
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router103,ENTER_DETECTOR,1;trigger_router38,ENTER_DETECTOR,1;trigger_router50,ENTER_DETECTOR,1;trigger_router22,ENTER_DETECTOR,1;trigger_router473,ENTER_DETECTOR,1;trigger_router_spawn_first_patrol,USER1,1;h_020415dc,ENTER_DETECTOR,1" BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#336) e23 staged as todo/132-mutants' F1: Tex warped in once the four live mutants are home
BF_MAP=e23 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 "BF_TEST_WARP=25,me,-57.7,64.8,180" BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_GOAL_WATCH=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) take06's staging: Tex warped 4.2 m from the Ugly's take position at 24 s (the warp sets him on the block there; he drops to the ground), the Ugly put there facing away at 24.5 s and turned to him at 25.5 s, no squad: its cover at h_157fd721
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="24,me,-63.59,51.01,26" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0" BF_TEST_TURN=25.5,10 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) the same with 500 hit points and Tex's aim helper from 33 s (the other Ugly near Tex given 1 hit point, the helper's first kill): three hits blow the cover
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="24,me,-63.59,51.01,26" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0,500;24.5,h_f5f8f2ca,-51.5,55.5,,1" BF_TEST_TURN=25.5,10 BF_TEST_AIM_HOSTILE=33 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_COMBAT_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) the same with Tex walking off south-east from 37 s: flanked
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-53,31 BF_TEST_GO_AT=37 BF_TEST_WARP="24,me,-63.59,51.01,26" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0,500;24.5,h_f5f8f2ca,-30,90" BF_TEST_TURN=25.5,10 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) take08's staging (F1): the Ugly 83302960 at its take position, the mutants within 20 m turned to Tex
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="25,me,-57.7,64.8,0" BF_TEST_PUT="25,h_eef1a370,-64.12,46.83" BF_TEST_TURN=25.5,20 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383 with #384) take07's staging (F2b) on take06's: the Ugly takes h_157fd721 and crouches by 30 s; Tex is warped 1.5 m from it at 36 s: the cover's hook keeps the first EVT_TGT_TOO_CLOSE, the cover fails on its 8 m rule, the next, 1.5 s on, draws ATTACK_FALLBACK, which ends the cover
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="24,me,-63.59,51.01,26;36,me,-66.09,38.82,25" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0" BF_TEST_TURN=25.5,10 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_ROUTE_LOG=1 BF_GOAL_WATCH=0.25 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the same with ATTACK_FALLBACK refused: the draw comes to ATTACK_COVER's pick (BF_COVER_LOG=2 says why no point passes: Tex stands between the Ugly and them)
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="24,me,-63.59,51.01,26;36,me,-66.09,38.82,25" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0" BF_TEST_TURN=25.5,10 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=2 BF_TEST_FALLBACK_REFUSE=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the fallback on e23: Tex warped 2 m north of the third Ugly (home (-71.6, 46.2), facing north
# at 20 s, so he's inside its 60 degrees) at 22 s; the squad off, or it joins in
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP=22,me,-71.6,44.2,180 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_GOAL_WATCH=0.25 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the tutorial's wall sentry (TUTORIAL_terrorist_5, a guard personality) with Tex 2 m from its post
BF_MAP=tutorial BF_NO_SQUAD=1 BF_TEST_GOTO=89,-55,89,-55 BF_TEST_WARP=8,me,-49.6,-10.0,270 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_GOAL_WATCH=0.5 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the test map: a Filthy Mutant set to fight 2 m from Tex falls back straight, firing
BF_TEST_GOTO=4.5,30,4.5,30 BF_TEST_NPC="0.5,h_0b5f6e21,enemy,fight,4.5,28" BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_GOAL_WATCH=0.5 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play -- --test
```
