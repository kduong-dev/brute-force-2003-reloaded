*Part of [brain/](README.md), the former page `brain.md`.*

## Sight and contacts

The brain's sensor (FUN_0004d140 with the fov test 0x3d300; the scan FUN_00044140; the contact
clock FUN_00054950; the target pick FUN_0004a780), each frame:

* **Seen**: a living hostile within its sight distance x 1.15 once it's a contact
  (FUN_000481c0) x the hostile's visibility (a stealthed Hawk's near 0); within the placement's
  `field-of-view` (h_1b2327a0) either way of the way it looks (its aim while it fires, else its
  facing); with the world's sight ray between the two characters clear ("The sight ray" below,
  #244). The sight distance is the
  type's `vision-range` (the Red Hand 75 m, the marines 80), or the placement's own `vision` when
  its h_11cddd9f says so (the sniper lesson's target: 10 m). The field of view: 60 degrees the Red
  Hand, 90 the marines. The gun's range plays no part in sight. (The game's field-of-view test,
  0x3d300, looks from the head node's place, char+0x3e8, along its matrix's Z column at the
  target's root node; the character's position and facing serve only one without a head node. Its
  reach test, FUN_0004d140, measures the owner's position record to the target's. The demo takes
  its own eye (1.6 m over the feet) and the target's hips, the root node's place the game looks at
  (since #244; it used a chest 1.1 m up), and looks along its aim or its facing: only the ray's
  ends are the game's.)
* **Hostile** means the team table's (0x3ffd80, `damage::hostile(theirs, mine)`: the candidate's row,
  my column, as the scan FUN_00044140 asks the candidate's sensor +0xc0 with my team; the table is
  symmetric) (#175). Not "another team": the squad
  (0) takes teams 1 (the Shrike), 7 and 8 and not a neutral (2), which is hostile to nobody but
  team 7; teams 3-6 are hostile to each other and not to their own, and 7 (PC_NO_TEAM) to itself, so two
  of team 7 see each other. e34's
  Shrike Hounds are neutral (team 2): the squad and they ignore each other. The same test holds for
  a shooter it's told of (`attacked`, EVT_SHOT_AT), the contacts kept (`Sensor::update`) and the aim
  helper's pick (BF_TEST_AIM_HOSTILE). The test map's NPCs use the same scan, so a neutral set to
  fight doesn't fire at the squad, nor the squad at it.
* **Contacts**: a new sighting becomes one; one unseen 15 s is forgotten, a dead or vanished one
  at once, a stealthed one (visibility 0.2 or under) at once, after the step's sightings as the
  scan's tail does (0x443f1): so a stealthed Hawk never keeps the 1.15 (#197).
* **The target**: the contact it fires at, kept while it's been seen in the last 3.75 s
  (EVT_TGT_LOST_SIGHT, FUN_0002e330; 10 s, FUN_0004a780) and its muzzle line hasn't been blocked
  that long (EVT_SHOT_BLOCKAGE, "The muzzle corridor" below), then the nearest seen, one whose
  line isn't blocked first.

**The sight ray** (#244, #340; the world's pair table, FUN_00057a30 -> FUN_00057730 ->
FUN_0004cfd0; `play_brain.rs` `ray_point` and `line_of_sight`). The world casts it once for a pair
of characters, and a brain reads the result as bit 0 of the pair's entry (FUN_00043000; the radar's
FUN_000572a0 reads the same bit):

* **Its end points** are each character's FUN_0004cea0 point: for a character, where its "Bip01
  Head" node is (char+0x3e8, looked up by name at 0x12f95d, the name being the type's bone prefix,
  "Bip01" by default, and " Head": the demo looks up "Bip01 Head" only; the node's +0x38..+0x40,
  the three floats the aim capsule 0x121d00 reads for the head) in the pose it has now; a
  character with no head node gives its root node's place (the hips, vtable +0x40). So the ray runs
  from head to head, not from an eye to a chest, and a crouch lowers its end. A standing Red
  Hand's head node is 1.76-1.83 m over its feet (the sighting line of `BF_BRAIN_LOG`), Tex's 2.01
  (his camera pivots on the same bone: 1.985 at idle, [camera.md](../camera.md)). The demo's ray used
  to run from 1.6 m to 1.1 m over the feet.
