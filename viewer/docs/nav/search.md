*Part of [nav/](README.md), the former page `nav.md`.*

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
