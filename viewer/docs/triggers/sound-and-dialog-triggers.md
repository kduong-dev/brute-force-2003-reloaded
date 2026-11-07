*Part of [triggers/](README.md), the former page `triggers.md`.*

## Sound- and dialog-triggers

#223, `play_sound_triggers.rs` (a child of play_triggers.rs: its classes are the runtime's).
From the code: the sound-trigger's vtable 0x395d50 (its parser 0x205d00, handler 0x206450,
update 0x206690, disable FUN_00206610 and the helpers FUN_00205fb0-FUN_00206b50), the
dialog-trigger's 0x3952a0 (FUN_00206cb0, 0x206d20, 0x206ee0), the world's dialog queue
(world+0x208: FUN_00179d60, FUN_00179ec0, FUN_0017a0f0, FUN_0017a130) and the sound system
(DAT_00469940, vtable 0x399c40).

* **Attributes** (0x205d00; the constructor 0x1fce50's defaults): `sound-object` the sound
  library entry; `enable-looping`; `Type` (the SND_IS_ names at 0x3bf200): 0 GLOBAL (heard alike
  everywhere), 1 POSITIONAL (at its place, 3D), 2 AMBIENT_OUTDOOR, 3 AMBIENT_INDOOR,
  4 POSITIONAL_INDOOR, 5 POSITIONAL_OUTDOOR (below); `max-distance` and h_0269d394 its max and min
  distance (-1: the library entry's `h_fd40e332` and `falloff`); `volume` (h_eff0deab, 0-100; -1:
  the entry's own h_eff0deab, 0-1, or 1 without one); h_1f625953 / h_05cad875 its voice's fade-in
  / fade-out (s); h_15bc1769 "only while a player is within its `max-distance`"; h_e725a1a5 the
  squad's chatter hushed while it plays (a timed mute for its length, keyed by the trigger, taken
  off at its stop: FUN_002062a0 / FUN_002063c0; the dialog queue's lines and a dialog-trigger's
  line hush it the same way: [chatter.md](../chatter.md), "Timed mutes"); `HARDPOINT` and `priority` (h_e146dc86)
  aren't read by these paths.
* **A sound's length** is the sound system's (its vtable +0x64, 0x17f620): the file's size over
  its format's bytes a second (FUN_00183c30: Xbox ADPCM counts 36 bytes a channel for each whole
  64 of the rate). The sound bank's own `time` attribute agrees (terminal_03: 12852 / 12384 =
  1.0378 s, `time="1.037791"`).
* **PLAY_SOUND, OPERATE**: it's on. Without h_15bc1769 it starts at once (FUN_002062a0): a
  one-shot still playing isn't restarted, a looping one playing goes on; otherwise its time is
  its sound's length and its voice starts (FUN_002060c0), 2D for Types 0, 2 and 3 and for any
  sound whose library entry's `falloff` is -1 (FUN_0017c170 makes a 3D voice only otherwise),
  else at its first object's place (a spawn-trigger's copy, a placed object) or its own.
* **A 3D voice's distance**: every one gets a rolloff curve as it's made (FUN_00181770 for a
  buffer, 0x1836eb for a stream: IDirectSound*_SetRolloffCurve with the five gains at 0x38dae8,
  1, 0.375, 0.125, 0.0625 and 0), which the dsound library's attenuation (FUN_00314e64) spreads
  over five equal stretches from the min distance to the max, each linear from the last point
  (1.0 at the min) to its next: full within the min, 0.375 two fifths of the way, nothing at the
  max and beyond (the distance is held at the max). The buffers aren't made with
  DSBCAPS_MUTE3DATMAXDISTANCE (their flags are DSBCAPS_CTRL3D alone, 0x1817c8), but the curve
  ends at 0, so they're silent past it all the same. The listener is where the character the
  camera follows stands (FUN_00178c10: the camera's facing, the character's place).
* **With h_15bc1769** its update starts it once the player's character is within its
  `max-distance` (its count, FUN_002062a0) and its voice once the listener is too (the same
  character: both together), and stops it, fading out, once none is (FUN_00206b50); a one-shot
  that has played is off. So a door's sound counts its length from the frame the player is within
  range, its voice with it: sdm_e34's door sounds start 0.07 s after their ANIM_START, beside
  the player, and have played their 0.93 s at 1.00 s.
* **The end**: its update counts the time down by dt while it runs; when it's out (a one-shot)
  FINISHED (25: its iD, no subject, data -1) goes to its targets, for Types 0 and 1
  (FUN_001fcf00), and it's off and stopped. The count starts in the frame it plays, so FINISHED
  comes on the frame the length has run since that frame's start.
* **STOP_SOUND, STOP, STOP_DIALOG**: stopped and off; the voice fades out over its fade-out.
  **PAUSE_SOUND / RESUME_SOUND** pause and resume the voice and the count (no tutorial trigger
  has them).
