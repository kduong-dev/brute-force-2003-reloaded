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
`common/ai/personality.xmb` (`bf/personality.rs`). Two things are the demo's choices: a walk
given up where it's stuck, and an AI holding fire while a friend stands in its line of fire
(below).

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
  h_f27d26c9 (the goal's +0x94), the speed mode FUN_0004d970 takes while the goal is on: 0 and 2
  a walk (the stick at half), 3 a run, 4 the dash, and 1 the stick at 0.01, read as staying put
  (the tutorial's 31 and 32, who spawn on their homes).
* The trigger's h_f63b86a2 / h_062fbaf0 (the home area's radii, at least 7: brain+0x214 /
  +0x218, for EVT_PAST_HOME_AREA) are read by the game, not used here.

The tutorial's: the opening's wall sentry, TUTORIAL_terrorist_5, walks 10.5 m west along the
walkway to wpoint1493 as the level starts (its trigger at FIRST_UPDATE); the base's patroller,
TUTORIAL_terrorist_6, is sent 55 m into the base to wpoint1123 before it patrols (the demo's
straight walk stalls just inside the gate: "Not done"); the lava yard's
29 and 30, the second building's 3 and 17, the base's 7, 9, 13 and 21 run home (h_f27d26c9 3);
the rest walk or are home already.

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

* **Seen**: a living hostile (another team) within its sight distance x 1.15 once it's a contact
  (FUN_000481c0) x the hostile's visibility (a stealthed Hawk's near 0); within the placement's
  `field-of-view` (h_1b2327a0) either way of the way it looks (its aim while it fires, else its
  facing); with nothing of the world between its eyes and the chest (the squad's ray, as
  play.rs `nearest_hostile` casts it; the game's pair table casts its ray between the two
  characters' FUN_0004cea0 points, FUN_00057a30, not matched here). The sight distance is the
  type's `vision-range` (the Red Hand 75 m, the marines 80), or the placement's own `vision` when
  its h_11cddd9f says so (the sniper lesson's target: 10 m). The field of view: 60 degrees the Red
  Hand, 90 the marines. The gun's range plays no part in sight.
* **Contacts**: a new sighting becomes one; one unseen 15 s is forgotten, a dead or vanished one
  at once, a stealthed one (visibility 0.2 or under) at once.
* **The target**: the contact it fires at, kept while it's been seen in the last 3.75 s
  (EVT_TGT_LOST_SIGHT, FUN_0002e330; 10 s, FUN_0004a780), then the nearest seen.

(`BF_SIGHT_LOG=1` still logs a brain finding or losing its target, as the squad's.)

## Alert and the fight

With a contact known the brain fights where it stands:

* its walk stops (the patrol is started again from the nearest point once it's over);
* it faces the target and fires at it while the target is seen and within its gun's reach (the
  shot's own: the MK-ASLT's 75 m), in the AI's bursts (play.rs `ai_fire`); unseen, it faces the
  target's last place, the gun raised (the turn toward a known contact, FUN_0002f960);
* **a friend in the line of fire** (the demo's choice): it holds fire while one of its own side
  stands within 0.9 m of the line from its eyes to where this frame's shots go (the chest, or the
  miss its aim drew), between that one's feet and its top, and keeps its aim on the target
  meanwhile. The game's AI fires when it sees its target and its gun is on it (FUN_00024f80,
  FUN_0004dc10: no test for friends found); a shot that hits one of the shooter's squad does 0.7
  of its damage and posts EVT_SHOT_BLOCKED_BY_FRIEND (34; FUN_0022c4d0 with FUN_0004b9c0), whose
  goals move the AI. Without the hold the first patrol's second terrorist, stopped right behind
  the first, shot it in the back; with it at 0.6 m one marine still hit the other, 6 m down his
  line of fire, in the arm. Moving for a clear line (EVT_SHOT_BLOCKAGE's
  GOAL_REPOSITION_LOS_TO_TGT, h_0bb586cf's GOAL_ATTACK_REPOS) is #26's;
* once it knows no contact it takes up its base goal again, home first if a shot sent it there.

Its personality's EVT_ENEMY_SEEN draws the attack goal (GOAL_DEBUG1 for `default`'s line,
ATTACK_GUARD / ATTACK_GUARD_CROUCH for h_0bb586cf, ATTACK_COVER for h_eed3a76a) and posts
EVT_USER0 (GOAL_PURSUE) and EVT_USER1 (GOAL_HIDE): their moves (pursuit, strafes, crouches,
cover) are #26's; here it stands.

**Shot, or shot at, while it knows no enemy**: a hit (EVT_DAMAGED_NO_ENEMIES, 0x3e756) or a
shot passing within 1.5 m (EVT_SHOT_AT_NO_ENEMIES; the radius is the squad's dodge's, a guess)
posts EVT_USER1, whose entry's quick goal h_f765fc45 adds the shooter to its contacts ("QUICK
GOAL!!! QG ADD OBJ TO CONTACTS being set here", the data's note on every Red Hand
personality), and EVT_RETURN_HOME. So a terrorist shot from behind turns on its shooter, fires
once it sees it, and goes home after the fight. A brain that knows an enemy registers its
attacker itself (0x3ea02). The hit's reaction (the involuntary KNOCKDOWN / STUN draw) is
play_enemies.rs's, which now counts a contact as an enemy known.

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
  says so, and lifts a miss by brain+0x3c4 bit 13: neither is read here.)
