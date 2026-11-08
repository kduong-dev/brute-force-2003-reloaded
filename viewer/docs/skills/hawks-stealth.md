*Part of [skills/](README.md), the former page `skills.md`.*

## Hawk's Stealth (#123)

"Turns invisible." (h_e0282878). Hawk fades out over a second and stays invisible while her
stamina lasts: the enemies lose her, the crosshair can't lock her, her footsteps go quiet; a
shot shows her to them for a quarter second and costs stamina, as an item does. `Stealth` in
`play_skills.rs` (skill vtable 0x39d190, constructor FUN_0013cf10) is her skill's own part; her
body's look is `play_stealth.rs`; who sees her, play_brain.rs (`Sensor`, the AI) and
play_reticle.rs (the crosshair). Her tuning is game-options' `<combat-skills>` block h_0ec919f1
(`CombatSkills::stealth`: only stealth-level-recovery-rate, h_f726177e, is named, by hashing).
Measured against takes 10-14, 18 and 19 (spec Part 4); her refraction (#198) against the takes
and leads in `todo/198-hawk-invis/`. **Gunthar Ghent's Cloak** (#205, [its page](ghents-cloak.md),
`play_cloak.rs`) is this skill without the clip: it switches the same invisibility controller
(`Skill::cloak`, her on and off 0x13c650 / 0x13c670 are in its vtable too) and is drawn by the same
`play_stealth.rs`, which asks `skills::cloaks` (`SkillKind::cloaks`, true for both skills) rather than her
skill's name; everything else on this page is hers alone (the clip, the request byte, the costs of a shot
at 2x and of an item, the hold fire, the x5 with the Power Blade).

* **The press** (her toggle, FUN_0013cf90) is a request: refused unless she's free
  (FUN_00117210, 0x13cfd2: her clip under way among the rest), no request is pending (skill+0x20,
  set by an accepted press and cleared by her next update), her level isn't mid-fade (skill+0x24
  exactly 0 or 1: the second her body takes to come back after an off at stamina 0, which has no
  clip, holds the key), and, switching on, at 10% (take11 52.0 s: nothing at 9.06). Accepted,
  `Sc_w1/w2_activate_stealth` (2.0 s) plays, on and off, and switches at its 0.83 s event.
  With the skill on at stamina 0 or dead: off at once, no clip (take11 15.26-15.31). Her start
  and end sounds and lines are #120's.
