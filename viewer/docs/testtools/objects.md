*Part of [testtools/](README.md), the former page `testtools.md`.*

* **Objects**, put on the ground at the crosshair turned the way the copy shows:
  * a pickup is a level's inventory-object as the grid's (`play_pickups.rs`: `LatePickup`
    seats it on its lowest point, upright as modelled): walked over, it's taken as on a map
    (the medkits and fruit, the DNA canister, the ammo boxes and grenade packs, the OrgSen, the
    power-ups) and gone for good; everything is loose. Home puts the taken ones back where they
    were (**Putting pickups back**, below). Two entries of one name (the placed and the carried
    Medkit, two Bio Ammo) carry their hash;
  * a weapon lies on its side, a lying gun with a full clip as the rack's (#17): the prompt
    offers it, X held swaps it in (the gun in hand dropped at the feet), and it's gone; walked
    over, a squad member using its ammo type takes its rounds, as from any lying gun;
  * a grenade lies as modelled; walking over it adds one of its type up to its stack-limit
    ("Took Sentry"), else "<type>: full" (the test map starts with full stacks). It's in reach as
    any pickup is, by the game's walk-over (FUN_0011c170, `powerups::in_reach`: 1.34 m across,
    1.8 m below to 1.0 m above the character's position). (It was 1 m across and 1.5 m up or
    down, copied from the medkits' old reach, until #18's review had the pickups' reach unified.)
  * **breakable scenery**: the radiation barrel (h_e04e5a0e), the missile rack (h_fbdcd828)
    and the supply crate (h_09a6856d), sdm_e34's kinds (see "Interactive scenery"), standing
    as modelled. One put down joins `play_scenery.rs`'s list as a map's does (its
    `LateBreakable`: hitpoints, centre, debris made ready) and breaks, explodes and chains as
    there: shots, grenade blasts and other objects' damage areas, by the type's factors (the
    rack's Type 10 blast sets off barrels and crates within its 8 m; a barrel's Type 3 cloud
    sets off nothing). The flat floor has no collision, so its model's box stands in for it:
    `ray_hit` stops at the box (shots end on it and set it off, the crosshair meets it);
    characters walk through it, and through what it leaves behind (the rack's stand).
  * **effects for a body** (#397): a group, "[effects (unspawned types)]", of two effect types of
    common's objecttypes that nothing in the game spawns (the cut-content catalogue, 3.1), put on a
    body by the demo: "Vengar aura (unused effect vengar)" (h_fb1e5895), which **is unused in the
    game**, and "pshield, the shields' hit effect (unspawned duplicate type)" (h_13687abf), which
    is **not**: that type duplicates h_1218cdc1, the `hit-effect` of every shield, whose effect
    `pshield` the game plays as a 0.3 s burst when a shielded character is hit, so a continuous
    swirl on the hips is the demo's own construct. They have no model, so the see-through copy is
    empty and nothing is marked at the crosshair. A click or Enter puts the picked one **on the
    controlled character**, wherever the crosshair is (in the free camera too): its ALE effects
    ride the character's hips and turn with it, and go with it when it dies or control passes to
    another (the effect stays on the character it was put on); the same entry again takes it off
    ("Vengar aura off Tex"; two requests for one type and character in one frame count once). Not
    placements: they aren't listed in the editor's `Placements`. See [effects.md](../effects.md),
    "Effects on a body". `BF_TEST_PLACER=<s>,vengar` picks one and `BF_TEST_PLACE=<s>` clicks
    (`BF_TEST_PLACE=<s>,vengar,<x>,<z>` puts it on too: the place is ignored);
    `BF_TEST_EFFECT=<s>,vengar,me` does the same without the menu.
  * **ALE effects nothing starts** (#397, the ticket's optional part): the last group, "[ALE effects
    (unused in the game)]", the 139 of the effect library's 603 effects (effects-common.ale) that
    the cut-content catalogue found unreferenced (3.2, `effects-unreferenced.txt`: no object type,
    trigger, world material, script or liquid names them, nor the XBE's strings or constants),
    by their names in order: bio_bullet, breakup_*, duststorm, exp-mirv-*, exp-thermolite-*,
    paralyzer_*, powerup, the splash_* family, stun_grenade, tornado, wmat_* ... (`UNUSED_ALE` in
    `play_testfx.rs`, each name checked against its hash). No model: the copy is empty and a ring
    on the ground marks where it will play (in the free camera a yellow ring hangs on the ray).
    A click or Enter plays the picked one **once** there, turned as the wheel has turned the copy
    (an `EffectSpawn` for a hash that isn't an effect type, which `start_effects` plays as an ALE
    effect with the rules the effects triggers make are played with, `load_recorded_facing`, for
    `Compiled::duration` seconds, an estimate). **The ones that never end are cut to 3 s**: an
    effect whose estimate is over 30 s (`ALE_ENDLESS`) plays for 3 s (`ALE_ENDLESS_RUNS`, the 3 s
    that counts for an emitter outlasting 100 s), then its emitters stop and the particles already
    made live out their lives. Four of the 139: `singeflame` (105 s: a flame that stays), `tornado_master`
    and `tornado_master_40` (1e9 s: a particle life with no end) and `breakup_console_master` (63 s,
    which shows nothing); the longest left alone are `stunned_fx` (30 s) and `godray40_bounds`
    (28 s). There is **no way to clear** an ALE effect once started (the object menu removes
    nothing it places: [level-switch.md](level-switch.md), "Not done", #187), which is why they're
    cut; the log says "(an estimate of 105 s: taken as one that never ends)" and "plays ... for
    3.00 s" (checked with `BF_TEST_EFFECT`: the flame's column is gone about 1.5 s after the cut,
    the tornados' dust fades out about 5 s after it). They aren't placements and don't stay.
    Nothing in the game plays them, so how each was to look is the demo's ALE runtime's reading:
    some are fragments (a `-master` that spawns its `-add`, a `-trail`). `BF_TEST_PLACER=<s>,tornado` picks one,
    `BF_TEST_PLACE=<s>,tornado,<x>,<z>` plays it there, and `BF_TEST_EFFECT=<s>,tornado,<x>,<z>`
    plays it without the menu.
* **In the air** (the free camera): a pickup gets play_pickups.rs's `DropIn` (it falls from
  where it's put, turned as the copy showed, and tumbles to rest as the grid's do), a weapon or
  a grenade `Thrown` from rest (the loose-body physics a dead character's gun uses; it settles
  on whatever face it lands on, not on its side). Breakable scenery hangs where it's put: a
  level's scenery never moves, and its box (shots, the crosshair) and its blast are where it
  hangs; its debris falls from there. The follow camera keeps the ground placement.
