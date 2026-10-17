# Cutscenes (`src/bin/play_script.rs`, `src/bf/gscr.rs`, #222)

A level's in-engine cutscenes are scripts its script-triggers play: camera shots and moves, the
letterbox, warps, holds on the characters, and signals to the level's triggers at set times. The
tutorial (#28, plan step 3) has nine: its opening (the wall gate, the lava, Tex's farcast), the
first patrol, the grenade, mine and sniper lessons, the second patrol, the V-squad's farcast and
its extension, and the ending. This page is how the demo plays them. Everything is read from
default.xbe's script player and the triggers that drive it (the addresses are in the code's
comments and in [code-map.md](code-map.md), "Cutscenes"). One value is fitted: the letterbox
share above which the HUD is hidden.

## The scripts (`bf/gscr.rs`)

* **The format** is Digital Anvil's THN (Freelancer's "Thorn"; the engine's interface is
  "23.23_IScriptEngine"): Lua 4 source, `scripts/<name>.gscr` in the level's archive, that sets
  three globals. `duration` is in seconds. `entities` is a list of tables: `entity_name`, `type`,
  `template_name`, `spatialprops` (`pos`, and `orient`, whose rows are the entity's axes) and
  `cameraprops`. `events` is a list of `{time, type, {entity names}, {parameters}}`. The game
  runs the file in its Lua and reads the globals back (FUN_0026c960; an entity's record
  FUN_0026db20, an event's FUN_0026cb40). The files are only table assignments, so the demo
  reads them without a Lua VM.
* **The capitalised names** (`CAMERA`, `USER_EVENT`, `Y_AXIS`, `POSITION + ORIENTATION`) are
  globals the game registers before it runs a script (FUN_0026b7d0):
  * entity types (names 0x3bb1dc, values 0x3bb214): UNKNOWN_ENTITY 0, COMPOUND 1, DEFORMABLE 2,
    CAMERA 3, MONITOR 4, LIGHT 5, SOUND 6, MARKER 7, HARDPOINT 8, SCENE 9, SUB_SCENE 10,
    MOTION_PATH 11, DELETED 12, PSYS 13;
  * event types (names 0x3bb24c, values 0x3bb288): UNDEFINED_EVENT 0, USER_EVENT 1,
    SET_CAMERA 2, START_SOUND 3, START_LIGHT_PROP_ANIM 4, START_CAMERA_PROP_ANIM 5,
    START_PATH_ANIMATION 6, START_SPATIAL_PROP_ANIM 7, ATTACH_ENTITY 8, CONNECT_HARDPOINTS 9,
    START_MOTION 10, START_IK 11, START_SUB_SCENE 12, START_PSYS 13, START_PSYS_PROP_ANIM 14;
  * axes X_AXIS .. NEG_Z_AXIS 0-5 (0x3bb2c4);
  * the path flags USE_SCRIPT_DURATION 1, POSITION 2 (PATH_POSITION 2), ORIENTATION 4,
    LOOK_AT 8, ENTITY_RELATIVE 32 and PARENT_CHILD 64.
* **A USER_EVENT's `user_event_data`** is `"<code>, <n>, args"` (FUN_001753f0 reads `%d, %d`,
  then the code's own fields). The code says what it does; the entity it names says which
  controller runs it ("The events").
* **All of the game's 179 scripts read** (`BF_SCRIPT_CHECK=1` in `bf_level`): their events are
  user events 100 x20, 101 x203, 103 x875, 104 x20, 105 x60, 111 x5, 113 x3, 114 x58, 115 x5,
  119 x249, 120 x20, 122 x76, 123 x69, START_PATH_ANIMATION x154 and ATTACH_ENTITY x47. The
  other event types don't occur. `Game::load_level` parses a level's scripts and tags each with
  its archive. The demo plays only the current map's: bf_play also loads e01's archive for its
  audio, and the game registers only the level's own files (FUN_00174180 lists the level's
  `scripts/*.gscr`).

## The player

* **One player per script.** As a level loads (`end_load`, FUN_000d6520) FUN_00174180 makes a
  player (0x130 bytes, FUN_00174620) for each of its scripts and registers it by its name's
  hash (FUN_00174460). FUN_000d8140 then binds each script's entities to the world
  (FUN_00175090):
  * a COMPOUND, CAMERA, LIGHT or SOUND to the world object of its name (a CAMERA to the level's
    camera-object);
  * a MARKER with template `trig` or `wpoint` to the level's trigger or world point of that
    name (FUN_000da6f0);
  * a MARKER with template `script` to the script itself;
  * a MOTION_PATH to the level's path-object.
  An entity whose object isn't in the world isn't bound. A spawn's copy is bound as it's made
  (FUN_002096e0 -> FUN_00174fe0).
* **The script-trigger** (vtable 0x395ab8, handler 0x205000): OPERATE asks its script's player
  (FUN_00204ec0, looked up by `script-name` with FUN_001744f0). The entry's `<script command>`
  says what: 0 or 1 plays it from the start (the restart flag +0xa4, rate 1.0), 2 pauses, 3
  resumes (FUN_00174c10), 4 stops (FUN_00174bd0). The tutorial's are all 0.
* **One start.** The restart happens in the player's next update (FUN_00174930 ->
  FUN_00174b30), so two OPERATEs in a frame start it once. The level's start sends FIRST_UPDATE
  to every script-trigger and then runs its update (FUN_001fe260): the opening's trigger gets it
  twice and logs two "plays", and the opening starts once.
