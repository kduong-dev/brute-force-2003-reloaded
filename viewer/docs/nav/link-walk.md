*Part of [nav/](README.md), the former page `nav.md`.*

## The link walk: away from a target (#383)

`Graph::away` (bf/nav.rs; FUN_0004bbd0 with FUN_0004bb70 and FUN_0004ba00, the rows are in
[code-map.md](../code-map/README.md)): a route that is not searched for. From a place it takes, point by point,
**the first link in the current point's list that lies within a cone round a direction**, until the legs
add up to more than a length. ATTACK_FALLBACK and MOVE_BACK walk it away from a target or behind the
character (cos 0.6, 7 m: [brain.md](../brain/fallback.md) "The fallback"), MOVE_LATERAL (3 m sideways: its length is
the brain's +0x4f4, 7 by the constructor and 3 from a lateral draw) and GOAL_HOUND_ATTACK (9 and 12 m;
nothing posts it) call it too. Read from the asm; every step below is checked against take07's Ugly
(the next section).

* **The place**: a character's (FUN_0011dfa0: its position, its own point, and a flag that says the
  position is its own) or a point's. A place with no point gives no route (before the old route is
  reset: FUN_0004e8f0 runs after that test).
* **The origin** of every test is the place's own position, or the point's position for a place
  without one. **It never moves**: each candidate is held to the cone as seen from where the walk
  started, not from the point before it. (A step straight across, 90 degrees off the way, is taken
  while the point is within 53 degrees of the start's bearing.)
* **The first entry**: with the character's own position, its own point is tried as a candidate and is the
  route's first entry when it lies in the cone from the character (one behind it isn't, nor one exactly
  under it: an offset of 0 passes nothing); either way its links are the ones walked first and **the first
  leg is counted from it** (the character's way to the first entry isn't: the length is point to point).
* **A candidate** passes when it isn't on the route yet (the same point, or a place within 1e-10 on
  every coordinate squared), the filter (none in any caller) says yes, it is valid (+0x28 bit 2) and
  not avoided (bit 3), and the unit vector of the xz offset from the origin, dotted with the direction's
  x and z (its y is multiplied by 0), is **over** the cosine. **The direction isn't normalised**: ATTACK_FALLBACK's
  is me - the target, so its length across scales the dot: a target 1.1 m off needs the unit
  bearing within cos^-1 (0.6 / 1.1) = 57 degrees, one 3 m off within 78; a target within 0.6 m
  across leaves no link at all. MOVE_BACK's is a unit vector (53 degrees).
* **The order** is the list's (the data's order of the point's links, FUN_0020d220), and the first
  passing link is taken, not the best: F2b's cover point has five links, the one straight away
  (dot 1.12) is the last, the Ugly took the first that passed (dot 0.90), west.
* **The end**: the sum of the 3D legs **strictly over** the length (the last leg may carry it past:
  8.0 m for the route below); no link passing at some step is no route (reset). The global the game
  tests first, 0x3bc930, is 1 in the binary and nothing writes it.
* **No first-entry drop and no search**: FUN_00025730(route, 1, 1) starts the walk as it is
  (`routes::start`), unlike `Graph::plan`'s FUN_00041320 drop.
* **Walked** by 0x29920 (MOVE_LATERAL's update, which MOVE_BACK and ATTACK_FALLBACK reuse): FUN_00026d80
  at the radius **brain+0x4f8 = 1.5 m** (the brain constructor's value, 0x1693e; a lateral draw sets
  0.6), the steering through the avoidance and FUN_0004d970 at mode 3 with the pace level and the pace near
  the route's end, as the other goals'. The fallback's update sets the pace record's +0x24 to 0.1
  (`Router::step_time`): the ramp's notches take a tenth of a second while it runs; the pace near a
  route's end clears it.

**BF_AWAY=<x>,<z>,<dx>,<dz>[,<cos>,<length>]** (bf_level, `away_dump`): the walk from a character
standing at (x, the floor, z) with the direction as given (raw), the graph as the level starts and the
character's own point the nearest valid point; it prints the own point and its links with each one's
dot, the route with each leg and the total, then exits.

```sh
# take07 (F2b): the Ugly on its cover point, Tex 1.1 m off at (-66.09, 38.82): dir = me - Tex
BF_AWAY=-66.57,37.81,-0.48,-1.01 cargo run --bin bf_level -- e23
```
