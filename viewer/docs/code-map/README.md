# Code map: default.xbe

*This folder was the page `code-map.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../code-map.md` is now a pointer to them. Where the text below says to search the page, search this folder (`grep -r 22c4d0 viewer/docs/code-map/`).*

An index of the default.xbe functions and addresses the project relies on: what each one does,
how sure we are, and where it's used or written up. Look here before tracing a function; it may
be done already. Addresses are virtual addresses in the US `default.xbe` (base 0x10000; `.text`
0x12000-0x3118b0, `.rdata` from 0x386340, `.data` from 0x3b5b00).

* `FUN_0022c4d0` is an address where Ghidra made a function: its C is in
  `decompiled/xbe/ghidra/c/` (`python tools/fn.py 22c4d0` prints it). `0x22d0b0` is code Ghidra
  didn't split out: read it in `decompiled/xbe/default.text.asm`. Data (tables, vtables,
  globals) is written `0x3bd9d4`; `tools/xbe_peek.py f|i|x ADDR` reads values. Search the page
  for an address's hex digits without `0x` or leading zeros (`22c4d0`, `7c720`) to find it in
  either form.
* `bit N` is a bit index (bit 0 is mask 1, bit 3 is mask 8); `flag 0x40` is a mask.
* A row covers a function; the places inside it that a page cites, and the constants it reads,
  are in that row. The constants also have their own table at the end.
* Name tables are `{length, name, value}` records. Their address here is where the game's text
  parser starts reading: the first name pointer, 4 bytes past the first record.
* Offsets: `char+0x490` is a character object's field, `type+0x220` its character type's
  (`[char+0xc]`), `skill+0x1c` its skill object's (`[char+0x5a4]`), `weapon+0x218` a weapon
  object's, `data+0xd0` the weapon data a weapon points at from +0x228.

**How sure** (the third column):
* **code**: read from the disassembly or Ghidra's C (static, not run).
* **live**: also checked in the running game (xemu's gdb stub, or a take that matches it).
* **data**: read from the game's data files (`decompiled/xml/`), with the code that uses it.
* **inferred**: deduced from its use, the data or the footage, not read line by line.
* **guess**: a reading nobody has checked.

Brackets narrow a word: the part that is only inferred, or the source's own rating (medium,
low).

**Where** (the fourth column): a docs page and its section, a source file under `viewer/src/`
(`play_shots.rs` for `src/bin/play_shots.rs`), a ticket. `todo/35` leads, spec or notes are the
research in the git-ignored `todo/35-*/` folder; `todo/damage-factor` is the lead's code
question in `todo/damage-factor/answer.md`. "This page" means it was read for this index.

**Adding a row.** Put it in its area's table, at its address. One line: what it does, with the
game's own names (enum names, strings, attribute names); how sure; where it's used. When a row
is wrong, correct it and say so in your report. When two places disagree, read the code, record
what it does in the row and say which place was wrong.

**Sections**

* [Damage and combat](damage-and-combat.md)
* [Weapons and projectiles](weapons-and-projectiles.md)
* [Characters and motion](characters-and-motion.md)
* [Faces](faces.md)
* [Skills and stamina](skills-and-stamina.md)
* [HUD](hud.md)
* [Effects / ALE (materials and shaders too)](effects-ale.md)
* [Levels and triggers](levels-and-triggers.md)
* [Cutscenes](cutscenes.md)
* [AI](ai.md)
* [Camera](camera.md)
* [Sound](sound.md)
* [Front end, movies and text](front-end-movies-and-text.md)
* [Other](other.md)
* [Constants (values read from `.rdata` / `.data`)](constants.md)
