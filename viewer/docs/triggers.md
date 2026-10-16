# Level triggers (`src/bin/play_triggers.rs`, `src/bf/triggers.rs`, #220)

A level's logic is its triggers: routers, timers, calculators, spawn-triggers, detectors,
anim-triggers and some twenty more classes, wired to each other in the level file. The tutorial
has 500 (58 routers, 54 timers, 47 spawn-triggers, 27 plates, 14 proximity detectors ...). This
is the runtime that runs them, the first step of the tutorial (#28, plan step 1); the later
steps plug their classes into it (gamevar and score: step 2, #221; script, letterbox and warp:
step 3, #222; sound and dialog: step 4, #223; message, input, menu and event: step 5, #224). Everything here is read from
default.xbe's trigger code (the addresses are in the code's comments and in
[code-map.md](code-map.md), "Levels and triggers"); nothing is fitted.

`bf/triggers.rs` reads them: every element of the level file that carries the trigger base's
`h_137926ff` (FUN_001fedb0's attributes), with its entries, targets (`h_e91cf6a8`), objects
(`h_0b6b92ba`), transform and its class's own attributes (`Trigger`, `Class`); the element is
kept whole for the classes later steps run. `Level::triggers` holds them in the file's order.

## How the game runs them

* **Events.** An event is {signal, the sender's `iD`, a subject object, data} (0x14 bytes,
  vtable 0x3a3fb4). A trigger sends to its targets only while it's enabled and not dead
  (FUN_001ff0a0). An object (a character, a world button, a pickup, a spawn's copy) sends to the
  triggers that list it among their objects (FUN_001fe3d0). `h_0884288c` is the editor's
  back-reference list: read and dropped.
* **Entries** (`<h_031cd9f5>`: `<base Type signal action match-id h_170a9b80 h_f136d22d
  h_e2da3955>`, alone or inside an `<anim>`, `<calculator value>` or `<script>`). A trigger
  answers an event with its entries in order whose signal and match-id are the event's and
  whose use count (h_e2da3955, 0 in the data) is under their limit (h_f136d22d: 1 once,
  2147483647 always) (FUN_001feb60). The match is exact: the data's "any" entries (signal -1)
  never match; the routers that have them pass everything without reading their entries.
* **An entry acting** counts a use (FUN_001feac0). Spent with h_170a9b80, or any entry acting
  with h_137926ff, the trigger sends DISABLED to its targets and disables itself. Enabling a
  trigger zeroes all its entries' counts (0x1fefc0).
* **After an event** other than FIRST_UPDATE (its own handling of one, or a send), h_f657f799
  kills the trigger (it never acts or sends again) and h_1eec0e75 disables it, once each
  (FUN_001ff000). The DESTROYED / DISABLED they send then is lost: the trigger is off by then.
* **The base's actions** (every class, FUN_001ff0f0): ENABLE (ENABLED to the targets), DISABLE,
  TOGGLE_ENABLE, and USER_SIG1-8 (USER_SIGNAL1-8 to the targets, while enabled). A DISABLE's own
  DISABLED is lost the same way. A disabled trigger runs only these.
* **FIRST_UPDATE** (13): each enabled trigger's first update sends it to the trigger itself,
  once (FUN_001fe610 / FUN_001fe660); one disabled at the start gets it on its first update once
  enabled. Before that, the level's start sends it to every script-trigger and runs its update
  (FUN_001fe260), so an enabled script-trigger gets it twice (the tutorial's opening one logs two
  OPERATEs; its script player is step 3's).

## The classes it runs

| Class (tutorial count) | What it does | Code |
|---|---|---|
| router (58) | Passes every event but FIRST_UPDATE to its targets. With h_f0d6857f it passes only those its OPERATE entries accept, as `out-signal` when that isn't -1 (e.g. `h_e9413a18`: ENTER_DETECTOR -> USER_SIGNAL1) | 0x208fc0 |
| timer (54) | OPERATE starts it, TIMER_FULL_RESET stops and reloads it, RESET_COUNTDOWN_TIME reloads it. Running and enabled it counts `current-value` down; at 0 it sends TIMER_PULSE (with the subject its OPERATE came with), then stops and reloads (h_f92b0b15), reloads (enable-looping), or stays at 0 until reset | 0x200190, 0x2002f0 |
| calculator (12) | INC / DEC / MODIFY_COUNTER (by the `<calculator value>`) / SET_COUNTER_FROM_SIG_DATA, then its test h_015914ab (0 ==, 1 <, 2 >, 3 <=, 4 >= its `target`, unsigned): CALCULATOR then TRUE, or FALSE, with the counter as data | 0x1ff5f0 |
| spawn-trigger (47) | See "Spawn-triggers" | 0x209ac0, 0x209bd0 |
| plate-detector (27), proximity-detector (14) | See "Detectors" | FUN_001f9680, FUN_001f8db0 |
| anim-trigger (7) | OPERATE plays its entry's clip (`animation-archetype`) on its first object, forward or backward (`wwit-debugging`, AM_: 3-5 backward) at its `time-scale`; with AC_PLAY or AC_RESUME it sends ANIM_START when it starts forward after playing backward (its first forward play too), ANIM_BACK_START when it starts backward after playing forward (or ANIM_START for the other modes). The demo moves doors and gates with it ([maps.md](maps.md), "Doors and gates"); other animated objects (the tutorial's two looping props) are logged | 0x204d00, FUN_00204bb0 |
| destroyer (5) | OPERATE: on its next update it removes its objects (a listed spawn-trigger's copy), then disables itself | 0x200d70, 0x200e40 |
| waypoint (h_10ecb459, 31), evaluate (6) | Their base actions only: a waypoint trigger marks AI route points; the evaluate-triggers' random tests pick ambient lines (both no-ops here) | |

Every other class (gamevar, score, script, letterbox, warp, sound, dialog, menu, message
h_e9b3ae8e, input h_e2b9c3a2, event, radar, level-over h_1f07be84, property h_14cccfaa, the
damage plate h_130818c0) runs its base actions; each of its own actions is counted as used, as
its handler counts it, and logged with the step that runs it ("trigger_sound439 (sound)
PLAY_SOUND on FIRST_UPDATE: not run yet (step 4, #223)").

## Detectors

* **Volume.** A plate is a box: its `size` is the box's half-extents in the trigger's frame. The
  game tests a physics phantom, the Ipion surface named by the plate's `h_051ac63e`
  (FUN_001f99a0), and those surfaces are exactly that box: the tutorial's base plate
  h_eefcb023 (`size` 30 10 4) is a surface spanning +-30, +-10, +-4; the building plates' (1.5 2 2)
  +-1.5, +-2, +-2 (measured from objects-tutorial.ipn). Its fallback test without a phantom
  (0x1f9e00) also takes `size` as half-extents. A proximity detector is a sphere of `radius`
  (its phantom FUN_001fa330). The character is the demo's upright cylinder (BODY_RADIUS 0.4,
  BODY_HEIGHT 1.9), not its Ipion body: a plate counts it when a point along its axis is within
  the box grown by its radius, a sphere when its axis comes within radius + 0.4.
* **Filter** (`<filter>`, FUN_001fb980; every part must pass, `h_ebfd2b20` turns each over):
  `h_19656b35` a character of a squad type (its type's `h_e78efbb2`: Tex and the three Confed
  marine types in the tutorial, not the Red Hand), `h_1d083f6a` one of any other type,
  `h_115c7b01` any character (every sdm_e34 door's), `h_e707ecc1` a character with a human
  player (the one the player controls), `alignment` its team (-1 any), `object` named objects,
  `object-type` types (`tex`). The detectors see the living characters: the player, the squad,
  the NPCs (the test tools' and those the triggers spawn).
* **Occupancy** (FUN_001f8db0). One in the volume becomes an occupant; after 0.1 s it sends
  ENTER_DETECTOR (and ENTER_DETECTOR_FIRST when no occupant had been counted in), with itself as
  the subject, and its own ENTER_DETECTOR_TO_SELF. One out of it for 0.2 s (in sum: the game
  never resets the time out) leaves: EXIT_DETECTOR and EXIT_DETECTOR_TO_SELF; the last one
  leaving: EMPTIED_DETECTOR. A disabled detector does nothing, unless it has `h_e25d95e3`: then
  it keeps its occupants silently and enables itself when one leaves (sdm_e34's two teleport
  plates).

## Spawn-triggers

* **The template.** A spawn-trigger's `h_1299668b` names a placement: a character, an effect
  object, a pickup. As the trigger links (0x209210 -> FUN_000dfe90) the placement is taken out
  of the world and counts how many triggers spawn it (-1 for a re-armable one: `h_0f95f576`). Each
  spawn (FUN_000dfd90) clones it, and the last one of a counted template takes the template
  itself: that's how the tutorial's Tex (only trigger_spawn8 spawns him) seems to "move". So
  every placed character a spawn-trigger names (the tutorial's eight Red Hand, Tex and the five
  marines) is absent until spawned.
* **Spawning.** OPERATE while armed spawns at once, or after `h_eab6952f` s (e40's delayed
  spawns); a trigger is armed once by its constructor (0x1fd680-0x1fd6e7), and SPAWN_RESET arms a
  re-armable one again (the tutorial's lava bursts: TIMER_PULSE -> SPAWN_RESET, then OPERATE).
  The copy goes to the trigger's own transform (FUN_00209620), named `h_14f15a34` ("tex_1",
  "TUTORIAL_terrorist_5": what the scripts and later triggers call it; a re-armable trigger's
  copies get no name, the runtime makes one up). It sends SPAWNED to its listeners, and the
  trigger OPERATED to its targets with the copy as the subject (FUN_002099e0). The triggers that
  list the spawn-trigger listen to its copies (FUN_001fe8a0, FUN_002096e0): router99's twelve,
  counting the base's kills.
* **Which way it faces.** Down the trigger transform's -z, as the level's cameras look. Checked
  on Tex: after the opening (take03 at 32 s) the game's follow camera looks past him west, at the
  dark peak and the palms; the demo shows that facing -z, and the rock face east of the start
  facing +z (the start points' way in arena.rs). `h_131de144`, a world point the copy's AI is
  given (FUN_00017b60), isn't where Tex faces (wpoint194 is south of him); what the AI does with
  it isn't traced.
* **`h_eeae96e7` is the spawn fade-in** (s), not a "mode": FUN_00209620 writes it to the copy's
  char+0x73c with the fade state +0x734 = 0 and alpha +0x738 = 0, and FUN_0011bd00 raises the
  alpha by dt / that each frame. 2 s for Tex and the marines (the farcast), 0 for the Red Hand
  (at once). The demo doesn't draw the fade: a copy appears at once (logged).
* **In the world** (`spawn_copy`): a character template becomes a character of its type with the
  placement's weapons, personality and team (as `BF_TEST_NPC` spawns a placement, #129), set to
  fight (stand and fire at what it sees: the patrols and covers are step 8, #227); an effect
  object plays its effect type's ALE effects once at the transform (the lava bursts, the farcast
  flashes; the farcast-out pad, which the game keeps until its destroyer, plays its emitters once
  here); a pickup template (sdm_e34's 118 re-armable ones) isn't made: the demo keeps the placed
  pickups as before.
* **The squad's start (#102).** A squad member (Brutus, Flint, Hawk, Tex) that a spawn-trigger
  spawns starts at that trigger's transform, named as its copy (`SquadSpawns`, play.rs
  `player_at_start`): the tutorial's Tex at (65.55, 28.53, 74.03), 3.1 m up, dropping to the
  ground as the game's farcast does (the take's Tex lands at (65.55, 25.25, 74.03)); e40's Brutus
  at (-14.0, 37.1, 160.7) (#102's live reading). A member placed in the world instead (m09_a's
  four, whose warp-triggers move them: step 3) starts at its placement. The level's other squad
  members aren't there: the tutorial has Tex alone (the marines are its own spawns). On a map
  without either (the deathmatch maps) the start points and the formation are as before. When
  the member's spawn-trigger fires later (the opening script at 17 s, step 3) it's moved there.
  `BF_TEST_GOTO` still puts the leader where it says.

## Object events

* **DESTROYED**: a character with a trigger name dies (a spawn copy: its spawn-trigger's
  listeners hear it; the squad started at its spawns counts as those copies); a named breakable
  object that a trigger listens to breaks (play_scenery.rs). The event's subject is the object;
  its id 0 and data -1 are a guess (the death's send isn't traced).
* **USE_OBJECT**: a wall panel used (play.rs `doors`): its signal h_eaf8a35b and data h_f1660a0d
  (0x163060), the user as the subject.
* **PICKUP_OBJECT**: a placed pickup taken (play_pickups.rs). The tutorial's listened ones (the
  DNA, the Sentry mines, the L-Shot-50) aren't takeable in the demo yet (step 9, #228).

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

```sh
# the tutorial's base: the plate opens the door and spawns four terrorists
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_TEST_GOTO=-36.4,37.3,-57.3,22.9 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the farcast: Tex spawned by the opening script's router
BF_MAP=tutorial BF_TRIGGER_LOG=1 "BF_TRIGGER_FIRE=trigger_router_spawn_tex,USER1,1" cargo run --bin bf_play
```

## Verified

`BF_MAP=tutorial` (and sdm_e34, e40, m09_a), `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps,
the log read and the frames looked at:

* **Level start** (no hooks): FIRST_UPDATE opens the door h_fa29ecfd that starts open (anim3,
  1.21 s), spawns the gate wall's sentry TUTORIAL_terrorist_5 at (-37.9, 30.3, -9.8)
  (trigger_spawn46), removes the four editor leftovers h_0247aefb round (-98, 18, 44)
  (destroyers 473, 498-500), starts timers 383 / 384 (the west lava pit's bursts every 4 / 6 s:
  trigger_spawn382 / 381 at (-23.0, 14.0, 9.3) / (-20.2, 14.0, 15.7)), 458, 525 and h_0279cdb2,
  and logs what steps 2-9 will run (the opening script, the gamevars, the music and ambience,
  the first radar blip). Tex drops in at trigger_spawn8's (65.55, 28.53, 74.03) facing west; the
  camera behind him shows what the game's does after the opening (take03 at 32 s: the dark peak,
  the palms, the rock face right).
* **The base** (`BF_TEST_GOTO=-36.4,37.3,-57.3,22.9`): plate h_eefcb023 sends ENTER_DETECTOR ->
  router103 -> terrorists 21, 13, 12, 9 at (-95.5, 18.1, 55.2), (-98.4, 18.1, 58.8),
  (-95.0, 18.1, 58.1), (-61.7, 18.1, 19.3), and anim2 opens the door h_06537687 once (frames
  58-120: the door slides open as Tex walks up and in, the ammo and the DNA inside), leads §7.8.
* **The lava yard** (`BF_TEST_GOTO=48.1,-1.5,48.1,-21.5`): the 18 m and 25 m circles at once
  (the chime, the blips, the east pit's burst sounds enabled: steps 4 and 9); plate h_0b18c4c8
  (a human player's) -> router22 -> terrorists 16, 3, 1 at (33.0, 25.1, -69.8),
  (33.5, 25.1, -64.7), (28.7, 19.2, -77.1); plate h_f42fa5b8 -> router473 -> terrorists 30, 29 at
  (73.1, 20.7, -44.6), (69.9, 20.7, -40.8), leads §7.5. Seen from the test tools' free camera with
  `BF_TRIGGER_FIRE=trigger_router473,ENTER_DETECTOR,1`: the two stand in the gully by the stone
  head.
* **The second building** (`BF_TEST_GOTO=74.4,-104.1,64.4,-97.7`): plate h_020415dc ->
  anim15 opens h_0f544611 27 m away (once: then it disables itself, h_137926ff) and spawns
  terrorist 17 inside at (38.3, 18.2, -88.8), leads §7.6.
* **The tutorial's gate panel** (`BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100
  BF_TEST_USE=0.5`): USE_OBJECT -> router349 -> the panel's sound-trigger h_0309a1b9 (step 4:
  its FINISHED opens the gate's two leaves, so the gate waits for it), router252 disables
  Bratton's "use the panel" line, router6 -> the panels' property triggers.
* **sdm_e34 (regression)**: walking through door h_10616962 along its axis
  (`BF_TEST_GOTO=-76.26,-2.04,-62.34,5.64`): plate h_f214c88d -> router h_e09f6796 -> anim
  h_f12ff6e9: ENTER_DETECTOR_FIRST opens it, EMPTIED_DETECTOR closes it 0.2 s after Tex is off the
  plate; the gate from its panel (`BF_TEST_GOTO=-23.23,79.05,-23.7,80.8 BF_TEST_GO_AT=100
  BF_TEST_USE=0.5`): USE_OBJECT -> router h_1c61561d -> both leaves' anim-triggers, once, 4.96 s.
  Its 118 pickup spawn-triggers fire at the start and make nothing (the placed pickups stay).
* **e40 / m09_a**: the squad starts at its spawn-triggers (Brutus at (-14.04, 37.10, 160.70))
  / at its placements (Brutus at (-177.2, 62.2, -225.3)).

## Not done here, or not known

* The classes left to steps 2-9 (#221-#228, above). The tutorial's gate opens on its panel sound's
  FINISHED: with step 4.
* The character's spawn fade-in isn't drawn.
* A character's body in a detector is the demo's cylinder, not its Ipion body against the
  phantom; a dead character leaves a detector 0.2 s after it dies (the game's ragdoll may stay
  in the phantom: not known).
* DESTROYED's and PICKUP_OBJECT's id and data from objects are guesses (0, -1); a death's send
  isn't traced.
* Usable game objects (`reticule-action` 1 consoles, e34's) don't send USE_OBJECT yet: only
  world buttons have the panel prompt.
* Animated objects other than doors (the tutorial's looping props h_e5b1f2c8, h_0cbffd71) aren't
  moved; an anim-trigger's AC_PAUSE / STOP / RESET_TIME and the looping and oscillating modes
  are read as forward or backward.
* The start points' facing (+z, arena.rs) hasn't been checked against footage; spawn-triggers
  face -z (above).
