*Part of [pickups/](README.md), the former page `pickups.md`.*

## OrgSen (#21)

`play_item_pickups.rs` (the pickup), `play_vision.rs` (the use), `play_vengar.rs` and
`src/vengar.wgsl` (the view), `play_reticle.rs` (the lock), from `todo/18-ammo-pickups/leads.md` §4
and checked against that folder's take09 (taking it) and take10 (using it), sdm_e13, xemu.

* **What it is.** "OrgSen" (h_f637c4c8; its description h_f9d7fa96: "Easily detects concealed
  organic signatures even through most obstacles.") is two `<inventory>` types of sdm_e13's (and
  16 other levels') objecttypes: the **carried** one, h_11433446 (function-type 12
  IFSET_VISION_ASSIST, group-type 6 IG_ENHANCEMENT, stack index 0, stack-limit 1, `h_1ee2f4ed` 1
  IOU_ATTACH_TO_FACE, `cost` 5000, icon h_1b73c691, mesh h_0a9b2b06), which the campaign's squad
  placements carry (#281's "OrgSen 1 NEW" on e34 and e40), and the **placed** one, h_05a87396
  (function-type 0, group 6, stack index 23, pickup-amount 1, pickup-archetype the carried one),
  which only sdm_e13 places, as two spawn-trigger templates: (4.3, 29.2, 27.4) and (-84.8, 27.9,
  -176.8); the live copy of the first lies at (2.95, 27.38, 27.74), 10 s timer. The ticket's
  guess that it's an objective item is wrong: nothing but the vision mode reads it, and no game
  mode is checked (the same on the campaign's zones as in the multiplayer map).
* **Taking it** is the walk-over of the ammo boxes (above): the base set's "can take" says yes,
  1 of the carried type goes into the squad's (6, 0) stack, held to 1, the pickup is removed, its
  sound plays, PICKUP_OBJECT goes out, the line is "OrgSen". The box (empty before) selects it and
  shows "NEW OrgSen" for 2 s, blinking (take09 4.6-6.6 s). With one carried: the red "OrgSen
  (full)" and the pickup stays (take09, take10's first seconds: Tex starts on the returned
  copy). The copy comes back 9.9 s after (take09; the demo's relay 10.00 s) and a character
  standing there is told "(full)" again.
* **Using it** (the use button, LT; the item box selects it first): FUN_00121170's case 1.
  * With nothing on the face: the stance's `bring2face` overlay (motion type 10, OMT_BRING_TO_FACE;
    `Sc_w1_` / `Sc_w2_` by the weapon in hand), Tex's 1.80 s, Hawk's 3.00, Flint's 3.43, Brutus's
    3.00, the upper body, the character free to move. Its first event (1a6b4920, 0.57 s of Tex's,
    0.43 Flint's, 0.67 Hawk's) is the OrgSen in the hand holder, as a copy (the stack keeps its
    1: FUN_0014abc0 takes nothing out for use types 1 and 7); at its **attach_item** (0x0775fceb,
    1.33 s in Tex's, Hawk's and Flint's first stance, 1.67 in Flint's second) it moves to the face holder and
    its put-on hook sets vision mode 1 (FUN_001479f0: char+0x644 = 1 and the camera's +0xd8 = 1).
    take10: LT at 3.0 s of video, the hand holder from 3.8 s and the face holder from 4.66 s,
    the picture thermal at 4.7 s: 0.67 and 1.42 s of game time after the press, the clip's 0.57
    and 1.33 s (the demo: the clip starts a frame after the press and the vision is on 1.34 s
    later: `BF_PICKUP_LOG`'s "bring2face clip starts" at 3.13 s, "vision mode 0 -> 1" at 4.47
    s, for LT at 3.0). A Hawk who is stealthed pays 5 stamina at the start (the skill's +0x104).
    The box stays pale blue and "OrgSen" through the clip (the colour is the skill's "can use",
    not "a use is pending": take10 3.0-4.6 s) and after; the count of 1 isn't drawn.
  * With it on: LT takes it off **at once**, no clip (FUN_00121010: the item's state 5, then the set's
    "taken off" hook 0x147a20 -> FUN_00126df0(character, 0)). Setting a state plays that state's
    event record of the item's type (FUN_0014a610): the OrgSen's state-5 event names the sound
    h_feaf631d, played at the take-off (take10's audio: a short broadband transient 40 ms after
    the press at 17.0 s, +25 dB above 1 kHz; the demo queues it at the press). The put-on sets the
    item's place state 2 (FUN_0014a880), which has no event record: no sound there, as the take
    has none at 4.66 s. take10: LT at 17.0 s, mode 0 and the normal picture the frame after
    (17.10 -> 17.20 s).
  * The gate is the item use's (FUN_001210c0: the character free, nothing in the second hand, the
    skill's word), as the Medkit's. **Brutus's skill refuses the use type**: "Cannot use", the box
    red (#196 take26). The OrgSen worn belongs to the character: another squad member's view has
    its own mode; a dead character's is dropped (a guess: the death camera is another view).
* **The view, vision mode 1** ([skills.md](../skills/brutuss-spirit-of-vengar.md), "The Vengar view": the same renderer
  branch, FUN_000e81e0, with its own world pass FUN_000e6a70 where mode 2 has the blur, the same
  vision lights and keyed terrain, and a last full-screen quad FUN_000a45a0). take10's picture,
  which fits what the code says of it: the world's luminance mapped onto a palette of dark
  blue-purple to magenta (the lava, the brightest, (190, 66, 214) at the scan lines' peaks); the
  living **and the dead** characters over it in a yellow figure (the heat figures' material,
  the same radial ramp: yellow-white (240, 227, 62) facing the view to orange-red (182, 76, 41) at
  the rim) with a halo, the gun in the hand a dark dithered purple and red; and over everything
  scan lines: a cycle every 4 pixels of the game's 480 lines, the picture times 0.27 at the
  darkest to 1.0, the HUD not under them. The switch is at once both ways (no colour fade: the
  red fade is Vengar's alone). The palette, the figures' ramp and the scan lines' profile are
  **fits** to take10 (the materials are made of their classes' defaults and are in no archive,
  FUN_000a42d0; the numbers are at `vengar.wgsl`'s `thermal` and `scan_lines` and
  `play_vengar.rs`'s `THERMAL`, `CARRIED` and `SCAN`, with the measurements). **The halo is not
  one of them**: mode 1 draws Vengar's (`play_vengar.rs`'s `GLOW`, 0.07 a tap, a fit to #122
  take09's Brutus), and its size and strength were never measured on take10. Against take10's
  terrain (the cycle averaged) the demo's mean is (37, 15, 70) for (36, 14, 73), its 10th / 50th /
  90th percentiles blue 50 / 66 / 94 for 43 / 72 / 104. `BF_THERMAL_RAW=1` shows the vision-lit
  picture the palette is fitted on. The figures are drawn against the world's depth, so
  **an enemy behind a ridge isn't shown** (take10: one 11 m away and 6 m higher behind the rock
  ridge, and two bodies burning in the lava, weren't: the look doesn't show through walls; the
  lock does).
* **The lock through walls.** With the OrgSen on and a gun in hand whose bullet passes through
  what it hits (the Rail RVR, the Rail CLVR and the Saryl-45; the others' h_11caf57b is false):
  FUN_000bf5a0 sets the target selector's "no sight ray" flag, and FUN_000bfeb0 (0xbfbf3) lets a
  character within 75 m of the controlled one (3D, the positions) through without it, so the
  crosshair locks on one behind a wall or another body; and the aim point (0xc5720) is the
  lock's, with no world ray. (`play_reticle.rs` `sees_through`; the test `the_orgsen_sees_
  through_within_75_m`.) Verified on sdm_e13's test session (`-- --test BF_MAP=sdm_e13 BF_TEST_WEAPON=
  "Saryl-45;Confed LZR-23" BF_TEST_INV=OrgSen:1 BF_TEST_ITEM=OrgSen BF_TEST_ITEM_USE=0.5
  BF_TEST_NPC=<a ring of frozen enemies> BF_TEST_SWEEP="0;1.0,11.0,36" BF_RETICLE_LOG=2`): an
  enemy standing 6 m behind another on the crosshair's line, "hidden" with the OrgSen off, is
  not with it on.
* **Blocked or not done.** A Rail's *shot* through a wall at a locked enemy: the lock sees it, but
  the demo's pass-through rays stop at an object's far side (play_rays.rs: n1 1) and no take has
  the shot (take10's notes: "Not recorded ... the Rail-lock-through-wall test with a gun in
  hand": a Rail gun wasn't staged), so whether it reaches is unmeasured. Whether the dead, or
  the characters of the other team, show differently isn't filmed (take10 had one living enemy
  in the open and the dead in lava); the demo draws every character the same. The state 4
  event's sound (h_f37f35df) isn't reached by these flows (which change of an item's state sets
  4 isn't traced), so it isn't played. The OrgSen's model in the hand and on the face (the
  holders' hardpoint h_08d7f793) isn't drawn. Flint's scope overlay, which isn't drawn while a
  mode is on (FUN_000b3970), isn't hidden. The mode isn't dropped by anything but death (and a
  respawn). The Hat (use type 7, IFSET_AIR_FILTER) is still unused.
* **Test hooks.** `BF_TEST_INV=OrgSen:1` carries one (the carried type), `BF_TEST_ITEM=OrgSen`
  selects it, `BF_TEST_ITEM_USE=<s>[,<s>...]` presses LT at those times (put on, then off),
  `BF_PICKUP_LOG=1` prints each use and the mode's changes, `BF_SKILL_LOG=1` the view's
  (`vision mode 0 -> 1`), `BF_THERMAL_RAW=1` the view without its palette.
* **Verified** (`BF_MAP=sdm_e13 BF_CHARACTER=3 BF_TEST_INV=Frag:0 BF_TEST_GOTO=7.34,26.75,3.3,27.6
  BF_TEST_WALK=1 BF_TEST_GO_AT=2.0 BF_PICKUP_LOG=1 BF_ITEM_LOG=1`, 330 frames at 15 fps): taken
  at 4.07 s, "OrgSen" under Tex, the box selecting it with NEW blinking for 2 s, the copy back at
  14.07 s and "OrgSen (full)" red from 14.13 with the green rings (take09). `BF_MAP=sdm_e13
  BF_CHARACTER=3 BF_TEST_INV=OrgSen:1 BF_TEST_ITEM=OrgSen BF_TEST_ITEM_USE=3.0,17.0
  BF_TEST_GOTO=3.51,27.48,3.51,27.48 BF_PICKUP_LOG=1`: the clip at 3.13 s, mode 1 at 4.47 s, off
  at 17.07 s; frames 40-100 the world dark purple, the lava magenta, the scan lines and Tex a
  yellow figure with a halo, as take10's 4.8-17.0 s (`BF_TEST_SWEEP=102 BF_CAMERA_PITCH=-0.05`
  turns the camera to take10's view). Flint's and Hawk's clips attach at 1.34 s; Brutus is
  told "Cannot use".
