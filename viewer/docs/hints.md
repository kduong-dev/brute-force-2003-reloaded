# HUD hints, button gates and the tutorial panels

The tutorial teaches its controls with three kinds of trigger (#224, step 5 of #28): message
triggers put a hint line on the HUD ("Press [BACK] to check mission objectives."), input triggers
wait for the button it names, and menu-triggers open the tutorial's panels over the game (BASIC
CONTROLS, HEADS-UP DISPLAY (HUD) ...). An event-trigger hears an AI event about a character (its
health under 25%) and shows the Medkit hint. The four classes are the trigger runtime's
(`play_triggers.rs`, [triggers.md](triggers.md)); their own code is in `play_hints.rs` (the
message, input and event classes, the virtual buttons, the AI events, the hint lines) and
`play_panels.rs` (the menu-trigger and the panels), with the panels' data read by `bf/menus.rs`.

```sh
# the start area's chain: the co-op hint, BASIC CONTROLS 5 s after Bratton's last line, HUD and
# ADVANCED HUD on leaving the 8 m circle, then the eight button hints, each gate pressed
BF_MAP=tutorial BF_HINT_LOG=1 BF_TRIGGER_FIRE="trigger_sound_bratton_refresher,USER1,2" \
  BF_TEST_GOTO=65.0,74.4,57.6,68.3 BF_TEST_GO_AT=16 BF_TEST_PANEL="a,1" \
  BF_TEST_VB="62,29;17,34;5,39;6,44;12,49;12,54,2.5;7,60;10,65" cargo run --bin bf_play
```

## What the game does

### The hint lines (message triggers)

* **The trigger** (element h_e9b3ae8e, vtable 0x395bf0; parser FUN_002077b0) holds two lines,
  each shown or not, a kind, a virtual button and a text (h_f69bd9c8 / h_172c4e2c,
  h_18c6906a / h_f39e6b58, h_ef8df28a / h_04d509b8, h_170f888b / h_0a22ead1: +0x40-+0x5c). Its
  OPERATE (handler 0x2078e0) sends OPERATED (its iD, the event's subject and data) to its
  targets, then sets the player view's two hint lines (+0x3c, +0x40): a line it shows takes the
  kind, button and text and starts blinking; one it doesn't is hidden at once. So the tutorial
  clears a line with a trigger that shows nothing (h_fb539597, h_f07ac787 ...).
* **The line** (a HUD widget, vtable 0x39f0f8; constructor FUN_000cb1c0, update 0xcb220) stands
  at the HUD safe area's left (51) and its top + 30 + 16 full screen: (51, 85) on the 640 x 480
  screen; the second line 20 lower. It is three texts (0xc9b50): kind 1 "Press " and kind 2
  "Hold " (h_ff43bc83, h_1cc51f04: FUN_000ca000), then the glyph of the pad button bound to its
  virtual button (FUN_000c9d70's table at 0xc9ee4), 3 units lower, then the text, 4 units after
  the glyph's advance (FUN_000c9b10's +0x70). A stick or a d-pad direction has a word instead of
  a glyph ("D-Pad Up", "Click Right thumbstick": FUN_000ca1a0); with no button (the tutorial's
  kind 0 lines, VB 0) the text alone starts the line.
* **Its look.** The words in the body font at 14 points (h_0c626fd5, record 0x3c35a8: the
  briefings' subtitles' glyphs, [briefing.md](briefing.md)), pale blue (c0 d7 ff) over a shadow
  1 unit down and right in 0a 0a 0a at half the alpha (FUN_000c99d0). The glyph in the button
  icon font h_0e38a4ca at 16 points (record 0x3c6920 on the atlas h_0b630034, v x 0.5 + 0.684,
  FUN_000cd5d0), white, its cell taken a texel up as FUN_000ccd20 draws it (the discs' tops cut
  flat, the START and BACK letters whole). A = A, B = B, C = X, D = Y, E / F the left and right
  triggers, G START, H BACK, I White, J Black.
* **The blink.** A line just shown has its timer (+0x78) set to 3 s, counted down by the frame's
  time; it's drawn while the fraction of 2 x the timer is under a half (FUN_000cb410 with +0x7c
  4.0): off 0.25 s, on 0.25 s, six times, then steady. take04's BACK hint: six off phases 0.51 s
  apart from 49.77 s, steady from 52.60 s (the watched take's clock runs at ~0.95).
* The skill's name under the bars stands 20 units lower for each hint line up (0xc73b4,
  `play_hud.rs` LABEL_STACK).

