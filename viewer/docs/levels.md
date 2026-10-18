# Level viewer (`bf_level`)

`cargo run --release --bin bf_level -- [level]` draws a level from the original archives:
terrain, every placed object (game objects, pickups) at its transform, the sky, and the level's
fog. The default is `sdm_e34`, the squad deathmatch map "Battle of Bulgar" (SDM Level 5 in
campaign-bf.xml). Right-drag looks around, W/A/S/D/Q/E fly (Shift for faster), 1-9 jump to the
level's own flyby cameras and 0 to an overview, F toggles fog, P prints the view. Test hooks:
`BF_LEVEL_CAMERA=<n>`, `BF_VIEW=x,y,z,yaw,pitch`, `BF_NO_FOG=1`, `BF_SCREENSHOT=<file.png>`,
`BF_LEVEL_DUMP=1` (placements, untextured materials, idle effects), `BF_PART_DUMP=<hex
archetype>` (each part's geosets: offset, bounds, normals, winding, UV direction; then exits).
In bf_level and bf_play: `BF_NO_DETAIL=1` (the terrain without its detail texture) and
`BF_ANISOTROPY=<1-16>` (the world textures' anisotropy; the game's is none), both below.

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
    layers in brackets):
    * `h_08e8e4c3`, the detail's tiling over the colour's uv (3.5, 3, 2.5; 2.5-7 in the data);
    * `h_fe22cc04`, the distance from the eye it fades out at (20, 20, 15 m; 10-35);
    * `h_153264c6`, its share at the eye (0.9, 1, 0.99; 0.8-1).

    The material's begin (FUN_00090f50) binds the detail at stage 2 and hands these to the
    vertex shader as c20 = (1 / s², s², k, w). The base pass that reads them, vertex program
    0x38 with pixel shaders 0x1a (a layer) and 0x1b (the ground), samples the colour at the
    vertex uv and the detail at k times it, and mixes them by w (1 - d² / s²), d the distance
    from the eye, 0 past s. So within s of the eye the ground shows its texture k times finer,
    fading to the colour's own tiling at s. The demo drew the colour's tiling everywhere: within
    a few metres of the camera a 256-texel texture over 8 m (the tutorial's grass) to 18 m (its
    rock ground), magnified to blurry blotches.
    A layer's shader now does the mix (`level_scene::terrain_detail`, `src/lamps.wgsl` under
    `TERRAIN_DETAIL`), per pixel where the console's share is per vertex. The colour's own uv is
    the vertex uv, as before: the programs read the first uv set as it is (the vertex
    declaration at 0x3c9860 puts it at v9), and take03's wall-gate shot, where the cliff is past
    20 m, shows the same rock features at the same size as the demo.
  * Not done: the bump map. Pixel shaders 0x1a / 0x1b light it per pixel by the key light,
    (2 t3 - 1) . (2 v1 - 1), at the detail's tiling, unfaded: on e40's beach the sand's ripples
    (`todo/49-bower-20/take06`, 1.0 s) come from it, where the demo's sand is smooth. It belongs
    with the terrain's own lighting.
  * The layers' masks: program 0x38 samples a layer's mask at the second uv set (oT1 = v10,
    pixel shader 0x1a's alpha), which on every masked layer's vertex is its place across the
    terrain, 0 to 1 with v toward -z: the orientation the demo bakes into the vertex alpha (2 m
    apart, the masks' own texel pitch). The ground's geoset (pixel shader 0x1b, no mask) carries
    another second uv set (the tutorial's: the position / 3.2 m), unread.
  * Not found in the code: what selects vertex program 0x38 and pixel shaders 0x1a / 0x1b. No
    call of the program setter (FUN_000a4a60, 72 sites) or the pixel shader setter
    (FUN_000a4d00) passes those indices, and the program tables hold their only references. The
    rest fits them: the begin computes c20's 1 / s², s² and w, which only 0x38 reads; the masked
    layers' second uv set is the mask's coordinate; the terrain lights are `enable-bump`; and the
    takes show the finer tiling near the camera (take03 at 25 s, take08 at 45 s, the e40 beach).
    A live read of the current program (0x3f7c58, set by FUN_000a4a60) while a terrain block
    draws would settle it.
  * Test hooks: `BF_NO_DETAIL=1` draws the colour's tiling alone, as before.
* **Mip levels and filtering (#257).** Textures ship with their mip chains (the tutorial's
  terrain colours DXT1 256 x 256 with 7 levels, its masks A8 128 x 128 with 8, most of
  common.tgz's HUD and effect textures with 1); `texture::decode_mips` reads them, and every
  world texture (terrain, scenery, sky, liquids, characters, weapons, effects, decals) is drawn
  with them (`scene::mip_image`). The HUD and the menus keep their textures' top level. Before,
  every texture was its top level alone: past a few metres the ground broke up into sparkling
  noise (take03's wall-gate shot: the valley floor) and close up it showed raw texels.
  * The sampler is linear within a level and between levels: the game's material classes set
    MIPFILTER from the draw context's +0x998, which its constructor (FUN_000e6ef0) sets to 2,
    D3DTEXF_LINEAR (0xe7300), and MAG / MINFILTER LINEAR. No LOD bias is written.
  * No anisotropy: no MAGFILTER or MINFILTER write in the game is D3DTEXF_ANISOTROPIC (the
    values are POINT, LINEAR and one GAUSSIANCUBIC), and no MAXANISOTROPY slot is ever set. The
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
      to run on main's build too), the characters (smoother at a distance, the same up close).
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
* **Terrain hue.** The baked light map gives the ground's brightness. Its hue comes from the
  terrain's own lights (ambient + key + fill, the pair marked `h_e02aba1c`), at 60% strength.
  On Bulgar those lights are blue-grey, and the captures show grey ground and hills, not sand.
  The 60% is calibrated on a capture of the gate: the ground there measures 73 81 79; ours was
  79 74 64 and is now 68 77 74. `BF_DOOR_LOG=1` lists the doors (type, place, leaves); Bulgar's
  big gate is `h_0792d16a` at (-23.2, 74.3).
* **Point lights.** A level's light-object `h_ea460e64` is a point light: a place, a colour
  (`h_f08eb2f3`, 0-1), a `range` (4-40 m) and a `falloff` kind (3 or 4 on all 1290 of them; the
  schema leaves its enum, `h_f30c3302`, unnamed, and the light's setter goes through the generic
  property table, so its meaning isn't traced; the other `falloff`s, explosions', are
  `DFALL_NONE / LINEAR / EXPONENTIAL / HALF_LIFE`). The game lights static geometry per vertex
  by them (the scenery meshes carry no baked colours). Here the level materials do it per pixel
  (`LevelMaterial`: bevy's standard material plus `src/lamps.wgsl`): each lamp's colour x
  (1 - distance / range) x N.L, unshadowed, nothing past the range, every falloff kind taken as
  linear. Bevy's own point lights fall off as 1 / d^2 and couldn't follow that. Characters take
  the lamps' light at their chest from every direction, half of it, as their texture added
  (emissive). Ammo Depot has 24, Cavern of Fire 22, Bulgar's squad map 2, campaign maps up to
  160. `BF_NO_LAMPS=1` leaves them out (to compare); `BF_LIGHT_LOG=1` lists them and each start
  point's nearest.
* **Grass.** Bulgar's grass clumps are see-through cards standing upright with their normals
  lying flat (every normal's y is 0). Lit by those, they caught almost none of the high key
  light and their backs none at all: they were black. Cut-out cards whose normals all lie flat
  (|y| < 0.1) are given normals pointing up, so they're lit like the ground, both faces alike.
  `BF_MAT_LOG=1` lists the materials (type, textures, average colour).
* **Shadows.** The scenery casts no shadow; only characters do. The game lights static
  geometry per vertex by the level's lights (every light-object is `enable-static`), with no
  shadows, so building interiors are lit like the walls outside. With shadow-casting roofs
  they were black, and a jungle canopy darkened the whole floor.
* **Sky.** A level's `<sky><object mesh-name>` is a set of layers, drawn in order. On Bulgar:
  a flat blue top, a panorama band of mountains and clouds, a moon, and a cloud swirl. They use
  the self-lit shader `h_f539fe8c`, whose texture is `h_e01baa40` (the texture lookup missed it,
  so skies were flat grey). See-through layers are blended. The sky moves sideways with the
  camera, so its mountains stay on the horizon.
* **sdm_m07's background.** It has no sky (`h_ee630063="false"`, a cavern), and its terrain
  mesh is missing, so the background shows through where the terrain should be. There, the
  background is the fog's colour rather than its bright blue.
* **sdm_m07.** It names a terrain mesh that's on no disc archive. It loads without terrain: the
  map is all objects.

Known gaps:

* sdm_m07 is much darker than its preview's lava glow.
* Bulgar's start ravine is darker than in the game (the terrain's baked light there is low).
* Bulgar's interiors are much darker than in the game: walls facing away from the level's
  two directional lights get only its dim ambient. Captured mean 40 55 60, ours 23 27 32.
  Static meshes carry no per-vertex lighting (24-byte vertices: position, normal, uv), so the
  game lights them some other way, not yet traced.
* Mission maps (e01, m01_a, …) load their geometry but have no start points; missions place
  the player with their triggers. bf_play starts the squad at its spawn-triggers or its
  placements where the level has them (the tutorial, e40, m09_a: [triggers.md](triggers.md),
  #102); the cutscenes and warps that move them next are #28's step 3.
