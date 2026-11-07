*Part of [levels/](README.md), the former page `levels.md`.*

* **Effect lights** (#272). A light effect's lamp (`light_barrelfire`, `light_muzzle`,
  `light_explosion`, `light_hit_*`, `light_phosphor` ...) is the same lamp, its range and
  colour the effect's newest particle's Size and Color curves (**Linked lights**: FUN_0021d270,
  0x2a1220), not its alpha and not its size again as a strength; lit above range 0.1 and a
  colour of squared length 0.000361, else out (`lamp_lit`), and out with no particle alive.
  ale_fx makes one bevy point light for each light effect (`EffectLamps`, `effect_lights`: made
  with the effect's first lit lamp, moved to the newest particle's place each frame, put out
  while its lamp is, gone with its effect, #308). Bevy draws it as the lamp: `lamp_light` gives
  the light the lamp's colour at an intensity of 4 pi and the lamp's range, and
  `level_scene::plugin` puts **src/lighting.wgsl** in place of bevy's lighting module
  (`PBR_LIGHTING_HANDLE`): a copy of bevy 0.16.1's pbr_lighting.wgsl whose `point_light` is the
  lamp (diffuse only, colour x (1 - (d / range)^2) x N.L: no 1 / d^2, no specular, which the
  game's lamp passes don't have either, no 1 / pi), so the scenery, the characters, the weapons
  and every other bevy-lit surface take the effect lights by the game's rule, per pixel (the
  game's skinned meshes per vertex, three lamps at most: not run here). The hand-made lights
  go the same way: the two flesh hit lights (play_fx.rs `Glow`) and the demo's own muzzle flash
  for guns whose muzzle effect names no light (play.rs `FLASH_LIGHT`: the iKhan-GPL's and the
  guns with no muzzle effect; the data has none there). Before #272 every particle was a light
  of its own, 1 / d^2, at colour x alpha x size x 30000 lumens (a fit to the old DNA effect's
  light), several overlapping ones for a fire (e01's barrel keeps ~7 alive, 0.3 m up inside
  the barrel, each reaching 4.7-7.3 m): whatever stood near burnt out to a flat disc.
  * **The data.** light_barrelfire (rate 3.7-6.6 a second, life 1.2-1.85 s, Size 4.71 -> 7.34,
    Color (0.69, 0.60, 0.26) -> (0.48, 0.35, 0.19) -> (0.74, 0.54, 0.33) -> (0.47, 0.33, 0.13)): the
    newest particle is 0-0.2 of its life old, so the lamp is (0.69, 0.60, 0.26) fading slowly
    to (0.6, 0.5, 0.23) with a range of 4.7-5.2 m, and the coals take at most that colour x
    their texture. light_muz_minigun (0.2 s): (0.81, 0.74, 0.35) -> (0.84, 0.48, 0.16) -> black,
    reach 0.28 -> 2.66 m; light_explosion (0.57 s): (0.99, 0.96, 0.80) held to a third of its
    life, then black, 4.63 -> 16.5 m; light_phosphor (the Light grenade, 5.995 a second, 0.514
    s): (0.655, 0.725, 0.847) -> (0.333, 0.620, 0.776), 18.8 -> 24.3 m. A lamp sitting on
    the ground (light_explosion has no offset) lights a flat floor by N.L = 0: only what stands
    up takes it. light_phosphor's emitter is 0.227 m up, so a flare on flat ground lights it by
    N.L ~ 0.18 / d at d m (the tester's figure), almost nothing a few metres out. No footage
    confirms that, nor the brightening above: the Light recording's ground beside its flare
    (+22 / +19 / +28 on the terrain, which the game lights through its dynamic list and the demo
    draws unlit, #181) is the one measure there is.
  * **Footage.** The recording of the Light grenade (todo/reference, "DNA + Weapon Pickups",
    the flare at 0-2 s, a still view; 2560 x 1440, 60 fps, 120 x 70 px of lit ground beside it,
    frames 10-124): the newest-particle rule gives a sawtooth, the colour falling as the newest
    particle ages and jumping back at each emission 1 / 5.995 s apart, of 0.105 of the lamp's
    red, 0.035 of its green and 0.027 of its blue. The ground's red series is that: it falls
    about 2.8 levels over each ~10 frames and jumps back (the autocorrelation of its detrended
    series peaks at lag 11-12 frames, 0.47, and 22, 0.62; the capture drops and doubles frames, so
    ~11 for the data's 10), its green and blue within 0.44 and 0.62 levels peak to peak. (The
    earlier "steady within 1/255", grenades.md, was a reading of those two.) Lights summed over
    the particles, as the demo's were, can't do it: the number lit stepped between 3 and 4.
    The demo's lamp on the same grenade: strength (the largest channel, linear) 0.66-0.69 over
    seven seconds (`BF_LINK_LOG=1 BF_LINK_AT=-0.28,-9.31`).
  * **Checked** (`BF_MUTE=1 BF_BACKGROUND=1`, fixed-step captures, the build before beside this
    one; scratchpad t272dev):
    * e01's fire barrel, the ticket's case (`BF_MAP=e01 BF_TEST_GOTO=-110.96,64,-114,50`, frames
      100 and 130; no e01 take exists, so by the code): the barrel's mouth (a 72 x 35 px patch)
      measured 238 238 182 on average, 79% of its pixels with a luma of 240 or more, a flat disc;
      now 122 91 52, none saturated: the coals' texture with its flame tongues over it, and
      Tex 2 m off warm on his side only.
    * The lamp by the numbers (`BF_TEST_LAMP`, the test map's floor, a lamp 1 or 2 m over a
      floor point, range 4, the floor's pixel under it measured as the light added against a
      capture without the lamp, linear): a lamp at 2 m adds 0.801 of what the lamp at 1 m does
      (the rule: (1 - 4 / 16) / (1 - 1 / 16) = 0.800; a 1 / d^2 light gives 0.25), a lamp of
      colour 0.5 adds 0.215 of colour 1's (linear 0.5 = 0.214).
    * #308's recipe on e01's barrel (`BF_TEST_HIT=3.0` ... `BF_LINK_LOG=1 BF_LINK_AT=...`):
      one lamp near it from 0.2 s, strength 0.34-0.44 (its colour's flicker from the newest
      particle's age), at the break 3.07 s still, "taken away with its object" at 3.20 s, none
      from 3.27 s: as before the change, one lamp where there were 6-8 lights.
    * m01_a's sconce (#308's recipe): 0.43% of the frame changed by more than 12 levels (the
      railing's wood near the torch, brighter by the quadratic falloff), nothing else.
    * Muzzle (the MK-ASLT hip-firing from m03_a's start, `BF_TEST_FIRE=hip`, frames 30-40),
      the Frag's explosion effect 2.2 m beside Tex (`BF_TEST_EFFECT=1.0,h_1fe9ae17,2.2,3.6`,
      frames 16 and 18) and beside a Filthy Mutant (`...,0,-4` with the Mutant 2.7 m off), the
      Fire Hound's fireball and blast (#83's recipe): the flashes and bursts are as before and
      the surfaces that face a lamp take it, a little less than before and never burnt out (Tex's
      side to the blast warm, slightly less orange than the build before's); surfaces facing
      away from it (Tex's back and the Mutant's visible side, the blast behind them) take N.L = 0
      and differ from the build before by under 1.5 levels (80 x 110 px regions, frames
      19-24). The ground, unlit, takes none, as before.
    * The Therm Sweeper's muzzle flash on Tex (`BF_TEST_GOTO=0,4,0,-10 BF_TEST_WEAPON="Therm
      Sweeper" BF_TEST_FIRE=hip`, 30 fps, frames 34-38 of the three builds: the build before,
      this one and this one with `BF_NO_EFFECT_LIGHTS=1`): the light adds +3.3, +3.6, +2.2, +1.9
      and +0.5 levels to the mean of a 280 x 370 px region round him (the build before's
      +3.4, +6.3, +12.5, +29 and +18: his back and legs lit gold by a lamp in front of him,
      not looked into); todo/73 take01, 170 x 120 px round Tex (the e34 gate wall 10 m
      ahead): his back doesn't change in the flash's frames (3.22-3.35 s: -0.8 to +0.1) and
      gains 1.6 to 5.4 in red alone (0.4 in green) over 3.38-3.48 s as the missile's glow
      grows, which the build before's gold doesn't match and this one's few levels do.
  * Hooks: `BF_TEST_LAMP=<x>,<y>,<z>,<range>,<r>,<g>,<b>[;...]` puts a lamp at each place from
    the start (play_lamps.rs); `BF_NO_EFFECT_LIGHTS=1` keeps every light effect's lamp out;
    `BF_LINK_LOG=1` with `BF_LINK_AT=<x>,<z>` logs the lamps lit within 3 m ten times a second
    ("1 point lights near, strength 0.40": their summed largest linear colour channel).
