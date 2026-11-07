*Part of [squad/](README.md), the former page `squad.md`.*

* **Memory chip** (`play_dna.rs`, #43; what the demo called "the DNA"). Each squad
  character-object's `<h_0f77963d inventory-drop>` is item h_f50f94f9 (function-type 18,
  h_0a811e94 2000, message h_1dd3a49d -> h_091ce603 "Memory Chip Recovered!", pickup-sound
  h_ee2c16da). Its mesh h_10960346 is a 0.375 m cube, UVs 0..1 on each face, in the self-lit
  shader h_f539fe8c: the circuit-trace texture h_1e02a5ef (two thirds alpha 0) times its
  h_e01baa40 colour 0.36 1 0.67 at alpha 0.7, blended SRCALPHA / INVSRCALPHA without z-writes
  and unculled (the shader's render-state setup, FUN_0008fd20; h_0f5ae13f 1 would be additive,
  the chip has 2), so the far faces show through the near ones. Its scroll (FUN_0008fca0)
  only runs for h_08c2d2ee = 0.12 s, so the texture sits at a fixed offset (0.446, 0.102).
  * **Drop** (measured from the game's memory and frames in `todo/43-memory-chip/`; the spawn
    code wasn't traced): on the frame the squadmate dies (one frame later in the demo when a
    grenade or gas kills them: those systems aren't ordered before the drop) (take02: the chip's first frame is the
    body's first reaction, full size, no build-up), 0.50 m in front of where they stood along
    their facing, its middle over their feet by Tex 1.098 m (standing and kneeling alike) and
    Hawk 0.925 m. Brutus and Flint weren't measured: theirs (1.016, 1.064) are their bind
    pose's lowest vertex under the model's origin, which is Hawk's within 2 mm but not Tex's
    (1.204), so a guess. It stays there (it doesn't follow the falling body), doesn't spin, bob
    or fall.
  * **Not on a multiplayer map** (#315, `play_dna.rs` `drops_chips`): the drop is the placement's
    `<h_0f77963d inventory-drop="h_f50f94f9">` (FUN_00156390 at 0x156666), and no multiplayer
    level has it: decoded with `xmb_tool.py`, sdm_e13, sdm_e34, sdm_e40, sdm_m03 and mp1 have no
    `inventory-drop` attribute at all (their squads are made by the match, not placed as
    character-objects: sdm_e13's levels file has 0 of them, e40's has 16 and the four squad members' carry
    the chip, as do m09_a's four of 15), and the chip item h_f50f94f9 is defined only in the campaign levels' objecttypes
    (e40, m09_a; none of sdm_e13, sdm_e34, sdm_m03, mp1). `#18` take09 and take10 (sdm_e13) show no
    chip and nobody taking one. So a death on a map `damage::multiplayer_map` says is multiplayer
    (campaign-bf.xmb's deathmatch and squad deathmatch levels) drops nothing; the missions, the
    tutorial and the test map still do. Unseen in the game: a squad member dying in a squad
    deathmatch (the footage has its AI frozen), so "none" rests on the data. The chip's own
    code path (`BF_TEST_CHIP`) is unchanged on every map. Checked: `BF_MAP=sdm_e13
    BF_TEST_GOTO=7.34,26.75,7.34,26.75 BF_TEST_DIE=1 BF_PICKUP_LOG=1` (120 frames): Tex dies, the
    hand-over goes to Brutus, no "dropped a memory chip" and no chip in the frames;
    `BF_MAP=m09_a BF_TEST_GOTO=-197.3,-216.9,-197.3,-216.9 BF_TEST_DIE=1` still has "tex dropped a
    memory chip" at 1.07 s, and the test map (`--test BF_TEST_GOTO=0,8,0,8 BF_TEST_DIE=1`) too, taken by
    Brutus.
  * **Pickup:** any living squad member (the player or an AI one) whose feet come within
    sqrt(3) m (3D) of its middle takes it (take03 and staging: 1.34 m across with the chip
    1.098 m up, every time from three directions; whether the game tests 3D or across only
    isn't settled, and the test wasn't found in the code). It goes on that frame, h_ee2c16da
    plays and the pickup lines show "Memory Chip Recovered!" (no count). The footage's green
    "+ 2000" by the radar on the same frame is the score, which the demo doesn't keep; nor does
    it reclone the squadmate.
  * The green
  sprites and blue light the demo drew before were effect h_ee11d51f (powerup_pill +
  light_powerup_pill), which belongs to the DNA canister pickup (mesh h_e3e4caad, "Alien
  Technology Acquired!"), not to a death. (`BF_PICKUP_LOG=1` prints the chip's material, where
  each is dropped and when it's taken.)
  * Its pixel program (pixel shader 0, defined at 0x3ddf80 in default.xbe, set from
    FUN_0008fd20 through FUN_000a4d00) is one combiner stage, texture x constant c0 for colour and
    alpha alike, c0 being the h_e01baa40 colour with `alpha` as its fourth component
    (FUN_0008fa50 maps the names to +0x70 / +0x7c; FUN_000a4d80 packs them), then a final
    combiner that fogs the colour. No vertex colour, lighting or second texture. So it's drawn
    as Bevy's alpha blend of that, fogged, with its texture's own mip levels (five, 64 down to
    4: box averages, mean alpha 39, 49, 52, 43, 40 of 255; since #257 the format reader decodes
    them, where this module used to rebuild the chain). Without mips the one-texel traces break
    up a few metres off.
  * Not used: `time-scale` (h_01590d7a, 0.4) is mapped to +0x58, but none of the shader's own
    functions read it (and it couldn't move where the scroll stops). The wrapper's h_e59d69a0
    (60) goes to render state +0x294 of the state cache (FUN_0009e960, flagging the material
    when it isn't 255): STENCILREF (#4), the stencil value the chip's pixels write, which the
    game's glare reads as a luminance threshold (#289, levels.md "The glare"; see below).
  * **Not matched: the footage's fill.** In `todo/43-memory-chip/` it's a filled translucent
    green box with a darker circuit pattern, not a lattice: over dark sand (take02, background
    about 10, 8, 6) its middle is 19, 79, 28; over lit grey rock (take05 at 16 s, background
    about 75, 70, 58) 177, 253, 172 with a soft pale halo past its edges. A Light grenade
    burning 2.5 m away changes it by under 10% (take08 / take09). The second is above the
    background in every channel, red by 100, which no SRCALPHA / INVSRCALPHA blend of this
    material's colour (red at most 16) can do, and the fill between the traces needs alpha the
    texture doesn't have there (alpha 0 in two thirds of it at the top level). So what the code
    reads (FUN_0008fd20's blend states, pixel shader 0) and what xemu draws disagree; the demo
    draws what the code reads. **Explained by the glare (#289):** the game keeps every pixel
    whose stencil is 60 (the chip's) and whose luminance is above 60 / 255, blurs it and adds
    2.35 times it back (levels.md "The glare"). Over the dark sand the chip's 19, 79, 28 has a
    luminance of 55 / 255, under the key: nothing added, as take02 shows; over the lit rock its
    blend is above it, and the glare raises every channel and spreads the soft halo. The demo
    doesn't key the chip yet: the glare also multiplies by the frame buffer's alpha, which the
    chip's blend leaves at a value not worked out (a follow-up).
  * Earlier footage (campaign e34, not a squad deathmatch): far off (Friendly Fire 2, 33.8 s) the chip's inside is
    green +58 over the background, red and blue up too; the demo's is green +30 to +48 at 6-15 m
    on sdm_e34, red and blue a little down. Up close in front of a lit wall (DNA + Weapon
    Pickups, 12.4-12.6 s) the footage's is whitish cyan, red 165 over 60, with a soft halo.
    The pixel program can't raise red above the background (texture red 45 x 0.36) and the
    texture's mips don't fill it, so that light comes from something outside the chip's
    material: not found. In the DNA footage the Light grenades' beams are close by, and at
    9.4 s a soldier walking through the chip is lit the same whitish cyan. A guess, not drawn.
  * It sorts 1000 m nearer among see-through things than it is: the terrain's
    blended texture layers are drawn in the same pass, sorted by their chunk's middle, and
    painted over it on sdm_e34 (it all but vanished: a 7/255 difference). Known limit: it now
    sorts after every other blended thing (gas clouds, blood mist, liquids, see-through
    materials), so it shows crisp over a cloud between it and the camera. (The same value is
    also the pipeline's depth bias, about 1e-4 of the depth: it doesn't draw through walls.)
  * `BF_TEST_CHIP=<x>,<z>[,<s>]` drops a chip at (x, z) at that time (default 0.5 s), on the
    floor below 2 m over the player's middle, without a death.
  * Verified: Brutus killing Tex on the test map (`cargo run --bin bf_play -- --test` with
    `BF_CHARACTER=0 BF_TEST_GOTO=0,0,0,0 BF_TEST_KILL=3 BF_VIEW_YAW=0.7`): the chip appears on
    the frame Tex is first knocked back (none the frame before, as take02 #162 / #163), 0.50 m
    ahead of where he stood and 1.098 m over his feet, stays as he falls, and Hawk's AI walks
    into it and takes it 1.72 m away (3D), "Memory Chip Recovered!" on that frame; on sdm_e34
    standing
    (`BF_TEST_GOTO=-44.4,15.5,-44.4,15.5 BF_TEST_CHIP=-43.2,13.0,0.5`, with `BF_VIEW_YAW=0` and
    `0.6`: one face square-on, then two faces and an edge, as the footage's camera pass at
    2.5-6.5 s) and running through one (`BF_TEST_GOTO=-44.4,15.5,-44.4,35
    BF_TEST_CHIP=-43.8,22,0.3`: it goes, h_ee2c16da plays, "Memory Chip Recovered!" shows).