The tutorial's lines (string ids; [glyph] kind 1, [hold] kind 2): "[START] on any other
controller to play co-op." (h_ece8df23, 0.1 s after Bratton's last opening line ends);
"[BACK] to check mission objectives." (h_e5933c3b), "[START] for game options." (h_1ba72ee9),
"Pull the Right trigger to fire weapon." (h_1ff40d0a, kind 0), "[X] to reload weapon."
(h_094bf7eb), "[B] button to cycle inventory." (h_1070c712), "[hold B] for full inventory view."
(h_11e5bd0a), "Pull the Left trigger to use item." (h_f449edbf, kind 0), "[Y] to switch
weapons." (h_fd019b39): the start area's chain; "[Black] to use Medkit." (h_e4bc535e, the
event-trigger below); the grenade, mine, Berserker ("[White] to toggle Berserker Mode.") and
sniper hints further on (todo/28-tutorial/leads.md §6.1).

### The use prompt

A world button in reach ("Hold [X] to activate panel.", the gate's wall panels) has a prompt of
its own: a HUD widget (vtable 0x3a02b4, its update 0xaa130) built as a hint line is (the same
three texts, 0xc9b50; the body font at 14 points full screen): for a use that takes a hold
(0xaad20), "Hold " and the ACTION_BUTTON's glyph (VB 6, X: FUN_000ca290), then the target's text
(FUN_000aa600: a world button's "to activate %s." h_f9d29692, with its name; the other targets'
pick-up, swap and disarm prompts are its other cases). It stands at the first hint line's place,
(51, 85), 20 lower while the skill's name shows (the HUD's message widget, hud+0xfc), and 20 lower
again for each of two tests of the first hint line (the player view's +0x3c read twice at
0xaa2cd-0xaa2e8, where the second line's +0x40 would be expected: 40 under a first line). It
doesn't blink. take09 and take11 show it alone.

### The button gates (input triggers)

* **The trigger** (element h_e2b9c3a2, vtable 0x3954a8; parser FUN_00200760): `controller`
  (+0x40, 4 by its constructor), h_fa80f605 (+0x44, the pad input the editor bound) and
  `virtual-button` (+0x48). Its update (0x2008a0): the base update, then while it's enabled,
  every frame its virtual button's value isn't 0 on one of the sixteen input objects (slots 1-8
  through FUN_00078df0 and FUN_00078d80), TRUE (iD 0, no subject, data -1) to its targets. It
  reads neither `controller` nor h_fa80f605: any pad's press counts (the co-op hint's A gates on
  controllers 2-4 open on the first pad's A too, by the code).
* **The virtual buttons** come from the pad's bindings (FUN_001e57d0 builds them from
  game-options' `<action>` block and the hard-coded START, BACK and d-pad records; FUN_001e7b30
  sets a virtual button from one by its mode): a mode 0 binding counts every frame its button is
  held, a mode 1 one the frame it goes down (2 the frame it's let go, 3 while it's up). Under a
  menu or a panel the world isn't updated: no gate sees a press.
* Each gate's TRUE clears its hint (a message trigger that shows nothing) and starts a timer (3 s)
  that shows the next hint and enables the next gate: BACK, START, the right trigger, X, B, B
  held, the left trigger, Y (take04's presses: 62.6, 91.9, 118, 131.5, 144.7, 158.3, 176, 191 s).

### AI events (event-triggers)

* **The trigger** (vtable 0x395440; parser FUN_00202d60): `event-id` (+0x40, an EVT_ number of
  the table 0x3bdb9c) and `event-filter` (+0x44; h_fe4bc988, none, by its constructor). An
  object's AI events reach the event-triggers that list it (FUN_001fe440: kind 0x1a only, as
  signal 17 with the object as the subject and the number at the event's +0x14); the handler
  (0x202dd0) passes one whose number is its `event-id` (FUN_0004c800: with no filter the numbers
  only) to its targets as EVENT_MATCH and disables itself.
* **EVT_HEALTH_PCT_75, _50, _25** (57, 56, 55): a character's damage (FUN_00115f90, at
  0x116180) posts each when its health, in whole percent of its most, falls from at least 75, 50
  or 25 to under it. The tutorial's event486 hears EVT_HEALTH_PCT_25 about Tex: EVENT_MATCH shows "[Black] to
  use Medkit.", enables the four Black gates and starts timer510 (10 s), which clears it.
