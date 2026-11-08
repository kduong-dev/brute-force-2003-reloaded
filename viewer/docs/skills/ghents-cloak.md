*Part of [skills/](README.md).*

## Gunthar Ghent's Cloak (#205)

CSKILL_CLOAK 8, "Cloak": the special skill of the deathmatch's and the Ferguson Platoon's Gunthar Ghent
(h_1dc1aa6d of objecttypes-dm, h_e714d929 of objecttypes-sdm_ferg: stamina cost 6, recovery 1.5, 80 HP,
max-stamina 100, no start or end sound). It is Hawk's invisibility ([Hawk's Stealth](hawks-stealth.md))
without her clip, without her item costs and without her hold fire: on and off at the press, a shot shows
him for half a second and costs the gun's damage in stamina once. The body's fade and the refraction are
Hawk's (`play_stealth.rs`, switched through the same invisibility controller, `Skill::cloak`); the skill's
own part is `src/bin/play_cloak.rs` (no systems of its own: a `SkillKind`, dispatched by an arm in each of
play_effect_skills.rs's three family points, `kind`, `own_type_skill`, `give_defaults`). **No take shows it**
(`todo/npc-skills/answer.md` §6; the shots C1-C5 of the ticket were never recorded), so every number here is
the code's or the data's, none a fit to footage, and the check is the demo's own logs and frames. From
default.xbe (the rows are in [code-map/skills-and-stamina.md](../code-map/skills-and-stamina.md): FUN_0013c590,
0x13c5f0, 0x13c6a0, FUN_0013c710, FUN_0013c770, 0x13c7d0, 0x13c7e0, 0x13c580, FUN_0019e900, the vtable
0x39d540) and the analyst's map of the non-squad characters' skills (`todo/npc-skills/answer.md` §3.5).

### Who has it

| Type | Where | HP | Cost / recovery |
|---|---|---|---|
| Gunthar Ghent h_1dc1aa6d | deathmatch (objecttypes-dm.xml, the `<combat-skill>` at line 415) | 80 | 6 / 1.5 |
| Gunthar Ghent h_e714d929 | the Ferguson Platoon (sdm_ferg, line 235) | 80 | 6 / 1.5 |
| the demo's **Terrorist** (`mp_terrorist`) | the Terrorist Cell ([character-types.md](../character-types.md)) | 50 | 6 / 1.5 |

Their `<sounds start-skill end-skill>` are empty (the skill plays no sound), and each type's HUD slot is 4
(Hawk's place). The last row is **not the game's**: the product owner asked (#205, 2026-11-08) that the test
map's Terrorist class carry Gunthar's Cloak, as #407 gave the Red Hand and the Subversive the Orbital Beam and
Edward Kingman the Berserker (`Class::skill`, `SkillOverride`, `character_class::CLOAK_DEMO`: skill-set 8,
Gunthar's cost and recovery, max-stamina 100, no sounds; its `why` row says demo-own, guess). The game's
Terrorist has skill-set 0. In the demo Gunthar is a **type of its own** (the test map's squad slots, #302) and
takes his type's Cloak as the Marine takes his Orbital Beam (`effect_skills::own_type_skill`).

### The class

The constructor FUN_0013c590 (the factory FUN_0012e590's jump table, entry 7 for skill-set 8, allocates 0x30
bytes and calls it) clears the on bit and +0x1c's bits 1-5 (so the type's sounds are never positional) and copies
two values of game-options' `<combat-skills>` block `<cloak h_f726177e="2" h_1d194665="0.25"/>` (`bf/character.rs`
`CloakSettings`; the element's name hashes to h_e806e6f2, which the dispatch at 0x19f50c binds to its handler
FUN_0019e900): the **recovery rate** 2 a second (settings +0x368 -> skill+0x28; Hawk's block has 4) and the
**noise floor** 0.25 (+0x36c -> +0x2c; Hawk's 0.25 too). Its own fields: the level (skill+0x20) 0 and the factor
(+0x24) 1.0. Its vtable 0x39d540 was compared slot by slot with the base's (0x39d8f0) and Hawk's (0x39d190): it
overrides the base at

| Slot | Method | |
|---|---|---|
| +4 | 0x13c5f0 | the toggle |
| +0x4c | 0x13c580 | the name: h_fd42048b "Cloak" (string-table-en.xml line 775) |
| +0xb0 | 0x13c6a0 | a round fired |
| +0xc0 | FUN_0013c710 | the update |
| +0xe4 / +0xe8 | 0x13c7d0 / 0x13c7e0 | the visibility and the noise factor |
| +0x120 / +0x124 | 0x13c650 / 0x13c670 | on and off: **Hawk's own methods**, her kind-2 controllers switched |

and **everything else is the base's**: the drain +0xbc 0x13c690 (`stamina-cost` x dt, `SkillKind::drain`'s
default: **6 a second**), free to act +0x14 0x213690 (always), the items +0x104 0x2c60c0 (nothing: **no cost**),
hold fire +0x108 0x20e200 (no: **an AI Gunthar keeps firing while cloaked**), the overlay filter, +0x7c and the
rest. What Hawk's vtable overrides and Gunthar's doesn't: her toggle's clip and request byte (+4), her item
cost (+0x104), her hold fire (+0x108), her drain (+0xbc, x5 with the Power Blade), and the slots of her Power
Blade (+8 the special kill, +0x5c / +0x68 its holder, +0x134 its item type) and four not traced (+0x10, +0x60,
+0x64, +0x8c).

### The toggle (0x13c5f0)

No clip, no request byte, no FUN_00117210, and it returns 0 either way, so the command handler plays no overlay
and asks for no motion (`Toggle::Now { overlay: false }`):

* **Switched on**: switched **off at once** (FUN_0010e3e0), at **any level**, a fade-in under way included.
  The skill's ends are the key, the stamina update's command at 0 and death.
* **Off**: switched on (FUN_0010e470, which asks the 10% again) only if the stamina is at least **10% of the
  maximum** (0x3a4ff0, equal allowed) **and** the level (skill+0x20, as the last update read it) is **exactly
  1.0 or exactly 0.0** (a bitwise compare with 1.0, an `fcomp` with 0.0); anything else nothing at all
  (`Toggle::Refused`: no motion, sound, line or name). An off skill's level is 0 or on its way there, the
  fade-out of the second after an off, so **a press in that second is refused**; the 1.0 half of the test is
  Hawk's and can't hold for an off skill here. (The ticket's and the analyst's "a press mid-fade is refused
  until the level is exactly 1 or 0" is the off-to-on half: the code lets a press mid fade-in switch it off.
  Hawk's toggle refuses a press mid-fade in both directions.)
* The first press after a map's start fades from 0; presses are read as the other skills' (V, `BF_TEST_SKILL`,
  the Squad Command menu's Toggle Ability, [The toggle](the-toggle.md); an AI Gunthar is toggled by its goal).

### On and off, the level and the look

On and off are Hawk's 0x13c650 / 0x13c670: the character's kind-2 controllers switched (FUN_002107d0), **in
the press's own frame**: a 1.0 s fade-in and a 1.0 s fade-out (0x212380), the controller's level
(`Cloak::level`, 0x210b10) read back each frame (FUN_00210880). Switched off mid fade-in it starts its
fade-out from the top (the level jumps to 1 and comes down, 0x210a90) and the body's own controller, which
was fading out, is on at once: his body snaps back while the refraction dies (Hawk's rule, [her
page](hawks-stealth.md)). The look is Hawk's in every respect (her skin's fade, the refraction over a copy of
the picture, its wobble and its jump at a shot): `play_stealth.rs` asks `skills::cloaks(u)`, which is now the
skill's own `SkillKind::cloaks` (true for her Stealth and his Cloak) where it was her skill's name. The game
gives the controller to a model by its type's name (FUN_00210060: the mask 0x1f for h_1dc1aa6d and
h_e714d929); the demo gives every character one, so the demo's Terrorist has it too.

### The update, visibility and noise

The update FUN_0013c710 runs every frame, on or off, alive: the **level** read back from the controllers
(skill+0x20 = 0, then FUN_00210880's highest level of the kind-2 controllers: 0 when none is active); the
**visibility** pushed into them (FUN_00210950, with the last frame's factor; it scales the refraction's
offset); the **factor** climbing at the recovery rate, `+ 2 x dt` clamped 0..1 (FUN_0013c770, 0x13c770-0x13c7c2:
`maxss` 0, `minss` 1); then the base update (FUN_0010e500: the HUD's name held 0.2 s each frame it is on). The
**visibility** (+0xe4) is `1 - factor x level`: what the AI's sight (play_brain.rs `Sensor`) and the crosshair's
selection (play_reticle.rs) read, exactly as for Hawk: forgotten at 0.2 or under, not locked under 0.1 (the
rules are [Hawk's page](hawks-stealth.md)). The **noise** (+0xe8) is `floor + (1 - floor) x visibility`, `0.25 +
0.75 x visibility`: his footsteps a quarter as loud cloaked (play_liquids.rs `step`).

### A round (0x13c6a0)

For each round the fire loop takes (FUN_0022f640, the demo's `skills::fired`), **while the skill is on** (and the
character has a gun): the factor to 0, and the stamina `max(0, stamina - the gun's <Damage max>)` (the weapon
data's +0x18, `WeaponDef::damage`), through FUN_001258d0's clamp. **1x the damage, where Hawk pays 2x**
(`fadd st(0), st(0)` at 0x13d0d0), and once a round (a pellet burst counts once, as for Hawk). The factor then
climbs at 2 a second, so he is seen by the AI and the crosshair for **half a second** (Hawk's 4 a second: a
quarter), while his body stays invisible: only the refraction's offset reads the visibility, so it jumps by
`0.15` and settles over the half second (Hawk's #198 take07). A shot that empties the bar leaves the skill on
until the stamina update's command at 0 switches it off. No `+0x104`: **a grenade or a medkit costs nothing**
(Hawk's: 5, and 10 more for one placed or thrown); no `+0x108`: an AI Gunthar under the squad AI **fires while
cloaked** (Hawk's stays silent, `skills::holds_fire`).

### In the demo

* Gunthar Ghent in a squad slot: `BF_TEST_SQUAD_SLOT="0.3,me,gunthar ghent@sdm_ferg"` on any map of a test
  session (or `h_e714d929`, `h_1dc1aa6d` for the deathmatch's), `BF_TEST_SQUAD="3:gunthar ghent@sdm_ferg"` on
  the test map (he takes Tex's place and leads), `BF_TEST_SQUAD="2:gunthar ghent@sdm_ferg"` in Hawk's place
  under the AI. V or `BF_TEST_SKILL=<s>[,<s>...]` toggles it.
* `BF_TEST_GIVE_SKILL=8[,<cost>[,<recovery>]]` gives the class to the character controlled at a map's start
  (Gunthar's 6 and 1.5 unless given), for the checks on any type.
* The demo's Terrorist: `BF_CHARACTER=flint BF_TEST_SQUAD_PRESET="0.3,terrorist_cell"` (Flint's place is the
  Terrorist's seat), V or `BF_TEST_SKILL`.
* `BF_SKILL_LOG=1` prints each press and its answer (`toggle (key) ... Now`), each refusal and why ("cloak
  refused: mid-fade, level 0.533" / "stamina 9.80 under 10% of 100"), each shot ("a shot while cloaked: seen
  (the factor 0), stamina 87.80 - 4.75 -> 83.05"), when the factor is back to 1 ("after 0.47 s": counted from
  the shot's frame, the 15th frame's update at 30 Hz, the 8th at a 15 fps capture) and, every half second, his
  controller's state, level, visibility, skin and refraction as Hawk's. `BF_TEST_STEALTH_FLASH=<s>` gives a
  cloaked one a round's flash with no shot or cost, as Hawk's.

### Verified (#205)

`bf_play --test` with `BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures (30 with `BF_CAPTURE_FPS=30`; `BF_CAPTURE_EVERY`
for the log-only runs), the frames cropped and tiled; the baseline is main 3c20f54 (`bf_play_3c20f54.exe`).
Frame n of a capture is the demo's clock (n + 1) / 15 s. Nothing here is compared with a take: none exists.

* **C1, on and off at the press** (Gunthar in Tex's place, `BF_TEST_SKILL=2.0,6.0`): the press logged at 2.07
  (the first frame after 2.0), the skill on in that frame, the HUD's **"Cloak"** in the same frame (frame 30,
  full alpha to 93, then the hold of 0.15 s and the fade at 3 a second, gone at 98), the level 0.47 at +0.47 s
  and 1.00 at +1.0 s, his skin 1 - level, the refraction from the press; the stamina **-6 a second** (100.00,
  96.80, 93.60, 90.80, 87.60 at the half seconds: 0.4 a 1/15 s frame, the press's own frame included); off at
  6.13, the level 0.60 at +0.40 s and 0.07 at +0.93 s, the body whole again at +1.0 s, the refill **+1.5 a
  second** (76.30 at 6.53, 77.10 at 7.07, 77.80 at 7.53). The frames (the whole figure, the HUD's name beside
  the bars) show the body fading into the refraction and back as Hawk's does.
* **C3, the presses mid-fade** (`BF_TEST_SKILL=2.0,2.4,2.9,3.6,6.0,6.4,7.2`): on at 2.07; at **2.47, mid
  fade-in** (level 0.4) off at once, his skin back whole in the same frame and the refraction fading out from
  the top; at 3.00 **refused** ("mid-fade, level 0.533"; the fade-out ran from 2.47 to 3.47); on at 3.73 (the
  level 0 since 3.53); off at 6.13; **refused** at 6.53 (level 0.667); on at 7.33 (0 since 7.13).
* **Under 10%** (`BF_TEST_STAMINA=9@1.0,10@3.0,3@4.0 BF_TEST_SKILL=1.5,3.0,6.0`): refused at 9.80, on at **10.00**
  (equal allowed), off by itself at 4.60 with the stamina at 0 ("toggle (at 0)", the chatter's line at 0,
  ee0a5cbc), refused again at 2.30.
* **C2, a shot** (`BF_TEST_TRIGGER=4.0-4.3,7.0-7.05`, 30 fps; his gun is the MK-ASLT, Damage max 4.75): "a shot
  while cloaked ... stamina 87.80 - 4.75 -> 83.05" and the next round 82.25 - 4.75 -> 77.50, **once a round**
  (no other drain: 4.03 88.00 to 4.53 75.50 is 3.0 of drain and the two rounds' 9.5); the **factor back to 1
  after 0.47 s** (the 15th update at 30 Hz; 0.50 s counted from the flash hook), visibility 0.33 at +0.3 s; a
  flash by the hook: 0.50 s. Hawk's shot costs 2x (14.50 for her 7.25 gun, the baseline's the same).
* **C4, items** (`BF_TEST_SQUAD="3:gunthar ghent@sdm_ferg" BF_TEST_HEALTH=40 BF_TEST_INV=Medkit:3
  BF_TEST_MEDKIT=4.0 BF_TEST_THROW=6.0,0.4`): the medkit used at 4.07 (40 -> 80 HP) and the Frag released at 7.13
  with **no stamina taken** (the half-second lines are pure drain: 87.60 -> 84.80 and 69.60 -> 66.80, 2.8 each,
  seven frames of 6 a second). Hawk's, the same recipe: the medkit "an item used while stealthed (use type 0):
  stamina 94.50 - 5 -> 89.50" and the Frag "(use type 3): 83.50 - 15 -> 68.50".
* **The enemies** (`BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,20 BF_TEST_NPC="0.5,shrike soldier@e40,enemy,fight,4.5,4.3"`
  `BF_SIGHT_LOG=1`, 30 fps; the Shrike's vision-range 75): it sees him at 15.7 m from 0.53 and hits him; the press
  at 6.07, its last hit at 6.70, "sees no hostile" at **6.90, 0.83 s into his fade** (visibility 0.17, the sensor's
  0.2: Hawk's take14 0.81-0.86 s); his shot at 9.07 shows him again ("sees unit 0 at 15.7 m, its visibility
  1.00"), the second round at 9.20, the factor back at 9.67.
* **The AI keeps firing** (`BF_TEST_SQUAD="2:gunthar ghent@sdm_ferg" BF_TEST_SQUAD_SKILL=2,3.0`, the same Shrike):
  the Squad Command order's toggle at 3.03 puts him on at 3.07; he goes on firing at it from 4.67, cloaked
  (11 rounds in the log by 6.5 s, 15 by the capture's end, 4.75 each), and the Shrike hits him in return.
* **The Terrorist** (`BF_CHARACTER=flint BF_TEST_SQUAD_PRESET="0.3,terrorist_cell" BF_TEST_SKILL=2.5,7.0
  BF_TEST_TRIGGER=5.0-5.2`): Flint's place is the Terrorist (50 hp); V at 2.60: on, the fade as above (frames:
  the black-armoured Terrorist fading into the refraction while the Red Hand, Subversive and Kingman stand
  seen), a round at 5.13 costs 8.00 (his Confed LZR-23's Damage max); `BF_TOOLS_LOG`'s class report gives the
  Terrorist a "skill: guess (Gunthar Ghent's Cloak ...)" row. A test map NPC Terrorist and Gunthar standing
  (`BF_TEST_NPC`, no skill step runs for them) are drawn exactly as on main: 75 frames pixel for pixel.
* **`BF_TEST_GIVE_SKILL=8,8,1`** on Tex: "has skill-set 8 (cost 8, recovery 1)", on at 1.60, -8 a second, +1 a
  second after the off at 4.13.
* **Hawk's Stealth is unchanged** against main 3c20f54 (`BF_CHARACTER=2 BF_TEST_SKILL=2.0,8.5 BF_TEST_GOTO=0,8,0,8
  BF_TEST_TRIGGER=5.0-5.4 BF_TEST_STEALTH_FLASH=7.0`, 150 frames): the logs identical but for the load-time line
  and the 150 frames **pixel for pixel**. With a medkit and a Frag thrown (`BF_TEST_MEDKIT=5.0
  BF_TEST_THROW=7.0,0.4`) her costs are the same (5 and 15) and the log lines identical but for four
  run-to-run ones (the load time, a hash order, two entity ids); the frames from the medkit's fall on differ as
  the baseline's own two runs do: **68 of the 210 differ in the new build against main, 68 in main against
  itself**, the same frames (79-86 and 116-175), worst 147 of 255 on at most 300 pixels, the rest 53 or less
  (debris and blast particles).
* **The other skills are unchanged** (`BF_CHARACTER=<tex|brutus|flint> BF_TEST_SKILL=1.5,5.0`, the Confed
  Marine's Orbital Beam and the Militia's Air Strike, new against main): the logs identical but for the load-time
  lines; the frames differ only where a second run of either build does (particle and beam-light noise of at most
  7 of 255, brutus' two frames of the red fade, 24 and 25, which two runs of main differ in just the same).
* Unit tests (`cargo test`: 108 in the library, 365 in bf_play, all passed; nine of them new): the factory and its dispatch (`cloak::tests`,
  `effect_skills::tests::the_cloak_is_dispatched`), the toggle's gates (9.99 refused, 10.0 on, the fade-out's
  levels refused, a settled one allowed, off at any level), the fades (1.0 s in and out), a round (4.75, once,
  the factor 0 -> 1 in 15 frames at 30 Hz, Hawk's 2x for the same gun), items free and no hold fire against
  Hawk's 5 and her hold fire, the drain 6 a second, the `<cloak>` block (`bf/character.rs`) and the Terrorist's
  override (`character_class.rs`).

### Open and not done

* **Not filmed**: C1-C5 of the ticket (a dm match with Gunthar Ghent, the unlock `h_1d604469 characters
  h_1dc1aa6d`): the 1.0 s fade from the press, the 6 a second, the shot's cost, the factor's half second, the
  refusals and the items are the code's, matched by the demo's own logs; nothing is compared with the game's
  picture. Not shot: C5, an enemy's reaction at 15 m (the demo's sensor is Hawk's, verified against her take14).
* **The mid-fade rule** (above): the ticket's wording had every press mid-fade refused; the code refuses only
  the off-to-on press. If a take shows the other, the toggle's first lines are the place.
* **The Terrorist's controller**: the game's FUN_00210060 gives the kind-2 controller to a model by its type's
  name, which a demo class has none of; the demo gives every character one, so the demo's Terrorist fades and
  refracts as Gunthar does. The choice, and the class's skill, are the product owner's.
* **Not in** (as Hawk's page): the radar's RCONTACT_INVISIBLE, the AI's hearing his noise (the demo's AI
  doesn't hear), the shadow's own fade, the chatter lines the type's library has no block for.
* **What the enemies' goals do** when they lose him: the demo's AI has no goals, so an enemy stands and fires
  again when it sees him (#123's "not in").
