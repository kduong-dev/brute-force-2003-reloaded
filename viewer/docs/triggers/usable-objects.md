*Part of [triggers/](README.md), the former page `triggers.md`.*

## Usable objects

The panels and consoles a player uses with X (play_usables.rs, #262; the tracker, the prompt and
the hold are in [maps.md](../maps/gates-wall-panels-and-consoles.md), "Gates, their wall panels and consoles") send their USE_OBJECT
as an object event (above): to the routers, gamevar-, score-, radar-, sound- and script-triggers
and the property trigger that list them. A router that hears several of them feeds one chain, so
e34's two consoles, and the tutorial's two panels, share it.

* **The property trigger** (h_14cccfaa, vtable 0x395780, handler 0x20b730, parser FUN_0020b680;
  `Triggers::property`): disabled, the base actions; enabled, an OPERATE entry that answers gives
  its **first object** the reticule-action in its h_e92285dc (+0x40; the object's byte +0x79)
  and is counted, the object found or not; the others are the base's. The triggers' other
  attributes (h_e8ce605a, the object's +0x71 bit 6; h_06761f1c and h_ea199c91, a special use's
  hazard, +0x110 / +0x114) are read (`Class::Property`) and not kept: no object the demo has needs
  them. The event can come from the object itself (it lists it among its objects) or from a router
  that sends to it, which is how both e34 consoles go dead on either's use. All 52 in the data set
  0: the object is no longer a candidate for the use tracker. Their signals: 14 USE_OBJECT (the
  consoles' and panels' own use: e34, e23, e16, e01, m02_b, m04_c, m08_a, m08_b, m14_d, sdm_e34 and
  the tutorial), 13 FIRST_UPDATE (e16's console h_133a9b4c and m08_a's panel h_00c61205 start dead),
  8 TIMER_PULSE (e16's three, from the panel router h_ff946049, whose out-signal 8 stands for "go";
  e13's), 6 DESTROYED (m09's eight consoles, e05's), 25 FINISHED (e13's), -1 (m08_a's, no object).
  The demo applies each OPERATE to the object of that name among the level's usables
  (`take_reticules`, play_usables.rs); an object that isn't one is no use target anyway.
* **A console's chain**, e34's h_09da26aa (`BF_TRIGGER_LOG`): USE_OBJECT [tex] to routers h_14eeda94,
  h_0e1c2378, h_1bbd1197 and the property trigger h_fccb606d; h_14eeda94 to the timer h_e4c2a242
  (starts, 5 s), the anim-trigger h_e5f08fe4 (ANIM_START to no one, one leaf's clip; spent after
  its one entry), four waypoint triggers (door points 32-35 valid), the gate's sound
  h_f396d45f (metal_door_5, 8.52 s, at the gate) and the voice line h_f10a4b7e (PLAY_DIALOG,
  dxud_bratton_us00565, 1.73 s: it waits on the dialog queue while another line plays); h_1bbd1197
  to both consoles' property triggers (h_eec50141: the other console's reticule-action to 0,
  h_fccb606d: its own), two terminal_07 sounds (2.76 s, one at the console) and h_14eeda94 once more
  (every entry behind it is once: nothing). 5 s later the timer's TIMER_PULSE: anim-trigger
  h_e446c3cf (the other leaf's clip), the gate's sound again (h_fa91e4c9), and the voice line's
  FINISHED goes on to h_ea031ac4 (us00566).
