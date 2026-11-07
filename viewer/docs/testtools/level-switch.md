*Part of [testtools/](README.md), the former page `testtools.md`.*

* **Level switch**: `AppState::Switching` between the maps. Leaving runs `end_play` (every
  entity the map spawned, its sounds and its collision go, as on Backspace), a loading screen
  shows while the level loads on a thread as the front end loads one, plus the test map's
  weapon data, and `begin_play` starts it. Everything a map starts with is made anew by each
  map's start (the tools' state, `Catalogue`, `AiHits`, the scenery's `PLACED_BOXES`, the
  pickups, the squad, the menus' state); the test session's own state stays: instant kill, the
  lists and the skies made ready. A leak this found is fixed: switching to the flat floor kept
  the last level's `Doors` and `UsePanel` (its gate panels' use prompt). (The rack's guns are
  taken by the swap now, so nothing waits across a switch; each flat-floor start fills the
  rack anew, `Rack`.) The deathmatch
  arenas are played alone, as the front end plays them (by the game's level list, also with
  `--test BF_MAP=<arena>`). A zone with no level file puts up the test map, saying so; one whose
  load fails (an error, or its thread stops without a result, e.g. a panic) is noted and the map
  the switch came from loads instead, then the test map; only if that fails too does the program
  stop. `BF_TEST_LEVEL_FAIL=<level>` makes that level's load panic, to try it.
* Not done: moving NPCs, moving and removing placed objects and scenery (#187), saving a layout
  (#188), the breakable types of the other levels (the scenery menu leaves them out; the object
  menu has sdm_e34's three), the enemy species' own behaviour (#110's sub-issues).
* The clicks (#191): `Tools::click`, made once a frame in `before_input`, is the menus' click: in
  play, a left click with the mouse already captured (not the one that captures it), as before;
  in edit mode any left click off the tools' panel (the panel knows the mouse over it,
  `RelativeCursorPosition`).
