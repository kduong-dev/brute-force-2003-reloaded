# Code map: default.xbe

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
question in `todo/damage-factor/answer.md`. "#73 branch" marks what only the unmerged branch
`worktree-agent-a592f37290834c399` has (`play_missile.rs`, play.md's *Missiles*). "This page"
means it was read for this index.

**Adding a row.** Put it in its area's table, at its address. One line: what it does, with the
game's own names (enum names, strings, attribute names); how sure; where it's used. When a row
is wrong, correct it and say so in your report. When two places disagree, read the code, record
what it does in the row and say which place was wrong.

## Damage and combat

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_00024ae0` | An AI percentage (char+0x49c or type+0x7c, x0.01). For non-squad characters it adds 0.05 x difficulty (halved when positive) and clamps to 0.5-1.5x its base and at most 1: -0.2 on STANDARD. That it's accuracy is a guess. | code (medium-low) | todo/damage-factor |
| `FUN_0004b9c0` | Squad-on-squad test of a hit: both are characters (object type 3) with a squad (+0x59c) and FUN_0005beb0 says they're on one side. FUN_0022c4d0 then deals 0.7 and posts EVT_SHOT_BLOCKED_BY_FRIEND (34). | code (the reading: medium) | #73 spec §6; todo/damage-factor; #73 branch `play_missile.rs` (`FRIENDLY`) |
| `FUN_00079e70` | Builds the game session (0x6f4 bytes, vtable 0x3a2228); the app object stores it at its +8 (0x684b8). | code | todo/damage-factor |
| `0x7c720` | Game session vtable +0x234: the difficulty n, the int game flag `campaign/difficulty-setting` (path at 0x401f60); 0 without a flag tree. STANDARD -4, HARD 0, BRUTAL 4. | code (n = -4 in the takes: by fit) | todo/damage-factor |
| `0x7c750` | Game session vtable +0x230: writes that flag. | code | todo/damage-factor |
| `FUN_000d9090` | Recomputes the world's load measure w (+0xc58) every second from counts of active characters and such; 0 in multiplayer. Pushes are scaled by (1 - w) x 0.4 + 0.6 (FUN_00157260, FUN_0021ade0); damage isn't. | code | scenery.md § Damage areas; `play_scenery.rs` |
| `FUN_000d9a20` | World update step: turns the first explosion queued on the world's list (+0x18, by a grenade's 0x149520) into a weapon blast (FUN_0022bfc0, with the queued type and factor) and its upright effect (FUN_0022cea0), then removes it (FUN_000e0810): one per update, no loop. Also ticks the blast pool (FUN_00225220). | code | this page; grenades.md § Sentry |
| `FUN_000e00b0` | Knock-back of a body killed by Brutus's charge, given h_f5dae098, h_0e406b26, h_fa20e794 (2, 2, 75); which value is which isn't traced. | inferred | todo/35 leads §4.2 |
| `FUN_000f13c0` | SELECT DIFFICULTY menu: STANDARD (h_087c40bb) takes game-options +0x50 (`easy`, -4), HARD (h_e4969005) +0x54 (`normal`, 0), anything else (BRUTAL) +0x58 (`hard`, 4), from `<difficulty-settings>` (h_ffc1becb). | code, data | todo/damage-factor |
| `FUN_001085b0` | Uses 0.1 x difficulty and 1 + 0.1 x difficulty; not traced. | code (use only) | todo/damage-factor |
| `FUN_0010f830` | Sets a combat target's hit points (+0xc), capped at its max (+8); not floored at 0. | code, live | xemu.md (`trainer.py`) |
| `0x10fa60` | A character's team: char+0x1d4 (through the +0x1b4 object's vtable +0x9c), or the type's default (+0x58) when it's -1. Squad 0, enemies 1 (xemu.md lists +0x1d4 as a guess). | code, live | todo/damage-factor; xemu.md |
| `FUN_00114010` | Builds a character's hit parts; puts each part's `Damage` (when it isn't 1) in the map at char+0x564 / +0x568. | code | todo/damage-factor |
| `FUN_00124a70` | Called after FUN_002232d0 in a character's damage handler; hit reactions is a guess. | guess | todo/damage-factor |
| `FUN_001252b0` | Called for BIOREACTIVE (3) hits by the character's damage handler (0x1257dc adds to char+0x554 first). Not traced. | code (use only) | todo/damage-factor |
| `0x125600` | A character's damage handler (combat-target vtable 0x39ddcc slot 0, set by FUN_00110d30; no Ghidra function). Two early exits with no damage (a world flag, char+8 then +0xc50 bit 4, and a check through the world's +0xbec; scripted invulnerability is a guess). Then x (1 - 0.1n) when the victim's team row has 1s in columns 3-6 of 0x3ffd80 (enemies), else x (1 + 0.1n) (the squad), n the difficulty (0x7c720, read at 0x12566d): on STANDARD enemies x1.4, the squad x0.6. Then x the victim's skill vtable +0xd8 (0x125747: Vengar's 0.85), its modifiers (+0x55c, +0x59c +0x1c, vtable +0x14), FUN_002232d0, FUN_00124a70. This is #73's unexplained x1.4 and the campaign squad's ~0.58 of a barrel blast (scenery.md). A live read: break at 0x125673 (eax = n) or 0x125735 (st0 = the factor). | code; n = -4 by fit | todo/damage-factor; todo/35 spec (Vengar) |
| `FUN_00125940` | Hit factor of a weapon hit on a character: the victim type's +0x220 (`damage-multiplier`, h_e7e9fcd3: 0.62 for the campaign squad) x the hit bone's `Damage` from the map at char+0x564 when it isn't 1 (head x5, or x3 for hounds, Brutus and ferals; upper arms x0.8; forearms and calves x0.5). Instant rays (0x2303a0) and projectiles (0x227a03) use it; blast shares and FUN_00223160 don't. | code (which bone each take's hit struck: fit) | todo/damage-factor; #73 spec §6; #73 branch play.md |
| `0x12ed40` | Character-type loader (no Ghidra function): copies the parsed record into the runtime type: max-stamina (parse +0x16c) to +0x224, `damage-multiplier` to +0x220; its neighbour h_1182c05d is scaled by pi / 180 (0x3a4fe0). | code | todo/damage-factor; todo/35 leads §2 |
| `FUN_0012f440` | A character's setup at spawn: stamina and max from the type's max-stamina (0x12f46e); the skill's +4 = stamina-cost (0x12f78c), +0x18 the character; resolves the type's collision bone list (+0x188, 0x14-byte records) against the model. | code (the bone list: medium) | todo/35 leads §2, §4, spec §1.2; todo/damage-factor |
| `FUN_00154740` | Fills a combat target type's per-damage-type factors (h_142be76f): all 12 to 1.0, then the data's pairs. | code | todo/damage-factor |
| `FUN_00156938` | Object constructor; at 0x15699f it sets the combat target's modifier (+0x18) to 0. | code | #73 branch play.md (damage factor) |
| `FUN_00156e50` | Fills a struck object's push (+0xe8..+0x100) from the damage message: the hit's direction x its damage. The debris's main entry gets it (FUN_0015ac90). | code | `play_scenery.rs` (`Push`, `DEBRIS_*`) |
| `FUN_00157070` | Scales a value from type+0x124 by 1 + 0.1 x difficulty and passes it to FUN_0005c730; a kill reward is a guess. | code (the reward: guess) | todo/damage-factor |
| `FUN_00157260` | A game object's message handler (item and projectile vtables +0x144). On CBT_DEAD (78, 0x4e) it queues the type's debris list (h_197caf14) and starts the countdown at its longest h_1b6a0ede; the push scaled by (1 - w) x 0.4 + 0.6. | code, live | scenery.md § Breaking; `play_scenery.rs`; `bf/character.rs` (`ObjectType`); #85 |
| `FUN_0015ac90` | Spawns one debris entry; the main one (h_1c1e17fe) is handed the hit's push. | code | `play_scenery.rs`; `bf/character.rs` (`Debris`) |
| `FUN_0015ae30` | Removes a broken object once nothing is left to spawn and the countdown is out. | code | `play_scenery.rs` |
| `FUN_0015af20` | Debris countdown, each frame: the first queued entry whose h_1b6a0ede is at least the time left spawns, one a frame, so the longest delay comes first. | code, live | scenery.md § Breaking; `play_scenery.rs`; `bf/character.rs` |
| `FUN_00194d70` | Parser of a motion set's `<collision>` bone list: `name` +0, h_1a19ff7a +4, `damage` (h_0fd8419b) +8, `recoil` +0xc. | code, data | todo/damage-factor |
| `FUN_001eee10` | System-link join ("SystemLink"): takes the host's difficulty from a signed 6-bit field (bits 4-9; packed at 0xf56f0). | code | todo/damage-factor |
| `FUN_00208bb0` | Level trigger conditions: `<evaluate-trigger difficulty-level=...>` (h_0e63f6cd) compares the difficulty with easy / normal / hard. | code | todo/damage-factor |
| `FUN_0020b900` | Uses 0.1 x difficulty and 1 + 0.1 x difficulty; not traced. | code (use only) | todo/damage-factor |
| `FUN_0021ade0` | Damage areas of an effect object (`<h_fb0a5f1d><Damage>`): each frame while h_f724cb8c < age <= h_f724cb8c + duration, every target within range (3D, no ray) whose factor for the Type is above 0 takes amount x dt: x (1 - d / range) for DFALL_LINEAR, (1 - d / range)^2 x (amount x dt)^2 for DFALL_EXPONENTIAL, in full for NONE and HALF_LIFE. | code, live | scenery.md § Damage areas; `play_scenery.rs` (`falloff`); #85 |
| `FUN_00222f60` | Starts a damage record (vtable 0x3948b8): type 1, amount 0, push +0x54 = 1.0, flags +0x5c. | code | this page; todo/damage-factor |
| `FUN_00223160` | Deals an amount of a damage-type from a source to a target: a bare record (FUN_00222f60) with the amount, type, attacker and the victim's position, sent to the victim's combat-target vtable slot 0 (0x125600 for a character): no hit bone, no `damage-multiplier`. Brutus's charge (0x13c463, and 0x13bd16) uses it with 200, DTYPE_EXPLOSION; also 0x13dd18 and 0x22ac6d. | code, live (take08: 200 to 280) | todo/35 leads §4.2; todo/damage-factor |
| `FUN_002232d0` | Combat-target take-damage (this = char+0x48 for a character). Slot 0 itself for other objects (vtables 0x39adec, 0x394874); a character's slot 0 is 0x125600, which ends here. A shield modifier (+0x18) takes its part first through its vtable +4 (0x223690 or 0x223400). Then x the type's factor for the damage-type (type+0x60 + 4 x Type, h_142be76f); 0 for DTYPE_GAS when +0x14 is set; CBT_DAMAGED (81) posted when not 0; the hit points down through FUN_0010f830; at 0, FUN_002230a0 (the kill: CBT_DEAD, which breaks scenery). The trainer's `hit` calls it directly, so it skips the difficulty and Vengar's 0.85. | code, live | xemu.md; scenery.md § Damage; `bf/character.rs`; `play_scenery.rs`; todo/35, 43, 85 notes; todo/damage-factor |
| `0x223400` | Energy shield absorb (vtable 0x3948cc +4; no Ghidra function): takes the damage from its pool (+0x10) at a rate per damage type; once the pool is spent, the rest goes through. | code | this page |
| `0x223690` | Shield absorb (vtable 0x39489c +4; no Ghidra function): takes min(the owner's stamina +0x490, the damage) off both, the stamina through FUN_001258d0. Likely the stamina shield (EST_STAMINA_SHIELD). | code (which shield: inferred) | this page |
| `FUN_00223780` | Falloff curve by mode (in EAX) at x: 1 damage x x, 2 damage x x^2, 3 all of it while x >= 0.5 else 2x x damage, otherwise all of it. An instant hit's range falloff (FUN_00230040, x = 1 - distance / range, Damage h_fb124e6c). Its other use, a blast's age taper (FUN_00225060), is unreachable: it needs flag 0x40, which no blast or bullet gets. grenades.md said its curve wasn't read (fixed). | code | play.md § Shots (Damage falloff); `play_shots.rs`; `bf/weapon.rs`; grenades.md § Sentry |
| `FUN_00224320` | Adds a body to a blast's gathered set, one entry per game object: point, normal, factor (1, or the hit-location factor FUN_00125940 when its last argument asks), physics body. A body without client data isn't added. FUN_00224a90 passes 0: blast shares get no hit factor. | code | this page; todo/damage-factor |
| `FUN_00224510` | Builds the hit record a blast deals to one gathered body: the entry's point, normal and object, the blast as the source (hit+0) and its centre, flags from the blast's. | code | this page |
| `FUN_00224770` | A blast deals: once its delay (+0x2c) is out, each gathered body goes through the weapon type's damage handler (0x22d0b0) with a hit record from FUN_00224510; with flag 0x10 (Damage h_0e373132: the Gas's) the share is scaled by dt. It runs every tick the blast lives: once for a life of 0 (the Sentry, the Therm Sweeper). | code | grenades.md § Sentry; #73 spec §5; #73 branch `play_missile.rs` |
| `FUN_00224a90` | Gathers a weapon blast's bodies when its delay reaches 0: physics bodies in reach of its radius (+0x24) round its fixed centre. Projectiles (object type 21) are skipped. Characters need a clear ray from the centre and their entry is at their position; a blast with flag 0x10 (the Gas's) skips characters whose +0x70 bit 0 is set. Other game objects (scenery) are taken with no ray and their entry stays at the blast's centre (normal from the object toward it), so their distance in FUN_0022c4d0 is 0. The static world's entry is at the blast's centre with its normal straight up: that entry lays a blast's scorch (FUN_0022ca20). Loose physics objects are pushed by (1.2 - d / r)^2 x damage x 1.5 (0x3bc6f0), only when the type's +0x234 is 1 or more. | code | scenery.md (debris push, line of sight); `play_scenery.rs`; #73 spec §5; todo/damage-factor; this page |
| `FUN_00225060` | A blast's damage now: +0x20. With flag 0x40 and a life (+0x30) above 0 it would be FUN_00223780(+0x20, 1 - +0x28 / +0x30), a taper over its life, its curve mode from the weapon type's +0x20c (h_fb124e6c). Nothing sets flag 0x40: the pool's init (FUN_00225000) clears 0x40 and 0x80 (0x225058), FUN_0022bfc0 sets only 0x80 from the hit, the copy at 0x226010 copies its source, and no write in 0x223000-0x234000 sets it. So the taper is unreachable. | code | grenades.md § Sentry |
| `FUN_00225220` | Tick of the blast pool (from FUN_000d9a20): counts each object's delay (+0x2c) to 0, then its life (+0x28). A weapon blast (flag bit 0) gathers through FUN_00225c40; a bullet flies (FUN_00225440, or FUN_00225c90 with a physics body); its effect follows (FUN_0021c380, life / start); then FUN_00224770 deals. | code | grenades.md § Sentry |
| `FUN_00225440` | Flies a bullet-type 0 projectile (FUN_0022bac0) in the blast pool: moves it by its speed (+0x1c) x dt along +0x10 and every third tick sweeps a ray from its last checked point, adding what it meets (FUN_00224320). grenades.md had it move a grenade's blast (fixed): weapon blasts don't move. | code | grenades.md § Sentry |
| `FUN_00225c40` | A weapon blast's gather step: FUN_0022c2e0 with its weapon type, then FUN_00224a90 (with a physics body, FUN_00225c90 first). | code | this page |
| `FUN_002278e0` | A projectile's collision callback: builds the direct hit, its factor the missile's +0x1cc x FUN_00125940 (at 0x227a03), and calls the type's damage handler (0x22d0b0); the missile dies (flag 2) without splitting. | code, live (#73 take01, take03) | #73 spec §3; todo/damage-factor; #73 branch `play_missile.rs` |
| `FUN_0022bfc0` | Makes a weapon blast (pool at world+0x1fc, flag bit 0) where a hit happened: damage +0x20 = a new roll (FUN_0022c940) after a direct hit, the source blast's (FUN_00225060) after a blast, x the factor; radius +0x24 = Damage radius; life +0x28 / +0x30 = h_04ea9251; flag 0x10 = h_0e373132; flag 0x80 = the hit's +0x58 bit 1, the other bits kept (0x22c0b7-0x22c0ca; nothing sets 0x40); delay +0x2c = 0.05 + 0.2 x U. It doesn't move. Made by the damage handler (radius not 0), the world's explosion queue (FUN_000d9a20: grenades), a missile's timeout (FUN_00227760) and a trigger-only bullet (FUN_0022f9b0). | code | #73 spec §5; #73 branch `play_missile.rs`; grenades.md § Sentry |
| `FUN_0022c4d0` | Builds the damage record of one hit. A direct hit (hit+0 = 0) takes a roll (FUN_0022c940). A hit by a blast takes the blast's damage (FUN_00225060); for a weapon blast (flag bit 0) the Damage's h_011cb154 (type+0x200) picks the share: false, half of it anywhere inside the radius; true, all of it within half of r' (the radius less the target's size, its skill's vtable +0xe0, at most r / 2; 0 for the four squad skills, 0x14e100), then (1 - 2 (d / r' - 0.5))^2 of it to the edge. Then x the hit's factor (+0x4c), the push (type+0x234; 2 for a blast hit when 0), and x 0.7 with EVT_SHOT_BLOCKED_BY_FRIEND (34) when FUN_0004b9c0 holds. grenades.md said a blast isn't tapered by distance and this adds "a second taper for some targets"; the distance rule is this one (fixed). | code, live (the half share fits #73's HP logs) | #73 spec §6; todo/damage-factor; #73 branch `play_missile.rs`, `play_scenery.rs`; grenades.md § Sentry |
| `FUN_0022c940` | The damage roll: min (type+0x1f4) + U x (max (+0x1f0) - min); for a character shooter x its +0x754 (a damage power-up), x its skill's vtable +0xec (Flint's double hit and scoped x1.5, 0x13cc70, so his blasts' rolls too), then its modifier list (+0x55c) and its squad's (+0x59c, +0x1c), vtable +0x10. | code | #73 spec §6; todo/damage-factor; #73 branch play.md |
| `FUN_0022ca20` | Lays a weapon's decal (one of its list at type+0x26c, at random) at a hit record's point along its normal, through the decal manager (world+0x200, FUN_00213e60); never on a character (object type 3), nor when the hit's +0x58 bit 1 is set. **Disagreement settled:** the #73 spec has it lay the Therm Sweeper's scorch on the struck surface at a direct hit, "not on characters", yet take03 shows a scorch on the ground after a direct hit on a hovering Shrike. The damage handler (0x22d0b0) calls it on a direct hit only for a Damage radius of 0.05 m or less (bullet holes). For blast weapons it calls it for each body the blast reaches that isn't a character, and the static world's entry sits at the blast's centre with its normal straight up (FUN_00224a90), so the scorch is laid round the blast point and projected down onto the ground or wall there, after a body hit too. | code, live (take03 4-7 s) | #73 spec §5; #73 branch `play_missile.rs` (the scorch); this page |
| `0x22d0b0` | The weapon type's damage handler (the object at type+0x1d4, vtable +8; no Ghidra function). FUN_0022c4d0 builds the record and it's sent to the struck object as MSG_DAMAGE (1, FUN_00012000), noting a kill. A hit by a weapon blast then gets only the scorch (FUN_0022ca20, Damage radius > 0.05). Any other hit gets a weapon blast (FUN_0022bfc0, radius not 0), the explosion effect (FUN_0022cfc0), the bullet hole (FUN_0022ca20, radius <= 0.05 and hit+0x58 bit 0 clear), then FUN_0022cd80 and FUN_0022b5b0 (not read), which it skips when hit+0x58 has bit 1 set and bit 2 clear. Reached from a projectile's collision (FUN_002278e0) and from blasts (FUN_00224770). | code (MSG_DAMAGE reaching the combat target's slot 0: inferred) | #73 spec §3, §5; #73 branch `play_missile.rs`; this page |
| `FUN_00230040` | An instant hit's damage: scaled by FUN_00223780(Damage h_fb124e6c, 1 - distance / range) x the hit factor FUN_00125940 (the victim's `damage-multiplier` x the bone, at 0x2303a0; the hit body's owner in edi at 0x23038a). | code | play.md § Shots (Damage falloff); `play.rs`; `play_shots.rs`; todo/damage-factor |
| `0x394874` | Combat-target vtable of other objects: slot 0 is FUN_002232d0. | code | todo/damage-factor |
| `0x39489c` | Vtable of a shield modifier: slot 0 the deleting destructor (FUN_00223610), +4 the absorb FUN_002232d0 calls (0x223690: from the owner's stamina). | code | todo/damage-factor; this page |
| `0x3948cc` | Vtable of the energy shield modifier: slot 0 the deleting destructor (FUN_00222d00), +4 the absorb (0x223400: from its pool, at a rate per damage type). | code | todo/damage-factor; this page |
| `0x39adec` | Combat-target vtable of other objects (scenery): slot 0 is FUN_002232d0. | code | todo/damage-factor; scenery.md § Damage |
| `0x39ddcc` | A character's combat-target vtable (set by FUN_00110d30): slot 0 is 0x125600. | code | todo/damage-factor |
| `0x3a2228` | Game session vtable: +0x230 sets the difficulty (0x7c750), +0x234 gets it (0x7c720). | code | todo/damage-factor |
| `0x3bd9d4` | DTYPE_ name table (records from 0x3bd9d0; parser FUN_001849f0): 0 NORMAL, 1 BALLISTIC, 2 BLADED, 3 BIOREACTIVE, 4 GAS, 5 FLAME, 6 PARTICLE, 7 SONIC, 8 LASER, 9 PSYCHIC, 10 EXPLOSION, 11 POWERBLADE. The docs cite 0x3bd9d0 (the first record's length word), todo/35 cites 0x3bd9d4: the same table. | code | grenades.md § Energy grenade; `bf/character.rs`; `play_energy.rs`; todo/35 leads §4.2 |
| `0x3bda70` | DFALL_ name table (parser FUN_00184ac0): 0 NONE, 1 LINEAR, 2 EXPONENTIAL, 3 HALF_LIFE. | code | `bf/character.rs` (`AreaDamage`) |
| `0x3ffd80` | Team hostility table, 10-byte rows (row = team, byte k = 1 if hostile to team k; 24 readers). Team 0 (the squad) `00 01 00 00 00 00 00 01 01 00`, team 1 (enemies) `01 00 00 01 01 01 01 01 01 00`, team 7 all 1 (hostile to itself). 0x125600 tests columns 3-6 (0x3ffd83..86) for the difficulty's sign. | code, data | grenades.md § Sentry; `play_sentry.rs`; #73 spec §4; todo/damage-factor |
| `0x401f60` | The game-flag path of the difficulty: {0xf1a4f931 `campaign`, 0x1a98de0a `difficulty-setting`}. | code, data | todo/damage-factor |
| `0x469aac` | Holds the world: [0x469aac]+0x1fc the blast pool, +0x200 the decal manager. | code | this page |
| `0x469b58` | Holds the app object (main builds it, 0x6980d; 0x469b54 holds it too); [0x469b58]+8 is the game session (vtable 0x3a2228). | code | todo/damage-factor |

## Weapons and projectiles

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_000c4e10` | Spread from accuracy: S = (100 - clamp(A + bonus, 0, 100)) x 0.03 degrees; the bonus is the accuracy object's +8 (-1 unless the holder's script property h_ec5337fb is set, then its h_072837bb). It sits among the HUD reticle's code. | code | play.md § Shots (Accuracy); `play_shots.rs` |
| `FUN_00112d90` | Character script properties; its neighbour at 0x11302c reads h_072837bb (-100..100), the accuracy bonus. | code | `play_shots.rs` |
| `FUN_0011e130` | Starts a weapon slot change (FUN_00122da0 / FUN_00122940 / FUN_00122b20 test the slot at +0x5b0), then FUN_0022dc00 on the gun put away. | code, live (the LZR takes) | play.md § Ammo; `play_ammo.rs` |
| `FUN_00120c50` | "Can reload": the clip is below its size and the reserve isn't empty. | code | todo/35 leads §3 |
| `FUN_00120d40` | The character's reload: out of the scope first (FUN_001249b0), the rounds leave the reserve at once (FUN_0022daf0), the reload motion starts. A recharging gun posts CBT_OUT_OF_CHARGE (83, 0x53) instead; clip and reserve empty, CBT_OUT_OF_AMMO (82, 0x52): the HUD's "switch to %s.". | code, live | play.md § Ammo; `play_ammo.rs`; `play_hud.rs`; `play_shots.rs`; #114 |
| `FUN_00121860` | Disarming an enemy's mine (the tutorial's "Hold [X] to disarm", with 0x14a410's state 6). Not followed. | inferred | `play_sentry.rs` |
| `FUN_00122940` | A slot-change step called by FUN_0011e130; tests the slot at +0x5b0. | code | `play_ammo.rs` |
| `FUN_00122b20` | Another slot-change step (as FUN_00122940). | code | `play_ammo.rs` |
| `FUN_00122da0` | Another slot-change step (as FUN_00122940). | code | `play_ammo.rs` |
| `FUN_00123ea0` | Raises the gun or keeps it raised: while the ready timer (char+0x7dc) runs it holds it at 4 s or more (FUN_00123f60); otherwise it starts it (FUN_00123f60, then FUN_00126320) unless a held item (+0x1fc) or a state (+0x214) blocks it. Called by the fire command (0x1225ec), the player controller (0x134a3f), the AI's attack check, and Flint's skill update every frame while on (0x13cbe4, 0x13cc01). **Disagreement:** `play_shots.rs` says it "counts" the timer and todo/35 leads read it as Flint's target selection; it's neither. | code | `play_shots.rs` (`READY_HOLD`); todo/35 leads §4.4 |
| `FUN_00123f60` | The ready timer: when char+0x7dc is out it's set to the type's ready time (FUN_0010fce0: type+0x230 or +0x238); while it runs it's kept at 4 s or more (0x3c05fc). | code (the takes' taps fit) | play.md § Shots (Taps and the raise); `play_shots.rs` |
| `FUN_001241b0` | The accuracy cap: the data's max, or its scoped cap while the holder's +0x7a8 is set (then FUN_00222ac0). | code (+0x7a8 as the scope: inferred) | `play_shots.rs` |
| `FUN_00146930` | At 0x1469f9 it registers the IFSET_PROXIMITY_EXPLOSIVE (8) item set with vtable 0x39bc50. | code | `play_sentry.rs` |
| `FUN_00146cb0` | The Sentry's range test: the item's h_0a811e94 (item type +0x178) squared against the 3D distance from the mine's node to a character's node. | code | grenades.md § Sentry; `bf/weapon.rs`; `play_sentry.rs`; #80 |
| `FUN_00146e00` | The Sentry's check: through every living character, a friend within the radius (the mine's team, kept at +0x1c0, unless the team table makes that team hostile to itself; or the thrower) ends it with no blast; anyone else there is a target. It goes off with a target and no friend. | code, live (#80 takes) | grenades.md § Sentry; `play_sentry.rs` |
| `0x147430` | The Sentry's target filter (no Ghidra function): slot 0 of the filter vtable 0x39c05c that 0x147620 passes to FUN_00146e00; slot 4 (0x1474f0) is the abort test. A character whose type doesn't handle mines passes. One whose type does (byte +0x297, hazard kind 3) asks its skill object (char+0x5a4, vtable +0x94, with 3 and the mine; not the brain at char+0x830). The default (0x280040) says 1, and the character then passes only within 0.3 m across (0.09 at 0x3a52f0) and 1.5 m up or down (0x3a4fb4, 0x3a52f4). Only Tex's (0x13d8f0) says 0, for a mine (object type 0x18) whose owner (+0x1e4) is in his squad (+0x59c), and then he passes. | code | grenades.md § Sentry; `play_sentry.rs` (`WISE_REACH`; its "brain" is this skill call) |
| `FUN_00147570` | Sentry on-placed (vtable +0xc): marks it placed (+0x1bc bit 1) and keeps its owner's team at +0x1c0; no arming delay. | code | grenades.md § Sentry; `play_sentry.rs` |
| `0x147620` | Sentry per frame (vtable +0; no Ghidra function): once placed, a timer (+0x1d0) passing 0.15 s (0x3a51e4) runs the check, then restarts at rand x 0.1 (0x3a4ff0); a check that says so sets it off at once (vtable +0x15c). | code | grenades.md § Sentry; `play_sentry.rs` |
| `0x149520` | A grenade going off (vtable +0x15c of the grenade class 0x39b688; no Ghidra function): builds an explosion hit at the grenade and queues it on the world's list with the explosion type and a factor, (float) word [grenade+0x1de] (0x149616-0x149648; set to 1 at 0x14932f); FUN_000d9a20 makes the weapon blast and the effect. It also sends an event (FUN_001fe3d0) and a noise of the explosion type's +0x2c0 (FUN_0004d490). grenades.md had the blast travel and be dealt without a distance rule (fixed). | code | grenades.md § Sentry; this page |
| `FUN_0014a410` | Item vtable +0x140; the Sentry's disarm uses its state 6. | inferred | `play_sentry.rs` |
| `FUN_0018a1d0` | Loader of the accuracy block h_fb327295 (data+0x4..+0x14, read as ints). | code | `bf/weapon.rs` (`ShotData::accuracy`) |
| `FUN_0018a310` | Damage loader; at 0x18a427 it stores h_fb124e6c (the falloff mode) at the Damage record's +0x1c. | code | `bf/weapon.rs` (`falloff`) |
| `FUN_0018a7a0` | Bullet-block loader (with FUN_0018a8d0): `<bullet>` at type+0x210 (data+0x38). | code | #73 spec §1; #73 branch `bf/weapon.rs` (`MeshBullet`) |
| `FUN_0018a8d0` | Bullet-block loader: the rest of `<bullet>` and its `<mirv>` (count +0x78, mode +0x7c, object +0x80, spread +0x84 / +0x88). | code | #73 spec §1; #73 branch `bf/weapon.rs` |
| `FUN_00193b70` | Weapon loader: stores the `<weapon>` attributes in the weapon data (offsets in `ShotData`). | code | `bf/weapon.rs` |
| `FUN_002229b0` | Turns a direction about the three world axes by uniform random angles within +-S degrees (degrees to radians 0.017444, 0x3a52e0). | code | play.md § Shots (Accuracy); `play_shots.rs` |
| `FUN_00222ac0` | Accuracy: holds the current value (accuracy object +0xc = weapon+0x1dc) within [min, cap]. | code | play.md § Shots; `play_shots.rs`; `bf/weapon.rs` |
| `FUN_00222b30` | Accuracy regain: + per-second x dt while the cooldown is out, doubled crouched (holder +0x1f8 = 6). | code | play.md § Shots; `play_shots.rs`; `bf/weapon.rs` |
| `FUN_00222c10` | Accuracy loss per trigger cycle: - per-shot, halved crouched, floor min + 15. | code | play.md § Shots; `play_shots.rs`; `bf/weapon.rs` |
| `FUN_002237c0` | Target search: up to N living characters hostile to a team (0x3ffd80) within a range and a cone of a heading, with a clear ray to each one's origin (the feet), most central first. The MIRV split asks for 30 degrees, a seeking child for 60. | code | #73 spec §4; #73 branch `play_missile.rs` (`search`) |
| `0x226000` | The projectile class, 0x226000-0x229000 (mesh projectiles, object vtable 0x3946c8). | code | #73 branch play.md § Missiles, `play_missile.rs` |
| `FUN_00226360` | Gives a projectile the weapon's target object (weapon+0x360) and aim point (+0x1ec). | code (what +0x360 holds for the player: inferred) | #73 spec §3; #73 branch |
| `FUN_002266d0` | Steering: each frame removes half the velocity across the way (0x406f94 = 0.5) and pulls the speed 15% toward the bullet speed. | code | #73 spec §3; #73 branch (`STEER_*`) |
| `FUN_00226b80` | Seek: a child without a target looks every 0.25 s for the most central hostile within its range and 60 degrees (FUN_002237c0). | code | #73 spec §4; #73 branch |
| `FUN_00226cc0` | Projectile tick (vtable +0x6c): steering; an aim point within 1 m or over 60 degrees off (dot < 0.5, 0x3a4f74) is given up for a point 200 m ahead; the proximity fuse every 0.25 s within Damage radius x 0.375, for a human player's missile (FUN_00112800); life (+0x1d8), then the second timer (+0x1d4); at life 0 MIRV modes 1 and 2 split (FUN_00227f80), mode 1 also at its apogee. | code, live (#73 watch logs) | #73 spec §3, §4; todo/73 notes; #73 branch |
| `FUN_00227760` | A missile going off without a collision (its timers or fuse): a weapon blast (FUN_0022bfc0), the upright effect (FUN_0022cea0), the shake and the noise. | code | #73 spec §5; #73 branch |
| `FUN_00227ce0` | Sets a projectile's velocity: the bullet speed along its heading. | code | #73 spec §3; #73 branch |
| `FUN_00227dd0` | Sets a projectile up from its bullet block: flags at +0x220, life, straight phase (+0x230), second timer, fuse reach (+0x214 = radius x 0.375). | code | #73 spec §3; #73 branch (`Missile`) |
| `FUN_00227f40` | Starts the bullet's flight effect on a projectile. | code | #73 spec §3 |
| `FUN_00227f80` | MIRV split: looks for up to the child count of hostiles (FUN_002237c0: the range, 30 degrees), spawns the children (at most 6) beside the parent at a random roll (+360 / count each), turned spread-max x 90 degrees off its heading, at its speed, living range / speed x U(0.85, 1.15), MIRV mode 0, the targets dealt out in turn (else a point 200 m ahead); the parent ends without a blast. spread-min isn't read. | code, live | #73 spec §4; #73 branch (`split`) |
| `FUN_00228c10` | Gives a projectile's attached effects their user parameter, life / start life. | code | #73 spec §3; #73 branch |
| `FUN_002291d0` | The casing pool: a pooled physics object (its life and material aren't traced). | code | `play_shots.rs` (`SHELL_*`) |
| `0x22b000` | The weapon class, 0x22b000-0x233000 (the fire loop, shots, ammo). | code | play.md § Shots; `play_shots.rs` |
| `FUN_0022bac0` | Launches a bullet-type 0 (PROJECTILE) or 3 (PHYSICAL_PROJECTILE) shot as a moving object in the blast pool: a roll, life = range / speed, the bullet speed (+0x1c) from the muzzle, its flight effect and sound; type 3 gets a physics body. Its hits reach the damage handler as direct hits. | code | this page |
| `0x22da30` | Weapon vtable +0x15c (no Ghidra function): a flag, whether the holder's +0x7a8 is set. When it is, FUN_0022f0e0 and FUN_0022dc00 (and FUN_002327f0) take the scoped rate h_019c314a (data+0xd0) instead of the rate (data+0xcc). | code (+0x7a8 as the scope: inferred) | `bf/weapon.rs` (`scoped_rate`) |
| `FUN_0022da80` | A weapon's reserve: its holder's inventory item (1, ammo-type at data+0xec) through FUN_0014cc30. | code, live | play.md § Ammo; `play_ammo.rs` |
| `FUN_0022daf0` | Starts a weapon's reload: the rounds leave the reserve, the burst counter (+0x28c) is cleared. Tex's berserk calls it on his second gun (0x118ae7). | code, live | play.md § Ammo, § Shots; `play_ammo.rs`; `play_shots.rs`; todo/35 leads §4.1 |
| `FUN_0022dc00` | A weapon put away: sets its cooldown to 1 / rate (data+0xcc, or data+0xd0 when 0x22da30 says scoped), clears the burst counter and the fire flag (+0x224 bit 2). | code, live | play.md § Ammo, § Shots; `bf/weapon.rs`; `play.rs`; `play_ammo.rs`; `play_shots.rs` |
| `FUN_0022e1d0` | A burst's shots after the first, each turned by up to h_e704fd69 degrees of yaw and of pitch, read as turned from the one before (the pellet walk). | code (the walk: inferred, open in play.md) | play.md § Shots (Bursts and pellets); `bf/weapon.rs`; `play_shots.rs` |
| `FUN_0022e4c0` | Once a frame before the fire loop, turns each muzzle's aim (FUN_0022e5b0) by the accuracy spread (FUN_002229b0 with FUN_000c4e10's S). | code | play.md § Shots (Accuracy); `play_shots.rs`; #73 spec §2 |
| `FUN_0022e5b0` | The aim from one muzzle. | code | `play_shots.rs` |
| `FUN_0022ebc0` | Recharging clips, every weapon object every frame: while the clip isn't full and the cooldown is out, a timer runs and adds a round each ammo-regen (h_1d1e0e9c, type+0x14c) seconds. | code, live | play.md § Ammo; `bf/weapon.rs`; `play_ammo.rs` |
| `FUN_0022ec90` | Counts the cooldown (weapon+0x218) down by the frame time. | code | play.md § Shots (Fire timing); `play_shots.rs` |
| `FUN_0022ee30` | The weapon's per-frame update: the aim's turn (FUN_0022e4c0), then the fire loop (FUN_0022efe0). | code | `play_shots.rs` |
| `FUN_0022eee0` | Throws a casing (FUN_00231b10) for each whole one owed (+0x208). | code | play.md § Shots (Casings); `play_shots.rs` |
| `FUN_0022efe0` | Runs the fire loop while the weapon's fire flag (+0x224 bit 2) is set; with the world's +0xc50 bit 4 set it clears the cooldown, the burst counter and the fire flag instead. | code | `play_shots.rs` |
| `FUN_0022f0e0` | After a shot sets (not adds) the cooldown: the burst delay h_e6c60892 (data+0xd8) while the burst counter (+0x28c) is below h_e4076713 (data+0xdc) and a round is left in the clip (+0x1f8); otherwise 1 / rate (data+0xcc, or data+0xd0 when 0x22da30 says scoped), clearing the counter and the fire flag (+0x224 bit 2) at the cycle's end. | code, live | play.md § Shots; `bf/weapon.rs`; `play_ammo.rs`; `play_shots.rs` |
| `FUN_0022f170` | Sets off the muzzle effect h_fd88830d on a shot. | code | `bf/weapon.rs`; `play_shots.rs`; #73 spec §0 |
| `FUN_0022f1d0` | Launches one shot by bullet-type: 0 and 3 FUN_0022bac0, 2 FUN_0022fb50 (mesh projectile), 4 FUN_002317e0 (an instant ray, whatever the bullet's speed), 5 FUN_0022f9b0; 1 nothing. | code | play.md § Shots; `bf/weapon.rs`; `play.rs`; #73 spec §3; #73 branch |
| `FUN_0022f2a0` | The fire loop, once a game frame: while the trigger holds and the cooldown is out, the shot (FUN_0022f1d0), the clip drop when FUN_0022fe00 allows, the fire sound (FUN_002326e0) and the muzzle effect (FUN_0022f170) in the same frame, casings owed (+0x208 += h_fdc93b33), recoil (FUN_00222c10); an empty clip ends it (FUN_002327f0). | code, live | play.md § Shots, § Ammo; `play_shots.rs`; `play_ammo.rs`; `bf/weapon.rs`; #73 spec §0, §2 |
| `FUN_0022f840` | Turns a launched projectile to the aim (weapon+0x25c, after the accuracy turn). | code | #73 spec §3 |
| `FUN_0022f9b0` | Bullet-type 5 (TRIGGER_ONLY): makes a weapon blast (FUN_0022bfc0); the rest isn't read. | code (partly read) | this page |
| `FUN_0022fb50` | Launches a mesh projectile (bullet-type 2): the bullet's mesh-name object from its pool, set up (FUN_00227dd0), at the muzzle (FUN_00232180), turned (FUN_0022f840), at speed (FUN_00227ce0), with the weapon's target and aim point (FUN_00226360) and its flight effect (FUN_00227f40); life = range (FUN_002324c0) / speed. | code | #73 spec §3; #73 branch |
| `FUN_0022fe00` | Whether a shot takes a round: excuses a zero-delay burst's shots before its last. | code, live | play.md § Shots; `play_shots.rs` |
| `FUN_002317e0` | Instant ray (bullet-type 4): carries the flight effect only when its counter (weapon+0x290) is 0, which then restarts at h_eeb9e75a. | code, live | play.md § Shots (Tracers); `bf/weapon.rs`; `play_shots.rs` |
| `FUN_00231b10` | Throws a casing object (h_19b21bf5) from the h_ea1abbbe hardpoint: 5 cm back along its z, at 2-3 m/s along its z turned a quarter turn about the vertical, plus up to 0.5 m/s on each other axis; nothing without a casing object. | code (its life and bounce: guess) | play.md § Shots (Casings); `bf/weapon.rs`; `play_shots.rs` |
| `FUN_00232060` | Attaches the muzzle effect object at each muzzle hardpoint, in the hardpoint's frame, with h_e60e2074's rate. | code | `bf/weapon.rs`; #73 branch play.md (Muzzle effects), `play_shots.rs` |
| `FUN_00232180` | Places a launched projectile at the muzzle. | code | #73 spec §3 |
| `FUN_002324c0` | The bullet range through the holder's skill (char+0x5a4, vtable +0x54): unchanged by default (0x10e3a0), x1.25 for Flint's rifles (0x13ce80). | code | #73 spec §3; #73 branch |
| `FUN_002326e0` | The shot's fire sound, in the fire loop's frame. | inferred | #73 spec §0 |
| `FUN_002327f0` | The empty trigger cycle (the trigger held on an empty clip once the cooldown is out): the reload (FUN_00120d40) when there's a reserve, else the empty-fire click (h_e6acaff7, type+0x114) with the cooldown set as for a shot; clears the burst counter. | code, live | play.md § Ammo, § Shots; `bf/weapon.rs`; `play_ammo.rs`; `play_shots.rs` |
| `0x3946c8` | Projectile object vtable (mesh projectiles): +0x6c the tick FUN_00226cc0; +0x1d8 life and +0x1d4 the second timer, watched live. | code, live | todo/73 notes |
| `0x39b688` | Grenade object vtable (the Roller's): +0x15c going off (0x149520); the fuse counts down at +0x1d4 (25 s for the Roller). | code, live | todo/79 notes |
| `0x39bc50` | IFSET_PROXIMITY_EXPLOSIVE (the Sentry) vtable: +0 per frame (0x147620), +0xc on-placed (FUN_00147570), +0x58 FUN_001473f0. | code | grenades.md § Sentry; `play_sentry.rs` |
| `0x3bd63c` | BTYPE_ name table (parser FUN_00184510): 0 PROJECTILE, 1 RAY_DO_NOT_USE, 2 MESH_PROJECTILE, 3 PHYSICAL_PROJECTILE, 4 S_BEAM_EFFECT, 5 TRIGGER_ONLY. | code | #73 spec §1; #73 branch play.md |
| `0x3be7f4` | WM_ name table (parser FUN_00185f10): 0 NO_MIRV, 1 MIRV_ON_APOGEE, 2 MIRV_ON_EXPLODE. | code | #73 spec §1; #73 branch `bf/weapon.rs` |
| `0x3bf260` | Axis name table (bullet h_f096af84): 0 ROLL_AXIS, 1 PITCH_AXIS, 2 YAW_AXIS. | code | #73 spec §1 |
| `0x3bf714` | WTYPE_ name table (parser FUN_00186e80): 0 PISTOL, 1 RIFLE, 2 CANNON, 3 HEAVY, 4 VEHICLE, 5 GRENADE, 6 DATA, 7 NATURAL, 8 NONE. | code | todo/35 leads §4.4 |

## Characters and motion

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_001098f0` | Read with FUN_0014dd50: the character's crouch height, for motion states 6 and 7 (MT_CROUCH, MT_CROUCH_WALK). | code (a reading) | `play_shots.rs` (`Trigger::crouched`) |
| `0x10f8e0` | Character vtable +0x124 (no Ghidra function): reads h_17a554bf (object +0x70 bit 4), the mark of the characters a player can take control of; only FUN_000da8f0 and FUN_000db3e0 read it. | code | xemu.md; #102 |
| `FUN_00110d30` | Character constructor: vtable 0x39ddd8, combat-target vtable 0x39ddcc. | code, live | xemu.md; todo/damage-factor |
| `FUN_00112790` | Whether a character is the locally controlled one. | code (meaning inferred) | todo/35 spec §3 |
| `FUN_00112800` | Whether a character has a human player (its player record's flag 2). | code (meaning inferred) | #73 spec §3; #73 branch (`Missile::human`) |
| `0x1154b0` | `FUN_001154b0` in the docs (no Ghidra function): the character position setter (vtable +0x38); moves the body and the model; the trainer's teleport calls it. | code, live | xemu.md |
| `FUN_00115d50` | Character death; at 0x115df5 it switches the skill off through FUN_0010e3e0 (its end sound and chatter, as for an off). | code, live (#35 take30) | todo/35 spec §3, §6 |
| `FUN_00117210` | Whether a character is free to act (no blocking state at +0x214 / +0x225, not dead, no held item of kind 0x2e at +0x1fc / +0x200), then its skill's vtable +0x14. Hawk's toggle needs it. | code (meaning inferred) | todo/35 leads §4.3 |
| `FUN_00117660` | Animation event switch (called from FUN_00118ef0): eaba3d48 at 0x118ae7 (Tex's berserk on: his second gun out through FUN_0014d090, FUN_0014d310, FUN_0014a880, FUN_0022daf0), 14ee2201 at 0x1180bb (off: stowed), ead033a3 at 0x118b8f (Hawk's stealth on or off), 0f8408d5 at 0x117b12 (clears the fire request). | code, live | play.md § Shots (Taps); `play_shots.rs`; todo/35 leads §4.1, §4.3, spec §2.3 |
| `FUN_00118ef0` | The character's animation-event callback: jump_launch to FUN_0012b630, every other event to FUN_00117660. | code | `decompiled/xbe/ghidra/README.md` (Player movement); this page |
| `FUN_0011caa0` | Character update (vtable +0x6c); calls the stamina update FUN_0011a840 at 0x11cedb. | code | todo/35 leads §2, spec §1.2 |
| `0x122040` | The character's command handler (vtable +0x15c; no Ghidra function): a jump table at 0x12276c over the CT_ commands 0-35. CT_TOGGLE_SPECIAL (8) at 0x1224a5 calls the skill's toggle (vtable +4; ignored while +0x640 is set unless the skill is on at 0 stamina). CT_FIRE_WEAPON (10) at 0x1225d7 raises the gun (FUN_00123ea0) and sets the weapon's fire bits (0x122614-0x122655) from the request at +0x260 bit 3. CT_TOGGLE_SNIPE (16) at 0x122399 holds the scope-in (0x1223da). CT_RELOAD_WEAPON (17) at 0x122069. CT_TARGET_LOCK_TOGGLE (18) at 0x12238d (no sender found). CT_DASH (19) at 0x1220ef. CT_JUMP (22) at 0x1226ef. | code | play.md § Shots (Taps, Muzzle effects); `play_shots.rs`; todo/35 leads §3 |
| `0x12b000` | The character controller, 0x12b000-0x12e000. | code | `decompiled/xbe/ghidra/README.md` |
| `FUN_0012b630` | Jump launch: 5.8 m/s up. From the jump_launch event (FUN_00118ef0), the jump command and the AI. | code | `play.rs` (`JUMP_SPEED`); ghidra README |
| `FUN_0012b740` | Landing: the sound, fall damage 100 x ((drop - 5) / 25)^2 from 5 to 30 m, a stop from 2 m. | code | `play.rs` (`FALL_HURT`); ghidra README |
| `FUN_0012b950` | Starts a fall (walked off a ledge). | code | ghidra README |
| `FUN_0012bc60` | Slide start: the push downhill, the surface's slide sound and effect (surface record +0x1e8). | code | `play.rs` (`SLIDE_*`); ghidra README |
| `FUN_0012bf40` | Ends a slide whose sweep achieved under a tenth of the move. | code | `play.rs` |
| `FUN_0012c020` | Slide update: the amount ramp (+0x71c) and the slide velocity. | code | `play.rs`; ghidra README |
| `FUN_0012cd10` | Character physics step: gravity 18 m/s^2, air drag 0.99 a frame, the collision sweep (FUN_00153810), ground contact within 65 degrees. | code | `play.rs` (`GRAVITY`); ghidra README |
| `FUN_0012dab0` | Fills the ground record at +0x43c: the slide flag, the ground kind, the world material, the normal. | code | ghidra README |
| `0x1332a0` | `FUN_001332a0` in the docs (no Ghidra function): the player controller update (vtable 0x39db74 +4), the pad's virtual buttons to commands. White (VB 9) at 0x134cff sends CT_TOGGLE_SPECIAL; LT (VB 7) or black (VB 63) at 0x134367, and holding X long enough to reload (FUN_00120c50) at 0x134c50, switch Tex's and Flint's skill off first; the left stick past 0.9 (0x3a50b8) sends CT_DASH (19) at 0x134f52; it raises the gun (FUN_00123ea0) at 0x134a3f. +0x10 the character, +0x20 the pad port. | code, live | xemu.md; todo/35 leads §3, §4.2, spec §2.1 |
| `FUN_001351c0` | A squad entry's character (entry vtable 0x39f878, +0xe0). | code, live | xemu.md |
| `FUN_0014dd50` | See FUN_001098f0. | code | `play_shots.rs` |
| `FUN_00153410` | Contact classification after the collision sweep. | code | ghidra README |
| `FUN_00153810` | Collision sweep (Ipion). | code | ghidra README |
| `FUN_001876a0` | MT_ text parser (table 0x3bfdac). | code | ghidra README |
| `FUN_00187770` | IK type text parser. | code | ghidra README |
| `0x30c440` | Startup code computing the ground-contact cosine (65 degrees, with fcos). | code | `arena.rs` (`FLOOR_NORMAL`) |
| `0x39db74` | Player controller vtable (+4 the update 0x1332a0). | code, live | xemu.md; todo/35 leads |
| `0x39ddd8` | Character vtable: +0x38 the position setter (0x1154b0), +0x6c the update (FUN_0011caa0), +0x124 0x10f8e0, +0x15c the command handler (0x122040). Characters sit in contiguous memory (virtual = 0x80000000 + physical); the live field offsets are in xemu.md and the todo/35 spec. | code, live | xemu.md; todo/35 leads |
| `0x39f878` | Squad entry vtable (+0xe0 the character). | code, live | xemu.md |
| `0x3bfdac` | MT_ name table (records from 0x3bfda8; parser FUN_001876a0): 6 CROUCH, 7 CROUCH_WALK, 8 DASH, 19 IDLE, 20-24 JUMP_*, 25 KNOCKDOWN, 34 RUN, 35 SLIDE, 36 SLIDE_END, 45 TOGGLE_SPECIAL, 46 TRANSITION, 47 WALK, 49 LUNGE. | code, live | ghidra README; todo/35 leads §2 |
| `0x3c0360` | CT_ command name table: {name, flag} records by index (no parser found). 0 IDLE, 8 TOGGLE_SPECIAL, 10 FIRE_WEAPON, 16 TOGGLE_SNIPE, 17 RELOAD_WEAPON, 18 TARGET_LOCK_TOGGLE, 19 DASH, 22 JUMP, 35 SURPRISE. | code | todo/35 leads §2 |
| `0x3c0550` | Movement constants, .rdata 0x3c0550-0x3c05ac (angles computed at startup with fcos). | code | ghidra README |
| `0x469ab0` | A global: `[[0x469ab0] + 0x1348]` heads the squad's `std::list` (each node's +8 an entry of vtable 0x39f878). | code, live | xemu.md |

## Skills and stamina

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_00023c80` | Whether a skill is on: +0x1c bit 0 and an owner at +0x18. | code | todo/35 leads §3 |
| `0x10e3a0` | Default skill vtable +0x54: returns the range unchanged. | code | #73 spec §3 |
| `0x10e3b0` | Default skill vtable +0x58: returns 0.2 (0x3a4f90); Flint's returns 0.8. | code | this page; todo/35 leads §4.4 |
| `0x10e3d0` | A shared method returning 1.0: the default skill's +0xd8, +0xe4, +0xe8 and +0xec. | code | #73 branch play.md |
| `FUN_0010e3e0` | Skill deactivate: clears +0x1c bit 0; the skill's own off (vtable +0x124); the end-skill sound (+0x12c); stamina tested at 0x10e40d: chatter h_09087c9c while it's above 0 (0x10e425), h_ee0a5cbc at 0 (0x10e442). The death handler calls it. **Disagreement:** todo/35 leads had the two tags swapped; the spec and the code agree on this order. | code, live | todo/35 leads §3, spec §4, §6 |
| `FUN_0010e470` | Skill activate: needs stamina >= 0.1 x max (0x3a4ff0; 0x10e482-0x10e4a6; equal is allowed); sets +0x1c bit 0; the skill's own on (vtable +0x120); the start-skill sound (+0x128); chatter h_1c0514bd (0x10e4c7); clears char+0x5a8 / +0x5ac. | code, live | todo/35 leads §3, spec §2.2, §4 |
| `FUN_0010e560` | Whether stamina is at least 10% of max (a skill method with no direct caller). | code | todo/35 leads §2 |
| `0x10e5b0` | `FUN_0010e5b0` in todo/35 (no Ghidra function): skill vtable +0x128, the type's start-skill sound (type+0x328) at volume 100 through FUN_0010e7a0, positional if +0x1c bit 2. | code | todo/35 spec §3 |
| `0x10e5e0` | `FUN_0010e5e0` in todo/35 (no Ghidra function): skill vtable +0x12c, the end-skill sound (type+0x330), positional if bit 3. | code | todo/35 spec §3 |
| `FUN_0010e610` | Skill vtable +0x130: a looping sound while on (type `h_f12894f0`, +0x32c); empty for the squad. | code | todo/35 leads §3 |
| `FUN_0010e7a0` | Plays a skill sound: in 2D at full volume (x0.01, 0x3a4f98) for the locally controlled character (FUN_00112790), otherwise only if positional, at the character. | code, live (#35 take18: a squadmate's switch is silent) | todo/35 spec §3 |
| `FUN_0011a440` | Character item update; at 0x11a7b9 a held object (+0x1fc) drains stamina by dt x its +0x25c (which items: not traced). | code | todo/35 leads §2 |
| `FUN_0011a840` | Stamina, each frame, for types with h_e78efbb2 (type+0x1a8 bit 0, tested at 0x11a8b8). On: stamina -= the skill's drain (vtable +0xbc); at 0 or below (0x11a8f5-0x11a900) it's set to 0 and CT_TOGGLE_SPECIAL sent (char vtable +0x15c, 0x11a914), every frame while the skill stays on. Then the skill's update (+0xc0) and its visibility (+0xe4) to char+0x68. Off: += stamina-recovery x dt, x0.25 in MT_DASH (0x3a5028), x2 in MT_CROUCH, x4 while the byte +0x5c is set (0x3a4fd0; never set in the takes), clamped (0x11a95a-0x11a9b3). | code, live | todo/35 leads §2, spec §1.2 |
| `FUN_00125870` | Sets max stamina (+0x494). | code | todo/35 leads §2 |
| `FUN_001258d0` | Sets stamina (+0x490), clamped to 0..max. | code | todo/35 leads §2, spec §1.2 |
| `FUN_00126df0` | Brutus's off (through vtable +0x124): fades the screen colour back, his material off. | code | todo/35 leads §4.2 |
| `FUN_0012e590` | Skill factory by skill-set (jump table 0x12e810): the object at char+0x5a4. | code | todo/35 leads §4 |
| `FUN_0013bef0` | Brutus's skill constructor (CSKILL_BRUTUS, vtable 0x39d678, block h_f5dc6aac): copies the regeneration (h_eaa78fdf), vengar-damage-multiplier, the charge's cost, reach, cone and damage, and the knock-back values. | code | todo/35 leads §4.2 |
| `FUN_0013bfe0` | Brutus's toggle (+4): on or off at once; sets char+0x260 bit 1. | code, live | todo/35 leads §4.2, spec §2.3 |
| `0x13c020` | Brutus's on (+0x120): vision mode 2 (char+0x644 and [+0x5e4]+0xd8), his kind-0x100 material controllers on (FUN_002107d0), a 0.6 s screen colour fade (FUN_000c6980). | code, live (vision mode 2 read) | todo/35 leads §4.2, spec §6 |
| `0x13c0b0` | Brutus's update (+0xc0): regenerates h_eaa78fdf (1) hit point a second while on, up to max; runs the charge while dashing. | code | todo/35 leads §4.2 |
| `0x13c140` | Brutus's +0xd8: 0.85 (vengar-damage-multiplier) while on, else 1.0. Called at 0x125747 in the character damage handler 0x125600, so it scales every hit that reaches him but not a direct FUN_002232d0 call. todo/35 leads couldn't find its caller. | code, live (#35 take27 fits) | todo/35 leads §4.2; todo/damage-factor |
| `FUN_0013c160` | Brutus's +0x1c: "Cannot use" (h_e784869e) for items worn on the face or an equip slot (IOU 1 / 7), with or without Vengar. | code | todo/35 leads (found, not in scope) |
| `FUN_0013c1c0` | Brutus's charge, dashing with Vengar on: enemies within 1.5 m and 30 degrees take 200 DTYPE_EXPLOSION (FUN_00223160, at 0x13c463), the vengar_hit effect, the type's sound, chatter h_13601f7b, and on a kill a knock-back (FUN_000e00b0); 100 stamina per hit. | code, live (#35 take08, take22) | todo/35 leads §4.2 |
| `0x13c570` | Brutus's drain (+0xbc): dt x 2.0 + the charge's pending cost (+0x2c). | code, live | todo/35 leads §4.2 |
| `0x13c650` | Hawk's on (+0x120): her kind-2 material controllers on (FUN_002107d0). | code | todo/35 leads §4.3 |
| `0x13c670` | Hawk's off (+0x124): the same, off. | code | todo/35 leads §4.3 |
| `0x13c690` | Flint's drain (+0xbc): dt x 3.2. | code, live | todo/35 leads §4.4 |
| `FUN_0013c9e0` | Flint's skill constructor (vtable 0x39d2c8, block h_0725cdb1): tracking-weapon-type, the range modifier, the double-damage chance and factor, the scope factor, the reticule speed. | code | todo/35 leads §4.4 |
| `FUN_0013cad0` | Flint's toggle (+4): off if on; on needs a gun in hand with rounds; with a rifle it marks +0x48 bit 0 and returns 2 (MT_TOGGLE_SPECIAL), otherwise it raises the gun (FUN_00123ea0). | code, live | todo/35 leads §4.4, spec §2.3 |
| `0x13cbf0` | Flint's update (+0xc0): raises the gun (FUN_00123ea0) every frame while on; switches off when the clip in hand runs dry. | code, live (#35 take29) | todo/35 leads §3, §4.4 |
| `0x13cc70` | Flint's +0xec, the attacker hook in the roll (FUN_0022c940): x2 on 15% of hits (h_18bf34d4, h_fe103e1b) while on, x1.5 while scoped (h_191dafb6). | code, live (fits) | todo/35 leads §4.4; todo/damage-factor |
| `0x13cd30` | Flint's +0xc4: the HUD marker (+0x4c, made by FUN_000b9120) follows the target (+0x20); writes h_081520f0 / h_108f0925 into the target's [+0xf0] object while on, 0.75x its own +0x68 when off (aim or tracking rates). | code (the rates' meaning: guess) | todo/35 leads §4.4 |
| `0x13ce80` | Flint's +0x54: range x1.25 (weapon-range-modifier) for WTYPE_RIFLE, on or off. | code | todo/35 leads §4.4; #73 spec §3 |
| `FUN_0013cf10` | Hawk's skill constructor (vtable 0x39d190, block h_0ec919f1). | code | todo/35 leads §4.3 |
| `FUN_0013cf90` | Hawk's toggle (+4): only requests (>= 10% stamina, not mid-fade, FUN_00117210); the activate_stealth overlay switches at its 0.83 s event; off at once at stamina 0 or dead. | code, live | todo/35 leads §3, §4.3, spec §2.3, §2.4 |
| `FUN_0013d030` | Hawk's drain (+0xbc): dt x 2.5, x5 (0x3a4fd4) with the Power Blade in hand. | code, live | todo/35 leads §4.3 |
| `0x13d0a0` | Hawk's +0xb0, firing while on: the stealth factor to 0, stamina - 2 x the weapon type's +0x18. | code, live (#35 takes 12, 13) | todo/35 leads §4.3 |
| `FUN_0013d1a0` | Hawk's stealth factor (+0x28) climbing back at h_f726177e (4) a second, clamped 0..1. | code | todo/35 leads §4.3 |
| `0x13d340` | Hawk's +0x104, using an item while on: stamina -5 (0x3a4fd4), -15 for IOU_PLACE_ON_GROUND and IOU_THROW_TO_USE, nothing for IOU_SKILL_MELEE_ATTACK. | code, live | todo/35 leads §4.3 |
| `FUN_0013d440` | Tex's skill constructor (vtable 0x39d058, block h_04aa86aa). | code | todo/35 leads §4.1 |
| `0x13d490` | Tex's toggle (+4): refused crouched, scoped, within his 1.0 s timer (+0x20), without a second gun or 10% stamina; on, it tops both clips up from the reserve (0x13d643-0x13d692) or shows "Out of Ammo" (h_ff471882); plays MT 45. | code, live | todo/35 leads §4.1, spec §2.3 |
| `0x13d720` | Tex's drain (+0xbc): dt x 2.2, x0.5 while the gun in hand's clip is empty. | code | todo/35 leads §4.1 |
| `FUN_0013d780` | Tex's update (+0xc0): switches off when the other gun's clip runs dry. | code, live (#35 take26) | todo/35 leads §3 |
| `FUN_0014a880` | Called when Tex's berserk brings his second gun out (0x118ae7); its own job isn't traced. | code (role only) | todo/35 leads §4.1 |
| `FUN_0014d090` | As FUN_0014a880. | code (role only) | todo/35 leads §4.1 |
| `FUN_0014d310` | As FUN_0014a880. | code (role only) | todo/35 leads §4.1 |
| `FUN_00190330` | Character-type field parser: max-stamina at the parsed type's +0x16c. | code | todo/35 leads §2 |
| `FUN_0019e1f0` | Parser of `<combat-skills>` (h_f7bc2603), with FUN_0019e210, FUN_0019e3c0 and FUN_0019e450: into the settings the app's vtable +0x48 returns. | code | todo/35 leads §4 |
| `FUN_0019e210` | See FUN_0019e1f0. | code | todo/35 leads §4 |
| `FUN_0019e3c0` | See FUN_0019e1f0. | code | todo/35 leads §4 |
| `FUN_0019e450` | See FUN_0019e1f0. | code | todo/35 leads §4 |
| `FUN_002107d0` | Switches a character's material controllers of one kind on or off (char+0x54c): Brutus's kind 0x100, Hawk's kind 2. | code | todo/35 leads §4.2, §4.3 |
| `FUN_00210880` | Reads a kind's level back from the material controllers (Hawk's stealth level, skill+0x24). | code | todo/35 leads §4.3 |
| `FUN_00210950` | Pushes a visibility value into the material controllers. | code | todo/35 leads §4.3 |
| `0x212380` | Makes a BF_InvisMaterial ("Invisibility Material", class 0xf3bf04d1, strings 0x40d914 and 0x40da9c) with texture h_ff43a70b (no Ghidra function). Its link to Hawk's kind-2 controllers is likely, not proven. | code (the link: inferred) | todo/35 leads §4.3 |
| `FUN_00212430` | BF_InvisMaterial's update: states with 0.5 s ramps (0x407070, 0x40706c), a sine at 2.5 (0x3a51f0, a double) x 0.1 (0x407078). | code | todo/35 leads §4.3 |
| `FUN_00213fc0` | Renderer; at 0x214093 it checks vision mode 1 (mode 2, Vengar's, wasn't found). | code | todo/35 leads §4.2 |
| `0x39d058` | Tex's skill vtable: +4 toggle 0x13d490, +0xa4 overlay filter 0x13d850, +0xbc drain 0x13d720, +0xc0 update FUN_0013d780. | code, live | todo/35 leads §4.1, notes |
| `0x39d190` | Hawk's skill vtable: +4 FUN_0013cf90, +0xb0 0x13d0a0, +0xbc FUN_0013d030, +0xc0 0x13d120, +0xe4 0x13d200, +0xe8 0x13d210, +0x104 0x13d340, +0x108 0x13d3f0, +0x120 0x13c650, +0x124 0x13c670. | code, live | todo/35 leads §4.3, notes |
| `0x39d2c8` | Flint's skill vtable: +4 FUN_0013cad0, +0x38 0x13ceb0, +0x54 0x13ce80, +0x58 0x13c990, +0xbc 0x13c690, +0xc0 0x13cbf0, +0xc4 0x13cd30, +0xec 0x13cc70, +0xf8 0x13c9b0, +0xfc 0x13cce0, +0x100 0x13cd10. | code, live | todo/35 leads §4.4, notes; #73 spec §3 |
| `0x39d678` | Brutus's skill vtable: +4 FUN_0013bfe0, +0x1c FUN_0013c160, +0xbc 0x13c570, +0xc0 0x13c0b0, +0xd8 0x13c140, +0x120 0x13c020, +0x124 0x13c0a0. | code, live | todo/35 leads §4.2, notes |
| `0x3bd814` | CSKILL_ name table (parser FUN_00184850): 1 BRUTUS, 2 FLINT, 3 TEX, 4 HAWK, 5 KINGMAN, 6 AIR_STRIKE, 7 BREATH_OF_VENGAR, 8 CLOAK, ... 14 SPIRIT_OF_VENGAR, ... 21 SHRIKE_THINKER. | code | todo/35 leads §1 |
| `0x3bec64` | SA_ name table (parser FUN_00186660): 1 SYSTEM_BYPASS, 2 BARRIER_BASH, 3 DISARM, 4 SILENT_KILL. | code | todo/35 leads §4.1 |
| `0x3bf99c` | EST_ name table (parser FUN_00187290): 1 ENERGY_SHIELD, 2 STAMINA_SHIELD. | code | todo/35 leads §2 |
| `0x469b54` | Points at the app object, as 0x469b58; its vtable +0x48 returns the combat-skills settings each skill constructor copies. | code | todo/35 leads §4; todo/damage-factor |

## HUD

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_000a9910` | At 0xa9ac2 the reticule-action 6 path shows "Cannot use this skill" (h_0848550b) when no squad member has the special action. | code | todo/35 leads (found, not in scope) |
| `FUN_000b6cc0` | The HUD bar widget's base class (the stamina and health bars). | code | todo/35 spec §5 |
| `FUN_000b7050` | Bar length: value / +0x6c (150) x +0x84 (249), at least 2 units above 0: 1.66 units per hit point. | code, live | todo/35 spec §5.1 |
| `0xb71d0` | Bar widget's draw (vtable +0x18; no Ghidra function): tints the frame while the blink (+0x40) is below 0, colour bytes a0 20 20 ff at 0xb720a. | code (how the tint combines: open) | todo/35 spec §5.4 |
| `FUN_000b7240` | Draws the bar's icon (h_1ac41530 for stamina) white, 16 x 16. | code | todo/35 spec §5.1 |
| `FUN_000b73d0` | Draws the fill, mixed toward white by the brighten value b (widget +0x14). | code, live | todo/35 spec §5.2 |
| `FUN_000b75a0` | Decays the brighten value by 0.5 a second. | code, live | todo/35 spec §5.2 |
| `FUN_000b76c0` | The low-value blink: a triangle wave (+0x40) between -0.5 and 0.5 at 1 a second while the value is at or below the threshold (+0x68). | code, live | todo/35 spec §5.4 |
| `FUN_000b7850` | Sets the value: brighten 1.0 when it falls (+0xbc), 0.5 when it rises (+0xb8); the trail behind it. | code, live | todo/35 spec §5.2, §5.3 |
| `FUN_000b7940` | Sets the value, the shown value and the trail at once (a hand-over). | code, live | todo/35 spec §5.6 |
| `FUN_000b79a0` | The health bar (the same base class). | code | todo/35 spec §5 |
| `0xb79f0` | `FUN_000b79f0` in todo/35: the health bar's frame pieces (full screen h_e5634eaf, h_eee38815, h_17470408) and fill (h_e801997b); no Ghidra function. | code | todo/35 spec (found, not in scope) |
| `FUN_000b7bf0` | The stamina bar subclass (vtable 0x39fcf8): threshold +0x68 = 0.15. | code | todo/35 spec §5 |
| `0xb7cb0` | `FUN_000b7cb0` in todo/35: the stamina bar's frame pieces (full screen h_1289112b, h_f5678943, h_0565a96e; split screen h_fb3507f2, h_fb342365, h_14837f08) and fill (h_e11c7d04); no Ghidra function. | code | todo/35 spec §5.4 |
| `FUN_000b7fb0` | The HUD feeding the bars each frame (0xb8166-0xb81d4, 0xb817d): the stamina bar's max = max health (char+0x50), its value = stamina / max stamina x max health, at least 1 when not 0. | code | todo/35 leads §2, spec §5 |
| `FUN_000b9120` | Makes a HUD marker object (Flint's tracking marker, skill+0x4c). | code | todo/35 leads §4.4 |
| `FUN_000c1280` | Builds the Squad Command menu ("Toggle Ability" h_19565285, entry 5); that order gives GOAL_TOGGLE_COMBAT_SKILL (127) at priority 2 (0xc1591, brain vtable +0xf4). | code, live (#35 take18) | todo/35 leads §5 |
| `FUN_000c5d20` | HUD reticle; at 0xc5d82 asks the skill's +0x100 (Flint's sets the reticle's +0x7c to 1.0 while on). | code | todo/35 leads §4.4 |
| `FUN_000c5e20` | HUD reticle; at 0xc5e48 reads the skill's +0xf8 (tracking-reticule-speed). | code | todo/35 leads §4.4 |
| `FUN_000c6280` | HUD reticle; at 0xc62a7 asks the skill's +0xfc (0.9 x (1 - marker+0x7c) while Flint's skill is on). | code (meaning: guess) | todo/35 leads §4.4 |
| `FUN_000c6980` | Starts a screen colour fade on the player's view (view+0xec): Vengar's 0.6 s, level 1, colour bytes ff 00 00. | code (the takes show a red flash) | todo/35 leads §4.2 |
| `FUN_000c72b0` | The HUD message widget h_1a5ceeb9 (hud+0xfc) that shows the skill's name: 20 units lower for each other message showing (0xc73b4-0xc73cb); fades at 3.0 a second (0xc7473; 3.0 at 0x39a848) once its timer is out; the hold before the fade (+0x74, 0xc7438) isn't traced. | code, live | todo/35 spec §5.5 |
| `FUN_000c75c0` | Sets that widget's text (font h_0c626fd5, size 14 full screen, 12 split). | code | todo/35 spec §5.5 |
| `0x39fcf8` | Stamina bar vtable (+0x18 the draw 0xb71d0, +0x30 0xb7cb0). | code | todo/35 spec §5 |

## Effects / ALE

Materials and shaders are here too.

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_0001d0c0` | Builds a frame from a direction (y up for a level one): the explosion turned along a missile's way (FUN_0022cbe0), a decal's frame (FUN_0022ca20). | code | #73 branch play.md, `ale_fx.rs` |
| `0x65320` | ALE emitter constructor: sets the curve base values (100 a second, 2 s), the editor's defaults, unused once a curve has keys. | code | #73 branch play.md (the launch glow) |
| `0x8fa50` | `FUN_0008fa50` in the docs (no Ghidra function): the self-lit shader's (h_f539fe8c) names to offsets: h_e01baa40 colour +0x70, alpha +0x7c, time-scale (h_01590d7a) +0x58. | code | squad.md § Memory chip; `play_dna.rs` |
| `FUN_0008fb90` | Sets the scroll's time left (+0x54) to FLT_MAX when h_08c2d2ee is absent. | code | `play_dna.rs` |
| `0x8fca0` | `FUN_0008fca0` in the docs (no Ghidra function): the self-lit shader's scroll, h_fb0bff34 / h_e002ae8e a second, only while h_08c2d2ee seconds are left. | code | squad.md § Memory chip; `play_dna.rs` |
| `FUN_0008fd20` | The self-lit shader's render-state setup: h_0f5ae13f 1 is additive (ONE / ONE); with h_17aba73c (or alpha below 1) and h_1a08f318 0, SRCALPHA / INVSRCALPHA without z-writes; h_18954f8d 1 no culling; pixel shader 0 (FUN_000a4d00). What xemu draws for the memory chip doesn't match it. | code (no culling: medium) | squad.md § Memory chip; `play_dna.rs` |
| `FUN_0009e960` | Writes a material's h_e59d69a0 (60) to render state +0x294 of the state cache, flagging it when not 255; which state that is isn't established. | code | squad.md § Memory chip; `play_dna.rs` |
| `FUN_000a4d00` | Sets a pixel shader (51 callers). | code | squad.md § Memory chip; `play_dna.rs` |
| `FUN_000a4d80` | Packs four floats into a shader constant (33 callers). | code | squad.md § Memory chip; `play_dna.rs` |
| `FUN_00213e60` | Adds a decal through the decal manager: point, normal, frame, the decal type, two sizes, an object's node or the world. | code (arguments: inferred) | this page |
| `FUN_00214710` | Decal manager: clips the world's triangles for a decal (with FUN_00217920), so a scorch wraps the rock. | code | #73 branch play.md |
| `FUN_00217920` | See FUN_00214710. | code | #73 branch play.md |
| `FUN_0021c380` | Sets an effect's user parameter (+0xbc). | code | #73 branch `play_missile.rs` |
| `FUN_0022cbe0` | Spawns an effect type at a hit, turned along the hit (FUN_0001d0c0), with a factor (1.0 from FUN_0022cfc0); also called by FUN_0022cd80. | code | #73 branch play.md, `ale_fx.rs`, `play_missile.rs` |
| `FUN_0022cea0` | A bullet's explosion without a collision: its effect type (type+0x250, or a world material's override from the list at type+0x25c) upright, and its sound (+0x254). Missile timeouts and the world's explosion queue. | code | #73 spec §5; #73 branch |
| `FUN_0022cfc0` | A bullet's explosion at a hit: the effect type (type+0x250; per-material overrides at type+0x25c when the hit's +0x58 bit 1 is set) through FUN_0022cbe0, and the impact sound (+0x254) at the hit. | code | #73 spec §5; #73 branch |
| `0x29ea40` | ALE curve evaluation (to FUN_002dfc10; no Ghidra function): a curve holds its first key before that key (before-mode 0). | code | #73 branch play.md |
| `0x2a09f0` | ALE appearance setter of GeneralApp_AngleFadeOut (bit 1 of +0x40), the flag the demo calls PERP (0d645074); no Ghidra function. | code | #73 branch `ale_fx.rs` |
| `FUN_002a1a50` | Draws an appearance's particles as quads with the whole texture (a table at 0x443658 + n x 0x48), not as point sprites. | code (medium) | `play_fx.rs` |
| `0x2b26d0` | Converter for Freelancer's perp appearances: AngleFadeOut and facing 0 (their transform) or 2 (their velocity); no Ghidra function. | code | #73 branch `ale_fx.rs` |
| `FUN_002dfc10` | The curve lookup behind 0x29ea40. | code | #73 branch play.md |
| `0x3ddf80` | Pixel shader 0's definition: one combiner stage, texture x constant c0 for colour and alpha, then a final combiner that fogs. | code | squad.md § Memory chip; `play_dna.rs` |
| `0x403904` | ALE "GeneralApp_" parameter table (names at 0x403a58-0x403bc0): 0d645074 GeneralApp_AngleFadeOut, e3542300 GeneralApp_FacingType (1 camera-facing; 3 on exp-rocket-add, exp-rocket-shrap.rect and the Frag's exp-lrg-add). | code | #73 branch play.md, `ale_fx.rs` |
| `0x443658` | The particle quad table read by FUN_002a1a50 (n x 0x48). | code | `play_fx.rs` |

## Levels and triggers

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_00156390` | A game object's attribute parser (object vtable +0x14 by default): h_17a554bf into +0x70 bit 4; at 0x156666 inventory-drop (h_0f77963d, into +0xc4: the squad's memory chip); h_f2990a77 the lights. `play_dna.rs` calls it the character type's parser; it's every game object's, placements included. | code | xemu.md; `play_dna.rs` (`CHIP_AHEAD`) |
| `FUN_00160da0` | Type maps of [0x469980], keyed by objecttypes hashes (not by placements' `name=`); FUN_0022ca20 finds a decal type through it. | code, live | xemu.md |
| `FUN_00187fc0` | Parser of a debris entry `<h_19c8df19>`: a 16-byte record (archetype-name, h_1c1e17fe, h_1b6a0ede, h_f2e4e1a3). | code | scenery.md § Data; `bf/character.rs` (`Debris`) |
| `FUN_0018ba00` | objecttypes loader: a switch on hashed element names; reload-time lands in the weapon record's +0x288. | code | ghidra README; #73 spec §1 |
| `0x18f8b0` | objecttypes field-parser tables embedded in `.text`, 0x18f8b0-0x190fd0. | code | ghidra README |
| `FUN_00191170` | Parser of an effect object's `<h_fb0a5f1d><Damage>`: a 0x1c-byte record (amount, Type, h_ed582b3c, h_f724cb8c, duration, range, falloff). | code | scenery.md § Data; `bf/character.rs` (`AreaDamage`) |
| `FUN_00193420` | Liquid types (`<h_fa2f5452>`): liquid-type and three damage rates. | code | levels.md (Liquids aren't solid) |
| `FUN_001e1090` | A field parser that takes its arguments in EAX / ECX (read the asm). | code | ghidra README |
| `0x209250` | `FUN_00209250` in the docs (no Ghidra function): spawn-trigger parser (vtable +0x14). | code | xemu.md |
| `FUN_002099e0` | Spawn-trigger spawn: clones its template (+0xcc) at its spawn point (+0xb4) and keeps the copy at +0xc4. | code, live | xemu.md |
| `0x209ac0` | `FUN_00209ac0` in the docs (no Ghidra function): spawn-trigger actions (vtable +0x60). TRIG_ACT_OPERATE (1) while armed (+0xe4 bit 3) spawns at once or after h_eab6952f (+0xe0); TRIG_ACT_SPAWN_RESET (36) re-arms when h_0f95f576 (+0xe4 bit 2) is set. | code, live | xemu.md |
| `0x395838` | Spawn-trigger vtable. | code, live | xemu.md |
| `0x3d50f8` | Terrain vertex shader (NV2A): row x 65 + column on a 2 m grid from the first short, heights in 1/16 m; its constants are set at run time. | code | levels.md; `bf/character.rs` (`TERRAIN_VERTEX`) |
| `0x469948` | Player slots: [0x469948] + 0x22c + 4n, whose +0xc4 is a team object (not the character). | live | xemu.md |
| `0x469980` | Object type maps (FUN_00160da0), not the list of placed objects. | live | xemu.md |

## AI

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_0001a910` | AI; at 0x1aa6d an AI character with Tex's or Flint's skill on gets GOAL_WEAPON_SWITCH (45). | code | todo/35 leads §5 |
| `FUN_0001aeb0` | Movement noise EVT_SOUND_QUIET_UNIDENTIFIED (52): radius = gait factor (0.8 / 0.3 / 0.15) x type+0x21c x the skill's +0xe8 (Hawk's 0.25 + 0.75 v) x 15. | code | todo/35 leads §4.3 |
| `FUN_0001afe0` | AI update (brain vtable +4): runs the goal machine FUN_000377b0 only while the brain's mode (+0x58) is 0-5; mode 6 stands, not shooting (the trainer's freeze). | code, live | xemu.md |
| `FUN_00024f80` | AI attack check: at 0x2507a it holds fire while its own character's skill +0x108 is true (Hawk's: stealth on); at 0x252d0 it reads the skill again (not traced). | code | todo/35 leads §4.3, §5 |
| `FUN_000377b0` | AI goal machine. | code, live | xemu.md |
| `FUN_0004d490` | A noise for the AI to hear, from an owner over a radius: 75 m for a missile blast, the explosion type's +0x2c0 for a grenade. | code (AI hearing: inferred) | #73 spec §5; #73 branch |
| `FUN_00183e90` | GOAL_ text parser (table 0x3bcbb4). | code | this page |
| `FUN_00184ed0` | MSG_ / EVT_ / CBT_ text parser (table 0x3bdb9c). | code | this page |
| `0x3a4f04` | AI brain vtable (+4 the update FUN_0001afe0); the brain is at char+0x830, its goal at +0x44, its mode at +0x58. | code, live | xemu.md; todo/35 notes |
| `0x3bcbb4` | GOAL_ name table (records from 0x3bcbb0): 0 INVALID ... 134 ATTACK_BY_THREAT; 25 SPECIAL, 45 WEAPON_SWITCH, 57 MOTION_SPECIAL, 59 DODGE, 127 TOGGLE_COMBAT_SKILL. squad.md's "goals at 0x3bce78" is GOAL_DODGE's own record in this table. | code | squad.md § The squad's AI table; todo/35 leads §5, notes |
| `0x3bdb9c` | MSG_ / EVT_ / CBT_ name table, values 0-96: MSG_DAMAGE 1, EVT_DEAD 10 (at 0x3bdc14, squad.md's "events at"), EVT_SHOT_BLOCKED_BY_FRIEND 34, EVT_SOUND_QUIET_UNIDENTIFIED 52, CBT_DEAD 78 (0x4e), CBT_DAMAGED 81 (0x51), CBT_OUT_OF_AMMO 82 (0x52), CBT_OUT_OF_CHARGE 83 (0x53), EVT_CANT_ATTACK 96. | code | squad.md § The squad's AI table; todo/35 leads §4.3; this page |

## Camera

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_0006bdd0` | A decaying value on a camera (+0xdc): the shake (intensity, 0.5, time). | code (units: low confidence) | #73 spec §5 |
| `FUN_000da8f0` | Gives an object a `follow_cam_%X` camera (the characters a player can take control of, h_17a554bf). | code | xemu.md |
| `FUN_000db3e0` | Deletes that camera with the object. | code | xemu.md |
| `FUN_001249b0` | Leaves the scope: clears +0x7a8, with the zoom sound and the sway. A reload (FUN_00120d40) and an item use (FUN_00121170) call it first. | code (+0x7a8 as the scope: inferred) | play.md § Ammo, § Shots (Muzzle effects); `play_ammo.rs`; `play_shots.rs` |
| `FUN_00126320` | Called when the gun is raised (FUN_00123ea0): for the locally controlled character a camera step (FUN_0017c020 with 0.5). | guess | this page |
| `FUN_0018fa80` | objecttypes parser of a character type's camera block (h_0d41e5f1): seven offsets from +0x1b0 (walk, run, dash, ready, h_e42d50a5, dead, snipe). | code | squad.md § Death camera; `bf/character.rs` |
| `FUN_00228c70` | A missile blast's camera shake: its only callers are the missile code (0x227867 in FUN_00227760, 0x227b1d in FUN_002278e0). Each camera within 7 x the Damage radius gets clamp(Damage max x 0.00769, 0, 1) x f over 0.75 f s (FUN_0006bdd0), with f = 1 within 3.5 x the radius, then 1 - (d - 3.5 r) / 3.5 r, and nothing once f is 0.1 or less. The Therm Sweeper's 96 gives 0.74 x f. The #73 branch calls it every blast's shake. | code (units: low confidence) | #73 spec §5; #73 branch (not done) |

## Sound

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_000cbec0` | Weapon switch; at 0xcbfda the two hard-coded sounds ff820fda (as it starts) and e1e97460 (as it completes). | code, live | `play.rs` (`SWITCH_SOUNDS`) |
| `FUN_000ffb60` | Menu focus movement, the neighbours of 0x100250. | code | menu.md |
| `FUN_00100250` | Menu focus moving to the previous or next item: sends h_e29fd995 (-> h_ed2b5e99). Ghidra cut it to 5 bytes: read the asm. | code, live | menu.md; `play_menu.rs` |
| `FUN_001038e0` | A menu control stepping its value by one (the map carousel): sends h_ebf26601 (-> h_1dffc5a1), h_f06bc0ab when A selects, menu_error when it can't. A menu opening sends h_f97ceaf0 from around 0x10a9xx (not pinned). | code, live | menu.md; `play_menu.rs` |
| `FUN_00119020` | Footsteps; at 0x11918e the volume is scaled by the skill's +0xe8 (Hawk stealthed: 25%). | code | todo/35 leads §4.3 |
| `FUN_001372c0` | A check in the chatter gate (FUN_00140a60); not traced. | code (use only) | todo/35 spec §4 |
| `FUN_001374c0` | As FUN_001372c0. | code (use only) | todo/35 spec §4 |
| `FUN_001409a0` | A check in the chatter gate, with 0x1409c0; not traced. | code (use only) | todo/35 spec §4 |
| `FUN_00140a40` | The chatter gate's priority check (h_e146dc86, likely). | inferred | todo/35 spec §4 |
| `FUN_00140a60` | Says a chatter tag for a character (28 callers): a priority check (FUN_00140a40) and a time throttle on the line record (+0x50, FUN_00141020); how often a line is said isn't settled (h_ea21ae4b doesn't fit as a percentage). | code (partly) | todo/35 spec §4 |
| `FUN_00141020` | The time since the last line of that priority. | code | todo/35 spec §4 |

## Other

| Address | What it does | How sure | Where |
|---|---|---|---|
| `FUN_00012000` | Sends a message (an id from 0x3bdb9c) with a data pointer to the object in EAX, through FUN_000e4190: MSG_DAMAGE from the damage handler, CBT_DAMAGED from take-damage. | code | this page |
| `FUN_000120f0` | Dereferences an object handle (196 callers): the brain's character, for one. | code | todo/35 leads §4.3 |
| `FUN_000f3fb0` | Profile names "VENGAR" (with FUN_00101810), "GARNER", "HVYMTL", "NINJA": cheat or unlock names. | guess | todo/35 leads (found, not in scope) |
| `FUN_00101810` | See FUN_000f3fb0. | guess | todo/35 leads (found, not in scope) |
| `FUN_00121170` | Uses the selected item: out of the scope first (FUN_001249b0) unless the item's +0x174 is 6, then by its use type (IOU_, 0x3be61c). IOU_RELOAD (4) starts a reload (FUN_00120d40); throws, placements and melee go through FUN_0014abc0 and the skill's +0x104 (at 0x12123b, Hawk's stamina cost). It runs while a use is pending (char+0x6a0, FUN_0011aa20). `play_ammo.rs` reads its IOU_RELOAD case as the reload at the trigger's release (measured); that path isn't read. | code (the release link: inferred) | `play_ammo.rs`; todo/35 leads §4.3 |
| `0x146f40` | Power-up handler (no Ghidra function): adds the item type's +0x18 (health, h_0a811e94) if below max and +0x1c (stamina, h_11884f2e), each clamped; STAMINA POWER gives +50. | code, live (#35 take20) | todo/35 leads §6, spec §11 |
| `FUN_001478d0` | Sets the character's damage factor +0x754 to the item's +0x18 (a damage power-up; Brutal Damage is likely). | code (which power-up: inferred) | #73 spec §6; #73 branch play.md |
| `0x147900` | `FUN_00147900` in the #73 spec (no Ghidra function): sets +0x754 back to 1. | code | #73 spec §6; #73 branch play.md |
| `0x1479f0` | Item code that sets vision mode 1 (an item's, not Vengar's). | code | todo/35 leads §4.2 |
| `FUN_0014cc30` | The item at (category, index) in an item holder's store (+0x1b4: a vector per category at +0xc + 0x10 x category); the reserve is (1, ammo-type). | code, live | `play_ammo.rs`; xemu.md (items) |
| `FUN_001597d0` | Object position setter (item vtable +0x38): changes the data, but the drawn memory chip stays put. | code, live | todo/43 notes |
| `FUN_001e7b30` | Reads the pad bindings: 16-byte records {pad input, VB, mode, threshold}, eight blocks, from game-options' `<action>`. | code, live | xemu.md |
| `FUN_001e8160` | Stores the name `common/debug-config-xbox.xmb` in the `std::string` at 0x400438; the file is never loaded. | code | xemu.md |
| `FUN_001fe3d0` | Sends an event from an object (a missile's or a grenade's going off; 11 callers). | code (meaning inferred) | #73 spec §5; #73 branch |
| `FUN_00233b90` | The game's random numbers: x x 0x19660d + 0x3c6ef35f, the top 23 bits as a float in [0, 1). The same step is inlined on the globals 0x46cfd8 and 0x46cfdc. | code | `play_shots.rs` (`random`) |
| `FUN_00238d00` | Fills a static table that holds motion-type numbers; not the movement code. | code | ghidra README |
| `FUN_00254c60` | BXML reader (with FUN_00255330 and FUN_00281d30). | code | `xmb_tool.py` |
| `FUN_00255330` | See FUN_00254c60. | code | `xmb_tool.py` |
| `FUN_00281bf0` | BXML token-stream decompressor: block LZ over u16 words (with FUN_00281cc0, FUN_00281d30). | code | `bf/bxml.rs`; `xmb_tool.py` |
| `FUN_00281cc0` | See FUN_00281bf0. | code | `xmb_tool.py` |
| `FUN_00281d30` | See FUN_00281bf0. | code | `bf/bxml.rs`; `xmb_tool.py` |
| `FUN_002930e0` | A Lua bytecode interpreter's opcode switch. | code | ghidra README |
| `FUN_002c72b0` | Ipion physics (`insert_active_float`). | code | ghidra README |
| `FUN_00311070` | Static destructor of the debug-config name string (0x400438). | code | xemu.md |
| `0x39b508` | Item object vtable (the memory chip, STAMINA POWER): +0x38 FUN_001597d0, +0x140 FUN_0014a410, +0x144 FUN_00157260. | code, live | todo/43, todo/35 notes |
| `0x3ba968` | The fixed 256-entry normal palette of P8 bump maps (`.data`), copied into a D3D palette by FUN_00285780. | code | `tex_tool.py` |
| `0x3bc150` | The name hash's CRC table: CRC32's polynomial, with the top byte of most entries different. | code | `bf/hash.rs`; `xmb_tool.py` |
| `0x3be3cc` | IFSET_ name table (parser FUN_00185620): 5 GENERIC_HEALING, 8 PROXIMITY_EXPLOSIVE, 13 ROLLING_BOMB, 14 AMMO_BOX, 16 MINIGUN, 18 POWERUP_POWER_CRYSTAL (the memory chip's function-type), 19 POWERUP_MEDKIT, 20 POWERUP_BRUTAL_DAMAGE, 21 POWERUP_FORCE_SHIELD, 22 POWERUP_SQUAD_REGENERATOR, 23 POWERUP_PAIN_BROADCASTER, 24 SHIELD_BATTERY. | code | pickups.md; `bf/character.rs`; `play_ammo.rs` |
| `0x3be61c` | IOU_ name table (parser at 0x185a30, not split out): 0 USE_IN_HAND, 1 ATTACH_TO_FACE, 2 PLACE_ON_GROUND, 3 THROW_TO_USE, 4 RELOAD, 5 EAT, 6 JUST_USE, 7 ATTACH_TO_EQUIP_SLOT, 8 SKILL_MELEE_ATTACK, 9 MELEE_ATTACK_RANDOM, 10-12 MELEE_ATTACK_A-C. | code | `bf/weapon.rs` (`H_USE_TYPE`); todo/35 leads §4.3 |
| `0x3bf39c` | VB_ name table (records from 0x3bf398, which xemu.md cites; parser FUN_00186c10): 0 NONE, 9 SPECIAL_ABILITY, 43-60 the VB_DEBUG_ actions (no reader), 63 USE_HEALTHPACK. | code, live | xemu.md; todo/35 leads §3 |
| `0x400438` | The `std::string` holding the debug-config file's name. | code | xemu.md |
| `0x46cfc8` | BXML schema global: the hash of the `stringid` type (values stored as a 4-byte name hash). | code | `xmb_tool.py` |
| `0x46cfcc` | BXML schema global: the hash of the `wstring` type. | code | `xmb_tool.py` |
| `0x46cfd4` | BXML schema global: the index-string type (values stored as a varint, index + 1). | code | `xmb_tool.py` |
| `0x46cfd8` | Random state word stepped by the damage roll (FUN_0022c940) and the blast delay (FUN_0022bfc0); 0x46cfdc is the decal pick's (FUN_0022ca20). | code | this page |

### Constants

Values read from `.rdata` / `.data`; the reader is the function that loads them.

| Address | Value | Read by, for | Where |
|---|---|---|---|
| `0x39a848` | 3.0 | FUN_000c72b0: the skill name's fade rate (0xc7473) | todo/35 spec §5.5 |
| `0x3a4f74` | 0.5 | FUN_00226cc0: the cosine past which an aim point is given up (60 degrees) | #73 branch `play_missile.rs` |
| `0x3a4f78` | 1.0 | 0x125600: the 1 in 1 -+ 0.1n; 0x10e3d0 returns it; FUN_0022f0e0 and FUN_0022dc00: 1 / rate; FUN_00224a90: the push needs the type's +0x234 at 1 or more | todo/damage-factor; this page |
| `0x3a4f88` | 0.0 | FUN_0010e3e0 (stamina at 0), 0x22d0b0 (radius not 0) | this page |
| `0x3a4f90` | 0.2 | 0x10e3b0: the default skill +0x58 | this page |
| `0x3a4f94` | 0.05 | 0x22d0b0: the Damage radius between bullet holes and blast scorches | this page |
| `0x3a4f98` | 0.01 | FUN_0010e7a0: volume 100 to 1.0 | todo/35 spec §3 |
| `0x3a4fb4` | 1.5 | 0x147430: the Sentry filter's height | `play_sentry.rs` |
| `0x3a4fd0` | 4.0 | FUN_0011a840: the x4 refill | todo/35 leads §2, spec §1.2 |
| `0x3a4fd4` | 5.0 | FUN_0013d030 (Hawk's x5 with the Power Blade), 0x13d340 (an item's -5) | todo/35 leads §4.3 |
| `0x3a4fe0` | pi / 180 | 0x12ed40: h_1182c05d to radians | todo/damage-factor |
| `0x3a4ff0` | 0.1 | FUN_0010e470 (the 10% rule), 0x147620 (the Sentry check's jitter), 0x125600 (0.1n) | todo/35 leads §2; `play_sentry.rs`; todo/damage-factor |
| `0x3a5014` | 0.8 | Flint's skill +0x58 | todo/35 leads §4.4 |
| `0x3a5028` | 0.25 | FUN_0011a840: the refill in MT_DASH | todo/35 leads §2, spec §1.2 |
| `0x3a50b8` | 0.9 | 0x1332a0: the stick past which it sends CT_DASH | todo/35 leads §4.2 |
| `0x3a51e4` | 0.15 | 0x147620: the Sentry's check interval | `play_sentry.rs` |
| `0x3a51f0` | 2.5 (double) | FUN_00212430: BF_InvisMaterial's sine | todo/35 leads §4.3 |
| `0x3a5208` | 0.55 | Flint's skill +0x38 while on | todo/35 leads §4.4 |
| `0x3a52e0` | 0.017444 | FUN_002229b0, FUN_0022e1d0: degrees to radians | `play_shots.rs` (`DEG`) |
| `0x3a52f0` | 0.09 | 0x147430: 0.3 m squared | `play_sentry.rs` |
| `0x3a52f4` | -1.5 | 0x147430: the Sentry filter's lower height | `play_sentry.rs` |
| `0x3bc6f0` | 1.5 | FUN_00224a90: the push on loose bodies | scenery.md (debris push) |
| `0x3c05fc` | 4.0 | FUN_00123f60: the ready hold | `play_shots.rs` (`READY_HOLD`) |
| `0x406f94` | 0.5 | FUN_002266d0: the cut across the steering way | #73 spec §3; #73 branch |
| `0x40706c` | 0.5 | FUN_00212430: a ramp | todo/35 leads §4.3 |
| `0x407070` | 0.5 | FUN_00212430: a ramp | todo/35 leads §4.3 |
| `0x407078` | 0.1 | FUN_00212430: the sine's factor | todo/35 leads §4.3 |
