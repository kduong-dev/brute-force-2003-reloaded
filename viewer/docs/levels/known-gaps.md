*Part of [levels/](README.md), the former page `levels.md`.*

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
  placements where the level has them (the tutorial, e40, m09_a: [triggers.md](../triggers/README.md),
  #102); the cutscenes and warps that move them next are #28's step 3.