* **The ray** runs from the first character's point along the line to the second's over the whole
  distance (FUN_000df6f0 -> FUN_000df0a0; two points within 0.01 m of each other are in sight
  with no ray; the length is 0.9 of it when either object is of class 0x18, the live grenades and
  mines by the code map, which no character is).
  The two characters are on its exclusion list.
* **What it meets** is the standard line-of-sight filter's (vtable 0x39cc18, which FUN_00057730
  builds with the kind mask 0x58 and FUN_0004cfd0 adds 0x150 to, 0x158, and the object-type mask
  0x415a1018): FUN_000df0a0 leaves out the PID kinds whose bit is set, 3, 4, 6 and 8
  (PROJECTILE, BLOCKER, HINTER, FLY_BLOCKER; with 4, 6 and 8 all set it doesn't walk the blockers'
  list at all) and the object types 3, 4, 12, 17, 19, 20, 22, 24 and 30 (characters, effects, items,
  grenades, liquids ...). It meets the terrain and the placed objects' collision, and **never the
  level's blockers and hinters, nor a character** (#340): an out-of-bounds wall or the band round a
  lava pit hides nobody from the AI. The Roller's sight line is the same filter with the mask 0x58
  ([grenades.md](../grenades/roller.md)); the demo's AI ray is the same function (`roller::clear`,
  `Arena::ray_shot`'s filter). take08's radar agrees: it shows the lava yard's terrorists, across
  the pit's hinter band, for seconds at a time while Tex stands in reach of them ("Verified", the
  lava yard).
* **Not modelled**: the pair table casts the ray only for the pairs due for it, five at most an
  update, each again after FUN_00057890's interval (1.5 s while neither of the two is alerted
  (brain+0x770), 0.53 s with one, 0.083 s when either has the other as its target, 5 s when one is dead
  or has no brain, 4 s when the flag in a brain's +0x794 object (+0xd) is set; all but the 4 s x 1
  to 1.7 by a load figure of the world's (+0xc58), and a random fraction of n x 0.0867 s added for
  the nth pair of an update), and only while one of the two has the other in reach; the bit keeps
  its value between. The demo casts it every frame for every
  candidate in reach and in its field of view, so a sighting can be up to a second or two sooner
  than the game's (the pair's first test too). The game also casts a pair's ray at once when a
  contact is added by "ADD OBJ TO CONTACTS" (FUN_0004a860's tail) and in GOAL_ATTACK_COVER's update.

**The muzzle corridor** (#244; the pair table's second bit; `play_brain.rs` `muzzle_line`,
`corridor`, `Sensor::lines`, `fire_gate`). Seeing a target isn't enough to shoot it: the attack
check (FUN_00024f80) refuses, in this order (the reason is written to brain+0x3b8): 1 no target,
2 no entry for it, 0 the skill's hold (Hawk's stealth), [the gun raised or kept raised,
FUN_00123ea0 at 0x2509c], **4 the entry's bit 4 clear (0x250a1)**, 5 no allow_combat, 6 the burst
pause (FUN_00024950), 7 no weapon, 9 not seen with a clear ray, 10 the lob hold, 11 the gun's
aim more than the goal's angle off. Reason 4 is a second bit of the pair's entry, beside the
ray's, that the demo didn't have:

