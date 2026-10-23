# Mission flow: from CAMPAIGN to the end screen and the next mission (#226)

Step 7 of the tutorial (#28): a new campaign started from the front end's CAMPAIGN, a mission's
end (its level-over trigger, or the squad's death), the end screen (MISSION SUCCESSFUL, SQUAD
KILLED), and what follows it: the next mission's movies and zone, the zone again, the front end.

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
  413.4 s, mb02 from 413.8 s), Clone Squad (the demo restarts the zone behind LOADING: the
  faithful reclone at the last save point is #41's), Replay Chapter / Mission. X asks "Are you
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
  screen's figures and choices, what follows; the front end's new campaign.
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
* **A zone's end** (m01_a, its zone-trigger h_fd436c00 and false level-over h_ffc73ff3 fired):
  the fade, "Saving Campaign...", m01_b (zone 1 of the mission).
* **Regressions**: the main menu, SELECT MISSION and a deathmatch map from it (`BF_MENU_GO`),
  sdm_e34 straight away, the editor (`--edit`), `BF_MOVIES=m00` with its subtitles. Unit tests
  (`cargo test --bin bf_play`): the end screen's count, the keyboard's layout, the records'
  widths.

## Not done, or guessed

* Saving (#27): nothing is written; the profiles' settings (INVERTED), New Profile, deletes,
  QUICK CAMPAIGN and RESUME LAST give the error sound. The front end's save stalls (half a
  second after DONE and after START) aren't reproduced; the end screen's is, as a fitted wait.
* Clone Squad restarts the zone: the flags carry on as they are, the cash and Operatives Lost
  start again with the level's new record. The game reclones the squad at its last save point
  in the same level, its record kept (#41).
* The 1 s fade an end screen choice starts (FUN_00085200 -> FUN_00069780(1.0) at 0x852ea) isn't
  run: take12 shows the end screen until 413.57 s (A at about 413.4 s), then black at once,
  mb02's first frame at 413.77 s and the movie's own fade-in over 0.5 s; the demo cuts to the
  movie (black, fading in) the frame after A.
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