* **The gun's spread** takes the holder's bonus (FUN_00222b30: h_072837bb with h_ec5337fb, else
  -1): the Red Hand's +25 narrows the MK-ASLT's spread ([play.md](play.md), "Shots";
  play_shots.rs).

## The Confed marines

The V-squad's two marines (Stout and Cogburn: trigger_spawn64 and 76, `h_0fa4f510` false, so the
member check keeps them and sets their flags, [triggers.md](triggers.md)) farcast in during
tutorial_farcast_vsquad_short (7.57 / 7.87 s), held by its 119s until it ends; then they follow
Tex with the squad AI's wingman follow (`squad_ai`: places 6-11 m ahead of the leader, set off 3 m
from it, dash when far behind, kneel or look about while he stands), from the formation places
after the squad's own (Tex alone in the tutorial: the first two). They're on the player's team (0,
their placements' alignment-type), see by the sensor above (90 degrees either way of their
facing), fire at what they see with their aim (above), and the Red Hand fire at them. Not done:
switching to them (the d-pad), their portraits, the squad commands.

Their walk to their h_131de144 points: the spawn makes those points their homes (wpoint1380 and
wpoint1006, 0.5 and 0.9 m from their pads), but without a home-point nothing sends them there,
and a wingman follows its leader. take11's watch log agrees: the two stood on their spawn points
to 0.1 m (-16.9, -75.4) and (-21.1, -76.3) from the farcast (29.3 s) until Tex moved off at
156.9 s, set off 1.6-2.2 s after him and kept 8-40 m behind him in bursts of a run, and stopped
9-10 m behind him when he stopped. The demo's follow sets off once a marine's place is 3 m off,
and those places are the squad's, ahead of him: the game's wingman follow (FUN_00020b10, a
state machine of its own) isn't read. The script's 105 walks to those points
(0.26 s) name the marines before they're spawned and do nothing ([cutscenes.md](cutscenes.md)).

## Held by a script

