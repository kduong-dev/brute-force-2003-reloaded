*Part of [brain/](README.md), the former page `brain.md`.*

## The Confed marines

The V-squad's two marines (Stout and Cogburn: trigger_spawn64 and 76, `h_0fa4f510` false, so the
member check keeps them and sets their flags, [triggers.md](../triggers/spawn-triggers.md)) farcast in during
tutorial_farcast_vsquad_short (7.57 / 7.87 s), held by its 119s until it ends; then they follow
Tex with the squad AI's wingman follow (`squad_ai`; since #331 the game's GOAL_WINGMAN_FOLLOW
states, play_follow.rs, [squad.md](../squad/the-follow.md) "The follow": held while Tex is within 11.2 m, a
route to his own point at the dash once he's farther, stopped 9.8 m from him once he stops, a cover
point near him taken when there's one, kneeling or looking about while they hold; their mode never
changes, so they join no formation list and follow him; #243, closed). The tutorial's
yard (t335test's recipe): they reach 9.5-9.7 m from Tex at 85.7 and 88.9 s (main's formation
places within 8 m of the yard's end at 87.1 and 90.1 s), Cogburn walking to a cover point by him,
both behind him where main's stood in front. They're on the player's team (0,
their placements' alignment-type), see by the sensor above (90 degrees either way of their
facing), fire at what they see with their aim (above), and the Red Hand fire at them. Not done:
switching to them (the d-pad), their portraits, the squad commands. The test map's NPCs can follow
the controlled character the way they do, without a brain (#364, [testtools.md](../testtools/npcs.md)
"Follower": the same follow, no formation list, no squad slot).

Their walk to their h_131de144 points: the spawn makes those points their homes (wpoint1380 and
wpoint1006, 0.5 and 0.9 m from their pads), but without a home-point nothing sends them there,
and a wingman follows its leader. take11's watch log agrees: the two stood on their spawn points
to 0.1 m (-16.9, -75.4) and (-21.1, -76.3) from the farcast (29.3 s) until Tex moved off at
156.9 s, set off 1.6-2.2 s after him and kept 8-40 m behind him in bursts of a run, and stopped
9-10 m behind him when he stopped. The demo's wingman follow (#331) sets them off when Tex is
beyond d0, 11.2 m from them: 1.16-1.23 s (Stout) and 1.92-1.99 s (Cogburn) after his first step,
against take11's about 1.5 and 2.1 s (its watch log samples every 0.54 s), so within the take's
sampling and without its ~0.15 s reaction (the #243 check, 2026-11-09). The script's 105 walks to those points
(0.26 s) name the marines before they're spawned and do nothing ([cutscenes.md](../cutscenes.md)).
