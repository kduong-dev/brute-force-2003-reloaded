*Part of [brain/](README.md), the former page `brain.md`.*

## Cover (#384)

`src/bin/play_cover.rs` (`Brain::cover`), from `think`: GOAL_ATTACK_COVER (19) and GOAL_DEBUG1's
(120) cover half, part A of #26. Read from ATTACK_COVER's start 0x2f8e0 (the pick FUN_0002f270),
update FUN_0002fd40, event hook FUN_000313e0 and end 0x2f900, and DEBUG1's start 0x36a70, update
0x36b70, hook 0x36ca0 and end 0x36b50 ([code-map.md](../code-map/ai.md), "AI");
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
  There's no sight test. Then a route to it ([nav.md](../nav/enemies-on-routes.md) "The enemies on routes": walked at a
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
  light hit while it holds cover), EVT_SHOT_AT, the two shot blockages, TGT_TOO_CLOSE (the post
  is #383's, "The fallback" below: `fallback::post` asks `cover::takes`) and, but out of cover,
  TGT_ENTRENCHED from the personality.
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
