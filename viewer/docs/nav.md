# AI navigation: world points and routes (`src/bf/nav.rs`, `src/bin/play_nav.rs`, #334)

How the game's AI finds its way: the level's world points as a graph, the route search over it,
where a character's routes start, and how a route is walked. The first slice of #242 (AI
routes, #334) is the shared machinery; since the second (#335) the AI squad's follow and its Move
To walk routes ("The squad on routes" below, `src/bin/play_routes.rs`), steered through the AI's
avoidance since #344 ("The steering", `src/bin/play_avoid.rs`). The enemies' goals are
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
   points FUN_00026940, jumps FUN_00026520 / FUN_000267b0: `Step::reached` reports the kind). The
   squad's routes take the door step since #346 ("The door step" below).
5. **The next entry** is the target; **the look-ahead skip**: when it's a plain world point
   (kind 0) without bit 4 and the one after it has no bit 4 either, and the corridor from the
   character (at the reached point's height) to that one is clear, the walk goes to it at once,
   the skipped point kept (route+0x24). This update's target stays the skipped one's.
6. **Past the end**: the last point's destination bit cleared, the route reset; the walk is over.

The steering is the caller's: in the game the AI steers for the target through its avoidance
(FUN_000255d0) and its move (FUN_0004d970); the squad's since #344 ("The steering" below).

## The squad on routes (#335)

`src/bin/play_routes.rs` (`Router`, one per character, `Player::router`), from play_follow.rs
(the follow, FOLLOW and WINGMAN, through play.rs `squad_ai`, and the marines' brains), play_orders.rs `move_out`
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
  player, but a route to a character ends at its point: the follow's chase goes there and its
  cover search floods from there) and of
  the AI squad and its followers, every frame before the AI (`keep_point`).
* **The follow** (#331, play_follow.rs; the states and the formation in [squad.md](squad.md) "The
  follow"): GOAL_WINGMAN_FOLLOW's states (FUN_00020b10) walk two kinds of route on brain+0x930:
  * **the chase** (state 0, FUN_00022130): with no route, a route to the leader's place
    (FUN_000416a0 with FUN_0011dfa0's places: to his own point, or to his position when the
    member's own point is his: `Dest::Character`), started (FUN_00025730); with one, planned again
    while the leader is farther than d3 (9.8 m, 7 in WINGMAN) from its last entry (its point's
    position, else its own: 0x22251-0x222d7); walked at 2 m a point (FUN_00026d80 at 2.0) at the
    dash (FUN_0004d970 mode 4, no fifth argument: no pace near its points) as far as the pace
    level allows; within d2 (9.8, 7) of a leader who isn't running (FUN_00020ad0 under 3) it's
    reset (FUN_0004e8f0) and the member stops, FOLLOW for state 4, WINGMAN to hold; at its end, the
    reset and a hold with the leader within d4 (5.6, 4), else a new route at the next update. No
    route found: a hold;
  * **the walk to a cover point** (state 4, FUN_000225d0, FOLLOW only): the flood from the leader's
    own point (FUN_000594a0 within 12 m of it) for the first `<cover-point>` (kind 10, its +0x64 bit
    5, valid) within 6 m of the member (0x20410: the filter's 12 m x 0.5), not claimed (+0x28 bit 0:
    the filter's claim test, which FUN_00020170 turns on), that no squad member has for its own
    point (FUN_00045310, FUN_00020260) and no nearer member in FOLLOW could take (FUN_00020260:
    those walking to one, state 5, or holding at one, left out; of two as near, the first in the
    squad list), claimed (+0x28 bit 0, brain+0x6c0) and walked to (state 5, FUN_00022790: 1 m an
    entry, 0.1 m the point, `step`'s last radius) at the dash, then held. The route planned, the
    turn check (FUN_00058740): its first entry more than about 75 deg off the member's facing, the
    pace level at a walk for 1.5 s (play_follow.rs `turn_check`), so a member turns at a walk to a
    point behind it;
  * **the holds** (states 2 and 3): the pace level at 1 for 0.3 s (FUN_000223c0), then until the
    leader is beyond d0 (11.2 m, 8; x1.5 with the member's own point a cover point, FUN_00022420);
    the route reset as a hold begins (the demo's, so no other goal's route is walked on).

  The stuck handling runs on these routes as on any (below): a route it ends, the chase holds at
  with the leader within d4 or plans again, as the game's; a bare position (the one-entry route to
  a leader sharing the member's own point), which FUN_000425b0 can't put it at, the demo ends.
  Without a level's points (the test map's floor) the chase walks straight at the leader with the
  same tests. Gone with #331: the demo's formation place and its points (the leader's flood
  sorted by the place, the four tried for a clear corridor, the leader's point for a place no one
  could stand on), the 3 m hold, the pace by the way left, the holds when the stuck handling acted
  within d0 (`Router::gave_up`, `stuck_fired`) and the planned-from position: the follow routes to
  the leader's own point and holds near him, as the game's, so the back-steps of #344's e34 run
  (Flint and Hawk sent to their places' points behind them) and the gathering beside a leader
  whose places couldn't be stood on are gone.
* **Move To** (GOAL_MOVE_OUT: its start 0x45d00, its update FUN_00045fa0): for each order a
  route to the point (brain+0x8ac: the cursor's point, which the order's position is; for a
  position that isn't one, `BF_TEST_ORDER`'s, the valid point nearest it that the cursor would
  take: +0x24 bit 14 set, bit 15 clear; the hand-over's, the one left's own point; with no point,
  its own point, FUN_00058b60: it walks there, to within 2 m, then the end), walked at 2 m at the
  dash (held to its pace level), slowed near the route's points (`pace_near`, #344's second fix
  round: FUN_00045fa0 calls FUN_0004d970 with its fifth argument set, 0x4627d, so FUN_0004d840
  runs: a run within 6.0 m of the route's last entry and a walk within 3.2 m, across, FUN_00058590
  with the pace record's distances for a brain out of FOLLOW and WINGMAN; else, within 3.2 m of
  the current point (3D), FUN_000587d0 by the corner on to the next: a walk for a turn sharper
  than about 84 degrees, a notch under the dash for 69-84, a walk for a gentler one while its move
  is 53 degrees or more off the way to the point within 2.4 m, each with the level's timer at 1.5
  s, after which the ramp climbs a notch a second), steered through the avoidance (#344,
  FUN_000255d0 as MOVE_OUT calls it at 0x45cb6), to its end, then GUARD and e635ecc6. No route: it
  stands in MOVE_OUT (the update's empty route asks FUN_0004d970 for mode 1). `pass_round` and
  the 0.1 m / 1 s stall guess are gone; the test map's floor keeps a straight walk with that stall
  rule (no points to route over).
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
     avoidance (FUN_000255d0's push, "The steering"), as #33 take08's Hawk ran past Tex.
  2. **The catch-up** (FUN_00041ff0, `Walker::resync`): the character's own point being the point
     of the entry after the current one or the one after that, the index moves there (the later
     when both: each match overwrites).
  3. **Stuck** (FUN_000426b0, `stuck`): with a route and the pace level at 2 or more (0x426f7;
     under 2, after a detour, a re-route or a hold, the time is 0), the squared 3D distance to the
     current target over the best since the target was set adds the frame's time; under or equal
     to it, the best and the time 0 (asm 0x42760-0x4278f: no margin, the game's since #344; #335's
     5 cm is gone, see the demo's choices; `progress`: one asking to move whose distance is its
     best's to the bit hasn't moved at all and is no closer, the demo's). Past 2 s (strictly), the time 0 again, and:
     the speed factor (char+0x720, FUN_00048270's, which the follow's FUN_00020b10 keeps at 1.25
     far from the leader) at 0.01 or under put back to 1 and nothing more (the demo's is never
     under 1); else the own point the same as at the last stuck, counting: the third in a row puts
     the character on the route's current entry's point, the one it couldn't reach
     (FUN_000425b0 with it in EAX, 0x42839-0x42857: the point's position + 0.25 m, then
     FUN_0004c5e0; medium, read as a set-position; here its feet on the floor under it. Round 1
     put it on its own point, as todo/242's leads had it: corrected from todo/344's leads §7). For
     a bare position (the follow's one-entry route to a leader whose own point is the member's,
     FUN_00022d70) FUN_000425b0 does nothing and the game's member pushes on until the leader moves
     d3 off; the demo ends the route there (route+0x20), and the follow's chase holds at its end
     with the leader within d4, else plans again (#331; until then the demo's formation place,
     `Router::gave_up`, now gone); else the detour (FUN_00042280): the
     current entry the last, the route done; else, the current point valid, it and the own
     point lose bit 2 (valid) for one search, the next entry the last, done, else the next point
     loses it too and a new route to the destination is planned (FUN_00041660) and started with
     the cap at 1 for 1 s and route+0x1c -20 (moot: the cap's 1 holds the time at 0), the bits
     given back (FUN_00042240); only a search that finds nothing goes on to the re-route
     (FUN_00042490: a plain new route to the last point, the cap at 1 for 1.5 s, +0x1c -5).
     FUN_00042620's GOAL_ATTACK_GUARD between them is for a brain out of the squad.
* **The pace record** ([brain+0x1a8]: its level +0, `Router::cap`; the mode asked +4,
  `Router::asked`): FUN_0004d970 moves at the lower of the two (`pace`: 1 stands, 2 walks, 3
  runs, 4 dashes, a dash in a prism a run), and the follow and Move To ask for what they want,
  the level holding them to it. The level is a ramp ("The steering", `pace_ramp`): it starts at a
  stand and steps a notch toward the mode asked, up and down, each time its timer runs out; the
  drops set it down with the timer: the detour 1 for 1 s, the re-route 1 for 1.5 s, the blocker
  at most 3 (or 1) for 0.5 s, the follow's hold 1 for 0.3 s (FUN_000223c0), and since #344 the
  feelers' and the hard turn's, and on a Move To the pace near the route's points (FUN_0004d840:
  the move asked lowered near the route's end, the level set down at a corner for 1.5 s; Move To
  above). (Round 1 lifted it at once when the timer ran out, inferred; until
  #344 it started at the dash and climbed only.)
* **The liquid guard** (play_liquids.rs `burning_ahead`): a safety net only, the routes' points
  lying on no harmful liquid. A member whose next step would go into one holds there;
  `BF_LIQUID_LOG` says so ("on its route (the routes' safety net)"). In the runs below it never
  acted.
* **The demo's choices**, labelled in the code: the follow's (play_follow.rs: the route reset as a
  hold begins, a stuck bare position's route ended, the straight chase on the test map's floor;
  until #331 its formation place, its points and holds, above); the blocker's slow friend, for
  the controlled character, read from its stick (it walks or stands: under 3); no prisms; and the
  avoidance's ("The steering").
  Gone with #344: the walk at a corner hold, and in the stuck test, while the member asked to
  move, 5 cm closer to count as progress (#335's stand-in: without the feelers its members pressed
  against e13's hinter band, where the collision jitters a held body by a centimetre, a new best
  every few frames, so the game's test never fired; with them the game's own test, no margin,
  moves every member on: the e13 runs below have no stuck at all, and with the feelers off,
  `BF_AVOID_OFF=1`, Hawk is pinned at the band for good). Kept from it, since #344's fix round:
  a member asking to move whose distance is its best's to the bit counts as no closer
  (`progress`). The game's characters are never held to the bit (no AI ledge refusal: its routes
  jump down between a jump's points, FUN_000267b0 with bit 12; its physics leaves a body pressed
  into a wall moving); the demo's AI ledge stop (play.rs `ledge_ahead`) puts one back exactly where
  it was every frame. The tutorial's lava yard: its route from the upper ledge goes from
  h_0656d0f4 (kind 0x1b, y 26.4) to h_03090779 (0x1c, y 16.4), a jump's two points 10 m apart in
  height, which the demo's marines can't take; held at the edge, they were never stuck and stood
  there for good; now the detour finds no other way down (its search round the three points
  fails, and the re-route plans the same way) and the third stuck puts each on the lower point
  (FUN_000425b0), 12-13 s after it reached the edge, where the game's would have dropped down at
  once (the controlled Tex, whom the ledge stop doesn't hold, walks off it).

## The steering: the AI's avoidance (#344)

`src/bin/play_avoid.rs` (`steer`), for the route walkers: the follow (play.rs `squad_ai`: the
squad's, the tutorial's marines', the test map's straight walk) and Move To (play_orders.rs
`move_out`). The reading is `todo/344-ai-avoidance/leads.md` §1-§6, checked against the asm; every
function is in [code-map.md](code-map.md), "AI". The enemies' goals walk no routes yet (#336) and
aren't steered.

**FUN_000255d0** makes the direction of every walking goal from its steering target (the route's,
brain+0x478; none for a target at the world's origin) in four stages and takes the first that
steers: the target's direction only when nothing is in the way. A stage that steers is the whole
direction (no blend): the stages write into the AI update's direction buffer, which is zero when
they run (zeroed before the goal machine, 0x1b338, and no walking goal writes the target's
direction into it first: the route walker FUN_00026d80 writes the target's position), so the
push, which adds into it (0x523fd), gives its sum alone (#363's reading; #344 read the stage as
overwriting, the same in effect).

1. **The feelers** (FUN_00018c40, `feel`): three rays along the character's heading (iface +0x34:
   its commanded move, the stick the last AI frame gave it, else its body's forward; #363, which
   put right #344's reading of the speed it makes) from its hips 0.25 m lower and
   0.5 m back: the centre 3.15 m, the left and right 2.0475 m at 25 degrees (the brain's list,
   brain+0x354, which the constructor fills and nothing changes). Their lengths: x0.6 at a walk,
   x1.6 at a dash, and x0.9 x the speed factor char+0x720 when that's over 1 (the follow's boost:
   x1.125); off at a stand, and while brain+0x924 runs (the walk's door step sets it, "The door
   step"). Each ray is tilted by the
   ground ahead (FUN_00115b00: the height change to the ground 0.8 m ahead along the move, its y
   added before it's normalised). They meet the terrain, the placed objects, the blockers, the
   hinters and the fly-blockers, never a character, an item or a liquid (the corridor ray's
   filter; `Arena::ray_kind_normal`); a terrain hit whose normal's y is over 0.7 is walkable ground,
   not a block; an object's or a blocker's always is. The response:
   * at a dash (or a run inside a prism) the first block only drops the pace: the level a notch
     down with its timer at 1.5 s, or, with the speed factor over 1, the speed record's timer
     (brain+0x8a4) at 1.5 s, which takes the follow's boost off (FUN_00048270); nothing steers
     that frame. So a boosted dash meets a wall twice: the boost off, then the run;
   * a side ray's block turns it square to its heading, away from it (a left hit: the heading's
     right); both sides blocked, the side whose block is farther;
   * the centre's alone (a wall ahead): two more rays along the side rays, four times as long, and
     the side with more room (a missed long ray keeps its slot's last distance, as FUN_000ddd10
     writes it only on a hit: copied as read); then, at a run or a dash, held to the side the
     target lies on (FUN_00018b50), with the level 2 for 2 s when that overrides the pick.
   A side ray reaches 0.87 m across at a run, 1.56 m at a boosted dash, 0.52 m at a walk: what
   keeps a member about a metre off e13's hinter band.
2. **The push** (the sensor's +0x3c, 0x521c0, `push`): every other living character (of any
   side: the world's two character lists, no team test) within 1.5 m across, overlapping in
   height (1.0 m margin) and in the half in front of the heading (the commanded move, as the
   feelers'), or within 0.5 m anywhere, pushes the direction away from it by 1 over the distance;
   the sum (not normalised there: FUN_0004d970 does it). The stage has no memory: what keeps a push
   of a frame or two from showing is the move's turn after it (below), which starts from the last
   command and turns at most 200 degrees a second.
3. **The side-step** (the sensor's +0x4c, 0x52750, `side_step`): the nearest obstacle disc in the
   path (its offset across the heading under the two radii, the walker's 0.5 m and its own), within
   its radius + 2 m and in front: a step across, away from it. Its discs come from two world lists,
   [world+0xa4] and [world+0x12c], neither identified (the radius method, 0x52c90, takes the body's
   bounding sphere for OT_STATIC_GEOMETRY, OT_PLATFORM and OT_WORLDBUTTON, and 0.5 m for a
   character, the walker's own disc); the demo has none to give it (`obstacles`).
4. **The plain direction** (the sensor's +0x24, 0x519f0, `plain`): to the target, flat.

**The move** (FUN_00019af0, `move_turn`, #344's fix round). The direction goes to FUN_0004d970
(play_routes.rs `pace`: the move at the lower of the mode asked and the pace level, the direction
sized x0.01 at a stand, x0.5 at a walk, x1 at a run or a dash) and the AI update's steering
FUN_00019ed0, which turns the character's view toward it (below) and, for the follow and Move To
(no FOLLOW_HOLD_BACK route at brain+0x684), hands the move to FUN_00019af0: the move turns
from the heading toward the direction **at most 200 degrees a second** (3.49 rad/s), its rate eased
from the last frame's, a third of the way a game frame at a run or a dash and a fifth at a walk,
dropped to 0 when the turn reverses, never past what closes the angle; within a degree it keeps the
heading, and after a frame that ended with no direction (a stand) it takes the direction at once.
The move is the character's stick (char+0x5e8, `Avoid::moved`), and the heading it turns from
next frame is that move (#363): the chain is command to command, the speed the character makes
never feeds back. The demo moves a character along its facing by its clip's root and turns the
facing at 9 rad/s, so its facing follows that move; the ease is applied by the frame's share of
the game's 30 Hz frames.
So a feeler's square steer, a frame or two at a time, bends the path by a few degrees where before
it turned the member a third of the way round at 15 fps: the weave of the first round's e13 runs
(Hawk and Brutus 1.18 and 1.13 changes of side a 10 m, take01's 0.36 and 0.42) is gone. A turn at
200 degrees a second has a radius of 1.8 m at a run (6.3 m/s) and 2.4 m at a dash (8.5 m/s), so a
member dashing at a doorway at an angle can miss it, or loop round a corner point it has to close
on. The game's facing turns no faster for the squad (FUN_00019ed0: the type's turn rate x
char+0x7ec, which the brain's constructor sets to 200 / (type+0x2d0 x 57.3): 200 degrees a second
too). What slows its members there: on a Move To the pace near the route's points (FUN_0004d840,
`pace_near`: a walk within 3.2 m of a sharp corner); indoors the level's prisms (FUN_0004d970: a
dash is a run inside one; FUN_00018c40: a feeler's hit there drops a run to a walk), which the
demo doesn't read: see "Not done". The follow's walk has neither in the demo, and in the game only
the prisms (its state 0 passes FUN_0004d970 no fifth argument, 0x22377). A turn past 180 degrees
at the cap (a loop to close on a corner point) lasts over a second at more than 2.5 rad/s, so it
ends in the hard turn's walk; a plain U-turn at the cap, 0.9 s, doesn't.

**The view and the motion** (#363; play_avoid.rs `view_turn`, `pick`; play.rs `step_player`,
`ai_gait`; the reading is `todo/363-squad-combat-freeze/leads.md` §1-§3). A character's motion
comes from its stick in its own view frame (R, char+0x5bc), and the AI turns that view itself:
* **The view** (FUN_00019ed0, `view_turn`): toward the steering direction at **200 degrees a
  second** (the type's turn rate x char+0x7ec, which the brain's set-up makes 200 / (the rate x
  57.3) for a squad type), the fraction of that eased as the move's (a third of the way a game
  frame at a run, a fifth at a walk), a turn the frame can finish closed at 2/3 of what's left
  (its kept fraction first halved, 0x1a086), none within a degree. Its one override (brain+0x488)
  is the follow's look-point near a cover point (FUN_00022790, not done); no fight goal sets it,
  so in a fight the view turns to the move, never to the target. FUN_0011e5e0 hands the turn to
  the character and FUN_00119fa0 applies it, and while the character stands its body follows its
  view.
* **The motion** (FUN_001195c0, `pick`): forward when the move is within about 66 degrees of the
  view (a cosine of 0.4; 0.25, about 75 degrees, with the gun up: the ready timer char+0x7dc over
  0), backward past 114 (105); in between, by the body's half the move is in. Forward, the body
  turns to the move and it runs (CT_RUN: the run, the walk or the dash by its pace) while the
  body is within 45 degrees of the view or still 30 or more off the move, and steps sideways
  (CT_MOVE_LEFT / RIGHT, toward the side of the view the body is on) only with the body on its
  move and the view more than 45 off it. Backward, the body turns to the move's reverse and it
  backs off (CT_MOVE_BACK) with the body within 45 of the view, sideways (CT_MOVE_*_BACK) past
  that once the body is on the reverse, and gives no command while it turns. So on a straight
  the three agree and it runs facing its move; a side step plays for the frames of a sharp turn
  its view lags (from a stand, say, the move taken at once and the view turning after it), a
  backpedal on a reversal until the view has swung to within 66 degrees of the move. The gun
  finds a target with the upper body as it runs (play.rs's aim, the arm chain turned up to 1.9
  rad). The side and back motions at a run or a dash are running states (FUN_00126a40: MT 26 /
  30, 27 / 31, MT_BACKPEDAL) and play the sets' `f_side_run`, `b_side_run` and `back_run` clips at
  the run's speed, x0.8 more while backing off (FUN_0012cb40, char+0x800 = 0: 0.64 x the type's
  run without the follow's boost, play_speed.rs); at a walk the side and back walks keep their
  clip's root, as every walk does.
* **What it replaced.** Until #363 a demo member took the controlled character's camera for its
  own and moved by the player's aimed strafe against it whenever it aimed (a side walk for a move
  30-150 degrees off his look, a walk back past that, a walk for a run: the clips' 0.4-2 m/s), its
  heading read from the speed it made: a member with a target and its way more than 30 degrees
  off the leader's look crawled at 0.3-0.7 m/s, its gait flipping about 12 times a second and its
  clip layers stacking 10-11 deep, and the push's frames swayed a running group. Its move was also
  put in the leader's camera's frame before the fire turned the camera onto its target, so it
  moved off its way by the angle between the two. The game's AI reads no other character's view.

**The pace record** ([brain+0x1a8], `Router::cap`, play_routes.rs `pace_ramp`): FUN_00058530
starts it at the level 1 (a stand) with its timer at 0.3 s; FUN_00058670, each AI frame (0x1b119,
before the route steps), runs the timer and, once it has run out with the level not the mode
asked, steps the level one notch toward it, up (FUN_000589b0 to 2, FUN_00058a60 to 3, FUN_00058aa0
to 4) or down (FUN_00058b10 to 3, FUN_00058aa0 to 2, FUN_00058a20 to 1, that one with 0.3 s), and
starts the timer again at the step: 0.17 s for a squad brain in FOLLOW or WINGMAN, 1.0 s for any
other. A follower stopping steps down to a stand and climbs again when it sets off: a walk at
once, a run 0.17 s later, the dash 0.34 s after it; on a Move To, the run after 1 s and the dash
after 2. Then **the hard turn** (FUN_0001ad50, 0x1b16b, `hard_turn`): a route walker at a run or a
dash turning faster than 2.5 rad/s (char+0x488; the demo's: its facing's turn over the last frame)
for over 1 s walks for 1 s. The ramp runs for every AI character the routes' goals move, with a
level's points or not.

**The speed record's timer** (brain+0x8a4, `Avoid::boost_off`): counted down each AI frame
(FUN_00048270, after the goal; the demo's a frame sooner); while it runs the follow's boost isn't
handed to char+0x720 (play_speed.rs `record_boost`). The feelers read char+0x720 as the last AI
frame left it (`Avoid::factor`).

**The demo's choices**, labelled in the code: the push's height test (the two bodies' spans, 1.9
m each, grown by the margin: FUN_00052120's extents aren't read); the push's list, the characters
within 2 m at the frame's start (`routes::near`); no side-step obstacles; the hard turn's rate from
the facing's turn; the move's and the view's ease by the frame's share of the game's frames. Since
#363: a walk from a stand starts its view from the way it faces (the game's standing body follows
its view; the demo's stands face their own way); the body turns at the demo's 9 rad/s, not the
game's 400 degrees a second x k (FUN_001195c0's request, the step at 0x11d7fb), as the demo moves a
character along its body: so the body keeps up with the move and the path is the command's; a
character the player has had starts its steering afresh when the AI takes it back (the game's
would find the player's last stick in char+0x5e8); the walks' speeds are the clips' (#350).

**Test hooks**: `BF_AVOID_LOG=1` prints each frame a stage before the plain direction steers:
which feelers met what (steep terrain, an object, a blocker) and how far, the response (the
direction, the long rays' slots, the hysteresis' walk), the pace drops and the boost's, the push
and from whom. `BF_AVOID_OFF=1` (a diagnostic, not the game) steers by the plain direction alone,
as before #344. `BF_AVOID_FREE_TURN=1` (a diagnostic, not the game) leaves the move's turn to the
facing's 9 rad/s, as before #344's fix round. `BF_ROUTE_LOG=1` also says each hard turn's walk,
`=2` each step of the pace level. `BF_GAIT_LOG=1` (#363) prints every frame each AI member's
motion after its step: its gait and action, whether it aims and fires, its view, body and
commanded move (degrees, the demo's yaw; `-` for none), `back` in the backward case, the speed its
step made, where it is and its clip layers (clip:weight): to count gait changes and layers.

## The door step and the corner hold (#346)

The reading is `todo/346-route-doors/leads.md`; no footage shows a squad member at a door
(its shot list's A and B), so the demo follows the code alone.

**The door step** (FUN_00026820, play_routes.rs `door_step`): FUN_00026d80 hands it a door point
reached (kind 6; the corner rule skips door points). It acts only when the **next** entry's point
is a door point too, the pair either side of a door: the 0.8 m corridor between the two points at
their positions (FUN_000410a0, as the route's other tests: the arena's ray, which a door's leaves
block until 0.6 of their clip) blocked, the pace level goes to a stand with its timer at 0.5 s and
the feelers go off for 1.5 s (brain+0x924). The walk moves on regardless (the step's return is
ignored): no wait for the door, no read of its state, no new plan. So a member reaching a shut
door stands half a second, then the ramp walks it on toward the far point with nothing turning it
off its line; a door its detector opened in time is walked straight through. A door that stays
shut is left to the stuck handling. The step's first branch, a world button at the point (+0x3c,
an OT_WORLDBUTTON "used" with TRIG_SIG_USE_OBJECT), is left out: no door point in the data has an
object (every h_e5851820 is empty, no h_0b6b92ba record attaches one), so it never runs. Doors
open by their own triggers, not by the walker; locked doors never get this far, their points made
invalid by the level's waypoint triggers (e34's six, sdm_e34's two, the tutorial's base gate
until its panels).

**The corner hold** (brain+0x9dc, `Walker::corner`) has no timer in the game: it's set at a point
reached whose next leg is blocked and cleared only when a point is passed with the rule not
holding (the corner point itself once within 0.6 m, a door's or a jump's point, or the rays clear
again) or by the brain's reset (FUN_00026d80's row). A member that can't close on the corner point
is moved by the stuck handling alone. The demo's never had a timer either; what pinned its
members at e13's corners was its missing feelers, and with #344's none needs the stuck handling:
in the e13 runs every hold (14 of them, 0.3-1.4 s) ended by passing within 0.6 m.

**Test hooks**: `BF_ROUTE_LOG=1` says each door step (the two points, the corridor clear or
blocked); `BF_TEST_DOOR_SHUT=<h_xxxxxxxx>[,...]` holds the named doors shut (maps.md, the doors'
hooks: `BF_DOOR_LOG=1` lists their names).

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
  point with a 1 m radius of its own instead of closing on it (the hook's choice; the pace near a
  route's points, FUN_0004d840, is a brain's, `pace_near`). The camera faces the way it goes, or keeps
  `BF_CAMERA_YAW`; with `BF_CAMERA_PITCH` and `BF_CAMERA_DISTANCE` it looks from above. The
  level's prisms aren't read, so no radius is held to 1 m indoors.
* `BF_ROUTE_LOG=1` also logs the AI squad's routes (#335): each plan and why (the follow's chase
  to its leader's point, its leader over d3 from the route's end, the walk to a cover point, Move
  To, the detour, the re-route), from where, the own point, the destination (a point, or a
  character at his point), the entries, the dropped first entry and the search's cost; each point
  reached, corner held, point skipped, catch-up, stuck, detour, put-at-the-point and blocker step;
  `=2` also each one's own point's changes, the pace level's climbs, the pace near a route's points
  setting the level down (near its end or at a corner, FUN_0004d840: Move To's) and, once a
  second, its walk's state (the entry, the target, the radius, the distance, the corner, the stuck
  time and best, the pace level and its timer). The follow's states, holds and cover points are
  `BF_FOLLOW_LOG=1`'s (#331, [squad.md](squad.md)).
* `BF_TEST_WARP=<s>,<who>,<x>,<z>[,<heading>][;...]` (bf_play): at time s `who` (`me`, the
  controlled character, or a member by name or slot, as `BF_TEST_ORDER`) is put on the floor at
  (x, z), facing the heading (degrees, 0 = -z) or as it faced: the takes' trainer `tp`.
* `BF_TEST_PRISM=1` (bf_play): every character counts as inside one of the level's prisms
  (brain+0x3c4 bit 10, whose records the demo doesn't read): the walk's radius at most 1 m, the
  dash a run, and FUN_00041870's route round a friend, which only runs then.
* `BF_ROUTE_NO_HINTERS=1` (bf_play, a diagnostic, not the game): the corridor rays leave the
  hinters out, which the game's never do: they meet the hinters for everyone (FUN_0004ce30 to
  FUN_000df0a0, which tests every PID kind whose bit is clear in the filter's +0x10c, 8: the
  blockers, hinters and fly-blockers; todo/344's leads §7). Only to see the routes without the
  hinter bands the demo's members hug (e13 below); what it shows isn't the game's.
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

* **The squad on routes (#335)**, `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps. The times
  below are game seconds (the demo's clock); the take's were measured off its video at first,
  wall time, which ran at 0.71 of the game's clock in xemu. The e13 staging of
  `todo/242-ai-routes` take01 (Tex on the north rim facing south, the three on the south rim):
  `BF_MAP=e13 BF_TEST_GOTO=37.3,25.6,37.3,25.6 "BF_TEST_WARP=0,me,37.3,25.6,180;
  0,brutus,44.6,73.8;0,flint,43.1,75.8;0,hawk,45.7,75.6" BF_TEST_NO_HOSTILES=1 BF_ROUTE_LOG=1
  BF_ORDER_LOG=2 BF_LIQUID_LOG=1 BF_CAMERA_PITCH=-1.2 BF_CAMERA_DISTANCE=70`, 900 frames:
  * each member's first route goes round by the west shore through the take's points in the
    take's order, h_0f143f8f (37.3, 73.6), h_141d6e35, h_1fe8f610, h_fb8c63b3, h_f079fb96 (9.3,
    73.6), h_e9ba9a68, h_f2b3cbd2, h_029e248d (2.3, 57.6), h_efb7f462, h_f3024e36, h_1df21e91,
    h_1c93b982, h_ef15c18a, h_f6599a9a, h_1469ef23, h_1b3a3faf, h_e41a0444, as the game's three
    (notes.md: Brutus's and Hawk's 20 points, Flint's 19); then Brutus's to his place beside
    h_e7a4bd37, and Flint's and Hawk's on by h_ed1d34d2 to Tex's own point h_f6146568, where the
    game's three ended: their places, 8 and 11.5 m out from him, lie on the slope down to the
    lava, 3.3 m under his feet (more than 2 m: since the fix round a place no one could stand on
    is given up for the leader's point). bf_level's route from Brutus's spot to Tex's point is the take's 20
    to the point, 116.8 m for 48.7 straight. Two differences: the game's three keep their own
    point as the first entry, which the demo drops (both corridor rays clear from Brutus's spot
    in the demo: `BF_ROUTE_PROBE`; what the game's met isn't found); and the corners;
  * the corners: the game held its members (take01's brain+0x9dc) at h_f079fb96, h_e9ba9a68,
    h_029e248d and h_1469ef23, as the demo does, but not at h_f2b3cbd2 or h_f3024e36, where the
    demo's rays meet e13's hinter band h_09910e17. The band runs along the west walkway with
    both points just beyond it (h_f2b3cbd2 0.21 m, h_f3024e36 about 1.5 m: `BF_ROUTE_FACES`);
    the game's members kept about a metre off it (their avoidance, FUN_000255d0's feelers, not
    done) and the game's Flint skipped h_f3024e36 from h_efb7f462 (his steering target h_1df2
    from 17.08 s), where the demo's, steering straight at the points, slide along the band, are
    held at those corners and stop 0.6 m short of them against it. The stuck handling then moves
    them on (with the fix round's 5 cm: round 1 counted only an equal distance as none closer,
    and the review saw a centimetre's jitter start the time again for good): Brutus's detour at
    9.7 s at h_f2b3cbd2 takes the -0.7 m lane (h_1021c864, h_ee8f64b4, h_f79d787b, h_0ec4ef6b,
    h_eaf9b73f, h_fec3851a, h_ef0679a7), the lane the game's Brutus took after his own detour,
    and he is at his place at 21 s; Flint's at 11.8 s takes it too, and he is at Tex's point at
    24.1 s; Hawk is detoured twice at each of h_f2b3cbd2, h_029e248d, h_f3024e36 and h_1df21e91
    and put on the point the third time (FUN_000425b0, the route's current point), about 10 s a
    corner, and is at Tex's point at 58.1 s. The game's three were there at 16.5 s (Flint), 18.5
    s (Hawk) and 23.0 s (Brutus) by the game's clock (the 25-32 s once given here was the take's
    wall time). With `BF_ROUTE_NO_HINTERS=1` (round 1's run; not the game: its rays meet the
    hinters) the holds match the game's but for the brief ones at h_f079fb96, h_e9ba9a68 and
    h_029e248d, h_f3024e36 is skipped from h_efb7f462 and h_f2b3cbd2 reached at 2 m as the
    game's were, and all three arrive in 14-18 s: the pinning is the missing feelers' (#344);
  * none enters lava: `BF_LIQUID_LOG` never fires (the routes' safety net never acts);
  * **with #345's ground speeds** (the squad's run and dash by the types' speeds: the dash 8.5-8.7
    m/s while the leader is farther than 14 m and 6.8-7.0 inside it, the run 7.8-7.9 and 6.2-6.3;
    play.md "Ground speed"; `BF_ROOT_SPEED=1` runs the staging as the numbers above were): the
    same first routes round the west shore through the same points, and never into lava
    (`BF_LIQUID_LOG` silent). The arrival times move: Flint's route ends at Tex's point at 18.6 s
    (24.1 s; the game's 16.5 s), Hawk's at 57.5 s (58.1 s; the game's 18.5 s), and Brutus reaches
    his place beside h_e7a4bd37 at 42.5 s (21 s; the game's 23.0 s). The members' dash on the
    first straight is the take's (Brutus 8.72, Flint 8.62, Hawk 8.52 m/s over 5 s on the test
    floor from 80-90 m off the leader; take01's 8.59-8.62, 8.53-8.55, 8.42-8.46). The pinning at
    the band's corners isn't removed, it moves from member to member with the speeds (#344's
    feelers): Brutus is now stuck at h_f2b3cbd2 (10.1 and 15.1 s, through on the second detour), at
    h_f3024e36 (22.1, 25.1 and 28.2 s, put on the point the third time) and at h_1df21e91 (31.5,
    34.6 and 37.7 s, put on it), where he was detoured once; Flint is stuck once, at h_f2b3cbd2
    (8.5 s); Hawk as before at the same four corners (put on the point at 22.1, 32.1, 42.7 and
    52.2 s).
  * **with #344's avoidance** (the feelers, the push, the pace ramp from a stand and down, the hard
    turn, the game's stuck test; the same command with `BF_AVOID_LOG=1`; `BF_ORDER_LOG=2`'s 15 Hz
    rows against take01's 10 Hz watch rows, both from the release, "within 10.5 m of Tex" as the
    arrival, the take's state 3 being 9.8-10.0 m off him): Flint 17.6 s (the take's 16.6), Hawk
    18.1 s (18.6) and Brutus 18.9 s (23.1; the game's Brutus took a detour lane after a stuck, 145 m
    to his 128, the demo's none). Along the west walkway, the first time each is past z 64, 50 and
    38 and then on the north rim (x over 20): Hawk 8.9, 11.5, 14.1, 17.2 s (the take's 9.1, 11.7,
    13.8, 17.5), Flint 8.4, 10.8, 13.6, 16.7 (7.8, 10.1, 12.3, 15.8: the game's Flint ran the first
    straight on the lane of points at z 77.6, his own point carried from before the freeze, the
    demo's spawn picks h_06130f19 on the z 73.6 one, whose hinter band 1-4 m south of it his feelers
    meet twice at the dash), Brutus 9.3, 13.2, 14.9, 18.0. No stuck at all (21 before, 6 of them
    put on the point), no detour, no lava (`BF_LIQUID_LOG` silent), 116 frames of feelers steering
    and 70 of the push, two hard turns' walks. The corner holds: h_f079fb96, h_e9ba9a68,
    h_029e248d and h_1469ef23 for all three, the take's four, plus one member each at h_f2b3cbd2
    (Hawk) and h_f3024e36 (Flint), which the game's didn't hold; each held 0.3-1.4 s and passed
    within its 0.6 m. A track plot against `BF_ROUTE_FACES`' hinter faces shows the three down the
    west walkway on the band's outer side as the take's, a metre or so off it, wavering at
    h_e9ba9a68-h_f2b3cbd2 where the take's curve is smooth (the push and the square steers turn
    the demo's members, which move along their facing). With `BF_AVOID_OFF=1` (the plain direction
    alone, the rest as #344's): Hawk pinned at the band for good, 8 stucks, Flint at 40.5 s.
  * **with #344's fix round** (the move's turn, FUN_00019af0; the stuck test's `progress`; the
    follow's hold within d0): Flint 18.8 s (the take's 16.6: +13%), Hawk 17.1 s (18.6: -8%) and
    Brutus 21.9 s (23.1: -5%), no stuck, no detour, no lava. The weave (the tester's measure, 0-25
    s from the release, the track resampled every metre: changes of side over 0.3 rad a 10 m, the
    turn a metre): Flint 0.42 and 0.158 rad, Hawk 0.25 and 0.123, Brutus 0.74 and 0.182; take01's
    0.28 and 0.138, 0.36 and 0.133, 0.42 and 0.155; the first round's 0.27 and 0.117, 1.18 and
    0.183, 1.13 and 0.192. The spread off the track's 9 m mean grows (0.77, 0.68 and 0.88 m; the
    take's 0.59, 0.59 and 0.67; the first round's 0.66, 0.60, 0.67): the turn's wider arcs. Over
    six staging shifts (all three 2 m west, swapped round, 2 m toward the pit, bunched, 14 and 24 m
    west along the rim) all arrive in 13.7-20.5 s, with one stuck in three of them and three in
    one, each a detour at a corner of the band (h_f079fb96, h_e9ba9a68, h_f2b3cbd2), none put on a
    point; with `BF_AVOID_FREE_TURN=1` none (the first round's numbers to the frame).
* Move To across the pit (e13, the squad and Tex on the south rim: `BF_MAP=e13
  BF_TEST_GOTO=44.3,73.6,44.3,73.6 "BF_TEST_WARP=0,me,44.3,73.6,0;0,brutus,45.7,75.8;
  0,flint,42.3,75.8;0,hawk,43.0,72.0" BF_TEST_NO_HOSTILES=1 "BF_TEST_ORDER=hawk,move,34.3,29.6@2"`,
  1800 frames): a 20-entry route round the west shore to h_e7a4bd37 on the north rim. The same
  four corners pin her against the hinter band; at each the stuck handling detours her twice and
  the third stuck puts her on the point (20.4, 31.1, 42.5 and 52.9 s), each detour's drop to a
  stand climbing back a notch a second (MOVE_OUT's step); she reaches the point at 58.3 s, 1.9 m
  off: GUARD and e635ecc6. Never into lava. With #345's speeds (her dash 6.8 m/s, 8.5 x 0.8,
  as take08's plateau 6.84; the clips' 7.2 before) the same four corners pin her (put on the point at
  20.5, 30.6, 42.1 and 52.2 s) and she reaches it at 57.9 s. (The review's run, round 1: held 3.69 m short of
  h_f2b3cbd2 for good, the jitter starting the stuck time again.) With #344's avoidance: no stuck,
  held at five corners for a moment each, two hard turns' walks; she reaches the point at 20.2 s,
  18.2 s after the order, 1.8 m off: GUARD. With the fix round: held at seven corners for a moment
  each, no stuck; she reaches the point at 22.7 s, 20.7 s after the order, 1.8 m off. Brutus and
  Flint, sent to places 7 and 9 m out in front of the standing Tex that lie across each other's
  way, run south side by side 1.2 m apart: each turning toward its place brings the other into the
  half in front, whose push turns it back, and at 200 degrees a second neither breaks away (with
  the free turn both do). They run on past their places (Brutus 9 m) down the slope, where the
  feelers' blockers stop them, stuck at 3.9 and 4.1 s, 16 and 13 m off Tex; held there, as the
  fix round's first build did, they stood so for good. Beyond d0 the follow plans again: Flint
  reaches his place, 8.6 m off Tex, and Brutus, stuck twice more short of h_f7672590, is put on it
  (9.1 s) and holds there, 10.1 m off Tex, 3.1 m short of his place. With the second fix round's
  pace near the route's points (`pace_near`, `BF_ROUTE_LOG=2`): the level dropped to a walk by
  the corners at h_f079fb96, h_f2b3cbd2 and h_1b3a3faf and a notch at h_1c93b982 (FUN_000587d0,
  each with the timer at 1.5 s, then the ramp's notch a second), so she walks most of the west
  walkway, whose corners lie 5 m apart; a run from 5.9 m off the point, a walk from 2.9 m; she
  reaches it at 27.9 s, 25.9 s after the order, 1.9 m off, no stuck (no footage of this one).
  On e34 (#318's take08 recipe
  with Tex on her way, `BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999
  "BF_TEST_ORDER=hawk,move,-89.5,-87.8@1;hawk,move,-70.8,-79.5@10"`): a 5-entry route past Tex,
  at h_ff113737 3.13 s after the order, GUARD 1.8 m short (#318: 3.1 s and 1.8 m; take08 4.2 s,
  1.9-2.0 m), the same in round 1 and the fix round; with #345's speeds 3.26 s and 1.9 m short
  (her dash 6.8 m/s), the same route, no stuck; with #344's 3.86 s and 1.9 m short (Move To's ramp
  from a stand: a walk, the run after 1 s, the dash after 2), the same with the fix round; with
  the second fix round's pace near the route's points 4.13 s and 2.0 m short, the take's 4.2 s
  and 1.9-2.0 m: she dashes from 12.2 m out, runs from 5.1 m out and walks from 2.7 m out (the
  watch rows at 7.5 Hz; take08's 5.6 and 2.8 m at 10 Hz, FUN_00058590's 6.0 and 3.2 m), where she
  had dashed to 1.7 m.
* The follow on e34 (`BF_TEST_GOTO=-83.39,-82.79,-30,-80,27 BF_TEST_GO_AT=4 BF_TEST_NO_HOSTILES=1
  BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, 300 frames, Tex running east past the rocks):
  8-12 plans a member from 4.5 to 16 s (4-5 at most in any 2 s), 1-2 holds each at points short
  of a place behind a rock (Object 464 by h_f6632343), 0.6-0.9 s each; of the 87 watch rows in
  that time, standing 8 (Brutus), 10 (Flint) and 5 (Hawk) (round 1: 3, 5 and 4; with d3 as the
  hold's distance 21, 15 and 12, Brutus ending 10 m behind Tex); all three at their places when
  Tex stops (6.4-10.7 m off him), no stuck. With #345's speeds (the follow's dash 8.5-8.7 beyond
  14 m of Tex, 6.8-7.0 inside; Tex's run 7.8 m/s where it was the clip's 4.8): 7-8 plans a member
  in that time, 0-1 holds each, standing 12 (Brutus), 6 (Flint) and 4 (Hawk) of the 87 rows, all
  three at their places when Tex stops, 6.5, 7.9 and 10.9 m off him, no stuck, no detour. With
  #344's avoidance: no stuck, no detour, the same final distances (6.5, 7.8, 10.8 m), the members
  never closer than 1.3 m to each other (0.72 before: the push), the lag behind Tex while he runs
  a mean of 5.8 (Brutus), 9.5 (Flint) and 10.1 m (Hawk) against 5.0, 8.4 and 6.7 (Hawk's back-step
  to the point behind her at 5.4 s, #335's follow, comes a little sooner, and her hold there
  ends later). With the fix round: no stuck, no detour, the same final distances (6.4, 7.8 and
  10.8 m), the members never closer than 0.81 m, the lag a mean of 5.9, 12.9 and 11.3 m: Flint's
  route at 6.9 s goes back to his own point behind him, and turning round at 200 degrees a second
  takes over a second at more than 2.5 rad/s, which ends in the hard turn's walk (8.3 s), 21 m
  behind Tex at most; Hawk's back-step the same. (The tester's sprint recipe, `BF_MAP=e34
  BF_SCRIPT=off BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4
  BF_TEST_SPRINT=4`, 375 frames: a mean lag of 3.7, 5.0 and 7.0 m, the first round's 3.6, 5.1 and
  7.8, main's 3.4, 6.9 and 8.8; never closer than 0.78 m.) The second fix round changes neither
  (the follow's walk has no pace near its points). Both back-steps are #335's follow: main makes
  them too (Hawk's route at 5.5 s to h_e90ebcab, Flint's at 6.2 s to h_e0098c3d, toward the
  place's point behind them), turning round at once; the game's follow routes to the leader's own
  point (state 0) and wouldn't (#331). Over the rows before each one's back-step the lag is 4.8
  (Flint) and 8.3 m (Hawk), main's 5.4 and 8.3; Brutus, who has none, 5.9 (main 5.0: a 2 s walk
  at 8.7 s, the long rays' pick held to the target's side, FUN_00018b50). A plain U-turn at 200
  degrees a second (0.9 s) doesn't set off the hard turn's walk; Flint's and Hawk's went on round
  their corner points (h_e0098c3d, closing to 0.6 m) past 1 s. (Tried and dropped, not the game's:
  the pace near the route's points for the follow too. Hawk then goes through sdm_e34's door at a
  dash, but on e13 she arrives at 25.9 s, 39% over the take, m07's Flint walks 23 m at rest, the
  sprint recipe's lag grows to 6.4, 9.7 and 7.6 m, and the e34 run's stays at 5.9, 11.3 and 10.5.)
  Stand Ground holds where ordered while Tex goes 32 m
  off, Fire At Will and Cover Me follow (`"BF_TEST_ORDER=hawk,guard@3;flint,fire@3"`, round 1).
  sdm_e13 (#1's recipe, the controlled Brutus walking into the lava): the three stay on the
  shore's points, 8-9 m from him (round 1). The tutorial's marines (brain.md's recipe) follow Tex
  on routes (round 1).
* The leader in the lava (#335's review: e13, `BF_TEST_GOTO=143.9,68.7,120,100 BF_TEST_HEALTH=5000
  BF_TEST_NO_HOSTILES=1`, 450 frames: Tex runs into the lava at 6.4 s and goes round and round his
  goal there, within 2 m of (125.3, 95.1)): from 10 to 30 s no plans at all and none of the three
  moves more than 0.2 m. They wait on the shore at his own point, 11-14 m from him, as their
  places are in the lava. The review's run (round 1): 34-49 plans each in those 20 s, 11-13 in
  2 s, each pacing 2-5 m; keyed on the leader with the 3 m hold but the place's nearest point,
  1 in 2 s at most, but Hawk ran to and fro between h_ebea0733 and h_00bdf10f, 21 m apart. None
  enters the lava (`BF_LIQUID_LOG`). The same with #345's speeds: no plan from 10 to 30 s, none of
  the three moves more than 0.2 m. The same with #344's fix round: no plan and no stuck from 10
  to 30 s, none moves more than 0.2 m.
* #344's review recipes, on the fix round's build (the tester's, `BF_MUTE=1 BF_BACKGROUND=1`):
  * the tutorial's lava yard (`BF_MAP=tutorial BF_TEST_GOTO=-15.6,-87.4,-15.6,-87.4
    BF_TEST_NO_HOSTILES=1`, the base gate's ten waypoint triggers fired, `BF_TRIGGER_FIRE`, and its
    panels pressed, `BF_TEST_PANEL`, Tex sent down the yard by `BF_TEST_ROUTE=73.1,-44.6,16`, 3000
    frames, and from 10, 24 and 40 s, 2400 frames): the two marines' routes from the upper ledge
    go by the jump h_0656d0f4 -> h_03090779; each is stuck there three times, the detours finding
    no other way down, and put on h_03090779 below (Cogburn 75.0 s, Stout 77.5 s in the first);
    within 8 m of the yard's end (73.1, -44.6) at 87 and 90 s, 82 and 84 (10 s), 94 and 97 (24 s),
    and at 112 s for Stout (40 s; Cogburn stops at his place 8.4 m off it at 98 s). main: 84-86,
    78-81, 91-92, 100-107 s; #344's first build: held at the edge for good;
  * m07_a (`BF_MAP=m07_a BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-341.5,39.1,-341.5,39.1
    BF_TEST_ROUTE=-270.4,38.0,2`, 1000 frames, Tex standing from about 20 s, his test walk held at
    a corner by an object):
    from 30 to 66 s Brutus, Flint and Hawk walk 0.0, 0.8 and 0.0 m (main 1.1, 8.9, 0.3; #344's
    first build 124.7, 47.3, 1.7), each held where the stuck handling acted near Tex;
  * sdm_e34's door pair at a run and at a dash (`BF_MAP=sdm_e34 BF_CHARACTER=0 BF_TEST_NO_HOSTILES=1
    BF_TEST_GOTO=-44.6,18.6,-44.6,18.6 BF_TEST_ROUTE=-58.3,3.4,2 "BF_TEST_WARP=0,hawk,-47,21,90;
    0,flint,-46,23,90;0,tex,-47,16,90"`, and `BF_TEST_SPRINT=2`, 900 frames; Brutus led through
    h_f519ca12 / h_117d5fb1 into the building): from 15 to 60 s Flint, Hawk and Tex walk 0.6, 1.5
    and 1.8 m at a run (main 0.5, 0.1, 0.3; the first build 1.0, 0.2, 2.1), and 0.7, 42.5 and 1.6
    m at the dash (main 1.5, 0.2, 1.1; the first build 8.6, 3.2, 0.8): Hawk, on through at the
    first door point at 6.3 s, swings wide of the doorway at 200 degrees a second, is stuck outside
    it at 8.9 s, and the detour takes her round the building by its other doors, at her point at
    22.9 s (with `BF_AVOID_FREE_TURN=1` through in 2 s, 3.2 m walked after 15 s);
  * e34's Fire At Will then Cover Me (`BF_MAP=e34 BF_TEST_GOTO=-60,-79,-60,-79,30
    "BF_TEST_ORDER=all,fire@1;all,cover@6"`, 450 frames): two stucks (Brutus and Hawk, 8.4 s),
    each held at once where it stood near Tex (main none; the first build six and a place given
    up).
* **The game's follow (#331)**, the recipes above on its build against main's (`b3f6a9c`), `BF_MUTE=1
  BF_BACKGROUND=1`, the lag the mean over the 15 Hz rows with the leader moving (the takes' own
  numbers are in [squad.md](squad.md)):
  * the e34 run (`BF_TEST_GOTO=-83.39,-82.79,-30,-80,27 BF_TEST_GO_AT=4`, 300 frames): a lag of
    14.7 (Brutus), 13.5 (Flint) and 13.3 m (Hawk), at most 16.3, 16.1 and 16.0, never ahead of Tex,
    and at rest 9.3, 9.0 and 9.2 m behind him; main 5.9, 12.9 and 11.3, at most 13.7, 21.6 and 16.5
    (Flint's and Hawk's back-steps to their places' points behind them), 17-21 rows of 87 ahead of
    him, at rest 6.4, 7.8 and 10.8. 20 plans in all (main 28). The lag is the game's: the chase
    holds a member about 14 m behind a leader at its own top speed (take04's 13.7-14.4 m); main's
    was less because its places ran beside and ahead of him;
  * the sprint recipe (`BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4 BF_TEST_SPRINT=4`):
    15.3, 16.7 and 13.7 m, at rest 8.2, 9.2 and 9.1 (main 3.7, 5.0 and 7.0, 15-21 rows ahead);
  * #242 take01's staging on e13: arrivals within 10.5 m of Tex at 18.8 (Flint), 17.1 (Hawk) and
    21.9 s (Brutus), main's to the frame (the game's 16.6, 18.6 and 23.1: +13%, -8%, -5%), the paths
    116, 113 and 126 m (main 124, 121, 137; the game's 109, 112, 145); each stops 9.4-9.7 m off him
    (the take's state 3 at 9.8-10.0 m); no stuck, no lava, no warp;
  * e13rev (Tex running the route to the south rim, `BF_TEST_ROUTE=44.3,73.6,2` from the north rim,
    700 frames): a lag of 16.2, 15.2 and 14.6 m, at most 33.9, 23.8 and 18.3 (main 23.9, 21.7 and
    20.6, at most 46.6-49.9; #344's tester's main before #344: 18.5, 11.5 and 14.1); one corner
    hold each at h_1df21e91 (#344's Brutus held there four times, 5 s), Brutus held twice at
    h_029e248d; at rest 9.3, 9.4 and 8.5 m; no stuck, no lava;
  * take08's Move To: at the point 4.13 s after the order, 1.9 m short (main 4.13 s, 2.0 m);
  * m07_a (Tex standing from about 20 s): from 30 to 66 s the three walk 0.6, 0.2 and 1.0 m (main
    0.0, 0.8, 0.0), at rest 9.0, 9.4 and 9.2 m off Tex where main's Flint and Hawk stood 1.4 and 2.5 m
    off him (its places' fallback to his point); no stuck;
  * the leader in the lava (e13, `BF_TEST_GOTO=143.9,68.7,120,100 BF_TEST_HEALTH=5000`): from 10 to
    30 s the three walk 0.2, 4.0 and 0.2 m on the shore, 11-15 m off him, none into the lava (main
    0.9, 0.6, 0.1). Without `play_follow.rs` `in_burning` (the demo's: no route to a leader's position
    in a liquid that burns) they walked 105-134 m in those 20 s, 500 plans: the chase plans every
    update with him farther than d3 from his own point, and the route of a member on his own point
    is his position, in the lava, which the shore guard held them out of as they slid along it;
    sdm_e13's Brutus in the lava: the three on the shore 9.3-10.6 m off him (main 9.4-11.6);
  * the tutorial's lava yard (t335test's recipe): the marines stuck at the jump's edge and put on
    h_03090779 below as main's (Cogburn 74.3 s, Stout 77.0 s), within d2 of Tex at the yard's end at
    85.7 and 88.9 s (main's within 8 m of the end at 87.1 and 90.1 s), Cogburn to a cover point;
  * sdm_e34's door pair at a run: from 15 to 20 s Brutus, Flint and Hawk walk 0.0 m (main 0.2, 0.0,
    0.6); e34's Fire At Will then Cover Me with its enemies: 3 stucks (main 6), the squad fights
    (16 rows firing, main 76; main's Hawk died at 12.5 s, here no one: the members hold 9-15 m off
    Tex, not 4-9);
  * the test map's floor (no world points, `--test BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,0,0,14
    BF_TEST_SPRINT=2`): the straight chase, 12.3-13.4 m behind Tex's dash, at rest 9.2-9.3 m (main
    1.3-3.4 m running beside him, at rest 6.3-10.7). A goal Tex can't reach (`0,0,0,60`: he's
    held at z 17.3) keeps his stick held, so he's running to the follow and the three close
    to 1-2 m of him (FUN_00020ad0 reads the motion, not the speed: take04's Tex too, pushing at the
    rocks).
* **#363: the command heading, the view and the motion** (`BF_MUTE=1 BF_BACKGROUND=1`, the 15 Hz
  watch rows and `BF_GAIT_LOG=1`; main is `a897a30`):
  * **The fight** (`BF_MAP=e34 BF_SCRIPT=off BF_TEST_GOTO=-83.39,-82.79,-43,-80,27 BF_TEST_GO_AT=4`,
    620 frames; Hawk is killed at 3.3 s by the start valley's enemies in both builds): Flint and
    Brutus keep up with Tex's run through the fight, dashing at 6.9-8.6 m/s and running at 6.3 m/s
    with the gun on their target, and are within 10 m of him from 11.9 s (he stops at 11.1 s),
    holding 9.1-9.3 m off; main's Flint crawled at 0.4 m/s and was within 10 m at 26.7 s, its
    Brutus at 20.0 s; before #344, 18.3 and 17.6 s. Their gait changes once or twice a second
    moving with a target (main: 12-13), their clip layers 1.1-1.6 deep (max 3-4, a crossfade;
    main 10-11), their view, body and move within a degree or two of each other on a straight. From
    13 s they lose sight of their target where they hold and turn to its last place, gun up (the
    contacts' turn, play_brain.rs). With Fire At Will (`"BF_TEST_ORDER=all,fire@0.5"`) Flint and
    Brutus stay within 8 m of Tex from his stop (5.9 and 6.7 m off at 11.9 s), Hawk, who follows
    Flint, 12.3 m off; main's two within 8 m at 23.3 and 21.7 s, its Hawk never. On the test map's
    fight (`--test BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,0,0,70 BF_TEST_GO_AT=3 "BF_TEST_NPC=0.5,shrike
    soldier,enemy,fight,-22,25" BF_TEST_NPC_HP=1.5,1,9999`) the three chase Tex dashing, guns turned
    onto the Shrike by the upper body (frames 102-114 looking back, `BF_VIEW_YAW=3.1416`), 0.2-0.9
    gait changes a second, layers 1.1 deep; Hawk backs off for a moment on a reversal (12% of her
    moving frames). They reach Tex where he's held at z 17 by a pillar, in the Shrike's fire, and
    Brutus and Hawk die there; main's crawled 20-42 m behind and lived.
  * **The sway**: the paths' wobble (the tester's measure, the 10 Hz track's heading about its 1 s
    mean) in the fight 12.8 (Flint) and 5.7 degrees (Brutus) RMS at 1.4 and 1.0 crossings a second,
    on the no-hostiles runs (the run to -43, -80; the sprint; take04's) 5-19 degrees at 0.9-1.7;
    #33 take04's 13-18 degrees at 0.7-1.25; main's 5-17 at 0.7-2.2. Where it remains, the members
    run bunched within 1.5 m (the run to -43, -80: Flint with Hawk 1.4 m ahead of him for half a
    second at a time), and the push (the game's stage) swerves the move until the neighbour is
    out of the half in front: the squad's start bunched 1.3-3 m apart is #363's out of scope.
  * **Unchanged**: take08's Move To (GUARD 4.06 s after the order, 2.0 m short; main 4.13 s, 1.9 m;
    the take 4.2 s, 1.9-2.0 m); the e34 run (a lag of 12.1, 13.6 and 13.5 m, at rest 9.3-9.9; main
    14.7, 13.5, 13.3 and 9.0-9.3), the sprint recipe (11.8, 13.2, 13.9 m; main 15.3, 16.7, 13.7), the
    walk (9.8-10.5 m as main's) and take04's (13.0, 18.8, 18.6 m, at rest 9.0-9.3; main 13.0, 19.7,
    18.6 and 8.7-10.6); the yard (the marines put on h_03090779 at 75.7 and 78.7 s, within d2 of Tex
    at the yard's end at 87.1 and 90.1 s; main 77.0 / 74.3 and 88.9 / 85.7); the tutorial's follow
    (to the row); m07_a (from 30 to 66 s the three walk 0.0, 0.6 and 0.6 m; main 0.6, 0.2, 1.0); the
    leader in the lava (at most 0.1 m walked, no lava); sdm_e34's door pair (at most 1.2 m walked
    from 15 to 60 s, run and dash; one stuck at the run, main two); the door step (to the frame as
    main's); with the door shut the first stuck ends the Move To at 9.8 s, 5.5 m short (main's went
    round to a stuck at 14.0 s, 8.5 m off); e34's Fire At Will then Cover Me with its enemies: no
    stuck (main 2), Hawk killed at 5.7 s after a knock-down (main's lived).
  * **e13**: take01's staging, arrivals within 10.5 m of Tex at 18.5 (Flint), 18.8 (Brutus) and
    17.5 s (Hawk) (main 18.8, 21.9, 17.1; the take 16.6, 23.1, 18.6), two stucks (Flint 8.3 s and
    Brutus 9.2 s short of h_f2b3cbd2, a detour each onto the -0.7 m lane), no lava; the weave 0.08,
    0.16 and 0.44 changes of side a 10 m (Flint, Brutus, Hawk; main 0.45, 0.82, 0.27; the take 0.28,
    0.42, 0.36). Over three staging shifts as well (all 2 m west, swapped round, bunched) the mean
    arrival is 20.3, 17.6 and 17.3 s (main 16.9, 18.8, 18.4): Flint late in two (22.1, 24.7 s), each
    time held at the west walkway's band. There a member's move can be caught facing straight away
    from its next point: the left feeler's tip (1.0-1.2 m at a walk) meets the band every other
    frame, so the steering alternates between the feelers' square steer and the way to the point
    behind it, each frame's turn reverses the last (the rate dropped at each change of side,
    FUN_00019a20), and the move holds where it is till the stuck handling moves it on; the view,
    which has both on one side, turns away meanwhile, so a side step and a backpedal show. The
    game's logic, met where the demo's members hug e13's hinter band (#344's open end). e13rev
    over five stagings: a mean lag of 11.9, 20.0 and 21.5 m (Brutus, Flint, Hawk; main 13.6, 17.3,
    22.9), each build with runs where one falls 34-49 m behind at the walkway's corners.
* The door step (#346; sdm_e34's pair h_f519ca12 (-49.2, 11.8) / h_117d5fb1 (-53.1, 10.0), door 0
  h_06537687 between them, opened by the anim-trigger h_134c9934 on a detector; Hawk's Move To to
  h_f4e3da16 (-56.0, 8.2) inside: `BF_MAP=sdm_e34 BF_CHARACTER=0 BF_TEST_NO_HOSTILES=1
  BF_TEST_GOTO=-36,20,-36,20 "BF_TEST_WARP=0,me,-36,20,90;0,hawk,-38,18,90;0,flint,-37,22,90;
  0,brutus,-38,21,90" "BF_TEST_ORDER=hawk,move,-56.0,8.2@3" BF_ROUTE_LOG=1 BF_DOOR_LOG=1
  BF_AVOID_LOG=1`, 300 frames): her route goes through the pair; the detector opens the door at
  4.33 s as she nears it, she reaches the first door point at 5.07 s, the corridor between the
  two is clear by then (the leaves past 0.6 of their 1.21 s clip), "on through", the far point at
  5.73 s, GUARD at the Move To point at 6.60 s. With `BF_TEST_DOOR_SHUT=h_06537687` (the same
  run): at 5.07 s "the corridor between them blocked: stands 0.5 s, its feelers off 1.5 s"; no
  feeler steers until 6.60 s (1.5 s on), then they turn her along the leaves; with the route's next
  entry its last, the first stuck (8.73 s) ends it (FUN_00042280) and the Move To is over, GUARD
  7.6 m short. (Tex warped beside the door instead, the follow walks Hawk through it at 1.13 s with
  the door already open.) The frames of these runs show the tower beside the building, not the
  door: the logs are the check.
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
  three points off for one search, the bits back, the pace level) and the third stuck's warp onto
  the route's current point; the stuck test without a margin (#344: a jitter that comes back to
  its best never stuck, a drift away stuck after 2 s; since the fix round one asking to move and
  held to the bit stuck, one standing by choice not), and the follow's place given up on the
  third stuck; the stuck handling's marks for the follow, which holds within d0 of its leader and
  plans again beyond it; the pace near a route's points (the second fix round: FUN_00058590 at,
  within and beyond its two distances, a walk asked not raised; FUN_000587d0's square, bent and
  gentle corners, moving its way or off it, beyond 2.4 m, standing, past the walk distance in 3D,
  at a stand's level; on a route, the corner and then the end, and a Move To's distances against
  the follow's); the pace ramp (a notch every 0.17 s in FOLLOW, every 1.0 s on a Move To, once the
  drop's timer runs out; no higher than asked; the move at the lower of the two) and, since #344,
  its start at a stand with 0.3 s and its steps down after the mode asked; the follow's points on
  the leader's side, the nearest the place first. The avoidance (`cargo test --bin bf_play
  avoid`): the feelers' lengths by the pace and the speed factor (x0.945 for a factor of 1.05, as
  read, not max(1, 0.9 f)), a side block's square steer, a wall ahead's long rays and the
  hysteresis (the missed long ray's slot as read), both sides blocked, the dash's drops, walkable
  terrain, the tilt; FUN_00018b50's side; the push; the side-step; the plain direction; the move's
  turn (FUN_00019af0: at once from a stand, else at most 3.49 rad/s with the rate eased a third of
  the way a game frame, the same ease over two at 15 fps, the heading within a degree, a stand's
  direction as it is). The door
  step (#346): blocked between two door points, a stand and the feelers off; clear, nothing; a door
  point followed by a plain one, nothing.

## Not done

* The enemies on routes (#336: RETURN_HOME, PATROL's first leg FUN_0004c2e0, PURSUE, the attack
  approach with its radius budget FUN_0002e800), the script 105 walks (#337, an open code
  question). The follow's route-point states (6-16: WINGMAN's first entry walking a route to a point
  ahead of its leader on his path, FUN_0001adf0, FUN_0005acb0; [squad.md](squad.md) "The follow").
  The per-type capsule (#173: the squad's lf 0.35 m against the demo's 0.4). The prisms
  (FUN_0004cb60's records) and brain+0x3c8.
* The motion's open ends (#363): the game's members fight from attack goals (#33 take10: goal 120
  DEBUG1 from when each sees the enemy), which run and walk; the demo's keep the follow's dash,
  and a member dashing with a target holds fire (the game's dash can't aim, its MOTION's
  aim="false"; the demo's sprint plays the ready run with the gun raised, as the player's does
  after a shot). The facing override (brain+0x488, the follow's look-point near a cover point,
  FUN_00022790). The body's turn (the game's 400 degrees a second x k; the demo's 9 rad/s). The
  aim-points block (0x11d271-0x11d45e) behind the game's upper-body aim on the move. The shot
  list's live reads (`todo/363-squad-combat-freeze/shotlist.md` A-E: the gait against a member's own
  view in a fight, the push's sway, a corner's side step, a reversal's backpedal at 0.64 x) aren't
  filmed.
* The avoidance's open ends (#344): the side-step's two world lists ([world+0xa4], [world+0x12c]:
  a live read at 0x529d0 / 0x527f7 would say what they hold; the stage has nothing to step round
  until then); FUN_00019ed0's other branch, for a brain with FOLLOW_HOLD_BACK's route at
  brain+0x684 (the move straight along the facing; the demo has no such goal); FUN_00019af0's
  bit 14 of brain+0x3c4 (its setter not traced); FUN_00052120's height extents; brain+0x3c4
  bit 1 (skips the push: cleared at 0x1b15c when a brain timer runs out, its setter not found);
  FUN_00018c40's other callers (0x2a61c, 0x2a66e with a forced 3.0 m; 0x2addb, 0x2aec5). The walk's
  speed (the clip's 1.6-1.9 m/s where the game's follows the stick, #350) makes the feelers' and
  the hard turn's walks slower than the game's. The level's prisms (FUN_0004cb60's records, the
  level record's +0x114), inside which the game's AI runs instead of dashing and drops to a walk at
  a feeler's hit: without them the demo's members dash through buildings, and with the move's turn
  at 200 degrees a second a member dashing at a doorway at an angle can miss it and go round
  (#344's fix round, sdm_e34's door pair h_f519ca12 / h_117d5fb1 with three members close behind
  the leader's dash: Hawk round the building, 20 s where she went through in 2 with the free turn,
  `BF_AVOID_FREE_TURN=1`). The jump steps (FUN_00026520 / FUN_000267b0: bit 12, the hinters and the
  feelers off between a jump's points), so the demo's routes still lead to a drop they can't take
  (the stuck handling puts the member down, `progress`). Footage: todo/344's shot list (the
  feelers' hits, the pace level, a dash past a squadmate) isn't filmed.
* FUN_00059f70's and FUN_00059d40's temporary points; the 20 m look at a route's end (route+0x14,
  FUN_0001fdd0); the kinds' own steps at a point reached (patrol points, jumps; the doors' since
  #346, but its world-button branch, dead in the data); the prisms (bit 10). Of the pace near a
  route's points (FUN_0004d840, done for Move To since #344's second fix round, `pace_near`): its
  first branch, by a position of the brain's own at brain+0x1b4 (its setter not found), and
  FUN_00058590's +0x24 (set to -1); MOVE_OUT's run (mode 3) while it faces a target of its aim
  record (FUN_00045fa0's first block). The follow's walk doesn't call it (state 0 passes
  FUN_0004d970 no fifth argument, 0x22377), so it slows no follower at a corner or a door.
* Footage of a squad member at a door (todo/346's shot list A and B: the stand, the door's state
  when it's reached, the stuck steps at a door that stays shut) isn't filmed.
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
