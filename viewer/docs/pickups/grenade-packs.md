*Part of [pickups/](README.md), the former page `pickups.md`.*

## Grenade packs, the "Bio Ammo" of 3 (#19)

`play_item_pickups.rs` again (the same walk-over, "Ammo boxes" above), from the same leads and
`todo/18-ammo-pickups` takes 04-07 (sdm_e34, xemu).

* **What they are.** Four inventory types the data names "Bio Ammo" (stringtable-name
  h_fca63a02: the ammo box's string, copied), `function-type` 0 (the base set, not the ammo
  box's 14), `group-type` 1, `pickup-amount` 3, `stack-limit` 999, each with a
  `pickup-archetype` that is a grenade: the pack gives *that*, so what goes into the store is
  the archetype's, 3 of it, and the pack's own 999 and key (1, 20-24) are never used:

  | Pack | grenade it gives (its key) | the line | on the multiplayer maps |
  |---|---|---|---|
  | h_fd437c4d | Frag h_060f95f7 (2, 0) | "Frag Grenades" | 5 |
  | h_11291ea2 | Gas h_e8c67904 (2, 1) | "Gas Grenades" | 33 |
  | h_eb44e4f4 | Sonic h_ec0bf28f (2, 4) | "Sonic Grenades" | 40 |
  | h_1913256a | Energy h_0cc3c2c8 (2, 5) | "Energy Grenades" | 32 |

  (And the Light's, h_fa558016 -> h_fd1a966d, "Flares", on m09_b. The campaign places the packs
  too: the Frag's in 30 levels, the tutorial's three at (-62.5, 15.6, 20.8), (-63.3, 15.6, 20.0)
  and (51.1, 16.1, 20.0).) The grenades are the demo's grenade kits (`GrenadeKits`, the squad's
  (2, index) stacks `Player::grenades`), each held to its `stack-limit`, **10**.
* **Taking one** is the walk-over as the boxes' (any living member in reach, the AI's too, the
  same reach): 3 of the grenade go into its stack up to 10 (FUN_0014ce90 with the pack's
  `pickup-amount` as the multiplier, the archetype's class-type and stack-limit; FUN_0014bfe0),
  the pack is removed, its sound plays, PICKUP_OBJECT goes out and the line is the grenade's
  display name (h_1dd3a49d of the type FUN_00146340 gives, not the pack's "Bio Ammo"):
  * take04: Tex with no grenades (the Gas stack at 0, no type) walks through three Gas packs
    lying within 1 m of each other: "3x Gas Grenades", "6x", "9x" on the one line, the box (it
    was empty) selecting Gas and showing "NEW 3", then 6 and "NEW 9" ([play.md](../play/item-box-and-item-list.md), "The
    item box": the label is the stack's first showing), the packs gone. The pack that returned
    10 s later, with Tex standing there and the stack at 9, was taken at once: 10.
  * take05: the stack at 10: red "Gas Grenades (full)", the three packs stay and the line stays
    while he stands there.
  * take06: the stack at 8: the first pack gives "2x Gas Grenades" (10) and is gone; the other
    two stay. The line is blue for 4 s, and **then the red "Gas Grenades (full)" appears under
    it** (8.65 s: the walk-over keeps finding the full stack, and a count-0 line can only start
    once the "2x" line has run out: FUN_000ad790's rule, `PickupFeed::add`).
  * take07: two Energy packs 0.7 m apart, the Energy stack empty and typeless: "3x", then "6x
    Energy Grenades" (the (2, 5) stack). Sonic packs weren't filmed (all three clusters sit
    against rock): the same code.
* **Not done.** The multiplayer maps' 75 Roller and Sentry spawns, `<h_1d1d4f05>` placements of
  object-type 24 types (h_15f9c51a "Roller": the carried Roller (3, 4); h_ecfe0d8c "Sentry": the
  carried Sentry (3, 3); stack-limit 10 each, `cost` 350 and 500), all spawn-trigger templates:
  the game takes them by the same walk-over (+1 each), but `spawn_copy` copies inventory-objects
  and weapon-objects only, so none shows (#278, above): a ticket of its own. The pack of a
  grenade type the map's kits lack (the Light's off m09_b) isn't taken.
* **Verified** (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_INV=Frag:0 BF_TEST_GOTO=-77.52,15.19,
  -78.3,11.4 BF_TEST_WALK=1 BF_TEST_GO_AT=2.0 BF_PICKUP_LOG=1 BF_ITEM_LOG=1`, 330 frames at 15
  fps; `BF_TEST_INV=Frag:0` takes the demo's start Frags away, as the takes' squad had none):
  taken at 3.47, 3.60 and 4.07 s, "3x" -> "6x" -> "9x Gas Grenades" on one line, the box
  selecting Gas at 3.47 s with NEW (the log: "none -> grenade 1", "NEW on grenade 1"), the same
  as take04's 3.7-5.5 s frames; the first pack back at 13.47 s and taken: 10, the other two full.
  With `BF_TEST_INV=Frag:0,Gas:10`: "Gas Grenades (full)" red, the packs staying (take05); with
  `Gas:8`: "2x Gas Grenades" at 3.47 s, the red "(full)" from 8.0 s, the box 8 -> 10, the two
  packs staying (take06's 4.15 and 8.65 s); take07's start (`BF_TEST_GOTO=-66.8,37.2,-71.5,38.2`):
  "3x", "6x" Energy Grenades.
