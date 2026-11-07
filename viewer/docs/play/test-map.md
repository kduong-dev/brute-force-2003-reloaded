*Part of [play/](README.md), the former page `play.md`.*

## Test map (`cargo run --bin bf_play -- --test`)

A flat test floor for trying things out, only reachable with the `--test` flag. It opens
straight into play: no loading screen, intro or menu (`src/bin/play_testmap.rs`).

* **Tex alone** (#302, the product owner's choice): the squad starts with the player's character
  (Tex, or `BF_CHARACTER`'s: a squad name, `brutus`, `flint`, `hawk` or `tex`, any case, or
  its number 0-3; #330) and its other three slots empty. The squad menu (T) puts any
  character type in a slot, a real squad member (the formation, 1-4, the portraits and health
  channels, the hand-over at a death), empties one, or changes the player's own type where they
  stand; or puts a whole predefined squad in all four (the campaign squad, or one of the five
  squad deathmatch offers, #339: [testtools.md](../testtools/the-squad.md), "The squad"). `BF_TEST_FULL_SQUAD=1` starts with the four
  as before, `BF_TEST_SQUAD=<type>,<type>...` with those; a hook that acts on the squad's members
  (`BF_TEST_SELECT`, `BF_TEST_KILL`, `BF_TEST_TARGET` ...) starts it with the four too, so the
  recipes written for them run as before. Any other map (the L menu's levels, `--test
  BF_MAP=<level>`, the main game) keeps its own squad.
* **m04_a's music** (#302): its bank plays from the start, mixed as the music menu (U) mixes any
  level's (test_casp1_amb01, the ambience bed ambience19-2ch under it); U still changes it.

* **Every hand weapon on a rack**, floating and turning in front of the start: the 27 weapon
  definitions with a clip whose model loads (one per model), from the first mission's data,
  the multiplayer archives (`mp_common`, `mp1`-`mp8`) and m02_a, which holds the three
  campaign-only ones (A10 Bioreactive, Confed LZR-50, Jax-iP); about 0.7 s. (m09_a adds the
  Light grenade and sdm_e34 the missile rack for the object tool: about 1 s in all.) Each is a
  lying gun with a full clip, as the game's weapon pickups ([pickups.md](../pickups/weapon-pickups.md), "Weapon
  pickups", #17): in reach (2.5 m across, 1.7 m up or down) the prompt offers the one most in
  front, "Hold [X] to swap <name>." with the blue ring, or "<character> cannot pick up <name>."
  with the red one for a class the character's slots don't take (Tex and Brutus refuse the
  pistols, Hawk and Flint the cannons). X (E) held 0.3 s swaps it in with the character's
  drop-and-pickup clip: the gun in hand drops at the feet (a lying gun too: its rounds go into the
  reserve first, which starts full here, so it keeps them), the new one comes with its full clip
  into the slot the game's rules pick (Tex's in hand; Flint's and Hawk's by class; Brutus's
  cannons to slot 2), its name over its count. The rack puts a new one in its place at once, and
  walking past it takes no rounds from it (`KeepClip`: the test map's convenience).
  `BF_TEST_GOTO=0.8,-4.3,0.8,-6 BF_TEST_GO_AT=999 BF_TEST_SWAP=2.0` stands Tex at the first row's
  fifth gun and swaps it.
  * The gun dropped there never vanishes: the flat floor plays as a mission (it isn't a
    multiplayer map in the game's level list), where a swap's dropped gun gets no vanish timer
    ([pickups.md](../pickups/weapon-pickups.md), "The vanish timers").
  * The swap changes the guns a character holds, not its loadout (`Game::character_weapons`,
    which the rack's old walk-in respawn rewrote): a character made anew has its original guns
    again, as when play resumes from the map editor's edit mode (the squad comes back fresh,
    [editor.md](../editor.md)) or a level switch brings the squad back.
  * Loading every level (about 6 s) adds no others: the rest are the creatures' built-in
    weapons (no model), Feral Cutter (the same model as another) and the bioreactive death's
    vaporise blast h_1f4a5a0a, which carries the iKhan-GPL's name (a model under 0.1 m; the gun
    itself, h_e9be70f0, is on the rack: #55).
  * `BF_TESTMAP_LEVELS=<level>,<level>...` loads a different list.
  * `BF_TESTMAP_LOG=1` lists every weapon on the rack with its type, clip and model size, and
    every hand weapon left off it, with why.
* **One of every pickup type** with a model (one per model) in a grid behind the rack.
  * They're dropped in from 0.3 m, upright as modelled but turned at random and tipped up to
    0.2 rad, so each settles as it would. A crate lands on its base; the Garo fruit (modelled
    on its point) and the cards (on their edges) fall over onto their sides.
  * They're inventory-objects as on a map, so medkits and fruit work as in "Health pickups",
    and everything is loose (knocked by shots, thrown by blasts, tumbling; a walk moves none of
    them, #356). The ammo boxes, grenade packs and the OrgSen are taken as on a map too
    ([pickups.md](../pickups/README.md)), and a taken pickup is gone for good: **Home** puts back every
    one taken (the grid's and the object tool's), where it was (`BF_TEST_RESET_PICKUPS=<s>`;
    [testtools.md](../testtools/putting-pickups-back.md), "Putting pickups back").
* **Every squad grenade type**, a full stack of each (its stack-limit, 10): Frag, Gas,
  Sonic, Energy, Light, Sentry, Roller (the game's store order, #281). m09_a is
  loaded for the Light (only in m09_a/b/c/x). Tab steps through them. See **Grenades**.
* **Every character type of the data** (128) for the NPC spawner (#180): its levels define
  55; the other 73 come from the 31 level archives that define them, without those levels
  (`Game::load_type_assets`, about 0.9 s more; `BF_TEST_TYPES` picks a smaller set). See
  [testtools.md](../testtools/npcs.md) and [character-types.md](../character-types.md).
* **Instant kill**, on at the start (`BF_TEST_INSTANT_KILL=0`: off), K toggles it (`BF_TEST_TOGGLE_KILL=<s>[,<s>...]` presses it at those times). The player's shots and grenades kill any
  squad member they hurt, in one hit. The player still takes normal damage.
* **X kills the controlled character** on the spot, as any hurt does: the death cry and
  ragdoll, then the death camera and the hand-over to the next squad member (or, with nobody
  left, the camera stays on the body; on the test map's floor the player respawns, below).
  `BF_TEST_SUICIDE=<s>` presses it at that time. Its hurt is K's (`enemies::lethal`), so a type
  with a stamina shield in the player's slot (a deathmatch Shrike, #302) dies too; so does
  `BF_TEST_DIE`'s.
* **Dying alone respawns** (#302, the demo's rule for the test map, not the game's): with nobody
  to hand over to, 4 s after the death (the death camera's time to its hand-over) the player
  comes back where it stood, a fresh one of its type, its view and the squad's inventory kept.
  Only on the test map's floor, not on a level a test session loads ([testtools.md](../testtools/the-squad.md),
  "The squad").
* **The controls panel**, which the main game no longer shows: the keys (the tools' too) and
  the debug line (character, state, clip, surface, weapon), top left under the health bars. It
  starts hidden, with a small "H: help" at the bottom; H shows and hides it.
  `BF_TEST_HELP=<s>[;<s>...]` presses H then.
* The status message ("Instant kill off", the tools' messages) goes after its 2 s here too: it
  was counted down only on maps with doors, so on the flat floor it stayed.
* **Developer tools**: a free camera with a teleport (F), an NPC spawner (N), an object
  spawner (O, in the air from the free camera), any level's buildings and static scenery to put
  down, drawn and solid as in their levels (P, #185), the squad menu (T, #302), a sky picker (Y),
  a music picker (U) and a level switch (L), which takes the test session to any level and back. On a level the rack and the
  pickup grid aren't there; the rest of this list is. `--test` with `BF_MAP=<level>` starts on
  that level. See [testtools.md](../testtools/README.md).
* **The map editor** (#191): F5 switches to its edit mode, out of the gunplay (no character, gun
  or HUD; a free camera, the right mouse button held looks; the tools at the mouse pointer), and
  F5 again plays from where the pointer is, the camera flying down to the squad and back up after.
  `cargo run --bin bf_play -- --edit` starts the session editing, and so does the front end's
  EDITOR (#193). See [editor.md](../editor.md).
* **Backspace** leaves for the front end only when pressed twice within 2 s in a test session (a
  stray key doesn't throw away what was built); the first press says so, and the controls panel's
  tools line names it. `BF_TEST_BACK` still goes at once. A session opened from EDITOR comes back
  to the main menu on EDITOR, its test-map state gone.
  Where it goes depends on the map (#312): a **campaign mission** (the mission flow is running:
  `MissionMode::active`, the tutorial or any zone reached from CAMPAIGN) goes to the main menu on
  CAMPAIGN, as the end menu's Quit does (the game has no Backspace; its pause menu's Quit goes
  there, FUN_00084f50); a deathmatch or squad deathmatch map goes back to its SELECT MISSION on
  that map ([menu.md](../menu.md)). A map started with `BF_MAP` and no front end goes by its kind,
  the same: `BF_MAP=e34` (a campaign zone) leaves for the main menu on CAMPAIGN, `BF_MAP=sdm_e34`
  for SQUAD DEATHMATCH's SELECT MISSION on it. So does a campaign zone a test session switched
  to (the test world's level switch): after the double Backspace it leaves for the main menu on
  CAMPAIGN, not that zone's SELECT MISSION. Checked with `BF_TEST_BACK=70 BF_MENU_SHOT=<png>
  BF_MENU_SHOT_FRAME=400` (the shot is of the menu it arrives at) on both.
