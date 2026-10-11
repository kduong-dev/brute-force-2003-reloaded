# Special skills: stamina and the skill toggle (#120)

Each squad member has a special skill that stamina (the blue bar) pays for: Tex's Berserker,
Brutus's Spirit of Vengar, Hawk's Stealth, Flint's Advanced Targeting. This page is the part the
four share (`src/bin/play_skills.rs`, the HUD in `play_hud.rs`): stamina, the toggle, its sounds,
chatter and HUD, and the interface each skill plugs into. Each skill's own effect is its own
ticket: #121 Tex, #122 Brutus, #123 Hawk, #124 Flint. So far a skill switches on and off, drains
and shows on the HUD, but does nothing else.

Everything here comes from default.xbe (the functions are in [code-map.md](code-map.md), "Skills
and stamina" and "HUD") and the data, checked against the 32 takes in
`todo/35-special-skills/` (the spec is `spec.md` there, Part 1).

## Keys and test hooks

| Key | |
|---|---|
| V | the controlled character's skill on / off: the game's White button (VB_SPECIAL_ABILITY, `game-options-en.xml` line 151, which sends CT_TOGGLE_SPECIAL at 0x134cff). Not while dead |

* `BF_TEST_SKILL=<s>[,<s>...]`: presses V at those times (once each).
* `BF_TEST_STAMINA=<n>[@<s>][,...]`: sets the controlled character's stamina to n at those times
  (at the start without one), as the takes' pokes: `30@1.9` with `BF_TEST_SKILL=2.0` is take02.
* `BF_SKILL_CHATTER=<percent>`: the skill lines' chance (100: every switch), to hear them.
* `BF_SKILL_LOG=1`: each toggle command (the key's, at 0, a skill's own) and its answer, each
  overlay and when it switches, each switch with its sound and line, and every squad member's
  stamina each half second with its state (`on`, or `off x` its refill factor).
* `BF_TEST_SPRINT=<s>` (play.md): sprints toward the `BF_TEST_GOTO` goal from that time, for the
  dash's refill. With `BF_TEST_CROUCH`, `BF_TEST_SELECT`, `BF_TEST_DIE`, `BF_CAPTURE`
  (`BF_CAPTURE_FPS=60` for the blink and the fades).

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
  the crouch walk and the kneel's way down and up (MT_TRANSITION, measured ×1) included. A ×4
  while a byte at char+0x5c is set is left out: nothing in the takes set it.
* Clamped to 0..max on every write. Full at spawn (and a respawn). The dead don't change (take30:
  Brutus's 94.11 stays for the 9.4 s after his death).

## The toggle

V asks the controlled character's skill to toggle (`Skill::request`); a squad order will too
(#33's "Toggle Ability"). The command handler (0x1224a5) calls the skill's own toggle, which
answers one of three ways (`Toggle`):

* **Refused**: nothing at all happens. No motion, sound, text or HUD change (take03 2.0 s, take11
  52.0 s; the low-stamina blink already running isn't a reaction to it).
* **Now**: the skill switches at once, and the motion set's toggle overlay plays if it has one.
* **At the clip's event**: the overlay plays and the skill switches at its event (the game's
  animation event handler, FUN_00117660).

The overlays are the motion sets' `<overlay motion-type=44>` (OMT_SPECIAL_ON) and `43`
(OMT_SPECIAL_OFF) (`Game::skill_overlays`, read from `ms_<name>.xmb`; `prefix-type` 1 puts
`Sc_w1_` / `Sc_w2_` before the script for the held weapon's stance). They're played as the
reload is, the whole body standing and the upper body on the move (`Loaded::skill_clips`,
`step_player`).

| | Toggle | Overlay | Switches |
|---|---|---|---|
| Tex | 0x13d490 | `Sc_w1_bz_start` / `Sc_w1_bz_stop`, 1.0 s | at the clip's 0.50 s event (eaba3d48 on, 14ee2201 off). A 1.0 s timer from an accepted press refuses presses meanwhile; switching on also needs him free (FUN_00117210) |
| Brutus | FUN_0013bfe0 | `Sc_w1/w2_spirit`, 1.5 s, at the on only | at the press |
| Hawk | FUN_0013cf90 | `Sc_w1/w2_activate_stealth`, 2.0 s, on and off | at the clip's 0.83 s event (ead033a3); refused unless she's free (her clip under way counts); at stamina 0 or dead: off at once, no clip |
| Flint | FUN_0013cad0 | none | at the press |

The demo switches 0.50 s (Tex) and 0.83 s (Hawk) after the press frame (a clip started this frame
starts moving the next); the takes measured 0.53 s for Tex, the event plus about one 30 Hz frame,
and 0.89-0.97 s for Hawk less the rig's 0.02-0.06 s input delay.

* **Switching on** (FUN_0010e470) needs stamina ≥ 10% of the maximum (equal is allowed). Tex's and
  Hawk's toggles check it before their clip; Brutus's and Flint's switch is refused by it the
  same silent way (no overlay plays then).
* **At 0** each skill ends as its toggle does: Tex plays his stop clip and goes off 0.5 s later,
  on at 0 meanwhile (the repeated commands refused by his timer; take02); Brutus, Hawk and Flint
  go off in that frame (take07, take11, take17).
* **What ends a skill**: the key again; stamina at 0; death (the death handler calls the
  deactivate, 0x115df5: in the frame of death, with its end sound and line); and each skill's
  own ends, from its `update` (their tickets: Tex's other gun dry, Y, holding the reload; Flint's
  clip dry; Brutus's charge). Being hit doesn't (take27).
* **Handing control over doesn't end it** (take19): the skill stays on and drains on the
  character you left, under the AI; it's on when control comes back. A clip under way plays on.
* A knock-down cuts a toggle's clip short, and the switch it carried with it.
* **One overlay at a time**: no reload, throw, item use or weapon switch starts during a toggle's
  overlay, and the aim doesn't turn the gun over it.
* The squad AI never switches a skill on by itself (take32: 60 s of fighting); only the order will
  (#33).

## Sounds and chatter

* On and off (FUN_0010e470 / FUN_0010e3e0) play the type's `start-skill` / `end-skill` sound
  (Tex ed7492f4 / e00d91fa, Brutus ee753485 / 190cd1f6, Hawk fa7509ca / f57b8dca, Flint 07168977
  / 1656217b) at full volume, in 2D, **for the controlled character only** (FUN_0010e7a0 asks
  FUN_00112790; take18: an AI Hawk's switches are silent). At the switch, not the press. The off
  doesn't cut the start sound (take08: Brutus's 6.1 s rumble plays on).
* Each switch sends a chatter tag, said through `say()` 0.1 s later (the takes: 0.03-0.5 s, mostly
  0.05-0.2): on h_1c0514bd; off with stamina left h_09087c9c; off at 0 h_ee0a5cbc (the stamina
  tested at 0x10e40d; leads.md had the two off tags swapped). The AI's lines are heard too.
* **The chance is a guess** (spec Part 1 section 4, open question 1): the chatter block's
  `h_ea21ae4b` (10 on, 20 off, 30 at 0) read as a percentage, as `play_pickups.rs` reads the
  medkit line's. The game's gate (FUN_00140a60) does roll a percentage against a field of the
  line's record (FUN_001374c0), and the parser stores this attribute at that offset of the record
  it fills (0x19ca9a), but they may not be the same record, and the gate also checks a condition
  per line (FUN_001372c0: the off line's needs a count at char+0x5a8 of 2 or more, not traced)
  and a time throttle. The takes heard the on line 13 times in 35, the off line 3 in 23 (all
  Flint's), the line at 0 2 in 9.

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
| `toggle` | +0x04 | | `Refused`, `Now { overlay }` or `AtEvent(event)` |
| `drain(u, cost, dt)` | +0xbc | `cost` × dt | Tex's ×0.5 with the gun in hand empty, Brutus's charge lump, Hawk's blade ×5 |
| `on` / `off` | +0x120 / +0x124 | nothing | the skill's effects start and end |
| `update(u, dt)` | +0xc0 | nothing | every frame, on or off, alive; true sends the toggle command: the skill's own ends |

Shared, not per skill (FUN_0010e470 / FUN_0010e3e0): the 10% check (`can_activate`), the on
flag, the start / end sound for the controlled character, the chatter line. `free_to_act`
(FUN_00117210) is what Tex's and Hawk's toggles ask. A skill's effects outside the character (a
material, the post-process, the HUD marker, the AI's view of Hawk) read `Player::skill`
(`on`, `toggling`) from their own systems. Other base-class hooks the tickets will want (the
game's fire hook +0xb0, item use +0x104, damage taken +0xd8, visibility +0xe4, noise +0xe8,
damage dealt +0xec, the reticle's +0xf8 / +0xfc / +0x100, the AI's hold fire +0x108, the overlay
filter +0xa4) are added to the trait with the base class's answer as their default.

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
Brutus and Flint in the press frame, off in the frame of 0; 9.1 refused (`BF_TEST_STAMINA=9@1.9`),
11.2 on (take03's sequence). The AI's Tex at 0 after a hand-over: his stop clip, off 0.53 s
later, no sound, its line heard (`BF_SKILL_CHATTER=100`). Death with Vengar on: off in the frame
of death, the end sound, the stamina frozen.

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

**Guessed or open**: the chatter chance (above); what holds the name 0.15 s; which message
widgets push the name down; the blink's colour operation (the red copy matches the takes'
pixels); the ×4 refill; Brutus and Flint at the 10% threshold (code only); the demo's frame is
~3 units longer on the right than the game's (the tutorial picture's proportions, play.md).
