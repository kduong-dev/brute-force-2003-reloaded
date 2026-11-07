*Part of [maps/](README.md), the former page `maps.md`.*

* **ALE blends (#228).** An appearance's blend pair (h_1daea8c0) is the PC's D3DBLEND (src,
  dest), which the game turns into the console's through the table at 0x4036b4 (0x2a6f00): 5,2
  (SRCALPHA, ONE) on 518 appearances, 5,6 (SRCALPHA, INVSRCALPHA) on 207, 3,2 (SRCCOLOR, ONE) on
  33 (the lava's eruptions slava-erupt*, its wading rings and splashes env_sl_*, the teleport and
  ion beams), 2,2 (ONE, ONE) on 13 (slava-fuzz, the laser and flesh hits, campfire sparks), and
  a few 3,6 / 6,6 / 2,6 / 4,6. Every pair with a dest of ONE adds the particle to the picture
  (before #228 only 5,2 did: the others were alpha blends, and the tutorial's lava bursts came
  out as brown balls). The particle's colour is the texel x its vertex colour and its alpha the
  texel's x its vertex alpha, side by side (pixel shader 0x3b89e0, which FUN_0029f5d0 sets for
  any pair whose dest isn't INVSRCALPHA; the vertex colour is the appearance's colour with its
  alpha curve in the top byte, FUN_002a1a50, and the particle vertex shader 0x3b8bf8 passes the
  colour through): so the alpha curve fades a SRCALPHA particle only. With ONE the particle adds
  texel x colour whatever its alpha, with SRCCOLOR (texel x colour) squared: drawn here with the
  alpha at 1, the texture's alpha opaque and, for SRCCOLOR, the texels and the colour squared
  (`ale_fx.rs` `Source`). A beam's ribbon keeps its colour x alpha whatever its pair (its own
  draw isn't read).
* **Particle facing (#286).** Every effect's quads are turned as the game's particle vertex
  shader 0x3b8bf8 builds them (read in full; `ale_fx.rs` `FACING`, `Facing`), by the
  appearance's GeneralApp_FacingType (e3542300), which FUN_002a1a50 hands the shader as a one-hot
  selector whatever else the appearance says:

  | Facing | The quad | On (of common.tgz's 784 basic appearances) |
  |---|---|---|
  | 0 | its particle space's own x and y (the effect's turn and the appearance transform's), turned over its life by GeneralApp_ParticleTransform where ApplyParticleTransform is set (FUN_002a2850) | 20: the liquids' and the rain's rings (env_xl_*_rip, xwater_ring: turned 90 degrees about x, so they lie on the surface), the L-Shots' tracer_snipe_orient |
  | 1 | the camera's right and up: a billboard | 411: smoke, fire puffs, flashes, the embers |
  | 2 | square to the particle's motion: its height along the particle's side, a vector that starts as its space's y (x when the motion points up past 0.95), kept square to the motion | 70: the Sonic's dome, the power-ups' flares, the Rails' and L-Shots' rings, the teleport rings, powerup_spawn's ring, the leaves |
  | 3 | its height along the motion, its width square to the motion and to the way to the eye: it faces the eye about its motion (Freelancer's "rect") | 283: the Frag's and the Therm Sweeper's fireballs (exp-lrg-add, exp-rocket-add), every .rect spark, the tracer, the muzzle flashes |

  The motion is the particle's velocity, normalised as it's made and, for 2 and 3, each update
  while it moves (above 1.42e-14 m2/s2; 0x2a14b0, 0x2a17b0); one born still keeps its space's +z.
  Then the quad turns in its plane by its Rotate (radians, clockwise on the screen for a
  billboard), and nothing else: the particle record has no roll, so the demo's random roll on its
  billboards is gone. Two flags change the quad without turning it:
  * **GeneralApp_AngleFadeOut** (0d645074, the flag the demo read as "perp", lying the quad flat
    across its emitter's y): the quad's alpha x clamp((|cos| - 0.0872) x 3.9237, 0, 1) squared,
    the cosine between the facing's normal and the way from the particle to the eye (0x2a1c9d):
    whole beyond 20 degrees from edge-on, gone within 5; a facing-3 quad seen along its motion. It's
    the vertex alpha's, which only a SRCALPHA source reads (ONE and SRCCOLOR quads don't fade).
    Drawn per particle in 32 steps: the quad mesh with that alpha in its vertex colours (bevy's
    StandardMaterial multiplies them in), so the materials stay shared per life step.
  * **GeneralApp_MotionBlur** (fe7c2071): the quad's trailing edge (the corners behind the
    particle along its axis) drawn a game frame's move back (velocity x 1/30 s), a parallelogram,
    placed by its GlobalTransform after the transforms propagate (`place_quads`): by the data, a
    54 m/s rain drop (xrain.app) 1.8 m longer, the hits' sparks 0.1-0.6 m. Not drawn for a quad
    standing on its particle (shape 3), whose trailing corners aren't one edge. Before, the demo
    turned a motion-blurred billboard along its motion instead (and any one taller than 1.5 times
    its width).
  Gone with them, the demo's rules fitted to single recordings: the grenades' outward-facing perp
  quads from fast sphere emitters and the Sonic's cone (`PERP_RADIAL_SPEED`, `PERP_OUT_CONES`:
  facings 3 and 2 now), the Light's sparks drawn as streaks (`SPARK_ASPECT`: facing 3). Kept, as
  guesses: the Sonic's godrays built about the world's up (`VERTICAL_STREAKS`, grenades.md "Sonic
  grenade"), the Light's spike rising by its appearance's offset (`UPRIGHT_WIDTH`), and in the
  holder's scope a muzzle flash's facing-0 and -2 quads lying along its barrel and its smoke
  cards left out (`AleEffect::hide_flat`, play.md "Muzzle effects").
  * Checked (15 fps captures beside main's build and the takes; scratchpad `t286dev/`): the
    Frag's blast on e40 at #119 take01's spot and on e34 near #85 take21's (grenades.md, "The
    blast": round, no flat discs or horizontal streaks, still narrower than the take's, #164),
    the Sonic on sdm_e34 (as before, its dome's rim softer), the Energy on the test map (as
    before), the Therm Sweeper's explosion at #73 take04's spot (as before, a little less white;
    play.md "Missiles"), the MK's muzzle flash from the hip (its flash quads along the barrel)
    and in the scope (the take's soft ball; play.md "Muzzle effects"), the power-ups' orbs
    against #35 take20 (a ball where they were a thin disc; pickups.md), sdm_e13's embers from
    the south shore (unchanged), and in bf_level (`BF_FX_TEST=<effect> BF_SCREENSHOT=<png>`)
    powerup_spawn (its white rings lie flat as they rise, where they faced the camera) and the
    water's wading ring env_xl_wadering (facing 0: concentric rings flat as on water, where they
    faced the camera). Not seen: the teleport rings and the leaves (`BF_FX_TEST` runs at user
    parameter 0, where teleport_base emits nothing; xleaves showed nothing either).
* **Effects the triggers make** (the tutorial's lava bursts, its farcast flashes) are compiled
  with the game's particle rules, the grenades' and the Therm Sweeper's (`load_recorded_facing`:
  the shapes, the count once a game frame, the facing; [triggers.md](../triggers/spawn-triggers.md),
  "Spawn-triggers"). take08's lava burst (slava-erupt-jet: small-flare at 3,2, 100 a second for
  0.6 s, 0.47 -> 1.7 m) is a bright fire column that rises and fades within a second; with the
  demo's own rules (half-size quads, its count) it was a string of small blobs.
