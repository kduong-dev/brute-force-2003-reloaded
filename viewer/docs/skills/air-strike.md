*Part of [skills/](README.md).*

## The Air Strike: the Militia's special skill (#203)

The Militia of deathmatch and of the Ferguson Platoon has the Air Strike (CSKILL_AIR_STRIKE 6): while
it is on, every 0.75 s a strike is called on the crosshair's target for 20 stamina, five from a full
bar, and each shell lands 2 s after its call. It is one of the effect skills ([Other characters'
skills](other-characters-skills.md): the spawner FUN_0013e410, the target point 0x13df70, the effect type
started at that point and its damage areas, all #200's); this page is what is the Air Strike's own
(`src/bin/play_air_strike.rs`). From default.xbe (the rows are in
[code-map/skills-and-stamina.md](../code-map/skills-and-stamina.md): FUN_0013b5e0, FUN_0013b660,
FUN_0013b6d0, 0x13b5d0, the vtable 0x39da28), the analyst's answers (`todo/200-effect-skills/answers.md`
Q4, Q6, Q7) and the five takes of `todo/200-effect-skills/` (the spec is `spec.md` there, §3): take05 and
take17 (a locked enemy at 20 m, held to the end; take17 at the real 70 HP), take06 (the press again after
two strikes), take07 (no lock), take13 (a teammate under the crosshair).

### Who has it

| Type | Where | HP | Cost / recovery |
|---|---|---|---|
| Militia h_055acb5f | deathmatch (objecttypes-dm.xml line 355) | 80 | 20 / 1 |
| Militia h_1563301d | the Ferguson Platoon (sdm_ferg, line 175) | 80 | 20 / 1 |

