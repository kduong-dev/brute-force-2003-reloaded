# Character types (`src/bf/character_type.rs`, #128)

The game's own definition of every character, enemies included, keyed the way the game keys
it: the groundwork for the enemies (#110). The species tickets (#129-#134) build their
behaviour on it. Sources: the decoded data (`decompiled/xml/`), default.xbe (the functions in
[code-map.md](code-map.md)), and the analyst's leads in `todo/128-enemy-types/leads.md` and
`todo/110-enemy-species/leads.md`.

## What the game loads

| File (per archive) | What it gives | Read by (code) | Here |
|---|---|---|---|
| `objecttypes-<lib>.xmb` `<character><h_e275fb80 ..>` | a **character type**: its `<base name>` (the key), model, stats, AI, inventory, speeds, capsule, decals, face | the type factory FUN_0015fde0 (archetype-type 4), the loader 0x12ed40 (FUN_00190330 attributes, FUN_00190580 elements) | `CharacterType`, `Game::character_types` |
| `characters/ms_<x>.xmb` | the **motion set** a type's `motion-archetype` names: its animation set, the MOTION table, the overlays, the hit bones | FUN_00131a10 (FUN_00130c50 the MOTIONs' clips, FUN_001314c0 the overlays', FUN_00130aa0 the bones) | `MotionSet`, `Game::motion_sets` |
| `characters/<x>_chatter.xmb` | the type's voice lines (`chatter-library`) | FUN_001304b0 | `Game::chatter` (by file stem) |
| `levels-<lvl>.xmb` `<character-object object-type-name=TYPE name=..>` | a **placed character**: its type with this instance's team, AI and inventory | the character's property handler 0x112eb0 | `Placement`, `Game::placements` |

* **The key** is the hash of the type's `<base name>`: `brutus`, `flint`, `hawk`, `tex` for the
  campaign squad, `mp_brutus` ... for the deathmatch one, a hash for everyone else. The game
  looks a name up library by library and takes the first (FUN_00160da0); here the archive
  loaded first wins. The data has 198 definitions of 128 names; 33 names are defined in 2 to 8
  archives, and every copy is the same definition (checked against `decompiled/xml`: the leads'
  count of 7 is off).
* **Motion sets are keyed by their file**, not their own name: `ms_lf.xmb` and `ms_dm_lf.xmb`
  both call themselves `lf`, and their MOTION tables differ. A type names its set by the file
  (`motion-archetype="ms_lf"`); `Game::motion_sets` is keyed by that string's hash. There are 22
  files, each the same wherever it appears.
* **The animation set** is the motion set's `animation-set-name` (h_fd06d1b2, the type's +0xcc).
  Seven sets play another's clips: ms_feral_colonist plays `brutus`, ms_mutant2 `mutant`,
  ms_mctavish `militia`, the four `ms_dm_*` their campaign set. 15 animation sets for the 22
  motion sets.
* **The face clips** are the set the type's `<face viseme-file>` names (`brutusface` on the
  ferals, `militiaface`, `mutantface`, `seerface` ...; 39 of the 128 types have none: the
  Shrikes, the hounds, some humans such as e01's Sgt. Gonzales).
* `Game::load_level` now opens each level's `ms_*.xmb` and `*_chatter.xmb` too (they live in
  its `characters/` folder; the squad's are in common only, so nothing of the squad's is
  replaced). A chatter file already loaded from another archive is kept, as the motion sets are.

## A type's fields