* **EVT_SOUND_ENEMY_NOISE** (49): event156 hears it about the second building's guards (filter
  h_08267484, which compares two values of the object's brain: FUN_0003a990, not traced).

### The tutorial panels (menu-triggers)

* **The trigger** (vtable 0x396108; parser FUN_00208140: `menu`, `Title`, `message`). OPERATE
  (handler 0x2082d0) opens its menu (FUN_002081d0: the menu system's open, with the trigger's
  callback at the menu's +0xac; the menus 0xf3b06f04 and 0x0ec41ef2 are message boxes of its
  Title and message instead) and sends OPERATED. Its update (0x208430) sends FINISHED (its iD)
  once its menu isn't open any more. A name game-options doesn't define opens nothing: the
  tutorial's trigger_menu509 (h_194eab9a) sends OPERATED, never FINISHED.
* **The buttons** go to the callback (vtable 0x3961d4), which sends to the trigger's targets (iD
  0, no subject, data -1): A on the panel's h_1966467f item closes the menu, then MENU_ACCEPT
  (51; 0x100ea0 then the callback's +0); Y sends MENU_Y_BUTTON (54; 0x100f60, the callback's
  +0xc) and closes it. take14: B, BACK, X and RB on BASIC CONTROLS do nothing.
* **The clock** stops while a panel is up, as under the objectives menu (take04's watch log:
  31081 ms from 4.9 s to the A at 12.2 s under BASIC CONTROLS). The panel shows whole in its
  first frame. The menu-trigger opens its menu without the cutscene test the player's own menus
  have (FUN_0008c8c0's +0xc50 bit 4).
* **Skipping.** Every panel sends to router h_0d29c80f, which passes all on to three: router
  h_10820d7c turns MENU_Y_BUTTON into USER1 to every panel (each one's USER1 disables it),
  router524 (h_fe0954d0) into USER2 to timer464 (2 s, ADVANCED HUD's and the hint chain's), and
  the sound-trigger h_165b85c6 plays on it. take13: Y on BASIC CONTROLS, then no HUD, ADVANCED
  HUD or WEAPONS panel, and the BACK hint 4.0 s of clock after the Y.
* **The panels** are game-options menus (`<game-menus>`, 470 x 360 at (85, 60)): pictures (the
  shade over the screen, the frame h_0d0ce2e2, the controller with its highlight rings, the
  samples), static texts in their styles (the body font at 10, 12 and 16 points: records
  0x3c0b68, 0x3c2088, 0x3c4ac8), wrapped as the game's static text does (FUN_0010a880 by the
  records' advances), and two legends, "A Accept" and "Y skip tutorial menus" (style h_e35bb628:
  the button in the icon font at 12 points, record 0x3c5fe8 on its own atlas h_0e233fad, outlined
  discs cut flat on top; the word in the body font at 12).

  | Menu | Panel | Opened by |
  |---|---|---|
  | h_f0ea44dc | BASIC CONTROLS | h_11313ffd: 5 s after Bratton's last opening line, or leaving the 6 m circle round the start |
  | h_ebe31566 | HEADS-UP DISPLAY (HUD) | h_f4cd3654: leaving the 8 m circle (h_e30f11a4) |
  | h_e2e425f0 | ADVANCED HUD | h_ec015b3c: timer464, 2 s after |
  | h_0680b053 | WEAPONS | h_f9eab62e: the patrol's radar line ending (h_eb8d3d96's FINISHED) |
  | h_0f8780c5 | ADVANCED WEAPONS | h_1092288c: timer529, 2 s after both patrol terrorists die |
  | h_148ed17f | INVENTORY | h_009231d0: timer521, 11 s after the grenade plate |
  | h_1d89e1e9 | SPECIAL ABILITIES | h_11b10113: the plate h_eef624e6 |
  | h_0736fc78 | SELECTING CHARACTERS | h_fcc6ec4a: timer393, 10 s after the V-squad plate |
  | h_0e31ccee | SQUAD COMMANDS | h_0a3f40d3: a d-pad gate |

### The start area, in order

Bratton's refresher line ends: the co-op hint 0.1 s later (timer429); BASIC CONTROLS 5 s later
(timer462); leaving the 8 m circle: the HUD panel, then ADVANCED HUD 2 s later (timer464) and
the BACK hint 2 s after that (timer391); timer523 clears the line 8 s after leaving the circle
(take04: gone by 54.0 s). Then each gate pressed shows the next hint 3 s later: START, the right
trigger, X, B, B held, the left trigger, Y. Walking onto the plate h_1ee3534e before ADVANCED HUD
shows skips it: the plate disables timer464, starts timer391 and clears the co-op hint.

