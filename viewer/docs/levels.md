# Level viewer (`bf_level`)

`cargo run --release --bin bf_level -- [level]` draws a level from the original archives:
terrain, every placed object (game objects, pickups) at its transform, the sky, and the level's
fog, with the power-ups' icons and the level's placed effect objects running (its flies,
embers, fires, steam and dust: maps.md "Placed effect objects", #258). The default is `sdm_e34`,
the squad deathmatch map "Battle of Bulgar" (SDM Level 5 in
campaign-bf.xml). Right-drag looks around, W/A/S/D/Q/E fly (Shift for faster), 1-9 jump to the
level's own flyby cameras and 0 to an overview, F toggles fog, P prints the view. Test hooks:
`BF_LEVEL_CAMERA=<n>`, `BF_VIEW=x,y,z,yaw,pitch`, `BF_NO_FOG=1`, `BF_SCREENSHOT=<file.png>`,
`BF_LEVEL_DUMP=1` (placements, untextured materials, idle effects, placed effect objects),
`BF_NO_PLACED_FX=1` (no placed effect objects), `BF_PART_DUMP=<hex
archetype>` (each part's geosets: offset, bounds, normals, winding, UV direction; then exits),
`BF_TEX_LOG=1|squad` (where each texture came from, below).
In bf_level and bf_play: `BF_NO_DETAIL=1` (the terrain without its detail texture),
`BF_NO_BUMP=1` (the terrain without its bump map), `BF_ANISOTROPY=<1-16>` (the world textures'
anisotropy; the game's is none), `BF_SKY_TIME=<s>` (the sky's clock started that far into the
level), `BF_GLARE=0` (no glare) and `BF_GLARE_DEBUG=1|2` (the glare's keys, or its blur alone),
all below.

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
      * m03_a's sand (`BF_MAP=m03_a BF_CHARACTER=0 BF_TEST_GOTO=-182.6,-43.5,-188.2,-51.8
        BF_TEST_GO_AT=100`, frame 15, against take01 at 1.0 s, the view not matched): 2.61 /
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
* **Which copy of a texture (#284).** A texture is found by name in the loaded libraries,
  common's first, then the level's own `textures-<lvl>` (every level file lists its texture
  libraries as `common <lvl>`; FUN_000d61a0 loads them in that order, after common's from boot).
  `Game::load_level` reads the level's library and its entries replace any found before but
  common's; a name neither holds is searched for in the other archives in name order (e01 first),
  as before. The copies matter: of the 1613 texture names, 29 have copies that differ between
  libraries (byte-hashed over every archive): the squad's eight skins (five outfits,
  character-types.md "The squad's outfits") and 21 world textures with 2-12 variants (h_f7bcbdb1
  in 53 libraries, h_12452ad9 in 46, h_0578cf17 in 14 ...), named by the levels' own materials.
  Before #284 each was e01's copy, or the first archive's by name, on every level. None of the
  29 is in common, and the 20 names common shares with the levels are byte-equal everywhere.
  * A multiplayer map (a deathmatch arena or a squad deathmatch map in the level list) adds the
    match's library from mp_common after the map's (`Game::load_match_textures`, from bf_play's
    `load_map`): `dm` in an arena, the Brute Force squad's `sdm_bf` in squad deathmatch. It
    shares only the squad's skins with the maps, and its copy wins (the takes; see
    character-types.md).
  * What's not the played level's is loaded with `Game::load_level_data` (no texture library):
    bf_play's first-mission sounds (`BF_LEVEL`), the test map's weapon levels, `BF_DUMP_TYPES=all`.
  * The test map's scenery (`Game::load_scenery`, scenery.md) loads its archive's library the
    same way: a piece wears its own level's copy, not the played map's.
  * Test hooks: `BF_TEX_LOG=1` (bf_level, bf_play) prints each texture the first time it's asked
    for after the libraries change: `tex h_<name> <- <archive> (common | library | search)
    bytes <fingerprint>` (FNV-1a of its bytes: equal copies print the same) or `missing`, and a
    line per library loaded. `BF_TEX_LOG=squad bf_level <zone>` prints the squad's skins on that
    zone (with the match's library on a multiplayer map) and exits, without a window.
* **Mip levels and filtering (#257).** Textures ship with their mip chains (the tutorial's
  terrain colours DXT1 256 x 256 with 7 levels, its masks A8 128 x 128 with 8, most of
  common.tgz's HUD and effect textures with 1); `texture::decode_mips` reads them, and every
  world texture (terrain, scenery, sky, liquids, characters, weapons, effects, decals) is drawn
  with them (`scene::mip_image`). The HUD and the menus keep their textures' top level. Before,
  every texture was its top level alone: past a few metres the ground broke up into sparkling
  noise (take03's wall-gate shot: the valley floor) and close up it showed raw texels.
  * The sampler is linear within a level and between levels: the game's material classes set
    MIPFILTER from the draw context's +0x998, which its constructor (FUN_000e6ef0) sets to 2,
    D3DTEXF_LINEAR (0xe7300), and MAG / MINFILTER LINEAR. No call sets an LOD bias.
  * No anisotropy: every MAGFILTER / MINFILTER write to the render-state cache (0x4698e0) is
    POINT, LINEAR or, in one pass, GAUSSIANCUBIC, so no stage is ever ANISOTROPIC, the only
    filter that reads MAXANISOTROPY; and no call of the state setters (FUN_0006a610,
    FUN_000907a0) targets a MAXANISOTROPY slot (+0x7f4, +0xaac, +0xd64, +0x101c). The raw
    stores to a +0x7f4 at 0x16c50, 0x174cd, 0x27df7 and 0x43cfc are other objects' fields. The
    takes' distant ground is soft, as the demo's is now; at 16x it was crisper than the takes.
    `BF_ANISOTROPY=<1-16>` sets it (to compare).
  * A texture that ships one level keeps one, as on the console.
  * Cost: the images' memory rose by about a quarter (the tutorial's 68 to 85 MB, sdm_e34's 51
    to 63 MB, by `BF_TOOLS_LOG`); the frame time fell a little (the tutorial's start, no vsync:
    4.2 to 3.8 ms; sdm_e34's 3.7 to 3.3 ms).
  * Checked (#257, captures at 15 fps beside main's build of the same hooks):
    * the tutorial's opening (no hooks; take03 at the game clock's 3, 8.5, 13, 15, 18, 21 and
      25 s, the demo's frames 45 ... 374): the valley floor past the wall gate is soft, as in
      the take, where main's sparkled; the cliff by the gate, past 20 m, keeps its rock
      features at the take's size; the hill behind Tex (frame 374) and the ground at the
      cutscene's end (frame 410) show the finer tiling near the camera, where main's were
      blotches;
    * sdm_e34's firing run (`BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20 BF_TEST_FIRE=1
      BF_TEST_HIT=2`, 75 frames): the same run frame for frame (the ammo counts, the bullet
      holes), the slope in front of the camera at frame 70 the detail's grain where main's was
      a magnified blur;
    * e40's beach, against `todo/49-bower-20/take06` at 1 s (`BF_MAP=e40 BF_CHARACTER=brutus
      BF_TEST_GOTO=0,167.3,-40.6,147.9 BF_TEST_GO_AT=100`): the near sand shows the detail's
      ripples, fainter than the take's (the bump map, above, drawn since #271); sdm_e13's lava
      surface unchanged,
      its shore's lava veins sharp underfoot;
    * unchanged: the HUD (the flat floor's bars, ammo and radar the same to the pixel), the
      main menu (the same to the pixel but the highlighted entry's glow, which differs from run
      to run on main's build too), the characters (smoother at a distance, the same up close);
    * the tester's check against the takes: the far ground as soft as take03's, the near grain
      the takes' size, 1x anisotropy right, no regressions. Short of the takes: the near
      hillsides and rock 1-5 m off have less contrast (a fine band of 5.7 against 10.3 on
      take03's 21 s hillside) and e40's sand ripples are 3-4 times weaker, both the missing bump
      map (**The bump map**, above, since #271).
* A level `<transform>` is a 3 × 3 rotation followed by the position. The rotation's columns are
  the object's axes; read that way, the level's cameras come out level and looking down −z.
* Object types map to mesh archetypes through objecttypes (`mesh-name` beside the type's
  `<base name>`). Material types h_02dca948 and h_1c5f7aab are alpha-tested (grass, fences,
  grates, signs, cables); the rest use texture alpha for shine.
* Material type h_0e7658e4 is the glow shader: a colour texture (its alpha a shine mask) plus a
  glow texture (`h_e01baa40`) drawn as emissive. That's what lights the ceiling light strips in
  Bulgar's buildings and the lava cracks in sdm_m07's and sdm_e13's rock. It used to be treated
  as alpha-tested, which cut the rock into floating dark flakes.

## Other maps

Every squad deathmatch map loads (`BF_MAP=sdm_e01|sdm_e10|sdm_e13|sdm_e34|sdm_e40|sdm_m03|sdm_m07`,
or `cargo run --bin bf_level -- <map>`). What it took:

* **Terrain grid per level.** The terrain vertex shader splits a vertex's first short into row
  and column by 8 × the terrain's width in blocks + 1, and scales both by the cell size
  (`Terrain h_e023cdc9`: 1, 2 or 4 m). Blocks are 16 cells.

  | Map | Blocks per side | Cell |
  |---|---|---|
  | sdm_e34 | 8 | 2 m |
  | sdm_e01, sdm_e10 | 16 | 1 m |
  | sdm_e13 | 8 | 4 m |
  | sdm_e40 | 16 | 2 m |
  | sdm_m03 | 19 | 2 m |

  The terrain layer masks span the whole grid.
* **Baked terrain light.** Each terrain block carries 16 × 16 cell values (`blocks
  h_e2d79bad`, 0–255): the terrain's baked lighting, cliff shadows included. Terrain is drawn as
  texture × that light. The orientation (blocks row by row, x and z increasing) was found by
  correlating it with the terrain's slope lighting.

  The level's dynamic terrain lights are dim; sdm_e40's key is 0.2, low in the sky. The baked
  light is what the map-select previews show: bright sdm_e40 and sdm_e10, night sdm_e01.
* **Liquid surfaces (#4).** A surface whose material is one of the game's two liquid material
  classes is drawn as that class draws it (`LiquidMaterial`, `src/liquid.wgsl`; the code read in
  todo/146-liquids leads.md §6 and answer-spec.md, the code map's Effects rows 0x91a10-FUN_00093210):
  a liquid object's, and the few other placed objects that wear one (e13's h_e9f071d8, m02_a's
  h_fe3dec33, m14_c's three h_e82c9d42). The textures are taken as stored and the shaders' sums
  and products are on those values, as the console's combiners take the texels' bytes; the result
  is clamped as they clamp it, and fogged. Nothing doubles a colour, and no liquid gives its colour
  as light.
  * **BF_LavaMaterial** (`h_f124a774`: lava, sludge, toxic waste, chloride). t0 `texture-0` at uv /
    `texture-scale` + the clock x (`translate-u`, `translate-v`), t1 `texture-1` likewise with the
    `-1` constants (repeats per second, not divided by the scale; the names and offsets from its
    parameter table 0x91a10), the clock starting at the material's `lifetime` (0x91ba0: sludge 10,
    toxic waste 2). Its two blend modes pick the pixel shader (FUN_00091bc0, FUN_00091e60;
    `lava_passes`; the combiners decoded again for #4: no stage scales its output):

    | blend-mode / -1 | Shader | Colour | Blending | Used by |
    |---|---|---|---|---|
    | 2 / 0 | 0x2b | lerp(t0, t1 x v0, t1.a) | opaque | the lava of sdm_e13, sdm_e40, sdm_m07, the tutorial, m02 |
    | 2 / 1 | 0x2a | t0 + t1 | opaque | sludge (sdm_m03, m03_a/b, e01, mp6) |
    | 0 / 1 | 0x29 | t0 x t0.a + t1, alpha t0.a | ONE / INVSRCALPHA | toxic waste (m09, m14): see-through |
    | 1 / 1 | 0x27 | t0 + t1 | ONE / ONE | m14_c's chloride |
    | 1 / 0 | 0x28 | lerp(t0, t1, t1.a), alpha t1.a | ONE / INVSRCALPHA | none |
    | any / 2 | 0x2c | t1 | opaque | none |
    | 0 / 0 | 0x2c twice | t0, then t1 at LESSEQUAL | SRCALPHA / INVSRCALPHA each | m02_a's h_fa91cd3b (a second entity draws the second pass) |
    | one texture | 0x2c | that layer | by its own mode: 0 alpha, 1 added, else opaque | e13's h_eabb45fa (mode 2: opaque, inferred) |

    v0 is the colour of the level's key light, slot 0 of the outdoor pair (`Level::objects_key`;
    FUN_00091fe0 slot 24 from the world's +0xba0): sdm_e13 0.96 0.87 0.80, the tutorial and e40
    the h_157b3160 light's; black where slot 0 is empty (only mp2, whose lights are both indoor:
    FUN_000dcc10 then leaves it as it was, 0 after boot, inferred). It's unlit otherwise. The wrapper's h_e59d69a0 (lava 60, sludge 200, toxic waste 1) is the stencil value
    its pixels write for the game's glare, not an opacity (below).
  * **BF_OceanMaterialBase** (`h_0f8904b9`: water). Pixel shader 0x2f: (t0 x `color` + t1 x
    `color-1`) / 2 x v0, alpha the ramp's, SRCALPHA / INVSRCALPHA (0x92f70).
    * t0 `texture-0` at uv / (`scale-u`, `scale-v`) + frac(clock x (`translate-u`, `translate-v`)
      / scale), t1 likewise with the `-1` constants: the drift is near still (the pond's 0.005 and
      3e-5 repeats a second). The e40 ocean has no `texture-0`: t0 is the console's default white
      (FUN_00062280; bevy's white fallback here).
    * `texture-1` is the shimmer `h_e757ea7c`, an animated texture of 34 separate 128 x 128 frames
      (`Flipbook::frame_textures`), played in a loop at the material's `framerate-1`
      (FUN_00092d10, 0x262680): sdm_e10's pond 17 fps (2.0 s), the e40 pools 25 (1.36 s), the
      ocean 30 (1.13 s). It's what moves the water.
    * v0 is the vertex light: the object ambient + max(N.L, 0) x colour of the outdoor pair's two
      lights (answer-spec Q7), clamped, in the console's terms (`water_light`: bevy's ambient and
      directional lights taken back to the values the game hands the shader, so the Vengar view's
      vision lights reach it too).
    * Alpha: the ramp texture at the vertex's own height in its mesh (its model-space y, the
      second uv). The game makes the ramp from the material's six key / value pairs (FUN_00092df0,
      0x92eb0: 128 texels, trunc(ramp(i / 127) x 255), an ease-in-out between keys), which gives the
      archives' own ramp textures byte for byte (`water_ramp`'s test; BF_LIQUID_LOG compares each
      map's). The pond and the pools sit at model height 1: alpha 1, drawn opaque (the same
      picture; and it writes depth, as the game's does). The ocean spans 0 to 1: see-through
      where its mesh is low, at the shore.
    * Height: every vertex at its node's world height (vertex program 0x45 takes slot 28's y,
      FUN_00093210: the draw's world matrix's translation y; `water_mesh`). The pond is drawn at
      15.22 (its mesh 1 m higher), the e40 pools at 27.0-27.1, the e40 ocean at 22.10, 1 m under
      its collision's top (23.10). This replaces the lift onto the collision's top.
    * No reflection: no render target or reflection camera exists (leads §6.3), and #73's take20
      mirrors nothing in the e40 sea. The sky-colour reflection, the fresnel ramp and its 15-50%,
      the 92% opacity, the x2, the 35% self-light, the ripple tilt and the 0.02 tiles/s least drift
      the demo drew before are gone.
  * Both are drawn one-sided: take03 of #146 (26-41 s, the camera under sdm_e13's lava) shows no
    surface from under it.
  * Their clocks (`LiquidClock`) run on the game's time from the level's start: stopped under a
    menu (0x1ea830 skips the material and texture updates while paused), fixed steps under
    BF_CAPTURE.
  * Liquid surfaces have a depth bias: Singe's lake lies centimetres over terrain painted with
    lava veins, and seen low the two fought, the veins showing through the lava.
  * Test hooks: `BF_LIQUID_TEX=<hex>` puts one texture on every liquid layer;
    `BF_LIQUID_DEBUG=1` draws every liquid surface solid white; `BF_LIQUID_LOG=1` lists a map's
    liquid surfaces and materials (constants, passes, frames, the ramp against the archive's) and
    the other objects wearing a liquid material.
  * **The glare (#289).** The takes' lava is brighter and yellower than these shaders give, up to
    flat yellow close by, and glows onto the dark bank: the game's glare, a bloom laid over the
    picture after the world (`src/glare.rs`, `src/glare.wgsl`; the code map's rows for
    FUN_000e5f60, FUN_000e5d40, FUN_000e47b0, FUN_000e49d0).
    * **The keys.** Every pixel the world pass draws writes a stencil value (FUN_000d4180: REPLACE,
      reference 255, over a stencil cleared to 255 at 0xd9ea4); a wrapper material's draw sets the
      reference to its h_e59d69a0 for what it wraps (0x9ea90 -> FUN_0009e960). Liquids: lava 60
      (sdm_e13), 95 (the tutorial, e40), 80 (sdm_m07, m02, m04, m07, mp7), 20 (m02_a's two-pass
      lava); sludge 200; toxic waste 1; chloride 150 / 200 (m14_c); water and e05 / e13's h_eabb45fa
      255.
    * **The passes.** (1) The back buffer and the depth-stencil buffer, read as a texture whose
      alpha is the stencil byte, into the 320 x 240 target h_0796c3b0: pixel shader 0x1f keeps a
      texel's colour where its luminance (0.299, 0.587, 0.114) is above stencil / 255, else black,
      so the stencil value is a luminance threshold and 255 never passes (x the frame buffer's
      alpha, 1 for the lava and sludge shaders). (2) 24 copies of that into the 160 x 120 target
      h_0575b9b9, 1, 2 and 3 steps of 0.006 of the screen out in eight directions, each x 25 / 255
      (trunc(0.1 x 255): FUN_000e47b0), added ONE / ONE: 2.35 times the kept picture, saturating
      at 255. (3) That target stretched over the picture, added ONE / ONE (FUN_000e5d40). After the
      decals, before the refraction copy; in the normal view and in the Vengar view (after its
      world pass), on every level whenever a keyed surface was drawn.
    * **Here:** a keyed liquid writes its stencil value / 255 as its alpha (liquid.wgsl
      `GLARE_KEY`); every other surface leaves the picture's alpha at 1. After the tone mapping
      and the Vengar pass, a post pass reads the key from the alpha: the bright pass into a target
      of 240 rows (each texel the average of the pixels it covers, in the picture's sRGB values,
      then kept above its key), the blur into one of 120 rows (the steps as a share of the game's
      4:3 screen), added to the picture's sRGB values and clamped; the picture's alpha goes back
      to 1. The frame buffer's alpha (the game's second factor) is taken as 1. A liquid added ONE /
      ONE (chloride) writes its key too; one whose blend needs its own alpha (toxic waste's ONE /
      INVSRCALPHA, m02_a's SRCALPHA / INVSRCALPHA) writes 0, the nearest a blend can write (toxic
      waste's 1 / 255 exactly enough; m02_a's 20 / 255 not). A blend over the whole view must keep
      the alpha: the grenades' red damage tint quad does (play_grenade.rs `KeepAlpha`: bevy's
      multiply wrote 1 over every key). Something see-through drawn over a liquid raises its key
      toward 1 by its opacity (the game's REPLACE puts 255 there outright).
    * **Not keyed yet:** the game's other keyed surfaces, which multiply by their own frame buffer
      alpha, not worked out for them: the wrappers' values on self-lit glow shells (pickups' 50,
      the memory chip's 60: the halo and the fill play_dna.rs couldn't account for), on lit glow
      maps (h_1df292a1, 20-210) and on the glow shader's lamps (h_0e7658e4, sdm_e13's 0); the decal
      types' byte +0x22 (FUN_00213fc0); the Vengar view's heat figure material (0xa8c80).
    * Test hooks: `BF_GLARE=0` adds nothing (the passes run with no weight), to compare;
      `BF_GLARE_DEBUG=1` shows the keys in place of the picture (grey: a key; white: none), `=2` the
      blur alone. bf_level draws the glare too.
    * Checked against #146's takes at Brutus's start on sdm_e13 (`BF_MAP=sdm_e13 BF_CHARACTER=0
      BF_TEST_GOTO=-19.97,-122.80,-19.97,-110,25.09 BF_TEST_GO_AT=100`, frames 5-20), the lava near
      the camera (a box on the stream's near side): the demo 255 182 29, brightest tenth 255 238 31
      (without the glare 182 55 9, 210 76 10); takes 02 and 03 standing at the same place (0.5-4.5
      s) 253 189-190 28, brightest 254 253 33; take01 252 120 26, brightest 254 185 29 (its lava's
      pattern darker there: the clocks differ). Close to it the takes go flat yellow (take03 14 s
      and 23.9 s: 246-252 209-215 27-30) and the lava glows a few pixels onto the bank and over
      Brutus's outline, as the demo's does. The tutorial's pit (take08 of #28, 163.5-165.5 s, Tex at
      46.3, -25.9: `BF_MAP=tutorial BF_TEST_GOTO=45.6,-25.5,46.26,-25.92`, frames 25-29): orange
      blobs with yellow cores glowing over the grey crust; the right-hand lava 124 80 51, brightest
      252 184 80 (without 89 56 41, 187 89 40), the take's 168 103 60, brightest 252 200 94 (its
      framing has more lava and less crust). sdm_e34's start (30 frames) and the e40 sea view
      below (20 frames) are pixel for pixel main's. sdm_m03's sludge from bf_level's start view is
      fogged under its key (88 67 65): nothing added. In the Vengar view (the sdm_e13 start with
      `BF_TEST_SKILL=0.5`) the lava, greyed by the view's world pass first, glares to white: the
      code's order, which no take shows.
  * Checked (#4): sdm_e10's pond from its start (`BF_MAP=sdm_e10 BF_CHARACTER=3
    BF_TEST_GOTO=74,5,60,5 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.4`): 59 57 28 near, the shimmer
    changing frame to frame and repeating after 2.00 s (30 frames at 15 fps: a mean change of 1.5
    against 6.8 half a loop on). The e40 sea from take20's place (`BF_MAP=e40 BF_CHARACTER=3
    BF_TEST_GOTO=-1.97,169.30,-5.77,178.55,26.12 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=0.02`): the
    same warm grey-brown as the take, R : G : B about 1 : 0.84 : 0.67, from the horizon's fog to
    61 51 41 nearer (the take's 84 64 53 to 44 38 30: the code's colour before fog is 44 42 34; the
    framing isn't the take's), mirroring nothing; at e40's shore the sand shows through the
    shallow edge. sdm_e40's pools are opaque and flat in their basins; sdm_m03's sludge
    yellow-green. No take shows the shimmer up close, the pools, the pond or sludge (B4, C1).
* **Liquids aren't solid.** A pool (sdm_m03's sludge river, sdm_e13's lava, the tutorial's pits,
  and pools on sdm_e10, sdm_e40, sdm_m07, mp4, mp6 and mp7) is a placed object of class
  `h_04366a6a` (`water-lava-flow-object`) whose collision is a closed volume under its surface
  (sdm_e13's lava a box from 12.94 to 23.89 m; sdm_e10's pond and the e40 ocean 40- and
  420-triangle volumes): the game makes it a phantom (FUN_002218e0), so it's left out of the floor
  and walls here, and characters wade and sink to the bed. No movement code reads it: no
  slowdown, swimming, drowning or tint under it (todo/146-liquids leads §4). A liquid's type
  (objecttypes `<h_fa2f5452>`, `water-lava-flow`, loaded by FUN_00193420) gives its kind
  (`liquid-type`, the LT_ names at 0x3be754) and three damage values (`LiquidType::damage`):

  | liquid-type | Liquid | h_06f6a40d: an object's hit | h_eba69d33 | h_e0489ea0: a character's rate |
  |---|---|---|---|---|
  | 0 | ocean water (e40) | 0 | 0 | 0 |
  | 1 | swamp water (sdm_e10's pond, the e40 pools) | 0 | 0 | 0 |
  | 2 | lava (sdm_e13, sdm_e40, sdm_m07, the tutorial) | 50 | 50 | 30 |
  | 3 | toxic waste (m09, m14) | 0 | 10 | 30 |
  | 4 | sludge (sdm_m03, m03, e01, mp6) | 50 | 50 | 50 |
  | 5 | chloride (m14_c) | 0 | 0 | 0 |

  h_eba69d33 is read by nothing; h_06f6a40d is one hit on an object entering (a pickup, a
  grenade, a missile: #146's Part 5, not in the demo).
* **What a liquid does to a character** (#152, play_liquids.rs; the code read in
  todo/146-liquids leads.md §2-§4 and answer-spec.md Q1 / Q9, the code map's Levels rows
  FUN_00221af0-FUN_00222400):
  * The liquid sees a living character by its eleven hit-part bodies (PID_BONE), never by its
    movement capsule (answer-spec Q1; L1's live read on e40's pools: 16 enter events, all
    PID_BONE). A part is in while it's in the volume, however deep: here, while the lowest corner
    of its collision surface, posed as drawn, is inside the volume (`Arena::liquid_inside`: an
    odd number of the liquid's faces above it). The count is the game's char+0x104.
    * Each face counts once. A point on an edge or a corner its triangles share is over one of
      them alone (`Arena::over_once`, a half-open rule with no slack): the slack the floor test
      gives an edge counted both halves of a split face near their diagonal, an even count, so
      a part crossing the diagonal of sdm_e13's lava top (x + z = 0, some 4 cm either side) went
      out and back in, a whole entry hit each time. And a twin pair (a face that comes twice,
      wound both ways) counts once: the e40 ocean's 420 triangles are 210 such pairs, as the e40
      lava flow's 168, m02_a's lava's 720 and e05's water's 184, so before they held nobody.
    * Every liquid of the 19 maps that have one closes round its volume (`BF_FIND_BODIES=1`: no
      open edge, a twin pair counted once; sdm_e10's pond and its copies 40 triangles, the boxes
      12, m03_a's 8); sdm_m07's four lava quads are open sheets, under which a point is in.
  * The damage is the character rate alone, once a game frame (30 Hz, the liquid's update): a
    frame on which a part enters deals the whole rate, any other with a part in its share of
    the second, and one hit a frame at most (FUN_00221a40). No depth ramp, no tick, nothing
    more for going under: a body fully under keeps burning at the rate. Every hit, an entry's and
    each frame's share alike, goes through the character's damage handler as DTYPE_FLAME
    (0x221780 -> 0x125600; play.rs `burn` through `take_hit`): a stamina shield takes it first,
    nothing in a cutscene or out of the world, the difficulty step (the multiplayer maps none;
    STANDARD's squad x0.6), Vengar, the type's factor, BF_COMBAT_LOG's line. It floors nobody.
    Only an entry hit may draw a hurt line (the demo's rule: take03 has no sound between the foot
    events while Brutus stands burning, todo/5-liquid-sounds leads §4.1).
    Its blood is the game's decals alone (FUN_00124a70: the entry hits are big enough, the
    shares never; the spray straight up), and a death in a liquid leaves out the demo's death
    spray (play_fx.rs). The game frames run between the drawn ones with the parts' lowest points
    between their two places, so 15 Hz captures and a 60 Hz screen deal what the game's 30 Hz
    does.
  * A fall into a liquid hurts by the drop to its bed, as on land (FUN_0012b740 has no liquid
    check; answer-spec Q9); only the landing's sound (and dust, which the demo doesn't draw) is
    left out while a part is in one (0x12d842).
  * Checked against the takes (`BF_LIQUID_LOG=1` prints the parts in and each hit):
    * #146's take01 (sdm_e13, Brutus at 75 HP walking north into the lava at half stick:
      `BF_MAP=sdm_e13 BF_CHARACTER=0 BF_TEST_GOTO=-19.97,-122.80,-20.92,-100,25.09 BF_TEST_WALK=1
      BF_TEST_HEALTH=75`): the first part in at (-20.3, -116.0), 75 to 45; the second 0.24 s
      later (the take's 0.20), down to 9 (the take's 9.3); dead 0.54 s after the first (the
      take's 0.53), ankle-deep.
    * take03 (Brutus at 5000, `BF_TEST_HEALTH=5000`, to (-21.44, -110.15), then on to
      (-23.73, -87.26)): thirteen entry hits of 30 walking in (the take's twelve or thirteen),
      then 9 of 11 parts in waist-deep (the take's 10) at 30.0 HP a second (the take's 30.0); all
      11 in, 3 m under, still 30.0 a second and alive (the take's 30.2, alive 9 s under). The view
      from under the lava shows no surface (one-sided, as the take).
    * #28's take08 (the tutorial, STANDARD: `BF_MAP=tutorial BF_TEST_GOTO=46.26,-25.92,45.96,-30.40
      BF_TEST_HEALTH=101.6`): Tex runs off the east pit's rim and falls in: five entry hits of 18
      (30 x 0.6), 101.6 to 83.6, 64.4, 45.8, 27.8, 9.8 (the take's 83.0, 64.5, 45.9, then 36 in
      one step to 9.9), then 18 a second on the bed, dead 0.80 s after the first hit (the take's
      0.87). The same at 15 Hz and 30 Hz capture steps. (With the goal at z -34, past where the
      take's Tex stopped at -30.4, and his full 115, he runs on over the bed and up out of the
      pit's south side with 4.6 left.)
    * Across sdm_e13's lava diagonal (`BF_MAP=sdm_e13 BF_CHARACTER=0 BF_TEST_GOTO=-12,2,-2,12
      BF_TEST_WALK=1 BF_TEST_HEALTH=5000`: Brutus starts 4 m under the surface and walks across
      x + z = 0 at about 4.6 s): one entry hit as he starts, then 30 HP a second to the end. With
      the old slack the same walk took seven more entry hits of 30 between 4.60 and 5.20 s, as his
      parts crossed the diagonal.
  * Not filmed: sludge's rate (50, sdm_m03: C1), toxic waste's (30, m09: D1), a fall into a pool
    (B3), squad members or enemies in a liquid; built from the code and the data alone. The
    waters do nothing (their rates are 0).
  * The tutorial's lava pits (h_ea243aea, two) are a box of collision (12 triangles, the east
    one's top at 14.40 m, its bottom at 3.45 m) in a pit whose walls are steeper than a floor
    (maps.md, "Collision": steep ground with no floor under it, #264).
  * The AI keeps out of the liquids that burn (#1): squad.md, "Liquids that burn".
  * Test hooks: `BF_LIQUID_LOG=1` prints each character's parts in a liquid as their count
    changes, each entry hit, a death, the hit points each second in one, and a wingman waiting at
    the shore of one (#1); `BF_TEST_WALK=1`
    makes BF_TEST_GOTO walk (Ctrl), as the takes' half stick; `BF_TEST_HEALTH=<hp>` above the most
    raises the most too (play_pickups.rs).
* **What a liquid plays** (#5: play_liquids.rs, play.rs `WorldSound`; the code read in
  todo/5-liquid-sounds leads.md and todo/146-liquids leads.md §5, the code map's Sound rows
  FUN_00179250-FUN_0017f580 and Levels rows FUN_00221af0-FUN_00222480). A liquid type names four
  effects and four sounds (`LiquidType::effects` / `sounds`), in slots the code reads alike for
  every liquid:

  | Slot | Effect (`env_<x>l_*`) | Sound | Plays |
  |---|---|---|---|
  | `LIQUID_SHOT` | h_08f6cd94 `_projectile` | h_e259380a | an instant ray through the surface (FUN_0022feb0) |
  | `LIQUID_SPLASH` | h_e4791abc `_smsplash` | h_0ed21145 | a part or an object going in with a bounding radius under h_e12e7b9d (0.1 m; chloride 0.25) |
  | `LIQUID_BIG_SPLASH` | h_07ab4143 `_mdsplash` | h_0c46dff8 | the same with a radius of at least that (FUN_00221af0) |
  | `LIQUID_WADE` | h_f458daca `_wadering`, silent | h_ed055bce | the ring round a wading part (FUN_00221ec0); the sound is the wading step (FUN_00119020) |

  `<x>` is x water, s lava, f toxic, t type 3, i type 0 (with bubbles). Every type of one
  liquid-type names the same four sounds, five recordings among the sixteen ids: lava's big splash
  and its wading step are one wave, the ocean's small and big splash another; chloride has none
  (todo/5 leads §1). The effect types carry no sounds, and no liquid has a sound of its own: the
  shore's and the lava's loops are the levels' sound-triggers (triggers.md).
  * **Splashing in.** Each hit part going in (at any speed: a walk, a jump and a fall alike)
    casts FUN_00222480's ray from the top of its bounding sphere 2r down against that liquid
    (`Arena::liquid_below`); the sphere is the part archetype's `bounding-sphere-radius`
    h_1ab09530 round its `bounding-sphere-center` h_e99750a9 (`Character::part_spheres`; the
    squad's eleven parts 0.14-0.45 m: all big). Meeting the surface, the character keeps its
    height (char+0x414, `Wading::surface`) and the part splashes there: the effect upright at user
    parameter 1.0, the sound at the surface point, volume 1.0 (in place of the bank's 0.5), pitch
    U x 0.1 - 0.05 octaves. A part whose sphere doesn't reach the surface as it goes in splashes
    nothing: walking in, the feet and calves go in first without one, and the splashes start
    with the part that does (take03: no splash for the first four parts, the first at the fifth;
    the demo's Brutus the same). A part out and in again splashes again.
  * **Rings.** Each part in a liquid, at most every 0.2 s, casts its ray again; where the surface
    point has moved more than h_049350bb (0.1 m) from its last ring's (its node as it went in, at
    first), and not straight up or down, a ring there, turned to the move, at user parameter
    clamp((moved - 0.1) / dt x h_fd22997d, 0, 1). Silent. So a part crossing the surface leaves up
    to five a second while it moves, and none once it's under or still.
  * **Wading steps.** Each game frame a part is in, the character takes the liquid's fourth sound
    as its next step's (char+0x410). At a foot event (the demo's: a foot planting on the move,
    play.rs `step_player`) in a liquid, the step plays it only while the surface of the last
    splash is more than 0.25 m above that foot, and plays **nothing** otherwise: the ground's step
    never sounds in a liquid, and before any splash has found the surface (char+0x414 still -1)
    every step is silent. Its volume is max(min(depth, 1), the skill's noise) x the gait's factor
    (1.0 running or dashing, 0.5 walking, 0.35 the crouch walk) x 0.75 for the controlled
    character, whose own step plays 2D (inferred: FUN_00119020 puts its point where the cull reads
    none); another's plays at its foot; the pitch U x 0.7 - 0.35 octaves. The first step after
    coming out still plays the wading step the last frame in left (the code's order); the ground's
    come back from the next.
  * **Objects** (`object_enters`). A grenade (play_grenade.rs) or a missile (play_missile.rs) going
    into a liquid, from out of every one, at h_1f6de6f9 (0.4 m/s) or faster, splashes where its
    body crosses the surface, by its mesh archetype's radius (`Game::archetype_sphere`): the five
    hand grenades 0.081 m small, the Sentry 0.33 m and the Roller 0.19 m big, the iKhan-GPL's
    round 0.046 m small. They go on down to the bed (the liquids aren't solid). Not done: what
    happens to them in it (a grenade's fuse cut to at most 0.3 s, 0x149420; a missile's timers x5,
    FUN_00226cc0), the object damage h_06f6a40d, the blast's liquid override effect (#146).
  * **Shots.** Only an instant ray (bullet-type 4: the MK-ASLT, Bower 20, L-Shots, Minigun,
    Foley, Kman, Rails) splashes, once where it goes through the surface: the shot effect upright
    and the shot sound at the library's volume (FUN_0022feb0 sets no volume or pitch). A bolt or a
    flying shot passes through with nothing (the blast pool has no liquid code); the ocean's trail
    of bubbles along the ray under its surface (+0x168) isn't drawn.
  * **Nothing** plays for going under, staying under, getting out or burning: no reader of the
    liquid state in the sound system (todo/5 leads §2.5).
  * **How they're heard** (play.rs `play_sounds`, `WorldSound`: FUN_00179250's request). The
    listener is FUN_00178c10's, as the sound-triggers have it: the followed character's feet, or a
    cutscene's camera while one is shown. A positional sound is heard by its library entry's
    distances (`distance_gain`: every liquid sound full to 5 m, nothing at 15 m) and dropped at
    15 m or beyond (0x17932a); a 2D one at its volume. The pitch is the playback speed, 2^octaves
    (DirectSound's pitch is a resampling too). Not done: the sound manager's queue and its budget
    of 16 positional voices a frame (FUN_001793e0: here every one plays), and placing a sound in
    3D (every demo sound plays 2D at its distance's volume).
  * Checked with `BF_SOUND_LOG=1 BF_LIQUID_LOG=1` (BF_MUTE: nothing was played aloud; how they
    sound is the user's check by ear):
    * sdm_e13's lava, Brutus walking in to 2.6 m deep (`BF_MAP=sdm_e13 BF_CHARACTER=0
      BF_TEST_GOTO=-19.97,-122.80,-22.6,-102.0 BF_TEST_WALK=1 BF_TEST_HEALTH=5000`): his first
      four parts in without a splash and his steps silent (no surface found yet), then big
      splashes (ef2c3bf6, the lava's C recording) at full volume, 0.7-1.5 m away, as more parts go
      in, and wading steps (188ca605) 2D at 0.38 every ~0.55 s, 1.0-2.6 m under the surface, with
      their +-0.35 octave pitch; frames 80-118 show the rings at his feet, then the splashes'
      orange puffs round his waist (take03 at 10.2-10.9 s: orange puffs round Brutus's waist as
      the splashes start; the demo's brighter and yellower, the ALE effect's colour).
    * The tutorial's pit (#28's take08 recipe above): Tex falls in, six big splashes over 0.07 s
      at 1.3-1.9 m (a fall splashes as a walk does), the damage unchanged (five hits of 18, dead
      0.80 s after the first).
    * sdm_e10's pond (water: `BF_MAP=sdm_e10 BF_CHARACTER=3 BF_TEST_GOTO=74,5,74,-30
      BF_TEST_WALK=1`): wading steps (fa93070a) from 0.60 to 0.31 m deep, silent at 0.25, 0.17,
      0.10 and 0 m, the first step out still the wading step, then the ground's (fe306c9e,
      e5393d24) at 0.35. With `BF_TEST_GOTO=74,5,74,5 BF_TEST_TARGET=1` Brutus stands in it 8 m
      off: his five big splashes (0d339af9) heard at 0.54-0.66 by the distance.
    * Brutus at sdm_e13's shore (`BF_TEST_GOTO=-20.6,-117.5,-20.62,-117.0`): `BF_TEST_THROW=1.0,0.05
      BF_CAMERA_PITCH=-0.2` lobs a Frag in at 15.1 m/s, a small splash (1eb5691c, the B recording) 5.3 m off at 1.00;
      thrown long (`BF_TEST_GOTO=-19.97,-122.80,-20.0,-121.8 BF_TEST_THROW=1.0,0.6 BF_CAMERA_PITCH=0.1`) it goes in
      15.3 m off, past the 15 m: not heard. `BF_TEST_WEAPON=iKhan-GPL BF_TEST_FIRE=1
      BF_CAMERA_PITCH=-0.25`: each round a small splash 8.6 m off at about 0.5. `BF_TEST_FIRE=2`
      (the Bower 20, bullet-type 4) at `BF_CAMERA_PITCH=-0.3`: a shot effect and sound
      (0a94d4e9) where each ray goes in, 12.9-24 m off: those under 15 m at 0.00-0.03, the rest
      not heard.
  * Not filmed: every water item is **code only, needs a take**: which of the B and D recordings
    plays for each event on water, the silence in the shallows (0.25 m), the player's own step 2D
    and a squadmate's at its foot, a grenade into water, a ray through it (todo/5 shot list).
    The lava's are checked against take03's offline audio analysis only (todo/5 leads §4: a burst
    within 0.06 s of each foot event waist-deep; no burst for the first four parts in), which
    can't tell the lava's big splash from its wading step (one recording). The idle animation's
    foot events (take03's step every ~2.1 s standing waist-deep) and the ragdoll's splashes
    (take01's corpse) aren't reproduced: the demo's steps come from feet planting on the move,
    and its ragdoll has no part bodies.
* **Terrain hue.** The baked light map gives the ground's brightness. Its hue comes from the
  lights marked `h_e02aba1c` with the terrain's ambient, at 60% strength. Before #143 those were
  read as the terrain's own pair; the game's code has them as the indoor pair (see **Light
  objects**), and what lights its terrain is #181's, so the hue is kept as it was calibrated. If
  vertex program 0x38 is the terrain's base pass (see **The detail**: no code was found loading
  it), its vertex light is the Terrain `<ambient>` plus slot 1's light of whichever pair was
  uploaded last (c18 / c19, x the material's colour; the pair's fill where it has a key and a
  fill), each view's draw starting with the outdoor pair (FUN_000dcc10(0, ..) at 0xda2aa), and
  the bump map takes slot 0's (c16, the key; **The bump map**), all of it x the vertex's baked
  light. So the outdoor pair, not the `h_e02aba1c` lights, would reach the ground. A lead for
  #181 (inferred: c26 and c27, which decode the baked light's byte, are set for no terrain
  program): read as its low 7 bits / 127.5 x (the ambient + the pair's N.L), e40's beach comes
  out about 0.6 of the demo's whole-cell light, and take06's beach is darker by about that (25
  luma against the demo's 35, the same view).
  On Bulgar those lights are blue-grey, and the captures show grey ground and hills, not sand.
  The 60% is calibrated on a capture of the gate: the ground there measures 73 81 79; ours was
  79 74 64 and is now 68 77 74. `BF_DOOR_LOG=1` lists the doors (type, place, leaves); Bulgar's
  big gate is `h_0792d16a` at (-23.2, 74.3).
* **Light objects** (#143). A level's `<light-object>` is read by its type's `light-type`
  (objecttypes `<light><h_e275fb80 light-type=N>`, parsed by 0x192970 through the LT_ names at
  0x3be720: 1 point, 2 spot, 3 directional; `Game::light_types`), not by a list of type names.
  The game's data has four directional types (h_e1cac6f7 and h_f5302dc6 in common, h_157b3160
  in e05, e40, m03_a/b/c, m08_a/b, mp8 and the tutorial, h_e8d6efe4 in e06, e10, m05_a/c/d/x)
  and six point types (h_ea460e64, h_09020c30 and h_0b51040f in common, h_e81ba579, h_087ae059,
  h_f1557cc8, h_ff66877a); no spot. The three directional types' records are alike: nothing in
  them says key or fill. Before #143 the demo kept h_e1cac6f7, h_f5302dc6 and h_ea460e64 only,
  so e40 and the tutorial lost their main light (h_157b3160) and drew their squads black, 15
  maps lost a directional light and 959 point lights were dropped.
  * **Two pairs.** The game lights an object with two directional lights at most
    (FUN_000dcc10): the *outdoor* pair, the lights flagged `h_1f416687` (the light's +0x94 bit
    2), or the *indoor* pair, those flagged `h_e02aba1c` (bit 1); a light may be in both (the
    tutorial's fill). The object's `light-usage` picks one (the wrapper material's draw
    0x9f3c0): LU_OUTDOOR the outdoor pair, LU_INDOOR the indoor one, LU_GEOSET each geoset's
    own `h_f89cd77d` (`Geoset::indoor`, a building's inner faces: the tutorial has 59 such
    geosets on 16 meshes), LU_NONE none (here the outdoor pair, the one each view starts with).
    Each pair takes its lights from the level file; no level has more than two in one. Its two
    slots (c16 / c17, c18 / c19) light an object alike; FUN_000dcc10 gives slot 0 to the pair's
    key light, the one whose type has enable-specular (#288, **The bump map**: only the
    terrain's bump map and the lava's v0 tell them apart). The demo read `h_e02aba1c` as "lights
    the terrain" before; it's the indoor pair.
  * **Here.** The outdoor pair are bevy's directional lights (`spawn_lighting`), at PI lux with
    their sRGB colour, the brightest (by luma) casting the characters' shadow: the game's
    lights cast none, so the one shadow is the demo's choice. Characters take the outdoor pair
    (535 of the game's 542 placed characters are LU_OUTDOOR). A placed object's indoor-lit
    parts get an indoor variant of their material (`Lamps::indoor`, shader def INDOOR_PAIR):
    `src/lamps.wgsl` takes bevy's directional lights that are the outdoor pair out of
    `apply_pbr_lighting`'s result exactly as it put them in (with the shadow) and puts the
    indoor pair in by the same diffuse and specular. It knows the outdoor pair's lights by their
    direction and their colour (`light_buffer` gives it both): by the direction alone the Vengar
    view's light from below passed for sdm_m07's outdoor key, which also shines from below. In
    the Vengar view, whose vision pair replaces both pairs in the game, nothing is swapped: the
    outdoor pair is put out then, and on a level with no outdoor pair the indoor pair is lit only
    while bevy has no directional light at all. A level with no outdoor pair (mp2: both its
    lights are indoor) lights its outdoor objects and its characters with the ambient alone, as
    the game's code does; one with no directional light at all gets a made-up sun.
  * **Strength.** The shaders take N.L against the light matrix's third column as it is, not
    normalised (vertex program 0x19's `max(0, n . c16)`): a hand-made transform whose column
    is short lights less. 34 of the 161 directional placements are off unit length by over
    2% (e40's key 0.944, its outdoor fill 0.834); here a light's colour is scaled by it
    (`LevelLight::lit_color`).
  * **Linked lights.** A `<parent Type="1">` puts a light at its parent object times the
    link's offset (FUN_0015f0b0, FUN_0015ed30); 373 point lights (all 159 h_09020c30, 214
    h_e81ba579, white, range 10) hang on effect objects, whose effect types name them and run
    them with an ALE light-effect: the placed effects draw those lights (#258), so these aren't
    drawn again. A `<parent Type="2">` (a hardpoint) keeps its own transform.
  * `BF_LIGHT_LOG=1` lists every light-object (name, type, kind, pairs, colour, strength,
    direction or place), the two pairs and FUN_000dcc10's slots for each (`Level::slots`: the
    outdoor slot 0 is the bump map's light and the lava's v0), the ambients, the ones not drawn
    and why, and each placement lit by the indoor pair.
* **Point lights.** A point light-object (every point type, #143) is a place, a colour
  (`h_f08eb2f3`, 0-1), a `range` (4-40 m) and a `falloff` kind (3 or 4 on all of them; the
  schema leaves its enum, `h_f30c3302`, unnamed, and the light's setter goes through the generic
  property table, so its meaning isn't traced; the other `falloff`s, explosions', are
  `DFALL_NONE / LINEAR / EXPONENTIAL / HALF_LIFE`). The game lights static geometry per vertex
  by them (the scenery meshes carry no baked colours), each object by the lamps of its pair
  only (FUN_0009f0b0 takes those with the pair's flag; one with neither, 32 on m09_c, lights
  nothing). Here the level materials do it per pixel (`LevelMaterial`: bevy's standard material
  plus `src/lamps.wgsl`, which skips the lamps of the other pair): each lamp's colour x
  (1 - distance / range) x N.L, unshadowed, nothing past the range, every falloff kind taken as
  linear. Bevy's own point lights fall off as 1 / d^2 and couldn't follow that. Characters take
  the outdoor pair's lamps at their chest from every direction, half of it, as their texture
  added (emissive). The terrain takes none: it's drawn unlit, by its baked light map. The game
  adds lamps to its terrain in a pass of their own per block (FUN_00091070: additive, three a
  pass, vertex program 0x51) from a list whose filling isn't traced, so which lamps light the
  ground there, and by which pair if any, is unknown (#181). Ammo Depot has 24, Cavern of Fire
  22, Bulgar's squad map 2, campaign maps up to 184 (m07_a). `BF_NO_LAMPS=1` leaves them out
  (to compare); `BF_LIGHT_LOG=1` lists them and each start point's nearest.
* **Grass.** Bulgar's grass clumps are see-through cards standing upright with their normals
  lying flat (every normal's y is 0). Lit by those, they caught almost none of the high key
  light and their backs none at all: they were black. Cut-out cards whose normals all lie flat
  (|y| < 0.1) are given normals pointing up, so they're lit like the ground, both faces alike.
  `BF_MAT_LOG=1` lists the materials (type, textures, average colour).
* **Shadows.** The scenery and the terrain cast no shadow; only characters do, from the
  brightest light of the outdoor pair (on e34, whose outdoor pair is two h_e1cac6f7, both cast
  one before #143). The game lights static geometry per vertex by the level's lights (every
  light-object is `enable-static`), with no shadows. With shadow-casting roofs interiors were
  black, and a jungle canopy darkened the whole floor. The terrain cast one until #143: a
  character standing where it shades the ground was drawn black (Tex on e40's beach, which
  take17 of todo/73 shows lit), and it darkened sdm_e34's start ravine's grass and the water
  along sdm_e40's shore.
* **Checked** (#143; `BF_MUTE=1 BF_BACKGROUND=1`, frames beside the takes):
  * e40 at take04's spot (`BF_MAP=e40 BF_TEST_GOTO=-16.00,152.32,-16.50,152.26
    BF_CAMERA_PITCH=-0.10 BF_TEST_WEAPON="Therm Sweeper"`, frame 30, todo/73 take04 1.5 s): Tex,
    Flint, Brutus and the standing stone are lit (main drew them black); the stone's lit face
    measures 76 55 42 against the take's 77 55 43. The cliff (terrain, bluish here, #181) isn't
    this. At take17's spot (`BF_TEST_GOTO=17.15,175.43,16.70,175.22`, the rest the same, frame
    39 against take17 1.5 s) Tex is lit (his arm 72 38 27 against the take's 94 47 35) and the
    two near stones measure 75 53 42 and 78 57 43 against 83 60 47 and 61 44 34; while the
    terrain cast a shadow they were black (17 10 7, 10 9 8) and so was Tex.
  * The tutorial's opening (`BF_MAP=tutorial`, frames 285-375 against take03 at the same clock +
    1.66 s): Tex lit as the take has him (vest 146-161 luma against the take's 133-138, his
    shaded arm 50 against 52; main 1-12). The second building (`BF_TEST_GOTO=37.22,-88.80,36.9,
    -88.55` against take11 2.5 s) and the base building (`-62.22,19.23,-62.40,19.47` against
    take12 300 s): their insides take the bluish indoor pair as the takes show (the base
    building's wall 33 54 74 against take12's 34 39 51; main black and white); the gate wall
    (`-43.76,-13.46,-43.81,-13.16`, take11 715 s) shows its dark blue panels where main drew it
    black; the sniper crate and the base's containers are lit (take11 297 s, take12 50 and
    120 s).
  * Bulgar's interiors: inside sdm_e34's building at (-51.2, 10.9) (`BF_MAP=sdm_e34
    BF_TEST_GOTO=-51.2,10.9,-48.2,10.9 BF_TEST_GO_AT=99`) the walls measure 34-42 47-57 55-65,
    where main had 17-28 20-33 24-37: #2's capture measured 40 55 60 and the demo 23 27 32.
  * Unchanged: by the lights, sdm_e40's and sdm_e34's start views are the same pixel for pixel;
    the terrain's shadow gone lightens the grass in sdm_e34's ravine (2.3% of the frame) and the
    water along sdm_e40's shore (0.3%). e34 at take01's spot differs only by Tex's second
    shadow. sdm_e40's ruin at (52.9, -54) has 27 indoor geosets and its 6 indoor lamps: it's now
    a dim, green-lit room where the outdoor key lit it (no take of it); its floor, part of the
    ruin, takes those lamps. The Vengar view is the same as main's (to within 1 of 255) inside
    an e34 building (`BF_MAP=e34 BF_CHARACTER=0 BF_TEST_GOTO=-51.3,10.4,-48.3,10.4
    BF_TEST_GO_AT=99 BF_TEST_SKILL=2.0`, frame 74), looking up at sdm_m07's indoor-lit ceiling
    (`BF_MAP=sdm_m07 BF_CHARACTER=0 BF_TEST_GOTO=27.3,2.6,27.3,4.0,87.5 BF_TEST_GO_AT=99
    BF_TEST_SKILL=1.0 BF_CAMERA_PITCH=0.75`, frames 40 and 59) and at mp2's start with
    `BF_NO_LAMPS=1` (frame 44).
  * The 15 maps that had lost a directional light (e05, e06, e10, e40, m03_a/b/c, m05_a/c/d/x,
    m08_a/b, mp8, the tutorial): their start views now light the squads and the scenery the
    outdoor pair faces (m03_a's, m05_a's and m08_a's squads were black). Lamps: m04_c's new red
    lamps light its rock spikes and nothing burns out, but no capture of m04_c's lamps (nor of
    any other cluster of them) exists, so their look is untested against the game. Frame time
    (no vsync, median of 40 s, before the terrain stopped casting shadows) m07_a 4.62 -> 4.90 ms
    (85 -> 184 lamps), m04_c 5.43 -> 6.04 ms (160 -> 181).
* **Sky.** A level's `<sky><object mesh-name>` is a set of layers, drawn in order. On Bulgar:
  a flat blue top, a panorama band of mountains and clouds, a moon, and a cloud swirl. They use
  the self-lit shader `h_f539fe8c`, whose texture is `h_e01baa40` (the texture lookup missed it,
  so skies were flat grey). See-through layers are blended. The sky moves sideways with the
  camera, so its mountains stay on the horizon (the game's stays at its `<position>`,
  FUN_00177d50: a few degrees of horizon parallax across a map, left as it is).
* **The sky turns (#199).** The `<sky><object>` carries `<rotation>` (the starting angles,
  radians: `0 360 0` on e05, e40, m03 and the tutorial, 0.3 on e23, 0.05 on e34) and
  `angular-speed` (h_18180282, rad/s about x, y, z). The game turns the whole model about its
  origin every world step (FUN_00177ba0: angles += dt x speed, the model's frame the identity
  turned about x, y and z by them), every layer together. Turning: e23 and e34 (0.01 about y),
  e05, e16 and the tutorial (0.005), e40 and sdm_e40 (0.004), m03_a/b/c (0.0032), m09_a/b/c/x
  and m14_a/b (0.005 about x and 0.01 about z), m14_c (0.008, 0.005, 0.007), mp3 (0.05, 0.04,
  0.05); every other sky, sdm_e34's among them, is still. `Level::sky_of` reads both
  (`SkySpin`); each layer (`SkyLayer`) keeps the sky's clock, which runs on the game's
  (`frame_dt`: stopped under a menu, the capture's fixed steps), and `follow_sky` turns it about
  the sky's origin, each layer's lifted origin turned with it, so a tumbling sky keeps its
  layers together. The sense is the takes': the helpers' frame read with its columns as the
  model's axes (`SkySpin::rotation`); read the other way, e40's clouds drifted right. Test hook:
  `BF_SKY_TIME=<s>` starts the sky's clock that far into the level (bf_play and bf_level).
  Verified (`BF_MAP=e40 BF_TEST_GOTO=0.29,166.87,-3.46,176.14,26.13 BF_TEST_GO_AT=99
  BF_CAMERA_PITCH=0.2`, the view of #198 take06; crops phase-correlated against a frame 2 s in):
  the clouds drift screen-left, 2.4-2.6 px/s above the picture's centre and 4.0-4.1 px/s at the
  left (the takes' 2.9 and 4.0: 0.004 rad/s at the game's focal length, 686 px, which the demo's
  55.4-degree view shares), the same at 15 fps and at `BF_CAPTURE_FPS=5`; the rock holds within
  1-2 px (the idle's sway). sdm_e34 over 20 s: the sky still (0.04 px). m09_a's sky over the test
  map (`BF_TEST_SKY="1,m09_a"`, `BF_SKY_TIME=0` and `60`): its planet and nebula turned together.
  Not checked against the game: the starting angle (no take is known to be at a level's start:
  e23's start would show it) and the tumbling skies (none filmed).
* **sdm_m07's background.** It has no sky (`h_ee630063="false"`, a cavern), and its terrain
  mesh is missing, so the background shows through where the terrain should be. There, the
  background is the fog's colour rather than its bright blue.
* **sdm_m07.** It names a terrain mesh that's on no disc archive. It loads without terrain: the
  map is all objects.

Known gaps:

* sdm_m07 is much darker than its preview's lava glow.
* Bulgar's start ravine is darker than in the game (the terrain's baked light there is low).
* Characters standing indoors look darker than takes 11 and 12 show (Tex's back inside the
  tutorial's buildings 33-35 luma against 40-48): they take the outdoor pair, where the game's
  characters may take another indoors (their light-usage, char+0x44, isn't read live), or the
  squad's shine (vertex program 0x19's environment map coordinate) adds what's missing.
* Characters take the object ambient: the game gives skinned meshes `<character-ambient>` (c15,
  0xda19b; e16 0.28 against its object ambient 0.04, m09_a 0.004 against 0.06), here read
  (`Level::character_ambient`) but not drawn. Bevy's ambient term is also 0.45 x texture x
  ambient (EnvBRDFApprox at roughness 1), not texture x ambient: the shade on maps with a strong
  ambient (mp2, m07_c, sdm_m07) is about half the game's.
* e40's characters outdoors are brighter and warmer than the takes show (take17 of todo/73:
  Tex's vest luma 68 against 45, his shaded pants about 42 against about 10). That hints the
  game lights e40's characters by something other than the outdoor key: the same open question
  of their light-usage (char+0x44).
* e05's sea went from navy to a flat light grey with #143's lights; there's no take of it.
* The terrain takes no point lights (see **Point lights**), where the game adds them per block.
* Mission maps (e01, m01_a, …) load their geometry but have no start points; missions place
  the player with their triggers. bf_play starts the squad at its spawn-triggers or its
  placements where the level has them (the tutorial, e40, m09_a: [triggers.md](triggers.md),
  #102); the cutscenes and warps that move them next are #28's step 3.
