*Part of [levels/](README.md), the former page `levels.md`.*

* **Point lights.** A point light-object (every point type, #143) is a place, a colour
  (`h_f08eb2f3`, 0-1), a `range` (4-40 m) and a `falloff` kind (3 or 4 on all of them; the
  schema leaves its enum, `h_f30c3302`, unnamed; the other `falloff`s, explosions', are
  `DFALL_NONE / LINEAR / EXPONENTIAL / HALF_LIFE`). **The kind means nothing to the lighting**
  (#3, #272): the placement's handler keeps it at the light's +0x90 (0x14e88d), and none of
  the code that gathers or lights lamps reads that field (FUN_0009f0b0, the dynamic gatherer
  0x9ed80, the wrapper material's draw 0x9f3c0, the standard material's draw FUN_0009cc60, the
  terrain's FUN_001f5600, FUN_000dcc10: none reads the light's +0x90; the +0x90 FUN_0009cc60
  reads, at 0x9d2a7, is its draw context's, a stack argument's).
  * **The rule, every lamp alike (#272; read from the shaders, code-map.md).** A lamp adds to a
    surface **texture x colour x (1 - (d / range)^2) x max(0, N.L)**, nothing at the range and
    past it, d the distance from the lamp to the point (`level_scene::lamp_attenuation`). The
    scenery's lamp pass (FUN_0009cc60: additive, three lamps a pass, vertex program 0x50 and
    pixel shader 0x37) gets (p - lamp) / (2 range) + 0.5 as texture coordinates from the vertex
    program, whose first combiner takes the dot product of (2 t - 1) with itself, d^2 / range^2,
    and second takes 1 minus it, times the lamp's colour (x the material's) and the cosine the
    vertex program worked out per vertex; the terrain's pass (FUN_00091070: programs 0x51 and
    0x38) the same; a skinned mesh (a character) takes up to three lamps in its base pass, vertex
    programs 0x1a-0x35, the same term per vertex. **No 1 / d^2**: a lamp's colour (0-1 a
    channel) is the most it adds, at the lamp itself. FUN_0009f0b0 and the dynamic gatherer
    0x9ed80 weigh lamps ((1 - max(0, d - r) / range)^2 x the colour's largest channel x
    max(0, 1 - type+0x4c x age) x (cos x 0.5 + 1.5)) only to sort them, the best 64 three a pass;
    how many a draw gets falls from 24 to 3 as its distance from the eye goes from 20 to 50 m
    (not run here: all lamps in range count). Until #272 the demo drew the level's lamps with
    1 - d / range; it's the quadratic now (`src/lamps.wgsl`, `lamp_light_at`), 1 + d / range
    times as much (up to 2). The tester measured the lamp-lit interiors against the build before:
    sdm_m07 +11%, e16's indoor view +11%, m09_a's tunnel +18%, m07_a +0.1-2.5%, and no pixel
    darker. No take measures the lamps' brightness, so that is against the build before, not the
    game.
  * The game lights static geometry by them (the scenery meshes carry no baked colours), each
    object by the lamps of its pair only (FUN_0009f0b0 takes those with the pair's flag; one
    with neither, 32 on m09_c, lights nothing). Here the level materials do it per pixel
    (`LevelMaterial`: bevy's standard material plus `src/lamps.wgsl`, which skips the lamps of
    the other pair): each lamp's colour x `lamp_attenuation` x N.L, unshadowed, nothing past the
    range. Characters take the outdoor pair's lamps at their chest from every direction, half of
    it, as their texture added (emissive). The terrain takes none: it's drawn unlit, by its
    baked light map. The game adds lamps to its terrain in a pass of their own per block
    (FUN_00091070: additive, three a pass, vertex program 0x51), from the block's static list
    and the world's second dynamic list (0x638, count at +0xa38: the walk in FUN_001f5600 has
    no pair test, #272), so which static lamps light the ground there, and by which pair if
    any, is unknown (#181), and the effect lights lighting it (a muzzle flash, a blast) are
    missing here: the ground is unlit. Ammo Depot has 24, Cavern of Fire 22, Bulgar's squad map
    2, campaign maps up to 184 (m07_a). `BF_NO_LAMPS=1` leaves them out (to compare);
    `BF_LIGHT_LOG=1` lists them and each start point's nearest.
  * **The arithmetic.** The console works each lamp's term out in the stored (gamma) values and
    adds it to the picture in them: pixel = base + texture x colour x share. The demo works in
    linear light (a texture and a colour decoded, multiplied, added to the lit base, encoded:
    the same for a product, not for a sum), so a lamp's term reads weaker over a lit surface
    than the console's, by (base + term)^(1 / 2.2) - base^(1 / 2.2) against term (a term 0.2
    over a base 0.4 in stored values comes out as +0.04 instead of +0.2), and as strong or
    stronger over a dark one. The same for the directional lights and every other sum; not
    changed (#272): the barrel's coals, lit over a dark base, read right; a lamp on a
    brightly lit character reads dim. A follow-up if the lamps' look is to match the takes
    over lit surfaces.