## In the demo

* **The hint lines** are drawn as the game does: the body font's glyphs from its own atlas (the
  briefings' `FontAtlas`), the icon font's 16-point glyphs from h_0b630034 (`ICON_GLYPHS`, read
  from the record), placed, coloured, shadowed and blinking as above, in a HUD root of their own
  (`HudRoot`: hidden with the HUD in the editor's edit mode and while the letterbox's bars are
  in, play_script.rs). `BF_HINT_LOG=1` logs each line shown and cleared.
* **The use prompt** (`draw_prompt`) is the same glyph line: "Hold [X] to activate panel." while
  play.rs `doors` has a wall panel usable (not while scoped), lowered as above, steady. The ring
  on the panel's button stays play_hud.rs's.
* **The virtual buttons** stand for the pad's: the demo's keys for the same actions, and a
  pad's buttons (none read under `BF_BACKGROUND`):

  | VB | Game button | Glyph | Demo key | Pad |
  |---|---|---|---|---|
  | 5 | right trigger (held) | F | left mouse (captured), or the autopilot's fire | right trigger |
  | 6 | X (held) | C | E, R | X |
  | 7 | left trigger (held) | E | G | left trigger |
  | 9 | White (press) | I | V | left shoulder (xemu's White) |
  | 10 | Y (press) | D | Q | Y |
  | 11 | left stick click (held) | "Click Left thumbstick" | Z | left stick |
  | 12 | B (held) | B | Tab | B |
  | 13-16 | d-pad up, right, down, left (held) | "D-Pad Up" ... | 4, 2, 1, 3 | d-pad |
  | 17 | START (press) | G | Escape | Start |
  | 34 | right stick click (press) | "Click Right thumbstick" | right mouse (captured) | right stick |
  | 36 | A (press) | A | Space | A |
  | 62 | BACK (press) | H | I | Back |
  | 63 | Black (held) | J | B | right shoulder (xemu's Black) |

  B is the demo's Black: it uses no Medkit yet (the item box's G does); Escape stands for START,
  whose pause menu the demo hasn't got. The mouse buttons count only while the mouse is
  captured (the click that captures it doesn't fire).
* **The AI events** are posted by `ai_events` each frame, before the triggers' frame, from the
  characters the triggers know by name (the player, the squad, the spawn copies): the health
  marks against last frame's. EVT_SOUND_ENEMY_NOISE is **a stub**: posted for a character the
  player doesn't control the frame it starts to see a hostile or is hurt.
* **The panels** are drawn from game-options' data (`bf/menus.rs`: menus, items, styles and
  picture sets), their texts wrapped by the body font records' own advances, so the lines break
  where the takes' do. The words themselves are the demo's HUD font (play_text.rs) at capital
  heights measured on take04: a few percent wider than the game's 10 and 12-point glyphs. The
  legends' icons are the 12-point icon glyphs (`ICON12_GLYPHS`, `legend_icon`, which the
  objectives menu's legends use too: the same style). The clock stops from the frame
  after a panel opens to the frame it closes in (`set_paused`, as the objectives menu); no key,
  mouse or pad press reaches the game under it. Enter, Space or the pad's A accept; Q or the
  pad's Y skip; nothing else closes one.
* `play_hud.rs` moves the skill's name 20 units lower for each hint line up (the switch hint, in
  the first line's place, counts as it: a guess).

### Test hooks

* `BF_TEST_VB=<vb>,<s>[,<hold s>][;...]`: holds a virtual button from that time (the game's
  clock) for that long (0.1 s by default, and at least the first frame from that time), as a pad
  would; a press button counts its first frame.
* `BF_TEST_PANEL=<a|y>[,<open s>][;...]`: the n-th panel to show is closed with that button once
  it's been up that long (1 s by default; real time, or the capture's steps under `BF_CAPTURE`);
  the last entry stands for the panels after it.
* `BF_HINT_LOG=1`: the hint lines shown and cleared, the AI events posted, the first frame of
  each press that reaches an enabled gate, the panels shown and closed.
