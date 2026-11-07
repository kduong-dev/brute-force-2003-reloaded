*Part of [play/](README.md), the former page `play.md`.*

### Shots: fire timing, pellets, accuracy, holes, muzzle effects, tracers, casings (#115)

`src/bin/play_shots.rs` follows default.xbe's weapon class (0x22b000-0x233000); the attributes it
reads are `weapon::ShotData` (each with its data offset and the function that reads it).

* **Fire timing** (FUN_0022f2a0, FUN_0022f0e0, FUN_0022ec90). The fire loop runs in 30 Hz game
  frames. A shot *sets* the cooldown to 1 / rate (the rest of the frame is dropped), it counts
  down a frame at a time, and the trigger fires again once it's <= 0. So an interval is the
  rate's period rounded up to whole frames: MK-ASLT 8/s -> 4 frames, 7.5/s (the takes: 7.0-7.4,
  the emulator ran at 27-29 fps). Most periods are a whole number of frames (Minigun 2, Foley 10,
  LZR-50 8, Jax-iC 15): the cooldown lands on 0 and the frame time's wobble decides between
  that frame and the next, about half each, which the takes show (Minigun 0.086 s a round =
  2.5 frames, ~12/s; Foley 0.352 = 10.5; LZR-50 0.284 = 8.5). The demo models that wobble as
  +-0.01% on each 30 Hz frame (`TICK_JITTER`), which moves no other period. Its random numbers
  are the frame clock's own (`FireState::clock_seed`, #54): drawn from the gun's stream between
  its shots' draws, a cycle of a fixed number of draws sampled the LCG at a fixed stride, and at
  the Foley's 16 (a shot's 3 accuracy draws and 2 effect seeds, then 11 frames) a cycle's frames
  ran short together every cycle: 11 frames every round, 2.73 a second against the takes'
  2.79-2.93; now 10-11, 2.89 a second. (Every gun's random patterns moved with it, not their
  spread.) In
  the scope the rate is h_019c314a (the MK's 6/s; medium confidence that the holder's +0x7a8 is
  the scope); a dry click (`play_ammo.rs`) and the Charge's shot cooldown use the same
  `shots::period` (FUN_002327f0, FUN_0022dc00).
