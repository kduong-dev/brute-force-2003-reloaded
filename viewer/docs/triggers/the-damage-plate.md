*Part of [triggers/](README.md), the former page `triggers.md`.*

## The damage plate

#228, `play_damage_plate.rs` (a child of play_triggers.rs: its detector is the runtime's). From
the code: the class h_130818c0 (vtable 0x396530; its parser 0x1fa740, constructor 0x1fa680,
update 0x1fa7e0) and FUN_001f8510, FUN_001f8420.

* **A proximity detector**: its sphere of `radius` (5 by the constructor: the tutorial's plate's
  `size` 4 2 4 isn't read, a proximity detector's parser takes `radius` alone), its occupants and
  their ENTER / EXIT as any detector's ("Detectors").
* **While it's enabled** each occupant (from its first frame in the sphere until 0.2 s after it
  leaves) that's a living character in the world takes `amount` (h_18499bb9, +0x120) x the
  falloff (h_e022d922, +0x124, FUN_001f8420: 0 all of it; 1 (1 - d / r) and 2 (1 - d / r)^2;
  3 all of it to r / 2, then 1 - 2 (d / r - 0.5); none beyond r) x dt each frame, of its
  `damage-type` (+0x128), from the plate's middle, through the character's damage handler
  (0x125600: its difficulty step and its factor for the type; nobody is hurt during a cutscene).
  A negative amount heals each occupant by it x dt, up to the most, with no falloff and no test
  (FUN_001f8510's other branch).
* **Not a weapon's hit**: no hit part, no damage-multiplier, no push (the record's is the
  constructor's 1.0), and no blood (take15 shows none). A plate's kill is thrown **straight up**
  (#366, from the code, not filmed): the record has no body and no way, so the hit request is
  body-less with a zero way and the up-kick merges into it (0.5 x (2 x the frame's share + 1) x
  37 N s a body, at most 75: the tutorial's 20 HP/s plate on Tex, 0.4 HP a frame, is a hop of
  ~6 m/s and 1.4 m; ragdoll.md, play_damage_plate.rs). The first hit on a character since the plate was enabled goes through the demo's `hurt`;
  the rest come off its hit points directly, unless one would kill it. Each asks its hurt line
  of the chatter gate (a bare record: the line by the hit points; [chatter.md](../chatter.md)). Enabled
  again, it starts again (`BF_TRIGGER_FIRE=trigger_damage_plate524,USER1,2` after its second:
  the first hit through `hurt` once more).
* **The tutorial's** trigger_damage_plate524 at (27.9, 17.4, 67.6): BALLISTIC 20 a second,
  falloff 0, squad types. The show_patrol_2 script's 103s at 0.108 s start timer528 (1 s) and
  enable it; the timer's pulse disables it. Tex, warped to direction_point2 2.6 m from its middle,
  takes 20 x 0.6 (STANDARD's step on the squad) = 12 a second for that second: take15's 115 ->
  102.7.

`BF_COMBAT_LOG=1` prints each hit from a plate (this frame's, the total so far and the hit
points left), the first with its factors.
