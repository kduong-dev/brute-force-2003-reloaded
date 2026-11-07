*Part of [nav/](README.md), the former page `nav.md`.*

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
