*Part of [skills/](README.md), the former page `skills.md`.*

## Stamina

Each character type's data (`common/objecttypes-common.xml`, `Game::character_skills`):
`max-stamina` 100 for all four, and `<combat-skill skill-set stamina-cost stamina-recovery>`
(`stamina-cost` and `stamina-recovery` are hashes 105c13aa / f2a2beb9 in the file):

| | Tex | Brutus | Hawk | Flint |
|---|---|---|---|---|
| drain while on, per s | 2.2 | 2.0 | 2.5 | 3.2 |
| refill while off, per s | 1.8 | 1.2 | 0.6 | 0.8 |
| on squad deathmatch maps (`sdm_*`) | 6.5 / 1.2 | 2.5 / 1.1 | 4 / 1.1 | 6.5 / 0.8 |

Squad deathmatch plays its own Brute Force squad, the library `sdm_bf` (`game-options-en.xml`
line 84; `mp_common/objecttypes-sdm_bf.xml`), and take21 measured its rate on sdm_e34 (Tex
-6.55/s). So on an `sdm_*` map `load_map` puts that squad's whole `<combat-skill>` block in place
of the campaign's (`Game::load_squad_library`): its rates and sounds (Flint's end sound there is
Hawk's f57b8dca). The rest of that squad (hitpoints, the `sdm_*_chatter` sets) stays the
campaign's in the demo. A type of its own seated on the test map takes its skill from its own
`<combat-skill>` by the set (#412): `brutus@sdm_bf` runs the squad's Spirit of Vengar over
sdm_bf's 2.5 / 1.1, `hawk@sdm_bf` Stealth over 4 / 1.1 and `flint@sdm_bf` Advanced Targeting over
6.5 / 0.8, the deathmatch library's types the same with their own blocks.

Each frame, for every squad member with a skill, the controlled one and the AI alike
(FUN_0011a840, run from the character update; types with the flag h_e78efbb2, all four):

* **On**: stamina drops by the skill's drain (the base: `stamina-cost` × dt). At 0 the update
  sends the toggle command itself (CT_TOGGLE_SPECIAL, the same as the key), every frame while the
  skill stays on.
* **Off** (checked again after that, so a skill that went off this frame refills this frame; no
  delay before the refill): stamina rises by `stamina-recovery` × dt × a factor by what the
  character is doing (the motion state): ×0.25 dashing, the demo's sprint (Shift, and the AI's
  sprint to catch up); ×2 crouched still (kneeling, `Action::Crouched` not moving); ×1 otherwise,
  the crouch walk, the kneel's way down and up (MT_TRANSITION, measured ×1) and a knock-down
  (MT_KNOCKDOWN, a sprint's too) included. A ×4
  while a byte at char+0x5c is set is left out: nothing in the takes set it.
* Clamped to 0..max on every write. Full at spawn (and a respawn). The dead don't change (take30:
  Brutus's 94.11 stays for the 9.4 s after his death).
