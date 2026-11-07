*Part of [skills/](README.md), the former page `skills.md`.*

## Sounds and chatter

* On and off (FUN_0010e470 / FUN_0010e3e0) play the type's `start-skill` / `end-skill` sound
  (Tex ed7492f4 / e00d91fa, Brutus ee753485 / 190cd1f6, Hawk fa7509ca / f57b8dca, Flint 07168977
  / 1656217b) at full volume, in 2D, **for the controlled character only** (FUN_0010e7a0 asks
  FUN_00112790; take18: an AI Hawk's switches are silent). At the switch, not the press. The off
  doesn't cut the start sound (take08: Brutus's 6.1 s rumble plays on).
* Each switch sends a chatter tag (on h_1c0514bd; off with stamina left h_09087c9c; off at 0
  h_ee0a5cbc, the stamina tested at 0x10e40d; leads.md had the two off tags swapped) through the
  game's chatter gate, FUN_00140a60, and its one queue, as every line does since #155
  ([chatter.md](../chatter.md): the checks in the code's order, the queue, the voice). Their blocks
  are the same for the four: `probability` 50, `replay-delay` 10 / 20 / 30 s, `priority` 1 / 1 /
  5, a 5 s wait. What that makes of the skill lines:
  * the replay delay holds a character's line, not another's (Hawk's doesn't hold Tex's);
  * a squadmate more than 15 m from the player never says the on and off lines (priority 1, a
    step less: 0, refused); the player's own lines are at 0 m;
  * the on line's condition kind is the default (0): any variant. The off line with stamina left
    tests char+0x5a8, which only ever holds 0 in default.xbe: Tex's and Brutus's variants need
    2..100, Hawk's 1..100, so only Flint (no ranges) says it. The line at 0 tests FUN_0001b460,
    read as the enemies the character is fighting (inferred): Hawk's variants need 1..100, so
    she says it only in a fight (the demo's count is a guess, chatter.md);
  * `BF_SKILL_CHATTER=<percent>` sets their chance;
  * a dead speaker is silent (FUN_0013fab0): the line of the death's off isn't heard;
  * the line goes in the frame of the switch, at once with nothing of its priority playing (the
    code has no delay of its own, and the samples no silence before the voice, 0-0.07 s); the
    takes' 0.03-0.5 s from the switch to the voice isn't explained (a wait behind another line,
    inferred).
* What the takes heard fits: the on line 13 times in 26 for Tex, Hawk and Flint (50%); the off
  line 3 in 6 for Flint and never for the others (0 of 17); the line at 0 never for Hawk (out of a
  fight). Brutus's on line was never heard (0 of 9): the code doesn't explain it; his 6.1 s start
  rumble may have hidden it (answer-120-chatter 1.6). Through the gate (#155: 26 of Tex's
  switches on the test map, `BF_TEST_SKILL` every 14 s) the on line was said 14 times and the
  off line with stamina left never.
