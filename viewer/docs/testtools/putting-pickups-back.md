*Part of [testtools/](README.md), the former page `testtools.md`.*

* **Putting pickups back** (`play_testpickups.rs`, the **Home** key; the product owner's request
  on #18's review, with #19 and #21; the demo's own, no footage): a taken pickup is gone for good
  as on a level ([pickups.md](../pickups/health-pickups.md), "A taken pickup is gone for good"), so the grid's
  pickups and those put down with the object tool could be taken once each. Home brings them
  back, on the test map only: the game's levels keep the game's rule.
  * **Which**: the `Shelf` lists every pickup the test map put out, in the order put out: the grid's
    (44 with the default levels, `spawn_test_map`) and each one the object tool puts down (`place`),
    a pickup being an inventory-object. A new map starts with an empty shelf. A level the session
    switched to (L) has only the object tool's: its own pickups are the level's and stay taken. A
    lying gun or a grenade put down with the tool isn't on it (the rack restocks its own guns as
    they're taken; place the others again), nor breakable scenery.
  * **What it does**: each piece whose pickup is gone from the world is made again by the same code
    that made the grid's (`testpickups::spawn_pickup`), at the pose it was put out in: the grid's
    drop pose (0.3 m up, tipped and turned: it falls and settles as it did: the two medkits compared
    frame by frame came to rest in the same place and pose as the first time), the object tool's
    spot turned as the copy showed (on the ground, or in the air to drop again). It joins the map's
    pickups as one placed after the map started (`LatePickup`: `late_pickups` seats it, gives it its
    loose body and lists the medkits, fruit and DNA canisters as takeable; `take_items` (the ammo
    boxes and grenade packs) and `take_powerups` find it as they find any), so it can be taken
    again, and Home again after. A pickup still there isn't touched (not moved, not doubled). "N
    pickups put back" shows (the edit mode's status line too; "No pickup to put back" when none was
    taken). A character in reach when one comes back takes it at once, as the game's box does when
    its timer returns it.
  * One the object tool put down keeps its number in the editor's `Placements`: `track` showed
    it `Gone` when it was taken and shows it `Here` again (`Placements::remake`, editor.md).
  * It works in play, in the free camera and in the editor's edit mode (the squad out of the
    world: the pickups drop and lie as they will; F5 to play from there takes them again).
  * `BF_TEST_RESET_PICKUPS=<s>[;<s>...]` presses it at those times (hooks, below);
    `BF_TOOLS_LOG=1` prints "t 3.40: 2 of 46 pickups put back: Medkit at [3.00, -0.98, -3.00],
    Medkit at [4.50, 1.08, -3.00]" ("0 of 46" for none taken) and the editor's placements
    changing.
