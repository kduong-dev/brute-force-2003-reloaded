*Part of [testtools/](README.md), the former page `testtools.md`.*

* **Skies** (`play_testworld.rs`): the list is every level whose level file names a sky mesh
  (`<sky><object mesh-name>`, `Level::sky_mesh`), read from each level archive (up to its
  levels-*.xmb) on a thread when the session starts. A pick loads that level's data on a thread
  (`Game::load`, its archive, `Level::load_sky`), makes its layers as a level's own are made
  (`level_scene::sky_layers`, the same layers, order and blending) and keeps them for the
  session (a second pick is at once). While one loads, other sky picks are ignored ("Still
  loading the sky of ..."): each load reads the game data anew. It replaces the map's sky
  (`SkyLayer` entities) and the clear colour becomes its `<background-color>`; "none" puts the map's own clear colour back
  (the flat floor's is black). The camera's far plane goes out to 5000 m, a level's, so the flat
  floor's (Bevy's 1000 m) reaches the layers.
* **Music**: the list is every level of the game's list whose `data/sounds/<level>.xwb` has
  tracks (its first 16 KB read for the names; mp2, mp3, mp6 and splash_screen are empty
  stubs). A pick decodes the bank and reads that level's sound bank for the tracks' Types on a
  thread, then mixes it as `setup` mixes a level's (play.rs `music_mix`: the music, the first
  ambience bed under it at 80%, all beds if it has no music), looping at the music volume. The
  map's own music is stopped first (`LevelMusic`). `BF_MUTE` loads and logs but plays nothing.
  As with skies, picks while a bank loads are ignored.
  * The test map's own music is m04_a's bank (#302, `TEST_MUSIC`: test_casp1_amb01 with the
    ambience bed ambience19-2ch under it at 80%), loaded so as the flat floor starts and playing
    from about a second in, without a message; "this map's own" plays it again there, and Left /
    Right step from it. A pick while it's still loading takes its place. `BF_NO_MUSIC` leaves it
    off. `BF_TOOLS_LOG` or `BF_SOUND_LOG` print "test map: its music, m04_a's bank, loading" and
    then "test tools: music of m04_a: music test_casp1_amb01 x1.00, ambience ambience19-2ch x0.80,
    looping" ("(BF_MUTE: not played)" under `BF_MUTE`).