* **The fades** are the voice's envelope (the sound system's +0x50, FUN_0017f330 -> the voice's
  +0x2c, 0x181c90: IDirectSoundBuffer_SetEG with the fade-in as its attack and the fade-out as
  its release, x 48000 / 512 samples, sustain 255); their shape is the sound chip's, not the
  game's code. A stop with its fade-out is StopEx(0): for a stream (the music) 0x182f30 calls
  IDirectSoundStream_FlushEx with ASYNC | ENVELOPE (5), and CMcpxStream_Stop runs the
  envelope's release for flag 4; StopEx(1) is ASYNC alone, at once. Measured: take04's tutorial_01 stop (a 3 s fade-out) falls linearly in dB, about
  60 dB over the fade (its level at 0.25 s steps from 199.0 s: 0.65, 0.30, 0.25, 0.16, 0.09,
  0.03), so the music is all but gone 1.3 s in; the fade-ins (take09, take11) rise linearly.
* **PLAY_DIALOG** queues its sound, once, on the world's dialog queue, which plays one line at a
  time, each for its length, 2D at its library volume, the next as soon as one has ended
  (FUN_00179ec0, the world update's last step); FINISHED once its line is done (FUN_0017a0f0),
  on the next frame's update.
* **DISABLE** is its own (its vtable +0x48, FUN_00206610): the voice plays out, but its count
  and queued line are dropped, so no FINISHED comes; its update, which the world runs only while
  +0x3d bit 6 is set, stops once the voice has ended. Its update sends its FIRST_UPDATE even when
  it's disabled (0x2066a4).
* **Types 2-5**, the outdoor and indoor ones, run the same logic and send no FINISHED; they play
  at a level (+0xa0) that follows where the listener stands (FUN_00206880): their `volume` / 100
  raised to 1 by the share of the listeners indoors (3 and 4) or outdoors (2 and 5), and 0 with
  h_15bc1769 while the listener is beyond its `max-distance`; the level moves there at 4 a second,
  a voice starting when it rises from 0, stopping at 0, its volume following (FUN_00206a20). A
  listener is indoors (the sound system's +0xc8 byte for it) when the character its camera
  follows stands on an indoor floor (FUN_00178c10 from the character's ground record bit 0,
  FUN_0012dab0): on the terrain, or on static geometry (a game-object, OT_STATIC_GEOMETRY) or a
  platform (OT_PLATFORM) of light-usage LU_GEOSET, the triangle's own bit (header bit 30,
  `MATERIAL_INDOOR`; no terrain triangle has it); on one of LU_INDOOR always, of LU_OUTDOOR never
  (`light-usage` h_1eb6a442, the LU_ names at 0x3bf954); on anything else it stays as it was.
  39 of the game's 1417 sound-triggers; the levels' ambience beds are these:
  amb_ferix_06 (m01_a, m01_b, m05_b, m05_y), amb_shrike_01 / _02 (m09_a, m09_x, m09_b),
  light_bird_bed_1 (sdm_e40) and sdm_e34's wind are Type 2 at 15-35, so full outdoors and that
  share indoors; amb_shrike_int_01 (m09_c) and m01_a's mechanical_amb_int_1 are Type 3. The
  tutorial has none. The same flag picks a 3D voice's reverb send (0x17f390: the world's indoor
  or outdoor amount, its +0xb58 / +0xb5c; not modelled).
* **The dialog-trigger**: PLAY_DIALOG or OPERATE on its first object's character (the tutorial's
  four name trigger_spawn8, whose copy tex_1 is the demo's Tex, and trigger_spawn76): a dead one
  sends FINISHED at once; a living one says the line (FUN_001267b0) and the update sends
  FINISHED once it's said (FUN_00126810). Here the line plays as a plain sound at the character
  (full for the player's, quieter with distance for another's, as the demo's character sounds
  are), and the character's face says it: its mouth follows the line's viseme tracks
  (FUN_001435d0 at 0x1267d1; a STOP stops it, FUN_001267f0; `speakers` puts both on the face after
  the characters' step, as the game's trigger loop comes after the objects' ticks: #325,
  [faces.md](../faces.md), "Lip-sync"). The world's dialog queue has no speaker and moves no face.
  With no such character in the world an entry does nothing, not even counted.
* **Music**: a level's music bank's tracks are its sound-triggers' sounds (resource-type 2), played
  through the music volume and marked as the level's music ([maps.md](../maps/music.md), "Music").
