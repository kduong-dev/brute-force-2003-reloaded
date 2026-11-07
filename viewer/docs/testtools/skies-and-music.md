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
  * **The unused tracks** (#396; the product owner's request, from the cut-content catalogue,
    section 3.3). The list ends with a group headed "[unused in the game: no level plays these]":
    five music and ambience tracks that sit in the levels' streaming wave banks
    (`data/sounds/<level>.xwb`) with no sound entry naming them, so no level can play them (the
    banks' wave names hash straight to the sound ids, and none of these five names' hashes is the
    id of a `Sound` in any of the 61 sound banks, `sounds-*.xml`; the used tracks' are, e.g.
    caspian_action1 in e01's). Eight bank entries in all:

    | Track | Banks (the first is read) | Plays |
    |---|---|---|
    | `creepy_estuary_mutant_01` | e05.xwb, sdm_e40.xwb | 104 s |
    | `amb_singe_02` | e13.xwb, sdm_e13.xwb | 45 s |
    | `ambience19-2ch` | m02_a.xwb, m04_a.xwb | 22 s |
    | `caspian_amb_mux_temp` | m08_a.xwb | 165 s |
    | `ambiencehellish-2ch` | sdm_m07.xwb | 157 s |

    (The lengths are what the demo's decoder gives, one more sample in 64 than the game's byte rate
    counts, 1.6% longer: the log prints them; each wave is the same bytes in each of its banks.) The
    pick's panel says "in the wave banks e05.xwb, sdm_e40.xwb only: no sound entry names it, so no
    level plays it". Picking one (Enter, a click) reads that wave out of its first bank on a
    thread (the bank file read whole, the one wave decoded: `load_unused`), stops the map's music as
    the track comes in (as a level's bank does) and plays it **alone**, looping, at the music volume
    (`BF_MUSIC_VOLUME`) times its share by `music_mix`'s name rule, the one a bank's track no sound
    bank defines gets: a name starting "amb" is ambience, which has no music to sit under and plays at
    the bed's share (x0.80; `amb_singe_02`, `ambience19-2ch`, `ambiencehellish-2ch`), the other two
    are music at x1.00 (`creepy_estuary_mutant_01`, `caspian_amb_mux_temp`). Left / Right step
    through the banks only, from the bank the track was read from; the banks' list, the "this map's
    own" entry and every bank's mix are as before. The hook `BF_TEST_MUSIC=<s>,unused:<name>` picks
    one (its name in any case, a part of it, or its place in the list from 0:
    `BF_TEST_MUSIC=2,unused:caspian`). Logs (`BF_TOOLS_LOG` or `BF_SOUND_LOG`): "test tools: unused
    track creepy_estuary_mutant_01 (e05.xwb) read: 2541744 bytes of 22050 Hz 2-channel audio, 104.1 s
    once decoded; the same wave is in sdm_e40.xwb", then "test tools: music of unused track
    creepy_estuary_mutant_01 (e05.xwb): music creepy_estuary_mutant_01 x1.00" and, under `BF_MUTE`,
    "(BF_MUTE: not played)" (else ", looping"). As with a bank, picks while one loads are ignored.
    Assigning the tracks to levels is out of scope: the game never did.
