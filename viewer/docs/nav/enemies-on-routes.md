*Part of [nav/](README.md), the former page `nav.md`.*

## The enemies on routes (#336)

`src/bin/play_goals.rs` (`Goals`, one per character, `Player::goals`), from play_brain.rs
`think` ([brain.md](../brain/goals-on-routes.md) "Its goals on routes"). A level character's brain walks its goals
over the world points as the game's goals do. The reading is `todo/242-ai-routes/leads.md` §3-§5,
with `todo/132-mutants/leads.md` §2 for the stand-off. The goals share the brain's one route record,
brain+0x930 (`Router::walker`: the goal holder's +0x8f0, the corrections below), so the AI update's
route steps run on them whatever the goal, as on the squad's: the catch-up, the stuck handling with
its detour and re-route, the route round a friend. They move as the squad does (`go`): the
avoidance's steering, the pace near the route's points, the move at the mode asked held to the
pace level, turned at 200 degrees a second. An enemy type moves by its type's speeds
(play_enemies.rs's drive, FUN_0012c6c0).

* **GOAL_RETURN_HOME** (53; start 0x345d0, update FUN_00034690): a route to the home point. Its
  start refuses for good (brain+0x3c4 bit 11) when the home point's h_1346678f (+0x24 bit 15)
  isn't the own point's or no route is found. The route is walked at 2 m (the home point's own
  radius when it has one) at the spawn's `locomotion`. At its end the home area's leash goes on
  (brain+0x21c bit 1, 0x3472a).
* **GOAL_PATROL** (4; start FUN_0002af70, update FUN_0002b490): the chain as before ([brain.md](../brain/patrol.md)
  "Patrol"). Its first leg is a route to the valid point FUN_0004c2e0 finds visible within 20 m
  (3D) of the chain's current patrol point (`Graph::visible_near`: the world's list, nearest first,
  the first whose ray, the distance + 2 m long, is clear). With none, or no route to it, the start
  refuses (0x2b348, 0x2b425) and the base goal is drawn again. The chain is walked once the leg is
  over (`Walker::chain`): at its last point an open chain is turned round and walked back from its
  second entry, and a loop goes on from its start (FUN_00026940, `ChainEnd`).
* **The attack goals' approach** (GOAL_ATTACK 7: start FUN_0002e490, update FUN_0002e9c0;
  ATTACK_VARY 8: start FUN_0002c9d0, update FUN_0002d710): a route to the target's own point (a
  character is routed to its point) that must be shorter than a budget, FUN_0002e800's: the
  distance x 2.5 under 10 m, x 2 under 25, x 1.75 under 50, else x 1.5, and x 1.2 more when the
  two are over 10 m apart in height. The search is held within it (DAT_004021a8, `Graph::search`'s
  `within`) and the length is FUN_00023460's (the 3D legs from the route's first entry). Over the
  budget, or no route: the target is unreachable for 2 s (its contact's +0x80 bit 6) and the
  character stands and fires. On its route it holds fire (both updates jump from the walk to the
  move past their attack check FUN_00024f80: 0x2f0f6, 0x2e114) and moves at the dash beyond the
  stand-off (FUN_00023e40: the type's h_150b4e02, at most the gun's reach, x0.8 for a squared
  falloff) and at a run inside it, planning again every 4 s; it stops at (the near bound
  FUN_00023da0 + 3 x the stand-off) / 4 from the target, 26.25 m for the types of 30 and 15 m (the
  Red Hand, the campaign hounds, most mutants), and stands and fires there until the target is
  beyond the stand-off again (or beyond the held gun's reach, FUN_00023d00, which the stand-off
  covers).
  GOAL_ATTACK's move is held to the pace level and slowed near its stop point (brain+0x1b4,
  FUN_0004d840's first branch: `Router::stop`); ATTACK_VARY's is neither (`FUN_0004d970(brain, dir,
  mode, 0, 0)` at 0x2e1d1).
* **GOAL_PURSUE** (10; start 0x2c630, update FUN_0002c730): once the ray to the target has met
  something for 3.75 s (FUN_0002e330's EVT_TGT_LOST_SIGHT), a route to the target's own point when
  the target is inside the home area's leash (FUN_00023630), dashed, planned again at its end,
  done as soon as a contact has been seen for 0.1 s.
* **The home area** (brain+0x1ec's record: the home point; the spawn-trigger's h_f63b86a2 across
  and h_062fbaf0 high, each at least 7 m when both are given, else 100 and 20; on from the spawn
  for a copy without a home-point, at home for one with). The sense step posts EVT_PAST_HOME_AREA
  each step the character is outside it (FUN_00044480 with FUN_00023630: beyond the radius across,
  or over half the span above or below the home point), and `default`'s and h_0bb586cf's
  personalities answer it with GOAL_PATH_TOWARD_HOME at 20.19, over EVT_ENEMY_SEEN's 18:
  RETURN_HOME's route at a run (FUN_000347b0), no fire, done once more than 7 m inside
  (FUN_000235d0: the lower of the radius less the distance across and the whole span less the
  height over the home point) or at the route's end. ATTACK_VARY's start refuses, and its update
  fails, within 1 m of the edge with its target outward (FUN_00024080: the way home and the way to
  the target, their dot under 0.3), so it fights from there.
* **ATTACK_FALLBACK and MOVE_BACK** (#383): a route of the link walk ("The link walk" below), walked
  by 0x29920 at 1.5 m and mode 3 on this same route record, so the AI update's steps and the stuck
  handling run on it as on any; its start ends the cover goal that held the character
  (`cover::end` resets the route record first, then the walk's route starts) and the cover's hook
  keeps the post while the cover holds; [brain.md](../brain/fallback.md) "The fallback" has the rest.
* **GOAL_ATTACK_COVER's walk** (19, and DEBUG1's cover half; #384, `src/bin/play_cover.rs`,
  [brain.md](../brain/cover.md) "Cover"): the pick floods from the own point through the links within 30 m
  (FUN_0002f270 with FUN_000594a0, `Graph::flood`) for the first `<cover-point>` its filter
  passes, then a route to it (FUN_000416a0) on the same route record, walked at 1 m an entry and
  0.15 m its last (FUN_0002fd40) at a run, the dash when the point is over 5 m off, the pick was
  over 13 m and the character is under half its hit points; the look stays on the target unless it
  dashes, so it backs and side-steps to cover facing the enemy. The point is claimed (+0x28 bit 0)
  while it's held.
* **GOAL_ATTACK_GUARD from the stuck handling** (FUN_00042620): after a detour that found nothing,
  a brain out of the squad whose target was seen in the last second stands and fires for 6 s
  instead of the re-route; its goal is done and starts again after.
* **The own point's far pick leaves the patrol points out.** They aren't on the world's list
  world+0x110 (FUN_000daa10, the corrections below), which FUN_000ab260 and FUN_0004c2e0 search.
  Before #336 a patroller standing by its chain could get one as its own point: a point with no
  links, from which no route leaves.
* The demo's stall guess (a walk given up after gaining under 0.1 m in 1 s) is gone: a walk that
  can't be routed refuses its goal, as the game's starts do, and a stuck walk is the stuck
  handling's.

The test map has no world points: an NPC set to fight there (`BF_TEST_NPC=...,fight`) walks
straight at its target as the approach would, its budget always met, with no pursuit and no
leash ([testtools.md](../testtools/npcs.md)).
