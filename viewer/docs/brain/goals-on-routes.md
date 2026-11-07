*Part of [brain/](README.md), the former page `brain.md`.*

## Its goals on routes (#336)

The brain's goals walk the level's world points as the game's do: GOAL_RETURN_HOME, GOAL_PATROL's
first leg, the attack goals' approach, GOAL_PURSUE and GOAL_PATH_TOWARD_HOME, on the brain's one
route record, with the squad's stuck handling and steering (`src/bin/play_goals.rs`; how each is
read, and the route machinery, are [nav.md](../nav/enemies-on-routes.md)'s "The enemies on routes").

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
  fire through its approach and first hits him later ([nav.md](../nav/verified.md) "Verified" has the timing
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