* **One music voice** (#254): a sound of Type h_f92fb2b9 is class 1 (the library loader,
  0x17bbe9: FUN_0017aa00 maps h_f92fb2b9 to 1, h_e34c5f66 2, h_012ed199 3, anything else 0, into
  the entry's +0x18, which FUN_0017c170 gives the voice through the system's +0x4c). As a voice
  plays (the system's +0x28, 0x17f070), 0x17eff0 runs before its Play: for a class-1 voice, when
  the current one (the system's +0x1a4) is still playing (0x17efb0), that one is stopped with
  StopEx(0), its fade-out (a stream's 0x182f30: IDirectSoundStream_FlushEx, ASYNC | ENVELOPE),
  and the handle-map node of the **new** voice is erased; then the new voice is the current one.
  So a track that starts while another plays crossfades with it (the old one's fade-out, the
  new one's fade-in), and **its own trigger loses it**: its later STOP, volume, pause and move
  find no voice and do nothing. The tracks don't share a stream (the pool has 8, FUN_00182120 at
  0x17f999; FUN_001822d0 takes a free one): this is the only rule that keeps two from playing.
  The beds (Type "ambient", class 0) play under the music untouched. This is the game's own
  behaviour read from the code; no take has a track starting while another plays. In the
  tutorial:
  * the mine field's circle before timer525's 168 s: tutorial_01 fades out over its 3 s as
    caspian_tut_mellow1 fades in over its 3 s, and then **mellow plays on past timer527's 72 s**
    (its stop does nothing), looping until the gate;
  * the circle after 168 s: tutorial_01 has stopped, mellow keeps its handle and timer527 stops
    it at 72 s (take09: mellow from ~29.5 s to ~100.5 s, cross-correlated with its wave);
  * the gate: tutorial_02 replaces mellow if mellow still plays (then **tutorial_02 plays on
    past timer530's 238 s**), else it starts alone;
  * the victory sting caspian_victory1 (Type h_f92fb2b9 too) replaces tutorial_02 if it still
    plays (tutorial_02's 3 s fade-out, the sting's 0.5 s fade-in): it doesn't layer;
  * how long the old voice counts as playing after a stop isn't traced (a stream's +0xa8, which an
    envelope stop leaves set): the demo takes it to the end of its fade-out.
* **The tutorial's** (73 sound-triggers, 4 dialog-triggers):
  * the gate: either panel's USE_OBJECT plays terminal_03 (h_144f3dbb, 1.04 s) at the panel; its
    FINISHED opens the gate's two leaves (anim0, anim1: 2.0 s at time-scale 2.5) and plays
    metal_gate_05 (h_12c4f1ee, 2.53 s) at each leaf; the second's FINISHED starts the gate music
    (tutorial_02, a 3 s fade-in) and its 238 s stop timer (timer530);
  * the opening: Bratton's four lines on the script's USER1s (its 103s, #222):
    baseisguarded (1.97 s) at 3.02 s, envhazards (4.59 s) at 6.18, goingtoguide (6.11 s) at 11.22,
    refresher (6.75 s) at 19.55, whose FINISHED starts timer429 (0.1 s: the co-op hint) and
    timer462 (5 s: BASIC CONTROLS; both #224's: [hints.md](../hints.md));
  * the hint chain's lines (dxud_ai_00047 "[BACK] to check mission objectives." ...) on its
    timers, stopped and disabled by router400's plates; Bratton's PLAY_DIALOG lines (the radar
    explanation, whose FINISHED opens the WEAPONS panel; first patrol, take cover ...) through the
    queue;
  * at the first update the start music tutorial_01 (stopped by timer525 at 168 s, a 3 s
    fade-out), the beds amb_estu_01 and light_bird_bed_1 (within 60 m of the start), and the
    lava's loops and bursts (3D, 30 to 40 m); the mine field's caspian_tut_mellow1 (72 s) and the
    base's caspian_victory1;
  * the save points' "chime" is the AI's voice line dxud_ai_00008 (h_f674fc4f), 2D.

Test hooks: `BF_TRIGGER_LOG=1` lists every sound- and dialog-trigger at the start (its sound and
length, how it plays) and logs what each does (plays, stops, queues, has played, says, the level
a Type 2-5 one heads to); `BF_SOUND_LOG=1` logs each of their voices as it starts and stops (its
volume now and in full, where, its distances, also under `BF_MUTE`); either logs the listener
going indoors or out, a music voice replacing another ("replaces the music voice N") and each
act on a handle the replace erased ("stop dropped"); `BF_MUTE=1` runs them without a voice; `BF_TRIGGER_FIRE` fires them.

```sh
# the tutorial's gate: the panel's sound, then the gate and its grind, then the gate music
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100 BF_TEST_USE=0.5 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the opening's four lines, from the opening script (#222)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the same lines fired by hand at the script's times, without it (#223's check, before #222)
BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 BF_SCRIPT=off BF_TRIGGER_FIRE="trigger_sound_bratton_baseisguarded,USER1,3.02;trigger_sound_bratton_envhazards,USER1,6.176;trigger_sound_bratton_goingtoguide,USER1,11.217;trigger_sound_bratton_refresher,USER1,19.551" cargo run --bin bf_play
# an outdoor and an indoor bed: m01_a's start into the lit hall (indoors at 6.8 s)
BF_MAP=m01_a BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_TEST_GOTO=79.6,-100.5,90.1,-60.7,132 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
```
