*Part of [play/](README.md), the former page `play.md`.*

Test hooks: `BF_MISSILE_LOG=1` prints each missile's launch (a lobbed one's ends "lobbed (gravity
9.81, damping 0.1 a second)"), seeks, retargets, apogee ("at its apogee: rising -0.45 m/s, 5.09 m
above its launch"), split (children,
targets), fuse and end (where, how far, how long after the launch, what it struck), each direct
hit's amount with its roll, hit-location factor, struck bone and step (`x0.31 at bone
h_044f3505, x0.60 the step`), and each body its blast reaches (with its step) or finds behind
cover (`BF_COMBAT_LOG` adds each hit's factors, see **Damage**); a give-up says how far and how far off
its heading the aim point was. `BF_MISSILE_DEBUG=1` also prints every body each target search
looks at, with the root it measures to; `BF_SHAKE_LOG=1` prints each shake kick and its end.
`BF_MISSILE_CUBES=1` (#83) draws the placeholder meshes too, the hounds' ball's 0.5 m cube, which
is left out by default (**The hounds' Breath Attacks**).
The takes' places, with
`BF_TEST_WEAPON="Therm Sweeper"` and taps as `BF_TEST_TRIGGER=3.0-3.15,...`:
* take01 `BF_MAP=e34 BF_TEST_GOTO=-53.64,31.07,-53.76,31.56`; take04 `BF_MAP=e40
  BF_TEST_GOTO=-16.00,152.32,-16.50,152.26` (the take's view, the impact on the reticle:
  `BF_CAMERA_PITCH=-0.10 BF_TEST_FREECAM=0.5,-13.24,28.9,152.81,80,-7.5`); take06 `-- --test
  BF_MAP=e40
  BF_TEST_GOTO=17.15,175.43,16.69,175.23 BF_CAMERA_PITCH=-0.2 BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC=0.5,brutus,enemy,dummy,-5.81,165.55` (the crosshair on him; the take's view:
  `BF_TEST_FREECAM=0.5,19.72,31.14,176.54,66.7,-15`).
* The gate takes (10-16, 22, 23): `-- --test BF_MAP=e34 BF_TEST_GOTO=-55.2,z,-55.2,z+0.5` with
  Tex d m from the panel (its surface at z 41.285, so z = 41.285 - d), and Flint or an enemy put
  on the floor under y 31.5 (`BF_TEST_NPC`'s height, or it lands on the gate's top): take14
  `BF_TEST_NPC=0.5,flint,friend,dummy,-54.2,39.285,31.5`, take16 at x -56.2, take22's enemy
  behind the panel at (-56.5, 42.8) (the take's (-55.2, 42.8) is within 5 m of the gate's door,
  which opens for anyone in the demo, enemies too), take23's in front at (-56.7, 40.5).
* The homing takes (17-19) on the test map: `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_CAMERA_PITCH=-0.08 BF_TEST_INSTANT_KILL=0` and Brutus dummies 24 m out 2 m either side of
  the line (17), 30 m out (18), or 30 m out 4 m left (19); the takes' camera 3.5 m behind Tex:
  `BF_TEST_FREECAM=0.5,4.5,1.3,23.5,0,-3`. take07's 45 m: one dummy at (4.5, -25), with
  `BF_CAMERA_PITCH=-0.115` to put the crosshair on it (at -0.08 the line passes over him). Four shots
  2.4 s apart, each at fresh dummies (`BF_TEST_TRIGGER=3.0-3.15,5.4-5.55,7.8-7.95,10.2-10.35`,
  the dummies spawned again at 5.0, 7.4 and 9.8 s, and
  `BF_TEST_NPC_REMOVE=4.95,all;7.35,all;9.75,all`) give eight children each.
* A side view of a launch (the dust): the same spot, `BF_TEST_FREECAM=0.5,10.5,1.6,19.0,90,-5`.
* The reload (08): `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5 BF_TEST_RESERVE=32 BF_AMMO_LOG=1
  BF_START_WEAPON=1 BF_TEST_TRIGGER=2.5-11.9,15.5-15.65,...`; the open air (20): the same spot,
  `BF_CAMERA_PITCH=0.17`, one tap.
* The Sweeper V's takes (#72), with `BF_TEST_WEAPON="Sweeper V" BF_NO_SQUAD=1` and taps
  `BF_TEST_TRIGGER=3.0-3.15,5.7-5.85,...`: take05's cliff `BF_MAP=e40
  BF_TEST_GOTO=-17.21,152.02,-17.70,151.93 BF_CAMERA_PITCH=-0.10` (the goal 0.5 m ahead: he
  turns to it and stands; the take's view `-- --test BF_TEST_FREECAM=0.5,-14.48,28.9,152.66,77,-7.5`);
  take01's gate `BF_MAP=e34 BF_TEST_GOTO=-53.64,31.07,-53.76,31.56`, and 1.5 m back
  `-53.31,29.62,-53.43,30.11`; take03's enemy at the gate spot, `-- --test BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC=0.5,h_1c5780f8,enemy,frozen,-55.61,37.79 BF_TEST_NPC_HP=1.0,1,300`; the open air
  `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5 BF_CAMERA_PITCH=0.17` (to see it,
  `BF_TEST_FREECAM=0.5,4.5,1.3,23.5,0,14`); the fuse, the same spot with `BF_CAMERA_PITCH=-0.115
  BF_TEST_NPC=0.5,brutus,enemy,dummy,6.2,-15.5` (33 m out and 1.5 m off the line, where the
  second check, 36 m out, finds him just passed); an enemy's launches, e40's Shrike Heavy
  placement `BF_TEST_NPC=0.5,h_17bc6519,enemy,fight,2.15,168.43` with take06's spot.
* The iKhan-GPL's takes (#239, #55: `todo/55-ikhan-gpl`), on e40 with Brutus as in the takes,
  `BF_MAP=e40 BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_WEAPON=e9be70f0`, taps
  `BF_TEST_TRIGGER=3.0-3.08,5.2-5.28,7.4-7.48`: take01's cliff at 10 m
  `BF_TEST_GOTO=-30.63,151.66,-31.10,151.48 BF_CAMERA_AIM_AT=-40.22,28.5,147.40` (the crosshair
  1 m up the cliff from its foot, as the take's sits; at the foot itself the round comes down 2 m
  short on the rising ground); take08's at 25 m `BF_TEST_GOTO=-16.45,152.86,-16.94,152.76
  BF_CAMERA_AIM_AT=-40.6,28.0,147.9`; take03's frozen enemy 7 m out `-- --test
  BF_TEST_INSTANT_KILL=0 BF_TEST_NPC="0.5,shrike soldier,enemy,frozen,-6.62,164.35"
  BF_TEST_NPC_HP=1.0,1,300 BF_TEST_GOTO=-0.19,167.13,-0.65,166.93` with
  `BF_CAMERA_AIM_AT=-6.62,27.6,164.35` (on his chest) or `-10.01,27.6,162.12` (past his side, the
  fuse's case); the apogee, open air from take08's spot turned round,
  `BF_TEST_GOTO=-16.45,152.86,-15.96,152.96 BF_CAMERA_PITCH=0.35`.
* The hounds (#83): an NPC set to fight, `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_TEST_NPC="0.5,shrike hound@m09_a,enemy,fight,4.5,-5"` (`fire hound@e13`; `shrike hound@dm`
  and `fire hound@dm` for the deathmatch balls; e34's own placement `h_15b107ab` on that level),
  with `BF_MISSILE_LOG=1 BF_COMBAT_LOG=1` and a side camera `BF_TEST_FREECAM=0.6,-3,1.8,12,-90,-4`
  (the recipes and what they showed are under **Verified**). No new hook was needed.

Verified (15 fps captures and the logs; the takes' damage x1.4 or x0.6, see the difficulty
step):
* Timing: the missile leaves on the fire loop's frame with the clip drop, ed761bae and 0021996c
  (`BF_SOUND_LOG`); the split comes 20.0 m and 0.667 s after the launch every time. take04's
  cliff: the children strike it 23.4 and 24.3 m out, 0.77 and 0.80 s after the launch (the
  takes: blasts 0.80 s after the glow at ~25 m).
* Open air (take20): the children's timers run out 5.67 s after the split, 168.8 m beyond it,
  and they go off where they are (the take: 5.58 s game time, 168.6 m, a climb with no drop).
* Homing (round 5: the root, the fixed heading, the give-up before the fuse; the dummies are
  Brutus, damage-multiplier 0.62 and holding his gun in front, where the takes' Shrikes are 1;
  four shots each, `BF_MISSILE_LOG`):
  * take17 (two enemies 4 m apart, 24 m out): the split, 2.9 m short of their hips and ~38
    degrees off, deals neither out in any of the four; the children fly on between them, nobody
    hurt, as the take shows.
  * take18 (30 m out): the split gives each child an enemy; all eight strike, at a forearm
    (x0.5, so x0.31) but one at a thigh (x1, x0.62), then each enemy takes both blasts' shares
    (42-47): a x0.5 direct and both shares, as the analyst reads the take.
  * take19 (one enemy 30 m out, 4 m left of the line): 5 of the 8 children strike him, each at a
    forearm (x0.31); 3 give their aim up 4-6 m short at 61-62 degrees off their heading and fly
    on, landing 36-42 m out. The take: all four children of its two shots struck, at x1 parts.
    Not matched (see below).
  * take07's 45 m (the enemy on the line): the split, 25 m short, finds nobody; the children,
    turned 9 degrees and homing on the parent's point 200 m ahead, fly parallel 1.05 m either
    side of his hips (the demo's body is 0.4 m round) and on: no hit, no share. take07's two
    children struck him directly (-256.8 at once, two x1 directs): not matched (see below).
  * take06 (25 m, three shots 2.37 s apart, the tester's setup): the children give their aim up
    within 1 m of his hips at the end of their straight phase (0.95 m, 93-95 degrees off) and
    fly on: shot 1, one share (46.5, the blast 4.3 m from him; the other child lands 26 m on),
    shot 2 one (44.8 at 1.3 m), shot 3 none (the nearer lands 6.1 m from him). take06: one share
    per shot (61.6, 64.6, 64.0), both on shot 4. Round 4 gave two on shot 1.
* On sdm_e34 (the sdm_bf squad, damage-multiplier 1): a Brutus dummy 7 m ahead of the first
  start point (`BF_MAP=sdm_e34 -- --test BF_TEST_NPC=0.5,brutus,enemy,dummy,-0.26,41.69`, a tap
  at 3 s) takes the missile on his thigh, x1.00 (93.6 of a 93.6 roll), where the campaign's types
  give x0.62.
* The count once a game frame (round 4): take18's two blasts emit the same at 15, 30, 60 and
  144 fps (`BF_CAPTURE_FPS`, `BF_ALE_LOG=1` prints each batch): per blast 3 flash, 2 + 2 fire,
  5 fire-add, 6 dirt and 43 shrapnel particles, the analyst's simulation of FUN_00065800. The
  trails differ by a game frame or two at each missile's end (see below).
* Self-damage (takes 10-16, the gate): Tex 2, 3, 4 and 5 m from the panel takes 32.8 from his
  own blast (half of 93.6, x 0.7), at 6 m nothing (the takes: 18.6-20.1 at 2-5 m, x0.6; nothing
  at 6 m); he isn't knocked down. Flint 1 m to his left or right takes the same 32.8 (takes 14,
  16: the same for both, 19.3 and 18.7).
* Cover (takes 22, 23): an enemy 1.5 m behind the gate's panel is never reached by three blasts
  on it (behind cover from the blast's point); one in front of it, right of the hits (take23's
  spot), takes a half roll from each (46.8, 42.5; take23: 65.9, 62.0, 65.3, x1.4) and isn't
  moved.
* take03's geometry (an enemy 7 m out): 93.6, then 42.7 0.13 s later (take03: 128.8, then 64.9
  0.2 s later). A friend on the line: 93.6 x 0.7 = 65.5.
* Reload (take08, the gun in Tex's second slot): the held trigger fires every 1.67 s; the
  reload starts at the release, the reserve dropping 32 -> 26 at once, and the clip refills
  1.13-1.2 s later at Tex's Sc_w2_reload magazine-in event (1.20 s; FUN_00117660, event
  1b2ec99e moves the pending rounds into the clip); the take: 1.26 s held, 1.15 s after taps
  (Tex's Sc_w1_reload has it at 1.00 s: `BF_TEST_WEAPON` alone puts the gun in his first slot).
  After the last tap the reload starts 0.13 s later (the take: 0.19 s). So reload-time 5 isn't
  a 5 s reload.
* Camera shake (`BF_SHAKE_LOG`): kicks of 0.264 (take06) and 0.225 (take04) for blasts ~26-27 m
  from the camera, over 0.77 and 0.73 s: a turn of up to +-0.0066 rad, which shows as a jitter
  of the frames. The takes: +-1-5 of 960 units for ~0.7 s at 25 m (take04), +-5-10 for ~1 s at
  6 m (take03, where the code gives the full 0.74).
* The look, from the takes' own views (`BF_TEST_FREECAM` above), in 640 x 480 units² above
  luminance 200 (and 240):
  * take04 (the cliff, 25 m), with the impact on the reticle (round 4's view cut the fireball's
    top off, the impact ~50 units above the reticle, and read 17.8k): the two fireballs peak at
    25.4k (18.3k), box 227 x 162 units and reaching the HUD band, bright for ~0.5 s (frames
    54-61); the take at 16.9k (12.7k), 216 x 125, ~0.47 s. **Not matching: 1.5x**, taller and
    whiter. The launch glow lights ~14.0k units² by 40 or more over the frame before (the take
    ~19.2k), 0.2-0.27 s after the shot (the take's 0.3 s): short, and not from the trail's birth
    position (the analyst's Q1). Before the terrain-layer fix neither showed at the cliff.
  * take06 (the sand, 25 m), one fireball at a time: the first shot's nearer blast peaks at
    10.3k, box 137 x 115, bright ~0.4 s; the take's first (f250-f278) 6.5k, 154 x 103, 0.47 s or
    more: 1.6x, the same excess as take04's. The launch glow up to 5.6k (the take up to 4.7k).
  * take20 (the open air, ~190 m, LOD 0): no white flash; each child's blast makes 1 fire-add
    sprite, 1 dirt card and 3 sparks (the analyst's counts), an orange flame ~30 x 25 units for
    ~0.4 s from the takes' camera (the take: 12 x 15 units, ~0.33 s, no flash); round 2's
    ~55-unit white disc is gone.
  * The launch dust: one brown card rising behind the launcher from ~0.13 s, out of the follow
    camera's view (the side view above). From the game's camera aimed (#126's leads: the aim
    shift brings Tex's 3.9 m boom to ~2.2 m behind his head while the gun is up;
    `BF_TEST_FREECAM=0.5,4.5,0.9,22.2,0,-3`) the card is born behind the camera: no veil (the
    frame's mean rises by 2.6 at most, the launch's light). From the 3.5 m spot of the takes'
    view above it's born just in front of the camera and veils the view for ~0.5 s (mean red
    48 -> 91 -> 50). Which of the two the takes' launches are seen from is #126's to settle.
* Muzzle effects (above), the branch against main in the same side views (`-- --test
  BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12 BF_TEST_FIRE=1 BF_CAMERA_DISTANCE=2.2
  BF_VIEW_YAW=<yaw>`): the sphere emitters' glows (A10, Saryl-45, Jax-iC) are round as on main;
  the Minigun's flash leaves the barrel forward (main's fires across it); the scoped MK draws as
  on main.
* The main game on sdm_e34 (`BF_AUTOPILOT=1`, 300 frames) differs from main's by at most 0.4%
  of the pixels from frame 8 to the first shot (296; #120's stamina bar; frames 1-7 differ while
  the scene's render pipelines finish compiling). From the shot on, Tex's LZR-23 plays its
  muzzle effect type's light (light_muz_laser): a reddish light, as the LZR-23's take01 shows on
  the ground at Hawk's feet (f196-f200), where main's demo light was orange-white and brighter.
  Not measured.
* **The Sweeper V** (#72, the recipes above; `BF_MISSILE_LOG`):
  * take05's cliff (three taps): each rocket strikes it 22.4-22.6 m from the muzzle, 0.367 s
    after the launch (11 game frames; the take's fire-to-blast on the sound 0.356-0.413 s, mean
    0.378). From the take's view the glow sits over the reticle from the frame after the launch,
    the blast is at the cliff's foot on the reticle, and the scorch shows there from ~1 s, a
    little left of and below the take's. The blast is the Therm's, with its known excess (below).
  * Self-damage at the gate (take01's spot, three taps): Tex takes 7.4, 5.9 and 6.0 from blasts
    whose ball meets his nearest part 9.50-9.57 m away (the take: 6.5-7.3 per blast, six of six;
    the rule 0.21 x the roll, 5.7-7.8); from 1.5 m further back nothing. The Therm at the gate
    (the part reach, the same runs): 5 m from the panel 19.7, 17.9, 18.0 (takes 10-16: 18.6-20.1),
    6 m nothing (his nearest part 5.18-5.20 m away); take14's Flint 1 m to his left the same as him
    (19.7, 17.9, 18.0); take22's enemy behind the panel never reached (behind cover), take23's in
    front of it 65.5 and 59.6 (the take 65.9, 62.0, x1.4); take06's dummy one share per shot
    (65.2, 62.7; the take 61.6-64.6).
  * take03's enemy at 7 m (the e34 Shrike Heavy frozen, 300 HP, four taps): the direct hit at an x1
    bone, 46.9-49.3, then the share 0.07-0.25 s later, 19.7-25.0: 67-74 a rocket. take03 logged
    two half-roll drops (19-26 each, 41-47 a rocket): its Shrike was frozen 1.2 m off the floor (y
    30.24 against Tex's feet at 29.04), so its rockets met the legs (x0.5) where the demo's frozen
    NPC stands on the floor and is met at the hips. Every drop is inside the rule. Tex, 6.4 m
    from each blast, takes his own share (5.9-7.5); the take didn't log his health.
  * The open air: the rocket goes off where it is, 202.0 m out, 3.37 s after the launch (101 game
    frames: the 101st from the float sum of the 1/30 s steps; the code's 3.33 s and 200 m), a small
    orange burst with no white flash (LOD 0) and no shake (the camera beyond 63 m).
  * The fuse: three rockets past a dummy 33 m out and 1.5 m off the line each go off at their fuse's
    second check, 36.0 m and 0.600 s after the launch, 0.8-1.5 m from him; he takes a share each
    (19.5-24.5). Not filmed (the spec's shot list).
  * An enemy's (e40's Shrike Heavy placement, 15 m away, fighting): a rocket every 2.0 s at Tex's
    hips, each a direct hit (12.5-13.1: the roll x0.62, his type's multiplier, x0.6) and its share
    (8.5-10.7).
* **Lobbed** (#239, the iKhan-GPL's recipes above; `BF_MISSILE_LOG`):
  * The Therm Sweeper on #73's take01 setup and the Sweeper V on #72's take01 setup log the same
    missile lines as main's build, line for line (only the kits' load lines change order, as the
    weapons' map does run to run).
  * take01's cliff (three taps): each round leaves at 23 m/s and strikes the cliff's foot 9.5 m
    from the muzzle, 0.40 s after the launch, the blast out of Brutus's reach (his nearest part
    9.5 m away). The take, at 60 fps: the burst on the cliff at the crosshair's bottom bracket
    0.47-0.50 s after the launch glow's first frame, on two shots. Not matched: 0.07-0.1 s
    longer in the take, cause not found. The code's launch is the bullet's speed exactly. Both
    iKhan takes (01 and 08) would fit a stronger damping, ~0.7 a second; the reviewer noted that
    the objection to it assumed the steady bodies take the damping too. They do, by the code: the
    steady missiles' bodies come from the same template (FUN_00158cd0's 0.1; h_11e7d864 takes them
    off the gravity only), so at 0.7 the Sweeper V's steering would hold ~52 m/s where #72's
    take05 timed 59.5 on average. Kept at the code's 0.1; a follow-up. Where the take's crosshair
    sat on the cliff, and so how far its round went, is read from the picture.
  * take08's 25 m: each round comes down on the sand 12.4-12.7 m from the muzzle, 0.53 s after the
    launch (the take: ~0.47 s after the glow, "~8-10 m ahead" by eye).
  * take03's spot, a frozen Shrike Soldier 7 m out (the take's was a Shrike of e40's spawn
    template 834ae160), aimed at his chest: every round strikes him 0.23 s
    after the launch, a direct roll of 25.5-26.2 (18.2-18.7 x1.4) and then its blast's half share
    of 12.8-13.8, 0.07-0.25 s later; Brutus takes 3.8-4.2 of his own blast (5.7 m from him).
    Aimed past his side (the round 0.7 m from his hips), the fuse splits it 6.8 m out at 0.3 s
    and the three bomblets come down 7.7-13.8 m out, 0.33-0.6 s after the launch: two or three
    half shares of 12.7-13.9, 0.07-0.35 s apart, and no direct roll; Brutus takes one share when a
    bomblet lands within ~8 m of him. take03 logged both patterns: 26 at once (shots 1, 4 and 7),
    two or three shares of 13.1-14.0 0.1-0.2 s apart (shots 2, 3, 5), 26.2 then 13.2 (shot 6),
    300 down to 85.8 over seven rounds where the demo's seven direct hits take 273. That reads as
    most of the take's rounds passing him and splitting at the fuse (an inference from the shares'
    count and spacing; the take's blasts hide the rounds), which the demo's body, a cylinder 0.4 m
    round, doesn't let a round aimed at him do (the game strikes his collision parts; a round
    through the gap between his legs isn't checked). The rolls: 18-20 as the code has them (the
    shares 13.1-14.0 = x1.4 of 18.7-20.0), where the demo rolled 18 every time before.
  * The apogee, 20 degrees up in the open: the split 1.07 s after the launch (its straight phase
    out, rising at -0.45 m/s), 21.3 m out and 5.1 m up; the bomblets fan out (three trails in the
    capture) and land 32-39 m out, 1.7-2.1 s after the launch. Not filmed.
  * An AI's (the tester's repro, `-- --test BF_MAP=e40 BF_NO_SQUAD=1 BF_CHARACTER=0
    BF_TEST_GOTO=17.15,175.43,17.15,175.0 BF_TEST_NPC=0.5,h_1f979a8a,enemy,fight,17.7,196.0`):
    on #239's first build (the tester's run) every round of the Shrike's came down ~5.3 m in front
    of it and its own blasts (9.0-9.7 each) took it from 64 HP to 0 by 11.4 s, Brutus untouched.
    Then (#239's stand-in) it pitched them 13.4 degrees up and each struck Brutus 19.5 m off,
    0.9 s after the launch, a round every ~1.5 s: 6.8-7.4 (the roll x0.62, his multiplier, x0.6)
    and the blast's share 5.5-5.9; it took nothing itself, and Brutus was down at 14.9 s. With
    the squad (without `BF_NO_SQUAD`) Hawk and Tex shot it from 37-40 m, and it got two rounds off
    at Brutus, both hits. Put 6 m from Brutus (17.7, 181.5) it held its fire (no round in 10 s);
    at 10 m (17.7, 185.5) it fired, nearly level, and its own blast missed it by 0.4 m (its
    nearest part 8.97 m away).
    **Since #83 (the game's aim raise and 12 m refusal, see An AI's lobbed rounds):** it aims
    the iKhan-GPL 2.1 m over Brutus's hips (7.7 degrees up, `BF_MISSILE_LOG`), and each round
    comes down on the slope 11.2 m from the muzzle, 0.5 s after the launch, 8.55-8.71 m from
    Brutus's nearest part: of the eight rounds that went off in 15 s, five blasts reach him
    (5.5-5.9) and three fall just out of reach (8.64-8.71 m against 8 + 0.62), and no round
    strikes him. At 10 m
    (17.7, 185.5) it now holds fire (no round in 8 s), where the 8.6 m rule let it fire.
    Not filmed: this is the rule's own result, which the stand-in hid.
* **The hounds' Breath Attacks** (#83; 15 fps captures with `BF_MISSILE_LOG=1 BF_COMBAT_LOG=1`,
  the frames looked at; no footage to compare with). The recipes, Tex at (4.5, 20) facing the
  hound 25 m off: `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_TEST_NPC="0.5,shrike hound@m09_a,enemy,fight,4.5,-5"`, the Fire Hound `fire hound@e13`,
  the deathmatch ones `shrike hound@dm` and `fire hound@dm`; a side view
  `BF_TEST_FREECAM=0.6,-3,1.8,12,-90,-4` (the gas blast at z 15; `-2,2.0,14,-90,-3` for the
  fireball on Tex), the flight alone from `-4,2.0,5,-90,-2`, the launch's light from `9,1.7,-9,131,-4`.
  * The gas ball (24 m/s): each round leaves 1.40 m over the floor, 6.7 degrees up (the aim raised
    ~3.1 m over Tex's chest), comes down on the floor 19.7-19.8 m out 0.83 s later (the same
    flight worked out apart: 18.7-19.1 m) and the blast, 4.4-5.1 m short of Tex, deals him (his
    nearest part 4.4-5.0 m away) half its roll, 14.1-15.3 x 0.6 = 8.5-9.2. At 12.7 m (the hound
    at z 7.3) it fires and the ball, aimed 1.2 m high, strikes Tex in 0.5 s: a direct hit and the
    blast's 8.5; at 11.4 m (z 8.6) it raises its gun and fires nothing in 8 s. On e34's own
    terrain (`BF_MAP=e34 BF_CHARACTER=1 BF_NO_SQUAD=1 BF_TEST_GOTO=-53.6,30.8,-51.6,24.0
    BF_TEST_NPC="0.5,h_15b107ab,enemy,fight,-45,12"`: the placement's own Shrike Hound, 14-20 m
    from Flint as she walks toward it) the ball strikes her 12.6-13.7 m from the muzzle, 0.53-0.57 s
    after the launch: 0.3-0.6 for the direct hit and 0.4-0.5 for the blast (GAS x0.05).
  * The fireball (30 m/s, `fire hound@e13`): the same aim at 25 m puts it on Tex's calf 24.1 m out,
    0.83 s later (1.7 m past, by the worked flight): a direct 3.9 (roll 21.2 x 0.62 x the calf's 0.5
    x 0.6) and a blast share 5.8-6.4 (half of 19.3-21.4 x 0.6), FLAME x1. At 10.1 m the Fire
    Hound fires nothing in 6.6 s; at 10.5 m (the node-to-chest distance is a hair over it in
    3D) and 11.1 m it fires four rounds.
  * The deathmatch hounds (45 m/s, 2 s of life; player types, so **no aim raise**: they fire
    straight at the chest, -0.4 to -0.9 degrees from a muzzle 0.3 m over it): rolls 26 (gas) and
    13 (fire) in 3.5 m; at 25 m (the gas hound beside it, 28 m) each ball falls 0.43-0.47 s
    later to the floor 19.8-21.8 m from the launch (`hits the world`), 3-6 m short of Tex.
    The gas hound's blasts land 5.7-6.5 m from him, out of reach; the fire hound's reach his
    nearest part at 2.8-4.2 m (the core rule: the whole roll to half the radius, 1.75 m, and
    nothing at the edge) or fall just out of it: one share of 0.3, the others 0.0. (Before the
    review's gate, with the aim raised, the balls went over him to 47-50 m.)
  * The rate: launches at 1.47, 3.27, 4.93, 7.40, 9.07, 11.00 s, 1.66-2.47 s apart (the gun's
    cooldown, 1.67 s, and the AI's burst timing); before the infinite clip (above) 3.0, 2.4, 1.7,
    2.0, 1.9, 1.9, 2.4 s, with a reload starting ~1.1 s after each round.
  * What shows (the frames): the gas ball leaves as a line of green puffs with a pale yellow flash
    on the ground round the hound (light_muzzle, ~0.2 s), the blast a translucent green cloud
    ~6.5 m wide and 4.3 m high at its fullest, 0.3-0.55 s after the blast, with white-green
    spark streaks, gone in ~0.75 s (the fixed side camera at 7.5 m: the green pixels' extent), the
    red hurt tint on the view as Tex is hit, and a black scorch under it from ~0.7 s; the fireball
    a line of orange flame wisps, then an orange fireball and embers (0.75 s) with a warm light on
    the floor. The decals read from the data: h_096b8634 (texture 36409627) and h_1e7e7d76 (253765166),
    1 x 1, black, 25 s fading over 5, rotation 180, laid 0.7 s after the blast. The sounds
    (`BF_SOUND_LOG`): the fire sound h_1c9fb6e2 at the launch, the trail's h_0021996c 0.2 s later,
    the impact h_0997ea6b (and the fire blast's h_e297c1ef) at the blast.
  * A player's hound (`BF_TEST_SQUAD_SLOT="0.3,me,shrike hound@m09_a"`, `BF_TEST_TRIGGER=2.0-2.1`):
    the ball leaves along the crosshair and, fired level, comes down 10 m out.
  * `cargo test` (three tests, `missile::tests`): the aim raise's figures (0.5, 2.125, 4.16, 7 m);
    the balls' landing points (gas 15.8, 17.6 and 20.4 m for targets 13, 20 and 30 m off, fire
    21.1, 23.9 and 28.5) and that the blast reaches the target from 12 to 28 m (gas) and 15 to 35 m
    (fire); the refusal at 12, 10.5 and 5.25 m in 3D.
* **The iKhan-GPL** (#55, the recipes above with `BF_TEST_WEAPON="iKhan-GPL"`, which now gives the
  gun every run; `BF_CAPTURE_FPS=60` against the takes' 60 fps):
  * take01's look: the launch is an orange smoke beside the gun and a yellow glow leaving it with
    its ribbon, as the take's (its smoke shows on its second shot; the take's glow is a round halo
    ~90 px across for its first 0.1 s, the demo's a streak: the demo's camera sees the round's way
    from the side, where the take's is behind it, and the flare is a facing-3 quad stretched along
    its motion). The burst is a white dome at the crosshair on the cliff's foot; above luminance
    200 (in 640 x 480 units², 15 fps from the burst) the take's grows 0.6k, 3.2k, 5.2k, 6.7k and is
    gone after 0.33 s, the demo's 0.5k, 1.5k, 4.7k, 4.0k, 3.9k, 2.3k, 1.2k, over 0.47 s: the same
    light all told, 0.7x at its peak and longer. The scorch shows under the crosshair from ~0.7 s
    (the take ~0.8 s).
  * take02 (`BF_TEST_CLIP=3 BF_TEST_RESERVE=0`, the trigger held): three rounds 1.53 s apart (the
    take 1.51-1.60), no reload with no reserve, then a dry click (f740ecdd) on a tap and the
    switch.
  * take06's frozen enemy 25 m out (Brutus at the take's spot, 17.13, 175.49, the crosshair on the
    enemy): each round comes down 14.4-14.6 m out, 9.5 m short of him, and doesn't reach him (the
    take: no hits). take05's 45 m wall is the same arc: down on the sand ~12-15 m out.

Not matching or not done:
* The explosion is too big: 1.5x the take's bright area on take04's cliff (two fireballs) and
  1.6x on take06's sand (one), taller and whiter. Nothing in the game's explosion depends on
  the surface (the analyst's Q2: placement, frame, slots and LOD are the same on both), and no
  exp-rocket appearance uses shape 3 (Q4). Left for a ticket of its own; its leads: the angle
  fade of the facing-3 quads near edge-on (exp-rocket-add's, up to 13.6 m), which the demo
  doesn't do, and the added sprites summed in linear light (#104). A third, now done (#269): the
  demo's view, free camera included, was 45 degrees down where the game's 4:3 picture is 70
  degrees across, 55.4 down, so the same scene was drawn tan(27.7) / tan(22.5) = 1.27x larger
  across the 640 x 480 units, 1.61x in area, about the excess measured (1.5x, 1.6x): expected now
  0.93x and 1.0x, to be measured again ([camera.md](../camera.md)). take20's open-air flame is
  ~2x the take's. take04's late fire column (the take's at +1.4 s) is missing. The curves'
  easing (the demo reads easing bytes 0, 5 and 6 as linear, `bf/ale.rs`'s ease; no exp-rocket
  curve uses them) stays open.
* The launch glow is ~0.7x the take's (14.0k against 19.2k): not the trail's birth position
  (Q1); the trail head's look or the muzzle light (light_muzzle), not traced.
* A missile's trail stops with the missile: its entity goes in the frame the missile ends,
  before the effects run, so the game frames of that render frame aren't emitted (up to 2 at
  15 fps, 1 at 60 fps). take18's three trails make 11, 25 and 8 particles (mtrail, mtrail#1,
  the flare) at 15 fps, 15, 31 and 8 at 30 and 60 fps, 15, 32 and 9 at 144 fps.
* The scorch is a flat quad: at the cliff's foot it has a straight edge where it leaves the
  rock, where the game's wraps the rock (its decal manager clips the world's triangles,
  FUN_00214710 -> FUN_00217920). On flat ground it's the takes' dark ragged blot.
* The children's hits on take19 and take07: they home on the hips now (the analyst's Q3), and
  take06's one share per shot holds on two of three shots, but take19's children strike
  Brutus's forearms (x0.5, his gun held in front) where the take's struck the Shrikes' x1 parts,
  and 3 of 8 give up at 61-62 degrees off their fixed heading 4-6 m short and miss, where the
  take's four all struck. take07's children pass 1.05 m either side of him (the split's 9-degree
  turn and the straight phase's drift, then homing on the point 200 m ahead) where the take's
  both struck him. The demo has no Shrike to test on, and its body is a 0.4 m cylinder where the
  game's are the collision parts; which of those (or the paths) makes the difference isn't
  settled.
* The demo's follow camera put Tex over the crosshair, so the launch and the blasts were mostly
  behind him (before #126: its 45-degree view and the aiming dolly; the game's camera now,
  [camera.md](../camera.md), not checked against these takes again). On e40 Tex and the standing
  stones draw black (main too): not this ticket's.
* The other effects keep the demo's quads, perp rules and count, and have no LOD: the shape 2
  appearances elsewhere (half the game's size), facing 3 on non-perp appearances (the Psionic
  Blast's muzzle ball), the Frag's facing 3 quads. A gun's hit lights (an effect type's light
  on a bullet's hit) are played since #57 (**Bolts and their hits**).
* The Sweeper V (#72): no real reload was filmed (the takes' reserve edit; #73's takes 08 / 09
  stand in: Tex's heavy reload clip); nor its fuse, its end in open air or a side view of a launch
  (the spec's shot list). The blast's margin past its radius is a fit (`BLAST_PART_REACH`: the
  engine's overlap rule isn't in the game's code). The shove of a frozen enemy with no damage after
  a rocket (take08 / 09, 3-9.5 m) isn't matched: the blast never pushes a character (FUN_00224a90
  pushes loose objects only, and deals what it pushes), so it's something else of the physics
  (the analyst, answers-batch1 §13). Its 0.3 m ball strikes a body where the demo's ray meets the
  cylinder: a pass within 0.3 m of the body misses here (the hit part's back-off, `STRUCK_BACK`, is
  #73's 0.5 m).
* Lobbed (#239): take01's flight is 0.07-0.1 s shorter than the take's (above, cause not found);
  a round aimed at a body always strikes the demo's cylinder, where most of take03's passed and
  split at the fuse; a projectile's timers running x5 in a liquid aren't done.
* The hounds' Breath Attacks (#83) have **no footage**: the ball's invisible cube, its trail's
  look, the gas cloud and the fireball (sizes, colours, lights, sounds), the arc an AI's aim
  gives, the attack clips and the cadence of the rounds, the hound's spacing and what it does
  inside 12 m are the data's and the code's, unchecked against the game (the shot list is
  `todo/83-fire-breath/shotlist.md`, A3-A5, B2). The air fields of the gas cloud and the sparks
  (`AIR_FIELDS`) aren't run; the AI's lead on a moving target isn't done; the demo's fire is its
  burst timing (1.9 s a round where the cooldown gives 1.67 s).
* The iKhan-GPL (#55): its explosion's flames. The take's rise and spread up and out from the
  dome (+0.13-0.33 s); the demo's lie flat along the ground. exp_mirvgrenade has a radial field,
  exp-mirvgren_rad.fld (class h_03d20af6; its parameters 5.9, a curve 1 to 0.001, a curve of 370
  falling to 62 over 0.2 s and 0.43, read by Freelancer's FxRadialField as its radius,
  attenuation, magnitude and approach), which `src/ale_fx.rs` doesn't run, and two air fields
  (exp_mirvgrenade_air.fld, a wind of 0.004 m/s at approach 1.49; exp-mirvgren-air.fld, 1 m/s at
  0.3) left off (`AIR_FIELDS`): run without the push, the first would stop every particle where
  it's born. The weapon panel reads "16 / 0" where take01's reads "16": the game's panel
  (0xcbdd0) writes "%d / %d" (0x40d654) only when the held gun's data +0x174 is set and it doesn't
  recharge, "%d Regen" (0x40d664) for a recharging one, else the clip alone (FUN_000bb920); data
  +0x174 names an object type that FUN_0022b6d0 makes with each weapon (0x22d820) and a
  character's animation event makes again (0x118390), which reads as the gun's magazine. What sets
  it isn't found, so the demo keeps "clip / reserve" (a code question). No level defines an ammo
  item for the iKhan-GPL's ammo-type 9.
* The iKhan-GPL (#55): the round's glow and ribbon are pale cream, ~(229-240, 200-215, 125-136)
  (the tester's), where the takes' are a saturated gold, ~(245, 172, 67). The ribbon's colour,
  mirvgrenade_ribbon.app's (0.808, 0.545, 0.149), and the flare's, (0.788, 0.710, 0.271) to (0.792,
  0.459, ...), are added as linear amounts, as `src/ale_fx.rs` adds every additive appearance's
  (the convention play_fx.rs gives: the console adds to the picture's stored gamma values, and
  amounts taken as linear come out about as bright as its sums over a mid-grey picture); encoded,
  the ribbon's is (232, 194, 108). Taken as the console's gamma values it's (206, 139, 38), which
  over the take's dark cliff (~40, 30, 30) adds up to about the take's gold. Not this effect's slip
  but the blend every added effect is drawn with (and checked against its takes with): a
  follow-up, e.g. adding the effects in gamma values.
