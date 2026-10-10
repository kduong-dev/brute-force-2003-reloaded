# Test map developer tools (`src/bin/play_testtools.rs`, #111; `src/bin/play_testworld.rs`, #116)

Demo tooling for trying features out on the test map ([play.md](play.md), "Test map"),
not the game: the keys, speeds and layout are the demo's choices, none of them used by the
game's controls. A panel on the right lists the open tool's keys; the controls panel (H, top
left) lists them all. They work on any map of a test session: `--test` starts on the test map
(`BF_MAP=<level>` with `--test`: on that level), and L switches to any other.

| Key | Tool |
|---|---|
| F | **free camera** on / off. Mouse: look (once the mouse is captured); WASD: fly along the view; Space / Ctrl: up / down; Shift: 4x faster (6 m/s, 24 m/s). The character stands still meanwhile (its inputs are dropped). **Left click**: teleport the controlled character to the ground under the crosshair, **right click**: to the ground under the camera, both facing the camera's way, the follow camera behind them. F alone goes back without moving them |
| N | **NPC menu**: Up / Down or the wheel picks Brutus, Flint, Hawk or Tex; J sets friend / ENEMY, B dummy / fight (for the next one; enemy dummy at first); click or Enter spawns one on the ground under the crosshair, facing the controlled character. N or Esc closes it (Esc keeps the mouse captured) |
| J / B / Delete | outside the menu, on the NPC under the crosshair (one the ray passes through, else the nearest within 1.2 m of where it meets the ground, the dead too): switch its team / switch dummy and fight / remove it. Shift+Delete removes every NPC |
| O | **object menu**: every pickup type, every hand weapon (the rack's list), each grenade type and the breakable scenery, in groups (81 entries). Up / Down picks, Page Up / Down jumps a group; a see-through copy (alpha 0.6, a faint blue glow) stands on the ground under the crosshair; the wheel turns it 15 degrees a notch; click or Enter puts one there, and the menu stays open for more. O or Esc closes it. **In the free camera** the copy hangs in the air on the crosshair's ray, 4 m out (Shift + the wheel: 1-40 m, 0.5 m a notch; short of whatever the ray meets first by 0.3 m, and on the ground if that's where it ends up), with a cyan line down to where it will land and a ring there; a loose one placed there drops (see below), scenery hangs where it's put (a yellow ring round its foot instead) |
| Y | **sky menu**: none, the map's own, or the sky of any level whose level file names one; click or Enter shows it over the map in place of its own, with that level's background colour |
| U | **music menu**: off, the map's own, or any level's music bank, mixed as the game mixes a level's (its music with its ambience bed under it); click or Enter plays it; Left / Right: the previous / next bank at once |
| L | **level menu**: the test map, the squad deathmatch maps, the deathmatch arenas and every mission zone, from the game's level list; click or Enter loads it in the same window, the tools with it |

* **NPCs** are `Player`s added at the end of the squad with `npc` set, so everything that acts
  on the squad acts on them: the player's shots (with instant kill), blasts, Sentries,
  ragdolls, blood. They aren't in the formation, can't be taken control of (1-4, the hand-over
  after a death) and don't show on the portraits, health bars or speech icons. Friends are on
  the squad's team (0), enemies on `ENEMY_TEAM` (1).
  * An enemy sets off the squad's Sentries, and the crosshair isn't green on it.
  * A **dummy** stands where it was put, facing the way it was put, a target for the player:
    the AI never fires at it (as BF_TEST_HOSTILE's held squadmate; the product owner's
    choice: the squad shoots only enemies set to fight). It doesn't dive from grenades.
  * One set to **fight** stays put (but dives from a grenade landing near, as the squad
    does) and aims and fires at the nearest hostile in sight, and the squad AI fires at it. This is the squad AI's fire, which until now never had a target
    (`play.rs`: `hostile_in_sight`, `ai_fire`): an AI character sees anyone on another team
    within its weapon's range (at least 20 m) with nothing of the world or the pillars between
    its eyes (1.6 m up) and their chest (1.1 m up), all round; it faces them, turns the gun onto
    them and fires in its bursts (the existing random delay, bursts and pauses); the aim is
    exact. Its shots stop at the first body on their line but its own, a teammate in the way
    included, and hurt by the held weapon's damage when they get there (`AiHits`). In the main
    game everyone is on team 0, so nobody has a target and nothing changes. The target
    (`Player::ai_target`) is the AI's only: it's dropped when control passes to a character
    (the number keys, the hand-over after a death) and never kept by the controlled one, whose
    gun and body follow the crosshair from the first frame.
  * NPCs take no pickups (medkits and fruit are the squad's).
  * Removing one takes its body; the gun it dropped, its blood and its DNA stay.
  * The menu lists the data's other character types (militia, mutant, feral_colonist,
    shadoon, the hounds ...) as "not yet, #110": `Player` needs a squad character's model and
    clips. #110 adds them to `npc_types`.
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
  * a weapon lies on its side; walking into it takes it as from the rack, and it's gone;
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
* Not done: moving NPCs, removing placed objects, saving a layout, enemy species (#110).

## Test hooks

Times in s of the controlled character's clock; several entries of one hook separated by `;`;
yaw and pitch in degrees. `BF_TOOLS_LOG=1` prints each step as it's due and what it did. The
help panel: `BF_TEST_HELP=<s>[;<s>...]` presses H.

| Hook | Does |
|---|---|
| `BF_TEST_FREECAM=<s>[,<x>,<y>,<z>,<yaw>,<pitch>]` | F: the free camera from where the camera is, or put there. `BF_TEST_FREECAM_OFF=<s>`: F again |
| `BF_TEST_FLY=<from>,<to>,<right>,<up>,<forward>[,fast]` | fly that way (each -1..1) |
| `BF_TEST_TELEPORT=<s>[,camera]` | the free camera's left click (teleport to the crosshair's ground); `camera`: the right click |
| `BF_TEST_NPC_MENU=<s>` / `BF_TEST_TOOL_CLOSE=<s>` | N / Esc |
| `BF_TEST_NPC=<s>,<character>,<friend\|enemy>,<dummy\|fight>[,<x>,<z>]` | spawn one at x, z or under the crosshair, facing the controlled character |
| `BF_TEST_NPC_TEAM`, `BF_TEST_NPC_FIGHT`, `BF_TEST_NPC_REMOVE` `=<s>,<n\|aim\|all>` | switch the team / the behaviour of, or remove, the n-th NPC spawned, the one under the crosshair, or all |
| `BF_TEST_PLACER=<s>,<object>[,<yaw>]` | O with that object picked (its label in any case, else the first containing it), turned that way |
| `BF_TEST_PLACE=<s>[,<object>,<x>,<z>[,<yaw>[,<height>]]]` | the object menu's click (at the crosshair: in the air in the free camera); or that object put at x, z, `height` m above the ground (in the air) |
| `BF_TEST_AIR=<s>,<m>` | the free camera's air distance (Shift + the wheel) |
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
("brutus (NPC 3) shoots hawk (1.2 m)").

## Verified

Test map, captures at 15 fps, frames looked at:
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
  respawned with it, 30 / 300).
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
