*Part of [play/](README.md), the former page `play.md`.*

### Ground speed: the run and the dash (#345)

The game doesn't move a running character by its clip's root motion. FUN_0012c6c0 sets its ground
speed from the type's `speeds` (crouch walk, walk, run, dash; `CharacterType::speeds`) by the
motion type, plays the run and dash clips at rate 1, and the factor on it depends on who drives
the character. The demo moves the squad by root motion; its run and dash take the game's speed by
scaling the clip's root step (`src/bin/play_speed.rs`: the notes there have the code and the
footage, the rows in [code-map.md](../code-map/README.md) the addresses). The ticket was opened for "the
squad's dash is 10% fast (7.2-7.6 m/s against take01's 6.5-6.9)": that set the clips' constant dash
against take01's route means and its stretches with the follow's boost off (0.8 x the dash, 6.8-7.0);
the dash on a straight is **8.5**, and the demo's was too slow.

| m/s | Brutus | Flint | Hawk | Tex |
|---|---|---|---|---|
| the type's run / dash (`speeds`) | 7.8 / 8.7 | 7.9 / 8.6 | 7.9 / 8.5 | 7.8 / 8.5 |
| the clips' own root, the demo's run / dash before | 4.55 / 7.59 | 4.35 / 6.71 | 5.03 / 7.17 | 4.82 / 8.01 |
| the player (a human drives it: x1.0), run / dash | 7.8 / 8.7 | 7.9 / 8.6 | 7.9 / 8.5 | 7.8 / 8.5 |
| an AI member (x0.8: FUN_0012cb40), run / dash | 6.24 / 6.96 | 6.32 / 6.88 | 6.32 / 6.80 | 6.24 / 6.80 |
| an AI member following a leader beyond d1 (x0.8 x 1.25), run / dash | 7.8 / 8.7 | 7.9 / 8.6 | 7.9 / 8.5 | 7.8 / 8.5 |

* **The player** (`Player::controlled`): the type's speed as it is (FUN_0012cd10 calls
  FUN_0012c6c0 itself for a character FUN_00112800 says a human drives). #264 take01 (Tex dashing
  east on e34, the game's clock): the velocity eases up to 8.45 (type 8.5), take02 Brutus 8.7;
  #129 take34, a player's deathmatch Soldier: run 6.49, dash 8.44 (6.5 and 8.5). So the demo's
  plain run, WASD (the game's RUN: a pad's stick between a walk and 0.9), is 7.8-7.9, not the clips'
  4.4-5.0, and Shift's dash 8.5-8.7, not 6.7-8.0.
