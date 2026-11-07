*Part of [maps/](README.md), the former page `maps.md`.*

* **Music.** The game streams `data/sounds/<level>.xwb` by level name (sdm_e34's is
  caspian_action20, 59.7 s, stereo Xbox ADPCM, 22050 Hz). A bank can hold music tracks and
  ambience beds; each track's role is the Type of the sound entry that plays it (`h_f92fb2b9` music,
  `ambient` a bed), found by the wave's place in the bank file through the sound library's file
  records (see the next bullet), not by its name. The names mislead: `ambience19-2ch` and
  `ambiencehellish-2ch` are music, `caspian_amb_mux_temp` too. The music loops at half volume with
  the bed under it at 80% of that; a bank without music plays all its beds. (The `amb` name rule
  survives only as the test tools' fallback for a wave no sound entry plays, `play.rs` `is_bed`; the
  game's 38 banks have none.) The pairs the test tools' music picker mixes:

  | Map | Music | Ambience |
  |---|---|---|
  | sdm_e01 | caspian_action1 | |
  | sdm_e10 | ferix_action8 | amb_ferix_02 |
  | sdm_e13 | tmp_full-on1a-f | amb_singe_02 |
  | sdm_e34 | caspian_action20 | |
  | sdm_e40 | creepy_estuary_mutant_01 | light_bird_bed_1 |
  | sdm_m03 | caspian_action5 | amb_casp_04 |
  | sdm_m07 | ambiencehellish-2ch | amb_shad_int2 |

  `BF_NO_MUSIC=1` turns it off, `BF_MUSIC_VOLUME=<0-1>` sets the level, and
  `BF_DUMP_MUSIC=<file.wav>` writes the track and exits.

  **Since #223 the level's sound-triggers play it.** The bank's tracks are sounds of the level's
  sound bank (resource-type 2), and every level with a bank has sound-triggers that play them, most
  at FIRST_UPDATE (sdm_e34's h_1844a2b7: caspian_action20, looping, a 1 s fade-in), some as cues
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
  is the bank's mix the test tools' music picker plays. Deathmatch plays no music-bank
  track of the sound-triggers (play.rs `setup`, a demo choice from the first version), but for the
  four sounds of the two maps below (#403). The ambience beds are the levels' outdoor and indoor sound-triggers (Types 2
  and 3): an outdoor bed (sdm_e40's light_bird_bed_1, sdm_e34's wind, m01_a's amb_ferix_06) is
  full while the player stands outdoors and down at its `volume` (15-35%) indoors, an indoor one
  (m09_c's amb_shrike_int_01) the other way (triggers.md, "Types 2-5").

* **How a sound reaches its wave (#398).** A sound-trigger names a sound id. The sound entry has a
  wave-file hash, and the file record of that name (a `resource-type="2"` record in
  `sounds-<level>.xml`) carries an **absolute offset and size into `data/sounds/<level>.xwb`**. The
  game finds the record by the sound id (FUN_0017bf40) and seeks the bank file to its offset to
  stream the sound (FUN_00183710, FUN_00183570's path `<data dir>\sounds\<library>.xwb`); the bank's
  own header, entry table and wave names are **never read**. The sound id is the hash of the sound
  *entry's* name, which is the wave's name for 163 of the 187 entries of the 38 level banks and not
  for the other 24 (10 distinct ids): sdm_e40's music is h_10fb2011 (an entry whose name isn't
  recovered; the wave is `creepy_estuary_mutant_01`), sdm_m07's is h_e70fb918 (the entry
  `ambiencehellish_2`, the wave `ambiencehellish-2ch`). Every record lands exactly on one wave and
  every wave of every bank is some record's (the sweep, todo/398-level-music/answers.md).
  The demo looked a wave up by the hash of its name, so those sounds were silent; it now does what
  the game does: `SoundBank::music_file` (sound id -> file -> `MusicFile`: library, offset, size,
  format), then `MusicFile::read` (a seek and a read of that bank file, not the whole bank) and the
  record's format to decode (`play_sound_triggers.rs` `MusicBank`, which forgets its decoded tracks
  when another level is played). Only a record of the played level's own library plays: the game
  loads that level's sound library and also common's and mp_common's, but common.xwb is empty and
  mp_common has no bank file, so no other level's bank is reachable. The check is needed because the
  demo's `load_default_level` adds e01's and mp1's libraries, which the game never loads. What that
  brought back, where the sound-triggers start them:

  | Sound | Wave | Plays on |
  |---|---|---|
  | h_10fb2011 (music) | `creepy_estuary_mutant_01` | sdm_e40 (FIRST_UPDATE, looping, under the bird bed) and e05 (the level's opening music) |
  | h_e70fb918 (music) | `ambiencehellish-2ch` | sdm_m07 (FIRST_UPDATE, looping; not a bed: `amb_shad_int2` is its bed at 0.25) |
  | h_1494837e (ambient) | `amb_singe_02` | e13 and sdm_e13 (the bed under tmp_full-on1a-f) |
  | h_0aca9f39 (music) | `ambience19-2ch` | m02_a and m04_a (a one-shot cue) |
  | h_e80a7ee6 (music) | `caspian_amb_mux_temp` | m08_a (looping) |
  | h_f934a4be, h_ea61dfae, h_0169c0ce, h_f59329da, h_ee9a7860 (music) | the stings and cues: `osiris_victory1` (e05, e06, e40, m09_b, m09_c), `caspian_defeat1` (e05, m09_c), `ferix_ground_battle_02` (m05_a, m05_c, m05_x), `tutorial_01` and `tutorial_02` (m09_b's own cues, apart from the tutorial's) | where their triggers fire |

  e05's creepy trigger has `volume` 100 (h_eff0deab), against the -1 of sdm_e40's: it plays at 1.0
  (`volume / 100`, FUN_00206250), not the entry's 0.95: the log's "vol 0.50" is that 1.0 times the
  music volume 0.5, sdm_e40's "vol 0.47" the entry's 0.95 times it. The cut-content catalogue's 3.3
  ("five tracks no level can play") took a sound's id for the hash of its wave's name, which is why
  it was wrong.

* **Deathmatch music: sdm_e40 and sdm_m07 (#403, #398).** Deathmatch plays no music-bank track:
  play.rs `setup` plays no mix and play_sound_triggers.rs `voices` silences the sound-triggers'
  (the beds among them), the demo's own choice from its first version. The product owner asked for
  Estuary Showdown and Cavern of Fire to keep their music, and `play_demo_music.rs` (`CHOICES`) is
  the one named place that exempts four sounds, by sound id. **The tracks are the game's own**
  (#403 first took them for the demo's: the shipped game plays both, as the previous bullet shows),
  so what remains the product owner's choice is the exemption, not the tracks:

  | Map | Sound | Track | Role |
  |---|---|---|---|
  | sdm_e40 | h_10fb2011 | `creepy_estuary_mutant_01` | music (Type music, 102.6 s) |
  | sdm_e40 | h_e59d5087 | `light_bird_bed_1` | bed (Type 2 trigger h_fa8c83e7: full outdoors, 15-35% indoors) |
  | sdm_m07 | h_e70fb918 | `ambiencehellish-2ch` | music (Type music, 155.0 s; takes the one music voice's slot, #254, not a bed's share) |
  | sdm_m07 | h_0544355e | `amb_shad_int2` | bed (Type 0 trigger h_fa8c83e7 at vol 0.25) |

  * **Deathmatch** (a session, `BF_DEATHMATCH=1`, or an arena played alone): every other map's
    music-bank tracks stay silent as before (sdm_e34's caspian_action20 "no music here"); on these
    two maps `trigger_plays` lets the four sounds through and the map's own triggers play them at
    their own volume and level. `BF_NO_MUSIC` silences everything, these too. Outside a deathmatch
    session (a plain `BF_MAP=sdm_e40` run) nothing is silenced, so the triggers play the same.
  * **One music voice (#254).** Each map has one music trigger, so nothing is replaced; no
    deathmatch map has a match-end music.
  * **Logs.** A deathmatch session on these maps prints one line a row: `deathmatch music (#403, the
    product owner's choice) on sdm_e40: music creepy_estuary_mutant_01 (h_10fb2011) is played by the
    map's sound-trigger (kept in deathmatch)`. `BF_SOUND_LOG` prints each sound-trigger voice with
    where its bytes are, and under `BF_MUTE` reads and decodes the track all the same: `... music
    (sdm_e40.xwb, 2541744 bytes at 2048): not played (BF_MUTE; it would play, 104.1 s decoded)`, or
    "(BF_MUTE; and silenced here: no music here)" where deathmatch silences it, or "no sample
    data: <why>" where the record's bytes can't be had. `BF_TRIGGER_LOG` lists at the start every
    sound-trigger's track with its place (`... a music-bank track (e05.xwb, 538560 bytes at
    8833024)`), so the cues no trigger fires yet resolve too.
  * **Verified (#398)**, `BF_MUTE=1 BF_BACKGROUND=1 BF_SOUND_LOG=1 BF_TRIGGER_LOG=1`, `BF_MAP=<map>`
    (with `BF_DEATHMATCH=1` on the two deathmatch maps), `BF_CAPTURE_EVERY=1000` so only the log
    matters; "it would play" is with the track decoded to the length shown (the decoder's, 1.6%
    over the game's byte rate, 102.6 s for creepy):
    * sdm_e40, deathmatch: voice 2 `creepy_estuary_mutant_01 (h_10fb2011)` vol 0.47 "music (sdm_e40.xwb,
      2541744 bytes at 2048): not played (BF_MUTE; it would play, 104.1 s decoded)"; voice 1
      `light_bird_bed_1 (h_e59d5087)` (sdm_e40.xwb, 2473272 bytes at 2545664), 101.3 s. On #403's
      build voice 2 was "h_10fb2011 ... silenced here" and the demo's own loop played creepy.
    * sdm_m07, deathmatch: voice 2 `ambiencehellish-2ch (h_e70fb918)` (sdm_m07.xwb, 3839328 bytes at
      2195456), 157.2 s; voice 1 `amb_shad_int2` vol 0.25 (2193408 bytes at 2048), 89.8 s.
    * e05: voice 1 `creepy_estuary_mutant_01 (h_10fb2011)` vol 0.50 (e05.xwb, 2541744 bytes at
      2072576), 104.1 s; voice 2 `amb_estu_01` (2068992 at 2048). The stings, fired with
      `BF_TRIGGER_FIRE="h_e313b4db,10,1;h_ec19ba04,10,3"`: `osiris_victory1 (h_f934a4be)` (8833024,
      538560 bytes) 22.0 s and `caspian_defeat1 (h_ea61dfae)` (8187904, 644616) 26.4 s, each
      replacing the music voice (#254).
    * e13: voice 2 `amb_singe_02 (h_1494837e)` (e13.xwb, 1093392 bytes at 540672), 44.8 s; sdm_e13's
      (sdm_e13.xwb, at 2048).
    * m02_a and m04_a: `ambience19-2ch (h_0aca9f39)`, the cue's trigger listed at the start (m02_a.xwb,
      530496 bytes at 890880; m04_a.xwb, at 1830912), their beds amb_shad_05 and amb_shad_04 play at 0.
    * m08_a: voice 2 `caspian_amb_mux_temp (h_e80a7ee6)` (m08_a.xwb, 4018608 bytes at 3966976),
      164.5 s, replaced in the same frame by caspian_mellow5's trigger (the one music voice).
    * m09_b's tutorial cues, fired with `BF_TRIGGER_FIRE="h_1ffc313f,0,1;h_f55afb90,0,2.5"`:
      `tutorial_01 (h_f59329da)` (m09_b.xwb, 2092176 bytes at 15149056) 85.7 s and `tutorial_02
      (h_ee9a7860)` (2968056 at 12179456) 121.5 s.
    * Unmuted (`BF_MUSIC_VOLUME=0.01 BF_AUDIO_LOG=1`, sdm_e40 deathmatch, real time): "2 players, 2
      sinks, 2 playing" every 2 s from t = 2 to 20.
    * Unchanged against main's build (`bf_play_8dd94a2.exe`, the same recipe): sdm_e34 in
      deathmatch (the wind bed "would play", caspian_action20 "no music here"), the tutorial (the
      same 12 voices at the same times and volumes, tutorial_01 at 0.40) and m01_a (ferix_mellow1,
      amb_ferix_06 at 0), the logs differing only in the track names and places they now carry.
    * `cargo test`: `audio::tests` (a music sound finds its bytes by its file record on a fixture
      bank whose entry is named otherwise than its wave; the bytes read at the record's offset and
      decoded by its format; and on the real data, five banks' every wave lands on one record with
      its Type, and the records above), `demo_music::tests`, `sound_triggers::tests` (a track is
      located in the played level's bank only).
    * Not checked by ear: the tracks were counted playing (sinks), not listened to.
