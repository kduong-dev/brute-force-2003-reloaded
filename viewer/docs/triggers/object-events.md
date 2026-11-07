*Part of [triggers/](README.md), the former page `triggers.md`.*

## Object events

* **DESTROYED**: a character with a trigger name dies (a spawn copy: its spawn-trigger's
  listeners hear it; the squad started at its spawns counts as those copies); a named placed
  object with hit points that a trigger listens to dies (play_scenery.rs, `Arena::is_dead`):
  one with a debris list breaks, one without stays as it was (#263: e23's door h_054428e4 opens
  on its own DESTROYED, shot for its 40 HP, and so does m08_b's gate h_f4d77cfd); a level's own Sentry mine goes off
  (#228, play_sentry.rs: 0x149520's send at 0x1496fc; the tutorial's calculator471 counts the
  mine field's three). Its id 0 and data -1 are the game's
  (the death's send: the object's +0x144(1), FUN_00157260, #221); the demo's subject is the dead
  object, the game's a character from the dead one's (see "Not done").
* **USE_OBJECT**: a wall panel used (play_usables.rs): its signal h_eaf8a35b and data h_f1660a0d
  (0x163060), or a console used (#262): the game object's own use, {USE_OBJECT, iD 0, data -1}
  (FUN_00158140); the user as the subject either way. See "Usable objects".
* **PICKUP_OBJECT**: a placed pickup taken (play_pickups.rs): since #228 the tutorial's DNA
  canister h_ef649b0f, whose PICKUP_OBJECT starts timer467 (4 s) and then Bratton's line
  ([pickups.md](../pickups/dna-canisters.md), "DNA canisters"); since #17 a lying gun taken by the swap or walked
  over (the tutorial's L-Shot-50: the sniper hint); since #278 a spawn-trigger's pickup copy, to
  the timer its relay bound to it ("Subject relays"). The Sentry mines are armed instead (their
  disarm, #145, would take them). {iD 0, the taker as the subject, data -1}, as FUN_0011c170 posts
  it (0x11c3b7-0x11c3db) and the grab_weapon event.
* **AI events** (RESERVE_4EVT, 17): a character's EVT_ events (its health under 75, 50 and 25
  percent of its most; EVT_SOUND_ENEMY_NOISE as a stub) to the event-triggers listening to it
  only (FUN_001fe440), with the character as the subject (#224, play_hints.rs `ai_events`).
