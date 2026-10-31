# AI navigation: world points and routes (`src/bf/nav.rs`, `src/bin/play_nav.rs`, #334)

How the game's AI finds its way: the level's world points as a graph, the route search over it,
where a character's routes start, and how a route is walked. The first slice of #242 (AI
routes, #334) is the shared machinery; since the second (#335) the AI squad's follow and its Move
To walk routes ("The squad on routes" below, `src/bin/play_routes.rs`). The enemies' goals are
#336, the script walks #337; the Move To cursor's filter takes a point's valid bit from the graph
([squad.md](squad.md)).

Everything is read from default.xbe (static, not run): the route request FUN_000416a0, the search
FUN_00059000, the own point's upkeep FUN_0014f860, the walk FUN_00025730 / FUN_00026d80, the
waypoint trigger 0x202c50; every function is in [code-map.md](code-map.md), "AI". The reading is
`todo/242-ai-routes/leads.md` (the analyst's), with the corrections at the end of this page. take01 of
`todo/242-ai-routes` logs the game's squad routes round a lava pit ("The squad on routes").

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
| 0 | claimed | a waypoint trigger's h_06d92de9 (every one in the data is -1); the follow's state 4 on the point it picks near the leader (FUN_000225d0; the demo's follow keeps its formation, #331) | GOAL_IDLE's start, the point filters |
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

## The squad on routes (#335)

`src/bin/play_routes.rs` (`Router`, one per character, `Player::router`), from play.rs
`squad_ai` (the follow, FOLLOW and WINGMAN, and the marines' brains), play_orders.rs `move_out`
(Move To) and play.rs `update_player` (the own points and the AI update's route steps). Every
function is in [code-map.md](code-map.md), "AI".

* **One route record.** A brain walks one route, brain+0x930. The goals reach it as the goal
  holder's +0x8f0 (the holder is brain+0x40): MOVE_OUT's update FUN_00045fa0, PATROL's
  FUN_0002b490 and RETURN_HOME's FUN_00034690 hand FUN_00026d80 `in_ECX + 0x8f0` with `in_ECX -
  0x40` the brain; the follow's states use brain+0x930 itself (FUN_00022130). So the AI update's
  route steps, which run on brain+0x930 for whatever goal walks it, apply to Move To as to the
  follow (and to the enemies' routes, #336). (todo/242's leads called the goals' route brain+0x8f0;
  #242's xemu notes read no route there.)
* **The own points** (FUN_0014f860) of the controlled character (its brain idles under the
  player, but a route to a character ends at its point: the follow's flood starts there) and of
  the AI squad and its followers, every frame before the AI (`keep_point`).
* **The follow.** The demo keeps its formation places, set-off (3 m), settle (1 m) and dash (8 m)
  distances until #331 brings the game's follow states (FUN_00020b10), but the way to the place
  is a route, never a straight chase:
  * its destination: the point nearest the place of the points the leader's own point floods to
    within 12 m (FUN_000594a0 with FUN_000225d0's radius, the follow's state 4 picking its point
    near the leader; the demo takes the nearest to its place where state 4 takes the first its
    filter takes), so a point on the leader's side of a pit; and the place itself beside that
    point (`Dest::Beside`: FUN_0005a440's FUN_00059d40 temporary, linked both ways to the point
    and its neighbours, the route ending at the position) when the place stands on a floor no
    more than 2 m under the leader's feet, in no liquid that burns, the 0.8 m corridor from the
    point to it clear; else the point (it stands there, as near as the graph comes, until its
    place moves 1 m);
  * planned again when the place is more than d3 (9.8 m, 7 in WINGMAN: FUN_00020b10's +0x14)
    from where its route was planned to (FUN_00022130 plans again while the leader is farther
    than d3 from the route's end; the place stands in for the leader), and at the route's end
    short of the place (state 0 with its route empty plans);
  * no route found: it stands (state 2, the hold) until the place moves 1 m (the demo's: a
    search costs under 50 us, so a leader on the move is followed without a pause);
  * walked at 2 m a point (FUN_00026d80 at 2.0, the follow's), its last entry, the place, within
    the formation's 1 m (0.3 m out of the leader's line of fire);
  * the pace: the dash when the way left (along the route) or the place itself is over 8 m off,
    a walk when both are under 2.5 m and while the corner rule holds it (as `BF_TEST_ROUTE`'s
    walk: the run turns wider than the corner's 0.6 m; the AI's own pace near a point,
    FUN_0004d840, isn't read).
  Without a level's points (the test map's floor) the follow is the straight walk it was.
* **Move To** (GOAL_MOVE_OUT: its start 0x45d00, its update FUN_00045fa0): for each order a
  route to the point (brain+0x8ac: the cursor's point, which the order's position is; for a
  position that isn't one, `BF_TEST_ORDER`'s, the valid point nearest it that the cursor would
  take: +0x24 bit 14 set, bit 15 clear; the hand-over's, the one left's own point), walked at 2 m
  at the dash to its end, then GUARD and e635ecc6. No route: it stands in MOVE_OUT (the update's
  empty route asks FUN_0004d970 for mode 1). `pass_round` and the 0.1 m / 1 s stall guess are
  gone; the test map's floor keeps a straight walk with that stall rule (no points to route
  over).
* **The AI update's route steps** (FUN_0001afe0, 0x1b17f-0x1b18b, for a character with an own
  point, before the goal machine), `ai_update`:
  1. **The blocker** (FUN_00041870, `blocker`): with a route, each friend (not hostile, alive)
     within 8 m (3D) whose speed cap is under 3, whose own point is the member's current point,
     its own point or the point its look-ahead skipped, and that lies its way (the x and z of the
     unit directions to it and to the current point, dot over 0.75), gets its own point marked
     avoid (bit 3). With one, and the member or the first such friend inside one of the level's
     prisms (brain+0x3c4 bit 10, FUN_0004cb60), or that friend's brain+0x3c8 set (not modelled):
     the friend's point the route's last: the last entry dropped and the route done
     (FUN_00040f70, route+0x20); else the flood from the member's own point to the route's next
     point, within 15 m, round the marked points (FUN_000594a0; its filter, 0x41070, takes the
     next point alone: its inner test 0x262a80 returns 1); found, the route is the flood's way
     and then the old route's entries after the first entry of that point (FUN_00041810),
     started from its first entry with the speed cap at most 3 for 0.5 s; none and the friend a
     human player's (FUN_00112800), the member stands (cap 1 for 0.5 s, brain+0x9c8 bit 0) and
     counts brain+0x370, and at 7 s in MOVE_OUT it's GUARD with 12d51a1c (silent: no block in
     any chatter file). Outside the prisms nothing but the count's reset. The marks are cleared
     at its end: they only steer the flood. The demo doesn't read the prisms, so this runs only
     under `BF_TEST_PRISM=1`; in open ground the game's members pass each other by their
     avoidance (FUN_000255d0, not done), as #33 take08's Hawk ran past Tex.
  2. **The catch-up** (FUN_00041ff0, `Walker::resync`): the character's own point being the point
     of the entry after the current one or the one after that, the index moves there (the later
     when both: each match overwrites).
  3. **Stuck** (FUN_000426b0, `stuck`): with a route and the speed cap at 2 or more, the squared
     3D distance to the current target not under the best since the target was set adds the
     frame's time; under it, the best and the time 0 (a cap under 2, after a detour or a
     re-route, holds the time at 0). Past 2 s, the time 0 again, and: the speed factor (char+0x720,
     FUN_00048270's, which the follow's FUN_00020b10 keeps at 1.25 far from the leader) at 0.01 or
     under put back to 1 and nothing more (the demo has no speed factor); else the own point the
     same as at the last stuck, counting: the third in a row puts the character on its own point
     (FUN_000425b0: the point's position + 0.25 m, then FUN_0004c5e0; medium, read as a
     set-position; here its feet on the floor under it); else the detour (FUN_00042280): the
     current entry the last, the route done; else, the current point valid, it and the own
     point lose bit 2 (valid) for one search, the next entry the last, done, else the next point
     loses it too and a new route to the destination is planned (FUN_00041660) and started with
     the cap at 1 for 1 s and route+0x1c -20 (moot: the cap's 1 holds the time at 0), the bits
     given back (FUN_00042240); only a search that finds nothing goes on to the re-route
     (FUN_00042490: a plain new route to the last point, the cap at 1 for 1.5 s, +0x1c -5).
     FUN_00042620's GOAL_ATTACK_GUARD between them is for a brain out of the squad.
* **The speed cap** ([brain+0x1a8]+0, `Router::cap`): FUN_0004d970 moves at the lower of the mode
  asked (+4) and it; the holds above set it with a timer (+0x28, FUN_000124f0). What restores it
  when the timer runs out isn't found: the demo lifts it (inferred). The follow and Move To
  stand at 1, walk at 2, don't dash at 3.
* **The liquid guard** (play_liquids.rs `burning_ahead`): a safety net only, the routes' points
  lying on no harmful liquid. A member whose next step would go into one holds there;
  `BF_LIQUID_LOG` says so ("on its route (the routes' safety net)"). In the runs below it never
  acted.
* **The demo's choices**, labelled in the code: the formation place and its point (above); the
  walk at a corner hold; the pace by the longer of the way left and the place's distance; an
  equal distance counting as no closer in the stuck test while the member asks to move (the
  game starts its time again on an equal one, but the demo's collision holds a pinned character
  at exactly the same spot, or cycles it through the same few, where the game's physics never
  repeats a float: with the game's test a character held against a wall was never stuck; one
  standing by its goal's choice, a Toggle Ability's hold, still isn't); the controlled character's
  speed cap read from its stick (it walks or stands: under 3) for the blocker; no prisms.

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
  level's prisms aren't read, so no radius is held to 1 m indoors.
* `BF_ROUTE_LOG=1` also logs the AI squad's routes (#335): each plan and why (the follow, its
  place moved, the route's end short of its place, Move To, the detour, the re-route), from
  where, the own point, the destination (a point, or the place beside one), the entries, the
  dropped first entry and the search's cost; each point reached, corner held, point skipped,
  catch-up, stuck, detour, put-at-the-point and blocker step; `=2` also each one's own point's
  changes and, once a second, its walk's state (the entry, the target, the radius, the distance,
  the corner, the stuck time and best, the speed cap).
* `BF_TEST_WARP=<s>,<who>,<x>,<z>[,<heading>][;...]` (bf_play): at time s `who` (`me`, the
  controlled character, or a member by name or slot, as `BF_TEST_ORDER`) is put on the floor at
  (x, z), facing the heading (degrees, 0 = -z) or as it faced: the takes' trainer `tp`.
* `BF_TEST_PRISM=1` (bf_play): every character counts as inside one of the level's prisms
  (brain+0x3c4 bit 10, whose records the demo doesn't read): the walk's radius at most 1 m, the
  dash a run, and FUN_00041870's route round a friend, which only runs then.
* `BF_ROUTE_NO_HINTERS=1` (bf_play, a diagnostic): the corridor rays leave the hinters out, which
  the code as read doesn't (FUN_0004ce30's filter, +0x10c = 8, leaves out PID 3 alone): to see the
  routes without the hinter bands the demo's members hug (e13 below).
* `BF_ROUTE=<x>,<z>;<x2>,<z2>` (bf_level): the route a character standing at (x, the floor, z)
  takes to the valid point nearest (x2, the floor, z2), the graph as the level starts (the
  waypoint triggers' FIRST_UPDATEs applied), point by point with the floor under each, the
  dropped first entry, the length along it and straight, the search's cost; then it exits.
  `BF_ROUTE_COST=1` also times the search between every 16th point and every 16th, as the level
  starts and with every point valid; `BF_ROUTE_RAYS=1` prints what each leg's corner rays meet,
  from the point and from 0.5 and 1 m round it (`=2` also maps the 2 m square round each point,
  a 0.1 m cell each: where the corner rule would let the walk on); `BF_ROUTE_PROBE=ax,ay,az,bx,
  by,bz` what one ray from a to b meets (a corridor ray from `BF_ROUTE_LOG`'s corner line,
  replayed), and every triangle it crosses, which side it meets (the hinters' faces come in
  front and back twins); `BF_ROUTE_FACES=x0,z0,x1,z1` prints the blockers' and hinters'
  triangles in that box, to map them round a route's points.

```sh
# the lava yard's route, both ways (bf_level)
BF_ROUTE="48.7,-14.1;73.1,-44.6" cargo run --bin bf_level -- tutorial
BF_ROUTE="73.1,-44.6;48.7,-14.1" cargo run --bin bf_level -- tutorial
# a route between levels on sdm_e34; the cost on e34
BF_ROUTE="-49.9,43.0;2.0,-15.2" cargo run --bin bf_level -- sdm_e34
BF_ROUTE_COST=1 BF_ROUTE="-60,-70;-10,-20" cargo run --bin bf_level -- e34
# Tex walks the yard's route, seen from above (bf_play)
BF_MAP=tutorial BF_TEST_GOTO=48.7,-14.1,48.7,-14.1 BF_TEST_ROUTE=73.1,-44.6 BF_ROUTE_LOG=1 BF_CAMERA_YAW=0 BF_CAMERA_PITCH=-1.0 BF_CAMERA_DISTANCE=16 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=300 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# and up from sdm_e34's valley to its high ground
BF_MAP=sdm_e34 BF_TEST_GOTO=-49.9,43.0,-49.9,43.0 BF_TEST_ROUTE=2.0,-15.2 BF_ROUTE_LOG=1 BF_CAMERA_PITCH=-0.6 BF_CAMERA_DISTANCE=10 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=360 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
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
  object stands in the next leg's left ray.
* #338: bf_play loaded e01's data before the map's, and a collision surface loaded first wins its
  name, so on every map but e01 a blocker or hinter named as one of e01's had e01's shape (the
  tutorial's five hinters, 5 of sdm_e34's 14 blockers, 1 of e34's 17: maps.md, "Which level's
  shapes"). These walks had to run with `BF_LEVEL=<the map>`, which takes the default data from
  the map itself: without it the tutorial's walk held at h_1862fb9d from 4.9 s to the end of a
  20 s capture, the left ray meeting a hinter of e01's shape that bf_level's collision doesn't
  have (the same two rays replayed in bf_level, `BF_ROUTE_PROBE`, meet nothing). The map's own
  data loads first now: the commands above run as written, the tutorial's walk ends at 13.20 s
  and sdm_e34's at 19.67 s (eight points skipped), and the tutorial's with `BF_LEVEL=tutorial`
  gives the same log line for line (the search's microseconds aside) and the same frames to the
  noise.

* **The squad on routes (#335)**, `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps; the e13
  staging of `todo/242-ai-routes` take01 (Tex on the north rim facing south, the three on the
  south rim): `BF_MAP=e13 BF_TEST_GOTO=37.3,25.6,37.3,25.6 "BF_TEST_WARP=0,me,37.3,25.6,180;
  0,brutus,44.6,73.8;0,flint,43.1,75.8;0,hawk,45.7,75.6" BF_TEST_NO_HOSTILES=1 BF_ROUTE_LOG=1
  BF_ORDER_LOG=2 BF_LIQUID_LOG=1 BF_CAMERA_PITCH=-1.2 BF_CAMERA_DISTANCE=70`, 600 frames:
  * each member's first route goes round by the west shore through the take's points in the
    take's order, h_0f143f8f (37.3, 73.6), h_141d6e35, h_1fe8f610, h_fb8c63b3, h_f079fb96 (9.3,
    73.6), h_e9ba9a68, h_f2b3cbd2, h_029e248d (2.3, 57.6), h_efb7f462, h_f3024e36, h_1df21e91,
    h_1c93b982, h_ef15c18a, h_f6599a9a, h_1469ef23, h_1b3a3faf, h_e41a0444, as the game's three
    (notes.md: Brutus's and Hawk's 20 points, Flint's 19), then to its own formation place south
    of Tex (the game's end at Tex's point, h_f6146568); bf_level's route from Brutus's spot to
    Tex's point is the take's 20 to the point, 116.8 m for 48.7 straight. Two differences: the
    game's three keep their own point as the first entry, which the demo drops (both corridor
    rays clear from Brutus's spot in the demo: `BF_ROUTE_PROBE`; what the game's met isn't
    found); and the corners;
  * the corners: the game held its members (take01's brain+0x9dc) at h_f079fb96, h_e9ba9a68,
    h_029e248d and h_1469ef23, as the demo does, but not at h_f2b3cbd2 or h_f3024e36, where the
    demo's rays meet e13's hinter band h_09910e17. The band runs along the west walkway with
    both points just beyond it (h_f2b3cbd2 0.21 m, h_f3024e36 about 1.5 m: `BF_ROUTE_FACES`);
    the game's members kept about a metre off it (their avoidance, FUN_000255d0's feelers, not
    done) and the game's Flint skipped h_f3024e36 from h_efb7f462 (his steering target h_1df2
    from 17.08 s), where the demo's, steering straight at the points, slide along the band, are
    held at those corners and stop 0.6 m short of them against it. The stuck handling then moves
    them on: Brutus's detour at 8.5 s takes the -0.7 m lane (h_1021c864, h_ee8f64b4, h_f79d787b,
    h_0ec4ef6b, h_eaf9b73f, h_fec3851a, h_ef0679a7), the lane the game's Brutus took after his
    own detour at 16.9 s, and he is at his place at 18.6 s; Hawk and Flint are detoured twice at
    a corner and put on its point the third time (FUN_000425b0), about 10 s a corner, and are
    still on the way at 40 s (the game's three were there in 25-32 s). With
    `BF_ROUTE_NO_HINTERS=1` the holds match the game's but for the brief ones at h_f079fb96,
    h_e9ba9a68 and h_029e248d, h_f3024e36 is skipped from h_efb7f462 and h_f2b3cbd2 reached at
    2 m as the game's were, and all three arrive in 14-18 s;
  * none enters lava: `BF_LIQUID_LOG` never fires (the routes' safety net never acts).
* Move To across the pit (e13, the squad and Tex on the south rim, `"BF_TEST_ORDER=hawk,move,
  34.3,29.6@2"`): a 19-entry route round the west shore to h_e7a4bd37 on the north rim, the same
  corners holding her; never into lava. On e34 (#318's take08 recipe with Tex on her way,
  `BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999
  "BF_TEST_ORDER=hawk,move,-89.5,-87.8@1;hawk,move,-70.8,-79.5@10"`): a 5-entry route past Tex,
  at h_ff113737 3.13 s after the order, GUARD 1.8 m short (#318: 3.1 s and 1.8 m; take08 4.2 s,
  1.9-2.0 m).
* The follow on e34 (`BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4 BF_TEST_SPRINT=4`,
  375 frames): a plan each time a place moves 9.8 m from where its route goes, the members 2-12 m
  off Tex on the way, settling 5.5-9 m off and kneeling, no stuck; Stand Ground holds where
  ordered while Tex goes 32 m off, Fire At Will and Cover Me follow (`"BF_TEST_ORDER=hawk,guard@3;
  flint,fire@3"`). sdm_e13 (#1's recipe, the controlled Brutus walking into the lava): the three
  stay on the shore's points, 8-9 m from him. The tutorial's marines (brain.md's recipe) follow
  Tex on routes.
* The cost (the plans' times in the logs, opt-level 2): 4-45 us a search, at most 225 points off
  the heap (e13), the worst frame three plans in 90 us.
* `BF_TEST_PRISM=1` (e34, Move To with Tex on Hawk's line, a squadmate on the other way round):
  the blocker route runs; with two friends standing on the two ways round it, as read, swaps
  between them every update, each new route starting from the member's own point, which holds
  it there, and growing by the flood's way each time (FUN_00041810 appends the old route's tail
  after the first entry of the next point).
* Unit tests (`cargo test`): the route to a position beside a point (through the point or a
  neighbour, the position alone from the point itself); the flood's order (a link further each
  pass, the last met first, the radius, the avoided points); the catch-up; the stuck detour (the
  three points off for one search, the bits back, the cap) and the third stuck's warp; the
  follow's point on the leader's side.

## Not done

* The enemies on routes (#336: RETURN_HOME, PATROL's first leg FUN_0004c2e0, PURSUE, the attack
  approach with its radius budget FUN_0002e800), the script 105 walks (#337, an open code
  question). The follow's own states (FUN_00020b10: its five distances, state 4's point near the
  leader with the claimed bit, the speed factor): #331. The AI's avoidance (FUN_000255d0, its
  feelers FUN_00018c40 and their speed caps) and the per-type capsule (#173: the squad's lf 0.35
  m against the demo's 0.4) are what would keep the members off e13's hinter band as the game's
  are. FUN_0001ad50 (the cap to a walk after a second of hard turning). The prisms
  (FUN_0004cb60's records) and brain+0x3c8.
* FUN_00059f70's and FUN_00059d40's temporary points; the 20 m look at a route's end (route+0x14,
  FUN_0001fdd0); the kinds' own steps at a point reached (doors, patrol points, jumps); the
  prisms (bit 10); the avoidance FUN_000255d0; the AI's pace near a point (FUN_0004d840).
* Kind 0x13's and the save points' set-valid (kept by the game elsewhere); the cover points' bit 5
  is kept (`Node::cover`) for #336's cover goals.
* Footage: take01 (e13, the squad round the pit, its routes logged) is the one take; Move To across a pit, the tutorial's lava yard and the blocker and stuck shots (shotlist.md A1, A3-A5, B, C) aren't filmed.

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
* (#335) The goals' route is the goal holder's +0x8f0, brain+0x930, the follow's own record, not
  brain+0x8f0 (FUN_00045fa0, FUN_0002b490, FUN_00034690 pass `in_ECX + 0x8f0` with `in_ECX - 0x40`
  the brain); +0x918 is the holder's too (brain+0x958, PATROL's chain).
* FUN_00041ff0 looks at the next two entries (index + 1 and + 2), not three.
* FUN_000426b0's char+0x720 is the speed factor FUN_00048270 writes (brain+0x8a0 when active, else
  1), not the character's speed: its "standing" case is a speed factor at 0.
* FUN_00041870 builds its way round (and its 7 s wait) only with the member or the friend inside
  a prism (brain+0x3c4 bit 10) or the friend's brain+0x3c8 set, and clears the avoid bits it set
  at its end; its flood goes to the route's next point (filter 0x41070) and FUN_00041810 joins the
  old route's tail after it.
* FUN_00042490 re-routes to the route's last point (0x42497), and FUN_00042280 returns true but
  for a search that finds nothing.
