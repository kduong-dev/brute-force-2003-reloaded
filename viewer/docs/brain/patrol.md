*Part of [brain/](README.md), the former page `brain.md`.*

## Patrol

GOAL_PATROL (start FUN_0002af70, update FUN_0002b490; the walk FUN_00026d80, a point reached
FUN_00026940):

* **The route.** The level's patrol points (`bf/level.rs` `PatrolPoint`: `<patrol-point>`, kind
  11, and `<patrol-first-point>`, kind 12, a chain's head; the world keeps both kinds in one
  list, world+0x1d0) are linked by their forward-object and backward-object and grouped by their
  `iD`. The start takes the point nearest the character (3D, FUN_000394b0), the first head of that
  point's iD, and the chain from the head along the forward links until there's none or it comes
  back to the head; it starts at the nearest point's place on it (the head's when it isn't on it).
  Under two points it refuses.
* **The walk.** To each point in turn: the stick at half (FUN_0004d970 mode 2), a full walk by
  FUN_0012c6c0 (the type's walk speed, 3 m/s for the Red Hand), the relaxed walk clip, turning at
  the type's turn-rate (90 degrees a second); a point is reached within 2 m across (FUN_0002b490
  passes 2.0; a point's own h_e6564e8a when it's above 0: the four by the gate, 1 m).
* **The ends.** A point with no forward or no backward link ends the chain: the route is turned
  round (FUN_00023320 -> FUN_00038d30) and walked back. A chain whose last point links on to the
  head is walked round and round (FUN_00012650). The tutorial's two chains are open: walked to and
  fro.
* **The first leg** (since #336): the patrol's start routes the character to the valid point it
  can see within 20 m of the chain's current point (FUN_0004c2e0) and walks the chain once it's
  there; with no such point, or no route, the start refuses and the draw goes again.
* **In scripted play** (the world's cutscene bit, from a script's 122 to its 123) a character
  moves by its clip's root, at the rate it had, not by the speeds (FUN_0012cb40): the first patrol
  walks at its walk clip's 1.86 m/s through tutorial_show_patrol (play_enemies.rs `steer`).
* A point's trigger-object would be sent an event as a patroller reaches it (FUN_00026940); no
  tutorial point has one (logged).

The tutorial's two chains:

| iD | Head | Points | Walked by |
|---|---|---|---|
| 1 | h_11daf471 (18.8, 35.2) | patrol_point3 (11.1, 44.5) north to patrol_point11 (52.2, 76.9): 10 in all | TUTORIAL_terrorist_2 and _18 (from the head: their spawns are 2.5 and 6.6 m from it); one round trip ~66 s |
| 2 | h_18ddc4e7 (-37.7, 0.1), by the gate | patrol_point30 to 33 round the gate, 14 to 20, 29, 21 through the base to (-78.6, 47.6): 14 in all | TUTORIAL_terrorist_6, from the point nearest its home |
