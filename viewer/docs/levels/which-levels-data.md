*Part of [levels/](README.md), the former page `levels.md`.*

* **Which level's data (#338).** The game loads common's data and the played level's and nothing
  of another level (FUN_000d61a0: a level file's `<data-libraries>` list is `common <level>`).
  bf_play adds the default level's beside them, `BF_LEVEL` (e01): its sound bank and world
  materials are what the footsteps and landings play (and mp1's sounds, since a mission carries
  only its own squad's footsteps), and its objects, materials, types, animations and chatter fill
  in what a map's own data lacks. What two archives name alike is the first loaded one's
  (`Game::add_objects`, `add_materials`, `add_animations`, the types, the sound bank, the
  scripts ...), so the order is common's, **the played map's** (`Game::load_level`, bf_play's
  `load_map`), then the default level's (`load_default_level`) and the test map's extras
  (`testmap::load_weapon_data`). Until #338 the default level came before the map's, so where the
  two named something alike it was e01's that the map got.
  * The collision surfaces are what that cost (maps.md, "Which level's shapes"): the blockers' and
    hinters' names are numbered alike in every level, 291 of the 958 on the 53 maps took e01's shape.
    The other things the loaders merge by name hardly differ. e01 against the 52 other archives,
    decoded and compared field for field (the offsets into an archive's own data files left out;
    sound files, animation channels and scripts byte for byte), of the names both define: archetypes
    0 of 2873 differ, meshes 0 of 2578, materials 1 of 6348, object types (weapons and items among
    them) 1 of 4740, sounds 0 of 11340 and their files 0 of 10556, animation channels 0 of 34407,
    animation sets 5 of 40, scripts 0 of 8; the world materials are one file, byte-equal in all 54
    archives. The differing six: e16's material h_ff8c786e (other textures), e23's effect type
    h_edb05c62 (a smaller sphere, no effect names), and animation set h_ed14db04 in e23 and m03_a and
    h_09af546b in m04_a, m07_b and m09_b (the same 60 or 66 clips on other data channels). Those six
    maps now get their own.
  * The character types are the same in every archive that has one (#128), and stay so; what
    changes is which archive a type is first loaded from (`CharacterType::archive`, the test tools'
    `@archive`). The tables keyed by motion set (`Game::characters`, `character_hitpoints` ...) take the
    first type that names a set: where marines and terrorists share `ms_militia` the tutorial's
    "militia" is its Red Hand now (23 hp, `BF_DUMP_HITPOINTS`) and not e01's marine (90 hp). They are
    read for the squad's four sets, which no other type shares, and by the type report.
  * `Game::levels` holds the played level's file alone (`Level::load` and `Level::load_sky` read the
    last), and the first loaded of a script name stays (`Game::scripts`, which `play_script.rs` filters
    by the played map's name); the world materials are read once.
  * Checked (`cargo test --lib load_order_tests`; a collision check before and after on the 53 maps,
    maps.md; the dump hooks `BF_DUMP_WEAPONS`, `BF_DUMP_TYPES`, `BF_DUMP_HITPOINTS`,
    `BF_FIND_BODIES`, `BF_FIND_STEEP` on the tutorial, e34, sdm_e34, e01 and m09_a, before and after: the
    same lines but the changed bodies' and the triangle counts'; e01's all the same).
