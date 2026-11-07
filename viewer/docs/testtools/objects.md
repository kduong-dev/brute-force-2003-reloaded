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
* **In the air** (the free camera): a pickup gets play_pickups.rs's `DropIn` (it falls from
  where it's put, turned as the copy showed, and tumbles to rest as the grid's do), a weapon or
  a grenade `Thrown` from rest (the loose-body physics a dead character's gun uses; it settles
  on whatever face it lands on, not on its side). Breakable scenery hangs where it's put: a
  level's scenery never moves, and its box (shots, the crosshair) and its blast are where it
  hangs; its debris falls from there. The follow camera keeps the ground placement.
