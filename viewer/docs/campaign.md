# Campaign state: game flags, objectives and cash (#221)

`src/bf/gameflags.rs` (the flag tree), `src/bf/campaign.rs` (the missions), `src/bin/play_flags.rs`
(the state as the game keeps it), `src/bin/play_objectives.rs` (the MISSION OBJECTIVES menu). The
level's gamevar and score triggers that read and write it are [triggers.md](triggers.md)'s. This
is the tutorial's step 2 (#28's plan; the first slice of #25). Everything here is read from
default.xbe (the addresses are in the code's comments and in [code-map.md](code-map.md)) and
checked against the tutorial's takes (`todo/28-tutorial/`, their `.watch.csv` logs: the member
flags, the objectives' `state`, the clock); what's fitted says so.

## The game flags

`common/default-gameflags.xmb` (the file campaign-bf.xmb's root names, h_1a468238
"default-gameflags"; schema `gameflags`) is a tree of folders and records: 504 records, each a
float, an int or a bool as the schema types it, with a value and a default. The game keeps one
tree per save slot (the session's vtable +0x210); its records are {name hash, value, default,
type}, found name by name (FUN_000782c0), read as a float (FUN_00077e80), an int (FUN_00077e00) or
a bool (FUN_00077e40), and written by FUN_00077eb0: a float as it is, an int truncated, a bool
true unless |v| < 1e-6. A path names a record folder by folder:
`missions/m00/h_eaf792f4/state` (h_0630ae70 is "missions", h_e26b9b3e "m00").

The demo makes the tree from the defaults the first time a level starts and keeps it as long as
bf_play runs, level after level, as a campaign carries it (`flags::Campaign::flags`), through
the mission flow ([mission.md](mission.md)). A new campaign started from the front end (JOIN
CAMPAIGN's START, #226) makes it anew as its first level starts and writes
`campaign/difficulty-setting` (SELECT DIFFICULTY's -4, 0 or 4: the session's +0x230, 0x7c750); in
a mission the difficulty is that flag (the session's +0x234, 0x7c720; `BF_DIFFICULTY` still
overrides it). Saving it is #27's. Only a mission's triggers see it: on a
multiplayer map the gamevars have no flags (they read 0 and write nothing) and the score-triggers
no cash, so a match doesn't touch the campaign's (inferred: [triggers.md](triggers.md),
"Gamevar-triggers").

## The missions (`common/campaign-bf.xmb`)

`bf::campaign::Campaign`: the squad's four `squad-char`s (tex, brutus, flint, hawk), the three
extra members `h_1f45a087` (the Confed marine types h_ef8787e4 Sgt. Gonzales, h_f48ed65e Private
Cogburn, h_fd89e6c8 Corporal Stout), and each mission (`<level h_046391fd="m00">`): its zones
(`file-name`, and `h_ed54d255`, the squad members a zone doesn't have: the tutorial's "brutus hawk
flint"), its primary and secondary objectives (`PATH`, the objective's flag folder;
`display-id`, its line; `h_f71c3e99`, its cash on the end screen: 5000 and 1000 in the
tutorial). A list attribute is the attribute repeated, one value per name.

## The squad's flags

Two sets of flags say who is in the squad.

* **The save-game slots** `campaign/save-game/character-1..4/{archetype, h_03761692, ...}`. When
  a level other than the current one is asked for (FUN_0007f730), the session's +0xb4 (0x841e0)
  sets them back to their defaults (FUN_00083fb0), clears the save-game cash (FUN_000846b0), then
  fills them (FUN_00084230): first the squad members the zone has (the session's +0xec,
  FUN_00080f20: the `squad-char`s not in its h_ed54d255), alive (`h_03761692` true); then as many
  extra members as the zone lacks (+0xf0, 0x80ff0), alive only if a player controls the type. In
  the tutorial: slot 1 Tex alive; 2-4 Gonzales, Cogburn, Stout, not alive. e40: Tex, Brutus,
  Flint, Hawk, all alive.
* **The member flags** `<type>/h_03761692` (default-gameflags' top-level `<tex>`, `<hawk>`,
  `<brutus>`, `<flint>`, `<h_ef8787e4>`, `<h_f48ed65e>`, `<h_fd89e6c8>`: all false by default).
  Only the session's +0xbc writes them (FUN_000814b0, in a mission), from four places:
  * a squad-type character's set-up (FUN_0012f440) sets its type's to 1: each placed one as the
    level loads, a spawn's clone;
  * its death sets it to 0 and its revival back to 1 (the character's +0x144, FUN_00112830);
  * a spawn-trigger with h_0fa4f510 whose template is a squad-type character, as it links
    (FUN_00209890), sets it to whether the session counts the template in the squad (+0xb8,
    0x83ee0: its save-game slot alive, or a player controlling its type, or no slot of its type
    at all);
  * a spawn from a trigger without h_0fa4f510 sets it to 1 (FUN_00209900).

  That gives every value the takes show: tex 1 and the marines 0 at the tutorial's start (take03's
  `1|0|0|0`: the marines' five placements set theirs, then their spawn-triggers' links clear
  them, their slots not alive); `0|0|0|0` while Tex is dead (take05, take08) and 1 again when he's
  recloned; Cogburn and Stout 1 once the V-squad's spawns (no h_0fa4f510) have run (take11's
  `1|0|1|1`). The start marines' spawns at the farcast are re-armable (clones): each clone's
  set-up sets its flag, the member check takes it away at once (its death clears the flag again).

The level's gamevar-triggers test the member flags (the tutorial's four at its first update:
the farcast flashes, the co-op hint, the V-squad); a spawn's member check (FUN_00209900,
[triggers.md](triggers.md)) reads the save-game slots.

