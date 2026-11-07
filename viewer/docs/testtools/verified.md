*Part of [testtools/](README.md), the former page `testtools.md`.*

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
* NPCs (`BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=4.5,24,4.5,23.5 BF_TEST_FREECAM=0.5,17,2.5,14,90,-10
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
* Sentry (`BF_TEST_FULL_SQUAD=1 BF_TEST_GRENADE_TYPE=Sentry BF_TEST_GOTO=4.5,24,4.5,24 BF_TEST_THROW=1,0.1
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
* Energy and Sonic (`BF_TEST_FULL_SQUAD=1 BF_TEST_GRENADE_TYPE=Energy` / `Sonic BF_TEST_GOTO=4.5,24,4.5,24
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
* Putting pickups back (Home, `BF_TEST_RESET_PICKUPS`; `BF_PICKUP_LOG=1 BF_TOOLS_LOG=1`, frames
  looked at). The grid's two medkits (`BF_TEST_GOTO=0,0,0,0
  BF_TEST_FREECAM='0.5,-1.4,2.5,-12,0,-36.5;1.8,<same>;2.7,<same>' BF_TEST_TELEPORT='1.2;2.4,camera;4.5'
  BF_TEST_RESET_PICKUPS=3.3`: the free camera 5.6 m off looking at their spot, Tex teleported onto
  it, away and back): both lie on the floor in frame 8; Tex lands on them at 1.20 s and takes
  them ("2x Medkit", medkits 2; the spot is bare in frame 28); he is teleported 6 m off at 2.40 s;
  the hook at 3.33 s puts both back ("2 of 44 pickups put back: Medkit at [-2.10, -0.75, -17.60],
  Medkit at [-0.70, -0.75, -17.60]"; "2 pickups put back" under the camera from frame 50): they
  fall and come to rest where and as the first ones did (frame 66 against the frames 15-70 of a
  run with no take: the same two poses, the same places), and the teleport back at 4.53 s takes
  them again ("4x Medkit" with the earlier line's two, medkits 4). The same with the grid's DNA
  canister (a camera at `2.1,2.5,-9`: taken at 1.27 s, put back at 3.40 s, listed again by
  `late_pickups`, taken at 4.60 s) and with the object tool's pickups
  (`BF_TEST_PLACE='0.2,medkit h_f5123ace,3,-3,0;0.2,medkit h_192d5337,4.5,-3,0,2.0'`, the second
  dropped from 2 m: taken at 1.27 s and the editor's placements 1 and 2 `Gone` at 1.33 s; put
  back at 3.40 s, the second dropping again from 2 m (frames 58-60), both `Here` at 3.47 s; taken
  again at 4.60 s and `Gone`; the dropped one came to rest at 4.9, -2.9 both times). In the edit
  mode (`BF_TEST_EDIT=1.8,-1.4,2.5,-12,0,-36.5 BF_TEST_RESET_PICKUPS=2.5 BF_TEST_PLAY=3.5`): "2
  pickups put back" on the edit status line at 2.53 s with both medkits on the floor in the edit
  camera's view; F5 plays from the pointer's place and Tex takes them at 3.67 s. On a level
  (`BF_TEST_GOTO=-51.7,82.1,-51.7,82.1 BF_TEST_LEVEL='0.5,sdm_e34' BF_TEST_ON_MAP=2
  BF_TEST_PLACE='1.0,medkit h_f5123ace,-52.4,82.1,0'`): the medkit placed there is taken at 1.07
  s, "1 of 1 pickups put back" at 3.07 s (the shelf has the placed one alone; the level's own
  stay taken) and it's taken again at 3.20 s. With nothing taken: "0 of 46 pickups put back" and
  nothing moves.
* Not tested in a capture: the keys themselves (the hooks drive the same code, but the key
  and mouse handling is only read, not run; Home included), PageUp / PageDown, Shift+Delete.

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
* Fights (`BF_TEST_FULL_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=6,30,6,30 BF_TEST_FREECAM=0.3,6,7,36,0,-25`, each
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
* Switching (`BF_TEST_FULL_SQUAD=1 BF_TEST_LEVEL="3.5,sdm_e34;3,flat;3,sdm_e10;3,m01_a;3,flat"`, with an enemy
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
  * Tex running at it from 32 m (`BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=4.5,10,4.5,-60
    BF_TEST_SCENERY_PLACE="0.3,sdm_e34:4,4.5,-30,0" BF_MOVE_LOG=1`; the squad since #302) stops at its wall (z -22.5,
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
  * Bulgar's 39 m rock ridge (sdm_e34:1; the squad's members with `BF_TEST_FULL_SQUAD=1` since
    #302): Brutus and Hawk climbed it ("hawk walks off a ledge ...
    4.8 m to the floor"); Tex, running into its steep end, went into it: a slope steeper than
    about 51 degrees (under the 65-degree wall limit, so not a wall) rises more than the 0.5 m
    step in one 0.4 m capture step. That's the demo's floor rule on any level. Since #285
    (`BF_TEST_GOTO=4.5,10,4.5,-60 BF_TEST_SCENERY_PLACE="0.3,sdm_e34:1,4.5,-30,0"
    BF_TEST_SPRINT=1.0 BF_INSIDE_LOG=1`) he runs up its steep end in the ground step's 0.12 m
    steps and along its flank; where the flank steepens past 65 degrees (at 6.1 s, and once more
    before 8.5 s) he's on it as on terrain too steep to stand on and slides down it to its foot,
    and sprints on along it to the goal at z -60, never in it: nothing logged
    ([maps.md](../maps/rocks-and-boulders.md), Rocks and boulders).
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

The squad and the music (#302), `--test` with `BF_MUTE=1 BF_BACKGROUND=1 BF_TOOLS_LOG=1`,
captures at 15 fps, frames looked at:
* The start (`BF_TEST_GOTO=0,8,0,8 BF_SOUND_LOG=1`): "test map: the squad at the start: 3 tex (tex
  leads)", the audit's "squad 0 (0 NPCs)"; the radar has Tex's face at the top alone, the other
  three places dark (no face, tab or channel fill; the channels' outlines stay). "test map: its music, m04_a's bank,
  loading", then about 1.5 s in "test tools: music of m04_a: music test_casp1_amb01 x1.00,
  ambience ambience19-2ch x0.80 (BF_MUTE: not played)".
* Filling (`BF_TEST_SQUAD_MENU=0.5 BF_TEST_SQUAD_SLOT="1.0,0,shrike soldier@m09_a;1.5,2,shrike
  heavy@sdm_shrike;2.0,1,hawk"`): each one made at its formation place round Tex ("squad slot 1
  (Brutus's place): Shrike Soldier (m09_a, 73 hp) (h_fcec1024) at [1.50, -1.05, 11.25], was
  empty"), with its type's model, clips and gun; the radar a "?" at Brutus's place (m09_a's
  Shrike Soldier has no portrait), sdm_shrike's Shrike Heavy's portrait at Hawk's, Hawk's face at
  Flint's, the four health channels; the panel lists the slots and the types. Seen from above
  (`BF_TEST_SQUAD="shrike heavy@sdm_shrike,shrike soldier@m09_a"` beside NPCs of the same two
  types, `BF_TEST_FREECAM`): the members solid, in formation behind the NPCs, the same models.
* Control (`BF_TEST_SQUAD="shrike soldier@m09_a" BF_TEST_SELECT=0,1.0 BF_TEST_GOTO=0,8,0,-20
  BF_TEST_GO_AT=3.0 BF_MOVE_LOG=1`): "SHRIKE SOLDIER" with the chevrons toward Brutus's place, the
  cut at 1.5 s; from 3 s it runs to the goal on its MOTION table's run (clip 38, about 5 m/s),
  behind it its type's camera (1.55 m at 10 degrees, so close the body fades), Tex following.
* The player's type (`BF_TEST_SQUAD=flint BF_TEST_SQUAD_SLOT="2.0,me,shrike heavy@sdm_shrike"
  BF_TEST_GOTO=0,8,0,-20`): Tex, running, becomes the deathmatch Shrike Heavy where he was ([-0.01,
  -1.05, 3.83]), on his heading, and it runs on to the goal on its own run (clip 37); the camera
  eases to its 3.2 m boom, the bars to its 55 HP, the weapon panel to its gun (80 / 600), its
  portrait at Tex's place; "You're Shrike Heavy (mp_common/sdm_shrike, 55 hp) now", and at the
  rack "Shrike Heavy cannot pick up Rail RVR."; Flint follows.
* Death and hand-over (`BF_TEST_SQUAD="shrike soldier@m09_a,flint" BF_TEST_SELECT=0,1.0
  BF_TEST_DIE=3.0 BF_ENEMY_LOG=1 BF_PICKUP_LOG=1`): the controlled Shrike dies with its type's death
  line ("Shrike Soldier's death line fb465ea7 of 3"), drops a memory chip, the death camera circles
  it with the skull at its place, "TEX" and the chevrons up, and control passes to Tex, who takes
  the chip.
* Emptying and replacing (`BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,8,0,7.5
  BF_TEST_SQUAD_SLOT="1.0,me,empty;1.5,1,empty;2.0,2,shrike heavy@sdm_shrike;2.5,0,flint"
  BF_TEST_KILL=2 BF_COMBAT_LOG=1`): "Your own slot can't be emptied: give it a type"; Hawk killed
  (her skull), Flint's slot emptied (her face, tab and channel fill gone, the outline stays), the dead Hawk replaced by a
  Shrike Heavy where she fell (its portrait and channel fill back; it takes her chip), which Tex's fire
  kills (its death line f80210fa, its chip, the human skull), and Brutus's place given Flint (her
  face at the bottom).
* The editor's edit mode: editor.md, "The squad menu in edit mode".
* A level (`BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.6,15.1 BF_TEST_SQUAD="3:shrike heavy@sdm_shrike"
  BF_TEST_SQUAD_SLOT="1.0,0,shrike soldier@m09_a" BF_TEST_LEVEL="2.5,flat"`): sdm_e34 keeps its four
  ("squad 3"; `BF_TEST_SQUAD` is the test map's), Brutus replaced by the Shrike Soldier ("?" at his
  place); the switch back starts the test map with the Shrike Heavy leading alone in Tex's place
  ("squad 0"), and m04_a's music loading.
* The panels (`BF_TEST_GOTO=0,8,0,7.5 BF_TEST_HELP=0.5 BF_TEST_SQUAD="shrike soldier@m09_a"
  BF_TEST_SQUAD_SLOT="1.0,me,feral outcast" BF_TEST_SQUAD_MENU=1.5`): the controls panel's first line
  "Feral Outcast idle ...", its 1-4 line and the tools' T line; "You're Feral Outcast (mp_common/dm,
  75 hp) now", the dm feral's portrait at Tex's place.
* Unchanged, against main's build (a6b3af4, the same hooks): sdm_e34's `BF_TEST_GOTO=0,0,0,-20
  BF_TEST_FIRE=1 BF_TEST_HIT=2` (75 frames) frame for frame. The test map with
  `BF_TEST_FULL_SQUAD=1`: the NPC run above (135 frames) with the same combat log (51 lines), the
  frames the same but for a tracer's streak at frames 59-64 (effects' seeds follow entity numbers:
  the HUD's four "?" are new entities); the death and hand-over (`BF_TEST_DIE=1`, 100 frames) with
  the same chip lines, the frames but for the chevrons' phase (it runs on real time) and the blood
  mist's particles. Recipes with a squad hook and no flag: pickups.md's dead squad member's guns
  (`BF_TEST_KILL=1 BF_TEST_SWAP=4.0,7.0`), grenades.md's hand-over during a throw
  (`BF_TEST_SELECT=1,1.6`), skills.md's crosshair on the stealthed Hawk (`BF_TEST_HOSTILE=hawk
  BF_TEST_SQUAD_SKILL=hawk,2.0`; since #333 the hook goes through Toggle Ability's goal, which
  runs for the held Hawk too: its 75 frames the one-shot request's to the pixel, its log with the
  goal's lines added) and this page's hand-over (`BF_TEST_SELECT=0,2.2`): their logs
  main's line for line but for the loading times; the crosshair run's 75 frames the same, the
  others but for a few dozen pixels where the members, running through the pickup grid at the
  start, kick a loose item a little differently (before #356 removed the kick) (the order of two systems Bevy leaves open, which
  the new systems shift: two runs of either build agree there).
* `cargo test --bin bf_play`: the slot changes (`play_testsquad.rs` tests: the controlled slot
  changed in place and not emptied, a member replaced or removed, an empty slot filled before the
  NPCs at its formation place, a roster applied).
* Not tested: the keys and the mouse themselves (T, Left / Right, Up / Down, the wheel, Page Up /
  Down, Enter, the click, Delete): the hooks drive the same code from the menu's asks on. V's "has
  no special skill" (the key's only). The music heard: under `BF_MUTE` only its log.

After the review (the product owner's two decisions and the review's points), the same way:
* A level in a test session (`--test BF_MAP=tutorial BF_TEST_GOTO=46,-20,46,-20`, 30 frames): the
  tutorial's Tex alone with the four faces and their tabs, as on main; plain `--test`
  (`BF_TEST_GOTO=0,8,0,7.5`): Tex's face alone.
* A lone death (`BF_TEST_GOTO=0,8,0,7.5 BF_TEST_SQUAD=none BF_TEST_DIE=1 BF_TEST_HELP=3.0
  BF_TEST_SQUAD_SLOT="7.0,0,hunter lord@mp_common/sdm_shrike" BF_COMBAT_LOG=1 BF_PICKUP_LOG=1`, 150
  frames): Tex dies at 1.07 s (his skull at his place, the chip dropped), H's hook at 3.07 s on time
  over the body, "t 5.13: tex died alone: respawned after 4 s at [-0.00, -1.05, 8.00]": standing
  where he fell, "Respawned", his face back, full health; he takes his chip at once ("Memory Chip
  Recovered!"); his old guns lie at his feet. The member hook at 7.07 s, on time ("squad slot 1
  (Brutus's place): Hunter Lord (mp_common/sdm_shrike, 80 hp)": the menu's label form accepted),
  its portrait at the bottom.
* A death recipe (play.md's death box, `BF_TEST_DIE=2.0 BF_TEST_GRENADE_TYPE=Energy`, 140 frames):
  "the squad at the start: 0 brutus, 1 flint, 2 hawk, 3 tex (tex leads; BF_TEST_DIE: the four)",
  the death camera, "BRUTUS" and the cut to him with his box at full, no respawn.
* The arena in edit mode (`--edit BF_MAP=mp1 BF_TEST_SQUAD_SLOT="1.0,0,flint;1.5,me,shrike
  soldier@m09_a"`): "Flint refused: No squad here: the map is played alone", the leader's type
  taken ("(the roster, for play)").
* The combat log (`BF_TEST_SQUAD="shrike soldier@m09_a"` against an enemy Brutus set to fight):
  "Shrike Soldier -> brutus (NPC 1): roll 6.67 ...", the squad member named by its type as shooter
  and as victim.
* The controls panel keeps main's first line ("tex idle clip 48 ..."); its 1-4 line names the four
  and T.
* A lone death of a type with a stamina shield (`BF_TEST_GOTO=0,8,0,7.5 BF_TEST_SQUAD="3:shrike
  heavy@sdm_shrike" BF_TEST_SUICIDE=1.0`, 110 frames): before, X's hurt of its hit points went to
  the shield and it lived; with `enemies::lethal` it falls at 1.07 s, and "t 5.07: Shrike Heavy died
  alone: respawned after 4 s at [0.00, -1.05, 8.00]", "Respawned" over it standing where it fell.

The predefined squads (#339), `--test` with `BF_MUTE=1 BF_BACKGROUND=1 BF_TOOLS_LOG=1`, captures at
15 fps, frames looked at (the six rows come from the data: "squad menu: 6 predefined squads", each
with its types and keys in the order of the places):
* Each squad on the test map (`BF_TEST_GOTO=0,8,0,7.5 BF_TEST_SQUAD_MENU="0.3;2.0"
  BF_TEST_SQUAD_PRESET="1.0,<squad>"`, 60 frames, the squad `campaign`, `brute force`, `feral hunting
  pack`, `shrike raiders`, `seer coven`, `ferguson platoon`): at 1.0 s "squad Shrike Raiders
  (mp_common/sdm_shrike) in all four slots", then the four slots in the keys' order, the three
  empty ones filled at the formation places round Tex ("slot 1 (Brutus's place): Shrike Soldier
  (mp_common/sdm_shrike, 50 hp) (h_e08868ca) at [-1.50, -1.05, 4.75], was empty", Flint's at [1.75,
  -1.05, 4.00], Hawk's at [-0.80, -1.05, 2.25]) and Tex's own slot changed where he stood ("slot 4
  (Tex's place): Hunter Lord (mp_common/sdm_shrike, 80 hp) (h_0959b0fc) at [-0.00, -1.05, 8.00], was
  Tex"), "squad Shrike Raiders: 4 of 4 slots put", and "Squad: Shrike Raiders (you're Hunter Lord
  now)" on the screen. In the world the three members stand ahead of the player (he faces -z, the
  formation's way), each with its model and gun, and the player is the new type with its weapon
  panel (the Hunter Lord's minigun 80 / 600, the Feral Colonist 30 / 120, the Shadoon's 1 / 0,
  the Militia 30 / 200) and its health bar. The radar's four portraits are in the places by the
  types' slots, the four channels full: campaign and Brute Force Tex top, Hawk left, Flint right,
  Brutus bottom; Feral Hunting Pack the Feral Colonist top, Outcast Shaman left, Feral Outcast
  right, Feral Shaman bottom; Shrike Raiders the Hunter Lord, Shrike Hound, Shrike Heavy, Shrike
  Soldier; Seer Coven the Shadoon, Fire Hound, Seer Priest, Seer Follower; Ferguson Platoon the
  Militia, Gunthar Ghent, McTavish, Confed Marine.
* Health follows (`BF_TEST_HURT="0,40,2.0;1,20,2.0;2,60,2.0;3,30,2.0"` after the Ferguson
  Platoon, 75 frames, 80 hp each): at 4.7 s the radar's channels are at a half (Brutus's place,
  hit 40), three quarters (Flint's, 20), a quarter (Hawk's, 60) and five eighths (Tex's, 30): the
  channels' pixels counted before and after, 328 to 207 (63%) at Tex's, 344 to 268 (78%) at
  Flint's, 328 to 176 (54%) at Brutus's, 317 to 84 (27%) at Hawk's. The player's own bar is at 124
  of its panel's 199 px (the Militia's panel is shorter than Tex's 286 px one, which the new type
  changes: its health fills it, 62%); the members knocked down by the hits.
* They are a squad: `BF_TEST_GOTO=0,8,0,-30` after Seer Coven (100 frames): the Shadoon runs on,
  the Follower, Priest and Fire Hound follow it through the pickup grid and take what they pass
  ("2x Medkit" over the Shadoon). With `BF_CHARACTER=0` Brutus leads: "slot 1 (Brutus's place): Shrike
  Soldier ... at [-0.01, -1.05, 8.01], was Brutus", the other three at their formation places.
* Over a squad already there (`BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_PRESET=
  "1.0,shrike raiders;3.0,brute force" BF_TEST_SQUAD_SLOT="2.0,1,empty;2.2,3,flint"`, 75 frames): the
  four replaced where they stood ("was Brutus", "was Flint" ...), then the single-slot path as
  before (Flint's slot emptied, the player's given Flint), then the second squad over those ("was
  Shrike Soldier (mp_common/sdm_shrike, 50 hp)", the emptied slot filled at its formation place,
  "was empty").
* Other maps: a squad deathmatch map (`BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.6,15.1`, Feral
  Hunting Pack): the map's own four replaced where they stood, the player the Feral Colonist. An
  arena (`BF_MAP=mp1`, Shrike Raiders): the player the Hunter Lord, the three others refused one by
  one in the log, "1 of 4 slots put", and on the screen "Squad: Shrike Raiders (you're Hunter Lord
  now; 3 not put: played alone)" (it read "3 not put: No squad here: the map is played alone"
  until the fixes below shortened it).
* The editor's edit mode (`--edit BF_TEST_SQUAD_MENU=0.5 BF_TEST_SQUAD_PRESET="1.0,seer coven"
  BF_TEST_PLAY=2.5,start BF_TEST_EDIT=5.0`, 100 frames): "Squad: Seer Coven (when play starts)", the
  four slots "was empty (the roster, for play)" (Tex's "was Tex"), Shift+F5 "play from the map's
  start: h_fcc10a43 leads at [0.00, -1.05, 0.00], 3 more in the squad, 0 NPCs" with the Shadoon, its
  camera and the Seers' four portraits, and F5 "Shadoon and the squad (3) out of the world". The
  help lists T as "any character type in each squad slot, or empty; or a predefined squad". An
  arena in edit mode (`--edit BF_MAP=mp1 BF_TEST_SQUAD_PRESET="1.0,ferguson platoon"
  BF_TEST_PLAY=2.0,start`): the leader's slot to the Militia, the three others refused, "0 more in
  the squad" in play.
* The panel (`BF_TEST_SQUAD_MENU` and `BF_TEST_SQUAD_PICK="0.5,feral hunting pack,1;1.0,seer
  coven,2;1.5,ferguson,3;2.0,flint,0;2.5,shrike soldier@m09_a,3"`): the six squads under
  "[predefined squads]" and the types under "[character types]", the row picked marked, the list
  window moving with it (the squads, with the types after them; the squads' last rows and the first
  types; the types alone), for a squad who goes in each place ("(you)" at the player's; the first
  version took one line a place and covered the item box, see the fixes below), for a type the
  slot's text as before ("click / Enter: put it in slot 1   Delete: empty slot 1", "you become it
  (slot 4)").
* The single slot unchanged (the "Filling" recipe above with `BF_TEST_GOTO=0,8,0,8`, and "Emptying
  and replacing"): the same log lines as before the change ("squad slot 1 (Brutus's place): Shrike
  Soldier (m09_a, 73 hp) (h_fcec1024) at [1.50, -1.05, 11.25], was empty", "Your own slot can't be
  emptied", "was Hawk").
* `cargo test --bin bf_play` (147): the seating by HUD slot (Brute Force and Seer Coven as the data
  has them, a slot taken or out of range), the hook's squad names, the list's rows, and a squad put
  slot by slot (the leader in place, the three at their places before the NPC, refused in deathmatch).
* #365's two squads (`--test` with `BF_MUTE=1 BF_BACKGROUND=1 BF_TOOLS_LOG=1`, 15 fps captures, the
  frames looked at): "squad menu: 8 predefined squads", the last two "Kingman's squad (e05, m03_c): 0
  Red Hand h_12073efa, 1 Terrorist h_11fe9c5e, 2 Subversive h_0de6c6dd, 3 Edward Kingman h_ffdc629e"
  and "Monsters (e23): 0 Filthy Mutant h_0b5f6e21, 1 Molotov Mutant h_f0c5e50e, 2 Ugly Mutant
  h_ef4ed3dc, 3 Supermutant h_ece04235". `BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_MENU="0.2;5.0"
  BF_TEST_SQUAD_PICK="0.4,Kingman's squad" BF_TEST_SQUAD_PRESET="1.0,Kingman's squad;3.0,Monsters"`
  (90 frames): both rows are in the panel (Kingman's squad picked, "Brutus: Red Hand   Flint:
  Terrorist / Hawk: Subversive   Tex: Edward Kingman (you)"); at 1.0 s "squad Kingman's squad (e05,
  m03_c) in all four slots", the three empty ones at the formation places and Tex's own "was Tex",
  "Squad: Kingman's squad (you're Edward Kingman now)" over his face; at 3.07 s Monsters over it, every
  slot "was Red Hand ..." and so on, "(you're Supermutant now)". The hook took both full names
  (`kingman` and `monsters` too, with `BF_TEST_SQUAD_PRESET="1.0,kingman"` and `"1.0,monsters"` in
  the runs below). All four radar places show the "?" in both squads (no portrait among the eight).
  **They follow**: `BF_CHARACTER=0 BF_TEST_GOTO=0,8,0,-60 BF_TEST_SQUAD_PRESET="0.4,monsters"`: the
  Filthy Mutant runs on by his own clips with the Molotov, Ugly and Supermutant behind and beside him
  on the follow camera. **They fight**: `BF_TEST_INSTANT_KILL=0 BF_COMBAT_LOG=1 BF_TEST_GOTO=0,8,0,8
  BF_TEST_SQUAD_PRESET="1.0,kingman" BF_TEST_NPC="2.0,shrike heavy@e34,enemy,fight,0,-6"
  BF_TEST_NPC_HP="3.0,all,400"`: the Red Hand, Terrorist and Subversive each fire on it ("Red Hand ->
  Shrike Heavy (NPC 1): ... = 11.15") and it fires on Kingman and the Subversive; with Monsters and a
  Shrike Soldier 9 m ahead (`0,3`, 240 frames) the mutants close in and "Filthy Mutant -> Shrike
  Soldier (NPC 1) ... = 15.65" from 14.9 s, 12.9 s after the spawn (a slow engagement; the Shrike's
  shots on the Supermutant and Ugly Mutant come first). With the Shrike Heavy 14 m ahead the
  mutants did not reach it in 10 s, and one stray "Ugly Mutant -> Supermutant" at 6.2 s (own side
  0.7) landed on its neighbour. The squad AI's handling of melee types isn't part of this ticket.
* Not tested: the keys and the mouse themselves (Up / Down over the squad rows, the wheel, Enter or
  a click on one): the hooks drive the same code from the menu's asks on, and `BF_TEST_SQUAD_PICK`
  sets what the keys set (the row's mapping is a unit test).

**#394's deathmatch classes** (the Kingman Posse and the Mutant Horde, `character_class.rs`), `--test` with
`BF_MUTE=1 BF_BACKGROUND=1 BF_NO_FADE=1`, 15 fps captures with the frames looked at, `BF_TOOLS_LOG=1`; the real
squads against main's build (56079a8, the same code):
* **The data** (`BF_DUMP_TYPES=all BF_DUMP_TYPES_FAST=1 BF_MAP=sdm_e34`): the totals line is main's ("128
  character types (0 models not loaded, 0 without a motion set, 0 without clips; 30 MOTION clips missing),
  22 motion sets, 542 placements"), then "8 demo classes (character_class.rs) and the 20 squad deathmatch
  types, with the columns of the rule each doesn't satisfy": all eight classes "rule gaps: none"; of the 20
  sdm types 15 "none" and five with a species trait: the Fire Hound (vulnerabilities), the Shrike Hound
  (damage-multiplier 1.8, vulnerabilities) and the Hunter Lord, Shrike Heavy and Shrike Soldier
  (vulnerabilities, the stamina shield, max-stamina 40 / 45 / 28). So the rule's skulls by slot, slots 1 - 4,
  kill reward 0, radar-range of 60 or more and hit points in 45 - 95 hold on all 20.
* **The log** (`BF_TESTMAP_LOG=1`): "test map: 8 of 8 deathmatch classes (#394)"; the NPC menu has 136
  entries, "Edward Kingman (demo/kingman_posse, 95 hp)" beside "Edward Kingman (m03_c, 311 hp)" and
  "Supermutant (demo/mutant_horde, 95 hp)" beside "Supermutant (e23, 669 hp)"; "squad menu: 8 predefined
  squads", the last "Kingman Posse (demo/kingman_posse): 0 Red Hand h_ebefd26c, 1 Subversive h_1f330430, 2
  Terrorist h_03589fbb, 3 Edward Kingman h_1b39dea1" and "Mutant Horde (demo/mutant_horde): 0 Ugly Mutant
  h_0ca8ee04, 1 Molotov Mutant h_02f7069a, 2 Filthy Mutant h_fa73c507, 3 Supermutant h_16122b66", each class
  under them with its values (hp, slot, face, skull, skill-set / cost / recovery, speeds, items,
  damage-multiplier, player flag, vulnerabilities, shield, radar-range, kill reward) and a line a value saying
  real, rule or guess. A preset's slot lines read "squad slot 4 (Tex's place): Edward Kingman
  (demo/kingman_posse, 95 hp) (h_1b39dea1) at [-0.00, -1.05, 8.00], was Tex".
* **Portraits and places** (`BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_PRESET="0.5,kingman_posse"`, 75 frames, the
  radar ring cropped): Kingman's cut face at the top (Tex's place), the Subversive's at the right (Flint's),
  the Terrorist's at the left (Hawk's) and the "?" at the bottom (Brutus's: the Red Hand); the Mutant Horde:
  the Supermutant's face at the top, the one mutant face at the right, left and bottom (Molotov, Filthy, Ugly).
  `BF_TEST_HURT="0,999,2.0;1,999,2.0;2,999,2.0"` (three members killed at 2.0 s, the capture's frame 55): the
  bottom place (the Red Hand, the "?") shows the wide Brutus skull, the left and the right the narrow Hawk
  and Flint skulls, Kingman's face stays at the top; `BF_TEST_HURT="3,999,2.0"` Kingman's own: Tex's skull
  at the top. **Hit points** by exact damage (`BF_TEST_HURT="0,54,2.0;0,1,3.0;1,69,2.0;1,1,3.0;2,49,2.0;2,1,3.0;3,94,2.0;3,1,3.0"`):
  all four faces alive after the first hits (frames 30 - 44, the speech icons of the hurt lines up) and all four
  skulls after the second (frame 47 on), which is exactly the Kingman Posse's 55 / 70 / 50 /
  95 for slots 0 - 3 and the Mutant Horde's (`0,74` ... `3,94`) 75 / 55 / 70 / 95. The same on sdm_e34
  (`--test BF_MAP=sdm_e34 BF_TEST_GOTO=-81.78,63.86,-81.78,63.86`, the map's own members replaced where they
  stood). The panel (`BF_TEST_SQUAD_MENU` and `BF_TEST_SQUAD_PICK`): "Kingman Posse (demo/kingman_posse)" and
  "Mutant Horde (demo/mutant_horde)" in the list, "Brutus: Ugly Mutant   Flint: Molotov Mutant / Hawk: Filthy
  Mutant   Tex: Supermutant (you)", ending at y 520 (the item box starts at 565); the window opens with the
  first type at its foot and the last six of the eight squads above it. The red flash of the faces
  for a second after a preset is every preset's, not the classes' (main's Ferguson Platoon shows it too).
* **The five real squads and the campaign squad unchanged**
  (`BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_PRESET="0.5,<squad>"`, 45 frames each, main's exe and this one): the
  mean absolute difference over every frame is 0.000, in the whole frame and in the radar ring, for all six
  (two different squads differ by 9 on the same frame, so the comparison sees a change), and their
  "squad ..." log lines are equal.
* **Hooks**: `BF_TEST_SQUAD_PRESET="0.4,monsters;2.0,Kingman's squad;2.6,posse;3.0,nonesuch"` puts the
  Mutant Horde, the Kingman Posse twice, and the last "no predefined squad nonesuch (they are: Campaign squad,
  ... Kingman Posse, Mutant Horde)". `BF_TEST_SKILL=1.5` as Edward Kingman: nothing toggles (the skill block is
  data, the briefcase isn't built) and nothing breaks; as Tex the same recipe gives the same berserk lines as
  main (`BF_SKILL_LOG=1`: the toggle at 1.07 s, the second gun at 1.60 s).
* **Damage** (`BF_COMBAT_LOG=1 BF_TEST_INSTANT_KILL=0 BF_TEST_WEAPON="MNR-7 Bio" BF_TEST_FIRE=1
  BF_TEST_NPC="0.5,<type>,enemy,dummy,0,-2"`, the MNR-7 Bio's damage-type 3): the campaign Filthy Mutant
  (`filthy mutant@e23`) takes "x type 3 0.00 = 0.00" on every hit; the class (`mp_mutant_filthy`, taken by
  its key) "x type 3 1.00 = 14.66", "16.29", "15.08" ..., "hit Filthy Mutant (NPC 1) for 14.7 -> 55.3 hp"
  from its 70.
* `cargo build --bins` with no warnings and `cargo test` (the library's 87, nine of them new in `character_class`:
  the rule on a campaign fixture, the eight classes as the product owner's table, the names' hashes against the
  data's, a sdm type fitting the rule, the report's lines, `add_classes` adding the loaded ones once after the
  data and skipping a base without its model or a name in the way; and `bf_play`'s 299, two of them rewritten or
  new here: the class squads' names, aliases and words, and their rosters seated by the rule with the owner's hit points).

After the product owner's two fixes (the menu opened on the Campaign squad, so Enter straight after T
changed four slots; with a squad picked the panel ended at y 630 and covered the Frag item box, x
955-1045, y 565-657), `--test` with `BF_MUTE=1 BF_BACKGROUND=1`, 45-frame captures, main merged in
(70dfaae: #338's load order, #334's nav core):
* **T opens on the old row** (`BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_MENU=0.3`): "> Brutus", the first
  character type, is the row picked, as on main ("(1 of 128 character types)", "click / Enter: put
  it in slot 1   Delete: empty slot 1"), with the six squads above it in the window (the list's top,
  the pick at its foot); the panel ends at y 426. Enter there puts Brutus in slot 1 as before;
  a squad needs Up first (a unit test: `Menu::default` picks `Row::Type(0)`, whatever the number of
  squads; the window's rule `window_top`).
* **The panel above the item box in every state**: its bottom edge measured in the captures (the
  rows the panel darkens, against the same frame with the menu closed; the item box's top is 565):
  the worst state, a filled squad of long names with a squad picked (`BF_TEST_SQUAD_PRESET="0.8,<squad>"
  BF_TEST_SQUAD_PICK="1.2,<squad>,3"`), ends at y 520 for the Feral Hunting Pack, Seer Coven and
  Shrike Raiders (24 lines, 45 px above the box), 504 for the Ferguson Platoon, 458 for Brute Force;
  a long type picked after a squad was put 489; the window at the list's end (the last type,
  h_115268fc) 489; the editor's edit mode with the Feral Hunting Pack in the roster 520 (no HUD there);
  an arena (played alone) 458. The unit test `the_panel_ends_above_the_item_box_in_every_state`
  counts the panel's wrapped lines over 1,728 states of the six squads' real names (every squad
  held and picked, every slot yours, play and edit mode, dead or not, three type rows): at most 24 of
  the 26 lines that fit.
* The rest rerun on the merged tree with the same logs as before: the six squads on the test map
  (the radar's four portraits by place, the channels full), the editor (`--edit`) and an arena
  refusing the three members ("3 not put: played alone" now), sdm_e34, and the single slot's
  "Filling" and "Emptying and replacing" recipes. `cargo test --bin bf_play`: 150 passed.

**Followers** (#364), `--test` with `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps, frames looked at
(a side view of the run: `BF_CAMERA_PITCH=-0.5 BF_CAMERA_DISTANCE=11 BF_VIEW_YAW=1.5708`), `BF_FOLLOW_LOG=2`
for the rows ("followers | flint (NPC 1) state 0 17.7 m off (-12.0, 29.1)"). Against main's build
(`e1cbe41`, built from `git archive`), the same recipes:
* **The run and the stop** (`BF_TEST_GOTO=-20,30,20,30 BF_TEST_GO_AT=2 BF_TEST_SPRINT=2
  BF_TEST_NPC="0.3,flint,friend,fight+follow,-23,28;0.3,hawk,friend,dummy+follow,-23,32"`, 270 frames):
  Tex stands and both hold (3.6 m off) until he's 11.3 m off at 2.53 s (d0 11.2), then chase at the
  dash and trail his 40 m dash at 17.5-17.8 m (Flint) and 16.1-16.3 m (Hawk), at his speed (8.6 m/s
  while beyond d1's 14 m, the boost), and stop at 9.8 m (Hawk, 7.20 s) and 9.8 m (Flint, 7.40 s) once he
  has stopped ("follow state 0 -> 4 (within d2 of a leader who isn't running; its leader 9.8 m off)",
  "4 -> 2 (no world points to look for cover on)" on the flat floor). At rest 9.3-9.5 m off him, 1.6
  m from each other, kneeling (the frames). The squad on the same floor and run (`BF_TEST_FULL_SQUAD=1`)
  stops at 9.0-9.2 m, and take04's three 9.1, 8.5 and 8.9 m ([squad.md](../squad/the-follow.md)).
* **On a level, with its cover points** (take04's staging on e34, `BF_MAP=e34 BF_SCRIPT=off
  BF_TEST_NO_HOSTILES=1 "BF_TEST_WARP=0,flint,-91.08,-83.87;0,brutus,-87.21,-84.16;0,hawk,-89.53,-87.82"
  BF_TEST_GOTO=-83.39,-82.79,-49.25,-71.14,27 BF_TEST_GO_AT=2 BF_TEST_SPRINT=2` with
  `BF_TEST_NPC="0.5,flint,friend,fight+follow,-90.5,-86.5;0.5,tex,friend,dummy+follow,-88,-89.5"`, 270
  frames): the squad's three and the two followers all set off as Tex gets 11.3-11.8 m off; Brutus
  claims h_e1182318 (take04's Brutus's) and the follower Flint h_f7d1227d, 1.3 m off it; the others
  (the squad's Flint and Hawk, the follower Tex) find none free ("no cover point near its leader") and
  hold 9.4-9.7 m from Tex at the stop. No point is claimed twice (the unit test
  `two_followers_never_take_one_cover_point` runs it too: two followers, two points, a third finding none).
* **A fighter among them** (`BF_TEST_NPC="0.3,brutus,friend,fight+follow,-23,28;0.3,hawk,friend,dummy+follow,-23,32;0.3,flint,enemy,fight,0,48,,270"
  BF_TEST_NPC_HP=0.8,all,999 BF_COMBAT_LOG=1`, the same run, the enemy Flint 18 m off the run's line):
  the follower Brutus fires at him as it holds and as it sets off ("brutus (NPC 1) -> flint (NPC 3)"
  at 1.6, 2.0, 2.4, 3.33 and 3.73 s; the shots of the hold land late), none while it dashes (a dash
  can't aim, play.rs `can_aim`: 3.7-8.2 s), and again from 8.2 s, stopped, to the end; Flint's shots
  go to Tex and the followers; the dummy follower Hawk fires none.
* **Toggled**: `BF_TEST_NPC_FOLLOW="3,1,off;5,all;8,2,on"` with two followers 40 m out, chasing: NPC 1
  off at 3.07 ("NPC 1 (Hawk): friend fight", "no longer follows") stands in the field, about 22 m
  from Tex; the toggle of all at 5.07 puts her on again (she chases and stops 9.6 m off) and NPC 2 off
  (already at 9.7 m: stands); NPC 2 on at 8.07 holds, within d0. With the NPC under the crosshair, the
  free camera aimed at her down the run (`BF_TEST_FREECAM=0.5,-20,-0.3,-4,180,-5`),
  **the key itself**, `BF_TEST_KEYS="1.0,quote;3.5,quote"` ("test tools: key Quote pressed"): off at 1.0
  s ("NPC 1 (Hawk): friend dummy" on the screen, the frames), on at 3.5 s ("friend dummy, follow"),
  chasing to 9.4 m. **The menu by its keys**: `BF_TEST_KEYS="0.5,n;0.8,j;1.1,semicolon;1.4,quote;1.7,enter;
  2.0,down;2.3,enter;2.6,quote;2.9,down;3.2,enter;3.5,n"` (Tex dashing from 4 s): "NPC 1: friend Brutus
  (fight, follow)", "NPC 2: friend Flint (fight, follow)" (the setting kept for the next one), the key
  again, "NPC 3: friend Hawk (fight)"; the first two trail Tex and stop 9.3 and 9.4 m off, Hawk stays
  where she was put. The panel at 3 s (cropped frame): "J team: FRIEND   ;: DUMMY   ': FOLLOW YOU" and
  "0 spawned   J / ; / ' / Del on the one under the crosshair, Shift+Del all".
* **A hand-over** (`BF_TEST_SQUAD=flint BF_TEST_GOTO=-20,30,-20,30 BF_TEST_ORDER=flint,guard@0.5
  "BF_TEST_WARP=1,flint,10,30,90" BF_TEST_SELECT=1,3`, followers Hawk (fight) and Brutus (dummy) 8 m
  from Tex): they hold (8.2 m) while Tex leads; Flint takes control at 3.1 s, 33.2 and 29.3 m from them,
  and both chase him ("follow state 3 -> 0 (its leader beyond d0)") and stop 9.5 and 9.7 m from him; Tex,
  now an AI member, does the same.
* **Not a friend, no leader**: `BF_TEST_NPC_TEAM="2,1;3.5,1"` (J twice): at 2.0 s "NPC 1 (Hawk): enemy
  dummy, follow (idle: only a friend follows)", she stands where she was 60 m out (the rows say
  "(idle)"), at 3.53 s a friend again and chasing; in the edit mode (`BF_TEST_EDIT=5.5`) she stands
  ("no longer follows") and the placement stays "dummy, follow", and `BF_TEST_PLAY=8,start` brings the
  leader back and she chases ("tex leads at [-20.00, -1.05, 30.00]").
* **Types of their own**: a Confed Marine (fight+follow) and a Shrike Soldier (dummy+follow)
  (`confed marine@mp_common/dm`, `shrike soldier@m09_a`): the same states, at their clips' root
  speeds (trailing 21.8 and 17.5 m), stopping at 9.6 and 9.7 m, the Shrike kneeling.
* **Unchanged** (main's build against this one, the same recipe, the logs compared line by line and
  the frames pixel by pixel; two runs of main's alone differ in 114 of the 270 frames, 42 levels at
  most, the effects' noise): take04's e34 follow with the squad alone (`BF_ORDER_LOG=2` rows and
  follow lines, 146 lines, identical; frames within 11 levels); the tutorial's marines
  (`BF_MAP=tutorial BF_TEST_GOTO=-15.6,-87.4,-60,-75 BF_TEST_GO_AT=14`, 420 frames: 163 lines
  identical; 32 frames differ by up to 14 levels); the squad's follow on the test map's floor (131
  lines identical); and the NPC fight recipe above (`BF_TEST_NPC="1,flint,enemy,dummy,...;3,brutus,
  enemy,fight,..."`, 85 combat lines identical; 114 frames differ by up to 42, as main's alone do).
* `cargo test --bin bf_play`: 240 passed, among them the follow's (the leader and the `mates`, two
  followers and the cover points, the let-go, the removal, the pace record), the hooks' (the
  behaviour word, the messages, the keys by name, the key held one frame, the follow key's acts).
* Not done: a real keyboard (`BF_TEST_KEYS` presses the `ButtonInput` the keys are read from, so every
  Update system sees what a key gives it (not the `PreUpdate` ones: the squad menu, the objectives, the panels), but no key was pressed by hand: the window is unfocused under
  BF_BACKGROUND), and Delete on a follower that holds a cover point on a level (the unit test
  `a_follower_taken_off_the_map_frees_its_cover_point` only).
