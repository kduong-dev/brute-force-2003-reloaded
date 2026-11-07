*Part of [levels/](README.md), the former page `levels.md`.*

* **What a liquid plays** (#5: play_liquids.rs, play.rs `WorldSound`; the code read in
  todo/5-liquid-sounds leads.md and todo/146-liquids leads.md §5, the code map's Sound rows
  FUN_00179250-FUN_0017f580 and Levels rows FUN_00221af0-FUN_00222480). A liquid type names four
  effects and four sounds (`LiquidType::effects` / `sounds`), in slots the code reads alike for
  every liquid:

  | Slot | Effect (`env_<x>l_*`) | Sound | Plays |
  |---|---|---|---|
  | `LIQUID_SHOT` | h_08f6cd94 `_projectile` | h_e259380a | an instant ray through the surface (FUN_0022feb0) |
  | `LIQUID_SPLASH` | h_e4791abc `_smsplash` | h_0ed21145 | a part or an object going in with a bounding radius under h_e12e7b9d (0.1 m; chloride 0.25) |
  | `LIQUID_BIG_SPLASH` | h_07ab4143 `_mdsplash` | h_0c46dff8 | the same with a radius of at least that (FUN_00221af0) |
  | `LIQUID_WADE` | h_f458daca `_wadering`, silent | h_ed055bce | the ring round a wading part (FUN_00221ec0); the sound is the wading step (FUN_00119020) |

  `<x>` is x water, s lava, f toxic, t type 3, i type 0 (with bubbles). Every type of one
  liquid-type names the same four sounds, five recordings among the sixteen ids: lava's big splash
  and its wading step are one wave, the ocean's small and big splash another; chloride has none
  (todo/5 leads §1). The effect types carry no sounds, and no liquid has a sound of its own: the
  shore's and the lava's loops are the levels' sound-triggers (triggers.md).
  * **Splashing in.** Each hit part going in (at any speed: a walk, a jump and a fall alike)
    casts FUN_00222480's ray from the top of its bounding sphere 2r down against that liquid
    (`Arena::liquid_below`); the sphere is the part archetype's `bounding-sphere-radius`
    h_1ab09530 round its `bounding-sphere-center` h_e99750a9 (`Character::part_spheres`; the
    squad's eleven parts 0.14-0.45 m: all big). Meeting the surface, the character keeps its
    height (char+0x414, `Wading::surface`) and the part splashes there: the effect upright at user
    parameter 1.0, the sound at the surface point, volume 1.0 (in place of the bank's 0.5), pitch
    U x 0.1 - 0.05 octaves. A part whose sphere doesn't reach the surface as it goes in splashes
    nothing: walking in, the feet and calves go in first without one, and the splashes start
    with the part that does (take03: no splash for the first four parts, the first at the fifth;
    the demo's Brutus the same). A part out and in again splashes again.
  * **Rings.** Each part in a liquid, at most every 0.2 s, casts its ray again; where the surface
    point has moved more than h_049350bb (0.1 m) from its last ring's (its node as it went in, at
    first), and not straight up or down, a ring there, turned to the move, at user parameter
    clamp((moved - 0.1) / dt x h_fd22997d, 0, 1). Silent. So a part crossing the surface leaves up
    to five a second while it moves, and none once it's under or still.
  * **Wading steps.** Each game frame a part is in, the character takes the liquid's fourth sound
    as its next step's (char+0x410). At a foot event (the demo's: a foot planting on the move,
    play.rs `step_player`) in a liquid, the step plays it only while the surface of the last
    splash is more than 0.25 m above that foot, and plays **nothing** otherwise: the ground's step
    never sounds in a liquid, and before any splash has found the surface (char+0x414 still -1)
    every step is silent. Its volume is max(min(depth, 1), the skill's noise) x the gait's factor
    (1.0 running or dashing, 0.5 walking, 0.35 the crouch walk) x 0.75 for the controlled
    character, whose own step plays 2D (FUN_00119020 puts its point in the request's second slot,
    where neither the cull nor the voice reads a position: play.md, "Footsteps (#324)", which
    gives the ground's step the same rule); another's plays at its foot; the pitch U x 0.7 - 0.35
    octaves. The first step after coming out still plays the wading step the last frame in left
    (the code's order); the ground's come back from the next.
  * **Objects** (`object_enters`). A grenade (play_grenade.rs) or a missile (play_missile.rs) going
    into a liquid, from out of every one, at h_1f6de6f9 (0.4 m/s) or faster, splashes where its
    body crosses the surface, by its mesh archetype's radius (`Game::archetype_sphere`): the five
    hand grenades 0.081 m small, the Sentry 0.33 m and the Roller 0.19 m big, the iKhan-GPL's
    round 0.046 m small. They go on down to the bed (the liquids aren't solid). Not done: what
    happens to them in it (a grenade's fuse cut to at most 0.3 s, 0x149420; a missile's timers x5,
    FUN_00226cc0), the object damage h_06f6a40d, the blast's liquid override effect (#146).
  * **Shots.** Only an instant ray (bullet-type 4: the MK-ASLT, Bower 20, L-Shots, Minigun,
    Foley, Kman, Rails) splashes, once where it goes through the surface: the shot effect upright
    and the shot sound at the library's volume (FUN_0022feb0 sets no volume or pitch). A bolt or a
    flying shot passes through with nothing (the blast pool has no liquid code); the ocean's trail
    of bubbles along the ray under its surface (+0x168) isn't drawn.
  * **Nothing** plays for going under, staying under, getting out or burning: no reader of the
    liquid state in the sound system (todo/5 leads §2.5).
  * **How they're heard** (play.rs `play_sounds`, `WorldSound`: FUN_00179250's request). The
    listener is FUN_00178c10's, as the sound-triggers have it: the followed character's feet, or a
    cutscene's camera while one is shown. A positional sound is heard by its library entry's
    distances (`distance_gain`: every liquid sound full to 5 m, nothing at 15 m) and dropped at
    15 m or beyond (0x17932a); a 2D one at its volume. The pitch is the playback speed, 2^octaves
    (DirectSound's pitch is a resampling too). Not done: the sound manager's queue and its budget
    of 16 positional voices a frame (FUN_001793e0: here every one plays), and placing a sound in
    3D (every demo sound plays 2D at its distance's volume).
  * Checked with `BF_SOUND_LOG=1 BF_LIQUID_LOG=1` (BF_MUTE: nothing was played aloud; how they
    sound is the user's check by ear; the Frags at sdm_e13's shore were thrown before #161 made
    the throw the game's, which now goes faster and higher, so their splash distances don't hold
    as written; not run again):
    * sdm_e13's lava, Brutus walking in to 2.6 m deep (`BF_MAP=sdm_e13 BF_CHARACTER=0
      BF_TEST_GOTO=-19.97,-122.80,-22.6,-102.0 BF_TEST_WALK=1 BF_TEST_HEALTH=5000`): his first
      four parts in without a splash and his steps silent (no surface found yet), then big
      splashes (ef2c3bf6, the lava's C recording) at full volume, 0.7-1.5 m away, as more parts go
      in, and wading steps (188ca605) 2D at 0.38 every ~0.55 s, 1.0-2.6 m under the surface, with
      their +-0.35 octave pitch; frames 80-118 show the rings at his feet, then the splashes'
      orange puffs round his waist (take03 at 10.2-10.9 s: orange puffs round Brutus's waist as
      the splashes start; the demo's brighter and yellower, the ALE effect's colour).
    * The tutorial's pit (#28's take08 recipe above): Tex falls in, six big splashes over 0.07 s
      at 1.3-1.9 m (a fall splashes as a walk does), the damage unchanged (five hits of 18, dead
      0.80 s after the first).
    * sdm_e10's pond (water: `BF_MAP=sdm_e10 BF_CHARACTER=3 BF_TEST_GOTO=74,5,74,-30
      BF_TEST_WALK=1`): wading steps (fa93070a) from 0.60 to 0.31 m deep, silent at 0.25, 0.17,
      0.10 and 0 m, the first step out still the wading step, then the ground's (fe306c9e,
      e5393d24) at 0.35 (0.38 since #324: the walk's 0.5 x 0.75; the wading steps' lines are the
      same as before it, volume, pitch and time). With `BF_TEST_GOTO=74,5,74,5 BF_TEST_TARGET=1` Brutus stands in it 8 m
      off: his five big splashes (0d339af9) heard at 0.54-0.66 by the distance.
    * Brutus at sdm_e13's shore (`BF_TEST_GOTO=-20.6,-117.5,-20.62,-117.0`): `BF_TEST_THROW=1.0,0.05
      BF_CAMERA_PITCH=-0.2` lobs a Frag in at 15.1 m/s, a small splash (1eb5691c, the B recording) 5.3 m off at 1.00;
      thrown long (`BF_TEST_GOTO=-19.97,-122.80,-20.0,-121.8 BF_TEST_THROW=1.0,0.6 BF_CAMERA_PITCH=0.1`) it goes in
      15.3 m off, past the 15 m: not heard. `BF_TEST_WEAPON=iKhan-GPL BF_TEST_FIRE=1
      BF_CAMERA_PITCH=-0.25`: each round a small splash 8.6 m off at about 0.5. `BF_TEST_FIRE=2`
      (the Bower 20, bullet-type 4) at `BF_CAMERA_PITCH=-0.3`: a shot effect and sound
      (0a94d4e9) where each ray goes in, 12.9-24 m off: those under 15 m at 0.00-0.03, the rest
      not heard.
  * Not filmed: every water item is **code only, needs a take**: which of the B and D recordings
    plays for each event on water, the silence in the shallows (0.25 m), the player's own step 2D
    and a squadmate's at its foot (the 2D read in the code since #324, play.md, "Footsteps"), a
    grenade into water, a ray through it (todo/5 shot list).
    The lava's are checked against take03's offline audio analysis only (todo/5 leads §4: a burst
    within 0.06 s of each foot event waist-deep; no burst for the first four parts in), which
    can't tell the lava's big splash from its wading step (one recording). The idle animation's
    foot events (take03's step every ~2.1 s standing waist-deep) and the ragdoll's splashes
    (take01's corpse) aren't reproduced: the demo's steps come from feet planting on the move,
    and its ragdoll's part bodies (#259, ragdoll.md) don't tell the liquids when they enter.
