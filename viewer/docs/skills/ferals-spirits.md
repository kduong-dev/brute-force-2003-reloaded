*Part of [skills/](README.md).*

## The Feral Colonist's Spirit of Vengar (#202) and the Feral Outcast's Spirit of Garner (#209)

Two more users of Brutus's skill ([Brutus's Spirit of Vengar](brutuss-spirit-of-vengar.md)): the
Feral Colonist of deathmatch and the Feral Hunting Pack (CSKILL_SPIRIT_OF_VENGAR 14) and the Feral
Outcast (CSKILL_SPIRIT_OF_GARNER 15). **Not filmed**: neither skill was recorded in the original
game, so everything below is from the game's data and code; the acceptance items are marked "not
filmed" and the shot list is at the end.

**What the game does** [code]: the factory (FUN_0012e590) makes the Colonist's skill with
FUN_0013dd80 and the Outcast's with FUN_0013f000. Each runs Brutus's constructor (FUN_0013bef0)
and then copies the nine values of its own block of game-options' `<combat-skills>` over his
fields (the settings +0x450.. for the Colonist, +0x424.. for the Outcast, in Brutus's order). The
vtables (0x39cc28, 0x39c5f0) are Brutus's slot for slot but the two effect getters (+0x134 and
+0x138, the block's effect names) and, for the Outcast, the name (+0x4c). So the toggle, the drain,
the regeneration, the damage taken, the charge and the view are the code of Brutus's, and only the
values differ. The code map has the rows (FUN_0013dd80, FUN_0013f000, FUN_0019ef70, FUN_0019edc0,
the vtables: [Skills and stamina](../code-map/skills-and-stamina.md)).

| | Brutus (h_f5dc6aac) | Spirit of Vengar (h_fec2c7d8) | Spirit of Garner (h_08c5e089) |
|---|---|---|---|
| regeneration, hit points a second while on (h_eaa78fdf) | 1 | **5** | **0** |
| damage taken, x while on (h_f4f602c6) | 0.85 | **1.2** | **0.5** |
| charge hit's stamina (h_0461a627) | 100 | 100 | 100 |
| knock-back up / weight / force (h_f5dae098, h_0e406b26, h_fa20e794) | 2 / 2 / 75 | 2 / 2 / 75 | 2 / 2 / 75 |
| charge reach m, cone degrees (h_e26c9e84, h_f3d82857) | 1.5, 30 | 1.5, 30 | 1.5, 30 |
| charge damage, DTYPE_EXPLOSION (h_14935eb6) | 200 | **100** | **700** |
| hit effect (h_0571f62e) | h_0dcd4fc7 | h_0dcd4fc7 | h_0dcd4fc7 |
| second effect (h_e5417809) | none | none | h_1e404572 |
| HUD name (+0x4c) | "Vengar" h_055b624b | "Vengar" | "Spirit of Garner" h_0e51020f |

The values are game-options-en.xml's lines 186 (Garner) and 187 (Vengar); `VengarSkill`
(`bf/character.rs`) holds one record for each block, `CombatSkills::spirit_of_vengar` and
`spirit_of_garner`, read from the file (a block's missing attribute keeps the shipped value).
The second effect h_1e404572 is defined in no objecttypes file and the getter's (+0x138) caller
wasn't traced: nothing to draw (the Outcast's charge kill starts no effect, as Brutus's).

**Who has them** [data]: the dm Feral Colonist h_086bd9cc (`<combat-skill skill-set="14"` cost 2.9,
recovery 1.05; 75 HP; start / end / hit sounds h_ee753485 / h_190cd1f6 / h_e4c94dfe, Brutus's)
and sdm_feral's h_01208936; the dm Feral Outcast h_e3a313c1 (skill-set 15, cost 3.3, recovery 1; the
same sounds) and sdm_feral's h_19766085. The view's mask (FUN_00210060 gives the ferals' keys the
mask 0x119, the Vengar controllers: the analyst's reading in the tickets, not re-read here) is
Brutus's too.

**In the demo** (`Vengar` in `play_skills.rs`, `play_vengar.rs`):

* `Vengar` is now the class of all three: it carries its block (`s`) and its name; it was
  Brutus's with his values read from `settings.vengar` in two places. Before this, the charge read
  `combat_skills.vengar` whatever character charged and `vengar_on` asked for Brutus's name; now
  the charge reads the charging character's own skill (`skills::vengar_tuning`, `SkillKind::vengar`)
  and `vengar_on` is "a Vengar class and on", so the view, the lights, the heat figures, the
  OrgSen / Vengar vision switch and the stealth view's rule are the same for all three.
* The dispatch is the effect skills' family's three points in `play_effect_skills.rs`: `kind`
  (arms 14 and 15 call `skills::spirit_kind`), `own_type_skill` (a type of its own with skill-set 14
  or 15 gets its type's `<combat-skill>`: the Feral Colonist and Outcast as a squad member or an NPC,
  with their sounds, cost and recovery) and `give_defaults` (BF_TEST_GIVE_SKILL's cost and recovery).
* The charge's damage, knock-back, reach, cone, cost and hit effect, the regeneration and the factor
  on damage taken are all the block's. Nothing is fixed to Brutus's numbers in the code now.
* Not in the demo (as Brutus's): the kind-0x100 material controllers, the health bar's brighten
  while healing. The skill's on line h_1c0514bd is refused by the ferals' chatter libraries
  (BF_CHATTER_LOG: "refused, no block in sdm_feral_warrior_chatter" / `_outcast_`): the library has no
  block for that tag, so the ferals stay silent there as the game's gate would (the sounds play). The
  charge's line h_13601f7b goes through the same gate; its answer wasn't looked at.

**Test hooks.** `BF_TEST_GIVE_SKILL=14` / `15` (the cost and recovery of the Colonist's 2.9 /
1.05 and the Outcast's 3.3 / 1 unless given) gives the character controlled at the map's start the
class (Brutus's model, his 105 HP and 100 max stamina). The types themselves, on the test map:
`BF_CHARACTER=3 BF_TEST_SQUAD="3:feral colonist@sdm_feral"` (or `feral outcast@sdm_feral`) puts the
type in Tex's place with its own 75 HP, sounds and costs; its models load. The rest are Brutus's
hooks ([Keys and test hooks](keys-and-test-hooks.md)): `BF_TEST_SKILL`, `BF_TEST_HEALTH`,
`BF_TEST_NPC` with `BF_TEST_NPC_HP`, `BF_SKILL_LOG`, `BF_COMBAT_LOG`.

**How it was checked** (test map, `--test`, `BF_MUTE=1 BF_BACKGROUND=1`, the Brutus-model hook, then
the types):

* **Regeneration** (`BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,20 BF_TEST_GIVE_SKILL=14
  BF_TEST_HEALTH=50 BF_TEST_SKILL=1.0,9.0 BF_SKILL_LOG=1`): the Colonist, on at 1.07 s: 52.67 hp at
  1.53 s, 55.00 at 2.00 s, 90.33 at 9.07 s: +5.00 a second (Brutus +1.00). The Outcast (`15`): 50.00
  hp at every half second of the 8 s it's on (none).
* **Damage taken** (take27's test-map fight: `BF_TEST_NPC=0.5,brutus,enemy,fight,4.5,7
  BF_TEST_WEAPON=Saryl-45 BF_TEST_INSTANT_KILL=0 BF_TEST_HEALTH=5000 BF_TEST_SKILL=8
  BF_COMBAT_LOG=1`): 11 hits off, mean 5.28; on, each with " x the victim's skill": the Colonist
  x1.20 (mean 6.21, 24 hits), the Outcast x0.50 (mean 2.59), Brutus x0.85 (4.40; take27's 4.43).
  The roll and the other factors are the same hit for hit (the same seed).
* **The charge** (the test-map hooks of Brutus's: `BF_TEST_NPC=0.5,tex,enemy,frozen,4.5,14`, the
  dash from 4 s): on a 300 HP Tex, the Colonist's 100 x 1.40 (STANDARD, an enemy) = 140.0, 300 ->
  160 hp; on a 999 HP one, 999 -> 859. The Outcast's 700 x 1.40 = 980.0: 300 -> 0, "killed:
  thrown" (the body thrown as Brutus's kill is, `frames`: 69-76 of the capture), 999 -> 19. All
  three at 1.36 m, 0 degrees, the hit at 4.67 s; each takes 100 stamina with the next drain
  (stamina 0 and off at 4.73 s) and the ferals' hit sound h_e4c94dfe plays with the effect type's
  h_e6a4d091 (as an own-type Colonist: `BF_SOUND_LOG`).
* **The drain**: the Colonist 2.9 a second (98.45 -> 97.10 -> 95.55 in half seconds: 2.87-2.92),
  the Outcast 3.3 (own type: 98.24 -> 96.70 -> 94.94, 3.28-3.32); Brutus 2.0.
* **The view and the HUD**: the same grey streaked view, the heat figures and the red fade at the
  switch as Brutus's (frames 17 and 22 of an own-type Colonist and Outcast); the HUD name under the
  bars reads "Vengar" for the Colonist and "Spirit of Garner" for the Outcast.
* **Brutus's own**: his charge test (take08's hooks, 38 log lines) against the baseline build
  (968c35c): the log identical, and the 120 frames pixel for pixel.
* **Unit tests**: `play_effect_skills.rs` `the_ferals_spirits_are_dispatched` (the arms, the names, the
  blocks' values, the factor only while on, Brutus's untouched). A temporary default of 9999 / 8888
  damage showed the file's blocks override the defaults (the charge dealt 700 and 100).

**Acceptance.**

| Item | |
|---|---|
| Colonist: the view, +5 a second, 2.9 a second, "Vengar" | demo matches the code; **not filmed** (V1) |
| Colonist: hits x1.2 while on | 1.20 in `BF_COMBAT_LOG`; **not filmed** (V2) |
| Colonist: the charge 100 EXPLOSION, the knock-back, 100 stamina | as above; **not filmed** (V3) |
| Outcast: the view, no regeneration, 3.3 a second, "Spirit of Garner" | as above; **not filmed** (G1) |
| Outcast: hits x0.5 while on; the charge 700 | 0.50, 980 on an enemy; **not filmed** (G2, G3) |
| the second effect h_1e404572 | not defined in the data: nothing to draw |

**Shot list for a recording** (xemu, a dm match, the Feral Colonist unlock `h_1d604469 characters
h_086bd9cc` or the Outcast's `h_e3a313c1`): V1 / G1 on and off at 50 HP (the view, the HP slope,
the stamina slope, the name); V2 / G2 hit by a Saryl-45 at 13 m on and off; V3 / G3 a charge at a
frozen 300 / 999 HP enemy; watch char+0x490, char+0x54 and the victim's HP.
