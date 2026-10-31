# The follow camera and the field of view (#126, #269)

`src/bin/play_camera.rs` (the rig: `Rig`, `update`, the wall test `wall_test`, the body's fade
`step_alpha` / `body_alpha`, the scope's re-aim `reaim`, the hooks), `src/bin/play.rs` (`FOV_Y`,
`fov_at`, `aim_ray`, `follow_camera`, `can_aim`, `clear_follow_yaw`), `src/arena.rs`
(`hull_triangles`, `convex_meet`) and `src/bin/play_stealth.rs` (the fade drawn). Sources:
`todo/126-follow-camera/` (`spec.md`, `leads.md`, the analyst's `answers.md` Q1-Q11, `notes.md`
and takes 01-21, each with the game's own memory of the follow camera logged at 20 Hz) and the
functions in [code-map.md](code-map.md) ("Camera"). Where the takes and the code agree the page
says so; a fit is called a fit.

The camera is default.xbe's follow camera: its update FUN_000702a0 runs each frame from the
parameters the character gives it (0x128280, the character's vtable +0x12c). It is not an orbit:
it looks along the character's aim from behind a leashed pivot at the head.

## Controls (keyboard and mouse)

* **The mouse turns the aim**, and the camera's view is the aim (the game's right stick). The
  pitch is held to +-60 degrees (FUN_0011e7c0). No lag and no rate cap: the game's camera may
  turn 360 degrees/s toward the aim (FUN_00073e40 compares the limit with half the rotation's
  angle), which never binds. **No zoom wheel**: the game has a fixed boom per character (the
  product owner's call).
* **WASD stays the demo's camera-relative free run** (the product owner's call): in the game the
  character always faces its aim, back-pedals and side-steps (take07). Out of #126.
* **Right mouse (hold, a gun without a scope)** holds the gun raised, its ready timer at 4 s or
  more (`FireState::keep_raised`), so the camera stays in as after a shot and eases out the
  game's way after the release: in full 6.3 s more, out by 7 s. A keyboard-and-mouse adaptation:
  the game has no aim without shooting.
* **Not done** (no gamepad): the stick's profile (answers Q4: x 1/0.85, the pitch at half the
  yaw's rate through (2 - x) x^2, the full-stick boost 1 / 1.075 / 1.15 / 1.375 / 1.87 / 2.5 at
  0.2 s steps, x 0.75 while the gun is raised, the dead zone ~0.29 a fit). The game's numbers
  are in the spec's G1.

## The field of view (#269)

* **70 degrees across the game's 4:3 picture, 55.413 down** (`FOV_Y` = 0.967138 rad): the
  levels' follow-camera archetypes' `fov` (70 in all 54 levels), halved into the view's +0x14 by
  FUN_0006e450 with the aspect held to 4:3. The takes' live read: fov 70.00, half vertical
  27.706; the radiation barrel and Tex's feet project where the picture shows them
  (`fov-check.png`). The demo drew 45 degrees down before, every view 1.27x the game's.
* **Any window keeps the 55.413 degrees down and shows more at the sides** (86.1 across at
  16:9): the window's central 4:3 (960 x 720 of a 1280 x 720 capture) is the game's picture
  pixel for pixel, the takes' black bars being exactly the extra width. The HUD is 4:3 centred
  already. A window narrower than 4:3 shows less than 70 across (the product owner's call).
* **The scope** narrows it by the level's zoom (FUN_0006c720: 2 atan(tan 35 / zoom) across):
  the L-Shot-75's 10 and 25 give 8.01 / 3.21 degrees across, 6.01 / 2.41 down (take15 / 16's
  zfov). `BF_SCOPE_LOG` prints each level's.
* **What follows by itself**: the lock box's and Hawk's refraction's scale (play_targeting.rs,
  play_stealth.rs: the game's view over the demo's, now 1), the crosshair's ray (6.0 degrees
  above the view's axis, was 4.74), the free, editor, death and Vengar cameras (`FOV_Y`), the
  cutscene cameras (already the game's: across / aspect). ale_fx.rs's LOD never read the demo's.
* The far plane stays the demo's 5000 m (the game's is the level's, 100-800 m): not this page's.

## The rig

* **Orientation**: the camera's frame is the aim frame (yaw, pitch; the game's char+0x5bc). It
  looks parallel to the aim, not at the character: at level aim Tex's head is 11 px below the
  centre, Brutus's lower (his boom's 7 degrees). The crosshair is CROSSHAIR_UP of the
  half-height above the centre (HUD y 192), so its ray is 5.996 degrees above the view's axis.
