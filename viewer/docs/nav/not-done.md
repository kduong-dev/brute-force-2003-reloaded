*Part of [nav/](README.md), the former page `nav.md`.*

## Not done

* The script 105 walks (#337, an open code question). Of #336: the goals the demo's brain has no
  draw for (INVESTIGATE, GOTO, HIDE, SNIPE, SCOUT; GOAL_IDLE's route to an event's point, as no
  event posts one), ATTACK_VARY's state 2 (strafes, dodges, #26), the
  approach 3 to an event's point (+0x88 bit 13), the
  pursuer's and the approach's chatter (#36), the patrol's 4 s timers. The follow's route-point
  states (6-16: WINGMAN's first entry walking a route to a point ahead of its leader on his path,
  FUN_0001adf0, FUN_0005acb0; [squad.md](../squad/the-follow.md) "The follow").
  The per-type capsule (#173: the squad's lf 0.35 m against the demo's 0.4). The prisms
  (FUN_0004cb60's records) and brain+0x3c8.
* Follow-ups from #336's runs: e40's patroller h_f490d291 turns back at its loop's fourth point
  every round (its 15 m home area against a loop reaching 19.5 m from home: the code's rule as
  read, not seen in footage). TUTORIAL_terrorist_6 pauses 3.4 s at two corners of its 66.5 m route
  home (the stuck handling's 2 s, then its detour). e23's ledge Ugly (h_e8008de4, home on a ledge at
  y 24.8) steps off the ledge on PATH_TOWARD_HOME's one-leg route home (the corridor rays don't
  see a drop) and stays stuck below until the stuck handling puts it on the route's point.
* The motion's open ends (#363): the game's members fight from attack goals (#33 take10: goal 120
  DEBUG1 from when each sees the enemy), which run and walk; the demo's keep the follow's dash,
  and a member dashing with a target holds fire (the game's dash can't aim, its MOTION's
  aim="false"; the demo's sprint plays the ready run with the gun raised, as the player's does
  after a shot). The facing override (brain+0x488, the follow's look-point near a cover point,
  FUN_00022790). The body's turn (the game's 400 degrees a second x k; the demo's 9 rad/s). The
  aim-points block (0x11d271-0x11d45e) behind the game's upper-body aim on the move. The shot
  list's live reads (`todo/363-squad-combat-freeze/shotlist.md` A-E: the gait against a member's own
  view in a fight, the push's sway, a corner's side step, a reversal's backpedal at 0.64 x) aren't
  filmed.
* The avoidance's open ends (#344): the side-step's two world lists ([world+0xa4], [world+0x12c]:
  a live read at 0x529d0 / 0x527f7 would say what they hold; the stage has nothing to step round
  until then); FUN_00019ed0's other branch, for a brain with FOLLOW_HOLD_BACK's route at
  brain+0x684 (the move straight along the facing; the demo has no such goal); FUN_00019af0's
  bit 14 of brain+0x3c4 (its setter not traced); FUN_00052120's height extents; brain+0x3c4
  bit 1 (skips the push: cleared at 0x1b15c when a brain timer runs out, its setter not found);
  FUN_00018c40's other callers (0x2a61c, 0x2a66e with a forced 3.0 m; 0x2addb, 0x2aec5). The walk's
  speed (the clip's 1.6-1.9 m/s where the game's follows the stick, #350) makes the feelers' and
  the hard turn's walks slower than the game's. The level's prisms (FUN_0004cb60's records, the
  level record's +0x114), inside which the game's AI runs instead of dashing and drops to a walk at
  a feeler's hit: without them the demo's members dash through buildings, and with the move's turn
  at 200 degrees a second a member dashing at a doorway at an angle can miss it and go round
  (#344's fix round, sdm_e34's door pair h_f519ca12 / h_117d5fb1 with three members close behind
  the leader's dash: Hawk round the building, 20 s where she went through in 2 with the free turn,
  `BF_AVOID_FREE_TURN=1`). The jump steps (FUN_00026520 / FUN_000267b0: bit 12, the hinters and the
  feelers off between a jump's points), so the demo's routes still lead to a drop they can't take
  (the stuck handling puts the member down, `progress`). Footage: todo/344's shot list (the
  feelers' hits, the pace level, a dash past a squadmate) isn't filmed.
* FUN_00059f70's and FUN_00059d40's temporary points; the 20 m look at a route's end (route+0x14,
  FUN_0001fdd0); the kinds' own steps at a point reached (patrol points, jumps; the doors' since
  #346, but its world-button branch, dead in the data); the prisms (bit 10). Of the pace near a
  route's points (FUN_0004d840, done for Move To since #344's second fix round, `pace_near`): its
  first branch, by a position of the brain's own at brain+0x1b4 (its setter not found), and
  FUN_00058590's +0x24 (set to -1); MOVE_OUT's run (mode 3) while it faces a target of its aim
  record (FUN_00045fa0's first block). The follow's walk doesn't call it (state 0 passes
  FUN_0004d970 no fifth argument, 0x22377), so it slows no follower at a corner or a door.
* Footage of a squad member at a door (todo/346's shot list A and B: the stand, the door's state
  when it's reached, the stuck steps at a door that stays shut) isn't filmed.
* Kind 0x13's and the save points' set-valid (kept by the game elsewhere); the cover points' bit 5
  is kept (`Node::cover`), which the cover pick reads (#384).
* Footage: take01 (e13, the squad round the pit, its routes logged) is the one take; Move To across a pit, the tutorial's lava yard and the blocker and stuck shots (shotlist.md A1, A3-A5, B, C) aren't filmed.
