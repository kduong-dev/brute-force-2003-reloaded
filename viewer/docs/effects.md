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
breakable object it came from, which its own areas leave alone), `look`, and since #200 `owner`
(the character whose skill started it), `user` (its ALE effects' user parameter) and `rules`
(below). `EffectSpawn::new(kind, at)` is the simple one; `.without_look()` starts only the damage
areas and the shakes, for a caller that plays the effect, its sounds and its push on the loose bodies
itself; `.checked()` takes the checked rules; `EffectSpawn::of_skill(kind, at, rotation, owner,
user)` is a skill's spawner's (FUN_0013e600, play_effect_skills.rs). A system that asks for one runs
before `start_effects` in the same frame (the effects systems sit in the scenery's chain, between
`break_scenery` and `set_off_mines`) or the next.

**The rules** (`Rules`, #200; todo/200-effect-skills/answers.md Q1, Q2). The game has one effect
object: its clock starts at its spawn (the tick adds dt to its age before the window test, so a
record starting at 0 deals its first damage at age dt), and FUN_0021ade0 measures each character
to its root node (its vtable +0x40, FUN_0010fda0: the hips) from the effect's place, `d^2 <
range^2`. The delays the takes show are in what spawns an effect. So:
* `Rules::Game`, the effect object as the code has it: the clock from the spawn, the characters
  measured to their root node (`missile::root_point`), the ALE effects by the game's count and shapes
  (`load_recorded_facing`, the triggers' and the hits' rules). A skill's effects (#200), a grenade's
  blast's (FUN_000d9a20 makes the weapon blast, 0x22bfc0 at 0xd9cdc, and the explosion's effect,
  0x22cea0 at 0xd9cf3, in the same pass, and the effect ticks from its spawn: the blast object's
  delay, +0x2c in FUN_00225220, gates the blast's own gather and deal alone, the grenade's
  DAMAGE_DELAY) and the test hooks' (`BF_TEST_EFFECT`, the object menu's) start by them.
* `Rules::Checked`, the fits the breaking scenery was checked with (#85, #373), kept until it's
  checked again against the code's reading (its debris list spawned one entry a frame, the pieces
  first, so its effect comes a frame or two after the hit points reach 0): the clock AREA_DELAY
  (0.1 s) after the start, the characters measured to a point CENTRE_UP (1.45 m) above their feet
  from `at`, the ALE effects by the grenades' rules (`load_recorded`). The analyst's reading for the
  barrels (their effect at the object's origin on the floor, to the hips) fits their takes too
  (2.95 m reached and 3.13 m not, at Tex's 1.19 m hips); moving them over is a follow-up.

`start_effects` (once a frame) plays the ALE effects and light effect once at `at` in `rotation`
(`load_recorded`, run once, as the grenades' and the barrels' always were; each effect from its alchemy
entry's creation-time, `ale_fx::StartAfter`, FUN_00219fc0: 0 for every type the demo's spawns start but the
Air Strike's six shell effects at 2 s, #203; an entry at or past the type's max-life is never made, as the placed
objects leave it out: e01's h_09af3040 of max-life 0, the Vengar hit's `vengar_hit` at 6.9e28 s of 657656),
queues the sounds (each
after its own delay; one with none plays on the frame after, as the barrels' did), throws the
loose pickups and debris about for each damage area with a push (`pickups::Blasts`), and sets the
areas and shakes going on the effect's clock.

Who asks now:
* **an effect skill's spawner** (#200, play_effect_skills.rs: the Orbital Beam's h_ff8e79d6 at the
  target point, owned by its character; [skills/other-characters-skills.md](skills/other-characters-skills.md);
  #203, play_air_strike.rs: the Air Strike's h_007aa97d, [skills/air-strike.md](skills/air-strike.md);
  #208, the Outcast Shaman's Wrath of Garner's h_1e72e858, the same spawner from play_wrath.rs:
  [skills/wrath-of-garner.md](skills/wrath-of-garner.md), whose blast's look is the one effect here
  checked against its takes where the demo's late phase differs; #211, play_molotov_mutant.rs: the Molotov
  Mutant's cloud h_1a256542 at his own root node, when his bottle is let go at the lunge's 0.70 s or he is
  killed holding it, which hurts everyone within 7 m, his squad included:
  [skills/molotov-mutant.md](skills/molotov-mutant.md));
* **the breaking scenery** (`break_scenery`: an effect in a broken object's debris list, the
  barrels', the rack's, the crates', with `source` the object), by the checked rules;
* **a grenade's blast** (`fly_grenades`): when the explosion's effect type has damage areas or
  shakes of its own (`look` false: the blast plays its own), by the game's rules (its clock from
  the explosion, #200). No squad kit has one; the Molotov's
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
frame's amount, so what it deals depends on the frame's length). The test is strict (`d^2 <
range^2`), with no ray, no bounding radius and **no team test**: an effect hurts its owner and his
team as it hurts anyone (#200's takes: the Orbital Beam's owner at a wall 4.9 m off took both its
areas, a teammate took what an enemy takes). What the demo does with it:
* the target's place is a character's root node, the hips (`missile::root_point`), by the game's
  rules, or a point `CENTRE_UP` (1.45 m) above its feet by the checked ones (a fit,
  [scenery.md](scenery.md)); the centre of a breakable object's archetype; the effect's is `at`;
* the record FUN_0021ade0 builds carries the effect's owner (+0xcc) as its attacker (+0x14) and the
  owner's player record (+0x1c): a skill's effect's hits are its character's (`Cause::by`: the kill
  line, the hurt line's attacker), with no FRIENDLY share (the record isn't a weapon's,
  FUN_0022c4d0); the others' have none;
* a character takes it through its own damage handler (0x125600, `damage::Hit`): its **difficulty
  step** (STANDARD: the squad x0.6, enemies x1.4; none on the multiplayer maps), its skill's and its
  modifiers' factors, and its **factor for the damage type** (`CharacterType::vulnerability`, the
  type's `h_142be76f`: the Filthy Mutant FLAME x0.75 and BIOREACTIVE x0, the Supermutant FLAME x0.5
  and POWERBLADE x0.059, the squad 1 for both);
* the first tick on a character is a hurt as a blast's (blood, its hurt line), the later ones its hurt
  line and the hit points; one that kills is a kill (its death line, the ragdoll). The red tint, when
  it's the player: once an area, from the first tick that hits him at **more than** 30 HP a second
  (`AREA_TINT_RATE`, a guess between the takes' none at 30.0 and tint at 80: #203 set it, #208 made the
  test strict on take11, the Wrath of Garner's owner hurt exactly 30.0 a second with no red in his
  picture; so any falloff-0 record whose amount x step is exactly 30 gives none, as the pound does on
  STANDARD, 50 x the squad's 0.6, which no take has measured). It goes by the rate at him, so not the frame's tick, which is a sliver on the frame an area
  starts in and 1/60 s of the rate at 60 fps;
* **a kill's shove** (#366, [ragdoll.md](ragdoll.md)): the damage record names no body and no
  attacker, its way runs from the effect's place (`origin`) to the victim's root body, and its push
  is the area's h_ed582b3c scaled by the world's load (`ragdoll::area_push`: x0.9 here). So the
  CBT_DEAD handler's hit request is body-less, the up-kick merges into it and the body is thrown
  at the cap, whatever the damage type (the Type-3 kill is vaporised instead). The cloud's kill
  asks for factor 0.9 x 1, the pound's 0.9 x 2.5;
* the breakable objects in range take it too (`Scenery::area_at_work`): the rack's explosion sets
  off the barrels around it; a Sentry that's down goes off in an area at work if its own factor for
  the Type isn't 0 (the demo's choice).

**When**: by the game's rules the clock of an effect starts at its spawn, so a record starting at
0 deals its first damage in the spawn's frame at age dt (#200's takes: the Orbital Beam's and the
Wrath's first tick in the sample of the press), and a record's window starts h_f724cb8c after the
spawn: a skill's effect, a grenade's (its cloud ticks from the explosion's frame, the blast's own
damage DAMAGE_DELAY later), a hook's. By the checked rules (the breaking scenery alone) it starts
`AREA_DELAY` (0.1 s) later: the barrels' and the rack's takes' first HP loss comes about 0.1 s
after the object's hit points reach 0 (takes 01-04 and 07), which is their break chain's frames
(answers.md Q1).

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

## Effects on a body (#397)

An effect object can have a parent and then follows it: a level's `<parent Type="1">` link makes
its tick place it at its parent's transform each frame (FUN_0015ed30, run by FUN_0021a700), and for
a character that is its position record, the hips (vtable +0x40: what the damage areas' reach is
measured to). Two effect types of common's objecttypes are put on a body here; **no body in the
game's data or code carries either**, and only one of them is unused in the game. Their own names
aren't recovered; they're called by their ALE effects' (the hash of "vengar" is the ALE effect's,
not the type's), and the cut-content catalogue (3.1) listed both as unreferenced types, which is
right of the types but not of `pshield`:

| Type | Its ALE effect | What the data says |
|---|---|---|
| h_fb1e5895, `vengar` | `vengar` (h_f8db2797): a cube emitter `vengar.emt` (0.13 x 0.43 x 0.13 m, lifespan 120 s, **rate 0, initial count 0, emit count 0**), a field `vengar_rad.fld`, and `vengar_basic.app`, a billow of the `exp5` explosion flipbook | **unused in the game**: nothing spawns the type, and the ALE effect is named by this type alone. max-life 5e11 s (the ALE entry 5e6); one sound, h_effbbcaf (a common sfx, 2 s); no light, no bounding sphere; user parameter 0 |
| h_13687abf, `pshield` | `pshield` (h_fb0d848c): a cone emitter `pshield_Cone.emt` (**lifespan 0.3 s**), a turbulence field `pshield_Turbulence.fld` and `pshield_face.app` (`small-flare.tga`) | **not unused in the game**: the type is unreferenced, but it duplicates, byte for byte but for its name, the record after it in objecttypes-common, h_1218cdc1, the `hit-effect` of every `<shield>` (shield-type 1 in m02_a, m02_b, m04_c, m07_c, m09_c, m14_c and m14_d, type 2 in objecttypes-dm and sdm_shrike: [enemies.md](enemies.md), "The stamina shield"), so the game plays this ALE effect, aligned to the hit's normal, as a **0.3 s burst when a shielded character is hit**. alignment-type 2 on its alchemy entry; max-life FLT_MAX; no sound, no light; bounding sphere of radius 1.023 centred (0.061, 0.059, -0.327); user parameter 1 |

A third ALE effect, `pshield_sub` (a second cone emitter, turbulence field and flare appearance,
sphere radius 1.23), is named by no type. `vengar_heal` and `vengar_hit` are named by other effect
types (`vengar_hit`'s never starts, `EffectType::starts`); [the Spirit of Vengar
page](skills/brutuss-spirit-of-vengar.md) notes that who plays the ALE effects `vengar` and
`vengar_heal` for Brutus wasn't found, and the type h_fb1e5895 that names `vengar` is one nothing
spawns, so it may be what the skill was to play (a guess). The ticket's "permanent aura" is
only that type's max-life: nothing else in its data says it stays on a body.

**What the demo does** (`play_effects.rs`; the demo's reading, since nothing in the game places
them on a body, and for `pshield` a continuous swirl on the hips is the demo's own construct, as
`AleEffect::new` starts the 0.3 s emitter over again and again): `BodyRequests` asks for a type on a
character (the object menu's "effects (unspawned types)" group, the hook's `<who>` form; the shields'
real hit-effect, h_1218cdc1, is the same record and would do the same). `start_body_effects` makes
the type's ALE effects and light effect riding the character
(`AleAssets::load_flight`, the rules a bolt's flight effect has: the particles that are attached
travel with the effect, the others stay where they're born; `AleEffect::new`, so each emitter starts
over as its lifespan ends), at the type's user parameter and bounding sphere (the distance LOD), plays
the type's sounds after their delays, and `follow_bodies` keeps every one at the character's hips
(`missile::root_point`) turned its yaw (the game's is the whole transform), written as the effect's own
and its global transform (an effect with no parent is placed from the global one when the ALE system
emits). A repeated request for the same type on the same character takes it off (`TakenAway`, the
game's destructor's removal of every node at once, FUN_0021a4d0); an effect ends when its character
dies or is gone. The type's damage areas and shakes aren't run for a body effect (neither of the two
has any). pshield's alignment-type 2 is the struck surface's normal, which FUN_0021a850 falls back
from to the effect's own frame when there is none, as there is none here.

**What it shows**: see "Verified".

## Test hooks

* `BF_TEST_EFFECT=<s>,<effect type>,<x>,<z>[,<y>][;...]`: at sim time `s` starts the effect type at
  (x, z), on the floor there (`floor_y` at or below 2 m above the controlled character's feet; the
  flat test floor's -1.05) or at world height `y`, with everything a spawned effect has:
  look, sounds, damage areas, shakes, by the game's rules since #200 (the clock from the start, the
  characters to their root nodes, the game's ALE count and shapes: `Rules::Game`). The effect type is `h_xxxxxxxx` or a name (hashed). An entry
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

**#200**: the hook's effects start by the game's rules since then (`Rules`), so the numbers below
that come from `BF_TEST_EFFECT` (the cloud's and the pound's) are the checked rules' (the clock 0.1 s
after the start, the characters 1.45 m above their feet). Re-run with the five mutants at once
(`BF_TEST_NPC="0.3,h_0b5f6e21,enemy,frozen,1,-12;...;0.3,h_0b5f6e21,enemy,frozen,8,-12"`), the cloud
now ticks from t 2.00 (age 0.067, in the start's frame) to 3.00, and its distances are to the
mutants' root nodes, 0.94 m above their feet (x = 1: 1.37 m, share 0.805; 3: 3.14 m, 0.551; 5:
5.09 m, 0.273; 6.5: 6.57 m, 0.062; 8: out of range): 47.1, 32.2, 16.0 and 3.6 HP over the second,
78 x share x 0.75 each. A thrown Molotov's cloud runs by the game's rules too (`BF_TEST_INV=Molotov:5
BF_TEST_GRENADE_TYPE=Molotov BF_TEST_THROW=2.0,0.5`, five mutants in a line at 3 m steps from z =
-24, `BF_DIFFICULTY=0`): the cloud starts in the explosion's frame (t 3.93) with its first tick
there (age 0.067; before #200 at t 4.00, 0.1 s later) and measures to the mutants' root nodes (6.39
and 3.75 m; before 6.48 and 3.90): 5.1 and 27.2 HP over the second (before 4.4 and 25.9); the
bottle's own blast is as it was (4.98 and 26.91 at t 4.07, DAMAGE_DELAY after the explosion), and
a Frag's blast too (12.77 at 6.4 m, line for line). The barrels and the rack keep the checked
rules and their logs are line for line the build before #200's (238, 529 and 640 lines). How the
hook's effects look by the game's ALE rules: the cloud's and the pound's green flash and the
loose pickups thrown as before, a few more of the cloud's small green particles lingering
(frames looked at side by side).

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

* **The effects on a body** (#397; `BF_TEST_EFFECT="1,vengar,me;1,pshield,me"`, and through
  the object menu with `BF_TEST_PLACER="1,vengar;3,pshield" BF_TEST_PLACE="2;4;7"`, frames looked at,
  `BF_TOOLS_LOG=1`):
  * **pshield**: a continuous swirl of small blue-white flares round Tex's hips and legs, up to
    about a metre out, turning with him while his idle clip moves, for as long as it's on
    (three seconds looked at, `AleEffect::new` starts its 0.3 s emitter over again and again): the
    demo's construct, the game's being a single burst at a shield hit. "(h_13687abf) put on tex at
    [-0.00, 0.14, 0.00]: ALE effects ["pshield"], 0 sounds, user parameter 1, bounds Some(..., 1.023)".
    Its cone emitter and flare appearance render through `load_flight`.
  * **vengar**: one warm orange puff of the `exp5` explosion flipbook, born at his hips, drifting
    sideways and fading in about 0.4 s (frames 15-19 of the capture), then nothing. **The single puff
    is the demo's reading**: the emitter has rate 0 and no initial or emit count, and `emit` gives
    such an emitter one particle when it starts (the rule the guns' tracers were fitted to; the
    game's own code for it wasn't traced), "vengar pair 0: t 0.000 emits 1 (alive 0)", born with a
    life of 0.5 s and a speed of 1.9 m/s along the effect's z axis (the cube emitter's 90 degree
    spread: any way across the horizontal; which way depends on the seed). Seen from in front (a free
    camera) it's born inside the body and goes behind it, so nothing shows; from behind (the follow
    camera) it's plain. The ticket's "permanent aura" is only the type's max-life (5e11 s, the ALE
    entry's 5e6): nothing else in the data keeps anything on, so the aura is a name and a number,
    not a look the demo could show.
  * **On the ground** (`BF_TEST_EFFECT="1,vengar,-3,-5;1.5,pshield,3,-5"`): both start there as any
    effect type does (`start_effects`, played once: "ALE effects run 3.50 s, 1 sounds" and "0.82 s,
    0 sounds"); pshield shows a small spot of white flares for a moment, vengar nothing I could
    see. They aren't made to stand on the floor.
  * **The object menu**: the "[effects (unspawned types)]" group after the scenery with the two
    entries, "Vengar aura (unused effect vengar)" and "pshield, the shields' hit effect (unspawned
    duplicate type)" (the panel says "put it ON THE CONTROLLED CHARACTER; again: take it off";
    seen again after the relabel with `BF_TEST_PLACER=1,pshield`); `BF_TEST_PLACE` puts vengar's
    puff and the shield on Tex ("Vengar aura on tex", "Shield hit effect (pshield) on tex": the
    status line), and a third click takes the shield off: "Shield hit effect (pshield)
    (h_13687abf) taken off tex: 1 effects gone", the flares gone at once. After the relabel,
    `BF_TEST_EFFECT="0.5,pshield,me;1.5,h_13687abf,me;2.0,vengar,me"` logs the type's put on, taken
    off, and "Vengar aura (h_fb1e5895) put on tex", and the status line reads "Shield hit effect
    (pshield) on tex", "... off tex", "Vengar aura on tex" (frames 12 to 54 looked at).
  * **Two requests for one type and character in one frame count once**
    (`BF_TEST_EFFECT="0.5,pshield,me;0.5,pshield,me"`: both are logged as requests, one "put on tex"
    line follows, and the effect stays on; the toggle would otherwise take it off again at once).
  * **ALE effects nothing starts** (`BF_TEST_PLACE="1,exp-mirv-flash,-4,-6;1.3,splash_med,-2,-6;
    1.6,powerup,0,-6;1.9,stun_grenade,2,-6;2.2,energyball,4,-6;2.5,tornado,0,-14;2.8,paralyzer_hit,-6,-9"`):
    each plays once at its place and logs "ALE effect exp-mirv-flash (h_e301ce6b) plays at [-4.00,
    -1.05, -6.00] for 1.01 s" (splash_med 1.29 s, powerup 4.65 s, stun_grenade 0.81 s, energyball
    2.40 s, tornado 5.74 s, paralyzer_hit 0.77 s): a big white flash, a white-blue lightning ball,
    sparks, a haze; their look is the ALE runtime's reading.
  * **The ones that never end are cut to 3 s**
    (`BF_TEST_EFFECT="0.5,singeflame,0,-8;0.5,tornado_master,4,-8;0.5,tornado_master_40,-4,-8"`, a
    frame every 0.8 s): "ALE effect singeflame (h_fa65dee6) plays at [0.00, -1.05, -8.00] for 3.00 s
    (an estimate of 105 s: taken as one that never ends)", and the same for `tornado_master`
    (h_f7a4f80f) and `tornado_master_40` (h_115c724a), their estimate 1000000000 s. The flame's column
    is gone by 5.6 s (the cut is at 3.5 s) and the tornados' dust has faded out by about 8.5 s. A
    name that is no effect type and no ALE effect says "effect h_1fb0775a: no such effect type, and
    no ALE effect of that name hash" (`BF_TEST_EFFECT="0.5,nonexistentthing,0,-8"`); a body request
    for a type that doesn't exist, which isn't looked up as an ALE effect, says "effect h_e03f60f1:
    no such effect type" (the old message claimed the ALE lookup for both).
  * **The barrels' and the clouds' effects unchanged** (main's build against this one, `BF_DAMAGE_LOG=1`,
    a dummy Hawk at 7 m, `BF_TEST_EFFECT="2,h_1a256542,0,-7;3,h_ece33110,2,-5"`): the 30 damage ticks and
    both start lines identical, line for line.

