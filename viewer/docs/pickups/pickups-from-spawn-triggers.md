*Part of [pickups/](README.md), the former page `pickups.md`.*

## Pickups from spawn-triggers (#278)

`play_pickup_spawns.rs` (the copies), `play_triggers.rs` (`spawn_copy`, the subject relay), from
default.xbe's code (the addresses below) and checked against #35 take20 (sdm_e34's STAMINA POWER)
and #129 take38 (mp1's walkway).

* **What they are.** Every pickup the multiplayer maps place is a spawn-trigger's template
  (sdm_e34 54 of 54, mp1 29 of 29 ... sdm_e40 66), and so are the campaign's supply drops and
  caches (e34's Therm Sweeper, medkits and ammo after a radio line; m02_a's 15, m04_c's 20 ...).
  A template is out of the world (FUN_000dfe90): it isn't drawn, taken or counted where it's
  placed, and its idle effect doesn't start there. Its model is kept hidden
  (`level_scene::SpawnTemplate`) to draw the copies with. The level's other pickups (the
  tutorial's all) are as before.
* **The copies.** Each spawn of the trigger (on the multiplayer maps: its FIRST_UPDATE, then each
  TIMER_PULSE, with OPERATE and SPAWN_RESET; [triggers.md](../triggers/spawn-triggers.md), "Spawn-triggers") puts a
  copy of the template at the trigger's transform (FUN_00209620; an item's place call is the game
  object's 0x159b70, with no ray to the ground), named by the trigger (a re-armable one's get
  made-up names, as the game gives them none):
  * a type without a collision body (`collision-type` CT_NONE, the enum 0x3bd79c: the four
    power-ups' orb h_e9e61b85) stays at the transform with its idle effect: sdm_e34's STAMINA
    POWER at (-84.07, 29.78, 60.58), where take20's live object lies, 2 m from its placement
    (-82.11, 32.06, 60.06). Walking over one takes it ("Power-ups" below, #15);
  * another item (CT_CONVEX) lies on the ground under the transform, upright and tilted to the
    ground, loose, as the level's own pickups do ("Pickups are loose" above); medkits and fruit
    can be taken (no level has a DNA canister as a template). The multiplayer maps' triggers stand
    ~0.5 m over the ground, so the copy is the body the game's physics drops there, at rest (its
    fall isn't drawn);
  * a weapon lies on its side on the ground under the transform, turned the trigger's way about
    the vertical (take38: the guns flat on the walkway under their green spawn glows; where the
    game's physics rests it isn't read), a lying gun with its type's full clip ("Weapon pickups"
    above: the prompt, the swap, the walk-over).
* **Taking one** removes it for good (FUN_0011c170 removes an item it takes; the grab the gun) and
  sends PICKUP_OBJECT from it {iD 0, the taker, data -1}. A gun copy walked over keeps lying with
  what's left (its 10 s vanish timer outside a mission, above).
* **The respawn** is the data's ([triggers.md](../triggers/subject-relays.md), "Subject relays"): the trigger's
  OPERATED goes through a subject relay that binds the copy to a timer, the PICKUP_OBJECT starts
  the timer, and its pulse makes the next copy and, beside most, the spawn flash (an effect
  object of type h_117c1805). sdm_e34: the STAMINA POWER 30 s (h_0021e47f; take20: back ~31 s
  after it was taken, fading in over ~0.3 s), its medkits 10 or 45 s, its guns 20 or 45 s, its
  ammo and grenades 10 or 20 s, BRUTE POWER 180 s, FORCE POWER 165 s, HEALTH POWER 30 s.
* **Not done:** the copy's fall and its fade-in (take20's ~0.3 s; the spawn-trigger's h_eeae96e7
  is 2 s, the demo draws no fade; take01 of #18 saw the Particle box appear whole in one frame,
  then the green rings of the spawn flash for ~0.6 s); where
  the game's physics rests a gun copy (lying on its side is take38's look); the multiplayer maps'
  75 grenade pickups, item spawns `<h_1d1d4f05>` (types h_ecfe0d8c and h_15f9c51a, object-type
  24): their spawn-triggers fire and their relays bind, but `spawn_copy` copies inventory-objects
  and weapon-objects only, so none shows (none did before either: #230 hid them as templates).
* **Test hooks:** `BF_PICKUP_LOG=1` (or `BF_TRIGGER_LOG`, `BF_WEAPON_LOG`) prints each copy made
  (its trigger, name, template, type, place and kind), the item copies listed and each taken.
* **Verified** ([triggers.md](../triggers/verified.md) "Verified" has the commands): sdm_e34's 54 copies at
  the start, the STAMINA POWER's ahead left of Tex from take20's spot; three medkits taken and
  back 10.00 s later with the flash; a Saryl-45 swapped and back 20 s after the grab, the dropped
  LZR-23 gone 20 s after the release; mp1's walkway against take38's player 2 view (the
  L-Shot-75 on its side at Flint's feet, the medkits 14 m on); the tutorial and e34's own pickups
  as before.
