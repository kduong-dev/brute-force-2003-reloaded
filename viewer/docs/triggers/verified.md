*Part of [triggers/](README.md), the former page `triggers.md`.*

## Verified

`BF_MAP=tutorial` (and sdm_e34, e40, m09_a), `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps,
the log read and the frames looked at:

* **Level start** (no hooks; #220's check, before the opening played: since #222 the level's
  start plays it, and Tex comes at 17.09 s, [cutscenes.md](../cutscenes.md)): FIRST_UPDATE opens
  the door h_fa29ecfd that starts open (anim3, 1.21 s), spawns the gate wall's sentry
  TUTORIAL_terrorist_5 on the wall walkway at (-37.9, 27.25, -9.8), 3.07 m below trigger_spawn46, removes the four editor leftovers h_0247aefb round (-98, 18, 44)
  (destroyers 473, 498-500), starts timers 383 / 384 (the west lava pit's bursts every 4 / 6 s:
  trigger_spawn382 / 381 at (-23.0, 14.0, 9.3) / (-20.2, 14.0, 15.7)), 458, 525 and h_0279cdb2,
  runs the gamevars (since #221: the objectives' states 0, the member tests, campaign.md) and
  logs what steps 3-9 will run (the opening script, the music and ambience, the first radar
  blip). Tex stands on the ground under trigger_spawn8 at (65.55, 25.25, 74.03),
  take03's height, facing west; the camera behind him shows what the game's does after the
  opening (take03 at 32 s: the dark peak, the palms, the rock face right), and the HUD his
  template's MK-ASLT at 48 / 500, as take03's (frames 0-59, `BF_AMMO_LOG`: type 2 500).
* **The base** (`BF_TEST_GOTO=-36.4,37.3,-57.3,22.9`): plate h_eefcb023 sends ENTER_DETECTOR ->
  router103 -> terrorists 21, 13, 12, 9 on the floor at (-95.5, 15.0, 55.2), (-98.4, 15.0, 58.8),
  (-95.0, 15.0, 58.1), (-61.7, 15.6, 19.3) (their triggers 3.1 and 2.5 m up), and anim2 opens the door h_06537687 once (frames
  58-120: the door slides open as Tex walks up and in, the ammo and the DNA inside), leads §7.8.
* **The lava yard** (`BF_TEST_GOTO=48.1,-1.5,48.1,-21.5`): the 18 m and 25 m circles at once
  (the chime, the blips, the east pit's burst sounds enabled: steps 4 and 9); plate h_0b18c4c8
  (a human player's) -> router22 -> terrorists 16, 3, 1 on the ground at (33.0, 20.1, -69.8),
  (33.5, 20.1, -64.7), (28.7, 15.3, -77.1) (3.9-5.0 m below their triggers); plate h_f42fa5b8 ->
  router473 -> terrorists 30, 29 at (73.1, 14.9, -44.6), (69.9, 15.3, -40.8), leads §7.5. Seen from the test tools' free camera with
  `BF_TRIGGER_FIRE=trigger_router473,ENTER_DETECTOR,1`: the two stand in the gully by the stone
  head.
* **The second building** (`BF_TEST_GOTO=74.4,-104.1,64.4,-97.7`): plate h_020415dc ->
  anim15 opens h_0f544611 27 m away (once: then it disables itself, h_137926ff) and spawns
  terrorist 17 inside at (38.3, 15.5, -88.8), on the floor, leads §7.6.
* **The tutorial's gate panel** (`BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100
  BF_TEST_USE=0.5`): USE_OBJECT -> router349 -> the panel's sound-trigger h_0309a1b9, router252
  disables Bratton's "use the panel" line, router6 -> the panels' property triggers. Since #223
  (`BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_DOOR_LOG=1`, 15 fps): terminal_03 plays at 1.07 s at the
  panel (5.3 m away); its FINISHED at 2.07 s opens both leaves (x2.5, open at 4.00 s) and plays
  metal_gate_05 at each leaf; the second's FINISHED at 4.53 s starts the gate music (fading in
  over 3 s) and timer530 (238 s). The frames show the gate shut while the panel sound plays
  (frames 16-30) and rising from frame 31-33. take11 has the same chain: the panel sound's onset
  at 718.60 s, the grind's 1.05 s later as the gate starts to rise (between 719.75 and 720.0 at
  4 fps), the music from ~722.5 s rising over ~3 s (its audio envelope).
* **sdm_e34 (regression)**: walking through door h_10616962 along its axis
  (`BF_TEST_GOTO=-76.26,-2.04,-62.34,5.64`): plate h_f214c88d -> router h_e09f6796 -> anim
  h_f12ff6e9: ENTER_DETECTOR_FIRST opens it, EMPTIED_DETECTOR closes it 0.2 s after Tex is off the
  plate; the gate from its panel (`BF_TEST_GOTO=-23.23,79.05,-23.7,80.8 BF_TEST_GO_AT=100
  BF_TEST_USE=0.5`): USE_OBJECT -> router h_1c61561d -> both leaves' anim-triggers, once, 4.96 s.
  Its pickup spawn-triggers fire at the start and make their copies (#278, below).
* **Consoles (#262)**, `BF_DOOR_LOG=1 BF_TRIGGER_LOG=1 BF_TEST_NO_HOSTILES=1`, `BF_MUTE=1 BF_BACKGROUND=1`,
  15 fps; e34's opening letterbox taken out with `BF_TRIGGER_FIRE=trigger_letterbox204,12,0.2` (the
  start script `BF_TEST_GOTO` skips would have), the player put in front of a console with
  `BF_TEST_GOTO=<start>,<its button>` (`BF_TEST_GO_AT=999` standing) and X held by `BF_TEST_USE`:
  * e34's console h_09da26aa, take01's and take04's walk (`BF_TEST_GOTO=-33.3,23.7,-34.5,28.2,33.7
    BF_TEST_WALK=1 BF_TEST_USE=4`): "Hold [X] to activate console." and the ring from 1.87 s (the use
    point 2.4 m off); X from 4.0 s, used at 4.33 (0.3 s), USE_OBJECT [tex] to routers h_14eeda94,
    h_0e1c2378 and h_1bbd1197 and the property trigger h_fccb606d at 4.40, the prompt gone that
    frame (take04: 0.43 s after the press); the other console's property trigger fires too, so both are
    dead. The gate h_06676419: clip h_0a48ec76 4.40-9.33 (4.96 s, both its parts to 3.92 m), the
    timer h_e4c2a242 pulses at 9.40 and clip h_10d3fde3 plays 9.40-14.33; metal_door_5 at 4.40 and
    9.40 (out of hearing at 27 m), four door points valid, the voice line queued at 4.40 (the
    opening's own dialog holds the queue to 8.67; it plays at once when the queue is free).
  * the gate from the yard (`BF_TEST_GOTO=-53.56,30.81,-51.2,41.5,29.1 BF_TEST_GO_AT=999
    "BF_TRIGGER_FIRE=trigger_letterbox204,12,0.2;@h_09da26aa,14,4.4"`, take04's view after its
    teleport): the left leaf rises from the event, the middle one from 5.0 s on, the gate open at
    14.4 s; take04's frames (a frame-difference of the leaf regions: the left one moves from the
    first frames in the yard, the middle one from 10.4 to 10.65 s of video, 5.2 s after the
    activation: its first motion is slow) show the same two stages. The lighting differs (the
    take is darker: not this ticket's).
  * the gate's collision (`BF_TEST_GOTO=-52.5,31.0,-52.5,50.0,29.1 BF_TEST_GO_AT=6
    "BF_TRIGGER_FIRE=trigger_letterbox204,12,0.2;@h_09da26aa,14,1"`): the first leaf is past 0.6
    of its clip at 4.0 s, the gate stops blocking, and Tex walked through it from 6 s while the
    middle leaf was still down until 6.0 s: frames 105-135.
  * console h_00dd163c alone (take03: `BF_TEST_GOTO=-34.2,60.5,-36.5,60.4,35.3
    BF_TEST_USE="3,0.2;4.5,1.2"`): the 0.2 s tap does nothing (the prompt stays), the hold from 4.5
    uses it at 4.80; the same gate chain from 4.87. Three holds on h_09da26aa (take05:
    `BF_TEST_USE="4.5,0.9;8,1.5;12,1.5"`): used at 4.80, the other two find no target.
  * e23 (`BF_FLAG_LOG=1`): h_f9bcceaf (-66.2, 25.3, -98.1) opens gate h_14690535 (clip h_10d3fde3 at
    3.40, h_0a48ec76 at 8.40, the 5 s timer trigger_timer263) and its gamevar sets
    missions/h_fd6709be/h_12f6e3b9/state: the objectives menu (`BF_TEST_OBJECTIVES=6,2`) reads "Open
    the main gate. (completed)", +$1200 (2000 x 0.6); h_ebb2af83 opens h_1d6e35a3 (h_0a48ec76 at
    3.40, h_10d3fde3 at 8.40) and sets .../h_fbd70481/state ("Open the rear gate."), +$1200;
    h_e141eb63 opens h_fdd04c6b (its one clip, h_19dbcaa7). m08_a's h_f0bbfe39: its router's
    sound, then the m08_gate1 script's USER_SIGNAL1 1.07 s later opens h_e3b5bd91 (x2.5, both
    clips). m14: h_e447bf5f (m14_a) opens h_e3b56e9c at the use (the script's 103 on its
    anim-trigger) and sets missions/m14/h_f761dcc1/state, +$48000 (80000 x 0.6); m14_b's
    h_ed408fc9 and h_f649de73 open h_eab25e0a and h_f1bb0fb0 and set h_e56fbded and h_ec688d7b
    (+$48000 each); m14_d's h_1b2a7b46 opens h_eab25e0a at x0.5. e16: h_e846dbf5 opens h_f4d77cfd
    (one clip), h_00dd163c h_14690535 (both clips at once, and the panel h_066a5235 dead),
    h_1bd44786 h_06676419 and h_0f60548f (and the panel h_0f6d62a3 dead). e16's first gate h_e203f1ba
    (panels h_14643319 and h_1d63038f, `BF_TEST_GOTO=-186.26,-71.12,-184.0,-70.7,11.6`
    `BF_TEST_USE=3`): the panel's router turns the USE_OBJECT into TIMER_PULSE for the gate's anim-trigger
    h_fef9de5e (clip h_0a48ec76) and the three property triggers, which make both panels and the
    "panel" h_f5bee5a1 dead: one leaf rises (the other panel's router would play the other clip,
    h_10d3fde3), where the demo's single clock opened both before #262. The m14 consoles stay
    usable (no property trigger on their use): m14_a's prompt is back at 7.33 s, when the gate's
    script, which held the player, has ended.
  * regression: the tutorial's panel (`BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100
    BF_TEST_USE=1.5`): the prompt from the start, used at 2.00 s (0.5 s), both panels dead, the
    gate's two leaves from 3.07 s at x2.5 (2.0 s); sdm_e34's (`BF_TEST_GOTO=-23.23,79.05,-23.7,80.8
    BF_TEST_GO_AT=100 BF_TEST_USE=0.5`): used at 1.00 s, both leaves' clips from 1.07 s, 4.96 s.
* **Pickup copies and their respawn (#278)**, `BF_TRIGGER_LOG=1 BF_PICKUP_LOG=1 BF_WEAPON_LOG=1`:
  * sdm_e34 (`BF_CHARACTER=3 BF_TEST_GOTO=-81.78,63.86,-81.78,59.0 BF_TEST_GO_AT=999`): 54 copies at
    the first update, each OPERATED bound to its timer by its relay ("h_f106c777 (relay) binds
    h_7e00001b to h_0021e47f"); the STAMINA POWER's at (-84.07, 29.78, 60.58) with its idle effect,
    ahead left of Tex as in #35 take20 (frame 19 against the take at 1.0 s; the effect's look is
    #15's); the level's own power-up idle effects none (they're all templates);
  * the medkits (`BF_TEST_GOTO=16.5,99.1,12.1,99.1`): Tex takes the three copies at 7.73-7.93 s,
    each PICKUP_OBJECT starts its timer (trigger_timer267/271/275, 10 s), whose pulses at
    17.73-17.93 s spawn the next three and the spawn flash beside them (trigger_spawn269 ...); he
    takes those at once, and so on every 10 s;
  * a gun (`BF_TEST_GOTO=-53.3,78.6,-55.08,79.45 BF_TEST_GO_AT=999 BF_TEST_SWAP=2.0`): "Hold [X] to
    swap Saryl-45.", the swap (the LZR-23 released at 2.55 s, "gone in 20 s"), the grab's
    PICKUP_OBJECT starts trigger_timer355 (20 s); the dropped LZR-23 vanishes at 22.55 s, and at
    22.85 s the next Saryl-45 lies there with the spawn flash (h_117c1805) and the prompt back
    (5 fps capture, frames 113-118);
  * mp1 (`BF_CHARACTER=1 BF_TEST_GOTO=-42.68,-3.6,-43.13,-10.59,36.5 BF_TEST_GO_AT=999`, #129
    take38's player 2): 28 copies at the first update (its five pulse-only power-up spawns wait
    for their timers); Flint stands on the walkway's L-Shot-75 lying on its side under its green
    spawn glow with the 11mm Ammo beside it ("Hold [X] to swap L-Shot-75.", "11mm Ammo (full)"),
    and the two medkits lie 14 m on under theirs, as the take's bottom half has them.
  * The campaign: the tutorial has no pickup template (its L-Shot-50 swap, its medkits and DNA as
    before); e34's iKhan-GPL and 12 medkits are found as before, its six templates (the supply
    drops after a radio line, the finished campaign's air filter h_028920ea behind its flag test)
    wait for their triggers.
* **e40 / m09_a**: the squad starts on the ground under its spawn-triggers (Brutus at
  (-14.04, 26.20, 160.70), xemu.md's live read) with its placements' weapons, and `BF_SLIDE_LOG`
  shows no fall in its first 6 s (it fell ~10 m before, with fall damage); m09_a's squad at its
  placements (Brutus at (-177.2, 62.1, -225.3)), its two Shrikes from trigger_spawn2 / 749 on the
  ground 6.75 / 9.46 m below their triggers. With `BF_DEATHMATCH=1` (no squad in the world) and
  `BF_TRIGGER_FIRE=h_14640a2e,USER1,1` e40's Brutus spawn makes nothing (no NPC Brutus).
* **The marines' member check** (#221, `BF_FLAG_LOG=1 BF_TRIGGER_FIRE=trigger_router_confed_one_fx,USER1,1;
  ...two_fx...;...three_fx...;trigger_spawn64,USER1,2;trigger_spawn76,USER1,2.2`): each start
  marine's copy is a clone (a re-armable trigger's) whose set-up sets its type's member flag,
  then it's taken away at once (its save-game slot not alive) and its flag goes back to 0; their
  farcast flashes stay disabled (their gamevar tests were FALSE at the first update). spawn64
  and spawn76 (the V-squad, `h_0fa4f510` false, the templates themselves) spawn Stout and
  Cogburn on the ground and set their flags: tex 1, Gonzales 0, Cogburn 1, Stout 1, take11's
  `1|0|1|1`.
* **Destroyers**: m09_a's trigger_destroyer556 (`BF_TRIGGER_FIRE=h_039f8930,USER1,2`) removes
  trigger_spawn2's Shrike; no level's destroyer lists a squad member, so the guard against
  removing one isn't exercised by the data.
  * **What a destroyer takes** (#308; code-map FUN_00200b70). Of the objects the game's
    destroyers list (h_0b6b92ba), by the data: 1019 point lights in 25 levels (the mission
    levels' lamps, each behind a router that listens to its fixture's DESTROYED: m04_c's 140, m07_a's
    139, m07_b's 115, m02_c's 105 ...), 152 spawn-triggers' copies, 54 game objects, 32 effect
    objects (31 with a duration of 0, one with 0.8 s), 5 characters. A lamp is taken out of the
    world at once; the demo put none out before (it logged "removes the placed ..."), so a shot
    fixture's lamp stayed lit for good: now `LampsOut::put_out` (play_lamps.rs). m08_a:
    `BF_MAP=m08_a BF_NO_SQUAD=1 BF_TEST_GOTO=-33.9,124.0,-33.9,124.0,26.0
    BF_CAMERA_AIM_AT=-33.9,22.8,130.9 BF_TEST_HIT=3.0 BF_TEST_HIT_NEAR=-33.9,130.9
    BF_TRIGGER_LOG=1 BF_LINK_LOG=1`: the lamp post h_ec4a51ce (1 hp) breaks at 3.07 s,
    "trigger_destroyer166 removes the lamp h_f31fce33", the lamp goes out a frame on and the
    crate beside it, lit by it, falls from 38 to 20.5 mean red (once the fixture's own burst
    has gone); m04_c's h_ff9dba99 (40 hp, 3 debris) takes h_fcfdc60a the same way. An effect
    object is destroyed at the next tick (a duration under 0.01 sets its parameter to the end and
    ends it, FUN_0021c3e0): its particles and lights go on that frame (`TakenAway`), where the
    demo used to let them fade (e34's tutorial pointer's glow for 0.9 s). The one destroyer
    with a duration (m07_c's h_1c810669: 0.8 s from 1 to 0 on an effect) ramps the effect's
    user parameter first: not done, it's taken at once. A game object with a debris list is
    killed by the destroyer in the game, so it breaks and sends its DESTROYED; the demo hides
    it without debris (unchanged here).
* **Sound- and dialog-triggers (#223)**, `BF_MUTE=1` with `BF_TRIGGER_LOG=1 BF_SOUND_LOG=1`
  unless said:
  * the tutorial's opening lines, fired at the script's times (the `BF_TRIGGER_FIRE` above, 10
    fps): baseisguarded 3.10-5.00 s, envhazards 6.20-10.70, goingtoguide 11.30-17.40, refresher
    19.60-26.30, whose FINISHED starts timer429 and timer462, which sends BASIC CONTROLS its USER2
    at 31.30 (step 5). take03 matches: the lines' onsets (cross-correlated with their waves) 0.07
    to 0.16 s after the script's times, as its pictures lag (the camera cuts 0.1-0.15 s), and the
    co-op hint (FINISHED + 0.1 s) on screen by clk 26.46-26.56 s;
  * at the first update (frame 0) the start music, amb_estu_01 and the east lava loop play, the
    bird bed waits for a player within 60 m;
  * `BF_TRIGGER_FIRE` on the cues: h_0378b249's TIMER_PULSE stops tutorial_01 fading over 3 s;
    h_17ebe01b plays caspian_tut_mellow1 once (a second ENTER does nothing: its entry is once)
    and its TIMER_PULSE stops it over 3 s; h_04a4176a's sting plays and its USER1 stops it over
    0.5 s (since #254 those stops reach the voice only when it replaced no playing track);
  * the one music voice (#254, `BF_TRIGGER_LOG=1 BF_SOUND_LOG=1 BF_MUTE=1`):
    * the mine field before 168 s (`BF_TEST_GOTO=70,-48,92.4,-63.9 BF_TEST_GO_AT=30`): the circle
      at 32.55 s plays mellow, which "replaces the music voice 3" (tutorial_01, fading out over
      3 s) as it fades in; the mine plate's script at 34.32 s sends nothing to the music;
      timer527's TIMER_PULSE at 104.55 s: "stop dropped", mellow plays on;
    * after 168 s (the same with `BF_TEST_GO_AT=175`): timer525 stops tutorial_01 at 168.00 s,
      the circle at 177.51 s plays mellow alone, timer527 stops it at 249.51 s;
    * the gate (`BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100 BF_TEST_USE=180
      BF_TRIGGER_FIRE="h_17ebe01b,ENTER_DETECTOR,175;h_04a4176a,TIMER_PULSE,205"`): mellow at
      175 s alone, tutorial_02 at 184.07 s replaces it, the sting at 205.01 s replaces
      tutorial_02 (fading out over 3 s, the sting in over 0.5 s); with the panel at once and
      mellow fired at 50 s the replace reaches a voice whose own handle was erased (tutorial_02's)
      all the same;
    * m01_a (its music and the two beds into the hall), sdm_e40 and sdm_e34 (the bed and the
      music at the first update): no replace, the beds' levels as before;
    * muted runs only (the user may be at the PC): the crossfade is checked in the logs, not by
      ear;
  * the dialog queue: the radar explanation (ticket 0, 6.96 s) plays at once, first patrol
    (ticket 1, queued at 1.53 s) at 8.00 when it has ended; their FINISHEDs at 8.07 (to the
    WEAPONS panel's menu-trigger) and 10.60, the frame after each ends;
  * trigger_dialog_tex_yeahyeah: tex_1 (the demo's Tex) says dxmi_tex_m00003 at 1.00 s, FINISHED
    at 4.07; trigger_sound422 disabled while its burst plays: no FINISHED; h_e82b011b's ENTER
    stops its line and disables it;
  * h_15bc1769: walking north from (40.5, 0) the bird bed starts at 59.3 m from it; walking back
    south it stops at 60.2 m;
  * sdm_e34 (regression): the doors' sounds come from their sound-triggers (on at the
    anim-trigger's ANIM_START / ANIM_BACK_START, playing once Tex is within their 7 m
    `max-distance`: door h_196659f4's opening one at 0.07 s with its count, played at 1.00, its
    closing one at 1.47), the gate's metal_door_5 at the panel's use (within its 15 m); the
    doors and the gate move as before; its music caspian_action20 from its FIRST_UPDATE trigger;
    its Type 2 wind bed at 1.0 from the gate's start outdoors, 0.20 from the doors' start in the
    barracks (`BF_TEST_GOTO=-76.26,-2.04,-62.34,5.64`: indoors, as the frames show). e40 and
    m09_a: their music from their triggers (e40's h_0378b249, m09_a's trigger_sound_music41), no
    fault;
  * the beds (the fix round, each level muted for 30 frames): at the first update m01_a, m01_b,
    m05_b and m05_y play amb_ferix_06, m09_a and m09_x amb_shrike_01, sdm_e40 light_bird_bed_1,
    at 1.0 (the start is outdoors); m09_b starts indoors (a hive chamber), its amb_shrike_02 at
    0.15, with strange_insect_bed_1 and the tunnel music; m09_c starts indoors, amb_shrike_int_01
    at 1.0; m14_d plays shrike_distantspace_001a and shrike_tunnel_bed (its outriders' Type 5
    engines wait for a player within 15 m). m01_a walking from its start
    (`BF_TEST_GOTO=79.6,-100.5,90.1,-60.7,132`): mechanical_amb_int_1 (Type 3) starts at 0.05 when
    Tex is within its 20 m (6.33 s), and as he steps through the doorway into the lit hall
    (6.80 s, indoors) amb_ferix_06 heads down to 0.15 and the hum up to 1.0;
  * the rolloff curve: the tutorial's east lava loop (max 40) is silent from the start, 92 m off;
    a west burst at 33.5 m from its min 30 plays at 0.52, at 38.8 m at 0.04;
  * unmuted (`BF_MUSIC_VOLUME=0.2 BF_AUDIO_LOG=1`, the gate run, ~6 s; before the fix round):
    five voices playing (sinks) by 2 s, the three music-bank tracks decoded.
* **Step 9 (#228)**, the commands under "Test hooks", 15 fps, against the tutorial's takes:
  * radar-triggers: h_eaec0c5a gives h_fe303e53 (25.4, 34.6) contact 3 and a 2 s blink on
    FIRST_UPDATE; entering circle h_ed8c0a43 at 2.87 s, h_165b8f36 moves it to h_e89e2471 (56.8,
    -0.5) and h_ebaef50e clears the first, as trigger_sound452's chime (1.51 s) plays. On the
    radar the first dot is gone from ~1.3 s to 2 s; the new one, ahead from 2.87 s, holds ~1.1 s,
    is gone ~0.75 s, then stays (frames 0-90), as take07's does when its circle is entered
    ([play.md](../play/radar.md), "Radar");
  * the damage plate: enabled at 0.13 s, Tex hit 0.80 a frame (20 x 1/15 x 0.6) from 0.20 s,
    disabled by timer528's pulse at 1.20 s: 115 -> 102.2 (take15: 115 -> 102.7 over 1.07 s; the
    15 fps frames make it a frame more), no blood on him (the grunt of the first hit not
    listened to: the runs are muted);
  * the lava bursts: the east pit's three spawn-triggers at 1.53 / 2.47 / 3.00 s after their
    timers' 1 / 2 / 2.5 s, then every 3.53 / 6.07 / 5.07 s (their loops' 3.5, 6 and 5 s plus a
    frame for the re-arm); take08's near one (trigger_spawn356, (50.9, -26.8)) bursts every
    5.9 s of its watch log's clock (109.2, 115.4, 123.4 and 129.6 s of its video, with a 1.7 s
    stall between the second and the third). Each a bright fire column that rises over ~0.5 s
    and fades by ~1 s, as take08's (more orange there: see "Not done");
  * the level's mines and the DNA canister: [grenades.md](../grenades/sentry.md) "Sentry",
    [pickups.md](../pickups/dna-canisters.md) "DNA canisters".
* **The music bank's lookup (#398)**: a sound-trigger's music-bank track is found by its sound's file
  record (offset and size in the level's bank file), not by its wave's name; the levels' logs are in
  [maps/music.md](../maps/music.md), "Verified (#398)". The sound-triggers themselves are unchanged
  (`BF_MUTE=1 BF_SOUND_LOG=1`, main's build against this one): the tutorial's twelve voices in 150
  frames start at the same times at the same volumes (tutorial_01 at 0.40, amb_estu_01 and the bird
  bed at 0.50), m01_a's three likewise, sdm_e34 in deathmatch its wind bed and its silenced music;
  only the tracks' names and places in the log lines are new.