Their `<sounds>` are empty: the skill plays no start or end sound, and the Militia's chatter library
has no block for the skill's on and off tags (BF_CHATTER_LOG: "refused, no block in sdm_militia_chatter").
In the demo the Militia is a **type of its own** (the test map's squad slots, #302), and takes its
type's Air Strike as the Confed Marine takes his Orbital Beam (`effect_skills::own_type_skill`).

### What it does

* **The press** (V; FUN_0013b660, `AirStrike::toggle`): with the skill **on**, off at once (the shared
  switch's `deactivate`, FUN_0010e3e0) and the timer cleared; the strikes already called are effect
  objects of their own and land all the same (take06: the press again at 3.7 s after two calls, off in
  the frame, no third, both shells land: the damage 20.21 and 18.41, the picture's flashes, the sound's
  second burst). Off, with stamina at least the cost (20, the equality in): a call, the cost off at once
  and the timer set to the frequency (0.75), then the shared switch on. Under the cost, nothing at all
  (`Toggle::Refused`: no name, no sound, no effect).
* **A call** (FUN_0013e410(0), then FUN_001258d0): `Skill::fire`, which `effect_skills::fire_effects`
  carries out in the frame: the target point found **anew** at each call (the reticle's lock's hips, else
  the aim's rays: take05 and take07 read the same point at all five calls, the scatter being added at the
  spawn, not kept), the effect h_007aa97d made at the point plus up to 2 m either way on x and on z
  (`EffectData::air_strike`: the XZ offset 2, h_f6bfb82f, a uniform square), turned by the pitch 1 rad and a
  random angle about the vertical, its owner the Militia. The rise (h_e272cd2c, 1) lifts a point found on the
  terrain by 1 m; the Arena's floor is a static object, so take07's point is on it (no lift).
* **The update** (FUN_0013b6d0, after the stamina rule's drain step): while on, the timer counts down by
  the frame's time, and **in the frame it is at or under that time** (`timer <= dt`) the next call is
  made if the stamina is there for it (the timer set to 0.75 again, none of the frame carried over), else
  the skill is switched off. So the interval is whole frames: 23 at the game's 30 Hz (0.767 s; the first
  one 22 in the demo, whose press's own frame counts the timer down too: the command comes before the update,
  and the takes' 20 Hz samples can't tell 22 from 23), 12 at a 15 fps capture (0.8 s), 45 at 60 Hz (0.75 s). The takes: 0.756 `gt` s a call over 17 intervals (each +-0.05 from the 20 Hz sampling; the
  timer itself counted 0.9885 game seconds per `gt` second in take05) and the timer read at 0.0077, 0.0034
  and 0.0016 just before a call, never under 0.
* **The end**: the drain is 0 (0x13b5d0: nothing is taken while it's on but each call's cost; no
  refill either, the steps are 80.00, 60.00, 40.00, 20.00 exactly), so the fifth call from a full bar
  leaves 0 and **the stamina rule ends it the frame after** (`skills::step`: at 0 the update sends the
  toggle command itself; take05's 0.03 in the sample of the off is that frame's refill, +1.0 a second
  after). The update's own end comes only from a bar that isn't a multiple of 20 (from 70: calls at 70,
  50, 30, then off at the fourth timer's end with 10 left; the code, not filmed). And a press at **20 to
  29.9** stamina calls, pays 20 and the skill is **never on**: the shared switch's gate (10% of the
  maximum, FUN_0010e470) is tested on the stamina left, so one strike and nothing more (the code only).
* **The look and the damage** are the effect type's (play_effects.rs `Rules::Game`): eight ALE effects,
  the smoke trail and its flare at the call and the six of the shell's burst **2 s later** (the alchemy
  entries' creation-time), the sound h_065168c9 at 2 s, and the damage area, 40 EXPLOSION a second from
  1 s for 0.5 s within 20 m with a linear falloff: so a strike's hit points go **1.0-1.5 s after its call,
  before the picture** (take05: the first HP loss at 3.57 s, the flare at 4.70, the sound at 4.63), at most
  20 and `20 x (1 - d / 20)` at d m from the effect to the victim's hips. Everyone in range is hurt, the
  owner and his team too (take05: the owner 20 m off took 1.94 in all, take13 ~4).
* **The HUD name** "Air Strike" (h_f91b25db, vtable +0x4c) shows while it's on, as any skill's, and
  fades 0.15 s after the off at 3 a second ([HUD](hud.md); take06 full to +0.2 s after the off, faint at
  +0.4, gone at +0.5, the demo's the same frames).
* **The AI**: `SkillKind::attack_skill` is true (the vtable +0x10c), so the Squad Command menu's Toggle
  Ability refuses it for a squad member, as for the Orbital Beam (the goal would write its target's
  position into the skill, which the demo's goal doesn't).

### Three shared changes it needed

* **Creation-times** (`play_effects.rs` `start_effects`): an effect type's ALE effects are made at once
  only when their alchemy entry's creation-time is 0, the others when the object's age reaches it
  (FUN_00219fc0: `ale_fx.rs` `StartAfter`, which the placed effect objects already had). The Air Strike's six
  shell effects are at 2 s; every effect type the other spawns start (the barrels', the grenades', the
  Orbital Beam's) has all of them at 0, so none of those changed (**Unchanged**, below). An entry at or past
  the type's max-life is never made (the object has ended: `spawn_placed_effects` leaves it out too), so the
  ones that would wait for ever make no entity: e01's h_09af3040 (max-life 0, an entry at 7.7e31 s) and the
  Vengar hit's `vengar_hit` (6.9e28 s of 657656).
* **The red screen tint** (`AREA_TINT_RATE`): an area's first tick on the controlled character gave the red
  tint whatever its size, so the owner 18-20 m from a shell flashed red at each strike, which neither
  take05 nor take13 shows. The tint now needs the area to hit him at **30 HP a second or more** (a guess: the
  Sonic's 1 HP a hit, play_sonic.rs, as one HP a frame at the game's 30 Hz), once an area, from the first tick
  that does. By the rate at him (`per_second`: the record's rate x the falloff x every factor of the hit, the
  tick over its slice), not by the frame's tick: the first tick of an area is a sliver of a frame, and 1/60 s
  of a 50 a second pound is under an HP (a first version gated on the tick and lost the barrels' and the
  rack's tint at 30 fps and the pound's and the cloud's at 60). take03's owner at the wall, the first phase's 300 a
  second, goes red at the first tick (its green and blue fall to 0.12 of its red, 2.73 s), and at the 15 a second
  area's first tick (4.73 s) they don't (0.49 -> 0.43); the barrels (80 a second and more: take29, "the whole
  top half turns red") and the pound (50, x0.6 for the squad on STANDARD: 30, the edge) keep theirs; the Air
  Strike's owner, 0.6-7 a second, none. The Molotov's cloud keeps its tint within about 2.5 m of its centre
  (78 x 0.6 x (1 - d / 7) from 30 up) and loses it beyond: unmeasured.

### Keys and test hooks

* V, or `BF_TEST_SKILL=<s>[,<s>...]`: the toggle (the second time off).
* The Militia on the test map: `BF_TEST_SQUAD="3:militia@sdm_ferg"` (he takes Tex's place and leads), or on
  any map of a test session `BF_TEST_SQUAD_SLOT="0.3,me,militia@sdm_ferg"`.
* `BF_TEST_GIVE_SKILL=6[,<cost>[,<recovery>]]` gives the character controlled at a map's start the class
  with the Militia's cost 20 and recovery 1 (`9` gives the Orbital Beam its Marine's 55 and 2.3).
* `BF_TEST_STAMINA=<n>@<s>` for a bar that isn't full (`25@2.0`: the single strike; `70@2.0`: the update's
  own end); `BF_SKILL_LOG=1`: each call ("air strike call 3 (the timer): stamina -20 -> 40.00"), the target point
  and how it was found, the switch on and off and why; `BF_DAMAGE_LOG=1` each tick and each area's totals,
  `BF_SOUND_LOG=1` the shell's sound.

### Verified (#203)

`cargo run --bin bf_play -- --test` with `BF_MUTE=1 BF_BACKGROUND=1 BF_DIFFICULTY=0`, `BF_CAPTURE`
at `BF_CAPTURE_FPS=30`, take05's staging on the Arena (`BF_MAP=mp1 BF_TEST_SQUAD_SLOT="0.3,me,militia@sdm_ferg"
BF_TEST_GOTO=-15,-26,-15,-26,20 BF_TEST_SWEEP=180 BF_TEST_NPC="0.5,mp_tex,enemy,frozen,-14.89,-6.0,19.6"
BF_TEST_NPC_HP=1.0,1,999 BF_TEST_INSTANT_KILL=0 BF_TEST_SKILL=2.5`), the frames laid beside the takes' and
aligned at the call:

* **A1, take05** (locked, held to the end): the lock and the point (-14.88, 20.12, -6.00) (take05:
  (-14.89, 20.12, -5.98)), the crosshair red with its lock; calls at 2.57, 3.30, 4.07, 4.83 and 5.60 s,
  22, 23, 23 and 23 frames of 1/30 s apart (take05: 0.767, 0.762, 0.769, 0.761 `gt` s each +-0.05; first to
  fifth 3.03 s against 3.0-3.06), the bar 80, 60, 40, 20, 0.00, off in the frame after the fifth with
  0.03 in it (take05: 0.03) and +1.0 a second after. The areas start 1.0 s after each call (the first
  tick at 3.57 for the call at 2.57; take05 0.95-1.0 `gt` s), the shells' flare at +2.0 s, its flash on
  the target at +2.15 and the fireball from +2.25 (take05: the flare at +2.0, the flash at +2.05-2.14, the
  fireball from +2.2 game seconds), each strike's sound `h_065168c9` at +2.03 (the frame after +2.0; take05:
  +1.95-1.98 game seconds). The strike's damage on the target, `20 x (1 - d / 20)` at the scatter's d:
  19.0, 18.7, 18.5, 19.1, 19.8 for d 1.05, 1.28, 1.53, 0.86, 0.17 m; take05's 18.61, 18.45, 17.25, 18.32,
  17.45 are d 1.4, 1.6, 2.8, 1.7, 2.6 m. The ten strikes on a lock in take05, take06 and take17 average 18.28
  HP, between what a uniform square of half-side 2 (mean d 1.53 m) gives for an exact 0.5 s window, 18.47,
  and for the game's 15 ticks of 0.0329 s, 0.4935 s, 18.23. The owner 20 m off: 0.4, 1.2, 1.5 HP from the strikes that fell toward him (take05
  1.94 in all) and no red tint.
* **A1b, take17** (the real 70 HP): the target dies in the fourth strike's burst, 0.37 s into it, with
  14.0 of its 18.5 dealt (take17: 15.94 dealt and dead at 0.45 s), the kill the owner's ("the kill of Tex"
  in the chatter log).
* **A2, take06** (`BF_TEST_SKILL=2.5,3.7`): calls at 2.57 and 3.30, the press at 3.77: off in that frame
  with 60.00 and +0.03 the frame after (take06: 60.00, then 60.06), no third call, the stamina refilling
  at 1.0 from then; both strikes land (19.0 and 18.7), their sounds at 4.60 and 5.33; the name full until
  0.2 s after the off, faint at 0.4 and gone at 0.5, as take06's frames.
* **A3, take07** (no lock: `BF_TEST_NPC` left out): the blue crosshair, the point (-15.00, 18.93, -1.94 to
  -2.25) found as the floor's face (take07: (-14.83, 18.93, -2.49 to -2.24); the Arena's floor is an object:
  no lift), 23.8-24.1 m from the feet (23.5), the effect 0.2-1.5 m from it each time: the scatter on top, within
  +-2; the strips beside take07's match (the flare from the upper right, the flash on the floor, the orange burst).
* **A4, take13** (`BF_TEST_NPC=...,friend,frozen,...`): the crosshair green, no lock, the point on the body
  (-15.00, 19.36, -6.38) (take13: (-14.86, 19.26, -6.23)); the teammate takes 18.5, 18.2, 17.9, 19.1, 19.2
  (take13: 16.99, 16.85, 17.33, 17.77, 18.28 with the point 0.86 m under his hips) and the owner 20 m off
  0.7, 1.6, 1.9 and 0.2 HP (take13 ~4).
* **A press at 25 stamina** (`BF_TEST_STAMINA=25@2.0`): at 25.5 one call (-> 5.50), "not switched on at
  stamina 5.50 (under 10%)", no more; **from 70** (`70@2.0`): calls at 70.5, 50.5 and 30.5, off at the fourth
  timer's end with 10.50 left, by the update's own end.
* **The pitch** (`effect_skills::turn`, the reviewer's request of #200): the effect is tilted 1 rad by
  FUN_0013e800 and swung about the vertical by a random angle by FUN_0001fa20 (M = Ry(a) x Rx(pitch), the
  matrices the engine multiplies as columns, as #49's pellet walk reads them), a random side each time. The
  takes agree: each shell's flare comes in from a different side of the picture (take05: centre-left, right,
  right, centre, right; take07: upper left, upper left, right, centre), where a fixed side would bring every one in
  the same way; the demo's five are as varied (`the_turn_leans_by_the_pitch_toward_a_random_side` holds
  the composition: the up axis 1 rad off the vertical, the azimuths spread round it).
* **Unchanged**: the four squad skills (`BF_CHARACTER=<tex|brutus|hawk|flint> BF_TEST_SKILL=1.5,5.0
  BF_SKILL_LOG=1`) and the Orbital Beam (take01 locked at 15 m, take03 at the wall) against
  `bf_play_968c35c.exe`: the skill, damage and effect log lines the same, the captures pixel for pixel (take01 to
  0.016 of a grey level a frame) but take03's frame at 4.77 s, the 0.5 HP tick that no longer tints (above).
  The tint gate's re-check (the green over red of the picture, frame by frame, the baseline against the branch):
  the sdm_e34 barrel at take29's spot at 30 fps (`BF_MAP=sdm_e34 BF_NO_SQUAD=1 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
  BF_TEST_HIT=2.0`: tinted frames 64-66 and the fade to 75 on both, the combat log line for line), the e34 rack and its
  chain at 30 fps (`BF_MAP=e34 BF_TEST_GOTO=-31.86,14.77,-31.86,14.77 BF_TEST_HIT=2.0 BF_TEST_HIT_NEAR=-31.86,21.87`:
  frame 63 and the fade, the same), the pound (`BF_NO_SQUAD=1 BF_TEST_GOTO=0,8,0,8 BF_TEST_EFFECT=2.0,h_ece33110,0,5`) and
  the Molotov's cloud 1.9 m off (`h_1a256542,0,6.5`) at a 60 fps capture (frames 121-125 on both, 0.5 and 0.57 HP a
  tick), the cloud 4.2 m off the one place it differs (19 a second: tint on the baseline, none now); take03's
  15 a second phase, no tint; the Air Strike's owner, hurt 0.4, 1.2 and 1.5 HP, no tinted frame in the run.
* Unit tests (`cargo test`): the five calls from a full bar at 30 Hz (the frames 0, 22, 45, 68, 91, the
  bar, off in the frame after), 60 Hz, the second press, the gate under the cost, the single strike, a bar
  that isn't a multiple of the cost, the turn; the `<air-strike>` block's read (`bf/character.rs`), the hook's
  defaults; the tint gate (`play_effects.rs` `the_tint_goes_by_the_areas_rate`: the data's two sides, and the tick
  over its slice the same rate at 15, 30 and 60 fps and in a sliver).

### Open and not done

* **The scatter's distribution** isn't measured by an object read: the ten lock strikes' mean agrees with a
  uniform square, take13's five (the point on a body) are a little further (the body's offset is not known).
  The effect's own position would settle it (shot S6, `todo/200-effect-skills/shotlist.md`).
* **A moving target** (the point at each call): the code reads it anew at each call (the demo's `fire_effects`
  does); not filmed (shot S4).
* **The tint's rate** (`AREA_TINT_RATE`, 30 a second) is a guess between the measured 15 (none) and 80 (tint); the
  game's own trigger isn't found (FUN_00125ad0 has none), and #85's take28 (FORCE POWER, half the damage) shows one red
  frame where take29 shows 0.2 s of it, so the game's tint may scale with the hit.
* **The shell's descent** is hidden by the corridor's ceiling in every take; the ALE data's trail and flare
  are what the demo draws, whiter than the console's (the ALE runtime's colours, as the Orbital Beam's).
* The skill for an AI Militia (no personality posts the toggle, and the demo's goal refuses an attack skill) and
  the dm character select (#45): the class is reached by the squad slot hooks above.
