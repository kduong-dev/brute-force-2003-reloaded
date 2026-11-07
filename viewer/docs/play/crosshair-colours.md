*Part of [play/](README.md), the former page `play.md`.*

### The crosshair's colours (#141)

`src/bin/play_reticle.rs`, drawn by `play_hud.rs`. Read from `default.xbe` (each function is
cited in the code and has a row in [code-map.md](../code-map/README.md)), checked against the takes in
`todo/141-crosshair-colours/` (takes 01-09 and xemu's live reads in their notes).

The crosshair is drawn in one of four tints, worked out every frame and switched on the frame
its condition changes (no fade, as in every take):

| Tint | When | The game's colour (FUN_000bbd70) | Drawn as |
|---|---|---|---|
| red | the reticle holds a **lock** on a hostile | 160 32 32 | 152 32 32 |
| yellow | no lock; the first thing under the centre has **radar-contact 2 or 4** | 200 200 0 | 190 190 0 |
| green | no lock; the first thing under the centre is a living **teammate** (not a 2 or 4) | 68 220 68 | 66 209 67 |
| blue | anything else | 50 150 255 | 49 143 243 |

At alpha 229 / 255 (FUN_000c6280's 0.9 x 255; it was 0.95). The console blends in the stored
sRGB values, Bevy's UI in linear light, so each is drawn as the colour that gives the game's
result over a dark ground, as the grenade meter's orange is. The demo's own green on a squadmate
(77 255 89, no source) is gone: the game's is the code's 68 220 68, by its own rule.
Under the Squad Command menu (#319) FUN_000c6280 takes FUN_000c17c0's share too: the
crosshair fades out as the menu fades in (1 - fade / 0.3) and is gone while the Move To cursor is
up (take01, take08); the item box's widget takes the same share (0xabdb5: `play_itembox.rs`
`draw`).

* **The lock** (the target selection's FUN_000bf7a0, the best kept by FUN_000bfe40): any living
  character but the player when
  * it's hostile by the team table (0x3ffd80, the row of the player's team): the squad locks
    teams 1, 7 and 8; a neutral (2) or a teammate never;
  * its **aim point** is within reach of the gun's muzzle: game-options' `<reticule
    h_0c3eb4ff="0.7">` x the gun's reach (FUN_00232520): the bullet's `range`, x0.9 for a burst
    of pellets (the Bower 20), and for a mesh projectile its second timer x its speed added (and
    its range again with MIRV on explode): LZR-50 63 m, Jax-iP 28 m, Therm Sweeper
    0.7 x (20 + 20 + 5 x 30) = 133 m. With a gun in the second hand (Tex's berserk) the smaller
    of the two guns' reaches counts, measured from that gun's muzzle (0xbf82a-0xbf8aa, then
    FUN_000bff40 at 0xbfb14): with the LZR-50 in hand and the Jax-iC second, the Jax-iC's 42 m
    from his left hand (`LockGun`); the box below and the centre ray stay the gun in hand's
    (0xc57e1). The aim point is the point of the character's **aim
    capsule** (its vtable +0x134, 0x121d00) nearest the crosshair's ray (FUN_0015b450): from 40%
    of the way from its "Bip01 Head" node to its root node (the hips) to half that length below
    the hips, 0.15 of it thick. So a sweep locks as the body's middle, chest to thighs, comes
    near the centre, not one point of it;
  * projected on the HUD, it's within a box round the crosshair's centre of 33.6 x s units each
    way (`<reticule h_f079aebd="0.14">` x 480 / 2; across, x 0.75 x 640 / 2: square on screen),
    s the weapon's `h_e1e41c2d` (`WeaponDef::lock_box`: LZR-50 0.85, Jax-iP 0.6, Therm Sweeper
    0.8, the iKhan-GPL 0: never), **x1.2 for the one locked last frame** (0xbfae9): the LZR-50
    locks within ±28.6 units and keeps it to ±34.3;
  * a ray from the camera to it (256 m) meets it before the world or another body: a squadmate
    walking in front hides it.

  Of those, the most central wins: the lowest depth x (0.3 + 0.8 (1 - score)), the score a
  smoothstep of how near the box's centre it is.
* **The centre ray** (0xc5720), with no lock: the first body or placed object the ray through
  the crosshair's centre meets within the gun's range (256 m with no gun) decides. A hostile is
  locked at once in the scope (and with no gun, or a gun whose lock reach outruns its range: the
  Therm Sweeper's 133 m against its 20 m); out of the scope it stays blue (take02 and take03: out
  of reach, blue), whatever its radar-contact: for a hostile the radar test is skipped (0xc5943 ->
  0xc5a3e). A living teammate (the same team, one not hostile to itself) is green; then anything
  else with a radar-contact of 2 or 4 (a teammate, a neutral, a placed object) is yellow. So red
  beats yellow and yellow beats green. No box: green and yellow only while the centre is on the
  thing.
* **Radar-contact** (an object's +0x7a, `radar-contact-type` h_1905f381; RCONTACT_ 2 OBJECTIVE,
  4 OBJECTIVE_SECONDARY): every character has 1 (`Player::radar_contact`). A level's objects take
  the placement's own `h_1905f381`, else their type's, then the radar-triggers that operate at the
  level's start (signal 13, FIRST_UPDATE) set theirs (`bf/level.rs`: `Placement::radar_contact`,
  `Level::radar_triggers`, for the radar's #160 too). On e34 that's the five missile racks and
  three h_e8589e7e devices at 4 and the device h_00bc61ab at 2, exactly as the game had them
  live; no squad deathmatch map has any. The centre ray tells which placed object it met from
  the arena (`Arena::ray_object`).
* **The guns aim at the lock** (#124): with a lock the reticle gives the controlled character its
  point (0xc5b90 -> FUN_0011e9d0, char+0x804): the centre ray's point when it meets the locked
  body (0xc59fe), else the lock's aim point (0xc5330 -> 0xc5170). The character update aims each
  of its guns there (0x11aa20 -> 0x22dc60) and each muzzle's shot goes to it (FUN_0022e5b0):
  `Reticle::aim`, play.rs `Player::lock_aim`, the arms turned onto it and the shots sent to it
  (from the eye to it in the scope), the second hand's gun's too, a missile's aim point too. So a
  burst fired with the crosshair red hits the locked body (the LZR-50 4 of 4 at 9 m with the
  crosshair inside its box; blue, 49 units off, 4 misses). Without a lock nothing changes: the
  shots go to what the crosshair is on. The AI aims at its own target.
* **Flint's Advanced Targeting** ([skills.md](../skills/flints-advanced-targeting.md)): while
  it's on her skill sets the box: 0.4 of the screen each way (her +0xc4, 0x13cd30, writes the
  selection's shares) at scale 1.0 (her +0x100, through FUN_000c5d20 each frame): 128 x 96
  units round the crosshair; her +0xfc fades the crosshair's alpha (0.9 x (1 - her marker's
  fill), `Reticle::alpha`); her rifles' range is x1.25 (FUN_002324c0 through her +0x54), the
  centre ray's and the lock's reach with it.
* **Hawk's stealth** (#123, skills.md): a character less visible than 0.1 (its skill's +0xe4,
  `skills::visibility`; FUN_000bf6d0, `LOCK_VISIBILITY`) is neither a lock candidate nor locked
  under the centre (FUN_000bf550): a stealthed Hawk on another side leaves the crosshair blue.
