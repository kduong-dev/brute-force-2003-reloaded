*Part of [triggers/](README.md), the former page `triggers.md`.*

## Detectors

* **Volume.** A plate is a box: its `size` is the box's half-extents in the trigger's frame. The
  game tests a physics phantom, the Ipion surface named by the plate's `h_051ac63e`
  (FUN_001f99a0), and those surfaces are exactly that box: the tutorial's base plate
  h_eefcb023 (`size` 30 10 4) is a surface spanning +-30, +-10, +-4; the building plates' (1.5 2 2)
  +-1.5, +-2, +-2 (measured from objects-tutorial.ipn). Its fallback test without a phantom
  (0x1f9e00) also takes `size` as half-extents. A proximity detector is a sphere of `radius`
  (its phantom FUN_001fa330). The character is the demo's upright cylinder (BODY_RADIUS 0.4,
  BODY_HEIGHT 1.9), not its Ipion body: a plate counts it when a point along its axis is within
  the box grown by its radius, a sphere when its axis comes within radius + 0.4.
* **Filter** (`<filter>`, FUN_001fb980; every part must pass, `h_ebfd2b20` turns each over):
  `h_19656b35` a character of a squad type (its type's `h_e78efbb2`: Tex and the three Confed
  marine types in the tutorial, not the Red Hand), `h_1d083f6a` one of any other type,
  `h_115c7b01` any character (every sdm_e34 door's), `h_e707ecc1` a character with a human
  player (the one the player controls), `alignment` its team (-1 any), `object` named objects,
  `object-type` types (`tex`). The detectors see the living characters: the player, the squad,
  the NPCs (the test tools' and those the triggers spawn).
* **Occupancy** (FUN_001f8db0). One in the volume becomes an occupant; after 0.1 s it sends
  ENTER_DETECTOR (and ENTER_DETECTOR_FIRST when no occupant had been counted in), with itself as
  the subject, and its own ENTER_DETECTOR_TO_SELF. One out of it for 0.2 s (in sum: the game
  never resets the time out) leaves: EXIT_DETECTOR and EXIT_DETECTOR_TO_SELF; the last one
  leaving: EMPTIED_DETECTOR. A disabled detector does nothing, unless it has `h_e25d95e3`: then
  it keeps its occupants silently and enables itself when one leaves (sdm_e34's two teleport
  plates).
