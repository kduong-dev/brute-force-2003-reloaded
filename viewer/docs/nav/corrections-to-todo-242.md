*Part of [nav/](README.md), the former page `nav.md`.*

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
* (#336) The world's point list world+0x110 isn't every kind: FUN_000daa10 keeps kinds 8, 11, 12,
  15, 17, 18 and 23 on lists of their own only (the patrol points on world+0x1d0), so FUN_000ab260
  (the own point's far pick) and FUN_0004c2e0 never take a patrol point. The search itself walks
  links, and no patrol point has one.
* (#383, todo/26's leads §5) FUN_0004bbd0's cone is measured **from the walk's start** (the character's
  position, or the start point's), not from the current point; its `dir` is the raw vector (me - the
  target for ATTACK_FALLBACK), not a unit one, so the 53 degrees are 53 degrees only for a direction of
  length 1; the place's own point is the first entry only when it lies in the cone; the length is passed
  when the legs are strictly over it; brain+0x3c4 bit 14 (set by FUN_00029a00) is read by FUN_00019af0 as
  "start the move afresh", not "the facing held"; the walk's arrival radius is brain+0x4f8, 1.5 m by
  the brain's constructor.
* (#336) The pace's first branch, brain+0x1b4 (FUN_0004d840), is set by the attack goals'
  approach: FUN_0002e9c0 writes its stop point there (holder+0x174).