* **The start** holds every character the script binds (FUN_00174e10 -> FUN_00174cd0: char+0x834
  bit 0, its AI and its player's controls off, FUN_00129530). Each update moves the player's
  clock on by the frame's whole milliseconds (the manager's FUN_00174270 carries the fraction
  over). It runs the events now due, in order (the file's order for equal times), then the path
  animations under way (the list at +0x98). At `duration` it stops (FUN_00174bd0) and releases
  the characters it held (FUN_00174e70).
* **In the demo's frame** (play_triggers.rs `run`): the triggers' updates, what they asked of
  the world (spawns, warps, doors), then the cutscenes (`script::step`). An event whose signal
  asks something of the world (a 103 to a spawn- or warp-trigger) ends the pass. `run` does
  what it asked, then the script goes on from its next event, because the game's signals act at
  once. The opening's 119 on tex_1 at 17.106 s holds the Tex its 103 at 17.091 s spawned, in the
  same frame.

## The events

USER_EVENT goes to the controller of the entity it names. Each controller runs its own codes:

| Code | On | What it does | Code |
|---|---|---|---|
| 103 | a trigger (MARKER `trig`) | its signal (the first argument) straight to the trigger's handler, from iD 0, no subject, data -1: the lines, the routers, the spawns, the warps, the letterbox | 0x1772b0 (the controller FUN_00176f70) |
| 101 | a camera, argument 0 | the view cuts to it (FUN_00173270: the view's +0x8c camera, its cut flag +0xc8). Only in a mission (the world's +0xbec +0x18, 0x8cd30) | 0x1732c0 (FUN_00172f10's controller) |
| 101 | the script or a camera, arguments 1-3 | back to the game's camera (FUN_00072f00: +0x8c cleared, the follow camera cut to its place) | 0x1761b0 (table 0x1764b4 / 0x176498) |
| 122 / 123 | the script | the world's cutscene bit on / off (world+0xc50 bit 4, "The cutscene bit") | 0x176227 / 0x176258 |
| 119 | a character | the cutscene flags: v0 without v1, or v9 without v10, hold it (FUN_00129470); v0 with v1 hand it to the script's movement (FUN_001294a0); v9 with v10 release it (FUN_001294c0). A held one joins the script's list (FUN_00174dc0), released at the end. All of the tutorial's 119s are `1,0,0,0,0,0,1,0,0,1,0,0`: hold | 0x17206d (fields FUN_001753f0 case 0x77) |
| 105 | a character | hand it to the script's movement and send it to the world points listed after the `:` (FUN_001294a0, the goal FUN_00171710) | 0x171f7a |

* **START_PATH_ANIMATION** moves its first entity along its second (a path) over its `duration`
  (the event class at vtable 0x392520: FUN_00275180 builds it, 0x275290 updates it each frame).
  See "Cameras and paths".
* **A character not in the world gets nothing.** Its entity isn't bound, so its events reach no
  controller. The V-squad's 105s at 0.259 s name the two marines its 103s spawn at 7.57 and
  7.87 s, and do nothing. Take11 has the marines walk off their pads after the cutscene. That
  walk is their AI's: each spawn-trigger gives its copy's AI the 105's own point (`h_131de144`:
  trigger_spawn64 wpoint1380, trigger_spawn76 wpoint1006), and the demo has no such AI yet
  (#227).
* **Not run** (logged, #232): codes 100, 104, 110 (another script by name), 111, 113, 114 (a
  camera shake), 115 and 120 (a fade, FUN_00069780), and ATTACH_ENTITY. The tutorial uses none
  of them. Of 119's other fields (v2 and v7 the character's FUN_001268b0, v6 an entity flag, v8,
  v11 its brain reset) none is run; the tutorial sets only v6.

## Cameras and paths

* **A CAMERA entity is the level's camera-object of that name** (`level::LevelCamera`, attribute
  handler 0x6cb90), looking down its transform's -z, not the script's own copy. Its h_1435f288
  is the horizontal half angle in degrees: the handler passes twice it to FUN_0006e450, which
  takes the whole angle, and the aspect is forced to 4:3 (0x6ba30). h_19bc8fdf and h_125978e4
  are near and far. The tutorial's angles across are:
  * 90: wall gate, lava, mine lesson, sniper, V-squad, ending;
  * 80: the opening's start area;
  * 70: behind Tex, first patrol;
  * 60: grenade lesson, second patrol.
  The scripts' own `cameraprops` (fovh 36.87 on all of them) aren't what shows.
