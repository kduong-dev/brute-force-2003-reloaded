*Part of [skills/](README.md).*

## The Molotov Mutant's Self-Destruct (#211)

CSKILL_MOLOTOV_MUTANT 20: the skill of the campaign's Molotov Mutants (e01's h_fc30a700, e23's h_f0c5e50e and
m03_a's and m03_b's h_10a12c64: stamina cost 3, recovery 1.5, 22 - 55 HP). In the game it is **AI only**: the
mutant carries a lit bottle in its left hand, runs at the first enemy it sees, and within 5 m plays a lunge in
which it lets the bottle go, takes 500 FLAME itself and bursts into a cloud that hurts everyone round it. No
character a player controls has it. The demo gives it to a player who controls one (the test map's Molotov
Mutant, the Mutant Horde's, `mp_mutant_molotov`), by **the product owner's design of 2026-11-08, demo-own**: the
bottle is lit and held until an enemy comes near, and can be put away. This page is the class (`src/bin/
play_molotov_mutant.rs`, one of the effect skills: the spawner, the target point and the damage areas are
[#200's](other-characters-skills.md), and the class has an arm in each of its three dispatch points), what
the game does, what the demo does, and how it was checked. From default.xbe (the rows are in
[code-map/skills-and-stamina.md](../code-map/skills-and-stamina.md): FUN_0013db80, FUN_0013dce0,
FUN_0013dc90, 0x13b830, 0x13b800, 0x13b7c0, 0x13dbc0, 0x13dc10, the vtable 0x39cd68 and
GOAL_ATTACK_MOLOTOV's run FUN_000370b0), the analyst's answers (`todo/npc-skills/answer.md` §3.1,
`todo/132-mutants/leads.md` §4) and the footage of `todo/132-mutants` (take02 M1, take09 and take10 T2,
take11 M2: the AI's own run, the lunge and the burst).

### What the game does

* **The class** (vtable 0x39cd68, constructor FUN_0013db80): the boss base's (FUN_0013b770, also the
  Supermutant's and the Shadoon's) with the `<molotov-mutant>` block of game-options (`MolotovSkill`,
  `bf/character.rs`): its `<effect-data>` (one h_1a256542, no scatter, no pitch, reach 1: FUN_0013de50's copy),
  `animation-duration` 3 (skill+0x3c) and the bottle's item type h_e8eb77f8 and hardpoint h_f8474385 (the
  support hand: `weapon::SUPPORT_HAND`, the game's "left hand"). Its skill item holder (the inventory's +0x1ac:
  +0x68 0x13dbc0 makes it at that hardpoint, +0x5c 0x13dc10 fills it) is filled when the character is made:
  **the bottle is always in the mutant's left hand, lit** (take09's idle: the flame at his belly, beside the
  torso). Its name slot (+0x4c, 0x2bf0a0) returns 0: the game's skill has no name.
* **The toggle** (0x13b830, CT_TOGGLE_SPECIAL, which GOAL_ATTACK_MOLOTOV, 114, sends from FUN_000370b0 once the
  squared distance between its mutant's and its target's root nodes (`vtable +0x40`'s record, the hips) is
  under 25: **5 m**, 3D): refused while on, with a motion request pending, with no current MOTION or none with a
  motion type 45 transition (ms_mutant2's idles, walk, run, side steps, back walk, crouch idle and dashes
  have one; the crouch's transitions, the throw, the dodges, the dive, the stun, the knock-down and the jumps
  don't), with both overlay slots busy under a MOTION that blocks overlays, and with a stamina under the
  cost x `animation-duration`, 3 x 3 = 9 (equal passes: the same `fcompp` / `test ah, 0x41` / `jp` as the
  Wrath's, 0x13b8b3). Then FUN_0010e470 (on) and **return 2**: the MOTION h_fe5d7332 `molotov`
  (`Sc_w1_molotov`, 0.73 s, `transition="0.1"`, `h_f6a410e6` true: the stick's move and turn are skipped).
  The stamina of a type with no stamina update (`h_e78efbb2` false: all of e01's, e23's and m03's mutants) stays
  100, so the 9 never bites in the game.
* **The clip's events** (the boss base's handler 0x13b800): **04592c53 at 0.70 s** calls the class's +0x138
  (FUN_0013dce0: the holder's bottle is let go, holder +0x38, and FUN_00223160(500.0, DTYPE_FLAME 5) with no
  attacker: **500 FLAME to the mutant itself**, through its damage handler 0x125600) and then the spawner
  FUN_0013e410(0), at the boss base's target point (+0x134 0x13b7c0: **the character's own root node**), the
  cloud h_1a256542: ALE h_e49f4396, light h_ef1056ea, sound h_02bcfca8 at 0.2 s and **78 FLAME (5) a second for
  1 s within 7 m, linear falloff**, push 1, which hurts every character in range, the mutant's side too
  ([effects.md](../effects.md)); ead033a3 at 0.73 s, the generic toggle event, switches the skill off. (19f8311b
  at 0.57 s is the generic release event: nothing here.)
* **Its death** (+0x70 FUN_0013dc90, from the ragdoll build FUN_00127290 and the vaporise FUN_001252b0): a
  bottle still in the holder is let go and the spawner called, so a mutant shot dead on its run bursts where it
  falls (take11: the kill's score shows at 5.60 s, the fireball at 5.67, nothing before: it is the death's).
  One killed after the event, by its own burst, holds none. (`FUN_0013e410`'s argument there is char+0xdc's
  object when that is a character, else 0: what char+0xdc holds wasn't traced, so the cloud's owner here is
  the mutant.)
* **The footage's picture and numbers** (todo/132-mutants take02, 09, 10): the lunge **leaps the mutant's
  picture 2.4 - 3 m forward** (the clip's root travel, 2.37 m: take02's mutant ends beside Tex's shoulder)
  **but not the character's node**, which keeps its place: the cloud is centred 4.2 m from Tex and 4.7 m
  from Brutus, and they lost 18.29 and 16.21 HP, where 78 x (1 - d / 7) x 0.6 (the squad's STANDARD step) is
  18.6 and 15.3: **one cloud and nothing more**. take09 (Hawk 4.5 m -16, Tex 6.0 m -7, Brutus 6.5 m -4) and
  take10 agree. The bottle that falls loose (holder +0x38) would go off at its first contact by the grenade
  rules (fuse 0, h_13fe011e: 13 - 26 FLAME within 7 m and the same cloud again: the analyst's reading, not
  traced); no step of that size is in the takes' HP (a blast of 26 at 4.2 m would add 6.2 to Tex's 18.3), so
  the demo doesn't make one.

### The player's design (demo-own)

The product owner's request, 2026-11-08 (#211's comment): what the game's AI version does at the press the
player makes in two steps, the second by the enemy.

* **V** (or `BF_TEST_SKILL`): the game's gates of the toggle (a motion with a type 45 transition, no overlay
  under way, a stamina of 9) and the shared switch on (10% of the maximum, the line asked of the chatter gate):
  **he lights the bottle and holds it in his left hand** (`Bottle`: the throwable Molotov's model, h_0157c126,
  and the attached flame h_f61f99b7 on it, the sound of the flame played once as it is lit), the skill on,
  the HUD names it ("Self-Destruct": the demo's pick, see below), and **the stamina drains at the class's
  cost, 3 a second** (the base's `stamina-cost` x dt: 33 s from full), no refill meanwhile.
* **Holding**: he moves, jumps, dodges and cancels, and does nothing else: **no fire and no aim** (the
  controlled character's `fire` and `aim` are cleared before his step, `hands_full`, after every control and
  test hook that sets them), and he isn't free to act (`lets_act`: **the demo's choice** for the product
  owner's "bottle only", not the game's rule: the class's vtable +0x14 is 0x213690, free to act always, and the
  demo gives the bottle the Wrath's rule, 0x13b980, free only while the skill is off): **no reload, weapon switch, grenade, medkit, weapon pickup or console, nor crouch key** (the
  key is gated by the same test, play.rs `read_input`; a crouch he is already in stands). A squad member the
  AI plays while he holds the bottle holds fire too (`SkillKind::holds_fire`).
* **V again** (or the stamina's 0, the same command the stamina update sends every frame it stays on): the
  skill's toggle while it holds the bottle is the **off**: the bottle is put away (the flame goes out with its
  model), **no refund** of the stamina spent, the refill at the recovery, 1.5 a second. At 0 it is a **fizzle**:
  the same put away, a line in the log, **no burst**.
* **A hostile within 5 m** (`watch_foes`: any living character on a side hostile to his
  (`damage::hostile`), its root node (`missile::root_point`) under 5 m of his by the squared 3D distance, as
  FUN_000370b0 compares; the nearest is the one he turns to): the **lunge**. `update` asks the command for it
  (`Skill::foe`), and the toggle, with the skill on already, answers `Toggle::Pose`: the same return 2 as the
  game's, without a second switch on, sound or line (the Wrath's `Toggle::Motion` without its activate). It
  waits for a motion that has the type 45 transition and no overlay under way, as the game's toggle does (a
  dodge or a jump in the way delays it). He turns to the hostile at once (the demo's: the game's AI is facing
  its target already), the `molotov` clip plays over his whole body (`Toggling::pose`, held where he stands,
  `holds_still`), blended in over the MOTION's 0.1 s and not out (`SkillKind::pose_blend`: the burst kills him
  at 0.70 s, and the Wrath's 0.2 s out would have taken the pose back to the idle by then). The clip's
  root travel leaps his picture forward as the footage shows; his node, and so the camera, stays.
* **At 0.70 s (the clip's 04592c53)**: `let_go`: the bottle is let go, **500 FLAME to himself**
  (`Skill::self_damage`, taken by `self_burst`: `Hit::new(500, "its own burst", FLAME)` through his damage
  handler like any hit, no attacker, a kill shoved straight up with the up-kick at the cap, FUN_00223160's
  record: `ragdoll::death_shoves`) and the **cloud h_1a256542 at his root node** (`Skill::fire`,
  `SkillKind::own_point`: `fire_effects` takes `missile::root_point` for his character, for a squad member
  too, where the Orbital Beam's target point comes from the aim). Its damage is #200's: 78 FLAME a second from
  the spawn for 1 s within 7 m of the point, x (1 - d / 7), to everyone whose root node is in range, **friend
  or foe, the mutant's squad included**, at their own difficulty step (STANDARD: the squad x0.6, enemies
  x1.4) and factors. He is dead on the spot (55 HP, the class's: no FLAME factor, the campaign type's x0.5 is
  stripped by the rule, 500 x 0.6 = 300), and the clip's 0.73 s switches the skill off.
* **Killed while holding it** (`SkillKind::off` at the death: play_skills.rs `step` switches a dead
  character's skill off): the cloud at where he falls, as FUN_0013dc90's. Killed in the lunge before the event
  too (the bottle is still in his hand); not after it (it is let go), not with it put away.
* A **knock-down in the lunge** cuts the clip short (`step` drops the pose) and he is holding the bottle
  again (it was not let go): the lunge starts over once his motion takes it.
* A **hand-over while he holds the bottle puts it away**, as a press of V would (`skills::control_left`, called
  from play_orders.rs `released`: the press is queued on the member control has left, and the next skills' frame
  takes it before the skill's own update): no cloud and no damage, the stamina spent stays spent, and the AI that
  plays him from then on never lunges or bursts on its own, a hostile within 5 m or not. **The product owner's
  call (2026-11-08), the demo's own choice**, not the game's: the game has no player who holds the bottle (its
  AI use is GOAL_ATTACK_MOLOTOV's, #374, out of scope; the squad's Toggle Ability order refuses the skill as the
  other attack skills' `attack_skill`). A lunge already under way is carried through, as a press of V is refused
  in it (control passes 0.5 s after the pick, `SELECT_TIME`, so the swap lands in the clip's 0.73 s only for a pick made
  from 0.5 s before the lunge starts to 0.23 s after it).

**The name.** The game's class names nothing (0x2bf0a0 returns 0). The string table has one skill name for
this idea, "Self-Destruct" (h_01fdf1ca), which is CSKILL_SELF_DESTRUCT 11's (vtable 0x39c870, whose only
method is its name, 0x13ef10: a class no character has), and the demo shows it on the HUD while the bottle is
held. A pick of the demo's, one constant (`NAME`) to change.

### The bottle

`Bottle` is loaded the first frame of a map on which a character has the skill (`load_bottle`, a lazy `Update` system:
the squad's slot, `BF_TEST_GIVE_SKILL`, the N and T menus' rows; **not an `OnEnter` system, whose presence reordered
its neighbours and moved the rack's pickups' entity indices, and so the Orbital Beam's, the Air Strike's and the
Wrath's debris**, see Verified): the skill's item h_e8eb77f8 (`MolotovSkill::item`) from the weapon table, its model h_0157c126 (a bottle 0.47 m long along its y axis, neck up: 46 vertices of radius 0.055
m on the body, 0.025 on the shoulder and 0.019 at the neck, and an 8-vertex label band; the throwable Molotov's
h_f42e0faa has the same mesh and the same attached flame h_f61f99b7), and that effect type's ALE effect (h_fd5dee85,
`boss_mutant_bottlefire` in the common library, compiled with the game's count and shapes as the hits' and triggers'
effects are, its fire riding on the effect: `The flame`, below) and sound (h_f6dbdb17, 0.51 s, queued once as the bottle is lit). `show_bottle` puts a copy on the joint
of the bone the skill's hardpoint (`MolotovSkill::hardpoint`, h_f8474385) is on, for every character who holds one,
and takes it away when he no longer does.

* **How it is held.** The model's two hardpoints are `FOREGRIP` (h_ea5e02f2, 0.067 m under the middle of the body:
  where the support hand holds a gun) and h_05517dfe (0.215 m up the neck). The bottle is mounted as a gun is, its
  foregrip lined up with the hand's hardpoint (`Hardpoint::mount`): the foregrip frame's -z is the neck's
  direction, as the guns' is the barrel's. In the mutant's dash clip the hand's -z points up (the forearm is
  raised: its frame in the model at 1.4 s of a run, -z = (0.52, 0.85, -0.07)), so **the neck is up and tilted 30
  degrees out, as in take02's dash (6.20 - 6.47 s)**. A rigid grip turns with the hand: in the idle clip the arm
  hangs and the neck points forward and down, with the flame at its end, which take09's idling mutant (the flame at
  his hip, standing up beside the torso) doesn't show; whether the game's idle clip holds the arm higher isn't known,
  and the demo has no pose to raise it.
* **The flame.** The attached-effect list names the hardpoint h_e9680caa, which the model doesn't have; the game finds
  the entry's node by name (FUN_000cdd10 through FUN_0023c4e0), and with none of that name the effect hangs on the
  object's own node, the model's origin. The effect agrees: its emitter's xform puts it 0.16 m along +y of its origin
  (`EMITTER_UP`), just under the neck. Its structure (`ale_tool.py effects common.tgz bottlefire`): the **fire**
  pair (`boss_mutant_bottlefire.emt` / `.app`: 40 particles a second, 0.3 - 0.5 s lives, 0.07 - 0.29 m sprites,
  taller than wide, along the particle's motion) has its emitter and its appearance both under the library's
  builtin world node, which the ALE runtime reads as a head, not a trail (`Compiled::ride`, the bolts' heads:
  todo/57, todo/59), so **its particles ride on the effect** (`AleAssets::load_recorded_riding`): they travel with
  the bottle and are gone with it, put away or let go. The **smoke** pair (`..._smoke`, `smokecard.tga`, 1.1 -
  1.3 s lives) has its appearance at the root: a trail, left where it is born (take02 6.30 and 6.33 s show faint
  puffs along the runner's path, which are probably it). Round 1 compiled the whole effect with
  `load_recorded_facing`, which leaves every particle in the world: a runner left fire puffs along his path, a
  bottle held still piled them into a thigh-sized blob at the hip, and the lunge left a sprite hanging above where
  it began for 0.4 s (the tester's a1_zoom and b1_zoom2). The flame's effect object is not a child of the bottle
  either: its frame is the world's, its origin 0.16 m below the point of the bottle the data puts the source at
  (`lift_flame`), so **the flame rises whichever way the hand turns the bottle**: take09's idle shows a tongue
  standing up at his belly and take02's dash one over the hand, and a flame that turned with the demo's bottle
  (its neck forward and down in the armed idle, above) burnt along the thigh in a blob.

### Keys and test hooks

* V, or `BF_TEST_SKILL=<s>[,<s>...]`; `BF_SKILL_LOG=1` prints the toggles and their refusals, the bottle lit,
  put away or fizzled, the lunge's start and its burst, the damage to himself, the target point (Own) and the
  half-second stamina lines; `BF_DAMAGE_LOG=1` the cloud's ticks, `BF_COMBAT_LOG=1` the hit's factors.
* **The class**: `BF_TEST_SQUAD_PRESET="0.3,mutant_horde"` (the Mutant Horde: the Molotov Mutant is slot 1, the
  player's; the Ugly, the Supermutant and the Filthy Mutant the squad), or `BF_TEST_SQUAD_SLOT="0.3,me,
  mp_mutant_molotov"`. The campaign types (`molotov mutant@e23`, h_f0c5e50e, and e01's, m03_a's and m03_b's, the
  N and T menus' rows) have the class too, and their `h_e78efbb2` is false (the game's AI mutant: the stamina stays
  100, nothing drains it). **The character's skill runs the stamina update whatever the type says**
  (`Skill::new` sets the flag on the character's own copy of the data, for this skill alone: the drain, the
  refill, the lunge's `update` and the fizzle are that update's work), so a controlled campaign Molotov Mutant
  plays as the class does. The type's own data stays the game's (a test map NPC of the type reads it).
* `BF_TEST_GIVE_SKILL=20[,<cost>[,<recovery>]]` gives the character controlled at a map's start the class (3 and
  1.5 unless given): a squad member stands in with his own `Sc_w1_bz_start` overlay for the `molotov` clip (the
  burst at 0.70 s and the end at 0.73 s are the clip's numbers, used where the stand-in has no such event).
* `BF_TEST_NPC=<s>,<type>,enemy,frozen,<x>,<z>` for the hostiles (`friend` for a squadmate to see the cloud
  hurt: frozen NPCs stand where they're put), `BF_TEST_STAMINA=<n>@<s>` for the fizzle (`10.5@0.5` with V at 1.0
  drains to 0 at 4.8 s), `BF_TEST_DIE=<s>` for a death while holding, `BF_TEST_KNOCK=<s>`, `BF_TEST_FIRE=1`
  (fire held from 1 s: nothing leaves the gun while the bottle is held), `BF_TEST_SELECT=<character>[,<s>]`
  (the hand-over). Under `BF_TEST_FREECAM` the movement hooks are dropped (`testtools::gate_input`), the
  skill's are not.

### Verified (#211)

`bf_play --test` with `BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures (`BF_CAPTURE`; the lunge frame by frame at 15 fps
puts the burst at the 11th frame, 0.733 s, where the 30 Hz game's is the 21st, 0.700 s), `BF_SKILL_LOG=1` and
`BF_DAMAGE_LOG=1` for the numbers, the frames looked at (the sheets are in the report). The test floor is on
STANDARD (the squad x0.6, enemies x1.4). The Mutant Horde, `BF_TEST_SQUAD_PRESET="0.3,mutant_horde"`, the player the
Molotov Mutant (55 HP):

* **Lighting and holding** (V at 1.0 s): "toggle (key) at stamina 100.00 ... Now", "the bottle is lit and in his left
  hand, stamina 100.00 draining 3 a second", the HUD's "Self-Destruct" from that frame and fading 0.4 s after the off;
  the stamina 100.0 -> 98.4 at +0.5 s ... 91.0 at +3.9 s, 3 a second, 33 s from full. `BF_TEST_FIRE=1` (the trigger
  held from 1 s, `BF_SHOT_LOG=1`): **no bolt launched in 10 s**; the fizzle's run, with the trigger held too, fires from
  4.87 s, the frame after the bottle went. Walk, run and dash move him as before.
* **Not free to act**: the crouch key (`BF_TEST_CROUCH_KEY=0.5,3.0`, V at 1.5 s) kneels him at 0.5 s, V is accepted from
  the crouch (MT 6 has the transition), the press at 3.0 s is lost and he stays down (the refill x2, kneeling, goes on
  after the off at 5.1 s).
* **Cancel** (V at 1.0 s and again at 3.2 s): "the bottle is put away at stamina 93.80", off with the line 09087c9c, no
  cloud, no damage; the stamina is 94.2 at 3.40 s (the refill 1.5 a second from 93.8, not the 100 back), and V again
  lights it (3.40 s) and puts it away (4.13 s) the same.
* **The lunge at 5 m** (a frozen enemy at 5.2 m of a standing mutant: none in 5 s; at 4.8 m: "a hostile within 5 m:
  the lunge" the frame after V). Walking in from 14 m toward an enemy at 2 m the lunge came at 3.00 s with the enemy
  4.5 m off (the mutant 0.4 m on from the trigger, which the frame's step at 3.9 m/s explains). The command: "toggle (its
  own end) at stamina 99.60, on: Pose", "special motion (MT_TOGGLE_SPECIAL), 0.73 s": the `Sc_w1_molotov` clip,
  whole-body, he held where he stands. Frames from the side (`BF_TEST_FREECAM`): he turns to the foe at once, leaps
  2.8 m toward it (the clip's root travel: the picture only; his node, the camera and the cloud's centre stay),
  the bottle over his head, comes down hunched beside it and, at the burst, goes up in the cloud.
* **The burst** (0.73 s on at 15 fps): "the clip's 04592c53 at 0.73 s: the bottle is let go, 500 FLAME to himself, the
  cloud h_1a256542 at his root node", "its own burst 500.00 x STANDARD (n -4) squad 0.60 x type 5 1.00 = 300.00", dead
  (55 HP), "the target point [0.02, -0.12, 2.55] (Own; ... 0.9 m from the feet)", "effect h_1a256542 starts at [0.02, -0.12,
  2.55]: 1 damage areas (78.0, 5, 1.0, 7.0, 1)", owner the Molotov Mutant. Then the skill is off in the same frame
  (the clip's end 0.73 s: at 30 Hz the burst is a frame before it).
* **The cloud's damage** (the squad and the foe at known distances, `BF_TEST_NPC` friends frozen, STANDARD): 78 x (1 - d / 7)
  x the step, over the second from +0.07 s: a friend at 4.22 m **18.6** (take02's Tex at the same distance lost 18.29: 78 x
  0.397 x 0.6 = 18.6), at 4.72 m **15.2** (take02's Brutus 16.21; the formula 15.3), at 6.5 m **3.3** (take09's Brutus at 6.5 m
  lost 4), at 2 m 33.4, and the enemy at 3.01 m **62.2** (x1.4), the three Horde mutants who followed him 23.6, 18.1 and 8.7. The
  ticks are the 1/15 s frames' (`BF_DAMAGE_LOG`: "area h_1a256542 at 4.22 m: Hawk takes 1.24 (x0.60 the step)", 14 and a half
  of them): the [effects.md](../effects.md) rules, #373 and #200.
* **The fizzle** (`BF_TEST_STAMINA=10.5@0.5`, V at 1.0 s): lit at 11.3, the stamina falls 3 a second, "toggle (at 0) at
  stamina 0.00, on: Now" at 4.73 s, "out of stamina, the bottle fizzles: no burst", "skill off ... line ee0a5cbc" (the
  game's out-of-stamina line), the bottle gone the next frame, no cloud and no damage, the trigger held through it firing
  again from that frame.
* **Killed while holding** (`BF_TEST_DIE=3.0`, V at 1.0 s): "killed holding the bottle: it bursts where he falls", the
  cloud at [-0.00, -0.12, 8.00], his root node; two frozen friends at 3.2 and 3.0 m lost 26.2 and 24.3 HP. **Knocked down in
  the lunge** (`BF_TEST_KNOCK=1.3`, the lunge begun at 1.13 s): the clip is cut, he is holding again, and the lunge starts over
  at 1.47 s, once the knock-down is over ("a hostile within 5 m" twice), the burst at 2.67 s.
* **`BF_TEST_GIVE_SKILL=20`** on Tex (`BF_CHARACTER=tex`): lit, the 1.00 s `bz_start` overlay stands in for the clip, the
  burst at the 0.70 s fallback, killed.
* **Unchanged against main** (round 1: 5482136 for the eleven below; round 2: c665902, rebased onto it, for Hawk, the Orbital Beam, the Wrath, Tex's Berserker through a hand-over (`BF_CHARACTER=tex BF_TEST_SKILL=1.0 BF_TEST_SELECT=1,2.0`) and the Horde: 5 of 5 with no difference in 22 - 32 lines each), the skill and effect logs line for line (`BF_SKILL_LOG`, `BF_DAMAGE_LOG`,
  `BF_COMBAT_LOG`, 10 s of 15 fps captures each): the four squad skills switched on and off (`BF_CHARACTER=<tex|brutus|
  hawk|flint> BF_TEST_SKILL=1.5,5.0`: 30, 37, 28 and 27 lines), the Orbital Beam, the Air Strike, the Wrath of Garner
  (given to Brutus), the Cloak (Hawk), McTavish's Tracking (Flint) and the Vengar's charge (`BF_TEST_GIVE_SKILL=9`, `6`, `13`,
  `8`, `12`, `14`) and the Mutant Horde with no skill pressed: 11 of 11 with no difference. The Wrath's pose frames
  (`apply_overlay_blend` takes over from `apply_overlay` for the pose; its blend is the same 0.2 s both ways) are pixel for
  pixel main's: the difference over the character's region 0.000 - 0.025 in 8-bit means against 0.000 - 0.025 between two runs
  of main.
* **Round 2** (the tester's failure and look findings, the reviewer's labels and the product owner's hand-over call;
  `BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures, main c665902's exe as the baseline and round 1's own as the "before"):
  * **The campaign types** (`BF_TEST_SQUAD_SLOT="0.3,me,molotov mutant@e23"`, V at 1.0 s, a frozen enemy 3.5 m off from
    2.5 s). Round 1's build lit the bottle and kept the stamina at 100.00 "on" for the 6 s, and never lunged; this one
    drains it (98.40 at 1.53 s, 97.00, 95.40: 3 a second), has "a hostile within 5 m: the lunge" at 2.67 s and the burst
    at 3.40 s ("takes 150.0 of its own burst (55.0 -> 0.0 hp), dead": the campaign type keeps its FLAME x0.5, 500 x 0.6 x
    0.5, where the class strips it, 300). e01's row the same, 22 HP. m03_a's and m03_b's types aren't in the test map's
    loaded data; `Skill::new` sets the flag for the skill-set whichever type brings it. (Sheet J.)
  * **The held flame.** Standing still (2.0 and 3.0 s): an upright tongue 0.3 m tall beside the hand, over no thigh and
    in no haze (round 1: a thigh-sized blob with smoke). Running: a tongue on the bottle and no fire trail along the legs
    (the smoke's puffs, yellow at birth, still mark the path). The lunge: the flame goes with the bottle through the
    swing, and the fire sprites that hung above the start for 0.4 s are gone; one faint smoke puff is left there, which
    take02 6.30 and 6.33 s show too. The flame's place is this frame's (the first version, put from the last transform
    propagation, trailed the bottle by 0.13 s in a 15 fps capture, half a metre in the swing). (Sheets G and H.)
  * **The blast debris.** Cause: the pickups' spin jitter is by entity index (play_pickups.rs `physics`), and the first
    round's `OnEnter(Playing)` loader reordered its unordered neighbours there, so every pickup of the rack had main's
    index + 3 (+ 5 before the warm-up quads were taken out of it): `BF_PICKUP_LOG` "came to rest" lines 420v1 and 413v1
    in main, 423v1 and 416v1 in the build. Bisected by switching the plugin's systems off one at a time
    (`BF_MOLO_PLUGIN`, a temporary hook that was not kept): only the `OnEnter` one moved them. Now `load_bottle` is a
    lazy `Update` system and the indices are main's: the four blast scenarios (the Orbital Beam, the Air Strike, the Wrath of Garner, Brutus's
    charge; `BF_TEST_GOTO=0,4,0,4 BF_NO_SQUAD=1`, 75 frames) have the same "came to rest" lines as main's, and the same
    frames: the flying debris is identical. The only differences left are a faint screen-wide shimmer of at most 6 / 255
    on a few frames of the Orbital Beam (33, 63, 66 - 68), which two runs of main show as much (33, 56, 64, 65, 67, means up
    to 0.24). The Mutant Horde's maps do allocate the bottle's warm-up quads (once a character has the skill), as any
    feature adds its own.
  * **A hand-over while he holds the bottle** (`BF_TEST_SQUAD_PRESET="0.3,mutant_horde" BF_TEST_GOTO=0,4,0,4
    BF_TEST_SKILL=1.0 BF_TEST_SELECT=0,1.2`, a frozen enemy 3.5 m off from 1.8 s, or from 1.7 s, the swap's frame):
    round 1's build had the AI Molotov Mutant lunge at 1.93 s and burst at 2.67 s (the Ugly Mutant now the player); now
    "control has left him holding the bottle", the toggle's put-away at 1.73 s (stamina 98.00, refilling 98.50, 99.30,
    100 from there), and **no lunge and no burst in the 4 s that follow**, the foe in range or not. With the foe coming
    within 5 m at 1.4 s, before the swap at 1.67 s, the lunge starts at 1.53 s and is carried through: the press is
    "refused: the lunge is under way" and the burst comes at 2.27 s. (Sheet I.)
* Unit tests (`cargo test`: 113 + 394): the factory and the three dispatch points, the toggle's gates (8.99 refused, 9.0 passes, the
  crouch's transitions, a dodge, a jump, a slide, a grenade in the hand and a reload refuse), a press lighting and putting
  it away and the stamina's 0 doing the same with no cloud, the lunge's start and burst (the burst at the 21st frame of 30 Hz,
  once; the end the frame after), the motion that takes the lunge, the death's cloud only with the bottle in his hand,
  the campaign type's stamina update set on the character's copy of its data (and another skill's left as its type has
  it), and the hand-over's press taking the put-away with a hostile in reach; the block read in `bf/character.rs`.

### Open and not done

* **The game's bottle that falls loose** (holder +0x38) isn't made, and with it the grenade rules' burst on its first
  contact (13 - 26 FLAME in 7 m and the same cloud again): not traced for this bottle, and the takes' damage is one
  cloud's (above). A follow-up if footage shows a second burst.
* **The AI's own use** (GOAL_ATTACK_MOLOTOV on sight: the dash at the target, the shout f937e583 between 20 and 5 m, the
  toggle, the throw before it, EVT_TGT_IN_GROUP) is #374's and #26's: nothing in the demo has the enemy mutants run at
  anyone, so a test map NPC Molotov Mutant never lights its bottle (its skill is made and never stepped). The squad's
  Toggle Ability order refuses the skill (an attack skill).
* **The HUD name** is the demo's pick ("Self-Destruct").
* **The idle pose** holds the bottle neck-forward-down (above), which the flame no longer follows (it rises); the bottle
  itself is a guess for the armed idle. The smoke is a world-space trail by the ALE runtime's reading of its emitter
  under the world node (a faint haze round the hip held still, puffs along a run), consistent with take02's puffs
  but not compared with the game's.
* **What char+0xdc holds**: the owner of the cloud at a death is the mutant here; the game's is that object when it is
  a character (the killer? the last attacker?), which would make the cloud's kills the killer's.
* **The leap's picture and the ragdoll**: the clip's root travel (2.37 m) leaps the mutant's picture and his corpse
  starts from that pose, 2.8 m from the cloud's centre at his node; take02's body is in the middle of its fireball, which
  the camera's line of sight doesn't settle (the fireball is 4.2 m from Tex, in front of Tex's shoulder on the screen).
* **Footage not taken**: the player-controlled design is the product owner's, so no take shows it; M3 (a bioreactive kill: the
  mutants' BIOREACTIVE x0) and M4 (the idle bottle) of #211's list weren't filmed.
