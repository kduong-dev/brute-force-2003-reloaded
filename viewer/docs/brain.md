# The level's characters: their brains (`src/bin/play_brain.rs`, #227)

What a character a level's spawn-trigger makes does on its own: the Red Hand's patrols, their
walk home, their sight and alert, their fight and aim, and the Confed marines following Tex.
It's the tutorial's step 8 (#28) and the first slices of #26 (the AI): the goals on routes and
the pursuit (#336), the cover (#384, "Cover" below), the fallback from a target too close (#383,
"The fallback" below); strafing, grenades and calling for help are #26's other parts. The squad's
own AI (`squad_ai`, [squad.md](squad.md)) and the
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
  facing); with nothing of the world between its eyes and the chest (the demo's eye-to-chest
  ray; the game's pair table casts its ray between the two characters' FUN_0004cea0 points,
  FUN_00057a30, not matched here: #244). The sight distance is the
  type's `vision-range` (the Red Hand 75 m, the marines 80), or the placement's own `vision` when
  its h_11cddd9f says so (the sniper lesson's target: 10 m). The field of view: 60 degrees the Red
  Hand, 90 the marines. The gun's range plays no part in sight.
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
(below), the cover #384's ("Cover" below) and the fallback from a target within 3 m #383's ("The
fallback", below); the strafes and hiding are #26's: there it stands.

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
  (FUN_00054c40), and again when ATTACK_COVER ends; a goal whose start refuses is struck out and
  the draw made again (FUN_00054bc0). ATTACK_COVER and DEBUG1's cover half are "Cover" below
  (#384): ATTACK_COVER refused (no cover point) gives h_eed3a76a's next goal, ATTACK_VARY, and
  DEBUG1 not in cover runs ATTACK_VARY. GOAL_ATTACK and ATTACK_VARY approach a target beyond
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
so GOAL_IDLE's route to one isn't reached; those goals and ATTACK_VARY's state 2 are #26's.

**Against todo/132-mutants** (e23, the takes of the mutants' fight; the strafes, the grenades and
the specials are #26's and #374's, the cover #384's): the Uglys (`default`'s line) never came
nearer than 10.6 m, held 17.9-31.5 m in the open and stood off 11.6-13.6 m at their cover points;
the demo's approach stops at 26.25 m and a mutant within its 30 m stand-off stands where it is, so
it holds inside the open-ground band and never closes under it (no fallback at 3 m). Since #384 an
Ugly outnumbered by its target's threat (every one against Tex in the takes) goes to a cover point
first, as take06's does ("Cover"). The
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

## Cover (#384)

`src/bin/play_cover.rs` (`Brain::cover`), from `think`: GOAL_ATTACK_COVER (19) and GOAL_DEBUG1's
(120) cover half, part A of #26. Read from ATTACK_COVER's start 0x2f8e0 (the pick FUN_0002f270),
update FUN_0002fd40, event hook FUN_000313e0 and end 0x2f900, and DEBUG1's start 0x36a70, update
0x36b70, hook 0x36ca0 and end 0x36b50 ([code-map.md](code-map.md), "AI");
`todo/26-enemy-behaviour/leads.md` §3, with the corrections below. The level's `<cover-point>`s
are read with their facing (the frame's third column negated), their cone (`fov-angle`, 100 by
default, 60 at least) and their bits (`enable-left`, `enable-right`, h_18ed19af, h_f9f3561f,
`enable-dodge`; `bf/level.rs` `CoverPoint`).

* **Who.** DEBUG1 is `default`'s EVT_ENEMY_SEEN goal: the tutorial's first patrol (h_e176c6ce),
  e23's Uglys (h_e399930f). h_eed3a76a draws ATTACK_COVER (e23's h_e187e355 and h_e880d3c3), and
  h_032b0602 (e23's ledge Ugly) ATTACK_COVER, then ATTACK_VARY when it finds no point. A test map
  NPC has no brain to hold a point: its ATTACK_COVER refuses, its DEBUG1 runs ATTACK_VARY.
* **DEBUG1** takes cover only when FUN_00036970 says so: the brain under half its hit points, or
  its target's threat (FUN_0004a3a0 at the target's distance) over twice its own at 25 m, or the
  sum over its contacts over four times its own. Otherwise, or with no point to be had, it runs
  ATTACK_VARY (the approach, "Its goals on routes"); once the cover ends, ATTACK_VARY; from
  ATTACK_VARY, 5 s on, cover again whenever FUN_00036970 says so (tried every frame until a point
  is found). The threat score is the gun's: its rounds a second x damage x the holder's accuracy
  (over 20 1.0, 10 0.5, 5 0.3, else 0.1; 0 out of its reach), x2.5 to x1.25 by its blast radius,
  x2/3 for an accuracy max under 80, x1.25 for bullet-type 2, x1.5 when its target is the brain
  scoring it, then by the distance (x1.5 and at least 0.5 under 15 m, x1.1 at 15-35, x0.75 over 50;
  other bands with a scope). An e23 Ugly scores itself 0.33 at 25 m and Tex 1.50, the first patrol's
  Red Hand 0.08 against 1.10: against Tex every one is outnumbered.
* **The pick** (FUN_0002f270 at 0.75-30 m): a target is needed, within the held gun's reach when
  it's seen with a clear ray. The flood from the character's own point through the links within
  30 m takes the first cover point that is valid, not claimed, not the one held, 0.75-30 m from
  the character's node position (its vtable +0x40, the hips: the feet + the playing clip's
  root-height, 0.93 m for a standing mutant, 0.45-0.50 crouched; 3D, both bounds taken, 0x5e070;
  nothing exempts a point it stands by), inside its home area, its set-valid on, not on its
  blacklist (10 s) and, on a first pass, not one of its last four; and, for every enemy it knows
  (at their hips too), 5-50 m from it, not past it (an enemy nearer than the point within ~75
  degrees of its bearing) and inside the point's cone (under half the fov-angle off its facing).
  There's no sight test. Then a route to it ([nav.md](nav.md) "The enemies on routes": walked at a
  run facing the target, the dash only for one under half its hit points going far; the run lets
  the crouch go, FUN_0004d970), the point claimed, the re-pick timer 23 s. So a mutant standing
  0.65 m across from a point (take08's Ugly by h_eed0869f, 0.70 m from its feet and 0.93 m from its
  hips) takes it and crouches in place, and one standing on its point after a grenade throw
  (take06's on h_157fd721, 0.60 m from its standing hips) can't take that point again.
* **At the point** (the state brain+0x518): it settles (the point on its last-four list), then
  holds: it faces the point's facing, crouched where the point has h_18ed19af (a type with an
  MT_CROUCH motion); the nearest enemy in sight is faced and fired at, which keeps the hold timer
  (1-2.5 s from the arrival); the target seen within 8 m blacklists the point and fails the goal;
  over 0.9 m off a point that isn't a crouch point fails it too. After 10 s at the point, or the
  hold out, a re-pick, when an enemy is in sight or the 23 s re-pick timer is out, moves it to
  another point; otherwise, or with none found, it peeks: at a crouch point (h_18ed19af without
  `enable-dodge`) it stands up and fires (the pop-up); at a point open on neither side it fires
  from the point for 3-4 s; else to the open side (with both, the target's: where it is, or where
  it was last seen when it was seen out there the time before, FUN_000237b0): a dodge motion (with
  `enable-dodge`, 70% of the time; 30% of those a short one) or a step out along the point's right
  or against it until it's 2 m from the point, 1.5 m from where it started and clear of the cover
  in front; out there 5-7 s firing (cut to 1 s when it hasn't seen its target since it came out),
  then back: the opposite dodge (always after a short one, 30% of the time after a full one) or a
  run back.
* **Exits.** An enemy it sees outside the point's cone (30 degrees more field of view while it
  holds) flanks it: done. The third hit while it holds blows the cover (the point blacklisted):
  done. ATTACK_COVER over, the attack goal is drawn again; DEBUG1 goes on with ATTACK_VARY. The
  point stays claimed until the next pick, the goal's end or the character's death.
* **The hook** (FUN_000313e0, before the brain's own handling of an event: the handler returns at
  once when the hook says 1): a hit while it holds counts toward the three, and one while it's
  out cuts a peek over 1 s to 0.5-1 s; a shot passing close cuts a peek over 2 s to 1-2 s and holds
  the hold timer at 1.5 s at least. It keeps EVT_DAMAGE_LIGHT (no knock-down or stun drawn for a
  light hit while it holds cover), EVT_SHOT_AT, the two shot blockages, TGT_TOO_CLOSE and, but out
  of cover, TGT_ENTRENCHED from the personality.
* **Its crouch and dodges.** The crouch is the posture request (brain+0x3cc): `crouch_wanted`,
  and a crouched enemy turns to the goal's look as the AI's body follows its view at a stand
  (`step_player`, FUN_00119fa0); a run or the dash lets it go, and a walk for a type without
  MT_CROUCH_WALK (FUN_0004d970), so a character crouched at one point stands at the next unless
  that one asks for the crouch. A dodge is the command CT_DODGE_* played as MT_DODGE_LEFT /
  _RIGHT / _SHORT_LEFT / _SHORT_RIGHT (15-18, the command handler's cases 0x122321-0x122357),
  with the DODGE cooldown 3 s (and the knock-down and stun cooldowns 2 s); a dodge sent while a
  hit reaction plays is dropped (logged at `BF_COVER_LOG=1`). The side is the game's: the point's
  right is the ORIENTATION's first column, and `enable-left` alone peeks against it
  (todo/26-enemy-behaviour/answers-384.md §1).
* **A goal pushed on top** (for #386, the grenade throws): when the goal machine pops a goal
  (FUN_000377b0), the one under it is started again (its start, 0x37d85), not resumed. So after a
  THROW_GRENADE DEBUG1's start runs again: FUN_00036970, then a fresh pick from where the
  character is, which can't take the point it stands on (under 0.75 m from its hips): it runs
  ATTACK_VARY for 5 s and picks again from where that leaves it. The demo pushes no goal yet.

**Against the footage** (todo/132-mutants; the cover's own shots C1-C5 aren't filmed, so its
timings are the code's):

* **take06** (F2: the Ugly 83302960, the demo's h_eef1a370, put at its take position (-64.1,
  46.8), Tex 4.2 m from it): the game's goes ALERT, backsteps, starts DEBUG1 at 3.08 s and crouches
  at its cover point (-66.6, 37.8) 13.5 m from Tex by 8.1 s, then holds 11.6-13.6 m. The demo's
  starts DEBUG1 at once (no ALERT: #26 part F) and takes the same point, h_157fd721 (9.5 m off,
  13.4 m from Tex's hips); h_eed0869f beside it is refused, Tex's hips being 3.9 m from it (under
  5). It's there 4.5 s later (the game's 5.0 s after its DEBUG1), crouches facing Tex and fires from
  the crouch at 13.4-13.5 m. Side by side at take06's 13 s the two crouch in the same spot. After
  its 10 s it finds no other point and peeks left, as the game's code does at a point open on the
  left only (todo/26-enemy-behaviour/answers-384.md §1), which takes it out of its 9 m home area:
  GOAL_PATH_TOWARD_HOME brings it back and DEBUG1 takes the point again, about every 16 s. That
  loop is the game's own rule when nothing interrupts the cover; take06's Ugly never shows it
  because a grenade throw (THROW_GRENADE on top, #386) came every 9.2 s, under the 10 s hold, and
  after each DEBUG1 was started again: the point under it refused (0.60 m from its hips),
  ATTACK_VARY's coin dodges and link strafe for 5 s (take06's moves toward home, 19.1-29.0 s),
  then a fresh pick that walked it back to the point (22.6-23.9, 30.8-32.3 s).
* **take08** (F1: Tex at (-57.7, 64.8), the three mutants within 20 m turned to him): the game's
  Ugly 83302960 crouched where it stood, by h_eed0869f, from 6.6 s until its ALERT at 11.1 s, and
  came no nearer Tex than 18.9 m. The demo's takes h_eed0869f at once (0.64 m across, 0.70 m from
  its feet but 0.93 m from its standing hips), steps 0.6 m onto it and is crouched 1.0 s after it
  saw Tex, 18.8 m from him, firing; after its 10 s, with Tex in sight, it moves to h_157fd721
  (28.6 m from him) and back, every ~14.6 s. 832fe760 (h_f5f8f2ca, 11.4 m from Tex) takes
  h_faf90c97 18.1 m from him and peeks from it by dodges and step-outs (take08's: no nearer than
  10.6 m, at its start). Main's build at the same spot has 832fe760 standing in the open 11 m from
  Tex.

**Corrections to the leads** (todo/26 §3 and the code map's rows before #384): the hook's 1 keeps
the event from the brain (the handler returns at once; EVT_DAMAGED and the rest go on); its
EVT_SHOT_BLOCKAGE / BLOCKED_BY_FRIEND case sets ATTACK_VARY's behaviour timers, not the peek
timer; the timed re-pick waits on the re-pick timer, not the peek timer, and a failed one sets it
again; the threat's x2/3 is for a gun whose accuracy max (the weapon data's +8) is under 80, and
its x1.25 is for bullet-type 2 (+0x40), not a falloff mode; DEBUG1's start tries cover only when
FUN_00036970 says so; FUN_0002f960's extra field of view is 30 degrees and its caller is
ATTACK_COVER's update (the flank test); FUN_0002e330's 1.5 s is by the target's goal (the target
in cover or dodging), not the brain's own. And from the review of #384 (answers-384 §2): the pick's
0.75-30 m is from the character's hips, 3D, both bounds taken, with the enemies' hips likewise (the
first round measured from the feet, so take08's Ugly couldn't take the point it stood by).

## The fallback: a target too close (#383)

`src/bin/play_fallback.rs` (ATTACK_FALLBACK and MOVE_BACK; the link walk is `Graph::away`, [nav.md](nav.md)
"The link walk"). Read from default.xbe (the asm of 0x44b57, 0x31610, 0x316d0, 0x29920, FUN_00029a00,
FUN_0004bbd0) and checked against todo/132-mutants take07 (F2b); the rows are in
[code-map.md](code-map.md).

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
* **The draw**: the personality's entry for 85 (`default`'s and so the Red Hand's, the mutants',
  the hounds'; h_0bb586cf's guards too): priority 20.17, **ATTACK_FALLBACK 1 : ATTACK_COVER 0**, over the
  fight's 18, under PATH_TOWARD_HOME's 20.19 (that goal comes first in `think`). One that refuses
  (the start finds no way) is struck out and the draw repeated, which comes to ATTACK_COVER: the
  demo has no cover (#26's), so nothing then and the attack goes on. The Molotov Mutant's entry has
  priority -1 (NOOP); the Supermutant's and the Shadoon's draw their own goals first (116 the
  pound, 117), which the demo doesn't have (#212): they stand as before. An event that comes while a
  hit reaction plays waits for its end (STUN and KNOCKDOWN aren't interrupted), and a reaction
  that starts during the walk ends it.
* **The start** (0x31610, FUN_00029a00): the direction **me - the target** (the two position
  records, 3D, **not normalised**) and the link walk from the character's place (its position and
  own point) with cos 0.6 and length 7.0 over its route: the first link in each point's list that
  lies within the cone, 7 m of legs, [nav.md](nav.md). None: refused. The move's heading memory is
  dropped (`Avoid::dir_was`: it turns at once), the attack goal's approach and pursuit are broken
  off (they start again after) and the character says fe56cd97 ("the fallback", the mutants' chatter:
  not said, the chatter is #36's). On a map without world points (the test map's NPCs) the demo
  walks 7 m straight along the direction.
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
* **Not here**: ATTACK_COVER, the second draw; F2b's first post (9.60 s) did nothing and the second
  (11.19 s) started the goal 0.12 s on: the Ugly was in DEBUG1's cover state, crouched at its point,
  whose event hook (0x36ca0) isn't read (#26 part A); it stood up for 0.6 s before it backed off,
  and the demo's has no crouch.
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

## Held by a script

A character a cutscene holds (119, the script's start, a 105 walk: [cutscenes.md](cutscenes.md))
has no brain meanwhile (char+0x834 bit 0): its walk stops, and it takes its goal up again once
let go. The grenade lesson holds the first patrol's two while it plays.

## Test hooks

* `BF_BRAIN_LOG=1`: each brain at its spawn (home, its pace there, sight, field of view,
  accuracy, gun bonus, personality), its base goal and the draw, its patrol route (iD, points,
  where it starts), each point reached, home reached or given up short of (and how far), each
  goal's start, refusal and end (#336), each contact seen, its target, contacts forgotten (and why), a shooter it learns of,
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
  (not following) and each test map NPC set to fight: its goal (ATTACK_FALLBACK and MOVE_BACK
  since #383), where it is and its facing (degrees, the demo's yaw), its own point, its route's
  entry, its target and how far, its pace, and the motion it plays ([nav.md](nav.md), test hooks).
* `BF_TEST_MOVE_BACK=<s>[,<name>[,<x>,<z>]]` (#383): at s seconds a level character with a brain (not
  following) whose name contains `name` (any when empty), the nearest to (x, z) when they're given,
  starts GOAL_MOVE_BACK, which nothing draws yet; with a target in sight it backs off facing and
  firing at it. `BF_BRAIN_LOG=1` also logs the post, the draw, the start with its route (or why it
  was refused) and the end (and why); `BF_ROUTE_LOG=1` the route's points.
* The tutorial's panels (#224) stop the clock until closed: `BF_TEST_PANEL=a,1` closes each
  after 1 s (in the first patrol's run below the WEAPONS panel comes up at ~17 s).
* `BF_COVER_LOG=1` (#384): the cover goals' events: each pick (the point, how far, from the
  target, its cone and bits), DEBUG1's halves with FUN_00036970's numbers (the hit points' share,
  its own threat at 25 m, its target's, the sum), the states, the peeks, the exits, the hits that
  blow the cover. `=2` also why each cover point the flood met was passed over, every retry of a
  DEBUG1 waiting in ATTACK_VARY, and a row every half second of each cover goal (`cover t=...`:
  its state, point, how far off it, the hold, the time at the point, the peek and re-pick timers,
  the crouch, the motion, where it is, its facing and how far its target).
* `BF_TEST_TURN=<s>[,<r>]` (#384): at s every level character with a brain within r m (60 by
  default) of the controlled character turns to face it at once: the takes' `!yaw` staging, so
  its sensor can see him.
* `BF_TEST_PUT=<s>,<name>,<x>,<z>[,<heading>[,<hp>]][;...]` (#384): at s the level character whose
  brain is named `name` (as BF_BRAIN_LOG names it) is put on the floor at x, z (its heading in
  degrees, 0 facing -z; its hit points, now and at most): the takes' `!place` and `hp`.

```sh
# the first patrol from its plate: the script, the walk past the camera, the fight
BF_MAP=tutorial BF_TEST_GOTO=44,91,34.4,94.3 BF_TEST_PANEL=a,1 BF_BRAIN_LOG=1 BF_COMBAT_LOG=1 BF_TEST_AIM_HOSTILE=14 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the first patrol walked to and fro, Tex far away at the mine field
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router_spawn_first_patrol,USER1,1" BF_BRAIN_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#336) every spawn router of the tutorial fired, Tex away: the walks home by route, the patrols
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router103,ENTER_DETECTOR,1;trigger_router38,ENTER_DETECTOR,1;trigger_router50,ENTER_DETECTOR,1;trigger_router22,ENTER_DETECTOR,1;trigger_router473,ENTER_DETECTOR,1;trigger_router_spawn_first_patrol,USER1,1;h_020415dc,ENTER_DETECTOR,1" BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#336) e23 staged as todo/132-mutants' F1: Tex warped in once the four live mutants are home
BF_MAP=e23 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 "BF_TEST_WARP=25,me,-57.7,64.8,180" BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_GOAL_WATCH=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) take06's staging: Tex warped 4.2 m from the Ugly's take position at 24 s (the warp sets him on the block there; he drops to the ground), the Ugly put there facing away at 24.5 s and turned to him at 25.5 s, no squad: its cover at h_157fd721
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="24,me,-63.59,51.01,26" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0" BF_TEST_TURN=25.5,10 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) the same with 500 hit points and Tex's aim helper from 33 s (the other Ugly near Tex given 1 hit point, the helper's first kill): three hits blow the cover
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="24,me,-63.59,51.01,26" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0,500;24.5,h_f5f8f2ca,-51.5,55.5,,1" BF_TEST_TURN=25.5,10 BF_TEST_AIM_HOSTILE=33 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_COMBAT_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) the same with Tex walking off south-east from 37 s: flanked
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-53,31 BF_TEST_GO_AT=37 BF_TEST_WARP="24,me,-63.59,51.01,26" BF_TEST_PUT="24.5,h_eef1a370,-64.12,46.83,0,500;24.5,h_f5f8f2ca,-30,90" BF_TEST_TURN=25.5,10 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#384) take08's staging (F1): the Ugly 83302960 at its take position, the mutants within 20 m turned to Tex
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP="25,me,-57.7,64.8,0" BF_TEST_PUT="25,h_eef1a370,-64.12,46.83" BF_TEST_TURN=25.5,20 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_COVER_LOG=1 BF_NO_FADE=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the fallback on e23: Tex warped 2 m north of the third Ugly (home (-71.6, 46.2), facing north
# at 20 s, so he's inside its 60 degrees) at 22 s; the squad off, or it joins in
BF_MAP=e23 BF_NO_SQUAD=1 BF_TEST_GOTO=-56.5,101.8,-56.5,101.8 BF_TEST_WARP=22,me,-71.6,44.2,180 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_GOAL_WATCH=0.25 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the tutorial's wall sentry (TUTORIAL_terrorist_5, a guard personality) with Tex 2 m from its post
BF_MAP=tutorial BF_NO_SQUAD=1 BF_TEST_GOTO=89,-55,89,-55 BF_TEST_WARP=8,me,-49.6,-10.0,270 BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_ROUTE_LOG=1 BF_GOAL_WATCH=0.5 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# (#383) the test map: a Filthy Mutant set to fight 2 m from Tex falls back straight, firing
BF_TEST_GOTO=4.5,30,4.5,30 BF_TEST_NPC="0.5,h_0b5f6e21,enemy,fight,4.5,28" BF_TEST_HEALTH=5000 BF_BRAIN_LOG=1 BF_GOAL_WATCH=0.5 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play -- --test
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
  first hit came 14.2 s after the script's start, the demo's 17.2 s. The second stops right
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
  and 2.3 s); 29, home, saw no one in 40 s. 30 did too once the tutorial's own hinters were in
  (#338: it ran on e01's shapes before, and saw Tex at 26.9 m on its way, stopped and fired, -1.7 a
  hit, and died to the aim helper); it is home at 2.53 s, where the demo's sight rays stop at the
  hinter band h_ee99c575 (body 64) that its line to the plate crosses twice (`Arena::ray` takes
  a hinter as any blocker: which blockers the game's sight ray leaves out isn't traced, #298). In
  take08 nobody fired on Tex on the plate: his first hits (-1.5 to -2.8) came 35 s after the
  spawn, from across the pit, once he'd walked on to (46.3, -25.9). Who sees whom there is
  #197's (the game's ray between the two characters' points, FUN_0004cea0, isn't the demo's eye
  to chest).
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
* **#384, the cover** (muted, in the background window, `BF_NO_FADE=1`; the runs are the test hooks'
  above; the fix round's, after the pick measured from the hips): take06's staging (Tex warped in
  first: the warp sets him on top of the block at that spot, the first floor it finds, and he drops
  to the ground in about half a second; warped in the frame the Ugly first looks, he was still up
  there, 5.7 m from h_eed0869f, and the Ugly took that point): the Ugly sees Tex at 3.9 m, refuses
  h_eed0869f (Tex 3.9 m from it, under 5), takes h_157fd721 (DEBUG1 in cover: its
  threat 0.33, Tex's 1.50), is there 4.5 s later and crouches, 13.5 m from Tex, facing him and
  firing; at 10 s it finds no other point and dodges left out of its home area, comes back by
  GOAL_PATH_TOWARD_HOME and takes the point again 2.9 s later, then steps out left the next times,
  every ~16 s. The three hits: at 500 hit points, hit for 97.3, 97.7 and 109.9 at the point, the
  cover is blown on the third (34.40 s, blacklisted) and DEBUG1 runs ATTACK_VARY, which fires 13.4 m
  from Tex until it's past its home area's edge (35.07 s, 1 m on from the point) and walks home.
  Flanked: with Tex walking off from 37 s it's flanked at its point at 38.60 s, done, ATTACK_VARY.
  take08's staging is "Cover" above: crouched at h_eed0869f 1.0 s after it saw Tex, 18.8 m from him
  (the first round, from the feet, sent it to h_157fd721 9.5 m off). The first patrol at take07's
  spot (`BF_TEST_GOTO=44,91,34.4,94.3`, Tex on the plate 8-9 m above them, the aim helper from 14
  s): TUTORIAL_terrorist_2, shot at 37.5 m, takes h_1772d797 (13.0 m off, 42.8 m from Tex) and is
  there at 23.8 s; _18 finds no point facing Tex (h_1e75e701 now just outside its 30-degree half
  cone by Tex's hips, h_1772d797 claimed) and approaches to 26.1 m as main's does; Tex's aim helper
  kills both by 25 s. The first round's had _2 at h_1e75e701 and _18 at h_1772d797. Not checked
  against the tutorial's footage. Against main's build (2138fb3), the first round: the tutorial's
  routes (every spawn router fired) and m09_a's start give the same brain log line for line; the
  test map's NPC fight draws the same goals (DEBUG1 for both, no cover for an NPC) and tells the
  same story, its hits' rolls differing between runs of either build.
* **#383, the fallback** (`BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures; the numbers and take07's
  comparison are in [nav.md](nav.md) "Verified"): on e23 the third Ugly with Tex 2 m off posts
  EVT_TGT_TOO_CLOSE at once, starts ATTACK_FALLBACK (a one-entry route of the link walk, 9.1 m
  from its own point), backs off facing him and firing, BACKSTEP at 1.84 m/s (take07's 1.82-1.83),
  and is done 8.0 m from him after 3.5 s; the tutorial's wall sentry backs off along its walkway
  at 4.4 m/s (BACKPEDAL), is taken home by PATH_TOWARD_HOME and falls back again every 4 s (the review
  round's run: posts at 8.07, 11.47, 15.47 and 19.47 s, each fallback done 8.0-8.1 m off at 9.73, 13.73,
  17.73 and 21.73 s, as before); a test map NPC with Tex walking after it (`BF_TEST_WALK=1`, 2 m off)
  posts once, holds the tests that pass 1.5 s apart while its walk runs (its clock 1.47 and 3.00 s:
  "one is queued while its ATTACK_FALLBACK walks") and runs its 7 m straight to the route's end at
  4.20 s, then posts again at 4.53 s: a walk of 4.1-4.2 s a post, posts at 0.0, 4.5, 9.1, 13.7 and
  18.3 s (before the hold it re-posted every 1.53 s and started the walk again each time); on e23 a
  second warp of Tex 1.55 m from the third Ugly in the middle of its walk (23.45 s, its timer out)
  gives the same "not posted" at 23.47 s and the walk goes on to its route's end at 25.80 s;
  `BF_TEST_MOVE_BACK` walks the Ugly's 7 m behind it facing Tex, done at the route's end. The
  first patrol (`BF_BRAIN_LOG`), the tutorial's routes recipe, sdm_e34's autopilot (298 of 300
  frames) and e23's F1 are unchanged.

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
  reach and the ray clear: `Sensor::sees_at`); FUN_0002e330's 1.5 s lost-sight time for a target
  that is itself in cover or dodging (a level character fighting another); the timed re-pick's gate
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
* Of the fallback (#383): ATTACK_COVER, the draw's second goal; the
  Supermutant's pound and the Shadoon's rocks, which its entry draws first; the crouch an enemy
  stands up from; the line fe56cd97; MOVE_BACK's own draw (ATTACK_VARY's RUN_BACK / WALK_BACK, part
  C); a squad type's walk (it has no drive); the 85 that stays queued under a preemption and starts
  the fallback again when it ends ("Not here" above).
* Of the goals on routes (#336): the follow-ups in [nav.md](nav.md)'s "Not done" (e40's patroller
  called home each round of a chain that leaves its 15 m area, terrorist 6's pauses at two corners
  on its way home, e23's ledge Ugly stepping off its ledge on its way home).
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
