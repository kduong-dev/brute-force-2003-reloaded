# Map menu

`cargo run --bin bf_play` with no `BF_MAP` opens the front end, rebuilt after the game's own (the
capture in `todo/`). It's laid out on the game's 640 × 480 screen and scaled to the window.

* **Title.** START (DEMOS is shown but not here).
* **Main menu.** CAMPAIGN, MULTIPLAYER, EDITOR, OPTIONS, CREDITS, opening on CAMPAIGN (take01 at
  2 s), with the selected word white over a faint, larger echo, and its description beside the
  logo. CAMPAIGN leads to the campaign path, MULTIPLAYER to a submenu of DEATHMATCH and SQUAD
  DEATHMATCH (below), EDITOR to the map editor, CREDITS to the credits; OPTIONS isn't in the demo
  (the error sound). **This is not the game's list.** The game's main menu is CAMPAIGN,
  DEATHMATCH, SQUAD DEATHMATCH, OPTIONS, CREDITS (game-options `h_f228042f`, take01), with
  DEATHMATCH and SQUAD DEATHMATCH leading to its mode panels. The demo departs from it twice, both
  the product owner's choices: it groups the two deathmatch modes under MULTIPLAYER (#310), and it
  adds EDITOR (#193). Both are the demo's own entries, below.
* **Layout** (`MAIN_BOXES`): the words are right-aligned in boxes ending at x 279, centred in
  them. The game's boxes are 100, 160, 220 (SQUAD DEATHMATCH, two lines, 80 tall), 320 and 380
  (OPTIONS, CREDITS) in y, a one-line entry's 40 tall and 60 from the next. The demo's five
  entries are all one line, so they take **that 60 pitch from the game's first top, evenly: 100,
  160, 220, 280, 340**. CAMPAIGN, MULTIPLAYER (where DEATHMATCH was) and EDITOR (where SQUAD
  DEATHMATCH began) are at the game's tops. OPTIONS and CREDITS are at 280 and 340, not the game's
  320 and 380: the 20-unit gap that SQUAD DEATHMATCH's two-line box left before them has no
  reason to stay, and keeping 320 and 380 would leave the five unevenly spaced (or start CAMPAIGN
  at 140, off the game's first top). CREDITS' box ends at 380, 100 above the screen's bottom edge;
  #193 had pushed it to 440-480, where its echo (1.7 times the word, centred on it) was clipped
  by the edge.
* **MULTIPLAYER** (#310, `play_menu.rs` `MULTIPLAYER`, `play_multiplayer_menu.rs`): the demo's
  own entry, not the game's (see "Main menu"). Its word and its description line ("Play a
  deathmatch in an arena, or lead your squad in a squad deathmatch.") are the demo's text
  (`Say::Demo`), drawn in the game's font, colours, outline, layout and animations as the others
  are, with their sounds (the game's own string "Multiplayer", `h_eb279513`, sits among its
  debug-config labels: Debug Camera, Look Pitch). It opens a submenu drawn as the game's own
  sub-menus are, made from **CAMPAIGN TYPE's items** (game-options `h_f9728897`, the menu the
  game's CAMPAIGN entry opens; the same helpers draw both, `play_campaign_menus.rs`):
  * the panel (`h_ff6b96da`), the typed title (484 × 38 at (90, 50), style `h_189b1ae9`), the
    group's first two buttons (200 × 20 at (80, 152) and (80, 180), style `h_ea5acf71`:
    `color-normal`, the selected one in `color-selected` with its echo), each button's
    description in the box at (314, 118), 216 × 254 (style `h_15405823`, shown with its button:
    its buddy-control), and the A SELECT and B BACK legends. Places, styles and animations are
    the data's (the buttons slide in, the texts fade in, the title types out, the selected button
    echoes: as CAMPAIGN TYPE's). The sounds are the ones CAMPAIGN TYPE's code plays: the first
    button's slide as the screen opens, the move sound as the selection moves, the error sound at
    the ends, select and open as a map list opens.
  * The demo's choices, all the words and what's left out: the title says MULTIPLAYER; the two
    buttons are **DEATHMATCH and SQUAD DEATHMATCH**, in the game's words: the first buttons of
    its own DEATHMATCH and SQUAD DEATHMATCH panels (`h_f4475e09`, `h_0e51acdd`, one line each as
    CAMPAIGN TYPE's buttons are: the main menu's `h_18653c47` has a line break for its two-line
    box), with the descriptions the game's main menu gives the two entries (`h_163f7c9d`,
    `h_1e9ff6eb`; their line breaks, made for the main menu's 270-unit box, are made spaces so the
    216-unit box wraps them). No RESUME LAST, and no System Link group (no saves, no network
    play).
  * **No heading band** (the product owner's choice). CAMPAIGN TYPE's two bands, SINGLE XBOX
    CONSOLE (`h_1cc9f852`, at (70, 118)) and SYSTEM LINK (`h_0e49a355`, at (70, 246)), are both
    left out of the submenu: the descriptions shown with the buttons are the game's main-menu
    texts, and they mention System Link ("Play head-to-head, up to 4 players on one Xbox
    console, or up to 8 players over System Link"), which a SINGLE XBOX CONSOLE band over them
    would contradict. The buttons stay where CAMPAIGN TYPE's data puts them, (80, 152) and (80,
    180), not moved up into the band's place, so the panel is bare above them.
  * A goes on to SELECT MISSION for the mode (the game's mode panel, split-screen / System Link,
    is still skipped, as it was from the main menu). Up and down move, B (Esc, Backspace) goes
    back to the main menu on MULTIPLAYER, with no sound of its own. SELECT MISSION's B goes back
    to the submenu **on that mode**; in a map, Backspace still comes back to its SELECT MISSION,
    so B, B walks on to the submenu and the main menu on MULTIPLAYER. A campaign
    mission's Backspace goes to the main menu on CAMPAIGN instead (the pause menu's Quit's place,
    #312: the same `menu::Return` as the end menu's Quit). The submenu opens on
    DEATHMATCH every time it's entered from the main menu (the product owner's choice; CAMPAIGN
    TYPE too opens on its first button). Coming back from SELECT MISSION it's on that mode.
  * The mouse works here as on the main menu (CAMPAIGN TYPE's screens take none): each button has
    an invisible target on its data box; pointing at it selects it (the move sound) and a click
    selects it and goes on. The pad's d-pad, A and B and START (which selects) are the others'.
* **EDITOR** (#193, `play_menu.rs` `EDITOR`): the demo's own entry, not the game's. Its word
  and its description line ("Build your own map on the test level: place buildings, objects and
  characters, then play it with your squad.") are the demo's text, drawn in the game's font,
  colours, outline, layout and animations as the others are, with their sounds (moving onto it,
  selecting it). It's the third entry, at 220 (see Layout).
  Choosing it shows the LOADING screen while the flat test map loads on a thread with the test
  map's data (`testmap::load_weapon_data`, as `--test` loads it), then opens the map editor on it
  in edit mode, as `--edit` does ([editor.md](editor.md), "From the front end"). Leaving the
  editor (Backspace twice, in edit mode or in play) comes back to the main menu on EDITOR.
* **The campaign path** (#226, `play_campaign_menus.rs`): CAMPAIGN TYPE, SELECT PROFILE, SELECT
  CAMPAIGN, ENTER NAME, SELECT DIFFICULTY and JOIN CAMPAIGN, drawn from their game-options menus
  in the game's font records with their own animations; START on JOIN CAMPAIGN starts a new
  campaign on the tutorial at the difficulty chosen (its movies, LOADING, the zone). Each screen,
  what it draws and what's stubbed (profiles and saves are #27): [mission.md](mission.md).
* **SELECT MISSION.** A carousel of the mode's maps (previous, current outlined, next) with the
  map's name, planet and description. The level list (`common/campaign-bf.xmb`) gives each map
  its kind (`Type`): `h_120785ef` deathmatch (the 7 arenas, mp*), `h_efb18d28` squad deathmatch
  (the 7 sdm_* maps); missions are `h_17506391`.
* **Playing.** One window throughout. bf_play's app has three states: `Menu`, `Loading` and
  `Playing`. Choosing a map shows the LOADING screen while a thread loads the game data, the
  level and its collision; the map then plays in the same window. Deathmatch plays alone, without
  music and without the radar (the squad's portraits, blips and health channels), as in a capture. In the
  game, Backspace removes everything the map spawned (whatever wasn't there when it began),
  and the menu comes back on SELECT MISSION with that map selected. Each map installs its own
  collision (`arena::Arena::install`, replacing the last one).
* **Controls.** Arrows / WASD / d-pad move, Enter / Space / A select, Esc / Backspace / B back
  (Esc at the title quits); the pad's START selects too, but on JOIN CAMPAIGN, where it begins
  (Enter or Space a second time does on the keyboard). The mouse points and clicks (on the main
  menu, the MULTIPLAYER submenu and SELECT MISSION's carousel; not on the campaign path's screens).

All of it is the game's but the demo's own entries (MULTIPLAYER, EDITOR) and their words. The
front end itself is defined by `common/game-options-en.xmb`: its
menus, item positions, menu-style colours and fonts, music player and `global-sounds` (menu
event -> sound). The menu code in default.xbe sends the events: 0x1038e0, a control
stepping its own value by one (the map carousel scrolling), sends `h_ebf26601` (-> `h_1dffc5a1`),
`h_f06bc0ab` (A selected) or `menu_error`; 0x100250, the focus moving to the previous / next item
(through 0xffb60's neighbours), sends `h_e29fd995` (-> `h_ed2b5e99`); a menu opening sends
`h_f97ceaf0`. (`media/Sound.xsb` +
`Wave.xwb`, with cues like DownloadComplete and CardPut, belong to the content-download UI.)

| What | From |
|---|---|
| Words and descriptions | the string table (`SELECT MISSION` h_e7b4fd5a, `Planet: %s` h_fa5634c8, …) |
| Heading font | atlas h_e0afcd52 (the styles' `heading` font; rows in ASCII order) |
| Body font | atlas h_e4e4d2f4 (glyphs from `!` on in reading order, two sizes; the first is used): `!` to `~`, boxes for the codes without a glyph, then Latin-1's `¡` to `ÿ` (counted back from the last) |
| Copyright | the title's static text `h_e3603ae4` at (50, 418), 542 × 20, centred, style h_e76e1577: 153 209 251 at alpha 200, black outline, body font 12 pt |
| Button icons | atlas h_0b630034 (A, B, X, Y, L, R row) |
| Colours | game-options' menu-style: words normal 30 110 150, selected 215 236 251; descriptions 153 209 251; every style's outline (`h_ed70ff4f`) black, drawn as a 1-unit outline plus drop shadow |
| SELECT MISSION | the panel h_e462c760 (7 splash_screen textures at alpha 180, at (66, 50)); the carousel's arrow h_158f87c1 stretched to 20 × 128 (the right one mirrored); title 153 209 251 in the heading font, no outline, typed; map name 153 209 251 (body font 16 pt); recommended players (dm-data `h_0398b8bf` min/max, `Recommended for %d - %d players.`; none on the squad deathmatch maps), planet and description 148 198 149 (14 pt) |
| Loading screen | game-options' LOADING menu (`h_f6551df5`): a 256 × 256 ring at (192, 112) on black, four mirrored quarters of `h_06477be9` or `h_1ec0f086` (the ring with its big blocks on the diagonals / on the axes), shown in turn every 0.1 s (a capture), "LOADING" (`h_0f28682d`) in the heading font, no movie or music. Shown while the map loads |
| Layout | game-options' item positions: words right-aligned in boxes ending at x 279 (main menu tops 100, 160, 220 (80 tall), then 320, 380 in the game; the demo's five one-line entries at 100, 160, 220, 280, 340: see Layout above); logo at its own 512 × 256 (title (194, 106), main (194, 18)); description box (312, 240) |
| MULTIPLAYER submenu | CAMPAIGN TYPE's items (game-options `h_f9728897`: panel `h_ff6b96da`, typed title, the first two buttons and their descriptions, A / B legends; not its two heading bands), their places, styles, animations and sounds; the demo's own words for the title and the main menu's entry (`Say::Demo`), the game's for DEATHMATCH (`h_f4475e09`), SQUAD DEATHMATCH (`h_0e51acdd`) and their descriptions (`h_163f7c9d`, `h_1e9ff6eb`) |
| Animations | translation-anim: items slide in from y 240 (0.25 s); color-anim: fade in (0.5 s); select-anim: the selected word's echo grows (from the word's middle) to 1.7× and fades from alpha 150 in 50 185 250 (0.5 s), then pulsing every 0.3 s while the word stays selected or hovered; the logo slides up from the title's place into the main menu; panel titles type out (0.125 s a letter); their sounds `h_e8ace091` / `h_e1abd007` (splash_screen) |
| Logo, title bar | splash_screen textures h_f1e0ac39, h_190a4ea7 |
| Previews | the level list's `texture-name` (splash_screen) |
| Music | `menu_dub1` in `ml-sounds/en/splash_screen-en.tgz` (the music player's sound `h_fa71a2f8`; `sounds/splash_screen.xwb` is an empty stub) |
| Menu sounds | game-options' `global-sounds` (sounds-common): moving between words `h_ed2b5e99` (heard at both moves in a capture), carousel scrolling `h_1dffc5a1` (the code's value-stepping event, above), select `h_e6b8bf54`, can't `h_e47668bc`, menu opens `h_f3a5b12b`. Going back has no sound of its own: as in a capture, the menu it returns to plays its opening sounds (item slide `h_e8ace091`; the main menu also its logo slide `h_e1abd007`, the logo sliding down from the title's place each time) |
| Background | `data/movies/menuBack.bik` |

Bink can't be decoded here, so the background plays from JPEG frames made once with ffmpeg:

```
ffmpeg -i "Brute Force/data/movies/menuBack.bik" -vf fps=15 -q:v 4 decompiled/movies/menuBack/%04d.jpg
```

(571 frames, 8.6 MB; without them the menu shows on black.)

Starting up, as in a capture: the window opens at once on the boot screen, a dim "B" emblem with
"Loading" over it at the screen's bottom left (44, 364), while the game data is read on a thread.
The emblem pulses every 2 s, measured frame by frame from a capture (against its brightest: 0.29,
down to 0.12, up to 1 and held 0.9 s, back to 0.3), and fades out as the loading ends. Then the opening movies play: the Microsoft Game Studios
logo, the Digital Anvil logo and the intro (`data/movies/MGS_Logo_Final.bik`, `DA_Logo_Final1.bik`,
`Intro_Montage.bik`). Any key, click or pad button skips the one playing; after the last, the
title. The boot screen is drawn by the game before any data is read and isn't in the data files or
the XBE's images (`$$XTIMAGE` is the dashboard's title picture, `$$XSIMAGE` the save icon), so
`decompiled/boot/loading_emblem.png` and `loading_text.png` are cut from a capture (2560 x 1440,
the 640 x 480 screen at 3x, scaled back to 1:1 and averaged over the emblem's brightest frames;
greys as colours, opaque). The movies are converted once. Their sound is in several tracks (as Bink on the Xbox
plays them in surround): 0 front left / right, 1 a mono low channel, 2 the rear pair, and from 3
on the centre, one track per language (the narration with the centre's share of the music: five
on the intro, nearly alike but for the voice; 3 is English: the only one Windows' English speech
recogniser follows, and the Xbox's language order). default.xbe's movie player opens tracks 0, 1,
2 and the language's (FUN_001781c0; FUN_00178090: English 3) and sends them to the front pair, the
low channel, the rear pair and the centre. They are mixed down to stereo, the centre and
rears at -3 dB, the low channel at -6 dB, under a limiter (track 0 alone left out the intro's
narration and most of the MGS logo's sound). The credits movie has one stereo track. The tracks
are taken by their Bink track id (`0:i:N`): these movies list them in that order, but some of the
campaign's don't (briefing.md, which converts the campaign's briefing movies the same way).

```
mix='[0:i:0]aresample=44100,aformat=channel_layouts=stereo[f];[0:i:3]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[c];[0:i:1]aresample=44100,aformat=channel_layouts=stereo,volume=0.5[e];[0:i:2]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[s];[f][c][e][s]amix=inputs=4:normalize=0,alimiter=limit=0.95:level=0[out]'
for m in MGS_Logo_Final DA_Logo_Final1 Intro_Montage credits; do
  ffmpeg -i "Brute Force/data/movies/$m.bik" -vf fps=30 -q:v 5 decompiled/movies/$m/%04d.jpg
done
for m in MGS_Logo_Final DA_Logo_Final1 Intro_Montage; do
  ffmpeg -i "Brute Force/data/movies/$m.bik" -filter_complex "$mix" -map "[out]" -ac 2 -ar 44100 decompiled/movies/$m/audio.wav
done
ffmpeg -i "Brute Force/data/movies/credits.bik" -map 0:a:0 -ac 2 -ar 44100 decompiled/movies/credits/audio.wav
```

(10 s, 15.7 s, 67 s and the credits 150 s, 163 MB; a movie without frames is left out. The
demo finds `decompiled/` beside `viewer/`, in the working folder, or, with `BF_DATA_DIR` set as a
git worktree needs it, beside the `Brute Force` folder.) The main
menu's CREDITS plays the credits straight away, their frames read from disk as they're shown (any
key, click or button stops them) and comes back to the main menu on CREDITS. Movies played on their own fit
inside the window, whole (the menu's background covers it). `BF_MENU_SCREEN=credits` picks CREDITS. `BF_NO_INTRO=1` skips the movies;
the menu screenshot hooks skip them too and count frames from the menu, unless `BF_SHOT_EARLY=1`
(frames then count from the start: the boot screen, the movies).

When the menu is skipped: whenever `BF_MAP` or a test hook (`BF_CAPTURE`, `BF_TEST_GOTO`, …) is
set, or with `BF_NO_MENU=1` (`BF_MENU_PRESS` without `BF_MAP` shows it, with `BF_CAPTURE` too).
Test hooks: `BF_MENU_SHOT=<file.png>` saves the menu and quits;
`BF_MENU_SCREEN=title|main|multiplayer|dm|sdm|loading|editor` (or the campaign path's
`campaign|profile|slot|name|difficulty|join`) with `BF_MENU_PICK=<n>` picks the screen and entry:
`main`, the main menu: 0 CAMPAIGN, 1 MULTIPLAYER, 2 EDITOR, 3 OPTIONS, 4 CREDITS (**#310 moved
these**: it was 1 DEATHMATCH, 2 SQUAD DEATHMATCH, 3 EDITOR, 4 OPTIONS, 5 CREDITS); `multiplayer`,
the MULTIPLAYER submenu: 0 DEATHMATCH, 1 SQUAD DEATHMATCH; `dm` and `sdm`, SELECT MISSION of the
deathmatch arenas / the squad deathmatch maps (as before #310), the pick being the map;
`editor` is the main menu on EDITOR;
`BF_MENU_SHOT_FRAME=<n>` takes the shot at frame n (screenshots step a fixed 1/60 s a frame, to
catch the animations); `BF_MENU_GO=1` plays the picked map (on `multiplayer`: the picked mode's
first map), or on EDITOR opens the map editor (the screenshot hook runs on into it);
`BF_TEST_BACK=<n>` goes back to the menu after n frames of a map
(of the map editor too; under `BF_CAPTURE` the first 60 frames of a run are its warm-up, uncaptured).
`BF_MENU_PRESS=<key>@<s>[;...]` presses keys (`a`, `b`, `x`, `y`, `up`, `down`, `left`, `right`,
`start`) that many seconds after the menu first opens, one a frame (the opening movies are
skipped); with `BF_CAPTURE=<dir>` each menu frame is saved as `menu_NNN.png`, a fixed 1/15 s a
frame, and the run goes on through a briefing (`brief_NNN.png`) into the map (`frame_NNN.png`,
`BF_CAPTURE_FRAMES` of them, then it quits); `BF_MENU_FRAMES=<n>` quits after n menu frames.

```sh
# the campaign path to JOIN CAMPAIGN on take01's timeline, 470 menu frames
BF_MENU_PRESS="a@0.5;a@2.0;a@4.7;a@7.3;a@12.5;a@15.7;a@18.3;a@27.8" BF_MENU_FRAMES=470 BF_CAPTURE=cap \
  BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# EDITOR (two downs), the map editor, back to the menu after 130 frames, then MULTIPLAYER >
# DEATHMATCH's first arena (the presses' clock runs in the menu and LOADING only, and the
# editor's LOADING takes about 11 s of it: the second set waits it out)
BF_MENU_PRESS="a@0.5;down@1.0;down@1.5;a@2.5;up@22;a@23;a@24;a@25.5" BF_TEST_BACK=130 \
  BF_CAPTURE=cap BF_CAPTURE_FRAMES=140 BF_MENU_FRAMES=700 BF_TOOLS_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# a screenshot in the editor, opened from the menu's EDITOR
BF_MENU_SCREEN=editor BF_MENU_GO=1 BF_MENU_SHOT=editor.png BF_MENU_SHOT_FRAME=900 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# MULTIPLAYER > DEATHMATCH > the first arena (mp1), Backspace out of it after 70 frames, then B
# back through SELECT MISSION, the submenu and the main menu to the title
BF_MENU_PRESS="a@0.5;down@1.5;a@2.5;a@3.5;a@4.5;b@12;b@13;b@14" BF_TEST_BACK=70 BF_MENU_FRAMES=260 \
  BF_CAPTURE=cap BF_CAPTURE_FRAMES=100 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the same through SQUAD DEATHMATCH (one more down in the submenu; its first map is sdm_e40)
BF_MENU_PRESS="a@0.5;down@1.5;a@2.5;down@3.5;a@4.5;a@5.5;b@13;b@14;b@15" BF_TEST_BACK=70 BF_MENU_FRAMES=300 \
  BF_CAPTURE=cap BF_CAPTURE_FRAMES=100 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# screenshots: the submenu on SQUAD DEATHMATCH; its first map straight away (BF_MENU_GO)
BF_MENU_SCREEN=multiplayer BF_MENU_PICK=1 BF_MENU_SHOT=sub.png BF_MENU_SHOT_FRAME=120 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
BF_MENU_SCREEN=multiplayer BF_MENU_PICK=1 BF_MENU_GO=1 BF_MENU_SHOT=map.png BF_MENU_SHOT_FRAME=700 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
```

EDITOR was checked that way (#193): the main menu with EDITOR selected (its echo and
description) and moved through; LOADING (menu frames 28-206 of the second recipe), the
editor in edit mode (the help panel, F5's ring, no squad: "edit mode from the start: the edit
camera at [0.00, 3.95, 9.00]", as on `--edit`), back on the main menu with EDITOR selected
(menu frame 207 on), its items sliding in, then MULTIPLAYER, DEATHMATCH, SELECT MISSION and the
arena (mp1) loading with no test map line in the log and no controls panel (#193 ran it through
DEATHMATCH on the main menu; #310 put it under MULTIPLAYER). #193 also ran the campaign path after
it: a new campaign on the tutorial, the mission flow in the campaign mode.

MULTIPLAYER was checked that way (#310), in captures and screenshots of the recipes above
(1280 × 720): the main menu with the five entries evenly spaced and each selected in turn
(`BF_MENU_SCREEN=main`, `BF_MENU_PICK=0` to `4`: CREDITS well clear of the bottom edge, its echo
whole, its description "See the roster of the 23rd Special Forces."); the submenu with
DEATHMATCH and with SQUAD DEATHMATCH selected, set beside the game's CAMPAIGN TYPE (take01 at
7.5 s and the demo's `BF_MENU_SCREEN=campaign`: same panel, title, button and description
places, fonts, colours and echo; the heading bands are the one thing left out, and the two
buttons are at the same places with and without them); the full runs through DEATHMATCH to mp1
and through SQUAD DEATHMATCH to sdm_e40 (SELECT MISSION, LOADING, the map with the squad and the
radar); B at every step (map -> SELECT MISSION on that map -> the submenu on that mode -> the
main menu on MULTIPLAYER -> the title); EDITOR and CAMPAIGN (CAMPAIGN TYPE, B back to the main
menu on CAMPAIGN) reached as before; `BF_MENU_SCREEN=dm|sdm` with `BF_MENU_GO` landing on the
same maps. Unit tests (`cargo test --bin bf_play`): the submenu's derived menu (CAMPAIGN TYPE's
first two buttons in their places, no heading, the other buttons gone), its modes and Back
mapping, the main menu's order and even spacing, and that a map's `Return` is spent once the menu
has opened on it (a second `open_menu` opens on the title: a stale one would reopen that map's
SELECT MISSION after a later way into the menu that leaves nothing, a campaign's last briefing).
The mouse can't be captured (`BF_BACKGROUND` drops the mouse's events), so its path was run with a
temporary probe (removed again): the hit boxes were at the buttons' data boxes in screen units
((80, 152) and (80, 180), 200 × 20), and a simulated hover on SQUAD DEATHMATCH moved the
selection to it and a simulated click on it opened its SELECT MISSION. Not tried: real key,
pad and mouse input (`BF_BACKGROUND` takes the pad and the mouse out; `BF_MENU_PRESS` fills the
same `Asked` the keys and the pad do, which the submenu reads as the other screens do), and not
heard: the sounds (`BF_MUTE`); they're the calls CAMPAIGN's path makes.

Record texts (the campaign path's, the end screen's, the status line) are made before the UI's
frame starts (`play_fonts.rs` runs before bevy_ui's Prepare and Stack sets and the visibility
propagation): a glyph node made after them isn't drawn until the next frame, and a text made
anew every frame (sliding or typing in) would never show.
`BF_MOVIES=<mission>` plays that mission's briefing movies after the boot instead of the opening
movies, then the title (briefing.md).

The arenas Chamber (mp2), Ammo Depot (mp3) and Cavern of Fire (sdm_m07) are built from objects
alone, with no terrain mesh. Their rooms are lit by the
level's point lights (see Point lights below).
