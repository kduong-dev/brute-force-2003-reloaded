*Part of [maps/](README.md), the former page `maps.md`.*

* **Doors and gates.** Their leaves are the parts of a placed object that its clips move: parts
  on sliding joints (joint type h_fd8f670c: sdm_e34's and the tutorial's doors and gates) and,
  since #261, parts on transform joints (h_f35d5b3c) and hinges (h_e94eb66a): m09's and m14's
  Shrike doors, m01's hatches and m01_a's gate, m05's and e06's doors, m05_y's hut, the 8-part
  doors of m08, e10, sdm_e10 and mp8, e10's ramp (Motion, below). A door moves
  only when one of the level's anim-triggers that lists it plays its clip (#220, #125:
  [triggers.md](../triggers/README.md)): forward opens it, backward closes it (the opening played back), at
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
    end of its panel's sound (its sound-trigger's FINISHED, #223: [triggers.md](../triggers/sound-and-dialog-triggers.md),
    "Sound- and dialog-triggers").
  * gates with wall panels (below).
  * the campaign's other doors: the door survey (todo/door-survey/survey.md) traces every
    one's chain.
  * Motion: the level's animations-<level>.xmb has an archetype-set named after each door's
    archetype, with one target per leaf (by part name) naming a data channel. The game plays a
    channel only on a joint of its kind (FUN_002340f0), sets the joint's value from it
    (FUN_00234dc0) and places the part by the joint's type (FUN_00234f90; the rows are in
    [code-map.md](../code-map/levels-and-triggers.md), "Levels and triggers"). The demo does the same for three kinds:
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
    * Each clip of a door runs on a clock of its own (#262, play.rs `DoorState` and `Clock`): an
      anim entry plays one clip forward or backward at its time-scale, and the leaves that clip
      names (`Leaf::clip`) follow that clock alone, as the game plays each clip on its own. A gate
      has two clips, h_10d3fde3 and h_0a48ec76, each moving two of its four parts: its two leaves.
      Where the level starts them together (sdm_e34's and the tutorial's gates, m08's) they move
      as one; where it doesn't they open one after the other: e34's gate h_06676419 (the consoles'
      USE_OBJECT starts one clip and a 5 s timer, h_e4c2a242, the other: take04's yard view, from
      0.8 s after the activation, has the left leaf rising already and the middle one start 5.2 s
      after it), e23's h_14690535 and h_1d6e35a3 (the same 5 s timer), e34's h_0f60548f and e16's
      h_e203f1ba (their second clip on a TIMER_PULSE); m01_a's gate plays its two at x5.5 and x1.
      A door's collision is one flag (`Arena::set_door_open`; its triangles aren't told apart by
      leaf): open once some clip is past 0.6 of its length (`DoorState::open`), so a gate stops
      blocking as its first leaf is up, the other still down for a while (e34's, 5 s) or for good
      (e16's h_e203f1ba plays one clip for each of its two panels, and the first use kills both
      panels: one leaf opens). An entry whose clip names none of the door's leaves plays them all
      (`DoorState::play`, as the demo did before the clocks). Five entries in the data have no
      clip (an empty animation-archetype), and none of them takes that path in play: three have no
      object (m04_a's h_e9d96e23, m04_c's h_e830dbac, m14_b's h_f993d7ef); e34's anim-trigger
      h_e8ad85ec has one on gate h_14690535 (its second entry: signal 0, ENTER_DETECTOR, OPERATE,
      once, which would play both leaves) but its one feeder is the timer h_f36b65f5, whose
      TIMER_PULSE answers the first entry (clip h_0a48ec76) and never a signal 0; m08_a's
      h_e03b5c79, on gate h_f90aa000, has one on signal 14 (USE_OBJECT) with action 3 (DISABLE),
      which is the base's action (it turns the trigger off) and plays nothing. Every other entry
      names a clip of its object's set that has targets. `BF_DOOR_LOG` names the clip on a door's
      lines when it has several. Not done: a moving part's collision doesn't move with it: a
      door's leaves stop blocking past 0.6 of their clip, the transform and hinge doors' as the
      sliding ones'.
  * Sound: a door's anim-trigger sends ANIM_START (31) as it starts opening and ANIM_BACK_START
    (49) as it starts closing to sound-triggers beside it, which play them (#223,
    [triggers.md](../triggers/README.md)): on sdm_e34 h_1edc357e (0.93 s) opening, metal_gate_05_close
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
      [triggers.md](../triggers/verified.md), "Verified") log the same door lines, and their frames are the
      same but for the world streaming in (frames 1-6) and a few pixels round the walking
      character that also differ between two runs of the same build. The door lists (`BF_DOOR_LOG`,
      before and after, all 53 levels): every door listed before is listed after, in the same
      order; the tutorial, the squad deathmatch maps and mp1-mp7 are the same; the campaign gains
      the survey's 69 doors on transform joints and hinges and e16's prop, m07_c's dome and
      m08_b's panel; sdm_e10 gains e10's two 8-part doors and mp8 its eight (opened at its start:
      FIRST_UPDATE, x1 and x100). m07_c's dome moves by the log (2.13 m over 6.17 s), but no
      camera spot tried showed it.
