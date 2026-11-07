# Test map developer tools (`src/bin/play_testtools.rs`, #111; `src/bin/play_testworld.rs`, #116; `src/bin/play_testscenery.rs`, #185; `src/bin/play_testsquad.rs`, #302)

*This folder was the page `testtools.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../testtools.md` is now a pointer to them.*

Demo tooling for trying features out on the test map ([play.md](../play/test-map.md), "Test map"),
not the game: the keys, speeds and layout are the demo's choices, none of them used by the
game's controls. A panel on the right lists the open tool's keys; the controls panel (H, top
left) lists them all. They work on any map of a test session: `--test` starts on the test map
(`BF_MAP=<level>` with `--test`: on that level), and L switches to any other.

The map editor ([editor.md](../editor.md), #191) is these tools' home: F5 switches to its edit mode
(no character in the world, the edit camera, the tools at the mouse pointer) and back to play,
and `--edit` starts the session there. The front end's EDITOR (#193, [menu.md](../menu.md)) starts
one too, without the command line: a test session of its own, as `--edit`'s (the test map's
data, `TestMap`, the tools' session, edit mode), from that choice until the front end is back
(Backspace twice), when its state goes (editor.md, "From the front end"). The hooks below run in
it as on `--edit` (its first map), driven from the menu with `BF_MENU_PRESS` or
`BF_MENU_SCREEN=editor BF_MENU_GO=1`. The menus' keys below are its tool registry's (`TOOLS`,
`open_tools`), the same in both modes; what the menus put down is listed in its `Placements`.
In edit mode "the crosshair" below is the mouse pointer (the crosshair's place with no pointer),
a click is any left click off the panel, and F and its teleports do nothing.

| Key | Tool |
|---|---|
| F | **free camera** on / off (in play). Mouse: look (once the mouse is captured); WASD: fly along the view; Space / Ctrl (or E / Q): up / down; Shift: 4x faster (6 m/s, 24 m/s). The character stands still meanwhile (its inputs are dropped). **Left click**: teleport the controlled character to the ground under the crosshair, **right click**: to the ground under the camera, both facing the camera's way, the follow camera behind them. F alone goes back without moving them |
| N | **NPC menu**: Up / Down or the wheel picks the character type: Brutus, Flint, Hawk or Tex, then every other type the loaded archives define (#128, [character-types.md](../character-types.md)) by name, where it's defined and HP ("Shrike Soldier (m09_a, 73 hp)", "Brutus (mp_common/sdm_bf, 75 hp)"; the key added to two that still read the same; nine lines show round the one picked; on the test map all 128 of the data's types, below); J sets friend / ENEMY, ; dummy / fight, ' follows you / stays (for the next one; enemy dummy that stays at first); click or Enter spawns one on the ground under the crosshair, facing the controlled character. N or Esc closes it (Esc keeps the mouse captured) |
| J / ; / ' / Delete | outside the menu, on the NPC under the crosshair (one the ray passes through, else the nearest within 1.2 m of where it meets the ground, the dead too): switch its team / switch dummy and fight / switch whether it follows the controlled character (#364) / remove it. Shift+Delete removes every NPC. (The fight toggle is ; so that B stays the Black button's Medkit shortcut in a test session too, as in play: [play.md](../play/item-box-and-item-list.md) **The item box and the item list**. The follow key is the apostrophe, the next key along the home row after J and ;: every letter is another control's in the demo, the game's, the test map's or the hint and Squad Command panels', and so are the digits 1-4) |
| T | **squad menu** (#302): the squad's four slots, its places round the radar and the keys 1-4 that take control of them (Brutus's, Flint's, Hawk's, Tex's), each with who holds it ("(empty)", a type, DEAD, "(you)"). Left / Right picks a slot, Up / Down or the wheel (Page Up / Down: a page, seven rows) a row of the list: first the **predefined squads** (#339: the campaign squad and the five squads squad deathmatch offers, then the demo's own Kingman Posse and Mutant Horde, #365 and #394: eight rows under "[predefined squads]"), then the character types under "[character types]", any the NPC menu offers (the four, then all 128 of the test map and the eight deathmatch classes of #394). It opens with the first character type, Brutus, picked as before the squads, the squads above it in view, so Enter straight after T changes one slot; a squad is picked on purpose. Click or Enter on a type puts a fresh one of that type in the slot: a real squad member (below), and in your own slot you become it where you stand. Click or Enter on a squad fills all four slots at once with that squad's members (the panel lists who goes where), your own slot becoming its member in place. Delete empties the slot (not yours: "Your own slot can't be emptied"). T or Esc closes it |
| O | **object menu**: every pickup type, every hand weapon (the rack's list), each grenade type and the breakable scenery, in groups (81 entries). Up / Down picks, Page Up / Down jumps a group; a see-through copy (alpha 0.6, a faint blue glow) stands on the ground under the crosshair; the wheel turns it 15 degrees a notch; click or Enter puts one there, and the menu stays open for more. O or Esc closes it. **In the free camera** the copy hangs in the air on the crosshair's ray, 4 m out (Shift + the wheel: 1-40 m, 0.5 m a notch; short of whatever the ray meets first by 0.3 m, and on the ground if that's where it ends up), with a cyan line down to where it will land and a ring there; a loose one placed there drops (see below), scenery hangs where it's put (a yellow ring round its foot instead) |
| P | **scenery menu** (#185): every level's buildings and static scenery (2,412 pieces of 53 levels, below), by level, each by its type's name (a hash) with its size, parts and how many its level places, the largest first. Up / Down picks, Page Up / Down jumps a level; a see-through copy stands at the crosshair, the wheel turns it 15 degrees a notch; click or Enter puts one there, drawn as its level draws it and solid as there, and the menu stays open for more. P or Esc closes it. In the free camera the copy hangs on the crosshair's ray as the object menu's does (Shift + the wheel: how far), a yellow ring at its foot, and it's put where it hangs |
| Y | **sky menu**: none, the map's own, or the sky of any level whose level file names one; click or Enter shows it over the map in place of its own, with that level's background colour |
| U | **music menu**: off, the map's own, or any level's music bank, mixed as the game mixes a level's (its music with its ambience bed under it); click or Enter plays it; Left / Right: the previous / next bank at once |
| L | **level menu**: the test map, the squad deathmatch maps, the deathmatch arenas and every mission zone, from the game's level list; click or Enter loads it in the same window, the tools with it |
| Home | **put back the pickups taken** (`play_testpickups.rs`): every pickup of the test map's grid and every one the object tool put down that has been taken is made again where it was put, and "2 pickups put back" shows. Works in play and in edit mode. See **Putting pickups back**, below |

**Sections**

* [NPCs](npcs.md)
* [The squad (#302, #339)](the-squad.md)
* [Objects, on the ground and in the air](objects.md)
* [Putting pickups back (Home)](putting-pickups-back.md)
* [Scenery (#185)](scenery.md)
* [Skies and music](skies-and-music.md)
* [Level switch; what's not done; the clicks (#191)](level-switch.md)
* [Test hooks](test-hooks.md)
* [Verified](verified.md)