A character a cutscene holds (119, the script's start, a 105 walk: [cutscenes.md](cutscenes.md))
has no brain meanwhile (char+0x834 bit 0): its walk stops, and it takes its goal up again once
let go. The grenade lesson holds the first patrol's two while it plays.

## Test hooks

* `BF_BRAIN_LOG=1`: each brain at its spawn (home, its pace there, sight, field of view,
  accuracy, gun bonus, personality), its base goal and the draw, its patrol route (iD, points,
  where it starts), each point reached or stalled, home reached or stalled short of (and how
  far), each contact seen, its target, contacts forgotten (and why), a shooter it learns of,
  holding fire for a friend, stopping to fight. The copies are named as the triggers' log names
  them.
* `BF_TEST_AIM_HOSTILE=<s>`: from that time the controlled character's camera turns onto the
  nearest living hostile it can see (within 120 m, nothing of the world in between: its chest,
  else its head over cover) and it aims and fires at it: the takes' aim helper
  (todo/28-tutorial/notes.md), to stage a fight. With `BF_TEST_GOTO` it walks on meanwhile,
  toward the hostile while it aims (the walk follows the camera).

```sh
# the first patrol from its plate: the script, the walk past the camera, the fight
BF_MAP=tutorial BF_TEST_GOTO=44,91,34.4,94.3 BF_BRAIN_LOG=1 BF_COMBAT_LOG=1 BF_TEST_AIM_HOSTILE=14 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# the first patrol walked to and fro, Tex far away at the mine field
BF_MAP=tutorial BF_TEST_GOTO=89,-55,89,-55 "BF_TRIGGER_FIRE=trigger_router_spawn_first_patrol,USER1,1" BF_BRAIN_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
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
  6.5 s into the script (take07: ~5.7 s). With Tex standing on the plate, the leader sees him at
  35.5 m 15.3 s after the spawn, stops and fires (-1.3 to -1.6 HP a hit; take07 -1.1 to -1.8);
  the second, stopped behind it, holds its fire. Take07's first hit came 14.2 s after the
  script's start, the demo's 17.2 s. With the aim helper (`BF_TEST_AIM_HOSTILE=14`) Tex kills
  both: +$60 each.
* **The patrol walked out** (Tex away at the mine field, the router fired): the chain to
  patrol_point11 and back, turned round at each end, ~66 s a round trip, on and on.
* **The marines** (`BF_TEST_GOTO=-15.6,-87.4,-60,-75 BF_TEST_GO_AT=14`, take11): they farcast
  in at 7.57 and 7.87 s of tutorial_farcast_vsquad_short, held until it ends, stand on their
  spawn points until Tex moves, then run with him (`squad_ai`) and, when he stops at
  (-60, -75), fight terrorists 31 and 32 at 20-40 m (they see and shoot the marines too): both
  terrorists die to the marines' fire, 13 and 19 s after they spawn (+$60 each), without a hit
  between the marines (the hold; before it, one marine hit the other in the arm twice).
* **The lava yard** (`BF_TEST_GOTO=49.5,-15.5,62,-34 BF_TEST_GO_AT=999`, Tex on the plate
  h_f42fa5b8, take08): the plate's router spawns 29 and 30, who run home across the pit (1.5
  and 2.3 s); 30 sees Tex at 26.9 m on its way, stops and fires (-1.7 a hit), and dies to the aim
  helper; 29, home, saw no one in 40 s. In take08 nobody fired on Tex on the plate: his first
  hits (-1.5 to -2.8) came 35 s after the spawn, from across the pit, once he'd walked on to
  (46.3, -25.9). Who sees whom there is #197's (the game's ray between the two characters'
  points, FUN_0004cea0, isn't the demo's eye to chest).
* **The base** (staged: its routers fired with `BF_TRIGGER_FIRE`, Tex standing at the gate, in
  the plaza and at the third building with the aim helper): terrorists 0, 6, 8, 9, 10, 11, 13 and
  15 were killed, most having seen and fired on Tex (the sniper 11 hit him for 8.6-10 from 51 m
  and killed him once, Tex standing at the third building); every death paid +$60 and counted on
  h_e8ea7ba2 (router99's twelve). The wall sentry (on its walkway), the sniper 7 and the third
  building's 12 and 21 (inside its walls) were out of the aim helper's sight from the spots
  tried, so no run cleared all twelve (take12's clear had the marines and healing).
* **Every spawn at once** (Tex away, every router fired): who arrives home and who stalls, as
  "Not done" lists; the paces from h_f27d26c9 as logged.
* **No regressions** (main's build, e5d2f74, against this one): e40's first 40 s (the same log,
  no frame differing by more than 24 levels), sdm_e34's autopilot (the same log, 296 of 300
  frames the same: the first four differ between two runs of one build too), the test map's NPC
  fight of testtools.md (the same combat log, 222 of 225 frames the same and the other three by
  1-2 pixels). m09_a's four Shrikes now run home at its start (two Hounds stall 23 and 29 m
  short, off their ledge): seen in the opening shot, where they stood before.

## Not done

* #26: the attack goals' moves (pursuit, strafes, crouches, cover, repositioning for a clear
  line), calling for help, investigating noises and bodies, the idle variations (pauses, cautious
  looks, crouches, walks), the home area (EVT_PAST_HOME_AREA), grenades.
* **Routes.** The walks are straight lines, given up where they stall (not 0.1 m closer in 1 s:
  a guess): a walk home stops there, a patrol goes on to its next point. The game routes over
  the level's world points: each `<world-point>` lists its neighbours in h_e3439383 (946 points
  and 5982 links in the tutorial), and a waypoint-trigger (h_10ecb459) marks one valid or not;
  the route search itself isn't traced. So in the tutorial (`BF_BRAIN_LOG`, every spawn fired
  with Tex away): the base's patroller, TUTORIAL_terrorist_6, stalls 48 m short of its home just
  inside the gate and patrols from there, its first legs stalling until it's on the chain; the
  third building's 21 and 13 stall against its walls 12 and 31 m short of theirs (in take12 they
  come out), 17 stalls 6.5 m short, and 9 stays behind the base building's door, shut there (its
  router fired without the plate that opens the door). The other walks home arrive.
* The patrol's 4 s timers in FUN_0002b490 (brain+0x1e0 / +0x1e8, and a 0x1f / 0x20 call on the
  character for a type with type+0x1a8 bit 15) aren't read.
* FUN_0004d970's speed mode can be lowered by FUN_0004d840 and the brain's +0x1a8 record: not
  read; the demo walks a patrol at mode 2 and takes h_f27d26c9's mode home as it is.
* A shot's EVT_SHOT_AT radius is the squad's 1.5 m (a guess); grenades and blasts don't tell a
  brain who threw them.
* A warp's FUN_0004c5e0 for a living character (it checks the home point again, FUN_0014f7a0,
  and goes through a world list at [0x469ab0]+0x1348) isn't done.
* The marines' follow is the squad's (`squad_ai`); the game's wingman follow (FUN_00020b10, a
  state machine with its own distances) trailed Tex in take11 and stopped 9-10 m behind him.
