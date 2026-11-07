*Part of [pickups/](README.md), the former page `pickups.md`.*

## Ammo boxes (#18)

`play_item_pickups.rs`, from default.xbe's walk-over (FUN_0011c540 -> FUN_0011c170's item path;
the reading is `todo/18-ammo-pickups/leads.md`'s, its addresses are in
[code-map.md](../code-map/README.md)) and checked against that folder's takes 01-03 and 08 (sdm_e34, xemu,
Tex's half of the split screen).

* **What they are.** Ten ammo items (objecttypes `<inventory>`, function-type 14 IFSET_AMMO_BOX,
  group-type 1 IG_AMMO, `class-type` 2): each one's `h_e5f51266` is the ammo-type that its guns'
  `ammo-type` names, so the box feeds every gun of that type (the reserve the weapon panel and
  the reload already use, play_ammo.rs `Reserve`):

  | Ammo (item type) | ammo-type | guns | `pickup-amount` | `stack-limit` |
  |---|---|---|---|---|
  | 11mm Ammo (h_12302d03) | 1 | Foley 356 Tact, L-Shot-50, L-Shot-75 | 10 | 50 |
  | High ROF Ammo (h_02735ad0) | 2 | Kman Auto, MK-ASLT, RVG50 Minigun | 50 | 600 |
  | Sonic Ammo (h_ef0c4c1e) | 3 | Jax-iP, Jax-iR, Jax-iC | 20 | 200 |
  | Particle Ammo (h_fc0d262e) | 4 | Saryl-23, Saryl-45, Saryl-99 | 25 | 200 |
  | Rail Ammo (h_e4286255) | 5 | Rail RVR, Rail CLVR | 10 | 400 |
  | Shotgun Ammo (h_e5d6b39e) | 6 | Bower 20 | 10 | 80 |
  | Cutter Ammo (h_fe018d0b) | 7 | Feral Cutter, Rusty Cutter, PU Cutter | 25 | 120 |
  | Bio Ammo (h_01c1965b) | 8 | MNR-7 Bio, A10 / A50 Bioreactive | 40 | 200 |
  | Rocket Ammo (h_11119186) | 10 | Sweeper V, Therm Sweeper | 8 | 40 |
  | Energy Ammo (h_15c866a4) | 12 | Ion Beamer | 20 | 200 |

  (Cutter and Rocket Ammo are the campaign's: the multiplayer maps place the other eight.) The
  ticket's column `h_0a811e94` ("a respawn time?") is the ammo-type written twice: the AMMO_BOX
  set's +0x44 returns it as such (0x147180). Respawns are the spawn-triggers' timers.
* **Taking one** is the walk-over (above): any living member of the squad in reach, the AI's too.
  The set's "can take" is the unconditional 0x280040, so nothing refuses a box because no squad
  gun uses the ammo or because the stack is full: `pickup-amount` goes into the squad's reserve of
  that ammo-type (FUN_0014ce90 with its multiplier flag, FUN_0014bfe0) up to the item's
  `stack-limit`, and
  * **something fits**: the box is removed for good, its pickup sound (h_e4097f62) plays at the
    character, PICKUP_OBJECT goes out from it, and the line "<n>x <name>" shows (n what went in,
    "Particle Ammo" alone for one): take01's box of 25 on a stack of 100 made the line "25x
    Particle Ammo" and the panel's reserve 38 / 100 -> 38 / 125 in the same moment (video 4.457
    s, Tex 1.26 m from the box's middle; take02: 1.21 m). A partial fit loses the rest: take02's
    stack of 195 took 5, "5x Particle Ammo", the box gone. Two boxes in a frame are one line,
    "20x Rail Ammo" (take03: two Rail boxes 0.5 m apart, taken in the same 50 ms, the two Rail
    guns 1.5-1.7 m away offering "Hold [X] to swap Rail RVR."). A stack nobody holds the gun of
    takes the type: take03's Rail stack was unassigned (count 0), the squad had no Rail gun, and
    "20x Rail Ammo" went in with no message;
  * **nothing fits** (the stack at its limit): the line "<name> (full)" in red, the box stays,
    and the walk-over finds it again every frame, so the line stays while the character stands
    there (take02, 19.8-25 s: "Particle Ammo (full)" under Tex, the stack 200 throughout). It
    goes to the HUD of the player who controls the character alone (FUN_0011c6a0 with AL clear):
    **an AI squadmate** standing on a full box shows nothing (take08, 14-22 s; the demo:
    `BF_TEST_TARGET=1`'s squadmate on it), and one who takes a box with room puts the line on the
    player's HUD, the reserve following (take08 3.26 s: Hawk on the box, "25x Particle Ammo" on
    Tex's half, 100 -> 125).
* **The respawn** of a multiplayer copy is its trigger's timer ([triggers.md](../triggers/subject-relays.md),
  "Subject relays"): the Particle box 10 s after it was taken (take01 9.78 s of game time, take02
  9.8 s), the Rail boxes 20 s (take03 19.7 s), the demo's 10.00 and 20.00; a character in reach
  when it returns takes it at once (take04 for the Gas packs: 9 -> 10 by a returning pack; the
  demo, take02's stack: "(full)" from the frame it returns). The copy comes whole in a frame and
  the spawn flash's green rings rise over it for ~0.6 s (take01 15.5-16.1 s, take03 27.9-28.2
  s): the demo's flash is the same effect object (the medkits' too). A campaign level's box is
  a plain placement and never comes back (above).
