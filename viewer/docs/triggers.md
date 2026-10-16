# Level triggers (`src/bin/play_triggers.rs`, `src/bf/triggers.rs`, #220)

A level's logic is its triggers: routers, timers, calculators, spawn-triggers, detectors,
anim-triggers and some twenty more classes, wired to each other in the level file. The tutorial
has 500 (58 routers, 54 timers, 47 spawn-triggers, 27 plates, 14 proximity detectors ...). This
is the runtime that runs them, the first step of the tutorial (#28, plan step 1); the later
steps plug their classes into it (gamevar and score since step 2, #221, with the game flags and
the cash they work on: [campaign.md](campaign.md); script, letterbox and warp: step 3, #222;
sound and dialog: step 4, #223; message, input, menu and event: step 5, #224). Everything here is read from
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
| destroyer (5) | OPERATE marks it; its update removes its objects (a listed spawn-trigger's copy), then disables itself. The update runs the base update first, so an OPERATE on its own FIRST_UPDATE removes them in that same update (the tutorial's four props at t 0). It removes a character the triggers spawned, never a member of the demo's squad (no level's destroyer lists one) | 0x200d70, 0x200e40 |
| waypoint (h_10ecb459, 31), evaluate (6) | Their base actions only: a waypoint trigger marks AI route points; the evaluate-triggers' random tests pick ambient lines (both no-ops here) | |
| gamevar (15) | See "Gamevar-triggers" | 0x1ffc40, FUN_001ffb80 |
| score (2) | OPERATE: its `amount` x (1 + 0.1 x the difficulty, with h_11819686) to the campaign squad's cash, under its key h_19ca8cf3 with h_08f457d8, else the null name (every score-trigger's is the null name: the cash's own totals). The tutorial's: score510 5000 on the primary's timer pulse, score509 1000 on the secondary's calculator: $3000 and $600 on STANDARD ([campaign.md](campaign.md), "The cash") | 0x20ba00, FUN_0005c5e0 |

Every other class (script, letterbox, warp, sound, dialog, menu, message h_e9b3ae8e, input
h_e2b9c3a2, event, radar, level-over h_1f07be84, property h_14cccfaa, the damage plate
h_130818c0) runs its base actions; each of its own actions is counted as used, as its handler
counts it, and logged with the step that runs it ("trigger_sound439 (sound) PLAY_SOUND on
FIRST_UPDATE: not run yet (step 4, #223)").

## Gamevar-triggers

A gamevar-trigger (#221; vtable 0x3960a0, parser 0x1ffac0, handler 0x1ffc40) has an operation,
h_015914ab, and three operands as it keeps them: `out`, A (`<h_1554ca7a>`) and B
(`<h_0e5d9bc0>`) (FUN_0018cbe0: each a kind h_18bbf97f and an invert h_1fe1ecd1).

* **An operand** (FUN_001ffb80): kind 0 a game flag by its path (`gameflag`, read as a float; 0
  without it), 1 its `constant`, 2 a random float in [0, 1) x the constant (FUN_00233b90), 3 the
  event's data (its 32 bits read as a float), 4 floor(random x the constant); inverted, 1 - it.
* **OPERATE** works out the operation (the jump table 0x200010): 0 A == B (|A - B| < 1e-5),
  1 A < B, 2 A <= B, 3 A > B, 4 A >= B (1 or 0), 5 A, 6 the out flag's default, 7 A + B, 8 A - B,
  9 A x B, 10 A / B (FLT_MAX for B 0). The entry is counted then (an entry that disables the
  trigger silences what follows), the result turned over if `out` is inverted, and written: to
  `out`'s flag (kind 0, FUN_00077eb0), kept as `out`'s constant (kind 1), or neither (the other
  kinds). VALUE_CHANGED (41) goes to the targets when the result isn't the value it replaced
  (always for the kinds that keep nothing, and for a flag the tree lacks), then TRUE when the
  result isn't 0 (|r| >= 1e-6), else FALSE; both carry the result as their data and the event's
  subject.
* **On a multiplayer map** the gamevars have no flags: a flag operand reads 0 and a write does
  nothing (VALUE_CHANGED still goes out, as for a flag the tree lacks), and the score-triggers
  pay no cash, so a match can't change the campaign's flags or cash for a later mission in the
  same run. The data doesn't need it: no multiplayer map's gamevar names a flag (the mp maps'
  4 to 8 each work on constants, the sdm_ maps have none; the `missions/h_e466ad5c/..` writes
  are e34's, a mission's). That a match has none of the campaign's flags is inferred: the
  session's difficulty getter reads 0 in a split-screen match (live, #129), which it does only
  without a tree or without the flag (0x7c720), where the default flags would give -4.
* The data uses ops 0 (178), 3 (4), 5 (309) and 7 (5); operand kinds 0, 1, 3 and 4. The
  tutorial's 15: four ops 5 (the objectives' `state`: 0 at the first update, h_ed0c1544 and
  h_e0a5fecf; 1 on completion, h_11b9e83d and h_089d8e5f); ten member tests
  `<member>/h_03761692 == 1` (four at the first update: the farcast flashes, the co-op hint's
  calculator496, the V-squad's calculator501; two on the V-squad's plate; four at the end pad, the
  farcast-out flashes); and `missions/<m14>/h_08bbbbc1/state == 1` (a reward of the finished
  campaign: false).

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
  marines) is absent until spawned, and its collision isn't in the demo's arena either (arena.rs:
  Tex's template put a surface 1.7 m above the ground where his spawn stands him).
* **Spawning.** OPERATE while armed spawns at once, or after `h_eab6952f` s (e40's delayed
  spawns); a trigger is armed once by its constructor (0x1fd680-0x1fd6e7), and SPAWN_RESET arms a
  re-armable one again (the tutorial's lava bursts: TIMER_PULSE -> SPAWN_RESET, then OPERATE).
  Re-armable doesn't mean unarmed: the constructor arms every one ((+0xe4 & 0xe8) | 0x28 at
  0x1fd6ba) and the parser only adds bit 2 for `h_0f95f576`.
  The copy goes to the trigger's own transform (FUN_00209620), named `h_14f15a34` ("tex_1",
  "TUTORIAL_terrorist_5": what the scripts and later triggers call it; a re-armable trigger's
  copies get no name, the runtime makes one up). It sends SPAWNED to its listeners, and the
  trigger OPERATED to its targets with the copy as the subject (FUN_002099e0). The triggers that
  list the spawn-trigger listen to its copies (FUN_001fe8a0, FUN_002096e0): router99's twelve,
  counting the base's kills.
* **On the ground.** A character copy stands on the ground under the transform, at once: the
  place call (vtable +0x50, 0x1159c0) runs the position setter 0x1154b0, which casts a ray
  straight down from the point (FUN_0012dc00 -> FUN_000dddd0, up to 50 m) and puts the feet on
  what it meets. take03's watch log has Tex at y 25.25 within 69 ms of his spawn at 28.53 (the
  terrain there: 25.25 in the demo too), no fall. The demo takes the highest floor below the
  point within 50 m (`on_ground`), for the copies and for the squad's start: e40's squad at 26.2,
  10.9 m below its triggers (Brutus's live read, xemu.md); m09_a's two Shrikes 6.7 and 9.5 m
  below theirs; the tutorial's sentry TUTORIAL_terrorist_5 on the wall walkway (27.25), 3.1 m
  below his. An effect stays at the transform (a game object's place, 0x159b70, casts no ray).
* **The squad member check** (FUN_00209900, #221). A copy that's a character of a squad type (its
  type's `h_e78efbb2`: the four, the tutorial's Confed marines) in a mission is checked against
  the session's squad. A clone (a re-armable trigger's, or one of a template several triggers
  name: FUN_000dfd90's count, `Triggers::takes_clone`) is a new character, whose set-up has set
  its type's member flag first (FUN_0012f440). From a trigger with `h_0fa4f510` (+0xe4 bit 5,
  set on nearly every one) it's kept if the session counts it in the squad (vtable +0xb8,
  0x83ee0: its type's save-game slot `campaign/save-game/character-N/h_03761692` true, or a
  player controlling its type, or no slot of its type), else taken away at once: its death
  (+0x144(1), which clears its type's member flag, FUN_00112830), out of the world, and no
  SPAWNED goes out (OPERATED still does). From one without, it sets its type's member flag
  (vtable +0xbc, 0x814b0) and is kept. So the tutorial's three start marines (Gonzales, Cogburn,
  Stout: re-armable spawn-triggers h_0d7313c9, h_167a4273, h_1f7d72e5, `h_0fa4f510` true), whose
  save-game slots aren't alive, never come with Tex's farcast and leave their member flags 0
  (take03-09's `1|0|0|0`); the V-squad's spawn64 and spawn76 (`h_0fa4f510` false) do come, in
  take11, and set theirs (`1|0|1|1`). The slots, the member flags and who else writes them are
  [campaign.md](campaign.md)'s ("The squad's flags"). The roster check before it (0x8d430, the
  world's +0xbec) is taken to pass every squad type: an inference.
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
  spawns starts on the ground under that trigger's transform, named as its copy (`SquadSpawns`,
  play.rs `player_at_start`): the tutorial's Tex at (65.55, 25.25, 74.03), where take03 has him
  after the farcast; e40's Brutus at (-14.0, 26.2, 160.7) (#102's live reading). A member placed
  in the world instead (m09_a's four, whose warp-triggers move them: step 3) starts at its
  placement. It carries its template's inventory, as the copy is the placement
  (`SquadSpawn::dress`): its `<weapon>`s (the tutorial's Tex two MK-ASLTs, h_0c3db625, 48 a clip,
  the icon take03 shows) and its ammo, which sets the squad's reserve of that type (play_ammo.rs:
  his 500 of type 2, take03's 48 / 500). The level's other squad members aren't there: the
  tutorial has Tex alone (the marines are its own spawns). On a map without either (the
  deathmatch maps) the start points and the formation are as before. When the member's
  spawn-trigger fires later (the opening script at ~17 s, step 3) it's moved there; if the member
  isn't in the world to move (the map editor's stand-in leads), nothing is made. `BF_TEST_GOTO`
  still puts the leader where it says.
