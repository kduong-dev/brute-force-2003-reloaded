*Part of [nav/](README.md), the former page `nav.md`.*

## The steering: the AI's avoidance (#344)

`src/bin/play_avoid.rs` (`steer`), for the route walkers: the follow (play.rs `squad_ai`: the
squad's, the tutorial's marines', the test map's straight walk) and Move To (play_orders.rs
`move_out`). The reading is `todo/344-ai-avoidance/leads.md` §1-§6, checked against the asm; every
function is in [code-map.md](../code-map/ai.md), "AI". Since #336 the enemies' goals walk routes and
are steered by it too (play_goals.rs `go`, "The enemies on routes").

**FUN_000255d0** makes the direction of every walking goal from its steering target (the route's,
brain+0x478; none for a target at the world's origin) in four stages and takes the first that
steers: the target's direction only when nothing is in the way. A stage that steers is the whole
direction (no blend): the stages write into the AI update's direction buffer, which is zero when
they run (zeroed before the goal machine, 0x1b338, and no walking goal writes the target's
direction into it first: the route walker FUN_00026d80 writes the target's position), so the
push, which adds into it (0x523fd), gives its sum alone (#363's reading; #344 read the stage as
overwriting, the same in effect).

1. **The feelers** (FUN_00018c40, `feel`): three rays along the character's heading (iface +0x34:
   its commanded move, the stick the last AI frame gave it, else its body's forward; #363, which
   put right #344's reading of the speed it makes) from its hips 0.25 m lower and
   0.5 m back: the centre 3.15 m, the left and right 2.0475 m at 25 degrees (the brain's list,
   brain+0x354, which the constructor fills and nothing changes). Their lengths: x0.6 at a walk,
   x1.6 at a dash, and x0.9 x the speed factor char+0x720 when that's over 1 (the follow's boost:
   x1.125); off at a stand, and while brain+0x924 runs (the walk's door step sets it, "The door
   step"). Each ray is tilted by the
   ground ahead (FUN_00115b00: the height change to the ground 0.8 m ahead along the move, its y
   added before it's normalised). They meet the terrain, the placed objects, the blockers, the
   hinters and the fly-blockers, never a character, an item or a liquid (the corridor ray's
   filter; `Arena::ray_kind_normal`); a terrain hit whose normal's y is over 0.7 is walkable ground,
   not a block; an object's or a blocker's always is. The response:
   * at a dash (or a run inside a prism) the first block only drops the pace: the level a notch
     down with its timer at 1.5 s, or, with the speed factor over 1, the speed record's timer
     (brain+0x8a4) at 1.5 s, which takes the follow's boost off (FUN_00048270); nothing steers
     that frame. So a boosted dash meets a wall twice: the boost off, then the run;
   * a side ray's block turns it square to its heading, away from it (a left hit: the heading's
     right); both sides blocked, the side whose block is farther;
   * the centre's alone (a wall ahead): two more rays along the side rays, four times as long, and
     the side with more room (a missed long ray keeps its slot's last distance, as FUN_000ddd10
     writes it only on a hit: copied as read); then, at a run or a dash, held to the side the
     target lies on (FUN_00018b50), with the level 2 for 2 s when that overrides the pick.
   A side ray reaches 0.87 m across at a run, 1.56 m at a boosted dash, 0.52 m at a walk: what
   keeps a member about a metre off e13's hinter band.
2. **The push** (the sensor's +0x3c, 0x521c0, `push`): every other living character (of any
   side: the world's two character lists, no team test) within 1.5 m across, overlapping in
   height (1.0 m margin) and in the half in front of the heading (the commanded move, as the
   feelers'), or within 0.5 m anywhere, pushes the direction away from it by 1 over the distance;
   the sum (not normalised there: FUN_0004d970 does it). The stage has no memory: what keeps a push
   of a frame or two from showing is the move's turn after it (below), which starts from the last
   command and turns at most 200 degrees a second.
3. **The side-step** (the sensor's +0x4c, 0x52750, `side_step`): the nearest obstacle disc in the
   path (its offset across the heading under the two radii, the walker's 0.5 m and its own), within
   its radius + 2 m and in front: a step across, away from it. Its discs come from two world lists,
   [world+0xa4] and [world+0x12c], neither identified (the radius method, 0x52c90, takes the body's
   bounding sphere for OT_STATIC_GEOMETRY, OT_PLATFORM and OT_WORLDBUTTON, and 0.5 m for a
   character, the walker's own disc); the demo has none to give it (`obstacles`).
4. **The plain direction** (the sensor's +0x24, 0x519f0, `plain`): to the target, flat.

**The move** (FUN_00019af0, `move_turn`, #344's fix round). The direction goes to FUN_0004d970
(play_routes.rs `pace`: the move at the lower of the mode asked and the pace level, the direction
sized x0.01 at a stand, x0.5 at a walk, x1 at a run or a dash) and the AI update's steering
FUN_00019ed0, which turns the character's view toward it (below) and, for the follow and Move To
(no FOLLOW_HOLD_BACK route at brain+0x684), hands the move to FUN_00019af0: the move turns
from the heading toward the direction **at most 200 degrees a second** (3.49 rad/s), its rate eased
from the last frame's, a third of the way a game frame at a run or a dash and a fifth at a walk,
dropped to 0 when the turn reverses, never past what closes the angle; within a degree it keeps the
heading, and after a frame that ended with no direction (a stand) it takes the direction at once.
The move is the character's stick (char+0x5e8, `Avoid::moved`), and the heading it turns from
next frame is that move (#363): the chain is command to command, the speed the character makes
never feeds back. The demo moves a character along its facing by its clip's root and turns the
facing at 9 rad/s, so its facing follows that move; the ease is applied by the frame's share of
the game's 30 Hz frames.
So a feeler's square steer, a frame or two at a time, bends the path by a few degrees where before
it turned the member a third of the way round at 15 fps: the weave of the first round's e13 runs
(Hawk and Brutus 1.18 and 1.13 changes of side a 10 m, take01's 0.36 and 0.42) is gone. A turn at
200 degrees a second has a radius of 1.8 m at a run (6.3 m/s) and 2.4 m at a dash (8.5 m/s), so a
member dashing at a doorway at an angle can miss it, or loop round a corner point it has to close
on. The game's facing turns no faster for the squad (FUN_00019ed0: the type's turn rate x
char+0x7ec, which the brain's constructor sets to 200 / (type+0x2d0 x 57.3): 200 degrees a second
too). What slows its members there: on a Move To the pace near the route's points (FUN_0004d840,
`pace_near`: a walk within 3.2 m of a sharp corner); indoors the level's prisms (FUN_0004d970: a
dash is a run inside one; FUN_00018c40: a feeler's hit there drops a run to a walk), which the
demo doesn't read: see "Not done". The follow's walk has neither in the demo, and in the game only
the prisms (its state 0 passes FUN_0004d970 no fifth argument, 0x22377). A turn past 180 degrees
at the cap (a loop to close on a corner point) lasts over a second at more than 2.5 rad/s, so it
ends in the hard turn's walk; a plain U-turn at the cap, 0.9 s, doesn't.

**The view and the motion** (#363; play_avoid.rs `view_turn`, `pick`; play.rs `step_player`,
`ai_gait`; the reading is `todo/363-squad-combat-freeze/leads.md` §1-§3). A character's motion
comes from its stick in its own view frame (R, char+0x5bc), and the AI turns that view itself:
* **The view** (FUN_00019ed0, `view_turn`): toward the steering direction at **200 degrees a
  second** (the type's turn rate x char+0x7ec, which the brain's set-up makes 200 / (the rate x
  57.3) for a squad type), the fraction of that eased as the move's (a third of the way a game
  frame at a run, a fifth at a walk), a turn the frame can finish closed at 2/3 of what's left
  (its kept fraction first halved, 0x1a086), none within a degree. Its one override (brain+0x488)
  is the follow's look-point near a cover point (FUN_00022790, not done); no fight goal sets it,
  so in a fight the view turns to the move, never to the target. FUN_0011e5e0 hands the turn to
  the character and FUN_00119fa0 applies it, and while the character stands its body follows its
  view.
* **The motion** (FUN_001195c0, `pick`): forward when the move is within about 66 degrees of the
  view (a cosine of 0.4; 0.25, about 75 degrees, with the gun up: the ready timer char+0x7dc over
  0), backward past 114 (105); in between, by the body's half the move is in. Forward, the body
  turns to the move and it runs (CT_RUN: the run, the walk or the dash by its pace) while the
  body is within 45 degrees of the view or still 30 or more off the move, and steps sideways
  (CT_MOVE_LEFT / RIGHT, toward the side of the view the body is on) only with the body on its
  move and the view more than 45 off it. Backward, the body turns to the move's reverse and it
  backs off (CT_MOVE_BACK) with the body within 45 of the view, sideways (CT_MOVE_*_BACK) past
  that once the body is on the reverse, and gives no command while it turns. So on a straight
  the three agree and it runs facing its move; a side step plays for the frames of a sharp turn
  its view lags (from a stand, say, the move taken at once and the view turning after it), a
  backpedal on a reversal until the view has swung to within 66 degrees of the move. The gun
  finds a target with the upper body as it runs (play.rs's aim, the arm chain turned up to 1.9
  rad). The side and back motions at a run or a dash are running states (FUN_00126a40: MT 26 /
  30, 27 / 31, MT_BACKPEDAL) and play the sets' `f_side_run`, `b_side_run` and `back_run` clips at
  the run's speed, x0.8 more while backing off (FUN_0012cb40, char+0x800 = 0: 0.64 x the type's
  run without the follow's boost, play_speed.rs); at a walk the side and back walks keep their
  clip's root, as every walk does.
* **What it replaced.** Until #363 a demo member took the controlled character's camera for its
  own and moved by the player's aimed strafe against it whenever it aimed (a side walk for a move
  30-150 degrees off his look, a walk back past that, a walk for a run: the clips' 0.4-2 m/s), its
  heading read from the speed it made: a member with a target and its way more than 30 degrees
  off the leader's look crawled at 0.3-0.7 m/s, its gait flipping about 12 times a second and its
  clip layers stacking 10-11 deep, and the push's frames swayed a running group. Its move was also
  put in the leader's camera's frame before the fire turned the camera onto its target, so it
  moved off its way by the angle between the two. The game's AI reads no other character's view.

**The pace record** ([brain+0x1a8], `Router::cap`, play_routes.rs `pace_ramp`): FUN_00058530
starts it at the level 1 (a stand) with its timer at 0.3 s; FUN_00058670, each AI frame (0x1b119,
before the route steps), runs the timer and, once it has run out with the level not the mode
asked, steps the level one notch toward it, up (FUN_000589b0 to 2, FUN_00058a60 to 3, FUN_00058aa0
to 4) or down (FUN_00058b10 to 3, FUN_00058aa0 to 2, FUN_00058a20 to 1, that one with 0.3 s), and
starts the timer again at the step: 0.17 s for a squad brain in FOLLOW or WINGMAN, 1.0 s for any
other. A follower stopping steps down to a stand and climbs again when it sets off: a walk at
once, a run 0.17 s later, the dash 0.34 s after it; on a Move To, the run after 1 s and the dash
after 2. Then **the hard turn** (FUN_0001ad50, 0x1b16b, `hard_turn`): a route walker at a run or a
dash turning faster than 2.5 rad/s (char+0x488; the demo's: its facing's turn over the last frame)
for over 1 s walks for 1 s. The ramp runs for every AI character the routes' goals move, with a
level's points or not.

**The speed record's timer** (brain+0x8a4, `Avoid::boost_off`): counted down each AI frame
(FUN_00048270, after the goal; the demo's a frame sooner); while it runs the follow's boost isn't
handed to char+0x720 (play_speed.rs `record_boost`). The feelers read char+0x720 as the last AI
frame left it (`Avoid::factor`).

**The demo's choices**, labelled in the code: the push's height test (the two bodies' spans, 1.9
m each, grown by the margin: FUN_00052120's extents aren't read); the push's list, the characters
within 2 m at the frame's start (`routes::near`); no side-step obstacles; the hard turn's rate from
the facing's turn; the move's and the view's ease by the frame's share of the game's frames. Since
#363: a walk from a stand starts its view from the way it faces (the game's standing body follows
its view; the demo's stands face their own way); the body turns at the demo's 9 rad/s, not the
game's 400 degrees a second x k (FUN_001195c0's request, the step at 0x11d7fb), as the demo moves a
character along its body: so the body keeps up with the move and the path is the command's; a
character the player has had starts its steering afresh when the AI takes it back (the game's
would find the player's last stick in char+0x5e8); the walks' speeds are the clips' (#350).

**Test hooks**: `BF_AVOID_LOG=1` prints each frame a stage before the plain direction steers:
which feelers met what (steep terrain, an object, a blocker) and how far, the response (the
direction, the long rays' slots, the hysteresis' walk), the pace drops and the boost's, the push
and from whom. `BF_AVOID_OFF=1` (a diagnostic, not the game) steers by the plain direction alone,
as before #344. `BF_AVOID_FREE_TURN=1` (a diagnostic, not the game) leaves the move's turn to the
facing's 9 rad/s, as before #344's fix round. `BF_ROUTE_LOG=1` also says each hard turn's walk,
`=2` each step of the pace level. `BF_GAIT_LOG=1` (#363) prints every frame each AI member's
motion after its step: its gait and action, whether it aims and fires, its view, body and
commanded move (degrees, the demo's yaw; `-` for none), `back` in the backward case, the speed its
step made, where it is and its clip layers (clip:weight): to count gait changes and layers.
