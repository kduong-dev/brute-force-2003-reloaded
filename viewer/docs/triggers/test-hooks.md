*Part of [triggers/](README.md), the former page `triggers.md`.*

## Test hooks

* `BF_TRIGGER_LOG=1`: at the start, the level's trigger counts by class, the squad's spawns and
  every detector (its volume and filter); then every event a trigger or an object sends (who,
  the signal, its subject and data, to whom), what each class does with it, the doors moved, the
  copies made, what's left to later steps. Names come from the level's cutscene scripts
  (`Game::script_names`: `entity_name = "trigger_spawn8"`), else the editor's
  `trigger_spawn<n>`-style ones that hash to them, else `h_xxxxxxxx`.
* `BF_TRIGGER_FIRE=<trigger>,<signal>[,<s>][;...]`: sends the signal to that trigger at that
  time (default 0) as a sender with iD 0 would. The signal: a number or its name (`USER1`,
  `USER_SIGNAL1`, `ENTER_DETECTOR`, `TRIG_SIG_TIMER_PULSE`). The trigger: its name
  (`trigger_router_spawn_tex`) or `h_xxxxxxxx`. `@<object>,<signal>` sends it from an object to
  its listeners instead (`@TUTORIAL_terrorist_5,DESTROYED`).
* `BF_TEST_NO_HOSTILES=1` (#318): a spawn-trigger makes no character of another side than the
  player's (logged with `BF_TRIGGER_LOG` as "would spawn"): a campaign level with the squad
  alone, as the #33 takes' snapshot had e34 (every spawn-trigger disarmed, every enemy killed;
  todo/33-squad-commands notes.md). The squad's own spawns and the pickups' still come. With
  `BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05` e34's opening letterbox goes too
  ([squad.md](../squad/command-modes.md), **Squad commands**).
* The game flags, the cash and the objectives menu have their own (`BF_FLAG_LOG`,
  `BF_FLAG_SET`, `BF_CASH_LOG`, `BF_TEST_OBJECTIVES`): [campaign.md](../campaign.md). The
  cutscenes' (`BF_SCRIPT`, `BF_SCRIPT_LOG`): [cutscenes.md](../cutscenes.md).
* `BF_RADAR_LOG=1`: the radar-triggers' contacts (above). The damage plate's hits:
  `BF_COMBAT_LOG=1`. The level's mines: `BF_SENTRY_LOG=1` ([grenades.md](../grenades/sentry.md),
  "Sentry"); the DNA canister: `BF_PICKUP_LOG=1` ([pickups.md](../pickups/README.md)).

```sh
# the tutorial's base: the plate opens the door and spawns four terrorists
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_TEST_GOTO=-36.4,37.3,-57.3,22.9 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# step 9 (#228; BF_TEST_PANEL closes the panels that would stop the clock): the first patrol's
# circle moves the radar's blip and chimes
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_RADAR_LOG=1 BF_TEST_GOTO=10.64,44.15,22.28,34.31 "BF_TEST_PANEL=a,1;a,1;a,1;a,1" BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the damage plate: on plate h_e8c8e32a (the start's height 32 puts Tex on its slab) the
# show_patrol_2 script starts, warps Tex beside the damage plate and enables it for its second
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_COMBAT_LOG=1 BF_TEST_GOTO=37,67.5,37,67.5,32 "BF_TEST_PANEL=a,1;a,1;a,1;a,1" BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the east pit's lava bursts, turned on as the opening script does at 7.2 s, from take08's place
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_TEST_GOTO=43.69,-21.63,46.26,-25.92 "BF_TRIGGER_FIRE=trigger_router_turn_on_lava,USER1,0.5" "BF_TEST_PANEL=a,1;a,1;a,1;a,1" BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the farcast: the opening script's router spawns Tex at 17.09 s (#222)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SCRIPT_LOG=1 cargo run --bin bf_play
# or by hand, without the script (the squad is then in the world from the start)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SCRIPT=off "BF_TRIGGER_FIRE=trigger_router_spawn_tex,USER1,1" cargo run --bin bf_play
```
