*Part of [grenades/](README.md), the former page `grenades.md`.*

* **The blast**, on the ground below where it went off: the effect type's ALE effects and its
  light effect run once from that frame (`src/ale_fx.rs`; an effect without a parent now starts
  on the frame it's spawned; a "light_" effect is one lamp for the whole effect, its newest
  particle's: reach = its Size curve, colour = its Color curve, lit as the game's lamp is
  (#272: levels.md "Effect lights"; before, a light a particle at colour x alpha x size x 30000
  lumens, 1 / d^2, fitted to the old DNA effect's light), its sounds (h_f724cb8c read as a delay: the
  Light's 0.2 s) and the impact sound. The decal is laid 0.7 s later, once the fireball has
  gone (laid at once, it showed as a hard dark disc through the added fireball), its
  width / height read as half sizes as the blood decals' are (the Frag's 3 x 3 is a 6 m quad,
  half dark over ~3.7 m; the recording's scorch is half dark over ~4.1 m), lasting 60 s and
  fading over the last 2 s (h_199870ec / h_fe4e1d82 read as life and fade, an inference), laid
  along the ground's slope under its middle. It darkens as the console's blend does: the Xbox
  blends in stored (sRGB) values, Bevy in linear light, so its alpha a is drawn as
  1 - (1 - a)^2.2 (the scorch's middle at ~0.25 of the ground, the recording's 0.2-0.3; at the
  plain alpha it was 0.47). Blasts that hurt throw loose pickups (not the Light's).
  * ALE details the blast showed up (`src/ale_fx.rs`). These three rules are checked against
    the Frag recording only, so they apply only to the
    grenades' blast and trail effects (`AleAssets::load_recorded`, `Compiled::recorded`);
    every other effect (guns, hits, muzzle flashes, level fires, icons) plays as before them,
    until checked against its own captures:
    * An animated texture's frame comes from the appearance's 18657e4a over the particle's
      life: a ramp plays the flipbook over the life (exp-lrg-add's Exp5 0.61 -> 1.0: from the
      orange frames to the smoke), a constant above 0 holds that frame (exp-lrg-fire 0.70, the
      muzzle flashes 0.60), 0 plays it at its own rate (the shield icon). An inference;
      before, every flipbook played at 30 fps and wrapped, so exp-lrg-add went back to its
      bright first frames at 0.53 s: a second white flash the recording doesn't have.
    * An emitter with no rate and no initial count starts with its e7221f95 (read at 0) as a
      burst: exp-lrg-flash's 4 flares, the recording's wide white haze on the first frame
      (an inference; one particle otherwise, a gun's tracer).
    * Each effect's materials are drawn once out of sight while the map loads
      (`ale_fx::warm_up`), so their render pipelines are built before the first blast. In one
      of five captures run beside two other instances, the flash and the add layer were
      missing for the blast's first half second: the likely cause, a pipeline still being
      built. Not reproduced since (four captures, three at once, alike).
  * Added for the Energy's stun_grenade_master (grenade effects only):
    * An effect appearance (class h_0ec77ea0) makes each particle of its emitter carry the
      effect named by its h_0ec7a290 ("stun_grenade": a bolt), turned to face the way the
      particle moves along the ground (an inference). The particle lives no longer than its
      effect runs once (0.8 s, not the master's 7.49 s).
    * Those two of its pairs whose emitter isn't in the node library (stun_grenade_init's
      h_ed10c55f, stun_grenade_init#1's h_f48dc74d) get a stand-in: one particle at the middle,
      living the appearance's own lifespan (a guess, fitting the recording's flash and wash).
      Only those two, by name: exp-lrg-dirt (the Frag's, the Roller's) and exp-mine-dirt (the
      Sentry's) miss their emitters too and still draw nothing.
    * A beam whose texture is an animated one ("ARCb": a 4 x 4 sheet of arcs, 30 fps) steps
      through its frames over the ribbon's life.
    * An effect entity's scale scales its emitters' offsets (a bolt's reach is fitted to the
      body it strikes so).
  * Beam ribbons are never frustum-culled (any effect): their mesh is rebuilt every frame, and
    the bounds taken from its first few points culled the Energy's bolts as they spread.
  * **How the blast's quads are turned** (#286; [maps.md](../maps/ale-blends-and-particle-facing.md), **Particle facing**): by
    each appearance's GeneralApp_FacingType, as the game's particle vertex shader builds them.
    The Frag's fireball layer exp-lrg-add.app (sphere emitter exp-lrg-add.emt, 0.48 m/s; Exp5 at
    5,2; quads 3 -> 0 -> 9.2 -> 12.7 m) and its sparks exp-lrg-shrap.rect.app (spark.tga, cone at
    11-15 m/s) are facing 3 with AngleFadeOut: each quad runs along its particle's motion, turned
    about it to face the eye, and fades seen along it. The flash, the fire puffs and the dirt
    (exp-lrg-flash, exp-lrg-fire, exp-lrg-dirt) are billboards (facing 1), turned only by their
    Rotate. Before #286 the demo lay AngleFadeOut ("perp") quads flat across their emitter's y,
    and turned the Frag's exp-lrg-add out along each particle's way (a rule fitted to the Frag
    recording, `PERP_RADIAL_SPEED`): its sparks lay flat, horizontal streaks across the blast,
    and the fireball was a ball of discs, the ones on the line of sight facing the eye. Now the
    fireball is round with a bright rim from the first frame, its middle softer, with no streaks
    (scratchpad `t286dev/side_e40.png`, below).
    * **Checked against #119 take01's first blast** (e40, ~26 m out; the recipe in **Throwing**
      above, `-- --test BF_MAP=e40 BF_CHARACTER=3 BF_TEST_GOTO=-1.97,169.30,-25.63,158.34
      BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.157 BF_TEST_THROW=2.0,2.0`, the blast at t 7.07,
      frames 105-115 against the take's 30 fps frames from 8.0 s): the flash, then a round
      yellow-white fireball growing to ~170 x 130 px, turning orange and then to a dark ball. **Not
      matched:** the take's fireball is wider and flatter (~220 x 150 px and more, from the
      stone on the left to the one on the right, flat on the sand) and its smoke a wide brown
      haze, where ours is round, ~70% of its width, and ends as a dark ball. Not the facing:
      every fireball particle starts at the emitter (#164: in the game exp-lrg-add's start over
      its sphere's 1-3.3 m radius, exp-lrg-fire's puffs over its 2-4.6 m box, 0x2a8a90,
      0x2b17c0), and the fire puffs, shape 2, are drawn at half the game's size (`SHAPE`,
      `Compiled::shapes`, off for the grenades).
    * #85 take21 (e34, a Frag ~11 m from Tex behind a crate) shows the game's blast the same
      way: a fireball as wide as the crate stack, flat on the ground, brown smoke from +0.6 s.
      Ours there (`-- --test BF_MAP=e34 BF_NO_SQUAD=1 BF_CHARACTER=3
      BF_TEST_GOTO=-49.93,22.79,-39.0,26.4 BF_TEST_GO_AT=100 BF_TEST_GRENADE_TYPE=Frag
      BF_TEST_THROW=2.0,0.3 BF_CAMERA_PITCH=-0.25`, the Frag 15 m out at (-35.07, 28.90, 27.03),
      frames 76-86;
      `t286dev/side_e34.png`): the same round, narrower fireball behind the crate.
  * Added for the Light's phosphor_grenade (grenade effects only): an upright streak - an
    appearance without AngleFadeOut or motion blur whose width factor (04b5fc1b) stays under
    half its height factor (0db2cc8d) all its life, with a constant Rotate - rises by its
    appearance transform's offset over its age (`UPRIGHT_WIDTH`, `Pair::rise`). Of the grenades'
    appearances only phosphor_grenade_init_spike.app is one (a billboard, width 0.05 -> 0.30 of a
    size peaking at 5.9 m; offset 0 -> 2.26 m up over 0.92 s): the recording's tall thin blue
    beam. It was also the one quad kept from the demo's random roll, which drew a fan of long
    rays; since #286 no quad has a roll (the game has none, [maps.md](../maps/ale-blends-and-particle-facing.md) **Particle
    facing**). The offset is read over the particle's age, not the effect's time (an inference:
    over the effect's time, repeating every 0.92 s, the whole beam would rise and drop back each
    0.92 s; the recording's beam top flickers by ~25% about every 0.3 s with no slower cycle).
    The 0.5 threshold is the demo's. A billboard is upright on the screen, not in the world:
    under a steep camera pitch the streaks lie back with the view.
  * Also for the Light: phosphor_grenade-shrap.rect.app (spark.tga, aspect 2.2 -> 0.25, ~100/s
    at 4-6 m/s within 49 degrees of up) is facing 3 with AngleFadeOut: since #286 a streak along
    its motion facing the eye, as the game's shader builds it, the recording's starburst of rays
    fanning up from the flare at ignition and the sparks round its base after (lg 0397-0411,
    1073-1085, 1200). Before, the demo drew it so by a rule of its own (`SPARK_ASPECT`: a perp
    appearance born longer than 1.5 times its width on a fast cone), which lying flat had drawn
    as short horizontal lines. Their gravity field stays off (#101), so the sparks fly farther
    than recorded (up to ~5 m, faint by then, where the recording's stay within ~1-2 m).
  * Also for the Light: a steady light - a "light_" effect whose emitter has a constant rate
    at which its particles overlap two deep or more - starts each particle at the moment
    within the frame its rate says it's due (`steady_rate`), evenly spaced. Only light_phosphor
    is one (5.99/s, 0.514 s: ~3 at once). **Until #272** each particle was a light of its own,
    which this lit up over its first 1/rate s and down over its last (x rate x life / (rate x
    life - 1)) so that their sum stayed constant: as plain particles the number lit stepped
    between 3 and 4 and the ground light pulsed by 3-11% about three times a second, where the
    recording's was "steady within 1/255" (the tester's measurement). **The console never summed
    them**: it has one lamp for the effect, its newest particle's (levels.md "Effect lights"),
    and the recording's ground light has the sawtooth that gives, in red (2.8 levels peak to
    peak, a period of about 11 frames at 60 fps), with green and blue within 0.6: the "steady"
    was a reading of those. The ramp is gone. The Frag's and the Sentry's light_explosion (one
    particle) and the Sonic's light_sonic_grenade (a keyed rate) were never steady.
