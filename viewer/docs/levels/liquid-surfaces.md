*Part of [levels/](README.md), the former page `levels.md`.*

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
    | 0 / 1 | 0x29 | t0 x t0.a + t1, alpha t0.a | ONE / INVSRCALPHA | toxic waste (m09, m14; m14_c's pools over the violet crystals): see-through |
    | 1 / 1 | 0x27 | t0 + t1 | ONE / ONE | m14_c's chloride (the blue h_1c711b16) |
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
      to 1. The frame buffer's alpha (the game's second factor) is taken as 1, as the code leaves
      it nearly everywhere (#291): the back buffer is A8R8G8B8 (FUN_0006a750), the view's clear
      writes alpha 255 (the background colour packed 0xFFRRGGBB, FUN_000d8d80), and the world pass
      writes no alpha (COLORWRITEENABLE red, green and blue, 0x10101, FUN_001ea890), all four
      channels only for the sorted list (0xe86dd), where ONE / INVSRCALPHA and ONE / ONE keep 255
      over 255 and SRCALPHA / INVSRCALPHA a little less. A liquid added ONE /
      ONE (chloride) writes its key too; one whose blend needs its own alpha (toxic waste's ONE /
      INVSRCALPHA, m02_a's SRCALPHA / INVSRCALPHA) writes 0, the nearest a blend can write (toxic
      waste's 1 / 255 exactly enough; m02_a's 20 / 255 not). A blend over the whole view must keep
      the alpha: the grenades' red damage tint quad does (play_grenade.rs `KeepAlpha`: bevy's
      multiply wrote 1 over every key). Something see-through drawn over a liquid raises its key
      toward 1 by its opacity (the game's REPLACE puts 255 there outright).
    * **Not keyed yet:** the game's other keyed surfaces, which multiply by their own frame buffer
      alpha, not worked out for them: the wrappers' values on self-lit glow shells (pickups' 50,
      the memory chip's 60: the halo and the fill play_dna.rs couldn't account for; the placed
      Sentry's red tips, 50, #300, which are opaque and so drawn in the world pass, where the
      frame buffer's alpha is 1), on lit glow maps (h_1df292a1, 20-210) and on the glow shader's
      lamps (h_0e7658e4, sdm_e13's 0); the decal types' byte +0x22 (FUN_00213fc0); the Vengar
      view's heat figure material (0xa8c80).
    * **Placed objects' untextured self-lit parts** (the shader h_f539fe8c with no texture, behind
      a wrapper) are drawn as before #289 read the wrapper's value: their colour added to the
      picture at that value / 255 (a flat one at full strength: the medkits' crosses). The game's
      rule (FUN_0008fd20, `level_scene::self_lit`, which the models' materials follow since #300:
      grenades.md, Sentry) is opaque for nine of the levels' thirteen such materials (the Sentry
      pickup's blue lights; lamps; big planes on e01, m01_a, m01_b, m02_b, mp6, mp7 and sdm_e01,
      some 30 x 25 m, which the demo draws faint or not at all: m01_a's blue h_ff823904 has the
      value 0) and blended by their `alpha` for the other four (common's glass shells). Not
      applied to the placed objects: no take shows those planes yet.
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
    * m14_c's toxic pools (#291; `BF_MAP=m14_c BF_SCRIPT=off
      BF_TEST_FREECAM=0.3,-29.8,52,36,180,-35 --edit`, frame 24): the see-through surface over the
      violet crystals is toxic waste (h_19563c4f, liquid-type 3; wrapper h_1b1bb811, key 1, over
      h_e2f14539), not the chloride (h_1c711b16, liquid-type 5, key 200, the blue pool). Its texture's
      alpha is 0 over most of it, so the crystals show through nearly as they are (95 79 147 without
      the glare), and with a key of 1 and the frame buffer's alpha 255 everything seen through it
      passes: white (246 240 252), as the code has it. `BF_GLARE_DEBUG=1` shows key 0 over
      exactly the surface `BF_LIQUID_DEBUG=1` draws, 200 over the chloride. No take shows them.
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
