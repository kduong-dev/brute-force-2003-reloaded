*Part of [levels/](README.md), the former page `levels.md`.*

* **Grass.** Bulgar's grass clumps are see-through cards standing upright with their normals
  lying flat (every normal's y is 0). Lit by those, they caught almost none of the high key
  light and their backs none at all: they were black. Cut-out cards whose normals all lie flat
  (|y| < 0.1) are given normals pointing up, so they're lit like the ground, both faces alike.
  `BF_MAT_LOG=1` lists the materials (type, textures, average colour).
* **Shadows.** The scenery and the terrain cast no shadow; only characters do, from the
  brightest light of the outdoor pair (on e34, whose outdoor pair is two h_e1cac6f7, both cast
  one before #143). The game lights static geometry per vertex by the level's lights (every
  light-object is `enable-static`), with no shadows. With shadow-casting roofs interiors were
  black, and a jungle canopy darkened the whole floor. The terrain cast one until #143: a
  character standing where it shades the ground was drawn black (Tex on e40's beach, which
  take17 of todo/73 shows lit), and it darkened sdm_e34's start ravine's grass and the water
  along sdm_e40's shore.
* **Checked** (#143; `BF_MUTE=1 BF_BACKGROUND=1`, frames beside the takes):
  * e40 at take04's spot (`BF_MAP=e40 BF_TEST_GOTO=-16.00,152.32,-16.50,152.26
    BF_CAMERA_PITCH=-0.10 BF_TEST_WEAPON="Therm Sweeper"`, frame 30, todo/73 take04 1.5 s): Tex,
    Flint, Brutus and the standing stone are lit (main drew them black); the stone's lit face
    measures 76 55 42 against the take's 77 55 43. The cliff (terrain, bluish here, #181) isn't
    this. At take17's spot (`BF_TEST_GOTO=17.15,175.43,16.70,175.22`, the rest the same, frame
    39 against take17 1.5 s) Tex is lit (his arm 72 38 27 against the take's 94 47 35) and the
    two near stones measure 75 53 42 and 78 57 43 against 83 60 47 and 61 44 34; while the
    terrain cast a shadow they were black (17 10 7, 10 9 8) and so was Tex.
  * The tutorial's opening (`BF_MAP=tutorial`, frames 285-375 against take03 at the same clock +
    1.66 s): Tex lit as the take has him (vest 146-161 luma against the take's 133-138, his
    shaded arm 50 against 52; main 1-12). The second building (`BF_TEST_GOTO=37.22,-88.80,36.9,
    -88.55` against take11 2.5 s) and the base building (`-62.22,19.23,-62.40,19.47` against
    take12 300 s): their insides take the bluish indoor pair as the takes show (the base
    building's wall 33 54 74 against take12's 34 39 51; main black and white); the gate wall
    (`-43.76,-13.46,-43.81,-13.16`, take11 715 s) shows its dark blue panels where main drew it
    black; the sniper crate and the base's containers are lit (take11 297 s, take12 50 and
    120 s).
  * Bulgar's interiors: inside sdm_e34's building at (-51.2, 10.9) (`BF_MAP=sdm_e34
    BF_TEST_GOTO=-51.2,10.9,-48.2,10.9 BF_TEST_GO_AT=99`) the walls measure 34-42 47-57 55-65,
    where main had 17-28 20-33 24-37: #2's capture measured 40 55 60 and the demo 23 27 32.
  * Unchanged: by the lights, sdm_e40's and sdm_e34's start views are the same pixel for pixel;
    the terrain's shadow gone lightens the grass in sdm_e34's ravine (2.3% of the frame) and the
    water along sdm_e40's shore (0.3%). e34 at take01's spot differs only by Tex's second
    shadow. sdm_e40's ruin at (52.9, -54) has 27 indoor geosets and its 6 indoor lamps: it's now
    a dim, green-lit room where the outdoor key lit it (no take of it); its floor, part of the
    ruin, takes those lamps. The Vengar view is the same as main's (to within 1 of 255) inside
    an e34 building (`BF_MAP=e34 BF_CHARACTER=0 BF_TEST_GOTO=-51.3,10.4,-48.3,10.4
    BF_TEST_GO_AT=99 BF_TEST_SKILL=2.0`, frame 74), looking up at sdm_m07's indoor-lit ceiling
    (`BF_MAP=sdm_m07 BF_CHARACTER=0 BF_TEST_GOTO=27.3,2.6,27.3,4.0,87.5 BF_TEST_GO_AT=99
    BF_TEST_SKILL=1.0 BF_CAMERA_PITCH=0.75`, frames 40 and 59) and at mp2's start with
    `BF_NO_LAMPS=1` (frame 44).
  * The 15 maps that had lost a directional light (e05, e06, e10, e40, m03_a/b/c, m05_a/c/d/x,
    m08_a/b, mp8, the tutorial): their start views now light the squads and the scenery the
    outdoor pair faces (m03_a's, m05_a's and m08_a's squads were black). Lamps: m04_c's new red
    lamps light its rock spikes and nothing burns out, but no capture of m04_c's lamps (nor of
    any other cluster of them) exists, so their look is untested against the game. Frame time
    (no vsync, median of 40 s, before the terrain stopped casting shadows) m07_a 4.62 -> 4.90 ms
    (85 -> 184 lamps), m04_c 5.43 -> 6.04 ms (160 -> 181).