**#208, the red tint's edge** (`AREA_TINT_RATE`, now strict: more than 30 a second at the victim). Each
staging run with the build before the change (main 094042c; the Wrath's with this branch's build before it)
and after, `BF_MUTE=1 BF_BACKGROUND=1`, the damage logs line for line the same in all six, the pictures
compared frame by frame by the whole frame's red excess (mean R - (G + B) / 2) and green over red:

* **The Wrath's owner**, take11's W4 (30 fps; `BF_MAP=mp1 BF_TEST_SQUAD_SLOT="0.3,me,h_e973af9e"
  BF_TEST_GOTO=-15.07,-24.29,-15.07,-24.29,20 BF_TEST_SWEEP=180 BF_TEST_NPC="0.5,mp_tex,friend,frozen,-14.90,-9.29,19.6"
  BF_TEST_NPC_HP=1.0,1,999 BF_TEST_INSTANT_KILL=0 BF_TEST_SKILL=2.5`): the Shaman takes 1.00 a tick, 30.0
  a second, from t 4.57 at 14.64 m. Before: a full-screen red flash at frame 137, red 21.3, green over red
  0.22, mean luma 13.3, gone by frame 145. After: 2.2, 0.95, 26.0, no flash (W1, the owner not hurt:
  1.7, 0.96, 24.7); take11's owner has none. W1 itself frame for frame the same.
