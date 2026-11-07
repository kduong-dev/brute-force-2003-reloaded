*Part of [levels/](README.md), the former page `levels.md`.*

* **Sky.** A level's `<sky><object mesh-name>` is a set of layers, drawn in order. On Bulgar:
  a flat blue top, a panorama band of mountains and clouds, a moon, and a cloud swirl. They use
  the self-lit shader `h_f539fe8c`, whose texture is `h_e01baa40` (the texture lookup missed it,
  so skies were flat grey). See-through layers are blended. The sky moves sideways with the
  camera, so its mountains stay on the horizon (the game's stays at its `<position>`,
  FUN_00177d50: a few degrees of horizon parallax across a map, left as it is).
* **The sky is drawn first (#290).** The game draws the sky first in a camera's render
  (FUN_000d9f00 runs FUN_00177d50 before the terrain, FUN_001f5600, and the object lists), so
  everything else lies over it. Every level with a sky has see-through layers (`BF_SKY_CHECK=1`
  in bf_level lists them: 51 levels; sdm_e13's are an opaque top, a blended panorama, the rock
  walls round the map, 256 x 512 with 25% of its texels see-through, from 179 m below the sky's
  origin to 1990 m above it, and a blended band 369-563 m up, 99% see-through), and bevy draws
  those with the other see-through meshes, back to front by their origin's depth in the view
  plus their material's `depth_bias`. Each layer has a material of its own whose `depth_bias`
  sorts it first of all, before the
  terrain's layers, +10 for each layer after the first (`SKY_SORT`, -2e6; its depth-test bias
  stays 0, `Lamps::layer_bias`): the layers draw in the mesh's order whatever the camera's
  pitch, and everything see-through goes over them.
  * Before, the layers were sorted by their origins alone, stacked 1000 m apart straight above
    the camera. With the camera level or looking down those lay on the view's plane or behind
    it, so a blended layer was drawn after every see-through thing in front of the camera and
    painted over it wherever no solid surface stood behind it: particles, smoke, glows and
    see-through materials showed against the ground and were cut off along the skyline. With
    the camera pitched up the layers sorted far off and the same things showed, which is how
    #258's checks (pitched up 0.25) saw the tutorial's embers in its sky. The layers' own
    order turned over with the pitch as well.
  * Verified (#290; scratchpad t290dev): sdm_e13 at the south shore (`BF_MAP=sdm_e13
    BF_TEST_GOTO=-19.97,-122.8,-19.97,-122.8 BF_CAMERA_PITCH=0`; frames 60 and 75 differenced to
    find what moves): before, the embers (slava-fuzz) covered the rock mound, the spikes and the
    lava and stopped at the mound's outline, none over the rock walls of the sky; now they go on
    over the sky, near ones as large soft dots, far ones as specks, as #146 take01's difference
    (frames at 1 and 4 s) shows them all over both players' views. Pitched up 0.35, where main
    drew them there already, the frames differ from main's only by a pixel's fringe along the
    skyline (0.06% of the picture): the terrain's blended layers, 0.02 m above the ground, now
    go over the sky's edge instead of under it. Every slava-fuzz particle is born 1.6 to 36.5 m
    above its object (`BF_ALE_DUMP=slava-fuzz`, 256 births), the game's box (0x64e30), so the
    placement was never the fault. The tutorial's horizon looking down (its landing spot,
    take03 at 31.5 s) now has the pale haze layer over the far island, as the take does, where
    main's reversed order left a dark band and ghostly pale palms. The other placed effects
    against the sky, before and after: [maps.md](../maps/placed-effect-objects.md) "Placed effect objects", "Over the
    sky".
* **The sky turns (#199).** The `<sky><object>` carries `<rotation>` (the starting angles,
  radians: `0 360 0` on e05, e40, m03 and the tutorial, 0.3 on e23, 0.05 on e34) and
  `angular-speed` (h_18180282, rad/s about x, y, z). The game turns the whole model about its
  origin every world step (FUN_00177ba0: angles += dt x speed, the model's frame the identity
  turned about x, y and z by them), every layer together. Turning: e23 and e34 (0.01 about y),
  e05, e16 and the tutorial (0.005), e40 and sdm_e40 (0.004), m03_a/b/c (0.0032), m09_a/b/c/x
  and m14_a/b (0.005 about x and 0.01 about z), m14_c (0.008, 0.005, 0.007), mp3 (0.05, 0.04,
  0.05); every other sky, sdm_e34's among them, is still. `Level::sky_of` reads both
  (`SkySpin`); each layer (`SkyLayer`) keeps the sky's clock, which runs on the game's
  (`frame_dt`: stopped under a menu, the capture's fixed steps), and `follow_sky` turns it about
  the sky's origin, every layer's mesh in the model's frame there, so a tumbling sky keeps its
  layers together. The sense is the takes': the helpers' frame read with its columns as the
  model's axes (`SkySpin::rotation`); read the other way, e40's clouds drifted right. Test hook:
  `BF_SKY_TIME=<s>` starts the sky's clock that far into the level (bf_play and bf_level).
  Verified (`BF_MAP=e40 BF_TEST_GOTO=0.29,166.87,-3.46,176.14,26.13 BF_TEST_GO_AT=99
  BF_CAMERA_PITCH=0.2`, the view of #198 take06; crops phase-correlated against a frame 2 s in):
  the clouds drift screen-left, 2.4-2.6 px/s above the picture's centre and 4.0-4.1 px/s at the
  left (the takes' 2.9 and 4.0: 0.004 rad/s at the game's focal length, 686 px, which the demo's
  55.4-degree view shares), the same at 15 fps and at `BF_CAPTURE_FPS=5`; the rock holds within
  1-2 px (the idle's sway). sdm_e34 over 20 s: the sky still (0.04 px). m09_a's sky over the test
  map (`BF_TEST_SKY="1,m09_a"`, `BF_SKY_TIME=0` and `60`): its planet and nebula turned together.
  Not checked against the game: the starting angle (no take is known to be at a level's start:
  e23's start would show it) and the tumbling skies (none filmed).
* **sdm_m07's background.** It has no sky (`h_ee630063="false"`, a cavern), and its terrain
  mesh is missing, so the background shows through where the terrain should be. There, the
  background is the fog's colour rather than its bright blue.
* **sdm_m07.** It names a terrain mesh that's on no disc archive. It loads without terrain: the
  map is all objects.
