*Part of [play/](README.md), the former page `play.md`.*

### Items: the squad's starting inventory (#281)

`src/bin/play_inventory.rs`. Logic from `default.xbe` (the functions are cited in the code and
have rows in [code-map.md](../code-map/README.md)), the counts from the levels' squad placements, checked
against xemu's reads of the game's store at fresh starts (`todo/196-inventory-hud/notes.md`,
"Starting inventories") and the analyst's audit (`todo/inventory-audit/answer.md`, whose
Appendix A lists every campaign zone).

* **Each member's placement** (`levels-<zone>.xml`'s squad `<character-object>`, `<inventory>`)
  carries its two `<weapon>`s and `<h_10dd6db2 name count>` items: ammo, grenades, medkits, the
  OrgSen, the Hat, "Alien Technology Acquired!". The character's property handler applies them
  (FUN_0014dd50): the weapons into the slots, each item into the character's own store by
  FUN_0014bfe0, which keeps one stack per (category, index) (the item type's group-type and
  h_e5f51266) and holds it to the type's stack-limit.
* **One store for the squad.** Joining the squad (FUN_0005b9a0) gives a member the squad's
  store, its own merged into it first with the same clamp (FUN_0014c060). So the squad's items are
  every member's summed and held to a stack (m05_a's Frag 15 is 10, m09_b's Medkit 28 is 25), and
  every member's item box shows the same counts. The demo builds that store as the map starts
  (`StartStore`, `BF_INVENTORY_LOG=1` prints it) and hands it out: the IG_AMMO stacks are the
  reserve (above), the grenades and explosives `Player::grenades` (by the kit of the same label:
  m14_c's and m14_d's Sentry is h_f73de83d, which the kits load when a level defines no
  h_e5f1f063), the Medkit (5, 0) `Player::medkits`, the rest `Player::others`. They go with
  control, as the grenades always have.
