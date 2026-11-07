*Part of [nav/](README.md), the former page `nav.md`.*

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
