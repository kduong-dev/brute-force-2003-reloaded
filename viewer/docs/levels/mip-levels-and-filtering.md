*Part of [levels/](README.md), the former page `levels.md`.*

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
