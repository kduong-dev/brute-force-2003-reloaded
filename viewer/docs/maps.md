# Playing on a map

`bf_play` now runs on Battle of Bulgar by default (`BF_MAP=<level>` picks another level,
`BF_MAP=flat` brings back the checkered test floor, `BF_START=<n>` picks a start point,
`BF_TEST_GOTO=x,z,x2,z2[,y]` starts the player at one point (on the floor below height y if
given, e.g. a roof) and runs them toward another, or stands still when both points are the same;
`BF_FIND_STEEP=1` lists slide surfaces steeper than 50° and ledges with a drop of more than 6 m,
then exits; `BF_SLIDE_LOG=1` prints slides, falls and landings; `BF_ALE_LOG=1` prints particle emission;
`BF_NO_IDLE_FX=1` leaves out the power-ups' effects):

* **Spawning.** You start at the level's start point. Points 0–3 are one squad's spawn on the
  hill, 4–7 the other's, and 9 is in the base.
* **Collision.** It uses the game's own collision surfaces (`src/bf/collision.rs`,
  `src/arena.rs`). On sdm_e34 that's 43,348 triangles.
  * Where they come from: `objects-<level>.ipn` holds them in the Ipion physics engine's format.
    The terrain blocks, the 23 invisible blockers and every placed object's parts each name
    theirs. Door leaves have their own and stop blocking while open.
  * Material: every triangle carries one, a world-material id plus a slide flag.
  * Standing: characters stand on surfaces up to 65° (the game's ground limit) within a 0.5 m
    step; steeper surfaces are walls.
  * Not solid: grass, wires and other undrawn or cut-out parts have no collision surface, so
    they don't block.
  * Fallback: a level without collision data uses the drawn triangles instead.
* **Doors and gates.** Their leaves sit on sliding joints (joint type h_fd8f670c). Doors open
  when anyone is within 5 m, close again afterwards (the opening played backwards), and stop
  blocking while open. Gates with wall panels open from those instead (below).
  * Motion: the level's animations-<level>.xmb has an archetype-set named after each door's
    archetype, with one target per leaf (by part name) on a float channel: metres along the
    joint's axis per frame at 24 fps. Doors take 1.21 s (upper leaf 1.89 m, lower 1.70 m); the
    gate takes 4.96 s (3.92 m).
  * Sound: a sound-trigger stands beside each door (within 6 m). Signal 31 plays as it opens
    (h_1edc357e, metal_door_1a; the gate's h_e1b1f167, an 8.6 s grind) and signal 49 as it
    closes (h_f25b1597). These waves are streamed (resource-type 1) from the language wave bank
    `ml-sounds/en/<level>-en.tgz` at the offsets in the level's sound bank.
  * The gate's two leaves have a clip each in its set (h_10d3fde3 and h_0a48ec76, 4.96 s, one
    target each); each leaf takes its channel from whichever clip names its part.
* **Gates and their wall panels.** A gate with wall panels opens only from them:
  * The panels are `world-button-object`s (h_16f22d4b, type h_ee4c83c9, `reticule-action` 1)
    sending signal 14 (h_eaf8a35b).
  * A `router-trigger` lists the panels among its objects (h_0b6b92ba) and passes their signal
    on (h_e91cf6a8) to the gate's `anim-trigger`s. Those animate the gate object (their own
    h_0b6b92ba) with one clip per leaf on signal 14, once only (h_f136d22d 1, where the
    proximity doors' signal 50 / 42 entries say 2147483647), and send on to the gate's sound
    trigger. So a used gate opens and stays open. On sdm_e34: panels h_14643319 / h_1d63038f ->
    router h_1c61561d -> anim-triggers h_f993d7ef / h_e8ad85ec -> gate h_14690535.
    `Level::button_opens` follows that chain.
  * Using one (capture todo/gate button.mp4): from in front of its button (hardpoint
    h_10f81d34, facing the model's +z), within 2.5 m and looking within ~45° of it, hold E for
    0.5 s. Reach, angle and hold time are guesses; the capture shows no progress bar and no
    gate message. While it's usable, "Hold E to activate panel." shows under the health bar.
    That's the game's "Hold " + its X-button icon + "to activate %s." with "panel", and the
    key here in place of the icon. The blue target ring (the four-notch reticle texture
    h_1d2a68d0, tinted) sits on the button.
  * The button glows green: material h_f87dca06 is shader type h_1df292a1 (851 materials), a
    lit shader whose glow texture h_e01baa40 is tinted by its constant of the same name
    (0.276 0.914 0.220 here) and added as light.
  * `BF_DOOR_LOG=1` lists the panels, what each opens, and the gates with panels.
    `BF_TEST_USE=<s>` presses use at that time.
* **HUD text** (`play_text.rs`) is drawn in the game's own body font, atlas h_e4e4d2f4 (as the
  menu draws it), its glyphs 1 atlas pixel apart over a black outline and shadow, as in the
  captures. Colours sampled from them: messages and prompts pale blue (186 210 249), ammo and
  NEW orange (254 165 104), an item that can't be used red. Capitals are ~11.5 units (14 for the ammo).

* **Movement constants from default.xbe** (see `decompiled/xbe/ghidra/README.md`, "Player
  movement"):
  * Gravity is 18 m/s² and the jump takes off at 5.8 m/s. In the air, horizontal speed keeps
    0.99 of itself per game frame (taken as 30 Hz).
  * Walking off a drop of more than 0.5 m is a fall.
  * Landing after a drop of 5 m to 30 m costs 100 × ((drop − 5) / 25)² health (a 10.6 m fall
    cost 4.9).
* **Sliding** works as the game does it, and only on collision triangles flagged as slide
  surfaces (the canyon walls).
  * Building up: a slide amount (0–1) grows while you move. On ground steeper than 50° it grows
    at up to 8 per second, at the full rate from 55°; flagged ground always counts at least half
    rate. At 1 the slide starts.
  * Sliding: it carries on until the ground eases under 45°.
  * Speed: the slide velocity starts at (1 − normal y) × 8 downhill plus 5 m/s down, then
    builds by that push 3 times a second, up to 40 m/s. You move by it times the amount.
  * Ending: under an amount of 0.2 your own movement blends back in. A slide that's blocked (a
    wall, or flat flagged ground where it only presses down) ends.
  * What you see and hear: the slide clip (motion "slide", script Sc_w1_slide_idle), facing
    down the slope, the surface's slide sound, and the slide dust (effect type h_01513b0f, ALE
    effect slide_puff) at your feet.
  * Steep but not flagged: unflagged slopes up to 65° can be walked up.
