*Part of [nav/](README.md), the former page `nav.md`.*

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
  `BF_FOLLOW_LOG=1`'s (#331, [squad.md](../squad/the-follow.md)).
* `BF_GOAL_WATCH=<s>` (bf_play, #336): every s seconds of the game's clock a row for each level
  character with a brain (not following) and each test map NPC set to fight: the goal it runs
  (RETURN_HOME, PATROL, PATH_TOWARD_HOME, the attack's approach or stand, PURSUE, ATTACK_GUARD),
  where it is, its own point, its route's entry, its target and how far, its pace level, the mode
  asked and its speed. `BF_BRAIN_LOG=1` logs each goal's start, refusal and end, the approach's
  budget and stop, the home area's size at the spawn and each time it's left.
* `BF_GOAL_WATCH`'s rows since #383 also give the character's facing (degrees, the demo's yaw: 0
  down -z, positive to the left) and the motion it plays (`motion BACKSTEP`, `-` with none); ATTACK_FALLBACK
  and MOVE_BACK read as their own goals. `BF_TEST_MOVE_BACK=<s>[,<name>[,<x>,<z>]]` starts GOAL_MOVE_BACK
  ([brain.md](../brain/test-hooks.md), test hooks).
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