## The cash

The campaign mode's squad (mode+0x1c, made with the mode as each level starts) keeps the
mission's cash (`flags::Cash`): its own totals, what went in under the null name above and below
0 (+0x2c, +0x30), and a map of entries by key, each a sum and a count (+0x38).

* **A score-trigger** (OPERATE, 0x20ba00): its `amount` x (1 + 0.1 x the difficulty) with
  h_11819686 (all 75 of the data's), into the totals (FUN_0005c5e0; every score-trigger's key is
  the null name). The difficulty is the session's n (play_damage.rs `Difficulty`: STANDARD -4),
  so the tutorial's 5000 and 1000 are $3000 and $600: take12's end screen ("Primary Objectives
  +$3000", "Secondary Objectives +$600").
* **A death** (FUN_00157070, on the character's CBT_DEAD): its type's award (`<base><h_1edc28b5
  amount ...>`, `character_type::Award`) x (1 + 0.1 x n), keyed by the type's display name
  (type+0x120), to the campaign squad when h_f61ecbaf is set (whoever killed it: the marines'
  kills count) and to its own squad when h_199ff965 is and it's on the squad's side
  (FUN_0005c730). The Red Hand 100: +$60 each (take12: "24 x Red Hand +$1440"); the four squad
  types -5000: -$3000 (take12: "1 x Tex -$3000"); the three marine types -1000: -$600. Every other
  type's award is 0.

take12's end screen, from those: Cash Awards +$5040 (3000 + 600 + 1440), Cash Penalties -$3000,
Cash $2040. The record also counts the squad's members who die on the level (+0x20: FUN_00115d50
at 0x115e10, unless a cutscene has the world), the end screen's "Operatives Lost". The end screen
([mission.md](mission.md), #226) shows them: "Cash $x / $5040", 5040 being the zone's
`max-score` (BRUTAL's 11760) back at STANDARD (11760 / 1.4 x 0.6), x the penalties plus the
awards capped at it; take05's "$-1526" is a frame of the number counting to $-3000.

## The MISSION OBJECTIVES menu

The game's BACK (VB 62, OPEN_OBJECTIVES_MENU) opens it over the game in a mission, but not
during a cutscene (FUN_0008c8c0 opens no menu while the world's cutscene bit is set, #222:
[cutscenes.md](cutscenes.md)), nor under another menu that holds the game's clock (the item list
and the Squad Command menu lock the player's game input, BACK's binding with it: FUN_000ac080,
FUN_000c11f0; #319: the pad's BACK closing the Squad Command menu opens nothing); the demo's I
key (or the pad's Back). I again, Enter, Space, or the pad's A or B close it. The game's clock
stops under it (take04: the clock held at 60275 from the press at 62.6 s to the B at 74.5 s), so
the demo's does: `frame_dt` is 0 (nothing in the world moves: characters, triggers' timers,
effects). The menu takes the controls: the keys and mouse buttons pressed in a frame it's up
(the one it opens in and the one it closes in too) are taken from the frame's input before the
game's systems run, so Space or Enter closing it doesn't make Tex jump or click a test tool's
panel, and Backspace waits; `read_input` passes nothing to the character, which stops firing
and aiming, and a Tab held as it opened doesn't step the item when let go. It shows at once,
whole (take04: in the first frame after the press), over the HUD.

It's game-options' menu h_0cfeaf6f, 470 x 360 at (85, 60) on the 640 x 480 screen:

| What | From |
|---|---|
| Panel | the picture h_0d0ce2e2: the bar h_f5392b60 (512 x 32, alpha 180) at the top and, flipped, at the bottom (y 328); the body h_ee1efd50 stretched to 470 x 296 at y 32 |
| Title | "MISSION OBJECTIVES" (h_08300dcd), centred in (2, 2) 246 x 24, style h_e53b67a8: 153 209 251, outlined black, 16 pt |
| Headings | "Primary Objectives:" (h_e7ba5b50) at (8, 40), "Secondary Objectives:" (h_e4ff251d) at (8, 182), h_0a959b2e: 153 209 251, 14 pt |
| Lists | `primary-objectives` (33, 76) 400 x 100, `secondary-objectives` (33, 214) 400 x 106, h_ff54aaed: 30 110 150, outlined, 14 pt, centred up and down. Their lines (FUN_000f8480 / FUN_000f8540): each objective whose `enabled` flag is true (FUN_000f8310), its text, then " (completed)" (h_eb2ff8cb) for state 1 or " (failed)" (h_fdc04956) for -1, each followed by a "\n" |
| Legends | "A Accept" (h_18291278) at (12, 332), "B Back" (h_f1e06461) at (136, 332), h_e35bb628: the button's letter in the icon font h_0e38a4ca at 12 points (its record 0x3c5fe8 on the atlas h_0e233fad, FUN_000cd5d0: outlined discs, cut flat on top; drawn as the tutorial panels' legends, [hints.md](hints.md)), then the word, 153 209 251, 12 pt |

Measured on take04's frame at 63.0 s (1.5 px a unit), as the demo places them: the capitals
13.3 units tall at 14 pt, 10.0 at 12 pt; the title's glyphs at 15.5 (a fit: the take's are 21 px
where the demo's font draws 19 at 14); the headings' capitals 5.3 under their boxes' tops (a fit);
the lists' line pitch 24 units (the height the menu gives a 14 pt line), the lines and the empty
one after the last centred in the box (take04's single lines at 166.7 and 308.0, the rule's 167.3
and 308.3); the words' first ink 2.3 (headings) and 3.3 (lines) right of their boxes. A line
wider than its box wraps (the tutorial's primary does with " (completed)"): no take shows one.

## Test hooks

* `BF_FLAG_LOG=1`: at a level's start the campaign's squad, the save-game slots, the member flags
  and the mission's objectives' flags; every flag write after (the time, the path, the value
  before and after, who: a gamevar-trigger, FUN_0012f440, FUN_00209890 ...). `BF_FLAG_LOG=all`
  lists the whole tree at the start too. The objectives menu logs its lines as it opens.
* `BF_FLAG_SET=<path>=<value>[;...]`: sets flags at the level's start, after the squad's set-up
  and before the triggers' first update. The path is names or h_xxxxxxxx joined by `/`
  (`h_f48ed65e/h_03761692=1;h_fd89e6c8/h_03761692=1`: both V-squad marines present, so the
  tutorial skips the V-squad); the value a number or true / false. (The objectives' `state`
  is set back to 0 by the tutorial's own first update.)
* `BF_CASH_LOG=1`: every change to the cash: who, the amount and its scaling, the total and the
  record.
* `BF_TEST_OBJECTIVES=<s>[,<open s>[,<key>]]`: opens the objectives menu at that time (the game's
  clock), for that long (2 s by default, in real time, or the capture's steps under `BF_CAPTURE`:
  the game's clock stands still meanwhile). With a key (`space`, `enter` or `i`), that key's press
  closes it, put into the frame's input as a player's would be; the press must not reach the game.

```sh
# the tutorial's start: the squad slots and the member flags (take03's 1|0|0|0)
BF_MAP=tutorial BF_FLAG_LOG=1 BF_MUTE=1 BF_BACKGROUND=1 cargo run --bin bf_play
# both objectives through their chains, then the menu at 7 s: "(completed)" twice, $3600
BF_MAP=tutorial BF_FLAG_LOG=1 BF_CASH_LOG=1 BF_TEST_OBJECTIVES=7,2 \
  BF_TRIGGER_FIRE="trigger_router103,ENTER_DETECTOR,1;trigger_router38,ENTER_DETECTOR,1;trigger_router50,ENTER_DETECTOR,1;@TUTORIAL_terrorist_0,DESTROYED,3;...;trigger_calculator344,ENTER_DETECTOR,6;trigger_calculator346,ENTER_DETECTOR,6;trigger_calculator348,ENTER_DETECTOR,6" \
  cargo run --bin bf_play
```

## Verified

`BF_MAP=tutorial` unless said, `BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps, the logs read and
the frames looked at:

* **The start** (`BF_FLAG_LOG=1`): the slots Tex alive, Gonzales, Cogburn, Stout not; the member
  flags tex 1, the three marines 0, brutus, hawk, flint 0 (take03's `1|0|0|0`); both objectives
  `enabled` true, `state` 0 (written 0 by h_ed0c1544 and h_e0a5fecf at the first update). The
  four member tests at the first update: Tex's TRUE, the marines' FALSE; calculator496 counts 1 of
  4 (the co-op hint stays, as take03 shows it), calculator501 none (the V-squad stays, take11).
* **The primary** (`BF_TRIGGER_FIRE`: routers 103, 38, 50 at 1 s spawn the twelve, their copies'
  DESTROYED at 3 s): router99 passes the twelve to h_e8ea7ba2, at 12 h_fb6dc902 starts timer193,
  whose pulse 1.5 s later pays score510's 5000 x 0.60 = +3000 and sets
  `missions/m00/h_eaf792f4/state` 0 -> 1 (h_11b9e83d).
* **The secondary** (calculators 344, 346, 348 at 6 s, as the three building plates feed them):
  calculator345 at 3: `missions/m00/h_1f7b9cf2/state` 0 -> 1 (h_089d8e5f) and score509's 1000 x
  0.60 = +600. Total $3600.
* **Deaths** (`-- --test`, three Red Hand dummies in front of Tex, `BF_TEST_FIRE=hip`, then
  `BF_TEST_DIE=6`): +60 each, "3 x Red Hand +180" (take12's "+ 180" in its cash display); Tex's
  death: his flag 1 -> 0 and -3000, "1 x Tex -3000".
* **The farcast and the V-squad** (`BF_TRIGGER_FIRE=trigger_router_confed_one_fx,USER1,1;...two...;
  ...three...;trigger_spawn64,USER1,2;trigger_spawn76,USER1,2.2`): each start marine's clone sets
  its flag, is taken away (the save-game slot not alive), its flag back to 0; spawn64 and spawn76
  set Stout's and Cogburn's: tex 1, Gonzales 0, Cogburn 1, Stout 1 (take11's `1|0|1|1`).
* **The menu** (`BF_TEST_GOTO=57.5,68.22,56.8,68.22 BF_TEST_OBJECTIVES=1,2`, take04's spot, against
  take04's frame at 63.0 s): the panel, headings, lines and legends at the take's places within a
  pixel or two (the legends since #224: the 12-point icon discs line up to the pixel, the words'
  capitals 396.0-408.0 as the take's); the title's 176 units across to the take's 183 (the take spaces its letters
  wider). With both objectives completed (the chains above, the menu at 7 s): "Eliminate every
  terrorist at their main base." / "(completed)" (wrapped) and "Investigate all terrorist
  buildings. (completed)". It's whole in its first frame; the world's frames while it's open are
  identical (the clock stopped), and a `BF_TRIGGER_FIRE` due at 7.5 s waited for it to close.
* **A new campaign** (#226: the front end's path at HARD, `BF_FLAG_LOG=1`): the flags made anew
  from default-gameflags.xmb (504), then `campaign/difficulty-setting -4 -> 0 (0x7c750: SELECT
  DIFFICULTY)`. Tex's death counts "operatives lost 1" (take05's), not while a cutscene has the
  world.
* **Regressions**: sdm_e34 (a multiplayer map: no member check, the menu doesn't open; door
  h_10616962 opens and closes from its plate as before); e40 (the four slots alive, their flags
  1, its two objectives read; the squad spawns as before).

## Not done here, or not known

* **The cash display**: the game shows each change on the HUD (FUN_000b1d70, "+%8.0f" /
  "-%8.0f", accumulating while one is up; take12: "+ 180" in pale blue as three kills land, then
  "+ 3000" in green as the primary pays, rising above the radar). Not drawn yet: its widget's
  place, motion and colours aren't traced.
* **Carrying the flags to the next mission**: take12 shows Cogburn's and Stout's flags back to 0
  once Rescue Brutus (M01_a, no marines) has started (`1|0|1|1` -> `1|0|0|0`); what clears them
  isn't traced (the squad's save at a level's end, FUN_00082920, resets only Stout's, through
  FUN_00082040). The demo carries the flags as they are into the next mission.
* The mission's results (its score, max-score, the award bits) aren't written to the flags at
  its end (FUN_00106d10: [mission.md](mission.md)).
* The title's letters are spaced closer than the take's.
* A wrapped objective line isn't in any take; nor are more lines than one per list.
* The random operands' seed (FUN_00233b90's) is the demo's own; the data's random operands (13
  gamevars, A kind 4) aren't in the tutorial.
* A score-trigger's h_1747360f (bit 0) isn't read by its handler; an award's h_e925f48b isn't read
  by FUN_00157070; `per_damage` and `on_hit` (h_ffc62853, h_06961311) are set by no type, and the
  demo pays deaths only.
* Under the menu the pad's presses aren't taken from the game's other systems as the keys' and
  the mouse's are (the demo reads the pad in few places).
