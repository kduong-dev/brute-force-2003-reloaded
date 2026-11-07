*Part of [nav/](README.md), the former page `nav.md`.*

## The own point (FUN_0014f860)

A character's world point (char+0x24c) is where its routes start and where routes to it end.
The character update keeps it every frame once it's wanted (+0x250 bit 0: the first time a
place of the character is made, at a spawn, at a route's start), for a living character with a
brain, outside a deathmatch arena:

* no point yet, a forced refresh due (+0x250 bit 6 and 2 s), or its point over 20 m away (3D):
  FUN_0014f7a0: kept within 5 m, else the nearest valid point of all (FUN_000ab260, 3D, any kind);
* else, while it isn't in a jump (FUN_00126a90: MT_JUMP_CROUCH, _FALL, _LAND, _LAUNCH): the
  nearest to its position of its point and the valid points one link on (two every eighth frame,
  by a frame count in +0x250 bits 3-5; FUN_0004bf30). When that's another point, or the point's
  height is off (more than 1.5 m under the character's position less 0.25, or 0.5 m over), and
  0.25 s have passed since the last deeper search, the deeper search decides: 6 links deep from
  the current point, the nearest (FUN_0014f5c0); after 1 s with the height off by over 2.5 /
  1.5 m, 10 links deep and the first of the 6 nearest the sight ray reaches. It keeps the point
  when it finds none. Within the 0.25 s the near walk's pick is taken as it is.

The position is the character's transform's (vtable +0x40), read as its hips, the skeleton's
origin (FUN_000bf2f0's row calls that position the node, the hips; inferred): bf_play's
`Player::position`, which stands 1.05 m over the feet. `OwnPoint` carries the point, its flags
and its three timers; nothing keeps one per character yet (#335 does, for the squad and its
leader).
