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
  ambience bed under it at 80%, all beds if it has no music), looping at the music volume. **A track's
  role is the Type of the sound entry that plays it** (#398), found by the wave's place in the bank
  file through the sound library's file records, as the game finds a sound's wave
  ([maps/music.md](../maps/music.md), "How a sound reaches its wave"); the `amb` name rule is only
  the fallback for a wave no entry plays (`play.rs` `is_bed`; the game's banks have none). The
  map's own music is stopped first (`LevelMusic`). `BF_MUTE` loads and logs but plays nothing.
  As with skies, picks while a bank loads are ignored.
  * The test map's own music is m04_a's bank (#302, `TEST_MUSIC`: test_casp1_amb01 with the
    ambience bed amb_shad_04 under it at 80%; before #398 the bed was `ambience19-2ch`, which the name
    rule took for a bed and is a music track, the cue m04_a's trigger h_f4d6ed63 plays), loaded so
    as the flat floor starts and playing from about a second in, without a message; "this map's own"
    plays it again there, and Left / Right step from it. A pick while it's still loading takes its
    place. `BF_NO_MUSIC` leaves it off. `BF_TOOLS_LOG` or `BF_SOUND_LOG` print "test map: its music,
    m04_a's bank, loading" and then "test tools: music of m04_a: music test_casp1_amb01 x1.00,
    ambience amb_shad_04 x0.80, looping" ("(BF_MUTE: not played)" under `BF_MUTE`).
  * **The tracks of a bank (#398, the product owner's request).** The highlighted bank's "tracks:"
    line names its waves, but none could be played on its own. **Shift+Enter (or Shift+click) on a
    bank, or on "this map's own", opens its track list**; the music menu's footer says so. (Not Tab,
    which is the game's B button and opens the item box, nor T, the squad tool, Space, which
    jumps, Right, which is the next bank, nor Backspace, the demo's key for leaving the session:
    Shift+Enter is bound to nothing.) The list is every wave of the bank file in the bank's order,
    numbered from 0, with **the Type of the sound entry that plays it** (`music`, `ambience`, or "no
    entry" for a wave none plays: every wave of the game's banks has one) and its **length**, as
    the game reckons it (the file record's size over the format's bytes a second: 102.6 s for
    creepy, the figure the log gives; the decoder's own is 1.6% longer). Under the picked track a
    line gives its file record (`sdm_e40.xwb: 2541744 bytes at 2048`), the sound ids that play it
    (`sound h_10fb2011`) and what it plays as alone (`music x1.00`, or `ambience x0.80` for a bed).
    A bank's list is read once, on a thread (its sound bank and the start of its bank file,
    `SoundBank::bank_tracks`: each wave is matched to its file record by its offset, and the entries
    that name that record give its Type), and kept for the session.
    * **Keys.** In the list: Up / Down (or the wheel) pick a track, **Enter or a click plays it
      alone, looping**, at its share of the music volume (music x1.00, a bed x0.80), the map's music
      stopped as a bank's mix stops it; the list stays open to try the next one. **Esc or Backspace
      goes back to the banks** (Esc no longer closes the whole menu while a list is open, and
      Backspace, which leaves the session, is taken off the demo's own handling while it is: the
      first Backspace on the banks still says "Backspace again to leave for the menu"; `U` closes
      the menu from either). Left / Right and the banks' picks wait while a list is open.
    * **Hooks** (`BF_TEST_MUSIC=<s>,...`): `tracks:<level>` opens the menu on that level's list (a
      frame of it, as Shift+Enter), and `track:<level>:<which>` plays one track of that level's bank
      alone, by its place from 0 (`track:m03_b:1`), its name in any case or a part of it
      (`track:m03_b:caspian_action7`; a number counts as a place first, since the names have digits
      in them): the list is read first, then the track, with no menu on screen; a list that is open
      for that bank moves its arrow to the track. `BF_TEST_KEYS` presses the keys: `shift` and
      `enter` due at the same time make Shift+Enter, `backspace` and `esc` go back.
    * **Logs** (`BF_TOOLS_LOG`): "test tools: reading the tracks of m03_b", then "test tools: tracks
      of m03_b: 5: caspian_action18 (music, 36.8 s), caspian_action7 (music, 27.4 s),
      caspian_action10 (music, 39.5 s), amb_casp_01 (ambience, 87.6 s), caspian_action5 (music, 77.5
      s)"; a track played: "test tools: track caspian_action7 (m03_b.xwb) read: 679536 bytes at 915456
      of m03_b.xwb, 22050 Hz 2-channel audio, 27.8 s once decoded" (`BF_TOOLS_LOG` or
      `BF_SOUND_LOG`; the decoder's length) and "test tools: music of track caspian_action7
      (m03_b.xwb): music caspian_action7 x1.00" with "(BF_MUTE: not played)" under `BF_MUTE`, else
      ", looping". A track that isn't there says "no track 99 in e05 (6 tracks)" and a level that
      isn't one "track:nowhere:1 isn't track:<level>:<name or place> of a level". As with a bank,
      picks while one loads are ignored.
  * **The bank tracks** (#396, called "unused" until #398). The list ends with a group headed
    "[bank tracks: sound entry named otherwise]": five music and ambience tracks that sit in the
    levels' streaming wave banks (`data/sounds/<level>.xwb`) and whose sound entries are named
    otherwise than their waves, so a lookup by the wave's name found no sound for them: the
    cut-content catalogue's 3.3 took that for "no level plays these". **The levels do play all
    five**, by the sound's file record (#398, [maps/music.md](../maps/music.md)); the group stays
    as a quick way to each. Eight bank entries in all:

    | Track | Banks (the first is read) | Plays | Sound | Role |
    |---|---|---|---|---|
    | `creepy_estuary_mutant_01` | e05.xwb, sdm_e40.xwb | 104 s | h_10fb2011 | music |
    | `amb_singe_02` | e13.xwb, sdm_e13.xwb | 45 s | h_1494837e | bed |
    | `ambience19-2ch` | m02_a.xwb, m04_a.xwb | 22 s | h_0aca9f39 | music |
    | `caspian_amb_mux_temp` | m08_a.xwb | 165 s | h_e80a7ee6 | music |
    | `ambiencehellish-2ch` | sdm_m07.xwb | 157 s | h_e70fb918 | music |

    (The lengths are what the demo's decoder gives, one more sample in 64 than the game's byte rate
    counts, 1.6% longer: the log prints them; each wave is the same bytes in each of its banks.) The
    pick's panel says "in the wave banks e05.xwb, sdm_e40.xwb; its sound is h_10fb2011, not the hash
    of its name". Picking one (Enter, a click) reads that wave out of its first bank on a thread
    (the wave's bytes at its file record's place, the record's Type from that bank's sound bank:
    `load_track`), stops the map's music as the track comes in (as a level's bank does) and plays it
    **alone**, looping, at the music volume (`BF_MUSIC_VOLUME`) times its share by the Type of its
    sound entry: the four music tracks at x1.00, the bed `amb_singe_02` at the bed's x0.80. (The name rule
    of #396 took `ambience19-2ch` and `ambiencehellish-2ch` for beds and played them at x0.80: their
    entries are Type music.) Left / Right step through the banks only, from the bank the track was
    read from; the banks' list, the "this map's own" entry and every bank's mix are as before. The
    hook `BF_TEST_MUSIC=<s>,bank:<name>` picks one (its name in any case, a part of it, or its place
    in the group from 0: `BF_TEST_MUSIC=2,bank:caspian`; `unused:<name>` is its first name and still
    works). Logs (`BF_TOOLS_LOG` or `BF_SOUND_LOG`): "test tools: bank track creepy_estuary_mutant_01
    (e05.xwb) read: 2541744 bytes at 2072576 of e05.xwb, 22050 Hz 2-channel audio, 102.5 s once
    decoded (#399: 104.1 before the decoder gave 64 samples a block); the same wave is in sdm_e40.xwb", then "test tools: music of bank track
    creepy_estuary_mutant_01 (e05.xwb): music creepy_estuary_mutant_01 x1.00" and, under `BF_MUTE`,
    "(BF_MUTE: not played)" (else ", looping"); `bank:nothing` says "no bank track nothing". As with
    a bank, picks while one loads are ignored: a pick reads the level's sound bank for the Type, which
    takes a second or two, as a bank's mix does.