* **The Orbital Beam at the wall**, take03 (30 fps; `BF_MAP=mp1 BF_TEST_SQUAD_SLOT="0.3,me,confed
  marine@sdm_ferg" BF_TEST_GOTO=-15.4,-24,-15.4,-24,20 BF_TEST_SWEEP=90 BF_TEST_SKILL=2.5`): the first
  phase, 300 a second (10 a tick at 4.95 m), still tints in the press's frame 77: red 5.7, green over red
  0.34, luma 10.3 -> 5.7 (take03's tint frame, 2.717 s: 5.8, luma 9.3 -> 7.0). The 15 a second phase from
  frame 137 (0.5 a tick): none, green over red 0.69-0.70 throughout. Every frame of the run as before
  (the worst mean difference 0.14 of 255).
* **The sdm_e34 barrel** (30 fps; `BF_MAP=sdm_e34 BF_NO_SQUAD=1 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
  BF_TEST_HIT=2.0`): 80 a second (2.67 a tick), still tints: frame 64 red 39.7, green over red 0.22,
  fading to frame 71, as before.
* **The pound on STANDARD at 60 fps** (`BF_MAP=test BF_DIFFICULTY=standard BF_NO_SQUAD=1
  BF_TEST_GOTO=0,8,0,8 BF_TEST_EFFECT=2.0,h_ece33110,0,5`): Tex at 3.23 m takes 0.50 a tick, 50 x 0.6 =
  30 a second. Before: a red tint from frame 121 (red 46.3, green over red 0.24) fading to frame 136.
  After: none (red -1.8, green over red 1.04 at frame 121), the pound's own green flash as it was; the
  frames from 137 on identical. This case isn't measured: take11's none at 30.0 decides it.
* **The Molotov's cloud 1.9 m off** (the pound's staging with `BF_TEST_EFFECT=2.0,h_1a256542,0,6.5`): 34 a
  second, still tints, every frame as before.