* **The view** while a cutscene camera is shown is that camera's transform, with the vertical
  angle that the horizontal one gives on a 4:3 screen. Back to the game's camera, the follow
  camera snaps to its place behind the player (the cut flag), with no ease in.
* **The listener.** While a cutscene camera is shown, the sound listener is at that camera
  (FUN_00178c10: a camera that follows no character puts the listener at the camera and leaves
  the indoor flag as it was; play_sound_triggers.rs `listener`, `Cutscenes::camera_at`). The
  opening's lava shot hears the east pit's bursts at 13-29 m. From Tex's spawn they're over
  100 m off and silent.
* **A path** is a level path-object (`level::LevelPath`, handler 0x16cca0): keys of a time (s)
  and a transform in the path's own frame. A START_PATH_ANIMATION's share of the path now is
  `start_percent + elapsed / duration x (stop_percent - start_percent)`, or 1 when the two are
  equal (0x275630). With POSITION its target goes to the path's point at that share, plus
  `offset`, in the path's frame:
  * **By a share of its length** (no `event_flags`): the path is a Catmull-Rom spline through
    its keys' positions (CV_CRSplinePath h_f28ddde7, FUN_0028c430; uniform CR, FUN_0028b860).
    A made-up end point sits before the first key and after the last: the curve through the end
    segment, each end doubled, at 0.1 (0.9 at the far end), mirrored about the key. That puts it
    0.064 of the end segment out. Each segment is cut into 16 chords (FUN_0028c750), and the
    point at a share of the length is that far along them (0x28d2e0 -> FUN_0028d480).
  * **By the keys' own times** (`event_flags` not 0, the path controller's +0x8c, FUN_001765a0):
    the keyed spline FUN_0016c190 builds, with the ends mirrored (P0 - (P1 - P0)). It is
    Catmull-Rom between the two keys the elapsed time falls between (FUN_0016c4d0,
    FUN_0016b5c0). The opening's start-area shot (path0) is one.
  * **ORIENTATION**: the keys' rotations by their times (ms, truncated as 0x16ce20 reads them),
    slerped between the two either side (FUN_0016f080), the end keys outside them. By a share,
    the time is share x (the last key's time - the first's) (0x16cbf0, FUN_00170f00).
  * **USE_SCRIPT_DURATION** runs it to the script's end; otherwise it ends at its `duration`,
    at its last point.

