*Part of [grenades/](README.md), the former page `grenades.md`.*

## Sonic grenade (`src/bin/play_sonic.rs`)

"Triggers a powerful sonic blast upon impact." Timer 0: off at its first contact, no bounce;
the explosion h_faced66c (Damage 39-71.5, damage-type 7 = DTYPE_SONIC, radius 10); the effect
type h_0c18f6cc: grenade_sonic (sonic_grenade: the dome and ring; sonic_grenade_flash: godray
shafts) with light_sonic_grenade and the sound f36fb063 at the first blast frame; decal
h_fb24bcb7; its trail grenade_trail with f6dbdb17. Measured from the recording
(todo/reference/Sonic Grenade.mp4, the reference and tester agents' frames; the second blast at frame
449, camera still): a glowing dome growing from +0.03 s, flattening into a ground ring at
~0.25-0.3 s, godray shafts +0.27-0.85 s, the glow gone by ~1 s; hits at +0.17 s (~2 m),
+0.23 s and +0.47 s; no camera shake.

* **The damage comes with the ring** (an inference from those times): every living body within
  the 10 m when it goes off is hurt once, d / 11.68 m/s after it (sonic_grenade.emt's speed at
  its first key), no sooner than the other blasts' 0.1 s: +0.17 s at 2 m, +0.43 s at 5 m,
  +0.86 s at 10 m (on the first frame at or after that time). A body out of the radius by then
  is missed. Damage max falling to nothing at the radius (the other blasts' rule), times the
  damage-type factor.
* **The thrower** takes 0.653 x Damage max x (1 - d / 5 m): 28 HP at 2 m, nothing from 5 m
  (on STANDARD: measured there, it's taken through the difficulty step like the Frag's, see
  **Damage**). Fitted to the recording's two throws, not the game's formula: Tex lost ~28 HP at ~2 m (T1,
  below the data's minimum of 39) and nothing at ~5 m (T2: his health frame flashed red, the
  bar didn't move, no tint; with ~9.5 HP left a flat share would have killed him). A thrower at
  1, 3 and 4 m (a new shot) would settle the shape. The red tint comes only with a hit of
  1 HP or more on the player (`TINT_MIN`, a guess: the falloff still gives a few tenths of an
  HP at 5.0 m, measured from the pelvis, where T2 had no tint); it isn't scaled with the
  damage, since only the Frag's ~12.5 HP tint was measured.
  The tint quad keeps the picture's alpha (`KeepAlpha`, play_grenade.rs), which carries the
  glare's key (#289, levels.md § The glare). The game lays the tint over the finished picture,
  after the glare; the demo multiplies it in before, so while it's on the glare dims with it.
* **Nobody is knocked down by it** (the recording's Tex and an enemy ~1 m away stayed up; a
  hit of 15 HP or more floors a body two times in three otherwise). The ring's hit holds the
  body's knock-down cooldown for the call, so a knock-down already pending from another hit
  that frame is kept and the log doesn't claim one.
* **The dome** (`src/ale_fx.rs`, Sonic only, by node name):
  * sonic_grenade.emt has no rate and no initial count; its emit count (e7221f95, keyed
    27.6 -> 38.3 over 0.007-0.297 s) is read as the opening burst (28 particles, the rule
    the Frag's flash uses) and then as particles per second for its 0.35 s life (~11 more,
    `EMIT_COUNT_RATE`). With the burst alone the spread never got to open into the ring; with
    one particle at the start (and the rate) the first frames were a few flat sheets where the
    recording's dome is whole; per 30 fps frame (~300) it was a solid white blob.
  * Its quads (sonic_grenade.app) are GeneralApp_FacingType 2 with AngleFadeOut: square to
    their own motion, so they face out along it (#286: the game's shader, [maps.md](../maps/ale-blends-and-particle-facing.md)
    **Particle facing**; before, the demo's `PERP_OUT_CONES` rule by name did the same), and
    fade seen edge-on: a see-through dome while its spread is 28-74 degrees, then, as it opens to
    90 by 0.25 s, a ring of quads standing round the blast (from above, a ring with a dark
    middle). Lying flat in the emitter's frame (before #81), it was a flat streak. Tried and not
    kept: the quads lying along the cone's surface (a funnel flattening into a disc) - from above
    a filled disc, not a ring. With the fade (#286, `t286dev/side_sonic.png` beside main's build
    and the recording) the dome is as before but its rim softer: the quads at its outline, seen
    edge-on, are gone.
  * Its air field sonic_grenade_air.fld (a drag: a wind of 0.004 m/s, Approach 0.80 -> 0.15
    by 0.43 s) stays off. Applied 6 times a second (the first version's fit to the early
    half-widths) it held the particles in a ball: 18-26% of the dome's brightened pixels fully
    white (the recording 0-2%), and the ring only 5.3 m out at +0.6 s where the recording's is
    past 6.3-9.5 m. Off: 8% white on the first frame, 0-2% after (sdm_e34, `s81/sat.py`), and
    it spreads faster. The early dome is still wider than recorded (1.4-2x at +0.07-0.2 s, the
    tester's measure with the field; wider without it): the quads' own size, 0 -> 5.5 m over a
    quarter of their life, sets that width.
  * sonic_grenade_flash.app's godrays (godray.tga, aspect 0.5 -> 15.8; facing 3) stand
    vertical from their first frame, turned about the vertical to the camera: the game's facing
    3 built about the world's up instead of their motion (`VERTICAL_STREAKS`, a guess kept by
    #286). Along their motion (54-83 degrees out), as the game's facing 3 builds them, they lay
    nearly flat as a white glare over the ground (#286's capture on sdm_e34; also #81's), where
    the recording's stand up. Why isn't settled: the game's FxCubeEmitter spread reaches down to
    26 degrees (r in [-1, 1), code-map 0x2b17c0) where the demo's spawn takes 54-83 (#164). The
    Energy's stun_hit_s godrays are unchanged.
* `BF_GRENADE_LOG` prints, per body in reach, when the ring will reach it and when it hits;
  `BF_COMBAT_LOG` the damage.

Verified (15 fps; scratchpad `s81/`):
* Test map, `--test BF_TEST_GRENADE_TYPE=Sonic BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.4
  BF_TEST_INSTANT_KILL=0 BF_TEST_TARGET=1 BF_CAMERA_PITCH=-0.3 BF_CAMERA_DISTANCE=6
  BF_VIEW_YAW=0.9 BF_GRENADE_LOG=1 BF_COMBAT_LOG=1` (`W1`): blast at t 2.80; due times
  Brutus +0.28, Tex +0.43, Hawk +0.46, Flint +0.51 s, hit on the next 15 fps frames (+0.33,
  +0.47, +0.47, +0.53 s; the first version hit a frame early); Brutus 40.6 HP at 4.3 m, Tex
  (the thrower) 0.1 at 5.0 m, Hawk 18.5, Flint 3.4; nobody floored, no "knocked down" lines.
  With `BF_TEST_THROW=1.5,0.15` (`W2`): Tex 7.8 HP at 4.2 m. After `TINT_MIN` (`W4`): the
  same hits, and Tex's 0.1 HP comes with no tint.
* sdm_e34 (`BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.4,16.3`, the same throw and camera),
  side by side with the recording's frames 449-509 at matching times (`side_fina/b.png`): a
  see-through dome with a brighter rim from the first frame (the recording's shape; bigger),
  then the ring and the vertical shafts to ~+0.8 s, the decal from +0.8 s. Tex (the thrower,
  4.9 m) takes 1.1 HP and the tint (the recording: nothing at ~5 m; a thrower 0.1 m farther
  gets no tint, `TINT_MIN`). The 0.4-0.67 s pale
  white-blue on Tex the tester saw (the field version) is gone; the light effect is main's.
  **Doesn't match**: the demo is whiter and brighter overall than the recording's dark blue
  (the shared additive rule, #104); the shafts are several broad pale columns where the
  recording has two narrow ones, one upright and one leaning ~15 degrees out (the data's
  ~45 a second, 1.1-3.5 m wide quads); the ring is a soft wall of quads, not a thin flat
  band; the decal is hard concentric rings, ~3.8 m, seen from +0.8 s, where the recording's is
  a soft oval of ~2.2-3.3 m (an earlier measure said ~2.5x Tex's height) first seen at ~+1 s -
  left at the Frag's half-size reading and delay.
* **The look and the additive blend (#104).** The dome too wide early, its saturation, the
  characters inside it turning pale grey-blue and the broad godray columns all come mostly
  from how added sprites are summed. The demo adds each sprite's colour as linear light: over
  a dark ground a sprite's faint soft edge (a stored 0.02) shows as a stored ~0.15, so the
  5.5 m quads' and the godray texture's wide faint margins show where the console's would
  vanish. A test (not kept) turned each Sonic sprite's colour x alpha x texel into linear
  light first, as a stored value (exact for one sprite over black): on sdm_e34 the added
  colour came to 0.53-0.62 : 0.75-0.79 : 1 (the recording's 0.53 : 0.77 : 1; linear: 0.70-0.77
  : 0.85-0.91), no white pixels, the godrays narrow and Tex unlit - but far too faint, because
  overlapping sprites then sum in linear light (N sprites give N^0.45 x one, where the console
  gives N x one): a thin pale dome, the ring and shafts nearly gone (scratchpad
  `s81/side_dka/b.png`: recording, linear, stored per sprite, over the darker floor at
  -78.4,0.6). Matching the console needs the added sprites summed in stored values (e.g. drawn
  to their own buffer and added as stored values), a renderer change for every added sprite:
  left to #104. Until then the Sonic stays on the shared linear rule.
* Frag, Energy and Gas, pixel-diffed against main's build (the Frag command in **Verified**
  above, with each type): Frag 9.3k pixels over 75 frames (main against itself 7.7k), Gas 5.5k
  (5.6k), Energy 18.6k (8.7k) - small scattered differences in the Energy's random bolts; the
  frames look the same.
