*Part of [brain/](README.md), the former page `brain.md`.*

## Not done

* #26: the attack goals' other moves (strafes, crouches out of cover, repositioning for a clear
  line), calling for help, investigating noises and bodies,
  the idle variations (pauses, cautious looks, crouches, walks), grenades.
* Of the cover (#384), not done: the help call EVT_ALARM_CLOSE_RANGE the pick posts (part F); the
  squad's side (a squad record's 10 s re-pick, the leader's EVT_ORDER_* broadcasts, FUN_0002fa90's
  turns out of cover: no level character has a squad record); the cover grenadier's throws
  (brain+0x464 bit 2, THROW_GRENADE from the walk and from state 7: part D) and the restart of
  DEBUG1 after a goal pushed on top (#386); the fire on the walk to cover and back (FUN_00024f80 at
  20 and 35 degrees: the demo's enemy drive holds fire while it moves); the arrival's slow-down
  (FUN_0002fba0); the look the pick sets toward its route (brain+0x478, the sensor's vtable +0x3c,
  brain+0x3c4 bit 14); the threat score's Molotov runner and grenadier cases; FUN_00023d00's second
  gun (the held gun's reach only); the flank test as the game's (it reads FUN_0004d140's answer
  alone, which is 1 out of reach too, and the ray only in state 2; the demo's needs the target in
  reach and the ray clear, from the target's hips and head since #244's fix round:
  `Sensor::sees_at`); the timed re-pick's gate
  for a pushed goal (+0x88 bit 13; no goal is pushed in the demo); FUN_000237b0's carrying on of the
  last seen place by the target's velocity (the sensor keeps no velocity: the peek's side takes the
  place itself); a crouch-walking type's walk kept crouched (FUN_0004d970 keeps the crouch at a walk
  for one; the demo's walk stands it up); a dodge sent while a hit reaction plays is dropped
  (logged), where the game's reaction is a goal of its own on top. Guessed or inferred: the names of
  bits h_18ed19af and h_f9f3561f (read by their use: the crouch, the crouched fire), a dead
  character's brain letting its point go (the goal's end, inferred). As read and left so: the pop-up
  (state 8) and the crouched fire at a h_f9f3561f point (9) end only when the character is flanked
  (only state 7 counts the peek timer down); DEBUG1 left in ATTACK_VARY tries cover every frame;
  with no target the peek's side is decided by the point's place (the coin's offset isn't added to
  it, 0x3057b). The draw again after a refusal uses the same number (the game draws a fresh one).
  Footage: C1-C5 (todo/26's shot list: the cover cycle's timings and peeks with the 23 s re-pick,
  the 8 m rule, the three hits, the flank with its +30 degrees, DEBUG1's half-hit-points cover)
  aren't filmed, nor answers-384's peek with the grenades off, the restart across a throw, or a live
  read of the hips.
* Of the fallback (#383): the Supermutant's pound and the Shadoon's rocks, which its entry draws
  first; the crouch-to-stand wait before the walk; the line fe56cd97; MOVE_BACK's own draw
  (ATTACK_VARY's RUN_BACK / WALK_BACK, part C); a squad type's walk (it has no drive); the 85 that
  stays queued under a preemption and starts the fallback again when it ends ("Not here" above).
* Of the goals on routes (#336): the follow-ups in [nav.md](../nav/not-done.md)'s "Not done" (e40's patroller
  called home each round of a chain that leaves its 15 m area, terrorist 6's pauses at two corners
  on its way home, e23's ledge Ugly stepping off its ledge on its way home).
* Of the sight ray and the muzzle corridor (#244): the pair table's schedule (FUN_00057890's
  intervals, five pairs an update, the bits kept between, the ray cast only while one has the
  other in reach): the demo reads the ray and the muzzle bit every frame, where the game's bit
  trails the geometry by 0.083 s x 1-1.7 with a target (0.53 s alerted, 1.5 s neither), and
  shows the raise through it for a retest; the reach and the field of view from the game's
  points (the reach from the owner's position record to the target's; the fov test, 0x3d300,
  from the head node's place, char+0x3e8, along its matrix's Z column at the target's root
  node: the demo keeps its eye and looks along its aim or facing, at the target's hips); the ray
  cast at once when a contact is added or in GOAL_ATTACK_COVER's update; EVT_SHOT_BLOCKAGE's
  goal it draws, REPOSITION_LOS_TO_TGT (#26: the demo tries GOAL_PURSUE, which ends at once
  with a contact in sight). And what the two takes show that the ray and the corridor don't
  explain: take07's first-patrol leader stands ~41 m from Tex and fires from 0.4 s after it's
  seen, where the demo's walks in to 26 m (the attack goal's mapping, #26) and, at take07's own
  spot, fires its hips-aimed rounds into the slope from there; take08's lava-yard terrorists fire
  from their homes for 25 s without a hit and then move (ATTACK_COVER's re-pick at 23 s, #384's),
  where the demo's 30 stops 26 m off on its run home and hits Tex from 6.7 s.
* The patrol's 4 s timers in FUN_0002b490 (brain+0x1e0 / +0x1e8, and a 0x1f / 0x20 call on the
  character for a type with type+0x1a8 bit 15) aren't read.
* A shot's EVT_SHOT_AT radius is the squad's 1.5 m (a guess); grenades and blasts don't tell a
  brain who threw them.
* A warp's FUN_0004c5e0 for a living character (it checks the home point again, FUN_0014f7a0,
  and goes through a world list at [0x469ab0]+0x1348) isn't done.
* The marines' follow is the squad's (`squad_ai`): they set off within 0.2 s of Tex and run
  abreast or ahead of him. In take11 they set off 1.6-2.2 s after him, trailed him, and stopped
  9.1 and 10.2 m behind him, opposite his heading. The game's wingman follow (FUN_00020b10) is a
  state machine of 17 states with its own distances (8, 10, 7, 7, 4 m); its follow distance,
  side and start delay aren't readable without tracing those states: the wingman follow's own
  ticket.
