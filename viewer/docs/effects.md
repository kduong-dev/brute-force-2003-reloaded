# Spawned effect types (`src/bin/play_effects.rs`, #373)

An effect type (an objecttypes `<effect>`, `EffectType` in `src/bf/character.rs`) is more than
particles. default.xbe's effect object, each game frame it runs (its tick, 0x21b7a0), updates its
ALE effects and then works through the lists its type carries: the **damage areas** (FUN_0021ade0),
then the **rumbles** and the **shakes** (FUN_0021b220). This module is that work for any effect type
the demo starts. Before #373 only the barrels' and the rack's effects dealt damage, and the code was
play_scenery.rs's (`AreaDamage`, [scenery.md](scenery.md)); now the scenery, a grenade's blast and
the test hook all ask this module to start an effect type, and tickets that follow (the Molotov,
#78 and #211; the Supermutant's pound, #212) only have to ask too.

## What an effect type carries

| Record | Where in the data | Parser | What it does |
|---|---|---|---|
| ALE effects, light effect | `<alchemy-effects>`, `<light-effect>` | FUN_00188a90 (an entry) | the look; `ale_fx.rs` |
| sounds | `<sounds><Sound sound-id h_f724cb8c play-length>` | | each played after its h_f724cb8c seconds |
| **damage areas** | `<h_fb0a5f1d><Damage amount Type h_ed582b3c h_f724cb8c duration range falloff>` | FUN_00191170 (0x1c bytes a record) | an area that hurts for a while |
| **shakes** | `<shake-effects><shake h_f724cb8c duration range falloff h_f944a10f h_f6d585b1>` | FUN_00191470 (0x18 bytes) | a shake held on the camera of whoever is in range |
| rumbles | `<rumble-effects><rumble ..>` | FUN_001912f0 (0x20 bytes) | the pad's motors (FUN_0021b410, FUN_001f2950): **not played**, the demo has no pad vibration |

The names are the data's: `amount` h_e01a523c, `Type` h_f632635b (the DTYPE_ enum: 5 FLAME, 11
POWERBLADE, 10 EXPLOSION, 3 BIOREACTIVE ...), `duration` h_e7bcd633, `range` h_169716c2, `falloff`
h_f879ec8a (the DFALL_ enum: 0 NONE, 1 LINEAR, 2 EXPONENTIAL, 3 HALF_LIFE); `h_ed582b3c` (the
**push**) and `h_f724cb8c` (the **start**: the effect's age the record starts after) aren't
recovered. The shake's two amplitudes are the roles FUN_0021b620 gives its record's +0x10
(`from`, h_f944a10f) and +0x14 (`to`, h_f6d585b1). 44 effect types in the levels' and common's
objecttypes have a Damage block (46 records) and 9 of them also a shake; the two this ticket is
for:

| Effect type | Look | Damage | Shake |
|---|---|---|---|
| **h_1a256542** (the Molotov's cloud: the explosion h_193e39d9's `h_ec23d593`) | ALE h_e49f4396, light h_ef1056ea, sound h_02bcfca8 at 0.2 s | 78 FLAME (5) a second for 1 s within 7 m, falloff 1 (linear), push 1 | none |
| **h_ece33110** (the Supermutant's pound) | ALE h_18058148, light h_ef1056ea, sound h_1ea097fa | 50 POWERBLADE (11) a second for 1 s within 7.5 m, falloff 0, push 2.5 | from 1 s on, 2 s long, within 10 m, falloff 2, 0.7 falling to 0 |

## Starting one

`EffectSpawns` (a resource) takes `EffectSpawn` requests: the effect type, `at` (where its
effects are and its range is measured from), `origin` (where a kill's shove goes from: the
effect object's place; the barrel's origin where `at` is its centre), `rotation`, `source` (the
breakable object it came from, which its own areas leave alone) and `look`. `EffectSpawn::new(kind,
at)` is the simple one; `.without_look()` starts only the damage areas and the shakes, for a
caller that plays the effect, its sounds and its push on the loose bodies itself. A system that asks
for one runs before `start_effects` in the same frame (the effects systems sit in the scenery's
chain, between `break_scenery` and `set_off_mines`) or the next.

`start_effects` (once a frame) plays the ALE effects and light effect once at `at` in `rotation`
(`load_recorded`, run once, as the grenades' and the barrels' always were), queues the sounds (each
after its own delay; one with none plays on the frame after, as the barrels' did), throws the
loose pickups and debris about for each damage area with a push (`pickups::Blasts`), and sets the
areas and shakes going on the effect's clock.

Who asks now:
* **the breaking scenery** (`break_scenery`: an effect in a broken object's debris list, the
  barrels', the rack's, the crates', with `source` the object);
* **a grenade's blast** (`fly_grenades`): when the explosion's effect type has damage areas or
  shakes of its own (`look` false: the blast plays its own). No squad kit has one; the Molotov's
  does, so `BF_TEST_INV=Molotov:5 BF_TEST_GRENADE_TYPE=Molotov BF_TEST_THROW=2.0,0.5` throws one
  whose cloud then burns (the kit and the AI that throws it are #78's);
* **`BF_TEST_EFFECT`**, below.

Not asked yet, each a line (an `EffectSpawn::new(kind, at).without_look()` where the look is
already made): the blasts of bullet-type 2 weapons (play_missile.rs: h_17dca7d1 on m02, m04, m05_d,
m07_a and m07_b has the effect h_eac77cc5, 30 PSYCHIC a second for 1 s within 5 m; h_09bc0f70 on
m07_c has h_0ed5ab8e, 50 FLAME a second for 3 s within 5 m), the effects a level's triggers make
(play_triggers.rs: the scripted explosions h_1ce6167d, h_01a7a984, h_fb81cde3 ... carry damage) and
the effect objects a level places (play_placed_fx.rs).

## Damage areas (FUN_0021ade0)

Each frame, for each record while h_f724cb8c < the effect's age <= h_f724cb8c + `duration`, every
target within `range` of the effect's place (3D, to the target's place: no ray, so no line of sight
and no cover) whose factor for the record's Type is above 0 and whose hit points are above 0 takes
amount x dt:
* falloff 1 (LINEAR): x (1 - d / range);
* falloff 2 (EXPONENTIAL): (1 - d / range)^2 x (amount x dt)^2 (squared value and all);
* 0 (NONE) and 3 (HALF_LIFE): in full.

So a record with falloff 0, 1 or 3 deals `amount` x `duration` to someone at the centre however
the frames fall, 78 FLAME for the cloud, 50 POWERBLADE for the pound (falloff 2 squares the
frame's amount, so what it deals depends on the frame's length). What the demo does with it:
* the target's place is a point `CENTRE_UP` (1.45 m) above a character's feet (the game's is the
  character's position record, the hips: a fit, [scenery.md](scenery.md)), the centre of a
  breakable object's archetype; the effect's is `at`;
* a character takes it through its own damage handler (0x125600, `damage::Hit`): its **difficulty
  step** (STANDARD: the squad x0.6, enemies x1.4; none on the multiplayer maps), its skill's and its
  modifiers' factors, and its **factor for the damage type** (`CharacterType::vulnerability`, the
  type's `h_142be76f`: the Filthy Mutant FLAME x0.75 and BIOREACTIVE x0, the Supermutant FLAME x0.5
  and POWERBLADE x0.059, the squad 1 for both);
* the first tick on a character is a hurt as a blast's (blood, its hurt line, the red tint when it's
  the player), the later ones its hurt line and the hit points; one that kills is a kill (its death
  line, the ragdoll);
* **a kill's shove** (#366, [ragdoll.md](ragdoll.md)): the damage record names no body and no
  attacker, its way runs from the effect's place (`origin`) to the victim's root body, and its push
  is the area's h_ed582b3c scaled by the world's load (`ragdoll::area_push`: x0.9 here). So the
  CBT_DEAD handler's hit request is body-less, the up-kick merges into it and the body is thrown
  at the cap, whatever the damage type (the Type-3 kill is vaporised instead). The cloud's kill
  asks for factor 0.9 x 1, the pound's 0.9 x 2.5;
* the breakable objects in range take it too (`Scenery::area_at_work`): the rack's explosion sets
  off the barrels around it; a Sentry that's down goes off in an area at work if its own factor for
  the Type isn't 0 (the demo's choice).

**When**: the clock of an effect starts `AREA_DELAY` (0.1 s) after it is started, and a record's
window starts h_f724cb8c after that. 0.1 s is the barrels' and the rack's: their takes' first HP loss
comes about 0.1 s after the object's hit points reach 0 (takes 01-04 and 07; also a grenade's
blast to its damage). It is **assumed for every other spawn**: no take shows a Molotov's cloud or a
pound.

## Shakes (FUN_0021b220, FUN_0021b620)

For each record while `start` < the effect's age <= `start` + `duration`, the camera of every
character within `range` of the effect's place gets FUN_0006bdd0 with the amplitude lerp(`from`,
`to`, (age - start) / duration) x (1 - d / range) for falloff 1, or (1 - d / range)^2 x
amplitude^2 for 2, with no hold and no fall: the camera's one-frame amplitude (+0xe4), set again
every frame the effect runs. The camera then turns the view by a random pitch and yaw within
+-0.025 x the amplitude rad and half that of roll each game frame ([play.md](play.md), "The camera
shake"). Only the controlled character's follow camera exists here, and not while it's dead:
`shake_view` holds the strongest amplitude on `CameraShake` (`hold_each_frame`, play_shake.rs: held
for every game frame a render frame holds, not kicked once a frame, which a 15 fps capture's
second game frame would spend). The pound's: 0.49 at the pound and 0.162 at 4.25 m when its
window starts at age 1 s, down to nothing at age 3 s.

The explosive barrel's own shake (h_02b37a16: 0.25 s, 1 m, falloff 2, 0.2 to 0) can't reach
anyone standing on the ground in the demo (the point measured to, 1.45 m above the feet, is 1.0 m
above the barrel's centre already).
The ones that can: h_02c667c7 (e34, m09: 1 s, 10 m, no falloff, 1 to 0), h_076ac299 (e23: 0.5 s
on, 1 s, 10 m, 0.5 to 0.25), h_13347406 (e01: 2 s, 100 m, falloff 1, 1 to 0), h_e51fcf57 (m02, m04:
1 s on, 1.48 s, 20 m, falloff 1, 0.7 to 0.4).

## The effect object's life

The game's effect object ends when every one of its eight ALE slots is empty (0x21bc34; a slot is
emptied when its ALE instance answers true to its vtable +0x2c with 4, which FUN_0021a850 asks at
the end of each update: read as "finished"), so a damage area or shake that outlasts the effect's
particles is cut short there. The demo runs each record its whole length, as the
barrels' always did, and says when it matters: with `BF_DAMAGE_LOG` an effect that starts with
a record longer than its ALE effects logs it. The cloud's h_e49f4396 runs 5.77 s and the pound's
h_18058148 10.93 s (`Compiled::duration`, an estimate), both well past their records.

## Test hooks

* `BF_TEST_EFFECT=<s>,<effect type>,<x>,<z>[,<y>][;...]`: at sim time `s` starts the effect type at
  (x, z), on the floor there (`floor_y` at or below 2 m above the controlled character's feet; the
  flat test floor's -1.05) or at world height `y`, with everything a spawned effect has:
  look, sounds, damage areas, shakes. The effect type is `h_xxxxxxxx` or a name (hashed). An entry
  that isn't four or five fields is skipped. The test map has the effect types of every level
  (m02's h_e51fcf57, m01_a's h_1ce6167d and e01's h_02b37a16 started alike); one it doesn't know
  prints "effect h_xxxxxxxx: no such effect type".
* `BF_DAMAGE_LOG=1`: each effect started (its damage areas as (amount, type, duration, range,
  falloff), its shakes, how long its ALE effects run) and each damage tick: `t 2.13: area
  h_1a256542 at 3.33 m: Filthy Mutant takes 2.04 (x1.00 the step) -> 496.9 / 500 [age 0.100
  (+0.067), falloff 1 share 0.524, type 5 x0.750, push 1]` (the time, the effect, the distance
  from its place to the target's point, who, what it takes after every factor and the step,
  the hit points left, the record's age and the part of this frame it covers, the falloff and its
  share, the target's factor for the Type, the push, "killed" for the tick that kills), and
  each area's end with its victims' totals. `BF_COMBAT_LOG=1` prints the same ticks without the
  bracket (the form the scenery's takes were checked with), `BF_SCENERY_LOG=1` the ends.
* `BF_SHAKE_LOG=1`: also the shake the effects hold on the camera, four times a second ("shake held
  0.156 by h_ece33110 (age 1.03 s, 4.25 m)").

## Verified

Test map (`cargo run -q --bin bf_play -- --test`), `BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures
(`BF_CAPTURE`), the numbers in the logs; `BF_TEST_INSTANT_KILL=0` where an NPC mustn't die to
the test map's instant kill. Filthy Mutants are e23's h_0b5f6e21 (55 HP; the hook gave them 500).

* **The Molotov's cloud**, `BF_DIFFICULTY=0` (no step) `BF_TEST_GOTO=0,-20,0,-20
  BF_TEST_NPC="0.3,h_0b5f6e21,enemy,frozen,x,-12"` for x = 1, 3, 5, 6.5, 8 `BF_TEST_NPC_HP=1.0,all,500
  BF_TEST_EFFECT=2.0,h_1a256542,0,-12 BF_DAMAGE_LOG=1`, the cloud at (0, -12) on the floor:

  | x | d to the point 1.45 m up | share 1 - d / 7 | per tick (1/15 s) | total over 1 s | 78 x share x 0.75 |
  |---|---|---|---|---|---|
  | 1 | 1.76 m | 0.748 | 2.92 | 43.8 | 43.76 |
  | 3 | 3.33 m | 0.524 | 2.04 | 30.7 | 30.66 |
  | 5 | 5.21 m | 0.256 | 1.00 | 15.0 | 14.98 |
  | 6.5 | 6.66 m | 0.049 | 0.19 | 2.8 | 2.87 |
  | 8 | 8.13 m | out of range | none | none | |

  The ticks run from t 2.07 to 3.07: 14 whole ones (78 x 1/15 x share x 0.75) and a half at each
  end (1.46 for the first mutant), 1 s in all, from 0.1 s after the effect started. At STANDARD
  (`BF_TEST_GOTO=0,-14,0,-14`, Tex 2.47 m off): Tex takes 2.02 a tick = 78 x 1/15 x 0.647 x 0.6
  (his step); a Filthy Mutant 4.09 at 1.76 m = 78 x 1/15 x 0.748 x 0.75 x 1.4, a Supermutant
  1.43 at 4.25 m (FLAME x0.5); Tex at 6.86 m, 0.06 a tick (share 0.021).
* **The pound** (`BF_TEST_EFFECT=2.0,h_ece33110,0,-12`, Tex at (0, -16), `BF_DIFFICULTY=0`): Tex
  (4.25 m) and the Filthy Mutants at 1.76, 4.25 and 6.66 m each take 3.33 a tick, 50.0 over the
  second, **the same at every distance** (falloff 0); the one at 8.13 m none; the Supermutant 0.20 a
  tick, 3.0 in all (POWERBLADE x0.059: 50 x 0.059 = 2.95).
* **A kill's shove**: a Filthy Mutant with 5 HP (4.7 m from the pound) dies on the second tick, "death
  shoves asked: magnitude 6.7 type 11 factor 2.2 at [-4.00, -0.12, -14.00] along [-0.88, 0.20, -0.44]
  on None, then the up-kick": 2 x the tick's 3.33, the push 0.9 x 2.5, the way from the pound's
  place to the hips; merged with the up-kick it is magnitude 7.7, factor 38.2, "67.5 N s on every
  body", and the body leaves at 10.1 m/s (3.3 up) and lies 9 m off by 5 s. The cloud's kills (STANDARD,
  `BF_TEST_NPC_HP=1.1,1,8` and `BF_TEST_HEALTH=8`): the Mutant at 1.76 m dies on the third tick
  (magnitude 8.2 = 2 x 4.09, type 5, factor 0.9), Tex 2.47 m off on the fifth (magnitude 4.0, type
  5, factor 0.9).
* **The shake** (`BF_SHAKE_LOG=1`): the pound's from age 1 s: 0.156, 0.117, 0.083, 0.055, 0.033,
  0.016, 0.005, 0.000 at 0.25 s steps at 4.25 m, (1 - 0.425)^2 x (0.7 x (1 - (age - 1) / 2))^2.
  The view's shifts, measured by phase correlation of the pillars and horizon (rows 100-300
  of the 1280 x 720 frames): none before the effect, +-1-3 px frame to frame from 3.0 s to 4.3 s
  (a turn of up to 0.0039 rad at 0.156), none after. h_02c667c7 (1.0 for 1 s, no falloff) at 4.25 m
  shakes the view by up to 18 px from the first frame, easing to 0 by 2.8 s.
* **A thrown Molotov** (`BF_TEST_INV=Molotov:5 BF_TEST_GRENADE_TYPE=Molotov BF_TEST_THROW=2.0,0.5`,
  mutants 2 to 5.5 m from where it lands): the bottle's own blast (the grenades' rule, Damage max
  falling to nothing at 7 m, x0.75: 38.6, 26.5 and 13.2 at 2.4, 3.8 and 5.4 m) and the cloud's
  ticks from the same blast (2.49, 1.71 and 0.84 a tick at 2.54, 3.93 and 5.49 m).
* **The barrels unchanged**: every `area h_`, `breakable`, `hurt from`, `spawns` and `over` line of
  the logs (`BF_COMBAT_LOG=1 BF_SCENERY_LOG=1`) is the build before #373's, line for line: sdm_e34's
  radiation barrel at take29's spot (`BF_MAP=sdm_e34 BF_NO_SQUAD=1 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
  BF_TEST_HIT=2.0`: 238 lines, Tex's 40.0 over +0.03 to +0.50 s), campaign e34's rack from 7.1 m with
  its chain reaction (`BF_TEST_GOTO=-31.86,14.77,-31.86,14.77 BF_TEST_HIT=2.0
  BF_TEST_HIT_NEAR=-31.86,21.87`: 529 lines, 60.0 HP) and its barrel at -31.52, 16.64 (501 lines),
  and an e01 explosive barrel (`BF_MAP=e01 ... BF_TEST_GOTO=31.62,40.9,31.62,40.9
  BF_TEST_HIT_NEAR=31.62,43.58`: 640 lines). The captured frames differ between two runs of the
  *same* build in the bursts' particles (they aren't pixel-deterministic), so the frames can't be
  compared; the barrel bursts look as they did.

Unit tests (`cargo test`): the falloffs, the clock's wait, that a second of damage sums to its
amount at 15, 30 and 60 fps, the shake's window, ramp and falloff, the hook's entries, and the
effect type's Damage and shake records read from the data's attributes (`bf/character.rs`).

## Not done

* The rumble lists (FUN_0021b410): no pad vibration.
* The effect object's end with its last ALE effect (above), and an effect's own `max-life`.
* The other spawn paths listed under "Starting one": missiles, triggers, placed effect objects.
* The game's shake reaches every character's camera in range; here the controlled character's.
* What the demo's NPCs do about a cloud (they stand in it: nothing flees fire), and any knock-down
  from a heavy tick: the pound's trigger and its effect on the squad are #212's.
