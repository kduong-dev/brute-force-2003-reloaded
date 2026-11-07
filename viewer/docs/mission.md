# Mission flow: from CAMPAIGN to the end screen and the next mission (#226)

Step 7 of the tutorial (#28): a new campaign started from the front end's CAMPAIGN, a level's
start (its fade from black, #266), a mission's end (its level-over trigger, or the squad's
death), the end screen (MISSION SUCCESSFUL, SQUAD KILLED), and what follows it: the next
mission's movies and zone, the zone again, the front end.

| File | What |
|---|---|
| `src/bin/play_campaign_menus.rs` | the front end's campaign path: CAMPAIGN TYPE to JOIN CAMPAIGN, then a new campaign (a child of `play_menu.rs`) |
| `src/bin/play_mission.rs` | the campaign game mode, the world's end step, the screen fade, the status line, what an end screen choice leads to |
| `src/bin/play_endmenu.rs` | the end screen h_e8e14fcf and its confirmation box (a child of `play_mission.rs`) |
| `src/bin/play_level_over.rs` | the level-over and zone triggers (a child of `play_triggers.rs`) |
| `src/bin/play_fonts.rs` | the game's font records (body 10, 12, 14, 16 points, heading 24) drawn glyph by glyph: the end screen, the campaign path, the status line |
| `src/bf/menus.rs` | game-options' menus as data: their buttons, lists, keyboard, animations, styles |

Written from default.xbe (the addresses are in the code's comments and in
[code-map.md](code-map.md)); the takes in `todo/28-tutorial/` (take01 the menu path and the
movies, take05 and take10 the squad's death, take12 the ending) confirm the timing and the
layout. What's fitted to the footage says so.

```sh
# the campaign path from the title to the tutorial, on take01's timeline (its presses less 4.8 s),
# the movies skipped; frames: menu_NNN, brief_NNN, frame_NNN
BF_MENU_PRESS="a@0.5;a@2.0;a@4.7;a@7.3;a@12.5;a@15.7;a@18.3;a@27.8;start@30.5" BF_MOVIES_SKIP=4,7,10 \
  BF_MISSION_LOG=1 BF_CAPTURE=cap BF_CAPTURE_FRAMES=45 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the tutorial's ending: both objectives, Tex on the base pad, the end screen's pages, Continue
BF_MAP=tutorial BF_TEST_GOTO=-55,80,-55,80 BF_MISSION_LOG=1 BF_END_PRESS="y@6;y@8;y@10;a@12" \
  BF_TRIGGER_FIRE="<the objectives' chains, then h_f82e0ba6,ENTER_DETECTOR,4>" cargo run --bin bf_play
# the squad's death: Tex dies at 3 s, SQUAD KILLED, its pages, Clone Squad
BF_MAP=tutorial BF_TEST_GOTO=57.5,68.22,57.5,68.22 BF_TEST_DIE=3 BF_MISSION_LOG=1 BF_END_PRESS="y@5;y@7;a@11" cargo run --bin bf_play
```

## The campaign path (the front end)

The main menu opens on CAMPAIGN (take01 at 2 s). Each screen is a game-options menu drawn from
its data: its panel (the picture set h_e462c760, h_ff6b96da for CAMPAIGN TYPE: a narrower body,
the background movie showing through on the right), its texts, buttons, list or keyboard, and its
legends, in the game's font records (a style with no font draws the body font at 14 points:
take01's "CAMPAIGN" 81 units of ink, the record's 83). The animations are the items' own
(`animation-id`): the buttons slide in from y 240 (translation-anims h_e2027fa6, h_f87fb560,
0.25 s; the first one's sound h_e8ace091), the texts fade in (color-anim h_17494bb2, 0.5 s), the
titles type out (style h_189b1ae9: 0.125 s a letter), the selected button or row echoes
(select-anim h_e150ead3: 1.7 x, 50 185 250 at 150 fading, then pulsing as the main menu's).

| Screen | What | A |
|---|---|---|
| CAMPAIGN TYPE h_f9728897 | CAMPAIGN, QUICK CAMPAIGN, RESUME LAST under SINGLE XBOX CONSOLE; HOST SESSION, JOIN SESSION under SYSTEM LINK (headings: style h_e8b4c2d6, filled 100 100 160 at 160); the selected button's description (its buddy-control) at (314, 118) | CAMPAIGN runs the menu sequence h_0ff0b19d (its event-script, event 2 action 1); the others give the error sound (#27, System Link) |
| SELECT PROFILE h_14761942 | a listbox (6 rows in 206 units, each centred; the arrows texture h_1cc0d333 above and below): DEFAULT, INVERTED (the profiles' profile-id strings), "New Profile"; legends A Select, B Back, Y New Profile, X Delete Profile (grey: a built-in profile) | a profile: SELECT CAMPAIGN; New Profile: the error sound (#27) |
| SELECT CAMPAIGN h_e61ae053 | the campaigns saved (none: #27), then "New Campaign" (FUN_000f1430); X Delete Campaign grey | ENTER NAME |
| ENTER NAME h_e37b5c35 | "Enter a name for your campaign"; the keyboard h_f1eabb1b: its keys from the data (ten 40-unit slots a row, five rows; BACK, EXTRA ALPHA, SPACE, DONE wider), the name "CAMP001" ("CAMP" and the first number not taken, 0x7e1f0) standing selected over a lavender box, DONE selected, its word pulsing once a second | a key types (12 at most), BACK takes one back, EXTRA ALPHA switches to the keys' second set (guesses for the selected name: the first key replaces it, BACK clears it; no take types), DONE saves the campaign ("Saving..." on the status line, 1 s) and opens SELECT DIFFICULTY |
| SELECT DIFFICULTY h_1a6884c5 | STANDARD, HARD, BRUTAL (heading 24, centred), the selected one's description; opens on STANDARD (FUN_000f1340: the session's, a new campaign's) | JOIN CAMPAIGN (SELECT MISSION, h_16ddaa42, is passed: a new campaign has one mission, as take01 shows) |
| JOIN CAMPAIGN h_0130d4e8 | four quadrant menus 232 x 129 from (68, 114): the first controller's h_ebad668d (the profile's name, Tex's picture h_0f35d3d4, "Select Character", the spinner with "Tex" between the arrows h_158f87c1), the others' h_1ca7cb7b ("Press START to Join" and the pad h_e8047c6d in their disabled colours); their frames' texture isn't in the data: drawn as 1-unit borders in 153 209 251 at 100 | A: the ready line h_e3cb03c9 ("Controller 1 ready! / Press START to begin!"); START then begins |

B goes back a screen (JOIN after A: back to choosing). START begins the campaign: its flags anew
from the defaults with `campaign/difficulty-setting` written (-4, 0 or 4: game-options'
`<difficulty-settings>`, FUN_000f13c0), the status line's "Saving...", "Saving Campaign...",
"Saving..." (1, 1 and 0.5 s: take01 35.85-38.38 s) over the first movie, the first mission's
pre-movies (`Campaign_Intro`, `Tex_Intro_Final`, `mb01`: [briefing.md](briefing.md)), LOADING,
the tutorial. The game freezes for about half a second after DONE and after START while it
saves (take01 20.56-20.97 s, 35.35-35.83 s): the demo saves nothing (#27) and goes on at once.

## A level's start (#266)

Every level load, a campaign zone or a deathmatch map, ends the same way, and the screen comes
out of black.

* **The code.** The session's pre-update (0x7a070, the first thing each app step runs, FUN_00068d20)
  takes the queued requests (0x7f730; the list at session+0x6cc, each request's flags byte at +8:
  bit 4 frees the world, bit 0 goes to 0x7f670, which clears the in-play bit, session+0x5bc bit 3,
  and goes on to FUN_00080700; otherwise it is a level start). The level start is **FUN_0007f960**: it sets the in-play
  bit, frees the last world (FUN_0007f620), installs the LOADING screen's progress callback
  (FUN_00078e60), loads the level (FUN_00080700, which ends by setting app+0x58, a reset request),
  takes the callback down (0x7fa06) and ends with **FUN_00069780(1.0) at 0x7fa2f, ECX 0 (from
  black), the colour record zeroed (black)**, then the session's +0x240 (0x84a80: its level clock
  session+0x5b8 = 0). The fade is the app's screen fade (FUN_000694f0 draws it every frame):
  k = (the app clock - its start) x one over its time, a fade in draws a black full-screen quad of
  alpha 255 x (1 - k) over everything drawn before it, blended on the picture's own (gamma)
  values, and turns itself off once k passes 1; it is cancelled outright while the session isn't in
  play. The clock is the app's (0x67d00: app+0x38 as seconds, which FUN_00068d20 adds each step's
  (int)(dt x 1000) to and which stands still while the app is paused), so the fade is **one
  second of the game's clock, linear, with the world live under it**: the script, the AI and the
  title card's clock all run from the same start.
* **Where it comes after the load.** FUN_00080700's app+0x58 makes the load frame's update step
  skip (FUN_00068d20) and the next frame's delta zero (FUN_00069270), so the step after the
  load has dt 0 and the one after that the real time of the frame before it, capped at 0.1 s
  (FUN_00069270 in session state 0). Nothing waits.
* **take01** (the tutorial, from the campaign path; LOADING 267.6-269.35 s) and **take12**
  (m01_a after the end screen, mb02, Brutus_Intro_Final; LOADING 574.27-578.87 s), the mean luma
  of the picture at 60 fps. Each shows the world under black for 0.53 / 0.47 s (luma 0.30-0.36
  and 0.22: the world under 253 of 255 black, the level's first frames), then a **straight line**:
  luma / plateau = 0.938 x (t - 269.693 s) in take01, 1 at 270.759 s (plateau 44.7; rms 0.023 of
  full over 42 frames) and 0.855 x (t - 579.200 s) in take12, 1 at 580.370 s (plateau 26.3; rms
  0.016 over 52 frames). take12's watch log has the app clock itself: `clk` held at 2235249 from
  578.88 to 579.21 s, then running (+75 ms at 579.28 s, +1022 at 580.29 s: 0.94-0.95 of the wall's
  over those seconds; the video's seconds are the wall's to 3e-10), and the picture follows it
  sample by sample: **k of the picture = 0.93 x (clk - 2235249) / 1000 + 0.03, rms 0.02, the
  picture 0.045 s after the sample**, k = 1 as the clock reaches about +1.0 s (sample 580.29 s,
  the picture's plateau at 580.33 s; the fit puts it at +1.04 s, within the samples' 0.07 s
  spacing). So the fade is one second of the app clock, and the video's slopes (0.94, 0.855)
  are the guest's speed against the recording's. The first lit frames come
  0.17-0.22 s after the lines' zero (take01 k 0.07 at 269.92 s; take12 0.03, 0.11, 0.21 at
  579.37-579.40 s): the level's first frames are long ones, the picture stands still while the
  clock holds, then the clock steps on (at most 0.1 s a frame, FUN_00069270) and the picture
  catches up. That reads as the guest's first frames being slow (every object's first update on
  the console, or xemu compiling the new level's shaders on their first draw: the takes can't
  tell which; the watch log's samples are 0.22 s apart across it, 578.99 to 579.21 s, while
  the guest's step is held), not a wait in the code, and it isn't reproduced: the demo's level
  is ready behind its LOADING cover (play_warmup.rs).
* **#232's takes** (`todo/232-campaign-cutscenes/`, xemu with a gdb tap on FUN_00069780 and the
  fade's fields at +0xf0, on a fresh start, so the app clock is 0 at the level's start) read the
  same call live and add the clock. Every level start hits the function with **type 0, colour 0 0
  0, 1.000 s** (take02 m01_a at 8.90 s, take01 e34 6.61, take03 m09_x 15.16 and m09_b 61.43), 0.04
  s before the script's start, and the fade's fields in the watch log read on 1, way 0, rgb 0,
  rate 1.0 and start 0.0 (the clock at the call), the on byte still 1 at clk 1000 ms and 0 by 1098
  (m01_a): the fade runs out as the clock passes its second. **The first running step is 100 ms**
  (clk 0, then 100 at the next sample, 0.12 s later: FUN_00069270's cap on the long first frame),
  and the picture's k (take02's luma / its plateau 26.3, 0.06 s after the sample) follows the
  clock: 0.097, 0.409, 0.505, 0.637, 0.737, 0.846, 0.994 at clk 0.10, 0.39, 0.52, 0.64, 0.74,
  0.87, 1.00 s (and 0.339 at 0.25: a 60 Hz step of the luma; the third m01_a start, with take12's; the black before the first lit frame is
  0.30 s there, 0.47 s in take12, 0.53 s in take01: the long first frames vary). The letterbox
  is still coming in during the fade: the game's view amount is 0.20 at clk 100, 0.50 at 252,
  0.78 at 386 and 1.0 at 515 ms (its show-duration, 0.5 s), and take12's picture has the bars'
  edge at row 19 of 103 at its first lit frame (579.383 s), 36 at 579.40, 51 at 579.50, 67 at
  579.60, 85 at 579.70, 91 at 579.75, 103 at 579.80 (the same amounts as the view's, at take12's clock): the demo's bars and its HUD fading with them (alpha 1 - 2
  x the amount, #380) are the game's, the take only starts later. **A script's own fade replaces
  the level's**: e34_startup's code 120 (dur 2000, type 0, black) at its start makes the fields
  read rate 0.5 (take01 at 6.70 s, off at clk 2055), and m09_b_startup's at its first step (clk
  100) restarts a 1 s one. Code 120 in the script controller (0x1761e5: type 0 from the colour,
  1 to it, colour 0 black or the script's r g b ints, dur ms / 1000, 0 for 1 s) is also what
  take03 and take04 film in colour: m09_b_startup's 1 s fade to red 191 0 0 (the letterbox bars
  go red as 191 x a: top bar R 36 at 74.40 s to 180 at 75.25 s; the pillar columns beside the
  4:3 picture stay black: the fade is the game's 640 x 480 frame) and M09_B_Whiteout's 0.5 s to
  white and 0.5 s from it (luma 38 to 252 and back).
* **The fade as machinery** (`Fade::start_colour(colour, secs, out)` is FUN_00069780's three
  arguments; `start(secs, out)` the black one the level's starts and ends use; the script
  controller's code 120 calls it through `start_script`, #390, [cutscenes.md](cutscenes.md) "The
  screen fade"): a colour (three ints), a way (to it or from
  it) and a time (0 is one second), replacing the fade under way. It is a full-window UI node over
  the world, the HUD and the letterbox bars (z 15), under the panels and the end screen. Its
  quad is blended in linear light: the alpha that darkens a grey of `GREY` (0.30) right, and for
  each channel the colour has the colour that makes a grey of `GREY_COLOUR` (0.12: a brightening
  weighs on the darker pixels) come out as the game's `(1 - a) v + a c` (`quad_colour`). Tested
  with `BF_TEST_FADE=<s>,<r>,<g>,<b>,<to|from>,<secs>[;...]` (fades started at those seconds of
  the game's clock; `BF_NO_FADE=1` to see them alone), 60 a second on the tutorial's opening
  against the same frames without them: **take04's whiteout** (`0.2,255,255,255,to,0.5;0.7,255,255,255,from,0.5`)
  has the mean luma's progress (L - L0) / (255 - L0) within 0.04 of the ideal a on the way up and
  0.06 on the way down (about 0.03 of either is the hook's start a frame early or late), against
  0.12 with the colour fitted at GREY; **take03's red** (`0.2,191,0,0,to,1.0`): the picture's R, G and B whole-frame
  means within 4.1 of the game's blend of them (of 255) all through (apart, R lags by up to 9.9:
  "Not done"), the letterbox bars' R at 191 x a plus
  up to 22 early (a black pixel has no weight in the fit: 16 against 3 at a = 0.017, 50 against
  29 at 0.15, 98 against 80 at 0.42, 185 against 181 at 0.95, 191 at 1.0), G and B 0; **e34's
  2.0 s from black** (`0,0,0,0,from,2.0`, replacing the level's, as the script's does): the
  ratio of the mean luma to the picture's is 0.071, 0.193, 0.320, 0.452, 0.591, 0.729, 0.870,
  0.939, 1.000 at 0.15, 0.42, 0.68, 0.95, 1.22, 1.48, 1.75, 1.88, 2.02 s against the line's
  0.075, 0.208, 0.342, 0.475, 0.608, 0.742, 0.875, 0.942, 1.0.
* **In the demo** (`Fade::waiting`, `LEVEL_FADE`, play_mission.rs): a map's start leaves the fade's
  black node opaque from its first frame, and the fade starts as the cover comes down and the held
  clock is let go (`fade`, the game's clock `frame_dt`, so it stands still under a pause and the
  capture's steps run it): 1.0 s linear in the picture's values (`linear_alpha`, GREY), the first
  frame after the cover at one frame's k. The cover, the title card (its clock is the same
  `frame_dt`: FadeIn at 4.00 s, Full 5.00, FadeOut 15.20, Gone 16.20 at 5 captured frames a
  second, with the fade and with `BF_NO_FADE=1`) and the letterbox are as they were: the frames
  from the fade's end on are pixel for pixel the ones without it (mean absolute difference 0.000,
  max 0). Not in the map editor's edit mode. Clone Squad's restart is a stand-in for the game's
  reclone (#41), which fades in for 1 s as well (FUN_00089da0, ECX 0), and gets the level fade.
  **Test hook `BF_NO_FADE=1`**: no level start fade (the first second of a capture as it was),
  and since #390 none of a script's 120s. Every `BF_CAPTURE` run now starts at the fade's first
  frame: its frames 0-14 at 15 a second are the ramp, so a comparison of early frames with an
  older capture needs the hook. **A script's 120 replaces it** (#390, `script_fades`): e34's, E01's,
  e13's, e23's and m07_a's, at the script's start behind the cover, are the only fade (the level's
  never begins); m09_b_startup's, a frame after the cover, restarts it; m09_x_intro's is the same
  1.0 s, one step later: its first frame in sight is black (k = 0, a script's fade; the level's own
  is one step in), the ramp as the level's from the 15th frame on ([cutscenes.md](cutscenes.md)
  "The screen fade").
* **The gaps and fades the takes show beside it, and what the demo does with them.**
  * **Between briefing movies** (take01: 0.35 s and 0.69 s): **not reproduced**, see
    [briefing.md](briefing.md) "The gaps between movies": no wait in the code, none after a skip
    (take02), and the takes' own picture-to-sound drift makes up most of the rest.
  * **After an end screen's Continue** (take12: the end screen until 413.58 s, black 413.60-413.72,
    mb02's first frames 413.73): **not reproduced**: the level's picture is gone as the session
    ends it and the movie's first frame comes after its open, as at the other movie starts (no
    wait in the code, [briefing.md](briefing.md)). FUN_00085200 ends by starting a 1 s fade to
    black (FUN_00069780(1.0) at 0x85304, ECX 1; Replay Mission's FUN_00084da0 and Replay Chapter's
    FUN_00084e80 start one of their own, Quit's FUN_00084f50 none), and only Clone Squad's is
    ever seen: take12's Continue and take10's Quit cut from the picture to black in one frame
    (Replay isn't filmed). **Why a fade never shows is open.** FUN_000694f0 cancels a fade while
    the session isn't in play, and by FUN_00085200's code a Continue to a next mission queues
    flags bits 1 and 2 with no bit 0, which 0x7f730's dispatch sends to the level start
    FUN_0007f960, which frees the world at once; only the last mission's Continue and Quit (flags
    `| 9`) take bit 0, 0x7f670. Read from the disassembly, not traced to the picture.
  * **Clone Squad** is the one choice that keeps the level in play (the mode's bit 2, FUN_00089da0
    returns while the fade runs): take05 shows the menu gone at 223.50 s (luma 40.7 to 33.3) and
    the end view then **fading linearly to black, 33.3 to 0.06 by 224.55 s** (a line of 0.98 of the
    picture a second, zero at 224.515 s, rms 0.24 of 33), the new squad out of black from 224.64 s.
    The demo does the first half: the menu hidden and the 1 s fade to black
    (`CHOICE_FADE`, `MissionMode::pending`; the frame's presses are still taken off the game
    meanwhile, so a Backspace doesn't reach `back_to_menu` and leave at once with no quit box),
    then the restart behind LOADING with the level fade.
  * **Before the LOADING ring** the takes show 1.1 s (take01, after mb01) and 2-3 s (take12, after
    the end screen and its movies) of black: the load itself (the last world freed, the
    archives opened: the progress callback draws only once it has had 100 ms of calls). Not
    reproduced.
* **All of FUN_00069780's callers** (13): ECX 0 (from black) in the level start (1 s), the
  reclone (FUN_00089da0, 1 s) and the end screen's opening (FUN_001092e0, 1 s); ECX 1 (to black)
  in the level's end (FUN_000e0410, 0.5 s), the level-over trigger (0x208092, 0.5 s), the end
  choices (FUN_00085200, FUN_00084da0, FUN_00084e80 and two accepts of the message box, 0xfb113
  and 0x10767a: 1 s each), and the script event 120 (0x176216: run by the demo since #390); two
  dead functions (FUN_000888f0, FUN_00088980).

## A mission's end

* **The campaign mode** (FUN_00089a50, vtable 0x3a21fc; made with each mission level,
  FUN_000d6490): its +0x24 flags (bit 0 the level is over, bit 1 the squad is dead, bit 2 a
  restart asked, bit 4 its countdown not run yet) and +0x20 the squad-dead countdown, 3.0 s. Its
  update (FUN_00089c90) runs the countdown (FUN_00089d30): while none of the squad record's
  members lives (FUN_0005baf0), the record's +0x20 has a death counted (a death outside a
  cutscene, the end screen's Operatives Lost) and no cutscene has the world, down by the frame's
  time; at 0 bits 1 and 0. take05: Tex dead at clk 324330, +0x24 48 -> 32 at once, 35 at clk
  327269. The record's members are the demo's squad and the squad types on its side that the
  session counts as members (their member flags set): the V-squad's marines once they've
  spawned, as take11 hands the control to one of them. With Tex dead and a marine alive there's
  no countdown (the demo doesn't hand the control over to a marine: its squad is the four).
* **The level-over trigger** (h_1f07be84, handler 0x207fe0): OPERATE with h_18f66101 (true by
  default) calls the mode's +0x14, its "over" bit: the mission's end (the tutorial's at 8.93 s of
  its ending script, [cutscenes.md](cutscenes.md)). Without it, the zone's end: a 0.5 s fade, the
  session's checkpoint save ("Saving Campaign..." for 1 s, 0x7ed30), then once the fade is done
  (0x207f70) the zone's post-movies and the zone the last **zone-trigger** named (its
  h_14b0089f, 0x207de0): its pre-movies, LOADING, the zone. A false level-over is not a failure:
  across the campaign it fires with a zone-trigger.
* **The world's end step** (FUN_000e0410, every world update): once the mode says over, every
  menu closes, a 0.5 s fade to black starts (FUN_00069780) and the cutscene bit is set (no input,
  no damage, no menus); once the fade is done (FUN_00087d70) the mode's end menu opens. The fade
  darkens the picture's own (gamma) values linearly (its draw, FUN_000694f0: a black quad at
  k x 255). The demo's black is blended in linear light, so its alpha is the one that darkens a
  grey of 0.3 by k in sRGB values (a fit: the end view's mean luma then falls linearly, within
  1%; a power law 1 - (1 - k)^2.2 crushed it to 2% at 0.92, against the take's 9%). take12:
  the luma falls linearly from 39.8 at 357.43 s to 3.56 at 357.90 s, stays there until the end
  screen at 358.37 s (the game saving the results as the screen opens, FUN_00106d10: the last
  frame drawn stays up), then the screen's world fades in from black over 1 s. The demo holds its
  fade at 0.92 for 0.47 s after a successful mission (a fit: the stall isn't in the code); after
  SQUAD KILLED nothing is saved and the screen opens with the fade.

## The end screen (h_e8e14fcf)

Drawn from its data, filled as its controller fills it (vtable 0x39e1cc; FUN_001070b0 by item
name):

* **Title** by its selector (FUN_00107e50): MISSION SUCCESSFUL when every enabled primary is
  complete, else SQUAD KILLED when the squad died, else MISSION FAILED; the A legend with it
  (Continue, Replay Mission, Replay Chapter, Clone Squad).
* **Three pages**, Y going round them (its three button items' buddy-controls): the summary
  ("Objectives n / n", "Cash $x / $y", "Operatives Lost", "DNA Sequences Recovered"), the
  objectives (each enabled one, its cash x the difficulty's scale, "Failed" or "Incomplete"), the
  cash ("Cash Awards" with the entries above 0, "Cash Penalties" with those below).
* **The cash**: x = the penalties + the awards capped at y (FUN_00106860); y = the zone's
  `max-score` (BRUTAL's) back at the difficulty played: 11760 / 1.4 x 0.6 = **5040** on STANDARD
  (FUN_00106b50): take12's and take05's "/ $5040". The numbers count up for 10 s as it opens
  (FUN_00107e90: t / 10 x the greater of a step and the target): take05's "$-1526" is a frame of
  the count to $-3000 (one death: -5000 x 0.6).
* **Operatives Lost**: the squad record's members who died on the level, the joined marines
  among them (FUN_00115d50 counts a death unless a cutscene has the world; take12's 1, Tex in the
  lava).
* The icons one every 0.5 s on success (objectives by the difficulty; cash when all of y was
  earned; DNA). The texts in their styles' disabled colours (FUN_000ef3a0), each after its shadow.
* As it opens (0x106ef0): the status line's "Saving...", "Saving...", "Saving Campaign..." on
  success; the view cut to the level's end camera, the letterbox in at once and a 1 s fade from
  black (FUN_001092e0).
* **Its buttons**: A (or X) while it counts finishes the count; A then goes on (FUN_00085200):
  Continue (the zone's post-movies, the next mission's pre-movies, its first zone; take12: A at
  413.4 s, mb02 from 413.8 s), Clone Squad (the menu goes and the view fades to black for 1 s,
  take05; the demo then restarts the zone behind LOADING: the faithful reclone at the last save
  point is #41's), Replay Chapter / Mission. X asks "Are you
  sure you want to restart the mission?" and B "Are you sure you want to quit?" in the message
  box h_0ec41ef2 (A Accept, B Back; take10 at 209-226 s): Replay Mission, or the main menu on
  CAMPAIGN.

## The status line

The menu system's queue of texts (FUN_000fdd80; 0xff280 / 0xff2c0), each shown for its time in
turn, at game-options' `<console-text>` place (72, 378) in style h_f57d9843 (153 209 251, a
black shadow, the body font at 12 points). It shows over everything in every state: the front
end, a briefing, a map.

## Test hooks

* `BF_MISSION_LOG=1`: the mode's flags, the countdown, the fades, the status line, the end
  screen's figures and choices, what follows; the front end's new campaign; the level start's
  fade ("the LOADING cover is down: a 1 s fade from black begins", then "the fade is done after
  1.00 s of the game's clock (n s of real time)": the real time is the machine's under
  `BF_CAPTURE`).
* `BF_END_PRESS=<a|b|x|y>@<s>[;...]`: the end screen's (and its box's) buttons that long after
  it opened (real time, the capture's steps under `BF_CAPTURE`).
* `BF_MENU_PRESS=<key>@<s>[;...]` (a, b, x, y, up, down, left, right, start): the front end's
  presses from the menu's first frame; with `BF_CAPTURE` each menu frame is `menu_NNN.png` (a
  fixed 1/15 s step a frame) and the run goes on through the briefing (`brief_NNN.png`) into the
  map (`frame_NNN.png`, `BF_CAPTURE_FRAMES` of them, then it quits); `BF_MENU_FRAMES=<n>` quits
  after n menu frames. [menu.md](menu.md)'s `BF_MENU_SCREEN` takes the path's screens too
  (`campaign`, `profile`, `slot`, `name`, `difficulty`, `join`).
* `BF_TEST_DIE=<s>` ([play.md](play.md)) kills the player: the squad-dead path.
* `BF_TRIGGER_FIRE` ([triggers.md](triggers.md)) fires the level-over and zone triggers.
* `BF_NO_FADE=1` (#266): no fade from black as a level starts (`Fade::waiting`), and since #390
  none of a script's 120s either (`Fade::scripts_off`), the picture as it was before the fades.
  Every level's start has it otherwise, behind the LOADING cover, so a capture's first second is
  the ramp. `BF_CAPTURE_FPS=60` shows it a frame at a time (`BF_CAPTURE_FRAMES=60` is its whole
  second).
* `BF_TEST_FADE=<s>,<r>,<g>,<b>,<to|from>,<secs>[;...]` (#266): screen fades of any colour, way
  and time (FUN_00069780's arguments, `Fade::start_colour`) started at those seconds of the game's
  clock (a time of 0 is one second: a script's `dur=0`); with `BF_MISSION_LOG` each is printed as
  it starts. Take04's whiteout is `0.2,255,255,255,to,0.5;0.7,255,255,255,from,0.5`, take03's red
  `11,191,0,0,to,1`, e34's 2.0 s from black `0,0,0,0,from,2` (it replaces the level's 1 s one).
  The scripts' code 120 calls the same (#390).

## Verified

`BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps, the logs read and the frames set side by side
with the takes':

* **The path** (the first command above): CAMPAIGN TYPE, SELECT PROFILE, SELECT CAMPAIGN, ENTER
  NAME, SELECT DIFFICULTY and JOIN CAMPAIGN before and after A, against take01 at 8.6, 11.6, 17.0,
  20.0, 22.4, 31.5 and 34.5 s: the panels, titles, headings, buttons, descriptions, rows, keys,
  quadrants, pictures and legends at the take's places (the keyboard's keys, the quadrants' lines
  and Tex's picture to a unit); the take lists its saved CAMP001 and QUICK001 (the demo has none)
  and types its titles a little slower. Then "Saving..." / "Saving Campaign..." / "Saving..."
  over Campaign_Intro, the three movies, LOADING, the tutorial (the mission h_e26b9b3e, zone 0).
  At HARD (`down` on SELECT DIFFICULTY): `campaign/difficulty-setting -4 -> 0`.
* **The ending** (take12): the fade, MISSION SUCCESSFUL over the end camera, the pages and their
  lines at the take's places; Continue: mb02, Brutus_Intro_Final, LOADING, m01_a.
* **The squad's death** (take05, take10): the 3 s countdown, SQUAD KILLED with Objectives 0 / 2,
  Cash counting to $-3000 / $5040, Operatives Lost 1, "No DNA Found", its pages; Clone Squad: the
  tutorial again behind LOADING. B and X open the box.
* **The end view's colours** (#250): the end screen neither dims nor tints the world. As it opens
  FUN_001092e0 only cuts the view to the end camera, brings the letterbox in at once and fades
  from black over 1 s (FUN_00069780); the menu h_e8e14fcf has no full-screen picture (its three
  pictures are the 32-unit icons), and its texts and the title's fill are its only other
  drawing. The tester's lower-left patch ((0, 300)-(100, 380), the demo's (83, 78, 72) against
  take05's (58, 51, 22)) isn't the same picture: the takes' end view has a grass mound with
  yellow plants in its lower-left foreground where the demo's shows the lava pool, its embers
  and a dark rock (`BF_MAP=tutorial BF_TEST_GOTO=57.5,68.22,57.5,68.22 BF_TEST_DIE=3`, frame 140
  of a 15 fps capture, against take05 at 192 s: the far fence, cabin, tower and sandbags line up,
  the camera at take12's (-12.82, -5.63)). Where the two show the same thing the demo's world is
  bluer: the middle ground (57, 69, 77) against take05's (68, 68, 63) and take12's (56, 63, 60),
  the right-hand fence (39, 45, 72) against (38, 38, 51): the level's lighting, #181's.
* **A zone's end** (m01_a, its zone-trigger h_fd436c00 and false level-over h_ffc73ff3 fired):
  the fade, "Saving Campaign...", m01_b (zone 1 of the mission).
* **The level's start fade** (#266; `BF_MAP=tutorial BF_CAPTURE_FPS=60 BF_CAPTURE_FRAMES=100`
  with and without `BF_NO_FADE=1`, the mean luma of each frame over the other's): the ratio is the
  line k = (frame + 1) / 60 to 0.013 (rms 0.0055; fitted 0.998 x t + 0.013, 1 at 0.989 s), frame 0
  at 0.017, frame 59 at 1.000; against take01's `luma / plateau` at the same levels (k 0.1, 0.25,
  0.5, 0.75, 1.0: the demo's frames 5, 14, 29, 44, 59 beside take01 at 269.983, 270.000, 270.183,
  270.467, 270.733 s) the same shot, the gate wall's, the sentry, the cliff and the sky, rising
  from black, the letterbox's bars in both. The fade's end on, the frames are the no-fade run's
  pixel for pixel; the title card's phases are the same (see above). With `BF_CAPTURE_EARLY=1`
  the cover's frames 0-23 (the ring), then the ramp from frame 24 (2.1 of the plateau's 47.1), 30
  frames to the plateau at 30 a second: no unfaded frame between. Through the front end's
  campaign path (the first command, `BF_CAPTURE_FPS=30`): the three movies skipped, LOADING, the
  tutorial's ramp 2.1 to 47.1 in 30 frames. sdm_e34 straight away: 2.4 to 54.4 in 30 frames. The
  tutorial's ending and Continue (`BF_END_PRESS="a@1;a@3"`, the objectives fired as pickups.md
  does): the level's 0.5 s fade and its stall, the end screen's 1 s fade from black, the end screen,
  mb02 and Brutus_Intro_Final skipped, m01_a's ramp at 15 a second 3.3 to 57.9 in 15 frames.
  **m01_a** (`BF_MAP=m01_a`, take12 at 579.38-580.33 s): the same shot (the walkway's pipe and
  the jungle) fading in at matching levels; the demo's mean luma against its no-fade run is below
  the line in the first half (0.12 at k 0.18, 0.44 at 0.50, 0.79 at 0.82, 0.979 at 0.98; at most
  0.075 below, rms 0.044): the linear-light black's known error on a dark picture (`GREY` 0.30 is
  the tutorial's end view's fit). Computed on the demo's own frames, the largest error of the
  mean luma ratio is -0.07 (where the picture is at 30% of its brightness) with GREY 0.30 on this
  picture and +0.033 with 0.15, and 0.009 on the tutorial's with 0.30: an exact multiply in the
  game's gamma values isn't a blend of the linear light, and the best GREY follows the picture's
  mean (about 1.6 x its mean luma). The frames 0-12 at 60 a second show the HUD fading with the
  letterbox's bars coming in (menu.md's "not fixed" on m01_a): that is the game's own, the view's
  amount is 0.20 at clk 100 ms and 0.50 at 252 (#232's take02) and take12's picture has the bars
  coming in through its fade too (above); the take's first lit frame is already at clk about
  0.18 s with the bars at 0.18 and the HUD at 0.64 alpha, hardly lit under the fade, so it
  never shows it. Clone Squad (`BF_END_PRESS="a@0.5;a@1.0"`, 30 a second): the menu hidden at the
  frame after A (luma 52.5 to 47.0), then 47.0 to 0.0 in 30 frames, 0.0 for two frames, the
  restarted tutorial's ramp 2.3 to 56.7 in 30 frames (take05: 33.3 to 0.06 in 1.05 s with the
  menu gone, black, the new squad's fade 1.08 s). The editor (`--edit`) starts at its full
  brightness (78.0 in the frames checked): no fade in edit mode.
* **Regressions**: the main menu, SELECT MISSION and a deathmatch map from it (`BF_MENU_GO`),
  sdm_e34 straight away, the editor (`--edit`), `BF_MOVIES=m00` with its subtitles. Unit tests
  (`cargo test --bin bf_play`): the end screen's count, the keyboard's layout, the records'
  widths, and (#266) the level start's, the choice's and a colour's fades, the quad's blend for
  black, white and red, and BF_TEST_FADE's list.

## Not done, or guessed

* Saving (#27): nothing is written; the profiles' settings (INVERTED), New Profile, deletes,
  QUICK CAMPAIGN and RESUME LAST give the error sound. The front end's save stalls (half a
  second after DONE and after START) aren't reproduced; the end screen's is, as a fitted wait.
* Clone Squad restarts the zone: the flags carry on as they are, the cash and Operatives Lost
  start again with the level's new record. The game reclones the squad at its last save point
  in the same level, its record kept (#41), with no LOADING screen and black for 0.05 s between
  the two fades (take05); the demo's stand-in shows LOADING and the cover in between.
* The 1 s fade an end screen choice starts (FUN_00085200 -> FUN_00069780(1.0) at 0x85304) is run
  for Clone Squad only, as the game shows it (#266, "A level's start"). take12's Continue and
  take10's Quit never show one (take12 shows the end screen until 413.58 s, A at 413.4 s, then
  black at once, mb02's first frame at 413.73 s and the movie's own fade-in over 0.5 s; Replay
  isn't filmed), and the demo cuts to the movie (black, fading in) the frame after A. Why is
  open ("A level's start"). The 0.17-0.24 s between a press
  and the picture changing in the takes (A at 413.408 s, the picture gone at 413.60 s; take10 A at
  226.43, gone at 226.67 s) is the console's input and display latency, not reproduced.
* The level start's first 0.5 s of black (the first update's long frames, above) and the picture's
  lag behind the clock in the takes' first lit frames aren't reproduced; nor is the black before
  LOADING's ring (1.1-3 s after the last movie), the load's.
* **The fade's blend** is linear-light (`GREY` 0.30), a fit; #379 (the stored-value blend) is
  the fix. What the tester measured of it, beside the m01_a figure above (the mean luma up to 0.07
  under the line at mid-ramp):
  * the black fade is not a uniform multiply as the game's is (take01's every pixel is the same
    fraction of itself): per pixel the ratio depends on the pixel's level, at k 0.25 0.13 of a
    dark pixel against 0.32 of a bright one;
  * a fade to a colour is fitted for a darker grey (`GREY_COLOUR` 0.12); the "within 0.04 (white)
    and 4.1 of 255 (red)" above are whole-frame means. Measured apart, the picture's red lags the
    game's blend by up to 9.9 (rms 5.1), and the letterbox bars lead it: by up to 0.139 of
    progress in the white fade (35 of 255) and up to 22 red in the red fade (a black pixel isn't
    in the fit); take04's bars track its picture;
  * the takes' first lit frame is already at k 0.05-0.17, a visible ramp of about 0.85-0.9 s; the
    demo's starts at one frame's k and runs the whole second.
  An exact gamma-space multiply with alpha blending is impossible, and a shader pass would
  darken the menus drawn over the fade (the end screen, the panels, the status line) as well.
* The scripts' fades (code 120: 0x1761e5) run since #390, through `Fade::start_script`
  ([cutscenes.md](cutscenes.md) "The screen fade"). On m09_b_startup's own red fade (the real
  script, the letterbox in) the blend's errors against the game's `(1 - a) v + a c`, on the mean
  over the bars and over the rest of the frame: the bars' R up to +18 early (k 0.13-0.27), the
  rest's R up to -22 late (k 0.27-0.33), G and B within 3.7 (a single bright pixel 25 off); the
  black fade-ins of e34 and m09_b up to 0.023 and 0.059 under the line of the mean luma; the
  white-out within 0.025 of the line. `BF_TEST_FADE` still tries colours and times without a
  script.
* **A fade's first frame.** The game draws every FUN_00069780 fade at k = 0 in its first frame:
  the call stores the app clock and the same step's draw reads it (FUN_00068d20 adds the step to
  the clock, runs the update, then FUN_00069390 draws). The demo does this for a script's 120
  only (`Fade::start_script`, #390: e34's first frame in sight is black, then one frame's k).
  The end fade (`END_FADE`), the zone end's (`ZONE_FADE`), an end screen choice's
  (`CHOICE_FADE`), the end screen's opening fade, `BF_TEST_FADE`'s and the level's start
  (`LEVEL_FADE`) take the clock's step in the frame they start, so their first frame is one step
  in: up to 1/15 s at a capture's step, 1/60 s in play. The level's start one was left so by
  #266 (one frame's k at a capture's 15 fps, 0.067, is inside the takes' first lit frames at k
  0.05-0.17, above); moving the others to k = 0 is not done, and no take shows one at a frame's
  resolution.
* The DNA line names the DNA canister the squad took on the level (#228: "Confed Marine DNA" and
  its icon, as take12; [pickups.md](pickups.md), "DNA canisters"), not the save's
  campaign/save-game/dna-archetype, which isn't kept (#27); the count's ticking and the
  icons' sounds aren't played; the end camera is the level's first (the game picks one with its
  random state); the mission's score isn't written to the flags; the music isn't stopped at the
  countdown (FUN_00179e10).
* Fits from take01: the keyboard's slots centred in the item, the keys 4 units smaller than their
  slots, the name's lavender box (199 199 254) and the selected key's pulse (1 s; the code isn't
  traced); the quadrants' borders for their missing texture; the JOIN profile name in its normal
  colour (a static text elsewhere takes its disabled one).
* The menu path has no mouse: the keys and the pad drive it.
