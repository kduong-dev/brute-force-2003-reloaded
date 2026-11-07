*Part of [play/](README.md), the former page `play.md`.*

Sounds (`src/bf/audio.rs`) come straight from the game's banks: `sounds-<level>.xmb` maps sound
ids to Xbox ADPCM data in `sounds-<level>.mem`, decoded to 22 kHz WAV in memory. Each character's
archetype names its jump grunt and footstep type (Tex 1, Flint 2, Brutus 3, Hawk 4), and
`world-materials.xmb` gives every surface four footstep variants per type plus landing and slide
sounds. Footsteps fire when a toe comes down into the lowest quarter of its swing in the clip
being played, each one FUN_00119020's request (**Footsteps (#324)**, below: the surface's pick, a
volume by the gait, 2D for the controlled character and at the foot for the others, a random
pitch); in a liquid the step is the liquid's wading step or nothing, by the depth
(FUN_00119020's liquid branch, #5: levels.md, "What a liquid plays"). The footsteps' and the
liquids' sounds are requests with play parameters of their own (`WorldSound`: a position heard
by the library's distances from the listener and dropped beyond its max distance, a volume, a
pitch in octaves played as the speed); `BF_SOUND_LOG` prints each with its kind, volume, a
footstep's gait factor, pitch and distance. Surface sounds live in level archives: `BF_LEVEL=<name>` (default `e01`) picks the
level; `mp1`'s bank is merged in too, since a mission only carries its own squad's footsteps. They
load after the played map's own data (`load_map`, `load_default_level`) and only fill in what it
lacks: where two levels name something alike the map's is the one found (#338, levels.md "Which
level's data").
On a level, its sound-triggers play their own voices (#223: voice lines, door and gate sounds,
ambience, the music cues; 3D ones by DirectSound's distance law from the camera): see
[triggers.md](../triggers/sound-and-dialog-triggers.md), "Sound- and dialog-triggers", and their `BF_SOUND_LOG` lines
("trigger voice ...").

**The Xbox ADPCM decode (#399).** A block is 36 bytes a channel: a 4-byte header (an i16 sample, a
u8 step index, a 0 byte) and 32 bytes of 4-bit codes, low nibble first; with two channels both
headers come first, then 4-byte runs of codes taking turns. **It gives 64 samples: the header's
sample, which is the block's first, and nibbles 0 to 62; the 64th nibble is padding** and isn't
decoded (`audio::adpcm_decode_mono`, `ADPCM_SAMPLES`; `tools/xwb_tool.py` the same). Until #399 the
decoder gave 65 (the header and all 64 nibbles): every sound played 1.6% long, with one extra
sample per block, the decode of padding, which shows as lines at the multiples of 22050 / 65 =
339 Hz in a track's spectrum (the spectrum bullet). What says 64, and the header rather than the
last nibble:
* **The game's format.** FUN_00183c30 fills the Xbox ADPCM WAVEFORMATEX with a `wSamplesPerBlock`
  of 0x40 (at +0x12) and a bytes a second of (rate >> 6) x 36 x channels (12384 at 22050 Hz mono),
  which counts 64 samples to 36 bytes; the sound system's length (0x17f620) is the file's size over
  it. The sound libraries' `time` agrees: the streamed voice lines' and the music tracks' (559 and
  6 records in the tutorial's) are that number, the in-memory sounds' (140) are the size over 12403,
  which is 22050 x 36 / 64 with the rate kept whole, and so the audio's length at 64 samples a
  block to a part in 10^5. Over every file record of the 61 decoded libraries (24,323 Xbox ADPCM
  records, each a whole number of blocks) the decoded length over `time` is 0.99999 to 1.00000 for
  the 9,840 in-memory records, 0.99846 to 0.99991 for the 14,297 streamed ones (the rate's
  truncation: 0.99846 at 22050 Hz) and 0.99846 for the 186 music records; at 65 samples a block it
  would be 1.0156, 1.0141 to 1.0155 and 1.0141.
* **The last nibble is padding.** In each of 21,020,334 channel-blocks (the music banks' 186 waves,
  the English voice banks' 14,297 lines, the in-memory sounds of the tutorial, e34 and common) the
  64th nibble is 0, where the 63 before it have the full spread of codes (entropy 3.68 bits each).
  The header's reserved byte is 0 in all of them.
* **The step index chains through 63 nibbles.** The next block's header index equals the decoder's
  index after the 63rd nibble in each of 21,005,242 consecutive pairs (after the 64th it is 2% by
  chance): the encoder coded the header's sample and 63 more.
* **The seams.** The mean step into a block's first sample is 1.002 times the mean step inside the
  blocks (30 music waves, 400 voice lines) with the header kept and the last nibble dropped. With
  the header dropped and the 64 nibbles kept it is 1.58 (music) to 1.72 (voice): a sample is
  missing at every seam. Wave by wave (the first 4000 blocks of each of the 186 music waves) the
  header-kept ratio lies between 0.96 and 1.06 and the header-dropped one between 1.1 and 2.0
  (mean 1.77): clear in aggregate and on most waves, not on every one (the test reads the
  tutorial's bank: 1.003 against 1.652). The old 65-sample reading's is 1.03, with the padding's
  decode in it. The
  spectrum agrees: over the first 30 harmonics of the block rate, a track's long-term spectrum has
  the lines +0.8 dB over the floor beside them with the header kept (what the music's own partials
  give) and +4 to +6 dB with the header dropped or the old reading, a buzz.
* **The capture.** take03 of #28 (the tutorial's opening cutscene, from xemu) plays Bratton's four
  lines (script and per-stretch results: `todo/399-adpcm-block/`). Each decoded line was cut into
  2 s stretches; each, resampled by a factor f, was matched against the take's sound (normalised
  cross-correlation of the 300-3400 Hz band): the best f is 1.000 on 12 of the 13 stretches
  (r 0.36-0.81; the other 1.002, r 0.30), where 65 samples a block would give 1.0156 (r there
  0.12-0.27, the noise). The check can tell: the old 65-sample decode of the same lines needs
  f = 64 / 65 = 0.9846 (on 11 of its 14 stretches; 0.985 to 0.991 on the three weakest). The
  take's sound has gaps, which a whole line shows (the "refresher" line, 6.74 s, is found 123 ms
  later 5.6 s into it than at its start, in steps of roughly 10-20 ms, while each stretch between
  them keeps f = 1.000): xemu's audio underruns while the game runs slow (the take's watch log
  has its app clock advance 0.962 s a second), so only the time scale within a stretch can be
  read, not a line's length.