* `BF_TRIGGER_FIRE` fires the triggers directly (a message trigger on TIMER_PULSE, a
  menu-trigger on its entry's signal by number: [triggers.md](triggers.md)).

```sh
# every start-area hint 4 s apart, then the Medkit and Berserker ones (take04's spot)
BF_MAP=tutorial BF_HINT_LOG=1 BF_TEST_GOTO=57.5,68.22,56.8,68.22 \
  BF_TRIGGER_FIRE="h_fea44081,TIMER_PULSE,1;h_1e5b0db9,TIMER_PULSE,5;h_e0cab867,TIMER_PULSE,9;h_02255a3c,TIMER_PULSE,13;h_06c20e19,TIMER_PULSE,17;h_e277e10e,TIMER_PULSE,21;h_ef4bd10d,TIMER_PULSE,25;h_ffbdb2e3,TIMER_PULSE,29;h_009b6fa7,EVENT_MATCH,33;h_1246f92c,ENTER_DETECTOR,37" \
  cargo run --bin bf_play
# the nine panels, each closed with A after 2 s
BF_MAP=tutorial BF_HINT_LOG=1 BF_TEST_GOTO=57.5,68.22,56.8,68.22 BF_TEST_PANEL="a,2" \
  BF_TRIGGER_FIRE="h_11313ffd,12,1;h_f4cd3654,12,2;h_ec015b3c,8,3;h_0a3f40d3,2,4;h_11b10113,8,5;h_1092288c,8,6;h_009231d0,8,7;h_fcc6ec4a,8,8;h_f9eab62e,25,9;h_e28e679f,8,10" \
  cargo run --bin bf_play
# Y on BASIC CONTROLS: the later panels disabled, the BACK hint from timer464
BF_MAP=tutorial BF_HINT_LOG=1 BF_TRIGGER_LOG=1 BF_TRIGGER_FIRE="trigger_sound_bratton_refresher,USER1,2" \
  BF_TEST_GOTO=65.0,74.4,50.0,64.0 BF_TEST_GO_AT=16 BF_TEST_PANEL="y,1" cargo run --bin bf_play
# a 37 m fall: health 100% -> 14%, EVT_HEALTH_PCT_75 / 50 / 25, the Medkit hint
BF_MAP=tutorial BF_HINT_LOG=1 BF_TEST_GOTO=57.5,68.22,57.5,68.22 BF_TEST_FALL=37 cargo run --bin bf_play
```

## Not done

* **The panels' words** are the demo's HUD font, not the game's 10, 12 and 16-point glyphs (a few
  percent wider: a line can run a little past its box). The controller's highlight rings look
  dimmer than the takes' (BASIC CONTROLS' left stick ring is pink where take04's is orange): how
  the game blends those pictures isn't traced. The message boxes (0xf3b06f04, 0x0ec41ef2) aren't
  drawn; the tutorial opens none.
