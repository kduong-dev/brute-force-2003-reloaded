*Part of [brain/](README.md), the former page `brain.md`.*

## Alert and the fight

With a contact known the brain fights: its attack goal's approach and pursuit walk routes
("Its goals on routes" below); where those stand, it fights where it is:

* its walk stops (the patrol is started again from the nearest point once it's over);
* it faces the target and fires at it while the target is seen, within its gun's reach (the
  shot's own: the MK-ASLT's 75 m) and **its muzzle corridor to the target is clear** ("The muzzle
  corridor" above), in the AI's bursts (play.rs `ai_fire`); with the corridor blocked it faces the
  target, gun up, and holds fire; unseen, it faces the target's last place, the gun raised (the
  turn toward a known contact, FUN_0002f960);
* **a friend in the line of fire** holds it, since #244. The head-to-head sight ray leaves
  characters out (the pair table's query, FUN_00057730, skips the object types in its mask
  0x415a1018, characters (type 3) among them), but the muzzle corridor the attack check reads
  next does not (mask 0x415a1010): a third character in the line clears its bit and the attack
  check refuses with reason 4. A terrorist stopped behind its buddy holds fire until one of them
  moves, and after 3.75 s of it drops the target (EVT_SHOT_BLOCKAGE). Before #244 the demo fired
  into the friend, who took the shot at the own side's 0.7 of the damage (play_damage.rs,
  FUN_0004b9c0) without taking it as an attack (no EVT_DAMAGED_NO_ENEMIES from a friend, 0x3e8fc);
  a friend can still take a round the corridor didn't see (a shot that strays, the gun's spread,
  a friend who steps in). The game also posts EVT_SHOT_BLOCKED_BY_FRIEND (34, FUN_0022c4d0) from
  the instant ray of a squad AI at a squadmate, whose goals reposition the shooter; that move
  (with EVT_SHOT_BLOCKAGE's GOAL_REPOSITION_LOS_TO_TGT and h_0bb586cf's GOAL_ATTACK_REPOS) is
  #26's;
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
