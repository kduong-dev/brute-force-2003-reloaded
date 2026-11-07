*Part of [play/](README.md), the former page `play.md`.*

### Bolts and their hits: the sweep, hit effects, lights and sounds (#57)

`src/bin/play_bolts.rs` (the flight and its sweeps, #136), `src/bin/play.rs` (`world_hit`,
`hit_effect_at`, `start_flight`, `land_player_shot`, `land_ai_shot`), `src/bin/play_fx.rs`
(`spawn_fx`), `src/ale_fx.rs` (`load_flight`) and `src/arena.rs` (`ray_shot`, `ray_material`),
from default.xbe's bolt pool (FUN_0022bac0, FUN_00225220, FUN_00225440) and the damage
handler's tail (0x22d0b0 -> FUN_0022cd80, FUN_0022b5b0); the analyst's answers are
todo/weapons-plan/answers-batch1.md (group A), the takes and their measures todo/57-jax-ic/.
The bolt guns of weapons batch 1 (the Jax-iC, -iP, -iR, the Saryls, the Bioreactives) and the
squad's LZRs and cutter go this way.

* **The launch** (#136; FUN_0022bac0, play_bolts.rs `launch_bolts`). A flying shot of bullet-type
  0 or 3 (under the demo's `INSTANT_SHOT` speed) is a bolt (`Shot::bolt`), FUN_0022bac0's pooled
  object: a moving object from the muzzle along the shot at the bullet's speed, living the
  bullet's range in the holder's hands over its speed (FUN_002324c0), its damage roll made at the
  launch (FUN_0022c940: Damage min..max from the game's random state, `GameRandom`, x the
  shooter's skill's factor then) and kept as the record of every hit it makes.
* **The sweep** (#136; FUN_00225220 -> FUN_00225440, play_bolts.rs `fly_bolts`, `Bolt::advance`).
  Each 30 Hz game frame its life runs down (out: it ends where it is) and its byte +3 counts down
  from 3; not yet 0 it moves on unchecked, at 0 a ray is swept from its last checked point over the
  way since and this frame's step: 8 m at 80 m/s, 10 m at 100 m/s. So a hit lands k x 0.1 s after
  the shot, on the first sweep that meets it, as the takes count it (from the frame after the
  shot's; the code would count the launch frame as the first, a frame sooner: answers-batch1.md
  §1 leaves it to the takes). The sweep meets what's there then (`sweep`): the world as the
  weapons' ray filter takes it (`Arena::ray_shot`: the terrain and the placed objects, a breakable
  among them, not the level's invisible blockers, which the filter's PID mask 0xfffffff8 leaves
  out), a character where it is now (standing, its cylinder; knocked down, its ragdoll's bones; a
  corpse, its ragdoll), a Sentry; never its shooter. On a hit the bolt is set back 0.1 % of the
  way toward its last checked point, the hit is dealt that frame (FUN_00224770 -> 0x22d0b0: the
  damage through `land_player_shot` / `land_ai_shot`, the hole, the hit effect and its sound, a
  breakable's roll through `ObjectHits`, a Sentry set off, a body on the ground shoved) and the
  bolt ends. Before #136 the demo picked a bolt's hit at its shot (the body or the world on its
  line then) and landed it on the sweep that would reach it: a target that stepped out of the way
  was still hit, one that stepped in wasn't. The loose pickups a sweep passes are knocked as by a
  shot's line (`Sweeps`, play_pickups.rs).
* **The flight.** A bolt moves from its first frame (before, it waited two at the muzzle): at
  the muzzle in the shot's frame, then on by its speed each game frame (drawn on by the frame's
  time not yet run in game frames), on into what it struck until the sweep that finds it (the
  Jax-iR's ball gone into the wall before its burst, todo/59 take01); one that meets nothing ends
  with its life, without the sweeps after it (the Jax-iC's last reaches 56 m of its 60). Its
  flight effect rides it (`start_flight`, compiled by `load_flight`): the effect's emitters ride
  with the bolt and their particles go with it when it ends (the ball, its halo), except an emitter
  under the library's world node (h_ee223b51, node flag 1) whose appearance hangs at the root,
  which leaves its particles in the world (the trails). The Jax-iR's ball hangs under the node
  with its appearance and rides (#59: read as a trail it lay a violet band along the whole way,
  where take01 shows nothing between the gun and the wall).
  Where an emitter has the demo's ATTACHED (h_e2999ffd: the LZRs' bolts) it still decides,
  though code-map's 0x64e30 reads that parameter as the cube emitter's offset height. The rule
  is inferred from the effects' trees and the takes (the balls gone the frame their bolt ends,
  trails fading where they were laid), not from code. Before, a root emitter without the flag
  left its particles in the world, and a bolt's effect began emitting a frame after it.
* **A hit on the world** (`world_hit`): the struck triangle's world material
  (`Arena::ray_material`: the collision's terrain comes as twin triangles, and the one whose
  front faces the shot carries the material and flags, the back twin 0) and the shot's ammo-type
  pick from world-materials, as FUN_0022cd80 and FUN_0022b5b0 do:
  * the hit effect: the material row's for the ammo-type (h_f6f08598; only the flesh materials
    have one), else the weapon's own (h_08d28037) at user parameter 1.0;
  * the material's debris effect (`debris-effect`: wmat_dirtpuff's tan puff on e34's ground,
    wmat_concrete's grey dust ...) at user parameter 0.036 x the hit record's damage, at most 0.9,
    1.0 for a killing hit (`hit_sp`; the record's damage is the roll x the falloff here);
  * one of the row's four hit sounds at random (FUN_0022b5b0: the random state's low 2 bits),
    from the hit by the bank's distances (`distance_gain`: the sonic rows full within 9 m,
    silent beyond 20 m). Before, a world hit had no sound and no debris.
  * The roll and the sound's pick are draws of the demo's copy of the game's one random state
    (`GameRandom`: the LCG at 0x46cfd8, FUN_0022c940's roll from its bits 9-31), drawn per hit;
    the hit effects' and each shot's flight effects' seeds are draws of it too (the demo's). The
    game picks the sound by the state's low 2 bits, which such an LCG steps through with a
    period of 4; there the rest of the game's draws mix them, here each shot draws the same count
    and the low bits would pick one sound for good, so the demo picks by the top 2 of a draw
    stirred by an integer hash (`GameRandom::stirred`; the raw top bits clustered too: a lone
    shooter's first five hits drew one sound). A seed from the muzzle's place stood in at first
    and saturated to 0 wherever its sum was negative: every hit at e34's gate had one sound, one
    roll and one burst.
  * Each effect in its alignment's frame (`hit_frame`; the effect type's `alignment-type` per
    alchemy effect, FUN_0021a850, the ALIGN_TO_ table at 0x3bd298): 0 the shot's way (the frame
    FUN_0001d0c0 builds: its -z along the way), 1 that way reflected off the surface, 2 the
    surface's normal, outward, 3 the world's axes. The hits' emitters fire along their -z (they
    are turned -90 degrees about x), so an alignment-2 burst (the sonic's, the particles')
    comes out of the surface and an alignment-0 one goes on along the shot. Before, every hit
    effect had its +y turned back along the shot, which threw those emitters across it.
  * Each effect type's light-effect is a point light at the hit (the hit lights:
    light_hit_sonic's cyan wash, #57), as the muzzle lights and the blasts' are played. The
    debris types have none.
  * The effects are compiled with the game's emission count, shapes and facing 3
    (`load_recorded_facing`), with the type's bounding sphere for the distance LOD.
