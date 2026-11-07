*Part of [play/](README.md), the former page `play.md`.*

### Footsteps: the gait's volume, the controlled character's 2D step and the pitch (#324)

A foot event (`step_player`: a toe coming down, as above) asks `play_liquids.rs` `step`, which makes
FUN_00119020's sound request (the code map's row has every address; todo/36-squad-voice leads §5):

* **Which sound.** In a liquid its wading step or none (#5, levels.md). Out of one, one of the
  surface's four variants for the character's footstep type (FUN_0012af30: the low two bits of a
  draw of its random state).
* **How loud.** The request's own volume, which replaces the library entry's (0.1 for every
  footstep): the skill's noise factor (1.0; a stealthed Hawk's quarter, skills.md) x the gait's
  factor x 0.75 for the controlled character (FUN_00112790).

  | The gait (the game's motion state, char+0x1f8) | Factor | The controlled character's step | Another's |
  |---|---|---|---|
  | the run, the run back, an aimed run (MT_RUN, MT_BACKPEDAL, MT_MOVE_*: FUN_00126a40) and the sprint (MT_DASH) | 1.0 | 0.75 | 1.0 |
  | the crouch walk (MT_CROUCH_WALK; every move crouched) | 0.35 | 0.26 | 0.35 |
  | the rest: the walk (MT_WALK), the back step, the aimed side steps (MT_MOVE_*_SLOW), a dodge | 0.5 | 0.38 | 0.5 |

  (Before #324 the volumes were the demo's own: 0.35 for the walk and the side steps, 0.6 the run,
  0.85 the sprint, whoever stepped.) Which of the demo's gaits is which motion is play_liquids.rs
  `gait`'s: the sprint dashes, a crouched move crouch-walks (its back and side steps too, inferred),
  the aimed side steps are the `_SLOW` walks (inferred: if they're the running ones the factor is 1.0),
  and an AI member's side runs (#363, `Gait::SideRun`: the `f_side_run` / `b_side_run` clips of
  MT 26 / 30 / 27 / 31) run.
* **Where.** The controlled character's own step is **2D**: heard at its volume wherever the listener
  is. FUN_00119020 puts its point into the request block's second slot (+0x10), not the first
  (+0, the position with its set byte +0xc), and FUN_00179250 (the cull) and FUN_0017ec80 (the
  voice) read only the first: a request without a position is never culled and plays 2D. Anyone
  else's step is positional at its foot (the bone, as drawn): `WorldSound::heard` takes the library
  entry's distances from the listener (every footstep sound in e34's library has falloff 1 and max
  distance 10: full to 2.8 m, 0.375 at 4.6 m, 0.125 at 6.4 m, silent from 10 m) and drops it at
  the max. (Before #324 a squadmate's steps went in the plain queue, faded linearly to 35 m and
  never below 0.12 of their volume: every step on the map was heard, a far enemy's at 0.10.)
* **The pitch.** U x 0.7 - 0.35 octaves, U in [0, 1) (FUN_00233b90): played at 2^pitch the speed
  (`WorldSound::pitch`), for the wading step and the ground's alike.
* **One draw.** The game draws twice from its one random stream, the pick and then U. The demo spends
  one draw of the character's own random state (`Player::random`) on both, the pick its low bits and
  U its value (`DRAW_STEPS`): it spent one on the pick alone before and one on a wading step's
  pitch alone, and that state also rolls the character's damage, dodges, knock-downs and its AI's
  delays and idle choices, which a second draw each step would move. The same run on main
  (cf6ef48) and here has the same squad, positions, goals and idle choices (below).
* **`BF_SOUND_LOG`** prints each footstep with its kind (4), the volume heard and the volume asked, the
  gait factor, the pitch in octaves and where:
  `t=  3.87 sound 1353c9ab kind 4 vol 0.38 (asked 0.38) gait 0.50 pitch -0.101 oct 2D` is the
  controlled character's own step; `... vol 0.15 (asked 0.50) gait 0.50 pitch -0.041 oct at
  [x, y, z], 5.2 m from the listener` a squadmate's, and past 10 m the line ends "not heard
  (beyond the entry's max distance, or no entry)" (every character on the map asks, near or far).

Verified (`BF_MUTE=1 BF_BACKGROUND=1 BF_SOUND_LOG=1`; 15 fps; the four unit tests of
`liquids::tests`, 113 in all, pass). The staging is the squad takes' (squad.md): `BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,-25,-78,27
BF_TEST_NO_HOSTILES=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_TEST_GO_AT=3`, 360 frames,
Tex's gait by one hook each:

| Hook | Tex's steps (all 2D) | The squad's steps (3D) |
|---|---|---|
| `BF_TEST_WALK=1` | 39 at gait 0.50, 0.38 | 81 at gait 0.50 asked 0.50 (walking after him), 76 at 1.00 asked 1.00 (running into their places, and to him as he stops) |
| `BF_TEST_SPRINT=9` (the run, then the dash) | 29 at gait 1.00, 0.75, the run's and the dash's alike | 159 at 1.00, 9 at 0.50 |
| `BF_TEST_CROUCH=3` | 35 at gait 0.35, 0.26 | 84 at 1.00, 83 at 0.50 |

* One run shows the three factors (`BF_MAP=flat BF_NO_SQUAD=1 BF_TEST_GOTO=0,0,0,-150
  BF_TEST_WALK=1 BF_TEST_SPRINT=6 BF_TEST_CROUCH=12`, 330 frames): Tex's steps walking to 6 s are
  gait 0.50 at 0.38, the dash from 6.1 to 12 s gait 1.00 at 0.75, then the crouch walk gait 0.35 at
  0.26.
* A squadmate's heard volume follows `distance_gain` on the entry's 1 m and 10 m: running (asked
  1.0) 1.00 at 2.4 m, 0.97 at 2.9 m, 0.80 at 3.4 m, 0.47 at 4.3 m, 0.12 at 6.4 m; walking (asked
  0.5) 0.15 at 5.2 m, 0.12 at 5.6 m, 0.06 at 6.4 m; beyond 10 m "not heard".
* `BF_AUTOPILOT=1` (`BF_MAP=sdm_e34`, 500 frames): Tex's 36 steps are 25 at gait 1.00, 0.75 (the run,
  its turn, the sprint, the firing run) and 11 at gait 0.50, 0.38 (the aimed side step, the aimed
  back-pedal, the dodge). The script has no walk or crouch walk: the hooks above do.
* The pitch over those runs' 857 footstep requests runs from -0.348 to +0.350 octaves, each
  tenth-octave bin within -15% and +12% of an even share (the characters start from the same random
  state, so their draws repeat each other: not 857 independent ones); the unit test draws 1,400
  steps of one character and checks each bin (130 to 270) and each of the four variants (270 to 430).
* Hawk fading in (`BF_MAP=flat BF_CHARACTER=2 BF_TEST_GOTO=0,0,0,-60 BF_TEST_SKILL=0.5`): her run's
  steps 0.67, 0.49, 0.26, then 0.19 (a quarter of 0.75), the noise factor on the same rule.
* The liquid branch is unchanged: sdm_e10's pond (`BF_MAP=sdm_e10 BF_CHARACTER=3
  BF_TEST_GOTO=74,5,74,-30 BF_TEST_WALK=1 BF_LIQUID_LOG=1`) prints the same 25 wading and silent
  lines on main and here (volume 0.38, pitch, depth and time), and Tex's 33 steps in all are the
  same sounds at the same times (the ground's after the pond at 0.38 where main's were 0.35).
* No regression: e34 for 60 s with the squad (`BF_TEST_GOTO=-83.39,-82.79,-25,-78,27
  BF_TEST_SPRINT=20 BF_ORDER_LOG=2 BF_AI_LOG=1 BF_MOVE_LOG=1`, 900 frames) logs the same 608
  non-sound lines on main and here: the same positions, modes, goals, dashes and idle choices. Its
  frames (600 at 15 fps, every tenth, two runs of each build) are within the renderer's own
  run-to-run noise: no more than 5 levels apart anywhere but a mist at a rock's edge (x 705-715, y
  138-170) that flickers between two states by up to 12 levels over 5 to 90 pixels, in main
  against main as in main against this (frame 150: main's first run against the other three;
  frame 160: this build's two against main's two).
* take15 (todo/36-squad-voice F1; the footsteps are in the audio only, no step log, and the audio is
  dual-mono: its two channels are one signal, so it can't show where a step is): Tex walks at stick
  (0.5, 0.5) 2-6 s (MT_WALK 47 and MT_MOVE_RIGHT_SLOW 33: factor 0.5), crouches without moving 7-12.5
  s, runs from 13.1 s and dashes from 13.4 s (MT_RUN 34, MT_DASH 8: 1.0). The developer's reading of
  the 3.5-10 kHz band's step transients above the quiet background (17.6-18.9 s) was 0.55e-3 on
  average for the walk's 8 steps and 1.1e-3 to 1.2e-3 for the dash's 9 and the run's 1: about
  twice. The tester's sensitivity runs on the same take give 1.55 to 2.65 by band and detection
  threshold and 2.9 to 3.4 by band power, and the estimator can't tell main's old volumes (1.7 and
  2.4 times) from the rule's (2.0). So the take is consistent with the dash's steps being at least
  about twice the walk's, and no more than that: the rule rests on the asm (FUN_00119020), and the
  take can't confirm the 0.75, the falloff or the pitch spread (the steps are barely above its
  hiss, the squad's steps are mixed in, and with one channel there's no panning to tell 2D from 3D).

Guessed or open: the gait of the aimed side steps and of the crouched back and side steps (above);
the foot events themselves (the game's come from its clips, the demo's from feet planting); whether
a cutscene camera takes the controller's view out of the view manager's slots, which would make the
controlled character's step 3D at x1.0 (FUN_00112790's second gate: FUN_0008b790 finds the
controller's view, FUN_00072b60 asks whether it's in the manager's seven slots, FUN_0008b880 whether
its team plays on; the script camera 101 leaves the view in its slot, per the code map's
FUN_00072b60 row); the sound manager's queue and its budget of 16 positional voices
(FUN_001793e0: every step plays); that every character starts with the same random state, so their
k-th steps have the same variant and pitch (the game's one shared stream doesn't); how it sounds
(muted: by ear is the user's check); the wet footprint decal (char+0x418, not in the demo).

The shot that would settle the level, the place and the pitch (the tester's tightened F1): the squad
on Stand Ground far away, about 30 steps each of walk, crouch-walk and dash on one surface, plus an AI
member at 3, 6 and 9 m, captured in stereo (the stereo and the distances are for the 2D place and the
falloff, the 30 steps for the level and the pitch spread).