* **The pivot** (`wanted_pivot`, 0x128280): the character's x, z; its feet + its head's height,
  x 0.7 kneeling or going down to it (motion state 6 / 7 or 6 queued; take07: 30.577 = 29.187 +
  0.7 x 1.985). The head's height is "Bip01 Head" at spawn (FUN_0012f440 0x12fefe): the model
  posed by its idle's first frame on its root at the idle's root-height (`camera::head_height`):
  Tex 1.985, Brutus 1.559, Flint 1.645, Hawk 1.488 against the takes' 1.985 / 1.555 / 1.644 /
  1.487.
* **The leash** (FUN_00073710, 0.25 m) and **the height's dead zone** (FUN_00073970, 0.25 m)
  smooth it; the camera reads their **inner points** (cam+0x154, +0x17c; answers Q2): inside the
  leash the point follows by its time inside (less and less: a slow slope creeps), outside it
  catches up over 0.5 s with a weight of 2t a game frame, so at a steady run it is on the
  character and on a stop nothing jumps. The weights are the game's at its 30 Hz; a demo frame of
  dt takes 1 - (1 - w)^(dt x 30) of the gap (`game_frames`).
* **The boom** (`params`): the type's offset-walk (h_0d41e5f1: distance, elevation, yaw;
  FUN_0012eca0) x (1 - 0.4 pitch): Tex 3.9 m at 0.9 degrees, Brutus 3.4 at 7, Flint 3.2 at 1,
  Hawk 2.9 at 1.5. `BF_CAMERA_DISTANCE` overrides the distance (a test hook).
* **The aim shift** (0x1284e1-0x1287fe): the boom gains 1.75 m x s x min(1, 1 - 0.4 pitch) along
  the crosshair's ray, s = clamp(1.5 x the shift timer, 0, 1). The timer (char+0x430) is held at
  3 s while the gun's ready timer runs (char+0x7dc: 4.1 s from a raise, kept at 4 or more by
  each shot; a skill's, Flint's targeting) or the scope is up, for the controlled character, and
  counts down after, 3x dashing. A motion that can't aim lowers the gun at once, the ready timer
  to 0 (FUN_0011e000: the crouch's transitions, a dash, a jump, a dodge, a dive, a slide, a throw,
  a reload, a weapon switch, death; `can_aim`, answers Q7 / Q11). So a tap keeps the camera in
  6.4 s and takes it out over 0.67 s; Tex's boom 3.915 -> 2.187 (back 2.174, up 0.244).
* **Walls** (`wall_test`, FUN_000718f0): seven points from the pivot: two straight back along the
  boom at its distance D, four at D / 2 turned 30 degrees either way, each pair 5 degrees above
  and below the boom. The world's triangles that meet their hull (GJK: FUN_00167c40,
  `convex_meet`, #275) are gathered (none: D); the hull at 0.75 D and 0.5 D is tested against them,
  the first clear one's distance taken; none clear: 0.4 m. No memory between frames. What it
  meets: the static mesh and the collidable objects as they stand (a closed gate's leaf),
  not characters or weapons (answers Q8), **nor the level's blockers** (the invisible walls round
  objects: the takes' camera passes e34's hangar blocker and the range spot's fence's).
* **"Meets" is the game's GJK** (#275): the triangle less the hull, with Johnson's sub-algorithm
  for the simplex's nearest point (FUN_00166c60), in f32, starting from the triangle's first corner
  less the wedge's first point. A search point that doesn't get past the origin is a miss, so is a
  101st round; four points round the origin are a meeting, and so is a search vector within
  0.033 m (its square 0.0011, 0x3c0278). That last makes a wedge a few centimetres short of the
  ground or a roof meet it, but only when the search closes in before it finds a separating
  direction: not a fixed margin. It's what cuts take13's boom looking up at the range spot (the
  wedge clears the ground there by less than 0.033 m: the exact test the demo had missed it) and
  take17's looking straight down in the hangar (2.5 mm under the roof).
* **The distance eases** to min(the wall test, D) along a cubic re-planned each frame
  (FUN_00070090), over 0.2 s moving out and 0.14 s moving in; a frame longer than that snaps. **The
  boom's direction** turns toward the wanted one by its angle x dt / 0.2 a frame (FUN_00073f90):
  the aim shift's 0.9 -> 6.4 degrees of Tex's elevation settles over ~0.6 s while the length
  settles in 0.25 s (take02).
