*Part of [squad/](README.md), the former page `squad.md`.*

* **Squad hand-over.** It plays one sound as the camera cuts: `h_07949f0d` (common bank,
  0.18 s), found in a capture at all five hand-overs. It replaced the weapon-switch pair used
  before.
* **The name label** (#88, play_hud.rs `LABEL_SHOW`): the select widget's label (FUN_000c92e0),
  the member's name in the heading font at 24 points (the record 0x3c7b90 on h_e0afcd52,
  `fonts::HEADING24`) centred on (320, 240), orange (FUN_000b87e0's ff a0 6a) at 0.8 alpha.
  Each selection shows it for 0.7 s (0xc9440: its timer 0.4 + 0.3): whole for 0.4 s from its
  first frame, then over the last 0.3 s its alpha falls to 0 and it swells to 1 + 3k^3 times
  (k the share of that 0.3 s gone: 4x at its end; 0xc9690). Control passes 0.5 s after the
  selection (the select widget's countdown, FUN_000c8760's +0x7c: `SELECT_TIME`, which was 0.6),
  so the swell runs across the cut. The recordings (`todo/reference/Friendly Fire 2 + Death Cam.mp4`) agree:
  "BRUTUS" from 22.867 s, whole and the same size from its first frame, fading from 23.267 s,
  the cut at 23.367 s; "HAWK" from 31.847 s (3.5 s after the death: FUN_000c9170's 3.5 s), fading
  from about 32.23 s, the cut at 32.333 s; their ink 272.0-368.7 x 230-251 for HAWK and
  256-385 for BRUTUS, the record's cells to a unit, and (209, 138, 86) over the gas. The
  recording's swell runs about 0.05 s ahead of the code's curve counted from the label's first
  visible frame (1.24, 1.56, 2.04 and 2.62 times HAWK's width at 32.330, 32.363, 32.397 and
  32.430 s; the curve gives 1.13 to 2.3 there): its select widget may show the label a frame or
  two after the selection (not traced). Not done: the blue label (0x4a5c90's 3c 69 dc) the
  game gives a member the select widget already holds. `BF_TEST_SELECT=<character>[,<s>]` picks
  that member at 1 s (or s), also together with `BF_TEST_GOTO`.
  Checked at 60 fps on sdm_e34 (`BF_CAPTURE_FPS=60`): `BF_TEST_SELECT=0,1.0`: "BRUTUS" whole from
  frame 61 (1.017 s; 256-385.3 across, (221, 136, 92)), fading from 1.433 s, swelling to 2.3x
  by 1.617 s; `BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`: "BRUTUS" from 4.517 s, fading from
  4.933 s, the cut at 5.0 s, swelling past it.
