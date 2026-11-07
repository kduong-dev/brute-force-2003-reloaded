*Part of [maps/](README.md), the former page `maps.md`.*

  * Rocks and boulders (#285; the code: FUN_00153810's sweep, [code-map.md](../code-map/characters-and-motion.md)
    "Characters and motion"). Before #285 a character could run into a boulder and not get out.
    * The rocks: placed scenery models (game-objects; `BF_FIND_BODIES=1` lists every body of a
      map and exits), each a closed shell of 330-660 triangles (h_e729625c, h_0a4ac769,
      h_05736b8b, h_e117fe28, ...; down to 40 for the small h_09a6856d), not convex hulls. Like
      nearly everything in the data (all the terrain, every blocker, most placed objects) their
      triangles come twinned: each also wound the other way, as Ipion keeps a lone triangle as a
      flat ledge of its two sides, so of each pair one faces out of the rock and one into it. The
      terrain is a height field with no rocks of its own; there are no physics blocks among them.
    * Why a body got in: the wall test pushed the body's spheres out from the nearest point of
      each steep triangle where the move ended. A move that carried the body's middle past a face
      in one frame (a sprint, a dive, a knock: more than its 0.4 m radius at 15 fps) left it
      behind the face, where the push sent it further in; inside, the inner faces held it, and the
      twins of the rock's underside facing up into it were floors to stand on. A thin wall went
      the same way: sprinting slantwise at sdm_e34's barracks
      (`BF_TEST_GOTO=-70,-12,-90,8 BF_TEST_SPRINT=1.0`) took Tex through its wall into the
      building at (-75, -8).
    * Which (before, with `BF_INSIDE_LOG=1`: frames the body ended inside): sdm_e34's boulder
      h_e729625c at (0.1, 69.5) let Tex in sprinting (trapped from 1.7 s), diving (5 frames) and
      knocked into it (trapped); e34's h_09a6856d at (-59.9, 34.2) sprinting (trapped); the
      tutorial's h_e729625c at (54.0, 16.6) sprinting and diving, and its h_0a4ac769 at (56.7, 5.1)
      sprinting (trapped). A run (5 m/s) didn't get in, nor did a dodge. e40's low boulders
      (h_034df7ff, h_142e34fd, h_0f18f5e6) are walked onto, before and after.
    * **Swept, as the game does it.** The game moves a character's body (a vertical cylinder)
      along its move and stops it before it passes into a face: nothing lets it in, and nothing
      pushes it out (FUN_00153810, FUN_00152f20, FUN_00152d20). Now a character's move each frame
      is swept from where the last frame left it (`Arena::move_body`): a circle of BODY_RADIUS
      against each wall's part between STEP_UP over the feet and the head, seen from above. It
      stops at the first wall the move goes into (its direction into the contact under -0.01),
      however far the move goes, and the rest of the move slides along it (its part into the
      wall taken out, no friction: FUN_00150f90 / FUN_00168950 / FUN_00151170), up to the
      game's passes: 4 times for the character a player controls, 3 for the AI's (#292,
      FUN_0012cd10; it was 3 for every one). Each push of the squad apart is swept too, and a
      slide's move, at the height of the feet where it starts. What's left overlapping a wall
      after (a door shut on a body) is pushed out from the front, as before, by the body's radius
      at most, unless that puts the body in a rock. One in the air whose move across would end in a rock's solid (into
      a rock's slope, which isn't a wall, rising over its feet faster than it falls) stays where it
      was across and comes down on the slope. The game raises the cylinder's bottom 0.4 m on the
      ground and 0.1 m in the air for a steep triangle only and meets any other with the whole
      cylinder (FUN_00152d20); the demo cuts every wall at STEP_UP (0.5 m) over the feet, so a
      ceiling lower than that over the feet isn't met.
    * **Walls by their winding.** A triangle is met from its front only (the game leaves out one
      whose plane has the body's point behind it: the feet for a steep triangle, the middle for
      others), its front by the winding ((v1 - v0) x (v2 - v0), FUN_001516c0). A contact with a
      wall's face pushes along the face's normal across, as the game's does (FUN_00150a10), and
      one with its edge or corner away from it; the demo asks for the body's point to be 1 cm in
      front, and leaves out a face contact whose normal faces away from the body, so the twin of
      a slope facing into the ground never holds a body walking up it (a bump ahead can put the
      feet in front of it). A lone face (a few small hulls: sdm_e34's h_fc2033e6 and h_04c527a0,
      e34's h_f809f8de and h_fbad6fba) blocks only from its front. A wall is any triangle that
      isn't a floor by its winding: a steep face, and the underside of an overhang (a ceiling,
      normal y at or under cos 130), which the game meets with the whole cylinder.
      * A rock's faces that face into it (`Tri::inner`: of a twin pair in a closed body, the one
        with a point 5 mm in front of it inside the rock) are neither walls nor floors. The game
        never has a body inside a rock to meet them from their front; the demo met them: the twin
        of a rock's underside, facing up inside it within the step from the ground, was a floor
        that a body walking at the rock's foot stepped up onto, inside (#285's test: sdm_e34's
        boulder h_e729625c from the north, `BF_TEST_GOTO=0.10,65.50,0.10,77.50
        BF_TEST_SPRINT=1.0`, went in and out 57 times in 120 frames; e40's h_05736b8b at
        `113.78,68.74,134.64,76.34`; dives and sprints at sdm_e34's boulder), and from inside
        they held a body there. Of a pair that tests in both ways or neither (a doubled or thin
        part: one triangle of each of e40's and the tutorial's low boulders h_0f18f5e6 and
        h_142e34fd, the blockers round some objects) both are kept. On sdm_e34, e34, e40, the
        tutorial and m09_a, 6268, 12200, 14098, 8620 and 8632 triangles face in.
      * The twin facing down of a floor that's stood on (`Tri::floor_back`: the terrain's, a
        rock top's) is tested at the body's middle, as a ceiling's: the demo's rule. By the game's
        split that twin of a 50-65° slope is steep and tested at the feet, and where the slope eases
        ahead its plane is over the feet, so the sweep met it and held a body walking up to the
        junction (#285's review: a 60° slope easing to 53°, and a 55°-52°-45° flank, held 0.4 m
        short of the junction; `cargo test --lib up_a_twinned_slope`). A rock's underside keeps the
        game's split (its twin facing up is inside the rock).
      * A floor faces up by its winding, so a lone hull's underside holds no one, and the floor
        under the feet is the twin facing up, which carries the material and the slide flag
        (`Arena::ray_material`), as the game's ground is: its ground contact is the lowest contact
        whose normal is within 65° of up (FUN_00153410), and that contact's triangle header gives
        the material and the slide flag (FUN_0012dab0); the twin facing down has its normal down
        and its plane behind the body, so it's never one. Before, both twins were floors and a
        rounding tie picked either: the twin facing down (material 0, no slide flag) wherever its
        height came out higher (on e40's canyon rim at (-54.23, 84.80), by 4 µm). So on every
        level slides on slide-flagged ground start as the game's would, sooner than before (#285's
        test: `BF_MAP=sdm_e34 BF_TEST_GOTO=30,10,-30,10` slides at 2.47 s on a 36° face, where main
        slid at 3.00 s on a 57° one; e34's `5.47,35.95,30,60` at 2.93 s against 3.13 s; 26,878 of
        e34's 30,962 terrain triangles carry the flag).
    * **Climbing onto a rock.** The game's sweep moves the body up a slope with its move, so a
      rock overhanging the slope stops it. The demo moves level and raises the feet to the floor
      after, so a step up that would raise the body more than 5 cm into a wall isn't taken
      (`Arena::clear`, `CLIMB_SLACK`, in `ground_walk` and the ground loop: the sweep leaves a body
      touching a wall it walks along, and where the wall leans over the slope a rise of a few
      centimetres brings it inside the radius; with no slack each such step was refused, #285's
      test on sdm_e34's outcrop h_f5270370), and a body on its feet that the floor carried into a
      rock's solid (a dome rising faster than the step along one move) goes back where it stood.
    * **A rock's steep side.** A body with its middle over a rock's face too steep to stand on,
      more than a step over the floor under it (`Arena::flank_below`: a face of a placed object or
      a blocker facing out, normal y from 0.05 to cos 65), is on it as on terrain too steep to
      stand on (#264's `slide_on_face`; not one the AI steers): stepping onto it starts a fall,
      and it slides down the face to the floor below. Before, it fell straight through the rock
      to the floor under it and was taken out at the bottom, half a metre or more sideways at
      once (#285's test: 108 drops of 4 and 9 m onto the tops and sides of six rocks of sdm_e34,
      the tutorial, e40 and e34, 5 taken out; now none). The game's sweep meets the body with the
      face and moves it off along it. A rock's steep face is met from where the body's middle is
      in front of it as well as its feet (`WallSection::also`, the demo's), so the face over the
      step pushes a body on the flank off, and the face's steps (`face_walk`) follow a rock's
      flank as they do steep terrain.
    * **Out of a rock.** One in a rock's solid anyway (a spawn or a test teleport there, or a
      fall's drop past a rock's side that it slid off under the rock's widest part) is taken out
      at the start of the collision, before it's drawn, to the nearest place clear of it
      (`Arena::escape`: of 32 ways round, the one where the rock's surface is nearest at the
      heights of its body that are inside, then its radius beyond; its feet on the floor there,
      unless it's in the air). The game's sweep would slide a falling body off along the rock;
      here it's one step sideways.
      Inside is told by the rock's surface (`Arena::inside`): a body whose faces, a twin pair
      counted once, close round a solid (no edge left open; not a door's, not a liquid's) and that
      the vertical lines below and above the point each cross an odd number of times.
      `BF_INSIDE_LOG=1` logs a character ending a frame inside one and each one taken out.
    * The fall onto a rock's top is the air step's, which lands on the floor it was over (a 6 m
      and an 8 m drop onto sdm_e34's boulders land on their tops).
    * Test hooks for the moves (with `BF_TEST_GOTO`): `BF_TEST_DODGE=<s>[,<s>...]` presses the dodge
      at those times (to the right, standing), `BF_TEST_DIVE=<s>` dives toward the goal then (as
      from a grenade behind); `BF_TEST_SPRINT`, `BF_TEST_KNOCK` (thrown back) as before.
    * Checked (15 fps captures, `BF_INSIDE_LOG=1 BF_NO_SQUAD=1`):
      * From 6-7 m out sprinting at a rock's middle, or 3.4-4.2 m out dodging, diving or knocked
        at it (27 runs over sdm_e34, e34, e40 and the tutorial): before #285, 7 ended in a rock;
        now none, and Tex runs on the spot against the boulder (frames 26-60 of
        `BF_MAP=sdm_e34 BF_TEST_GOTO=0.10,76.50,0.10,69.50 BF_TEST_SPRINT=1.0`). Started on a
        rock's middle (`BF_TEST_GOTO=0.1,69.5,0.1,69.5`; the tutorial's 54.0,16.6; e34's
        -59.9,34.2) he's taken out on the first frame, beside it: 1.9, 2.4 and 1.2 m out.
      * #285's test's sets (its scripts, run again): 108 drops of 4 and 9 m onto the tops and
        sides of six rocks, none taken out (5 before this round); 67 sprints at 14 rocks, 16 at
        sdm_e34's boulder from the north, 46 at pairs and clusters of rocks, 64 dives at sdm_e34's
        boulder from 8 sides and 0.7-1.45 m clear, and 16 walks and sprints over open ground: none
        ends a frame inside or is taken out, but for a run that starts in a rock (before this
        round 1, 6, 2 and 11 of the sprints, the sprints from the north, the pairs and the dives
        ended a frame inside). Its 192 dives at three rocks, run on the build before the steep
        sides: none either (12 before).
      * The loops it found: sdm_e34's boulder from the north (`BF_TEST_GOTO=0.10,65.50,0.10,77.50
        BF_TEST_SPRINT=1.0`, and from 63.0) is climbed and left over its far side, to (0.16, 77.59)
        where main's run ends at (0.06, 77.54); e40's h_05736b8b (`113.78,68.74,134.64,76.34`) ends
        at (128.7, 79.6), main's at (127.9, 79.8); sdm_e34's outcrop h_f5270370
        (`29.23,44.02,38.53,47.40`) holds Tex at its crest, where main ran him through it (its
        frame 23 is inside the rock) to the goal.
      * The test tools' ridge: [testtools.md](../testtools/README.md).
      * `cargo test --lib arena`: a 2 m twinned box holds a body out in one 5 m move and in 0.6 m
        steps, slides it along, takes one put inside out to its nearest side; its faces that face
        in are neither floors nor walls; a lone face holds from its front only; a low rock is
        stepped over; an overhang, one leaning 15° out and a ceiling at 1.2 m hold; a 55° twinned
        slope doesn't, nor one easing from 60° to 53° or from 55° to 52° to 45°, walked up from its
        foot.
    * Against main, the same runs (15 fps): the tutorial's base door identical, its second
      building within 0.06 m; sdm_e34's barracks door closes a frame later, Tex within 0.36 m.
      #264's checks: the tutorial's rim and pit wall identical; e40's s3 takes off and lands
      (20.9 m) as before, Tex's slide after it starting 0.13 s later and ending 0.04 m off, the
      Shrikes' slides within a frame; m09_a's squad doesn't fall (Flint steps down a 1.2 m ledge
      on the way), its Shrike Hound's slide starts a frame sooner and doesn't start again at 6.8
      s. e40's s2 steps onto the 66° face where it did but lands after 17.2 m, not 19.4 m: on the
      rim the floor is now the twin with the slide flag (above), so the slide starts a frame
      sooner and he leaves the face slower (#264's 30 and 60 fps runs landed after 17.6 and
      17.8 m). Sprinting slantwise at sdm_e34's barracks he's held at its wall (-75.1, -9.7),
      where main took him through it. The squad following Tex on e34, and past the boulders of
      sdm_e34 and e40 (`BF_TEST_GOTO=6,60,6,90`, `-5,-150,-5,-180`): within 1.22, 0.04 and
      0.43 m of main's (on e34 Tex ends this run sliding: slides start sooner, above). #285's
      test's 16 walks and sprints over open ground on e34, sdm_e34, e40, the tutorial and m09_a
      end where main's do (within 0.25 m) but where a slide on slide-flagged ground starts
      sooner: e34's sprint 2.4 m off, sdm_e34's flat run and sprint 1.5 and 3.4 m.
