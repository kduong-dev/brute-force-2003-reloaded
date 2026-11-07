*Part of [levels/](README.md), the former page `levels.md`.*

* **Terrain hue.** The baked light map gives the ground's brightness. Its hue comes from the
  lights marked `h_e02aba1c` with the terrain's ambient, at 60% strength. Before #143 those were
  read as the terrain's own pair; the game's code has them as the indoor pair (see **Light
  objects**), and what lights its terrain is #181's, so the hue is kept as it was calibrated. If
  vertex program 0x38 is the terrain's base pass (see **The detail**: no code was found loading
  it), its vertex light is the Terrain `<ambient>` plus slot 1's light of whichever pair was
  uploaded last (c18 / c19, x the material's colour; the pair's fill where it has a key and a
  fill), each view's draw starting with the outdoor pair (FUN_000dcc10(0, ..) at 0xda2aa), and
  the bump map takes slot 0's (c16, the key; **The bump map**), all of it x the vertex's baked
  light. So the outdoor pair, not the `h_e02aba1c` lights, would reach the ground. A lead for
  #181 (inferred: c26 and c27, which decode the baked light's byte, are set for no terrain
  program): read as its low 7 bits / 127.5 x (the ambient + the pair's N.L), e40's beach comes
  out about 0.6 of the demo's whole-cell light, and take06's beach is darker by about that (25
  luma against the demo's 35, the same view).
  On Bulgar those lights are blue-grey, and the captures show grey ground and hills, not sand.
  The 60% is calibrated on a capture of the gate: the ground there measures 73 81 79; ours was
  79 74 64 and is now 68 77 74. `BF_DOOR_LOG=1` lists the doors (type, place, leaves); Bulgar's
  big gate is `h_0792d16a` at (-23.2, 74.3).
* **Light objects** (#143). A level's `<light-object>` is read by its type's `light-type`
  (objecttypes `<light><h_e275fb80 light-type=N>`, parsed by 0x192970 through the LT_ names at
  0x3be720: 1 point, 2 spot, 3 directional; `Game::light_types`), not by a list of type names.
  The game's data has four directional types (h_e1cac6f7 and h_f5302dc6 in common, h_157b3160
  in e05, e40, m03_a/b/c, m08_a/b, mp8 and the tutorial, h_e8d6efe4 in e06, e10, m05_a/c/d/x)
  and six point types (h_ea460e64, h_09020c30 and h_0b51040f in common, h_e81ba579, h_087ae059,
  h_f1557cc8, h_ff66877a); no spot. The three directional types' records are alike: nothing in
  them says key or fill. Before #143 the demo kept h_e1cac6f7, h_f5302dc6 and h_ea460e64 only,
  so e40 and the tutorial lost their main light (h_157b3160) and drew their squads black, 15
  maps lost a directional light and 959 point lights were dropped.
  * **Two pairs.** The game lights an object with two directional lights at most
    (FUN_000dcc10): the *outdoor* pair, the lights flagged `h_1f416687` (the light's +0x94 bit
    2), or the *indoor* pair, those flagged `h_e02aba1c` (bit 1); a light may be in both (the
    tutorial's fill). The object's `light-usage` picks one (the wrapper material's draw
    0x9f3c0): LU_OUTDOOR the outdoor pair, LU_INDOOR the indoor one, LU_GEOSET each geoset's
    own `h_f89cd77d` (`Geoset::indoor`, a building's inner faces: the tutorial has 59 such
    geosets on 16 meshes), LU_NONE none (here the outdoor pair, the one each view starts with).
    Each pair takes its lights from the level file; no level has more than two in one. Its two
    slots (c16 / c17, c18 / c19) light an object alike; FUN_000dcc10 gives slot 0 to the pair's
    key light, the one whose type has enable-specular (#288, **The bump map**: only the
    terrain's bump map and the lava's v0 tell them apart). The demo read `h_e02aba1c` as "lights
    the terrain" before; it's the indoor pair.
  * **Here.** The outdoor pair are bevy's directional lights (`spawn_lighting`), at PI lux with
    their sRGB colour, the brightest (by luma) casting the characters' shadow: the game's
    lights cast none, so the one shadow is the demo's choice. Characters take the outdoor pair
    (535 of the game's 542 placed characters are LU_OUTDOOR). A placed object's indoor-lit
    parts get an indoor variant of their material (`Lamps::indoor`, shader def INDOOR_PAIR):
    `src/lamps.wgsl` takes bevy's directional lights that are the outdoor pair out of
    `apply_pbr_lighting`'s result exactly as it put them in (with the shadow) and puts the
    indoor pair in by the same diffuse and specular. It knows the outdoor pair's lights by their
    direction and their colour (`light_buffer` gives it both): by the direction alone the Vengar
    view's light from below passed for sdm_m07's outdoor key, which also shines from below. In
    the Vengar view, whose vision pair replaces both pairs in the game, nothing is swapped: the
    outdoor pair is put out then, and on a level with no outdoor pair the indoor pair is lit only
    while bevy has no directional light at all. A level with no outdoor pair (mp2: both its
    lights are indoor) lights its outdoor objects and its characters with the ambient alone, as
    the game's code does; one with no directional light at all gets a made-up sun.
  * **Strength.** The shaders take N.L against the light matrix's third column as it is, not
    normalised (vertex program 0x19's `max(0, n . c16)`): a hand-made transform whose column
    is short lights less. 34 of the 161 directional placements are off unit length by over
    2% (e40's key 0.944, its outdoor fill 0.834); here a light's colour is scaled by it
    (`LevelLight::lit_color`).
  * **Linked lights.** A `<parent Type="1">` puts a light at its parent object times the
    link's offset (FUN_0015f0b0, FUN_0015ed30); 373 point lights (all 159 h_09020c30, 214
    h_e81ba579, white, range 10) hang on effect objects, whose effect types name them and run
    them with an ALE light-effect: the placed effects draw those lights (#258), so these aren't
    drawn again. A `<parent Type="2">` (a hardpoint) keeps its own transform. The light-object
    binds to its effect object as it's made (0x14f370 -> FUN_0021c1c0 -> FUN_0021d040 when its
    parent is an effect object, object-type 17) and the light effect sets its colour each update
    (FUN_0021d270: the appearance's current value into the light's +0xf0..+0xf8 and +0x8c, lit
    while it's above 0.1 and the colour's square over 0.000361): that value is the light
    effect's **newest particle's** Size curve (the range) and Color curve (the colour) at its
    age / life (the appearance's vtable +0xd0, 0x2a1220: #272), so one lamp a light effect,
    which the demo draws now (**Effect lights**, below). **A placed effect object seems to make
    no light of its own** (from the code, not run): FUN_00219d00, which makes the light-objects
    an effect type lists (its h_f8c6a8b4), has two callers, the effect object's make-from-type
    (0x21a2dd, second argument 0: what starting an effect type does) and the first update an
    object added to the world gets (0x21c930, second argument 1: the level's placed objects,
    inferred), and for a light it makes nothing while the second argument isn't 0 (0x219d85). A
    placed effect object's light would be the level-file light-object linked to it: 373 of the
    459 placed effect objects whose type has a light-effect have one (the other 86, e16's 7 and
    m02_a's 7 among them, have none), where the demo draws all 459. A spawned effect (a muzzle,
    a hit, a blast) makes the lights its type lists: 1857 of the 1910 effect types with a
    light-effect list one (nearly all h_09020c30); the other 53, the Molotov's cloud h_1a256542
    and the pound h_ece33110 among them (their light-effect is light_hound_fire), would get
    none in the game and get one in the demo. Not changed here (found, not done; no take
    shows one of them).
  * **Lights that go with their object** (#308). A light-object's tick (0x14f1d0, vtable +0x6c;
    every light is in the world's update list, 0x14f2a0 -> 0x1554d0) asks its link
    (FUN_0015ed30) whether its object is still there and returns 0, to be taken out of the
    world (FUN_000d9840 -> FUN_000db3e0), when that returns 1, which it does for a link whose
    object pointer is clear, whatever the link's type (and for a type 2 link whose hardpoint isn't
    in the parent's model, FUN_00236e00). The object's destructor clears the pointer
    (FUN_001553f0 -> FUN_0008f720), so a light goes a tick after its object: the 373 on effect
    objects after the effect object they hang on (which goes with its own parent, a barrel's
    fire; `ale_fx::TakenAway`), and the **lamps hung on a game object** (`LevelLamp::parent`:
    type 1 or 2 to a placed object that isn't an effect; 15 on breakable objects over 7
    levels: the torches' glows on m01_a / m01_b / m05_b / m05_y's holders (h_ff23484f 10 hp,
    h_1149a76f 25 hp), e05's and m08_a's one each (1 hp), m14_d's three capsules, 16 more on
    the 40 hp sconces that only die and stay) the moment their object has been taken out of
    the world (`Arena::is_gone`). play_lamps.rs puts those out in the light buffer the level
    materials read (`lamp_off`: its colour zeroed, then the materials are marked changed
    a frame on so their bind groups take the new buffer: `touch_materials`; without that the
    scenery stayed lit) and for the characters (`lamp_light_at`'s `off`). **A link with no
    object at all** takes its light away at its first tick in the same way, and the demo never
    makes such a light now (#327: `bf/level.rs` `Link` and `link_of`; it's listed among
    `other_lights` as `LINK_GONE`): a type 1 or 2 link whose parent name is empty (`parent=""`)
    or names nothing in the level (FUN_000d6ac0 finds no object, link +4 stays 0), or a type 2
    link whose `<HARDPOINT name>` isn't a node of the parent's model (FUN_00236e00: the
    archetype's `HARDPOINT`s and its parts', `weapon::hardpoint_names`) or that names none.
    `BF_LINK_CHECK=1 bf_level` runs every level and lists them: **7 lights over 2 levels**
    (m14_d's six, type 1 links, `enable-static` false, all at one point (-143.3, 104.1, 29.9),
    and e16's h_e8169f89, a type 2 link with an empty name), and **no hardpoint missing from a
    parent's model** on any light or effect (all the type 2 links of the 328 effects find their
    node, and no light's link falls to the hardpoint test); none whose parent the demo can't
    read a model of. m14_d goes from 36 point lights to 30 and e16 loses one. **Before and
    after frames:** m14_d's six hang in open space at (-143.3, 104.1, 29.9), with no surface in
    their range at the places the demo reaches (frames there identical); m01_a's default start
    is identical too. **e16's lamp does light something:** at `BF_MAP=e16
    BF_TEST_GOTO=-198,213,-198,213` the green interior is darker without it (frame 11's mean
    33.4 on the build before, 26.2 on this one, most of the picture differing; first seen by
    batch A's tester, then measured). An earlier version of this note said nothing the demo
    reaches was lit by it: that was wrong: the check stood at z 222, under the roof, where it isn't lit.
    Checked on m01_a's sconce (scenery.md "Verified": the lamp's glow on the railing goes with
    the holder), `BF_LINK_LOG=1` logging each lamp as it goes out. A **destroyer-trigger** takes
    lamps out of the world at once too (FUN_00200b70 -> FUN_000db3e0): the mission levels name
    1019 over 25 levels, each fixture's DESTROYED to a router to a destroyer to its lamp (m04_c:
    140; triggers.md "What a destroyer takes"). The demo ran those chains and left every lamp
    lit; now each goes out as its destroyer runs (`LampsOut::put_out`).
  * `BF_LIGHT_LOG=1` lists every light-object (name, type, kind, pairs, colour, strength,
    direction or place), the two pairs and FUN_000dcc10's slots for each (`Level::slots`: the
    outdoor slot 0 is the bump map's light and the lava's v0), the ambients, the ones not drawn
    and why, and each placement lit by the indoor pair.
