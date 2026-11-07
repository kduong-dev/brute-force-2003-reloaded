*Part of [grenades/](README.md), the former page `grenades.md`.*

## Sentry (`src/bin/play_sentry.rs`)

The proximity mine (h_e5f1f063, function-type 8 IFSET_PROXIMITY_EXPLOSIVE): set down as the
Roller is, its `timer` 9999 s never runs out; what sets it off is the item set's handlers in
default.xbe (vtable 0x39bc50; the range test FUN_00146cb0, the check FUN_00146e00 and the
on-placed FUN_00147570 have Ghidra's C; the per-frame 0x147620 and the target filter 0x147430
have none and were read in the disassembly):
* Once it's down (on-placed 0x147570; there is no arming delay in the code), it checks every
  0.05-0.15 s (0x147620: a timer passing 0.15 s, restarted at a random 0-0.1 s); the first
  check comes 0.15 s after it's down. A check that says so sets it off at once.
* The check (0x146e00) goes through every living character. A friend - on the mine's team
  (its owner's when it was set down, kept at +0x1c0 by 0x147570), or the thrower - within the
  radius ends it: no blast this time. Anyone else within it is a target: characters on other
  teams always, and on the mine's own team only for a team the team table (0x3ffd80) makes
  hostile to itself (its diagonal: team 7 only). It goes off with a target and no friend within the radius. The radius is the item's
  h_0a811e94 (3 on every Sentry; read into `WeaponDef::proximity_radius`), a 3D sphere round
  the mine (0x146cb0).
* A character's point for it is 0.9 m above the feet: a fit to the recording's enemy brought
  up a slope (set it off with its feet 2.51 m across and 0.63 m above the mine, not at
  2.61 / 0.67; the 3 m sphere puts the point 0.81-1.01 m up). On flat ground that's 2.88 m
  across. The mine's point is its model's middle (the model is 0.14 m tall).
* Characters whose type handles mines and that know of this one count only standing on it:
  within 0.3 m across and 1.5 m up or down (0x147430). A type handles mines when its
  `<combat-skill>`'s hazard list (`<h_0c437951 action>`, the type's bytes +0x294 the filter reads
  at +0x294 + 3) holds 3, the hazard kind a Sentry gives itself (0x147690): Tex's alone
  (`SkillData::hazards`). It knows of every mine but his own squad's: his skill's +0x94 (Tex's
  Berserker, 0x13d8f0) says no for a mine whose owner is in his squad, every other skill's
  (0x280040) yes. So Tex steps round the level's mines (#228) and counts as anyone for the
  squad's own; the test hook's `wise` hostile has the rule too.
* It has 1 hitpoint: a shot (anyone's) whose line passes through its model's bounding sphere
  stops there and sets it off when it arrives. The shots are tested against the mines as they're
  made (play.rs's `update_player`, before friendly fire): the first thing on the line takes the
  shot - a teammate in front of the mine is hit and the mine isn't; a teammate behind it isn't
  hit. Bodies on the ground don't stop shots in the demo (they're shoved and the shot goes on),
  so a mine behind one is still struck, and bodies beyond the mine aren't shoved. The loose
  pickups (play_pickups.rs, after `update_player`) see the shortened shots.
* The squad is one team, so the squad's mines never go off for the squad (as recorded: Tex
  walking or running over his own, standing beside it, Flint stepping onto it at 0.16 m). A
  test hook's hostile, the test map's enemy NPCs ([testtools.md](../testtools/npcs.md)), the levels'
  terrorists, or a shot, sets one off. Nobody can pick one back up.