* **Cuts** (Q9): a wanted place 10 m from the camera, or the cut flag (a new camera, a hand-over,
  a teleport, a script handing the view back), puts the camera there at once: the leash and the
  dead zone on the wanted pivot, the boom as wanted, no wall test that frame. Every other
  character's camera is cut to its place every frame (the game's flag for a character no player
  controls), so a hand-over is instant (take21). The hand-over still turns the new character's
  heading to one the wall test leaves clear (`clear_follow_yaw`, the demo's choice: the game's
  hand-over with a wall behind isn't filmed).

### The scope

* **Its camera** (0x12887f-0x128ebe; answers Q1): a second rig, blended in by the scope's
  progress (char+0x7cc, 0 to 1 in 0.15 s): a pivot 0.5 m above the root node (the hips: the
  feet + the clip's root-height) and the camera 0.5 m straight behind it on the aim's axis, every
  character. The blend's boom then goes through the leash, the wall test (steps 0.375 / 0.25 /
  0.4 m) and the ease like any other. The demo's eye (0.45 up, 0.35 ahead) is gone.
* **The re-aim** (`reaim`; answers Q10): entering and leaving the scope the game keeps the point
  the reticle is on (the lock's, else the crosshair ray's within the gun's range, else at that
  range: FUN_00122fa0), relative to the root node, and while the scope button's lockout runs
  (0.425 s in, 0.45 s out at the update's two calls a frame) sets the aim's pitch so the
  crosshair's ray from the camera passes through it (FUN_0011caa0 0x11d0e6; the heading too when
  it lies more than 0.025 to the side). The view doesn't jump at the click. Under the hooks that
  set the aim every frame (`BF_TEST_GOTO`'s `BF_CAMERA_PITCH`, `BF_TEST_SWEEP`) the turn it gives is
  kept on top of theirs (`Rig::hook_turn`).
* **The body** is hidden while the zoom is above 1 (cam+0x4c bit 0).

### The body

* **Its fade** (`step_alpha`, FUN_0011be80): each frame, from the camera's view of the frame
  before, the share of the game's 4:3 screen its bounding sphere's square covers (FUN_0006d240:
  the centre in view space +- the radius over the depth and tan 35 / tan 27.7, clamped, (x1 -
  x0)(y1 - y0) / 4); under 0.6 the alpha is 1, from 0.6 to 1 0.65 - 0.15 (1 - ((1 - c) / 0.4)^2)
  (0.65 at 0.6, 0.54 at 0.8, 0.5 full). It drops at once and rises over 0.25 s. The guns the
  character holds and carries fade with it (answers Q6: 0x212d60 multiplies the same alpha).
  Drawn as Hawk's skin fade is (play_stealth.rs: the nearest surface blended in sRGB terms),
  for all four of the squad.
* **The sphere** [fit]: the game's is its scene node's (the union of the model's part spheres,
  answers Q5); the demo's union (`Character::bounds`) sits 0.13 m lower on Tex and misjudges 193
  of the takes' 3,190 steady rows (rms 0.10), so the fade takes a sphere on the scene node with
  radii fitted to the takes' alpha: Tex 1.28 (rms 0.0035, none misjudged), Flint 1.13, Hawk
  0.97, Brutus 1.0 (their pitch sweeps only: weakly bound).
* **Hidden** in the scope and with the camera within 0.15 m of its near plane (0x126bc0; the
  wall test's 0.4 m floor keeps it out of reach).
* **Another character between the camera and the one it follows** fades by its depth
  (FUN_00116340: |its depth / the followed one's|; `body_alpha`) [code only: take06 is too faint
  to measure]. The demo does it for the squad's four, whose bodies are made ready for fades; an
  NPC or enemy there is drawn whole.

## Test hooks

* `BF_CAMERA_LOG=1`: the camera every 0.05 s in the takes' watch-log order: the time, the feet,
  the aim's pitch and yaw, the smoothed pivot, the camera, the boom before the wall test (d'),
  the eased distance's target and itself (d190), the wall test's step (1 / 0.75 / 0.5 /
  floor), the ready and shift timers (t7dc, t430), the cover and the alpha (a72c), the scope
  level and the field across; each scope re-aim; each character's head height and model sphere
  at spawn. `BF_CAMERA_LOG=2` also prints the triangles the wedge meets.
* `BF_CAMERA_AIM_AT=x,y,z`: every frame the aim turns so the crosshair's ray passes through that
  point (`aim_onto`: the camera swings with the aim, so a few rounds), over `BF_CAMERA_PITCH` /
  `BF_TEST_SWEEP`. Within 0.001 m at 14 m (the e34 barrel from the range spot).
* `BF_CAMERA_PITCH` / `BF_CAMERA_YAW` (the start; with `BF_TEST_GOTO` the pitch is held), held to
  +-60 degrees; `BF_CAMERA_DISTANCE` (the boom's distance, every character); `BF_VIEW_YAW` (the
  drawn view turned about the pivot, the aim and controls unchanged).
* **The recipes in these pages from before #126** aimed with `BF_CAMERA_PITCH` on the old orbit
  camera (3.6 m back, looking at the hips from above): with the game's camera the same pitch puts
  the crosshair elsewhere (the camera at the head's height, the crosshair's ray 6 degrees above
  the view's axis). Re-aim one with `BF_CAMERA_AIM_AT` on the point it meant.
* e34's start runs a letterbox-trigger: `BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05` hides it
  (signal 12 is its STOP) for captures with the HUD.

## Checked (against the takes' rows; e34 unless said, `BF_NO_SQUAD=1`, `BF_CAPTURE_FPS=30`)

Recipes: the range spot `BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-53.56,30.81 BF_TEST_SWEEP=167.8
BF_CAMERA_PITCH=-0.0093` (Brutus `BF_CHARACTER=0` at -50.50,32.70 / 180.3 / -0.0147, Flint 1 at
-52.96,30.96 / 164.4 / -0.0199, Hawk 2 at -52.76,31.02 / 163.2 / -0.0165); the gate
`BF_TEST_GOTO=-55.2,Z,-55.2,Z BF_CAMERA_PITCH=-0.0185` (a goal 0.5 m on, `Z-0.5`, turns Tex's back
to the camera as in take04: a goal on the start faces him the other way); the hangar
`BF_TEST_GOTO=-59.61,6.86,-59.61,6.86 BF_TEST_SWEEP=-0.56` (take17's heading, -0.56 degrees:
the camera 2 cm to -x of the pivot at -1.047, as the take's; at heading 0 the 0.5 D wedge sits
2.5 mm under the roof's collision without meeting it and the boom stays at 2.767); with
`BF_CAMERA_LOG=1` and the letterbox hidden.

| What | The take | The demo |
|---|---|---|
| Tex at rest, the range spot (take01) | camera (-52.73, 31.19, 26.99), boom 3.915 | (-52.733, 31.191, 26.985), 3.915; the central 960 x 720 tiles with take01 v 2.0: the barrel, the cactus, Tex's feet on the bottom edge |
| The limits, +60 / -60 degrees (takes 01, 10-12) | Tex 1.700 (alpha 0.5) / 5.534; Brutus 1.482 / 4.824; Flint 1.395 / 4.540; Hawk 1.264 / 4.115 | the same, to the mm |
| Level, each (takes 10-12) | Brutus 3.420, Flint 3.225, Hawk 2.919 | the same |
| A tap (take02) | 3.915 -> 2.172-2.187 in 0.25 s; out from 6.4 s after the tap, done by 7.2 | 2.187 by +0.23 s (an undershoot to 2.161); out from 6.5 s, done by 7.4 (the press seen a frame late; the ramp itself 0.67 s as the game's) |
| The gate, Z 35.285 / 37.6 / 38.6 / 40.0 (takes 04, 19) | 3.929 / 2.947 / 1.964 (alpha 0.60) / 0.400 (0.50) | the same (alpha 0.61, 0.50); take04 v 6.9 / 9.0 tile with the demo's see-through Tex (at v 9.0 the take's head is larger: Tex is still stepping back there, motion 4, where the demo's stands; the camera is where the take's is but 9 cm higher, 6 of them the demo's ground and 3 the take's dead zone, still on its height from before the walk) |
| A tap with the gate behind (take20) | 0.40 -> 1.10, alpha 0.54, back to 0.40 at t430 ~0.65 | 0.40 -> 1.101 (alpha 0.53), back at t430 0.6 |
| Turning a full circle 1.1 m from the gate (take05) | 0.4 on [-56, +49]; 0.5 D on [-117, -57], [+50, +65]; clear [-134, +109]; 0.75 D [+100, +70] | 0.4 to -57.0 and from +55.5; 0.5 D to -117.0; 0.75 D to -126.6; clear to +105.8; 0.75 D to +71.0 (#275: each boundary within a row or two of #126's, -55.1, +53.6, -115.0, -126.6, +103.8, +69.0) |
| The hangar at -0.479 / +0.061 / +1.047 / -1.047 (take17) | 2.323 / 1.902 / 1.700 / 0.400, camera at -1.047 (-59.610, 31.375, 7.053) | 2.324 / 1.902 / 1.700 / 0.400 (#275), camera (-59.609, 31.377, 7.055) |
| Looking up at the range spot, +0.757 (take13) | 2.039 (0.75 of 2.719); a tap: 1.508, out from t430 0.66, a peak of 2.58 and back to 2.039 | 2.039 (#275; 2.719 before: the exact test missed the ground); the tap 1.511, a peak of 2.63 and back to 2.039 |
| A pitch sweep at the range spot (take01 4.1-5.3 s, 8.6-9.7 s) | cut (0.75 D) from +0.05 to about +0.43, then easing out toward D through +0.5 to +0.7, cut again by +0.8 | steady pitches: 0.75 D at +0.05 / 0.1 / 0.2 / 0.35, D at +0.5 / +0.6, 0.75 D at +0.757 / +0.9 / +1.047 |
| The beach, a tap (take09, e40) | 3.928 -> 2.20, alpha stays 1.0 | 3.928 -> 2.200, 1.0 |
| Flint scoped in the open (take16) | boom 0.500, zfov 8.01, pitch -0.018 -> +0.123 | 0.500, 8.01, -0.0177 -> +0.1230; level 2 3.21 |
| Flint scoped at the gate (take15) | pitch -0.019 -> +0.206, out -0.008 | +0.2119, out -0.0025; after it 1.482 m, alpha 0.57 (take 1.495, 0.58) |
| A crouch (take07) | the pivot down 0.6 m in ~0.3 s, up in ~0.4 | 31.093 -> 30.492 in 0.4 s |
| A dash after a tap | t7dc to 0, t430 down at 3/s (answers Q11) | the same |
| A run's start (take07) | the pivot trails 0.29 m, 0 within ~0.65 s | 0.43 m, 0 within 0.6 s: the demo's run accelerates faster than the take's (the leash itself gives the code's numbers on the demo's motion to the mm). **#345:** take07's stick of 0.7 was the game's WALK (mot 47, about 3.5 m/s: Tex's type walk is 3.6), not a run, so the row compared the demo's run with a walk; the demo's run now moves at the type's 7.8 (play.md "Ground speed") and its pivot trails 0.72 m at a start (0.48 m before: `BF_CAMERA_LOG=1 BF_CAPTURE_FPS=60`, Tex from the gate spot, `BF_ROOT_SPEED=1` for before), under 1 cm again 0.4 s after the peak; the demo's walk is still the clip's 1.6 m/s; no take of the game's run start exists |
| The hand-over to Brutus (take21) | the camera at his place at once, no ease | the same (boom 3.413) |

Checked again after main's level lights (#143, which relit e34, e40 and the tutorial): the
same numbers, and the tiles against take01 v 2.0, take02 v 3.0, take04 v 5.3 / 6.9 / 9.0, take09
v 3.0 and take16 v 3.5 line up as before; the tutorial's start (`BF_SCRIPT=off`) frames Tex as e34
does.

The unit tests (`cargo test --bin bf_play camera`) check the convex test (with a triangle 1.5 cm
past the wedge's tip meeting it and one 9.5 cm past not, #275), the wedge's steps against
a wall, the alpha by cover, the leash's catch-up, the ease's first step and the field of view.

## Not matching, not done

* **Blockers are left out of the wall test** on the takes' evidence (the hangar and the fence);
  which object class keeps them out of the game's camera world isn't traced (the mask's types 15 /
  19 / 24, or no collidable flag).
* **Within millimetres the heading decides**: the hangar's 0.5 D wedge meets the roof at take17's
  heading (-0.56 degrees) and not at 0; the demo's ground and roofs are the game's collision, so a
  spot where the game's camera and the demo's disagree by one step may be a few millimetres of
  heading or pivot.
* **Blocked** (not filmed; the spec's S1-S6): the wall test in the scope, the hand-over with a
  wall behind, a run along a jagged wall (whether the steps hunt: no hysteresis in the code), the
  scoped turn rate, the others' depth fade, Brutus's and Hawk's tap.
* The demo's own: the pose after a shot (`aim_hold`, 0.7 s) where the game holds the ready pose
  the ready timer's 4 s (take02: the gun lowered between 6.5 and 8.0 s after the tap); a gamepad.
