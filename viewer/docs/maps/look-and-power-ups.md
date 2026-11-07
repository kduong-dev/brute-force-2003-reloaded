*Part of [maps/](README.md), the former page `maps.md`.*

* **Look.** The game lights each surface `texture × (ambient + light₁·N·L + light₂·N·L)` with
  the level's own light colours, in gamma space, with no tone mapping. The demo does the same
  (`level_scene::spawn_lighting` and `console_look`):
  * Lights: the level's outdoor pair of directional lights (the brightest casting the
    characters' shadow; the scenery and the terrain cast none) and its object ambient, given as
    sRGB colours; scenery placed indoors takes the indoor pair instead ([levels.md](../levels/terrain-hue-and-light-objects.md),
    "Light objects", #143).
  * Camera: tone mapping off, and an exposure where a light of 1.0 leaves a texture as it is.
  * Terrain: tinted by the hue of the lights marked h_e02aba1c (a bluish 0.44, 0.57, 0.65 on
    sdm_e34), read as the terrain's own before #143; the game's code has them as the indoor
    pair, and what lights its terrain is #181's.

  Before this, an invented 9000 lux sun, a strong ambient and Bevy's default tone mapper (which
  desaturates) made everything pale. Point lights (muzzle flash, hit glows, explosions: the
  light effects' lamps) are the game's lamp, colour x (1 - (d / range)^2) x N.L, not physical
  1 / d^2 lights: src/lighting.wgsl, [levels.md](../levels/point-lights.md) "Point lights" and "Effect lights"
  (#272; they were scaled to this exposure by `POINT_LIGHT_SCALE`, which is gone). The falloff
  is read from the game's lamp shaders (pixel shaders 0x37 / 0x38, vertex programs 0x50, 0x51
  and 0x1a-0x35: `FUN_0009cc60`, `0x3e1310`, `0x3daf70` in [code-map.md](../code-map/README.md)); the
  lights' `falloff` kinds 3 and 4 (#3) are read by none of it.
  * Textures (#257): every world texture (terrain, scenery, sky, liquids, characters, weapons,
    effects, decals) is drawn with the mip levels its archive ships, linear within and between
    levels and with no anisotropy, as the console samples them; before, each was its top level
    alone, so the ground sparkled past a few metres and showed raw texels close up. The HUD and
    the menus are unchanged. The terrain also mixes in its detail texture near the camera: the
    same texture mostly 3-5 times finer (0-16 over the data), fading out mostly 15-25 m away
    (0-35). Both are in
    [levels.md](../levels/terrain.md) (Terrain layers), with their hooks (`BF_NO_DETAIL=1`,
    `BF_ANISOTROPY=<1-16>`).
* **Power-ups.** Each power-up type's idle event (`<events><event state="1" h_ed0c9fac=…>`)
  names an effect type. Its objecttypes `<effect>` entry lists the ALE effects:
  | Power-up | Effect | What it draws |
  |---|---|---|
  | Brute (quad) | multi_quad_icon | godray.tga rays |
  | Force (shield) | multi_shield_icon | brokenring.tga ring, plus the animated texture "arcb" (4 × 4 frames at 30 fps) |
  | Health | multi_health_icon | red-flare.tga |
  | Stamina | multi_stamina_icon | blue-flare.tga |

  `src/ale_fx.rs` runs them from the effect library's own parameters (see that file).
  * The icons' appearances are GeneralApp_FacingType 2 (square to the particle's motion) with
    AngleFadeOut, on sphere emitters that throw their particles out from the middle (0.025 m/s,
    0.17-0.21 m out): since #286 each flare faces out along its own way and fades near edge-on,
    as the game's do (**Particle facing** below), a glowing ball as #35 take20 shows at 2.0 s (an
    oval glow ~1 m across). Before, they lay flat in the emitter's frame ("perp"): a disc, thin
    from a low camera.
  * Each particle's space keeps the appearance's turn at the moment it's born (about 220°/s): it
    sets where a flare's side starts (`AleParticle::space`).
  * Walking over one takes it ([pickups.md](../pickups/power-ups.md), "Power-ups", #15).
