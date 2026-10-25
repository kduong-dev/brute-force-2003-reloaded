# Test map developer tools (`src/bin/play_testtools.rs`, #111; `src/bin/play_testworld.rs`, #116; `src/bin/play_testscenery.rs`, #185)

Demo tooling for trying features out on the test map ([play.md](play.md), "Test map"),
not the game: the keys, speeds and layout are the demo's choices, none of them used by the
game's controls. A panel on the right lists the open tool's keys; the controls panel (H, top
left) lists them all. They work on any map of a test session: `--test` starts on the test map
(`BF_MAP=<level>` with `--test`: on that level), and L switches to any other.

The map editor ([editor.md](editor.md), #191) is these tools' home: F5 switches to its edit mode
(no character in the world, the edit camera, the tools at the mouse pointer) and back to play,
and `--edit` starts the session there. The menus' keys below are its tool registry's (`TOOLS`,
`open_tools`), the same in both modes; what the menus put down is listed in its `Placements`.
In edit mode "the crosshair" below is the mouse pointer (the crosshair's place with no pointer),
a click is any left click off the panel, and F and its teleports do nothing.

| Key | Tool |
|---|---|
| F | **free camera** on / off (in play). Mouse: look (once the mouse is captured); WASD: fly along the view; Space / Ctrl (or E / Q): up / down; Shift: 4x faster (6 m/s, 24 m/s). The character stands still meanwhile (its inputs are dropped). **Left click**: teleport the controlled character to the ground under the crosshair, **right click**: to the ground under the camera, both facing the camera's way, the follow camera behind them. F alone goes back without moving them |
| N | **NPC menu**: Up / Down or the wheel picks the character type: Brutus, Flint, Hawk or Tex, then every other type the loaded archives define (#128, [character-types.md](character-types.md)) by name, where it's defined and HP ("Shrike Soldier (m09_a, 73 hp)", "Brutus (mp_common/sdm_bf, 75 hp)"; the key added to two that still read the same; nine lines show round the one picked; on the test map all 128 of the data's types, below); J sets friend / ENEMY, B dummy / fight (for the next one; enemy dummy at first); click or Enter spawns one on the ground under the crosshair, facing the controlled character. N or Esc closes it (Esc keeps the mouse captured) |
| J / B / Delete | outside the menu, on the NPC under the crosshair (one the ray passes through, else the nearest within 1.2 m of where it meets the ground, the dead too): switch its team / switch dummy and fight / remove it. Shift+Delete removes every NPC. (In a test session B is this tool's, not the Black button's Medkit shortcut it is in play: `BF_TEST_ITEM_INPUT`'s `black` drives that here, [play.md](play.md) **The item box and the item list**) |
| O | **object menu**: every pickup type, every hand weapon (the rack's list), each grenade type and the breakable scenery, in groups (81 entries). Up / Down picks, Page Up / Down jumps a group; a see-through copy (alpha 0.6, a faint blue glow) stands on the ground under the crosshair; the wheel turns it 15 degrees a notch; click or Enter puts one there, and the menu stays open for more. O or Esc closes it. **In the free camera** the copy hangs in the air on the crosshair's ray, 4 m out (Shift + the wheel: 1-40 m, 0.5 m a notch; short of whatever the ray meets first by 0.3 m, and on the ground if that's where it ends up), with a cyan line down to where it will land and a ring there; a loose one placed there drops (see below), scenery hangs where it's put (a yellow ring round its foot instead) |
| P | **scenery menu** (#185): every level's buildings and static scenery (2,412 pieces of 53 levels, below), by level, each by its type's name (a hash) with its size, parts and how many its level places, the largest first. Up / Down picks, Page Up / Down jumps a level; a see-through copy stands at the crosshair, the wheel turns it 15 degrees a notch; click or Enter puts one there, drawn as its level draws it and solid as there, and the menu stays open for more. P or Esc closes it. In the free camera the copy hangs on the crosshair's ray as the object menu's does (Shift + the wheel: how far), a yellow ring at its foot, and it's put where it hangs |
| Y | **sky menu**: none, the map's own, or the sky of any level whose level file names one; click or Enter shows it over the map in place of its own, with that level's background colour |
| U | **music menu**: off, the map's own, or any level's music bank, mixed as the game mixes a level's (its music with its ambience bed under it); click or Enter plays it; Left / Right: the previous / next bank at once |
| L | **level menu**: the test map, the squad deathmatch maps, the deathmatch arenas and every mission zone, from the game's level list; click or Enter loads it in the same window, the tools with it |

* **NPCs** are `Player`s added at the end of the squad with `npc` set, so everything that acts
  on the squad acts on them: the player's shots (with instant kill), blasts, Sentries,
  ragdolls, blood. They aren't in the formation, can't be taken control of (1-4, the hand-over
  after a death) and don't show on the portraits, health bars or speech icons. Friends are on
  the squad's team (0), enemies on `ENEMY_TEAM` (1); the hooks can put one on any of the game's
  ten teams, `neutral` (2, ALIGN_NEUTRAL) among them, and give it a radar-contact (#141).
  * An enemy sets off the squad's Sentries. The crosshair turns red on an enemy in its lock box
    (out of the box an enemy under its centre stays blue, whatever its radar-contact: a hostile
    skips the radar test), green with its centre on a friend, and yellow with its centre on a
    friend or a neutral given radar-contact 2 or 4; a neutral with its 1 is blue ([play.md](play.md),
    "The crosshair's colours").
  * A **dummy** stands where it was put, facing the way it was put, a target for the player:
    the AI never fires at it (as BF_TEST_HOSTILE's held squadmate; the product owner's
    choice: the squad shoots only enemies set to fight). It doesn't dive from grenades.
  * One set to **fight** stays put (but dives from a grenade landing near, as the squad
    does) and aims and fires at the nearest hostile in sight, and the squad AI fires at it. This is the squad AI's fire, which until now never had a target
    (play_brain.rs `glance`, play.rs `ai_fire`): an AI character sees by its sensor, as a level
    character's brain does (#197, [brain.md](brain.md) "Sight and contacts"): anyone on another
    team within its type's vision-range (its placement's `vision` when that says so) x 1.15 once
    it's a contact x their visibility (a stealthed Hawk's falls to 0, skills.md § Hawk's
    Stealth), within its field of view (its placement's, else its type's `half-fov`: 60 or 90
    degrees either way of its facing, or of its aim while it fires), with nothing of the world or
    the pillars between its eyes (1.6 m up) and their chest (1.1 m up). A contact unseen 15 s,
    stealthed to 0.2 or gone is forgotten. One who hits it becomes a contact at any range, and
    with nothing to fire at it turns to the last place of a contact seen in the last 3.75 s, so
    one shot from behind turns and fires back. It fires at the nearest contact it sees within its
    weapon's range (at least 20 m; the shot's own limit, not sight's). `BF_SIGHT_LOG=1` prints
    each one finding or losing its target and its contacts. It faces them, turns the gun onto
    them and fires in its bursts (the existing random delay, bursts and pauses); the aim is
    exact. Its shots stop at the first body on their line but its own, a teammate in the way
    included, and hurt by the held weapon's damage when they get there (`AiHits`). In the main
    game everyone is on team 0, so nobody has a target and nothing changes. The target
    (`Player::ai_target`) is the AI's only: it's dropped when control passes to a character
    (the number keys, the hand-over after a death) and never kept by the controlled one, whose
    gun and body follow the crosshair from the first frame.
  * NPCs take no pickups (medkits and fruit are the squad's).
  * Removing one takes its body; the gun it dropped, its blood and its DNA stay.
  * Any loaded character type can be spawned (#128). The test map's own levels define 55
    (common's squad, e01's, mp_common's deathmatch and squad deathmatch types, m02_a's and
    m09_a's); it adds the other 73 of the data's 128 from the level archives that define them
    (#180, `BF_TEST_TYPES`, below; character-types.md, "The test map's other types"): the
    terrorists (Red Hand, Terrorist, Subversive), Kingman, Gunthar Ghent, McTavish, the
    Militia, the Supermutants, the ferals, the Seers and the Shadoon, the Hunter Lord, the Fire
    Hounds, e34's, e40's and m14's Shrikes and the campaign's other copies of the marines,
    mutants and Brutus. The menu names each by the archive it came from (`Red Hand (e05, 31
    hp)`), as `name@archive` does. A squad type is that squad slot's character; any other is slot
    `OTHER`, with its own model, clips, face, HP, weapons, flesh and blood decals, and it takes
    damage by its type's factors, hit parts and damage-multiplier (play_damage.rs). It runs its
    type's own behaviour (#129, [enemies.md](enemies.md)): the MOTION table's clips and the
    type's speeds, hit reactions by its personality, the game's ragdoll bodies and leg limits,
    its death line from its type's chatter, and on Shrike flesh green blood and the game's decal
    rule. It says nothing else and has no skill; its ground is the squad's (the sole lift); its
    capsule is the squad's and its skin's glow isn't drawn (#173). The other species' tickets
    (#130-#134) check theirs. See character-types.md, "In play" and "Not done".
  * A **frozen** one (`BF_TEST_NPC`'s behaviour) is a dummy with its brain off, as the takes'
    trainer freeze: no hit reactions (enemies.md).
  * Several characters can share a squad character (an NPC Tex beside the squad's Tex), so
    what reaches a body later tells them apart by `Player::who` (a squad member's character, an
    NPC's number): Energy bolts and Sonic rings land on the body they were drawn to, and the
    thrower's own share of a Frag, Gas, Energy or Sonic blast is the squad thrower's only, not an
    NPC's of the same character.
* **Objects**, put on the ground at the crosshair turned the way the copy shows:
  * a pickup is a level's inventory-object as the grid's (`play_pickups.rs`: `LatePickup`
    seats it on its lowest point, upright as modelled): medkits and fruit can be taken,
    everything is loose. Two entries of one name (the placed and the carried Medkit, two Bio
    Ammo) carry their hash;
  * a weapon lies on its side, a lying gun with a full clip as the rack's (#17): the prompt
    offers it, X held swaps it in (the gun in hand dropped at the feet), and it's gone; walked
    over, a squad member using its ammo type takes its rounds, as from any lying gun;
  * a grenade lies as modelled; walking over it (within 1 m) adds one of its type up to its
    stack-limit ("Took Sentry"), else "<type>: full" (the test map starts with full stacks);
  * **breakable scenery**: the radiation barrel (h_e04e5a0e), the missile rack (h_fbdcd828)
    and the supply crate (h_09a6856d), sdm_e34's kinds (see "Interactive scenery"), standing
    as modelled. One put down joins `play_scenery.rs`'s list as a map's does (its
    `LateBreakable`: hitpoints, centre, debris made ready) and breaks, explodes and chains as
    there: shots, grenade blasts and other objects' damage areas, by the type's factors (the
    rack's Type 10 blast sets off barrels and crates within its 8 m; a barrel's Type 3 cloud
    sets off nothing). The flat floor has no collision, so its model's box stands in for it:
    `ray_hit` stops at the box (shots end on it and set it off, the crosshair meets it);
    characters walk through it, and through what it leaves behind (the rack's stand).
* **In the air** (the free camera): a pickup gets play_pickups.rs's `DropIn` (it falls from
  where it's put, turned as the copy showed, and tumbles to rest as the grid's do), a weapon or
  a grenade `Thrown` from rest (the loose-body physics a dead character's gun uses; it settles
  on whatever face it lands on, not on its side). Breakable scenery hangs where it's put: a
  level's scenery never moves, and its box (shots, the crosshair) and its blast are where it
  hangs; its debris falls from there. The follow camera keeps the ground placement.
* **Scenery** (`play_testscenery.rs`, #185, the first part of the map creation epic #190): any
  level's buildings and static scenery, on the test map or on any level the session is on.
  * Which pieces (`src/bf/pieces.rs`): a level file's `<game-object>` whose object type's
    `<base object-type>` is 2, OT_STATIC_GEOMETRY by the game's OT_ name table (0x3be87c), with
    a mesh archetype. The game fixes such an object's physics bodies and never adds it to the
    world's list of objects that move (FUN_001590a0: types 2, 26 OT_PLATFORM, 31
    OT_WORLDBUTTON). Left out: the types that break (a debris list, h_197caf14: the object
    menu's kind, FUN_00157260) and the doors and gates (a part on a sliding joint, h_fd8f670c:
    they need the door machinery). Every other model a level places has its own element
    (characters, pickups, weapons, liquids, world buttons, platforms, turrets); its other 77
    game-objects are OT_BODY (object-type 0, loose bodies). In the 53 level files: 23,670
    game-objects, 19,671 of them static scenery (3,823 break, 99 are doors), of 675 types; 2
    (mp3) to 80 (m03_b) types a level, 2,412 entries counted level by level.
  * The list: the levels in the level menu's order (the squad deathmatch maps, the deathmatch
    arenas, the missions' zones), each its pieces, the largest first: its type's name (the data
    names none, so the hash), its size across (twice its bounding radius, `h_1ab09530`, about its
    origin), its parts, "no collision" for the few without a collision surface (grass, cables,
    backdrops), and how many its level places ("x11 in its level"). Read when the menu first
    opens, once a session: each level archive up to its objecttypes (its level file, objects and
    objecttypes; not its models, textures or sounds), on 8 threads, with common's types and
    archetypes first as the game looks them up (FUN_00160da0, the level file's libraries
    "common <level>"). `BF_TOOLS_LOG=1` prints every entry ("sdm_e34:0 h_0baf091a 58 m, 19 parts
    (x1 in its level) arch h_0baf091a").
  * Building a piece: from its own archive, the only data it needs (all 53 checked: its objects,
    materials, textures and collision surfaces; two of common's types placed in five missions
    are built from common.tgz, and 16 texture references in m01_a, m05_a and m05_b are common's).
    A picked piece's archive is read on a thread into a Game of its own (`Game::load_scenery`:
    its objects files whole with their .ivd / .ipn, its materials and its textures, with the
    test map's textures shared), and the piece made from it as its level makes its objects
    (`level_scene::object_parts`, the code `spawn_level` uses: cut-outs, glows, grass lit from
    above, glow decals), lit by the map's lights (on a level, its lamps too), casting no shadow.
  * A piece's origin goes where it's put, as its level places it: on the ground, a building's
    floor is the ground's (what its level sinks into its terrain goes under the floor). The
    wheel turns it about its origin.
  * Collision: its parts' collision surfaces (`arena::physics_tris`) are added to the map's
    arena, solid like its own placed objects (`Arena::place`: the arena is copied with them
    added, as it's shared read-only; 7-12 ms on m09_b's 149,166 triangles, none for a piece
    without collision). The squad stands on it and is pushed out of its walls; shots,
    grenades, missiles, the crosshair and the AI's sight meet it. One with no collision surface
    is walked and shot through, as in its level.
  * The flat floor has no arena: the first piece with collision put on it gives it one
    (`Arena::flat`) of its ground, a plane at the floor's height under every point (terrain,
    material 0). Its pillars
    and the boxes of the breakables the object menu put down stay its own (`ray_hit`,
    `hit_normal`, `pillars` keep them on that arena). Before a piece is placed nothing changes;
    after, the floor's code paths are the arena's, which on flat ground give the same (the NPC
    run below: the same combat log, the same frames). What does change: the follow camera stays
    out of the pieces and above the ground, as on a level.
  * Memory: one archive's data is held at a time (the one a piece was last built from, 4-16
    MB: dropped when another's is loaded), and the pieces built: every placed one, and at most 6
    others (`KEEP_UNPLACED`), the least recently picked dropped first with their meshes and
    materials. A map switch drops them all (the menu's state is a map's); the list is the
    session's.
  * Hooks wait for what they need: a pick or a placement made before the list is in waits for
    it, and a placement for its piece to be built (placed the frame it is).
* **Skies** (`play_testworld.rs`): the list is every level whose level file names a sky mesh
  (`<sky><object mesh-name>`, `Level::sky_mesh`), read from each level archive (up to its
  levels-*.xmb) on a thread when the session starts. A pick loads that level's data on a thread
  (`Game::load`, its archive, `Level::load_sky`), makes its layers as a level's own are made
  (`level_scene::sky_layers`, the same layers, order and blending) and keeps them for the
  session (a second pick is at once). While one loads, other sky picks are ignored ("Still
  loading the sky of ..."): each load reads the game data anew. It replaces the map's sky
  (`SkyLayer` entities) and the clear colour becomes its `<background-color>`; "none" puts the map's own clear colour back
  (the flat floor's is black). The camera's far plane goes out to 5000 m, a level's, so the flat
  floor's (Bevy's 1000 m) reaches the layers.
* **Music**: the list is every level of the game's list whose `data/sounds/<level>.xwb` has
  tracks (its first 16 KB read for the names; mp2, mp3, mp6 and splash_screen are empty
  stubs). A pick decodes the bank and reads that level's sound bank for the tracks' Types on a
  thread, then mixes it as `setup` mixes a level's (play.rs `music_mix`: the music, the first
  ambience bed under it at 80%, all beds if it has no music), looping at the music volume. The
  map's own music is stopped first (`LevelMusic`). `BF_MUTE` loads and logs but plays nothing.
  As with skies, picks while a bank loads are ignored.
* **Level switch**: `AppState::Switching` between the maps. Leaving runs `end_play` (every
  entity the map spawned, its sounds and its collision go, as on Backspace), a loading screen
  shows while the level loads on a thread as the front end loads one, plus the test map's
  weapon data, and `begin_play` starts it. Everything a map starts with is made anew by each
  map's start (the tools' state, `Catalogue`, `AiHits`, the scenery's `PLACED_BOXES`, the
  pickups, the squad, the menus' state); the test session's own state stays: instant kill, the
  lists and the skies made ready. Two leaks this found are fixed: switching to the flat floor
  kept the last level's `Doors` and `UsePanel` (its gate panels' use prompt), and a weapon
  taken just before a switch was put in the next map's hand (`PendingSlot`). The deathmatch
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

## Test hooks

Times in s of the controlled character's clock; several entries of one hook separated by `;`;
yaw and pitch in degrees. `BF_TOOLS_LOG=1` prints each step as it's due and what it did, and
the NPC menu's entries once a map's list is made ("NPC menu: 128 entries", then each with its
key). The help panel: `BF_TEST_HELP=<s>[;<s>...]` presses H. For the scenery menu it prints the
list once it's read (each level's count and each piece), each archive loaded (how long, how much
it holds) and each piece built (how long, its meshes and collision triangles and their extent in
its frame, what's held and built, the asset counts and the process's working set), dropped and
placed (where, how high, its turn, the arena's triangles).

`BF_TEST_TYPES` (read as the test map's data loads, on every map of the session) sets which
character types the test map adds to its levels' 55 (`testmap::load_types`,
`Game::load_type_assets`): `all` (the default) the other 73, from the 31 archives that define
them (`TYPE_ARCHIVES`: for each, the first archive by name that has it, as `BF_DUMP_TYPES=all`
loads them); `terrorists` only the 10 terrorist types (chatter-library `terrorist_chatter`, from
e05, e13, e16, m03_c, m08_a and the tutorial); a comma-separated list of archives (`m03_c,e16`)
every type those define that isn't loaded; `none` nothing. The log says what it added ("test
map: character types (all) in 0.9 s: 73 more, 128 in all"; `BF_TESTMAP_LOG` per archive).
Any other value is read as an archive list, so a typo (`terorists`) loads nothing and logs
"test map: no character types from terorists: ..." The archives are read on one thread per
core, at most that many held at once.

Hook-driven sessions run beside someone at the PC with `BF_BACKGROUND=1 BF_MUTE=1`: no gamepad
is read (pads only drive the front end), the mouse's events are dropped (the free camera's look and clicks, and the menus' wheel and
clicks, with it; Esc closing a menu doesn't grab the cursor), and the window opens behind the
others without focus. `BF_WINDOW_MONITOR=<n>` opens it on monitor n. See play.md, "Running in
the background (#163)".

| Hook | Does |
|---|---|
| `BF_TEST_FREECAM=<s>[,<x>,<y>,<z>,<yaw>,<pitch>]` | F: the free camera from where the camera is, or put there. `BF_TEST_FREECAM_OFF=<s>`: F again. In the editor's edit mode the pose moves the edit camera |
| `BF_TEST_EDIT`, `BF_TEST_PLAY`, `BF_TEST_POINTER` | the map editor's F5 both ways and its mouse pointer ([editor.md](editor.md), "Test hooks") |
| `BF_TEST_FLY=<from>,<to>,<right>,<up>,<forward>[,fast]` | fly that way (each -1..1) |
| `BF_TEST_TELEPORT=<s>[,camera]` | the free camera's left click (teleport to the crosshair's ground); `camera`: the right click |
| `BF_TEST_NPC_MENU=<s>` / `BF_TEST_TOOL_CLOSE=<s>` | N / Esc |
| `BF_TEST_NPC=<s>,<character>,<friend\|enemy\|neutral\|team>,<dummy\|fight\|frozen>[,<x>,<z>[,<y>[,<bearing>]]]` | spawn one at x, z or under the crosshair, facing the controlled character, or `bearing` when it's given (#167: degrees, the xemu watch logs' atan2(dz, dx), so 0 faces +x and 90 faces +z; an empty y keeps its default, `x,z,,90`); on the first floor under y (world height; default 50 m above the controlled character, which finds a roof or a gate's top first where there is one). Its team: friend 0, enemy 1, neutral 2 or a number 0-9 of the game's team table. The character: a squad name or 0-3, or any loaded type by its display name (any case, the first loaded: `shrike soldier`), `name@archive` (`shrike soldier@m09_a`) or `name@library` (`shrike soldier@sdm_shrike`), its `<base name>` (`mp_brutus`) or `h_xxxxxxxx`; else a placement of a loaded level by its name (`h_1c5780f8`: its type with the placement's weapons and personality, enemies.md); an unknown one is logged and skipped. `frozen`: a dummy with no hit reactions |
| `BF_TEST_NPC_MOTION=<s>,<n\|aim\|all>,<motion>[,<x>,<z>[,relaxed]]` | drive an enemy of a type of its own in a motion type (`walk`, `run`, `dash`, `move_left`, `backpedal`, `idle`, `dizzy`, `knockdown` ... or its MT_ number) to x, z, or in place: its MOTION table's clip, its type's speed (enemies.md, "Test hooks") |
| `BF_TEST_NPC_TEAM`, `BF_TEST_NPC_FIGHT`, `BF_TEST_NPC_REMOVE` `=<s>,<n\|aim\|all>` | switch the team / the behaviour of, or remove, the n-th NPC spawned, the one under the crosshair, or all. `BF_TEST_NPC_TEAM=<s>,<n\|aim\|all>,<team>` puts them on that team (named as for `BF_TEST_NPC`; one it doesn't know is skipped, with a warning) |
| `BF_TEST_NPC_RADAR=<s>,<n\|aim\|all>,<value>` | give them that radar-contact (every character has 1), as #141's take08 rewrote the Shrike's: 2 or 4 turns the crosshair yellow with its centre on a friend or a neutral; a hostile skips the radar test (red in its lock box, else blue) |
| `BF_TEST_NPC_HP=<s>,<n\|aim\|all>,<hp>` | give them that many hit points, now and at most, as #35 take04's trainer gave its Shrike 999 so it outlived both of Tex's berserk bursts (#121). A spawn's own come with its model the frame after it's spawned: give the HP a little later (`BF_TEST_NPC=0.5,...` then `BF_TEST_NPC_HP=1.0,1,999`). The maximum changes too, and an enemy's heavy-hit and blood-decal thresholds are shares of it ([enemies.md](enemies.md)): at 999 a Shrike takes no heavy hit and lays almost no decals |
| `BF_TEST_PLACER=<s>,<object>[,<yaw>]` | O with that object picked (its label in any case, else the first containing it), turned that way |
| `BF_TEST_PLACE=<s>[,<object>,<x>,<z>[,<yaw>[,<height>]]]` | the object menu's click (at the crosshair: in the air in the free camera); or that object put at x, z, `height` m above the ground (in the air) |
| `BF_TEST_AIR=<s>,<m>` | the free camera's air distance (Shift + the wheel) |
| `BF_TEST_SCENERY_MENU=<s>` | P (the scenery menu opens; its list is read the first time) |
| `BF_TEST_SCENERY=<s>,<level>:<piece>[,<yaw>]` | P with that piece picked, turned that way: `<piece>` its type's or archetype's `h_xxxxxxxx`, or its place in the level's list (0 the largest): `sdm_e34:h_0baf091a`, `e06:0`; without `<level>:`, the first level's that has that type |
| `BF_TEST_SCENERY_PLACE=<s>[,<level>:<piece>[,<x>,<z>[,<yaw>[,<height>]]]]` | the scenery menu's click (the picked piece at the crosshair: in the air in the free camera); or that piece at the crosshair; or put at x, z turned that way, `height` m above the floor there (found under 50 m above the controlled character, as `BF_TEST_PLACE`'s: a roof if a piece is there), where it hangs. Each is placed once its piece is built |
| `BF_TEST_SKY_MENU`, `BF_TEST_MUSIC_MENU`, `BF_TEST_LEVEL_MENU` `=<s>` | Y / U / L |
| `BF_TEST_SKY=<s>,<level\|none\|map>` | that level's sky (its archive's name, e.g. `sdm_e10`, else the first whose title contains it), none, or the map's own |
| `BF_TEST_MUSIC=<s>,<level\|off\|map\|next\|prev>` | that level's music bank, off, the map's own, or the next / previous bank of the list (Right / Left) |
| `BF_TEST_LEVEL=<s>,<level>[;<s>,<level>...]` | switch to that level (`flat` or `test`: the test map). One entry per map in turn, each on its own map's clock: the first on the first map, the second on the map it switched to, and so on |
| `BF_TEST_LEVEL_FAIL=<level>` | loading that level for a switch panics on its thread (the fall back to the map it came from) |

The hooks of this page (`BF_TEST_HELP` too) but `BF_TEST_LEVEL` act on one map of the
session only (a switch would otherwise replay them on each map): the first, or the n-th with
`BF_TEST_ON_MAP=<n>`. `BF_TEST_LEVEL` has an entry per map. The other test hooks (play.md and
the other pages) don't follow `BF_TEST_ON_MAP`; each keeps its own behaviour across a switch:
* on every map, on its clock: those read as a map starts or each frame, e.g. `BF_TEST_GOTO`,
  `BF_CAMERA_*`, `BF_TEST_HEALTH`, `BF_TEST_KILL` (it fires at the squad member
  on whichever map); `BF_TEST_DIE` once per map (`TestDied` is reset with each one);
* once per process, latched in a system's `Local` the first time: `BF_TEST_DROP` (gas),
  `BF_TEST_DETONATE` (grenades), `BF_TEST_TOGGLE_KILL`, `BF_TEST_SUICIDE`, and (by #114's review;
  not on this branch) its test clip hook; on a later map they don't fire again.

`--test` with `BF_MAP=<level>` starts the session on that level (a deathmatch arena alone, as
the switch plays one: by the game's level list).
`BF_TOOLS_LOG=1` also prints what each map starts with and what's left when it ends (see
"Verified").

`BF_COMBAT_LOG=1` also names NPCs ("flint (NPC 1)") and prints each AI shot that meets a body
("brutus (NPC 3) shoots hawk (1.2 m)"). `BF_ENEMY_LOG=1` prints an enemy's MOTION table, its hit
reactions' draws, its motions and drives, its stamina shield and its death line (enemies.md).

## Verified

Test map, captures at 15 fps, frames looked at (the Frag at a rack, the Energy and Sonic, and the
Frag beside a dummy were thrown before #161 made the throw the game's, which now goes faster and
higher: where they land and who stands in a blast don't hold as written; not run again):
* Help: hidden at the start with "H: help"; `BF_TEST_HELP=2` shows the panel with the tools'
  line.
* Free camera and teleport (`BF_TEST_GOTO=0,10,0,10 BF_TEST_FREECAM=1.5,6,5,16,20,-30
  BF_TEST_FLY=2,3,0,0,1 BF_TEST_TELEPORT=3.5`): the view cuts to the pose and flies 6 m along
  the view while Tex stands still; the teleport puts him at 1.96, 4.91, where the crosshair ray
  from the flown-to pose meets the floor, with the follow camera behind him. `camera` put him
  under the camera (30, 40). (With a `BF_TEST_GOTO` goal off the start the autopilot walks him
  back: use one equal to the start.)
* NPCs (`BF_TEST_GOTO=4.5,24,4.5,23.5 BF_TEST_FREECAM=0.5,17,2.5,14,90,-10
  BF_TEST_NPC="1,flint,enemy,dummy,1.5,15;1,hawk,friend,dummy,7.5,16;3,brutus,enemy,fight,4.5,9"
  BF_TEST_NPC_FIGHT=8,1 BF_TEST_NPC_TEAM=11,1 BF_TEST_NPC_REMOVE="13,2;14.5,all"
  BF_COMBAT_LOG=1`): the two dummies stand facing Tex and nobody fires at them; enemy Brutus
  (fight) opens up on the squad's Hawk and they fire back (Hawk, Flint, Brutus), the enemy
  Flint dummy in the line taking some of the squad's Brutus's shots; Brutus is knocked down
  and killed. Flint switched to fight is fired on by the squad and fires back; switched to
  friend, the firing stops. The removals take Hawk, then the rest.
* Spawning at the crosshair (`BF_TEST_NPC=2,hawk,enemy,dummy` with the free camera and the
  menu open): Hawk stands where the crosshair meets the floor; `aim` switched his team and
  removed him.
* A facing (#167; `BF_TEST_GOTO=0,8,0,8 BF_TEST_FREECAM=0.5,0,6,7,0,-35
  BF_TEST_NPC="0.3,flint,neutral,dummy,-4.5,0,,0;0.3,flint,neutral,dummy,-1.5,0,,90;0.3,flint,neutral,dummy,1.5,0,,180;0.3,flint,neutral,dummy,4.5,0,,270;0.3,flint,neutral,dummy,0,-3"
  -- --test`, frame 28 looked at): from left to right the four face +x (screen right), +z (the
  camera), -x and -z (away), logged "turned -90 / 180 / 90 / -0 deg"; the fifth, with no
  bearing, faces Tex as before ("turned 180 deg").
* Character types (#128, character-types.md "Verified"): a Shrike Soldier, Shrike Heavy and
  Shrike Hound (`@m09_a`), a Lesser Priest, a Filthy Mutant and a Feral Outcast by name each
  stand with their own model, skin and gun beside an NPC Flint ("NPC 2: enemy Shrike Soldier
  (dummy) at [2.00, -1.05, 14.00], type h_fcec1024"). The names: `shrike soldier@sdm_shrike`
  gave h_e08868ca, `mp_brutus` the deathmatch Brutus, `h_f678d520` m09's Shrike Heavy, `2` Hawk,
  `SHRIKE HOUND` the first loaded (dm's h_18ce0512); `h_e96829a0` (e34's, not loaded on the test
  map) "no character type h_e96829a0 in the loaded data". The menu (`BF_TEST_NPC_MENU=0.5`): the
  four, then "Brutus (mp_common/dm, 75 hp)", "Brutus (mp_common/sdm_bf, 75 hp)", "Confed Marine
  (mp_common/dm, 80 hp)" ..., "(1 of 55 character types)". The NPC run above, captured with
  main's build and this one: the same frames (within the noise of two runs of main's) and the
  same combat log, the spawn lines now naming the type.
* Sentry (`BF_TEST_GRENADE_TYPE=Sentry BF_TEST_GOTO=4.5,24,4.5,24 BF_TEST_THROW=1,0.1
  BF_TEST_FREECAM="2.5,30,3,40,30,-25;8,9.5,2.5,29,30,-22" BF_TEST_TELEPORT=3,camera
  BF_TEST_NPC=9,flint,enemy,dummy,4.5,22 BF_SENTRY_LOG=1`): Tex sets one down, is teleported
  away with the squad following, an enemy Flint dummy is spawned 1.6 m from it and it goes off
  at the first check, killing him.
* Objects (`BF_TEST_GOTO=4.5,30,4.5,30 BF_TEST_FREECAM=1,4.5,2,25,0,-45
  BF_TEST_PLACER="1.2,medkit,0;3,Confed LZR-50,45;4.5,frag,0;6.3,Confed LZR-50,135"
  BF_TEST_PLACE="2;3.8;5.3;7" BF_TEST_FLY=...`): the copy stands at the crosshair, turned
  45 and 135 degrees as asked, and each click leaves the real one there. Walking over them
  (`BF_TEST_GOTO=4.5,30,4.5,12 BF_TEST_HEALTH=50 BF_TEST_GRENADE_TYPE=Sentry BF_TEST_THROW=0.3,0.1
  BF_TEST_PLACE="0.2,medkit h_f5123ace,4.5,26,0;0.2,sentry,4.5,22,0;0.2,Saryl-45,4.5,18,30"`):
  the Medkit goes into the inventory, "Took Sentry" (9 to 10), "Took Saryl-45" (the character
  respawned with it, 30 / 300; since the rack's swap, the Saryl-45 is a lying gun the prompt
  offers instead).
* Breakable scenery: `BF_TEST_GOTO=6,30,6,29.5 BF_TEST_FIRE=1 BF_CAMERA_PITCH=-0.12
  BF_TEST_PLACE="0.2,missile rack,6,18,0;0.2,radiation barrel,3.5,18,0;0.2,radiation
  barrel,9.5,17,0;0.2,supply crate,1.5,15,0" BF_SCENERY_LOG=1`: Tex's first shot stops on the
  rack (1 hp) and breaks it; its blast (Type 10, x10) breaks both barrels and the crate the
  next frame; the fireballs, the pieces flying, the rack's stand left behind; and the shot no
  longer flies on through it into Flint 18 m behind (it did before `ray_hit` met the boxes).
  A Frag thrown at a rack (`BF_TEST_THROW=1,0.6 BF_CAMERA_PITCH=-0.15`, rack at 6,16, barrel at
  8,11): the blast breaks the rack 4.5 m away, and the rack's area the barrel 9.6 m from the
  blast, outside the Frag's 8 m. A barrel hit next to a rack (`BF_TEST_HIT=1.5
  BF_TEST_HIT_NEAR=3.5,18`): the barrel goes, the rack stays (the data's Type 3 x0). The
  rack's see-through copy shows turned 30 degrees.
* The hand-over (`--test BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=4.5,24,4.5,24 BF_TEST_FIRE=1
  BF_TEST_NPC="1,brutus,enemy,fight,12.5,14" BF_TEST_SELECT=0,2.2 BF_TEST_NPC_FIGHT=4,1
  BF_SHOT_LOG=1`): the squad's Brutus, firing at the enemy Brutus, is taken control of at
  2.8 s; his body turns from 116 degrees off onto the crosshair within 0.4 s (`BF_SHOT_LOG`:
  residual 106 at 2.93 s, 10 from 3.33 s on, the standing aim's own) and his shots follow the
  crosshair (dir along the ray), where before the fix they stayed on the old target for 6 s and
  more.
* Energy and Sonic (`BF_TEST_GRENADE_TYPE=Energy` / `Sonic BF_TEST_GOTO=4.5,24,4.5,24
  BF_CAMERA_PITCH=-0.4 BF_TEST_THROW=1.5,0.3
  BF_TEST_NPC="0.5,flint,enemy,dummy,4.5,18;0.5,tex,enemy,dummy,6,16;0.5,hawk,friend,dummy,3,15"
  BF_TEST_INSTANT_KILL=0 BF_COMBAT_LOG=1`): each bolt and ring lands on its own body, the NPCs
  at their places ("flint (NPC 1)" 1.4 m out, the squad's Flint 2.6 m out, each its own
  damage); Tex, the thrower, takes his own share (33.2 at 7.4 m), the NPC Tex the blast's
  (77.4 at 1.9 m).
* A Frag beside a dummy (`BF_TEST_THROW=1.5,0.3`, enemy Flint dummy at 4.5,19, friend Hawk
  set to fight at 7,19): Flint stands where he was put, hurt (2.6 m); Hawk dives off to 10.8,
  18.0.
* The main game is unchanged (after #85 merged): `BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20
  BF_TEST_FIRE=1 BF_TEST_HIT=2`, 75 frames, the same as main's build but for frames that also
  differ between two runs of main's own build (frames 2-3 and a few 1-level pixels: run to run
  noise); on the test map the Sentry's
  `BF_TEST_HOSTILE=flint,4,0,0.5` run logs the same checks and blast (a few pixels of smoke
  differ: effect seeds come from entity numbers, and the tools add an entity).
* Not tested in a capture: the keys themselves (the hooks drive the same code, but the key
  and mouse handling is only read, not run), PageUp / PageDown, Shift+Delete.

Every character type (#180), `--test` with `BF_TOOLS_LOG=1 BF_COMBAT_LOG=1 BF_ENEMY_LOG=1`,
captures at 15 fps, frames looked at:
* The menu: "NPC menu: 128 entries" (55 with `BF_TEST_TYPES=none`, 65 with `terrorists`),
  among them "Red Hand (e05, 31 hp)", "Red Hand (e13, 128 hp)", "Red Hand (tutorial, 23 hp)",
  "Terrorist (e16, 57 hp)", "Terrorist (m03_c, 25 hp) h_0571d345", "Terrorist (m03_c, 25 hp)
  h_11fe9c5e", "Terrorist (m08_a, 72 hp)", "Subversive (m03_c, 36 hp) h_0de6c6dd", "Subversive
  (m03_c, 36 hp) h_1597807e", "Subversive (m08_a, 88 hp)". After a switch to e16
  (`BF_TEST_LEVEL=0.5,e16 BF_TEST_ON_MAP=2 BF_TEST_NPC_MENU=0.8`) the menu reads "(1 of 128
  character types)" there too (e16's own two come from its level: "71 more").
* The ten terrorists by key in a row (`BF_TEST_GOTO=6,30,6,30 BF_TEST_FREECAM=0.3,6,2.4,24,0,-5
  BF_TEST_NPC="0.5,h_12073efa,enemy,dummy,-0.75,14;..."`, 1.5 m apart): each stands in its idle
  with its model, skin and LZR-23: the Red Hands' red-marked armour, the Terrorists' black, the
  Subversives' olive.
* Fights (`BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=6,30,6,30 BF_TEST_FREECAM=0.3,6,7,36,0,-25`, each
  `enemy,fight` 14 m in front of the squad): e05's Red Hand, m03_c's Terrorist and m08_a's
  Subversive fire on Tex ("Red Hand (NPC 1) -> tex: ... = 2.15"), the squad fires back, and each
  dies (ragdoll, blood) with its death line from terrorist_chatter ("Red Hand's death line
  e0027b57 of 18"). Gunthar Ghent, Edward Kingman, the Militia, McTavish and the Supermutant (HP
  set to 60) all fire and take hits; Ghent and Kingman die with their lines, the squad falls
  before the rest. The Hunter Lord, a Shadoon, the Feral Shaman, the Seer Spy, e13's Fire Hound,
  Corporal Stout and m01_a's Feral Colonist (HP 40) all fire, are hit and die, each with its
  death line.
* `BF_DUMP_TYPES=1` on `--test` gives the full dump's line: "128 character types (0 models not
  loaded, 0 without a motion set, 0 without clips; 30 MOTION clips missing), 22 motion sets",
  and each of the 128 types' blocks (model bones and vertices, face clips, motion set and
  clips, flesh, HP, weapons, chatter loaded) is the same as `BF_DUMP_TYPES=all`'s, which loads
  the whole archives.
* Unchanged: the rack (27 weapons) and the pickups (44); the NPC run above with
  `BF_TEST_TYPES=none` and with the default: the same log, and the frames differ only as two
  runs of `none` differ from each other (a few pixels of effects in 128 of 240 frames, both
  ways). With `none` the `BF_DUMP_TYPES` outputs on `--test` and with `all` are the same as
  before the change (`Game::load_archive`'s parts were split out for this).
* Cost (this PC, 20 threads): the data loads in about 2.9 s against 1.9 s with `none` (the
  types 0.8-0.9 s, 1.7 s on a cold file cache; with the threads bounded to one per core, measured
  later with the PC busier, 1.7-1.9 s while the unchanged weapon step took 1.7-2.1 s instead of
  1.2 s); the window's start to its first frames takes
  8-10 s either way. The process is about 145 MB larger (`BF_TOOLS_LOG`'s "process 1074 MB"
  against 931 MB; `terrorists` 963 MB), mostly the objects' whole .ivd / .ipn of the archives
  whose models it takes. Every level switch of the session loads them again with the test
  map's weapon data.

The world menus and air placement (#116), captures at 15 fps with `BF_TOOLS_LOG=1`, frames
looked at:
* The scan: "53 levels, 51 with a sky, 50 with a music bank (listed in 1.5-2.6 s)": every zone
  of the game's list but sdm_m07 and m07_c has a sky mesh; mp2, mp3 and mp6 have no music.
* Skies on the test map (`BF_TEST_GOTO=0,10,0,10 BF_TEST_FREECAM=0.5,0,3,12,0,12
  BF_TEST_SKY_MENU=0.6 BF_TEST_SKY="2,sdm_e34;5,sdm_e10;8,none;9.5,sdm_e34"`): the black
  background gives way to sdm_e34's mountains and clouds (4 layers at 0,100,0, background 0.76
  0.71 0.52, about 1 s to load), then sdm_e10's hazy olive sky (2 layers at 0,-500,0), black
  again with none, and sdm_e34's at once the second time (made ready already).
* Skies on sdm_e34 (`--test BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.4,16.3
  BF_CAMERA_PITCH=0.3 BF_TEST_SKY="2,sdm_e10;5,e01;8,none;10,map"`): its own clouds, sdm_e10's,
  e01's night sky with its moon (7 layers), the beige background alone with none, its own again.
* Music (the same runs, `BF_TEST_MUSIC=...`): sdm_e10 "music ferix_action8, ambience
  amb_ferix_02", next "sdm_e13: music tmp_full-on1a-f, ambience amb_singe_02", sdm_e40
  "creepy_estuary_mutant_01, light_bird_bed_1", off, the map's own "caspian_action20": the
  pairs maps.md's table lists. On sdm_e34 prev / next go from its own bank: sdm_m03, back to
  sdm_e34, then sdm_e01; after off, next starts at the list's first (sdm_e40).
* In the air (`BF_TEST_GOTO=4.5,32,4.5,32 BF_TEST_FREECAM=0.5,2,2,28,0,-12 BF_TEST_AIR=0.8,5
  BF_TEST_PLACER="1,medkit,0;5,Confed LZR-50,45;8,supply crate,0;10,frag,0"
  BF_TEST_PLACE="4;7;9;11;12,radiation barrel,9,18,0;12,medkit,10,20,0,3"` and flights between):
  each copy hangs 5 m along the crosshair, 2.46 m up, a cyan line down to a ring on the floor;
  the medkit, the LZR-50 and the Frag each drop from there when placed and land on the floor
  under it ("came to rest at [3.67, -0.89, 23.55]" for one, the medkit put 3 m up at 10, 20 at
  [10.68, -1.00, 20.14] 1.6 s later); the supply crate hangs where its copy was, a yellow ring
  round its foot; the barrel put at height 0 stands on the floor.
* On the ground (the follow camera, `BF_CAMERA_PITCH=-0.6 BF_TEST_PLACER="1,supply crate,30"
  BF_TEST_PLACE=3`): the copy stands on the floor under the crosshair, no line or ring, and the
  crate is put there ("on the ground", y -1.05).
* The medkit's copy didn't show in the air (and by the same cause on the ground): its texture's alpha is a
  shine mask (about 0), which the see-through copy took for coverage. The copy of an opaque
  surface now keeps its colours only (its texture with full alpha).
* Switching (`BF_TEST_LEVEL="3.5,sdm_e34;3,flat;3,sdm_e10;3,m01_a;3,flat"`, with an enemy
  Flint, a supply crate, a medkit in the air, sdm_e10's sky, sdm_e40's music and the free
  camera on the first map): six maps in a row, no panic. Each "left <map>: 6 entities remain (6
  from before it, 0 of its own), 0 sounds"; each new one starts with "6 from before it", its own
  sky layers and music only, "squad 3 (0 NPCs), 0 placed boxes" and the tools closed (the free
  camera off). sdm_e34 lists 38 breakables, sdm_e10 13, m01_a 71, the flat floor 0. m01_a loads
  (the player at 0, 0: missions have no start points, levels.md).
* The tools on a map switched to (`BF_TEST_ON_MAP=2`, the test map then sdm_e34 then mp1): the
  free camera, a medkit dropped from the air (it fell on Tex, who took it), a missile rack
  hanging, an enemy Hawk dummy under the crosshair, e40's sky, music; then on mp1 the
  deathmatch arena alone ("squad 0"), with its own 7 sky layers.
* Missions, one per act (`BF_TEST_LEVEL="2.5,tutorial;2,e05;2,m05_c;2,m09_b;2,m14_d;2,sdm_e10;2,flat"`
  with `BF_TEST_LEVEL_FAIL=sdm_e10`): tutorial (71 breakables), e05 (56), m05_c, m09_b (360) and
  m14_d each load and play (at 0, 0: views from inside or under the scenery). sdm_e10's load
  panicked as asked: "sdm_e10 didn't load: back to m14_d", m14_d came back with the note "sdm_e10
  didn't load (the loading thread stopped): m14_d instead" on screen (frames 208-224; now without the reason, see below), and the
  next entry took it to the test map. Each "left ..." line: "0 of its own"; the screenshots on
  their way are left to finish (all 260 frames saved). Picks during a load: "sky pick ignored:
  still loading the sky of sdm_e34", the same for music.
* `--test BF_MAP=mp1`: "squad 0", alone as through L.
* A red frame at each switch: the damage tint's quad (play_grenade.rs `tint`), made anew for
  each map's camera, started at its deepest red for a frame. It starts at the tint of the moment
  (none) now: 0 red frames in 660 over 20 switches (a frame counted red when its mean red is
  over 1.6 times its green and blue), where the earlier run had them on the switches' first
  frames. The first two frames of a level can still show its clear colour alone, before its
  meshes are on the GPU.
* Memory over 20 switches (sdm_e34 and the flat floor in turn,
  `BF_TEST_LEVEL="2,sdm_e34;2,flat;..."`, `BF_TOOLS_LOG` prints the asset counts and the
  process's working set at each map's start and between maps): the asset counts are the same on
  every visit (the flat floor: meshes 150, images 359 (28 MB), materials 1681, sounds 8; sdm_e34:
  meshes 558, images 486 (45 MB), materials 2823, level materials 142, one lamp buffer; no
  clips or graphs), so nothing of Bevy's assets is kept. What grew was each level's collision
  (`arena::Arena`), leaked on purpose ("a few MB a map"): it's freed now (an `Arc`, dropped when
  the next map's replaces it). The working set settles instead of climbing: the flat floor
  918, 987, 995, 999, 1027, 1040, 1076, 1083, 1081, 1079 MB on visits 1-21, sdm_e34 985 ...
  1066, 1075, 1079, 1081, 1089, 1085 MB (private bytes 1262-1367, flat over the last eight
  visits). The first visits' rise is the allocator and the renderer's caches filling once.
* A bank picked before the level list is in (`BF_TEST_MUSIC="0.6,sdm_e10;4,next"`, the list in
  at 1.5 s): next plays sdm_e13, the bank after sdm_e10 (it went to the list's first before):
  the picked level is kept and looked up in the list when it's needed.
* The note for a level that didn't load is short ("sdm_e10 didn't load: flat instead", the
  reason in the log), clear of the radar.
* Capturing across a switch panicked once: `end_play` despawned a screenshot the renderer was
  still finishing. It's left alone now, with its observer (the save to disk: without it the
  last frame before a switch wasn't saved).
* Not tested: the keys and the mouse (Y, U, L, Left / Right, Shift + the wheel, the clicks):
  the hooks drive the same code. A real load failure (none found: the fallback was driven by
  BF_TEST_LEVEL_FAIL), the second step of the fallback (the test map when the map it came from
  fails too). Mission zones other than the six above.

The scenery menu (#185), `--test` with `BF_TOOLS_LOG=1`, captures at 15 fps (60 for the
grenade), frames looked at:
* The list: "test tools: scenery of 53 levels, 2412 pieces (19671 placements of 675 types),
  listed in 0.5 s", the counts the decoded XML gives (`decompiled/xml`, the same rule).
* As in its own level: Bulgar's barracks set (sdm_e34:0, h_0baf091a, 19 parts) hung 40 m over a
  hill of sdm_e34 (`BF_MAP=sdm_e34 BF_TEST_SCENERY_PLACE="0.3,sdm_e34:h_0baf091a,-73,-40,-29,40"`),
  turned as its level turns the original (-29 degrees), and seen from the same offset from each
  ((12, 6, 14) m, `BF_TEST_FREECAM`): the walls and pillars clear of the original's cactus match
  its pixels (95% within 8 levels, mean colour 34.4 38.8 38.5 against 34.5 39.0 38.6). On the
  flat floor from that offset: the same geometry and textures under the flat floor's warmer key
  light (and from 49 m away, its fog: 25 to 90 m, where sdm_e34's starts at 50).
* Collision, on the flat floor, with Bulgar's Kingman silo (sdm_e34:4, h_ebace085, 15 m across,
  14 m tall):
  * Tex running at it from 32 m (`BF_TEST_GOTO=4.5,10,4.5,-60
    BF_TEST_SCENERY_PLACE="0.3,sdm_e34:4,4.5,-30,0" BF_MOVE_LOG=1`) stops at its wall (z -22.5,
    its face -22.4) and stays there, running into it; Flint stops beside him. (At the capture's
    0.4 m steps he ends about 0.1 m from the wall rather than his 0.4 m radius: the arena's
    push-out, on a level's walls the same.)
  * Teleported onto its roof from the free camera (`BF_TEST_FREECAM="0.3,-40,25,-8,0,-80"
    BF_TEST_TELEPORT=2.5`, silo at -40, -15): "teleported tex to [-40.00, 12.36, -11.32]", he runs
    across the roof, "walks off a ledge at [-40.0, 0.0, -22.6]: 13.0 m to the floor" (its edge)
    and "lands at [-40.3, 0.0, -27.7] after a 13.0 m drop".
  * Firing at it from 7 m (`BF_TEST_FIRE=hip`, `BF_SHOT_LOG=1`): every shot ends at 6.7 m, where
    each ended on the floor at 15 m before it was placed; a mark on the wall.
  * A Frag thrown at its wall from 4.4 m (`BF_TEST_GRENADE_TYPE=Frag BF_TEST_THROW=2,0.8
    BF_CAMERA_PITCH=0.1 BF_CAPTURE_FPS=60 BF_GRENADE_LOG=1`): "down 0.50 s after leaving the
    hand, 2.2 m from the thrower", back off the wall; the same throw without it, 11.1 m.
  * Bulgar's 39 m rock ridge (sdm_e34:1): Brutus and Hawk climbed it ("hawk walks off a ledge ...
    4.8 m to the floor"); Tex, running into its steep end, went into it: a slope steeper than
    about 51 degrees (under the 65-degree wall limit, so not a wall) rises more than the 0.5 m
    step in one 0.4 m capture step. That's the demo's floor rule on any level. Since #285
    (`BF_TEST_GOTO=4.5,10,4.5,-60 BF_TEST_SCENERY_PLACE="0.3,sdm_e34:1,4.5,-30,0"
    BF_TEST_SPRINT=1.0 BF_INSIDE_LOG=1`) he runs up its steep end in the ground step's 0.12 m
    steps and along its flank; where the flank steepens past 65 degrees (at 6.1 s, and once more
    before 8.5 s) he's on it as on terrain too steep to stand on and slides down it to its foot,
    and sprints on along it to the goal at z -60, never in it: nothing logged
    ([maps.md](maps.md), Rocks and boulders).
* In the air (`BF_TEST_FREECAM="0.3,-30,4,10,0,5" BF_TEST_SCENERY="0.5,sdm_e34:13,45"
  BF_TEST_AIR=0.6,12 BF_TEST_SCENERY_PLACE=2.5`): the see-through cactus hangs 12 m along the
  crosshair with the yellow ring at its foot, turned 45 degrees; the click leaves it "7.08 m up
  (hangs) turned 45 deg", the copy still at the crosshair as the camera flies on.
* Twelve of sdm_e34's pieces placed over two runs, one a frame as each was built: the barracks
  set, the silo, a water tower, two sheds, a catwalk with its ramp, two rocks, the cactus.
* Common's two pieces (m02_a:46 and m02_a:48, blue hologram markers) built from common.tgz.
* Memory and load (`BF_TEST_SCENERY` picking the largest piece of 30 levels in turn, 0.8 s
  apart; 26 were reached, the first while the list was still being read; three pieces placed on
  the way): each archive read in 0.02-0.08 s on its thread (4-16 MB of vertex, collision and
  texture data held), its Game started in 0.3 ms on the main thread, each piece built in 0-5 ms
  (the barracks' 172 meshes in 3 ms). The process's working set: 1071 MB at the map's start,
  1089 MB once the list was read, 1094-1143 MB while browsing, 1135 MB at the end; images 401,
  at most 542, 431 at the end; at most 7 pieces built and not placed (6 and the one picked).
* A level switch with two pieces on the flat floor (`BF_TEST_LEVEL="4,sdm_e34;3,flat"`): "left
  flat: ... 0 of its own"; back on the flat floor its assets are as before any piece (meshes
  152, level materials 0, buffers 0).
* Unchanged: sdm_e34 (`BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20 BF_TEST_FIRE=1 BF_TEST_HIT=2`, 75
  frames), whose objects `spawn_level` now makes with `object_parts`: frame for frame the same as
  main's build. The flat floor's NPC run (the "NPCs" check above, to 7 s): the same as main's
  build (effects' noise: at most 0.005% of a frame's pixels), and with a piece placed out of view
  (so the floor has its arena): the same combat log (65 lines) and the same frames but for the
  "Placed ..." message.
* Not tested: the keys and the mouse (P, Up / Down, Page Up / Down, the wheel, the click): the
  hooks drive the same code. Missiles and the AI's sight at a piece (they meet the world through
  `hit_normal` and `ray_hit`, as the shots and the teleport's crosshair above do). A piece lit by
  a mission's lamps.
