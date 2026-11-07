*Part of [grenades/](README.md), the former page `grenades.md`.*

## Roller (`src/bin/play_roller.rs`)

"Round and intelligent - the rolling bomb seeks enemies when deployed." The squad's h_fba7c475
(m09_b's four pickups are the twin h_15f9c51a): set down as the Sentry is, its `timer` 25 s, its
explosion h_f213e5b1 (Damage 117-156, type 10, radius 5, core; effect h_013b3ca8: the Frag's six
ALE effects, light_explosion and h_065168c9; the Frag's scorch). What it does between is its item
set's, IFSET_ROLLING_BOMB (function-type 13, vtable 0x39ba60; every function is in
[code-map.md](../code-map/README.md); the leads and the takes are `todo/79-roller-grenade/`):
* **The let-go** (0x148640 -> FUN_00148720) makes its seek helper: a point h_11884f2e (50) m along
  the placer's horizontal aim (0x10fbf0; the demo's aim frame, the camera's turn) at the hand's
  height, the first check due in 1.0 s, no target, the placer's team (+0x1c0) as the bomb's.
  Measured live in the takes: the point 50.00 m along the aim, at the hand's height.
* **Every frame** once it's out of the hand (0x148690 -> FUN_001481c0, FUN_00148220): the check
  timer runs down; a target out of sight has 5 s before it's dropped. **A check every 0.5 s**
  (the first 1.0 s after the let-go): with no living target, or every 10th check (h_0a811e94), a
  **target search** (FUN_001483e0): the nearest living character of a team the team table makes
  hostile to the bomb's (0x3ffd80: never the squad for the squad's), not the one who set it down,
  whose hips (the character's node, `missile::root_point`: the feet + its clip's root-height,
  1.12 m on the Shrike Heavy; the takes' point is the enemy's record + 1.1 m) are within 50 m (3D)
  and in sight; a held target is only replaced by one within half its distance. No cone. Then the
  target in sight moves the point to its hips; out of sight, the point stays.
