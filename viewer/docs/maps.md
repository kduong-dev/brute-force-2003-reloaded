# Playing on a map

`bf_play` now runs on Battle of Bulgar by default (`BF_MAP=<level>` picks another level,
`BF_MAP=flat` brings back the checkered test floor, `BF_START=<n>` picks a start point,
`BF_TEST_GOTO=x,z,x2,z2[,y]` starts the player at one point (on the floor below height y if
given, e.g. a roof) and runs them toward another from 1 s on (`BF_TEST_GO_AT=<s>` sets off at
that time instead: #35 take08's dash after standing 4 s, skills.md), or stands still when both
points are the same;
`BF_FIND_STEEP=1` lists slide surfaces steeper than 50° and ledges with a drop of more than 6 m,
then exits; `BF_SLIDE_LOG=1` prints slides, falls and landings; `BF_ALE_LOG=1` prints particle emission;
`BF_NO_IDLE_FX=1` leaves out the power-ups' effects; `BF_NO_PLACED_FX=1` the level's placed
effect objects, `BF_PLACED_FX_VARIANTS=1` starts the ones held out, `BF_LEVEL_DUMP=1` lists
them):

* **Spawning.** You start at the level's start point. Points 0–3 are one squad's spawn on the
  hill, 4–7 the other's, and 9 is in the base. A level whose spawn-triggers spawn the squad (the
  tutorial, e40) starts each member on the ground under its own spawn-trigger, and one that
  places the squad in the world (m09_a) at its placements, each with its template's weapons and
  ammo ([triggers.md](triggers.md), "Spawn-triggers", #102).
* **The level's triggers** run: detectors, routers, timers, calculators, spawn-triggers, the
  doors' anim-triggers, destroyers ([triggers.md](triggers.md), #220).
* **Collision.** It uses the game's own collision surfaces (`src/bf/collision.rs`,
  `src/arena.rs`). On sdm_e34 that's 43,348 triangles.
  * Where they come from: `objects-<level>.ipn` holds them in the Ipion physics engine's format.
    The terrain blocks, the 23 invisible blockers and every placed object's parts each name
    theirs. Door leaves have their own and stop blocking while open.
  * Material: every triangle carries one, a world-material id plus a slide flag.
  * Standing: characters stand on surfaces up to 65° (the game's ground limit) within a 0.5 m
    step; steeper surfaces are walls.
  * Not solid: grass, wires and other undrawn or cut-out parts have no collision surface, so
    they don't block. Nor does a spawn-trigger's template, which the game takes out of the world
    until it's spawned ([triggers.md](triggers.md), #220).
  * Fallback: a level without collision data uses the drawn triangles instead.
* **Doors and gates.** Their leaves are the parts of a placed object that its clips move: parts
  on sliding joints (joint type h_fd8f670c: sdm_e34's and the tutorial's doors and gates) and,
  since #261, parts on transform joints (h_f35d5b3c) and hinges (h_e94eb66a): m09's and m14's
  Shrike doors, m01's hatches and m01_a's gate, m05's and e06's doors, m05_y's hut, the 8-part
  doors of m08, e10, sdm_e10 and mp8, e10's ramp (Motion, below). A door moves
  only when one of the level's anim-triggers that lists it plays its clip (#220, #125:
  [triggers.md](triggers.md)): forward opens it, backward closes it (the opening played back), at
  the entry's time-scale; an open door stops blocking. There's no door flag: a door nothing
  signals stays shut. What signals them, from the data:
  * sdm_e34's eight doors: their plate (a box, `size` its half-extents: e.g. h_f214c88d, +-4 x 2
    x 2 round the doorway) admits any character; its ENTER_DETECTOR_FIRST, through a router,
    opens the door, its EMPTIED_DETECTOR closes it once the last one has been off it 0.2 s. The
    5 m circle the demo used before is gone, and an enemy on a campaign map no longer opens a
    door its plate doesn't admit (#125's e34 gate).
  * the tutorial's two building doors: their plates (squad types), once: h_eefcb023 opens the
    base's h_06537687, h_020415dc the second building's h_0f544611 27 m away; the door
    h_fa29ecfd opens by itself at the level's start (FIRST_UPDATE); the gate h_14690535 from the
    end of its panel's sound (its sound-trigger's FINISHED, #223: [triggers.md](triggers.md),
    "Sound- and dialog-triggers").
  * gates with wall panels (below).
  * the campaign's other doors: the door survey (todo/door-survey/survey.md) traces every
    one's chain.
  * Motion: the level's animations-<level>.xmb has an archetype-set named after each door's
    archetype, with one target per leaf (by part name) naming a data channel. The game plays a
    channel only on a joint of its kind (FUN_002340f0), sets the joint's value from it
    (FUN_00234dc0) and places the part by the joint's type (FUN_00234f90; the rows are in
    [code-map.md](code-map.md), "Levels and triggers"). The demo does the same for three kinds:
    * A sliding joint takes a float channel (data type h_e0b5650b): metres along the joint's
      axis per frame at 24 fps. sdm_e34's doors take 1.21 s (upper leaf 1.89 m, lower 1.70 m);
      the gate takes 4.96 s (3.92 m). A sliding leaf without a channel slides its own length,
      eased (none in the data).
    * A transform joint takes a transform channel (h_06901441, `DT_XFORM`; each one in the data
      is compressed, 18 bytes a frame: a translation and a packed rotation; sampled, or keyed
      with a time before each frame): the part sits at its joint's child-point plus the
      channel's translation, turned by the channel's rotation after the joint's ORIENTATION.
      Between frames the translation goes straight and the rotation by a normalised blend
      (FUN_00233b10). A Shrike door's eight segments (h_e509629f, 43 keys each) turn 60° and
      retract about 6 m into the ring over 3.29 s; its other kind (h_0fb9cdec) rises 6 m over
      3.97 s; m08's 8-part doors (h_01ea42aa) roll up, each slat rising to stack under the
      lintel, over 3.79 s; m05's and e06's palisade gates (h_f3e957e1) rise 5.86 m over 3.97 s;
      m05_y's hut doors (h_0d8aa8f7) swing out 80° on their hinges in 0.8 s (a translation and a
      turn together); m01_a's gate (h_f8f66fa8) slides its two leaves 2.5 m along its own z over
      4.96 s, under the craft-like structure it's part of; m01's hatches (h_fda9208e) turn their
      ring 11.4° in 1.47 s.
    * A hinge takes an angle channel (h_fed99c09, radians; between keys the short way round,
      0x292230), clamped to the joint's limits (`<h_fab3703e min max>`): the part turns about
      the joint's axis with its child-point on its parent-point. e10's ramp h_f3d6c977 (type
      h_1e50b5e0) lowers from the hull to the ground, 16° (0.279 rad) over 1.47 s.
    * Before its clip first plays, a leaf sits at its joint's rest, as the level draws it (a new
      joint's value is zero, FUN_00235c00); played back to the start, it's at the clip's first
      frame (a Shrike segment's is within 5 cm and 1° of the rest).
    * Which objects are doors: one with a sliding part, as before. One whose parts on transform
      joints or hinges have channels of their kind is a door when an anim-trigger of its level
      plays it once (AM_FORWARD or AM_BACKWARD, with AC_PLAY or AC_RESUME:
      `level_scene::played_once`). Those the level only loops (FORWARD_LOOP: the fans and other
      props of e01, e16, e23, m03_a and the tutorial, m08_b's dish), which the runtime would play
      once (triggers.md "Not done"), stay as placed, as they did. So does m09_b's AA gun
      (h_0849d1d8): its gun hangs from its turning mount, and the demo places a compound's parts
      one level deep. The other objects played once move as doors do and stop blocking past 0.6
      of their clip: m07_c's dome, e16's prop h_e1dc5844, m08_b's special-use panel (never
      played: #145).
    * Not done: a door's clips run on one clock, at the time-scale of the anim entry that played
      last. m01_a's gate has two anim-triggers off the same plate playing its leaves at x5.5 and
      x1; the demo opens both at x1 (the one that runs second), where the game plays each clip
      on its own. And a moving part's collision doesn't move with it: a door's leaves stop
      blocking past 0.6 of its clip, the transform and hinge doors' as the sliding ones'.
  * Sound: a door's anim-trigger sends ANIM_START (31) as it starts opening and ANIM_BACK_START
    (49) as it starts closing to sound-triggers beside it, which play them (#223,
    [triggers.md](triggers.md)): on sdm_e34 h_1edc357e (0.93 s) opening, metal_gate_05_close
    (h_f25b1597, 1.47 s) closing, each played while the player is within its `max-distance`: 7 m
    for three of the eight doors (h_196659f4, h_10616962, h_0b6838d8; full within the min distance,
    3 m, then quieter along the rolloff curve to nothing at 7), 15 m for the other five (full
    within 7 m), and
    the gate's metal_door_5 (h_e1b1f167, 8.5 s) within its 15 m (7 m full). These waves are streamed (resource-type
    1) from the language wave bank `ml-sounds/en/<level>-en.tgz` at the offsets in the level's
    sound bank. (Before #223 the demo played the sound-trigger nearest the door itself.)
  * The gate's two leaves have a clip each in its set (h_10d3fde3 and h_0a48ec76, 4.96 s, one
    target each); each leaf takes its channel from whichever clip names its part (a transform or
    hinge part: the first whose channel is of its joint's kind).
  * Verified (#261; `--test` for the free camera, `BF_NO_SQUAD=1 BF_NO_HUD=1 BF_DOOR_LOG=1`,
    each door's anim-trigger fired at 2 s with `BF_TRIGGER_FIRE=<trigger>,<signal>,2`, 15 fps):
    * `BF_DOOR_LOG` on m09_b, m09_c, m08_a, m01_a, m05_a, m05_y, e06 and e10 lists every door of
      the survey's tables for them, with "n on transform joints, m on hinges"; m09_b's AA gun
      and the looping props aren't doors.
    * m09_c's Shrike door h_1ced0204 (`BF_MAP=m09_c BF_TEST_GOTO=2,-271.6,2,-271.6
      BF_TEST_FREECAM=0.3,0,12,-271.6,-90,-3 BF_TRIGGER_FIRE=h_faf72c55,0,2`): open from 2.00 to
      5.27 s (its 3.29 s clip); frames 29-52 show the iris opening from its centre, its segments
      turning as they retract into the ring, a little further each frame, the tunnel behind
      clear from frame 62. Its other kind, h_e3b56e9c (trigger h_19088a3c, the camera at
      144.44,47.60,194.55,124,4.1), rises from 2.00 to 5.93 s. Closing: m09_c's h_f4e541da,
      opened at x10 by the level's start, closes backward at x2 on its USER_SIGNAL1
      (`BF_TRIGGER_FIRE=h_17ed0d14,11,3`, the camera at -62.90,11.97,-259.19,134.8,-5.7) from
      3.07 to 4.67 s, its segments closing in from the ring (frames 52-70), back on its clip's
      first frame.
    * m08_a's 8-part door h_027fe272 (trigger h_f2bc037b, x2; the camera at
      -62.33,26.71,11.24,75.8,-2.6): rolls up from 2.00 to 3.87 s, its 3.79 s clip at x2.
    * e10's ramp h_f3d6c977 (trigger h_120ddb37; the camera at 76,24,18,80,-10): lowers from
      the hull to the ground between 2.00 and 3.40 s.
    * m01_a's hatch h_06b98a8e (h_f62e3e2c; a round mosaic door in a circular hut's wall, seen at 80.3,108,-16.6,0,-3, the #261 tester's camera) slides about 2.9 m along the wall in 1.47 s;
      its gate h_fa445160 (h_1267e76e and h_15c8b4ad, signal 50; from under it at
      59.5,133,-49.5,105,-12) slides its leaves apart; m05_y's hut h_f0ce0274 (h_ff133988)
      swings its doors open in 0.8 s; m05_a's gate h_fcb796db (h_0f77a93c) rises 5.86 m.
    * Regression, run with the build before #261 and after: sdm_e34's door walk-through and gate
      panel and the tutorial's gate panel and base door (the commands in
      [triggers.md](triggers.md), "Verified") log the same door lines, and their frames are the
      same but for the world streaming in (frames 1-6) and a few pixels round the walking
      character that also differ between two runs of the same build. The door lists (`BF_DOOR_LOG`,
      before and after, all 53 levels): every door listed before is listed after, in the same
      order; the tutorial, the squad deathmatch maps and mp1-mp7 are the same; the campaign gains
      the survey's 69 doors on transform joints and hinges and e16's prop, m07_c's dome and
      m08_b's panel; sdm_e10 gains e10's two 8-part doors and mp8 its eight (opened at its start:
      FIRST_UPDATE, x1 and x100). m07_c's dome moves by the log (2.13 m over 6.17 s), but no
      camera spot tried showed it.
* **Gates and their wall panels.** A gate with wall panels opens from them:
  * The panels are `world-button-object`s (h_16f22d4b, type h_ee4c83c9, `reticule-action` 1).
    Used, a panel sends its signal (h_eaf8a35b: 14, USE_OBJECT) and data (h_f1660a0d) to the
    triggers listening to it (0x163060; the trigger runtime, [triggers.md](triggers.md)).
  * A `router-trigger` lists the panels among its objects (h_0b6b92ba) and passes their signal
    on (h_e91cf6a8) to the gate's `anim-trigger`s. Those animate the gate object (their own
    h_0b6b92ba) with one clip per leaf on signal 14, once only (h_f136d22d 1, where the
    proximity doors' signal 50 / 42 entries say 2147483647), and send on to the gate's sound
    trigger. So a used gate opens and stays open. On sdm_e34: panels h_14643319 / h_1d63038f ->
    router h_1c61561d -> anim-triggers h_f993d7ef / h_e8ad85ec -> gate h_14690535. The
    tutorial's panels go through their sound-triggers instead: the panel's terminal_03 plays
    (1.04 s), and its end (FINISHED) opens the gate's two leaves (2.0 s at time-scale 2.5) and
    plays the gate's grind (#223, take11).
  * Using one (capture todo/gate button.mp4): from in front of its button (hardpoint
    h_10f81d34, facing the model's +z), within 2.5 m and looking within ~45° of it, hold E for
    0.5 s. Reach, angle and hold time are guesses; the capture shows no progress bar and no
    gate message. While it's usable, "Hold [X] to activate panel." shows under the health bar:
    the game's use prompt, "Hold " + the X button's glyph + "to activate %s." with "panel", a
    glyph line drawn as the hint lines are, at the first one's place ([hints.md](hints.md), "The
    use prompt"; take11 matches it to a unit). E is the demo's X. The blue target ring (the four-notch reticle texture
    h_1d2a68d0, tinted) sits on the button. The prompt goes once the panel is used, or once the
    doors its signal reaches unchanged (`Level::button_opens`: sdm_e34's gate) are open (the
    demo's choice: the game's panel stays usable until a property trigger says otherwise).
  * The button glows green: material h_f87dca06 is shader type h_1df292a1 (851 materials), a
    lit shader whose glow texture h_e01baa40 is tinted by its constant of the same name
    (0.276 0.914 0.220 here) and added as light.
  * `BF_DOOR_LOG=1` lists the doors and the panels (what each opens unchanged; a door on
    transform joints or hinges with how many leaves are on each), and logs each door the
    triggers open or close and its leaves each second: how far each sliding leaf has slid, or,
    for a door on transform joints or hinges, how far each leaf is from its closed place (m) and
    how far it has turned (degrees). `BF_TEST_USE=<s>` holds use from that time for a second.
* **HUD text** (`play_text.rs`) is drawn in the game's own body font, atlas h_e4e4d2f4 (as the
  menu draws it), its glyphs 1 atlas pixel apart over a black outline and shadow, as in the
  captures. Colours sampled from them: messages and prompts pale blue (186 210 249), ammo and
  NEW orange (254 165 104), an item that can't be used red. Capitals are ~11.5 units (14 for the ammo).

* **Movement constants from default.xbe** (see `decompiled/xbe/ghidra/README.md`, "Player
  movement"):
  * Gravity is 18 m/s² and the jump takes off at 5.8 m/s. In the air, horizontal speed keeps
    0.99 of itself per game frame (taken as 30 Hz).
  * Walking off a drop of more than 0.5 m is a fall.
  * Landing after a drop of more than 5 m costs 100 × ((drop − 5) / 25)² health, with no upper
    limit (FUN_0012b740; #157): 16 at 15 m, 100 at 30 m, 144 at 35 m, 164 at 37 m (m09_a's
    waste pit), before the difficulty step. So a long fall kills Brutus (105) and Tex (115) too;
    the demo stopped at 100 from 30 m before. A 10.6 m fall cost 4.9.
    * `BF_TEST_FALL=<m>` starts the controlled character that many metres above the floor it
      would stand on, falling from rest. Checked with `BF_MAP=sdm_e34
      BF_TEST_GOTO=5.47,35.95,5.47,35.95 BF_TEST_FALL=37 BF_SLIDE_LOG=1 BF_COMBAT_LOG=1`: "tex
      lands ... after a 37.0 m drop", "fall 163.84 x none (n 0) ... = 163.84" at 2.07 s, and
      Tex dies (frames 0-30 show the fall onto the start's slope). With
      `BF_DIFFICULTY=standard` and 35 m: "fall 144.00 x STANDARD (n -4) squad 0.60 ... = 86.40",
      28.6 HP left.
* **Sliding** works as the game does it, and only on collision triangles flagged as slide
  surfaces (the canyon walls).
  * Building up: a slide amount (0–1) grows while you move. On ground steeper than 50° it grows
    at up to 8 per second, at the full rate from 55°; flagged ground always counts at least half
    rate. At 1 the slide starts.
  * Sliding: it carries on until the ground eases under 45°.
  * Speed: the slide velocity starts at (1 − normal y) × 8 downhill plus 5 m/s down, then
    builds by that push 3 times a second, up to 40 m/s. You move by it times the amount.
  * Ending: under an amount of 0.2 your own movement blends back in. A slide that's blocked (a
    wall, or flat flagged ground where it only presses down) ends.
  * What you see and hear: the slide clip (motion "slide", script Sc_w1_slide_idle), facing
    down the slope, the surface's slide sound, and the slide dust (effect type h_01513b0f, ALE
    effect slide_puff) at your feet.
  * Steep but not flagged: unflagged slopes up to 65° can be walked up.
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
  trigger's fades ([triggers.md](triggers.md), "Sound- and dialog-triggers"). **One music
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
* **Shots.** Each weapon's bullet definition names its own effects, and they're played from
  the effect library: `effect-name` is the shot in flight, h_08d28037 is where it lands.
  | Weapon | Speed | In flight | Hit |
  |---|---|---|---|
  | LZR-23 laser | 150 m/s | laser_bolt_s + laser_bolt_rifle_s, a glowing bolt | laserhit_s, laserhitb_* |
  | Laser pistol | 100 m/s | laser_bolt_s | laser hits |
  | Brutus's cutter | 120 m/s | cuttertrail_ribbon, a bluish ribbon trailing the shot, and cutter_flare | — |
  | Minigun, ballistic guns (bullet-type 4) | instant | `tracer`, one particle shot from the muzzle at 113.6 m/s along the shot, on every (h_eeb9e75a + 1)th shot (MK every 3rd, Minigun every 4th) | sminigun_hit |
  | L-Shot | instant | tracer_snipe, a white beam drawn 200 m down the line in 0.38 s, smoke, rings | sminigun_hit |

  * Bolts carry their effect with them: the emitters flagged "attached" (h_e2999ffd, the
    demo's reading) and, since #57, every emitter of a bolt's flight effect but its trails
    have their particles move with the shot and go with it; a trail (an emitter under the
    library's world node h_ee223b51 with its appearance at the root) leaves its particles in
    the world. See play.md, **Bolts and their hits**.
  * Hit effects play on world hits with the struck material's debris, hit sound and the
    effect's light, each turned by its alignment-type; on bodies the flesh row's effect plays
    (made by hand for the squad's ballistic, cutter and laser rows, from the data for the rest)
    with the blood (play_fx).
  * World hits also leave the bullet's h_06a27365 hole or scorch on the collision triangle they
    meet, and each gun has its own muzzle effect and casings; the fire timing, pellets and
    accuracy spread are in [play.md](play.md), "Shots".
  * A shot's damage (friendly fire) lands when it reaches the body: an instant ray's at the
    distance / the demo's 400 m/s, a bolt's on the sweep that finds it, k x 0.1 s later
    (play.md, **Bolts and their hits**; its
    factors: the hit part struck, x0.7 on the own side, the difficulty step, none on the
    multiplayer maps as read live in a match, STANDARD on the missions: play.md **Damage**).
  * Beam appearances (class h_1f55f13e) are ribbons through their emitter's live particles in
    birth order, facing the camera. They have their own colour, alpha and width parameters, and
    their texture runs along v.
  * A frame's new particles are spread over the frame: along the emitter's animated offset
    (the sniper beam's sweep) and the effect's own movement (trails behind fast shots).
  * Effect libraries refer to their nodes by the case-kept name hash (e.g.
    "laserhit_Cone.emt"). Matching that recovered the emitters of 262 effect parts, the laser
    hits among them.
  * Test hooks:
    * `BF_TEST_FIRE=1` (or `2` for the second weapon) makes a `BF_TEST_GOTO` player aim and
      fire.
    * `BF_TEST_TARGET=1` stands the first squadmate still, 8 m in the line of fire.
    * `BF_CAPTURE_FPS=<n>` sets the capture's fixed step (default 15).
    * `bf_level` with `BF_FX_TEST=<effect name>` (and `BF_FX_MOVE=<m/s>`) runs one effect in
      front of the view.

* **Look.** The game lights each surface `texture × (ambient + key·N·L + fill·N·L)` with the
  level's own light colours, in gamma space, with no tone mapping. The demo does the same
  (`level_scene::spawn_lighting` and `console_look`):
  * Lights: the level's key (with shadows) and fill directional lights and its object ambient,
    given as sRGB colours.
  * Camera: tone mapping off, and an exposure where a light of 1.0 leaves a texture as it is.
  * Terrain: the level has a separate, dimmer key light for terrain (a bluish 0.44, 0.57, 0.65 on
    sdm_e34), so the terrain is tinted by its ratio to the objects' key light.

  Before this, an invented 9000 lux sun, a strong ambient and Bevy's default tone mapper (which
  desaturates) made everything pale. Point lights (muzzle flash, hit glows, explosions) are
  scaled to the new exposure by `POINT_LIGHT_SCALE`.
* **Power-ups.** Each power-up type's idle event (`<events><event state="1" h_ed0c9fac=…>`)
  names an effect type. Its objecttypes `<effect>` entry lists the ALE effects:
  | Power-up | Effect | What it draws |
  |---|---|---|
  | Brute (quad) | multi_quad_icon | godray.tga rays |
  | Force (shield) | multi_shield_icon | brokenring.tga ring, plus the animated texture "arcb" (4 × 4 frames at 30 fps) |
  | Health | multi_health_icon | red-flare.tga |
  | Stamina | multi_stamina_icon | blue-flare.tga |

  `src/ale_fx.rs` runs them from the effect library's own parameters (see that file).
  * These are "perp" particles: flat in the emitter's frame. The shield's arcs emitter is turned
    −87.6° about x to stand its arcs up.
  * Each particle keeps the appearance's turn at the moment it's born (about 220°/s), so the
    particles fan out round the icon.
* **ALE blends (#228).** An appearance's blend pair (h_1daea8c0) is the PC's D3DBLEND (src,
  dest), which the game turns into the console's through the table at 0x4036b4 (0x2a6f00): 5,2
  (SRCALPHA, ONE) on 518 appearances, 5,6 (SRCALPHA, INVSRCALPHA) on 207, 3,2 (SRCCOLOR, ONE) on
  33 (the lava's eruptions slava-erupt*, its wading rings and splashes env_sl_*, the teleport and
  ion beams), 2,2 (ONE, ONE) on 13 (slava-fuzz, the laser and flesh hits, campfire sparks), and
  a few 3,6 / 6,6 / 2,6 / 4,6. Every pair with a dest of ONE adds the particle to the picture
  (before #228 only 5,2 did: the others were alpha blends, and the tutorial's lava bursts came
  out as brown balls). The particle's colour is the texel x its vertex colour and its alpha the
  texel's x its vertex alpha, side by side (pixel shader 0x3b89e0, which FUN_0029f5d0 sets for
  any pair whose dest isn't INVSRCALPHA; the vertex colour is the appearance's colour with its
  alpha curve in the top byte, FUN_002a1a50, and the particle vertex shader 0x3b8bf8 passes the
  colour through): so the alpha curve fades a SRCALPHA particle only. With ONE the particle adds
  texel x colour whatever its alpha, with SRCCOLOR (texel x colour) squared: drawn here with the
  alpha at 1, the texture's alpha opaque and, for SRCCOLOR, the texels and the colour squared
  (`ale_fx.rs` `Source`). A beam's ribbon keeps its colour x alpha whatever its pair (its own
  draw isn't read).
* **Effects the triggers make** (the tutorial's lava bursts, its farcast flashes) are compiled
  with the game's particle rules, the grenades' and the Therm Sweeper's (`load_recorded_facing`:
  the shapes, the count once a game frame, the facing; [triggers.md](triggers.md),
  "Spawn-triggers"). take08's lava burst (slava-erupt-jet: small-flare at 3,2, 100 a second for
  0.6 s, 0.47 -> 1.7 m) is a bright fire column that rises and fades within a second; with the
  demo's own rules (half-size quads, its count) it was a string of small blobs.
* **Placed effect objects (#258)** (`ale_fx.rs` `spawn_placed_effects`, `play_placed_fx.rs`;
  `bf/level.rs` `EffectObject`). The air particles a level places: the tutorial's 18 swarms of
  flies (xfly), 7 of butterflies (xbfly) and the lava yard's 3 clouds of embers (slava-fuzz),
  and other levels' fires, torches, sconces, steam, dust storms and power-up spot glows (1453
  placements over the levels; m01_a has 65).
  * **What the game does.** An `<effect-object>` (object-type 17) is an object of an effect type
    (objecttypes `<effect>`). The world ticks it from the level's start (0x21b7a0), and it made
    its ALE effects as it was set up from its type (0x21a180, FUN_00219fc0): it runs its effect
    type's ALE effects and its light effect where it's placed. A spawn-trigger's template isn't
    in the world (FUN_000dfe90): only its copies run (the tutorial's lava jet and farcast pads,
    "Effects the triggers make"). Nothing else starts these: no trigger names the tutorial's
    flies, and their `lifetime` is FLT_MAX.
  * **The object.** Its life is the placement's `lifetime`, else its type's `max-life`: a 0 is
    taken away at the first tick (e01's exp_gasmain, never seen); every other placed life is 90
    days or more. Its type's h_0c88c46c is the user parameter it starts with (FUN_0021c380, 1
    on nearly every type), its type's bounding sphere its distance LOD. Each emitter runs once,
    for its lifespan (0x2a5b10): the ambient ones for 1e10 s, the few explosions levels place go
    off once at the start. An alchemy entry with a creation-time starts that much later (m14_c's
    meteor strike, 2 s).
  * **Camera-attached types** (h_1e40fd68, the type's flag: the placement's own isn't read,
    0x21c000) are kept at the camera each frame, h_0e528ccb m along its view (FUN_0021a700). The
    levels place eight, all at 0 m: six start with their level (e01's dust storm, m03_a/b/c's,
    m14_a/b's meteor rain), and m09_b/c's two are spawn-trigger templates the demo doesn't
    spawn yet. Their particles stay where they're born.
  * **Taken away** by a destroyer-trigger that names one (play_triggers.rs takes them in with
    its copies: e34's six tutorial pointers, e40's io tablet glow ...), or with the breakable
    object it's attached to (`<parent Type="1">`, FUN_0021a700: e01's barrel fires go when
    their barrels break). A hardpoint link (Type 2: the torches) doesn't take it away.
  * **Drawn** with the game's particle rules (the triggers' effects' shapes, count once a game
    frame, facing and LOD; the shapes are the table at 0x402a4c, where a shape-2 quad spans -1 to
    +1 of its Scale, twice the demo's own quad, and a shape-3 one stands on its particle) and
    more read from the game's code for these (`Compiled::placed`):
    * every particle starts in its emitter's box, which reaches its Width / Height / Depth to
      either side (both cube classes' spawns draw [-1, 1) of each: 0x2b17c0, 0x64e30; a game
      cube's OffsetHeight h_e2999ffd from the emitter up), and none is attached; FxCubeEmitter's
      polar spread is MinSpread + r x (MaxSpread - MinSpread) with r in [-1, 1). So xfly's flies
      start anywhere in 39 x 18 x 39 m, centred 12 m above their object;
    * a sphere emitter's particles start MinRadius + r x (MaxRadius - MinRadius) out along their
      direction (0x2a8a90) and a cone's the same out along their own way, its polar angle
      MinSpread + r x (MaxSpread - MinSpread) (0x2aa190), each r in [-1, 1). So a smoke
      column's puffs (0.04-0.21 m, spread up to 30 degrees) start within 21 cm of their emitter
      along their rise, where the demo's own reading (the other effects') puts them out flat
      from the cone's axis;
    * every facing-3 appearance is drawn stretched along its motion and turned about it to face
      the camera (`Pair::axial`): the draw picks the shader's facing by GeneralApp_FacingType
      alone (0x2a1de6), where the other effects do so only for their AngleFadeOut ones. So
      e01's barrel and trash fires, its campfires and m01_a's braziers are flame tongues rising
      from their coals (spun at random about their foot they were a flat star of flame), and
      the godrays (e06, m01_a/b: 4 m quads up to 70 times as tall, falling at 0.06 m/s from 171
      m up) hang as pale shafts;
    * turbulence fields run (FxTurbulenceField, 0x2a9440 / 0x2a9590): once a game frame each
      particle's velocity is pulled by dt x Approach x 4 of the way toward a new random
      velocity, a random direction x r x Magnitude with r in [-1, 1). xfly's 96.8 and 0.73
      keep the flies darting about at ~13 m/s (the root mean square in the demo), turning
      within a few tenths of a second; the butterflies' 24-35 and 0.20 at ~1-2 m/s.

    The other effects (the grenades', the guns', the triggers') keep their own checked rules:
    the box and the spawns for all of them are #164, the fields #101.
  * **Held out:** the five placed effects whose curves vary with the user parameter
    (s-lavalight, s-lavapool: the lava pools' glows; slava-cavern-splash, duststorm_med,
    env_fl_mdsplash). The game draws them at the parameter their objects start with, 1:
    FUN_0021c380 sets it before FUN_00219fc0 makes the slots, FUN_0008f5d0 hands it to each ALE
    effect (the effect's +0x18, 0x2abaa0), which sets it on each node (0x66a00, the +0x88 its
    curves are read at). There s-lavalight is 14 a second 4.9 m sheets of sidedistortion2 up
    to 21 m tall over a 23 x 22 m box, added at an alpha of 0.108: a faint sheet. The demo draws
    it far too strongly, an orange curtain over the whole view of the tutorial's lava yard that
    take08 doesn't show (100-134 s: the far hills stay clear behind it); the likely cause is the
    faint added sheet taken as linear light (`Source`, #165's blend). They stay out until that's
    drawn as the console does (`BF_PLACED_FX_VARIANTS=1` runs them anyway).
  * **Budget:** the placed effects' particles have their own cap (4000; m01_a keeps ~2000
    alive, the tutorial ~300, e40 ~600, sdm_e34 ~430), so they never crowd out a fight's
    (3000). Both are counted afresh each frame from the particles there: a count kept from frame
    to frame missed a map's end, which takes them all at once, so m01_a loaded a second time
    drew half its torches and a third time none. Frame time (bf_play, 40 s, mean): the tutorial
    5.5 ms against 5.2 without them, m01_a 5.6 against 5.1, e40 5.3 against 5.1.
  * **Not done:** their sounds (the fires' and torches' loops) and damage areas (e34's exp-oxy),
    a finite life's end and the start over of a type flagged h_0d5e990b (FUN_0021c500: none
    is placed with a life under 90 days), each alchemy entry's own max-life (the slot's +8,
    which FUN_0021a850 reads: FLT_MAX on nearly every entry, 10 s on h_f90c728a's in m01_a/b
    and m05_b/y, 207 and 600 s on a few in e01, m01_a and sdm_e01; the demo runs them on), the
    lights levels hang on them (`<parent>` light objects of type h_09020c30: #143's).
  * **Checked** (scratchpad t258dev):
    * The tutorial's landing spot (take03 30-33 s: `BF_MAP=tutorial
      BF_TEST_GOTO=65.55,74.03,55.6,73.05 BF_TEST_GO_AT=999`): dark winged flies dart about the
      sky over the hill and the palms, one to a few in view at once, as in take03.
    * The lava yard (take08 45 s: `BF_TEST_GOTO=50.68,17.53,55.8,8.9 BF_TEST_GO_AT=999
      BF_CAMERA_PITCH=0.25`): the sky above the pits full of drifting light specks, the embers,
      with flies and butterflies; take08's are as many by a dot count through the takes'
      encoding (the demo's frames scaled to 480 lines, x264 crf 16: ~80 specks of contrast 25
      or more in the same region, take08 ~80), but the demo's are warmer and crisper.
    * The pool at 120 s (`BF_TEST_GOTO=46.26,-25.92,51.4,-34.5 BF_CAMERA_PITCH=0.12`): no
      curtain, as take08 (the orange light on the hill there is #143's).
    * e34: `BF_TRIGGER_FIRE=h_f211fe52,6,4.0` removes the tutorial pointer by (69.2, -92.5)
      ("removes the effect h_f3bee12f"). e01: `BF_TEST_HIT=5.0 BF_TEST_HIT_NEAR=-110.96,51.77`
      breaks the fire barrel, and its fire goes with it.
    * The tutorial's opening: the lava bursts and the farcast come once each, as before (the
      templates are left to the triggers); e40, sdm_e34 and the test map play as before.
    * m01_a loaded three times in one test session (`--test` with
      `BF_TEST_LEVEL=1,m01_a;30,m01_a;30,m01_a BF_ALE_LOG=1`): each load starts from 0
      particles and keeps ~1950 alive, the same each time; so does a map played after
      `BF_TEST_BACK` has gone back to the menu.
    * e01's fire barrel (`BF_MAP=e01 BF_TEST_GOTO=-110.96,64,-114,50`, frames 100-140): flame
      tongues stand up out of the coals and over the rim. The bright flat disc over the barrel's
      mouth is its light (light_barrelfire: several point lights 0.3 m up inside the barrel,
      4.7-7.3 m reach), see below.
    * m01_a (`BF_TEST_GOTO=30.6,29,30.6,32`, frame 89): the godrays hang as pale shafts in the
      mist beyond the trees.
  * **Doesn't match yet:**
    * The flies look about 1.65 times as big as take03's and take08's and cross the picture
      about twice as fast; their speed for their size on screen matches within 25%, so the
      turbulence (read from the code: ~12.6 m/s root mean square) is likely right and their size
      is off. 1.27 of it is the demo's narrower view (#126); the rest wants a shot of a fly at a
      known distance in xemu. Their size is xfly-basic's Scale x 2 (shape 2, 0x402a4c).
    * The embers are warmer and crisper (the demo's additive blend in linear light at 720
      lines, #165), and they come and go in waves of 5-10 s (1 to 42 specks within 6 s).
      Nothing in slava-fuzz is periodic: its rate (4.68 a second), life (7.97 s), speed and
      transform are constants, and each particle fades in over its first quarter and out over
      its last fifth. It keeps ~37 alive, each a 14-35 m sheet of noise2's dots; the waves are
      the few sheets in front of the camera coming and going. Not checked against a take's
      steadiness.
    * e01's fire barrels' light (light_barrelfire, ~7 point lights at once 0.3 m up inside the
      barrel, 4.7-7.3 m reach) burns the coals at the barrel's mouth out to a flat bright
      yellow disc: the demo's point lights (inverse-square, `LIGHT_LUMENS`) against the game's
      light falloff, not traced. Every effect's light is drawn so; not this ticket's.
* **Pickups.** They're stored at their spawn height in the level file; they now settle onto the
  floor below.
* **Compound joints.** A compound object's joints carry a rotation (a w-x-y-z quaternion),
  applied to the part (the wall sets turn pieces by 90° and 180°). A part sits at the joint's
  parent-point + child-point, both in the parent's frame; the rotation turns only the part about
  its own origin. Bulgar's compound `h_0baf091a` (hangar, bunk rooms: 19 pieces) is the one placed
  object with non-zero child-points, and only this reading puts every piece inside the building:
  read as parent-point − rotation × child-point, its roof bays landed outside the hangar and its
  turned bunks went through the walls (issue #12). `BF_OVERLAP_LOG=1` (bf_level) lists large
  objects whose boxes overlap much.
* **Winding.** Object triangles are wound to face along their stored normals, so a mirrored mesh
  isn't culled from the front.
* **The gate (h_0792d16a) and texture address modes.** Its two leaves (h_e783bd32 → mesh
  h_e01d9aa6, h_fcd25d86 → h_178bc697) are the same panel shifted 0.996 m, with matching UVs
  and no joint rotation. The mirroring is in the texture's sampling: each leaf runs its
  texture u 0..2 across, and the texture entry's `address-u` / `address-v` (h_1c82a17b /
  h_078bf0c1) is `TAM_MIRROR` (h_18712765), so the second half is the first mirrored and each
  leaf shows one symmetric brace. The other modes are `TAM_WRAP` (h_e3012d9e, most textures)
  and `TAM_CLAMP` (h_e041e0b9). Every level and model texture is now sampled by its own modes
  (`Game::texture_address`); 24 levels use mirroring somewhere.
* **Invisible or glowing materials.**
  * Power-ups (Brute / Force / Health / Stamina) use a placeholder cube whose material no level
    defines; it isn't drawn.
  * Untextured glow materials (type h_f539fe8c, e.g. the medkits' shell) are drawn as their
    glow colour at the wrapper material's opacity.
* **Hits.** Shots, grenades, decals, the memory chip and the follow camera use the same collision.
