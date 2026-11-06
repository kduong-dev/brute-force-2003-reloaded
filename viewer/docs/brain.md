# The level's characters: their brains (`src/bin/play_brain.rs`, #227)

What a character a level's spawn-trigger makes does on its own: the Red Hand's patrols, their
walk home, their sight and alert, their fight and aim, and the Confed marines following Tex.
It's the tutorial's step 8 (#28) and the first slice of #26 (the AI): cover, pursuit, strafing
and calling for help are #26's. The squad's own AI (`squad_ai`, [squad.md](squad.md)) and the
test map's NPCs ([testtools.md](testtools.md)) are unchanged; the hit reactions, death and blood
are [enemies.md](enemies.md)'s.

Everything is read from default.xbe's AI: the brain at char+0x830, its goal machine
FUN_000377b0 and the goal table it calls (FUN_00012b40 fills a 0x40-byte record per goal id with
its start, end and update, each a member function of the goal holder's vtable 0x3a47b8); every
function cited is in [code-map.md](code-map.md), "AI" and "Levels and triggers". The sight rules
are #197's answer (`todo/197-ai-sight/answer.md`). The personalities are
`common/ai/personality.xmb` (`bf/personality.rs`). Since #336 the goals walk the level's
routes ("Its goals on routes" below; `src/bin/play_goals.rs`, [nav.md](nav.md) "The enemies on
routes").

A brain also has a command mode (brain+0x58, the PCMD_ table 0x3bd210: FREEWILL, FOLLOW, GUARD,
WINGMAN, MOVE_OUT ...), which picks the personality it runs from its type's `<ai>`; the squad's
orders set it (#318: `play_brain.rs`'s `Command`, run for the squad's AI by `play_orders.rs`,
[squad.md](squad.md) **Squad commands**). The level characters below keep the personality they
spawn with; the marines' mode isn't read yet.

## The spawn: its home

A spawn-trigger's copy gets its brain as it's made (FUN_000dfd90 -> FUN_00017b60, then
FUN_002096e0 -> FUN_00018510):

* **Its home** (brain+0x1ec): the trigger's `home-point` (h_1b747c8d). Without one, the trigger's
  world point h_131de144, kept when it's within 5 m of where the copy stands (FUN_0014f7a0 tests
  25, the square); otherwise the game takes a nav point near the copy, the demo the copy's own
  spot.
* **Home first.** With a home-point the brain is sent there at once: FUN_00018510 posts
  EVT_RETURN_HOME (31), whose personality entry (pri 1.5, over EVT_BASE's 1) is GOAL_RETURN_HOME
  (its update FUN_00034690), home within the point's own h_e6564e8a when it has one over 0
  (FUN_00026d80; the wall sentry's wpoint1493: 1 m), else 2 m. Its pace is the trigger's
  `locomotion` (h_f27d26c9, +0x84: FUN_00018510's 5th argument, stored as the goal's +0x94 at
  0x18750), the speed mode FUN_0004d970 takes while the goal is on: 0 and 2 a walk (the stick at
  half), 3 a run, 4 the dash, and 1 the stick at 0.01, read as staying put (the tutorial's 31 and
  32, who spawn on their homes).
* **Its home area**: the trigger's h_f63b86a2 across and h_062fbaf0 high round the home
  (brain+0x214 / +0x218: each at least 7 when both are given, else 100 and 20), on from the spawn
  for a copy without a home-point and once home for one with (since #336: "Its goals on
  routes").

The tutorial's: the opening's wall sentry, TUTORIAL_terrorist_5, walks 10.5 m west along the
walkway to wpoint1493 as the level starts (its trigger at FIRST_UPDATE); the base's patroller,
TUTORIAL_terrorist_6, is sent 55 m into the base to wpoint1123 before it patrols (since #336 by a
route of 18 points, 66.5 m, home 27 s after its spawn); the lava yard's 29 and 30, the second
building's 3 and 17, the base's 7, 9, 13 and 21 run home (locomotion 3); the rest walk or are
home already.

## Its base goal

When nothing else is on (no contact, not going home), the brain does what its personality's
EVT_BASE (3) entry draws (FUN_00054c40, by weight):

| Personality | EVT_BASE | Who |
|---|---|---|
| h_e176c6ce | GOAL_PATROL 1, GOAL_IDLE_VARY 0 | the first patrol's TUTORIAL_terrorist_2 and _18 (template h_f2b759fb), the base's _6 (h_ffd92afd) |
| h_f0330ad7, h_eed3a76a, h_0bb586cf | GOAL_IDLE_VARY | the other Red Hand: they stand where they are (its pauses, cautious looks, crouches and walks home and about are #26's) |
| the placement's wingman one, h_ebe6044c (-> h_ea20a17d) | GOAL_WINGMAN_FOLLOW 1, GOAL_IDLE 0 | a squad-type character on the player's side: the V-squad's two Confed marines |

A goal whose start refuses (a patrol with no route) is struck out and the draw made again
(FUN_00054bc0); with every weight 0 the first goal is taken.

## Patrol

GOAL_PATROL (start FUN_0002af70, update FUN_0002b490; the walk FUN_00026d80, a point reached
FUN_00026940):

* **The route.** The level's patrol points (`bf/level.rs` `PatrolPoint`: `<patrol-point>`, kind
  11, and `<patrol-first-point>`, kind 12, a chain's head; the world keeps both kinds in one
  list, world+0x1d0) are linked by their forward-object and backward-object and grouped by their
  `iD`. The start takes the point nearest the character (3D, FUN_000394b0), the first head of that
  point's iD, and the chain from the head along the forward links until there's none or it comes
  back to the head; it starts at the nearest point's place on it (the head's when it isn't on it).
  Under two points it refuses.
* **The walk.** To each point in turn: the stick at half (FUN_0004d970 mode 2), a full walk by
  FUN_0012c6c0 (the type's walk speed, 3 m/s for the Red Hand), the relaxed walk clip, turning at
  the type's turn-rate (90 degrees a second); a point is reached within 2 m across (FUN_0002b490
  passes 2.0; a point's own h_e6564e8a when it's above 0: the four by the gate, 1 m).
* **The ends.** A point with no forward or no backward link ends the chain: the route is turned
  round (FUN_00023320 -> FUN_00038d30) and walked back. A chain whose last point links on to the
  head is walked round and round (FUN_00012650). The tutorial's two chains are open: walked to and
  fro.
* **The first leg** (since #336): the patrol's start routes the character to the valid point it
  can see within 20 m of the chain's current point (FUN_0004c2e0) and walks the chain once it's
  there; with no such point, or no route, the start refuses and the draw goes again.
* **In scripted play** (the world's cutscene bit, from a script's 122 to its 123) a character
  moves by its clip's root, at the rate it had, not by the speeds (FUN_0012cb40): the first patrol
  walks at its walk clip's 1.86 m/s through tutorial_show_patrol (play_enemies.rs `steer`).
* A point's trigger-object would be sent an event as a patroller reaches it (FUN_00026940); no
  tutorial point has one (logged).

The tutorial's two chains:

| iD | Head | Points | Walked by |
|---|---|---|---|
| 1 | h_11daf471 (18.8, 35.2) | patrol_point3 (11.1, 44.5) north to patrol_point11 (52.2, 76.9): 10 in all | TUTORIAL_terrorist_2 and _18 (from the head: their spawns are 2.5 and 6.6 m from it); one round trip ~66 s |
| 2 | h_18ddc4e7 (-37.7, 0.1), by the gate | patrol_point30 to 33 round the gate, 14 to 20, 29, 21 through the base to (-78.6, 47.6): 14 in all | TUTORIAL_terrorist_6, from the point nearest its home |

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
  its own eye and chest points for both, 1.6 and 1.1 m over the feet, and looks along its aim or
  its facing: only the ray's ends are the game's.)
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
  (EVT_TGT_LOST_SIGHT, FUN_0002e330; 10 s, FUN_0004a780), then the nearest seen.

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
  (his camera pivots on the same bone: 1.985 at idle, [camera.md](camera.md)). The demo's ray used
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
  ([grenades.md](grenades.md)); the demo's AI ray is the same function (`roller::clear`,
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

## Alert and the fight

With a contact known the brain fights: its attack goal's approach and pursuit walk routes
("Its goals on routes" below); where those stand, it fights where it is:

* its walk stops (the patrol is started again from the nearest point once it's over);
* it faces the target and fires at it while the target is seen and within its gun's reach (the
  shot's own: the MK-ASLT's 75 m), in the AI's bursts (play.rs `ai_fire`); unseen, it faces the
  target's last place, the gun raised (the turn toward a known contact, FUN_0002f960);
* **a friend in the line of fire** doesn't stop it, as in the game. The AI fires when the
  target is seen with a clear ray and its gun is on it (FUN_00024f80, FUN_0004dc10, neither of
  which tests anyone in the line), and that ray leaves characters out: the pair table's query
  (FUN_00057730) skips the object types in its mask 0x415a1018, characters (type 3) among them.
  So the friend takes the shot, at the own side's 0.7 of the damage (play_damage.rs, FUN_0004b9c0),
  and doesn't take it as an attack (no EVT_DAMAGED_NO_ENEMIES from a friend, 0x3e8fc). A
  terrorist stopped behind its buddy keeps firing into it until one of them moves: the game then
  posts EVT_SHOT_BLOCKED_BY_FRIEND (34, FUN_0022c4d0), whose goals reposition the shooter, and
  that move (with EVT_SHOT_BLOCKAGE's GOAL_REPOSITION_LOS_TO_TGT and h_0bb586cf's
  GOAL_ATTACK_REPOS) is #26's;
* once it knows no contact it takes up its base goal again, home first if a shot sent it there.

Its personality's EVT_ENEMY_SEEN draws the attack goal (GOAL_DEBUG1 for `default`'s line,
ATTACK_GUARD / ATTACK_GUARD_CROUCH for h_0bb586cf, ATTACK_COVER for h_eed3a76a) and posts
EVT_USER0 (GOAL_PURSUE) and EVT_USER1 (GOAL_HIDE). The approach and the pursuit are #336's
(below); the strafes, crouches, cover and hiding are #26's: there it stands.

**Shot, or shot at, while it knows no enemy**: a hit (EVT_DAMAGED_NO_ENEMIES, 0x3e756) or a
shot passing within 1.5 m (EVT_SHOT_AT_NO_ENEMIES; the radius is the squad's dodge's, a guess)
posts EVT_USER1, whose entry's quick goal h_f765fc45 adds the shooter to its contacts ("QUICK
GOAL!!! QG ADD OBJ TO CONTACTS being set here", the data's note on every Red Hand
personality), and EVT_RETURN_HOME. So a terrorist shot from behind turns on its shooter, fires
once it sees it, and goes home after the fight. A brain that knows an enemy registers the one
who hits it itself (0x3ea02), but a near miss tells it nothing then. The player's shots and the
AI's count alike; a hit from its own side posts nothing of this (0x3e8fc, FUN_0004b9c0). The
hit's reaction (the involuntary KNOCKDOWN / STUN draw) is play_enemies.rs's, which now counts a
contact as an enemy known. A contact knocked down stays a contact (the scan drops one only when
it's dead, unseen 15 s or stealthed).

## Aim

* **The AI's aim** (FUN_00024f80 -> FUN_00024c00, with FUN_00024ae0): each time it fires at a
  character the aim is on the chest with the probability of its accuracy, else off it by `up`
  turned about the line of fire by up to 45 degrees either way, 1 m long (1.2 m when it points up
  more than 0.7, as it always does for a level shot: a miss goes over the head), shrunk by the
  distance under 10 m (x d / 10). The accuracy is the placement's h_072837bb when its h_ec5337fb
  is set, else the type's h_12a04758 (percent), then + 0.05 x the difficulty (halved above 0),
  held within a half and one and a half of itself, at most 1: the Red Hand's 25 is 0.125 on
  STANDARD (-4), the sniper template's (h_ec5337fb false) type 50 is 0.3. A character of a squad
  a player controls skips the difficulty step (0x8d9f0 counts the players on its squad): the
  marines' 60 is 0.6. (FUN_00024c00 also never misses for a character whose skill's vtable +0x110
  says so, which also skips the raise below; not read here. With brain+0x3c4 bit 13, which
  FUN_00017f80 sets at the brain's set-up for a gun whose round falls (the iKhan-GPL's, the
  campaign hounds' balls) and only for a brain whose bit 8 is clear, so never for a squad type's
  (h_e78efbb2: the squad, the allies, every multiplayer type), it raises the aim point straight up
  by 0.5 x (1 + 13 x (min(d, 40) / 40)^2) m whether it missed or not; and FUN_00024f80 refuses
  within 1.5 x the gun's Damage radius, for every brain: both done since #83, in the shot builder
  and the trigger hold, [play.md](play.md), "An AI's lobbed rounds".)
* **The gun's spread** takes the holder's bonus (FUN_00222b30: h_072837bb with h_ec5337fb, else
  -1): the Red Hand's +25 narrows the MK-ASLT's spread ([play.md](play.md), "Shots";
  play_shots.rs).

## The Confed marines

The V-squad's two marines (Stout and Cogburn: trigger_spawn64 and 76, `h_0fa4f510` false, so the
member check keeps them and sets their flags, [triggers.md](triggers.md)) farcast in during
tutorial_farcast_vsquad_short (7.57 / 7.87 s), held by its 119s until it ends; then they follow
Tex with the squad AI's wingman follow (`squad_ai`; since #331 the game's GOAL_WINGMAN_FOLLOW
states, play_follow.rs, [squad.md](squad.md) "The follow": held while Tex is within 11.2 m, a
route to his own point at the dash once he's farther, stopped 9.8 m from him once he stops, a cover
point near him taken when there's one, kneeling or looking about while they hold; their mode never
changes, so they join no formation list and follow him; #243 is their own ticket). The tutorial's
yard (t335test's recipe): they reach 9.5-9.7 m from Tex at 85.7 and 88.9 s (main's formation
places within 8 m of the yard's end at 87.1 and 90.1 s), Cogburn walking to a cover point by him,
both behind him where main's stood in front. They're on the player's team (0,
their placements' alignment-type), see by the sensor above (90 degrees either way of their
facing), fire at what they see with their aim (above), and the Red Hand fire at them. Not done:
switching to them (the d-pad), their portraits, the squad commands. The test map's NPCs can follow
the controlled character the way they do, without a brain (#364, [testtools.md](testtools.md)
"Follower": the same follow, no formation list, no squad slot).

Their walk to their h_131de144 points: the spawn makes those points their homes (wpoint1380 and
wpoint1006, 0.5 and 0.9 m from their pads), but without a home-point nothing sends them there,
and a wingman follows its leader. take11's watch log agrees: the two stood on their spawn points
to 0.1 m (-16.9, -75.4) and (-21.1, -76.3) from the farcast (29.3 s) until Tex moved off at
156.9 s, set off 1.6-2.2 s after him and kept 8-40 m behind him in bursts of a run, and stopped
9-10 m behind him when he stopped. The demo's follow sets off once a marine's place is 3 m off,
and those places are the squad's, ahead of him: the game's wingman follow (FUN_00020b10, a
state machine of its own) isn't read. The script's 105 walks to those points
(0.26 s) name the marines before they're spawned and do nothing ([cutscenes.md](cutscenes.md)).

## Its goals on routes (#336)

The brain's goals walk the level's world points as the game's do: GOAL_RETURN_HOME, GOAL_PATROL's
first leg, the attack goals' approach, GOAL_PURSUE and GOAL_PATH_TOWARD_HOME, on the brain's one
route record, with the squad's stuck handling and steering (`src/bin/play_goals.rs`; how each is
read, and the route machinery, are [nav.md](nav.md)'s "The enemies on routes").

* **The fight.** The attack goal is drawn once a fight from the personality's EVT_ENEMY_SEEN
  (FUN_00054c40). ATTACK_COVER's and DEBUG1's cover choice are #26's, so the demo takes their
  no-cover path: ATTACK_COVER is refused and struck out (the draw's next goal, h_eed3a76a's
  ATTACK_VARY) and DEBUG1 runs ATTACK_VARY. GOAL_ATTACK and ATTACK_VARY approach a target beyond
  their stand-off by a route within FUN_0002e800's budget and stop short of it by (the near
  bound + 3 x the stand-off) / 4: 26.25 m for the Red Hand, the hounds and most mutants, 10.5 m
  for the Supermutant. There, or with the target unreachable, they stand and fire. The guards
  (h_0bb586cf's ATTACK_GUARD and ATTACK_GUARD_CROUCH) stand. A target whose ray has been cut 3.75 s
  is pursued (GOAL_PURSUE, a dash to its place, inside the home area) until a contact is seen
  again.
* **No fire on the approach.** The game's approach holds fire, and the demo's does: GOAL_ATTACK's
  update (FUN_0002e9c0) walks its route (FUN_00026d80 at 0x2f0b9) and jumps straight to the move
  (0x2f0f6 `je 0x2efd7`; at the route's end 0x2f120 `jmp 0x2efd3`), past its attack check and fire
  FUN_00024f80 at 0x2efce, which only its standing approach (2) and an event point's (3) reach;
  ATTACK_VARY's (FUN_0002d710) the same (0x2e114 `je 0x2e1ad` and 0x2e141, past FUN_00024f80 at
  0x2e19e). So the first patrol's leader, which main's build had firing where it saw Tex, holds
  fire through its approach and first hits him later ([nav.md](nav.md) "Verified" has the timing
  against take07).
* **The home area.** Outside it the sense step posts EVT_PAST_HOME_AREA and the brain walks home by
  GOAL_PATH_TOWARD_HOME (20.19, over the fight's 18): RETURN_HOME's route at a run, no fire, until
  it's more than 7 m inside or its route ends. ATTACK_VARY's approach stops within 1 m of the area's
  edge when the target lies outward. A copy with a home-point has it from the moment it's home
  (RETURN_HOME's end); one without, from its spawn.
* **The stuck handling.** A route walk that makes no progress for 2 s gets the squad's detour; when
  that finds nothing and the target was seen within the last second, GOAL_ATTACK_GUARD for 6 s
  (it stands and fires), else the re-route.
* **No stall guess.** A goal whose route can't be found refuses at its start (RETURN_HOME for
  good, PATROL drawn again), as the game's do.

The demo's brain has no INVESTIGATE, GOTO, HIDE, SNIPE or SCOUT draw and no event posts a point,
so GOAL_IDLE's route to one isn't reached; those goals, the cover choice and ATTACK_VARY's state 2
are #26's.

**Against todo/132-mutants** (e23, the takes of the mutants' fight; the cover, the strafes, the
grenades and the specials are #26's and #374's): the Uglys (`default`'s line) never came nearer
than 10.6 m, held 17.9-31.5 m in the open and stood off 11.6-13.6 m at their cover points; the
demo's approach stops at 26.25 m and a mutant within its 30 m stand-off stands where it is, so it
holds inside the open-ground band and never closes under it (no cover, no fallback at 3 m). The
home area matches the data and the code to the footage: take07's Ugly (its trigger's 9 m area
round (-71.4, 44.4)) takes PATH_TOWARD_HOME as it crosses 9.05 m out and ends it 1.99 m from the
centre (7.01 m in), and take08's Ugly on the ledge (7 m) goes out on ATTACK_VARY and walks back by
PATH_TOWARD_HOME, as the demo's does in the same staging. The Molotov Mutant's trigger gives no
area (100 m by 20), which fits its fighting 16 m from home in the takes.

The notes' copies that "ignore the player and walk home" are those without a home-point, as the
review reads them: 27 of e23's 76 spawn triggers have `home-point=""`. FUN_00018510 gives such a
copy its leash at once (+0x21c bit 1), so out of its home area PATH_TOWARD_HOME (20.19, over the
fight's 18) carries it home without firing, in the game and the demo alike. (16 of the 27 give no
size and 10 a radius without a height, which FUN_00018510 doesn't take: 100 m by 20; one gives 9
by 1: 9 m by 7.) A copy with a home-point has no leash until it's home (FUN_00018510 clears the
bit, RETURN_HOME's end sets it), so on its way there it fights whom it sees, EVT_RETURN_HOME's 1.5
being under EVT_ENEMY_SEEN's 18, as read; the demo's e23 copies do.

## Held by a script

A character a cutscene holds (119, the script's start, a 105 walk: [cutscenes.md](cutscenes.md))
has no brain meanwhile (char+0x834 bit 0): its walk stops, and it takes its goal up again once
let go. The grenade lesson holds the first patrol's two while it plays.

## Test hooks

* `BF_BRAIN_LOG=1`: each brain at its spawn (home, its pace there, sight, field of view,
  accuracy, gun bonus, personality), its base goal and the draw, its patrol route (iD, points,
  where it starts), each point reached, home reached or given up short of (and how far), each
  goal's start, refusal and end (#336), each contact seen (with the heights over their feet the
  sight ray ran between, #244), its target, contacts forgotten (and why), a shooter it learns of,
  stopping to fight. The copies are named as the triggers' log names them.
* `BF_TEST_AIM_HOSTILE=<s>`: from that time the controlled character's camera turns onto the
  nearest living hostile it can see (within 120 m, nothing of the world in between: its chest,
  else its head over cover) and it aims and fires at it: the takes' aim helper
  (todo/28-tutorial/notes.md), to stage a fight. With `BF_TEST_GOTO` it walks on meanwhile,
  toward the hostile while it aims (the walk follows the camera). It runs after the other input
  hooks (its fire wins over `BF_TEST_TRIGGER`'s) and after the free camera's input gate. With a
  gun that has a scope it aims through the scope's first level, as the other hooks' aim does
  (play.md, The scope), and leaves the scope it went into once no hostile is in sight.
* `BF_GOAL_WATCH=<s>` (#336): every s seconds a row for each level character with a brain
  (not following) and each test map NPC set to fight: its goal, where it is, its own point, its
  route's entry, its target and how far, its pace ([nav.md](nav.md), test hooks).
* The tutorial's panels (#224) stop the clock until closed: `BF_TEST_PANEL=a,1` closes each
  after 1 s (in the first patrol's run below the WEAPONS panel comes up at ~17 s).

```sh
# the first patrol from its plate: the script, the walk past the camera, the fight
BF_MAP=tutorial BF_TEST_GOTO=44,91,34.4,94.3 BF_TEST_PANEL=a,1 BF_BRAIN_LOG=1 BF_COMBAT_LOG=1 BF_TEST_AIM_HOSTILE=14 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the first patrol walked to and fro, Tex far away at the mine field
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router_spawn_first_patrol,USER1,1" BF_BRAIN_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#336) every spawn router of the tutorial fired, Tex away: the walks home by route, the patrols
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router103,ENTER_DETECTOR,1;trigger_router38,ENTER_DETECTOR,1;trigger_router50,ENTER_DETECTOR,1;trigger_router22,ENTER_DETECTOR,1;trigger_router473,ENTER_DETECTOR,1;trigger_router_spawn_first_patrol,USER1,1;h_020415dc,ENTER_DETECTOR,1" BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#336) e23 staged as todo/132-mutants' F1: Tex warped in once the four live mutants are home
BF_MAP=e23 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 "BF_TEST_WARP=25,me,-57.7,64.8,180" BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_GOAL_WATCH=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
```

## Verified

Muted runs in the background window (`BF_MUTE=1 BF_BACKGROUND=1`), captured at 15 fps, against
todo/28-tutorial's takes:

* **The wall sentry** (`BF_MAP=tutorial`, the level's start with tutorial_start_long):
  TUTORIAL_terrorist_5 walks west along the walkway through the first shot by its walk clip's
  root (scripted play) and is home, within wpoint1493's 1 m, 5.1 s into the 6.1 s shot. In
  take03 it's still walking at ~5.9 s (the game's route home isn't the straight line).
* **The first patrol** (`BF_TEST_GOTO=44,91,34.4,94.3`, take07): TUTORIAL_terrorist_2 and _18
  come with tutorial_show_patrol and patrol chain iD 1 from its head (2.5 and 6.6 m off it),
  through the shot at the walk clip's root speed, 1.86 m/s: the leader passes its second point
  6.5 s into the script (take07: ~5.7 s). With Tex standing on the plate (`BF_TEST_PANEL=a,1
  BF_COMBAT_LOG=1 BF_SHOT_LOG=1`), the leader sees him at 35.5 m 15.3 s after the spawn, stops
  and fires: 10 of its 59 shots hit him (17%; 0.7 to 1.7 HP a hit; take07 1.1 to 1.8). Take07's
  first hit came 14.2 s after the script's start, the demo's 17.2 s (#227's build: it fired where
  it saw him; since #336 it walks in first, and since #244 it sees him sooner and farther: the
  table under "#244" below). The second stops right
  behind it (its muzzle 0.1 m from the leader's body) and fires through it: of its 88 shots 5 hit
  the leader (x0.7: 2.1, 1.4, a 16.6 head shot, 1.7, 2.0), killing it 13 s in, and 3 hit Tex once
  the leader was down (3.4%). With the aim helper (`BF_TEST_AIM_HOSTILE=14`) Tex kills both:
  +$60 each.
* **The patrol walked out** (Tex away at the mine field, the router fired): the chain to
  patrol_point11 and back, turned round at each end, ~66 s a round trip, on and on.
* **The marines** (`BF_TEST_GOTO=-15.6,-87.4,-60,-75 BF_TEST_GO_AT=14`, take11): they farcast
  in at 7.57 and 7.87 s of tutorial_farcast_vsquad_short, held until it ends, stand on their
  spawn points until Tex moves, then run with him (`squad_ai`) and, when he stops at
  (-60, -75), fight terrorists 31 and 32 at 20-40 m (they see and shoot the marines too): both
  terrorists die to the marines' fire 10 and 12 s after they spawn (+$60 each). Stout hit with 7
  of his 11 shots, Cogburn with 8 of 10 (accuracy 0.6); one of Stout's went into Cogburn, in his
  line (x0.7: 1.6 HP). The terrorists hit with 3 of 16 and 1 of 12.
* **The lava yard** (`BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999`, Tex on the plate
  h_f42fa5b8, take08): the plate's router spawns 29 and 30, who run home across the pit (1.5
  and 2.3 s). Before #244 nobody saw Tex in 60 s: the demo's sight ray took the hinters for
  blockers, and the band h_ee99c575 (body 64) lies twice across the line from their homes to the
  plate (with #338's own hinters in; on e01's shapes before, 30 saw Tex at 26.9 m on its way,
  stopped and fired, -1.7 a hit, and died to the aim helper). Since #244 the ray leaves the
  hinters out and runs head to head ("The sight ray"): 30 sees Tex at 28.5 m at 2.67 s, still on
  its run home, stops 26.0 m off at 3.2 s and fires, first hit at 8.9 s (-1.7 to -2.9, 9 hits in
  the 60 s); 29, home at 2.53 s, stands facing along its run and sees no one. **take08 doesn't
  agree with that:** nobody fired on Tex while he stood on the plate (93.9-106 s) nor in the 20 s
  after he'd walked on to (46.3, -25.9); his first hits (-1.5 to -2.8) came at 129.68 s, 35.8 s
  after he stepped on the plate.
  The game's ray was clear for long stretches of it: its radar shows red blips 12 px (25 m, at
  2.05 m a pixel) ahead of him at 95.5-96.0 s, as a ping, and 10 px (20-22 m) from 104.5 s to 132
  s, with gaps (the two at their homes, 19-21 m off, through the band), and the frame at 118 s has
  one standing in the open facing him. So the divergence is a **hold upstream of the ray**: the
  pair's ray is clear from 104.5 s, the terrorists are in view, and nothing is fired until 129.68
  s, 25 s later. The demo has no such hold, and what makes it in the game isn't found (the
  brains' state at their spawn, allow_combat's grant and a script are all unread). The demo's 30
  fires on its way home where take08's wait 35 s: the match the hinter band gave was by the wrong
  cause.
* **The base** (staged: its routers fired with `BF_TRIGGER_FIRE`, Tex standing at the gate, in
  the plaza and at the third building with the aim helper): terrorists 0, 6, 8, 9, 10, 11, 13 and
  15 were killed, most having seen and fired on Tex (the sniper 11 hit him for 8.6-10 from 51 m
  and killed him once, Tex standing at the third building); every death paid +$60 and counted on
  h_e8ea7ba2 (router99's twelve). The wall sentry (on its walkway), the sniper 7 and the third
  building's 12 and 21 (inside its walls) were out of the aim helper's sight from the spots
  tried, so no run cleared all twelve (take12's clear had the marines and healing).
* **Every spawn at once** (Tex away, every router fired): who arrives home and who stalled on
  the straight walks before #336 (the next bullet: since #336 they walk routes); the paces from
  `locomotion` as logged.
* **#336, the goals on routes** (nav.md's "Verified" has the runs): the tutorial's walks home
  arrive (21, 13, 6 and 3 included, which main's straight walks stalled), 9 stopped at its shut
  door and brought inside its area by PATH_TOWARD_HOME; the patrols walk their chains; m09_a's
  Hounds are home at 15.8 s; the first patrol approaches to ~26 m, pursues when it loses Tex and
  first hits him at 24.9 s (main 20.6); the test map's Filthy Mutant closes from 45 to 25.9 m and
  fires; e23's mutants keep their home areas as in todo/132-mutants; testtools.md's NPC fight
  tells the same story (the NPCs within their stand-off stand, as before; the hits after the
  first few differ, the attack goal's draw taking a random number).
* **No regressions** (main's build, e5d2f74, against this one): e40's first 40 s (the same log,
  no frame differing by more than 24 levels), sdm_e34's autopilot (the same log, 296 of 300
  frames the same: the first four differ between two runs of one build too), the test map's NPC
  fight of testtools.md (the same combat log, 222 of 225 frames the same and the other three by
  1-2 pixels). m09_a's four Shrikes now run home at its start (two Hounds stall 23 and 29 m
  short, off their ledge): seen in the opening shot, where they stood before.
* **After main's #224 merged in** (main 9686256's build against this one): the hooks under the
  free camera, `-- --test` e40's take04 scene (`BF_TEST_TRIGGER=3.0-3.15` with
  `BF_TEST_FREECAM`: 69 of 70 frames the same, the Therm Sweeper's 80 -> 79 on the same frame)
  and the test map (`BF_TEST_TRIGGER=2.0-2.3`: 50 of 50; `BF_TEST_FIRE=1 BF_TEST_SWITCH=2.2`: 75
  of 75, the fire held off by the gate in both, the switch at 2.2 s in both); sdm_e34's
  autopilot (the same log, 297 of 300); the NPC fight (the same combat log, 221 of 225: one
  tracer two frames earlier, where main had it before #224, and 1-2 pixels). The first patrol
  (with `BF_TEST_PANEL=a,1`) and the marines' fight as above.
* **The shared sensor** (#197; main 403630b's build against this one): the first patrol's run
  above, 40 s, gives the same brain and combat log line for line. The squad's and the NPCs'
  sight on the test map (an e40 Shrike set to fight at (4.5, 4.3), Tex at z = 84.3 / 114.3, i.e.
  80 / 110 m): no sighting, no shot, as main since #123; Tex walking in from 95 m: a contact at
  74.8 m and fire; walking out from 70 m: fired on out to 81.6 m, lost at ~86 m (75 x 1.15,
  7.53 s), forgotten 3.75 s later (11.27). take14's recipe (skills.md): lost at 5.87 as main,
  "forgets Squad(2) (stealthed)". testtools.md's NPC fight: the same story (enemy Brutus fights
  the squad and dies, Flint set to fight fires back, set to friend stops), the hits' order
  shifted by the squad's field of view.
* **#244, the sight ray's end points** (main d61b190's build against this one; `BF_MUTE=1
  BF_BACKGROUND=1`, 15 fps). The first patrol (the recipe in "Test hooks": Tex walks onto the plate,
  (34.4, 94.3), `BF_TEST_PANEL=a,1 BF_COMBAT_LOG=1 BF_SHOT_LOG=1`; times from
  tutorial_show_patrol's start, sim 2.27 s; take07's from its letterbox, video 41.77 s, where Tex
  stood at (32.67, 93.7), 1.8 m from the plate's spot):

  | | main | #244 | take07 |
  |---|---|---|---|
  | the leader sees Tex | 16.0 s, 35.3 m | 14.3 s, 40.0 m | radar blip at 12.8 s (video 54.6), 20.8 px from the middle: 43 m (the radar's 120 m across 39 units, 2.05 m a pixel in the 1280 x 720 frame; #245's 39-42 m) |
  | the second sees him | 17.3 s | 15.7 s | |
  | the leader's approach stops | 17.7 s, 25.9 m | 16.7 s, 26.2 m | it doesn't: its blip stays at 20.2-21.3 px, ~41 m, from 54.6 to 68.8 s (7.4 s of game time, the WEAPONS panel's stop taken out) |
  | its first round / first hit | 21.1 s / 22.7 s | 17.1 s / 21.7 s | tracer 13.2 s (video 55.0) / hit 14.2 s (video 56.0) |

  On the plate's spot the new end points put the leader's sighting 4.7 m farther and 1.7 s sooner
  than main's (40.0 m, against take07's 41-43 m), and its ray stays clear once it has stopped (main:
  cut from 16.5 s, so it pursued at 20.2 s; now only the second, 1 m behind it, loses the ray, from
  18.1 s, and pursues at 21.8 s). The leader's head is 1.76 m over its feet at the sighting, Tex's
  2.01 (the sighting line of `BF_BRAIN_LOG`). The lava yard is above: the terrorists now see
  across the pit, and the demo's 30 sees Tex at 2.7 s and hits him from 8.9 s where take08's wait
  35 s.

  **The tester's findings** (this build against main's, on the same recipes):
  * **At take07's own spot** (Tex standing at (32.67, 93.7)) the sighting is not nearer the take
    than main's: main's leader sees him at 12.66 s, 42.8 m; this build's at 11.60 s, 45.3 m;
    take07's blip shows at 12.8 s, 42.6-43.7 m. There the old ray was the nearer, and the new one is
    1.2 s early and about 2 m far. The pair table's delay ("Not modelled", above: an unalerted pair
    is tested about every 1.5 s) would put the game's sighting after the geometry's by about that, a
    reading and not a measurement. The "#244" column of the table is the plate's spot alone.
  * **Rounds into the slope.** At take07's spot the terrorists now land 4 hits in 40 s where main's
    land 27, and 31 of 143 and 58 of 183 of their rounds (two runs) strike the terrain less than 20
    m from the shooter, against main's 0 and 3. The AI aims at the demo's chest, 1.1 m over the feet
    (`AI_AIM_HEIGHT`, play.rs `ai_fire`), where its ray now clears at head height, so a chest line
    the slope hides is fired on as one it sees. (FUN_00024f80 aims at the target's vtable +0x40
    position, its root node, about 1 m over the feet, plus FUN_00024c00's miss; so the game's AI
    also fires a hips line under a head-height ray, and why the demo's rounds strike the slope so
    much oftener is open.) A follow-up: **which point the game's AI aims at** (FUN_0022dc60 /
    FUN_00024c00). The first hit stays behind take07's besides because take07's leader stands ~41 m
    off and fires 0.4 s after the blip, where the demo's walks in to 26 m holding fire (the attack
    goal's approach, GOAL_DEBUG1's mapping to ATTACK_VARY, #26's).
  * **The lava yard's divergence** is a missing hold upstream of the ray: take08's radar has the
    blips from 104.5 s to 132 s and nothing is fired at Tex until 129.68 s ("The lava yard" above).
  * **e23's ledge Ugly** is now shot by the squad within ~1.1 s, so that staging no longer shows the
    PATH_TOWARD_HOME loop (in my run it makes one out-and-back, at 34.7 s and 39.0 s, where main's
    makes three, at 33.6 / 37.9, 50.8 and 66.2 / 70.5 s).
  * **Frame time** is up 10% in a 16-Shrike fight: the ray is cast every frame for every candidate
    in reach and in view (the pair table's schedule, in "Not done", would cut it).

  **Unchanged**: `cargo test` (`scan_by_the_team_table`, #175's table; the new
  `the_sight_ray_runs_between_two_points`); the test map's NPC fight of testtools.md, 270 frames, and
  the sensor run above (Brutus set to fight at (4.5, 4.3), Tex walking in from 95 m): the same
  sighting lines (the first at 77.5 m, "sees unit 4 at 42.2 m" ...), the hits' counts within what
  two runs of main's alone differ by (the load times move the frames a runs' events fall in);
  take14's recipe (skills.md): "forgets Squad(2) (stealthed)" at 5.87 and a contact again at 18.20
  in both; e23 staged as todo/132-mutants' F1: the same mutants act (the Ugly at 16.9 m stands and
  fires; the ledge Ugly approaches, leaves its 7 m area and walks back by PATH_TOWARD_HOME, but the
  squad shoots it too soon for it to loop, above), their contacts made from farther (a mutant sees
  the squad at 56.9 m where main's saw Tex at 48.0); e40's first 40 s (the same log, a frame 18
  pixels off) and sdm_e34's autopilot (the same log, 300 frames the same). What did change is in
  the tester's findings above.

## Not done

* #26: the attack goals' other moves (strafes, crouches, cover, repositioning for a clear line,
  the fallback from a target too close), calling for help, investigating noises and bodies, the
  idle variations (pauses, cautious looks, crouches, walks), grenades.
* Of the goals on routes (#336): the follow-ups in [nav.md](nav.md)'s "Not done" (e40's patroller
  called home each round of a chain that leaves its 15 m area, terrorist 6's pauses at two corners
  on its way home, e23's ledge Ugly stepping off its ledge on its way home).
* Of the sight ray (#244): the pair table's schedule (FUN_00057890's intervals, five pairs an
  update, the bit kept between, the ray cast only while one has the other in reach); the reach and
  the field of view from the game's points (the reach from the owner's position record to the
  target's; the fov test, 0x3d300, from the head node's place, char+0x3e8, along its matrix's Z
  column at the target's root node: the demo keeps its eye and chest and looks along its aim or
  facing); the ray cast at once when a contact is added or in GOAL_ATTACK_COVER's update; the AI's
  aim point (the follow-up in "Verified", #244). And what the two takes show that the ray doesn't
  explain: take07's first-patrol leader stands ~41 m from Tex and fires from 0.4 s after it's seen,
  where the demo's walks in to 26 m holding fire (the attack goal's mapping, #26); take08's
  lava-yard terrorists don't fire on Tex for 25 s with the pair's ray clear and one in plain view
  facing him (a hold upstream of the ray; the brains' state is unread).
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
