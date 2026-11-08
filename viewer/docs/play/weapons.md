*Part of [play/](README.md), the former page `play.md`.*

Weapons (`src/bf/weapon.rs`): every character carries their real starting inventory
(objecttypes: Brutus Feral Cutter + Bower 20, Flint L-Shot-50 + Confed LZR-10, Hawk Foley 356
Tact + Confed LZR-23, Tex Confed LZR-23 + RVG50 Minigun; on a campaign zone each member's
placement's two instead, slot 1 in hand: see **Items: the squad's starting inventory**), with names from the English string
table, models from the level's weapon archetypes (24-byte static vertices; the minigun is a
compound whose barrel spins), and the definition's fire rate, fire sounds and range. A held gun
hangs from the trigger hand: the character's hand hardpoint lined up with the weapon's trigger
grip hardpoint (both point + w-x-y-z quaternion). That pairing makes every squad weapon point
straight ahead and level in the game's ready-pose clips, and puts the support hand on the
foregrip in the carry clips. Locomotion follows the weapon's stance. Each character has two motion
sets, `Sc_w1_*` and `Sc_w2_*` (idle, walk, run, dash, backpedal, reload, throw), and the game
picks one every frame from the gun in hand (char+0x5b0, FUN_0011aa20) by the character type's
pickup mode (`<limit-weapon h_fb08d4c6>`, the one that also chooses a picked-up gun's slot):

| Character | pickup mode | the set |
|---|---|---|
| Flint, Hawk | 1 | the slot's: slot 1 `Sc_w1_*`, slot 2 `Sc_w2_*` |
| Brutus, Tex | 2, 3 | the gun's class (its `weapon-type`, WTYPE 0 pistol, 1 rifle, 2 cannon): `Sc_w2_*` for a cannon, `Sc_w1_*` for any other |
| any other type | 0, 4 | `Sc_w1_*` |

So Brutus holds his rifle-class Bower 20 in his rifle stance, like the Feral Cutter, though it's
his second weapon, and a rifle Tex picks up into his cannon's slot is held as a rifle (#297: the
demo had keyed Tex's by the slot, from his `limit-weapon` slots, both "2 1"). Arms then carry each gun
the way the game animates it (Tex's minigun low in both hands, Flint's LZR-10 one-handed), and
aiming or firing switches to that set's `rp_*` ready-pose clips. The barrel is then turned onto the crosshair (yaw and pitch, two
passes, over the spine bones that carry the arm), so shots leave along the barrel (within 1
degree). The other weapon is stowed: the weapon's holster hardpoint (`h_10ab40ac`, on every gun) on
one of the character's three stow hardpoints (`weapon::STOW`), the one its slot's holder is on
(its place, `StowPlaces`, #297: the holders' +0x40 through their hardpoint getter 0x14ba70). Slot 1
starts on 0xff4c691b and slot 2 on 0xf4ccf78c, which hang Brutus's first gun from his pelvis
and his second diagonally across his back (on Bip01 Spine2), Flint's and Hawk's rifles across
their backs and their sidearms barrel-down on the hip (Bip01 R Thigh); only Tex's places move. His draw swaps them (FUN_00123fa0) and his switch clips' events move
them, so his stowed gun hangs vertical on his left ramp (0xff4c691b) whichever slot it's from, as
the takes have it (#61 take04: the LZR-50 of his slot 2; #68 take02: the LZR-23, then the
Minigun after Y), where the demo had hung slot 2's on his right ramp. Switching plays the character's own weapon-change overlay from its motion script (Flint and
Hawk `Sc_w1_2_w2` / `Sc_w2_2_w1`, Brutus `Sc_w1_2_w2rifle` / `Sc_w2rifle_2_w1`, Tex
`Sc_wc_rifle2cannon` / `Sc_wc_cannon2rifle`, whose ramps swing the gun over his shoulder) on the
upper body over whatever the legs are doing. The clips carry timed events (an event channel of
f32 time + name hash per event): at `drop_weapon` the held gun goes to its stow point, at the
grab event the other one comes to the hand, each with the weapon's handling (`pickup-sound`)
sound; no firing or aiming until the clip ends. Tex's clips swing his two ramps across his back
(the left one up to his right shoulder with the gun to draw, 0.50-0.83 s; the right one out and
over to his left side, 0.75-1.08 s; both back to rest between 1.25 and 1.33 s) and carry two more
events: the gun let go goes on his right ramp at `drop_weapon` and rides it over, ed585109
(1.20 s) hands it to Spine2's 0xe3307376, which sits where the left ramp rests, for the ramps'
snap back, and f18780b8 (1.37-1.40 s) onto the left ramp (FUN_0014d500, FUN_0014d570). Shots: muzzle flash and light,
recoil, tracers that fly to what the crosshair is on (ground or pillars) and a spark there.

A switch cut short (a knock-down, or an enemy's hit reaction) between its `drop_weapon` and its grab gives the gun let go back to the hand (`Player::cancel_switch`, #409); before, both guns stayed stowed until the next switch. That is the demo's choice: what the game does after a cut switch isn't traced, as for a cut throw ([throwing-and-placing.md](../grenades/throwing-and-placing.md)).

**Kingman's left hip (demo-own, #409, `src/bin/play_holster.rs`).** The demo's Edward Kingman class carries Tex's
Berserker and a second Kman Auto (#407), and his model has none of the stow hardpoints (nor Tex's SECOND_HAND), so
a gun that wasn't in his hand was hidden. The type that has the Berserker on such a model gets a holster of the
demo's own on his **left hip**, the mirror of Flint's pistol holster (h_f4ccf78c on `Bip01 R Thigh`) scaled to his longer
thigh and moved 0.10 m back, which **whichever pistol isn't in his right hand** hangs from: both of the places the guns'
slots start on (0 and 1) are it, so the second gun always comes from the nearer hip and the right hip stays empty. His
switch (Q) has no clip, so the demo gives it a timed one with the same machinery: the drawn gun goes to the left hip and the
other comes out of it into his right hand (the first across the front of him, the second round his back, so they never meet),
each in a quarter of a second, nothing aiming or firing for 0.35 s; and the berserk draw and stow move the second gun between his
left hip and his left hand the same way. Every move of a gun between a hand and a hip is a glide of its model through the
model's space, so none pops in or out. The full account, with the left arm's mirror, is in
[skills/other-characters-skills.md](../skills/other-characters-skills.md) ("Demo-own skills", "Kingman's left hip and the swap").
Tex's ramps, Brutus's, Flint's and Hawk's places are the game's as above and don't glide.

`BF_WEAPON_LOG=1` (pickups.md) prints, whenever it changes, where each gun hangs (in the hand, the
second hand, or stowed at which hardpoint and bone) with its class and stance, and (#409) each glide of
Kingman's guns between a hand and a hip. Checked (#297) on
the test map (`--test`), with it:

* #61 take04's recreation (pickups.md, with `BF_TEST_SWITCH=10.0,13.0`): Tex's LZR-50 (slot 2)
  hangs on his left ramp before the swap and after it, as at take04's 1.0 s and 9.0 s (it had hung
  on his right one); after each Q the stowed gun is there again (the L-Shot-50 at 11.47 s, the
  LZR-50 at 14.47 s), having gone right ramp, Spine2, left ramp at the clip's drop, 1.20 and
  1.40 s;
* #68 take02's switch (`BF_TEST_WEAPON="Confed LZR-23;RVG50 Minigun" BF_START_WEAPON=1
  BF_TEST_GOTO=0,0,0,-1.5 BF_TEST_SWITCH=2.0,5.0`, every frame): the Minigun put on the right
  ramp at the drop (2.53 s) and carried over to the left as the take's (from 22.15 s on the right,
  22.55 s on the left; the demo's clip runs at its data's rate, slower than the take's: see
  **The crosshair** below);
* Tex swapping his Minigun for an MK-ASLT (`BF_TEST_GUN="MK-ASLT,3.6,0.3"`, the same weapons; the
  swap alone runs with `--test BF_TEST_WEAPON="Confed LZR-23;RVG50 Minigun" BF_START_WEAPON=1
  BF_TEST_GOTO=0,0,0,-1.8 BF_TEST_WALK=1 BF_TEST_GUN="MK-ASLT,3.0,0.0" BF_TEST_SWAP=5`: the gun
  is his target from 1.53 s and the pickup starts at 5.33 s; at a run, without `BF_TEST_WALK`,
  he loses the gun and never swaps): the
  rifle goes to slot 2 (`Sc_w_drop_cannon_pickup_rifle`) and is held in `Sc_w1_*` (it had been
  `Sc_w2_*`: lower in the hands, as the Minigun; side view with `BF_VIEW_YAW=1.4
  BF_TEST_TRIGGER=4.0-5.0`, the carry and the ready pose against the old build's);
* Brutus (#52's recreation: the LZR-50 to slot 2 after the switch), Flint (an LZR-10 to slot 2)
  and Hawk (a Saryl-23 to slot 1), each with two Qs after: their guns' hardpoints and stances as
  before (each slot on its own, Flint's and Hawk's by the slot, Brutus's by the class).
