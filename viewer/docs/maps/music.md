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
  is the bank's mix the test tools' music picker still plays. Deathmatch plays no music-bank
  track of the sound-triggers (play.rs `setup`, a demo choice from the first version), but for
  the two maps below (#403). The ambience beds are the levels' outdoor and indoor sound-triggers (Types 2
  and 3): an outdoor bed (sdm_e40's light_bird_bed_1, sdm_e34's wind, m01_a's amb_ferix_06) is
  full while the player stands outdoors and down at its `volume` (15-35%) indoors, an indoor one
  (m09_c's amb_shrike_int_01) the other way (triggers.md, "Types 2-5").

* **The product owner's music for two deathmatch maps (#403).** These are **demo choices, not the
  game's behaviour**: the shipped game plays neither `creepy_estuary_mutant_01` nor
  `ambiencehellish-2ch` on any level (the cut-content catalogue, 3.3; #396; #398). They sit in
  their map's own wave bank (`sdm_e40.xwb`, `sdm_m07.xwb`). Each map's music sound-trigger
  (h_1844a2b7, Type 0, FIRST_UPDATE, looping, a 1 s fade-in, vol 0.47) asks for a sound (sdm_e40's
  h_10fb2011, sdm_m07's h_e70fb918) whose music-bank file is as long as that wave (102.62 and
  155.01 s by the byte rate; the waves are 2541744 and 3839328 bytes) but whose id is the hash of
  no wave of the bank, so the demo finds no sample data for it and it stays silent. (The sdm_e40 and
  sdm_m07 rows of the first table above are the *bank's* mix, which the test map's U menu plays;
  this table is what the maps play.) It is `CHOICES` in `play_demo_music.rs`, one named place, each
  row commented as the product owner's choice; it is the explicit exception to any rule that drops
  the waves of a bank that no sound entry names (#398):

  | Map | Track | Role | How it plays |
  |---|---|---|---|
  | sdm_e40 | `creepy_estuary_mutant_01` | music | the demo: a loop at the music volume x 1.0 (`music_mix`'s music share), `LevelMusic` |
  | sdm_e40 | `light_bird_bed_1` | ambience bed | kept: its own sound-trigger (h_fa8c83e7, Type 2: full outdoors, 15-35% indoors), at its own level, under the music |
  | sdm_m07 | `ambiencehellish-2ch` | ambience bed | the demo: a loop at the music volume x 0.8 (`AMBIENCE_VOLUME`, the bed share of `music_mix`, as the test map's U menu gives it alone) |
  | sdm_m07 | `amb_shad_int2` | ambience bed | kept: its own sound-trigger (h_fa8c83e7, Type 0, vol 0.25), beside the new bed |

  * **Deathmatch.** The sound-triggers' music-bank tracks are silenced in deathmatch (play.rs `setup`
    and play_sound_triggers.rs `voices`), the beds among them, since a bed is a bank track too. Before
    #403 the two kept beds were silent in a deathmatch session (a plain `BF_MAP=sdm_e40` run, which
    isn't one, played the bird bed). On these two maps `trigger_plays` lets the rows marked "kept"
    through, so the bed is audible; every other track on them (the two music triggers' unplayable
    sounds) and on the other deathmatch maps stays silent as before. `BF_NO_MUSIC` still silences
    everything, the table too.
  * **The demo's rows** start as the level does (`OnEnter(Playing)`, after `setup`), read the track by
    name from the bank, and are `LevelMusic`: the test tools' music picker and level switch stop them
    like a level's music. They play whether or not the session is deathmatch, but only on those two
    maps (a campaign level never has the names).
  * **One music voice (#254).** There is one music row a map at most (a test), and the demo's loop
    isn't a sound-trigger voice, so the 0x17eff0 slot isn't involved. The maps' own music triggers
    find no data and start no voice, so nothing is replaced. The demo has no deathmatch match-end
    music, so there is nothing for the loop to collide with.
  * **Logs.** On these maps, always, one line per row: `demo music (#403, the product owner's choice)
    on sdm_e40: music creepy_estuary_mutant_01 x1.00, looping` ("(BF_MUTE: not played)" under
    `BF_MUTE`), and `... ambience light_bird_bed_1 is played by the map's sound-trigger (kept in
    deathmatch)`. `BF_SOUND_LOG` prints each sound-trigger voice; under `BF_MUTE` it ends "not
    played (BF_MUTE; it would play)" or "(BF_MUTE; and silenced here: no music here)".
  * **Verified (#403)**, `BF_MUTE=1 BF_BACKGROUND=1 BF_SOUND_LOG=1`, `BF_MAP=<map>` with and without
    `BF_DEATHMATCH=1`, then unmuted at `BF_MUSIC_VOLUME=0.01` with `BF_AUDIO_LOG=1`:
    * sdm_e40, deathmatch: the demo's creepy loop x1.00; voice 1 `h_e59d5087` (light_bird_bed_1) "it
      would play" (before: silenced); voice 2 `h_10fb2011` silenced. Unmuted: "2 players, 2 sinks,
      2 playing" (creepy and the bird bed), held at t = 2 and 4 s.
    * sdm_m07, deathmatch: the demo's `ambiencehellish-2ch` x0.80; voice 1 `h_0544355e`
      (amb_shad_int2, vol 0.25) "would play" (before: silenced), voice 2 `h_e70fb918` silenced, voice 3
      the positional `h_14abd271` as before. Unmuted: 3 players, 3 sinks, 3 playing.
    * Unchanged: sdm_e34 and sdm_e13 in deathmatch (their music triggers silenced, e34's wind not a
      bank track as before), the campaign level m01_a (no demo-music lines, its triggers as
      before), the test map's U menu (the five unused tracks still pick and log as under #396:
      `BF_TEST_MUSIC=...,unused:<name>`; the `--test` session on `BF_MAP=sdm_e40` logs the bank's
      mix "music creepy_estuary_mutant_01 x1.00, ambience light_bird_bed_1 x0.80").
    * `cargo test`: `demo_music::tests` (the table; only the kept beds pass the deathmatch silence).
    * Not checked by ear: the lead's check with sound on.
