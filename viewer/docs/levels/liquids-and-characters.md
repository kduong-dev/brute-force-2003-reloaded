*Part of [levels/](README.md), the former page `levels.md`.*

* **Liquids aren't solid.** A pool (sdm_m03's sludge river, sdm_e13's lava, the tutorial's pits,
  and pools on sdm_e10, sdm_e40, sdm_m07, mp4, mp6 and mp7) is a placed object of class
  `h_04366a6a` (`water-lava-flow-object`) whose collision is a closed volume under its surface
  (sdm_e13's lava a box from 12.94 to 23.89 m; sdm_e10's pond and the e40 ocean 40- and
  420-triangle volumes): the game makes it a phantom (FUN_002218e0), so it's left out of the floor
  and walls here, and characters wade and sink to the bed. No movement code reads it: no
  slowdown, swimming, drowning or tint under it (todo/146-liquids leads §4). A liquid's type
  (objecttypes `<h_fa2f5452>`, `water-lava-flow`, loaded by FUN_00193420) gives its kind
  (`liquid-type`, the LT_ names at 0x3be754) and three damage values (`LiquidType::damage`):

  | liquid-type | Liquid | h_06f6a40d: an object's hit | h_eba69d33 | h_e0489ea0: a character's rate |
  |---|---|---|---|---|
  | 0 | ocean water (e40) | 0 | 0 | 0 |
  | 1 | swamp water (sdm_e10's pond, the e40 pools) | 0 | 0 | 0 |
  | 2 | lava (sdm_e13, sdm_e40, sdm_m07, the tutorial) | 50 | 50 | 30 |
  | 3 | toxic waste (m09, m14) | 0 | 10 | 30 |
  | 4 | sludge (sdm_m03, m03, e01, mp6) | 50 | 50 | 50 |
  | 5 | chloride (m14_c) | 0 | 0 | 0 |

  h_eba69d33 is read by nothing; h_06f6a40d is one hit on an object entering (a pickup, a
  grenade, a missile: #146's Part 5, not in the demo).
* **What a liquid does to a character** (#152, play_liquids.rs; the code read in
  todo/146-liquids leads.md §2-§4 and answer-spec.md Q1 / Q9, the code map's Levels rows
  FUN_00221af0-FUN_00222400):
  * The liquid sees a living character by its eleven hit-part bodies (PID_BONE), never by its
    movement capsule (answer-spec Q1; L1's live read on e40's pools: 16 enter events, all
    PID_BONE). A part is in while it's in the volume, however deep: here, while the lowest corner
    of its collision surface, posed as drawn, is inside the volume (`Arena::liquid_inside`: an
    odd number of the liquid's faces above it). The count is the game's char+0x104.
    * Each face counts once. A point on an edge or a corner its triangles share is over one of
      them alone (`Arena::over_once`, a half-open rule with no slack): the slack the floor test
      gives an edge counted both halves of a split face near their diagonal, an even count, so
      a part crossing the diagonal of sdm_e13's lava top (x + z = 0, some 4 cm either side) went
      out and back in, a whole entry hit each time. And a twin pair (a face that comes twice,
      wound both ways) counts once: the e40 ocean's 420 triangles are 210 such pairs, as the e40
      lava flow's 168, m02_a's lava's 720 and e05's water's 184, so before they held nobody.
    * Every liquid of the 19 maps that have one closes round its volume (`BF_FIND_BODIES=1`: no
      open edge, a twin pair counted once; sdm_e10's pond and its copies 40 triangles, the boxes
      12, m03_a's 8); sdm_m07's four lava quads are open sheets, under which a point is in.
  * The damage is the character rate alone, once a game frame (30 Hz, the liquid's update): a
    frame on which a part enters deals the whole rate, any other with a part in its share of
    the second, and one hit a frame at most (FUN_00221a40). No depth ramp, no tick, nothing
    more for going under: a body fully under keeps burning at the rate. Every hit, an entry's and
    each frame's share alike, goes through the character's damage handler as DTYPE_FLAME
    (0x221780 -> 0x125600; play.rs `burn` through `take_hit`): a stamina shield takes it first,
    nothing in a cutscene or out of the world, the difficulty step (the multiplayer maps none;
    STANDARD's squad x0.6), Vengar, the type's factor, BF_COMBAT_LOG's line. It floors nobody.
    Each hit asks its hurt line of the chatter gate (#155, [chatter.md](../chatter.md): a record
    with no attacker and +0x5c bit 2, ef32191d, 50% with a 20 s replay delay), so one is said at
    the entry and none after while the character stands burning (todo/5-liquid-sounds leads
    §4.1: no sound between the foot events in take03; §4.3: a voice-like burst after the first
    entry hit in both takes).
    Its blood is the game's decals alone (FUN_00124a70: the entry hits are big enough, the
    shares never; the spray straight up), and a death in a liquid leaves out the demo's death
    spray (play_fx.rs). The game frames run between the drawn ones with the parts' lowest points
    between their two places, so 15 Hz captures and a 60 Hz screen deal what the game's 30 Hz
    does.
  * A fall into a liquid hurts by the drop to its bed, as on land (FUN_0012b740 has no liquid
    check; answer-spec Q9); only the landing's sound (and dust, which the demo doesn't draw) is
    left out while a part is in one (0x12d842).
  * Checked against the takes (`BF_LIQUID_LOG=1` prints the parts in and each hit):
    * #146's take01 (sdm_e13, Brutus at 75 HP walking north into the lava at half stick:
      `BF_MAP=sdm_e13 BF_CHARACTER=0 BF_TEST_GOTO=-19.97,-122.80,-20.92,-100,25.09 BF_TEST_WALK=1
      BF_TEST_HEALTH=75`): the first part in at (-20.3, -116.0), 75 to 45; the second 0.24 s
      later (the take's 0.20), down to 9 (the take's 9.3); dead 0.54 s after the first (the
      take's 0.53), ankle-deep.
    * take03 (Brutus at 5000, `BF_TEST_HEALTH=5000`, to (-21.44, -110.15), then on to
      (-23.73, -87.26)): thirteen entry hits of 30 walking in (the take's twelve or thirteen),
      then 9 of 11 parts in waist-deep (the take's 10) at 30.0 HP a second (the take's 30.0); all
      11 in, 3 m under, still 30.0 a second and alive (the take's 30.2, alive 9 s under). The view
      from under the lava shows no surface (one-sided, as the take).
    * #28's take08 (the tutorial, STANDARD: `BF_MAP=tutorial BF_TEST_GOTO=46.26,-25.92,45.96,-30.40
      BF_TEST_HEALTH=101.6`): Tex runs off the east pit's rim and falls in: five entry hits of 18
      (30 x 0.6), 101.6 to 83.6, 64.4, 45.8, 27.8, 9.8 (the take's 83.0, 64.5, 45.9, then 36 in
      one step to 9.9), then 18 a second on the bed, dead 0.80 s after the first hit (the take's
      0.87). The same at 15 Hz and 30 Hz capture steps. (With the goal at z -34, past where the
      take's Tex stopped at -30.4, and his full 115, he runs on over the bed and up out of the
      pit's south side with 4.6 left.)
    * Across sdm_e13's lava diagonal (`BF_MAP=sdm_e13 BF_CHARACTER=0 BF_TEST_GOTO=-12,2,-2,12
      BF_TEST_WALK=1 BF_TEST_HEALTH=5000`: Brutus starts 4 m under the surface and walks across
      x + z = 0 at about 4.6 s): one entry hit as he starts, then 30 HP a second to the end. With
      the old slack the same walk took seven more entry hits of 30 between 4.60 and 5.20 s, as his
      parts crossed the diagonal.
  * Not filmed: sludge's rate (50, sdm_m03: C1), toxic waste's (30, m09: D1), a fall into a pool
    (B3), squad members or enemies in a liquid; built from the code and the data alone. The
    waters do nothing (their rates are 0).
  * The tutorial's lava pits (h_ea243aea, two) are a box of collision (12 triangles, the east
    one's top at 14.40 m, its bottom at 3.45 m) in a pit whose walls are steeper than a floor
    (maps.md, "Collision": steep ground with no floor under it, #264).
  * The AI keeps out of the liquids that burn (#1): squad.md, "Liquids that burn".
  * Test hooks: `BF_LIQUID_LOG=1` prints each character's parts in a liquid as their count
    changes, each entry hit, a death, the hit points each second in one, and a wingman waiting at
    the shore of one (#1); `BF_TEST_WALK=1`
    makes BF_TEST_GOTO walk (Ctrl), as the takes' half stick; `BF_TEST_HEALTH=<hp>` above the most
    raises the most too (play_pickups.rs).
