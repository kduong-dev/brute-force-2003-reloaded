*Part of [play/](README.md), the former page `play.md`.*

### The scope: levels, zoom, ramp and lockouts (#62, #61, #7)

`src/bin/play.rs` (`scope_levels`, `level_zoom`, `press_scope`, `scope_input`, `follow_camera`),
the skills' hooks in `src/bin/play_skills.rs` (`SkillKind::scope_levels`, `scope_zoom`,
`scope_view`), the zoom times in `bf/character.rs` (`ScopeOptions`). From default.xbe, as #62's
spec (`todo/62-l-shot-75/spec.md` §3) and the analyst's answers
(`todo/weapons-plan/answers-batch1.md` §10, §11) read it; the views are in
[squad.md](../squad/standing-on-the-floor-and-scopes.md) ("Scopes").

* **Levels** (FUN_00124840): a gun whose zoom (h_efce7f77, data+0x16c) is above 0 has a scope
  (the MK-ASLT and the Kman Auto 2, the Foley 3, the L-Shot-50 5, the L-Shot-75 10); none with a
  gun in the second hand (Tex's berserk). How many levels is the holder's skill's +0x28 asked
  with 1: the base returns it (0x262a40), one level; Flint's (0x13c910) one more, two, with any
  gun that has a scope.
* **A level's zoom** (FUN_00124920): 1 at level 0, else the gun's zoom through the skill's
  +0x2c: the base gives it (0x222cf0); Flint's (0x13c930) gives z + max(10, z + 5) past her first
  level (10.0 at 0x13c94f, 5.0 at 0x3a4fd4): the L-Shot-75 10 then 25 (x2.5), the L-Shot-50 5
  then 15 (x3), the MK and the Kman 2 then 12, the Foley 3 then 13. The takes: x2.47-2.52 (#62
  takes 01, 06, 08, 10) and x2.97 (#61 take01). The demo's x2 second step (a guess) is gone.
* **The button** (CT_AIM_SNIPE, 0x122399-0x122479; right mouse): out of the scope, into level 1
  when the gun has levels and the button's lockout (char+0x7c8) is out, setting it to 0.85 s
  (0x12241a); below the top level, up one with no lockout test; at the top, out of it once the
  lockout is out (FUN_001249b0, which sets the lockout to 0.9 s, 0x124a1c). A reload
  (FUN_00120d40) leaves it the same way, as do the demo's switch, death and Tex's berserk. The
  in sound (the type's +0x260, the character's h_00dd1fe0) plays with each level gone up
  (FUN_00124340), the out sound (+0x264, h_02c94d34) as it leaves.
* **The ramp** (FUN_0011aa20, 0x11acc6-0x11ae79), linear in the zoom: in the scope the zoom
  (+0x7ac) climbs at (the level's zoom - the level below's) / T_in to the level's, never past it
  (FUN_0011ae80), and the progress (+0x7cc: the camera's move to the eye, the model hidden, the
  crosshair to the middle) at 1 / T_in; out of it the zoom falls at the top level's zoom / T_out
  (FUN_0011af10) and the progress at 1 / T_out. T_in and T_out are game-options'
  `<character-options><h_ebe93276 h_1858b846="0.3" h_03a75ca9="0.3">` (0x19f640 stores them at
  settings +0x4e4 / +0x4e8; FUN_0012dfa0 copies them to 0x4699d0 / 0x4699d4). FUN_0011aa20 runs
  twice a frame for the controlled character (its own tick FUN_0011caa0 at 0x11cf03, and its
  follow camera's FUN_000702a0 through FUN_0011c0b0), so a level takes 0.15 s and the lockouts
  0.425 and 0.45 s (`SCOPE_CALLS`; medium confidence on the double call). The takes: 0.17 +-
  0.03 s a level, the L-Shot-50's second level growing 2.0-2.3 zoom units a game frame against
  2.22. The demo's eased ramp (`SCOPE_RATE`, exponential, ~0.3 s) is gone. The game's
  FUN_0011af10 never meets a gun without a scope in hand: its switch swaps the gun at the clip's
  event, after the scope was left and its zoom fell (that gun's top of 0 would stop the fall). The
  demo's gun can change at once (a test hook, a squad hand-back) with the zoom still up: then it
  falls at its own zoom / T_out. A gun in the second hand, or none, keeps the game's top of 1.
* **The view** is the skill's (+0xa8: FUN_000b4070 shows the scope frame h_1807ec04 only while
  its bit 0 is clear, and 0xc6030 makes Flint's overlay only when it's set): Flint's (0x2d5e40)
  1, no frame, her overlay of a corner smear and a dotted ring; everyone else's 0 (Tex's 8:
  nothing reads bit 3), the frame. Both are drawn by `src/bin/play_scope_view.rs` (#283), one
  pass over the picture in the console's sRGB values: the frame 1:1 from the middle, darkening
  the picture to 40% outside its rim, whole at the press and growing 3x as it fades on the way
  out; the overlay added, easing in from 3x to 2x over 0.17 s ([squad.md](../squad/standing-on-the-floor-and-scopes.md), "Two kinds
  of scope view", with how it was checked). `BF_SCOPE_VIEW_LOG=1` prints their state.
* **The field of view** at a zoom is FUN_0006c720's 2 atan(tan(FOV / 2) / zoom) from the game's
  follow camera's 70 degrees across its 4:3 picture, 55.4 down (`FOV_Y`, #269,
  [camera.md](../camera.md)): the L-Shot-75's first level shows 8.01 degrees across, 6.01 down, its
  second 3.21 / 2.41 (take15 / 16's live zfov), the MK's 38.6 / 29.4 (#62 spec §3.3). Before #269
  the demo drew 45 degrees down, every view 1.27x the game's. `BF_SCOPE_LOG` prints each level's.
* **The camera in the scope** is the follow camera's own (todo/126 answers Q1): blended by the
  progress toward 0.5 m behind a point 0.5 m above the root node, the aim re-aimed so the
  crosshair's point stays under it (camera.md, "The scope"); the demo's eye at the head went.

Test hooks: `BF_TEST_SCOPE=<level>[,<from s>]` holds that level (at most the scope's top) from
that time on, under `BF_TEST_GOTO` too; `BF_TEST_SCOPE_PRESS=<s>[,<s>...]` presses the scope
button at those times, with its lockouts; without either, a hook's aim (`BF_TEST_AIM`,
`BF_TEST_FIRE`, `BF_TEST_AIM_HOSTILE`, brain.md) with a gun that has a scope is its first level, as
before; `BF_SCOPE_LOG=1` prints each press, each level change and the ramp's zoom and progress on
every frame they move.

Verified (`BF_SCOPE_LOG`, `BF_SOUND_LOG`; e40's cliff at take05's spot, `--test BF_MAP=e40
BF_TEST_GOTO=-17.21,152.02,-17.70,151.93 BF_CAMERA_PITCH=-0.10 BF_NO_SQUAD=1`, the scope on the
rock 24 m off, `BF_CAPTURE_FPS=30`; e34 opens letterboxed with no HUD):
* Flint (`BF_CHARACTER=1`) with the L-Shot-75, presses at 2, 3 and 4 s: level 1 climbs 1 -> 10 by
  2.0 a game frame (0.15 s), level 2 10 -> 25 by 3.33, out 25 -> 1 by 5.56 a frame (0.144 s).
  Level 2's picture against level 1's: x2.49 (frames 85 and 115, the best of the scales about
  the middle, r 0.96; the takes x2.47-2.52). The L-Shot-50: x2.99 (r 0.96; take01 x2.97), its
  second level 2.22 a game frame. Her MK-ASLT: levels 2 and 12.
* The lockouts (presses at 2.0, 3.0, 4.0, 4.2, 4.6, 4.8, 4.9, 5.2): the entry at 4.2, 0.2 s after
  leaving, is refused (0.5 s left); 4.6 enters; 4.8 goes up to level 2 (no test); the exit at 4.9
  is refused (0.25 s left); 5.2 leaves. Flint's in sound 1fd08ea5 with each level up, her out
  sound 03e0936d with each exit.
* Hawk (`BF_CHARACTER=2`) with the Kman Auto: one level, 2x, in the frame, the gun's crosshair in
  the middle; an exit 0.3 s after entering refused; the shared sounds 135ec2ce / f9ffc972.
* A reload in the scope (`BF_TEST_CLIP=1`, the L-Shot-50's last round): out of it as the reload
  starts, with the out sound, and in again 1.2 s later.
* `BF_TEST_FIRE=1` with Tex's MK on the test map: level 1 at 0.93 s, the scoped rounds 5-6 game
  frames apart (its scoped rate 6/s). `BF_TEST_SCOPE=2,2.0` with Flint's L-Shot-75: level 2 at
  2.07 s, zoom 25.
