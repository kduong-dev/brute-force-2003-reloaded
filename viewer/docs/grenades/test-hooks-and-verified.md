*Part of [grenades/](README.md), the former page `grenades.md`.*

Test hooks (with `BF_TEST_GOTO`, on the test map or a map):
* `BF_TEST_GRENADE_TYPE=<label>` (any case, e.g. `Sonic`) selects that type at the start,
  giving a stack of it if none is carried.
* `BF_TEST_THROW=<s>[,<hold s>][,<s>,<hold s>...]` holds G from s for hold s (default
  `CHARGE_TIME`, 0.625 s: a full charge), as many times as given (a placed type: one per press). At
  15 fps a hold under one step (0.067 s) can fall between steps.
* `BF_TEST_NEXT_GRENADE=<s>[,<s>...]` taps Tab (0.1 s) at each time: the next item (#196; it
  pressed the T key before, which stepped the grenade types only).
* `BF_TEST_DETONATE=<s>` sets off every grenade out at that time (through the fuse: a cutscene
  holds it). (The Sentry's own hooks: see **Sentry**; the Roller's `BF_ROLLER_LOG` and
  `BF_ROLLER_RAYS`: see **Roller**.)
* `BF_TEST_EXPLOSION=1` sets off the selected type 6 m ahead every 1.5 s.
* `BF_TEST_DROP=<s>[,<s>...]` sets off the selected type at the controlled character's feet at
  each time (to stand in a Gas cloud). `BF_GRENADE_LOG` prints each cloud, when it's over and,
  per character it reached, when it poisoned them, how much and at what rate.

