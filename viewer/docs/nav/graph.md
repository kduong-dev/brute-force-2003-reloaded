*Part of [nav/](README.md), the former page `nav.md`.*

## The graph

One node per world point of every kind: the level's `Level::points` (`bf/level.rs`), the
world's one list at world+0x110 that the search, the own point's walks and FUN_000ab260 all read.

* **Kinds** (each point's vtable +0x3c, read from the world point factory FUN_000ee070's
  constructors): 0 `<world-point>` (the plain navigation point), 1 start, 2 hide, 5 sniper,
  6 `<door-point>`, 10 `<cover-point>`, 11 / 12 patrol points, 16 direction, 0x13 h_16308887,
  24 progress, 0x1a h_f87c8847 (a jump's take-off), 0x1b h_e22441fa and 0x1c h_085b36d2 (its
  landings), 0x1d `<save-point>`, and thirteen more (7-9, 13-15, 17, 18, 20-23, 25; the code
  map's FUN_000ee070 row has them all). Every point type in the game's 54 levels is one of them.
* **Links** (h_e3439383): as the data lists them, one way each, in its order (FUN_0020d220 pushes
  one node per name and adds no reverse). The data is symmetric but for 12 links in the
  tutorial, 1 in sdm_e34 and 1 in e34; no level has a link to a missing point, to itself, or
  two points of one name.
* **The run-time flags** (+0x28):

| Bit | | Set / cleared by | Read by |
|---|---|---|---|
| 0 | claimed | a waypoint trigger's h_06d92de9 (every one in the data is -1); the follow's state 4 on the cover point it picks near the leader (FUN_000225d0, #331: play_follow.rs `claim`; its search passes over a claimed point, FUN_00020170 turning its filter's claim test on, 0x201df: `cover_ok`) | GOAL_IDLE's start, the point filters |
| 1 | a route's destination | a route's start (FUN_00025730), cleared at its end and on a reset | (#33 take08 read 6 on the point Hawk went to) |
| 2 | **valid** | every point at the start (FUN_00059b60); a waypoint trigger's `set-valid` | the search, the own point's walks, FUN_000ab260, the cursor |
| 3 | **avoid** | FUN_00041870 on the own points of the friends in a member's way, for its flood alone (cleared at its end, but on a point that had it); FUN_0004e8f0 clears it on a reset route's points | the search, the flood (FUN_000594a0) |
| 4 | no look-ahead skip | a non-empty h_ed6a7490 (FUN_000ecdc0; 175 of the tutorial's 1070 points, 27 of e34's 1286) | the walk |

* **Waypoint triggers** (`<h_10ecb459 world-point set-valid h_06d92de9>`, the handler 0x202c50
  -> FUN_00202ba0): OPERATE sets the point's valid bit to `set-valid` (0 or 1; -1 leaves it), by
  the point's kind: a cover point's set-valid goes to its own +0x64 bit 5 instead (not the
  graph's; read as whether it serves as cover, a guess), kind 0x13's to a byte of its own, a save
  point's to the
  world (vtable +0x88, not read). Links never change. As the tutorial starts (their
  FIRST_UPDATE) they make three door points invalid (h_03733e9d and h_0a740e0b at the base's
  gate, h_eb5dd521) and seven world points of the mine field (93-98, -71..-83); using the gate's
  two wall panels (h_066a5235, h_14643319: USE_OBJECT) makes its two door points valid again,
  and the object h_e45338bd's DESTROYED makes h_1932caf7 invalid. e34's make six door points
  invalid as it starts, sdm_e34's two. The trigger runtime runs them (play_triggers.rs
  `waypoint`) and play_nav.rs applies them to the graph.
