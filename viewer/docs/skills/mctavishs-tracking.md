*Part of [skills/](README.md).*

## McTavish's Tracking (#207)

CSKILL_TRACKING 12, "Tracking" (h_0c2de503, string-table-en.xml line 777): the special skill of the
deathmatch's McTavish (h_1e04e3d4 of objecttypes-dm: stamina cost 10, recovery 1.1, 80 HP, mesh
h_ed7bb328, motion set `ms_mctavish` = the militia's) and the Ferguson Platoon's (h_fac6fa93). It is
**Flint's Advanced Targeting** ([her page](flints-advanced-targeting.md)) under another name with another
block of values. **No take shows it** (the ticket's shots T1-T3 were never recorded), so every number is
the code's or the data's and the check is the demo's own logs and frames.

### The class

The factory's arm for skill-set 12 builds the skill with FUN_0013f0e0: it runs **Flint's constructor
FUN_0013c9e0**, sets the vtable 0x39c4b0, then copies the `<tracking>` block of game-options' `<combat-skills>`
(game-options-en.xml line 188; the element's name hashes to `h("tracking")`, bound at the dispatch to the
handler FUN_0019f120, settings +0x47c..+0x4a0) over the same fields of the skill:

| Field | Attribute | Settings | Flint's block (h_0725cdb1) | McTavish's `<tracking>` |
|---|---|---|---|---|
| +0x44 | tracking-weapon-type h_108b56f9 | +0x47c | 1 (rifle) | 1 |
| +0x24 | h_108f0925, the lock box across | +0x484 | 0.4 | **0.6** |
| +0x28 | h_081520f0, the lock box down | +0x480 | 0.4 | **0.3** |
| +0x2c | h_18bf34d4, the double chance | +0x488 | 0.15 | **0.25** |
| +0x30 | h_fe103e1b, the double factor | +0x48c | 2 | 2 |
| +0x38 | h_1b947b90, reticule speed | +0x490 | 2 | 2 |
| +0x34 | h_191dafb6, the scoped factor | +0x498 | 1.5 | **0.8** |
| +0x3c | h_ec0b2194, the range's weapon class | +0x49c | 1 | 1 |
| +0x40 | h_f82a974c, the range factor | +0x4a0 | 1.25 | 1.25 |

(The ticket and the analyst's notes give "shares 0.3 / 0.6" in the order of the hashes, h_081520f0 then
h_108f0925: the **0.6 is the width and the 0.3 the height**, the handler's offsets above, +0x24 being the
across share as in Flint's.) The marker widget name h_f192e10e (h_121cf7bb) is Flint's too. The 0.8 is a
**penalty**: a scoped roll is x0.8, where Flint's is x1.5.

The vtable 0x39c4b0 was compared slot by slot with Flint's 0x39d2c8: the only difference is the name,
+0x4c 0x13f0d0 (h_0c2de503) in place of 0x13c9a0 (h_19e636bd "Targeting"); the toggle 0x13cad0, update
0x13cbf0, roll 0x13cc70, range 0x13ce80, move cap 0x13ceb0, lock shares 0x13cd30, crosshair alpha 0x13cce0,
lock box 0x13cd10, speed 0x13c9b0, marker 0x13c9c0 and the scope slots (0x13c910, 0x13c930, 0x13c990) are
the same functions. So everything on [Flint's page](flints-advanced-targeting.md) holds for him with
these values: the toggle's refusals, 10 stamina a second (the type's cost), the walk cap 0.55 of the stick,
the gun kept raised, the not-free-to-act rule, the doubled rolls (a quarter of them while on), the rifle's
range x1.25 and the marker, whose wide box is (0.6 x 640, 0.3 x 480) round the crosshair.

### In the demo

* `play_skills.rs`: `Targeting` carries its `Tracking` block and its name (`name`); `tracking_kind`
  builds McTavish's from `CombatSkills::mctavish_tracking` with `NAME_TRACKING`. Flint's keeps
  `CombatSkills::tracking` and "Targeting". `SkillKind::marker_shares` (new, Flint's and his: the skill's
  +0x24 and +0x28) gives `play_targeting.rs` the marker's wide box, which read Flint's block only.
* `bf/character.rs`: `Tracking::read_block` (Flint's handler's reading, shared) and `Tracking::mctavish`
  (the shipped values); the `<tracking>` block is read over them.
* `play_effect_skills.rs`: one arm in `kind`, `own_type_skill` and `give_defaults` (10 / 1.1), as the
  Cloak's.
* The motion set `ms_mctavish` has no `Sc_w1_sniper_idle`, so `SpecialClips` finds no sniper set and
  the rifle mark (his MK-ASLT is weapon class 1) changes nothing on screen: no sniper pose, the militia's
  own walk (the ticket's "no sniper pose"). A type of its own takes the skill from its type
  (`own_type_skill`): `BF_TEST_SQUAD_SLOT="0.3,me,mctavish@sdm_ferg"` on any map of a test session, or
  `BF_TEST_GIVE_SKILL=12[,<cost>[,<recovery>]]` on the starting character (10 and 1.1 unless given).
* The HUD name "Tracking" shows as every skill's does (`Skill::name`).

### How it was checked

Against main 054473a's build (`bf_play_054473a.exe`), `BF_MAP=e34 -- --test`, an L-Shot-75 / Jax-iP loadout,
a 999-HP Shrike Heavy 6.7 m ahead (`BF_TEST_NPC`, `BF_TEST_NPC_HP`), `BF_SKILL_LOG`,
`BF_TARGETING_LOG`, `BF_COMBAT_LOG`, 15 fps captures:

* **Flint is unchanged**: `BF_CHARACTER=flint BF_TEST_SKILL=2,9 BF_TEST_TRIGGER=4-8
  BF_TEST_INSTANT_KILL=0 BF_NO_SQUAD=1`, 150 frames: the logs (45 lines: the toggle, the marker's corners,
  the stamina, the hits 40.6, 19.3, 42.7, 21.9) are **identical** to main's, and the frames match (main's
  own two runs differ by 26 frames, all after the first shot, from the particle effects; the new build's 29,
  the same frames).
* **The name**: `BF_MAP=flat BF_TEST_GIVE_SKILL=12 BF_TEST_SKILL=1.5,4` (a normal run with the HUD): "Tracking"
  under the bars while on.
* **The box**: no enemy, on at 2.07: the corners slide to (128, 120) and (512, 264) HUD units by 2.87 s,
  384 x 144 = 0.6 x 0.3 of 640 x 480 round the crosshair at y 192 (Flint's: (192, 96)-(448, 288)); a Shrike
  in it is locked at once (the tight box closes on it as Flint's does, frames at 2.07-2.67 s).
* **The drain**: 100 to 49.33 in 5.0 s (10 a second), then back up at about 1.1 a second after the off (the type's recovery,
  49.33 at 7.07, 50.43 at 8.07).
* **The walk**: `BF_MOVE_LOG=1`, the autopilot's run (gait Run, clip 58) becomes gait Walk (clip 57) in the
  frame after the press, no sniper clip.
* **The double**: `BF_COMBAT_LOG`, many hits on a 999-HP Shrike over several weapons: the doubled rolls
  (" x the shooter's skill 2.00", the damage x2) were 15 of 40, 13 of 38, 12 of 36, 10 of 39 ... (the demo's
  rolls share one xorshift stream per character, so the runs repeat each other: not independent samples).
  Pooled, 21-31 % against the 25 % set, with Flint's the same way at 15 % (7 of 47 on the same weapon and
  start, where his block gives 0.15): the threshold is the block's, the count is small.
* **The scoped factor**: `BF_TEST_AIM=1`, standing, skill on, an L-Shot hit on a calf: " x the shooter's
  skill 0.80" (29.01 x 0.80 x 0.5 x 1.40 = 16.25).
* **The range**: x1.25 is Flint's code (`SkillKind::range` over `Tracking::range_factor` 1.25 and the weapon
  class 1), not run separately here.

**Not filmed (acceptance, blocked on shots T1-T3):** the marker's box and the walk against take footage,
the 10 a second, the double rate over twenty hits and the scoped factor; the numbers above are the game's
code and data, not a match to a take.
