*Part of [play/](README.md), the former page `play.md`.*

### Damage: the difficulty step and the hit parts (#127)

`src/bin/play_damage.rs`, from default.xbe's damage path as the analyst read it
(`todo/damage-factor/answer.md`; every function cited is in [code-map.md](../code-map/README.md)). A hit
on a character goes:

* **The roll** (FUN_0022c940): Damage min + U x (max - min), x a character shooter's skill's
  factor (its +0xec at 0x22c9a3; `Hit::skill`, the log's "x the shooter's skill"): Flint's x2 on
  15% of her rolls while her targeting is on and x1.5 scoped (0x13cc70, [skills.md](../skills/flints-advanced-targeting.md));
  1 for the others; then x each of its modifiers' (char+0x55c, `Hit::power`, the log's "x the
  shooter's power-up"): BRUTE POWER's x2 for its 45 s ([pickups.md](../pickups/power-ups.md), "Power-ups",
  #15). Every roll a shooter makes takes them: a shot's on a body or a breakable object, a bolt's
  and a ray's, a missile's direct hit and its blast's own roll. The Hat's char+0x754 (x4,
  FUN_001478d0) isn't in.
* **The hit record** (FUN_0022c4d0): a direct hit takes the roll, a blast its share of the
  blast's roll (half for h_011cb154 false, the Therm Sweeper's); then x the hit factor: an
  instant ray's range falloff (FUN_00230040, **Shots**) x FUN_00125940; a projectile's
  FUN_00125940 (the missile's +0x1cc is always 1.0, FUN_002278e0); a blast's share 1
  (FUN_00224a90 passes 0 to FUN_00224320). **FUN_00125940** is the victim type's
  `damage-multiplier` (h_e7e9fcd3, type+0x220: the campaign squad 0.62; the Shrikes and the
  squad deathmatch's sdm_bf squad 1) x the struck hit part's bone's `Damage` (the motion set's
  `<collision>` list: the head x5, x3 for Brutus and the hounds; upper arms x0.8; forearms and
  calves x0.5; Bip01, Spine2 and the thighs x1). Then x0.7 when the shooter and the victim are
  both the squad's (FUN_0004b9c0; the demo's sides stand for its squads: the same team, the
  shooter's own blast included).
* **The victim** (0x125600, a character's combat-target vtable slot 0): x (1 - 0.1n) on an
  enemy team, x (1 + 0.1n) on the squad's, n the difficulty; then its skill's factor (Vengar's
  0.85 while on, `Hit::victim_skill`, skills.md); then each of its modifiers' (`Hit::victim_power`,
  "x the victim's power-up": FORCE POWER's x0.5 on any hit but a BLADED one, #15); then
  FUN_002232d0's factor for the damage type. Every hit on a character
  reaches it: the guns' hits and blasts, the scenery's damage areas (FUN_0021ade0), falls
  (FUN_0012b740, DTYPE_NORMAL), FUN_00223160 (Brutus's charge), the damage plate's frames
  (FUN_001f8510, #228: [triggers.md](../triggers/the-damage-plate.md), "The damage plate"). Only the trainer's `!hit` skips
  it (it calls FUN_002232d0 itself), so older xemu logs of `!hit` carry no step.

The victim's values are its character type's (`Player::kind`, #128,
[character-types.md](../character-types.md)): its hit parts (its motion set's `<collision>` list)
and its damage-type factors by type; its damage-multiplier by its squad slot's own entry first
(the sdm_bf squad's 1 on a squad deathmatch map), else its type's. The squad's are what they
were; a test map NPC of an enemy type takes its own (a Shrike's SONIC x2 and GAS x0.2, its head
x5).

**The difficulty** (`Difficulty`, set as each map begins):

| Menu (FUN_000f13c0) | `<difficulty-settings>` (game-options-en) | n | Enemies take | The squad takes |
|---|---|---|---|---|
| STANDARD | `easy` -4 | -4 | x1.4 | x0.6 |
| HARD | `normal` 0 | 0 | x1 | x1 |
| BRUTAL | `hard` 4 | 4 | x0.6 | x1.4 |

* The values are read from `common/game-options-en.xmb` (`Game::difficulty_settings`). The
  sign goes by the victim's team: its row of the team table (0x3ffd80) has 1s in columns 3-6
  for an enemy (teams 1, 7, 8); the squad's 0 and the others take the plus.
* **The default**: the missions and the test map play STANDARD, the takes' (both sides of their
  HP logs fit n = -4: answer.md; #129's campaign takes read n = -4 live). The multiplayer maps
  (the game's level list: the squad deathmatch maps sdm_*, the arenas mp*) play none, n = 0.
  **Read live**: in #129's deathmatch takes on mp1 (todo/129-shrikes/notes.md, B2: takes 35, 36,
  38, 39) the getter gave n = 0 at 0x12566d, and each hit was the roll x the bone's `Damage` with
  no x1.4 (Soldier 30.1-31.4, Heavy 29.6-31.9). Two takes agree: #85's take29 on sdm_e34 had
  Tex take 39.4 of a barrel's 40 (the campaign's took ~24, takes 01-04), and #146's takes 01-03
  on sdm_e13 had Brutus lose exactly 30.00 per lava entry, the lava's rate.
* **How a match gets n = 0 isn't traced** (a note): 0x125600 applies the step on every map, with
  no mode check. The getter (0x7c720) reads the flag from the current slot's tree (the session's
  vtable +0x210, 0x78d60: [session + 0x5cc + 4 x [session + 0x5c8]]) through FUN_00078320,
  which gives 0 only without a tree or without the flag; the default game flags hold it at -4
  (`common/default-gameflags.xml`'s `<campaign>`, where the decoder names its hash h_1a98de0a
  `shadoon-look`, a name collision with `difficulty-setting`). The setter's callers are SELECT
  DIFFICULTY, RESUME LAST and QUICK CAMPAIGN (STANDARD: FUN_000f7140, FUN_000f7230) and the
  system-link join at 0x1eef81; none is on a split-screen match's start path. So a match's slot
  evidently lacks the flag; which slot it uses isn't traced. System-link play, whose join calls
  the setter, is untested (the demo gives it n = 0 like any match).
* **What changes on sdm_e34**, where the demo has no step:
  * the grenades' thrower shares are x1 / 0.6 of STANDARD's: the Frag's thrower takes 21.0
    where it takes 12.6 on STANDARD, as the game's half roll x0.7 gives (0.35 of a roll, 22.1
    for the same roll); the Sonic's and the Energy's thrower shares the same;
  * the Gas poisons at 29.5 HP/s (17.7 on STANDARD);
  * #85 take29's barrel deals 40.0 to Tex, the take's 39.4;
  * every gun's hit on a squadmate takes the struck part's `Damage` and x0.7 (sdm_bf's
    damage-multiplier is 1): a head shot x3.5 (Brutus's x2.1), a forearm or calf x0.35; the
    AI's shots on their own side take the x0.7 too;
  * the missiles (#73 had the part and the x0.7 already) are as #73 had them.
* `BF_DIFFICULTY=<standard|hard|brutal>` (any case) or a number n sets it, on any map (on a
  multiplayer map too: a test hook, to put the takes' STANDARD on the sdm_bf squad's
  damage-multiplier 1, the Shrikes').

**The hit parts** (`struck_bone`). On the squad's models exactly the eleven bones of the
`<collision>` list have a collision surface in their part archetype (`<h_f3f067ad
h_051ac63e>`, objects-common.ipn; `Character::part_collision`, in the bone's frame): the bodies
FUN_00114010 makes a character's hit parts of (FUN_00114d00 builds each from the part's node).
A shot that meets a standing body (the demo's 0.4 m cylinder, `ray_body`) is followed through
it, and the first part surface it meets, posed as drawn, is the part struck: the game's ray or
missile strikes those bodies themselves and FUN_00115000 gives the body's bone. A shot through
the cylinder that meets none of them (between the legs, past an arm) takes the part whose bone
comes nearest its way through the body: the demo's (the cylinder still decides what's hit; in
the game that shot would pass). Knocked down, the part whose bone is nearest the contact on the
ragdoll (#73's rule). The surfaces sit on the drawn models (`BF_HIT_PARTS`): a character
holding its gun in front takes a frontal shot at the hips on a forearm (x0.5), and Brutus's
head (x3) hangs low in front of his chest.

**The grenades and the scenery.** The grenades keep their own rules (grenades.md; the game's
blast rule is #112's) and take the step and the damage-type factor through the same `Hit`.
Their amounts fitted to recordings on the campaign squad on STANDARD (the thrower's share of a
Frag, Sentry or Roller blast, of the Sonic's and of the Energy's bolt, and the Gas's rate) hold
that x0.6 already: they're divided by it (`footage_squad_step`), so they come out as measured
on STANDARD and scale with the difficulty like any hit (with no step, on the multiplayer maps
or with `BF_DIFFICULTY=hard`, x1 / 0.6 of them: the Frag's thrower 21.0 for a roll of 63,
where the game's half roll x0.7 gives 22.1). Measured on the thrower in its own blast, they hold FUN_0004b9c0's x0.7 too,
so no x0.7 is added (`Hit::source`); the others' rule (Damage max falling to nothing at the
radius) is the demo's, without it. The Gas's 41.3 HP/s on an enemy on STANDARD is the demo's
extrapolation (with the x0.7 taken out too it would be ~59). The scenery's damage areas take
the step: the campaign squad's ~0.58 of a barrel (scenery.md). Not stepped: the test hooks'
kills (`BF_TEST_DIE`, the test map's suicide and instant kill) and a harmful liquid (its game
path isn't traced).

**The log** (`BF_COMBAT_LOG=1`): the map's difficulty as it begins (`difficulty on e34:
STANDARD (n -4): enemies take x1.40, the squad x0.60`), then each hit with every factor, before
`hurt`'s line with the hit points left:
`t 3.60: tex -> tex (NPC 1): roll 93.62 x damage-multiplier 1.00 x Bip01 1 x STANDARD (n -4) enemy 1.40 x type 10 1.00 = 131.07`.
A part found by the demo's own rules is tagged after its `Damage`: `(nearest)` for a shot that
passed the cylinder between the part surfaces, `(nearest, lying)` on a body knocked down; an
untagged part is the first surface the shot met. Damage dealt every frame (the Gas, a damage
area) gets no hit line; the areas' own line shows the step. `BF_HIT_PARTS=1` draws every standing character's hit-part surfaces as lines (the
head red, upper arms yellow, forearms and calves cyan, the x1 parts white).

Verified (15 fps, `BF_COMBAT_LOG`; the takes' numbers and allowed ranges are answer.md's):
* **#73 take03** (Therm Sweeper, a Shrike Heavy 7 m out, damage-multiplier 1): `BF_MAP=sdm_e34
  BF_DIFFICULTY=standard -- --test BF_TEST_NPC="0.5,tex,enemy,dummy,-0.26,41.69;0.5,hawk,enemy,dummy,1.3,41.69"
  BF_TEST_WEAPON="Therm Sweeper" BF_TEST_TRIGGER=3.0-3.15 BF_TEST_INSTANT_KILL=0` (the sdm_bf
  squad's 1 standing in for the Shrike's): the direct hit at Bip01, 93.62 x 1.4 = 131.1 (the
  takes 118.2-134.1, allowed 117.6-134.4); the other enemy's share 42.70 x 1.4 = 59.8 (59.07-66.9);
  the squad near it 17.9 (0.21 of the roll). A Brutus dummy in Tex's place takes the missile on
  his forearm (x0.5: 65.5). BRUTAL: 56.2 direct and 25.6 shares (answer.md: 50.4-57.6,
  25.2-28.8), the squad 41.9 (41.2-47.0). sdm_e34's own default (none): 93.6 and 42.7.
* **#73 takes 10-16** (Tex's own blast): at a test map pillar 2 m ahead (`-- --test
  BF_TEST_GOTO=0,-15.7,0,-16.2 BF_TEST_NPC=0.5,flint,friend,dummy,-1,-15.7 BF_TEST_WEAPON="Therm Sweeper"
  BF_TEST_TRIGGER=3.0-3.15 BF_CAMERA_PITCH=-0.05 BF_TEST_INSTANT_KILL=0`): Tex and Flint 19.66 each, 0.5 x 93.62 x 0.7
  x 0.6 (the takes 18.17-20.11; Flint 19.98, 19.67). (At the takes' e34 gate, #73's recipe
  above, the gate opened in this build with Tex 4.6 m from door 6, which has no panels on e34,
  and the missile flew through it: the pillar stands in.)
* **#35 take27** (Brutus shot by an enemy's Saryl-45 at 13 m): `-- --test BF_CHARACTER=0
  BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,20 BF_TEST_NPC=0.5,brutus,enemy,fight,4.5,7
  BF_TEST_WEAPON=Saryl-45 BF_TEST_INSTANT_KILL=0` (the enemy, a Brutus too, carries the hook's gun): 14 hits of
  4.77-5.69, rolls 12.83-15.30 x 0.62 x Bip01 1 x 0.6 (the take 4.93-5.81, allowed 4.74-5.86).
  `BF_DIFFICULTY=hard`: 7.95-9.15 (answer.md's HARD 7.9-9.8). With Vengar on (`BF_TEST_SKILL=8`,
  #122) each hit x0.85: 4.09-4.87 (mean 4.50; the take 4.43), skills.md.
* **#35 takes 04 / 05** (the hovering Shrike's legs): `BF_MAP=sdm_e34 BF_DIFFICULTY=standard -- --test
  BF_TEST_GOTO=4.77,36.82,4.41,37.17 BF_TEST_NPC=0.5,tex,enemy,dummy,-0.26,41.69 BF_TEST_FIRE=1
  BF_CAMERA_PITCH=-0.28 BF_TEST_INSTANT_KILL=0`: the LZR-50 (`BF_TEST_WEAPON="Confed LZR-50"`) 19 hits, all on a calf
  (x0.5), 2.65-5.22 (the takes 2.6-4.9, allowed 2.45-5.25); the Jax-iC 15.4 and 17.5 on the
  calves, then 32.5 on the knocked-down dummy's right thigh: since #124 the shots go to the
  crosshair's lock, which follows the body down. With m09_a's Shrike Heavy in Tex's place
  (`BF_TEST_NPC=0.5,shrike heavy@m09_a,enemy,dummy,-0.26,41.69`, #128) its SONIC x2 makes the
  Jax-iC's 30.7 and 29.7 on the calves (the takes 30.1-35.2; the second roll 21.2, the takes'
  21.5-25.1), then 70.2 and 68.8 on a thigh (#124's lock aim, as above; before it and #129's
  merge, 30.7 and 34.8); the LZR-50 2.48-5.22.
* **#35 take28** (Flint's L-Shot-75, x1 parts): `BF_MAP=sdm_e34 BF_DIFFICULTY=standard -- --test
  BF_CHARACTER=1 BF_TEST_NPC=0.5,tex,enemy,dummy,-0.26,41.69 BF_TEST_WEAPON=L-Shot-75
  BF_TEST_TRIGGER=3.0-3.1,4.5-4.6,6.0-6.1 BF_TEST_INSTANT_KILL=0`, from the sdm_e34 start (no
  `BF_TEST_GOTO`): 44.7, 45.1, 40.7 on the right thigh (the take 40-44, allowed 38.2-46.6).
  Flint's doubled and scoped rolls (#124): skills.md § How it was checked.
* **#115** (the Bower 20's falloff): Brutus's shell at a campaign Tex 7.7 m out on the test map
  (`-- --test BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_TEST_NPC=0.5,tex,enemy,dummy,5.1,11.5 BF_TEST_WEAPON="Bower 20" BF_TEST_TRIGGER=3.0-3.08
  BF_CAMERA_PITCH=-0.16 BF_TEST_INSTANT_KILL=0`): two pellets, aimed at the crosshair's lock
  since #124: 16.70 x falloff 0.682 x 0.62 x 1.4 = 9.88 (Bip01) and 18.57 x 0.678 x 0.62 x 0.8
  (an upper arm) x 1.4 = 8.74 (before the lock aim, 9.85 on a thigh and 5.40 on a forearm). The
  take's m03_a mutant (BALLISTIC x0.8, multiplier 1) isn't in the demo.
* **The scenery's areas**: campaign e34 at #85 take01's spot (`BF_MAP=e34
  BF_TEST_GOTO=-31.52,16.64,-31.52,16.64 BF_TEST_HIT=2.0`): Tex 115 -> 91.0 over 2.13-2.60 s
  (take01: 91.22 by 2.59 s). take29's spot on sdm_e34 (`BF_MAP=sdm_e34
  BF_TEST_GOTO=-53.96,24.6,-53.96,24.6 BF_TEST_HIT=2.0`): 40.0 (take29: 39.4); the
  start-of-map line there reads `difficulty on sdm_e34: none (n 0)`.
* **The grenades** on the test map (STANDARD): the Frag's thrower takes 12.6 of 115 (as before:
  the thrower's share 21.0 x 0.6); the Sonic's (W2 of grenades.md) 8.0 at 4.1 m (7.8 at 4.2 m
  before); the squadmates it reaches x0.6 of before.
* **The hit parts** (`-- --test BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_TEST_NPC="0.5,brutus,enemy,dummy,4.5,13;0.5,tex,enemy,dummy,6,13;0.5,hawk,enemy,dummy,3,13;0.5,flint,enemy,dummy,1.5,13"
  BF_HIT_PARTS=1 BF_TEST_FREECAM=0.5,3.75,1.2,17.5,0,-5`): the four squad models' parts sit on
  their drawn bodies: the head, the upper arms, the forearms on the guns, the torso, thighs and
  calves.
