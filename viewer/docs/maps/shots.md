*Part of [maps/](README.md), the former page `maps.md`.*

* **Shots.** Each weapon's bullet definition names its own effects, and they're played from
  the effect library: `effect-name` is the shot in flight, h_08d28037 is where it lands.
  | Weapon | Speed | In flight | Hit |
  |---|---|---|---|
  | LZR-23 laser | 150 m/s | laser_bolt_s + laser_bolt_rifle_s, a glowing bolt | laserhit_s, laserhitb_* |
  | Laser pistol | 100 m/s | laser_bolt_s | laser hits |
  | Brutus's cutter | 120 m/s | cuttertrail_ribbon, a bluish ribbon trailing the shot, and cutter_flare | — |
  | Minigun, ballistic guns (bullet-type 4) | instant | `tracer`, one particle shot from the muzzle at 113.6 m/s along the shot, on every (h_eeb9e75a + 1)th shot (MK every 3rd, Minigun every 4th) | sminigun_hit |
  | L-Shot | instant | tracer_snipe, a white beam drawn 200 m down the line in 0.38 s, smoke, rings | sminigun_hit |

  * Bolts carry their effect with them: the emitters flagged "attached" (h_e2999ffd, the
    demo's reading) and, since #57, every emitter of a bolt's flight effect but its trails
    have their particles move with the shot and go with it; a trail (an emitter under the
    library's world node h_ee223b51 with its appearance at the root) leaves its particles in
    the world. See play.md, **Bolts and their hits**.
  * Hit effects play on world hits with the struck material's debris, hit sound and the
    effect's light, each turned by its alignment-type; on bodies the flesh row's effect plays
    (made by hand for the squad's ballistic, cutter and laser rows, from the data for the rest)
    with the blood (play_fx).
  * World hits also leave the bullet's h_06a27365 hole or scorch on the collision triangle they
    meet, and each gun has its own muzzle effect and casings; the fire timing, pellets and
    accuracy spread are in [play.md](../play/shots.md), "Shots".
  * A shot's damage (friendly fire) lands when it reaches the body: an instant ray's at the
    distance / the demo's 400 m/s, a bolt's on the sweep that finds it, k x 0.1 s later
    (play.md, **Bolts and their hits**; its
    factors: the hit part struck, x0.7 on the own side, the difficulty step, none on the
    multiplayer maps as read live in a match, STANDARD on the missions: play.md **Damage**).
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
    * `BF_CAPTURE_EARLY=1` captures from the map's first frame, with no warm-up and no wait for
      the character or the LOADING cover (#311, menu.md "The level's start behind LOADING").
    * `bf_level` with `BF_FX_TEST=<effect name>` (and `BF_FX_MOVE=<m/s>`) runs one effect in
      front of the view.
