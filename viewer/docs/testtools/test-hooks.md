*Part of [testtools/](README.md), the former page `testtools.md`.*

## Test hooks

Lamps (#272): `BF_TEST_LAMP=<x>,<y>,<z>,<range>,<r>,<g>,<b>[;...]` puts a lamp (a bevy point
light as the effects' are, colour x (1 - (d / range)^2) x N.L) at each place from the start,
`BF_NO_EFFECT_LIGHTS=1` keeps the light effects' lamps out and `BF_NO_LAMPS=1` the level's
([levels.md](../levels/point-lights.md) "Point lights", "Effect lights").

Times in s of the controlled character's clock; several entries of one hook separated by `;`;
yaw and pitch in degrees. `BF_TOOLS_LOG=1` prints each step as it's due and what it did, and
the NPC menu's entries once a map's list is made ("NPC menu: 140 entries", the 128 types and the twelve
deathmatch class types of #394, then each with its key). The help panel: `BF_TEST_HELP=<s>[;<s>...]` presses H. For the scenery menu it prints the
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
map: character types (all) in 0.9 s: 73 more, 128 in all"; `BF_TESTMAP_LOG` per archive), then how many of
the twelve deathmatch class types (#394, character-types.md: the eight classes and the Kingman Posse's four seats) it
could make from them ("test map: 12 of 12 deathmatch class types (#394)"; `BF_TESTMAP_LOG` each, and why one was skipped).
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
| `BF_TEST_EDIT`, `BF_TEST_PLAY`, `BF_TEST_POINTER` | the map editor's F5 both ways and its mouse pointer ([editor.md](../editor.md), "Test hooks") |
| `BF_TEST_FACE_CAM=<s>,<who>[,<m>[,<rise>]]` | the free camera put `m` metres in front of a character's face and kept there as its head moves (`who`: `me`, a squad name, 0-3): [faces.md](../faces.md) (#255) |
| `BF_TEST_FLY=<from>,<to>,<right>,<up>,<forward>[,fast]` | fly that way (each -1..1) |
| `BF_TEST_TELEPORT=<s>[,camera]` | the free camera's left click (teleport to the crosshair's ground); `camera`: the right click |
| `BF_TEST_NPC_MENU=<s>` / `BF_TEST_TOOL_CLOSE=<s>` | N / Esc |
| `BF_TEST_NPC=<s>,<character>,<friend\|enemy\|neutral\|team>,<dummy\|fight\|frozen>[+follow][,<x>,<z>[,<y>[,<bearing>]]]` | spawn one at x, z or under the crosshair, facing the controlled character, or `bearing` when it's given (#167: degrees, the xemu watch logs' atan2(dz, dx), so 0 faces +x and 90 faces +z; an empty y keeps its default, `x,z,,90`); on the first floor under y (world height; default 50 m above the controlled character, which finds a roof or a gate's top first where there is one). Its team: friend 0, enemy 1, neutral 2 or a number 0-9 of the game's team table. The character: a squad name or 0-3, or any loaded type by its display name (any case, the first loaded: `shrike soldier`), `name@archive` (`shrike soldier@m09_a`), `name@library` (`shrike soldier@sdm_shrike`) or both as the menus show them (`hunter lord@mp_common/sdm_shrike`, #302), its `<base name>` (`mp_brutus`) or `h_xxxxxxxx`; else a placement of a loaded level by its name (`h_1c5780f8`: its type with the placement's weapons and personality, enemies.md); an unknown one is logged and skipped. `frozen`: a dummy with no hit reactions. `+follow` after the behaviour (`fight+follow`, `dummy+follow`, any case; `follow` alone is a dummy that follows; #364): it trails the controlled character as the squad does (**Follower**, above; only a friend acts on it). A word it doesn't know (`dummy+bogus`) is warned about and ignored |
| `BF_TEST_NPC_MOTION=<s>,<n\|aim\|all>,<motion>[,<x>,<z>[,relaxed]]` | drive an enemy of a type of its own in a motion type (`walk`, `run`, `dash`, `move_left`, `backpedal`, `idle`, `dizzy`, `knockdown` ... or its MT_ number) to x, z, or in place: its MOTION table's clip, its type's speed (enemies.md, "Test hooks") |
| `BF_TEST_NPC_TEAM`, `BF_TEST_NPC_FIGHT`, `BF_TEST_NPC_REMOVE` `=<s>,<n\|aim\|all>` | switch the team / the behaviour of, or remove, the n-th NPC spawned, the one under the crosshair, or all. `BF_TEST_NPC_TEAM=<s>,<n\|aim\|all>,<team>` puts them on that team (named as for `BF_TEST_NPC`; one it doesn't know is skipped, with a warning) |
| `BF_TEST_NPC_FOLLOW=<s>,<n\|aim\|all>[,<on\|off>]` | the ' key (#364): switch whether the n-th NPC spawned, the one under the crosshair or all of them follow the controlled character, or set it (`on`, `off`; another word is skipped, with a warning). A level character (a brain of its own) is left as it is. `BF_TOOLS_LOG` prints "t 3.07: NPC 1 (Hawk): friend fight, follow", and with `BF_FOLLOW_LOG` the "no longer follows" of one let go |
| `BF_TEST_KEYS=<s>,<key>[;...]` | that key pressed at that time as a keyboard presses it, for one frame (#364; the window needs no focus, and BF_BACKGROUND drops the real keyboard's): a letter, `semicolon`, `quote` (or `'`), `bracketleft` (or `[`, the gallery's key, #397), `enter`, `delete`, `up`, `down`, `pageup`, `pagedown`, `left`, `right`, `esc`, `space` (`;` separates the entries, so the key is named). The NPC menu by its keys: `BF_TEST_KEYS="0.5,n;0.8,j;1.1,semicolon;1.4,quote;1.7,enter"` opens it, makes the next one a friend that fights and follows, and spawns it at the crosshair; outside it `quote` switches the follow of the NPC under the crosshair, as J and `semicolon` do the team and the fight. `BF_TOOLS_LOG` prints "test tools: key Quote pressed". Only the systems that read the keys in `Update` see the press (the tools, `read_input`, the item box ...): the ones in `PreUpdate` (the Squad Command menu's `menu_input`, the objectives menu, the tutorial panels) run after bevy_input has cleared the frame's presses and never do; use `BF_TEST_COMMAND` for the squad menu ([squad.md](../squad/squad-command-menu.md)) |
| `BF_TEST_NPC_RADAR=<s>,<n\|aim\|all>,<value>` | give them that radar-contact (every character has 1), as #141's take08 rewrote the Shrike's: 2 or 4 turns the crosshair yellow with its centre on a friend or a neutral; a hostile skips the radar test (red in its lock box, else blue) |
| `BF_TEST_NPC_HP=<s>,<n\|aim\|all>,<hp>` | give them that many hit points, now and at most, as #35 take04's trainer gave its Shrike 999 so it outlived both of Tex's berserk bursts (#121). A spawn's own come with its model the frame after it's spawned: give the HP a little later (`BF_TEST_NPC=0.5,...` then `BF_TEST_NPC_HP=1.0,1,999`). The maximum changes too, and an enemy's heavy-hit and blood-decal thresholds are shares of it ([enemies.md](../enemies.md)): at 999 a Shrike takes no heavy hit and lays almost no decals |
| `BF_TEST_PLACER=<s>,<object>[,<yaw>]` | O with that object picked (its label in any case, else the first containing it), turned that way. Since #397 the list's last groups are the two effect types and the 139 ALE effects ([objects.md](objects.md)), so a part of a name can now hit an effect's label when no earlier entry has it: `BF_TEST_PLACER=1,rifle` picks `sonic_shotgun_rifle` (checked: the baseline's "no object rifle"); a part an object's label also has still picks the object, as the groups come in the list's order. Name the whole label to be sure |
| `BF_TEST_PLACE=<s>[,<object>,<x>,<z>[,<yaw>[,<height>]]]` | the object menu's click (at the crosshair: in the air in the free camera); or that object put at x, z, `height` m above the ground (in the air). The object is found as `BF_TEST_PLACER`'s is (the same partial-name rule, the same effect labels) |
| `BF_TEST_AIR=<s>,<m>` | the free camera's air distance (Shift + the wheel) |
| `BF_TEST_EFFECT=<s>,<effect type>,<x>,<z>[,<y>][;...]` | start an effect type (`h_xxxxxxxx` or a name) at sim time `s` at (x, z), on the floor there or at world height `y`: its look and sounds, its damage areas (over their duration, within their range, by their falloff, with their push and the targets' damage-type factors) and its shakes, as a barrel's burst or a grenade's blast starts its own (#373, [effects.md](../effects.md); `h_1a256542` the Molotov's cloud, `h_ece33110` the Supermutant's pound). `BF_DAMAGE_LOG=1` prints each tick, `BF_SHAKE_LOG=1` the shake held on the view. With `BF_TEST_NPC` and `BF_TEST_INSTANT_KILL=0` for victims at distances. **On a body** (#397): `BF_TEST_EFFECT=<s>,<effect type>,<who>` puts the type on a character's body instead (`who`: `me`, the controlled character; `brutus`, `flint`, `hawk` or `tex`; `npc<n>`, the n-th NPC spawned), riding its hips; the same entry again takes it off. The two types are `vengar` (h_fb1e5895, unused in the game) and `pshield` (h_13687abf, an unspawned duplicate of the shields' hit-effect h_1218cdc1, which works the same), named by their ALE effects (the types' own names aren't recovered: the hash of "vengar" is the ALE effect's). A name that isn't an effect type but is an ALE effect's (one of the 139 nothing in the game starts, [objects.md](objects.md): `BF_TEST_EFFECT=2,tornado,0,-8`) plays that effect once there |
| `BF_TEST_RESET_PICKUPS=<s>[;<s>...]` | Home: every pickup taken on the map is put back where it was (the grid's and the object tool's; "Putting pickups back", above). A hook of the first map, or `BF_TEST_ON_MAP`'s, like the others of this page |
| `BF_TEST_SCENERY_MENU=<s>` | P (the scenery menu opens; its list is read the first time) |
| `BF_TEST_SCENERY=<s>,<level>:<piece>[,<yaw>]` | P with that piece picked, turned that way: `<piece>` its type's or archetype's `h_xxxxxxxx`, or its place in the level's list (0 the largest): `sdm_e34:h_0baf091a`, `e06:0`; without `<level>:`, the first level's that has that type |
| `BF_TEST_SCENERY_PLACE=<s>[,<level>:<piece>[,<x>,<z>[,<yaw>[,<height>]]]]` | the scenery menu's click (the picked piece at the crosshair: in the air in the free camera); or that piece at the crosshair; or put at x, z turned that way, `height` m above the floor there (found under 50 m above the controlled character, as `BF_TEST_PLACE`'s: a roof if a piece is there), where it hangs. Each is placed once its piece is built |
| `BF_TEST_SKY_MENU`, `BF_TEST_MUSIC_MENU`, `BF_TEST_LEVEL_MENU` `=<s>` | Y / U / L |
| `BF_TEST_SKY=<s>,<level\|none\|map>` | that level's sky (its archive's name, e.g. `sdm_e10`, else the first whose title contains it), none, or the map's own |
| `BF_TEST_MUSIC=<s>,<level\|off\|map\|next\|prev\|unused:<name>>` | that level's music bank, off, the map's own, the next / previous bank of the list (Right / Left), or (#396) one of the five tracks no level plays: `unused:creepy_estuary_mutant_01`, `unused:amb_singe_02`, `unused:ambience19-2ch`, `unused:caspian_amb_mux_temp` or `unused:ambiencehellish-2ch`, by its name in any case, a part of it (`unused:caspian`) or its place from 0. It's read from its first bank and plays alone, looping, the map's music stopped; `BF_MUTE` loads and logs it ([skies-and-music.md](skies-and-music.md)) |
| `BF_TEST_GALLERY=<s>[,<page>][;...]` | `[` (#397): the gallery of the game's unused pictures opens, at that page if one is given: its number from 1, or a group's name (`faces`, `other`). Entries at later times page through it; `BF_TOOLS_LOG=1` prints the pictures read and each page shown ([gallery.md](gallery.md)) |
| `BF_TEST_LEVEL=<s>,<level>[;<s>,<level>...]` | switch to that level (`flat` or `test`: the test map). One entry per map in turn, each on its own map's clock: the first on the first map, the second on the map it switched to, and so on |
| `BF_TEST_LEVEL_FAIL=<level>` | loading that level for a switch panics on its thread (the fall back to the map it came from) |
| `BF_TEST_SQUAD_MENU=<s>` | T (the squad menu opens, or closes) |
| `BF_TEST_SQUAD_SLOT=<s>,<slot>,<type\|empty>[;...]` | the squad menu's Enter or Delete (#302): that slot gets a fresh one of that type, or is emptied. The slot as `BF_TEST_SELECT` names a squad member, 0-3 (not the keys' 1-4) or `brutus` / `flint` / `hawk` / `tex` for its place, or `me` for the controlled character's (its type changes in place); the type as `BF_TEST_NPC` names one (`shrike soldier@m09_a`, `hunter lord@mp_common/sdm_shrike` as the menu shows its place, `h_fcec1024`, `flint`). One it doesn't know (or whose model isn't loaded) is logged and skipped |
| `BF_TEST_SQUAD_PRESET=<s>,<squad>[;...]` | the squad menu's Enter on a predefined squad (#339): all four slots get that squad's members (the controlled character's changing in place; see "The predefined squads" above). The squad: its number in the menu's list from 0 (the campaign squad is 0, the Terrorist Cell 6, the Kingman Posse 7, the Mutant Horde 8), its name in any case (`shrike raiders`, `Seer Coven`, `Ferguson Platoon`, `Feral Hunting Pack`, `Brute Force`, `Terrorist Cell`, `Kingman Posse`, `Mutant Horde`), its library (`sdm_shrike` or `shrike`, `sdm_bf`, `sdm_feral`, `sdm_seers`, `sdm_ferg`; `common`; `terrorist_cell`, `kingman_posse`, `mutant_horde`), an alias (#365's `kingman`, which is the Cell's, and `monsters`, its names `Kingman's squad` and `Monsters`, kept for the existing recipes; `cell`, `posse`) or a word of its name (`campaign`, `raiders`, `horde`, `monster`). On a squad deathmatch map it needs `--test` (`--test` with `BF_MAP=sdm_e34`): the squad menu and the hook are part of a test session. One it doesn't know is logged and skipped |
| `BF_TEST_SQUAD_PICK=<s>,<row>[,<slot>]` | T with that row of its list picked (a squad as for `BF_TEST_SQUAD_PRESET`, else a character type as `BF_TEST_NPC` names one) and, if given, that slot (as `BF_TEST_SQUAD_SLOT` names one): the picks Up / Down and Left / Right make, so the panel shows what Enter would do; it puts nothing |
| `BF_TEST_SQUAD=<entry>[,<entry>...]` | the test map's start squad beside the player's character (read as each flat-floor start is made, every one of the session): each entry a type as `BF_TEST_NPC` names one, a squad member's type into its own slot if that's free and any other into the first free slot in the keys' order, or `<slot>:<type>` into that slot (`3:shrike heavy@sdm_shrike` with Tex leading: he starts as one); `none`: nobody. Unknown types are logged and skipped |
| `BF_TEST_FULL_SQUAD=1` | the test map starts with the four, as before #302 (over `BF_TEST_SQUAD`) |
| `BF_TEST_COMMAND="<s>:<what>[:<hold s>];..."` | the Squad Command menu's buttons (#319, any map; [squad.md](../squad/squad-command-menu.md) **The Squad Command menu**): `<what>` a member (brutus / flint / hawk / tex or 0-3) holds its d-pad key (1-4) `<hold>` s (0.1, a tap: the hand-over; 0.4 selects and opens the menu), `stand` / `fire` / `cover` / `move` / `medkit` / `toggle` (or y / b / a / x / black / white) presses that order's key, `back` Esc, `accept` / `cancel` the Move To cursor's; on the hooks' clock (the sim time plus the time the menu's pause held it). With it the test map starts with the four (below) |
| `BF_TEST_CURSOR_AT=<x>,<z>` | the Move To cursor takes the floor at x, z as its centre ray's point, as aiming there would (it snaps to the world point nearest, if that's on the screen). `BF_SQUAD_MENU_LOG=1` logs the menu: the level's world points at the start, the selection, the orders, the cursor's point |

Without `BF_TEST_SQUAD`, a hook of the other pages that acts on the squad's members, or a death
whose recipes look at the hand-over after it, starts the test map with the four, as before #302,
so the recipes written for them run as they did, each member at the place it had
(`testsquad::SQUAD_HOOKS`: `BF_TEST_SELECT`, `BF_TEST_KILL`, `BF_TEST_TARGET`,
`BF_TEST_SQUAD_SKILL`, `BF_TEST_HOSTILE`, `BF_TEST_MINE_FRIEND`, `BF_TEST_DIE`,
`BF_TEST_SUICIDE`, `BF_TEST_ORDER`, the squad's orders, #318: [squad.md](../squad/command-modes.md) **Squad
commands**, and `BF_TEST_COMMAND`, the Squad Command menu's buttons, #319: [squad.md](../squad/squad-command-menu.md)
**The Squad Command menu**; `BF_TEST_SQUAD=none` with a death: a lone one, and the respawn). A
recipe that wants the squad for anything else (the formation in the line of fire, the AI's fire,
a blast's other victims) adds `BF_TEST_FULL_SQUAD=1`.
`BF_TOOLS_LOG=1` prints the start squad and why ("test map: the squad at the start: 0 brutus, 1
flint, 2 hawk, 3 tex (tex leads; BF_TEST_SELECT: the four)"), and each change of the menu ("t 1.00:
squad slot 1 (Brutus's place): Shrike Soldier (m09_a, 73 hp) (h_fcec1024) at [1.50, -1.05,
11.25], was empty"). It also lists the predefined squads when the menu first needs them ("squad
menu: 6 predefined squads", each with the type in each place and its key), and a squad put as
"t 1.00: squad Shrike Raiders (mp_common/sdm_shrike) in all four slots", its four slot lines as
above, and "squad Shrike Raiders: 4 of 4 slots put".

The hooks of this page (`BF_TEST_HELP` too) but `BF_TEST_LEVEL`, `BF_TEST_SQUAD` and
`BF_TEST_FULL_SQUAD` act on one map of the session only (a switch would otherwise replay them on
each map): the first, or the n-th with `BF_TEST_ON_MAP=<n>`. `BF_TEST_LEVEL` has an entry per
map. The other test hooks (play.md and the other pages) don't follow `BF_TEST_ON_MAP`; each keeps
its own behaviour across a switch:
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
("brutus (NPC 3) shoots hawk (1.2 m)"); a squad member, shooter or victim, by its type (a Shrike
in Brutus's place: "Shrike Soldier -> ...", play.rs `who_name` by the roster's types, #302).
`BF_ENEMY_LOG=1` prints an enemy's MOTION table, its hit
reactions' draws, its motions and drives, its stamina shield and its death line (enemies.md).