* **The bit** is the pair table's muzzle bit (entry+0x14 bit 1 or 2: FUN_00057570 sets both
  after a clear ray, FUN_00057360 each; with the ray blocked they keep their old values).
  FUN_00043000 copies "my muzzle to this contact" into the contact entry's +0x80 bit 4 while the
  contact is in reach (clear out of it). FUN_00057360 (shooter, target): the shooter's ready
  timer (char+0x7dc) **over 4.0 is clear without a test** (the raise is under way: FUN_00123f60
  starts it at 4 + the type's in-time); no gun in hand is blocked; else **the corridor from the
  gun's muzzle 0** (FUN_00232180) **to the target's head node** (its +0x3e8's place, the sight
  ray's own end), and when that is blocked **to its hips** (vtable +0x40), clear when either is.
* **The corridor** is a plain world ray over the whole distance (FUN_0004cd10) with FUN_00056d20's
  filter: the sight ray's kinds 0x58 but the object-type mask **0x415a1010**, the sight ray's
  0x415a1018 with the characters' bit clear. So it meets the terrain, the placed objects'
  collision and **a third character** (anyone but the shooter and the target), and still not the
  blockers, hinters or projectiles. The demo's third character is the cylinder its shots hit
  (`ray_cylinder`, 0.4 m across the radius, 1.9 m high; a body lying down isn't met).
* **In the demo** each AI, every frame, tests the corridor to each contact it sees (the ray clear,
  in reach and its field of view) and keeps the pair's last value for the others
  (`Contact::pair`); the entry's bit is the pair's while the contact is in reach (`line`).
  `fire_gate` (play.rs `update_player`, after the skill's hold and before the command mode's
  allow_combat) leaves a blocked bit nothing to fire at: the AI turns to the target with the gun up
  (`hold_up`: raised, or kept raised, before the bit is read, so no raise is needed when the corridor
  clears; the first round still waits `ai_fire`'s 0.2-0.8 s delay, as for any new target) and
  holds fire. The squad's broadcast and the sighting log keep the target
  unfiltered, as the target pick (FUN_0004a860) knows nothing of the bit.
* **What it changes in practice**: with the gun up the muzzle is at about head height, so the
  corridor to the head is the sight ray's twin and is rarely cut once the target is seen. Reason 4
  holds fire mainly for **a third character in the line** (a friend, a dummy, the controlled
  character), for a gun not yet up (a patrol's walk carries it low, 1.06 m on the first patrol's
  leader at its sighting: both corridors blocked until the raise starts), and for a frame now and
  then. It does not stop the rounds that strike the terrain on the way to a target's hips with
  its head in view (the first patrol at take07's spot, under "Verified").
* **EVT_SHOT_BLOCKAGE** (FUN_0002e330, called from the target pick FUN_0004a780 in the goals
  ATTACK 7, VARY 8, GUARD 14, MOVE_STRAFE 51 and DEBUG1 120 only): the entry's +0x38, how long the
  bit has been clear in a row (FUN_00054950), over T = 3.75 s, with the ray blocked no longer than
  T (else it is EVT_TGT_LOST_SIGHT, 7) and the target not raising its own gun (FUN_00054920: its
  timer 4.0 or under), posts EVT_SHOT_BLOCKAGE (0x21), and the target pick gives the target up.
  `default`'s draw for it (the first patrol's h_e176c6ce and the lava yard's h_eed3a76a inherit
  it) is REPOSITION_LOS_TO_TGT (110, weight 1), GOAL_PURSUE (weight 0). The demo gives the target
  up as a lost one, in those goals only (`sense`, `glance`: another seen contact whose line isn't
  blocked that long is taken instead; the squad's AI, which has no attack goal, and a brain in
  ATTACK_COVER keep theirs) and, the reposition being #26's, tries GOAL_PURSUE once for the spell
  (`fight`): which ends at once while a contact is in sight (0x2c886), so nothing walks, and the
  AI holds fire facing the target until the corridor clears. In the game a brain in DEBUG1's cover
  half takes the post to its cover goal's hook, which keeps it; the demo posts nothing there (`fight`
  doesn't run in the cover half), so only `sense`'s target drop applies. **T is 1.5 s when the
  *target's* brain is in a dodge, stun, knock-down or cover-attack goal** (FUN_0002e2e0 reads the
  contact's brain, 0x2e35d-0x2e365, not the shooter's), for both of the function's clocks, the
  ray's (EVT_TGT_LOST_SIGHT) and the muzzle line's: the demo applies it where the target's goal is
  known, a level character's brain in ATTACK_COVER or DEBUG1's cover half, stunned, knocked down
  or dodging (`brain::quick`, `Seen::quick`, `Lead::quick`; the squad's and a player's
  characters are 3.75 s: their dodges and falls aren't brain goals here).
* **Logs**: `BF_BRAIN_LOG` (and `BF_SIGHT_LOG` for the squad and the NPCs) print `muzzle corridor
  to <who>: head clear, hips not tested` / `head blocked, hips blocked: the attack check refuses
  it, reason 4` / `the gun is still rising: taken as clear` each time the result for a contact
  changes (the hips' corridor is tested with the log on even when the head's is clear, to show
  both); the brain log also prints the `EVT_SHOT_BLOCKAGE` post, and its `sees ... a contact`
  line the ends of the sight ray, the target's hips and the gun's muzzle in metres over the feet
  ("its sight ray runs from 1.82 m to 2.01 m over their feet, their hips 1.19 m, its muzzle
  1.06 m", the first patrol's leader at its sighting, its gun still down: the corridor starts at
  the muzzle and ends at the head or the hips).
* **Not modelled**: the pair table renews the bits every 0.083 s x 1 to 1.7 with a target (0.53 s
  alerted, 1.5 s neither, FUN_00057890), so the game's bit trails the geometry by about 0.1 to
  0.15 s with a target, and the raise shows through it (clear while the gun rises, renewed at the
  next retest after). The demo reads it every frame.

The sensor is `play_brain.rs`'s `Sensor` (`Player::sensor`: its sight distance, field of view
and contacts), and the squad's AI and the test map's NPCs, which have no brain, see by it too
(#197, `glance`): the same reach, 1.15, field of view (the placement's, else the type's
`half-fov`: 90 degrees for the squad) and ray; their contacts are forgotten as the brain's
(unseen 15 s, gone, or stealthed to 0.2), and they fire at the nearest contact they see within
the gun's reach. One that hits them becomes a contact at any range when it's over 0.1 visible
(EVT_DAMAGED's registration, 0x3ea02; a near miss tells them nothing), and with nothing to fire
at they turn to the last place of the nearest contact seen in the last 3.75 s (the target's
EVT_TGT_LOST_SIGHT time), gun up, as a brain turns to its target's: so a squadmate shot from
behind, outside his 90 degrees, turns and fires back, and one whose enemy steps behind cover
keeps facing where it went.

The squad tells each other of its targets (FUN_00045090 after every sense step, FUN_00045250:
EVT_ENEMY_BROADCAST, 39): a member holding a target (a brainless one's, a following brain's,
the controlled character's nearest hostile in sight) tells every other living member at once
and then every 2 s while it holds it (a look every 1 s without one). The wingmen's personality
maps it to GOAL_WINGMAN_PURSUE and the quick goal "ADD OBJ TO CONTACTS": the demo makes it a
contact of each (its last place renewed), so they turn to it and fire once they see it
(`broadcast`; the pursuit is #26's). A wingman that knows no enemy and is hit gets
EVT_DAMAGED_NO_ENEMIES, GOAL_INVESTIGATE_GUARD in its personality, which registers no one: the
demo's registering and turning to the shooter stands in for that goal. With `sqbehind`'s
staging (a Shrike behind Brutus's left on the test map, out of the squad's 90 degrees) Brutus is
hit at 6.27 s and turns, Flint and Hawk are told at 6.40 and see it at 6.53 and 6.93: 24, 8 and
23 shots (main, which saw all round: 29, 9, 24).

(`BF_SIGHT_LOG=1` still logs a brain finding or losing its target, as the squad's; for the squad
and the NPCs it also logs each contact found and forgotten.)
