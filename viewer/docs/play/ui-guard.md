*Part of [play/](README.md), the former page `play.md`.*

### Keeping non-finite numbers out of the UI (#349)

`src/bin/play_ui_guard.rs`, and guards where the numbers are computed (play_hud.rs `at`, the
radar's blips and the bars; play_text.rs; play_fonts.rs; play_targeting.rs). Not the game's
behaviour: the demo keeping its own layout from panicking. The ticket is **open**: the cause of the
crash below wasn't found (no run reproduced it), so what's here keeps it from crashing and, if
it comes again, says where it started.

**The crash** (the product owner's, main, 2026-10-31): `bf_play` panicked in
`bevy_ui::layout::ui_layout_system`, "f32 clamp, max NaN", just after the log's ALE warning
"effect h_eb4128d7: not in the library, or nothing it draws is supported" for effect object
h_0969d0fb. The warning is the game's data, not the fault: h_0969d0fb is the Rails' flight effect
(the Rail RVR's and CLVR's: "Shots", the Rail RVR above) and its `rgun_side_s` emitter file is
empty, so `ale_fx` logs it the first time a Rail shot loads it. It says only that a Rail was fired
shortly before. The panic is bevy_ui 0.16.1's `BorderRadius::resolve_single_corner`
(ui_node.rs:2448), `.clamp(0.0, 0.5 * node_size.min_element())`, which `ui_layout_system` runs for
every node (each `Node` has a `BorderRadius`, 0 by default): a node whose laid-out size is NaN
panics it.

**What gets there**, by hand: one bad number (a NaN, then an infinity; as a `Val::Px`, then as a
`Val::Percent`) in one field of the style of the radar's four blips and dot, the net off
(`BF_UI_GUARD_OFF=1 BF_UI_GUARD_TEST=0.6,<field>[,px][,inf]`, 30 frames on the test map each; every
field in both units and both numbers):

| field | result (the same for NaN and infinity, px and percent) |
|---|---|
| `margin` | `ui_layout_system` panics with the product owner's message, "min > max, or either was NaN. min = 0.0, max = NaN" (f32.rs:1605) |
| `left`, `top` | the layout doesn't, but the next frame `ui_picking` and `ui_focus_system` do: "Rect size must be positive" (rect.rs:100) |
| `width`, `height`, `right`, `bottom`, `padding`, `border`, `gap`, `basis`, `min_*`, `max_*`, `aspect`, `grow`, `shrink` | no panic |

The only margins the demo writes are its glyph nodes' (play_text.rs `spawn_glyphs`, play_menu.rs
`text`: the glyph scale times a font constant), and a game text's scale is its constant caps times
the window's height over 480. Read that way no margin is NaN while the window has a height; a
window with none (minimised) made that unit 0, which divided the HUD ring's screen place
(`on_screen`: NaN or infinite, so a NaN `left`) and now returns before it. That, and a NaN position
or heading reaching the blips' `left` and `top`, are the paths found; neither is known to be the
product owner's.

**The guards**
* At the source: `ui_guard::finite` / `percent` turn a computed number that isn't finite into 0
  and log it once with the node and the call site (`#[track_caller]`): `hud::at` (so every HUD
  node placed by it, the target marker's corners and glyph strips included), the radar's blips
  (a blip with no place is hidden, saying whose position, the player's and the camera's), the
  radar panel's slide, the health, stamina and frame widths, the charge meter, the game texts'
  and the record texts' places, caps and scales. The target marker's box ignores a locked target
  whose place or size isn't finite, and puts a corner that went NaN back where the marker is off
  (the corners are carried frame to frame, so a NaN would stay). `play_text` and `on_screen`
  return while the window has no height.
* The net (`ui_guard::net`, PostUpdate after the systems that write nodes there, before the
  layout, **in every state**: the front end's and the briefing's glyphs, play_menu.rs `text`,
  carry margins, the one field that reproduces the layout panic; only the test hook is
  Playing's): every `Node` changed this frame is scanned, any non-finite length reset (a position or
  size to auto, a margin, padding, border or gap to 0, the aspect ratio to none, the flex factors
  to their defaults) with a warning naming the node (its `Name`, or its nearest named ancestor's)
  and then its components, which show the writer's own marker (`SquadPart`, `Part`, ...); a
  `UiScale` that isn't finite and above 0 is put back to 1. A style no longer reaches
  `ui_layout_system` with a non-finite length, whoever wrote it. Not covered: a size an image's or
  a text's `ContentSize` measure gives, which isn't a style.
* The watch (`ui_guard::watch`): the first frame a character's position, facing, stamina or
  health, or the camera's heading, isn't finite, a warning gives the time, the character, what it
  was doing and its last finite position: the frame that started it. It doesn't repair them.

**Hooks**: `BF_UI_GUARD_TEST=<s>[,<field>[,px][,inf]]` sets that style field (`width` by default;
the fields are the table's: `width height left right top bottom margin padding border aspect grow
shrink basis gap min_width max_width min_height max_height`) of the nodes with a border radius of
their own to NaN (`inf`: infinity; `px`: as a `Val::Px`, else a `Val::Percent`) at that sim time,
after the frame's writers; `BF_UI_GUARD_TEST=<s>,member` sets the first squad member's position
to NaN; `BF_UI_GUARD_OFF=1` turns the net off (the source guards stay), which shows the panic the
net prevents.

Verified (`--test BF_TEST_GOTO=0,8,0,7.5`, `BF_MUTE=1 BF_BACKGROUND=1`, 30 frames each): the table
above is 66 runs, the 33 field and unit cases (aspect, grow and shrink have no unit) for a NaN and
for an infinity, the net off: 12 panic (`margin`, `left`, `top`, each in both units and for both
numbers), 54 don't. With the net on, the cases that panicked (`margin` as a percent, as a px and as
an infinite px; `left`, an infinite `left`, `top` as a px) and `width` as a px: all 30 frames, exit
0, the dot and the three blips named in the log ("under \"radar panel\"", the writer's `SquadPart`
among their components). `BF_TEST_FULL_SQUAD=1 BF_UI_GUARD_TEST=0.6,member`: Brutus's position
NaN at 0.6 s, the watch names him a frame later with his last finite position and the blip's guard
hides his dot, saying whose position and the camera's, no panic. **Not reproduced**: the Rail RVR
held down on e34 for 26 s (`BF_MAP=e34 BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR;Foley 356 Tact"
BF_TEST_GOTO=-52.76,31.02,-52.9,31.5 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.08
BF_TEST_TRIGGER=2.0-28.0 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, the HUD and the squad
shown, 112 rays; the log has the same "ALE effect h_eb4128d7" warning at the first shot as the
product owner's, then nothing), and held down 18 s on the test map with the full squad about, at
Shrike Soldiers set in line (`BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR" BF_TEST_GOTO=-1.93,8,-1.93,8
BF_TEST_TRIGGER=1.5-20.0 BF_TEST_NPC="0.5,h_e96829a0,enemy,frozen,-1.93,<z>"` for z 6, 4, 2 and
0.5, `BF_TEST_NPC_HP=1.0,all,300 BF_TEST_INSTANT_KILL=0`; the first two were struck, 14 and 3 rays,
0.6 and 2.6 m out, each going on through): no "UI guard" line and no panic in either.

Not found, not done: **what made a number NaN in the product owner's run**; a size from a
`ContentSize` measure (an image or a text with no size of its own) isn't scanned by the net; a
character that went NaN is reported, not repaired. If it comes again, the log's first "UI guard:"
line names the node and the call site, or the character and the sim time.
