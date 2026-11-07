*Part of [skills/](README.md), the former page `skills.md`.*

## Brutus's Spirit of Vengar (#122)

"Heightens perception and regenerates health." (h_e051254d). While it's on Brutus heals, takes
less damage and kills by charging, and his view changes: the world grey and streaked toward the
middle, every living character a glowing heat figure. `Vengar` in `play_skills.rs` (skill vtable
0x39d678, constructor FUN_0013bef0) is his own part; the charge and the view, which reach beyond
him, are `play_vengar.rs` (with `src/vengar.wgsl` and `src/heat.wgsl`). His tuning is
game-options' `<combat-skills>` block h_f5dc6aac (`VengarSkill`; only `vengar-damage-multiplier`
is named). Measured against takes 06-09, 22-24, 27 and 30 (spec Part 3).

* **On and off** at the press ([the toggle](the-toggle.md)). His on (0x13c020) sets the vision mode
  2 (char+0x644 and the view's +0xd8), switches on his material controllers of kind 0x100
  (FUN_002107d0) and starts the view's colour fade; his off, FUN_00126df0(0), sets mode 0 and the
  controllers off, with the fade again when the view was Vengar's. The kind-0x100 controllers
  aren't in the demo: in his own view the heat figure covers him (the renderer masks that kind
  out while drawing the heat figures, FUN_000e81e0), and nobody else sees him yet.
