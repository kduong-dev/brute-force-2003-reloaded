# Map editor (`src/bin/play_editor.rs`, #191)

The test map's own edit mode: the home of the map creation epic (#190). Editing, the user is out
of the gunplay: no character in the world, no gun, crosshair or squad HUD, a free camera, and
every editing tool on a key with a help panel. F5 plays the map from where the mouse points and
goes back to editing, the camera where it was. Demo tooling, not the game: the keys, the camera
and the layout are the demo's choices (the camera's controls are the level viewer's, `bf_level`:
the right button held looks, WASD flies, E / Q up and down).

```sh
cargo run --bin bf_play -- --edit                 # the test map, editing
BF_MAP=sdm_e34 cargo run --bin bf_play -- --edit  # building on a level
cargo run --bin bf_play -- --test                 # as before: playing, F5 to edit
```

`--test` still starts in play, so every test hook runs as it did; `--edit` in its place starts
the same session in edit mode. F5 switches either way, and so do the hooks (below). The tools
themselves (NPCs, objects, scenery, sky, music, level) are [testtools.md](testtools.md)'s.

## Controls in edit mode

| Key / mouse | Does |
|---|---|
| right mouse button (hold) | look: the cursor hides and the mouse turns the view; let go, the cursor is back where it was |
| WASD | fly along the view |
| E or Space / Q or Ctrl | up / down |
| Shift | four times faster |
| wheel (no menu open) | the camera's speed: x1.25 a notch, 1.5-96 m/s (6 at first), shown in the help |
| the mouse pointer | where the tools act: the ray through it places, spawns and picks, and the menus' see-through copies follow it. With no pointer (the cursor off the window, or `BF_BACKGROUND`) the crosshair's place, marked with a small white cross; while looking, the cross stands where the pointer was |
| left click | the open menu's click at the pointer (place, spawn, pick); a click on the panel isn't one in the world |
| P O N Y U L | the tools (the registry, below): scenery, objects, NPCs, sky, music, level. The key again or Esc closes the menu |
| J / B / Delete | on the NPC under the pointer: team, fight or dummy, remove (Shift+Delete: all) |
| F5 | play from the pointer (Shift+F5: from the map's start) |
| H | hide or show the help |
| Esc | closes the open menu, and nothing else |
| Backspace | twice within 2 s: leave for the front end (the map isn't kept). In play too, in a test session; the first press says so |

Nothing else leaves edit mode: F, the game's keys (1-4, X, V, G, T, Tab, Z, C, M) and the
clicks that teleport in play's free camera do nothing there. A cyan ring with an arrow marks
where F5 puts the squad's leader and the way they'll face (no menu open). The tools' messages
show on a line at the bottom (the HUD's message line is hidden with the HUD).

In play (after F5) everything is as on the test map ([play.md](play.md)): the squad, the follow
camera, the HUD and the controls panel (H), and the tools' menus at the crosshair as before. F5
goes back to editing.

## The modes (`EditorMode`)

A Bevy state, `Play` (the default, and the main game's) or `Edit`, switched by F5 or the hooks:
the key asks for the switch (`edit_input`) and the state's change makes it (`to_edit`,
`to_play`, in `OnEnter`), so `in_state(EditorMode::Edit)` holds exactly while the squad is out.

* **Edit**: no character is in the world. The squad leaves it (`bench`): the controlled
  character's body and the squad's go, the squad's members leave `Squad` (its NPCs stay, in their
  order) and the AI's hits on their way are dropped (counted by place in the squad). Every system
  takes `Player` as the one in control, so a stand-in without a body takes its place
  (`Player::stand_in`): it's never given a body (`spawn_player`), and it counts as dead, so
  everything that acts on characters passes it over (sight and the AI's targets, shots, blasts,
  gas, Energy bolts, Sentries, pickups, doors, the death camera's hand-over), with a death's marks
  counted as made (no blood, pool or memory chip). It stands at the edit camera: the NPCs' sounds
  are heard from there (`play_sounds`), and the NPC menu puts its NPCs facing it. It carries on
  the controlled character's clock, which the test hooks run on (the editor counts it on). The
  HUD (`HudRoot`) and the controls panel are hidden; `read_input`, the follow camera and the
  camera shake don't run; `capture`, the map's pickups and the tools' map audit don't wait for a
  body.
* **The world goes on** in edit mode: NPCs set to fight fight each other (a dummy is nobody's
  target), loose things fall, effects play, a medkit put in the air drops. Freezing it is #192's.
  A level's triggers don't (#230): they're held while edit mode is up. Their timers keep where
  they were, no detector, spawn-trigger or damage plate acts, no script or cutscene they started
  goes on, and the world's events for them (a death, a pickup) wait; back in play they go on
  from there ([triggers.md](triggers.md), "Edit mode"). A cutscene under way doesn't take the
  edit view: no letterbox, no cutscene camera, the HUD left hidden ([cutscenes.md](cutscenes.md));
  one still under way when play comes back has the view again. Checked with `BF_MAP=tutorial
  --test BF_TEST_GOTO=46,-20,46,-20 BF_TEST_EDIT=4 BF_TEST_PLAY=9 BF_TRIGGER_LOG=1`: no trigger
  line from 4.07 s (edit) to 9.07 s (play); the lava yard's timer383 pulsed at 4.00 s and next at
  13.07 s (its 4 s plus the 5 s held), timer384, due at 6.00 s, at 11.00 s.
* **Play**: the test map as it always was.

## Play from here, and back

* **F5 in edit mode**: a fresh squad (`to_play`). Its leader is the character who last led (Tex
  at first, `BF_CHARACTER`), standing where the pointer's ray meets the ground (`ground_point`:
  the floor under what it meets first, a roof or a ledge if that's it; with nothing within 300 m,
  the ground under the camera), facing the camera's way, pushed out of any wall there
  (`player_at`); the others in formation round them, on the floor and out of the walls, as at a
  map's start (`squad_around`, which `squad_at_start` now calls). Shift+F5 (`BF_TEST_PLAY=..,start`)
  puts them at the map's start instead (`player_at_start`, `squad_at_start`). On a level whose
  triggers spawn or place the squad (the tutorial, e40, m09_a) only the members it has come, the
  map's start being each one's spawn ([triggers.md](triggers.md), #220), in the world at once
  once the level's start has run: they don't wait for a spawn-trigger that may have fired while
  editing (#222, [cutscenes.md](cutscenes.md)). The NPCs stay after
  them in `Squad`. The squad is fresh each time: full health and stamina, a full clip, the start's
  grenades (play_grenade.rs `stock_inventory`) and the reserve stocked again (play_ammo.rs
  `stock`); a weapon taken from the rack stays in its slot (it's the session's data). The mouse is
  captured, the menus close.
* **The transition**: the camera flies from the edit camera to the follow camera behind the
  leader in FLIGHT_TIME (0.75 s, eased in and out; the demo's choice), following it as it moves;
  the HUD shows once it's there.
* **F5 in play**: the squad leaves again (`to_edit`), the mouse is freed, the menus close, and
  the camera flies back to the edit camera where it was when play began (or the pose a hook asks
  for; the first time in a session started in play, from where the camera is).
* **A map's start in edit mode** (`--edit`, or a level switch while editing): the squad leaves at
  once and the edit camera is behind and above the map's start, looking at it: the first of 9 m
  back and 5 m up, 6 / 9, 3 / 13, 1 / 18 with nothing of the map between it and the start's
  eyes (sdm_e34's start is in a canyon with a rock behind it: 6 / 9 there), else the one with
  the most room, pulled in 0.3 m short of what's in the way.
* A level switch (L) keeps the mode; the new map starts with nothing placed (the help says so on
  L's line).

## What carries over

**The map as it is, both ways.** Nothing placed is lost in either direction: the switch doesn't
touch the map. What was broken, taken or killed in play stays so: a broken barrel is gone, an NPC
killed lies where it fell. Pickups come back on their own timer as on a level (play_pickups.rs:
taken, hidden, back after their respawn time). The squad isn't the map's: it leaves with its
bodies (the guns they dropped, their blood and their memory chips stay) and comes back fresh. To
start again from the edited state, #188's save and load will do it; until then, what play changed
stays.

## The tool registry and the placed things

* **`TOOLS`**: each editing tool's key, its menu (`testtools::Panel`), its name and what it's for.
  `open_tools` opens and closes the menus from it, in both modes (the tools no longer read their
  own keys), and the edit mode's help lists it. A new tool (#186 liquids, #189 lights, #187's
  selection, #194, #195) adds an entry and a `Panel`.
* **`Placements`**: the one list that owns what the tools have put on the map. Each entry: its
  number (1, 2, ... in the order placed), what it is (`Thing`: an object menu entry by its label and
  kind; a level's scenery by its level, type and archetype; an NPC by its number, type, team,
  behaviour and the level placement it came from), where it was put (an object's lowest point, a
  piece's origin, an NPC's feet) and its turn, whether it was put in the air, and its root in the
  world (an object's or a piece's, marked `PlacedId`; an NPC is `Squad`'s, by its number). The
  object, scenery and NPC tools add to it; J / B and their hooks keep an NPC's team and behaviour
  there; Delete strikes an NPC off (taken off the map, not lost). `track` follows what play does:
  `Here`, `Dead` (an NPC's body) or `Gone` (its root is no more: broken, a weapon or grenade taken;
  an NPC no longer in `Squad`). A thing that drops comes to rest elsewhere: its root has where it is
  now. The list is a map's: a level switch starts the next one empty.
* **`Tools`** (play_testtools.rs) now carries the edit mode (`edit`), the pointer (`pointer`,
  `aspect`), the frame's click in the world (`click`, made once in `before_input`) and the camera's
  speed; `crosshair()` is the ray every tool acts along (the pointer's in edit mode).

What the later tickets plug into: #187 selects with `crosshair()` and `Tools::click`, moves and
deletes through `Placements` (by `PlacedId`), and adds its tool to `TOOLS`; #192 runs its freeze
on `EditorMode`; #188 saves `Placements` with the world menus' sky and music; #193 enters the
editor by setting `EditorMode::Edit` with (or before) `AppState::Playing` (`start_map` benches the
squad then), and leaves it as Backspace does. `EditorMode` isn't reset on leaving for the menu
(#193's to decide).

## Test hooks

On the controlled character's clock (the stand-in carries it on), one map's like the other tools'
hooks (`BF_TEST_ON_MAP`, [testtools.md](testtools.md)).

| Hook | Does |
|---|---|
| `--edit` (command line) | the test session starts in edit mode (in place of `--test`) |
| `BF_TEST_EDIT=<s>[,<x>,<y>,<z>,<yaw>,<pitch>][;...]` | F5 in play: edit mode, the edit camera put there (else where it last was, else where the camera is) |
| `BF_TEST_PLAY=<s>[,start][;...]` | F5 in edit mode: play from the pointer; `start`: Shift+F5, from the map's start |
| `BF_TEST_POINTER=<s>,<x>,<y>` | the edit mode's pointer at that place of the window (0..1 across and down from the top left), as if the mouse were there |
| `BF_TEST_FREECAM=<s>,<x>,<y>,<z>,<yaw>,<pitch>` | in edit mode: the edit camera put there (without a pose: nothing) |
| `BF_TEST_FLY=...`, `BF_TEST_HELP`, `BF_TEST_TOOL_CLOSE` | fly the edit camera; H (the editor's help); Esc (a menu closes) |
| `BF_TOOLS_LOG=1` | each switch (who leaves and comes, where, the edit camera) with every placement and where it is now, each placement as it's added and as it changes |

`BF_TEST_GOTO`'s autopilot drives the leader in play as always: from wherever F5 put them, with
its own camera heading (a goal equal to the start looks down -z). Leave it out to watch a play
from here as it starts.

## Verified

Captures at 15 fps (`BF_MUTE=1 BF_BACKGROUND=1`, `BF_TOOLS_LOG=1`), frames looked at:

* **Edit, play, edit** on the flat floor (`--edit BF_TEST_FREECAM=0.3,4.5,10,40,0,-30
  BF_TEST_POINTER=0.3,0.5,0.65 BF_TEST_SCENERY_PLACE="0.5,sdm_e34:4,-14,-4,0"
  BF_TEST_PLACE="0.5,supply crate,9,20,0;0.5,radiation barrel,11,20,0;0.5,medkit,2,22,0"
  BF_TEST_NPC="0.5,brutus,enemy,fight,4.5,10;0.5,flint,enemy,dummy,0,14;0.5,hawk,friend,dummy,10,12"
  BF_TEST_PLAY=3 BF_TEST_EDIT=9 BF_COMBAT_LOG=1`): from the first frame no squad, HUD or
  controls panel; the help panel, the ring and arrow at the pointer's ground and the cross; the
  silo, the crate, the barrel, the medkit and the three NPCs; the enemy Brutus, set to fight, holds
  fire (his only others are dummies). At 3.13 s "play from [4.50, -1.05, 25.38] facing 0 deg"
  (the ray through (0.5, 0.65) of the window from that pose meets the floor at z 25.38: 14.62 m
  out at 37.1 degrees down, worked by hand), the squad there, the camera flying down to Tex over
  frames 47-58 and the HUD on at the end. The squad and Brutus fight, Brutus dies at 6.87 s
  ("placement 1 ... Dead"). At 9.13 s "edit mode: tex and the squad (3) out of the world, 3 NPCs
  stay; the edit camera at [4.50, 10.00, 40.00]": the HUD gone at once, the camera flying back up
  to the pose it left, Brutus's body and blood where he fell, the seven placements listed the same
  either side of each switch ("7 placed: 1 scenery, 3 objects, 3 NPCs (0 gone, 1 dead)").
* **On a level** (`BF_MAP=sdm_e34 --edit BF_TEST_NPC="0.8,brutus,enemy,fight"
  BF_TEST_PLAY="2,start;7" BF_TEST_EDIT="5;9.5" BF_TEST_POINTER="6.5,0.5,0.7"
  BF_TEST_LEVEL="10.5,flat"`): the NPC spawned under the crosshair's place; play from the map's
  start ("tex leads at [5.47, 37.19, 35.95]", the start point), back to the same edit camera, play
  from the pointer, Brutus killed, back again; the level switch from edit mode starts the flat
  floor in edit mode ("edit mode from the start ... [0.00, 3.95, 9.00]"). The map's audit in edit
  mode reads "squad 0 (0 NPCs)". The first build put the start camera 9 m behind and 5 m up, inside
  the canyon's rock there: the start views above came of it, and sdm_e34 now starts from 6 m back
  and 9 m up over the canyon floor.
* **The tools at the pointer** (sdm_e34, `--edit BF_TEST_HELP=1.0 BF_TEST_PLACER="1.2,medkit"
  BF_TEST_POINTER="1.6,0.35,0.6" BF_TEST_PLACE=1.9`): H folds the help to one line; the object
  menu's see-through medkit hangs 4 m along the crosshair's place, then moves with the pointer
  ("along the pointer" in the panel); placed there it drops in edit mode ("5.55 m up (drops)").
* **From play** (`--test BF_TEST_GOTO=0,10,0,10 BF_TEST_EDIT="1;7" BF_TEST_FLY=1.5,3.0,0,0.5,-1
  BF_TEST_SCENERY=3.2,sdm_e34:13 BF_TEST_TOOL_CLOSE=4.5 BF_TEST_PLAY=5`): edit mode from the follow
  camera's pose, the flight back and up, the scenery menu, Esc closing it and edit mode staying,
  play 72 m out along the crosshair's place (the flight down covers it in 0.75 s), and back to the
  flown pose ([0.00, 7.49, 22.54]).
* **With Hawk's Stealth** (after #123 merged; the first run's hooks, `BF_CHARACTER=2
  BF_TEST_SKILL=4.5 BF_TEST_PLAY="3;10.5" BF_TEST_EDIT=9 BF_SKILL_LOG=1`): Hawk leads, her Stealth
  comes on in play and she fades; F5 takes her and the squad out with it on; the second play brings
  her back fresh, visible, her stamina full, from the same pointer ("hawk leads at [4.50, -1.05,
  25.38]"). The stand-in has no body, so the Stealth's fade and its skill input pass it over.
* **Play mode unchanged**, against the branch's base build, the same hooks: the NPC run
  ([testtools.md](testtools.md) "NPCs", to 9 s): the same combat log (119 lines); the frames the
  same but for the free camera's panel (its up / down line now names E / Q) and a tracer's streak
  (effects' seeds follow entity numbers, and the editor adds its entities). The breakable scenery
  run with the object menu open (`BF_TEST_GOTO=6,30,6,29.5 BF_TEST_FIRE=1 BF_CAMERA_PITCH=-0.12
  BF_TEST_PLACE=... BF_TEST_PLACER=0.5,medkit`, 4 s): 58 of 60 frames identical, the other two a few
  pixels of the first frames' noise.
* Not tested in a capture: the keys and the mouse themselves (F5, Shift+F5, the right button's
  look and the cursor put back, the wheel's speed, H, Esc, Backspace's second press, a click on the
  panel): agents run with `BF_BACKGROUND`, whose mouse events are dropped and whose window has no
  keyboard focus. The hooks drive the same code from the switch on (`edit_input`'s request).