* **Her invisibility controller** (`Skill::cloak`, `Cloak`): her model carries the game's
  BF_InvisMaterial controller, kind 2 (FUN_00210060 gives the character type named "hawk" one,
  FUN_002101d0 makes it: the link leads.md had as likely is in the code). Her on and off switch
  it (0x13c650 / 0x13c670 through FUN_002107d0, its switch 0x210a90); its level (0x210b10) goes 0
  to 1 in 1.0 s after the on and back in 1.0 s after the off (its init 0x212380 sets both fades
  to 1.0 s; the takes: 0.05 a 50 ms log row), and her update reads it back each frame. Switched
  off while still fading in, it starts its fade-out from the top: the level jumps to 1 and comes
  down (the game's switch). The dead's runs on: the death's off fades her back in on the ground.
* **Her skin's controller** (`Skill::skin`, `Skin`, #198): every character's model carries a
  body controller (kind 1, vtable 0x394e18, the map's primary), on from the start
  (FUN_00210840); its alpha (0x212cd0) is how much of her own skin is drawn. Each frame the
  controller map (FUN_00210560) switches it off while the cloak fades in or is on and on
  otherwise, with the same 1.0 s fades, so her skin fades out linearly as her level rises and
  back as it falls (`skills::opacity`). Switched off mid fade-in (an off at once: stamina emptied
  by a shot, or death), the cloak starts a full fade-out and her skin, still fading out, is on at
  once (the switch takes a fade-out straight to on): her body snaps back while the refraction
  dies. For everyone else it stays on.
* **Visibility** (+0xe4, 0x13d200): 1 - the factor x the level. The factor (skill+0x28) is 1,
  0 after a shot, and climbs back at stealth-level-recovery-rate, 4 a second (FUN_0013d1a0). It's
  what the AI's sight and the crosshair read (the stamina update pushes it into her combat
  target each frame, 0x11a934). `BF_SKILL_LOG` prints the level and the visibility.
* **Her look** (`play_stealth.rs`): two layers, as the game draws her.
  * **Her skin**: her body, the guns she carries and what she holds (not the muzzle flash) at her
    skin controller's alpha, as a whole: the parts are blended over the picture and a depth-only
    copy of each puts her nearest surface in the depth buffer first (the game's own pair of
    passes, depth then colour, 0x394e18), so her arms and the gun in front of her don't show
    through her back. On screen her presence falls and rises linearly with the level (take11,
    see `skills::opacity`). The alpha is the console's blend in sRGB values (`srgb_alpha`), the
    parts' textures copied with their alpha at 255 (it's a specular mask).
  * **Her refraction** (#198): while her cloak is active the game copies the picture, after the
    see-through things and the decals (FUN_000e81e0: FUN_000e4da0 at 0xe877a), and draws her
    whole model and the guns she holds with BF_InvisMaterial over it (FUN_000e5240): each surface
    shows the copy at its own place on screen shifted by the cloak's offset x n.z x (n.x, n.y) of
    the screen, n its normal in world space (vertex program 0x3f), tinted (0.9 - 0.05 n.z, 1, 1)
    (in the stored sRGB values) and blended at the cloak's alpha (pixel shader 0x26). The offset is
    0.15 x her visibility + 0.1 x sin(2.5 t) while she's stealthed (FUN_00212430): a slow wobble of
    the picture inside her outline, 2.51 s round, still twice a round, at most 2.5% of the
    screen, which a shot strengthens for the quarter second her visibility takes to fall. Fading
    in, the tinted copy comes in as (2x)^2 over her thinning skin with no shift, then at 1 with
    a shift growing to 0.15 x her visibility; fading out, the shift dies as 1 - (2x)^2 from its
    last value (no wobble), then the tint as (1 - y)^2. Here each of her parts gets a twin drawn
    with `InvisMaterial` (src/invis.wgsl) while the cloak is active: bevy draws a material that
    reads its view's transmission texture in its Transmissive3d phase and copies the picture into
    that texture first, and `RefractionOrder` moves that phase after the see-through things, so
    the copy holds the terrain's blended layers, the decals, her fading skin and any smoke, as the
    game's does. The game's shift is a share of its 4:3 screen, 70 degrees across; the demo's
    view is the game's since #269 (55.4 degrees down, [camera.md](../camera.md)), so the shift is
    the same angle (`game_screen`: x1.333 across, x1 down, in the demo's half-heights; x1.69 and
    x1.27 when the demo drew 45 degrees down), the same against her body and what's behind her as
    in the takes. The streaky texture the material loads (h_ff43a70b) is never bound: the
    wobble is the shift. Fully stealthed she's drawn by the refraction alone.
  * Bevy builds a pipeline the first time it's drawn with and skips the draw meanwhile, so as her
    model first shows each part gets an invisible blended copy, and its twin is drawn at alpha 0,
    for a second. A part she takes up while stealthed (a grenade in her hand: its type's material
    is every such grenade's) gets a see-through copy of its material and a twin, its own back and
    the twin gone when it leaves her: the grenade's in flight is seen (take13 7.0-7.6 s, none in
    her hand before).
* **A shot** (+0xb0, 0x13d0a0; FUN_0022f640 calls it from the fire loop for each round, the
  demo's `skills::fired` in `step_player`): while she's stealthed the factor drops to 0, so the
  AI and the crosshair see her for about a quarter second, and her stamina loses 2 x the gun's
  Damage max (the Saryl-23's 13.5: 27; the Jax-iR's 34.125: 68.25). Her body stays invisible:
  her skin follows the cloak's state, not the visibility; only the refraction's offset reads it
  (+0x1c), so on screen a shot makes the ripple jump by 0.15 (up to 2.5x) and settle over the
  quarter second (#198 take07: no body at the shots). The skill stays on unless the cost
  empties the bar.
* **An item** (+0x104, 0x13d340; FUN_00121170 calls it as the use starts): while she's stealthed
  5 stamina, 15 for one placed or thrown (a grenade at the throw's release, a Roller or Sentry at
  the press, as the count drops), nothing for the Power Blade's melee. The medkit (IOU_USE_IN_HAND)
  costs 5.
* **Noise** (+0xe8, 0x13d210): her footsteps x (0.25 + 0.75 x her visibility), a quarter
  stealthed (FUN_00119020 at 0x11918e; play_liquids.rs `step`). The game's movement noise that the AI
  hears (FUN_0001aeb0) is scaled the same: the demo's AI doesn't hear.
* **The AI's sight** (play_brain.rs `Sensor`, #197; todo/197-ai-sight/answer.md): FUN_0004d140
  gives the AI the reach of its sight distance (the sensor's +0xa8, 0x10fb90: the type's
  `<ai vision-range>`, 75 for e40's Shrikes, 80 for Brutus, Hawk and Tex, 105 for Flint, or the
  placement's `vision` when its h_11cddd9f is set) x 1.15 for a target it already knows
  (FUN_000481c0) x the target's visibility, then its field of view and the ray. And the scan
  forgets a contact whose stealth visibility is 0.2 or under at once (FUN_00044140), after the
  step's sightings, so the 1.15 never holds her. So an enemy loses her as she fades: take14's
  Shrike, 15.7 m off with a vision-range of 75, stopped firing at her 0.81-0.86 s into her fade
  (visibility 0.20-0.17); the demo's at 0.87 s (5.87). It sees her again in a shot's flash and
  as she comes back. A fully visible character is seen out to the vision-range, and kept out to
  1.15 x it once known; the held gun's range limits only the shot.
* **The crosshair** (play_reticle.rs): a character less visible than 0.1 (FUN_000bf6d0) isn't
  locked, by the selection nor under the centre (FUN_000bf550): a stealthed Hawk on another
  side leaves the crosshair blue from 0.9 s into her fade.
* **An AI Hawk holds fire** (+0x108, 0x13d3f0): the AI's attack check (FUN_00024f80) refuses
  while she's stealthed: under the squad AI she follows and doesn't aim or fire (play.rs
  `update_player` gives her no target), and opens up once it's over.
* **The drain** (FUN_0013d030): `stamina-cost` 2.5 x dt, x h_00894f2a (1) with her second weapon
  slot's gun in hand; x5 with the Power Blade in hand, which the demo hasn't.
* **Hand-over and death** (#120's rule): handed over, it stays on under the AI, drains and she
  stays invisible (take19); dead, it goes off in the frame of death and her body fades back in.
  The gun she drops is fully seen from the frame it leaves her hand (#197): the game draws a
  held item through its holder's invisibility controller (FUN_000e7850 asks FUN_000e51e0 for one
  on the item or the character holding it), and a dropped gun has no holder
  (play_stealth.rs `fade_bodies` gives it its own material back at once; before, it stayed
  invisible through her fade and then popped in).
* **Not in**: what the enemies' goals do (take14's Shrike went RETURN_HOME when it lost her and
  INVESTIGATE at her shot, and didn't fire again: the demo's AI has no goals, so it stands and
  fires again once it sees her); the AI hearing her noise; the Power Blade (its x5 drain, the
  silent kill); the squad order's Toggle Ability (#33: `BF_TEST_SQUAD_SKILL` stands in); the
  radar's RCONTACT_INVISIBLE (not traced); her shadow's own fade (the game's follows the cloak's
  body alpha, FUN_002109b0: full for the first half of the fade-out, gone stealthed; the demo's
  is her skin's, which bevy cuts once its alpha is under 0.05). In the Vengar view
  (play_vengar.rs) a stealthed Hawk shows as a dim heat figure: the game's heat pass draws her
  with her controllers on (FUN_000e7d40's mask 0xfffffeff), and in a vision mode her cloak draws
  with the vision material (FUN_000e7320), not the refraction: the twins are hidden there. In her
  own scope her model, and the twins with it, isn't drawn (#198 take05).