`tints` is `rate > 30 x (1 + 1e-4)`, not a bare `rate > 30`: the squad's STANDARD step is 1 + 0.1 x -4,
which f32 rounds to 0.6 (0.60000002), so the pound's 50 x 0.6 comes out as 30.000002 and the digit would
tint it (`the_tint_goes_by_the_areas_rate` pins it, with the Wrath's 30.0 x 1.0).

Unit tests (`cargo test`): the falloffs, the clock's wait, that a second of damage sums to its
amount at 15, 30 and 60 fps, the shake's window, ramp and falloff, the hook's entries, and the
effect type's Damage and shake records read from the data's attributes (`bf/character.rs`); and
(#397) the hook's `<who>` form and the two types' names, a word naming a character, a body
effect's name ("Shield hit effect (pshield)"), a repeat in one frame counting once, and an effect
that never ends being cut to three seconds.

## Not done

* The rumble lists (FUN_0021b410): no pad vibration.
* The checked rules' spawn (the breaking scenery) moved to the game's (`Rules`): the barrel's
  effect at the object's origin, its chain's frames before it, the characters to their root nodes,
  checked again against #85's takes.
* The effect object's end with its last ALE effect (above), and an effect's own `max-life`.
* The other spawn paths listed under "Starting one": missiles, triggers, placed effect objects.
* The game's shake reaches every character's camera in range; here the controlled character's.
* What the demo's NPCs do about a cloud (they stand in it: nothing flees fire), and any knock-down
  from a heavy tick: the pound's trigger and its effect on the squad are #212's.