`CharacterType` holds what the loader 0x12ed40 copies into the runtime type (`type+X` is the
runtime offset; see the code map's rows for each).

| Field | Data | Game | In play |
|---|---|---|---|
| `key`, `archive`, `label` | `<base name>`, the archive, `<base h_1dd3a49d>` (string id: "Shrike Soldier") | FUN_00160da0 | the NPC menu, the dump, logs |
| `mesh` | `<base mesh-name>` | type+0xc4 | the model (`Character::load_type`) |
| `motion_archetype`, `motion_set` | `motion-archetype` | type+0x1a4, FUN_00131a10 | the clips (its set's animation set) |
| `face_set` | `<face viseme-file>` | (the face block) | the face clips |
| `chatter` | `chatter-library` (h_f18a456f) | FUN_001304b0 | loaded; not yet said by enemies |
| `hitpoints`, `target_type` | `<combat-target hitpoints target-type>` (the TTYPE_ enum, 0x3bf294) | FUN_00189f80; type+0x1c, +0x20 | HP |
| `vulnerabilities` | `<vulnerabilities><vulnerability Type modifier>` | type+0x78, FUN_00154740 | every hit's factor for its damage type (play_damage.rs `vulnerability`) |
| `shield` | `<shield shield-type ..>` (1 energy, 2 stamina) | FUN_00189c40, FUN_00189a90, FUN_00154780; 0x223400 / 0x223690 | carried: the deathmatch Shrikes' and Hunter Lord's stamina shields, six bosses' energy shields |
| `damage_multiplier` | `damage-multiplier` (h_e7e9fcd3) | type+0x220, FUN_00125940 | a direct hit's factor when the squad slot has none (play_damage.rs `damage_multiplier`; 0.62 the campaign squad and its allies, 1.8 the deathmatch hounds) |
| `max_stamina`, `height_offset`, `turn_rate`, `player_type` | the attributes | type+0x224, +0x2ac, +0x2d0, +0x1a8 bit 0 | stamina for OTHER; the rest carried |
| `ragdoll_death` | `enable-ragdoll-death` (h_f397278e) | type+0x34c bit 0, FUN_00115d50 at 0x115e1f | carried: false on the 9 Seer Priest types and the 1189-HP Shadoon (they have death clips, #133 / #134); everyone ragdolls in the demo |
| `ai` | `<ai alignment-type personality-* vision-range half-fov squad-leash-dist>` | type+0x358, FUN_0001ff70 | the leash for OTHER; the team rule below |
| `weapons`, `items`, `weapon_classes` | `<inventory weapon-name><item>`, `<limit-weapon>` | type+0x2d4.. | OTHER's weapons and stances |
| `speeds` | `<speeds speed>`: crouch walk, walk, run, dash | type+0x6f8.., FUN_0012c6c0 | carried (the demo moves by root motion) |
| `capsule` | `<cylinder-extents radius height crouch-height><offset>` | type+0x1d4.. | carried (the demo uses one cylinder, `BODY_RADIUS` / `BODY_HEIGHT`) |
| `decals` | the two lists h_fb4bfc68 / h_efc2a874, up to 3 each | type+0x1b4 / +0x1c4, FUN_00124af0 | the blood decals (one of the list at random) |
| `death_effect`, `attached_effects` | `<death-effect>`, `<base><h_18b6ab72>` | type+0x350, FUN_00115d50 | carried (the priests', the Supermutant's; the hounds' body effect) |
| `footstep_type`, `jump_sound` | h_072ab4af, `<movement-sounds jump-sound>` | | OTHER's footsteps |

Lists of the data are stored as the attribute repeated, one per entry (the parsers advance a
cursor per entry: FUN_00190250 the speeds, FUN_00195710 a MOTION's scripts), so every entry of
the attribute is read, not the first.

* **Flesh** (`Game::flesh`): what a model is made of, the world material most of its parts'
  collision triangles carry (`(header >> 24) & 0x1f`, through each PART's archetype's surface).
  The data's 128 types: 61 FLESH_HUMAN (21), 10 MUTANT (22), 23 SHRIKE (23, the Supermutant's
  too, as stored), 5 SYNTHETIC (24: Flint and Kingman), 23 SEER (25), 6 FIREHOUND (26).
* **Root-height** (`Game::default_root_height`): the default motion's clip's `root-height`, or
  0.75 when it has none (FUN_00131a10 at 0x131b2f: type+0x1b0). Brutus 0.980, Flint 1.057, Hawk
  0.897, Tex 1.190, militia 1.119, lf 1.014, hf 1.121, hounds 0.758. The demo still stands
  everyone on `GROUND` plus `sole_lift` (squad.md).

## The motion set's clips

A MOTION has a motion type (the MT_ enum, `MOTION_TYPES`), two slots of `script` and two of
`ready-script` (the gun raised; the parser's two strings of each, FUN_00195710: `stunned
stunned2` fills both slots, most MOTIONs only the first, and an empty slot stays empty in its
place) and a prefix-type. `Motion::clip` resolves a slot as FUN_00130c50 does:

* a slot whose script and ready-script are both empty has no clip (0x130f1a-0x130f26);
* the string a slot is named from is its ready-script for a ready slot, its script for a
  script's, **and its script too when its ready-script is empty** (FUN_00130ba0: `test dl,dl`
  at 0x130bb3 and `cmp byte ptr [eax],0` at 0x130bb7 both go to the script, EDI). 90 MOTIONs in
  21 of the 22 motion sets have a slot like that (all but ms_kingman): stand2crouch,
  crouch2stand and throw_grenade in 15 sets, the priest's and the Shadoon's deaths, the hounds'
  gas attacks, the mutants' eat_*, teleport_*, prayer_*, the Shadoon head's idle; with the gun
  up they play their script's clip;
* the name is `"Sc_" + stance + string + suffix` for prefix-type 1 or 2 (MS_PREFIX_WEAPON,
  MS_PREFIX_WEAPON_NUMBER), the string + suffix for 0 (MS_PREFIX_NONE; the table at 0x40cae8);
  the stances are `w1_` and `w2_` (0x40cb30), the suffixes "", `_cr`, `_left`, `_right`,
  `_left_cr`, `_right_cr` (0x40cafc);
* a name the animation set lacks falls back by suffix (0x38db80: `_left_cr` / `_right_cr` to
  `_cr`, the rest to none), then to the name with no stance and no suffix (the third call,
  0x131063, zeroes both: `xor esi,esi` at 0x131058, `xor ecx,ecx` at 0x131061);
* clips are found by the name's hash in the animation set.

The game builds 48 clips per MOTION this way (two slots x script and ready-script x two
stances x six suffixes). The demo's locomotion still picks its clips by fixed names
(`pick_clips`, play.rs); the table is for the species tickets and the dump.

Over the whole cast, 30 of the MOTION clips resolve to nothing, all of them the data's:
ms_mutant2's MT_SURPRISE has prefix-type 1 with a whole clip name as its script
(`Sc_w1_Sc_w1_surprise`: 7 mutant types), and the Shadoon's set has no `Sc_w1_stunned2` and no
`Sc_w1_rp_f_side_lev_r` (4 types), each counted for script and ready-script.

## Placements and teams

`Placement` is a `<character-object>`: its type (object-type-name), name, transform, `<ai>`
(alignment-type, personalities, vision, goal-id), inventory (two weapon slots, ammo and
grenades), drops (inventory-drop), whether a player can take control of it (h_17a554bf) and
its accuracy bonus. **Its team** (`Placement::team`, 0x10fa60): its own alignment-type
(char+0x1d4), or its type's (type+0x3b0) when that's -1 (ALIGN_NONE). The teams are the ALIGN_
enum (`TEAMS`: 0 CONFEDERATION, 1 SHRIKE, 2 NEUTRAL, 3-6 TEAM1-4, 7 PC_NO_TEAM, 8 HOSTILE,
9 SEER_SLAVE). 30 of the 542 placements set a team other than their type's (the squad as TEAM1
in some missions, e34's neutral Shrike Hound, the Seer Spy on the squad's side in m02_b/c ...).
The level loads them dormant and a spawn trigger clones them (#102): nothing spawns them in the
demo yet.

## In play

* `Player::kind` is a character's type (`Loaded::kind` the one its model was built from).
  `Player::character` stays its squad slot, a `CHARACTERS` index, and squad-only rules stay
  keyed to the four (portraits, skills, chips, the hand-over, chatter answers, Brutus not
  kneeling). A character of any other type (a test map NPC) is slot `OTHER`, whose name is
  empty.
* `spawn_unit` loads the model by type (`Character::load_type`: the mesh, the motion set's
  animation set, the face set). For the squad that's the model `Character::load` gives by name;
  `BF_DUMP_TYPES` compares the two (bones, geosets, clips and face clips). Its other data comes
  from the tables keyed by the slot's name where they have it (the squad's, unchanged: the test
  map's rack changes `character_weapons`, a squad deathmatch map the skills and the
  multiplier), else from the type: HP, weapons and their stances, footsteps, the leash, stamina.
* The blood: a character's flesh is read from its parts (`Loaded::flesh`; play_fx.rs picked
  Flint's SYNTHETIC by her name before) and its decals are its type's lists, one of a list at
  random per decal (the squad's have one each, so nothing changes for them; the Shrikes' hit
  list has two). Fleshes other than 21, 23 and 24 still get FLESH_HUMAN's hit effects and
  blood bundle: each species' own is its ticket's. Shrike flesh (23, #129) has its green bundle
  and lays its decals by the game's rule (enemies.md, "Blood").
* **The damage** (play_damage.rs, #127): the victim's hit parts (`hit_bones`: its type's motion
  set's `<collision>` bones, `Game::hit_bones`) and its factor for the damage type
  (`vulnerability`: `CharacterType::vulnerability`) are its type's. Its damage-multiplier is its
  squad slot's name-keyed entry when it has one, else its type's: on a squad deathmatch map
  `load_squad_library` puts sdm_bf's 1 in the squad's entry while their campaign types say 0.62,
  and that override stays. So an enemy NPC takes its own type's numbers (a Shrike Soldier: GAS
  x0.2, SONIC x2, its head x5, multiplier 1), and the squad's are what they were: everything of
  theirs equals their types' but that override (`BF_DUMP_TYPES` on a campaign map: "type's data
  == the name-keyed tables'").
* **An enemy's own behaviour** (OTHER, #129: play_enemies.rs, [enemies.md](enemies.md)): its
  gait's and motions' clips from its type's MOTION table, its speeds, hit reactions by its
  personality (`Game::personalities`, bf/personality.rs), the game's ragdoll bodies and leg
  limits, its death line from its type's `chatter-library`, the deathmatch types' stamina
  shield; a `BF_TEST_NPC` placement gives it the placement's weapons and personality.
* What's still looked up by the slot's name, so reads nothing for OTHER: the rest of the
  chatter (an enemy says only its death line), skills, the scope and the camera.
* Logs name a character by `unit_name` (its slot's name, or for OTHER its type's display name:
  `Player::name`, set at spawn and kept on the Player, so it holds while the update systems have
  its `Loaded` taken out) and, where only its identity is at hand (a shot's or a blast's
  source, a gas cloud's or a damage area's victims), by `who_name`: `Who::Npc` carries the NPC's
  type for it.
* The gas clouds and damage areas keep their victims per character (`Who`), not per squad slot:
  two NPCs that share a slot (OTHER, or an NPC of a squad member's type) each take a damage
  area's first hurt and get their own log line. The squad's are one per member, as before.

## The test map's other types (#180)

The test map's levels (common, e01 for the first mission, mp_common, mp1-mp8, m02_a, m09_a,
sdm_e34) define 55 of the 128 types. The other 73 are each defined only in campaign
archives: the terrorists (chatter-library terrorist_chatter: Red Hand in e05, e13 and the
tutorial; Terrorist in e16, m03_c and m08_a; Subversive in m03_c and m08_a), Edward Kingman
(m03_c), Gunthar Ghent (e16), McTavish (m08_b), the Militia (m03_b), the Supermutants (e23),
the ferals (e06, e10, m01_a/b, m05_*), the Seers and the Shadoon (m02_b/c, m04_a, m05_c/d,
m07_*), the Hunter Lord (m09_c, m14_d), the Fire Hounds, e34's, e40's and m14's Shrikes, the
Shadoon's head (m02_c, no weapon), and the campaign's own copies of the marines, mutants and
Brutus. `Game::load_type_assets` (`src/bf/type_assets.rs`) adds such types from a level
archive without its level:

* only the types not loaded yet (the first loaded wins, FUN_00160da0, as for whole archives);
  `TYPE_ARCHIVES` (play_testmap.rs) lists, for each of the 73, the first archive by name that
  defines it, so each comes from the archive `BF_DUMP_TYPES=all` names: 31 archives;
* the weapon definitions they carry that aren't loaded (and what those fire), the archive's
  effect and decal types (looked up by name: first wins);
* from the objects file only the archetypes and meshes their models and those weapons' reach
  (each kept element's every name reference followed: PARTs, LOD meshes, hardpoints) with the
  collision surfaces those name (the flesh), not the level's geometry; the materials those
  meshes name; textures are found on demand as for anyone (`Game::texture_rgba`);
* their motion sets and chatter files, and the animation and face sets the motion sets name,
  with only the data channels those clips use;
* the archive's sound bank (first wins), and their chatter's lines: those are streamed files of
  the bank (resource-type 1) in the level's language wave bank,
  `data/ml-sounds/<lang>/<level>-<lang>.tgz`, of which only those lines' bytes are kept
  (`SoundBank::subset` / `merge`), so a terrorist says its death line;
* not the placements, items, object types, level file or anything else: the rack, the pickups
  and the object menu don't change.

The archives are unpacked and parsed on threads, then added in list order. The 128 types'
`BF_DUMP_TYPES` blocks are the same as with the whole archives (`BF_DUMP_TYPES=all`). On the
test map it's `testmap::load_types`, by `BF_TEST_TYPES` (testtools.md): every type by default,
the terrorists alone, some archives' or none.

## Test hooks

* `BF_DUMP_TYPES=1` (bf_play, with `BF_MAP` or `--test`): every character type of the loaded
  archives, then their placements, the totals and the squad's checks, and exits. Per type: its
  key, archive, name, model (bones, LOD0 vertices, face clips: loaded to check it loads), motion
  set and animation set (clips), face set, flesh, default root-height, HP, target-type,
  vulnerabilities, shield, damage-multiplier, team, stamina, turn-rate, the ragdoll switch,
  speeds, capsule, height-offset, weapons, decals, death and attached effects, chatter (and
  whether it's loaded), footsteps; then a line per motion type with each MOTION's clips for the
  w1_ stance (script / ready-script, duration, the idles' root-height; `!name` where the set
  lacks it). Per placement: its archive, name, type (`UNRESOLVED` if no loaded type has it),
  resolved team (its own and its type's), position, weapons and drops.
  `BF_DUMP_TYPES=all` loads every archive of `data/` first (about 12 s), for the whole cast.
  `BF_DUMP_TYPES_FAST=1` leaves the models unloaded.
* `BF_TEST_NPC=<s>,<character>,...` and the N menu take any loaded type (testtools.md): a squad
  name or 0-3, a display name (`shrike soldier`: the first loaded), `name@archive`
  (`shrike soldier@m09_a`) or `name@library` (mp_common's libraries: `shrike
  soldier@sdm_shrike`), a `<base name>` (`mp_brutus`) or `h_xxxxxxxx`. The menu lists them by
  name, where they're defined (`CharacterType::place`: the archive, `mp_common/dm` for
  mp_common's) and HP, with the key added to any two that still read the same. On `--test`
  that's every type (128; `BF_TEST_TYPES`, above), each added one by the archive it came from
  (`red hand@e05`; `terrorist@m03_c` is m03_c's first Terrorist, h_11fe9c5e: its two read the
  same but for their key, which picks the other).
* `BF_TEST_TYPES=all|terrorists|none|<archive>,...` (testtools.md): the test map's other types.

## Verified

* #180 (testtools.md, "Every character type"): on `--test`, `BF_DUMP_TYPES=1` gives "128
  character types (0 models not loaded, 0 without a motion set, 0 without clips; 30 MOTION
  clips missing), 22 motion sets", and each type's block is the same as `BF_DUMP_TYPES=all`'s;
  with `BF_TEST_TYPES=none` the dumps on `--test` and with `all` are the same as before the
  change. Terrorists, bosses, ferals, Seers, the Hunter Lord and a Fire Hound spawned with
  `fight`: each with its own model and gun, firing, hit, dying with its death line.

* `BF_DUMP_TYPES=all` (BF_MAP=sdm_e34): "128 character types (0 models not loaded, 0 without a
  motion set, 0 without clips; 30 MOTION clips missing), 22 motion sets, 542 placements (0
  unresolved, 30 with their own team)", the four squad members "by type == by name". Against
  the leads, field by field: the speeds (Brutus 3.5 4.55 7.8 8.7, lf 0 2.16 4.70 7.11, hound 0
  2 5.5 11.42), the stun and knockdown pairs (lf 1.20 / 1.33 and 2.30 / 2.00 s), the capsules
  (lf 0.35 x 1.9, hf 0.45 x 2.0, the hound's 0.5 x 1.4 with its offset 0 0 -0.2), the idles'
  root-heights, the shields (six energy, six stamina with their max-stamina 25-45), the 0.62
  and 1.8 multipliers, the death effects, the Shadoon head's height-offset -2.3, the hounds'
  attached effect, the flesh counts above; every face set and chatter library the types name
  is loaded.
* `BF_DUMP_TYPES=1` on e01: the squad's type data equals the name-keyed tables'; on sdm_e34
  every field but the damage-multiplier (the sdm_bf override above).
* In play (`--test BF_TEST_GOTO=6,30,6,30 BF_TEST_FREECAM=0.3,6,2.4,25,0,-5
  BF_TEST_NPC="0.5,flint,enemy,dummy,0,14;0.5,shrike soldier@m09_a,enemy,dummy,2,14;0.5,shrike
  heavy@m09_a,enemy,dummy,4,14;0.5,shrike hound@m09_a,enemy,dummy,6,14;0.5,lesser
  priest,enemy,dummy,8,14;0.5,filthy mutant,enemy,dummy,10,14;0.5,feral
  outcast,enemy,dummy,12,14"`): each stands in its own idle with its own model, skin and gun
  beside an NPC Flint (types h_fcec1024, h_f678d520, h_ffdca587, h_e95d12ff, h_0c5aff87,
  h_e3a313c1; flesh 23, 23, 23, 25, 22, 21 logged at spawn). A closer look
  (`BF_TEST_FREECAM=0.3,3,1.6,19.5,0,-4`, m09_a's Soldier and Heavy and the deathmatch ones,
  `shrike soldier` and `shrike heavy`): on their digitigrade legs, with their guns. Next to
  #35 take04's e34 Shrike Heavy (small, at night) the model and its red markings are the
  same; its glow lines aren't drawn (below). The hound is lowered only 0.29 m: `sole_lift`
  stops at 0.3 m and its pelvis is 0.4 m over its paws (#130's ground height).
* The squad unchanged, against main's build (the same hooks, captures at 15 fps compared frame
  by frame, with a second run of main as the run-to-run noise): Brutus killing Flint on the test
  map (her grey sparks, splats and pool h_0d7a5245, her chip; flesh 24 now read from her parts)
  and Tex (red, Hawk taking his chip); the player's death, the death camera and the hand-over on
  sdm_e34; a hand-over by `BF_TEST_SELECT`; a Frag; fire and a scenery hit; the test map's NPC
  fight (testtools.md's run). Every frame and the combat, pickup, grenade and scenery logs are
  the same as main's but for the first frames after a map loads, which differ between runs of
  either build (when its scenery reaches the screen), and the NPC lines, which now name the
  type. Again after #127's merge (main 9a2ac43): sdm_e34's `BF_AUTOPILOT=1` (300 frames), the
  death and hand-over (100 frames) and Tex's and Hawk's skill overlays (`--test
  BF_TEST_SKILL=1.0,4.0`, 90 frames: every frame the same as main's; the overlays are now read
  from the parsed motion sets).
* The damage by type (`BF_COMBAT_LOG`, #127's recipes in play.md § Damage): the squad's numbers
  are main's line for line (take03, takes 04 / 05's LZR-50 and Jax-iC, take27, take28, #115),
  but for an NPC shooter's name (`brutus (NPC 1)` where main printed `NPC 1`). A Shrike Heavy
  dummy in the NPC Tex's place (`shrike heavy@m09_a`, 139 HP) takes its own: take03's missile
  131.07 at its Bip01 L Thigh (x1), then the blast's share 59.79, which kills it; the Jax-iC
  30.73 and 34.79 (the calves' x0.5 and its SONIC x2: the takes' 30.1-35.2); the LZR-50
  2.48-5.22 (LASER x1). ms_hf's hit list is ms_tex's, so for a list of its own: the test map's
  Feral Outcast (ms_feral_colonist, the head x3) under the LZR-50 at head height (takes 04 / 05's
  recipe with `BF_CAMERA_PITCH=-0.035`) takes `Bip01 Head 3`, where an NPC Tex takes `Bip01 Head
  5` (`-0.02`).
* The ready slots (`Motion::clip`): with the script's fallback the dump's totals are unchanged
  (the 30 missing clips are the data's); the slots it fills now show their script's clip on both
  sides, e.g. the Shrike Soldier's stand2crouch and the hounds' gas attacks.

## Not done

* The glow of the h_1df292a1 skins on characters (the Shrikes', the hounds', Brutus's one such
  material): drawn for level objects only (`level_scene.rs`), not by `scene::material`: #173.
* The per-type capsule for shots and collision: #173.
* The per-type ground height (root-height), the ragdoll switch and death effects in play, the
  other species' blood bundles, and every species' reactions and chatter checked against their
  own footage: the species tickets (#130-#134; the Shrikes' are #129's, enemies.md).
* Spawning placements (#102), and placements' teams, AI and inventories in play.
