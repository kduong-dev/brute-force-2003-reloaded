# Playing on a map

*This folder was the page `maps.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../maps.md` is now a pointer to them.*

`bf_play` now runs on Battle of Bulgar by default (`BF_MAP=<level>` picks another level,
`BF_MAP=flat` brings back the checkered test floor, `BF_START=<n>` picks a start point,
`BF_TEST_GOTO=x,z,x2,z2[,y]` starts the player at one point (on the floor below height y if
given, e.g. a roof; with no floor below it, on the steep face there if there's one, #264;
without y on the terrain, or where there's none on the highest floor, #292; kept on the levels
whose spawn-triggers place the squad, #292) and runs them toward another from 1 s on (`BF_TEST_GO_AT=<s>` sets off at
that time instead: #35 take08's dash after standing 4 s, skills.md), or stands still when both
points are the same;
`BF_FIND_STEEP=1` lists slide surfaces steeper than 50° and ledges with a drop of more than 6 m,
then exits; `BF_FIND_BODIES=1` lists the collision bodies (size, closed or open, twinned, how
many faces face into the body; blockers and hinters, a placed object's object-type, its faces by
the way they face) and exits, `BF_FIND_BODIES=b<n>` lists body n's triangles (#292);
`BF_FIND_HOLES=1` or `=<file>.ppm` scans the map for holes in its collision and for the hinters'
walls and exits (#292, "Holes" below); `BF_FIND_HINTER=1` scans the terrain's extent every half
metre for the spots where a character's floor or flank (one over its floor by more than a
step) isn't the same for the AI and the player (a hinter's faces, #341) and exits; `BF_COLLISION_CHECK=1` counts the collision surfaces the
map uses (terrain blocks, blockers, hinters, placed objects' parts) that aren't the map's own
and exits (#338, "Which level's shapes" below); `BF_HELD_LOG=1` prints what stops the controlled
character's move (#292); `BF_INSIDE_LOG=1` prints a character in a rock and one taken out (#285);
`BF_SLIDE_LOG=1` prints slides, falls and landings; `BF_ALE_LOG=1` prints particle emission;
`BF_ALE_DUMP=<effect name>` prints each particle that effect makes as it's born (where, how high
above its effect, its velocity and life: "Placed effect objects", #290);
`BF_NO_IDLE_FX=1` leaves out the power-ups' effects; `BF_NO_PLACED_FX=1` the level's placed
effect objects, `BF_PLACED_FX_VARIANTS=1` starts the ones held out, `BF_LEVEL_DUMP=1` lists
them):

**Sections**

* [Spawning, and the level's triggers](spawning-and-triggers.md)
* [Collision: where the surfaces come from, which level's shapes (#338), material, standing](collision.md)
* [Steep ground with no floor under it (#264): the ledge grace, the fall, the face](steep-ground.md)
* [Rocks and boulders (#285)](rocks-and-boulders.md)
* [Blockers and hinters (#292, #341)](blockers-and-hinters.md)
* [Holes (#292, #296): the game's rule found, not applied](holes.md)
* [Collision: test hooks (#292), what was checked, what isn't solid, the fallback](collision-test-hooks-and-checks.md)
* [Doors and gates (#261, #262)](doors-and-gates.md)
* [Gates, their wall panels and consoles (#262)](gates-wall-panels-and-consoles.md)
* [HUD text, the movement constants from default.xbe, sliding](hud-text-movement-and-sliding.md)
* [Music](music.md)
* [Shots: each weapon's effects](shots.md)
* [Look (the game's lighting), power-ups](look-and-power-ups.md)
* [ALE blends (#228), particle facing (#286), the effects the triggers make](ale-blends-and-particle-facing.md)
* [Placed effect objects (#258)](placed-effect-objects.md)
* [Pickups, compound joints, winding, the gate and texture address modes, invisible or glowing materials, hits](objects-and-materials.md)