* **MENU_ACCEPT** is sent whenever A closes a panel (the callback's +0 once the menu has closed on
  its accept item, as FUN_00100ea0 has it when the menu's controller takes the press: whether it
  does isn't traced); nothing in the tutorial acts on it.
* **Black uses no Medkit** (B only clears the Medkit hint); START has no pause menu (Escape only
  opens its gate). The input triggers' `controller` isn't read, as in the code: player 1's A
  opens the co-op hint's gates too (no take shows it either way).
* **EVT_SOUND_ENEMY_NOISE** is a stub (the game's noise sources aren't traced), and event156's
  filter h_08267484 is taken as passing.
* **The wall panel's hold** ("Hold [X] to activate panel.", `play.rs` HOLD_TIME 0.5 s, a guess;
  take11 needs about 0.8 s): the game uses a world button when the user's push-button motion
  (OMT_PUSH_BUTTON) reaches its `push_button` animation event (FUN_00117660's case 0xf2cfbb92 at
  0x118b71: the use target at char+0x640, its vtable +0x138, 0x163060), but no clip in
  common.tgz, tutorial.tgz, mp_common.tgz or m01_a.tgz has that event: which clip plays and when
  it fires isn't found. The prompt is the game's glyph line (above); the other targets' prompts
  (FUN_000aa600's pick-up, swap and disarm cases, the mines' ring: #145) and the switch hint
  ("[Y] switch to %s.", still play_hud.rs's text with Q) aren't.
* The other hints of the tutorial (the grenade plate, the mines, the sniper rifle) need the
  pickups and plates of later steps to fire on their own (#228); fired directly they draw.

## How it was checked

`BF_MAP=tutorial`, `BF_MUTE=1 BF_BACKGROUND=1`, `BF_CAPTURE_FPS=5`, the logs read and the frames
set beside the takes' (todo/28-tutorial):

* **The lines** (the first command of the test hooks): each steady line against take04 (BACK
  53.0 s, START 88, the right trigger 112, X 126, B 140, B held 155, the left trigger 172, Y 186),
  take08 (Black, 171.3 s) and take09 (White, 285 s): "Press" from x 52.0 to 92.7 at y 90.0 in
  both, the texts' first ink at the same unit (146.7, 140.0, 124.0, 122.0), the glyphs within 0.7
  of a unit, the same rows of each glyph drawn. The blink against take04 (49.5-53.5 s).
* **The chain** (the command at the top): the co-op hint at 8.60 s, BASIC CONTROLS 5.0 s later,
  HUD on leaving the circle at 18.00, ADVANCED HUD at 19.80, BACK at 21.80, cleared by timer523
  at 26.00 (8.0 s after leaving the circle), then each hint 2.8 s after its gate's press (3 s, its
  timer counting the frame it starts in), in take04's order. A walk across the plate h_1ee3534e
  skips ADVANCED HUD, as the data says.
* **The panels** (the second command): all nine against take04 (BASIC CONTROLS 10.0 s, HUD 34.0,
  ADVANCED HUD 46.5), take07 (WEAPONS 63.0, ADVANCED WEAPONS 100.0, INVENTORY 209.0), take09
  (SPECIAL ABILITIES 278.0) and take11 (SELECTING CHARACTERS 119.0, SQUAD COMMANDS 123.2): the
  same pictures, texts, colours and line breaks; the legends' discs line up with the takes' to
  the pixel. h_194eab9a opens nothing.
* **Y** (the third): BASIC CONTROLS closed with Y at 13.80 s, h_10820d7c's USER1 disabling every
  panel, router524's USER2 starting timer464, the BACK hint 3.8 s of clock after the Y (take13:
  4.0 s; at 5 frames a second each of the two timers counts the frame it starts in), no HUD
  panel on leaving the circle.
* **The Medkit hint** (the fourth): "fall 163.84 x STANDARD ... = 98.30", EVT_HEALTH_PCT_75, 50
  and 25 about tex_1, event486's EVENT_MATCH, "Press [Black] to use Medkit." (take08 171.3 s),
  the four Black gates enabled, timer510 clearing it 10 s later; with `BF_TEST_VB=63,5` the
  four gates' TRUE clears it at the first frame from 5 s.
* **With the cutscenes** (#222): the tutorial's opening (`BF_MAP=tutorial BF_TEST_PANEL=a,1`, 10
  frames a second): the bars out from 25.90 s, the cutscene bit off at 26.10, the script's end at
  26.20, the co-op hint at 26.30, on screen with the HUD as the bars go (take03: by clk 26.46-26.56),
  BASIC CONTROLS at 31.30. A hint up when a cutscene starts (`BF_SCRIPT=tutorial_grenade_lesson,4`
  after the BACK hint at 1 s) goes with the HUD under the bars and is back, steady, as they go.
* **The legends** of a panel and of the objectives menu (`BF_TRIGGER_FIRE=h_0a3f40d3,2,1
  BF_TEST_PANEL=a,2 BF_TEST_OBJECTIVES=4,2`): the A, Y and B discs line up with take11's and
  take04's (63.0 s) to the pixel, the words' capitals at the takes' 396.0-408.0.
* **The use prompt** (`BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100 BF_TEST_USE=1.5`):
  "Hold" from x 52.0 to 86.7, the X glyph 96.0-123.3, "to activate panel." 123.3-250.7, as take11
  at 716 s (52.7-86.7, 96.0-123.3, 123.3-250.7); the gate opens on the use as before.
* **The clock under two menus** (the hooks only can cross them): a panel at 1 s
  (`BF_TRIGGER_FIRE=h_0a3f40d3,2,1 BF_TEST_PANEL=a,3`) with the objectives menu opened over it
  (`BF_TEST_OBJECTIVES=1.1,1`): the menu closing leaves the clock stopped under the panel (each
  holds its own bit of play.rs `PAUSED`), which closes at the same 1.20 s.
* **No regressions**: the objectives menu (`BF_TEST_OBJECTIVES=1,2` at take04's spot, against
  take04 63.0 s), the gate chain (`BF_TEST_GOTO=-45.7,-11.2,-43.9,-11.2 BF_TEST_GO_AT=100
  BF_TEST_USE=0.5`: the panel's use, its sound, the gate opening, the prompt where it was),
  sdm_e34 and the test map run as before.