* **Not done, or not the game's yet.** What `h_11884f2e` (20-50 on the ammo items) does: the
  AMMO_BOX set's +4 slot takes it with 3 (the stack loses it less 1), a caller nobody traced;
  not the walk-over, not the reload. The weapon panel's reserve text flashing 0.3 s at a pickup
  (`[[entry+0xf0]+0x98]+0x94 = 0.3`, FUN_0011c6a0): the widget isn't traced and take01's frames
  (4.5-4.9 s) show the "38 / 125" unchanged in colour and size; nothing is drawn. The AI's
  interest in boxes (the set's +0x54 / +0x58, 75 m, GOAL_GRAB_RESOURCE?) isn't traced: the squad
  takes boxes only by walking over them. The pickup effect: none (every ammo item's
  `pickup-effect` is empty). A dead enemy's drops (`inventory-drop`: the tutorial's terrorists
  carry "High ROF Ammo 48") are not made.
* **Test hooks:** `BF_PICKUP_LOG=1` prints each box taken (who, how many went in, the stack) and
  each one a full stack refuses (once). `BF_TEST_AMMO=<ammo-type or ammo name>:<n>[,...]` sets
  those reserves at the start, held to their stack limits (`4:195`, `rail:0`, `particle:200`).
  The test map's pickup row and its object menu (O) carry every ammo type.
* **Verified** (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_WEAPON="Saryl-99;Confed LZR-23"
  BF_TEST_GOTO=-51.70,82.10,-56.5,81.0 BF_TEST_WALK=1 BF_TEST_GO_AT=2.0 BF_PICKUP_LOG=1`, 330-420
  frames at 15 fps, take01's start and box): with `BF_TEST_AMMO=4:100`: "25x Particle Ammo" at
  4.00 s, the panel 38 / 100 -> 38 / 125, the box back at 14.07 s and taken again (the stack 150);
  with `BF_TEST_AMMO=4:195`: "5x Particle Ammo" (200) at 4.00 s, and at 14.07 s the box returns
  with the green rings and "Particle Ammo (full)" in red stays up under Tex while he stands
  there (frames 222-267), as take02; take03's start
  (`BF_TEST_AMMO=rail:0 BF_TEST_GOTO=19.53,70.07,13.0,69.5`, 480 frames): "10x Rail Ammo" at
  4.13 s and 4.20 s, "20x Rail Ammo" on the one line, the stack 20, the two boxes back 20.00 s
  later (24.13 s and 24.20 s); take08 (`BF_TEST_TARGET=1 BF_TEST_GOTO=-55.98,89.09,-55.98,96.0`,
  the squadmate standing on the box): Brutus takes it at once, "25x Particle Ammo" on Tex's HUD
  and 38 / 125, and with `BF_TEST_AMMO=4:200` nothing shows under the standing squadmate for the
  whole run. The tutorial's two medkits (`BF_MAP=tutorial BF_TEST_GOTO=51,14,51.0,18.0`): taken
  at 1.51 and 1.55 s, and not again in the minute after; its DNA canister at 1.74 s (1.80 s with
  the old 1.3 m).