* **The squad is there from the start; the game's Tex isn't.** The game spawns Tex with the
  opening cutscene's farcast, ~17 s in (take03's watch log: the template at its placement until
  then). The demo has him (and any squad) in the world from t 0, so the detectors round the
  start see him at once: the tutorial's proximity detectors h_ea082132 and h_e30f11a4 send
  ENTER_DETECTOR at t 0.07 instead of after the farcast. That holds until step 3 (#222) plays the
  opening and fires the spawn.

## Object events

* **DESTROYED**: a character with a trigger name dies (a spawn copy: its spawn-trigger's
  listeners hear it; the squad started at its spawns counts as those copies); a named breakable
  object that a trigger listens to breaks (play_scenery.rs). Its id 0 and data -1 are the game's
  (the death's send: the object's +0x144(1), FUN_00157260, #221); the demo's subject is the dead
  object, the game's a character from the dead one's (see "Not done").
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
* The game flags, the cash and the objectives menu have their own (`BF_FLAG_LOG`,
  `BF_FLAG_SET`, `BF_CASH_LOG`, `BF_TEST_OBJECTIVES`): [campaign.md](campaign.md).

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
  1.21 s), spawns the gate wall's sentry TUTORIAL_terrorist_5 on the wall walkway at
  (-37.9, 27.25, -9.8), 3.07 m below trigger_spawn46, removes the four editor leftovers h_0247aefb round (-98, 18, 44)
  (destroyers 473, 498-500), starts timers 383 / 384 (the west lava pit's bursts every 4 / 6 s:
  trigger_spawn382 / 381 at (-23.0, 14.0, 9.3) / (-20.2, 14.0, 15.7)), 458, 525 and h_0279cdb2,
  runs the gamevars (since #221: the objectives' states 0, the member tests, campaign.md) and
  logs what steps 3-9 will run (the opening script, the music and ambience, the first radar
  blip). Tex stands on the ground under trigger_spawn8 at (65.55, 25.25, 74.03),
  take03's height, facing west; the camera behind him shows what the game's does after the
  opening (take03 at 32 s: the dark peak, the palms, the rock face right), and the HUD his
  template's MK-ASLT at 48 / 500, as take03's (frames 0-59, `BF_AMMO_LOG`: type 2 500).
* **The base** (`BF_TEST_GOTO=-36.4,37.3,-57.3,22.9`): plate h_eefcb023 sends ENTER_DETECTOR ->
  router103 -> terrorists 21, 13, 12, 9 on the floor at (-95.5, 15.0, 55.2), (-98.4, 15.0, 58.8),
  (-95.0, 15.0, 58.1), (-61.7, 15.6, 19.3) (their triggers 3.1 and 2.5 m up), and anim2 opens the door h_06537687 once (frames
  58-120: the door slides open as Tex walks up and in, the ammo and the DNA inside), leads §7.8.
* **The lava yard** (`BF_TEST_GOTO=48.1,-1.5,48.1,-21.5`): the 18 m and 25 m circles at once
  (the chime, the blips, the east pit's burst sounds enabled: steps 4 and 9); plate h_0b18c4c8
  (a human player's) -> router22 -> terrorists 16, 3, 1 on the ground at (33.0, 20.1, -69.8),
  (33.5, 20.1, -64.7), (28.7, 15.3, -77.1) (3.9-5.0 m below their triggers); plate h_f42fa5b8 ->
  router473 -> terrorists 30, 29 at (73.1, 14.9, -44.6), (69.9, 15.3, -40.8), leads §7.5. Seen from the test tools' free camera with
  `BF_TRIGGER_FIRE=trigger_router473,ENTER_DETECTOR,1`: the two stand in the gully by the stone
  head.
* **The second building** (`BF_TEST_GOTO=74.4,-104.1,64.4,-97.7`): plate h_020415dc ->
  anim15 opens h_0f544611 27 m away (once: then it disables itself, h_137926ff) and spawns
  terrorist 17 inside at (38.3, 15.5, -88.8), on the floor, leads §7.6.
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
* **e40 / m09_a**: the squad starts on the ground under its spawn-triggers (Brutus at
  (-14.04, 26.20, 160.70), xemu.md's live read) with its placements' weapons, and `BF_SLIDE_LOG`
  shows no fall in its first 6 s (it fell ~10 m before, with fall damage); m09_a's squad at its
  placements (Brutus at (-177.2, 62.1, -225.3)), its two Shrikes from trigger_spawn2 / 749 on the
  ground 6.75 / 9.46 m below their triggers. With `BF_DEATHMATCH=1` (no squad in the world) and
  `BF_TRIGGER_FIRE=h_14640a2e,USER1,1` e40's Brutus spawn makes nothing (no NPC Brutus).
* **The marines' member check** (#221, `BF_FLAG_LOG=1 BF_TRIGGER_FIRE=trigger_router_confed_one_fx,USER1,1;
  ...two_fx...;...three_fx...;trigger_spawn64,USER1,2;trigger_spawn76,USER1,2.2`): each start
  marine's copy is a clone (a re-armable trigger's) whose set-up sets its type's member flag,
  then it's taken away at once (its save-game slot not alive) and its flag goes back to 0; their
  farcast flashes stay disabled (their gamevar tests were FALSE at the first update). spawn64
  and spawn76 (the V-squad, `h_0fa4f510` false, the templates themselves) spawn Stout and
  Cogburn on the ground and set their flags: tex 1, Gonzales 0, Cogburn 1, Stout 1, take11's
  `1|0|1|1`.
* **Destroyers**: m09_a's trigger_destroyer556 (`BF_TRIGGER_FIRE=h_039f8930,USER1,2`) removes
  trigger_spawn2's Shrike; no level's destroyer lists a squad member, so the guard against
  removing one isn't exercised by the data.

## Not done here, or not known

* The classes left to steps 3-9 (#222-#228, above). The tutorial's gate opens on its panel sound's
  FINISHED: with step 4.
* The character's spawn fade-in isn't drawn.
* The member check's roster test (0x8d430) is assumed to pass every squad type.
* The squad is in the world from the start, so the start area's detectors fire before the
  farcast (above); step 3 (#222).
* Placed characters that no spawn-trigger names still add their bodies' collision to the arena
  at their placements (arena.rs), which the game's characters don't; only the templates are left
  out here.
* A character's body in a detector is the demo's cylinder, not its Ipion body against the
  phantom; a dead character leaves a detector 0.2 s after it dies (the game's ragdoll may stay
  in the phantom: not known).
* A death's DESTROYED comes from the object's +0x144(1) (a game object's FUN_00157260, which a
  character's FUN_00112830 runs first): id 0 and data -1, as the demo sends it, but for a
  character its subject is a character from its vtable +0xf4 object's +0x45c (perhaps its last
  attacker: not traced), not the dead object as the demo has it (#221's reading; nothing in the
  tutorial reads it).
  PICKUP_OBJECT's id and data are guesses (0, -1).
* Usable game objects (`reticule-action` 1 consoles, e34's) don't send USE_OBJECT yet: only
  world buttons have the panel prompt.
* Animated objects other than doors (the tutorial's looping props h_e5b1f2c8, h_0cbffd71) aren't
  moved; an anim-trigger's AC_PAUSE / STOP / RESET_TIME and the looping and oscillating modes
  are read as forward or backward.
* The start points' facing (+z, arena.rs) hasn't been checked against footage; spawn-triggers
  face -z (above).
