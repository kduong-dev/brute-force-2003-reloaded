# Map menu

`cargo run --bin bf_play` with no `BF_MAP` opens the front end, rebuilt after the game's own (the
capture in `todo/`). It's laid out on the game's 640 × 480 screen and scaled to the window.

* **Title.** START (DEMOS is shown but not here).
* **Main menu.** CAMPAIGN, DEATHMATCH, SQUAD DEATHMATCH, OPTIONS, CREDITS, with the selected
  word white over a faint, larger echo, and its description beside the logo. Only DEATHMATCH
  and SQUAD DEATHMATCH lead on; the game's mode panel (split-screen / System Link) is skipped.
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
  (Esc at the title quits). The mouse points and clicks.

All of it is the game's. The front end itself is defined by `common/game-options-en.xmb`: its
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
| Layout | game-options' item positions: words right-aligned in boxes ending at x 279; logo at its own 512 × 256 (title (194, 106), main (194, 18)); description box (312, 240) |
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
recogniser follows, and the Xbox's language order). They are mixed down to stereo, the centre and
rears at -3 dB, the low channel at -6 dB, under a limiter (track 0 alone left out the intro's
narration and most of the MGS logo's sound). The credits movie has one stereo track.

```
mix='[0:a:0]aresample=44100,aformat=channel_layouts=stereo[f];[0:a:3]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[c];[0:a:1]aresample=44100,aformat=channel_layouts=stereo,volume=0.5[e];[0:a:2]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[s];[f][c][e][s]amix=inputs=4:normalize=0,alimiter=limit=0.95:level=0[out]'
for m in MGS_Logo_Final DA_Logo_Final1 Intro_Montage credits; do
  ffmpeg -i "Brute Force/data/movies/$m.bik" -vf fps=30 -q:v 5 decompiled/movies/$m/%04d.jpg
done
for m in MGS_Logo_Final DA_Logo_Final1 Intro_Montage; do
  ffmpeg -i "Brute Force/data/movies/$m.bik" -filter_complex "$mix" -map "[out]" -ac 2 -ar 44100 decompiled/movies/$m/audio.wav
done
ffmpeg -i "Brute Force/data/movies/credits.bik" -map 0:a:0 -ac 2 -ar 44100 decompiled/movies/credits/audio.wav
```

(10 s, 15.7 s, 67 s and the credits 150 s, 163 MB; a movie without frames is left out.) The main
menu's CREDITS plays the credits straight away, their frames read from disk as they're shown (any
key, click or button stops them) and comes back to the main menu on CREDITS. Movies played on their own fit
inside the window, whole (the menu's background covers it). `BF_MENU_SCREEN=credits` picks CREDITS. `BF_NO_INTRO=1` skips the movies;
the menu screenshot hooks skip them too and count frames from the menu, unless `BF_SHOT_EARLY=1`
(frames then count from the start: the boot screen, the movies).

When the menu is skipped: whenever `BF_MAP` or a test hook (`BF_CAPTURE`, `BF_TEST_GOTO`, …) is
set, or with `BF_NO_MENU=1`. Test hooks: `BF_MENU_SHOT=<file.png>` saves the menu and quits;
`BF_MENU_SCREEN=title|main|dm|sdm|loading` with `BF_MENU_PICK=<n>` picks the screen and entry;
`BF_MENU_SHOT_FRAME=<n>` takes the shot at frame n (screenshots step a fixed 1/60 s a frame, to
catch the animations); `BF_MENU_GO=1` plays the picked map (the screenshot hook runs on into it); `BF_TEST_BACK=<n>` goes back to the menu after n frames of a map.

The arenas Chamber (mp2), Ammo Depot (mp3) and Cavern of Fire (sdm_m07) are built from objects
alone, with no terrain mesh. Their rooms are lit by the
level's point lights (see Point lights below).
