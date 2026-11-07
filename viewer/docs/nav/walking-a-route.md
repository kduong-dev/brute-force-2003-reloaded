*Part of [nav/](README.md), the former page `nav.md`.*

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
