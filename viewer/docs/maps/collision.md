*Part of [maps/](README.md), the former page `maps.md`.*

* **Collision.** It uses the game's own collision surfaces (`src/bf/collision.rs`,
  `src/arena.rs`). On sdm_e34 that's 43,444 triangles.
  * Where they come from: `objects-<level>.ipn` holds them in the Ipion physics engine's format.
    The terrain blocks, the 23 invisible blockers and every placed object's parts each name
    theirs. Door leaves have their own and stop blocking while open. A blocker is a wall to
    every character or, a hinter, to the AI's alone (#292, "Blockers and hinters" below), and to
    nothing else: grenades, ragdolls, pickups, shots and decals pass through (#298).
  * **Which level's shapes (#338).** A surface is found by the name its owner gives it, and the
    levels number their blockers and hinters alike. Of the 9057 surface names in the 53 level
    archives' tables 1967 are in more than one archive and 291 of those have more than one shape:
    `h_fcc5d473` is in 42 archives with 39 shapes, `h_ee99c575` in 38 with 37, `h_09910e17` in 37
    with 36, `h_1bcd1f11` in 36 with 35, `h_fee0807e` in 32 with 31, and e01's shape of each is no
    other archive's (34 of e01's 492 surfaces are shaped otherwise somewhere, and 46 of the other
    52 archives have at least one that e01 shapes otherwise). The game loads common's data and
    the level's and nothing of another level (FUN_000d61a0), so a blocker is the shape its own
    archive has. bf_play loaded e01's data (the first mission's, `BF_LEVEL`: for its sounds)
    before the map's, and the first surface of a name wins (`Game::add_objects`), so on every map
    but e01 a blocker or hinter named as one of e01's took e01's shape. `BF_COLLISION_CHECK` over
    the 53 maps found 774 blocker surfaces (2768 placed) and 184 hinter surfaces (184 placed); 290
    and 142 had a name of e01's, and 155 (on 37 maps) and 136 (on 24) took another shape than the
    map's own: 291 of the 958. The terrain blocks (5960 surfaces) and the placed objects' parts
    (3851, 771 of them named in e01 too) never did. Examples (surfaces, those with a name of
    e01's, those that took another shape): the tutorial's blockers and hinters 24, 11 (6 blockers
    that were the same, and the 5 hinters), 5; sdm_e34's 14, 9, 5; e34's 17, 5, 1; e13's 6, 5, 5;
    m03_c's 32, 20, 11; m09_a's 18, 11, 11; sdm_e40's 5, 2, 2.
    * Now the map's data is loaded first, right after common's, and e01's after it, to fill in
      what the map lacks (`load_map`, `load_default_level`; the extras of the test map come after
      that): the check finds none on all 53 maps, and every surface a map names has a shape in its
      own data. e01's own map is as it was.
    * What else the levels name alike is the same in both, but for six archives (levels.md,
      "Which level's data"): so the order changes nothing for archetypes, meshes, materials,
      object types, sounds, animations and scripts, and gives e16, e23, m03_a, m04_a, m07_b and
      m09_b their own material, effect type and animation sets.
    * `BF_COLLISION_CHECK=1` (bf_play): reads the map's own data and e01's again alone (common's
      with the map's, as bf_level loads it) and prints for the terrain blocks, the blockers, the
      hinters and the placed objects' parts: the surfaces and their uses, how many `BF_LEVEL`'s
      level (e01) also has under the name, how many the map has no shape for, and how many the
      loaded data gives another shape than the map's own (with the first names); then exits.
    * What changed on screen. sdm_e34's `h_fee0807e` blocker at (14.7, 99.8) was e01's 19.8 x
      12.0 x 22.3 m box and is the map's own 7.1 x 8.0 x 10.3 m one: Tex running north along
      x 14.7 from z 84 (`BF_MAP=sdm_e34 BF_TEST_GOTO=14.7,84,14.7,112 BF_HELD_LOG=1`) was held
      for the rest of the run at (13.60, 36.46, 92.71) by body 51, a blocker, and now walks on,
      held a moment by the new body 51's west face (x 11.89) at (11.59, 37.78, 102.33). The
      tutorial's five hinters (the AI's alone) span 9-38 m where e01's spanned 3-22 m: four of
      them lie round the lava yard's pits and walkways. Terrorist 30 of the yard's plate trigger
      (`BF_MAP=tutorial BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999 BF_BRAIN_LOG=1
      BF_COMBAT_LOG=1 BF_TEST_AIM_HOSTILE=14`) used to see Tex across the pit at 26.9 m and stop
      at 2.33 s, still on its way, and fire (it died at 15 s to the aim helper); now it reaches
      its home (62.1, -38.2) at 2.53 s and sees no one for the 40 s: the demo's sight rays stop at
      a hinter's band as at any blocker (`Arena::ray`; which blockers the game's own sight ray
      leaves out isn't traced, #298): the line from its home to Tex on the plate crosses body 64,
      h_ee99c575's band at (48.6, -28.1), twice. That agrees with todo/28-tutorial take08, where
      nobody fired on Tex on the plate (brain.md).
      The other runs compared, before against after (`BF_MUTE=1 BF_BACKGROUND=1`, 15 fps, the
      frames saved at every 3rd to 15th): e01's start (48 of 60 frames identical, the rest
      differing in at most 0.008% of the pixels by over 8 levels; two runs of one build differ
      so, and in their first frames when the load is slow) and its autopilot run (46 of 60, at
      most 0.002%), sdm_e34's autopilot run (60 of 60), the tutorial's marines (63 of 70, at most
      0.005%) and first patrol (34 of 40, at most 0.001%), e34's squad follow (29 of 60, at most
      0.023%) and its fight with the Shrike Hound (49 of 50), the tutorial's start (27 of 30, at
      most 0.07%: a flicker) and e34's (19 of 30, at most 0.001%), and the test map, `-- --test`
      and `-- --test BF_MAP=tutorial` (9 of 9 each): the same to the particles' noise, and
      their logs the same line for line but the map's triangle count.
  * Material: every triangle carries one, a world-material id plus a slide flag.
  * Standing: characters stand on surfaces up to 65° (the game's ground limit), facing up by
    their winding, within a 0.5 m step; steeper surfaces are walls (#285: so is a surface facing
    down, under the body).
