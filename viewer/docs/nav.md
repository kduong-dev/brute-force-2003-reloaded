# AI navigation: world points and routes (`src/bf/nav.rs`, `src/bin/play_nav.rs`, #334)

How the game's AI finds its way: the level's world points as a graph, the route search over it,
where a character's routes start, and how a route is walked. This is the first slice of #242
(AI routes): the shared machinery. The squad's follow and Move To on routes are #335, the
enemies' goals #336, the script walks #337; until then nothing walks a route in the demo but the
`BF_TEST_ROUTE` hook below, and the only live reader is the Move To cursor's filter, which takes
a point's valid bit from the graph ([squad.md](squad.md)).

Everything is read from default.xbe (static, not run): the route request FUN_000416a0, the search
FUN_00059000, the own point's upkeep FUN_0014f860, the walk FUN_00025730 / FUN_00026d80, the
waypoint trigger 0x202c50; every function is in [code-map.md](code-map.md), "AI". The reading is
`todo/242-ai-routes/leads.md` (the analyst's), with the corrections at the end of this page. No
footage shows a route yet (`todo/242-ai-routes/shotlist.md` lists what to film).

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
| 0 | claimed | a waypoint trigger's h_06d92de9 (every one in the data is -1); the follow's point pick (#335) | GOAL_IDLE's start, the point filters |
| 1 | a route's destination | a route's start (FUN_00025730), cleared at its end and on a reset | (#33 take08 read 6 on the point Hawk went to) |
| 2 | **valid** | every point at the start (FUN_00059b60); a waypoint trigger's `set-valid` | the search, the own point's walks, FUN_000ab260, the cursor |
| 3 | **avoid** | the blocker route (FUN_00041870, #335), cleared on a reset route's points | the search |
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

## The search (FUN_00059000)

A* from the start forward along each point's own links, to the goal:

* the open list is a binary heap on f, the STL's (`push_heap` / `pop_heap` as FUN_00059920,
  FUN_00059850 and FUN_000599b0 do them); it stops when the goal comes off it;
* g grows by twice the link's 3D length (FUN_00058f50) and **is truncated to a whole number at
  each step** (`cvttss2si` at 0x591e1): a link under 0.5 m costs nothing, one of 2.49 m costs 4;
* h is the straight 3D distance to the goal (FUN_00058ef0), f = g + h: with the cost doubled the
  heuristic counts for half, so the search is close to Dijkstra's, the shortest way winning;
* a neighbour must be valid (bit 2) and not avoided (bit 3); the start's first neighbours met
  before any passes go through without bit 2 (the record vector holds only the start), so a
  character whose own point has been made invalid still gets out of it;
* a point already open or closed is taken again only for a strictly lower g; a closed one is
  opened again; an open one whose f falls isn't moved in the heap (no sift for it);
* no node or time cap; a radius round the start can limit it (DAT_004021a8, -1 in the binary:
  off; GOAL_ATTACK's approach sets it for one search, #336: `Graph::search`'s `within`).

Its cost in the demo (`BF_ROUTE_COST`, opt-level 2): on e34, the biggest graph (1286 points),
6561 searches between every 16th point and every 16th average 56 us, at most 1108 points off the
heap (1230 with every point valid), the slowest single search 1.9 ms (one outlier; 7.4 ms in the
all-valid pass). The tutorial's lava-yard route: 62 points popped, 33-51 us.

## A route (FUN_000416a0 with FUN_0005a440)

1. The character's own point and the destination's the same (none on both counts too, as the
   game compares them): the one-entry route to the destination (the point, or for a character a
   bare position).
2. The search runs from a **temporary start at the character's position linked one way to its
   own point** (char+0x24c) to the **destination's point**: a point, or a character's own point
   (a character is routed to its point, not its position). The temporary is dropped from the
   path: one entry per point, its position its live one.
3. **The first entry is dropped** (FUN_00041320) when the route has two entries or more, the
   first point isn't a door's or a jump's, and the corridor from the character (at that point's
   height) to the second entry is clear: it goes straight to the second instead of back to its
   own point.

The corridor (FUN_000410a0) is two rays offset 0.8 m either side across (x, z), each the straight
length + 0.1 at most; blocked when either meets something. Its ray (FUN_0004ce30) meets the
terrain, the placed objects' collision and the blockers and hinters, never characters, items or
liquids (its object-type mask skips type 22, OT_WATERLAVAFLOW): the demo's is the arena's ray
(`play_nav.rs` `clear`). The rays run at the points' heights, mostly 0.4-0.5 m over the floor in
the levels looked at (up to 1.3 m on a slope).

Not done: the temporary points of FUN_00059f70 (a place with no point, linked to up to 6
visible points within 20 m) and FUN_00059d40 (beside a point): `NoRoute::NoStart` /
`NoGoal`. FUN_000413f0, which links a destination point with h_1346678f (+0x24 bit 15) to the
valid point nearest it within 40 m for the search: as read that's the destination itself
whenever it's valid, and an invalid one stays unexpandable, so it changes no route.

**Islands.** sdm_e34's sniper walkway (34 points at y 41.5-42.5, its world points with
h_1346678f) has no link in or out: no route reaches it from the rest of the map (the demo's
"unreachable", as the game's search as read). e34 is in 31 weak components (one of 1238 points),
and its six door points made invalid at the start cut it further: of the 6561 pairs above 628
have no route with every point valid, the count a plain reachability walk over the data gives,
1608 as the level starts.

## The own point (FUN_0014f860)

A character's world point (char+0x24c) is where its routes start and where routes to it end.
The character update keeps it every frame once it's wanted (+0x250 bit 0: the first time a
place of the character is made, at a spawn, at a route's start), for a living character with a
brain, outside a deathmatch arena:

* no point yet, a forced refresh due (+0x250 bit 6 and 2 s), or its point over 20 m away (3D):
  FUN_0014f7a0: kept within 5 m, else the nearest valid point of all (FUN_000ab260, 3D, any kind);
* else, while it isn't in a jump (FUN_00126a90: MT_JUMP_CROUCH, _FALL, _LAND, _LAUNCH): the
  nearest to its position of its point and the valid points one link on (two every eighth frame,
  by a frame count in +0x250 bits 3-5; FUN_0004bf30). When that's another point, or the point's
  height is off (more than 1.5 m under the character's position less 0.25, or 0.5 m over), and
  0.25 s have passed since the last deeper search, the deeper search decides: 6 links deep from
  the current point, the nearest (FUN_0014f5c0); after 1 s with the height off by over 2.5 /
  1.5 m, 10 links deep and the first of the 6 nearest the sight ray reaches. It keeps the point
  when it finds none. Within the 0.25 s the near walk's pick is taken as it is.

The position is the character's transform's (vtable +0x40), read as its hips, the skeleton's
origin (FUN_000bf2f0's row calls that position the node, the hips; inferred): bf_play's
`Player::position`, which stands 1.05 m over the feet. `OwnPoint` carries the point, its flags
and its three timers; nothing keeps one per character yet (#335 does, for the squad and its
leader).

## Walking a route (FUN_00025730, FUN_00026d80)

A route is started (FUN_00025730): its first entry the target, its last point marked as a
destination; a character standing on a jump's take-off (kind 0x1a) whose route starts at a
landing (0x1b / 0x1c) gets its own point put first, so the jump is made. Then each update
(`Walker::step`):

1. **The target** is the current entry's position; **the radius** the caller's (2.0 m: the
   follow, MOVE_OUT, PATROL and RETURN_HOME all pass it), at most 0.6 at the corner point (step
   3), then the point's own h_e6564e8a when that's over 0 and the radius over 0.5 (so a point
   with its own radius keeps it at a corner too), at most 1.0 inside one of the level's prisms
   (brain+0x3c4 bit 10).
2. **Reached**: within the radius across (x, z); under 1 m also when the character passed beside
   it (FUN_00025870: within the radius of the line it moved along since its last update, short of
   its end).
3. **The corner rule**: reached at a point that isn't a door's or a jump's, with a radius over
   0.7, the corridor from the character (at the point's height + 0.08) to the next point (+ 0.08)
   blocked: the point is the corner point and the walk goes on toward it, which brings the
   radius to 0.6 (or keeps the point's own), so it closes on the corner before it turns.
4. **The point is reached**: its kind's own step is the caller's (doors FUN_00026820, patrol
   points FUN_00026940, jumps FUN_00026520 / FUN_000267b0: `Step::reached` reports the kind).
5. **The next entry** is the target; **the look-ahead skip**: when it's a plain world point
   (kind 0) without bit 4 and the one after it has no bit 4 either, and the corridor from the
   character (at the reached point's height) to that one is clear, the walk goes to it at once,
   the skipped point kept (route+0x24). This update's target stays the skipped one's.
6. **Past the end**: the last point's destination bit cleared, the route reset; the walk is over.

The steering is the caller's: in the game the AI steers for the target through its avoidance
(FUN_000255d0) and its move (FUN_0004d970).

## Lava

Nothing in the route code sees lava: the corridor ray passes through liquids, and no world point
lies over a harmful one. The graph's layout keeps routes off it. The tutorial's lava yard: its 42
world points run along the walkways at y 15.8-16 (0.5 m over the floor), the chain between the
two pits h_f5015ea8 (38.1, -29.1) -> h_1862fb9d -> h_0a6c9ab1 -> h_19d4b7b6 (46.4, -20.0) with
links of 3.6-9 m. The walkway's points from h_19d4b7b6 round to h_fe0e9b66 all have h_ed6a7490
(so the walk never skips one) and, on to h_1a6a0ec5, their own 1 m radius; the walkway's edges
are lined with hinters (blockers only the AI meets), which the corridor rays meet when a
character stands off the line of the walk. So a route there goes point by point along the
walkway; the straight line between its ends crosses a pit.

## Test hooks

* `BF_ROUTE_LOG=1` (bf_play): the graph at the map's start (points, links, one-way links, no-skip
  points), each waypoint trigger's change to a point, and for `BF_TEST_ROUTE` the plan (from
  where, the own point, the destination, each entry with its kind, no-skip and own radius, the
  dropped first entry, the search's cost), the own point's changes and how, and each step of the
  walk that does something (a corner held and what its two rays meet, a point reached and within
  what radius, a point skipped, the end).
* `BF_TEST_ROUTE=<x>,<z>[,<s>]` or `<h_xxxxxxxx>[,<s>]` (bf_play): the controlled character
  walks a route at 2 m to the valid point nearest (x, the floor there, z), or to the named point,
  from time s (BF_TEST_GO_AT's, 1 s by default). Use it with a `BF_TEST_GOTO` that stands still
  (its goal its start). Its own point is kept every frame; it runs straight at each steering
  target (`BF_TEST_WALK` / `BF_TEST_SPRINT` as for `BF_TEST_GOTO`) and walks while the corner rule
  holds it: the controlled character's run turns in a circle of some 0.7 m and would go round a
  point with a 1 m radius of its own instead of closing on it (the hook's choice; the AI's pace
  near a point, FUN_0004d840, isn't read). The camera faces the way it goes, or keeps
  `BF_CAMERA_YAW`; with `BF_CAMERA_PITCH` and `BF_CAMERA_DISTANCE` it looks from above. The
  level's prisms aren't read, so no radius is held to 1 m indoors. **Run it with
  `BF_LEVEL=<the map>`**: bf_play loads e01's level data before the map's (its default audio,
  `load_game`) and a collision surface loaded first wins its name (`Game::add_objects`), so 11
  of the tutorial's 24 blocker surfaces (5 of them hinters), 9 of sdm_e34's 14 and 5 of e34's 17
  take e01's shapes, and the corridor rays meet phantom hinters on the lava yard's walkway (the
  walk then holds at h_1862fb9d for good). `BF_LEVEL` takes that audio from the map itself.
* `BF_ROUTE=<x>,<z>;<x2>,<z2>` (bf_level): the route a character standing at (x, the floor, z)
  takes to the valid point nearest (x2, the floor, z2), the graph as the level starts (the
  waypoint triggers' FIRST_UPDATEs applied), point by point with the floor under each, the
  dropped first entry, the length along it and straight, the search's cost; then it exits.
  `BF_ROUTE_COST=1` also times the search between every 16th point and every 16th, as the level
  starts and with every point valid; `BF_ROUTE_RAYS=1` prints what each leg's corner rays meet,
  from the point and from 0.5 and 1 m round it (`=2` also maps the 2 m square round each point,
  a 0.1 m cell each: where the corner rule would let the walk on); `BF_ROUTE_PROBE=ax,ay,az,bx,
  by,bz` what one ray from a to b meets (a corridor ray from `BF_ROUTE_LOG`'s corner line,
  replayed).

```sh
# the lava yard's route, both ways (bf_level)
BF_ROUTE="48.7,-14.1;73.1,-44.6" cargo run --bin bf_level -- tutorial
BF_ROUTE="73.1,-44.6;48.7,-14.1" cargo run --bin bf_level -- tutorial
# a route between levels on sdm_e34; the cost on e34
BF_ROUTE="-49.9,43.0;2.0,-15.2" cargo run --bin bf_level -- sdm_e34
BF_ROUTE_COST=1 BF_ROUTE="-60,-70;-10,-20" cargo run --bin bf_level -- e34
# Tex walks the yard's route, seen from above (bf_play)
BF_LEVEL=tutorial BF_MAP=tutorial BF_TEST_GOTO=48.7,-14.1,48.7,-14.1 BF_TEST_ROUTE=73.1,-44.6 BF_ROUTE_LOG=1 BF_CAMERA_YAW=0 BF_CAMERA_PITCH=-1.0 BF_CAMERA_DISTANCE=16 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=300 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# and up from sdm_e34's valley to its high ground
BF_LEVEL=sdm_e34 BF_MAP=sdm_e34 BF_TEST_GOTO=-49.9,43.0,-49.9,43.0 BF_TEST_ROUTE=2.0,-15.2 BF_ROUTE_LOG=1 BF_CAMERA_PITCH=-0.6 BF_CAMERA_DISTANCE=10 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=360 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
```

## Verified

* Unit tests (`cargo test`, `bf::nav::tests`): links one way as stored; invalid and avoided
  points gone round, and no route when every way is through them; the start's first neighbour
  exempt from the valid bit; a closed point opened again (a chain of links under 0.5 m, g 0,
  reaching it after it came off the heap: it comes off twice and the route goes the chain's way);
  g truncated (two 2.49 m links, 8, beat one 4.6 m link, 9); the radius limit; the first-entry
  drop with the corridor clear and kept with it blocked, and the one-entry route; the walk's
  radii, the look-ahead skip and its NO_SKIP stop, the corner rule's hold to 0.6 m, the end's
  reset; the pass test; the own point's first pick, its deeper search, the near walk within the
  0.25 s, and the far pick; the heap's order.
* The tutorial's lava yard (`BF_ROUTE`, both ways): from h_0e4f1e60 (48.7, -14.1) to the
  terrorists' end of the yard, h_187143b0 (71.0, -44.7), 12 entries: h_19d4b7b6, h_0a6c9ab1,
  h_1862fb9d, h_f5015ea8, then the walkway along the south pit to h_ee1bb73f and h_187143b0;
  60.5 m along it for 37.8 m straight across the pit; the own point dropped (the corridor to
  h_19d4b7b6 clear). Back the other way: h_f5015ea8 -> h_1862fb9d -> h_0a6c9ab1 -> h_19d4b7b6,
  62.1 m. Every leg's corridor is clear from its point (`BF_ROUTE_RAYS`); 1 m off the walkway's
  line the rays meet its hinters.
* sdm_e34 (`BF_ROUTE`): from the valley (-49.9, 43.0, y 30.3) to the high ground at (2.0, -15.2,
  y 40.2), 21 entries up the ramp (31.4, 32.8, 34.4, 36.6, 38.2 m ...), 95.0 m for 78.6 straight.
  To the sniper walkway: no route (the island above).
* e34: from the start valley (-60, -70) to the plateau at (-10.4, -16.9, y 38.7), 83 entries,
  332 m for 73 m straight: the graph reaches the plateau only from the north-east, through two
  door points. The cost above.
* In bf_play (`BF_TEST_ROUTE`, the commands above, captured at 15 fps): on the tutorial the
  waypoint triggers' ten changes logged at t 0.00 (and the save point's, kind 0x1d, not kept, at
  0.07); the plan at 1 s the same route as bf_level's (33-69 us); Tex runs h_19d4b7b6 (2.33 s),
  h_0a6c9ab1, h_1862fb9d (4.00 s), h_f5015ea8 and h_e507cadc (each held a frame at the corner,
  a hinter 1.2 / 2.0 m along the right ray), h_ec00fa4a ... h_e4b186f7 (10.07 s), skips
  h_1a6a0ec5 (neither it nor the next has h_ed6a7490, and the corridor on was clear), and ends by
  h_187143b0 at 13.20 s, the walkway's points each reached within its own 1 m and the last two
  within 2 m; his own point follows by the 6-link search. From above (yaw 0,
  pitch -1, 16 m) he crosses on the dark strip between the two lava pools at 3-5 s and follows
  the walkway round the south pit, never in lava. On sdm_e34 he climbs the ramp from the valley
  (y 30) to the high ground (y 40) in 19.7 s, the look-ahead skipping eight points (sdm_e34 has
  only 24 no-skip points) and the corner rule holding him to 0.6 m of h_1195e686, where a placed
  object stands in the next leg's left ray. Without `BF_LEVEL` (e01's shapes for the shared
  surfaces) the tutorial's walk holds at h_1862fb9d from 4.9 s to the end of a 20 s capture,
  the left ray meeting a hinter that bf_level's collision doesn't have: the same two rays
  replayed in bf_level (`BF_ROUTE_PROBE`) meet nothing.

## Not done

* The squad on routes (#335: the follow's chase and its five distances, Move To, the stuck
  detour FUN_000426b0 and the resync FUN_00041ff0, the blocker route FUN_00041870 with bit 3),
  the enemies' (#336: RETURN_HOME, PATROL's first leg FUN_0004c2e0, PURSUE, the attack approach
  with its radius budget FUN_0002e800), the script 105 walks (#337, an open code question).
* FUN_00059f70's and FUN_00059d40's temporary points; the 20 m look at a route's end (route+0x14,
  FUN_0001fdd0); the kinds' own steps at a point reached (doors, patrol points, jumps); the
  prisms (bit 10); the avoidance FUN_000255d0; the AI's pace near a point (FUN_0004d840).
* Kind 0x13's and the save points' set-valid (kept by the game elsewhere); the cover points' bit 5
  is kept (`Node::cover`) for #336's cover goals.
* Footage: no take shows a route yet (todo/242-ai-routes/shotlist.md).

## Corrections to todo/242's leads

* g is truncated to a whole number at each step (0x591e1), and an open point's f falls in place
  without a sift (the leads: "g += 2 x the link length").
* FUN_000413f0's radius is 40 m (0x42200000), not 10, and as read it links the destination to
  itself (no effect).
* FUN_00202ba0's kind 10 is the cover point (its +0x64 bit 5), not a path: sdm_e34's fourteen
  cover-point triggers don't touch the graph.
* FUN_0014f860: the 1-2 link walk runs every frame (one link, two every eighth frame by the
  +0x250 count) and measures from the character's position; the 0.25 s timer gates the deeper
  search, and y - 0.25 is only the height test.
* FUN_00025730's own-point insertion is for a route that starts at a landing (0x1b / 0x1c) from a
  take-off (0x1a).
* FUN_00026d80: the point's own radius replaces the corner's 0.6 (it's tested after it), so a
  point with its own radius over 0.7 keeps the corner rule's hold until its corridor is clear.
* +0x28 bit 4 is "the h_ed6a7490 list isn't empty" (FUN_0001bb00 tests the vector for empty).
