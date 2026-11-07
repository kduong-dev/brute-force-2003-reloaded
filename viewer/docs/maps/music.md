*Part of [maps/](README.md), the former page `maps.md`.*

* **Music.** The game streams `data/sounds/<level>.xwb` by level name (sdm_e34's is
  caspian_action20, 59.7 s, stereo Xbox ADPCM, 22050 Hz). A bank can hold a music track and an
  ambience bed; the level's sound bank gives each its Type (`h_f92fb2b9` music, `ambient`; for
  names it doesn't define, an `amb` prefix is ambience). The music loops at half volume with the
  bed under it at 80% of that; a bank without music plays all its beds. Before, the first wave
  played, which on sdm_e13 and sdm_m03 was the bed, so their music was missing:

  | Map | Music | Ambience |
  |---|---|---|
  | sdm_e01 | caspian_action1 | |
  | sdm_e10 | ferix_action8 | amb_ferix_02 |
  | sdm_e13 | tmp_full-on1a-f | amb_singe_02 |
  | sdm_e34 | caspian_action20 | |
  | sdm_e40 | creepy_estuary_mutant_01 | light_bird_bed_1 |
  | sdm_m03 | caspian_action5 | amb_casp_04 |
  | sdm_m07 | | amb_shad_int2, ambiencehellish-2ch |

  `BF_NO_MUSIC=1` turns it off, `BF_MUSIC_VOLUME=<0-1>` sets the level, and
  `BF_DUMP_MUSIC=<file.wav>` writes the track and exits.

  **Since #223 the level's sound-triggers play it.** The bank's tracks are sounds of the level's
  sound bank (resource-type 2; a track's name hashes to its sound id: tutorial_01 is
  h_f5862d3f), and every level with a bank has sound-triggers that play them, most at
  FIRST_UPDATE (sdm_e34's h_1844a2b7: caspian_action20, looping, a 1 s fade-in), some as cues
  (the tutorial's: tutorial_01 from the start, stopped by a 168 s timer with a 3 s fade-out;
  caspian_tut_mellow1 in the mine field for 72 s; tutorial_02 when the gate has opened, for 238
  s; caspian_victory1 when the base is cleared). For such a level (`play_sound_triggers.rs`
  `plays_music`) `setup` plays nothing itself: the triggers do, at the music volume times the
  sound's own (the trigger's `volume`, else its library entry's: 0.8 for tutorial_01), with the
  trigger's fades ([triggers.md](../triggers/sound-and-dialog-triggers.md), "Sound- and dialog-triggers"). **One music
  track at a time** (#254, the game's 0x17eff0): a track (Type h_f92fb2b9) that starts while
  another still plays fades that one out over its fade-out as it fades in, and its own trigger
  loses hold of it, so its stop timer does nothing. In a fast tutorial run (the mine field
  before 168 s) mellow plays on past its 72 s until tutorial_02 replaces it at the gate, and
  tutorial_02 then past its 238 s until the victory sting replaces it. That is the game's own
  behaviour as its code reads; no take shows it (triggers.md, "One music voice"). The table above
  is the bank's mix the test tools' music picker still plays. Deathmatch still plays no
  music-bank track. The ambience beds are the levels' outdoor and indoor sound-triggers (Types 2
  and 3): an outdoor bed (sdm_e40's light_bird_bed_1, sdm_e34's wind, m01_a's amb_ferix_06) is
  full while the player stands outdoors and down at its `volume` (15-35%) indoors, an indoor one
  (m09_c's amb_shrike_int_01) the other way (triggers.md, "Types 2-5").
