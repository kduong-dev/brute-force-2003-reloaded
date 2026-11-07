*Part of [triggers/](README.md), the former page `triggers.md`.*

## Radar-triggers

#228, `play_radar.rs` (a child of play_triggers.rs: its class is the runtime's). From the code:
the radar-trigger's vtable 0x395168 (its parser 0x2071a0, handler 0x207230, constructor 0x2073b0)
and FUN_0015b310, which the handler calls for each object.

* **Attributes** (0x2071a0): `radar-contact` (+0x40, the RCONTACT_ names at 0x3beaac: 0 NONE,
  1 ALIGNMENT, 2 OBJECTIVE, 3 OBJECTIVE_NO_CURSOR, 4 OBJECTIVE_SECONDARY, 5 INVISIBLE,
  6 BOSS_ALIGNMENT, 7 IMPORTANT_PICKUP), h_e2145a6c its blink time (+0x44, s) and h_12b33046
  (+0x48) the radar's sound, true unless the data says not (the constructor's).
* **OPERATE** (enabled; disabled it runs its base actions only): each of its objects that's in
  the world (a listed spawn-trigger's copy; any other trigger is skipped) takes the contact
  through FUN_0015b310: the object's +0x7a is the value, kept in the world's contact list while
  it's 1, 2, 3, 6 or 7, and its +0x10c the larger of +0x10c and the time, which the object's
  update (0x157c40) counts down by dt to 0. With h_12b33046 and a contact of 2 or 3 the game
  options' radar sound (`<radar>` h_ee13e943: h_f674fc4f, dxud_ai_00008) goes on the world's
  dialog queue (FUN_00179d60, flag 0) when an object in the world took the contact (the flag its
  loop sets, 0x20730a-0x20733a). Any other action is the base's. The entry is counted after.
* **The radar** (play_hud.rs; [play.md](../play/radar.md), "Radar") draws the objects whose contact is
  2, 3 or 7 (FUN_000ae580 hands those to FUN_000aece0) as yellow dots, blinking while their
  +0x10c is above 0, and as "L" markers on its rim beyond its range.
* **The tutorial's 18**: nine pairs along the route (leads §7.9), each a set (contact 3, time 2,
  once, h_12b33046 false: no sound) on the next route object (h_fe303e53 at (25.4, 34.6) on
  FIRST_UPDATE, h_e89e2471 at (56.8, -0.5) as the first patrol's circle h_ed8c0a43 is entered,
  ...) and a clear of the last (contact 0, time 0, h_12b33046 true, but no sound for a 0). The
  circles and plates that drive them also play the save points' chime, sound-triggers of
  dxud_ai_00008 (trigger_sound452 ..., 1.51 s, 2D: "Sound- and dialog-triggers"). What a save
  point saves (the restart there) is #27's.

`BF_RADAR_LOG=1` logs each contact set on an object (its name, type, place, value and time) and
each name a radar-trigger sets that's in the world under no object.
