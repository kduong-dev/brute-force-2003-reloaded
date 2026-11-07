*Part of [grenades/](README.md), the former page `grenades.md`.*

* **Damage**: everyone within the radius takes Damage max, falling to nothing at the radius;
  the thrower takes 0.2 x a roll of Damage min..max anywhere inside it. That rule is the
  demo's, fitted to the Frag recording (six blasts, near and far: 12.5-13.3 HP of Tex's 115
  each, ~11%, no falloff). The game's rule is read from the code (see **Sentry**,
  FUN_0022c4d0) but not implemented (#112): half the roll x0.7 on the thrower's own squad x0.6,
  the squad's difficulty step on STANDARD, is 0.21 of a roll, the recording's ~11%. A core blast
  (`h_011cb154` true: the Roller's, the Thermo's) gives the whole roll within half the radius, so
  its thrower takes 0.42 of a roll there (FUN_0004b9c0 has no self exclusion): #79's take10 Tex
  -50.6 at 2.46 m from a Roller, #82's take04 Brutus -13.9 at 1.43 m from a Thermo; the demo's 0.2
  gives a Roller's thrower 23-31 (#112's, not changed by #79 / #82).
  **The difficulty step** (#127, play.md **Damage**): every blast's hit on a character takes the
  victim's step (STANDARD on the missions and the test map: enemies x1.4, the squad x0.6; none
  on the multiplayer maps, as read live in a match, #129) and its damage-type factor. The
  recordings were the campaign's on STANDARD, so the amounts fitted to the squad there (this
  thrower's 0.2, the Sonic's and the Energy's thrower shares, the Gas's rate) already hold its
  x0.6: they're divided by it before the step, which gives them back as measured on STANDARD
  (12.6 of Tex's 115 for this Frag test) and scales them with the difficulty; on sdm_e34, with
  no step, the thrower takes x1 / 0.6 of the recording's share (21.0 for that roll; the game's
  rule there: half x0.7 = 0.35 of a roll, 22.1). Measured on the thrower in its own blast, they hold
  FUN_0004b9c0's x0.7 too, so none is added. The others in reach take the rule above x their
  step, with no x0.7 (the demo's rule; the game's, with it, is #112's). No damage from a
  blast without any (the Light); one whose damage is dealt over time (h_04ea9251 > 0: the Gas,
  whose recording shows none at once) leaves a poison cloud instead (see **The Gas's cloud**
  below). The Energy's damage comes with its bolts instead (see **Energy grenade**), the
  Sonic's with its ring (see **Sonic grenade**).
* **Damage types.** Each character's combat-target lists factors per damage-type
  (`<h_142be76f><h_1d403525 Type h_04653d86>`, read into its character type's
  `vulnerabilities`, #128, and taken by play_damage.rs's `vulnerability`; the shield's own list
  is empty): Flint takes type 4 (the Gas) x0.05 and type 6 (the Energy)
  x2; Brutus, Hawk and Tex list none; the mutants 3 x0, 5 x0.5-0.75, 1 x0.8. Blasts, the gas
  and the player's shots on squadmates (their weapon's damage-type) are multiplied by them.
  `BF_DUMP_HITPOINTS=1` (with `BF_MAP`) prints them.
* **Damage and red tint land 0.1 s after the blast** (the recordings: the tint starts 3 game
  frames after the flash, Frag 693 -> 699, Sonic 229-231 -> 235). Hurt by it, the 3D
  picture's green and blue follow the recording's factors per game frame: 0.20, 0.29, 0.39,
  0.51, 0.61, 0.69, 0.78, 0.92, 1 (the reference agent's measurement), red and the HUD
  untouched, as strong near or far. (A 15 fps capture samples every second game frame: 0.20,
  0.39, 0.61, 0.78, 1 - measured on unlit ground 0.19-0.24, 0.40-0.43, 0.60-0.62, 0.81-0.82,
  1.) A quad in front of the camera, multiplied into the
  picture; the picture blends in linear light, so each factor is the linear one that scales
  a stored value of 0.4 (the ground's ~0.35-0.45) by it.
* **HUD**: the item box shows the selected type's icon (64 units square at the box's corner, as
  every item's), label and count (from 2 up), the game's widget since #196 ([play.md](../play/item-box-and-item-list.md),
  **The item box and the item list**). The meter's fill is drawn as (226, 148,
  54) at 0.65: what the console's stored-value blend of (255, 160, 53) at 0.65 gives over the
  recording's ground (63, 63, 55), since the UI blends in linear light. The charge sound is
  queued on the frame the meter first shows (`BF_SOUND_LOG` and the capture agree). The fade's
  alpha is drawn so its strength over take01's ground (37, 27, 20) is the game's stored-value
  one (`METER_GROUND`, `METER_FULL`). When the selected type runs out, the box is blank a frame
  and then shows the first stack in the store's order, whatever its category (FUN_0014db60; the
  Sentry recording: the last Sentry set down, the box shows Frag 10; #196 take09).
