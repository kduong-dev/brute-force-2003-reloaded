# Briefing movies

Before a campaign mission's zone loads the game plays that zone's movies, each skippable, with
subtitles typed over them (`play_briefing.rs`, #225). The tutorial (Welcome to the 23rd!, `m00`)
plays `Campaign_Intro`, `Tex_Intro_Final` and `mb01`; Rescue Brutus (`m01`) plays `mb02` and
`Brutus_Intro_Final`. The demo has no CAMPAIGN entry yet (#226): `BF_MOVIES` plays a list.

```
BF_MOVIES=m00 cargo run --bin bf_play
```

## What the game does

* **The lists** (`common/campaign-bf.xmb`, read by `bf/character.rs` into `CampaignLevel::movies`).
  Each `<level>` has its id (`h_046391fd`: the mission's game-flag name, `m00`, `m01`, `e01`...
  recovered by hashing) and its `<zone>`s; a zone has `pre-movies` (played before it loads) and
  `post-movies` (after), each a `bink-file` in `data/movies` with its subtitles
  (`<h_e5d32e85>`: `text-string` a string table id, `h_f724cb8c`, `display-time`, and a
  `<style>` with `h_1927c9d0` and one or more `<h_05387560>` colours). Parsers: the zone
  FUN_001a3d40, the movie's file FUN_0018d750, the subtitle FUN_0018d7b0 / FUN_0018d690 and its
  style FUN_0018d610.

  | Mission | Zone | Movies (subtitles: start, shown, colour) |
  |---|---|---|
  | m00 | tutorial | Campaign_Intro (h_07665533: 18.5 s, 17 s, 60 80 251; h_1c6f0489: 2 s, 10 s, 60 251 80), Tex_Intro_Final, mb01 (h_1568341f: 13.5 s, 17 s, 60 80 251) |
  | m01 | M01_a | mb02 (h_070c696a: 68.8 s, 7 s, 60 80 251), Brutus_Intro_Final |
  | m01 | M01_b | M01_CS2 |

* **Where they play.** The game session's +0x30 (0x80220) opens the game-options menu
  `h_f5750fe9`: a full-screen movie player (`h_0462ccad`) and a static text (`h_1ad19da6`, at
  (60, 100), 520 x 360, style `h_1d46e79b`: the body font `h_0c626fd5` at 14 points, colour
  153 209 251, no outline). It queues the zone's movies in the player and hands the text each
  movie's subtitles. Its +0x34 (FUN_00080490) does the same for the post-movies in the menu
  `h_1d6f7c4e` (the text at (60, 60), 520 x 320). When a movie starts (FUN_00178730 calls
  0x10b0f0 with its index) the text is cleared and given that movie's subtitles.
* **When a subtitle shows** (the sequence 0x10be50, FUN_0010c060). A movie's subtitles follow
  each other: each takes its `h_f724cb8c` + `display-time` seconds from the end of the one
  before (the first from the movie's start) and is on screen for the last `display-time` of
  them. Campaign_Intro's report shows from 18.5 s to 35.5 s and the cloning archive from 37.5 s
  to 47.5 s (the list order, not the times' order).
* **Typing.** The text appears a character every `h_1927c9d0` seconds (0.033; FUN_0010a240 puts
  it in place of the static text's own style's 0.0675; the typing 0x10a630). Markup characters
  and line breaks count as characters.
* **Layout** (FUN_0010a880, FUN_0010abb0). Characters go on the last line; a line break starts a
  new line; a character (not a space or a tab) that makes the line wider than the box moves what
  follows the line's last space, hyphen or tab to a new line, so a word typed past the edge
  jumps down mid-word. Line n's top is the box's top + 24n.
* **The font.** The body font at 14 points is the record at default.xbe 0x3c35a8 (FUN_000cd5d0
  registers it, FUN_000cca40 reads it): the top, larger size of the atlas `h_e4e4d2f4`, line
  height 23 + 1, and per character (32 to 255) its cell on the atlas, the offset the pen moves
  before it, its width and its advance. `play_briefing.rs` keeps the record's table (`GLYPHS`).
  The renderer (FUN_000ccd20) draws each glyph's cell, (width + 1) x 24, at the pen plus its
  offset; tab stops are every 50 units from the screen's left (FUN_000ccae0, measuring a line
  for the wrap, counts them from the line's start).
* **Markup** (FUN_000ccd20). `^cN` switches to colour N of the text's colour list: the text's own
  colour first (153 209 251), then the subtitle's style colours (0x10be50), so `^c1` is the
  style's colour and `^c0` the text's own. `^f(N` .. `^)` blinks what's between at N a second
  (FUN_000cd510: hidden while the fraction of the clock x N is over 0.5), but only once the line
  holds its `^)`. A markup still typing out draws nothing past it.
* **Skipping.** A or START skips the movie playing (the movie item's input 0x1026a0 takes the
  menu's buttons 0 and 1 of FUN_000fff00; take02: A and START each skip one movie, B and X do
  nothing). After the last movie, played out or skipped, the menu closes and the zone loads.
* **Sound.** FUN_001781c0 opens a movie with its tracks 0, 1, 2 and the language's
  (FUN_00178090: English 3, Spanish 4, French 5, Italian 6, German 7, Japanese 8), routed to the
  front pair, the low channel, the rear pair and the centre. Campaign_Intro's language tracks
  are silent; mb01's are the voices (the take's sound follows track 3 and none of the others).

## In the demo

* `AppState::Briefing`. The entry point is `briefing::start(commands, next, mission, zone, list,
  then)`: the mission's id (`h("m00")`), the zone's index, `List::Pre` or `List::Post`, and what
  follows the last movie (`Then::Menu` the title, `Then::State(..)` a state the caller set up,
  for the mission flow, `Then::Quit`). The front end's own parts go as the briefing opens.
* The movie fills the 640 x 480 screen, which is scaled to fit the window whole (black bars on a
  wide window, as the takes show it). The subtitles are glyph nodes over it, rebuilt when a
  character is typed; blinking glyphs are shown and hidden every frame.
* Enter, Space, a click, or the pad's A or START skips the movie playing; B / Escape don't.
* A movie that wasn't converted is left out (a line on the console says which), and with none the
  hand-over comes at once.

### Converting the movies

As the front end's (menu.md), once: JPEG frames at 30 a second and the sound mixed to stereo,
into `decompiled/movies/<name>/` (git-ignored, never committed). The tracks are picked by their
Bink track id (`0:i:N`), not by their order in the file: Campaign_Intro and mb02 list track 1
last, so `0:a:N` would take the wrong ones.

```
mix='[0:i:0]aresample=44100,aformat=channel_layouts=stereo[f];[0:i:3]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[c];[0:i:1]aresample=44100,aformat=channel_layouts=stereo,volume=0.5[e];[0:i:2]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[s];[f][c][e][s]amix=inputs=4:normalize=0,alimiter=limit=0.95:level=0[out]'
for m in Campaign_Intro Tex_Intro_Final mb01 mb02 Brutus_Intro_Final; do
  mkdir -p decompiled/movies/$m
  ffmpeg -i "Brute Force/data/movies/$m.bik" -vf fps=30 -q:v 5 decompiled/movies/$m/%04d.jpg
  ffmpeg -i "Brute Force/data/movies/$m.bik" -filter_complex "$mix" -map "[out]" -ac 2 -ar 44100 decompiled/movies/$m/audio.wav
done
```

(48.3 s, 47.9 s, 132.7 s, 77.5 s and 72.9 s: 246 MB. The folder is named as the list names the
movie, without `.bik`; any other mission's movies convert the same way.) The demo looks for
`decompiled/` beside `viewer/`, in the working folder, or, with `BF_DATA_DIR` set (a git
worktree), beside the `Brute Force` folder.

### Test hooks

* `BF_MOVIES=<mission>[,<zone>[,post]]`: plays that zone's list straight after the boot (zone 0's
  pre-movies unless told), then the title; with `BF_CAPTURE`, quits after the list. It wins over
  `BF_MAP` and the other hooks that start a map.
* `BF_MOVIES_AT=<s>`: starts the list s seconds in (the movies before that point left out, the
  sound from there).
* `BF_MOVIES_SKIP=<s>[,<s>..]`: presses skip at those times on the list's clock.
* `BF_MOVIES_LOG=1`: prints each movie's start, end or skip and each subtitle's showing and going,
  with the list's and the movie's clocks.
* `BF_CAPTURE=<dir>` (`BF_CAPTURE_FRAMES`, `BF_CAPTURE_FPS`): after a short warm-up, the clock
  steps a fixed 1/15 s a frame and every frame is saved as `frame_NNN.png`. `BF_MUTE=1`: no sound.

## Not done

* The "Saving Campaign..." card the game shows before the first movie of a new campaign (#27), and
  the short black gaps between movies (0.35 s and 0.7 s in take01, 0.1 s after a skip in take02;
  the next file opening, a guess): the next movie starts at once.
* The blink's phase: the game's clock is the app's (FUN_000cd4c0), the demo's the list's; the rate
  and the half-on, half-off match.
* The CAMPAIGN entry, the LOADING screen and the zone after the list, and the next mission's list
  after the end screen: the mission flow (#226) calls `start`.
* Post-movies (MB17, EndGame, credits, epilogue) play through the same path with their box at
  (60, 60), unchecked against footage.

## How it was checked

Against `todo/28-tutorial/`: take01 (the three movies after a new campaign), take02 (skips) and
take12 (mb02 after the end screen), the demo with `BF_MUTE=1 BF_BACKGROUND=1 BF_MOVIES=m00` and
`BF_CAPTURE`, frames side by side.

* **Timing.** In take01 Campaign_Intro's sound starts at 35.84 s; its report is on screen from
  54.40 s (the first glyph) to 71.32 s, the cloning archive from 73.32 s to 83.33 s, mb01's text
  (its sound from 133.16 s) from about 146.5 s to 163.6 s: 18.5-35.5 s, 37.5-47.5 s and
  13.5-30.5 s into the movies, the starts a frame or two early (the first glyph appears ~0.07 s
  before the demo's). `BF_MOVIES_LOG` shows the demo's on those times to the capture's step
  (the archive at 37.53 s and gone at 47.53 s at 15 a second; in real time the report went at
  35.50 s).
* **Place and font.** Thirteen lines measured in take01 (66 s, 78 s, 156 s) and take12 (490 s):
  every line's ink starts and ends within 0.7 units of the demo's (`AT=18.3` frame 180, `AT=37.2`
  frame 78, `AT=109.5` frame 140; `BF_MOVIES=m01 AT=68.5` frame 68). Lines 24 units apart from
  y 104; tabs put "Deputy" at 200.
* **Colours.** The text 156 207 252 in the take, 153 209 251 in the demo; `^c1` blue 63 80 249 /
  60 80 251; TEX's green 68 247 93 / 60 251 80.
* **Typing and wrap.** mb01's text wraps after "procedure" and "Ship", "suc" typed on the first
  line then moving down (take01 148.6 s; the demo's frames 32-34 of `AT=109.5`).
* **Blink.** "<<LOADING>>" 0.5 s on, 0.5 s off in both.
* **Skips.** take02's presses at the same times from the list's start (A 13.61 s, START 29.61 s, A
  60.81 s: `BF_MOVIES_SKIP=13.61,29.61,60.81`): the demo skips Campaign_Intro at 13.75 s,
  Tex_Intro_Final at 29.75 s and mb01 at 61.0 s (the 0.25 s steps of `BF_CAPTURE_FPS=4`), as
  take02 starts Tex_Intro_Final 13.74 s and mb01 29.70 s after Campaign_Intro and goes to the
  LOADING screen.
* **Sound.** The converted sound lines up with the takes' (loudness correlation 0.78 / 0.89 /
  0.49 over the three movies); of mb01's language tracks only id 3 follows the take's.
* Unit tests (`cargo test --bin bf_play briefing`): the subtitle slots, the markup and tabs in
  the measure, the wrap, the sound cut for `BF_MOVIES_AT`.
