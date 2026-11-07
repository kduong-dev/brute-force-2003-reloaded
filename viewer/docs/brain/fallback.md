*Part of [brain/](README.md), the former page `brain.md`.*

## The fallback: a target too close (#383)

`src/bin/play_fallback.rs` (ATTACK_FALLBACK and MOVE_BACK; the link walk is `Graph::away`, [nav.md](../nav/link-walk.md)
"The link walk"). Read from default.xbe (the asm of 0x44b57, 0x31610, 0x316d0, 0x29920, FUN_00029a00,
FUN_0004bbd0) and checked against todo/132-mutants take07 (F2b); the rows are in
[code-map.md](../code-map/README.md). Wired to the cover goals ("Cover" above, `play_cover.rs`): the cover's hook
keeps the event while the cover holds, the fallback's start ends the cover, and the draw's second goal
starts it.

* **The post** (the sense step, FUN_000447b0 at 0x44b57): once the brain's re-arm timer (+0x118) has
  run out, a contact that is a character and is seen now whose position record lies **strictly
  inside FUN_0002e210's radius** (3D, hip to hip: the larger of the type's `<ai>` h_f1dbdb99, 3 m
  for the Red Hand and every mutant, 4 or 5 elsewhere, and 1.7 x the held gun's Damage radius) posts
  EVT_TGT_TOO_CLOSE (85) and sets the timer to **1.5 s**, queued or not; one timer a brain, whoever
  is near. It posts **unless an 85 is queued already** (the walk over brain+0x40c..+0x410 at
  0x44c8b-0x44cbd), and **one stays queued while its ATTACK_FALLBACK walks** (`fallback::State::queued`):
  the goal machine takes the front event once and then only refreshes the running goal's copy of it
  (FUN_0003abe0, 0x3b3bc-0x3b437), nothing pops it when the goal starts and the 85 entry's decay is 0.
  So an enemy that is pursued posts once, walks its 7 m and is not started again every 1.5 s. take07's
  breakpoint on the post (0x44cc0) logged 9.60 and 11.19 game seconds (1.59 apart) and nothing until
  22.25 (Flint), although from 12.69 to 13.1 s the Ugly stood 2.19 to 3.05 m from Tex with the timer
  out; that the walk holds the queue is inferred from this and the code, not seen in a dump. The demo
  counts the timer down inside `run`, so it waits while PATH_TOWARD_HOME or the stuck ATTACK_GUARD
  have the character (the game's runs every frame; it only delays the next post).
  **A post goes to the brain's message handler, which asks the running goal's hook first**
  (0x3dce0 -> FUN_0003d890): while the cover holds (ATTACK_COVER, or DEBUG1's cover half) its hook
  FUN_000313e0 keeps EVT_TGT_TOO_CLOSE (`cover::takes`, "Cover" above), so nothing is queued or drawn
  and the next test, 1.5 s on, goes through again. take07's first post (10.11 s, the Ugly crouched at
  its point, Tex 1.8 m off) drew nothing; the cover failed meanwhile on its 8 m rule (the target seen
  within 8 m), and the post 1.63 s on drew ATTACK_FALLBACK, 0.12 s later.
* **The draw**: the personality's entry for 85 (`default`'s and so the Red Hand's, the mutants',
  the hounds'; h_0bb586cf's guards too): priority 20.17, **ATTACK_FALLBACK 1 : ATTACK_COVER 0**, over the
  fight's 18, under PATH_TOWARD_HOME's 20.19 (that goal comes first in `think`). One that refuses
  (the start finds no way) is struck out and the draw repeated, which comes to ATTACK_COVER (weight
  0, so only then): its start is the cover pick (`cover::start`, FUN_0002f270), and started the
  cover goal runs as the character's attack goal from there (`Goals::attack`, over when it is
  done or failed, whereupon the attack goal is drawn again); with no point to be had, or for a test
  map NPC, it is struck out too and the attack goes on. The Molotov Mutant's entry has
  priority -1 (NOOP); the Supermutant's and the Shadoon's draw their own goals first (116 the
  pound, 117), which the demo doesn't have (#212): they stand as before. An event that comes while a
  hit reaction plays waits for its end (STUN and KNOCKDOWN aren't interrupted), and a reaction
  that starts during the walk ends it.
* **The start** (0x31610, FUN_00029a00): the direction **me - the target** (the two position
  records, 3D, **not normalised**) and the link walk from the character's place (its position and
  own point) with cos 0.6 and length 7.0 over its route: the first link in each point's list that
  lies within the cone, 7 m of legs, [nav.md](../nav/link-walk.md). None: refused. The move's heading memory is
  dropped (`Avoid::dir_was`: it turns at once), the attack goal's approach and pursuit are broken
  off (they start again after) and the character says fe56cd97 ("the fallback", the mutants' chatter:
  not said, the chatter is #36's). On a map without world points (the test map's NPCs) the demo
  walks 7 m straight along the direction. **The cover goal ends with the start** (`cover::end`, 0x2f900:
  the point unclaimed, the crouch cleared, the cover's route reset, before the walk's route starts): the
  point a failed cover leaves claimed (DEBUG1 running ATTACK_VARY keeps it until its own end) is let
  go. A start that refuses leaves the cover as it was.
* **The update** (0x316d0, then 0x29920, MOVE_LATERAL's walk, the same for MOVE_BACK): **done once
  the target is over the radius + 5 m away** (8 m: 3D, strictly); else the look is the target, it fires
  (FUN_00024f80 at 25 degrees: the demo's `ai_fire` bursts, with the placement's accuracy as the
  standing fire's, the gun up) and the route is walked at the arrival radius **brain+0x4f8, 1.5 m**
  (the brain constructor's: 0x1693e; ATTACK_VARY's lateral draws set it to 0.6, not here), steered
  (FUN_000255d0) and moved at **mode 3 held to the pace level with the pace near the route's end**
  (FUN_0004d970(.., 3, 1, 1)); the route's end ends it too. The update also sets the pace record's
  +0x24 to 0.1 s each frame, so each notch of the ramp takes a tenth of a second (`Router::step_time`;
  the pace near the end clears it, as FUN_00058590 does): the first notch still waits out the timer
  the record had.
* **How it moves**: facing the target while it moves away. The motion is picked as FUN_001195c0
  picks it (`avoid::pick` against the view toward the target, as the squad's) and mapped to the
  type's motion types by the command handler's cases (0x122149-0x1221ff: a back or side step with
  the stick at 65 percent or over is the run's variant (CT_RUN's is over 65), the walk level's the
  walking one): backing off is **BACKSTEP at the walk,
  BACKPEDAL at the run, MOVE_*_BACK_SLOW / MOVE_*_BACK** sideways, a side step MOVE_LEFT / RIGHT (the
  SLOW ones at the walk). A request for a motion type the current motion has no transition to
  fails and changes nothing (FUN_0012eab0), and the mutants have no BACKPEDAL: take07's Ugly asked
  BACKSTEP at the walk level and kept it for the 3 s of its fallback while the level climbed. The
  demo's stand-in is the walking variant where the table lacks the run's. The speed is the motion's
  (the walk speed for BACKSTEP) **x0.8, FUN_0012cb40's backward case** (`Drive::back`,
  `speed::AI_BACK_FACTOR`): the Ugly's 2.3 m/s walk backs off at 1.84, the footage's 1.82-1.83; the
  Red Hand's 5.5 m/s run at 4.4. The walk fires: `Drive::fire` keeps the brain's aim and fire
  where the approach's walk drops them.
* **After it**: the personality's attack goal is drawn again (F2b's Ugly went from ATTACK_FALLBACK
  to ATTACK_GUARD_CROUCH). Outside its home area the sense step posts EVT_PAST_HOME_AREA and
  PATH_TOWARD_HOME (20.19) takes over, walks it home without firing, and a character whose target
  stands within its home area meets it again within 3 m and falls back again, by a fresh post: the
  tutorial's wall sentry (a 7 m area, Tex standing 2 m from its post) falls back 7 m east along its
  walkway, is taken home, falls back, and so on every 4 s. F2b's Ugly goes through the same goals
  (ATTACK_FALLBACK, PATH_TOWARD_HOME, THROW_GRENADE, PATH_TOWARD_HOME, ATTACK_FALLBACK, 11-21 game
  seconds), but its second fallback is no fresh post (below).
* **GOAL_MOVE_BACK** (48; start 0x29a80, the same update): the same walk 7 m **behind the character**
  (the third column of its transform, a unit vector, so the 53 degrees of the cone) with no done
  distance. ATTACK_VARY's RUN_BACK / WALK_BACK (#26 part C) start it; the demo draws none, so
  `BF_TEST_MOVE_BACK` does. It faces and fires at its target as ATTACK_VARY's state 6 does.
* **The stuck ATTACK_GUARD** (play_goals.rs `stuck_guard`, FUN_00042620) marks the goal that ran
  done, so it ends a walk away, and `think` does not run the fallback while it stands guard
  (`fallback::guarded`): the guard has no target to give it (an event would only be refused for want
  of one and burn a draw).
* **Squad types** (a test map NPC of Brutus, Flint, Hawk or Tex set to fight, a level's marine) have
  no drive in the demo (`go` moves enemy types only): they never start it and stand and fire as
  before.
* **Not here**: F2b's Ugly stood up (its crouch's transition, mot 6, 46, then BACKSTEP) before it
  backed off, 0.12 s after the post that drew the fallback and 1.2 s before its BACKSTEP; the demo
  starts the walk in the frame of the post and plays BACKSTEP 0.3 s on (the drive lets the crouch go
  at once), 0.5-0.6 s ahead of the footage along the same route.
* **Not here: the queued event outlives a preemption.** take07's Ugly goals (game seconds, `watch.csv`):
  ATTACK_FALLBACK 11.31 (post 11.19), PATH_TOWARD_HOME 15.0 (Tex 6.3 m off), THROW_GRENADE 16.35,
  PATH_TOWARD_HOME 17.45, **ATTACK_FALLBACK 18.56** (Tex 6.7 m off, the Ugly running home), ATTACK_GUARD_CROUCH
  21.14 (14.3 m off); then ATTACK_FALLBACK 22.37 (post 22.25, Flint within 3 m), PATH_TOWARD_HOME 23.59,
  **ATTACK_FALLBACK 26.05** (9.4 m off), DEBUG1 26.67; ATTACK_FALLBACK 37.62 (post 37.50, Flint),
  PATH_TOWARD_HOME 38.98, **ATTACK_FALLBACK 41.93** (9.1 m off), DEBUG1 42.42. The three in bold have no
  post (the breakpoint logged four in the whole take) and their target is outside 3 m: the 85 stayed
  queued under PATH_TOWARD_HOME and the goal machine took it again when that ended, which starts
  ATTACK_FALLBACK anew from where the Ugly is (the first of the three walked 10 m away in 2.6 s; the
  other two lasted 0.5-0.6 s, with the Ugly at home and its target, not read, over the done distance
  of 8 m or soon past it). A fallback that ended on its own (21.14, 26.67) took its event with it:
  the next posts (22.25, 37.50) were fresh. The demo drops the walk when PATH_TOWARD_HOME or a hit
  reaction takes the character and posts afresh, which it does within 3 m only; the restart from
  the queue (and the pop at the goal's end, not read) would be a small change in
  `fallback::State`: a follow-up.
