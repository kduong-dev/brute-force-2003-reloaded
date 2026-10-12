# Special skills: stamina, the skill toggle (#120) and Tex's Berserker (#121)

Each squad member has a special skill that stamina (the blue bar) pays for: Tex's Berserker,
Brutus's Spirit of Vengar, Hawk's Stealth, Flint's Advanced Targeting. This page is the part the
four share (`src/bin/play_skills.rs`, the HUD in `play_hud.rs`): stamina, the toggle, its sounds,
chatter and HUD, and the interface each skill plugs into; and the skills done so far. Each
skill's own effect is its own ticket: #121 Tex, #122 Brutus, #123 Hawk, #124 Flint. Tex's
Berserker works ([below](#texs-berserker-121)); the other three switch on and off, drain and
show on the HUD, but do nothing else yet.

Everything here comes from default.xbe (the functions are in [code-map.md](code-map.md), "Skills
and stamina" and "HUD") and the data, checked against the 32 takes in
`todo/35-special-skills/` (the spec is `spec.md` there, Parts 1 and 2).

## Keys and test hooks

| Key | |
|---|---|
| V | the controlled character's skill on / off: the game's White button (VB_SPECIAL_ABILITY, `game-options-en.xml` line 151, which sends CT_TOGGLE_SPECIAL at 0x134cff). Not while dead |
| Q, R | in Tex's berserk: end it, instead of switching weapons (Q, the game's Y) or reloading (R, the game's X; only when a reload could start, which then follows berserk's end; on R's press, where the game's X ends it when let go). See [Tex's Berserker](#texs-berserker-121) |

* `BF_TEST_SKILL=<s>[,<s>...]`: presses V at those times (once each).
* `BF_TEST_STAMINA=<n>[@<s>][,...]`: sets the controlled character's stamina to n at those times
  (at the start without one), as the takes' pokes: `30@1.9` with `BF_TEST_SKILL=2.0` is take02.
* `BF_SKILL_CHATTER=<percent>`: the skill lines' `probability` (100: every line the rest of the
  chatter gate lets through), to hear them.
* `BF_SKILL_LOG=1`: each toggle command (the key's, at 0, a skill's own) and its answer, each
  overlay and when it switches, each switch with its sound and line, and every squad member's
  stamina each half second with its state (`on`, or `off x` its refill factor). For Tex also
  each refusal and why (not those of the command at 0, which comes every frame), the top-up at
  the press, the second gun drawn, and the switch and reload keys ending berserk.
* `BF_TEST_SPRINT=<s>` (play.md): sprints toward the `BF_TEST_GOTO` goal from that time, for the
  dash's refill. With `BF_TEST_CROUCH`, `BF_TEST_SELECT`, `BF_TEST_DIE`, `BF_CAPTURE`
  (`BF_CAPTURE_FPS=60` for the blink and the fades).
* For Tex's berserk (play.md's ammo and shot hooks): `BF_TEST_WEAPON="Confed LZR-50;Jax-iC"` (a
  list fills the weapon slots in order: the takes' loadout), `BF_START_WEAPON=1` (the Jax-iC in
  hand, as takes 05 and 26), `BF_TEST_CLIP=<n>[,<n>...]` (a clip per slot: `99,4` is take05's
  poke), `BF_TEST_TRIGGER`, `BF_TEST_SWITCH`, `BF_TEST_RELOAD=<s>[,...]` (presses R),
  `BF_TEST_THROW`, `BF_TEST_CROUCH_KEY=<s>[,...]` (presses Z: a crouch press, not
  `BF_TEST_CROUCH`'s hold), `BF_TEST_SCOPE=<step>,<from s>` (the scope asked for from that time,
  without `BF_TEST_GOTO`, whose autopilot doesn't read it); on the test map
  `BF_TEST_NPC_HP=<s>,<n|aim|all>,<hp>` (testtools.md) gives an NPC take04's 999 HP, and
  `BF_RETICLE_LOG=1` (play.md) shows the lock's reach.

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
campaign's in the demo.

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

## The toggle

V asks the controlled character's skill to toggle (`Skill::request`, read after the hand-over so
a press in that frame goes to the new character); a squad order will too (#33's "Toggle
Ability"). The command handler (0x1224a5) calls the skill's own toggle, which switches the skill
itself or not, and whose return picks the motion (0x1224e4): 0 none, 1 the type's overlay, 2
MT_TOGGLE_SPECIAL. In the demo (`Toggle`):

* **Refused**: nothing at all happens. No motion, sound, text or HUD change (take03 2.0 s, take11
  52.0 s; the low-stamina blink already running isn't a reaction to it).
* **Now**: the skill switches at once; `overlay` would play the overlay too (a toggle that
  switches and returns 1; none of the four).
* **At the clip's event**: the toggle returns 1 without switching, the overlay plays and the skill
  switches at its event (the game's animation event handler, FUN_00117660).

For a 1 the handler looks the overlay up by the skill's state after the toggle, OMT_SPECIAL_OFF
(43) if it's on and OMT_SPECIAL_ON (44) if not (0x122500-0x122564), the only lookup of those two
overlay types in default.xbe, and queues it only into a free overlay slot (char+0x218, or +0x244
by the overlay record's +0x3d bit 2: 0x12253a-0x122587). The overlays are the motion sets'
`<overlay motion-type=44 / 43>` (`Game::skill_overlays`, read from `ms_<name>.xmb`; `prefix-type`
1 puts `Sc_w1_` / `Sc_w2_` before the script for the held weapon's stance). They're played as the
reload is, the whole body standing and the upper body on the move (`Loaded::skill_clips`,
`step_player`).

| | Toggle | Returns | Overlay | Switches |
|---|---|---|---|---|
| Tex | 0x13d490 | 1 | `Sc_w1_bz_start` / `Sc_w1_bz_stop`, 1.0 s | at the clip's 0.50 s event (eaba3d48 on, 14ee2201 off). A 1.0 s timer from an accepted press refuses presses meanwhile; switching on also needs him free (FUN_00117210, 0x13d5c4), 10% (0x13d5d3) and the rest of [his refusals](#texs-berserker-121) |
| Brutus | FUN_0013bfe0 | 0 (0x13c00e, both ways) | none: his motion set's `Sc_w1/w2_spirit` never plays from the button | at the press |
| Hawk | FUN_0013cf90 | 1; 0 at stamina 0 or dead | `Sc_w1/w2_activate_stealth`, 2.0 s, on and off | at the clip's 0.83 s event (ead033a3); refused unless she's free (FUN_00117210, 0x13cfd2; her clip under way counts); at stamina 0 or dead: off at once, no clip |
| Flint | FUN_0013cad0 | 0, or 2 with a rifle (MT 45, never seen in the takes) | none | at the press |

Brutus's and Flint's toggles don't ask FUN_00117210 (Flint's makes checks of its own, #124's).

The demo switches 0.50 s (Tex) and 0.83 s (Hawk) after the press frame (a clip started this frame
starts moving the next); the takes measured 0.53 s for Tex, the event plus about one 30 Hz frame,
and 0.89-0.97 s for Hawk less the rig's 0.02-0.06 s input delay.

* **Switching on** (FUN_0010e470) needs stamina ≥ 10% of the maximum (equal is allowed). Tex's and
  Hawk's toggles check it before their clip; Brutus's and Flint's switch is refused by it the
  same silent way.
* **Free to act** (`free_to_act`, the demo's FUN_00117210): alive, not knocked down, no grenade
  held (charging one: V then is refused instead of dropping the throw) and no reload, throw,
  medkit, weapon switch or toggle under way. FUN_00117210 itself tests the action slot +0x214,
  +0x225, the dead flag and the items held in +0x1fc / +0x200.
* **At 0** each skill ends as its toggle does: Tex plays his stop clip and goes off 0.5 s later,
  on at 0 meanwhile (the repeated commands refused by his timer; take02); Brutus, Hawk and Flint
  go off in that frame (take07, take11, take17).
* **What ends a skill**: the key again; stamina at 0; death (the death handler calls the
  deactivate, 0x115df5: in the frame of death, with its end sound and line); and each skill's
  own ends, from its `update` and the player controller (Tex's other gun dry, the switch and
  reload keys: below; Flint's clip dry and the same keys, #124's; Brutus's charge, #122's). Being
  hit doesn't (take27).
* **Handing control over doesn't end it** (take19): the skill stays on and drains on the
  character you left, under the AI; it's on when control comes back. A clip under way plays on.
* **The demo's choices** (not traced in the game):
  * a knock-down cuts a toggle's clip short, and the switch it carried with it (the ragdoll
    takes over the pose);
  * one overlay at a time: no reload, throw, medkit or weapon switch starts during a toggle's
    overlay. The code shows only that a toggle overlay goes into a free overlay slot (above);
    whether a reload, throw or switch waits for one isn't traced.
  * The aim doesn't turn the gun over a toggle's overlay: every motion set's `<overlay>` entry
    says aim="false".
* The squad AI never switches a skill on by itself (take32: 60 s of fighting); only the order will
  (#33).

## Sounds and chatter

* On and off (FUN_0010e470 / FUN_0010e3e0) play the type's `start-skill` / `end-skill` sound
  (Tex ed7492f4 / e00d91fa, Brutus ee753485 / 190cd1f6, Hawk fa7509ca / f57b8dca, Flint 07168977
  / 1656217b) at full volume, in 2D, **for the controlled character only** (FUN_0010e7a0 asks
  FUN_00112790; take18: an AI Hawk's switches are silent). At the switch, not the press. The off
  doesn't cut the start sound (take08: Brutus's 6.1 s rumble plays on).
* Each switch sends a chatter tag (on h_1c0514bd; off with stamina left h_09087c9c; off at 0
  h_ee0a5cbc, the stamina tested at 0x10e40d; leads.md had the two off tags swapped) through the
  game's chatter gate, FUN_00140a60 (`skill_line`; the analyst's
  `todo/35-special-skills/answer-120-chatter.md`). Each `<chatter_line>` block gives it its
  values (`Game::chatter_blocks`, read with the parser's defaults, FUN_0019cd70), the same for
  the four: `probability` 50, `replay-delay` 10 / 20 / 30 s, `priority` 1 / 1 / 5. In the code's
  order:
  * **Replay delay** (FUN_001263e0): once a character has said the tag it can't say it again for
    its replay-delay (started when the line is said, FUN_00140cd0 -> FUN_00126460; counted down by
    the character update, FUN_0011c100). Per character: Hawk's line doesn't hold Tex's.
  * **Distance**: beyond 45 m of the player's character no line (FUN_001409a0); beyond 15 m a step
    of priority less (FUN_001409c0), so a squadmate more than 15 m away never says the on and off
    lines (priority 1 -> 0, refused). The player's own lines are at 0 m.
  * **Condition** (FUN_001372c0, FUN_00137360): the block's condition value must lie in one of
    its variants' ranges (each sound_id's h_0ab96f95 / h_16087610, 0..100 when not given). The on
    line's kind is the default (0): any variant. The off line with stamina left tests char+0x5a8,
    which only ever holds 0 in default.xbe: Tex's and Brutus's variants need 2..100, Hawk's
    1..100, so only Flint (no ranges) says it. The line at 0 tests FUN_0001b460, read as the
    enemies the character is fighting (inferred): Hawk's variants need 1..100, so she says it only
    in a fight. The demo counts 1 while a hostile is in the speaker's sight (`hostile_in_sight`):
    **a guess**; the main game has no enemies yet, so she never says it there.
  * **Roll**: `probability` % (FUN_001374c0): 50. `BF_SKILL_CHATTER=<percent>` sets it.
  * **Variant**: picked at random among those that fit and haven't been said yet; when all have,
    the list starts over (FUN_001371f0; per character and tag, `LineHistory`, one per map).
  * **A dead speaker is silent** (FUN_0013fab0): the line of the death's off isn't heard.
  * **The queue: a stand-in.** The game has one chatter queue for everyone: a playing line of the
    same or lower priority makes a new one wait (up to the block's 5 s), a waiting line of the same
    or higher priority refuses it (FUN_00141020, FUN_00140a40). The demo refuses a skill line while
    its speaker is talking or has a line due (its own `say()` check), and nothing else.
  * The line starts the next frame: the code has no delay of its own (the manager's update starts
    it), and the samples have no silence before the voice (0-0.07 s); the takes' 0.03-0.5 s from
    the switch to the voice isn't explained (a wait behind another line, inferred).
* What the takes heard fits: the on line 13 times in 26 for Tex, Hawk and Flint (50%); the off
  line 3 in 6 for Flint and never for the others (0 of 17); the line at 0 never for Hawk (out of a
  fight). Brutus's on line was never heard (0 of 9): the code doesn't explain it; his 6.1 s start
  rumble may have hidden it (answer-120-chatter 1.6). The other lines (`hurt`'s, the follow line,
  the death cry) still go through `say()` without the gate: a ticket of their own.

## HUD

The stamina bar is the game's HUD bar widget (base FUN_000b6cc0, stamina subclass FUN_000b7bf0;
the health bar is the same base). `StaminaBar` in `play_hud.rs` keeps its state:

* **Length**: value / 150 × 249 units (FUN_000b7050), the value stamina / max-stamina × max
  health (FUN_000b7fb0), at least 1 when not 0, and the bar at least 2 units above 0: 1.66 units
  per hit point from x 73.0, as long as the health bar at full (see play.md's HUD frame).
* **Brighter while it changes**: the fill (h_e11c7d04) is mixed toward white by a brighten set to
  1.0 on a frame the value falls and 0.5 on one it rises (FUN_000b7850), fading by 0.5 a second
  (FUN_000b75a0). So it's white while a skill drains it, pale while it refills, white for a frame
  after a drop, and pale fades to teal in 1 s once full. The mix is in sRGB values, as the
  console's: the fill image is remade with it (Bevy would mix colours in linear light). Measured
  (rows y 63-67 near the start): white 248-254 against the takes' 246-255; pale (144, 187, 203)
  against take01's (140-142, 182-183, 201).
* **Trail**: when the value drops, the lost part shows in the plain teal texture from the new end
  to a shown value that follows the value down at 12 bar units a second (19.9 screen units) with
  no delay (+0x60 0 for stamina; the health bar's is 1.0, not done here). It follows a rise at 8,
  but nothing is drawn then. A steady drain shows none.
* **Low-stamina blink** (FUN_000b76c0): while the value is at most 15% of the max, a triangle
  wave between -0.5 and 0.5 at 1 a second, reset heading down whenever the value is above; the
  frame's lower half (the stamina widget's own frame, from the tutorial picture's row 30, y 57) is
  tinted while it's below 0: red for 1.0 s, normal for 1.0 s, red first. The tint is a red copy of
  those texels (their luminance in red, 0.15 of it in green, no blue, their own alpha), which
  matches the takes' red-phase pixels (~ (L, 0.15 L + 14, ~24) of the normal pixel's luminance;
  the game's own tint, bytes a0 20 20 ff at 0xb720a, combines in a way not traced). The bar
  channel's inside and the fill stay as they are.
* **The skill's name** (Berserker, Vengar, Stealth, Targeting: each skill's string id) under the
  bars in the message blue, from the switch-on frame, while on; after the off it stays 0.15 s
  (measured 0.13-0.18 s; what holds it in the code isn't traced) and fades at 3 a second (the
  widget's 3.0 at 0x39a848). The fade's alpha is raised to 2.2: the console fades in sRGB values,
  Bevy in linear light. It goes 20 units lower while the "switch to" hint shows (the code moves it
  down for each other message widget showing; which ones count, and the hint among them, is a
  guess: the takes never show the two together).
* **Control passing** resets it to the new character's at once: no trail or brighten carried
  over, the name shown or hidden (take19 4.867 s, 14.867 s). Death: the name fades as at an off,
  the bar stops changing.

## The skill interface (#121-#124)

`SkillKind` is a skill's own part, as the game's skill classes override their base class's
vtable; the four are `Berserker`, `Vengar`, `Stealth` and `Targeting` in `play_skills.rs`, made by
skill-set (CSKILL_ 3, 1, 4, 2) when the character spawns. Each method's default is the base
class's; the skill is taken out of its character while they run, so they get the character
(`u.skill.on`, `u.stamina`, its weapons...):

| Method | vtable | Default | |
|---|---|---|---|
| `name` | +0x4c | | the string id the HUD shows |
| `toggle(u, out)` | +0x04 | | `Refused`, `Now { overlay }` or `AtEvent(event)` |
| `drain(u, cost, dt)` | +0xbc | `cost` × dt | Tex's ×0.5 with the gun in hand empty, Brutus's charge lump, Hawk's blade ×5 |
| `on(u, out)` / `off(u)` | +0x120 / +0x124 | nothing | the skill's effects start and end |
| `update(u, dt)` | +0xc0 | nothing | every frame, on or off, alive; true sends the toggle command: the skill's own ends |
| `ended_by_actions` | +0x7c | false (0x20e200) | true (0x213690, Tex's and Flint's): while on, the switch key, the reload key and the black button end the skill instead (`skill_input`) |
| `lets_overlays_play` | +0xa4 | true (0x262a80) | false (Tex's 0x13d850): while on, no other overlay starts (`overlays_blocked`: the reload, the switch, the throw, item use) |

`toggle` and `on` get `Outside`: what a skill reaches beyond its character, the squad's ammo
`Reserve` (Tex's top-ups) and the log switch.

Shared, not per skill (FUN_0010e470 / FUN_0010e3e0): the 10% check (`can_activate`), the on
flag, the start / end sound for the controlled character, the chatter line. `free_to_act`
(FUN_00117210) is what Tex's and Hawk's toggles ask. A skill's effects outside the character (a
material, the post-process, the HUD marker, the AI's view of Hawk) read `Player::skill`
(`on`, `toggling`, `special_motions`) and the character (Tex's `Player::second`) from their own
systems. Other base-class hooks the tickets will want (the game's fire hook +0xb0, item use
+0x104, damage taken +0xd8, visibility +0xe4, noise +0xe8, damage dealt +0xec, the reticle's
+0xf8 / +0xfc / +0x100, the AI's hold fire +0x108) are added to the trait with the base class's
answer as their default.

Known limit the skill tickets will meet: a system outside the character can't reach a concrete
skill's own state (Hawk's level, Flint's target) through `Box<dyn SkillKind>`: it will need an
`as_any_mut` on the trait to downcast, or the state kept on `Skill` / `Player` where those
systems can read it (as Tex's second gun is, `Player::second`).

## Tex's Berserker (#121)

"Uses two weapons at once." (h_0bd0cbed). Tex draws the gun of his other weapon slot into his
left hand and fires both. `Berserker` in `play_skills.rs` (skill vtable 0x39d058, constructor
FUN_0013d440), the second gun's fire in `play_shots.rs` / `play.rs` (`Player::second`,
`SecondGun`), its motions in `play.rs` (`SpecialClips`); measured against takes 01-05, 21, 25
and 26 (spec Part 2).

* **The press** (his toggle, 0x13d490), either way, needs a berserk transition from the motion
  he's in (the current MOTION's transition for motion type 45, char+0x1fc's +0x534): in
  `ms_tex.xml` the stand, walk, run, side and back steps and the dash have one, so not crouched
  or getting up or down, in a jump, a dodge, a dive or a slide, nor knocked down. Switching on is
  also refused with a gun already in the second hand, scoped (char+0x7a8), when he isn't free
  (FUN_00117210), under 10%, within his 1.0 s timer, with no gun in the other slot, or when that
  gun's clip is still 0 after its top-up (take25 10.0 s: both guns and the reserve emptied,
  nothing happens). Switching off needs the second gun out and the timer run out. A refused press
  shows and plays nothing.
* **The top-up**: an accepted press switching on tops the other slot's gun up from the squad's
  reserve, min(clip size − clip, reserve) (0x13d643-0x13d692). take25: the Jax-iC 2 → 36 and the
  reserve 200 → 166 at the press, before the switch; the demo the same. A recharging gun (the
  LZRs) has no reserve and isn't topped up. "Out of Ammo" (h_ff471882, 0x13d6e2) can't show:
  the code refuses without a second gun before it gets there.
* **The motions** change at the press (the toggle asks for motion type 45, FUN_0012eab0, at
  0x13d6ca): idle, walk, run, the side and back steps, the jump and the slide become
  `Sc_w1_bz_*` (`SpecialClips`; prefix-type 0, the same with either gun in hand), back at the
  press switching off. The berserk set has no transition to a crouch, a dash, a dodge or a dive,
  so in berserk Z doesn't crouch, Shift runs (`bz_run`) and C doesn't roll. A crouch pressed in
  it is dropped, not kept for after the off (kept, he would drop into a crouch at the off): the
  demo's choice until shot T3 (crouch in berserk) shows the game's. `Sc_w1_bz_start` /
  `bz_stop` (upper body, 1.0 s) play over them (#120's toggle).
* **The switch** (0.50 s into `bz_start`, the event eaba3d48 at 0x118ae7, after the activate):
  the other slot's gun goes into his left hand (FUN_0014d090), held by its trigger grip at his
  `SECOND_HAND` hardpoint h_15ab6d29 (the inventory's second-hand holder names it at 0x14c7b8),
  and loaded from the reserve at once if it isn't full (FUN_0022daf0 with 0). The weapon box
  lists both guns with their names for `HUD_LIST_TIME` (take01: 2.750-5.417 s), then shows two
  rows, both in the held guns' orange: the gun in hand on top, the second gun below (take04 8.5
  s). At the off event (14ee2201, 0x1180bb) the gun goes back to its slot (FUN_0014d1a0) and the
  box lists the weapons again, the stowed one in blue (take01 18.767 s).
* **Fire**: the fire command sets both guns' fire flags (0x122621-0x122655), so the trigger fires
  both from the same frame, each at its own rate from its own clip, with its own cooldown, burst
  counter, accuracy and muzzle effect (`play_shots.rs`: `FireState::run` steps every gun in the
  same game frames). Each gun's shots fly from its own muzzle to the one crosshair's point and
  hit with its own damage (each `Shot` carries its gun's Damage, damage-type and ammo-type). No
  damage factor: berserk's firepower is the second gun. With the gun in hand empty the second
  fires on alone (take05), and a recharging second gun keeps recharging. A tap fires both: the
  press's request sets both guns' fire flags each frame until an animation event clears it
  (0x0f8408d5 at 0x117b12, in a clip not placed), and the demo keeps it for each gun until that
  gun's first shot (`FireState::pull`), so one still cooling down fires when it's ready. A
  minigun's barrels spin in either hand.
* **The crosshair's lock** reaches as far as the shorter-reaching gun, measured from its muzzle
  (FUN_000bf7a0 at 0xbf82a-0xbf8aa keeps the smaller of the two guns' FUN_00232520 x 0.7, and
  that gun for FUN_000bff40 at 0xbfb14): with the LZR-50 in hand (63 m) and the Jax-iC second
  (42 m), the Jax-iC's, from his left hand. The lock box and the centre ray stay the gun in
  hand's (play.md § The crosshair's colours).
* **The drain** (0x13d720): `stamina-cost` × dt, × 0.5 (game-options' `<combat-skills>` block
  h_04aa86aa, h_1469a0df, `CombatSkills::berserk_empty_drain`) while the gun in hand's clip is 0.
* **What ends it**, each through the toggle, so `bz_stop` plays and it goes off 0.5 s later: V;
  stamina at 0; the second gun's clip at 0 (his update FUN_0013d780, once the timer has run out);
  Q, the switch key (the game's Y: the controller sends the toggle instead of the switch,
  0x134db4-0x134dde); R, the reload key (the game's X: 0x134c50, when a reload could start,
  FUN_00120c50; the reload asked for stays, +0xa1 bit 2, and follows berserk's end); death. The
  game's X ends it when let go (0x134c2b-0x134c4a: held, but less than its hold time; take26's
  off came about 0.6 s after X came up), the demo's R on the press: the demo's choice (its R
  reloads on the press too). The black button (the medkit shortcut, 0x134367) ends it the same
  way in the game; the demo has no key for it. G (the game's LT) doesn't: it throws or uses
  nothing (take26 20.5 s).
* **While it's on** no other overlay starts (his overlay filter 0x13d850, asked for each queued
  overlay at 0x11a507): no weapon switch, grenade throw (the count stays) or medkit. And no
  reload: the character's reload (FUN_00120d40) returns at once while a gun is in the second
  hand, so an empty gun in hand clicks dry under the trigger (each gun's empty cycle clicks with
  its own empty sound) and reloads once berserk is over (take05: the reserve taken at 15.19 s).
* **No scope** from the accepted press to the second gun's stowing (his toggle only refuses a
  press made scoped, char+0x7a8): the demo's choice until shot T2 (the scope in berserk) shows
  whether the game lets him. Scoped, the second gun's muzzle effect washed the view purple and a
  shot in the start clip left from behind his shoulder.
* Handing control over, as for every skill: it stays on, the AI fires both guns, and the stamina
  drains (take19's rule).

## How it was checked

Captures on the flat floor (`BF_MAP=flat BF_TEST_GOTO=0,4,0,4`, campaign rates) and sdm_e34,
`BF_CAPTURE` at 15 fps (60 for the fades), with `BF_SKILL_LOG`, against the takes' watch logs
and frames (scripts in the developer's scratchpad: rates fitted per state, the frames cropped to
the HUD at the same scale).

**Rates** (per s; the takes' logs per second of game time, all 0.8-1.4% above the data, the
rig's time base):

| | data | takes | demo |
|---|---|---|---|
| Tex on / off | -2.2 / +1.8 | -2.231 / +1.822 (take02) | -2.200 / +1.800 |
| Tex crouched, sprinting | +3.6, +0.45 | +3.655, +0.456 (take02) | +3.600, +0.450 |
| Brutus | -2.0 / +1.2 | -2.020 / +1.211 (take07) | -2.000 / +1.200 |
| Hawk | -2.5 / +0.6 | -2.527 / +0.605 (take11) | -2.500 / +0.600 |
| Flint | -3.2 / +0.8 | -3.230 / +0.806 (take17) | -3.200 / +0.800 |
| Tex on sdm_e34 | -6.5 / +1.2 | -6.555 / +1.215 (take21) | -6.50 / +1.20 |

**Timings**: Tex on 0.53 s after the press frame at 15 fps (0.50 at 60), off at 0 0.53 s after
the frame it reached 0 (take02: 0.51-0.56); Hawk on 0.87 s after the press frame at 15 fps;
Brutus and Flint in the press frame, off in the frame of 0, Brutus with no overlay; 9.9 refused
and 10.0 on (the poke and the press in one frame; take03's 9.2 / 11.2). The AI's Tex at 0 after a
hand-over: his stop clip, off 0.53 s later, no sound, its line heard (`BF_SKILL_CHATTER=100`).
Death with Vengar on: off in the frame of death, the end sound, the stamina frozen. V while a
grenade charges (`BF_TEST_THROW=1.0,1.0 BF_TEST_SKILL=1.5`): refused, the grenade thrown. A
knock-down mid-sprint (`BF_TEST_SPRINT`, `BF_TEST_KNOCK`): x1 while down.

**Chatter** (`BF_SKILL_CHATTER=100`, presses at 1, 6 and 9 s, `BF_TEST_STAMINA=2@13`): each
character said the on line at 1 s and was refused it at 9 s (its replay delay, 1.9 s left); the off
line with stamina left at 6 s was said by Flint only (Tex, Brutus: no variant for 0 in 2..100;
Hawk: 1..100); the line at 0 by Tex, Brutus and Flint, not Hawk (no fight).

**HUD**, side by side with the takes at the same scale:
* Bars at full: x 73.4-263.9 for Tex, 73.4-247.3 for Brutus, as take01 / take06.
* take03 (pokes to 9 and 11, presses at 2, 7, 11 s): the trail's end within 1.4 units of the
  take's at 3, 4.1, 5 and 6 s (245.3 / 221.3 / 205.3 / 185.3 against 243.9 / 221.9 / 204.6 /
  184.6: 20.0 units a second, the take's 19.8), the fill's end within 1.3, white then pale, the
  name, the see-through channel; the blink's red phases from 2.000 / 4.067 / 7.000 / 9.067 s,
  1.00-1.07 s each (the take's 2.050 / 4.117 / 7.017 / 9.050, 1.00-1.05 s; the take's picture
  lags its log by 0.13 s, and at 15 fps each half of the wave is whole frames, 0.53 s).
* take02 red phase: the frame's lower half (41-58, 13-21, 14-36) in the take, (36-63, 4-7, 0-13)
  in the demo over a black sky (the take's green and blue are the wall behind).
* take01 off: pale in one frame; the name full 0.15-0.17 s, then gone 0.33 s later (the take: 0.13
  s, then 0.30 s).
* take25: pale to teal linearly in 1.0 s once full (the take 1.0 s).
* take19: at a hand-over Hawk's white bar and "Stealth" become Tex's full teal bar in one frame.
* The name's letter cores span x 56.7-130.0, y 89.6-103.6 ("Berserker"; take01 56.7-130.0, 90.0-103.3);
  "Vengar" 56.7-108.0 against take06's 55.3-110.0 (the demo's font isn't the game's).

**Guessed or open**: the chatter queue (a stand-in) and the fight count for the line at 0 (a
guess); Brutus's unheard on line; what holds the name 0.15 s; which message widgets push the name
down; the blink's colour operation (the red copy matches the takes' pixels); the ×4 refill;
Brutus and Flint at the 10% threshold (code only); the demo's choices under "The toggle"; the
demo's frame is ~3 units longer on the right than the game's (the tutorial picture's proportions,
play.md).

### Tex's Berserker (#121)

On campaign e34 at the takes' range spot (`BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-53.69,31.29
BF_TEST_WEAPON="Confed LZR-50;Jax-iC"`, `BF_SKILL_LOG`, `BF_AMMO_LOG`, 15 fps captures), against the
takes' watch logs and frames:

* **take01** (`BF_TEST_SKILL=2,18`): on 0.53 s after the press frame and off 0.53 s after the
  second (the takes' mean 0.537 / 0.527); 100 → 64.80 over the 16 s on (2.2/s; take01 65.52 at
  18.64). The weapon box: one row before, both guns named for 2.5 s from the switch, then two
  rows; the list after the off with the Jax-iC blue, as the take's.
* **take04** (`-- --test`, an e34 Shrike Heavy at the take's place `BF_TEST_NPC="0.5,shrike
  heavy,enemy,dummy,-55.39,37.57" BF_TEST_NPC_HP=1.0,1,999 BF_TEST_INSTANT_KILL=0
  BF_TEST_TRIGGER=2-5,9.5-12.5 BF_TEST_SKILL=7,15 BF_COMBAT_LOG=1 BF_SHOT_LOG=1`): both guns' first
  shots in one game frame; the LZR-50 every 8-9 frames (0.27-0.30 s, 10 shots in the 3 s: take04
  0.292 s, 10 shots), the Jax-iC every 16 (0.533 s, 6 shots: take04 0.516 s, 6). Each gun hits
  with its own damage, as in its normal fire: the Jax-iC 32.9-33.4 on a calf (the take's 30.1-35.2,
  its calf hits) and 63-69 on a thigh, the LZR-50 3.5-4.9 on a calf (the take's 2.8-4.9) and
  6.6-10.3 on a thigh; the burst's damage per second is the two guns' sum (take04's 87.6/s is ten
  LZR hits and six Jax-iC hits at the calf values). In the frames the LZR's orange bolt leaves his
  right hand and the Jax-iC's purple ball his left, both to the crosshair (take04 9.67-9.90 s);
  from behind, the berserk idle's pose as take04 8.5 s's (both guns held out in front, their
  muzzles showing either side of his shoulders from the raised camera; from the front, which no
  take shows, the guns are at his hips).
* **take05** (`BF_START_WEAPON=1 BF_TEST_CLIP=99,4 BF_TEST_SKILL=2,14 BF_TEST_TRIGGER=4-10`): the
  LZR-50 drawn; nothing topped up (it recharges); the drain 2.21/s, then 1.10/s once the Jax-iC is
  empty (~5.8 s; take05 5.84 s, -2.22 / -1.12); the Jax-iC clicks dry (16bbf95c), no reload while
  on; the LZR-50 fires on alone; off at 14.67, the reload at 15.20 with the reserve 200 → 164
  (take05: 15.19 s, 164). The weapon box frame by frame with the take's: "4 / 200", the two rows
  named, "0 / 200" red over "92 Regen" at 6.2 s (the take the same), 78 / 80 Regen at 10.2 /
  13.0 s (the take 79 / 81), "0 / 164" then "36 / 164" with the list after the off.
* **take25** (`BF_TEST_CLIP=2 BF_TEST_SKILL=2,6`): the Jax-iC 2 → 36 (34 from the reserve) at the
  press, before the switch; the LZR-50 not topped up. With every clip and reserve at 0
  (`BF_TEST_CLIP=0 BF_TEST_RESERVE=0`): refused, "the second gun has no rounds".
* **take26** (`BF_START_WEAPON=1 BF_TEST_CLIP=99,20 BF_TEST_SKILL=2,10,18,24 BF_TEST_SWITCH=4.5
  BF_TEST_RELOAD=12.5 BF_TEST_THROW=20.5,0.15`): Q at 4.5 ends it (off 5.01, no switch: take26
  5.08-5.13 after Y at 4.5); R at 12.5 ends it (off 13.01) and the reload follows as the stop clip
  ends (13.52: 16 rounds, 200 → 184, the take's 184); G at 20.5 throws nothing, the count stays,
  berserk stays on to the press at 24. The other gun dry (`BF_TEST_CLIP=4,36`, trigger 4.5-8.5):
  the LZR-50 empty at 6.80, the toggle in that frame, off at 7.33 (take26: 30.23 → 30.73-30.78);
  the box "0 Regen" red, then the list with "Confed LZR-50 0 Regen" in blue, the Jax-iC firing on
  (as the take's).
* **Refusals**: crouched (`BF_TEST_CROUCH=1 BF_TEST_SKILL=3`: "no berserk transition"), scoped
  (`BF_TEST_WEAPON="L-Shot-75;Jax-iC" BF_TEST_SCOPE=1`, without `BF_TEST_GOTO`, whose autopilot
  doesn't read it), no rounds for the second gun (above).
* **Motions** (`BF_MAP=flat BF_AUTOPILOT=1 BF_TEST_SKILL=1.0 BF_MOVE_LOG=1`): from the press on,
  bz_idle, bz_run (the autopilot's sprint stays bz_run), the aimed side step bz_f_side_walk_r, the
  backpedal bz_back_walk, the dodge at 13.0 s a plain run, both jumps bz_jump_launch; the
  autopilot's Q at 21.8 s ends it and the normal set is back at the press. Aimed left
  (`BF_TEST_AIM=1 BF_TEST_STRAFE=-1,0`): bz_f_side_walk_l moved by bz_walk's root (1.3 m/s). No
  take shows these (spec shot T1).
* **Death** (`BF_TEST_DIE=3`): off in the frame of death with the end sound; both guns fall.
  **Hand-over** (`-- --test BF_TEST_SELECT=1,3.0` and a fighting enemy Brutus NPC): the AI's Tex
  stays in berserk, drains 2.2/s and fires both guns (the LZR-23's and the Minigun's hits, ammo
  types 11 and 2).
* **The lock's reach** (`-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.535,19.5
  BF_CAMERA_PITCH=-0.07 BF_TEST_NPC=0.5,tex,enemy,dummy,8,-30 BF_TEST_SKILL=2 BF_RETICLE_LOG=1`,
  the takes' loadout): the enemy 50 m out red with the LZR-50 alone (0.60 s: "locks", 53.6 m
  deep), blue from the Jax-iC's draw (2.60 s: "out of reach: 50.9 m from the muzzle, reach
  42.0"); the frames red at 1.3-2.0 s, blue at 3.0-4.0.
* **Taps** (the same spot, no NPC, `BF_TEST_SKILL=2 BF_TEST_TRIGGER=3.5-3.75,3.98-4.06,6.0-6.08
  BF_SHOT_LOG=1`): the hold's first shots both at 3.80; the tap at 3.98 fires the LZR-50 at 4.07
  and the Jax-iC, still cooling down, at 4.27, after the button came up (the one request this
  replaces went with the LZR-50's shot); the tap at 6.0 both at 6.13.
* **The scope** (`-- --test BF_NO_SQUAD=1 BF_TEST_WEAPON="L-Shot-75;Jax-iC" BF_TEST_SKILL=2,6
  BF_TEST_SCOPE=1,3`): asked for from 3.0 s, it stays out through berserk (the frames at 3.3,
  4.7 and 6.1 s from behind him) and comes in after the off event (6.67 s: in at 6.7-7.7 s).
* **The crouch** (the taps' spot, `BF_TEST_SKILL=2,5 BF_TEST_CROUCH_KEY=3.5,6.5
  BF_MOVE_LOG=1`): Z at 3.5 s in berserk does nothing (bz_idle); after the off (5.67 s) he stands
  on; Z at 6.5 kneels (crouching at 7.07, crouched at 7.53).
* **Regression**: one gun's fire is unchanged, against main (0f9e876): `BF_MAP=flat
  BF_AUTOPILOT=1` as Tex and as Brutus (`BF_CHARACTER=0`, the Bower's walked pellets), 480
  frames, and on the test map the LZR-50's held fire, taps and dry cycles (`-- --test
  BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.535,19.5 BF_TEST_WEAPON="Confed LZR-50"
  BF_TEST_TRIGGER=1-4,5-5.08,5.5-5.58,6.5-6.58`, and with `BF_TEST_CLIP=6`), 120 frames: the
  `BF_SHOT_LOG` and `BF_AMMO_LOG` lines are main's line for line once each pellet's new `hand:
  0, ` is taken out (the only change to them). The test map's frames are main's pixel for
  pixel; the autopilot's differ only where two runs of one build differ too: the hand-over
  chevrons' phase at 31 s, and in some runs a smoke cloud's particles at 30 s.

**Guessed or open (Tex)**: the berserk walk, run, side steps and jumps aren't in any take (shot
T1); the demo moves them by their clips' roots (bz_walk 1.29 m/s against the walk's 1.62), where
the game moves at the type's speeds. The scope in berserk (shot T2), a crouch in it and the black
button (T3) aren't filmed: the demo's no scope and dropped crouch are its choices meanwhile, and
the black button has no demo key. A tap's request lasts, in the demo, until each gun's first
shot; the game's until the 0x0f8408d5 event of a clip not placed, so whether a gun still cooling
down when it comes fires at all is open. After an off with a reload following (take05), the
game's weapon list goes 1.6 s after the off, the demo's holds its 2.5 s (`HUD_LIST_TIME`): the
weapon box's timer isn't traced. Whether the game's camera pulls in when berserk fires (the
demo's does, as for every gun; take04's doesn't) is play.md's question. A toggle overlay cut
short by a knock-down (the demo's choice) leaves the motions as the skill is.
