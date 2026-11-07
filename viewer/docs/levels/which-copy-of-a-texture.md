*Part of [levels/](README.md), the former page `levels.md`.*

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
  * What's not the played level's is loaded with `Game::load_level_data` (no texture library), after
    the played level's data (below, #338): bf_play's first-mission data (`BF_LEVEL`), the test map's
    weapon levels, `BF_DUMP_TYPES=all`.
  * The test map's scenery (`Game::load_scenery`, scenery.md) loads its archive's library the
    same way: a piece wears its own level's copy, not the played map's.
  * Test hooks: `BF_TEX_LOG=1` (bf_level, bf_play) prints each texture the first time it's asked
    for after the libraries change: `tex h_<name> <- <archive> (common | library | search)
    bytes <fingerprint>` (FNV-1a of its bytes: equal copies print the same) or `missing`, and a
    line per library loaded. `BF_TEX_LOG=squad bf_level <zone>` prints the squad's skins on that
    zone (with the match's library on a multiplayer map) and exits, without a window.