* **Not in:** the line-of-fire ray (0xc54d0: an obstacle between the gun and a locked target
  would show its own colour after two frames; never filmed); the alpha's blink; the
  scoped lock box (0x11cfb5: once the scope-in's 0.85 s timer, char+0x7c8, has run out, the box
  scale is the skill's +0x58 x the gun's, 0.2 for the squad, 0.8 for Flint, where the demo keeps
  the gun's); the crosshair moving onto a lock (0xc5a50, FUN_000c5e20 at the skill's +0xf8 speed:
  the takes' crosshair stays put); a moving target's lead (FUN_0022e5b0, for a bullet that isn't
  instant: the demo's shots reach their target at once). The demo's bodies are cylinders, so the centre ray
  doesn't slip between an arm and the body (take04's 2-3 blue frames on Flint), and it passes
  the dead (the game's meets a body and tests its radar-contact). A choice of the demo's: the
  candidates are tried in the squad's order: the game walks the world's target list (0xba1c0),
  whose order isn't traced, and with its running best scores that order decides between close
  candidates. (The second hand's reach, a choice here before #121, is the game's now: the lock
  above.)

Test hooks: `BF_RETICLE_LOG=1` prints the tint, the lock and what the centre ray is on at each
change (`=2`: every frame), with each hostile's aim point on the HUD, the box it's tested
against and why it isn't locked, and the camera; `BF_TEST_SWEEP=<deg>|<from>,<to>,<deg/s>[;...]`
turns the camera (degrees, + to the left; with `BF_TEST_GOTO` from its start-to-goal heading);
`BF_TEST_RADAR=<object>,<value>[;...]` sets a level object's radar-contact by its name as the map
starts (sdm_e34's rack: `h_128f3de2,4`). On the test map `BF_TEST_NPC` takes `neutral` or a team
number, `BF_TEST_NPC_TEAM=<s>,<n>,<team>` sets one and `BF_TEST_NPC_RADAR=<s>,<n>,<value>` gives an
NPC a radar-contact ([testtools.md](../testtools/test-hooks.md)).

Checked on the test map, Tex kneeling with an NPC 9 m ahead (`BF_NO_SQUAD=1
BF_TEST_GOTO=4.5,20,4.5,19.5 BF_TEST_CROUCH=0.2 BF_CAMERA_PITCH=-0.12 BF_TEST_WEAPON="Confed
LZR-50"`, `BF_CAPTURE`):

* take08's sequence on one NPC (`BF_TEST_NPC=0.5,tex,enemy,dummy,4.5,11 BF_TEST_SWEEP=1.1
  BF_TEST_NPC_TEAM="3,1,neutral;9,1,friend;12,1,enemy" BF_TEST_NPC_RADAR="6,1,2;9,1,1"`): red,
  blue, yellow, green, red, each a one-frame step. The arcs against take08 (the 15 strongest
  pixels' median; the mean of the arc's pixels): red 120 25 24 / 100 31 29 (the take 125 26 32 /
  95 22 26), blue 43 114 196 / 38 89 148 (44 114 201 / 35 85 142), yellow 150 151 1 / 116 116 17
  (156 161 16 / 118 126 20), green 52 168 52 / 44 129 45 (64 184 56 / 48 134 46). The peaks run
  a few levels low: the game draws the 64 x 64 texture 1:1 at 640 x 480 and xemu scales the
  picture, the demo samples it at 1.5x (so every HUD texture); the take's yellow leans green
  from the video's colour encoding.
* A sweep like take01 (`BF_TEST_SWEEP="20;2,9,-6;11,18,6"`): red from +27.5 units (the
  LZR-50's ±28.6) to -34.8 (its ±34.3 kept), and back from -28.3 to +35.9: it locks nearer the
  centre than it lets go, as take01 does.
* A friend (`BF_TEST_NPC=0.5,flint,friend,dummy,4.5,14`, `BF_CAMERA_PITCH=-0.17`): green only
  while the centre ray is on her, a narrower window than the red's. With the squad in play,
  squadmates walking across the crosshair turn it green, and one walking between Tex and the
  enemy hid the enemy (blue) until it passed.
* Precedence: an enemy with radar-contact 2 stays red; a friend with 2 is yellow; with 1, green;
  shot dead (`BF_TEST_TRIGGER=12-12.6`), blue.
* Reach: the Jax-iP (28 m) locks an enemy at 25 m and not one at 33 m with the centre on it
  (blue; the log: "out of reach: 32.7 m from the muzzle, reach 28.0"). Flint's MK-ASLT on an
  enemy 60 m away: blue unscoped, red in the scope (`BF_TEST_AIM=1`: the centre ray's lock). The
  Therm Sweeper locks it at 63 m (box 26.9) while its centre ray stops at its 20 m range.
* The second hand's reach (#121, standing: `-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.535,19.5
  BF_CAMERA_PITCH=-0.07 BF_TEST_NPC=0.5,tex,enemy,dummy,8,-30 BF_TEST_SKILL=2 BF_RETICLE_LOG=1
  BF_TEST_WEAPON="Confed LZR-50;Jax-iC"`): the enemy 50 m out is red with the LZR-50 alone
  (0.60 s, "locks", 53.6 m deep) and blue from the Jax-iC's draw into Tex's left hand (2.60 s:
  "out of reach: 50.9 m from the muzzle, reach 42.0"), as the frames show at 1.3-2.0 and 3.0-4.0 s.
* Level data: on e34 the level's radar-contacts load (`BF_RETICLE_LOG` lists them as the map
  starts) and the crosshair is yellow on the missile rack h_00815cce from take06's spot (`BF_MAP=e34
  BF_NO_SQUAD=1 BF_TEST_GOTO=-44.68,17.61,-44.21,17.77 BF_TEST_CROUCH=0.2 BF_CAMERA_PITCH=-0.2
  BF_CAMERA_DISTANCE=7 BF_TEST_WEAPON="Confed LZR-50" BF_TEST_SWEEP="15;1.5,11.5,-3"`), blue on
  the pillar beside it.