* **The bullet's explosion** (#53; play.rs `shot_explosion`): before the struck material's hit
  effect, the damage handler plays the bullet's own effect type at every direct hit, on the world
  or a body (0x22d0b0 -> FUN_0022cfc0 -> FUN_0022cbe0: h_ec23d593, the weapon type's +0x250, at
  user parameter 1.0, with its light) and its impact sound (h_e5618348, +0x254). Of the hand guns'
  shots only the two cutters' come here with one: cutter_hit and plut_cutter_hit with
  light_hit_cutter, the white star where a blade strikes (#53 take01 f17-f18 of 5.20 s on, on the
  tower; take03, on the enemy's chest); every other gun names its hit in h_08d28037 instead. The
  iKhan-GPL names one too (exp_mirvgrenade), which its round, a plain shot after batch 2, played
  here (#287); since #239 the round is a lobbed missile and plays it where it goes off
  (**Missiles** below). A bolt's hit on a body, a corpse or a Sentry plays it from `deal`
  (play_bolts.rs), a world hit from `world_hit`. Not
  done: the per-material overrides at +0x25c, taken when the hit record's +0x58 bit 1 is set (not
  traced for a bolt). Before, the cutters' hits had only the material's debris.
* **A hit on a body** (play_fx.rs `spawn_fx`): the flesh row's effect for the ammo-type, when the
  demo doesn't make it by hand (the squad's ballistic, cutter and laser ones), is played from
  the data with its light: the sonic h_177a62d8 (wmat_fleshit_sonic), the particle h_ed6363bd,
  the bio h_f8d4e76d, the rail's and the energy's. Its user parameter is the hit record's
  (`damage::Hit::record`: the roll x the skill's factor x the falloff x the part struck x the own
  side's 0.7, which 0x22d0b0 hands to FUN_0022cd80; the victim's step and type factor come
  after), 1.0 for a killing hit; aligned as above, the body's normal taken out from the hit
  cylinder's axis (the demo's stand-in for the struck part's collision).