* **The level's own mines** (#228): an item object of a Sentry type placed with h_1cb7d5f2
  (`Placement::placed`; the tutorial's three `<h_1d1d4f05>` item spawns in the mine field, at
  (92.6, -71.8), (95.3, -82.3), (98.2, -75.8)) is set down and armed as the level loads (its
  attribute handler 0x14a0a0: +0x1bc bit 1, the item set's +0x10), on its `item-alignment`'s
  team (+0x1c0: 1, the enemies') and owned by nobody; the level's own model of it (the one of its
  name, or for an unnamed one the nearest of its type within 1 m) is hidden and the mine drawn
  in its place. It goes off for the squad's Confed marines within its 3 m, but
  Tex only sets it off standing on it (above): take09 has him 0.84 m from the first for 8 s
  (71-79 s) before the disarm. Going off it sends DESTROYED to the triggers that listen to it
  (0x149520 at 0x1496fc): the tutorial's routers 446 and 470, and calculator471, which counts the
  three to end the mine lesson. The same rule arms e16's, e34's and e40's Sentry mines
  (h_f73de83d: 14, 6 and 7) now that `load_kits` makes their kit (#82, **Thermo**); before, the
  demo had no kit for them there and left them out. Not checked against those levels' footage.
* **Its look: red tips, whoever set it down (#300).** The mine's model h_075885e5 (every Sentry
  set down: the squad's h_e5f1f063, the levels' h_f73de83d, the AI's) has a lights geoset on
  its four tips: the wrapper h_0a08f6b5 (stencil 50) round the self-lit material h_f9761b6c,
  untextured, h_e01baa40 0.757 0 0, `blend-mode` 2, `alpha` 1, no scroll. The self-lit shader's
  setup (FUN_0008fd20) binds the default white texture for none and draws it opaque, lit by
  nothing: saturated red, steady (the tutorial's takes 20 and 21: the tips measure (167, 18, 26)
  on average through the video's blur, up to (199, 41, 35); #80's t80_07 / t80_08, Tex's own on
  m03_a: the same). Nothing colours a mine by its team: its on-placed (FUN_00147570) and its
  updates write no effect, light or material, and no code tints a material at run time
  (code-map.md, FUN_00201710). What does differ by who set it down is the HUD: the use ring (blue
  on an enemy's, the disarm; red on your own, refused: FUN_00121860) and the crosshair's lock
  (red on an enemy's only: FUN_000bf6d0), #145's. The blue Sentry the product owner saw is its
  pickup form h_ecfe0d8c (model h_095daa18, lights h_031724e6 -> h_0ed4a3dd 0.098 0.342 0.829),
  which only sdm_e34 (five) and mp8 (two) lay, as spawn-trigger templates: the demo copies no
  grenade pickup yet (pickups.md, "Pickups from spawn-triggers"), and the level viewer draws them
  as it draws placed glow shells, blue and see-through.
  * The demo drew every untextured material of a model plain grey (`scene::material`); now an
    untextured self-lit one is its colour, unlit, blended as FUN_0008fd20 says
    (`level_scene::self_lit`). A scan of the 53 levels' object types and character types: the
    Sentry's tips are the only such parts of the grenades, weapons, missiles and characters;
    two debris pieces (h_fc5e4d5f and h_fd5c98c6, on 9 and 8 levels) carry a dark red one
    (h_0049b1eb, 0.529 0.024 0.008, opaque), drawn so now when a broken object throws them; the
    medkits' crosses are flat glows that play_pickups.rs and level_scene draw their own way, and
    the placed objects' self-lit parts are level_scene's (unchanged: levels.md).
  * Not done: the tips' glare. Their wrapper's stencil 50 is a glare key (#289: the bright pass
    keeps the red's luminance, 0.226, over 50/255), but a bevy standard material writes 1 to the
    picture's alpha; a keyed material would have to follow Hawk's fade and the Vengar view,
    which swap standard materials. The takes show the halo faint at this size.
  * Checked (`BF_MAP=tutorial BF_NO_SQUAD=1 BF_TEST_GOTO=92.6,-68.6,92.6,-80 BF_TEST_GO_AT=100
    BF_CAMERA_PITCH=-0.35`, frame 24: the level's mines red-tipped where main's were grey; and
    Tex's own on m03_a, `BF_MAP=m03_a BF_NO_SQUAD=1 BF_TEST_GOTO=-185.7,-39.1,-185.7,-30
    BF_TEST_GO_AT=100 BF_TEST_GRENADE_TYPE=Sentry BF_TEST_THROW=1.0,0.1 BF_CAMERA_PITCH=-0.4`,
    frame 43: at his left foot with four red tips, as t80_08 shows it).

Its blast is the shared one (h_01f142eb: 62.5-88.5, type 10, radius 4; effect type h_0b69c2f2:
exp-mine + light_explosion, 145f09e5; decal h_f5ccedb0). On sdm_e34 the squad's Sentry is still
h_e5f1f063 (e34 defines it as well as h_f73de83d, whose explosion is the Energy's h_f7d6b42e:
the squad carries h_e5f1f063 wherever it's defined, `SQUAD_GRENADES`). Not matched: the
recording's Tex took 16.5 and 14.8 HP from blasts 6.0 m away, outside the data's 4 m radius;
the demo's thrower is hurt only within the radius (0.2 x 62.5-88.5 = 12.5-17.7 HP), so at 6 m
he takes nothing. Traced so far, not implemented (it would change every grenade's blast, its
own ticket; each function is in [code-map.md](../code-map/README.md)): 0x149520 queues the explosion on
the world, whose update (FUN_000d9a20) makes it a blast object (FUN_0022bfc0) at the grenade:
damage +0x20 a roll of min..max (times the queued factor, 1), radius +0x24, life +0x28 /
+0x30 the Damage's h_04ea9251 (0 here: it deals once), a delay of 0.05-0.25 s (+0x2c), and no
speed: it doesn't move. The world turns one queued explosion into a blast per update. The blast
pool's tick (FUN_00225220) waits out the delay; FUN_00224a90 gathers the bodies within the
radius of that fixed point (characters need a clear ray to it); FUN_00224770 deals each one
through the explosion weapon's damage handler, where FUN_0022c4d0 applies the Damage's
distance rule: h_011cb154 false (the Sentry's) half the roll anywhere inside the radius; true,
the whole roll within half the radius, then (1 - 2 (d / r - 0.5))^2 of it to the edge. The age
taper (FUN_00225060 -> FUN_00223780) needs flag 0x40, which nothing sets, and FUN_00225440
flies the bullet-type 0 projectiles that share the pool, not blasts. So the code
read doesn't explain Tex's hits at 6 m. The demo keeps its shared rule (Damage max falling to
nothing at the radius; the thrower 0.2 x a roll of min..max inside it).

`BF_SENTRY_LOG=1` prints each check with a target within radius + 1.5 m (3D and across
distances to the nearest target and friend, and the result), each shot that strikes a mine and
each mine set off. Test hooks (with `BF_TEST_GOTO`):
* `BF_TEST_HOSTILE=<character>[,<from m>,<to m>,<m/s>][,wise]`: that squadmate (brutus, flint,
  hawk, tex or 0-3) is on another team and stands still out of the squad AI. Once the first
  Sentry is down it's put `from` m (5) beyond it, on the far side from the thrower, waits 1 s,
  then is stepped in toward it at `m/s` (0.25) down to `to` m (0) across; held where it was
  when the mine goes. `wise`: it handles mines and knows this one.
* `BF_TEST_MINE_FRIEND=<character>,<m>[,<s>]`: once the first mine is down, that squadmate
  is held `m` m beside it for `s` s (default for good), then let go to the squad AI; until
  then it follows the squad as usual.
* `BF_TEST_SHOOT_MINE=<s>`: `s` s after the first mine is down the player turns the crosshair
  onto it and fires from the hip.

Verified (test map, `--test BF_TEST_INSTANT_KILL=0 BF_TEST_GRENADE_TYPE=Sentry
BF_SENTRY_LOG=1 BF_COMBAT_LOG=1`, Tex backing away from it with `BF_TEST_GOTO=20,20,20,10
BF_TEST_STRAFE=0,-1 BF_TEST_THROW=1.05,0.1` unless said):
* Trigger distance (`BF_TEST_HOSTILE=flint,4,0,0.5`): stays at 3.05 m (2.93 across), goes off
  at the next check, 2.99 m (2.87 across), with Tex 5.3 m away; Flint, 3.0 m from the blast,
  takes 22 HP and is knocked down. The blast frames show exp-mine's fireball as before.
* A friend near (`BF_TEST_HOSTILE=flint,4,2,1 BF_TEST_MINE_FRIEND=hawk,2,6`): Flint stands at
  2.0 m from +3.2 s, the mine stays while Hawk is at 2.2 m, and goes off at the first check
  with Hawk past 3 m as he walks off (3.02 m; an earlier run, 2.95 m: stays, 4.19 m: goes off).
* A teammate behind the mine on the shot's line (`BF_TEST_SHOOT_MINE=1.5
  BF_TEST_HOSTILE=hawk,4,4,0`: Hawk held 4 m beyond it): the shot strikes the mine 6.5 m out
  and Hawk isn't hit. (A teammate in front of a mine wasn't staged.)
* The thrower near (`BF_TEST_GOTO=20,20,20,20 BF_TEST_THROW=1.5,0.1
  BF_TEST_HOSTILE=flint,4,0.5,1`): Tex at 0.58 m across, Flint walks in to 0.5 m and stands
  there 4 s: it never goes off.
* `wise` (`BF_TEST_HOSTILE=flint,2,0,0.5,wise`): stays at 0.47 m across, goes off at 0.20 m.
* Not matched or untested:
  * The blast's first ~0.2 s is grey-white in the demo where take10's is orange from the start
    (the shared exp-mine rendering, `src/ale_fx.rs`).
  * A mine set down on the move slides ~0.46 m after it lands (the placed grenade keeps the
    hand's speed; no footage of placing on the move; untested against the game).
  * Placing isn't possible crouched (`BF_TEST_CROUCH` with `BF_TEST_THROW` places nothing): the
    shared `can_throw` (the crouch is an action), unchanged from main; whether the game lets
    one place crouched isn't recorded.
* Shot (`BF_TEST_SHOOT_MINE=0.7`): struck 2.3 m out, goes off the same frame; Tex 3.1 m from it
  takes 15.7 HP. From 10 m (`BF_TEST_SHOOT_MINE=2.2`, a laser bolt): goes off 0.07 s after the
  shot; Tex takes nothing (outside the radius, where the recording's Tex took ~15, above). Since
  #136 the bolt strikes it on its first sweep that meets it (play_bolts.rs): the LZR-23's from
  9.6 m, 0.1 s after the shot.
* The level's mines (#228, `BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SENTRY_LOG=1
  "BF_TEST_PANEL=a,1;a,1;a,1;a,1"`): the three armed at the start, team 1. Tex walked up to the
  first (`BF_TEST_GOTO=92.6,-62,92.6,-71.0`, after the mine lesson's cutscene) stops 1.08 m
  across from it and it stays through every check; stood on it
  (`BF_TEST_GOTO=92.75,-71.6,92.75,-71.6`) it goes off at the first check, 0.22 m across:
  DESTROYED -> routers 446 and 470 -> calculator471 at 1 of 3; Tex takes 39.7 (115 -> 75.3).
* Regression after #228 (the hazard rule): the trigger-distance and `wise` runs above give what
  main's build gives (stays at 3.05 m, off at 2.99 m, Flint taking 31.0 in both: the roll's,
  where this page's first run had 22; `wise` off at 0.20 m across).