## The letterbox

* **The view's bars** (the view manager world+0xbf0: its amount +0x78, target +0x7c, time left
  +0x80): FUN_00073460 brings them in toward 1 over its time x (1 - the amount), FUN_000734b0
  out toward 0 over its time x the amount, each time clamped to 0-10 s. The step (0x733ee) is
  linear and lands once the time left is under the frame's. At full they cover 145.83 / 1000
  of the screen's height at the top and at the bottom (FUN_00073110 crops the 3D view's
  viewport by that: 70 of 480 lines). Take03's frames at 26.0 and 26.8 s have the same picture
  under the bars and without them, so the demo draws two black bars over the full view.
* **The letterbox-trigger** (handler 0x207480): OPERATE brings them in over its `show-duration`,
  STOP takes them out over its `hide-duration`. Its other call, the world's +0xbec +0x54 / +0x58,
  is the co-op join, not the HUD. The tutorial's seven: trigger_letterbox14 (the opening: in at
  once, out over 0.5 s), the patrol's, the lessons', the V-squad's (0.5 s each way) and the
  ending's (in at once). Every one is driven by its script's 103s.
* **The HUD** is hidden while the bars are in beyond 0.25 of full. **A fit**: the code that
  hides it wasn't found. Take03 has no HUD at 26.3 s with the bars at 0.37, a faint one at 26.4 s
  (0.17) and all of it at 26.6 s (gone). Take09 at 39.95 s is half there at 0.14. Take15 has it
  hidden through show_patrol_2, which has no 122 / 123, so it follows the bars and not the
  cutscene bit.

## Warps

The warp-trigger (handler 0x200650, FUN_00200480) runs on OPERATE only. It moves each of its
objects to its `world-point`: a spawn-trigger names its copy, and the squad's start stands for
its spawn's copy (`Triggers::warp_object`). A point with an orientation of its own (a
direction-point: a `<transform>`) gives the whole transform, so the character faces down its -z
(the place call +0x50, as a spawn's copy faces its trigger's -z). A world-point gives only the
position (+0x48), and the facing stays. A living character stands on the ground under the
point (`on_ground`, the place call's ray). The controlled one's camera goes with it. A dead
one isn't moved. A squad member not spawned yet has no copy to move. The tutorial's warps:
* Tex to direction_point1 for the patrol, direction_point2 for the second patrol and
  direction_point0 for the V-squad;
* Tex twice in the ending, to h_f0631043 (-58.6, 14.6, 85.2) and at 6.67 s to h_e7e1af46
  (-88.7, 17.8, -19.9);
* the marines in the second patrol and the ending. The three start marines never come (their
  member check fails), so theirs move nothing. The ending also warps the V-squad's two, which
  the `BF_SCRIPT` run checked didn't have in the world.

## The cutscene bit

A script's 122 sets the world's +0xc50 bit 4 and its 123 clears it (take03's watch log: its
`wflag` 137456 from the level's start, 137440 at clk 26188, the opening's 123 at 26.176 s).
While it's set:
* no menu opens on a player's button (FUN_0008c8c0, the objectives menu among them:
  play_objectives.rs);