Test hooks: `BF_SHOT_LOG=1` also prints the map's world materials as it loads ("world
materials (id: triangles) 0: 15482 terrain + 4596, ..."), each world hit ("t=2.37 hit on world
material 5 at [...] 8.4 m out (ammo-type 3): effect h_fed64261 sp 1.00 light h_eb110330 (the
weapon's), debris h_fe578f8f sp 0.79, sound h_0655c729 vol 0.57 (slot 1)": which of the row's
four sounds the random state picked; a bolt's says how far along its sweep ("10.3 m along its
sweep") where a shot's says how far out; a cutter's adds "explosion h_0557b326 light h_16d39d8b" before its effect) and each
hole's life; and each bolt's launch ("t=2.20 bolt 1 launched by tex from [...]
along [...] at 80 m/s: roll 21.97 x skill 1.00, life 0.750 s (range 60.0 m)"), each sweep that
meets something ("t=2.50 bolt 1 sweep at game frame 9 (0.30 s): hits Shrike Soldier (NPC 1) 7.4
m on from its last checked point 23.4 m out", with "(past a level blocker 0.40 m on)" where the
sweep went by one) and its end at its life's ("bolt 1 (Jax-iC) ends at its life's end after 23
game frames, 58.7 m from its muzzle"). `BF_COMBAT_LOG=1` each flesh hit effect played from the
data ("flesh hit effect h_177a62d8 at [...] sp 0.41 light h_eb110330").

**Moving targets** (#136; the test map, `-- --test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0
BF_TEST_WEAPON=Jax-iC BF_TEST_GOTO=4.5,20,4.5,20 BF_CAMERA_PITCH=-0.115 BF_TEST_NPC_HP=1.0,1,300
BF_SHOT_LOG=1 BF_COMBAT_LOG=1 BF_CAPTURE_FPS=30`, a dm Shrike Soldier 25 m out):
* Standing (`BF_TEST_NPC="0.3,shrike soldier@dm,enemy,dummy,4.5,-5" BF_TEST_TRIGGER=2.0-2.08`):
  the bolt fired at 2.20 meets it on its sweep at game frame 9 (0.30 s), 23.4 m out.
* Stepping aside (the same with `BF_TEST_NPC_MOTION="2.23,1,dash,10.5,-5"`, the dash starting a
  frame after the shot): the same bolt, along the same line, meets nothing and ends with its life
  58.7 m out; frames 66-77 show the ball pass where the Shrike stood as it dashes off to the
  right. The demo before #136 picked the hit at the shot and landed it on the Shrike.
* Crossing (`BF_TEST_NPC="0.3,shrike soldier@dm,friend,dummy,-1.5,-5"` running across the line
  from 2.2 s, `BF_TEST_NPC_MOTION="2.2,1,dash,10.5,-5" BF_TEST_TRIGGER=2.65-2.73`): fired at 2.80
  with the Shrike 3.3 m left of the line, the sweep at frame 9 (24 m) meets nothing and the one
  at frame 12 meets the Shrike 24.2 m out, where it is by then (its left upper arm, x0.7 on the
  own side: a friend); before #136 the bolt missed it.
* The pillar, e34's gate and the frozen Shrike Heavy recipes below land as before: 8.5-8.8 m out
  at frame 6 (0.2 s), 5.3 m out at frame 3 (0.1 s). A supply crate put down 6.6 m out
  (`BF_TEST_PLACE="0.5,supply crate,4.5,12,0"`) breaks on the sweep that meets it (24.38 x 5,
  SONIC), the bolt ending there; a dead Shrike's body on the ground 8 m out stops the bolts after
  it and is shoved ("hits Shrike Soldier (NPC 1)'s body"); a Sentry 9.6 m out (the grenade
  page's `BF_TEST_SHOOT_MINE=2.2`, an LZR-23 bolt) goes off on the first sweep, 0.1 s after the
  shot. `cargo test --bin bf_play bolts` checks the sweeps' frames and lengths and the life's
  end (80 m/s and 60 m: sweeps on frames 3-21, the end on 23; 100 m/s and 80 m: the end on 24).
* Not reproduced: the game sweeps against its bone bodies; the demo's characters are its
  cylinder (standing) and ragdoll bones (lying), as the shots' have been. A bolt passes a liquid's
  surface without a splash and goes on to the bed: the blast pool has no liquid code (#5,
  todo/146-liquids leads §5.1; only an instant ray splashes, levels.md "What a liquid plays").
  Whether the game's sweep stops on the liquid's body isn't checked.

Verified (`BF_CAPTURE_FPS=30`, the logs, against todo/57-jax-ic's takes and spec):
* **The Jax-iC at the test map's pillar** (`-- --test BF_TEST_WEAPON=Jax-iC BF_TEST_GOTO=0,-8,0,-8
  BF_CAMERA_PITCH=-0.12 BF_TEST_FREECAM=0.5,1.5,1.2,-13.5,18.3,-10
  BF_TEST_TRIGGER=2.0-2.08,3.6-3.68,5.0-9.0 BF_TEST_CLIP=3 BF_SHOT_LOG=1 BF_AMMO_LOG=1`): held fire
  every 15-16 game frames (the take 15.5); the reload 0.50 s after the last round (reserve 200 ->
  164) and the next round 1.30 s later (the take 0.50 and 1.30 s); each hit 0.2 s after its shot
  8.5 m out, the weapon's sonic burst with its light washing the pillar, the default material's
  debris, the hole.
* **The gate's foot on e34** (take01: `-- --test BF_MAP=e34 BF_TEST_WEAPON=Jax-iC
  BF_TEST_GOTO=-55.2,31.3,-55.2,31.8 BF_CAMERA_PITCH=-0.26
  BF_TEST_TRIGGER=2.0-2.08,4.2-4.28,6.4-6.48 BF_SHOT_LOG=1`): the ground is material 5 (DIRT):
  the hits 8.4 m out land 0.2 s after the shot and the ones 7.2-7.4 m out 0.1 s after (the take
  T0 + 0.2 s at ~10 m), with wmat_dirtpuff's tan puff after the burst and one of the row's
  sounds (h_0655c729; the take's h_0f52f7bf and h_1d5c9693 are two of the same four).
* **A frozen e34 Shrike Heavy 7 m out** (take03: the same with `BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC="0.3,shrike heavy,enemy,frozen,-55.39,38.3" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_TRIGGER=2.0-2.08,4.2-4.28,6.0-12.0 BF_COMBAT_LOG=1`): every hit 0.1 s after its shot;
  29.6-35.1 on its right calf (roll x0.5 x STANDARD 1.4 x SONIC 2: the take 30.5-34.9 on its
  hovering legs), dead on the 10th hit as in the take; each hit plays wmat_fleshit_sonic with
  light_hit_sonic at user parameter 0.38-0.45 (0.036 x the record's 10.6-12.5), 1.00 on the
  kill. At `BF_CAMERA_PITCH=-0.12` the hits go to a thigh: 61.0-68.3 (the takes' standing
  Shrike 59.5-68.8).
* Against the takes' pictures the burst's core is about the take's (14-16 units at ~10 m) but
  paler, and the take's violet wash of the light over the gate's foot (80-140 units wide) is
  missing on e34's lit surfaces; on the test map's pillar the light shows. The light's reach on
  the level's own surfaces is #143's.
* **The Jax-iP** (#58, todo/58-jax-ip): at the pillar (`BF_NO_SQUAD=1` and the recipe above with
  `BF_TEST_WEAPON=Jax-iP`): held fire every 20-21 game frames (the take 19.8), the reload 0.67 s
  after the last round (the take 0.71), the hits 9.4 m out 0.2 s after their shots, sonic_proj_hit's
  white core with light_hit_sonic. On a frozen e40 Shrike Heavy (`-- --test BF_NO_SQUAD=1
  BF_TEST_INSTANT_KILL=0 BF_TEST_WEAPON=Jax-iP BF_TEST_GOTO=4.5,20,4.5,20 BF_CAMERA_PITCH=-0.08
  BF_TEST_NPC="0.3,shrike heavy@e40,enemy,frozen,4.5,-5" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_TRIGGER=2.0-2.08,3.6-3.68,5.0-9.0 BF_COMBAT_LOG=1`): 25 m out every hit lands 0.4 s
  after its shot (take07 0.4, 0.4, 0.4), 37.0-39.0 on the body and 208.0 on the head (roll x
  SONIC 2 x 1.4: take07 35.1-38.5); 9 m out (`...,4.5,11`) 0.2 s (take03's 0.2 in 5 of 8),
  34.4-39.0 on the body, 208.0 on the head, 30.3 on an upper arm; 45 m out (`...,4.5,-25`,
  `BF_CAMERA_PITCH=-0.115`) nothing is hit, the bolt ending in the air at its 40 m (take08: no
  hit), the crosshair blue there and red at 25 m (the 28 m lock reach). The test map's own
  "shrike heavy" is the dm Heavy, whose SONIC is x1 and whose stamina shield takes the first
  40: use `@e40` (or `@e34`) for the campaign's x2.
* **The Jax-iR** (#59, todo/59-jax-ir): at the pillar, held fire every 25-26 game frames (the
  take 24.8), the reload 0.83 s after the last round (the take 0.81; reserve 200 -> 176), the
  hits 8.9 m out 0.2 s after their shots. At e34's gate as take01 (Brutus carries it there:
  `BF_CHARACTER=0` with the gate recipe above; e34's squad keeps its own loadout, so
  `BF_TEST_WEAPON` isn't read): the ball goes into the ground at the gate's foot a frame or two
  before the burst at +0.2 s and nothing is left along its way, as the take shows (`ride`, the
  trail rule above; before it a violet band lay from Brutus to the wall for 0.6 s). On the
  frozen e34 Shrike Heavy 7 m out (the take03 recipe above, `BF_CHARACTER=0
  BF_CAMERA_PITCH=-0.12`): every hit 0.1 s after its shot, 88.9-95.1 on the body (roll x 1.4 x
  SONIC 2: take03 86.4-88.2, the range 86.1-95.5), wmat_fleshit_sonic at 0.90 (the cap), dead on
  the 4th. On the test map's clear line (the Jax-iP's recipe with `BF_TEST_WEAPON=Jax-iR`): 25 m
  out 0.3 s (the rifle's muzzle 0.4 m further out puts the body at 23.8 m, inside tick 9's 24 m:
  take08 0.4 s, the takes' 25 m at 80 m/s 0.3 s in 6 of 24), 86.8-90.6 on the body; 45 m out
  0.6 s, on the calves at `BF_CAMERA_PITCH=-0.115`.
  * Not reproduced: take07's glow at 65 m lasting to T0 + 1.50-1.55 s, 0.4 s past the bolt's
    1.125 s life; the demo's ball goes when the bolt does.
* **The Saryl-45** (#70, todo/70-saryl-45; the particle family's look): at the pillar, held fire
  every 12-13 game frames (the take 12.1), the reload 0.40 s after the last round (the take
  0.40; reserve 200 -> 170), the hits 8.9 m out 0.1 s after their shots (100 m/s: 10 m a
  step). From behind Tex (`BF_TEST_FREECAM=0.5,0.6,2.2,-4.5,0,-8`) the comet's white head rises
  from the gun with its pale-blue tail down to the muzzle (plasma_bullet riding, its trail left
  in the world) and ends in plasma_hit's white star with blue shards out of the face and
  light_hit_particle's wash, the muzzle's blue glow at Tex's hand meanwhile, as take01 shows.
  Seen from 4.5 m the star's core is ~0.5 m and the shards ~1.4 m across (take01's 16-unit
  core at ~13 m is ~0.65 m), so the data's 3.74 m end size (the spec's open point) reads right:
  the quads fade out as they grow. On a frozen e40 Shrike Heavy (the Jax-iP's recipe, the
  Saryl-45): 7 m out 0.1 s, 25 m out 0.3 s (take08's 10 of 10), 45 m out 0.5 s; 18.1-20.8 on the
  body, 15.1-17.4 on an upper arm, 96.8 and 103.8 on the head, 9.0-10.7 on the calves (PARTICLE
  x1: take08 18.1-22.0, take09 106.8 on the head, take03 8.9-11.0 on the hovering Shrike's
  calves); each hit plays wmat_fleshit_particle (h_ed6363bd) with light_hit_particle.
* **The Saryl-23** (#69, todo/69-saryl-23): Hawk carries it on e34 (`BF_CHARACTER=2` with the
  gate recipe above). At the gate's foot as take01: each hit 0.1 s after its shot 7.7-8.8 m
  out, plasma_hit_sml's small white star with blue shards out of the ground, then
  wmat_dirtpuff's tan dust and one of the row's sounds (h_0655c729), the comet's head with its
  tail on the way; the star shows on the hit's next frame (+0.13 s), take01's at +0.18 s. On the
  frozen e34 Shrike Heavy 7 m out (the take03 recipe, `BF_CHARACTER=2 BF_CAMERA_PITCH=-0.12`):
  held fire every 13 game frames (the rate's 12.5 rounded up; the take 12.7), every hit 0.1 s
  after its shot, 12.9-18.9 on the body (PARTICLE x1: the range 11.9-18.9; take03 12.8 and 16.9
  on the body, the rest on the hovering Shrike's calves), wmat_fleshit_particle with
  light_hit_particle on each.
* **The Saryl-99** (#71, todo/71-saryl-99; the Saryl-45's effects): Brutus's second gun on e34
  (`BF_CHARACTER=0 BF_START_WEAPON=1`). On the frozen e34 Shrike Heavy 7 m out (the take03
  recipe, `BF_CAMERA_PITCH=-0.12`): held fire every 10-11 game frames (the take 10.1), every hit
  0.1 s after its shot (take03's 17 of 17), 11.0-16.4 on the body (PARTICLE x1: take03
  11.2-15.7), wmat_fleshit_particle with light_hit_particle. On the test map's clear line (the
  Jax-iP's recipe with `BF_TEST_WEAPON=Saryl-99`) 25 m out every hit 0.3 s after its shot
  (take08's 2 of 2), 11.2-15.0 on the body, 64.3 on the head.
* **The A10 Bioreactive** (#47, todo/47-a10-bioreactive; the bio family's look): at the pillar
  (`BF_TEST_WEAPON="A10 Bioreactive"`), the hits 8.8 m out 0.2 s after their shots (take01's
  burst at T0 + 0.20 s), the weapon's own bio-flash_hit and bio-shrap_hit out of the face with
  light_hit_bio, and its green hole h_1ce6db03 living 60 s (`BF_SHOT_LOG`'s "hole ... life 60
  s, fading over its last 5 s"); the reload 0.40 s after the last round. Seen from a free camera
  standing in for the take's (`BF_TEST_FREECAM=0.5,2.5,2.0,6.0,5.5,-3.9`, 23.7 m back: chosen when
  the demo drew 45 degrees down against a guessed ~74; the game's view is 55.4 down since #269, so
  the take's own camera, about 13 m back, is the one to stand in for now) the flash's white-green
  core was ~17 units
  (take01 12-14) and its green haze thins over ~0.8 s (take01: to 10% at +1.3 s); the green
  bolt and the muzzle's green glow at Tex's gun as the take's. On a frozen e40 Shrike Heavy 7 m
  out (the Jax-iP's recipe with the A10): every hit 0.1 s after its shot, 11.0-12.8 on the body
  (BIO x1: take03 11.0-13.7), 9.0-9.6 on an upper arm, 63.6 on the head, the hit points moving
  only at the hits (no damage over time, as take06's log), wmat_fleshit_bio (h_f8d4e76d) with
  light_hit_bio on each. The bioreactive kill ("massive cellular deterioration") is #147's: the
  demo's BIO kills are plain.
* **The A50 Bioreactive** (#48, todo/48-a50-bioreactive; thin footage, no enemy or reload take):
  at the pillar (`BF_TEST_WEAPON="A50 Bioreactive"`) held fire every 10-11 game frames (take01
  10.4), the hits 8.5 m out 0.2 s after their shots with its hit type h_fe10809d and
  light_hit_bio, the green hole; its two-arc crosshair blue on the pillar, red on a Shrike 7 m
  out. On the frozen e40 Shrike Heavy 7 m out (the Jax-iP's recipe with the A50): every hit 0.1
  s after its shot, 16.9-18.5 on the body (the rules' 16.1-18.9, no take to check), 14.1-14.8 on
  an upper arm, 90.2 on the head, wmat_fleshit_bio with light_hit_bio. Not checked: the reload
  (none filmed: take02's reserve was written where the game reloads from nothing) and any
  damage against a take; the bioreactive kill is #147's.
* Regressions: the squad's eight guns at the test map's pillar (MK-ASLT, Minigun, Bower 20,
  Foley, L-Shot-50, LZR-23, LZR-50, Feral Cutter: two taps each, against main's frames, the
  free camera 4.5 m from the pillar's face): the instant rays' sminigun_hit now flashes out of
  the surface where it lay as a thin streak across the shot, with the default material's
  debrishit dust; the LZRs' and the cutter's hits land on the sweep and their bursts come out of
  the surface (the LZR-23's ~1.5 m burst, as its take01 shows); the LZRs' bolts show their red
  streak in flight in this side view, where main's showed almost nothing between the muzzle and
  the hit (no take shows a bolt from the side). The test map's NPC fights (testtools.md), the
  tutorial's first patrol (brain.md) and the Therm Sweeper at the pillar play as before.
