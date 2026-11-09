*Part of [skills/](README.md).*

## The Mutants' Corpse Eater (#210)

CSKILL_CORPSE_EATER 16: the skill of m03_a's and m03_b's Filthy Mutants (h_1662a599, h_0b10f748: stamina cost 3,
recovery 1.5). In the game it is **set dressing the level's script plays**: a mutant kneels over a body and feeds,
and the class's only work is a gore effect at the clip's events. No character a player controls has it, and the
campaign's Filthy Mutant (e23's h_fa73c507, `mp_mutant_filthy`) carries skill-set 0. The demo gives it to its
Filthy and Ugly Mutant classes (the test map's, the Mutant Horde's) by **the product owner's design of 2026-11-09, demo-own**:
V next to a corpse kneels and feeds, gore at the hand, and health comes back. This page is the class
(`src/bin/play_corpse_eater.rs`, one of the effect skills: an arm in each of their three dispatch points,
`play_effect_skills.rs`), what the game does, what the demo does, and how it was checked. From default.xbe (the
rows are in [code-map/skills-and-stamina.md](../code-map/skills-and-stamina.md): FUN_0013c800, 0x13c830,
FUN_0013c850, the vtable 0x39d408), the analyst's answers (`todo/npc-skills/answer.md` §3.3,
`todo/132-mutants/leads.md` §7) and the footage of `todo/132-mutants` (take05 and take13: the kneel, the loop, the
red drip at the hand).

### What the game does

