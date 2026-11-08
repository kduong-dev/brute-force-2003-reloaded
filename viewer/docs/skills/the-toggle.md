*Part of [skills/](README.md), the former page `skills.md`.*

## The toggle

V asks the controlled character's skill to toggle (`Skill::request`, read after the hand-over so
a press in that frame goes to the new character), unless its controls are off (#256): the toggle
is the player controller's command (0x134cff), which a script's hold switches off
(FUN_00129530), and in a cutscene no control reaches the character (`read_input`, play_script.rs
`blocks_input`), so V or `BF_TEST_SKILL` then is lost, not kept for later. Before #256 V toggled
the skill under the letterbox. An AI member's skill is switched by the Squad Command menu's
Toggle Ability (#333): its goal, GOAL_TOGGLE_COMBAT_SKILL (squad.md, "Toggle Ability"), sends the
same command every frame until the skill has switched (`Skill::ordered`; its refusals unlogged,
as the command at 0's), Tex and Flint standing up first (their skill's +0x11c, `stands_to_toggle`).
The command handler (0x1224a5) calls the skill's own toggle, which switches the skill
itself or not, and whose return picks the motion (0x1224e4): 0 none, 1 the type's overlay, 2
MT_TOGGLE_SPECIAL. In the demo (`Toggle`):

* **Refused**: nothing at all happens. No motion, sound, text or HUD change (take03 2.0 s, take11
  52.0 s; the low-stamina blink already running isn't a reaction to it).
* **Now**: the skill switches at once; `overlay` would play the overlay too (a toggle that
  switches and returns 1; none of the four).
* **At the clip's event**: the toggle returns 1 without switching, the overlay plays and the skill
  switches at its event (the game's animation event handler, FUN_00117660).
* **Stays** (#200): the toggle did its own work and returns 0 without switching: the Orbital
  Beam's (FUN_0013e910), which starts its effect, pays its cost and holds its name, and is never on
  ([Other characters' skills](other-characters-skills.md)).
* **Now after its own work** (#203): the Air Strike's toggle (FUN_0013b660) makes its call and pays
  *before* it asks the shared switch to go on, so the 10% gate is tested on the stamina left: a press
  at 20-29.9 calls once and stays off ([The Air Strike](air-strike.md)); pressed while on it is the
  plain off.
* **Motion** (#208): the toggle did its own work, switched the skill on itself (its FUN_0010e470) and
  returns 2: the handler asks for motion type 45, MT_TOGGLE_SPECIAL (`mov [esp+0xc], 0x2d` at
  0x1224ee), a **whole-body motion** of the character's motion set rather than an overlay. The
  Outcast Shaman's Wrath of Garner (FUN_0013f240): the effect asked for, the skill on at the press,
  his `spirit` pose for 1.5 s (`Toggling::pose`: the clip over the whole body, root included, the
  character held where he stands, `holds_still`), and the skill's own update switches it off when its
  timer runs out ([The Wrath of Garner](wrath-of-garner.md)).

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
| Tex | 0x13d490 | 1 | `Sc_w1_bz_start` / `Sc_w1_bz_stop`, 1.0 s | at the clip's 0.50 s event (eaba3d48 on, 14ee2201 off). A 1.0 s timer from an accepted press refuses presses meanwhile; switching on also needs him free (FUN_00117210, 0x13d5c4), 10% (0x13d5d3) and the rest of [his refusals](texs-berserker.md) |
| Brutus | FUN_0013bfe0 | 0 (0x13c00e, both ways) | none: his motion set's `Sc_w1/w2_spirit` never plays from the button | at the press |
| Hawk | FUN_0013cf90 | 1; 0 at stamina 0 or dead | `Sc_w1/w2_activate_stealth`, 2.0 s, on and off | at the clip's 0.83 s event (ead033a3); refused unless she's free (FUN_00117210, 0x13cfd2; her clip under way counts), no request is pending and her fade isn't under way ([below](hawks-stealth.md)); at stamina 0 or dead: off at once, no clip |
| Flint | FUN_0013cad0 | 0, or 2 with a rifle (motion type 45: her sniper motions, whose motion state is still 19 or 47, so the takes' logs never showed 45) | none | at the press; refused unless her motion has a type 45 transition, and switching on without a gun with rounds in hand ([below](flints-advanced-targeting.md)) |
| Outcast Shaman | FUN_0013f240 | 2 (0 while on, or refused) | none: the MT 45 MOTION `spirit` (`Sc_w1/w2_spirit`, 1.5 s), whole-body | at the press (its own FUN_0010e470); refused unless his motion has a type 45 transition, the overlay slots are free and the stamina is at least 33 x 3 = 99; off by its 3 s timer ([the Wrath](wrath-of-garner.md)) |

Brutus's and Flint's toggles don't ask FUN_00117210 (Flint's makes checks of its own).

The demo switches 0.50 s (Tex) and 0.83 s (Hawk) after the press frame (a clip started this frame
starts moving the next); the takes measured 0.53 s for Tex, the event plus about one 30 Hz frame,
and 0.89-0.97 s for Hawk less the rig's 0.02-0.06 s input delay.

* **Switching on** (FUN_0010e470) needs stamina ≥ 10% of the maximum (equal is allowed). Tex's and
  Hawk's toggles check it before their clip; Brutus's and Flint's switch is refused by it the
  same silent way.
* **Free to act** (`free_to_act`, the demo's FUN_00117210): alive, not knocked down, no grenade
  held (charging one: V then is refused instead of dropping the throw) and no reload, throw,
  medkit, weapon switch or toggle under way. FUN_00117210 itself tests the action slot +0x214,
  +0x225, the dead flag and the items held in +0x1fc / +0x200. The skill's own word ends it
  (+0x14, `lets_act`: false for Flint's targeting and the Wrath of Garner while on): those two also
  refuse the crouch command (#208: the key is lost) and, as the demo's simplification, the X hold,
  where the reload, switch and item use already asked (the game doesn't gate the hold: it defers or
  cancels the completed use, FUN_00120290 at 0x1207b9).
* **At 0** each skill ends as its toggle does: Tex plays his stop clip and goes off 0.5 s later,
  on at 0 meanwhile (the repeated commands refused by his timer; take02); Brutus, Hawk and Flint
  go off in that frame (take07, take11, take17). The Wrath's toggle answers 0 while on, so the
  command at 0 does nothing: it goes off by its timer.
* **What ends a skill**: the key again; stamina at 0; death (the death handler calls the
  deactivate, 0x115df5: in the frame of death, with its end sound and line); and each skill's
  own ends, from its `update` and the player controller (Tex's other gun dry, Flint's gun in hand
  dry, and for both the switch and reload keys; Brutus's charge hit, whose 100 stamina leaves him
  at 0: below). Being hit doesn't (take27).
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
* The squad AI never switches a skill on by itself (take32: 60 s of fighting; no personality uses
  goal 127, #35 leads §5); only the menu's Toggle Ability does (#333). A skill so switched on stays
  on under the AI until its stamina runs out, its own end, death or the next Toggle Ability.