* **An AI member**: FUN_0012cb40's x0.8 for a squad type (type+0x1a8 bit 0). #33 take08 (Hawk's
  Move To): the dash levels at 6.84 (6.8). The **follow's boost**: the follow's update sets the
  brain's speed record to 1.25 each update and its chase state takes it off while the leader is
  within d1 (14 m in FOLLOW, 10 in WINGMAN: play_follow.rs `distances`, #331), and the AI
  update's FUN_00048270 hands it to the character at any pace level above a walk, so a member
  moving at a run or a dash after a leader farther than that moves at 0.8 x 1.25 = 1.0 x the
  type's speed: take01's first straight (the leader 48-50 m away, the game's clock) Flint
  8.53-8.55, Brutus 8.59-8.62, Hawk 8.42-8.46 (types 8.6, 8.7, 8.5); #33 take04 (Tex dashing
  east): Brutus and Hawk hold 13.7-14.5 m behind him, at 8-9 m/s beyond 14 m and 6.3-7.1 inside
  it. `squad_ai` sets `Player::speed_boost`
  after `routes::pace` (#335) at pace level 3 or 4 (`Router::cap`, which follows the move asked up
  and down since #344) with the leader beyond d1 and the speed record's timer run out (brain+0x8a4,
  `Avoid::boost_off`: a feeler's hit at a dash starts it at 1.5 s, #344); `update_player` keeps
  what the last frame left (char+0x720, the feelers' factor) and clears it before every member's
  goal. Move To writes nothing to the record and the follow goal's end resets it (FUN_00020a10), so
  it moves at 0.8 x.
* **Take01's "6.5-6.9"** is a mean of that kind: over their moving time the three covered 6.1
  (Flint), 6.6 (Brutus) and 5.9 (Hawk) m/s, 12-35% of it at the boosted dash (8.4-8.9), 20-29% at the
  run with the boost off (6.2-6.3: the feelers' level drop, FUN_00018c40 / the pace record, #344),
  26-36% under 5.9 (walks after the hard turns, the climbs back up the pace ramp: 0.17 s a notch
  for a squad member). The demo's AI walks routes and climbs its pace back after a drop since #335,
  and picks its gait by its distance to its place; since #344 also the feelers' drops, the climb
  from a stand, the step down after the move asked and the hard turn's walk (nav.md "The
  steering"): take01's staging then runs within a second of the take at each stretch of the west
  walkway.
* **The AI's side and back runs** (#363, nav.md "The view and the motion"): an AI member's motion
  is FUN_001195c0's against its own view, and at a run or a dash its side steps and backpedal are
  running states (FUN_00126a40): the sets' `f_side_run` / `b_side_run` / `back_run` clips, scaled to
  the run's speed as the run is, and x0.8 more while it backs off (FUN_0012cb40's second 0.8, while
  char+0x800 is 0: `speed::AI_BACK_FACTOR`), 0.64 x the type's run without the follow's boost. A
  player's back run keeps its clip's root.
* **Not done** (the walk and the ease are #350's): the walk, the crouch walk, the back steps and
  the side steps keep their clips' root (the game's walk follows the stick: #126 take07's "run" at a
  stick of 0.7 was the game's WALK, mot 47, at 3.5 m/s, Tex's type walk 3.6, where the demo's walk
  is the clip's 1.6); the velocity ease (4.5 a second, 7 slowing down: the demo's 0.2 s crossfade
  stands for it, so a run starts a little snappier and stops dead); the follow's states 5 and 6
  (the arrival's slow-down factor, the 1.5); Tex's berserk and Flint's sniper sets' run (their
  own clips); a character of a type of its own (`enemies::enemy`: the test hooks' drive) and
  cutscenes (scripted play moves everyone by the root, FUN_0012cb40's other branch). A jump keeps the ground speed
  (`last_velocity`), so a running jump is longer, and a grenade thrown on the run carries the
  thrower's speed (#161, play_grenade.rs `carry`: not checked against the game).
* **Test hooks**: `BF_SPEED_LOG=1` prints, every 0.25 s, each running or dashing character: who
  drives it (player or AI), the type's speed, the factor and the target, the speed it covers this
  frame, where it is, and the mean speed of its gait so far (the path counted every frame from 0.5 s
  in). `BF_ROOT_SPEED=1` moves the run and the dash by their clips' root, as before #345.
* **Verified** (`BF_MUTE=1 BF_BACKGROUND=1`, the means are the log's, exact):
  * the four as the player on the test map's floor (`--test BF_NO_SQUAD=1 BF_CHARACTER=<0-3>
    BF_TEST_GOTO=0,60,0,17 [BF_TEST_SPRINT=1.0] BF_SPEED_LOG=1`): run Brutus 7.81, Flint 7.91, Hawk
    7.91, Tex 7.84; dash 8.72, 8.62, 8.52, 8.50 (before, `BF_ROOT_SPEED=1`: run 4.55, 4.35, 5.03,
    4.82; dash 7.59, 6.71, 7.17, 8.01);
  * the squad after take01's staging (`--test BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=-70,45,70,45
    BF_TEST_GO_AT=1.0 BF_TEST_SPRINT=1.0 BF_SPEED_LOG=1`, the three 80-90 m from Tex): their dash
    over 5 s Brutus 8.72, Flint 8.62, Hawk 8.52 (take01: 8.60, 8.54, 8.44), and from 14 m off Tex
    7.0, 6.9, 6.8; run on 30 s (`BF_TEST_GOTO=-150,45,150,45`) they fall back to 13.5 m behind
    Tex's dash from the 20th second and hold it, at 8.5-8.7 beyond 14 m and 6.8-7.0 inside it (#33
    take04: 13.7-14.5 m);
  * Hawk's Move To (`BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,-83.39,-82.79,27 BF_TEST_NO_HOSTILES=1
    BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 "BF_TEST_ORDER=hawk,move,-70.8,-79.5@2"
    BF_SPEED_LOG=1`): 6.80 m/s (take08 6.84; 7.2 before);
  * on the build with #335's routes merged (the boost keyed on the pace level, `asked.min(cap)`):
    the four players' means are the same; the far-leader dash over 5 s is Brutus 8.72, Flint 8.61,
    Hawk 8.49-8.52; the e34 follow (`BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,-30,-80,27
    BF_TEST_GO_AT=4 BF_TEST_NO_HOSTILES=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05
    BF_ROUTE_LOG=1`) plans 7-8 routes a member with no stuck, detour or warp and ends 6.5-10.9 m
    off Tex; take01's staging on e13 goes by the same routes round the west shore and never into
    lava (its arrival times with the new speeds: [nav.md](../nav/squad-on-routes.md), "The squad on routes");
  * `cargo test --bin bf_play speed`: the rule's numbers.
