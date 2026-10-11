# Interactive scenery

* **Interactive scenery** (`src/bin/play_scenery.rs`, issue #85; the test map's object tool
  puts them down too, see [testtools.md](testtools.md)). Placed game objects whose
  type has a debris list (objecttypes `h_197caf14`, 467 types; `Game::breakable`) break when
  their hitpoints run out. On sdm_e34 that's 38 objects: 23 radiation barrels (h_e04e5a0e, 1 hp),
  6 supply crates (h_09a6856d, 25 hp), the missile rack (h_fbdcd828, 1 hp) and 8 others. Footage:
  `todo/85-interactive-scenery/` (29 xemu takes with 20 Hz HP logs) and
  `todo/interactive scenery objects.mp4`.
  * **Data** (`ObjectType`, `Debris`, `AreaDamage` in `src/bf/character.rs`):
    * the type's `combat-target` hitpoints and its own `h_142be76f` damage-type factors (these
      three: Type 7 x5, 10 x10, 2 x0.25, 3 / 4 / 9 x0);
    * the debris entries `<h_19c8df19 archetype-name h_1c1e17fe h_1b6a0ede h_f2e4e1a3>`
      (parser FUN_00187fc0: type, main, delay, life);
    * an effect type's `<h_fb0a5f1d><Damage amount Type h_ed582b3c h_f724cb8c duration range
      falloff>` (FUN_00191170);
    * the archetype's `<h_e99750a9>` point (`Game::archetype_centre`), read as its physics
      body's centre: where the object is, for its effects and blasts. The barrel's is 0.435 m
      up. With the origin on the ground instead, the barrel's burst was half buried and came
      late and faint (take26 has it full on its first frame). An inference;
    * the level's `<blocker object-instance>`: the object a blocker belongs to.
  * **Damage** (FUN_002232d0): value x the type's factor for its damage-type.
    * Shots: anyone's shot that stops on an intact object's collision
      (`Arena::ray_breakable`) deals its weapon's Damage min..max when it gets there
      (`Shot::damage`).
    * Grenade blasts: Damage max falling to nothing at the radius, measured to the object's
      centre, on the frame the characters take a Frag's (play_grenade.rs pushes an
      `ObjectBlast`). The Energy and the Sonic are dealt the same plain way, though their
      characters' damage comes with their bolts and ring: a simplification. The Gas cloud does
      nothing to objects (Type 4: x0 on these).
  * **Breaking** (FUN_00157260 on message 0x4e, then the countdown FUN_0015af20 each frame):
    * a countdown starts at the list's longest `h_1b6a0ede`;
    * each frame the first queued entry whose delay is at least what's left spawns, one a
      frame;
    * on a frame with none due and the countdown at 0, the object goes.

    So the longest delay comes first: the rack's effect (0.2) at once, its pieces and stand from
    0.2 s, then the model. That matches the takes' order (light and fireball while the missiles
    are still drawn, then the model goes; take13 has 0.067 s between them, the first capture
    ~0.17 s). A barrel's pieces come on the first frame, its effect on the next.

    The object's collision and blocker stop blocking at once. What it leaves behind blocks from
    then on (`Arena::set_broken`; its triangles are in the arena from the start, switched off).
    By the entry type's object-type:
    * 0, a compound: each part flies off as a loose tumbling body (play_pickups.rs' rigid
      body). A crate's go out from the hit, along it and up. An explosive object's go out from
      its centre by (1.2 - d / range)^2, the area push's shape (FUN_00224a90), and only a little
      up: the barrel's go low and fast, the rack's missiles (above its centre) up. The speeds
      are fitted to take25 / take26. The parts shrink away after 1.2 s. That time is a guess:
      the takes lose them out of frame, and the first capture has them gone after ~0.8 s;
    * 2, a game object: stays where the object stood (the rack's stand h_f77cc3a0);
    * 17, an effect object: its ALE effects and light effect at the object's centre, its sounds
      (barrel 14295eb1, rack 14d0b600, crate fa344138), and its damage areas. Loose pickups and
      debris near it are thrown (`pickups::Blasts`).
  * **Damage areas**, the effect object's own code (FUN_0021ade0): starting 0.1 s after the
    effect (the takes' first HP loss) for `duration` seconds.
    * Who: every target within `range` of the object's centre, measured in 3D to the
      character's point 1.45 m above the feet, with a factor above 0 for the Type.
    * How much: amount x dt x their factor each frame. Falloff 1 gives x (1 - d / range);
      2 gives (1 - d / range)^2 x (amount x dt)^2, as the code has it; 0 and 3 (the rack's
      HALF_LIFE) deal it in full. So the barrel's 80 x 0.5 = 40 HP (take29 on sdm_e34: 39.4),
      the rack's 200 x 0.5 = 100, flat at 4 and 7 m as in takes 08 / 09.
    * The 1.45 m point is a fit: the barrel reached Tex at 2.7 m along the ground but not at
      2.9 m.
    * No line of sight: FUN_0021ade0 sends no ray. The weapons' blasts (FUN_00224a90) ray-test
      the characters they reach, not other objects. So a barrel hurts through a wall, as the
      code does.
    * The player hurt by an area gets the red tint.
    * The breakable objects in range take it too, times their factor: the rack (Type 10, x10)
      sets off the barrels 4.4 m and 5.3 m from it on its first tick (take13, campaign e34). The
      barrel's (Type 3, x0) does nothing to barrels or crates.
    * A Sentry that's down goes off inside a grenade blast landing, on any map, or inside a
      damage area at work, if its own combat-target takes that damage-type (h_e5f1f063: 1 hp;
      Type 3 / 4 x0, so not in the barrel's cloud). This is the demo's choice; no take shows a
      Sentry in a blast.
    * A breakable object in front of a Sentry shields it from shots: the shot's length comes
      from the arena, which holds the intact object's collision.
    * `h_ed582b3c` is the push the damage message carries, scaled by ((1 - w) x 0.4 + 0.6).
      w is world +0xc58, a load measure FUN_000d9090 recomputes every second (0 in
      multiplayer). It scales pushes, not damage.
    * The campaign's squad took ~0.58 of a blast, against SDM's 1. That isn't traced; the demo
      uses 1.
  * **Not done**:
    * intact objects aren't pushed (the takes: they never move, they break);
    * the objects' `h_f2990a77` light lists aren't removed (empty on every sdm_e34 object);
    * loose pickups or scorch decals resting on a broken object stay where they were;
    * no score popup ("+1200") and no objective signals;
    * no damage markers on the HUD (the HUD has none yet).
  * **Doesn't match yet**:
    * The barrel's burst peaks later than take26's (+0.4-0.7 s against +0.13-0.33 s) and lasts
      longer.
    * Up close, the rack's exp-misrack-smoke (about 45 alpha-blended 17-22 m quads a second,
      yellow at birth) stacks to flat yellow, where take07 stays readable. Both are how
      `ale_fx` reads these effects, not this module.
  * **Test hooks**:
    * `BF_TEST_HIT=<s>[,<s>...]` hits the intact breakable object nearest the controlled character
      (or `BF_TEST_HIT_NEAR=<x>,<z>`) with 50 ballistic at those times, as the takes' gdb call
      did;
    * `BF_SCENERY_LOG=1` lists the breakable objects and logs each hit, break, spawn and area;
    * `BF_COMBAT_LOG=1` prints each area tick's damage.
  * **Verified**:
    * take29's spot (`BF_MAP=sdm_e34 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6 BF_TEST_HIT=2.0`): Tex
      loses 40.0 HP from +0.07 to +0.53 s (take29: 39.4, +0.10 to +0.64 s).
    * take26 (`BF_MAP=e34`): the barrel's green burst shows from +0.13 s.
    * The rack from 7.1 m: 100 HP; the fireball and light, then the pieces and stand.
    * Campaign e34: the rack sets off both barrels on its first tick.
    * A Frag 2.3 / 2.4 m from a barrel and a crate breaks both.
    * Tex's laser (Type 8) breaks a crate in 5 shots (takes 16 / 22: 5-6).
