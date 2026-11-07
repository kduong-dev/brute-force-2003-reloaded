*Part of [brain/](README.md), the former page `brain.md`.*

## Aim

* **The AI's aim** (FUN_00024f80 -> FUN_00024c00, with FUN_00024ae0): each time it fires at a
  character the gun is given the target's **hips**, its root node's place (the vtable +0x40
  position, 0x25391: about 1 m over its feet, `missile::root_point`; the demo aimed at a chest
  1.1 m up before #244) with the probability of its accuracy, else a point off it by `up`
  turned about the line of fire (hips to hips) by up to 45 degrees either way, 1 m long (1.2 m when it points up
  more than 0.7, as it always does for a level shot: a miss goes over the head, and never
  below the hips), shrunk by the
  distance under 10 m (x d / 10). The accuracy is the placement's h_072837bb when its h_ec5337fb
  is set, else the type's h_12a04758 (percent), then + 0.05 x the difficulty (halved above 0),
  held within a half and one and a half of itself, at most 1: the Red Hand's 25 is 0.125 on
  STANDARD (-4), the sniper template's (h_ec5337fb false) type 50 is 0.3. A character of a squad
  a player controls skips the difficulty step (0x8d9f0 counts the players on its squad): the
  marines' 60 is 0.6. (FUN_00024c00 also never misses for a character whose skill's vtable +0x110
  says so, which also skips the raise below; not read here. With brain+0x3c4 bit 13, which
  FUN_00017f80 sets at the brain's set-up for a gun whose round falls (the iKhan-GPL's, the
  campaign hounds' balls) and only for a brain whose bit 8 is clear, so never for a squad type's
  (h_e78efbb2: the squad, the allies, every multiplayer type), it raises the aim point straight up
  by 0.5 x (1 + 13 x (min(d, 40) / 40)^2) m whether it missed or not; and FUN_00024f80 refuses
  within 1.5 x the gun's Damage radius, for every brain: both done since #83, in the shot builder
  and the trigger hold, [play.md](../play/missiles.md), "An AI's lobbed rounds".)
* **The gun's spread** takes the holder's bonus (FUN_00222b30: h_072837bb with h_ec5337fb, else
  -1): the Red Hand's +25 narrows the MK-ASLT's spread ([play.md](../play/shots.md), "Shots";
  play_shots.rs).
