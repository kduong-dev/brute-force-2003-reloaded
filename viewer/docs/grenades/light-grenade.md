*Part of [grenades/](README.md), the former page `grenades.md`.*

## Light grenade

The Light (h_fd1a966d, only in m09_a/b/c/x) is a flare: thrown as the others, it goes off
1.5 s after it lands with its explosion h_ebdfb28e (Damage 0-0, radius 1, no decal), whose
effect type h_1d09922e is phosphor_grenade and light_phosphor, with the impact sound
h_1080aaf1 at once and f5260e35 0.2 s later.
* phosphor_grenade: init_spike (3 at once, then ~30/s until 28.8 s, none from 29.45 s; each
  streak lives 0.6-1.3 s, white turning blue, 0.2 m growing to 5.9 m tall at 58% of its life
  and 0.3 m wide, rising 2.26 m) makes the tall thin blue beam, drawn as upright streaks (see
  the ALE rules above); init_glow (small-flare.tga - a lens flare with rays - at 60/s for
  0.068 s of every 0.103 s, ~40/s all through the burn, each 0.545 s, white turning blue,
  alpha <= 0.4, thrown up at 1.5-3.7 m/s) the white glow round the base, keeping the random
  roll; shrap (~100/s all through the burn) the rays fanning up at ignition and the sparks
  after, drawn as sparks (see the ALE rules above).
* light_phosphor: one lamp (a particle ~6/s, each 0.51 s, the newest's colour and reach),
  (0.655, 0.725, 0.847) -> (0.333, 0.620, 0.776), reach 18.8 -> 24.3 m, lighting the characters
  and scenery round it (the ground is unlit here: levels.md "Point lights"); a sawtooth in red
  as the recording's (see the steady light above).
* The canister stays where it lies, its trail smoke stopped, until the effects have run (31.1 s:
  the 30 s emitters and their last particles), then goes (`play_grenade.rs` `stays`: a grenade
  whose explosion does no damage; the recording's flares lie under their beams, and the data
  has no flag for it - an inference). `BF_GRENADE_LOG` prints "canister gone" when it goes.

Verified (captures in the scratchpad's light/):
* Test map, close (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.1 BF_CAMERA_PITCH=-0.35
  BF_CAMERA_DISTANCE=4 BF_VIEW_YAW=0.3 BF_TEST_GRENADE_TYPE=Light BF_GRENADE_LOG=1`): down
  2.40 s, ignition 3.93 s ("effects run 31.1 s"); a starburst of rays fanning up on the
  first frames, then a tall thin beam of upright blue streaks from +0.27 s with a white core
  and sparks at its base, Tex's arm and the floor lit blue-white, the canister lying in the
  glow. Side by side with the recording's third ignition (lg f1070, by time since it): the
  starburst at +0.07-0.27 s, the beam's height against a character and its flicker match; the
  recording's also shows two or three parallel streaks at times (f1130). Before, the same
  capture showed a squat glow with long rays fanned in every direction, and no starburst.
* The ground light: the mean of a patch of floor near the flare over 30 frames (+0.8 to +2.7
  s) varied by 2.7% before, dipping one frame in five; now by 0.6%, with no dip.
* Test map, the whole burn at 5 fps (`BF_CAPTURE_FPS=5`, throw hold 0.2: at 5 fps a 0.1 s hold
  falls between steps): blast at t 4.20, the beam and light gone by +30.2 s; "t 35.40: Light
  canister gone" (+31.2 s, the 31.1 s rounded up to a 0.2 s step), and the canister is in the
  frames up to t 35.2 and gone at t 35.4. A capture's frame n shows the time (n + 1) / fps (the
  blast logged at t 4.20 is first seen in frame 20 at 5 fps), so frame numbers read as n / fps
  come out one step early.
* m09_a (`BF_MAP=m09_a BF_TEST_GOTO=28,-118,28,-118 BF_VIEW_YAW=0.3 BF_TEST_THROW=3,0.3`; the
  level has no start points, so the test start is needed): the flare lands ~12 m off in a pool
  and the beam stands against the dark rock. The demo's m09_a is lit much brighter than the
  recording's, so the flare's light is far less striking than in the dark recording.
* Frag and Sonic, the same capture on main's build and this one: the frames differ no more
  than two runs of one build do (Energy likewise at the first change; the later rules pick out
  none of its appearances).

Not matching:
* The beam is paler and whiter than the recording's saturated blue (its middle measured
  (97,145,174) on black against the recording's (48,107,176) to (93,196,255)), and looks less
  solid: the recording's is one dense column, white at the bottom. The demo adds sprites taking
  their colour as linear light (play_fx.rs: right on a mid-grey picture); the console added
  stored values, where a few overlapping dark-blue streaks already reach a solid bright blue,
  and more of them white. That rule is shared by every added sprite and is left to its own
  ticket.
* The white core at the base is smaller than the recording's (about a character's shoulder
  width there), likely the same blend rule: it's the glow's overlapping flares.