* **Ordered, under the AI** (the Squad Command menu's Toggle Ability, #333): everything after the
  vision mode in his on, the view's mode, the kind-0x100 controllers and the fade, needs a
  player's view on him (char+0x5e4, the camera FUN_0006ff00 attaches to the character it
  follows, so the controlled one); without it 0x13c038 jumps past all three, and his off
  (FUN_00126df0) the same. So an AI Brutus in Vengar looks and sounds as before (his skill sounds
  are the controlled character's alone too, FUN_0010e7a0) and has its rules: the regeneration,
  the x0.85, the drain, the charge when he dashes into an enemy. The demo does the same (the
  view and the fade are the controlled character's: play_vengar.rs). No take shows an AI Brutus
  in Vengar: a shot of one ordered, seen from Tex, would confirm it. (His kind-0x100 controller,
  vtable 0x394df4, set up by 0x213630 with no fade, draws him again with the material
  h_1ce41323: a wrapper, glare key 180, of the shell material h_19a2b8bc, class h_0112da6f as the
  power-ups' shells, play_powerups.rs: texture h_fa3c2612, colour 0.55 0.53 1, translate-v -0.6,
  pushed out 0.02 m; materials-common.xml. It shows only where a view switched it on: after a hand-over
  from a Brutus in Vengar, to whoever looks at him until he's switched off with a view again.
  Not in the demo.)
* **Regeneration** (his update 0x13c0b0): h_eaa78fdf, 1 hit point a second while on, up to the
  max; none after the off. take07: 50.0 -> 65.04 over the 15 s on, +1.01/s. **Not in the demo:**
  take07 draws the health bar's fill pale while it rises (the HUD bar widget's brighten, the
  stamina bar's rule, [HUD](hud.md)); the demo's health bar has no brighten (nor trail) yet: a
  follow-up for the shared bar widget, not this skill.
* **Damage taken** x0.85 while on (`vengar-damage-multiplier` h_f4f602c6: his +0xd8, 0x13c140,
  which the damage handler 0x125600 multiplies in at 0x125747 after the difficulty step;
  play_damage.rs `Hit::victim_skill`), on every hit that reaches the handler: shots, blasts,
  damage areas, falls, a charge. take27, a Shrike's Saryl-45 at 13 m: 5.28 a hit off, 4.43 on.
* **The drain** (0x13c570): `stamina-cost` 2.0 x dt, and a charge hit's 100 the frame after it
  (`Skill::pending`, which his update clears).
* **The charge**, "Kill enemies by charging them" (FUN_0013c1c0, which his update runs every frame
  while he dashes, motion state 8, the demo's sprint, with the skill on):
  * the look ahead (FUN_00123080): a 2 m ray from his body along his facing, or 0.2 rad either
    side, must meet a hostile character (the team table);
  * then every character not on his side, alive and not on the ground, whose body is within
    h_e26c9e84 1.5 m and h_f3d82857 30 degrees of his facing is hit: h_14935eb6 200 as
    DTYPE_EXPLOSION (10) through FUN_00223160, a record with no hit part and no shooter's
    factors, then the victim's own (on STANDARD x1.4 on an enemy: take08's trooper 300 -> 20 HP).
    The hit draws the victim's reactions as any damage does (#128, #129);
  * on each victim, the effect type h_0571f62e names, h_0dcd4fc7: its sound h_e6a4d091 (0.81 s).
    Its ALE effect `vengar_hit` has a creation-time of 6.9e28 s, so it never shows, as in takes
    08 and 22 (`EffectType::starts`: an effect type's ALE effects now start at their
    creation-time, here the ones due at once);
  * his type's sound h_1629a90a, h_e4c94dfe (`SkillData::hit_sound`, played as his skill sounds
    are), and his charge line h_13601f7b through the chatter gate (Brutus's block: probability
    60, replay-delay 15 s, priority 5);
  * a kill throws the body (FUN_000e00b0 queues a shove on its ragdoll: along his velocity across
    and h_f5dae098 2 up, h_fa20e794 75 and h_0e406b26 2 as its magnitude and factor, which
    FUN_00127c60 applies as min(0.5 x 75 x 2, 75) = 75 N s on every body, #280; ~15 m/s for the
    whole body: [ragdoll.md](../ragdoll.md)). On the test map the body leaves at 13 m/s; against
    e34's gate panel 3.8 m behind it's there in 0.3 s and slides down it, as take22's trooper
    against the tower 3-4 m away;
  * the hit sets his pending cost to h_0461a627 100 (0x13c474): his next drain takes the rest of
    his stamina and the skill ends at 0 (takes 08, 22: stamina 0 and off the tick after the hit),
    so one charge per activation. Running into an enemy (take23) or dashing with the skill off
    (take24) does nothing.
* **The Vengar view**, while the controlled character's Vengar is on (the renderer takes the
  vision mode from his view, FUN_000e81e0):
  * the world's two lights, the terrain's pair and the objects', are a fixed white pair from
    straight up and straight down in place of the level's, the ambient kept (FUN_000dcc10 in a
    vision mode: colour 1.0, the table at 0x41b3a8): floors lit, walls by the ambient alone
    (take08: the sunlit tower dark, the shaded sand light). The demo puts the level's directional
    lights out and lights the world with that pair (`vision_lights`), and the terrain per vertex
    with it and the terrain's own ambient, unlit (`vision_terrain`): the game's terrain takes its
    light from its pair, not from the baked map the demo's normal view draws it with. The pair's
    strength, `VISION_LIGHT` 0.35, is a fit: the takes' Vengar views are lit at about a third of
    the code's white (worked back from takes 06, 08 and 09);
  * then the world pass (FUN_000e66d0; its vertex shader 0x6d and pixel shader 0x45 are
    `vengar.wgsl`'s): four taps of the picture zoomed toward the middle (1, 0.985, 0.97, 0.955),
    blended, made grey by the dot with (0.9, 0.9, 0.5), the darks tinted blue by (1 - (2g)^2)^2,
    and laid over the picture at 70%;
  * then the living characters as heat figures (the vision set's material h_eb7a6be1, class
    0xe2c39ac2, FUN_000a8610): unlit, coloured by the normal in view space (a sphere map), bright
    (k 0.8) for him and his hostiles, dim (0.2) for his friends, drawn by a second camera over the
    main one's picture (`HeatCamera`, `HeatMaterial`). The material's texture wasn't found: the
    colours are a radial ramp fitted to the takes (yellow facing the view, orange to red at the
    rims; dark orange for the dim). The texture scrolls by the material's time, and the takes'
    figures change colour with a period of 1.0 s, all in step: the ramp swings up and down by a
    sine of that period (`scroll`, a fit). The dead are drawn as the world is (take22's body).
    Round the figures a halo of their colour (the glare's 24 offset copies, FUN_000e47b0, at
    0.006 of the screen; its weight a fit). It is the glare (#289, levels.md "The glare"): the
    heat material keys it at 0 (0xa8c80, its byte +0x20), so every pixel of a figure is kept;
    the demo draws it on the heat camera with its own fitted weight, where the code's is 25 /
    255 a copy;
  * the HUD on top, unchanged.
  * **The colour fade** at each switch, on and off (FUN_000c6980: 0.6 s, level 1, colour ff 00
    00): the picture, not the HUD, times red, back to white linearly in 0.6 s (take06: 2.20-2.80 s
    and 20.17-20.80 s). Death ends the skill in its frame, so the view fades back from red there
    (take30). A hand-over to or from a character with Vengar on swaps the view without a fade:
    the demo's choice, not traced.
* **The same view machinery is the OrgSen's, vision mode 1** (#21, [pickups.md](../pickups/orgsen.md),
  "OrgSen"; `vision::mode` gives the view its mode, 2 for this skill's, 1 for an OrgSen on the
  face): the renderer's branch for mode 1 draws the same lights and the keyed terrain, its own
  world pass in place of the blur (a thermal palette), the heat camera's figures for every
  character, the dead too, in one yellow-to-orange look, and scan lines over both cameras, with
  no red fade at its switches (`VengarPass::world` 2, `scan`; `THERMAL`, `CARRIED`, `SCAN`).
* Not in the demo: his `spirit` overlay (never played from the button, #120); the ALE effects
  `vengar` and `vengar_heal` (who plays them for Brutus wasn't found, and the takes show neither);
  the health bar's brighten while he heals; his kind-0x100 material controllers.
