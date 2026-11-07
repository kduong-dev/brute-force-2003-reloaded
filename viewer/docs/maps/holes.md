*Part of [maps/](README.md), the former page `maps.md`.*

  * **Holes (#292: the game's rule found, not applied; #296).** Where a character has nothing
    under it (no floor, no terrain too steep to stand on, no rock's flank: a hole in the level's
    collision, or past the terrain's edge), the demo puts it back where it stood, as at the edge
    of the map: an invisible wall round the hole, older than #285 (62b6cd9 the same).
    * sdm_e34's chasm, between the mountains east of the valley (x 44-70, z -5 to 16, the valley
      floor beside it at 29-32 m), is such a hole, with no blocker or trigger near it: running
      north from (56, 32) Tex stops at its rim, (55.95, 15.67), and sliding down the mountain's
      face west of it (`BF_TEST_GOTO=30,5,60,5`) he's held at its foot, (40.52, 8.53). The drawn
      terrain has the hole too: the background's colour shows through.
    * What the game does at a hole, read in its code: it has no edge. Its sweep meets nothing
      there and sets the falling flag (FUN_00153810, 0x1539de), and past the ledge grace
      (FUN_0012b950) the character falls. A living character under -500 m (0x3a5524) is out of
      its world: a game object's update (FUN_00157c40) refuses it and the character's
      (FUN_0011caa0) returns before its physics step, so it hangs there.
    * Not applied. #292's first round let a character fall through (a JUMP_FALL to a hang at
      -500 m, taken out); the test against it found the chasm's fall a flat field of the
      background's colour and then a hang with no way on, and new falls near the levels' starts:
      sdm_e34 about 10 m south-west of its start, at (-5.25, 44.93), m02_a's west wall, twice on
      m14_a and once on m14_b. Those holes look like terrain the demo doesn't load: 17 of
      sdm_e34's 64 terrain blocks name no collision surface at all (`<blocks h_051ac63e="">`),
      and its silo stands over a hole. #296 looks for it; until then the put-back stays for
      every character.
    * Where (`BF_FIND_HOLES`): every level with terrain but e10, e13, e23, m01_a, m01_b, m05_b,
      m05_y, m07_c, mp4, mp6, mp7, mp8, sdm_e10 and sdm_e13 has holes, nearly all of them the
      terrain's outer edge behind the mountains round the level (sdm_e34 908 rim metres, m09_c
      3122). The scan's upper bound (it climbs any slope a floor allows, as no slide holds it)
      counts most of each edge as reachable; in play the slides on the slide-flagged slopes keep
      the player off most of them.