* **Taps and the raise.** A press is not remembered, except across the raise (#360). The
  controller posts CT_FIRE_WEAPON every frame the trigger is past 0.2 (0x134531-0x1345c1); the
  command sets the fire flag (+0x224 value 4) of the gun in hand and of the second hand's, and
  the flag lives one weapon update: the fire loop (FUN_0022efe0) fires only with the cooldown
  out, and FUN_0022f770 then clears the flag whether a round went or not. So a press inside the
  cooldown fires nothing, and a round needs the trigger down on a frame where the cooldown has
  run out: on the L-Shot-75 (1 round/s) a tap inside the second after a shot is dropped, a hold
  across the cooldown's end fires at the end. (Before #360 the demo latched such a press and
  fired a ghost shot when the cooldown ended, the button long up; the +0x260 bit 3 it was read
  from is the may-aim bit FUN_0011e000 recomputes every frame, not a latch.) An empty gun's dry
  click and auto reload (FUN_002327f0) run only from the fire loop, so a tap inside the
  cooldown gives none either. The one latch is the controller's +0xa1 bit 3 (0x1345b8): a
  press that finds the gun down raises it and sets the bit, and while it is set the controller
  posts the command every frame without reading the trigger, until the ready timer is at 4 or
  below (0x134a25 clears it), so an 80 ms tap from idle fires once the gun is up
  (`FireState::pull`: `latch_left`, the raise's in-time and two game frames, `LATCH_GRACE`; the
  latch ends with the raise, shot or not). During held fire the command holds the timer at 4,
  so the bit stays clear. With two guns (Tex's berserk) the latch is kept for each hand
  and both are raised together, so a tap from idle fires both guns, each when ready; a gun
  still cooling down does not fire later on its own (#121: a tap at 3.98 s fired the LZR-50 at
  4.07 and the Jax-iC at 4.27, after the button came up; read as the Jax-iC's draw into the
  second hand finishing after the tap, by the code its request is dropped in the frame it is
  set), and while a latched press is pending an empty one runs its dry cycle (the request sets
  both flags), but the demo clicks and syncs the LZR recharge only while the button is down
  (`u.fire`, play_ammo.rs); the game's FUN_002327f0 clicks on every such cycle, so an empty gun
  after a short tap stays silent in the demo. From idle the gun is raised first, for the character's own ready time (#140;
  todo/140-footage-timing/answer.md §1): FUN_00123ea0 starts the ready timer at 4 + its motion
  set's `pc-transition-ready in-time` while a player controls it, else its
  `npc-transition-ready in-time` (FUN_0010fce0; `characters/ms_<name>.xml`, h_f599b114 /
  h_170db5cd), and the fire command fires nothing while the timer is over 4. The squad's pc
  in-time is 0.1 s, so a tap's round comes on the 3rd or 4th 30 Hz game frame after the press,
  0.10-0.13 s (`raise`: the in-time and one game frame, as the game tests the timer before it
  counts it down); as AI the squad's is 0.2 s (Brutus 0.3), e40's Shrike Soldier 0.3 and Heavy
  0.4. The demo's 0.2 s before was fitted to the takes' 0.19-0.24 s press-to-shot, which ran
  from the pad input to the 10 Hz watch log, so it held the rig's 0.02-0.06 s input delay and
  the log's 0-0.1 s lateness on top of the game's 0.10-0.13 (xemu.md, "Timing in the takes").
  The ready pose blends in over the same in-time and out over the out-time (FUN_0011a440: 0.2 s
  for the squad), where other clip changes take the demo's 0.2 s. **#172** (the aim "snap", not
  reproduced, not closed): `BF_TEST_AIM_FOR=<from>,<to>` aims between two times and
  `BF_TEST_STOP_AT=<s>` lets go of the stick (with `BF_TEST_GOTO`, `-- --test`); at
  `BF_CAPTURE_FPS=60` the pose changes smoothly in both directions. Raise (`BF_TEST_GOTO=0,8,0,8
  BF_TEST_AIM_FOR=1.0,2.0`): the picture moves from the first frame after 1.0 s, at full rate
  at once (the ready clip's linear 0.1 s blend, the arm easing onto the crosshair at 10/s, the
  camera moving in), and settles by about 1.35 s (these frame differences include the
  camera's own move, so they time the picture, not the pose alone); **lower**: the pose
  (`BF_MOVE_LOG` clip 70, the ready idle, to 48, the carried one) lowers at the release, 2.0 s,
  over the 0.2 s out-time, because the demo's aim has no ready timer behind its pose
  (`upper_aim` is `p.aim`, the fire's `aim_hold` or a skill's hold, not `FireState::ready`: the
  game keeps the gun up for the timer's 4 s after a shot, #277). The **follow camera** doesn't
  come out then: the aim holds the ready timer at 4 (`keep_raised`), so after the release the
  timer runs its 4 s, the aim shift holds 2.33 s more and eases out over the last 0.67 s
  (play_camera.rs `SHIFT_HOLD`; #126 take02's 4 + 2.33 + 0.67), so with a release at 2.0 s the
  camera starts out at about 8.3 s and is out by 9.0 s, for every member, at any frame rate
  (the tester's 15 and 60 fps runs). That is the cause of "the gun and camera stay raised"
  until about release + 6.4 s: the camera, not the gun; the pose is down from 2.0 s (frames
  25-125 of a 15 fps run show the carried pose throughout). A side step let go (`BF_TEST_AIM=1 BF_TEST_STRAFE=-1,0 BF_TEST_STOP_AT=2.0`):
  the step-to-idle crossfade runs a steady 0.2 s (frames 121-133 at 60 fps, then still). At
  the 15 fps of the tester's captures that is 3 frames, 1.5 of them for the raise's blend: no
  one-frame jump comes from the pose code, so what #170's tester saw (stop_L f47 to f48) isn't
  in the demo as it stands; what could differ from the game (the arm's own 10/s easing, the
  game's upper-body aim block, a 70 degree body turn) needs an xemu shot or a trace of the
  game's aim code, as the ticket's comment says. An AI character raises its gun
  as it takes up a target (the attack check's FUN_00123ea0, 0x2509c; play.rs `ai_fire`), so its
  first round waits its npc in-time, which runs alongside the demo's own opening delay (0.2-0.8
  s, `ai_delay`) and is mostly hidden by it. After a shot the gun stays up 4 s
  (`READY_HOLD`: FUN_00123f60 keeps +0x7dc at 4 or more, medium confidence), so a tap within it
  fires in the frame the press is seen (the takes' taps 2.2 s apart logged 0.07-0.11 s after the
  input: the same two delays). Checked (`-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=0,4,0,4
  BF_TEST_WEAPON="Bower 20" BF_TEST_TRIGGER=2.0-2.08,4.0-4.08,6.0-6.08 BF_SHOT_LOG=1`): at
  `BF_CAPTURE_FPS=60` the press is seen at 2.03 ("raises the gun: the first round in 0.1 s
  (pc-transition-ready in-time)") and the round goes at 2.13, +0.10; the taps at 4 and 6 s fire
  in their own frame (4.03, 6.03). At 15 fps steps the first is seen at 2.07 and fires at 2.13
  (the next step); before #140 these were +0.20 and +0.03 at 60 fps. At 15 fps a
  `BF_TEST_TRIGGER` window starting on a frame's time (e.g. 3.0 s) can be seen only on the next
  step (3.067 s). An e40 Shrike Soldier on the test map (`BF_TEST_NPC=...,fight`) logs "raises
  the gun: the first round in 0.3 s (npc-transition-ready in-time)" as it takes Tex up, and fires
  0.87 s later (its opening delay).
* **Taps in the cooldown (#360), checked** (`BF_MAP=e34 BF_CHARACTER=1 BF_TEST_WEAPON=L-Shot-75
  BF_TEST_GOTO=-53.64,31.08,-53.64,31.08 BF_SHOT_LOG=1 BF_CAPTURE_FPS=60`, the rounds in the shot
  log): `BF_TEST_TRIGGER=3.0-3.08,3.5-3.58` one round (3.17; before, a second at ~4.2);
  `3.0-3.08,3.5-3.9` and `3.0-3.08,3.9-3.98` one; `3.0-3.08,3.5-4.6` two, 1.03 s apart (the
  hold across the cooldown's end), none after; `3.0-3.08` alone one, 0.14 s after the press
  seen; `3.0-3.08,8.0-8.08` two (the second raised again: 8.02, round 8.13); `3.0-8.0` rounds
  1.0-1.03 s apart; the L-Shot-50 (`BF_MAP=m03_a`) held 3.17, 4.87; MK-ASLT held every 0.13 s;
  the Bower's tap inside its cooldown (`3.0-3.08,3.2-3.28`) one burst and no click, its dry
  click only at a tap after (`BF_TEST_CLIP=1 BF_AMMO_LOG=1`, `...,5.0-5.08`: 5.03). Tex's
  berserk (`BF_CHARACTER=3 BF_TEST_WEAPON="Confed LZR-50;Jax-iC" BF_START_WEAPON=1
  BF_TEST_SKILL=2`, trigger `4.0-4.08,6.0-6.08,6.3-6.38,8.0-9.0`): both guns at 4.13, both at 6.03,
  at 6.33 only the second hand's (the gun in hand was cooling down, and fires nothing after),
  both from 8.03 when held.
* **The burst counter** is cleared by a reload starting (FUN_0022daf0), a dry click
  (FUN_002327f0) and a switch (FUN_0022dc00), as in the game.
* **The seed** of a shooter's fire loop is the demo's own hash, character x 7919 + NPC number x
  104729 (`play.rs` `shot_seed`), in wrapping arithmetic (#229): an NPC numbered above ~41,010
  overflowed a u32 and panicked a debug build. The seeds below that are unchanged, so the shot
  patterns are too. Checked by `cargo test --bin bf_play shot_seed`.
* **Bursts and pellets**. A trigger cycle is h_e4076713 shots, h_e6c60892 s apart. At 0 s apart
  (the Bower 20's 6) they all go in the same frame for one round and one report, and each after
  the first is turned from the one before (FUN_0022e1d0: the pellets walk away from the first)
  **in the world's frame** (#49): the aim d becomes Ry(a2) Rx(a1) d, a1 then a2 drawn uniform in
  +-h_e704fd69 degrees (6 for the Bower, 3.5 for the MNR-7's darts), about the world's x axis and
  then its vertical (FUN_0013e800 builds Rx, FUN_0001fa20 turns it by Ry, FUN_002334b0 multiplies
  the aim's frame by it, FUN_0001f9c0 takes the frame's -z: every instruction read for #49's spec,
  §4). So a heading along x keeps a shell's pellets nearly level (the x turn spins the aim about
  itself) and one along z scatters them up and down as much as across; across, every heading
  spreads the same. The demo turned the pitch about the gun's own right before: up and down as
  much as across at every heading.
  The round is taken on the cycle's last shot: FUN_0022fe00 excuses every shot while the
  counter is running, and only once FUN_0022f0e0 has cleared it (the 6th) does the clip drop.
  So the clip stays put through a burst and every shell is 6 pellets, the last of a clip too;
  the reload then starts 1 s later (the takes: Bower +1.0 s).
  bullet-type 4 is an instant ray whatever the bullet's speed (FUN_0022f1d0): the Bower's and
  the MK's 80 m/s aren't used.
* **Accuracy** (weapon+0x1dc). The accuracy block h_fb327295 {min, max, scoped cap,
  per shot, per second}: the current accuracy A is held in [min, cap] (cap: max, or the scoped
  cap in the scope), each trigger cycle takes "per shot" off, and each frame the cooldown is out
  it regains "per second" x dt (crouched: half the loss, a floor of min + 15, twice the gain;
  FUN_00222ac0 / 00222c10 / 00222b30). **Where it's used** (found for this ticket):
  FUN_0022e4c0, once a frame before the fire loop, turns each muzzle's aim about the three world
  axes by uniform random angles within +-S degrees (FUN_002229b0), with
  S = (100 - clamp(A - 1, 0, 100)) x 0.03 (FUN_000c4e10; the -1 is the holder's script bonus,
  unset: the squad's `accuracy-bonus` h_072837bb flag h_ec5337fb is false in the levels; some
  enemies have 75 or 90). A character a spawn-trigger made from a placement that sets
  h_ec5337fb takes its h_072837bb instead (#227, `Trigger::bonus`: the tutorial's Red Hand +25,
  S = (100 - clamp(A + 25, 0, 100)) x 0.03). So the MK's first shot spreads +-0.33 degrees and its 14th on in held
  fire +-2.3; the Minigun's +-1.2 to +-1.8 over a clip; the Bower's aim +-0.5 before its
  pellets' walk.
  * Against the takes: the clusters measured in screen units, read with a 45 degree vertical
    field of view, came out 1.5-2x tighter than this. The ~74 degrees vertical that the data's
    `<camera> fov="90"` would give, and under which the MK, Minigun and Foley matched within
    ~10%, isn't the game's view: the follow camera's is 70 across its 4:3 picture, 55.4 down
    (#126's live read, [camera.md](../camera.md)). Read at 55.4 the clusters are still about 1.2-1.6x
    tighter than this (1.5-2 / 1.27): the accuracy fit is unexplained again (for the weapons'
    tickets).
  * The Bower's "~2x too wide horizontally" (read from the tank's holes, where the pellets past
    ~+-7 degrees miss the 2.6 m cylinder and leave none) is settled by the walk's frame (#49, above):
    its takes' sparks at the 25 m cliff (take09, heading -170) spread 10.3 degrees across (sd 3.8)
    and 0.8 up and down, the world-frame rule's 9.3 (sd 3.5) and 1.7 (0.7-2.9), where the old
    gun-frame turn gave 9.3 up and down.
* **Damage falloff** (FUN_00230040 -> FUN_00223780). An instant hit's damage is scaled by the
  Damage's h_fb124e6c mode at x = 1 - distance / range: 1 x, 2 x^2, 3 full to half range then
  2x, else none. Only the Bower has one (2): 0.60 at 9 m of its 40 m range. It's applied to the
  squad's, the NPCs' and breakable scenery's hits. The rest of the take's ~1.3x on the trooper
  is the difficulty step and the struck hit part (#127): see **Damage** below.
* **Range in the holder's hands** (FUN_002324c0): a shot's ray and its falloff run over the
  bullet's range through the holder's skill (+0x54): unchanged, but x1.25 for Flint's rifles
  (WTYPE_RIFLE, on or off; #124, `skills::weapon_range`): the L-Shot-75's 180 m is 225 m in
  her hands.
* **Holes** (`play_fx.rs`, `HoleRequests`). Every gun shot that meets the world leaves its
  bullet's h_06a27365 decal (0.3 x 0.3 m holes and scorches, read as full sizes; the decal's
  life and fade, h_199870ec / h_fe4e1d82: 25 s fading over the last 5 for most, 60 s for the
  bio guns' green h_1ce6db03) laid on the
  surface it met: the map's collision triangle (`Arena::ray_normal`) or, on the flat test floor,
  the ground or a pillar's side. Up to 160 stay (a guess). A bolt's hole comes with its hit, on
  its sweep (**Bolts and their hits** below). An instant shot, as a bolt's sweep and a missile,
  passes through the level's invisible blockers and hinters to what's behind them (#298: the
  weapons' filter, PID mask 0xfffffff8, leaves them out; maps.md, "Blockers and hinters"), and a
  breakable object's own blocker no longer takes the hit meant for the object. A hole on a
  breakable object goes with it when it breaks (#301: scenery.md, "Breaking"), the hole of the
  shot that breaks it too: the shot finds the object with its hit (`Shot::breakable`), as the
  game's hit record holds it.
* **Muzzle effects**. The weapon's h_fd88830d effect type plays at its muzzle hardpoint once a
  frame in which it fired (the MK's orange flash and grey smoke); it replaces the plain flash
  quad on guns that have one. It takes the hardpoint's own frame (#73): FUN_00232060 attaches
  the effect object at the hardpoint. Every weapon's muzzle hardpoint has the barrel along its
  -z (unturned, or half a turn about y on the models built facing +z) and the flash emitters
  are turned -90 degrees about x to fire along it (the MK's sminigun_muzzle.emt), so the
  flashes now leave the barrel forward; turned +y along the barrel (before) they fired across
  it. One exception: the Psionic Blast's (ORIENTATION w x y z 0 0 0.707 0.707) puts its -z
  down the model's -y; its effect (seer_fball_mzl_purple) isn't checked against a take. The
  frame was found on the Therm Sweeper, whose back-blast dust (mtrail-dust, 4.3 m along +z,
  2 m down) hung 4.3 m above Tex and now lies behind the launcher, where the follow camera
  doesn't see it (the takes show none from behind Tex); by the game's emission count it's one
  brown card (see **Missiles**, the effects' count). An effect parented to the gun now starts on
  the shot's frame, placed from the gun's last placement (`ale_fx.rs`'s emit; it waited a frame
  before).
  * **Facing 3** (#73; `src/ale_fx.rs`, `FACING`): an appearance flagged AngleFadeOut (the
    demo's "perp") whose GeneralApp_FacingType is 3 is drawn as the game's particle vertex
    shader (0x3b8bf8) builds it: a quad stretched along its particle's motion and turned about
    it to face the camera. In the hardpoint frame, lying flat across its y, the sphere
    emitters' glows (the A10's, the Saryl-45's, the Jax-iC's) were edge-on lines; they're round
    again, as before it. Since #286 every appearance of every effect is turned by its facing
    type, faded edge-on where it's AngleFadeOut and stretched where it's motion-blurred
    ([maps.md](../maps/ale-blends-and-particle-facing.md), **Particle facing**): the MK's flash quads (sminigun_muzzle.app, facing 3
    without AngleFadeOut) run along the barrel from the hip, where they were billboards rolled at
    random.
  * **The light**: a muzzle effect type with a light-effect plays it as a lamp at the muzzle
    (one lamp for the effect, its newest particle's: levels.md "Effect lights", #272), as the
    blasts' lights are played (`src/ale_fx.rs`), in place of the demo's own point light
    (`HeldWeapon::data_light`); a gun without one keeps the demo's (`FLASH_LIGHT`, also a lamp
    since #272: colour (1, 0.75, 0.4), 6 m). Each kind of gun names
    its own: light_muz_minigun (h_08b5bbab, 0.2 s growing 0.28 -> 2.66 m) on the ballistic guns
    (MK, Minigun, Bower, Foley, Kman, L-Shots), light_muz_laser (h_1a96e11e, 0.2 s, 0.26 ->
    2.53 m) on the LZRs, light_muzzle (h_e7e8d51b, 0.24 s, 0.5 -> 4.6 m) on the Therm Sweeper and
    the Sweeper V, and _bio, _cutter, _sonic, _particle, _rail, _seer and _energy on the others;
    none on the iKhan-GPL. Its seed is made from the effects', so the gun's random stream
    (spread, frame jitter) runs as before it. It's lit from the shot's frame (`BF_SHOT_LOG`'s
    round, the effect and the light's first update share it) but draws from the next: a point
    light made that frame shows a frame late in the demo, where the demo's own light (an
    existing one turned up) showed at once. Not fixed.
  * **In the controlled character's own scope** it's drawn 3.8 m from the eye with its axis
    along the line of sight, where the take shows it (`SCOPE_FLASH_REACH`, `SCOPE_FLASH_AT`; the
    demo's rule): at the muzzle, a few tenths of a metre from the scoped eye and magnified, its
    sparks covered half the view. The scope-in code (0x1223da, FUN_001249b0) doesn't move or
    hide it, and nothing else that does was found. Since #286 it's turned with its barrel (its
    -z, the hardpoint frame's) along the sight, as the game's nearly is from the scope: its flash
    quads, facing 3 along their motion down the barrel, are seen end-on and fade, and the game's
    facing types draw it a soft round ball, as the take shows it. Before, it kept the turn it had
    before the hardpoint frame (+y along the sight) with the demo's perp rules, a star; with the
    facing types and that turn its flash stood up from the ball as a tall flame. No light there, the
    demo's old rule and a choice: from the gun under the eye the data light washed the whole
    scope view orange for a frame (the scoped MK pitched down, +29 red on average), and no take
    shows the game's light in the scope.
    * The take (63/take01 at 28.77-28.85 s, the bright pixels that changed, 640 x 480 units): a
      soft orange ball, centroid 95-104 right of the crosshair and level, 124 x 144, its left
      edge (x 344) clear of the crosshair (303-336), gone in ~0.1 s.
    * Ours (frames 16-23 of `-- --test BF_NO_SQUAD=1 BF_TEST_WEAPON=MK-ASLT BF_TEST_FIRE=1
      BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12`, #286): a soft ball, centroid (91-94, 14-17)
      at the old aim point [92, 18], ~95-105 units across by the pixels 40 brighter than frame 10
      and over 120 (the take's own centroid (92-103, -7..2) by that measure, over 90, its lit wall
      round it): `SCOPE_FLASH_AT` moved to [98, 3], where it measures centroid (97-99, 2-4),
      91-111 x 99-106 (frames 16-21). Before #286: centroid (98-102, -3..+4), 145-155 x 78-97, a
      star.
    * In the scope the effect leaves out its quads lying along its barrel (facings 0 and 2 whose
      normal is more than 60 degrees off it, `AleEffect::hide_flat`, the demo's rule from when
      they were perp quads, seen from the eye as bright ellipses; the MK's flash has none), and
      since #286 its alpha-blended smoke cards: thrown down the barrel from 3.8 m out, the
      L-Shot-75's sniper_asmoke (0.23 -> 0.64 m) hung over the whole zoomed view as a grey veil
      (`BF_MAP=e40 BF_CHARACTER=1 BF_TEST_WEAPON=L-Shot-75 BF_TEST_GOTO=-16.16,152.62,-16.66,152.52
      BF_CAMERA_PITCH=-0.05 BF_TEST_SCOPE=2,1.5 BF_TEST_TRIGGER=3.0-3.08`), where #62 take12
      (f192-f230) shows smoke only along the beam, and #63 take01's flash is gone in ~0.1 s with no
      smoke after it. A demo rule, as the scoped placement is.
* **Muzzles** (#246; `bf/weapon.rs`, `WeaponDef::muzzles`). The muzzle block's h_e41cb542 is a
  list of hardpoint names (the weapon data's vector at +0x140), and the game uses every one: the
  muzzle effect is attached at each (FUN_00232060), each one's aim is turned every frame
  (FUN_0022e4c0), and a shot leaves from the muzzle the weapon's index (+0x1fc) points at, which
  then moves on to the next, back to the first past the last (FUN_0022f5e0;
  `WeaponDef::next_muzzle`). The muzzle effect that follows the shot is the one at the index as
  moved (FUN_0022f170 reads +0x1fc after it): the flash shows at the barrel that fires next.
  Every hand-held gun names one, MUZZLE (h_ecc0aa78), so its index never moves and the demo's
  guns fire from it as before (`WeaponDef::muzzle`); the four turrets (object-type 29: e05, e16,
  m03_b, m08_b, m09_b/c/x, m14_a-d) name two, h_f9eed613 and h_e2e787a9, of which the reader
  kept only the first; 14 definitions without a model name none. The demo has no turrets yet,
  so the turn is read from the code and not seen. `BF_DUMP_WEAPONS=1` lists each gun's muzzles
  and then the map's definitions with more than one, each found on its model or not (e05:
  `h_13e090b2 ... muzzles [f9eed613 ok, e2e787a9 ok]`).
* **Tracers and beams** (FUN_002317e0 -> FUN_002312e0, #237; play.rs `projectiles`, `beam_line`;
  ale_fx.rs `load_beam`). An instant ray carries its flight effect only when its counter is 0,
  which then restarts at h_eeb9e75a: the MK and the Kman every 3rd shot (the first among them),
  the Minigun every 4th, the rest every shot. The effect is placed once, at the muzzle, turned
  along the ray (FUN_0001d0c0's frame: its -z along the ray, its x the world's up x that, Bevy's
  `looking_to`), with the user parameter min(d, 200) / 200 for the ray's length d: where it hit,
  where a Rail stopped (§ Ricochet and pass-through), or its range when it met nothing. The
  effects are made for that: their emitters, turned -90 degrees about x, throw or carry their
  particles out along the ray (tracer.emt's at 113.6 m/s; tracer_snipe_beam.emt's offset runs
  out to 200 m in 0.38 s, rgun_beam.emt's to 100 m in 0.24 s), and their kill field (class
  h_027b6cd9, `CLASS_KILL_FIELD`: a plane facing back along the ray, past which a particle dies)
  stands 200 m x the parameter out (tracer.fld, rgun_s_kill.fld; tracer_snipe_kill.fld 179.4 m
  x it). So the MK's, Kman's and Minigun's tracer, the L-Shots' trail and the Rails' beam end
  where the ray did. Fired from the scoped eye, the beam starts 0.25 m right of it and 0.1 m
  below and is aimed at the ray's end, so in the scope it comes in from the lower right. A beam's
  appearances turn by their facing type as the game's vertex shader does (0x3b8bf8: facing 3
  along the motion, the tracer's streak and the Rails' side arcs, which stand on their particles,
  shape 3; facing 2 square to it, the L-Shots' rings and smoke cards; facing 0 in the effect's
  frame, the L-Shots' tracer_snipe_orient rings square to the ray; every effect's since #286,
  [maps.md](../maps/ale-blends-and-particle-facing.md) **Particle facing**), and its particles start
  where the game's spawn for their emitter puts them (`Compiled::game_spawn`, #67: the Rails'
  rings, rgun_perp_s.emt, a game cube whose height grows 116 m a second with OffsetHeight set
  (code-map 0x64e30), are strewn along the ray from the muzzle out, where the demo's spawn had
  them all within a few metres of the gun, a tube of rings round the camera). Before, the effect was
  spun so its emitters' own turn threw it down into the ground at the muzzle, a tracer's
  camera-facing streak could cross the whole scoped view, and nothing stopped it at the hit (and
  before #115 it flew straight up, the thin vertical line over the Minigun). Checked against the
  takes:
  * The MK on the test map (`-- --test BF_NO_SQUAD=1 BF_TEST_WEAPON=MK-ASLT BF_TEST_GOTO=0,-8,0,-8
    BF_CAMERA_PITCH=-0.12 BF_TEST_FIRE=hip BF_SHOT_LOG=1`, the pillar 10 m out): on every 3rd
    round a short orange streak at the strike end, gone into the pillar within 0.1 s, as #63
    take01 shows it (f325-f328 of the take at 60 fps: the orange streak below the reticle,
    shrinking into it); the Minigun's on every 4th the same (`BF_TEST_WEAPON="RVG50 Minigun"`).
    Scoped (`BF_TEST_FIRE=1`) it comes in from the right to the reticle. The phase: a new weapon
    object's counter starts at 0 (FUN_0022d330 zeroes +0x290 at 0x22d419; only FUN_002317e0
    writes it after), so a fresh gun's first round carries the tracer, as the demo's does; #63
    take01's first tracer on its second tap is the snapshot's counter (its MK had fired 3k + 2
    rounds before it, the only start the code allows), not a different rule (#287).
  * The L-Shot-75 at e40's cliff 25 m out (#62 take05's spot: `BF_MAP=e40 BF_CHARACTER=1
    BF_TEST_WEAPON=L-Shot-75 BF_TEST_GOTO=-16.16,152.62,-16.66,152.52 BF_CAMERA_PITCH=-0.05
    BF_TEST_TRIGGER=2.0-2.08`, 27 m to the hit): a thin pale line from beside Flint's head to just
    short of the reticle for ~0.15 s, then a grey smoke trail along it, gone by ~0.5 s, as take05
    f168-f196. Scoped at level 2 (`BF_TEST_SCOPE=2,1.5 BF_TEST_TRIGGER=3.0-3.08`): the line from
    the lower right to short of the reticle, with white rings along it, as take12 f195-f209.
  * The Rail RVR at e34's gate 10 m out (#67 take01's spot: `BF_MAP=e34 BF_CHARACTER=2
    BF_TEST_WEAPON="Rail RVR" BF_TEST_GOTO=-52.76,31.02,-52.9,31.5 BF_CAMERA_PITCH=-0.08
    BF_TEST_TRIGGER=2.0-2.08`; for the HUD on e34 add `BF_NO_SQUAD=1
    BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, the start's letterbox off, camera.md, the
    tester's): the spark at the hit, red-violet arcs from it down toward the gun
    turning blue over ~0.15 s, then blue sparks at the hit, as take01 f19-f41. The ray goes
    through the gate and stops at a wall 38.3 m out, the beam's length.
  * **The Rails' beam on the game's emitter clock** (#287; ale_fx.rs `GAME_COUNT_BEAMS`,
    `Batch::ages`, `Pair::fade`, `Pair::facing`). The Rails' flight effect h_0969d0fb plays
    rgun_perp_s (three emitters: the rings rgun_perp_s.emt, the side arcs rgun_side_s.emt and the
    muzzle ring rail_mflash_ring_Cone.emt, the case-kept hash h_f7b6d4a5 that the ALE tool's
    lower-case lookup misses), rgun_side_s (empty) and rgun_beam (the ribbon). rgun_perp_s now
    runs as FUN_00065800 runs every emitter:
    * once a 30 Hz game frame, its emitter's time stepped first (0x2a5b10) and every curve read
      at the stepped time: the frequency, life and pressure (0x6584b-0x658ae) and the cube's
      width, height and depth, which 0x64d60 reads at the time plus the step;
    * the count trunc(elapsed x frequency), plus the emit-count keys passed (0x6591b-0x659d6), the
      fraction lost (`game_count`);
    * each batch of n spread over the emitter's time since its last batch: particle k is born
      elapsed x (1 - k / n) old, the newest new (0x65a87-0x65a99, 0x65c1a-0x65c33, the loop
      0x65ca2-0x65ce5), and the cube's spawn moves it on along its way by its pressure x that age
      (0x64e30: base + offset + direction x pressure x age, 0x65001-0x65075); and placed back
      along the emitter's local move since then (0xa0c60 maps the last place into the
      emitter's frame), the demo's offset spread.
    So the first rings are 16 strewn over the cube's [0, 2 x 7.1 m) one game frame in, where the
    demo strewed its render frame's (16 at 30 fps, 32 at 15) over [0, 2 x 3.2 m) at time 0, and
    the side arcs start where the side emitter is then, 3.3 m out along the ray (its offset runs
    100 m a second), as #287 take05 shows them from the side (f011 of 3.0 s on at 60 fps: the
    blue ribbon from the muzzle, the red arcs from about 3 m to past the Heavy). The muzzle ring's
    two emit-count keys (12.8 at 0.004 s, 12.7 at 0.12 s) make 12 glows each, where the demo's
    count made one. The ribbon (rgun_beam) and every other gun's tracer and trail keep the demo's
    count: under the game's, rgun_beam's first three points would lie 4.6-13.9 m out where
    take05 f011 has the ribbon from the muzzle (how the beam appearance h_1f55f13e joins its
    ribbon isn't traced), and the MK's tracer.emt would make eight streaks where it was checked
    with one.
    * **GeneralApp_AngleFadeOut** (FUN_002a1a50 at 0x2a1c9d, the vertex shader 0x3b8bf8): a
      beam's AngleFadeOut quads (the Rails' rings, the muzzle ring's glows, the L-Shots' smoke
      cards) fade by clamp((|cos| - 0.0872) x 3.9237, 0, 1) squared of the angle between the quad
      and the eye: whole beyond 20 degrees from edge-on, gone within 5 (in four steps of material
      then; since #286 for every effect's quads, in 32 steps of the quad mesh's vertex alpha,
      [maps.md](../maps/ale-blends-and-particle-facing.md) **Particle facing**). From the side the rings are gone along the middle
      of a beam and show as short ticks near its ends, where the view isn't square to it, as
      take05's few ticks; from behind the shooter they face the eye and nothing changes.
    * **Facing 2** (0x3b8bf8): a beam's facing-2 quads lie square to their own particle's motion,
      where the demo laid them square to the emitter's axis. The same for the rings and the
      L-Shots' smoke (thrown within 0.3 degrees of the ray); the muzzle ring's glows, thrown 73.7
      degrees off it, now stand round the muzzle edge-on to the ray. Their height runs along the
      game's side vector (0x2a14b0: the particle space's y, x when the motion points up), turned
      by their Rotate (#286; before, along whatever the turn from +z gave).
    * A curve's ends (FUN_002dfc10): its flags' low four bits say what it does before its first
      key (0 holds it) and the next four after its last (1 repeats the keys' span from the first
      key, 2 repeats it adding the change, 3 runs it back and forth, 4 goes on along the end key's
      tangent); between keys it's a Hermite curve from their tangents (FUN_002df650,
      FUN_002df730). The demo repeats a flagged curve from 0 over its last key's time
      (`Curve::at`) and runs straight between keys. rgun_perp_s's life (48: back and forth) and
      tracer_snipe's (16: repeat) come within a few hundredths of a second either way.
    * **The other beams a game frame at a time** (#287; ale_fx.rs, the beam ticks in `emit`).
      The beams on the demo's count (the Rails' ribbon rgun_beam, the MK's, Kman's and Minigun's
      tracer, the L-Shots' trail) carried their rate's fraction over each render frame, so at 60
      frames a second the ribbon's first render frame held 0.9 of a point, and its first point
      left a frame later, 7 m out (the tester's 60 fps side capture: the ribbon ~0.12 s late); at
      30 it left the muzzle. They now step a game frame at a time, as FUN_00065800 runs, each
      frame's batch read at its start: at any render rate the shot's first game frame puts the
      ribbon's first point at the muzzle and the next two a game frame out along its emitter's
      move (`BF_ALE_LOG`: "rgun_beam pair 0: t 0.000 ... emits 1", then "t 0.033 ... emits 2"),
      as take05 f190 shows it from the muzzle. At 30 frames a second nothing changes (the cliff
      and the L-Shot-75 recipes, pixel for pixel against the build before); at 60 the L-Shot's
      trail and the MK's tracer look as before (the trail's smoke cards a little moved, the MK
      39 pixels in 160 frames).
    * Checked from the side, take05's staging on e34 (`-- --test BF_MAP=e34 BF_CHARACTER=2
      BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_TEST_WEAPON="Rail RVR;Foley
      356 Tact" BF_TEST_GOTO=-40,22,-50,22 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.14
      BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-50,22" BF_TEST_NPC_HP=1.0,1,3000
      BF_TEST_INSTANT_KILL=0 BF_TEST_FREECAM=1.0,-44.4,30.6,6.5,180,-5 BF_TEST_TRIGGER=2.0-2.08
      BF_CAPTURE_FPS=60 BF_ALE_LOG=1`, frames 129-147): in the shot's first game frame the blue
      ribbon from the muzzle and the red arcs from a few metres out to past the Heavy, as take05
      f190; "rgun_perp_s pair 0: t 0.033 ... emits 16" each game frame to 0.233 s; the arcs turn
      blue and run out along the ray. From behind at take06's spot (`-- --test BF_MAP=e40
      BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR;Foley 356 Tact" BF_TEST_GOTO=-16.2,152.68,-16.69,152.58
      BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=2.0-2.08`, frames 64-78 at 30 fps;
      with `BF_TEST_FREECAM=1.5,-15.15,28.16,153.33,79.6,-3.2` the camera stands where take06's
      does, 2.2 m behind the muzzle, 0.6 m left of it and 0.5 m above, its picture's muzzle at the
      lower right) the first game frame's rings run out along the ray from the gun: five of 20-45
      px (1280 x 720) 2-8 m out, as take06 f168's four of 15-60 px.
  * Not matched (#287, open):
    * **Too many and too bright rings after the first game frame.** At take06's camera the demo's
      rings make a tube of ~10 large white-blue rings from the second game frame on (frames 66-69
      of the free-camera run), where take06 f170-f175 shows faint ring outlines behind the red
      arcs; from the side its rings show past the Heavy, where take05 shows none (the spec's
      erratum, §2: ~41 px/m there, the core 0.22-0.24 m, the ring ticks 0.3-0.6 m). The count
      (16 a game frame), the cube's strew, the sizes (0.165-1.0 m across) and the lives
      (InitLifeSpan 0.05-0.17 s, back and forth) are the data's through FUN_00065800 and 0x64e30.
      What the demo draws differently: the added rings' faint colour (0.27, 0.35, 0.72) is summed
      as linear light (#104 / #165's blend: grenades.md "The look and the additive blend"),
      brighter than the console's sum in stored values on a dark ground; and the light of the
      shot, whose luminance rise is 1.9 times take05's, over 8 frames against its 6 (the
      tester's measure). Past the Heavy the take's ray ends sooner: its hit log has the 40 HP
      object 7f1daa00 right after the Heavy on every shot, twice on the first (n1 1 spent: the
      ray's end, its far kill plane), most likely the Heavy's gun (a weapon's `<combat-target
      hitpoints="40">`); the demo's rays don't meet a held gun (not in scope), so its beam runs on
      to the terrain or the full 90 m, and its rings past the Heavy, 25-40 degrees off square,
      stay whole.
    * **The arcs stay red longer.** In both takes the side arcs are red only in the shot's first
      one or two game frames (take05 f190, take06 f170-f172) and blue after along the whole beam:
      from the side gone after 3 game frames (take05 f196), from behind after ~8 (take06 f185,
      blue arcs at the far end to the last). The demo adds a red arc each game frame for 5 (the
      side emitter's 45.6 a second until 0.19 s, as FUN_00065800 counts it), so red shows for ~5
      game frames, and from the side the beam lasts ~8 where take05's lasts 3. What stops the side
      emitter's later arcs, or turns them blue at once, isn't found: FUN_00065800, the particle
      update 0x2a17b0 (each particle aged once a game frame), its draw FUN_002a1a50, the effect
      object's tick 0x21b7a0 and its slot update FUN_0021a850 (each node updated once a game
      frame, through the ALE manager's 0x2a5460) were read for it, and rgun-mkill sweeps out too
      slowly by its curve (0.31 m behind the muzzle to 8.7 m out over 0.04-0.46 s). A code
      question, or a live count of the side emitter's spawns (0x64e30) in a Rail shot.
    * **From behind in the demo's own camera, a bulb of rings** round the beam's start (#67
      take01's gate, take06's cliff): the gun sits behind Hawk's head (the muzzle 20 px right of
      and 55 px below the reticle at e40, take06's 135 and 178), so the beam runs down the line
      of sight, its rings nest round the reticle and its arcs, seen end-on, vanish. That's the
      follow camera's and the stance's (camera.md), not the beam's: at take06's camera (above)
      the beam runs from the gun at the lower right to the reticle, as the take's.
    * A white glow at a Rail's hit on a body (light_hit_rail's point light, 0.68-4.2 m, 2.83 m back
      along the shot, with wmat_fleshit_rail's white flares: every flesh row of the world
      materials, 21 and 23-26, gives the Rail's ammo-type 5 h_15bd992f) where take05 shows the
      Heavy's green splash only; and at a wall from behind, the small far rings and railhit_s's
      early slow flares (0.95 s lives, 0.08-2 m/s for the first 0.08 s) pile into a white ring at
      the hit for ~0.2 s, where take01 shows blue sparks.
    * In the L-Shot's scope the demo draws more rings than take12's three or four, and a smoke
      cloud far denser than its faint haze, along the line to the reticle where take12's gathers
      at the lower right near the gun. No new footage of it: its trail keeps the demo's count
      (`GAME_COUNT_BEAMS`) and its smoke cards, AngleFadeOut quads, face the scoped eye.
* **The Minigun's spin** (#68; `src/bin/play_minigun.rs`): its item set is IFSET_MINIGUN
  (function-type 16), whose per-frame hook FUN_00147f90 (vtable 0x39b8f8, the item-set factory
  0x146920; the analyst's reading, todo/weapons-plan/answers-batch1.md §12) keeps a spin level:
  1 at once while the gun's cooldown runs with its fire flag set, else falling 1.0 a second
  (FUN_00147e50). At 1 the loop h_0e8038d2 starts, once, fading in over 0.1 s (FUN_00147bc0,
  FUN_00147c20: 2D for the player's holder, at the gun for another's); below 1 the spin-down
  h_f00b38e4 plays once in its place (FUN_00147d70), and the loop can't start again until that
  ends (its flags clear then, FUN_00147b90: a reading). Both are in the common bank. The item's
  attached effect while it spins (FUN_00147ed0) is empty for the Minigun. Holstered, switched or
  dropped, it all stops (FUN_00148030). The barrel turns at 28 rad/s x the level (the demo's
  rate; the game's barrel turn isn't traced), where the demo eased it up and down on its own
  before; the spin starts with the first round (the demo's raise before it is held in the gun's
  cooldown, which the game's isn't). Nothing changes the rate: 15 a second from the first round,
  no spin-up (#68 take01).
* **Casings** (FUN_0022eee0 -> FUN_00231b10). Each trigger cycle owes h_fdc93b33 casings; each
  whole one throws the h_19b21bf5 object (the casing h_16415157's model) from the h_ea1abbbe
  hardpoint, 5 cm back along its z, at 2-3 m/s along its z turned a quarter turn about the
  vertical, plus up to 0.5 m/s on each other axis. MK, Minigun, L-Shot and Foley; the Bower has
  the hardpoint but no casing object. Their 2 s life, gravity and bounce are guesses.

* **Two guns at once** (Tex's berserk, [skills.md](../skills/texs-berserker.md)): the fire loop steps each gun as
  its own weapon object in the same game frames (`FireState::run` over `Gun`s: its cooldown,
  burst counter, empty cycles, muzzle effects and casings per hand; accuracy and tracers per
  weapon as before), both on the one trigger. Each shot carries its own gun's Damage,
  damage-type and ammo-type to the body it hits (it used to take the gun in hand's for every
  shot of the frame). One gun's fire is unchanged: against main, the autopilot's shot and ammo
  logs as Tex and as Brutus, and the LZR-50's held fire, taps and dry cycles on the test map
  (`BF_TEST_TRIGGER=1-4,5-5.08,5.5-5.58,6.5-6.58`, with and without `BF_TEST_CLIP=6`), are the
  same line for line once each pellet's new `hand: 0, ` is taken out of the shot log, and the
  test map's frames pixel for pixel.

Test hooks: `BF_TEST_WEAPON=<label or hex>` (e.g. `"Bower 20"`, `"RVG50 Minigun"`, `0c3db625`)
puts that weapon in the controlled character's slot `BF_START_WEAPON` (default the first) if the
map's data has it (the test map has every hand weapon; a label two definitions share takes the
one that isn't WTYPE_DATA, so `"iKhan-GPL"` is the gun h_e9be70f0 and never #147's vaporise
blast, #55); a list, `<a>;<b>`, fills the slots in
order from the first (#35's Tex: `"Confed LZR-50;Jax-iC"`). On a level whose spawn-triggers start
the squad (e34, e40, the tutorial: #220) the character carries its spawn's template weapons, and
the hook now sets those too (#72): it did nothing there before. `BF_SHOT_LOG=1` now also prints each
pellet (its hand, frame, spread, walk, tracer), the gun's accuracy and, last, who fired it (as
the combat log names it, #227); `BF_TEST_FIRE=hip` fires
without aiming (no scope on a gun that zooms).

Verified on the test map (`-- --test`, `BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12
BF_TEST_FIRE=1 BF_SHOT_LOG=1`, the pillar 10 m ahead) and sdm_e34:
* Minigun (`BF_START_WEAPON=1`): 2-3 frames a round, 12.3/s over 66 rounds; a tracer on every
  4th; accuracy 59 -> 10 over the clip. Seen from 13 m (`BF_TEST_FREECAM=4.4,1.5,-0.2,-4.7,..`)
  its holes span ~+-10 units up and down, the spec's cluster (+-10).
* MK-ASLT (`BF_TEST_WEAPON=MK-ASLT`): 4 frames, 7.5/s; a tracer on every 3rd; scoped 5-6
  frames (5.6/s). The game's MK holes after the take (63_zoom) spread ~+-26 x +-18 units, about
  the +-2.3 degrees the code reaches in held fire.
* Bower 20 (`BF_CHARACTER=0 BF_START_WEAPON=1`): one round and one report per shell, 6 pellets
  walking up to ~15 degrees from the first; holes on the pillar and on sdm_e34's walls. The
  world-frame walk (#49; `BF_TEST_WEAPON="Bower 20" BF_SHOT_LOG=1`, the trigger held, each shell's
  six directions from the log): across, 9.3 degrees at every heading (24 shells each way; over 77
  facing +x 9.45, sd 4.1, the quantiles p10-p75 within 0.3 degree of the rule's 5.3 / 6.5 / 8.7 /
  11.2); up and down 0.8-0.9 facing +x (the rule 0.7), 9.6 facing +z (9.2), 2.2 at e40's 25 m spot
  (take09's, `BF_MAP=e40 BF_TEST_GOTO=-15.2,152.3,-15.69,152.21`, heading -167: the rule 1.7 at
  -170, the take 0.8); the cadence, the round per shell and the reload as before. (Its
  muzzle effect along the barrel in a side view, `BF_CAMERA_DISTANCE=2.5 BF_VIEW_YAW=1.3`, was
  #115's check, before #73 put the muzzle effects in the hardpoint's frame: see **Muzzle
  effects**.)
* Casings land to Tex's right on the floor; the muzzle smoke and flash at the Minigun's barrel.
* With `play_ammo.rs` (#114): the Minigun reloads after its 80 rounds and fires on; the Bower
  with `BF_TEST_CLIP=2` fires two 6-pellet shells, reloads 1.0 s later (reserve
  80 -> 68) and fires full shells again; with `BF_TEST_RESERVE=0` it clicks dry about once a
  second instead.
* The main game on sdm_e34 (`BF_AUTOPILOT=1`, 340 frames) is pixel for pixel the same as
  before until the autopilot's first shot.
* **The compare round** (#63 MK-ASLT, #54 Foley 356 Tact, #68 RVG50 Minigun, #50 / #51 / #52 the
  Confed LZRs; their combined spec `todo/63-mk-aslt/spec.md`), on the test map (`-- --test
  BF_NO_SQUAD=1 BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12`, held fire from `BF_TEST_CLIP`;
  `BF_SHOT_LOG`, `BF_AMMO_LOG`, `BF_SOUND_LOG`):
  * Cadence: the MK 4 frames a round (7.5 a second; the takes 6.9-7.3, xemu's slower frames),
    the Foley 10-11 (2.89; the takes 2.79-2.93), the Minigun 2-3 (12.3; the takes 11.1-11.8), the
    LZR-10 16 (1.88; 1.76-1.90), the LZR-23 12-13 (2.45; 2.34-2.49) with one shot a cycle (its
    burst count 1, the delay 5 unused), the LZR-50 8-9 (3.53; the takes' mean 3.53). A tracer on
    the MK's every 3rd round, the Minigun's every 4th, the Foley's every round; a bolt on every
    LZR shot.
  * Reload (the clip set small, the trigger held): the reserve drops 0.07-0.14 s after the last
    round (the cooldown's end; the takes 0.1) for the MK and the Minigun, 0.33 s for the Foley
    (0.4), and the clip is full 1.00 s after that, the magazine-in of Tex's and Hawk's reload
    clips (the takes: Hawk 1.0; Tex 1.1 and 1.2, within the 10 Hz log's 0.1 s).
  * The LZRs recharge a round every 2.00, 1.27 and 1.00 s (the takes 2.006, 1.266, 1.0: 1.25 s
    lands on the 30 Hz frame after it).
  * Damage, on e40's Shrike Soldier frozen 9 m out (`BF_MAP=e40 BF_TEST_GOTO=-17.21,152.02,-17.70,151.93
    BF_TEST_NPC='0.5,shrike soldier@e40,enemy,frozen,-26.08,150.46' BF_TEST_NPC_HP=1.0,1,300`,
    `BF_COMBAT_LOG`): every hit of the six guns (78) inside the roll x the part's factor x 1.4.
    The takes' m03_a trooper (BALLISTIC x0.8) isn't loadable on the test map.
  * The Minigun's spin (`BF_SOUND_LOG`): the loop with the first round, the spin-down 0.1 s after
    the release, a tap's round spinning it too.
  * The scopes (the MK's 2, the Foley's 3: one level each for everyone but Flint): § The scope.
  * Not done: the Foley's silence (its noise radius 10: no AI hearing yet); the MK's scoped held
    rate (no take). (The LZR bolts fly since #136: § Bolts; the tracers and beams are drawn as
    the game places them since #237: **Tracers and beams** above.)
* **The Kman Auto** (#60; `todo/60-kman-auto/spec.md`), Hawk (`BF_CHARACTER=2`) with it: nothing of
  its own to build. Held fire 5-6 game frames a round, 5.38 a second (the takes 5.18-5.62: 6 a
  second is exactly 5 frames, so the frame-time wobble splits 5 and 6), the same scoped (its
  scoped rate is also 6); the accuracy from 90 down 10 a round to its floor of 40; a tracer on
  the first round and every 3rd. From a clip of 3 (`BF_TEST_CLIP=3`, e40's take05 spot) the
  reserve drops 600 -> 565 0.2 s after the last round (take02: +0.2), the clip is full at her
  light reload clip's magazine-in 1.0 s later (the Foley's run above times it by its reload sound;
  take02: +1.0 s), and she fires again 1.33 s after the drop, at the clip's end. Its scope (§ The
  scope): one level for Hawk's skill, 2x, the frame with the gun's 32.7-unit crosshair in the
  middle, in in 0.15 s; an exit press 0.3 s after entering waits for the lockout; the shared
  in / out sounds 135ec2ce / f9ffc972. On e40's Shrike Soldier frozen 20 m out (frozen, 300 HP)
  14 hits of 22 rounds: x1 parts 2.73-5.53 (the rule 2.45-6.65), forearms and calves 1.33-2.99
  (1.22-3.32); take08's at 25 m were 2.0-6.3, none on a forearm. Its tracer is the MK's
  (§ Tracers and beams, #237). Not done: hits at its 50 m range and scoped hits (no take with the
  aim verified).