* **Music.** The game streams `data/sounds/<level>.xwb` by level name (sdm_e34's is
  caspian_action20, 59.7 s, stereo Xbox ADPCM, 22050 Hz). A bank can hold a music track and an
  ambience bed; the level's sound bank gives each its Type (`h_f92fb2b9` music, `ambient`; for
  names it doesn't define, an `amb` prefix is ambience). The music loops at half volume with the
  bed under it at 80% of that; a bank without music plays all its beds. Before, the first wave
  played, which on sdm_e13 and sdm_m03 was the bed, so their music was missing:

  | Map | Music | Ambience |
  |---|---|---|
  | sdm_e01 | caspian_action1 | |
  | sdm_e10 | ferix_action8 | amb_ferix_02 |
  | sdm_e13 | tmp_full-on1a-f | amb_singe_02 |
  | sdm_e34 | caspian_action20 | |
  | sdm_e40 | creepy_estuary_mutant_01 | light_bird_bed_1 |
  | sdm_m03 | caspian_action5 | amb_casp_04 |
  | sdm_m07 | | amb_shad_int2, ambiencehellish-2ch |

  `BF_NO_MUSIC=1` turns it off, `BF_MUSIC_VOLUME=<0-1>` sets the level, and
  `BF_DUMP_MUSIC=<file.wav>` writes the track and exits.
