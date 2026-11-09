*Part of [skills/](README.md), the former page `skills.md`.*

## The Supermutant's ground pound (#212)

The two e23 Supermutants (`h_ece04235`, `h_1ffe2615`; 669 HP, CSKILL_BOSS_SUPERMUTANT **17**, stamina
cost 1 and recovery 1.5 in the data; the demo's class pays 6, below) pound the ground when the target is too close, and the demo's **Supermutant
class** (`mp_supermutant`) has the skill: `src/bin/play_ground_pound.rs`, one of the effect skills'
family ([other characters' skills](other-characters-skills.md): the spawner, the target point and the
effect's damage areas are #200's and #373's). Read from default.xbe (0x13b940, the boss base
0x13b770, the toggle 0x13b830, the animation-event handler 0x13b800, the own-point target 0x13b7c0;
the vtable 0x39c730; [code-map](../code-map/skills-and-stamina.md)), the analyst's answers
(`todo/npc-skills/answer.md` §3.1, §5) and the data (`game-options-en.xml` lines 169-171,
`characters/ms_mutant.xml` line 629, e23's `objecttypes-e23.xml` line 1742). **Not filmed**: the
ticket's S1/S2 takes (walk up to a Supermutant, watch the squad's HP at 3 m and 6 m; stay at 10 m)
weren't made, so the timings and numbers below are the code's and the data's, not footage's.

### The class and the toggle

* **The class** (FUN_0013b940, constructor of the boss base FUN_0013b770): the game-options block
  `<boss-supermutant animation-duration="3"><effect-data h_1fbc7e1e="1" h_17d82c44="h_ece33110"
  h_f6bfb82f="0" orientation-pitch="0" h_05215e19="false" h_f71f4e3f="1" h_e272cd2c="0"/></boss-supermutant>`
  (`bf/character.rs` `BossSupermutantSkill`): the effect and the animation-duration, which the
  toggle's stamina gate multiplies the cost by. The class's vtable +0x4c names no string the English
  table has (there is no "Ground Pound" entry), so the HUD's caption is the demo's own,
  **Ground Pound** (`play_ground_pound.rs`'s `NAME`, put in `Game::strings` by `name_skill` when a
  character has the skill, as the corpse eater's *Feast* is): the HUD draws a skill's name only from
  the table, so without it a press would show nothing.
* **The toggle** (0x13b830, the boss base's): refused while the skill is on, with a motion request
  pending, without a current motion or an MT 45 transition in it, on the overlay-slot states, or with
  stamina under the animation-duration × the cost (3 × 6 = 18; equal passes). Else it switches the
  skill on itself (FUN_0010e470: the 10% of the maximum, the type's start sound, the on line) and
  returns 2, MT_TOGGLE_SPECIAL: the motion set's MT 45 motion (`Toggle::Motion`, `start_pose`). With
  the shipped Supermutant's `h_e78efbb2` false its stamina never drains in the game, so the gate
  never bites there; the demo turns the update on for the class (`Skill::new`, as for the Molotov
  Mutant and the corpse eater) and pays 6 a second (`character_class::GROUND_POUND_DEMO`, the product
  owner's choice of 2026-11-09, #212): a press needs 18, the clip takes about 12 of the 100, and the
  1.5 a second recovery refills that in about 8 s. The campaign type's 1 a second took about 2,
  which the recovery undid at once, so the bar never showed the pound.
* **The pose**: ms_mutant's MOTION h_eb4a1ed4 (`motion-type="45"`, `script="ground_pound"`,
  `prefix-type="1"`, `transition="0.1"`, `block-overlay="true"`, `h_f6a410e6="true"`) →
  **`Sc_w1_ground_pound`, 1.97 s**, no root motion. A whole-body motion: the character is held where
  he stands (`holds_still`), takes no crouch or jump from him, and the aim doesn't turn the gun over
  it. The pose blends in and out over the MOTION's 0.1 s.
* **The end**: the clip's **ead033a3** at 1.97 s switches the skill off, as the game's event handler
  does; nothing in the class otherwise ends it. The demo's update asks for the command at the clip's
  end and the toggle, on with the clip over, answers the off. A knock-down cuts the clip short
  (`play_skills.rs` `step` drops the pose, as for every pose), and the skill goes off with it rather
  than waiting for an event that will never come.

### The burst

At the clip's **04592c53**, 0.97 s, the handler calls the class's +0x138 (0x50b40: nothing for the
Supermutant) and the spawner FUN_0013e410(0). The target point is the boss base's own (+0x134,
0x13b7c0: the character's root node, `SkillKind::own_point`), so the effect starts where he stands —
count 1, no scatter, no pitch, reach 1.

The effect **h_ece33110** (e23's `objecttypes-e23.xml` line 1742; built by #373, and the same type
`BF_TEST_EFFECT=…,h_ece33110,…` starts): ALE h_18058148, light h_ef1056ea, sound h_1ea097fa (2.5 s),
**50 POWERBLADE (11) a second for 1 s within 7.5 m**, no falloff, push h_ed582b3c 2.5, and a camera
shake from 1 s for 2 s within 10 m (falloff 2, h_f944a10f 0.7, so 0.49 at the pound at 1 s). The
areas hurt everyone in range at each one's POWERBLADE factor (play_damage.rs), their hits his (the
effect's owner); the shake rides the controlled character's camera (play_shake.rs). So at 3 m a
squad member takes 50 over the second, and at 10 m nothing at all (the area is 7.5 m; the shake
reaches 10 m). **The owner is spared** (demo-own, `SkillKind::spares_owner`): the game's own
Supermutant stands in its blast and takes 50 × its type's 0.059 POWERBLADE factor, ~3 HP of its
669, while the demo's class has no vulnerabilities by the class rule and would take 50 of its 95 —
half its health a press. Every other effect still hurts its owner (the Orbital Beam's and the
Wrath's takes show it).

### Keys and test hooks

* The demo's class as the controlled character: `--test BF_TEST_SQUAD_SLOT="0.3,me,mp_supermutant"`;
  the campaign type on any map of a test session the same way (`0.3,me,supermutant@e23`). V or
  `BF_TEST_SKILL=<s>` pounds. `BF_TEST_GIVE_SKILL=17` gives the skill (cost 1, recovery 1.5) to the
  character controlled at a map's start, so a squad member pounds on any map.
* `BF_SKILL_LOG=1`: the press, its refusals, the MT 45 motion and, at 0.97 s, the burst and the
  effect's place; `BF_DAMAGE_LOG=1`: the effect's damage ticks; `BF_SHAKE_LOG=1`: the shake held on
  the view; `BF_COMBAT_LOG=1`: the hits.
* Against NPCs at distances: `BF_TEST_NPC="0.5,mp_tex,enemy,frozen,0,-3;0.5,mp_tex,enemy,frozen,0,-10"`
  with `BF_TEST_INSTANT_KILL=0 BF_DIFFICULTY=0`.

### What's not built

* **The AI's own use**: the personality h_08cf55a2 maps EVT_TGT_TOO_CLOSE (85, priority 20.17) to
  GOAL_ATTACK_SM_GROUNDSHAKE 116 (weight 1; 107 at 0), whose start 0x37440 sends CT_TOGGLE_SPECIAL
  when the skill is off and succeeds if it is then on, and whose run 0x374b0 is busy while on and
  done once off. What posts the event is the AI's sensor code (#26), not read. The demo's squad-order
  goal refuses an attack skill (play_orders.rs), so 116 can't be ordered either.
* The gutterball (GOAL 115) and the Supermutant's model, clips and death (#134).

### Checked

* Unit tests (`cargo test`): the toggle's gate at 3 (2.99 refused, 3 passes, no burst at the press),
  the clip's events (the burst at 0.97 s once, the off asked for at 1.97 s, a fresh clip after it
  firing again), the effect record and the own point, the knock-down's cut, the caption's id, and the
  three dispatch points (a type with skill-set 17 gets the class; BF_TEST_GIVE_SKILL=17 defaults to
  1 / 1.5).
* A live test-map run (`--test` with `BF_TEST_SQUAD_SLOT="0.3,me,mp_supermutant"`, frozen `mp_tex`
  NPCs at 3 m and 10 m, `BF_DIFFICULTY=0`, `BF_TEST_SKILL=2.0`, `BF_SKILL_LOG=1 BF_DAMAGE_LOG=1
  BF_SHAKE_LOG=1`): the press at 2.01 (the bar 100, the gate 18) starts `special motion
  (MT_TOGGLE_SPECIAL), 1.97 s` plays; the burst at 2.97 is at his root node (0.00, -0.12, 0.00, `Own`)
  and starts h_ece33110 with `1 damage areas ... (50.0, 11, 1.0, 7.5, 0), 1 shakes ... (1.0, 2.0,
  10.0, 2, 0.7, 0.0)`; the 3.01 m Tex goes 70 -> 20.0 over the second (50 POWERBLADE, falloff 0,
  type 11 x1.000) and the owner 95 -> 95.0 (spared), while the 10 m NPC takes no tick at all; the shake holds
  0.490 at 3.97 (`BF_SHAKE_LOG`: `held 0.490 by h_ece33110 (age 1.00 s, 0.00 m)`) and falls to 0.007
  by 5.73; the skill is off at 3.97 (its end) with the bar at 88.18 (6 a second over the 1.97 s clip; a
  15 fps capture run of the same press: 88.00 at 4.00 s, 100 at the press), and refills at 1.5 a
  second, the 12 a pound back in about 8 s.
* A `BF_CAPTURE` run of the same press (15 fps frames, `BF_TEST_SQUAD_SLOT="0.3,me,mp_supermutant"`,
  `BF_TEST_SKILL=2.0`): the stamina bar's message line is empty at 1.0 s, before the press; at
  3.0 s, with the skill on, it shows **Ground Pound** at full in the HUD's blue (the left-aligned line
  under the health and stamina bars); by 4.4 s it is gone (the off at 3.97 and the label's brief
  hold). The bar's own fill drops with the pound: full at 1.0 s, about an eighth shorter at 3.0 s,
  shorter still at 4.2 s (88 of 100), where before the change the whole pound moved it 2 points.