Verified (captures at 15 fps in the scratchpad; the reference frames are the user's xemu
recordings at 60 fps):
* Frag, test map (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.3 BF_CAMERA_PITCH=-0.5
  BF_CAMERA_DISTANCE=9 BF_VIEW_YAW=0.2`): button-up 1.8 s, leaves the hand 2.40 s, down 0.13 s
  later 3.2 m ahead, blast 4.07 s: 2.27 s after the button (recorded 2.28-2.38, input to
  picture: the game's ~2.1-2.2 s once the lag and input delay are taken off, #140, so the demo's
  is likely ~0.1 s late, by the release above; the fuse itself fits), 1.67 s after leaving the
  hand (recorded 1.65-1.78, picture to picture). Side by side with frag/a from its flash (frame 693) at
  matching times, 0-1 s: a white haze on the blast frame; the red tint two capture frames later
  (the recording's 701), as Tex 6.6 m away takes 12.6 of 115; the fireball turns orange at
  ~0.4-0.5 s and brown after, with no second white flash; a dark 6 m scorch after. The demo's
  blast is farther from the camera, so smaller on screen; the recording's smoke spreads wider
  and darker. Four captures (three run at once) give the same bright-pixel counts frame for
  frame.
* Sonic: see **Sonic grenade**.
* Light (`BF_TEST_GRENADE_TYPE=light`): see **Light grenade** above.
* Hand-over during a throw (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.07
  BF_TEST_SELECT=1,1.6 BF_GRENADE_LOG=1`): the count drops to 9 at the button, control passes at
  2.07 s, before the grenade would leave the hand (2.20 s without the hand-over), and
  `BF_GRENADE_LOG` prints "t 2.07: throw cut short, grenade back (10 now)": nothing is thrown.
  The recipe asked for the select at 1.1 s, which no longer shows it: the hand-over comes 0.5 s
  after the ask (`SELECT_TIME`, 0.6 when this was written), at 1.6 s, as G is let go, and
  depending on the frame it lands in the throw goes ahead (the tester's runs, on main's build and
  #161's) or never starts (a run at 15 fps here).
* Sentry (`BF_TEST_GRENADE_TYPE=Sentry`, ten presses 1 s apart): the box counts 10, 9 ... with
  no digit at 1, then shows Frag 10; `BF_TEST_NEXT_GRENADE=12.5,13` steps to Energy, then Gas.
* Roller: see **Roller** for its seeking and roll. Its sound (take09's staging, `BF_SOUND_LOG=1`,
  30 fps): down at 2.00 s, h_19dbc65d at 2.00, 3.00, 4.00, 5.00 s (0.97 s on the 30 fps step),
  0.80, 0.80, 0.53, 0.20 as it goes away.
* Gas, test map, standing in the cloud (`--test BF_TEST_GOTO=0,4,0,3.5 BF_CAMERA_PITCH=-0.15
  BF_TEST_GRENADE_TYPE=Gas BF_TEST_DROP=2 BF_GRENADE_LOG=1`): blast at 2.07 s, Tex poisoned from
  +0.13 to +5.60 s (the first 15 fps step past +0.083 s), 97.5 HP at 17.8 HP/s, health 115 ->
  17.5. As Flint (`BF_TEST_SELECT=1,0.5 BF_TEST_DROP=3`): 0.9 HP/s
  (x0.05). Side by side with the recording's second cloud (gas f844 on, by time since the
  blast): the cloud thickens by 1-1.5 s and from 1.5 s veils Tex and the ground olive as
  recorded; the recording's is darker and browner (over a darker map) and its big white
  flash flickers for ~1 s where the demo's is one short burst. The demo's cloud still hangs at
  6.5-7 s (the data's puffs live 2.3-4.4 s from an emitter running 4.3 s); the recording's
  first cloud is half gone by ~6 s (low confidence: Tex walks out of it then). From 12 m the
  cloud is ~6.5 m wide and ~5 m tall (recorded ~6 x 4 m, low confidence).
* Death in the cloud with a squadmate in it (`--test BF_TEST_TARGET=1 BF_TEST_INSTANT_KILL=0
  BF_TEST_GOTO=0,4,0,-3 BF_TEST_HEALTH=40 BF_TEST_GRENADE_TYPE=Gas BF_TEST_DROP=4
  BF_GRENADE_LOG=1`): Tex dies at +2.40 s (41.1 HP at 18.1 HP/s); Brutus, standing in it, is
  poisoned from +0.13 to +5.60 s, 97.5 HP at 17.8 HP/s, through the death camera and the
  hand-over (the whole 5.5 s at the rate: no gap, no catch-up).
* Frag and Sonic with the field allow-list (the Frag command above, and with
  `BF_TEST_GRENADE_TYPE=Sonic`), pixel-diffed against main's build frame for frame: the only
  differences are the trail's puffs (around the hand and along the flight, and the Frag's
  rising trail column lit by its blast); the Sonic's dome and ring and the Frag's blast are
  identical (before #81, which changed the Sonic's).
* Energy: see **Energy grenade**.
* No crosshair of the grenade's own (#119; `BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.4,15.5
  BF_TEST_THROW=1.5,2.0`, 75 frames): the gun's crosshair keeps the same box (HUD-blue pixels
  x 306.7-334.7, y 177.7-207.0, centred on (320.7, 192.3)) on every frame, before, through the
  2 s charge, the release and the meter's hold; no blue pixel outside its first-frame mask in
  the window x 280-352, y 155-230;
  the meter's outline x 354-368, y 162-222 from 1.53 to 4.07 s. Side by side with `todo/119`
  take01 f138, f170, f264, f286, f313 (the Therm Sweeper's arcs stay, the meter beside them in
  the same place). Edge cases: a hand-over mid-charge (`BF_TEST_SELECT=1,2.0`) shows the new
  member's crosshair (Flint's) and nothing else; a death mid-charge (`BF_TEST_DIE=2.5`) hides
  the crosshair from the death frame; a weapon switch (`BF_TEST_SWITCH=1.5`) still hides it
  from the drop to the grab; a Roller press shows no meter and keeps the crosshair. In the scope
  (`BF_TEST_WEAPON=L-Shot-75 BF_TEST_AIM=1`: the autopilot aims from `BF_TEST_AIM`, and a
  zooming gun scopes; `BF_TEST_SCOPE` isn't read under the autopilot) the crosshair stays at
  (320, 239.7) through the charge and the throw, and since #162 the meter beside it (the fill
  x 355.3-366.0, y 212.0-266.7 at frame 40: the unscoped place + 48, the corner at (352, 208));
  the game's scoped throw isn't filmed, and the demo's throw doesn't leave the scope as the
  game's LT release does (a follow-up).
* The meter's fade and inside (#162; `BF_CAPTURE_FPS=30 BF_MAP=sdm_e34
  BF_TEST_GOTO=-44.4,15.5,-44.6,15.1 BF_TEST_THROW=1.5,2.0 BF_CAMERA_DISTANCE=12`, the meter
  over the dark wall panel): the count drops at frame 106, the meter is full through 121 and its
  fill's strength over the wall from 122 on is 0.80, 0.64, 0.50, 0.39, 0.28, 0.21, 0.15, 0.10,
  0.05, 0.04, 0.02, then gone (the take, #119 spec §5.2: 0.80, 0.66, 0.51, 0.42, 0.33, 0.24,
  0.15, 0.11, 0.07, 0.04, 0.02; the code's curve 0.81, 0.65, 0.51, 0.39, 0.30, 0.22, 0.15,
  0.10, 0.06, 0.04, 0.02). The first faded frame is 0.533 s after the count drop (the take's
  0.517). The ground (wall, Tex's arm) shows through the box above the fill while it charges,
  as take01 f170's sand does. The outline keeps its colour (brighter and harder than the
  take's, out of #162's scope). The Jax-iR
  (`BF_TEST_WEAPON=Jax-iR`) keeps its own crosshair (h_0ef11ee8) through a charge.
