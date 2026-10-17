# Level triggers (`src/bin/play_triggers.rs`, `src/bf/triggers.rs`, #220)

A level's logic is its triggers: routers, timers, calculators, spawn-triggers, detectors,
anim-triggers and some twenty more classes, wired to each other in the level file. The tutorial
has 500 (58 routers, 54 timers, 47 spawn-triggers, 27 plates, 14 proximity detectors ...). This
is the runtime that runs them, the first step of the tutorial (#28, plan step 1); the later
steps plug their classes into it (gamevar and score since step 2, #221, with the game flags and
the cash they work on: [campaign.md](campaign.md); script, letterbox and warp since step 3,
#222, with the cutscenes they drive: [cutscenes.md](cutscenes.md); sound and dialog since step
4, #223, with the voices and the level's music they play; message, input, menu and event: step
5, #224). Everything here is read from default.xbe's trigger code (the addresses are in the
code's comments and in [code-map.md](code-map.md), "Levels and triggers"); nothing is fitted.

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
  OPERATEs; its script starts once: [cutscenes.md](cutscenes.md)).

## The classes it runs

| Class (tutorial count) | What it does | Code |
|---|---|---|
| router (58) | Passes every event but FIRST_UPDATE to its targets. With h_f0d6857f it passes only those its OPERATE entries accept, as `out-signal` when that isn't -1 (e.g. `h_e9413a18`: ENTER_DETECTOR -> USER_SIGNAL1) | 0x208fc0 |
| timer (54) | OPERATE starts it, TIMER_FULL_RESET stops and reloads it, RESET_COUNTDOWN_TIME reloads it. Running and enabled it counts `current-value` down; at 0 it sends TIMER_PULSE (with the subject its OPERATE came with), then stops and reloads (h_f92b0b15), reloads (enable-looping), or stays at 0 until reset | 0x200190, 0x2002f0 |
| calculator (12) | INC / DEC / MODIFY_COUNTER (by the `<calculator value>`) / SET_COUNTER_FROM_SIG_DATA, then its test h_015914ab (0 ==, 1 <, 2 >, 3 <=, 4 >= its `target`, unsigned): CALCULATOR then TRUE, or FALSE, with the counter as data | 0x1ff5f0 |
| spawn-trigger (47) | See "Spawn-triggers" | 0x209ac0, 0x209bd0 |
| plate-detector (27), proximity-detector (14) | See "Detectors" | FUN_001f9680, FUN_001f8db0 |
| anim-trigger (7) | OPERATE plays its entry's clip (`animation-archetype`) on its first object, forward or backward (`wwit-debugging`, AM_: 3-5 backward) at its `time-scale`; with AC_PLAY or AC_RESUME it sends ANIM_START when it starts forward after playing backward (its first forward play too), ANIM_BACK_START when it starts backward after playing forward (or ANIM_START for the other modes). The demo moves doors and gates with it ([maps.md](maps.md), "Doors and gates"); other animated objects (the tutorial's two looping props) are logged | 0x204d00, FUN_00204bb0 |
| sound (73) | PLAY_SOUND plays its sound (alike everywhere, or at its place), PLAY_DIALOG queues it on the world's dialog queue, STOP / PAUSE / RESUME; FINISHED when its length has run (#223: see "Sound- and dialog-triggers") | 0x206450, 0x206690 |
| dialog (4) | Its character says its line; FINISHED once it's said (#223) | 0x206d20, 0x206ee0 |
| message h_e9b3ae8e (37) | OPERATE: OPERATED, then the HUD's two hint lines set from its own (shown ones blink in, the others hidden) (#224: see "Hints, button gates and panels") | 0x2078e0 |
| input h_e2b9c3a2 (37) | Enabled, TRUE to its targets every frame its virtual button is held on any pad (#224) | 0x2008a0 |
| menu (10) | OPERATE opens its game-options menu over the game (a tutorial panel) and sends OPERATED; the panel's A and Y send MENU_ACCEPT / MENU_Y_BUTTON, then FINISHED once it's closed (#224) | 0x2082d0, 0x208430 |
| event (2) | An AI event about an object it listens to whose number is its `event-id`: EVENT_MATCH, then it disables itself (#224) | 0x202dd0 |
| destroyer (5) | OPERATE marks it; its update removes its objects (a listed spawn-trigger's copy), then disables itself. The update runs the base update first, so an OPERATE on its own FIRST_UPDATE removes them in that same update (the tutorial's four props at t 0). It removes a character the triggers spawned, never a member of the demo's squad (no level's destroyer lists one) | 0x200d70, 0x200e40 |
| waypoint (h_10ecb459, 31), evaluate (6) | Their base actions only: a waypoint trigger marks AI route points; the evaluate-triggers' random tests pick ambient lines (both no-ops here) | |
| gamevar (15) | See "Gamevar-triggers" | 0x1ffc40, FUN_001ffb80 |
| script (10) | OPERATE asks its script's player (by `script-name`) to play it from the start, pause, resume or stop, by its entry's `<script command>`; the start waits for the player's update, so two in a frame start it once (#222: [cutscenes.md](cutscenes.md)) | 0x205000, FUN_00204ec0, FUN_00204e40 |
| letterbox (7) | OPERATE brings the letterbox in over its `show-duration`, STOP takes it out over its `hide-duration` (FUN_00073460 / FUN_000734b0); 0.5 s each without one (the constructor, 0x207599) | 0x207480, FUN_00207410 |
| warp (18) | OPERATE moves its objects (a spawn-trigger's copy) to its `world-point`: on the ground under it, facing down a direction-point's -z or as it was for a world-point; a dead character stays, and one not spawned yet has no copy to move. Its fade-in h_0ede4719 is 2 s without one (0x200734; not drawn) | 0x200650, FUN_00200480, 0x200400 |
| score (2) | OPERATE: its `amount` x (1 + 0.1 x the difficulty, with h_11819686) to the campaign squad's cash, under its key h_19ca8cf3 with h_08f457d8, else the null name (every score-trigger's is the null name: the cash's own totals). The tutorial's: score510 5000 on the primary's timer pulse, score509 1000 on the secondary's calculator: $3000 and $600 on STANDARD ([campaign.md](campaign.md), "The cash") | 0x20ba00, FUN_0005c5e0 |

Every other class (menu, message h_e9b3ae8e, input h_e2b9c3a2, event, radar, level-over
h_1f07be84, property h_14cccfaa, the damage plate h_130818c0) runs its base actions; each of its
own actions is counted as used, as its handler counts it, and logged with the step that runs it
("h_11313ffd (menu) OPERATE on USER_SIGNAL2: not run yet (step 5, #224)").

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
  in the world instead (m09_a's four, whose warp-triggers move them: #222) starts at its
  placement. It carries its template's inventory, as the copy is the placement
  (`SquadSpawn::dress`): its `<weapon>`s (the tutorial's Tex two MK-ASLTs, h_0c3db625, 48 a clip,
  the icon take03 shows) and its ammo, which sets the squad's reserve of that type (play_ammo.rs:
  his 500 of type 2, take03's 48 / 500). The level's other squad members aren't there: the
  tutorial has Tex alone (the marines are its own spawns). On a map without either (the
  deathmatch maps) the start points and the formation are as before. When the member's
  spawn-trigger fires later (the opening script at 17.09 s, #222) it's moved there; if the member
  isn't in the world to move (the map editor's stand-in leads), nothing is made. `BF_TEST_GOTO`
  still puts the leader where it says.
* **A member a spawn-trigger spawns is out of the world until then** (#222). The game spawns Tex
  with the opening cutscene's farcast at 17.09 s (take03's watch log: the template at its
  placement until then, clk 17102). The demo keeps him out (`Player::absent`: not drawn, not
  seen by the detectors, nobody's target, no script or warp binds him) until `spawn_copy` brings
  him in, so the start area's proximity detectors h_ea082132 and h_e30f11a4 send ENTER_DETECTOR
  at 17.27 s, as he lands. Under `BF_TEST_GOTO` or `BF_SCRIPT` the level's start scripts don't
  play and the squad is in the world from t 0, as before ([cutscenes.md](cutscenes.md)).

## Sound- and dialog-triggers

#223, `play_sound_triggers.rs` (a child of play_triggers.rs: its classes are the runtime's).
From the code: the sound-trigger's vtable 0x395d50 (its parser 0x205d00, handler 0x206450,
update 0x206690, disable FUN_00206610 and the helpers FUN_00205fb0-FUN_00206b50), the
dialog-trigger's 0x3952a0 (FUN_00206cb0, 0x206d20, 0x206ee0), the world's dialog queue
(world+0x208: FUN_00179d60, FUN_00179ec0, FUN_0017a0f0, FUN_0017a130) and the sound system
(DAT_00469940, vtable 0x399c40).

* **Attributes** (0x205d00; the constructor 0x1fce50's defaults): `sound-object` the sound
  library entry; `enable-looping`; `Type` (the SND_IS_ names at 0x3bf200): 0 GLOBAL (heard alike
  everywhere), 1 POSITIONAL (at its place, 3D), 2 AMBIENT_OUTDOOR, 3 AMBIENT_INDOOR,
  4 POSITIONAL_INDOOR, 5 POSITIONAL_OUTDOOR (below); `max-distance` and h_0269d394 its max and min
  distance (-1: the library entry's `h_fd40e332` and `falloff`); `volume` (h_eff0deab, 0-100; -1:
  the entry's own h_eff0deab, 0-1, or 1 without one); h_1f625953 / h_05cad875 its voice's fade-in
  / fade-out (s); h_15bc1769 "only while a player is within its `max-distance`"; h_e725a1a5 the
  squad's chatter hushed while it plays (not modelled); `HARDPOINT` and `priority` (h_e146dc86)
  aren't read by these paths.
* **A sound's length** is the sound system's (its vtable +0x64, 0x17f620): the file's size over
  its format's bytes a second (FUN_00183c30: Xbox ADPCM counts 36 bytes a channel for each whole
  64 of the rate). The sound bank's own `time` attribute agrees (terminal_03: 12852 / 12384 =
  1.0378 s, `time="1.037791"`).
* **PLAY_SOUND, OPERATE**: it's on. Without h_15bc1769 it starts at once (FUN_002062a0): a
  one-shot still playing isn't restarted, a looping one playing goes on; otherwise its time is
  its sound's length and its voice starts (FUN_002060c0), 2D for Types 0, 2 and 3 and for any
  sound whose library entry's `falloff` is -1 (FUN_0017c170 makes a 3D voice only otherwise),
  else at its first object's place (a spawn-trigger's copy, a placed object) or its own.
* **A 3D voice's distance**: every one gets a rolloff curve as it's made (FUN_00181770 for a
  buffer, 0x1836eb for a stream: IDirectSound*_SetRolloffCurve with the five gains at 0x38dae8,
  1, 0.375, 0.125, 0.0625 and 0), which the dsound library's attenuation (FUN_00314e64) spreads
  over five equal stretches from the min distance to the max, each linear from the last point
  (1.0 at the min) to its next: full within the min, 0.375 two fifths of the way, nothing at the
  max and beyond (the distance is held at the max). The buffers aren't made with
  DSBCAPS_MUTE3DATMAXDISTANCE (their flags are DSBCAPS_CTRL3D alone, 0x1817c8), but the curve
  ends at 0, so they're silent past it all the same. The listener is where the character the
  camera follows stands (FUN_00178c10: the camera's facing, the character's place).
* **With h_15bc1769** its update starts it once the player's character is within its
  `max-distance` (its count, FUN_002062a0) and its voice once the listener is too (the same
  character: both together), and stops it, fading out, once none is (FUN_00206b50); a one-shot
  that has played is off. So a door's sound counts its length from the frame the player is within
  range, its voice with it: sdm_e34's door sounds start 0.07 s after their ANIM_START, beside
  the player, and have played their 0.93 s at 1.00 s.
* **The end**: its update counts the time down by dt while it runs; when it's out (a one-shot)
  FINISHED (25: its iD, no subject, data -1) goes to its targets, for Types 0 and 1
  (FUN_001fcf00), and it's off and stopped. The count starts in the frame it plays, so FINISHED
  comes on the frame the length has run since that frame's start.
* **STOP_SOUND, STOP, STOP_DIALOG**: stopped and off; the voice fades out over its fade-out.
  **PAUSE_SOUND / RESUME_SOUND** pause and resume the voice and the count (no tutorial trigger
  has them).
* **The fades** are the voice's envelope (the sound system's +0x50, FUN_0017f330 -> the voice's
  +0x2c, 0x181c90: IDirectSoundBuffer_SetEG with the fade-in as its attack and the fade-out as
  its release, x 48000 / 512 samples, sustain 255); their shape is the sound chip's, not the
  game's code. Measured: take04's tutorial_01 stop (a 3 s fade-out) falls linearly in dB, about
  60 dB over the fade (its level at 0.25 s steps from 199.0 s: 0.65, 0.30, 0.25, 0.16, 0.09,
  0.03), so the music is all but gone 1.3 s in; the fade-ins (take09, take11) rise linearly.
* **PLAY_DIALOG** queues its sound, once, on the world's dialog queue, which plays one line at a
  time, each for its length, 2D at its library volume, the next as soon as one has ended
  (FUN_00179ec0, the world update's last step); FINISHED once its line is done (FUN_0017a0f0),
  on the next frame's update.
* **DISABLE** is its own (its vtable +0x48, FUN_00206610): the voice plays out, but its count
  and queued line are dropped, so no FINISHED comes; its update, which the world runs only while
  +0x3d bit 6 is set, stops once the voice has ended. Its update sends its FIRST_UPDATE even when
  it's disabled (0x2066a4).
* **Types 2-5**, the outdoor and indoor ones, run the same logic and send no FINISHED; they play
  at a level (+0xa0) that follows where the listener stands (FUN_00206880): their `volume` / 100
  raised to 1 by the share of the listeners indoors (3 and 4) or outdoors (2 and 5), and 0 with
  h_15bc1769 while the listener is beyond its `max-distance`; the level moves there at 4 a second,
  a voice starting when it rises from 0, stopping at 0, its volume following (FUN_00206a20). A
  listener is indoors (the sound system's +0xc8 byte for it) when the character its camera
  follows stands on an indoor floor (FUN_00178c10 from the character's ground record bit 0,
  FUN_0012dab0): on the terrain, or on static geometry (a game-object, OT_STATIC_GEOMETRY) or a
  platform (OT_PLATFORM) of light-usage LU_GEOSET, the triangle's own bit (header bit 30,
  `MATERIAL_INDOOR`; no terrain triangle has it); on one of LU_INDOOR always, of LU_OUTDOOR never
  (`light-usage` h_1eb6a442, the LU_ names at 0x3bf954); on anything else it stays as it was.
  39 of the game's 1417 sound-triggers; the levels' ambience beds are these:
  amb_ferix_06 (m01_a, m01_b, m05_b, m05_y), amb_shrike_01 / _02 (m09_a, m09_x, m09_b),
  light_bird_bed_1 (sdm_e40) and sdm_e34's wind are Type 2 at 15-35, so full outdoors and that
  share indoors; amb_shrike_int_01 (m09_c) and m01_a's mechanical_amb_int_1 are Type 3. The
  tutorial has none. The same flag picks a 3D voice's reverb send (0x17f390: the world's indoor
  or outdoor amount, its +0xb58 / +0xb5c; not modelled).
* **The dialog-trigger**: PLAY_DIALOG or OPERATE on its first object's character (the tutorial's
  four name trigger_spawn8, whose copy tex_1 is the demo's Tex, and trigger_spawn76): a dead one
  sends FINISHED at once; a living one says the line (FUN_001267b0) and the update sends
  FINISHED once it's said (FUN_00126810). Here the line plays as a plain sound at the character
  (full for the player's, quieter with distance for another's, as the demo's character sounds
  are). With no such character in the world an entry does nothing, not even counted.
* **Music**: a level's music bank's tracks are its sound-triggers' sounds (resource-type 2), played
  through the music volume and marked as the level's music ([maps.md](maps.md), "Music").
* **The tutorial's** (73 sound-triggers, 4 dialog-triggers):
  * the gate: either panel's USE_OBJECT plays terminal_03 (h_144f3dbb, 1.04 s) at the panel; its
    FINISHED opens the gate's two leaves (anim0, anim1: 2.0 s at time-scale 2.5) and plays
    metal_gate_05 (h_12c4f1ee, 2.53 s) at each leaf; the second's FINISHED starts the gate music
    (tutorial_02, a 3 s fade-in) and its 238 s stop timer (timer530);
  * the opening: Bratton's four lines on the script's USER1s (its 103s, #222):
    baseisguarded (1.97 s) at 3.02 s, envhazards (4.59 s) at 6.18, goingtoguide (6.11 s) at 11.22,
    refresher (6.75 s) at 19.55, whose FINISHED starts timer429 (0.1 s: the co-op hint) and
    timer462 (5 s: BASIC CONTROLS; both step 5, #224);
  * the hint chain's lines (dxud_ai_00047 "[BACK] to check mission objectives." ...) on its
    timers, stopped and disabled by router400's plates; Bratton's PLAY_DIALOG lines (the radar
    explanation, whose FINISHED opens the WEAPONS panel; first patrol, take cover ...) through the
    queue;
  * at the first update the start music tutorial_01 (stopped by timer525 at 168 s, a 3 s
    fade-out), the beds amb_estu_01 and light_bird_bed_1 (within 60 m of the start), and the
    lava's loops and bursts (3D, 30 to 40 m); the mine field's caspian_tut_mellow1 (72 s) and the
    base's caspian_victory1;
  * the save points' "chime" is the AI's voice line dxud_ai_00008 (h_f674fc4f), 2D.

Test hooks: `BF_TRIGGER_LOG=1` lists every sound- and dialog-trigger at the start (its sound and
length, how it plays) and logs what each does (plays, stops, queues, has played, says, the level
a Type 2-5 one heads to); `BF_SOUND_LOG=1` logs each of their voices as it starts and stops (its
volume now and in full, where, its distances, also under `BF_MUTE`); either logs the listener
going indoors or out; `BF_MUTE=1` runs them without a voice; `BF_TRIGGER_FIRE` fires them.

```sh
# the tutorial's gate: the panel's sound, then the gate and its grind, then the gate music
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100 BF_TEST_USE=0.5 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the opening's four lines, from the opening script (#222)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the same lines fired by hand at the script's times, without it (#223's check, before #222)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 BF_SCRIPT=off BF_TRIGGER_FIRE="trigger_sound_bratton_baseisguarded,USER1,3.02;trigger_sound_bratton_envhazards,USER1,6.176;trigger_sound_bratton_goingtoguide,USER1,11.217;trigger_sound_bratton_refresher,USER1,19.551" cargo run --bin bf_play
# an outdoor and an indoor bed: m01_a's start into the lit hall (indoors at 6.8 s)
BF_MAP=m01_a BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_TEST_GOTO=79.6,-100.5,90.1,-60.7,132 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
```

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
* **AI events** (RESERVE_4EVT, 17): a character's EVT_ events (its health under 75, 50 and 25
  percent of its most; EVT_SOUND_ENEMY_NOISE as a stub) to the event-triggers listening to it
  only (FUN_001fe440), with the character as the subject (#224, play_hints.rs `ai_events`).

## Hints, button gates and panels

The message, input, menu and event classes (#224) are the runtime's, dispatched from
`Triggers::deliver` and `Triggers::update` like the rest; their code is in two child modules,
`play_hints.rs` (the message trigger's hint lines, the input trigger's virtual buttons, the
event-trigger's AI events) and `play_panels.rs` (the menu-trigger's panels), with their own page:
[hints.md](hints.md). Their state lives in `Triggers::hints` (one per level). A menu-trigger
whose menu game-options doesn't define (the tutorial's trigger_menu509) opens nothing and never
sends FINISHED; a panel up stops the game's clock as the objectives menu does, so the timers
wait under it. `BF_HINT_LOG=1` logs the lines, the gates' presses, the AI events and the
panels; `BF_TEST_VB` presses a virtual button and `BF_TEST_PANEL` closes the panels.

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
  `BF_FLAG_SET`, `BF_CASH_LOG`, `BF_TEST_OBJECTIVES`): [campaign.md](campaign.md). The
  cutscenes' (`BF_SCRIPT`, `BF_SCRIPT_LOG`): [cutscenes.md](cutscenes.md).

```sh
# the tutorial's base: the plate opens the door and spawns four terrorists
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_TEST_GOTO=-36.4,37.3,-57.3,22.9 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the farcast: the opening script's router spawns Tex at 17.09 s (#222)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SCRIPT_LOG=1 cargo run --bin bf_play
# or by hand, without the script (the squad is then in the world from the start)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SCRIPT=off "BF_TRIGGER_FIRE=trigger_router_spawn_tex,USER1,1" cargo run --bin bf_play
```

## Verified

`BF_MAP=tutorial` (and sdm_e34, e40, m09_a), `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps,
the log read and the frames looked at:

* **Level start** (no hooks; #220's check, before the opening played: since #222 the level's
  start plays it, and Tex comes at 17.09 s, [cutscenes.md](cutscenes.md)): FIRST_UPDATE opens
  the door h_fa29ecfd that starts open (anim3, 1.21 s), spawns the gate wall's sentry
  TUTORIAL_terrorist_5 on the wall walkway at (-37.9, 27.25, -9.8), 3.07 m below trigger_spawn46, removes the four editor leftovers h_0247aefb round (-98, 18, 44)
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
  BF_TEST_USE=0.5`): USE_OBJECT -> router349 -> the panel's sound-trigger h_0309a1b9, router252
  disables Bratton's "use the panel" line, router6 -> the panels' property triggers. Since #223
  (`BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_DOOR_LOG=1`, 15 fps): terminal_03 plays at 1.07 s at the
  panel (5.3 m away); its FINISHED at 2.07 s opens both leaves (x2.5, open at 4.00 s) and plays
  metal_gate_05 at each leaf; the second's FINISHED at 4.53 s starts the gate music (fading in
  over 3 s) and timer530 (238 s). The frames show the gate shut while the panel sound plays
  (frames 16-30) and rising from frame 31-33. take11 has the same chain: the panel sound's onset
  at 718.60 s, the grind's 1.05 s later as the gate starts to rise (between 719.75 and 720.0 at
  4 fps), the music from ~722.5 s rising over ~3 s (its audio envelope).
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
* **Sound- and dialog-triggers (#223)**, `BF_MUTE=1` with `BF_TRIGGER_LOG=1 BF_SOUND_LOG=1`
  unless said:
  * the tutorial's opening lines, fired at the script's times (the `BF_TRIGGER_FIRE` above, 10
    fps): baseisguarded 3.10-5.00 s, envhazards 6.20-10.70, goingtoguide 11.30-17.40, refresher
    19.60-26.30, whose FINISHED starts timer429 and timer462, which sends BASIC CONTROLS its USER2
    at 31.30 (step 5). take03 matches: the lines' onsets (cross-correlated with their waves) 0.07
    to 0.16 s after the script's times, as its pictures lag (the camera cuts 0.1-0.15 s), and the
    co-op hint (FINISHED + 0.1 s) on screen by clk 26.46-26.56 s;
  * at the first update (frame 0) the start music, amb_estu_01 and the east lava loop play, the
    bird bed waits for a player within 60 m;
  * `BF_TRIGGER_FIRE` on the cues: h_0378b249's TIMER_PULSE stops tutorial_01 fading over 3 s;
    h_17ebe01b plays caspian_tut_mellow1 once (a second ENTER does nothing: its entry is once)
    and its TIMER_PULSE stops it over 3 s; h_04a4176a's sting plays and its USER1 stops it over
    0.5 s;
  * the dialog queue: the radar explanation (ticket 0, 6.96 s) plays at once, first patrol
    (ticket 1, queued at 1.53 s) at 8.00 when it has ended; their FINISHEDs at 8.07 (to the
    WEAPONS panel's menu-trigger) and 10.60, the frame after each ends;
  * trigger_dialog_tex_yeahyeah: tex_1 (the demo's Tex) says dxmi_tex_m00003 at 1.00 s, FINISHED
    at 4.07; trigger_sound422 disabled while its burst plays: no FINISHED; h_e82b011b's ENTER
    stops its line and disables it;
  * h_15bc1769: walking north from (40.5, 0) the bird bed starts at 59.3 m from it; walking back
    south it stops at 60.2 m;
  * sdm_e34 (regression): the doors' sounds come from their sound-triggers (on at the
    anim-trigger's ANIM_START / ANIM_BACK_START, playing once Tex is within their 7 m
    `max-distance`: door h_196659f4's opening one at 0.07 s with its count, played at 1.00, its
    closing one at 1.47), the gate's metal_door_5 at the panel's use (within its 15 m); the
    doors and the gate move as before; its music caspian_action20 from its FIRST_UPDATE trigger;
    its Type 2 wind bed at 1.0 from the gate's start outdoors, 0.20 from the doors' start in the
    barracks (`BF_TEST_GOTO=-76.26,-2.04,-62.34,5.64`: indoors, as the frames show). e40 and
    m09_a: their music from their triggers (e40's h_0378b249, m09_a's trigger_sound_music41), no
    fault;
  * the beds (the fix round, each level muted for 30 frames): at the first update m01_a, m01_b,
    m05_b and m05_y play amb_ferix_06, m09_a and m09_x amb_shrike_01, sdm_e40 light_bird_bed_1,
    at 1.0 (the start is outdoors); m09_b starts indoors (a hive chamber), its amb_shrike_02 at
    0.15, with strange_insect_bed_1 and the tunnel music; m09_c starts indoors, amb_shrike_int_01
    at 1.0; m14_d plays shrike_distantspace_001a and shrike_tunnel_bed (its outriders' Type 5
    engines wait for a player within 15 m). m01_a walking from its start
    (`BF_TEST_GOTO=79.6,-100.5,90.1,-60.7,132`): mechanical_amb_int_1 (Type 3) starts at 0.05 when
    Tex is within its 20 m (6.33 s), and as he steps through the doorway into the lit hall
    (6.80 s, indoors) amb_ferix_06 heads down to 0.15 and the hum up to 1.0;
  * the rolloff curve: the tutorial's east lava loop (max 40) is silent from the start, 92 m off;
    a west burst at 33.5 m from its min 30 plays at 0.52, at 38.8 m at 0.04;
  * unmuted (`BF_MUSIC_VOLUME=0.2 BF_AUDIO_LOG=1`, the gate run, ~6 s; before the fix round):
    five voices playing (sinks) by 2 s, the three music-bank tracks decoded.

## Not done here, or not known

* The classes left to steps 5-9 (#224-#228, above).
* The character's spawn fade-in isn't drawn.
* The member check's roster test (0x8d430) is assumed to pass every squad type.
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
* Sound-triggers (#223):
  * The opening script's USER2s to router443 (6.575 s) and router441 (11.025 s) silence the lava
    pits after its lava shot (#222). Runs that don't play it (`BF_TEST_GOTO`, `BF_SCRIPT`) have
    the west pit's bursts (every 4 / 6 s) and the east pit's loop play on from the start (heard
    only within their 40 m: the rolloff curve);
    `BF_TRIGGER_FIRE="trigger_router443,USER2,6.575;trigger_router441,USER2,11.025"` sends them.
  * The squad's chatter isn't hushed by an h_e725a1a5 line (FUN_00141100 / FUN_00141180 on the
    chatter manager world+0xbe8: while its list isn't empty no chatter is said, and the first
    line stops the chatter playing at priority 10 or under).
  * A dialog-trigger's line is a plain sound, not the character's voice (its priority 10 over
    chatter, its mouth); Bratton's lines have no radio filter.
  * 3D voices are a volume only (the demo's sounds have no left and right) and have no reverb:
    the level's I3DL2 environment (the world's +0xb54, the sound system's +0x78) and each 3D
    voice's send to it, the world's indoor or outdoor amount as its listener stands (0x17f390),
    aren't modelled. The listener and h_15bc1769 measure from the player character's feet (which
    point of the character FUN_00178c10 and 0x8d660 take isn't traced). The demo has the one
    listener; the game counts a share of up to four (split screen) for Types 2-5.
  * A stop's flag 0 (FUN_002063c0) is read as "with the fade-out" from take04, where the start
    music fades over its 3 s as timer525 runs out. The fade-out's dB-linear shape and its 60 dB
    are measured (above), not read from the code.
  * Types 2-5's beds are checked only in the demo's logs and frames: no take covers a level
    with them (the tutorial has none).
  * The demo's ADPCM decoder makes 65 samples of a 36-byte block, where the game reckons 64 for a
    sound's length: a voice runs ~1.5% past its FINISHED and is left to run out, not cut.
  * The takes' timers run ~3-4% fast against their watch logs' app clock: timer525's 168 s stop
    of the start music fades from clk ~163.2 s in take04, timer462's 5 s after the refresher
    opens BASIC CONTROLS at clk 31.08-31.10 s (take03 / take04) where the code says 31.30, while
    the script's and the sounds' times match the clock (Tex's spawn at 17.09, the co-op hint at
    FINISHED + 0.1 s). The tester reads it as the app clock running slow (0.94-0.96) through the
    watcher's pauses: the code's 31.30 stands; compare durations with that in mind.