* **What it gives** (`BF_INVENTORY_LOG=1` on all 39 campaign zones, against Appendix A: the
  same everywhere but where the appendix's sum passes a stack-limit, held to it: m05_a's Frag,
  m09_b's and m09_c's Medkit; m14_c's and m14_d's Sentry is the h_f73de83d kit):

  | Zone | Items | Reserve |
  |---|---|---|
  | tutorial | Frag 3, Gas 2 | High ROF 500 |
  | m01_a | Frag 5, Medkit 3 | High ROF 400, Shotgun 40 |
  | e34 | Sonic 4, Energy 5, Roller 5, Medkit 6, OrgSen 1 | 11mm 40, Sonic 200, Particle 200, Rail 200 |
  | e40 | Frag 5, Sonic 5, Energy 5, Roller 5, Medkit 20, OrgSen 1 | 11mm 40, High ROF 600, Sonic 200, Shotgun 80, Cutter 120 |
  | m09_a | Frag 10, Sonic 6, Energy 8, Light 4, Roller 6, Medkit 10, OrgSen 1 | 11mm 40, Sonic 200 (480 carried), Particle 200, Rail 200, Bio 200 (380) |

  e34's and e40's are what xemu read in the game's store (e40's ammo after its opening: 11mm 39,
  Sonic 199). A map without squad placements keeps the demo's own stock: the test map a full
  stack of every grenade type, the deathmatch maps 3 Frags (the sdm loadout is #144's).
* **The OrgSen and the Hat** are items of the box (their type's h_1852ab6c lists them; "Alien
  Technology Acquired!" is carried but not listed), with their own icons and names, the count
  shown from 2 as for every item (the game's box: #196 take26). The OrgSen's use is #21's
  (G puts it on with the `bring2face` clip and, at its attach_item event, vision mode 1: the
  thermal view, the lock through walls; G again takes it off at once: [pickups.md](../pickups/orgsen.md),
  "OrgSen"; the squad picks one up on sdm_e13); the Hat's is its own: G with it selected does
  nothing yet. A character whose skill
  refuses its use type can't use it (the skill's +0x1c: Brutus's FUN_0013c160 refuses
  IOU_ATTACH_TO_FACE, the OrgSen, and IOU_ATTACH_TO_EQUIP_SLOT, the Hat): his box shows it red
  with the icon grey, and G says "Cannot use" (h_e784869e), as #196 take26 shows.
* **The gun in hand** at a start is slot 1: a placement leaves none in hand, and the character's
  activation (vtable +0xa4) draws the first slot with a gun (FUN_00123fa0). A zone's opening then
  switches some by its script's code 100 (CT_TOGGLE_WEAPON, [cutscenes.md](../cutscenes.md)): after
  e34's opening Tex holds the Confed LZR-50 and Flint the Jax-iP, after e40's Tex the Jax-iC,
  Hawk the Jax-iR and Brutus the PU Cutter, as xemu read them and take27 / take30 show ("99 Regen",
  "36 / 199"). With `BF_TEST_GOTO` or `BF_SCRIPT` the opening doesn't play, so slot 1 stays in
  hand.
* **The first item shown** is the first listable stack in the game's store order (category,
  index: Frag 2/0, Gas 2/1, Sonic 2/4, Energy 2/5, Light 2/32, Sentry 3/1, Roller 3/4, Medkit
  5/0, the 60-point Medkit 5/2, OrgSen 6/0, Hat 6/3; `load_kits` sorts the grenade kits by it):
  the front's selection starts at none and FUN_0014db60 steps it to the first (#196). Frag on the
  tutorial, m01_a, e40 and m09_a, Sonic on e34 (take27's "Sonic 4"), Gas on e16 and e23. Its NEW
  flag is clear and every other stack's set (**The item box and the item list**). Tab steps in
  that order.
* **Not done**: carrying the store from zone to zone (the save game's `item-%d` pool and each
  member's weapons, clips and hit points: FUN_000816f0, FUN_00081d10, FUN_00129080,
  FUN_00089ea0; #231), so a zone the game only reaches from the one before starts with what its
  own placements carry (m03_b, m03_c, m08_b: no item, and the reserve's full stacks, the stopgap
  above); the order of the joins and the mode's
  clear on a zone whose squad is placed in the world (m07_a, m09_a: the demo takes the
  placements', a live read would settle it); the Hat's use (the OrgSen's is #21's, pickups.md); a Hat picked up (nothing takes it: the pickups taken are medkits, fruit, DNA canisters, ammo, the grenade packs and the OrgSen).
* **The switch sounds in a cutscene** (#282): the controlled character's scripted toggle makes
  them, as the demo plays them (e40's Tex at 7.7 s, e34's at 17.98 s). From the code: the squad's
  HUD entries, with their weapon panels, update every frame of play unless a modal menu is open
  (the session's frame update 0x7a140), every member's entry included; the sound's gate is
  FUN_000bb220's, a member whom a player slot controls (0x8cf60), not "the entry's character".
  The cutscene's HUD hide is the HUD's alpha (1 - 2 x the letterbox amount, FUN_000b8e30), which
  no widget's hidden bit follows. The takes can't confirm it: #196's take30 starts 10.2 s into
  e40's opening, after the toggles, and take27 (e34, Tex's toggle at 8.6 s of the take) has no
  match for either sound, but a pair at the level #73's take02 has them is lost under the
  opening's music and voices there too (a masking test, injected and correlated).

Test hooks: `BF_INVENTORY_LOG=1` prints each member's placement items, the pooled store, what the
controlled character is given and each use of an item that isn't a grenade or a Medkit;
`BF_TEST_ITEM=<label>` selects the carried item of that name at the start (`OrgSen`, `Medkit`,
`Medkit60`, `Sonic`); `BF_TEST_ITEM_USE=<s>[,<s>...]` presses G for a frame at each time for an
item that isn't a grenade; `BF_TEST_INV` and `BF_TEST_INV_NEW` set the pool and its NEW flags at
the start (**The item box and the item list**).

Verified (`BF_INVENTORY_LOG=1 BF_AMMO_LOG=1`, 15 fps captures in the scratchpad, against the
takes in `todo/196-inventory-hud/`): the five zones above print the table's counts; e34 played
through its opening (`BF_MAP=e34`, 375 frames): the script's 100s switch Flint at 13.0 s and Tex
at 17.9 s, and the first controllable frame reads "99 Regen" as take27's; e40 through its
opening: Hawk, Tex and Brutus switch at 7.1-8.0 s, Flint's and Tex's reloads take 11mm 40 -> 39
and Sonic 200 -> 199, and the HUD reads "36 / 199" and "Frag 5" as take30 (14 s); with
`BF_TEST_SELECT=2,24.5` Hawk's reads "24 / 199" with the Jax-iR held low at her right and no gun
on her back, as `stills/still-e40-hawk.png`; on e34 (`BF_TEST_SELECT=1,23.5`) Flint's panel shows
the Jax-iP. m09_a (`BF_SCRIPT=off "BF_TRIGGER_FIRE=trigger_letterbox92,12,0.05"`): Tex's A50
"40 / 200" and "Frag 10"; m01_a: "48 / 400" and "Frag 5". The OrgSen box on Tex
(`BF_TEST_ITEM=OrgSen`, e34's range spot) tiled beside take26's at 1.5 s: the same icon and name
in the same place; Brutus's (`BF_CHARACTER=0 BF_TEST_ITEM_USE=1`) is red and grey with "Cannot
use" over him, as take26 at 7 s. Regressions: the tutorial's "48 / 500" and "Frag 3"; e40's Frag
thrown, 5 -> 4; e34's Medkit used at 50 HP, 6 -> 5 and healed; the test map's full stacks and its
weapon switch (`BF_TEST_SWITCH=1.5`, the switch now `start_switch`); sdm_e34's 3 Frags and full
stacks. After the kits' reorder (the review round): e34's box "Sonic 4" at the range spot, tiled
beside take27's at 14 s; e16's and e23's first item Gas; the test map's kits Frag, Gas, Sonic,
Energy, Light, Sentry, Roller; m14_c's Sentry (3, 3) before its Roller; m03_b, m03_c and m08_b
with the full stacks (the stopgap), m02_b with its 11mm 40 and Cutter 75 alone. e40's opening with
`BF_SOUND_LOG=1`: Hawk's and Brutus's toggles silent, Tex's ff820fda and, 0.65 s on, e1e97460,
beside the opening's own `trigger_sound_weapon_select` (e1e97460 at 7.7 s).