* **Shots.** Each weapon's bullet definition names its own effects, and they're played from
  the effect library: `effect-name` is the shot in flight, h_08d28037 is where it lands.
  | Weapon | Speed | In flight | Hit |
  |---|---|---|---|
  | LZR-23 laser | 150 m/s | laser_bolt_s + laser_bolt_rifle_s, a glowing bolt | laserhit_s, laserhitb_* |
  | Laser pistol | 100 m/s | laser_bolt_s | laser hits |
  | Brutus's cutter | 120 m/s | cuttertrail_ribbon, a bluish ribbon trailing the shot, and cutter_flare | — |
  | Minigun, ballistic guns | instant | `tracer`, one particle shot from the muzzle at 113.6 m/s | sminigun_hit |
  | L-Shot | instant | tracer_snipe, a white beam drawn 200 m down the line in 0.38 s, smoke, rings | sminigun_hit |

  * Bolts carry their effect with them: the emitters are flagged "attached" (h_e2999ffd), so
    their particles move with the shot.
  * Hit effects play on world hits; hits on bodies bleed instead (play_fx).
  * A shot's damage (friendly fire) lands when it reaches the body: distance / speed later.
  * Beam appearances (class h_1f55f13e) are ribbons through their emitter's live particles in
    birth order, facing the camera. They have their own colour, alpha and width parameters, and
    their texture runs along v.
  * A frame's new particles are spread over the frame: along the emitter's animated offset
    (the sniper beam's sweep) and the effect's own movement (trails behind fast shots).
  * Effect libraries refer to their nodes by the case-kept name hash (e.g.
    "laserhit_Cone.emt"). Matching that recovered the emitters of 262 effect parts, the laser
    hits among them.
  * Test hooks:
    * `BF_TEST_FIRE=1` (or `2` for the second weapon) makes a `BF_TEST_GOTO` player aim and
      fire.
    * `BF_TEST_TARGET=1` stands the first squadmate still, 8 m in the line of fire.
    * `BF_CAPTURE_FPS=<n>` sets the capture's fixed step (default 15).
    * `bf_level` with `BF_FX_TEST=<effect name>` (and `BF_FX_MOVE=<m/s>`) runs one effect in
      front of the view.

* **Look.** The game lights each surface `texture × (ambient + key·N·L + fill·N·L)` with the
  level's own light colours, in gamma space, with no tone mapping. The demo does the same
  (`level_scene::spawn_lighting` and `console_look`):
  * Lights: the level's key (with shadows) and fill directional lights and its object ambient,
    given as sRGB colours.
  * Camera: tone mapping off, and an exposure where a light of 1.0 leaves a texture as it is.
  * Terrain: the level has a separate, dimmer key light for terrain (a bluish 0.44, 0.57, 0.65 on
    sdm_e34), so the terrain is tinted by its ratio to the objects' key light.

  Before this, an invented 9000 lux sun, a strong ambient and Bevy's default tone mapper (which
  desaturates) made everything pale. Point lights (muzzle flash, hit glows, explosions) are
  scaled to the new exposure by `POINT_LIGHT_SCALE`.
* **Power-ups.** Each power-up type's idle event (`<events><event state="1" h_ed0c9fac=…>`)
  names an effect type. Its objecttypes `<effect>` entry lists the ALE effects:
  | Power-up | Effect | What it draws |
  |---|---|---|
  | Brute (quad) | multi_quad_icon | godray.tga rays |
  | Force (shield) | multi_shield_icon | brokenring.tga ring, plus the animated texture "arcb" (4 × 4 frames at 30 fps) |
  | Health | multi_health_icon | red-flare.tga |
  | Stamina | multi_stamina_icon | blue-flare.tga |

  `src/ale_fx.rs` runs them from the effect library's own parameters (see that file).
  * These are "perp" particles: flat in the emitter's frame. The shield's arcs emitter is turned
    −87.6° about x to stand its arcs up.
  * Each particle keeps the appearance's turn at the moment it's born (about 220°/s), so the
    particles fan out round the icon.
* **Pickups.** They're stored at their spawn height in the level file; they now settle onto the
  floor below.
* **Compound joints.** A compound object's joints carry a rotation (a w-x-y-z quaternion),
  applied to the part (the wall sets turn pieces by 90° and 180°). A part sits at the joint's
  parent-point + child-point, both in the parent's frame; the rotation turns only the part about
  its own origin. Bulgar's compound `h_0baf091a` (hangar, bunk rooms: 19 pieces) is the one placed
  object with non-zero child-points, and only this reading puts every piece inside the building:
  read as parent-point − rotation × child-point, its roof bays landed outside the hangar and its
  turned bunks went through the walls (issue #12). `BF_OVERLAP_LOG=1` (bf_level) lists large
  objects whose boxes overlap much.
* **Winding.** Object triangles are wound to face along their stored normals, so a mirrored mesh
  isn't culled from the front.
* **The gate (h_0792d16a) and texture address modes.** Its two leaves (h_e783bd32 → mesh
  h_e01d9aa6, h_fcd25d86 → h_178bc697) are the same panel shifted 0.996 m, with matching UVs
  and no joint rotation. The mirroring is in the texture's sampling: each leaf runs its
  texture u 0..2 across, and the texture entry's `address-u` / `address-v` (h_1c82a17b /
  h_078bf0c1) is `TAM_MIRROR` (h_18712765), so the second half is the first mirrored and each
  leaf shows one symmetric brace. The other modes are `TAM_WRAP` (h_e3012d9e, most textures)
  and `TAM_CLAMP` (h_e041e0b9). Every level and model texture is now sampled by its own modes
  (`Game::texture_address`); 24 levels use mirroring somewhere.
* **Invisible or glowing materials.**
  * Power-ups (Brute / Force / Health / Stamina) use a placeholder cube whose material no level
    defines; it isn't drawn.
  * Untextured glow materials (type h_f539fe8c, e.g. the medkits' shell) are drawn as their
    glow colour at the wrapper material's opacity.
* **Hits.** Shots, grenades, decals, the memory chip and the follow camera use the same collision.
