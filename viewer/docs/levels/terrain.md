*Part of [levels/](README.md), the former page `levels.md`.*

* Terrain vertices (type h_ef44f398) carry no float position. Slot 8 of the game's vertex
  format table gives the layout: SHORT2, packed normal, two float2 UVs, NORMSHORT3, 2 bytes.
  The terrain vertex shader (default.xbe 0x3d8bf8, program 0x38, disassembled with the NV2A
  instruction layout; this page had 0x3d50f8, which is no program) decodes the position:
  * |a| = row × 65 + column on a 2 m grid;
  * the signs of a and b give x and z;
  * |b| is the height in 1/16 m.

  Those scales come from run-time shader constants. I fitted them to the data:
  * the second UV set is exactly the grid ÷ 10;
  * heights land on the level's 26–54 m range;
  * placed objects sit on the ground.

  Terrain triangles wind the other way from objects'.
* Terrain layers: one geoset per texture. Layers after the first are blended in by their
  material's "alpha" mask. Each mask is a single 128 px image over the 256 m terrain (x and z
  from −128 to 128, v toward −z), the orientation that best matches where each layer's
  triangles are.
  * A layer is drawn over the ground and the layers under it (a depth-test bias of 10 each)
    and sorted before every other see-through thing (`TERRAIN_LAYER_SORT` in
    `src/level_scene.rs`, #73). Bevy sorts see-through meshes by their origin's depth plus the
    material's depth bias, and a layer's mesh is in world coordinates, so its origin is the
    world's: where that was nearer the camera than an effect, the layer was drawn after it and
    could paint the ground over any effect or decal behind its sort point. e40's
    cliff hid the Therm Sweeper's blasts and its launch glow from take04's view that way. The
    layer's material now carries its depth-test bias apart (`Lamps::layer_bias`, set in the
    pipeline by `Lamps::specialize`), since bevy takes both the sort and the test from the one
    `depth_bias`.
  * Checked: take04's view on e40 (`play.md`'s Missiles: with the impact on the reticle the
    blasts' bright area peaks at 25.4k units², the take's 16.9k, the explosion's own excess;
    nothing showed before); sdm_e34's autopilot run differs from main's by at most
    0.4% of the pixels before its first shot (#120's stamina bar); a Frag going off 6 m ahead at
    sdm_e34's start (`BF_MAP=sdm_e34 BF_TEST_EXPLOSION=1 BF_TEST_GOTO=0,0,0,0`) shows its fire
    from its first frame, where main drew the ground over all of it.
  * **The detail (#257).** Every terrain layer's material is of the terrain class
    (`h_0ec4edf0`), whose texture slots are `color`, `detail` (`h_17fc684a`), a P8 bump map
    (`h_ed0b519d`) and `alpha` (0x90bf0). On all 141 terrain materials of the game's levels the
    detail is the colour texture itself. Three constants place it (0x90e40; the tutorial's three
    layers, then the range over the 141 materials):
    * `h_08e8e4c3`, the detail's tiling over the colour's uv (3.5, 3, 2.5; 0-16, 112 of them
      3-5, the others 1 on two, 2 on four, 2.4, 2.5 on three, 2.8, 6 on eight, 7 on five, 9 on
      two and 16 on two);
    * `h_fe22cc04`, the distance from the eye it fades out at (20, 20, 15 m; 0-35, 117 of them
      15-25, 10-14 on nineteen, 1 on two);
    * `h_153264c6`, its share at the eye (0.9, 1, 0.99; 0-1, 105 of them 0.8-0.9, 0.95-1 on
      31, 0.4 on three, 0.7 on one).

    e05's `h_fa6eaf30` has 0 for all three: no detail, its colour alone.

    The material's begin (FUN_00090f50) binds the detail at stage 2 and hands these to the
    vertex shader as c20 = (1 / s², s², k, w). The base pass that reads them, vertex program
    0x38 with pixel shaders 0x1a (a layer) and 0x1b (the ground), samples the colour at the
    vertex uv and the detail at k times it, and mixes them by w (1 - d² / s²), d the distance
    from the eye, 0 past s. So within s of the eye the ground shows its texture k times finer,
    fading to the colour's own tiling at s. The demo drew the colour's tiling everywhere: within
    a few metres of the camera a 256-texel texture over 8 m (the tutorial's grass) to 18 m (its
    rock ground), magnified to blurry blotches.
    A layer's shader now does the mix (`level_scene::terrain_detail`, `src/lamps.wgsl` under
    `TERRAIN_DETAIL`), per pixel where the console's share is per vertex, and on linear values
    where the console's 8-bit combiner mixes the gamma ones (a small difference between two
    samples of one texture). The colour's own uv is the vertex uv, as before: the programs read
    the first uv set as it is (the vertex declaration at 0x3c9860 puts it at v9), and take03's
    wall-gate shot, where the cliff is past 20 m, shows the same rock features at the same size
    as the demo.
  * **The bump map (#271).** The terrain materials name a normal map in their slot h_ed0b519d
    ("normalmap" by its hash; the class is `BF_DetailTextureBlend`, h_0ec4edf0, described in the
    executable as "Terrain Detail-Bump-Alpha Blend"): 140 of the 141, all P8, 132 of them 256 x
    256, 3 a flat 2 x 2; mp7's h_1c7f8bd5 has none. A P8 texel is an index into a fixed palette
    of 256 normals in default.xbe (0x3ba968, FUN_00285780), which the demo now reads from the
    executable beside the data (`Game::bump_mips`; without default.xbe the ground has no bump).
    The normal maps are their colour textures' relief (the sand's ripples, the rock's cracks),
    and their mip levels are filtered and renormalised: the normals' spread falls from 0.31-0.44
    at 256 x 256 to 0.10-0.13 at 32 x 32, so the far ground goes flat.
    * What the shaders do. Program 0x38 hands slot 0's light of the pair the shaders hold (c16)
      to the bump map, turned into the vertex's tangent frame: oD1.rgb = q c16 q* x 0.5 + 0.5,
      q the vertex's quaternion (v11's three NORMSHORTs, w = 1 - 2 x v12.x's byte); oD0.w =
      the vertex's baked light where n . c16 >= 0, else 0; and lights the vertex by the ambient
      and slot 1's light, oD0.rgb = (c23 + max(0, n . c18) c19 c21) x that baked light. Pixel
      shaders 0x1a / 0x1b add max(0, (2 t3 - 1) . (2 v1 - 1)) x v0.a x c0 to v0, t3 the normal
      map at the detail's tiling (oT3 = v9 x c20.z), not faded, and light the colour and detail
      mix with the sum, clamped at 1.
    * The tangent frame and the baked light are in the vertex (`Geoset::frames`, `character.rs`
      TERRAIN_FRAME): checked on 3311 of e40's vertices, the quaternion turns each vertex's
      normal to +z (within 8 degrees on 58%, the rest from a normal of the frame's own), its x
      axis is the way the colour's u grows (cos 0.98-0.99 on the flat vertices of all three
      layers), its y the way v falls; and v12.y is the block's baked light cell (h_e2d79bad) at
      the vertex (98.8% within their four cells' values: the bright cells, 183-255, have the top
      bit, the cliffs' shadows, 35-86, don't).
    * Which light (#288). FUN_000dcc10 fills a pair's two slots by each light type's
      enable-specular bit (the type's +0x50 bit 3, which the light type copy sets at 0x14e3d5): a
      light with it takes slot 0 if that's free, else slot 1; any other takes slot 1 and moves the
      one there to slot 0; with both full the rest are passed over (0xdcc8e-0xdccf4). It meets
      the world's directional lights last to first: 0x14f2a0 puts each new one at the front of
      the world's list (+0x228; a point light goes to the back). The key types (h_e1cac6f7,
      h_157b3160, h_e8d6efe4) have the bit and the fill (h_f5302dc6) hasn't, so slot 0 is the
      pair's key whichever the level file lists first; of two keys or two fills it's the file's
      second; one key alone is slot 0 (mp3); one fill alone is slot 1, and slot 0 is black (no
      bump). (`Level::slots`, `bf::level::file_slots`; the lava's v0 is the same light,
      `Level::objects_key`.) The objects' programs light both slots alike, so only the bump map
      and the lava tell them apart. The outdoor pair is the one each view's draw starts with.
      * e40's key h_1a801d2e (0.85 0.66 0.59, from +x and 32 degrees up), sdm_e34's h_e7c392fd
        (0.82 0.91 0.99, 44 degrees up), sdm_e13's h_e7c392fd (0.96 0.87 0.80, 41 degrees up).
        Every level with an outdoor pair has a key in it (mp2 has none: both its lights are
        indoor).
      * #271 took the file's second light, the key only where the file lists it second. 31
        levels list the key first and drew the bump by their fill instead: e01, e13, e16, e23,
        m01_a/b, m02_a/c, m04_a/c, m05_b/y, m07_a/b, m09_a/b/c/x, m14_a/b, mp1, mp4, mp6, mp7 and
        every sdm map (sdm_e34's dim h_eec4a26b, 0.15 0.20 0.27; sdm_e13's h_eec4a26b from 10
        degrees up). mp3's lone key took slot 1: no bump (mp3 draws no terrain anyway, its mesh
        h_0eb16f84 being on no disc, nor does sdm_m07). The rest (e40, the tutorial, e05, e06,
        e10, e34, m02_b, m03_a/b/c, m05_a/c/d/x, m07_c, m08_a/b, m14_c/d, mp8) keep their light.
      * In the file's order the bump would take e40's fill, which comes from just under the
        horizon and fails n . c16 >= 0 on the beach: no relief at all, where take06 shows it.
    * c0, stage 2's constant (pixel constant 0 by the shaders' C0 mapping 0xfffff0ff; their own
      PSConstant0 is 0), sets the relief's strength. It's set nowhere for these shaders (nothing
      selects them, below); the demo takes slot 0's colour, the light the bump map stands for. A
      guess, not fitted: with it the near ground's fine contrast is the takes' to within about
      15% where the views match (Checked).
    * The demo (`level_scene::terrain_bump`, lamps.wgsl TERRAIN_BUMP) lights the ground by the
      baked light alone, which already holds the light as a flat normal gives it, so it adds the
      bump term's change from the flat normal's: the ground's light x (1 + c0 (max(0, N . c16) -
      max(0, n . c16))) where n . c16 >= 0, N the normal map's normal in the vertex's frame, n
      the vertex's normal, in the console's gamma terms (the linear light takes it to the power
      2.2). The baked light stands for v0.a, so the change is the console's own, whatever lights
      the rest of the ground (#181). A flat texel leaves the ground as it was; over a whole bump
      map the mean of N . c16 is mostly below the flat one, so the bumped ground comes out up to
      7% darker (e40's sand 3%, the tutorial's hillside 7%, its rock walls 0-2%, m03_a's sand
      under 1%), its hue the same (the patches' chromaticity within 0.002). The console's clamp of
      the sum at 1 is left out.
    * Taken over the whole vertex light instead, (v0 + c0 max(0, N . c16)) / (v0 + c0 max(0,
      n . c16)) with v0 from the level's lights, the relief came out as strong as the takes' against
      the demo's brighter ground, but slopes the key grazes went white in specks (the tutorial's
      rock behind Tex at take03's 24 s, 0.46% of the frame): there the console's v0 is dark
      where the demo's baked light isn't.
    * Not found: what draws the terrain's base pass. No code selects program 0x38 or pixel
      shaders 0x1a / 0x1b (below), and the live read (`todo/257-terrain-mips/live-read.md`)
      never saw 0x38 loaded in the tutorial's, e34's or e40's terrain scenes. The material's
      begin (FUN_00090f50) binds only stage 2 (the detail); its colour (stage 0), mask (1) and
      normal map (3) aren't bound by any of its methods (vtable 0x3a1b58; the stage-3 binds
      elsewhere in the executable are other classes', e.g. 0x96130's), and its draw (FUN_00091070)
      draws only the lamp pass. These are still the only shaders in the executable that read the
      terrain's colour uv, its tangent frame and its normal map, and the takes show the relief
      they give. A live read of what's bound and selected while a terrain block's base pass draws
      (a breakpoint on DrawIndexedVertices, 0x36ca20, with the terrain's stream: 0x3f7c58,
      0x3f7c68) would settle it.
    * Checked (`BF_MUTE=1 BF_BACKGROUND=1`, each against `BF_NO_BUMP=1`; the measure: luminance at
      480 lines, differences of Gaussians at 1-2, 2-4 and 4-8 px, their spread (in 8-bit luma)
      over a patch of ground, the take's patch beside the same ground in the demo; in brackets
      the take, then the demo without the bump):
      * e40's beach (`BF_MAP=e40 BF_CHARACTER=0 BF_TEST_GOTO=0,167.3,-40.6,147.9
        BF_TEST_GO_AT=100`, frame 15, against `todo/49-bower-20/take06` at 1.0 s): the sand right
        of Brutus 2.09 / 1.57 / 1.12 (2.18 / 1.68 / 1.08; 0.93 / 0.88 / 0.88), left of him 0.97 /
        1.55 / 1.38 (0.83 / 1.51 / 1.51; 0.45 / 0.50 / 0.49). The ripples are the take's shape,
        size and direction; the cliff's rock gains its cracks' relief as in the take.
      * The tutorial's opening (`BF_MAP=tutorial`, frames 162, 230 and 256 against take03 at 13,
        18 and 21 s, matched by the picture): the hillside 3.55 / 3.15 / 2.96 at 21 s (3.38 /
        3.30 / 2.98; 3.29 / 3.16 / 3.02) and 3.50 / 3.11 / 2.96 at 18 s (3.15 / 2.84 / 2.67;
        3.25 / 3.14 / 3.02); the rock wall 5.68 / 5.90 / 6.80 at 18 s (4.32 / 5.11 / 6.31; 5.47 /
        5.87 / 6.81) and 4.60 / 4.20 / 4.04 at 13 s (3.28 / 3.62 / 3.69; 4.46 / 4.19 / 4.07),
        brighter than the take's and so more contrasty without the bump already. The rock behind
        Tex (frame 330, take03 24 s) shows the take's fine relief, no specks (no pixel of the run
        both twice as bright and 40 luma brighter than without the bump).
      * m03_a's sand (`BF_MAP=m03_a BF_CHARACTER=0 BF_TEST_GOTO=-221.61,-113.87,-188.2,-51.8
        BF_TEST_GO_AT=100`, frame 15, against take01 at 1.0 s, the view not matched; the start is
        Brutus's spawn, where the run was made when it read `-182.6,-43.5`, which the spawn
        overrode before #292: the same view, his body now turned to the goal): 2.61 /
        2.46 / 1.69 and 6.50 / 6.03 / 5.48 (the take's 2.01 / 2.20 / 2.44 and 3.01 / 3.71 / 3.27;
        1.23 / 1.21 / 1.18 and 5.84 / 5.17 / 5.28 without); the ripples are there as in the take,
        on sand 3 times as bright.
      * Against the ground's brightness the relief is weaker than the takes' (e40's 5.8% against
        8.7%, the tutorial's hillside 6.8% against 7.5%): the takes' ground is darker (25 luma
        against e40's 36, 44 against the hillside's 52), #181's.
      * The far ground stays soft: e40's beach 20-40 m off 2.83 and 3.27 (2.80 and 3.30 without).
      * Unchanged: the HUD (sdm_e34's firing run, `BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20
        BF_TEST_FIRE=1 BF_TEST_HIT=2`, 75 frames: the two builds differ only on the ground, 0.7-2.6%
        of the pixels, some of it seen through the radar's glass), the run itself (the same
        shots, holes and counts), the ground's hue. The start views (`BF_CAMERA_PITCH=-0.25`,
        frame 30): sdm_e34 (its slot 0 the weak fill: little change), sdm_e40, e34, sdm_e13 (its
        rock more grained), sdm_m03 (little change); m03_a's (strong ripples, as its take01 shows).
      * The key by the slot rule (#288; `BF_LIGHT_LOG=1` prints the slots):
        * sdm_e34 by take20's orb (`BF_MAP=sdm_e34 BF_CHARACTER=3
          BF_TEST_GOTO=-81.78,63.86,-82.3,63.2 BF_CAMERA_PITCH=-0.6`, frame 19, against
          `todo/35-special-skills/take20` at 1.0 s, the view near but not matched): the rock right
          of Tex 1.70 / 1.38 / 1.12 at 59 luma (the take's 1.43 / 1.35 / 1.13 at 61; with #271's
          fill 1.18 / 1.11 / 0.99). The rock's grain shows all over the near ground, as in the take.
        * sdm_e13's shore (`BF_MAP=sdm_e13 BF_CHARACTER=0 BF_TEST_GOTO=-19.97,-122.80,-19.97,-110,25.09
          BF_TEST_GO_AT=100`, frame 19, against #146's take01 at 1.5 s): the ground left of Brutus
          3.47 / 3.30 / 3.11 at 51 luma (with the fill 3.41 / 3.63 / 3.50 at 56; the take's 2.67 /
          2.71 / 2.80 at 37, its ground darker: #181's), its relief dark grooves like the take's;
          the rock wall right of the lava 3.60 / 3.77 / 4.70 (2.60 / 3.32 / 4.48).
        * e40's beach (the recipe above, frames 10-19) and the tutorial's opening (`BF_MAP=tutorial`,
          frames 30-39): the same pixels as #271's build. Their slot 0 is the same light.
    * Test hook: `BF_NO_BUMP=1` draws the ground without its bump map, as before.
  * The layers' masks: program 0x38 samples a layer's mask at the second uv set (oT1 = v10,
    pixel shader 0x1a's alpha), which on every masked layer's vertex is its place across the
    terrain, 0 to 1 with v toward -z: the orientation the demo bakes into the vertex alpha (2 m
    apart, the masks' own texel pitch). The ground's geoset (pixel shader 0x1b, no mask) carries
    another second uv set (the tutorial's: the position / 3.2 m), unread.
  * Not found in the code: what selects vertex program 0x38 and pixel shaders 0x1a / 0x1b. No
    call of the program setter (FUN_000a4a60, 72 sites) or the pixel shader setter
    (FUN_000a4d00, 77) passes those indices: the immediates are other programs, and the eight
    computed program indices (0x97479, 0x9af13, 0x9cffb, 0x9e2df, 0xa38c1, 0xa6711, 0xa6837,
    0xba8d9) and ten computed shader ones (0x91c5f-0x91e4e, 0x9bad4, 0x9cbec, 0x9e56d, 0xa30ac)
    come out as others too. The two other program loads (0x21d683, 0x2a1d7a) load the ALE's
    0x3b8bf8, the tables hold the only pointers to the programs and shaders, and nothing points
    at those table slots or at the setters (#271). The live read (`todo/257-terrain-mips/
    live-read.md`) caught no 0x38 in three terrain scenes. The rest fits them: the begin
    computes c20's 1 / s², s² and w, which of the terrain's (declaration-8) programs only 0x38
    reads (c20 is the world matrix's first row in programs 0x2-0x19, 0x27, 0x3f, 0x50 and
    others); the masked layers' second uv set is the mask's coordinate; the vertices carry the
    tangent frame and baked light it reads (v11, v12); the terrain lights are `enable-bump`; and
    the takes show the finer tiling near the camera (take03 at 25 s, take08 at 45 s, the e40
    beach) and the relief of the bump map (above).
  * Test hooks: `BF_NO_DETAIL=1` draws the colour's tiling alone, as before.