What it leaves alone: `SoundBank::length` is the game's count, so a sound-trigger's FINISHED, the
dialog queue's lines and the faces' visemes (keyed in seconds into the line) are unchanged; the
audio is now 344 x 64 / 22050 = 0.99846 of that count (the byte rate takes the rate's whole 64s):
10 ms in a 6.7 s line, 0.13 s in an 84 s track, the game's own difference. Before #399 the audio
ran 1.4% over it (0.09 s past FINISHED on the 6.75 s "refresher" line). The `voice_end` of a
sound-trigger (play_sound_triggers.rs) still counts the game's length, not the hardware's end of
data. The step's arithmetic is unchanged: the IMA reference's sum (a step's eighth, and its quarter,
half and whole by the code's bits), which fits the game's data better than the formula ffmpeg uses
((2 x code + 1) x step >> 3): on quiet stretches the next header's miss from the decoder's last
sample averages 7.6 with the sum and 11.2 with the formula on the voice lines (7.8 and 12.4 on the
music). ffmpeg reads the banks' tag 0x69 as standard IMA, 65 samples a block, with that formula
(its decode of `xwb_tool.py extract --raw`'s WAVs equals the formula's, header and 64 nibbles, in
all 3.8 million samples tried), so it plays them 1.6% long.
Verified: `cargo test --lib audio` (`a_block_is_the_header_and_63_nibbles`,
`a_stereo_block_is_two_channels_of_64_samples`, `the_real_blocks_keep_the_header_and_drop_the_padding`
(the tutorial's music bank), `a_levels_sounds_play_as_long_as_the_game_counts_them` (all the
tutorial's sounds against `SoundBank::length`)); `BF_MAP=tutorial BF_DUMP_MUSIC=<file.wav>` and
`BF_DUMP_SOUND_IDS=<dir>:e3656325,ea6253b3,028b1cfc,e759c574` give WAVs equal sample for sample to
`xwb_tool.py extract`'s; the tutorial's `BF_SOUND_LOG` / `BF_TRIGGER_LOG` before and after differ
only in the decoded lengths ("it would play, 84.7 s decoded" is now 83.4). Not checked by ear.

Test hooks: `BF_AUTOPILOT=1` plays a scripted run (idle, run, turn, sprint, aimed walk,
backpedal, dodge, standing jump, running jump, aimed fire, weapon switch, firing on the run); `BF_TEST_EXPLOSION=1` sets off the selected grenade type 6 m ahead every 1.5 s; with `BF_TEST_GOTO`, `BF_TEST_THROW=<s>[,<hold s>,...]` holds the grenade button (G) at those times (see **Grenades** for the rest of its hooks); `BF_ANIM_PROBE=<script>,...` prints each character's clips by script name (duration, root motion over a cycle, the clip's own speed h_f9628571, events); `BF_DUMP_TEXTURE=<file>:<hex id>` writes a decoded texture (raw RGBA after a u32 width and height). `BF_AUTOPILOT_SPIN=<rad/s>` makes it look down and turn the camera during the standing fire; with `BF_CAPTURE=<dir>` it saves every frame at a
fixed 15 fps (`BF_CAPTURE_EVERY=<n>`: only every n-th, for a long run at that step whose logs
are what's wanted, #294). `BF_CAPTURE_EARLY=1` captures from the map's first frame instead of after the warm-up and the wait for the character and the LOADING cover (#311, [menu.md](../menu.md) "The level's start behind LOADING"): a capture otherwise starts once the cover is down, and so at the start of the level's fade from black (#266, [mission.md](../mission.md) "A level's start": its first second is the ramp; `BF_NO_FADE=1` leaves the fade out, and since #390 a script's 120 fades too: [cutscenes.md](../cutscenes.md) "The screen fade"). `BF_MUTE=1` silences audio, `BF_SOUND_LOG=1` prints every sound event with its time
and action, `BF_DUMP_SOUND_IDS=<dir>:<id,id..|all>` decodes sound ids to WAV, `BF_DUMP_SOUNDS=<dir>` writes the character's jump and surface sounds as WAV files
and exits. `BF_DUMP_WEAPONS=1` lists every character's weapons as loaded (definition, model parts,
hardpoints, muzzles, fire sounds), then the map's definitions with more than one muzzle, and exits; `BF_DUMP_TYPES=1` (or `all`: every archive) lists the
character types, their motion sets' clips and the placements, and exits
([character-types.md](../character-types.md)); `BF_SHOT_LOG=1` prints each shot's barrel and shot
directions; `BF_FEET_LOG=1` prints, every frame, the controlled character's soles (the posed
model's lowest skinned vertex) and its two foot bones over its ground, whether it moves, and the
overlays playing with their times (a pickup, a use, a reload, a throw: #293's check of the
pickup clips' feet). Camera hooks ([camera.md](../camera.md)): `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` set the
start (with `BF_TEST_GOTO`, `BF_CAMERA_PITCH` holds the pitch throughout, e.g. to fire down at
something; held to +-60 degrees), `BF_CAMERA_DISTANCE` sets every character's boom,
`BF_CAMERA_AIM_AT=x,y,z` turns the aim so the crosshair's ray passes through a point (the recipes
here from before #126 aimed with `BF_CAMERA_PITCH` on the old camera: the same pitch now puts the
crosshair elsewhere), `BF_CAMERA_LOG=1` prints the camera, `BF_VIEW_YAW` turns only the rendered
view about the pivot (aim unchanged), `BF_START_WEAPON=1` starts with
the second weapon. Captures wait 60 frames for shaders to compile before the clock starts.