* **The class** (FUN_0013c800, vtable 0x39d408, 0x20 bytes): the base skill class with **one override**, +0x18
  0x13c830, the animation-event handler (FUN_00117660 hands every event to it first, 0x11768d). No toggle, no
  drain, no update, no name (its +0x4c is the base's, 0x2bf0a0, which returns 0): nothing a press can do.
* **The block** (`<corpse-eater h_e698b665="h_1eeb36f7" h_0e13b0f0="h_f8474385"/>`, game-options-en.xml line 174;
  `CorpseEaterSkill` in `bf/character.rs`, parsed by FUN_0019e940 into settings +0x370 / +0x374): the effect type
  and the hardpoint it is attached at, **h_f8474385, the left hand** (`weapon::SUPPORT_HAND`, the attribute the
  Molotov Mutant's bottle has too).
* **The event e8262ae5** (0x13c830: `cmp` against it, 0x13c850): the effect type spawned **attached to the
  character** at the hardpoint — effect vtable +0xdc, flag |= 2, FUN_0021c380(1.0). The type h_1eeb36f7 is only in
  objecttypes-m03_a.xml (line 1228) and m03_b's (line 1509): one ALE effect h_1d563f48, **no damage, no light, no
  sound, max-life FLT_MAX** (so the game's own effect would never end by itself). It is the red gore at the hand in
  take05 and take13.
* **The clips**: MT 37 `eat_start` (1.13 s: the body drops 0.43 m, the kneel), next-motion MT 41 `eat_idle` (a
  9.13 s loop) and MT 40 `eat_stop` (0.83 s, the stand-up). The times sit in **`eat_idle`, at 0.37, 1.70 and
  6.13 s**. (A second event, 060975c0, sits at 1.37, 2.90 and 7.67 s; what it carries wasn't identified.)
* **Nothing heals**: the class has no health code, and the data gives no rate. The ticket's E3 (the design take)
  was never filmed either, so there is no game number to copy.

### The player's design (demo-own)

* **The body is whatever lies at his feet** (`Skill::corpse`, `watch_bodies` + `nearest_body`): **any dead
  character's body** — the squad's, a test NPC's, a level enemy's, either side, any type — at the point the corpse
  is at (its ragdoll's root, where the death camera, the blood pool and the memory chip go by, else where it died),
  within **2.2 m across** from his feet and **on his floor** (1.8 m above or below; a body on a ledge below or a
  balcony above isn't his). It is measured in the ground's plane and from the corpse's own point, so a body he
  stands over counts whichever way he faces, and a body that slid or was thrown away from where it died is found
  where it lies. The game's script has no check to copy: it plays the motion at a placed mutant.
* **V next to it**: the shared switch on (the 10% of the maximum, the class's cost 3 a second draining from the
  press), **he turns to the body and kneels** (the `eat_start` clip, then `eat_idle`'s loop), and the gore effect
  starts at his left hand at each of the loop's three events, once a cycle.
* **While the loop plays he regains health, 6 HP a second** (`HEAL_PER_SECOND`): **a demo value**, labelled so,
  because the class has none and the data no rate. The class's own drain (3 a second) is what caps a feed: about 33 s
  from a full bar, from half health to full in 8.3 s.
* **Exposed**: no fire and no aim (the controlled character's `fire` and `aim` cleared before his step,
  `hands_busy`; `holds_fire` for an AI one) and he isn't free to act (`lets_act`: no reload, weapon switch, grenade,
  medkit, weapon pickup or crouch key), as the Molotov Mutant's bottle is. The game's class says free to act always
  (0x213690); the demo's rule is the product owner's.
* **V again** stands him up (`eat_stop`, the skill stays on until the clip has played, then the off). **Moving off
  the body** (0.02 m from where he knelt) does the same, and so does **the body's own going** — thrown out of the
  reach by a blast, taken out of the world by a trigger: the body is what the feed is about, and the scan finds none
  — and a **knock-down**, the **death**, or the **stamina's 0** (the base's command, which fizzles the feed as it
  puts the bottle away).
* **The HUD says "Feast" while he feeds** (`NAME`, the demo's own caption put in `Game::strings` as the demo's
  class names are): the class names nothing (its +0x4c returns 0) and the English table has no corpse- or eating-word
  at all, so it is a demo pick like the Molotov Mutant's "Self-Destruct". **Demo-own, the product owner's choice**
  (2026-11-09: `Feast`, over the first pick `Scavenge` and the table's `Meatgrinder`).
* **Kept open, documented**: no **auto-stop at full health** (he keeps feeding and keeps draining until the bar or
  the stamina ends it), and **the body isn't used up** (the game's script has no such rule either; a corpse model
  isn't the demo's to consume). Both are the demo's choices where the data says nothing.
* **A map whose archives don't define h_1eeb36f7** plays the chain with no gore: `load_gore` says so in the log.
  The test map's own level data carries it (`corpse eater: the gore effect h_1eeb36f7: 1 bursts`), so the gore is
  there to be looked at.

### How the clips and the gore are played

* The three clips are `play.rs`'s `Loaded::eat_clips`, from the type's motion set (MT 37 / 41 / 40,
  `Motion::clip`: the script and the prefix, `Sc_w1_eat_start` / `_idle` / `_stop`), the whole body's with their
  roots, played after the skill toggle's overlays (`apply_overlay`, `full` and `root` both true) from
  `Skill::feeding`'s slot and clock. The skill itself only times them: a skill's methods see no `Character`.
* **His own soles on the floor.** A whole-body clip's lowest point isn't a sole: a kneel's is its knee, and the
  stance's sole lift leaves a body in the air by that much (the crouch clips' own had to be kept for the same reason,
  `clip_lift`'s note: "Hawk and Flint knelt in the air"). The three eat clips' lifts are `clip_lift`'s, kept in
  `Loaded::crouch_lift` beside the crouch's, and the pose sets `lift_now` from the clip's own at its time while he
  feeds (`clip_lift_at`), the clip's root bringing the 0.43 m drop on top.
* The gore is spawned by `show_gore` from `Skill::gore` (the flag `CorpseEater::update` sets at each event, the
  way the Molotov Mutant's `Skill::self_damage` carries what its methods can't do themselves): the type's ALE
  effects compiled with the game's facing 3 (`load_recorded_facing`), one run each (`AleEffect::once`) at the
  block's hardpoint, **children of the hand's joint**, so they ride with the hand as the game's attached effect
  does, and despawned by `AleExpire` after the effects' own lengths (the type's FLT_MAX can't end them).

### Keys and test hooks

* V, or `BF_TEST_SKILL=<s>[,<s>...]`; `BF_SKILL_LOG=1` prints the presses and their refusals, the body found or
  lost (how far off it is), the kneel (with the block's effect and hardpoint), each loop event's gore, the heal's
  whole points, the stand-up and its reason, and the feed's end.
* **The class**: `BF_TEST_SQUAD_PRESET="0.3,mutant_horde"` and switch to the Filthy or Ugly Mutant, or
  `BF_TEST_SQUAD_SLOT="0.3,me,mp_mutant_filthy"` (or `,mp_mutant_ugly`).
* `BF_TEST_GIVE_SKILL=16[,<cost>[,<recovery>]]` gives it to the controlled character (3 and 1.5 unless given), on
  any class: the clips are his own type's motion set's, and a type without them plays the chain's clock with no pose.
* **A body**: `BF_TEST_NPC=<s>,<type>,enemy|friend,frozen,<x>,<z>` (any character type, any side) with
  `BF_TEST_DIE`, or shoot it; a level character's or a squad member's corpse counts too.

### Verified (#210)

* `cargo check --bin bf_play`: no warnings from the module.
* Unit tests (`cargo test --bin bf_play corpse_eater::`): the factory and the dispatch points (skill-set 16 makes
  the class, the demo's `Feast` caption, nothing feeding off), **the body the press finds** (`nearest_body`: a body 3 m off, 2.4 m up a
  ledge or 2.4 m below one is out; at his feet whichever way he faces, a standing body's point at the hips and a
  corpse's near the floor both in; the nearest across wins), no body within the reach refusing the press and one
  there kneeling him over it (turning him to it, the start clip's slot, exposed while it plays), V again standing
  him up with the skill on until the stop clip has played, the loop's three e8262ae5 events at 0.37, 1.70 and
  6.13 s (once each, none in the kneel) with the heal at 6 HP a second, moving off the body standing him up with
  nothing healed, and the body's going (the scan finding none) standing him up in the loop with nothing healed
  after.
* A run of the test map with the horde (`BF_TEST_SQUAD_PRESET="0.3,mutant_horde"`): `load_gore` finds the effect
  (one burst), and every type with the clips has their lifts (spawn's line: `the corpse eater's clips' lifts ...`):
  the Filthy Mutant's kneel sits at -0.542 m and his stop at -0.541 to -0.523, against his stance's -0.109 m — the
  0.43 m drop the clip's root brings, so `lift_now` takes the kneel's own and not the sole's.
* **Not yet seen in play**: no capture has been taken of the chain's frames or of the gore, so the pose, the hand
  the effect sits on, the "Feast" caption on the bar and the effect's look are **unverified**, and so is the
  reach's feel (2.2 m across). What to look at first: the frames of the kneel at the body (his knees on the floor,
  not hovering), the loop's three bursts at the left hand, the bar's "Feast", and the stand-up.