* **The sight line** (FUN_00148100: the standard line-of-sight filter 0x39cc18, skipping the ball
  and the target) meets the terrain and the placed objects' collision only: not the level's
  blockers or hinters, characters, corpses, items, grenades, effects or liquids (`clear`: the
  arena's `ray_shot`, which leaves the blockers out; on the flat test floor its ground, pillars
  and pieces), from the ball's middle to the hips. Cast from the takes' logged positions (the
  ball's node and the enemy's hips at each of the 116 checks of takes 05-18, `BF_ROLLER_RAYS`),
  the demo's ray agrees with every one: takes 05 (h_09a6856d), 08 (h_fc0ef12e, h_e04e5a0e,
  h_09a6856d, h_04e79fb8, h_fbdcd828), 11 and 12 (h_04e79fb8, the 10 m object near the enemy:
  not the terrain, the leads' open question) and the gate panel of 17 / 18 (h_0792d16a) block
  the misses, every find is clear, and take12's one out-of-sight check meets h_f455bbb6.
* **It goes off** when its point is within 2.5 m across (half the explosion's radius) and between
  1.5 m below and 2 m above the ball (exclusive): the fuse is set to 0 and armed (+0x1d4, +0x1dc),
  so the blast is this frame's, through the fuse's cutscene gate. The rule is on the point, not
  on anyone: an enemy unseen behind a wall 1.7-2.1 m away doesn't set it off (take18). With no
  target it rolls on and goes off at its 25 s fuse, counted from its first contact.
* **How it rolls** is Ipion's: the game spins the ball toward the point every frame with the same
  angular push (FUN_002bad80 with 4.0, about the level axis square to the way to the point) and
  the ground's friction turns the spin into the roll, none of it in the game's code. The demo's is
  **a fit** to the takes' 20 Hz logs of the ball (`DRIVE_PUSH` and the constants after it). The
  logs show the ball **hopping**: between ground contacts every ~0.6 s (0.4-0.85 s, 0.25-0.8 m
  high) it flies straight (its speed along the ground holds, gravity 9.8), and only a contact
  changes its way. So the demo keeps a drive velocity for the spin, growing toward the point at
  11.5 m/s^2 (times the horizontal part of the unit vector to it, as the push axis is) and held
  back at 1.35 /s along the way to the point (a ceiling of 8.5 m/s) and 1.6 /s across it; each
  contact takes the ball's velocity along the ground (in the ground's plane) 0.8 of the way to it,
  and it leaves again at 2.94 m/s along the ground's normal. A ball that isn't coming down into the
  ground (the ground rose under it) is only set on it, so slopes slow it. It turns with its drive
  (the drive's speed over its radius, 0.19 m: its archetype's sphere, which it rests and meets
  walls by).
* **Walls** stop it dead (no bounce): its move along the ground is swept with a ray first (a thin
  gate leaf let the push-out send it through), and it stays against the wall, pressed toward
  its point (takes 17 / 18).
* **The level's own Rollers** are left out for now, pending #316: m09_a sets one down live as it
  loads (h_0bde6982, item-alignment 1), which the item handler 0x14a0a0 lets go with no placer (its
  point 50 m back along its own frame's third column), but its side is in doubt: FUN_00148720
  copies the item's team into the helper once, at the let-go, and if 0x14a0a0 lets it go before
  `item-alignment` is applied, the helper's team stays -1 and it never seeks. So `place_mines`
  skips it and the level's own model of it stands as before. The other levels' (e16, e23, e34,
  m05_b, m09_x, m14_a, m14_c, m14_d) are spawn-trigger templates, out of the world until a copy
  is made, which the demo's spawn copies don't do for items.

`BF_ROLLER_LOG=1` prints the let-go (the point and its heading in the logs' atan2(dz, dx)), every
check (the search's candidates with their distance and what blocks their sight line, the target
and the point), each ground contact (the speed along the ground before and after, the drive) and
the proximity blast; `=2` also the ball, its drive and its point every frame.
`BF_ROLLER_RAYS=<file>` casts the sight line once the map is up for each line
`x1 y1 z1 x2 y2 z2 [label]` of the file (`#` comments) and prints what it meets first, how far
along and where, with a placed object's index and type.

Verified on e34 against the takes, each staged from its log (`-- --test BF_MAP=e34 BF_NO_SQUAD=1
BF_TEST_GOTO=-49.49,25.25,48.64,6.05 BF_TEST_GO_AT=100 BF_TEST_GRENADE_TYPE=Roller
BF_TEST_THROW=1.0,0.1 BF_TEST_INSTANT_KILL=0 BF_ROLLER_LOG=2 BF_CAPTURE=<dir> BF_CAPTURE_FPS=30`,
the enemy as `BF_TEST_NPC=0.3,h_1c5780f8,enemy,frozen,<x>,<z>,<y>`: the take's A50 Heavy, the
Soldier h_e9a41520; the gate from `BF_TEST_GOTO=-55.20,35.28,-56.40,135.28`): the let-go's point
lies at -11.1 degrees (the takes' -11.1). Straight (take09's staging, no enemy): the hops leave at
2.97, 5.72, 7.25, 7.91 m/s (the take: 2.95, 6.08, 7.23, 7.93), the 0.5 s windows from the let-go
0.4, 3.0, 5.6, 7.0, 7.7, 8.1, 8.3 m/s (the spec: 0.5, 3.1, 5.6, 7.0, 7.5, 8.4), the first check
0.97 s after the let-go; it climbs the rock ~38 m out and wanders on it, as the take's did (the
demo's higher up), and goes off 25.03 s after its first contact. The blasts (the let-go to the
blast; the take's in brackets): take10 1.1 s (0.94), take11 2.7 (2.97: unseen through h_04e79fb8
until 3.1 m, as the take), take12 5.67 (6.05: found at the third check, 31.5 m, the take's 30.9;
lost once), take14 2.17 (2.38; Flint never a target), take15 2.77 on A, the nearer (3.16), take16
2.90 on B, the nearer (3.78), take06 3.73 (4.07), take07 4.07 (4.48); takes 08 and 13 never find
one. The wide turns: take06's 109 degrees, the way first off by 20 degrees 0.6 s after the find
(0.7), within 20 degrees of the bearing 1.8 s after (1.2), slowing to 3.5 m/s (0.9); take07's
132, 0.6 s (0.5), 1.3 s (1.0), 2.4 m/s (2.5). Takes 17 / 18: stopped at the panel at z 41.13
(the take's 41.1) and stayed, no target, no blast (path within 0.9 / 1.7 m rms of the take's).
take10's frames (`BF_CAPTURE_EVERY=2`, frames 30-94) beside the take's (2.0-4.6 s): the same
view, the ball let go and hopping off toward the enemy, then a white flash, the red damage tint
and the yellow-white fireball; Tex takes 50 HP in the take where the demo's thrower share gives
less (**Damage**: #112's).
* Not matched or untested:
  * take05's 42-degree target: the take's first check missed it (h_09a6856d) and the ball
    circled it for 4 s (the blast at 5.9 s); the demo's ball, a little to the left of the take's
    at that check, sees it at once and goes off at 2.77 s. In the fit's own simulation, fed the
    take's logged points, the ball circles it for 2.5 s and goes off at 5.3 s, but the circle is
    a knife edge of the fit (1.6 /s across circles it, 1.65 doesn't).
  * The game's ball turns its first hops 5-13 degrees off the aim (the takes scatter -21 to +2
    degrees round the -11 point); the demo's go straight at the point. The demo's hop is always
    2.94 m/s (the takes' 1.9-4.2); a wide turn slows it less (3.5 m/s against take06's 0.9).
  * On the rock (take09) the take's ball rolled back from y 33.8; the demo's climbs to 37 and
    wanders there.
  * At the gate (take17) both keep hopping against the panel, but the take's shuffles 3 m along
    it (x -54.2 to -57.1) and comes off it up to 1 m; the demo's hops in place, its drive straight
    into the wall (frames 90-270 of the staging with `BF_VIEW_YAW=0.7`).
  * The set-down hand and point (above, **Placing**). A Roller shot or caught in a blast (1 hp),
    a target on a ledge or in a pit, a walking target, the re-search at the 10th check, the
    half-distance switch and the 5 s drop (code only: no take reaches them), a squadmate in the
    blast, the fall-off between 2.7 and 5 m: not filmed.
  * The per-material ground puff of its explosion (h_0cbcb8f2: dirt h_e919bf9e ...): no grenade's
    is drawn yet.
  * Its blast's damage to enemies (the tester's runs of the stagings): the A50 Heavy survives a
    Roller going off 2.4-2.6 m from it with 10-22 HP, where 8 of the 9 takes' blasts killed it. The
    shared rule (`hurt_by_blast`: Damage max falling to nothing at the radius) gives about half
    there (156 x 0.5 x 1.4 = 109); the game's (FUN_0022c4d0) gives a core blast's whole roll within
    half of r', the radius less the victim's size (117-156 x 1.4 = 164-218): #112's.
  * The sight line starts at the ball's middle, its radius (0.19 m) over the floor at rest, where
    the game's starts at its node, ~0.3 m over the floor (the spec's §4: the logged node 29.3-29.5 on
    ground at 28.9-29.1). A ray grazing a bump is a little likelier to be blocked in the demo.
  * The game's sight ray is also met by fly-blockers (its kind mask 0x58 keeps PID_FLY_BLOCKER,
    kind 8); the demo's arena has none.
  * The wall sweep (play_grenade.rs `fly_grenades`) casts once along the frame's move and keeps the
    first face it meets: when that face is floor-like (a slope under 65 degrees ahead), the frame's
    sweep stops nothing, and a wall just beyond it is caught by the next frame's sweep.