* the player's controls don't reach the game (play.rs `read_input`, `script::blocks_input`);
* nobody takes damage (the damage handler 0x125600's early exit: play.rs `hurt`);
* the squad's chatter is quiet (FUN_00140a60) and the squad-dead countdown waits
  (FUN_00089d30). Neither is done here.

A character a script holds (119, or bound at its start) has no controls and no AI: play.rs
`read_input` and `update_player` hand it to `script::held_step`, which zeroes its inputs and
walks it to its 105 points, if any, in straight lines at a walk.

## The squad's farcast

A squad member that a spawn-trigger spawns isn't in the world until then. Its template is taken
out as the trigger links (FUN_000dfe90), so the game's Tex appears with the opening's farcast at
17.09 s (take03's watch log: clk 17102). The demo keeps it out (`Player::absent`, play.rs
`player_at_start` / `squad_at_start`) until `spawn_copy` brings it in. Until then:
* it isn't drawn (`view` hides its model);
* the detectors don't see it, so the start area's proximity detectors h_ea082132 and h_e30f11a4
  send ENTER_DETECTOR at 17.27 s, not at 0.07 s as before;
* nobody targets it;
* scripts and warps don't bind it.

Under `BF_TEST_GOTO` or `BF_SCRIPT` the level's start scripts don't play (they'd hold the player
for the opening's 26 s), and the squad is in the world from the start, as before #222.

## The tutorial's scripts

| Script | Played by | Length | What it shows | Take |
|---|---|---|---|---|
| tutorial_start_long | h_1a52bddf, the level's FIRST_UPDATE | 26.254 s | the wall gate's sentry (path8, 90), the lava (path9; router443 / router441 stop the pits' sounds at 6.575 / 11.025 s), the start area (path0 by its keys' times, 80), Tex's farcast (17.056 s the flash, 17.091 s his spawn), behind Tex (path10, 70); Bratton's four lines at 3.02, 6.176, 11.217, 19.551 s; back to the game's camera at 25.763 s, the bars out from 25.985 s | take01, take03 |
| tutorial_show_patrol | h_13ad4055: plate h_1219602a (34.4, 94.3) | 8.270 s | Tex warped to direction_point1, camera_patrol on the patrol | take07 |
| tutorial_grenade_lesson | h_0990aa24: plate h_f47500d1 (25.5, 32.9) | 6.709 s | three shots of the path and the terrorist past the palms | take07 (184.6 s) |
| tutorial_mine_lesson | h_1f59e0d0: plate h_18bb084d (92.4, -63.9) | 5.309 s | the mine field | take09 |
| tutorial_sniper_lesson | h_fcd412be: plate h_1edb93d1 (-103.0, -48.4) | 7.785 s | the L-Shot-50's crate, the tower, terrorist 4 (held) | take11 (170.3 s) |
| tutorial_show_patrol_2 | h_02f6c592: plate h_e8c8e32a (34.5, 67.5) | 13.231 s | Tex warped to direction_point2, then the second patrol (path2) | take15 |
| tutorial_farcast_vsquad_short | h_f88aae21: timer526's pulse after the V-squad plate | 8.468 s | Tex warped to direction_point0, camera_vsquad_script along path6, the two marines' farcast | take11 (21.2 s) |
| tutorial_vsquad_farcast_extension | trigger_script462: timer468's pulse | 8.507 s | two 103s, no camera | |
| tutorial_ending | h_f82e0ba6: the base pad's proximity detector h_ffa38718 | 9.172 s | the ending; the level-over at 8.93 s (#226) | take12 (347.8 s) |

The archive also holds e05_transmitter_explosion_shake (one 114), which no tutorial trigger
names.

## Test hooks

* `BF_SCRIPT=<name>[,<s>]`: plays that script at that time (sim seconds, default 0), as a
  script-trigger would. Then, as with `BF_TEST_GOTO`, the level's start doesn't play its scripts
  and the squad is in the world from the start. `BF_SCRIPT=off` (or `none`, `0`): no script
  plays. The hook doesn't use the script's trigger, so its once-only entry is still there: a
  script that warps Tex onto its own plate (the V-squad's) plays again from it.
* `BF_SCRIPT_LOG=1`: at the start, the map's scripts, cameras and paths. Then each script's
  start (what it holds) and end (what it releases), and every event as it runs: the cuts with
  their angle, the paths, the holds, the signals sent, what isn't run. Also the letterbox and the
  cutscene bit.
* `BF_SCRIPT_CHECK=1` (`bf_level`): reads every `.gscr` of every `data/*.tgz` and prints each
  script's length, entities and events by code, then the totals. It exits after.
* With `BF_TRIGGER_LOG=1` the trigger runtime logs the script-, letterbox- and warp-triggers'
  actions, the 103s' signals and the warps.

```sh
# the opening, as the level starts
BF_MAP=tutorial BF_SCRIPT_LOG=1 BF_TRIGGER_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=420 cargo run --bin bf_play
# a lesson from its plate: the patrol, the mine field, the V-squad (standing on its plate)
BF_MAP=tutorial BF_SCRIPT_LOG=1 BF_TEST_GOTO=44,91,34.4,94.3 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
BF_MAP=tutorial BF_SCRIPT_LOG=1 BF_TEST_GOTO=89,-55,92.4,-63.9 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
BF_MAP=tutorial BF_SCRIPT_LOG=1 BF_TEST_GOTO=-15.6,-87.4,-15.6,-87.4 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# a script by its name
BF_MAP=tutorial BF_SCRIPT_LOG=1 BF_SCRIPT=tutorial_ending,1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# every script of the game read
BF_SCRIPT_CHECK=1 cargo run --bin bf_level
```

`BF_TEST_GOTO`'s autopilot turns the camera to its goal every frame (or down -z when it stands
still), and that goes on once a script lets the player go. A script's "back to the game's camera"
is seen properly without it (`BF_SCRIPT`).

## How it was checked

Against `todo/28-tutorial/`. The demo ran with `BF_MUTE=1 BF_BACKGROUND=1`, captured at 15 fps.
The take's frame for script time s is `start + s / ratio`, with the start and ratio measured
from each take's letterbox and cuts: the watched takes' clock runs at 0.92-0.96 of the recorded
time (the watcher pauses the VM). The demo's frame is `round((s + its start) x 15) - 1`. The
frames were compared side by side.

* **The opening** (no hooks; take03 at the game clock's times through its watch log, and take01,
  unwatched, for timing):
  * the four shots and their framing: the wall gate with the sentry, the lava pits, the start
    area moving by its keys, behind Tex. Their angles are 90, 90, 80 and 70 (70 confirmed for
    the last against take03);
  * the cuts at 6.105, 10.750 and 19.358 s and back at 25.763 s. Take01 cuts 6.066, 10.800,
    19.533 and 26.066 s after the level's first frame (269.917 s), within 0.05 s of the
    script's for the first two and 0.18 / 0.30 s late for the last two;
  * the bars in from the first frame and out over 0.5 s (take01: 26.25 to 26.73 s), the same
    crop;
  * the HUD gone through it and back as the bars go;
  * no Tex until the farcast at 17.09 s, then Tex on the ground under trigger_spawn8, held by
    the 119 in the same frame;
  * Bratton's lines, the lava pits' sounds stopped by router443 / router441, the start area's
    detectors at 17.27 s, the cutscene bit on at 0 and off at 26.176 s (the objectives menu
    `BF_TEST_OBJECTIVES=2,1` doesn't open then; it does on a `BF_TEST_GOTO` run);
  * the sound listener at the cameras (`BF_SOUND_LOG`): the west pit's burst at 4.0 s is 35.9 m
    from camera_show_wallgate_terrorist, and the east pit's at 8.2-9.7 s are 13-29 m from
    camera_show_lava.
* **The first patrol** (`BF_TEST_GOTO=44,91,34.4,94.3`, take07 from 41.57 s at 0.917): the warp
  to direction_point1, camera_patrol's shot, the bars in and out over 0.5 s.
* **The grenade lesson** (`BF_TEST_GOTO=25.5,41,25.5,32.9`, take07 from 184.55 s at 0.95, bars
  in 184.55-185.1 s, out 191.1-191.65 s): the three shots (60 across) at their times. The take
  frames Tex in the first shot, where the demo's Tex, started elsewhere on the big plate, isn't.
* **The mine lesson** (`BF_TEST_GOTO=89,-55,92.4,-63.9`, take09 from 39.75 s at 0.93): the mine
  field's shot (90 across) at its times, the bars in and out over 0.5 s, the HUD back as they
  go (the game's fades in over them; the demo's is there at once below 0.25).
* **The sniper lesson** (`BF_TEST_GOTO=-103,-40,-103,-48.4`, take11 from 170.27 s at 0.946):
  the crate, the tower and terrorist 4's shots. The script holds and releases
  TUTORIAL_terrorist_4.
* **The second patrol** (`BF_TEST_GOTO=34.5,75,34.5,67.5`, take15 from 7.4 s at 0.93): Tex
  warped to direction_point2 (take15's notes: ~(25.7, 68.0)), the close shot, the path2 move
  at 6.958 s, the patrol's shot. The bars out at 12.93 s.
* **The V-squad** (`BF_TEST_GOTO=-15.6,-87.4,-15.6,-87.4`, and `BF_SCRIPT=tutorial_farcast_vsquad_short,1`
  for the camera after it; take11 from 20.81 s at 0.943): timer526's pulse plays it; Tex warped
  to direction_point0, the camera along path6, the marines spawned at 7.57 / 7.87 s and held by
  the 119s at 7.695 / 8.028 s, released at the end. The follow camera is then behind Tex,
  looking at the pads, as take11's is. Then the extension from timer468.
* **The ending** (`BF_SCRIPT=tutorial_ending,1 BF_TEST_GOTO=-53,80,-53,80`, take12 from
  347.757 s at 0.93): camera_ending, the two warps, the level-over trigger's OPERATE at 8.93 s.
* **Regressions**: sdm_e34's doors (`BF_TEST_GOTO=-76.26,-2.04,-62.34,5.64`): they open and
  close as before. The map's own scripts are none. The tutorial's base plate
  (`BF_TEST_GOTO=-36.4,37.3,-57.3,22.9`): the plate, the four terrorists and the door as
  before, the opening not played. The objectives menu (above). The briefing movies
  (`BF_MOVIES=m00 BF_MOVIES_SKIP=3,6,9`): the three movies play and skip as before. Unit tests:
  `cargo test --lib` (`gscr::reads_a_script`) and `cargo test --bin bf_play`.

## Not done, or not known

* The codes and event types the tutorial doesn't use (above, #232). LOOK_AT is read as "along
  the path" (FUN_0028b000 from `up` and `front`), unchecked: no tutorial path has it.
* 105's walk is a straight line at a walk to each point, not the brain's goal (FUN_00171710:
  the AI's routes, #227). No tutorial character it names is in the world at the time.
* Turning along a path of more than two keys slerps between the two keys either side. What
  the game does there (FUN_0016f080 with more keys; FUN_0016c5e0's eased segments on the keyed
  path) isn't read.
* The spawn and warp fade-ins (h_eeae96e7, h_0ede4719: 2 s on Tex and the marines) aren't
  drawn: they appear at once.
* The HUD comes back at once as the bars fall below 0.25 (the fit). The game's fades in over
  them (take03 faint at 26.4 s, take09 half there at 39.95 s); its code wasn't found.
* The level's start fades in from black over ~0.5-0.7 s in take01. That isn't in the scripts
  and isn't done.
* The opening's title card ("Brute Force" / "Welcome to the 23rd!", take03 from about 6 s to
  15 s) is the HUD's and isn't done.
* The squad's chatter isn't hushed by the cutscene bit, and the squad-dead countdown doesn't
  wait for it.
* The demo's characters are darker than the game's in the front-lit shots, and its follow
  camera is closer: both are the demo's own, not the cutscenes'.
* Take01's last two cuts come 0.18 and 0.30 s after the script's times while the first two are
  within 0.05 s. The demo cuts at the script's times.
* What a script-trigger's command 1 does differently from 0 (the engine's +0x50 gets 0 or 1)
  isn't traced. The tutorial uses 0.
* The terrorists' patrols in the patrol shots stand still (#227). The V-squad's marines stand
  on their pads after the cutscene, where take11's walk to their AI's points (#227).
