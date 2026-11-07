*Part of [grenades/](README.md), the former page `grenades.md`.*

* **Throwing** (thrown types): hold G to charge. The meter (the recordings, every grenade): a
  blue outlined bar at x 354.0-367.7, y 162.0-221.7 right of the crosshair, its fill
  x 355.3-365.7 growing linearly from y 218.7 to 164.3 as the charge grows, sRGB 255 160 53 at 0.65, a
  dark divider at y 179, the box clear inside (only the fill and the frame are drawn, 0xadac0).
  Its timer (the meter widget FUN_000ad8d0's +0x40) is set to 1 with the level every frame the
  charge is above 0 (0x134823) and counts down after, so the meter stays full 0.5 s from the
  last charging frame, thrown or not, then fades as clamp(2 x timer, 0, 1)^3 to nothing at 1 s
  (its update 0xad950; #162). It sits beside the crosshair point (FUN_000ab390), so in the
  scope it moves with the crosshair to the middle (corner (352, 208)). A thrown grenade has no crosshair of its own:
  the held gun's stays, unchanged, through the charge and the throw (#119: `todo/119` take01,
  the Therm Sweeper's crosshair in the same box on every frame of three full-charge Frag
  throws). h_0ef11ee8, the two-bracket texture the demo used to show here, is the Jax-iR's
  crosshair (its `reticule-prefix`), and no grenade definition has one. The count drops at
  the button (the recordings: one game frame after it). The stance's throw clip takes the grenade in hand at its reach
  event - its trail (grenade_trail's smoke, riding on it) and the trail effect type's sound
  (the Frag's hiss, ~0.33 s after the button in the recordings, input to sound: ~0.27-0.31 s
  less the rig's input delay, #140) start then - and lets go at its release event (0.60 s after
  the button for Tex, as recorded: if that was the picture, the game's own release is ~0.40-0.46
  s, the picture lagging 0.12-0.15 s and the input 0.02-0.06; which one the recording timed is
  open, to be re-measured from a marker-synced take by the hiss onset, #140).
  * **The charge** (#161; the player controller 0x1332a0): while G is held on a grenade (an item
    of group 2, IG_GRENADE) the charge grows 1.6 a second (0x3a5088, 0x1347cf-0x1347e0), held at
    1.0 (FUN_000abf30): full in 0.625 s (`CHARGE_TIME`; #119 take01 0.600-0.617 s, the demo's
    0.6 before was fitted to it). At G's release the throw's power is clamp((charge - 0.3) / 0.7,
    0.01, 1) (0x3a50b0, the double 0x3a54c8; 0x1348fd-0x134941): a hold under 0.19 s throws as a
    tap, the meter's bottom 30% no farther (`grenade::throw_power`). The release is the item use
    (FUN_000abf60 -> FUN_00121170), which leaves the scope first (FUN_001249b0), with its sound
    and its button's lockout. Every other kind of item is used at the press, with no charge.
  * **The launch** (FUN_00117660 at 0x117be4, the throw clip's release event): the grenade is let
    go, moved 0.4 m along (0, sin 15, -cos 15) in the world's axes (0x3ecccccd; a quirk: the
    offset isn't turned by the aim) and pushed at the hand with J = 16 + 7 p^2 (FUN_00122ea0) along
    the aim frame x (0, sin 15, -cos 15): 15 degrees above the aim, whatever the charge
    (0x3e84816f, 0xbf774539). The demo's aim frame is the camera's turn (`camera::aim_turn`: its
    yaw and `cam_pitch`), not the crosshair's ray, which is 6 degrees above the view's axis: the
    throw leaves 9 degrees above the crosshair. Ipion's push (FUN_002b3bc0) changes its speed by
    J over the body's mass, the item type's `mass` (h_1a19ff7a, `WeaponDef::mass`: the Frag and
    the Light 0.9, the Energy, the Gas and the Sonic 1.0): a Frag at 17.8 m/s for a tap to 25.6
    m/s full, an Energy 16.0-23.0. It flies under gravity 9.81 (FUN_0016a250's Ipion world). Not
    done: Ipion's speed damping, read as 0.1 in the body template (FUN_00158f80: -10% of the speed
    a second; an inference, #161's leads: take01 fits with or without it, 25.1 against 26.3 m); the
    off-centre push's spin. The thrower's own speed along the ground is added (`carry`), the
    demo's: the game's release gives the body only its push, and whether it already moves with
    the hand isn't traced (#161's shot D would show it). Before #161 the demo threw at 12-20 m/s,
    0.25 rad below the crosshair's ray (fitted to a recording looking down), so a level throw
    landed ~5 m ahead where the game's goes ~26 m.
  * Checked (`-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=0,30,0,60 BF_TEST_GO_AT=100
    BF_GRENADE_LOG=1 BF_CAPTURE_FPS=60`, facing away from the rack): at `BF_CAMERA_PITCH=-0.157`
    (#119 take01's -9 degrees) a full Frag (`BF_TEST_THROW=2.0,1.0`) leaves the hand 2.13 m over
    the floor at 25.6 m/s, 6.0 degrees up, and comes down 24.9 m out 0.95 s later (#161: 24-27 m,
    1.0-1.1 s from take01's 2.5 m hand; the demo's hand is lower); a tap (hold 0.1) 15.7 m at
    17.8 m/s (15-17 m); a 0.4 s hold 17.6 m at 19.6 m/s (the leads' 17.9-18.6 m from 2.5 m); a full
    Energy (`BF_TEST_NEXT_GRENADE=0.6,0.9,1.2`) 21.7 m at 23.0 m/s (21-23 m). At -0.52 rad
    (-30 degrees) a full Frag 6.9 m in 0.25 s (7-8 m from 2.5 m; the world-axis nudge takes 0.39
    m off facing +z, as here). On e40 at take01's spot (`-- --test BF_MAP=e40 BF_CHARACTER=3
    BF_TEST_GOTO=-1.97,169.30,-25.63,158.34 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.157
    BF_TEST_THROW=2.0,2.0`) a 2 s charge comes down 25.5 m out 0.93 s after leaving the hand, at
    the foot of the crosshair (take01: its three Frags first touch the sand 26.1 m out, at the
    bottom of the crosshair box). Thrown while scoped (Flint's L-Shot-75,
    `BF_TEST_SCOPE_PRESS=1.0 BF_TEST_THROW=2.0,0.5 BF_SCOPE_LOG=1`), the scope is left at G's
    release (2.53 s: level 0, its lockout 0.83 s), and the throw goes at 21.7 m/s (p 0.71).
  * The recipes elsewhere on this page and in testtools.md, skills.md and levels.md that throw
    with `BF_TEST_THROW` were run with the throw before #161: the same recipe now throws faster
    and higher and its grenade lands farther (a tap at `BF_CAMERA_PITCH=-0.35` ~10 m out where it
    came down ~3 m), so their distances and who stood in a blast don't hold as written; they
    weren't run again. To land one close, aim lower (-0.6 rad: a tap ~5 m, by the launch above) or stand nearer.
  * The level's invisible blockers and hinters don't stop it, nor hold it up: it flies, rolls and
    comes down through them onto what's beyond (#298; Ipion's pair table keeps them off every
    body but a character's: maps.md, "Blockers and hinters"). Before, one that ended a frame at a
    blocker's wall bounced back off it, and one could come to rest on a blocker's top.
  * It keeps 0.3 of its speed along the ground per bounce, a value fitted to the Frag recording
    (resting near where it came down), not from the data: the game's is its physics material's,
    not traced (#161 leaves the bounces and the roll out). Bounces are
  silent; the trail keeps puffing where it lies until the blast.
* **Placing** (Roller, Sentry): the press drops the count and plays the event sound; the
  stance's `Sc_w1/w2_place_hi` clip takes it in hand at its reach event (0.23-0.47 s over the
  squad's clips: the recordings' "in hand 0.38-0.47 s", input to picture, so ~0.2-0.3 s in the
  game's time, #140) and lets go at its 19f8311b event (0.50-0.60 s); it drops from the hand at
  the feet (Tex's Roller is down 0.97 s after the press; recorded ~1.05 s input to picture, ~0.85-0.9
  s in the game's time: so the demo's, from the clip's events, is probably 0.1-0.15 s late, not
  0.08 s early; to be re-checked from a marker-synced take, #140). Squadmates don't dive from one set down. The Roller then
  hops off after enemies (see **Roller**), its object sound h_19dbc65d again every 0.97 s once
  it's down (the recording's loop), full within 5 m and silent at 25 m (the sound's own
  `falloff 5` / `h_fd40e332 25`, sounds-mp1.xml); the fuse (25 s; the Sentry's 9999 s; the
  Thermo's 15 s) runs from when it's down. The take's Tex holds the Roller low in his right hand
  and lets it go by his right foot, 0.72 m ahead along the aim and 1.29 m up (#79 take10 at
  2.7-3.15 s, the spec's §4); the demo's place drops it from the throw clip's hand bone, by his
  left side, 1.16 m up and 0.6 m from him (the shared place, not changed by #79).
* **The fuse** (the grenade update 0x149420, #79 / #82): it counts, and ends in the blast, only
  once the grenade is out of the hand, armed by its first contact (the contact handler 0x149510:
  +0x1dc bit 1; a grenade set down live by the level is armed as it settles at the load), and
  while no cutscene holds the world: the world's cutscene bit (+0xc50 bit 4, a script's 122 sets
  it and its 123 clears it, play_script.rs `in_cutscene`) returns before the subtraction and the
  test, so a fuse at 0 waits too (0x1494be). The demo had no cutscene test before. A Sentry's own
  check, a shot on a mine and a blast setting one off go off at once whatever the bit
  (`Grenade::set_off`: 0x147620 calls the grenade's +0x15c itself).
* **Tab** (the game's B) steps to the next item carried, every category in the store's order,
  and drops a charge in progress, G then dead until it's let go (#196: [play.md](../play/item-box-and-item-list.md), **The
  item box and the item list**; the T key that stepped the grenade types is gone, the game has
  no such button). A throw cut short (knocked down, or control handed over, before the grenade
  leaves the hand) puts the grenade back.
