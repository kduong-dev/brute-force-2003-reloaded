*Part of [nav/](README.md), the former page `nav.md`.*

## The squad on routes (#335)

`src/bin/play_routes.rs` (`Router`, one per character, `Player::router`), from play_follow.rs
(the follow, FOLLOW and WINGMAN, through play.rs `squad_ai`, and the marines' brains), play_orders.rs `move_out`
(Move To) and play.rs `update_player` (the own points and the AI update's route steps). Every
function is in [code-map.md](../code-map/ai.md), "AI".

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
* **The follow** (#331, play_follow.rs; the states and the formation in [squad.md](../squad/the-follow.md) "The
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
