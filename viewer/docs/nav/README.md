# AI navigation: world points and routes (`src/bf/nav.rs`, `src/bin/play_nav.rs`, #334)

*This folder was the page `nav.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../nav.md` is now a pointer to them.*

How the game's AI finds its way: the level's world points as a graph, the route search over it,
where a character's routes start, and how a route is walked. The first slice of #242 (AI
routes, #334) is the shared machinery; since the second (#335) the AI squad's follow and its Move
To walk routes ("The squad on routes" below, `src/bin/play_routes.rs`), steered through the AI's
avoidance since #344 ("The steering", `src/bin/play_avoid.rs`); since the third (#336) the level
characters' goals walk them too ("The enemies on routes", `src/bin/play_goals.rs`); since #383 their
fallback from a target too close walks a route of another kind, the link walk ("The link walk",
`Graph::away`, `src/bin/play_fallback.rs`). The script walks are #337; the Move To cursor's filter takes a point's valid bit from the graph
([squad.md](../squad/README.md)).

Everything is read from default.xbe (static, not run): the route request FUN_000416a0, the search
FUN_00059000, the own point's upkeep FUN_0014f860, the walk FUN_00025730 / FUN_00026d80, the
waypoint trigger 0x202c50; every function is in [code-map.md](../code-map/ai.md), "AI". The reading is
`todo/242-ai-routes/leads.md` (the analyst's), with the corrections at the end of this page. take01 of
`todo/242-ai-routes` logs the game's squad routes round a lava pit ("The squad on routes").

**Sections**

* [The graph](graph.md)
* [The search (FUN_00059000)](search.md)
* [A route (FUN_000416a0 with FUN_0005a440)](route.md)
* [The own point (FUN_0014f860)](own-point.md)
* [Walking a route (FUN_00025730, FUN_00026d80)](walking-a-route.md)
* [The squad on routes (#335)](squad-on-routes.md)
* [The steering: the AI's avoidance (#344)](steering.md)
* [The door step and the corner hold (#346)](door-step-and-corner-hold.md)
* [The enemies on routes (#336)](enemies-on-routes.md)
* [The link walk: away from a target (#383)](link-walk.md)
* [Lava](lava.md)
* [Test hooks](test-hooks.md)
* [Verified](verified.md)
* [Not done](not-done.md)
* [Corrections to todo/242's leads](corrections-to-todo-242.md)
