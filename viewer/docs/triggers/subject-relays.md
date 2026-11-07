*Part of [triggers/](README.md), the former page `triggers.md`.*

## Subject relays

#278. The class h_0cb602d0 (its name isn't in the data; vtable 0x395308, parser FUN_002012c0,
handler 0x201310). The multiplayer maps have 716 (mp1-mp4, mp6-mp8 and the seven sdm_ maps):
* 645 keep their binding (h_f49c30af) and send to timers only: 644 hear a spawn-trigger whose
  template is a pickup (394 inventory-objects, 175 weapon-objects, 75 item spawns
  `<h_1d1d4f05>`), the pickups' respawns; one in mp7 nothing sends to;
* 39 keep it and send to routers: mp1-mp8's power-up spot pickers;
* 32 don't keep it: the teleporter pads (plate -> relay -> warp).

The campaign's 122 are its teleporter pads and its AI and destroyer hand-offs; with m04_c's plate
relay to a timer, the levels have 646 relays that keep their binding and send to timers only.

* **What it does** (0x201310). Disabled, its base actions. Enabled, it doesn't read its entries:
  an event with a subject binds that subject to each of its targets (FUN_001fe6c0: the target's
  first object becomes the subject, and the subject hears the target from now on, the old first
  object no longer), goes on to the targets (FUN_001ff0a0), and unless its h_f49c30af is set the
  binding is undone after (FUN_001fea10). An event without a subject goes nowhere.
* **The pickups' respawn.** Each pickup spawn-trigger of a multiplayer map sends its OPERATED (the
  copy as the subject) to a relay with h_f49c30af, whose target is a timer: the timer now hears the
  copy, so the copy's PICKUP_OBJECT, as it's taken, starts the timer (its OPERATE), and its pulse
  (TIMER_PULSE) goes back to the spawn-trigger: OPERATE (a new copy, whose OPERATED binds the
  timer to it) and SPAWN_RESET (armed for the next), and to a second spawn-trigger beside most,
  whose template is the spawn flash's effect object (h_117c1805). The timers stop and reload at
  their pulse (h_f92b0b15). sdm_e34: the STAMINA POWER's h_f069b2c2 -> relay h_f106c777 -> timer
  h_0021e47f, 30 s (#35 take20: back ~31 s after it was taken); its medkits 10 or 45 s, its guns
  20 or 45 s, its ammo and grenades 10 or 20 s, its other power-ups 30, 165 and 180 s. mp1's
  power-ups spawn only on a pulse, through routers and gamevars that pick one of three spots.
* **The teleporter pads.** A plate's ENTER_DETECTOR (the occupant the subject) through a relay
  without h_f49c30af to a warp-trigger: the warp moves its first object, the occupant, and the
  binding is undone. The warp takes the objects it had at its OPERATE (`Act::Warp`), as the game's
  warps in its handler. It finds a squad member by the name the triggers know it by (a mission's:
  "tex_1" ...); the multiplayer maps' players have none, so their pads don't warp them yet. The
  pads themselves are #236's (not checked against footage).
