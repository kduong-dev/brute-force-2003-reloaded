*Part of [triggers/](README.md), the former page `triggers.md`.*

## Gamevar-triggers

A gamevar-trigger (#221; vtable 0x3960a0, parser 0x1ffac0, handler 0x1ffc40) has an operation,
h_015914ab, and three operands as it keeps them: `out`, A (`<h_1554ca7a>`) and B
(`<h_0e5d9bc0>`) (FUN_0018cbe0: each a kind h_18bbf97f and an invert h_1fe1ecd1).

* **An operand** (FUN_001ffb80): kind 0 a game flag by its path (`gameflag`, read as a float; 0
  without it), 1 its `constant`, 2 a random float in [0, 1) x the constant (FUN_00233b90), 3 the
  event's data (its 32 bits read as a float), 4 floor(random x the constant); inverted, 1 - it.
* **OPERATE** works out the operation (the jump table 0x200010): 0 A == B (|A - B| < 1e-5),
  1 A < B, 2 A <= B, 3 A > B, 4 A >= B (1 or 0), 5 A, 6 the out flag's default, 7 A + B, 8 A - B,
  9 A x B, 10 A / B (FLT_MAX for B 0). The entry is counted then (an entry that disables the
  trigger silences what follows), the result turned over if `out` is inverted, and written: to
  `out`'s flag (kind 0, FUN_00077eb0), kept as `out`'s constant (kind 1), or neither (the other
  kinds). VALUE_CHANGED (41) goes to the targets when the result isn't the value it replaced
  (always for the kinds that keep nothing, and for a flag the tree lacks), then TRUE when the
  result isn't 0 (|r| >= 1e-6), else FALSE; both carry the result as their data and the event's
  subject.
* **On a multiplayer map** the gamevars have no flags: a flag operand reads 0 and a write does
  nothing (VALUE_CHANGED still goes out, as for a flag the tree lacks), and the score-triggers
  pay no cash, so a match can't change the campaign's flags or cash for a later mission in the
  same run. The data doesn't need it: no multiplayer map's gamevar names a flag (the mp maps'
  4 to 8 each work on constants, the sdm_ maps have none; the `missions/h_e466ad5c/..` writes
  are e34's, a mission's). That a match has none of the campaign's flags is inferred: the
  session's difficulty getter reads 0 in a split-screen match (live, #129), which it does only
  without a tree or without the flag (0x7c720), where the default flags would give -4.
* The data uses ops 0 (178), 3 (4), 5 (309) and 7 (5); operand kinds 0, 1, 3 and 4. The
  tutorial's 15: four ops 5 (the objectives' `state`: 0 at the first update, h_ed0c1544 and
  h_e0a5fecf; 1 on completion, h_11b9e83d and h_089d8e5f); ten member tests
  `<member>/h_03761692 == 1` (four at the first update: the farcast flashes, the co-op hint's
  calculator496, the V-squad's calculator501; two on the V-squad's plate; four at the end pad, the
  farcast-out flashes); and `missions/<m14>/h_08bbbbc1/state == 1` (a reward of the finished
  campaign: false).
