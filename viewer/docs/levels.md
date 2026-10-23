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
`BF_ANISOTROPY=<1-16>` (the world textures' anisotropy; the game's is none) and
`BF_SKY_TIME=<s>` (the sky's clock started that far into the level), all below.

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
  * Not done: the bump map. Pixel shaders 0x1a / 0x1b give final = (vertex light + bump) x
    lerp(colour, detail, share), the bump being the P8 normal map lit per pixel by the key light,
    (2 t3 - 1) . (2 v1 - 1), at the detail's tiling and unfaded. Without it the near ground has
    less contrast than the takes: the hillsides and rock 1-5 m off have a fine band of 5.7
    against 10.3 on take03's 21 s hillside, and e40's sand ripples (`todo/49-bower-20/take06`,
    1.0 s) are 3-4 times weaker (the tester's measures). It belongs with the terrain's own
    lighting.
  * The layers' masks: program 0x38 samples a layer's mask at the second uv set (oT1 = v10,
    pixel shader 0x1a's alpha), which on every masked layer's vertex is its place across the
    terrain, 0 to 1 with v toward -z: the orientation the demo bakes into the vertex alpha (2 m
    apart, the masks' own texel pitch). The ground's geoset (pixel shader 0x1b, no mask) carries
    another second uv set (the tutorial's: the position / 3.2 m), unread.
  * Not found in the code: what selects vertex program 0x38 and pixel shaders 0x1a / 0x1b. No
    call of the program setter (FUN_000a4a60, 72 sites) or the pixel shader setter
    (FUN_000a4d00) passes those indices, and the program tables hold their only references. The
    rest fits them: the begin computes c20's 1 / s², s² and w, which of the terrain's
    (declaration-8) programs only 0x38 reads (c20 is the world matrix's first row in programs
    0x2-0x19, 0x27, 0x3f, 0x50 and others); the masked layers' second uv set is the mask's
    coordinate; the terrain lights are `enable-bump`; and the takes show the finer tiling near
    the camera (take03 at 25 s, take08 at 45 s, the e40 beach).
    A live read of the current program (0x3f7c58, set by FUN_000a4a60) while a terrain block
    draws would settle it.
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
      ripples, fainter than the take's (the bump map, above); sdm_e13's lava surface unchanged,
      its shore's lava veins sharp underfoot;
    * unchanged: the HUD (the flat floor's bars, ammo and radar the same to the pixel), the
      main menu (the same to the pixel but the highlighted entry's glow, which differs from run
      to run on main's build too), the characters (smoother at a distance, the same up close);
    * the tester's check against the takes: the far ground as soft as take03's, the near grain
      the takes' size, 1x anisotropy right, no regressions. Short of the takes: the near
      hillsides and rock 1-5 m off have less contrast (a fine band of 5.7 against 10.3 on
      take03's 21 s hillside) and e40's sand ripples are 3-4 times weaker, both the missing bump
      map ("Not done" above).
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
* **Liquid surfaces.** A liquid object's surfaces use one of two shaders, drawn by
  `LiquidMaterial` (`src/liquid.wgsl`):
  * the animated liquid `h_f124a774` (lava, toxic rivers): `texture-0` and `texture-1`, each
    a tile `texture-scale` / `texture-scale-1` of the mesh's uv across (by the sizes that gives:
    sdm_e13's lava, 1.5 km over 15 uv, has 10 m tiles at 0.1; read as repeats, a 256-texel
    texture stretched over a kilometre, blurred, and scrolled tens of m/s), scrolling in tiles
    per second (two unnamed pairs, by their values u and v of each layer: `h_0e41af5b
    h_0e2b9302`, `h_e002ae8e h_fb0bff34`), combined by
    `blend-mode-1` (0: over by its alpha, sdm_e40's lava crust; 1: added, the toxic river's
    green over its dark swirl). Every layered liquid's `blend-mode` is 2, read as the console's
    modulate x 2: the colour doubled. Lava (liquid-type 2) gives all its colour as light, toxic
    (4) 80%; both are solid (the materials' own opacity, 60-200 / 255, is left out: captures of
    Singe's and Cavern of Fire's lava and Shanty Town's acid show them opaque, near yellow).
  * the water `h_0f8904b9`: `texture-0` tiled (`scale-u`, `scale-v`) and scrolling, tinted by
    `color`; over it the reflection by a fresnel ramp (`h_0b85cb09`, a 128 x 1 alpha strip: more
    at a glance). The game's reflection is `texture-1`, a texture no archive holds (rendered at
    run time); here it's the level's sky colour tinted by `color-1`, rippled by the water's own
    texture, 15% looking straight down (without any a pool seen from above was its murky colour
    over a bed of the same colour: it vanished) to 50% at a glance; 92% opaque, its colour doubled
    too (no blend-mode: a choice; a capture of sdm_e10's pond measures 57 60 36, ours 49 49 18).
  * Water ripples: its texture twice (scale-u, scale-v and scale-u-1, scale-v-1, each drifting:
    at least 0.02 tiles/s, the two crossing; sdm_e10's own speeds, 0.0002 and 0.01, left it still;
    the water's `framerate` constants, 17 and 25, hint at an animation of the game's own, not
    traced). Their brightness slopes, as heights, tilt the surface: the light, the sun's glint
    and the reflection ripple, and the water's colour is shaded by the tilt from one side. It
    gives 35% of its colour as its own light: sdm_e10's dim lights alone left the pond near
    black, its ripples lost, where a capture shows it evenly lit.
  * Liquid surfaces have a depth bias: Singe's lake lies centimetres over terrain painted with
    lava veins, and seen low the two fought, the veins showing through the lava.
    `BF_LIQUID_TEX=<hex>` puts one texture on every liquid layer (a test).
  * A liquid's drawn surface is moved onto its collision's top (what's waded in): sdm_e40's
    pools draw theirs 1 m higher, so from the side the water floated over the floor (from above
    it lined up, so it seemed to come and go with the view); lava and the rivers already agree.
    `BF_LIQUID_DEBUG=1` draws every liquid surface solid white.
  Other names found by hash: `framerate`, `framerate-1`, `lifetime`, `blend-mode`.
  `BF_LIQUID_LOG=1` lists a map's liquid surfaces (material, textures, constants).
* **Splashes.** A liquid type also names its effects and sounds (`<h_fa2f5452>` entries:
  h_08f6cd94 a shot striking it, h_e4791abc a small splash, h_07ab4143 a big one, h_f458daca the
  ring round a wader; `env_xl_*` water, `env_sl_*` lava, `env_fl_*` toxic, `env_tl_*` type 3,
  `env_il_*` type 0 with bubbles) and four sounds (unnamed; taken in the effects' order). Going
  into a liquid splashes (big when falling faster than 5 m/s), wading leaves a ring every 0.5 s
  while moving, and a shot through a surface splashes where it goes in.
* **Liquids aren't solid.** A pool (sdm_m03's toxic river, sdm_e13's lava, and pools on sdm_e10,
  sdm_e40, sdm_m07, mp4, mp6 and mp7) is a placed object of class `h_04366a6a` whose collision
  is one flat plane at its surface. That plane is left out of the floor and walls: characters
  wade and sink to the bed. A liquid's type
  (objecttypes `<h_fa2f5452>`, read by 0x193420) gives its kind (`liquid-type`) and three damage
  rates per second (h_06f6a40d, h_eba69d33, h_e0489ea0):

  | liquid-type | Liquid | Damage per second |
  |---|---|---|
  | 1 | water | 0, 0, 0 |
  | 2 | lava | 50, 50, 30 |
  | 3 | (unnamed) | 0, 10, 30 |
  | 4 | toxic | 50, 50, 50 |

  How the game uses these values isn't traced (a capture of sdm_e13 had Brutus wade waist-deep
  through lava for 10 s at full health, then die the moment he went under). Here a harmful
  liquid burns from the first touch: 20% of its highest rate at the surface, rising with depth
  to all of it at 1.6 m (lava: 10 hp/s ankle-deep, about 30 waist-deep), dealt every 0.4 s
  without blood; going under kills. A fall into a liquid doesn't hurt.
  * The tutorial's lava pits (h_ea243aea, two) are a box of collision (12 triangles, the east
    one's top at 14.40 m, its bottom at 3.45 m) in a pit whose walls are steeper than a floor
    (maps.md, "Collision": steep ground with no floor under it, #264). Tex runs off the east
    pit's rim into it as in take08 (todo/28-tutorial), and dies sooner than there: the demo kills
    him as his feet go 1.6 m under, about 0.3-0.5 s after he leaves the rim, where take08's Tex
    (running off it at 167.16 s) loses 18.6 HP a 67 ms step from 0.6 m under (167.56 s), 36 in
    one step at 2.3 m, then about 18 a second on the bed, and is at 0 at 168.43 s: 1.3 s after
    the rim. How the game deals it is #152's.
* **Terrain hue.** The baked light map gives the ground's brightness. Its hue comes from the
  lights marked `h_e02aba1c` with the terrain's ambient, at 60% strength. Before #143 those were
  read as the terrain's own pair; the game's code has them as the indoor pair (see **Light
  objects**), and what lights its terrain is #181's, so the hue is kept as it was calibrated. If
  vertex program 0x38 is the terrain's base pass (see **The detail**: no code was found loading
  it), its vertex light is the Terrain `<ambient>` plus the second light of whichever pair was
  uploaded last (c18 / c19, x the material's colour): each view's draw starts with the outdoor
  pair (FUN_000dcc10(0, ..) at 0xda2aa), and the bump takes the first light's direction (c16).
  So the outdoor pair, not the `h_e02aba1c` lights, would reach the ground.
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
    Each pair takes its lights in the level file's order; no level has more than two in one.
    The demo read `h_e02aba1c` as "lights the terrain" before; it's the indoor pair.
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
    direction or place), the two pairs, the ambients, the ones not drawn and why, and each
    placement lit by the indoor pair.
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
